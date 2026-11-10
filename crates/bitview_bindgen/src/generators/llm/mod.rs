use std::{
    collections::{BTreeMap, BTreeSet},
    fmt::Write,
    io,
    path::Path,
};

use bitview_catalog::TreeNode;
use serde_json::{Value, json, to_string_pretty};
use sha2::{Digest, Sha256};

use super::write_if_changed;
use crate::{
    ClientOutputPaths, Endpoint, ResponseKind, Ty, TypeSchemas, collect_schema_refs, model::Model,
    openapi::Spec,
};

mod manifest;
mod skill;

const BASE_URL: &str = "https://bitview.space";
const MCP_URL: &str = "https://mcp.bitview.space/";
const DEFINITION: &str = "Bitview is a free, open-source Bitcoin data server: on-chain metrics, blocks, mempool and mining data, with charts and an API, at bitview.space or beside your own Bitcoin Core node.";
/// What an agent can ask Bitview: the skill's trigger and the MCP instructions' opening.
const SCOPE: &str = "on-chain metrics (MVRV, realized price, SOPR, holder cohorts, supply by age), blocks, transactions, addresses, mempool and fees, mining and difficulty, and the BTC/USD price computed from the chain";
const ACCESS: &str = "Free and read-only, with no API key.";
/// The series flow, written once: each step and the operations it calls, named as routes in
/// llms.txt and the skill, as tools in the MCP instructions.
const SERIES_FLOW: [(&str, &[&str]); 3] = [
    (
        "search by meaning with plain words; it returns series ids",
        &["search_series"],
    ),
    (
        "read a returned id's description, supported indexes, value type, unit (what the value type measures), and the indexes whose values can be null (missing or undefined)",
        &["get_series_info"],
    ),
    (
        "pick one of those indexes and fetch its values (`start` inclusive, `end` exclusive), the latest value alone, or several comma-separated series at once",
        &["get_series", "get_series_latest", "get_series_bulk"],
    ),
];
const SERIES_RULE: &str = "Never invent a series id: only ids the API returned exist.";
/// What agents get wrong reading series, for the skill and the MCP instructions.
const TRAPS: &str = "- Index names are exact (`day1`, not `day`); a wrong one is rejected with the valid list.\n\
- `end` is exclusive: one block is start 900000, end 900001; one day is start 2025-01-01, end 2025-01-02. The latest value is the newest one only, never a past one.\n\
- A `Ratio` or `Multiple` is a plain number (0.55, not 55%); a `Percent` is already times 100. Convert only as asked.\n\
- Compute derived numbers at full precision; round only the final answer, if at all.\n\
- `null` means undefined (before the data exists), never zero.\n";
const SKILL: &str = "bitview";
/// Where a site serves its skills: the agent-skills discovery RFC's convention (v0.2.0).
const SKILLS_PATH: &str = ".well-known/agent-skills";
const REPOSITORY: &str = "https://github.com/bitcoinresearchkit/mono";

pub(crate) fn generate_llm_clients(
    catalog: &TreeNode,
    model: &Model,
    spec: &Spec,
    endpoints: &[Endpoint],
    schemas: &TypeSchemas,
    paths: &ClientOutputPaths,
) -> io::Result<()> {
    let (roots, plugins) = (&paths.llm, &paths.plugin);
    if !plugins.is_empty() || !roots.is_empty() {
        let generated = endpoints
            .iter()
            .filter(|endpoint| endpoint.should_generate())
            .collect::<Vec<_>>();
        let flow = render_series_flow(&generated, route)?;
        let metric_count = count_metrics(catalog);
        let skill = skill::render(catalog, model, endpoints, metric_count, &flow)?;
        let plugin = render_plugin(&spec.info.version)?;
        for root in plugins {
            write_skill(&root.join("skills").join(SKILL), &skill)?;
            write_if_changed(&root.join(".claude-plugin/plugin.json"), &plugin)?;
        }
        let index = render_skill_index(&skill[0].1)?;
        let llms = render_llms(
            &spec.info.title,
            &spec.info.version,
            metric_count,
            &generated,
            &flow,
        );
        let llms_full = render_llms_full(
            &spec.info.title,
            &spec.info.version,
            metric_count,
            &generated,
            schemas,
        );
        for root in roots {
            write_if_changed(&root.join("llms.txt"), &llms)?;
            write_if_changed(&root.join("llms-full.txt"), &llms_full)?;
            let skills = root.join(SKILLS_PATH);
            write_if_changed(&skills.join("index.json"), &index)?;
            write_skill(&skills.join(SKILL), &skill)?;
        }
    }
    if let Some(path) = &paths.llm_manifest {
        manifest::generate_tool_manifest(
            endpoints,
            schemas,
            &render_mcp_instructions(endpoints)?,
            path,
        )?;
    }
    Ok(())
}

