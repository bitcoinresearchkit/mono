// Auto-generated Bitview Rust client
// Do not edit manually

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(clippy::useless_format)]
#![allow(clippy::unnecessary_to_owned)]

use crate::{DateSeriesData, FormatResponse};
pub use bitview_catalog::*;
pub use bitview_cohort::*;
pub use bitview_primitives::*;
pub use bitview_types::*;
pub use brk_types::*;
use serde::de::DeserializeOwned;
use std::ops::{Bound, RangeBounds};
use std::str::FromStr;
use std::sync::{Arc, LazyLock, OnceLock};

/// Lazily initialized typed series-tree node.
pub type LazyNode<T> = LazyLock<Box<T>, Box<dyn FnOnce() -> Box<T> + Send + Sync>>;

/// Error type for Bitview client operations.
#[derive(Debug)]
pub struct BitviewError {
    /// HTTP status of the server's error answer.
    pub status: Option<u16>,
    /// The server's machine-readable code, from its error body.
    pub code: Option<ErrorCode>,
    pub message: String,
}

impl BitviewError {
    /// A client-side error, not an answer from the server.
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            status: None,
            code: None,
            message: message.into(),
        }
    }
}

impl std::fmt::Display for BitviewError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for BitviewError {}

/// Result using the Bitview client's error type.
pub type Result<T> = std::result::Result<T, BitviewError>;

/// BRK address type and raw payload bytes used by the hash-prefix index.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AddressPayload {
    pub addr_type: OutputType,
    pub payload: Vec<u8>,
}

/// BRK address type and leading hex nibbles of the address-payload hash.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AddressHashPrefix {
    pub addr_type: OutputType,
    pub prefix: String,
}

/// Compute the RapidHash v3 hash-prefix used by `/api/address/hash-prefix/{addr_type}/{prefix}`.
pub fn address_payload_hash_prefix(payload: &[u8], nibbles: usize) -> Result<String> {
    if payload.is_empty() {
        return Err(BitviewError::new("Expected a non-empty address payload"));
    }
    if payload.len() > 65 {
        return Err(BitviewError::new(
            "Expected at most 65 address payload bytes",
        ));
    }
    if !(1..=16).contains(&nibbles) {
        return Err(BitviewError::new(
            "Expected hash-prefix length from 1 to 16 hex nibbles",
        ));
    }
    Ok(format!("{:016x}", rapidhash::v3::rapidhash_v3(payload))[..nibbles].to_string())
}

fn validate_address_payload_for_type(addr_type: OutputType, payload: &[u8]) -> Result<()> {
    let expected: &[usize] = match addr_type {
        OutputType::P2A => &[2],
        OutputType::P2PK33 => &[33],
        OutputType::P2PK65 => &[65],
        OutputType::P2PKH | OutputType::P2SH | OutputType::P2WPKH => &[20],
        OutputType::P2WSH | OutputType::P2TR => &[32],
        OutputType::P2MS | OutputType::OpReturn | OutputType::Empty | OutputType::Unknown => {
            return Err(BitviewError::new(format!(
                "Unsupported address type for address payload hash-prefix: {addr_type:?}"
            )));
        }
    };

    if !expected.contains(&payload.len()) {
        let joined = expected
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join(" or ");
        return Err(BitviewError::new(format!(
            "Expected {addr_type} address payload length {joined} bytes"
        )));
    }

    Ok(())
}

/// Decode a mainnet Bitcoin address into the BRK address type and raw payload bytes.
pub fn decode_address_payload(address: &str) -> Result<AddressPayload> {
    if address.is_empty() {
        return Err(BitviewError::new("Expected an address string"));
    }
    let addr_bytes = AddrBytes::from_str(address).map_err(|e| BitviewError::new(e.to_string()))?;
    let addr_type = OutputType::from(&addr_bytes);

    Ok(AddressPayload {
        addr_type,
        payload: addr_bytes.as_slice().to_vec(),
    })
}

/// Decode a mainnet Bitcoin address and compute its hash prefix.
pub fn address_hash_prefix(address: &str, nibbles: usize) -> Result<AddressHashPrefix> {
    let decoded = decode_address_payload(address)?;
    Ok(AddressHashPrefix {
        addr_type: decoded.addr_type,
        prefix: address_payload_hash_prefix(&decoded.payload, nibbles)?,
    })
}

/// Options for configuring the Bitview client.
#[derive(Debug, Clone)]
pub struct BitviewClientOptions {
    pub base_url: String,
    pub timeout_secs: u64,
}

impl Default for BitviewClientOptions {
    fn default() -> Self {
        Self {
            base_url: "http://localhost:3110".to_string(),
            timeout_secs: 30,
        }
    }
}

/// Base HTTP client for making requests. Reuses connections via ureq::Agent.
#[derive(Debug, Clone)]
pub struct BitviewClientBase {
    agent: ureq::Agent,
    base_url: String,
}

impl BitviewClientBase {
    /// Create a new client with the given base URL.
    pub fn new(base_url: impl Into<String>) -> Self {
        Self::with_options(BitviewClientOptions {
            base_url: base_url.into(),
            ..Default::default()
        })
    }

    /// Create a new client with options.
    pub fn with_options(options: BitviewClientOptions) -> Self {
        let agent = ureq::Agent::config_builder()
            .timeout_global(Some(std::time::Duration::from_secs(options.timeout_secs)))
            .http_status_as_error(false)
            .build()
            .into();
        Self {
            agent,
            base_url: options.base_url.trim_end_matches('/').to_string(),
        }
    }

    fn url(&self, path: &str) -> String {
        format!("{}{}", self.base_url, path)
    }

    /// Passes a successful answer through; an error status becomes a `BitviewError` with the
    /// server's code and message when it sent an error body.
    fn check(
        response: std::result::Result<ureq::http::Response<ureq::Body>, ureq::Error>,
    ) -> Result<ureq::http::Response<ureq::Body>> {
        let mut response = response.map_err(|e| BitviewError::new(e.to_string()))?;
        let status = response.status();
        if status.is_success() {
            return Ok(response);
        }
        let text = response.body_mut().read_to_string().unwrap_or_default();
        // Fields read one by one: a code this client doesn't know keeps the server's message.
        let detail = serde_json::from_str::<serde_json::Value>(&text)
            .ok()
            .and_then(|mut body| body.get_mut("error").map(serde_json::Value::take));
        let field = |name: &str| detail.as_ref().and_then(|detail| detail.get(name)).cloned();
        let code = field("code").and_then(|code| serde_json::from_value::<ErrorCode>(code).ok());
        let message = match field("message") {
            Some(serde_json::Value::String(message)) => message,
            _ if text.is_empty() => status.to_string(),
            _ => text,
        };
        Err(BitviewError {
            status: Some(status.as_u16()),
            code,
            message,
        })
    }

    /// Make a GET request and deserialize JSON response.
    pub fn get_json<T: DeserializeOwned>(&self, path: &str) -> Result<T> {
        Self::check(self.agent.get(&self.url(path)).call())?
            .body_mut()
            .read_json()
            .map_err(|e| BitviewError::new(e.to_string()))
    }

    /// Make a GET request and return raw text response.
    pub fn get_text(&self, path: &str) -> Result<String> {
        Self::check(self.agent.get(&self.url(path)).call())?
            .body_mut()
            .read_to_string()
            .map_err(|e| BitviewError::new(e.to_string()))
    }

    /// Make a GET request and return raw bytes response.
    pub fn get_bytes(&self, path: &str) -> Result<Vec<u8>> {
        Self::check(self.agent.get(&self.url(path)).call())?
            .body_mut()
            .read_to_vec()
            .map_err(|e| BitviewError::new(e.to_string()))
    }

    /// Make a POST request and deserialize JSON response.
    pub fn post_json<T: DeserializeOwned>(&self, path: &str, body: &str) -> Result<T> {
        Self::check(self.agent.post(&self.url(path)).send(body))?
            .body_mut()
            .read_json()
            .map_err(|e| BitviewError::new(e.to_string()))
    }

    /// Make a POST request and return raw text response.
    pub fn post_text(&self, path: &str, body: &str) -> Result<String> {
        Self::check(self.agent.post(&self.url(path)).send(body))?
            .body_mut()
            .read_to_string()
            .map_err(|e| BitviewError::new(e.to_string()))
    }

    /// Make a POST request and return raw bytes response.
    pub fn post_bytes(&self, path: &str, body: &str) -> Result<Vec<u8>> {
        Self::check(self.agent.post(&self.url(path)).send(body))?
            .body_mut()
            .read_to_vec()
            .map_err(|e| BitviewError::new(e.to_string()))
    }
}

/// Non-generic trait for series patterns (usable in collections).
pub trait AnySeriesPattern {
    /// Get the series name.
    fn name(&self) -> &str;

    /// Get the list of available indexes for this series.
    fn indexes(&self) -> &'static [Index];
}

/// Generic trait for series patterns with endpoint access.
pub trait SeriesPattern<T>: AnySeriesPattern {
    /// Get an endpoint builder for a specific index, if supported.
    fn get(&self, index: Index) -> Option<SeriesEndpoint<T>>;
}

impl<P: AnySeriesPattern> AnySeriesPattern for LazyNode<P> {
    fn name(&self) -> &str {
        (**self).name()
    }

    fn indexes(&self) -> &'static [Index] {
        (**self).indexes()
    }
}

impl<T, P: SeriesPattern<T>> SeriesPattern<T> for LazyNode<P> {
    fn get(&self, index: Index) -> Option<SeriesEndpoint<T>> {
        (**self).get(index)
    }
}

/// Shared endpoint configuration.
#[derive(Clone)]
struct EndpointConfig {
    client: Arc<BitviewClientBase>,
    name: Arc<str>,
    index: Index,
    start: Option<i64>,
    end: Option<i64>,
}

impl EndpointConfig {
    fn new(client: Arc<BitviewClientBase>, name: Arc<str>, index: Index) -> Self {
        Self {
            client,
            name,
            index,
            start: None,
            end: None,
        }
    }

    fn path(&self) -> String {
        format!("/api/series/{}/{}", self.name, self.index.name())
    }

    fn build_path(&self, format: Option<&str>) -> String {
        let mut params = Vec::new();
        if let Some(s) = self.start {
            params.push(format!("start={}", s));
        }
        if let Some(e) = self.end {
            params.push(format!("end={}", e));
        }
        if let Some(fmt) = format {
            params.push(format!("format={}", fmt));
        }
        let p = self.path();
        if params.is_empty() {
            p
        } else {
            format!("{}?{}", p, params.join("&"))
        }
    }

    fn get_json<T: DeserializeOwned>(&self, format: Option<&str>) -> Result<T> {
        self.client.get_json(&self.build_path(format))
    }

    fn get_text(&self, format: Option<&str>) -> Result<String> {
        self.client.get_text(&self.build_path(format))
    }

    fn get_len(&self) -> Result<i64> {
        self.client.get_json(&format!(
            "/api/series/{}/{}/len",
            self.name,
            self.index.name()
        ))
    }

    fn get_version(&self) -> Result<u32> {
        self.client.get_json(&format!(
            "/api/series/{}/{}/version",
            self.name,
            self.index.name()
        ))
    }
}

/// Builder for series endpoint queries.
///
/// Parameterized by element type `T` and response type `D` (defaults to `SeriesData<T>`).
/// For date-based indexes, use `DateSeriesEndpoint<T>` which sets `D = DateSeriesData<T>`.
///
/// # Examples
/// ```ignore
/// let data = endpoint.fetch()?;                   // all data
/// let data = endpoint.get(5).fetch()?;             // single item
/// let data = endpoint.range(..10).fetch()?;        // first 10
/// let data = endpoint.range(100..200).fetch()?;    // range [100, 200)
/// let data = endpoint.take(10).fetch()?;           // first 10 (convenience)
/// let data = endpoint.last(10).fetch()?;           // last 10
/// let data = endpoint.skip(100).take(10).fetch()?; // iterator-style
/// ```
pub struct SeriesEndpoint<T, D = SeriesData<T>> {
    config: EndpointConfig,
    _marker: std::marker::PhantomData<fn() -> (T, D)>,
}

/// Date-based series endpoint with date and timestamp selectors.
pub struct DateSeriesEndpoint<T>(SeriesEndpoint<T, DateSeriesData<T>>);

impl<T: DeserializeOwned, D: DeserializeOwned> SeriesEndpoint<T, D> {
    pub fn new(client: Arc<BitviewClientBase>, name: Arc<str>, index: Index) -> Self {
        Self {
            config: EndpointConfig::new(client, name, index),
            _marker: std::marker::PhantomData,
        }
    }

    /// Select a specific index position.
    pub fn get(mut self, index: usize) -> SingleItemBuilder<T, D> {
        self.config.start = Some(index as i64);
        self.config.end = Some(index as i64 + 1);
        SingleItemBuilder {
            config: self.config,
            _marker: std::marker::PhantomData,
        }
    }

    /// Select a range using Rust range syntax.
    ///
    /// # Examples
    /// ```ignore
    /// endpoint.range(..10)      // first 10
    /// endpoint.range(100..110)  // indices 100-109
    /// endpoint.range(100..)     // from 100 to end
    /// ```
    pub fn range<R: RangeBounds<usize>>(mut self, range: R) -> RangeBuilder<T, D> {
        self.config.start = match range.start_bound() {
            Bound::Included(&n) => Some(n as i64),
            Bound::Excluded(&n) => Some(n as i64 + 1),
            Bound::Unbounded => None,
        };
        self.config.end = match range.end_bound() {
            Bound::Included(&n) => Some(n as i64 + 1),
            Bound::Excluded(&n) => Some(n as i64),
            Bound::Unbounded => None,
        };
        RangeBuilder {
            config: self.config,
            _marker: std::marker::PhantomData,
        }
    }

    /// Take the first n items.
    pub fn take(self, n: usize) -> RangeBuilder<T, D> {
        self.range(..n)
    }

    /// Take the last n items.
    pub fn last(mut self, n: usize) -> RangeBuilder<T, D> {
        if n == 0 {
            self.config.end = Some(0);
        } else {
            self.config.start = Some(-(n as i64));
        }
        RangeBuilder {
            config: self.config,
            _marker: std::marker::PhantomData,
        }
    }

    /// Skip the first n items. Chain with `take(n)` to get a range.
    pub fn skip(mut self, n: usize) -> SkippedBuilder<T, D> {
        self.config.start = Some(n as i64);
        SkippedBuilder {
            config: self.config,
            _marker: std::marker::PhantomData,
        }
    }

    /// Fetch all data as parsed JSON.
    pub fn fetch(self) -> Result<D> {
        self.config.get_json(None)
    }

    /// Fetch all data as CSV string.
    pub fn fetch_csv(self) -> Result<String> {
        self.config.get_text(Some("csv"))
    }

    /// Total number of data points for this series.
    #[allow(clippy::len_without_is_empty)]
    pub fn len(&self) -> Result<i64> {
        self.config.get_len()
    }

    /// Current version of the series.
    pub fn version(&self) -> Result<u32> {
        self.config.get_version()
    }

    /// Get the base endpoint path.
    pub fn path(&self) -> String {
        self.config.path()
    }
}

fn required_date_index(position: Option<usize>) -> Result<usize> {
    position
        .ok_or_else(|| BitviewError::new("Date or timestamp is not representable for this index"))
}

/// Date-specific methods available only on `DateSeriesEndpoint`.
impl<T: DeserializeOwned> SeriesEndpoint<T, DateSeriesData<T>> {
    /// Select a specific date position (for day-precision or coarser indexes).
    pub fn get_date(self, date: Date) -> Result<SingleItemBuilder<T, DateSeriesData<T>>> {
        let index = required_date_index(self.config.index.date_to_index(date))?;
        Ok(self.get(index))
    }

    /// Select a date range (for day-precision or coarser indexes).
    pub fn date_range(self, start: Date, end: Date) -> Result<RangeBuilder<T, DateSeriesData<T>>> {
        let s = required_date_index(self.config.index.date_to_index(start))?;
        let e = required_date_index(self.config.index.date_to_index(end))?;
        Ok(self.range(s..e))
    }

    /// Select a specific timestamp position (works for all date-based indexes including sub-daily).
    pub fn get_timestamp(self, ts: Timestamp) -> Result<SingleItemBuilder<T, DateSeriesData<T>>> {
        let index = required_date_index(self.config.index.timestamp_to_index(ts))?;
        Ok(self.get(index))
    }

    /// Select a timestamp range (works for all date-based indexes including sub-daily).
    pub fn timestamp_range(
        self,
        start: Timestamp,
        end: Timestamp,
    ) -> Result<RangeBuilder<T, DateSeriesData<T>>> {
        let s = required_date_index(self.config.index.timestamp_to_index(start))?;
        let e = required_date_index(self.config.index.timestamp_to_index(end))?;
        Ok(self.range(s..e))
    }
}

impl<T: DeserializeOwned> DateSeriesEndpoint<T> {
    pub fn new(client: Arc<BitviewClientBase>, name: Arc<str>, index: Index) -> Self {
        Self(SeriesEndpoint::new(client, name, index))
    }

    pub fn get(self, index: usize) -> SingleItemBuilder<T, DateSeriesData<T>> {
        self.0.get(index)
    }

    pub fn range<R: RangeBounds<usize>>(self, range: R) -> RangeBuilder<T, DateSeriesData<T>> {
        self.0.range(range)
    }

    pub fn take(self, n: usize) -> RangeBuilder<T, DateSeriesData<T>> {
        self.0.take(n)
    }

    pub fn last(self, n: usize) -> RangeBuilder<T, DateSeriesData<T>> {
        self.0.last(n)
    }

    pub fn skip(self, n: usize) -> SkippedBuilder<T, DateSeriesData<T>> {
        self.0.skip(n)
    }

    pub fn fetch(self) -> Result<DateSeriesData<T>> {
        self.0.fetch()
    }

    pub fn fetch_csv(self) -> Result<String> {
        self.0.fetch_csv()
    }

    #[allow(clippy::len_without_is_empty)]
    pub fn len(&self) -> Result<i64> {
        self.0.len()
    }

    pub fn version(&self) -> Result<u32> {
        self.0.version()
    }

    pub fn path(&self) -> String {
        self.0.path()
    }

    pub fn get_date(self, date: Date) -> Result<SingleItemBuilder<T, DateSeriesData<T>>> {
        self.0.get_date(date)
    }

    pub fn date_range(self, start: Date, end: Date) -> Result<RangeBuilder<T, DateSeriesData<T>>> {
        self.0.date_range(start, end)
    }

    pub fn get_timestamp(
        self,
        timestamp: Timestamp,
    ) -> Result<SingleItemBuilder<T, DateSeriesData<T>>> {
        self.0.get_timestamp(timestamp)
    }

    pub fn timestamp_range(
        self,
        start: Timestamp,
        end: Timestamp,
    ) -> Result<RangeBuilder<T, DateSeriesData<T>>> {
        self.0.timestamp_range(start, end)
    }
}

/// Builder for single item access.
pub struct SingleItemBuilder<T, D = SeriesData<T>> {
    config: EndpointConfig,
    _marker: std::marker::PhantomData<fn() -> (T, D)>,
}

impl<T: DeserializeOwned, D: DeserializeOwned> SingleItemBuilder<T, D> {
    /// Fetch the single item.
    pub fn fetch(self) -> Result<D> {
        self.config.get_json(None)
    }

    /// Fetch the single item as CSV.
    pub fn fetch_csv(self) -> Result<String> {
        self.config.get_text(Some("csv"))
    }
}

/// Builder after calling `skip(n)`. Chain with `take(n)` to specify count.
pub struct SkippedBuilder<T, D = SeriesData<T>> {
    config: EndpointConfig,
    _marker: std::marker::PhantomData<fn() -> (T, D)>,
}

impl<T: DeserializeOwned, D: DeserializeOwned> SkippedBuilder<T, D> {
    /// Take n items after the skipped position.
    pub fn take(mut self, n: usize) -> RangeBuilder<T, D> {
        let start = self.config.start.unwrap_or(0);
        self.config.end = Some(start + n as i64);
        RangeBuilder {
            config: self.config,
            _marker: std::marker::PhantomData,
        }
    }

    /// Fetch from the skipped position to the end.
    pub fn fetch(self) -> Result<D> {
        self.config.get_json(None)
    }

    /// Fetch from the skipped position to the end as CSV.
    pub fn fetch_csv(self) -> Result<String> {
        self.config.get_text(Some("csv"))
    }
}

/// Builder with range fully specified.
pub struct RangeBuilder<T, D = SeriesData<T>> {
    config: EndpointConfig,
    _marker: std::marker::PhantomData<fn() -> (T, D)>,
}

impl<T: DeserializeOwned, D: DeserializeOwned> RangeBuilder<T, D> {
    /// Fetch the range as parsed JSON.
    pub fn fetch(self) -> Result<D> {
        self.config.get_json(None)
    }

    /// Fetch the range as CSV string.
    pub fn fetch_csv(self) -> Result<String> {
        self.config.get_text(Some("csv"))
    }
}

// Static index arrays
const _I1: &[Index] = &[
    Index::Minute10,
    Index::Minute30,
    Index::Hour1,
    Index::Hour4,
    Index::Hour12,
    Index::Day1,
    Index::Day3,
    Index::Week1,
    Index::Month1,
    Index::Month3,
    Index::Month6,
    Index::Year1,
    Index::Year10,
    Index::Halving,
    Index::Epoch,
    Index::Height,
];
const _I2: &[Index] = &[
    Index::Minute10,
    Index::Minute30,
    Index::Hour1,
    Index::Hour4,
    Index::Hour12,
    Index::Day1,
    Index::Day3,
    Index::Week1,
    Index::Month1,
    Index::Month3,
    Index::Month6,
    Index::Year1,
    Index::Year10,
    Index::Halving,
    Index::Epoch,
    Index::Height,
];
const _I3: &[Index] = &[
    Index::Minute10,
    Index::Minute30,
    Index::Hour1,
    Index::Hour4,
    Index::Hour12,
    Index::Day1,
    Index::Day3,
    Index::Week1,
    Index::Month1,
    Index::Month3,
    Index::Month6,
    Index::Year1,
    Index::Year10,
    Index::Halving,
    Index::Epoch,
    Index::Height,
];
const _I4: &[Index] = &[
    Index::Minute10,
    Index::Minute30,
    Index::Hour1,
    Index::Hour4,
    Index::Hour12,
    Index::Day1,
    Index::Day3,
    Index::Week1,
    Index::Month1,
    Index::Month3,
    Index::Month6,
    Index::Year1,
    Index::Year10,
    Index::Halving,
    Index::Epoch,
];
const _I5: &[Index] = &[
    Index::Minute10,
    Index::Minute30,
    Index::Hour1,
    Index::Hour4,
    Index::Hour12,
    Index::Day1,
    Index::Day3,
    Index::Week1,
    Index::Month1,
    Index::Month3,
    Index::Month6,
    Index::Year1,
    Index::Year10,
    Index::Halving,
    Index::Epoch,
];
const _I6: &[Index] = &[Index::Minute10];
const _I7: &[Index] = &[Index::Minute30];
const _I8: &[Index] = &[Index::Hour1];
const _I9: &[Index] = &[Index::Hour4];
const _I10: &[Index] = &[Index::Hour12];
const _I11: &[Index] = &[Index::Day1];
const _I12: &[Index] = &[Index::Day3];
const _I13: &[Index] = &[Index::Week1];
const _I14: &[Index] = &[Index::Month1];
const _I15: &[Index] = &[Index::Month3];
const _I16: &[Index] = &[Index::Month6];
const _I17: &[Index] = &[Index::Year1];
const _I18: &[Index] = &[Index::Year10];
const _I19: &[Index] = &[Index::Halving];
const _I20: &[Index] = &[Index::Epoch];
const _I21: &[Index] = &[Index::Height];
const _I22: &[Index] = &[Index::TxIndex];
const _I23: &[Index] = &[Index::TxInIndex];
const _I24: &[Index] = &[Index::TxOutIndex];
const _I25: &[Index] = &[Index::EmptyOutputIndex];
const _I26: &[Index] = &[Index::OpReturnIndex];
const _I27: &[Index] = &[Index::P2AAddrIndex];
const _I28: &[Index] = &[Index::P2MSOutputIndex];
const _I29: &[Index] = &[Index::P2PK33AddrIndex];
const _I30: &[Index] = &[Index::P2PK65AddrIndex];
const _I31: &[Index] = &[Index::P2PKHAddrIndex];
const _I32: &[Index] = &[Index::P2SHAddrIndex];
const _I33: &[Index] = &[Index::P2TRAddrIndex];
const _I34: &[Index] = &[Index::P2WPKHAddrIndex];
const _I35: &[Index] = &[Index::P2WSHAddrIndex];
const _I36: &[Index] = &[Index::UnknownOutputIndex];
const _I37: &[Index] = &[Index::FundedAddrIndex];
const _I38: &[Index] = &[Index::ExtendedEmptyAddrIndex];

#[inline]
fn _ep<T: DeserializeOwned>(
    c: &Arc<BitviewClientBase>,
    n: &Arc<str>,
    i: Index,
) -> SeriesEndpoint<T> {
    SeriesEndpoint::new(c.clone(), n.clone(), i)
}

#[inline]
fn _dep<T: DeserializeOwned>(
    c: &Arc<BitviewClientBase>,
    n: &Arc<str>,
    i: Index,
) -> DateSeriesEndpoint<T> {
    DateSeriesEndpoint::new(c.clone(), n.clone(), i)
}

