use std::sync::Arc;
use lighthouse_types::{ChainSpec, EthSpec, MainnetEthSpec};
use ruc::*;
use crate::module::rpc_client::RpcClient;

pub async fn compute_period_at_slot<T: EthSpec>(slot: u64, rpc_client: Arc<RpcClient>, spec: &ChainSpec) -> Result<u64> {

    let block = rpc_client.beacon_get_block_by_slot::<T>(slot).await.c(d!())?;

    let period = block.slot().epoch(MainnetEthSpec::slots_per_epoch()).sync_committee_period(spec).map_err(|e|eg!("{:?}",e)).c(d!())?;

    Ok(period)

}

