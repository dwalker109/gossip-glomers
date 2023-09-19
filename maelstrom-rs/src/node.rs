use crate::message::{Body, DataFields, Message};
use crate::workload::Workload;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering::AcqRel;
use tokio::io::{
    AsyncBufReadExt, AsyncRead, AsyncWrite, AsyncWriteExt, BufReader, BufWriter, Error, ErrorKind,
    Result,
};
use tokio::sync::mpsc;

/// A single node in the network.
pub struct Node<R: AsyncRead + Unpin + 'static> {
    node_id: String,
    node_ids: Vec<String>,
    reader: BufReader<R>,
    reader_buf: String,
    tx: mpsc::Sender<Message>,
}

impl<R: AsyncRead + Unpin + 'static> Node<R> {
    /// Starts a new Node connected to the provided input and output.
    ///
    /// Will not complete until an init message is received on the provided reader.
    pub async fn startup<W: AsyncWrite + Unpin + Send + Sync + 'static>(
        reader: R,
        writer: W,
    ) -> Result<Self> {
        let mut reader = BufReader::new(reader);
        let mut reader_buf = String::with_capacity(256);

        reader.read_line(&mut reader_buf).await?;
        let msg: Message = serde_json::from_str(&reader_buf)?;
        reader_buf.clear();

        let (node_id, node_ids) = if let DataFields::Init { node_id, node_ids } = &msg.body.r#type {
            (node_id.to_owned(), node_ids.to_owned())
        } else {
            return Err(Error::new(
                ErrorKind::InvalidData,
                "init message must be received on startup",
            ));
        };

        let writer = BufWriter::new(writer);
        let (tx, rx) = mpsc::channel::<Message>(32);

        let node = Self {
            node_id,
            node_ids,
            reader,
            reader_buf,
            tx,
        };

        let reply = make_reply(msg, Body::new(DataFields::InitOk));
        node.tx.send(reply).await.ok();

        tokio::spawn(Self::handle_write(node.node_id.clone(), writer, rx));

        Ok(node)
    }

    /// Run the node, receiving and handling messages indefinitely.
    pub async fn run(&mut self, workload: impl Workload) {
        while let Ok(Some(recv)) = self.recv().await {
            workload.handle(recv, self.tx.clone()).await;
        }
    }

    /// Receive a single message from the node's reader.
    async fn recv(&mut self) -> Result<Option<Message>> {
        self.reader.read_line(&mut self.reader_buf).await?;
        let msg: Message = serde_json::from_str(&self.reader_buf)?;
        self.reader_buf.clear();

        Ok(Some(msg))
    }

    /// Emit messages from the node.
    async fn handle_write<W: AsyncWrite + Unpin + Send + Sync + 'static>(
        node_id: String,
        mut w: BufWriter<W>,
        mut rx: mpsc::Receiver<Message>,
    ) {
        let next_id = AtomicUsize::default();

        while let Some(mut msg) = rx.recv().await {
            // Set src if not already set
            msg.src = msg.src.or(Some(node_id.clone()));

            // Set message id if not already set
            msg.body.msg_id = msg
                .body
                .msg_id
                .or_else(|| Some(next_id.fetch_add(1, AcqRel)));

            if let Ok(b) = serde_json::to_vec(&msg) {
                w.write_all(&b).await.ok();
                w.write_u8(b'\n').await.ok();
                w.flush().await.ok();
            }
        }
    }
}

/// Build a reply message by consuming the subject, and a new message body.
pub fn make_reply(recv: Message, send_body: Body) -> Message {
    let mut reply = Message {
        src: None, // handle_write will set this if we don't
        dest: recv.src,
        body: send_body,
    };

    // Set reply to the received message id if not already set
    reply.body.in_reply_to = reply.body.in_reply_to.or(recv.body.msg_id);

    reply
}

#[cfg(test)]
mod tests {}
