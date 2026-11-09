use std::fmt::Write;

use crate::{
    Ty, escape_python_keyword,
    generators::{
        normalize_return_type,
        operation::{Decode, Operation, PathPart, Series},
        write_description,
    },
    to_snake_case,
};

use super::client::generate_class_constants;
use super::types::python_type;

/// A parameter or return type: endpoints quote no reference.
fn api_type(ty: &Ty) -> String {
    python_type(ty, &|_| false)
}

pub(crate) fn generate_main_client(output: &mut String, operations: &[Operation<'_>]) {
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
    generate_api_methods(output, operations);
}

fn generate_api_methods(output: &mut String, operations: &[Operation<'_>]) {
    for operation in operations {
        let endpoint = operation.endpoint;

        let method_name = to_snake_case(&endpoint.operation_id);
        let response = operation.response.ty;
        let base_return_type = response.map_or_else(
            || "bytes".to_owned(),
            |ty| normalize_return_type(&api_type(ty)),
        );

        let return_type = if operation.csv {
            format!("Union[{}, str]", base_return_type)
        } else {
            base_return_type
        };

        let params = build_method_params(operation);
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

        let path = build_path_template(operation);

        let fetch_method = match (operation.post, operation.response.decode) {
            (false, Decode::Binary) => "get",
            (false, Decode::Json) => "get_json",
            (false, _) => "get_text",
            (true, Decode::Binary) => "post",
            (true, Decode::Json) => "post_json",
            (true, _) => "post_text",
        };

        // A body goes as its declared media type, a JSON one serialized.
        let body_arg = match operation.post_body() {
            Some(body) if body.json => {
                format!(", json.dumps(body), '{}'", body.declaration.content_type)
            }
            Some(body) => format!(", body, '{}'", body.declaration.content_type),
            None => String::new(),
        };

        // Series data gets the same helpers however it is fetched.
        let (wrap_prefix, wrap_suffix) = if operation.response.decode == Decode::NumericText {
            ("int(", ")")
        } else if operation.response.series == Series::One {
            ("_series_data(", ")")
        } else if operation.response.series == Series::Many {
            ("[_series_data(raw) for raw in ", "]")
        } else {
            ("", "")
        };

        if operation.query_params.is_empty() {
            if operation.path_params.is_empty() {
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
            for argument in &operation.query_params {
                let param = argument.parameter;
                // Use safe name for Python variable, original name for API query parameter
                let safe_name = escape_python_keyword(&param.arg);
                if argument.rule.repeated() {
                    let values = if argument.rule.required() {
                        safe_name.to_string()
                    } else {
                        format!("({safe_name} or [])")
                    };
                    writeln!(
                        output,
                        "        for _v in {values}: params.append(('{}', _v))",
                        param.name
                    )
                    .unwrap();
                } else if argument.rule.required() {
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

            if operation.csv {
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

fn build_method_params(operation: &Operation<'_>) -> String {
    let mut params = String::new();
    // Path params are always required
    for param in &operation.path_params {
        let safe_name = escape_python_keyword(&param.arg);
        let py_type = api_type(&param.ty);
        write!(params, ", {}: {}", safe_name, py_type).unwrap();
    }
    // Required query params must come before optional ones (Python syntax requirement)
    for argument in &operation.query_params {
        let param = argument.parameter;
        if argument.rule.required() {
            let safe_name = escape_python_keyword(&param.arg);
            let py_type = api_type(&param.ty);
            write!(params, ", {}: {}", safe_name, py_type).unwrap();
        }
    }
    for argument in &operation.query_params {
        let param = argument.parameter;
        if !argument.rule.required() {
            let safe_name = escape_python_keyword(&param.arg);
            let py_type = api_type(&param.ty);
            write!(params, ", {}: Optional[{}] = None", safe_name, py_type).unwrap();
        }
    }
    if let Some(body) = &operation.body {
        let py_type = api_type(&body.declaration.ty);
        if body.declaration.required {
            write!(params, ", body: {}", py_type).unwrap();
        } else {
            write!(params, ", body: Optional[{}] = None", py_type).unwrap();
        }
    }
    params
}

/// The path as an f-string body, each parameter percent-encoded as one segment.
fn build_path_template(operation: &Operation<'_>) -> String {
    let mut result = String::new();
    for part in &operation.path {
        match part {
            PathPart::Literal(text) => result.push_str(text),
            PathPart::Parameter { parameter, .. } => {
                let safe_name = escape_python_keyword(&parameter.arg);
                write!(result, "{{_segment({})}}", safe_name).unwrap();
            }
        }
    }
    result
}
