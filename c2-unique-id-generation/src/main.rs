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
enum UniqueIdBody {
    Generate,
    GenerateOk { id: String },
}

struct UniqueIdWorkload;

impl Workload<UniqueIdBody> for UniqueIdWorkload {
    fn handle(&mut self, message: Message<UniqueIdBody>, tx: Sender<Message<UniqueIdBody>>) {
        let future = match message.data().to_owned() {
            UniqueIdBody::Generate => {
                async move {
                    let reply = make_reply(
                        message,
                        Body::new(UniqueIdBody::GenerateOk {
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
