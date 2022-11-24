use crate::module::rpc_client::RpcClient;
use crate::module::utils::{
    all_zero_hash_list, compute_period_at_slot, get_subtree_index, is_valid_merkle_branch,
    verify_signature,
};
use crate::module::{Message, UpdateMessage};
use lighthouse_ssz::Encode;
use lighthouse_types::light_client_optimistic_update::LightClientOptimisticUpdate;
use lighthouse_types::light_client_update::{
    LightClientUpdate, FINALIZED_ROOT_INDEX, NEXT_SYNC_COMMITTEE_INDEX,
};
use lighthouse_types::{
    BeaconBlockHeader, ChainSpec, EthSpec, Hash256, SyncAggregate, SyncCommittee,
};
use ruc::*;
use std::ops::Deref;
use std::sync::Arc;
use tokio::sync::mpsc::UnboundedReceiver;

pub struct LightClientStore<T: EthSpec> {
    pub snapshot: LightClientSnapShot<T>,
    // TODO: The way this field is used feels redundant,
    //  In a function, first add and finally clear, there is a lookup operation in between
    pub valid_updates: Vec<UpdateMessage<T>>,
    pub rpc_client: Arc<RpcClient>,
    pub spec: ChainSpec,
    pub receiver: UnboundedReceiver<Message<T>>,
}

pub struct LightClientSnapShot<T: EthSpec> {
    pub header: BeaconBlockHeader,
    pub current_sync_committee: SyncCommittee<T>,
    pub current_sync_committee_branch: Vec<Hash256>,
    pub next_sync_committee: SyncAggregate<T>,
}

impl<T: EthSpec> LightClientStore<T> {
    pub async fn process(&mut self, update_message: UpdateMessage<T>) -> Result<()> {
        let (vote_sum, sync_committee_sum, update_slot) = match &update_message {
            UpdateMessage::FinalityUpdate(update) => {
                self.snapshot
                    .finality_validate(update, self.rpc_client.clone(), &self.spec)
                    .await
                    .c(d!())?;
                (
                    update.sync_aggregate.sync_committee_bits.num_set_bits(),
                    update.sync_aggregate.sync_committee_bits.len(),
                    update.finalized_header.slot.as_u64(),
                )
            }
            UpdateMessage::OptimisticUpdate(update) => {
                self.snapshot
                    .optimistic_validate(update, self.rpc_client.clone(), &self.spec)
                    .await
                    .c(d!())?;
                (
                    update.sync_aggregate.sync_committee_bits.num_set_bits(),
                    update.sync_aggregate.sync_committee_bits.len(),
                    update.attested_header.slot.as_u64(),
                )
            }
        };

        self.valid_updates.push(update_message.clone());

        // MainnetEthSpec = 32 * 256
        // GnosisEthSpec = 16 * 512
        // MinimalEthSpec = 8 * 8
        let update_time =
            T::slots_per_epoch() * self.spec.epochs_per_sync_committee_period.as_u64();

        // vote/all >= 2/3
        if vote_sum * 3 >= sync_committee_sum * 2 {
            self.apply(update_message).await.c(d!())?;
        } else if update_slot > self.snapshot.header.slot.as_u64() + update_time {
            // Long time no update

            // After sorting in desc order, take the first
            let mut sum_bits = usize::MIN;
            let mut max_bit_update_from_valid_updates = None;
            for valid_update in &self.valid_updates {
                let tmp_sum_bits = valid_update.num_set_bits();
                if tmp_sum_bits > sum_bits {
                    sum_bits = tmp_sum_bits;
                    max_bit_update_from_valid_updates.replace(valid_update.clone());
                }
            }

            self.apply(max_bit_update_from_valid_updates.unwrap())
                .await
                .c(d!())?;
        }

        self.valid_updates.clear();

        Ok(())
    }

    pub async fn apply(&mut self, update_message: UpdateMessage<T>) -> Result<()> {
        Ok(())
    }

    pub async fn run(mut self) -> Result<()> {
        // 1. loop receive
        loop {
            if let Some(msg) = self.receiver.recv().await {
                match msg {
                    Message::Bootstrap(data) => {
                        self.snapshot.current_sync_committee =
                            data.current_sync_committee.deref().clone();
                        self.snapshot.current_sync_committee_branch =
                            data.current_sync_committee_branch.to_vec().clone();
                    }
                    Message::Update(data) => {}
                }
            }
        }
    }
}

