//! Async client for the Maelstrom distributed systems workbench.
//!
//! Provides a Node implementation generic over an async reader and writer,
//! and a Workflow trait which should be implemented for the specific workflow
//! you are exploring.

mod message;
mod node;
mod workload;

pub use message::{Body, Id, Message, Outbox};
pub use node::Node;
pub use workload::Workload;
