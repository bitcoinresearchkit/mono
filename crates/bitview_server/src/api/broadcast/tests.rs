use std::{collections::BTreeMap, io::{Read, Write}, net::{SocketAddr, TcpListener, TcpStream}, thread, time::Duration};

use axum::{Router, middleware::from_fn, routing::{get, post}};
use brk_rpc::{Auth, Client};
use serde_json::{Value, json};

use super::*;

enum Reply { Json(Value), Disconnect, AwaitCancellation }

fn read_request(stream: &mut TcpStream) -> Vec<u8> {
    stream.set_read_timeout(Some(Duration::from_secs(12))).unwrap();
    let mut bytes = Vec::new();
    loop {
        let mut byte = [0];
        stream.read_exact(&mut byte).unwrap();
        bytes.push(byte[0]);
        if bytes.ends_with(b"\r\n\r\n") { break; }
    }
    let headers = String::from_utf8(bytes).unwrap();
    let length: usize = headers.lines().find_map(|line| {
        let (name, value) = line.split_once(':')?;
        name.eq_ignore_ascii_case("content-length").then(|| value.trim().parse().unwrap())
    }).unwrap_or(0);
    let mut body = vec![0; length];
    stream.read_exact(&mut body).unwrap();
    body
}

fn rpc(replies: Vec<Reply>) -> (AsyncClient, thread::JoinHandle<Vec<Value>>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let url = format!("http://{}", listener.local_addr().unwrap());
    let worker = thread::spawn(move || replies.into_iter().map(|reply| {
        let (mut stream, _) = listener.accept().unwrap();
        let request: Value = serde_json::from_slice(&read_request(&mut stream)).unwrap();
        match reply {
            Reply::Json(response) => {
                let body = serde_json::to_vec(&response).unwrap();
                write!(stream, "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nContent-Type: application/json\r\nConnection: close\r\n\r\n", body.len()).unwrap();
                stream.write_all(&body).unwrap();
            }
            Reply::Disconnect => {}
            Reply::AwaitCancellation => {
                assert_eq!(stream.read(&mut [0]).unwrap(), 0, "timeout must close the active RPC");
            }
        }
        request
    }).collect());
    (Client::new(&url, Auth::None).unwrap().asynchronous().unwrap(), worker)
}

async fn server(node: AsyncClient, capacity: usize) -> (SocketAddr, Arc<Semaphore>, tokio::task::JoinHandle<()>) {
    let requests = Arc::new(Semaphore::new(capacity));
    let router = Router::new()
        .route("/api/tx", post(serve))
        .route("/api/broadcast", get(serve_query))
        .route_layer(from_fn(crate::canonical_url::refuse_respelled))
        .with_state(BroadcastState { node, requests: requests.clone() })
        .layer(from_fn(crate::request_deadline::apply))
        .layer(from_fn(crate::json_error::respond))
        .layer(from_fn(crate::response_time::respond));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let worker = tokio::spawn(async move { axum::serve(listener, router).await.unwrap(); });
    (address, requests, worker)
}

async fn request(address: SocketAddr, method: &'static str, path: &str, body: &str) -> (u16, BTreeMap<String, String>, String) {
    let wire = format!("{method} {path} HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\nIf-None-Match: *\r\nContent-Length: {}\r\n\r\n{body}", body.len());
    tokio::task::spawn_blocking(move || {
        let mut stream = TcpStream::connect(address).unwrap();
        stream.set_read_timeout(Some(Duration::from_secs(12))).unwrap();
        stream.write_all(wire.as_bytes()).unwrap();
        let mut response = String::new();
        stream.read_to_string(&mut response).unwrap();
        let (head, body) = response.split_once("\r\n\r\n").unwrap();
        let status = head.lines().next().unwrap().split_whitespace().nth(1).unwrap().parse().unwrap();
        let headers = head.lines().skip(1).map(|line| {
            let (key, value) = line.split_once(':').unwrap();
            (key.to_ascii_lowercase(), value.trim().to_owned())
        }).collect();
        (status, headers, body.to_owned())
    }).await.unwrap()
}

fn no_store(headers: &BTreeMap<String, String>) {
    assert_eq!(headers["cache-control"], "no-store");
    assert_eq!(headers["cdn-cache-control"], "no-store");
    assert!(!headers.contains_key("etag"));
}

