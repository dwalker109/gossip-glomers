use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Clone)]
pub struct Message {
    pub(crate) src: Option<String>,
    pub(crate) dest: Option<String>,
    pub(crate) body: Body,
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Clone)]
pub struct Body {
    #[serde(flatten)]
    pub(crate) r#type: DataFields,
    pub(crate) msg_id: Option<usize>,
    pub(crate) in_reply_to: Option<usize>,
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Clone)]
#[serde(rename_all = "snake_case")]
#[serde(tag = "type")]
pub enum DataFields {
    Init {
        node_id: String,
        node_ids: Vec<String>,
    },
    InitOk,
    Topology,
    TopologyOk,
    Broadcast,
    BroadcastOk,
    Read,
    ReadOk,
    Echo {
        echo: String,
    },
    EchoOk {
        echo: String,
    },
    Add,
    AddOk,
    Send,
    SendOk,
    Poll,
    PollOk,
    CommitOffsets,
    CommitOffsetsOk,
    ListCommittedOffsets,
    ListCommittedOffsetsOk,
    Write,
    WriteOk,
    Cas,
    CasOk,
    Txn,
    TxnOk,
    Generate,
    GenerateOk,
}

impl Message {
    pub fn data(&self) -> &DataFields {
        &self.body.r#type
    }
}

impl Body {
    pub fn new(r#type: DataFields) -> Self {
        Self {
            r#type,
            msg_id: None,
            in_reply_to: None,
        }
    }
}