/// Distinct series ids: a series reached through several typed paths counts once.
fn count_metrics(node: &TreeNode) -> usize {
    let mut names = BTreeSet::new();
    node.for_each_leaf(&mut |leaf| {
        names.insert(&leaf.leaf.name);
    });
    names.len()
}

fn write_skill(folder: &Path, files: &skill::Files) -> io::Result<()> {
    for (path, content) in files {
        write_if_changed(&folder.join(path), content)?;
    }
    Ok(())
}

/// The operation `id` among those offered: one renamed or removed fails generation.
fn find<'a>(endpoints: &[&'a Endpoint], id: &str) -> io::Result<&'a Endpoint> {
    endpoints
        .iter()
        .copied()
        .find(|endpoint| endpoint.operation_id == id)
        .ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("the LLM references name a missing operation: {id}"),
            )
        })
}

/// The series flow as a numbered list, each step naming its operations through `name`.
fn render_series_flow(
    endpoints: &[&Endpoint],
    name: fn(&Endpoint) -> String,
) -> io::Result<String> {
    let mut output = String::new();
    for (step, (text, operations)) in SERIES_FLOW.iter().enumerate() {
        let names = operations
            .iter()
            .map(|id| Ok(format!("`{}`", name(find(endpoints, id)?))))
            .collect::<io::Result<Vec<_>>>()?;
        writeln!(output, "{}. {}: {text}.", step + 1, names.join(", ")).unwrap();
    }
    Ok(output)
}

/// `GET /api/series/search?q=<q>`: the route with its required query parameters.
fn route(endpoint: &Endpoint) -> String {
    let query = endpoint
        .query_params
        .iter()
        .filter(|parameter| parameter.required)
        .map(|parameter| format!("{0}=<{0}>", parameter.name))
        .collect::<Vec<_>>();
    if query.is_empty() {
        format!("{} {}", endpoint.method, endpoint.path)
    } else {
        format!("{} {}?{}", endpoint.method, endpoint.path, query.join("&"))
    }
}

/// The site's skill index, per the agent-skills discovery RFC v0.2.0: the digest is of the
/// served SKILL.md's bytes, its url relative to the index.
fn render_skill_index(skill: &str) -> io::Result<String> {
    let digest =
        Sha256::digest(skill.as_bytes())
            .iter()
            .fold(String::from("sha256:"), |mut hex, byte| {
                write!(hex, "{byte:02x}").unwrap();
                hex
            });
    pretty(&json!({
        "$schema": "https://schemas.agentskills.io/discovery/0.2.0/schema.json",
        "skills": [{
            "name": SKILL,
            "type": "skill-md",
            "description": skill::description(),
            "url": format!("{SKILL}/SKILL.md"),
            "digest": digest,
        }],
    }))
}

/// The Claude Code plugin: the skill beside the MCP server, versioned with the API.
fn render_plugin(version: &str) -> io::Result<String> {
    pretty(&json!({
        "name": SKILL,
        "displayName": "Bitview",
        "version": version.trim_start_matches('v'),
        "description": format!("Bitcoin data from Bitview, read from a full node: {SCOPE}. {ACCESS}"),
        "author": { "name": "Bitview" },
        "homepage": BASE_URL,
        "repository": REPOSITORY,
        "license": "MIT",
        "privacyPolicyUrl": format!("{BASE_URL}/privacy/"),
        "keywords": ["bitcoin", "blockchain", "on-chain", "mempool", "mining", "market-data"],
        "mcpServers": { "bitview": { "type": "http", "url": MCP_URL } },
    }))
}

