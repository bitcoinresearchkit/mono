use serde_json::json;
use tokio::spawn;

use super::*;

async fn reply(socket: &mut BufReader<TcpStream>, body: Value) {
    let body = body.to_string();
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

#[tokio::test]
async fn submissions_never_replay_after_response_loss() {
    for reused in [false, true] {
        let (client, listener) = client().await;
        let server = spawn(async move {
            let mut socket = BufReader::new(listener.accept().await.unwrap().0);
            if reused {
                request(&mut socket).await;
                reply(&mut socket, json!({"id": 1, "result": 0})).await;
            }
            let (body, _) = message(&mut socket).await;
            assert_eq!(body["method"], "sendrawtransaction");
            drop(socket); // The node received the action, but its outcome is lost.
            assert!(
                timeout(Duration::from_millis(100), listener.accept())
                    .await
                    .is_err()
            );
        });
        timeout(Duration::from_secs(5), async {
            if reused {
                client.get_last_height().await.unwrap();
            }
            assert!(client.send_raw_transaction("aa").await.is_err());
            server.await.unwrap();
        })
        .await
        .unwrap();
    }
}

#[tokio::test]
async fn cancelled_waiter_cannot_submit_after_the_connection_is_released() {
    let (client, listener) = client().await;
    let (entered, observed) = oneshot::channel();
    let (release, released) = oneshot::channel();
    let server = spawn(async move {
        let mut socket = BufReader::new(listener.accept().await.unwrap().0);
        request(&mut socket).await;
        entered.send(()).unwrap();
        released.await.unwrap();
        reply(&mut socket, json!({"id": 1, "result": 0})).await;
        let (body, _) = message(&mut socket).await;
        assert_eq!(
            body["params"],
            json!(["bb"]),
            "cancelled aa must never dispatch"
        );
        reply(&mut socket, json!({"id": 1, "result": "b".repeat(64)})).await;
    });
    let active = spawn({
        let client = client.clone();
        async move { client.get_last_height().await }
    });
    timeout(Duration::from_secs(5), async {
        observed.await.unwrap();
        assert!(
            timeout(Duration::from_millis(30), client.send_raw_transaction("aa"))
                .await
                .is_err()
        );
        release.send(()).unwrap();
        active.await.unwrap().unwrap();
        assert_eq!(
            client.send_raw_transaction("bb").await.unwrap().to_string(),
            "b".repeat(64)
        );
        server.await.unwrap();
    })
    .await
    .unwrap();
}
