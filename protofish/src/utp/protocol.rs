use async_trait::async_trait;
use bytes::Bytes;

use crate::{
    schema::{common::schema::IntegrityType, payload::schema::StreamId},
    utp::error::UTPError,
};

#[async_trait]
pub trait UTPStream: Send + Sync + 'static {
    fn id(&self) -> StreamId;
    async fn send(&self, data: &Bytes) -> Result<(), UTPError>;
    async fn receive(&self, len: usize) -> Result<Bytes, UTPError>;
    async fn close(&self) -> Result<(), UTPError>;
}

#[async_trait]
pub trait UTP: Send + Sync + 'static {
    type Stream: UTPStream;

    async fn connect(&self) -> Result<(), UTPError>;
    async fn next_event(&self) -> UTPEvent;

    async fn open_stream(&self, integrity: IntegrityType) -> Result<Self::Stream, UTPError>;
    async fn wait_stream(&self, id: StreamId) -> Result<Self::Stream, UTPError>;
}

#[derive(Clone, Debug)]
pub enum UTPEvent {
    UnexpectedClose,
    NewStream(StreamId),
}
