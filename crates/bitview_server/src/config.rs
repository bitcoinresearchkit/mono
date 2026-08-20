use std::{
    net::{IpAddr, Ipv4Addr},
    path::PathBuf,
};

use bitview_website::Website;
use brk_types::Port;

use crate::cache::CdnCacheMode;

/// Default max series-query response weight.
/// 50 MB - generous enough for any honest query, low enough to limit cache-buster leverage.
pub const DEFAULT_MAX_WEIGHT: usize = 50 * 1_000_000;

/// Default max UTXOs returned per address.
/// Bounds worst-case work and response size, prevents heavy-address DDoS.
pub const DEFAULT_MAX_UTXOS: usize = 1000;

/// Default HTTP bind address. Accepts connections on every IPv4 interface.
pub const DEFAULT_BIND: IpAddr = IpAddr::V4(Ipv4Addr::UNSPECIFIED);

/// Server-wide configuration set at startup.
#[derive(Debug, Clone)]
pub struct ServerConfig {
    pub bind: IpAddr,
    pub port: Port,
    pub data_path: PathBuf,
    pub website: Website,
    pub cdn_cache_mode: CdnCacheMode,
    pub max_weight: usize,
    pub max_utxos: usize,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            bind: DEFAULT_BIND,
            port: Port::DEFAULT,
            data_path: PathBuf::default(),
            website: Website::default(),
            cdn_cache_mode: CdnCacheMode::default(),
            max_weight: DEFAULT_MAX_WEIGHT,
            max_utxos: DEFAULT_MAX_UTXOS,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_to_all_interfaces_on_the_default_port() {
        let config = ServerConfig::default();

        assert_eq!(config.bind, DEFAULT_BIND);
        assert_eq!(config.port, Port::DEFAULT);
    }
}
