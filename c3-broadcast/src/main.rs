use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};

use maelstrom_rs::{Body, Message, Node, Workload};
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
        topology: HashMap<String, Vec<String>>,
    },
    TopologyOk,
}

struct BroadcastWorkload {
    messages: Arc<Mutex<HashSet<usize>>>,
    topology: Arc<Mutex<HashMap<String, Vec<String>>>>,
}

impl Workload<BroadcastBody> for BroadcastWorkload {
    fn handle(&mut self, recv: Message<BroadcastBody>, tx: Sender<Message<BroadcastBody>>) {
        let messages = Arc::clone(&self.messages);
        let topology = Arc::clone(&self.topology);

        tokio::spawn(async move {
            match recv.body().r#type() {
                BroadcastBody::Broadcast { message } => {
                    messages.lock().unwrap().insert(*message);

                    tx.send(recv.into_reply(BroadcastBody::BroadcastOk.into()))
                        .await
                        .ok();
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
