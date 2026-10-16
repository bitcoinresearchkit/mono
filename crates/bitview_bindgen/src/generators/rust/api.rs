//! Rust API method generation.

use std::fmt::Write;

use crate::{Endpoint, ResponseKind, Ty, VERSION, generators::write_description, to_snake_case};

use super::types::{rust_param_type, rust_type};

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

    let (path, path_args) = build_path_template(endpoint);
    if endpoint.query_params.is_empty() {
        writeln!(
            output,
            "        self.base.{fetch_method}(&format!(\"{path}\"{path_args}){body_arg}){decode}"
        )
        .unwrap();
    } else {
        write_query_assembly(output, endpoint, &path, &path_args);
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
    } else if matches!(
        endpoint.response_kind,
        ResponseKind::Text { ty: Ty::Ref(_), .. }
    ) {
        ("text", ".and_then(parse_text)")
    } else {
        ("text", "")
    };
    (format!("{verb}_{kind}"), decode)
}

fn build_return_type(endpoint: &Endpoint) -> String {
    let base = endpoint
        .response_kind
        .ty()
        .map_or_else(|| "Vec<u8>".to_owned(), rust_type);
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

fn write_query_assembly(output: &mut String, endpoint: &Endpoint, path: &str, path_args: &str) {
    writeln!(output, "        let mut query = Vec::new();").unwrap();
    for param in &endpoint.query_params {
        let ident = sanitize_ident(&param.name);
        let name = &param.name;
        let (list, item) = match &param.ty {
            Ty::List(item) => (true, &**item),
            ty => (false, ty),
        };
        // `Index` displays its variant (`TxIndex`); the server parses its name (`tx_index`).
        let value = |v: &str| {
            if item.is_ref("Index") {
                format!("{v}.name()")
            } else {
                v.to_string()
            }
        };
        if list {
            let v = value("v");
            writeln!(
                output,
                "        for v in {ident} {{ query.push(query_pair(\"{name}\", {v})); }}"
            )
            .unwrap();
        } else if param.required {
            let v = value(&ident);
            writeln!(output, "        query.push(query_pair(\"{name}\", {v}));").unwrap();
        } else {
            let v = value("v");
            writeln!(
                output,
                "        if let Some(v) = {ident} {{ query.push(query_pair(\"{name}\", {v})); }}"
            )
            .unwrap();
        }
    }
    writeln!(output, "        let query_str = if query.is_empty() {{ String::new() }} else {{ format!(\"?{{}}\", query.join(\"&\")) }};").unwrap();
    writeln!(
        output,
        "        let path = format!(\"{}{{}}\"{}, query_str);",
        path, path_args
    )
    .unwrap();
}

fn endpoint_to_method_name(endpoint: &Endpoint) -> String {
    to_snake_case(&endpoint.operation_id)
}

fn build_method_params(endpoint: &Endpoint) -> String {
    let mut params = String::new();
    for param in &endpoint.path_params {
        let rust_type = rust_param_type(&param.ty);
        write!(params, ", {}: {}", sanitize_ident(&param.name), rust_type).unwrap();
    }
    for param in &endpoint.query_params {
        let rust_type = rust_param_type(&param.ty);
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

/// The path as a format template (`{}` per path parameter) and its format args, in path order:
/// `Index` by name, every other value percent-encoded as one segment.
fn build_path_template(endpoint: &Endpoint) -> (String, String) {
    let mut template = String::new();
    let mut args = String::new();
    let mut rest = endpoint.path.as_str();
    while let Some(open) = rest.find('{') {
        let close = open + rest[open..].find('}').expect("unclosed path parameter");
        let name = &rest[open + 1..close];
        let param = endpoint
            .path_params
            .iter()
            .find(|param| param.name == name)
            .expect("undeclared path parameter");
        let ident = sanitize_ident(name);
        if param.ty.is_ref("Index") {
            write!(args, ", {ident}.name()").unwrap();
        } else {
            write!(args, ", path_segment({ident})").unwrap();
        }
        template.push_str(&rest[..open]);
        template.push_str("{}");
        rest = &rest[close + 1..];
    }
    template.push_str(rest);
    (template, args)
}
