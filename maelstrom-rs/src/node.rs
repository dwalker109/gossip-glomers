use crate::message::{Body, Message, Outbox};
use crate::workload::Workload;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use tokio::io::{
    AsyncBufReadExt, AsyncRead, AsyncWrite, AsyncWriteExt, BufReader, BufWriter, Error, ErrorKind,
    Result,
};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "snake_case")]
#[serde(tag = "type")]
enum InitBody {
    Init {
        node_id: String,
        node_ids: Vec<String>,
    },
    InitOk,
}

/// A single node in the network, connected to the provided reader and writer.
pub struct Node<
    R: AsyncRead + Unpin + 'static,
    M: Clone + DeserializeOwned + Serialize + Send + Sync + 'static,
> {
    _node_id: String,
    _node_ids: Vec<String>,
    reader: BufReader<R>,
    reader_buf: String,
    outbox: Outbox<M>,
}

impl<
        R: AsyncRead + Unpin + 'static,
        M: Clone + DeserializeOwned + Serialize + Send + Sync + 'static,
    > Node<R, M>
{
    /// Starts a new Node connected to the provided input and output.
    ///
    /// Will not complete until an init message is received on the provided reader.
    pub async fn startup<W: AsyncWrite + Unpin + Send + Sync + 'static>(
        reader: R,
        writer: W,
    ) -> Result<Self> {
        let mut reader = BufReader::new(reader);
        let mut reader_buf = String::with_capacity(256);
        let mut writer = BufWriter::new(writer);

        reader.read_line(&mut reader_buf).await?;
        let msg_init: Message<InitBody> = serde_json::from_str(&reader_buf)?;
        let (node_id, _node_ids) = Self::init(msg_init, &mut writer).await?;

        let node = Self {
            _node_id: node_id.clone(),
            _node_ids,
            reader,
            reader_buf,
            outbox: Outbox::new(node_id, 1, writer, 32),
        };

        Ok(node)
    }

    async fn init(
        msg_init: Message<InitBody>,
        writer: &mut BufWriter<impl AsyncWrite + Unpin>,
    ) -> Result<(String, Vec<String>)> {
        match &msg_init.body().r#type() {
            InitBody::Init { node_id, node_ids } => {
                let msg_init_ok = Message {
                    src: node_id.to_owned(),
                    dest: msg_init.src.clone(),
                    body: Body::new(InitBody::InitOk, Some(0), msg_init.body().msg_id),
                };

                let b = serde_json::to_vec(&msg_init_ok)?;
                writer.write_all(&b).await.ok();
                writer.write_u8(b'\n').await.ok();
                writer.flush().await.ok();

                Ok((node_id.to_owned(), node_ids.to_owned()))
            }
            _ => Err(Error::new(
                ErrorKind::InvalidData,
                "init message must be received on startup",
            )),
        }
    }

    /// Run the node, receiving and handling (via the workload impl) messages indefinitely.
    pub async fn run(&mut self, mut workload: impl Workload<M>) {
        while let Ok(Some(recv)) = self.recv().await {
            workload.handle(recv, self.outbox.clone());
        }
    }

    /// Receive a single message from the node's reader.
    async fn recv(&mut self) -> Result<Option<Message<M>>> {
        self.reader_buf.clear();
        self.reader.read_line(&mut self.reader_buf).await?;
        let msg: Message<M> = serde_json::from_str(&self.reader_buf)?;

        Ok(Some(msg))
    }
}
