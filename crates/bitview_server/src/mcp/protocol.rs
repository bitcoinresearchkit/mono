//! The MCP methods a stateless server answers, under the lifecycle rules of the three revisions it serves:
//! `initialize` for 2025-06-18 and 2025-11-25, per-request `_meta` for 2026-07-28. Every revision has the tools;
//! 2026-07-28 also has Bitview's skill, through the skills extension (SEP-2640) and the resources that carry its files.
//! Nothing is kept between requests.

use std::borrow::Cow;

use axum::http::{HeaderMap, StatusCode};
use base64::{Engine as _, prelude::BASE64_STANDARD};
use serde_json::{Map, Value, json};

use super::{AppState, catalog::Skill};

/// The newest revision, back to the oldest one a real client still opens with: Codex and Gemini CLI at 2025-06-18.
pub const SUPPORTED_PROTOCOL_VERSIONS: [&str; 3] = ["2026-07-28", "2025-11-25", "2025-06-18"];
/// The first revision with per-request `_meta`, `server/discover` and the standard headers, and without `initialize`
/// or `ping`.
const PER_REQUEST_REVISION: &str = "2026-07-28";
/// What `initialize` answers a client asking for a revision it can't have: the newest one that still has it.
const NEWEST_WITH_INITIALIZE: &str = "2025-11-25";
const META_PROTOCOL_VERSION: &str = "io.modelcontextprotocol/protocolVersion";
const META_CLIENT_CAPABILITIES: &str = "io.modelcontextprotocol/clientCapabilities";
const SERVER_INFO_META: &str = "io.modelcontextprotocol/serverInfo";
const CATALOG_TTL_MS: u64 = 3_600_000;
const SKILLS_EXTENSION: &str = "io.modelcontextprotocol/skills";
/// How a 2026-07-28 result says it is the answer itself.
const COMPLETE: &str = r#""resultType":"complete","#;

const PARSE_ERROR: i32 = -32700;
const INVALID_REQUEST: i32 = -32600;
const METHOD_NOT_FOUND: i32 = -32601;
const INVALID_PARAMS: i32 = -32602;
const HEADER_MISMATCH: i32 = -32020;

/// A JSON-RPC answer: `None` is a notification's 202, which has no body.
pub struct Answer {
    pub status: StatusCode,
    pub body: Option<Vec<u8>>,
}

/// The results that never change, written once at startup.
pub struct Results {
    server_info: Value,
    instructions: String,
    tools_legacy: String,
    tools_per_request: String,
    discover: String,
    skills_list: String,
    resources_list: String,
    /// The skill's `SKILL.md` URI, and the `skills/get` result for it.
    skill: (String, String),
    /// Each skill file's URI and `resources/read` result.
    skill_files: Vec<(String, String)>,
}

