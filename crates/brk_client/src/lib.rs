// Auto-generated BRK Rust client
// Do not edit manually

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(clippy::useless_format)]
#![allow(clippy::unnecessary_to_owned)]

use std::str::FromStr;
use std::sync::Arc;
use std::ops::{Bound, RangeBounds};
use serde::de::DeserializeOwned;
pub use brk_cohort::*;
pub use brk_types::*;


/// Error type for BRK client operations.
#[derive(Debug)]
pub struct BrkError {
    pub message: String,
}

impl std::fmt::Display for BrkError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for BrkError {}

/// Result type for BRK client operations.
pub type Result<T> = std::result::Result<T, BrkError>;

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
        return Err(BrkError { message: "Expected a non-empty address payload".to_string() });
    }
    if payload.len() > 65 {
        return Err(BrkError { message: "Expected at most 65 address payload bytes".to_string() });
    }
    if !(1..=16).contains(&nibbles) {
        return Err(BrkError { message: "Expected hash-prefix length from 1 to 16 hex nibbles".to_string() });
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
            return Err(BrkError { message: format!("Unsupported address type for address payload hash-prefix: {addr_type:?}") });
        },
    };

    if !expected.contains(&payload.len()) {
        let joined = expected
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join(" or ");
        return Err(BrkError { message: format!("Expected {addr_type} address payload length {joined} bytes") });
    }

    Ok(())
}

#[cfg(test)]
mod address_payload_tests {
    use super::*;

    #[test]
    fn p2pk_payload_lengths_are_distinct() {
        assert!(validate_address_payload_for_type(OutputType::P2PK33, &[0; 33]).is_ok());
        assert!(validate_address_payload_for_type(OutputType::P2PK65, &[0; 65]).is_ok());
        assert!(validate_address_payload_for_type(OutputType::P2PK33, &[0; 65]).is_err());
        assert!(validate_address_payload_for_type(OutputType::P2PK65, &[0; 33]).is_err());
    }
}

/// Decode a mainnet Bitcoin address into the BRK address type and raw payload bytes.
pub fn decode_address_payload(address: &str) -> Result<AddressPayload> {
    if address.is_empty() {
        return Err(BrkError { message: "Expected an address string".to_string() });
    }
    let addr_bytes = AddrBytes::from_str(address).map_err(|e| BrkError { message: e.to_string() })?;
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

/// Options for configuring the BRK client.
#[derive(Debug, Clone)]
pub struct BrkClientOptions {
    pub base_url: String,
    pub timeout_secs: u64,
}

impl Default for BrkClientOptions {
    fn default() -> Self {
        Self {
            base_url: "http://localhost:3000".to_string(),
            timeout_secs: 30,
        }
    }
}

/// Base HTTP client for making requests. Reuses connections via ureq::Agent.
#[derive(Debug, Clone)]
pub struct BrkClientBase {
    agent: ureq::Agent,
    base_url: String,
}

impl BrkClientBase {
    /// Create a new client with the given base URL.
    pub fn new(base_url: impl Into<String>) -> Self {
        Self::with_options(BrkClientOptions { base_url: base_url.into(), ..Default::default() })
    }

    /// Create a new client with options.
    pub fn with_options(options: BrkClientOptions) -> Self {
        let agent = ureq::Agent::config_builder()
            .timeout_global(Some(std::time::Duration::from_secs(options.timeout_secs)))
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

    /// Make a GET request and deserialize JSON response.
    pub fn get_json<T: DeserializeOwned>(&self, path: &str) -> Result<T> {
        self.agent.get(&self.url(path))
            .call()
            .and_then(|mut r| r.body_mut().read_json())
            .map_err(|e| BrkError { message: e.to_string() })
    }

    /// Make a GET request and return raw text response.
    pub fn get_text(&self, path: &str) -> Result<String> {
        self.agent.get(&self.url(path))
            .call()
            .and_then(|mut r| r.body_mut().read_to_string())
            .map_err(|e| BrkError { message: e.to_string() })
    }

    /// Make a GET request and return raw bytes response.
    pub fn get_bytes(&self, path: &str) -> Result<Vec<u8>> {
        self.agent.get(&self.url(path))
            .call()
            .and_then(|mut r| r.body_mut().read_to_vec())
            .map_err(|e| BrkError { message: e.to_string() })
    }

    /// Make a POST request and deserialize JSON response.
    pub fn post_json<T: DeserializeOwned>(&self, path: &str, body: &str) -> Result<T> {
        self.agent.post(&self.url(path))
            .send(body)
            .and_then(|mut r| r.body_mut().read_json())
            .map_err(|e| BrkError { message: e.to_string() })
    }

    /// Make a POST request and return raw text response.
    pub fn post_text(&self, path: &str, body: &str) -> Result<String> {
        self.agent.post(&self.url(path))
            .send(body)
            .and_then(|mut r| r.body_mut().read_to_string())
            .map_err(|e| BrkError { message: e.to_string() })
    }

    /// Make a POST request and return raw bytes response.
    pub fn post_bytes(&self, path: &str, body: &str) -> Result<Vec<u8>> {
        self.agent.post(&self.url(path))
            .send(body)
            .and_then(|mut r| r.body_mut().read_to_vec())
            .map_err(|e| BrkError { message: e.to_string() })
    }
}

/// Build series name with suffix.
#[inline]
fn _m(acc: &str, s: &str) -> String {
    if s.is_empty() { acc.to_string() }
    else if acc.is_empty() { s.to_string() }
    else { format!("{acc}_{s}") }
}

/// Build series name with prefix.
#[inline]
fn _p(prefix: &str, acc: &str) -> String {
    if acc.is_empty() { prefix.to_string() } else { format!("{prefix}_{acc}") }
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


/// Shared endpoint configuration.
#[derive(Clone)]
struct EndpointConfig {
    client: Arc<BrkClientBase>,
    name: Arc<str>,
    index: Index,
    start: Option<i64>,
    end: Option<i64>,
}

impl EndpointConfig {
    fn new(client: Arc<BrkClientBase>, name: Arc<str>, index: Index) -> Self {
        Self { client, name, index, start: None, end: None }
    }

    fn path(&self) -> String {
        format!("/api/series/{}/{}", self.name, self.index.name())
    }

    fn build_path(&self, format: Option<&str>) -> String {
        let mut params = Vec::new();
        if let Some(s) = self.start { params.push(format!("start={}", s)); }
        if let Some(e) = self.end { params.push(format!("end={}", e)); }
        if let Some(fmt) = format { params.push(format!("format={}", fmt)); }
        let p = self.path();
        if params.is_empty() { p } else { format!("{}?{}", p, params.join("&")) }
    }

    fn get_json<T: DeserializeOwned>(&self, format: Option<&str>) -> Result<T> {
        self.client.get_json(&self.build_path(format))
    }

    fn get_text(&self, format: Option<&str>) -> Result<String> {
        self.client.get_text(&self.build_path(format))
    }

    fn get_len(&self) -> Result<i64> {
        self.client.get_json(&format!("/api/series/{}/{}/len", self.name, self.index.name()))
    }

    fn get_version(&self) -> Result<Version> {
        self.client.get_json(&format!("/api/series/{}/{}/version", self.name, self.index.name()))
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

/// Builder for date-based series endpoint queries.
///
/// Like `SeriesEndpoint` but returns `DateSeriesData` and provides
/// date-based access methods (`get_date`, `date_range`).
pub type DateSeriesEndpoint<T> = SeriesEndpoint<T, DateSeriesData<T>>;

impl<T: DeserializeOwned, D: DeserializeOwned> SeriesEndpoint<T, D> {
    pub fn new(client: Arc<BrkClientBase>, name: Arc<str>, index: Index) -> Self {
        Self { config: EndpointConfig::new(client, name, index), _marker: std::marker::PhantomData }
    }

    /// Select a specific index position.
    pub fn get(mut self, index: usize) -> SingleItemBuilder<T, D> {
        self.config.start = Some(index as i64);
        self.config.end = Some(index as i64 + 1);
        SingleItemBuilder { config: self.config, _marker: std::marker::PhantomData }
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
        RangeBuilder { config: self.config, _marker: std::marker::PhantomData }
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
        RangeBuilder { config: self.config, _marker: std::marker::PhantomData }
    }

    /// Skip the first n items. Chain with `take(n)` to get a range.
    pub fn skip(mut self, n: usize) -> SkippedBuilder<T, D> {
        self.config.start = Some(n as i64);
        SkippedBuilder { config: self.config, _marker: std::marker::PhantomData }
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
    pub fn version(&self) -> Result<Version> {
        self.config.get_version()
    }

    /// Get the base endpoint path.
    pub fn path(&self) -> String {
        self.config.path()
    }
}

/// Date-specific methods available only on `DateSeriesEndpoint`.
impl<T: DeserializeOwned> SeriesEndpoint<T, DateSeriesData<T>> {
    /// Select a specific date position (for day-precision or coarser indexes).
    pub fn get_date(self, date: Date) -> SingleItemBuilder<T, DateSeriesData<T>> {
        let index = self.config.index.date_to_index(date).unwrap_or(0);
        self.get(index)
    }

    /// Select a date range (for day-precision or coarser indexes).
    pub fn date_range(self, start: Date, end: Date) -> RangeBuilder<T, DateSeriesData<T>> {
        let s = self.config.index.date_to_index(start).unwrap_or(0);
        let e = self.config.index.date_to_index(end).unwrap_or(0);
        self.range(s..e)
    }

    /// Select a specific timestamp position (works for all date-based indexes including sub-daily).
    pub fn get_timestamp(self, ts: Timestamp) -> SingleItemBuilder<T, DateSeriesData<T>> {
        let index = self.config.index.timestamp_to_index(ts).unwrap_or(0);
        self.get(index)
    }

    /// Select a timestamp range (works for all date-based indexes including sub-daily).
    pub fn timestamp_range(self, start: Timestamp, end: Timestamp) -> RangeBuilder<T, DateSeriesData<T>> {
        let s = self.config.index.timestamp_to_index(start).unwrap_or(0);
        let e = self.config.index.timestamp_to_index(end).unwrap_or(0);
        self.range(s..e)
    }
}

/// Builder for single item access.
pub struct SingleItemBuilder<T, D = SeriesData<T>> {
    config: EndpointConfig,
    _marker: std::marker::PhantomData<fn() -> (T, D)>,
}

/// Date-aware single item builder.
pub type DateSingleItemBuilder<T> = SingleItemBuilder<T, DateSeriesData<T>>;

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

/// Date-aware skipped builder.
pub type DateSkippedBuilder<T> = SkippedBuilder<T, DateSeriesData<T>>;

impl<T: DeserializeOwned, D: DeserializeOwned> SkippedBuilder<T, D> {
    /// Take n items after the skipped position.
    pub fn take(mut self, n: usize) -> RangeBuilder<T, D> {
        let start = self.config.start.unwrap_or(0);
        self.config.end = Some(start + n as i64);
        RangeBuilder { config: self.config, _marker: std::marker::PhantomData }
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

/// Date-aware range builder.
pub type DateRangeBuilder<T> = RangeBuilder<T, DateSeriesData<T>>;

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
const _I1: &[Index] = &[Index::Minute10, Index::Minute30, Index::Hour1, Index::Hour4, Index::Hour12, Index::Day1, Index::Day3, Index::Week1, Index::Month1, Index::Month3, Index::Month6, Index::Year1, Index::Year10, Index::Halving, Index::Epoch, Index::Height];
const _I2: &[Index] = &[Index::Minute10, Index::Minute30, Index::Hour1, Index::Hour4, Index::Hour12, Index::Day1, Index::Day3, Index::Week1, Index::Month1, Index::Month3, Index::Month6, Index::Year1, Index::Year10, Index::Halving, Index::Epoch];
const _I3: &[Index] = &[Index::Minute10];
const _I4: &[Index] = &[Index::Minute30];
const _I5: &[Index] = &[Index::Hour1];
const _I6: &[Index] = &[Index::Hour4];
const _I7: &[Index] = &[Index::Hour12];
const _I8: &[Index] = &[Index::Day1];
const _I9: &[Index] = &[Index::Day3];
const _I10: &[Index] = &[Index::Week1];
const _I11: &[Index] = &[Index::Month1];
const _I12: &[Index] = &[Index::Month3];
const _I13: &[Index] = &[Index::Month6];
const _I14: &[Index] = &[Index::Year1];
const _I15: &[Index] = &[Index::Year10];
const _I16: &[Index] = &[Index::Halving];
const _I17: &[Index] = &[Index::Epoch];
const _I18: &[Index] = &[Index::Height];
const _I19: &[Index] = &[Index::TxIndex];
const _I20: &[Index] = &[Index::TxInIndex];
const _I21: &[Index] = &[Index::TxOutIndex];
const _I22: &[Index] = &[Index::EmptyOutputIndex];
const _I23: &[Index] = &[Index::OpReturnIndex];
const _I24: &[Index] = &[Index::P2AAddrIndex];
const _I25: &[Index] = &[Index::P2MSOutputIndex];
const _I26: &[Index] = &[Index::P2PK33AddrIndex];
const _I27: &[Index] = &[Index::P2PK65AddrIndex];
const _I28: &[Index] = &[Index::P2PKHAddrIndex];
const _I29: &[Index] = &[Index::P2SHAddrIndex];
const _I30: &[Index] = &[Index::P2TRAddrIndex];
const _I31: &[Index] = &[Index::P2WPKHAddrIndex];
const _I32: &[Index] = &[Index::P2WSHAddrIndex];
const _I33: &[Index] = &[Index::UnknownOutputIndex];
const _I34: &[Index] = &[Index::FundedAddrIndex];
const _I35: &[Index] = &[Index::EmptyAddrIndex];

#[inline]
fn _ep<T: DeserializeOwned>(c: &Arc<BrkClientBase>, n: &Arc<str>, i: Index) -> SeriesEndpoint<T> {
    SeriesEndpoint::new(c.clone(), n.clone(), i)
}

#[inline]
fn _dep<T: DeserializeOwned>(c: &Arc<BrkClientBase>, n: &Arc<str>, i: Index) -> DateSeriesEndpoint<T> {
    DateSeriesEndpoint::new(c.clone(), n.clone(), i)
}

// Index accessor structs

pub struct SeriesPattern1By<T> { client: Arc<BrkClientBase>, name: Arc<str>, _marker: std::marker::PhantomData<T> }
impl<T: DeserializeOwned> SeriesPattern1By<T> {
    pub fn minute10(&self) -> DateSeriesEndpoint<T> { _dep(&self.client, &self.name, Index::Minute10) }
    pub fn minute30(&self) -> DateSeriesEndpoint<T> { _dep(&self.client, &self.name, Index::Minute30) }
    pub fn hour1(&self) -> DateSeriesEndpoint<T> { _dep(&self.client, &self.name, Index::Hour1) }
    pub fn hour4(&self) -> DateSeriesEndpoint<T> { _dep(&self.client, &self.name, Index::Hour4) }
    pub fn hour12(&self) -> DateSeriesEndpoint<T> { _dep(&self.client, &self.name, Index::Hour12) }
    pub fn day1(&self) -> DateSeriesEndpoint<T> { _dep(&self.client, &self.name, Index::Day1) }
    pub fn day3(&self) -> DateSeriesEndpoint<T> { _dep(&self.client, &self.name, Index::Day3) }
    pub fn week1(&self) -> DateSeriesEndpoint<T> { _dep(&self.client, &self.name, Index::Week1) }
    pub fn month1(&self) -> DateSeriesEndpoint<T> { _dep(&self.client, &self.name, Index::Month1) }
    pub fn month3(&self) -> DateSeriesEndpoint<T> { _dep(&self.client, &self.name, Index::Month3) }
    pub fn month6(&self) -> DateSeriesEndpoint<T> { _dep(&self.client, &self.name, Index::Month6) }
    pub fn year1(&self) -> DateSeriesEndpoint<T> { _dep(&self.client, &self.name, Index::Year1) }
    pub fn year10(&self) -> DateSeriesEndpoint<T> { _dep(&self.client, &self.name, Index::Year10) }
    pub fn halving(&self) -> SeriesEndpoint<T> { _ep(&self.client, &self.name, Index::Halving) }
    pub fn epoch(&self) -> SeriesEndpoint<T> { _ep(&self.client, &self.name, Index::Epoch) }
    pub fn height(&self) -> SeriesEndpoint<T> { _ep(&self.client, &self.name, Index::Height) }
}

pub struct SeriesPattern1<T> { name: Arc<str>, pub by: SeriesPattern1By<T> }
impl<T: DeserializeOwned> SeriesPattern1<T> {
    pub fn new(client: Arc<BrkClientBase>, name: String) -> Self { let name: Arc<str> = name.into(); Self { name: name.clone(), by: SeriesPattern1By { client, name, _marker: std::marker::PhantomData } } }
    pub fn name(&self) -> &str { &self.name }
}

impl<T> AnySeriesPattern for SeriesPattern1<T> { fn name(&self) -> &str { &self.name } fn indexes(&self) -> &'static [Index] { _I1 } }
impl<T: DeserializeOwned> SeriesPattern<T> for SeriesPattern1<T> { fn get(&self, index: Index) -> Option<SeriesEndpoint<T>> { _I1.contains(&index).then(|| _ep(&self.by.client, &self.by.name, index)) } }

pub struct SeriesPattern2By<T> { client: Arc<BrkClientBase>, name: Arc<str>, _marker: std::marker::PhantomData<T> }
impl<T: DeserializeOwned> SeriesPattern2By<T> {
    pub fn minute10(&self) -> DateSeriesEndpoint<T> { _dep(&self.client, &self.name, Index::Minute10) }
    pub fn minute30(&self) -> DateSeriesEndpoint<T> { _dep(&self.client, &self.name, Index::Minute30) }
    pub fn hour1(&self) -> DateSeriesEndpoint<T> { _dep(&self.client, &self.name, Index::Hour1) }
    pub fn hour4(&self) -> DateSeriesEndpoint<T> { _dep(&self.client, &self.name, Index::Hour4) }
    pub fn hour12(&self) -> DateSeriesEndpoint<T> { _dep(&self.client, &self.name, Index::Hour12) }
    pub fn day1(&self) -> DateSeriesEndpoint<T> { _dep(&self.client, &self.name, Index::Day1) }
    pub fn day3(&self) -> DateSeriesEndpoint<T> { _dep(&self.client, &self.name, Index::Day3) }
    pub fn week1(&self) -> DateSeriesEndpoint<T> { _dep(&self.client, &self.name, Index::Week1) }
    pub fn month1(&self) -> DateSeriesEndpoint<T> { _dep(&self.client, &self.name, Index::Month1) }
    pub fn month3(&self) -> DateSeriesEndpoint<T> { _dep(&self.client, &self.name, Index::Month3) }
    pub fn month6(&self) -> DateSeriesEndpoint<T> { _dep(&self.client, &self.name, Index::Month6) }
    pub fn year1(&self) -> DateSeriesEndpoint<T> { _dep(&self.client, &self.name, Index::Year1) }
    pub fn year10(&self) -> DateSeriesEndpoint<T> { _dep(&self.client, &self.name, Index::Year10) }
    pub fn halving(&self) -> SeriesEndpoint<T> { _ep(&self.client, &self.name, Index::Halving) }
    pub fn epoch(&self) -> SeriesEndpoint<T> { _ep(&self.client, &self.name, Index::Epoch) }
}

pub struct SeriesPattern2<T> { name: Arc<str>, pub by: SeriesPattern2By<T> }
impl<T: DeserializeOwned> SeriesPattern2<T> {
    pub fn new(client: Arc<BrkClientBase>, name: String) -> Self { let name: Arc<str> = name.into(); Self { name: name.clone(), by: SeriesPattern2By { client, name, _marker: std::marker::PhantomData } } }
    pub fn name(&self) -> &str { &self.name }
}

impl<T> AnySeriesPattern for SeriesPattern2<T> { fn name(&self) -> &str { &self.name } fn indexes(&self) -> &'static [Index] { _I2 } }
impl<T: DeserializeOwned> SeriesPattern<T> for SeriesPattern2<T> { fn get(&self, index: Index) -> Option<SeriesEndpoint<T>> { _I2.contains(&index).then(|| _ep(&self.by.client, &self.by.name, index)) } }

pub struct SeriesPattern3By<T> { client: Arc<BrkClientBase>, name: Arc<str>, _marker: std::marker::PhantomData<T> }
impl<T: DeserializeOwned> SeriesPattern3By<T> {
    pub fn minute10(&self) -> DateSeriesEndpoint<T> { _dep(&self.client, &self.name, Index::Minute10) }
}

pub struct SeriesPattern3<T> { name: Arc<str>, pub by: SeriesPattern3By<T> }
impl<T: DeserializeOwned> SeriesPattern3<T> {
    pub fn new(client: Arc<BrkClientBase>, name: String) -> Self { let name: Arc<str> = name.into(); Self { name: name.clone(), by: SeriesPattern3By { client, name, _marker: std::marker::PhantomData } } }
    pub fn name(&self) -> &str { &self.name }
}

impl<T> AnySeriesPattern for SeriesPattern3<T> { fn name(&self) -> &str { &self.name } fn indexes(&self) -> &'static [Index] { _I3 } }
impl<T: DeserializeOwned> SeriesPattern<T> for SeriesPattern3<T> { fn get(&self, index: Index) -> Option<SeriesEndpoint<T>> { _I3.contains(&index).then(|| _ep(&self.by.client, &self.by.name, index)) } }

pub struct SeriesPattern4By<T> { client: Arc<BrkClientBase>, name: Arc<str>, _marker: std::marker::PhantomData<T> }
impl<T: DeserializeOwned> SeriesPattern4By<T> {
    pub fn minute30(&self) -> DateSeriesEndpoint<T> { _dep(&self.client, &self.name, Index::Minute30) }
}

pub struct SeriesPattern4<T> { name: Arc<str>, pub by: SeriesPattern4By<T> }
impl<T: DeserializeOwned> SeriesPattern4<T> {
    pub fn new(client: Arc<BrkClientBase>, name: String) -> Self { let name: Arc<str> = name.into(); Self { name: name.clone(), by: SeriesPattern4By { client, name, _marker: std::marker::PhantomData } } }
    pub fn name(&self) -> &str { &self.name }
}

impl<T> AnySeriesPattern for SeriesPattern4<T> { fn name(&self) -> &str { &self.name } fn indexes(&self) -> &'static [Index] { _I4 } }
impl<T: DeserializeOwned> SeriesPattern<T> for SeriesPattern4<T> { fn get(&self, index: Index) -> Option<SeriesEndpoint<T>> { _I4.contains(&index).then(|| _ep(&self.by.client, &self.by.name, index)) } }

pub struct SeriesPattern5By<T> { client: Arc<BrkClientBase>, name: Arc<str>, _marker: std::marker::PhantomData<T> }
impl<T: DeserializeOwned> SeriesPattern5By<T> {
    pub fn hour1(&self) -> DateSeriesEndpoint<T> { _dep(&self.client, &self.name, Index::Hour1) }
}

pub struct SeriesPattern5<T> { name: Arc<str>, pub by: SeriesPattern5By<T> }
impl<T: DeserializeOwned> SeriesPattern5<T> {
    pub fn new(client: Arc<BrkClientBase>, name: String) -> Self { let name: Arc<str> = name.into(); Self { name: name.clone(), by: SeriesPattern5By { client, name, _marker: std::marker::PhantomData } } }
    pub fn name(&self) -> &str { &self.name }
}

impl<T> AnySeriesPattern for SeriesPattern5<T> { fn name(&self) -> &str { &self.name } fn indexes(&self) -> &'static [Index] { _I5 } }
impl<T: DeserializeOwned> SeriesPattern<T> for SeriesPattern5<T> { fn get(&self, index: Index) -> Option<SeriesEndpoint<T>> { _I5.contains(&index).then(|| _ep(&self.by.client, &self.by.name, index)) } }

pub struct SeriesPattern6By<T> { client: Arc<BrkClientBase>, name: Arc<str>, _marker: std::marker::PhantomData<T> }
impl<T: DeserializeOwned> SeriesPattern6By<T> {
    pub fn hour4(&self) -> DateSeriesEndpoint<T> { _dep(&self.client, &self.name, Index::Hour4) }
}

pub struct SeriesPattern6<T> { name: Arc<str>, pub by: SeriesPattern6By<T> }
impl<T: DeserializeOwned> SeriesPattern6<T> {
    pub fn new(client: Arc<BrkClientBase>, name: String) -> Self { let name: Arc<str> = name.into(); Self { name: name.clone(), by: SeriesPattern6By { client, name, _marker: std::marker::PhantomData } } }
    pub fn name(&self) -> &str { &self.name }
}

impl<T> AnySeriesPattern for SeriesPattern6<T> { fn name(&self) -> &str { &self.name } fn indexes(&self) -> &'static [Index] { _I6 } }
impl<T: DeserializeOwned> SeriesPattern<T> for SeriesPattern6<T> { fn get(&self, index: Index) -> Option<SeriesEndpoint<T>> { _I6.contains(&index).then(|| _ep(&self.by.client, &self.by.name, index)) } }

pub struct SeriesPattern7By<T> { client: Arc<BrkClientBase>, name: Arc<str>, _marker: std::marker::PhantomData<T> }
impl<T: DeserializeOwned> SeriesPattern7By<T> {
    pub fn hour12(&self) -> DateSeriesEndpoint<T> { _dep(&self.client, &self.name, Index::Hour12) }
}

pub struct SeriesPattern7<T> { name: Arc<str>, pub by: SeriesPattern7By<T> }
impl<T: DeserializeOwned> SeriesPattern7<T> {
    pub fn new(client: Arc<BrkClientBase>, name: String) -> Self { let name: Arc<str> = name.into(); Self { name: name.clone(), by: SeriesPattern7By { client, name, _marker: std::marker::PhantomData } } }
    pub fn name(&self) -> &str { &self.name }
}

impl<T> AnySeriesPattern for SeriesPattern7<T> { fn name(&self) -> &str { &self.name } fn indexes(&self) -> &'static [Index] { _I7 } }
impl<T: DeserializeOwned> SeriesPattern<T> for SeriesPattern7<T> { fn get(&self, index: Index) -> Option<SeriesEndpoint<T>> { _I7.contains(&index).then(|| _ep(&self.by.client, &self.by.name, index)) } }

pub struct SeriesPattern8By<T> { client: Arc<BrkClientBase>, name: Arc<str>, _marker: std::marker::PhantomData<T> }
impl<T: DeserializeOwned> SeriesPattern8By<T> {
    pub fn day1(&self) -> DateSeriesEndpoint<T> { _dep(&self.client, &self.name, Index::Day1) }
}

pub struct SeriesPattern8<T> { name: Arc<str>, pub by: SeriesPattern8By<T> }
impl<T: DeserializeOwned> SeriesPattern8<T> {
    pub fn new(client: Arc<BrkClientBase>, name: String) -> Self { let name: Arc<str> = name.into(); Self { name: name.clone(), by: SeriesPattern8By { client, name, _marker: std::marker::PhantomData } } }
    pub fn name(&self) -> &str { &self.name }
}

impl<T> AnySeriesPattern for SeriesPattern8<T> { fn name(&self) -> &str { &self.name } fn indexes(&self) -> &'static [Index] { _I8 } }
impl<T: DeserializeOwned> SeriesPattern<T> for SeriesPattern8<T> { fn get(&self, index: Index) -> Option<SeriesEndpoint<T>> { _I8.contains(&index).then(|| _ep(&self.by.client, &self.by.name, index)) } }

pub struct SeriesPattern9By<T> { client: Arc<BrkClientBase>, name: Arc<str>, _marker: std::marker::PhantomData<T> }
impl<T: DeserializeOwned> SeriesPattern9By<T> {
    pub fn day3(&self) -> DateSeriesEndpoint<T> { _dep(&self.client, &self.name, Index::Day3) }
}

pub struct SeriesPattern9<T> { name: Arc<str>, pub by: SeriesPattern9By<T> }
impl<T: DeserializeOwned> SeriesPattern9<T> {
    pub fn new(client: Arc<BrkClientBase>, name: String) -> Self { let name: Arc<str> = name.into(); Self { name: name.clone(), by: SeriesPattern9By { client, name, _marker: std::marker::PhantomData } } }
    pub fn name(&self) -> &str { &self.name }
}

impl<T> AnySeriesPattern for SeriesPattern9<T> { fn name(&self) -> &str { &self.name } fn indexes(&self) -> &'static [Index] { _I9 } }
impl<T: DeserializeOwned> SeriesPattern<T> for SeriesPattern9<T> { fn get(&self, index: Index) -> Option<SeriesEndpoint<T>> { _I9.contains(&index).then(|| _ep(&self.by.client, &self.by.name, index)) } }

pub struct SeriesPattern10By<T> { client: Arc<BrkClientBase>, name: Arc<str>, _marker: std::marker::PhantomData<T> }
impl<T: DeserializeOwned> SeriesPattern10By<T> {
    pub fn week1(&self) -> DateSeriesEndpoint<T> { _dep(&self.client, &self.name, Index::Week1) }
}

pub struct SeriesPattern10<T> { name: Arc<str>, pub by: SeriesPattern10By<T> }
impl<T: DeserializeOwned> SeriesPattern10<T> {
    pub fn new(client: Arc<BrkClientBase>, name: String) -> Self { let name: Arc<str> = name.into(); Self { name: name.clone(), by: SeriesPattern10By { client, name, _marker: std::marker::PhantomData } } }
    pub fn name(&self) -> &str { &self.name }
}

impl<T> AnySeriesPattern for SeriesPattern10<T> { fn name(&self) -> &str { &self.name } fn indexes(&self) -> &'static [Index] { _I10 } }
impl<T: DeserializeOwned> SeriesPattern<T> for SeriesPattern10<T> { fn get(&self, index: Index) -> Option<SeriesEndpoint<T>> { _I10.contains(&index).then(|| _ep(&self.by.client, &self.by.name, index)) } }

pub struct SeriesPattern11By<T> { client: Arc<BrkClientBase>, name: Arc<str>, _marker: std::marker::PhantomData<T> }
impl<T: DeserializeOwned> SeriesPattern11By<T> {
    pub fn month1(&self) -> DateSeriesEndpoint<T> { _dep(&self.client, &self.name, Index::Month1) }
}

pub struct SeriesPattern11<T> { name: Arc<str>, pub by: SeriesPattern11By<T> }
impl<T: DeserializeOwned> SeriesPattern11<T> {
    pub fn new(client: Arc<BrkClientBase>, name: String) -> Self { let name: Arc<str> = name.into(); Self { name: name.clone(), by: SeriesPattern11By { client, name, _marker: std::marker::PhantomData } } }
    pub fn name(&self) -> &str { &self.name }
}

impl<T> AnySeriesPattern for SeriesPattern11<T> { fn name(&self) -> &str { &self.name } fn indexes(&self) -> &'static [Index] { _I11 } }
impl<T: DeserializeOwned> SeriesPattern<T> for SeriesPattern11<T> { fn get(&self, index: Index) -> Option<SeriesEndpoint<T>> { _I11.contains(&index).then(|| _ep(&self.by.client, &self.by.name, index)) } }

pub struct SeriesPattern12By<T> { client: Arc<BrkClientBase>, name: Arc<str>, _marker: std::marker::PhantomData<T> }
impl<T: DeserializeOwned> SeriesPattern12By<T> {
    pub fn month3(&self) -> DateSeriesEndpoint<T> { _dep(&self.client, &self.name, Index::Month3) }
}

pub struct SeriesPattern12<T> { name: Arc<str>, pub by: SeriesPattern12By<T> }
impl<T: DeserializeOwned> SeriesPattern12<T> {
    pub fn new(client: Arc<BrkClientBase>, name: String) -> Self { let name: Arc<str> = name.into(); Self { name: name.clone(), by: SeriesPattern12By { client, name, _marker: std::marker::PhantomData } } }
    pub fn name(&self) -> &str { &self.name }
}

impl<T> AnySeriesPattern for SeriesPattern12<T> { fn name(&self) -> &str { &self.name } fn indexes(&self) -> &'static [Index] { _I12 } }
impl<T: DeserializeOwned> SeriesPattern<T> for SeriesPattern12<T> { fn get(&self, index: Index) -> Option<SeriesEndpoint<T>> { _I12.contains(&index).then(|| _ep(&self.by.client, &self.by.name, index)) } }

pub struct SeriesPattern13By<T> { client: Arc<BrkClientBase>, name: Arc<str>, _marker: std::marker::PhantomData<T> }
impl<T: DeserializeOwned> SeriesPattern13By<T> {
    pub fn month6(&self) -> DateSeriesEndpoint<T> { _dep(&self.client, &self.name, Index::Month6) }
}

pub struct SeriesPattern13<T> { name: Arc<str>, pub by: SeriesPattern13By<T> }
impl<T: DeserializeOwned> SeriesPattern13<T> {
    pub fn new(client: Arc<BrkClientBase>, name: String) -> Self { let name: Arc<str> = name.into(); Self { name: name.clone(), by: SeriesPattern13By { client, name, _marker: std::marker::PhantomData } } }
    pub fn name(&self) -> &str { &self.name }
}

impl<T> AnySeriesPattern for SeriesPattern13<T> { fn name(&self) -> &str { &self.name } fn indexes(&self) -> &'static [Index] { _I13 } }
impl<T: DeserializeOwned> SeriesPattern<T> for SeriesPattern13<T> { fn get(&self, index: Index) -> Option<SeriesEndpoint<T>> { _I13.contains(&index).then(|| _ep(&self.by.client, &self.by.name, index)) } }

pub struct SeriesPattern14By<T> { client: Arc<BrkClientBase>, name: Arc<str>, _marker: std::marker::PhantomData<T> }
impl<T: DeserializeOwned> SeriesPattern14By<T> {
    pub fn year1(&self) -> DateSeriesEndpoint<T> { _dep(&self.client, &self.name, Index::Year1) }
}

pub struct SeriesPattern14<T> { name: Arc<str>, pub by: SeriesPattern14By<T> }
impl<T: DeserializeOwned> SeriesPattern14<T> {
    pub fn new(client: Arc<BrkClientBase>, name: String) -> Self { let name: Arc<str> = name.into(); Self { name: name.clone(), by: SeriesPattern14By { client, name, _marker: std::marker::PhantomData } } }
    pub fn name(&self) -> &str { &self.name }
}

impl<T> AnySeriesPattern for SeriesPattern14<T> { fn name(&self) -> &str { &self.name } fn indexes(&self) -> &'static [Index] { _I14 } }
impl<T: DeserializeOwned> SeriesPattern<T> for SeriesPattern14<T> { fn get(&self, index: Index) -> Option<SeriesEndpoint<T>> { _I14.contains(&index).then(|| _ep(&self.by.client, &self.by.name, index)) } }

pub struct SeriesPattern15By<T> { client: Arc<BrkClientBase>, name: Arc<str>, _marker: std::marker::PhantomData<T> }
impl<T: DeserializeOwned> SeriesPattern15By<T> {
    pub fn year10(&self) -> DateSeriesEndpoint<T> { _dep(&self.client, &self.name, Index::Year10) }
}

pub struct SeriesPattern15<T> { name: Arc<str>, pub by: SeriesPattern15By<T> }
impl<T: DeserializeOwned> SeriesPattern15<T> {
    pub fn new(client: Arc<BrkClientBase>, name: String) -> Self { let name: Arc<str> = name.into(); Self { name: name.clone(), by: SeriesPattern15By { client, name, _marker: std::marker::PhantomData } } }
    pub fn name(&self) -> &str { &self.name }
}

impl<T> AnySeriesPattern for SeriesPattern15<T> { fn name(&self) -> &str { &self.name } fn indexes(&self) -> &'static [Index] { _I15 } }
impl<T: DeserializeOwned> SeriesPattern<T> for SeriesPattern15<T> { fn get(&self, index: Index) -> Option<SeriesEndpoint<T>> { _I15.contains(&index).then(|| _ep(&self.by.client, &self.by.name, index)) } }

pub struct SeriesPattern16By<T> { client: Arc<BrkClientBase>, name: Arc<str>, _marker: std::marker::PhantomData<T> }
impl<T: DeserializeOwned> SeriesPattern16By<T> {
    pub fn halving(&self) -> SeriesEndpoint<T> { _ep(&self.client, &self.name, Index::Halving) }
}

pub struct SeriesPattern16<T> { name: Arc<str>, pub by: SeriesPattern16By<T> }
impl<T: DeserializeOwned> SeriesPattern16<T> {
    pub fn new(client: Arc<BrkClientBase>, name: String) -> Self { let name: Arc<str> = name.into(); Self { name: name.clone(), by: SeriesPattern16By { client, name, _marker: std::marker::PhantomData } } }
    pub fn name(&self) -> &str { &self.name }
}

impl<T> AnySeriesPattern for SeriesPattern16<T> { fn name(&self) -> &str { &self.name } fn indexes(&self) -> &'static [Index] { _I16 } }
impl<T: DeserializeOwned> SeriesPattern<T> for SeriesPattern16<T> { fn get(&self, index: Index) -> Option<SeriesEndpoint<T>> { _I16.contains(&index).then(|| _ep(&self.by.client, &self.by.name, index)) } }

pub struct SeriesPattern17By<T> { client: Arc<BrkClientBase>, name: Arc<str>, _marker: std::marker::PhantomData<T> }
impl<T: DeserializeOwned> SeriesPattern17By<T> {
    pub fn epoch(&self) -> SeriesEndpoint<T> { _ep(&self.client, &self.name, Index::Epoch) }
}

pub struct SeriesPattern17<T> { name: Arc<str>, pub by: SeriesPattern17By<T> }
impl<T: DeserializeOwned> SeriesPattern17<T> {
    pub fn new(client: Arc<BrkClientBase>, name: String) -> Self { let name: Arc<str> = name.into(); Self { name: name.clone(), by: SeriesPattern17By { client, name, _marker: std::marker::PhantomData } } }
    pub fn name(&self) -> &str { &self.name }
}

impl<T> AnySeriesPattern for SeriesPattern17<T> { fn name(&self) -> &str { &self.name } fn indexes(&self) -> &'static [Index] { _I17 } }
impl<T: DeserializeOwned> SeriesPattern<T> for SeriesPattern17<T> { fn get(&self, index: Index) -> Option<SeriesEndpoint<T>> { _I17.contains(&index).then(|| _ep(&self.by.client, &self.by.name, index)) } }

pub struct SeriesPattern18By<T> { client: Arc<BrkClientBase>, name: Arc<str>, _marker: std::marker::PhantomData<T> }
impl<T: DeserializeOwned> SeriesPattern18By<T> {
    pub fn height(&self) -> SeriesEndpoint<T> { _ep(&self.client, &self.name, Index::Height) }
}

pub struct SeriesPattern18<T> { name: Arc<str>, pub by: SeriesPattern18By<T> }
impl<T: DeserializeOwned> SeriesPattern18<T> {
    pub fn new(client: Arc<BrkClientBase>, name: String) -> Self { let name: Arc<str> = name.into(); Self { name: name.clone(), by: SeriesPattern18By { client, name, _marker: std::marker::PhantomData } } }
    pub fn name(&self) -> &str { &self.name }
}

impl<T> AnySeriesPattern for SeriesPattern18<T> { fn name(&self) -> &str { &self.name } fn indexes(&self) -> &'static [Index] { _I18 } }
impl<T: DeserializeOwned> SeriesPattern<T> for SeriesPattern18<T> { fn get(&self, index: Index) -> Option<SeriesEndpoint<T>> { _I18.contains(&index).then(|| _ep(&self.by.client, &self.by.name, index)) } }

pub struct SeriesPattern19By<T> { client: Arc<BrkClientBase>, name: Arc<str>, _marker: std::marker::PhantomData<T> }
impl<T: DeserializeOwned> SeriesPattern19By<T> {
    pub fn tx_index(&self) -> SeriesEndpoint<T> { _ep(&self.client, &self.name, Index::TxIndex) }
}

pub struct SeriesPattern19<T> { name: Arc<str>, pub by: SeriesPattern19By<T> }
impl<T: DeserializeOwned> SeriesPattern19<T> {
    pub fn new(client: Arc<BrkClientBase>, name: String) -> Self { let name: Arc<str> = name.into(); Self { name: name.clone(), by: SeriesPattern19By { client, name, _marker: std::marker::PhantomData } } }
    pub fn name(&self) -> &str { &self.name }
}

impl<T> AnySeriesPattern for SeriesPattern19<T> { fn name(&self) -> &str { &self.name } fn indexes(&self) -> &'static [Index] { _I19 } }
impl<T: DeserializeOwned> SeriesPattern<T> for SeriesPattern19<T> { fn get(&self, index: Index) -> Option<SeriesEndpoint<T>> { _I19.contains(&index).then(|| _ep(&self.by.client, &self.by.name, index)) } }

pub struct SeriesPattern20By<T> { client: Arc<BrkClientBase>, name: Arc<str>, _marker: std::marker::PhantomData<T> }
impl<T: DeserializeOwned> SeriesPattern20By<T> {
    pub fn txin_index(&self) -> SeriesEndpoint<T> { _ep(&self.client, &self.name, Index::TxInIndex) }
}

pub struct SeriesPattern20<T> { name: Arc<str>, pub by: SeriesPattern20By<T> }
impl<T: DeserializeOwned> SeriesPattern20<T> {
    pub fn new(client: Arc<BrkClientBase>, name: String) -> Self { let name: Arc<str> = name.into(); Self { name: name.clone(), by: SeriesPattern20By { client, name, _marker: std::marker::PhantomData } } }
    pub fn name(&self) -> &str { &self.name }
}

impl<T> AnySeriesPattern for SeriesPattern20<T> { fn name(&self) -> &str { &self.name } fn indexes(&self) -> &'static [Index] { _I20 } }
impl<T: DeserializeOwned> SeriesPattern<T> for SeriesPattern20<T> { fn get(&self, index: Index) -> Option<SeriesEndpoint<T>> { _I20.contains(&index).then(|| _ep(&self.by.client, &self.by.name, index)) } }

pub struct SeriesPattern21By<T> { client: Arc<BrkClientBase>, name: Arc<str>, _marker: std::marker::PhantomData<T> }
impl<T: DeserializeOwned> SeriesPattern21By<T> {
    pub fn txout_index(&self) -> SeriesEndpoint<T> { _ep(&self.client, &self.name, Index::TxOutIndex) }
}

pub struct SeriesPattern21<T> { name: Arc<str>, pub by: SeriesPattern21By<T> }
impl<T: DeserializeOwned> SeriesPattern21<T> {
    pub fn new(client: Arc<BrkClientBase>, name: String) -> Self { let name: Arc<str> = name.into(); Self { name: name.clone(), by: SeriesPattern21By { client, name, _marker: std::marker::PhantomData } } }
    pub fn name(&self) -> &str { &self.name }
}

impl<T> AnySeriesPattern for SeriesPattern21<T> { fn name(&self) -> &str { &self.name } fn indexes(&self) -> &'static [Index] { _I21 } }
impl<T: DeserializeOwned> SeriesPattern<T> for SeriesPattern21<T> { fn get(&self, index: Index) -> Option<SeriesEndpoint<T>> { _I21.contains(&index).then(|| _ep(&self.by.client, &self.by.name, index)) } }

pub struct SeriesPattern22By<T> { client: Arc<BrkClientBase>, name: Arc<str>, _marker: std::marker::PhantomData<T> }
impl<T: DeserializeOwned> SeriesPattern22By<T> {
    pub fn empty_output_index(&self) -> SeriesEndpoint<T> { _ep(&self.client, &self.name, Index::EmptyOutputIndex) }
}

pub struct SeriesPattern22<T> { name: Arc<str>, pub by: SeriesPattern22By<T> }
impl<T: DeserializeOwned> SeriesPattern22<T> {
    pub fn new(client: Arc<BrkClientBase>, name: String) -> Self { let name: Arc<str> = name.into(); Self { name: name.clone(), by: SeriesPattern22By { client, name, _marker: std::marker::PhantomData } } }
    pub fn name(&self) -> &str { &self.name }
}

impl<T> AnySeriesPattern for SeriesPattern22<T> { fn name(&self) -> &str { &self.name } fn indexes(&self) -> &'static [Index] { _I22 } }
impl<T: DeserializeOwned> SeriesPattern<T> for SeriesPattern22<T> { fn get(&self, index: Index) -> Option<SeriesEndpoint<T>> { _I22.contains(&index).then(|| _ep(&self.by.client, &self.by.name, index)) } }

pub struct SeriesPattern23By<T> { client: Arc<BrkClientBase>, name: Arc<str>, _marker: std::marker::PhantomData<T> }
impl<T: DeserializeOwned> SeriesPattern23By<T> {
    pub fn op_return_index(&self) -> SeriesEndpoint<T> { _ep(&self.client, &self.name, Index::OpReturnIndex) }
}

pub struct SeriesPattern23<T> { name: Arc<str>, pub by: SeriesPattern23By<T> }
impl<T: DeserializeOwned> SeriesPattern23<T> {
    pub fn new(client: Arc<BrkClientBase>, name: String) -> Self { let name: Arc<str> = name.into(); Self { name: name.clone(), by: SeriesPattern23By { client, name, _marker: std::marker::PhantomData } } }
    pub fn name(&self) -> &str { &self.name }
}

impl<T> AnySeriesPattern for SeriesPattern23<T> { fn name(&self) -> &str { &self.name } fn indexes(&self) -> &'static [Index] { _I23 } }
impl<T: DeserializeOwned> SeriesPattern<T> for SeriesPattern23<T> { fn get(&self, index: Index) -> Option<SeriesEndpoint<T>> { _I23.contains(&index).then(|| _ep(&self.by.client, &self.by.name, index)) } }

pub struct SeriesPattern24By<T> { client: Arc<BrkClientBase>, name: Arc<str>, _marker: std::marker::PhantomData<T> }
impl<T: DeserializeOwned> SeriesPattern24By<T> {
    pub fn p2a_addr_index(&self) -> SeriesEndpoint<T> { _ep(&self.client, &self.name, Index::P2AAddrIndex) }
}

pub struct SeriesPattern24<T> { name: Arc<str>, pub by: SeriesPattern24By<T> }
impl<T: DeserializeOwned> SeriesPattern24<T> {
    pub fn new(client: Arc<BrkClientBase>, name: String) -> Self { let name: Arc<str> = name.into(); Self { name: name.clone(), by: SeriesPattern24By { client, name, _marker: std::marker::PhantomData } } }
    pub fn name(&self) -> &str { &self.name }
}

impl<T> AnySeriesPattern for SeriesPattern24<T> { fn name(&self) -> &str { &self.name } fn indexes(&self) -> &'static [Index] { _I24 } }
impl<T: DeserializeOwned> SeriesPattern<T> for SeriesPattern24<T> { fn get(&self, index: Index) -> Option<SeriesEndpoint<T>> { _I24.contains(&index).then(|| _ep(&self.by.client, &self.by.name, index)) } }

pub struct SeriesPattern25By<T> { client: Arc<BrkClientBase>, name: Arc<str>, _marker: std::marker::PhantomData<T> }
impl<T: DeserializeOwned> SeriesPattern25By<T> {
    pub fn p2ms_output_index(&self) -> SeriesEndpoint<T> { _ep(&self.client, &self.name, Index::P2MSOutputIndex) }
}

pub struct SeriesPattern25<T> { name: Arc<str>, pub by: SeriesPattern25By<T> }
impl<T: DeserializeOwned> SeriesPattern25<T> {
    pub fn new(client: Arc<BrkClientBase>, name: String) -> Self { let name: Arc<str> = name.into(); Self { name: name.clone(), by: SeriesPattern25By { client, name, _marker: std::marker::PhantomData } } }
    pub fn name(&self) -> &str { &self.name }
}

impl<T> AnySeriesPattern for SeriesPattern25<T> { fn name(&self) -> &str { &self.name } fn indexes(&self) -> &'static [Index] { _I25 } }
impl<T: DeserializeOwned> SeriesPattern<T> for SeriesPattern25<T> { fn get(&self, index: Index) -> Option<SeriesEndpoint<T>> { _I25.contains(&index).then(|| _ep(&self.by.client, &self.by.name, index)) } }

pub struct SeriesPattern26By<T> { client: Arc<BrkClientBase>, name: Arc<str>, _marker: std::marker::PhantomData<T> }
impl<T: DeserializeOwned> SeriesPattern26By<T> {
    pub fn p2pk33_addr_index(&self) -> SeriesEndpoint<T> { _ep(&self.client, &self.name, Index::P2PK33AddrIndex) }
}

pub struct SeriesPattern26<T> { name: Arc<str>, pub by: SeriesPattern26By<T> }
impl<T: DeserializeOwned> SeriesPattern26<T> {
    pub fn new(client: Arc<BrkClientBase>, name: String) -> Self { let name: Arc<str> = name.into(); Self { name: name.clone(), by: SeriesPattern26By { client, name, _marker: std::marker::PhantomData } } }
    pub fn name(&self) -> &str { &self.name }
}

impl<T> AnySeriesPattern for SeriesPattern26<T> { fn name(&self) -> &str { &self.name } fn indexes(&self) -> &'static [Index] { _I26 } }
impl<T: DeserializeOwned> SeriesPattern<T> for SeriesPattern26<T> { fn get(&self, index: Index) -> Option<SeriesEndpoint<T>> { _I26.contains(&index).then(|| _ep(&self.by.client, &self.by.name, index)) } }

pub struct SeriesPattern27By<T> { client: Arc<BrkClientBase>, name: Arc<str>, _marker: std::marker::PhantomData<T> }
impl<T: DeserializeOwned> SeriesPattern27By<T> {
    pub fn p2pk65_addr_index(&self) -> SeriesEndpoint<T> { _ep(&self.client, &self.name, Index::P2PK65AddrIndex) }
}

pub struct SeriesPattern27<T> { name: Arc<str>, pub by: SeriesPattern27By<T> }
impl<T: DeserializeOwned> SeriesPattern27<T> {
    pub fn new(client: Arc<BrkClientBase>, name: String) -> Self { let name: Arc<str> = name.into(); Self { name: name.clone(), by: SeriesPattern27By { client, name, _marker: std::marker::PhantomData } } }
    pub fn name(&self) -> &str { &self.name }
}

impl<T> AnySeriesPattern for SeriesPattern27<T> { fn name(&self) -> &str { &self.name } fn indexes(&self) -> &'static [Index] { _I27 } }
impl<T: DeserializeOwned> SeriesPattern<T> for SeriesPattern27<T> { fn get(&self, index: Index) -> Option<SeriesEndpoint<T>> { _I27.contains(&index).then(|| _ep(&self.by.client, &self.by.name, index)) } }

pub struct SeriesPattern28By<T> { client: Arc<BrkClientBase>, name: Arc<str>, _marker: std::marker::PhantomData<T> }
impl<T: DeserializeOwned> SeriesPattern28By<T> {
    pub fn p2pkh_addr_index(&self) -> SeriesEndpoint<T> { _ep(&self.client, &self.name, Index::P2PKHAddrIndex) }
}

pub struct SeriesPattern28<T> { name: Arc<str>, pub by: SeriesPattern28By<T> }
impl<T: DeserializeOwned> SeriesPattern28<T> {
    pub fn new(client: Arc<BrkClientBase>, name: String) -> Self { let name: Arc<str> = name.into(); Self { name: name.clone(), by: SeriesPattern28By { client, name, _marker: std::marker::PhantomData } } }
    pub fn name(&self) -> &str { &self.name }
}

impl<T> AnySeriesPattern for SeriesPattern28<T> { fn name(&self) -> &str { &self.name } fn indexes(&self) -> &'static [Index] { _I28 } }
impl<T: DeserializeOwned> SeriesPattern<T> for SeriesPattern28<T> { fn get(&self, index: Index) -> Option<SeriesEndpoint<T>> { _I28.contains(&index).then(|| _ep(&self.by.client, &self.by.name, index)) } }

pub struct SeriesPattern29By<T> { client: Arc<BrkClientBase>, name: Arc<str>, _marker: std::marker::PhantomData<T> }
impl<T: DeserializeOwned> SeriesPattern29By<T> {
    pub fn p2sh_addr_index(&self) -> SeriesEndpoint<T> { _ep(&self.client, &self.name, Index::P2SHAddrIndex) }
}

pub struct SeriesPattern29<T> { name: Arc<str>, pub by: SeriesPattern29By<T> }
impl<T: DeserializeOwned> SeriesPattern29<T> {
    pub fn new(client: Arc<BrkClientBase>, name: String) -> Self { let name: Arc<str> = name.into(); Self { name: name.clone(), by: SeriesPattern29By { client, name, _marker: std::marker::PhantomData } } }
    pub fn name(&self) -> &str { &self.name }
}

impl<T> AnySeriesPattern for SeriesPattern29<T> { fn name(&self) -> &str { &self.name } fn indexes(&self) -> &'static [Index] { _I29 } }
impl<T: DeserializeOwned> SeriesPattern<T> for SeriesPattern29<T> { fn get(&self, index: Index) -> Option<SeriesEndpoint<T>> { _I29.contains(&index).then(|| _ep(&self.by.client, &self.by.name, index)) } }

pub struct SeriesPattern30By<T> { client: Arc<BrkClientBase>, name: Arc<str>, _marker: std::marker::PhantomData<T> }
impl<T: DeserializeOwned> SeriesPattern30By<T> {
    pub fn p2tr_addr_index(&self) -> SeriesEndpoint<T> { _ep(&self.client, &self.name, Index::P2TRAddrIndex) }
}

pub struct SeriesPattern30<T> { name: Arc<str>, pub by: SeriesPattern30By<T> }
impl<T: DeserializeOwned> SeriesPattern30<T> {
    pub fn new(client: Arc<BrkClientBase>, name: String) -> Self { let name: Arc<str> = name.into(); Self { name: name.clone(), by: SeriesPattern30By { client, name, _marker: std::marker::PhantomData } } }
    pub fn name(&self) -> &str { &self.name }
}

impl<T> AnySeriesPattern for SeriesPattern30<T> { fn name(&self) -> &str { &self.name } fn indexes(&self) -> &'static [Index] { _I30 } }
impl<T: DeserializeOwned> SeriesPattern<T> for SeriesPattern30<T> { fn get(&self, index: Index) -> Option<SeriesEndpoint<T>> { _I30.contains(&index).then(|| _ep(&self.by.client, &self.by.name, index)) } }

pub struct SeriesPattern31By<T> { client: Arc<BrkClientBase>, name: Arc<str>, _marker: std::marker::PhantomData<T> }
impl<T: DeserializeOwned> SeriesPattern31By<T> {
    pub fn p2wpkh_addr_index(&self) -> SeriesEndpoint<T> { _ep(&self.client, &self.name, Index::P2WPKHAddrIndex) }
}

pub struct SeriesPattern31<T> { name: Arc<str>, pub by: SeriesPattern31By<T> }
impl<T: DeserializeOwned> SeriesPattern31<T> {
    pub fn new(client: Arc<BrkClientBase>, name: String) -> Self { let name: Arc<str> = name.into(); Self { name: name.clone(), by: SeriesPattern31By { client, name, _marker: std::marker::PhantomData } } }
    pub fn name(&self) -> &str { &self.name }
}

impl<T> AnySeriesPattern for SeriesPattern31<T> { fn name(&self) -> &str { &self.name } fn indexes(&self) -> &'static [Index] { _I31 } }
impl<T: DeserializeOwned> SeriesPattern<T> for SeriesPattern31<T> { fn get(&self, index: Index) -> Option<SeriesEndpoint<T>> { _I31.contains(&index).then(|| _ep(&self.by.client, &self.by.name, index)) } }

pub struct SeriesPattern32By<T> { client: Arc<BrkClientBase>, name: Arc<str>, _marker: std::marker::PhantomData<T> }
impl<T: DeserializeOwned> SeriesPattern32By<T> {
    pub fn p2wsh_addr_index(&self) -> SeriesEndpoint<T> { _ep(&self.client, &self.name, Index::P2WSHAddrIndex) }
}

pub struct SeriesPattern32<T> { name: Arc<str>, pub by: SeriesPattern32By<T> }
impl<T: DeserializeOwned> SeriesPattern32<T> {
    pub fn new(client: Arc<BrkClientBase>, name: String) -> Self { let name: Arc<str> = name.into(); Self { name: name.clone(), by: SeriesPattern32By { client, name, _marker: std::marker::PhantomData } } }
    pub fn name(&self) -> &str { &self.name }
}

impl<T> AnySeriesPattern for SeriesPattern32<T> { fn name(&self) -> &str { &self.name } fn indexes(&self) -> &'static [Index] { _I32 } }
impl<T: DeserializeOwned> SeriesPattern<T> for SeriesPattern32<T> { fn get(&self, index: Index) -> Option<SeriesEndpoint<T>> { _I32.contains(&index).then(|| _ep(&self.by.client, &self.by.name, index)) } }

pub struct SeriesPattern33By<T> { client: Arc<BrkClientBase>, name: Arc<str>, _marker: std::marker::PhantomData<T> }
impl<T: DeserializeOwned> SeriesPattern33By<T> {
    pub fn unknown_output_index(&self) -> SeriesEndpoint<T> { _ep(&self.client, &self.name, Index::UnknownOutputIndex) }
}

pub struct SeriesPattern33<T> { name: Arc<str>, pub by: SeriesPattern33By<T> }
impl<T: DeserializeOwned> SeriesPattern33<T> {
    pub fn new(client: Arc<BrkClientBase>, name: String) -> Self { let name: Arc<str> = name.into(); Self { name: name.clone(), by: SeriesPattern33By { client, name, _marker: std::marker::PhantomData } } }
    pub fn name(&self) -> &str { &self.name }
}

impl<T> AnySeriesPattern for SeriesPattern33<T> { fn name(&self) -> &str { &self.name } fn indexes(&self) -> &'static [Index] { _I33 } }
impl<T: DeserializeOwned> SeriesPattern<T> for SeriesPattern33<T> { fn get(&self, index: Index) -> Option<SeriesEndpoint<T>> { _I33.contains(&index).then(|| _ep(&self.by.client, &self.by.name, index)) } }

pub struct SeriesPattern34By<T> { client: Arc<BrkClientBase>, name: Arc<str>, _marker: std::marker::PhantomData<T> }
impl<T: DeserializeOwned> SeriesPattern34By<T> {
    pub fn funded_addr_index(&self) -> SeriesEndpoint<T> { _ep(&self.client, &self.name, Index::FundedAddrIndex) }
}

pub struct SeriesPattern34<T> { name: Arc<str>, pub by: SeriesPattern34By<T> }
impl<T: DeserializeOwned> SeriesPattern34<T> {
    pub fn new(client: Arc<BrkClientBase>, name: String) -> Self { let name: Arc<str> = name.into(); Self { name: name.clone(), by: SeriesPattern34By { client, name, _marker: std::marker::PhantomData } } }
    pub fn name(&self) -> &str { &self.name }
}

impl<T> AnySeriesPattern for SeriesPattern34<T> { fn name(&self) -> &str { &self.name } fn indexes(&self) -> &'static [Index] { _I34 } }
impl<T: DeserializeOwned> SeriesPattern<T> for SeriesPattern34<T> { fn get(&self, index: Index) -> Option<SeriesEndpoint<T>> { _I34.contains(&index).then(|| _ep(&self.by.client, &self.by.name, index)) } }

pub struct SeriesPattern35By<T> { client: Arc<BrkClientBase>, name: Arc<str>, _marker: std::marker::PhantomData<T> }
impl<T: DeserializeOwned> SeriesPattern35By<T> {
    pub fn empty_addr_index(&self) -> SeriesEndpoint<T> { _ep(&self.client, &self.name, Index::EmptyAddrIndex) }
}

pub struct SeriesPattern35<T> { name: Arc<str>, pub by: SeriesPattern35By<T> }
impl<T: DeserializeOwned> SeriesPattern35<T> {
    pub fn new(client: Arc<BrkClientBase>, name: String) -> Self { let name: Arc<str> = name.into(); Self { name: name.clone(), by: SeriesPattern35By { client, name, _marker: std::marker::PhantomData } } }
    pub fn name(&self) -> &str { &self.name }
}

impl<T> AnySeriesPattern for SeriesPattern35<T> { fn name(&self) -> &str { &self.name } fn indexes(&self) -> &'static [Index] { _I35 } }
impl<T: DeserializeOwned> SeriesPattern<T> for SeriesPattern35<T> { fn get(&self, index: Index) -> Option<SeriesEndpoint<T>> { _I35.contains(&index).then(|| _ep(&self.by.client, &self.by.name, index)) } }

// Reusable pattern structs

/// Pattern struct for repeated tree structure.
pub struct Pct05Pct10Pct15Pct20Pct25Pct30Pct35Pct40Pct45Pct50Pct55Pct60Pct65Pct70Pct75Pct80Pct85Pct90Pct95Pattern {
    pub pct05: CentsSatsUsdPattern,
    pub pct10: CentsSatsUsdPattern,
    pub pct15: CentsSatsUsdPattern,
    pub pct20: CentsSatsUsdPattern,
    pub pct25: CentsSatsUsdPattern,
    pub pct30: CentsSatsUsdPattern,
    pub pct35: CentsSatsUsdPattern,
    pub pct40: CentsSatsUsdPattern,
    pub pct45: CentsSatsUsdPattern,
    pub pct50: CentsSatsUsdPattern,
    pub pct55: CentsSatsUsdPattern,
    pub pct60: CentsSatsUsdPattern,
    pub pct65: CentsSatsUsdPattern,
    pub pct70: CentsSatsUsdPattern,
    pub pct75: CentsSatsUsdPattern,
    pub pct80: CentsSatsUsdPattern,
    pub pct85: CentsSatsUsdPattern,
    pub pct90: CentsSatsUsdPattern,
    pub pct95: CentsSatsUsdPattern,
}

impl Pct05Pct10Pct15Pct20Pct25Pct30Pct35Pct40Pct45Pct50Pct55Pct60Pct65Pct70Pct75Pct80Pct85Pct90Pct95Pattern {
    /// Create a new pattern node with accumulated series name.
    pub fn new(client: Arc<BrkClientBase>, acc: String) -> Self {
        Self {
            pct05: CentsSatsUsdPattern::new(client.clone(), _m(&acc, "pct05")),
            pct10: CentsSatsUsdPattern::new(client.clone(), _m(&acc, "pct10")),
            pct15: CentsSatsUsdPattern::new(client.clone(), _m(&acc, "pct15")),
            pct20: CentsSatsUsdPattern::new(client.clone(), _m(&acc, "pct20")),
            pct25: CentsSatsUsdPattern::new(client.clone(), _m(&acc, "pct25")),
            pct30: CentsSatsUsdPattern::new(client.clone(), _m(&acc, "pct30")),
            pct35: CentsSatsUsdPattern::new(client.clone(), _m(&acc, "pct35")),
            pct40: CentsSatsUsdPattern::new(client.clone(), _m(&acc, "pct40")),
            pct45: CentsSatsUsdPattern::new(client.clone(), _m(&acc, "pct45")),
            pct50: CentsSatsUsdPattern::new(client.clone(), _m(&acc, "pct50")),
            pct55: CentsSatsUsdPattern::new(client.clone(), _m(&acc, "pct55")),
            pct60: CentsSatsUsdPattern::new(client.clone(), _m(&acc, "pct60")),
            pct65: CentsSatsUsdPattern::new(client.clone(), _m(&acc, "pct65")),
            pct70: CentsSatsUsdPattern::new(client.clone(), _m(&acc, "pct70")),
            pct75: CentsSatsUsdPattern::new(client.clone(), _m(&acc, "pct75")),
            pct80: CentsSatsUsdPattern::new(client.clone(), _m(&acc, "pct80")),
            pct85: CentsSatsUsdPattern::new(client.clone(), _m(&acc, "pct85")),
            pct90: CentsSatsUsdPattern::new(client.clone(), _m(&acc, "pct90")),
            pct95: CentsSatsUsdPattern::new(client.clone(), _m(&acc, "pct95")),
        }
    }
}

/// Pattern struct for repeated tree structure.
pub struct _0sdM0M1M1sdM2M2sdM3sdP0P1P1sdP2P2sdP3sdSdZscorePattern {
    pub _0sd: CentsSatsUsdPattern,
    pub m0_5sd: PriceRatioPattern,
    pub m1_5sd: PriceRatioPattern,
    pub m1sd: PriceRatioPattern,
    pub m2_5sd: PriceRatioPattern,
    pub m2sd: PriceRatioPattern,
    pub m3sd: PriceRatioPattern,
    pub p0_5sd: PriceRatioPattern,
    pub p1_5sd: PriceRatioPattern,
    pub p1sd: PriceRatioPattern,
    pub p2_5sd: PriceRatioPattern,
    pub p2sd: PriceRatioPattern,
    pub p3sd: PriceRatioPattern,
    pub sd: SeriesPattern1<StoredF32>,
    pub zscore: SeriesPattern1<StoredF32>,
}

/// Pattern struct for repeated tree structure.
pub struct AllEmptyOpP2aP2msP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshUnknownPattern {
    pub all: AverageBlockCumulativeSumPattern<StoredU64>,
    pub empty: AverageBlockCumulativeSumPattern<StoredU64>,
    pub op_return: AverageBlockCumulativeSumPattern<StoredU64>,
    pub p2a: AverageBlockCumulativeSumPattern<StoredU64>,
    pub p2ms: AverageBlockCumulativeSumPattern<StoredU64>,
    pub p2pk33: AverageBlockCumulativeSumPattern<StoredU64>,
    pub p2pk65: AverageBlockCumulativeSumPattern<StoredU64>,
    pub p2pkh: AverageBlockCumulativeSumPattern<StoredU64>,
    pub p2sh: AverageBlockCumulativeSumPattern<StoredU64>,
    pub p2tr: AverageBlockCumulativeSumPattern<StoredU64>,
    pub p2wpkh: AverageBlockCumulativeSumPattern<StoredU64>,
    pub p2wsh: AverageBlockCumulativeSumPattern<StoredU64>,
    pub unknown: AverageBlockCumulativeSumPattern<StoredU64>,
}

impl AllEmptyOpP2aP2msP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshUnknownPattern {
    /// Create a new pattern node with accumulated series name.
    pub fn new(client: Arc<BrkClientBase>, acc: String) -> Self {
        Self {
            all: AverageBlockCumulativeSumPattern::new(client.clone(), _m(&acc, "bis")),
            empty: AverageBlockCumulativeSumPattern::new(client.clone(), _m(&acc, "with_empty_outputs_output")),
            op_return: AverageBlockCumulativeSumPattern::new(client.clone(), _m(&acc, "with_op_return_output")),
            p2a: AverageBlockCumulativeSumPattern::new(client.clone(), _m(&acc, "with_p2a_output")),
            p2ms: AverageBlockCumulativeSumPattern::new(client.clone(), _m(&acc, "with_p2ms_output")),
            p2pk33: AverageBlockCumulativeSumPattern::new(client.clone(), _m(&acc, "with_p2pk33_output")),
            p2pk65: AverageBlockCumulativeSumPattern::new(client.clone(), _m(&acc, "with_p2pk65_output")),
            p2pkh: AverageBlockCumulativeSumPattern::new(client.clone(), _m(&acc, "with_p2pkh_output")),
            p2sh: AverageBlockCumulativeSumPattern::new(client.clone(), _m(&acc, "with_p2sh_output")),
            p2tr: AverageBlockCumulativeSumPattern::new(client.clone(), _m(&acc, "with_p2tr_output")),
            p2wpkh: AverageBlockCumulativeSumPattern::new(client.clone(), _m(&acc, "with_p2wpkh_output")),
            p2wsh: AverageBlockCumulativeSumPattern::new(client.clone(), _m(&acc, "with_p2wsh_output")),
            unknown: AverageBlockCumulativeSumPattern::new(client.clone(), _m(&acc, "with_unknown_outputs_output")),
        }
    }
}

/// Pattern struct for repeated tree structure.
pub struct _10y1m1w1y2y3m3y4y5y6m6y8yPattern2 {
    pub _10y: BpsPercentRatioPattern,
    pub _1m: BpsPercentRatioPattern,
    pub _1w: BpsPercentRatioPattern,
    pub _1y: BpsPercentRatioPattern,
    pub _2y: BpsPercentRatioPattern,
    pub _3m: BpsPercentRatioPattern,
    pub _3y: BpsPercentRatioPattern,
    pub _4y: BpsPercentRatioPattern,
    pub _5y: BpsPercentRatioPattern,
    pub _6m: BpsPercentRatioPattern,
    pub _6y: BpsPercentRatioPattern,
    pub _8y: BpsPercentRatioPattern,
}

impl _10y1m1w1y2y3m3y4y5y6m6y8yPattern2 {
    /// Create a new pattern node with accumulated series name.
    pub fn new(client: Arc<BrkClientBase>, acc: String) -> Self {
        Self {
            _10y: BpsPercentRatioPattern::new(client.clone(), _m(&acc, "10y")),
            _1m: BpsPercentRatioPattern::new(client.clone(), _m(&acc, "1m")),
            _1w: BpsPercentRatioPattern::new(client.clone(), _m(&acc, "1w")),
            _1y: BpsPercentRatioPattern::new(client.clone(), _m(&acc, "1y")),
            _2y: BpsPercentRatioPattern::new(client.clone(), _m(&acc, "2y")),
            _3m: BpsPercentRatioPattern::new(client.clone(), _m(&acc, "3m")),
            _3y: BpsPercentRatioPattern::new(client.clone(), _m(&acc, "3y")),
            _4y: BpsPercentRatioPattern::new(client.clone(), _m(&acc, "4y")),
            _5y: BpsPercentRatioPattern::new(client.clone(), _m(&acc, "5y")),
            _6m: BpsPercentRatioPattern::new(client.clone(), _m(&acc, "6m")),
            _6y: BpsPercentRatioPattern::new(client.clone(), _m(&acc, "6y")),
            _8y: BpsPercentRatioPattern::new(client.clone(), _m(&acc, "8y")),
        }
    }
}

/// Pattern struct for repeated tree structure.
pub struct _10y1m1w1y2y3m3y4y5y6m6y8yPattern3 {
    pub _10y: BtcCentsSatsUsdPattern,
    pub _1m: BtcCentsSatsUsdPattern,
    pub _1w: BtcCentsSatsUsdPattern,
    pub _1y: BtcCentsSatsUsdPattern,
    pub _2y: BtcCentsSatsUsdPattern,
    pub _3m: BtcCentsSatsUsdPattern,
    pub _3y: BtcCentsSatsUsdPattern,
    pub _4y: BtcCentsSatsUsdPattern,
    pub _5y: BtcCentsSatsUsdPattern,
    pub _6m: BtcCentsSatsUsdPattern,
    pub _6y: BtcCentsSatsUsdPattern,
    pub _8y: BtcCentsSatsUsdPattern,
}

impl _10y1m1w1y2y3m3y4y5y6m6y8yPattern3 {
    /// Create a new pattern node with accumulated series name.
    pub fn new(client: Arc<BrkClientBase>, acc: String) -> Self {
        Self {
            _10y: BtcCentsSatsUsdPattern::new(client.clone(), _m(&acc, "10y")),
            _1m: BtcCentsSatsUsdPattern::new(client.clone(), _m(&acc, "1m")),
            _1w: BtcCentsSatsUsdPattern::new(client.clone(), _m(&acc, "1w")),
            _1y: BtcCentsSatsUsdPattern::new(client.clone(), _m(&acc, "1y")),
            _2y: BtcCentsSatsUsdPattern::new(client.clone(), _m(&acc, "2y")),
            _3m: BtcCentsSatsUsdPattern::new(client.clone(), _m(&acc, "3m")),
            _3y: BtcCentsSatsUsdPattern::new(client.clone(), _m(&acc, "3y")),
            _4y: BtcCentsSatsUsdPattern::new(client.clone(), _m(&acc, "4y")),
            _5y: BtcCentsSatsUsdPattern::new(client.clone(), _m(&acc, "5y")),
            _6m: BtcCentsSatsUsdPattern::new(client.clone(), _m(&acc, "6m")),
            _6y: BtcCentsSatsUsdPattern::new(client.clone(), _m(&acc, "6y")),
            _8y: BtcCentsSatsUsdPattern::new(client.clone(), _m(&acc, "8y")),
        }
    }
}

/// Pattern struct for repeated tree structure.
pub struct AllEmptyP2aP2msP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshUnknownPattern {
    pub all: AverageBlockCumulativeSumPattern<StoredU64>,
    pub empty: AverageBlockCumulativeSumPattern<StoredU64>,
    pub p2a: AverageBlockCumulativeSumPattern<StoredU64>,
    pub p2ms: AverageBlockCumulativeSumPattern<StoredU64>,
    pub p2pk33: AverageBlockCumulativeSumPattern<StoredU64>,
    pub p2pk65: AverageBlockCumulativeSumPattern<StoredU64>,
    pub p2pkh: AverageBlockCumulativeSumPattern<StoredU64>,
    pub p2sh: AverageBlockCumulativeSumPattern<StoredU64>,
    pub p2tr: AverageBlockCumulativeSumPattern<StoredU64>,
    pub p2wpkh: AverageBlockCumulativeSumPattern<StoredU64>,
    pub p2wsh: AverageBlockCumulativeSumPattern<StoredU64>,
    pub unknown: AverageBlockCumulativeSumPattern<StoredU64>,
}

/// Pattern struct for repeated tree structure.
pub struct CapCapitalizedGrossLossMvrvNetPeakPriceProfitSellSoprPattern {
    pub cap: CentsDeltaToUsdPattern,
    pub capitalized: PricePattern,
    pub gross_pnl: BlockCumulativeSumPattern,
    pub loss: BlockCumulativeNegativeSumPattern,
    pub mvrv: SeriesPattern1<StoredF32>,
    pub net_pnl: BlockChangeCumulativeDeltaSumPattern,
    pub peak_regret: BlockCumulativeSumPattern,
    pub price: BpsCentsPercentilesRatioSatsSmaStdUsdPattern,
    pub profit: BlockCumulativeSumPattern,
    pub profit_to_loss_ratio: _1m1w1y24hPattern<StoredF64>,
    pub sell_side_risk_ratio: _1m1w1y24hPattern8,
    pub sopr: AdjustedRatioValuePattern,
}

/// Pattern struct for repeated tree structure.
pub struct CapCapitalizedGrossLossMvrvNetPeakPriceProfitSellSoprPattern2 {
    pub cap: CentsDeltaToUsdPattern,
    pub capitalized: PricePattern,
    pub gross_pnl: BlockCumulativeSumPattern,
    pub loss: BlockCumulativeNegativeSumPattern,
    pub mvrv: SeriesPattern1<StoredF32>,
    pub net_pnl: BlockChangeCumulativeDeltaSumPattern,
    pub peak_regret: BlockCumulativeSumPattern,
    pub price: BpsCentsPercentilesRatioSatsSmaStdUsdPattern,
    pub profit: BlockCumulativeSumPattern,
    pub profit_to_loss_ratio: _1m1w1y24hPattern<StoredF64>,
    pub sell_side_risk_ratio: _1m1w1y24hPattern8,
    pub sopr: RatioValuePattern2,
}

/// Pattern struct for repeated tree structure.
pub struct EmptyOpP2aP2msP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshUnknownPattern2 {
    pub empty: _1m1w1y24hBpsPercentRatioPattern,
    pub op_return: _1m1w1y24hBpsPercentRatioPattern,
    pub p2a: _1m1w1y24hBpsPercentRatioPattern,
    pub p2ms: _1m1w1y24hBpsPercentRatioPattern,
    pub p2pk33: _1m1w1y24hBpsPercentRatioPattern,
    pub p2pk65: _1m1w1y24hBpsPercentRatioPattern,
    pub p2pkh: _1m1w1y24hBpsPercentRatioPattern,
    pub p2sh: _1m1w1y24hBpsPercentRatioPattern,
    pub p2tr: _1m1w1y24hBpsPercentRatioPattern,
    pub p2wpkh: _1m1w1y24hBpsPercentRatioPattern,
    pub p2wsh: _1m1w1y24hBpsPercentRatioPattern,
    pub unknown: _1m1w1y24hBpsPercentRatioPattern,
}

impl EmptyOpP2aP2msP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshUnknownPattern2 {
    /// Create a new pattern node with accumulated series name.
    pub fn new(client: Arc<BrkClientBase>, acc: String) -> Self {
        Self {
            empty: _1m1w1y24hBpsPercentRatioPattern::new(client.clone(), _m(&acc, "empty_outputs_output")),
            op_return: _1m1w1y24hBpsPercentRatioPattern::new(client.clone(), _m(&acc, "op_return_output")),
            p2a: _1m1w1y24hBpsPercentRatioPattern::new(client.clone(), _m(&acc, "p2a_output")),
            p2ms: _1m1w1y24hBpsPercentRatioPattern::new(client.clone(), _m(&acc, "p2ms_output")),
            p2pk33: _1m1w1y24hBpsPercentRatioPattern::new(client.clone(), _m(&acc, "p2pk33_output")),
            p2pk65: _1m1w1y24hBpsPercentRatioPattern::new(client.clone(), _m(&acc, "p2pk65_output")),
            p2pkh: _1m1w1y24hBpsPercentRatioPattern::new(client.clone(), _m(&acc, "p2pkh_output")),
            p2sh: _1m1w1y24hBpsPercentRatioPattern::new(client.clone(), _m(&acc, "p2sh_output")),
            p2tr: _1m1w1y24hBpsPercentRatioPattern::new(client.clone(), _m(&acc, "p2tr_output")),
            p2wpkh: _1m1w1y24hBpsPercentRatioPattern::new(client.clone(), _m(&acc, "p2wpkh_output")),
            p2wsh: _1m1w1y24hBpsPercentRatioPattern::new(client.clone(), _m(&acc, "p2wsh_output")),
            unknown: _1m1w1y24hBpsPercentRatioPattern::new(client.clone(), _m(&acc, "unknown_outputs_output")),
        }
    }
}

/// Pattern struct for repeated tree structure.
pub struct AverageBlockCumulativeMaxMedianMinPct10Pct25Pct75Pct90SumPattern {
    pub average: _1m1w1y24hPattern<StoredF32>,
    pub block: SeriesPattern18<StoredU64>,
    pub cumulative: SeriesPattern1<StoredU64>,
    pub max: _1m1w1y24hPattern<StoredU64>,
    pub median: _1m1w1y24hPattern<StoredU64>,
    pub min: _1m1w1y24hPattern<StoredU64>,
    pub pct10: _1m1w1y24hPattern<StoredU64>,
    pub pct25: _1m1w1y24hPattern<StoredU64>,
    pub pct75: _1m1w1y24hPattern<StoredU64>,
    pub pct90: _1m1w1y24hPattern<StoredU64>,
    pub sum: _1m1w1y24hPattern<StoredU64>,
}

impl AverageBlockCumulativeMaxMedianMinPct10Pct25Pct75Pct90SumPattern {
    /// Create a new pattern node with accumulated series name.
    pub fn new(client: Arc<BrkClientBase>, acc: String) -> Self {
        Self {
            average: _1m1w1y24hPattern::new(client.clone(), _m(&acc, "average")),
            block: SeriesPattern18::new(client.clone(), acc.clone()),
            cumulative: SeriesPattern1::new(client.clone(), _m(&acc, "cumulative")),
            max: _1m1w1y24hPattern::new(client.clone(), _m(&acc, "max")),
            median: _1m1w1y24hPattern::new(client.clone(), _m(&acc, "median")),
            min: _1m1w1y24hPattern::new(client.clone(), _m(&acc, "min")),
            pct10: _1m1w1y24hPattern::new(client.clone(), _m(&acc, "pct10")),
            pct25: _1m1w1y24hPattern::new(client.clone(), _m(&acc, "pct25")),
            pct75: _1m1w1y24hPattern::new(client.clone(), _m(&acc, "pct75")),
            pct90: _1m1w1y24hPattern::new(client.clone(), _m(&acc, "pct90")),
            sum: _1m1w1y24hPattern::new(client.clone(), _m(&acc, "sum")),
        }
    }
}

/// Pattern struct for repeated tree structure.
pub struct EmptyP2aP2msP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshUnknownPattern2 {
    pub empty: _1m1w1y24hBpsPercentRatioPattern,
    pub p2a: _1m1w1y24hBpsPercentRatioPattern,
    pub p2ms: _1m1w1y24hBpsPercentRatioPattern,
    pub p2pk33: _1m1w1y24hBpsPercentRatioPattern,
    pub p2pk65: _1m1w1y24hBpsPercentRatioPattern,
    pub p2pkh: _1m1w1y24hBpsPercentRatioPattern,
    pub p2sh: _1m1w1y24hBpsPercentRatioPattern,
    pub p2tr: _1m1w1y24hBpsPercentRatioPattern,
    pub p2wpkh: _1m1w1y24hBpsPercentRatioPattern,
    pub p2wsh: _1m1w1y24hBpsPercentRatioPattern,
    pub unknown: _1m1w1y24hBpsPercentRatioPattern,
}

impl EmptyP2aP2msP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshUnknownPattern2 {
    /// Create a new pattern node with accumulated series name.
    pub fn new(client: Arc<BrkClientBase>, acc: String) -> Self {
        Self {
            empty: _1m1w1y24hBpsPercentRatioPattern::new(client.clone(), _m(&acc, "empty_outputs_prevout")),
            p2a: _1m1w1y24hBpsPercentRatioPattern::new(client.clone(), _m(&acc, "p2a_prevout")),
            p2ms: _1m1w1y24hBpsPercentRatioPattern::new(client.clone(), _m(&acc, "p2ms_prevout")),
            p2pk33: _1m1w1y24hBpsPercentRatioPattern::new(client.clone(), _m(&acc, "p2pk33_prevout")),
            p2pk65: _1m1w1y24hBpsPercentRatioPattern::new(client.clone(), _m(&acc, "p2pk65_prevout")),
            p2pkh: _1m1w1y24hBpsPercentRatioPattern::new(client.clone(), _m(&acc, "p2pkh_prevout")),
            p2sh: _1m1w1y24hBpsPercentRatioPattern::new(client.clone(), _m(&acc, "p2sh_prevout")),
            p2tr: _1m1w1y24hBpsPercentRatioPattern::new(client.clone(), _m(&acc, "p2tr_prevout")),
            p2wpkh: _1m1w1y24hBpsPercentRatioPattern::new(client.clone(), _m(&acc, "p2wpkh_prevout")),
            p2wsh: _1m1w1y24hBpsPercentRatioPattern::new(client.clone(), _m(&acc, "p2wsh_prevout")),
            unknown: _1m1w1y24hBpsPercentRatioPattern::new(client.clone(), _m(&acc, "unknown_outputs_prevout")),
        }
    }
}

/// Pattern struct for repeated tree structure.
pub struct AverageBaseCumulativeMaxMedianMinPct10Pct25Pct75Pct90SumPattern<T> {
    pub average: _1m1w1y24hPattern<T>,
    pub base: SeriesPattern18<T>,
    pub cumulative: SeriesPattern1<T>,
    pub max: _1m1w1y24hPattern<T>,
    pub median: _1m1w1y24hPattern<T>,
    pub min: _1m1w1y24hPattern<T>,
    pub pct10: _1m1w1y24hPattern<T>,
    pub pct25: _1m1w1y24hPattern<T>,
    pub pct75: _1m1w1y24hPattern<T>,
    pub pct90: _1m1w1y24hPattern<T>,
    pub sum: _1m1w1y24hPattern<T>,
}

impl<T: DeserializeOwned> AverageBaseCumulativeMaxMedianMinPct10Pct25Pct75Pct90SumPattern<T> {
    /// Create a new pattern node with accumulated series name.
    pub fn new(client: Arc<BrkClientBase>, acc: String) -> Self {
        Self {
            average: _1m1w1y24hPattern::new(client.clone(), _m(&acc, "average")),
            base: SeriesPattern18::new(client.clone(), acc.clone()),
            cumulative: SeriesPattern1::new(client.clone(), _m(&acc, "cumulative")),
            max: _1m1w1y24hPattern::new(client.clone(), _m(&acc, "max")),
            median: _1m1w1y24hPattern::new(client.clone(), _m(&acc, "median")),
            min: _1m1w1y24hPattern::new(client.clone(), _m(&acc, "min")),
            pct10: _1m1w1y24hPattern::new(client.clone(), _m(&acc, "pct10")),
            pct25: _1m1w1y24hPattern::new(client.clone(), _m(&acc, "pct25")),
            pct75: _1m1w1y24hPattern::new(client.clone(), _m(&acc, "pct75")),
            pct90: _1m1w1y24hPattern::new(client.clone(), _m(&acc, "pct90")),
            sum: _1m1w1y24hPattern::new(client.clone(), _m(&acc, "sum")),
        }
    }
}

/// Pattern struct for repeated tree structure.
pub struct AllP2aP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshSharePattern {
    pub all: BtcCentsSatsUsdPattern,
    pub p2a: BtcCentsSatsUsdPattern,
    pub p2pk33: BtcCentsSatsUsdPattern,
    pub p2pk65: BtcCentsSatsUsdPattern,
    pub p2pkh: BtcCentsSatsUsdPattern,
    pub p2sh: BtcCentsSatsUsdPattern,
    pub p2tr: BtcCentsSatsUsdPattern,
    pub p2wpkh: BtcCentsSatsUsdPattern,
    pub p2wsh: BtcCentsSatsUsdPattern,
    pub share: AllP2aP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshPattern5,
}

/// Pattern struct for repeated tree structure.
pub struct IndexPct0Pct1Pct2Pct5Pct95Pct98Pct99ScorePattern {
    pub index: SeriesPattern1<StoredI8>,
    pub pct0_5: CentsSatsUsdPattern,
    pub pct1: CentsSatsUsdPattern,
    pub pct2: CentsSatsUsdPattern,
    pub pct5: CentsSatsUsdPattern,
    pub pct95: CentsSatsUsdPattern,
    pub pct98: CentsSatsUsdPattern,
    pub pct99: CentsSatsUsdPattern,
    pub pct99_5: CentsSatsUsdPattern,
    pub score: SeriesPattern1<StoredI8>,
}

impl IndexPct0Pct1Pct2Pct5Pct95Pct98Pct99ScorePattern {
    /// Create a new pattern node with accumulated series name.
    pub fn new(client: Arc<BrkClientBase>, acc: String) -> Self {
        Self {
            index: SeriesPattern1::new(client.clone(), _m(&acc, "index")),
            pct0_5: CentsSatsUsdPattern::new(client.clone(), _m(&acc, "pct0_5")),
            pct1: CentsSatsUsdPattern::new(client.clone(), _m(&acc, "pct01")),
            pct2: CentsSatsUsdPattern::new(client.clone(), _m(&acc, "pct02")),
            pct5: CentsSatsUsdPattern::new(client.clone(), _m(&acc, "pct05")),
            pct95: CentsSatsUsdPattern::new(client.clone(), _m(&acc, "pct95")),
            pct98: CentsSatsUsdPattern::new(client.clone(), _m(&acc, "pct98")),
            pct99: CentsSatsUsdPattern::new(client.clone(), _m(&acc, "pct99")),
            pct99_5: CentsSatsUsdPattern::new(client.clone(), _m(&acc, "pct99_5")),
            score: SeriesPattern1::new(client.clone(), _m(&acc, "score")),
        }
    }
}

/// Pattern struct for repeated tree structure.
pub struct AllP2aP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshPattern6 {
    pub all: AverageBlockCumulativeSumPattern<StoredU64>,
    pub p2a: AverageBlockCumulativeSumPattern<StoredU64>,
    pub p2pk33: AverageBlockCumulativeSumPattern<StoredU64>,
    pub p2pk65: AverageBlockCumulativeSumPattern<StoredU64>,
    pub p2pkh: AverageBlockCumulativeSumPattern<StoredU64>,
    pub p2sh: AverageBlockCumulativeSumPattern<StoredU64>,
    pub p2tr: AverageBlockCumulativeSumPattern<StoredU64>,
    pub p2wpkh: AverageBlockCumulativeSumPattern<StoredU64>,
    pub p2wsh: AverageBlockCumulativeSumPattern<StoredU64>,
}

impl AllP2aP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshPattern6 {
    /// Create a new pattern node with accumulated series name.
    pub fn new(client: Arc<BrkClientBase>, acc: String) -> Self {
        Self {
            all: AverageBlockCumulativeSumPattern::new(client.clone(), acc.clone()),
            p2a: AverageBlockCumulativeSumPattern::new(client.clone(), _p("p2a", &acc)),
            p2pk33: AverageBlockCumulativeSumPattern::new(client.clone(), _p("p2pk33", &acc)),
            p2pk65: AverageBlockCumulativeSumPattern::new(client.clone(), _p("p2pk65", &acc)),
            p2pkh: AverageBlockCumulativeSumPattern::new(client.clone(), _p("p2pkh", &acc)),
            p2sh: AverageBlockCumulativeSumPattern::new(client.clone(), _p("p2sh", &acc)),
            p2tr: AverageBlockCumulativeSumPattern::new(client.clone(), _p("p2tr", &acc)),
            p2wpkh: AverageBlockCumulativeSumPattern::new(client.clone(), _p("p2wpkh", &acc)),
            p2wsh: AverageBlockCumulativeSumPattern::new(client.clone(), _p("p2wsh", &acc)),
        }
    }
}

/// Pattern struct for repeated tree structure.
pub struct AllP2aP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshPattern5 {
    pub all: BpsPercentRatioPattern2,
    pub p2a: BpsPercentRatioPattern2,
    pub p2pk33: BpsPercentRatioPattern2,
    pub p2pk65: BpsPercentRatioPattern2,
    pub p2pkh: BpsPercentRatioPattern2,
    pub p2sh: BpsPercentRatioPattern2,
    pub p2tr: BpsPercentRatioPattern2,
    pub p2wpkh: BpsPercentRatioPattern2,
    pub p2wsh: BpsPercentRatioPattern2,
}

impl AllP2aP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshPattern5 {
    /// Create a new pattern node with accumulated series name.
    pub fn new(client: Arc<BrkClientBase>, acc: String) -> Self {
        Self {
            all: BpsPercentRatioPattern2::new(client.clone(), acc.clone()),
            p2a: BpsPercentRatioPattern2::new(client.clone(), _p("p2a", &acc)),
            p2pk33: BpsPercentRatioPattern2::new(client.clone(), _p("p2pk33", &acc)),
            p2pk65: BpsPercentRatioPattern2::new(client.clone(), _p("p2pk65", &acc)),
            p2pkh: BpsPercentRatioPattern2::new(client.clone(), _p("p2pkh", &acc)),
            p2sh: BpsPercentRatioPattern2::new(client.clone(), _p("p2sh", &acc)),
            p2tr: BpsPercentRatioPattern2::new(client.clone(), _p("p2tr", &acc)),
            p2wpkh: BpsPercentRatioPattern2::new(client.clone(), _p("p2wpkh", &acc)),
            p2wsh: BpsPercentRatioPattern2::new(client.clone(), _p("p2wsh", &acc)),
        }
    }
}

/// Pattern struct for repeated tree structure.
pub struct AllP2aP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshPattern4 {
    pub all: SeriesPattern1<StoredU64>,
    pub p2a: SeriesPattern1<StoredU64>,
    pub p2pk33: SeriesPattern1<StoredU64>,
    pub p2pk65: SeriesPattern1<StoredU64>,
    pub p2pkh: SeriesPattern1<StoredU64>,
    pub p2sh: SeriesPattern1<StoredU64>,
    pub p2tr: SeriesPattern1<StoredU64>,
    pub p2wpkh: SeriesPattern1<StoredU64>,
    pub p2wsh: SeriesPattern1<StoredU64>,
}

impl AllP2aP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshPattern4 {
    /// Create a new pattern node with accumulated series name.
    pub fn new(client: Arc<BrkClientBase>, acc: String) -> Self {
        Self {
            all: SeriesPattern1::new(client.clone(), acc.clone()),
            p2a: SeriesPattern1::new(client.clone(), _p("p2a", &acc)),
            p2pk33: SeriesPattern1::new(client.clone(), _p("p2pk33", &acc)),
            p2pk65: SeriesPattern1::new(client.clone(), _p("p2pk65", &acc)),
            p2pkh: SeriesPattern1::new(client.clone(), _p("p2pkh", &acc)),
            p2sh: SeriesPattern1::new(client.clone(), _p("p2sh", &acc)),
            p2tr: SeriesPattern1::new(client.clone(), _p("p2tr", &acc)),
            p2wpkh: SeriesPattern1::new(client.clone(), _p("p2wpkh", &acc)),
            p2wsh: SeriesPattern1::new(client.clone(), _p("p2wsh", &acc)),
        }
    }
}

/// Pattern struct for repeated tree structure.
pub struct AllP2aP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshPattern7 {
    pub all: _1m1w1y24hBpsPercentRatioPattern,
    pub p2a: _1m1w1y24hBpsPercentRatioPattern,
    pub p2pk33: _1m1w1y24hBpsPercentRatioPattern,
    pub p2pk65: _1m1w1y24hBpsPercentRatioPattern,
    pub p2pkh: _1m1w1y24hBpsPercentRatioPattern,
    pub p2sh: _1m1w1y24hBpsPercentRatioPattern,
    pub p2tr: _1m1w1y24hBpsPercentRatioPattern,
    pub p2wpkh: _1m1w1y24hBpsPercentRatioPattern,
    pub p2wsh: _1m1w1y24hBpsPercentRatioPattern,
}

impl AllP2aP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshPattern7 {
    /// Create a new pattern node with accumulated series name.
    pub fn new(client: Arc<BrkClientBase>, acc: String) -> Self {
        Self {
            all: _1m1w1y24hBpsPercentRatioPattern::new(client.clone(), acc.clone()),
            p2a: _1m1w1y24hBpsPercentRatioPattern::new(client.clone(), _p("p2a", &acc)),
            p2pk33: _1m1w1y24hBpsPercentRatioPattern::new(client.clone(), _p("p2pk33", &acc)),
            p2pk65: _1m1w1y24hBpsPercentRatioPattern::new(client.clone(), _p("p2pk65", &acc)),
            p2pkh: _1m1w1y24hBpsPercentRatioPattern::new(client.clone(), _p("p2pkh", &acc)),
            p2sh: _1m1w1y24hBpsPercentRatioPattern::new(client.clone(), _p("p2sh", &acc)),
            p2tr: _1m1w1y24hBpsPercentRatioPattern::new(client.clone(), _p("p2tr", &acc)),
            p2wpkh: _1m1w1y24hBpsPercentRatioPattern::new(client.clone(), _p("p2wpkh", &acc)),
            p2wsh: _1m1w1y24hBpsPercentRatioPattern::new(client.clone(), _p("p2wsh", &acc)),
        }
    }
}

/// Pattern struct for repeated tree structure.
pub struct AverageMaxMedianMinPct10Pct25Pct75Pct90SumPattern {
    pub average: _1m1w1y24hPattern<StoredF32>,
    pub max: _1m1w1y24hPattern<StoredU64>,
    pub median: _1m1w1y24hPattern<StoredU64>,
    pub min: _1m1w1y24hPattern<StoredU64>,
    pub pct10: _1m1w1y24hPattern<StoredU64>,
    pub pct25: _1m1w1y24hPattern<StoredU64>,
    pub pct75: _1m1w1y24hPattern<StoredU64>,
    pub pct90: _1m1w1y24hPattern<StoredU64>,
    pub sum: _1m1w1y24hPattern<StoredU64>,
}

impl AverageMaxMedianMinPct10Pct25Pct75Pct90SumPattern {
    /// Create a new pattern node with accumulated series name.
    pub fn new(client: Arc<BrkClientBase>, acc: String) -> Self {
        Self {
            average: _1m1w1y24hPattern::new(client.clone(), _m(&acc, "average")),
            max: _1m1w1y24hPattern::new(client.clone(), _m(&acc, "max")),
            median: _1m1w1y24hPattern::new(client.clone(), _m(&acc, "median")),
            min: _1m1w1y24hPattern::new(client.clone(), _m(&acc, "min")),
            pct10: _1m1w1y24hPattern::new(client.clone(), _m(&acc, "pct10")),
            pct25: _1m1w1y24hPattern::new(client.clone(), _m(&acc, "pct25")),
            pct75: _1m1w1y24hPattern::new(client.clone(), _m(&acc, "pct75")),
            pct90: _1m1w1y24hPattern::new(client.clone(), _m(&acc, "pct90")),
            sum: _1m1w1y24hPattern::new(client.clone(), _m(&acc, "sum")),
        }
    }
}

/// Pattern struct for repeated tree structure.
pub struct CapitalizedGrossInvestedLossNetNuplProfitSentimentPattern2 {
    pub capitalized_cap_in_loss_raw: SeriesPattern18<CentsSquaredSats>,
    pub capitalized_cap_in_profit_raw: SeriesPattern18<CentsSquaredSats>,
    pub gross_pnl: CentsUsdPattern3,
    pub invested_capital: InPattern2,
    pub loss: CentsNegativeToUsdPattern2,
    pub net_pnl: CentsToUsdPattern3,
    pub nupl: BpsRatioPattern,
    pub profit: CentsToUsdPattern4,
    pub sentiment: GreedNetPainPattern,
}

impl CapitalizedGrossInvestedLossNetNuplProfitSentimentPattern2 {
    /// Create a new pattern node with accumulated series name.
    pub fn new(client: Arc<BrkClientBase>, acc: String) -> Self {
        Self {
            capitalized_cap_in_loss_raw: SeriesPattern18::new(client.clone(), _m(&acc, "capitalized_cap_in_loss_raw")),
            capitalized_cap_in_profit_raw: SeriesPattern18::new(client.clone(), _m(&acc, "capitalized_cap_in_profit_raw")),
            gross_pnl: CentsUsdPattern3::new(client.clone(), _m(&acc, "unrealized_gross_pnl")),
            invested_capital: InPattern2::new(client.clone(), _m(&acc, "invested_capital_in")),
            loss: CentsNegativeToUsdPattern2::new(client.clone(), _m(&acc, "unrealized_loss")),
            net_pnl: CentsToUsdPattern3::new(client.clone(), _m(&acc, "net_unrealized_pnl")),
            nupl: BpsRatioPattern::new(client.clone(), _m(&acc, "nupl")),
            profit: CentsToUsdPattern4::new(client.clone(), _m(&acc, "unrealized_profit")),
            sentiment: GreedNetPainPattern::new(client.clone(), acc.clone()),
        }
    }
}

/// Pattern struct for repeated tree structure.
pub struct BpsCentsPercentilesRatioSatsSmaStdUsdPattern {
    pub bps: SeriesPattern1<BasisPoints32>,
    pub cents: SeriesPattern1<Cents>,
    pub percentiles: Pct0Pct1Pct2Pct5Pct95Pct98Pct99Pattern,
    pub ratio: SeriesPattern1<StoredF32>,
    pub sats: SeriesPattern1<SatsFract>,
    pub sma: _1m1w1y2y4yAllPattern,
    pub std_dev: _1y2y4yAllPattern,
    pub usd: SeriesPattern1<Dollars>,
}

/// Pattern struct for repeated tree structure.
pub struct Pct0Pct1Pct2Pct5Pct95Pct98Pct99Pattern {
    pub pct0_5: BpsPriceRatioPattern,
    pub pct1: BpsPriceRatioPattern,
    pub pct2: BpsPriceRatioPattern,
    pub pct5: BpsPriceRatioPattern,
    pub pct95: BpsPriceRatioPattern,
    pub pct98: BpsPriceRatioPattern,
    pub pct99: BpsPriceRatioPattern,
    pub pct99_5: BpsPriceRatioPattern,
}

impl Pct0Pct1Pct2Pct5Pct95Pct98Pct99Pattern {
    /// Create a new pattern node with accumulated series name.
    pub fn new(client: Arc<BrkClientBase>, acc: String) -> Self {
        Self {
            pct0_5: BpsPriceRatioPattern::new(client.clone(), acc.clone(), "pct0_5".to_string()),
            pct1: BpsPriceRatioPattern::new(client.clone(), acc.clone(), "pct1".to_string()),
            pct2: BpsPriceRatioPattern::new(client.clone(), acc.clone(), "pct2".to_string()),
            pct5: BpsPriceRatioPattern::new(client.clone(), acc.clone(), "pct5".to_string()),
            pct95: BpsPriceRatioPattern::new(client.clone(), acc.clone(), "pct95".to_string()),
            pct98: BpsPriceRatioPattern::new(client.clone(), acc.clone(), "pct98".to_string()),
            pct99: BpsPriceRatioPattern::new(client.clone(), acc.clone(), "pct99".to_string()),
            pct99_5: BpsPriceRatioPattern::new(client.clone(), acc.clone(), "pct99_5".to_string()),
        }
    }
}

/// Pattern struct for repeated tree structure.
pub struct _10y2y3y4y5y6y8yPattern {
    pub _10y: BpsPercentRatioPattern,
    pub _2y: BpsPercentRatioPattern,
    pub _3y: BpsPercentRatioPattern,
    pub _4y: BpsPercentRatioPattern,
    pub _5y: BpsPercentRatioPattern,
    pub _6y: BpsPercentRatioPattern,
    pub _8y: BpsPercentRatioPattern,
}

impl _10y2y3y4y5y6y8yPattern {
    /// Create a new pattern node with accumulated series name.
    pub fn new(client: Arc<BrkClientBase>, acc: String) -> Self {
        Self {
            _10y: BpsPercentRatioPattern::new(client.clone(), _m(&acc, "10y")),
            _2y: BpsPercentRatioPattern::new(client.clone(), _m(&acc, "2y")),
            _3y: BpsPercentRatioPattern::new(client.clone(), _m(&acc, "3y")),
            _4y: BpsPercentRatioPattern::new(client.clone(), _m(&acc, "4y")),
            _5y: BpsPercentRatioPattern::new(client.clone(), _m(&acc, "5y")),
            _6y: BpsPercentRatioPattern::new(client.clone(), _m(&acc, "6y")),
            _8y: BpsPercentRatioPattern::new(client.clone(), _m(&acc, "8y")),
        }
    }
}

/// Pattern struct for repeated tree structure.
pub struct _1m1w1y24hBpsPercentRatioPattern {
    pub _1m: BpsPercentRatioPattern2,
    pub _1w: BpsPercentRatioPattern2,
    pub _1y: BpsPercentRatioPattern2,
    pub _24h: BpsPercentRatioPattern2,
    pub bps: SeriesPattern1<BasisPoints16>,
    pub percent: SeriesPattern1<StoredF32>,
    pub ratio: SeriesPattern1<StoredF32>,
}

impl _1m1w1y24hBpsPercentRatioPattern {
    /// Create a new pattern node with accumulated series name.
    pub fn new(client: Arc<BrkClientBase>, acc: String) -> Self {
        Self {
            _1m: BpsPercentRatioPattern2::new(client.clone(), _m(&acc, "1m")),
            _1w: BpsPercentRatioPattern2::new(client.clone(), _m(&acc, "1w")),
            _1y: BpsPercentRatioPattern2::new(client.clone(), _m(&acc, "1y")),
            _24h: BpsPercentRatioPattern2::new(client.clone(), _m(&acc, "24h")),
            bps: SeriesPattern1::new(client.clone(), _m(&acc, "bps")),
            percent: SeriesPattern1::new(client.clone(), acc.clone()),
            ratio: SeriesPattern1::new(client.clone(), _m(&acc, "ratio")),
        }
    }
}

/// Pattern struct for repeated tree structure.
pub struct ActiveInputOutputSpendablePattern {
    pub active_reused_addr_count: _1m1w1y24hBlockPattern,
    pub active_reused_addr_share: _1m1w1y24hBlockPattern2,
    pub input_from_reused_addr_count: AllP2aP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshPattern6,
    pub input_from_reused_addr_share: AllP2aP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshPattern7,
    pub output_to_reused_addr_count: AllP2aP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshPattern6,
    pub output_to_reused_addr_share: AllP2aP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshPattern7,
    pub spendable_output_to_reused_addr_share: _1m1w1y24hBpsPercentRatioPattern,
}

/// Pattern struct for repeated tree structure.
pub struct ActivityCostInvestedOutputsRealizedSupplyUnrealizedPattern2 {
    pub activity: CoindaysCoinyearsDormancyTransferPattern,
    pub cost_basis: InMaxMinPerSupplyPattern,
    pub invested_capital: InPattern,
    pub outputs: SpendingSpentUnspentPattern,
    pub realized: CapCapitalizedGrossLossMvrvNetPeakPriceProfitSellSoprPattern2,
    pub supply: DeltaDominanceHalfInTotalPattern2,
    pub unrealized: CapitalizedGrossInvestedLossNetNuplProfitSentimentPattern2,
}

/// Pattern struct for repeated tree structure.
pub struct CapLossMvrvNetPriceProfitSoprPattern {
    pub cap: CentsDeltaUsdPattern,
    pub loss: BlockCumulativeNegativeSumPattern,
    pub mvrv: SeriesPattern1<StoredF32>,
    pub net_pnl: BlockCumulativeDeltaSumPattern,
    pub price: BpsCentsRatioSatsUsdPattern,
    pub profit: BlockCumulativeSumPattern,
    pub sopr: RatioValuePattern,
}

impl CapLossMvrvNetPriceProfitSoprPattern {
    /// Create a new pattern node with accumulated series name.
    pub fn new(client: Arc<BrkClientBase>, acc: String) -> Self {
        Self {
            cap: CentsDeltaUsdPattern::new(client.clone(), _m(&acc, "realized_cap")),
            loss: BlockCumulativeNegativeSumPattern::new(client.clone(), _m(&acc, "realized_loss")),
            mvrv: SeriesPattern1::new(client.clone(), _m(&acc, "mvrv")),
            net_pnl: BlockCumulativeDeltaSumPattern::new(client.clone(), _m(&acc, "net_realized_pnl")),
            price: BpsCentsRatioSatsUsdPattern::new(client.clone(), _m(&acc, "realized_price")),
            profit: BlockCumulativeSumPattern::new(client.clone(), _m(&acc, "realized_profit")),
            sopr: RatioValuePattern::new(client.clone(), acc.clone()),
        }
    }
}

/// Pattern struct for repeated tree structure.
pub struct InMaxMinPerSupplyPattern {
    pub in_loss: PerPattern,
    pub in_profit: PerPattern,
    pub max: CentsSatsUsdPattern,
    pub min: CentsSatsUsdPattern,
    pub per_coin: Pct05Pct10Pct15Pct20Pct25Pct30Pct35Pct40Pct45Pct50Pct55Pct60Pct65Pct70Pct75Pct80Pct85Pct90Pct95Pattern,
    pub per_dollar: Pct05Pct10Pct15Pct20Pct25Pct30Pct35Pct40Pct45Pct50Pct55Pct60Pct65Pct70Pct75Pct80Pct85Pct90Pct95Pattern,
    pub supply_density: BpsPercentRatioPattern2,
}

impl InMaxMinPerSupplyPattern {
    /// Create a new pattern node with accumulated series name.
    pub fn new(client: Arc<BrkClientBase>, acc: String) -> Self {
        Self {
            in_loss: PerPattern::new(client.clone(), _m(&acc, "cost_basis_in_loss_per")),
            in_profit: PerPattern::new(client.clone(), _m(&acc, "cost_basis_in_profit_per")),
            max: CentsSatsUsdPattern::new(client.clone(), _m(&acc, "cost_basis_max")),
            min: CentsSatsUsdPattern::new(client.clone(), _m(&acc, "cost_basis_min")),
            per_coin: Pct05Pct10Pct15Pct20Pct25Pct30Pct35Pct40Pct45Pct50Pct55Pct60Pct65Pct70Pct75Pct80Pct85Pct90Pct95Pattern::new(client.clone(), _m(&acc, "cost_basis_per_coin")),
            per_dollar: Pct05Pct10Pct15Pct20Pct25Pct30Pct35Pct40Pct45Pct50Pct55Pct60Pct65Pct70Pct75Pct80Pct85Pct90Pct95Pattern::new(client.clone(), _m(&acc, "cost_basis_per_dollar")),
            supply_density: BpsPercentRatioPattern2::new(client.clone(), _m(&acc, "supply_density")),
        }
    }
}

/// Pattern struct for repeated tree structure.
pub struct MaxMedianMinPct10Pct25Pct75Pct90Pattern2 {
    pub max: SeriesPattern18<Weight>,
    pub median: SeriesPattern18<Weight>,
    pub min: SeriesPattern18<Weight>,
    pub pct10: SeriesPattern18<Weight>,
    pub pct25: SeriesPattern18<Weight>,
    pub pct75: SeriesPattern18<Weight>,
    pub pct90: SeriesPattern18<Weight>,
}

impl MaxMedianMinPct10Pct25Pct75Pct90Pattern2 {
    /// Create a new pattern node with accumulated series name.
    pub fn new(client: Arc<BrkClientBase>, acc: String) -> Self {
        Self {
            max: SeriesPattern18::new(client.clone(), _m(&acc, "max")),
            median: SeriesPattern18::new(client.clone(), _m(&acc, "median")),
            min: SeriesPattern18::new(client.clone(), _m(&acc, "min")),
            pct10: SeriesPattern18::new(client.clone(), _m(&acc, "pct10")),
            pct25: SeriesPattern18::new(client.clone(), _m(&acc, "pct25")),
            pct75: SeriesPattern18::new(client.clone(), _m(&acc, "pct75")),
            pct90: SeriesPattern18::new(client.clone(), _m(&acc, "pct90")),
        }
    }
}

/// Pattern struct for repeated tree structure.
pub struct MaxMedianMinPct10Pct25Pct75Pct90Pattern<T> {
    pub max: SeriesPattern1<T>,
    pub median: SeriesPattern1<T>,
    pub min: SeriesPattern1<T>,
    pub pct10: SeriesPattern1<T>,
    pub pct25: SeriesPattern1<T>,
    pub pct75: SeriesPattern1<T>,
    pub pct90: SeriesPattern1<T>,
}

impl<T: DeserializeOwned> MaxMedianMinPct10Pct25Pct75Pct90Pattern<T> {
    /// Create a new pattern node with accumulated series name.
    pub fn new(client: Arc<BrkClientBase>, acc: String) -> Self {
        Self {
            max: SeriesPattern1::new(client.clone(), _m(&acc, "max")),
            median: SeriesPattern1::new(client.clone(), _m(&acc, "median")),
            min: SeriesPattern1::new(client.clone(), _m(&acc, "min")),
            pct10: SeriesPattern1::new(client.clone(), _m(&acc, "pct10")),
            pct25: SeriesPattern1::new(client.clone(), _m(&acc, "pct25")),
            pct75: SeriesPattern1::new(client.clone(), _m(&acc, "pct75")),
            pct90: SeriesPattern1::new(client.clone(), _m(&acc, "pct90")),
        }
    }
}

/// Pattern struct for repeated tree structure.
pub struct _1m1w1y2y4yAllPattern {
    pub _1m: BpsRatioPattern2,
    pub _1w: BpsRatioPattern2,
    pub _1y: BpsRatioPattern2,
    pub _2y: BpsRatioPattern2,
    pub _4y: BpsRatioPattern2,
    pub all: BpsRatioPattern2,
}

impl _1m1w1y2y4yAllPattern {
    /// Create a new pattern node with accumulated series name.
    pub fn new(client: Arc<BrkClientBase>, acc: String) -> Self {
        Self {
            _1m: BpsRatioPattern2::new(client.clone(), _m(&acc, "1m")),
            _1w: BpsRatioPattern2::new(client.clone(), _m(&acc, "1w")),
            _1y: BpsRatioPattern2::new(client.clone(), _m(&acc, "1y")),
            _2y: BpsRatioPattern2::new(client.clone(), _m(&acc, "2y")),
            _4y: BpsRatioPattern2::new(client.clone(), _m(&acc, "4y")),
            all: BpsRatioPattern2::new(client.clone(), _m(&acc, "all")),
        }
    }
}

/// Pattern struct for repeated tree structure.
pub struct ActivityAddrOutputsRealizedSupplyUnrealizedPattern {
    pub activity: TransferPattern,
    pub addr_count: BaseDeltaPattern,
    pub outputs: SpendingSpentUnspentPattern,
    pub realized: CapLossMvrvPriceProfitPattern,
    pub supply: DeltaDominanceTotalPattern,
    pub unrealized: NuplPattern,
}

impl ActivityAddrOutputsRealizedSupplyUnrealizedPattern {
    /// Create a new pattern node with accumulated series name.
    pub fn new(client: Arc<BrkClientBase>, acc: String) -> Self {
        Self {
            activity: TransferPattern::new(client.clone(), _m(&acc, "transfer_volume")),
            addr_count: BaseDeltaPattern::new(client.clone(), _m(&acc, "addr_count")),
            outputs: SpendingSpentUnspentPattern::new(client.clone(), acc.clone()),
            realized: CapLossMvrvPriceProfitPattern::new(client.clone(), acc.clone()),
            supply: DeltaDominanceTotalPattern::new(client.clone(), _m(&acc, "supply")),
            unrealized: NuplPattern::new(client.clone(), _m(&acc, "nupl")),
        }
    }
}

/// Pattern struct for repeated tree structure.
pub struct AverageBlockCumulativeInSumPattern {
    pub average: _1m1w1y24hPattern3,
    pub block: BtcCentsSatsUsdPattern3,
    pub cumulative: BtcCentsSatsUsdPattern,
    pub in_loss: AverageBlockCumulativeSumPattern3,
    pub in_profit: AverageBlockCumulativeSumPattern3,
    pub sum: _1m1w1y24hPattern4,
}

impl AverageBlockCumulativeInSumPattern {
    /// Create a new pattern node with accumulated series name.
    pub fn new(client: Arc<BrkClientBase>, acc: String) -> Self {
        Self {
            average: _1m1w1y24hPattern3::new(client.clone(), _m(&acc, "average")),
            block: BtcCentsSatsUsdPattern3::new(client.clone(), acc.clone()),
            cumulative: BtcCentsSatsUsdPattern::new(client.clone(), _m(&acc, "cumulative")),
            in_loss: AverageBlockCumulativeSumPattern3::new(client.clone(), _m(&acc, "in_loss")),
            in_profit: AverageBlockCumulativeSumPattern3::new(client.clone(), _m(&acc, "in_profit")),
            sum: _1m1w1y24hPattern4::new(client.clone(), _m(&acc, "sum")),
        }
    }
}

/// Pattern struct for repeated tree structure.
pub struct BpsCentsPercentilesRatioSatsUsdPattern {
    pub bps: SeriesPattern1<BasisPoints32>,
    pub cents: SeriesPattern1<Cents>,
    pub percentiles: Pct0Pct1Pct2Pct5Pct95Pct98Pct99Pattern,
    pub ratio: SeriesPattern1<StoredF32>,
    pub sats: SeriesPattern1<SatsFract>,
    pub usd: SeriesPattern1<Dollars>,
}

impl BpsCentsPercentilesRatioSatsUsdPattern {
    /// Create a new pattern node with accumulated series name.
    pub fn new(client: Arc<BrkClientBase>, acc: String) -> Self {
        Self {
            bps: SeriesPattern1::new(client.clone(), _m(&acc, "ratio_bps")),
            cents: SeriesPattern1::new(client.clone(), _m(&acc, "cents")),
            percentiles: Pct0Pct1Pct2Pct5Pct95Pct98Pct99Pattern::new(client.clone(), acc.clone()),
            ratio: SeriesPattern1::new(client.clone(), _m(&acc, "ratio")),
            sats: SeriesPattern1::new(client.clone(), _m(&acc, "sats")),
            usd: SeriesPattern1::new(client.clone(), acc.clone()),
        }
    }
}

/// Pattern struct for repeated tree structure.
pub struct CentsNegativeToUsdPattern2 {
    pub cents: SeriesPattern1<Cents>,
    pub negative: SeriesPattern1<Dollars>,
    pub to_mcap: BpsPercentRatioPattern2,
    pub to_own_gross_pnl: BpsPercentRatioPattern2,
    pub to_own_mcap: BpsPercentRatioPattern4,
    pub usd: SeriesPattern1<Dollars>,
}

impl CentsNegativeToUsdPattern2 {
    /// Create a new pattern node with accumulated series name.
    pub fn new(client: Arc<BrkClientBase>, acc: String) -> Self {
        Self {
            cents: SeriesPattern1::new(client.clone(), _m(&acc, "cents")),
            negative: SeriesPattern1::new(client.clone(), _m(&acc, "neg")),
            to_mcap: BpsPercentRatioPattern2::new(client.clone(), _m(&acc, "to_mcap")),
            to_own_gross_pnl: BpsPercentRatioPattern2::new(client.clone(), _m(&acc, "to_own_gross_pnl")),
            to_own_mcap: BpsPercentRatioPattern4::new(client.clone(), _m(&acc, "to_own_mcap")),
            usd: SeriesPattern1::new(client.clone(), acc.clone()),
        }
    }
}

/// Pattern struct for repeated tree structure.
pub struct DeltaDominanceHalfInTotalPattern2 {
    pub delta: AbsoluteRatePattern3,
    pub dominance: BpsPercentRatioPattern2,
    pub half: BtcCentsSatsUsdPattern,
    pub in_loss: BtcCentsSatsShareUsdPattern,
    pub in_profit: BtcCentsSatsShareUsdPattern,
    pub total: BtcCentsSatsUsdPattern,
}

impl DeltaDominanceHalfInTotalPattern2 {
    /// Create a new pattern node with accumulated series name.
    pub fn new(client: Arc<BrkClientBase>, acc: String) -> Self {
        Self {
            delta: AbsoluteRatePattern3::new(client.clone(), _m(&acc, "delta")),
            dominance: BpsPercentRatioPattern2::new(client.clone(), _m(&acc, "dominance")),
            half: BtcCentsSatsUsdPattern::new(client.clone(), _m(&acc, "half")),
            in_loss: BtcCentsSatsShareUsdPattern::new(client.clone(), _m(&acc, "in_loss")),
            in_profit: BtcCentsSatsShareUsdPattern::new(client.clone(), _m(&acc, "in_profit")),
            total: BtcCentsSatsUsdPattern::new(client.clone(), acc.clone()),
        }
    }
}

/// Pattern struct for repeated tree structure.
pub struct DeltaDominanceHalfInTotalPattern {
    pub delta: AbsoluteRatePattern3,
    pub dominance: BpsPercentRatioPattern2,
    pub half: BtcCentsSatsUsdPattern,
    pub in_loss: BtcCentsSatsUsdPattern,
    pub in_profit: BtcCentsSatsUsdPattern,
    pub total: BtcCentsSatsUsdPattern,
}

impl DeltaDominanceHalfInTotalPattern {
    /// Create a new pattern node with accumulated series name.
    pub fn new(client: Arc<BrkClientBase>, acc: String) -> Self {
        Self {
            delta: AbsoluteRatePattern3::new(client.clone(), _m(&acc, "delta")),
            dominance: BpsPercentRatioPattern2::new(client.clone(), _m(&acc, "dominance")),
            half: BtcCentsSatsUsdPattern::new(client.clone(), _m(&acc, "half")),
            in_loss: BtcCentsSatsUsdPattern::new(client.clone(), _m(&acc, "in_loss")),
            in_profit: BtcCentsSatsUsdPattern::new(client.clone(), _m(&acc, "in_profit")),
            total: BtcCentsSatsUsdPattern::new(client.clone(), acc.clone()),
        }
    }
}

/// Pattern struct for repeated tree structure.
pub struct _1m1w1y24hBlockPattern2 {
    pub _1m: SeriesPattern1<StoredF32>,
    pub _1w: SeriesPattern1<StoredF32>,
    pub _1y: SeriesPattern1<StoredF32>,
    pub _24h: SeriesPattern1<StoredF32>,
    pub block: SeriesPattern18<StoredF32>,
}

impl _1m1w1y24hBlockPattern2 {
    /// Create a new pattern node with accumulated series name.
    pub fn new(client: Arc<BrkClientBase>, acc: String) -> Self {
        Self {
            _1m: SeriesPattern1::new(client.clone(), _m(&acc, "average_1m")),
            _1w: SeriesPattern1::new(client.clone(), _m(&acc, "average_1w")),
            _1y: SeriesPattern1::new(client.clone(), _m(&acc, "average_1y")),
            _24h: SeriesPattern1::new(client.clone(), _m(&acc, "average_24h")),
            block: SeriesPattern18::new(client.clone(), acc.clone()),
        }
    }
}

/// Pattern struct for repeated tree structure.
pub struct _1m1w1y24hBlockPattern {
    pub _1m: SeriesPattern1<StoredF32>,
    pub _1w: SeriesPattern1<StoredF32>,
    pub _1y: SeriesPattern1<StoredF32>,
    pub _24h: SeriesPattern1<StoredF32>,
    pub block: SeriesPattern18<StoredU32>,
}

impl _1m1w1y24hBlockPattern {
    /// Create a new pattern node with accumulated series name.
    pub fn new(client: Arc<BrkClientBase>, acc: String) -> Self {
        Self {
            _1m: SeriesPattern1::new(client.clone(), _m(&acc, "average_1m")),
            _1w: SeriesPattern1::new(client.clone(), _m(&acc, "average_1w")),
            _1y: SeriesPattern1::new(client.clone(), _m(&acc, "average_1y")),
            _24h: SeriesPattern1::new(client.clone(), _m(&acc, "average_24h")),
            block: SeriesPattern18::new(client.clone(), acc.clone()),
        }
    }
}

/// Pattern struct for repeated tree structure.
pub struct ActiveBidirectionalReactivatedReceivingSendingPattern {
    pub active: _1m1w1y24hBlockPattern,
    pub bidirectional: _1m1w1y24hBlockPattern,
    pub reactivated: _1m1w1y24hBlockPattern,
    pub receiving: _1m1w1y24hBlockPattern,
    pub sending: _1m1w1y24hBlockPattern,
}

impl ActiveBidirectionalReactivatedReceivingSendingPattern {
    /// Create a new pattern node with accumulated series name.
    pub fn new(client: Arc<BrkClientBase>, acc: String) -> Self {
        Self {
            active: _1m1w1y24hBlockPattern::new(client.clone(), _m(&acc, "active_addrs")),
            bidirectional: _1m1w1y24hBlockPattern::new(client.clone(), _m(&acc, "bidirectional_addrs")),
            reactivated: _1m1w1y24hBlockPattern::new(client.clone(), _m(&acc, "reactivated_addrs")),
            receiving: _1m1w1y24hBlockPattern::new(client.clone(), _m(&acc, "receiving_addrs")),
            sending: _1m1w1y24hBlockPattern::new(client.clone(), _m(&acc, "sending_addrs")),
        }
    }
}

/// Pattern struct for repeated tree structure.
pub struct ActivityOutputsRealizedSupplyUnrealizedPattern {
    pub activity: CoindaysTransferPattern,
    pub outputs: SpendingSpentUnspentPattern,
    pub realized: CapLossMvrvNetPriceProfitSoprPattern,
    pub supply: DeltaDominanceHalfInTotalPattern,
    pub unrealized: LossNetNuplProfitPattern,
}

impl ActivityOutputsRealizedSupplyUnrealizedPattern {
    /// Create a new pattern node with accumulated series name.
    pub fn new(client: Arc<BrkClientBase>, acc: String) -> Self {
        Self {
            activity: CoindaysTransferPattern::new(client.clone(), acc.clone()),
            outputs: SpendingSpentUnspentPattern::new(client.clone(), acc.clone()),
            realized: CapLossMvrvNetPriceProfitSoprPattern::new(client.clone(), acc.clone()),
            supply: DeltaDominanceHalfInTotalPattern::new(client.clone(), _m(&acc, "supply")),
            unrealized: LossNetNuplProfitPattern::new(client.clone(), acc.clone()),
        }
    }
}

/// Pattern struct for repeated tree structure.
pub struct ActivityOutputsRealizedSupplyUnrealizedPattern3 {
    pub activity: TransferPattern,
    pub outputs: SpendingSpentUnspentPattern,
    pub realized: CapLossMvrvPriceProfitPattern,
    pub supply: DeltaDominanceHalfInTotalPattern,
    pub unrealized: LossNuplProfitPattern,
}

impl ActivityOutputsRealizedSupplyUnrealizedPattern3 {
    /// Create a new pattern node with accumulated series name.
    pub fn new(client: Arc<BrkClientBase>, acc: String) -> Self {
        Self {
            activity: TransferPattern::new(client.clone(), _m(&acc, "transfer_volume")),
            outputs: SpendingSpentUnspentPattern::new(client.clone(), acc.clone()),
            realized: CapLossMvrvPriceProfitPattern::new(client.clone(), acc.clone()),
            supply: DeltaDominanceHalfInTotalPattern::new(client.clone(), _m(&acc, "supply")),
            unrealized: LossNuplProfitPattern::new(client.clone(), acc.clone()),
        }
    }
}

/// Pattern struct for repeated tree structure.
pub struct ActivityOutputsRealizedSupplyUnrealizedPattern2 {
    pub activity: TransferPattern,
    pub outputs: SpendingSpentUnspentPattern,
    pub realized: CapLossMvrvPriceProfitPattern,
    pub supply: DeltaDominanceTotalPattern,
    pub unrealized: NuplPattern,
}

impl ActivityOutputsRealizedSupplyUnrealizedPattern2 {
    /// Create a new pattern node with accumulated series name.
    pub fn new(client: Arc<BrkClientBase>, acc: String) -> Self {
        Self {
            activity: TransferPattern::new(client.clone(), _m(&acc, "transfer_volume")),
            outputs: SpendingSpentUnspentPattern::new(client.clone(), acc.clone()),
            realized: CapLossMvrvPriceProfitPattern::new(client.clone(), acc.clone()),
            supply: DeltaDominanceTotalPattern::new(client.clone(), _m(&acc, "supply")),
            unrealized: NuplPattern::new(client.clone(), _m(&acc, "nupl")),
        }
    }
}

/// Pattern struct for repeated tree structure.
pub struct BlockChangeCumulativeDeltaSumPattern {
    pub block: CentsUsdPattern4,
    pub change_1m: ToPattern,
    pub cumulative: CentsUsdPattern,
    pub delta: AbsoluteRatePattern2,
    pub sum: _1m1w1y24hPattern5,
}

impl BlockChangeCumulativeDeltaSumPattern {
    /// Create a new pattern node with accumulated series name.
    pub fn new(client: Arc<BrkClientBase>, acc: String) -> Self {
        Self {
            block: CentsUsdPattern4::new(client.clone(), _m(&acc, "realized_pnl")),
            change_1m: ToPattern::new(client.clone(), _m(&acc, "pnl_change_1m_to")),
            cumulative: CentsUsdPattern::new(client.clone(), _m(&acc, "realized_pnl_cumulative")),
            delta: AbsoluteRatePattern2::new(client.clone(), _m(&acc, "realized_pnl_delta")),
            sum: _1m1w1y24hPattern5::new(client.clone(), _m(&acc, "realized_pnl_sum")),
        }
    }
}

/// Pattern struct for repeated tree structure.
pub struct BpsCentsRatioSatsUsdPattern {
    pub bps: SeriesPattern1<BasisPoints32>,
    pub cents: SeriesPattern1<Cents>,
    pub ratio: SeriesPattern1<StoredF32>,
    pub sats: SeriesPattern1<SatsFract>,
    pub usd: SeriesPattern1<Dollars>,
}

impl BpsCentsRatioSatsUsdPattern {
    /// Create a new pattern node with accumulated series name.
    pub fn new(client: Arc<BrkClientBase>, acc: String) -> Self {
        Self {
            bps: SeriesPattern1::new(client.clone(), _m(&acc, "ratio_bps")),
            cents: SeriesPattern1::new(client.clone(), _m(&acc, "cents")),
            ratio: SeriesPattern1::new(client.clone(), _m(&acc, "ratio")),
            sats: SeriesPattern1::new(client.clone(), _m(&acc, "sats")),
            usd: SeriesPattern1::new(client.clone(), acc.clone()),
        }
    }
}

/// Pattern struct for repeated tree structure.
pub struct BtcCentsDeltaSatsUsdPattern {
    pub btc: SeriesPattern1<Bitcoin>,
    pub cents: SeriesPattern1<Cents>,
    pub delta: AbsoluteRatePattern3,
    pub sats: SeriesPattern1<Sats>,
    pub usd: SeriesPattern1<Dollars>,
}

impl BtcCentsDeltaSatsUsdPattern {
    /// Create a new pattern node with accumulated series name.
    pub fn new(client: Arc<BrkClientBase>, acc: String) -> Self {
        Self {
            btc: SeriesPattern1::new(client.clone(), acc.clone()),
            cents: SeriesPattern1::new(client.clone(), _m(&acc, "cents")),
            delta: AbsoluteRatePattern3::new(client.clone(), _m(&acc, "delta")),
            sats: SeriesPattern1::new(client.clone(), _m(&acc, "sats")),
            usd: SeriesPattern1::new(client.clone(), _m(&acc, "usd")),
        }
    }
}

/// Pattern struct for repeated tree structure.
pub struct BtcCentsSatsShareUsdPattern {
    pub btc: SeriesPattern1<Bitcoin>,
    pub cents: SeriesPattern1<Cents>,
    pub sats: SeriesPattern1<Sats>,
    pub share: BpsPercentRatioPattern2,
    pub usd: SeriesPattern1<Dollars>,
}

impl BtcCentsSatsShareUsdPattern {
    /// Create a new pattern node with accumulated series name.
    pub fn new(client: Arc<BrkClientBase>, acc: String) -> Self {
        Self {
            btc: SeriesPattern1::new(client.clone(), acc.clone()),
            cents: SeriesPattern1::new(client.clone(), _m(&acc, "cents")),
            sats: SeriesPattern1::new(client.clone(), _m(&acc, "sats")),
            share: BpsPercentRatioPattern2::new(client.clone(), _m(&acc, "share")),
            usd: SeriesPattern1::new(client.clone(), _m(&acc, "usd")),
        }
    }
}

/// Pattern struct for repeated tree structure.
pub struct CapLossMvrvPriceProfitPattern {
    pub cap: CentsDeltaUsdPattern,
    pub loss: BlockCumulativeSumPattern,
    pub mvrv: SeriesPattern1<StoredF32>,
    pub price: BpsCentsRatioSatsUsdPattern,
    pub profit: BlockCumulativeSumPattern,
}

impl CapLossMvrvPriceProfitPattern {
    /// Create a new pattern node with accumulated series name.
    pub fn new(client: Arc<BrkClientBase>, acc: String) -> Self {
        Self {
            cap: CentsDeltaUsdPattern::new(client.clone(), _m(&acc, "realized_cap")),
            loss: BlockCumulativeSumPattern::new(client.clone(), _m(&acc, "realized_loss")),
            mvrv: SeriesPattern1::new(client.clone(), _m(&acc, "mvrv")),
            price: BpsCentsRatioSatsUsdPattern::new(client.clone(), _m(&acc, "realized_price")),
            profit: BlockCumulativeSumPattern::new(client.clone(), _m(&acc, "realized_profit")),
        }
    }
}

/// Pattern struct for repeated tree structure.
pub struct CentsToUsdPattern4 {
    pub cents: SeriesPattern1<Cents>,
    pub to_mcap: BpsPercentRatioPattern2,
    pub to_own_gross_pnl: BpsPercentRatioPattern2,
    pub to_own_mcap: BpsPercentRatioPattern2,
    pub usd: SeriesPattern1<Dollars>,
}

impl CentsToUsdPattern4 {
    /// Create a new pattern node with accumulated series name.
    pub fn new(client: Arc<BrkClientBase>, acc: String) -> Self {
        Self {
            cents: SeriesPattern1::new(client.clone(), _m(&acc, "cents")),
            to_mcap: BpsPercentRatioPattern2::new(client.clone(), _m(&acc, "to_mcap")),
            to_own_gross_pnl: BpsPercentRatioPattern2::new(client.clone(), _m(&acc, "to_own_gross_pnl")),
            to_own_mcap: BpsPercentRatioPattern2::new(client.clone(), _m(&acc, "to_own_mcap")),
            usd: SeriesPattern1::new(client.clone(), acc.clone()),
        }
    }
}

/// Pattern struct for repeated tree structure.
pub struct EmaHistogramLineSignalPattern {
    pub ema_fast: SeriesPattern1<StoredF32>,
    pub ema_slow: SeriesPattern1<StoredF32>,
    pub histogram: SeriesPattern1<StoredF32>,
    pub line: SeriesPattern1<StoredF32>,
    pub signal: SeriesPattern1<StoredF32>,
}

/// Pattern struct for repeated tree structure.
pub struct PhsReboundThsPattern {
    pub phs: SeriesPattern1<StoredF32>,
    pub phs_min: SeriesPattern1<StoredF32>,
    pub rebound: BpsPercentRatioPattern,
    pub ths: SeriesPattern1<StoredF32>,
    pub ths_min: SeriesPattern1<StoredF32>,
}

impl PhsReboundThsPattern {
    /// Create a new pattern node with accumulated series name.
    pub fn new(client: Arc<BrkClientBase>, acc: String) -> Self {
        Self {
            phs: SeriesPattern1::new(client.clone(), _m(&acc, "phs")),
            phs_min: SeriesPattern1::new(client.clone(), _m(&acc, "phs_min")),
            rebound: BpsPercentRatioPattern::new(client.clone(), _m(&acc, "rebound")),
            ths: SeriesPattern1::new(client.clone(), _m(&acc, "ths")),
            ths_min: SeriesPattern1::new(client.clone(), _m(&acc, "ths_min")),
        }
    }
}

/// Pattern struct for repeated tree structure.
pub struct _1m1w1y24hPattern2 {
    pub _1m: BpsPercentRatioPattern,
    pub _1w: BpsPercentRatioPattern,
    pub _1y: BpsPercentRatioPattern,
    pub _24h: BpsPercentRatioPattern,
}

impl _1m1w1y24hPattern2 {
    /// Create a new pattern node with accumulated series name.
    pub fn new(client: Arc<BrkClientBase>, acc: String) -> Self {
        Self {
            _1m: BpsPercentRatioPattern::new(client.clone(), _m(&acc, "1m_rate")),
            _1w: BpsPercentRatioPattern::new(client.clone(), _m(&acc, "1w_rate")),
            _1y: BpsPercentRatioPattern::new(client.clone(), _m(&acc, "1y_rate")),
            _24h: BpsPercentRatioPattern::new(client.clone(), _m(&acc, "24h_rate")),
        }
    }
}

/// Pattern struct for repeated tree structure.
pub struct _1m1w1y24hPattern8 {
    pub _1m: BpsPercentRatioPattern4,
    pub _1w: BpsPercentRatioPattern4,
    pub _1y: BpsPercentRatioPattern4,
    pub _24h: BpsPercentRatioPattern4,
}

impl _1m1w1y24hPattern8 {
    /// Create a new pattern node with accumulated series name.
    pub fn new(client: Arc<BrkClientBase>, acc: String) -> Self {
        Self {
            _1m: BpsPercentRatioPattern4::new(client.clone(), _m(&acc, "1m")),
            _1w: BpsPercentRatioPattern4::new(client.clone(), _m(&acc, "1w")),
            _1y: BpsPercentRatioPattern4::new(client.clone(), _m(&acc, "1y")),
            _24h: BpsPercentRatioPattern4::new(client.clone(), _m(&acc, "24h")),
        }
    }
}

/// Pattern struct for repeated tree structure.
pub struct _1m1w1y24hPattern4 {
    pub _1m: BtcCentsSatsUsdPattern,
    pub _1w: BtcCentsSatsUsdPattern,
    pub _1y: BtcCentsSatsUsdPattern,
    pub _24h: BtcCentsSatsUsdPattern,
}

impl _1m1w1y24hPattern4 {
    /// Create a new pattern node with accumulated series name.
    pub fn new(client: Arc<BrkClientBase>, acc: String) -> Self {
        Self {
            _1m: BtcCentsSatsUsdPattern::new(client.clone(), _m(&acc, "1m")),
            _1w: BtcCentsSatsUsdPattern::new(client.clone(), _m(&acc, "1w")),
            _1y: BtcCentsSatsUsdPattern::new(client.clone(), _m(&acc, "1y")),
            _24h: BtcCentsSatsUsdPattern::new(client.clone(), _m(&acc, "24h")),
        }
    }
}

/// Pattern struct for repeated tree structure.
pub struct _1m1w1y24hPattern3 {
    pub _1m: BtcCentsSatsUsdPattern2,
    pub _1w: BtcCentsSatsUsdPattern2,
    pub _1y: BtcCentsSatsUsdPattern2,
    pub _24h: BtcCentsSatsUsdPattern2,
}

impl _1m1w1y24hPattern3 {
    /// Create a new pattern node with accumulated series name.
    pub fn new(client: Arc<BrkClientBase>, acc: String) -> Self {
        Self {
            _1m: BtcCentsSatsUsdPattern2::new(client.clone(), _m(&acc, "1m")),
            _1w: BtcCentsSatsUsdPattern2::new(client.clone(), _m(&acc, "1w")),
            _1y: BtcCentsSatsUsdPattern2::new(client.clone(), _m(&acc, "1y")),
            _24h: BtcCentsSatsUsdPattern2::new(client.clone(), _m(&acc, "24h")),
        }
    }
}

/// Pattern struct for repeated tree structure.
pub struct _1m1w1y24hPattern7 {
    pub _1m: BtcSatsPattern,
    pub _1w: BtcSatsPattern,
    pub _1y: BtcSatsPattern,
    pub _24h: BtcSatsPattern,
}

impl _1m1w1y24hPattern7 {
    /// Create a new pattern node with accumulated series name.
    pub fn new(client: Arc<BrkClientBase>, acc: String) -> Self {
        Self {
            _1m: BtcSatsPattern::new(client.clone(), _m(&acc, "1m")),
            _1w: BtcSatsPattern::new(client.clone(), _m(&acc, "1w")),
            _1y: BtcSatsPattern::new(client.clone(), _m(&acc, "1y")),
            _24h: BtcSatsPattern::new(client.clone(), _m(&acc, "24h")),
        }
    }
}

/// Pattern struct for repeated tree structure.
pub struct _1m1w1y2wPattern {
    pub _1m: CentsSatsUsdPattern,
    pub _1w: CentsSatsUsdPattern,
    pub _1y: CentsSatsUsdPattern,
    pub _2w: CentsSatsUsdPattern,
}

impl _1m1w1y2wPattern {
    /// Create a new pattern node with accumulated series name.
    pub fn new(client: Arc<BrkClientBase>, acc: String) -> Self {
        Self {
            _1m: CentsSatsUsdPattern::new(client.clone(), _m(&acc, "1m")),
            _1w: CentsSatsUsdPattern::new(client.clone(), _m(&acc, "1w")),
            _1y: CentsSatsUsdPattern::new(client.clone(), _m(&acc, "1y")),
            _2w: CentsSatsUsdPattern::new(client.clone(), _m(&acc, "2w")),
        }
    }
}

/// Pattern struct for repeated tree structure.
pub struct _1m1w1y24hPattern5 {
    pub _1m: CentsUsdPattern,
    pub _1w: CentsUsdPattern,
    pub _1y: CentsUsdPattern,
    pub _24h: CentsUsdPattern,
}

impl _1m1w1y24hPattern5 {
    /// Create a new pattern node with accumulated series name.
    pub fn new(client: Arc<BrkClientBase>, acc: String) -> Self {
        Self {
            _1m: CentsUsdPattern::new(client.clone(), _m(&acc, "1m")),
            _1w: CentsUsdPattern::new(client.clone(), _m(&acc, "1w")),
            _1y: CentsUsdPattern::new(client.clone(), _m(&acc, "1y")),
            _24h: CentsUsdPattern::new(client.clone(), _m(&acc, "24h")),
        }
    }
}

/// Pattern struct for repeated tree structure.
pub struct _1m1w1y24hPattern6 {
    pub _1m: CentsUsdPattern3,
    pub _1w: CentsUsdPattern3,
    pub _1y: CentsUsdPattern3,
    pub _24h: CentsUsdPattern3,
}

impl _1m1w1y24hPattern6 {
    /// Create a new pattern node with accumulated series name.
    pub fn new(client: Arc<BrkClientBase>, acc: String) -> Self {
        Self {
            _1m: CentsUsdPattern3::new(client.clone(), _m(&acc, "1m")),
            _1w: CentsUsdPattern3::new(client.clone(), _m(&acc, "1w")),
            _1y: CentsUsdPattern3::new(client.clone(), _m(&acc, "1y")),
            _24h: CentsUsdPattern3::new(client.clone(), _m(&acc, "24h")),
        }
    }
}

/// Pattern struct for repeated tree structure.
pub struct _1y2y4yAllPattern {
    pub _1y: _0sdM0M1M1sdM2M2sdM3sdP0P1P1sdP2P2sdP3sdSdZscorePattern,
    pub _2y: _0sdM0M1M1sdM2M2sdM3sdP0P1P1sdP2P2sdP3sdSdZscorePattern,
    pub _4y: _0sdM0M1M1sdM2M2sdM3sdP0P1P1sdP2P2sdP3sdSdZscorePattern,
    pub all: _0sdM0M1M1sdM2M2sdM3sdP0P1P1sdP2P2sdP3sdSdZscorePattern,
}

/// Pattern struct for repeated tree structure.
pub struct AverageBlockCumulativeSumPattern2 {
    pub average: _1m1w1y24hPattern<StoredF32>,
    pub block: SeriesPattern18<StoredU32>,
    pub cumulative: SeriesPattern1<StoredU64>,
    pub sum: _1m1w1y24hPattern<StoredU64>,
}

impl AverageBlockCumulativeSumPattern2 {
    /// Create a new pattern node with accumulated series name.
    pub fn new(client: Arc<BrkClientBase>, acc: String) -> Self {
        Self {
            average: _1m1w1y24hPattern::new(client.clone(), _m(&acc, "average")),
            block: SeriesPattern18::new(client.clone(), acc.clone()),
            cumulative: SeriesPattern1::new(client.clone(), _m(&acc, "cumulative")),
            sum: _1m1w1y24hPattern::new(client.clone(), _m(&acc, "sum")),
        }
    }
}

/// Pattern struct for repeated tree structure.
pub struct AverageBlockCumulativeSumPattern3 {
    pub average: _1m1w1y24hPattern3,
    pub block: BtcCentsSatsUsdPattern3,
    pub cumulative: BtcCentsSatsUsdPattern,
    pub sum: _1m1w1y24hPattern4,
}

impl AverageBlockCumulativeSumPattern3 {
    /// Create a new pattern node with accumulated series name.
    pub fn new(client: Arc<BrkClientBase>, acc: String) -> Self {
        Self {
            average: _1m1w1y24hPattern3::new(client.clone(), _m(&acc, "average")),
            block: BtcCentsSatsUsdPattern3::new(client.clone(), acc.clone()),
            cumulative: BtcCentsSatsUsdPattern::new(client.clone(), _m(&acc, "cumulative")),
            sum: _1m1w1y24hPattern4::new(client.clone(), _m(&acc, "sum")),
        }
    }
}

/// Pattern struct for repeated tree structure.
pub struct BlockCumulativeNegativeSumPattern {
    pub block: CentsUsdPattern2,
    pub cumulative: CentsUsdPattern3,
    pub negative: BaseSumPattern,
    pub sum: _1m1w1y24hPattern6,
}

impl BlockCumulativeNegativeSumPattern {
    /// Create a new pattern node with accumulated series name.
    pub fn new(client: Arc<BrkClientBase>, acc: String) -> Self {
        Self {
            block: CentsUsdPattern2::new(client.clone(), acc.clone()),
            cumulative: CentsUsdPattern3::new(client.clone(), _m(&acc, "cumulative")),
            negative: BaseSumPattern::new(client.clone(), _m(&acc, "neg")),
            sum: _1m1w1y24hPattern6::new(client.clone(), _m(&acc, "sum")),
        }
    }
}

/// Pattern struct for repeated tree structure.
pub struct BlockCumulativeDeltaSumPattern {
    pub block: CentsUsdPattern4,
    pub cumulative: CentsUsdPattern,
    pub delta: AbsoluteRatePattern2,
    pub sum: _1m1w1y24hPattern5,
}

impl BlockCumulativeDeltaSumPattern {
    /// Create a new pattern node with accumulated series name.
    pub fn new(client: Arc<BrkClientBase>, acc: String) -> Self {
        Self {
            block: CentsUsdPattern4::new(client.clone(), acc.clone()),
            cumulative: CentsUsdPattern::new(client.clone(), _m(&acc, "cumulative")),
            delta: AbsoluteRatePattern2::new(client.clone(), _m(&acc, "delta")),
            sum: _1m1w1y24hPattern5::new(client.clone(), _m(&acc, "sum")),
        }
    }
}

/// Pattern struct for repeated tree structure.
pub struct BtcCentsSatsUsdPattern {
    pub btc: SeriesPattern1<Bitcoin>,
    pub cents: SeriesPattern1<Cents>,
    pub sats: SeriesPattern1<Sats>,
    pub usd: SeriesPattern1<Dollars>,
}

impl BtcCentsSatsUsdPattern {
    /// Create a new pattern node with accumulated series name.
    pub fn new(client: Arc<BrkClientBase>, acc: String) -> Self {
        Self {
            btc: SeriesPattern1::new(client.clone(), acc.clone()),
            cents: SeriesPattern1::new(client.clone(), _m(&acc, "cents")),
            sats: SeriesPattern1::new(client.clone(), _m(&acc, "sats")),
            usd: SeriesPattern1::new(client.clone(), _m(&acc, "usd")),
        }
    }
}

/// Pattern struct for repeated tree structure.
pub struct BtcCentsSatsUsdPattern2 {
    pub btc: SeriesPattern1<Bitcoin>,
    pub cents: SeriesPattern1<StoredF32>,
    pub sats: SeriesPattern1<StoredF32>,
    pub usd: SeriesPattern1<Dollars>,
}

impl BtcCentsSatsUsdPattern2 {
    /// Create a new pattern node with accumulated series name.
    pub fn new(client: Arc<BrkClientBase>, acc: String) -> Self {
        Self {
            btc: SeriesPattern1::new(client.clone(), acc.clone()),
            cents: SeriesPattern1::new(client.clone(), _m(&acc, "cents")),
            sats: SeriesPattern1::new(client.clone(), _m(&acc, "sats")),
            usd: SeriesPattern1::new(client.clone(), _m(&acc, "usd")),
        }
    }
}

/// Pattern struct for repeated tree structure.
pub struct BtcCentsSatsUsdPattern3 {
    pub btc: SeriesPattern18<Bitcoin>,
    pub cents: SeriesPattern18<Cents>,
    pub sats: SeriesPattern18<Sats>,
    pub usd: SeriesPattern18<Dollars>,
}

impl BtcCentsSatsUsdPattern3 {
    /// Create a new pattern node with accumulated series name.
    pub fn new(client: Arc<BrkClientBase>, acc: String) -> Self {
        Self {
            btc: SeriesPattern18::new(client.clone(), acc.clone()),
            cents: SeriesPattern18::new(client.clone(), _m(&acc, "cents")),
            sats: SeriesPattern18::new(client.clone(), _m(&acc, "sats")),
            usd: SeriesPattern18::new(client.clone(), _m(&acc, "usd")),
        }
    }
}

/// Pattern struct for repeated tree structure.
pub struct CentsDeltaToUsdPattern {
    pub cents: SeriesPattern1<Cents>,
    pub delta: AbsoluteRatePattern2,
    pub to_own_mcap: BpsPercentRatioPattern4,
    pub usd: SeriesPattern1<Dollars>,
}

impl CentsDeltaToUsdPattern {
    /// Create a new pattern node with accumulated series name.
    pub fn new(client: Arc<BrkClientBase>, acc: String) -> Self {
        Self {
            cents: SeriesPattern1::new(client.clone(), _m(&acc, "cents")),
            delta: AbsoluteRatePattern2::new(client.clone(), _m(&acc, "delta")),
            to_own_mcap: BpsPercentRatioPattern4::new(client.clone(), _m(&acc, "to_own_mcap")),
            usd: SeriesPattern1::new(client.clone(), acc.clone()),
        }
    }
}

/// Pattern struct for repeated tree structure.
pub struct CentsToUsdPattern3 {
    pub cents: SeriesPattern1<CentsSigned>,
    pub to_own_gross_pnl: BpsPercentRatioPattern,
    pub to_own_mcap: BpsPercentRatioPattern,
    pub usd: SeriesPattern1<Dollars>,
}

impl CentsToUsdPattern3 {
    /// Create a new pattern node with accumulated series name.
    pub fn new(client: Arc<BrkClientBase>, acc: String) -> Self {
        Self {
            cents: SeriesPattern1::new(client.clone(), _m(&acc, "cents")),
            to_own_gross_pnl: BpsPercentRatioPattern::new(client.clone(), _m(&acc, "to_own_gross_pnl")),
            to_own_mcap: BpsPercentRatioPattern::new(client.clone(), _m(&acc, "to_own_mcap")),
            usd: SeriesPattern1::new(client.clone(), acc.clone()),
        }
    }
}

/// Pattern struct for repeated tree structure.
pub struct CoindaysCoinyearsDormancyTransferPattern {
    pub coindays_destroyed: AverageBlockCumulativeSumPattern<StoredF64>,
    pub coinyears_destroyed: SeriesPattern1<StoredF64>,
    pub dormancy: _1m1w1y24hPattern<StoredF32>,
    pub transfer_volume: AverageBlockCumulativeInSumPattern,
}

impl CoindaysCoinyearsDormancyTransferPattern {
    /// Create a new pattern node with accumulated series name.
    pub fn new(client: Arc<BrkClientBase>, acc: String) -> Self {
        Self {
            coindays_destroyed: AverageBlockCumulativeSumPattern::new(client.clone(), _m(&acc, "coindays_destroyed")),
            coinyears_destroyed: SeriesPattern1::new(client.clone(), _m(&acc, "coinyears_destroyed")),
            dormancy: _1m1w1y24hPattern::new(client.clone(), _m(&acc, "dormancy")),
            transfer_volume: AverageBlockCumulativeInSumPattern::new(client.clone(), _m(&acc, "transfer_volume")),
        }
    }
}

/// Pattern struct for repeated tree structure.
pub struct LossNetNuplProfitPattern {
    pub loss: CentsNegativeUsdPattern,
    pub net_pnl: CentsUsdPattern,
    pub nupl: BpsRatioPattern,
    pub profit: CentsUsdPattern3,
}

impl LossNetNuplProfitPattern {
    /// Create a new pattern node with accumulated series name.
    pub fn new(client: Arc<BrkClientBase>, acc: String) -> Self {
        Self {
            loss: CentsNegativeUsdPattern::new(client.clone(), _m(&acc, "unrealized_loss")),
            net_pnl: CentsUsdPattern::new(client.clone(), _m(&acc, "net_unrealized_pnl")),
            nupl: BpsRatioPattern::new(client.clone(), _m(&acc, "nupl")),
            profit: CentsUsdPattern3::new(client.clone(), _m(&acc, "unrealized_profit")),
        }
    }
}

/// Pattern struct for repeated tree structure.
pub struct NuplRealizedSupplyUnrealizedPattern {
    pub nupl: BpsRatioPattern,
    pub realized_cap: AllSthPattern,
    pub supply: AllSthPattern2,
    pub unrealized_pnl: AllSthPattern,
}

impl NuplRealizedSupplyUnrealizedPattern {
    /// Create a new pattern node with accumulated series name.
    pub fn new(client: Arc<BrkClientBase>, acc: String) -> Self {
        Self {
            nupl: BpsRatioPattern::new(client.clone(), _m(&acc, "nupl")),
            realized_cap: AllSthPattern::new(client.clone(), acc.clone(), "realized_cap".to_string()),
            supply: AllSthPattern2::new(client.clone(), acc.clone()),
            unrealized_pnl: AllSthPattern::new(client.clone(), acc.clone(), "unrealized_pnl".to_string()),
        }
    }
}

/// Pattern struct for repeated tree structure.
pub struct _1m1w1y24hPattern<T> {
    pub _1m: SeriesPattern1<T>,
    pub _1w: SeriesPattern1<T>,
    pub _1y: SeriesPattern1<T>,
    pub _24h: SeriesPattern1<T>,
}

impl<T: DeserializeOwned> _1m1w1y24hPattern<T> {
    /// Create a new pattern node with accumulated series name.
    pub fn new(client: Arc<BrkClientBase>, acc: String) -> Self {
        Self {
            _1m: SeriesPattern1::new(client.clone(), _m(&acc, "1m")),
            _1w: SeriesPattern1::new(client.clone(), _m(&acc, "1w")),
            _1y: SeriesPattern1::new(client.clone(), _m(&acc, "1y")),
            _24h: SeriesPattern1::new(client.clone(), _m(&acc, "24h")),
        }
    }
}

/// Pattern struct for repeated tree structure.
pub struct AverageBlockCumulativeSumPattern<T> {
    pub average: _1m1w1y24hPattern<T>,
    pub block: SeriesPattern18<T>,
    pub cumulative: SeriesPattern1<T>,
    pub sum: _1m1w1y24hPattern<T>,
}

impl<T: DeserializeOwned> AverageBlockCumulativeSumPattern<T> {
    /// Create a new pattern node with accumulated series name.
    pub fn new(client: Arc<BrkClientBase>, acc: String) -> Self {
        Self {
            average: _1m1w1y24hPattern::new(client.clone(), _m(&acc, "average")),
            block: SeriesPattern18::new(client.clone(), acc.clone()),
            cumulative: SeriesPattern1::new(client.clone(), _m(&acc, "cumulative")),
            sum: _1m1w1y24hPattern::new(client.clone(), _m(&acc, "sum")),
        }
    }
}

/// Pattern struct for repeated tree structure.
pub struct AdjustedRatioValuePattern {
    pub adjusted: RatioTransferValuePattern,
    pub ratio: _1m1w1y24hPattern<StoredF64>,
    pub value_destroyed: AverageBlockCumulativeSumPattern<Cents>,
}

impl AdjustedRatioValuePattern {
    /// Create a new pattern node with accumulated series name.
    pub fn new(client: Arc<BrkClientBase>, acc: String) -> Self {
        Self {
            adjusted: RatioTransferValuePattern::new(client.clone(), acc.clone()),
            ratio: _1m1w1y24hPattern::new(client.clone(), _m(&acc, "sopr")),
            value_destroyed: AverageBlockCumulativeSumPattern::new(client.clone(), _m(&acc, "value_destroyed")),
        }
    }
}

/// Pattern struct for repeated tree structure.
pub struct BlockCumulativeSumPattern {
    pub block: CentsUsdPattern2,
    pub cumulative: CentsUsdPattern3,
    pub sum: _1m1w1y24hPattern6,
}

impl BlockCumulativeSumPattern {
    /// Create a new pattern node with accumulated series name.
    pub fn new(client: Arc<BrkClientBase>, acc: String) -> Self {
        Self {
            block: CentsUsdPattern2::new(client.clone(), acc.clone()),
            cumulative: CentsUsdPattern3::new(client.clone(), _m(&acc, "cumulative")),
            sum: _1m1w1y24hPattern6::new(client.clone(), _m(&acc, "sum")),
        }
    }
}

/// Pattern struct for repeated tree structure.
pub struct BlocksDominanceRewardsPattern {
    pub blocks_mined: AverageBlockCumulativeSumPattern2,
    pub dominance: _1m1w1y24hBpsPercentRatioPattern,
    pub rewards: AverageBlockCumulativeSumPattern3,
}

impl BlocksDominanceRewardsPattern {
    /// Create a new pattern node with accumulated series name.
    pub fn new(client: Arc<BrkClientBase>, acc: String) -> Self {
        Self {
            blocks_mined: AverageBlockCumulativeSumPattern2::new(client.clone(), _m(&acc, "blocks_mined")),
            dominance: _1m1w1y24hBpsPercentRatioPattern::new(client.clone(), _m(&acc, "dominance")),
            rewards: AverageBlockCumulativeSumPattern3::new(client.clone(), _m(&acc, "rewards")),
        }
    }
}

/// Pattern struct for repeated tree structure.
pub struct BpsPercentRatioPattern2 {
    pub bps: SeriesPattern1<BasisPoints16>,
    pub percent: SeriesPattern1<StoredF32>,
    pub ratio: SeriesPattern1<StoredF32>,
}

impl BpsPercentRatioPattern2 {
    /// Create a new pattern node with accumulated series name.
    pub fn new(client: Arc<BrkClientBase>, acc: String) -> Self {
        Self {
            bps: SeriesPattern1::new(client.clone(), _m(&acc, "bps")),
            percent: SeriesPattern1::new(client.clone(), acc.clone()),
            ratio: SeriesPattern1::new(client.clone(), _m(&acc, "ratio")),
        }
    }
}

/// Pattern struct for repeated tree structure.
pub struct BpsPercentRatioPattern4 {
    pub bps: SeriesPattern1<BasisPoints32>,
    pub percent: SeriesPattern1<StoredF32>,
    pub ratio: SeriesPattern1<StoredF32>,
}

impl BpsPercentRatioPattern4 {
    /// Create a new pattern node with accumulated series name.
    pub fn new(client: Arc<BrkClientBase>, acc: String) -> Self {
        Self {
            bps: SeriesPattern1::new(client.clone(), _m(&acc, "bps")),
            percent: SeriesPattern1::new(client.clone(), acc.clone()),
            ratio: SeriesPattern1::new(client.clone(), _m(&acc, "ratio")),
        }
    }
}

/// Pattern struct for repeated tree structure.
pub struct BpsPriceRatioPattern {
    pub bps: SeriesPattern1<BasisPoints32>,
    pub price: CentsSatsUsdPattern,
    pub ratio: SeriesPattern1<StoredF32>,
}

impl BpsPriceRatioPattern {
    /// Create a new pattern node with accumulated series name.
    pub fn new(client: Arc<BrkClientBase>, acc: String, disc: String) -> Self {
        Self {
            bps: SeriesPattern1::new(client.clone(), _m(&acc, &format!("ratio_{disc}_bps", disc=disc))),
            price: CentsSatsUsdPattern::new(client.clone(), _m(&acc, &disc)),
            ratio: SeriesPattern1::new(client.clone(), _m(&acc, &format!("ratio_{disc}", disc=disc))),
        }
    }
}

/// Pattern struct for repeated tree structure.
pub struct BpsPercentRatioPattern5 {
    pub bps: SeriesPattern1<BasisPointsSigned16>,
    pub percent: SeriesPattern1<StoredF32>,
    pub ratio: SeriesPattern1<StoredF32>,
}

impl BpsPercentRatioPattern5 {
    /// Create a new pattern node with accumulated series name.
    pub fn new(client: Arc<BrkClientBase>, acc: String) -> Self {
        Self {
            bps: SeriesPattern1::new(client.clone(), _m(&acc, "bps")),
            percent: SeriesPattern1::new(client.clone(), acc.clone()),
            ratio: SeriesPattern1::new(client.clone(), _m(&acc, "ratio")),
        }
    }
}

/// Pattern struct for repeated tree structure.
pub struct BpsPercentRatioPattern {
    pub bps: SeriesPattern1<BasisPointsSigned32>,
    pub percent: SeriesPattern1<StoredF32>,
    pub ratio: SeriesPattern1<StoredF32>,
}

impl BpsPercentRatioPattern {
    /// Create a new pattern node with accumulated series name.
    pub fn new(client: Arc<BrkClientBase>, acc: String) -> Self {
        Self {
            bps: SeriesPattern1::new(client.clone(), _m(&acc, "bps")),
            percent: SeriesPattern1::new(client.clone(), acc.clone()),
            ratio: SeriesPattern1::new(client.clone(), _m(&acc, "ratio")),
        }
    }
}

/// Pattern struct for repeated tree structure.
pub struct CentsSatsUsdPattern3 {
    pub cents: SeriesPattern2<Cents>,
    pub sats: SeriesPattern2<Sats>,
    pub usd: SeriesPattern2<Dollars>,
}

impl CentsSatsUsdPattern3 {
    /// Create a new pattern node with accumulated series name.
    pub fn new(client: Arc<BrkClientBase>, acc: String) -> Self {
        Self {
            cents: SeriesPattern2::new(client.clone(), _m(&acc, "cents")),
            sats: SeriesPattern2::new(client.clone(), _m(&acc, "sats")),
            usd: SeriesPattern2::new(client.clone(), acc.clone()),
        }
    }
}

/// Pattern struct for repeated tree structure.
pub struct CentsDeltaUsdPattern {
    pub cents: SeriesPattern1<Cents>,
    pub delta: AbsoluteRatePattern2,
    pub usd: SeriesPattern1<Dollars>,
}

impl CentsDeltaUsdPattern {
    /// Create a new pattern node with accumulated series name.
    pub fn new(client: Arc<BrkClientBase>, acc: String) -> Self {
        Self {
            cents: SeriesPattern1::new(client.clone(), _m(&acc, "cents")),
            delta: AbsoluteRatePattern2::new(client.clone(), _m(&acc, "delta")),
            usd: SeriesPattern1::new(client.clone(), acc.clone()),
        }
    }
}

/// Pattern struct for repeated tree structure.
pub struct CentsNegativeUsdPattern {
    pub cents: SeriesPattern1<Cents>,
    pub negative: SeriesPattern1<Dollars>,
    pub usd: SeriesPattern1<Dollars>,
}

impl CentsNegativeUsdPattern {
    /// Create a new pattern node with accumulated series name.
    pub fn new(client: Arc<BrkClientBase>, acc: String) -> Self {
        Self {
            cents: SeriesPattern1::new(client.clone(), _m(&acc, "cents")),
            negative: SeriesPattern1::new(client.clone(), _m(&acc, "neg")),
            usd: SeriesPattern1::new(client.clone(), acc.clone()),
        }
    }
}

/// Pattern struct for repeated tree structure.
pub struct CentsSatsUsdPattern {
    pub cents: SeriesPattern1<Cents>,
    pub sats: SeriesPattern1<SatsFract>,
    pub usd: SeriesPattern1<Dollars>,
}

impl CentsSatsUsdPattern {
    /// Create a new pattern node with accumulated series name.
    pub fn new(client: Arc<BrkClientBase>, acc: String) -> Self {
        Self {
            cents: SeriesPattern1::new(client.clone(), _m(&acc, "cents")),
            sats: SeriesPattern1::new(client.clone(), _m(&acc, "sats")),
            usd: SeriesPattern1::new(client.clone(), acc.clone()),
        }
    }
}

/// Pattern struct for repeated tree structure.
pub struct CountEventsSupplyPattern {
    pub count: FundedTotalPattern,
    pub events: ActiveInputOutputSpendablePattern,
    pub supply: AllP2aP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshSharePattern,
}

/// Pattern struct for repeated tree structure.
pub struct CumulativeRollingSumPattern {
    pub cumulative: SeriesPattern1<StoredU64>,
    pub rolling: AverageMaxMedianMinPct10Pct25Pct75Pct90SumPattern,
    pub sum: SeriesPattern18<StoredU64>,
}

impl CumulativeRollingSumPattern {
    /// Create a new pattern node with accumulated series name.
    pub fn new(client: Arc<BrkClientBase>, acc: String) -> Self {
        Self {
            cumulative: SeriesPattern1::new(client.clone(), _m(&acc, "cumulative")),
            rolling: AverageMaxMedianMinPct10Pct25Pct75Pct90SumPattern::new(client.clone(), acc.clone()),
            sum: SeriesPattern18::new(client.clone(), _m(&acc, "sum")),
        }
    }
}

/// Pattern struct for repeated tree structure.
pub struct DeltaDominanceTotalPattern {
    pub delta: AbsoluteRatePattern3,
    pub dominance: BpsPercentRatioPattern2,
    pub total: BtcCentsSatsUsdPattern,
}

impl DeltaDominanceTotalPattern {
    /// Create a new pattern node with accumulated series name.
    pub fn new(client: Arc<BrkClientBase>, acc: String) -> Self {
        Self {
            delta: AbsoluteRatePattern3::new(client.clone(), _m(&acc, "delta")),
            dominance: BpsPercentRatioPattern2::new(client.clone(), _m(&acc, "dominance")),
            total: BtcCentsSatsUsdPattern::new(client.clone(), acc.clone()),
        }
    }
}

/// Pattern struct for repeated tree structure.
pub struct GreedNetPainPattern {
    pub greed_index: CentsUsdPattern3,
    pub net: CentsUsdPattern,
    pub pain_index: CentsUsdPattern3,
}

impl GreedNetPainPattern {
    /// Create a new pattern node with accumulated series name.
    pub fn new(client: Arc<BrkClientBase>, acc: String) -> Self {
        Self {
            greed_index: CentsUsdPattern3::new(client.clone(), _m(&acc, "greed_index")),
            net: CentsUsdPattern::new(client.clone(), _m(&acc, "net_sentiment")),
            pain_index: CentsUsdPattern3::new(client.clone(), _m(&acc, "pain_index")),
        }
    }
}

/// Pattern struct for repeated tree structure.
pub struct LossNuplProfitPattern {
    pub loss: CentsNegativeUsdPattern,
    pub nupl: BpsRatioPattern,
    pub profit: CentsUsdPattern3,
}

impl LossNuplProfitPattern {
    /// Create a new pattern node with accumulated series name.
    pub fn new(client: Arc<BrkClientBase>, acc: String) -> Self {
        Self {
            loss: CentsNegativeUsdPattern::new(client.clone(), _m(&acc, "unrealized_loss")),
            nupl: BpsRatioPattern::new(client.clone(), _m(&acc, "nupl")),
            profit: CentsUsdPattern3::new(client.clone(), _m(&acc, "unrealized_profit")),
        }
    }
}

/// Pattern struct for repeated tree structure.
pub struct RatioTransferValuePattern {
    pub ratio: _1m1w1y24hPattern<StoredF64>,
    pub transfer_volume: AverageBlockCumulativeSumPattern<Cents>,
    pub value_destroyed: AverageBlockCumulativeSumPattern<Cents>,
}

impl RatioTransferValuePattern {
    /// Create a new pattern node with accumulated series name.
    pub fn new(client: Arc<BrkClientBase>, acc: String) -> Self {
        Self {
            ratio: _1m1w1y24hPattern::new(client.clone(), _m(&acc, "asopr")),
            transfer_volume: AverageBlockCumulativeSumPattern::new(client.clone(), _m(&acc, "adj_value_created")),
            value_destroyed: AverageBlockCumulativeSumPattern::new(client.clone(), _m(&acc, "adj_value_destroyed")),
        }
    }
}

/// Pattern struct for repeated tree structure.
pub struct RsiStochPattern {
    pub rsi: BpsPercentRatioPattern2,
    pub stoch_rsi_d: BpsPercentRatioPattern2,
    pub stoch_rsi_k: BpsPercentRatioPattern2,
}

impl RsiStochPattern {
    /// Create a new pattern node with accumulated series name.
    pub fn new(client: Arc<BrkClientBase>, acc: String, disc: String) -> Self {
        Self {
            rsi: BpsPercentRatioPattern2::new(client.clone(), _m(&acc, &disc)),
            stoch_rsi_d: BpsPercentRatioPattern2::new(client.clone(), _m(&acc, &format!("stoch_d_{disc}", disc=disc))),
            stoch_rsi_k: BpsPercentRatioPattern2::new(client.clone(), _m(&acc, &format!("stoch_k_{disc}", disc=disc))),
        }
    }
}

/// Pattern struct for repeated tree structure.
pub struct SpendingSpentUnspentPattern {
    pub spending_rate: SeriesPattern1<StoredF32>,
    pub spent_count: AverageBlockCumulativeSumPattern2,
    pub unspent_count: BaseDeltaPattern,
}

impl SpendingSpentUnspentPattern {
    /// Create a new pattern node with accumulated series name.
    pub fn new(client: Arc<BrkClientBase>, acc: String) -> Self {
        Self {
            spending_rate: SeriesPattern1::new(client.clone(), _m(&acc, "spending_rate")),
            spent_count: AverageBlockCumulativeSumPattern2::new(client.clone(), _m(&acc, "spent_utxo_count")),
            unspent_count: BaseDeltaPattern::new(client.clone(), _m(&acc, "utxo_count")),
        }
    }
}

/// Pattern struct for repeated tree structure.
pub struct _6bBlockTxPattern<T> {
    pub _6b: MaxMedianMinPct10Pct25Pct75Pct90Pattern<T>,
    pub block: MaxMedianMinPct10Pct25Pct75Pct90Pattern<T>,
    pub tx_index: SeriesPattern19<T>,
}

impl<T: DeserializeOwned> _6bBlockTxPattern<T> {
    /// Create a new pattern node with accumulated series name.
    pub fn new(client: Arc<BrkClientBase>, acc: String) -> Self {
        Self {
            _6b: MaxMedianMinPct10Pct25Pct75Pct90Pattern::new(client.clone(), _m(&acc, "6b")),
            block: MaxMedianMinPct10Pct25Pct75Pct90Pattern::new(client.clone(), acc.clone()),
            tx_index: SeriesPattern19::new(client.clone(), acc.clone()),
        }
    }
}

/// Pattern struct for repeated tree structure.
pub struct AbsoluteRatePattern {
    pub absolute: _1m1w1y24hPattern<StoredI64>,
    pub rate: _1m1w1y24hPattern2,
}

impl AbsoluteRatePattern {
    /// Create a new pattern node with accumulated series name.
    pub fn new(client: Arc<BrkClientBase>, acc: String) -> Self {
        Self {
            absolute: _1m1w1y24hPattern::new(client.clone(), acc.clone()),
            rate: _1m1w1y24hPattern2::new(client.clone(), acc.clone()),
        }
    }
}

/// Pattern struct for repeated tree structure.
pub struct AbsoluteRatePattern2 {
    pub absolute: _1m1w1y24hPattern5,
    pub rate: _1m1w1y24hPattern2,
}

impl AbsoluteRatePattern2 {
    /// Create a new pattern node with accumulated series name.
    pub fn new(client: Arc<BrkClientBase>, acc: String) -> Self {
        Self {
            absolute: _1m1w1y24hPattern5::new(client.clone(), acc.clone()),
            rate: _1m1w1y24hPattern2::new(client.clone(), acc.clone()),
        }
    }
}

/// Pattern struct for repeated tree structure.
pub struct AbsoluteRatePattern3 {
    pub absolute: _1m1w1y24hPattern7,
    pub rate: _1m1w1y24hPattern2,
}

impl AbsoluteRatePattern3 {
    /// Create a new pattern node with accumulated series name.
    pub fn new(client: Arc<BrkClientBase>, acc: String) -> Self {
        Self {
            absolute: _1m1w1y24hPattern7::new(client.clone(), acc.clone()),
            rate: _1m1w1y24hPattern2::new(client.clone(), acc.clone()),
        }
    }
}

/// Pattern struct for repeated tree structure.
pub struct AddrUtxoPattern {
    pub addr: BtcCentsSatsUsdPattern,
    pub utxo: BtcCentsSatsUsdPattern,
}

impl AddrUtxoPattern {
    /// Create a new pattern node with accumulated series name.
    pub fn new(client: Arc<BrkClientBase>, acc: String) -> Self {
        Self {
            addr: BtcCentsSatsUsdPattern::new(client.clone(), _m(&acc, "addr_amount")),
            utxo: BtcCentsSatsUsdPattern::new(client.clone(), _m(&acc, "utxo_amount")),
        }
    }
}

/// Pattern struct for repeated tree structure.
pub struct AllSthPattern2 {
    pub all: BtcCentsDeltaSatsUsdPattern,
    pub sth: BtcCentsSatsUsdPattern,
}

impl AllSthPattern2 {
    /// Create a new pattern node with accumulated series name.
    pub fn new(client: Arc<BrkClientBase>, acc: String) -> Self {
        Self {
            all: BtcCentsDeltaSatsUsdPattern::new(client.clone(), _m(&acc, "supply")),
            sth: BtcCentsSatsUsdPattern::new(client.clone(), _m(&acc, "sth_supply")),
        }
    }
}

/// Pattern struct for repeated tree structure.
pub struct AllSthPattern {
    pub all: SeriesPattern1<Dollars>,
    pub sth: SeriesPattern1<Dollars>,
}

impl AllSthPattern {
    /// Create a new pattern node with accumulated series name.
    pub fn new(client: Arc<BrkClientBase>, acc: String, disc: String) -> Self {
        Self {
            all: SeriesPattern1::new(client.clone(), _m(&acc, &disc)),
            sth: SeriesPattern1::new(client.clone(), _m(&acc, &format!("sth_{disc}", disc=disc))),
        }
    }
}

/// Pattern struct for repeated tree structure.
pub struct BaseSumPattern {
    pub base: SeriesPattern18<Dollars>,
    pub sum: _1m1w1y24hPattern<Dollars>,
}

impl BaseSumPattern {
    /// Create a new pattern node with accumulated series name.
    pub fn new(client: Arc<BrkClientBase>, acc: String) -> Self {
        Self {
            base: SeriesPattern18::new(client.clone(), acc.clone()),
            sum: _1m1w1y24hPattern::new(client.clone(), _m(&acc, "sum")),
        }
    }
}

/// Pattern struct for repeated tree structure.
pub struct BaseDeltaPattern {
    pub base: SeriesPattern1<StoredU64>,
    pub delta: AbsoluteRatePattern,
}

impl BaseDeltaPattern {
    /// Create a new pattern node with accumulated series name.
    pub fn new(client: Arc<BrkClientBase>, acc: String) -> Self {
        Self {
            base: SeriesPattern1::new(client.clone(), acc.clone()),
            delta: AbsoluteRatePattern::new(client.clone(), _m(&acc, "delta")),
        }
    }
}

/// Pattern struct for repeated tree structure.
pub struct BlockCumulativePattern {
    pub block: BtcCentsSatsUsdPattern3,
    pub cumulative: BtcCentsSatsUsdPattern,
}

impl BlockCumulativePattern {
    /// Create a new pattern node with accumulated series name.
    pub fn new(client: Arc<BrkClientBase>, acc: String) -> Self {
        Self {
            block: BtcCentsSatsUsdPattern3::new(client.clone(), acc.clone()),
            cumulative: BtcCentsSatsUsdPattern::new(client.clone(), _m(&acc, "cumulative")),
        }
    }
}

/// Pattern struct for repeated tree structure.
pub struct BlocksDominancePattern {
    pub blocks_mined: AverageBlockCumulativeSumPattern2,
    pub dominance: BpsPercentRatioPattern2,
}

impl BlocksDominancePattern {
    /// Create a new pattern node with accumulated series name.
    pub fn new(client: Arc<BrkClientBase>, acc: String) -> Self {
        Self {
            blocks_mined: AverageBlockCumulativeSumPattern2::new(client.clone(), _m(&acc, "blocks_mined")),
            dominance: BpsPercentRatioPattern2::new(client.clone(), _m(&acc, "dominance")),
        }
    }
}

/// Pattern struct for repeated tree structure.
pub struct BpsRatioPattern2 {
    pub bps: SeriesPattern1<BasisPoints32>,
    pub ratio: SeriesPattern1<StoredF32>,
}

impl BpsRatioPattern2 {
    /// Create a new pattern node with accumulated series name.
    pub fn new(client: Arc<BrkClientBase>, acc: String) -> Self {
        Self {
            bps: SeriesPattern1::new(client.clone(), _m(&acc, "bps")),
            ratio: SeriesPattern1::new(client.clone(), acc.clone()),
        }
    }
}

/// Pattern struct for repeated tree structure.
pub struct BpsRatioPattern {
    pub bps: SeriesPattern1<BasisPointsSigned32>,
    pub ratio: SeriesPattern1<StoredF32>,
}

impl BpsRatioPattern {
    /// Create a new pattern node with accumulated series name.
    pub fn new(client: Arc<BrkClientBase>, acc: String) -> Self {
        Self {
            bps: SeriesPattern1::new(client.clone(), _m(&acc, "bps")),
            ratio: SeriesPattern1::new(client.clone(), acc.clone()),
        }
    }
}

/// Pattern struct for repeated tree structure.
pub struct BtcSatsPattern {
    pub btc: SeriesPattern1<Bitcoin>,
    pub sats: SeriesPattern1<SatsSigned>,
}

impl BtcSatsPattern {
    /// Create a new pattern node with accumulated series name.
    pub fn new(client: Arc<BrkClientBase>, acc: String) -> Self {
        Self {
            btc: SeriesPattern1::new(client.clone(), acc.clone()),
            sats: SeriesPattern1::new(client.clone(), _m(&acc, "sats")),
        }
    }
}

/// Pattern struct for repeated tree structure.
pub struct CentsUsdPattern3 {
    pub cents: SeriesPattern1<Cents>,
    pub usd: SeriesPattern1<Dollars>,
}

impl CentsUsdPattern3 {
    /// Create a new pattern node with accumulated series name.
    pub fn new(client: Arc<BrkClientBase>, acc: String) -> Self {
        Self {
            cents: SeriesPattern1::new(client.clone(), _m(&acc, "cents")),
            usd: SeriesPattern1::new(client.clone(), acc.clone()),
        }
    }
}

/// Pattern struct for repeated tree structure.
pub struct CentsUsdPattern2 {
    pub cents: SeriesPattern18<Cents>,
    pub usd: SeriesPattern18<Dollars>,
}

impl CentsUsdPattern2 {
    /// Create a new pattern node with accumulated series name.
    pub fn new(client: Arc<BrkClientBase>, acc: String) -> Self {
        Self {
            cents: SeriesPattern18::new(client.clone(), _m(&acc, "cents")),
            usd: SeriesPattern18::new(client.clone(), acc.clone()),
        }
    }
}

/// Pattern struct for repeated tree structure.
pub struct CentsUsdPattern {
    pub cents: SeriesPattern1<CentsSigned>,
    pub usd: SeriesPattern1<Dollars>,
}

impl CentsUsdPattern {
    /// Create a new pattern node with accumulated series name.
    pub fn new(client: Arc<BrkClientBase>, acc: String) -> Self {
        Self {
            cents: SeriesPattern1::new(client.clone(), _m(&acc, "cents")),
            usd: SeriesPattern1::new(client.clone(), acc.clone()),
        }
    }
}

/// Pattern struct for repeated tree structure.
pub struct CentsUsdPattern4 {
    pub cents: SeriesPattern18<CentsSigned>,
    pub usd: SeriesPattern18<Dollars>,
}

impl CentsUsdPattern4 {
    /// Create a new pattern node with accumulated series name.
    pub fn new(client: Arc<BrkClientBase>, acc: String) -> Self {
        Self {
            cents: SeriesPattern18::new(client.clone(), _m(&acc, "cents")),
            usd: SeriesPattern18::new(client.clone(), acc.clone()),
        }
    }
}

/// Pattern struct for repeated tree structure.
pub struct CoindaysTransferPattern {
    pub coindays_destroyed: AverageBlockCumulativeSumPattern<StoredF64>,
    pub transfer_volume: AverageBlockCumulativeInSumPattern,
}

impl CoindaysTransferPattern {
    /// Create a new pattern node with accumulated series name.
    pub fn new(client: Arc<BrkClientBase>, acc: String) -> Self {
        Self {
            coindays_destroyed: AverageBlockCumulativeSumPattern::new(client.clone(), _m(&acc, "coindays_destroyed")),
            transfer_volume: AverageBlockCumulativeInSumPattern::new(client.clone(), _m(&acc, "transfer_volume")),
        }
    }
}

/// Pattern struct for repeated tree structure.
pub struct FundedTotalPattern {
    pub funded: AllP2aP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshPattern4,
    pub total: AllP2aP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshPattern4,
}

impl FundedTotalPattern {
    /// Create a new pattern node with accumulated series name.
    pub fn new(client: Arc<BrkClientBase>, acc: String) -> Self {
        Self {
            funded: AllP2aP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshPattern4::new(client.clone(), acc.clone()),
            total: AllP2aP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshPattern4::new(client.clone(), _p("total", &acc)),
        }
    }
}

/// Pattern struct for repeated tree structure.
pub struct InPattern2 {
    pub in_loss: CentsUsdPattern3,
    pub in_profit: CentsUsdPattern3,
}

impl InPattern2 {
    /// Create a new pattern node with accumulated series name.
    pub fn new(client: Arc<BrkClientBase>, acc: String) -> Self {
        Self {
            in_loss: CentsUsdPattern3::new(client.clone(), _m(&acc, "loss")),
            in_profit: CentsUsdPattern3::new(client.clone(), _m(&acc, "profit")),
        }
    }
}

/// Pattern struct for repeated tree structure.
pub struct InPattern {
    pub in_loss: SharePattern,
    pub in_profit: SharePattern,
}

impl InPattern {
    /// Create a new pattern node with accumulated series name.
    pub fn new(client: Arc<BrkClientBase>, acc: String) -> Self {
        Self {
            in_loss: SharePattern::new(client.clone(), _m(&acc, "loss_share")),
            in_profit: SharePattern::new(client.clone(), _m(&acc, "profit_share")),
        }
    }
}

/// Pattern struct for repeated tree structure.
pub struct PerPattern {
    pub per_coin: CentsSatsUsdPattern,
    pub per_dollar: CentsSatsUsdPattern,
}

impl PerPattern {
    /// Create a new pattern node with accumulated series name.
    pub fn new(client: Arc<BrkClientBase>, acc: String) -> Self {
        Self {
            per_coin: CentsSatsUsdPattern::new(client.clone(), _m(&acc, "coin")),
            per_dollar: CentsSatsUsdPattern::new(client.clone(), _m(&acc, "dollar")),
        }
    }
}

/// Pattern struct for repeated tree structure.
pub struct PriceRatioPattern {
    pub price: CentsSatsUsdPattern,
    pub ratio: SeriesPattern1<StoredF32>,
}

impl PriceRatioPattern {
    /// Create a new pattern node with accumulated series name.
    pub fn new(client: Arc<BrkClientBase>, acc: String, disc: String) -> Self {
        Self {
            price: CentsSatsUsdPattern::new(client.clone(), _m(&acc, &disc)),
            ratio: SeriesPattern1::new(client.clone(), _m(&acc, &format!("ratio_{disc}", disc=disc))),
        }
    }
}

/// Pattern struct for repeated tree structure.
pub struct RatioValuePattern2 {
    pub ratio: _1m1w1y24hPattern<StoredF64>,
    pub value_destroyed: AverageBlockCumulativeSumPattern<Cents>,
}

impl RatioValuePattern2 {
    /// Create a new pattern node with accumulated series name.
    pub fn new(client: Arc<BrkClientBase>, acc: String) -> Self {
        Self {
            ratio: _1m1w1y24hPattern::new(client.clone(), _m(&acc, "sopr")),
            value_destroyed: AverageBlockCumulativeSumPattern::new(client.clone(), _m(&acc, "value_destroyed")),
        }
    }
}

/// Pattern struct for repeated tree structure.
pub struct RatioValuePattern {
    pub ratio: _24hPattern,
    pub value_destroyed: AverageBlockCumulativeSumPattern<Cents>,
}

impl RatioValuePattern {
    /// Create a new pattern node with accumulated series name.
    pub fn new(client: Arc<BrkClientBase>, acc: String) -> Self {
        Self {
            ratio: _24hPattern::new(client.clone(), _m(&acc, "sopr_24h")),
            value_destroyed: AverageBlockCumulativeSumPattern::new(client.clone(), _m(&acc, "value_destroyed")),
        }
    }
}

/// Pattern struct for repeated tree structure.
pub struct SdSmaPattern {
    pub sd: SeriesPattern1<StoredF32>,
    pub sma: SeriesPattern1<StoredF32>,
}

/// Pattern struct for repeated tree structure.
pub struct ToPattern {
    pub to_mcap: BpsPercentRatioPattern,
    pub to_rcap: BpsPercentRatioPattern,
}

impl ToPattern {
    /// Create a new pattern node with accumulated series name.
    pub fn new(client: Arc<BrkClientBase>, acc: String) -> Self {
        Self {
            to_mcap: BpsPercentRatioPattern::new(client.clone(), _m(&acc, "mcap")),
            to_rcap: BpsPercentRatioPattern::new(client.clone(), _m(&acc, "rcap")),
        }
    }
}

/// Pattern struct for repeated tree structure.
pub struct _24hPattern {
    pub _24h: SeriesPattern1<StoredF64>,
}

impl _24hPattern {
    /// Create a new pattern node with accumulated series name.
    pub fn new(client: Arc<BrkClientBase>, acc: String) -> Self {
        Self {
            _24h: SeriesPattern1::new(client.clone(), acc.clone()),
        }
    }
}

/// Pattern struct for repeated tree structure.
pub struct NuplPattern {
    pub nupl: BpsRatioPattern,
}

impl NuplPattern {
    /// Create a new pattern node with accumulated series name.
    pub fn new(client: Arc<BrkClientBase>, acc: String) -> Self {
        Self {
            nupl: BpsRatioPattern::new(client.clone(), acc.clone()),
        }
    }
}

/// Pattern struct for repeated tree structure.
pub struct PricePattern {
    pub price: BpsCentsPercentilesRatioSatsUsdPattern,
}

impl PricePattern {
    /// Create a new pattern node with accumulated series name.
    pub fn new(client: Arc<BrkClientBase>, acc: String) -> Self {
        Self {
            price: BpsCentsPercentilesRatioSatsUsdPattern::new(client.clone(), acc.clone()),
        }
    }
}

/// Pattern struct for repeated tree structure.
pub struct SharePattern {
    pub share: BpsPercentRatioPattern2,
}

impl SharePattern {
    /// Create a new pattern node with accumulated series name.
    pub fn new(client: Arc<BrkClientBase>, acc: String) -> Self {
        Self {
            share: BpsPercentRatioPattern2::new(client.clone(), acc.clone()),
        }
    }
}

/// Pattern struct for repeated tree structure.
pub struct TransferPattern {
    pub transfer_volume: AverageBlockCumulativeSumPattern3,
}

impl TransferPattern {
    /// Create a new pattern node with accumulated series name.
    pub fn new(client: Arc<BrkClientBase>, acc: String) -> Self {
        Self {
            transfer_volume: AverageBlockCumulativeSumPattern3::new(client.clone(), acc.clone()),
        }
    }
}

// Series tree

/// Series tree node.
pub struct SeriesTree {
    pub blocks: SeriesTree_Blocks,
    pub transactions: SeriesTree_Transactions,
    pub inputs: SeriesTree_Inputs,
    pub outputs: SeriesTree_Outputs,
    pub addrs: SeriesTree_Addrs,
    pub scripts: SeriesTree_Scripts,
    pub mining: SeriesTree_Mining,
    pub cointime: SeriesTree_Cointime,
    pub constants: SeriesTree_Constants,
    pub indexes: SeriesTree_Indexes,
    pub indicators: SeriesTree_Indicators,
    pub investing: SeriesTree_Investing,
    pub market: SeriesTree_Market,
    pub pools: SeriesTree_Pools,
    pub price: SeriesTree_Price,
    pub supply: SeriesTree_Supply,
    pub cohorts: SeriesTree_Cohorts,
}

impl SeriesTree {
    pub fn new(client: Arc<BrkClientBase>, base_path: String) -> Self {
        Self {
            blocks: SeriesTree_Blocks::new(client.clone(), format!("{base_path}_blocks")),
            transactions: SeriesTree_Transactions::new(client.clone(), format!("{base_path}_transactions")),
            inputs: SeriesTree_Inputs::new(client.clone(), format!("{base_path}_inputs")),
            outputs: SeriesTree_Outputs::new(client.clone(), format!("{base_path}_outputs")),
            addrs: SeriesTree_Addrs::new(client.clone(), format!("{base_path}_addrs")),
            scripts: SeriesTree_Scripts::new(client.clone(), format!("{base_path}_scripts")),
            mining: SeriesTree_Mining::new(client.clone(), format!("{base_path}_mining")),
            cointime: SeriesTree_Cointime::new(client.clone(), format!("{base_path}_cointime")),
            constants: SeriesTree_Constants::new(client.clone(), format!("{base_path}_constants")),
            indexes: SeriesTree_Indexes::new(client.clone(), format!("{base_path}_indexes")),
            indicators: SeriesTree_Indicators::new(client.clone(), format!("{base_path}_indicators")),
            investing: SeriesTree_Investing::new(client.clone(), format!("{base_path}_investing")),
            market: SeriesTree_Market::new(client.clone(), format!("{base_path}_market")),
            pools: SeriesTree_Pools::new(client.clone(), format!("{base_path}_pools")),
            price: SeriesTree_Price::new(client.clone(), format!("{base_path}_price")),
            supply: SeriesTree_Supply::new(client.clone(), format!("{base_path}_supply")),
            cohorts: SeriesTree_Cohorts::new(client.clone(), format!("{base_path}_cohorts")),
        }
    }
}

/// Series tree node.
pub struct SeriesTree_Blocks {
    pub blockhash: SeriesPattern18<BlockHash>,
    pub coinbase_tag: SeriesPattern18<CoinbaseTag>,
    pub difficulty: SeriesTree_Blocks_Difficulty,
    pub time: SeriesTree_Blocks_Time,
    pub size: SeriesTree_Blocks_Size,
    pub weight: AverageBaseCumulativeMaxMedianMinPct10Pct25Pct75Pct90SumPattern<Weight>,
    pub segwit_txs: SeriesPattern18<StoredU32>,
    pub segwit_size: SeriesPattern18<StoredU64>,
    pub segwit_weight: SeriesPattern18<Weight>,
    pub count: SeriesTree_Blocks_Count,
    pub lookback: SeriesTree_Blocks_Lookback,
    pub interval: SeriesTree_Blocks_Interval,
    pub vbytes: AverageBlockCumulativeMaxMedianMinPct10Pct25Pct75Pct90SumPattern,
    pub fullness: SeriesTree_Blocks_Fullness,
    pub halving: SeriesTree_Blocks_Halving,
}

impl SeriesTree_Blocks {
    pub fn new(client: Arc<BrkClientBase>, base_path: String) -> Self {
        Self {
            blockhash: SeriesPattern18::new(client.clone(), "blockhash".to_string()),
            coinbase_tag: SeriesPattern18::new(client.clone(), "coinbase_tag".to_string()),
            difficulty: SeriesTree_Blocks_Difficulty::new(client.clone(), format!("{base_path}_difficulty")),
            time: SeriesTree_Blocks_Time::new(client.clone(), format!("{base_path}_time")),
            size: SeriesTree_Blocks_Size::new(client.clone(), format!("{base_path}_size")),
            weight: AverageBaseCumulativeMaxMedianMinPct10Pct25Pct75Pct90SumPattern::new(client.clone(), "block_weight".to_string()),
            segwit_txs: SeriesPattern18::new(client.clone(), "segwit_txs".to_string()),
            segwit_size: SeriesPattern18::new(client.clone(), "segwit_size".to_string()),
            segwit_weight: SeriesPattern18::new(client.clone(), "segwit_weight".to_string()),
            count: SeriesTree_Blocks_Count::new(client.clone(), format!("{base_path}_count")),
            lookback: SeriesTree_Blocks_Lookback::new(client.clone(), format!("{base_path}_lookback")),
            interval: SeriesTree_Blocks_Interval::new(client.clone(), format!("{base_path}_interval")),
            vbytes: AverageBlockCumulativeMaxMedianMinPct10Pct25Pct75Pct90SumPattern::new(client.clone(), "block_vbytes".to_string()),
            fullness: SeriesTree_Blocks_Fullness::new(client.clone(), format!("{base_path}_fullness")),
            halving: SeriesTree_Blocks_Halving::new(client.clone(), format!("{base_path}_halving")),
        }
    }
}

/// Series tree node.
pub struct SeriesTree_Blocks_Difficulty {
    pub value: SeriesPattern1<StoredF64>,
    pub hashrate: SeriesPattern1<StoredF64>,
    pub adjustment: BpsPercentRatioPattern,
    pub epoch: SeriesPattern1<Epoch>,
    pub blocks_to_retarget: SeriesPattern1<StoredU32>,
    pub days_to_retarget: SeriesPattern1<StoredF32>,
}

impl SeriesTree_Blocks_Difficulty {
    pub fn new(client: Arc<BrkClientBase>, base_path: String) -> Self {
        Self {
            value: SeriesPattern1::new(client.clone(), "difficulty".to_string()),
            hashrate: SeriesPattern1::new(client.clone(), "difficulty_hashrate".to_string()),
            adjustment: BpsPercentRatioPattern::new(client.clone(), "difficulty_adjustment".to_string()),
            epoch: SeriesPattern1::new(client.clone(), "difficulty_epoch".to_string()),
            blocks_to_retarget: SeriesPattern1::new(client.clone(), "blocks_to_retarget".to_string()),
            days_to_retarget: SeriesPattern1::new(client.clone(), "days_to_retarget".to_string()),
        }
    }
}

/// Series tree node.
pub struct SeriesTree_Blocks_Time {
    pub timestamp: SeriesPattern18<Timestamp>,
}

impl SeriesTree_Blocks_Time {
    pub fn new(client: Arc<BrkClientBase>, base_path: String) -> Self {
        Self {
            timestamp: SeriesPattern18::new(client.clone(), "timestamp".to_string()),
        }
    }
}

/// Series tree node.
pub struct SeriesTree_Blocks_Size {
    pub base: SeriesPattern18<StoredU64>,
    pub cumulative: SeriesPattern1<StoredU64>,
    pub sum: _1m1w1y24hPattern<StoredU64>,
    pub average: _1m1w1y24hPattern<StoredF32>,
    pub min: _1m1w1y24hPattern<StoredU64>,
    pub max: _1m1w1y24hPattern<StoredU64>,
    pub pct10: _1m1w1y24hPattern<StoredU64>,
    pub pct25: _1m1w1y24hPattern<StoredU64>,
    pub median: _1m1w1y24hPattern<StoredU64>,
    pub pct75: _1m1w1y24hPattern<StoredU64>,
    pub pct90: _1m1w1y24hPattern<StoredU64>,
}

impl SeriesTree_Blocks_Size {
    pub fn new(client: Arc<BrkClientBase>, base_path: String) -> Self {
        Self {
            base: SeriesPattern18::new(client.clone(), "total_size".to_string()),
            cumulative: SeriesPattern1::new(client.clone(), "block_size_cumulative".to_string()),
            sum: _1m1w1y24hPattern::new(client.clone(), "block_size_sum".to_string()),
            average: _1m1w1y24hPattern::new(client.clone(), "block_size_average".to_string()),
            min: _1m1w1y24hPattern::new(client.clone(), "block_size_min".to_string()),
            max: _1m1w1y24hPattern::new(client.clone(), "block_size_max".to_string()),
            pct10: _1m1w1y24hPattern::new(client.clone(), "block_size_pct10".to_string()),
            pct25: _1m1w1y24hPattern::new(client.clone(), "block_size_pct25".to_string()),
            median: _1m1w1y24hPattern::new(client.clone(), "block_size_median".to_string()),
            pct75: _1m1w1y24hPattern::new(client.clone(), "block_size_pct75".to_string()),
            pct90: _1m1w1y24hPattern::new(client.clone(), "block_size_pct90".to_string()),
        }
    }
}

/// Series tree node.
pub struct SeriesTree_Blocks_Count {
    pub target: _1m1w1y24hPattern<StoredU64>,
    pub total: AverageBlockCumulativeSumPattern2,
}

impl SeriesTree_Blocks_Count {
    pub fn new(client: Arc<BrkClientBase>, base_path: String) -> Self {
        Self {
            target: _1m1w1y24hPattern::new(client.clone(), "block_count_target".to_string()),
            total: AverageBlockCumulativeSumPattern2::new(client.clone(), "block_count".to_string()),
        }
    }
}

/// Series tree node.
pub struct SeriesTree_Blocks_Lookback {
    pub _1h: SeriesPattern18<Height>,
    pub _24h: SeriesPattern18<Height>,
    pub _3d: SeriesPattern18<Height>,
    pub _1w: SeriesPattern18<Height>,
    pub _8d: SeriesPattern18<Height>,
    pub _9d: SeriesPattern18<Height>,
    pub _12d: SeriesPattern18<Height>,
    pub _13d: SeriesPattern18<Height>,
    pub _2w: SeriesPattern18<Height>,
    pub _21d: SeriesPattern18<Height>,
    pub _26d: SeriesPattern18<Height>,
    pub _1m: SeriesPattern18<Height>,
    pub _34d: SeriesPattern18<Height>,
    pub _55d: SeriesPattern18<Height>,
    pub _2m: SeriesPattern18<Height>,
    pub _9w: SeriesPattern18<Height>,
    pub _12w: SeriesPattern18<Height>,
    pub _89d: SeriesPattern18<Height>,
    pub _3m: SeriesPattern18<Height>,
    pub _14w: SeriesPattern18<Height>,
    pub _111d: SeriesPattern18<Height>,
    pub _144d: SeriesPattern18<Height>,
    pub _6m: SeriesPattern18<Height>,
    pub _26w: SeriesPattern18<Height>,
    pub _200d: SeriesPattern18<Height>,
    pub _9m: SeriesPattern18<Height>,
    pub _350d: SeriesPattern18<Height>,
    pub _12m: SeriesPattern18<Height>,
    pub _1y: SeriesPattern18<Height>,
    pub _14m: SeriesPattern18<Height>,
    pub _2y: SeriesPattern18<Height>,
    pub _26m: SeriesPattern18<Height>,
    pub _3y: SeriesPattern18<Height>,
    pub _200w: SeriesPattern18<Height>,
    pub _4y: SeriesPattern18<Height>,
    pub _5y: SeriesPattern18<Height>,
    pub _6y: SeriesPattern18<Height>,
    pub _8y: SeriesPattern18<Height>,
    pub _9y: SeriesPattern18<Height>,
    pub _10y: SeriesPattern18<Height>,
    pub _12y: SeriesPattern18<Height>,
    pub _14y: SeriesPattern18<Height>,
    pub _26y: SeriesPattern18<Height>,
}

impl SeriesTree_Blocks_Lookback {
    pub fn new(client: Arc<BrkClientBase>, base_path: String) -> Self {
        Self {
            _1h: SeriesPattern18::new(client.clone(), "height_1h_ago".to_string()),
            _24h: SeriesPattern18::new(client.clone(), "height_24h_ago".to_string()),
            _3d: SeriesPattern18::new(client.clone(), "height_3d_ago".to_string()),
            _1w: SeriesPattern18::new(client.clone(), "height_1w_ago".to_string()),
            _8d: SeriesPattern18::new(client.clone(), "height_8d_ago".to_string()),
            _9d: SeriesPattern18::new(client.clone(), "height_9d_ago".to_string()),
            _12d: SeriesPattern18::new(client.clone(), "height_12d_ago".to_string()),
            _13d: SeriesPattern18::new(client.clone(), "height_13d_ago".to_string()),
            _2w: SeriesPattern18::new(client.clone(), "height_2w_ago".to_string()),
            _21d: SeriesPattern18::new(client.clone(), "height_21d_ago".to_string()),
            _26d: SeriesPattern18::new(client.clone(), "height_26d_ago".to_string()),
            _1m: SeriesPattern18::new(client.clone(), "height_1m_ago".to_string()),
            _34d: SeriesPattern18::new(client.clone(), "height_34d_ago".to_string()),
            _55d: SeriesPattern18::new(client.clone(), "height_55d_ago".to_string()),
            _2m: SeriesPattern18::new(client.clone(), "height_2m_ago".to_string()),
            _9w: SeriesPattern18::new(client.clone(), "height_9w_ago".to_string()),
            _12w: SeriesPattern18::new(client.clone(), "height_12w_ago".to_string()),
            _89d: SeriesPattern18::new(client.clone(), "height_89d_ago".to_string()),
            _3m: SeriesPattern18::new(client.clone(), "height_3m_ago".to_string()),
            _14w: SeriesPattern18::new(client.clone(), "height_14w_ago".to_string()),
            _111d: SeriesPattern18::new(client.clone(), "height_111d_ago".to_string()),
            _144d: SeriesPattern18::new(client.clone(), "height_144d_ago".to_string()),
            _6m: SeriesPattern18::new(client.clone(), "height_6m_ago".to_string()),
            _26w: SeriesPattern18::new(client.clone(), "height_26w_ago".to_string()),
            _200d: SeriesPattern18::new(client.clone(), "height_200d_ago".to_string()),
            _9m: SeriesPattern18::new(client.clone(), "height_9m_ago".to_string()),
            _350d: SeriesPattern18::new(client.clone(), "height_350d_ago".to_string()),
            _12m: SeriesPattern18::new(client.clone(), "height_12m_ago".to_string()),
            _1y: SeriesPattern18::new(client.clone(), "height_1y_ago".to_string()),
            _14m: SeriesPattern18::new(client.clone(), "height_14m_ago".to_string()),
            _2y: SeriesPattern18::new(client.clone(), "height_2y_ago".to_string()),
            _26m: SeriesPattern18::new(client.clone(), "height_26m_ago".to_string()),
            _3y: SeriesPattern18::new(client.clone(), "height_3y_ago".to_string()),
            _200w: SeriesPattern18::new(client.clone(), "height_200w_ago".to_string()),
            _4y: SeriesPattern18::new(client.clone(), "height_4y_ago".to_string()),
            _5y: SeriesPattern18::new(client.clone(), "height_5y_ago".to_string()),
            _6y: SeriesPattern18::new(client.clone(), "height_6y_ago".to_string()),
            _8y: SeriesPattern18::new(client.clone(), "height_8y_ago".to_string()),
            _9y: SeriesPattern18::new(client.clone(), "height_9y_ago".to_string()),
            _10y: SeriesPattern18::new(client.clone(), "height_10y_ago".to_string()),
            _12y: SeriesPattern18::new(client.clone(), "height_12y_ago".to_string()),
            _14y: SeriesPattern18::new(client.clone(), "height_14y_ago".to_string()),
            _26y: SeriesPattern18::new(client.clone(), "height_26y_ago".to_string()),
        }
    }
}

/// Series tree node.
pub struct SeriesTree_Blocks_Interval {
    pub block: SeriesPattern18<Timestamp>,
    pub _24h: SeriesPattern1<StoredF32>,
    pub _1w: SeriesPattern1<StoredF32>,
    pub _1m: SeriesPattern1<StoredF32>,
    pub _1y: SeriesPattern1<StoredF32>,
}

impl SeriesTree_Blocks_Interval {
    pub fn new(client: Arc<BrkClientBase>, base_path: String) -> Self {
        Self {
            block: SeriesPattern18::new(client.clone(), "block_interval".to_string()),
            _24h: SeriesPattern1::new(client.clone(), "block_interval_average_24h".to_string()),
            _1w: SeriesPattern1::new(client.clone(), "block_interval_average_1w".to_string()),
            _1m: SeriesPattern1::new(client.clone(), "block_interval_average_1m".to_string()),
            _1y: SeriesPattern1::new(client.clone(), "block_interval_average_1y".to_string()),
        }
    }
}

/// Series tree node.
pub struct SeriesTree_Blocks_Fullness {
    pub bps: SeriesPattern18<BasisPoints16>,
    pub ratio: SeriesPattern18<StoredF32>,
    pub percent: SeriesPattern18<StoredF32>,
}

impl SeriesTree_Blocks_Fullness {
    pub fn new(client: Arc<BrkClientBase>, base_path: String) -> Self {
        Self {
            bps: SeriesPattern18::new(client.clone(), "block_fullness_bps".to_string()),
            ratio: SeriesPattern18::new(client.clone(), "block_fullness_ratio".to_string()),
            percent: SeriesPattern18::new(client.clone(), "block_fullness".to_string()),
        }
    }
}

/// Series tree node.
pub struct SeriesTree_Blocks_Halving {
    pub epoch: SeriesPattern1<Halving>,
    pub blocks_to_halving: SeriesPattern1<StoredU32>,
    pub days_to_halving: SeriesPattern1<StoredF32>,
}

impl SeriesTree_Blocks_Halving {
    pub fn new(client: Arc<BrkClientBase>, base_path: String) -> Self {
        Self {
            epoch: SeriesPattern1::new(client.clone(), "halving_epoch".to_string()),
            blocks_to_halving: SeriesPattern1::new(client.clone(), "blocks_to_halving".to_string()),
            days_to_halving: SeriesPattern1::new(client.clone(), "days_to_halving".to_string()),
        }
    }
}

/// Series tree node.
pub struct SeriesTree_Transactions {
    pub raw: SeriesTree_Transactions_Raw,
    pub count: SeriesTree_Transactions_Count,
    pub size: SeriesTree_Transactions_Size,
    pub fees: SeriesTree_Transactions_Fees,
    pub versions: SeriesTree_Transactions_Versions,
    pub volume: SeriesTree_Transactions_Volume,
}

impl SeriesTree_Transactions {
    pub fn new(client: Arc<BrkClientBase>, base_path: String) -> Self {
        Self {
            raw: SeriesTree_Transactions_Raw::new(client.clone(), format!("{base_path}_raw")),
            count: SeriesTree_Transactions_Count::new(client.clone(), format!("{base_path}_count")),
            size: SeriesTree_Transactions_Size::new(client.clone(), format!("{base_path}_size")),
            fees: SeriesTree_Transactions_Fees::new(client.clone(), format!("{base_path}_fees")),
            versions: SeriesTree_Transactions_Versions::new(client.clone(), format!("{base_path}_versions")),
            volume: SeriesTree_Transactions_Volume::new(client.clone(), format!("{base_path}_volume")),
        }
    }
}

/// Series tree node.
pub struct SeriesTree_Transactions_Raw {
    pub first_tx_index: SeriesPattern18<TxIndex>,
    pub txid: SeriesPattern19<Txid>,
    pub tx_version: SeriesPattern19<TxVersion>,
    pub raw_locktime: SeriesPattern19<RawLockTime>,
    pub base_size: SeriesPattern19<StoredU32>,
    pub total_size: SeriesPattern19<StoredU32>,
    pub total_sigop_cost: SeriesPattern19<SigOps>,
    pub is_explicitly_rbf: SeriesPattern19<StoredBool>,
    pub first_txin_index: SeriesPattern19<TxInIndex>,
    pub first_txout_index: SeriesPattern19<TxOutIndex>,
}

impl SeriesTree_Transactions_Raw {
    pub fn new(client: Arc<BrkClientBase>, base_path: String) -> Self {
        Self {
            first_tx_index: SeriesPattern18::new(client.clone(), "first_tx_index".to_string()),
            txid: SeriesPattern19::new(client.clone(), "txid".to_string()),
            tx_version: SeriesPattern19::new(client.clone(), "tx_version".to_string()),
            raw_locktime: SeriesPattern19::new(client.clone(), "raw_locktime".to_string()),
            base_size: SeriesPattern19::new(client.clone(), "base_size".to_string()),
            total_size: SeriesPattern19::new(client.clone(), "total_size".to_string()),
            total_sigop_cost: SeriesPattern19::new(client.clone(), "total_sigop_cost".to_string()),
            is_explicitly_rbf: SeriesPattern19::new(client.clone(), "is_explicitly_rbf".to_string()),
            first_txin_index: SeriesPattern19::new(client.clone(), "first_txin_index".to_string()),
            first_txout_index: SeriesPattern19::new(client.clone(), "first_txout_index".to_string()),
        }
    }
}

/// Series tree node.
pub struct SeriesTree_Transactions_Count {
    pub total: AverageBlockCumulativeMaxMedianMinPct10Pct25Pct75Pct90SumPattern,
}

impl SeriesTree_Transactions_Count {
    pub fn new(client: Arc<BrkClientBase>, base_path: String) -> Self {
        Self {
            total: AverageBlockCumulativeMaxMedianMinPct10Pct25Pct75Pct90SumPattern::new(client.clone(), "tx_count".to_string()),
        }
    }
}

/// Series tree node.
pub struct SeriesTree_Transactions_Size {
    pub vsize: _6bBlockTxPattern<VSize>,
    pub weight: SeriesTree_Transactions_Size_Weight,
}

impl SeriesTree_Transactions_Size {
    pub fn new(client: Arc<BrkClientBase>, base_path: String) -> Self {
        Self {
            vsize: _6bBlockTxPattern::new(client.clone(), "tx_vsize".to_string()),
            weight: SeriesTree_Transactions_Size_Weight::new(client.clone(), format!("{base_path}_weight")),
        }
    }
}

/// Series tree node.
pub struct SeriesTree_Transactions_Size_Weight {
    pub tx_index: SeriesPattern19<Weight>,
    pub block: MaxMedianMinPct10Pct25Pct75Pct90Pattern2,
    pub _6b: MaxMedianMinPct10Pct25Pct75Pct90Pattern2,
}

impl SeriesTree_Transactions_Size_Weight {
    pub fn new(client: Arc<BrkClientBase>, base_path: String) -> Self {
        Self {
            tx_index: SeriesPattern19::new(client.clone(), "tx_weight".to_string()),
            block: MaxMedianMinPct10Pct25Pct75Pct90Pattern2::new(client.clone(), "tx_weight".to_string()),
            _6b: MaxMedianMinPct10Pct25Pct75Pct90Pattern2::new(client.clone(), "tx_weight_6b".to_string()),
        }
    }
}

/// Series tree node.
pub struct SeriesTree_Transactions_Fees {
    pub input_value: SeriesPattern19<Sats>,
    pub output_value: SeriesPattern19<Sats>,
    pub fee: _6bBlockTxPattern<Sats>,
    pub fee_rate: SeriesPattern19<FeeRate>,
    pub effective_fee_rate: _6bBlockTxPattern<FeeRate>,
}

impl SeriesTree_Transactions_Fees {
    pub fn new(client: Arc<BrkClientBase>, base_path: String) -> Self {
        Self {
            input_value: SeriesPattern19::new(client.clone(), "input_value".to_string()),
            output_value: SeriesPattern19::new(client.clone(), "output_value".to_string()),
            fee: _6bBlockTxPattern::new(client.clone(), "fee".to_string()),
            fee_rate: SeriesPattern19::new(client.clone(), "fee_rate".to_string()),
            effective_fee_rate: _6bBlockTxPattern::new(client.clone(), "effective_fee_rate".to_string()),
        }
    }
}

/// Series tree node.
pub struct SeriesTree_Transactions_Versions {
    pub v1: AverageBlockCumulativeSumPattern<StoredU64>,
    pub v2: AverageBlockCumulativeSumPattern<StoredU64>,
    pub v3: AverageBlockCumulativeSumPattern<StoredU64>,
}

impl SeriesTree_Transactions_Versions {
    pub fn new(client: Arc<BrkClientBase>, base_path: String) -> Self {
        Self {
            v1: AverageBlockCumulativeSumPattern::new(client.clone(), "tx_v1".to_string()),
            v2: AverageBlockCumulativeSumPattern::new(client.clone(), "tx_v2".to_string()),
            v3: AverageBlockCumulativeSumPattern::new(client.clone(), "tx_v3".to_string()),
        }
    }
}

/// Series tree node.
pub struct SeriesTree_Transactions_Volume {
    pub transfer_volume: AverageBlockCumulativeSumPattern3,
    pub tx_per_sec: _1m1w1y24hPattern<StoredF32>,
}

impl SeriesTree_Transactions_Volume {
    pub fn new(client: Arc<BrkClientBase>, base_path: String) -> Self {
        Self {
            transfer_volume: AverageBlockCumulativeSumPattern3::new(client.clone(), "transfer_volume_bis".to_string()),
            tx_per_sec: _1m1w1y24hPattern::new(client.clone(), "tx_per_sec".to_string()),
        }
    }
}

/// Series tree node.
pub struct SeriesTree_Inputs {
    pub raw: SeriesTree_Inputs_Raw,
    pub spent: SeriesTree_Inputs_Spent,
    pub count: CumulativeRollingSumPattern,
    pub per_sec: _1m1w1y24hPattern<StoredF32>,
    pub by_type: SeriesTree_Inputs_ByType,
}

impl SeriesTree_Inputs {
    pub fn new(client: Arc<BrkClientBase>, base_path: String) -> Self {
        Self {
            raw: SeriesTree_Inputs_Raw::new(client.clone(), format!("{base_path}_raw")),
            spent: SeriesTree_Inputs_Spent::new(client.clone(), format!("{base_path}_spent")),
            count: CumulativeRollingSumPattern::new(client.clone(), "input_count".to_string()),
            per_sec: _1m1w1y24hPattern::new(client.clone(), "inputs_per_sec".to_string()),
            by_type: SeriesTree_Inputs_ByType::new(client.clone(), format!("{base_path}_by_type")),
        }
    }
}

/// Series tree node.
pub struct SeriesTree_Inputs_Raw {
    pub first_txin_index: SeriesPattern18<TxInIndex>,
    pub outpoint: SeriesPattern20<OutPoint>,
    pub tx_index: SeriesPattern20<TxIndex>,
    pub output_type: SeriesPattern20<OutputType>,
    pub type_index: SeriesPattern20<TypeIndex>,
}

impl SeriesTree_Inputs_Raw {
    pub fn new(client: Arc<BrkClientBase>, base_path: String) -> Self {
        Self {
            first_txin_index: SeriesPattern18::new(client.clone(), "first_txin_index".to_string()),
            outpoint: SeriesPattern20::new(client.clone(), "outpoint".to_string()),
            tx_index: SeriesPattern20::new(client.clone(), "tx_index".to_string()),
            output_type: SeriesPattern20::new(client.clone(), "output_type".to_string()),
            type_index: SeriesPattern20::new(client.clone(), "type_index".to_string()),
        }
    }
}

/// Series tree node.
pub struct SeriesTree_Inputs_Spent {
    pub txout_index: SeriesPattern20<TxOutIndex>,
    pub value: SeriesPattern20<Sats>,
}

impl SeriesTree_Inputs_Spent {
    pub fn new(client: Arc<BrkClientBase>, base_path: String) -> Self {
        Self {
            txout_index: SeriesPattern20::new(client.clone(), "txout_index".to_string()),
            value: SeriesPattern20::new(client.clone(), "value".to_string()),
        }
    }
}

/// Series tree node.
pub struct SeriesTree_Inputs_ByType {
    pub input_count: SeriesTree_Inputs_ByType_InputCount,
    pub input_share: SeriesTree_Inputs_ByType_InputShare,
    pub tx_count: SeriesTree_Inputs_ByType_TxCount,
    pub tx_share: EmptyP2aP2msP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshUnknownPattern2,
}

impl SeriesTree_Inputs_ByType {
    pub fn new(client: Arc<BrkClientBase>, base_path: String) -> Self {
        Self {
            input_count: SeriesTree_Inputs_ByType_InputCount::new(client.clone(), format!("{base_path}_input_count")),
            input_share: SeriesTree_Inputs_ByType_InputShare::new(client.clone(), format!("{base_path}_input_share")),
            tx_count: SeriesTree_Inputs_ByType_TxCount::new(client.clone(), format!("{base_path}_tx_count")),
            tx_share: EmptyP2aP2msP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshUnknownPattern2::new(client.clone(), "tx_share_with".to_string()),
        }
    }
}

/// Series tree node.
pub struct SeriesTree_Inputs_ByType_InputCount {
    pub all: AverageBlockCumulativeSumPattern<StoredU64>,
    pub p2pk65: AverageBlockCumulativeSumPattern<StoredU64>,
    pub p2pk33: AverageBlockCumulativeSumPattern<StoredU64>,
    pub p2pkh: AverageBlockCumulativeSumPattern<StoredU64>,
    pub p2ms: AverageBlockCumulativeSumPattern<StoredU64>,
    pub p2sh: AverageBlockCumulativeSumPattern<StoredU64>,
    pub p2wpkh: AverageBlockCumulativeSumPattern<StoredU64>,
    pub p2wsh: AverageBlockCumulativeSumPattern<StoredU64>,
    pub p2tr: AverageBlockCumulativeSumPattern<StoredU64>,
    pub p2a: AverageBlockCumulativeSumPattern<StoredU64>,
    pub unknown: AverageBlockCumulativeSumPattern<StoredU64>,
    pub empty: AverageBlockCumulativeSumPattern<StoredU64>,
}

impl SeriesTree_Inputs_ByType_InputCount {
    pub fn new(client: Arc<BrkClientBase>, base_path: String) -> Self {
        Self {
            all: AverageBlockCumulativeSumPattern::new(client.clone(), "input_count_bis".to_string()),
            p2pk65: AverageBlockCumulativeSumPattern::new(client.clone(), "p2pk65_prevout_count".to_string()),
            p2pk33: AverageBlockCumulativeSumPattern::new(client.clone(), "p2pk33_prevout_count".to_string()),
            p2pkh: AverageBlockCumulativeSumPattern::new(client.clone(), "p2pkh_prevout_count".to_string()),
            p2ms: AverageBlockCumulativeSumPattern::new(client.clone(), "p2ms_prevout_count".to_string()),
            p2sh: AverageBlockCumulativeSumPattern::new(client.clone(), "p2sh_prevout_count".to_string()),
            p2wpkh: AverageBlockCumulativeSumPattern::new(client.clone(), "p2wpkh_prevout_count".to_string()),
            p2wsh: AverageBlockCumulativeSumPattern::new(client.clone(), "p2wsh_prevout_count".to_string()),
            p2tr: AverageBlockCumulativeSumPattern::new(client.clone(), "p2tr_prevout_count".to_string()),
            p2a: AverageBlockCumulativeSumPattern::new(client.clone(), "p2a_prevout_count".to_string()),
            unknown: AverageBlockCumulativeSumPattern::new(client.clone(), "unknown_outputs_prevout_count".to_string()),
            empty: AverageBlockCumulativeSumPattern::new(client.clone(), "empty_outputs_prevout_count".to_string()),
        }
    }
}

/// Series tree node.
pub struct SeriesTree_Inputs_ByType_InputShare {
    pub p2pk65: _1m1w1y24hBpsPercentRatioPattern,
    pub p2pk33: _1m1w1y24hBpsPercentRatioPattern,
    pub p2pkh: _1m1w1y24hBpsPercentRatioPattern,
    pub p2ms: _1m1w1y24hBpsPercentRatioPattern,
    pub p2sh: _1m1w1y24hBpsPercentRatioPattern,
    pub p2wpkh: _1m1w1y24hBpsPercentRatioPattern,
    pub p2wsh: _1m1w1y24hBpsPercentRatioPattern,
    pub p2tr: _1m1w1y24hBpsPercentRatioPattern,
    pub p2a: _1m1w1y24hBpsPercentRatioPattern,
    pub unknown: _1m1w1y24hBpsPercentRatioPattern,
    pub empty: _1m1w1y24hBpsPercentRatioPattern,
}

impl SeriesTree_Inputs_ByType_InputShare {
    pub fn new(client: Arc<BrkClientBase>, base_path: String) -> Self {
        Self {
            p2pk65: _1m1w1y24hBpsPercentRatioPattern::new(client.clone(), "p2pk65_prevout_share".to_string()),
            p2pk33: _1m1w1y24hBpsPercentRatioPattern::new(client.clone(), "p2pk33_prevout_share".to_string()),
            p2pkh: _1m1w1y24hBpsPercentRatioPattern::new(client.clone(), "p2pkh_prevout_share".to_string()),
            p2ms: _1m1w1y24hBpsPercentRatioPattern::new(client.clone(), "p2ms_prevout_share".to_string()),
            p2sh: _1m1w1y24hBpsPercentRatioPattern::new(client.clone(), "p2sh_prevout_share".to_string()),
            p2wpkh: _1m1w1y24hBpsPercentRatioPattern::new(client.clone(), "p2wpkh_prevout_share".to_string()),
            p2wsh: _1m1w1y24hBpsPercentRatioPattern::new(client.clone(), "p2wsh_prevout_share".to_string()),
            p2tr: _1m1w1y24hBpsPercentRatioPattern::new(client.clone(), "p2tr_prevout_share".to_string()),
            p2a: _1m1w1y24hBpsPercentRatioPattern::new(client.clone(), "p2a_prevout_share".to_string()),
            unknown: _1m1w1y24hBpsPercentRatioPattern::new(client.clone(), "unknown_outputs_prevout_share".to_string()),
            empty: _1m1w1y24hBpsPercentRatioPattern::new(client.clone(), "empty_outputs_prevout_share".to_string()),
        }
    }
}

/// Series tree node.
pub struct SeriesTree_Inputs_ByType_TxCount {
    pub all: AverageBlockCumulativeSumPattern<StoredU64>,
    pub p2pk65: AverageBlockCumulativeSumPattern<StoredU64>,
    pub p2pk33: AverageBlockCumulativeSumPattern<StoredU64>,
    pub p2pkh: AverageBlockCumulativeSumPattern<StoredU64>,
    pub p2ms: AverageBlockCumulativeSumPattern<StoredU64>,
    pub p2sh: AverageBlockCumulativeSumPattern<StoredU64>,
    pub p2wpkh: AverageBlockCumulativeSumPattern<StoredU64>,
    pub p2wsh: AverageBlockCumulativeSumPattern<StoredU64>,
    pub p2tr: AverageBlockCumulativeSumPattern<StoredU64>,
    pub p2a: AverageBlockCumulativeSumPattern<StoredU64>,
    pub unknown: AverageBlockCumulativeSumPattern<StoredU64>,
    pub empty: AverageBlockCumulativeSumPattern<StoredU64>,
}

impl SeriesTree_Inputs_ByType_TxCount {
    pub fn new(client: Arc<BrkClientBase>, base_path: String) -> Self {
        Self {
            all: AverageBlockCumulativeSumPattern::new(client.clone(), "non_coinbase_tx_count".to_string()),
            p2pk65: AverageBlockCumulativeSumPattern::new(client.clone(), "tx_count_with_p2pk65_prevout".to_string()),
            p2pk33: AverageBlockCumulativeSumPattern::new(client.clone(), "tx_count_with_p2pk33_prevout".to_string()),
            p2pkh: AverageBlockCumulativeSumPattern::new(client.clone(), "tx_count_with_p2pkh_prevout".to_string()),
            p2ms: AverageBlockCumulativeSumPattern::new(client.clone(), "tx_count_with_p2ms_prevout".to_string()),
            p2sh: AverageBlockCumulativeSumPattern::new(client.clone(), "tx_count_with_p2sh_prevout".to_string()),
            p2wpkh: AverageBlockCumulativeSumPattern::new(client.clone(), "tx_count_with_p2wpkh_prevout".to_string()),
            p2wsh: AverageBlockCumulativeSumPattern::new(client.clone(), "tx_count_with_p2wsh_prevout".to_string()),
            p2tr: AverageBlockCumulativeSumPattern::new(client.clone(), "tx_count_with_p2tr_prevout".to_string()),
            p2a: AverageBlockCumulativeSumPattern::new(client.clone(), "tx_count_with_p2a_prevout".to_string()),
            unknown: AverageBlockCumulativeSumPattern::new(client.clone(), "tx_count_with_unknown_outputs_prevout".to_string()),
            empty: AverageBlockCumulativeSumPattern::new(client.clone(), "tx_count_with_empty_outputs_prevout".to_string()),
        }
    }
}

/// Series tree node.
pub struct SeriesTree_Outputs {
    pub raw: SeriesTree_Outputs_Raw,
    pub spent: SeriesTree_Outputs_Spent,
    pub count: SeriesTree_Outputs_Count,
    pub per_sec: _1m1w1y24hPattern<StoredF32>,
    pub unspent: SeriesTree_Outputs_Unspent,
    pub by_type: SeriesTree_Outputs_ByType,
    pub value: SeriesTree_Outputs_Value,
}

impl SeriesTree_Outputs {
    pub fn new(client: Arc<BrkClientBase>, base_path: String) -> Self {
        Self {
            raw: SeriesTree_Outputs_Raw::new(client.clone(), format!("{base_path}_raw")),
            spent: SeriesTree_Outputs_Spent::new(client.clone(), format!("{base_path}_spent")),
            count: SeriesTree_Outputs_Count::new(client.clone(), format!("{base_path}_count")),
            per_sec: _1m1w1y24hPattern::new(client.clone(), "outputs_per_sec".to_string()),
            unspent: SeriesTree_Outputs_Unspent::new(client.clone(), format!("{base_path}_unspent")),
            by_type: SeriesTree_Outputs_ByType::new(client.clone(), format!("{base_path}_by_type")),
            value: SeriesTree_Outputs_Value::new(client.clone(), format!("{base_path}_value")),
        }
    }
}

/// Series tree node.
pub struct SeriesTree_Outputs_Raw {
    pub first_txout_index: SeriesPattern18<TxOutIndex>,
    pub value: SeriesPattern21<Sats>,
    pub output_type: SeriesPattern21<OutputType>,
    pub type_index: SeriesPattern21<TypeIndex>,
    pub tx_index: SeriesPattern21<TxIndex>,
}

impl SeriesTree_Outputs_Raw {
    pub fn new(client: Arc<BrkClientBase>, base_path: String) -> Self {
        Self {
            first_txout_index: SeriesPattern18::new(client.clone(), "first_txout_index".to_string()),
            value: SeriesPattern21::new(client.clone(), "value".to_string()),
            output_type: SeriesPattern21::new(client.clone(), "output_type".to_string()),
            type_index: SeriesPattern21::new(client.clone(), "type_index".to_string()),
            tx_index: SeriesPattern21::new(client.clone(), "tx_index".to_string()),
        }
    }
}

/// Series tree node.
pub struct SeriesTree_Outputs_Spent {
    pub txin_index: SeriesPattern21<TxInIndex>,
}

impl SeriesTree_Outputs_Spent {
    pub fn new(client: Arc<BrkClientBase>, base_path: String) -> Self {
        Self {
            txin_index: SeriesPattern21::new(client.clone(), "txin_index".to_string()),
        }
    }
}

/// Series tree node.
pub struct SeriesTree_Outputs_Count {
    pub total: CumulativeRollingSumPattern,
}

impl SeriesTree_Outputs_Count {
    pub fn new(client: Arc<BrkClientBase>, base_path: String) -> Self {
        Self {
            total: CumulativeRollingSumPattern::new(client.clone(), "output_count".to_string()),
        }
    }
}

/// Series tree node.
pub struct SeriesTree_Outputs_Unspent {
    pub count: SeriesPattern1<StoredU64>,
}

impl SeriesTree_Outputs_Unspent {
    pub fn new(client: Arc<BrkClientBase>, base_path: String) -> Self {
        Self {
            count: SeriesPattern1::new(client.clone(), "utxo_count_bis".to_string()),
        }
    }
}

/// Series tree node.
pub struct SeriesTree_Outputs_ByType {
    pub output_count: SeriesTree_Outputs_ByType_OutputCount,
    pub spendable_output_count: AverageBlockCumulativeSumPattern<StoredU64>,
    pub output_share: SeriesTree_Outputs_ByType_OutputShare,
    pub tx_count: AllEmptyOpP2aP2msP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshUnknownPattern,
    pub tx_share: EmptyOpP2aP2msP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshUnknownPattern2,
}

impl SeriesTree_Outputs_ByType {
    pub fn new(client: Arc<BrkClientBase>, base_path: String) -> Self {
        Self {
            output_count: SeriesTree_Outputs_ByType_OutputCount::new(client.clone(), format!("{base_path}_output_count")),
            spendable_output_count: AverageBlockCumulativeSumPattern::new(client.clone(), "spendable_output_count".to_string()),
            output_share: SeriesTree_Outputs_ByType_OutputShare::new(client.clone(), format!("{base_path}_output_share")),
            tx_count: AllEmptyOpP2aP2msP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshUnknownPattern::new(client.clone(), "tx_count".to_string()),
            tx_share: EmptyOpP2aP2msP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshUnknownPattern2::new(client.clone(), "tx_share_with".to_string()),
        }
    }
}

/// Series tree node.
pub struct SeriesTree_Outputs_ByType_OutputCount {
    pub all: AverageBlockCumulativeSumPattern<StoredU64>,
    pub p2pk65: AverageBlockCumulativeSumPattern<StoredU64>,
    pub p2pk33: AverageBlockCumulativeSumPattern<StoredU64>,
    pub p2pkh: AverageBlockCumulativeSumPattern<StoredU64>,
    pub p2ms: AverageBlockCumulativeSumPattern<StoredU64>,
    pub p2sh: AverageBlockCumulativeSumPattern<StoredU64>,
    pub p2wpkh: AverageBlockCumulativeSumPattern<StoredU64>,
    pub p2wsh: AverageBlockCumulativeSumPattern<StoredU64>,
    pub p2tr: AverageBlockCumulativeSumPattern<StoredU64>,
    pub p2a: AverageBlockCumulativeSumPattern<StoredU64>,
    pub unknown: AverageBlockCumulativeSumPattern<StoredU64>,
    pub empty: AverageBlockCumulativeSumPattern<StoredU64>,
    pub op_return: AverageBlockCumulativeSumPattern<StoredU64>,
}

impl SeriesTree_Outputs_ByType_OutputCount {
    pub fn new(client: Arc<BrkClientBase>, base_path: String) -> Self {
        Self {
            all: AverageBlockCumulativeSumPattern::new(client.clone(), "output_count_bis".to_string()),
            p2pk65: AverageBlockCumulativeSumPattern::new(client.clone(), "p2pk65_output_count".to_string()),
            p2pk33: AverageBlockCumulativeSumPattern::new(client.clone(), "p2pk33_output_count".to_string()),
            p2pkh: AverageBlockCumulativeSumPattern::new(client.clone(), "p2pkh_output_count".to_string()),
            p2ms: AverageBlockCumulativeSumPattern::new(client.clone(), "p2ms_output_count".to_string()),
            p2sh: AverageBlockCumulativeSumPattern::new(client.clone(), "p2sh_output_count".to_string()),
            p2wpkh: AverageBlockCumulativeSumPattern::new(client.clone(), "p2wpkh_output_count".to_string()),
            p2wsh: AverageBlockCumulativeSumPattern::new(client.clone(), "p2wsh_output_count".to_string()),
            p2tr: AverageBlockCumulativeSumPattern::new(client.clone(), "p2tr_output_count".to_string()),
            p2a: AverageBlockCumulativeSumPattern::new(client.clone(), "p2a_output_count".to_string()),
            unknown: AverageBlockCumulativeSumPattern::new(client.clone(), "unknown_outputs_output_count".to_string()),
            empty: AverageBlockCumulativeSumPattern::new(client.clone(), "empty_outputs_output_count".to_string()),
            op_return: AverageBlockCumulativeSumPattern::new(client.clone(), "op_return_output_count".to_string()),
        }
    }
}

/// Series tree node.
pub struct SeriesTree_Outputs_ByType_OutputShare {
    pub p2pk65: _1m1w1y24hBpsPercentRatioPattern,
    pub p2pk33: _1m1w1y24hBpsPercentRatioPattern,
    pub p2pkh: _1m1w1y24hBpsPercentRatioPattern,
    pub p2ms: _1m1w1y24hBpsPercentRatioPattern,
    pub p2sh: _1m1w1y24hBpsPercentRatioPattern,
    pub p2wpkh: _1m1w1y24hBpsPercentRatioPattern,
    pub p2wsh: _1m1w1y24hBpsPercentRatioPattern,
    pub p2tr: _1m1w1y24hBpsPercentRatioPattern,
    pub p2a: _1m1w1y24hBpsPercentRatioPattern,
    pub unknown: _1m1w1y24hBpsPercentRatioPattern,
    pub empty: _1m1w1y24hBpsPercentRatioPattern,
    pub op_return: _1m1w1y24hBpsPercentRatioPattern,
}

impl SeriesTree_Outputs_ByType_OutputShare {
    pub fn new(client: Arc<BrkClientBase>, base_path: String) -> Self {
        Self {
            p2pk65: _1m1w1y24hBpsPercentRatioPattern::new(client.clone(), "p2pk65_output_share".to_string()),
            p2pk33: _1m1w1y24hBpsPercentRatioPattern::new(client.clone(), "p2pk33_output_share".to_string()),
            p2pkh: _1m1w1y24hBpsPercentRatioPattern::new(client.clone(), "p2pkh_output_share".to_string()),
            p2ms: _1m1w1y24hBpsPercentRatioPattern::new(client.clone(), "p2ms_output_share".to_string()),
            p2sh: _1m1w1y24hBpsPercentRatioPattern::new(client.clone(), "p2sh_output_share".to_string()),
            p2wpkh: _1m1w1y24hBpsPercentRatioPattern::new(client.clone(), "p2wpkh_output_share".to_string()),
            p2wsh: _1m1w1y24hBpsPercentRatioPattern::new(client.clone(), "p2wsh_output_share".to_string()),
            p2tr: _1m1w1y24hBpsPercentRatioPattern::new(client.clone(), "p2tr_output_share".to_string()),
            p2a: _1m1w1y24hBpsPercentRatioPattern::new(client.clone(), "p2a_output_share".to_string()),
            unknown: _1m1w1y24hBpsPercentRatioPattern::new(client.clone(), "unknown_outputs_output_share".to_string()),
            empty: _1m1w1y24hBpsPercentRatioPattern::new(client.clone(), "empty_outputs_output_share".to_string()),
            op_return: _1m1w1y24hBpsPercentRatioPattern::new(client.clone(), "op_return_output_share".to_string()),
        }
    }
}

/// Series tree node.
pub struct SeriesTree_Outputs_Value {
    pub op_return: BlockCumulativePattern,
}

impl SeriesTree_Outputs_Value {
    pub fn new(client: Arc<BrkClientBase>, base_path: String) -> Self {
        Self {
            op_return: BlockCumulativePattern::new(client.clone(), "op_return_value".to_string()),
        }
    }
}

/// Series tree node.
pub struct SeriesTree_Addrs {
    pub raw: SeriesTree_Addrs_Raw,
    pub indexes: SeriesTree_Addrs_Indexes,
    pub data: SeriesTree_Addrs_Data,
    pub funded: AllP2aP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshPattern4,
    pub empty: AllP2aP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshPattern4,
    pub activity: SeriesTree_Addrs_Activity,
    pub total: AllP2aP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshPattern4,
    pub new: AllP2aP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshPattern6,
    pub reused: SeriesTree_Addrs_Reused,
    pub respent: SeriesTree_Addrs_Respent,
    pub exposed: SeriesTree_Addrs_Exposed,
    pub delta: SeriesTree_Addrs_Delta,
    pub avg_amount: SeriesTree_Addrs_AvgAmount,
}

impl SeriesTree_Addrs {
    pub fn new(client: Arc<BrkClientBase>, base_path: String) -> Self {
        Self {
            raw: SeriesTree_Addrs_Raw::new(client.clone(), format!("{base_path}_raw")),
            indexes: SeriesTree_Addrs_Indexes::new(client.clone(), format!("{base_path}_indexes")),
            data: SeriesTree_Addrs_Data::new(client.clone(), format!("{base_path}_data")),
            funded: AllP2aP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshPattern4::new(client.clone(), "addr_count".to_string()),
            empty: AllP2aP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshPattern4::new(client.clone(), "empty_addr_count".to_string()),
            activity: SeriesTree_Addrs_Activity::new(client.clone(), format!("{base_path}_activity")),
            total: AllP2aP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshPattern4::new(client.clone(), "total_addr_count".to_string()),
            new: AllP2aP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshPattern6::new(client.clone(), "new_addr_count".to_string()),
            reused: SeriesTree_Addrs_Reused::new(client.clone(), format!("{base_path}_reused")),
            respent: SeriesTree_Addrs_Respent::new(client.clone(), format!("{base_path}_respent")),
            exposed: SeriesTree_Addrs_Exposed::new(client.clone(), format!("{base_path}_exposed")),
            delta: SeriesTree_Addrs_Delta::new(client.clone(), format!("{base_path}_delta")),
            avg_amount: SeriesTree_Addrs_AvgAmount::new(client.clone(), format!("{base_path}_avg_amount")),
        }
    }
}

/// Series tree node.
pub struct SeriesTree_Addrs_Raw {
    pub p2pk65: SeriesTree_Addrs_Raw_P2pk65,
    pub p2pk33: SeriesTree_Addrs_Raw_P2pk33,
    pub p2pkh: SeriesTree_Addrs_Raw_P2pkh,
    pub p2sh: SeriesTree_Addrs_Raw_P2sh,
    pub p2wpkh: SeriesTree_Addrs_Raw_P2wpkh,
    pub p2wsh: SeriesTree_Addrs_Raw_P2wsh,
    pub p2tr: SeriesTree_Addrs_Raw_P2tr,
    pub p2a: SeriesTree_Addrs_Raw_P2a,
}

impl SeriesTree_Addrs_Raw {
    pub fn new(client: Arc<BrkClientBase>, base_path: String) -> Self {
        Self {
            p2pk65: SeriesTree_Addrs_Raw_P2pk65::new(client.clone(), format!("{base_path}_p2pk65")),
            p2pk33: SeriesTree_Addrs_Raw_P2pk33::new(client.clone(), format!("{base_path}_p2pk33")),
            p2pkh: SeriesTree_Addrs_Raw_P2pkh::new(client.clone(), format!("{base_path}_p2pkh")),
            p2sh: SeriesTree_Addrs_Raw_P2sh::new(client.clone(), format!("{base_path}_p2sh")),
            p2wpkh: SeriesTree_Addrs_Raw_P2wpkh::new(client.clone(), format!("{base_path}_p2wpkh")),
            p2wsh: SeriesTree_Addrs_Raw_P2wsh::new(client.clone(), format!("{base_path}_p2wsh")),
            p2tr: SeriesTree_Addrs_Raw_P2tr::new(client.clone(), format!("{base_path}_p2tr")),
            p2a: SeriesTree_Addrs_Raw_P2a::new(client.clone(), format!("{base_path}_p2a")),
        }
    }
}

/// Series tree node.
pub struct SeriesTree_Addrs_Raw_P2pk65 {
    pub first_index: SeriesPattern18<P2PK65AddrIndex>,
    pub bytes: SeriesPattern27<P2PK65Bytes>,
}

impl SeriesTree_Addrs_Raw_P2pk65 {
    pub fn new(client: Arc<BrkClientBase>, base_path: String) -> Self {
        Self {
            first_index: SeriesPattern18::new(client.clone(), "first_p2pk65_addr_index".to_string()),
            bytes: SeriesPattern27::new(client.clone(), "p2pk65_bytes".to_string()),
        }
    }
}

/// Series tree node.
pub struct SeriesTree_Addrs_Raw_P2pk33 {
    pub first_index: SeriesPattern18<P2PK33AddrIndex>,
    pub bytes: SeriesPattern26<P2PK33Bytes>,
}

impl SeriesTree_Addrs_Raw_P2pk33 {
    pub fn new(client: Arc<BrkClientBase>, base_path: String) -> Self {
        Self {
            first_index: SeriesPattern18::new(client.clone(), "first_p2pk33_addr_index".to_string()),
            bytes: SeriesPattern26::new(client.clone(), "p2pk33_bytes".to_string()),
        }
    }
}

/// Series tree node.
pub struct SeriesTree_Addrs_Raw_P2pkh {
    pub first_index: SeriesPattern18<P2PKHAddrIndex>,
    pub bytes: SeriesPattern28<P2PKHBytes>,
}

impl SeriesTree_Addrs_Raw_P2pkh {
    pub fn new(client: Arc<BrkClientBase>, base_path: String) -> Self {
        Self {
            first_index: SeriesPattern18::new(client.clone(), "first_p2pkh_addr_index".to_string()),
            bytes: SeriesPattern28::new(client.clone(), "p2pkh_bytes".to_string()),
        }
    }
}

/// Series tree node.
pub struct SeriesTree_Addrs_Raw_P2sh {
    pub first_index: SeriesPattern18<P2SHAddrIndex>,
    pub bytes: SeriesPattern29<P2SHBytes>,
}

impl SeriesTree_Addrs_Raw_P2sh {
    pub fn new(client: Arc<BrkClientBase>, base_path: String) -> Self {
        Self {
            first_index: SeriesPattern18::new(client.clone(), "first_p2sh_addr_index".to_string()),
            bytes: SeriesPattern29::new(client.clone(), "p2sh_bytes".to_string()),
        }
    }
}

/// Series tree node.
pub struct SeriesTree_Addrs_Raw_P2wpkh {
    pub first_index: SeriesPattern18<P2WPKHAddrIndex>,
    pub bytes: SeriesPattern31<P2WPKHBytes>,
}

impl SeriesTree_Addrs_Raw_P2wpkh {
    pub fn new(client: Arc<BrkClientBase>, base_path: String) -> Self {
        Self {
            first_index: SeriesPattern18::new(client.clone(), "first_p2wpkh_addr_index".to_string()),
            bytes: SeriesPattern31::new(client.clone(), "p2wpkh_bytes".to_string()),
        }
    }
}

/// Series tree node.
pub struct SeriesTree_Addrs_Raw_P2wsh {
    pub first_index: SeriesPattern18<P2WSHAddrIndex>,
    pub bytes: SeriesPattern32<P2WSHBytes>,
}

impl SeriesTree_Addrs_Raw_P2wsh {
    pub fn new(client: Arc<BrkClientBase>, base_path: String) -> Self {
        Self {
            first_index: SeriesPattern18::new(client.clone(), "first_p2wsh_addr_index".to_string()),
            bytes: SeriesPattern32::new(client.clone(), "p2wsh_bytes".to_string()),
        }
    }
}

/// Series tree node.
pub struct SeriesTree_Addrs_Raw_P2tr {
    pub first_index: SeriesPattern18<P2TRAddrIndex>,
    pub bytes: SeriesPattern30<P2TRBytes>,
}

impl SeriesTree_Addrs_Raw_P2tr {
    pub fn new(client: Arc<BrkClientBase>, base_path: String) -> Self {
        Self {
            first_index: SeriesPattern18::new(client.clone(), "first_p2tr_addr_index".to_string()),
            bytes: SeriesPattern30::new(client.clone(), "p2tr_bytes".to_string()),
        }
    }
}

/// Series tree node.
pub struct SeriesTree_Addrs_Raw_P2a {
    pub first_index: SeriesPattern18<P2AAddrIndex>,
    pub bytes: SeriesPattern24<P2ABytes>,
}

impl SeriesTree_Addrs_Raw_P2a {
    pub fn new(client: Arc<BrkClientBase>, base_path: String) -> Self {
        Self {
            first_index: SeriesPattern18::new(client.clone(), "first_p2a_addr_index".to_string()),
            bytes: SeriesPattern24::new(client.clone(), "p2a_bytes".to_string()),
        }
    }
}

/// Series tree node.
pub struct SeriesTree_Addrs_Indexes {
    pub p2a: SeriesPattern24<AnyAddrIndex>,
    pub p2pk33: SeriesPattern26<AnyAddrIndex>,
    pub p2pk65: SeriesPattern27<AnyAddrIndex>,
    pub p2pkh: SeriesPattern28<AnyAddrIndex>,
    pub p2sh: SeriesPattern29<AnyAddrIndex>,
    pub p2tr: SeriesPattern30<AnyAddrIndex>,
    pub p2wpkh: SeriesPattern31<AnyAddrIndex>,
    pub p2wsh: SeriesPattern32<AnyAddrIndex>,
    pub funded: SeriesPattern34<FundedAddrIndex>,
    pub empty: SeriesPattern35<EmptyAddrIndex>,
}

impl SeriesTree_Addrs_Indexes {
    pub fn new(client: Arc<BrkClientBase>, base_path: String) -> Self {
        Self {
            p2a: SeriesPattern24::new(client.clone(), "any_addr_index".to_string()),
            p2pk33: SeriesPattern26::new(client.clone(), "any_addr_index".to_string()),
            p2pk65: SeriesPattern27::new(client.clone(), "any_addr_index".to_string()),
            p2pkh: SeriesPattern28::new(client.clone(), "any_addr_index".to_string()),
            p2sh: SeriesPattern29::new(client.clone(), "any_addr_index".to_string()),
            p2tr: SeriesPattern30::new(client.clone(), "any_addr_index".to_string()),
            p2wpkh: SeriesPattern31::new(client.clone(), "any_addr_index".to_string()),
            p2wsh: SeriesPattern32::new(client.clone(), "any_addr_index".to_string()),
            funded: SeriesPattern34::new(client.clone(), "funded_addr_index".to_string()),
            empty: SeriesPattern35::new(client.clone(), "empty_addr_index".to_string()),
        }
    }
}

/// Series tree node.
pub struct SeriesTree_Addrs_Data {
    pub funded: SeriesPattern34<FundedAddrData>,
    pub empty: SeriesPattern35<EmptyAddrData>,
}

impl SeriesTree_Addrs_Data {
    pub fn new(client: Arc<BrkClientBase>, base_path: String) -> Self {
        Self {
            funded: SeriesPattern34::new(client.clone(), "funded_addr_data".to_string()),
            empty: SeriesPattern35::new(client.clone(), "empty_addr_data".to_string()),
        }
    }
}

/// Series tree node.
pub struct SeriesTree_Addrs_Activity {
    pub all: SeriesTree_Addrs_Activity_All,
    pub p2pk65: ActiveBidirectionalReactivatedReceivingSendingPattern,
    pub p2pk33: ActiveBidirectionalReactivatedReceivingSendingPattern,
    pub p2pkh: ActiveBidirectionalReactivatedReceivingSendingPattern,
    pub p2sh: ActiveBidirectionalReactivatedReceivingSendingPattern,
    pub p2wpkh: ActiveBidirectionalReactivatedReceivingSendingPattern,
    pub p2wsh: ActiveBidirectionalReactivatedReceivingSendingPattern,
    pub p2tr: ActiveBidirectionalReactivatedReceivingSendingPattern,
    pub p2a: ActiveBidirectionalReactivatedReceivingSendingPattern,
}

impl SeriesTree_Addrs_Activity {
    pub fn new(client: Arc<BrkClientBase>, base_path: String) -> Self {
        Self {
            all: SeriesTree_Addrs_Activity_All::new(client.clone(), format!("{base_path}_all")),
            p2pk65: ActiveBidirectionalReactivatedReceivingSendingPattern::new(client.clone(), "p2pk65".to_string()),
            p2pk33: ActiveBidirectionalReactivatedReceivingSendingPattern::new(client.clone(), "p2pk33".to_string()),
            p2pkh: ActiveBidirectionalReactivatedReceivingSendingPattern::new(client.clone(), "p2pkh".to_string()),
            p2sh: ActiveBidirectionalReactivatedReceivingSendingPattern::new(client.clone(), "p2sh".to_string()),
            p2wpkh: ActiveBidirectionalReactivatedReceivingSendingPattern::new(client.clone(), "p2wpkh".to_string()),
            p2wsh: ActiveBidirectionalReactivatedReceivingSendingPattern::new(client.clone(), "p2wsh".to_string()),
            p2tr: ActiveBidirectionalReactivatedReceivingSendingPattern::new(client.clone(), "p2tr".to_string()),
            p2a: ActiveBidirectionalReactivatedReceivingSendingPattern::new(client.clone(), "p2a".to_string()),
        }
    }
}

/// Series tree node.
pub struct SeriesTree_Addrs_Activity_All {
    pub reactivated: _1m1w1y24hBlockPattern,
    pub sending: _1m1w1y24hBlockPattern,
    pub receiving: _1m1w1y24hBlockPattern,
    pub bidirectional: _1m1w1y24hBlockPattern,
    pub active: _1m1w1y24hBlockPattern,
}

impl SeriesTree_Addrs_Activity_All {
    pub fn new(client: Arc<BrkClientBase>, base_path: String) -> Self {
        Self {
            reactivated: _1m1w1y24hBlockPattern::new(client.clone(), "reactivated_addrs".to_string()),
            sending: _1m1w1y24hBlockPattern::new(client.clone(), "sending_addrs".to_string()),
            receiving: _1m1w1y24hBlockPattern::new(client.clone(), "receiving_addrs".to_string()),
            bidirectional: _1m1w1y24hBlockPattern::new(client.clone(), "bidirectional_addrs".to_string()),
            active: _1m1w1y24hBlockPattern::new(client.clone(), "active_addrs".to_string()),
        }
    }
}

/// Series tree node.
pub struct SeriesTree_Addrs_Reused {
    pub count: FundedTotalPattern,
    pub events: SeriesTree_Addrs_Reused_Events,
    pub supply: SeriesTree_Addrs_Reused_Supply,
}

impl SeriesTree_Addrs_Reused {
    pub fn new(client: Arc<BrkClientBase>, base_path: String) -> Self {
        Self {
            count: FundedTotalPattern::new(client.clone(), "reused_addr_count".to_string()),
            events: SeriesTree_Addrs_Reused_Events::new(client.clone(), format!("{base_path}_events")),
            supply: SeriesTree_Addrs_Reused_Supply::new(client.clone(), format!("{base_path}_supply")),
        }
    }
}

/// Series tree node.
pub struct SeriesTree_Addrs_Reused_Events {
    pub output_to_reused_addr_count: AllP2aP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshPattern6,
    pub output_to_reused_addr_share: AllP2aP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshPattern7,
    pub spendable_output_to_reused_addr_share: _1m1w1y24hBpsPercentRatioPattern,
    pub input_from_reused_addr_count: AllP2aP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshPattern6,
    pub input_from_reused_addr_share: AllP2aP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshPattern7,
    pub active_reused_addr_count: _1m1w1y24hBlockPattern,
    pub active_reused_addr_share: _1m1w1y24hBlockPattern2,
}

impl SeriesTree_Addrs_Reused_Events {
    pub fn new(client: Arc<BrkClientBase>, base_path: String) -> Self {
        Self {
            output_to_reused_addr_count: AllP2aP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshPattern6::new(client.clone(), "output_to_reused_addr_count".to_string()),
            output_to_reused_addr_share: AllP2aP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshPattern7::new(client.clone(), "output_to_reused_addr_share".to_string()),
            spendable_output_to_reused_addr_share: _1m1w1y24hBpsPercentRatioPattern::new(client.clone(), "spendable_output_to_reused_addr_share".to_string()),
            input_from_reused_addr_count: AllP2aP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshPattern6::new(client.clone(), "input_from_reused_addr_count".to_string()),
            input_from_reused_addr_share: AllP2aP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshPattern7::new(client.clone(), "input_from_reused_addr_share".to_string()),
            active_reused_addr_count: _1m1w1y24hBlockPattern::new(client.clone(), "active_reused_addr_count".to_string()),
            active_reused_addr_share: _1m1w1y24hBlockPattern2::new(client.clone(), "active_reused_addr_share".to_string()),
        }
    }
}

/// Series tree node.
pub struct SeriesTree_Addrs_Reused_Supply {
    pub all: BtcCentsSatsUsdPattern,
    pub p2pk65: BtcCentsSatsUsdPattern,
    pub p2pk33: BtcCentsSatsUsdPattern,
    pub p2pkh: BtcCentsSatsUsdPattern,
    pub p2sh: BtcCentsSatsUsdPattern,
    pub p2wpkh: BtcCentsSatsUsdPattern,
    pub p2wsh: BtcCentsSatsUsdPattern,
    pub p2tr: BtcCentsSatsUsdPattern,
    pub p2a: BtcCentsSatsUsdPattern,
    pub share: AllP2aP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshPattern5,
}

impl SeriesTree_Addrs_Reused_Supply {
    pub fn new(client: Arc<BrkClientBase>, base_path: String) -> Self {
        Self {
            all: BtcCentsSatsUsdPattern::new(client.clone(), "reused_addr_supply".to_string()),
            p2pk65: BtcCentsSatsUsdPattern::new(client.clone(), "p2pk65_reused_addr_supply".to_string()),
            p2pk33: BtcCentsSatsUsdPattern::new(client.clone(), "p2pk33_reused_addr_supply".to_string()),
            p2pkh: BtcCentsSatsUsdPattern::new(client.clone(), "p2pkh_reused_addr_supply".to_string()),
            p2sh: BtcCentsSatsUsdPattern::new(client.clone(), "p2sh_reused_addr_supply".to_string()),
            p2wpkh: BtcCentsSatsUsdPattern::new(client.clone(), "p2wpkh_reused_addr_supply".to_string()),
            p2wsh: BtcCentsSatsUsdPattern::new(client.clone(), "p2wsh_reused_addr_supply".to_string()),
            p2tr: BtcCentsSatsUsdPattern::new(client.clone(), "p2tr_reused_addr_supply".to_string()),
            p2a: BtcCentsSatsUsdPattern::new(client.clone(), "p2a_reused_addr_supply".to_string()),
            share: AllP2aP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshPattern5::new(client.clone(), "reused_addr_supply_share".to_string()),
        }
    }
}

/// Series tree node.
pub struct SeriesTree_Addrs_Respent {
    pub count: FundedTotalPattern,
    pub events: SeriesTree_Addrs_Respent_Events,
    pub supply: SeriesTree_Addrs_Respent_Supply,
}

impl SeriesTree_Addrs_Respent {
    pub fn new(client: Arc<BrkClientBase>, base_path: String) -> Self {
        Self {
            count: FundedTotalPattern::new(client.clone(), "respent_addr_count".to_string()),
            events: SeriesTree_Addrs_Respent_Events::new(client.clone(), format!("{base_path}_events")),
            supply: SeriesTree_Addrs_Respent_Supply::new(client.clone(), format!("{base_path}_supply")),
        }
    }
}

/// Series tree node.
pub struct SeriesTree_Addrs_Respent_Events {
    pub output_to_reused_addr_count: AllP2aP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshPattern6,
    pub output_to_reused_addr_share: AllP2aP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshPattern7,
    pub spendable_output_to_reused_addr_share: _1m1w1y24hBpsPercentRatioPattern,
    pub input_from_reused_addr_count: AllP2aP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshPattern6,
    pub input_from_reused_addr_share: AllP2aP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshPattern7,
    pub active_reused_addr_count: _1m1w1y24hBlockPattern,
    pub active_reused_addr_share: _1m1w1y24hBlockPattern2,
}

impl SeriesTree_Addrs_Respent_Events {
    pub fn new(client: Arc<BrkClientBase>, base_path: String) -> Self {
        Self {
            output_to_reused_addr_count: AllP2aP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshPattern6::new(client.clone(), "output_to_respent_addr_count".to_string()),
            output_to_reused_addr_share: AllP2aP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshPattern7::new(client.clone(), "output_to_respent_addr_share".to_string()),
            spendable_output_to_reused_addr_share: _1m1w1y24hBpsPercentRatioPattern::new(client.clone(), "spendable_output_to_respent_addr_share".to_string()),
            input_from_reused_addr_count: AllP2aP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshPattern6::new(client.clone(), "input_from_respent_addr_count".to_string()),
            input_from_reused_addr_share: AllP2aP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshPattern7::new(client.clone(), "input_from_respent_addr_share".to_string()),
            active_reused_addr_count: _1m1w1y24hBlockPattern::new(client.clone(), "active_respent_addr_count".to_string()),
            active_reused_addr_share: _1m1w1y24hBlockPattern2::new(client.clone(), "active_respent_addr_share".to_string()),
        }
    }
}

/// Series tree node.
pub struct SeriesTree_Addrs_Respent_Supply {
    pub all: BtcCentsSatsUsdPattern,
    pub p2pk65: BtcCentsSatsUsdPattern,
    pub p2pk33: BtcCentsSatsUsdPattern,
    pub p2pkh: BtcCentsSatsUsdPattern,
    pub p2sh: BtcCentsSatsUsdPattern,
    pub p2wpkh: BtcCentsSatsUsdPattern,
    pub p2wsh: BtcCentsSatsUsdPattern,
    pub p2tr: BtcCentsSatsUsdPattern,
    pub p2a: BtcCentsSatsUsdPattern,
    pub share: AllP2aP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshPattern5,
}

impl SeriesTree_Addrs_Respent_Supply {
    pub fn new(client: Arc<BrkClientBase>, base_path: String) -> Self {
        Self {
            all: BtcCentsSatsUsdPattern::new(client.clone(), "respent_addr_supply".to_string()),
            p2pk65: BtcCentsSatsUsdPattern::new(client.clone(), "p2pk65_respent_addr_supply".to_string()),
            p2pk33: BtcCentsSatsUsdPattern::new(client.clone(), "p2pk33_respent_addr_supply".to_string()),
            p2pkh: BtcCentsSatsUsdPattern::new(client.clone(), "p2pkh_respent_addr_supply".to_string()),
            p2sh: BtcCentsSatsUsdPattern::new(client.clone(), "p2sh_respent_addr_supply".to_string()),
            p2wpkh: BtcCentsSatsUsdPattern::new(client.clone(), "p2wpkh_respent_addr_supply".to_string()),
            p2wsh: BtcCentsSatsUsdPattern::new(client.clone(), "p2wsh_respent_addr_supply".to_string()),
            p2tr: BtcCentsSatsUsdPattern::new(client.clone(), "p2tr_respent_addr_supply".to_string()),
            p2a: BtcCentsSatsUsdPattern::new(client.clone(), "p2a_respent_addr_supply".to_string()),
            share: AllP2aP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshPattern5::new(client.clone(), "respent_addr_supply_share".to_string()),
        }
    }
}

/// Series tree node.
pub struct SeriesTree_Addrs_Exposed {
    pub count: FundedTotalPattern,
    pub supply: SeriesTree_Addrs_Exposed_Supply,
}

impl SeriesTree_Addrs_Exposed {
    pub fn new(client: Arc<BrkClientBase>, base_path: String) -> Self {
        Self {
            count: FundedTotalPattern::new(client.clone(), "exposed_addr_count".to_string()),
            supply: SeriesTree_Addrs_Exposed_Supply::new(client.clone(), format!("{base_path}_supply")),
        }
    }
}

/// Series tree node.
pub struct SeriesTree_Addrs_Exposed_Supply {
    pub all: BtcCentsSatsUsdPattern,
    pub p2pk65: BtcCentsSatsUsdPattern,
    pub p2pk33: BtcCentsSatsUsdPattern,
    pub p2pkh: BtcCentsSatsUsdPattern,
    pub p2sh: BtcCentsSatsUsdPattern,
    pub p2wpkh: BtcCentsSatsUsdPattern,
    pub p2wsh: BtcCentsSatsUsdPattern,
    pub p2tr: BtcCentsSatsUsdPattern,
    pub p2a: BtcCentsSatsUsdPattern,
    pub share: AllP2aP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshPattern5,
}

impl SeriesTree_Addrs_Exposed_Supply {
    pub fn new(client: Arc<BrkClientBase>, base_path: String) -> Self {
        Self {
            all: BtcCentsSatsUsdPattern::new(client.clone(), "exposed_addr_supply".to_string()),
            p2pk65: BtcCentsSatsUsdPattern::new(client.clone(), "p2pk65_exposed_addr_supply".to_string()),
            p2pk33: BtcCentsSatsUsdPattern::new(client.clone(), "p2pk33_exposed_addr_supply".to_string()),
            p2pkh: BtcCentsSatsUsdPattern::new(client.clone(), "p2pkh_exposed_addr_supply".to_string()),
            p2sh: BtcCentsSatsUsdPattern::new(client.clone(), "p2sh_exposed_addr_supply".to_string()),
            p2wpkh: BtcCentsSatsUsdPattern::new(client.clone(), "p2wpkh_exposed_addr_supply".to_string()),
            p2wsh: BtcCentsSatsUsdPattern::new(client.clone(), "p2wsh_exposed_addr_supply".to_string()),
            p2tr: BtcCentsSatsUsdPattern::new(client.clone(), "p2tr_exposed_addr_supply".to_string()),
            p2a: BtcCentsSatsUsdPattern::new(client.clone(), "p2a_exposed_addr_supply".to_string()),
            share: AllP2aP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshPattern5::new(client.clone(), "exposed_addr_supply_share".to_string()),
        }
    }
}

/// Series tree node.
pub struct SeriesTree_Addrs_Delta {
    pub all: AbsoluteRatePattern,
    pub p2pk65: AbsoluteRatePattern,
    pub p2pk33: AbsoluteRatePattern,
    pub p2pkh: AbsoluteRatePattern,
    pub p2sh: AbsoluteRatePattern,
    pub p2wpkh: AbsoluteRatePattern,
    pub p2wsh: AbsoluteRatePattern,
    pub p2tr: AbsoluteRatePattern,
    pub p2a: AbsoluteRatePattern,
}

impl SeriesTree_Addrs_Delta {
    pub fn new(client: Arc<BrkClientBase>, base_path: String) -> Self {
        Self {
            all: AbsoluteRatePattern::new(client.clone(), "addr_count".to_string()),
            p2pk65: AbsoluteRatePattern::new(client.clone(), "p2pk65_addr_count".to_string()),
            p2pk33: AbsoluteRatePattern::new(client.clone(), "p2pk33_addr_count".to_string()),
            p2pkh: AbsoluteRatePattern::new(client.clone(), "p2pkh_addr_count".to_string()),
            p2sh: AbsoluteRatePattern::new(client.clone(), "p2sh_addr_count".to_string()),
            p2wpkh: AbsoluteRatePattern::new(client.clone(), "p2wpkh_addr_count".to_string()),
            p2wsh: AbsoluteRatePattern::new(client.clone(), "p2wsh_addr_count".to_string()),
            p2tr: AbsoluteRatePattern::new(client.clone(), "p2tr_addr_count".to_string()),
            p2a: AbsoluteRatePattern::new(client.clone(), "p2a_addr_count".to_string()),
        }
    }
}

/// Series tree node.
pub struct SeriesTree_Addrs_AvgAmount {
    pub all: AddrUtxoPattern,
    pub p2pk65: AddrUtxoPattern,
    pub p2pk33: AddrUtxoPattern,
    pub p2pkh: AddrUtxoPattern,
    pub p2sh: AddrUtxoPattern,
    pub p2wpkh: AddrUtxoPattern,
    pub p2wsh: AddrUtxoPattern,
    pub p2tr: AddrUtxoPattern,
    pub p2a: AddrUtxoPattern,
}

impl SeriesTree_Addrs_AvgAmount {
    pub fn new(client: Arc<BrkClientBase>, base_path: String) -> Self {
        Self {
            all: AddrUtxoPattern::new(client.clone(), "avg".to_string()),
            p2pk65: AddrUtxoPattern::new(client.clone(), "p2pk65_avg".to_string()),
            p2pk33: AddrUtxoPattern::new(client.clone(), "p2pk33_avg".to_string()),
            p2pkh: AddrUtxoPattern::new(client.clone(), "p2pkh_avg".to_string()),
            p2sh: AddrUtxoPattern::new(client.clone(), "p2sh_avg".to_string()),
            p2wpkh: AddrUtxoPattern::new(client.clone(), "p2wpkh_avg".to_string()),
            p2wsh: AddrUtxoPattern::new(client.clone(), "p2wsh_avg".to_string()),
            p2tr: AddrUtxoPattern::new(client.clone(), "p2tr_avg".to_string()),
            p2a: AddrUtxoPattern::new(client.clone(), "p2a_avg".to_string()),
        }
    }
}

/// Series tree node.
pub struct SeriesTree_Scripts {
    pub raw: SeriesTree_Scripts_Raw,
}

impl SeriesTree_Scripts {
    pub fn new(client: Arc<BrkClientBase>, base_path: String) -> Self {
        Self {
            raw: SeriesTree_Scripts_Raw::new(client.clone(), format!("{base_path}_raw")),
        }
    }
}

/// Series tree node.
pub struct SeriesTree_Scripts_Raw {
    pub empty: SeriesTree_Scripts_Raw_Empty,
    pub op_return: SeriesTree_Scripts_Raw_OpReturn,
    pub p2ms: SeriesTree_Scripts_Raw_P2ms,
    pub unknown: SeriesTree_Scripts_Raw_Unknown,
}

impl SeriesTree_Scripts_Raw {
    pub fn new(client: Arc<BrkClientBase>, base_path: String) -> Self {
        Self {
            empty: SeriesTree_Scripts_Raw_Empty::new(client.clone(), format!("{base_path}_empty")),
            op_return: SeriesTree_Scripts_Raw_OpReturn::new(client.clone(), format!("{base_path}_op_return")),
            p2ms: SeriesTree_Scripts_Raw_P2ms::new(client.clone(), format!("{base_path}_p2ms")),
            unknown: SeriesTree_Scripts_Raw_Unknown::new(client.clone(), format!("{base_path}_unknown")),
        }
    }
}

/// Series tree node.
pub struct SeriesTree_Scripts_Raw_Empty {
    pub first_index: SeriesPattern18<EmptyOutputIndex>,
    pub to_tx_index: SeriesPattern22<TxIndex>,
}

impl SeriesTree_Scripts_Raw_Empty {
    pub fn new(client: Arc<BrkClientBase>, base_path: String) -> Self {
        Self {
            first_index: SeriesPattern18::new(client.clone(), "first_empty_output_index".to_string()),
            to_tx_index: SeriesPattern22::new(client.clone(), "tx_index".to_string()),
        }
    }
}

/// Series tree node.
pub struct SeriesTree_Scripts_Raw_OpReturn {
    pub first_index: SeriesPattern18<OpReturnIndex>,
    pub to_tx_index: SeriesPattern23<TxIndex>,
}

impl SeriesTree_Scripts_Raw_OpReturn {
    pub fn new(client: Arc<BrkClientBase>, base_path: String) -> Self {
        Self {
            first_index: SeriesPattern18::new(client.clone(), "first_op_return_index".to_string()),
            to_tx_index: SeriesPattern23::new(client.clone(), "tx_index".to_string()),
        }
    }
}

/// Series tree node.
pub struct SeriesTree_Scripts_Raw_P2ms {
    pub first_index: SeriesPattern18<P2MSOutputIndex>,
    pub to_tx_index: SeriesPattern25<TxIndex>,
}

impl SeriesTree_Scripts_Raw_P2ms {
    pub fn new(client: Arc<BrkClientBase>, base_path: String) -> Self {
        Self {
            first_index: SeriesPattern18::new(client.clone(), "first_p2ms_output_index".to_string()),
            to_tx_index: SeriesPattern25::new(client.clone(), "tx_index".to_string()),
        }
    }
}

/// Series tree node.
pub struct SeriesTree_Scripts_Raw_Unknown {
    pub first_index: SeriesPattern18<UnknownOutputIndex>,
    pub to_tx_index: SeriesPattern33<TxIndex>,
}

impl SeriesTree_Scripts_Raw_Unknown {
    pub fn new(client: Arc<BrkClientBase>, base_path: String) -> Self {
        Self {
            first_index: SeriesPattern18::new(client.clone(), "first_unknown_output_index".to_string()),
            to_tx_index: SeriesPattern33::new(client.clone(), "tx_index".to_string()),
        }
    }
}

/// Series tree node.
pub struct SeriesTree_Mining {
    pub rewards: SeriesTree_Mining_Rewards,
    pub hashrate: SeriesTree_Mining_Hashrate,
}

impl SeriesTree_Mining {
    pub fn new(client: Arc<BrkClientBase>, base_path: String) -> Self {
        Self {
            rewards: SeriesTree_Mining_Rewards::new(client.clone(), format!("{base_path}_rewards")),
            hashrate: SeriesTree_Mining_Hashrate::new(client.clone(), format!("{base_path}_hashrate")),
        }
    }
}

/// Series tree node.
pub struct SeriesTree_Mining_Rewards {
    pub coinbase: AverageBlockCumulativeSumPattern3,
    pub subsidy: SeriesTree_Mining_Rewards_Subsidy,
    pub fees: SeriesTree_Mining_Rewards_Fees,
    pub output_volume: SeriesPattern18<Sats>,
    pub unclaimed: BlockCumulativePattern,
}

impl SeriesTree_Mining_Rewards {
    pub fn new(client: Arc<BrkClientBase>, base_path: String) -> Self {
        Self {
            coinbase: AverageBlockCumulativeSumPattern3::new(client.clone(), "coinbase".to_string()),
            subsidy: SeriesTree_Mining_Rewards_Subsidy::new(client.clone(), format!("{base_path}_subsidy")),
            fees: SeriesTree_Mining_Rewards_Fees::new(client.clone(), format!("{base_path}_fees")),
            output_volume: SeriesPattern18::new(client.clone(), "output_volume".to_string()),
            unclaimed: BlockCumulativePattern::new(client.clone(), "unclaimed_rewards".to_string()),
        }
    }
}

/// Series tree node.
pub struct SeriesTree_Mining_Rewards_Subsidy {
    pub block: BtcCentsSatsUsdPattern3,
    pub cumulative: BtcCentsSatsUsdPattern,
    pub sum: _1m1w1y24hPattern4,
    pub average: _1m1w1y24hPattern3,
    pub dominance: _1m1w1y24hBpsPercentRatioPattern,
}

impl SeriesTree_Mining_Rewards_Subsidy {
    pub fn new(client: Arc<BrkClientBase>, base_path: String) -> Self {
        Self {
            block: BtcCentsSatsUsdPattern3::new(client.clone(), "subsidy".to_string()),
            cumulative: BtcCentsSatsUsdPattern::new(client.clone(), "subsidy_cumulative".to_string()),
            sum: _1m1w1y24hPattern4::new(client.clone(), "subsidy_sum".to_string()),
            average: _1m1w1y24hPattern3::new(client.clone(), "subsidy_average".to_string()),
            dominance: _1m1w1y24hBpsPercentRatioPattern::new(client.clone(), "subsidy_dominance".to_string()),
        }
    }
}

/// Series tree node.
pub struct SeriesTree_Mining_Rewards_Fees {
    pub block: BtcCentsSatsUsdPattern3,
    pub cumulative: BtcCentsSatsUsdPattern,
    pub sum: _1m1w1y24hPattern4,
    pub average: _1m1w1y24hPattern3,
    pub min: _1m1w1y24hPattern4,
    pub max: _1m1w1y24hPattern4,
    pub pct10: _1m1w1y24hPattern4,
    pub pct25: _1m1w1y24hPattern4,
    pub median: _1m1w1y24hPattern4,
    pub pct75: _1m1w1y24hPattern4,
    pub pct90: _1m1w1y24hPattern4,
    pub dominance: _1m1w1y24hBpsPercentRatioPattern,
    pub to_subsidy_ratio: SeriesTree_Mining_Rewards_Fees_ToSubsidyRatio,
}

impl SeriesTree_Mining_Rewards_Fees {
    pub fn new(client: Arc<BrkClientBase>, base_path: String) -> Self {
        Self {
            block: BtcCentsSatsUsdPattern3::new(client.clone(), "fees".to_string()),
            cumulative: BtcCentsSatsUsdPattern::new(client.clone(), "fees_cumulative".to_string()),
            sum: _1m1w1y24hPattern4::new(client.clone(), "fees_sum".to_string()),
            average: _1m1w1y24hPattern3::new(client.clone(), "fees_average".to_string()),
            min: _1m1w1y24hPattern4::new(client.clone(), "fees_min".to_string()),
            max: _1m1w1y24hPattern4::new(client.clone(), "fees_max".to_string()),
            pct10: _1m1w1y24hPattern4::new(client.clone(), "fees_pct10".to_string()),
            pct25: _1m1w1y24hPattern4::new(client.clone(), "fees_pct25".to_string()),
            median: _1m1w1y24hPattern4::new(client.clone(), "fees_median".to_string()),
            pct75: _1m1w1y24hPattern4::new(client.clone(), "fees_pct75".to_string()),
            pct90: _1m1w1y24hPattern4::new(client.clone(), "fees_pct90".to_string()),
            dominance: _1m1w1y24hBpsPercentRatioPattern::new(client.clone(), "fee_dominance".to_string()),
            to_subsidy_ratio: SeriesTree_Mining_Rewards_Fees_ToSubsidyRatio::new(client.clone(), format!("{base_path}_to_subsidy_ratio")),
        }
    }
}

/// Series tree node.
pub struct SeriesTree_Mining_Rewards_Fees_ToSubsidyRatio {
    pub _24h: BpsRatioPattern2,
    pub _1w: BpsRatioPattern2,
    pub _1m: BpsRatioPattern2,
    pub _1y: BpsRatioPattern2,
}

impl SeriesTree_Mining_Rewards_Fees_ToSubsidyRatio {
    pub fn new(client: Arc<BrkClientBase>, base_path: String) -> Self {
        Self {
            _24h: BpsRatioPattern2::new(client.clone(), "fee_to_subsidy_ratio_24h".to_string()),
            _1w: BpsRatioPattern2::new(client.clone(), "fee_to_subsidy_ratio_1w".to_string()),
            _1m: BpsRatioPattern2::new(client.clone(), "fee_to_subsidy_ratio_1m".to_string()),
            _1y: BpsRatioPattern2::new(client.clone(), "fee_to_subsidy_ratio_1y".to_string()),
        }
    }
}

/// Series tree node.
pub struct SeriesTree_Mining_Hashrate {
    pub rate: SeriesTree_Mining_Hashrate_Rate,
    pub price: PhsReboundThsPattern,
    pub value: PhsReboundThsPattern,
}

impl SeriesTree_Mining_Hashrate {
    pub fn new(client: Arc<BrkClientBase>, base_path: String) -> Self {
        Self {
            rate: SeriesTree_Mining_Hashrate_Rate::new(client.clone(), format!("{base_path}_rate")),
            price: PhsReboundThsPattern::new(client.clone(), "hash_price".to_string()),
            value: PhsReboundThsPattern::new(client.clone(), "hash_value".to_string()),
        }
    }
}

/// Series tree node.
pub struct SeriesTree_Mining_Hashrate_Rate {
    pub base: SeriesPattern1<StoredF64>,
    pub sma: SeriesTree_Mining_Hashrate_Rate_Sma,
    pub ath: SeriesPattern1<StoredF64>,
    pub drawdown: BpsPercentRatioPattern5,
}

impl SeriesTree_Mining_Hashrate_Rate {
    pub fn new(client: Arc<BrkClientBase>, base_path: String) -> Self {
        Self {
            base: SeriesPattern1::new(client.clone(), "hash_rate".to_string()),
            sma: SeriesTree_Mining_Hashrate_Rate_Sma::new(client.clone(), format!("{base_path}_sma")),
            ath: SeriesPattern1::new(client.clone(), "hash_rate_ath".to_string()),
            drawdown: BpsPercentRatioPattern5::new(client.clone(), "hash_rate_drawdown".to_string()),
        }
    }
}

/// Series tree node.
pub struct SeriesTree_Mining_Hashrate_Rate_Sma {
    pub _1w: SeriesPattern1<StoredF64>,
    pub _1m: SeriesPattern1<StoredF64>,
    pub _2m: SeriesPattern1<StoredF64>,
    pub _1y: SeriesPattern1<StoredF64>,
}

impl SeriesTree_Mining_Hashrate_Rate_Sma {
    pub fn new(client: Arc<BrkClientBase>, base_path: String) -> Self {
        Self {
            _1w: SeriesPattern1::new(client.clone(), "hash_rate_sma_1w".to_string()),
            _1m: SeriesPattern1::new(client.clone(), "hash_rate_sma_1m".to_string()),
            _2m: SeriesPattern1::new(client.clone(), "hash_rate_sma_2m".to_string()),
            _1y: SeriesPattern1::new(client.clone(), "hash_rate_sma_1y".to_string()),
        }
    }
}

/// Series tree node.
pub struct SeriesTree_Cointime {
    pub activity: SeriesTree_Cointime_Activity,
    pub supply: SeriesTree_Cointime_Supply,
    pub value: SeriesTree_Cointime_Value,
    pub cap: SeriesTree_Cointime_Cap,
    pub prices: SeriesTree_Cointime_Prices,
    pub adjusted: SeriesTree_Cointime_Adjusted,
    pub reserve_risk: SeriesTree_Cointime_ReserveRisk,
}

impl SeriesTree_Cointime {
    pub fn new(client: Arc<BrkClientBase>, base_path: String) -> Self {
        Self {
            activity: SeriesTree_Cointime_Activity::new(client.clone(), format!("{base_path}_activity")),
            supply: SeriesTree_Cointime_Supply::new(client.clone(), format!("{base_path}_supply")),
            value: SeriesTree_Cointime_Value::new(client.clone(), format!("{base_path}_value")),
            cap: SeriesTree_Cointime_Cap::new(client.clone(), format!("{base_path}_cap")),
            prices: SeriesTree_Cointime_Prices::new(client.clone(), format!("{base_path}_prices")),
            adjusted: SeriesTree_Cointime_Adjusted::new(client.clone(), format!("{base_path}_adjusted")),
            reserve_risk: SeriesTree_Cointime_ReserveRisk::new(client.clone(), format!("{base_path}_reserve_risk")),
        }
    }
}

/// Series tree node.
pub struct SeriesTree_Cointime_Activity {
    pub coinblocks_created: AverageBlockCumulativeSumPattern<StoredF64>,
    pub coinblocks_stored: AverageBlockCumulativeSumPattern<StoredF64>,
    pub liveliness: SeriesPattern1<StoredF64>,
    pub vaultedness: SeriesPattern1<StoredF64>,
    pub ratio: SeriesPattern1<StoredF64>,
    pub coinblocks_destroyed: AverageBlockCumulativeSumPattern<StoredF64>,
}

impl SeriesTree_Cointime_Activity {
    pub fn new(client: Arc<BrkClientBase>, base_path: String) -> Self {
        Self {
            coinblocks_created: AverageBlockCumulativeSumPattern::new(client.clone(), "coinblocks_created".to_string()),
            coinblocks_stored: AverageBlockCumulativeSumPattern::new(client.clone(), "coinblocks_stored".to_string()),
            liveliness: SeriesPattern1::new(client.clone(), "liveliness".to_string()),
            vaultedness: SeriesPattern1::new(client.clone(), "vaultedness".to_string()),
            ratio: SeriesPattern1::new(client.clone(), "activity_to_vaultedness".to_string()),
            coinblocks_destroyed: AverageBlockCumulativeSumPattern::new(client.clone(), "coinblocks_destroyed".to_string()),
        }
    }
}

/// Series tree node.
pub struct SeriesTree_Cointime_Supply {
    pub vaulted: BtcCentsSatsUsdPattern,
    pub active: BtcCentsSatsUsdPattern,
}

impl SeriesTree_Cointime_Supply {
    pub fn new(client: Arc<BrkClientBase>, base_path: String) -> Self {
        Self {
            vaulted: BtcCentsSatsUsdPattern::new(client.clone(), "vaulted_supply".to_string()),
            active: BtcCentsSatsUsdPattern::new(client.clone(), "active_supply".to_string()),
        }
    }
}

/// Series tree node.
pub struct SeriesTree_Cointime_Value {
    pub destroyed: AverageBlockCumulativeSumPattern<StoredF64>,
    pub created: AverageBlockCumulativeSumPattern<StoredF64>,
    pub stored: AverageBlockCumulativeSumPattern<StoredF64>,
    pub vocdd: AverageBlockCumulativeSumPattern<StoredF64>,
}

impl SeriesTree_Cointime_Value {
    pub fn new(client: Arc<BrkClientBase>, base_path: String) -> Self {
        Self {
            destroyed: AverageBlockCumulativeSumPattern::new(client.clone(), "cointime_value_destroyed".to_string()),
            created: AverageBlockCumulativeSumPattern::new(client.clone(), "cointime_value_created".to_string()),
            stored: AverageBlockCumulativeSumPattern::new(client.clone(), "cointime_value_stored".to_string()),
            vocdd: AverageBlockCumulativeSumPattern::new(client.clone(), "vocdd".to_string()),
        }
    }
}

/// Series tree node.
pub struct SeriesTree_Cointime_Cap {
    pub thermo: CentsUsdPattern3,
    pub investor: CentsUsdPattern3,
    pub vaulted: CentsUsdPattern3,
    pub active: CentsUsdPattern3,
    pub cointime: CentsUsdPattern3,
    pub aviv: BpsRatioPattern2,
}

impl SeriesTree_Cointime_Cap {
    pub fn new(client: Arc<BrkClientBase>, base_path: String) -> Self {
        Self {
            thermo: CentsUsdPattern3::new(client.clone(), "thermo_cap".to_string()),
            investor: CentsUsdPattern3::new(client.clone(), "investor_cap".to_string()),
            vaulted: CentsUsdPattern3::new(client.clone(), "vaulted_cap".to_string()),
            active: CentsUsdPattern3::new(client.clone(), "active_cap".to_string()),
            cointime: CentsUsdPattern3::new(client.clone(), "cointime_cap".to_string()),
            aviv: BpsRatioPattern2::new(client.clone(), "aviv_ratio".to_string()),
        }
    }
}

/// Series tree node.
pub struct SeriesTree_Cointime_Prices {
    pub vaulted: BpsCentsPercentilesRatioSatsUsdPattern,
    pub active: BpsCentsPercentilesRatioSatsUsdPattern,
    pub true_market_mean: BpsCentsPercentilesRatioSatsUsdPattern,
    pub cointime: BpsCentsPercentilesRatioSatsUsdPattern,
}

impl SeriesTree_Cointime_Prices {
    pub fn new(client: Arc<BrkClientBase>, base_path: String) -> Self {
        Self {
            vaulted: BpsCentsPercentilesRatioSatsUsdPattern::new(client.clone(), "vaulted_price".to_string()),
            active: BpsCentsPercentilesRatioSatsUsdPattern::new(client.clone(), "active_price".to_string()),
            true_market_mean: BpsCentsPercentilesRatioSatsUsdPattern::new(client.clone(), "true_market_mean".to_string()),
            cointime: BpsCentsPercentilesRatioSatsUsdPattern::new(client.clone(), "cointime_price".to_string()),
        }
    }
}

/// Series tree node.
pub struct SeriesTree_Cointime_Adjusted {
    pub inflation_rate: BpsPercentRatioPattern,
    pub tx_velocity_native: SeriesPattern1<StoredF64>,
    pub tx_velocity_fiat: SeriesPattern1<StoredF64>,
}

impl SeriesTree_Cointime_Adjusted {
    pub fn new(client: Arc<BrkClientBase>, base_path: String) -> Self {
        Self {
            inflation_rate: BpsPercentRatioPattern::new(client.clone(), "cointime_adj_inflation_rate".to_string()),
            tx_velocity_native: SeriesPattern1::new(client.clone(), "cointime_adj_tx_velocity_btc".to_string()),
            tx_velocity_fiat: SeriesPattern1::new(client.clone(), "cointime_adj_tx_velocity_usd".to_string()),
        }
    }
}

/// Series tree node.
pub struct SeriesTree_Cointime_ReserveRisk {
    pub value: SeriesPattern1<StoredF64>,
    pub vocdd_median_1y: SeriesPattern18<StoredF64>,
    pub hodl_bank: SeriesPattern18<StoredF64>,
}

impl SeriesTree_Cointime_ReserveRisk {
    pub fn new(client: Arc<BrkClientBase>, base_path: String) -> Self {
        Self {
            value: SeriesPattern1::new(client.clone(), "reserve_risk".to_string()),
            vocdd_median_1y: SeriesPattern18::new(client.clone(), "vocdd_median_1y".to_string()),
            hodl_bank: SeriesPattern18::new(client.clone(), "hodl_bank".to_string()),
        }
    }
}

/// Series tree node.
pub struct SeriesTree_Constants {
    pub _0: SeriesPattern1<StoredU16>,
    pub _1: SeriesPattern1<StoredU16>,
    pub _2: SeriesPattern1<StoredU16>,
    pub _3: SeriesPattern1<StoredU16>,
    pub _4: SeriesPattern1<StoredU16>,
    pub _20: SeriesPattern1<StoredU16>,
    pub _30: SeriesPattern1<StoredU16>,
    pub _38_2: SeriesPattern1<StoredF32>,
    pub _50: SeriesPattern1<StoredU16>,
    pub _61_8: SeriesPattern1<StoredF32>,
    pub _70: SeriesPattern1<StoredU16>,
    pub _80: SeriesPattern1<StoredU16>,
    pub _100: SeriesPattern1<StoredU16>,
    pub _600: SeriesPattern1<StoredU16>,
    pub minus_1: SeriesPattern1<StoredI8>,
    pub minus_2: SeriesPattern1<StoredI8>,
    pub minus_3: SeriesPattern1<StoredI8>,
    pub minus_4: SeriesPattern1<StoredI8>,
}

impl SeriesTree_Constants {
    pub fn new(client: Arc<BrkClientBase>, base_path: String) -> Self {
        Self {
            _0: SeriesPattern1::new(client.clone(), "constant_0".to_string()),
            _1: SeriesPattern1::new(client.clone(), "constant_1".to_string()),
            _2: SeriesPattern1::new(client.clone(), "constant_2".to_string()),
            _3: SeriesPattern1::new(client.clone(), "constant_3".to_string()),
            _4: SeriesPattern1::new(client.clone(), "constant_4".to_string()),
            _20: SeriesPattern1::new(client.clone(), "constant_20".to_string()),
            _30: SeriesPattern1::new(client.clone(), "constant_30".to_string()),
            _38_2: SeriesPattern1::new(client.clone(), "constant_38_2".to_string()),
            _50: SeriesPattern1::new(client.clone(), "constant_50".to_string()),
            _61_8: SeriesPattern1::new(client.clone(), "constant_61_8".to_string()),
            _70: SeriesPattern1::new(client.clone(), "constant_70".to_string()),
            _80: SeriesPattern1::new(client.clone(), "constant_80".to_string()),
            _100: SeriesPattern1::new(client.clone(), "constant_100".to_string()),
            _600: SeriesPattern1::new(client.clone(), "constant_600".to_string()),
            minus_1: SeriesPattern1::new(client.clone(), "constant_minus_1".to_string()),
            minus_2: SeriesPattern1::new(client.clone(), "constant_minus_2".to_string()),
            minus_3: SeriesPattern1::new(client.clone(), "constant_minus_3".to_string()),
            minus_4: SeriesPattern1::new(client.clone(), "constant_minus_4".to_string()),
        }
    }
}

/// Series tree node.
pub struct SeriesTree_Indexes {
    pub addr: SeriesTree_Indexes_Addr,
    pub height: SeriesTree_Indexes_Height,
    pub epoch: SeriesTree_Indexes_Epoch,
    pub halving: SeriesTree_Indexes_Halving,
    pub minute10: SeriesTree_Indexes_Minute10,
    pub minute30: SeriesTree_Indexes_Minute30,
    pub hour1: SeriesTree_Indexes_Hour1,
    pub hour4: SeriesTree_Indexes_Hour4,
    pub hour12: SeriesTree_Indexes_Hour12,
    pub day1: SeriesTree_Indexes_Day1,
    pub day3: SeriesTree_Indexes_Day3,
    pub week1: SeriesTree_Indexes_Week1,
    pub month1: SeriesTree_Indexes_Month1,
    pub month3: SeriesTree_Indexes_Month3,
    pub month6: SeriesTree_Indexes_Month6,
    pub year1: SeriesTree_Indexes_Year1,
    pub year10: SeriesTree_Indexes_Year10,
    pub tx_index: SeriesTree_Indexes_TxIndex,
    pub txin_index: SeriesTree_Indexes_TxinIndex,
    pub txout_index: SeriesTree_Indexes_TxoutIndex,
    pub timestamp: SeriesTree_Indexes_Timestamp,
}

impl SeriesTree_Indexes {
    pub fn new(client: Arc<BrkClientBase>, base_path: String) -> Self {
        Self {
            addr: SeriesTree_Indexes_Addr::new(client.clone(), format!("{base_path}_addr")),
            height: SeriesTree_Indexes_Height::new(client.clone(), format!("{base_path}_height")),
            epoch: SeriesTree_Indexes_Epoch::new(client.clone(), format!("{base_path}_epoch")),
            halving: SeriesTree_Indexes_Halving::new(client.clone(), format!("{base_path}_halving")),
            minute10: SeriesTree_Indexes_Minute10::new(client.clone(), format!("{base_path}_minute10")),
            minute30: SeriesTree_Indexes_Minute30::new(client.clone(), format!("{base_path}_minute30")),
            hour1: SeriesTree_Indexes_Hour1::new(client.clone(), format!("{base_path}_hour1")),
            hour4: SeriesTree_Indexes_Hour4::new(client.clone(), format!("{base_path}_hour4")),
            hour12: SeriesTree_Indexes_Hour12::new(client.clone(), format!("{base_path}_hour12")),
            day1: SeriesTree_Indexes_Day1::new(client.clone(), format!("{base_path}_day1")),
            day3: SeriesTree_Indexes_Day3::new(client.clone(), format!("{base_path}_day3")),
            week1: SeriesTree_Indexes_Week1::new(client.clone(), format!("{base_path}_week1")),
            month1: SeriesTree_Indexes_Month1::new(client.clone(), format!("{base_path}_month1")),
            month3: SeriesTree_Indexes_Month3::new(client.clone(), format!("{base_path}_month3")),
            month6: SeriesTree_Indexes_Month6::new(client.clone(), format!("{base_path}_month6")),
            year1: SeriesTree_Indexes_Year1::new(client.clone(), format!("{base_path}_year1")),
            year10: SeriesTree_Indexes_Year10::new(client.clone(), format!("{base_path}_year10")),
            tx_index: SeriesTree_Indexes_TxIndex::new(client.clone(), format!("{base_path}_tx_index")),
            txin_index: SeriesTree_Indexes_TxinIndex::new(client.clone(), format!("{base_path}_txin_index")),
            txout_index: SeriesTree_Indexes_TxoutIndex::new(client.clone(), format!("{base_path}_txout_index")),
            timestamp: SeriesTree_Indexes_Timestamp::new(client.clone(), format!("{base_path}_timestamp")),
        }
    }
}

/// Series tree node.
pub struct SeriesTree_Indexes_Addr {
    pub p2pk33: SeriesTree_Indexes_Addr_P2pk33,
    pub p2pk65: SeriesTree_Indexes_Addr_P2pk65,
    pub p2pkh: SeriesTree_Indexes_Addr_P2pkh,
    pub p2sh: SeriesTree_Indexes_Addr_P2sh,
    pub p2tr: SeriesTree_Indexes_Addr_P2tr,
    pub p2wpkh: SeriesTree_Indexes_Addr_P2wpkh,
    pub p2wsh: SeriesTree_Indexes_Addr_P2wsh,
    pub p2a: SeriesTree_Indexes_Addr_P2a,
    pub p2ms: SeriesTree_Indexes_Addr_P2ms,
    pub empty: SeriesTree_Indexes_Addr_Empty,
    pub unknown: SeriesTree_Indexes_Addr_Unknown,
    pub op_return: SeriesTree_Indexes_Addr_OpReturn,
}

impl SeriesTree_Indexes_Addr {
    pub fn new(client: Arc<BrkClientBase>, base_path: String) -> Self {
        Self {
            p2pk33: SeriesTree_Indexes_Addr_P2pk33::new(client.clone(), format!("{base_path}_p2pk33")),
            p2pk65: SeriesTree_Indexes_Addr_P2pk65::new(client.clone(), format!("{base_path}_p2pk65")),
            p2pkh: SeriesTree_Indexes_Addr_P2pkh::new(client.clone(), format!("{base_path}_p2pkh")),
            p2sh: SeriesTree_Indexes_Addr_P2sh::new(client.clone(), format!("{base_path}_p2sh")),
            p2tr: SeriesTree_Indexes_Addr_P2tr::new(client.clone(), format!("{base_path}_p2tr")),
            p2wpkh: SeriesTree_Indexes_Addr_P2wpkh::new(client.clone(), format!("{base_path}_p2wpkh")),
            p2wsh: SeriesTree_Indexes_Addr_P2wsh::new(client.clone(), format!("{base_path}_p2wsh")),
            p2a: SeriesTree_Indexes_Addr_P2a::new(client.clone(), format!("{base_path}_p2a")),
            p2ms: SeriesTree_Indexes_Addr_P2ms::new(client.clone(), format!("{base_path}_p2ms")),
            empty: SeriesTree_Indexes_Addr_Empty::new(client.clone(), format!("{base_path}_empty")),
            unknown: SeriesTree_Indexes_Addr_Unknown::new(client.clone(), format!("{base_path}_unknown")),
            op_return: SeriesTree_Indexes_Addr_OpReturn::new(client.clone(), format!("{base_path}_op_return")),
        }
    }
}

/// Series tree node.
pub struct SeriesTree_Indexes_Addr_P2pk33 {
    pub identity: SeriesPattern26<P2PK33AddrIndex>,
    pub addr: SeriesPattern26<Addr>,
}

impl SeriesTree_Indexes_Addr_P2pk33 {
    pub fn new(client: Arc<BrkClientBase>, base_path: String) -> Self {
        Self {
            identity: SeriesPattern26::new(client.clone(), "p2pk33_addr_index".to_string()),
            addr: SeriesPattern26::new(client.clone(), "p2pk33_addr".to_string()),
        }
    }
}

/// Series tree node.
pub struct SeriesTree_Indexes_Addr_P2pk65 {
    pub identity: SeriesPattern27<P2PK65AddrIndex>,
    pub addr: SeriesPattern27<Addr>,
}

impl SeriesTree_Indexes_Addr_P2pk65 {
    pub fn new(client: Arc<BrkClientBase>, base_path: String) -> Self {
        Self {
            identity: SeriesPattern27::new(client.clone(), "p2pk65_addr_index".to_string()),
            addr: SeriesPattern27::new(client.clone(), "p2pk65_addr".to_string()),
        }
    }
}

/// Series tree node.
pub struct SeriesTree_Indexes_Addr_P2pkh {
    pub identity: SeriesPattern28<P2PKHAddrIndex>,
    pub addr: SeriesPattern28<Addr>,
}

impl SeriesTree_Indexes_Addr_P2pkh {
    pub fn new(client: Arc<BrkClientBase>, base_path: String) -> Self {
        Self {
            identity: SeriesPattern28::new(client.clone(), "p2pkh_addr_index".to_string()),
            addr: SeriesPattern28::new(client.clone(), "p2pkh_addr".to_string()),
        }
    }
}

/// Series tree node.
pub struct SeriesTree_Indexes_Addr_P2sh {
    pub identity: SeriesPattern29<P2SHAddrIndex>,
    pub addr: SeriesPattern29<Addr>,
}

impl SeriesTree_Indexes_Addr_P2sh {
    pub fn new(client: Arc<BrkClientBase>, base_path: String) -> Self {
        Self {
            identity: SeriesPattern29::new(client.clone(), "p2sh_addr_index".to_string()),
            addr: SeriesPattern29::new(client.clone(), "p2sh_addr".to_string()),
        }
    }
}

/// Series tree node.
pub struct SeriesTree_Indexes_Addr_P2tr {
    pub identity: SeriesPattern30<P2TRAddrIndex>,
    pub addr: SeriesPattern30<Addr>,
}

impl SeriesTree_Indexes_Addr_P2tr {
    pub fn new(client: Arc<BrkClientBase>, base_path: String) -> Self {
        Self {
            identity: SeriesPattern30::new(client.clone(), "p2tr_addr_index".to_string()),
            addr: SeriesPattern30::new(client.clone(), "p2tr_addr".to_string()),
        }
    }
}

/// Series tree node.
pub struct SeriesTree_Indexes_Addr_P2wpkh {
    pub identity: SeriesPattern31<P2WPKHAddrIndex>,
    pub addr: SeriesPattern31<Addr>,
}

impl SeriesTree_Indexes_Addr_P2wpkh {
    pub fn new(client: Arc<BrkClientBase>, base_path: String) -> Self {
        Self {
            identity: SeriesPattern31::new(client.clone(), "p2wpkh_addr_index".to_string()),
            addr: SeriesPattern31::new(client.clone(), "p2wpkh_addr".to_string()),
        }
    }
}

/// Series tree node.
pub struct SeriesTree_Indexes_Addr_P2wsh {
    pub identity: SeriesPattern32<P2WSHAddrIndex>,
    pub addr: SeriesPattern32<Addr>,
}

impl SeriesTree_Indexes_Addr_P2wsh {
    pub fn new(client: Arc<BrkClientBase>, base_path: String) -> Self {
        Self {
            identity: SeriesPattern32::new(client.clone(), "p2wsh_addr_index".to_string()),
            addr: SeriesPattern32::new(client.clone(), "p2wsh_addr".to_string()),
        }
    }
}

/// Series tree node.
pub struct SeriesTree_Indexes_Addr_P2a {
    pub identity: SeriesPattern24<P2AAddrIndex>,
    pub addr: SeriesPattern24<Addr>,
}

impl SeriesTree_Indexes_Addr_P2a {
    pub fn new(client: Arc<BrkClientBase>, base_path: String) -> Self {
        Self {
            identity: SeriesPattern24::new(client.clone(), "p2a_addr_index".to_string()),
            addr: SeriesPattern24::new(client.clone(), "p2a_addr".to_string()),
        }
    }
}

/// Series tree node.
pub struct SeriesTree_Indexes_Addr_P2ms {
    pub identity: SeriesPattern25<P2MSOutputIndex>,
}

impl SeriesTree_Indexes_Addr_P2ms {
    pub fn new(client: Arc<BrkClientBase>, base_path: String) -> Self {
        Self {
            identity: SeriesPattern25::new(client.clone(), "p2ms_output_index".to_string()),
        }
    }
}

/// Series tree node.
pub struct SeriesTree_Indexes_Addr_Empty {
    pub identity: SeriesPattern22<EmptyOutputIndex>,
}

impl SeriesTree_Indexes_Addr_Empty {
    pub fn new(client: Arc<BrkClientBase>, base_path: String) -> Self {
        Self {
            identity: SeriesPattern22::new(client.clone(), "empty_output_index".to_string()),
        }
    }
}

/// Series tree node.
pub struct SeriesTree_Indexes_Addr_Unknown {
    pub identity: SeriesPattern33<UnknownOutputIndex>,
}

impl SeriesTree_Indexes_Addr_Unknown {
    pub fn new(client: Arc<BrkClientBase>, base_path: String) -> Self {
        Self {
            identity: SeriesPattern33::new(client.clone(), "unknown_output_index".to_string()),
        }
    }
}

/// Series tree node.
pub struct SeriesTree_Indexes_Addr_OpReturn {
    pub identity: SeriesPattern23<OpReturnIndex>,
}

impl SeriesTree_Indexes_Addr_OpReturn {
    pub fn new(client: Arc<BrkClientBase>, base_path: String) -> Self {
        Self {
            identity: SeriesPattern23::new(client.clone(), "op_return_index".to_string()),
        }
    }
}

/// Series tree node.
pub struct SeriesTree_Indexes_Height {
    pub minute10: SeriesPattern18<Minute10>,
    pub minute30: SeriesPattern18<Minute30>,
    pub hour1: SeriesPattern18<Hour1>,
    pub hour4: SeriesPattern18<Hour4>,
    pub hour12: SeriesPattern18<Hour12>,
    pub day1: SeriesPattern18<Day1>,
    pub day3: SeriesPattern18<Day3>,
    pub epoch: SeriesPattern18<Epoch>,
    pub halving: SeriesPattern18<Halving>,
    pub week1: SeriesPattern18<Week1>,
    pub month1: SeriesPattern18<Month1>,
    pub month3: SeriesPattern18<Month3>,
    pub month6: SeriesPattern18<Month6>,
    pub year1: SeriesPattern18<Year1>,
    pub year10: SeriesPattern18<Year10>,
    pub tx_index_count: SeriesPattern18<StoredU64>,
}

impl SeriesTree_Indexes_Height {
    pub fn new(client: Arc<BrkClientBase>, base_path: String) -> Self {
        Self {
            minute10: SeriesPattern18::new(client.clone(), "minute10".to_string()),
            minute30: SeriesPattern18::new(client.clone(), "minute30".to_string()),
            hour1: SeriesPattern18::new(client.clone(), "hour1".to_string()),
            hour4: SeriesPattern18::new(client.clone(), "hour4".to_string()),
            hour12: SeriesPattern18::new(client.clone(), "hour12".to_string()),
            day1: SeriesPattern18::new(client.clone(), "day1".to_string()),
            day3: SeriesPattern18::new(client.clone(), "day3".to_string()),
            epoch: SeriesPattern18::new(client.clone(), "epoch".to_string()),
            halving: SeriesPattern18::new(client.clone(), "halving".to_string()),
            week1: SeriesPattern18::new(client.clone(), "week1".to_string()),
            month1: SeriesPattern18::new(client.clone(), "month1".to_string()),
            month3: SeriesPattern18::new(client.clone(), "month3".to_string()),
            month6: SeriesPattern18::new(client.clone(), "month6".to_string()),
            year1: SeriesPattern18::new(client.clone(), "year1".to_string()),
            year10: SeriesPattern18::new(client.clone(), "year10".to_string()),
            tx_index_count: SeriesPattern18::new(client.clone(), "tx_index_count".to_string()),
        }
    }
}

/// Series tree node.
pub struct SeriesTree_Indexes_Epoch {
    pub first_height: SeriesPattern17<Height>,
}

impl SeriesTree_Indexes_Epoch {
    pub fn new(client: Arc<BrkClientBase>, base_path: String) -> Self {
        Self {
            first_height: SeriesPattern17::new(client.clone(), "first_height".to_string()),
        }
    }
}

/// Series tree node.
pub struct SeriesTree_Indexes_Halving {
    pub first_height: SeriesPattern16<Height>,
}

impl SeriesTree_Indexes_Halving {
    pub fn new(client: Arc<BrkClientBase>, base_path: String) -> Self {
        Self {
            first_height: SeriesPattern16::new(client.clone(), "first_height".to_string()),
        }
    }
}

/// Series tree node.
pub struct SeriesTree_Indexes_Minute10 {
    pub first_height: SeriesPattern3<Height>,
}

impl SeriesTree_Indexes_Minute10 {
    pub fn new(client: Arc<BrkClientBase>, base_path: String) -> Self {
        Self {
            first_height: SeriesPattern3::new(client.clone(), "first_height".to_string()),
        }
    }
}

/// Series tree node.
pub struct SeriesTree_Indexes_Minute30 {
    pub first_height: SeriesPattern4<Height>,
}

impl SeriesTree_Indexes_Minute30 {
    pub fn new(client: Arc<BrkClientBase>, base_path: String) -> Self {
        Self {
            first_height: SeriesPattern4::new(client.clone(), "first_height".to_string()),
        }
    }
}

/// Series tree node.
pub struct SeriesTree_Indexes_Hour1 {
    pub first_height: SeriesPattern5<Height>,
}

impl SeriesTree_Indexes_Hour1 {
    pub fn new(client: Arc<BrkClientBase>, base_path: String) -> Self {
        Self {
            first_height: SeriesPattern5::new(client.clone(), "first_height".to_string()),
        }
    }
}

/// Series tree node.
pub struct SeriesTree_Indexes_Hour4 {
    pub first_height: SeriesPattern6<Height>,
}

impl SeriesTree_Indexes_Hour4 {
    pub fn new(client: Arc<BrkClientBase>, base_path: String) -> Self {
        Self {
            first_height: SeriesPattern6::new(client.clone(), "first_height".to_string()),
        }
    }
}

/// Series tree node.
pub struct SeriesTree_Indexes_Hour12 {
    pub first_height: SeriesPattern7<Height>,
}

impl SeriesTree_Indexes_Hour12 {
    pub fn new(client: Arc<BrkClientBase>, base_path: String) -> Self {
        Self {
            first_height: SeriesPattern7::new(client.clone(), "first_height".to_string()),
        }
    }
}

/// Series tree node.
pub struct SeriesTree_Indexes_Day1 {
    pub date: SeriesPattern8<Date>,
    pub first_height: SeriesPattern8<Height>,
}

impl SeriesTree_Indexes_Day1 {
    pub fn new(client: Arc<BrkClientBase>, base_path: String) -> Self {
        Self {
            date: SeriesPattern8::new(client.clone(), "date".to_string()),
            first_height: SeriesPattern8::new(client.clone(), "first_height".to_string()),
        }
    }
}

/// Series tree node.
pub struct SeriesTree_Indexes_Day3 {
    pub date: SeriesPattern9<Date>,
    pub first_height: SeriesPattern9<Height>,
}

impl SeriesTree_Indexes_Day3 {
    pub fn new(client: Arc<BrkClientBase>, base_path: String) -> Self {
        Self {
            date: SeriesPattern9::new(client.clone(), "date".to_string()),
            first_height: SeriesPattern9::new(client.clone(), "first_height".to_string()),
        }
    }
}

/// Series tree node.
pub struct SeriesTree_Indexes_Week1 {
    pub date: SeriesPattern10<Date>,
    pub first_height: SeriesPattern10<Height>,
}

impl SeriesTree_Indexes_Week1 {
    pub fn new(client: Arc<BrkClientBase>, base_path: String) -> Self {
        Self {
            date: SeriesPattern10::new(client.clone(), "date".to_string()),
            first_height: SeriesPattern10::new(client.clone(), "first_height".to_string()),
        }
    }
}

/// Series tree node.
pub struct SeriesTree_Indexes_Month1 {
    pub date: SeriesPattern11<Date>,
    pub first_height: SeriesPattern11<Height>,
}

impl SeriesTree_Indexes_Month1 {
    pub fn new(client: Arc<BrkClientBase>, base_path: String) -> Self {
        Self {
            date: SeriesPattern11::new(client.clone(), "date".to_string()),
            first_height: SeriesPattern11::new(client.clone(), "first_height".to_string()),
        }
    }
}

/// Series tree node.
pub struct SeriesTree_Indexes_Month3 {
    pub date: SeriesPattern12<Date>,
    pub first_height: SeriesPattern12<Height>,
}

impl SeriesTree_Indexes_Month3 {
    pub fn new(client: Arc<BrkClientBase>, base_path: String) -> Self {
        Self {
            date: SeriesPattern12::new(client.clone(), "date".to_string()),
            first_height: SeriesPattern12::new(client.clone(), "first_height".to_string()),
        }
    }
}

/// Series tree node.
pub struct SeriesTree_Indexes_Month6 {
    pub date: SeriesPattern13<Date>,
    pub first_height: SeriesPattern13<Height>,
}

impl SeriesTree_Indexes_Month6 {
    pub fn new(client: Arc<BrkClientBase>, base_path: String) -> Self {
        Self {
            date: SeriesPattern13::new(client.clone(), "date".to_string()),
            first_height: SeriesPattern13::new(client.clone(), "first_height".to_string()),
        }
    }
}

/// Series tree node.
pub struct SeriesTree_Indexes_Year1 {
    pub date: SeriesPattern14<Date>,
    pub first_height: SeriesPattern14<Height>,
}

impl SeriesTree_Indexes_Year1 {
    pub fn new(client: Arc<BrkClientBase>, base_path: String) -> Self {
        Self {
            date: SeriesPattern14::new(client.clone(), "date".to_string()),
            first_height: SeriesPattern14::new(client.clone(), "first_height".to_string()),
        }
    }
}

/// Series tree node.
pub struct SeriesTree_Indexes_Year10 {
    pub date: SeriesPattern15<Date>,
    pub first_height: SeriesPattern15<Height>,
}

impl SeriesTree_Indexes_Year10 {
    pub fn new(client: Arc<BrkClientBase>, base_path: String) -> Self {
        Self {
            date: SeriesPattern15::new(client.clone(), "date".to_string()),
            first_height: SeriesPattern15::new(client.clone(), "first_height".to_string()),
        }
    }
}

/// Series tree node.
pub struct SeriesTree_Indexes_TxIndex {
    pub identity: SeriesPattern19<TxIndex>,
    pub input_count: SeriesPattern19<StoredU64>,
    pub output_count: SeriesPattern19<StoredU64>,
}

impl SeriesTree_Indexes_TxIndex {
    pub fn new(client: Arc<BrkClientBase>, base_path: String) -> Self {
        Self {
            identity: SeriesPattern19::new(client.clone(), "tx_index".to_string()),
            input_count: SeriesPattern19::new(client.clone(), "input_count".to_string()),
            output_count: SeriesPattern19::new(client.clone(), "output_count".to_string()),
        }
    }
}

/// Series tree node.
pub struct SeriesTree_Indexes_TxinIndex {
    pub identity: SeriesPattern20<TxInIndex>,
}

impl SeriesTree_Indexes_TxinIndex {
    pub fn new(client: Arc<BrkClientBase>, base_path: String) -> Self {
        Self {
            identity: SeriesPattern20::new(client.clone(), "txin_index".to_string()),
        }
    }
}

/// Series tree node.
pub struct SeriesTree_Indexes_TxoutIndex {
    pub identity: SeriesPattern21<TxOutIndex>,
}

impl SeriesTree_Indexes_TxoutIndex {
    pub fn new(client: Arc<BrkClientBase>, base_path: String) -> Self {
        Self {
            identity: SeriesPattern21::new(client.clone(), "txout_index".to_string()),
        }
    }
}

/// Series tree node.
pub struct SeriesTree_Indexes_Timestamp {
    pub monotonic: SeriesPattern18<Timestamp>,
    pub resolutions: SeriesPattern2<Timestamp>,
}

impl SeriesTree_Indexes_Timestamp {
    pub fn new(client: Arc<BrkClientBase>, base_path: String) -> Self {
        Self {
            monotonic: SeriesPattern18::new(client.clone(), "timestamp_monotonic".to_string()),
            resolutions: SeriesPattern2::new(client.clone(), "timestamp".to_string()),
        }
    }
}

/// Series tree node.
pub struct SeriesTree_Indicators {
    pub puell_multiple: BpsRatioPattern2,
    pub nvt: BpsRatioPattern2,
    pub gini: BpsPercentRatioPattern2,
    pub rhodl_ratio: BpsRatioPattern2,
    pub thermo_cap_multiple: BpsRatioPattern2,
    pub coindays_destroyed_supply_adj: SeriesPattern1<StoredF32>,
    pub coinyears_destroyed_supply_adj: SeriesPattern1<StoredF32>,
    pub dormancy: SeriesTree_Indicators_Dormancy,
    pub stock_to_flow: SeriesPattern1<StoredF32>,
    pub seller_exhaustion: SeriesPattern1<StoredF32>,
    pub rarity_meter: SeriesTree_Indicators_RarityMeter,
}

impl SeriesTree_Indicators {
    pub fn new(client: Arc<BrkClientBase>, base_path: String) -> Self {
        Self {
            puell_multiple: BpsRatioPattern2::new(client.clone(), "puell_multiple".to_string()),
            nvt: BpsRatioPattern2::new(client.clone(), "nvt".to_string()),
            gini: BpsPercentRatioPattern2::new(client.clone(), "gini".to_string()),
            rhodl_ratio: BpsRatioPattern2::new(client.clone(), "rhodl_ratio".to_string()),
            thermo_cap_multiple: BpsRatioPattern2::new(client.clone(), "thermo_cap_multiple".to_string()),
            coindays_destroyed_supply_adj: SeriesPattern1::new(client.clone(), "coindays_destroyed_supply_adj".to_string()),
            coinyears_destroyed_supply_adj: SeriesPattern1::new(client.clone(), "coinyears_destroyed_supply_adj".to_string()),
            dormancy: SeriesTree_Indicators_Dormancy::new(client.clone(), format!("{base_path}_dormancy")),
            stock_to_flow: SeriesPattern1::new(client.clone(), "stock_to_flow".to_string()),
            seller_exhaustion: SeriesPattern1::new(client.clone(), "seller_exhaustion".to_string()),
            rarity_meter: SeriesTree_Indicators_RarityMeter::new(client.clone(), format!("{base_path}_rarity_meter")),
        }
    }
}

/// Series tree node.
pub struct SeriesTree_Indicators_Dormancy {
    pub supply_adj: SeriesPattern1<StoredF32>,
    pub flow: SeriesPattern1<StoredF32>,
}

impl SeriesTree_Indicators_Dormancy {
    pub fn new(client: Arc<BrkClientBase>, base_path: String) -> Self {
        Self {
            supply_adj: SeriesPattern1::new(client.clone(), "dormancy_supply_adj".to_string()),
            flow: SeriesPattern1::new(client.clone(), "dormancy_flow".to_string()),
        }
    }
}

/// Series tree node.
pub struct SeriesTree_Indicators_RarityMeter {
    pub full: IndexPct0Pct1Pct2Pct5Pct95Pct98Pct99ScorePattern,
    pub local: IndexPct0Pct1Pct2Pct5Pct95Pct98Pct99ScorePattern,
    pub cycle: IndexPct0Pct1Pct2Pct5Pct95Pct98Pct99ScorePattern,
}

impl SeriesTree_Indicators_RarityMeter {
    pub fn new(client: Arc<BrkClientBase>, base_path: String) -> Self {
        Self {
            full: IndexPct0Pct1Pct2Pct5Pct95Pct98Pct99ScorePattern::new(client.clone(), "rarity_meter".to_string()),
            local: IndexPct0Pct1Pct2Pct5Pct95Pct98Pct99ScorePattern::new(client.clone(), "local_rarity_meter".to_string()),
            cycle: IndexPct0Pct1Pct2Pct5Pct95Pct98Pct99ScorePattern::new(client.clone(), "cycle_rarity_meter".to_string()),
        }
    }
}

/// Series tree node.
pub struct SeriesTree_Investing {
    pub sats_per_day: SeriesPattern18<Sats>,
    pub period: SeriesTree_Investing_Period,
    pub class: SeriesTree_Investing_Class,
}

impl SeriesTree_Investing {
    pub fn new(client: Arc<BrkClientBase>, base_path: String) -> Self {
        Self {
            sats_per_day: SeriesPattern18::new(client.clone(), "dca_sats_per_day".to_string()),
            period: SeriesTree_Investing_Period::new(client.clone(), format!("{base_path}_period")),
            class: SeriesTree_Investing_Class::new(client.clone(), format!("{base_path}_class")),
        }
    }
}

/// Series tree node.
pub struct SeriesTree_Investing_Period {
    pub dca_stack: _10y1m1w1y2y3m3y4y5y6m6y8yPattern3,
    pub dca_cost_basis: SeriesTree_Investing_Period_DcaCostBasis,
    pub dca_return: _10y1m1w1y2y3m3y4y5y6m6y8yPattern2,
    pub dca_cagr: _10y2y3y4y5y6y8yPattern,
    pub lump_sum_stack: _10y1m1w1y2y3m3y4y5y6m6y8yPattern3,
    pub lump_sum_return: _10y1m1w1y2y3m3y4y5y6m6y8yPattern2,
}

impl SeriesTree_Investing_Period {
    pub fn new(client: Arc<BrkClientBase>, base_path: String) -> Self {
        Self {
            dca_stack: _10y1m1w1y2y3m3y4y5y6m6y8yPattern3::new(client.clone(), "dca_stack".to_string()),
            dca_cost_basis: SeriesTree_Investing_Period_DcaCostBasis::new(client.clone(), format!("{base_path}_dca_cost_basis")),
            dca_return: _10y1m1w1y2y3m3y4y5y6m6y8yPattern2::new(client.clone(), "dca_return".to_string()),
            dca_cagr: _10y2y3y4y5y6y8yPattern::new(client.clone(), "dca_cagr".to_string()),
            lump_sum_stack: _10y1m1w1y2y3m3y4y5y6m6y8yPattern3::new(client.clone(), "lump_sum_stack".to_string()),
            lump_sum_return: _10y1m1w1y2y3m3y4y5y6m6y8yPattern2::new(client.clone(), "lump_sum_return".to_string()),
        }
    }
}

/// Series tree node.
pub struct SeriesTree_Investing_Period_DcaCostBasis {
    pub _1w: CentsSatsUsdPattern,
    pub _1m: CentsSatsUsdPattern,
    pub _3m: CentsSatsUsdPattern,
    pub _6m: CentsSatsUsdPattern,
    pub _1y: CentsSatsUsdPattern,
    pub _2y: CentsSatsUsdPattern,
    pub _3y: CentsSatsUsdPattern,
    pub _4y: CentsSatsUsdPattern,
    pub _5y: CentsSatsUsdPattern,
    pub _6y: CentsSatsUsdPattern,
    pub _8y: CentsSatsUsdPattern,
    pub _10y: CentsSatsUsdPattern,
}

impl SeriesTree_Investing_Period_DcaCostBasis {
    pub fn new(client: Arc<BrkClientBase>, base_path: String) -> Self {
        Self {
            _1w: CentsSatsUsdPattern::new(client.clone(), "dca_cost_basis_1w".to_string()),
            _1m: CentsSatsUsdPattern::new(client.clone(), "dca_cost_basis_1m".to_string()),
            _3m: CentsSatsUsdPattern::new(client.clone(), "dca_cost_basis_3m".to_string()),
            _6m: CentsSatsUsdPattern::new(client.clone(), "dca_cost_basis_6m".to_string()),
            _1y: CentsSatsUsdPattern::new(client.clone(), "dca_cost_basis_1y".to_string()),
            _2y: CentsSatsUsdPattern::new(client.clone(), "dca_cost_basis_2y".to_string()),
            _3y: CentsSatsUsdPattern::new(client.clone(), "dca_cost_basis_3y".to_string()),
            _4y: CentsSatsUsdPattern::new(client.clone(), "dca_cost_basis_4y".to_string()),
            _5y: CentsSatsUsdPattern::new(client.clone(), "dca_cost_basis_5y".to_string()),
            _6y: CentsSatsUsdPattern::new(client.clone(), "dca_cost_basis_6y".to_string()),
            _8y: CentsSatsUsdPattern::new(client.clone(), "dca_cost_basis_8y".to_string()),
            _10y: CentsSatsUsdPattern::new(client.clone(), "dca_cost_basis_10y".to_string()),
        }
    }
}

/// Series tree node.
pub struct SeriesTree_Investing_Class {
    pub dca_stack: SeriesTree_Investing_Class_DcaStack,
    pub dca_cost_basis: SeriesTree_Investing_Class_DcaCostBasis,
    pub dca_return: SeriesTree_Investing_Class_DcaReturn,
}

impl SeriesTree_Investing_Class {
    pub fn new(client: Arc<BrkClientBase>, base_path: String) -> Self {
        Self {
            dca_stack: SeriesTree_Investing_Class_DcaStack::new(client.clone(), format!("{base_path}_dca_stack")),
            dca_cost_basis: SeriesTree_Investing_Class_DcaCostBasis::new(client.clone(), format!("{base_path}_dca_cost_basis")),
            dca_return: SeriesTree_Investing_Class_DcaReturn::new(client.clone(), format!("{base_path}_dca_return")),
        }
    }
}

/// Series tree node.
pub struct SeriesTree_Investing_Class_DcaStack {
    pub from_2015: BtcCentsSatsUsdPattern,
    pub from_2016: BtcCentsSatsUsdPattern,
    pub from_2017: BtcCentsSatsUsdPattern,
    pub from_2018: BtcCentsSatsUsdPattern,
    pub from_2019: BtcCentsSatsUsdPattern,
    pub from_2020: BtcCentsSatsUsdPattern,
    pub from_2021: BtcCentsSatsUsdPattern,
    pub from_2022: BtcCentsSatsUsdPattern,
    pub from_2023: BtcCentsSatsUsdPattern,
    pub from_2024: BtcCentsSatsUsdPattern,
    pub from_2025: BtcCentsSatsUsdPattern,
    pub from_2026: BtcCentsSatsUsdPattern,
}

impl SeriesTree_Investing_Class_DcaStack {
    pub fn new(client: Arc<BrkClientBase>, base_path: String) -> Self {
        Self {
            from_2015: BtcCentsSatsUsdPattern::new(client.clone(), "dca_stack_from_2015".to_string()),
            from_2016: BtcCentsSatsUsdPattern::new(client.clone(), "dca_stack_from_2016".to_string()),
            from_2017: BtcCentsSatsUsdPattern::new(client.clone(), "dca_stack_from_2017".to_string()),
            from_2018: BtcCentsSatsUsdPattern::new(client.clone(), "dca_stack_from_2018".to_string()),
            from_2019: BtcCentsSatsUsdPattern::new(client.clone(), "dca_stack_from_2019".to_string()),
            from_2020: BtcCentsSatsUsdPattern::new(client.clone(), "dca_stack_from_2020".to_string()),
            from_2021: BtcCentsSatsUsdPattern::new(client.clone(), "dca_stack_from_2021".to_string()),
            from_2022: BtcCentsSatsUsdPattern::new(client.clone(), "dca_stack_from_2022".to_string()),
            from_2023: BtcCentsSatsUsdPattern::new(client.clone(), "dca_stack_from_2023".to_string()),
            from_2024: BtcCentsSatsUsdPattern::new(client.clone(), "dca_stack_from_2024".to_string()),
            from_2025: BtcCentsSatsUsdPattern::new(client.clone(), "dca_stack_from_2025".to_string()),
            from_2026: BtcCentsSatsUsdPattern::new(client.clone(), "dca_stack_from_2026".to_string()),
        }
    }
}

/// Series tree node.
pub struct SeriesTree_Investing_Class_DcaCostBasis {
    pub from_2015: CentsSatsUsdPattern,
    pub from_2016: CentsSatsUsdPattern,
    pub from_2017: CentsSatsUsdPattern,
    pub from_2018: CentsSatsUsdPattern,
    pub from_2019: CentsSatsUsdPattern,
    pub from_2020: CentsSatsUsdPattern,
    pub from_2021: CentsSatsUsdPattern,
    pub from_2022: CentsSatsUsdPattern,
    pub from_2023: CentsSatsUsdPattern,
    pub from_2024: CentsSatsUsdPattern,
    pub from_2025: CentsSatsUsdPattern,
    pub from_2026: CentsSatsUsdPattern,
}

impl SeriesTree_Investing_Class_DcaCostBasis {
    pub fn new(client: Arc<BrkClientBase>, base_path: String) -> Self {
        Self {
            from_2015: CentsSatsUsdPattern::new(client.clone(), "dca_cost_basis_from_2015".to_string()),
            from_2016: CentsSatsUsdPattern::new(client.clone(), "dca_cost_basis_from_2016".to_string()),
            from_2017: CentsSatsUsdPattern::new(client.clone(), "dca_cost_basis_from_2017".to_string()),
            from_2018: CentsSatsUsdPattern::new(client.clone(), "dca_cost_basis_from_2018".to_string()),
            from_2019: CentsSatsUsdPattern::new(client.clone(), "dca_cost_basis_from_2019".to_string()),
            from_2020: CentsSatsUsdPattern::new(client.clone(), "dca_cost_basis_from_2020".to_string()),
            from_2021: CentsSatsUsdPattern::new(client.clone(), "dca_cost_basis_from_2021".to_string()),
            from_2022: CentsSatsUsdPattern::new(client.clone(), "dca_cost_basis_from_2022".to_string()),
            from_2023: CentsSatsUsdPattern::new(client.clone(), "dca_cost_basis_from_2023".to_string()),
            from_2024: CentsSatsUsdPattern::new(client.clone(), "dca_cost_basis_from_2024".to_string()),
            from_2025: CentsSatsUsdPattern::new(client.clone(), "dca_cost_basis_from_2025".to_string()),
            from_2026: CentsSatsUsdPattern::new(client.clone(), "dca_cost_basis_from_2026".to_string()),
        }
    }
}

/// Series tree node.
pub struct SeriesTree_Investing_Class_DcaReturn {
    pub from_2015: BpsPercentRatioPattern,
    pub from_2016: BpsPercentRatioPattern,
    pub from_2017: BpsPercentRatioPattern,
    pub from_2018: BpsPercentRatioPattern,
    pub from_2019: BpsPercentRatioPattern,
    pub from_2020: BpsPercentRatioPattern,
    pub from_2021: BpsPercentRatioPattern,
    pub from_2022: BpsPercentRatioPattern,
    pub from_2023: BpsPercentRatioPattern,
    pub from_2024: BpsPercentRatioPattern,
    pub from_2025: BpsPercentRatioPattern,
    pub from_2026: BpsPercentRatioPattern,
}

impl SeriesTree_Investing_Class_DcaReturn {
    pub fn new(client: Arc<BrkClientBase>, base_path: String) -> Self {
        Self {
            from_2015: BpsPercentRatioPattern::new(client.clone(), "dca_return_from_2015".to_string()),
            from_2016: BpsPercentRatioPattern::new(client.clone(), "dca_return_from_2016".to_string()),
            from_2017: BpsPercentRatioPattern::new(client.clone(), "dca_return_from_2017".to_string()),
            from_2018: BpsPercentRatioPattern::new(client.clone(), "dca_return_from_2018".to_string()),
            from_2019: BpsPercentRatioPattern::new(client.clone(), "dca_return_from_2019".to_string()),
            from_2020: BpsPercentRatioPattern::new(client.clone(), "dca_return_from_2020".to_string()),
            from_2021: BpsPercentRatioPattern::new(client.clone(), "dca_return_from_2021".to_string()),
            from_2022: BpsPercentRatioPattern::new(client.clone(), "dca_return_from_2022".to_string()),
            from_2023: BpsPercentRatioPattern::new(client.clone(), "dca_return_from_2023".to_string()),
            from_2024: BpsPercentRatioPattern::new(client.clone(), "dca_return_from_2024".to_string()),
            from_2025: BpsPercentRatioPattern::new(client.clone(), "dca_return_from_2025".to_string()),
            from_2026: BpsPercentRatioPattern::new(client.clone(), "dca_return_from_2026".to_string()),
        }
    }
}

/// Series tree node.
pub struct SeriesTree_Market {
    pub ath: SeriesTree_Market_Ath,
    pub lookback: SeriesTree_Market_Lookback,
    pub returns: SeriesTree_Market_Returns,
    pub volatility: _1m1w1y24hPattern<StoredF32>,
    pub range: SeriesTree_Market_Range,
    pub moving_average: SeriesTree_Market_MovingAverage,
    pub technical: SeriesTree_Market_Technical,
}

impl SeriesTree_Market {
    pub fn new(client: Arc<BrkClientBase>, base_path: String) -> Self {
        Self {
            ath: SeriesTree_Market_Ath::new(client.clone(), format!("{base_path}_ath")),
            lookback: SeriesTree_Market_Lookback::new(client.clone(), format!("{base_path}_lookback")),
            returns: SeriesTree_Market_Returns::new(client.clone(), format!("{base_path}_returns")),
            volatility: _1m1w1y24hPattern::new(client.clone(), "price_volatility".to_string()),
            range: SeriesTree_Market_Range::new(client.clone(), format!("{base_path}_range")),
            moving_average: SeriesTree_Market_MovingAverage::new(client.clone(), format!("{base_path}_moving_average")),
            technical: SeriesTree_Market_Technical::new(client.clone(), format!("{base_path}_technical")),
        }
    }
}

/// Series tree node.
pub struct SeriesTree_Market_Ath {
    pub high: CentsSatsUsdPattern,
    pub drawdown: BpsPercentRatioPattern5,
    pub days_since: SeriesPattern1<StoredF32>,
    pub years_since: SeriesPattern1<StoredF32>,
    pub max_days_between: SeriesPattern1<StoredF32>,
    pub max_years_between: SeriesPattern1<StoredF32>,
}

impl SeriesTree_Market_Ath {
    pub fn new(client: Arc<BrkClientBase>, base_path: String) -> Self {
        Self {
            high: CentsSatsUsdPattern::new(client.clone(), "price_ath".to_string()),
            drawdown: BpsPercentRatioPattern5::new(client.clone(), "price_drawdown".to_string()),
            days_since: SeriesPattern1::new(client.clone(), "days_since_price_ath".to_string()),
            years_since: SeriesPattern1::new(client.clone(), "years_since_price_ath".to_string()),
            max_days_between: SeriesPattern1::new(client.clone(), "max_days_between_price_ath".to_string()),
            max_years_between: SeriesPattern1::new(client.clone(), "max_years_between_price_ath".to_string()),
        }
    }
}

/// Series tree node.
pub struct SeriesTree_Market_Lookback {
    pub _24h: CentsSatsUsdPattern,
    pub _1w: CentsSatsUsdPattern,
    pub _1m: CentsSatsUsdPattern,
    pub _3m: CentsSatsUsdPattern,
    pub _6m: CentsSatsUsdPattern,
    pub _1y: CentsSatsUsdPattern,
    pub _2y: CentsSatsUsdPattern,
    pub _3y: CentsSatsUsdPattern,
    pub _4y: CentsSatsUsdPattern,
    pub _5y: CentsSatsUsdPattern,
    pub _6y: CentsSatsUsdPattern,
    pub _8y: CentsSatsUsdPattern,
    pub _10y: CentsSatsUsdPattern,
}

impl SeriesTree_Market_Lookback {
    pub fn new(client: Arc<BrkClientBase>, base_path: String) -> Self {
        Self {
            _24h: CentsSatsUsdPattern::new(client.clone(), "price_past_24h".to_string()),
            _1w: CentsSatsUsdPattern::new(client.clone(), "price_past_1w".to_string()),
            _1m: CentsSatsUsdPattern::new(client.clone(), "price_past_1m".to_string()),
            _3m: CentsSatsUsdPattern::new(client.clone(), "price_past_3m".to_string()),
            _6m: CentsSatsUsdPattern::new(client.clone(), "price_past_6m".to_string()),
            _1y: CentsSatsUsdPattern::new(client.clone(), "price_past_1y".to_string()),
            _2y: CentsSatsUsdPattern::new(client.clone(), "price_past_2y".to_string()),
            _3y: CentsSatsUsdPattern::new(client.clone(), "price_past_3y".to_string()),
            _4y: CentsSatsUsdPattern::new(client.clone(), "price_past_4y".to_string()),
            _5y: CentsSatsUsdPattern::new(client.clone(), "price_past_5y".to_string()),
            _6y: CentsSatsUsdPattern::new(client.clone(), "price_past_6y".to_string()),
            _8y: CentsSatsUsdPattern::new(client.clone(), "price_past_8y".to_string()),
            _10y: CentsSatsUsdPattern::new(client.clone(), "price_past_10y".to_string()),
        }
    }
}

/// Series tree node.
pub struct SeriesTree_Market_Returns {
    pub periods: SeriesTree_Market_Returns_Periods,
    pub cagr: _10y2y3y4y5y6y8yPattern,
    pub sd_24h: SeriesTree_Market_Returns_Sd24h,
}

impl SeriesTree_Market_Returns {
    pub fn new(client: Arc<BrkClientBase>, base_path: String) -> Self {
        Self {
            periods: SeriesTree_Market_Returns_Periods::new(client.clone(), format!("{base_path}_periods")),
            cagr: _10y2y3y4y5y6y8yPattern::new(client.clone(), "price_cagr".to_string()),
            sd_24h: SeriesTree_Market_Returns_Sd24h::new(client.clone(), format!("{base_path}_sd_24h")),
        }
    }
}

/// Series tree node.
pub struct SeriesTree_Market_Returns_Periods {
    pub _24h: BpsPercentRatioPattern,
    pub _1w: BpsPercentRatioPattern,
    pub _1m: BpsPercentRatioPattern,
    pub _3m: BpsPercentRatioPattern,
    pub _6m: BpsPercentRatioPattern,
    pub _1y: BpsPercentRatioPattern,
    pub _2y: BpsPercentRatioPattern,
    pub _3y: BpsPercentRatioPattern,
    pub _4y: BpsPercentRatioPattern,
    pub _5y: BpsPercentRatioPattern,
    pub _6y: BpsPercentRatioPattern,
    pub _8y: BpsPercentRatioPattern,
    pub _10y: BpsPercentRatioPattern,
}

impl SeriesTree_Market_Returns_Periods {
    pub fn new(client: Arc<BrkClientBase>, base_path: String) -> Self {
        Self {
            _24h: BpsPercentRatioPattern::new(client.clone(), "price_return_24h".to_string()),
            _1w: BpsPercentRatioPattern::new(client.clone(), "price_return_1w".to_string()),
            _1m: BpsPercentRatioPattern::new(client.clone(), "price_return_1m".to_string()),
            _3m: BpsPercentRatioPattern::new(client.clone(), "price_return_3m".to_string()),
            _6m: BpsPercentRatioPattern::new(client.clone(), "price_return_6m".to_string()),
            _1y: BpsPercentRatioPattern::new(client.clone(), "price_return_1y".to_string()),
            _2y: BpsPercentRatioPattern::new(client.clone(), "price_return_2y".to_string()),
            _3y: BpsPercentRatioPattern::new(client.clone(), "price_return_3y".to_string()),
            _4y: BpsPercentRatioPattern::new(client.clone(), "price_return_4y".to_string()),
            _5y: BpsPercentRatioPattern::new(client.clone(), "price_return_5y".to_string()),
            _6y: BpsPercentRatioPattern::new(client.clone(), "price_return_6y".to_string()),
            _8y: BpsPercentRatioPattern::new(client.clone(), "price_return_8y".to_string()),
            _10y: BpsPercentRatioPattern::new(client.clone(), "price_return_10y".to_string()),
        }
    }
}

/// Series tree node.
pub struct SeriesTree_Market_Returns_Sd24h {
    pub _24h: SeriesTree_Market_Returns_Sd24h_24h,
    pub _1w: SeriesTree_Market_Returns_Sd24h_1w,
    pub _1m: SeriesTree_Market_Returns_Sd24h_1m,
    pub _1y: SeriesTree_Market_Returns_Sd24h_1y,
}

impl SeriesTree_Market_Returns_Sd24h {
    pub fn new(client: Arc<BrkClientBase>, base_path: String) -> Self {
        Self {
            _24h: SeriesTree_Market_Returns_Sd24h_24h::new(client.clone(), format!("{base_path}_24h")),
            _1w: SeriesTree_Market_Returns_Sd24h_1w::new(client.clone(), format!("{base_path}_1w")),
            _1m: SeriesTree_Market_Returns_Sd24h_1m::new(client.clone(), format!("{base_path}_1m")),
            _1y: SeriesTree_Market_Returns_Sd24h_1y::new(client.clone(), format!("{base_path}_1y")),
        }
    }
}

/// Series tree node.
pub struct SeriesTree_Market_Returns_Sd24h_24h {
    pub sma: SeriesPattern1<StoredF32>,
    pub sd: SeriesPattern1<StoredF32>,
}

impl SeriesTree_Market_Returns_Sd24h_24h {
    pub fn new(client: Arc<BrkClientBase>, base_path: String) -> Self {
        Self {
            sma: SeriesPattern1::new(client.clone(), "price_return_24h_sma_24h".to_string()),
            sd: SeriesPattern1::new(client.clone(), "price_return_24h_sd_24h".to_string()),
        }
    }
}

/// Series tree node.
pub struct SeriesTree_Market_Returns_Sd24h_1w {
    pub sma: SeriesPattern1<StoredF32>,
    pub sd: SeriesPattern1<StoredF32>,
}

impl SeriesTree_Market_Returns_Sd24h_1w {
    pub fn new(client: Arc<BrkClientBase>, base_path: String) -> Self {
        Self {
            sma: SeriesPattern1::new(client.clone(), "price_return_24h_sma_1w".to_string()),
            sd: SeriesPattern1::new(client.clone(), "price_return_24h_sd_1w".to_string()),
        }
    }
}

/// Series tree node.
pub struct SeriesTree_Market_Returns_Sd24h_1m {
    pub sma: SeriesPattern1<StoredF32>,
    pub sd: SeriesPattern1<StoredF32>,
}

impl SeriesTree_Market_Returns_Sd24h_1m {
    pub fn new(client: Arc<BrkClientBase>, base_path: String) -> Self {
        Self {
            sma: SeriesPattern1::new(client.clone(), "price_return_24h_sma_1m".to_string()),
            sd: SeriesPattern1::new(client.clone(), "price_return_24h_sd_1m".to_string()),
        }
    }
}

/// Series tree node.
pub struct SeriesTree_Market_Returns_Sd24h_1y {
    pub sma: SeriesPattern1<StoredF32>,
    pub sd: SeriesPattern1<StoredF32>,
}

impl SeriesTree_Market_Returns_Sd24h_1y {
    pub fn new(client: Arc<BrkClientBase>, base_path: String) -> Self {
        Self {
            sma: SeriesPattern1::new(client.clone(), "price_return_24h_sma_1y".to_string()),
            sd: SeriesPattern1::new(client.clone(), "price_return_24h_sd_1y".to_string()),
        }
    }
}

/// Series tree node.
pub struct SeriesTree_Market_Range {
    pub min: _1m1w1y2wPattern,
    pub max: _1m1w1y2wPattern,
    pub true_range: SeriesPattern1<StoredF32>,
    pub true_range_sum_2w: SeriesPattern1<StoredF32>,
    pub choppiness_index_2w: BpsPercentRatioPattern2,
}

impl SeriesTree_Market_Range {
    pub fn new(client: Arc<BrkClientBase>, base_path: String) -> Self {
        Self {
            min: _1m1w1y2wPattern::new(client.clone(), "price_min".to_string()),
            max: _1m1w1y2wPattern::new(client.clone(), "price_max".to_string()),
            true_range: SeriesPattern1::new(client.clone(), "price_true_range".to_string()),
            true_range_sum_2w: SeriesPattern1::new(client.clone(), "price_true_range_sum_2w".to_string()),
            choppiness_index_2w: BpsPercentRatioPattern2::new(client.clone(), "price_choppiness_index_2w".to_string()),
        }
    }
}

/// Series tree node.
pub struct SeriesTree_Market_MovingAverage {
    pub sma: SeriesTree_Market_MovingAverage_Sma,
    pub ema: SeriesTree_Market_MovingAverage_Ema,
}

impl SeriesTree_Market_MovingAverage {
    pub fn new(client: Arc<BrkClientBase>, base_path: String) -> Self {
        Self {
            sma: SeriesTree_Market_MovingAverage_Sma::new(client.clone(), format!("{base_path}_sma")),
            ema: SeriesTree_Market_MovingAverage_Ema::new(client.clone(), format!("{base_path}_ema")),
        }
    }
}

/// Series tree node.
pub struct SeriesTree_Market_MovingAverage_Sma {
    pub _1w: BpsCentsRatioSatsUsdPattern,
    pub _8d: BpsCentsRatioSatsUsdPattern,
    pub _13d: BpsCentsRatioSatsUsdPattern,
    pub _21d: BpsCentsRatioSatsUsdPattern,
    pub _1m: BpsCentsRatioSatsUsdPattern,
    pub _34d: BpsCentsRatioSatsUsdPattern,
    pub _55d: BpsCentsRatioSatsUsdPattern,
    pub _89d: BpsCentsRatioSatsUsdPattern,
    pub _111d: BpsCentsRatioSatsUsdPattern,
    pub _144d: BpsCentsRatioSatsUsdPattern,
    pub _200d: SeriesTree_Market_MovingAverage_Sma_200d,
    pub _350d: SeriesTree_Market_MovingAverage_Sma_350d,
    pub _1y: BpsCentsRatioSatsUsdPattern,
    pub _2y: BpsCentsRatioSatsUsdPattern,
    pub _200w: BpsCentsRatioSatsUsdPattern,
    pub _4y: BpsCentsRatioSatsUsdPattern,
}

impl SeriesTree_Market_MovingAverage_Sma {
    pub fn new(client: Arc<BrkClientBase>, base_path: String) -> Self {
        Self {
            _1w: BpsCentsRatioSatsUsdPattern::new(client.clone(), "price_sma_1w".to_string()),
            _8d: BpsCentsRatioSatsUsdPattern::new(client.clone(), "price_sma_8d".to_string()),
            _13d: BpsCentsRatioSatsUsdPattern::new(client.clone(), "price_sma_13d".to_string()),
            _21d: BpsCentsRatioSatsUsdPattern::new(client.clone(), "price_sma_21d".to_string()),
            _1m: BpsCentsRatioSatsUsdPattern::new(client.clone(), "price_sma_1m".to_string()),
            _34d: BpsCentsRatioSatsUsdPattern::new(client.clone(), "price_sma_34d".to_string()),
            _55d: BpsCentsRatioSatsUsdPattern::new(client.clone(), "price_sma_55d".to_string()),
            _89d: BpsCentsRatioSatsUsdPattern::new(client.clone(), "price_sma_89d".to_string()),
            _111d: BpsCentsRatioSatsUsdPattern::new(client.clone(), "price_sma_111d".to_string()),
            _144d: BpsCentsRatioSatsUsdPattern::new(client.clone(), "price_sma_144d".to_string()),
            _200d: SeriesTree_Market_MovingAverage_Sma_200d::new(client.clone(), format!("{base_path}_200d")),
            _350d: SeriesTree_Market_MovingAverage_Sma_350d::new(client.clone(), format!("{base_path}_350d")),
            _1y: BpsCentsRatioSatsUsdPattern::new(client.clone(), "price_sma_1y".to_string()),
            _2y: BpsCentsRatioSatsUsdPattern::new(client.clone(), "price_sma_2y".to_string()),
            _200w: BpsCentsRatioSatsUsdPattern::new(client.clone(), "price_sma_200w".to_string()),
            _4y: BpsCentsRatioSatsUsdPattern::new(client.clone(), "price_sma_4y".to_string()),
        }
    }
}

/// Series tree node.
pub struct SeriesTree_Market_MovingAverage_Sma_200d {
    pub usd: SeriesPattern1<Dollars>,
    pub cents: SeriesPattern1<Cents>,
    pub sats: SeriesPattern1<SatsFract>,
    pub bps: SeriesPattern1<BasisPoints32>,
    pub ratio: SeriesPattern1<StoredF32>,
    pub x2_4: CentsSatsUsdPattern,
    pub x0_8: CentsSatsUsdPattern,
}

impl SeriesTree_Market_MovingAverage_Sma_200d {
    pub fn new(client: Arc<BrkClientBase>, base_path: String) -> Self {
        Self {
            usd: SeriesPattern1::new(client.clone(), "price_sma_200d".to_string()),
            cents: SeriesPattern1::new(client.clone(), "price_sma_200d_cents".to_string()),
            sats: SeriesPattern1::new(client.clone(), "price_sma_200d_sats".to_string()),
            bps: SeriesPattern1::new(client.clone(), "price_sma_200d_ratio_bps".to_string()),
            ratio: SeriesPattern1::new(client.clone(), "price_sma_200d_ratio".to_string()),
            x2_4: CentsSatsUsdPattern::new(client.clone(), "price_sma_200d_x2_4".to_string()),
            x0_8: CentsSatsUsdPattern::new(client.clone(), "price_sma_200d_x0_8".to_string()),
        }
    }
}

/// Series tree node.
pub struct SeriesTree_Market_MovingAverage_Sma_350d {
    pub usd: SeriesPattern1<Dollars>,
    pub cents: SeriesPattern1<Cents>,
    pub sats: SeriesPattern1<SatsFract>,
    pub bps: SeriesPattern1<BasisPoints32>,
    pub ratio: SeriesPattern1<StoredF32>,
    pub x2: CentsSatsUsdPattern,
}

impl SeriesTree_Market_MovingAverage_Sma_350d {
    pub fn new(client: Arc<BrkClientBase>, base_path: String) -> Self {
        Self {
            usd: SeriesPattern1::new(client.clone(), "price_sma_350d".to_string()),
            cents: SeriesPattern1::new(client.clone(), "price_sma_350d_cents".to_string()),
            sats: SeriesPattern1::new(client.clone(), "price_sma_350d_sats".to_string()),
            bps: SeriesPattern1::new(client.clone(), "price_sma_350d_ratio_bps".to_string()),
            ratio: SeriesPattern1::new(client.clone(), "price_sma_350d_ratio".to_string()),
            x2: CentsSatsUsdPattern::new(client.clone(), "price_sma_350d_x2".to_string()),
        }
    }
}

/// Series tree node.
pub struct SeriesTree_Market_MovingAverage_Ema {
    pub _1w: BpsCentsRatioSatsUsdPattern,
    pub _8d: BpsCentsRatioSatsUsdPattern,
    pub _12d: BpsCentsRatioSatsUsdPattern,
    pub _13d: BpsCentsRatioSatsUsdPattern,
    pub _21d: BpsCentsRatioSatsUsdPattern,
    pub _26d: BpsCentsRatioSatsUsdPattern,
    pub _1m: BpsCentsRatioSatsUsdPattern,
    pub _34d: BpsCentsRatioSatsUsdPattern,
    pub _55d: BpsCentsRatioSatsUsdPattern,
    pub _89d: BpsCentsRatioSatsUsdPattern,
    pub _144d: BpsCentsRatioSatsUsdPattern,
    pub _200d: BpsCentsRatioSatsUsdPattern,
    pub _1y: BpsCentsRatioSatsUsdPattern,
    pub _2y: BpsCentsRatioSatsUsdPattern,
    pub _200w: BpsCentsRatioSatsUsdPattern,
    pub _4y: BpsCentsRatioSatsUsdPattern,
}

impl SeriesTree_Market_MovingAverage_Ema {
    pub fn new(client: Arc<BrkClientBase>, base_path: String) -> Self {
        Self {
            _1w: BpsCentsRatioSatsUsdPattern::new(client.clone(), "price_ema_1w".to_string()),
            _8d: BpsCentsRatioSatsUsdPattern::new(client.clone(), "price_ema_8d".to_string()),
            _12d: BpsCentsRatioSatsUsdPattern::new(client.clone(), "price_ema_12d".to_string()),
            _13d: BpsCentsRatioSatsUsdPattern::new(client.clone(), "price_ema_13d".to_string()),
            _21d: BpsCentsRatioSatsUsdPattern::new(client.clone(), "price_ema_21d".to_string()),
            _26d: BpsCentsRatioSatsUsdPattern::new(client.clone(), "price_ema_26d".to_string()),
            _1m: BpsCentsRatioSatsUsdPattern::new(client.clone(), "price_ema_1m".to_string()),
            _34d: BpsCentsRatioSatsUsdPattern::new(client.clone(), "price_ema_34d".to_string()),
            _55d: BpsCentsRatioSatsUsdPattern::new(client.clone(), "price_ema_55d".to_string()),
            _89d: BpsCentsRatioSatsUsdPattern::new(client.clone(), "price_ema_89d".to_string()),
            _144d: BpsCentsRatioSatsUsdPattern::new(client.clone(), "price_ema_144d".to_string()),
            _200d: BpsCentsRatioSatsUsdPattern::new(client.clone(), "price_ema_200d".to_string()),
            _1y: BpsCentsRatioSatsUsdPattern::new(client.clone(), "price_ema_1y".to_string()),
            _2y: BpsCentsRatioSatsUsdPattern::new(client.clone(), "price_ema_2y".to_string()),
            _200w: BpsCentsRatioSatsUsdPattern::new(client.clone(), "price_ema_200w".to_string()),
            _4y: BpsCentsRatioSatsUsdPattern::new(client.clone(), "price_ema_4y".to_string()),
        }
    }
}

/// Series tree node.
pub struct SeriesTree_Market_Technical {
    pub rsi: SeriesTree_Market_Technical_Rsi,
    pub pi_cycle: BpsRatioPattern2,
    pub macd: SeriesTree_Market_Technical_Macd,
}

impl SeriesTree_Market_Technical {
    pub fn new(client: Arc<BrkClientBase>, base_path: String) -> Self {
        Self {
            rsi: SeriesTree_Market_Technical_Rsi::new(client.clone(), format!("{base_path}_rsi")),
            pi_cycle: BpsRatioPattern2::new(client.clone(), "pi_cycle".to_string()),
            macd: SeriesTree_Market_Technical_Macd::new(client.clone(), format!("{base_path}_macd")),
        }
    }
}

/// Series tree node.
pub struct SeriesTree_Market_Technical_Rsi {
    pub _24h: RsiStochPattern,
    pub _1w: RsiStochPattern,
    pub _1m: RsiStochPattern,
}

impl SeriesTree_Market_Technical_Rsi {
    pub fn new(client: Arc<BrkClientBase>, base_path: String) -> Self {
        Self {
            _24h: RsiStochPattern::new(client.clone(), "rsi".to_string(), "24h".to_string()),
            _1w: RsiStochPattern::new(client.clone(), "rsi".to_string(), "1w".to_string()),
            _1m: RsiStochPattern::new(client.clone(), "rsi".to_string(), "1m".to_string()),
        }
    }
}

/// Series tree node.
pub struct SeriesTree_Market_Technical_Macd {
    pub _24h: SeriesTree_Market_Technical_Macd_24h,
    pub _1w: SeriesTree_Market_Technical_Macd_1w,
    pub _1m: SeriesTree_Market_Technical_Macd_1m,
}

impl SeriesTree_Market_Technical_Macd {
    pub fn new(client: Arc<BrkClientBase>, base_path: String) -> Self {
        Self {
            _24h: SeriesTree_Market_Technical_Macd_24h::new(client.clone(), format!("{base_path}_24h")),
            _1w: SeriesTree_Market_Technical_Macd_1w::new(client.clone(), format!("{base_path}_1w")),
            _1m: SeriesTree_Market_Technical_Macd_1m::new(client.clone(), format!("{base_path}_1m")),
        }
    }
}

/// Series tree node.
pub struct SeriesTree_Market_Technical_Macd_24h {
    pub ema_fast: SeriesPattern1<StoredF32>,
    pub ema_slow: SeriesPattern1<StoredF32>,
    pub line: SeriesPattern1<StoredF32>,
    pub signal: SeriesPattern1<StoredF32>,
    pub histogram: SeriesPattern1<StoredF32>,
}

impl SeriesTree_Market_Technical_Macd_24h {
    pub fn new(client: Arc<BrkClientBase>, base_path: String) -> Self {
        Self {
            ema_fast: SeriesPattern1::new(client.clone(), "macd_ema_fast_24h".to_string()),
            ema_slow: SeriesPattern1::new(client.clone(), "macd_ema_slow_24h".to_string()),
            line: SeriesPattern1::new(client.clone(), "macd_line_24h".to_string()),
            signal: SeriesPattern1::new(client.clone(), "macd_signal_24h".to_string()),
            histogram: SeriesPattern1::new(client.clone(), "macd_histogram_24h".to_string()),
        }
    }
}

/// Series tree node.
pub struct SeriesTree_Market_Technical_Macd_1w {
    pub ema_fast: SeriesPattern1<StoredF32>,
    pub ema_slow: SeriesPattern1<StoredF32>,
    pub line: SeriesPattern1<StoredF32>,
    pub signal: SeriesPattern1<StoredF32>,
    pub histogram: SeriesPattern1<StoredF32>,
}

impl SeriesTree_Market_Technical_Macd_1w {
    pub fn new(client: Arc<BrkClientBase>, base_path: String) -> Self {
        Self {
            ema_fast: SeriesPattern1::new(client.clone(), "macd_ema_fast_1w".to_string()),
            ema_slow: SeriesPattern1::new(client.clone(), "macd_ema_slow_1w".to_string()),
            line: SeriesPattern1::new(client.clone(), "macd_line_1w".to_string()),
            signal: SeriesPattern1::new(client.clone(), "macd_signal_1w".to_string()),
            histogram: SeriesPattern1::new(client.clone(), "macd_histogram_1w".to_string()),
        }
    }
}

/// Series tree node.
pub struct SeriesTree_Market_Technical_Macd_1m {
    pub ema_fast: SeriesPattern1<StoredF32>,
    pub ema_slow: SeriesPattern1<StoredF32>,
    pub line: SeriesPattern1<StoredF32>,
    pub signal: SeriesPattern1<StoredF32>,
    pub histogram: SeriesPattern1<StoredF32>,
}

impl SeriesTree_Market_Technical_Macd_1m {
    pub fn new(client: Arc<BrkClientBase>, base_path: String) -> Self {
        Self {
            ema_fast: SeriesPattern1::new(client.clone(), "macd_ema_fast_1m".to_string()),
            ema_slow: SeriesPattern1::new(client.clone(), "macd_ema_slow_1m".to_string()),
            line: SeriesPattern1::new(client.clone(), "macd_line_1m".to_string()),
            signal: SeriesPattern1::new(client.clone(), "macd_signal_1m".to_string()),
            histogram: SeriesPattern1::new(client.clone(), "macd_histogram_1m".to_string()),
        }
    }
}

/// Series tree node.
pub struct SeriesTree_Pools {
    pub pool: SeriesPattern18<PoolSlug>,
    pub major: SeriesTree_Pools_Major,
    pub minor: SeriesTree_Pools_Minor,
}

impl SeriesTree_Pools {
    pub fn new(client: Arc<BrkClientBase>, base_path: String) -> Self {
        Self {
            pool: SeriesPattern18::new(client.clone(), "pool".to_string()),
            major: SeriesTree_Pools_Major::new(client.clone(), format!("{base_path}_major")),
            minor: SeriesTree_Pools_Minor::new(client.clone(), format!("{base_path}_minor")),
        }
    }
}

/// Series tree node.
pub struct SeriesTree_Pools_Major {
    pub unknown: BlocksDominanceRewardsPattern,
    pub luxor: BlocksDominanceRewardsPattern,
    pub btccom: BlocksDominanceRewardsPattern,
    pub btctop: BlocksDominanceRewardsPattern,
    pub btcguild: BlocksDominanceRewardsPattern,
    pub eligius: BlocksDominanceRewardsPattern,
    pub f2pool: BlocksDominanceRewardsPattern,
    pub braiinspool: BlocksDominanceRewardsPattern,
    pub antpool: BlocksDominanceRewardsPattern,
    pub btcc: BlocksDominanceRewardsPattern,
    pub bwpool: BlocksDominanceRewardsPattern,
    pub bitfury: BlocksDominanceRewardsPattern,
    pub viabtc: BlocksDominanceRewardsPattern,
    pub poolin: BlocksDominanceRewardsPattern,
    pub spiderpool: BlocksDominanceRewardsPattern,
    pub binancepool: BlocksDominanceRewardsPattern,
    pub foundryusa: BlocksDominanceRewardsPattern,
    pub sbicrypto: BlocksDominanceRewardsPattern,
    pub marapool: BlocksDominanceRewardsPattern,
    pub secpool: BlocksDominanceRewardsPattern,
    pub ocean: BlocksDominanceRewardsPattern,
    pub whitepool: BlocksDominanceRewardsPattern,
}

impl SeriesTree_Pools_Major {
    pub fn new(client: Arc<BrkClientBase>, base_path: String) -> Self {
        Self {
            unknown: BlocksDominanceRewardsPattern::new(client.clone(), "unknown".to_string()),
            luxor: BlocksDominanceRewardsPattern::new(client.clone(), "luxor".to_string()),
            btccom: BlocksDominanceRewardsPattern::new(client.clone(), "btccom".to_string()),
            btctop: BlocksDominanceRewardsPattern::new(client.clone(), "btctop".to_string()),
            btcguild: BlocksDominanceRewardsPattern::new(client.clone(), "btcguild".to_string()),
            eligius: BlocksDominanceRewardsPattern::new(client.clone(), "eligius".to_string()),
            f2pool: BlocksDominanceRewardsPattern::new(client.clone(), "f2pool".to_string()),
            braiinspool: BlocksDominanceRewardsPattern::new(client.clone(), "braiinspool".to_string()),
            antpool: BlocksDominanceRewardsPattern::new(client.clone(), "antpool".to_string()),
            btcc: BlocksDominanceRewardsPattern::new(client.clone(), "btcc".to_string()),
            bwpool: BlocksDominanceRewardsPattern::new(client.clone(), "bwpool".to_string()),
            bitfury: BlocksDominanceRewardsPattern::new(client.clone(), "bitfury".to_string()),
            viabtc: BlocksDominanceRewardsPattern::new(client.clone(), "viabtc".to_string()),
            poolin: BlocksDominanceRewardsPattern::new(client.clone(), "poolin".to_string()),
            spiderpool: BlocksDominanceRewardsPattern::new(client.clone(), "spiderpool".to_string()),
            binancepool: BlocksDominanceRewardsPattern::new(client.clone(), "binancepool".to_string()),
            foundryusa: BlocksDominanceRewardsPattern::new(client.clone(), "foundryusa".to_string()),
            sbicrypto: BlocksDominanceRewardsPattern::new(client.clone(), "sbicrypto".to_string()),
            marapool: BlocksDominanceRewardsPattern::new(client.clone(), "marapool".to_string()),
            secpool: BlocksDominanceRewardsPattern::new(client.clone(), "secpool".to_string()),
            ocean: BlocksDominanceRewardsPattern::new(client.clone(), "ocean".to_string()),
            whitepool: BlocksDominanceRewardsPattern::new(client.clone(), "whitepool".to_string()),
        }
    }
}

/// Series tree node.
pub struct SeriesTree_Pools_Minor {
    pub blockfills: BlocksDominancePattern,
    pub ultimuspool: BlocksDominancePattern,
    pub terrapool: BlocksDominancePattern,
    pub onethash: BlocksDominancePattern,
    pub bitfarms: BlocksDominancePattern,
    pub huobipool: BlocksDominancePattern,
    pub wayicn: BlocksDominancePattern,
    pub canoepool: BlocksDominancePattern,
    pub bitcoincom: BlocksDominancePattern,
    pub pool175btc: BlocksDominancePattern,
    pub gbminers: BlocksDominancePattern,
    pub axbt: BlocksDominancePattern,
    pub asicminer: BlocksDominancePattern,
    pub bitminter: BlocksDominancePattern,
    pub bitcoinrussia: BlocksDominancePattern,
    pub btcserv: BlocksDominancePattern,
    pub simplecoinus: BlocksDominancePattern,
    pub ozcoin: BlocksDominancePattern,
    pub eclipsemc: BlocksDominancePattern,
    pub maxbtc: BlocksDominancePattern,
    pub triplemining: BlocksDominancePattern,
    pub coinlab: BlocksDominancePattern,
    pub pool50btc: BlocksDominancePattern,
    pub ghashio: BlocksDominancePattern,
    pub stminingcorp: BlocksDominancePattern,
    pub bitparking: BlocksDominancePattern,
    pub mmpool: BlocksDominancePattern,
    pub polmine: BlocksDominancePattern,
    pub kncminer: BlocksDominancePattern,
    pub bitalo: BlocksDominancePattern,
    pub hhtt: BlocksDominancePattern,
    pub megabigpower: BlocksDominancePattern,
    pub mtred: BlocksDominancePattern,
    pub nmcbit: BlocksDominancePattern,
    pub yourbtcnet: BlocksDominancePattern,
    pub givemecoins: BlocksDominancePattern,
    pub multicoinco: BlocksDominancePattern,
    pub bcpoolio: BlocksDominancePattern,
    pub cointerra: BlocksDominancePattern,
    pub kanopool: BlocksDominancePattern,
    pub solock: BlocksDominancePattern,
    pub ckpool: BlocksDominancePattern,
    pub nicehash: BlocksDominancePattern,
    pub bitclub: BlocksDominancePattern,
    pub bitcoinaffiliatenetwork: BlocksDominancePattern,
    pub exxbw: BlocksDominancePattern,
    pub bitsolo: BlocksDominancePattern,
    pub twentyoneinc: BlocksDominancePattern,
    pub digitalbtc: BlocksDominancePattern,
    pub eightbaochi: BlocksDominancePattern,
    pub mybtccoinpool: BlocksDominancePattern,
    pub tbdice: BlocksDominancePattern,
    pub hashpool: BlocksDominancePattern,
    pub nexious: BlocksDominancePattern,
    pub bravomining: BlocksDominancePattern,
    pub hotpool: BlocksDominancePattern,
    pub okexpool: BlocksDominancePattern,
    pub bcmonster: BlocksDominancePattern,
    pub onehash: BlocksDominancePattern,
    pub bixin: BlocksDominancePattern,
    pub tatmaspool: BlocksDominancePattern,
    pub connectbtc: BlocksDominancePattern,
    pub batpool: BlocksDominancePattern,
    pub waterhole: BlocksDominancePattern,
    pub dcexploration: BlocksDominancePattern,
    pub dcex: BlocksDominancePattern,
    pub btpool: BlocksDominancePattern,
    pub fiftyeightcoin: BlocksDominancePattern,
    pub bitcoinindia: BlocksDominancePattern,
    pub shawnp0wers: BlocksDominancePattern,
    pub phashio: BlocksDominancePattern,
    pub rigpool: BlocksDominancePattern,
    pub haozhuzhu: BlocksDominancePattern,
    pub sevenpool: BlocksDominancePattern,
    pub miningkings: BlocksDominancePattern,
    pub hashbx: BlocksDominancePattern,
    pub dpool: BlocksDominancePattern,
    pub rawpool: BlocksDominancePattern,
    pub haominer: BlocksDominancePattern,
    pub helix: BlocksDominancePattern,
    pub bitcoinukraine: BlocksDominancePattern,
    pub secretsuperstar: BlocksDominancePattern,
    pub tigerpoolnet: BlocksDominancePattern,
    pub sigmapoolcom: BlocksDominancePattern,
    pub okpooltop: BlocksDominancePattern,
    pub hummerpool: BlocksDominancePattern,
    pub tangpool: BlocksDominancePattern,
    pub bytepool: BlocksDominancePattern,
    pub novablock: BlocksDominancePattern,
    pub miningcity: BlocksDominancePattern,
    pub minerium: BlocksDominancePattern,
    pub lubiancom: BlocksDominancePattern,
    pub okkong: BlocksDominancePattern,
    pub aaopool: BlocksDominancePattern,
    pub emcdpool: BlocksDominancePattern,
    pub arkpool: BlocksDominancePattern,
    pub purebtccom: BlocksDominancePattern,
    pub kucoinpool: BlocksDominancePattern,
    pub entrustcharitypool: BlocksDominancePattern,
    pub okminer: BlocksDominancePattern,
    pub titan: BlocksDominancePattern,
    pub pegapool: BlocksDominancePattern,
    pub btcnuggets: BlocksDominancePattern,
    pub cloudhashing: BlocksDominancePattern,
    pub digitalxmintsy: BlocksDominancePattern,
    pub telco214: BlocksDominancePattern,
    pub btcpoolparty: BlocksDominancePattern,
    pub multipool: BlocksDominancePattern,
    pub transactioncoinmining: BlocksDominancePattern,
    pub btcdig: BlocksDominancePattern,
    pub trickysbtcpool: BlocksDominancePattern,
    pub btcmp: BlocksDominancePattern,
    pub eobot: BlocksDominancePattern,
    pub unomp: BlocksDominancePattern,
    pub patels: BlocksDominancePattern,
    pub gogreenlight: BlocksDominancePattern,
    pub bitcoinindiapool: BlocksDominancePattern,
    pub ekanembtc: BlocksDominancePattern,
    pub canoe: BlocksDominancePattern,
    pub tiger: BlocksDominancePattern,
    pub onem1x: BlocksDominancePattern,
    pub zulupool: BlocksDominancePattern,
    pub wiz: BlocksDominancePattern,
    pub wk057: BlocksDominancePattern,
    pub futurebitapollosolo: BlocksDominancePattern,
    pub carbonnegative: BlocksDominancePattern,
    pub portlandhodl: BlocksDominancePattern,
    pub phoenix: BlocksDominancePattern,
    pub neopool: BlocksDominancePattern,
    pub maxipool: BlocksDominancePattern,
    pub bitfufupool: BlocksDominancePattern,
    pub gdpool: BlocksDominancePattern,
    pub miningdutch: BlocksDominancePattern,
    pub publicpool: BlocksDominancePattern,
    pub miningsquared: BlocksDominancePattern,
    pub innopolistech: BlocksDominancePattern,
    pub btclab: BlocksDominancePattern,
    pub parasite: BlocksDominancePattern,
    pub redrockpool: BlocksDominancePattern,
    pub est3lar: BlocksDominancePattern,
    pub braiinssolo: BlocksDominancePattern,
    pub solopool: BlocksDominancePattern,
    pub noderunners: BlocksDominancePattern,
}

impl SeriesTree_Pools_Minor {
    pub fn new(client: Arc<BrkClientBase>, base_path: String) -> Self {
        Self {
            blockfills: BlocksDominancePattern::new(client.clone(), "blockfills".to_string()),
            ultimuspool: BlocksDominancePattern::new(client.clone(), "ultimuspool".to_string()),
            terrapool: BlocksDominancePattern::new(client.clone(), "terrapool".to_string()),
            onethash: BlocksDominancePattern::new(client.clone(), "onethash".to_string()),
            bitfarms: BlocksDominancePattern::new(client.clone(), "bitfarms".to_string()),
            huobipool: BlocksDominancePattern::new(client.clone(), "huobipool".to_string()),
            wayicn: BlocksDominancePattern::new(client.clone(), "wayicn".to_string()),
            canoepool: BlocksDominancePattern::new(client.clone(), "canoepool".to_string()),
            bitcoincom: BlocksDominancePattern::new(client.clone(), "bitcoincom".to_string()),
            pool175btc: BlocksDominancePattern::new(client.clone(), "pool175btc".to_string()),
            gbminers: BlocksDominancePattern::new(client.clone(), "gbminers".to_string()),
            axbt: BlocksDominancePattern::new(client.clone(), "axbt".to_string()),
            asicminer: BlocksDominancePattern::new(client.clone(), "asicminer".to_string()),
            bitminter: BlocksDominancePattern::new(client.clone(), "bitminter".to_string()),
            bitcoinrussia: BlocksDominancePattern::new(client.clone(), "bitcoinrussia".to_string()),
            btcserv: BlocksDominancePattern::new(client.clone(), "btcserv".to_string()),
            simplecoinus: BlocksDominancePattern::new(client.clone(), "simplecoinus".to_string()),
            ozcoin: BlocksDominancePattern::new(client.clone(), "ozcoin".to_string()),
            eclipsemc: BlocksDominancePattern::new(client.clone(), "eclipsemc".to_string()),
            maxbtc: BlocksDominancePattern::new(client.clone(), "maxbtc".to_string()),
            triplemining: BlocksDominancePattern::new(client.clone(), "triplemining".to_string()),
            coinlab: BlocksDominancePattern::new(client.clone(), "coinlab".to_string()),
            pool50btc: BlocksDominancePattern::new(client.clone(), "pool50btc".to_string()),
            ghashio: BlocksDominancePattern::new(client.clone(), "ghashio".to_string()),
            stminingcorp: BlocksDominancePattern::new(client.clone(), "stminingcorp".to_string()),
            bitparking: BlocksDominancePattern::new(client.clone(), "bitparking".to_string()),
            mmpool: BlocksDominancePattern::new(client.clone(), "mmpool".to_string()),
            polmine: BlocksDominancePattern::new(client.clone(), "polmine".to_string()),
            kncminer: BlocksDominancePattern::new(client.clone(), "kncminer".to_string()),
            bitalo: BlocksDominancePattern::new(client.clone(), "bitalo".to_string()),
            hhtt: BlocksDominancePattern::new(client.clone(), "hhtt".to_string()),
            megabigpower: BlocksDominancePattern::new(client.clone(), "megabigpower".to_string()),
            mtred: BlocksDominancePattern::new(client.clone(), "mtred".to_string()),
            nmcbit: BlocksDominancePattern::new(client.clone(), "nmcbit".to_string()),
            yourbtcnet: BlocksDominancePattern::new(client.clone(), "yourbtcnet".to_string()),
            givemecoins: BlocksDominancePattern::new(client.clone(), "givemecoins".to_string()),
            multicoinco: BlocksDominancePattern::new(client.clone(), "multicoinco".to_string()),
            bcpoolio: BlocksDominancePattern::new(client.clone(), "bcpoolio".to_string()),
            cointerra: BlocksDominancePattern::new(client.clone(), "cointerra".to_string()),
            kanopool: BlocksDominancePattern::new(client.clone(), "kanopool".to_string()),
            solock: BlocksDominancePattern::new(client.clone(), "solock".to_string()),
            ckpool: BlocksDominancePattern::new(client.clone(), "ckpool".to_string()),
            nicehash: BlocksDominancePattern::new(client.clone(), "nicehash".to_string()),
            bitclub: BlocksDominancePattern::new(client.clone(), "bitclub".to_string()),
            bitcoinaffiliatenetwork: BlocksDominancePattern::new(client.clone(), "bitcoinaffiliatenetwork".to_string()),
            exxbw: BlocksDominancePattern::new(client.clone(), "exxbw".to_string()),
            bitsolo: BlocksDominancePattern::new(client.clone(), "bitsolo".to_string()),
            twentyoneinc: BlocksDominancePattern::new(client.clone(), "twentyoneinc".to_string()),
            digitalbtc: BlocksDominancePattern::new(client.clone(), "digitalbtc".to_string()),
            eightbaochi: BlocksDominancePattern::new(client.clone(), "eightbaochi".to_string()),
            mybtccoinpool: BlocksDominancePattern::new(client.clone(), "mybtccoinpool".to_string()),
            tbdice: BlocksDominancePattern::new(client.clone(), "tbdice".to_string()),
            hashpool: BlocksDominancePattern::new(client.clone(), "hashpool".to_string()),
            nexious: BlocksDominancePattern::new(client.clone(), "nexious".to_string()),
            bravomining: BlocksDominancePattern::new(client.clone(), "bravomining".to_string()),
            hotpool: BlocksDominancePattern::new(client.clone(), "hotpool".to_string()),
            okexpool: BlocksDominancePattern::new(client.clone(), "okexpool".to_string()),
            bcmonster: BlocksDominancePattern::new(client.clone(), "bcmonster".to_string()),
            onehash: BlocksDominancePattern::new(client.clone(), "onehash".to_string()),
            bixin: BlocksDominancePattern::new(client.clone(), "bixin".to_string()),
            tatmaspool: BlocksDominancePattern::new(client.clone(), "tatmaspool".to_string()),
            connectbtc: BlocksDominancePattern::new(client.clone(), "connectbtc".to_string()),
            batpool: BlocksDominancePattern::new(client.clone(), "batpool".to_string()),
            waterhole: BlocksDominancePattern::new(client.clone(), "waterhole".to_string()),
            dcexploration: BlocksDominancePattern::new(client.clone(), "dcexploration".to_string()),
            dcex: BlocksDominancePattern::new(client.clone(), "dcex".to_string()),
            btpool: BlocksDominancePattern::new(client.clone(), "btpool".to_string()),
            fiftyeightcoin: BlocksDominancePattern::new(client.clone(), "fiftyeightcoin".to_string()),
            bitcoinindia: BlocksDominancePattern::new(client.clone(), "bitcoinindia".to_string()),
            shawnp0wers: BlocksDominancePattern::new(client.clone(), "shawnp0wers".to_string()),
            phashio: BlocksDominancePattern::new(client.clone(), "phashio".to_string()),
            rigpool: BlocksDominancePattern::new(client.clone(), "rigpool".to_string()),
            haozhuzhu: BlocksDominancePattern::new(client.clone(), "haozhuzhu".to_string()),
            sevenpool: BlocksDominancePattern::new(client.clone(), "sevenpool".to_string()),
            miningkings: BlocksDominancePattern::new(client.clone(), "miningkings".to_string()),
            hashbx: BlocksDominancePattern::new(client.clone(), "hashbx".to_string()),
            dpool: BlocksDominancePattern::new(client.clone(), "dpool".to_string()),
            rawpool: BlocksDominancePattern::new(client.clone(), "rawpool".to_string()),
            haominer: BlocksDominancePattern::new(client.clone(), "haominer".to_string()),
            helix: BlocksDominancePattern::new(client.clone(), "helix".to_string()),
            bitcoinukraine: BlocksDominancePattern::new(client.clone(), "bitcoinukraine".to_string()),
            secretsuperstar: BlocksDominancePattern::new(client.clone(), "secretsuperstar".to_string()),
            tigerpoolnet: BlocksDominancePattern::new(client.clone(), "tigerpoolnet".to_string()),
            sigmapoolcom: BlocksDominancePattern::new(client.clone(), "sigmapoolcom".to_string()),
            okpooltop: BlocksDominancePattern::new(client.clone(), "okpooltop".to_string()),
            hummerpool: BlocksDominancePattern::new(client.clone(), "hummerpool".to_string()),
            tangpool: BlocksDominancePattern::new(client.clone(), "tangpool".to_string()),
            bytepool: BlocksDominancePattern::new(client.clone(), "bytepool".to_string()),
            novablock: BlocksDominancePattern::new(client.clone(), "novablock".to_string()),
            miningcity: BlocksDominancePattern::new(client.clone(), "miningcity".to_string()),
            minerium: BlocksDominancePattern::new(client.clone(), "minerium".to_string()),
            lubiancom: BlocksDominancePattern::new(client.clone(), "lubiancom".to_string()),
            okkong: BlocksDominancePattern::new(client.clone(), "okkong".to_string()),
            aaopool: BlocksDominancePattern::new(client.clone(), "aaopool".to_string()),
            emcdpool: BlocksDominancePattern::new(client.clone(), "emcdpool".to_string()),
            arkpool: BlocksDominancePattern::new(client.clone(), "arkpool".to_string()),
            purebtccom: BlocksDominancePattern::new(client.clone(), "purebtccom".to_string()),
            kucoinpool: BlocksDominancePattern::new(client.clone(), "kucoinpool".to_string()),
            entrustcharitypool: BlocksDominancePattern::new(client.clone(), "entrustcharitypool".to_string()),
            okminer: BlocksDominancePattern::new(client.clone(), "okminer".to_string()),
            titan: BlocksDominancePattern::new(client.clone(), "titan".to_string()),
            pegapool: BlocksDominancePattern::new(client.clone(), "pegapool".to_string()),
            btcnuggets: BlocksDominancePattern::new(client.clone(), "btcnuggets".to_string()),
            cloudhashing: BlocksDominancePattern::new(client.clone(), "cloudhashing".to_string()),
            digitalxmintsy: BlocksDominancePattern::new(client.clone(), "digitalxmintsy".to_string()),
            telco214: BlocksDominancePattern::new(client.clone(), "telco214".to_string()),
            btcpoolparty: BlocksDominancePattern::new(client.clone(), "btcpoolparty".to_string()),
            multipool: BlocksDominancePattern::new(client.clone(), "multipool".to_string()),
            transactioncoinmining: BlocksDominancePattern::new(client.clone(), "transactioncoinmining".to_string()),
            btcdig: BlocksDominancePattern::new(client.clone(), "btcdig".to_string()),
            trickysbtcpool: BlocksDominancePattern::new(client.clone(), "trickysbtcpool".to_string()),
            btcmp: BlocksDominancePattern::new(client.clone(), "btcmp".to_string()),
            eobot: BlocksDominancePattern::new(client.clone(), "eobot".to_string()),
            unomp: BlocksDominancePattern::new(client.clone(), "unomp".to_string()),
            patels: BlocksDominancePattern::new(client.clone(), "patels".to_string()),
            gogreenlight: BlocksDominancePattern::new(client.clone(), "gogreenlight".to_string()),
            bitcoinindiapool: BlocksDominancePattern::new(client.clone(), "bitcoinindiapool".to_string()),
            ekanembtc: BlocksDominancePattern::new(client.clone(), "ekanembtc".to_string()),
            canoe: BlocksDominancePattern::new(client.clone(), "canoe".to_string()),
            tiger: BlocksDominancePattern::new(client.clone(), "tiger".to_string()),
            onem1x: BlocksDominancePattern::new(client.clone(), "onem1x".to_string()),
            zulupool: BlocksDominancePattern::new(client.clone(), "zulupool".to_string()),
            wiz: BlocksDominancePattern::new(client.clone(), "wiz".to_string()),
            wk057: BlocksDominancePattern::new(client.clone(), "wk057".to_string()),
            futurebitapollosolo: BlocksDominancePattern::new(client.clone(), "futurebitapollosolo".to_string()),
            carbonnegative: BlocksDominancePattern::new(client.clone(), "carbonnegative".to_string()),
            portlandhodl: BlocksDominancePattern::new(client.clone(), "portlandhodl".to_string()),
            phoenix: BlocksDominancePattern::new(client.clone(), "phoenix".to_string()),
            neopool: BlocksDominancePattern::new(client.clone(), "neopool".to_string()),
            maxipool: BlocksDominancePattern::new(client.clone(), "maxipool".to_string()),
            bitfufupool: BlocksDominancePattern::new(client.clone(), "bitfufupool".to_string()),
            gdpool: BlocksDominancePattern::new(client.clone(), "gdpool".to_string()),
            miningdutch: BlocksDominancePattern::new(client.clone(), "miningdutch".to_string()),
            publicpool: BlocksDominancePattern::new(client.clone(), "publicpool".to_string()),
            miningsquared: BlocksDominancePattern::new(client.clone(), "miningsquared".to_string()),
            innopolistech: BlocksDominancePattern::new(client.clone(), "innopolistech".to_string()),
            btclab: BlocksDominancePattern::new(client.clone(), "btclab".to_string()),
            parasite: BlocksDominancePattern::new(client.clone(), "parasite".to_string()),
            redrockpool: BlocksDominancePattern::new(client.clone(), "redrockpool".to_string()),
            est3lar: BlocksDominancePattern::new(client.clone(), "est3lar".to_string()),
            braiinssolo: BlocksDominancePattern::new(client.clone(), "braiinssolo".to_string()),
            solopool: BlocksDominancePattern::new(client.clone(), "solopool".to_string()),
            noderunners: BlocksDominancePattern::new(client.clone(), "noderunners".to_string()),
        }
    }
}

/// Series tree node.
pub struct SeriesTree_Price {
    pub split: SeriesTree_Price_Split,
    pub ohlc: SeriesTree_Price_Ohlc,
    pub spot: SeriesTree_Price_Spot,
}

impl SeriesTree_Price {
    pub fn new(client: Arc<BrkClientBase>, base_path: String) -> Self {
        Self {
            split: SeriesTree_Price_Split::new(client.clone(), format!("{base_path}_split")),
            ohlc: SeriesTree_Price_Ohlc::new(client.clone(), format!("{base_path}_ohlc")),
            spot: SeriesTree_Price_Spot::new(client.clone(), format!("{base_path}_spot")),
        }
    }
}

/// Series tree node.
pub struct SeriesTree_Price_Split {
    pub open: CentsSatsUsdPattern3,
    pub high: CentsSatsUsdPattern3,
    pub low: CentsSatsUsdPattern3,
    pub close: CentsSatsUsdPattern3,
}

impl SeriesTree_Price_Split {
    pub fn new(client: Arc<BrkClientBase>, base_path: String) -> Self {
        Self {
            open: CentsSatsUsdPattern3::new(client.clone(), "price_open".to_string()),
            high: CentsSatsUsdPattern3::new(client.clone(), "price_high".to_string()),
            low: CentsSatsUsdPattern3::new(client.clone(), "price_low".to_string()),
            close: CentsSatsUsdPattern3::new(client.clone(), "price_close".to_string()),
        }
    }
}

/// Series tree node.
pub struct SeriesTree_Price_Ohlc {
    pub usd: SeriesPattern2<OHLCDollars>,
    pub cents: SeriesPattern2<OHLCCents>,
    pub sats: SeriesPattern2<OHLCSats>,
}

impl SeriesTree_Price_Ohlc {
    pub fn new(client: Arc<BrkClientBase>, base_path: String) -> Self {
        Self {
            usd: SeriesPattern2::new(client.clone(), "price_ohlc".to_string()),
            cents: SeriesPattern2::new(client.clone(), "price_ohlc_cents".to_string()),
            sats: SeriesPattern2::new(client.clone(), "price_ohlc_sats".to_string()),
        }
    }
}

/// Series tree node.
pub struct SeriesTree_Price_Spot {
    pub usd: SeriesPattern1<Dollars>,
    pub cents: SeriesPattern1<Cents>,
    pub sats: SeriesPattern1<Sats>,
}

impl SeriesTree_Price_Spot {
    pub fn new(client: Arc<BrkClientBase>, base_path: String) -> Self {
        Self {
            usd: SeriesPattern1::new(client.clone(), "price".to_string()),
            cents: SeriesPattern1::new(client.clone(), "price_cents".to_string()),
            sats: SeriesPattern1::new(client.clone(), "price_sats".to_string()),
        }
    }
}

/// Series tree node.
pub struct SeriesTree_Supply {
    pub state: SeriesPattern18<SupplyState>,
    pub circulating: BtcCentsSatsUsdPattern,
    pub burned: BlockCumulativePattern,
    pub inflation_rate: BpsPercentRatioPattern,
    pub velocity: SeriesTree_Supply_Velocity,
    pub market_cap: CentsDeltaUsdPattern,
    pub market_minus_realized_cap_growth_rate: _1m1w1y24hPattern<BasisPointsSigned32>,
    pub hodled_or_lost: BtcCentsSatsUsdPattern,
}

impl SeriesTree_Supply {
    pub fn new(client: Arc<BrkClientBase>, base_path: String) -> Self {
        Self {
            state: SeriesPattern18::new(client.clone(), "supply_state".to_string()),
            circulating: BtcCentsSatsUsdPattern::new(client.clone(), "circulating_supply".to_string()),
            burned: BlockCumulativePattern::new(client.clone(), "unspendable_supply".to_string()),
            inflation_rate: BpsPercentRatioPattern::new(client.clone(), "inflation_rate".to_string()),
            velocity: SeriesTree_Supply_Velocity::new(client.clone(), format!("{base_path}_velocity")),
            market_cap: CentsDeltaUsdPattern::new(client.clone(), "market_cap".to_string()),
            market_minus_realized_cap_growth_rate: _1m1w1y24hPattern::new(client.clone(), "market_minus_realized_cap_growth_rate".to_string()),
            hodled_or_lost: BtcCentsSatsUsdPattern::new(client.clone(), "hodled_or_lost_supply".to_string()),
        }
    }
}

/// Series tree node.
pub struct SeriesTree_Supply_Velocity {
    pub native: SeriesPattern1<StoredF64>,
    pub fiat: SeriesPattern1<StoredF64>,
}

impl SeriesTree_Supply_Velocity {
    pub fn new(client: Arc<BrkClientBase>, base_path: String) -> Self {
        Self {
            native: SeriesPattern1::new(client.clone(), "velocity_btc".to_string()),
            fiat: SeriesPattern1::new(client.clone(), "velocity_usd".to_string()),
        }
    }
}

/// Series tree node.
pub struct SeriesTree_Cohorts {
    pub utxo: SeriesTree_Cohorts_Utxo,
    pub addr: SeriesTree_Cohorts_Addr,
}

impl SeriesTree_Cohorts {
    pub fn new(client: Arc<BrkClientBase>, base_path: String) -> Self {
        Self {
            utxo: SeriesTree_Cohorts_Utxo::new(client.clone(), format!("{base_path}_utxo")),
            addr: SeriesTree_Cohorts_Addr::new(client.clone(), format!("{base_path}_addr")),
        }
    }
}

/// Series tree node.
pub struct SeriesTree_Cohorts_Utxo {
    pub all: SeriesTree_Cohorts_Utxo_All,
    pub sth: SeriesTree_Cohorts_Utxo_Sth,
    pub lth: SeriesTree_Cohorts_Utxo_Lth,
    pub age_range: SeriesTree_Cohorts_Utxo_AgeRange,
    pub under_age: SeriesTree_Cohorts_Utxo_UnderAge,
    pub over_age: SeriesTree_Cohorts_Utxo_OverAge,
    pub epoch: SeriesTree_Cohorts_Utxo_Epoch,
    pub class: SeriesTree_Cohorts_Utxo_Class,
    pub entry: SeriesTree_Cohorts_Utxo_Entry,
    pub over_amount: SeriesTree_Cohorts_Utxo_OverAmount,
    pub amount_range: SeriesTree_Cohorts_Utxo_AmountRange,
    pub under_amount: SeriesTree_Cohorts_Utxo_UnderAmount,
    pub type_: SeriesTree_Cohorts_Utxo_Type,
    pub profitability: SeriesTree_Cohorts_Utxo_Profitability,
    pub matured: SeriesTree_Cohorts_Utxo_Matured,
}

impl SeriesTree_Cohorts_Utxo {
    pub fn new(client: Arc<BrkClientBase>, base_path: String) -> Self {
        Self {
            all: SeriesTree_Cohorts_Utxo_All::new(client.clone(), format!("{base_path}_all")),
            sth: SeriesTree_Cohorts_Utxo_Sth::new(client.clone(), format!("{base_path}_sth")),
            lth: SeriesTree_Cohorts_Utxo_Lth::new(client.clone(), format!("{base_path}_lth")),
            age_range: SeriesTree_Cohorts_Utxo_AgeRange::new(client.clone(), format!("{base_path}_age_range")),
            under_age: SeriesTree_Cohorts_Utxo_UnderAge::new(client.clone(), format!("{base_path}_under_age")),
            over_age: SeriesTree_Cohorts_Utxo_OverAge::new(client.clone(), format!("{base_path}_over_age")),
            epoch: SeriesTree_Cohorts_Utxo_Epoch::new(client.clone(), format!("{base_path}_epoch")),
            class: SeriesTree_Cohorts_Utxo_Class::new(client.clone(), format!("{base_path}_class")),
            entry: SeriesTree_Cohorts_Utxo_Entry::new(client.clone(), format!("{base_path}_entry")),
            over_amount: SeriesTree_Cohorts_Utxo_OverAmount::new(client.clone(), format!("{base_path}_over_amount")),
            amount_range: SeriesTree_Cohorts_Utxo_AmountRange::new(client.clone(), format!("{base_path}_amount_range")),
            under_amount: SeriesTree_Cohorts_Utxo_UnderAmount::new(client.clone(), format!("{base_path}_under_amount")),
            type_: SeriesTree_Cohorts_Utxo_Type::new(client.clone(), format!("{base_path}_type")),
            profitability: SeriesTree_Cohorts_Utxo_Profitability::new(client.clone(), format!("{base_path}_profitability")),
            matured: SeriesTree_Cohorts_Utxo_Matured::new(client.clone(), format!("{base_path}_matured")),
        }
    }
}

/// Series tree node.
pub struct SeriesTree_Cohorts_Utxo_All {
    pub supply: DeltaDominanceHalfInTotalPattern2,
    pub outputs: SeriesTree_Cohorts_Utxo_All_Outputs,
    pub activity: SeriesTree_Cohorts_Utxo_All_Activity,
    pub realized: SeriesTree_Cohorts_Utxo_All_Realized,
    pub cost_basis: SeriesTree_Cohorts_Utxo_All_CostBasis,
    pub unrealized: SeriesTree_Cohorts_Utxo_All_Unrealized,
    pub invested_capital: InPattern,
}

impl SeriesTree_Cohorts_Utxo_All {
    pub fn new(client: Arc<BrkClientBase>, base_path: String) -> Self {
        Self {
            supply: DeltaDominanceHalfInTotalPattern2::new(client.clone(), "supply".to_string()),
            outputs: SeriesTree_Cohorts_Utxo_All_Outputs::new(client.clone(), format!("{base_path}_outputs")),
            activity: SeriesTree_Cohorts_Utxo_All_Activity::new(client.clone(), format!("{base_path}_activity")),
            realized: SeriesTree_Cohorts_Utxo_All_Realized::new(client.clone(), format!("{base_path}_realized")),
            cost_basis: SeriesTree_Cohorts_Utxo_All_CostBasis::new(client.clone(), format!("{base_path}_cost_basis")),
            unrealized: SeriesTree_Cohorts_Utxo_All_Unrealized::new(client.clone(), format!("{base_path}_unrealized")),
            invested_capital: InPattern::new(client.clone(), "invested_capital_in".to_string()),
        }
    }
}

/// Series tree node.
pub struct SeriesTree_Cohorts_Utxo_All_Outputs {
    pub unspent_count: BaseDeltaPattern,
    pub spent_count: AverageBlockCumulativeSumPattern2,
    pub spending_rate: SeriesPattern1<StoredF32>,
}

impl SeriesTree_Cohorts_Utxo_All_Outputs {
    pub fn new(client: Arc<BrkClientBase>, base_path: String) -> Self {
        Self {
            unspent_count: BaseDeltaPattern::new(client.clone(), "utxo_count".to_string()),
            spent_count: AverageBlockCumulativeSumPattern2::new(client.clone(), "spent_utxo_count".to_string()),
            spending_rate: SeriesPattern1::new(client.clone(), "spending_rate".to_string()),
        }
    }
}

/// Series tree node.
pub struct SeriesTree_Cohorts_Utxo_All_Activity {
    pub transfer_volume: AverageBlockCumulativeInSumPattern,
    pub coindays_destroyed: AverageBlockCumulativeSumPattern<StoredF64>,
    pub coinyears_destroyed: SeriesPattern1<StoredF64>,
    pub dormancy: _1m1w1y24hPattern<StoredF32>,
}

impl SeriesTree_Cohorts_Utxo_All_Activity {
    pub fn new(client: Arc<BrkClientBase>, base_path: String) -> Self {
        Self {
            transfer_volume: AverageBlockCumulativeInSumPattern::new(client.clone(), "transfer_volume".to_string()),
            coindays_destroyed: AverageBlockCumulativeSumPattern::new(client.clone(), "coindays_destroyed".to_string()),
            coinyears_destroyed: SeriesPattern1::new(client.clone(), "coinyears_destroyed".to_string()),
            dormancy: _1m1w1y24hPattern::new(client.clone(), "dormancy".to_string()),
        }
    }
}

/// Series tree node.
pub struct SeriesTree_Cohorts_Utxo_All_Realized {
    pub cap: CentsDeltaToUsdPattern,
    pub profit: BlockCumulativeSumPattern,
    pub loss: BlockCumulativeNegativeSumPattern,
    pub price: SeriesTree_Cohorts_Utxo_All_Realized_Price,
    pub mvrv: SeriesPattern1<StoredF32>,
    pub net_pnl: BlockChangeCumulativeDeltaSumPattern,
    pub sopr: SeriesTree_Cohorts_Utxo_All_Realized_Sopr,
    pub gross_pnl: BlockCumulativeSumPattern,
    pub sell_side_risk_ratio: _1m1w1y24hPattern8,
    pub peak_regret: BlockCumulativeSumPattern,
    pub capitalized: PricePattern,
    pub profit_to_loss_ratio: _1m1w1y24hPattern<StoredF64>,
}

impl SeriesTree_Cohorts_Utxo_All_Realized {
    pub fn new(client: Arc<BrkClientBase>, base_path: String) -> Self {
        Self {
            cap: CentsDeltaToUsdPattern::new(client.clone(), "realized_cap".to_string()),
            profit: BlockCumulativeSumPattern::new(client.clone(), "realized_profit".to_string()),
            loss: BlockCumulativeNegativeSumPattern::new(client.clone(), "realized_loss".to_string()),
            price: SeriesTree_Cohorts_Utxo_All_Realized_Price::new(client.clone(), format!("{base_path}_price")),
            mvrv: SeriesPattern1::new(client.clone(), "mvrv".to_string()),
            net_pnl: BlockChangeCumulativeDeltaSumPattern::new(client.clone(), "net".to_string()),
            sopr: SeriesTree_Cohorts_Utxo_All_Realized_Sopr::new(client.clone(), format!("{base_path}_sopr")),
            gross_pnl: BlockCumulativeSumPattern::new(client.clone(), "realized_gross_pnl".to_string()),
            sell_side_risk_ratio: _1m1w1y24hPattern8::new(client.clone(), "sell_side_risk_ratio".to_string()),
            peak_regret: BlockCumulativeSumPattern::new(client.clone(), "realized_peak_regret".to_string()),
            capitalized: PricePattern::new(client.clone(), "capitalized_price".to_string()),
            profit_to_loss_ratio: _1m1w1y24hPattern::new(client.clone(), "realized_profit_to_loss_ratio".to_string()),
        }
    }
}

/// Series tree node.
pub struct SeriesTree_Cohorts_Utxo_All_Realized_Price {
    pub usd: SeriesPattern1<Dollars>,
    pub cents: SeriesPattern1<Cents>,
    pub sats: SeriesPattern1<SatsFract>,
    pub bps: SeriesPattern1<BasisPoints32>,
    pub ratio: SeriesPattern1<StoredF32>,
    pub percentiles: Pct0Pct1Pct2Pct5Pct95Pct98Pct99Pattern,
    pub sma: _1m1w1y2y4yAllPattern,
    pub std_dev: SeriesTree_Cohorts_Utxo_All_Realized_Price_StdDev,
}

impl SeriesTree_Cohorts_Utxo_All_Realized_Price {
    pub fn new(client: Arc<BrkClientBase>, base_path: String) -> Self {
        Self {
            usd: SeriesPattern1::new(client.clone(), "realized_price".to_string()),
            cents: SeriesPattern1::new(client.clone(), "realized_price_cents".to_string()),
            sats: SeriesPattern1::new(client.clone(), "realized_price_sats".to_string()),
            bps: SeriesPattern1::new(client.clone(), "realized_price_ratio_bps".to_string()),
            ratio: SeriesPattern1::new(client.clone(), "realized_price_ratio".to_string()),
            percentiles: Pct0Pct1Pct2Pct5Pct95Pct98Pct99Pattern::new(client.clone(), "realized_price".to_string()),
            sma: _1m1w1y2y4yAllPattern::new(client.clone(), "realized_price_ratio_sma".to_string()),
            std_dev: SeriesTree_Cohorts_Utxo_All_Realized_Price_StdDev::new(client.clone(), format!("{base_path}_std_dev")),
        }
    }
}

/// Series tree node.
pub struct SeriesTree_Cohorts_Utxo_All_Realized_Price_StdDev {
    pub all: SeriesTree_Cohorts_Utxo_All_Realized_Price_StdDev_All,
    pub _4y: SeriesTree_Cohorts_Utxo_All_Realized_Price_StdDev_4y,
    pub _2y: SeriesTree_Cohorts_Utxo_All_Realized_Price_StdDev_2y,
    pub _1y: SeriesTree_Cohorts_Utxo_All_Realized_Price_StdDev_1y,
}

impl SeriesTree_Cohorts_Utxo_All_Realized_Price_StdDev {
    pub fn new(client: Arc<BrkClientBase>, base_path: String) -> Self {
        Self {
            all: SeriesTree_Cohorts_Utxo_All_Realized_Price_StdDev_All::new(client.clone(), format!("{base_path}_all")),
            _4y: SeriesTree_Cohorts_Utxo_All_Realized_Price_StdDev_4y::new(client.clone(), format!("{base_path}_4y")),
            _2y: SeriesTree_Cohorts_Utxo_All_Realized_Price_StdDev_2y::new(client.clone(), format!("{base_path}_2y")),
            _1y: SeriesTree_Cohorts_Utxo_All_Realized_Price_StdDev_1y::new(client.clone(), format!("{base_path}_1y")),
        }
    }
}

/// Series tree node.
pub struct SeriesTree_Cohorts_Utxo_All_Realized_Price_StdDev_All {
    pub sd: SeriesPattern1<StoredF32>,
    pub zscore: SeriesPattern1<StoredF32>,
    pub _0sd: CentsSatsUsdPattern,
    pub p0_5sd: PriceRatioPattern,
    pub p1sd: PriceRatioPattern,
    pub p1_5sd: PriceRatioPattern,
    pub p2sd: PriceRatioPattern,
    pub p2_5sd: PriceRatioPattern,
    pub p3sd: PriceRatioPattern,
    pub m0_5sd: PriceRatioPattern,
    pub m1sd: PriceRatioPattern,
    pub m1_5sd: PriceRatioPattern,
    pub m2sd: PriceRatioPattern,
    pub m2_5sd: PriceRatioPattern,
    pub m3sd: PriceRatioPattern,
}

impl SeriesTree_Cohorts_Utxo_All_Realized_Price_StdDev_All {
    pub fn new(client: Arc<BrkClientBase>, base_path: String) -> Self {
        Self {
            sd: SeriesPattern1::new(client.clone(), "realized_price_ratio_sd".to_string()),
            zscore: SeriesPattern1::new(client.clone(), "realized_price_ratio_zscore".to_string()),
            _0sd: CentsSatsUsdPattern::new(client.clone(), "realized_price_0sd".to_string()),
            p0_5sd: PriceRatioPattern::new(client.clone(), "realized_price".to_string(), "p0_5sd".to_string()),
            p1sd: PriceRatioPattern::new(client.clone(), "realized_price".to_string(), "p1sd".to_string()),
            p1_5sd: PriceRatioPattern::new(client.clone(), "realized_price".to_string(), "p1_5sd".to_string()),
            p2sd: PriceRatioPattern::new(client.clone(), "realized_price".to_string(), "p2sd".to_string()),
            p2_5sd: PriceRatioPattern::new(client.clone(), "realized_price".to_string(), "p2_5sd".to_string()),
            p3sd: PriceRatioPattern::new(client.clone(), "realized_price".to_string(), "p3sd".to_string()),
            m0_5sd: PriceRatioPattern::new(client.clone(), "realized_price".to_string(), "m0_5sd".to_string()),
            m1sd: PriceRatioPattern::new(client.clone(), "realized_price".to_string(), "m1sd".to_string()),
            m1_5sd: PriceRatioPattern::new(client.clone(), "realized_price".to_string(), "m1_5sd".to_string()),
            m2sd: PriceRatioPattern::new(client.clone(), "realized_price".to_string(), "m2sd".to_string()),
            m2_5sd: PriceRatioPattern::new(client.clone(), "realized_price".to_string(), "m2_5sd".to_string()),
            m3sd: PriceRatioPattern::new(client.clone(), "realized_price".to_string(), "m3sd".to_string()),
        }
    }
}

/// Series tree node.
pub struct SeriesTree_Cohorts_Utxo_All_Realized_Price_StdDev_4y {
    pub sd: SeriesPattern1<StoredF32>,
    pub zscore: SeriesPattern1<StoredF32>,
    pub _0sd: CentsSatsUsdPattern,
    pub p0_5sd: PriceRatioPattern,
    pub p1sd: PriceRatioPattern,
    pub p1_5sd: PriceRatioPattern,
    pub p2sd: PriceRatioPattern,
    pub p2_5sd: PriceRatioPattern,
    pub p3sd: PriceRatioPattern,
    pub m0_5sd: PriceRatioPattern,
    pub m1sd: PriceRatioPattern,
    pub m1_5sd: PriceRatioPattern,
    pub m2sd: PriceRatioPattern,
    pub m2_5sd: PriceRatioPattern,
    pub m3sd: PriceRatioPattern,
}

impl SeriesTree_Cohorts_Utxo_All_Realized_Price_StdDev_4y {
    pub fn new(client: Arc<BrkClientBase>, base_path: String) -> Self {
        Self {
            sd: SeriesPattern1::new(client.clone(), "realized_price_ratio_sd_4y".to_string()),
            zscore: SeriesPattern1::new(client.clone(), "realized_price_ratio_zscore_4y".to_string()),
            _0sd: CentsSatsUsdPattern::new(client.clone(), "realized_price_0sd_4y".to_string()),
            p0_5sd: PriceRatioPattern::new(client.clone(), "realized_price".to_string(), "p0_5sd_4y".to_string()),
            p1sd: PriceRatioPattern::new(client.clone(), "realized_price".to_string(), "p1sd_4y".to_string()),
            p1_5sd: PriceRatioPattern::new(client.clone(), "realized_price".to_string(), "p1_5sd_4y".to_string()),
            p2sd: PriceRatioPattern::new(client.clone(), "realized_price".to_string(), "p2sd_4y".to_string()),
            p2_5sd: PriceRatioPattern::new(client.clone(), "realized_price".to_string(), "p2_5sd_4y".to_string()),
            p3sd: PriceRatioPattern::new(client.clone(), "realized_price".to_string(), "p3sd_4y".to_string()),
            m0_5sd: PriceRatioPattern::new(client.clone(), "realized_price".to_string(), "m0_5sd_4y".to_string()),
            m1sd: PriceRatioPattern::new(client.clone(), "realized_price".to_string(), "m1sd_4y".to_string()),
            m1_5sd: PriceRatioPattern::new(client.clone(), "realized_price".to_string(), "m1_5sd_4y".to_string()),
            m2sd: PriceRatioPattern::new(client.clone(), "realized_price".to_string(), "m2sd_4y".to_string()),
            m2_5sd: PriceRatioPattern::new(client.clone(), "realized_price".to_string(), "m2_5sd_4y".to_string()),
            m3sd: PriceRatioPattern::new(client.clone(), "realized_price".to_string(), "m3sd_4y".to_string()),
        }
    }
}

/// Series tree node.
pub struct SeriesTree_Cohorts_Utxo_All_Realized_Price_StdDev_2y {
    pub sd: SeriesPattern1<StoredF32>,
    pub zscore: SeriesPattern1<StoredF32>,
    pub _0sd: CentsSatsUsdPattern,
    pub p0_5sd: PriceRatioPattern,
    pub p1sd: PriceRatioPattern,
    pub p1_5sd: PriceRatioPattern,
    pub p2sd: PriceRatioPattern,
    pub p2_5sd: PriceRatioPattern,
    pub p3sd: PriceRatioPattern,
    pub m0_5sd: PriceRatioPattern,
    pub m1sd: PriceRatioPattern,
    pub m1_5sd: PriceRatioPattern,
    pub m2sd: PriceRatioPattern,
    pub m2_5sd: PriceRatioPattern,
    pub m3sd: PriceRatioPattern,
}

impl SeriesTree_Cohorts_Utxo_All_Realized_Price_StdDev_2y {
    pub fn new(client: Arc<BrkClientBase>, base_path: String) -> Self {
        Self {
            sd: SeriesPattern1::new(client.clone(), "realized_price_ratio_sd_2y".to_string()),
            zscore: SeriesPattern1::new(client.clone(), "realized_price_ratio_zscore_2y".to_string()),
            _0sd: CentsSatsUsdPattern::new(client.clone(), "realized_price_0sd_2y".to_string()),
            p0_5sd: PriceRatioPattern::new(client.clone(), "realized_price".to_string(), "p0_5sd_2y".to_string()),
            p1sd: PriceRatioPattern::new(client.clone(), "realized_price".to_string(), "p1sd_2y".to_string()),
            p1_5sd: PriceRatioPattern::new(client.clone(), "realized_price".to_string(), "p1_5sd_2y".to_string()),
            p2sd: PriceRatioPattern::new(client.clone(), "realized_price".to_string(), "p2sd_2y".to_string()),
            p2_5sd: PriceRatioPattern::new(client.clone(), "realized_price".to_string(), "p2_5sd_2y".to_string()),
            p3sd: PriceRatioPattern::new(client.clone(), "realized_price".to_string(), "p3sd_2y".to_string()),
            m0_5sd: PriceRatioPattern::new(client.clone(), "realized_price".to_string(), "m0_5sd_2y".to_string()),
            m1sd: PriceRatioPattern::new(client.clone(), "realized_price".to_string(), "m1sd_2y".to_string()),
            m1_5sd: PriceRatioPattern::new(client.clone(), "realized_price".to_string(), "m1_5sd_2y".to_string()),
            m2sd: PriceRatioPattern::new(client.clone(), "realized_price".to_string(), "m2sd_2y".to_string()),
            m2_5sd: PriceRatioPattern::new(client.clone(), "realized_price".to_string(), "m2_5sd_2y".to_string()),
            m3sd: PriceRatioPattern::new(client.clone(), "realized_price".to_string(), "m3sd_2y".to_string()),
        }
    }
}

/// Series tree node.
pub struct SeriesTree_Cohorts_Utxo_All_Realized_Price_StdDev_1y {
    pub sd: SeriesPattern1<StoredF32>,
    pub zscore: SeriesPattern1<StoredF32>,
    pub _0sd: CentsSatsUsdPattern,
    pub p0_5sd: PriceRatioPattern,
    pub p1sd: PriceRatioPattern,
    pub p1_5sd: PriceRatioPattern,
    pub p2sd: PriceRatioPattern,
    pub p2_5sd: PriceRatioPattern,
    pub p3sd: PriceRatioPattern,
    pub m0_5sd: PriceRatioPattern,
    pub m1sd: PriceRatioPattern,
    pub m1_5sd: PriceRatioPattern,
    pub m2sd: PriceRatioPattern,
    pub m2_5sd: PriceRatioPattern,
    pub m3sd: PriceRatioPattern,
}

impl SeriesTree_Cohorts_Utxo_All_Realized_Price_StdDev_1y {
    pub fn new(client: Arc<BrkClientBase>, base_path: String) -> Self {
        Self {
            sd: SeriesPattern1::new(client.clone(), "realized_price_ratio_sd_1y".to_string()),
            zscore: SeriesPattern1::new(client.clone(), "realized_price_ratio_zscore_1y".to_string()),
            _0sd: CentsSatsUsdPattern::new(client.clone(), "realized_price_0sd_1y".to_string()),
            p0_5sd: PriceRatioPattern::new(client.clone(), "realized_price".to_string(), "p0_5sd_1y".to_string()),
            p1sd: PriceRatioPattern::new(client.clone(), "realized_price".to_string(), "p1sd_1y".to_string()),
            p1_5sd: PriceRatioPattern::new(client.clone(), "realized_price".to_string(), "p1_5sd_1y".to_string()),
            p2sd: PriceRatioPattern::new(client.clone(), "realized_price".to_string(), "p2sd_1y".to_string()),
            p2_5sd: PriceRatioPattern::new(client.clone(), "realized_price".to_string(), "p2_5sd_1y".to_string()),
            p3sd: PriceRatioPattern::new(client.clone(), "realized_price".to_string(), "p3sd_1y".to_string()),
            m0_5sd: PriceRatioPattern::new(client.clone(), "realized_price".to_string(), "m0_5sd_1y".to_string()),
            m1sd: PriceRatioPattern::new(client.clone(), "realized_price".to_string(), "m1sd_1y".to_string()),
            m1_5sd: PriceRatioPattern::new(client.clone(), "realized_price".to_string(), "m1_5sd_1y".to_string()),
            m2sd: PriceRatioPattern::new(client.clone(), "realized_price".to_string(), "m2sd_1y".to_string()),
            m2_5sd: PriceRatioPattern::new(client.clone(), "realized_price".to_string(), "m2_5sd_1y".to_string()),
            m3sd: PriceRatioPattern::new(client.clone(), "realized_price".to_string(), "m3sd_1y".to_string()),
        }
    }
}

/// Series tree node.
pub struct SeriesTree_Cohorts_Utxo_All_Realized_Sopr {
    pub value_destroyed: AverageBlockCumulativeSumPattern<Cents>,
    pub ratio: _1m1w1y24hPattern<StoredF64>,
    pub adjusted: SeriesTree_Cohorts_Utxo_All_Realized_Sopr_Adjusted,
}

impl SeriesTree_Cohorts_Utxo_All_Realized_Sopr {
    pub fn new(client: Arc<BrkClientBase>, base_path: String) -> Self {
        Self {
            value_destroyed: AverageBlockCumulativeSumPattern::new(client.clone(), "value_destroyed".to_string()),
            ratio: _1m1w1y24hPattern::new(client.clone(), "sopr".to_string()),
            adjusted: SeriesTree_Cohorts_Utxo_All_Realized_Sopr_Adjusted::new(client.clone(), format!("{base_path}_adjusted")),
        }
    }
}

/// Series tree node.
pub struct SeriesTree_Cohorts_Utxo_All_Realized_Sopr_Adjusted {
    pub ratio: _1m1w1y24hPattern<StoredF64>,
    pub transfer_volume: AverageBlockCumulativeSumPattern<Cents>,
    pub value_destroyed: AverageBlockCumulativeSumPattern<Cents>,
}

impl SeriesTree_Cohorts_Utxo_All_Realized_Sopr_Adjusted {
    pub fn new(client: Arc<BrkClientBase>, base_path: String) -> Self {
        Self {
            ratio: _1m1w1y24hPattern::new(client.clone(), "asopr".to_string()),
            transfer_volume: AverageBlockCumulativeSumPattern::new(client.clone(), "adj_value_created".to_string()),
            value_destroyed: AverageBlockCumulativeSumPattern::new(client.clone(), "adj_value_destroyed".to_string()),
        }
    }
}

/// Series tree node.
pub struct SeriesTree_Cohorts_Utxo_All_CostBasis {
    pub in_profit: PerPattern,
    pub in_loss: PerPattern,
    pub min: CentsSatsUsdPattern,
    pub max: CentsSatsUsdPattern,
    pub per_coin: Pct05Pct10Pct15Pct20Pct25Pct30Pct35Pct40Pct45Pct50Pct55Pct60Pct65Pct70Pct75Pct80Pct85Pct90Pct95Pattern,
    pub per_dollar: Pct05Pct10Pct15Pct20Pct25Pct30Pct35Pct40Pct45Pct50Pct55Pct60Pct65Pct70Pct75Pct80Pct85Pct90Pct95Pattern,
    pub supply_density: BpsPercentRatioPattern2,
}

impl SeriesTree_Cohorts_Utxo_All_CostBasis {
    pub fn new(client: Arc<BrkClientBase>, base_path: String) -> Self {
        Self {
            in_profit: PerPattern::new(client.clone(), "cost_basis_in_profit_per".to_string()),
            in_loss: PerPattern::new(client.clone(), "cost_basis_in_loss_per".to_string()),
            min: CentsSatsUsdPattern::new(client.clone(), "cost_basis_min".to_string()),
            max: CentsSatsUsdPattern::new(client.clone(), "cost_basis_max".to_string()),
            per_coin: Pct05Pct10Pct15Pct20Pct25Pct30Pct35Pct40Pct45Pct50Pct55Pct60Pct65Pct70Pct75Pct80Pct85Pct90Pct95Pattern::new(client.clone(), "cost_basis_per_coin".to_string()),
            per_dollar: Pct05Pct10Pct15Pct20Pct25Pct30Pct35Pct40Pct45Pct50Pct55Pct60Pct65Pct70Pct75Pct80Pct85Pct90Pct95Pattern::new(client.clone(), "cost_basis_per_dollar".to_string()),
            supply_density: BpsPercentRatioPattern2::new(client.clone(), "supply_density".to_string()),
        }
    }
}

/// Series tree node.
pub struct SeriesTree_Cohorts_Utxo_All_Unrealized {
    pub nupl: BpsRatioPattern,
    pub profit: SeriesTree_Cohorts_Utxo_All_Unrealized_Profit,
    pub loss: SeriesTree_Cohorts_Utxo_All_Unrealized_Loss,
    pub net_pnl: SeriesTree_Cohorts_Utxo_All_Unrealized_NetPnl,
    pub gross_pnl: CentsUsdPattern3,
    pub invested_capital: InPattern2,
    pub capitalized_cap_in_profit_raw: SeriesPattern18<CentsSquaredSats>,
    pub capitalized_cap_in_loss_raw: SeriesPattern18<CentsSquaredSats>,
    pub sentiment: SeriesTree_Cohorts_Utxo_All_Unrealized_Sentiment,
}

impl SeriesTree_Cohorts_Utxo_All_Unrealized {
    pub fn new(client: Arc<BrkClientBase>, base_path: String) -> Self {
        Self {
            nupl: BpsRatioPattern::new(client.clone(), "nupl".to_string()),
            profit: SeriesTree_Cohorts_Utxo_All_Unrealized_Profit::new(client.clone(), format!("{base_path}_profit")),
            loss: SeriesTree_Cohorts_Utxo_All_Unrealized_Loss::new(client.clone(), format!("{base_path}_loss")),
            net_pnl: SeriesTree_Cohorts_Utxo_All_Unrealized_NetPnl::new(client.clone(), format!("{base_path}_net_pnl")),
            gross_pnl: CentsUsdPattern3::new(client.clone(), "unrealized_gross_pnl".to_string()),
            invested_capital: InPattern2::new(client.clone(), "invested_capital_in".to_string()),
            capitalized_cap_in_profit_raw: SeriesPattern18::new(client.clone(), "capitalized_cap_in_profit_raw".to_string()),
            capitalized_cap_in_loss_raw: SeriesPattern18::new(client.clone(), "capitalized_cap_in_loss_raw".to_string()),
            sentiment: SeriesTree_Cohorts_Utxo_All_Unrealized_Sentiment::new(client.clone(), format!("{base_path}_sentiment")),
        }
    }
}

/// Series tree node.
pub struct SeriesTree_Cohorts_Utxo_All_Unrealized_Profit {
    pub usd: SeriesPattern1<Dollars>,
    pub cents: SeriesPattern1<Cents>,
    pub to_mcap: BpsPercentRatioPattern2,
    pub to_own_gross_pnl: BpsPercentRatioPattern2,
}

impl SeriesTree_Cohorts_Utxo_All_Unrealized_Profit {
    pub fn new(client: Arc<BrkClientBase>, base_path: String) -> Self {
        Self {
            usd: SeriesPattern1::new(client.clone(), "unrealized_profit".to_string()),
            cents: SeriesPattern1::new(client.clone(), "unrealized_profit_cents".to_string()),
            to_mcap: BpsPercentRatioPattern2::new(client.clone(), "unrealized_profit_to_mcap".to_string()),
            to_own_gross_pnl: BpsPercentRatioPattern2::new(client.clone(), "unrealized_profit_to_own_gross_pnl".to_string()),
        }
    }
}

/// Series tree node.
pub struct SeriesTree_Cohorts_Utxo_All_Unrealized_Loss {
    pub usd: SeriesPattern1<Dollars>,
    pub cents: SeriesPattern1<Cents>,
    pub negative: SeriesPattern1<Dollars>,
    pub to_mcap: BpsPercentRatioPattern2,
    pub to_own_gross_pnl: BpsPercentRatioPattern2,
}

impl SeriesTree_Cohorts_Utxo_All_Unrealized_Loss {
    pub fn new(client: Arc<BrkClientBase>, base_path: String) -> Self {
        Self {
            usd: SeriesPattern1::new(client.clone(), "unrealized_loss".to_string()),
            cents: SeriesPattern1::new(client.clone(), "unrealized_loss_cents".to_string()),
            negative: SeriesPattern1::new(client.clone(), "unrealized_loss_neg".to_string()),
            to_mcap: BpsPercentRatioPattern2::new(client.clone(), "unrealized_loss_to_mcap".to_string()),
            to_own_gross_pnl: BpsPercentRatioPattern2::new(client.clone(), "unrealized_loss_to_own_gross_pnl".to_string()),
        }
    }
}

/// Series tree node.
pub struct SeriesTree_Cohorts_Utxo_All_Unrealized_NetPnl {
    pub usd: SeriesPattern1<Dollars>,
    pub cents: SeriesPattern1<CentsSigned>,
    pub to_own_gross_pnl: BpsPercentRatioPattern,
}

impl SeriesTree_Cohorts_Utxo_All_Unrealized_NetPnl {
    pub fn new(client: Arc<BrkClientBase>, base_path: String) -> Self {
        Self {
            usd: SeriesPattern1::new(client.clone(), "net_unrealized_pnl".to_string()),
            cents: SeriesPattern1::new(client.clone(), "net_unrealized_pnl_cents".to_string()),
            to_own_gross_pnl: BpsPercentRatioPattern::new(client.clone(), "net_unrealized_pnl_to_own_gross_pnl".to_string()),
        }
    }
}

/// Series tree node.
pub struct SeriesTree_Cohorts_Utxo_All_Unrealized_Sentiment {
    pub pain_index: CentsUsdPattern3,
    pub greed_index: CentsUsdPattern3,
    pub net: CentsUsdPattern,
}

impl SeriesTree_Cohorts_Utxo_All_Unrealized_Sentiment {
    pub fn new(client: Arc<BrkClientBase>, base_path: String) -> Self {
        Self {
            pain_index: CentsUsdPattern3::new(client.clone(), "pain_index".to_string()),
            greed_index: CentsUsdPattern3::new(client.clone(), "greed_index".to_string()),
            net: CentsUsdPattern::new(client.clone(), "net_sentiment".to_string()),
        }
    }
}

/// Series tree node.
pub struct SeriesTree_Cohorts_Utxo_Sth {
    pub supply: DeltaDominanceHalfInTotalPattern2,
    pub outputs: SpendingSpentUnspentPattern,
    pub activity: CoindaysCoinyearsDormancyTransferPattern,
    pub realized: SeriesTree_Cohorts_Utxo_Sth_Realized,
    pub cost_basis: InMaxMinPerSupplyPattern,
    pub unrealized: CapitalizedGrossInvestedLossNetNuplProfitSentimentPattern2,
    pub invested_capital: InPattern,
}

impl SeriesTree_Cohorts_Utxo_Sth {
    pub fn new(client: Arc<BrkClientBase>, base_path: String) -> Self {
        Self {
            supply: DeltaDominanceHalfInTotalPattern2::new(client.clone(), "sth_supply".to_string()),
            outputs: SpendingSpentUnspentPattern::new(client.clone(), "sth".to_string()),
            activity: CoindaysCoinyearsDormancyTransferPattern::new(client.clone(), "sth".to_string()),
            realized: SeriesTree_Cohorts_Utxo_Sth_Realized::new(client.clone(), format!("{base_path}_realized")),
            cost_basis: InMaxMinPerSupplyPattern::new(client.clone(), "sth".to_string()),
            unrealized: CapitalizedGrossInvestedLossNetNuplProfitSentimentPattern2::new(client.clone(), "sth".to_string()),
            invested_capital: InPattern::new(client.clone(), "sth_invested_capital_in".to_string()),
        }
    }
}

/// Series tree node.
pub struct SeriesTree_Cohorts_Utxo_Sth_Realized {
    pub cap: CentsDeltaToUsdPattern,
    pub profit: BlockCumulativeSumPattern,
    pub loss: BlockCumulativeNegativeSumPattern,
    pub price: SeriesTree_Cohorts_Utxo_Sth_Realized_Price,
    pub mvrv: SeriesPattern1<StoredF32>,
    pub net_pnl: BlockChangeCumulativeDeltaSumPattern,
    pub sopr: AdjustedRatioValuePattern,
    pub gross_pnl: BlockCumulativeSumPattern,
    pub sell_side_risk_ratio: _1m1w1y24hPattern8,
    pub peak_regret: BlockCumulativeSumPattern,
    pub capitalized: PricePattern,
    pub profit_to_loss_ratio: _1m1w1y24hPattern<StoredF64>,
}

impl SeriesTree_Cohorts_Utxo_Sth_Realized {
    pub fn new(client: Arc<BrkClientBase>, base_path: String) -> Self {
        Self {
            cap: CentsDeltaToUsdPattern::new(client.clone(), "sth_realized_cap".to_string()),
            profit: BlockCumulativeSumPattern::new(client.clone(), "sth_realized_profit".to_string()),
            loss: BlockCumulativeNegativeSumPattern::new(client.clone(), "sth_realized_loss".to_string()),
            price: SeriesTree_Cohorts_Utxo_Sth_Realized_Price::new(client.clone(), format!("{base_path}_price")),
            mvrv: SeriesPattern1::new(client.clone(), "sth_mvrv".to_string()),
            net_pnl: BlockChangeCumulativeDeltaSumPattern::new(client.clone(), "sth_net".to_string()),
            sopr: AdjustedRatioValuePattern::new(client.clone(), "sth".to_string()),
            gross_pnl: BlockCumulativeSumPattern::new(client.clone(), "sth_realized_gross_pnl".to_string()),
            sell_side_risk_ratio: _1m1w1y24hPattern8::new(client.clone(), "sth_sell_side_risk_ratio".to_string()),
            peak_regret: BlockCumulativeSumPattern::new(client.clone(), "sth_realized_peak_regret".to_string()),
            capitalized: PricePattern::new(client.clone(), "sth_capitalized_price".to_string()),
            profit_to_loss_ratio: _1m1w1y24hPattern::new(client.clone(), "sth_realized_profit_to_loss_ratio".to_string()),
        }
    }
}

/// Series tree node.
pub struct SeriesTree_Cohorts_Utxo_Sth_Realized_Price {
    pub usd: SeriesPattern1<Dollars>,
    pub cents: SeriesPattern1<Cents>,
    pub sats: SeriesPattern1<SatsFract>,
    pub bps: SeriesPattern1<BasisPoints32>,
    pub ratio: SeriesPattern1<StoredF32>,
    pub percentiles: Pct0Pct1Pct2Pct5Pct95Pct98Pct99Pattern,
    pub sma: _1m1w1y2y4yAllPattern,
    pub std_dev: SeriesTree_Cohorts_Utxo_Sth_Realized_Price_StdDev,
}

impl SeriesTree_Cohorts_Utxo_Sth_Realized_Price {
    pub fn new(client: Arc<BrkClientBase>, base_path: String) -> Self {
        Self {
            usd: SeriesPattern1::new(client.clone(), "sth_realized_price".to_string()),
            cents: SeriesPattern1::new(client.clone(), "sth_realized_price_cents".to_string()),
            sats: SeriesPattern1::new(client.clone(), "sth_realized_price_sats".to_string()),
            bps: SeriesPattern1::new(client.clone(), "sth_realized_price_ratio_bps".to_string()),
            ratio: SeriesPattern1::new(client.clone(), "sth_realized_price_ratio".to_string()),
            percentiles: Pct0Pct1Pct2Pct5Pct95Pct98Pct99Pattern::new(client.clone(), "sth_realized_price".to_string()),
            sma: _1m1w1y2y4yAllPattern::new(client.clone(), "sth_realized_price_ratio_sma".to_string()),
            std_dev: SeriesTree_Cohorts_Utxo_Sth_Realized_Price_StdDev::new(client.clone(), format!("{base_path}_std_dev")),
        }
    }
}

/// Series tree node.
pub struct SeriesTree_Cohorts_Utxo_Sth_Realized_Price_StdDev {
    pub all: SeriesTree_Cohorts_Utxo_Sth_Realized_Price_StdDev_All,
    pub _4y: SeriesTree_Cohorts_Utxo_Sth_Realized_Price_StdDev_4y,
    pub _2y: SeriesTree_Cohorts_Utxo_Sth_Realized_Price_StdDev_2y,
    pub _1y: SeriesTree_Cohorts_Utxo_Sth_Realized_Price_StdDev_1y,
}

impl SeriesTree_Cohorts_Utxo_Sth_Realized_Price_StdDev {
    pub fn new(client: Arc<BrkClientBase>, base_path: String) -> Self {
        Self {
            all: SeriesTree_Cohorts_Utxo_Sth_Realized_Price_StdDev_All::new(client.clone(), format!("{base_path}_all")),
            _4y: SeriesTree_Cohorts_Utxo_Sth_Realized_Price_StdDev_4y::new(client.clone(), format!("{base_path}_4y")),
            _2y: SeriesTree_Cohorts_Utxo_Sth_Realized_Price_StdDev_2y::new(client.clone(), format!("{base_path}_2y")),
            _1y: SeriesTree_Cohorts_Utxo_Sth_Realized_Price_StdDev_1y::new(client.clone(), format!("{base_path}_1y")),
        }
    }
}

/// Series tree node.
pub struct SeriesTree_Cohorts_Utxo_Sth_Realized_Price_StdDev_All {
    pub sd: SeriesPattern1<StoredF32>,
    pub zscore: SeriesPattern1<StoredF32>,
    pub _0sd: CentsSatsUsdPattern,
    pub p0_5sd: PriceRatioPattern,
    pub p1sd: PriceRatioPattern,
    pub p1_5sd: PriceRatioPattern,
    pub p2sd: PriceRatioPattern,
    pub p2_5sd: PriceRatioPattern,
    pub p3sd: PriceRatioPattern,
    pub m0_5sd: PriceRatioPattern,
    pub m1sd: PriceRatioPattern,
    pub m1_5sd: PriceRatioPattern,
    pub m2sd: PriceRatioPattern,
    pub m2_5sd: PriceRatioPattern,
    pub m3sd: PriceRatioPattern,
}

impl SeriesTree_Cohorts_Utxo_Sth_Realized_Price_StdDev_All {
    pub fn new(client: Arc<BrkClientBase>, base_path: String) -> Self {
        Self {
            sd: SeriesPattern1::new(client.clone(), "sth_realized_price_ratio_sd".to_string()),
            zscore: SeriesPattern1::new(client.clone(), "sth_realized_price_ratio_zscore".to_string()),
            _0sd: CentsSatsUsdPattern::new(client.clone(), "sth_realized_price_0sd".to_string()),
            p0_5sd: PriceRatioPattern::new(client.clone(), "sth_realized_price".to_string(), "p0_5sd".to_string()),
            p1sd: PriceRatioPattern::new(client.clone(), "sth_realized_price".to_string(), "p1sd".to_string()),
            p1_5sd: PriceRatioPattern::new(client.clone(), "sth_realized_price".to_string(), "p1_5sd".to_string()),
            p2sd: PriceRatioPattern::new(client.clone(), "sth_realized_price".to_string(), "p2sd".to_string()),
            p2_5sd: PriceRatioPattern::new(client.clone(), "sth_realized_price".to_string(), "p2_5sd".to_string()),
            p3sd: PriceRatioPattern::new(client.clone(), "sth_realized_price".to_string(), "p3sd".to_string()),
            m0_5sd: PriceRatioPattern::new(client.clone(), "sth_realized_price".to_string(), "m0_5sd".to_string()),
            m1sd: PriceRatioPattern::new(client.clone(), "sth_realized_price".to_string(), "m1sd".to_string()),
            m1_5sd: PriceRatioPattern::new(client.clone(), "sth_realized_price".to_string(), "m1_5sd".to_string()),
            m2sd: PriceRatioPattern::new(client.clone(), "sth_realized_price".to_string(), "m2sd".to_string()),
            m2_5sd: PriceRatioPattern::new(client.clone(), "sth_realized_price".to_string(), "m2_5sd".to_string()),
            m3sd: PriceRatioPattern::new(client.clone(), "sth_realized_price".to_string(), "m3sd".to_string()),
        }
    }
}

/// Series tree node.
pub struct SeriesTree_Cohorts_Utxo_Sth_Realized_Price_StdDev_4y {
    pub sd: SeriesPattern1<StoredF32>,
    pub zscore: SeriesPattern1<StoredF32>,
    pub _0sd: CentsSatsUsdPattern,
    pub p0_5sd: PriceRatioPattern,
    pub p1sd: PriceRatioPattern,
    pub p1_5sd: PriceRatioPattern,
    pub p2sd: PriceRatioPattern,
    pub p2_5sd: PriceRatioPattern,
    pub p3sd: PriceRatioPattern,
    pub m0_5sd: PriceRatioPattern,
    pub m1sd: PriceRatioPattern,
    pub m1_5sd: PriceRatioPattern,
    pub m2sd: PriceRatioPattern,
    pub m2_5sd: PriceRatioPattern,
    pub m3sd: PriceRatioPattern,
}

impl SeriesTree_Cohorts_Utxo_Sth_Realized_Price_StdDev_4y {
    pub fn new(client: Arc<BrkClientBase>, base_path: String) -> Self {
        Self {
            sd: SeriesPattern1::new(client.clone(), "sth_realized_price_ratio_sd_4y".to_string()),
            zscore: SeriesPattern1::new(client.clone(), "sth_realized_price_ratio_zscore_4y".to_string()),
            _0sd: CentsSatsUsdPattern::new(client.clone(), "sth_realized_price_0sd_4y".to_string()),
            p0_5sd: PriceRatioPattern::new(client.clone(), "sth_realized_price".to_string(), "p0_5sd_4y".to_string()),
            p1sd: PriceRatioPattern::new(client.clone(), "sth_realized_price".to_string(), "p1sd_4y".to_string()),
            p1_5sd: PriceRatioPattern::new(client.clone(), "sth_realized_price".to_string(), "p1_5sd_4y".to_string()),
            p2sd: PriceRatioPattern::new(client.clone(), "sth_realized_price".to_string(), "p2sd_4y".to_string()),
            p2_5sd: PriceRatioPattern::new(client.clone(), "sth_realized_price".to_string(), "p2_5sd_4y".to_string()),
            p3sd: PriceRatioPattern::new(client.clone(), "sth_realized_price".to_string(), "p3sd_4y".to_string()),
            m0_5sd: PriceRatioPattern::new(client.clone(), "sth_realized_price".to_string(), "m0_5sd_4y".to_string()),
            m1sd: PriceRatioPattern::new(client.clone(), "sth_realized_price".to_string(), "m1sd_4y".to_string()),
            m1_5sd: PriceRatioPattern::new(client.clone(), "sth_realized_price".to_string(), "m1_5sd_4y".to_string()),
            m2sd: PriceRatioPattern::new(client.clone(), "sth_realized_price".to_string(), "m2sd_4y".to_string()),
            m2_5sd: PriceRatioPattern::new(client.clone(), "sth_realized_price".to_string(), "m2_5sd_4y".to_string()),
            m3sd: PriceRatioPattern::new(client.clone(), "sth_realized_price".to_string(), "m3sd_4y".to_string()),
        }
    }
}

/// Series tree node.
pub struct SeriesTree_Cohorts_Utxo_Sth_Realized_Price_StdDev_2y {
    pub sd: SeriesPattern1<StoredF32>,
    pub zscore: SeriesPattern1<StoredF32>,
    pub _0sd: CentsSatsUsdPattern,
    pub p0_5sd: PriceRatioPattern,
    pub p1sd: PriceRatioPattern,
    pub p1_5sd: PriceRatioPattern,
    pub p2sd: PriceRatioPattern,
    pub p2_5sd: PriceRatioPattern,
    pub p3sd: PriceRatioPattern,
    pub m0_5sd: PriceRatioPattern,
    pub m1sd: PriceRatioPattern,
    pub m1_5sd: PriceRatioPattern,
    pub m2sd: PriceRatioPattern,
    pub m2_5sd: PriceRatioPattern,
    pub m3sd: PriceRatioPattern,
}

impl SeriesTree_Cohorts_Utxo_Sth_Realized_Price_StdDev_2y {
    pub fn new(client: Arc<BrkClientBase>, base_path: String) -> Self {
        Self {
            sd: SeriesPattern1::new(client.clone(), "sth_realized_price_ratio_sd_2y".to_string()),
            zscore: SeriesPattern1::new(client.clone(), "sth_realized_price_ratio_zscore_2y".to_string()),
            _0sd: CentsSatsUsdPattern::new(client.clone(), "sth_realized_price_0sd_2y".to_string()),
            p0_5sd: PriceRatioPattern::new(client.clone(), "sth_realized_price".to_string(), "p0_5sd_2y".to_string()),
            p1sd: PriceRatioPattern::new(client.clone(), "sth_realized_price".to_string(), "p1sd_2y".to_string()),
            p1_5sd: PriceRatioPattern::new(client.clone(), "sth_realized_price".to_string(), "p1_5sd_2y".to_string()),
            p2sd: PriceRatioPattern::new(client.clone(), "sth_realized_price".to_string(), "p2sd_2y".to_string()),
            p2_5sd: PriceRatioPattern::new(client.clone(), "sth_realized_price".to_string(), "p2_5sd_2y".to_string()),
            p3sd: PriceRatioPattern::new(client.clone(), "sth_realized_price".to_string(), "p3sd_2y".to_string()),
            m0_5sd: PriceRatioPattern::new(client.clone(), "sth_realized_price".to_string(), "m0_5sd_2y".to_string()),
            m1sd: PriceRatioPattern::new(client.clone(), "sth_realized_price".to_string(), "m1sd_2y".to_string()),
            m1_5sd: PriceRatioPattern::new(client.clone(), "sth_realized_price".to_string(), "m1_5sd_2y".to_string()),
            m2sd: PriceRatioPattern::new(client.clone(), "sth_realized_price".to_string(), "m2sd_2y".to_string()),
            m2_5sd: PriceRatioPattern::new(client.clone(), "sth_realized_price".to_string(), "m2_5sd_2y".to_string()),
            m3sd: PriceRatioPattern::new(client.clone(), "sth_realized_price".to_string(), "m3sd_2y".to_string()),
        }
    }
}

/// Series tree node.
pub struct SeriesTree_Cohorts_Utxo_Sth_Realized_Price_StdDev_1y {
    pub sd: SeriesPattern1<StoredF32>,
    pub zscore: SeriesPattern1<StoredF32>,
    pub _0sd: CentsSatsUsdPattern,
    pub p0_5sd: PriceRatioPattern,
    pub p1sd: PriceRatioPattern,
    pub p1_5sd: PriceRatioPattern,
    pub p2sd: PriceRatioPattern,
    pub p2_5sd: PriceRatioPattern,
    pub p3sd: PriceRatioPattern,
    pub m0_5sd: PriceRatioPattern,
    pub m1sd: PriceRatioPattern,
    pub m1_5sd: PriceRatioPattern,
    pub m2sd: PriceRatioPattern,
    pub m2_5sd: PriceRatioPattern,
    pub m3sd: PriceRatioPattern,
}

impl SeriesTree_Cohorts_Utxo_Sth_Realized_Price_StdDev_1y {
    pub fn new(client: Arc<BrkClientBase>, base_path: String) -> Self {
        Self {
            sd: SeriesPattern1::new(client.clone(), "sth_realized_price_ratio_sd_1y".to_string()),
            zscore: SeriesPattern1::new(client.clone(), "sth_realized_price_ratio_zscore_1y".to_string()),
            _0sd: CentsSatsUsdPattern::new(client.clone(), "sth_realized_price_0sd_1y".to_string()),
            p0_5sd: PriceRatioPattern::new(client.clone(), "sth_realized_price".to_string(), "p0_5sd_1y".to_string()),
            p1sd: PriceRatioPattern::new(client.clone(), "sth_realized_price".to_string(), "p1sd_1y".to_string()),
            p1_5sd: PriceRatioPattern::new(client.clone(), "sth_realized_price".to_string(), "p1_5sd_1y".to_string()),
            p2sd: PriceRatioPattern::new(client.clone(), "sth_realized_price".to_string(), "p2sd_1y".to_string()),
            p2_5sd: PriceRatioPattern::new(client.clone(), "sth_realized_price".to_string(), "p2_5sd_1y".to_string()),
            p3sd: PriceRatioPattern::new(client.clone(), "sth_realized_price".to_string(), "p3sd_1y".to_string()),
            m0_5sd: PriceRatioPattern::new(client.clone(), "sth_realized_price".to_string(), "m0_5sd_1y".to_string()),
            m1sd: PriceRatioPattern::new(client.clone(), "sth_realized_price".to_string(), "m1sd_1y".to_string()),
            m1_5sd: PriceRatioPattern::new(client.clone(), "sth_realized_price".to_string(), "m1_5sd_1y".to_string()),
            m2sd: PriceRatioPattern::new(client.clone(), "sth_realized_price".to_string(), "m2sd_1y".to_string()),
            m2_5sd: PriceRatioPattern::new(client.clone(), "sth_realized_price".to_string(), "m2_5sd_1y".to_string()),
            m3sd: PriceRatioPattern::new(client.clone(), "sth_realized_price".to_string(), "m3sd_1y".to_string()),
        }
    }
}

/// Series tree node.
pub struct SeriesTree_Cohorts_Utxo_Lth {
    pub supply: DeltaDominanceHalfInTotalPattern2,
    pub outputs: SpendingSpentUnspentPattern,
    pub activity: CoindaysCoinyearsDormancyTransferPattern,
    pub realized: SeriesTree_Cohorts_Utxo_Lth_Realized,
    pub cost_basis: InMaxMinPerSupplyPattern,
    pub unrealized: CapitalizedGrossInvestedLossNetNuplProfitSentimentPattern2,
    pub invested_capital: InPattern,
}

impl SeriesTree_Cohorts_Utxo_Lth {
    pub fn new(client: Arc<BrkClientBase>, base_path: String) -> Self {
        Self {
            supply: DeltaDominanceHalfInTotalPattern2::new(client.clone(), "lth_supply".to_string()),
            outputs: SpendingSpentUnspentPattern::new(client.clone(), "lth".to_string()),
            activity: CoindaysCoinyearsDormancyTransferPattern::new(client.clone(), "lth".to_string()),
            realized: SeriesTree_Cohorts_Utxo_Lth_Realized::new(client.clone(), format!("{base_path}_realized")),
            cost_basis: InMaxMinPerSupplyPattern::new(client.clone(), "lth".to_string()),
            unrealized: CapitalizedGrossInvestedLossNetNuplProfitSentimentPattern2::new(client.clone(), "lth".to_string()),
            invested_capital: InPattern::new(client.clone(), "lth_invested_capital_in".to_string()),
        }
    }
}

/// Series tree node.
pub struct SeriesTree_Cohorts_Utxo_Lth_Realized {
    pub cap: CentsDeltaToUsdPattern,
    pub profit: BlockCumulativeSumPattern,
    pub loss: BlockCumulativeNegativeSumPattern,
    pub price: SeriesTree_Cohorts_Utxo_Lth_Realized_Price,
    pub mvrv: SeriesPattern1<StoredF32>,
    pub net_pnl: BlockChangeCumulativeDeltaSumPattern,
    pub sopr: RatioValuePattern2,
    pub gross_pnl: BlockCumulativeSumPattern,
    pub sell_side_risk_ratio: _1m1w1y24hPattern8,
    pub peak_regret: BlockCumulativeSumPattern,
    pub capitalized: PricePattern,
    pub profit_to_loss_ratio: _1m1w1y24hPattern<StoredF64>,
}

impl SeriesTree_Cohorts_Utxo_Lth_Realized {
    pub fn new(client: Arc<BrkClientBase>, base_path: String) -> Self {
        Self {
            cap: CentsDeltaToUsdPattern::new(client.clone(), "lth_realized_cap".to_string()),
            profit: BlockCumulativeSumPattern::new(client.clone(), "lth_realized_profit".to_string()),
            loss: BlockCumulativeNegativeSumPattern::new(client.clone(), "lth_realized_loss".to_string()),
            price: SeriesTree_Cohorts_Utxo_Lth_Realized_Price::new(client.clone(), format!("{base_path}_price")),
            mvrv: SeriesPattern1::new(client.clone(), "lth_mvrv".to_string()),
            net_pnl: BlockChangeCumulativeDeltaSumPattern::new(client.clone(), "lth_net".to_string()),
            sopr: RatioValuePattern2::new(client.clone(), "lth".to_string()),
            gross_pnl: BlockCumulativeSumPattern::new(client.clone(), "lth_realized_gross_pnl".to_string()),
            sell_side_risk_ratio: _1m1w1y24hPattern8::new(client.clone(), "lth_sell_side_risk_ratio".to_string()),
            peak_regret: BlockCumulativeSumPattern::new(client.clone(), "lth_realized_peak_regret".to_string()),
            capitalized: PricePattern::new(client.clone(), "lth_capitalized_price".to_string()),
            profit_to_loss_ratio: _1m1w1y24hPattern::new(client.clone(), "lth_realized_profit_to_loss_ratio".to_string()),
        }
    }
}

/// Series tree node.
pub struct SeriesTree_Cohorts_Utxo_Lth_Realized_Price {
    pub usd: SeriesPattern1<Dollars>,
    pub cents: SeriesPattern1<Cents>,
    pub sats: SeriesPattern1<SatsFract>,
    pub bps: SeriesPattern1<BasisPoints32>,
    pub ratio: SeriesPattern1<StoredF32>,
    pub percentiles: Pct0Pct1Pct2Pct5Pct95Pct98Pct99Pattern,
    pub sma: _1m1w1y2y4yAllPattern,
    pub std_dev: SeriesTree_Cohorts_Utxo_Lth_Realized_Price_StdDev,
}

impl SeriesTree_Cohorts_Utxo_Lth_Realized_Price {
    pub fn new(client: Arc<BrkClientBase>, base_path: String) -> Self {
        Self {
            usd: SeriesPattern1::new(client.clone(), "lth_realized_price".to_string()),
            cents: SeriesPattern1::new(client.clone(), "lth_realized_price_cents".to_string()),
            sats: SeriesPattern1::new(client.clone(), "lth_realized_price_sats".to_string()),
            bps: SeriesPattern1::new(client.clone(), "lth_realized_price_ratio_bps".to_string()),
            ratio: SeriesPattern1::new(client.clone(), "lth_realized_price_ratio".to_string()),
            percentiles: Pct0Pct1Pct2Pct5Pct95Pct98Pct99Pattern::new(client.clone(), "lth_realized_price".to_string()),
            sma: _1m1w1y2y4yAllPattern::new(client.clone(), "lth_realized_price_ratio_sma".to_string()),
            std_dev: SeriesTree_Cohorts_Utxo_Lth_Realized_Price_StdDev::new(client.clone(), format!("{base_path}_std_dev")),
        }
    }
}

/// Series tree node.
pub struct SeriesTree_Cohorts_Utxo_Lth_Realized_Price_StdDev {
    pub all: SeriesTree_Cohorts_Utxo_Lth_Realized_Price_StdDev_All,
    pub _4y: SeriesTree_Cohorts_Utxo_Lth_Realized_Price_StdDev_4y,
    pub _2y: SeriesTree_Cohorts_Utxo_Lth_Realized_Price_StdDev_2y,
    pub _1y: SeriesTree_Cohorts_Utxo_Lth_Realized_Price_StdDev_1y,
}

impl SeriesTree_Cohorts_Utxo_Lth_Realized_Price_StdDev {
    pub fn new(client: Arc<BrkClientBase>, base_path: String) -> Self {
        Self {
            all: SeriesTree_Cohorts_Utxo_Lth_Realized_Price_StdDev_All::new(client.clone(), format!("{base_path}_all")),
            _4y: SeriesTree_Cohorts_Utxo_Lth_Realized_Price_StdDev_4y::new(client.clone(), format!("{base_path}_4y")),
            _2y: SeriesTree_Cohorts_Utxo_Lth_Realized_Price_StdDev_2y::new(client.clone(), format!("{base_path}_2y")),
            _1y: SeriesTree_Cohorts_Utxo_Lth_Realized_Price_StdDev_1y::new(client.clone(), format!("{base_path}_1y")),
        }
    }
}

/// Series tree node.
pub struct SeriesTree_Cohorts_Utxo_Lth_Realized_Price_StdDev_All {
    pub sd: SeriesPattern1<StoredF32>,
    pub zscore: SeriesPattern1<StoredF32>,
    pub _0sd: CentsSatsUsdPattern,
    pub p0_5sd: PriceRatioPattern,
    pub p1sd: PriceRatioPattern,
    pub p1_5sd: PriceRatioPattern,
    pub p2sd: PriceRatioPattern,
    pub p2_5sd: PriceRatioPattern,
    pub p3sd: PriceRatioPattern,
    pub m0_5sd: PriceRatioPattern,
    pub m1sd: PriceRatioPattern,
    pub m1_5sd: PriceRatioPattern,
    pub m2sd: PriceRatioPattern,
    pub m2_5sd: PriceRatioPattern,
    pub m3sd: PriceRatioPattern,
}

impl SeriesTree_Cohorts_Utxo_Lth_Realized_Price_StdDev_All {
    pub fn new(client: Arc<BrkClientBase>, base_path: String) -> Self {
        Self {
            sd: SeriesPattern1::new(client.clone(), "lth_realized_price_ratio_sd".to_string()),
            zscore: SeriesPattern1::new(client.clone(), "lth_realized_price_ratio_zscore".to_string()),
            _0sd: CentsSatsUsdPattern::new(client.clone(), "lth_realized_price_0sd".to_string()),
            p0_5sd: PriceRatioPattern::new(client.clone(), "lth_realized_price".to_string(), "p0_5sd".to_string()),
            p1sd: PriceRatioPattern::new(client.clone(), "lth_realized_price".to_string(), "p1sd".to_string()),
            p1_5sd: PriceRatioPattern::new(client.clone(), "lth_realized_price".to_string(), "p1_5sd".to_string()),
            p2sd: PriceRatioPattern::new(client.clone(), "lth_realized_price".to_string(), "p2sd".to_string()),
            p2_5sd: PriceRatioPattern::new(client.clone(), "lth_realized_price".to_string(), "p2_5sd".to_string()),
            p3sd: PriceRatioPattern::new(client.clone(), "lth_realized_price".to_string(), "p3sd".to_string()),
            m0_5sd: PriceRatioPattern::new(client.clone(), "lth_realized_price".to_string(), "m0_5sd".to_string()),
            m1sd: PriceRatioPattern::new(client.clone(), "lth_realized_price".to_string(), "m1sd".to_string()),
            m1_5sd: PriceRatioPattern::new(client.clone(), "lth_realized_price".to_string(), "m1_5sd".to_string()),
            m2sd: PriceRatioPattern::new(client.clone(), "lth_realized_price".to_string(), "m2sd".to_string()),
            m2_5sd: PriceRatioPattern::new(client.clone(), "lth_realized_price".to_string(), "m2_5sd".to_string()),
            m3sd: PriceRatioPattern::new(client.clone(), "lth_realized_price".to_string(), "m3sd".to_string()),
        }
    }
}

/// Series tree node.
pub struct SeriesTree_Cohorts_Utxo_Lth_Realized_Price_StdDev_4y {
    pub sd: SeriesPattern1<StoredF32>,
    pub zscore: SeriesPattern1<StoredF32>,
    pub _0sd: CentsSatsUsdPattern,
    pub p0_5sd: PriceRatioPattern,
    pub p1sd: PriceRatioPattern,
    pub p1_5sd: PriceRatioPattern,
    pub p2sd: PriceRatioPattern,
    pub p2_5sd: PriceRatioPattern,
    pub p3sd: PriceRatioPattern,
    pub m0_5sd: PriceRatioPattern,
    pub m1sd: PriceRatioPattern,
    pub m1_5sd: PriceRatioPattern,
    pub m2sd: PriceRatioPattern,
    pub m2_5sd: PriceRatioPattern,
    pub m3sd: PriceRatioPattern,
}

impl SeriesTree_Cohorts_Utxo_Lth_Realized_Price_StdDev_4y {
    pub fn new(client: Arc<BrkClientBase>, base_path: String) -> Self {
        Self {
            sd: SeriesPattern1::new(client.clone(), "lth_realized_price_ratio_sd_4y".to_string()),
            zscore: SeriesPattern1::new(client.clone(), "lth_realized_price_ratio_zscore_4y".to_string()),
            _0sd: CentsSatsUsdPattern::new(client.clone(), "lth_realized_price_0sd_4y".to_string()),
            p0_5sd: PriceRatioPattern::new(client.clone(), "lth_realized_price".to_string(), "p0_5sd_4y".to_string()),
            p1sd: PriceRatioPattern::new(client.clone(), "lth_realized_price".to_string(), "p1sd_4y".to_string()),
            p1_5sd: PriceRatioPattern::new(client.clone(), "lth_realized_price".to_string(), "p1_5sd_4y".to_string()),
            p2sd: PriceRatioPattern::new(client.clone(), "lth_realized_price".to_string(), "p2sd_4y".to_string()),
            p2_5sd: PriceRatioPattern::new(client.clone(), "lth_realized_price".to_string(), "p2_5sd_4y".to_string()),
            p3sd: PriceRatioPattern::new(client.clone(), "lth_realized_price".to_string(), "p3sd_4y".to_string()),
            m0_5sd: PriceRatioPattern::new(client.clone(), "lth_realized_price".to_string(), "m0_5sd_4y".to_string()),
            m1sd: PriceRatioPattern::new(client.clone(), "lth_realized_price".to_string(), "m1sd_4y".to_string()),
            m1_5sd: PriceRatioPattern::new(client.clone(), "lth_realized_price".to_string(), "m1_5sd_4y".to_string()),
            m2sd: PriceRatioPattern::new(client.clone(), "lth_realized_price".to_string(), "m2sd_4y".to_string()),
            m2_5sd: PriceRatioPattern::new(client.clone(), "lth_realized_price".to_string(), "m2_5sd_4y".to_string()),
            m3sd: PriceRatioPattern::new(client.clone(), "lth_realized_price".to_string(), "m3sd_4y".to_string()),
        }
    }
}

/// Series tree node.
pub struct SeriesTree_Cohorts_Utxo_Lth_Realized_Price_StdDev_2y {
    pub sd: SeriesPattern1<StoredF32>,
    pub zscore: SeriesPattern1<StoredF32>,
    pub _0sd: CentsSatsUsdPattern,
    pub p0_5sd: PriceRatioPattern,
    pub p1sd: PriceRatioPattern,
    pub p1_5sd: PriceRatioPattern,
    pub p2sd: PriceRatioPattern,
    pub p2_5sd: PriceRatioPattern,
    pub p3sd: PriceRatioPattern,
    pub m0_5sd: PriceRatioPattern,
    pub m1sd: PriceRatioPattern,
    pub m1_5sd: PriceRatioPattern,
    pub m2sd: PriceRatioPattern,
    pub m2_5sd: PriceRatioPattern,
    pub m3sd: PriceRatioPattern,
}

impl SeriesTree_Cohorts_Utxo_Lth_Realized_Price_StdDev_2y {
    pub fn new(client: Arc<BrkClientBase>, base_path: String) -> Self {
        Self {
            sd: SeriesPattern1::new(client.clone(), "lth_realized_price_ratio_sd_2y".to_string()),
            zscore: SeriesPattern1::new(client.clone(), "lth_realized_price_ratio_zscore_2y".to_string()),
            _0sd: CentsSatsUsdPattern::new(client.clone(), "lth_realized_price_0sd_2y".to_string()),
            p0_5sd: PriceRatioPattern::new(client.clone(), "lth_realized_price".to_string(), "p0_5sd_2y".to_string()),
            p1sd: PriceRatioPattern::new(client.clone(), "lth_realized_price".to_string(), "p1sd_2y".to_string()),
            p1_5sd: PriceRatioPattern::new(client.clone(), "lth_realized_price".to_string(), "p1_5sd_2y".to_string()),
            p2sd: PriceRatioPattern::new(client.clone(), "lth_realized_price".to_string(), "p2sd_2y".to_string()),
            p2_5sd: PriceRatioPattern::new(client.clone(), "lth_realized_price".to_string(), "p2_5sd_2y".to_string()),
            p3sd: PriceRatioPattern::new(client.clone(), "lth_realized_price".to_string(), "p3sd_2y".to_string()),
            m0_5sd: PriceRatioPattern::new(client.clone(), "lth_realized_price".to_string(), "m0_5sd_2y".to_string()),
            m1sd: PriceRatioPattern::new(client.clone(), "lth_realized_price".to_string(), "m1sd_2y".to_string()),
            m1_5sd: PriceRatioPattern::new(client.clone(), "lth_realized_price".to_string(), "m1_5sd_2y".to_string()),
            m2sd: PriceRatioPattern::new(client.clone(), "lth_realized_price".to_string(), "m2sd_2y".to_string()),
            m2_5sd: PriceRatioPattern::new(client.clone(), "lth_realized_price".to_string(), "m2_5sd_2y".to_string()),
            m3sd: PriceRatioPattern::new(client.clone(), "lth_realized_price".to_string(), "m3sd_2y".to_string()),
        }
    }
}

/// Series tree node.
pub struct SeriesTree_Cohorts_Utxo_Lth_Realized_Price_StdDev_1y {
    pub sd: SeriesPattern1<StoredF32>,
    pub zscore: SeriesPattern1<StoredF32>,
    pub _0sd: CentsSatsUsdPattern,
    pub p0_5sd: PriceRatioPattern,
    pub p1sd: PriceRatioPattern,
    pub p1_5sd: PriceRatioPattern,
    pub p2sd: PriceRatioPattern,
    pub p2_5sd: PriceRatioPattern,
    pub p3sd: PriceRatioPattern,
    pub m0_5sd: PriceRatioPattern,
    pub m1sd: PriceRatioPattern,
    pub m1_5sd: PriceRatioPattern,
    pub m2sd: PriceRatioPattern,
    pub m2_5sd: PriceRatioPattern,
    pub m3sd: PriceRatioPattern,
}

impl SeriesTree_Cohorts_Utxo_Lth_Realized_Price_StdDev_1y {
    pub fn new(client: Arc<BrkClientBase>, base_path: String) -> Self {
        Self {
            sd: SeriesPattern1::new(client.clone(), "lth_realized_price_ratio_sd_1y".to_string()),
            zscore: SeriesPattern1::new(client.clone(), "lth_realized_price_ratio_zscore_1y".to_string()),
            _0sd: CentsSatsUsdPattern::new(client.clone(), "lth_realized_price_0sd_1y".to_string()),
            p0_5sd: PriceRatioPattern::new(client.clone(), "lth_realized_price".to_string(), "p0_5sd_1y".to_string()),
            p1sd: PriceRatioPattern::new(client.clone(), "lth_realized_price".to_string(), "p1sd_1y".to_string()),
            p1_5sd: PriceRatioPattern::new(client.clone(), "lth_realized_price".to_string(), "p1_5sd_1y".to_string()),
            p2sd: PriceRatioPattern::new(client.clone(), "lth_realized_price".to_string(), "p2sd_1y".to_string()),
            p2_5sd: PriceRatioPattern::new(client.clone(), "lth_realized_price".to_string(), "p2_5sd_1y".to_string()),
            p3sd: PriceRatioPattern::new(client.clone(), "lth_realized_price".to_string(), "p3sd_1y".to_string()),
            m0_5sd: PriceRatioPattern::new(client.clone(), "lth_realized_price".to_string(), "m0_5sd_1y".to_string()),
            m1sd: PriceRatioPattern::new(client.clone(), "lth_realized_price".to_string(), "m1sd_1y".to_string()),
            m1_5sd: PriceRatioPattern::new(client.clone(), "lth_realized_price".to_string(), "m1_5sd_1y".to_string()),
            m2sd: PriceRatioPattern::new(client.clone(), "lth_realized_price".to_string(), "m2sd_1y".to_string()),
            m2_5sd: PriceRatioPattern::new(client.clone(), "lth_realized_price".to_string(), "m2_5sd_1y".to_string()),
            m3sd: PriceRatioPattern::new(client.clone(), "lth_realized_price".to_string(), "m3sd_1y".to_string()),
        }
    }
}

/// Series tree node.
pub struct SeriesTree_Cohorts_Utxo_AgeRange {
    pub under_1h: ActivityOutputsRealizedSupplyUnrealizedPattern,
    pub _1h_to_1d: ActivityOutputsRealizedSupplyUnrealizedPattern,
    pub _1d_to_1w: ActivityOutputsRealizedSupplyUnrealizedPattern,
    pub _1w_to_1m: ActivityOutputsRealizedSupplyUnrealizedPattern,
    pub _1m_to_2m: ActivityOutputsRealizedSupplyUnrealizedPattern,
    pub _2m_to_3m: ActivityOutputsRealizedSupplyUnrealizedPattern,
    pub _3m_to_4m: ActivityOutputsRealizedSupplyUnrealizedPattern,
    pub _4m_to_5m: ActivityOutputsRealizedSupplyUnrealizedPattern,
    pub _5m_to_6m: ActivityOutputsRealizedSupplyUnrealizedPattern,
    pub _6m_to_1y: ActivityOutputsRealizedSupplyUnrealizedPattern,
    pub _1y_to_2y: ActivityOutputsRealizedSupplyUnrealizedPattern,
    pub _2y_to_3y: ActivityOutputsRealizedSupplyUnrealizedPattern,
    pub _3y_to_4y: ActivityOutputsRealizedSupplyUnrealizedPattern,
    pub _4y_to_5y: ActivityOutputsRealizedSupplyUnrealizedPattern,
    pub _5y_to_6y: ActivityOutputsRealizedSupplyUnrealizedPattern,
    pub _6y_to_7y: ActivityOutputsRealizedSupplyUnrealizedPattern,
    pub _7y_to_8y: ActivityOutputsRealizedSupplyUnrealizedPattern,
    pub _8y_to_10y: ActivityOutputsRealizedSupplyUnrealizedPattern,
    pub _10y_to_12y: ActivityOutputsRealizedSupplyUnrealizedPattern,
    pub _12y_to_15y: ActivityOutputsRealizedSupplyUnrealizedPattern,
    pub over_15y: ActivityOutputsRealizedSupplyUnrealizedPattern,
}

impl SeriesTree_Cohorts_Utxo_AgeRange {
    pub fn new(client: Arc<BrkClientBase>, base_path: String) -> Self {
        Self {
            under_1h: ActivityOutputsRealizedSupplyUnrealizedPattern::new(client.clone(), "utxos_under_1h_old".to_string()),
            _1h_to_1d: ActivityOutputsRealizedSupplyUnrealizedPattern::new(client.clone(), "utxos_1h_to_1d_old".to_string()),
            _1d_to_1w: ActivityOutputsRealizedSupplyUnrealizedPattern::new(client.clone(), "utxos_1d_to_1w_old".to_string()),
            _1w_to_1m: ActivityOutputsRealizedSupplyUnrealizedPattern::new(client.clone(), "utxos_1w_to_1m_old".to_string()),
            _1m_to_2m: ActivityOutputsRealizedSupplyUnrealizedPattern::new(client.clone(), "utxos_1m_to_2m_old".to_string()),
            _2m_to_3m: ActivityOutputsRealizedSupplyUnrealizedPattern::new(client.clone(), "utxos_2m_to_3m_old".to_string()),
            _3m_to_4m: ActivityOutputsRealizedSupplyUnrealizedPattern::new(client.clone(), "utxos_3m_to_4m_old".to_string()),
            _4m_to_5m: ActivityOutputsRealizedSupplyUnrealizedPattern::new(client.clone(), "utxos_4m_to_5m_old".to_string()),
            _5m_to_6m: ActivityOutputsRealizedSupplyUnrealizedPattern::new(client.clone(), "utxos_5m_to_6m_old".to_string()),
            _6m_to_1y: ActivityOutputsRealizedSupplyUnrealizedPattern::new(client.clone(), "utxos_6m_to_1y_old".to_string()),
            _1y_to_2y: ActivityOutputsRealizedSupplyUnrealizedPattern::new(client.clone(), "utxos_1y_to_2y_old".to_string()),
            _2y_to_3y: ActivityOutputsRealizedSupplyUnrealizedPattern::new(client.clone(), "utxos_2y_to_3y_old".to_string()),
            _3y_to_4y: ActivityOutputsRealizedSupplyUnrealizedPattern::new(client.clone(), "utxos_3y_to_4y_old".to_string()),
            _4y_to_5y: ActivityOutputsRealizedSupplyUnrealizedPattern::new(client.clone(), "utxos_4y_to_5y_old".to_string()),
            _5y_to_6y: ActivityOutputsRealizedSupplyUnrealizedPattern::new(client.clone(), "utxos_5y_to_6y_old".to_string()),
            _6y_to_7y: ActivityOutputsRealizedSupplyUnrealizedPattern::new(client.clone(), "utxos_6y_to_7y_old".to_string()),
            _7y_to_8y: ActivityOutputsRealizedSupplyUnrealizedPattern::new(client.clone(), "utxos_7y_to_8y_old".to_string()),
            _8y_to_10y: ActivityOutputsRealizedSupplyUnrealizedPattern::new(client.clone(), "utxos_8y_to_10y_old".to_string()),
            _10y_to_12y: ActivityOutputsRealizedSupplyUnrealizedPattern::new(client.clone(), "utxos_10y_to_12y_old".to_string()),
            _12y_to_15y: ActivityOutputsRealizedSupplyUnrealizedPattern::new(client.clone(), "utxos_12y_to_15y_old".to_string()),
            over_15y: ActivityOutputsRealizedSupplyUnrealizedPattern::new(client.clone(), "utxos_over_15y_old".to_string()),
        }
    }
}

/// Series tree node.
pub struct SeriesTree_Cohorts_Utxo_UnderAge {
    pub _1w: ActivityOutputsRealizedSupplyUnrealizedPattern,
    pub _1m: ActivityOutputsRealizedSupplyUnrealizedPattern,
    pub _2m: ActivityOutputsRealizedSupplyUnrealizedPattern,
    pub _3m: ActivityOutputsRealizedSupplyUnrealizedPattern,
    pub _4m: ActivityOutputsRealizedSupplyUnrealizedPattern,
    pub _5m: ActivityOutputsRealizedSupplyUnrealizedPattern,
    pub _6m: ActivityOutputsRealizedSupplyUnrealizedPattern,
    pub _1y: ActivityOutputsRealizedSupplyUnrealizedPattern,
    pub _2y: ActivityOutputsRealizedSupplyUnrealizedPattern,
    pub _3y: ActivityOutputsRealizedSupplyUnrealizedPattern,
    pub _4y: ActivityOutputsRealizedSupplyUnrealizedPattern,
    pub _5y: ActivityOutputsRealizedSupplyUnrealizedPattern,
    pub _6y: ActivityOutputsRealizedSupplyUnrealizedPattern,
    pub _7y: ActivityOutputsRealizedSupplyUnrealizedPattern,
    pub _8y: ActivityOutputsRealizedSupplyUnrealizedPattern,
    pub _10y: ActivityOutputsRealizedSupplyUnrealizedPattern,
    pub _12y: ActivityOutputsRealizedSupplyUnrealizedPattern,
    pub _15y: ActivityOutputsRealizedSupplyUnrealizedPattern,
}

impl SeriesTree_Cohorts_Utxo_UnderAge {
    pub fn new(client: Arc<BrkClientBase>, base_path: String) -> Self {
        Self {
            _1w: ActivityOutputsRealizedSupplyUnrealizedPattern::new(client.clone(), "utxos_under_1w_old".to_string()),
            _1m: ActivityOutputsRealizedSupplyUnrealizedPattern::new(client.clone(), "utxos_under_1m_old".to_string()),
            _2m: ActivityOutputsRealizedSupplyUnrealizedPattern::new(client.clone(), "utxos_under_2m_old".to_string()),
            _3m: ActivityOutputsRealizedSupplyUnrealizedPattern::new(client.clone(), "utxos_under_3m_old".to_string()),
            _4m: ActivityOutputsRealizedSupplyUnrealizedPattern::new(client.clone(), "utxos_under_4m_old".to_string()),
            _5m: ActivityOutputsRealizedSupplyUnrealizedPattern::new(client.clone(), "utxos_under_5m_old".to_string()),
            _6m: ActivityOutputsRealizedSupplyUnrealizedPattern::new(client.clone(), "utxos_under_6m_old".to_string()),
            _1y: ActivityOutputsRealizedSupplyUnrealizedPattern::new(client.clone(), "utxos_under_1y_old".to_string()),
            _2y: ActivityOutputsRealizedSupplyUnrealizedPattern::new(client.clone(), "utxos_under_2y_old".to_string()),
            _3y: ActivityOutputsRealizedSupplyUnrealizedPattern::new(client.clone(), "utxos_under_3y_old".to_string()),
            _4y: ActivityOutputsRealizedSupplyUnrealizedPattern::new(client.clone(), "utxos_under_4y_old".to_string()),
            _5y: ActivityOutputsRealizedSupplyUnrealizedPattern::new(client.clone(), "utxos_under_5y_old".to_string()),
            _6y: ActivityOutputsRealizedSupplyUnrealizedPattern::new(client.clone(), "utxos_under_6y_old".to_string()),
            _7y: ActivityOutputsRealizedSupplyUnrealizedPattern::new(client.clone(), "utxos_under_7y_old".to_string()),
            _8y: ActivityOutputsRealizedSupplyUnrealizedPattern::new(client.clone(), "utxos_under_8y_old".to_string()),
            _10y: ActivityOutputsRealizedSupplyUnrealizedPattern::new(client.clone(), "utxos_under_10y_old".to_string()),
            _12y: ActivityOutputsRealizedSupplyUnrealizedPattern::new(client.clone(), "utxos_under_12y_old".to_string()),
            _15y: ActivityOutputsRealizedSupplyUnrealizedPattern::new(client.clone(), "utxos_under_15y_old".to_string()),
        }
    }
}

/// Series tree node.
pub struct SeriesTree_Cohorts_Utxo_OverAge {
    pub _1d: ActivityOutputsRealizedSupplyUnrealizedPattern,
    pub _1w: ActivityOutputsRealizedSupplyUnrealizedPattern,
    pub _1m: ActivityOutputsRealizedSupplyUnrealizedPattern,
    pub _2m: ActivityOutputsRealizedSupplyUnrealizedPattern,
    pub _3m: ActivityOutputsRealizedSupplyUnrealizedPattern,
    pub _4m: ActivityOutputsRealizedSupplyUnrealizedPattern,
    pub _5m: ActivityOutputsRealizedSupplyUnrealizedPattern,
    pub _6m: ActivityOutputsRealizedSupplyUnrealizedPattern,
    pub _1y: ActivityOutputsRealizedSupplyUnrealizedPattern,
    pub _2y: ActivityOutputsRealizedSupplyUnrealizedPattern,
    pub _3y: ActivityOutputsRealizedSupplyUnrealizedPattern,
    pub _4y: ActivityOutputsRealizedSupplyUnrealizedPattern,
    pub _5y: ActivityOutputsRealizedSupplyUnrealizedPattern,
    pub _6y: ActivityOutputsRealizedSupplyUnrealizedPattern,
    pub _7y: ActivityOutputsRealizedSupplyUnrealizedPattern,
    pub _8y: ActivityOutputsRealizedSupplyUnrealizedPattern,
    pub _10y: ActivityOutputsRealizedSupplyUnrealizedPattern,
    pub _12y: ActivityOutputsRealizedSupplyUnrealizedPattern,
}

impl SeriesTree_Cohorts_Utxo_OverAge {
    pub fn new(client: Arc<BrkClientBase>, base_path: String) -> Self {
        Self {
            _1d: ActivityOutputsRealizedSupplyUnrealizedPattern::new(client.clone(), "utxos_over_1d_old".to_string()),
            _1w: ActivityOutputsRealizedSupplyUnrealizedPattern::new(client.clone(), "utxos_over_1w_old".to_string()),
            _1m: ActivityOutputsRealizedSupplyUnrealizedPattern::new(client.clone(), "utxos_over_1m_old".to_string()),
            _2m: ActivityOutputsRealizedSupplyUnrealizedPattern::new(client.clone(), "utxos_over_2m_old".to_string()),
            _3m: ActivityOutputsRealizedSupplyUnrealizedPattern::new(client.clone(), "utxos_over_3m_old".to_string()),
            _4m: ActivityOutputsRealizedSupplyUnrealizedPattern::new(client.clone(), "utxos_over_4m_old".to_string()),
            _5m: ActivityOutputsRealizedSupplyUnrealizedPattern::new(client.clone(), "utxos_over_5m_old".to_string()),
            _6m: ActivityOutputsRealizedSupplyUnrealizedPattern::new(client.clone(), "utxos_over_6m_old".to_string()),
            _1y: ActivityOutputsRealizedSupplyUnrealizedPattern::new(client.clone(), "utxos_over_1y_old".to_string()),
            _2y: ActivityOutputsRealizedSupplyUnrealizedPattern::new(client.clone(), "utxos_over_2y_old".to_string()),
            _3y: ActivityOutputsRealizedSupplyUnrealizedPattern::new(client.clone(), "utxos_over_3y_old".to_string()),
            _4y: ActivityOutputsRealizedSupplyUnrealizedPattern::new(client.clone(), "utxos_over_4y_old".to_string()),
            _5y: ActivityOutputsRealizedSupplyUnrealizedPattern::new(client.clone(), "utxos_over_5y_old".to_string()),
            _6y: ActivityOutputsRealizedSupplyUnrealizedPattern::new(client.clone(), "utxos_over_6y_old".to_string()),
            _7y: ActivityOutputsRealizedSupplyUnrealizedPattern::new(client.clone(), "utxos_over_7y_old".to_string()),
            _8y: ActivityOutputsRealizedSupplyUnrealizedPattern::new(client.clone(), "utxos_over_8y_old".to_string()),
            _10y: ActivityOutputsRealizedSupplyUnrealizedPattern::new(client.clone(), "utxos_over_10y_old".to_string()),
            _12y: ActivityOutputsRealizedSupplyUnrealizedPattern::new(client.clone(), "utxos_over_12y_old".to_string()),
        }
    }
}

/// Series tree node.
pub struct SeriesTree_Cohorts_Utxo_Epoch {
    pub _0: ActivityOutputsRealizedSupplyUnrealizedPattern,
    pub _1: ActivityOutputsRealizedSupplyUnrealizedPattern,
    pub _2: ActivityOutputsRealizedSupplyUnrealizedPattern,
    pub _3: ActivityOutputsRealizedSupplyUnrealizedPattern,
    pub _4: ActivityOutputsRealizedSupplyUnrealizedPattern,
}

impl SeriesTree_Cohorts_Utxo_Epoch {
    pub fn new(client: Arc<BrkClientBase>, base_path: String) -> Self {
        Self {
            _0: ActivityOutputsRealizedSupplyUnrealizedPattern::new(client.clone(), "epoch_0".to_string()),
            _1: ActivityOutputsRealizedSupplyUnrealizedPattern::new(client.clone(), "epoch_1".to_string()),
            _2: ActivityOutputsRealizedSupplyUnrealizedPattern::new(client.clone(), "epoch_2".to_string()),
            _3: ActivityOutputsRealizedSupplyUnrealizedPattern::new(client.clone(), "epoch_3".to_string()),
            _4: ActivityOutputsRealizedSupplyUnrealizedPattern::new(client.clone(), "epoch_4".to_string()),
        }
    }
}

/// Series tree node.
pub struct SeriesTree_Cohorts_Utxo_Class {
    pub _2009: ActivityOutputsRealizedSupplyUnrealizedPattern,
    pub _2010: ActivityOutputsRealizedSupplyUnrealizedPattern,
    pub _2011: ActivityOutputsRealizedSupplyUnrealizedPattern,
    pub _2012: ActivityOutputsRealizedSupplyUnrealizedPattern,
    pub _2013: ActivityOutputsRealizedSupplyUnrealizedPattern,
    pub _2014: ActivityOutputsRealizedSupplyUnrealizedPattern,
    pub _2015: ActivityOutputsRealizedSupplyUnrealizedPattern,
    pub _2016: ActivityOutputsRealizedSupplyUnrealizedPattern,
    pub _2017: ActivityOutputsRealizedSupplyUnrealizedPattern,
    pub _2018: ActivityOutputsRealizedSupplyUnrealizedPattern,
    pub _2019: ActivityOutputsRealizedSupplyUnrealizedPattern,
    pub _2020: ActivityOutputsRealizedSupplyUnrealizedPattern,
    pub _2021: ActivityOutputsRealizedSupplyUnrealizedPattern,
    pub _2022: ActivityOutputsRealizedSupplyUnrealizedPattern,
    pub _2023: ActivityOutputsRealizedSupplyUnrealizedPattern,
    pub _2024: ActivityOutputsRealizedSupplyUnrealizedPattern,
    pub _2025: ActivityOutputsRealizedSupplyUnrealizedPattern,
    pub _2026: ActivityOutputsRealizedSupplyUnrealizedPattern,
}

impl SeriesTree_Cohorts_Utxo_Class {
    pub fn new(client: Arc<BrkClientBase>, base_path: String) -> Self {
        Self {
            _2009: ActivityOutputsRealizedSupplyUnrealizedPattern::new(client.clone(), "class_2009".to_string()),
            _2010: ActivityOutputsRealizedSupplyUnrealizedPattern::new(client.clone(), "class_2010".to_string()),
            _2011: ActivityOutputsRealizedSupplyUnrealizedPattern::new(client.clone(), "class_2011".to_string()),
            _2012: ActivityOutputsRealizedSupplyUnrealizedPattern::new(client.clone(), "class_2012".to_string()),
            _2013: ActivityOutputsRealizedSupplyUnrealizedPattern::new(client.clone(), "class_2013".to_string()),
            _2014: ActivityOutputsRealizedSupplyUnrealizedPattern::new(client.clone(), "class_2014".to_string()),
            _2015: ActivityOutputsRealizedSupplyUnrealizedPattern::new(client.clone(), "class_2015".to_string()),
            _2016: ActivityOutputsRealizedSupplyUnrealizedPattern::new(client.clone(), "class_2016".to_string()),
            _2017: ActivityOutputsRealizedSupplyUnrealizedPattern::new(client.clone(), "class_2017".to_string()),
            _2018: ActivityOutputsRealizedSupplyUnrealizedPattern::new(client.clone(), "class_2018".to_string()),
            _2019: ActivityOutputsRealizedSupplyUnrealizedPattern::new(client.clone(), "class_2019".to_string()),
            _2020: ActivityOutputsRealizedSupplyUnrealizedPattern::new(client.clone(), "class_2020".to_string()),
            _2021: ActivityOutputsRealizedSupplyUnrealizedPattern::new(client.clone(), "class_2021".to_string()),
            _2022: ActivityOutputsRealizedSupplyUnrealizedPattern::new(client.clone(), "class_2022".to_string()),
            _2023: ActivityOutputsRealizedSupplyUnrealizedPattern::new(client.clone(), "class_2023".to_string()),
            _2024: ActivityOutputsRealizedSupplyUnrealizedPattern::new(client.clone(), "class_2024".to_string()),
            _2025: ActivityOutputsRealizedSupplyUnrealizedPattern::new(client.clone(), "class_2025".to_string()),
            _2026: ActivityOutputsRealizedSupplyUnrealizedPattern::new(client.clone(), "class_2026".to_string()),
        }
    }
}

/// Series tree node.
pub struct SeriesTree_Cohorts_Utxo_Entry {
    pub discount: SeriesTree_Cohorts_Utxo_Entry_Discount,
    pub premium: SeriesTree_Cohorts_Utxo_Entry_Premium,
}

impl SeriesTree_Cohorts_Utxo_Entry {
    pub fn new(client: Arc<BrkClientBase>, base_path: String) -> Self {
        Self {
            discount: SeriesTree_Cohorts_Utxo_Entry_Discount::new(client.clone(), format!("{base_path}_discount")),
            premium: SeriesTree_Cohorts_Utxo_Entry_Premium::new(client.clone(), format!("{base_path}_premium")),
        }
    }
}

/// Series tree node.
pub struct SeriesTree_Cohorts_Utxo_Entry_Discount {
    pub supply: DeltaDominanceHalfInTotalPattern2,
    pub outputs: SpendingSpentUnspentPattern,
    pub activity: CoindaysCoinyearsDormancyTransferPattern,
    pub realized: SeriesTree_Cohorts_Utxo_Entry_Discount_Realized,
    pub cost_basis: InMaxMinPerSupplyPattern,
    pub unrealized: CapitalizedGrossInvestedLossNetNuplProfitSentimentPattern2,
    pub invested_capital: InPattern,
}

impl SeriesTree_Cohorts_Utxo_Entry_Discount {
    pub fn new(client: Arc<BrkClientBase>, base_path: String) -> Self {
        Self {
            supply: DeltaDominanceHalfInTotalPattern2::new(client.clone(), "veteran_supply".to_string()),
            outputs: SpendingSpentUnspentPattern::new(client.clone(), "veteran".to_string()),
            activity: CoindaysCoinyearsDormancyTransferPattern::new(client.clone(), "veteran".to_string()),
            realized: SeriesTree_Cohorts_Utxo_Entry_Discount_Realized::new(client.clone(), format!("{base_path}_realized")),
            cost_basis: InMaxMinPerSupplyPattern::new(client.clone(), "veteran".to_string()),
            unrealized: CapitalizedGrossInvestedLossNetNuplProfitSentimentPattern2::new(client.clone(), "veteran".to_string()),
            invested_capital: InPattern::new(client.clone(), "veteran_invested_capital_in".to_string()),
        }
    }
}

/// Series tree node.
pub struct SeriesTree_Cohorts_Utxo_Entry_Discount_Realized {
    pub cap: CentsDeltaToUsdPattern,
    pub profit: BlockCumulativeSumPattern,
    pub loss: BlockCumulativeNegativeSumPattern,
    pub price: SeriesTree_Cohorts_Utxo_Entry_Discount_Realized_Price,
    pub mvrv: SeriesPattern1<StoredF32>,
    pub net_pnl: BlockChangeCumulativeDeltaSumPattern,
    pub sopr: RatioValuePattern2,
    pub gross_pnl: BlockCumulativeSumPattern,
    pub sell_side_risk_ratio: _1m1w1y24hPattern8,
    pub peak_regret: BlockCumulativeSumPattern,
    pub capitalized: PricePattern,
    pub profit_to_loss_ratio: _1m1w1y24hPattern<StoredF64>,
}

impl SeriesTree_Cohorts_Utxo_Entry_Discount_Realized {
    pub fn new(client: Arc<BrkClientBase>, base_path: String) -> Self {
        Self {
            cap: CentsDeltaToUsdPattern::new(client.clone(), "veteran_realized_cap".to_string()),
            profit: BlockCumulativeSumPattern::new(client.clone(), "veteran_realized_profit".to_string()),
            loss: BlockCumulativeNegativeSumPattern::new(client.clone(), "veteran_realized_loss".to_string()),
            price: SeriesTree_Cohorts_Utxo_Entry_Discount_Realized_Price::new(client.clone(), format!("{base_path}_price")),
            mvrv: SeriesPattern1::new(client.clone(), "veteran_mvrv".to_string()),
            net_pnl: BlockChangeCumulativeDeltaSumPattern::new(client.clone(), "veteran_net".to_string()),
            sopr: RatioValuePattern2::new(client.clone(), "veteran".to_string()),
            gross_pnl: BlockCumulativeSumPattern::new(client.clone(), "veteran_realized_gross_pnl".to_string()),
            sell_side_risk_ratio: _1m1w1y24hPattern8::new(client.clone(), "veteran_sell_side_risk_ratio".to_string()),
            peak_regret: BlockCumulativeSumPattern::new(client.clone(), "veteran_realized_peak_regret".to_string()),
            capitalized: PricePattern::new(client.clone(), "veteran_capitalized_price".to_string()),
            profit_to_loss_ratio: _1m1w1y24hPattern::new(client.clone(), "veteran_realized_profit_to_loss_ratio".to_string()),
        }
    }
}

/// Series tree node.
pub struct SeriesTree_Cohorts_Utxo_Entry_Discount_Realized_Price {
    pub usd: SeriesPattern1<Dollars>,
    pub cents: SeriesPattern1<Cents>,
    pub sats: SeriesPattern1<SatsFract>,
    pub bps: SeriesPattern1<BasisPoints32>,
    pub ratio: SeriesPattern1<StoredF32>,
    pub percentiles: Pct0Pct1Pct2Pct5Pct95Pct98Pct99Pattern,
    pub sma: _1m1w1y2y4yAllPattern,
    pub std_dev: SeriesTree_Cohorts_Utxo_Entry_Discount_Realized_Price_StdDev,
}

impl SeriesTree_Cohorts_Utxo_Entry_Discount_Realized_Price {
    pub fn new(client: Arc<BrkClientBase>, base_path: String) -> Self {
        Self {
            usd: SeriesPattern1::new(client.clone(), "veteran_realized_price".to_string()),
            cents: SeriesPattern1::new(client.clone(), "veteran_realized_price_cents".to_string()),
            sats: SeriesPattern1::new(client.clone(), "veteran_realized_price_sats".to_string()),
            bps: SeriesPattern1::new(client.clone(), "veteran_realized_price_ratio_bps".to_string()),
            ratio: SeriesPattern1::new(client.clone(), "veteran_realized_price_ratio".to_string()),
            percentiles: Pct0Pct1Pct2Pct5Pct95Pct98Pct99Pattern::new(client.clone(), "veteran_realized_price".to_string()),
            sma: _1m1w1y2y4yAllPattern::new(client.clone(), "veteran_realized_price_ratio_sma".to_string()),
            std_dev: SeriesTree_Cohorts_Utxo_Entry_Discount_Realized_Price_StdDev::new(client.clone(), format!("{base_path}_std_dev")),
        }
    }
}

/// Series tree node.
pub struct SeriesTree_Cohorts_Utxo_Entry_Discount_Realized_Price_StdDev {
    pub all: SeriesTree_Cohorts_Utxo_Entry_Discount_Realized_Price_StdDev_All,
    pub _4y: SeriesTree_Cohorts_Utxo_Entry_Discount_Realized_Price_StdDev_4y,
    pub _2y: SeriesTree_Cohorts_Utxo_Entry_Discount_Realized_Price_StdDev_2y,
    pub _1y: SeriesTree_Cohorts_Utxo_Entry_Discount_Realized_Price_StdDev_1y,
}

impl SeriesTree_Cohorts_Utxo_Entry_Discount_Realized_Price_StdDev {
    pub fn new(client: Arc<BrkClientBase>, base_path: String) -> Self {
        Self {
            all: SeriesTree_Cohorts_Utxo_Entry_Discount_Realized_Price_StdDev_All::new(client.clone(), format!("{base_path}_all")),
            _4y: SeriesTree_Cohorts_Utxo_Entry_Discount_Realized_Price_StdDev_4y::new(client.clone(), format!("{base_path}_4y")),
            _2y: SeriesTree_Cohorts_Utxo_Entry_Discount_Realized_Price_StdDev_2y::new(client.clone(), format!("{base_path}_2y")),
            _1y: SeriesTree_Cohorts_Utxo_Entry_Discount_Realized_Price_StdDev_1y::new(client.clone(), format!("{base_path}_1y")),
        }
    }
}

/// Series tree node.
pub struct SeriesTree_Cohorts_Utxo_Entry_Discount_Realized_Price_StdDev_All {
    pub sd: SeriesPattern1<StoredF32>,
    pub zscore: SeriesPattern1<StoredF32>,
    pub _0sd: CentsSatsUsdPattern,
    pub p0_5sd: PriceRatioPattern,
    pub p1sd: PriceRatioPattern,
    pub p1_5sd: PriceRatioPattern,
    pub p2sd: PriceRatioPattern,
    pub p2_5sd: PriceRatioPattern,
    pub p3sd: PriceRatioPattern,
    pub m0_5sd: PriceRatioPattern,
    pub m1sd: PriceRatioPattern,
    pub m1_5sd: PriceRatioPattern,
    pub m2sd: PriceRatioPattern,
    pub m2_5sd: PriceRatioPattern,
    pub m3sd: PriceRatioPattern,
}

impl SeriesTree_Cohorts_Utxo_Entry_Discount_Realized_Price_StdDev_All {
    pub fn new(client: Arc<BrkClientBase>, base_path: String) -> Self {
        Self {
            sd: SeriesPattern1::new(client.clone(), "veteran_realized_price_ratio_sd".to_string()),
            zscore: SeriesPattern1::new(client.clone(), "veteran_realized_price_ratio_zscore".to_string()),
            _0sd: CentsSatsUsdPattern::new(client.clone(), "veteran_realized_price_0sd".to_string()),
            p0_5sd: PriceRatioPattern::new(client.clone(), "veteran_realized_price".to_string(), "p0_5sd".to_string()),
            p1sd: PriceRatioPattern::new(client.clone(), "veteran_realized_price".to_string(), "p1sd".to_string()),
            p1_5sd: PriceRatioPattern::new(client.clone(), "veteran_realized_price".to_string(), "p1_5sd".to_string()),
            p2sd: PriceRatioPattern::new(client.clone(), "veteran_realized_price".to_string(), "p2sd".to_string()),
            p2_5sd: PriceRatioPattern::new(client.clone(), "veteran_realized_price".to_string(), "p2_5sd".to_string()),
            p3sd: PriceRatioPattern::new(client.clone(), "veteran_realized_price".to_string(), "p3sd".to_string()),
            m0_5sd: PriceRatioPattern::new(client.clone(), "veteran_realized_price".to_string(), "m0_5sd".to_string()),
            m1sd: PriceRatioPattern::new(client.clone(), "veteran_realized_price".to_string(), "m1sd".to_string()),
            m1_5sd: PriceRatioPattern::new(client.clone(), "veteran_realized_price".to_string(), "m1_5sd".to_string()),
            m2sd: PriceRatioPattern::new(client.clone(), "veteran_realized_price".to_string(), "m2sd".to_string()),
            m2_5sd: PriceRatioPattern::new(client.clone(), "veteran_realized_price".to_string(), "m2_5sd".to_string()),
            m3sd: PriceRatioPattern::new(client.clone(), "veteran_realized_price".to_string(), "m3sd".to_string()),
        }
    }
}

/// Series tree node.
pub struct SeriesTree_Cohorts_Utxo_Entry_Discount_Realized_Price_StdDev_4y {
    pub sd: SeriesPattern1<StoredF32>,
    pub zscore: SeriesPattern1<StoredF32>,
    pub _0sd: CentsSatsUsdPattern,
    pub p0_5sd: PriceRatioPattern,
    pub p1sd: PriceRatioPattern,
    pub p1_5sd: PriceRatioPattern,
    pub p2sd: PriceRatioPattern,
    pub p2_5sd: PriceRatioPattern,
    pub p3sd: PriceRatioPattern,
    pub m0_5sd: PriceRatioPattern,
    pub m1sd: PriceRatioPattern,
    pub m1_5sd: PriceRatioPattern,
    pub m2sd: PriceRatioPattern,
    pub m2_5sd: PriceRatioPattern,
    pub m3sd: PriceRatioPattern,
}

impl SeriesTree_Cohorts_Utxo_Entry_Discount_Realized_Price_StdDev_4y {
    pub fn new(client: Arc<BrkClientBase>, base_path: String) -> Self {
        Self {
            sd: SeriesPattern1::new(client.clone(), "veteran_realized_price_ratio_sd_4y".to_string()),
            zscore: SeriesPattern1::new(client.clone(), "veteran_realized_price_ratio_zscore_4y".to_string()),
            _0sd: CentsSatsUsdPattern::new(client.clone(), "veteran_realized_price_0sd_4y".to_string()),
            p0_5sd: PriceRatioPattern::new(client.clone(), "veteran_realized_price".to_string(), "p0_5sd_4y".to_string()),
            p1sd: PriceRatioPattern::new(client.clone(), "veteran_realized_price".to_string(), "p1sd_4y".to_string()),
            p1_5sd: PriceRatioPattern::new(client.clone(), "veteran_realized_price".to_string(), "p1_5sd_4y".to_string()),
            p2sd: PriceRatioPattern::new(client.clone(), "veteran_realized_price".to_string(), "p2sd_4y".to_string()),
            p2_5sd: PriceRatioPattern::new(client.clone(), "veteran_realized_price".to_string(), "p2_5sd_4y".to_string()),
            p3sd: PriceRatioPattern::new(client.clone(), "veteran_realized_price".to_string(), "p3sd_4y".to_string()),
            m0_5sd: PriceRatioPattern::new(client.clone(), "veteran_realized_price".to_string(), "m0_5sd_4y".to_string()),
            m1sd: PriceRatioPattern::new(client.clone(), "veteran_realized_price".to_string(), "m1sd_4y".to_string()),
            m1_5sd: PriceRatioPattern::new(client.clone(), "veteran_realized_price".to_string(), "m1_5sd_4y".to_string()),
            m2sd: PriceRatioPattern::new(client.clone(), "veteran_realized_price".to_string(), "m2sd_4y".to_string()),
            m2_5sd: PriceRatioPattern::new(client.clone(), "veteran_realized_price".to_string(), "m2_5sd_4y".to_string()),
            m3sd: PriceRatioPattern::new(client.clone(), "veteran_realized_price".to_string(), "m3sd_4y".to_string()),
        }
    }
}

/// Series tree node.
pub struct SeriesTree_Cohorts_Utxo_Entry_Discount_Realized_Price_StdDev_2y {
    pub sd: SeriesPattern1<StoredF32>,
    pub zscore: SeriesPattern1<StoredF32>,
    pub _0sd: CentsSatsUsdPattern,
    pub p0_5sd: PriceRatioPattern,
    pub p1sd: PriceRatioPattern,
    pub p1_5sd: PriceRatioPattern,
    pub p2sd: PriceRatioPattern,
    pub p2_5sd: PriceRatioPattern,
    pub p3sd: PriceRatioPattern,
    pub m0_5sd: PriceRatioPattern,
    pub m1sd: PriceRatioPattern,
    pub m1_5sd: PriceRatioPattern,
    pub m2sd: PriceRatioPattern,
    pub m2_5sd: PriceRatioPattern,
    pub m3sd: PriceRatioPattern,
}

impl SeriesTree_Cohorts_Utxo_Entry_Discount_Realized_Price_StdDev_2y {
    pub fn new(client: Arc<BrkClientBase>, base_path: String) -> Self {
        Self {
            sd: SeriesPattern1::new(client.clone(), "veteran_realized_price_ratio_sd_2y".to_string()),
            zscore: SeriesPattern1::new(client.clone(), "veteran_realized_price_ratio_zscore_2y".to_string()),
            _0sd: CentsSatsUsdPattern::new(client.clone(), "veteran_realized_price_0sd_2y".to_string()),
            p0_5sd: PriceRatioPattern::new(client.clone(), "veteran_realized_price".to_string(), "p0_5sd_2y".to_string()),
            p1sd: PriceRatioPattern::new(client.clone(), "veteran_realized_price".to_string(), "p1sd_2y".to_string()),
            p1_5sd: PriceRatioPattern::new(client.clone(), "veteran_realized_price".to_string(), "p1_5sd_2y".to_string()),
            p2sd: PriceRatioPattern::new(client.clone(), "veteran_realized_price".to_string(), "p2sd_2y".to_string()),
            p2_5sd: PriceRatioPattern::new(client.clone(), "veteran_realized_price".to_string(), "p2_5sd_2y".to_string()),
            p3sd: PriceRatioPattern::new(client.clone(), "veteran_realized_price".to_string(), "p3sd_2y".to_string()),
            m0_5sd: PriceRatioPattern::new(client.clone(), "veteran_realized_price".to_string(), "m0_5sd_2y".to_string()),
            m1sd: PriceRatioPattern::new(client.clone(), "veteran_realized_price".to_string(), "m1sd_2y".to_string()),
            m1_5sd: PriceRatioPattern::new(client.clone(), "veteran_realized_price".to_string(), "m1_5sd_2y".to_string()),
            m2sd: PriceRatioPattern::new(client.clone(), "veteran_realized_price".to_string(), "m2sd_2y".to_string()),
            m2_5sd: PriceRatioPattern::new(client.clone(), "veteran_realized_price".to_string(), "m2_5sd_2y".to_string()),
            m3sd: PriceRatioPattern::new(client.clone(), "veteran_realized_price".to_string(), "m3sd_2y".to_string()),
        }
    }
}

/// Series tree node.
pub struct SeriesTree_Cohorts_Utxo_Entry_Discount_Realized_Price_StdDev_1y {
    pub sd: SeriesPattern1<StoredF32>,
    pub zscore: SeriesPattern1<StoredF32>,
    pub _0sd: CentsSatsUsdPattern,
    pub p0_5sd: PriceRatioPattern,
    pub p1sd: PriceRatioPattern,
    pub p1_5sd: PriceRatioPattern,
    pub p2sd: PriceRatioPattern,
    pub p2_5sd: PriceRatioPattern,
    pub p3sd: PriceRatioPattern,
    pub m0_5sd: PriceRatioPattern,
    pub m1sd: PriceRatioPattern,
    pub m1_5sd: PriceRatioPattern,
    pub m2sd: PriceRatioPattern,
    pub m2_5sd: PriceRatioPattern,
    pub m3sd: PriceRatioPattern,
}

impl SeriesTree_Cohorts_Utxo_Entry_Discount_Realized_Price_StdDev_1y {
    pub fn new(client: Arc<BrkClientBase>, base_path: String) -> Self {
        Self {
            sd: SeriesPattern1::new(client.clone(), "veteran_realized_price_ratio_sd_1y".to_string()),
            zscore: SeriesPattern1::new(client.clone(), "veteran_realized_price_ratio_zscore_1y".to_string()),
            _0sd: CentsSatsUsdPattern::new(client.clone(), "veteran_realized_price_0sd_1y".to_string()),
            p0_5sd: PriceRatioPattern::new(client.clone(), "veteran_realized_price".to_string(), "p0_5sd_1y".to_string()),
            p1sd: PriceRatioPattern::new(client.clone(), "veteran_realized_price".to_string(), "p1sd_1y".to_string()),
            p1_5sd: PriceRatioPattern::new(client.clone(), "veteran_realized_price".to_string(), "p1_5sd_1y".to_string()),
            p2sd: PriceRatioPattern::new(client.clone(), "veteran_realized_price".to_string(), "p2sd_1y".to_string()),
            p2_5sd: PriceRatioPattern::new(client.clone(), "veteran_realized_price".to_string(), "p2_5sd_1y".to_string()),
            p3sd: PriceRatioPattern::new(client.clone(), "veteran_realized_price".to_string(), "p3sd_1y".to_string()),
            m0_5sd: PriceRatioPattern::new(client.clone(), "veteran_realized_price".to_string(), "m0_5sd_1y".to_string()),
            m1sd: PriceRatioPattern::new(client.clone(), "veteran_realized_price".to_string(), "m1sd_1y".to_string()),
            m1_5sd: PriceRatioPattern::new(client.clone(), "veteran_realized_price".to_string(), "m1_5sd_1y".to_string()),
            m2sd: PriceRatioPattern::new(client.clone(), "veteran_realized_price".to_string(), "m2sd_1y".to_string()),
            m2_5sd: PriceRatioPattern::new(client.clone(), "veteran_realized_price".to_string(), "m2_5sd_1y".to_string()),
            m3sd: PriceRatioPattern::new(client.clone(), "veteran_realized_price".to_string(), "m3sd_1y".to_string()),
        }
    }
}

/// Series tree node.
pub struct SeriesTree_Cohorts_Utxo_Entry_Premium {
    pub supply: DeltaDominanceHalfInTotalPattern2,
    pub outputs: SpendingSpentUnspentPattern,
    pub activity: CoindaysCoinyearsDormancyTransferPattern,
    pub realized: SeriesTree_Cohorts_Utxo_Entry_Premium_Realized,
    pub cost_basis: InMaxMinPerSupplyPattern,
    pub unrealized: CapitalizedGrossInvestedLossNetNuplProfitSentimentPattern2,
    pub invested_capital: InPattern,
}

impl SeriesTree_Cohorts_Utxo_Entry_Premium {
    pub fn new(client: Arc<BrkClientBase>, base_path: String) -> Self {
        Self {
            supply: DeltaDominanceHalfInTotalPattern2::new(client.clone(), "rookie_supply".to_string()),
            outputs: SpendingSpentUnspentPattern::new(client.clone(), "rookie".to_string()),
            activity: CoindaysCoinyearsDormancyTransferPattern::new(client.clone(), "rookie".to_string()),
            realized: SeriesTree_Cohorts_Utxo_Entry_Premium_Realized::new(client.clone(), format!("{base_path}_realized")),
            cost_basis: InMaxMinPerSupplyPattern::new(client.clone(), "rookie".to_string()),
            unrealized: CapitalizedGrossInvestedLossNetNuplProfitSentimentPattern2::new(client.clone(), "rookie".to_string()),
            invested_capital: InPattern::new(client.clone(), "rookie_invested_capital_in".to_string()),
        }
    }
}

/// Series tree node.
pub struct SeriesTree_Cohorts_Utxo_Entry_Premium_Realized {
    pub cap: CentsDeltaToUsdPattern,
    pub profit: BlockCumulativeSumPattern,
    pub loss: BlockCumulativeNegativeSumPattern,
    pub price: SeriesTree_Cohorts_Utxo_Entry_Premium_Realized_Price,
    pub mvrv: SeriesPattern1<StoredF32>,
    pub net_pnl: BlockChangeCumulativeDeltaSumPattern,
    pub sopr: RatioValuePattern2,
    pub gross_pnl: BlockCumulativeSumPattern,
    pub sell_side_risk_ratio: _1m1w1y24hPattern8,
    pub peak_regret: BlockCumulativeSumPattern,
    pub capitalized: PricePattern,
    pub profit_to_loss_ratio: _1m1w1y24hPattern<StoredF64>,
}

impl SeriesTree_Cohorts_Utxo_Entry_Premium_Realized {
    pub fn new(client: Arc<BrkClientBase>, base_path: String) -> Self {
        Self {
            cap: CentsDeltaToUsdPattern::new(client.clone(), "rookie_realized_cap".to_string()),
            profit: BlockCumulativeSumPattern::new(client.clone(), "rookie_realized_profit".to_string()),
            loss: BlockCumulativeNegativeSumPattern::new(client.clone(), "rookie_realized_loss".to_string()),
            price: SeriesTree_Cohorts_Utxo_Entry_Premium_Realized_Price::new(client.clone(), format!("{base_path}_price")),
            mvrv: SeriesPattern1::new(client.clone(), "rookie_mvrv".to_string()),
            net_pnl: BlockChangeCumulativeDeltaSumPattern::new(client.clone(), "rookie_net".to_string()),
            sopr: RatioValuePattern2::new(client.clone(), "rookie".to_string()),
            gross_pnl: BlockCumulativeSumPattern::new(client.clone(), "rookie_realized_gross_pnl".to_string()),
            sell_side_risk_ratio: _1m1w1y24hPattern8::new(client.clone(), "rookie_sell_side_risk_ratio".to_string()),
            peak_regret: BlockCumulativeSumPattern::new(client.clone(), "rookie_realized_peak_regret".to_string()),
            capitalized: PricePattern::new(client.clone(), "rookie_capitalized_price".to_string()),
            profit_to_loss_ratio: _1m1w1y24hPattern::new(client.clone(), "rookie_realized_profit_to_loss_ratio".to_string()),
        }
    }
}

/// Series tree node.
pub struct SeriesTree_Cohorts_Utxo_Entry_Premium_Realized_Price {
    pub usd: SeriesPattern1<Dollars>,
    pub cents: SeriesPattern1<Cents>,
    pub sats: SeriesPattern1<SatsFract>,
    pub bps: SeriesPattern1<BasisPoints32>,
    pub ratio: SeriesPattern1<StoredF32>,
    pub percentiles: Pct0Pct1Pct2Pct5Pct95Pct98Pct99Pattern,
    pub sma: _1m1w1y2y4yAllPattern,
    pub std_dev: SeriesTree_Cohorts_Utxo_Entry_Premium_Realized_Price_StdDev,
}

impl SeriesTree_Cohorts_Utxo_Entry_Premium_Realized_Price {
    pub fn new(client: Arc<BrkClientBase>, base_path: String) -> Self {
        Self {
            usd: SeriesPattern1::new(client.clone(), "rookie_realized_price".to_string()),
            cents: SeriesPattern1::new(client.clone(), "rookie_realized_price_cents".to_string()),
            sats: SeriesPattern1::new(client.clone(), "rookie_realized_price_sats".to_string()),
            bps: SeriesPattern1::new(client.clone(), "rookie_realized_price_ratio_bps".to_string()),
            ratio: SeriesPattern1::new(client.clone(), "rookie_realized_price_ratio".to_string()),
            percentiles: Pct0Pct1Pct2Pct5Pct95Pct98Pct99Pattern::new(client.clone(), "rookie_realized_price".to_string()),
            sma: _1m1w1y2y4yAllPattern::new(client.clone(), "rookie_realized_price_ratio_sma".to_string()),
            std_dev: SeriesTree_Cohorts_Utxo_Entry_Premium_Realized_Price_StdDev::new(client.clone(), format!("{base_path}_std_dev")),
        }
    }
}

/// Series tree node.
pub struct SeriesTree_Cohorts_Utxo_Entry_Premium_Realized_Price_StdDev {
    pub all: SeriesTree_Cohorts_Utxo_Entry_Premium_Realized_Price_StdDev_All,
    pub _4y: SeriesTree_Cohorts_Utxo_Entry_Premium_Realized_Price_StdDev_4y,
    pub _2y: SeriesTree_Cohorts_Utxo_Entry_Premium_Realized_Price_StdDev_2y,
    pub _1y: SeriesTree_Cohorts_Utxo_Entry_Premium_Realized_Price_StdDev_1y,
}

impl SeriesTree_Cohorts_Utxo_Entry_Premium_Realized_Price_StdDev {
    pub fn new(client: Arc<BrkClientBase>, base_path: String) -> Self {
        Self {
            all: SeriesTree_Cohorts_Utxo_Entry_Premium_Realized_Price_StdDev_All::new(client.clone(), format!("{base_path}_all")),
            _4y: SeriesTree_Cohorts_Utxo_Entry_Premium_Realized_Price_StdDev_4y::new(client.clone(), format!("{base_path}_4y")),
            _2y: SeriesTree_Cohorts_Utxo_Entry_Premium_Realized_Price_StdDev_2y::new(client.clone(), format!("{base_path}_2y")),
            _1y: SeriesTree_Cohorts_Utxo_Entry_Premium_Realized_Price_StdDev_1y::new(client.clone(), format!("{base_path}_1y")),
        }
    }
}

/// Series tree node.
pub struct SeriesTree_Cohorts_Utxo_Entry_Premium_Realized_Price_StdDev_All {
    pub sd: SeriesPattern1<StoredF32>,
    pub zscore: SeriesPattern1<StoredF32>,
    pub _0sd: CentsSatsUsdPattern,
    pub p0_5sd: PriceRatioPattern,
    pub p1sd: PriceRatioPattern,
    pub p1_5sd: PriceRatioPattern,
    pub p2sd: PriceRatioPattern,
    pub p2_5sd: PriceRatioPattern,
    pub p3sd: PriceRatioPattern,
    pub m0_5sd: PriceRatioPattern,
    pub m1sd: PriceRatioPattern,
    pub m1_5sd: PriceRatioPattern,
    pub m2sd: PriceRatioPattern,
    pub m2_5sd: PriceRatioPattern,
    pub m3sd: PriceRatioPattern,
}

impl SeriesTree_Cohorts_Utxo_Entry_Premium_Realized_Price_StdDev_All {
    pub fn new(client: Arc<BrkClientBase>, base_path: String) -> Self {
        Self {
            sd: SeriesPattern1::new(client.clone(), "rookie_realized_price_ratio_sd".to_string()),
            zscore: SeriesPattern1::new(client.clone(), "rookie_realized_price_ratio_zscore".to_string()),
            _0sd: CentsSatsUsdPattern::new(client.clone(), "rookie_realized_price_0sd".to_string()),
            p0_5sd: PriceRatioPattern::new(client.clone(), "rookie_realized_price".to_string(), "p0_5sd".to_string()),
            p1sd: PriceRatioPattern::new(client.clone(), "rookie_realized_price".to_string(), "p1sd".to_string()),
            p1_5sd: PriceRatioPattern::new(client.clone(), "rookie_realized_price".to_string(), "p1_5sd".to_string()),
            p2sd: PriceRatioPattern::new(client.clone(), "rookie_realized_price".to_string(), "p2sd".to_string()),
            p2_5sd: PriceRatioPattern::new(client.clone(), "rookie_realized_price".to_string(), "p2_5sd".to_string()),
            p3sd: PriceRatioPattern::new(client.clone(), "rookie_realized_price".to_string(), "p3sd".to_string()),
            m0_5sd: PriceRatioPattern::new(client.clone(), "rookie_realized_price".to_string(), "m0_5sd".to_string()),
            m1sd: PriceRatioPattern::new(client.clone(), "rookie_realized_price".to_string(), "m1sd".to_string()),
            m1_5sd: PriceRatioPattern::new(client.clone(), "rookie_realized_price".to_string(), "m1_5sd".to_string()),
            m2sd: PriceRatioPattern::new(client.clone(), "rookie_realized_price".to_string(), "m2sd".to_string()),
            m2_5sd: PriceRatioPattern::new(client.clone(), "rookie_realized_price".to_string(), "m2_5sd".to_string()),
            m3sd: PriceRatioPattern::new(client.clone(), "rookie_realized_price".to_string(), "m3sd".to_string()),
        }
    }
}

/// Series tree node.
pub struct SeriesTree_Cohorts_Utxo_Entry_Premium_Realized_Price_StdDev_4y {
    pub sd: SeriesPattern1<StoredF32>,
    pub zscore: SeriesPattern1<StoredF32>,
    pub _0sd: CentsSatsUsdPattern,
    pub p0_5sd: PriceRatioPattern,
    pub p1sd: PriceRatioPattern,
    pub p1_5sd: PriceRatioPattern,
    pub p2sd: PriceRatioPattern,
    pub p2_5sd: PriceRatioPattern,
    pub p3sd: PriceRatioPattern,
    pub m0_5sd: PriceRatioPattern,
    pub m1sd: PriceRatioPattern,
    pub m1_5sd: PriceRatioPattern,
    pub m2sd: PriceRatioPattern,
    pub m2_5sd: PriceRatioPattern,
    pub m3sd: PriceRatioPattern,
}

impl SeriesTree_Cohorts_Utxo_Entry_Premium_Realized_Price_StdDev_4y {
    pub fn new(client: Arc<BrkClientBase>, base_path: String) -> Self {
        Self {
            sd: SeriesPattern1::new(client.clone(), "rookie_realized_price_ratio_sd_4y".to_string()),
            zscore: SeriesPattern1::new(client.clone(), "rookie_realized_price_ratio_zscore_4y".to_string()),
            _0sd: CentsSatsUsdPattern::new(client.clone(), "rookie_realized_price_0sd_4y".to_string()),
            p0_5sd: PriceRatioPattern::new(client.clone(), "rookie_realized_price".to_string(), "p0_5sd_4y".to_string()),
            p1sd: PriceRatioPattern::new(client.clone(), "rookie_realized_price".to_string(), "p1sd_4y".to_string()),
            p1_5sd: PriceRatioPattern::new(client.clone(), "rookie_realized_price".to_string(), "p1_5sd_4y".to_string()),
            p2sd: PriceRatioPattern::new(client.clone(), "rookie_realized_price".to_string(), "p2sd_4y".to_string()),
            p2_5sd: PriceRatioPattern::new(client.clone(), "rookie_realized_price".to_string(), "p2_5sd_4y".to_string()),
            p3sd: PriceRatioPattern::new(client.clone(), "rookie_realized_price".to_string(), "p3sd_4y".to_string()),
            m0_5sd: PriceRatioPattern::new(client.clone(), "rookie_realized_price".to_string(), "m0_5sd_4y".to_string()),
            m1sd: PriceRatioPattern::new(client.clone(), "rookie_realized_price".to_string(), "m1sd_4y".to_string()),
            m1_5sd: PriceRatioPattern::new(client.clone(), "rookie_realized_price".to_string(), "m1_5sd_4y".to_string()),
            m2sd: PriceRatioPattern::new(client.clone(), "rookie_realized_price".to_string(), "m2sd_4y".to_string()),
            m2_5sd: PriceRatioPattern::new(client.clone(), "rookie_realized_price".to_string(), "m2_5sd_4y".to_string()),
            m3sd: PriceRatioPattern::new(client.clone(), "rookie_realized_price".to_string(), "m3sd_4y".to_string()),
        }
    }
}

/// Series tree node.
pub struct SeriesTree_Cohorts_Utxo_Entry_Premium_Realized_Price_StdDev_2y {
    pub sd: SeriesPattern1<StoredF32>,
    pub zscore: SeriesPattern1<StoredF32>,
    pub _0sd: CentsSatsUsdPattern,
    pub p0_5sd: PriceRatioPattern,
    pub p1sd: PriceRatioPattern,
    pub p1_5sd: PriceRatioPattern,
    pub p2sd: PriceRatioPattern,
    pub p2_5sd: PriceRatioPattern,
    pub p3sd: PriceRatioPattern,
    pub m0_5sd: PriceRatioPattern,
    pub m1sd: PriceRatioPattern,
    pub m1_5sd: PriceRatioPattern,
    pub m2sd: PriceRatioPattern,
    pub m2_5sd: PriceRatioPattern,
    pub m3sd: PriceRatioPattern,
}

impl SeriesTree_Cohorts_Utxo_Entry_Premium_Realized_Price_StdDev_2y {
    pub fn new(client: Arc<BrkClientBase>, base_path: String) -> Self {
        Self {
            sd: SeriesPattern1::new(client.clone(), "rookie_realized_price_ratio_sd_2y".to_string()),
            zscore: SeriesPattern1::new(client.clone(), "rookie_realized_price_ratio_zscore_2y".to_string()),
            _0sd: CentsSatsUsdPattern::new(client.clone(), "rookie_realized_price_0sd_2y".to_string()),
            p0_5sd: PriceRatioPattern::new(client.clone(), "rookie_realized_price".to_string(), "p0_5sd_2y".to_string()),
            p1sd: PriceRatioPattern::new(client.clone(), "rookie_realized_price".to_string(), "p1sd_2y".to_string()),
            p1_5sd: PriceRatioPattern::new(client.clone(), "rookie_realized_price".to_string(), "p1_5sd_2y".to_string()),
            p2sd: PriceRatioPattern::new(client.clone(), "rookie_realized_price".to_string(), "p2sd_2y".to_string()),
            p2_5sd: PriceRatioPattern::new(client.clone(), "rookie_realized_price".to_string(), "p2_5sd_2y".to_string()),
            p3sd: PriceRatioPattern::new(client.clone(), "rookie_realized_price".to_string(), "p3sd_2y".to_string()),
            m0_5sd: PriceRatioPattern::new(client.clone(), "rookie_realized_price".to_string(), "m0_5sd_2y".to_string()),
            m1sd: PriceRatioPattern::new(client.clone(), "rookie_realized_price".to_string(), "m1sd_2y".to_string()),
            m1_5sd: PriceRatioPattern::new(client.clone(), "rookie_realized_price".to_string(), "m1_5sd_2y".to_string()),
            m2sd: PriceRatioPattern::new(client.clone(), "rookie_realized_price".to_string(), "m2sd_2y".to_string()),
            m2_5sd: PriceRatioPattern::new(client.clone(), "rookie_realized_price".to_string(), "m2_5sd_2y".to_string()),
            m3sd: PriceRatioPattern::new(client.clone(), "rookie_realized_price".to_string(), "m3sd_2y".to_string()),
        }
    }
}

/// Series tree node.
pub struct SeriesTree_Cohorts_Utxo_Entry_Premium_Realized_Price_StdDev_1y {
    pub sd: SeriesPattern1<StoredF32>,
    pub zscore: SeriesPattern1<StoredF32>,
    pub _0sd: CentsSatsUsdPattern,
    pub p0_5sd: PriceRatioPattern,
    pub p1sd: PriceRatioPattern,
    pub p1_5sd: PriceRatioPattern,
    pub p2sd: PriceRatioPattern,
    pub p2_5sd: PriceRatioPattern,
    pub p3sd: PriceRatioPattern,
    pub m0_5sd: PriceRatioPattern,
    pub m1sd: PriceRatioPattern,
    pub m1_5sd: PriceRatioPattern,
    pub m2sd: PriceRatioPattern,
    pub m2_5sd: PriceRatioPattern,
    pub m3sd: PriceRatioPattern,
}

impl SeriesTree_Cohorts_Utxo_Entry_Premium_Realized_Price_StdDev_1y {
    pub fn new(client: Arc<BrkClientBase>, base_path: String) -> Self {
        Self {
            sd: SeriesPattern1::new(client.clone(), "rookie_realized_price_ratio_sd_1y".to_string()),
            zscore: SeriesPattern1::new(client.clone(), "rookie_realized_price_ratio_zscore_1y".to_string()),
            _0sd: CentsSatsUsdPattern::new(client.clone(), "rookie_realized_price_0sd_1y".to_string()),
            p0_5sd: PriceRatioPattern::new(client.clone(), "rookie_realized_price".to_string(), "p0_5sd_1y".to_string()),
            p1sd: PriceRatioPattern::new(client.clone(), "rookie_realized_price".to_string(), "p1sd_1y".to_string()),
            p1_5sd: PriceRatioPattern::new(client.clone(), "rookie_realized_price".to_string(), "p1_5sd_1y".to_string()),
            p2sd: PriceRatioPattern::new(client.clone(), "rookie_realized_price".to_string(), "p2sd_1y".to_string()),
            p2_5sd: PriceRatioPattern::new(client.clone(), "rookie_realized_price".to_string(), "p2_5sd_1y".to_string()),
            p3sd: PriceRatioPattern::new(client.clone(), "rookie_realized_price".to_string(), "p3sd_1y".to_string()),
            m0_5sd: PriceRatioPattern::new(client.clone(), "rookie_realized_price".to_string(), "m0_5sd_1y".to_string()),
            m1sd: PriceRatioPattern::new(client.clone(), "rookie_realized_price".to_string(), "m1sd_1y".to_string()),
            m1_5sd: PriceRatioPattern::new(client.clone(), "rookie_realized_price".to_string(), "m1_5sd_1y".to_string()),
            m2sd: PriceRatioPattern::new(client.clone(), "rookie_realized_price".to_string(), "m2sd_1y".to_string()),
            m2_5sd: PriceRatioPattern::new(client.clone(), "rookie_realized_price".to_string(), "m2_5sd_1y".to_string()),
            m3sd: PriceRatioPattern::new(client.clone(), "rookie_realized_price".to_string(), "m3sd_1y".to_string()),
        }
    }
}

/// Series tree node.
pub struct SeriesTree_Cohorts_Utxo_OverAmount {
    pub _1sat: ActivityOutputsRealizedSupplyUnrealizedPattern2,
    pub _10sats: ActivityOutputsRealizedSupplyUnrealizedPattern2,
    pub _100sats: ActivityOutputsRealizedSupplyUnrealizedPattern2,
    pub _1k_sats: ActivityOutputsRealizedSupplyUnrealizedPattern2,
    pub _10k_sats: ActivityOutputsRealizedSupplyUnrealizedPattern2,
    pub _100k_sats: ActivityOutputsRealizedSupplyUnrealizedPattern2,
    pub _1m_sats: ActivityOutputsRealizedSupplyUnrealizedPattern2,
    pub _10m_sats: ActivityOutputsRealizedSupplyUnrealizedPattern2,
    pub _1btc: ActivityOutputsRealizedSupplyUnrealizedPattern2,
    pub _10btc: ActivityOutputsRealizedSupplyUnrealizedPattern2,
    pub _100btc: ActivityOutputsRealizedSupplyUnrealizedPattern2,
    pub _1k_btc: ActivityOutputsRealizedSupplyUnrealizedPattern2,
    pub _10k_btc: ActivityOutputsRealizedSupplyUnrealizedPattern2,
}

impl SeriesTree_Cohorts_Utxo_OverAmount {
    pub fn new(client: Arc<BrkClientBase>, base_path: String) -> Self {
        Self {
            _1sat: ActivityOutputsRealizedSupplyUnrealizedPattern2::new(client.clone(), "utxos_over_1sat".to_string()),
            _10sats: ActivityOutputsRealizedSupplyUnrealizedPattern2::new(client.clone(), "utxos_over_10sats".to_string()),
            _100sats: ActivityOutputsRealizedSupplyUnrealizedPattern2::new(client.clone(), "utxos_over_100sats".to_string()),
            _1k_sats: ActivityOutputsRealizedSupplyUnrealizedPattern2::new(client.clone(), "utxos_over_1k_sats".to_string()),
            _10k_sats: ActivityOutputsRealizedSupplyUnrealizedPattern2::new(client.clone(), "utxos_over_10k_sats".to_string()),
            _100k_sats: ActivityOutputsRealizedSupplyUnrealizedPattern2::new(client.clone(), "utxos_over_100k_sats".to_string()),
            _1m_sats: ActivityOutputsRealizedSupplyUnrealizedPattern2::new(client.clone(), "utxos_over_1m_sats".to_string()),
            _10m_sats: ActivityOutputsRealizedSupplyUnrealizedPattern2::new(client.clone(), "utxos_over_10m_sats".to_string()),
            _1btc: ActivityOutputsRealizedSupplyUnrealizedPattern2::new(client.clone(), "utxos_over_1btc".to_string()),
            _10btc: ActivityOutputsRealizedSupplyUnrealizedPattern2::new(client.clone(), "utxos_over_10btc".to_string()),
            _100btc: ActivityOutputsRealizedSupplyUnrealizedPattern2::new(client.clone(), "utxos_over_100btc".to_string()),
            _1k_btc: ActivityOutputsRealizedSupplyUnrealizedPattern2::new(client.clone(), "utxos_over_1k_btc".to_string()),
            _10k_btc: ActivityOutputsRealizedSupplyUnrealizedPattern2::new(client.clone(), "utxos_over_10k_btc".to_string()),
        }
    }
}

/// Series tree node.
pub struct SeriesTree_Cohorts_Utxo_AmountRange {
    pub _0sats: ActivityOutputsRealizedSupplyUnrealizedPattern2,
    pub _1sat_to_10sats: ActivityOutputsRealizedSupplyUnrealizedPattern2,
    pub _10sats_to_100sats: ActivityOutputsRealizedSupplyUnrealizedPattern2,
    pub _100sats_to_1k_sats: ActivityOutputsRealizedSupplyUnrealizedPattern2,
    pub _1k_sats_to_10k_sats: ActivityOutputsRealizedSupplyUnrealizedPattern2,
    pub _10k_sats_to_100k_sats: ActivityOutputsRealizedSupplyUnrealizedPattern2,
    pub _100k_sats_to_1m_sats: ActivityOutputsRealizedSupplyUnrealizedPattern2,
    pub _1m_sats_to_10m_sats: ActivityOutputsRealizedSupplyUnrealizedPattern2,
    pub _10m_sats_to_1btc: ActivityOutputsRealizedSupplyUnrealizedPattern2,
    pub _1btc_to_10btc: ActivityOutputsRealizedSupplyUnrealizedPattern2,
    pub _10btc_to_100btc: ActivityOutputsRealizedSupplyUnrealizedPattern2,
    pub _100btc_to_1k_btc: ActivityOutputsRealizedSupplyUnrealizedPattern2,
    pub _1k_btc_to_10k_btc: ActivityOutputsRealizedSupplyUnrealizedPattern2,
    pub _10k_btc_to_100k_btc: ActivityOutputsRealizedSupplyUnrealizedPattern2,
    pub over_100k_btc: ActivityOutputsRealizedSupplyUnrealizedPattern2,
}

impl SeriesTree_Cohorts_Utxo_AmountRange {
    pub fn new(client: Arc<BrkClientBase>, base_path: String) -> Self {
        Self {
            _0sats: ActivityOutputsRealizedSupplyUnrealizedPattern2::new(client.clone(), "utxos_0sats".to_string()),
            _1sat_to_10sats: ActivityOutputsRealizedSupplyUnrealizedPattern2::new(client.clone(), "utxos_1sat_to_10sats".to_string()),
            _10sats_to_100sats: ActivityOutputsRealizedSupplyUnrealizedPattern2::new(client.clone(), "utxos_10sats_to_100sats".to_string()),
            _100sats_to_1k_sats: ActivityOutputsRealizedSupplyUnrealizedPattern2::new(client.clone(), "utxos_100sats_to_1k_sats".to_string()),
            _1k_sats_to_10k_sats: ActivityOutputsRealizedSupplyUnrealizedPattern2::new(client.clone(), "utxos_1k_sats_to_10k_sats".to_string()),
            _10k_sats_to_100k_sats: ActivityOutputsRealizedSupplyUnrealizedPattern2::new(client.clone(), "utxos_10k_sats_to_100k_sats".to_string()),
            _100k_sats_to_1m_sats: ActivityOutputsRealizedSupplyUnrealizedPattern2::new(client.clone(), "utxos_100k_sats_to_1m_sats".to_string()),
            _1m_sats_to_10m_sats: ActivityOutputsRealizedSupplyUnrealizedPattern2::new(client.clone(), "utxos_1m_sats_to_10m_sats".to_string()),
            _10m_sats_to_1btc: ActivityOutputsRealizedSupplyUnrealizedPattern2::new(client.clone(), "utxos_10m_sats_to_1btc".to_string()),
            _1btc_to_10btc: ActivityOutputsRealizedSupplyUnrealizedPattern2::new(client.clone(), "utxos_1btc_to_10btc".to_string()),
            _10btc_to_100btc: ActivityOutputsRealizedSupplyUnrealizedPattern2::new(client.clone(), "utxos_10btc_to_100btc".to_string()),
            _100btc_to_1k_btc: ActivityOutputsRealizedSupplyUnrealizedPattern2::new(client.clone(), "utxos_100btc_to_1k_btc".to_string()),
            _1k_btc_to_10k_btc: ActivityOutputsRealizedSupplyUnrealizedPattern2::new(client.clone(), "utxos_1k_btc_to_10k_btc".to_string()),
            _10k_btc_to_100k_btc: ActivityOutputsRealizedSupplyUnrealizedPattern2::new(client.clone(), "utxos_10k_btc_to_100k_btc".to_string()),
            over_100k_btc: ActivityOutputsRealizedSupplyUnrealizedPattern2::new(client.clone(), "utxos_over_100k_btc".to_string()),
        }
    }
}

/// Series tree node.
pub struct SeriesTree_Cohorts_Utxo_UnderAmount {
    pub _10sats: ActivityOutputsRealizedSupplyUnrealizedPattern2,
    pub _100sats: ActivityOutputsRealizedSupplyUnrealizedPattern2,
    pub _1k_sats: ActivityOutputsRealizedSupplyUnrealizedPattern2,
    pub _10k_sats: ActivityOutputsRealizedSupplyUnrealizedPattern2,
    pub _100k_sats: ActivityOutputsRealizedSupplyUnrealizedPattern2,
    pub _1m_sats: ActivityOutputsRealizedSupplyUnrealizedPattern2,
    pub _10m_sats: ActivityOutputsRealizedSupplyUnrealizedPattern2,
    pub _1btc: ActivityOutputsRealizedSupplyUnrealizedPattern2,
    pub _10btc: ActivityOutputsRealizedSupplyUnrealizedPattern2,
    pub _100btc: ActivityOutputsRealizedSupplyUnrealizedPattern2,
    pub _1k_btc: ActivityOutputsRealizedSupplyUnrealizedPattern2,
    pub _10k_btc: ActivityOutputsRealizedSupplyUnrealizedPattern2,
    pub _100k_btc: ActivityOutputsRealizedSupplyUnrealizedPattern2,
}

impl SeriesTree_Cohorts_Utxo_UnderAmount {
    pub fn new(client: Arc<BrkClientBase>, base_path: String) -> Self {
        Self {
            _10sats: ActivityOutputsRealizedSupplyUnrealizedPattern2::new(client.clone(), "utxos_under_10sats".to_string()),
            _100sats: ActivityOutputsRealizedSupplyUnrealizedPattern2::new(client.clone(), "utxos_under_100sats".to_string()),
            _1k_sats: ActivityOutputsRealizedSupplyUnrealizedPattern2::new(client.clone(), "utxos_under_1k_sats".to_string()),
            _10k_sats: ActivityOutputsRealizedSupplyUnrealizedPattern2::new(client.clone(), "utxos_under_10k_sats".to_string()),
            _100k_sats: ActivityOutputsRealizedSupplyUnrealizedPattern2::new(client.clone(), "utxos_under_100k_sats".to_string()),
            _1m_sats: ActivityOutputsRealizedSupplyUnrealizedPattern2::new(client.clone(), "utxos_under_1m_sats".to_string()),
            _10m_sats: ActivityOutputsRealizedSupplyUnrealizedPattern2::new(client.clone(), "utxos_under_10m_sats".to_string()),
            _1btc: ActivityOutputsRealizedSupplyUnrealizedPattern2::new(client.clone(), "utxos_under_1btc".to_string()),
            _10btc: ActivityOutputsRealizedSupplyUnrealizedPattern2::new(client.clone(), "utxos_under_10btc".to_string()),
            _100btc: ActivityOutputsRealizedSupplyUnrealizedPattern2::new(client.clone(), "utxos_under_100btc".to_string()),
            _1k_btc: ActivityOutputsRealizedSupplyUnrealizedPattern2::new(client.clone(), "utxos_under_1k_btc".to_string()),
            _10k_btc: ActivityOutputsRealizedSupplyUnrealizedPattern2::new(client.clone(), "utxos_under_10k_btc".to_string()),
            _100k_btc: ActivityOutputsRealizedSupplyUnrealizedPattern2::new(client.clone(), "utxos_under_100k_btc".to_string()),
        }
    }
}

/// Series tree node.
pub struct SeriesTree_Cohorts_Utxo_Type {
    pub p2pk65: ActivityOutputsRealizedSupplyUnrealizedPattern3,
    pub p2pk33: ActivityOutputsRealizedSupplyUnrealizedPattern3,
    pub p2pkh: ActivityOutputsRealizedSupplyUnrealizedPattern3,
    pub p2ms: ActivityOutputsRealizedSupplyUnrealizedPattern3,
    pub p2sh: ActivityOutputsRealizedSupplyUnrealizedPattern3,
    pub p2wpkh: ActivityOutputsRealizedSupplyUnrealizedPattern3,
    pub p2wsh: ActivityOutputsRealizedSupplyUnrealizedPattern3,
    pub p2tr: ActivityOutputsRealizedSupplyUnrealizedPattern3,
    pub p2a: ActivityOutputsRealizedSupplyUnrealizedPattern3,
    pub unknown: ActivityOutputsRealizedSupplyUnrealizedPattern3,
    pub empty: ActivityOutputsRealizedSupplyUnrealizedPattern3,
}

impl SeriesTree_Cohorts_Utxo_Type {
    pub fn new(client: Arc<BrkClientBase>, base_path: String) -> Self {
        Self {
            p2pk65: ActivityOutputsRealizedSupplyUnrealizedPattern3::new(client.clone(), "p2pk65".to_string()),
            p2pk33: ActivityOutputsRealizedSupplyUnrealizedPattern3::new(client.clone(), "p2pk33".to_string()),
            p2pkh: ActivityOutputsRealizedSupplyUnrealizedPattern3::new(client.clone(), "p2pkh".to_string()),
            p2ms: ActivityOutputsRealizedSupplyUnrealizedPattern3::new(client.clone(), "p2ms".to_string()),
            p2sh: ActivityOutputsRealizedSupplyUnrealizedPattern3::new(client.clone(), "p2sh".to_string()),
            p2wpkh: ActivityOutputsRealizedSupplyUnrealizedPattern3::new(client.clone(), "p2wpkh".to_string()),
            p2wsh: ActivityOutputsRealizedSupplyUnrealizedPattern3::new(client.clone(), "p2wsh".to_string()),
            p2tr: ActivityOutputsRealizedSupplyUnrealizedPattern3::new(client.clone(), "p2tr".to_string()),
            p2a: ActivityOutputsRealizedSupplyUnrealizedPattern3::new(client.clone(), "p2a".to_string()),
            unknown: ActivityOutputsRealizedSupplyUnrealizedPattern3::new(client.clone(), "unknown_outputs".to_string()),
            empty: ActivityOutputsRealizedSupplyUnrealizedPattern3::new(client.clone(), "empty_outputs".to_string()),
        }
    }
}

/// Series tree node.
pub struct SeriesTree_Cohorts_Utxo_Profitability {
    pub range: SeriesTree_Cohorts_Utxo_Profitability_Range,
    pub profit: SeriesTree_Cohorts_Utxo_Profitability_Profit,
    pub loss: SeriesTree_Cohorts_Utxo_Profitability_Loss,
}

impl SeriesTree_Cohorts_Utxo_Profitability {
    pub fn new(client: Arc<BrkClientBase>, base_path: String) -> Self {
        Self {
            range: SeriesTree_Cohorts_Utxo_Profitability_Range::new(client.clone(), format!("{base_path}_range")),
            profit: SeriesTree_Cohorts_Utxo_Profitability_Profit::new(client.clone(), format!("{base_path}_profit")),
            loss: SeriesTree_Cohorts_Utxo_Profitability_Loss::new(client.clone(), format!("{base_path}_loss")),
        }
    }
}

/// Series tree node.
pub struct SeriesTree_Cohorts_Utxo_Profitability_Range {
    pub over_1000pct_in_profit: NuplRealizedSupplyUnrealizedPattern,
    pub _500pct_to_1000pct_in_profit: NuplRealizedSupplyUnrealizedPattern,
    pub _300pct_to_500pct_in_profit: NuplRealizedSupplyUnrealizedPattern,
    pub _200pct_to_300pct_in_profit: NuplRealizedSupplyUnrealizedPattern,
    pub _100pct_to_200pct_in_profit: NuplRealizedSupplyUnrealizedPattern,
    pub _90pct_to_100pct_in_profit: NuplRealizedSupplyUnrealizedPattern,
    pub _80pct_to_90pct_in_profit: NuplRealizedSupplyUnrealizedPattern,
    pub _70pct_to_80pct_in_profit: NuplRealizedSupplyUnrealizedPattern,
    pub _60pct_to_70pct_in_profit: NuplRealizedSupplyUnrealizedPattern,
    pub _50pct_to_60pct_in_profit: NuplRealizedSupplyUnrealizedPattern,
    pub _40pct_to_50pct_in_profit: NuplRealizedSupplyUnrealizedPattern,
    pub _30pct_to_40pct_in_profit: NuplRealizedSupplyUnrealizedPattern,
    pub _20pct_to_30pct_in_profit: NuplRealizedSupplyUnrealizedPattern,
    pub _10pct_to_20pct_in_profit: NuplRealizedSupplyUnrealizedPattern,
    pub _0pct_to_10pct_in_profit: NuplRealizedSupplyUnrealizedPattern,
    pub _0pct_to_10pct_in_loss: NuplRealizedSupplyUnrealizedPattern,
    pub _10pct_to_20pct_in_loss: NuplRealizedSupplyUnrealizedPattern,
    pub _20pct_to_30pct_in_loss: NuplRealizedSupplyUnrealizedPattern,
    pub _30pct_to_40pct_in_loss: NuplRealizedSupplyUnrealizedPattern,
    pub _40pct_to_50pct_in_loss: NuplRealizedSupplyUnrealizedPattern,
    pub _50pct_to_60pct_in_loss: NuplRealizedSupplyUnrealizedPattern,
    pub _60pct_to_70pct_in_loss: NuplRealizedSupplyUnrealizedPattern,
    pub _70pct_to_80pct_in_loss: NuplRealizedSupplyUnrealizedPattern,
    pub _80pct_to_90pct_in_loss: NuplRealizedSupplyUnrealizedPattern,
    pub _90pct_to_100pct_in_loss: NuplRealizedSupplyUnrealizedPattern,
}

impl SeriesTree_Cohorts_Utxo_Profitability_Range {
    pub fn new(client: Arc<BrkClientBase>, base_path: String) -> Self {
        Self {
            over_1000pct_in_profit: NuplRealizedSupplyUnrealizedPattern::new(client.clone(), "utxos_over_1000pct_in_profit".to_string()),
            _500pct_to_1000pct_in_profit: NuplRealizedSupplyUnrealizedPattern::new(client.clone(), "utxos_500pct_to_1000pct_in_profit".to_string()),
            _300pct_to_500pct_in_profit: NuplRealizedSupplyUnrealizedPattern::new(client.clone(), "utxos_300pct_to_500pct_in_profit".to_string()),
            _200pct_to_300pct_in_profit: NuplRealizedSupplyUnrealizedPattern::new(client.clone(), "utxos_200pct_to_300pct_in_profit".to_string()),
            _100pct_to_200pct_in_profit: NuplRealizedSupplyUnrealizedPattern::new(client.clone(), "utxos_100pct_to_200pct_in_profit".to_string()),
            _90pct_to_100pct_in_profit: NuplRealizedSupplyUnrealizedPattern::new(client.clone(), "utxos_90pct_to_100pct_in_profit".to_string()),
            _80pct_to_90pct_in_profit: NuplRealizedSupplyUnrealizedPattern::new(client.clone(), "utxos_80pct_to_90pct_in_profit".to_string()),
            _70pct_to_80pct_in_profit: NuplRealizedSupplyUnrealizedPattern::new(client.clone(), "utxos_70pct_to_80pct_in_profit".to_string()),
            _60pct_to_70pct_in_profit: NuplRealizedSupplyUnrealizedPattern::new(client.clone(), "utxos_60pct_to_70pct_in_profit".to_string()),
            _50pct_to_60pct_in_profit: NuplRealizedSupplyUnrealizedPattern::new(client.clone(), "utxos_50pct_to_60pct_in_profit".to_string()),
            _40pct_to_50pct_in_profit: NuplRealizedSupplyUnrealizedPattern::new(client.clone(), "utxos_40pct_to_50pct_in_profit".to_string()),
            _30pct_to_40pct_in_profit: NuplRealizedSupplyUnrealizedPattern::new(client.clone(), "utxos_30pct_to_40pct_in_profit".to_string()),
            _20pct_to_30pct_in_profit: NuplRealizedSupplyUnrealizedPattern::new(client.clone(), "utxos_20pct_to_30pct_in_profit".to_string()),
            _10pct_to_20pct_in_profit: NuplRealizedSupplyUnrealizedPattern::new(client.clone(), "utxos_10pct_to_20pct_in_profit".to_string()),
            _0pct_to_10pct_in_profit: NuplRealizedSupplyUnrealizedPattern::new(client.clone(), "utxos_0pct_to_10pct_in_profit".to_string()),
            _0pct_to_10pct_in_loss: NuplRealizedSupplyUnrealizedPattern::new(client.clone(), "utxos_0pct_to_10pct_in_loss".to_string()),
            _10pct_to_20pct_in_loss: NuplRealizedSupplyUnrealizedPattern::new(client.clone(), "utxos_10pct_to_20pct_in_loss".to_string()),
            _20pct_to_30pct_in_loss: NuplRealizedSupplyUnrealizedPattern::new(client.clone(), "utxos_20pct_to_30pct_in_loss".to_string()),
            _30pct_to_40pct_in_loss: NuplRealizedSupplyUnrealizedPattern::new(client.clone(), "utxos_30pct_to_40pct_in_loss".to_string()),
            _40pct_to_50pct_in_loss: NuplRealizedSupplyUnrealizedPattern::new(client.clone(), "utxos_40pct_to_50pct_in_loss".to_string()),
            _50pct_to_60pct_in_loss: NuplRealizedSupplyUnrealizedPattern::new(client.clone(), "utxos_50pct_to_60pct_in_loss".to_string()),
            _60pct_to_70pct_in_loss: NuplRealizedSupplyUnrealizedPattern::new(client.clone(), "utxos_60pct_to_70pct_in_loss".to_string()),
            _70pct_to_80pct_in_loss: NuplRealizedSupplyUnrealizedPattern::new(client.clone(), "utxos_70pct_to_80pct_in_loss".to_string()),
            _80pct_to_90pct_in_loss: NuplRealizedSupplyUnrealizedPattern::new(client.clone(), "utxos_80pct_to_90pct_in_loss".to_string()),
            _90pct_to_100pct_in_loss: NuplRealizedSupplyUnrealizedPattern::new(client.clone(), "utxos_90pct_to_100pct_in_loss".to_string()),
        }
    }
}

/// Series tree node.
pub struct SeriesTree_Cohorts_Utxo_Profitability_Profit {
    pub all: NuplRealizedSupplyUnrealizedPattern,
    pub _10pct: NuplRealizedSupplyUnrealizedPattern,
    pub _20pct: NuplRealizedSupplyUnrealizedPattern,
    pub _30pct: NuplRealizedSupplyUnrealizedPattern,
    pub _40pct: NuplRealizedSupplyUnrealizedPattern,
    pub _50pct: NuplRealizedSupplyUnrealizedPattern,
    pub _60pct: NuplRealizedSupplyUnrealizedPattern,
    pub _70pct: NuplRealizedSupplyUnrealizedPattern,
    pub _80pct: NuplRealizedSupplyUnrealizedPattern,
    pub _90pct: NuplRealizedSupplyUnrealizedPattern,
    pub _100pct: NuplRealizedSupplyUnrealizedPattern,
    pub _200pct: NuplRealizedSupplyUnrealizedPattern,
    pub _300pct: NuplRealizedSupplyUnrealizedPattern,
    pub _500pct: NuplRealizedSupplyUnrealizedPattern,
}

impl SeriesTree_Cohorts_Utxo_Profitability_Profit {
    pub fn new(client: Arc<BrkClientBase>, base_path: String) -> Self {
        Self {
            all: NuplRealizedSupplyUnrealizedPattern::new(client.clone(), "utxos_in_profit".to_string()),
            _10pct: NuplRealizedSupplyUnrealizedPattern::new(client.clone(), "utxos_over_10pct_in_profit".to_string()),
            _20pct: NuplRealizedSupplyUnrealizedPattern::new(client.clone(), "utxos_over_20pct_in_profit".to_string()),
            _30pct: NuplRealizedSupplyUnrealizedPattern::new(client.clone(), "utxos_over_30pct_in_profit".to_string()),
            _40pct: NuplRealizedSupplyUnrealizedPattern::new(client.clone(), "utxos_over_40pct_in_profit".to_string()),
            _50pct: NuplRealizedSupplyUnrealizedPattern::new(client.clone(), "utxos_over_50pct_in_profit".to_string()),
            _60pct: NuplRealizedSupplyUnrealizedPattern::new(client.clone(), "utxos_over_60pct_in_profit".to_string()),
            _70pct: NuplRealizedSupplyUnrealizedPattern::new(client.clone(), "utxos_over_70pct_in_profit".to_string()),
            _80pct: NuplRealizedSupplyUnrealizedPattern::new(client.clone(), "utxos_over_80pct_in_profit".to_string()),
            _90pct: NuplRealizedSupplyUnrealizedPattern::new(client.clone(), "utxos_over_90pct_in_profit".to_string()),
            _100pct: NuplRealizedSupplyUnrealizedPattern::new(client.clone(), "utxos_over_100pct_in_profit".to_string()),
            _200pct: NuplRealizedSupplyUnrealizedPattern::new(client.clone(), "utxos_over_200pct_in_profit".to_string()),
            _300pct: NuplRealizedSupplyUnrealizedPattern::new(client.clone(), "utxos_over_300pct_in_profit".to_string()),
            _500pct: NuplRealizedSupplyUnrealizedPattern::new(client.clone(), "utxos_over_500pct_in_profit".to_string()),
        }
    }
}

/// Series tree node.
pub struct SeriesTree_Cohorts_Utxo_Profitability_Loss {
    pub all: NuplRealizedSupplyUnrealizedPattern,
    pub _10pct: NuplRealizedSupplyUnrealizedPattern,
    pub _20pct: NuplRealizedSupplyUnrealizedPattern,
    pub _30pct: NuplRealizedSupplyUnrealizedPattern,
    pub _40pct: NuplRealizedSupplyUnrealizedPattern,
    pub _50pct: NuplRealizedSupplyUnrealizedPattern,
    pub _60pct: NuplRealizedSupplyUnrealizedPattern,
    pub _70pct: NuplRealizedSupplyUnrealizedPattern,
    pub _80pct: NuplRealizedSupplyUnrealizedPattern,
}

impl SeriesTree_Cohorts_Utxo_Profitability_Loss {
    pub fn new(client: Arc<BrkClientBase>, base_path: String) -> Self {
        Self {
            all: NuplRealizedSupplyUnrealizedPattern::new(client.clone(), "utxos_in_loss".to_string()),
            _10pct: NuplRealizedSupplyUnrealizedPattern::new(client.clone(), "utxos_over_10pct_in_loss".to_string()),
            _20pct: NuplRealizedSupplyUnrealizedPattern::new(client.clone(), "utxos_over_20pct_in_loss".to_string()),
            _30pct: NuplRealizedSupplyUnrealizedPattern::new(client.clone(), "utxos_over_30pct_in_loss".to_string()),
            _40pct: NuplRealizedSupplyUnrealizedPattern::new(client.clone(), "utxos_over_40pct_in_loss".to_string()),
            _50pct: NuplRealizedSupplyUnrealizedPattern::new(client.clone(), "utxos_over_50pct_in_loss".to_string()),
            _60pct: NuplRealizedSupplyUnrealizedPattern::new(client.clone(), "utxos_over_60pct_in_loss".to_string()),
            _70pct: NuplRealizedSupplyUnrealizedPattern::new(client.clone(), "utxos_over_70pct_in_loss".to_string()),
            _80pct: NuplRealizedSupplyUnrealizedPattern::new(client.clone(), "utxos_over_80pct_in_loss".to_string()),
        }
    }
}

/// Series tree node.
pub struct SeriesTree_Cohorts_Utxo_Matured {
    pub under_1h: AverageBlockCumulativeSumPattern3,
    pub _1h_to_1d: AverageBlockCumulativeSumPattern3,
    pub _1d_to_1w: AverageBlockCumulativeSumPattern3,
    pub _1w_to_1m: AverageBlockCumulativeSumPattern3,
    pub _1m_to_2m: AverageBlockCumulativeSumPattern3,
    pub _2m_to_3m: AverageBlockCumulativeSumPattern3,
    pub _3m_to_4m: AverageBlockCumulativeSumPattern3,
    pub _4m_to_5m: AverageBlockCumulativeSumPattern3,
    pub _5m_to_6m: AverageBlockCumulativeSumPattern3,
    pub _6m_to_1y: AverageBlockCumulativeSumPattern3,
    pub _1y_to_2y: AverageBlockCumulativeSumPattern3,
    pub _2y_to_3y: AverageBlockCumulativeSumPattern3,
    pub _3y_to_4y: AverageBlockCumulativeSumPattern3,
    pub _4y_to_5y: AverageBlockCumulativeSumPattern3,
    pub _5y_to_6y: AverageBlockCumulativeSumPattern3,
    pub _6y_to_7y: AverageBlockCumulativeSumPattern3,
    pub _7y_to_8y: AverageBlockCumulativeSumPattern3,
    pub _8y_to_10y: AverageBlockCumulativeSumPattern3,
    pub _10y_to_12y: AverageBlockCumulativeSumPattern3,
    pub _12y_to_15y: AverageBlockCumulativeSumPattern3,
    pub over_15y: AverageBlockCumulativeSumPattern3,
}

impl SeriesTree_Cohorts_Utxo_Matured {
    pub fn new(client: Arc<BrkClientBase>, base_path: String) -> Self {
        Self {
            under_1h: AverageBlockCumulativeSumPattern3::new(client.clone(), "utxos_under_1h_old_matured_supply".to_string()),
            _1h_to_1d: AverageBlockCumulativeSumPattern3::new(client.clone(), "utxos_1h_to_1d_old_matured_supply".to_string()),
            _1d_to_1w: AverageBlockCumulativeSumPattern3::new(client.clone(), "utxos_1d_to_1w_old_matured_supply".to_string()),
            _1w_to_1m: AverageBlockCumulativeSumPattern3::new(client.clone(), "utxos_1w_to_1m_old_matured_supply".to_string()),
            _1m_to_2m: AverageBlockCumulativeSumPattern3::new(client.clone(), "utxos_1m_to_2m_old_matured_supply".to_string()),
            _2m_to_3m: AverageBlockCumulativeSumPattern3::new(client.clone(), "utxos_2m_to_3m_old_matured_supply".to_string()),
            _3m_to_4m: AverageBlockCumulativeSumPattern3::new(client.clone(), "utxos_3m_to_4m_old_matured_supply".to_string()),
            _4m_to_5m: AverageBlockCumulativeSumPattern3::new(client.clone(), "utxos_4m_to_5m_old_matured_supply".to_string()),
            _5m_to_6m: AverageBlockCumulativeSumPattern3::new(client.clone(), "utxos_5m_to_6m_old_matured_supply".to_string()),
            _6m_to_1y: AverageBlockCumulativeSumPattern3::new(client.clone(), "utxos_6m_to_1y_old_matured_supply".to_string()),
            _1y_to_2y: AverageBlockCumulativeSumPattern3::new(client.clone(), "utxos_1y_to_2y_old_matured_supply".to_string()),
            _2y_to_3y: AverageBlockCumulativeSumPattern3::new(client.clone(), "utxos_2y_to_3y_old_matured_supply".to_string()),
            _3y_to_4y: AverageBlockCumulativeSumPattern3::new(client.clone(), "utxos_3y_to_4y_old_matured_supply".to_string()),
            _4y_to_5y: AverageBlockCumulativeSumPattern3::new(client.clone(), "utxos_4y_to_5y_old_matured_supply".to_string()),
            _5y_to_6y: AverageBlockCumulativeSumPattern3::new(client.clone(), "utxos_5y_to_6y_old_matured_supply".to_string()),
            _6y_to_7y: AverageBlockCumulativeSumPattern3::new(client.clone(), "utxos_6y_to_7y_old_matured_supply".to_string()),
            _7y_to_8y: AverageBlockCumulativeSumPattern3::new(client.clone(), "utxos_7y_to_8y_old_matured_supply".to_string()),
            _8y_to_10y: AverageBlockCumulativeSumPattern3::new(client.clone(), "utxos_8y_to_10y_old_matured_supply".to_string()),
            _10y_to_12y: AverageBlockCumulativeSumPattern3::new(client.clone(), "utxos_10y_to_12y_old_matured_supply".to_string()),
            _12y_to_15y: AverageBlockCumulativeSumPattern3::new(client.clone(), "utxos_12y_to_15y_old_matured_supply".to_string()),
            over_15y: AverageBlockCumulativeSumPattern3::new(client.clone(), "utxos_over_15y_old_matured_supply".to_string()),
        }
    }
}

/// Series tree node.
pub struct SeriesTree_Cohorts_Addr {
    pub over_amount: SeriesTree_Cohorts_Addr_OverAmount,
    pub amount_range: SeriesTree_Cohorts_Addr_AmountRange,
    pub under_amount: SeriesTree_Cohorts_Addr_UnderAmount,
}

impl SeriesTree_Cohorts_Addr {
    pub fn new(client: Arc<BrkClientBase>, base_path: String) -> Self {
        Self {
            over_amount: SeriesTree_Cohorts_Addr_OverAmount::new(client.clone(), format!("{base_path}_over_amount")),
            amount_range: SeriesTree_Cohorts_Addr_AmountRange::new(client.clone(), format!("{base_path}_amount_range")),
            under_amount: SeriesTree_Cohorts_Addr_UnderAmount::new(client.clone(), format!("{base_path}_under_amount")),
        }
    }
}

/// Series tree node.
pub struct SeriesTree_Cohorts_Addr_OverAmount {
    pub _1sat: ActivityAddrOutputsRealizedSupplyUnrealizedPattern,
    pub _10sats: ActivityAddrOutputsRealizedSupplyUnrealizedPattern,
    pub _100sats: ActivityAddrOutputsRealizedSupplyUnrealizedPattern,
    pub _1k_sats: ActivityAddrOutputsRealizedSupplyUnrealizedPattern,
    pub _10k_sats: ActivityAddrOutputsRealizedSupplyUnrealizedPattern,
    pub _100k_sats: ActivityAddrOutputsRealizedSupplyUnrealizedPattern,
    pub _1m_sats: ActivityAddrOutputsRealizedSupplyUnrealizedPattern,
    pub _10m_sats: ActivityAddrOutputsRealizedSupplyUnrealizedPattern,
    pub _1btc: ActivityAddrOutputsRealizedSupplyUnrealizedPattern,
    pub _10btc: ActivityAddrOutputsRealizedSupplyUnrealizedPattern,
    pub _100btc: ActivityAddrOutputsRealizedSupplyUnrealizedPattern,
    pub _1k_btc: ActivityAddrOutputsRealizedSupplyUnrealizedPattern,
    pub _10k_btc: ActivityAddrOutputsRealizedSupplyUnrealizedPattern,
}

impl SeriesTree_Cohorts_Addr_OverAmount {
    pub fn new(client: Arc<BrkClientBase>, base_path: String) -> Self {
        Self {
            _1sat: ActivityAddrOutputsRealizedSupplyUnrealizedPattern::new(client.clone(), "addrs_over_1sat".to_string()),
            _10sats: ActivityAddrOutputsRealizedSupplyUnrealizedPattern::new(client.clone(), "addrs_over_10sats".to_string()),
            _100sats: ActivityAddrOutputsRealizedSupplyUnrealizedPattern::new(client.clone(), "addrs_over_100sats".to_string()),
            _1k_sats: ActivityAddrOutputsRealizedSupplyUnrealizedPattern::new(client.clone(), "addrs_over_1k_sats".to_string()),
            _10k_sats: ActivityAddrOutputsRealizedSupplyUnrealizedPattern::new(client.clone(), "addrs_over_10k_sats".to_string()),
            _100k_sats: ActivityAddrOutputsRealizedSupplyUnrealizedPattern::new(client.clone(), "addrs_over_100k_sats".to_string()),
            _1m_sats: ActivityAddrOutputsRealizedSupplyUnrealizedPattern::new(client.clone(), "addrs_over_1m_sats".to_string()),
            _10m_sats: ActivityAddrOutputsRealizedSupplyUnrealizedPattern::new(client.clone(), "addrs_over_10m_sats".to_string()),
            _1btc: ActivityAddrOutputsRealizedSupplyUnrealizedPattern::new(client.clone(), "addrs_over_1btc".to_string()),
            _10btc: ActivityAddrOutputsRealizedSupplyUnrealizedPattern::new(client.clone(), "addrs_over_10btc".to_string()),
            _100btc: ActivityAddrOutputsRealizedSupplyUnrealizedPattern::new(client.clone(), "addrs_over_100btc".to_string()),
            _1k_btc: ActivityAddrOutputsRealizedSupplyUnrealizedPattern::new(client.clone(), "addrs_over_1k_btc".to_string()),
            _10k_btc: ActivityAddrOutputsRealizedSupplyUnrealizedPattern::new(client.clone(), "addrs_over_10k_btc".to_string()),
        }
    }
}

/// Series tree node.
pub struct SeriesTree_Cohorts_Addr_AmountRange {
    pub _0sats: ActivityAddrOutputsRealizedSupplyUnrealizedPattern,
    pub _1sat_to_10sats: ActivityAddrOutputsRealizedSupplyUnrealizedPattern,
    pub _10sats_to_100sats: ActivityAddrOutputsRealizedSupplyUnrealizedPattern,
    pub _100sats_to_1k_sats: ActivityAddrOutputsRealizedSupplyUnrealizedPattern,
    pub _1k_sats_to_10k_sats: ActivityAddrOutputsRealizedSupplyUnrealizedPattern,
    pub _10k_sats_to_100k_sats: ActivityAddrOutputsRealizedSupplyUnrealizedPattern,
    pub _100k_sats_to_1m_sats: ActivityAddrOutputsRealizedSupplyUnrealizedPattern,
    pub _1m_sats_to_10m_sats: ActivityAddrOutputsRealizedSupplyUnrealizedPattern,
    pub _10m_sats_to_1btc: ActivityAddrOutputsRealizedSupplyUnrealizedPattern,
    pub _1btc_to_10btc: ActivityAddrOutputsRealizedSupplyUnrealizedPattern,
    pub _10btc_to_100btc: ActivityAddrOutputsRealizedSupplyUnrealizedPattern,
    pub _100btc_to_1k_btc: ActivityAddrOutputsRealizedSupplyUnrealizedPattern,
    pub _1k_btc_to_10k_btc: ActivityAddrOutputsRealizedSupplyUnrealizedPattern,
    pub _10k_btc_to_100k_btc: ActivityAddrOutputsRealizedSupplyUnrealizedPattern,
    pub over_100k_btc: ActivityAddrOutputsRealizedSupplyUnrealizedPattern,
}

impl SeriesTree_Cohorts_Addr_AmountRange {
    pub fn new(client: Arc<BrkClientBase>, base_path: String) -> Self {
        Self {
            _0sats: ActivityAddrOutputsRealizedSupplyUnrealizedPattern::new(client.clone(), "addrs_0sats".to_string()),
            _1sat_to_10sats: ActivityAddrOutputsRealizedSupplyUnrealizedPattern::new(client.clone(), "addrs_1sat_to_10sats".to_string()),
            _10sats_to_100sats: ActivityAddrOutputsRealizedSupplyUnrealizedPattern::new(client.clone(), "addrs_10sats_to_100sats".to_string()),
            _100sats_to_1k_sats: ActivityAddrOutputsRealizedSupplyUnrealizedPattern::new(client.clone(), "addrs_100sats_to_1k_sats".to_string()),
            _1k_sats_to_10k_sats: ActivityAddrOutputsRealizedSupplyUnrealizedPattern::new(client.clone(), "addrs_1k_sats_to_10k_sats".to_string()),
            _10k_sats_to_100k_sats: ActivityAddrOutputsRealizedSupplyUnrealizedPattern::new(client.clone(), "addrs_10k_sats_to_100k_sats".to_string()),
            _100k_sats_to_1m_sats: ActivityAddrOutputsRealizedSupplyUnrealizedPattern::new(client.clone(), "addrs_100k_sats_to_1m_sats".to_string()),
            _1m_sats_to_10m_sats: ActivityAddrOutputsRealizedSupplyUnrealizedPattern::new(client.clone(), "addrs_1m_sats_to_10m_sats".to_string()),
            _10m_sats_to_1btc: ActivityAddrOutputsRealizedSupplyUnrealizedPattern::new(client.clone(), "addrs_10m_sats_to_1btc".to_string()),
            _1btc_to_10btc: ActivityAddrOutputsRealizedSupplyUnrealizedPattern::new(client.clone(), "addrs_1btc_to_10btc".to_string()),
            _10btc_to_100btc: ActivityAddrOutputsRealizedSupplyUnrealizedPattern::new(client.clone(), "addrs_10btc_to_100btc".to_string()),
            _100btc_to_1k_btc: ActivityAddrOutputsRealizedSupplyUnrealizedPattern::new(client.clone(), "addrs_100btc_to_1k_btc".to_string()),
            _1k_btc_to_10k_btc: ActivityAddrOutputsRealizedSupplyUnrealizedPattern::new(client.clone(), "addrs_1k_btc_to_10k_btc".to_string()),
            _10k_btc_to_100k_btc: ActivityAddrOutputsRealizedSupplyUnrealizedPattern::new(client.clone(), "addrs_10k_btc_to_100k_btc".to_string()),
            over_100k_btc: ActivityAddrOutputsRealizedSupplyUnrealizedPattern::new(client.clone(), "addrs_over_100k_btc".to_string()),
        }
    }
}

/// Series tree node.
pub struct SeriesTree_Cohorts_Addr_UnderAmount {
    pub _10sats: ActivityAddrOutputsRealizedSupplyUnrealizedPattern,
    pub _100sats: ActivityAddrOutputsRealizedSupplyUnrealizedPattern,
    pub _1k_sats: ActivityAddrOutputsRealizedSupplyUnrealizedPattern,
    pub _10k_sats: ActivityAddrOutputsRealizedSupplyUnrealizedPattern,
    pub _100k_sats: ActivityAddrOutputsRealizedSupplyUnrealizedPattern,
    pub _1m_sats: ActivityAddrOutputsRealizedSupplyUnrealizedPattern,
    pub _10m_sats: ActivityAddrOutputsRealizedSupplyUnrealizedPattern,
    pub _1btc: ActivityAddrOutputsRealizedSupplyUnrealizedPattern,
    pub _10btc: ActivityAddrOutputsRealizedSupplyUnrealizedPattern,
    pub _100btc: ActivityAddrOutputsRealizedSupplyUnrealizedPattern,
    pub _1k_btc: ActivityAddrOutputsRealizedSupplyUnrealizedPattern,
    pub _10k_btc: ActivityAddrOutputsRealizedSupplyUnrealizedPattern,
    pub _100k_btc: ActivityAddrOutputsRealizedSupplyUnrealizedPattern,
}

impl SeriesTree_Cohorts_Addr_UnderAmount {
    pub fn new(client: Arc<BrkClientBase>, base_path: String) -> Self {
        Self {
            _10sats: ActivityAddrOutputsRealizedSupplyUnrealizedPattern::new(client.clone(), "addrs_under_10sats".to_string()),
            _100sats: ActivityAddrOutputsRealizedSupplyUnrealizedPattern::new(client.clone(), "addrs_under_100sats".to_string()),
            _1k_sats: ActivityAddrOutputsRealizedSupplyUnrealizedPattern::new(client.clone(), "addrs_under_1k_sats".to_string()),
            _10k_sats: ActivityAddrOutputsRealizedSupplyUnrealizedPattern::new(client.clone(), "addrs_under_10k_sats".to_string()),
            _100k_sats: ActivityAddrOutputsRealizedSupplyUnrealizedPattern::new(client.clone(), "addrs_under_100k_sats".to_string()),
            _1m_sats: ActivityAddrOutputsRealizedSupplyUnrealizedPattern::new(client.clone(), "addrs_under_1m_sats".to_string()),
            _10m_sats: ActivityAddrOutputsRealizedSupplyUnrealizedPattern::new(client.clone(), "addrs_under_10m_sats".to_string()),
            _1btc: ActivityAddrOutputsRealizedSupplyUnrealizedPattern::new(client.clone(), "addrs_under_1btc".to_string()),
            _10btc: ActivityAddrOutputsRealizedSupplyUnrealizedPattern::new(client.clone(), "addrs_under_10btc".to_string()),
            _100btc: ActivityAddrOutputsRealizedSupplyUnrealizedPattern::new(client.clone(), "addrs_under_100btc".to_string()),
            _1k_btc: ActivityAddrOutputsRealizedSupplyUnrealizedPattern::new(client.clone(), "addrs_under_1k_btc".to_string()),
            _10k_btc: ActivityAddrOutputsRealizedSupplyUnrealizedPattern::new(client.clone(), "addrs_under_10k_btc".to_string()),
            _100k_btc: ActivityAddrOutputsRealizedSupplyUnrealizedPattern::new(client.clone(), "addrs_under_100k_btc".to_string()),
        }
    }
}

/// Main BRK client with series tree and API methods.
pub struct BrkClient {
    base: Arc<BrkClientBase>,
    series: SeriesTree,
}

impl BrkClient {
    /// Client version.
    pub const VERSION: &'static str = "v0.3.6";

    /// Create a new client with the given base URL.
    pub fn new(base_url: impl Into<String>) -> Self {
        let base = Arc::new(BrkClientBase::new(base_url));
        let series = SeriesTree::new(base.clone(), String::new());
        Self { base, series }
    }

    /// Create a new client with options.
    pub fn with_options(options: BrkClientOptions) -> Self {
        let base = Arc::new(BrkClientBase::with_options(options));
        let series = SeriesTree::new(base.clone(), String::new());
        Self { base, series }
    }

    /// Get the series tree for navigating series.
    pub fn series(&self) -> &SeriesTree {
        &self.series
    }

    /// Create a dynamic series endpoint builder for any series/index combination.
    ///
    /// Use this for programmatic access when the series name is determined at runtime.
    /// For type-safe access, use the `series()` tree instead.
    ///
    /// # Example
    /// ```ignore
    /// let data = client.series("realized_price", Index::Height)
    ///     .last(10)
    ///     .json::<f64>()?;
    /// ```
    pub fn series_endpoint(&self, series: impl Into<SeriesName>, index: Index) -> SeriesEndpoint<serde_json::Value> {
        SeriesEndpoint::new(
            self.base.clone(),
            Arc::from(series.into().as_str()),
            index,
        )
    }

    /// Create a dynamic date-based series endpoint builder.
    ///
    /// Returns `Err` if the index is not date-based.
    pub fn date_series_endpoint(&self, series: impl Into<SeriesName>, index: Index) -> Result<DateSeriesEndpoint<serde_json::Value>> {
        if !index.is_date_based() {
            return Err(BrkError { message: format!("{} is not a date-based index", index.name()) });
        }
        Ok(DateSeriesEndpoint::new(
            self.base.clone(),
            Arc::from(series.into().as_str()),
            index,
        ))
    }

    /// Decode a mainnet Bitcoin address into the BRK address type and raw payload bytes.
    pub fn decode_address_payload(address: &str) -> Result<AddressPayload> {
        decode_address_payload(address)
    }

    /// Compute the RapidHash v3 hash-prefix for raw address payload bytes.
    pub fn address_payload_hash_prefix(payload: &[u8], nibbles: usize) -> Result<String> {
        address_payload_hash_prefix(payload, nibbles)
    }

    /// Decode a mainnet Bitcoin address and compute its hash prefix.
    pub fn address_hash_prefix(address: &str, nibbles: usize) -> Result<AddressHashPrefix> {
        address_hash_prefix(address, nibbles)
    }

    /// Fetch address hash-prefix matches from raw payload bytes matching `addr_type` length.
    pub fn get_address_payload_hash_prefix_matches(&self, addr_type: OutputType, payload: &[u8], nibbles: usize) -> Result<AddrHashPrefixMatches> {
        validate_address_payload_for_type(addr_type, payload)?;
        let prefix = address_payload_hash_prefix(payload, nibbles)?;
        self.get_address_hash_prefix_matches(addr_type, &prefix)
    }

    /// Fetch address hash-prefix matches for a mainnet Bitcoin address.
    pub fn get_address_hash_prefix_matches_for_address(&self, address: &str, nibbles: usize) -> Result<AddrHashPrefixMatches> {
        let hashed = address_hash_prefix(address, nibbles)?;
        self.get_address_hash_prefix_matches(hashed.addr_type, &hashed.prefix)
    }

    /// Health check
    ///
    /// Liveness probe. Returns server identity, uptime, and indexed/computed heights from local state only (no bitcoind round-trip). For real chain-tip catch-up, see `/api/server/sync`.
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
    /// Returns the sync status of the indexer, including indexed height, tip height, blocks behind, and last indexed timestamp.
    ///
    /// Endpoint: `GET /api/server/sync`
    pub fn get_sync_status(&self) -> Result<SyncStatus> {
        self.base.get_json(&format!("/api/server/sync"))
    }

    /// Disk usage
    ///
    /// Returns the disk space used by BRK and Bitcoin data.
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
    pub fn get_series_count(&self) -> Result<Vec<SeriesCount>> {
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
        if let Some(v) = page { query.push(format!("page={}", v)); }
        if let Some(v) = per_page { query.push(format!("per_page={}", v)); }
        let query_str = if query.is_empty() { String::new() } else { format!("?{}", query.join("&")) };
        let path = format!("/api/series/list{}", query_str);
        self.base.get_json(&path)
    }

    /// Search series
    ///
    /// Fuzzy search for series by name. Supports partial matches and typos.
    ///
    /// Endpoint: `GET /api/series/search`
    pub fn search_series(&self, q: SeriesName, limit: Option<Limit>) -> Result<Vec<String>> {
        let mut query = Vec::new();
        query.push(format!("q={}", q));
        if let Some(v) = limit { query.push(format!("limit={}", v)); }
        let query_str = if query.is_empty() { String::new() } else { format!("?{}", query.join("&")) };
        let path = format!("/api/series/search{}", query_str);
        self.base.get_json(&path)
    }

    /// Get series info
    ///
    /// Returns the supported indexes and value type for the specified series.
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
    pub fn get_series(&self, series: SeriesName, index: Index, start: Option<RangeIndex>, end: Option<RangeIndex>, limit: Option<Limit>, format: Option<Format>) -> Result<FormatResponse<SeriesData>> {
        let mut query = Vec::new();
        if let Some(v) = start { query.push(format!("start={}", v)); }
        if let Some(v) = end { query.push(format!("end={}", v)); }
        if let Some(v) = limit { query.push(format!("limit={}", v)); }
        if let Some(v) = format { query.push(format!("format={}", v)); }
        let query_str = if query.is_empty() { String::new() } else { format!("?{}", query.join("&")) };
        let path = format!("/api/series/{series}/{}{}", index.name(), query_str);
        if format == Some(Format::CSV) {
            self.base.get_text(&path).map(FormatResponse::Csv)
        } else {
            self.base.get_json(&path).map(FormatResponse::Json)
        }
    }

    /// Get raw series data
    ///
    /// Returns just the data array without the SeriesData wrapper. Supports the same range and format parameters as the standard endpoint.
    ///
    /// Endpoint: `GET /api/series/{series}/{index}/data`
    pub fn get_series_data(&self, series: SeriesName, index: Index, start: Option<RangeIndex>, end: Option<RangeIndex>, limit: Option<Limit>, format: Option<Format>) -> Result<FormatResponse<Vec<bool>>> {
        let mut query = Vec::new();
        if let Some(v) = start { query.push(format!("start={}", v)); }
        if let Some(v) = end { query.push(format!("end={}", v)); }
        if let Some(v) = limit { query.push(format!("limit={}", v)); }
        if let Some(v) = format { query.push(format!("format={}", v)); }
        let query_str = if query.is_empty() { String::new() } else { format!("?{}", query.join("&")) };
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
        self.base.get_json(&format!("/api/series/{series}/{}/latest", index.name()))
    }

    /// Get series data length
    ///
    /// Returns the total number of data points for a series at the given index.
    ///
    /// Endpoint: `GET /api/series/{series}/{index}/len`
    pub fn get_series_len(&self, series: SeriesName, index: Index) -> Result<i64> {
        self.base.get_json(&format!("/api/series/{series}/{}/len", index.name()))
    }

    /// Get series version
    ///
    /// Returns the current version of a series. Changes when the series data is updated.
    ///
    /// Endpoint: `GET /api/series/{series}/{index}/version`
    pub fn get_series_version(&self, series: SeriesName, index: Index) -> Result<Version> {
        self.base.get_json(&format!("/api/series/{series}/{}/version", index.name()))
    }

    /// Bulk series data
    ///
    /// Fetch multiple series in a single request. Supports filtering by index and date range. Returns an array of SeriesData objects. For a single series, use `get_series` instead.
    ///
    /// Endpoint: `GET /api/series/bulk`
    pub fn get_series_bulk(&self, series: SeriesList, index: Index, start: Option<RangeIndex>, end: Option<RangeIndex>, limit: Option<Limit>, format: Option<Format>) -> Result<FormatResponse<Vec<SeriesData>>> {
        let mut query = Vec::new();
        query.push(format!("series={}", series));
        query.push(format!("index={}", index));
        if let Some(v) = start { query.push(format!("start={}", v)); }
        if let Some(v) = end { query.push(format!("end={}", v)); }
        if let Some(v) = limit { query.push(format!("limit={}", v)); }
        if let Some(v) = format { query.push(format!("format={}", v)); }
        let query_str = if query.is_empty() { String::new() } else { format!("?{}", query.join("&")) };
        let path = format!("/api/series/bulk{}", query_str);
        if format == Some(Format::CSV) {
            self.base.get_text(&path).map(FormatResponse::Csv)
        } else {
            self.base.get_json(&path).map(FormatResponse::Json)
        }
    }

    /// Available URPD cohorts
    ///
    /// Cohorts for which URPD data is available. Returns names like `all`, `sth`, `lth`, `utxos_under_1h_old`.
    ///
    /// Endpoint: `GET /api/urpd`
    pub fn list_urpd_cohorts(&self) -> Result<Vec<Cohort>> {
        self.base.get_json(&format!("/api/urpd"))
    }

    /// Available URPD dates
    ///
    /// Dates for which a URPD snapshot is available for the cohort. One entry per UTC day, sorted ascending.
    ///
    /// Endpoint: `GET /api/urpd/{cohort}/dates`
    pub fn list_urpd_dates(&self, cohort: Cohort) -> Result<Vec<Date>> {
        self.base.get_json(&format!("/api/urpd/{cohort}/dates"))
    }

    /// Latest URPD
    ///
    /// URPD for the most recent available date in the cohort. The response's `date` field echoes which date was served.
    ///
    /// See the URPD tag description for the response shape and `agg` options.
    ///
    /// Endpoint: `GET /api/urpd/{cohort}`
    pub fn get_urpd(&self, cohort: Cohort, agg: Option<UrpdAggregation>) -> Result<Urpd> {
        let mut query = Vec::new();
        if let Some(v) = agg { query.push(format!("agg={}", v)); }
        let query_str = if query.is_empty() { String::new() } else { format!("?{}", query.join("&")) };
        let path = format!("/api/urpd/{cohort}{}", query_str);
        self.base.get_json(&path)
    }

    /// URPD at date
    ///
    /// URPD for a (cohort, date) pair. Returns `{ cohort, date, aggregation, close, total_supply, buckets }` where each bucket is `{ price_floor, supply, realized_cap, unrealized_pnl }`.
    ///
    /// See the URPD tag description for unit conventions and `agg` options.
    ///
    /// Endpoint: `GET /api/urpd/{cohort}/{date}`
    pub fn get_urpd_at(&self, cohort: Cohort, date: &str, agg: Option<UrpdAggregation>) -> Result<Urpd> {
        let mut query = Vec::new();
        if let Some(v) = agg { query.push(format!("agg={}", v)); }
        let query_str = if query.is_empty() { String::new() } else { format!("?{}", query.join("&")) };
        let path = format!("/api/urpd/{cohort}/{date}{}", query_str);
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
        self.base.get_json(&format!("/api/v1/difficulty-adjustment"))
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
    /// Get historical BTC/USD price. Optionally specify a UNIX timestamp to get the price at that time.
    ///
    /// *[Mempool.space docs](https://mempool.space/docs/api/rest#get-historical-price)*
    ///
    /// Endpoint: `GET /api/v1/historical-price`
    pub fn get_historical_price(&self, timestamp: Option<Timestamp>) -> Result<HistoricalPrice> {
        let mut query = Vec::new();
        if let Some(v) = timestamp { query.push(format!("timestamp={}", v)); }
        let query_str = if query.is_empty() { String::new() } else { format!("?{}", query.join("&")) };
        let path = format!("/api/v1/historical-price{}", query_str);
        self.base.get_json(&path)
    }

    /// Address hash-prefix matches
    ///
    /// Find addresses by address type and by the first 1-16 hex nibbles of RapidHash v3 over the raw address payload bytes. Intended for privacy-preserving client-side wallet discovery without sending raw addresses or xpubs. Fetch metadata for the returned addresses through `/api/address/{address}`.
    ///
    /// Endpoint: `GET /api/address/hash-prefix/{addr_type}/{prefix}`
    pub fn get_address_hash_prefix_matches(&self, addr_type: OutputType, prefix: &str) -> Result<AddrHashPrefixMatches> {
        self.base.get_json(&format!("/api/address/hash-prefix/{addr_type}/{prefix}"))
    }

    /// Address information
    ///
    /// Retrieve address information including balance and transaction counts. Supports all standard Bitcoin address types (P2PKH, P2SH, P2WPKH, P2WSH, P2TR).
    ///
    /// *[Mempool.space docs](https://mempool.space/docs/api/rest#get-address)*
    ///
    /// Endpoint: `GET /api/address/{address}`
    pub fn get_address(&self, address: Addr) -> Result<AddrStats> {
        self.base.get_json(&format!("/api/address/{address}"))
    }

    /// Address transactions
    ///
    /// Get transaction history for an address, newest first. Returns up to 50 mempool transactions plus a confirmed page sized to fill the response to 50 total (chain floor of 25, so 25-50 confirmed depending on mempool weight). To paginate further confirmed history, use `/address/{address}/txs/chain/{last_seen_txid}`.
    ///
    /// *[Mempool.space docs](https://mempool.space/docs/api/rest#get-address-transactions)*
    ///
    /// Endpoint: `GET /api/address/{address}/txs`
    pub fn get_address_txs(&self, address: Addr) -> Result<Vec<Transaction>> {
        self.base.get_json(&format!("/api/address/{address}/txs"))
    }

    /// Address confirmed transactions
    ///
    /// Get the first 25 confirmed transactions for an address. For pagination, use the path-style form `/txs/chain/{last_seen_txid}`.
    ///
    /// *[Mempool.space docs](https://mempool.space/docs/api/rest#get-address-transactions-chain)*
    ///
    /// Endpoint: `GET /api/address/{address}/txs/chain`
    pub fn get_address_confirmed_txs(&self, address: Addr) -> Result<Vec<Transaction>> {
        self.base.get_json(&format!("/api/address/{address}/txs/chain"))
    }

    /// Address confirmed transactions (paginated)
    ///
    /// Get the next 25 confirmed transactions strictly older than `after_txid` (Esplora-canonical pagination form, matches mempool.space).
    ///
    /// *[Mempool.space docs](https://mempool.space/docs/api/rest#get-address-transactions-chain)*
    ///
    /// Endpoint: `GET /api/address/{address}/txs/chain/{after_txid}`
    pub fn get_address_confirmed_txs_after(&self, address: Addr, after_txid: Txid) -> Result<Vec<Transaction>> {
        self.base.get_json(&format!("/api/address/{address}/txs/chain/{after_txid}"))
    }

    /// Address mempool transactions
    ///
    /// Get unconfirmed transactions for an address from the mempool, newest first (up to 50).
    ///
    /// *[Mempool.space docs](https://mempool.space/docs/api/rest#get-address-transactions-mempool)*
    ///
    /// Endpoint: `GET /api/address/{address}/txs/mempool`
    pub fn get_address_mempool_txs(&self, address: Addr) -> Result<Vec<Transaction>> {
        self.base.get_json(&format!("/api/address/{address}/txs/mempool"))
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
        self.base.get_json(&format!("/api/v1/validate-address/{address}"))
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
    /// Find the block closest to a given UNIX timestamp.
    ///
    /// *[Mempool.space docs](https://mempool.space/docs/api/rest#get-block-timestamp)*
    ///
    /// Endpoint: `GET /api/v1/mining/blocks/timestamp/{timestamp}`
    pub fn get_block_by_timestamp(&self, timestamp: Timestamp) -> Result<BlockTimestamp> {
        self.base.get_json(&format!("/api/v1/mining/blocks/timestamp/{timestamp}"))
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
        self.base.get_text(&format!("/api/block/{hash}/txid/{index}"))
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
    pub fn get_block_txs_from_index(&self, hash: BlockHash, start_index: BlockTxIndex) -> Result<Vec<Transaction>> {
        self.base.get_json(&format!("/api/block/{hash}/txs/{start_index}"))
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
        self.base.get_json(&format!("/api/v1/mining/pools/{time_period}"))
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
        self.base.get_json(&format!("/api/v1/mining/hashrate/pools"))
    }

    /// All pools hashrate
    ///
    /// Get hashrate data for all mining pools for a time period. Valid periods: `1m`, `3m`, `6m`, `1y`, `2y`, `3y`.
    ///
    /// *[Mempool.space docs](https://mempool.space/docs/api/rest#get-mining-pool-hashrates)*
    ///
    /// Endpoint: `GET /api/v1/mining/hashrate/pools/{time_period}`
    pub fn get_pools_hashrate_by_period(&self, time_period: TimePeriod) -> Result<Vec<PoolHashrateEntry>> {
        self.base.get_json(&format!("/api/v1/mining/hashrate/pools/{time_period}"))
    }

    /// Mining pool hashrate
    ///
    /// Get hashrate history for a specific mining pool.
    ///
    /// *[Mempool.space docs](https://mempool.space/docs/api/rest#get-mining-pool-hashrate)*
    ///
    /// Endpoint: `GET /api/v1/mining/pool/{slug}/hashrate`
    pub fn get_pool_hashrate(&self, slug: PoolSlug) -> Result<Vec<PoolHashrateEntry>> {
        self.base.get_json(&format!("/api/v1/mining/pool/{slug}/hashrate"))
    }

    /// Mining pool blocks
    ///
    /// Get the 10 most recent blocks mined by a specific pool.
    ///
    /// *[Mempool.space docs](https://mempool.space/docs/api/rest#get-mining-pool-blocks)*
    ///
    /// Endpoint: `GET /api/v1/mining/pool/{slug}/blocks`
    pub fn get_pool_blocks(&self, slug: PoolSlug) -> Result<Vec<BlockInfoV1>> {
        self.base.get_json(&format!("/api/v1/mining/pool/{slug}/blocks"))
    }

    /// Mining pool blocks from height
    ///
    /// Get 10 blocks mined by a specific pool before (and including) the given height.
    ///
    /// *[Mempool.space docs](https://mempool.space/docs/api/rest#get-mining-pool-blocks)*
    ///
    /// Endpoint: `GET /api/v1/mining/pool/{slug}/blocks/{height}`
    pub fn get_pool_blocks_from(&self, slug: PoolSlug, height: Height) -> Result<Vec<BlockInfoV1>> {
        self.base.get_json(&format!("/api/v1/mining/pool/{slug}/blocks/{height}"))
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
        self.base.get_json(&format!("/api/v1/mining/hashrate/{time_period}"))
    }

    /// Difficulty adjustments (all time)
    ///
    /// Get historical difficulty adjustments including timestamp, block height, difficulty value, and percentage change.
    ///
    /// *[Mempool.space docs](https://mempool.space/docs/api/rest#get-difficulty-adjustments)*
    ///
    /// Endpoint: `GET /api/v1/mining/difficulty-adjustments`
    pub fn get_difficulty_adjustments(&self) -> Result<Vec<DifficultyAdjustmentEntry>> {
        self.base.get_json(&format!("/api/v1/mining/difficulty-adjustments"))
    }

    /// Difficulty adjustments
    ///
    /// Get historical difficulty adjustments for a time period. Valid periods: `24h`, `3d`, `1w`, `1m`, `3m`, `6m`, `1y`, `2y`, `3y`.
    ///
    /// *[Mempool.space docs](https://mempool.space/docs/api/rest#get-difficulty-adjustments)*
    ///
    /// Endpoint: `GET /api/v1/mining/difficulty-adjustments/{time_period}`
    pub fn get_difficulty_adjustments_by_period(&self, time_period: TimePeriod) -> Result<Vec<DifficultyAdjustmentEntry>> {
        self.base.get_json(&format!("/api/v1/mining/difficulty-adjustments/{time_period}"))
    }

    /// Mining reward statistics
    ///
    /// Get mining reward statistics for the last N blocks including total rewards, fees, and transaction count.
    ///
    /// *[Mempool.space docs](https://mempool.space/docs/api/rest#get-reward-stats)*
    ///
    /// Endpoint: `GET /api/v1/mining/reward-stats/{block_count}`
    pub fn get_reward_stats(&self, block_count: i64) -> Result<RewardStats> {
        self.base.get_json(&format!("/api/v1/mining/reward-stats/{block_count}"))
    }

    /// Block fees
    ///
    /// Get average total fees per block for a time period. Valid periods: `24h`, `3d`, `1w`, `1m`, `3m`, `6m`, `1y`, `2y`, `3y`.
    ///
    /// *[Mempool.space docs](https://mempool.space/docs/api/rest#get-block-fees)*
    ///
    /// Endpoint: `GET /api/v1/mining/blocks/fees/{time_period}`
    pub fn get_block_fees(&self, time_period: TimePeriod) -> Result<Vec<BlockFeesEntry>> {
        self.base.get_json(&format!("/api/v1/mining/blocks/fees/{time_period}"))
    }

    /// Block rewards
    ///
    /// Get average coinbase reward (subsidy + fees) per block for a time period. Valid periods: `24h`, `3d`, `1w`, `1m`, `3m`, `6m`, `1y`, `2y`, `3y`.
    ///
    /// *[Mempool.space docs](https://mempool.space/docs/api/rest#get-block-rewards)*
    ///
    /// Endpoint: `GET /api/v1/mining/blocks/rewards/{time_period}`
    pub fn get_block_rewards(&self, time_period: TimePeriod) -> Result<Vec<BlockRewardsEntry>> {
        self.base.get_json(&format!("/api/v1/mining/blocks/rewards/{time_period}"))
    }

    /// Block fee rates
    ///
    /// Get block fee rate percentiles (min, 10th, 25th, median, 75th, 90th, max) for a time period. Valid periods: `24h`, `3d`, `1w`, `1m`, `3m`, `6m`, `1y`, `2y`, `3y`.
    ///
    /// *[Mempool.space docs](https://mempool.space/docs/api/rest#get-block-feerates)*
    ///
    /// Endpoint: `GET /api/v1/mining/blocks/fee-rates/{time_period}`
    pub fn get_block_fee_rates(&self, time_period: TimePeriod) -> Result<Vec<BlockFeeRatesEntry>> {
        self.base.get_json(&format!("/api/v1/mining/blocks/fee-rates/{time_period}"))
    }

    /// Block sizes and weights
    ///
    /// Get average block sizes and weights for a time period. Valid periods: `24h`, `3d`, `1w`, `1m`, `3m`, `6m`, `1y`, `2y`, `3y`.
    ///
    /// *[Mempool.space docs](https://mempool.space/docs/api/rest#get-sizes-weights)*
    ///
    /// Endpoint: `GET /api/v1/mining/blocks/sizes-weights/{time_period}`
    pub fn get_block_sizes_weights(&self, time_period: TimePeriod) -> Result<BlockSizesWeights> {
        self.base.get_json(&format!("/api/v1/mining/blocks/sizes-weights/{time_period}"))
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

    /// Precise recommended fees
    ///
    /// Recommended fee rates with sub-integer precision.
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
    /// Returns an opaque hash that changes whenever the projected next block changes. Same value as the mempool ETag. Useful as a freshness/liveness signal: if it stays constant for tens of seconds on a live network, the mempool sync loop has stalled.
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
    /// Like `/api/v1/replacements`, but limited to trees where at least one predecessor was non-signaling (full-RBF).
    ///
    /// *[Mempool.space docs](https://mempool.space/docs/api/rest#get-fullrbf-replacements)*
    ///
    /// Endpoint: `GET /api/v1/fullrbf/replacements`
    pub fn get_fullrbf_replacements(&self) -> Result<Vec<ReplacementNode>> {
        self.base.get_json(&format!("/api/v1/fullrbf/replacements"))
    }

    /// Projected next block template
    ///
    /// Bitcoin Core's `getblocktemplate` selection: full transaction bodies in GBT order with aggregate stats. The returned `hash` is an opaque content token; pass it as `<hash>` on `/api/v1/mempool/block-template/diff/{hash}` to fetch deltas instead of refetching the whole template.
    ///
    /// Endpoint: `GET /api/v1/mempool/block-template`
    pub fn get_block_template(&self) -> Result<BlockTemplate> {
        self.base.get_json(&format!("/api/v1/mempool/block-template"))
    }

    /// Block template diff since hash
    ///
    /// Delta of the projected next block since `<hash>`. `order` is the full new template in order: each entry is either a number (index into the prior template the client cached at `<hash>`) or a transaction object (new body to insert at this position). Walk `order` once to rebuild; `removed` is a convenience list of txids that left so clients can evict cached bodies. After applying, use the response `hash` as `<hash>` on the next call to keep iterating. Returns `404` when `<hash>` has aged out of server history; clients should fall back to `/api/v1/mempool/block-template`.
    ///
    /// Endpoint: `GET /api/v1/mempool/block-template/diff/{hash}`
    pub fn get_block_template_diff(&self, hash: NextBlockHash) -> Result<BlockTemplateDiff> {
        self.base.get_json(&format!("/api/v1/mempool/block-template/diff/{hash}"))
    }

    /// Live BTC/USD price
    ///
    /// Returns the current BTC/USD price in dollars, derived from on-chain round-dollar output patterns in the last 12 blocks plus mempool.
    ///
    /// Endpoint: `GET /api/mempool/price`
    pub fn get_live_price(&self) -> Result<Dollars> {
        self.base.get_json(&format!("/api/mempool/price"))
    }

    /// Live BTC/USD price
    ///
    /// Current BTC/USD price in dollars. Same value as `/api/mempool/price`. Confirmed per-height history is available at `/api/vecs/height-to-price`.
    ///
    /// Endpoint: `GET /api/oracle/price`
    pub fn get_oracle_price(&self) -> Result<Dollars> {
        self.base.get_json(&format!("/api/oracle/price"))
    }

    /// Live payment output histogram
    ///
    /// Live smoothed histogram of oracle-eligible payment outputs, binned by output value on the oracle log scale. It combines the committed oracle window with the forming mempool block. A flat array of log-scale bins.
    ///
    /// Endpoint: `GET /api/oracle/histogram/payments/live`
    pub fn get_oracle_histogram_payments_live(&self) -> Result<Vec<i64>> {
        self.base.get_json(&format!("/api/oracle/histogram/payments/live"))
    }

    /// Payment output histogram at height or day
    ///
    /// Smoothed histogram of oracle-eligible payment outputs for a confirmed point. A block height (`840000`) gives that block's oracle payment histogram; a calendar date (`YYYY-MM-DD`) gives the average of that day's per-block payment histograms. A flat array of log-scale bins.
    ///
    /// Endpoint: `GET /api/oracle/histogram/payments/{point}`
    pub fn get_oracle_histogram_payments(&self, point: &str) -> Result<Vec<i64>> {
        self.base.get_json(&format!("/api/oracle/histogram/payments/{point}"))
    }

    /// Live output value histogram
    ///
    /// Live unfiltered output value histogram for the forming mempool block. Every live output is binned by value on the oracle log scale; no oracle payment filters are applied. A flat array of log-scale bins, all zero when no mempool is configured.
    ///
    /// Endpoint: `GET /api/oracle/histogram/outputs/live`
    pub fn get_oracle_histogram_outputs_live(&self) -> Result<Vec<i64>> {
        self.base.get_json(&format!("/api/oracle/histogram/outputs/live"))
    }

    /// Output value histogram at height or day
    ///
    /// Unfiltered output value histogram for a confirmed point. A block height (`840000`) gives every output in that block, coinbase included, binned by value on the oracle log scale; a calendar date (`YYYY-MM-DD`) sums every block that day. A flat array of log-scale bins.
    ///
    /// Endpoint: `GET /api/oracle/histogram/outputs/{point}`
    pub fn get_oracle_histogram_outputs(&self, point: &str) -> Result<Vec<i64>> {
        self.base.get_json(&format!("/api/oracle/histogram/outputs/{point}"))
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
        self.base.get_text(&format!("/api/tx/{txid}/merkleblock-proof"))
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
        self.base.get_json(&format!("/api/tx/{txid}/outspend/{vout}"))
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
        for v in txId { query.push(format!("txId[]={}", v)); }
        let query_str = if query.is_empty() { String::new() } else { format!("?{}", query.join("&")) };
        let path = format!("/api/v1/transaction-times{}", query_str);
        self.base.get_json(&path)
    }

    /// Broadcast transaction
    ///
    /// Broadcast a raw transaction to the network. The transaction should be provided as hex in the request body. The txid will be returned on success.
    ///
    /// *[Mempool.space docs](https://mempool.space/docs/api/rest#post-transaction)*
    ///
    /// Endpoint: `POST /api/tx`
    pub fn post_tx(&self, body: &str) -> Result<Txid> {
        self.base.post_json(&format!("/api/tx"), body)
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
    /// Compact OpenAPI specification optimized for LLM consumption. Removes redundant fields while preserving essential API information. Full spec available at `/openapi.json`.
    ///
    /// Endpoint: `GET /api.json`
    pub fn get_api(&self) -> Result<serde_json::Value> {
        self.base.get_json(&format!("/api.json"))
    }

}
