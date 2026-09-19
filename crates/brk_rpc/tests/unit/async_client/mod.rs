use std::{fs, time::Duration};

use serde_json::{Value, from_slice};
use tempfile::tempdir;
use tokio::{
    io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader},
    net::{TcpListener, TcpStream},
    spawn,
    sync::oneshot,
    time::timeout,
};

use super::AsyncClient;
use crate::Auth;

mod broadcast;

async fn request(socket: &mut BufReader<TcpStream>) -> (Value, String) {
    let (body, authorization) = message(socket).await;
    assert_eq!(body["method"], "getblockcount");
    (body["id"].clone(), authorization)
}

async fn message(socket: &mut BufReader<TcpStream>) -> (Value, String) {
    let mut line = String::new();
    let mut length = 0;
    let mut authorization = String::new();
    loop {
        line.clear();
        assert_ne!(socket.read_line(&mut line).await.unwrap(), 0);
        if line == "\r\n" {
            break;
        }
        if let Some(value) = line.to_ascii_lowercase().strip_prefix("content-length: ") {
            length = value.trim().parse::<usize>().unwrap();
        }
        if line.to_ascii_lowercase().starts_with("authorization: ") {
            authorization = line[15..].trim().to_owned();
        }
    }
    assert!(length < 1024);
    let mut body = vec![0; length];
    socket.read_exact(&mut body).await.unwrap();
    let body: Value = from_slice(&body).unwrap();
    (body, authorization)
}

#[tokio::test]
async fn reconnects_when_the_node_closes_an_idle_socket() {
    let (client, listener) = client().await;
    let (close, close_requested) = oneshot::channel();
    let (closed, closure) = oneshot::channel();
    let server = spawn(async move {
        let mut socket = BufReader::new(listener.accept().await.unwrap().0);
        request(&mut socket).await;
        let response = b"HTTP/1.1 200 OK\r\nContent-Length: 19\r\n\r\n{\"id\":1,\"result\":0}";
        socket.get_mut().write_all(response).await.unwrap();
        close_requested.await.unwrap();
        drop(socket);
        closed.send(()).unwrap();
        let mut socket = BufReader::new(listener.accept().await.unwrap().0);
        request(&mut socket).await;
        socket.get_mut().write_all(response).await.unwrap();
    });
    timeout(Duration::from_secs(5), async {
        assert_eq!(*client.get_last_height().await.unwrap(), 0);
        close.send(()).unwrap();
        closure.await.unwrap();
        assert_eq!(*client.get_last_height().await.unwrap(), 0);
        server.await.unwrap();
    })
    .await
    .unwrap();
}

#[tokio::test]
async fn refreshes_rotated_cookie_once_after_rejection() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let directory = tempdir().unwrap();
    let path = directory.path().join(".cookie");
    fs::write(&path, "u:p\n").unwrap();
    let client = AsyncClient::new(
        &format!("http://{}", listener.local_addr().unwrap()),
        Auth::CookieFile(path.clone()),
    )
    .unwrap();
    let server = spawn(async move {
        let mut socket = BufReader::new(listener.accept().await.unwrap().0);
        for (expected, rejected) in [
            ("Basic dTpw", false),
            ("Basic dTpw", true),
            ("Basic dTpx", false),
            ("Basic dTpx", false),
            ("Basic dTpx", true),
        ] {
            let (_, auth) = request(&mut socket).await;
            assert_eq!(auth, expected);
            let response: &[u8] = if rejected {
                b"HTTP/1.1 401 Unauthorized\r\nContent-Length: 0\r\n\r\n"
            } else {
                b"HTTP/1.1 200 OK\r\nContent-Length: 19\r\n\r\n{\"id\":1,\"result\":0}"
            };
            socket.get_mut().write_all(response).await.unwrap();
        }
    });
    timeout(Duration::from_secs(5), async {
        for cookie in ["u:p\n", "u:q\n"] {
            fs::write(&path, cookie).unwrap();
            assert_eq!(*client.get_last_height().await.unwrap(), 0);
        }
        assert_eq!(*client.get_last_height().await.unwrap(), 0);
        assert!(client.get_last_height().await.is_err());
        server.await.unwrap();
    })
    .await
    .unwrap();
}

async fn client() -> (AsyncClient, TcpListener) {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let url = format!("http://{}", listener.local_addr().unwrap());
    (AsyncClient::new(&url, Auth::None).unwrap(), listener)
}

