use std::fmt::Write;

use crate::{
    Endpoint, Parameter, Ty, escape_python_keyword,
    generators::{normalize_return_type, write_description},
    to_snake_case,
};

use super::client::generate_class_constants;
use super::types::python_type;

/// A parameter or return type: endpoints quote no reference.
fn api_type(ty: &Ty) -> String {
    python_type(ty, &|_| false)
}

pub(crate) fn generate_main_client(output: &mut String, endpoints: &[Endpoint]) {
    writeln!(output, "class BitviewClient(BitviewClientBase):").unwrap();
    writeln!(
        output,
        "    \"\"\"A method per API endpoint, and the typed series tree.\"\"\""
    )
    .unwrap();
    writeln!(output).unwrap();

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
    writeln!(output, "        return SeriesTree(self, '')").unwrap();
    writeln!(output).unwrap();

    writeln!(
        output,
        "    def series_endpoint(self, series: str, index: Index) -> Union[SeriesEndpoint[Any], DateSeriesEndpoint[Any]]:"
    )
    .unwrap();
    writeln!(
        output,
        "        \"\"\"A series by a name known only at runtime; `series` has the typed paths.\"\"\""
    )
    .unwrap();
    writeln!(output, "        return _endpoint(self, series, index)").unwrap();
    writeln!(output).unwrap();

    writeln!(
        output,
        "    def index_to_date(self, index: Index, i: int) -> Union[date, datetime]:"
    )
    .unwrap();
    writeln!(
        output,
        "        \"\"\"The start of bucket `i`: a UTC datetime for sub-daily indexes, a date otherwise.\"\"\""
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
        "        \"\"\"The floor index (latest index whose date is <= the given date).\"\"\""
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
        address_type: AddressOutputType,
        payload: Union[bytes, bytearray, memoryview],
        nibbles: int,
    ) -> AddressHashPrefixMatches:
        """Fetch address hash-prefix matches from raw payload bytes matching address_type length."""
        _validate_address_payload_for_type(address_type, payload)
        return self.get_address_hash_prefix_matches(address_type, address_payload_hash_prefix(payload, nibbles))

"#);
    generate_api_methods(output, endpoints);
}

fn generate_api_methods(output: &mut String, endpoints: &[Endpoint]) {
    for endpoint in endpoints {
        if !endpoint.should_generate() {
            continue;
        }

        let method_name = to_snake_case(&endpoint.operation_id);
        let response = endpoint.response_kind.ty();
        let base_return_type = response.map_or_else(
            || "bytes".to_owned(),
            |ty| normalize_return_type(&api_type(ty)),
        );

        let return_type = if endpoint.supports_csv {
            format!("Union[{}, str]", base_return_type)
        } else {
            base_return_type
        };

        let params = build_method_params(endpoint);
        writeln!(
            output,
            "    def {}(self{}) -> {}:",
            method_name, params, return_type
        )
        .unwrap();

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
            endpoint.method, endpoint.path
        )
        .unwrap();

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

        // Series data gets the same helpers however it is fetched.
        let (wrap_prefix, wrap_suffix) = if endpoint.response_kind.text_is_numeric() {
            ("int(", ")")
        } else if response.is_some_and(|ty| ty.is_ref("SeriesData")) {
            ("_series_data(", ")")
        } else if matches!(response, Some(Ty::List(item)) if item.is_ref("SeriesData")) {
            ("[_series_data(raw) for raw in ", "]")
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
            writeln!(output, "        params: List[Tuple[str, Any]] = []").unwrap();
            for param in &endpoint.query_params {
                // Use safe name for Python variable, original name for API query parameter
                let safe_name = escape_python_keyword(&param.arg);
                if matches!(param.ty, Ty::List(_)) {
                    writeln!(
                        output,
                        "        for _v in {}: params.append(('{}', _v))",
                        safe_name, param.name
                    )
                    .unwrap();
                } else if param.required {
                    writeln!(
                        output,
                        "        params.append(('{}', {}))",
                        param.name, safe_name
                    )
                    .unwrap();
                } else {
                    writeln!(
                        output,
                        "        if {} is not None: params.append(('{}', {}))",
                        safe_name, param.name, safe_name
                    )
                    .unwrap();
                }
            }
            writeln!(output, "        query = urlencode(params)").unwrap();
            writeln!(
                output,
                "        path = f'{}{{\"?\" + query if query else \"\"}}'",
                path
            )
            .unwrap();

            if endpoint.supports_csv {
                writeln!(output, "        if format == 'csv':").unwrap();
                writeln!(output, "            return self.get_text(path)").unwrap();
            }
            writeln!(
                output,
                "        return {}self.{}(path{}){}",
                wrap_prefix, fetch_method, body_arg, wrap_suffix
            )
            .unwrap();
        }

        writeln!(output).unwrap();
    }
}

fn build_method_params(endpoint: &Endpoint) -> String {
    let mut params = String::new();
    // Path params are always required
    for param in &endpoint.path_params {
        let safe_name = escape_python_keyword(&param.arg);
        let py_type = api_type(&param.ty);
        write!(params, ", {}: {}", safe_name, py_type).unwrap();
    }
    // Required query params must come before optional ones (Python syntax requirement)
    for param in &endpoint.query_params {
        if param.required {
            let safe_name = escape_python_keyword(&param.arg);
            let py_type = api_type(&param.ty);
            write!(params, ", {}: {}", safe_name, py_type).unwrap();
        }
    }
    for param in &endpoint.query_params {
        if !param.required {
            let safe_name = escape_python_keyword(&param.arg);
            let py_type = api_type(&param.ty);
            write!(params, ", {}: Optional[{}] = None", safe_name, py_type).unwrap();
        }
    }
    if let Some(body) = &endpoint.request_body {
        let py_type = api_type(&body.ty);
        if body.required {
            write!(params, ", body: {}", py_type).unwrap();
        } else {
            write!(params, ", body: Optional[{}] = None", py_type).unwrap();
        }
    }
    params
}

/// The path as an f-string body, each parameter percent-encoded as one segment.
fn build_path_template(path: &str, path_params: &[Parameter]) -> String {
    let mut result = path.to_string();
    for param in path_params {
        let placeholder = format!("{{{}}}", param.name);
        let safe_name = escape_python_keyword(&param.arg);
        let interpolation = format!("{{_segment({})}}", safe_name);
        result = result.replace(&placeholder, &interpolation);
    }
    result
}
