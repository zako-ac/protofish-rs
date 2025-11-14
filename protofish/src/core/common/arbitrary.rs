use bytes::Bytes;
use thiserror::Error;

use crate::{
    core::common::{
        context::{Context, ContextReader, ContextWriter},
        error::ConnectionError,
    },
    schema::payload::schema::{ArbitaryData, Payload},
    utp::UTPStream,
};

pub type ArbContext<S> = (ArbContextWriter<S>, ArbContextReader);

pub struct ArbContextWriter<U: UTPStream> {
    writer: ContextWriter<U>,
}

pub struct ArbContextReader {
    reader: ContextReader,
}

#[derive(Debug, Error)]
pub enum ArbError {
    #[error("connection error: {0}")]
    Connection(#[from] ConnectionError),

    #[error("unexpected data: {0}")]
    UnexpectedData(String),
}

impl<U: UTPStream> ArbContextWriter<U> {
    pub async fn write(&self, content: Bytes) -> Result<(), ArbError> {
        let payload = Payload::ArbitaryData(ArbitaryData {
            content: content.into(),
        });

        self.writer.write(payload).await?;

        Ok(())
    }
}

impl ArbContextReader {
    pub async fn read(&self) -> Result<Bytes, ArbError> {
        let data_got = self.reader.read().await?;

        if let Payload::ArbitaryData(data) = data_got {
            Ok(Bytes::from(data.content))
        } else {
            Err(ArbError::UnexpectedData("expected ArbitaryData".into()))
        }
    }
}

pub fn make_arbitrary<S: UTPStream>((writer, reader): Context<S>) -> ArbContext<S> {
    (ArbContextWriter { writer }, ArbContextReader { reader })
}
