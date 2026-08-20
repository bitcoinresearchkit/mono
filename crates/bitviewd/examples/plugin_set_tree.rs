use std::{env, fs, path::Path};

use bitview::ImportContext;
use bitview_default::DefaultPlugins;
use bitview_traversable::Traversable;
use brk_reader::Reader;
use brk_rpc::{Auth, Client};

pub fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;

    let tmp = env::temp_dir().join("bitview_tree_gen");
    fs::create_dir_all(&tmp)?;

    let client = Client::new("http://127.0.0.1:1", Auth::None)?;
    let reader = Reader::new_without_rlimit(tmp.join("blocks"), &client);
    let context = ImportContext::new(&tmp);
    let plugins = DefaultPlugins::import(context, &reader)?;
    let tree = plugins.to_tree_node();

    let json = serde_json::to_string_pretty(&tree)?;

    let out_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tree.json");
    fs::write(&out_path, &json)?;
    eprintln!("Wrote {} bytes to {}", json.len(), out_path.display());

    fs::remove_dir_all(&tmp)?;

    Ok(())
}
