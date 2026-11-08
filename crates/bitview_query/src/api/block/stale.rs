use std::{cmp::Reverse, collections::BTreeMap, sync::Arc};

use bitview_types::{BlockOrphan, BlockOrphanStatus};
use brk_types::Height;

use crate::{Error, Query, Result};

/// The stale blocks this server's node knows, by height. Only what the node saw itself: servers
/// can differ, a resynced one knows few, and a settled row's CDN copy keeps the list it was first
/// served with.
#[derive(Default)]
pub(crate) struct StaleBlocks(BTreeMap<Height, Vec<BlockOrphan>>);

impl StaleBlocks {
    pub(crate) fn at(&self, height: Height) -> &[BlockOrphan] {
        self.0.get(&height).map_or(&[], Vec::as_slice)
    }
}

impl Query {
    /// One read of the list, for a set of rows and their identity to share.
    pub(crate) fn stale_blocks(&self) -> Arc<StaleBlocks> {
        self.0.stale_blocks.load_full()
    }

    /// Reads the list anew from the node: every block of each fork it holds as valid.
    pub fn refresh_stale_blocks(&self) -> Result<()> {
        let client = self.client();
        let mut tips = client.get_chain_tips()?;
        // In no order of the node's: a block two forks share takes the longest's details, and a
        // height's blocks keep one order.
        tips.sort_unstable_by_key(|tip| (Reverse(tip.branchlen), tip.hash));
        let mut stale = BTreeMap::<Height, Vec<BlockOrphan>>::new();
        for tip in tips {
            let status = match tip.status.as_str() {
                "valid-fork" => BlockOrphanStatus::ValidFork,
                "valid-headers" => BlockOrphanStatus::ValidHeaders,
                _ => continue,
            };
            let mut hash = tip.hash;
            for depth in 0..tip.branchlen {
                let prevhash = client
                    .get_block_header_info(&hash)?
                    .previous_block_hash
                    .ok_or(Error::Internal("A stale block without a parent"))?;
                let height = Height::from(tip.height - depth);
                let at_height = stale.entry(height).or_default();
                // Two forks can share their first blocks.
                if at_height.iter().all(|block| block.hash != hash.into()) {
                    at_height.push(BlockOrphan {
                        height,
                        hash: hash.into(),
                        branchlen: tip.branchlen,
                        status,
                        prevhash: prevhash.into(),
                    });
                }
                hash = prevhash;
            }
        }
        self.0.stale_blocks.store(Arc::new(StaleBlocks(stale)));
        Ok(())
    }
}
