//! Offline snapshots of Bitview's API and storage surface, used as refactoring gates.
//!
//! The tools import every plugin (default composition plus the optional ones) into a
//! temporary directory without a node and render deterministic text committed under
//! `snapshots/`. Without arguments they rewrite the snapshots; `--check` compares instead.
//! Versions are those of a fresh import: dependency versions applied by computation are
//! not included, so a changed version formula must be reviewed by hand.

use std::{
    collections::{BTreeMap, BTreeSet},
    env, fs,
    path::{Path, PathBuf},
    process::ExitCode,
};

use aide::axum::ApiRouter;
use bitview_catalog::{TreeNode, extract_json_type};
use bitview_default::DefaultPlugins;
use bitview_plugin::ImportContext;
use bitview_plugin_blocks::HasBlocks;
use bitview_plugin_distribution_aggregated::HasDistributionAggregated;
use bitview_plugin_distribution_entry::Vecs as DistributionEntry;
use bitview_plugin_distribution_profitability::Vecs as DistributionProfitability;
use bitview_plugin_mappings::HasMappings;
use bitview_plugin_price::HasPrice;
use bitview_query::Vecs;
use bitview_runtime::PluginSet;
use bitview_server::{ApiRoutes, finish_openapi};
use bitview_traversable::Traversable;
use brk_error::Result;
use brk_reader::Reader;
use brk_rpc::{Auth, Client};
use tempfile::{TempDir, tempdir};
use vecdb::{Budgeted, Database, Header, ReadableCloneableVec, Rw, StorageMode};

/// Every plugin in the repository, composed as the optional-plugin examples do.
#[derive(PluginSet, Traversable)]
pub struct AllPlugins<M: StorageMode = Rw> {
    #[traversable(flatten)]
    #[plugin_set(flatten)]
    pub defaults: DefaultPlugins<M>,
    pub distribution_entry: DistributionEntry<M>,
    #[traversable(flatten)]
    pub distribution_profitability: DistributionProfitability<M>,
}

/// A fresh offline import; the directory lives as long as this value.
pub struct Imported {
    pub plugins: AllPlugins,
    pub dir: TempDir,
}

pub fn import() -> Result<Imported> {
    Budgeted::init_global(2 * 1024 * 1024 * 1024)?;
    let dir = tempdir()?;
    let client = Client::new("http://127.0.0.1:1", Auth::None)?;
    let reader = Reader::new_without_rlimit(dir.path().join("blocks"), &client);
    let context = ImportContext::new(dir.path());

    let defaults = DefaultPlugins::import(context, &reader)?;
    let window_starts = defaults.blocks().lookback.window_starts();
    let prices = defaults.price().spot.cents.height.read_only_boxed_clone();
    let distribution_entry = DistributionEntry::import(
        context,
        defaults.mappings(),
        &window_starts,
        &prices,
        defaults.distribution_aggregated().all_supply(),
    )?;
    let distribution_profitability =
        DistributionProfitability::import(context, defaults.mappings(), &window_starts, &prices)?;

    Ok(Imported {
        plugins: AllPlugins {
            defaults,
            distribution_entry,
            distribution_profitability,
        },
        dir,
    })
}

/// One rendered snapshot file, relative to `snapshots/`.
pub struct Snapshot {
    pub file: &'static str,
    pub contents: String,
}

