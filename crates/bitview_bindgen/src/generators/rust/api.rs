use std::fmt::Write;

use crate::{
    Endpoint, VERSION,
    generators::{
        operation::{Decode, Operation, PathPart},
        write_description,
    },
    to_snake_case,
};

use super::types::{rust_param_type, rust_type};

pub(crate) fn generate_main_client(output: &mut String, operations: &[Operation<'_>]) {
    writeln!(
        output,
        r#"/// A method per API endpoint, and the typed series tree.
pub struct BitviewClient {{
    base: Arc<BitviewClientBase>,
    series: OnceLock<Box<SeriesTree>>,
}}

impl BitviewClient {{
    pub const VERSION: &'static str = "v{VERSION}";

    pub fn new(base_url: impl Into<String>) -> Self {{
        let base = Arc::new(BitviewClientBase::new(base_url));
        Self {{ base, series: OnceLock::new() }}
    }}

    pub fn with_options(options: BitviewClientOptions) -> Self {{
        let base = Arc::new(BitviewClientBase::with_options(options));
        Self {{ base, series: OnceLock::new() }}
    }}

    /// The typed series tree, built on first use.
    pub fn series(&self) -> &SeriesTree {{
        self.series
            .get_or_init(|| Box::new(SeriesTree::build(self.base.clone(), String::new())))
    }}

    /// A series by a name known only at runtime; [`series`](Self::series) has the typed paths.
    pub fn series_endpoint(&self, series: impl Into<SeriesName>, index: Index) -> SeriesEndpoint<serde_json::Value> {{
        SeriesEndpoint::new(
            self.base.clone(),
            Arc::from(series.into().as_str()),
            index,
        )
    }}

    /// Like [`series_endpoint`](Self::series_endpoint) with date selectors; `Err` if the index is
    /// not date-based.
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
"#
    )
    .unwrap();

    output.push_str(r#"    /// Fetch address hash-prefix matches from raw payload bytes matching `address_type` length.
    pub fn get_address_payload_hash_prefix_matches(&self, address_type: AddressOutputType, payload: &[u8], nibbles: usize) -> Result<AddressHashPrefixMatches> {
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

    generate_api_methods(output, operations);

    writeln!(output, "}}").unwrap();
}

fn generate_api_methods(output: &mut String, operations: &[Operation<'_>]) {
    for operation in operations {
        generate_method(output, operation);
    }
}

fn generate_method(output: &mut String, operation: &Operation<'_>) {
    let endpoint = operation.endpoint;
    let post = operation.post;
    let (fetch_method, decode) = fetch(if post { "post" } else { "get" }, operation.response.decode);

    write_method_doc(output, endpoint);

    let mut params = build_method_params(operation);
    // A body goes as its declared media type; a JSON one is passed as its JSON text.
    let body_arg = match (post, operation.post_body()) {
        (false, _) => String::new(),
        (true, Some(body)) => {
            params.push_str(", body: &str");
            format!(", body, \"{}\"", body.declaration.content_type)
        }
        (true, None) => ", \"\", \"text/plain\"".to_string(),
    };
    writeln!(
        output,
        "    pub fn {}(&self{params}) -> Result<{}> {{",
        to_snake_case(&endpoint.operation_id),
        build_return_type(operation)
    )
    .unwrap();

    let (path, path_args) = build_path_template(operation);
    if operation.query_params.is_empty() {
        writeln!(
            output,
            "        self.base.{fetch_method}(&format!(\"{path}\"{path_args}){body_arg}){decode}"
        )
        .unwrap();
    } else {
        write_query_assembly(output, operation, &path, &path_args);
        if operation.csv && !post {
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
fn fetch(verb: &str, decode: Decode) -> (String, &'static str) {
    let (kind, decode) = match decode {
        Decode::Binary => ("bytes", ""),
        Decode::Json | Decode::NumericText => ("json", ""),
        Decode::NamedText => ("text", ".and_then(parse_text)"),
        Decode::Text => ("text", ""),
    };
    (format!("{verb}_{kind}"), decode)
}

fn build_return_type(operation: &Operation<'_>) -> String {
    let base = operation.response.ty
        .map_or_else(|| "Vec<u8>".to_owned(), rust_type);
    if operation.csv {
        format!("FormatResponse<{}>", base)
    } else {
        base
    }
}

fn write_method_doc(output: &mut String, endpoint: &Endpoint) {
    let description = endpoint
        .description
        .as_ref()
        .filter(|desc| endpoint.summary.as_ref() != Some(desc));
    for text in [endpoint.summary.as_ref(), description]
        .into_iter()
        .flatten()
    {
        write_description(output, text, "    /// ", "    ///");
        writeln!(output, "    ///").unwrap();
    }
    writeln!(
        output,
        "    /// Endpoint: `{} {}`",
        endpoint.method, endpoint.path
    )
    .unwrap();
}

fn write_query_assembly(output: &mut String, operation: &Operation<'_>, path: &str, path_args: &str) {
    writeln!(output, "        let mut query = Vec::new();").unwrap();
    for argument in &operation.query_params {
        let param = argument.parameter;
        let ident = &param.arg;
        let name = &param.name;
        // `Index` displays its variant (`TxIndex`); the server parses its name (`tx_index`).
        let value = |v: &str| {
            if argument.index_name {
                format!("{v}.name()")
            } else {
                v.to_string()
            }
        };
        if argument.rule.repeated() {
            let v = value("v");
            let values = if argument.rule.required() {
                ident.to_string()
            } else {
                format!("{ident}.into_iter().flatten()")
            };
            writeln!(
                output,
                "        for v in {values} {{ query.push(query_pair(\"{name}\", {v})); }}"
            )
            .unwrap();
        } else if argument.rule.required() {
            let v = value(ident);
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

fn build_method_params(operation: &Operation<'_>) -> String {
    let mut params = String::new();
    for param in &operation.path_params {
        let rust_type = rust_param_type(&param.ty);
        write!(params, ", {}: {}", param.arg, rust_type).unwrap();
    }
    for argument in &operation.query_params {
        let param = argument.parameter;
        let rust_type = rust_param_type(&param.ty);
        let name = &param.arg;
        if argument.rule.required() {
            write!(params, ", {}: {}", name, rust_type).unwrap();
        } else {
            write!(params, ", {}: Option<{}>", name, rust_type).unwrap();
        }
    }
    params
}

/// The path as a format template (`{}` per path parameter) and its format args, in path order:
/// `Index` by name, every other value percent-encoded as one segment.
fn build_path_template(operation: &Operation<'_>) -> (String, String) {
    let mut template = String::new();
    let mut args = String::new();
    for part in &operation.path {
        match part {
            PathPart::Literal(text) => template.push_str(text),
            PathPart::Parameter { parameter, index_name } => {
                let ident = &parameter.arg;
                if *index_name {
                    write!(args, ", {ident}.name()").unwrap();
                } else {
                    write!(args, ", path_segment({ident})").unwrap();
                }
                template.push_str("{}");
            }
        }
    }
    (template, args)
}
