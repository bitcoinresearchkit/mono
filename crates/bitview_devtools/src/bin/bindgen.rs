use std::{
    env, fs,
    path::{Path, PathBuf},
    process::Command,
};

use aide::axum::ApiRouter;
use bitview_bindgen::{ClientOutputPaths, generate_clients};
use bitview_default::DefaultPlugins;
use bitview_plugin::ImportContext;
use bitview_query::Vecs;
use bitview_server::{ApiRoutes, finish_openapi};
use brk_exit::Exit;
use brk_reader::Reader;
use brk_rpc::{Auth, Client};
use color_eyre::{
    eyre::{Result, bail},
    install,
};
use serde_json::{json, to_string, to_string_pretty};
use tempfile::tempdir;
use vecdb::Budgeted;

#[cfg(unix)]
use std::os::unix::process::CommandExt;
#[cfg(not(unix))]
use std::process::exit;

const GENERATED_OUTPUTS: &[(&str, &str)] = &[
    (
        "crates/bitview_mcp/server.json",
        "crates/bitview_mcp/server.json",
    ),
    (
        "crates/bitview_client/src/generated.rs",
        "crates/bitview_client/src/generated.rs",
    ),
    (
        "crates/bitview_cli/src/generated.rs",
        "crates/bitview_cli/src/generated.rs",
    ),
    (
        "modules/bitview-client/index.js",
        "modules/bitview-client/index.js",
    ),
    (
        "packages/bitview_client/bitview_client/__init__.py",
        "packages/bitview_client/bitview_client/__init__.py",
    ),
    ("website/llms.txt", "website/llms.txt"),
    ("website/llms-full.txt", "website/llms-full.txt"),
    (
        "crates/bitview_mcp/generated/manifest.json",
        "crates/bitview_mcp/generated/manifest.json",
    ),
];

#[derive(Clone, Copy)]
enum OutputScope {
    All,
    Rust,
}

impl OutputScope {
    fn includes(self, path: &str) -> bool {
        matches!(self, Self::All) || path.ends_with(".rs")
    }
}

fn main() -> Result<()> {
    Budgeted::init_global(bitview_plugin::DEFAULT_CACHE_BUDGET)?;
    install()?;

    let args = env::args().skip(1).collect::<Vec<_>>();
    match args.as_slice() {
        [] => generate(false, OutputScope::All),
        [arg] if arg == "--check" => generate(true, OutputScope::All),
        [arg] if arg == "--rust" => generate(false, OutputScope::Rust),
        [scope, check] if scope == "--rust" && check == "--check" => {
            generate(true, OutputScope::Rust)
        }
        [arg, daemon_args @ ..] if arg == "--run" => {
            // Drop generation's plugin state before starting the daemon.
            generate(false, OutputScope::All)?;
            let daemon_args = daemon_args
                .strip_prefix(&["--".to_owned()])
                .unwrap_or(daemon_args);
            let mut command = Command::new(env::var_os("CARGO").unwrap_or_else(|| "cargo".into()));
            command
                .args(["run", "-p", "bitviewd", "--bin", "bitviewd", "--"])
                .args(daemon_args);
            #[cfg(unix)]
            {
                Err(command.exec().into())
            }
            #[cfg(not(unix))]
            {
                exit(command.status()?.code().unwrap_or(1));
            }
        }
        _ => {
            bail!(
                "usage: cargo bindgen [-- --check | -- --rust [--check] | -- --run [-- daemon arguments]]"
            )
        }
    }
}

fn generate(check: bool, scope: OutputScope) -> Result<()> {
    let temporary = tempdir()?;
    let tmp = temporary.path();

    let client = Client::new("http://127.0.0.1:1", Auth::None)?;
    let reader = Reader::new_without_rlimit(tmp.join("blocks"), &client);
    let exit = Exit::new();
    let context = ImportContext::new(tmp, &exit);
    let plugins = DefaultPlugins::import(context, &reader)?;
    let vecs = Vecs::build(&plugins);

    let (_, openapi) = finish_openapi(ApiRouter::new().add_api_routes());

    let workspace_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .unwrap()
        .to_path_buf();

    let output_root = if check {
        tmp.join("generated")
    } else {
        workspace_root.clone()
    };
    let output_paths = output_paths(&output_root, scope);

    generate_clients(vecs.catalog(), &to_string(&openapi)?, &output_paths)?;
    if matches!(scope, OutputScope::All) {
        generate_registry_manifest(&output_root)?;
    }

    let result = if check {
        verify_outputs(&output_root, &workspace_root, scope)
    } else {
        Ok(())
    };

    result?;

    eprintln!(
        "{}",
        if check {
            "Generated outputs are current"
        } else {
            "Done"
        }
    );

    Ok(())
}

fn output_paths(root: &Path, scope: OutputScope) -> ClientOutputPaths {
    let paths = ClientOutputPaths::new()
        .rust(root.join("crates/bitview_client/src/generated.rs"))
        .cli(root.join("crates/bitview_cli/src/generated.rs"));
    if matches!(scope, OutputScope::Rust) {
        return paths;
    }
    paths
        .javascript(root.join("modules/bitview-client/index.js"))
        .python(root.join("packages/bitview_client/bitview_client/__init__.py"))
        .llm(root.join("website"))
        .llm_manifest(root.join("crates/bitview_mcp/generated/manifest.json"))
}

fn generate_registry_manifest(root: &Path) -> Result<()> {
    let path = root.join("crates/bitview_mcp/server.json");
    fs::create_dir_all(path.parent().unwrap())?;
    let mut contents = to_string_pretty(&json!({
        "$schema": "https://static.modelcontextprotocol.io/schemas/2025-12-11/server.schema.json",
        "name": "io.github.bitcoinresearchkit/bitview",
        "title": "Bitview",
        "description": "Read-only Bitcoin blockchain, mempool, mining, market, and on-chain analytics; no API key.",
        "version": env!("CARGO_PKG_VERSION"),
        "websiteUrl": "https://mcp.bitview.space/",
        "icons": [{
            "src": "https://mcp.bitview.space/logo.png",
            "mimeType": "image/png",
            "sizes": ["512x512"]
        }],
        "repository": {
            "url": env!("CARGO_PKG_REPOSITORY"),
            "source": "github",
            "id": "824866280",
            "subfolder": "crates/bitview_mcp"
        },
        "remotes": [{
            "type": "streamable-http",
            "url": "https://mcp.bitview.space/"
        }]
    }))?;
    contents.push('\n');

    if fs::read_to_string(&path).ok().as_deref() != Some(&contents) {
        fs::write(path, contents)?;
    }

    Ok(())
}

fn verify_outputs(generated_root: &Path, workspace_root: &Path, scope: OutputScope) -> Result<()> {
    let outputs: Vec<_> = GENERATED_OUTPUTS
        .iter()
        .copied()
        .filter(|(path, _)| scope.includes(path))
        .collect();
    verify_output_pairs(generated_root, workspace_root, &outputs)
}

fn verify_output_pairs(
    generated_root: &Path,
    workspace_root: &Path,
    outputs: &[(&str, &str)],
) -> Result<()> {
    let mut stale = Vec::new();
    for (generated, committed) in outputs {
        let generated = fs::read(generated_root.join(generated));
        let committed_bytes = fs::read(workspace_root.join(committed));
        if !matches!((generated, committed_bytes), (Ok(left), Ok(right)) if left == right) {
            stale.push(*committed);
        }
    }
    if !stale.is_empty() {
        bail!(
            "generated outputs are stale:\n{}\nrun `cargo bindgen`",
            stale.join("\n")
        );
    }
    Ok(())
}
