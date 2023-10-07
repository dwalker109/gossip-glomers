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

mod deferrable_id;
pub use deferrable_id::Id;

mod outbox;
pub use outbox::Outbox;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Message<T> {
    pub(super) src: Id<String>,
    pub(super) dest: Id<String>,
    pub(super) body: Body<T>,
}

impl<T> Message<T> {
    pub fn into_reply(self, reply_body: Body<T>) -> Self {
        let mut reply = Self::new(Id::Defer, self.src, reply_body);
        reply.body.in_reply_to = self.body.msg_id;

        reply
    }

    pub fn new(src: Id<String>, dest: Id<String>, body: Body<T>) -> Self {
        Message { src, dest, body }
    }

    pub fn for_send(dest: Id<String>, mut body: Body<T>) -> Self {
        body.msg_id = Id::Known(None);

        Message {
            src: Id::Defer,
            dest,
            body,
        }
    }
}

impl<T> Message<T> {
    pub fn src(&self) -> &Id<String> {
        &self.src
    }

    pub fn dest(&self) -> &Id<String> {
        &self.dest
    }

    pub fn body(&self) -> &Body<T> {
        &self.body
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Body<T> {
    #[serde(flatten)]
    r#type: T,
    pub(super) msg_id: Id<usize>,
    in_reply_to: Id<usize>,
}

impl<T> Body<T> {
    pub fn new(r#type: T, msg_id: Id<usize>, in_reply_to: Id<usize>) -> Self {
        Self {
            r#type,
            msg_id,
            in_reply_to,
        }
    }

    pub fn r#type(&self) -> &T {
        &self.r#type
    }

    pub fn can_reply(&self) -> bool {
        matches!(self.msg_id, Id::Known(Some(_)))
    }

    pub fn is_reply(&self) -> bool {
        matches!(self.in_reply_to, Id::Known(Some(_)))
    }
}

impl<T> From<T> for Body<T> {
    fn from(value: T) -> Self {
        Self::new(value, Id::Defer, Id::Defer)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "snake_case")]
#[serde(tag = "type")]
pub(super) enum InitBody {
    Init {
        node_id: String,
        node_ids: Vec<String>,
    },
    InitOk,
}
