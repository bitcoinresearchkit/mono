use std::fmt::Write;

use crate::{
    Endpoint, Parameter, Ty, TypeSchemas,
    generators::{
        javascript::{keep::keep_expr, types::js_type},
        jsdoc_continued, normalize_return_type, write_description,
    },
    to_camel_case,
};

// GET request options and their JSDoc types; POST accepts only signal.
const REQUEST_OPTIONS: &[(&str, &str)] = &[
    ("signal", "AbortSignal"),
    ("onValue", "(value: {return_type}) => void"),
    ("cache", "boolean"),
    ("memCache", "boolean"),
];

pub fn generate_api_methods(output: &mut String, endpoints: &[Endpoint], schemas: &TypeSchemas) {
    for endpoint in endpoints
        .iter()
        .filter(|endpoint| endpoint.should_generate())
    {
        generate_method(output, endpoint, schemas);
    }
}

fn generate_method(output: &mut String, endpoint: &Endpoint, schemas: &TypeSchemas) {
    let post = endpoint.method == "POST";
    let body = endpoint.request_body.as_ref().filter(|_| post);
    let return_type = build_return_type(endpoint);

    let name = to_camel_case(&endpoint.operation_id);
    let request_fields = if post {
        &REQUEST_OPTIONS[..1]
    } else {
        REQUEST_OPTIONS
    };
    for param in &endpoint.query_params {
        assert!(
            !REQUEST_OPTIONS.iter().any(|(name, _)| *name == param.arg)
                && !["params", "query", "path", "body", "_v"].contains(&param.arg.as_str())
                && !endpoint
                    .path_params
                    .iter()
                    .any(|path| path.arg == param.arg),
            "{name}: query parameter {} collides with a request option or generated local",
            param.arg
        );
    }
    write_method_doc(output, endpoint);
    for param in &endpoint.path_params {
        let desc = format_param_desc(param.description.as_deref());
        let ty = js_type(&param.ty);
        writeln!(output, "   * @param {{{}}} {}{}", ty, param.arg, desc).unwrap();
    }
    if let Some(body) = body {
        let optional = if body.required { "" } else { "=" };
        let ty = js_type(&body.ty);
        writeln!(output, "   * @param {{{ty}{optional}}} body").unwrap();
    }
    let required_options = endpoint.query_params.iter().any(|param| param.required);
    let options = if required_options {
        "options"
    } else {
        "[options]"
    };
    writeln!(output, "   * @param {{object}} {options}").unwrap();
    for param in &endpoint.query_params {
        let name = format!("options.{}", param.arg);
        let name = if param.required {
            name
        } else {
            format!("[{name}]")
        };
        let ty = js_type(&param.ty);
        let desc = format_param_desc(param.description.as_deref());
        writeln!(output, "   * @param {{{ty}}} {name}{desc}").unwrap();
    }
    for (field, ty) in request_fields {
        let ty = ty.replace("{return_type}", &return_type);
        writeln!(output, "   * @param {{{ty}}} [options.{field}]").unwrap();
    }
    writeln!(output, "   * @returns {{Promise<{}>}}", return_type).unwrap();
    writeln!(output, "   */").unwrap();

    let mut params = build_method_params(endpoint);
    if body.is_some() {
        if !params.is_empty() {
            params.push_str(", ");
        }
        params.push_str("body");
    }
    let request_args = request_options(request_fields, None);
    let fields = endpoint
        .query_params
        .iter()
        .map(|param| param.arg.as_str())
        .chain(request_fields.iter().map(|(name, _)| *name))
        .collect::<Vec<_>>()
        .join(", ");
    let options = format!("{{ {fields} }} = {{}}");
    if required_options {
        writeln!(
            output,
            "  // @ts-expect-error Missing options are checked at runtime."
        )
        .unwrap();
    }
    if params.is_empty() {
        writeln!(output, "  async {name}({options}) {{").unwrap();
    } else {
        writeln!(output, "  async {name}({params}, {options}) {{").unwrap();
    }
    for param in endpoint.query_params.iter().filter(|param| param.required) {
        writeln!(
            output,
            "    if ({} == null) throw new TypeError('{name}: options.{} is required');",
            param.arg, param.arg
        )
        .unwrap();
    }

    let (verb, args) = if post {
        // A body goes as its declared media type, a JSON one serialized.
        let args = match body {
            Some(body) => {
                let value = if body.is_json() {
                    "JSON.stringify(body)"
                } else {
                    "body"
                };
                format!(
                    "path, {value}, {{ {}, contentType: '{}' }}",
                    request_fields
                        .iter()
                        .map(|(name, _)| *name)
                        .collect::<Vec<_>>()
                        .join(", "),
                    body.content_type
                )
            }
            None => format!("path, '', {request_args}"),
        };
        ("post", args)
    } else {
        ("get", format!("path, {request_args}"))
    };
    // Series data gets the same helpers however it is fetched.
    let response = endpoint.response_kind.ty();
    let fetch_call = if endpoint.returns_binary() {
        format!("this.{verb}Bytes({args})")
    } else if response.is_some_and(|ty| ty.is_ref("SeriesData")) {
        format!(
            "_wrapSeriesData(await this._getCached(path, 'json', _readJson(_SERIES_KEEP), {}))",
            request_options(
                request_fields,
                Some("onValue && ((v) => onValue(_wrapSeriesData(v)))")
            )
        )
    } else if matches!(response, Some(Ty::List(item)) if item.is_ref("SeriesData")) {
        format!(
            "(await this._getCached(path, 'json', _readJson(_SERIES_KEEP), {})).map(_wrapSeriesData)",
            request_options(
                request_fields,
                Some("onValue && ((v) => onValue(v.map(_wrapSeriesData)))")
            )
        )
    } else if endpoint.returns_json() {
        // Map keys are data: they stay as sent while field names are camelCased. A GET reads
        // through the cache as `getJson` does, typed by the method (its `Frozen` return).
        let keep = response.and_then(|ty| keep_expr(ty, schemas));
        match (post, keep) {
            (false, keep) => format!(
                "this._getCached(path, 'json', _readJson({}), {request_args})",
                keep.as_deref().unwrap_or("undefined")
            ),
            (true, Some(keep)) => format!("this.postJson({args}, {keep})"),
            (true, None) => format!("this.postJson({args})"),
        }
    } else if endpoint.response_kind.text_is_numeric() && post {
        format!("Number(await this.postText({args}))")
    } else if endpoint.response_kind.text_is_numeric() {
        format!(
            "Number(await this.getText(path, {}))",
            request_options(
                request_fields,
                Some("onValue ? (v) => onValue(Number(v)) : undefined")
            )
        )
    } else {
        format!("this.{verb}Text({args})")
    };

    let path = build_path_template(&endpoint.path, &endpoint.path_params);
    write_path_assignment(output, endpoint, &path);

    if endpoint.supports_csv {
        writeln!(
            output,
            "    if (format === 'csv') return this.getText(path, {request_args});"
        )
        .unwrap();
    }
    writeln!(output, "    return {};", fetch_call).unwrap();
    writeln!(output, "  }}\n").unwrap();
}

