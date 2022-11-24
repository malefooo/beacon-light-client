use lighthouse_types::light_client_bootstrap::LightClientBootstrap;
use lighthouse_types::light_client_optimistic_update::LightClientOptimisticUpdate;
use lighthouse_types::light_client_update::LightClientUpdate;
use lighthouse_types::EthSpec;

pub mod config;
pub mod monitor;
pub mod p2p_client;
pub mod rpc_client;
pub mod server;
pub mod utils;
pub mod validate;

#[derive(Debug, Clone)]
pub enum Message<T: EthSpec> {
    Bootstrap(LightClientBootstrap<T>),
    Update(UpdateMessage<T>),
}

#[derive(Debug, Clone)]
pub enum UpdateMessage<T: EthSpec> {
    FinalityUpdate(LightClientUpdate<T>),
    OptimisticUpdate(LightClientOptimisticUpdate<T>),
}

impl<T: EthSpec> UpdateMessage<T> {
    pub fn num_set_bits(&self) -> usize {
        match self {
            UpdateMessage::FinalityUpdate(update) => update.sync_aggregate.num_set_bits(),
            UpdateMessage::OptimisticUpdate(update) => update.sync_aggregate.num_set_bits(),
        }
    }
}
