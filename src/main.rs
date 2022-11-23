use tokio::spawn;
use crate::module::monitor::{Monitor, P2pMonitor, RpcMonitor};
use crate::module::p2p_client::BeaconP2pClient;

mod common;
mod module;

#[tokio::main]
async fn main() {
    // let monitors = vec![
    //     Monitor::Rpc(RpcMonitor::new())
    // ];
    //
    // for monitor in monitors {
    //     spawn(async move{
    //        monitor.run().await;
    //     });
    // }

    let mut bpc = BeaconP2pClient::new("light_client_optimistic_update").await.unwrap();
    bpc.run().await;
}
