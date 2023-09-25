use maelstrom_rs::{make_reply, Body, Message, Node, Workload};
use serde::{Deserialize, Serialize};
use tokio::io::{stdin, stdout};
use tokio::sync::mpsc::Sender;

#[tokio::main]
async fn main() {
    let mut node = Node::startup(stdin(), stdout()).await.unwrap();
    let workload = UniqueIdWorkload {};
    node.run(workload).await;
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "snake_case")]
#[serde(tag = "type")]
enum UniqueIdType {
    Generate,
    GenerateOk { id: String },
}

struct UniqueIdWorkload;

impl Workload<UniqueIdType> for UniqueIdWorkload {
    fn handle(&self, message: Message<UniqueIdType>, tx: Sender<Message<UniqueIdType>>) {
        let future = match message.data().to_owned() {
            UniqueIdType::Generate => {
                async move {
                    let reply = make_reply(
                        message,
                        Body::new(UniqueIdType::GenerateOk {
                            id: uuid::Uuid::new_v4().to_string(),
                        }),
                    );
                    tx.send(reply).await.ok();
                }
            }
            _ => unimplemented!(),
        };

        tokio::spawn(future);
    }
}