/// Writes the snapshots, or with `--check` reports every stale one.
pub fn run(tool: &str, render: impl FnOnce() -> Result<Vec<Snapshot>>) -> ExitCode {
    let check = match env::args().skip(1).collect::<Vec<_>>().as_slice() {
        [] => false,
        [arg] if arg == "--check" => true,
        _ => {
            eprintln!("usage: cargo {tool} [-- --check]");
            return ExitCode::FAILURE;
        }
    };
    let snapshots = match render() {
        Ok(snapshots) => snapshots,
        Err(error) => {
            eprintln!("error: {error}");
            return ExitCode::FAILURE;
        }
    };

    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("snapshots");
    let mut stale = false;
    for snapshot in &snapshots {
        let path = dir.join(snapshot.file);
        let committed = fs::read_to_string(&path).unwrap_or_default();
        if committed == snapshot.contents {
            continue;
        }
        if check {
            stale = true;
            report_diff(&path, &committed, &snapshot.contents);
        } else if let Err(error) =
            fs::create_dir_all(&dir).and_then(|_| fs::write(&path, &snapshot.contents))
        {
            eprintln!("error: writing {}: {error}", path.display());
            return ExitCode::FAILURE;
        } else {
            eprintln!("updated {}", path.display());
        }
    }

    if stale {
        eprintln!("snapshots are stale: review the changes, then run `cargo {tool}`");
        ExitCode::FAILURE
    } else {
        eprintln!("{tool} snapshots are current");
        ExitCode::SUCCESS
    }
}

fn report_diff(path: &Path, committed: &str, current: &str) {
    let committed = committed.lines().collect::<BTreeSet<_>>();
    let current = current.lines().collect::<BTreeSet<_>>();
    let removed = committed.difference(&current).collect::<Vec<_>>();
    let added = current.difference(&committed).collect::<Vec<_>>();
    eprintln!(
        "{}: -{} +{} lines",
        path.display(),
        removed.len(),
        added.len()
    );
    for line in removed.iter().take(20) {
        eprintln!("  - {line}");
    }
    for line in added.iter().take(20) {
        eprintln!("  + {line}");
    }
}

fn header(tool: &str) -> String {
    format!("# Generated by `cargo {tool}` (crates/bitview_devtools). Do not edit.\n")
}

/// The series tree as clients see it: one line per leaf, then descriptions and value schemas.
pub fn render_api(plugins: &AllPlugins) -> Result<Vec<Snapshot>> {
    let defaults = Vecs::build(&plugins.defaults);
    let all = Vecs::build(plugins);

    let mut leaves = Leaves::default();
    let default_lines = leaves.lines(defaults.catalog());
    let all_lines = leaves.lines(all.catalog());
    let default_set = default_lines.iter().collect::<BTreeSet<_>>();
    let all_set = all_lines.iter().collect::<BTreeSet<_>>();

    let mut out = header("api");
    out.push_str("\n## Default composition (generated clients): path, series, value type, JSON type, indexes\n");
    for line in &default_lines {
        out.push_str(line);
        out.push('\n');
    }
    out.push_str("\n## Added by optional plugins\n");
    for line in all_lines.iter().filter(|line| !default_set.contains(line)) {
        out.push_str(line);
        out.push('\n');
    }
    let missing = default_lines.iter().filter(|line| !all_set.contains(line));
    out.push_str("\n## Default leaves missing from the full composition\n");
    for line in missing {
        out.push_str(line);
        out.push('\n');
    }
    out.push_str("\n## Descriptions\n");
    for (series, description) in &leaves.descriptions {
        out.push_str(&format!("{series}\t{description}\n"));
    }
    out.push_str("\n## Value schemas\n");
    for (kind, schema) in &leaves.schemas {
        out.push_str(&format!("{kind}\t{schema}\n"));
    }

    let (_, openapi) = finish_openapi(ApiRouter::new().add_api_routes());
    let mut openapi = serde_json::to_string_pretty(&openapi)?;
    openapi.push('\n');

    Ok(vec![
        Snapshot {
            file: "api.txt",
            contents: out,
        },
        Snapshot {
            file: "openapi.json",
            contents: openapi,
        },
    ])
}

#[derive(Default)]
struct Leaves {
    descriptions: BTreeMap<String, String>,
    schemas: BTreeSet<(String, String)>,
}

impl Leaves {
    fn lines(&mut self, catalog: &TreeNode) -> Vec<String> {
        let mut lines = Vec::new();
        self.walk(catalog, &mut String::new(), &mut lines);
        lines
    }

