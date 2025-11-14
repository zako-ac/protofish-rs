use bytes::Bytes;
use thiserror::Error;

use crate::{
    core::common::{
        context::{Context, ContextReader, ContextWriter},
        error::ConnectionError,
    },
    schema::{ArbitaryData, Payload},
    utp::UTPStream,
};

/// An arbitrary data context consisting of a writer and reader pair.
///
/// This type provides a simplified interface for sending and receiving
/// arbitrary binary data through a Protofish context.
pub type ArbContext<S> = (ArbContextWriter<S>, ArbContextReader);

/// Writer for arbitrary binary data within a context.
///
/// This wraps a `ContextWriter` and provides a convenient interface for
/// sending raw bytes as `ArbitaryData` payloads.
pub struct ArbContextWriter<U: UTPStream> {
    writer: ContextWriter<U>,
}

/// Reader for arbitrary binary data within a context.
///
/// This wraps a `ContextReader` and automatically extracts bytes from
/// `ArbitaryData` payloads.
pub struct ArbContextReader {
    reader: ContextReader,
}

/// Errors that can occur during arbitrary data operations.
#[derive(Debug, Error)]
pub enum ArbError {
    /// Error from the underlying connection
    #[error("connection error: {0}")]
    Connection(#[from] ConnectionError),

    /// Received a payload that was not `ArbitaryData`
    #[error("unexpected data: {0}")]
    UnexpectedData(String),
}

impl<U: UTPStream> ArbContextWriter<U> {
    /// Writes arbitrary binary data to this context.
    ///
    /// The bytes will be wrapped in an `ArbitaryData` payload and sent
    /// to the peer.
    ///
    /// # Errors
    ///
    /// Returns an error if the underlying write operation fails.
    pub async fn write(&self, content: Bytes) -> Result<(), ArbError> {
        let payload = Payload::ArbitaryData(ArbitaryData {
            content: content.into(),
        });

        self.writer.write(payload).await?;

        Ok(())
    }
}

impl ArbContextReader {
    /// Reads arbitrary binary data from this context.
    ///
    /// This method expects the next payload to be `ArbitaryData` and
    /// extracts the bytes from it.
    ///
    /// # Returns
    ///
    /// Returns the binary content, or an error if the payload is not
    /// `ArbitaryData` or the read fails.
    ///
    /// # Errors
    ///
    /// Returns `ArbError::UnexpectedData` if a non-`ArbitaryData` payload
    /// is received, or `ArbError::Connection` if the read fails.
    pub async fn read(&self) -> Result<Bytes, ArbError> {
        let data_got = self.reader.read().await?;

        if let Payload::ArbitaryData(data) = data_got {
            Ok(Bytes::from(data.content))
        } else {
            Err(ArbError::UnexpectedData("expected ArbitaryData".into()))
        }
    }
}

/// Converts a generic context into an arbitrary data context.
///
/// This helper function wraps the context writer and reader with the
/// arbitrary data interface.
pub fn make_arbitrary<S: UTPStream>((writer, reader): Context<S>) -> ArbContext<S> {
    (ArbContextWriter { writer }, ArbContextReader { reader })
}
