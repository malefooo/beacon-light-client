#![feature(int_log)]
#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(overflowing_literals)]
#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(unused_imports)]

extern crate core;

use crate::module::p2p_client::BeaconP2pClient;
use lighthouse_bls::Hash256;
use once_cell::sync::OnceCell;

mod common;
mod module;

static GENESIS_VALIDATORS_ROOT: OnceCell<Hash256> = OnceCell::new();

#[tokio::main]
async fn main() {
    let mut bpc = BeaconP2pClient::new("light_client_optimistic_update")
        .await
        .unwrap();
    bpc.run().await;
}
