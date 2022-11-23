use std::sync::Arc;
use lighthouse_types::light_client_update::LightClientUpdate;
use lighthouse_types::{BeaconBlockHeader, ChainSpec, EthSpec, MainnetEthSpec, SyncAggregate};
use serde::{Deserialize, Serialize};
use ruc::*;
use crate::module::rpc_client::RpcClient;
use crate::module::utils::compute_period_at_slot;

const GENESIS_VALIDATORS_ROOT:&str = "";


#[derive(Clone)]
pub struct LightClientStore<T: EthSpec>{
    pub snapshot: LightClientSnapShot<T>,
    pub valid_updates: Vec<LightClientUpdate<T>>,
    pub rpc_client: Arc<RpcClient>,
    pub spec: ChainSpec,
}

#[derive(Clone)]
pub struct LightClientSnapShot<T: EthSpec>{
    pub header: BeaconBlockHeader,
    pub current_sync_committee: SyncAggregate<T>,
    pub next_sync_committee: SyncAggregate<T>,
}

impl<T: EthSpec> LightClientStore<T> {

    pub async fn process(mut self, update: &LightClientUpdate<T>) -> Result<()>{
        Ok(())
    }

    pub async fn apply(mut self, update: &LightClientUpdate<T>) -> Result<()>{
        Ok(())
    }

}

impl<T: EthSpec> LightClientSnapShot<T>{

    pub async fn validate(&self, update: &LightClientUpdate<T>, rpc_client: Arc<RpcClient>, spec: &ChainSpec) -> Result<()>{

        // 1.
        if update.attested_header.slot > self.header.slot {
            return Err(eg!(""));
        }

        // 2.
        let snapshot_period = compute_period_at_slot::<T>(self.header.slot.as_u64(), rpc_client.clone(), spec).await?;
        let update_period = compute_period_at_slot::<T>(update.attested_header.slot.as_u64(), rpc_client.clone(), spec).await?;
        if update_period > snapshot_period + 1 {
            return Err(eg!(""));
        }

        // 3.


        // 4.
        // 5.
        // 6.

        Ok(())
    }


}