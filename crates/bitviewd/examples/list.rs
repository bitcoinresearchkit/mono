use std::{env, fs, path::Path};

use bitview::ImportContext;
use bitview_default::DefaultPlugins;
use bitview_query::Vecs;
use brk_reader::Reader;
use brk_rpc::{Auth, Client};
use vecdb::ReadOnlyClone;

pub fn main() -> brk_error::Result<()> {
    let tmp = env::temp_dir().join("brk_search_gen");
    fs::create_dir_all(&tmp)?;

    let client = Client::new("http://127.0.0.1:1", Auth::None)?;
    let reader = Reader::new_without_rlimit(tmp.join("blocks"), &client);
    let context = ImportContext::new(&tmp);
    let plugins = DefaultPlugins::import(context, &reader)?;

    let plugins_ro = plugins.read_only_clone();

    let vecs = Vecs::build(&plugins_ro);

    let out_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("series.txt");
    let content = vecs.series.join("\n");
    fs::write(&out_path, &content)?;
    eprintln!(
        "Wrote {} series to {}",
        vecs.series.len(),
        out_path.display()
    );

    fs::remove_dir_all(&tmp)?;

    Ok(())
}
