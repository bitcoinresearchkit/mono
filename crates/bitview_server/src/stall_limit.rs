//! A connection that takes no byte for [`SEND_STALL`] is closed. A client that stops reading would
//! otherwise hold its response, and the admission a large one keeps until sent, for as long as it
//! likes. It's enforced on the socket: while one is blocked, no one polls the response's body.

use std::{io, net::SocketAddr, pin::Pin, time::Duration};

use axum::serve::Listener;
use tokio::net::{TcpListener, TcpStream};
use tokio_io_timeout::TimeoutStream;

/// How long a write may make no progress. Slow readers keep their connection: any progress starts
/// this over.
const SEND_STALL: Duration = Duration::from_secs(30);

/// Accepts connections whose writes are limited by [`SEND_STALL`].
pub struct StallLimitListener(pub TcpListener);

impl Listener for StallLimitListener {
    type Io = Pin<Box<TimeoutStream<TcpStream>>>;
    type Addr = SocketAddr;

    async fn accept(&mut self) -> (Self::Io, Self::Addr) {
        let (stream, address) = Listener::accept(&mut self.0).await;
        let mut stream = TimeoutStream::new(stream);
        stream.set_write_timeout(Some(SEND_STALL));
        (Box::pin(stream), address)
    }

    fn local_addr(&self) -> io::Result<Self::Addr> {
        Listener::local_addr(&self.0)
    }
}
