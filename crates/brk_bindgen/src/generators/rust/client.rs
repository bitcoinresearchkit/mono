//! Rust base client and pattern factory generation.

use std::fmt::Write;

use crate::{
    ClientMetadata, GenericSyntax, IndexSetPattern, RustSyntax, StructuralPattern,
    escape_rust_keyword, generate_parameterized_field, index_to_field_name, to_snake_case,
};

/// Generate import statements.
pub fn generate_imports(output: &mut String) {
    writeln!(
        output,
        r#"use std::str::FromStr;
use std::sync::Arc;
use std::ops::{{Bound, RangeBounds}};
use serde::de::DeserializeOwned;
pub use brk_cohort::*;
pub use brk_types::*;

"#
    )
    .unwrap();
}

/// Generate the base BrkClientBase struct and error types.
pub fn generate_base_client(output: &mut String) {
    writeln!(
        output,
        r#"/// Error type for BRK client operations.
#[derive(Debug)]
pub struct BrkError {{
    pub message: String,
}}

impl std::fmt::Display for BrkError {{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {{
        write!(f, "{{}}", self.message)
    }}
}}

impl std::error::Error for BrkError {{}}

/// Result type for BRK client operations.
pub type Result<T> = std::result::Result<T, BrkError>;

/// BRK address type and raw payload bytes used by the hash-prefix index.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AddressPayload {{
    pub addr_type: OutputType,
    pub payload: Vec<u8>,
}}

/// BRK address type and leading hex nibbles of the address-payload hash.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AddressHashPrefix {{
    pub addr_type: OutputType,
    pub prefix: String,
}}

/// Compute the RapidHash v3 hash-prefix used by `/api/address/hash-prefix/{{addr_type}}/{{prefix}}`.
pub fn address_payload_hash_prefix(payload: &[u8], nibbles: usize) -> Result<String> {{
    if payload.is_empty() {{
        return Err(BrkError {{ message: "Expected a non-empty address payload".to_string() }});
    }}
    if payload.len() > 65 {{
        return Err(BrkError {{ message: "Expected at most 65 address payload bytes".to_string() }});
    }}
    if !(1..=16).contains(&nibbles) {{
        return Err(BrkError {{ message: "Expected hash-prefix length from 1 to 16 hex nibbles".to_string() }});
    }}
    Ok(format!("{{:016x}}", rapidhash::v3::rapidhash_v3(payload))[..nibbles].to_string())
}}

fn validate_address_payload_for_type(addr_type: OutputType, payload: &[u8]) -> Result<()> {{
    let expected: &[usize] = match addr_type {{
        OutputType::P2A => &[2],
        OutputType::P2PK33 => &[33],
        OutputType::P2PK65 => &[65],
        OutputType::P2PKH | OutputType::P2SH | OutputType::P2WPKH => &[20],
        OutputType::P2WSH | OutputType::P2TR => &[32],
        OutputType::P2MS | OutputType::OpReturn | OutputType::Empty | OutputType::Unknown => {{
            return Err(BrkError {{ message: format!("Unsupported address type for address payload hash-prefix: {{addr_type:?}}") }});
        }},
    }};

    if !expected.contains(&payload.len()) {{
        let joined = expected
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join(" or ");
        return Err(BrkError {{ message: format!("Expected {{addr_type}} address payload length {{joined}} bytes") }});
    }}

    Ok(())
}}

#[cfg(test)]
mod address_payload_tests {{
    use super::*;

    #[test]
    fn p2pk_payload_lengths_are_distinct() {{
        assert!(validate_address_payload_for_type(OutputType::P2PK33, &[0; 33]).is_ok());
        assert!(validate_address_payload_for_type(OutputType::P2PK65, &[0; 65]).is_ok());
        assert!(validate_address_payload_for_type(OutputType::P2PK33, &[0; 65]).is_err());
        assert!(validate_address_payload_for_type(OutputType::P2PK65, &[0; 33]).is_err());
    }}
}}

