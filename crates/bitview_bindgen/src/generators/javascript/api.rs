use std::fmt::Write;

use crate::{
    TypeSchemas,
    generators::{
        javascript::{keep::keep_expr, types::js_type},
        jsdoc_continued, normalize_return_type,
        operation::{Decode, Operation, PathPart, Series},
        write_description,
    },
    to_camel_case,
};

pub fn generate_api_methods(output: &mut String, operations: &[Operation<'_>], schemas: &TypeSchemas) {
    for operation in operations {
        generate_method(output, operation, schemas);
    }
}

fn generate_method(output: &mut String, operation: &Operation<'_>, schemas: &TypeSchemas) {
    let endpoint = operation.endpoint;
    let post = operation.post;
    let body = operation.post_body();
    let return_type = build_return_type(operation);

    write_method_doc(output, operation);
    for param in &operation.path_params {
        let desc = format_param_desc(param.description.as_deref());
        let ty = js_type(&param.ty);
        writeln!(output, "   * @param {{{}}} {}{}", ty, param.arg, desc).unwrap();
    }
    for argument in &operation.query_params {
        let param = argument.parameter;
        let optional = if argument.rule.required() { "" } else { "=" };
        let desc = format_param_desc(param.description.as_deref());
        let ty = js_type(&param.ty);
        let ident = param.arg.clone();
        let name_decl = if argument.rule.required() {
            ident
        } else {
            format!("[{}]", ident)
        };
        writeln!(
            output,
            "   * @param {{{}{}}} {}{}",
            ty, optional, name_decl, desc
        )
        .unwrap();
    }
    if let Some(body) = body {
        let optional = if body.declaration.required { "" } else { "=" };
        let ty = js_type(&body.declaration.ty);
        writeln!(output, "   * @param {{{ty}{optional}}} body").unwrap();
    }
    if post {
        writeln!(
            output,
            "   * @param {{{{ signal?: AbortSignal }}}} [options]"
        )
        .unwrap();
    } else {
        writeln!(
            output,
            "   * @param {{{{ signal?: AbortSignal, onValue?: (value: {}) => void, cache?: boolean, memCache?: boolean }}}} [options]",
            return_type
        )
        .unwrap();
    }
    writeln!(output, "   * @returns {{Promise<{}>}}", return_type).unwrap();
    writeln!(output, "   */").unwrap();

    let mut params = build_method_params(operation);
    if body.is_some() {
        if !params.is_empty() {
            params.push_str(", ");
        }
        params.push_str("body");
    }
    let options = if post {
        "{ signal }"
    } else {
        "{ signal, onValue, cache, memCache }"
    };
    let name = to_camel_case(&endpoint.operation_id);
    if params.is_empty() {
        writeln!(output, "  async {name}({options} = {{}}) {{").unwrap();
    } else {
        writeln!(output, "  async {name}({params}, {options} = {{}}) {{").unwrap();
    }

    let (verb, args) = if post {
        // A body goes as its declared media type, a JSON one serialized.
        let args = match body {
            Some(body) => {
                let value = if body.json {
                    "JSON.stringify(body)"
                } else {
                    "body"
                };
                format!(
                    "path, {value}, {{ signal, contentType: '{}' }}",
                    body.declaration.content_type
                )
            }
            None => format!("path, '', {options}"),
        };
        ("post", args)
    } else {
        ("get", format!("path, {options}"))
    };
    // Series data gets the same helpers however it is fetched.
    let response = operation.response.ty;
    let fetch_call = if operation.response.decode == Decode::Binary {
        format!("this.{verb}Bytes({args})")
    } else if operation.response.series == Series::One {
        "_wrapSeriesData(await this._getCached(path, 'json', _readJson(_SERIES_KEEP), { signal, cache, memCache, onValue: onValue && ((v) => onValue(_wrapSeriesData(v))) }))".to_string()
    } else if operation.response.series == Series::Many {
        "(await this._getCached(path, 'json', _readJson(_SERIES_KEEP), { signal, cache, memCache, onValue: onValue && ((v) => onValue(v.map(_wrapSeriesData))) })).map(_wrapSeriesData)".to_string()
    } else if operation.response.decode == Decode::Json {
        // Map keys are data: they stay as sent while field names are camelCased. A GET reads
        // through the cache as `getJson` does, typed by the method (its `Frozen` return).
        let keep = response.and_then(|ty| keep_expr(ty, schemas));
        match (post, keep) {
            (false, keep) => format!(
                "this._getCached(path, 'json', _readJson({}), {options})",
                keep.as_deref().unwrap_or("undefined")
            ),
            (true, Some(keep)) => format!("this.postJson({args}, {keep})"),
            (true, None) => format!("this.postJson({args})"),
        }
    } else if operation.response.decode == Decode::NumericText && post {
        format!("Number(await this.postText({args}))")
    } else if operation.response.decode == Decode::NumericText {
        "Number(await this.getText(path, { signal, cache, memCache, onValue: onValue ? (v) => onValue(Number(v)) : undefined }))".to_string()
    } else {
        format!("this.{verb}Text({args})")
    };

    let path = build_path_template(operation);
    write_path_assignment(output, operation, &path);

    if operation.csv {
        writeln!(
            output,
            "    if (format === 'csv') return this.getText(path, {{ signal, onValue, cache, memCache }});"
        )
        .unwrap();
    }
    writeln!(output, "    return {};", fetch_call).unwrap();
    writeln!(output, "  }}\n").unwrap();
}

fn build_return_type(operation: &Operation<'_>) -> String {
    let base = operation.response.ty.map_or_else(
        || "Uint8Array".to_owned(),
        |ty| normalize_return_type(&js_type(ty)),
    );
    // A JSON answer to a GET may be the memory cache's, shared and frozen (`_deepFreeze`); a
    // series, wrapped with its methods, types its own data so.
    let series = operation.response.series != Series::None;
    let base = if !operation.post
        && operation.response.decode == Decode::Json
        && base != "*"
        && !series
    {
        format!("Frozen<{base}>")
    } else {
        base
    };
    if operation.csv {
        format!("{} | string", base)
    } else {
        base
    }
}

fn write_method_doc(output: &mut String, operation: &Operation<'_>) {
    let endpoint = operation.endpoint;
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

    let has_body_param = operation.post_body().is_some();
    if !operation.path_params.is_empty() || !operation.query_params.is_empty() || has_body_param {
        writeln!(output, "   *").unwrap();
    }
}

fn write_path_assignment(output: &mut String, operation: &Operation<'_>, path: &str) {
    if operation.query_params.is_empty() {
        writeln!(output, "    const path = `{}`;", path).unwrap();
    } else {
        writeln!(output, "    const params = new URLSearchParams();").unwrap();
        for argument in &operation.query_params {
            let param = argument.parameter;
            let ident = &param.arg;
            if argument.rule.repeated() {
                if argument.rule.required() {
                    writeln!(
                        output,
                        "    for (const _v of {}) params.append('{}', String(_v));",
                        ident, param.name
                    )
                    .unwrap();
                } else {
                    writeln!(
                        output,
                        "    if ({}) for (const _v of {}) params.append('{}', String(_v));",
                        ident, ident, param.name
                    )
                    .unwrap();
                }
            } else if argument.rule.required() {
                writeln!(
                    output,
                    "    params.set('{}', String({}));",
                    param.name, ident
                )
                .unwrap();
            } else {
                writeln!(
                    output,
                    "    if ({} !== undefined) params.set('{}', String({}));",
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

fn build_method_params(operation: &Operation<'_>) -> String {
    let mut params = String::new();
    let query = operation.query_params.iter().map(|argument| argument.parameter);
    for (index, parameter) in operation.path_params.iter().copied().chain(query).enumerate() {
        if index != 0 {
            params.push_str(", ");
        }
        params.push_str(&parameter.arg);
    }
    params
}

/// The path as a template literal, each parameter percent-encoded as one segment.
fn build_path_template(operation: &Operation<'_>) -> String {
    let mut result = String::new();
    for part in &operation.path {
        match part {
            PathPart::Literal(text) => result.push_str(text),
            PathPart::Parameter { parameter, .. } => {
                write!(result, "${{encodeURIComponent({})}}", parameter.arg).unwrap();
            }
        }
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
