use std::time::Duration;

use criterion::{Criterion, Throughput, criterion_group, criterion_main};
use protofish::{
    IntegrityType, accept, connect,
    utp::{self, UTP},
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    task::yield_now,
    time::sleep,
};

async fn client_run<U: UTP>(utp: U, data: Vec<u8>) {
    let conn = connect(utp.into()).await.unwrap();
    let arb = conn.new_arb();
    let mut stream = arb.new_stream(IntegrityType::Reliable).await.unwrap();

    println!("1");
    stream.write_all(&data).await.unwrap();
    println!("2");

    yield_now().await;

    let mut got = vec![0u8; data.len()];
    stream.read_exact(&mut got).await.unwrap();
    println!("3");
}

async fn server_run<U: UTP>(utp: U, size: usize) {
    let conn = accept(utp.into()).await.unwrap();
    let arb = conn.next_arb().await.unwrap();
    let mut stream = arb.wait_stream().await.unwrap();

    let mut got = vec![0u8; size];
    println!("a1");
    stream.read_exact(&mut got).await.unwrap();
    println!("a2");
    tokio::task::yield_now().await;
    stream.write_all(&got).await.unwrap();
    tokio::task::yield_now().await;
    println!("a3");
}

fn benchmark_throughput(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    for size in [1024, 4096, 16384, 65536, 262144] {
        let mut group = c.benchmark_group(format!("throughput_{}_bytes", size));
        group.throughput(Throughput::Bytes((size * 2) as u64)); // Both directions

        group.bench_function("protofish", |b| {
            b.to_async(&rt).iter(|| async {
                let data = vec![0u8; size];
                let (usa, usb) = utp::mock_utp_pairs();

                let server_handle = tokio::spawn(async move {
                    server_run(usa, size).await;
                });

                client_run(usb, data).await;
                server_handle.await.unwrap();
            });
        });

        group.finish();
    }
}

criterion_group!(benches, benchmark_throughput);
criterion_main!(benches);
