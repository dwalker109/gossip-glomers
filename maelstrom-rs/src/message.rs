//! Messages are exchanged by nodes and clients, and are generic over their body.
//!
//! Each `Workload` will need to define a body type, which should look like this
//! to work properly since it will be flattened into the parent `Body`:
//!
//! #[derive(Serialize, Deserialize, Debug, Clone)]
//! #[serde(rename_all = "snake_case")]
//! #[serde(tag = "type")]
//! pub(super) enum Body {
//!     A,
//!     B,
//!     C
//! }

use serde::{Deserialize, Serialize};

mod outbox;
pub use outbox::Outbox;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Message<T> {
    pub(crate) src: String,
    pub(crate) dest: String,
    pub(crate) body: Body<T>,
}

impl<T> Message<T> {
    pub fn new(dest: String, body: Body<T>) -> Self {
        Message {
            src: String::default(), // Will be set on send
            dest,
            body,
        }
    }
}

impl<T> Message<T> {
    pub fn src(&self) -> &str {
        &self.src
    }

    pub fn dest(&self) -> &str {
        &self.dest
    }

    pub fn body(&self) -> &Body<T> {
        &self.body
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Body<T> {
    #[serde(flatten)]
    pub(crate) r#type: T,
    pub(crate) msg_id: Option<usize>,
    pub(crate) in_reply_to: Option<usize>,
}

impl<T> Body<T> {
    pub fn new(r#type: T, msg_id: Option<usize>, in_reply_to: Option<usize>) -> Self {
        Self {
            r#type,
            msg_id,
            in_reply_to,
        }
    }

    pub fn r#type(&self) -> &T {
        &self.r#type
    }
}

impl<T> From<T> for Body<T> {
    fn from(value: T) -> Self {
        Self::new(value, None, None)
    }
}
