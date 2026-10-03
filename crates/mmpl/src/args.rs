use std::path::PathBuf;

use brk_error::{Error, Result};
use brk_rpc::ConnectArgs;

#[derive(Default)]
pub struct Args {
    pub connect: ConnectArgs,
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
                "bitcoindir" => args.connect.bitcoindir = Some(PathBuf::from(value)),
                "rpcconnect" => args.connect.rpcconnect = Some(value),
                "rpcport" => {
                    args.connect.rpcport = Some(value.parse().map_err(|_| {
                        Error::Parse(format!("--rpcport: '{value}' is not a valid port"))
                    })?);
                }
                "rpccookiefile" => args.connect.rpccookiefile = Some(PathBuf::from(value)),
                "rpcuser" => args.connect.rpcuser = Some(value),
                "rpcpassword" => args.connect.rpcpassword = Some(value),
                other => return Err(Error::Parse(format!("unknown flag --{other}"))),
            }
        }
        Ok(args)
    }
}
