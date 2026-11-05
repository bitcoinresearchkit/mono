use std::{env, fmt, net::Ipv6Addr, path::PathBuf};

use brk_error::{Error, Result};

use crate::{Auth, Client};

/// How to reach a Bitcoin Core node: its data directory, block files and RPC endpoint.
/// Unset fields fall back to Bitcoin Core's defaults; callers expand `~` themselves.
#[derive(Clone, Default)]
pub struct ConnectArgs {
    pub bitcoindir: Option<PathBuf>,
    pub blocksdir: Option<PathBuf>,
    pub rpcconnect: Option<String>,
    pub rpcport: Option<u16>,
    pub rpccookiefile: Option<PathBuf>,
    pub rpcuser: Option<String>,
    pub rpcpassword: Option<String>,
}

/// As derived, but the password never shows.
impl fmt::Debug for ConnectArgs {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ConnectArgs")
            .field("bitcoindir", &self.bitcoindir)
            .field("blocksdir", &self.blocksdir)
            .field("rpcconnect", &self.rpcconnect)
            .field("rpcport", &self.rpcport)
            .field("rpccookiefile", &self.rpccookiefile)
            .field("rpcuser", &self.rpcuser)
            .field("rpcpassword", &self.rpcpassword.as_ref().map(|_| "…"))
            .finish()
    }
}

impl ConnectArgs {
    /// The node's data directory: `bitcoindir`, else Core's default under the home directory.
    fn bitcoin_dir(&self) -> Result<PathBuf> {
        self.bitcoindir
            .clone()
            .or_else(default_bitcoin_dir)
            .ok_or_else(|| {
                Error::NotFound("no home directory to find Bitcoin's in: set --bitcoindir".into())
            })
    }

    pub fn blocks_dir(&self) -> Result<PathBuf> {
        match &self.blocksdir {
            Some(dir) => Ok(dir.clone()),
            None => Ok(self.bitcoin_dir()?.join("blocks")),
        }
    }

    fn url(&self) -> String {
        let host = self.rpcconnect.as_deref().unwrap_or("localhost");
        let port = self.rpcport.unwrap_or(8332);
        // An IPv6 address (`::1`) needs brackets before the port.
        if host.parse::<Ipv6Addr>().is_ok() {
            format!("http://[{host}]:{port}")
        } else {
            format!("http://{host}:{port}")
        }
    }

    /// Relative cookie paths are inside the data directory, like `bitcoin-cli`'s.
    fn cookie_path(&self) -> Result<PathBuf> {
        Ok(match &self.rpccookiefile {
            Some(path) if path.is_relative() => self.bitcoin_dir()?.join(path),
            Some(path) => path.clone(),
            None => self.bitcoin_dir()?.join(".cookie"),
        })
    }

    /// User and password when a password is set, else the cookie file when there is one
    /// (`bitcoin-cli`'s rule).
    fn auth(&self) -> Result<Option<Auth>> {
        Ok(match self.rpcpassword.as_deref() {
            Some(password) if !password.is_empty() => Some(Auth::UserPass(
                self.rpcuser.clone().unwrap_or_default(),
                password.to_owned(),
            )),
            _ => {
                let cookie = self.cookie_path()?;
                cookie.is_file().then_some(Auth::CookieFile(cookie))
            }
        })
    }

    /// A client for the node, or [`Error::NoRpcCredentials`] when [`Self::auth`] finds none.
    pub fn client(&self) -> Result<Client> {
        let auth = self.auth()?.ok_or_else(|| Error::NoRpcCredentials {
            cookie: self.cookie_path().unwrap_or_default(),
        })?;
        Client::new(&self.url(), auth)
    }
}

/// Core's default data directory, under the home directory.
fn default_bitcoin_dir() -> Option<PathBuf> {
    let home = env::home_dir()?;
    Some(if env::consts::OS == "macos" {
        home.join("Library")
            .join("Application Support")
            .join("Bitcoin")
    } else {
        home.join(".bitcoin")
    })
}
