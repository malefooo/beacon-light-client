use std::str::FromStr;
use std::sync::Arc;
/// Listening to network data
/// p2p: P2pMonitor, Responsible for getting new messages from p2p networks
/// rpc: RpcMonitor, Responsible for polling for new messages from the rpc interface of the node-wide node


use ruc::*;
use serde_json::Value;
use crate::module::rpc_client::RpcClient;
use async_trait::async_trait;
use lighthouse_types::{BeaconBlock, BeaconState, chain_spec, ChainSpec, Eth1Data, EthSpec, Hash256, MainnetEthSpec};
use lighthouse_types::light_client_update::LightClientUpdate;
use crate::module::p2p_client::BeaconP2pClient;

pub enum Monitor{
    Rpc(RpcMonitor),
    P2p(P2pMonitor),
}

impl Monitor {
    pub async fn run(self){
        match self {
            Monitor::Rpc(m) => {}
            Monitor::P2p(m) => {m.run().await;}
        }
    }
}


pub struct P2pMonitor{
    pub config: Value,
    pub client: Box<BeaconP2pClient>
}

impl P2pMonitor {
    pub async fn new() -> Result<Box<Self>> {
        Ok(Box::new(Self{ config: Default::default(), client: BeaconP2pClient::new("").await.c(d!())? }))
    }

    pub async fn run(self){

    }
}


pub struct RpcMonitor{
    pub config: Value,
    pub client: Arc<RpcClient>,
}

impl RpcMonitor {
    pub fn new() -> Self {
        Self{ config: Default::default(), client: Arc::new(RpcClient::new().unwrap()) }
    }

    pub async fn run(self, sender: UnboundedSender<LightClientUpdate<MainnetEthSpec>>){

    }

    pub async fn create_light_client_update() -> Result<LightClientUpdate<MainnetEthSpec>> {

        return Err(eg!())
    }
}

use lighthouse_types::BeaconBlockBodyBase;
use tokio::sync::mpsc::UnboundedSender;

#[test]
fn test1() {
    use lighthouse_types::chain_spec;
    let spec = chain_spec::ChainSpec::mainnet();
    let attested_state_eth1_data = Eth1Data {
        deposit_root: Hash256::from_str("0xfd2c872995a93197358bb2104503d9bed31a3e335ff6cbba0d983d4721a34ed6").unwrap(),
        deposit_count: 147263,
        block_hash: Hash256::from_str("0x9ad6e5f596ff5ff90eac432f9a50a4767d8baf1160136b6f761d3199a4f1835b").unwrap(),
    };

    let mut attested_state = BeaconState::<MainnetEthSpec>::new(1621467220, attested_state_eth1_data, &spec);

    let attested_block = reqwest::blocking::get("http://44.234.44.27:5052/eth/v2/beacon/blocks/1224544").unwrap().json::<Value>().unwrap();
    let temp = attested_block["data"]["message"].clone();
    let attested_block = serde_json::from_value::<BeaconBlock<MainnetEthSpec>>(temp).unwrap();

    let unattested_state_eth1_data = Eth1Data{
        deposit_root: Hash256::from_str("0xe2a9d899d9c7026df62eab7b39cf88bc74098361466695323ad396734128abfb").unwrap(),
        deposit_count: 147300,
        block_hash: Hash256::from_str("0xbe8b4b45c4f17a783ecdb355da18045e1dd190be851334b1c0d6df90745c938a").unwrap(),
    };
    let unattested_state = BeaconState::<MainnetEthSpec>::new(1621491763, unattested_state_eth1_data, &spec);

    let unattested_block = reqwest::blocking::get("http://44.234.44.27:5052/eth/v2/beacon/blocks/1226592").unwrap().json::<Value>().unwrap();
    let temp = unattested_block["data"]["message"].clone();
    let unattested_block = serde_json::from_value::<BeaconBlock<MainnetEthSpec>>(temp).unwrap();

    let update = LightClientUpdate::new(spec, unattested_state , unattested_block, &mut attested_state, attested_block);
    println!("{:?}", update);
}

pub fn eth2_genesis_time(eth1_timestamp: u64, spec: &ChainSpec) -> Result<u64> {
    eth1_timestamp.checked_add(spec.genesis_delay).ok_or(eg!(""))
}

