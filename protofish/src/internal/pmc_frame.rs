use std::sync::Arc;

use bytes::Bytes;
use dashmap::DashMap;
use tokio::{
    sync::{
        Mutex, Notify,
        mpsc::{self, Receiver, Sender},
    },
    task::JoinHandle,
};

use crate::{
    constant::CHANNEL_BUFFER,
    internal::serialize::{deserialize_message, serialize_message},
    schema::payload::schema::{ContextId, Message, Payload},
    utp::{error::UTPError, protocol::UTPStream},
};

pub struct PMCFrame<S: UTPStream> {
    utp_stream: Arc<S>,
    senders: Arc<DashMap<ContextId, Sender<Payload>>>,
    context_rx: Mutex<Receiver<Message>>,
    shutdown_notify: Arc<Notify>,
    _task: JoinHandle<()>,
}

impl<S: UTPStream> PMCFrame<S> {
    pub fn new(stream: Arc<S>) -> Self {
        let senders: Arc<DashMap<ContextId, Sender<Payload>>> = Default::default();
        let (context_tx, context_rx) = mpsc::channel(CHANNEL_BUFFER);
        let shutdown_notify = Arc::new(Notify::new());

        let _task = {
            let stream = stream.clone();
            let senders = senders.clone();
            let notify = shutdown_notify.clone();

            tokio::spawn(async move {
                loop {
                    tokio::select! {
                        _ = notify.notified() => {
                            break;
                        }
                        success = match_frame(stream.clone(), senders.clone(), context_tx.clone()) => {
                            if !success {break;}
                        }
                    }
                }
            })
        };

        Self {
            utp_stream: stream,
            senders,
            context_rx: Mutex::new(context_rx),
            shutdown_notify,
            _task,
        }
    }

    pub fn subscribe_context(&self, context_id: ContextId) -> Receiver<Payload> {
        let (tx, rx) = mpsc::channel(CHANNEL_BUFFER);

        self.senders.insert(context_id, tx);

        rx
    }

    pub async fn next_context_message(&self) -> Option<Message> {
        self.context_rx.lock().await.recv().await
    }

    pub async fn send(&self, message: Message) -> Result<(), UTPError> {
        send_frame(self.utp_stream.as_ref(), message).await
    }
}

async fn match_frame(
    stream: Arc<impl UTPStream>,
    senders: Arc<DashMap<ContextId, Sender<Payload>>>,
    context_tx: Sender<Message>,
) -> bool {
    match recv_frame(stream).await {
        Ok(message_option) => {
            if let Some(message) = message_option {
                if let Some(sender) = senders.get(&message.context_id) {
                    tokio::spawn({
                        let sender = sender.clone();
                        async move {
                            if let Err(e) = sender.send(message.payload).await {
                                tracing::warn!("UTP error while sending to the channel: {:?}", e);
                            }
                        }
                    });
                } else {
                    tokio::spawn({
                        let context_tx = context_tx.clone();
                        async move {
                            if let Err(e) = context_tx.send(message).await {
                                tracing::warn!("Send error while sending to the channel: {:?}", e);
                            }
                        }
                    });
                }

                true
            } else {
                false
            }
        }
        Err(UTPError::Fatal(e)) => {
            tracing::error!("UTP receive failure: {}", e);
            false
        }
        Err(UTPError::Warn(e)) => {
            tracing::warn!("UTP receive warn: {}", e);
            true
        }
    }
}

pub async fn send_frame<S: UTPStream>(stream: &S, message: Message) -> Result<(), UTPError> {
    let buf = serialize_message(message);

    let len: u64 = buf.len() as u64;
    let len_bytes = len.to_le_bytes();
    let len_bytes = Bytes::copy_from_slice(&len_bytes);

    stream.send(&len_bytes).await?;
    stream.send(&buf).await?;

    Ok(())
}

async fn recv_frame<S: UTPStream>(stream: Arc<S>) -> Result<Option<Message>, UTPError> {
    let len_bytes_slice = stream.receive(8).await?;

    let len = u64::from_le_bytes(len_bytes_slice[..].try_into().unwrap());

    let buf = stream.receive(len as usize).await?;

    let message = deserialize_message(&buf);

    Ok(message)
}
