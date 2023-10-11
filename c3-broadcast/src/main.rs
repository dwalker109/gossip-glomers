use maelstrom_rs::{Id, Message, Node, Outbox, Workload};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex, RwLock};
use tokio::io::{stdin, stdout};
use tokio::sync;

#[tokio::main]
async fn main() {
    let mut node = Node::startup(stdin(), stdout()).await.unwrap();
    let workload = BroadcastWorkload {
        messages: Arc::new(RwLock::new(HashSet::new())),
        neighbours: Arc::new(RwLock::new(Vec::new())),
        awaiting_reply: HashMap::new(),
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
    awaiting_reply: HashMap<Id<usize>, sync::oneshot::Sender<()>>,
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
                    // TODO: Handle failure

                    if messages.read().unwrap().contains(message) {
                        // Already seen this message - don't do anything
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
                        let (tx, rx) = sync::oneshot::channel::<()>();
                        tokio::spawn(async move {
                            let msg_id = outbox.rpc(id, body.clone()).await;
                            body.
                        });
                    }

                    // Store and ack
                    messages.write().unwrap().insert(*message);
                    outbox.reply(&recv, BroadcastBody::BroadcastOk.into()).await;
                }
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
                BroadcastBody::BroadcastOk => unimplemented!(),
                BroadcastBody::TopologyOk => unimplemented!(),
            }
        });
    }
}
