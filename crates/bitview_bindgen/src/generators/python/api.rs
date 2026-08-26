//! Python API method generation.

use std::fmt::Write;

use crate::{
    Endpoint, Parameter, escape_python_keyword,
    generators::{normalize_return_type, write_description},
    to_snake_case,
};

use super::client::generate_class_constants;
use super::types::js_type_to_python;

/// Generate the main client class
pub fn generate_main_client(output: &mut String, endpoints: &[Endpoint]) {
    writeln!(output, "class BitviewClient(BitviewClientBase):").unwrap();
    writeln!(
        output,
        "    \"\"\"Main Bitview client with series tree and API methods.\"\"\""
    )
    .unwrap();
    writeln!(output).unwrap();

    // Generate class-level constants
    generate_class_constants(output);

    writeln!(
        output,
        "    def __init__(self, base_url: str = 'http://localhost:3110', timeout: float = 30.0):"
    )
    .unwrap();
    writeln!(output, "        super().__init__(base_url, timeout)").unwrap();
    writeln!(output).unwrap();

    writeln!(output, "    @cached_property").unwrap();
    writeln!(output, "    def series(self) -> SeriesTree:").unwrap();
    writeln!(output, "        return SeriesTree(self)").unwrap();
    writeln!(output).unwrap();

    // Generate series_endpoint() method for dynamic series access
    writeln!(
        output,
        "    def series_endpoint(self, series: str, index: Index) -> SeriesEndpoint[Any]:"
    )
    .unwrap();
    writeln!(
        output,
        "        \"\"\"Create a dynamic series endpoint builder for any series/index combination."
    )
    .unwrap();
    writeln!(output).unwrap();
    writeln!(
        output,
        "        Use this for programmatic access when the series name is determined at runtime."
    )
    .unwrap();
    writeln!(
        output,
        "        For type-safe access, use the `series` tree instead."
    )
    .unwrap();
    writeln!(output, "        \"\"\"").unwrap();
    writeln!(output, "        return SeriesEndpoint(self, series, index)").unwrap();
    writeln!(output).unwrap();

    // Generate helper methods
    writeln!(
        output,
        "    def index_to_date(self, index: Index, i: int) -> Union[date, datetime]:"
    )
    .unwrap();
    writeln!(
        output,
        "        \"\"\"Convert an index value to a date/datetime for date-based indexes.\"\"\""
    )
    .unwrap();
    writeln!(output, "        return _index_to_date(index, i)").unwrap();
    writeln!(output).unwrap();
    writeln!(
        output,
        "    def date_to_index(self, index: Index, d: Union[date, datetime]) -> int:"
    )
    .unwrap();
    writeln!(
        output,
        "        \"\"\"Convert a date/datetime to an index value for date-based indexes.\"\"\""
    )
    .unwrap();
    writeln!(output, "        return _date_to_index(index, d)").unwrap();
    writeln!(output).unwrap();

    output.push_str(r#"    @staticmethod
    def address_payload_hash_prefix(payload: Union[bytes, bytearray, memoryview], nibbles: int) -> str:
        """Compute the RapidHash v3 hash-prefix for raw address payload bytes."""
        return address_payload_hash_prefix(payload, nibbles)

    def get_address_payload_hash_prefix_matches(
        self,
        addr_type: OutputType,
        payload: Union[bytes, bytearray, memoryview],
        nibbles: int,
    ) -> AddrHashPrefixMatches:
        """Fetch address hash-prefix matches from raw payload bytes matching addr_type length."""
        _validate_address_payload_for_type(addr_type, payload)
        return self.get_address_hash_prefix_matches(addr_type, address_payload_hash_prefix(payload, nibbles))

"#);
    // Generate API methods
    generate_api_methods(output, endpoints);
}

/// Generate API methods from OpenAPI endpoints
pub fn generate_api_methods(output: &mut String, endpoints: &[Endpoint]) {
    for endpoint in endpoints {
        if !endpoint.should_generate() {
            continue;
        }

        let method_name = endpoint_to_method_name(endpoint);
        let base_return_type = if endpoint.returns_binary() {
            "bytes".to_string()
        } else {
            normalize_return_type(
                &endpoint
                    .schema_name()
                    .map(js_type_to_python)
                    .unwrap_or_else(|| "str".to_string()),
            )
        };

        let return_type = if endpoint.supports_csv {
            format!("Union[{}, str]", base_return_type)
        } else {
            base_return_type
        };

        // Build method signature
        let params = build_method_params(endpoint);
        writeln!(
            output,
            "    def {}(self{}) -> {}:",
            method_name, params, return_type
        )
        .unwrap();

        // Docstring
        match (&endpoint.summary, &endpoint.description) {
            (Some(summary), Some(desc)) if summary != desc => {
                writeln!(output, "        \"\"\"{}.", summary.trim_end_matches('.')).unwrap();
                writeln!(output).unwrap();
                write_description(output, desc, "        ", "");
            }
            (Some(summary), _) => {
                writeln!(output, "        \"\"\"{}", summary).unwrap();
            }
            (None, Some(desc)) => {
                // First line includes opening quotes
                let mut lines = desc.lines();
                if let Some(first) = lines.next() {
                    writeln!(output, "        \"\"\"{}", first).unwrap();
                }
                for line in lines {
                    if line.is_empty() {
                        writeln!(output).unwrap();
                    } else {
                        writeln!(output, "        {}", line).unwrap();
                    }
                }
            }
            (None, None) => {
                write!(output, "        \"\"\"").unwrap();
            }
        }
        writeln!(output).unwrap();
        writeln!(
            output,
            "        Endpoint: `{} {}`\"\"\"",
            endpoint.method.to_uppercase(),
            endpoint.path
        )
        .unwrap();

        // Build path
        let path = build_path_template(&endpoint.path, &endpoint.path_params);

        let is_post = endpoint.method == "POST";
        let fetch_method = match (is_post, &endpoint.response_kind) {
            (false, _) if endpoint.returns_binary() => "get",
            (false, _) if endpoint.returns_json() => "get_json",
            (false, _) => "get_text",
            (true, _) if endpoint.returns_binary() => "post",
            (true, _) if endpoint.returns_json() => "post_json",
            (true, _) => "post_text",
        };

        let body_arg = if is_post && endpoint.request_body.is_some() {
            ", body"
        } else {
            ""
        };

        let (wrap_prefix, wrap_suffix) = if endpoint.response_kind.text_is_numeric() {
            ("int(", ")")
        } else {
            ("", "")
        };

        if endpoint.query_params.is_empty() {
            if endpoint.path_params.is_empty() {
                writeln!(
                    output,
                    "        return {}self.{}('{}'{}){}",
                    wrap_prefix, fetch_method, path, body_arg, wrap_suffix
                )
                .unwrap();
            } else {
                writeln!(
                    output,
                    "        return {}self.{}(f'{}'{}){}",
                    wrap_prefix, fetch_method, path, body_arg, wrap_suffix
                )
                .unwrap();
            }
        } else {
            writeln!(output, "        params = []").unwrap();
            for param in &endpoint.query_params {
                // Use safe name for Python variable, original name for API query parameter
                let safe_name = escape_python_keyword(&param.name);
                let is_array = param.param_type.ends_with("[]");
                if is_array {
                    writeln!(
                        output,
                        "        for _v in {}: params.append(f'{}={{_v}}')",
                        safe_name, param.name
                    )
                    .unwrap();
                } else if param.required {
                    writeln!(
                        output,
                        "        params.append(f'{}={{{}}}')",
                        param.name, safe_name
                    )
                    .unwrap();
                } else {
                    writeln!(
                        output,
                        "        if {} is not None: params.append(f'{}={{{}}}')",
                        safe_name, param.name, safe_name
                    )
                    .unwrap();
                }
            }
            writeln!(output, "        query = '&'.join(params)").unwrap();
            writeln!(
                output,
                "        path = f'{}{{\"?\" + query if query else \"\"}}'",
                path
            )
            .unwrap();

            if endpoint.supports_csv {
                writeln!(output, "        if format == 'csv':").unwrap();
                writeln!(output, "            return self.get_text(path)").unwrap();
                writeln!(
                    output,
                    "        return {}self.{}(path{}){}",
                    wrap_prefix, fetch_method, body_arg, wrap_suffix
                )
                .unwrap();
            } else {
                writeln!(
                    output,
                    "        return {}self.{}(path{}){}",
                    wrap_prefix, fetch_method, body_arg, wrap_suffix
                )
                .unwrap();
            }
        }

        writeln!(output).unwrap();
    }
}

fn endpoint_to_method_name(endpoint: &Endpoint) -> String {
    to_snake_case(&endpoint.operation_name())
}

fn build_method_params(endpoint: &Endpoint) -> String {
    let mut params = Vec::new();
    // Path params are always required
    for param in &endpoint.path_params {
        let safe_name = escape_python_keyword(&param.name);
        let py_type = js_type_to_python(&param.param_type);
        params.push(format!(", {}: {}", safe_name, py_type));
    }
    // Required query params must come before optional ones (Python syntax requirement)
    for param in &endpoint.query_params {
        if param.required {
            let safe_name = escape_python_keyword(&param.name);
            let py_type = js_type_to_python(&param.param_type);
            params.push(format!(", {}: {}", safe_name, py_type));
        }
    }
    for param in &endpoint.query_params {
        if !param.required {
            let safe_name = escape_python_keyword(&param.name);
            let py_type = js_type_to_python(&param.param_type);
            params.push(format!(", {}: Optional[{}] = None", safe_name, py_type));
        }
    }
    if let Some(body) = &endpoint.request_body {
        let py_type = js_type_to_python(&body.body_type);
        if body.required {
            params.push(format!(", body: {}", py_type));
        } else {
            params.push(format!(", body: Optional[{}] = None", py_type));
        }
    }
    params.join("")
}

fn build_path_template(path: &str, path_params: &[Parameter]) -> String {
    let mut result = path.to_string();
    for param in path_params {
        let placeholder = format!("{{{}}}", param.name);
        // Use escaped name for Python variable interpolation in f-string
        let safe_name = escape_python_keyword(&param.name);
        let interpolation = format!("{{{}}}", safe_name);
        result = result.replace(&placeholder, &interpolation);
    }
    result
}
