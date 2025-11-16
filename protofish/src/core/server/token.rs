use rand::{RngCore, rng};

const CONNECTION_TOKEN_LEN: usize = 32;

pub fn generate_connection_token() -> Vec<u8> {
    let mut buf = vec![0u8; CONNECTION_TOKEN_LEN];

    rng().fill_bytes(&mut buf);

    buf
}
