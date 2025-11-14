use bytes::Bytes;
use protofish::{
    accept, connect,
    utp::{self, UTP},
};

#[tokio::test]
async fn test_protofish() {
    let (usa, usb) = utp::mock_utp_pairs();

    tokio::spawn(server_run(usa));

    client_run(usb).await;
}

async fn client_run<U: UTP>(utp: U) {
    let conn = connect(utp.into()).await.unwrap();

    let (tx, rx) = conn.next_arb().await.unwrap();

    let muffin = rx.read().await.unwrap();
    let expected = Bytes::copy_from_slice(b"muffin");
    assert_eq!(muffin, expected);

    let to_send = [muffin.as_ref(), muffin.as_ref()].concat();
    let to_send = Bytes::from(to_send);

    tx.write(to_send).await.unwrap();
}

async fn server_run<U: UTP>(utp: U) {
    let conn = accept(utp.into()).await.unwrap();

    let sample = Bytes::copy_from_slice(b"muffin");
    let (tx, rx) = conn.new_arb();
    tx.write(sample.clone()).await.unwrap();

    let expected = [sample.as_ref(), sample.as_ref()].concat();
    let expected = Bytes::from(expected);

    let got = rx.read().await.unwrap();
    assert_eq!(expected, got);
}
