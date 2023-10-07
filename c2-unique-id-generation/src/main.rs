use maelstrom_rs::{Message, Node, Outbox, Workload};
use serde::{Deserialize, Serialize};
use tokio::io::{stdin, stdout};

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
    fn handle(&mut self, message: Message<UniqueIdBody>, outbox: Outbox<UniqueIdBody>) {
        let future = match message.body().r#type().to_owned() {
            UniqueIdBody::Generate => {
                async move {
                    outbox
                        .reply(
                            &message,
                            UniqueIdBody::GenerateOk {
                                id: uuid::Uuid::new_v4().to_string(),
                            }
                            .into(),
                        )
                        .await;
                }
            }
            _ => unimplemented!(),
        };

        tokio::spawn(future);
    }
}
