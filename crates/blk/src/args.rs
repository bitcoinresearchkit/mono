use std::{collections::HashSet, path::PathBuf};

use brk_error::{Error, Result};
use brk_rpc::ConnectArgs;

use crate::path::Path;

pub struct Args {
    pub selector: String,
    pub paths: Vec<Path>,
    pub pretty: bool,
    pub compact: bool,
    pub connect: ConnectArgs,
}

impl Args {
    pub fn parse(raw: Vec<String>) -> Result<Self> {
        let mut pretty = false;
        let mut compact = false;
        let mut connect = ConnectArgs::default();
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
                    "bitcoindir" => connect.bitcoindir = Some(PathBuf::from(value)),
                    "blocksdir" => connect.blocksdir = Some(PathBuf::from(value)),
                    "rpcconnect" => connect.rpcconnect = Some(value),
                    "rpcport" => {
                        connect.rpcport = Some(value.parse().map_err(|_| {
                            Error::Parse(format!("--rpcport: '{value}' is not a valid port"))
                        })?);
                    }
                    "rpccookiefile" => connect.rpccookiefile = Some(PathBuf::from(value)),
                    "rpcuser" => connect.rpcuser = Some(value),
                    "rpcpassword" => connect.rpcpassword = Some(value),
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
        let paths: Vec<Path> = iter.map(|f| Path::parse(&f)).collect::<Result<_>>()?;
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
            connect,
        })
    }
}
