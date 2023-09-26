use serde::{Deserialize, Serialize};

mod outbox;
pub use outbox::Outbox;

/// All messages comprise of a source node, destination node, and body.
///
/// Messages are exchanged by nodes and clients, and are generic over their body.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Message<T> {
    pub(crate) src: String,
    pub(crate) dest: String,
    pub(crate) body: Body<T>,
}

impl<T> Message<T> {
    /// Create a new message.
    pub fn new(dest: String, body: Body<T>) -> Self {
        Message {
            src: String::default(), // Will be set on send
            dest,
            body,
        }
    }
}

impl<T> Message<T> {
    /// Get a reference to the message source id.
    pub fn src(&self) -> &str {
        &self.src
    }

    /// Get a reference to the message destination id.
    pub fn dest(&self) -> &str {
        &self.dest
    }

    /// Get a reference to the message body.
    pub fn body(&self) -> &Body<T> {
        &self.body
    }
}

/// A message body comprises of an optional message id and (for replies only) id of the subject of the reply.
/// Different payloads contain various other fields, flattened to/from the `r#type` field.
///
/// Each `Workload` will need to define a body type, which should look like this
/// to work properly since it will be flattened into the parent `Body`:
///
/// ```
/// #[derive(Serialize, Deserialize, Debug, Clone)]
/// #[serde(rename_all = "snake_case")]
/// #[serde(tag = "type")]
/// pub(super) enum Body {
///     A,
///     B,
///     C
/// }
/// ```
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Body<T> {
    #[serde(flatten)]
    pub(crate) r#type: T,
    pub(crate) msg_id: Option<usize>,
    pub(crate) in_reply_to: Option<usize>,
}

impl<T> Body<T> {
    /// Create a new body.
    pub fn new(r#type: T, msg_id: Option<usize>, in_reply_to: Option<usize>) -> Self {
        Self {
            r#type,
            msg_id,
            in_reply_to,
        }
    }

    /// Get a reference to the body type, which contains all custom fields for this kind of message.
    pub fn r#type(&self) -> &T {
        &self.r#type
    }
}

impl<T> From<T> for Body<T> {
    fn from(value: T) -> Self {
        Self::new(value, None, None)
    }
}
