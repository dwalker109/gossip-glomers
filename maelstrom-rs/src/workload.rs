use crate::message::Message;
use serde::de::DeserializeOwned;
use tokio::sync::mpsc;

#[async_trait::async_trait]
pub trait Workload<M: DeserializeOwned + Send + Sync + 'static> {
    async fn handle(&self, message: Message<M>, tx: mpsc::Sender<Message<M>>);
}
