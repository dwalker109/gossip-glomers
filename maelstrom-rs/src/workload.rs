use crate::message::Message;
use tokio::sync::mpsc;

#[async_trait::async_trait]
pub trait Workload {
    async fn handle(&self, message: Message, tx: mpsc::Sender<Message>);
}