impl Results {
    pub fn new(
        server_info: Value,
        server_meta: &Map<String, Value>,
        instructions: &str,
        tools_json: &str,
        skill: &Skill,
    ) -> Self {
        let meta = Value::Object(server_meta.clone());
        // A cacheable result: the same for everyone, for an hour.
        let cacheable = |result_type: &str, key: &str, value: &str| {
            format!(
                r#"{{{result_type}"_meta":{meta},"ttlMs":{CATALOG_TTL_MS},"cacheScope":"public","{key}":{value}}}"#
            )
        };

        let name = skill.frontmatter["name"].as_str().expect("a skill has a name");
        let uri = |path: &str| format!("skill://{name}/{path}");
        let skill_uri = uri("SKILL.md");
        debug_assert_eq!(skill.files[0].path, "SKILL.md");
        let entry = json!({
            "uri": skill_uri,
            "frontmatter": skill.frontmatter,
            "resources": skill.files.iter().map(|file| json!({
                "uri": uri(&file.path),
                "digest": file.digest,
                "size": file.size,
            })).collect::<Vec<_>>(),
        });
        // The skill's SKILL.md goes by its name and description, as the extension asks; a reference by its path.
        let resources = skill.files.iter().map(|file| {
            let mut resource = json!({ "uri": uri(&file.path), "name": file.path, "mimeType": "text/markdown" });
            if file.path == "SKILL.md" {
                resource["name"] = skill.frontmatter["name"].clone();
                resource["description"] = skill.frontmatter["description"].clone();
            }
            resource
        });
        let skill_files = skill
            .files
            .iter()
            .map(|file| {
                let contents = json!([{ "uri": uri(&file.path), "mimeType": "text/markdown", "text": file.text }]);
                (uri(&file.path), cacheable(COMPLETE, "contents", &contents.to_string()))
            })
            .collect();

        let discover = json!({
            "resultType": "complete",
            "supportedVersions": SUPPORTED_PROTOCOL_VERSIONS,
            "capabilities": { "tools": {}, "resources": {}, "extensions": { SKILLS_EXTENSION: {} } },
            // Only this revision's instructions point at the skill: older ones can't read it.
            "instructions": format!(
                "{instructions}\n\nThe skill {skill_uri} holds the whole workflow, every index and the tools by \
                 topic: load it before a series question."
            ),
            "ttlMs": CATALOG_TTL_MS,
            "cacheScope": "public",
            "_meta": { SERVER_INFO_META: server_info },
        });
        Self {
            tools_legacy: cacheable("", "tools", tools_json),
            tools_per_request: cacheable(COMPLETE, "tools", tools_json),
            discover: discover.to_string(),
            skills_list: cacheable(COMPLETE, "skills", &json!([entry]).to_string()),
            resources_list: cacheable(COMPLETE, "resources", &Value::Array(resources.collect()).to_string()),
            skill: (skill_uri, cacheable(COMPLETE, "skill", &entry.to_string())),
            skill_files,
            instructions: instructions.to_owned(),
            server_info,
        }
    }

    /// `skills/get`: the entry of the skill whose `SKILL.md` is at `params.uri`.
    fn skill(&self, params: &Map<String, Value>) -> Result<Reply<'_>, RpcError> {
        let uri = uri_param(params, "skills/get")?;
        let (skill_uri, result) = &self.skill;
        if uri != skill_uri {
            return Err(error(INVALID_PARAMS, format!("No skill is served at {uri}")));
        }
        Ok(Reply::Written(result))
    }

    /// `resources/read`: one of the skill's files.
    fn read(&self, params: &Map<String, Value>) -> Result<Reply<'_>, RpcError> {
        let uri = uri_param(params, "resources/read")?;
        self.skill_files
            .iter()
            .find(|(file_uri, _)| file_uri == uri)
            .map(|(_, result)| Reply::Written(result))
            .ok_or_else(|| error(INVALID_PARAMS, format!("Resource not found: {uri}")))
    }
}

fn uri_param<'a>(params: &'a Map<String, Value>, method: &str) -> Result<&'a str, RpcError> {
    params
        .get("uri")
        .and_then(Value::as_str)
        .ok_or_else(|| error(INVALID_PARAMS, format!("Invalid params: {method} needs params.uri")))
}

/// The `io.modelcontextprotocol/serverInfo` `_meta` every list and call result carries.
pub fn server_meta(name: &str) -> Map<String, Value> {
    let mut meta = Map::new();
    meta.insert(SERVER_INFO_META.to_owned(), json!({ "name": name, "version": env!("CARGO_PKG_VERSION") }));
    meta
}

/// A tool call's result: the content blocks, whether they report an error, and the result's `_meta`.
pub struct ToolResult {
    pub content: Value,
    pub is_error: bool,
    pub meta: Map<String, Value>,
}

/// What a method answers: a result written at startup, or one built for this request.
enum Reply<'a> {
    Written(&'a str),
    Built(Value),
}

struct RpcError {
    code: i32,
    message: Cow<'static, str>,
}

fn error(code: i32, message: impl Into<Cow<'static, str>>) -> RpcError {
    RpcError {
        code,
        message: message.into(),
    }
}