impl<T: EthSpec> LightClientSnapShot<T> {
    pub async fn optimistic_validate(
        &self,
        update: &LightClientOptimisticUpdate<T>,
        rpc_client: Arc<RpcClient>,
        spec: &ChainSpec,
    ) -> Result<()> {
        // 1. Verify update slot is larger than snapshot slot
        if update.attested_header.slot > self.header.slot {
            return Err(eg!(""));
        }

        // 2. Verify update does not skip a sync committee period
        let snapshot_period =
            compute_period_at_slot::<T>(self.header.slot.as_u64(), rpc_client.clone(), spec)
                .await?;
        let update_period = compute_period_at_slot::<T>(
            update.attested_header.slot.as_u64(),
            rpc_client.clone(),
            spec,
        )
        .await?;
        if update_period > snapshot_period + 1 {
            return Err(eg!(""));
        }

        // 3.
        // TODO: There is some ambiguity here, when finality=default is entered into the verification,
        //  that is, the verification is not the last slot of epoch,
        //  but the documentation requires the use of update.finality_branch
        //  Document address:  https://github.com/ethereum/annotated-spec/blob/master/altair/sync-protocol.md
        //  `assert update.finality_branch == [Bytes32() for _ in range(floorlog2(FINALIZED_ROOT_INDEX))]`
        let _data = all_zero_hash_list(FINALIZED_ROOT_INDEX);

        // 4.
        // TODO: update.next_sync_committee_branch
        //  `assert update.next_sync_committee_branch == [Bytes32() for _ in range(floorlog2(NEXT_SYNC_COMMITTEE_INDEX))]`
        let _data = all_zero_hash_list(NEXT_SYNC_COMMITTEE_INDEX);

        // 5. Verify sync committee has sufficient participants
        if update.sync_aggregate.sync_committee_bits.num_set_bits()
            < spec.min_sync_committee_participants as usize
        {
            return Err(eg!(""));
        }

        // 6. Verify sync committee aggregate signature
        if !verify_signature(
            &update.sync_aggregate,
            &self.current_sync_committee,
            spec,
            &update.attested_header,
        )
        .c(d!())?
        {
            return Err(eg!(""));
        }

        Ok(())
    }

    pub async fn finality_validate(
        &self,
        update: &LightClientUpdate<T>,
        rpc_client: Arc<RpcClient>,
        spec: &ChainSpec,
    ) -> Result<()> {
        // 1. Verify update slot is larger than snapshot slot
        if update.attested_header.slot > self.header.slot {
            return Err(eg!(""));
        }

        // 2. Verify update does not skip a sync committee period
        let snapshot_period =
            compute_period_at_slot::<T>(self.header.slot.as_u64(), rpc_client.clone(), spec)
                .await?;
        let update_period = compute_period_at_slot::<T>(
            update.attested_header.slot.as_u64(),
            rpc_client.clone(),
            spec,
        )
        .await?;
        if update_period > snapshot_period + 1 {
            return Err(eg!(""));
        }

        // 3. Verify update header root is the finalized root of the finality header
        if !is_valid_merkle_branch(
            update.attested_header.canonical_root(),
            &update.finality_branch.to_vec(),
            FINALIZED_ROOT_INDEX.ilog2(),
            get_subtree_index(FINALIZED_ROOT_INDEX).c(d!())?,
            update.finalized_header.state_root,
        )
        .c(d!())?
        {
            return Err(eg!(""));
        }

        // 4. Verify update next sync committee if the update period incremented
        if !is_valid_merkle_branch(
            Hash256::from_slice(&update.next_sync_committee.as_ssz_bytes()),
            &update.next_sync_committee_branch.to_vec(),
            NEXT_SYNC_COMMITTEE_INDEX.ilog2(),
            get_subtree_index(NEXT_SYNC_COMMITTEE_INDEX).c(d!())?,
            update.attested_header.state_root,
        )
        .c(d!())?
        {
            return Err(eg!(""));
        }

        // 5. Verify sync committee has sufficient participants
        if update.sync_aggregate.sync_committee_bits.num_set_bits()
            < spec.min_sync_committee_participants as usize
        {
            return Err(eg!(""));
        }

        // 6. Verify sync committee aggregate signature
        if !verify_signature(
            &update.sync_aggregate,
            &self.current_sync_committee,
            spec,
            &update.finalized_header,
        )
        .c(d!())?
        {
            return Err(eg!(""));
        }

        Ok(())
    }
}
