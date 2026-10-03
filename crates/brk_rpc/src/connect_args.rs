use std::{env, path::PathBuf};

use brk_error::{Error, Result};

use crate::{Auth, Client};

/// How to reach a Bitcoin Core node: its data directory, block files and RPC endpoint.
/// Unset fields fall back to Bitcoin Core's defaults; callers expand `~` themselves.
#[derive(Debug, Clone, Default)]
pub struct ConnectArgs {
    pub bitcoindir: Option<PathBuf>,
    pub blocksdir: Option<PathBuf>,
    pub rpcconnect: Option<String>,
    pub rpcport: Option<u16>,
    pub rpccookiefile: Option<PathBuf>,
    pub rpcuser: Option<String>,
    pub rpcpassword: Option<String>,
}

impl ConnectArgs {
    pub fn bitcoin_dir(&self) -> PathBuf {
        self.bitcoindir.clone().unwrap_or_else(default_bitcoin_dir)
    }

    pub fn blocks_dir(&self) -> PathBuf {
        self.blocksdir
            .clone()
            .unwrap_or_else(|| self.bitcoin_dir().join("blocks"))
    }

    pub fn url(&self) -> String {
        format!(
            "http://{}:{}",
            self.rpcconnect.as_deref().unwrap_or("localhost"),
            self.rpcport.unwrap_or(8332)
        )
    }

    pub fn cookie_path(&self) -> PathBuf {
        self.rpccookiefile
            .clone()
            .unwrap_or_else(|| self.bitcoin_dir().join(".cookie"))
    }

    /// User and password when a password is set, else the cookie file when there is one
    /// (`bitcoin-cli`'s rule).
    pub fn auth(&self) -> Option<Auth> {
        match self.rpcpassword.as_deref() {
            Some(password) if !password.is_empty() => Some(Auth::UserPass(
                self.rpcuser.clone().unwrap_or_default(),
                password.to_owned(),
            )),
            _ => {
                let cookie = self.cookie_path();
                cookie.is_file().then_some(Auth::CookieFile(cookie))
            }
        }
    }

    /// A client for the node, or [`Error::NoRpcCredentials`] when [`Self::auth`] finds none.
    pub fn client(&self) -> Result<Client> {
        let auth = self.auth().ok_or_else(|| Error::NoRpcCredentials {
            cookie: self.cookie_path(),
        })?;
        Client::new(&self.url(), auth)
    }
}

/// Bitcoin Core's default data directory on this OS.
fn default_bitcoin_dir() -> PathBuf {
    let home = PathBuf::from(env::var("HOME").unwrap());
    if env::consts::OS == "macos" {
        home.join("Library")
            .join("Application Support")
            .join("Bitcoin")
    } else {
        home.join(".bitcoin")
    }
}
