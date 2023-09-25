use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

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
    pub(crate) msg_id: Option<usize>,
    pub(crate) in_reply_to: Option<usize>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "snake_case")]
#[serde(tag = "type")]
pub(crate) enum InitType {
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
            msg_id: None,
            in_reply_to: None,
        }
    }
}
