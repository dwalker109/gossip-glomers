use maelstrom_rs::{Id, Message, Node, Outbox, Workload};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock};
use tokio::io::{stdin, stdout};

#[tokio::main]
async fn main() {
    let mut node = Node::startup(stdin(), stdout()).await.unwrap();
    let workload = BroadcastWorkload {
        messages: Arc::new(RwLock::new(HashSet::new())),
        neighbours: Arc::new(RwLock::new(Vec::new())),
    };
    node.run(workload).await;
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "snake_case")]
#[serde(tag = "type")]
enum BroadcastBody {
    Broadcast { message: usize },
    BroadcastOk,
    Read,
    ReadOk { messages: Arc<RwLock<Messages>> },
    Topology { topology: Topology },
    TopologyOk,
}

type Messages = HashSet<usize>;
type Topology = HashMap<Id<String>, Vec<Id<String>>>;
type Neighbours = Vec<Id<String>>;

struct BroadcastWorkload {
    messages: Arc<RwLock<Messages>>,
    neighbours: Arc<RwLock<Neighbours>>,
}

impl Workload<BroadcastBody> for BroadcastWorkload {
    fn handle(&mut self, recv: Message<BroadcastBody>, outbox: Outbox<BroadcastBody>) {
        let messages = Arc::clone(&self.messages);
        let neighbours = Arc::clone(&self.neighbours);

        tokio::spawn(async move {
            let self_node_id = recv.dest();
            let from_node_id = recv.src();

            match recv.body().r#type() {
                BroadcastBody::Broadcast { message } => {
                    if messages.read().unwrap().contains(message) {
                        // Already seen this message - don't amplify
                        outbox.reply(&recv, BroadcastBody::BroadcastOk.into()).await;
                        return;
                    }

                    // Broadcast to peers, excluding source
                    let ids = neighbours
                        .read()
                        .unwrap()
                        .iter()
                        .filter(|&n| n != from_node_id)
                        .cloned()
                        .collect::<Vec<_>>();
                    for id in ids {
                        let outbox = outbox.clone();
                        let body = recv.body().clone();
                        tokio::spawn(async move {
                            outbox.rpc(id, body.clone()).await;
                        });
                    }

                    // Store, and then ack if required
                    messages.write().unwrap().insert(*message);
                    if recv.can_reply() {
                        outbox.reply(&recv, BroadcastBody::BroadcastOk.into()).await;
                    }
                }

                BroadcastBody::BroadcastOk => outbox.ack(&recv),

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

                BroadcastBody::Topology { topology } => {
                    *neighbours.write().unwrap() = topology.get(self_node_id).unwrap().clone();
                    outbox.reply(&recv, BroadcastBody::TopologyOk.into()).await;
                }

                BroadcastBody::ReadOk { .. } => unimplemented!(),
                BroadcastBody::TopologyOk => unimplemented!(),
            }
        });
    }
}
