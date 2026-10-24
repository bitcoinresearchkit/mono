use std::fmt::Write;

use crate::{
    Endpoint, Parameter, Ty, TypeSchemas,
    generators::{
        javascript::{keep::keep_expr, types::js_type},
        normalize_return_type, write_description,
    },
    to_camel_case,
};

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

    write_method_doc(output, endpoint);
    for param in &endpoint.path_params {
        let desc = format_param_desc(param.description.as_deref());
        let ty = js_type(&param.ty);
        writeln!(output, "   * @param {{{}}} {}{}", ty, param.name, desc).unwrap();
    }
    for param in &endpoint.query_params {
        let optional = if param.required { "" } else { "=" };
        let desc = format_param_desc(param.description.as_deref());
        let ty = js_type(&param.ty);
        let ident = sanitize_ident(&param.name);
        let name_decl = if param.required {
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
        let optional = if body.required { "" } else { "=" };
        let ty = js_type(&body.ty);
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

    let mut params = build_method_params(endpoint);
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
        let body_arg = if body.is_some() { "body" } else { "''" };
        ("post", format!("path, {body_arg}, {options}"))
    } else {
        ("get", format!("path, {options}"))
    };
    // Series data gets the same helpers however it is fetched.
    let response = endpoint.response_kind.ty();
    let fetch_call = if endpoint.returns_binary() {
        format!("this.{verb}Bytes({args})")
    } else if response.is_some_and(|ty| ty.is_ref("SeriesData")) {
        "_wrapSeriesData(await this.getJson(path, { signal, cache, memCache, onValue: onValue && ((v) => onValue(_wrapSeriesData(v))) }))".to_string()
    } else if matches!(response, Some(Ty::List(item)) if item.is_ref("SeriesData")) {
        "(await this.getJson(path, { signal, cache, memCache, onValue: onValue && ((v) => onValue(v.map(_wrapSeriesData))) })).map(_wrapSeriesData)".to_string()
    } else if endpoint.returns_json() {
        // Map keys are data: they stay as sent while field names are camelCased.
        match response.and_then(|ty| keep_expr(ty, schemas)) {
            Some(keep) => format!("this.{verb}Json({args}, {keep})"),
            None => format!("this.{verb}Json({args})"),
        }
    } else if endpoint.response_kind.text_is_numeric() && post {
        format!("Number(await this.postText({args}))")
    } else if endpoint.response_kind.text_is_numeric() {
        "Number(await this.getText(path, { signal, cache, memCache, onValue: onValue ? (v) => onValue(Number(v)) : undefined }))".to_string()
    } else {
        format!("this.{verb}Text({args})")
    };

    let path = build_path_template(&endpoint.path, &endpoint.path_params);
    write_path_assignment(output, endpoint, &path);

    if endpoint.supports_csv {
        writeln!(
            output,
            "    if (format === 'csv') return this.getText(path, {{ signal, onValue, cache, memCache }});"
        )
        .unwrap();
    }
    writeln!(output, "    return {};", fetch_call).unwrap();
    writeln!(output, "  }}\n").unwrap();
}

fn build_return_type(endpoint: &Endpoint) -> String {
    let base = endpoint.response_kind.ty().map_or_else(
        || "Uint8Array".to_owned(),
        |ty| normalize_return_type(&js_type(ty)),
    );
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
            let ident = sanitize_ident(&param.name);
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
                        "    if ({}) for (const _v of {}) params.append('{}', String(_v));",
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

fn build_method_params(endpoint: &Endpoint) -> String {
    let mut params = String::new();
    for (index, param) in endpoint
        .path_params
        .iter()
        .chain(&endpoint.query_params)
        .enumerate()
    {
        if index != 0 {
            params.push_str(", ");
        }
        params.push_str(&sanitize_ident(&param.name));
    }
    params
}

/// Drop `[` and `]` (as in `txId[]`).
fn sanitize_ident(name: &str) -> String {
    name.replace(['[', ']'], "")
}

/// The path as a template literal, each parameter percent-encoded as one segment.
fn build_path_template(path: &str, path_params: &[Parameter]) -> String {
    let mut result = path.to_string();
    for param in path_params {
        let placeholder = format!("{{{}}}", param.name);
        let interpolation = format!("${{encodeURIComponent({})}}", param.name);
        result = result.replace(&placeholder, &interpolation);
    }
    result
}

/// Format param description with dash prefix, or empty string if no description.
fn format_param_desc(desc: Option<&str>) -> String {
    match desc {
        Some(d) if !d.is_empty() => format!(" - {}", d),
        _ => String::new(),
    }
}
