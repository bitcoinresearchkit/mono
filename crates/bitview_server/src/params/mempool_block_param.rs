use schemars::JsonSchema;
use serde::Deserialize;

use bitview_query::{Error, Result};
use brk_types::{MempoolBlockHash, MempoolBlockIndex};

/// Ok for a block served with its transactions, 404 otherwise: only the next one for now.
pub fn served(index: MempoolBlockIndex) -> Result<()> {
    if index == MempoolBlockIndex::NEXT {
        Ok(())
    } else {
        Err(Error::NotFound(format!(
            "Mempool block {index} isn't served with its transactions: only block 0 is"
        )))
    }
}

#[derive(Deserialize, JsonSchema)]
pub struct MempoolBlockParam {
    pub index: MempoolBlockIndex,
}

/// The block and the content token a client holds for it.
#[derive(Deserialize, JsonSchema)]
pub struct MempoolBlockDiffParam {
    pub index: MempoolBlockIndex,
    /// The `hash` of the client's copy, from the block or a previous diff
    pub since: MempoolBlockHash,
}
