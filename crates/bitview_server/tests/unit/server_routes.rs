use std::net::SocketAddr;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};

pub(crate) async fn exchange_with_etag(
    address: SocketAddr,
    method: &str,
    path: &str,
    etag: &str,
) -> String {
    exchange_with_limit(address, method, path, etag, 65536).await
}

async fn exchange_with_limit(
    address: SocketAddr,
    method: &str,
    path: &str,
    etag: &str,
    limit: u64,
) -> String {
    String::from_utf8(exchange_bytes(address, method, path, etag, limit).await).unwrap()
}

pub(crate) async fn exchange_bytes(
    address: SocketAddr,
    method: &str,
    path: &str,
    etag: &str,
    limit: u64,
) -> Vec<u8> {
    exchange_headers_bytes(
        address,
        method,
        path,
        &format!("If-None-Match: {etag}\r\n"),
        limit,
    )
    .await
}

pub(crate) async fn exchange_headers_bytes(
    address: SocketAddr,
    method: &str,
    path: &str,
    headers: &str,
    limit: u64,
) -> Vec<u8> {
    let mut socket = TcpStream::connect(address).await.unwrap();
    let request = format!(
        "{method} {path} HTTP/1.1\r\nHost: localhost\r\n{headers}Connection: close\r\n\r\n"
    );
    socket.write_all(request.as_bytes()).await.unwrap();
    let mut response = Vec::new();
    socket
        .take(limit + 1)
        .read_to_end(&mut response)
        .await
        .unwrap();
    assert!(
        response.len() as u64 <= limit,
        "fixture response exceeds {limit} bytes: {path}"
    );
    response
}
