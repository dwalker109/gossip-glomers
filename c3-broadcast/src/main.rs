use maelstrom_rs::{make_reply, Body, Message, Node, Workload};
use serde::{Deserialize, Serialize};
use tokio::io::{stdin, stdout};
use tokio::sync::mpsc::Sender;

#[tokio::main]
async fn main() {
    let mut node = Node::startup(stdin(), stdout()).await.unwrap();
    let workload = BroadcastWorkload {};
    node.run(workload).await;
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "snake_case")]
#[serde(tag = "type")]
enum BroadcastBody {
    Generate,
    GenerateOk { id: String },
}

struct BroadcastWorkload;

impl Workload<BroadcastBody> for BroadcastWorkload {
    fn handle(&self, message: Message<BroadcastBody>, tx: Sender<Message<BroadcastBody>>) {}
}