fn pretty(value: &Value) -> io::Result<String> {
    let mut content = to_string_pretty(value)
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
    content.push('\n');
    Ok(content)
}

/// The MCP server's instructions: the skill's pointer, with tools in place of routes.
fn render_mcp_instructions(endpoints: &[Endpoint]) -> io::Result<String> {
    let tools = endpoints
        .iter()
        .filter(|endpoint| !endpoint.deprecated && !endpoint.mcp_ignored)
        .collect::<Vec<_>>();
    let flow = render_series_flow(&tools, |endpoint| endpoint.operation_id.clone())?;
    Ok(format!(
        "Live and historical Bitcoin data from Bitview, read from a full node: {SCOPE}. {ACCESS}\n\n\
For a series:\n\n\
{flow}\n\
{SERIES_RULE}\n\n\
{TRAPS}\n\
Source of this data: Bitview ({BASE_URL}). Binary responses are returned as embedded MCP resources."
    ))
}

fn render_llms(
    title: &str,
    version: &str,
    metric_count: usize,
    endpoints: &[&Endpoint],
    flow: &str,
) -> String {
    format!(
        "# {title}\n\n\
> {DEFINITION} {metric_count} on-chain time-series and {} API operations.\n\n\
## API\n\n\
- Version: `{version}`\n\
- Base URL: {BASE_URL}\n\
- Authentication: None\n\
- Browser access: Permissive CORS\n\
- [Full plain-text reference]({BASE_URL}/llms-full.txt)\n\
- [Compact API summary]({BASE_URL}/api.json)\n\
- [Full OpenAPI]({BASE_URL}/openapi.json)\n\
- [Series catalog]({BASE_URL}/api/series)\n\
- [Interactive documentation]({BASE_URL}/api)\n\n\
Build tools from the full OpenAPI document; the compact summary is a smaller custom (non-OpenAPI) overview. Use `llms-full.txt` only when a complete readable reference is needed.\n\n\
## Series workflow\n\n\
{flow}\n\
{SERIES_RULE}\n\n\
## MCP\n\n\
- Endpoint: {MCP_URL}\n\
- Transport: Streamable HTTP\n\
- Authentication: None\n\
- Protocol: MCP 2025-06-18 to 2026-07-28\n\
- [Setup]({MCP_URL})\n\n\
Add it to an assistant with that one URL:\n\n\
- Claude: Customize > Connectors > Add custom connector, paste the URL, choose No sign-in.\n\
- ChatGPT: chatgpt.com/plugins > plus button > Add custom MCP server, paste the URL, choose no authentication.\n\
- Claude Code: `claude mcp add --transport http bitview {MCP_URL}`\n\
- Codex: `codex mcp add bitview --url {MCP_URL}`\n\
- Gemini CLI: `gemini mcp add --scope user --transport http bitview {MCP_URL}`\n\
- Qwen Code: `qwen mcp add --transport http bitview {MCP_URL}`\n\n\
Tested with Claude Code, Codex, Cursor's CLI, Gemini CLI, OpenCode and Qwen Code. Claude on the web, ChatGPT and other clients on MCP 2025-06-18 or newer should work but haven't been tested. Problems: support@bitview.space.\n\n\
The MCP server is stateless and read-only. Its tools are generated from these OpenAPI operations.\n\n\
## Clients\n\n\
- [JavaScript](https://www.npmjs.com/package/bitview-client)\n\
- [Python](https://pypi.org/project/bitview-client/)\n\
- [Rust](https://crates.io/crates/bitview_client)\n\n\
## Agent Skill\n\n\
- [SKILL.md]({BASE_URL}/{SKILLS_PATH}/{SKILL}/SKILL.md), listed in the [skill index]({BASE_URL}/{SKILLS_PATH}/index.json)\n\
- Claude Code plugin with the skill and the MCP server: `claude plugin marketplace add bitcoinresearchkit/mono`, then `claude plugin install {SKILL}@{SKILL}`\n\n\
## Source\n\n\
- [GitHub]({REPOSITORY})\n\
- MIT licensed\n",
        endpoints.len(),
    )
}

