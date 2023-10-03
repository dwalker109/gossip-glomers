use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};

use maelstrom_rs::{Body, Id, Message, Node, Workload};
use serde::{Deserialize, Serialize};
use tokio::io::{stdin, stdout};
use tokio::sync::mpsc::Sender;

#[tokio::main]
async fn main() {
    let mut node = Node::startup(stdin(), stdout()).await.unwrap();
    let workload = BroadcastWorkload {
        messages: Arc::new(Mutex::new(HashSet::new())),
        topology: Arc::new(Mutex::new(HashMap::new())),
    };
    node.run(workload).await;
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "snake_case")]
#[serde(tag = "type")]
enum BroadcastBody {
    Broadcast {
        message: usize,
    },
    BroadcastOk,
    Read,
    ReadOk {
        messages: Arc<Mutex<HashSet<usize>>>,
    },
    Topology {
        topology: HashMap<Id<String>, Vec<Id<String>>>,
    },
    TopologyOk,
}

struct BroadcastWorkload {
    messages: Arc<Mutex<HashSet<usize>>>,
    topology: Arc<Mutex<HashMap<Id<String>, Vec<Id<String>>>>>,
}

impl Workload<BroadcastBody> for BroadcastWorkload {
    fn handle(&mut self, recv: Message<BroadcastBody>, tx: Sender<Message<BroadcastBody>>) {
        let messages = Arc::clone(&self.messages);
        let topology = Arc::clone(&self.topology);

        tokio::spawn(async move {
            let node_id = recv.dest().clone();
            let from = recv.src().clone();

            match recv.body().r#type() {
                BroadcastBody::Broadcast { message } => {
                    messages.lock().unwrap().insert(*message);

                    tx.send(recv.into_reply(BroadcastBody::BroadcastOk.into()))
                        .await
                        .ok();

                    let neighbours = topology.lock().unwrap().get(&node_id);
                    if let Some(neighbours) = neighbours {
                        for n in neighbours.iter().filter(|&n| *n != from) {
                            tx.send(Message::new(Id::Defer, n.clone(), recv.body().clone()))
                                .await
                                .ok();
                        }
                    }
                }
                BroadcastBody::BroadcastOk => unimplemented!(),
                BroadcastBody::Read => {
                    tx.send(
                        recv.into_reply(
                            BroadcastBody::ReadOk {
                                messages: Arc::clone(&messages),
                            }
                            .into(),
                        ),
                    )
                    .await
                    .ok();
                }
                BroadcastBody::ReadOk { messages } => todo!(),
                BroadcastBody::Topology {
                    topology: new_topology,
                } => {
                    *topology.lock().unwrap() = new_topology.clone();
                    tx.send(recv.into_reply(BroadcastBody::TopologyOk.into()))
                        .await
                        .ok();
                }
                BroadcastBody::TopologyOk => todo!(),
            }
        });
    }
}
