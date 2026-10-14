use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};

use brk_error::{Error, Result};
use derive_more::Deref;

const BLK: &str = "blk";
const DOT_DAT: &str = ".dat";

#[derive(Debug, Default, Clone, Deref)]
pub struct BlkIndexToBlkPath(BTreeMap<u16, PathBuf>);

impl BlkIndexToBlkPath {
    /// Collects every `blkNNNNN.dat` in `blocks_dir`. Unrelated
    /// entries (`xor.dat`, `rev*.dat`, `index/`, …) are skipped
    /// silently; anything that **looks** like a blk file but fails to
    /// parse or isn't a regular file is a hard error, since silently
    /// dropping one would leave an undetectable hole in the chain.
    pub(crate) fn scan(blocks_dir: &Path) -> Result<Self> {
        let mut map = BTreeMap::new();

        for entry in fs::read_dir(blocks_dir)? {
            let entry = entry?;
            let file_name = entry.file_name();
            let Some(file_name) = file_name.to_str() else {
                continue;
            };
            let Some(index_str) = file_name
                .strip_prefix(BLK)
                .and_then(|s| s.strip_suffix(DOT_DAT))
            else {
                continue;
            };

            let blk_index = index_str
                .parse::<u16>()
                .map_err(|_| Error::Parse(format!("Malformed blk file name: {file_name}")))?;

            let path = entry.path();
            // The entry's own type needs no extra metadata call; only a
            // symlink is followed to its target.
            let file_type = entry.file_type()?;
            if !(file_type.is_file() || file_type.is_symlink() && path.is_file()) {
                return Err(Error::Parse(format!(
                    "blk entry is not a regular file: {}",
                    path.display()
                )));
            }

            map.insert(blk_index, path);
        }

        Ok(Self(map))
    }
}
