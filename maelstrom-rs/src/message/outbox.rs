use crate::{Body, Id, Message};
use serde::{de::DeserializeOwned, Serialize};
use std::sync::{Arc, Mutex};
use std::{sync::atomic::AtomicUsize, sync::atomic::Ordering::AcqRel};
use tokio::{
    io::{AsyncWrite, AsyncWriteExt, BufWriter},
    sync::mpsc,
};

#[derive(Clone)]
pub struct Outbox<T: Clone + DeserializeOwned + Serialize + Send + Sync + 'static>(
    mpsc::Sender<Message<T>>,
    Arc<Mutex<AtomicUsize>>,
);

impl<T: Clone + DeserializeOwned + Serialize + Send + Sync + 'static> Outbox<T> {
    pub fn new<W: AsyncWrite + Unpin + Send + Sync + 'static>(
        node_id: &str,
        init_msg_id: usize,
        writer: BufWriter<W>,
        buffer: usize,
    ) -> Self {
        let (tx, rx) = mpsc::channel::<Message<T>>(buffer);
        tokio::spawn(Self::handle_write(node_id.to_owned(), writer, rx));

        Self(tx, Arc::new(Mutex::new(AtomicUsize::new(init_msg_id))))
    }

    pub async fn reply(&self, subject: &Message<T>, body: Body<T>) -> Id<usize> {
        // Don't reply to "send" (server comms) messages
        if matches!(subject.body.msg_id, Id::Known(None)) {
            return Id::Known(None);
        }

        let mut message = Message::new(Id::Defer, subject.src.clone(), body);
        message.body.in_reply_to = subject.body.msg_id;
        message.body.msg_id = message.body.msg_id.else_coalesce(|| self.next_msg_id());
        let msg_id = message.body.msg_id.clone();

        self.0.send(message).await.unwrap();

        msg_id
    }

    pub async fn send(&self, dest: Id<String>, body: Body<T>) {
        let mut message = Message::new(Id::Defer, dest, body);
        message.body.in_reply_to = Id::Known(None);
        message.body.msg_id = Id::Known(None);

        self.0.send(message).await.unwrap();
    }

    pub async fn rpc(&self, dest: Id<String>, body: Body<T>) -> Id<usize> {
        let mut message = Message::new(Id::Defer, dest, body);
        message.body.in_reply_to = Id::Known(None);
        message.body.msg_id = message.body.msg_id.else_coalesce(|| self.next_msg_id());
        let msg_id = self.next_msg_id();

        self.0.send(message).await.unwrap();

        msg_id
    }

    fn next_msg_id(&self) -> Id<usize> {
        Id::Known(Some(self.1.lock().unwrap().fetch_add(1, AcqRel)))
    }

    async fn handle_write<W: AsyncWrite + Unpin + Send + Sync + 'static>(
        node_id: String,
        mut writer: BufWriter<W>,
        mut rx: mpsc::Receiver<Message<T>>,
    ) {
        while let Some(mut msg) = rx.recv().await {
            // Set src if deferred
            msg.src = msg.src.else_coalesce(|| Id::Known(Some(node_id.clone())));

            if let Ok(b) = serde_json::to_vec(&msg) {
                writer.write_all(&b).await.unwrap();
                writer.write_u8(b'\n').await.unwrap();
                writer.flush().await.unwrap();
            }
        }
    }
}
