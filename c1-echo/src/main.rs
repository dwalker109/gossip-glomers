use maelstrom_rs::{make_reply, Body, Message, Node, Workload};
use serde::{Deserialize, Serialize};
use tokio::io::{stdin, stdout};
use tokio::sync::mpsc::Sender;

#[tokio::main]
async fn main() {
    let mut node = Node::startup(stdin(), stdout()).await.unwrap();
    let workload = EchoWorkload {};
    node.run(workload).await;
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "snake_case")]
#[serde(tag = "type")]
enum EchoType {
    Echo { echo: String },
    EchoOk { echo: String },
}

struct EchoWorkload;

#[async_trait::async_trait]
impl Workload<EchoType> for EchoWorkload {
    async fn handle(&self, message: Message<EchoType>, tx: Sender<Message<EchoType>>) {
        let future = match message.data().to_owned() {
            EchoType::Echo { echo } => {
                async move {
                    let reply = make_reply(message, Body::new(EchoType::EchoOk { echo }));
                    tx.send(reply).await.ok();
                }
            }
            _ => unimplemented!(),
        };

        tokio::spawn(future);
    }
}
