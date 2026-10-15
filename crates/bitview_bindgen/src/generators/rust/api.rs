//! Rust API method generation.

use std::fmt::Write;

use crate::{Endpoint, VERSION, generators::write_description, to_snake_case};

use super::types::js_type_to_rust;

/// Generate the main BitviewClient struct.
pub(crate) fn generate_main_client(output: &mut String, endpoints: &[Endpoint]) {
    writeln!(
        output,
        r#"/// Main Bitview client with series tree and API methods.
pub struct BitviewClient {{
    base: Arc<BitviewClientBase>,
    series: OnceLock<Box<SeriesTree>>,
}}

impl BitviewClient {{
    /// Client version.
    pub const VERSION: &'static str = "v{VERSION}";

    /// Create a new client with the given base URL.
    pub fn new(base_url: impl Into<String>) -> Self {{
        let base = Arc::new(BitviewClientBase::new(base_url));
        Self {{ base, series: OnceLock::new() }}
    }}

    /// Create a new client with options.
    pub fn with_options(options: BitviewClientOptions) -> Self {{
        let base = Arc::new(BitviewClientBase::with_options(options));
        Self {{ base, series: OnceLock::new() }}
    }}

    /// Get the series tree for navigating series.
    pub fn series(&self) -> &SeriesTree {{
        self.series
            .get_or_init(|| Box::new(SeriesTree::build(self.base.clone(), String::new())))
    }}

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
    pub fn series_endpoint(&self, series: impl Into<SeriesName>, index: Index) -> SeriesEndpoint<serde_json::Value> {{
        SeriesEndpoint::new(
            self.base.clone(),
            Arc::from(series.into().as_str()),
            index,
        )
    }}

    /// Create a dynamic date-based series endpoint builder.
    ///
    /// Returns `Err` if the index is not date-based.
    pub fn date_series_endpoint(&self, series: impl Into<SeriesName>, index: Index) -> Result<DateSeriesEndpoint<serde_json::Value>> {{
        if !index.is_date_based() {{
            return Err(BitviewError::new(format!("{{}} is not a date-based index", index.name())));
        }}
        Ok(SeriesEndpoint::new(
            self.base.clone(),
            Arc::from(series.into().as_str()),
            index,
        ))
    }}
"#,
        VERSION = VERSION
    )
    .unwrap();

    output.push_str(r#"    /// Fetch address hash-prefix matches from raw payload bytes matching `address_type` length.
    pub fn get_address_payload_hash_prefix_matches(&self, address_type: OutputType, payload: &[u8], nibbles: usize) -> Result<AddressHashPrefixMatches> {
        validate_address_payload_for_type(address_type, payload)?;
        let prefix = address_payload_hash_prefix(payload, nibbles)?;
        self.get_address_hash_prefix_matches(address_type, &prefix)
    }

    /// Fetch address hash-prefix matches for a mainnet Bitcoin address.
    pub fn get_address_hash_prefix_matches_for_address(&self, address: &str, nibbles: usize) -> Result<AddressHashPrefixMatches> {
        let hashed = address_hash_prefix(address, nibbles)?;
        self.get_address_hash_prefix_matches(hashed.address_type, &hashed.prefix)
    }

"#);

    generate_api_methods(output, endpoints);

    writeln!(output, "}}").unwrap();
}

/// Generate API methods from OpenAPI endpoints.
fn generate_api_methods(output: &mut String, endpoints: &[Endpoint]) {
    for endpoint in endpoints
        .iter()
        .filter(|endpoint| endpoint.should_generate())
    {
        generate_method(output, endpoint);
    }
}

fn generate_method(output: &mut String, endpoint: &Endpoint) {
    let post = endpoint.method == "POST";
    let (fetch_method, decode) = fetch(if post { "post" } else { "get" }, endpoint);

    write_method_doc(output, endpoint);

    let mut params = build_method_params(endpoint);
    let body_arg = match (post, endpoint.request_body.is_some()) {
        (false, _) => "",
        (true, true) => {
            params.push_str(", body: &str");
            ", body"
        }
        (true, false) => ", \"\"",
    };
    writeln!(
        output,
        "    pub fn {}(&self{params}) -> Result<{}> {{",
        endpoint_to_method_name(endpoint),
        build_return_type(endpoint)
    )
    .unwrap();

    let (path, index_arg) = build_path_template(endpoint);
    if endpoint.query_params.is_empty() {
        writeln!(
            output,
            "        self.base.{fetch_method}(&format!(\"{path}\"{index_arg}){body_arg}){decode}"
        )
        .unwrap();
    } else {
        write_query_assembly(output, endpoint, &path, index_arg);
        if endpoint.supports_csv && !post {
            writeln!(
                output,
                "        if format == Some(Format::CSV) {{
            self.base.get_text(&path).map(FormatResponse::Csv)
        }} else {{
            self.base.{fetch_method}(&path).map(FormatResponse::Json)
        }}"
            )
            .unwrap();
        } else {
            writeln!(
                output,
                "        self.base.{fetch_method}(&path{body_arg}){decode}"
            )
            .unwrap();
        }
    }

    writeln!(output, "    }}\n").unwrap();
}

/// The base client method reading the response, and how its body is decoded: named text is
/// numeric JSON (`Height`) or a string type (`BlockHash`, `Txid`, `Hex`); opaque text stays `String`.
fn fetch(verb: &str, endpoint: &Endpoint) -> (String, &'static str) {
    let (kind, decode) = if endpoint.returns_binary() {
        ("bytes", "")
    } else if endpoint.returns_json() || endpoint.response_kind.text_is_numeric() {
        ("json", "")
    } else if endpoint.schema_name().is_some() {
        ("text", ".and_then(parse_text)")
    } else {
        ("text", "")
    };
    (format!("{verb}_{kind}"), decode)
}

fn build_return_type(endpoint: &Endpoint) -> String {
    let base = if endpoint.returns_binary() {
        "Vec<u8>".to_string()
    } else {
        endpoint
            .schema_name()
            .map(js_type_to_rust)
            .unwrap_or_else(|| "String".to_string())
    };
    if endpoint.supports_csv {
        format!("FormatResponse<{}>", base)
    } else {
        base
    }
}

fn write_method_doc(output: &mut String, endpoint: &Endpoint) {
    let method_name = endpoint_to_method_name(endpoint);
    writeln!(
        output,
        "    /// {}",
        endpoint.summary.as_deref().unwrap_or(&method_name)
    )
    .unwrap();
    if let Some(desc) = &endpoint.description
        && endpoint.summary.as_ref() != Some(desc)
    {
        writeln!(output, "    ///").unwrap();
        write_description(output, desc, "    /// ", "    ///");
    }
    writeln!(output, "    ///").unwrap();
    writeln!(
        output,
        "    /// Endpoint: `{} {}`",
        endpoint.method.to_uppercase(),
        endpoint.path
    )
    .unwrap();
}

fn write_query_assembly(output: &mut String, endpoint: &Endpoint, path: &str, index_arg: &str) {
    writeln!(output, "        let mut query = Vec::new();").unwrap();
    for param in &endpoint.query_params {
        let ident = sanitize_ident(&param.name);
        let is_array = param.param_type.ends_with("[]");
        if is_array {
            writeln!(
                output,
                "        for v in {} {{ query.push(query_pair(\"{}\", v)); }}",
                ident, param.name
            )
            .unwrap();
        } else if param.required {
            writeln!(
                output,
                "        query.push(query_pair(\"{}\", {}));",
                param.name, ident
            )
            .unwrap();
        } else {
            writeln!(
                output,
                "        if let Some(v) = {} {{ query.push(query_pair(\"{}\", v)); }}",
                ident, param.name
            )
            .unwrap();
        }
    }
    writeln!(output, "        let query_str = if query.is_empty() {{ String::new() }} else {{ format!(\"?{{}}\", query.join(\"&\")) }};").unwrap();
    writeln!(
        output,
        "        let path = format!(\"{}{{}}\"{}, query_str);",
        path, index_arg
    )
    .unwrap();
}

fn endpoint_to_method_name(endpoint: &Endpoint) -> String {
    to_snake_case(&endpoint.operation_id)
}

fn build_method_params(endpoint: &Endpoint) -> String {
    let mut params = String::new();
    for param in &endpoint.path_params {
        let rust_type = param_type_to_rust(&param.param_type);
        write!(params, ", {}: {}", sanitize_ident(&param.name), rust_type).unwrap();
    }
    for param in &endpoint.query_params {
        let rust_type = param_type_to_rust(&param.param_type);
        let name = sanitize_ident(&param.name);
        if param.required {
            write!(params, ", {}: {}", name, rust_type).unwrap();
        } else {
            write!(params, ", {}: Option<{}>", name, rust_type).unwrap();
        }
    }
    params
}

/// Strip characters invalid in Rust identifiers (e.g. `[]` from `txId[]`).
fn sanitize_ident(name: &str) -> String {
    name.replace(['[', ']'], "")
}

/// Convert parameter type to Rust type for function signatures.
fn param_type_to_rust(param_type: &str) -> String {
    if let Some(inner) = param_type.strip_suffix("[]") {
        return format!("&[{}]", param_type_to_rust(inner));
    }
    match param_type {
        "string" | "*" => "&str".to_string(),
        "integer" | "number" => "i64".to_string(),
        "boolean" => "bool".to_string(),
        other => other.to_string(),
    }
}

/// Build path template and extra format args for Index params.
fn build_path_template(endpoint: &Endpoint) -> (String, &'static str) {
    let has_index_param = endpoint
        .path_params
        .iter()
        .any(|p| p.name == "index" && p.param_type == "Index");
    if has_index_param {
        (endpoint.path.replace("{index}", "{}"), ", index.name()")
    } else {
        (endpoint.path.clone(), "")
    }
}