/// Decode a mainnet Bitcoin address into the BRK address type and raw payload bytes.
pub fn decode_address_payload(address: &str) -> Result<AddressPayload> {{
    if address.is_empty() {{
        return Err(BrkError {{ message: "Expected an address string".to_string() }});
    }}
    let addr_bytes = AddrBytes::from_str(address).map_err(|e| BrkError {{ message: e.to_string() }})?;
    let addr_type = OutputType::from(&addr_bytes);

    Ok(AddressPayload {{
        addr_type,
        payload: addr_bytes.as_slice().to_vec(),
    }})
}}

/// Decode a mainnet Bitcoin address and compute its hash prefix.
pub fn address_hash_prefix(address: &str, nibbles: usize) -> Result<AddressHashPrefix> {{
    let decoded = decode_address_payload(address)?;
    Ok(AddressHashPrefix {{
        addr_type: decoded.addr_type,
        prefix: address_payload_hash_prefix(&decoded.payload, nibbles)?,
    }})
}}

/// Options for configuring the BRK client.
#[derive(Debug, Clone)]
pub struct BrkClientOptions {{
    pub base_url: String,
    pub timeout_secs: u64,
}}

impl Default for BrkClientOptions {{
    fn default() -> Self {{
        Self {{
            base_url: "http://localhost:3000".to_string(),
            timeout_secs: 30,
        }}
    }}
}}

/// Base HTTP client for making requests. Reuses connections via ureq::Agent.
#[derive(Debug, Clone)]
pub struct BrkClientBase {{
    agent: ureq::Agent,
    base_url: String,
}}

impl BrkClientBase {{
    /// Create a new client with the given base URL.
    pub fn new(base_url: impl Into<String>) -> Self {{
        Self::with_options(BrkClientOptions {{ base_url: base_url.into(), ..Default::default() }})
    }}

    /// Create a new client with options.
    pub fn with_options(options: BrkClientOptions) -> Self {{
        let agent = ureq::Agent::config_builder()
            .timeout_global(Some(std::time::Duration::from_secs(options.timeout_secs)))
            .build()
            .into();
        Self {{
            agent,
            base_url: options.base_url.trim_end_matches('/').to_string(),
        }}
    }}

    fn url(&self, path: &str) -> String {{
        format!("{{}}{{}}", self.base_url, path)
    }}

    /// Make a GET request and deserialize JSON response.
    pub fn get_json<T: DeserializeOwned>(&self, path: &str) -> Result<T> {{
        self.agent.get(&self.url(path))
            .call()
            .and_then(|mut r| r.body_mut().read_json())
            .map_err(|e| BrkError {{ message: e.to_string() }})
    }}

    /// Make a GET request and return raw text response.
    pub fn get_text(&self, path: &str) -> Result<String> {{
        self.agent.get(&self.url(path))
            .call()
            .and_then(|mut r| r.body_mut().read_to_string())
            .map_err(|e| BrkError {{ message: e.to_string() }})
    }}

    /// Make a GET request and return raw bytes response.
    pub fn get_bytes(&self, path: &str) -> Result<Vec<u8>> {{
        self.agent.get(&self.url(path))
            .call()
            .and_then(|mut r| r.body_mut().read_to_vec())
            .map_err(|e| BrkError {{ message: e.to_string() }})
    }}

    /// Make a POST request and deserialize JSON response.
    pub fn post_json<T: DeserializeOwned>(&self, path: &str, body: &str) -> Result<T> {{
        self.agent.post(&self.url(path))
            .send(body)
            .and_then(|mut r| r.body_mut().read_json())
            .map_err(|e| BrkError {{ message: e.to_string() }})
    }}

    /// Make a POST request and return raw text response.
    pub fn post_text(&self, path: &str, body: &str) -> Result<String> {{
        self.agent.post(&self.url(path))
            .send(body)
            .and_then(|mut r| r.body_mut().read_to_string())
            .map_err(|e| BrkError {{ message: e.to_string() }})
    }}

    /// Make a POST request and return raw bytes response.
    pub fn post_bytes(&self, path: &str, body: &str) -> Result<Vec<u8>> {{
        self.agent.post(&self.url(path))
            .send(body)
            .and_then(|mut r| r.body_mut().read_to_vec())
            .map_err(|e| BrkError {{ message: e.to_string() }})
    }}
}}

/// Build series name with suffix.
#[inline]
fn _m(acc: &str, s: &str) -> String {{
    if s.is_empty() {{ acc.to_string() }}
    else if acc.is_empty() {{ s.to_string() }}
    else {{ format!("{{acc}}_{{s}}") }}
}}