fn request_options(fields: &[(&str, &str)], on_value: Option<&str>) -> String {
    let fields = fields
        .iter()
        .map(|(name, _)| {
            if *name == "onValue"
                && let Some(value) = on_value
            {
                return format!("{name}: {value}");
            }
            (*name).to_string()
        })
        .collect::<Vec<_>>()
        .join(", ");
    format!("{{ {fields} }}")
}

fn build_return_type(endpoint: &Endpoint) -> String {
    let base = endpoint.response_kind.ty().map_or_else(
        || "Uint8Array".to_owned(),
        |ty| normalize_return_type(&js_type(ty)),
    );
    // A JSON answer to a GET may be the memory cache's, shared and frozen (`_deepFreeze`); a
    // series, wrapped with its methods, types its own data so.
    let series = endpoint.response_kind.ty().is_some_and(|ty| {
        ty.is_ref("SeriesData") || matches!(ty, Ty::List(item) if item.is_ref("SeriesData"))
    });
    let base = if endpoint.method == "GET" && endpoint.returns_json() && base != "*" && !series {
        format!("Frozen<{base}>")
    } else {
        base
    };
    if endpoint.supports_csv {
        format!("{} | string", base)
    } else {
        base
    }
}

fn write_method_doc(output: &mut String, endpoint: &Endpoint) {
    writeln!(output, "  /**").unwrap();
    if let Some(summary) = &endpoint.summary {
        writeln!(output, "   * {}", summary).unwrap();
    }
    if let Some(desc) = &endpoint.description
        && endpoint.summary.as_ref() != Some(desc)
    {
        writeln!(output, "   *").unwrap();
        write_description(output, desc, "   * ", "   *");
    }
    writeln!(output, "   *").unwrap();
    writeln!(
        output,
        "   * Endpoint: `{} {}`",
        endpoint.method, endpoint.path
    )
    .unwrap();

    let has_body_param = endpoint.method == "POST" && endpoint.request_body.is_some();
    if !endpoint.path_params.is_empty() || !endpoint.query_params.is_empty() || has_body_param {
        writeln!(output, "   *").unwrap();
    }
}

fn write_path_assignment(output: &mut String, endpoint: &Endpoint, path: &str) {
    if endpoint.query_params.is_empty() {
        writeln!(output, "    const path = `{}`;", path).unwrap();
    } else {
        writeln!(output, "    const params = new URLSearchParams();").unwrap();
        for param in &endpoint.query_params {
            let ident = &param.arg;
            if matches!(param.ty, Ty::List(_)) {
                if param.required {
                    writeln!(
                        output,
                        "    for (const _v of {}) params.append('{}', String(_v));",
                        ident, param.name
                    )
                    .unwrap();
                } else {
                    writeln!(
                        output,
                        "    if ({} != null) for (const _v of {}) params.append('{}', String(_v));",
                        ident, ident, param.name
                    )
                    .unwrap();
                }
            } else if param.required {
                writeln!(
                    output,
                    "    params.set('{}', String({}));",
                    param.name, ident
                )
                .unwrap();
            } else {
                writeln!(
                    output,
                    "    if ({} != null) params.set('{}', String({}));",
                    ident, param.name, ident
                )
                .unwrap();
            }
        }
        writeln!(output, "    const query = params.toString();").unwrap();
        writeln!(
            output,
            "    const path = `{}${{query ? '?' + query : ''}}`;",
            path
        )
        .unwrap();
    }
}

fn build_method_params(endpoint: &Endpoint) -> String {
    let mut params = String::new();
    for (index, param) in endpoint.path_params.iter().enumerate() {
        if index != 0 {
            params.push_str(", ");
        }
        params.push_str(&param.arg);
    }
    params
}

/// The path as a template literal, each parameter percent-encoded as one segment.
fn build_path_template(path: &str, path_params: &[Parameter]) -> String {
    let mut result = path.to_string();
    for param in path_params {
        let placeholder = format!("{{{}}}", param.name);
        let interpolation = format!("${{encodeURIComponent({})}}", param.arg);
        result = result.replace(&placeholder, &interpolation);
    }
    result
}

/// Format param description with dash prefix, or empty string if no description.
fn format_param_desc(desc: Option<&str>) -> String {
    match desc {
        Some(d) if !d.is_empty() => format!(" - {}", jsdoc_continued(d, "   *")),
        _ => String::new(),
    }
}
