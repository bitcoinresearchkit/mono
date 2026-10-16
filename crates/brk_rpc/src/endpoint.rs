use brk_error::{Error, Result};

/// A node's RPC endpoint, from `http://host[:port][/path]` (port 80 when missing, IPv6 hosts in
/// brackets, credentials only through `Auth`): the one parse both transports connect and address
/// requests with.
pub(crate) struct Endpoint {
    pub host: String,
    pub port: u16,
    /// `Host` header value.
    pub authority: String,
    pub path: String,
}

impl Endpoint {
    pub fn parse(url: &str) -> Result<Self> {
        let invalid = || Error::Internal("invalid node RPC URL");
        let rest = url
            .strip_prefix("http://")
            .ok_or(Error::Internal("node RPC requires an HTTP URL"))?;
        let (authority, path) = rest.find('/').map_or((rest, "/"), |at| rest.split_at(at));
        if authority.contains('@') {
            return Err(Error::Internal(
                "node RPC URL holds credentials: set rpcuser and rpcpassword, or rpccookiefile",
            ));
        }
        let (host, port) = match authority.strip_prefix('[') {
            Some(bracketed) => {
                let (host, rest) = bracketed.split_once(']').ok_or_else(invalid)?;
                let port = match rest {
                    "" => None,
                    rest => Some(rest.strip_prefix(':').ok_or_else(invalid)?),
                };
                (host, port)
            }
            None => match authority.split_once(':') {
                Some((host, port)) => (host, Some(port)),
                None => (authority, None),
            },
        };
        let port = port.map_or(Ok(80), str::parse).map_err(|_| invalid())?;
        if host.is_empty() {
            return Err(invalid());
        }
        Ok(Self {
            host: host.to_owned(),
            port,
            authority: authority.to_owned(),
            path: path.to_owned(),
        })
    }
}
