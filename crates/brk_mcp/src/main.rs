mod arguments;
mod config;
mod logo;
mod manifest;
mod page;
mod server;
#[cfg(test)]
mod server_tests;
mod upstream;

use std::{env, error::Error, io, process};

use arguments::Arguments;
use manifest::Catalog;
use tokio::net::TcpListener;
use tracing::info;

const BIND_START: std::net::SocketAddr =
    std::net::SocketAddr::new(std::net::IpAddr::V4(std::net::Ipv4Addr::LOCALHOST), 3111);

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    brk_logger::init_with_default_level(None, "info")?;

    let arguments = Arguments::parse(env::args().skip(1)).unwrap_or_else(|error| usage(&error));
    let (api_bases, api_url, public_url, display_name) = arguments.into_parts();
    let catalog = Catalog::embedded().map_err(io::Error::other)?;
    let tool_count = catalog.tools().len();
    let pages = page::Pages::render(&display_name, &public_url, &api_url);
    let app = server::router(api_bases, catalog, display_name, public_url, pages);
    let (listener, bind) = bind_available(BIND_START).await?;

    info!(%bind, tool_count, "Starting stateless BRK MCP server");
    axum::serve(listener, app).await?;
    Ok(())
}

fn usage(error: &str) -> ! {
    eprintln!("Error: {error}");
    eprintln!(
        "Usage: brk_mcp --api <REST_API_URL_OR_HOST> --url <PUBLIC_MCP_URL> --name <DISPLAY_NAME>"
    );
    process::exit(2);
}

async fn bind_available(
    start: std::net::SocketAddr,
) -> io::Result<(TcpListener, std::net::SocketAddr)> {
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