/// Build series name with prefix.
#[inline]
fn _p(prefix: &str, acc: &str) -> String {{
    if acc.is_empty() {{ prefix.to_string() }} else {{ format!("{{prefix}}_{{acc}}") }}
}}

"#
    )
    .unwrap();
}

/// Generate the SeriesPattern trait.
pub fn generate_series_pattern_trait(output: &mut String) {
    writeln!(
        output,
        r#"/// Non-generic trait for series patterns (usable in collections).
pub trait AnySeriesPattern {{
    /// Get the series name.
    fn name(&self) -> &str;

    /// Get the list of available indexes for this series.
    fn indexes(&self) -> &'static [Index];
}}

/// Generic trait for series patterns with endpoint access.
pub trait SeriesPattern<T>: AnySeriesPattern {{
    /// Get an endpoint builder for a specific index, if supported.
    fn get(&self, index: Index) -> Option<SeriesEndpoint<T>>;
}}

"#
    )
    .unwrap();
}

/// Generate the SeriesEndpoint structs with typestate pattern.
pub fn generate_endpoint(output: &mut String) {
    writeln!(
        output,
        r#"/// Shared endpoint configuration.
#[derive(Clone)]
struct EndpointConfig {{
    client: Arc<BrkClientBase>,
    name: Arc<str>,
    index: Index,
    start: Option<i64>,
    end: Option<i64>,
}}

impl EndpointConfig {{
    fn new(client: Arc<BrkClientBase>, name: Arc<str>, index: Index) -> Self {{
        Self {{ client, name, index, start: None, end: None }}
    }}

    fn path(&self) -> String {{
        format!("/api/series/{{}}/{{}}", self.name, self.index.name())
    }}

    fn build_path(&self, format: Option<&str>) -> String {{
        let mut params = Vec::new();
        if let Some(s) = self.start {{ params.push(format!("start={{}}", s)); }}
        if let Some(e) = self.end {{ params.push(format!("end={{}}", e)); }}
        if let Some(fmt) = format {{ params.push(format!("format={{}}", fmt)); }}
        let p = self.path();
        if params.is_empty() {{ p }} else {{ format!("{{}}?{{}}", p, params.join("&")) }}
    }}

    fn get_json<T: DeserializeOwned>(&self, format: Option<&str>) -> Result<T> {{
        self.client.get_json(&self.build_path(format))
    }}

    fn get_text(&self, format: Option<&str>) -> Result<String> {{
        self.client.get_text(&self.build_path(format))
    }}

    fn get_len(&self) -> Result<i64> {{
        self.client.get_json(&format!("/api/series/{{}}/{{}}/len", self.name, self.index.name()))
    }}

    fn get_version(&self) -> Result<Version> {{
        self.client.get_json(&format!("/api/series/{{}}/{{}}/version", self.name, self.index.name()))
    }}
}}

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
pub struct SeriesEndpoint<T, D = SeriesData<T>> {{
    config: EndpointConfig,
    _marker: std::marker::PhantomData<fn() -> (T, D)>,
}}

/// Builder for date-based series endpoint queries.
///
/// Like `SeriesEndpoint` but returns `DateSeriesData` and provides
/// date-based access methods (`get_date`, `date_range`).
pub type DateSeriesEndpoint<T> = SeriesEndpoint<T, DateSeriesData<T>>;

impl<T: DeserializeOwned, D: DeserializeOwned> SeriesEndpoint<T, D> {{
    pub fn new(client: Arc<BrkClientBase>, name: Arc<str>, index: Index) -> Self {{
        Self {{ config: EndpointConfig::new(client, name, index), _marker: std::marker::PhantomData }}
    }}

    /// Select a specific index position.
    pub fn get(mut self, index: usize) -> SingleItemBuilder<T, D> {{
        self.config.start = Some(index as i64);
        self.config.end = Some(index as i64 + 1);
        SingleItemBuilder {{ config: self.config, _marker: std::marker::PhantomData }}
    }}

    /// Select a range using Rust range syntax.
    ///
    /// # Examples
    /// ```ignore
    /// endpoint.range(..10)      // first 10
    /// endpoint.range(100..110)  // indices 100-109
    /// endpoint.range(100..)     // from 100 to end
    /// ```
    pub fn range<R: RangeBounds<usize>>(mut self, range: R) -> RangeBuilder<T, D> {{
        self.config.start = match range.start_bound() {{
            Bound::Included(&n) => Some(n as i64),
            Bound::Excluded(&n) => Some(n as i64 + 1),
            Bound::Unbounded => None,
        }};
        self.config.end = match range.end_bound() {{
            Bound::Included(&n) => Some(n as i64 + 1),
            Bound::Excluded(&n) => Some(n as i64),
            Bound::Unbounded => None,
        }};
        RangeBuilder {{ config: self.config, _marker: std::marker::PhantomData }}
    }}

    /// Take the first n items.
    pub fn take(self, n: usize) -> RangeBuilder<T, D> {{
        self.range(..n)
    }}

    /// Take the last n items.
    pub fn last(mut self, n: usize) -> RangeBuilder<T, D> {{
        if n == 0 {{
            self.config.end = Some(0);
        }} else {{
            self.config.start = Some(-(n as i64));
        }}
        RangeBuilder {{ config: self.config, _marker: std::marker::PhantomData }}
    }}

    /// Skip the first n items. Chain with `take(n)` to get a range.
    pub fn skip(mut self, n: usize) -> SkippedBuilder<T, D> {{
        self.config.start = Some(n as i64);
        SkippedBuilder {{ config: self.config, _marker: std::marker::PhantomData }}
    }}

    /// Fetch all data as parsed JSON.
    pub fn fetch(self) -> Result<D> {{
        self.config.get_json(None)
    }}

    /// Fetch all data as CSV string.
    pub fn fetch_csv(self) -> Result<String> {{
        self.config.get_text(Some("csv"))
    }}

    /// Total number of data points for this series.
    #[allow(clippy::len_without_is_empty)]
    pub fn len(&self) -> Result<i64> {{
        self.config.get_len()
    }}

    /// Current version of the series.
    pub fn version(&self) -> Result<Version> {{
        self.config.get_version()
    }}

    /// Get the base endpoint path.
    pub fn path(&self) -> String {{
        self.config.path()
    }}
}}

/// Date-specific methods available only on `DateSeriesEndpoint`.
impl<T: DeserializeOwned> SeriesEndpoint<T, DateSeriesData<T>> {{
    /// Select a specific date position (for day-precision or coarser indexes).
    pub fn get_date(self, date: Date) -> SingleItemBuilder<T, DateSeriesData<T>> {{
        let index = self.config.index.date_to_index(date).unwrap_or(0);
        self.get(index)
    }}

    /// Select a date range (for day-precision or coarser indexes).
    pub fn date_range(self, start: Date, end: Date) -> RangeBuilder<T, DateSeriesData<T>> {{
        let s = self.config.index.date_to_index(start).unwrap_or(0);
        let e = self.config.index.date_to_index(end).unwrap_or(0);
        self.range(s..e)
    }}

    /// Select a specific timestamp position (works for all date-based indexes including sub-daily).
    pub fn get_timestamp(self, ts: Timestamp) -> SingleItemBuilder<T, DateSeriesData<T>> {{
        let index = self.config.index.timestamp_to_index(ts).unwrap_or(0);
        self.get(index)
    }}

    /// Select a timestamp range (works for all date-based indexes including sub-daily).
    pub fn timestamp_range(self, start: Timestamp, end: Timestamp) -> RangeBuilder<T, DateSeriesData<T>> {{
        let s = self.config.index.timestamp_to_index(start).unwrap_or(0);
        let e = self.config.index.timestamp_to_index(end).unwrap_or(0);
        self.range(s..e)
    }}
}}

/// Builder for single item access.
pub struct SingleItemBuilder<T, D = SeriesData<T>> {{
    config: EndpointConfig,
    _marker: std::marker::PhantomData<fn() -> (T, D)>,
}}

/// Date-aware single item builder.
pub type DateSingleItemBuilder<T> = SingleItemBuilder<T, DateSeriesData<T>>;

impl<T: DeserializeOwned, D: DeserializeOwned> SingleItemBuilder<T, D> {{
    /// Fetch the single item.
    pub fn fetch(self) -> Result<D> {{
        self.config.get_json(None)
    }}

    /// Fetch the single item as CSV.
    pub fn fetch_csv(self) -> Result<String> {{
        self.config.get_text(Some("csv"))
    }}
}}

/// Builder after calling `skip(n)`. Chain with `take(n)` to specify count.
pub struct SkippedBuilder<T, D = SeriesData<T>> {{
    config: EndpointConfig,
    _marker: std::marker::PhantomData<fn() -> (T, D)>,
}}

/// Date-aware skipped builder.
pub type DateSkippedBuilder<T> = SkippedBuilder<T, DateSeriesData<T>>;

impl<T: DeserializeOwned, D: DeserializeOwned> SkippedBuilder<T, D> {{
    /// Take n items after the skipped position.
    pub fn take(mut self, n: usize) -> RangeBuilder<T, D> {{
        let start = self.config.start.unwrap_or(0);
        self.config.end = Some(start + n as i64);
        RangeBuilder {{ config: self.config, _marker: std::marker::PhantomData }}
    }}

    /// Fetch from the skipped position to the end.
    pub fn fetch(self) -> Result<D> {{
        self.config.get_json(None)
    }}

    /// Fetch from the skipped position to the end as CSV.
    pub fn fetch_csv(self) -> Result<String> {{
        self.config.get_text(Some("csv"))
    }}
}}

/// Builder with range fully specified.
pub struct RangeBuilder<T, D = SeriesData<T>> {{
    config: EndpointConfig,
    _marker: std::marker::PhantomData<fn() -> (T, D)>,
}}

/// Date-aware range builder.
pub type DateRangeBuilder<T> = RangeBuilder<T, DateSeriesData<T>>;

impl<T: DeserializeOwned, D: DeserializeOwned> RangeBuilder<T, D> {{
    /// Fetch the range as parsed JSON.
    pub fn fetch(self) -> Result<D> {{
        self.config.get_json(None)
    }}

    /// Fetch the range as CSV string.
    pub fn fetch_csv(self) -> Result<String> {{
        self.config.get_text(Some("csv"))
    }}
}}

"#
    )
    .unwrap();
}

