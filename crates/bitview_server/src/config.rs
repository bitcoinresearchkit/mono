use std::{
    fmt,
    net::{IpAddr, Ipv4Addr},
    path::PathBuf,
    str::FromStr,
};

use axum::http::{HeaderValue, Uri};
use bitview_primitives::OHLCCents;
use serde::{Serialize, Serializer};

use crate::{CdnCacheMode, Port};

/// Default max series-query response weight, in raw value bytes: 10k OHLC values (320 KB), a
/// chart's largest request. Bounding each response this way needs no server-wide body
/// budget.
pub const DEFAULT_MAX_WEIGHT: usize = 10_000 * size_of::<OHLCCents>();

/// Default max UTXOs returned per address, electrs's `--utxos-limit` default as mempool.space serves.
/// Bounds worst-case work and response size, prevents heavy-address DDoS.
pub const DEFAULT_MAX_UTXOS: usize = 500;

/// Default HTTP bind address: this machine only (the server has no authentication); `0.0.0.0` serves its network.
pub const DEFAULT_BIND: IpAddr = IpAddr::V4(Ipv4Addr::LOCALHOST);

#[derive(Debug, Clone)]
pub struct ServerConfig {
    pub bind: IpAddr,
    pub port: Port,
    pub data_path: PathBuf,
    pub cdn_cache_mode: CdnCacheMode,
    pub max_weight: usize,
    pub max_utxos: usize,
    pub public_url: Option<PublicUrl>,
    /// The site's public URL; `None` is `http://localhost:<port>`.
    pub site_url: Option<SiteUrl>,
}

impl ServerConfig {
    /// Directory for daemon and benchmark logs.
    pub fn logs_path(&self) -> PathBuf {
        self.data_path.join("logs")
    }

    /// The site's public origin, without a trailing slash.
    pub fn site_url(&self) -> String {
        match &self.site_url {
            Some(SiteUrl(url)) => url.clone(),
            None => format!("http://localhost:{}", self.port),
        }
    }
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            bind: DEFAULT_BIND,
            port: Port::DEFAULT,
            data_path: PathBuf::default(),
            cdn_cache_mode: CdnCacheMode::default(),
            max_weight: DEFAULT_MAX_WEIGHT,
            max_utxos: DEFAULT_MAX_UTXOS,
            public_url: None,
            site_url: None,
        }
    }
}

/// The URL that reaches this server alone (`--publicurl`), sent as `bitview-origin` on every
/// answer. Only for several servers behind one URL with no affinity: clients then stay on the
/// one that answered.
#[derive(Debug, Clone)]
pub struct PublicUrl(pub(crate) HeaderValue);

impl FromStr for PublicUrl {
    type Err = String;

    fn from_str(url: &str) -> Result<Self, Self::Err> {
        let url = url.trim_end_matches('/');
        let host = url
            .strip_prefix("https://")
            .or_else(|| url.strip_prefix("http://"))
            .ok_or("not an http:// or https:// URL")?;
        if host.is_empty() || host.contains(['?', '#']) || url.contains(char::is_whitespace) {
            return Err("not a server URL".into());
        }
        HeaderValue::from_str(url)
            .map(Self)
            .map_err(|error| error.to_string())
    }
}

impl fmt::Display for PublicUrl {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(self.0.to_str().expect("built from a str"))
    }
}

impl Serialize for PublicUrl {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_str(self)
    }
}

/// The site's public URL (`--siteurl`), an origin: what the MCP server at `<URL>/mcp` names itself by.
#[derive(Debug, Clone)]
pub struct SiteUrl(String);

impl FromStr for SiteUrl {
    type Err = String;

    fn from_str(url: &str) -> Result<Self, Self::Err> {
        let url = url.trim().trim_end_matches('/');
        let uri: Uri = url.parse().map_err(|error| format!("not a URL: {error}"))?;
        if !matches!(uri.scheme_str(), Some("http" | "https")) || uri.host().is_none() {
            return Err("not an http:// or https:// URL".into());
        }
        if uri.query().is_some() || !matches!(uri.path(), "" | "/") || uri.authority().unwrap().as_str().contains('@') {
            return Err("an origin has no path, query or credentials".into());
        }
        Ok(Self(url.to_owned()))
    }
}

impl fmt::Display for SiteUrl {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl Serialize for SiteUrl {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_str(self)
    }
}
