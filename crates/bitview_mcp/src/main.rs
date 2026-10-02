use std::{
    env,
    error::Error,
    io,
    net::{IpAddr, Ipv4Addr, SocketAddr},
    process,
};

use arguments::Arguments;
use axum::serve;
use brk_logger::init_with_default_level;
use manifest::Catalog;
use page::Pages;
use tokio::net::TcpListener;
use tracing::info;

mod arguments;
mod config;
mod logo;
mod manifest;
mod page;
mod prepared_request;
mod server;
mod upstream;
mod upstream_response;

const BIND_START: SocketAddr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 3111);

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    init_with_default_level(None, "info")?;

    let arguments = Arguments::parse(env::args().skip(1)).unwrap_or_else(|error| usage(&error));
    let (api_bases, api_url, public_url, display_name) = arguments.into_parts();
    let catalog = Catalog::embedded().map_err(io::Error::other)?;
    let tool_count = catalog.tools().len();
    let pages = Pages::render(&display_name, &public_url, &api_url);
    let app = server::router(api_bases, catalog, display_name, public_url, pages);
    let (listener, bind) = bind_available(BIND_START).await?;

    info!("BRK MCP server listening on http://{bind} with {tool_count} tools");
    serve(listener, app).await?;
    Ok(())
}

fn usage(error: &str) -> ! {
    eprintln!("Error: {error}");
    eprintln!(
        "Usage: bitview_mcp --api <REST_API_URL_OR_HOST> --url <PUBLIC_MCP_URL> --name <DISPLAY_NAME>"
    );
    process::exit(2);
}

async fn bind_available(start: SocketAddr) -> io::Result<(TcpListener, SocketAddr)> {
    let last_port = start.port().saturating_add(100);
    let mut last_error = None;
    for port in start.port()..=last_port {
        let mut candidate = start;
        candidate.set_port(port);
        match TcpListener::bind(candidate).await {
            Ok(listener) => return Ok((listener, candidate)),
            Err(error) => last_error = Some(error),
        }
    }
    Err(last_error.unwrap_or_else(|| {
        io::Error::new(
            io::ErrorKind::AddrNotAvailable,
            "no MCP bind port available",
        )
    }))
}