/// Generate index accessor structs.
pub fn generate_index_accessors(output: &mut String, patterns: &[IndexSetPattern]) {
    if patterns.is_empty() {
        return;
    }

    // Generate static index arrays
    writeln!(output, "// Static index arrays").unwrap();
    for (i, pattern) in patterns.iter().enumerate() {
        write!(output, "const _I{}: &[Index] = &[", i + 1).unwrap();
        for (j, index) in pattern.indexes.iter().enumerate() {
            if j > 0 {
                write!(output, ", ").unwrap();
            }
            write!(output, "Index::{}", index).unwrap();
        }
        writeln!(output, "];").unwrap();
    }
    writeln!(output).unwrap();

    // Generate helper functions
    writeln!(
        output,
        r#"#[inline]
fn _ep<T: DeserializeOwned>(c: &Arc<BrkClientBase>, n: &Arc<str>, i: Index) -> SeriesEndpoint<T> {{
    SeriesEndpoint::new(c.clone(), n.clone(), i)
}}

#[inline]
fn _dep<T: DeserializeOwned>(c: &Arc<BrkClientBase>, n: &Arc<str>, i: Index) -> DateSeriesEndpoint<T> {{
    DateSeriesEndpoint::new(c.clone(), n.clone(), i)
}}
"#
    )
    .unwrap();

    // Generate index accessor structs
    writeln!(output, "// Index accessor structs\n").unwrap();

    for (i, pattern) in patterns.iter().enumerate() {
        let by_name = format!("{}By", pattern.name);
        let idx_const = format!("_I{}", i + 1);

        // Generate the "By" struct
        writeln!(output, "pub struct {}<T> {{ client: Arc<BrkClientBase>, name: Arc<str>, _marker: std::marker::PhantomData<T> }}", by_name).unwrap();
        writeln!(output, "impl<T: DeserializeOwned> {}<T> {{", by_name).unwrap();
        for index in &pattern.indexes {
            let method_name = index_to_field_name(index);
            if index.is_date_based() {
                writeln!(
                    output,
                    "    pub fn {}(&self) -> DateSeriesEndpoint<T> {{ _dep(&self.client, &self.name, Index::{}) }}",
                    method_name, index
                )
                .unwrap();
            } else {
                writeln!(
                    output,
                    "    pub fn {}(&self) -> SeriesEndpoint<T> {{ _ep(&self.client, &self.name, Index::{}) }}",
                    method_name, index
                )
                .unwrap();
            }
        }
        writeln!(output, "}}\n").unwrap();

        // Generate the main accessor struct
        writeln!(
            output,
            "pub struct {}<T> {{ name: Arc<str>, pub by: {}<T> }}",
            pattern.name, by_name
        )
        .unwrap();
        writeln!(output, "impl<T: DeserializeOwned> {}<T> {{", pattern.name).unwrap();
        writeln!(
            output,
            "    pub fn new(client: Arc<BrkClientBase>, name: String) -> Self {{ let name: Arc<str> = name.into(); Self {{ name: name.clone(), by: {} {{ client, name, _marker: std::marker::PhantomData }} }} }}",
            by_name
        )
        .unwrap();
        writeln!(output, "    pub fn name(&self) -> &str {{ &self.name }}").unwrap();
        writeln!(output, "}}\n").unwrap();

        // Implement AnySeriesPattern trait
        writeln!(
            output,
            "impl<T> AnySeriesPattern for {}<T> {{ fn name(&self) -> &str {{ &self.name }} fn indexes(&self) -> &'static [Index] {{ {} }} }}",
            pattern.name, idx_const
        )
        .unwrap();

        // Implement SeriesPattern<T> trait
        writeln!(
            output,
            "impl<T: DeserializeOwned> SeriesPattern<T> for {}<T> {{ fn get(&self, index: Index) -> Option<SeriesEndpoint<T>> {{ {}.contains(&index).then(|| _ep(&self.by.client, &self.by.name, index)) }} }}\n",
            pattern.name, idx_const
        )
        .unwrap();
    }
}

