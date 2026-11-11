//! The Agent Skill: `SKILL.md`, the workflow and its traps in a few hundred words, and the
//! references it points to, read only when needed.

use std::{collections::BTreeSet, fmt::Write, io};

use bitview_catalog::TreeNode;
use bitview_primitives::{Calendar, Index};

use super::{ACCESS, BASE_URL, SCOPE, SERIES_FLOW, SERIES_RULE, SKILL, SKILLS_PATH, TRAPS, find};
use crate::{Endpoint, model::Model};

/// The skill's files, by path inside its folder: `SKILL.md` first.
pub(super) type Files = [(&'static str, String); 3];

const ENDPOINTS: &str = "references/endpoints.md";
const SERIES: &str = "references/series.md";
/// How many ids the topic map shows per branch.
const TOPIC_IDS: usize = 5;

/// The skill's trigger, in its frontmatter and the site's skill index.
pub(super) fn description() -> String {
    format!(
        "Fetches live and historical Bitcoin data from Bitview, read from a full node: {SCOPE}. Use when a question needs Bitcoin on-chain, market, mempool, fee, mining or holder-cohort data."
    )
}

pub(super) fn render(
    catalog: &TreeNode,
    model: &Model,
    endpoints: &[Endpoint],
    series_count: usize,
    flow: &str,
    terms: &str,
) -> io::Result<Files> {
    let offered = endpoints
        .iter()
        .filter(|endpoint| endpoint.should_generate())
        .collect::<Vec<_>>();
    Ok([
        ("SKILL.md", render_skill(&offered, series_count, flow, terms)?),
        (ENDPOINTS, render_endpoints(&offered)),
        (SERIES, render_series(catalog, model, series_count)),
    ])
}

fn render_skill(
    endpoints: &[&Endpoint],
    series_count: usize,
    flow: &str,
    terms: &str,
) -> io::Result<String> {
    let data = find(endpoints, "get_series")?;
    let bulk = find(endpoints, "get_series_bulk")?;
    let range = data
        .query_params
        .iter()
        .filter(|parameter| matches!(parameter.name.as_str(), "start" | "end"))
        .filter_map(|parameter| {
            Some(format!(
                "`{}`: {}",
                parameter.name,
                parameter.description.as_deref()?.trim()
            ))
        })
        .collect::<Vec<_>>()
        .join(" ");
    let flow_ids = SERIES_FLOW
        .iter()
        .flat_map(|(_, operations)| operations.iter().copied())
        .collect::<BTreeSet<_>>();
    let others = endpoints
        .iter()
        .copied()
        .filter(|endpoint| {
            endpoint.method == "GET" && !flow_ids.contains(endpoint.operation_id.as_str())
        })
        .collect::<Vec<_>>();
    let tree_paths = render_path_tree(&others);
    let indexes = render_indexes();
    let description = description();
    Ok(format!(
        "---\n\
name: {SKILL}\n\
description: \"{description}\"\n\
---\n\n\
# Bitview\n\n\
Bitcoin data API at {BASE_URL}, read from a full node. {ACCESS} Plain HTTPS GETs returning JSON. {terms}\n\n\
## Series\n\n\
{series_count} time series, each read at an index: `GET {path}` returns the values in `data` and their unit in `type`. {range}\n\n\
Ids are snake_case metric names: request a likely one directly, as an unknown id returns 404 with the nearest ids and a wrong index 400 with the valid ones. Otherwise:\n\n\
{flow}\n\
{SERIES_RULE} `{SERIES}` maps the series by topic, for browsing.\n\n\
Fetch and compute in one command, Python reading the JSON:\n\n\
```sh\n\
python3 -c 'import json, urllib.request as r; q = r.Request(\"{BASE_URL}{bulk}?series=<a>,<b>&index=day1&start=2025-01-01&end=2025-01-03\", headers={{\"User-Agent\": \"bitview-skill\"}}); a, b = (s[\"data\"] for s in json.load(r.urlopen(q))); print(a, b, b[-1] / a[-1])'\n\
```\n\n\
Indexes:\n\n\
{indexes}\n\
Traps:\n\n\
{TRAPS}\
- Answer in as few commands as you can: fetch and compute in one `python3 -c` as above, with `time.sleep(1)` between requests.\n\n\
## Other endpoints\n\n\
{tree_paths}\n\
What each returns and takes: `{ENDPOINTS}`. Files under `references/` are also at {BASE_URL}/{SKILLS_PATH}/{SKILL}/references/. The full reference with response schemas: {BASE_URL}/llms-full.txt.\n",
        path = data.path,
        bulk = bulk.path,
    ))
}

/// The indexes by kind: time buckets with their size, then positions.
fn render_indexes() -> String {
    let (mut buckets, mut positions) = (Vec::new(), Vec::new());
    for index in Index::all() {
        match index.calendar() {
            Some(calendar) => buckets.push(format!("`{}` ({})", index.name(), size(calendar))),
            None => positions.push(format!("`{}`", index.name())),
        }
    }
    format!(
        "- Time buckets, UTC, `start`/`end` as dates or timestamps: {}.\n\
- Positions (block height, epoch, transaction, input, output, or address of a type), `start`/`end` as integers: {}.\n",
        buckets.join(", "),
        positions.join(", "),
    )
}

/// A calendar's bucket length: `10 min`, `1 day`, `3 months`, `10 years`.
fn size(calendar: Calendar) -> String {
    let (count, unit) = match calendar {
        Calendar::Seconds(seconds) if seconds % 3600 == 0 => (seconds / 3600, "hour"),
        Calendar::Seconds(seconds) => (seconds / 60, "minute"),
        Calendar::Days { size, .. } if size % 7 == 0 => (size / 7, "week"),
        Calendar::Days { size, .. } => (size, "day"),
        Calendar::Months { size, .. } if size % 12 == 0 => (size / 12, "year"),
        Calendar::Months { size, .. } => (size, "month"),
    };
    format!("{count} {unit}{}", if count == 1 { "" } else { "s" })
}

/// Each path once, under the shortest listed path it extends: `/api/tx/{txid}[/hex|/status]`,
/// one line per first segment, in the document's order.
fn render_path_tree(endpoints: &[&Endpoint]) -> String {
    let mut groups = Vec::<(&str, Vec<(&str, Vec<&str>)>)>::new();
    for endpoint in endpoints {
        let group = super::endpoint_segment(&endpoint.path);
        if !groups.iter().any(|(name, _)| *name == group) {
            groups.push((group, Vec::new()));
        }
    }
    let mut paths = endpoints
        .iter()
        .map(|endpoint| endpoint.path.as_str())
        .collect::<Vec<_>>();
    paths.sort_by_key(|path| (path.len(), *path));
    paths.dedup();
    for path in paths {
        let group = super::endpoint_segment(path);
        let roots = &mut groups
            .iter_mut()
            .find(|(name, _)| *name == group)
            .expect("every path's group is listed")
            .1;
        match roots.iter_mut().find(|(root, _)| {
            path.strip_prefix(root)
                .is_some_and(|rest| rest.starts_with('/'))
        }) {
            Some((root, rest)) => rest.push(&path[root.len()..]),
            None => roots.push((path, Vec::new())),
        }
    }
    let mut output = String::new();
    for (_, roots) in groups {
        let line = roots
            .iter()
            .map(|(root, rest)| {
                if rest.is_empty() {
                    format!("`{root}`")
                } else {
                    format!("`{root}[{}]`", rest.join("|"))
                }
            })
            .collect::<Vec<_>>()
            .join(", ");
        writeln!(output, "- {line}").unwrap();
    }
    output
}

/// Every operation in a line: route, summary and query parameters.
fn render_endpoints(endpoints: &[&Endpoint]) -> String {
    let mut output = format!(
        "# Bitview endpoints\n\nBase URL: {BASE_URL}. Every operation, with its query parameters (`*` required). Response schemas: {BASE_URL}/llms-full.txt.\n\n"
    );
    for endpoint in endpoints {
        write!(output, "- `{} {}`", endpoint.method, endpoint.path).unwrap();
        if let Some(summary) = endpoint.summary.as_deref().filter(|s| !s.trim().is_empty()) {
            write!(output, " {}", summary.trim()).unwrap();
        }
        let query = endpoint
            .query_params
            .iter()
            .map(|parameter| {
                format!(
                    "`{}`{}",
                    parameter.name,
                    if parameter.required { "*" } else { "" }
                )
            })
            .collect::<Vec<_>>();
        if !query.is_empty() {
            write!(output, ". Query: {}", query.join(", ")).unwrap();
        }
        output.push('\n');
    }
    output
}

/// The topic map: every branch two levels down with its first ids, then the families, whose
/// members repeat one set of series under their own base.
fn render_series(catalog: &TreeNode, model: &Model, series_count: usize) -> String {
    let families = model.families();
    let members = families
        .iter()
        .flat_map(|family| {
            family.members.iter().skip(1).map(|(key, _)| {
                let mut path = family.path.clone();
                path.push(key.clone());
                path.join(".")
            })
        })
        .collect::<BTreeSet<_>>();
    let mut output = format!(
        "# Bitview series by topic\n\n{series_count} series. Find one with `GET {BASE_URL}/api/series/search?q=<plain words>`; this map is for browsing. Each branch lists its series count and first ids; a family's later members are left out of the branches, as they repeat the first one's series.\n\n## Branches\n"
    );
    let TreeNode::Branch(root) = catalog else {
        return output;
    };
    for (key, node) in root.iter() {
        let TreeNode::Branch(branch) = node else {
            continue;
        };
        writeln!(output, "\n### {key}\n").unwrap();
        for (child_key, child) in branch.iter() {
            let path = format!("{key}.{child_key}");
            if members.contains(&path) {
                continue;
            }
            let mut names = Vec::new();
            child.for_each_leaf(&mut |leaf| names.push(leaf.leaf.name.as_str()));
            let first = names.iter().take(TOPIC_IDS).map(|name| format!("`{name}`"));
            writeln!(
                output,
                "- `{path}` ({}): {}",
                names.len(),
                first.collect::<Vec<_>>().join(", ")
            )
            .unwrap();
        }
    }
    output.push_str("\n## Families\n\nEach member holds the family's series, its ids built on the member's base (`key → base`).\n\n");
    for family in &families {
        let members = family
            .members
            .iter()
            .map(|(key, base)| {
                if base.is_empty() {
                    format!("`{key}` → (none)")
                } else {
                    format!("`{key}` → `{base}`")
                }
            })
            .collect::<Vec<_>>()
            .join(", ");
        writeln!(
            output,
            "- `{}` ({} series each): {members}",
            family.path.join("."),
            family.series
        )
        .unwrap();
    }
    output
}
