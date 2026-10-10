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
const _I5: &[Index] = &[Index::Minute10];
const _I6: &[Index] = &[Index::Minute30];
const _I7: &[Index] = &[Index::Hour1];
const _I8: &[Index] = &[Index::Hour4];
const _I9: &[Index] = &[Index::Hour12];
const _I10: &[Index] = &[Index::Day1];
const _I11: &[Index] = &[Index::Day3];
const _I12: &[Index] = &[Index::Week1];
const _I13: &[Index] = &[Index::Month1];
const _I14: &[Index] = &[Index::Month3];
const _I15: &[Index] = &[Index::Month6];
const _I16: &[Index] = &[Index::Year1];
const _I17: &[Index] = &[Index::Year10];
const _I18: &[Index] = &[Index::Halving];
const _I19: &[Index] = &[Index::Epoch];
const _I20: &[Index] = &[Index::Height];
const _I21: &[Index] = &[Index::TxIndex];
const _I22: &[Index] = &[Index::TxInIndex];
const _I23: &[Index] = &[Index::TxOutIndex];
const _I24: &[Index] = &[Index::EmptyOutputIndex];
const _I25: &[Index] = &[Index::OpReturnIndex];
const _I26: &[Index] = &[Index::P2AAddrIndex];
const _I27: &[Index] = &[Index::P2MSOutputIndex];
const _I28: &[Index] = &[Index::P2PK33AddrIndex];
const _I29: &[Index] = &[Index::P2PK65AddrIndex];
const _I30: &[Index] = &[Index::P2PKHAddrIndex];
const _I31: &[Index] = &[Index::P2SHAddrIndex];
const _I32: &[Index] = &[Index::P2TRAddrIndex];
const _I33: &[Index] = &[Index::P2WPKHAddrIndex];
const _I34: &[Index] = &[Index::P2WSHAddrIndex];
const _I35: &[Index] = &[Index::UnknownOutputIndex];
const _I36: &[Index] = &[Index::FundedAddrIndex];
const _I37: &[Index] = &[Index::ExtendedEmptyAddrIndex];

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
accessor! { SeriesPattern1, SeriesPattern1By, _I1, T::Nullable {
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
    plain halving: Halving -> T::Nullable,
    plain epoch: Epoch -> T::Nullable,
    plain height: Height -> T::Nullable,
} }
accessor! { SeriesPattern3, SeriesPattern3By, _I3, T {
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
accessor! { SeriesPattern4, SeriesPattern4By, _I4, T::Nullable {
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
accessor! { SeriesPattern5, SeriesPattern5By, _I5, T {
    date minute10: Minute10 -> T,
} }
accessor! { SeriesPattern6, SeriesPattern6By, _I6, T {
    date minute30: Minute30 -> T,
} }
accessor! { SeriesPattern7, SeriesPattern7By, _I7, T {
    date hour1: Hour1 -> T,
} }
accessor! { SeriesPattern8, SeriesPattern8By, _I8, T {
    date hour4: Hour4 -> T,
} }
accessor! { SeriesPattern9, SeriesPattern9By, _I9, T {
    date hour12: Hour12 -> T,
} }
accessor! { SeriesPattern10, SeriesPattern10By, _I10, T {
    date day1: Day1 -> T,
} }
accessor! { SeriesPattern11, SeriesPattern11By, _I11, T {
    date day3: Day3 -> T,
} }
accessor! { SeriesPattern12, SeriesPattern12By, _I12, T {
    date week1: Week1 -> T,
} }
accessor! { SeriesPattern13, SeriesPattern13By, _I13, T {
    date month1: Month1 -> T,
} }
accessor! { SeriesPattern14, SeriesPattern14By, _I14, T {
    date month3: Month3 -> T,
} }
accessor! { SeriesPattern15, SeriesPattern15By, _I15, T {
    date month6: Month6 -> T,
} }
accessor! { SeriesPattern16, SeriesPattern16By, _I16, T {
    date year1: Year1 -> T,
} }
accessor! { SeriesPattern17, SeriesPattern17By, _I17, T {
    date year10: Year10 -> T,
} }
accessor! { SeriesPattern18, SeriesPattern18By, _I18, T {
    plain halving: Halving -> T,
} }
accessor! { SeriesPattern19, SeriesPattern19By, _I19, T {
    plain epoch: Epoch -> T,
} }
accessor! { SeriesPattern20, SeriesPattern20By, _I20, T {
    plain height: Height -> T,
} }
accessor! { SeriesPattern21, SeriesPattern21By, _I21, T {
    plain tx_index: TxIndex -> T,
} }
accessor! { SeriesPattern22, SeriesPattern22By, _I22, T {
    plain txin_index: TxInIndex -> T,
} }
accessor! { SeriesPattern23, SeriesPattern23By, _I23, T {
    plain txout_index: TxOutIndex -> T,
} }
accessor! { SeriesPattern24, SeriesPattern24By, _I24, T {
    plain empty_output_index: EmptyOutputIndex -> T,
} }
accessor! { SeriesPattern25, SeriesPattern25By, _I25, T {
    plain op_return_index: OpReturnIndex -> T,
} }
accessor! { SeriesPattern26, SeriesPattern26By, _I26, T {
    plain p2a_addr_index: P2AAddrIndex -> T,
} }
accessor! { SeriesPattern27, SeriesPattern27By, _I27, T {
    plain p2ms_output_index: P2MSOutputIndex -> T,
} }
accessor! { SeriesPattern28, SeriesPattern28By, _I28, T {
    plain p2pk33_addr_index: P2PK33AddrIndex -> T,
} }
accessor! { SeriesPattern29, SeriesPattern29By, _I29, T {
    plain p2pk65_addr_index: P2PK65AddrIndex -> T,
} }
accessor! { SeriesPattern30, SeriesPattern30By, _I30, T {
    plain p2pkh_addr_index: P2PKHAddrIndex -> T,
} }
accessor! { SeriesPattern31, SeriesPattern31By, _I31, T {
    plain p2sh_addr_index: P2SHAddrIndex -> T,
} }
accessor! { SeriesPattern32, SeriesPattern32By, _I32, T {
    plain p2tr_addr_index: P2TRAddrIndex -> T,
} }
accessor! { SeriesPattern33, SeriesPattern33By, _I33, T {
    plain p2wpkh_addr_index: P2WPKHAddrIndex -> T,
} }
accessor! { SeriesPattern34, SeriesPattern34By, _I34, T {
    plain p2wsh_addr_index: P2WSHAddrIndex -> T,
} }
accessor! { SeriesPattern35, SeriesPattern35By, _I35, T {
    plain unknown_output_index: UnknownOutputIndex -> T,
} }
accessor! { SeriesPattern36, SeriesPattern36By, _I36, T {
    plain funded_addr_index: FundedAddrIndex -> T,
} }
accessor! { SeriesPattern37, SeriesPattern37By, _I37, T {
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
impl SeriesValue for Boolean {
    type Nullable = Option<Boolean>;
}
impl SeriesValue for Bytes {
    type Nullable = Option<Bytes>;
}
impl SeriesValue for Bytes32 {
    type Nullable = Option<Bytes32>;
}
impl SeriesValue for CapitalSentimentPhase {
    type Nullable = Option<CapitalSentimentPhase>;
}
impl SeriesValue for CoinbaseTag {
    type Nullable = Option<CoinbaseTag>;
}
impl SeriesValue for Count {
    type Nullable = Option<Count>;
}
impl SeriesValue for Count16 {
    type Nullable = Option<Count16>;
}
impl SeriesValue for Count32 {
    type Nullable = Option<Count32>;
}
impl SeriesValue for CountSigned {
    type Nullable = Option<CountSigned>;
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
impl SeriesValue for OHLCDollars {
    type Nullable = Option<OHLCDollars>;
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
impl SeriesValue for Rank {
    type Nullable = Option<Rank>;
}
impl SeriesValue for RawLockTime {
    type Nullable = Option<RawLockTime>;
}
impl SeriesValue for Sats {
    type Nullable = Option<Sats>;
}
impl SeriesValue for Score {
    type Nullable = Option<Score>;
}
impl SeriesValue for Seconds {
    type Nullable = Option<Seconds>;
}
impl SeriesValue for SigOps {
    type Nullable = Option<SigOps>;
}
impl SeriesValue for SigOps64 {
    type Nullable = Option<SigOps64>;
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

    shape! { Cycle at "series().rarity_meter.cycle" {
        pct0_1: SeriesPattern1<Option<Dollars>> = "*_pct0_1",
        pct0_5: SeriesPattern1<Option<Dollars>> = "*_pct0_5",
        pct1: SeriesPattern1<Option<Dollars>> = "*_pct1",
        pct2: SeriesPattern1<Option<Dollars>> = "*_pct2",
        pct5: SeriesPattern1<Option<Dollars>> = "*_pct5",
        pct10: SeriesPattern1<Option<Dollars>> = "*_pct10",
        pct20: SeriesPattern1<Option<Dollars>> = "*_pct20",
        pct30: SeriesPattern1<Option<Dollars>> = "*_pct30",
        pct40: SeriesPattern1<Option<Dollars>> = "*_pct40",
        median: SeriesPattern1<Option<Dollars>> = "*_median",
        pct60: SeriesPattern1<Option<Dollars>> = "*_pct60",
        pct70: SeriesPattern1<Option<Dollars>> = "*_pct70",
        pct80: SeriesPattern1<Option<Dollars>> = "*_pct80",
        pct90: SeriesPattern1<Option<Dollars>> = "*_pct90",
        pct95: SeriesPattern1<Option<Dollars>> = "*_pct95",
        pct98: SeriesPattern1<Option<Dollars>> = "*_pct98",
        pct99: SeriesPattern1<Option<Dollars>> = "*_pct99",
        pct99_5: SeriesPattern1<Option<Dollars>> = "*_pct99_5",
        pct99_9: SeriesPattern1<Option<Dollars>> = "*_pct99_9",
        level: SeriesPattern1<Score> = "*_level",
        score: SeriesPattern1<Score> = "*_score",
    } }
    shape! { Thresholds<A> at "series().rarity_meter.extremes.realized_loss_24h.thresholds" {
        tail0_1: SeriesPattern1<A> = "*_1",
        tail0_05: SeriesPattern1<A> = "*_05",
        tail0_025: SeriesPattern1<A> = "*_025",
    } }
    shape! { RealizedLoss24h<A> at "series().rarity_meter.extremes.realized_loss_24h" {
        thresholds: Thresholds<A> = "*_threshold_tail0",
        tail_share: SeriesPattern1<Option<Percent>> = "*_tail_share",
        rank: SeriesPattern1<Rank> = "*_rank",
    } }
    shape! { Extremes at "series().rarity_meter.extremes" {
        supply_in_loss: RealizedLoss24h<Option<Bitcoin>> = "*_supply_in_loss",
        realized_profit_24h: RealizedLoss24h<Option<Dollars>> = "*_realized_profit_24h",
        realized_loss_24h: RealizedLoss24h<Option<Dollars>> = "*_realized_loss_24h",
        realized_peak_regret_24h: RealizedLoss24h<Option<Dollars>> = "*_realized_peak_regret_24h",
        sell_side_risk_ratio_24h: RealizedLoss24h<Option<Ratio>> = "*_sell_side_risk_ratio_24h",
    } }
    shape! { Pct999 at "series().rarity_meter.components.active_price.pct99_9" {
        ratio: SeriesPattern1<Option<Ratio>> = "*_ratio_pct99_9",
        band: SeriesPattern1<Option<Dollars>> = "*_band_pct99_9",
    } }
    shape! { Pct995 at "series().rarity_meter.components.active_price.pct99_5" {
        ratio: SeriesPattern1<Option<Ratio>> = "*_ratio_pct99_5",
        band: SeriesPattern1<Option<Dollars>> = "*_band_pct99_5",
    } }
    shape! { Pct99 at "series().rarity_meter.components.active_price.pct99" {
        ratio: SeriesPattern1<Option<Ratio>> = "*_ratio_pct99",
        band: SeriesPattern1<Option<Dollars>> = "*_band_pct99",
    } }
    shape! { Pct98 at "series().rarity_meter.components.active_price.pct98" {
        ratio: SeriesPattern1<Option<Ratio>> = "*_ratio_pct98",
        band: SeriesPattern1<Option<Dollars>> = "*_band_pct98",
    } }
    shape! { Pct95 at "series().rarity_meter.components.active_price.pct95" {
        ratio: SeriesPattern1<Option<Ratio>> = "*_ratio_pct95",
        band: SeriesPattern1<Option<Dollars>> = "*_band_pct95",
    } }
    shape! { Pct90 at "series().rarity_meter.components.active_price.pct90" {
        ratio: SeriesPattern1<Option<Ratio>> = "*_ratio_pct90",
        band: SeriesPattern1<Option<Dollars>> = "*_band_pct90",
    } }
    shape! { Pct80 at "series().rarity_meter.components.active_price.pct80" {
        ratio: SeriesPattern1<Option<Ratio>> = "*_ratio_pct80",
        band: SeriesPattern1<Option<Dollars>> = "*_band_pct80",
    } }
    shape! { Pct70 at "series().rarity_meter.components.active_price.pct70" {
        ratio: SeriesPattern1<Option<Ratio>> = "*_ratio_pct70",
        band: SeriesPattern1<Option<Dollars>> = "*_band_pct70",
    } }
    shape! { Pct60 at "series().rarity_meter.components.active_price.pct60" {
        ratio: SeriesPattern1<Option<Ratio>> = "*_ratio_pct60",
        band: SeriesPattern1<Option<Dollars>> = "*_band_pct60",
    } }
    shape! { Median at "series().rarity_meter.components.active_price.median" {
        ratio: SeriesPattern1<Option<Ratio>> = "*_ratio_median",
        band: SeriesPattern1<Option<Dollars>> = "*_band_median",
    } }
    shape! { Pct40 at "series().rarity_meter.components.active_price.pct40" {
        ratio: SeriesPattern1<Option<Ratio>> = "*_ratio_pct40",
        band: SeriesPattern1<Option<Dollars>> = "*_band_pct40",
    } }
    shape! { Pct30 at "series().rarity_meter.components.active_price.pct30" {
        ratio: SeriesPattern1<Option<Ratio>> = "*_ratio_pct30",
        band: SeriesPattern1<Option<Dollars>> = "*_band_pct30",
    } }
    shape! { Pct20 at "series().rarity_meter.components.active_price.pct20" {
        ratio: SeriesPattern1<Option<Ratio>> = "*_ratio_pct20",
        band: SeriesPattern1<Option<Dollars>> = "*_band_pct20",
    } }
    shape! { Pct10 at "series().rarity_meter.components.active_price.pct10" {
        ratio: SeriesPattern1<Option<Ratio>> = "*_ratio_pct10",
        band: SeriesPattern1<Option<Dollars>> = "*_band_pct10",
    } }
    shape! { Pct5 at "series().rarity_meter.components.active_price.pct5" {
        ratio: SeriesPattern1<Option<Ratio>> = "*_ratio_pct5",
        band: SeriesPattern1<Option<Dollars>> = "*_band_pct5",
    } }
    shape! { Pct2 at "series().rarity_meter.components.active_price.pct2" {
        ratio: SeriesPattern1<Option<Ratio>> = "*_ratio_pct2",
        band: SeriesPattern1<Option<Dollars>> = "*_band_pct2",
    } }
    shape! { Pct1 at "series().rarity_meter.components.active_price.pct1" {
        ratio: SeriesPattern1<Option<Ratio>> = "*_ratio_pct1",
        band: SeriesPattern1<Option<Dollars>> = "*_band_pct1",
    } }
    shape! { Pct05 at "series().rarity_meter.components.active_price.pct0_5" {
        ratio: SeriesPattern1<Option<Ratio>> = "*_ratio_pct0_5",
        band: SeriesPattern1<Option<Dollars>> = "*_band_pct0_5",
    } }
    shape! { Pct01 at "series().rarity_meter.components.active_price.pct0_1" {
        ratio: SeriesPattern1<Option<Ratio>> = "*_ratio_pct0_1",
        band: SeriesPattern1<Option<Dollars>> = "*_band_pct0_1",
    } }
    shape! { AwakeCostBasisPerCoinMedian at "series().rarity_meter.components.awake_cost_basis_per_coin_median" {
        block: SeriesPattern1<Option<Dollars>> = "*",
        ratio: SeriesPattern1<Option<Ratio>> = "*_ratio",
        pct0_1: Pct01 = "*",
        pct0_5: Pct05 = "*",
        pct1: Pct1 = "*",
        pct2: Pct2 = "*",
        pct5: Pct5 = "*",
        pct10: Pct10 = "*",
        pct20: Pct20 = "*",
        pct30: Pct30 = "*",
        pct40: Pct40 = "*",
        median: Median = "*",
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
        median: Median = "*",
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
        realized_price: ActivePrice = "realized_price",
        capitalized_price: ActivePrice = "capitalized_price",
        cost_basis_per_coin_median: AwakeCostBasisPerCoinMedian = "*_coin_median",
        cost_basis_per_dollar_median: AwakeCostBasisPerCoinMedian = "*_dollar_median",
        sth_cost_basis_per_coin_median: AwakeCostBasisPerCoinMedian = "sth_*_coin_median",
        sth_cost_basis_per_dollar_median: AwakeCostBasisPerCoinMedian = "sth_*_dollar_median",
        lth_cost_basis_per_coin_median: AwakeCostBasisPerCoinMedian = "lth_*_coin_median",
        lth_cost_basis_per_dollar_median: AwakeCostBasisPerCoinMedian = "lth_*_dollar_median",
        awake_cost_basis_per_coin_median: AwakeCostBasisPerCoinMedian = "awake_*_coin_median",
        awake_cost_basis_per_dollar_median: AwakeCostBasisPerCoinMedian = "awake_*_dollar_median",
        mobile_cost_basis_per_coin_median: AwakeCostBasisPerCoinMedian = "mobile_*_coin_median",
        mobile_cost_basis_per_dollar_median: AwakeCostBasisPerCoinMedian = "mobile_*_dollar_median",
        sth_awake_cost_basis_per_coin_median: AwakeCostBasisPerCoinMedian = "sth_awake_*_coin_median",
        sth_awake_cost_basis_per_dollar_median: AwakeCostBasisPerCoinMedian = "sth_awake_*_dollar_median",
        lth_awake_cost_basis_per_coin_median: AwakeCostBasisPerCoinMedian = "lth_awake_*_coin_median",
        lth_awake_cost_basis_per_dollar_median: AwakeCostBasisPerCoinMedian = "lth_awake_*_dollar_median",
        sth_mobile_cost_basis_per_coin_median: AwakeCostBasisPerCoinMedian = "sth_mobile_*_coin_median",
        sth_mobile_cost_basis_per_dollar_median: AwakeCostBasisPerCoinMedian = "sth_mobile_*_dollar_median",
        lth_mobile_cost_basis_per_coin_median: AwakeCostBasisPerCoinMedian = "lth_mobile_*_coin_median",
        lth_mobile_cost_basis_per_dollar_median: AwakeCostBasisPerCoinMedian = "lth_mobile_*_dollar_median",
        sth_realized_price: ActivePrice = "sth_realized_price",
        sth_capitalized_price: ActivePrice = "sth_capitalized_price",
        lth_realized_price: ActivePrice = "lth_realized_price",
        lth_capitalized_price: ActivePrice = "lth_capitalized_price",
        utxos_over_6m_old_realized_price: ActivePrice = "utxos_over_6m_old_realized_price",
        utxos_over_4m_old_realized_price: ActivePrice = "utxos_over_4m_old_realized_price",
        utxos_under_4m_old_realized_price: ActivePrice = "utxos_under_4m_old_realized_price",
        utxos_under_6m_old_realized_price: ActivePrice = "utxos_under_6m_old_realized_price",
        utxos_under_4m_old_capitalized_price: ActivePrice = "utxos_under_4m_old_capitalized_price",
        utxos_under_6m_old_capitalized_price: ActivePrice = "utxos_under_6m_old_capitalized_price",
        vaulted_price: ActivePrice = "vaulted_price",
        active_price: ActivePrice = "active_price",
        true_market_mean: ActivePrice = "true_market_mean",
        cointime_price: ActivePrice = "cointime_price",
        awake_realized_price: ActivePrice = "awake_realized_price",
        mobile_realized_price: ActivePrice = "mobile_realized_price",
    } }
    shape! { RarityMeter at "series().rarity_meter" {
        components: Components = "cost_basis_per",
        extremes: Extremes = "*",
        full: Cycle = "*",
        full_v2: Cycle = "*_v2",
        local: Cycle = "local_*",
        local_v2: Cycle = "local_*_v2",
        cycle: Cycle = "cycle_*",
        cycle_v2: Cycle = "cycle_*_v2",
    } }
    shape! { CapitalSentiment at "series().capital_sentiment" {
        is_long: SeriesPattern1<Boolean> = "*_is_long",
        is_short: SeriesPattern1<Boolean> = "*_is_short",
        phase: SeriesPattern2<CapitalSentimentPhase> = "*_phase",
        score: SeriesPattern2<Score> = "*_score",
    } }
    shape! { CostBasisAboveFloor at "series().bedrock.awake.cost_basis_above_floor" {
        pct10: SeriesPattern1<Option<Dollars>> = "*_pct10",
        pct20: SeriesPattern1<Option<Dollars>> = "*_pct20",
        pct30: SeriesPattern1<Option<Dollars>> = "*_pct30",
        pct40: SeriesPattern1<Option<Dollars>> = "*_pct40",
        median: SeriesPattern1<Option<Dollars>> = "*_median",
        pct60: SeriesPattern1<Option<Dollars>> = "*_pct60",
        pct70: SeriesPattern1<Option<Dollars>> = "*_pct70",
        pct80: SeriesPattern1<Option<Dollars>> = "*_pct80",
        pct90: SeriesPattern1<Option<Dollars>> = "*_pct90",
    } }
    shape! { Floor<A> at "series().bedrock.awake.floor" {
        pct95: SeriesPattern1<A> = "*_pct95",
        pct98: SeriesPattern1<A> = "*_pct98",
        pct99: SeriesPattern1<A> = "*_pct99",
        pct99_5: SeriesPattern1<A> = "*_pct99_5",
        pct99_9: SeriesPattern1<A> = "*_pct99_9",
    } }
    shape! { Awake at "series().bedrock.awake" {
        supply_in_loss_share_threshold: Floor<Option<Percent>> = "*_supply_in_loss_share_threshold",
        floor: Floor<Option<Dollars>> = "*_floor",
        cost_basis_above_floor: CostBasisAboveFloor = "*_cost_basis_above_floor",
    } }
    shape! { Bedrock at "series().bedrock" {
        unweighted: Awake = "*_unweighted",
        awake: Awake = "*_awake",
        mobile: Awake = "*_mobile",
    } }
    shape! { TrueMarketMean at "series().cointime.prices.true_market_mean" {
        block: SeriesPattern1<Option<Dollars>> = "*",
        ratio: SeriesPattern1<Option<Ratio>> = "*_ratio",
        aviv: SeriesPattern1<Option<Ratio>> = "aviv",
        aviv_nupl: SeriesPattern1<Option<Ratio>> = "aviv_nupl",
    } }
    shape! { PricesActive at "series().cointime.prices.active" {
        block: SeriesPattern1<Option<Dollars>> = "*_price",
        ratio: SeriesPattern1<Option<Ratio>> = "*_price_ratio",
        mvrv: SeriesPattern1<Option<Ratio>> = "*_mvrv",
    } }
    shape! { CointimePrices at "series().cointime.prices" {
        vaulted: PricesActive = "*",
        active: PricesActive = "active",
        true_market_mean: TrueMarketMean = "true_market_mean",
        cointime: PricesActive = "cointime",
    } }
    shape! { Caps at "series().cointime.caps" {
        thermo: SeriesPattern1<Option<Dollars>> = "thermo_*",
        investor: SeriesPattern1<Option<Dollars>> = "investor_*",
        active: SeriesPattern1<Option<Dollars>> = "active_*",
        vaulted: SeriesPattern1<Option<Dollars>> = "vaulted_*",
        cointime: SeriesPattern1<Option<Dollars>> = "cointime_*",
        investorness: SeriesPattern1<Option<Percent>> = "investorness",
        producerness: SeriesPattern1<Option<Percent>> = "producerness",
    } }
    shape! { SupplyInLoss at "series().coinflow.cohorts.all.mobile.supply.in_loss" {
        share: SeriesPattern1<Option<Percent>> = "*",
    } }
    shape! { MobileSupply at "series().coinflow.cohorts.all.mobile.supply" {
        btc: SeriesPattern1<Option<Bitcoin>> = "*",
        usd: SeriesPattern1<Option<Dollars>> = "*_usd",
        in_loss: SupplyInLoss = "*_in_loss_share",
    } }
    shape! { Dormancy at "series().indicators.dormancy" {
        supply_adjusted: SeriesPattern1<Option<Float32>> = "*_supply_adjusted",
        flow: SeriesPattern1<Option<Float32>> = "*_flow",
    } }
    shape! { Indicators at "series().indicators" {
        puell_multiple: SeriesPattern1<Option<Ratio>> = "puell_multiple",
        nvt: SeriesPattern1<Option<Ratio>> = "nvt",
        gini: SeriesPattern1<Option<Ratio>> = "gini",
        rhodl_ratio: SeriesPattern1<Option<Ratio>> = "rhodl_ratio",
        thermo_cap_multiple: SeriesPattern1<Option<Ratio>> = "thermo_cap_multiple",
        coindays_destroyed_supply_adjusted: SeriesPattern1<Option<Days>> = "coindays_*",
        coinyears_destroyed_supply_adjusted: SeriesPattern1<Option<Years>> = "coinyears_*",
        dormancy: Dormancy = "dormancy",
        stock_to_flow: SeriesPattern1<Option<Years>> = "stock_to_flow",
        seller_exhaustion_constant: SeriesPattern1<Option<Ratio>> = "seller_exhaustion_constant",
    } }
    shape! { Velocity at "series().supply.velocity" {
        btc: SeriesPattern1<Option<Ratio64>> = "*_btc",
        usd: SeriesPattern1<Option<Ratio64>> = "*_usd",
    } }
    shape! { Adjusted at "series().cointime.adjusted" {
        inflation_rate: SeriesPattern1<Option<Percent>> = "*_inflation_rate",
        stock_to_flow: SeriesPattern1<Option<Years>> = "*_stock_to_flow",
        velocity: Velocity = "*_velocity",
    } }
    shape! { RookieUnrealized at "series().entry.rookie.unrealized" {
        profit: SeriesPattern1<Option<Dollars>> = "*_unrealized_profit",
        loss: SeriesPattern1<Option<Dollars>> = "*_unrealized_loss",
        net_pnl: SeriesPattern1<Option<Dollars>> = "*_net_unrealized_pnl",
        nupl: SeriesPattern1<Option<Ratio>> = "*_nupl",
    } }
    shape! { Relative at "series().holders.all.relative" {
        supply_share: SeriesPattern1<Option<Percent>> = "*_supply_share",
        supply_in_profit_share: SeriesPattern1<Option<Percent>> = "*_supply_in_profit_share",
        supply_in_loss_share: SeriesPattern1<Option<Percent>> = "*_supply_in_loss_share",
        unrealized_profit_to_mcap: SeriesPattern1<Option<Ratio>> = "*_unrealized_profit_to_mcap",
        unrealized_loss_to_mcap: SeriesPattern1<Option<Ratio>> = "*_unrealized_loss_to_mcap",
        unrealized_profit_to_own_mcap: SeriesPattern1<Option<Ratio>> = "*_unrealized_profit_to_own_mcap",
        unrealized_loss_to_own_mcap: SeriesPattern1<Option<Ratio>> = "*_unrealized_loss_to_own_mcap",
        unrealized_profit_to_own_gross_pnl: SeriesPattern1<Option<Ratio>> = "*_unrealized_profit_to_own_gross_pnl",
        unrealized_loss_to_own_gross_pnl: SeriesPattern1<Option<Ratio>> = "*_unrealized_loss_to_own_gross_pnl",
        net_unrealized_pnl_to_own_gross_pnl: SeriesPattern1<Option<Ratio>> = "*_net_unrealized_pnl_to_own_gross_pnl",
        invested_capital_in_profit_share: SeriesPattern1<Option<Percent>> = "*_invested_capital_in_profit_share",
        invested_capital_in_loss_share: SeriesPattern1<Option<Percent>> = "*_invested_capital_in_loss_share",
        realized_cap_to_own_mcap: SeriesPattern1<Option<Ratio>> = "*_realized_cap_to_own_mcap",
        net_pnl_change_1m_to_mcap: SeriesPattern1<Option<Ratio>> = "*_net_pnl_change_1m_to_mcap",
        net_pnl_change_1m_to_rcap: SeriesPattern1<Option<Ratio>> = "*_net_pnl_change_1m_to_rcap",
    } }
    shape! { SoprRatioExtended at "series().holders.all.ratios.sopr_ratio_extended" {
        _1w: SeriesPattern1<Option<Ratio>> = "*_1w",
        _1m: SeriesPattern1<Option<Ratio>> = "*_1m",
        _1y: SeriesPattern1<Option<Ratio>> = "*_1y",
    } }
    shape! { CapitalDensity at "series().holders.all.cost_basis.capital_density" {
        total: SeriesPattern1<Option<Percent>> = "*",
        in_profit: SeriesPattern1<Option<Percent>> = "*_in_profit",
        in_loss: SeriesPattern1<Option<Percent>> = "*_in_loss",
    } }
    shape! { PerCoin at "series().holders.all.cost_basis.per_coin" {
        pct5: SeriesPattern1<Option<Dollars>> = "*_pct5",
        pct10: SeriesPattern1<Option<Dollars>> = "*_pct10",
        pct15: SeriesPattern1<Option<Dollars>> = "*_pct15",
        pct20: SeriesPattern1<Option<Dollars>> = "*_pct20",
        pct25: SeriesPattern1<Option<Dollars>> = "*_pct25",
        pct30: SeriesPattern1<Option<Dollars>> = "*_pct30",
        pct35: SeriesPattern1<Option<Dollars>> = "*_pct35",
        pct40: SeriesPattern1<Option<Dollars>> = "*_pct40",
        pct45: SeriesPattern1<Option<Dollars>> = "*_pct45",
        median: SeriesPattern1<Option<Dollars>> = "*_median",
        pct55: SeriesPattern1<Option<Dollars>> = "*_pct55",
        pct60: SeriesPattern1<Option<Dollars>> = "*_pct60",
        pct65: SeriesPattern1<Option<Dollars>> = "*_pct65",
        pct70: SeriesPattern1<Option<Dollars>> = "*_pct70",
        pct75: SeriesPattern1<Option<Dollars>> = "*_pct75",
        pct80: SeriesPattern1<Option<Dollars>> = "*_pct80",
        pct85: SeriesPattern1<Option<Dollars>> = "*_pct85",
        pct90: SeriesPattern1<Option<Dollars>> = "*_pct90",
        pct95: SeriesPattern1<Option<Dollars>> = "*_pct95",
    } }
    shape! { MobileCostBasis at "series().coinflow.cohorts.all.mobile.cost_basis" {
        per_coin: PerCoin = "*_cost_basis_per_coin",
        per_dollar: PerCoin = "*_cost_basis_per_dollar",
        supply_density: CapitalDensity = "*_supply_density",
        capital_density: CapitalDensity = "*_capital_density",
    } }
    shape! { CostBasisInLoss at "series().holders.all.cost_basis.in_loss" {
        per_coin: SeriesPattern1<Option<Dollars>> = "*_coin",
        per_dollar: SeriesPattern1<Option<Dollars>> = "*_dollar",
    } }
    shape! { AllCostBasis at "series().holders.all.cost_basis" {
        in_profit: CostBasisInLoss = "*_cost_basis_in_profit_per",
        in_loss: CostBasisInLoss = "*_cost_basis_in_loss_per",
        min: SeriesPattern1<Option<Dollars>> = "*_cost_basis_min",
        max: SeriesPattern1<Option<Dollars>> = "*_cost_basis_max",
        per_coin: PerCoin = "*_cost_basis_per_coin",
        per_dollar: PerCoin = "*_cost_basis_per_dollar",
        supply_density: CapitalDensity = "*_supply_density",
        capital_density: CapitalDensity = "*_capital_density",
    } }
    shape! { AllUnrealized at "series().holders.all.unrealized" {
        profit: SeriesPattern1<Option<Dollars>> = "*_unrealized_profit",
        loss: SeriesPattern1<Option<Dollars>> = "*_unrealized_loss",
        net_pnl: SeriesPattern1<Option<Dollars>> = "*_net_unrealized_pnl",
        gross_pnl: SeriesPattern1<Option<Dollars>> = "*_unrealized_gross_pnl",
        invested_capital_in_profit: SeriesPattern1<Option<Dollars>> = "*_invested_capital_in_profit",
        invested_capital_in_loss: SeriesPattern1<Option<Dollars>> = "*_invested_capital_in_loss",
        pain_index: SeriesPattern1<Option<Dollars>> = "*_pain_index",
        greed_index: SeriesPattern1<Option<Dollars>> = "*_greed_index",
        net_sentiment: SeriesPattern1<Option<Dollars>> = "*_net_sentiment",
        nupl: SeriesPattern1<Option<Ratio>> = "*_nupl",
    } }
    shape! { Total at "series().holders.all.supply.total" {
        btc: SeriesPattern1<Option<Bitcoin>> = "*",
        usd: SeriesPattern1<Option<Dollars>> = "market_cap",
    } }
    shape! { State at "series().addresses.state" {
        p2a: SeriesPattern26<AddrState> = "*_state",
        p2pk33: SeriesPattern28<AddrState> = "*_state",
        p2pk65: SeriesPattern29<AddrState> = "*_state",
        p2pkh: SeriesPattern30<AddrState> = "*_state",
        p2sh: SeriesPattern31<AddrState> = "*_state",
        p2tr: SeriesPattern32<AddrState> = "*_state",
        p2wpkh: SeriesPattern33<AddrState> = "*_state",
        p2wsh: SeriesPattern34<AddrState> = "*_state",
        funded: SeriesPattern36<FundedAddrData> = "funded_*_data",
        extended_empty: SeriesPattern37<EmptyAddrData> = "extended_empty_*_data",
    } }
    shape! { ExposedCount at "series().addresses.exposed.count" {
        funded: SeriesPattern1<Count> = "*_exposed_address_count",
        total: SeriesPattern1<Count> = "*_total_exposed_address_count",
    } }
    shape! { RespentCount at "series().addresses.respent.count" {
        funded: SeriesPattern1<Count> = "*_respent_address_count",
        total: SeriesPattern1<Count> = "*_total_respent_address_count",
    } }
    shape! { ExposedSupply at "series().addresses.exposed.supply" {
        btc: SeriesPattern1<Option<Bitcoin>> = "*",
        usd: SeriesPattern1<Option<Dollars>> = "*_usd",
        share: SeriesPattern1<Option<Percent>> = "*_share",
    } }
    shape! { Exposed at "series().addresses.exposed" {
        count: ExposedCount = "*",
        supply: ExposedSupply = "*_exposed_address_supply",
    } }
    shape! { ReusedCount at "series().addresses.reused.count" {
        funded: SeriesPattern1<Count> = "*_reused_address_count",
        total: SeriesPattern1<Count> = "*_total_reused_address_count",
    } }
    shape! { Classes2009Unrealized at "series().age.classes._2009.unrealized" {
        profit: SeriesPattern1<Option<Dollars>> = "*_unrealized_profit",
        loss: SeriesPattern1<Option<Dollars>> = "*_unrealized_loss",
        net_pnl: SeriesPattern1<Option<Dollars>> = "*_net_unrealized_pnl",
    } }
    shape! { DeltaRate at "series().addresses.funded.delta.rate" {
        _24h: SeriesPattern1<Option<Percent>> = "*_24h_rate",
        _1w: SeriesPattern1<Option<Percent>> = "*_1w_rate",
        _1m: SeriesPattern1<Option<Percent>> = "*_1m_rate",
        _1y: SeriesPattern1<Option<Percent>> = "*_1y_rate",
    } }
    shape! { Macd1m at "series().market.macd._1m" {
        line: SeriesPattern1<Option<Dollars>> = "macd_line_*",
        signal: SeriesPattern1<Option<Dollars>> = "macd_signal_*",
        histogram: SeriesPattern1<Option<Dollars>> = "macd_histogram_*",
    } }
    shape! { Stochastic at "series().market.rsi._1m.stochastic" {
        k: SeriesPattern1<Option<Percent>> = "stochastic_rsi_k_*",
        d: SeriesPattern1<Option<Percent>> = "stochastic_rsi_d_*",
    } }
    shape! { Rsi1m at "series().market.rsi._1m" {
        block: SeriesPattern1<Option<Percent>> = "rsi_*",
        stochastic: Stochastic = "*",
    } }
    shape! { Macd<A> at "series().market.macd" {
        _24h: A = "*",
        _1w: A = "1w",
        _1m: A = "1m",
    } }
    shape! { Sma350d at "series().market.sma._350d" {
        block: SeriesPattern1<Option<Dollars>> = "*",
        ratio: SeriesPattern1<Option<Ratio>> = "*_ratio",
        x2: SeriesPattern1<Option<Dollars>> = "*_x2",
    } }
    shape! { Sma200d at "series().market.sma._200d" {
        block: SeriesPattern1<Option<Dollars>> = "*",
        ratio: SeriesPattern1<Option<Ratio>> = "*_ratio",
        x2_4: SeriesPattern1<Option<Dollars>> = "*_x2_4",
        x0_8: SeriesPattern1<Option<Dollars>> = "*_x0_8",
        mayer_multiple: SeriesPattern1<Option<Ratio>> = "mayer_multiple",
    } }
    shape! { Ema12d at "series().market.ema._12d" {
        block: SeriesPattern1<Option<Dollars>> = "*",
        ratio: SeriesPattern1<Option<Ratio>> = "*_ratio",
    } }
    shape! { Mobile at "series().coinflow.cohorts.all.mobile" {
        supply: MobileSupply = "*_supply",
        realized_cap: SeriesPattern1<Option<Dollars>> = "*_realized_cap",
        realized_price: Ema12d = "*_realized_price",
        capitalized_price: Ema12d = "*_capitalized_price",
        cost_basis: MobileCostBasis = "*",
    } }
    shape! { Ema at "series().market.ema" {
        _1w: Ema12d = "*_1w",
        _8d: Ema12d = "*_8d",
        _12d: Ema12d = "*_12d",
        _13d: Ema12d = "*_13d",
        _21d: Ema12d = "*_21d",
        _26d: Ema12d = "*_26d",
        _1m: Ema12d = "*_1m",
        _34d: Ema12d = "*_34d",
        _55d: Ema12d = "*_55d",
        _89d: Ema12d = "*_89d",
        _144d: Ema12d = "*_144d",
        _200d: Ema12d = "*_200d",
        _1y: Ema12d = "*_1y",
        _2y: Ema12d = "*_2y",
        _200w: Ema12d = "*_200w",
        _4y: Ema12d = "*_4y",
    } }
    shape! { MarketSma at "series().market.sma" {
        _1w: Ema12d = "*_1w",
        _8d: Ema12d = "*_8d",
        _13d: Ema12d = "*_13d",
        _21d: Ema12d = "*_21d",
        _1m: Ema12d = "*_1m",
        _34d: Ema12d = "*_34d",
        _50d: Ema12d = "*_50d",
        _55d: Ema12d = "*_55d",
        _89d: Ema12d = "*_89d",
        _111d: Ema12d = "*_111d",
        _144d: Ema12d = "*_144d",
        _200d: Sma200d = "*_200d",
        _350d: Sma350d = "*_350d",
        _1y: Ema12d = "*_1y",
        _2y: Ema12d = "*_2y",
        _200w: Ema12d = "*_200w",
        _4y: Ema12d = "*_4y",
    } }
    shape! { Max at "series().market.range.max" {
        _1w: SeriesPattern1<Option<Dollars>> = "*_1w",
        _2w: SeriesPattern1<Option<Dollars>> = "*_2w",
        _1m: SeriesPattern1<Option<Dollars>> = "*_1m",
        _1y: SeriesPattern1<Option<Dollars>> = "*_1y",
    } }
    shape! { Range at "series().market.range" {
        min: Max = "*_min",
        max: Max = "*_max",
        true_range: SeriesPattern1<Option<Dollars>> = "*_true_range",
        true_range_sum_2w: SeriesPattern1<Option<Dollars>> = "*_true_range_sum_2w",
        choppiness_index_2w: SeriesPattern1<Option<Percent>> = "*_choppiness_index_2w",
    } }
    shape! { Cagr at "series().market.returns.cagr" {
        _2y: SeriesPattern1<Option<Percent>> = "*_2y",
        _3y: SeriesPattern1<Option<Percent>> = "*_3y",
        _4y: SeriesPattern1<Option<Percent>> = "*_4y",
        _5y: SeriesPattern1<Option<Percent>> = "*_5y",
        _6y: SeriesPattern1<Option<Percent>> = "*_6y",
        _8y: SeriesPattern1<Option<Percent>> = "*_8y",
        _10y: SeriesPattern1<Option<Percent>> = "*_10y",
    } }
    shape! { Periods at "series().market.returns.periods" {
        _24h: SeriesPattern1<Option<Percent>> = "*_24h",
        _1w: SeriesPattern1<Option<Percent>> = "*_1w",
        _1m: SeriesPattern1<Option<Percent>> = "*_1m",
        _3m: SeriesPattern1<Option<Percent>> = "*_3m",
        _6m: SeriesPattern1<Option<Percent>> = "*_6m",
        _1y: SeriesPattern1<Option<Percent>> = "*_1y",
        _2y: SeriesPattern1<Option<Percent>> = "*_2y",
        _3y: SeriesPattern1<Option<Percent>> = "*_3y",
        _4y: SeriesPattern1<Option<Percent>> = "*_4y",
        _5y: SeriesPattern1<Option<Percent>> = "*_5y",
        _6y: SeriesPattern1<Option<Percent>> = "*_6y",
        _8y: SeriesPattern1<Option<Percent>> = "*_8y",
        _10y: SeriesPattern1<Option<Percent>> = "*_10y",
    } }
    shape! { Lookback at "series().market.lookback" {
        _24h: SeriesPattern1<Option<Dollars>> = "*_24h_ago",
        _1w: SeriesPattern1<Option<Dollars>> = "*_1w_ago",
        _1m: SeriesPattern1<Option<Dollars>> = "*_1m_ago",
        _3m: SeriesPattern1<Option<Dollars>> = "*_3m_ago",
        _6m: SeriesPattern1<Option<Dollars>> = "*_6m_ago",
        _1y: SeriesPattern1<Option<Dollars>> = "*_1y_ago",
        _2y: SeriesPattern1<Option<Dollars>> = "*_2y_ago",
        _3y: SeriesPattern1<Option<Dollars>> = "*_3y_ago",
        _4y: SeriesPattern1<Option<Dollars>> = "*_4y_ago",
        _5y: SeriesPattern1<Option<Dollars>> = "*_5y_ago",
        _6y: SeriesPattern1<Option<Dollars>> = "*_6y_ago",
        _8y: SeriesPattern1<Option<Dollars>> = "*_8y_ago",
        _10y: SeriesPattern1<Option<Dollars>> = "*_10y_ago",
    } }
    shape! { Ath at "series().market.ath" {
        high: SeriesPattern1<Option<Dollars>> = "*_ath",
        drawdown: SeriesPattern1<Option<Percent>> = "*_drawdown",
        days_since: SeriesPattern1<Option<Days>> = "days_since_*_ath",
        max_days_between: SeriesPattern1<Option<Days>> = "max_days_between_*_ath",
    } }
    shape! { UtxoSet2 at "series().utxo_set" {
        supply: SeriesPattern1<Option<Bitcoin>> = "*",
        count: SeriesPattern1<Count> = "utxo_count",
    } }
    shape! { HashratePrice at "series().mining.hashrate.price" {
        block: SeriesPattern1<Option<Float32>> = "*",
        atl: SeriesPattern1<Option<Float32>> = "*_atl",
        rebound: SeriesPattern1<Option<Percent>> = "*_rebound",
    } }
    shape! { RateSma at "series().mining.hashrate.rate.sma" {
        _1w: SeriesPattern1<Option<Hashrate>> = "*_1w",
        _1m: SeriesPattern1<Option<Hashrate>> = "*_1m",
        _2m: SeriesPattern1<Option<Hashrate>> = "*_2m",
        _1y: SeriesPattern1<Option<Hashrate>> = "*_1y",
    } }
    shape! { HashrateRate at "series().mining.hashrate.rate" {
        block: SeriesPattern1<Option<Hashrate>> = "*",
        sma: RateSma = "*_sma",
        ath: SeriesPattern1<Option<Hashrate>> = "*_ath",
        drawdown: SeriesPattern1<Option<Percent>> = "*_drawdown",
    } }
    shape! { MiningHashrate at "series().mining.hashrate" {
        rate: HashrateRate = "hashrate",
        price: HashratePrice = "*_price",
        value: HashratePrice = "*_value",
    } }
    shape! { EffectiveFeeRate6b<A> at "series().transactions.fees.effective_fee_rate._6b" {
        min: SeriesPattern1<A> = "*_min",
        max: SeriesPattern1<A> = "*_max",
        pct10: SeriesPattern1<A> = "*_pct10",
        pct25: SeriesPattern1<A> = "*_pct25",
        median: SeriesPattern1<A> = "*_median",
        pct75: SeriesPattern1<A> = "*_pct75",
        pct90: SeriesPattern1<A> = "*_pct90",
    } }
    shape! { SizeWeight at "series().transactions.size.weight" {
        block: EffectiveFeeRate6b<Weight> = "*",
        _6b: EffectiveFeeRate6b<Weight> = "*_6b",
    } }
    shape! { Vsize6b at "series().transactions.size.vsize._6b" {
        min: SeriesPattern20<VSize> = "*_min",
        max: SeriesPattern20<VSize> = "*_max",
        pct10: SeriesPattern20<VSize> = "*_pct10",
        pct25: SeriesPattern20<VSize> = "*_pct25",
        median: SeriesPattern20<VSize> = "*_median",
        pct75: SeriesPattern20<VSize> = "*_pct75",
        pct90: SeriesPattern20<VSize> = "*_pct90",
    } }
    shape! { EffectiveFeeRate<A, B> at "series().transactions.fees.effective_fee_rate" {
        tx_index: SeriesPattern21<A> = "*",
        block: B = "*",
        _6b: B = "*_6b",
    } }
    shape! { TransactionsSize at "series().transactions.size" {
        vsize: EffectiveFeeRate<VSize, Vsize6b> = "*_vsize",
        weight: SizeWeight = "*_weight",
    } }
    shape! { AvgBalance at "series().addresses.avg_balance" {
        btc: SeriesPattern1<Option<Bitcoin>> = "*",
        usd: SeriesPattern1<Option<Dollars>> = "*_usd",
    } }
    shape! { CointimeSupply at "series().cointime.supply" {
        vaulted: AvgBalance = "vaulted_*",
        hodled_or_lost: AvgBalance = "hodled_or_lost_*",
        active: AvgBalance = "active_*",
    } }
    shape! { Immobile at "series().coinflow.age_ranges._10y_to_12y.immobile" {
        supply: AvgBalance = "*",
    } }
    shape! { CoinflowCohortsAll at "series().coinflow.cohorts.all" {
        mobile: Mobile = "*_mobile",
        immobile: Immobile = "*_immobile_supply",
    } }
    shape! { CoinflowAgeRanges10yTo12y at "series().coinflow.age_ranges._10y_to_12y" {
        spending_rate: SeriesPattern1<Option<PerDay>> = "*_spending_rate",
        spending_exposure: SeriesPattern1<Option<Float64>> = "*_spending_exposure",
        mobility: SeriesPattern1<Option<Ratio64>> = "*_mobility",
        mobile: Immobile = "*_mobile_supply",
        immobile: Immobile = "*_immobile_supply",
    } }
    shape! { CointimeCohortsAll at "series().cointime.cohorts.all" {
        awake: Mobile = "*_awake",
        dormant: Immobile = "*_dormant_supply",
    } }
    shape! { Sum at "series().op_return.fees.sum" {
        _24h: AvgBalance = "*_24h",
        _1w: AvgBalance = "*_1w",
        _1m: AvgBalance = "*_1m",
        _1y: AvgBalance = "*_1y",
    } }
    shape! { AaopoolRewards at "series().pools.aaopool.rewards" {
        cumulative: AvgBalance = "*_cumulative",
        sum: Sum = "*_sum",
    } }
    shape! { Block at "series().op_return.fees.block" {
        btc: SeriesPattern20<Option<Bitcoin>> = "*",
        usd: SeriesPattern20<Option<Dollars>> = "*_usd",
    } }
    shape! { InscriptionFees at "series().transactions.inscription.fees" {
        block: Block = "*_fees",
        cumulative: AvgBalance = "*_fees_cumulative",
        sum: Sum = "*_fees_sum",
        chain_share: SeriesPattern1<Option<Percent>> = "*_fee_chain_share",
    } }
    shape! { OpReturnValue at "series().outputs.op_return_value" {
        block: Block = "*",
        cumulative: AvgBalance = "*_cumulative",
        sum: Sum = "*_sum",
    } }
    shape! { Balances0satsActivity at "series().addresses.balances._0sats.activity" {
        transfer_volume: OpReturnValue = "*",
    } }
    shape! { TransferVolume at "series().entry.rookie.activity.transfer_volume" {
        block: Block = "*",
        cumulative: AvgBalance = "*_cumulative",
        sum: Sum = "*_sum",
        in_profit: OpReturnValue = "*_in_profit",
        in_loss: OpReturnValue = "*_in_loss",
    } }
    shape! { Spent at "series().outputs.spent" {
        txin_index: SeriesPattern23<TxInIndex> = "*",
    } }
    shape! { ChainShare at "series().op_return.fees.chain_share" {
        cumulative: SeriesPattern1<Option<Percent>> = "*",
        _24h: SeriesPattern1<Option<Percent>> = "*_24h",
        _1w: SeriesPattern1<Option<Percent>> = "*_1w",
        _1m: SeriesPattern1<Option<Percent>> = "*_1m",
        _1y: SeriesPattern1<Option<Percent>> = "*_1y",
    } }
    shape! { OpReturnFees at "series().op_return.fees" {
        block: Block = "*_fees",
        cumulative: AvgBalance = "*_fees_cumulative",
        sum: Sum = "*_fees_sum",
        chain_share: ChainShare = "*_fee_chain_share",
    } }
    shape! { Subsidy at "series().mining.rewards.subsidy" {
        block: Block = "*",
        cumulative: AvgBalance = "*_cumulative",
        sum: Sum = "*_sum",
        share: ChainShare = "*_share",
    } }
    shape! { BlocksHalving at "series().blocks.halving" {
        epoch: SeriesPattern1<Halving> = "*_epoch",
        blocks_to_halving: SeriesPattern1<Count> = "blocks_to_*",
        days_to_halving: SeriesPattern1<Option<Days>> = "days_to_*",
    } }
    shape! { BlocksDifficulty at "series().blocks.difficulty" {
        block: SeriesPattern1<Option<Difficulty>> = "*",
        hashrate: SeriesPattern1<Option<Hashrate>> = "*_hashrate",
        adjustment: SeriesPattern1<Option<Percent>> = "*_adjustment",
        epoch: SeriesPattern1<Epoch> = "*_epoch",
        blocks_to_retarget: SeriesPattern1<Count> = "blocks_to_retarget",
        days_to_retarget: SeriesPattern1<Option<Days>> = "days_to_retarget",
    } }
    shape! { InputsPerSecond<A> at "series().inputs.per_second" {
        _24h: SeriesPattern1<A> = "*_24h",
        _1w: SeriesPattern1<A> = "*_1w",
        _1m: SeriesPattern1<A> = "*_1m",
        _1y: SeriesPattern1<A> = "*_1y",
    } }
    shape! { New at "series().addresses.new" {
        block: SeriesPattern20<Count> = "*",
        sum: InputsPerSecond<Count> = "*_sum",
    } }
    shape! { Delta<A> at "series().addresses.funded.delta" {
        absolute: InputsPerSecond<A> = "*",
        rate: DeltaRate = "*",
    } }
    shape! { LthSupply at "series().holders.lth.supply" {
        total: AvgBalance = "*",
        in_profit: AvgBalance = "*_in_profit",
        in_loss: AvgBalance = "*_in_loss",
        delta: Delta<Option<Bitcoin>> = "*_delta",
    } }
    shape! { AllSupply at "series().holders.all.supply" {
        total: Total = "circulating_*",
        in_profit: AvgBalance = "*_in_profit",
        in_loss: AvgBalance = "*_in_loss",
        delta: Delta<Option<Bitcoin>> = "*_delta",
    } }
    shape! { Balances0satsSupply at "series().addresses.balances._0sats.supply" {
        total: AvgBalance = "*",
        delta: Delta<Option<Bitcoin>> = "*_delta",
        share: SeriesPattern1<Option<Percent>> = "*_share",
    } }
    shape! { RookieSupply at "series().entry.rookie.supply" {
        total: AvgBalance = "*",
        delta: Delta<Option<Bitcoin>> = "*_delta",
        share: SeriesPattern1<Option<Percent>> = "*_share",
        in_profit: AvgBalance = "*_in_profit",
        in_loss: AvgBalance = "*_in_loss",
    } }
    shape! { Funded<A, B> at "series().addresses.funded" {
        block: SeriesPattern1<A> = "*",
        delta: Delta<B> = "*_delta",
    } }
    shape! { Supply at "series().supply" {
        circulating: SeriesPattern1<Option<Bitcoin>> = "circulating_supply",
        burned: OpReturnValue = "burned",
        inflation_rate: SeriesPattern1<Option<Percent>> = "inflation_*",
        velocity: Velocity = "velocity",
        market_cap: Funded<Option<Dollars>, Option<Dollars>> = "market_cap",
        market_minus_realized_cap_growth_rate: InputsPerSecond<Option<Percent>> = "market_minus_realized_cap_growth_*",
    } }
    shape! { Balances0satsOutputs at "series().addresses.balances._0sats.outputs" {
        unspent_count: Funded<Count, CountSigned> = "*",
    } }
    shape! { Ranges10yTo12ySupply at "series().age.ranges._10y_to_12y.supply" {
        total: AvgBalance = "*_supply",
        delta: Delta<Option<Bitcoin>> = "*_supply_delta",
        share: SeriesPattern1<Option<Percent>> = "*_supply_share",
        in_profit: AvgBalance = "*_supply_in_profit",
        in_loss: AvgBalance = "*_supply_in_loss",
        matured: OpReturnValue = "*_matured_supply",
    } }
    shape! { Daily at "series().market.returns.daily" {
        avg: InputsPerSecond<Option<Percent>> = "*_avg",
        sd: InputsPerSecond<Option<Percent>> = "*_sd",
    } }
    shape! { Returns at "series().market.returns" {
        periods: Periods = "*_return",
        cagr: Cagr = "*_cagr",
        daily: Daily = "*_return_24h",
    } }
    shape! { Market at "series().market" {
        ath: Ath = "*",
        lookback: Lookback = "*",
        returns: Returns = "*",
        volatility: InputsPerSecond<Option<Percent>> = "*_volatility",
        range: Range = "*",
        sma: MarketSma = "*_sma",
        ema: Ema = "*_ema",
        rsi: Macd<Rsi1m> = "24h",
        pi_cycle: SeriesPattern1<Option<Ratio>> = "pi_cycle",
        macd: Macd<Macd1m> = "24h",
    } }
    shape! { BlocksMined at "series().pools.aaopool.blocks_mined" {
        cumulative: SeriesPattern1<Count> = "*_cumulative",
        sum: InputsPerSecond<Count> = "*_sum",
    } }
    shape! { Aaopool at "series().pools.aaopool" {
        blocks_mined: BlocksMined = "*_blocks_mined",
        share: ChainShare = "*_share",
        rewards: AaopoolRewards = "*_rewards",
        fees_per_block: Sum = "*_fees_per_block",
        hashrate: InputsPerSecond<Option<Hashrate>> = "*_hashrate",
    } }
    shape! { Pools at "series().pools" {
        pool: SeriesPattern20<PoolSlug> = "*",
        unknown: Aaopool = "unknown",
        blockfills: Aaopool = "blockfills",
        ultimuspool: Aaopool = "ultimuspool",
        terrapool: Aaopool = "terrapool",
        luxor: Aaopool = "luxor",
        onethash: Aaopool = "onethash",
        btccom: Aaopool = "btccom",
        bitfarms: Aaopool = "bitfarms",
        huobipool: Aaopool = "huobipool",
        wayicn: Aaopool = "wayicn",
        canoepool: Aaopool = "canoepool",
        btctop: Aaopool = "btctop",
        bitcoincom: Aaopool = "bitcoincom",
        pool175btc: Aaopool = "pool175btc",
        gbminers: Aaopool = "gbminers",
        axbt: Aaopool = "axbt",
        asicminer: Aaopool = "asicminer",
        bitminter: Aaopool = "bitminter",
        bitcoinrussia: Aaopool = "bitcoinrussia",
        btcserv: Aaopool = "btcserv",
        simplecoinus: Aaopool = "simplecoinus",
        btcguild: Aaopool = "btcguild",
        eligius: Aaopool = "eligius",
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
        f2pool: Aaopool = "f2pool",
        hhtt: Aaopool = "hhtt",
        megabigpower: Aaopool = "megabigpower",
        mtred: Aaopool = "mtred",
        nmcbit: Aaopool = "nmcbit",
        yourbtcnet: Aaopool = "yourbtcnet",
        givemecoins: Aaopool = "givemecoins",
        braiinspool: Aaopool = "braiinspool",
        antpool: Aaopool = "antpool",
        multicoinco: Aaopool = "multicoinco",
        bcpoolio: Aaopool = "bcpoolio",
        cointerra: Aaopool = "cointerra",
        kanopool: Aaopool = "kanopool",
        solock: Aaopool = "solock",
        ckpool: Aaopool = "ckpool",
        nicehash: Aaopool = "nicehash",
        bitclub: Aaopool = "bitclub",
        bitcoinaffiliatenetwork: Aaopool = "bitcoinaffiliatenetwork",
        btcc: Aaopool = "btcc",
        bwpool: Aaopool = "bwpool",
        exxbw: Aaopool = "exxbw",
        bitsolo: Aaopool = "bitsolo",
        bitfury: Aaopool = "bitfury",
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
        viabtc: Aaopool = "viabtc",
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
        poolin: Aaopool = "poolin",
        secretsuperstar: Aaopool = "secretsuperstar",
        tigerpoolnet: Aaopool = "tigerpoolnet",
        sigmapoolcom: Aaopool = "sigmapoolcom",
        okpooltop: Aaopool = "okpooltop",
        hummerpool: Aaopool = "hummerpool",
        tangpool: Aaopool = "tangpool",
        bytepool: Aaopool = "bytepool",
        spiderpool: Aaopool = "spiderpool",
        novablock: Aaopool = "novablock",
        miningcity: Aaopool = "miningcity",
        binancepool: Aaopool = "binancepool",
        minerium: Aaopool = "minerium",
        lubiancom: Aaopool = "lubiancom",
        okkong: Aaopool = "okkong",
        aaopool: Aaopool = "aaopool",
        emcdpool: Aaopool = "emcdpool",
        foundryusa: Aaopool = "foundryusa",
        sbicrypto: Aaopool = "sbicrypto",
        arkpool: Aaopool = "arkpool",
        purebtccom: Aaopool = "purebtccom",
        marapool: Aaopool = "marapool",
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
        secpool: Aaopool = "secpool",
        ocean: Aaopool = "ocean",
        whitepool: Aaopool = "whitepool",
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
    shape! { MultipleDataBytes at "series().op_return.policies.multiple.data_bytes" {
        block: SeriesPattern20<Bytes> = "*_bytes",
        cumulative: SeriesPattern1<Bytes> = "*_bytes_cumulative",
        sum: InputsPerSecond<Bytes> = "*_bytes_sum",
        share: SeriesPattern1<Option<Percent>> = "*_share",
        chain_share: SeriesPattern1<Option<Percent>> = "*_chain_share",
    } }
    shape! { OpReturnDataBytes at "series().op_return.data_bytes" {
        block: SeriesPattern20<Bytes> = "*_bytes",
        cumulative: SeriesPattern1<Bytes> = "*_bytes_cumulative",
        sum: InputsPerSecond<Bytes> = "*_bytes_sum",
        chain_share: SeriesPattern1<Option<Percent>> = "*_chain_share",
    } }
    shape! { RewardsFees at "series().mining.rewards.fees" {
        block: Block = "*",
        cumulative: AvgBalance = "*_cumulative",
        sum: Sum = "*_sum",
        avg: Sum = "*_avg",
        min: Sum = "*_min",
        max: Sum = "*_max",
        pct10: Sum = "*_pct10",
        pct25: Sum = "*_pct25",
        median: Sum = "*_median",
        pct75: Sum = "*_pct75",
        pct90: Sum = "*_pct90",
        share: ChainShare = "fee_share",
        to_subsidy: InputsPerSecond<Option<Ratio>> = "fee_to_subsidy",
    } }
    shape! { MiningRewards at "series().mining.rewards" {
        coinbase: OpReturnValue = "*",
        subsidy: Subsidy = "subsidy",
        fees: RewardsFees = "fees",
        unclaimed: OpReturnValue = "unclaimed_rewards",
    } }
    shape! { Mining at "series().mining" {
        rewards: MiningRewards = "*",
        hashrate: MiningHashrate = "hash",
    } }
    shape! { BlocksSize<A, B, C> at "series().blocks.size" {
        cumulative: SeriesPattern1<A> = "*_cumulative",
        sum: InputsPerSecond<A> = "*_sum",
        avg: InputsPerSecond<B> = "*_avg",
        min: InputsPerSecond<C> = "*_min",
        max: InputsPerSecond<C> = "*_max",
        pct10: InputsPerSecond<C> = "*_pct10",
        pct25: InputsPerSecond<C> = "*_pct25",
        median: InputsPerSecond<C> = "*_median",
        pct75: InputsPerSecond<C> = "*_pct75",
        pct90: InputsPerSecond<C> = "*_pct90",
    } }
    shape! { Vbytes<A, B, C> at "series().blocks.vbytes" {
        block: SeriesPattern20<A> = "*",
        cumulative: SeriesPattern1<A> = "*_cumulative",
        sum: InputsPerSecond<A> = "*_sum",
        avg: InputsPerSecond<B> = "*_avg",
        min: InputsPerSecond<C> = "*_min",
        max: InputsPerSecond<C> = "*_max",
        pct10: InputsPerSecond<C> = "*_pct10",
        pct25: InputsPerSecond<C> = "*_pct25",
        median: InputsPerSecond<C> = "*_median",
        pct75: InputsPerSecond<C> = "*_pct75",
        pct90: InputsPerSecond<C> = "*_pct90",
    } }
    shape! { Interval<A, B> at "series().blocks.interval" {
        block: SeriesPattern20<A> = "*",
        avg: InputsPerSecond<B> = "*_avg",
    } }
    shape! { AddressesActivity at "series().addresses.activity" {
        reactivated: Interval<Count, Option<CountFract>> = "*_reactivated_address_count",
        sending: Interval<Count, Option<CountFract>> = "*_sending_address_count",
        receiving: Interval<Count, Option<CountFract>> = "*_receiving_address_count",
        bidirectional: Interval<Count, Option<CountFract>> = "*_bidirectional_address_count",
        active: Interval<Count, Option<CountFract>> = "*_active_address_count",
    } }
    shape! { CoinblocksDestroyed<A> at "series().age.coinblocks_destroyed" {
        block: SeriesPattern20<A> = "*",
        cumulative: SeriesPattern1<A> = "*_cumulative",
        sum: InputsPerSecond<A> = "*_sum",
    } }
    shape! { ReserveRisk at "series().cointime.reserve_risk" {
        block: SeriesPattern1<Option<Float64>> = "reserve_risk",
        hodl_bank: SeriesPattern20<Option<Float64>> = "hodl_bank",
        vocdd: CoinblocksDestroyed<Option<Float64>> = "*",
        vocdd_median_1m: SeriesPattern20<Option<Float64>> = "*_median_1m",
    } }
    shape! { Value at "series().cointime.value" {
        destroyed: CoinblocksDestroyed<Option<Float64>> = "*_destroyed",
        created: CoinblocksDestroyed<Option<Float64>> = "*_created",
        stored: CoinblocksDestroyed<Option<Float64>> = "*_stored",
    } }
    shape! { CointimeAgeRanges10yTo12y at "series().cointime.age_ranges._10y_to_12y" {
        coindays_created: CoinblocksDestroyed<Option<CoinDays>> = "*_coindays_created",
        coindays_consumed: CoinblocksDestroyed<Option<CoinDays>> = "*_coindays_consumed",
        coindays_stored: CoinblocksDestroyed<Option<CoinDays>> = "*_coindays_stored",
        wakefulness: SeriesPattern1<Option<Ratio64>> = "*_wakefulness",
        awake_to_dormant: SeriesPattern1<Option<Ratio64>> = "*_awake_to_dormant",
        awake: Immobile = "*_awake_supply",
        dormant: Immobile = "*_dormant_supply",
    } }
    shape! { CointimeActivity at "series().cointime.activity" {
        coinblocks_created: CoinblocksDestroyed<Option<CoinBlocks>> = "*_created",
        coinblocks_destroyed: CoinblocksDestroyed<Option<CoinBlocks>> = "*_destroyed",
        coinblocks_stored: CoinblocksDestroyed<Option<CoinBlocks>> = "*_stored",
        liveliness: SeriesPattern1<Option<Ratio64>> = "liveliness",
        vaultedness: SeriesPattern1<Option<Ratio64>> = "vaultedness",
        liveliness_to_vaultedness: SeriesPattern1<Option<Ratio64>> = "liveliness_to_vaultedness",
        concurrent_liveliness: InputsPerSecond<Option<Ratio>> = "concurrent_liveliness",
    } }
    shape! { Sopr at "series().entry.rookie.realized.sopr" {
        _24h: SeriesPattern1<Option<Ratio>> = "*_sopr_24h",
        value_destroyed: CoinblocksDestroyed<Option<Dollars>> = "*_value_destroyed",
    } }
    shape! { RookieRealized at "series().entry.rookie.realized" {
        cap: Funded<Option<Dollars>, Option<Dollars>> = "*_realized_cap",
        price: Ema12d = "*_realized_price",
        profit: CoinblocksDestroyed<Option<Dollars>> = "*_realized_profit",
        loss: CoinblocksDestroyed<Option<Dollars>> = "*_realized_loss",
        net_pnl: CoinblocksDestroyed<Option<Dollars>> = "*_net_realized_pnl",
        sopr: Sopr = "*",
        mvrv: SeriesPattern1<Option<Ratio>> = "*_mvrv",
    } }
    shape! { AdjustedSopr at "series().holders.all.ratios.adjusted_sopr" {
        ratio: InputsPerSecond<Option<Ratio>> = "*_sopr",
        transfer_volume: CoinblocksDestroyed<Option<Dollars>> = "*_value_created",
        value_destroyed: CoinblocksDestroyed<Option<Dollars>> = "*_value_destroyed",
    } }
    shape! { Ratios at "series().holders.all.ratios" {
        adjusted_sopr: AdjustedSopr = "*_adjusted",
        dormancy: InputsPerSecond<Option<Days>> = "*_dormancy",
        sopr: SeriesPattern1<Option<Ratio>> = "*_sopr_24h",
        sopr_ratio_extended: SoprRatioExtended = "*_sopr",
        sell_side_risk_ratio: InputsPerSecond<Option<Ratio>> = "*_sell_side_risk_ratio",
        profit_to_loss_ratio: InputsPerSecond<Option<Ratio>> = "*_realized_profit_to_loss_ratio",
    } }
    shape! { AllRealized at "series().holders.all.realized" {
        cap: Funded<Option<Dollars>, Option<Dollars>> = "*_realized_cap",
        price: SeriesPattern1<Option<Dollars>> = "*_realized_price",
        capitalized_price: Ema12d = "*_capitalized_price",
        profit: CoinblocksDestroyed<Option<Dollars>> = "*_realized_profit",
        loss: CoinblocksDestroyed<Option<Dollars>> = "*_realized_loss",
        net_pnl: CoinblocksDestroyed<Option<Dollars>> = "*_net_realized_pnl",
        value_destroyed: CoinblocksDestroyed<Option<Dollars>> = "*_value_destroyed",
        gross_pnl: CoinblocksDestroyed<Option<Dollars>> = "*_realized_gross_pnl",
        peak_regret: CoinblocksDestroyed<Option<Dollars>> = "*_realized_peak_regret",
        mvrv: SeriesPattern1<Option<Ratio>> = "*_mvrv",
    } }
    shape! { AllActivity at "series().holders.all.activity" {
        transfer_volume: OpReturnValue = "*_transfer_volume",
        transfer_volume_in_profit: OpReturnValue = "*_transfer_volume_in_profit",
        transfer_volume_in_loss: OpReturnValue = "*_transfer_volume_in_loss",
        coindays_destroyed: CoinblocksDestroyed<Option<CoinDays>> = "*_coindays_destroyed",
        coinyears_destroyed: SeriesPattern1<Option<CoinYears>> = "*_coinyears_destroyed",
    } }
    shape! { Balances0satsRealized at "series().addresses.balances._0sats.realized" {
        cap: SeriesPattern1<Option<Dollars>> = "*_cap",
        profit: CoinblocksDestroyed<Option<Dollars>> = "*_profit",
        loss: CoinblocksDestroyed<Option<Dollars>> = "*_loss",
    } }
    shape! { Balances0sats at "series().addresses.balances._0sats" {
        address_count: Funded<Count, CountSigned> = "*_address_count",
        supply: Balances0satsSupply = "*_supply",
        outputs: Balances0satsOutputs = "*_utxo_count",
        activity: Balances0satsActivity = "*_transfer_volume",
        realized: Balances0satsRealized = "*_realized",
    } }
    shape! { EventsActive<A> at "series().addresses.respent.events.active" {
        count: A = "*_count",
        share: ChainShare = "*_share",
    } }
    shape! { RespentEvents at "series().addresses.respent.events" {
        outputs: EventsActive<CoinblocksDestroyed<Count>> = "*_output_to_respent_address",
        inputs: EventsActive<CoinblocksDestroyed<Count>> = "*_input_from_respent_address",
        active: EventsActive<Interval<Count, Option<CountFract>>> = "*_active_respent_address",
    } }
    shape! { Respent at "series().addresses.respent" {
        count: RespentCount = "*",
        supply: ExposedSupply = "*_respent_address_supply",
        events: RespentEvents = "*",
    } }
    shape! { ReusedEvents at "series().addresses.reused.events" {
        outputs: EventsActive<CoinblocksDestroyed<Count>> = "*_output_to_reused_address",
        inputs: EventsActive<CoinblocksDestroyed<Count>> = "*_input_from_reused_address",
        active: EventsActive<Interval<Count, Option<CountFract>>> = "*_active_reused_address",
    } }
    shape! { Reused at "series().addresses.reused" {
        count: ReusedCount = "*",
        supply: ExposedSupply = "*_reused_address_supply",
        events: ReusedEvents = "*",
    } }
    shape! { P2pk at "series().addresses.types.p2pk" {
        funded: Funded<Count, CountSigned> = "*_address_count",
        empty: SeriesPattern1<Count> = "*_empty_address_count",
        total: SeriesPattern1<Count> = "*_total_address_count",
        new: New = "*_new_address_count",
        activity: AddressesActivity = "*",
        avg_balance: AvgBalance = "*_avg_address_balance",
        reused: Reused = "*",
        respent: Respent = "*",
        exposed: Exposed = "*",
    } }
    shape! { AddressesTypes at "series().addresses.types" {
        p2pk: P2pk = "*",
        p2pkh: P2pk = "p2pkh",
        p2sh: P2pk = "p2sh",
        p2wpkh: P2pk = "p2wpkh",
        p2wsh: P2pk = "p2wsh",
        p2tr: P2pk = "p2tr",
    } }
    shape! { EmptyOutputs at "series().utxos.types.empty.outputs" {
        unspent_count: Funded<Count, CountSigned> = "*_utxo_count",
        spent_count: CoinblocksDestroyed<Count> = "*_spent_utxo_count",
        avg_amount: AvgBalance = "*_avg_utxo_amount",
    } }
    shape! { Amounts0satsRealized at "series().utxos.amounts._0sats.realized" {
        cap: SeriesPattern1<Option<Dollars>> = "*_cap",
        price: SeriesPattern1<Option<Dollars>> = "*_price",
        profit: CoinblocksDestroyed<Option<Dollars>> = "*_profit",
        loss: CoinblocksDestroyed<Option<Dollars>> = "*_loss",
    } }
    shape! { Classes2009Realized at "series().age.classes._2009.realized" {
        cap: SeriesPattern1<Option<Dollars>> = "*_realized_cap",
        profit: CoinblocksDestroyed<Option<Dollars>> = "*_realized_profit",
        loss: CoinblocksDestroyed<Option<Dollars>> = "*_realized_loss",
        net_pnl: CoinblocksDestroyed<Option<Dollars>> = "*_net_realized_pnl",
        value_destroyed: CoinblocksDestroyed<Option<Dollars>> = "*_value_destroyed",
    } }
    shape! { RookieActivity at "series().entry.rookie.activity" {
        transfer_volume: TransferVolume = "*_transfer_volume",
        coindays_destroyed: CoinblocksDestroyed<Option<CoinDays>> = "*_coindays_destroyed",
    } }
    shape! { Ranges10yTo12yRealized at "series().age.ranges._10y_to_12y.realized" {
        cap: SeriesPattern1<Option<Dollars>> = "*_realized_cap",
        profit: CoinblocksDestroyed<Option<Dollars>> = "*_realized_profit",
        loss: CoinblocksDestroyed<Option<Dollars>> = "*_realized_loss",
        net_pnl: CoinblocksDestroyed<Option<Dollars>> = "*_net_realized_pnl",
        value_destroyed: CoinblocksDestroyed<Option<Dollars>> = "*_value_destroyed",
        price: SeriesPattern1<Option<Dollars>> = "*_realized_price",
    } }
    shape! { Ranges10yTo12yActivity at "series().age.ranges._10y_to_12y.activity" {
        transfer_volume: TransferVolume = "*_transfer_volume",
        coindays_destroyed: CoinblocksDestroyed<Option<CoinDays>> = "*_coindays_destroyed",
        coindays_created: CoinblocksDestroyed<Option<CoinDays>> = "*_coindays_created",
    } }
    shape! { RookieOutputs at "series().entry.rookie.outputs" {
        unspent_count: Funded<Count, CountSigned> = "*_utxo_count",
        spent_count: CoinblocksDestroyed<Count> = "*_spent_utxo_count",
    } }
    shape! { HoldersAll<A> at "series().holders.all" {
        supply: A = "*_supply",
        outputs: RookieOutputs = "*",
        activity: AllActivity = "*",
        realized: AllRealized = "*",
        unrealized: AllUnrealized = "*",
        cost_basis: AllCostBasis = "*",
        ratios: Ratios = "*",
        relative: Relative = "*",
    } }
    shape! { Holders<A, B> at "series().holders" {
        all: A = "",
        sth: B = "sth",
        lth: B = "lth",
        under_4m: B = "*_under_4m_old",
        under_6m: B = "*_under_6m_old",
        over_4m: B = "*_over_4m_old",
        over_6m: B = "*_over_6m_old",
    } }
    shape! { Amounts0sats<A> at "series().utxos.amounts._0sats" {
        supply: Balances0satsSupply = "*_supply",
        outputs: A = "*",
        activity: Balances0satsActivity = "*_transfer_volume",
        realized: Amounts0satsRealized = "*_realized",
    } }
    shape! { UtxosTypes at "series().utxos.types" {
        p2pk: Amounts0sats<EmptyOutputs> = "p2pk",
        p2pkh: Amounts0sats<EmptyOutputs> = "p2pkh",
        p2ms: Amounts0sats<EmptyOutputs> = "p2ms",
        p2sh: Amounts0sats<EmptyOutputs> = "p2sh",
        p2wpkh: Amounts0sats<EmptyOutputs> = "p2wpkh",
        p2wsh: Amounts0sats<EmptyOutputs> = "p2wsh",
        p2tr: Amounts0sats<EmptyOutputs> = "p2tr",
        p2a: Amounts0sats<EmptyOutputs> = "p2a",
        unknown: Amounts0sats<EmptyOutputs> = "unknown_*",
        empty: Amounts0sats<EmptyOutputs> = "empty_*",
    } }
    shape! { Balances<A> at "series().addresses.balances" {
        _0sats: A = "*_0sats",
        _1sat_to_10sats: A = "*_1sat_to_10sats",
        _10sats_to_100sats: A = "*_10sats_to_100sats",
        _100sats_to_1k_sats: A = "*_100sats_to_1k_sats",
        _1k_sats_to_10k_sats: A = "*_1k_sats_to_10k_sats",
        _10k_sats_to_100k_sats: A = "*_10k_sats_to_100k_sats",
        _100k_sats_to_1m_sats: A = "*_100k_sats_to_1m_sats",
        _1m_sats_to_10m_sats: A = "*_1m_sats_to_10m_sats",
        _10m_sats_to_1btc: A = "*_10m_sats_to_1btc",
        _1btc_to_10btc: A = "*_1btc_to_10btc",
        _10btc_to_100btc: A = "*_10btc_to_100btc",
        _100btc_to_1k_btc: A = "*_100btc_to_1k_btc",
        _1k_btc_to_10k_btc: A = "*_1k_btc_to_10k_btc",
        _10k_btc_to_100k_btc: A = "*_10k_btc_to_100k_btc",
        over_100k_btc: A = "*_over_100k_btc",
    } }
    shape! { Addresses at "series().addresses" {
        funded: Funded<Count, CountSigned> = "*_count",
        empty: SeriesPattern1<Count> = "empty_*_count",
        total: SeriesPattern1<Count> = "total_*_count",
        new: New = "new_*_count",
        activity: AddressesActivity = "",
        avg_balance: AvgBalance = "avg_*_balance",
        reused: Reused = "",
        respent: Respent = "",
        exposed: Exposed = "",
        types: AddressesTypes = "p2pk",
        balances: Balances<Balances0sats> = "balance",
        state: State = "*",
    } }
    shape! { Utxos at "series().utxos" {
        amounts: Balances<Amounts0sats<RookieOutputs>> = "*",
        types: UtxosTypes = "output",
        avg_amount: AvgBalance = "avg_utxo_amount",
    } }
    shape! { Rookie<A, B> at "series().entry.rookie" {
        supply: RookieSupply = "*_supply",
        outputs: RookieOutputs = "*",
        activity: RookieActivity = "*",
        realized: A = "*",
        unrealized: B = "*",
    } }
    shape! { Entry at "series().entry" {
        veteran: Rookie<RookieRealized, RookieUnrealized> = "*",
        rookie: Rookie<RookieRealized, RookieUnrealized> = "rookie",
    } }
    shape! { Classes at "series().age.classes" {
        _2009: Rookie<Classes2009Realized, Classes2009Unrealized> = "*_2009",
        _2010: Rookie<Classes2009Realized, Classes2009Unrealized> = "*_2010",
        _2011: Rookie<Classes2009Realized, Classes2009Unrealized> = "*_2011",
        _2012: Rookie<Classes2009Realized, Classes2009Unrealized> = "*_2012",
        _2013: Rookie<Classes2009Realized, Classes2009Unrealized> = "*_2013",
        _2014: Rookie<Classes2009Realized, Classes2009Unrealized> = "*_2014",
        _2015: Rookie<Classes2009Realized, Classes2009Unrealized> = "*_2015",
        _2016: Rookie<Classes2009Realized, Classes2009Unrealized> = "*_2016",
        _2017: Rookie<Classes2009Realized, Classes2009Unrealized> = "*_2017",
        _2018: Rookie<Classes2009Realized, Classes2009Unrealized> = "*_2018",
        _2019: Rookie<Classes2009Realized, Classes2009Unrealized> = "*_2019",
        _2020: Rookie<Classes2009Realized, Classes2009Unrealized> = "*_2020",
        _2021: Rookie<Classes2009Realized, Classes2009Unrealized> = "*_2021",
        _2022: Rookie<Classes2009Realized, Classes2009Unrealized> = "*_2022",
        _2023: Rookie<Classes2009Realized, Classes2009Unrealized> = "*_2023",
        _2024: Rookie<Classes2009Realized, Classes2009Unrealized> = "*_2024",
        _2025: Rookie<Classes2009Realized, Classes2009Unrealized> = "*_2025",
        _2026: Rookie<Classes2009Realized, Classes2009Unrealized> = "*_2026",
    } }
    shape! { Epochs at "series().age.epochs" {
        _0: Rookie<Classes2009Realized, Classes2009Unrealized> = "*_0",
        _1: Rookie<Classes2009Realized, Classes2009Unrealized> = "*_1",
        _2: Rookie<Classes2009Realized, Classes2009Unrealized> = "*_2",
        _3: Rookie<Classes2009Realized, Classes2009Unrealized> = "*_3",
        _4: Rookie<Classes2009Realized, Classes2009Unrealized> = "*_4",
    } }
    shape! { Ranges10yTo12y at "series().age.ranges._10y_to_12y" {
        supply: Ranges10yTo12ySupply = "*",
        outputs: RookieOutputs = "*",
        activity: Ranges10yTo12yActivity = "*",
        realized: Ranges10yTo12yRealized = "*",
        unrealized: Classes2009Unrealized = "*",
    } }
    shape! { Ranges<A> at "series().age.ranges" {
        under_1h: A = "*_under_1h_old",
        _1h_to_1d: A = "*_1h_to_1d_old",
        _1d_to_1w: A = "*_1d_to_1w_old",
        _1w_to_1m: A = "*_1w_to_1m_old",
        _1m_to_2m: A = "*_1m_to_2m_old",
        _2m_to_3m: A = "*_2m_to_3m_old",
        _3m_to_4m: A = "*_3m_to_4m_old",
        _4m_to_5m: A = "*_4m_to_5m_old",
        _5m_to_6m: A = "*_5m_to_6m_old",
        _6m_to_9m: A = "*_6m_to_9m_old",
        _9m_to_1y: A = "*_9m_to_1y_old",
        _1y_to_18m: A = "*_1y_to_18m_old",
        _18m_to_2y: A = "*_18m_to_2y_old",
        _2y_to_3y: A = "*_2y_to_3y_old",
        _3y_to_4y: A = "*_3y_to_4y_old",
        _4y_to_5y: A = "*_4y_to_5y_old",
        _5y_to_6y: A = "*_5y_to_6y_old",
        _6y_to_7y: A = "*_6y_to_7y_old",
        _7y_to_8y: A = "*_7y_to_8y_old",
        _8y_to_10y: A = "*_8y_to_10y_old",
        _10y_to_12y: A = "*_10y_to_12y_old",
        _12y_to_15y: A = "*_12y_to_15y_old",
        over_15y: A = "*_over_15y_old",
    } }
    shape! { Coinflow at "series().coinflow" {
        age_ranges: Ranges<CoinflowAgeRanges10yTo12y> = "*",
        cohorts: Holders<CoinflowCohortsAll, CoinflowCohortsAll> = "*",
    } }
    shape! { Cointime at "series().cointime" {
        activity: CointimeActivity = "coinblocks",
        age_ranges: Ranges<CointimeAgeRanges10yTo12y> = "*",
        cohorts: Holders<CointimeCohortsAll, CointimeCohortsAll> = "*",
        supply: CointimeSupply = "supply",
        value: Value = "cointime_value",
        caps: Caps = "cap",
        prices: CointimePrices = "vaulted",
        adjusted: Adjusted = "cointime_adjusted",
        reserve_risk: ReserveRisk = "vocdd",
    } }
    shape! { Age at "series().age" {
        ranges: Ranges<Ranges10yTo12y> = "*",
        epochs: Epochs = "epoch",
        classes: Classes = "class",
        coinblocks_destroyed: CoinblocksDestroyed<Option<CoinBlocks>> = "coinblocks_destroyed",
    } }
    shape! { Multiple at "series().op_return.policies.multiple" {
        output_count: CoinblocksDestroyed<Count> = "*_output_count",
        data_bytes: MultipleDataBytes = "*_data",
        tx_count: CoinblocksDestroyed<Count> = "*_tx_count",
        tx_vsize: CoinblocksDestroyed<VSize> = "*_tx_vsize",
        fees: OpReturnFees = "*",
    } }
    shape! { Policies at "series().op_return.policies" {
        pre_v30_standard: Multiple = "*_pre_v30_standard",
        pre_v30_nonstandard: Multiple = "*_pre_v30_nonstandard",
        oversized: Multiple = "*_oversized",
        multiple: Multiple = "*_multiple",
    } }
    shape! { Protocols at "series().op_return.protocols" {
        runes: Multiple = "*_runes",
        veri_block: Multiple = "*_veri_block",
        omni: Multiple = "*_omni",
        stacks: Multiple = "*_stacks",
        blockstack: Multiple = "*_blockstack",
        colu: Multiple = "*_colu",
        open_assets: Multiple = "*_open_assets",
        komodo: Multiple = "*_komodo",
        coin_spark: Multiple = "*_coin_spark",
        poet: Multiple = "*_poet",
        docproof: Multiple = "*_docproof",
        open_timestamps: Multiple = "*_open_timestamps",
        factom: Multiple = "*_factom",
        eternity_wall: Multiple = "*_eternity_wall",
        memo: Multiple = "*_memo",
        bitproof: Multiple = "*_bitproof",
        ascribe: Multiple = "*_ascribe",
        stampery: Multiple = "*_stampery",
        epobc: Multiple = "*_epobc",
        bare_hash: Multiple = "*_bare_hash",
        text: Multiple = "*_text",
        empty: Multiple = "*_empty",
        unknown: Multiple = "*_unknown",
    } }
    shape! { OpReturn at "series().op_return" {
        output_count: CoinblocksDestroyed<Count> = "*_output_count",
        data_bytes: OpReturnDataBytes = "*_data",
        tx_count: CoinblocksDestroyed<Count> = "*_tx_count",
        tx_vsize: CoinblocksDestroyed<VSize> = "*_tx_vsize",
        fees: OpReturnFees = "*",
        protocols: Protocols = "*",
        policies: Policies = "*",
    } }
    shape! { Versions at "series().transactions.versions" {
        v1: CoinblocksDestroyed<Count> = "v1_*",
        v2: CoinblocksDestroyed<Count> = "v2_*",
        v3: CoinblocksDestroyed<Count> = "v3_*",
        other: CoinblocksDestroyed<Count> = "other_version_*",
    } }
    shape! { Sigops at "series().transactions.sigops" {
        total: CoinblocksDestroyed<SigOps64> = "*",
    } }
    shape! { PolicyCount at "series().transactions.policy.count" {
        nonstandard: CoinblocksDestroyed<Count> = "*",
    } }
    shape! { Policy at "series().transactions.policy" {
        count: PolicyCount = "*_tx_count",
        is_nonstandard: SeriesPattern21<Boolean> = "is_*",
    } }
    shape! { PatternsCount at "series().transactions.patterns.count" {
        coinjoin: CoinblocksDestroyed<Count> = "coinjoin_*",
        consolidation: CoinblocksDestroyed<Count> = "consolidation_*",
        batch_payout: CoinblocksDestroyed<Count> = "batch_payout_*",
    } }
    shape! { Patterns at "series().transactions.patterns" {
        count: PatternsCount = "tx_count",
        is_coinjoin: SeriesPattern21<Boolean> = "*_coinjoin",
        is_consolidation: SeriesPattern21<Boolean> = "*_consolidation",
        is_batch_payout: SeriesPattern21<Boolean> = "*_batch_payout",
    } }
    shape! { Inscription at "series().transactions.inscription" {
        count: CoinblocksDestroyed<Count> = "*_tx_count",
        fees: InscriptionFees = "*",
    } }
    shape! { FeesCount at "series().transactions.fees.count" {
        cpfp_parent: CoinblocksDestroyed<Count> = "cpfp_parent_*",
        cpfp_child: CoinblocksDestroyed<Count> = "cpfp_child_*",
    } }
    shape! { TransactionsFees at "series().transactions.fees" {
        count: FeesCount = "tx_count",
        fee: EffectiveFeeRate<Sats, EffectiveFeeRate6b<Sats>> = "*",
        fee_rate: SeriesPattern21<Option<FeeRate>> = "*_rate",
        effective_fee_rate: EffectiveFeeRate<Option<FeeRate>, EffectiveFeeRate6b<Option<FeeRate>>> = "effective_*_rate",
        is_cpfp_parent: SeriesPattern21<Boolean> = "is_cpfp_parent",
        is_cpfp_child: SeriesPattern21<Boolean> = "is_cpfp_child",
    } }
    shape! { TransactionsFeatures at "series().transactions.features" {
        annex: CoinblocksDestroyed<Count> = "annex_*",
        sighash_all: CoinblocksDestroyed<Count> = "sighash_all_*",
        sighash_none: CoinblocksDestroyed<Count> = "sighash_none_*",
        sighash_single: CoinblocksDestroyed<Count> = "sighash_single_*",
        sighash_default: CoinblocksDestroyed<Count> = "sighash_default_*",
        sighash_anyone_can_pay: CoinblocksDestroyed<Count> = "sighash_anyone_can_pay_*",
        dust_output: CoinblocksDestroyed<Count> = "dust_output_*",
    } }
    shape! { Transactions at "series().transactions" {
        count: Vbytes<Count, Option<CountFract>, Count16> = "*_count",
        features: TransactionsFeatures = "*_count",
        size: TransactionsSize = "*",
        fees: TransactionsFees = "fee",
        inscription: Inscription = "inscription",
        patterns: Patterns = "is",
        policy: Policy = "nonstandard",
        sigops: Sigops = "total_sigop_cost",
        versions: Versions = "*_count",
        volume: OpReturnValue = "*_volume",
        per_second: InputsPerSecond<Option<PerSecond>> = "*_per_second",
    } }
    shape! { TypesEmpty at "series().inputs.types.empty" {
        count: CoinblocksDestroyed<Count> = "*_count",
        share: ChainShare = "*_share",
        tx_count: CoinblocksDestroyed<Count> = "*_tx_count",
        tx_share: ChainShare = "*_tx_share",
    } }
    shape! { OutputsTypes at "series().outputs.types" {
        p2pk: TypesEmpty = "p2pk_*",
        p2pkh: TypesEmpty = "p2pkh_*",
        p2ms: TypesEmpty = "p2ms_*",
        p2sh: TypesEmpty = "p2sh_*",
        p2wpkh: TypesEmpty = "p2wpkh_*",
        p2wsh: TypesEmpty = "p2wsh_*",
        p2tr: TypesEmpty = "p2tr_*",
        p2a: TypesEmpty = "p2a_*",
        unknown: TypesEmpty = "unknown_*",
        empty: TypesEmpty = "empty_*",
        op_return: TypesEmpty = "op_return_*",
    } }
    shape! { Outputs at "series().outputs" {
        spent: Spent = "txin_index",
        count: Vbytes<Count, Option<CountFract>, Count32> = "*_count",
        per_second: InputsPerSecond<Option<PerSecond>> = "outputs_per_second",
        spendable_count: CoinblocksDestroyed<Count> = "spendable_*_count",
        types: OutputsTypes = "*",
        op_return_value: OpReturnValue = "op_return_value",
    } }
    shape! { InputsTypes at "series().inputs.types" {
        p2pk: TypesEmpty = "p2pk_*",
        p2pkh: TypesEmpty = "p2pkh_*",
        p2ms: TypesEmpty = "p2ms_*",
        p2sh: TypesEmpty = "p2sh_*",
        p2wpkh: TypesEmpty = "p2wpkh_*",
        p2wsh: TypesEmpty = "p2wsh_*",
        p2tr: TypesEmpty = "p2tr_*",
        p2a: TypesEmpty = "p2a_*",
        unknown: TypesEmpty = "unknown_*",
        empty: TypesEmpty = "empty_*",
    } }
    shape! { Inputs at "series().inputs" {
        value: SeriesPattern22<Sats> = "value",
        count: Vbytes<Count, Option<CountFract>, Count16> = "*_count",
        per_second: InputsPerSecond<Option<PerSecond>> = "inputs_per_second",
        types: InputsTypes = "*",
    } }
    shape! { Blocks at "series().blocks" {
        count: CoinblocksDestroyed<Count> = "*_count",
        interval: Interval<Seconds, Option<SecondsFract>> = "*_interval",
        vbytes: Vbytes<VSize, Option<VSizeFract>, VSize> = "*_vbytes",
        size: BlocksSize<Bytes, Option<BytesFract>, Bytes32> = "*_size",
        weight: BlocksSize<Weight64, Option<WeightFract>, Weight> = "*_weight",
        fullness: SeriesPattern20<Option<Percent>> = "*_fullness",
        difficulty: BlocksDifficulty = "difficulty",
        halving: BlocksHalving = "halving",
    } }
    shape! { Split at "series().price.split" {
        open: SeriesPattern3<Option<Dollars>> = "*_open",
        high: SeriesPattern3<Option<Dollars>> = "*_high",
        low: SeriesPattern3<Option<Dollars>> = "*_low",
        close: SeriesPattern4<Option<Dollars>> = "*_close",
    } }
    shape! { Price at "series().price" {
        split: Split = "*",
        ohlc: SeriesPattern3<OHLCDollars> = "*_ohlc",
        spot: SeriesPattern1<Option<Dollars>> = "*",
        sats_per_dollar: SeriesPattern1<Sats> = "sats_per_dollar",
    } }
    shape! { MappingsTimestamp at "series().mappings.timestamp" {
        monotonic: SeriesPattern20<Timestamp> = "*_monotonic",
        resolutions: SeriesPattern3<Timestamp> = "*",
    } }
    shape! { TxoutIndex at "series().mappings.txout_index" {
        identity: SeriesPattern23<TxOutIndex> = "*",
    } }
    shape! { TxinIndex at "series().mappings.txin_index" {
        identity: SeriesPattern22<TxInIndex> = "*",
    } }
    shape! { MappingsTxIndex at "series().mappings.tx_index" {
        identity: SeriesPattern21<TxIndex> = "*_index",
        input_count: SeriesPattern21<Count> = "*_input_count",
        output_count: SeriesPattern21<Count> = "*_output_count",
    } }
    shape! { MappingsYear10 at "series().mappings.year10" {
        date: SeriesPattern17<Date> = "*",
        first_height: SeriesPattern17<Height> = "first_height",
    } }
    shape! { MappingsYear1 at "series().mappings.year1" {
        date: SeriesPattern16<Date> = "*",
        first_height: SeriesPattern16<Height> = "first_height",
    } }
    shape! { MappingsMonth6 at "series().mappings.month6" {
        date: SeriesPattern15<Date> = "*",
        first_height: SeriesPattern15<Height> = "first_height",
    } }
    shape! { MappingsMonth3 at "series().mappings.month3" {
        date: SeriesPattern14<Date> = "*",
        first_height: SeriesPattern14<Height> = "first_height",
    } }
    shape! { MappingsMonth1 at "series().mappings.month1" {
        date: SeriesPattern13<Date> = "*",
        first_height: SeriesPattern13<Height> = "first_height",
    } }
    shape! { MappingsWeek1 at "series().mappings.week1" {
        date: SeriesPattern12<Date> = "*",
        first_height: SeriesPattern12<Height> = "first_height",
    } }
    shape! { MappingsDay3 at "series().mappings.day3" {
        date: SeriesPattern11<Date> = "*",
        first_height: SeriesPattern11<Height> = "first_height",
    } }
    shape! { MappingsDay1 at "series().mappings.day1" {
        date: SeriesPattern10<Date> = "*",
        first_height: SeriesPattern10<Height> = "first_height",
    } }
    shape! { MappingsHour12 at "series().mappings.hour12" {
        first_height: SeriesPattern9<Height> = "*",
    } }
    shape! { MappingsHour4 at "series().mappings.hour4" {
        first_height: SeriesPattern8<Height> = "*",
    } }
    shape! { MappingsHour1 at "series().mappings.hour1" {
        first_height: SeriesPattern7<Height> = "*",
    } }
    shape! { MappingsMinute30 at "series().mappings.minute30" {
        first_height: SeriesPattern6<Height> = "*",
    } }
    shape! { MappingsMinute10 at "series().mappings.minute10" {
        first_height: SeriesPattern5<Height> = "*",
    } }
    shape! { MappingsHalving at "series().mappings.halving" {
        first_height: SeriesPattern18<Height> = "*",
    } }
    shape! { MappingsEpoch at "series().mappings.epoch" {
        first_height: SeriesPattern19<Height> = "*",
    } }
    shape! { MappingsHeight at "series().mappings.height" {
        minute10: SeriesPattern20<Minute10> = "*",
        minute30: SeriesPattern20<Minute30> = "minute30",
        hour1: SeriesPattern20<Hour1> = "hour1",
        hour4: SeriesPattern20<Hour4> = "hour4",
        hour12: SeriesPattern20<Hour12> = "hour12",
        day1: SeriesPattern20<Day1> = "day1",
        day3: SeriesPattern20<Day3> = "day3",
        epoch: SeriesPattern20<Epoch> = "epoch",
        halving: SeriesPattern20<Halving> = "halving",
        week1: SeriesPattern20<Week1> = "week1",
        month1: SeriesPattern20<Month1> = "month1",
        month3: SeriesPattern20<Month3> = "month3",
        month6: SeriesPattern20<Month6> = "month6",
        year1: SeriesPattern20<Year1> = "year1",
        year10: SeriesPattern20<Year10> = "year10",
    } }
    shape! { AddrOpReturn at "series().mappings.addr.op_return" {
        identity: SeriesPattern25<OpReturnIndex> = "*",
    } }
    shape! { AddrUnknown at "series().mappings.addr.unknown" {
        identity: SeriesPattern35<UnknownOutputIndex> = "*",
    } }
    shape! { AddrEmpty at "series().mappings.addr.empty" {
        identity: SeriesPattern24<EmptyOutputIndex> = "*",
    } }
    shape! { AddrP2ms at "series().mappings.addr.p2ms" {
        identity: SeriesPattern27<P2MSOutputIndex> = "*",
    } }
    shape! { AddrP2a at "series().mappings.addr.p2a" {
        identity: SeriesPattern26<P2AAddrIndex> = "*_index",
        addr: SeriesPattern26<Addr> = "*",
    } }
    shape! { AddrP2wsh at "series().mappings.addr.p2wsh" {
        identity: SeriesPattern34<P2WSHAddrIndex> = "*_index",
        addr: SeriesPattern34<Addr> = "*",
    } }
    shape! { AddrP2wpkh at "series().mappings.addr.p2wpkh" {
        identity: SeriesPattern33<P2WPKHAddrIndex> = "*_index",
        addr: SeriesPattern33<Addr> = "*",
    } }
    shape! { AddrP2tr at "series().mappings.addr.p2tr" {
        identity: SeriesPattern32<P2TRAddrIndex> = "*_index",
        addr: SeriesPattern32<Addr> = "*",
    } }
    shape! { AddrP2sh at "series().mappings.addr.p2sh" {
        identity: SeriesPattern31<P2SHAddrIndex> = "*_index",
        addr: SeriesPattern31<Addr> = "*",
    } }
    shape! { AddrP2pkh at "series().mappings.addr.p2pkh" {
        identity: SeriesPattern30<P2PKHAddrIndex> = "*_index",
        addr: SeriesPattern30<Addr> = "*",
    } }
    shape! { AddrP2pk65 at "series().mappings.addr.p2pk65" {
        identity: SeriesPattern29<P2PK65AddrIndex> = "*_index",
        addr: SeriesPattern29<Addr> = "*",
    } }
    shape! { AddrP2pk33 at "series().mappings.addr.p2pk33" {
        identity: SeriesPattern28<P2PK33AddrIndex> = "*_index",
        addr: SeriesPattern28<Addr> = "*",
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
        tx_index: MappingsTxIndex = "tx",
        txin_index: TxinIndex = "txin_index",
        txout_index: TxoutIndex = "txout_index",
        timestamp: MappingsTimestamp = "timestamp",
    } }
    shape! { IndexerOpReturn at "series().indexer.op_return" {
        first_index: SeriesPattern20<OpReturnIndex> = "first_op_return_*",
        to_tx_index: SeriesPattern25<TxIndex> = "tx_*",
        kind: SeriesPattern25<OpReturnKind> = "kind",
        post_op_return_bytes: SeriesPattern25<Bytes32> = "op_return_post_op_return_bytes",
    } }
    shape! { ScriptsUnknown at "series().indexer.scripts.unknown" {
        first_index: SeriesPattern20<UnknownOutputIndex> = "first_unknown_output_*",
        to_tx_index: SeriesPattern35<TxIndex> = "tx_*",
        legacy_sigops: SeriesPattern35<SigOps> = "unknown_legacy_sigops",
    } }
    shape! { ScriptsP2ms at "series().indexer.scripts.p2ms" {
        first_index: SeriesPattern20<P2MSOutputIndex> = "first_p2ms_output_*",
        to_tx_index: SeriesPattern27<TxIndex> = "tx_*",
        legacy_sigops: SeriesPattern27<SigOps> = "p2ms_legacy_sigops",
    } }
    shape! { ScriptsEmpty at "series().indexer.scripts.empty" {
        first_index: SeriesPattern20<EmptyOutputIndex> = "first_empty_output_*",
        to_tx_index: SeriesPattern24<TxIndex> = "tx_*",
    } }
    shape! { Scripts at "series().indexer.scripts" {
        empty: ScriptsEmpty = "*",
        p2ms: ScriptsP2ms = "*",
        unknown: ScriptsUnknown = "*",
    } }
    shape! { AddressesP2a at "series().indexer.addresses.p2a" {
        first_index: SeriesPattern20<P2AAddrIndex> = "first_*_addr_index",
        bytes: SeriesPattern26<P2ABytes> = "*_bytes",
    } }
    shape! { AddressesP2tr at "series().indexer.addresses.p2tr" {
        first_index: SeriesPattern20<P2TRAddrIndex> = "first_*_addr_index",
        bytes: SeriesPattern32<P2TRBytes> = "*_bytes",
    } }
    shape! { AddressesP2wsh at "series().indexer.addresses.p2wsh" {
        first_index: SeriesPattern20<P2WSHAddrIndex> = "first_*_addr_index",
        bytes: SeriesPattern34<P2WSHBytes> = "*_bytes",
    } }
    shape! { AddressesP2wpkh at "series().indexer.addresses.p2wpkh" {
        first_index: SeriesPattern20<P2WPKHAddrIndex> = "first_*_addr_index",
        bytes: SeriesPattern33<P2WPKHBytes> = "*_bytes",
    } }
    shape! { AddressesP2sh at "series().indexer.addresses.p2sh" {
        first_index: SeriesPattern20<P2SHAddrIndex> = "first_*_addr_index",
        bytes: SeriesPattern31<P2SHBytes> = "*_bytes",
    } }
    shape! { AddressesP2pkh at "series().indexer.addresses.p2pkh" {
        first_index: SeriesPattern20<P2PKHAddrIndex> = "first_*_addr_index",
        bytes: SeriesPattern30<P2PKHBytes> = "*_bytes",
    } }
    shape! { AddressesP2pk33 at "series().indexer.addresses.p2pk33" {
        first_index: SeriesPattern20<P2PK33AddrIndex> = "first_*_addr_index",
        bytes: SeriesPattern28<P2PK33Bytes> = "*_bytes",
    } }
    shape! { AddressesP2pk65 at "series().indexer.addresses.p2pk65" {
        first_index: SeriesPattern20<P2PK65AddrIndex> = "first_*_addr_index",
        bytes: SeriesPattern29<P2PK65Bytes> = "*_bytes",
    } }
    shape! { IndexerAddresses at "series().indexer.addresses" {
        p2pk65: AddressesP2pk65 = "*",
        p2pk33: AddressesP2pk33 = "p2pk33",
        p2pkh: AddressesP2pkh = "p2pkh",
        p2sh: AddressesP2sh = "p2sh",
        p2wpkh: AddressesP2wpkh = "p2wpkh",
        p2wsh: AddressesP2wsh = "p2wsh",
        p2tr: AddressesP2tr = "p2tr",
        p2a: AddressesP2a = "p2a",
    } }
    shape! { IndexerOutputs at "series().indexer.outputs" {
        first_txout_index: SeriesPattern20<TxOutIndex> = "first_txout_index",
        value: SeriesPattern23<Sats> = "value",
        output_type: SeriesPattern23<OutputType> = "output_*",
        type_index: SeriesPattern23<TypeIndex> = "*_index",
    } }
    shape! { IndexerInputs at "series().indexer.inputs" {
        first_txin_index: SeriesPattern20<TxInIndex> = "first_txin_*",
        outpoint: SeriesPattern22<OutPoint> = "outpoint",
        txout_index: SeriesPattern22<TxOutIndex> = "txout_*",
        tx_index: SeriesPattern22<TxIndex> = "tx_*",
        output_type: SeriesPattern22<OutputType> = "output_type",
        type_index: SeriesPattern22<TypeIndex> = "type_*",
    } }
    shape! { FeaturesCount at "series().indexer.transactions.features.count" {
        v1: SeriesPattern20<Count16> = "*_v1",
        v2: SeriesPattern20<Count16> = "*_v2",
        v3: SeriesPattern20<Count16> = "*_v3",
        other_version: SeriesPattern20<Count16> = "*_other_version",
        explicitly_rbf: SeriesPattern20<Count16> = "*_explicitly_rbf",
        one_input: SeriesPattern20<Count16> = "*_one_input",
        one_output: SeriesPattern20<Count16> = "*_one_output",
        p2pk: SeriesPattern20<Count16> = "*_p2pk",
        p2ms: SeriesPattern20<Count16> = "*_p2ms",
        p2pkh: SeriesPattern20<Count16> = "*_p2pkh",
        p2sh: SeriesPattern20<Count16> = "*_p2sh",
        p2wpkh: SeriesPattern20<Count16> = "*_p2wpkh",
        p2wsh: SeriesPattern20<Count16> = "*_p2wsh",
        p2tr: SeriesPattern20<Count16> = "*_p2tr",
        p2a: SeriesPattern20<Count16> = "*_p2a",
        op_return: SeriesPattern20<Count16> = "*_op_return",
        empty: SeriesPattern20<Count16> = "*_empty",
        unknown: SeriesPattern20<Count16> = "*_unknown",
        fake_pubkey: SeriesPattern20<Count16> = "*_fake_pubkey",
        fake_scripthash: SeriesPattern20<Count16> = "*_fake_scripthash",
    } }
    shape! { IndexerTransactionsFeatures at "series().indexer.transactions.features" {
        count: FeaturesCount = "tx_count",
        has_p2pk: SeriesPattern21<Boolean> = "*_p2pk",
        has_p2ms: SeriesPattern21<Boolean> = "*_p2ms",
        has_p2pkh: SeriesPattern21<Boolean> = "*_p2pkh",
        has_p2sh: SeriesPattern21<Boolean> = "*_p2sh",
        has_p2wpkh: SeriesPattern21<Boolean> = "*_p2wpkh",
        has_p2wsh: SeriesPattern21<Boolean> = "*_p2wsh",
        has_p2tr: SeriesPattern21<Boolean> = "*_p2tr",
        has_p2a: SeriesPattern21<Boolean> = "*_p2a",
        has_op_return: SeriesPattern21<Boolean> = "*_op_return",
        has_empty: SeriesPattern21<Boolean> = "*_empty",
        has_unknown: SeriesPattern21<Boolean> = "*_unknown",
        has_fake_pubkey: SeriesPattern21<Boolean> = "*_fake_pubkey",
        has_fake_scripthash: SeriesPattern21<Boolean> = "*_fake_scripthash",
        has_inscription: SeriesPattern21<Boolean> = "*_inscription",
        has_annex: SeriesPattern21<Boolean> = "*_annex",
        has_sighash_all: SeriesPattern21<Boolean> = "*_sighash_all",
        has_sighash_none: SeriesPattern21<Boolean> = "*_sighash_none",
        has_sighash_single: SeriesPattern21<Boolean> = "*_sighash_single",
        has_sighash_default: SeriesPattern21<Boolean> = "*_sighash_default",
        has_sighash_anyone_can_pay: SeriesPattern21<Boolean> = "*_sighash_anyone_can_pay",
        has_dust_output: SeriesPattern21<Boolean> = "*_dust_output",
    } }
    shape! { IndexerTransactions at "series().indexer.transactions" {
        first_tx_index: SeriesPattern20<TxIndex> = "first_*_index",
        txid: SeriesPattern21<Txid> = "txid",
        tx_version: SeriesPattern21<TxVersion> = "*_version",
        raw_locktime: SeriesPattern21<RawLockTime> = "raw_locktime",
        weight: SeriesPattern21<Weight> = "*_weight",
        total_size: SeriesPattern21<Bytes32> = "total_size",
        total_sigop_cost: SeriesPattern21<SigOps> = "total_sigop_cost",
        is_explicitly_rbf: SeriesPattern21<Boolean> = "is_explicitly_rbf",
        first_txin_index: SeriesPattern21<TxInIndex> = "first_txin_index",
        first_txout_index: SeriesPattern21<TxOutIndex> = "first_txout_index",
        features: IndexerTransactionsFeatures = "has",
    } }
    shape! { IndexerBlocksSize<A> at "series().indexer.blocks.size" {
        block: SeriesPattern20<A> = "*",
    } }
    shape! { Time at "series().indexer.blocks.time" {
        timestamp: SeriesPattern20<Timestamp> = "*",
    } }
    shape! { IndexerBlocks at "series().indexer.blocks" {
        blockhash: SeriesPattern20<BlockHash> = "blockhash",
        coinbase_tag: SeriesPattern20<CoinbaseTag> = "coinbase_tag",
        time: Time = "timestamp",
        size: IndexerBlocksSize<Bytes32> = "total_size",
        weight: IndexerBlocksSize<Weight> = "block_weight",
        segwit_txs: SeriesPattern20<Count16> = "*_txs",
        segwit_size: SeriesPattern20<Bytes32> = "*_size",
        segwit_weight: SeriesPattern20<Weight> = "*_weight",
    } }
    shape! { Indexer at "series().indexer" {
        blocks: IndexerBlocks = "segwit",
        transactions: IndexerTransactions = "tx",
        inputs: IndexerInputs = "*",
        outputs: IndexerOutputs = "type",
        addresses: IndexerAddresses = "p2pk65",
        scripts: Scripts = "*",
        op_return: IndexerOpReturn = "*",
    } }
    shape! { SeriesTree at "series()" {
        indexer: Indexer = "index",
        mappings: Mappings = "date",
        price: Price = "price",
        blocks: Blocks = "block",
        inputs: Inputs = "input",
        outputs: Outputs = "output",
        transactions: Transactions = "tx",
        mining: Mining = "coinbase",
        op_return: OpReturn = "op_return",
        pools: Pools = "pool",
        utxo_set: UtxoSet2 = "circulating_supply",
        market: Market = "price",
        age: Age = "utxos",
        utxos: Utxos = "utxos",
        addresses: Addresses = "address",
        holders: Holders<HoldersAll<AllSupply>, HoldersAll<LthSupply>> = "utxos",
        entry: Entry = "veteran",
        supply: Supply = "rate",
        indicators: Indicators = "destroyed_supply_adjusted",
        cointime: Cointime = "utxos",
        coinflow: Coinflow = "utxos",
        bedrock: Bedrock = "bedrock",
        capital_sentiment: CapitalSentiment = "capital_sentiment",
        rarity_meter: RarityMeter = "rarity_meter",
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
    /// Returns the optional description, supported indexes, the indexes whose values can be null, the value type, and the optional unit (what the value type measures) for the specified series. The decoded series name is limited to 1024 UTF-8 bytes.
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

    /// Latest UTXO set
    ///
    /// The UTXO set after the latest published block, by creation height. Returns `{ height, hash, date, count, supply, origins }`; entry `i` of `origins.count` and `origins.supply` (BTC) is what remains unspent of block `i`'s outputs.
    ///
    /// Endpoint: `GET /api/utxo-set`
    pub fn get_utxo_set_latest(&self) -> Result<UtxoSet> {
        self.base.get_json(&format!("/api/utxo-set"))
    }

    /// UTXO set at block height or date
    ///
    /// The UTXO set after a block (`840000`) or after the last published block of a UTC day (`YYYY-MM-DD`), by creation height. Returns `{ height, hash, date, count, supply, origins }`; entry `i` of `origins.count` and `origins.supply` (BTC) is what remains unspent of block `i`'s outputs.
    ///
    /// Endpoint: `GET /api/utxo-set/{point}`
    pub fn get_utxo_set(&self, point: &str) -> Result<UtxoSet> {
        self.base.get_json(&format!("/api/utxo-set/{point}"))
    }

    /// UTXO set changes of a block or date
    ///
    /// What a block (`840000`) or a UTC day's published blocks (`YYYY-MM-DD`) changed in the UTXO set. Returns `{ first, last, hash, date, created, spent }`: `created` has one row per block, `spent` one row per creation height, each as columnar `height`, `count` and `supply` (BTC). The set after block `first - 1` plus `created` minus `spent` is the set after block `last`.
    ///
    /// Endpoint: `GET /api/utxo-set/{point}/diff`
    pub fn get_utxo_set_diff(&self, point: &str) -> Result<UtxoSetDiff> {
        self.base.get_json(&format!("/api/utxo-set/{point}/diff"))
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
