use std::{
    io::{BufRead, BufReader, Read, Write},
    net::TcpListener,
    thread,
    time::Duration,
};

use serde_json::{Value, json};

use crate::{BitviewClientBase, Result};

/// One real HTTP exchange through ureq, with a bounded read and no external services.
fn exchange<T>(method: &str, status: u16, body: &str, call: impl FnOnce(BitviewClientBase) -> T) -> T {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap();
    let (method, body) = (method.to_owned(), body.to_owned());
    let server = thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        stream.set_read_timeout(Some(Duration::from_secs(3))).unwrap();
        let mut request = BufReader::new(&mut stream);
        let mut line = String::new();
        request.read_line(&mut line).unwrap();
        assert!(line.starts_with(&format!("{method} /fixture ")));
        let mut length = 0;
        loop {
            line.clear();
            request.read_line(&mut line).unwrap();
            if line == "\r\n" { break; }
            if let Some((name, value)) = line.split_once(':')
                && name.eq_ignore_ascii_case("content-length")
            { length = value.trim().parse::<usize>().unwrap(); }
        }
        request.read_exact(&mut vec![0; length]).unwrap();
        drop(request);
        write!(stream, "HTTP/1.1 {status} Test\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}", body.len()).unwrap();
    });
    let result = call(BitviewClientBase::new(format!("http://{address}")));
    server.join().unwrap();
    result
}

#[test]
fn optional_json_http_status_contract() {
    for method in ["GET", "POST"] {
        let call = |client: BitviewClientBase| -> Result<Option<Value>> {
            if method == "GET" { client.get_optional_json("/fixture") }
            else { client.post_optional_json("/fixture", "payload", "text/plain") }
        };
        assert_eq!(exchange(method, 200, r#"{"tx_id":"x"}"#, call).unwrap(), Some(json!({"tx_id":"x"})));
        assert_eq!(exchange(method, 200, "null", call).unwrap(), None);
        assert_eq!(exchange(method, 204, "", call).unwrap(), None);
        for body in ["", "{"] { assert!(exchange(method, 200, body, call).is_err()); }
        for status in [400, 404, 429, 503] {
            let error = exchange(method, status, r#"{"error":{"code":"unknown_future_code","message":"detail"}}"#, call).unwrap_err();
            assert_eq!(error.status, Some(status));
            assert_eq!(error.message, "detail");
            assert!(error.code.is_none(), "an unknown code must preserve the message");
        }
    }
    // The ordinary reader keeps its required-body contract, including when T is optional.
    for status in [200, 204] {
        assert!(exchange("GET", status, "", |client| client.get_json::<Option<Value>>("/fixture")).is_err());
    }
    assert_eq!(exchange("GET", 200, "42", |client| client.get_json::<u32>("/fixture")).unwrap(), 42);
}
