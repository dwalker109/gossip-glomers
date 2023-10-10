use crate::{Body, Id, Message};
use serde::{de::DeserializeOwned, Serialize};
use std::{sync::atomic::AtomicUsize, sync::atomic::Ordering::AcqRel};
use tokio::{
    io::{AsyncWrite, AsyncWriteExt, BufWriter},
    sync::mpsc,
};

#[derive(Clone)]
pub struct Outbox<T: Clone + DeserializeOwned + Serialize + Send + Sync + 'static>(
    mpsc::Sender<Message<T>>,
);

impl<T: Clone + DeserializeOwned + Serialize + Send + Sync + 'static> Outbox<T> {
    pub fn new<W: AsyncWrite + Unpin + Send + Sync + 'static>(
        node_id: &str,
        init_msg_id: usize,
        writer: BufWriter<W>,
        buffer: usize,
    ) -> Self {
        let (tx, rx) = mpsc::channel::<Message<T>>(buffer);
        tokio::spawn(Self::handle_write(
            node_id.to_owned(),
            init_msg_id,
            writer,
            rx,
        ));
        Self(tx)
    }

    pub async fn reply(&self, subject: &Message<T>, body: Body<T>) {
        // Don't reply to "send" (server comms) messages
        if matches!(subject.body.msg_id, Id::Known(None)) {
            return;
        }

        let mut reply = Message::new(Id::Defer, subject.src.clone(), body);
        reply.body.in_reply_to = subject.body.msg_id;

        self.0.send(reply).await.unwrap();
    }

    pub async fn send(&self, dest: Id<String>, mut body: Body<T>) {
        body.msg_id = Id::Known(None);
        body.in_reply_to = Id::Known(None);
        let message = Message::new(Id::Defer, dest, body);

        self.0.send(message).await.unwrap();
    }

    pub async fn rpc() {
        todo!()
    }

    async fn handle_write<W: AsyncWrite + Unpin + Send + Sync + 'static>(
        node_id: String,
        init_msg_id: usize,
        mut writer: BufWriter<W>,
        mut rx: mpsc::Receiver<Message<T>>,
    ) {
        let next_id = AtomicUsize::new(init_msg_id);

        while let Some(mut msg) = rx.recv().await {
            // Set src if deferred
            msg.src = msg.src.else_coalesce(|| Id::Known(Some(node_id.clone())));

            // Set msg_id if deferred
            msg.body.msg_id = msg
                .body
                .msg_id
                .else_coalesce(|| Id::Known(Some(next_id.fetch_add(1, AcqRel))));

            if let Ok(b) = serde_json::to_vec(&msg) {
                writer.write_all(&b).await.unwrap();
                writer.write_u8(b'\n').await.unwrap();
                writer.flush().await.unwrap();
            }
        }
    }
}
