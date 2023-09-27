use serde::{de::DeserializeOwned, Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
#[serde(untagged)]
pub enum Id<T> {
    Value(Option<T>),
    #[serde(skip)]
    Defer,
}

impl<T> Id<T> {
    pub fn coalesce(self, val: Id<T>) -> Self {
        match self {
            Id::Defer => val,
            _ => self,
        }
    }

    pub fn else_coalesce(self, coalesce_fn: impl Fn() -> Id<T>) -> Self {
        match self {
            Id::Defer => coalesce_fn(),
            _ => self,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Message<T> {
    pub(crate) src: Option<String>,
    pub(crate) dest: Option<String>,
    pub(crate) body: Body<T>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Body<T> {
    #[serde(flatten)]
    pub(crate) r#type: T,
    pub(crate) msg_id: Id<usize>,
    pub(crate) in_reply_to: Id<usize>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "snake_case")]
#[serde(tag = "type")]
pub(crate) enum InitBody {
    Init {
        node_id: String,
        node_ids: Vec<String>,
    },
    InitOk,
}

impl<T: DeserializeOwned> Message<T> {
    pub fn data(&self) -> &T {
        &self.body.r#type
    }
}

impl<T: DeserializeOwned> Body<T> {
    pub fn new(r#type: T) -> Self {
        Self {
            r#type,
            msg_id: Id::Defer,
            in_reply_to: Id::Defer,
        }
    }
}
