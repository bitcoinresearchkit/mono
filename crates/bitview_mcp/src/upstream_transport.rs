use std::{
    error::Error,
    io::{self, Read, Write},
    pin::Pin,
    sync::Arc,
    task::{Context, Poll, ready},
    time::Duration,
};

use axum::{body::Bytes, http::Uri};
use http_body_util::Empty;
use hyper_util::{
    client::legacy::{
        Client as HttpClient,
        connect::{Connected, Connection, HttpConnector},
    },
    rt::{TokioExecutor, TokioIo, TokioTimer},
};
use rustls::{ClientConfig, ClientConnection, RootCertStore, Stream, pki_types::ServerName};
use tokio::{
    io::{AsyncRead, AsyncWrite, AsyncWriteExt, ReadBuf},
    net::TcpStream,
    time::timeout,
};
use tower::Service;

use crate::upstream::{MAX_CALLS, UPSTREAM_TIMEOUT};

pub type Client = HttpClient<Connector, Empty<Bytes>>;

pub fn client() -> Client {
    let mut http = HttpConnector::new();
    http.enforce_http(false);
    http.set_nodelay(true);
    let tls = ClientConfig::builder()
        .with_root_certificates(RootCertStore {
            roots: webpki_roots::TLS_SERVER_ROOTS.to_vec(),
        })
        .with_no_client_auth();

    HttpClient::builder(TokioExecutor::new())
        .pool_timer(TokioTimer::new())
        .pool_idle_timeout(Duration::from_secs(15))
        .pool_max_idle_per_host(MAX_CALLS)
        .build(Connector {
            http,
            tls: Arc::new(tls),
        })
}

type ConnectError = Box<dyn Error + Send + Sync>;

#[derive(Clone)]
pub struct Connector {
    http: HttpConnector,
    tls: Arc<ClientConfig>,
}

impl Service<Uri> for Connector {
    type Response = TokioIo<Socket>;
    type Error = ConnectError;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.http.poll_ready(cx).map_err(Into::into)
    }

    fn call(&mut self, uri: Uri) -> Self::Future {
        let config = Arc::clone(&self.tls);
        let connect = self.http.call(uri.clone());
        Box::pin(async move {
            // Hyper may finish a speculative connection for its pool after checkout wins.
            timeout(UPSTREAM_TIMEOUT, async {
                let mut socket = Socket {
                    tcp: connect.await?.into_inner(),
                    tls: None,
                };
                if uri.scheme_str() == Some("https") {
                    let host = uri
                        .host()
                        .expect("HTTP connector requires a host")
                        .trim_matches(['[', ']']);
                    let name = ServerName::try_from(host.to_owned())?;
                    socket.tls = Some(Box::new(ClientConnection::new(config, name)?));
                    socket.flush().await?;
                }
                Ok(TokioIo::new(socket))
            })
            .await?
        })
    }
}

pub struct Socket {
    tcp: TcpStream,
    tls: Option<Box<ClientConnection>>,
}

impl Connection for Socket {
    fn connected(&self) -> Connected {
        self.tcp.connected()
    }
}

impl AsyncRead for Socket {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        let Self { tcp, tls } = self.get_mut();
        let Some(tls) = tls else {
            return Pin::new(tcp).poll_read(cx, buf);
        };
        let mut io = SocketIo { tcp, cx };
        let read = ready!(poll_io(
            Stream::new(tls.as_mut(), &mut io).read(buf.initialize_unfilled())
        ))?;
        buf.advance(read);
        Poll::Ready(Ok(()))
    }
}

impl AsyncWrite for Socket {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        let Self { tcp, tls } = self.get_mut();
        let Some(tls) = tls else {
            return Pin::new(tcp).poll_write(cx, buf);
        };
        poll_io(Stream::new(tls.as_mut(), &mut SocketIo { tcp, cx }).write(buf))
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        let Self { tcp, tls } = self.get_mut();
        match tls {
            Some(tls) => poll_io(Stream::new(tls.as_mut(), &mut SocketIo { tcp, cx }).flush()),
            None => Pin::new(tcp).poll_flush(cx),
        }
    }

    fn poll_shutdown(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        if let Some(tls) = &mut self.tls {
            tls.send_close_notify();
            ready!(self.as_mut().poll_flush(cx))?;
        }
        Pin::new(&mut self.tcp).poll_shutdown(cx)
    }
}

// Rustls handles TLS state and buffering. Its WouldBlock corresponds to a
// Tokio socket poll that has registered this task's waker.
struct SocketIo<'a, 'cx> {
    tcp: &'a mut TcpStream,
    cx: &'a mut Context<'cx>,
}

impl Read for SocketIo<'_, '_> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let mut buf = ReadBuf::new(buf);
        sync_io(Pin::new(&mut *self.tcp).poll_read(self.cx, &mut buf))?;
        Ok(buf.filled().len())
    }
}

impl Write for SocketIo<'_, '_> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        sync_io(Pin::new(&mut *self.tcp).poll_write(self.cx, buf))
    }

    fn flush(&mut self) -> io::Result<()> {
        sync_io(Pin::new(&mut *self.tcp).poll_flush(self.cx))
    }
}

fn sync_io<T>(result: Poll<io::Result<T>>) -> io::Result<T> {
    match result {
        Poll::Ready(result) => result,
        Poll::Pending => Err(io::ErrorKind::WouldBlock.into()),
    }
}

fn poll_io<T>(result: io::Result<T>) -> Poll<io::Result<T>> {
    match result {
        Err(error) if error.kind() == io::ErrorKind::WouldBlock => Poll::Pending,
        result => Poll::Ready(result),
    }
}
