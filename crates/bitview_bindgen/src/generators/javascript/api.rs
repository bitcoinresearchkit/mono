//! JavaScript API method generation.

use std::fmt::Write;

use crate::{
    Endpoint, Parameter,
    generators::{javascript::types::jsdoc_normalize, normalize_return_type, write_description},
    to_camel_case,
};

/// Generate API methods for the BitviewClient class.
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
    for param in &endpoint.path_params {
        let desc = format_param_desc(param.description.as_deref());
        let ty = jsdoc_normalize(&param.param_type);
        writeln!(output, "   * @param {{{}}} {}{}", ty, param.name, desc).unwrap();
    }
    for param in &endpoint.query_params {
        let optional = if param.required { "" } else { "=" };
        let desc = format_param_desc(param.description.as_deref());
        let ty = jsdoc_normalize(&param.param_type);
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
    writeln!(
        output,
        "   * @param {{{{ signal?: AbortSignal, onValue?: (value: {}) => void, cache?: boolean, memCache?: boolean }}}} [options]",
        return_type
    )
    .unwrap();
    writeln!(output, "   * @returns {{Promise<{}>}}", return_type).unwrap();
    writeln!(output, "   */").unwrap();

    let params = build_method_params(endpoint);
    let params_with_opts = if params.is_empty() {
        "{ signal, onValue, cache, memCache } = {}".to_string()
    } else {
        format!("{}, {{ signal, onValue, cache, memCache }} = {{}}", params)
    };
    writeln!(output, "  async {}({}) {{", method_name, params_with_opts).unwrap();

    let path = build_path_template(&endpoint.path, &endpoint.path_params);

    let fetch_call: String = if endpoint.returns_binary() {
        "this.getBytes(path, { signal, onValue, cache, memCache })".to_string()
    } else if endpoint.returns_json() {
        "this.getJson(path, { signal, onValue, cache, memCache })".to_string()
    } else if endpoint.response_kind.text_is_numeric() {
        "Number(await this.getText(path, { signal, cache, memCache, onValue: onValue ? (v) => onValue(Number(v)) : undefined }))".to_string()
    } else {
        "this.getText(path, { signal, onValue, cache, memCache })".to_string()
    };

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

fn generate_post_method(output: &mut String, endpoint: &Endpoint) {
    let method_name = endpoint_to_method_name(endpoint);
    let return_type = build_return_type(endpoint);

    write_method_doc(output, endpoint);
    for param in &endpoint.path_params {
        let desc = format_param_desc(param.description.as_deref());
        let ty = jsdoc_normalize(&param.param_type);
        writeln!(output, "   * @param {{{}}} {}{}", ty, param.name, desc).unwrap();
    }
    for param in &endpoint.query_params {
        let optional = if param.required { "" } else { "=" };
        let desc = format_param_desc(param.description.as_deref());
        let ty = jsdoc_normalize(&param.param_type);
        writeln!(
            output,
            "   * @param {{{}{}}} [{}]{}",
            ty, optional, param.name, desc
        )
        .unwrap();
    }
    if let Some(body) = &endpoint.request_body {
        let optional = if body.required { "" } else { "=" };
        let ty = jsdoc_normalize(&body.body_type);
        writeln!(
            output,
            "   * @param {{{}{}}} body - Request body",
            ty, optional
        )
        .unwrap();
    }
    writeln!(
        output,
        "   * @param {{{{ signal?: AbortSignal }}}} [options]"
    )
    .unwrap();
    writeln!(output, "   * @returns {{Promise<{}>}}", return_type).unwrap();
    writeln!(output, "   */").unwrap();

    let mut params = build_method_params(endpoint);
    if endpoint.request_body.is_some() {
        if !params.is_empty() {
            params.push_str(", ");
        }
        params.push_str("body");
    }
    let params_with_opts = if params.is_empty() {
        "{ signal } = {}".to_string()
    } else {
        format!("{}, {{ signal }} = {{}}", params)
    };
    writeln!(output, "  async {}({}) {{", method_name, params_with_opts).unwrap();

    let path = build_path_template(&endpoint.path, &endpoint.path_params);
    let body_arg = if endpoint.request_body.is_some() {
        "body"
    } else {
        "''"
    };

    let fetch_call: String = if endpoint.returns_binary() {
        format!("this.postBytes(path, {}, {{ signal }})", body_arg)
    } else if endpoint.returns_json() {
        format!("this.postJson(path, {}, {{ signal }})", body_arg)
    } else if endpoint.response_kind.text_is_numeric() {
        format!(
            "Number(await this.postText(path, {}, {{ signal }}))",
            body_arg
        )
    } else {
        format!("this.postText(path, {}, {{ signal }})", body_arg)
    };

    write_path_assignment(output, endpoint, &path);

    writeln!(output, "    return {};", fetch_call).unwrap();
    writeln!(output, "  }}\n").unwrap();
}

fn build_return_type(endpoint: &Endpoint) -> String {
    let base = if endpoint.returns_binary() {
        "Uint8Array".to_string()
    } else {
        jsdoc_normalize(&normalize_return_type(
            endpoint.schema_name().unwrap_or("*"),
        ))
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
        endpoint.method.to_uppercase(),
        endpoint.path
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
            let is_array = param.param_type.ends_with("[]");
            if is_array {
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

fn endpoint_to_method_name(endpoint: &Endpoint) -> String {
    to_camel_case(&endpoint.operation_name())
}

fn build_method_params(endpoint: &Endpoint) -> String {
    let mut params = Vec::new();
    for param in &endpoint.path_params {
        params.push(sanitize_ident(&param.name));
    }
    for param in &endpoint.query_params {
        params.push(sanitize_ident(&param.name));
    }
    params.join(", ")
}

/// Strip characters invalid in JS identifiers (e.g. `[]` from `txId[]`).
fn sanitize_ident(name: &str) -> String {
    name.replace(['[', ']'], "")
}

fn build_path_template(path: &str, path_params: &[Parameter]) -> String {
    let mut result = path.to_string();
    for param in path_params {
        let placeholder = format!("{{{}}}", param.name);
        let interpolation = format!("${{{}}}", param.name);
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
