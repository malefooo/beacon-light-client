use lighthouse_types::{BeaconBlock, EthSpec, Hash256, MainnetEthSpec};
use reqwest::Client;
use ruc::*;
use serde_json::Value;
use web3::transports::Http;
use web3::types::{Block, BlockId, H256, Transaction};
use web3::Web3;
use serde::{Deserialize, Serialize};

const API_PREFIX: &str = "eth";
const ACCEPT_HEADER: &'static str = "Accept";
const ACCEPT_HEADER_VALUE_JSON: &'static str = "application/json";
const ACCEPT_HEADER_VALUE_SSZ: &'static str = "application/octet-stream";

#[derive(Clone)]
pub struct RpcClient {
    pub beacon_http_client: Client,
    pub web3_client: Web3<Http>,
    pub base_url: String
}

impl RpcClient {
    pub fn new() -> Result<Self> {
        let transport = web3::transports::Http::new("addr").c(d!())?;
        let web3 = web3::Web3::new(transport);
        Ok(
            Self{
                beacon_http_client: Default::default(),
                base_url: "".to_string(),
                web3_client: web3,
            }
        )
    }

    pub async fn beacon_get_block_by_slot<T: EthSpec>(&self, slot: u64) -> Result<BeaconBlock<T>> {
        let url = format!("{}/{}/{}",self.base_url, "/v2/beacon/blocks", slot);
        let result = self.beacon_http_client
            .get(url)
            .send()
            .await
            .c(d!())?
            .json::<Value>()
            .await
            .c(d!())?;
        let msg = result["data"]["message"].clone();
        let block = serde_json::from_value::<BeaconBlock<T>>(msg).c(d!())?;
        Ok(block)
    }

    // pub async fn beacon_get_sync_committees_by_slot<T: EthSpec>(&self, slot: u64) -> Result<>

    pub async fn web3_get_block_by_hash(&self, hash: Hash256) -> Result<Option<Block<H256>>> {
        let web3_hash = H256{ 0: hash.0 };
        self.web3_client.eth().block(BlockId::Hash(web3_hash)).await.c(d!())
    }
}