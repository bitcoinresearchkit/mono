//! A connection that takes no byte for [`SEND_STALL`] is closed. A client that stops reading would
//! otherwise hold its response, and the admission a large one keeps until sent, for as long as it
//! likes. It's enforced on the socket: while one is blocked, no one polls the response's body.

use std::{
    future::Future,
    io::{self, IoSlice},
    net::SocketAddr,
    pin::Pin,
    task::{Context, Poll},
    time::Duration,
};

use axum::serve::Listener;
use tokio::{
    io::{AsyncRead, AsyncWrite, ReadBuf},
    net::{TcpListener, TcpStream},
    time::{Instant, Sleep, sleep},
};

/// How long a write may make no progress. Slow readers keep their connection: any progress starts
/// this over.
const SEND_STALL: Duration = Duration::from_secs(30);

/// Accepts connections whose writes are limited by [`SEND_STALL`].
pub struct StallLimitListener(pub TcpListener);

impl Listener for StallLimitListener {
    type Io = StallLimited;
    type Addr = SocketAddr;

    async fn accept(&mut self) -> (Self::Io, Self::Addr) {
        let (stream, address) = Listener::accept(&mut self.0).await;
        let stream = StallLimited {
            stream,
            stall: Box::pin(sleep(SEND_STALL)),
            stalled: false,
        };
        (stream, address)
    }

    fn local_addr(&self) -> io::Result<Self::Addr> {
        Listener::local_addr(&self.0)
    }
}

pub struct StallLimited {
    stream: TcpStream,
    stall: Pin<Box<Sleep>>,
    stalled: bool,
}

impl StallLimited {
    /// A write's outcome: if pending, an error once it has been for the whole limit.
    fn limit(
        &mut self,
        cx: &mut Context<'_>,
        write: Poll<io::Result<usize>>,
    ) -> Poll<io::Result<usize>> {
        if write.is_ready() {
            self.stalled = false;
            return write;
        }
        if !self.stalled {
            self.stalled = true;
            self.stall.as_mut().reset(Instant::now() + SEND_STALL);
        }
        self.stall
            .as_mut()
            .poll(cx)
            .map(|()| Err(io::ErrorKind::TimedOut.into()))
    }
}

impl AsyncRead for StallLimited {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        Pin::new(&mut self.get_mut().stream).poll_read(cx, buf)
    }
}

impl AsyncWrite for StallLimited {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        let this = self.get_mut();
        let write = Pin::new(&mut this.stream).poll_write(cx, buf);
        this.limit(cx, write)
    }

    fn poll_write_vectored(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        bufs: &[IoSlice<'_>],
    ) -> Poll<io::Result<usize>> {
        let this = self.get_mut();
        let write = Pin::new(&mut this.stream).poll_write_vectored(cx, bufs);
        this.limit(cx, write)
    }

    fn is_write_vectored(&self) -> bool {
        self.stream.is_write_vectored()
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        Pin::new(&mut self.get_mut().stream).poll_flush(cx)
    }

    fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        Pin::new(&mut self.get_mut().stream).poll_shutdown(cx)
    }
}
