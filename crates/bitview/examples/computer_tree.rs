use std::{env, fs, path::Path};

use bitview::Computer;
use bitview_traversable::{Traversable, TreeNode};
use brk_indexer::Indexer;
use brk_reader::Reader;
use brk_rpc::{Auth, Client};

pub fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;

    let tmp = env::temp_dir().join("bitview_tree_gen");
    fs::create_dir_all(&tmp)?;

    let client = Client::new("http://127.0.0.1:1", Auth::None)?;
    let reader = Reader::new_without_rlimit(tmp.join("blocks"), &client);
    let indexer = Indexer::import(&tmp, &reader)?;
    let computer = Computer::forced_import(&tmp, &indexer)?;

    let tree = TreeNode::Branch(
        [
            ("indexed".to_string(), indexer.vecs().to_tree_node()),
            ("computed".to_string(), computer.to_tree_node()),
        ]
        .into_iter()
        .collect(),
    )
    .merge_branches()
    .expect("Tree merge failed");

    let json = serde_json::to_string_pretty(&tree)?;

    let out_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tree.json");
    fs::write(&out_path, &json)?;
    eprintln!("Wrote {} bytes to {}", json.len(), out_path.display());

    fs::remove_dir_all(&tmp)?;

    Ok(())
}