#[tokio::test]
async fn validates_responses_and_reuses_connection() {
    let (client, listener) = client().await;
    let responses = [
        (
            200,
            r#"{"jsonrpc":"2.0","id":1,"result":900000}"#.to_owned(),
        ),
        (
            200,
            r#"{"jsonrpc":"2.0","id":2,"result":900000}"#.to_owned(),
        ),
        (
            200,
            r#"{"jsonrpc":"3.0","id":1,"result":900000}"#.to_owned(),
        ),
        (200, r#"{"id":1,"result":4294967296}"#.to_owned()),
        (200, r#"{"id":1,"result":-1}"#.to_owned()),
        (500, r#"{"id":1,"result":900000}"#.to_owned()),
        (
            200,
            r#"{"id":1,"error":{"code":-28,"message":"warming up"}}"#.to_owned(),
        ),
        (200, "invalid json".to_owned()),
        (
            200,
            format!("{{\"id\":1,\"result\":900000}}{}", " ".repeat(65537)),
        ),
    ];
    let count = responses.len();
    let server = spawn(async move {
        // All requests must use this one accepted socket.
        let mut socket = BufReader::new(listener.accept().await.unwrap().0);
        for (status, body) in responses {
            request(&mut socket).await;
            let response = format!(
                "HTTP/1.1 {status} Test\r\nContent-Length: {}\r\n\r\n{body}",
                body.len()
            );
            socket
                .get_mut()
                .write_all(response.as_bytes())
                .await
                .unwrap();
        }
        // The oversized response must discard this socket, not return it to reuse.
        let mut byte = [0];
        assert_eq!(socket.read(&mut byte).await.unwrap(), 0);
        let mut socket = BufReader::new(listener.accept().await.unwrap().0);
        request(&mut socket).await;
        socket
            .get_mut()
            .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 19\r\n\r\n{\"id\":1,\"result\":0}")
            .await
            .unwrap();
    });
    timeout(Duration::from_secs(5), async {
        assert_eq!(*client.get_last_height().await.unwrap(), 900000);
        for _ in 1..count {
            assert!(client.get_last_height().await.is_err());
        }
        assert_eq!(*client.get_last_height().await.unwrap(), 0);
        server.await.unwrap();
    })
    .await
    .unwrap();
}

#[tokio::test]
async fn cancellation_closes_stalled_header_and_body_connections() {
    for partial in ["", "HTTP/1.1 200 OK\r\nContent-Length: 100\r\n\r\n{"] {
        let (client, listener) = client().await;
        let (closed, closure) = oneshot::channel();
        let server = spawn(async move {
            let mut socket = BufReader::new(listener.accept().await.unwrap().0);
            request(&mut socket).await;
            socket
                .get_mut()
                .write_all(partial.as_bytes())
                .await
                .unwrap();
            let mut byte = [0];
            assert_eq!(socket.read(&mut byte).await.unwrap(), 0);
            closed.send(()).unwrap();
            let mut socket = BufReader::new(listener.accept().await.unwrap().0);
            request(&mut socket).await;
            socket
                .get_mut()
                .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 19\r\n\r\n{\"id\":1,\"result\":0}")
                .await
                .unwrap();
        });
        assert!(
            timeout(Duration::from_millis(100), client.get_last_height())
                .await
                .is_err()
        );
        // The client remains alive: cancellation, not dropping its pool, must close the socket.
        timeout(Duration::from_secs(2), async {
            closure.await.unwrap();
            assert_eq!(*client.get_last_height().await.unwrap(), 0);
            server.await.unwrap();
        })
        .await
        .unwrap();
    }
}

#[tokio::test]
async fn stops_after_one_stale_connection_retry() {
    let (client, listener) = client().await;
    let server = spawn(async move {
        let mut socket = BufReader::new(listener.accept().await.unwrap().0);
        request(&mut socket).await;
        socket
            .get_mut()
            .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 19\r\n\r\n{\"id\":1,\"result\":0}")
            .await
            .unwrap();
        request(&mut socket).await;
        drop(socket);
        let mut socket = BufReader::new(listener.accept().await.unwrap().0);
        request(&mut socket).await;
        drop(socket);
        // Keep the listener alive: an unwanted third attempt must not fail merely
        // because this fixture has stopped listening.
        listener
    });
    timeout(Duration::from_secs(5), async {
        assert_eq!(*client.get_last_height().await.unwrap(), 0);
        assert!(client.get_last_height().await.is_err());
        let listener = server.await.unwrap();
        assert!(
            timeout(Duration::from_millis(100), listener.accept())
                .await
                .is_err()
        );
    })
    .await
    .unwrap();
}

#[tokio::test]
async fn reconnects_after_close_delimited_and_chunked_responses() {
    for response in [
        "HTTP/1.1 200 OK\r\nConnection: close\r\nContent-Length: 19\r\n\r\n{\"id\":1,\"result\":0}",
        "HTTP/1.1 200 OK\r\nConnection: close\r\nTransfer-Encoding: chunked\r\n\r\n13\r\n{\"id\":1,\"result\":0}\r\n0\r\n\r\n",
        "HTTP/1.0 200 OK\r\n\r\n{\"id\":1,\"result\":0}",
    ] {
        let (client, listener) = client().await;
        let server = spawn(async move {
            for _ in 0..2 {
                let mut socket = BufReader::new(listener.accept().await.unwrap().0);
                request(&mut socket).await;
                socket
                    .get_mut()
                    .write_all(response.as_bytes())
                    .await
                    .unwrap();
            }
        });
        timeout(Duration::from_secs(5), async {
            for attempt in 0..2 {
                let height = client.get_last_height().await;
                assert!(
                    height.is_ok(),
                    "attempt={attempt} response={response:?} result={height:?}"
                );
                assert_eq!(*height.unwrap(), 0);
            }
            server.await.unwrap();
        })
        .await
        .unwrap();
    }
}
