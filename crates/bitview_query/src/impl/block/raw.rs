use brk_error::{Error, OptionData};
use brk_types::{BlockHash, Height};
use vecdb::ReadableVec;

use crate::Query;

impl Query {
    pub fn block_raw(&self, hash: &BlockHash) -> brk_error::Result<Vec<u8>> {
        let height = self.height_by_hash(hash)?;
        self.block_raw_by_height(height)
    }

    fn block_raw_by_height(&self, height: Height) -> brk_error::Result<Vec<u8>> {
        let bound = self.safe_lengths().height;
        if height >= bound {
            return Err(Error::OutOfRange(
                format!("Block height {height} out of range (tip {})", self.height()).into(),
            ));
        }

        let indexer = self.indexer();
        let position = indexer.vecs().blocks.position.collect_one(height).data()?;
        let size = indexer.vecs().blocks.total.collect_one(height).data()?;

        self.reader().read_raw_bytes(position, *size as usize)
    }
}
