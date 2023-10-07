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
        writer: BufWriter<W>,
        buffer: usize,
    ) -> Self {
        let (tx, rx) = mpsc::channel::<Message<T>>(buffer);
        tokio::spawn(Self::handle_write(node_id.to_owned(), writer, rx));
        Self(tx)
    }

    pub async fn reply(&self, subject: &Message<T>, body: Body<T>) {
        let mut reply = Message::new(Id::Defer, subject.src.clone(), body);
        reply.body.in_reply_to = subject.body.msg_id;

        self.0.send(reply).await.ok();
    }

    pub async fn send(&self, dest: Id<String>, mut body: Body<T>) {
        body.msg_id = Id::Known(None);
        let message = Message::new(Id::Defer, dest, body);

        self.0.send(message).await.ok();
    }

    pub async fn rpc() {
        todo!()
    }

    async fn handle_write<W: AsyncWrite + Unpin + Send + Sync + 'static>(
        node_id: String,
        mut w: BufWriter<W>,
        mut rx: mpsc::Receiver<Message<T>>,
    ) {
        let next_id = AtomicUsize::new(1);

        while let Some(mut msg) = rx.recv().await {
            // Set src if deferred
            msg.src = msg.src.else_coalesce(|| Id::Known(Some(node_id.clone())));

            // Set msg_id if deferred
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