/// A leaf accessor: the series name plus one endpoint method per index (`date` for date indexes),
/// each with its value type (`T::Nullable` where values can be missing); `get` takes the weakest.
macro_rules! accessor {
    ($name:ident, $by:ident, $indexes:ident, $any:ty { $($kind:ident $method:ident: $index:ident -> $value:ty,)* }) => {
        pub struct $by<T> {
            client: Arc<BitviewClientBase>,
            name: Arc<str>,
            _marker: std::marker::PhantomData<T>,
        }
        impl<T: DeserializeOwned + SeriesValue<Nullable: DeserializeOwned>> $by<T> {
            $(accessor!(@method $kind $method $index $value);)*
        }
        pub struct $name<T> {
            name: Arc<str>,
            pub by: $by<T>,
        }
        impl<T: DeserializeOwned> $name<T> {
            pub fn new(client: Arc<BitviewClientBase>, name: String) -> Self {
                let name: Arc<str> = name.into();
                Self { name: name.clone(), by: $by { client, name, _marker: std::marker::PhantomData } }
            }
            pub fn name(&self) -> &str {
                &self.name
            }
        }
        impl<T> AnySeriesPattern for $name<T> {
            fn name(&self) -> &str {
                &self.name
            }
            fn indexes(&self) -> &'static [Index] {
                $indexes
            }
        }
        impl<T: DeserializeOwned + SeriesValue<Nullable: DeserializeOwned>> SeriesPattern<$any> for $name<T> {
            fn get(&self, index: Index) -> Option<SeriesEndpoint<$any>> {
                $indexes.contains(&index).then(|| _ep(&self.by.client, &self.by.name, index))
            }
        }
        impl<T: DeserializeOwned + SeriesValue<Nullable: DeserializeOwned> + Send + Sync + 'static> Node for $name<T> {
            fn build(client: Arc<BitviewClientBase>, name: String) -> Self {
                Self::new(client, name)
            }
            #[cfg(test)]
            fn visit(&self, path: &str, f: &mut dyn FnMut(&str, &dyn AnySeriesPattern)) {
                f(path, self)
            }
        }
    };
    (@method date $method:ident $index:ident $value:ty) => {
        pub fn $method(&self) -> DateSeriesEndpoint<$value> {
            _dep(&self.client, &self.name, Index::$index)
        }
    };
    (@method plain $method:ident $index:ident $value:ty) => {
        pub fn $method(&self) -> SeriesEndpoint<$value> {
            _ep(&self.client, &self.name, Index::$index)
        }
    };
}
accessor! { SeriesPattern1, SeriesPattern1By, _I1, T {
    date minute10: Minute10 -> T,
    date minute30: Minute30 -> T,
    date hour1: Hour1 -> T,
    date hour4: Hour4 -> T,
    date hour12: Hour12 -> T,
    date day1: Day1 -> T,
    date day3: Day3 -> T,
    date week1: Week1 -> T,
    date month1: Month1 -> T,
    date month3: Month3 -> T,
    date month6: Month6 -> T,
    date year1: Year1 -> T,
    date year10: Year10 -> T,
    plain halving: Halving -> T,
    plain epoch: Epoch -> T,
    plain height: Height -> T,
} }
accessor! { SeriesPattern2, SeriesPattern2By, _I2, T::Nullable {
    date minute10: Minute10 -> T::Nullable,
    date minute30: Minute30 -> T::Nullable,
    date hour1: Hour1 -> T::Nullable,
    date hour4: Hour4 -> T::Nullable,
    date hour12: Hour12 -> T::Nullable,
    date day1: Day1 -> T::Nullable,
    date day3: Day3 -> T::Nullable,
    date week1: Week1 -> T::Nullable,
    date month1: Month1 -> T::Nullable,
    date month3: Month3 -> T::Nullable,
    date month6: Month6 -> T::Nullable,
    date year1: Year1 -> T::Nullable,
    date year10: Year10 -> T::Nullable,
    plain halving: Halving -> T,
    plain epoch: Epoch -> T,
    plain height: Height -> T,
} }
accessor! { SeriesPattern3, SeriesPattern3By, _I3, T::Nullable {
    date minute10: Minute10 -> T::Nullable,
    date minute30: Minute30 -> T::Nullable,
    date hour1: Hour1 -> T::Nullable,
    date hour4: Hour4 -> T::Nullable,
    date hour12: Hour12 -> T::Nullable,
    date day1: Day1 -> T::Nullable,
    date day3: Day3 -> T::Nullable,
    date week1: Week1 -> T::Nullable,
    date month1: Month1 -> T::Nullable,
    date month3: Month3 -> T::Nullable,
    date month6: Month6 -> T::Nullable,
    date year1: Year1 -> T::Nullable,
    date year10: Year10 -> T::Nullable,
    plain halving: Halving -> T::Nullable,
    plain epoch: Epoch -> T::Nullable,
    plain height: Height -> T::Nullable,
} }
accessor! { SeriesPattern4, SeriesPattern4By, _I4, T {
    date minute10: Minute10 -> T,
    date minute30: Minute30 -> T,
    date hour1: Hour1 -> T,
    date hour4: Hour4 -> T,
    date hour12: Hour12 -> T,
    date day1: Day1 -> T,
    date day3: Day3 -> T,
    date week1: Week1 -> T,
    date month1: Month1 -> T,
    date month3: Month3 -> T,
    date month6: Month6 -> T,
    date year1: Year1 -> T,
    date year10: Year10 -> T,
    plain halving: Halving -> T,
    plain epoch: Epoch -> T,
} }
accessor! { SeriesPattern5, SeriesPattern5By, _I5, T::Nullable {
    date minute10: Minute10 -> T::Nullable,
    date minute30: Minute30 -> T::Nullable,
    date hour1: Hour1 -> T::Nullable,
    date hour4: Hour4 -> T::Nullable,
    date hour12: Hour12 -> T::Nullable,
    date day1: Day1 -> T::Nullable,
    date day3: Day3 -> T::Nullable,
    date week1: Week1 -> T::Nullable,
    date month1: Month1 -> T::Nullable,
    date month3: Month3 -> T::Nullable,
    date month6: Month6 -> T::Nullable,
    date year1: Year1 -> T::Nullable,
    date year10: Year10 -> T::Nullable,
    plain halving: Halving -> T,
    plain epoch: Epoch -> T,
} }
accessor! { SeriesPattern6, SeriesPattern6By, _I6, T {
    date minute10: Minute10 -> T,
} }
accessor! { SeriesPattern7, SeriesPattern7By, _I7, T {
    date minute30: Minute30 -> T,
} }
accessor! { SeriesPattern8, SeriesPattern8By, _I8, T {
    date hour1: Hour1 -> T,
} }
accessor! { SeriesPattern9, SeriesPattern9By, _I9, T {
    date hour4: Hour4 -> T,
} }
accessor! { SeriesPattern10, SeriesPattern10By, _I10, T {
    date hour12: Hour12 -> T,
} }
accessor! { SeriesPattern11, SeriesPattern11By, _I11, T {
    date day1: Day1 -> T,
} }
accessor! { SeriesPattern12, SeriesPattern12By, _I12, T {
    date day3: Day3 -> T,
} }
accessor! { SeriesPattern13, SeriesPattern13By, _I13, T {
    date week1: Week1 -> T,
} }
accessor! { SeriesPattern14, SeriesPattern14By, _I14, T {
    date month1: Month1 -> T,
} }
accessor! { SeriesPattern15, SeriesPattern15By, _I15, T {
    date month3: Month3 -> T,
} }
accessor! { SeriesPattern16, SeriesPattern16By, _I16, T {
    date month6: Month6 -> T,
} }
accessor! { SeriesPattern17, SeriesPattern17By, _I17, T {
    date year1: Year1 -> T,
} }
accessor! { SeriesPattern18, SeriesPattern18By, _I18, T {
    date year10: Year10 -> T,
} }
accessor! { SeriesPattern19, SeriesPattern19By, _I19, T {
    plain halving: Halving -> T,
} }
accessor! { SeriesPattern20, SeriesPattern20By, _I20, T {
    plain epoch: Epoch -> T,
} }
accessor! { SeriesPattern21, SeriesPattern21By, _I21, T {
    plain height: Height -> T,
} }
accessor! { SeriesPattern22, SeriesPattern22By, _I22, T {
    plain tx_index: TxIndex -> T,
} }
accessor! { SeriesPattern23, SeriesPattern23By, _I23, T {
    plain txin_index: TxInIndex -> T,
} }
accessor! { SeriesPattern24, SeriesPattern24By, _I24, T {
    plain txout_index: TxOutIndex -> T,
} }
accessor! { SeriesPattern25, SeriesPattern25By, _I25, T {
    plain empty_output_index: EmptyOutputIndex -> T,
} }
accessor! { SeriesPattern26, SeriesPattern26By, _I26, T {
    plain op_return_index: OpReturnIndex -> T,
} }
accessor! { SeriesPattern27, SeriesPattern27By, _I27, T {
    plain p2a_addr_index: P2AAddrIndex -> T,
} }
accessor! { SeriesPattern28, SeriesPattern28By, _I28, T {
    plain p2ms_output_index: P2MSOutputIndex -> T,
} }
accessor! { SeriesPattern29, SeriesPattern29By, _I29, T {
    plain p2pk33_addr_index: P2PK33AddrIndex -> T,
} }
accessor! { SeriesPattern30, SeriesPattern30By, _I30, T {
    plain p2pk65_addr_index: P2PK65AddrIndex -> T,
} }
accessor! { SeriesPattern31, SeriesPattern31By, _I31, T {
    plain p2pkh_addr_index: P2PKHAddrIndex -> T,
} }
accessor! { SeriesPattern32, SeriesPattern32By, _I32, T {
    plain p2sh_addr_index: P2SHAddrIndex -> T,
} }
accessor! { SeriesPattern33, SeriesPattern33By, _I33, T {
    plain p2tr_addr_index: P2TRAddrIndex -> T,
} }
accessor! { SeriesPattern34, SeriesPattern34By, _I34, T {
    plain p2wpkh_addr_index: P2WPKHAddrIndex -> T,
} }
accessor! { SeriesPattern35, SeriesPattern35By, _I35, T {
    plain p2wsh_addr_index: P2WSHAddrIndex -> T,
} }
accessor! { SeriesPattern36, SeriesPattern36By, _I36, T {
    plain unknown_output_index: UnknownOutputIndex -> T,
} }
accessor! { SeriesPattern37, SeriesPattern37By, _I37, T {
    plain funded_addr_index: FundedAddrIndex -> T,
} }
accessor! { SeriesPattern38, SeriesPattern38By, _I38, T {
    plain extended_empty_addr_index: ExtendedEmptyAddrIndex -> T,
} }

/// A series value type, as a type parameter of the leaf accessors.
pub trait SeriesValue {
    /// The value where it can be missing (e.g. a period without blocks): `Option<Self>`, or the
    /// value itself when it is already optional.
    type Nullable;
}

impl<T> SeriesValue for Option<T> {
    type Nullable = Option<T>;
}

/// A series-tree node or leaf, built from its series name or base.
pub(crate) trait Node: Sized + Send + Sync + 'static {
    fn build(client: Arc<BitviewClientBase>, name: String) -> Self;
    #[cfg(test)]
    fn visit(&self, path: &str, f: &mut dyn FnMut(&str, &dyn AnySeriesPattern));
}

/// A child named by `template`: `*` stands for the parent's base, and with an empty base the `_`
/// joining it goes too.
fn child<T: Node>(
    client: &Arc<BitviewClientBase>,
    base: &Arc<str>,
    template: &'static str,
) -> LazyNode<T> {
    let (client, base) = (client.clone(), base.clone());
    LazyLock::new(Box::new(move || {
        let name = if !base.is_empty() {
            template.replacen('*', &base, 1)
        } else if let Some(rest) = template.strip_prefix("*_") {
            rest.to_owned()
        } else {
            template.replacen("_*", "", 1).replacen('*', "", 1)
        };
        Box::new(T::build(client, name))
    }))
}

