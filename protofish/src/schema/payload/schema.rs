use crate::schema::common::schema::Version;

#[derive(Debug, Clone)]
pub struct ClientHello {
    version: Version,
    resume_connection_token: Option<Vec<u8>>,
}
