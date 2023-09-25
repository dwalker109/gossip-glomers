use crate::message::Message;
use serde::de::DeserializeOwned;
use tokio::sync::mpsc;

/// Implement this trait to define how a node responds to client messages.
pub trait Workload<M: DeserializeOwned> {
    /// Handle will be called once per incoming message, and can send messages
    /// out again via the provided channel. Any number of message can be send -
    /// they do not have to be 1:1 replies.
    ///
    /// While this method is not async and is called synchronously, consider
    /// spawning a task to carry out your processing, allowing the node to carry on
    /// handling incoming messages.
    fn handle(&self, message: Message<M>, tx: mpsc::Sender<Message<M>>);
}
