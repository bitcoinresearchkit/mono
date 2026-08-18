use std::{
    env, fs,
    path::{Path, PathBuf},
};

use aide::axum::ApiRouter;
use brk_computer::Computer;
use brk_indexer::Indexer;
use brk_query::Vecs;
use brk_reader::Reader;
use brk_rpc::{Auth, Client};
use brk_server::{ApiRoutes, finish_openapi, generate_bindings};
use color_eyre::eyre::{Result, bail};

const GENERATED_OUTPUTS: &[(&str, &str)] = &[
    ("server.json", "server.json"),
    (
        "crates/brk_client/src/lib.rs",
        "crates/brk_client/src/lib.rs",
    ),
    ("modules/brk-client/index.js", "modules/brk-client/index.js"),
    (
        "modules/brk-client/index.js",
        "website/scripts/modules/brk-client/index.js",
    ),
    (
        "packages/brk_client/brk_client/__init__.py",
        "packages/brk_client/brk_client/__init__.py",
    ),
    ("website/llms.txt", "website/llms.txt"),
    ("website/llms-full.txt", "website/llms-full.txt"),
    ("website_next/llms.txt", "website_next/llms.txt"),
    ("website_next/llms-full.txt", "website_next/llms-full.txt"),
    (
        "crates/brk_mcp/generated/manifest.json",
        "crates/brk_mcp/generated/manifest.json",
    ),
];

pub fn main() -> Result<()> {
    color_eyre::install()?;

    let args = env::args().skip(1).collect::<Vec<_>>();
    let check = match args.as_slice() {
        [] => false,
        [arg] if arg == "--check" => true,
        _ => bail!(
            "usage: cargo run -p brk_server --example bindgen --features bindgen [-- --check]"
        ),
    };

    let tmp = env::temp_dir().join(format!("brk_bindgen_{}", std::process::id()));
    if tmp.exists() {
        fs::remove_dir_all(&tmp)?;
    }
    fs::create_dir_all(&tmp)?;

    let client = Client::new("http://127.0.0.1:1", Auth::None)?;
    let reader = Reader::new_without_rlimit(tmp.join("blocks"), &client);
    let indexer = Indexer::import(&tmp, &reader)?;
    let computer = Computer::forced_import(&tmp, &indexer)?;
    let vecs = Vecs::build_rw(&indexer, &computer);

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
    let output_paths = output_paths(&output_root);

    generate_bindings(&vecs, &openapi, &output_paths)?;
    generate_registry_manifest(&output_root)?;

    let result = if check {
        verify_outputs(&output_root, &workspace_root)
    } else {
        mirror_javascript_client(&workspace_root)
    };

    fs::remove_dir_all(&tmp)?;
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

fn output_paths(root: &Path) -> brk_bindgen::ClientOutputPaths {
    brk_bindgen::ClientOutputPaths::new()
        .rust(root.join("crates/brk_client/src/lib.rs"))
        .javascript(root.join("modules/brk-client/index.js"))
        .python(root.join("packages/brk_client/brk_client/__init__.py"))
        .llm(root.join("website"))
        .llm(root.join("website_next"))
        .llm_manifest(root.join("crates/brk_mcp/generated/manifest.json"))
}

fn generate_registry_manifest(root: &Path) -> Result<()> {
    let path = root.join("server.json");
    let mut contents = serde_json::to_string_pretty(&serde_json::json!({
        "$schema": "https://static.modelcontextprotocol.io/schemas/2025-12-11/server.schema.json",
        "name": "io.github.bitcoinresearchkit/bitview",
        "title": "Bitview",
        "description": "Read-only Bitcoin blockchain, mempool, market, and on-chain analytics.",
        "version": env!("CARGO_PKG_VERSION"),
        "websiteUrl": "https://mcp.bitview.space/",
        "icons": [{
            "src": "https://mcp.bitview.space/logo.png",
            "mimeType": "image/png",
            "sizes": ["512x512"]
        }],
        "repository": {
            "url": "https://github.com/bitcoinresearchkit/brk",
            "source": "github"
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

fn mirror_javascript_client(root: &Path) -> Result<()> {
    let source = root.join("modules/brk-client/index.js");
    let destination = root.join("website/scripts/modules/brk-client/index.js");
    let generated = fs::read(&source)?;
    if fs::read(&destination).ok().as_deref() != Some(generated.as_slice()) {
        fs::write(destination, generated)?;
    }
    Ok(())
}

fn verify_outputs(generated_root: &Path, workspace_root: &Path) -> Result<()> {
    verify_output_pairs(generated_root, workspace_root, GENERATED_OUTPUTS)
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
            "generated outputs are stale:\n{}\nrun the bindgen example without --check",
            stale.join("\n")
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_reports_stale_outputs() {
        let root = env::temp_dir().join(format!("brk_bindgen_check_test_{}", std::process::id()));
        let generated = root.join("generated");
        let workspace = root.join("workspace");
        fs::create_dir_all(&generated).unwrap();
        fs::create_dir_all(&workspace).unwrap();
        fs::write(generated.join("artifact"), "new").unwrap();
        fs::write(workspace.join("artifact"), "old").unwrap();

        let error =
            verify_output_pairs(&generated, &workspace, &[("artifact", "artifact")]).unwrap_err();

        assert!(error.to_string().contains("artifact"));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn registry_manifest_uses_package_version() {
        let root =
            env::temp_dir().join(format!("brk_registry_manifest_test_{}", std::process::id()));
        fs::create_dir_all(&root).unwrap();

        generate_registry_manifest(&root).unwrap();
        let manifest: serde_json::Value =
            serde_json::from_slice(&fs::read(root.join("server.json")).unwrap()).unwrap();

        assert_eq!(manifest["version"], env!("CARGO_PKG_VERSION"));
        assert_eq!(manifest["name"], "io.github.bitcoinresearchkit/bitview");
        assert_eq!(manifest["remotes"][0]["type"], "streamable-http");
        assert_eq!(manifest["remotes"][0]["url"], "https://mcp.bitview.space/");
        fs::remove_dir_all(root).unwrap();
    }
}
