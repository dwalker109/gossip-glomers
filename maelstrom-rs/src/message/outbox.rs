use crate::{Body, Id, Message};
use serde::{de::DeserializeOwned, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::{sync::atomic::AtomicUsize, sync::atomic::Ordering::AcqRel};
use tokio::time::interval;
use tokio::{
    io::{AsyncWrite, AsyncWriteExt, BufWriter},
    sync,
    sync::mpsc,
};

#[derive(Clone)]
pub struct Outbox<T: Clone + DeserializeOwned + Serialize + Send + Sync + 'static> {
    write_tx: mpsc::Sender<Message<T>>,
    next_msg_id: Arc<Mutex<AtomicUsize>>,
    in_flight: Arc<Mutex<HashMap<(Id<String>, Id<usize>), sync::oneshot::Sender<()>>>>,
}

impl<T: Clone + DeserializeOwned + Serialize + Send + Sync + 'static> Outbox<T> {
    pub fn new<W: AsyncWrite + Unpin + Send + Sync + 'static>(
        node_id: &str,
        init_msg_id: usize,
        writer: BufWriter<W>,
        buffer: usize,
    ) -> Self {
        let (write_tx, write_rx) = mpsc::channel::<Message<T>>(buffer);
        tokio::spawn(Self::handle_write(node_id.to_owned(), writer, write_rx));

        Self {
            write_tx,
            next_msg_id: Arc::new(Mutex::new(AtomicUsize::new(init_msg_id))),
            in_flight: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn reply(&self, subject: &Message<T>, body: Body<T>) {
        let mut message = Message::new(Id::Defer, subject.src.clone(), body);
        message.body.in_reply_to = subject.body.msg_id;
        message.body.msg_id = message.body.msg_id.else_coalesce(|| self.next_msg_id());

        self.write_tx.send(message).await.unwrap();
    }

    pub async fn send(&self, dest: Id<String>, body: Body<T>) {
        let mut message = Message::new(Id::Defer, dest, body);
        message.body.in_reply_to = Id::Known(None);
        message.body.msg_id = Id::Known(None);

        self.write_tx.send(message).await.unwrap();
    }

    pub async fn rpc(&self, dest: Id<String>, body: Body<T>) {
        let mut message = Message::new(Id::Defer, dest, body);
        message.body.in_reply_to = Id::Known(None);
        message.body.msg_id = message.body.msg_id.else_coalesce(|| self.next_msg_id());

        let (ack_tx, mut ack_rx) = sync::oneshot::channel::<()>();
        self.in_flight
            .lock()
            .unwrap()
            .insert((message.dest.clone(), message.body.msg_id), ack_tx);

        let write_tx = self.write_tx.clone();
        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(1));

            loop {
                tokio::select! {
                    _ = interval.tick() => {
                        write_tx.send(message.clone()).await.unwrap()
                    },
                    _ = &mut ack_rx => break,
                }
            }
        });
    }

    pub fn ack(&self, message: &Message<T>) {
        if let Some(ack_tx) = self
            .in_flight
            .lock()
            .unwrap()
            .remove(&(message.src.clone(), message.body.in_reply_to))
        {
            ack_tx.send(()).ok();
        }
    }

    fn next_msg_id(&self) -> Id<usize> {
        Id::Known(Some(self.next_msg_id.lock().unwrap().fetch_add(1, AcqRel)))
    }

    async fn handle_write<W: AsyncWrite + Unpin + Send + Sync + 'static>(
        node_id: String,
        mut writer: BufWriter<W>,
        mut write_rx: mpsc::Receiver<Message<T>>,
    ) {
        while let Some(mut msg) = write_rx.recv().await {
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
