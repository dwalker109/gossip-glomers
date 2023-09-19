use maelstrom_rs::{make_reply, Body, DataFields, Message, Node, Workload};
use tokio::io::{stdin, stdout};
use tokio::sync::mpsc::Sender;

#[tokio::main]
async fn main() {
    let mut node = Node::startup(stdin(), stdout()).await.unwrap();
    let workload = EchoWorkload {};

    node.run(workload).await;
}

struct EchoWorkload;

#[async_trait::async_trait]
impl Workload for EchoWorkload {
    async fn handle(&self, message: Message, tx: Sender<Message>) {
        let future = match message.data().clone() {
            DataFields::Echo { echo } => {
                async move {
                    let reply = make_reply(message, Body::new(DataFields::EchoOk { echo }));
                    tx.send(reply).await.ok();
                }
            }
            _ => unimplemented!(),
        };

        tokio::spawn(future);
    }
}