/// Answers one message. `header_version` is the `MCP-Protocol-Version` header, already checked to be one we serve.
pub async fn answer(state: &AppState, headers: &HeaderMap, header_version: Option<&str>, body: &[u8]) -> Answer {
    let message = match serde_json::from_slice::<Value>(body) {
        Ok(Value::Object(message)) => message,
        Ok(Value::Array(_)) => return refused(None, error(INVALID_REQUEST, "Invalid Request: batches are not supported")),
        Ok(_) => return refused(None, error(INVALID_REQUEST, "Invalid Request: not a JSON-RPC message")),
        Err(parse) => return refused(None, error(PARSE_ERROR, format!("Parse error: {parse}"))),
    };
    let method = match (message.get("jsonrpc"), message.get("method")) {
        (Some(Value::String(jsonrpc)), Some(Value::String(method))) if jsonrpc == "2.0" => method.as_str(),
        (_, None) if message.contains_key("result") || message.contains_key("error") => {
            return refused(None, error(INVALID_REQUEST, "stateless server does not accept JSON-RPC responses"));
        }
        _ => return refused(None, error(INVALID_REQUEST, "Invalid Request: not a JSON-RPC 2.0 request")),
    };
    let id = match message.get("id") {
        None => None,
        Some(id @ (Value::String(_) | Value::Number(_))) => Some(id),
        Some(_) => return refused(None, error(INVALID_REQUEST, "Invalid Request: an id is a string or a number")),
    };
    let empty = Map::new();
    let params = match message.get("params") {
        None | Some(Value::Null) => &empty,
        Some(Value::Object(params)) => params,
        Some(_) => return refused(id, error(INVALID_PARAMS, "Invalid params: params must be an object")),
    };
    let per_request_revision = header_version == Some(PER_REQUEST_REVISION);

    let Some(id) = id else {
        if per_request_revision && let Err(mismatch) = standard_headers(headers, method, params) {
            return refused(None, error(HEADER_MISMATCH, mismatch));
        }
        return Answer {
            status: StatusCode::ACCEPTED,
            body: None,
        };
    };

    if method == "initialize" {
        let Some(requested) = params.get("protocolVersion").and_then(Value::as_str) else {
            return replied(id, Err(error(INVALID_PARAMS, "Invalid params: initialize needs params.protocolVersion")), false);
        };
        if let Some(header) = header_version
            && header != requested
        {
            return refused(Some(id), error(INVALID_REQUEST, format!(
                "Invalid Request: MCP-Protocol-Version header ({header}) does not match initialize params.protocolVersion ({requested})"
            )));
        }
        if per_request_revision && let Err(mismatch) = initialize_method_header(headers) {
            return refused(Some(id), error(HEADER_MISMATCH, mismatch));
        }
        return replied(id, Ok(Reply::Built(initialize(&state.results, requested))), false);
    }

    if per_request_revision && let Err(mismatch) = standard_headers(headers, method, params) {
        return refused(Some(id), error(HEADER_MISMATCH, mismatch));
    }
    let discover = method == "server/discover";
    let meta = params.get("_meta").and_then(Value::as_object);
    let meta_version = meta.and_then(|meta| meta.get(META_PROTOCOL_VERSION)).and_then(Value::as_str);
    let has_capabilities = meta
        .and_then(|meta| meta.get(META_CLIENT_CAPABILITIES))
        .is_some_and(Value::is_object);
    match meta_version {
        None if discover || per_request_revision => {
            let missing = [Some(META_PROTOCOL_VERSION), (!has_capabilities).then_some(META_CLIENT_CAPABILITIES)];
            let missing = missing.into_iter().flatten().collect::<Vec<_>>().join(", ");
            return refused(Some(id), error(INVALID_PARAMS, format!(
                "Invalid params: request _meta is missing or has malformed required fields: {missing}"
            )));
        }
        None => {}
        Some(meta_version) => {
            let Some(header) = header_version else {
                return refused(Some(id), error(HEADER_MISMATCH, "request _meta protocolVersion requires MCP-Protocol-Version header"));
            };
            if header != meta_version {
                return refused(Some(id), error(HEADER_MISMATCH, format!(
                    "MCP-Protocol-Version header ({header}) does not match request _meta protocolVersion ({meta_version})"
                )));
            }
            if (per_request_revision || discover) && !has_capabilities {
                return refused(Some(id), error(INVALID_PARAMS, format!(
                    "request _meta is missing or has malformed required fields: {META_CLIENT_CAPABILITIES}"
                )));
            }
        }
    }

    let results = &state.results;
    let cursor = params.get("cursor").is_some_and(|cursor| !cursor.is_null());
    let result = match method {
        "ping" if !per_request_revision => Ok(Reply::Written("{}")),
        "server/discover" => Ok(Reply::Written(&results.discover)),
        "tools/list" if cursor => Err(error(INVALID_PARAMS, "The tool catalog does not accept a cursor")),
        "tools/list" if per_request_revision => Ok(Reply::Written(&results.tools_per_request)),
        "tools/list" => Ok(Reply::Written(&results.tools_legacy)),
        "tools/call" => tool_call(state, params, per_request_revision).await.map(Reply::Built),
        "skills/list" | "resources/list" if per_request_revision && cursor => {
            Err(error(INVALID_PARAMS, format!("{method} does not accept a cursor")))
        }
        "skills/list" if per_request_revision => Ok(Reply::Written(&results.skills_list)),
        "skills/get" if per_request_revision => results.skill(params),
        "resources/list" if per_request_revision => Ok(Reply::Written(&results.resources_list)),
        "resources/read" if per_request_revision => results.read(params),
        _ => Err(error(METHOD_NOT_FOUND, "Method not found")),
    };
    replied(id, result, meta_version.is_some() || discover)
}

