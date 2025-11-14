use bytes::Bytes;
use dashmap::DashMap;
use std::sync::{Arc, Weak};
use tokio::sync::mpsc;

use protofish::StreamId;

pub struct DatagramRouter {
    channels: DashMap<StreamId, mpsc::UnboundedSender<Bytes>>,
    pending_receivers: DashMap<StreamId, mpsc::UnboundedReceiver<Bytes>>,
    connection: Weak<quinn::Connection>,
}

impl DatagramRouter {
    pub fn new(connection: Weak<quinn::Connection>) -> Arc<Self> {
        Arc::new(Self {
            channels: DashMap::new(),
            pending_receivers: DashMap::new(),
            connection,
        })
    }

    pub fn register_stream(&self, id: StreamId) -> mpsc::UnboundedReceiver<Bytes> {
        if let Some((_, rx)) = self.pending_receivers.remove(&id) {
            rx
        } else {
            let (tx, rx) = mpsc::unbounded_channel();
            self.channels.insert(id, tx);
            rx
        }
    }

    pub async fn unregister_stream(&self, id: StreamId) {
        self.channels.remove(&id);
    }

    pub async fn send_datagram(&self, id: StreamId, data: &Bytes) -> crate::error::Result<()> {
        let conn = self
            .connection
            .upgrade()
            .ok_or(crate::error::Error::NotConnected)?;

        let datagram = Self::encode_datagram(id, data);
        conn.send_datagram(datagram)
            .map_err(|e| crate::error::Error::Datagram(e.to_string()))?;

        Ok(())
    }

    pub async fn route_incoming(&self, datagram: Bytes) {
        if let Some((stream_id, payload)) = Self::decode_datagram(datagram) {
            if let Some(tx) = self.channels.get(&stream_id) {
                let _ = tx.send(payload);
            } else {
                let (tx, rx) = mpsc::unbounded_channel();
                self.pending_receivers.insert(stream_id, rx);

                let _ = tx.send(payload);
            }
        }
    }

    fn encode_datagram(id: StreamId, data: &Bytes) -> Bytes {
        let mut buf = Vec::with_capacity(8 + data.len());
        buf.extend_from_slice(&id.to_be_bytes());
        buf.extend_from_slice(data);
        Bytes::from(buf)
    }

    fn decode_datagram(datagram: Bytes) -> Option<(StreamId, Bytes)> {
        if datagram.len() < 8 {
            return None;
        }

        let mut id_bytes = [0u8; 8];
        id_bytes.copy_from_slice(&datagram[..8]);
        let stream_id = u64::from_be_bytes(id_bytes);

        let payload = datagram.slice(8..);
        Some((stream_id, payload))
    }

    pub fn spawn_listener(self: Arc<Self>, conn: quinn::Connection) {
        tokio::spawn(async move {
            loop {
                match conn.read_datagram().await {
                    Ok(data) => {
                        self.route_incoming(data).await;
                    }
                    Err(_) => break,
                }
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_datagram_encoding() {
        let id: StreamId = 12345;
        let data = Bytes::from_static(b"hello world");

        let encoded = DatagramRouter::encode_datagram(id, &data);
        let (decoded_id, decoded_data) = DatagramRouter::decode_datagram(encoded).unwrap();

        assert_eq!(decoded_id, id);
        assert_eq!(decoded_data, data);
    }

    #[test]
    fn test_datagram_decode_too_short() {
        let short_data = Bytes::from_static(&[1, 2, 3]);
        assert!(DatagramRouter::decode_datagram(short_data).is_none());
    }
}
