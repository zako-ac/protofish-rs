use std::sync::Arc;

use bytes::Bytes;
use tokio::sync::{Mutex, mpsc::Receiver};

use crate::{
    core::common::{context::Context, error::ConnectionError},
    schema::{
        common::schema::{IntegrityType, StreamCreateMeta},
        payload::schema::{ArbitaryData, Payload, StreamOpen},
    },
    utp::protocol::{UTP, UTPStream},
};

pub struct ArbitraryContext<U>
where
    U: UTP,
{
    context: Context<U::Stream>,
    utp: Arc<U>,

    message_rx: Mutex<Receiver<Bytes>>,
    stream_rx: Mutex<Receiver<StreamOpen>>,
}

impl<U: UTP> ArbitraryContext<U> {
    pub async fn send_message(&self, data: impl AsRef<[u8]>) -> Result<(), ConnectionError> {
        let (ref tx, _) = self.context;

        tx.write(Payload::ArbitaryData(ArbitaryData {
            content: Vec::from(data.as_ref()),
        }))
        .await?;

        Ok(())
    }

    pub async fn recv_message(&self) -> Result<Bytes, ConnectionError> {
        if let Some(msg) = self.message_rx.lock().await.recv().await {
            Ok(msg)
        } else {
            Err(ConnectionError::ClosedStream)
        }
    }

    pub async fn create_stream(
        &self,
        integrity_type: IntegrityType,
    ) -> Result<U::Stream, ConnectionError> {
        // TODO: change stream to some AsyncRead wrapper
        let (ref tx, _) = self.context;

        let stream = self.utp.open_stream(integrity_type.clone()).await?;

        let stream_open = StreamOpen {
            stream_id: stream.id(),
            meta: StreamCreateMeta { integrity_type },
        };

        tx.write(Payload::StreamOpen(stream_open)).await?;

        Ok(stream)
    }

    pub async fn next_stream(&self) -> Result<U::Stream, ConnectionError> {
        if let Some(meta) = self.stream_rx.lock().await.recv().await {
            let stream = self.utp.wait_stream(meta.stream_id).await?;
            Ok(stream)
        } else {
            Err(ConnectionError::ClosedStream)
        }
    }
}