fn render_llms_full(
    title: &str,
    version: &str,
    metric_count: usize,
    endpoints: &[&Endpoint],
    schemas: &TypeSchemas,
) -> String {
    let mut output = String::new();
    writeln!(output, "# {title} — Full API Reference\n").unwrap();
    writeln!(output, "> {DEFINITION}\n").unwrap();
    writeln!(
        output,
        "> Generated from BRK's OpenAPI specification and metric tree. Do not edit this file manually.\n"
    )
    .unwrap();
    writeln!(output, "- Version: `{version}`").unwrap();
    writeln!(output, "- Base URL: {BASE_URL}").unwrap();
    writeln!(output, "- MCP endpoint: {MCP_URL}").unwrap();
    writeln!(output, "- Metrics: {metric_count}").unwrap();
    writeln!(output, "- Operations: {}\n", endpoints.len()).unwrap();
    writeln!(
        output,
        "For machine-readable tool construction, use [{BASE_URL}/openapi.json]({BASE_URL}/openapi.json). For the complete source-derived series tree, use [{BASE_URL}/api/series]({BASE_URL}/api/series).\n"
    )
    .unwrap();

    let mut groups = BTreeMap::<String, Vec<&Endpoint>>::new();
    for endpoint in endpoints {
        groups
            .entry(endpoint_group(&endpoint.path))
            .or_default()
            .push(endpoint);
    }
    for group in groups.values_mut() {
        group.sort_unstable_by(|left, right| {
            left.path
                .cmp(&right.path)
                .then_with(|| left.method.cmp(&right.method))
        });
    }

    writeln!(output, "## Operations\n").unwrap();
    for (group, operations) in groups {
        writeln!(output, "### {group}\n").unwrap();
        for endpoint in operations {
            render_endpoint(&mut output, endpoint);
        }
    }

    let referenced = referenced_schemas(endpoints, schemas);
    if !referenced.is_empty() {
        writeln!(output, "## Schemas\n").unwrap();
        for name in referenced {
            render_schema(&mut output, name, &schemas[name]);
        }
    }
    output
}

fn endpoint_group(path: &str) -> String {
    title_case(endpoint_segment(path))
}

/// The segment naming a path's group: the first past `api` and `v1`.
fn endpoint_segment(path: &str) -> &str {
    path.split('/')
        .find(|part| !part.is_empty() && *part != "api" && *part != "v1")
        .unwrap_or("server")
}

fn title_case(value: &str) -> String {
    let mut result = String::new();
    for part in value.split(['-', '_']) {
        let mut chars = part.chars();
        if let Some(first) = chars.next() {
            if !result.is_empty() {
                result.push(' ');
            }
            result.extend(first.to_uppercase());
            result.push_str(chars.as_str());
        }
    }
    result
}

