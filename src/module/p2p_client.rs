use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::time::Duration;
use libp2p::{gossipsub, identity, PeerId, Swarm};
use libp2p::futures::{select, StreamExt};
use libp2p::futures::stream::SelectNextSome;
use libp2p::gossipsub::{Gossipsub, GossipsubEvent, GossipsubMessage, MessageAuthenticity, MessageId, Topic, TopicHash, ValidationMode};

use libp2p::mdns::{MdnsConfig, MdnsEvent, GenMdns};
use libp2p::swarm::SwarmEvent;
use libp2p::swarm::NetworkBehaviour;
use libp2p::NetworkBehaviour;
use tokio::io;
use tokio::io::AsyncBufReadExt;
use ruc::*;
use libp2p::mdns::TokioMdns;
use libp2p::gossipsub::Hasher as TopicHasher;

#[derive(NetworkBehaviour)]
pub struct MyBehaviour {
    pub gossipsub: Gossipsub,
    pub mdns: TokioMdns,
}

pub struct BeaconP2pClient{
    pub swarm: Swarm<MyBehaviour>,
    pub topic_hash: TopicHash,
}

#[derive(Debug, Clone)]
pub struct Topic1{}

impl TopicHasher for Topic1 {
    fn hash(topic_string: String) -> TopicHash {
        TopicHash::from_raw(topic_string)
    }
}


impl BeaconP2pClient {
    pub async fn new(topic: &str) -> Result<Box<Self>> {

        // Create a random PeerId
        let local_key = identity::Keypair::generate_ed25519();
        let local_peer_id = PeerId::from(local_key.public());
        println!("Local peer id: {local_peer_id}");

        // Set up an encrypted DNS-enabled TCP Transport over the Mplex protocol.
        let transport = libp2p::development_transport(local_key.clone()).await.c(d!())?;
        println!("1");
        // To content-address message, we can take the hash of message and use it as an ID.
        let message_id_fn = |message: &GossipsubMessage| {
            let mut s = DefaultHasher::new();
            message.data.hash(&mut s);
            MessageId::from(s.finish().to_string())
        };
        println!("2");
        // Set a custom gossipsub configuration
        let gossipsub_config = gossipsub::GossipsubConfigBuilder::default()
            .heartbeat_interval(Duration::from_secs(10)) // This is set to aid debugging by not cluttering the log space
            .validation_mode(ValidationMode::Strict) // This sets the kind of message validation. The default is Strict (enforce message signing)
            .message_id_fn(message_id_fn) // content-address messages. No two messages of the same content will be propagated.
            .build()
            .expect("Valid config");
        println!("3");
        // build a gossipsub network behaviour
        let mut gossipsub = Gossipsub::new(MessageAuthenticity::Signed(local_key), gossipsub_config)
            .expect("Correct configuration");
        println!("4");
        // Create a Gossipsub topic
        // let topic = Topic::new(topic.to_string());
        let topic: Topic<Topic1> = Topic::new(topic.to_string());
        println!("5");
        // subscribes to our topic
        gossipsub.subscribe(&topic).c(d!())?;
        println!("6");
        // Create a Swarm to manage peers and events
        let swarm = {
            let mdns = TokioMdns::new(MdnsConfig::default()).c(d!())?;
            let behaviour = MyBehaviour { gossipsub, mdns };
            Swarm::new(transport, behaviour, local_peer_id)
        };
        println!("7");
        Ok(Box::new(Self{ swarm, topic_hash: topic.hash() }))
    }

    pub async fn run(&mut self) {

        let mut stdin = io::BufReader::new(io::stdin()).lines();
        // stdin.next_line()
        println!("8");
        self.swarm.listen_on("/ip4/0.0.0.0/tcp/0".parse().unwrap()).unwrap();

        let r = self.swarm.select_next_some();

        // Kick it off
        loop {
            println!("9");
            tokio::select! {
                line = stdin.next_line() => {
                    println!("10");
                    let line = line.unwrap().expect("stdin closed");
                    self.swarm.behaviour_mut().gossipsub.publish(self.topic_hash.clone(), line.as_bytes());
                }
                event = self.swarm.select_next_some() => {
                    println!("{:?}",event);
                    match event {
                        SwarmEvent::Behaviour(MyBehaviourEvent::Mdns(MdnsEvent::Discovered(list))) => {
                            for (peer_id, _multiaddr) in list {
                                println!("mDNS discovered a new peer: {peer_id}");
                                self.swarm.behaviour_mut().gossipsub.add_explicit_peer(&peer_id);
                            }
                        },
                        SwarmEvent::Behaviour(MyBehaviourEvent::Mdns(MdnsEvent::Expired(list))) => {
                            for (peer_id, _multiaddr) in list {
                                println!("mDNS discover peer has expired: {peer_id}");
                                self.swarm.behaviour_mut().gossipsub.remove_explicit_peer(&peer_id);
                            }
                        },
                        SwarmEvent::Behaviour(MyBehaviourEvent::Gossipsub(GossipsubEvent::Message {
                            propagation_source: peer_id,
                            message_id: id,
                            message,
                        })) => println!(
                                "Got message: '{}' with id: {id} from peer: {peer_id}",
                                String::from_utf8_lossy(&message.data),
                            ),
                        _ => {println!("11");}
                    }
                }
            }
        }
    }
}

#[test]
fn test_beacon_p2p_client(){
    let rt = tokio::runtime::Runtime::new().unwrap();

    rt.block_on(async {
        let mut bpc = BeaconP2pClient::new("light_client_finality_update").await.unwrap();
        bpc.run().await;
    });
}