#[tokio::test]
async fn get_and_post_share_rpc_and_never_reuse_a_submission() {
    let txid = "01".repeat(32);
    let answer = json!({"jsonrpc":"2.0", "id":1, "result":txid});
    let (node, rpc) = rpc(vec![Reply::Json(answer.clone()), Reply::Json(answer.clone()), Reply::Json(answer)]);
    let (address, permits, http) = server(node, BroadcastPermit::CAPACITY).await;
    for (method, path, body) in [("POST", "/api/tx", "00"), ("GET", "/api/broadcast?tx=00", ""), ("GET", "/api/broadcast?tx=00", "")] {
        let (status, headers, body) = request(address, method, path, body).await;
        assert_eq!(status, 200);
        assert_eq!(body, txid);
        no_store(&headers);
    }
    let calls = rpc.join().unwrap();
    assert_eq!(calls.len(), 3);
    for call in calls { assert_eq!(call, json!({"jsonrpc":"2.0", "id":1, "method":"sendrawtransaction", "params":["00"]})); }
    assert_eq!(permits.available_permits(), BroadcastPermit::CAPACITY);
    http.abort();
}

#[tokio::test]
async fn local_http_accepts_large_get_targets_without_truncating_rpc_hex() {
    let txid = "02".repeat(32);
    let answer = json!({"jsonrpc":"2.0", "id":1, "result":txid});
    let prefix = "/api/broadcast?tx=";
    // The pinned Hyper HTTP/1 parser accepts at most u16::MAX - 1 URI bytes.
    // Hex needs pairs; this is the largest raw value fitting this exact unescaped target.
    let maximum_raw = (usize::from(u16::MAX) - 1 - prefix.len()) / 2;
    let sizes = [4_096, 8_192, 16_384, maximum_raw];
    let (node, rpc) = rpc(sizes.map(|_| Reply::Json(answer.clone())).into());
    let (address, permits, http) = server(node, BroadcastPermit::CAPACITY).await;
    for bytes in sizes {
        // Only the local mock RPC accepts these artificial bytes; no real node is used.
        let hex = "00".repeat(bytes);
        let (status, headers, body) = request(address, "GET", &format!("{prefix}{hex}"), "").await;
        assert_eq!(status, 200, "raw transaction bytes: {bytes}");
        assert_eq!(body, txid);
        no_store(&headers);
    }
    let rejected = "00".repeat(maximum_raw + 1);
    let (status, _, _) = request(address, "GET", &format!("{prefix}{rejected}"), "").await;
    assert_eq!(status, 414);
    let calls = rpc.join().unwrap();
    assert_eq!(calls.len(), sizes.len());
    for (call, bytes) in calls.iter().zip(sizes) {
        assert_eq!(call["method"], "sendrawtransaction");
        assert_eq!(call["params"][0], "00".repeat(bytes));
    }
    assert_eq!(permits.available_permits(), BroadcastPermit::CAPACITY);
    http.abort();
}

#[tokio::test]
async fn rejected_inputs_head_and_overload_do_not_dispatch() {
    let (node, rpc) = rpc(Vec::new());
    let (address, permits, http) = server(node.clone(), BroadcastPermit::CAPACITY).await;
    for path in ["/api/broadcast", "/api/broadcast?tx=", "/api/broadcast?tx=0", "/api/broadcast?tx=gg", "/api/broadcast?tx=00&other=1", "/api/broadcast?tx=00&tx=00"] {
        let (status, headers, _) = request(address, "GET", path, "").await;
        assert_eq!(status, 400, "{path}");
        no_store(&headers);
    }
    let (status, _, body) = request(address, "HEAD", "/api/broadcast?tx=00", "").await;
    assert_eq!(status, 405);
    assert!(body.is_empty());
    assert!(rpc.join().unwrap().is_empty());
    assert_eq!(permits.available_permits(), BroadcastPermit::CAPACITY);
    let (overloaded, _, overloaded_http) = server(node, 0).await;
    let (status, headers, _) = request(overloaded, "GET", "/api/broadcast?tx=00", "").await;
    assert_eq!(status, 503);
    no_store(&headers);
    http.abort();
    overloaded_http.abort();
}

#[tokio::test]
async fn refusal_disconnect_and_timeout_preserve_action_semantics() {
    for (reply, expected) in [(Reply::Json(json!({"jsonrpc":"2.0", "id":1, "error":{"code":-26,"message":"fixture policy rejection"}})), 400),
                              (Reply::Disconnect, 500), (Reply::AwaitCancellation, 504)] {
        let (node, rpc) = rpc(vec![reply]);
        let (address, permits, http) = server(node, BroadcastPermit::CAPACITY).await;
        let (status, headers, body) = request(address, "GET", "/api/broadcast?tx=00", "").await;
        assert_eq!(status, expected);
        no_store(&headers);
        if status == 504 { assert!(body.contains("submission outcome may be unknown")); }
        assert_eq!(rpc.join().unwrap().len(), 1);
        assert_eq!(permits.available_permits(), BroadcastPermit::CAPACITY);
        http.abort();
    }
}
