use std::sync::Arc;

use bytes::{Bytes, BytesMut};
use dashmap::DashMap;
use protofish::StreamId;
use tokio::io::{AsyncWriteExt, ReadHalf, SimplexStream, WriteHalf};

#[derive(Clone)]
pub struct DatagramRouter {
    conn: Arc<quinn::Connection>,
    channels: Arc<DashMap<StreamId, WriteHalf<SimplexStream>>>,
    pending_readers: Arc<DashMap<StreamId, ReadHalf<SimplexStream>>>,
}

impl DatagramRouter {
    pub fn new(conn: Arc<quinn::Connection>) -> Self {
        Self {
            conn,
            channels: Default::default(),
            pending_readers: Default::default(),
        }
    }

    pub fn register(&self, stream_id: StreamId) -> ReadHalf<SimplexStream> {
        if let Some((_, read_half)) = self.pending_readers.remove(&stream_id) {
            read_half
        } else {
            let (read_half, write_half) = tokio::io::simplex(1024);

            self.channels.insert(stream_id, write_half);

            read_half
        }
    }

    fn register_lazy_writer(&self, stream_id: StreamId) {
        if !self.channels.contains_key(&stream_id) {
            let (read_half, write_half) = tokio::io::simplex(1024);
            self.channels.insert(stream_id, write_half);
            self.pending_readers.insert(stream_id, read_half);
        }
    }

    pub fn write(&self, stream_id: StreamId, data: Bytes) -> crate::error::Result<()> {
        let id_bytes = stream_id.to_le_bytes();

        let mut data_bytes = BytesMut::zeroed(id_bytes.len());
        data_bytes.copy_from_slice(&id_bytes);
        data_bytes.extend(data);

        self.conn.send_datagram(data_bytes.freeze())?;
        Ok(())
    }

    async fn route_datagram(&self, stream_id: StreamId, data: &Bytes) -> crate::error::Result<()> {
        self.register_lazy_writer(stream_id);
        let Some(mut channel) = self.channels.get_mut(&stream_id) else {
            return Err(crate::error::Error::StreamClosed);
        };

        channel.write(data).await?;

        Ok(())
    }

    async fn run_listener(&self) -> crate::error::Result<()> {
        loop {
            match self.conn.read_datagram().await {
                Ok(data) => {
                    if data.len() < 8 {
                        continue;
                    }

                    let mut id_bytes = [0u8; 8];
                    id_bytes.copy_from_slice(&data[..8]);
                    let stream_id = u64::from_le_bytes(id_bytes);

                    let payload = data.slice(8..);
                    self.route_datagram(stream_id, &payload).await?;
                }
                Err(err) => {
                    break Err(crate::error::Error::from(err));
                }
            }
        }
    }

    pub fn spawn_listener(&self) {
        let self_c = self.clone();
        tokio::spawn(async move {
            loop {
                if let Err(e) = self_c.run_listener().await {
                    tracing::warn!("datagram listener error: {:?}", e);
                }
            }
        });
    }
}
