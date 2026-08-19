use std::{collections::HashSet, path::PathBuf};

use brk_error::Error;
use brk_rpc::{Auth, Client};

use crate::path::Path;

pub struct Args {
    pub selector: String,
    pub paths: Vec<Path>,
    pub pretty: bool,
    pub compact: bool,
    bitcoindir: Option<PathBuf>,
    blocksdir: Option<PathBuf>,
    rpcconnect: Option<String>,
    rpcport: Option<u16>,
    rpccookiefile: Option<PathBuf>,
    rpcuser: Option<String>,
    rpcpassword: Option<String>,
}

impl Args {
    pub fn parse(raw: Vec<String>) -> brk_error::Result<Self> {
        let mut pretty = false;
        let mut compact = false;
        let mut bitcoindir = None;
        let mut blocksdir = None;
        let mut rpcconnect = None;
        let mut rpcport = None;
        let mut rpccookiefile = None;
        let mut rpcuser = None;
        let mut rpcpassword = None;
        let mut positional: Vec<String> = Vec::new();
        let mut iter = raw.into_iter();
        while let Some(a) = iter.next() {
            if a == "-p" || a == "--pretty" {
                pretty = true;
                continue;
            }
            if a == "-c" || a == "--compact" {
                compact = true;
                continue;
            }
            if let Some(rest) = a.strip_prefix("--") {
                let (key, value) = match rest.split_once('=') {
                    Some((k, v)) => (k.to_string(), v.to_string()),
                    None => (
                        rest.to_string(),
                        iter.next()
                            .ok_or_else(|| Error::Parse(format!("--{rest} requires a value")))?,
                    ),
                };
                match key.as_str() {
                    "bitcoindir" => bitcoindir = Some(PathBuf::from(value)),
                    "blocksdir" => blocksdir = Some(PathBuf::from(value)),
                    "rpcconnect" => rpcconnect = Some(value),
                    "rpcport" => {
                        rpcport = Some(value.parse().map_err(|_| {
                            Error::Parse(format!("--rpcport: '{value}' is not a valid port"))
                        })?);
                    }
                    "rpccookiefile" => rpccookiefile = Some(PathBuf::from(value)),
                    "rpcuser" => rpcuser = Some(value),
                    "rpcpassword" => rpcpassword = Some(value),
                    other => return Err(Error::Parse(format!("unknown flag --{other}"))),
                }
                continue;
            }
            if a.starts_with('-') {
                return Err(Error::Parse(format!("unknown flag {a}")));
            }
            positional.push(a);
        }

        let mut iter = positional.into_iter();
        let selector = iter
            .next()
            .ok_or_else(|| Error::Parse("missing selector".into()))?;
        let paths: Vec<Path> = iter
            .map(|f| Path::parse(&f))
            .collect::<brk_error::Result<_>>()?;
        let mut seen = HashSet::with_capacity(paths.len());
        for p in &paths {
            if !seen.insert(p.raw.as_str()) {
                return Err(Error::Parse(format!("duplicate field '{}'", p.raw)));
            }
        }
        Ok(Self {
            selector,
            paths,
            pretty,
            compact,
            bitcoindir,
            blocksdir,
            rpcconnect,
            rpcport,
            rpccookiefile,
            rpcuser,
            rpcpassword,
        })
    }

    pub fn bitcoin_dir(&self) -> PathBuf {
        self.bitcoindir
            .clone()
            .unwrap_or_else(Client::default_bitcoin_path)
    }

    pub fn blocks_dir(&self) -> PathBuf {
        self.blocksdir
            .clone()
            .unwrap_or_else(|| self.bitcoin_dir().join("blocks"))
    }

    pub fn rpc(&self) -> brk_error::Result<Client> {
        let host = self.rpcconnect.as_deref().unwrap_or("localhost");
        let port = self.rpcport.unwrap_or(8332);
        let url = format!("http://{host}:{port}");
        let cookie = self
            .rpccookiefile
            .clone()
            .unwrap_or_else(|| self.bitcoin_dir().join(".cookie"));
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