async fn tool_call(state: &AppState, params: &Map<String, Value>, per_request_revision: bool) -> Result<Value, RpcError> {
    let Some(name) = params.get("name").and_then(Value::as_str) else {
        return Err(error(INVALID_PARAMS, "Invalid params: tools/call needs params.name"));
    };
    let empty = Map::new();
    let arguments = match params.get("arguments") {
        None | Some(Value::Null) => &empty,
        Some(Value::Object(arguments)) => arguments,
        Some(_) => return Err(error(INVALID_PARAMS, "Invalid params: tools/call arguments must be an object")),
    };
    let ToolResult { content, is_error, meta } = state
        .call(name, arguments)
        .await
        .ok_or_else(|| error(INVALID_PARAMS, "Unknown tool name"))?;
    let mut result = Map::new();
    if per_request_revision {
        result.insert("resultType".to_owned(), "complete".into());
    }
    result.insert("content".to_owned(), content);
    result.insert("isError".to_owned(), is_error.into());
    result.insert("_meta".to_owned(), Value::Object(meta));
    Ok(Value::Object(result))
}

fn initialize(results: &Results, requested: &str) -> Value {
    let version = SUPPORTED_PROTOCOL_VERSIONS
        .into_iter()
        .find(|&version| version == requested && version < PER_REQUEST_REVISION)
        .unwrap_or(NEWEST_WITH_INITIALIZE);
    json!({
        "protocolVersion": version,
        "capabilities": { "tools": {} },
        "serverInfo": results.server_info,
        "instructions": results.instructions,
    })
}

/// SEP-2243: a 2026-07-28 request names its method in `Mcp-Method`, and a call its tool in `Mcp-Name`, as its body does.
fn standard_headers(headers: &HeaderMap, method: &str, params: &Map<String, Value>) -> Result<(), String> {
    match single_header(headers, "Mcp-Method")? {
        None => return Err("missing required Mcp-Method header".to_owned()),
        Some(header) if header != method => {
            return Err(format!("Mcp-Method header `{header}` does not match body method `{method}`"));
        }
        Some(_) => {}
    }
    let name_key = match method {
        "tools/call" | "prompts/get" => "name",
        "resources/read" => "uri",
        _ => return Ok(()),
    };
    let Some(expected) = params.get(name_key).and_then(Value::as_str) else {
        return Ok(());
    };
    let Some(header) = single_header(headers, "Mcp-Name")? else {
        return Err(format!("missing required Mcp-Name header for `{method}`"));
    };
    let decoded = match header.strip_prefix("=?base64?").and_then(|inner| inner.strip_suffix("?=")) {
        Some(encoded) => BASE64_STANDARD
            .decode(encoded)
            .ok()
            .and_then(|bytes| String::from_utf8(bytes).ok())
            .ok_or("Mcp-Name header is not valid Base64")?,
        None => header.to_owned(),
    };
    if decoded != expected {
        return Err(format!("Mcp-Name header `{decoded}` does not match body value `{expected}`"));
    }
    Ok(())
}

