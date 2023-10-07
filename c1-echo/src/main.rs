use maelstrom_rs::{Message, Node, Outbox, Workload};
use serde::{Deserialize, Serialize};
use tokio::io::{stdin, stdout};

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
    fn handle(&mut self, message: Message<EchoBody>, outbox: Outbox<EchoBody>) {
        let future = match message.body().r#type().to_owned() {
            EchoBody::Echo { echo } => {
                async move {
                    outbox
                        .reply(&message, EchoBody::EchoOk { echo }.into())
                        .await;
                }
            }
            _ => unimplemented!(),
        };

        tokio::spawn(future);
    }
}
