use crate::message::{Message, Outbox};
use serde::{de::DeserializeOwned, Serialize};

/// Trait defining how a node completes a given workload, responding to the node's incoming messages.
pub trait Workload<M: Clone + DeserializeOwned + Serialize + Send + Sync + 'static> {
    /// Handle will be called once per incoming message, and can send messages
    /// out again via the provided channel. Any number of message can be send -
    /// they do not have to be 1:1 replies.
    ///
    /// While this method is not async and is called synchronously, consider
    /// spawning a task to carry out your processing, allowing the node to carry on
    /// handling incoming messages.
    fn handle(&mut self, recv: Message<M>, outbox: Outbox<M>);
}