/// A series-tree shape: a struct with one lazy field per child, documented with its canonical path.
macro_rules! shape {
    ($name:ident $(<$($p:ident),+>)? at $at:literal { $($field:ident: $ty:ty = $template:literal,)* }) => {
        #[doc = concat!("Series-tree node, e.g. at `", $at, "`.")]
        pub struct $name $(<$($p),+>)? { $(pub $field: LazyNode<$ty>,)* }
        impl $(<$($p: Send + Sync + 'static),+>)? Node for $name $(<$($p),+>)? where $($ty: Node,)* {
            fn build(client: Arc<BitviewClientBase>, base: String) -> Self {
                let base: Arc<str> = base.into();
                Self { $($field: child(&client, &base, $template),)* }
            }
            #[cfg(test)]
            fn visit(&self, path: &str, f: &mut dyn FnMut(&str, &dyn AnySeriesPattern)) {
                $(self.$field.visit(&format!("{path}{}{}", if path.is_empty() { "" } else { "." }, stringify!($field)), f);)*
            }
        }
    };
}
impl SeriesValue for Addr {
    type Nullable = Option<Addr>;
}
impl SeriesValue for AddrState {
    type Nullable = Option<AddrState>;
}
impl SeriesValue for BlockHash {
    type Nullable = Option<BlockHash>;
}
impl SeriesValue for Bytes {
    type Nullable = Option<Bytes>;
}
impl SeriesValue for CapitalSentimentPhase {
    type Nullable = Option<CapitalSentimentPhase>;
}
impl SeriesValue for CentsSigned {
    type Nullable = Option<CentsSigned>;
}
impl SeriesValue for CoinbaseTag {
    type Nullable = Option<CoinbaseTag>;
}
impl SeriesValue for Date {
    type Nullable = Option<Date>;
}
impl SeriesValue for Day1 {
    type Nullable = Option<Day1>;
}
impl SeriesValue for Day3 {
    type Nullable = Option<Day3>;
}
impl SeriesValue for EmptyAddrData {
    type Nullable = Option<EmptyAddrData>;
}
impl SeriesValue for EmptyOutputIndex {
    type Nullable = Option<EmptyOutputIndex>;
}
impl SeriesValue for Epoch {
    type Nullable = Option<Epoch>;
}
impl SeriesValue for FundedAddrData {
    type Nullable = Option<FundedAddrData>;
}
impl SeriesValue for Halving {
    type Nullable = Option<Halving>;
}
impl SeriesValue for Height {
    type Nullable = Option<Height>;
}
impl SeriesValue for Hour1 {
    type Nullable = Option<Hour1>;
}
impl SeriesValue for Hour12 {
    type Nullable = Option<Hour12>;
}
impl SeriesValue for Hour4 {
    type Nullable = Option<Hour4>;
}
impl SeriesValue for Minute10 {
    type Nullable = Option<Minute10>;
}
impl SeriesValue for Minute30 {
    type Nullable = Option<Minute30>;
}
impl SeriesValue for Month1 {
    type Nullable = Option<Month1>;
}
impl SeriesValue for Month3 {
    type Nullable = Option<Month3>;
}
impl SeriesValue for Month6 {
    type Nullable = Option<Month6>;
}
impl SeriesValue for OHLCCents {
    type Nullable = Option<OHLCCents>;
}
impl SeriesValue for OHLCDollars {
    type Nullable = Option<OHLCDollars>;
}
impl SeriesValue for OHLCSats {
    type Nullable = Option<OHLCSats>;
}
impl SeriesValue for OpReturnIndex {
    type Nullable = Option<OpReturnIndex>;
}
impl SeriesValue for OpReturnKind {
    type Nullable = Option<OpReturnKind>;
}
impl SeriesValue for OutPoint {
    type Nullable = Option<OutPoint>;
}
impl SeriesValue for OutputType {
    type Nullable = Option<OutputType>;
}
impl SeriesValue for P2AAddrIndex {
    type Nullable = Option<P2AAddrIndex>;
}
impl SeriesValue for P2ABytes {
    type Nullable = Option<P2ABytes>;
}
impl SeriesValue for P2MSOutputIndex {
    type Nullable = Option<P2MSOutputIndex>;
}
impl SeriesValue for P2PK33AddrIndex {
    type Nullable = Option<P2PK33AddrIndex>;
}
impl SeriesValue for P2PK33Bytes {
    type Nullable = Option<P2PK33Bytes>;
}
impl SeriesValue for P2PK65AddrIndex {
    type Nullable = Option<P2PK65AddrIndex>;
}
impl SeriesValue for P2PK65Bytes {
    type Nullable = Option<P2PK65Bytes>;
}
impl SeriesValue for P2PKHAddrIndex {
    type Nullable = Option<P2PKHAddrIndex>;
}
impl SeriesValue for P2PKHBytes {
    type Nullable = Option<P2PKHBytes>;
}
impl SeriesValue for P2SHAddrIndex {
    type Nullable = Option<P2SHAddrIndex>;
}
impl SeriesValue for P2SHBytes {
    type Nullable = Option<P2SHBytes>;
}
impl SeriesValue for P2TRAddrIndex {
    type Nullable = Option<P2TRAddrIndex>;
}
impl SeriesValue for P2TRBytes {
    type Nullable = Option<P2TRBytes>;
}
impl SeriesValue for P2WPKHAddrIndex {
    type Nullable = Option<P2WPKHAddrIndex>;
}
impl SeriesValue for P2WPKHBytes {
    type Nullable = Option<P2WPKHBytes>;
}
impl SeriesValue for P2WSHAddrIndex {
    type Nullable = Option<P2WSHAddrIndex>;
}
impl SeriesValue for P2WSHBytes {
    type Nullable = Option<P2WSHBytes>;
}
impl SeriesValue for PoolSlug {
    type Nullable = Option<PoolSlug>;
}
impl SeriesValue for RawLockTime {
    type Nullable = Option<RawLockTime>;
}
impl SeriesValue for Sats {
    type Nullable = Option<Sats>;
}
impl SeriesValue for SatsSigned {
    type Nullable = Option<SatsSigned>;
}
impl SeriesValue for SigOps {
    type Nullable = Option<SigOps>;
}
impl SeriesValue for StoredBool {
    type Nullable = Option<StoredBool>;
}
impl SeriesValue for StoredI64 {
    type Nullable = Option<StoredI64>;
}
impl SeriesValue for StoredI8 {
    type Nullable = Option<StoredI8>;
}
impl SeriesValue for StoredU16 {
    type Nullable = Option<StoredU16>;
}
impl SeriesValue for StoredU32 {
    type Nullable = Option<StoredU32>;
}
impl SeriesValue for StoredU64 {
    type Nullable = Option<StoredU64>;
}
impl SeriesValue for StoredU8 {
    type Nullable = Option<StoredU8>;
}
impl SeriesValue for Timestamp {
    type Nullable = Option<Timestamp>;
}
impl SeriesValue for TxInIndex {
    type Nullable = Option<TxInIndex>;
}
impl SeriesValue for TxIndex {
    type Nullable = Option<TxIndex>;
}
impl SeriesValue for TxOutIndex {
    type Nullable = Option<TxOutIndex>;
}
impl SeriesValue for TxVersion {
    type Nullable = Option<TxVersion>;
}
impl SeriesValue for Txid {
    type Nullable = Option<Txid>;
}
impl SeriesValue for TypeIndex {
    type Nullable = Option<TypeIndex>;
}
impl SeriesValue for UnknownOutputIndex {
    type Nullable = Option<UnknownOutputIndex>;
}
impl SeriesValue for VSize {
    type Nullable = Option<VSize>;
}
impl SeriesValue for Week1 {
    type Nullable = Option<Week1>;
}
impl SeriesValue for Weight {
    type Nullable = Option<Weight>;
}
impl SeriesValue for Weight64 {
    type Nullable = Option<Weight64>;
}
impl SeriesValue for Year1 {
    type Nullable = Option<Year1>;
}
impl SeriesValue for Year10 {
    type Nullable = Option<Year10>;
}
/// The series tree's node types, one generic struct per shape.
pub mod tree {
    use super::*;

    shape! { UtxoHistory at "series().utxo_history" {
        supply: SeriesPattern21<Sats> = "*",
        count: SeriesPattern2<StoredU64> = "utxo_count_bis",
    } }
    shape! { Velocity at "series().supply.velocity" {
        native: SeriesPattern2<Option<StoredF64>> = "*_btc",
        fiat: SeriesPattern2<Option<StoredF64>> = "*_usd",
    } }
    shape! { SoprRatioExtended at "series().distribution_aggregated.cohorts.all.ratios.sopr_ratio_extended" {
        _1w: SeriesPattern2<Option<StoredF32>> = "*_1w",
        _1m: SeriesPattern2<Option<StoredF32>> = "*_1m",
        _1y: SeriesPattern2<Option<StoredF32>> = "*_1y",
    } }
    shape! { Close at "series().price.split.close" {
        usd: SeriesPattern5<Option<Dollars>> = "*",
        cents: SeriesPattern5<Option<Cents>> = "*_cents",
        sats: SeriesPattern5<Sats> = "*_sats",
    } }
    shape! { Ohlc<A, B, C> at "series().price.ohlc" {
        usd: SeriesPattern4<A> = "*",
        cents: SeriesPattern4<B> = "*_cents",
        sats: SeriesPattern4<C> = "*_sats",
    } }
    shape! { Split at "series().price.split" {
        open: Ohlc<Option<Dollars>, Option<Cents>, Sats> = "*_open",
        high: Ohlc<Option<Dollars>, Option<Cents>, Sats> = "*_high",
        low: Ohlc<Option<Dollars>, Option<Cents>, Sats> = "*_low",
        close: Close = "*_close",
    } }
    shape! { Macd1m at "series().market.technical.macd._1m" {
        ema_fast: SeriesPattern2<Option<StoredF32>> = "macd_ema_fast_*",
        ema_slow: SeriesPattern2<Option<StoredF32>> = "macd_ema_slow_*",
        line: SeriesPattern2<Option<StoredF32>> = "macd_line_*",
        signal: SeriesPattern2<Option<StoredF32>> = "macd_signal_*",
        histogram: SeriesPattern2<Option<StoredF32>> = "macd_histogram_*",
    } }
    shape! { Sd24h1m at "series().market.returns.sd_24h._1m" {
        sma: SeriesPattern2<Option<StoredF32>> = "price_return_24h_sma_*",
        sd: SeriesPattern2<Option<StoredF32>> = "price_return_24h_sd_*",
    } }
    shape! { Dormancy at "series().indicators.dormancy" {
        supply_adj: SeriesPattern2<Option<StoredF32>> = "*_supply_adj",
        flow: SeriesPattern2<Option<StoredF32>> = "*_flow",
    } }
    shape! { Nvt at "series().indicators.nvt" {
        bps: SeriesPattern2<Option<BasisPoints32>> = "*_bps",
        ratio: SeriesPattern2<Option<StoredF32>> = "*",
    } }
    shape! { MappingsTimestamp at "series().mappings.timestamp" {
        monotonic: SeriesPattern21<Timestamp> = "*_monotonic",
        resolutions: SeriesPattern4<Timestamp> = "*",
    } }
    shape! { TxoutIndex at "series().mappings.txout_index" {
        identity: SeriesPattern24<TxOutIndex> = "*",
    } }
    shape! { TxinIndex at "series().mappings.txin_index" {
        identity: SeriesPattern23<TxInIndex> = "*",
    } }
    shape! { MappingsTxIndex at "series().mappings.tx_index" {
        identity: SeriesPattern22<TxIndex> = "tx_index",
        input_count: SeriesPattern22<StoredU64> = "input_*",
        output_count: SeriesPattern22<StoredU64> = "output_*",
    } }
    shape! { MappingsYear10 at "series().mappings.year10" {
        date: SeriesPattern18<Date> = "*",
        first_height: SeriesPattern18<Height> = "first_height",
    } }
    shape! { MappingsYear1 at "series().mappings.year1" {
        date: SeriesPattern17<Date> = "*",
        first_height: SeriesPattern17<Height> = "first_height",
    } }
    shape! { MappingsMonth6 at "series().mappings.month6" {
        date: SeriesPattern16<Date> = "*",
        first_height: SeriesPattern16<Height> = "first_height",
    } }
    shape! { MappingsMonth3 at "series().mappings.month3" {
        date: SeriesPattern15<Date> = "*",
        first_height: SeriesPattern15<Height> = "first_height",
    } }
    shape! { MappingsMonth1 at "series().mappings.month1" {
        date: SeriesPattern14<Date> = "*",
        first_height: SeriesPattern14<Height> = "first_height",
    } }
    shape! { MappingsWeek1 at "series().mappings.week1" {
        date: SeriesPattern13<Date> = "*",
        first_height: SeriesPattern13<Height> = "first_height",
    } }
    shape! { MappingsDay3 at "series().mappings.day3" {
        date: SeriesPattern12<Date> = "*",
        first_height: SeriesPattern12<Height> = "first_height",
    } }
    shape! { MappingsDay1 at "series().mappings.day1" {
        date: SeriesPattern11<Date> = "*",
        first_height: SeriesPattern11<Height> = "first_height",
    } }
    shape! { MappingsHour12 at "series().mappings.hour12" {
        first_height: SeriesPattern10<Height> = "*",
    } }
    shape! { MappingsHour4 at "series().mappings.hour4" {
        first_height: SeriesPattern9<Height> = "*",
    } }
    shape! { MappingsHour1 at "series().mappings.hour1" {
        first_height: SeriesPattern8<Height> = "*",
    } }
    shape! { MappingsMinute30 at "series().mappings.minute30" {
        first_height: SeriesPattern7<Height> = "*",
    } }
    shape! { MappingsMinute10 at "series().mappings.minute10" {
        first_height: SeriesPattern6<Height> = "*",
    } }
    shape! { MappingsHalving at "series().mappings.halving" {
        first_height: SeriesPattern19<Height> = "*",
    } }
    shape! { MappingsEpoch at "series().mappings.epoch" {
        first_height: SeriesPattern20<Height> = "*",
    } }
    shape! { MappingsHeight at "series().mappings.height" {
        minute10: SeriesPattern21<Minute10> = "*",
        minute30: SeriesPattern21<Minute30> = "minute30",
        hour1: SeriesPattern21<Hour1> = "hour1",
        hour4: SeriesPattern21<Hour4> = "hour4",
        hour12: SeriesPattern21<Hour12> = "hour12",
        day1: SeriesPattern21<Day1> = "day1",
        day3: SeriesPattern21<Day3> = "day3",
        epoch: SeriesPattern21<Epoch> = "epoch",
        halving: SeriesPattern21<Halving> = "halving",
        week1: SeriesPattern21<Week1> = "week1",
        month1: SeriesPattern21<Month1> = "month1",
        month3: SeriesPattern21<Month3> = "month3",
        month6: SeriesPattern21<Month6> = "month6",
        year1: SeriesPattern21<Year1> = "year1",
        year10: SeriesPattern21<Year10> = "year10",
        tx_index_count: SeriesPattern21<StoredU64> = "tx_index_count",
    } }
    shape! { AddrOpReturn at "series().mappings.addr.op_return" {
        identity: SeriesPattern26<OpReturnIndex> = "*",
    } }
    shape! { AddrUnknown at "series().mappings.addr.unknown" {
        identity: SeriesPattern36<UnknownOutputIndex> = "*",
    } }
    shape! { AddrEmpty at "series().mappings.addr.empty" {
        identity: SeriesPattern25<EmptyOutputIndex> = "*",
    } }
    shape! { AddrP2ms at "series().mappings.addr.p2ms" {
        identity: SeriesPattern28<P2MSOutputIndex> = "*",
    } }
    shape! { AddrP2a at "series().mappings.addr.p2a" {
        identity: SeriesPattern27<P2AAddrIndex> = "*_index",
        addr: SeriesPattern27<Addr> = "*",
    } }
    shape! { AddrP2wsh at "series().mappings.addr.p2wsh" {
        identity: SeriesPattern35<P2WSHAddrIndex> = "*_index",
        addr: SeriesPattern35<Addr> = "*",
    } }
    shape! { AddrP2wpkh at "series().mappings.addr.p2wpkh" {
        identity: SeriesPattern34<P2WPKHAddrIndex> = "*_index",
        addr: SeriesPattern34<Addr> = "*",
    } }
    shape! { AddrP2tr at "series().mappings.addr.p2tr" {
        identity: SeriesPattern33<P2TRAddrIndex> = "*_index",
        addr: SeriesPattern33<Addr> = "*",
    } }
    shape! { AddrP2sh at "series().mappings.addr.p2sh" {
        identity: SeriesPattern32<P2SHAddrIndex> = "*_index",
        addr: SeriesPattern32<Addr> = "*",
    } }
    shape! { AddrP2pkh at "series().mappings.addr.p2pkh" {
        identity: SeriesPattern31<P2PKHAddrIndex> = "*_index",
        addr: SeriesPattern31<Addr> = "*",
    } }
    shape! { AddrP2pk65 at "series().mappings.addr.p2pk65" {
        identity: SeriesPattern30<P2PK65AddrIndex> = "*_index",
        addr: SeriesPattern30<Addr> = "*",
    } }
    shape! { AddrP2pk33 at "series().mappings.addr.p2pk33" {
        identity: SeriesPattern29<P2PK33AddrIndex> = "*_index",
        addr: SeriesPattern29<Addr> = "*",
    } }
    shape! { MappingsAddr at "series().mappings.addr" {
        p2pk33: AddrP2pk33 = "p2pk33_*",
        p2pk65: AddrP2pk65 = "p2pk65_*",
        p2pkh: AddrP2pkh = "p2pkh_*",
        p2sh: AddrP2sh = "p2sh_*",
        p2tr: AddrP2tr = "p2tr_*",
        p2wpkh: AddrP2wpkh = "p2wpkh_*",
        p2wsh: AddrP2wsh = "p2wsh_*",
        p2a: AddrP2a = "p2a_*",
        p2ms: AddrP2ms = "p2ms_output_index",
        empty: AddrEmpty = "empty_output_index",
        unknown: AddrUnknown = "unknown_output_index",
        op_return: AddrOpReturn = "op_return_index",
    } }
    shape! { Mappings at "series().mappings" {
        addr: MappingsAddr = "addr",
        height: MappingsHeight = "minute10",
        epoch: MappingsEpoch = "first_height",
        halving: MappingsHalving = "first_height",
        minute10: MappingsMinute10 = "first_height",
        minute30: MappingsMinute30 = "first_height",
        hour1: MappingsHour1 = "first_height",
        hour4: MappingsHour4 = "first_height",
        hour12: MappingsHour12 = "first_height",
        day1: MappingsDay1 = "*",
        day3: MappingsDay3 = "*",
        week1: MappingsWeek1 = "*",
        month1: MappingsMonth1 = "*",
        month3: MappingsMonth3 = "*",
        month6: MappingsMonth6 = "*",
        year1: MappingsYear1 = "*",
        year10: MappingsYear10 = "*",
        tx_index: MappingsTxIndex = "count",
        txin_index: TxinIndex = "txin_index",
        txout_index: TxoutIndex = "txout_index",
        timestamp: MappingsTimestamp = "timestamp",
    } }
    shape! { Constants at "series().constants" {
        _0: SeriesPattern1<StoredU16> = "*_0",
        _1: SeriesPattern1<StoredU16> = "*_1",
        _2: SeriesPattern1<StoredU16> = "*_2",
        _3: SeriesPattern1<StoredU16> = "*_3",
        _4: SeriesPattern1<StoredU16> = "*_4",
        _20: SeriesPattern1<StoredU16> = "*_20",
        _30: SeriesPattern1<StoredU16> = "*_30",
        _38_2: SeriesPattern1<Option<StoredF32>> = "*_38_2",
        _50: SeriesPattern1<StoredU16> = "*_50",
        _61_8: SeriesPattern1<Option<StoredF32>> = "*_61_8",
        _70: SeriesPattern1<StoredU16> = "*_70",
        _80: SeriesPattern1<StoredU16> = "*_80",
        _100: SeriesPattern1<StoredU16> = "*_100",
        _600: SeriesPattern1<StoredU16> = "*_600",
        minus_1: SeriesPattern1<StoredI8> = "*_minus_1",
        minus_2: SeriesPattern1<StoredI8> = "*_minus_2",
        minus_3: SeriesPattern1<StoredI8> = "*_minus_3",
        minus_4: SeriesPattern1<StoredI8> = "*_minus_4",
    } }
    shape! { CapitalSentiment at "series().capital_sentiment" {
        is_long: SeriesPattern2<StoredBool> = "*_is_long",
        is_short: SeriesPattern2<StoredBool> = "*_is_short",
        phase: SeriesPattern3<CapitalSentimentPhase> = "*_phase",
        score: SeriesPattern3<StoredI8> = "*_score",
    } }
    shape! { SupplyInLossThreshold at "series().bedrock.coinflow.supply_in_loss_threshold" {
        pct95: SeriesPattern2<Option<StoredF64>> = "*_pct95_ratio",
        pct98: SeriesPattern2<Option<StoredF64>> = "*_pct98_ratio",
        pct99: SeriesPattern2<Option<StoredF64>> = "*_pct99_ratio",
        pct99_5: SeriesPattern2<Option<StoredF64>> = "*_pct99_5_ratio",
        pct99_9: SeriesPattern2<Option<StoredF64>> = "*_pct99_9_ratio",
    } }
    shape! { ReserveRisk at "series().cointime.reserve_risk" {
        value: SeriesPattern2<Option<StoredF64>> = "*",
        vocdd_median_1y: SeriesPattern21<Option<StoredF64>> = "vocdd_median_1y",
        hodl_bank: SeriesPattern21<Option<StoredF64>> = "hodl_bank",
    } }
    shape! { RhodlRatio<A> at "series().indicators.rhodl_ratio" {
        ppm: SeriesPattern2<A> = "*_ppm",
        ratio: SeriesPattern2<Option<StoredF32>> = "*",
    } }
    shape! { Share at "series().cointime.supply.active.in_loss.share" {
        bounded: SeriesPattern2<Option<BoundedRatio>> = "*_bounded",
        ratio: SeriesPattern2<Option<StoredF64>> = "*",
    } }
    shape! { ActiveInLoss at "series().cointime.supply.active.in_loss" {
        share: Share = "*",
    } }
    shape! { Active at "series().cointime.supply.active" {
        btc: SeriesPattern2<Option<Bitcoin>> = "active_*",
        sats: SeriesPattern2<Sats> = "active_*_sats",
        usd: SeriesPattern2<Option<Dollars>> = "active_*_usd",
        cents: SeriesPattern2<Option<Cents>> = "active_*_cents",
        in_loss: ActiveInLoss = "cointime_*_in_loss_share",
    } }
    shape! { MobileInLoss at "series().coinflow.supply.mobile.in_loss" {
        share: SeriesPattern2<Option<StoredF64>> = "*",
    } }
    shape! { Mobile at "series().coinflow.supply.mobile" {
        btc: SeriesPattern2<Option<Bitcoin>> = "*_mobile_supply",
        sats: SeriesPattern2<Sats> = "*_mobile_supply_sats",
        usd: SeriesPattern2<Option<Dollars>> = "*_mobile_supply_usd",
        cents: SeriesPattern2<Option<Cents>> = "*_mobile_supply_cents",
        in_loss: MobileInLoss = "*_coinflow_supply_in_loss_share",
    } }
    shape! { AwakeSupply at "series().cointime.awake.supply" {
        btc: SeriesPattern2<Option<Bitcoin>> = "*",
        sats: SeriesPattern2<Sats> = "*_sats",
        usd: SeriesPattern2<Option<Dollars>> = "*_usd",
        cents: SeriesPattern2<Option<Cents>> = "*_cents",
        in_loss: MobileInLoss = "*_in_loss_share",
    } }
    shape! { CapitalizedPrice at "series().coinflow.capitalized_price" {
        usd: SeriesPattern2<Option<Dollars>> = "*",
        cents: SeriesPattern2<Option<Cents>> = "*_cents",
        sats: SeriesPattern2<Option<SatsFract>> = "*_sats",
        ppm: SeriesPattern2<Option<PriceRatio>> = "*_ratio_ppm",
        ratio: SeriesPattern2<Option<StoredF32>> = "*_ratio",
    } }
    shape! { Ema at "series().market.moving_average.ema" {
        _1w: CapitalizedPrice = "*_1w",
        _8d: CapitalizedPrice = "*_8d",
        _12d: CapitalizedPrice = "*_12d",
        _13d: CapitalizedPrice = "*_13d",
        _21d: CapitalizedPrice = "*_21d",
        _26d: CapitalizedPrice = "*_26d",
        _1m: CapitalizedPrice = "*_1m",
        _34d: CapitalizedPrice = "*_34d",
        _55d: CapitalizedPrice = "*_55d",
        _89d: CapitalizedPrice = "*_89d",
        _144d: CapitalizedPrice = "*_144d",
        _200d: CapitalizedPrice = "*_200d",
        _1y: CapitalizedPrice = "*_1y",
        _2y: CapitalizedPrice = "*_2y",
        _200w: CapitalizedPrice = "*_200w",
        _4y: CapitalizedPrice = "*_4y",
    } }
    shape! { CointimePrices at "series().cointime.prices" {
        vaulted: CapitalizedPrice = "vaulted_*",
        active: CapitalizedPrice = "active_*",
        true_market_mean: CapitalizedPrice = "true_market_mean",
        cointime: CapitalizedPrice = "cointime_*",
    } }
    shape! { Spot<A> at "series().price.spot" {
        usd: SeriesPattern2<Option<Dollars>> = "*",
        cents: SeriesPattern2<Option<Cents>> = "*_cents",
        sats: SeriesPattern2<A> = "*_sats",
    } }
    shape! { AgeBoundsAll at "series().cohorts.urpd.age_bounds.all" {
        min: Spot<Option<SatsFract>> = "*_min",
        max: Spot<Option<SatsFract>> = "*_max",
    } }
    shape! { AgeBounds at "series().cohorts.urpd.age_bounds" {
        all: AgeBoundsAll = "*_all_cost_basis",
        sth: AgeBoundsAll = "*_sth_cost_basis",
        lth: AgeBoundsAll = "*_lth_cost_basis",
        under_4m: AgeBoundsAll = "*_under_4m_cost_basis",
        under_6m: AgeBoundsAll = "*_under_6m_cost_basis",
        over_4m: AgeBoundsAll = "*_over_4m_cost_basis",
        over_6m: AgeBoundsAll = "*_over_6m_cost_basis",
    } }
    shape! { CohortsUrpd at "series().cohorts.urpd" {
        age_bounds: AgeBounds = "*",
    } }
    shape! { Price at "series().price" {
        split: Split = "*",
        ohlc: Ohlc<OHLCDollars, OHLCCents, OHLCSats> = "*_ohlc",
        spot: Spot<Sats> = "*",
    } }
    shape! { Sma350d at "series().market.moving_average.sma._350d" {
        usd: SeriesPattern2<Option<Dollars>> = "*",
        cents: SeriesPattern2<Option<Cents>> = "*_cents",
        sats: SeriesPattern2<Option<SatsFract>> = "*_sats",
        ppm: SeriesPattern2<Option<PriceRatio>> = "*_ratio_ppm",
        ratio: SeriesPattern2<Option<StoredF32>> = "*_ratio",
        x2: Spot<Option<SatsFract>> = "*_x2",
    } }
    shape! { Sma200d at "series().market.moving_average.sma._200d" {
        usd: SeriesPattern2<Option<Dollars>> = "*",
        cents: SeriesPattern2<Option<Cents>> = "*_cents",
        sats: SeriesPattern2<Option<SatsFract>> = "*_sats",
        ppm: SeriesPattern2<Option<PriceRatio>> = "*_ratio_ppm",
        ratio: SeriesPattern2<Option<StoredF32>> = "*_ratio",
        x2_4: Spot<Option<SatsFract>> = "*_x2_4",
        x0_8: Spot<Option<SatsFract>> = "*_x0_8",
    } }
    shape! { MovingAverageSma at "series().market.moving_average.sma" {
        _1w: CapitalizedPrice = "*_1w",
        _8d: CapitalizedPrice = "*_8d",
        _13d: CapitalizedPrice = "*_13d",
        _21d: CapitalizedPrice = "*_21d",
        _1m: CapitalizedPrice = "*_1m",
        _34d: CapitalizedPrice = "*_34d",
        _50d: CapitalizedPrice = "*_50d",
        _55d: CapitalizedPrice = "*_55d",
        _89d: CapitalizedPrice = "*_89d",
        _111d: CapitalizedPrice = "*_111d",
        _144d: CapitalizedPrice = "*_144d",
        _200d: Sma200d = "*_200d",
        _350d: Sma350d = "*_350d",
        _1y: CapitalizedPrice = "*_1y",
        _2y: CapitalizedPrice = "*_2y",
        _200w: CapitalizedPrice = "*_200w",
        _4y: CapitalizedPrice = "*_4y",
    } }
    shape! { MovingAverage at "series().market.moving_average" {
        sma: MovingAverageSma = "*_sma",
        ema: Ema = "*_ema",
    } }
    shape! { Max at "series().market.range.max" {
        _1w: Spot<Option<SatsFract>> = "*_1w",
        _2w: Spot<Option<SatsFract>> = "*_2w",
        _1m: Spot<Option<SatsFract>> = "*_1m",
        _1y: Spot<Option<SatsFract>> = "*_1y",
    } }
    shape! { Cycle at "series().rarity_meter.cycle" {
        pct0_1: Spot<Option<SatsFract>> = "*_pct0_1",
        pct0_5: Spot<Option<SatsFract>> = "*_pct0_5",
        pct1: Spot<Option<SatsFract>> = "*_pct01",
        pct2: Spot<Option<SatsFract>> = "*_pct02",
        pct5: Spot<Option<SatsFract>> = "*_pct05",
        pct10: Spot<Option<SatsFract>> = "*_pct10",
        pct20: Spot<Option<SatsFract>> = "*_pct20",
        pct30: Spot<Option<SatsFract>> = "*_pct30",
        pct40: Spot<Option<SatsFract>> = "*_pct40",
        pct50: Spot<Option<SatsFract>> = "*_pct50",
        pct60: Spot<Option<SatsFract>> = "*_pct60",
        pct70: Spot<Option<SatsFract>> = "*_pct70",
        pct80: Spot<Option<SatsFract>> = "*_pct80",
        pct90: Spot<Option<SatsFract>> = "*_pct90",
        pct95: Spot<Option<SatsFract>> = "*_pct95",
        pct98: Spot<Option<SatsFract>> = "*_pct98",
        pct99: Spot<Option<SatsFract>> = "*_pct99",
        pct99_5: Spot<Option<SatsFract>> = "*_pct99_5",
        pct99_9: Spot<Option<SatsFract>> = "*_pct99_9",
        index: SeriesPattern2<StoredI8> = "*_index",
        score: SeriesPattern2<StoredI8> = "*_score",
    } }
    shape! { Pct999 at "series().rarity_meter.components.active_price.pct99_9" {
        ppm: SeriesPattern2<Option<PartsPerMillion32>> = "*_ratio_pct99_9_ppm",
        ratio: SeriesPattern2<Option<StoredF32>> = "*_ratio_pct99_9",
        price: Spot<Option<SatsFract>> = "*_pct99_9",
    } }
    shape! { Pct995 at "series().rarity_meter.components.active_price.pct99_5" {
        ppm: SeriesPattern2<Option<PartsPerMillion32>> = "*_ratio_pct99_5_ppm",
        ratio: SeriesPattern2<Option<StoredF32>> = "*_ratio_pct99_5",
        price: Spot<Option<SatsFract>> = "*_pct99_5",
    } }
    shape! { Pct99 at "series().rarity_meter.components.active_price.pct99" {
        ppm: SeriesPattern2<Option<PartsPerMillion32>> = "*_ratio_pct99_ppm",
        ratio: SeriesPattern2<Option<StoredF32>> = "*_ratio_pct99",
        price: Spot<Option<SatsFract>> = "*_pct99",
    } }
    shape! { Pct98 at "series().rarity_meter.components.active_price.pct98" {
        ppm: SeriesPattern2<Option<PartsPerMillion32>> = "*_ratio_pct98_ppm",
        ratio: SeriesPattern2<Option<StoredF32>> = "*_ratio_pct98",
        price: Spot<Option<SatsFract>> = "*_pct98",
    } }
    shape! { Pct95 at "series().rarity_meter.components.active_price.pct95" {
        ppm: SeriesPattern2<Option<PartsPerMillion32>> = "*_ratio_pct95_ppm",
        ratio: SeriesPattern2<Option<StoredF32>> = "*_ratio_pct95",
        price: Spot<Option<SatsFract>> = "*_pct95",
    } }
    shape! { Pct90 at "series().rarity_meter.components.active_price.pct90" {
        ppm: SeriesPattern2<Option<PartsPerMillion32>> = "*_ratio_pct90_ppm",
        ratio: SeriesPattern2<Option<StoredF32>> = "*_ratio_pct90",
        price: Spot<Option<SatsFract>> = "*_pct90",
    } }
    shape! { Pct80 at "series().rarity_meter.components.active_price.pct80" {
        ppm: SeriesPattern2<Option<PartsPerMillion32>> = "*_ratio_pct80_ppm",
        ratio: SeriesPattern2<Option<StoredF32>> = "*_ratio_pct80",
        price: Spot<Option<SatsFract>> = "*_pct80",
    } }
    shape! { Pct70 at "series().rarity_meter.components.active_price.pct70" {
        ppm: SeriesPattern2<Option<PartsPerMillion32>> = "*_ratio_pct70_ppm",
        ratio: SeriesPattern2<Option<StoredF32>> = "*_ratio_pct70",
        price: Spot<Option<SatsFract>> = "*_pct70",
    } }
    shape! { Pct60 at "series().rarity_meter.components.active_price.pct60" {
        ppm: SeriesPattern2<Option<PartsPerMillion32>> = "*_ratio_pct60_ppm",
        ratio: SeriesPattern2<Option<StoredF32>> = "*_ratio_pct60",
        price: Spot<Option<SatsFract>> = "*_pct60",
    } }
    shape! { Pct50 at "series().rarity_meter.components.active_price.pct50" {
        ppm: SeriesPattern2<Option<PartsPerMillion32>> = "*_ratio_pct50_ppm",
        ratio: SeriesPattern2<Option<StoredF32>> = "*_ratio_pct50",
        price: Spot<Option<SatsFract>> = "*_pct50",
    } }
    shape! { Pct40 at "series().rarity_meter.components.active_price.pct40" {
        ppm: SeriesPattern2<Option<PartsPerMillion32>> = "*_ratio_pct40_ppm",
        ratio: SeriesPattern2<Option<StoredF32>> = "*_ratio_pct40",
        price: Spot<Option<SatsFract>> = "*_pct40",
    } }
    shape! { Pct30 at "series().rarity_meter.components.active_price.pct30" {
        ppm: SeriesPattern2<Option<PartsPerMillion32>> = "*_ratio_pct30_ppm",
        ratio: SeriesPattern2<Option<StoredF32>> = "*_ratio_pct30",
        price: Spot<Option<SatsFract>> = "*_pct30",
    } }
    shape! { Pct20 at "series().rarity_meter.components.active_price.pct20" {
        ppm: SeriesPattern2<Option<PartsPerMillion32>> = "*_ratio_pct20_ppm",
        ratio: SeriesPattern2<Option<StoredF32>> = "*_ratio_pct20",
        price: Spot<Option<SatsFract>> = "*_pct20",
    } }
    shape! { Pct10 at "series().rarity_meter.components.active_price.pct10" {
        ppm: SeriesPattern2<Option<PartsPerMillion32>> = "*_ratio_pct10_ppm",
        ratio: SeriesPattern2<Option<StoredF32>> = "*_ratio_pct10",
        price: Spot<Option<SatsFract>> = "*_pct10",
    } }
    shape! { Pct5 at "series().rarity_meter.components.active_price.pct5" {
        ppm: SeriesPattern2<Option<PartsPerMillion32>> = "*_ratio_pct5_ppm",
        ratio: SeriesPattern2<Option<StoredF32>> = "*_ratio_pct5",
        price: Spot<Option<SatsFract>> = "*_pct5",
    } }
    shape! { Pct2 at "series().rarity_meter.components.active_price.pct2" {
        ppm: SeriesPattern2<Option<PartsPerMillion32>> = "*_ratio_pct2_ppm",
        ratio: SeriesPattern2<Option<StoredF32>> = "*_ratio_pct2",
        price: Spot<Option<SatsFract>> = "*_pct2",
    } }
    shape! { Pct1 at "series().rarity_meter.components.active_price.pct1" {
        ppm: SeriesPattern2<Option<PartsPerMillion32>> = "*_ratio_pct1_ppm",
        ratio: SeriesPattern2<Option<StoredF32>> = "*_ratio_pct1",
        price: Spot<Option<SatsFract>> = "*_pct1",
    } }
    shape! { Pct05 at "series().rarity_meter.components.active_price.pct0_5" {
        ppm: SeriesPattern2<Option<PartsPerMillion32>> = "*_ratio_pct0_5_ppm",
        ratio: SeriesPattern2<Option<StoredF32>> = "*_ratio_pct0_5",
        price: Spot<Option<SatsFract>> = "*_pct0_5",
    } }
    shape! { Pct01 at "series().rarity_meter.components.active_price.pct0_1" {
        ppm: SeriesPattern2<Option<PartsPerMillion32>> = "*_ratio_pct0_1_ppm",
        ratio: SeriesPattern2<Option<StoredF32>> = "*_ratio_pct0_1",
        price: Spot<Option<SatsFract>> = "*_pct0_1",
    } }
    shape! { CoinflowMedianPriceBtcWeighted at "series().rarity_meter.components.coinflow_median_price_btc_weighted" {
        usd: SeriesPattern2<Option<Dollars>> = "*",
        cents: SeriesPattern2<Option<Cents>> = "*_cents",
        sats: SeriesPattern2<Option<SatsFract>> = "*_sats",
        ppm: SeriesPattern2<Option<PriceRatio>> = "*_ratio_ppm",
        ratio: SeriesPattern2<Option<StoredF32>> = "*_ratio",
        pct0_1: Pct01 = "*",
        pct0_5: Pct05 = "*",
        pct1: Pct1 = "*",
        pct2: Pct2 = "*",
        pct5: Pct5 = "*",
        pct10: Pct10 = "*",
        pct20: Pct20 = "*",
        pct30: Pct30 = "*",
        pct40: Pct40 = "*",
        pct50: Pct50 = "*",
        pct60: Pct60 = "*",
        pct70: Pct70 = "*",
        pct80: Pct80 = "*",
        pct90: Pct90 = "*",
        pct95: Pct95 = "*",
        pct98: Pct98 = "*",
        pct99: Pct99 = "*",
        pct99_5: Pct995 = "*",
        pct99_9: Pct999 = "*",
    } }
    shape! { ActivePrice at "series().rarity_meter.components.active_price" {
        pct0_1: Pct01 = "*",
        pct0_5: Pct05 = "*",
        pct1: Pct1 = "*",
        pct2: Pct2 = "*",
        pct5: Pct5 = "*",
        pct10: Pct10 = "*",
        pct20: Pct20 = "*",
        pct30: Pct30 = "*",
        pct40: Pct40 = "*",
        pct50: Pct50 = "*",
        pct60: Pct60 = "*",
        pct70: Pct70 = "*",
        pct80: Pct80 = "*",
        pct90: Pct90 = "*",
        pct95: Pct95 = "*",
        pct98: Pct98 = "*",
        pct99: Pct99 = "*",
        pct99_5: Pct995 = "*",
        pct99_9: Pct999 = "*",
    } }
    shape! { Components at "series().rarity_meter.components" {
        realized_price: ActivePrice = "realized_*",
        capitalized_price: ActivePrice = "capitalized_*",
        median_price_btc_weighted: CoinflowMedianPriceBtcWeighted = "median_*_btc_weighted",
        median_price_usd_weighted: CoinflowMedianPriceBtcWeighted = "median_*_usd_weighted",
        sth_median_price_btc_weighted: CoinflowMedianPriceBtcWeighted = "sth_median_*_btc_weighted",
        sth_median_price_usd_weighted: CoinflowMedianPriceBtcWeighted = "sth_median_*_usd_weighted",
        lth_median_price_btc_weighted: CoinflowMedianPriceBtcWeighted = "lth_median_*_btc_weighted",
        lth_median_price_usd_weighted: CoinflowMedianPriceBtcWeighted = "lth_median_*_usd_weighted",
        cointime_median_price_btc_weighted: CoinflowMedianPriceBtcWeighted = "cointime_median_*_btc_weighted",
        cointime_median_price_usd_weighted: CoinflowMedianPriceBtcWeighted = "cointime_median_*_usd_weighted",
        coinflow_median_price_btc_weighted: CoinflowMedianPriceBtcWeighted = "coinflow_median_*_btc_weighted",
        coinflow_median_price_usd_weighted: CoinflowMedianPriceBtcWeighted = "coinflow_median_*_usd_weighted",
        sth_cointime_median_price_btc_weighted: CoinflowMedianPriceBtcWeighted = "sth_cointime_median_*_btc_weighted",
        sth_cointime_median_price_usd_weighted: CoinflowMedianPriceBtcWeighted = "sth_cointime_median_*_usd_weighted",
        lth_cointime_median_price_btc_weighted: CoinflowMedianPriceBtcWeighted = "lth_cointime_median_*_btc_weighted",
        lth_cointime_median_price_usd_weighted: CoinflowMedianPriceBtcWeighted = "lth_cointime_median_*_usd_weighted",
        sth_coinflow_median_price_btc_weighted: CoinflowMedianPriceBtcWeighted = "sth_coinflow_median_*_btc_weighted",
        sth_coinflow_median_price_usd_weighted: CoinflowMedianPriceBtcWeighted = "sth_coinflow_median_*_usd_weighted",
        lth_coinflow_median_price_btc_weighted: CoinflowMedianPriceBtcWeighted = "lth_coinflow_median_*_btc_weighted",
        lth_coinflow_median_price_usd_weighted: CoinflowMedianPriceBtcWeighted = "lth_coinflow_median_*_usd_weighted",
        sth_realized_price: ActivePrice = "sth_realized_*",
        sth_capitalized_price: ActivePrice = "sth_capitalized_*",
        lth_realized_price: ActivePrice = "lth_realized_*",
        lth_capitalized_price: ActivePrice = "lth_capitalized_*",
        over_6m_realized_price: ActivePrice = "over_6m_realized_*",
        over_4m_realized_price: ActivePrice = "over_4m_realized_*",
        under_4m_realized_price: ActivePrice = "under_4m_realized_*",
        under_6m_realized_price: ActivePrice = "under_6m_realized_*",
        under_4m_capitalized_price: ActivePrice = "under_4m_capitalized_*",
        under_6m_capitalized_price: ActivePrice = "under_6m_capitalized_*",
        vaulted_price: ActivePrice = "vaulted_*",
        active_price: ActivePrice = "active_*",
        true_market_mean_price: ActivePrice = "true_market_mean_*",
        cointime_price: ActivePrice = "cointime_*",
        awake_price: ActivePrice = "awake_*",
        coinflow_price: ActivePrice = "coinflow_*",
    } }
    shape! { Level at "series().bedrock.coinflow.level" {
        pct10: Spot<Option<SatsFract>> = "*_pct10",
        pct20: Spot<Option<SatsFract>> = "*_pct20",
        pct30: Spot<Option<SatsFract>> = "*_pct30",
        pct40: Spot<Option<SatsFract>> = "*_pct40",
        pct50: Spot<Option<SatsFract>> = "*_pct50",
        pct60: Spot<Option<SatsFract>> = "*_pct60",
        pct70: Spot<Option<SatsFract>> = "*_pct70",
        pct80: Spot<Option<SatsFract>> = "*_pct80",
        pct90: Spot<Option<SatsFract>> = "*_pct90",
    } }
    shape! { Floor at "series().bedrock.coinflow.floor" {
        pct95: Spot<Option<SatsFract>> = "*_pct95",
        pct98: Spot<Option<SatsFract>> = "*_pct98",
        pct99: Spot<Option<SatsFract>> = "*_pct99",
        pct99_5: Spot<Option<SatsFract>> = "*_pct99_5",
        pct99_9: Spot<Option<SatsFract>> = "*_pct99_9",
    } }
    shape! { BedrockCoinflow at "series().bedrock.coinflow" {
        supply_in_loss_threshold: SupplyInLossThreshold = "*_supply_in_loss_threshold",
        floor: Floor = "*_floor",
        level: Level = "*_level",
    } }
    shape! { Bedrock at "series().bedrock" {
        raw: BedrockCoinflow = "*_raw",
        cointime: BedrockCoinflow = "*_cointime",
        coinflow: BedrockCoinflow = "*_coinflow",
    } }
    shape! { PerCoin at "series().coinflow.urpd.all.cost_basis.per_coin" {
        pct05: Spot<Option<SatsFract>> = "*_pct05",
        pct10: Spot<Option<SatsFract>> = "*_pct10",
        pct15: Spot<Option<SatsFract>> = "*_pct15",
        pct20: Spot<Option<SatsFract>> = "*_pct20",
        pct25: Spot<Option<SatsFract>> = "*_pct25",
        pct30: Spot<Option<SatsFract>> = "*_pct30",
        pct35: Spot<Option<SatsFract>> = "*_pct35",
        pct40: Spot<Option<SatsFract>> = "*_pct40",
        pct45: Spot<Option<SatsFract>> = "*_pct45",
        pct50: Spot<Option<SatsFract>> = "*_pct50",
        pct55: Spot<Option<SatsFract>> = "*_pct55",
        pct60: Spot<Option<SatsFract>> = "*_pct60",
        pct65: Spot<Option<SatsFract>> = "*_pct65",
        pct70: Spot<Option<SatsFract>> = "*_pct70",
        pct75: Spot<Option<SatsFract>> = "*_pct75",
        pct80: Spot<Option<SatsFract>> = "*_pct80",
        pct85: Spot<Option<SatsFract>> = "*_pct85",
        pct90: Spot<Option<SatsFract>> = "*_pct90",
        pct95: Spot<Option<SatsFract>> = "*_pct95",
    } }
    shape! { UrpdAllCostBasis<A> at "series().coinflow.urpd.all.cost_basis" {
        per_coin: A = "*_coin",
        per_dollar: A = "*_dollar",
    } }
    shape! { SpendingRate at "series().coinflow.age_range.spending_rate" {
        under_1h: SeriesPattern2<Option<StoredF64>> = "utxos_under_1h_*",
        _1h_to_1d: SeriesPattern2<Option<StoredF64>> = "utxos_1h_to_1d_*",
        _1d_to_1w: SeriesPattern2<Option<StoredF64>> = "utxos_1d_to_1w_*",
        _1w_to_1m: SeriesPattern2<Option<StoredF64>> = "utxos_1w_to_1m_*",
        _1m_to_2m: SeriesPattern2<Option<StoredF64>> = "utxos_1m_to_2m_*",
        _2m_to_3m: SeriesPattern2<Option<StoredF64>> = "utxos_2m_to_3m_*",
        _3m_to_4m: SeriesPattern2<Option<StoredF64>> = "utxos_3m_to_4m_*",
        _4m_to_5m: SeriesPattern2<Option<StoredF64>> = "utxos_4m_to_5m_*",
        _5m_to_6m: SeriesPattern2<Option<StoredF64>> = "utxos_5m_to_6m_*",
        _6m_to_9m: SeriesPattern2<Option<StoredF64>> = "utxos_6m_to_9m_*",
        _9m_to_1y: SeriesPattern2<Option<StoredF64>> = "utxos_9m_to_1y_*",
        _1y_to_18m: SeriesPattern2<Option<StoredF64>> = "utxos_1y_to_18m_*",
        _18m_to_2y: SeriesPattern2<Option<StoredF64>> = "utxos_18m_to_2y_*",
        _2y_to_3y: SeriesPattern2<Option<StoredF64>> = "utxos_2y_to_3y_*",
        _3y_to_4y: SeriesPattern2<Option<StoredF64>> = "utxos_3y_to_4y_*",
        _4y_to_5y: SeriesPattern2<Option<StoredF64>> = "utxos_4y_to_5y_*",
        _5y_to_6y: SeriesPattern2<Option<StoredF64>> = "utxos_5y_to_6y_*",
        _6y_to_7y: SeriesPattern2<Option<StoredF64>> = "utxos_6y_to_7y_*",
        _7y_to_8y: SeriesPattern2<Option<StoredF64>> = "utxos_7y_to_8y_*",
        _8y_to_10y: SeriesPattern2<Option<StoredF64>> = "utxos_8y_to_10y_*",
        _10y_to_12y: SeriesPattern2<Option<StoredF64>> = "utxos_10y_to_12y_*",
        _12y_to_15y: SeriesPattern2<Option<StoredF64>> = "utxos_12y_to_15y_*",
        over_15y: SeriesPattern2<Option<StoredF64>> = "utxos_over_15y_*",
    } }
    shape! { SpendingExposure at "series().coinflow.age_range.spending_exposure" {
        under_1h: SeriesPattern2<Option<StoredF64>> = "utxos_under_1h_*_spending_exposure",
        _1h_to_1d: SeriesPattern2<Option<StoredF64>> = "utxos_1h_to_1d_*_spending_exposure",
        _1d_to_1w: SeriesPattern2<Option<StoredF64>> = "utxos_1d_to_1w_*_spending_exposure",
        _1w_to_1m: SeriesPattern2<Option<StoredF64>> = "utxos_1w_to_1m_*_spending_exposure",
        _1m_to_2m: SeriesPattern2<Option<StoredF64>> = "utxos_1m_to_2m_*_spending_exposure",
        _2m_to_3m: SeriesPattern2<Option<StoredF64>> = "utxos_2m_to_3m_*_spending_exposure",
        _3m_to_4m: SeriesPattern2<Option<StoredF64>> = "utxos_3m_to_4m_*_spending_exposure",
        _4m_to_5m: SeriesPattern2<Option<StoredF64>> = "utxos_4m_to_5m_*_spending_exposure",
        _5m_to_6m: SeriesPattern2<Option<StoredF64>> = "utxos_5m_to_6m_*_spending_exposure",
        _6m_to_9m: SeriesPattern2<Option<StoredF64>> = "utxos_6m_to_9m_*_spending_exposure",
        _9m_to_1y: SeriesPattern2<Option<StoredF64>> = "utxos_9m_to_1y_*_spending_exposure",
        _1y_to_18m: SeriesPattern2<Option<StoredF64>> = "utxos_1y_to_18m_*_spending_exposure",
        _18m_to_2y: SeriesPattern2<Option<StoredF64>> = "utxos_18m_to_2y_*_spending_exposure",
        _2y_to_3y: SeriesPattern2<Option<StoredF64>> = "utxos_2y_to_3y_*_spending_exposure",
        _3y_to_4y: SeriesPattern2<Option<StoredF64>> = "utxos_3y_to_4y_*_spending_exposure",
        _4y_to_5y: SeriesPattern2<Option<StoredF64>> = "utxos_4y_to_5y_*_spending_exposure",
        _5y_to_6y: SeriesPattern2<Option<StoredF64>> = "utxos_5y_to_6y_*_spending_exposure",
        _6y_to_7y: SeriesPattern2<Option<StoredF64>> = "utxos_6y_to_7y_*_spending_exposure",
        _7y_to_8y: SeriesPattern2<Option<StoredF64>> = "utxos_7y_to_8y_*_spending_exposure",
        _8y_to_10y: SeriesPattern2<Option<StoredF64>> = "utxos_8y_to_10y_*_spending_exposure",
        _10y_to_12y: SeriesPattern2<Option<StoredF64>> = "utxos_10y_to_12y_*_spending_exposure",
        _12y_to_15y: SeriesPattern2<Option<StoredF64>> = "utxos_12y_to_15y_*_spending_exposure",
        over_15y: SeriesPattern2<Option<StoredF64>> = "utxos_over_15y_*_spending_exposure",
        mobility: SpendingRate = "*_mobility",
    } }
    shape! { AgeRangeActivity at "series().cointime.age_range.activity" {
        wakefulness: SpendingRate = "*_wakefulness",
        dormancy: SpendingRate = "*_dormancy",
        wakefulness_to_dormancy: SpendingRate = "*_wakefulness_to_dormancy",
    } }
    shape! { RateSma at "series().mining.hashrate.rate.sma" {
        _1w: SeriesPattern2<Option<StoredF64>> = "*_1w",
        _1m: SeriesPattern2<Option<StoredF64>> = "*_1m",
        _2m: SeriesPattern2<Option<StoredF64>> = "*_2m",
        _1y: SeriesPattern2<Option<StoredF64>> = "*_1y",
    } }
    shape! { OpReturnRaw at "series().op_return.raw" {
        first_index: SeriesPattern21<OpReturnIndex> = "first_op_return_*",
        to_tx_index: SeriesPattern26<TxIndex> = "tx_*",
        kind: SeriesPattern26<OpReturnKind> = "kind",
        post_op_return_bytes: SeriesPattern26<StoredU32> = "op_return_post_op_return_bytes",
    } }
    shape! { RawUnknown at "series().scripts.raw.unknown" {
        first_index: SeriesPattern21<UnknownOutputIndex> = "first_unknown_output_*",
        to_tx_index: SeriesPattern36<TxIndex> = "tx_*",
        legacy_sigops: SeriesPattern36<SigOps> = "unknown_legacy_sigops",
    } }
    shape! { RawP2ms at "series().scripts.raw.p2ms" {
        first_index: SeriesPattern21<P2MSOutputIndex> = "first_p2ms_output_*",
        to_tx_index: SeriesPattern28<TxIndex> = "tx_*",
        legacy_sigops: SeriesPattern28<SigOps> = "p2ms_legacy_sigops",
    } }
    shape! { RawEmpty at "series().scripts.raw.empty" {
        first_index: SeriesPattern21<EmptyOutputIndex> = "first_empty_output_*",
        to_tx_index: SeriesPattern25<TxIndex> = "tx_*",
    } }
    shape! { ScriptsRaw at "series().scripts.raw" {
        empty: RawEmpty = "*",
        p2ms: RawP2ms = "*",
        unknown: RawUnknown = "*",
    } }
    shape! { Scripts at "series().scripts" {
        raw: ScriptsRaw = "*",
    } }
    shape! { AddrsEmpty at "series().addrs.empty" {
        all: SeriesPattern2<StoredU64> = "*",
        p2pk65: SeriesPattern2<StoredU64> = "p2pk65_*",
        p2pk33: SeriesPattern2<StoredU64> = "p2pk33_*",
        p2pkh: SeriesPattern2<StoredU64> = "p2pkh_*",
        p2sh: SeriesPattern2<StoredU64> = "p2sh_*",
        p2wpkh: SeriesPattern2<StoredU64> = "p2wpkh_*",
        p2wsh: SeriesPattern2<StoredU64> = "p2wsh_*",
        p2tr: SeriesPattern2<StoredU64> = "p2tr_*",
        p2a: SeriesPattern2<StoredU64> = "p2a_*",
    } }
    shape! { ExposedCount at "series().addrs.exposed.count" {
        funded: AddrsEmpty = "*",
        total: AddrsEmpty = "total_*",
    } }
    shape! { RealizedLoss0satsBlock<A> at "series().addrs.by_balance.realized_loss._0sats.block" {
        usd: SeriesPattern21<Option<Dollars>> = "*",
        cents: SeriesPattern21<A> = "*_cents",
    } }
    shape! { CoinflowCap<A> at "series().coinflow.cap" {
        usd: SeriesPattern2<Option<Dollars>> = "*",
        cents: SeriesPattern2<A> = "*_cents",
    } }
    shape! { AllUnrealized at "series().distribution_aggregated.cohorts.all.unrealized" {
        profit: CoinflowCap<Option<Cents>> = "*_unrealized_profit",
        loss: CoinflowCap<Option<Cents>> = "*_unrealized_loss",
        net_pnl: CoinflowCap<CentsSigned> = "*_net_unrealized_pnl",
        gross_pnl: CoinflowCap<Option<Cents>> = "*_unrealized_gross_pnl",
        invested_capital_in_profit: CoinflowCap<Option<Cents>> = "*_invested_capital_in_profit",
        invested_capital_in_loss: CoinflowCap<Option<Cents>> = "*_invested_capital_in_loss",
        pain_index: CoinflowCap<Option<Cents>> = "*_pain_index",
        greed_index: CoinflowCap<Option<Cents>> = "*_greed_index",
        net_sentiment: CoinflowCap<CentsSigned> = "*_net_sentiment",
        nupl: RhodlRatio<Option<PartsPerMillionSigned32>> = "*_nupl",
    } }
    shape! { CointimeCap at "series().cointime.cap" {
        thermo: CoinflowCap<Option<Cents>> = "thermo_*",
        investor: CoinflowCap<Option<Cents>> = "investor_*",
        vaulted: CoinflowCap<Option<Cents>> = "vaulted_*",
        active: CoinflowCap<Option<Cents>> = "active_*",
        cointime: CoinflowCap<Option<Cents>> = "cointime_*",
        aviv: RhodlRatio<Option<PartsPerMillion32>> = "aviv_ratio",
    } }
    shape! { Awake at "series().cointime.awake" {
        supply: AwakeSupply = "*_supply",
        cap: CoinflowCap<Option<Cents>> = "*_cap",
        price: CapitalizedPrice = "*_price",
        capitalized_price: CapitalizedPrice = "*_capitalized_price",
    } }
    shape! { Absolute1m at "series().addrs.by_balance.supply._0sats.delta.absolute._1m" {
        btc: SeriesPattern2<Option<Bitcoin>> = "*",
        sats: SeriesPattern2<SatsSigned> = "*_sats",
    } }
    shape! { State at "series().addrs.state" {
        p2a: SeriesPattern27<AddrState> = "*_state",
        p2pk33: SeriesPattern29<AddrState> = "*_state",
        p2pk65: SeriesPattern30<AddrState> = "*_state",
        p2pkh: SeriesPattern31<AddrState> = "*_state",
        p2sh: SeriesPattern32<AddrState> = "*_state",
        p2tr: SeriesPattern33<AddrState> = "*_state",
        p2wpkh: SeriesPattern34<AddrState> = "*_state",
        p2wsh: SeriesPattern35<AddrState> = "*_state",
        funded: SeriesPattern37<FundedAddrData> = "funded_*_data",
        extended_empty: SeriesPattern38<EmptyAddrData> = "extended_empty_*_data",
    } }
    shape! { RawP2a at "series().addrs.raw.p2a" {
        first_index: SeriesPattern21<P2AAddrIndex> = "first_*_addr_index",
        bytes: SeriesPattern27<P2ABytes> = "*_bytes",
    } }
    shape! { RawP2tr at "series().addrs.raw.p2tr" {
        first_index: SeriesPattern21<P2TRAddrIndex> = "first_*_addr_index",
        bytes: SeriesPattern33<P2TRBytes> = "*_bytes",
    } }
    shape! { RawP2wsh at "series().addrs.raw.p2wsh" {
        first_index: SeriesPattern21<P2WSHAddrIndex> = "first_*_addr_index",
        bytes: SeriesPattern35<P2WSHBytes> = "*_bytes",
    } }
    shape! { RawP2wpkh at "series().addrs.raw.p2wpkh" {
        first_index: SeriesPattern21<P2WPKHAddrIndex> = "first_*_addr_index",
        bytes: SeriesPattern34<P2WPKHBytes> = "*_bytes",
    } }
    shape! { RawP2sh at "series().addrs.raw.p2sh" {
        first_index: SeriesPattern21<P2SHAddrIndex> = "first_*_addr_index",
        bytes: SeriesPattern32<P2SHBytes> = "*_bytes",
    } }
    shape! { RawP2pkh at "series().addrs.raw.p2pkh" {
        first_index: SeriesPattern21<P2PKHAddrIndex> = "first_*_addr_index",
        bytes: SeriesPattern31<P2PKHBytes> = "*_bytes",
    } }
    shape! { RawP2pk33 at "series().addrs.raw.p2pk33" {
        first_index: SeriesPattern21<P2PK33AddrIndex> = "first_*_addr_index",
        bytes: SeriesPattern29<P2PK33Bytes> = "*_bytes",
    } }
    shape! { RawP2pk65 at "series().addrs.raw.p2pk65" {
        first_index: SeriesPattern21<P2PK65AddrIndex> = "first_*_addr_index",
        bytes: SeriesPattern30<P2PK65Bytes> = "*_bytes",
    } }
    shape! { AddrsRaw at "series().addrs.raw" {
        p2pk65: RawP2pk65 = "*",
        p2pk33: RawP2pk33 = "p2pk33",
        p2pkh: RawP2pkh = "p2pkh",
        p2sh: RawP2sh = "p2sh",
        p2wpkh: RawP2wpkh = "p2wpkh",
        p2wsh: RawP2wsh = "p2wsh",
        p2tr: RawP2tr = "p2tr",
        p2a: RawP2a = "p2a",
    } }
    shape! { Spent at "series().outputs.spent" {
        txin_index: SeriesPattern24<TxInIndex> = "*",
    } }
    shape! { OutputsRaw at "series().outputs.raw" {
        first_txout_index: SeriesPattern21<TxOutIndex> = "first_txout_index",
        value: SeriesPattern24<Sats> = "value",
        output_type: SeriesPattern24<OutputType> = "output_*",
        type_index: SeriesPattern24<TypeIndex> = "*_index",
    } }
    shape! { InputsRaw at "series().inputs.raw" {
        first_txin_index: SeriesPattern21<TxInIndex> = "first_txin_*",
        outpoint: SeriesPattern23<OutPoint> = "outpoint",
        txout_index: SeriesPattern23<TxOutIndex> = "txout_*",
        tx_index: SeriesPattern23<TxIndex> = "tx_*",
        output_type: SeriesPattern23<OutputType> = "output_type",
        type_index: SeriesPattern23<TypeIndex> = "type_*",
    } }
    shape! { Circulating<A, B> at "series().supply.circulating" {
        btc: SeriesPattern2<Option<Bitcoin>> = "*",
        sats: SeriesPattern2<A> = "*_sats",
        usd: SeriesPattern2<Option<Dollars>> = "*_usd",
        cents: SeriesPattern2<B> = "*_cents",
    } }
    shape! { CoinflowSupply at "series().coinflow.supply" {
        mobile: Mobile = "*",
        immobile: Circulating<Sats, Option<Cents>> = "*_immobile_supply",
    } }
    shape! { CoinflowLth at "series().coinflow.lth" {
        supply: CoinflowSupply = "*",
        cap: CoinflowCap<Option<Cents>> = "*_coinflow_cap",
        price: CapitalizedPrice = "*_coinflow_price",
        capitalized_price: CapitalizedPrice = "*_coinflow_capitalized_price",
    } }
    shape! { CointimeSupply at "series().cointime.supply" {
        vaulted: Circulating<Sats, Option<Cents>> = "vaulted_*",
        active: Active = "*",
    } }
    shape! { Dormant at "series().cointime.dormant" {
        supply: Circulating<Sats, Option<Cents>> = "*",
    } }
    shape! { CointimeLth at "series().cointime.lth" {
        awake: Awake = "*_awake",
        dormant: Dormant = "*_dormant_supply",
    } }
    shape! { BurnedBlock at "series().supply.burned.block" {
        btc: SeriesPattern21<Option<Bitcoin>> = "*",
        sats: SeriesPattern21<Sats> = "*_sats",
        usd: SeriesPattern21<Option<Dollars>> = "*_usd",
        cents: SeriesPattern21<Option<Cents>> = "*_cents",
    } }
    shape! { Burned at "series().supply.burned" {
        block: BurnedBlock = "*",
        cumulative: Circulating<Sats, Option<Cents>> = "*_cumulative",
    } }
    shape! { OutputsValue at "series().outputs.value" {
        op_return: Burned = "*",
    } }
    shape! { EffectiveFeeRate6b<A> at "series().transactions.fees.effective_fee_rate._6b" {
        min: SeriesPattern2<A> = "*_min",
        max: SeriesPattern2<A> = "*_max",
        pct10: SeriesPattern2<A> = "*_pct10",
        pct25: SeriesPattern2<A> = "*_pct25",
        median: SeriesPattern2<A> = "*_median",
        pct75: SeriesPattern2<A> = "*_pct75",
        pct90: SeriesPattern2<A> = "*_pct90",
    } }
    shape! { SizeWeight at "series().transactions.size.weight" {
        block: EffectiveFeeRate6b<Weight> = "*",
        _6b: EffectiveFeeRate6b<Weight> = "*_6b",
    } }
    shape! { Vsize6b at "series().transactions.size.vsize._6b" {
        min: SeriesPattern21<VSize> = "*_min",
        max: SeriesPattern21<VSize> = "*_max",
        pct10: SeriesPattern21<VSize> = "*_pct10",
        pct25: SeriesPattern21<VSize> = "*_pct25",
        median: SeriesPattern21<VSize> = "*_median",
        pct75: SeriesPattern21<VSize> = "*_pct75",
        pct90: SeriesPattern21<VSize> = "*_pct90",
    } }
    shape! { EffectiveFeeRate<A, B> at "series().transactions.fees.effective_fee_rate" {
        tx_index: SeriesPattern22<A> = "*",
        block: B = "*",
        _6b: B = "*_6b",
    } }
    shape! { TransactionsSize at "series().transactions.size" {
        vsize: EffectiveFeeRate<VSize, Vsize6b> = "*_vsize",
        weight: SizeWeight = "*_weight",
    } }
    shape! { TransactionsRaw at "series().transactions.raw" {
        first_tx_index: SeriesPattern21<TxIndex> = "first_*_index",
        txid: SeriesPattern22<Txid> = "txid",
        tx_version: SeriesPattern22<TxVersion> = "*_version",
        raw_locktime: SeriesPattern22<RawLockTime> = "raw_locktime",
        weight: SeriesPattern22<Weight> = "*_weight",
        total_size: SeriesPattern22<StoredU32> = "total_size",
        total_sigop_cost: SeriesPattern22<SigOps> = "total_sigop_cost",
        is_explicitly_rbf: SeriesPattern22<StoredBool> = "is_explicitly_rbf",
        first_txin_index: SeriesPattern22<TxInIndex> = "first_txin_index",
        first_txout_index: SeriesPattern22<TxOutIndex> = "first_txout_index",
    } }
    shape! { BlocksHalving at "series().blocks.halving" {
        epoch: SeriesPattern2<Halving> = "*_epoch",
        blocks_to_halving: SeriesPattern2<StoredU32> = "blocks_to_*",
        days_to_halving: SeriesPattern2<Option<StoredF32>> = "days_to_*",
    } }
    shape! { Fullness at "series().blocks.fullness" {
        ppm: SeriesPattern21<Option<PartsPerMillion32>> = "*_ppm",
        ratio: SeriesPattern21<Option<StoredF32>> = "*_ratio",
        percent: SeriesPattern21<Option<StoredF32>> = "*",
    } }
    shape! { Interval<A> at "series().blocks.interval" {
        block: SeriesPattern21<A> = "*",
        _24h: SeriesPattern2<Option<StoredF32>> = "*_average_24h",
        _1w: SeriesPattern2<Option<StoredF32>> = "*_average_1w",
        _1m: SeriesPattern2<Option<StoredF32>> = "*_average_1m",
        _1y: SeriesPattern2<Option<StoredF32>> = "*_average_1y",
    } }
    shape! { BlocksLookback at "series().blocks.lookback" {
        _1h: SeriesPattern21<Height> = "*_1h_ago",
        _24h: SeriesPattern21<Height> = "*_24h_ago",
        _3d: SeriesPattern21<Height> = "*_3d_ago",
        _1w: SeriesPattern21<Height> = "*_1w_ago",
        _8d: SeriesPattern21<Height> = "*_8d_ago",
        _9d: SeriesPattern21<Height> = "*_9d_ago",
        _12d: SeriesPattern21<Height> = "*_12d_ago",
        _13d: SeriesPattern21<Height> = "*_13d_ago",
        _2w: SeriesPattern21<Height> = "*_2w_ago",
        _21d: SeriesPattern21<Height> = "*_21d_ago",
        _26d: SeriesPattern21<Height> = "*_26d_ago",
        _1m: SeriesPattern21<Height> = "*_1m_ago",
        _34d: SeriesPattern21<Height> = "*_34d_ago",
        _50d: SeriesPattern21<Height> = "*_50d_ago",
        _55d: SeriesPattern21<Height> = "*_55d_ago",
        _2m: SeriesPattern21<Height> = "*_2m_ago",
        _9w: SeriesPattern21<Height> = "*_9w_ago",
        _12w: SeriesPattern21<Height> = "*_12w_ago",
        _89d: SeriesPattern21<Height> = "*_89d_ago",
        _3m: SeriesPattern21<Height> = "*_3m_ago",
        _14w: SeriesPattern21<Height> = "*_14w_ago",
        _111d: SeriesPattern21<Height> = "*_111d_ago",
        _144d: SeriesPattern21<Height> = "*_144d_ago",
        _6m: SeriesPattern21<Height> = "*_6m_ago",
        _26w: SeriesPattern21<Height> = "*_26w_ago",
        _200d: SeriesPattern21<Height> = "*_200d_ago",
        _9m: SeriesPattern21<Height> = "*_9m_ago",
        _350d: SeriesPattern21<Height> = "*_350d_ago",
        _12m: SeriesPattern21<Height> = "*_12m_ago",
        _1y: SeriesPattern21<Height> = "*_1y_ago",
        _14m: SeriesPattern21<Height> = "*_14m_ago",
        _2y: SeriesPattern21<Height> = "*_2y_ago",
        _26m: SeriesPattern21<Height> = "*_26m_ago",
        _3y: SeriesPattern21<Height> = "*_3y_ago",
        _200w: SeriesPattern21<Height> = "*_200w_ago",
        _4y: SeriesPattern21<Height> = "*_4y_ago",
        _5y: SeriesPattern21<Height> = "*_5y_ago",
        _6y: SeriesPattern21<Height> = "*_6y_ago",
        _8y: SeriesPattern21<Height> = "*_8y_ago",
        _9y: SeriesPattern21<Height> = "*_9y_ago",
        _10y: SeriesPattern21<Height> = "*_10y_ago",
        _12y: SeriesPattern21<Height> = "*_12y_ago",
        _14y: SeriesPattern21<Height> = "*_14y_ago",
        _26y: SeriesPattern21<Height> = "*_26y_ago",
    } }
    shape! { Target at "series().blocks.count.target" {
        _24h: SeriesPattern1<StoredU64> = "*_24h",
        _1w: SeriesPattern1<StoredU64> = "*_1w",
        _1m: SeriesPattern1<StoredU64> = "*_1m",
        _1y: SeriesPattern1<StoredU64> = "*_1y",
    } }
    shape! { PerSec<A> at "series().inputs.per_sec" {
        _24h: SeriesPattern2<A> = "*_24h",
        _1w: SeriesPattern2<A> = "*_1w",
        _1m: SeriesPattern2<A> = "*_1m",
        _1y: SeriesPattern2<A> = "*_1y",
    } }
    shape! { BlocksMined at "series().pools.major.antpool.blocks_mined" {
        block: SeriesPattern21<StoredU64> = "*",
        cumulative: SeriesPattern2<StoredU64> = "*_cumulative",
        sum: PerSec<StoredU64> = "*_sum",
    } }
    shape! { Rolling at "series().inputs.count.rolling" {
        sum: PerSec<StoredU64> = "*_sum",
        average: PerSec<Option<StoredF32>> = "*_average",
        min: PerSec<StoredU64> = "*_min",
        max: PerSec<StoredU64> = "*_max",
        pct10: PerSec<StoredU64> = "*_pct10",
        pct25: PerSec<StoredU64> = "*_pct25",
        median: PerSec<StoredU64> = "*_median",
        pct75: PerSec<StoredU64> = "*_pct75",
        pct90: PerSec<StoredU64> = "*_pct90",
    } }
    shape! { InputsCount at "series().inputs.count" {
        sum: SeriesPattern21<StoredU64> = "*_sum",
        cumulative: SeriesPattern2<StoredU64> = "*_cumulative",
        rolling: Rolling = "*",
    } }
    shape! { Vbytes at "series().blocks.vbytes" {
        block: SeriesPattern21<StoredU64> = "*",
        cumulative: SeriesPattern2<StoredU64> = "*_cumulative",
        sum: PerSec<StoredU64> = "*_sum",
        average: PerSec<Option<StoredF32>> = "*_average",
        min: PerSec<StoredU64> = "*_min",
        max: PerSec<StoredU64> = "*_max",
        pct10: PerSec<StoredU64> = "*_pct10",
        pct25: PerSec<StoredU64> = "*_pct25",
        median: PerSec<StoredU64> = "*_median",
        pct75: PerSec<StoredU64> = "*_pct75",
        pct90: PerSec<StoredU64> = "*_pct90",
    } }
    shape! { NewAll<A> at "series().addrs.new.all" {
        block: SeriesPattern21<A> = "*",
        cumulative: SeriesPattern2<A> = "*_cumulative",
        sum: PerSec<A> = "*_sum",
        average: PerSec<Option<StoredF32>> = "*_average",
    } }
    shape! { CointimeValue at "series().cointime.value" {
        destroyed: NewAll<Option<StoredF64>> = "*_destroyed",
        created: NewAll<Option<StoredF64>> = "*_created",
        stored: NewAll<Option<StoredF64>> = "*_stored",
        vocdd: NewAll<Option<StoredF64>> = "vocdd",
    } }
    shape! { CointimeActivity at "series().cointime.activity" {
        coinblocks_created: NewAll<Option<StoredF64>> = "*_created",
        coinblocks_stored: NewAll<Option<StoredF64>> = "*_stored",
        liveliness: SeriesPattern2<Option<StoredF64>> = "liveliness",
        vaultedness: SeriesPattern2<Option<StoredF64>> = "vaultedness",
        ratio: SeriesPattern2<Option<StoredF64>> = "activity_to_vaultedness",
        coinblocks_destroyed: NewAll<Option<StoredF64>> = "*_destroyed",
    } }
    shape! { OutputsByTypeTxCount at "series().outputs.by_type.tx_count" {
        all: NewAll<StoredU64> = "*_bis",
        p2pk65: NewAll<StoredU64> = "*_with_p2pk65_output",
        p2pk33: NewAll<StoredU64> = "*_with_p2pk33_output",
        p2pkh: NewAll<StoredU64> = "*_with_p2pkh_output",
        p2ms: NewAll<StoredU64> = "*_with_p2ms_output",
        p2sh: NewAll<StoredU64> = "*_with_p2sh_output",
        p2wpkh: NewAll<StoredU64> = "*_with_p2wpkh_output",
        p2wsh: NewAll<StoredU64> = "*_with_p2wsh_output",
        p2tr: NewAll<StoredU64> = "*_with_p2tr_output",
        p2a: NewAll<StoredU64> = "*_with_p2a_output",
        unknown: NewAll<StoredU64> = "*_with_unknown_outputs_output",
        empty: NewAll<StoredU64> = "*_with_empty_outputs_output",
        op_return: NewAll<StoredU64> = "*_with_op_return_output",
    } }
    shape! { OutputCount at "series().outputs.by_type.output_count" {
        all: NewAll<StoredU64> = "*_bis",
        p2pk65: NewAll<StoredU64> = "p2pk65_*",
        p2pk33: NewAll<StoredU64> = "p2pk33_*",
        p2pkh: NewAll<StoredU64> = "p2pkh_*",
        p2ms: NewAll<StoredU64> = "p2ms_*",
        p2sh: NewAll<StoredU64> = "p2sh_*",
        p2wpkh: NewAll<StoredU64> = "p2wpkh_*",
        p2wsh: NewAll<StoredU64> = "p2wsh_*",
        p2tr: NewAll<StoredU64> = "p2tr_*",
        p2a: NewAll<StoredU64> = "p2a_*",
        unknown: NewAll<StoredU64> = "unknown_outputs_*",
        empty: NewAll<StoredU64> = "empty_outputs_*",
        op_return: NewAll<StoredU64> = "op_return_*",
    } }
    shape! { InputsByTypeTxCount at "series().inputs.by_type.tx_count" {
        all: NewAll<StoredU64> = "non_coinbase_*",
        p2pk65: NewAll<StoredU64> = "*_with_p2pk65_prevout",
        p2pk33: NewAll<StoredU64> = "*_with_p2pk33_prevout",
        p2pkh: NewAll<StoredU64> = "*_with_p2pkh_prevout",
        p2ms: NewAll<StoredU64> = "*_with_p2ms_prevout",
        p2sh: NewAll<StoredU64> = "*_with_p2sh_prevout",
        p2wpkh: NewAll<StoredU64> = "*_with_p2wpkh_prevout",
        p2wsh: NewAll<StoredU64> = "*_with_p2wsh_prevout",
        p2tr: NewAll<StoredU64> = "*_with_p2tr_prevout",
        p2a: NewAll<StoredU64> = "*_with_p2a_prevout",
        unknown: NewAll<StoredU64> = "*_with_unknown_outputs_prevout",
        empty: NewAll<StoredU64> = "*_with_empty_outputs_prevout",
    } }
    shape! { InputCount at "series().inputs.by_type.input_count" {
        all: NewAll<StoredU64> = "input_*_bis",
        p2pk65: NewAll<StoredU64> = "p2pk65_prevout_*",
        p2pk33: NewAll<StoredU64> = "p2pk33_prevout_*",
        p2pkh: NewAll<StoredU64> = "p2pkh_prevout_*",
        p2ms: NewAll<StoredU64> = "p2ms_prevout_*",
        p2sh: NewAll<StoredU64> = "p2sh_prevout_*",
        p2wpkh: NewAll<StoredU64> = "p2wpkh_prevout_*",
        p2wsh: NewAll<StoredU64> = "p2wsh_prevout_*",
        p2tr: NewAll<StoredU64> = "p2tr_prevout_*",
        p2a: NewAll<StoredU64> = "p2a_prevout_*",
        unknown: NewAll<StoredU64> = "unknown_outputs_prevout_*",
        empty: NewAll<StoredU64> = "empty_outputs_prevout_*",
    } }
    shape! { Versions at "series().transactions.versions" {
        v1: NewAll<StoredU64> = "*_v1",
        v2: NewAll<StoredU64> = "*_v2",
        v3: NewAll<StoredU64> = "*_v3",
        other: NewAll<StoredU64> = "*_other_version",
    } }
    shape! { PolicyCount at "series().transactions.policy.count" {
        nonstandard: NewAll<StoredU64> = "*",
    } }
    shape! { Policy at "series().transactions.policy" {
        count: PolicyCount = "*_count",
        is_nonstandard: SeriesPattern22<StoredBool> = "is_*",
    } }
    shape! { PatternsCount at "series().transactions.patterns.count" {
        coinjoin: NewAll<StoredU64> = "coinjoin_*",
        consolidation: NewAll<StoredU64> = "consolidation_*",
        batch_payout: NewAll<StoredU64> = "batch_payout_*",
    } }
    shape! { Patterns at "series().transactions.patterns" {
        count: PatternsCount = "count",
        is_coinjoin: SeriesPattern22<StoredBool> = "*_coinjoin",
        is_consolidation: SeriesPattern22<StoredBool> = "*_consolidation",
        is_batch_payout: SeriesPattern22<StoredBool> = "*_batch_payout",
    } }
    shape! { FeesCount at "series().transactions.fees.count" {
        cpfp_parent: NewAll<StoredU64> = "*_parent_count",
        cpfp_child: NewAll<StoredU64> = "*_child_count",
    } }
    shape! { TransactionsFees at "series().transactions.fees" {
        count: FeesCount = "cpfp",
        fee: EffectiveFeeRate<Sats, EffectiveFeeRate6b<Sats>> = "*",
        fee_rate: SeriesPattern22<Option<FeeRate>> = "*_rate",
        effective_fee_rate: EffectiveFeeRate<Option<FeeRate>, EffectiveFeeRate6b<Option<FeeRate>>> = "effective_*_rate",
        is_cpfp_parent: SeriesPattern22<StoredBool> = "is_cpfp_parent",
        is_cpfp_child: SeriesPattern22<StoredBool> = "is_cpfp_child",
    } }
    shape! { OutputsCount<A> at "series().outputs.count" {
        total: A = "*",
    } }
    shape! { FeaturesCount at "series().transactions.features.count" {
        v1: SeriesPattern21<StoredU64> = "*_v1",
        v2: SeriesPattern21<StoredU64> = "*_v2",
        v3: SeriesPattern21<StoredU64> = "*_v3",
        other_version: SeriesPattern21<StoredU64> = "*_other_version",
        explicitly_rbf: SeriesPattern21<StoredU64> = "*_explicitly_rbf",
        one_input: SeriesPattern21<StoredU64> = "*_one_input",
        one_output: SeriesPattern21<StoredU64> = "*_one_output",
        p2pk: SeriesPattern21<StoredU64> = "*_p2pk",
        p2ms: SeriesPattern21<StoredU64> = "*_p2ms",
        p2pkh: SeriesPattern21<StoredU64> = "*_p2pkh",
        p2sh: SeriesPattern21<StoredU64> = "*_p2sh",
        p2wpkh: SeriesPattern21<StoredU64> = "*_p2wpkh",
        p2wsh: SeriesPattern21<StoredU64> = "*_p2wsh",
        p2tr: SeriesPattern21<StoredU64> = "*_p2tr",
        p2a: SeriesPattern21<StoredU64> = "*_p2a",
        op_return: SeriesPattern21<StoredU64> = "*_op_return",
        empty: SeriesPattern21<StoredU64> = "*_empty",
        unknown: SeriesPattern21<StoredU64> = "*_unknown",
        fake_pubkey: SeriesPattern21<StoredU64> = "*_fake_pubkey",
        fake_scripthash: SeriesPattern21<StoredU64> = "*_fake_scripthash",
        annex: NewAll<StoredU64> = "*_annex",
        sighash_all: NewAll<StoredU64> = "*_sighash_all",
        sighash_none: NewAll<StoredU64> = "*_sighash_none",
        sighash_single: NewAll<StoredU64> = "*_sighash_single",
        sighash_default: NewAll<StoredU64> = "*_sighash_default",
        sighash_anyone_can_pay: NewAll<StoredU64> = "*_sighash_anyone_can_pay",
        dust_output: NewAll<StoredU64> = "*_dust_output",
    } }
    shape! { Features at "series().transactions.features" {
        count: FeaturesCount = "tx_count",
        has_p2pk: SeriesPattern22<StoredBool> = "*_p2pk",
        has_p2ms: SeriesPattern22<StoredBool> = "*_p2ms",
        has_p2pkh: SeriesPattern22<StoredBool> = "*_p2pkh",
        has_p2sh: SeriesPattern22<StoredBool> = "*_p2sh",
        has_p2wpkh: SeriesPattern22<StoredBool> = "*_p2wpkh",
        has_p2wsh: SeriesPattern22<StoredBool> = "*_p2wsh",
        has_p2tr: SeriesPattern22<StoredBool> = "*_p2tr",
        has_p2a: SeriesPattern22<StoredBool> = "*_p2a",
        has_op_return: SeriesPattern22<StoredBool> = "*_op_return",
        has_empty: SeriesPattern22<StoredBool> = "*_empty",
        has_unknown: SeriesPattern22<StoredBool> = "*_unknown",
        has_fake_pubkey: SeriesPattern22<StoredBool> = "*_fake_pubkey",
        has_fake_scripthash: SeriesPattern22<StoredBool> = "*_fake_scripthash",
        has_inscription: SeriesPattern22<StoredBool> = "*_inscription",
        has_annex: SeriesPattern22<StoredBool> = "*_annex",
        has_sighash_all: SeriesPattern22<StoredBool> = "*_sighash_all",
        has_sighash_none: SeriesPattern22<StoredBool> = "*_sighash_none",
        has_sighash_single: SeriesPattern22<StoredBool> = "*_sighash_single",
        has_sighash_default: SeriesPattern22<StoredBool> = "*_sighash_default",
        has_sighash_anyone_can_pay: SeriesPattern22<StoredBool> = "*_sighash_anyone_can_pay",
        has_dust_output: SeriesPattern22<StoredBool> = "*_dust_output",
    } }
    shape! { BlocksCount at "series().blocks.count" {
        target: Target = "*_target",
        total: NewAll<StoredU64> = "*",
    } }
    shape! { BlocksWeight at "series().blocks.weight" {
        base: SeriesPattern21<Weight> = "*",
        cumulative: SeriesPattern2<Weight64> = "*_cumulative",
        sum: PerSec<Weight64> = "*_sum",
        average: PerSec<Option<StoredF32>> = "*_average",
        min: PerSec<Weight64> = "*_min",
        max: PerSec<Weight64> = "*_max",
        pct10: PerSec<Weight64> = "*_pct10",
        pct25: PerSec<Weight64> = "*_pct25",
        median: PerSec<Weight64> = "*_median",
        pct75: PerSec<Weight64> = "*_pct75",
        pct90: PerSec<Weight64> = "*_pct90",
    } }
    shape! { BlocksSize at "series().blocks.size" {
        base: SeriesPattern21<StoredU64> = "total_*",
        cumulative: SeriesPattern2<StoredU64> = "block_*_cumulative",
        sum: PerSec<StoredU64> = "block_*_sum",
        average: PerSec<Option<StoredF32>> = "block_*_average",
        min: PerSec<StoredU64> = "block_*_min",
        max: PerSec<StoredU64> = "block_*_max",
        pct10: PerSec<StoredU64> = "block_*_pct10",
        pct25: PerSec<StoredU64> = "block_*_pct25",
        median: PerSec<StoredU64> = "block_*_median",
        pct75: PerSec<StoredU64> = "block_*_pct75",
        pct90: PerSec<StoredU64> = "block_*_pct90",
    } }
    shape! { Time at "series().blocks.time" {
        timestamp: SeriesPattern21<Timestamp> = "*",
    } }
    shape! { Gini<A> at "series().indicators.gini" {
        ppm: SeriesPattern2<A> = "*_ppm",
        ratio: SeriesPattern2<Option<StoredF32>> = "*_ratio",
        percent: SeriesPattern2<Option<StoredF32>> = "*",
    } }
    shape! { Relative at "series().distribution_aggregated.cohorts.all.relative" {
        supply_dominance: Gini<Option<PartsPerMillion32>> = "*_supply_dominance",
        supply_in_profit_share: Gini<Option<PartsPerMillion32>> = "*_supply_in_profit_share",
        supply_in_loss_share: Gini<Option<PartsPerMillion32>> = "*_supply_in_loss_share",
        unrealized_profit_to_mcap: Gini<Option<PartsPerMillion32>> = "*_unrealized_profit_to_mcap",
        unrealized_loss_to_mcap: Gini<Option<PartsPerMillion32>> = "*_unrealized_loss_to_mcap",
        unrealized_profit_to_own_mcap: Gini<Option<PartsPerMillion32>> = "*_unrealized_profit_to_own_mcap",
        unrealized_loss_to_own_mcap: Gini<Option<PartsPerMillion32>> = "*_unrealized_loss_to_own_mcap",
        unrealized_profit_to_own_gross_pnl: Gini<Option<PartsPerMillion32>> = "*_unrealized_profit_to_own_gross_pnl",
        unrealized_loss_to_own_gross_pnl: Gini<Option<PartsPerMillion32>> = "*_unrealized_loss_to_own_gross_pnl",
        net_unrealized_pnl_to_own_gross_pnl: Gini<Option<PartsPerMillionSigned32>> = "*_net_unrealized_pnl_to_own_gross_pnl",
        invested_capital_in_profit_share: Gini<Option<PartsPerMillion32>> = "*_invested_capital_in_profit_share",
        invested_capital_in_loss_share: Gini<Option<PartsPerMillion32>> = "*_invested_capital_in_loss_share",
        realized_cap_to_own_mcap: Gini<Option<PartsPerMillion32>> = "*_realized_cap_to_own_mcap",
        net_pnl_change_1m_to_mcap: Gini<Option<PartsPerMillionSigned64>> = "*_net_pnl_change_1m_to_mcap",
        net_pnl_change_1m_to_rcap: Gini<Option<PartsPerMillionSigned64>> = "*_net_pnl_change_1m_to_rcap",
    } }
    shape! { CohortsAllCostBasis at "series().distribution_aggregated.cohorts.all.cost_basis" {
        in_profit: UrpdAllCostBasis<Spot<Option<SatsFract>>> = "*_cost_basis_in_profit_per",
        in_loss: UrpdAllCostBasis<Spot<Option<SatsFract>>> = "*_cost_basis_in_loss_per",
        min: Spot<Option<SatsFract>> = "*_cost_basis_min",
        max: Spot<Option<SatsFract>> = "*_cost_basis_max",
        per_coin: PerCoin = "*_cost_basis_per_coin",
        per_dollar: PerCoin = "*_cost_basis_per_dollar",
        supply_density: Gini<Option<PartsPerMillion32>> = "*_supply_density",
    } }
    shape! { Aaopool at "series().pools.minor.aaopool" {
        blocks_mined: BlocksMined = "*_blocks_mined",
        dominance: Gini<Option<PartsPerMillion32>> = "*_dominance",
    } }
    shape! { Minor at "series().pools.minor" {
        blockfills: Aaopool = "*",
        ultimuspool: Aaopool = "ultimuspool",
        terrapool: Aaopool = "terrapool",
        onethash: Aaopool = "onethash",
        bitfarms: Aaopool = "bitfarms",
        huobipool: Aaopool = "huobipool",
        wayicn: Aaopool = "wayicn",
        canoepool: Aaopool = "canoepool",
        bitcoincom: Aaopool = "bitcoincom",
        pool175btc: Aaopool = "pool175btc",
        gbminers: Aaopool = "gbminers",
        axbt: Aaopool = "axbt",
        asicminer: Aaopool = "asicminer",
        bitminter: Aaopool = "bitminter",
        bitcoinrussia: Aaopool = "bitcoinrussia",
        btcserv: Aaopool = "btcserv",
        simplecoinus: Aaopool = "simplecoinus",
        ozcoin: Aaopool = "ozcoin",
        eclipsemc: Aaopool = "eclipsemc",
        maxbtc: Aaopool = "maxbtc",
        triplemining: Aaopool = "triplemining",
        coinlab: Aaopool = "coinlab",
        pool50btc: Aaopool = "pool50btc",
        ghashio: Aaopool = "ghashio",
        stminingcorp: Aaopool = "stminingcorp",
        bitparking: Aaopool = "bitparking",
        mmpool: Aaopool = "mmpool",
        polmine: Aaopool = "polmine",
        kncminer: Aaopool = "kncminer",
        bitalo: Aaopool = "bitalo",
        hhtt: Aaopool = "hhtt",
        megabigpower: Aaopool = "megabigpower",
        mtred: Aaopool = "mtred",
        nmcbit: Aaopool = "nmcbit",
        yourbtcnet: Aaopool = "yourbtcnet",
        givemecoins: Aaopool = "givemecoins",
        multicoinco: Aaopool = "multicoinco",
        bcpoolio: Aaopool = "bcpoolio",
        cointerra: Aaopool = "cointerra",
        kanopool: Aaopool = "kanopool",
        solock: Aaopool = "solock",
        ckpool: Aaopool = "ckpool",
        nicehash: Aaopool = "nicehash",
        bitclub: Aaopool = "bitclub",
        bitcoinaffiliatenetwork: Aaopool = "bitcoinaffiliatenetwork",
        exxbw: Aaopool = "exxbw",
        bitsolo: Aaopool = "bitsolo",
        twentyoneinc: Aaopool = "twentyoneinc",
        digitalbtc: Aaopool = "digitalbtc",
        eightbaochi: Aaopool = "eightbaochi",
        mybtccoinpool: Aaopool = "mybtccoinpool",
        tbdice: Aaopool = "tbdice",
        hashpool: Aaopool = "hashpool",
        nexious: Aaopool = "nexious",
        bravomining: Aaopool = "bravomining",
        hotpool: Aaopool = "hotpool",
        okexpool: Aaopool = "okexpool",
        bcmonster: Aaopool = "bcmonster",
        onehash: Aaopool = "onehash",
        bixin: Aaopool = "bixin",
        tatmaspool: Aaopool = "tatmaspool",
        connectbtc: Aaopool = "connectbtc",
        batpool: Aaopool = "batpool",
        waterhole: Aaopool = "waterhole",
        dcexploration: Aaopool = "dcexploration",
        dcex: Aaopool = "dcex",
        btpool: Aaopool = "btpool",
        fiftyeightcoin: Aaopool = "fiftyeightcoin",
        bitcoinindia: Aaopool = "bitcoinindia",
        shawnp0wers: Aaopool = "shawnp0wers",
        phashio: Aaopool = "phashio",
        rigpool: Aaopool = "rigpool",
        haozhuzhu: Aaopool = "haozhuzhu",
        sevenpool: Aaopool = "sevenpool",
        miningkings: Aaopool = "miningkings",
        hashbx: Aaopool = "hashbx",
        dpool: Aaopool = "dpool",
        rawpool: Aaopool = "rawpool",
        haominer: Aaopool = "haominer",
        helix: Aaopool = "helix",
        bitcoinukraine: Aaopool = "bitcoinukraine",
        secretsuperstar: Aaopool = "secretsuperstar",
        tigerpoolnet: Aaopool = "tigerpoolnet",
        sigmapoolcom: Aaopool = "sigmapoolcom",
        okpooltop: Aaopool = "okpooltop",
        hummerpool: Aaopool = "hummerpool",
        tangpool: Aaopool = "tangpool",
        bytepool: Aaopool = "bytepool",
        novablock: Aaopool = "novablock",
        miningcity: Aaopool = "miningcity",
        minerium: Aaopool = "minerium",
        lubiancom: Aaopool = "lubiancom",
        okkong: Aaopool = "okkong",
        aaopool: Aaopool = "aaopool",
        emcdpool: Aaopool = "emcdpool",
        arkpool: Aaopool = "arkpool",
        purebtccom: Aaopool = "purebtccom",
        kucoinpool: Aaopool = "kucoinpool",
        entrustcharitypool: Aaopool = "entrustcharitypool",
        okminer: Aaopool = "okminer",
        titan: Aaopool = "titan",
        pegapool: Aaopool = "pegapool",
        btcnuggets: Aaopool = "btcnuggets",
        cloudhashing: Aaopool = "cloudhashing",
        digitalxmintsy: Aaopool = "digitalxmintsy",
        telco214: Aaopool = "telco214",
        btcpoolparty: Aaopool = "btcpoolparty",
        multipool: Aaopool = "multipool",
        transactioncoinmining: Aaopool = "transactioncoinmining",
        btcdig: Aaopool = "btcdig",
        trickysbtcpool: Aaopool = "trickysbtcpool",
        btcmp: Aaopool = "btcmp",
        eobot: Aaopool = "eobot",
        unomp: Aaopool = "unomp",
        patels: Aaopool = "patels",
        gogreenlight: Aaopool = "gogreenlight",
        bitcoinindiapool: Aaopool = "bitcoinindiapool",
        ekanembtc: Aaopool = "ekanembtc",
        canoe: Aaopool = "canoe",
        tiger: Aaopool = "tiger",
        onem1x: Aaopool = "onem1x",
        zulupool: Aaopool = "zulupool",
        wiz: Aaopool = "wiz",
        wk057: Aaopool = "wk057",
        futurebitapollosolo: Aaopool = "futurebitapollosolo",
        carbonnegative: Aaopool = "carbonnegative",
        portlandhodl: Aaopool = "portlandhodl",
        phoenix: Aaopool = "phoenix",
        neopool: Aaopool = "neopool",
        maxipool: Aaopool = "maxipool",
        bitfufupool: Aaopool = "bitfufupool",
        gdpool: Aaopool = "gdpool",
        miningdutch: Aaopool = "miningdutch",
        publicpool: Aaopool = "publicpool",
        miningsquared: Aaopool = "miningsquared",
        innopolistech: Aaopool = "innopolistech",
        btclab: Aaopool = "btclab",
        parasite: Aaopool = "parasite",
        redrockpool: Aaopool = "redrockpool",
        est3lar: Aaopool = "est3lar",
        braiinssolo: Aaopool = "braiinssolo",
        solopool: Aaopool = "solopool",
        noderunners: Aaopool = "noderunners",
        dmnd: Aaopool = "dmnd",
    } }
    shape! { Rsi1m at "series().market.technical.rsi._1m" {
        rsi: Gini<Option<PartsPerMillion32>> = "rsi_*",
        stoch_rsi_k: Gini<Option<PartsPerMillion32>> = "rsi_stoch_k_*",
        stoch_rsi_d: Gini<Option<PartsPerMillion32>> = "rsi_stoch_d_*",
    } }
    shape! { Macd<A> at "series().market.technical.macd" {
        _24h: A = "*",
        _1w: A = "1w",
        _1m: A = "1m",
    } }
    shape! { Technical at "series().market.technical" {
        rsi: Macd<Rsi1m> = "*",
        pi_cycle: RhodlRatio<Option<PartsPerMillion32>> = "pi_cycle",
        macd: Macd<Macd1m> = "*",
    } }
    shape! { Range at "series().market.range" {
        min: Max = "*_min",
        max: Max = "*_max",
        true_range: SeriesPattern2<Option<StoredF32>> = "*_true_range",
        true_range_sum_2w: SeriesPattern2<Option<StoredF32>> = "*_true_range_sum_2w",
        choppiness_index_2w: Gini<Option<PartsPerMillion32>> = "*_choppiness_index_2w",
    } }
    shape! { Cagr at "series().market.returns.cagr" {
        _2y: Gini<Option<PartsPerMillionSigned64>> = "*_2y",
        _3y: Gini<Option<PartsPerMillionSigned64>> = "*_3y",
        _4y: Gini<Option<PartsPerMillionSigned64>> = "*_4y",
        _5y: Gini<Option<PartsPerMillionSigned64>> = "*_5y",
        _6y: Gini<Option<PartsPerMillionSigned64>> = "*_6y",
        _8y: Gini<Option<PartsPerMillionSigned64>> = "*_8y",
        _10y: Gini<Option<PartsPerMillionSigned64>> = "*_10y",
    } }
    shape! { MarketLookback<A> at "series().market.lookback" {
        _24h: A = "*_24h",
        _1w: A = "*_1w",
        _1m: A = "*_1m",
        _3m: A = "*_3m",
        _6m: A = "*_6m",
        _1y: A = "*_1y",
        _2y: A = "*_2y",
        _3y: A = "*_3y",
        _4y: A = "*_4y",
        _5y: A = "*_5y",
        _6y: A = "*_6y",
        _8y: A = "*_8y",
        _10y: A = "*_10y",
    } }
    shape! { Ath at "series().market.ath" {
        high: Spot<Option<SatsFract>> = "*_ath",
        drawdown: Gini<Option<PartsPerMillionSigned32>> = "*_drawdown",
        days_since: SeriesPattern2<Option<StoredF32>> = "days_since_*_ath",
        years_since: SeriesPattern2<Option<StoredF32>> = "years_since_*_ath",
        max_days_between: SeriesPattern2<Option<StoredF32>> = "max_days_between_*_ath",
        max_years_between: SeriesPattern2<Option<StoredF32>> = "max_years_between_*_ath",
    } }
    shape! { Indicators at "series().indicators" {
        puell_multiple: Nvt = "puell_multiple",
        nvt: Nvt = "nvt",
        gini: Gini<Option<PartsPerMillion32>> = "gini",
        rhodl_ratio: RhodlRatio<Option<PartsPerMillion64>> = "rhodl_ratio",
        thermo_cap_multiple: Nvt = "thermo_cap_multiple",
        coindays_destroyed_supply_adj: SeriesPattern2<Option<StoredF32>> = "coindays_*",
        coinyears_destroyed_supply_adj: SeriesPattern2<Option<StoredF32>> = "coinyears_*",
        dormancy: Dormancy = "dormancy",
        stock_to_flow: SeriesPattern2<Option<StoredF32>> = "stock_to_flow",
        seller_exhaustion: SeriesPattern2<Option<StoredF32>> = "seller_exhaustion",
    } }
    shape! { Capitulation<A> at "series().rarity_meter.extremes.capitulation" {
        threshold_pct0_1: SeriesPattern2<A> = "*_threshold_pct0_1",
        threshold_pct0_05: SeriesPattern2<A> = "*_threshold_pct0_05",
        threshold_pct0_025: SeriesPattern2<A> = "*_threshold",
        tail: Gini<Option<PartsPerMillion32>> = "*_tail",
        rank: SeriesPattern2<StoredU8> = "*_rank",
    } }
    shape! { Extremes at "series().rarity_meter.extremes" {
        coins_in_loss: Capitulation<Option<Bitcoin>> = "*_coins_in_loss",
        profit_taking: Capitulation<Option<Dollars>> = "*_profit_taking",
        capitulation: Capitulation<Option<Dollars>> = "*_capitulation",
        peak_regret: Capitulation<Option<Dollars>> = "*_peak_regret",
        seller_exhaustion: Capitulation<Option<StoredF32>> = "*_seller_exhaustion",
    } }
    shape! { RarityMeter at "series().rarity_meter" {
        components: Components = "price",
        extremes: Extremes = "*",
        full: Cycle = "*",
        full_v2: Cycle = "*_v2",
        local: Cycle = "local_*",
        local_v2: Cycle = "local_*_v2",
        cycle: Cycle = "cycle_*",
        cycle_v2: Cycle = "cycle_*_v2",
    } }
    shape! { Adjusted at "series().cointime.adjusted" {
        inflation_rate: Gini<Option<PartsPerMillionSigned32>> = "*_inflation_rate",
        tx_velocity_native: SeriesPattern2<Option<StoredF64>> = "*_tx_velocity_btc",
        tx_velocity_fiat: SeriesPattern2<Option<StoredF64>> = "*_tx_velocity_usd",
    } }
    shape! { SupplyDensity at "series().coinflow.urpd.all.supply_density" {
        total: Gini<Option<PartsPerMillion32>> = "*_total",
        in_profit: Gini<Option<PartsPerMillion32>> = "*_in_profit",
        in_loss: Gini<Option<PartsPerMillion32>> = "*_in_loss",
    } }
    shape! { CoinflowUrpdLth at "series().coinflow.urpd.lth" {
        cost_basis: UrpdAllCostBasis<PerCoin> = "*_coinflow_cost_basis_per",
        capitalized_price: CapitalizedPrice = "coinflow_urpd_*_capitalized_price",
        supply_density: SupplyDensity = "coinflow_urpd_*_supply_density",
    } }
    shape! { CointimeUrpdLth at "series().cointime.urpd.lth" {
        cost_basis: UrpdAllCostBasis<PerCoin> = "*_cointime_cost_basis_per",
        capitalized_price: CapitalizedPrice = "cointime_urpd_*_capitalized_price",
        supply_density: SupplyDensity = "cointime_urpd_*_supply_density",
    } }
    shape! { UrpdAll at "series().coinflow.urpd.all" {
        cost_basis: UrpdAllCostBasis<PerCoin> = "*_cost_basis_per",
        capitalized_price: CapitalizedPrice = "*_urpd_all_capitalized_price",
        supply_density: SupplyDensity = "*_urpd_all_supply_density",
    } }
    shape! { CoinflowUrpd<A> at "series().coinflow.urpd" {
        all: UrpdAll = "*",
        sth: A = "sth",
        lth: A = "lth",
        under_4m: A = "under_4m",
        under_6m: A = "under_6m",
        over_4m: A = "over_4m",
        over_6m: A = "over_6m",
    } }
    shape! { HashratePrice at "series().mining.hashrate.price" {
        ths: SeriesPattern2<Option<StoredF32>> = "*_ths",
        ths_min: SeriesPattern2<Option<StoredF32>> = "*_ths_min",
        phs: SeriesPattern2<Option<StoredF32>> = "*_phs",
        phs_min: SeriesPattern2<Option<StoredF32>> = "*_phs_min",
        rebound: Gini<Option<PartsPerMillionSigned32>> = "*_rebound",
    } }
    shape! { HashrateRate at "series().mining.hashrate.rate" {
        base: SeriesPattern2<Option<StoredF64>> = "*",
        sma: RateSma = "*_sma",
        ath: SeriesPattern2<Option<StoredF64>> = "*_ath",
        drawdown: Gini<Option<PartsPerMillionSigned32>> = "*_drawdown",
    } }
    shape! { Hashrate at "series().mining.hashrate" {
        rate: HashrateRate = "*_rate",
        price: HashratePrice = "*_price",
        value: HashratePrice = "*_value",
    } }
    shape! { DataBytesAscribe at "series().op_return.by_kind.data_bytes.ascribe" {
        block: SeriesPattern21<Bytes> = "*_data_bytes",
        cumulative: SeriesPattern2<Bytes> = "*_data_bytes_cumulative",
        sum: PerSec<Bytes> = "*_data_bytes_sum",
        average: PerSec<Option<StoredF32>> = "*_data_bytes_average",
        data_share: Gini<Option<PartsPerMillion32>> = "*_data_share",
        chain_share: Gini<Option<PartsPerMillion32>> = "*_chain_share",
    } }
    shape! { PolicyDataBytes<A> at "series().op_return.policy.data_bytes" {
        pre_v30_standard: A = "op_return_policy_pre_v30_standard_*",
        pre_v30_nonstandard: A = "op_return_policy_pre_v30_nonstandard_*",
        oversized: A = "op_return_policy_oversized_*",
        multiple: A = "op_return_policy_multiple_*",
    } }
    shape! { ByKindDataBytes<A> at "series().op_return.by_kind.data_bytes" {
        runes: A = "op_return_runes_*",
        veri_block: A = "op_return_veri_block_*",
        omni: A = "op_return_omni_*",
        stacks: A = "op_return_stacks_*",
        blockstack: A = "op_return_blockstack_*",
        colu: A = "op_return_colu_*",
        open_assets: A = "op_return_open_assets_*",
        komodo: A = "op_return_komodo_*",
        coin_spark: A = "op_return_coin_spark_*",
        poet: A = "op_return_poet_*",
        docproof: A = "op_return_docproof_*",
        open_timestamps: A = "op_return_open_timestamps_*",
        factom: A = "op_return_factom_*",
        eternity_wall: A = "op_return_eternity_wall_*",
        memo: A = "op_return_memo_*",
        bitproof: A = "op_return_bitproof_*",
        ascribe: A = "op_return_ascribe_*",
        stampery: A = "op_return_stampery_*",
        epobc: A = "op_return_epobc_*",
        bare_hash: A = "op_return_bare_hash_*",
        text: A = "op_return_text_*",
        empty: A = "op_return_empty_*",
        unknown: A = "op_return_unknown_*",
    } }
    shape! { AllRate at "series().addrs.delta.all.rate" {
        _24h: Gini<Option<PartsPerMillionSigned64>> = "*_24h_rate",
        _1w: Gini<Option<PartsPerMillionSigned64>> = "*_1w_rate",
        _1m: Gini<Option<PartsPerMillionSigned64>> = "*_1m_rate",
        _1y: Gini<Option<PartsPerMillionSigned64>> = "*_1y_rate",
    } }
    shape! { FeeShare at "series().op_return.total.fee_share" {
        ppm: SeriesPattern2<Option<PartsPerMillion32>> = "*_ppm",
        ratio: SeriesPattern2<Option<StoredF32>> = "*_ratio",
        percent: SeriesPattern2<Option<StoredF32>> = "*",
        _24h: Gini<Option<PartsPerMillion32>> = "*_24h",
        _1w: Gini<Option<PartsPerMillion32>> = "*_1w",
        _1m: Gini<Option<PartsPerMillion32>> = "*_1m",
        _1y: Gini<Option<PartsPerMillion32>> = "*_1y",
    } }
    shape! { FeesAscribe at "series().op_return.by_kind.fees.ascribe" {
        block: SeriesPattern21<Sats> = "*_fees",
        cumulative: SeriesPattern2<Sats> = "*_fees_cumulative",
        sum: PerSec<Sats> = "*_fees_sum",
        average: PerSec<Option<StoredF32>> = "*_fees_average",
        fee_share: FeeShare = "*_fee_share",
    } }
    shape! { PolicyFees at "series().op_return.policy.fees" {
        pre_v30_standard: FeesAscribe = "*_pre_v30_standard",
        pre_v30_nonstandard: FeesAscribe = "*_pre_v30_nonstandard",
        oversized: FeesAscribe = "*_oversized",
        multiple: FeesAscribe = "*_multiple",
    } }
    shape! { ByKindFees at "series().op_return.by_kind.fees" {
        runes: FeesAscribe = "*_runes",
        veri_block: FeesAscribe = "*_veri_block",
        omni: FeesAscribe = "*_omni",
        stacks: FeesAscribe = "*_stacks",
        blockstack: FeesAscribe = "*_blockstack",
        colu: FeesAscribe = "*_colu",
        open_assets: FeesAscribe = "*_open_assets",
        komodo: FeesAscribe = "*_komodo",
        coin_spark: FeesAscribe = "*_coin_spark",
        poet: FeesAscribe = "*_poet",
        docproof: FeesAscribe = "*_docproof",
        open_timestamps: FeesAscribe = "*_open_timestamps",
        factom: FeesAscribe = "*_factom",
        eternity_wall: FeesAscribe = "*_eternity_wall",
        memo: FeesAscribe = "*_memo",
        bitproof: FeesAscribe = "*_bitproof",
        ascribe: FeesAscribe = "*_ascribe",
        stampery: FeesAscribe = "*_stampery",
        epobc: FeesAscribe = "*_epobc",
        bare_hash: FeesAscribe = "*_bare_hash",
        text: FeesAscribe = "*_text",
        empty: FeesAscribe = "*_empty",
        unknown: FeesAscribe = "*_unknown",
    } }
    shape! { ByKind<A, B, C, D> at "series().op_return.by_kind" {
        output_count: A = "output_count",
        data_bytes: B = "",
        tx_count: A = "tx_count",
        tx_vsize: C = "tx_vsize",
        fees: D = "*",
    } }
    shape! { Total at "series().op_return.total" {
        data_bytes: NewAll<Bytes> = "*_data_bytes",
        tx_count: NewAll<StoredU64> = "*_tx_count",
        tx_vsize: NewAll<VSize> = "*_tx_vsize",
        fees: NewAll<Sats> = "*_fees",
        chain_share: Gini<Option<PartsPerMillion32>> = "*_chain_share",
        fee_share: FeeShare = "*_fee_share",
    } }
    shape! { OpReturn at "series().op_return" {
        raw: OpReturnRaw = "index",
        total: Total = "*",
        by_kind: ByKind<ByKindDataBytes<NewAll<StoredU64>>, ByKindDataBytes<DataBytesAscribe>, ByKindDataBytes<NewAll<VSize>>, ByKindFees> = "*",
        policy: ByKind<PolicyDataBytes<NewAll<StoredU64>>, PolicyDataBytes<DataBytesAscribe>, PolicyDataBytes<NewAll<VSize>>, PolicyFees> = "*_policy",
    } }
    shape! { OutputsByTypeTxShare at "series().outputs.by_type.tx_share" {
        p2pk65: FeeShare = "*_p2pk65_output",
        p2pk33: FeeShare = "*_p2pk33_output",
        p2pkh: FeeShare = "*_p2pkh_output",
        p2ms: FeeShare = "*_p2ms_output",
        p2sh: FeeShare = "*_p2sh_output",
        p2wpkh: FeeShare = "*_p2wpkh_output",
        p2wsh: FeeShare = "*_p2wsh_output",
        p2tr: FeeShare = "*_p2tr_output",
        p2a: FeeShare = "*_p2a_output",
        unknown: FeeShare = "*_unknown_outputs_output",
        empty: FeeShare = "*_empty_outputs_output",
        op_return: FeeShare = "*_op_return_output",
    } }
    shape! { OutputShare at "series().outputs.by_type.output_share" {
        p2pk65: FeeShare = "p2pk65_*",
        p2pk33: FeeShare = "p2pk33_*",
        p2pkh: FeeShare = "p2pkh_*",
        p2ms: FeeShare = "p2ms_*",
        p2sh: FeeShare = "p2sh_*",
        p2wpkh: FeeShare = "p2wpkh_*",
        p2wsh: FeeShare = "p2wsh_*",
        p2tr: FeeShare = "p2tr_*",
        p2a: FeeShare = "p2a_*",
        unknown: FeeShare = "unknown_outputs_*",
        empty: FeeShare = "empty_outputs_*",
        op_return: FeeShare = "op_return_*",
    } }
    shape! { OutputsByType at "series().outputs.by_type" {
        output_count: OutputCount = "*_count",
        spendable_output_count: NewAll<StoredU64> = "spendable_*_count",
        output_share: OutputShare = "*_share",
        tx_count: OutputsByTypeTxCount = "tx_count",
        tx_share: OutputsByTypeTxShare = "tx_share_with",
    } }
    shape! { Outputs at "series().outputs" {
        raw: OutputsRaw = "type",
        spent: Spent = "txin_index",
        count: OutputsCount<InputsCount> = "*_count",
        per_sec: PerSec<Option<StoredF32>> = "outputs_per_sec",
        by_type: OutputsByType = "*",
        value: OutputsValue = "op_return_value",
    } }
    shape! { InputsByTypeTxShare at "series().inputs.by_type.tx_share" {
        p2pk65: FeeShare = "*_p2pk65_prevout",
        p2pk33: FeeShare = "*_p2pk33_prevout",
        p2pkh: FeeShare = "*_p2pkh_prevout",
        p2ms: FeeShare = "*_p2ms_prevout",
        p2sh: FeeShare = "*_p2sh_prevout",
        p2wpkh: FeeShare = "*_p2wpkh_prevout",
        p2wsh: FeeShare = "*_p2wsh_prevout",
        p2tr: FeeShare = "*_p2tr_prevout",
        p2a: FeeShare = "*_p2a_prevout",
        unknown: FeeShare = "*_unknown_outputs_prevout",
        empty: FeeShare = "*_empty_outputs_prevout",
    } }
    shape! { Sd24h<A> at "series().market.returns.sd_24h" {
        _24h: A = "*_24h",
        _1w: A = "*_1w",
        _1m: A = "*_1m",
        _1y: A = "*_1y",
    } }
    shape! { Returns at "series().market.returns" {
        periods: MarketLookback<Gini<Option<PartsPerMillionSigned64>>> = "*_return",
        cagr: Cagr = "*_cagr",
        sd_24h: Sd24h<Sd24h1m> = "",
    } }
    shape! { Market at "series().market" {
        ath: Ath = "*",
        lookback: MarketLookback<Spot<Option<SatsFract>>> = "*_past",
        returns: Returns = "*",
        volatility: PerSec<Option<StoredF32>> = "*_volatility",
        range: Range = "*",
        moving_average: MovingAverage = "*",
        technical: Technical = "24h",
    } }
    shape! { RewardsFees at "series().mining.rewards.fees" {
        block: BurnedBlock = "*",
        cumulative: Circulating<Sats, Option<Cents>> = "*_cumulative",
        sum: Sd24h<Circulating<Sats, Option<Cents>>> = "*_sum",
        average: Sd24h<Circulating<Option<StoredF32>, Option<StoredF32>>> = "*_average",
        min: Sd24h<Circulating<Sats, Option<Cents>>> = "*_min",
        max: Sd24h<Circulating<Sats, Option<Cents>>> = "*_max",
        pct10: Sd24h<Circulating<Sats, Option<Cents>>> = "*_pct10",
        pct25: Sd24h<Circulating<Sats, Option<Cents>>> = "*_pct25",
        median: Sd24h<Circulating<Sats, Option<Cents>>> = "*_median",
        pct75: Sd24h<Circulating<Sats, Option<Cents>>> = "*_pct75",
        pct90: Sd24h<Circulating<Sats, Option<Cents>>> = "*_pct90",
        dominance: FeeShare = "fee_dominance",
        to_subsidy: Sd24h<Gini<Option<PartsPerMillion64>>> = "fee_to_subsidy",
    } }
    shape! { Subsidy at "series().mining.rewards.subsidy" {
        block: BurnedBlock = "*",
        cumulative: Circulating<Sats, Option<Cents>> = "*_cumulative",
        sum: Sd24h<Circulating<Sats, Option<Cents>>> = "*_sum",
        average: Sd24h<Circulating<Option<StoredF32>, Option<StoredF32>>> = "*_average",
        dominance: FeeShare = "*_dominance",
    } }
    shape! { RealizedLoss0sats at "series().addrs.by_balance.realized_loss._0sats" {
        block: RealizedLoss0satsBlock<Option<Cents>> = "*",
        cumulative: CoinflowCap<Option<Cents>> = "*_cumulative",
        sum: Sd24h<CoinflowCap<Option<Cents>>> = "*_sum",
    } }
    shape! { DeltaAll<A> at "series().addrs.delta.all" {
        absolute: A = "*",
        rate: AllRate = "*",
    } }
    shape! { MarketCap at "series().supply.market_cap" {
        usd: SeriesPattern2<Option<Dollars>> = "*",
        cents: SeriesPattern2<Option<Cents>> = "*_cents",
        delta: DeltaAll<Sd24h<CoinflowCap<CentsSigned>>> = "*_delta",
    } }
    shape! { Supply at "series().supply" {
        circulating: Circulating<Sats, Option<Cents>> = "circulating_*",
        burned: Burned = "unspendable_*",
        inflation_rate: Gini<Option<PartsPerMillionSigned64>> = "inflation_rate",
        velocity: Velocity = "velocity",
        market_cap: MarketCap = "market_cap",
        market_minus_realized_cap_growth_rate: PerSec<Option<PartsPerMillionSigned64>> = "market_minus_realized_cap_growth_rate",
        hodled_or_lost: Circulating<Sats, Option<Cents>> = "hodled_or_lost_*",
    } }
    shape! { AllSupply at "series().distribution_aggregated.cohorts.all.supply" {
        total: Circulating<Sats, Option<Cents>> = "*",
        in_profit: Circulating<Sats, Option<Cents>> = "*_in_profit",
        in_loss: Circulating<Sats, Option<Cents>> = "*_in_loss",
        delta: DeltaAll<Sd24h<Absolute1m>> = "*_delta",
    } }
    shape! { Age10yTo12y at "series().cohorts.realized.net_pnl.age._10y_to_12y" {
        block: RealizedLoss0satsBlock<CentsSigned> = "*",
        cumulative: CoinflowCap<CentsSigned> = "*_cumulative",
        sum: Sd24h<CoinflowCap<CentsSigned>> = "*_sum",
        delta: DeltaAll<Sd24h<CoinflowCap<CentsSigned>>> = "*_delta",
    } }
    shape! { AvgBalance<A> at "series().addrs.avg_balance" {
        all: A = "*",
        p2pk65: A = "p2pk65_*",
        p2pk33: A = "p2pk33_*",
        p2pkh: A = "p2pkh_*",
        p2sh: A = "p2sh_*",
        p2wpkh: A = "p2wpkh_*",
        p2wsh: A = "p2wsh_*",
        p2tr: A = "p2tr_*",
        p2a: A = "p2a_*",
    } }
    shape! { ExposedSupply at "series().addrs.exposed.supply" {
        all: Circulating<Sats, Option<Cents>> = "*",
        p2pk65: Circulating<Sats, Option<Cents>> = "p2pk65_*",
        p2pk33: Circulating<Sats, Option<Cents>> = "p2pk33_*",
        p2pkh: Circulating<Sats, Option<Cents>> = "p2pkh_*",
        p2sh: Circulating<Sats, Option<Cents>> = "p2sh_*",
        p2wpkh: Circulating<Sats, Option<Cents>> = "p2wpkh_*",
        p2wsh: Circulating<Sats, Option<Cents>> = "p2wsh_*",
        p2tr: Circulating<Sats, Option<Cents>> = "p2tr_*",
        p2a: Circulating<Sats, Option<Cents>> = "p2a_*",
        share: AvgBalance<Gini<Option<PartsPerMillion32>>> = "*_share",
    } }
    shape! { Exposed at "series().addrs.exposed" {
        count: ExposedCount = "*_count",
        supply: ExposedSupply = "*_supply",
    } }
    shape! { Events at "series().addrs.respent.events" {
        output_to_reused_addr_count: AvgBalance<NewAll<StoredU64>> = "output_to_*_count",
        output_to_reused_addr_share: AvgBalance<FeeShare> = "output_to_*_share",
        spendable_output_to_reused_addr_share: FeeShare = "spendable_output_to_*_share",
        input_from_reused_addr_count: AvgBalance<NewAll<StoredU64>> = "input_from_*_count",
        input_from_reused_addr_share: AvgBalance<FeeShare> = "input_from_*_share",
        active_reused_addr_count: Interval<StoredU32> = "active_*_count",
        active_reused_addr_share: Interval<Option<StoredF32>> = "active_*_share",
    } }
    shape! { Respent at "series().addrs.respent" {
        count: ExposedCount = "*_count",
        events: Events = "*",
        supply: ExposedSupply = "*_supply",
    } }
    shape! { AddrsActivity at "series().addrs.activity" {
        reactivated: AvgBalance<Interval<StoredU32>> = "reactivated_*",
        sending: AvgBalance<Interval<StoredU32>> = "sending_*",
        receiving: AvgBalance<Interval<StoredU32>> = "receiving_*",
        bidirectional: AvgBalance<Interval<StoredU32>> = "bidirectional_*",
        active: AvgBalance<Interval<StoredU32>> = "active_*",
    } }
    shape! { UtxoCount0sats at "series().addrs.by_balance.utxo_count._0sats" {
        base: SeriesPattern2<StoredU64> = "*",
        delta: DeltaAll<PerSec<StoredI64>> = "*_delta",
    } }
    shape! { AllOutputs at "series().distribution_aggregated.cohorts.all.outputs" {
        unspent_count: UtxoCount0sats = "*_utxo_count",
        spent_count: NewAll<StoredU64> = "*_spent_utxo_count",
    } }
    shape! { Supply0sats at "series().addrs.by_balance.supply._0sats" {
        total: Circulating<Sats, Option<Cents>> = "*",
        delta: DeltaAll<Sd24h<Absolute1m>> = "*_delta",
        dominance: Gini<Option<PartsPerMillion32>> = "*_dominance",
    } }
    shape! { Coinbase<A, B, C> at "series().mining.rewards.coinbase" {
        block: A = "*",
        cumulative: B = "*_cumulative",
        sum: Sd24h<B> = "*_sum",
        average: Sd24h<C> = "*_average",
    } }
    shape! { AdjustedSopr at "series().distribution_aggregated.cohorts.all.ratios.adjusted_sopr" {
        ratio: PerSec<Option<StoredF32>> = "*_adjusted_sopr",
        transfer_volume: Coinbase<RealizedLoss0satsBlock<Option<Cents>>, CoinflowCap<Option<Cents>>, CoinflowCap<Option<StoredF32>>> = "*_adj_value_created",
        value_destroyed: Coinbase<RealizedLoss0satsBlock<Option<Cents>>, CoinflowCap<Option<Cents>>, CoinflowCap<Option<StoredF32>>> = "*_adj_value_destroyed",
    } }
    shape! { Ratios at "series().distribution_aggregated.cohorts.all.ratios" {
        adjusted_sopr: AdjustedSopr = "*",
        dormancy: PerSec<Option<StoredF32>> = "*_dormancy",
        sopr: SeriesPattern2<Option<StoredF32>> = "*_sopr_24h",
        sopr_ratio_extended: SoprRatioExtended = "*_sopr",
        sell_side_risk_ratio: Sd24h<Gini<Option<PartsPerMillion32>>> = "*_sell_side_risk_ratio",
        profit_to_loss_ratio: PerSec<Option<StoredF32>> = "*_realized_profit_to_loss_ratio",
    } }
    shape! { AllRealized at "series().distribution_aggregated.cohorts.all.realized" {
        cap: MarketCap = "*_realized_cap",
        price: Spot<Option<SatsFract>> = "*_realized_price",
        capitalized_price: CapitalizedPrice = "*_capitalized_price",
        profit: RealizedLoss0sats = "*_realized_profit",
        loss: RealizedLoss0sats = "*_realized_loss",
        net_pnl: Age10yTo12y = "*_net_realized_pnl",
        value_destroyed: Coinbase<RealizedLoss0satsBlock<Option<Cents>>, CoinflowCap<Option<Cents>>, CoinflowCap<Option<StoredF32>>> = "*_value_destroyed",
        gross_pnl: RealizedLoss0sats = "*_realized_gross_pnl",
        peak_regret: RealizedLoss0sats = "*_realized_peak_regret",
        mvrv: RhodlRatio<Option<PriceRatio>> = "*_mvrv",
    } }
    shape! { AllActivity at "series().distribution_aggregated.cohorts.all.activity" {
        transfer_volume: Coinbase<BurnedBlock, Circulating<Sats, Option<Cents>>, Circulating<Option<StoredF32>, Option<StoredF32>>> = "*_transfer_volume",
        transfer_volume_in_profit: Coinbase<BurnedBlock, Circulating<Sats, Option<Cents>>, Circulating<Option<StoredF32>, Option<StoredF32>>> = "*_transfer_volume_in_profit",
        transfer_volume_in_loss: Coinbase<BurnedBlock, Circulating<Sats, Option<Cents>>, Circulating<Option<StoredF32>, Option<StoredF32>>> = "*_transfer_volume_in_loss",
        coindays_destroyed: NewAll<Option<StoredF64>> = "*_coindays_destroyed",
        coinyears_destroyed: SeriesPattern2<Option<StoredF64>> = "*_coinyears_destroyed",
    } }
    shape! { CohortsAll at "series().distribution_aggregated.cohorts.all" {
        supply: AllSupply = "*_supply",
        outputs: AllOutputs = "*",
        activity: AllActivity = "*",
        realized: AllRealized = "*",
        unrealized: AllUnrealized = "*",
        cost_basis: CohortsAllCostBasis = "*",
        ratios: Ratios = "*",
        relative: Relative = "*",
    } }
    shape! { DistributionAggregatedCohorts at "series().distribution_aggregated.cohorts" {
        all: CohortsAll = "",
        sth: CohortsAll = "sth",
        lth: CohortsAll = "lth",
        under_4m: CohortsAll = "*_4m",
        under_6m: CohortsAll = "*_6m",
        over_4m: CohortsAll = "over_4m",
        over_6m: CohortsAll = "over_6m",
    } }
    shape! { DistributionAggregated at "series().distribution_aggregated" {
        cohorts: DistributionAggregatedCohorts = "*",
    } }
    shape! { UtxoAmount<A> at "series().cohorts.activity.transfer_volume.utxo_amount" {
        _0sats: A = "utxos_0sats_*",
        _1sat_to_10sats: A = "utxos_1sat_to_10sats_*",
        _10sats_to_100sats: A = "utxos_10sats_to_100sats_*",
        _100sats_to_1k_sats: A = "utxos_100sats_to_1k_sats_*",
        _1k_sats_to_10k_sats: A = "utxos_1k_sats_to_10k_sats_*",
        _10k_sats_to_100k_sats: A = "utxos_10k_sats_to_100k_sats_*",
        _100k_sats_to_1m_sats: A = "utxos_100k_sats_to_1m_sats_*",
        _1m_sats_to_10m_sats: A = "utxos_1m_sats_to_10m_sats_*",
        _10m_sats_to_1btc: A = "utxos_10m_sats_to_1btc_*",
        _1btc_to_10btc: A = "utxos_1btc_to_10btc_*",
        _10btc_to_100btc: A = "utxos_10btc_to_100btc_*",
        _100btc_to_1k_btc: A = "utxos_100btc_to_1k_btc_*",
        _1k_btc_to_10k_btc: A = "utxos_1k_btc_to_10k_btc_*",
        _10k_btc_to_100k_btc: A = "utxos_10k_btc_to_100k_btc_*",
        over_100k_btc: A = "utxos_over_100k_btc_*",
    } }
    shape! { Class<A> at "series().cohorts.activity.coindays_destroyed.class" {
        _2009: A = "class_2009_*",
        _2010: A = "class_2010_*",
        _2011: A = "class_2011_*",
        _2012: A = "class_2012_*",
        _2013: A = "class_2013_*",
        _2014: A = "class_2014_*",
        _2015: A = "class_2015_*",
        _2016: A = "class_2016_*",
        _2017: A = "class_2017_*",
        _2018: A = "class_2018_*",
        _2019: A = "class_2019_*",
        _2020: A = "class_2020_*",
        _2021: A = "class_2021_*",
        _2022: A = "class_2022_*",
        _2023: A = "class_2023_*",
        _2024: A = "class_2024_*",
        _2025: A = "class_2025_*",
        _2026: A = "class_2026_*",
    } }
    shape! { CoindaysDestroyedEpoch<A> at "series().cohorts.activity.coindays_destroyed.epoch" {
        _0: A = "epoch_0_*",
        _1: A = "epoch_1_*",
        _2: A = "epoch_2_*",
        _3: A = "epoch_3_*",
        _4: A = "epoch_4_*",
    } }
    shape! { Antpool at "series().pools.major.antpool" {
        blocks_mined: BlocksMined = "*_blocks_mined",
        dominance: FeeShare = "*_dominance",
        rewards: Coinbase<BurnedBlock, Circulating<Sats, Option<Cents>>, Circulating<Option<StoredF32>, Option<StoredF32>>> = "*_rewards",
    } }
    shape! { Major at "series().pools.major" {
        unknown: Antpool = "*",
        luxor: Antpool = "luxor",
        btccom: Antpool = "btccom",
        btctop: Antpool = "btctop",
        btcguild: Antpool = "btcguild",
        eligius: Antpool = "eligius",
        f2pool: Antpool = "f2pool",
        braiinspool: Antpool = "braiinspool",
        antpool: Antpool = "antpool",
        btcc: Antpool = "btcc",
        bwpool: Antpool = "bwpool",
        bitfury: Antpool = "bitfury",
        viabtc: Antpool = "viabtc",
        poolin: Antpool = "poolin",
        spiderpool: Antpool = "spiderpool",
        binancepool: Antpool = "binancepool",
        foundryusa: Antpool = "foundryusa",
        sbicrypto: Antpool = "sbicrypto",
        marapool: Antpool = "marapool",
        secpool: Antpool = "secpool",
        ocean: Antpool = "ocean",
        whitepool: Antpool = "whitepool",
    } }
    shape! { Pools at "series().pools" {
        pool: SeriesPattern21<PoolSlug> = "*",
        major: Major = "unknown",
        minor: Minor = "blockfills",
    } }
    shape! { Matured<A> at "series().cohorts.supply.matured" {
        under_1h: A = "utxos_under_1h_*",
        _1h_to_1d: A = "utxos_1h_to_1d_*",
        _1d_to_1w: A = "utxos_1d_to_1w_*",
        _1w_to_1m: A = "utxos_1w_to_1m_*",
        _1m_to_2m: A = "utxos_1m_to_2m_*",
        _2m_to_3m: A = "utxos_2m_to_3m_*",
        _3m_to_4m: A = "utxos_3m_to_4m_*",
        _4m_to_5m: A = "utxos_4m_to_5m_*",
        _5m_to_6m: A = "utxos_5m_to_6m_*",
        _6m_to_9m: A = "utxos_6m_to_9m_*",
        _9m_to_1y: A = "utxos_9m_to_1y_*",
        _1y_to_18m: A = "utxos_1y_to_18m_*",
        _18m_to_2y: A = "utxos_18m_to_2y_*",
        _2y_to_3y: A = "utxos_2y_to_3y_*",
        _3y_to_4y: A = "utxos_3y_to_4y_*",
        _4y_to_5y: A = "utxos_4y_to_5y_*",
        _5y_to_6y: A = "utxos_5y_to_6y_*",
        _6y_to_7y: A = "utxos_6y_to_7y_*",
        _7y_to_8y: A = "utxos_7y_to_8y_*",
        _8y_to_10y: A = "utxos_8y_to_10y_*",
        _10y_to_12y: A = "utxos_10y_to_12y_*",
        _12y_to_15y: A = "utxos_12y_to_15y_*",
        over_15y: A = "utxos_over_15y_*",
    } }
    shape! { CoindaysDestroyed<A> at "series().cohorts.activity.coindays_destroyed" {
        age: Matured<A> = "old_*",
        epoch: CoindaysDestroyedEpoch<A> = "*",
        class: Class<A> = "*",
    } }
    shape! { CohortsUnrealized at "series().cohorts.unrealized" {
        profit: CoindaysDestroyed<CoinflowCap<Option<Cents>>> = "*_profit",
        loss: CoindaysDestroyed<CoinflowCap<Option<Cents>>> = "*_loss",
        net_pnl: CoindaysDestroyed<CoinflowCap<CentsSigned>> = "net_*_pnl",
    } }
    shape! { CoinflowAgeRangeSupply at "series().coinflow.age_range.supply" {
        mobile: Matured<Circulating<Sats, Option<Cents>>> = "*_mobile_supply",
        immobile: Matured<Circulating<Sats, Option<Cents>>> = "*_immobile_supply",
    } }
    shape! { CoinflowAgeRange at "series().coinflow.age_range" {
        spending_rate: SpendingRate = "*_spending_rate",
        spending_exposure: SpendingExposure = "*",
        supply: CoinflowAgeRangeSupply = "*",
    } }
    shape! { Coinflow at "series().coinflow" {
        age_range: CoinflowAgeRange = "old",
        urpd: CoinflowUrpd<CoinflowUrpdLth> = "*",
        supply: CoinflowSupply = "",
        cap: CoinflowCap<Option<Cents>> = "*_cap",
        price: CapitalizedPrice = "*_price",
        capitalized_price: CapitalizedPrice = "*_capitalized_price",
        sth: CoinflowLth = "sth",
        lth: CoinflowLth = "lth",
        under_4m_price: CapitalizedPrice = "under_4m_*_price",
        under_4m_capitalized_price: CapitalizedPrice = "under_4m_*_capitalized_price",
        under_6m_price: CapitalizedPrice = "under_6m_*_price",
        under_6m_capitalized_price: CapitalizedPrice = "under_6m_*_capitalized_price",
        over_4m_price: CapitalizedPrice = "over_4m_*_price",
        over_4m_capitalized_price: CapitalizedPrice = "over_4m_*_capitalized_price",
        over_6m_price: CapitalizedPrice = "over_6m_*_price",
        over_6m_capitalized_price: CapitalizedPrice = "over_6m_*_capitalized_price",
    } }
    shape! { CointimeAgeRangeSupply at "series().cointime.age_range.supply" {
        awake: Matured<Circulating<Sats, Option<Cents>>> = "*_awake_supply",
        dormant: Matured<Circulating<Sats, Option<Cents>>> = "*_dormant_supply",
    } }
    shape! { CointimeAgeRange at "series().cointime.age_range" {
        coindays_consumed: Matured<NewAll<Option<StoredF64>>> = "*_coindays_consumed",
        coindays_stored: Matured<NewAll<Option<StoredF64>>> = "*_coindays_stored",
        activity: AgeRangeActivity = "*",
        supply: CointimeAgeRangeSupply = "*",
        coindays_created: Matured<NewAll<Option<StoredF64>>> = "*_coindays_created",
    } }
    shape! { Cointime at "series().cointime" {
        activity: CointimeActivity = "coinblocks",
        age_range: CointimeAgeRange = "old",
        urpd: CoinflowUrpd<CointimeUrpdLth> = "cointime",
        awake: Awake = "*",
        dormant: Dormant = "dormant_supply",
        sth: CointimeLth = "sth",
        lth: CointimeLth = "lth",
        under_4m_awake_price: CapitalizedPrice = "under_4m_*_price",
        under_4m_awake_capitalized_price: CapitalizedPrice = "under_4m_*_capitalized_price",
        under_6m_awake_price: CapitalizedPrice = "under_6m_*_price",
        under_6m_awake_capitalized_price: CapitalizedPrice = "under_6m_*_capitalized_price",
        over_4m_awake_price: CapitalizedPrice = "over_4m_*_price",
        over_4m_awake_capitalized_price: CapitalizedPrice = "over_4m_*_capitalized_price",
        over_6m_awake_price: CapitalizedPrice = "over_6m_*_price",
        over_6m_awake_capitalized_price: CapitalizedPrice = "over_6m_*_capitalized_price",
        supply: CointimeSupply = "supply",
        value: CointimeValue = "cointime_value",
        cap: CointimeCap = "cap",
        prices: CointimePrices = "price",
        adjusted: Adjusted = "cointime_adj",
        reserve_risk: ReserveRisk = "reserve_risk",
    } }
    shape! { Rewards at "series().mining.rewards" {
        coinbase: Coinbase<BurnedBlock, Circulating<Sats, Option<Cents>>, Circulating<Option<StoredF32>, Option<StoredF32>>> = "*",
        subsidy: Subsidy = "subsidy",
        fees: RewardsFees = "fees",
        output_volume: SeriesPattern21<Sats> = "output_volume",
        unclaimed: Burned = "unclaimed_rewards",
    } }
    shape! { Mining at "series().mining" {
        rewards: Rewards = "*",
        hashrate: Hashrate = "hash",
    } }
    shape! { RealizedCap<A> at "series().addrs.by_balance.realized_cap" {
        _0sats: A = "addrs_0sats_*",
        _1sat_to_10sats: A = "addrs_1sat_to_10sats_*",
        _10sats_to_100sats: A = "addrs_10sats_to_100sats_*",
        _100sats_to_1k_sats: A = "addrs_100sats_to_1k_sats_*",
        _1k_sats_to_10k_sats: A = "addrs_1k_sats_to_10k_sats_*",
        _10k_sats_to_100k_sats: A = "addrs_10k_sats_to_100k_sats_*",
        _100k_sats_to_1m_sats: A = "addrs_100k_sats_to_1m_sats_*",
        _1m_sats_to_10m_sats: A = "addrs_1m_sats_to_10m_sats_*",
        _10m_sats_to_1btc: A = "addrs_10m_sats_to_1btc_*",
        _1btc_to_10btc: A = "addrs_1btc_to_10btc_*",
        _10btc_to_100btc: A = "addrs_10btc_to_100btc_*",
        _100btc_to_1k_btc: A = "addrs_100btc_to_1k_btc_*",
        _1k_btc_to_10k_btc: A = "addrs_1k_btc_to_10k_btc_*",
        _10k_btc_to_100k_btc: A = "addrs_10k_btc_to_100k_btc_*",
        over_100k_btc: A = "addrs_over_100k_btc_*",
    } }
    shape! { Funded at "series().addrs.funded" {
        all: SeriesPattern2<StoredU64> = "*",
        p2pk65: SeriesPattern2<StoredU64> = "p2pk65_*",
        p2pk33: SeriesPattern2<StoredU64> = "p2pk33_*",
        p2pkh: SeriesPattern2<StoredU64> = "p2pkh_*",
        p2sh: SeriesPattern2<StoredU64> = "p2sh_*",
        p2wpkh: SeriesPattern2<StoredU64> = "p2wpkh_*",
        p2wsh: SeriesPattern2<StoredU64> = "p2wsh_*",
        p2tr: SeriesPattern2<StoredU64> = "p2tr_*",
        p2a: SeriesPattern2<StoredU64> = "p2a_*",
        balance: RealizedCap<UtxoCount0sats> = "*",
    } }
    shape! { ByBalance at "series().addrs.by_balance" {
        supply: RealizedCap<Supply0sats> = "supply",
        utxo_count: RealizedCap<UtxoCount0sats> = "utxo_count",
        transfer_volume: RealizedCap<Coinbase<BurnedBlock, Circulating<Sats, Option<Cents>>, Circulating<Option<StoredF32>, Option<StoredF32>>>> = "transfer_volume",
        realized_cap: RealizedCap<CoinflowCap<Option<Cents>>> = "*_cap",
        realized_profit: RealizedCap<RealizedLoss0sats> = "*_profit",
        realized_loss: RealizedCap<RealizedLoss0sats> = "*_loss",
    } }
    shape! { Addrs at "series().addrs" {
        raw: AddrsRaw = "p2pk65",
        state: State = "*",
        by_balance: ByBalance = "realized",
        funded: Funded = "*_count",
        empty: AddrsEmpty = "empty_*_count",
        activity: AddrsActivity = "addrs",
        total: AddrsEmpty = "total_*_count",
        new: AvgBalance<NewAll<StoredU64>> = "new_*_count",
        reused: Respent = "reused_*",
        respent: Respent = "respent_*",
        exposed: Exposed = "exposed_*",
        delta: AvgBalance<DeltaAll<PerSec<StoredI64>>> = "*_count",
        avg_balance: AvgBalance<Circulating<Sats, Option<Cents>>> = "avg_*_amount",
    } }
    shape! { InputShare<A> at "series().inputs.by_type.input_share" {
        p2pk65: A = "p2pk65_*",
        p2pk33: A = "p2pk33_*",
        p2pkh: A = "p2pkh_*",
        p2ms: A = "p2ms_*",
        p2sh: A = "p2sh_*",
        p2wpkh: A = "p2wpkh_*",
        p2wsh: A = "p2wsh_*",
        p2tr: A = "p2tr_*",
        p2a: A = "p2a_*",
        unknown: A = "unknown_outputs_*",
        empty: A = "empty_outputs_*",
    } }
    shape! { RealizedPrice at "series().cohorts.realized.price" {
        utxo_amount: UtxoAmount<Spot<Option<SatsFract>>> = "*",
        type_: InputShare<Spot<Option<SatsFract>>> = "*",
    } }
    shape! { TransferVolume at "series().cohorts.activity.transfer_volume" {
        age: Matured<Coinbase<BurnedBlock, Circulating<Sats, Option<Cents>>, Circulating<Option<StoredF32>, Option<StoredF32>>>> = "old_*",
        epoch: CoindaysDestroyedEpoch<Coinbase<BurnedBlock, Circulating<Sats, Option<Cents>>, Circulating<Option<StoredF32>, Option<StoredF32>>>> = "*",
        class: Class<Coinbase<BurnedBlock, Circulating<Sats, Option<Cents>>, Circulating<Option<StoredF32>, Option<StoredF32>>>> = "*",
        in_profit: CoindaysDestroyed<Coinbase<BurnedBlock, Circulating<Sats, Option<Cents>>, Circulating<Option<StoredF32>, Option<StoredF32>>>> = "*_in_profit",
        in_loss: CoindaysDestroyed<Coinbase<BurnedBlock, Circulating<Sats, Option<Cents>>, Circulating<Option<StoredF32>, Option<StoredF32>>>> = "*_in_loss",
        utxo_amount: UtxoAmount<Coinbase<BurnedBlock, Circulating<Sats, Option<Cents>>, Circulating<Option<StoredF32>, Option<StoredF32>>>> = "*",
        type_: InputShare<Coinbase<BurnedBlock, Circulating<Sats, Option<Cents>>, Circulating<Option<StoredF32>, Option<StoredF32>>>> = "*",
    } }
    shape! { CohortsActivity at "series().cohorts.activity" {
        transfer_volume: TransferVolume = "*",
        coindays_destroyed: CoindaysDestroyed<NewAll<Option<StoredF64>>> = "coindays_destroyed",
    } }
    shape! { AvgAmount at "series().cohorts.outputs.avg_amount" {
        all: Circulating<Sats, Option<Cents>> = "*",
        by_type: InputShare<Circulating<Sats, Option<Cents>>> = "*",
    } }
    shape! { SpentCount<A> at "series().cohorts.outputs.spent_count" {
        age: Matured<A> = "old_*",
        epoch: CoindaysDestroyedEpoch<A> = "*",
        class: Class<A> = "*",
        utxo_amount: UtxoAmount<A> = "*",
        type_: InputShare<A> = "*",
    } }
    shape! { CohortsRealized at "series().cohorts.realized" {
        cap: SpentCount<CoinflowCap<Option<Cents>>> = "*_cap",
        profit: SpentCount<RealizedLoss0sats> = "*_profit",
        loss: SpentCount<RealizedLoss0sats> = "*_loss",
        net_pnl: CoindaysDestroyed<Age10yTo12y> = "net_*_pnl",
        value_destroyed: CoindaysDestroyed<Coinbase<RealizedLoss0satsBlock<Option<Cents>>, CoinflowCap<Option<Cents>>, CoinflowCap<Option<StoredF32>>>> = "value_destroyed",
        price: RealizedPrice = "*_price",
    } }
    shape! { CohortsOutputs at "series().cohorts.outputs" {
        unspent_count: SpentCount<UtxoCount0sats> = "*_count",
        spent_count: SpentCount<NewAll<StoredU64>> = "spent_*_count",
        avg_amount: AvgAmount = "avg_*_amount",
    } }
    shape! { CohortsSupply at "series().cohorts.supply" {
        total: SpentCount<Circulating<Sats, Option<Cents>>> = "*",
        matured: Matured<Coinbase<BurnedBlock, Circulating<Sats, Option<Cents>>, Circulating<Option<StoredF32>, Option<StoredF32>>>> = "old_matured_*",
        in_profit: CoindaysDestroyed<Circulating<Sats, Option<Cents>>> = "*_in_profit",
        in_loss: CoindaysDestroyed<Circulating<Sats, Option<Cents>>> = "*_in_loss",
        delta: SpentCount<DeltaAll<Sd24h<Absolute1m>>> = "*_delta",
        dominance: SpentCount<Gini<Option<PartsPerMillion32>>> = "*_dominance",
    } }
    shape! { Cohorts at "series().cohorts" {
        supply: CohortsSupply = "*",
        outputs: CohortsOutputs = "utxo",
        activity: CohortsActivity = "transfer_volume",
        realized: CohortsRealized = "realized",
        unrealized: CohortsUnrealized = "unrealized",
        urpd: CohortsUrpd = "utxos_urpd",
    } }
    shape! { InputsByType at "series().inputs.by_type" {
        input_count: InputCount = "*",
        input_share: InputShare<FeeShare> = "prevout_share",
        tx_count: InputsByTypeTxCount = "tx_*",
        tx_share: InputsByTypeTxShare = "tx_share_with",
    } }
    shape! { Inputs at "series().inputs" {
        raw: InputsRaw = "index",
        value: SeriesPattern23<Sats> = "value",
        count: InputsCount = "input_*",
        per_sec: PerSec<Option<StoredF32>> = "inputs_per_sec",
        by_type: InputsByType = "*",
    } }
    shape! { Volume at "series().transactions.volume" {
        transfer_volume: Coinbase<BurnedBlock, Circulating<Sats, Option<Cents>>, Circulating<Option<StoredF32>, Option<StoredF32>>> = "*",
        tx_per_sec: PerSec<Option<StoredF32>> = "tx_per_sec",
    } }
    shape! { Inscription at "series().transactions.inscription" {
        count: NewAll<StoredU64> = "tx_count_*",
        fees: NewAll<Sats> = "*_fees",
        fee_share: Gini<Option<PartsPerMillion32>> = "*_fee_share",
    } }
    shape! { Transactions at "series().transactions" {
        raw: TransactionsRaw = "*",
        features: Features = "has",
        count: OutputsCount<Vbytes> = "*_count",
        size: TransactionsSize = "*",
        fees: TransactionsFees = "fee",
        inscription: Inscription = "inscription",
        patterns: Patterns = "is",
        policy: Policy = "nonstandard",
        sigops: OutputsCount<NewAll<StoredU64>> = "total_sigop_cost",
        versions: Versions = "*",
        volume: Volume = "transfer_volume_bis",
    } }
    shape! { Difficulty at "series().blocks.difficulty" {
        value: SeriesPattern2<Option<StoredF64>> = "*",
        hashrate: SeriesPattern2<Option<StoredF64>> = "*_hashrate",
        adjustment: Gini<Option<PartsPerMillionSigned32>> = "*_adjustment",
        epoch: SeriesPattern2<Epoch> = "*_epoch",
        blocks_to_retarget: SeriesPattern2<StoredU32> = "blocks_to_retarget",
        days_to_retarget: SeriesPattern2<Option<StoredF32>> = "days_to_retarget",
    } }
    shape! { Blocks at "series().blocks" {
        blockhash: SeriesPattern21<BlockHash> = "blockhash",
        coinbase_tag: SeriesPattern21<CoinbaseTag> = "coinbase_tag",
        difficulty: Difficulty = "difficulty",
        time: Time = "timestamp",
        size: BlocksSize = "size",
        weight: BlocksWeight = "*_weight",
        segwit_txs: SeriesPattern21<StoredU32> = "segwit_txs",
        segwit_size: SeriesPattern21<StoredU64> = "segwit_size",
        segwit_weight: SeriesPattern21<Weight> = "segwit_weight",
        count: BlocksCount = "*_count",
        lookback: BlocksLookback = "height",
        interval: Interval<Timestamp> = "*_interval",
        vbytes: Vbytes = "*_vbytes",
        fullness: Fullness = "*_fullness",
        halving: BlocksHalving = "halving",
    } }
    shape! { SeriesTree at "series()" {
        blocks: Blocks = "block",
        transactions: Transactions = "tx",
        inputs: Inputs = "count",
        outputs: Outputs = "output",
        addrs: Addrs = "addr",
        scripts: Scripts = "index",
        op_return: OpReturn = "op_return",
        mining: Mining = "coinbase",
        cointime: Cointime = "awake",
        coinflow: Coinflow = "coinflow",
        bedrock: Bedrock = "bedrock",
        capital_sentiment: CapitalSentiment = "capital_sentiment",
        rarity_meter: RarityMeter = "rarity_meter",
        constants: Constants = "constant",
        mappings: Mappings = "date",
        indicators: Indicators = "destroyed_supply_adj",
        market: Market = "price",
        pools: Pools = "pool",
        price: Price = "price",
        cohorts: Cohorts = "supply",
        distribution_aggregated: DistributionAggregated = "under",
        supply: Supply = "supply",
        utxo_history: UtxoHistory = "unspent_sats",
    } }
}
pub use tree::SeriesTree;
fn create_series_tree(client: Arc<BitviewClientBase>) -> SeriesTree {
    SeriesTree::build(client, String::new())
}
/// Main Bitview client with series tree and API methods.
pub struct BitviewClient {
    base: Arc<BitviewClientBase>,
    series: OnceLock<Box<SeriesTree>>,
}

impl BitviewClient {
    /// Client version.
    pub const VERSION: &'static str = "v0.12.2";

    /// Create a new client with the given base URL.
    pub fn new(base_url: impl Into<String>) -> Self {
        let base = Arc::new(BitviewClientBase::new(base_url));
        Self {
            base,
            series: OnceLock::new(),
        }
    }

    /// Create a new client with options.
    pub fn with_options(options: BitviewClientOptions) -> Self {
        let base = Arc::new(BitviewClientBase::with_options(options));
        Self {
            base,
            series: OnceLock::new(),
        }
    }

    /// Get the series tree for navigating series.
    pub fn series(&self) -> &SeriesTree {
        self.series
            .get_or_init(|| Box::new(create_series_tree(self.base.clone())))
    }

    /// Create a dynamic series endpoint builder for any series/index combination.
    ///
    /// Use this for programmatic access when the series name is determined at runtime.
    /// For type-safe access, use the `series()` tree instead.
    ///
    /// # Example
    /// ```ignore
    /// let data = client.series_endpoint("realized_price", Index::Height)
    ///     .last(10)
    ///     .fetch()?;
    /// ```
    pub fn series_endpoint(
        &self,
        series: impl Into<SeriesName>,
        index: Index,
    ) -> SeriesEndpoint<serde_json::Value> {
        SeriesEndpoint::new(self.base.clone(), Arc::from(series.into().as_str()), index)
    }

    /// Create a dynamic date-based series endpoint builder.
    ///
    /// Returns `Err` if the index is not date-based.
    pub fn date_series_endpoint(
        &self,
        series: impl Into<SeriesName>,
        index: Index,
    ) -> Result<DateSeriesEndpoint<serde_json::Value>> {
        if !index.is_date_based() {
            return Err(BitviewError::new(format!(
                "{} is not a date-based index",
                index.name()
            )));
        }
        Ok(DateSeriesEndpoint::new(
            self.base.clone(),
            Arc::from(series.into().as_str()),
            index,
        ))
    }

    /// Fetch address hash-prefix matches from raw payload bytes matching `addr_type` length.
    pub fn get_address_payload_hash_prefix_matches(
        &self,
        addr_type: OutputType,
        payload: &[u8],
        nibbles: usize,
    ) -> Result<AddrHashPrefixMatches> {
        validate_address_payload_for_type(addr_type, payload)?;
        let prefix = address_payload_hash_prefix(payload, nibbles)?;
        self.get_address_hash_prefix_matches(addr_type, &prefix)
    }

    /// Fetch address hash-prefix matches for a mainnet Bitcoin address.
    pub fn get_address_hash_prefix_matches_for_address(
        &self,
        address: &str,
        nibbles: usize,
    ) -> Result<AddrHashPrefixMatches> {
        let hashed = address_hash_prefix(address, nibbles)?;
        self.get_address_hash_prefix_matches(hashed.addr_type, &hashed.prefix)
    }

    /// Health check
    ///
    /// Local health and query-readiness check. Returns server identity, uptime, and a coherent local sync snapshot without a bitcoind round-trip. Reads the published prefix during processing; an empty index waits until the request deadline, then returns 504. Responses are not cached. For chain-tip catch-up, request `GET /api/server/sync`.
    ///
    /// Endpoint: `GET /health`
    pub fn get_health(&self) -> Result<Health> {
        self.base.get_json(&format!("/health"))
    }

    /// API version
    ///
    /// Returns the current version of the API server
    ///
    /// Endpoint: `GET /version`
    pub fn get_version(&self) -> Result<String> {
        self.base.get_json(&format!("/version"))
    }

    /// Sync status
    ///
    /// Returns a coherent local index snapshot and a separately observed Bitcoin Core tip height. The two heights can differ during indexing or a reorg. Conditional requests refresh these observations before validation.
    ///
    /// Endpoint: `GET /api/server/sync`
    pub fn get_sync_status(&self) -> Result<SyncStatus> {
        self.base.get_json(&format!("/api/server/sync"))
    }

    /// Disk usage
    ///
    /// Returns allocated file bytes for BRK and Bitcoin data. Each request scans both trees; these are independent observations, not an atomic filesystem snapshot. Conditional requests validate the newly observed totals. Directory-link cycles and excessive nesting fail without returning partial totals.
    ///
    /// Endpoint: `GET /api/server/disk`
    pub fn get_disk_usage(&self) -> Result<DiskUsage> {
        self.base.get_json(&format!("/api/server/disk"))
    }

    /// Series catalog
    ///
    /// Returns the complete hierarchical catalog of available series organized as a tree structure. Series are grouped by categories and subcategories.
    ///
    /// Endpoint: `GET /api/series`
    pub fn get_series_tree(&self) -> Result<TreeNode> {
        self.base.get_json(&format!("/api/series"))
    }

    /// Series count
    ///
    /// Returns the number of series available per index type.
    ///
    /// Endpoint: `GET /api/series/count`
    pub fn get_series_count(&self) -> Result<DetailedSeriesCount> {
        self.base.get_json(&format!("/api/series/count"))
    }

    /// List available indexes
    ///
    /// Returns all available indexes with their accepted query aliases. Use any alias when querying series.
    ///
    /// Endpoint: `GET /api/series/indexes`
    pub fn get_indexes(&self) -> Result<Vec<IndexInfo>> {
        self.base.get_json(&format!("/api/series/indexes"))
    }

    /// Series list
    ///
    /// Paginated flat list of all available series names. Use `page` query param for pagination.
    ///
    /// Endpoint: `GET /api/series/list`
    pub fn list_series(&self, page: Option<i64>, per_page: Option<i64>) -> Result<PaginatedSeries> {
        let mut query = Vec::new();
        if let Some(v) = page {
            query.push(format!("page={}", v));
        }
        if let Some(v) = per_page {
            query.push(format!("per_page={}", v));
        }
        let query_str = if query.is_empty() {
            String::new()
        } else {
            format!("?{}", query.join("&"))
        };
        let path = format!("/api/series/list{}", query_str);
        self.base.get_json(&path)
    }

    /// Search series
    ///
    /// Search series by name or descriptive terms. Results prioritize whole query words in names, then descriptions, then fuzzy names, then fuzzy descriptions. Word order does not matter. Descriptions provide cohort terminology and formulas. The decoded q parameter is limited to 1024 UTF-8 bytes.
    ///
    /// Endpoint: `GET /api/series/search`
    pub fn search_series(&self, q: SeriesName, limit: Option<Limit>) -> Result<Vec<String>> {
        let mut query = Vec::new();
        query.push(format!("q={}", q));
        if let Some(v) = limit {
            query.push(format!("limit={}", v));
        }
        let query_str = if query.is_empty() {
            String::new()
        } else {
            format!("?{}", query.join("&"))
        };
        let path = format!("/api/series/search{}", query_str);
        self.base.get_json(&path)
    }

    /// Get series info
    ///
    /// Returns the optional description, supported indexes, and value type for the specified series. The decoded series name is limited to 1024 UTF-8 bytes.
    ///
    /// Endpoint: `GET /api/series/{series}`
    pub fn get_series_info(&self, series: SeriesName) -> Result<SeriesInfo> {
        self.base.get_json(&format!("/api/series/{series}"))
    }

    /// Get series data
    ///
    /// Fetch data for a specific series at the given index. Use query parameters to filter by date range and format (json/csv).
    ///
    /// Endpoint: `GET /api/series/{series}/{index}`
    pub fn get_series(
        &self,
        series: SeriesName,
        index: Index,
        start: Option<RangeIndex>,
        end: Option<RangeIndex>,
        limit: Option<Limit>,
        format: Option<Format>,
    ) -> Result<FormatResponse<SeriesData>> {
        let mut query = Vec::new();
        if let Some(v) = start {
            query.push(format!("start={}", v));
        }
        if let Some(v) = end {
            query.push(format!("end={}", v));
        }
        if let Some(v) = limit {
            query.push(format!("limit={}", v));
        }
        if let Some(v) = format {
            query.push(format!("format={}", v));
        }
        let query_str = if query.is_empty() {
            String::new()
        } else {
            format!("?{}", query.join("&"))
        };
        let path = format!("/api/series/{series}/{}{}", index.name(), query_str);
        if format == Some(Format::CSV) {
            self.base.get_text(&path).map(FormatResponse::Csv)
        } else {
            self.base.get_json(&path).map(FormatResponse::Json)
        }
    }

    /// Get raw series data
    ///
    /// Returns just the data array without the SeriesData wrapper. Supports the same range and format parameters as `GET /api/series/{series}/{index}`.
    ///
    /// Endpoint: `GET /api/series/{series}/{index}/data`
    pub fn get_series_data(
        &self,
        series: SeriesName,
        index: Index,
        start: Option<RangeIndex>,
        end: Option<RangeIndex>,
        limit: Option<Limit>,
        format: Option<Format>,
    ) -> Result<FormatResponse<Vec<serde_json::Value>>> {
        let mut query = Vec::new();
        if let Some(v) = start {
            query.push(format!("start={}", v));
        }
        if let Some(v) = end {
            query.push(format!("end={}", v));
        }
        if let Some(v) = limit {
            query.push(format!("limit={}", v));
        }
        if let Some(v) = format {
            query.push(format!("format={}", v));
        }
        let query_str = if query.is_empty() {
            String::new()
        } else {
            format!("?{}", query.join("&"))
        };
        let path = format!("/api/series/{series}/{}/data{}", index.name(), query_str);
        if format == Some(Format::CSV) {
            self.base.get_text(&path).map(FormatResponse::Csv)
        } else {
            self.base.get_json(&path).map(FormatResponse::Json)
        }
    }

    /// Get latest series value
    ///
    /// Returns the single most recent value for a series, unwrapped (not inside a SeriesData object).
    ///
    /// Endpoint: `GET /api/series/{series}/{index}/latest`
    pub fn get_series_latest(&self, series: SeriesName, index: Index) -> Result<serde_json::Value> {
        self.base
            .get_json(&format!("/api/series/{series}/{}/latest", index.name()))
    }

    /// Get series data length
    ///
    /// Returns the total number of data points for a series at the given index.
    ///
    /// Endpoint: `GET /api/series/{series}/{index}/len`
    pub fn get_series_len(&self, series: SeriesName, index: Index) -> Result<i64> {
        self.base
            .get_json(&format!("/api/series/{series}/{}/len", index.name()))
    }

    /// Get series version
    ///
    /// Returns the vector's schema/computation version, not its length or latest update. Appends and reorgs do not by themselves change this version.
    ///
    /// Endpoint: `GET /api/series/{series}/{index}/version`
    pub fn get_series_version(&self, series: SeriesName, index: Index) -> Result<u32> {
        self.base
            .get_json(&format!("/api/series/{series}/{}/version", index.name()))
    }

    /// Bulk series data
    ///
    /// Fetch multiple series in a single request. Supports filtering by index and date range. Returns an array of SeriesData objects. For a single series, use `get_series` instead.
    ///
    /// Endpoint: `GET /api/series/bulk`
    pub fn get_series_bulk(
        &self,
        series: SeriesList,
        index: Index,
        start: Option<RangeIndex>,
        end: Option<RangeIndex>,
        limit: Option<Limit>,
        format: Option<Format>,
    ) -> Result<FormatResponse<Vec<SeriesData>>> {
        let mut query = Vec::new();
        query.push(format!("series={}", series));
        query.push(format!("index={}", index));
        if let Some(v) = start {
            query.push(format!("start={}", v));
        }
        if let Some(v) = end {
            query.push(format!("end={}", v));
        }
        if let Some(v) = limit {
            query.push(format!("limit={}", v));
        }
        if let Some(v) = format {
            query.push(format!("format={}", v));
        }
        let query_str = if query.is_empty() {
            String::new()
        } else {
            format!("?{}", query.join("&"))
        };
        let path = format!("/api/series/bulk{}", query_str);
        if format == Some(Format::CSV) {
            self.base.get_text(&path).map(FormatResponse::Csv)
        } else {
            self.base.get_json(&path).map(FormatResponse::Json)
        }
    }

    /// Available URPD cohorts
    ///
    /// Cohorts for which URPD data is available. Returns names like `all`, `sth`, `lth`, `under_4m`, `under_6m`, `over_4m`, `over_6m`, `utxos_under_1h_old`.
    ///
    /// Endpoint: `GET /api/urpd`
    pub fn list_urpd_cohorts(&self) -> Result<Vec<Cohort>> {
        self.base.get_json(&format!("/api/urpd"))
    }

    /// Available URPD dates
    ///
    /// Dates for which a published block is available for the cohort and selected `weight`. One entry per UTC day, sorted ascending.
    ///
    /// Endpoint: `GET /api/urpd/{cohort}/dates`
    pub fn list_urpd_dates(&self, cohort: Cohort, weight: Option<UrpdWeight>) -> Result<Vec<Date>> {
        let mut query = Vec::new();
        if let Some(v) = weight {
            query.push(format!("weight={}", v));
        }
        let query_str = if query.is_empty() {
            String::new()
        } else {
            format!("?{}", query.join("&"))
        };
        let path = format!("/api/urpd/{cohort}/dates{}", query_str);
        self.base.get_json(&path)
    }

    /// Latest URPD
    ///
    /// URPD for the latest published block. The response's `date` field echoes which date was served. Returns `{ cohort, height, date, weight, aggregation, close, total_supply, buckets }`. `close` and each bucket's `price_floor`, `realized_cap`, and `unrealized_pnl` are USD; `total_supply` and bucket `supply` are BTC. `unrealized_pnl` can be negative.
    ///
    /// Endpoint: `GET /api/urpd/{cohort}`
    pub fn get_urpd(
        &self,
        cohort: Cohort,
        agg: Option<UrpdAggregation>,
        weight: Option<UrpdWeight>,
    ) -> Result<Urpd> {
        let mut query = Vec::new();
        if let Some(v) = agg {
            query.push(format!("agg={}", v));
        }
        if let Some(v) = weight {
            query.push(format!("weight={}", v));
        }
        let query_str = if query.is_empty() {
            String::new()
        } else {
            format!("?{}", query.join("&"))
        };
        let path = format!("/api/urpd/{cohort}{}", query_str);
        self.base.get_json(&path)
    }

    /// URPD at block height or date
    ///
    /// URPD for a cohort at a block height or the last block of a UTC day. Returns `{ cohort, height, date, weight, aggregation, close, total_supply, buckets }` where each bucket is `{ price_floor, supply, realized_cap, unrealized_pnl }`. `close`, `price_floor`, `realized_cap`, and `unrealized_pnl` are USD; `total_supply` and `supply` are BTC. `unrealized_pnl` can be negative.
    ///
    /// Endpoint: `GET /api/urpd/{cohort}/{point}`
    pub fn get_urpd_at(
        &self,
        cohort: Cohort,
        point: &str,
        agg: Option<UrpdAggregation>,
        weight: Option<UrpdWeight>,
    ) -> Result<Urpd> {
        let mut query = Vec::new();
        if let Some(v) = agg {
            query.push(format!("agg={}", v));
        }
        if let Some(v) = weight {
            query.push(format!("weight={}", v));
        }
        let query_str = if query.is_empty() {
            String::new()
        } else {
            format!("?{}", query.join("&"))
        };
        let path = format!("/api/urpd/{cohort}/{point}{}", query_str);
        self.base.get_json(&path)
    }

    /// Difficulty adjustment
    ///
    /// Get current difficulty adjustment progress and estimates.
    ///
    /// *[Mempool.space docs](https://mempool.space/docs/api/rest#get-difficulty-adjustment)*
    ///
    /// Endpoint: `GET /api/v1/difficulty-adjustment`
    pub fn get_difficulty_adjustment(&self) -> Result<DifficultyAdjustment> {
        self.base
            .get_json(&format!("/api/v1/difficulty-adjustment"))
    }

    /// Current BTC price
    ///
    /// Returns bitcoin latest price (on-chain derived, USD only).
    ///
    /// *[Mempool.space docs](https://mempool.space/docs/api/rest#get-price)*
    ///
    /// Endpoint: `GET /api/v1/prices`
    pub fn get_prices(&self) -> Result<Prices> {
        self.base.get_json(&format!("/api/v1/prices"))
    }

    /// Historical price
    ///
    /// Completed four-hour BTC/USD closes, oldest first, labeled by interval end. With a UNIX timestamp, returns the latest nonempty completed close at or before it; before the first close returns an empty list. The current partial interval is excluded. USD only; exchangeRates is empty.
    ///
    /// *[Mempool.space docs](https://mempool.space/docs/api/rest#get-historical-price)*
    ///
    /// Endpoint: `GET /api/v1/historical-price`
    pub fn get_historical_price(&self, timestamp: Option<Timestamp>) -> Result<HistoricalPrice> {
        let mut query = Vec::new();
        if let Some(v) = timestamp {
            query.push(format!("timestamp={}", v));
        }
        let query_str = if query.is_empty() {
            String::new()
        } else {
            format!("?{}", query.join("&"))
        };
        let path = format!("/api/v1/historical-price{}", query_str);
        self.base.get_json(&path)
    }

    /// Address hash-prefix matches
    ///
    /// Find addresses by address type and by the first 1-16 hex nibbles of RapidHash v3 over the raw address payload bytes. Intended for privacy-preserving client-side wallet discovery without sending raw addresses or xpubs. Fetch metadata with `GET /api/address/{address}`.
    ///
    /// Endpoint: `GET /api/address/hash-prefix/{addr_type}/{prefix}`
    pub fn get_address_hash_prefix_matches(
        &self,
        addr_type: OutputType,
        prefix: &str,
    ) -> Result<AddrHashPrefixMatches> {
        self.base
            .get_json(&format!("/api/address/hash-prefix/{addr_type}/{prefix}"))
    }

    /// Address information
    ///
    /// Retrieve address information including current balance and transaction counts. Supports all standard Bitcoin address types (P2PKH, P2SH, P2WPKH, P2WSH, P2TR).
    ///
    /// *[Mempool.space docs](https://mempool.space/docs/api/rest#get-address)*
    ///
    /// Endpoint: `GET /api/address/{address}`
    pub fn get_address(&self, address: Addr) -> Result<AddrStats> {
        self.base.get_json(&format!("/api/address/{address}"))
    }

    /// Address transactions
    ///
    /// Get transaction history for an address, newest first. Returns up to 50 mempool transactions plus a confirmed page sized to fill the response to 50 total (chain floor of 25, so 25-50 confirmed depending on mempool weight). To paginate further confirmed history, request `GET /api/address/{address}/txs/chain/{after_txid}` with the last returned txid.
    ///
    /// *[Mempool.space docs](https://mempool.space/docs/api/rest#get-address-transactions)*
    ///
    /// Endpoint: `GET /api/address/{address}/txs`
    pub fn get_address_txs(&self, address: Addr) -> Result<Vec<Transaction>> {
        self.base.get_json(&format!("/api/address/{address}/txs"))
    }

    /// Address confirmed transactions
    ///
    /// Get the first 25 confirmed transactions for an address. For pagination, request `GET /api/address/{address}/txs/chain/{after_txid}` with the last returned txid.
    ///
    /// *[Mempool.space docs](https://mempool.space/docs/api/rest#get-address-transactions-chain)*
    ///
    /// Endpoint: `GET /api/address/{address}/txs/chain`
    pub fn get_address_confirmed_txs(&self, address: Addr) -> Result<Vec<Transaction>> {
        self.base
            .get_json(&format!("/api/address/{address}/txs/chain"))
    }

    /// Address confirmed transactions (paginated)
    ///
    /// Get the next 25 confirmed transactions strictly older than `after_txid` (Esplora-canonical pagination form, matches mempool.space).
    ///
    /// *[Mempool.space docs](https://mempool.space/docs/api/rest#get-address-transactions-chain)*
    ///
    /// Endpoint: `GET /api/address/{address}/txs/chain/{after_txid}`
    pub fn get_address_confirmed_txs_after(
        &self,
        address: Addr,
        after_txid: Txid,
    ) -> Result<Vec<Transaction>> {
        self.base
            .get_json(&format!("/api/address/{address}/txs/chain/{after_txid}"))
    }

    /// Address mempool transactions
    ///
    /// Get unconfirmed transactions for an address from the mempool, newest first (up to 50).
    ///
    /// *[Mempool.space docs](https://mempool.space/docs/api/rest#get-address-transactions-mempool)*
    ///
    /// Endpoint: `GET /api/address/{address}/txs/mempool`
    pub fn get_address_mempool_txs(&self, address: Addr) -> Result<Vec<Transaction>> {
        self.base
            .get_json(&format!("/api/address/{address}/txs/mempool"))
    }

    /// Address UTXOs
    ///
    /// Get unspent transaction outputs (UTXOs) for an address. Returns txid, vout, value, and confirmation status for each UTXO.
    ///
    /// *[Mempool.space docs](https://mempool.space/docs/api/rest#get-address-utxo)*
    ///
    /// Endpoint: `GET /api/address/{address}/utxo`
    pub fn get_address_utxos(&self, address: Addr) -> Result<Vec<Utxo>> {
        self.base.get_json(&format!("/api/address/{address}/utxo"))
    }

    /// Validate address
    ///
    /// Validate a Bitcoin address and get information about its type and scriptPubKey. Returns `isvalid: false` with an error message for invalid addresses.
    ///
    /// *[Mempool.space docs](https://mempool.space/docs/api/rest#get-address-validate)*
    ///
    /// Endpoint: `GET /api/v1/validate-address/{address}`
    pub fn validate_address(&self, address: &str) -> Result<AddrValidation> {
        self.base
            .get_json(&format!("/api/v1/validate-address/{address}"))
    }

    /// Block information
    ///
    /// Retrieve block information by block hash. Returns block metadata including height, timestamp, difficulty, size, weight, and transaction count.
    ///
    /// *[Mempool.space docs](https://mempool.space/docs/api/rest#get-block)*
    ///
    /// Endpoint: `GET /api/block/{hash}`
    pub fn get_block(&self, hash: BlockHash) -> Result<BlockInfo> {
        self.base.get_json(&format!("/api/block/{hash}"))
    }

    /// Block (v1)
    ///
    /// Returns block details with extras by hash.
    ///
    /// *[Mempool.space docs](https://mempool.space/docs/api/rest#get-block-v1)*
    ///
    /// Endpoint: `GET /api/v1/block/{hash}`
    pub fn get_block_v1(&self, hash: BlockHash) -> Result<BlockInfoV1> {
        self.base.get_json(&format!("/api/v1/block/{hash}"))
    }

    /// Block header
    ///
    /// Returns the hex-encoded 80-byte block header.
    ///
    /// *[Mempool.space docs](https://mempool.space/docs/api/rest#get-block-header)*
    ///
    /// Endpoint: `GET /api/block/{hash}/header`
    pub fn get_block_header(&self, hash: BlockHash) -> Result<String> {
        self.base.get_text(&format!("/api/block/{hash}/header"))
    }

    /// Block hash by height
    ///
    /// Retrieve the block hash at a given height. Returns the hash as plain text.
    ///
    /// *[Mempool.space docs](https://mempool.space/docs/api/rest#get-block-height)*
    ///
    /// Endpoint: `GET /api/block-height/{height}`
    pub fn get_block_by_height(&self, height: Height) -> Result<String> {
        self.base.get_text(&format!("/api/block-height/{height}"))
    }

    /// Block by timestamp
    ///
    /// Find the block with the greatest header timestamp at or before the given UNIX timestamp, choosing the earliest height on ties.
    ///
    /// *[Mempool.space docs](https://mempool.space/docs/api/rest#get-block-timestamp)*
    ///
    /// Endpoint: `GET /api/v1/mining/blocks/timestamp/{timestamp}`
    pub fn get_block_by_timestamp(&self, timestamp: Timestamp) -> Result<BlockTimestamp> {
        self.base
            .get_json(&format!("/api/v1/mining/blocks/timestamp/{timestamp}"))
    }

    /// Raw block
    ///
    /// Returns the raw block data in binary format.
    ///
    /// *[Mempool.space docs](https://mempool.space/docs/api/rest#get-block-raw)*
    ///
    /// Endpoint: `GET /api/block/{hash}/raw`
    pub fn get_block_raw(&self, hash: BlockHash) -> Result<Vec<u8>> {
        self.base.get_bytes(&format!("/api/block/{hash}/raw"))
    }

    /// Block status
    ///
    /// Retrieve the status of a block. Returns whether the block is in the best chain and, if so, its height and the hash of the next block.
    ///
    /// *[Mempool.space docs](https://mempool.space/docs/api/rest#get-block-status)*
    ///
    /// Endpoint: `GET /api/block/{hash}/status`
    pub fn get_block_status(&self, hash: BlockHash) -> Result<BlockStatus> {
        self.base.get_json(&format!("/api/block/{hash}/status"))
    }

    /// Block tip height
    ///
    /// Returns the height of the last block.
    ///
    /// *[Mempool.space docs](https://mempool.space/docs/api/rest#get-block-tip-height)*
    ///
    /// Endpoint: `GET /api/blocks/tip/height`
    pub fn get_block_tip_height(&self) -> Result<String> {
        self.base.get_text(&format!("/api/blocks/tip/height"))
    }

    /// Block tip hash
    ///
    /// Returns the hash of the last block.
    ///
    /// *[Mempool.space docs](https://mempool.space/docs/api/rest#get-block-tip-hash)*
    ///
    /// Endpoint: `GET /api/blocks/tip/hash`
    pub fn get_block_tip_hash(&self) -> Result<String> {
        self.base.get_text(&format!("/api/blocks/tip/hash"))
    }

    /// Transaction ID at index
    ///
    /// Retrieve a single transaction ID at a specific index within a block. Returns plain text txid.
    ///
    /// *[Mempool.space docs](https://mempool.space/docs/api/rest#get-block-transaction-id)*
    ///
    /// Endpoint: `GET /api/block/{hash}/txid/{index}`
    pub fn get_block_txid(&self, hash: BlockHash, index: BlockTxIndex) -> Result<String> {
        self.base
            .get_text(&format!("/api/block/{hash}/txid/{index}"))
    }

    /// Block transaction IDs
    ///
    /// Retrieve all transaction IDs in a block. Returns an array of txids in block order.
    ///
    /// *[Mempool.space docs](https://mempool.space/docs/api/rest#get-block-transaction-ids)*
    ///
    /// Endpoint: `GET /api/block/{hash}/txids`
    pub fn get_block_txids(&self, hash: BlockHash) -> Result<Vec<Txid>> {
        self.base.get_json(&format!("/api/block/{hash}/txids"))
    }

    /// Block transactions
    ///
    /// Retrieve transactions in a block by block hash. Returns up to 25 transactions starting from index 0.
    ///
    /// *[Mempool.space docs](https://mempool.space/docs/api/rest#get-block-transactions)*
    ///
    /// Endpoint: `GET /api/block/{hash}/txs`
    pub fn get_block_txs(&self, hash: BlockHash) -> Result<Vec<Transaction>> {
        self.base.get_json(&format!("/api/block/{hash}/txs"))
    }

    /// Block transactions (paginated)
    ///
    /// Retrieve transactions in a block by block hash, starting from the specified index. Returns up to 25 transactions at a time.
    ///
    /// *[Mempool.space docs](https://mempool.space/docs/api/rest#get-block-transactions)*
    ///
    /// Endpoint: `GET /api/block/{hash}/txs/{start_index}`
    pub fn get_block_txs_from_index(
        &self,
        hash: BlockHash,
        start_index: BlockTxIndex,
    ) -> Result<Vec<Transaction>> {
        self.base
            .get_json(&format!("/api/block/{hash}/txs/{start_index}"))
    }

    /// Recent blocks
    ///
    /// Retrieve the last 10 blocks. Returns block metadata for each block.
    ///
    /// *[Mempool.space docs](https://mempool.space/docs/api/rest#get-blocks)*
    ///
    /// Endpoint: `GET /api/blocks`
    pub fn get_blocks(&self) -> Result<Vec<BlockInfo>> {
        self.base.get_json(&format!("/api/blocks"))
    }

    /// Blocks from height
    ///
    /// Retrieve up to 10 blocks going backwards from the given height. For example, height=100 returns blocks 100, 99, 98, ..., 91. Height=0 returns only block 0.
    ///
    /// *[Mempool.space docs](https://mempool.space/docs/api/rest#get-blocks)*
    ///
    /// Endpoint: `GET /api/blocks/{height}`
    pub fn get_blocks_from_height(&self, height: Height) -> Result<Vec<BlockInfo>> {
        self.base.get_json(&format!("/api/blocks/{height}"))
    }

    /// Recent blocks with extras
    ///
    /// Retrieve the last 15 blocks with extended data including pool identification and fee statistics.
    ///
    /// *[Mempool.space docs](https://mempool.space/docs/api/rest#get-blocks-v1)*
    ///
    /// Endpoint: `GET /api/v1/blocks`
    pub fn get_blocks_v1(&self) -> Result<Vec<BlockInfoV1>> {
        self.base.get_json(&format!("/api/v1/blocks"))
    }

    /// Blocks from height with extras
    ///
    /// Retrieve up to 15 blocks with extended data going backwards from the given height.
    ///
    /// *[Mempool.space docs](https://mempool.space/docs/api/rest#get-blocks-v1)*
    ///
    /// Endpoint: `GET /api/v1/blocks/{height}`
    pub fn get_blocks_v1_from_height(&self, height: Height) -> Result<Vec<BlockInfoV1>> {
        self.base.get_json(&format!("/api/v1/blocks/{height}"))
    }

    /// List all mining pools
    ///
    /// Get list of all known mining pools with their identifiers.
    ///
    /// *[Mempool.space docs](https://mempool.space/docs/api/rest#get-mining-pools)*
    ///
    /// Endpoint: `GET /api/v1/mining/pools`
    pub fn get_pools(&self) -> Result<Vec<PoolInfo>> {
        self.base.get_json(&format!("/api/v1/mining/pools"))
    }

    /// Mining pool statistics
    ///
    /// Get mining pool statistics for a time period. Valid periods: `24h`, `3d`, `1w`, `1m`, `3m`, `6m`, `1y`, `2y`, `3y`.
    ///
    /// *[Mempool.space docs](https://mempool.space/docs/api/rest#get-mining-pools)*
    ///
    /// Endpoint: `GET /api/v1/mining/pools/{time_period}`
    pub fn get_pool_stats(&self, time_period: TimePeriod) -> Result<PoolsSummary> {
        self.base
            .get_json(&format!("/api/v1/mining/pools/{time_period}"))
    }

    /// Mining pool details
    ///
    /// Get detailed information about a specific mining pool including block counts and shares for different time periods.
    ///
    /// *[Mempool.space docs](https://mempool.space/docs/api/rest#get-mining-pool)*
    ///
    /// Endpoint: `GET /api/v1/mining/pool/{slug}`
    pub fn get_pool(&self, slug: PoolSlug) -> Result<PoolDetail> {
        self.base.get_json(&format!("/api/v1/mining/pool/{slug}"))
    }

    /// All pools hashrate (all time)
    ///
    /// Get hashrate data for all mining pools.
    ///
    /// *[Mempool.space docs](https://mempool.space/docs/api/rest#get-mining-pool-hashrates)*
    ///
    /// Endpoint: `GET /api/v1/mining/hashrate/pools`
    pub fn get_pools_hashrate(&self) -> Result<Vec<PoolHashrateEntry>> {
        self.base
            .get_json(&format!("/api/v1/mining/hashrate/pools"))
    }

    /// All pools hashrate
    ///
    /// Get hashrate data for all mining pools for a time period. Valid periods: `1m`, `3m`, `6m`, `1y`, `2y`, `3y`.
    ///
    /// *[Mempool.space docs](https://mempool.space/docs/api/rest#get-mining-pool-hashrates)*
    ///
    /// Endpoint: `GET /api/v1/mining/hashrate/pools/{time_period}`
    pub fn get_pools_hashrate_by_period(
        &self,
        time_period: TimePeriod,
    ) -> Result<Vec<PoolHashrateEntry>> {
        self.base
            .get_json(&format!("/api/v1/mining/hashrate/pools/{time_period}"))
    }

    /// Mining pool hashrate
    ///
    /// Get hashrate history for a specific mining pool.
    ///
    /// *[Mempool.space docs](https://mempool.space/docs/api/rest#get-mining-pool-hashrate)*
    ///
    /// Endpoint: `GET /api/v1/mining/pool/{slug}/hashrate`
    pub fn get_pool_hashrate(&self, slug: PoolSlug) -> Result<Vec<PoolHashrateEntry>> {
        self.base
            .get_json(&format!("/api/v1/mining/pool/{slug}/hashrate"))
    }

    /// Mining pool blocks
    ///
    /// Get up to 100 recent blocks mined by a specific pool.
    ///
    /// *[Mempool.space docs](https://mempool.space/docs/api/rest#get-mining-pool-blocks)*
    ///
    /// Endpoint: `GET /api/v1/mining/pool/{slug}/blocks`
    pub fn get_pool_blocks(&self, slug: PoolSlug) -> Result<Vec<BlockInfoV1>> {
        self.base
            .get_json(&format!("/api/v1/mining/pool/{slug}/blocks"))
    }

    /// Mining pool blocks from height
    ///
    /// Get up to 100 blocks mined by a specific pool before (and including) the given height.
    ///
    /// *[Mempool.space docs](https://mempool.space/docs/api/rest#get-mining-pool-blocks)*
    ///
    /// Endpoint: `GET /api/v1/mining/pool/{slug}/blocks/{height}`
    pub fn get_pool_blocks_from(&self, slug: PoolSlug, height: Height) -> Result<Vec<BlockInfoV1>> {
        self.base
            .get_json(&format!("/api/v1/mining/pool/{slug}/blocks/{height}"))
    }

    /// Network hashrate (all time)
    ///
    /// Get network hashrate and difficulty data for all time.
    ///
    /// *[Mempool.space docs](https://mempool.space/docs/api/rest#get-hashrate)*
    ///
    /// Endpoint: `GET /api/v1/mining/hashrate`
    pub fn get_hashrate(&self) -> Result<HashrateSummary> {
        self.base.get_json(&format!("/api/v1/mining/hashrate"))
    }

    /// Network hashrate
    ///
    /// Get network hashrate and difficulty data for a time period. Valid periods: `24h`, `3d`, `1w`, `1m`, `3m`, `6m`, `1y`, `2y`, `3y`.
    ///
    /// *[Mempool.space docs](https://mempool.space/docs/api/rest#get-hashrate)*
    ///
    /// Endpoint: `GET /api/v1/mining/hashrate/{time_period}`
    pub fn get_hashrate_by_period(&self, time_period: TimePeriod) -> Result<HashrateSummary> {
        self.base
            .get_json(&format!("/api/v1/mining/hashrate/{time_period}"))
    }

    /// Difficulty adjustments (all time)
    ///
    /// Get historical difficulty adjustments including timestamp, block height, difficulty value, and percentage change.
    ///
    /// *[Mempool.space docs](https://mempool.space/docs/api/rest#get-difficulty-adjustments)*
    ///
    /// Endpoint: `GET /api/v1/mining/difficulty-adjustments`
    pub fn get_difficulty_adjustments(&self) -> Result<Vec<DifficultyAdjustmentEntry>> {
        self.base
            .get_json(&format!("/api/v1/mining/difficulty-adjustments"))
    }

    /// Difficulty adjustments
    ///
    /// Get historical difficulty adjustments for a time period. Valid periods: `24h`, `3d`, `1w`, `1m`, `3m`, `6m`, `1y`, `2y`, `3y`.
    ///
    /// *[Mempool.space docs](https://mempool.space/docs/api/rest#get-difficulty-adjustments)*
    ///
    /// Endpoint: `GET /api/v1/mining/difficulty-adjustments/{time_period}`
    pub fn get_difficulty_adjustments_by_period(
        &self,
        time_period: TimePeriod,
    ) -> Result<Vec<DifficultyAdjustmentEntry>> {
        self.base.get_json(&format!(
            "/api/v1/mining/difficulty-adjustments/{time_period}"
        ))
    }

    /// Mining reward statistics
    ///
    /// Get mining reward statistics for the last N blocks including total rewards, fees, and transaction count.
    ///
    /// *[Mempool.space docs](https://mempool.space/docs/api/rest#get-reward-stats)*
    ///
    /// Endpoint: `GET /api/v1/mining/reward-stats/{block_count}`
    pub fn get_reward_stats(&self, block_count: i64) -> Result<RewardStats> {
        self.base
            .get_json(&format!("/api/v1/mining/reward-stats/{block_count}"))
    }

    /// Block fees
    ///
    /// Get average total fees per block for a time period. Valid periods: `24h`, `3d`, `1w`, `1m`, `3m`, `6m`, `1y`, `2y`, `3y`.
    ///
    /// *[Mempool.space docs](https://mempool.space/docs/api/rest#get-block-fees)*
    ///
    /// Endpoint: `GET /api/v1/mining/blocks/fees/{time_period}`
    pub fn get_block_fees(&self, time_period: TimePeriod) -> Result<Vec<BlockFeesEntry>> {
        self.base
            .get_json(&format!("/api/v1/mining/blocks/fees/{time_period}"))
    }

    /// Block rewards
    ///
    /// Get average coinbase reward (subsidy + fees) per block for a time period. Valid periods: `24h`, `3d`, `1w`, `1m`, `3m`, `6m`, `1y`, `2y`, `3y`.
    ///
    /// *[Mempool.space docs](https://mempool.space/docs/api/rest#get-block-rewards)*
    ///
    /// Endpoint: `GET /api/v1/mining/blocks/rewards/{time_period}`
    pub fn get_block_rewards(&self, time_period: TimePeriod) -> Result<Vec<BlockRewardsEntry>> {
        self.base
            .get_json(&format!("/api/v1/mining/blocks/rewards/{time_period}"))
    }

    /// Block fee rates
    ///
    /// Get block fee rate percentiles (min, 10th, 25th, median, 75th, 90th, max) for a time period. Valid periods: `24h`, `3d`, `1w`, `1m`, `3m`, `6m`, `1y`, `2y`, `3y`.
    ///
    /// *[Mempool.space docs](https://mempool.space/docs/api/rest#get-block-feerates)*
    ///
    /// Endpoint: `GET /api/v1/mining/blocks/fee-rates/{time_period}`
    pub fn get_block_fee_rates(&self, time_period: TimePeriod) -> Result<Vec<BlockFeeRatesEntry>> {
        self.base
            .get_json(&format!("/api/v1/mining/blocks/fee-rates/{time_period}"))
    }

    /// Block sizes and weights
    ///
    /// Get average block sizes and weights for a time period. Valid periods: `24h`, `3d`, `1w`, `1m`, `3m`, `6m`, `1y`, `2y`, `3y`.
    ///
    /// *[Mempool.space docs](https://mempool.space/docs/api/rest#get-sizes-weights)*
    ///
    /// Endpoint: `GET /api/v1/mining/blocks/sizes-weights/{time_period}`
    pub fn get_block_sizes_weights(&self, time_period: TimePeriod) -> Result<BlockSizesWeights> {
        self.base.get_json(&format!(
            "/api/v1/mining/blocks/sizes-weights/{time_period}"
        ))
    }

    /// Projected mempool blocks
    ///
    /// Projected blocks for fee estimation. Block 0 reflects Bitcoin Core's actual next-block selection; blocks 1+ are a fee-tier approximation.
    ///
    /// *[Mempool.space docs](https://mempool.space/docs/api/rest#get-mempool-blocks-fees)*
    ///
    /// Endpoint: `GET /api/v1/fees/mempool-blocks`
    pub fn get_mempool_blocks(&self) -> Result<Vec<MempoolBlock>> {
        self.base.get_json(&format!("/api/v1/fees/mempool-blocks"))
    }

    /// Recommended fees
    ///
    /// Recommended fee rates by confirmation target.
    ///
    /// *[Mempool.space docs](https://mempool.space/docs/api/rest#get-recommended-fees)*
    ///
    /// Endpoint: `GET /api/v1/fees/recommended`
    pub fn get_recommended_fees(&self) -> Result<RecommendedFees> {
        self.base.get_json(&format!("/api/v1/fees/recommended"))
    }

    /// Recommended fee rates (precise)
    ///
    /// Recommended fee rates by confirmation target, with up to three decimal places and support for sub-sat/vB rates.
    ///
    /// *[Mempool.space docs](https://mempool.space/docs/api/rest#get-recommended-fees-precise)*
    ///
    /// Endpoint: `GET /api/v1/fees/precise`
    pub fn get_precise_fees(&self) -> Result<RecommendedFees> {
        self.base.get_json(&format!("/api/v1/fees/precise"))
    }

    /// Mempool statistics
    ///
    /// Get current mempool statistics including transaction count, total vsize, total fees, and fee histogram.
    ///
    /// *[Mempool.space docs](https://mempool.space/docs/api/rest#get-mempool)*
    ///
    /// Endpoint: `GET /api/mempool`
    pub fn get_mempool(&self) -> Result<MempoolInfo> {
        self.base.get_json(&format!("/api/mempool"))
    }

    /// Mempool content hash
    ///
    /// Returns an opaque content token for the published projected next block, including statistics and transaction bodies. This is not the HTTP ETag. An unchanged token means unchanged content, not necessarily a stalled sync loop.
    ///
    /// Endpoint: `GET /api/mempool/hash`
    pub fn get_mempool_hash(&self) -> Result<NextBlockHash> {
        self.base.get_json(&format!("/api/mempool/hash"))
    }

    /// Mempool transaction IDs
    ///
    /// Get all transaction IDs currently in the mempool.
    ///
    /// *[Mempool.space docs](https://mempool.space/docs/api/rest#get-mempool-transaction-ids)*
    ///
    /// Endpoint: `GET /api/mempool/txids`
    pub fn get_mempool_txids(&self) -> Result<Vec<Txid>> {
        self.base.get_json(&format!("/api/mempool/txids"))
    }

    /// Recent mempool transactions
    ///
    /// Get the last 10 transactions to enter the mempool.
    ///
    /// *[Mempool.space docs](https://mempool.space/docs/api/rest#get-mempool-recent)*
    ///
    /// Endpoint: `GET /api/mempool/recent`
    pub fn get_mempool_recent(&self) -> Result<Vec<MempoolRecentTx>> {
        self.base.get_json(&format!("/api/mempool/recent"))
    }

    /// Recent RBF replacements
    ///
    /// Returns up to 25 most-recent RBF replacement trees across the whole mempool. Each entry has the same shape as `tx_rbf().replacements`.
    ///
    /// *[Mempool.space docs](https://mempool.space/docs/api/rest#get-replacements)*
    ///
    /// Endpoint: `GET /api/v1/replacements`
    pub fn get_replacements(&self) -> Result<Vec<ReplacementNode>> {
        self.base.get_json(&format!("/api/v1/replacements"))
    }

    /// Recent full-RBF replacements
    ///
    /// Same response shape as `GET /api/v1/replacements`, but limited to trees where at least one predecessor was non-signaling (full-RBF).
    ///
    /// *[Mempool.space docs](https://mempool.space/docs/api/rest#get-fullrbf-replacements)*
    ///
    /// Endpoint: `GET /api/v1/fullrbf/replacements`
    pub fn get_fullrbf_replacements(&self) -> Result<Vec<ReplacementNode>> {
        self.base.get_json(&format!("/api/v1/fullrbf/replacements"))
    }

    /// Projected next block template
    ///
    /// Bitcoin Core's `getblocktemplate` selection: full transaction bodies in GBT order with aggregate stats. The returned `hash` is an opaque content token; pass it to `GET /api/v1/mempool/block-template/diff/{hash}` to fetch deltas instead of refetching the whole template.
    ///
    /// Endpoint: `GET /api/v1/mempool/block-template`
    pub fn get_block_template(&self) -> Result<BlockTemplate> {
        self.base
            .get_json(&format!("/api/v1/mempool/block-template"))
    }

    /// Block template diff since hash
    ///
    /// Delta of the projected next block since `<hash>`. `order` is the full new template in order: each entry is either a number (index into the prior template the client cached at `<hash>`) or a transaction object (new body to insert at this position). Walk `order` once to rebuild; `removed` is a convenience list of txids that left so clients can evict cached bodies. After applying, use the response `hash` as `<hash>` on the next call to keep iterating. Returns `404` when `<hash>` has aged out of server history; clients should fall back to `GET /api/v1/mempool/block-template`.
    ///
    /// Endpoint: `GET /api/v1/mempool/block-template/diff/{hash}`
    pub fn get_block_template_diff(&self, hash: NextBlockHash) -> Result<BlockTemplateDiff> {
        self.base
            .get_json(&format!("/api/v1/mempool/block-template/diff/{hash}"))
    }

    /// Live BTC/USD price
    ///
    /// Returns the current BTC/USD price in dollars, derived from on-chain round-dollar output patterns in the last 12 blocks plus mempool.
    ///
    /// Endpoint: `GET /api/mempool/price`
    pub fn get_live_price(&self) -> Result<Dollars> {
        self.base.get_json(&format!("/api/mempool/price"))
    }

    /// Txid by index
    ///
    /// Retrieve the transaction ID (txid) at a given global transaction index. Returns the txid as plain text.
    ///
    /// Endpoint: `GET /api/tx-index/{index}`
    pub fn get_tx_by_index(&self, index: TxIndex) -> Result<String> {
        self.base.get_text(&format!("/api/tx-index/{index}"))
    }

    /// CPFP info
    ///
    /// Returns ancestors and descendants for a CPFP (Child Pays For Parent) transaction, including the effective fee rate of the package.
    ///
    /// *[Mempool.space docs](https://mempool.space/docs/api/rest#get-children-pay-for-parent)*
    ///
    /// Endpoint: `GET /api/v1/cpfp/{txid}`
    pub fn get_cpfp(&self, txid: Txid) -> Result<CpfpInfo> {
        self.base.get_json(&format!("/api/v1/cpfp/{txid}"))
    }

    /// RBF replacement history
    ///
    /// Returns the RBF replacement tree for a transaction, if any. Both `replacements` and `replaces` are null when the tx has no known RBF history within the mempool monitor's retention window.
    ///
    /// *[Mempool.space docs](https://mempool.space/docs/api/rest#get-transaction-rbf-history)*
    ///
    /// Endpoint: `GET /api/v1/tx/{txid}/rbf`
    pub fn get_tx_rbf(&self, txid: Txid) -> Result<RbfResponse> {
        self.base.get_json(&format!("/api/v1/tx/{txid}/rbf"))
    }

    /// Transaction information
    ///
    /// Retrieve complete transaction data by transaction ID (txid). Returns inputs, outputs, fee, size, and confirmation status.
    ///
    /// *[Mempool.space docs](https://mempool.space/docs/api/rest#get-transaction)*
    ///
    /// Endpoint: `GET /api/tx/{txid}`
    pub fn get_tx(&self, txid: Txid) -> Result<Transaction> {
        self.base.get_json(&format!("/api/tx/{txid}"))
    }

    /// Transaction hex
    ///
    /// Retrieve the raw transaction as a hex-encoded string. Returns the serialized transaction in hexadecimal format.
    ///
    /// *[Mempool.space docs](https://mempool.space/docs/api/rest#get-transaction-hex)*
    ///
    /// Endpoint: `GET /api/tx/{txid}/hex`
    pub fn get_tx_hex(&self, txid: Txid) -> Result<String> {
        self.base.get_text(&format!("/api/tx/{txid}/hex"))
    }

    /// Transaction merkleblock proof
    ///
    /// Get the merkleblock proof for a transaction (BIP37 format, hex encoded).
    ///
    /// *[Mempool.space docs](https://mempool.space/docs/api/rest#get-transaction-merkleblock-proof)*
    ///
    /// Endpoint: `GET /api/tx/{txid}/merkleblock-proof`
    pub fn get_tx_merkleblock_proof(&self, txid: Txid) -> Result<String> {
        self.base
            .get_text(&format!("/api/tx/{txid}/merkleblock-proof"))
    }

    /// Transaction merkle proof
    ///
    /// Get the merkle inclusion proof for a transaction.
    ///
    /// *[Mempool.space docs](https://mempool.space/docs/api/rest#get-transaction-merkle-proof)*
    ///
    /// Endpoint: `GET /api/tx/{txid}/merkle-proof`
    pub fn get_tx_merkle_proof(&self, txid: Txid) -> Result<MerkleProof> {
        self.base.get_json(&format!("/api/tx/{txid}/merkle-proof"))
    }

    /// Output spend status
    ///
    /// Get the spending status of a transaction output. Returns whether the output has been spent and, if so, the spending transaction details.
    ///
    /// *[Mempool.space docs](https://mempool.space/docs/api/rest#get-transaction-outspend)*
    ///
    /// Endpoint: `GET /api/tx/{txid}/outspend/{vout}`
    pub fn get_tx_outspend(&self, txid: Txid, vout: Vout) -> Result<TxOutspend> {
        self.base
            .get_json(&format!("/api/tx/{txid}/outspend/{vout}"))
    }

    /// All output spend statuses
    ///
    /// Get the spending status of all outputs in a transaction. Returns an array with the spend status for each output.
    ///
    /// *[Mempool.space docs](https://mempool.space/docs/api/rest#get-transaction-outspends)*
    ///
    /// Endpoint: `GET /api/tx/{txid}/outspends`
    pub fn get_tx_outspends(&self, txid: Txid) -> Result<Vec<TxOutspend>> {
        self.base.get_json(&format!("/api/tx/{txid}/outspends"))
    }

    /// Transaction raw
    ///
    /// Returns a transaction as binary data.
    ///
    /// *[Mempool.space docs](https://mempool.space/docs/api/rest#get-transaction-raw)*
    ///
    /// Endpoint: `GET /api/tx/{txid}/raw`
    pub fn get_tx_raw(&self, txid: Txid) -> Result<Vec<u8>> {
        self.base.get_bytes(&format!("/api/tx/{txid}/raw"))
    }

    /// Transaction status
    ///
    /// Retrieve the confirmation status of a transaction. Returns whether the transaction is confirmed and, if so, the block height, hash, and timestamp.
    ///
    /// *[Mempool.space docs](https://mempool.space/docs/api/rest#get-transaction-status)*
    ///
    /// Endpoint: `GET /api/tx/{txid}/status`
    pub fn get_tx_status(&self, txid: Txid) -> Result<TxStatus> {
        self.base.get_json(&format!("/api/tx/{txid}/status"))
    }

    /// Transaction first-seen times
    ///
    /// Returns timestamps when transactions were first seen in the mempool. Returns 0 for mined or unknown transactions.
    ///
    /// *[Mempool.space docs](https://mempool.space/docs/api/rest#get-transaction-times)*
    ///
    /// Endpoint: `GET /api/v1/transaction-times`
    pub fn get_transaction_times(&self, txId: &[Txid]) -> Result<Vec<i64>> {
        let mut query = Vec::new();
        for v in txId {
            query.push(format!("txId[]={}", v));
        }
        let query_str = if query.is_empty() {
            String::new()
        } else {
            format!("?{}", query.join("&"))
        };
        let path = format!("/api/v1/transaction-times{}", query_str);
        self.base.get_json(&path)
    }

    /// Broadcast transaction
    ///
    /// Submit a raw transaction as hexadecimal text (at most 8,000,000 request bytes, including whitespace). Returns its txid as plain text. No responses are cached. Cancellation or a transport error after dispatch may leave the submission outcome unknown; do not automatically retry.
    ///
    /// *[Mempool.space docs](https://mempool.space/docs/api/rest#post-transaction)*
    ///
    /// Endpoint: `POST /api/tx`
    pub fn post_tx(&self, body: &str) -> Result<Txid> {
        self.base
            .post_text(&format!("/api/tx"), body)?
            .parse::<Txid>()
            .map_err(|error| {
                BitviewError::new(format!(
                    "Invalid submission response; outcome may be unknown: {error}"
                ))
            })
    }

    /// Live BTC/USD price
    ///
    /// Current BTC/USD price in dollars. Same value as `GET /api/mempool/price`. Confirmed per-height history is available at `GET /api/series/price/height`.
    ///
    /// Endpoint: `GET /api/oracle/price`
    pub fn get_oracle_price(&self) -> Result<Dollars> {
        self.base.get_json(&format!("/api/oracle/price"))
    }

    /// Live payment output histogram
    ///
    /// Live smoothed histogram of oracle-eligible payment outputs, binned by output value on the oracle log scale. It combines the committed oracle window with the complete mempool's eligible outputs from a matching chain publication. A flat array of log-scale bins.
    ///
    /// Endpoint: `GET /api/oracle/histogram/payments/live`
    pub fn get_oracle_histogram_payments_live(&self) -> Result<Vec<i64>> {
        self.base
            .get_json(&format!("/api/oracle/histogram/payments/live"))
    }

    /// Payment output histogram at height or day
    ///
    /// Smoothed histogram of oracle-eligible payment outputs for a confirmed point. A block height (`840000`) gives that block's oracle payment histogram; a calendar date (`YYYY-MM-DD`) gives the average of that day's per-block payment histograms. A flat array of log-scale bins.
    ///
    /// Endpoint: `GET /api/oracle/histogram/payments/{point}`
    pub fn get_oracle_histogram_payments(&self, point: &str) -> Result<Vec<i64>> {
        self.base
            .get_json(&format!("/api/oracle/histogram/payments/{point}"))
    }

    /// Live output value histogram
    ///
    /// Live unfiltered output value histogram for the complete published mempool. Every live output is binned by value on the oracle log scale; no oracle payment filters are applied. A flat array of log-scale bins, all zero when no mempool is configured.
    ///
    /// Endpoint: `GET /api/oracle/histogram/outputs/live`
    pub fn get_oracle_histogram_outputs_live(&self) -> Result<Vec<i64>> {
        self.base
            .get_json(&format!("/api/oracle/histogram/outputs/live"))
    }

    /// Output value histogram at height or day
    ///
    /// Unfiltered output value histogram for a confirmed point. A block height (`840000`) gives every output in that block, coinbase included, binned by value on the oracle log scale; a calendar date (`YYYY-MM-DD`) sums every block that day. A flat array of log-scale bins.
    ///
    /// Endpoint: `GET /api/oracle/histogram/outputs/{point}`
    pub fn get_oracle_histogram_outputs(&self, point: &str) -> Result<Vec<i64>> {
        self.base
            .get_json(&format!("/api/oracle/histogram/outputs/{point}"))
    }

    /// OpenAPI specification
    ///
    /// Full OpenAPI 3.1 specification for this API.
    ///
    /// Endpoint: `GET /openapi.json`
    pub fn get_openapi(&self) -> Result<String> {
        self.base.get_text(&format!("/openapi.json"))
    }

    /// Compact OpenAPI specification
    ///
    /// Compact OpenAPI specification optimized for LLM consumption. Removes redundant fields while preserving essential API information. The full specification is available at `GET /openapi.json`.
    ///
    /// Endpoint: `GET /api.json`
    pub fn get_api(&self) -> Result<serde_json::Value> {
        self.base.get_json(&format!("/api.json"))
    }
}
