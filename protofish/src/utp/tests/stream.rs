use std::sync::Arc;

use async_trait::async_trait;
use tokio::sync::{Mutex, Notify};

use crate::{
    IntegrityType,
    schema::StreamId,
    utp::{error::UTPError, protocol::UTPStream},
};

#[derive(Clone)]
pub struct MockUTPStream {
    pub id: StreamId,
    peer: Option<Arc<MockUTPStream>>,
    buffer: Arc<Mutex<Vec<u8>>>,
    notify: Arc<Notify>,
}

impl MockUTPStream {
    pub fn new(id: StreamId, peer: Option<MockUTPStream>) -> Self {
        Self {
            id,
            peer: peer.map(Arc::new),
            buffer: Default::default(),
            notify: Notify::new().into(),
        }
    }

    pub fn set_peer(&mut self, peer: Option<MockUTPStream>) {
        self.peer = peer.map(Arc::new);
    }

    pub(self) async fn write_buffer(&self, data: &[u8]) {
        self.buffer.lock().await.extend_from_slice(data);
        self.notify.notify_waiters();
    }

    pub(self) async fn read_buffer(&self, n: usize) -> Vec<u8> {
        loop {
            let mut buf = self.buffer.lock().await;
            if buf.len() >= n {
                let result = buf.drain(..n).collect();
                return result;
            }
            drop(buf);
            self.notify.notified().await;
        }
    }
}

#[async_trait]
impl UTPStream for MockUTPStream {
    fn id(&self) -> StreamId {
        self.id
    }

    fn integrity_type(&self) -> IntegrityType {
        IntegrityType::Reliable
    }

    async fn send(&self, data: &[u8]) -> Result<(), UTPError> {
        if let Some(peer) = self.peer.as_ref() {
            peer.write_buffer(data).await;
        }
        Ok(())
    }

    async fn receive(&self, data: &mut [u8]) -> Result<(), UTPError> {
        let d: Vec<u8> = self.read_buffer(data.len()).await;

        data[..d.len()].copy_from_slice(&d);

        Ok(())
    }

    async fn close(&self) -> Result<(), UTPError> {
        self.buffer.lock().await.clear();
        Ok(())
    }
}

// legacy compat
#[cfg(test)]
pub fn mock_pairs() -> (MockUTPStream, MockUTPStream) {
    let mut x = MockUTPStream::new(0, None);
    let y = MockUTPStream::new(1, Some(x.clone()));
    x.set_peer(Some(y.clone()));

    (x, y)
}

pub fn mock_utp_stream_pairs(id: StreamId) -> (MockUTPStream, MockUTPStream) {
    let mut x = MockUTPStream::new(id, None);
    let y = MockUTPStream::new(id, Some(x.clone()));
    x.set_peer(Some(y.clone()));

    (x, y)
}

#[cfg(test)]
mod tests {

    use crate::utp::{protocol::UTPStream, tests::stream::mock_pairs};

    #[tokio::test]
    async fn test_mock_pairs_atob() {
        let (a, b) = mock_pairs();

        a.send(&vec![0; 14].iter().map(|e| e + 1).collect::<Vec<_>>())
            .await
            .unwrap();

        let mut b_data = vec![0; 14];
        b.receive(&mut b_data).await.unwrap();

        assert_eq!(b_data[13], 1);
    }

    #[tokio::test]
    async fn test_mock_pairs_btoa() {
        let (a, b) = mock_pairs();

        b.send(&vec![0; 14].iter().map(|e| e + 1).collect::<Vec<_>>())
            .await
            .unwrap();

        let mut a_data = vec![0u8; 14];
        a.receive(&mut a_data).await.unwrap();

        assert_eq!(a_data[13], 1);
    }
}
