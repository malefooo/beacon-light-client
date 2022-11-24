use crate::module::rpc_client::RpcClient;
use crate::module::utils::{
    compute_period_at_slot, get_subtree_index, is_valid_merkle_branch, verify_signature,
};
use lighthouse_ssz::Encode;
use lighthouse_types::light_client_update::{
    LightClientUpdate, FINALIZED_ROOT_INDEX, NEXT_SYNC_COMMITTEE_INDEX,
};
use lighthouse_types::{
    BeaconBlockHeader, ChainSpec, EthSpec, Hash256, SyncAggregate, SyncCommittee,
};
use ruc::*;
use std::sync::Arc;

#[derive(Clone)]
pub struct LightClientStore<T: EthSpec> {
    pub snapshot: LightClientSnapShot<T>,
    pub valid_updates: Vec<LightClientUpdate<T>>,
    pub rpc_client: Arc<RpcClient>,
    pub spec: ChainSpec,
}

#[derive(Clone)]
pub struct LightClientSnapShot<T: EthSpec> {
    pub header: BeaconBlockHeader,
    pub current_sync_committee: SyncCommittee<T>,
    pub next_sync_committee: SyncAggregate<T>,
}

impl<T: EthSpec> LightClientStore<T> {
    pub async fn process(mut self, update: &LightClientUpdate<T>) -> Result<()> {
        Ok(())
    }

    pub async fn apply(mut self, update: &LightClientUpdate<T>) -> Result<()> {
        Ok(())
    }
}

impl<T: EthSpec> LightClientSnapShot<T> {
    pub async fn validate(
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
        let leaf = update.attested_header.canonical_root();
        let branch = update.finality_branch.to_vec();
        let depth = FINALIZED_ROOT_INDEX.ilog2();
        let index = get_subtree_index(FINALIZED_ROOT_INDEX).c(d!())?;
        let root = update.finalized_header.state_root;

        if !is_valid_merkle_branch(leaf, branch.as_slice(), depth, index, root).c(d!())? {
            return Err(eg!(""));
        }

        // 4. Verify update next sync committee if the update period incremented
        if update_period == snapshot_period {
            let mut data = vec![];
            let num = NEXT_SYNC_COMMITTEE_INDEX.ilog2();
            for _ in 0..num {
                data.push(Hash256::zero());
            }

            if !update.next_sync_committee_branch.to_vec().eq(&data) {
                println!("left : {:?}", update.next_sync_committee_branch);
                println!("right: {:?}", data);

                return Err(eg!(""));
            }
        } else {
            let leaf = Hash256::from_slice(&update.next_sync_committee.as_ssz_bytes());
            let branch = update.next_sync_committee_branch.to_vec();
            let depth = NEXT_SYNC_COMMITTEE_INDEX.ilog2();
            let index = get_subtree_index(NEXT_SYNC_COMMITTEE_INDEX).c(d!())?;
            let root = update.attested_header.state_root;

            if !is_valid_merkle_branch(leaf, branch.as_slice(), depth, index, root).c(d!())? {
                return Err(eg!(""));
            }
        }

        // 5. Verify sync committee has sufficient participants
        //      vote/all >= 2/3
        let vote_sum = update.sync_aggregate.sync_committee_bits.num_set_bits();
        let sync_committee_sum = update.sync_aggregate.sync_committee_bits.len();
        if vote_sum * 3 < sync_committee_sum * 2 {
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
