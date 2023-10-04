use std::collections::{HashMap, HashSet};
use std::iter::FilterMap;
use std::sync::{Arc, Mutex, RwLock};

use maelstrom_rs::{Body, Id, Message, Node, Workload};
use serde::{Deserialize, Serialize};
use tokio::io::{stdin, stdout};
use tokio::join;
use tokio::sync::mpsc::Sender;

#[tokio::main]
async fn main() {
    let mut node = Node::startup(stdin(), stdout()).await.unwrap();
    let workload = BroadcastWorkload {
        messages: Arc::new(Mutex::new(HashSet::new())),
        topology: Arc::new(RwLock::new(HashMap::new())),
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
    topology: Arc<RwLock<HashMap<Id<String>, Vec<Id<String>>>>>,
}

impl Workload<BroadcastBody> for BroadcastWorkload {
    fn handle(&mut self, recv: Message<BroadcastBody>, tx: Sender<Message<BroadcastBody>>) {
        let messages = Arc::clone(&self.messages);
        let topology = Arc::clone(&self.topology);

        tokio::spawn(async move {
            let self_node_id = recv.dest();
            let from_node_id = recv.src();

            match recv.body().r#type() {
                BroadcastBody::Broadcast { message } => {
                    messages.lock().unwrap().insert(*message);

                    tx.send(recv.clone().into_reply(BroadcastBody::BroadcastOk.into()))
                        .await
                        .ok();

                    let to_amplify = {
                        let neighbours = topology.read().unwrap();

                        match neighbours.get(self_node_id) {
                            None => Vec::with_capacity(0),
                            Some(n) => n
                                .iter()
                                .filter_map(|n| {
                                    (n != from_node_id).then(|| {
                                        Message::for_send(
                                            n.clone(),
                                            recv.body().r#type().to_owned().into(),
                                        )
                                    })
                                })
                                .collect::<Vec<_>>(),
                        }
                    };

                    for m in to_amplify {
                        tx.send(m).await.ok(); // Should join
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
                    *topology.write().unwrap() = new_topology.clone();
                    tx.send(recv.into_reply(BroadcastBody::TopologyOk.into()))
                        .await
                        .ok();
                }
                BroadcastBody::TopologyOk => todo!(),
            }
        });
    }
}
