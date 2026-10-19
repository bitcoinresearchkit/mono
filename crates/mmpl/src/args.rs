use std::path::PathBuf;

use brk_error::{Error, Result};
use brk_rpc::ConnectArgs;

pub fn parse(raw: Vec<String>) -> Result<ConnectArgs> {
    let mut connect = ConnectArgs::default();
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
            "bitcoindir" => connect.bitcoindir = Some(PathBuf::from(value)),
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
    }
    Ok(connect)
}