    fn walk(&mut self, node: &TreeNode, path: &mut String, lines: &mut Vec<String>) {
        match node {
            TreeNode::Branch(branch) => {
                for (key, child) in branch.iter() {
                    let len = path.len();
                    if !path.is_empty() {
                        path.push('.');
                    }
                    path.push_str(key);
                    self.walk(child, path, lines);
                    path.truncate(len);
                }
            }
            TreeNode::Leaf(leaf) => {
                let indexes = leaf
                    .indexes()
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
                    .join(",");
                lines.push(format!(
                    "{path}\t{}\t{}\t{}\t{indexes}",
                    leaf.name(),
                    leaf.kind(),
                    extract_json_type(&leaf.schema),
                ));
                if let Some(description) = &leaf.leaf.description {
                    self.descriptions
                        .insert(leaf.name().to_owned(), description.to_string());
                }
                self.schemas
                    .insert((leaf.kind().to_owned(), leaf.schema.to_string()));
            }
        }
    }
}

/// Plugin ids and root versions, every vector (hidden included), and what a fresh
/// import leaves on disk: rawdb regions with their header versions, and every other file.
pub fn render_surface(imported: Imported) -> Result<Vec<Snapshot>> {
    let Imported { plugins, dir } = imported;

    let mut out = header("surface");
    out.push_str("\n## Plugins: id, root version\n");
    let mut vecs = BTreeMap::<String, BTreeSet<String>>::new();
    plugins.for_each_plugin(&mut |plugin| {
        out.push_str(&format!(
            "{}\t{}\n",
            plugin.id(),
            plugin.storage().schema_version()
        ));
        let lines = vecs.entry(plugin.id().to_string()).or_default();
        plugin.for_each_exportable(&mut |vec| {
            lines.insert(format!(
                "{}\t{}\t{}\t{}\t{}",
                vec.name(),
                vec.index_type_to_string(),
                vec.value_type_to_string(),
                vec.version(),
                vec.region_names().join(","),
            ));
        });
    });
    for (plugin, lines) in &vecs {
        out.push_str(&format!(
            "\n## Vectors of {plugin}: name, index, value type, version, regions\n"
        ));
        for line in lines {
            out.push_str(line);
            out.push('\n');
        }
    }

    // Release every database before reopening them from disk.
    drop(plugins);
    out.push_str(
        "\n## Disk: rawdb regions (header, vec, computed version, format) and other files\n",
    );
    let root = dir.path().join("plugins");
    walk_disk(&root, &root, &mut out)?;

    Ok(vec![Snapshot {
        file: "surface.txt",
        contents: out,
    }])
}

fn walk_disk(root: &Path, dir: &Path, out: &mut String) -> Result<()> {
    let relative = |path: &Path| {
        path.strip_prefix(root)
            .unwrap_or(path)
            .display()
            .to_string()
    };
    let is_db = dir.join("data").is_file() && dir.join("regions").is_file();
    if is_db {
        let db = Database::open(dir)?;
        for id in db.region_ids() {
            let header = db
                .get_region(&id)
                .and_then(|region| Header::read(&region).ok())
                .map(|header| {
                    format!(
                        "h{} v{} c{} {:?}",
                        header.header_version(),
                        header.vec_version(),
                        header.computed_version(),
                        header.format()
                    )
                })
                .unwrap_or_else(|| "-".to_owned());
            out.push_str(&format!("{}/{id}\t{header}\n", relative(dir)));
        }
    }

    let mut entries = fs::read_dir(dir)?
        .map(|entry| entry.map(|entry| entry.path()))
        .collect::<std::io::Result<Vec<PathBuf>>>()?;
    entries.sort();
    for path in entries {
        let name = path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or_default();
        if is_db && (name == "data" || name == "regions") {
            continue;
        }
        if path.is_dir() {
            walk_disk(root, &path, out)?;
            continue;
        }
        // Tiny files are version and marker files: their bytes are the surface.
        let bytes = fs::read(&path)?;
        let contents = if bytes.len() <= 16 {
            bytes
                .iter()
                .map(|byte| format!("{byte:02x}"))
                .collect::<String>()
        } else {
            format!("{} bytes", bytes.len())
        };
        out.push_str(&format!("{}\tfile {contents}\n", relative(&path)));
    }
    Ok(())
}
