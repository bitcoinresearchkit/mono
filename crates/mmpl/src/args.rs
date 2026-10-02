use std::path::PathBuf;

use brk_error::{Error, Result};
use brk_rpc::{Auth, Client};

#[derive(Default)]
pub struct Args {
    bitcoindir: Option<PathBuf>,
    rpcconnect: Option<String>,
    rpcport: Option<u16>,
    rpccookiefile: Option<PathBuf>,
    rpcuser: Option<String>,
    rpcpassword: Option<String>,
}

impl Args {
    pub fn parse(raw: Vec<String>) -> Result<Self> {
        let mut args = Self::default();
        let mut iter = raw.into_iter();
        while let Some(a) = iter.next() {
            let rest = a
                .strip_prefix("--")
                .ok_or_else(|| Error::Parse(format!("unexpected arg: '{a}'")))?;
            let (key, value) = match rest.split_once('=') {
                Some((k, v)) => (k, v.to_string()),
                None => (
                    rest,
                    iter.next()
                        .ok_or_else(|| Error::Parse(format!("--{rest} requires a value")))?,
                ),
            };
            match key {
                "bitcoindir" => args.bitcoindir = Some(PathBuf::from(value)),
                "rpcconnect" => args.rpcconnect = Some(value),
                "rpcport" => {
                    args.rpcport = Some(value.parse().map_err(|_| {
                        Error::Parse(format!("--rpcport: '{value}' is not a valid port"))
                    })?);
                }
                "rpccookiefile" => args.rpccookiefile = Some(PathBuf::from(value)),
                "rpcuser" => args.rpcuser = Some(value),
                "rpcpassword" => args.rpcpassword = Some(value),
                other => return Err(Error::Parse(format!("unknown flag --{other}"))),
            }
        }
        Ok(args)
    }

    pub fn rpc(&self) -> Result<Client> {
        let host = self.rpcconnect.as_deref().unwrap_or("localhost");
        let port = self.rpcport.unwrap_or(8332);
        let url = format!("http://{host}:{port}");
        let bitcoin_dir = self
            .bitcoindir
            .clone()
            .unwrap_or_else(Client::default_bitcoin_path);
        let cookie = self
            .rpccookiefile
            .clone()
            .unwrap_or_else(|| bitcoin_dir.join(".cookie"));
        let auth = if cookie.is_file() {
            Auth::CookieFile(cookie)
        } else if let (Some(u), Some(p)) = (self.rpcuser.as_deref(), self.rpcpassword.as_deref()) {
            Auth::UserPass(u.to_string(), p.to_string())
        } else {
            return Err(Error::Parse(
                "no RPC auth: cookie file missing and --rpcuser/--rpcpassword not set".into(),
            ));
        };
        Client::new(&url, auth)
    }
}
