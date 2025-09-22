use thiserror::Error;

use crate::{schema::payload::schema::Payload, utp::error::UTPError};

#[derive(Error, Debug)]
pub enum ConnectionError {
    #[error("UTP error: {0}")]
    UTP(#[from] UTPError),

    #[error("stream closed")]
    ClosedStream,

    #[error("handshake rejected: {0}")]
    HandshakeReject(String),

    #[error("malformed data: {0}")]
    MalformedData(String),

    #[error("malformed payload: {0} {1:?}")]
    MalformedPayload(String, Payload),
}
