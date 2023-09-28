use crate::message::{Body, Id, InitBody, Message};
use crate::workload::Workload;
use serde::{de::DeserializeOwned, Serialize};
use std::sync::atomic::{AtomicUsize, Ordering::AcqRel};
use tokio::{
    io::{
        AsyncBufReadExt, AsyncRead, AsyncWrite, AsyncWriteExt, BufReader, BufWriter, Error,
        ErrorKind, Result,
    },
    sync::mpsc,
};

/// A single node in the network, connected to the provided reader and writer.
pub struct Node<
    R: AsyncRead + Unpin + 'static,
    M: DeserializeOwned + Serialize + Send + Sync + 'static,
> {
    node_id: String,
    _node_ids: Vec<String>,
    reader: BufReader<R>,
    reader_buf: String,
    tx: mpsc::Sender<Message<M>>,
}

impl<R: AsyncRead + Unpin + 'static, M: DeserializeOwned + Serialize + Send + Sync + 'static>
    Node<R, M>
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

        let (node_id, _node_ids, msg_init_ok) = Self::init(msg_init).await?;
        let b = serde_json::to_vec(&msg_init_ok)?;
        writer.write_all(&b).await.ok();
        writer.write_u8(b'\n').await.ok();
        writer.flush().await.ok();

        let (tx, rx) = mpsc::channel::<Message<M>>(32);

        let node = Self {
            node_id,
            _node_ids,
            reader,
            reader_buf,
            tx,
        };

        tokio::spawn(Self::handle_write(node.node_id.clone(), writer, rx));

        Ok(node)
    }

    /// Process the init message.
    async fn init(msg_init: Message<InitBody>) -> Result<(String, Vec<String>, Message<InitBody>)> {
        match &msg_init.body.r#type {
            InitBody::Init { node_id, node_ids } => {
                let msg_init_ok = Message {
                    src: Some(node_id.to_owned()),
                    dest: msg_init.src.to_owned(),
                    body: Body {
                        r#type: InitBody::InitOk,
                        msg_id: Id::Known(Some(0)),
                        in_reply_to: msg_init.body.msg_id,
                    },
                };

                Ok((node_id.to_owned(), node_ids.to_owned(), msg_init_ok))
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
            workload.handle(recv, self.tx.clone());
        }
    }

    /// Receive a single message from the node's reader.
    async fn recv(&mut self) -> Result<Option<Message<M>>> {
        self.reader_buf.clear();
        self.reader.read_line(&mut self.reader_buf).await?;
        let msg: Message<M> = serde_json::from_str(&self.reader_buf)?;

        Ok(Some(msg))
    }

    /// Emit messages from the node.
    async fn handle_write<W: AsyncWrite + Unpin + Send + Sync + 'static>(
        node_id: String,
        mut w: BufWriter<W>,
        mut rx: mpsc::Receiver<Message<M>>,
    ) {
        let next_id = AtomicUsize::new(1);

        while let Some(mut msg) = rx.recv().await {
            // Set src if not already set
            msg.src = msg.src.or(Some(node_id.clone()));

            // Set message id if deferred
            msg.body.msg_id = msg
                .body
                .msg_id
                .else_coalesce(|| Id::Known(Some(next_id.fetch_add(1, AcqRel))));

            if let Ok(b) = serde_json::to_vec(&msg) {
                w.write_all(&b).await.ok();
                w.write_u8(b'\n').await.ok();
                w.flush().await.ok();
            }
        }
    }
}

/// Build a reply message by consuming the subject, and a new message body.
pub fn make_reply<M: DeserializeOwned>(recv: Message<M>, send_body: Body<M>) -> Message<M> {
    let mut reply = Message {
        src: None, // handle_write will set this if we don't
        dest: recv.src,
        body: send_body,
    };

    // Set reply to the received message id if deferred
    reply.body.in_reply_to = reply.body.in_reply_to.coalesce(recv.body.msg_id);

    reply
}

#[cfg(test)]
mod tests {}
