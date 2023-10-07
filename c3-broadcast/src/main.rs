use maelstrom_rs::{Id, Message, Node, Outbox, Workload};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex, RwLock};
use tokio::io::{stdin, stdout};

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
    fn handle(&mut self, recv: Message<BroadcastBody>, outbox: Outbox<BroadcastBody>) {
        let messages = Arc::clone(&self.messages);
        let topology = Arc::clone(&self.topology);

        tokio::spawn(async move {
            let self_node_id = recv.dest();
            let from_node_id = recv.src();

            match recv.body().r#type() {
                BroadcastBody::Broadcast { message } => {
                    messages.lock().unwrap().insert(*message);

                    outbox.reply(&recv, BroadcastBody::BroadcastOk.into()).await;

                    let to_amp = { topology.read().unwrap().get(self_node_id).cloned() };
                    if let Some(to_amp) = to_amp {
                        for n in to_amp.iter().filter(|n| *n != from_node_id) {
                            outbox.send(n.to_owned(), recv.body().clone()).await;
                        }
                    }
                }
                BroadcastBody::BroadcastOk => unimplemented!(),
                BroadcastBody::Read => {
                    outbox
                        .reply(
                            &recv,
                            BroadcastBody::ReadOk {
                                messages: Arc::clone(&messages),
                            }
                            .into(),
                        )
                        .await;
                }
                BroadcastBody::ReadOk { messages: _ } => todo!(),
                BroadcastBody::Topology {
                    topology: new_topology,
                } => {
                    *topology.write().unwrap() = new_topology.clone();
                    outbox.reply(&recv, BroadcastBody::TopologyOk.into()).await;
                }
                BroadcastBody::TopologyOk => todo!(),
            }
        });
    }
}
