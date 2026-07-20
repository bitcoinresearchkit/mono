//! Rust API method generation.

use std::fmt::Write;

use crate::{Endpoint, VERSION, generators::write_description, to_snake_case};

use super::types::js_type_to_rust;

/// Generate the main BrkClient struct.
pub fn generate_main_client(output: &mut String, endpoints: &[Endpoint]) {
    writeln!(
        output,
        r#"/// Main BRK client with series tree and API methods.
pub struct BrkClient {{
    base: Arc<BrkClientBase>,
    series: SeriesTree,
}}

impl BrkClient {{
    /// Client version.
    pub const VERSION: &'static str = "v{VERSION}";

    /// Create a new client with the given base URL.
    pub fn new(base_url: impl Into<String>) -> Self {{
        let base = Arc::new(BrkClientBase::new(base_url));
        let series = SeriesTree::new(base.clone(), String::new());
        Self {{ base, series }}
    }}

    /// Create a new client with options.
    pub fn with_options(options: BrkClientOptions) -> Self {{
        let base = Arc::new(BrkClientBase::with_options(options));
        let series = SeriesTree::new(base.clone(), String::new());
        Self {{ base, series }}
    }}

    /// Get the series tree for navigating series.
    pub fn series(&self) -> &SeriesTree {{
        &self.series
    }}

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
            return Err(BrkError {{ message: format!("{{}} is not a date-based index", index.name()) }});
        }}
        Ok(DateSeriesEndpoint::new(
            self.base.clone(),
            Arc::from(series.into().as_str()),
            index,
        ))
    }}
"#,
        VERSION = VERSION
    )
    .unwrap();

    output.push_str(r#"    /// Decode a mainnet Bitcoin address into the BRK address type and raw payload bytes.
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

"#);

    generate_api_methods(output, endpoints);

    writeln!(output, "}}").unwrap();
}

/// Generate API methods from OpenAPI endpoints.
pub fn generate_api_methods(output: &mut String, endpoints: &[Endpoint]) {
    for endpoint in endpoints {
        if !endpoint.should_generate() {
            continue;
        }
        match endpoint.method.as_str() {
            "GET" => generate_get_method(output, endpoint),
            "POST" => generate_post_method(output, endpoint),
            _ => continue,
        }
    }
}

fn generate_get_method(output: &mut String, endpoint: &Endpoint) {
    let method_name = endpoint_to_method_name(endpoint);
    let return_type = build_return_type(endpoint);

    write_method_doc(output, endpoint);

    let params = build_method_params(endpoint);
    writeln!(
        output,
        "    pub fn {}(&self{}) -> Result<{}> {{",
        method_name, params, return_type
    )
    .unwrap();

    let (path, index_arg) = build_path_template(endpoint);
    let fetch_method = if endpoint.returns_binary() {
        "get_bytes"
    } else if endpoint.returns_json() {
        "get_json"
    } else {
        "get_text"
    };

    if endpoint.query_params.is_empty() {
        writeln!(
            output,
            "        self.base.{}(&format!(\"{}\"{}))",
            fetch_method, path, index_arg
        )
        .unwrap();
    } else {
        write_query_assembly(output, endpoint, &path, index_arg);

        if endpoint.supports_csv {
            writeln!(output, "        if format == Some(Format::CSV) {{").unwrap();
            writeln!(
                output,
                "            self.base.get_text(&path).map(FormatResponse::Csv)"
            )
            .unwrap();
            writeln!(output, "        }} else {{").unwrap();
            writeln!(
                output,
                "            self.base.{}(&path).map(FormatResponse::Json)",
                fetch_method
            )
            .unwrap();
            writeln!(output, "        }}").unwrap();
        } else {
            writeln!(output, "        self.base.{}(&path)", fetch_method).unwrap();
        }
    }

    writeln!(output, "    }}\n").unwrap();
}

fn generate_post_method(output: &mut String, endpoint: &Endpoint) {
    let method_name = endpoint_to_method_name(endpoint);
    let return_type = build_return_type(endpoint);

    write_method_doc(output, endpoint);

    let mut params = build_method_params(endpoint);
    if endpoint.request_body.is_some() {
        params.push_str(", body: &str");
    }
    writeln!(
        output,
        "    pub fn {}(&self{}) -> Result<{}> {{",
        method_name, params, return_type
    )
    .unwrap();

    let (path, index_arg) = build_path_template(endpoint);
    let body_arg = if endpoint.request_body.is_some() {
        "body"
    } else {
        "\"\""
    };
    let fetch_method = if endpoint.returns_binary() {
        "post_bytes"
    } else if endpoint.returns_json() {
        "post_json"
    } else {
        "post_text"
    };

    if endpoint.query_params.is_empty() {
        writeln!(
            output,
            "        self.base.{}(&format!(\"{}\"{}), {})",
            fetch_method, path, index_arg, body_arg
        )
        .unwrap();
    } else {
        write_query_assembly(output, endpoint, &path, index_arg);
        writeln!(
            output,
            "        self.base.{}(&path, {})",
            fetch_method, body_arg
        )
        .unwrap();
    }

    writeln!(output, "    }}\n").unwrap();
}

fn build_return_type(endpoint: &Endpoint) -> String {
    let base = if endpoint.returns_binary() {
        "Vec<u8>".to_string()
    } else if endpoint.returns_text() {
        "String".to_string()
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
                "        for v in {} {{ query.push(format!(\"{}={{}}\", v)); }}",
                ident, param.name
            )
            .unwrap();
        } else if param.required {
            writeln!(
                output,
                "        query.push(format!(\"{}={{}}\", {}));",
                param.name, ident
            )
            .unwrap();
        } else {
            writeln!(
                output,
                "        if let Some(v) = {} {{ query.push(format!(\"{}={{}}\", v)); }}",
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
    to_snake_case(&endpoint.operation_name())
}

fn build_method_params(endpoint: &Endpoint) -> String {
    let mut params = Vec::new();
    for param in &endpoint.path_params {
        let rust_type = param_type_to_rust(&param.param_type);
        params.push(format!(", {}: {}", sanitize_ident(&param.name), rust_type));
    }
    for param in &endpoint.query_params {
        let rust_type = param_type_to_rust(&param.param_type);
        let name = sanitize_ident(&param.name);
        if param.required {
            params.push(format!(", {}: {}", name, rust_type));
        } else {
            params.push(format!(", {}: Option<{}>", name, rust_type));
        }
    }
    params.join("")
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
