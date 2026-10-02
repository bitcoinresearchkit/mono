use std::{
    net::{IpAddr, Ipv4Addr},
    path::PathBuf,
};

use bitview_website::Website;
use brk_types::{OHLCCents, Port};

use crate::cache::CdnCacheMode;

/// Default max series-query response weight, in raw value bytes.
/// 10k OHLC values (320 KB): the website's largest chart request, so every
/// response stays light and fast without a server-wide body budget.
pub const DEFAULT_MAX_WEIGHT: usize = 10_000 * size_of::<OHLCCents>();

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

impl ServerConfig {
    /// Directory for daemon and benchmark logs.
    pub fn logs_path(&self) -> PathBuf {
        self.data_path.join("logs")
    }
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