/// Generate structural pattern structs.
pub fn generate_pattern_structs(
    output: &mut String,
    patterns: &[StructuralPattern],
    metadata: &ClientMetadata,
) {
    if patterns.is_empty() {
        return;
    }

    writeln!(output, "// Reusable pattern structs\n").unwrap();

    for pattern in patterns {
        let generic_params = if pattern.is_generic { "<T>" } else { "" };

        // Generate struct definition
        writeln!(output, "/// Pattern struct for repeated tree structure.").unwrap();
        writeln!(output, "pub struct {}{} {{", pattern.name, generic_params).unwrap();

        for field in &pattern.fields {
            let field_name = escape_rust_keyword(&to_snake_case(&field.name));
            let type_annotation = metadata.field_type_annotation(
                field,
                pattern.is_generic,
                None,
                GenericSyntax::RUST,
            );
            writeln!(output, "    pub {}: {},", field_name, type_annotation).unwrap();
        }

        writeln!(output, "}}\n").unwrap();

        // Skip constructor for non-parameterizable patterns (inlined at tree level)
        if !metadata.is_parameterizable(&pattern.name) {
            continue;
        }

        let impl_generic = if pattern.is_generic {
            "<T: DeserializeOwned>"
        } else {
            ""
        };
        writeln!(
            output,
            "impl{} {}{} {{",
            impl_generic, pattern.name, generic_params
        )
        .unwrap();

        writeln!(
            output,
            "    /// Create a new pattern node with accumulated series name."
        )
        .unwrap();
        if pattern.is_templated() {
            writeln!(
                output,
                "    pub fn new(client: Arc<BrkClientBase>, acc: String, disc: String) -> Self {{"
            )
            .unwrap();
        } else {
            writeln!(
                output,
                "    pub fn new(client: Arc<BrkClientBase>, acc: String) -> Self {{"
            )
            .unwrap();
        }
        writeln!(output, "        Self {{").unwrap();

        let syntax = RustSyntax;
        for field in &pattern.fields {
            generate_parameterized_field(output, &syntax, field, pattern, metadata, "            ");
        }

        writeln!(output, "        }}").unwrap();
        writeln!(output, "    }}").unwrap();
        writeln!(output, "}}\n").unwrap();
    }
}