/// `initialize` may leave out `Mcp-Method`, but a supplied one must be the only one and say `initialize`.
fn initialize_method_header(headers: &HeaderMap) -> Result<(), String> {
    let mut values = headers.get_all("Mcp-Method").iter();
    let Some(first) = values.next() else {
        return Ok(());
    };
    if values.next().is_some() {
        return Err("multiple Mcp-Method header values supplied; initialize requires exactly one".to_owned());
    }
    if first != "initialize" {
        let first = String::from_utf8_lossy(first.as_bytes());
        return Err(format!("Mcp-Method header `{first}` does not match body method `initialize`"));
    }
    Ok(())
}

/// A header that may appear once, as text.
fn single_header<'a>(headers: &'a HeaderMap, name: &str) -> Result<Option<&'a str>, String> {
    let mut values = headers.get_all(name).iter();
    let Some(value) = values.next() else {
        return Ok(None);
    };
    if values.next().is_some() {
        return Err(format!("duplicate {name} header"));
    }
    value
        .to_str()
        .map(Some)
        .map_err(|_| format!("{name} header contains invalid characters"))
}

/// A request refused before it ran: always 400, as every revision answers a malformed request.
fn refused(id: Option<&Value>, error: RpcError) -> Answer {
    Answer {
        status: StatusCode::BAD_REQUEST,
        body: Some(error_body(id, &error)),
    }
}

/// A method's answer. Only a request using per-request `_meta` gets 2026-07-28's status for an error: 404 for an
/// unknown method; every other error is a JSON-RPC answer with 200, as before it.
fn replied(id: &Value, result: Result<Reply, RpcError>, per_request: bool) -> Answer {
    match result {
        Ok(reply) => Answer {
            status: StatusCode::OK,
            body: Some(envelope(id, reply)),
        },
        Err(error) => Answer {
            status: if per_request && error.code == METHOD_NOT_FOUND { StatusCode::NOT_FOUND } else { StatusCode::OK },
            body: Some(error_body(Some(id), &error)),
        },
    }
}

/// The result inside its envelope, written once into the reply's bytes: a tool's answer is up to 2 MiB, so it is
/// never copied again on its way to the client.
fn envelope(id: &Value, reply: Reply) -> Vec<u8> {
    const HEAD: &[u8] = br#"{"jsonrpc":"2.0","id":"#;
    const RESULT: &[u8] = br#","result":"#;
    let result_len = match &reply {
        Reply::Written(result) => result.len(),
        // A tool's answer is one text or blob: room for it and a few escapes, so the bytes are rarely moved.
        Reply::Built(result) => ["/content/0/text", "/content/0/resource/blob"]
            .into_iter()
            .find_map(|pointer| result.pointer(pointer).and_then(Value::as_str))
            .map_or(256, |answer| answer.len() + answer.len() / 8 + 256),
    };
    let mut body = Vec::with_capacity(HEAD.len() + 32 + RESULT.len() + result_len + 1);
    body.extend_from_slice(HEAD);
    serde_json::to_writer(&mut body, id).expect("a JSON value serializes");
    body.extend_from_slice(RESULT);
    match reply {
        Reply::Written(result) => body.extend_from_slice(result.as_bytes()),
        Reply::Built(result) => serde_json::to_writer(&mut body, &result).expect("a JSON value serializes"),
    }
    body.push(b'}');
    body
}

/// An error reply; one whose request's id couldn't be read says `"id": null`, as JSON-RPC 2.0 asks.
fn error_body(id: Option<&Value>, error: &RpcError) -> Vec<u8> {
    let mut body = Map::new();
    body.insert("jsonrpc".to_owned(), "2.0".into());
    body.insert("id".to_owned(), id.cloned().unwrap_or(Value::Null));
    body.insert("error".to_owned(), json!({ "code": error.code, "message": error.message }));
    serde_json::to_vec(&body).expect("a JSON value serializes")
}
