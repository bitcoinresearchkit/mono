use std::{
    future::{Future, poll_fn},
    pin::Pin as FuturePin,
    task::Poll,
    time::Duration,
};

use brk_error::{Error, Result};
use http_body_util::{BodyExt, Full, Limited};
use hyper::{
    Request, StatusCode,
    body::Bytes,
    client::conn::http1::{self, Connection as HttpConnection, SendRequest},
};
use hyper_util::rt::TokioIo;
use tokio::{net::TcpStream, pin, select, time::timeout};

use crate::{endpoint::Endpoint, rpc_response::MAX_RESPONSE_BYTES};

pub struct Connection {
    sender: SendRequest<Full<Bytes>>,
    driver: Option<HttpConnection<TokioIo<TcpStream>, Full<Bytes>>>,
}

pub enum Exchange {
    Complete(StatusCode, Bytes),
    Disconnected,
}

impl Connection {
    pub async fn connect(endpoint: &Endpoint) -> Result<Self> {
        let socket = timeout(
            Duration::from_secs(3),
            TcpStream::connect((endpoint.host.as_str(), endpoint.port)),
        )
        .await
        .map_err(|_| Error::Internal("node RPC connection timed out"))??;
        let (sender, driver) = http1::handshake(TokioIo::new(socket))
            .await
            .map_err(|_| Error::Internal("node RPC handshake failed"))?;
        Ok(Self {
            sender,
            driver: Some(driver),
        })
    }

    pub async fn is_closed(&mut self) -> bool {
        if let Some(driver) = &mut self.driver {
            let finished =
                poll_fn(|cx| Poll::Ready(FuturePin::new(&mut *driver).poll(cx).is_ready())).await;
            if finished || self.sender.is_closed() {
                self.driver = None;
            }
        }
        self.driver.is_none()
    }

    pub async fn request(&mut self, request: Request<Full<Bytes>>) -> Result<Exchange> {
        let Some(mut driver) = self.driver.take() else {
            return Ok(Exchange::Disconnected);
        };
        let response = async {
            let response = match self.sender.send_request(request).await {
                Ok(response) => response,
                Err(error) if error.is_parse() || error.is_user() => {
                    return Err(Error::Internal("invalid node RPC HTTP exchange"));
                }
                Err(_) => return Ok(Exchange::Disconnected),
            };
            let status = response.status();
            let bytes = Limited::new(response.into_body(), MAX_RESPONSE_BYTES)
                .collect()
                .await
                .map_err(|_| Error::Internal("node RPC response unreadable or too large"))?
                .to_bytes();
            Ok(Exchange::Complete(status, bytes))
        };
        pin!(response);
        select! {
            biased;
            result = &mut response => {
                self.driver = Some(driver);
                result
            },
            _ = &mut driver => {
                // Dropping a finished driver releases any pending request notification.
                drop(driver);
                response.await
            },
        }
    }
}
