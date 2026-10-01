//! Exercise the action through the real HTTP server and a local-only Core peer.

use std::{
    net::{Ipv4Addr, SocketAddr},
    time::Duration,
};

use bitview_query::AsyncQuery;
use brk_rpc::{Auth, Client};
use serde_json::{Value, from_slice, json};
use tokio::{
    io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader},
    net::{TcpListener, TcpStream},
    spawn,
    sync::oneshot,
    time::timeout,
};

use crate::{
    Server, ServerConfig,
    api::broadcast::{BroadcastPermit, MAX_BODY_BYTES},
};

async fn exchange(
    address: SocketAddr,
    method: &str,
    path: &str,
    body: &str,
    length: usize,
) -> String {
    let mut socket = TcpStream::connect(address).await.unwrap();
    socket.write_all(format!("{method} {path} HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\nIf-None-Match: *\r\nContent-Length: {length}\r\n\r\n{body}").as_bytes()).await.unwrap();
    let mut response = String::new();
    socket.read_to_string(&mut response).await.unwrap();
    response
}

fn assert_action(response: &str, status: u16) {
    assert!(
        response.starts_with(&format!("HTTP/1.1 {status}")),
        "{response}"
    );
    assert!(
        response.contains("\r\ncache-control: no-store\r\n"),
        "{response}"
    );
    assert!(
        response.contains("\r\ncdn-cache-control: no-store\r\n"),
        "{response}"
    );
    assert!(!response.contains("\r\netag:"), "{response}");
}

async fn request(socket: &mut BufReader<TcpStream>) -> Value {
    let mut line = String::new();
    let mut length = 0;
    loop {
        line.clear();
        assert_ne!(socket.read_line(&mut line).await.unwrap(), 0);
        if line == "\r\n" {
            break;
        }
        if let Some(value) = line.to_ascii_lowercase().strip_prefix("content-length: ") {
            length = value.trim().parse::<usize>().unwrap();
        }
    }
    assert!(length < 1024);
    let mut body = vec![0; length];
    socket.read_exact(&mut body).await.unwrap();
    from_slice(&body).unwrap()
}

async fn reply(socket: &mut BufReader<TcpStream>, value: Value) {
    let body = value.to_string();
    socket
        .get_mut()
        .write_all(
            format!(
                "HTTP/1.1 200 OK\r\nContent-Length: {}\r\n\r\n{body}",
                body.len()
            )
            .as_bytes(),
        )
        .await
        .unwrap();
}

pub(crate) async fn check(query: &AsyncQuery) {
    timeout(Duration::from_secs(20), async {
        let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).await.unwrap();
        let node = Client::new(
            &format!("http://{}", listener.local_addr().unwrap()),
            Auth::None,
        )
        .unwrap()
        .asynchronous()
        .unwrap();
        let mut server = Server::bind(
            query,
            ServerConfig {
                bind: Ipv4Addr::LOCALHOST.into(),
                port: 0.into(),
                ..ServerConfig::default()
            },
        )
        .await
        .unwrap();
        server.state.node = node.clone();
        let permits = server.state.broadcast_requests.clone();
        let address = server.listener.local_addr().unwrap();
        let serving = spawn(server.serve());

        for body in ["", "0", "gg", "00 11", "éé"] {
            assert_action(
                &exchange(address, "POST", "/api/tx", body, body.len()).await,
                400,
            );
        }
        assert_action(
            &exchange(address, "POST", "/api/tx?unknown=1", "aa", 2).await,
            400,
        );
        // Reject oversized declared bodies and overload before waiting for their bytes.
        assert_action(
            &exchange(address, "POST", "/api/tx", "", MAX_BODY_BYTES + 1).await,
            413,
        );
        let held = permits
            .clone()
            .acquire_many_owned(BroadcastPermit::CAPACITY as u32)
            .await
            .unwrap();
        assert_action(&exchange(address, "POST", "/api/tx", "", 100).await, 503);
        drop(held);
        for method in ["GET", "HEAD"] {
            assert!(
                exchange(address, method, "/api/tx", "", 0)
                    .await
                    .starts_with("HTTP/1.1 405")
            );
        }
        assert!(
            timeout(Duration::from_millis(30), listener.accept())
                .await
                .is_err()
        );

        let (entered, observed) = oneshot::channel();
        let (release, released) = oneshot::channel();
        let mock = spawn(async move {
            let mut socket = BufReader::new(listener.accept().await.unwrap().0);
            for code in [0, -22, -25, -26, -27, -28] {
                let body = request(&mut socket).await;
                assert_eq!(body["method"], "sendrawtransaction");
                assert_eq!(body["params"], json!(["AA00"]));
                reply(
                    &mut socket,
                    if code == 0 {
                        json!({"id": 1, "result": "a".repeat(64)})
                    } else {
                        json!({"id": 1, "error": {"code": code, "message": "rejected"}})
                    },
                )
                .await;
            }
            assert_eq!(request(&mut socket).await["method"], "getblockcount");
            entered.send(()).unwrap();
            released.await.unwrap();
            reply(&mut socket, json!({"id": 1, "result": 0})).await;
            // The HTTP request queued behind the read timed out; it must never dispatch.
            assert_eq!(request(&mut socket).await["params"], json!(["bb"]));
            // The HTTP deadline must cancel the in-flight RPC and release admission.
            assert_eq!(socket.read(&mut [0]).await.unwrap(), 0);
            drop(socket);
            let mut socket = BufReader::new(listener.accept().await.unwrap().0);
            assert_eq!(request(&mut socket).await["params"], json!(["cc"]));
            reply(&mut socket, json!({"id":1,"result":"c".repeat(64)})).await;
        });
        let success = exchange(address, "POST", "/api/tx", " \nAA00\t", 7).await;
        assert_action(&success, 200);
        assert!(success.contains("\r\ncontent-type: text/plain; charset=utf-8\r\n"));
        assert_eq!(success.split_once("\r\n\r\n").unwrap().1, "a".repeat(64));
        for status in [400, 400, 400, 400, 500] {
            assert_action(
                &exchange(address, "POST", "/api/tx", "AA00", 4).await,
                status,
            );
        }
        let active = spawn(async move { node.get_last_height().await });
        observed.await.unwrap();
        assert_action(&exchange(address, "POST", "/api/tx", "dd", 2).await, 504);
        assert_eq!(permits.available_permits(), BroadcastPermit::CAPACITY);
        release.send(()).unwrap();
        active.await.unwrap().unwrap();
        let timed_out = exchange(address, "POST", "/api/tx", "bb", 2).await;
        assert_action(&timed_out, 504);
        assert!(timed_out.contains("submission outcome may be unknown"));
        assert_eq!(permits.available_permits(), BroadcastPermit::CAPACITY);
        assert_action(&exchange(address, "POST", "/api/tx", "cc", 2).await, 200);
        mock.await.unwrap();
        assert_eq!(permits.available_permits(), BroadcastPermit::CAPACITY);
        serving.abort();
        let _ = serving.await;
    })
    .await
    .unwrap();
}
