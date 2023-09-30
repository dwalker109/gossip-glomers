use maelstrom_rs::{Body, Id, Message, Node, Workload};
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
enum EchoBody {
    Echo { echo: String },
    EchoOk { echo: String },
}

struct EchoWorkload;

impl Workload<EchoBody> for EchoWorkload {
    fn handle(&mut self, message: Message<EchoBody>, tx: Sender<Message<EchoBody>>) {
        let future = match message.body().r#type().to_owned() {
            EchoBody::Echo { echo } => {
                async move {
                    let reply = message.into_reply(EchoBody::EchoOk { echo }.into());
                    tx.send(reply).await.ok();
                }
            }
            _ => unimplemented!(),
        };

        tokio::spawn(future);
    }
}
