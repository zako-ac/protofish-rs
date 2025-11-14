use std::sync::Arc;

use crate::{
    core::{
        common::{connection::Connection, error::ConnectionError, pmc::PMC},
        server::handshake::server_handshake,
    },
    error::ProtofishError,
    utp::{UTP, UTPEvent},
};

pub async fn accept<U>(utp: Arc<U>) -> Result<Connection<U>, ProtofishError>
where
    U: UTP,
{
    let event = utp.next_event().await; // TODO maybe timeout

    if let UTPEvent::NewStream(id) = event {
        let stream = utp.wait_stream(id).await?;
        let pmc = PMC::new(true, stream);

        server_handshake(&pmc).await?;

        Ok(Connection::new(pmc))
    } else {
        Err(ConnectionError::ClosedStream.into())
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        constant::VERSION,
        core::{common::pmc::PMC, server::accept},
        schema::{
            common::schema::IntegrityType,
            payload::schema::{ClientHello, Payload},
        },
        utp::{UTP, tests::utp::mock_utp_pairs},
    };

    async fn imitate_handshake(resume_connection_token: Option<Vec<u8>>, assert_ok: bool) {
        let (a, b) = mock_utp_pairs();

        tokio::spawn(async move {
            let stream = b.open_stream(IntegrityType::Reliable).await.unwrap();
            let pmc = PMC::new(false, stream);

            let (tx, rx) = pmc.create_context();

            let client_hello = ClientHello {
                version: VERSION,
                resume_connection_token,
            };

            tx.write(Payload::ClientHello(client_hello)).await.unwrap();
            let r = rx.read().await.unwrap();

            if let Payload::ServerHello(server_hello) = r {
                assert_eq!(server_hello.ok, assert_ok);
            } else {
                panic!("Expected ServerHello. Malformed req: {:?}", r);
            }
        });

        accept(a.into()).await.unwrap();
    }

    #[tokio::test]
    async fn test_server_accept_ok() {
        imitate_handshake(None, true).await;
    }

    #[tokio::test]
    async fn test_server_accept_fail() {
        imitate_handshake(Some(vec![]), false).await;
    }
}