fn render_endpoint(output: &mut String, endpoint: &Endpoint) {
    writeln!(output, "#### {} `{}`\n", endpoint.method, endpoint.path).unwrap();
    if let Some(summary) = endpoint
        .summary
        .as_deref()
        .filter(|value| !value.trim().is_empty())
    {
        writeln!(output, "{}\n", one_line(summary)).unwrap();
    }
    if let Some(description) = endpoint
        .description
        .as_deref()
        .filter(|value| !value.trim().is_empty())
    {
        let description = one_line(description);
        if endpoint.summary.as_deref().map(one_line).as_deref() != Some(description.as_str()) {
            writeln!(output, "{description}\n").unwrap();
        }
    }

    let parameters = endpoint
        .path_params
        .iter()
        .map(|parameter| ("path", parameter))
        .chain(
            endpoint
                .query_params
                .iter()
                .map(|parameter| ("query", parameter)),
        )
        .collect::<Vec<_>>();
    if !parameters.is_empty() {
        writeln!(output, "Parameters:").unwrap();
        for (location, parameter) in parameters {
            let requirement = if parameter.required {
                "required"
            } else {
                "optional"
            };
            write!(
                output,
                "- `{}` ({location}, {}, {requirement})",
                parameter.name, parameter.ty
            )
            .unwrap();
            if let Some(description) = parameter
                .description
                .as_deref()
                .filter(|value| !value.trim().is_empty())
            {
                write!(output, ": {}", one_line(description)).unwrap();
            }
            writeln!(output).unwrap();
        }
        writeln!(output).unwrap();
    }

    if let Some(body) = &endpoint.request_body {
        writeln!(
            output,
            "Request body: `{}` ({})\n",
            body.ty,
            if body.required {
                "required"
            } else {
                "optional"
            }
        )
        .unwrap();
    }
    writeln!(
        output,
        "Returns: {}\n",
        response_label(&endpoint.response_kind)
    )
    .unwrap();
}

fn response_label(response: &ResponseKind) -> String {
    match response {
        ResponseKind::Json(ty) => format!("JSON `{ty}`"),
        ResponseKind::Text { ty: Ty::String, .. } => "text".to_owned(),
        ResponseKind::Text { ty, .. } => format!("text `{ty}`"),
        ResponseKind::Binary => "binary data".to_owned(),
    }
}

fn referenced_schemas<'a>(
    endpoints: &[&'a Endpoint],
    schemas: &'a TypeSchemas,
) -> BTreeSet<&'a str> {
    // Roots: the schemas responses, parameters and request bodies reference, plus named text.
    let mut names = BTreeSet::new();
    for endpoint in endpoints {
        if let Some(schema) = &endpoint.json_response_schema {
            collect_schema_refs(schema, &mut names);
        }
        if let ResponseKind::Text {
            ty: Ty::Ref(name), ..
        } = &endpoint.response_kind
        {
            names.insert(name.as_str());
        }
        for parameter in endpoint.path_params.iter().chain(&endpoint.query_params) {
            collect_schema_refs(&parameter.schema, &mut names);
        }
        if let Some(Ty::Ref(name)) = endpoint.request_body.as_ref().map(|body| &body.ty) {
            names.insert(name.as_str());
        }
    }
    names.retain(|name| schemas.contains_key(*name));

    let mut pending = names.iter().copied().collect::<Vec<_>>();
    while let Some(name) = pending.pop() {
        let mut refs = BTreeSet::new();
        collect_schema_refs(&schemas[name], &mut refs);
        for referenced in refs {
            if schemas.contains_key(referenced) && names.insert(referenced) {
                pending.push(referenced);
            }
        }
    }
    names
}

fn render_schema(output: &mut String, name: &str, schema: &Value) {
    writeln!(output, "### `{name}`\n").unwrap();
    let required = schema
        .get("required")
        .and_then(Value::as_array)
        .map(|values| {
            values
                .iter()
                .filter_map(Value::as_str)
                .collect::<BTreeSet<_>>()
        })
        .unwrap_or_default();
    if let Some(properties) = schema.get("properties").and_then(Value::as_object) {
        for (property, shape) in properties {
            write!(
                output,
                "- `{property}`: `{}`{}",
                Ty::parse(shape),
                if required.contains(property.as_str()) {
                    " (required)"
                } else {
                    ""
                }
            )
            .unwrap();
            if let Some(description) = shape
                .get("description")
                .and_then(Value::as_str)
                .filter(|value| !value.trim().is_empty())
            {
                write!(output, " — {}", one_line(description)).unwrap();
            }
            writeln!(output).unwrap();
        }
    } else {
        writeln!(output, "`{}`", Ty::parse(schema)).unwrap();
    }
    writeln!(output).unwrap();
}

fn one_line(value: &str) -> String {
    let mut result = String::new();
    for word in value.split_whitespace() {
        if !result.is_empty() {
            result.push(' ');
        }
        result.push_str(word);
    }
    result
}
