use axum::http::Uri;

pub fn api_bases(input: &str) -> Result<Vec<String>, String> {
    let input = input.trim().trim_end_matches('/');
    if input.is_empty() {
        return Err("REST API URL or host must not be empty".to_string());
    }
    let bases = if input.contains("://") {
        vec![input.to_string()]
    } else {
        vec![format!("https://{input}"), format!("http://{input}")]
    };
    for base in &bases {
        validate_api_base(base)?;
    }
    Ok(bases)
}

pub fn public_url(input: &str) -> Result<String, String> {
    let input = input.trim().trim_end_matches('/');
    if input.is_empty() {
        return Err("public MCP URL must not be empty".to_owned());
    }
    validate_origin(input, "public MCP")?;
    Ok(format!("{input}/"))
}

fn validate_api_base(base: &str) -> Result<(), String> {
    validate_origin(base, "REST API")
}

fn validate_origin(base: &str, label: &str) -> Result<(), String> {
    let uri: Uri = base
        .parse()
        .map_err(|error| format!("invalid {label} origin: {error}"))?;
    if !matches!(uri.scheme_str(), Some("http" | "https")) || uri.authority().is_none() {
        return Err(format!("{label} origin must be an absolute HTTP(S) URL"));
    }
    if uri.query().is_some() || !matches!(uri.path(), "" | "/") {
        return Err(format!("{label} origin must not contain a path or query"));
    }
    if uri
        .authority()
        .is_some_and(|authority| authority.as_str().rsplit_once('@').is_some())
    {
        return Err(format!("{label} origin must not contain credentials"));
    }
    Ok(())
}
