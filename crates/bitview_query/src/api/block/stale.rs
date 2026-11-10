use std::{cmp::Reverse, collections::BTreeMap, sync::Arc};

use arc_swap::ArcSwapOption;
use bitview_types::{BlockOrphan, BlockOrphanStatus};
use brk_types::{ChainTip, Height};

use crate::{Error, Query, RepresentationId, Result};

/// The last complete Core tip response, encoded and identified before publication.
/// Absence means no successful refresh yet, never an observed empty chain.
#[derive(Default)]
pub(crate) struct ChainTips(ArcSwapOption<(Arc<[u8]>, RepresentationId)>);

impl ChainTips {
    pub(crate) fn publish(&self, tips: &[ChainTip]) -> Result<()> {
        // Core always includes its active tip. An empty response cannot replace known data.
        if tips.is_empty() {
            return Ok(());
        }
        let bytes = serde_json::to_vec(tips)?;
        let identity = RepresentationId::content(&bytes);
        self.0.store(Some(Arc::new((bytes.into(), identity))));
        Ok(())
    }

    pub(crate) fn json(&self) -> Result<(Arc<[u8]>, RepresentationId)> {
        self.0.load().as_deref().cloned().ok_or(Error::StateUpdating)
    }
}

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

    /// The last complete Core tip response. No RPC, chain pin or request-time encoding.
    pub fn chain_tips_json(&self) -> Result<(Arc<[u8]>, RepresentationId)> {
        self.0.chain_tips.json()
    }

    /// Publishes all node tips, then rebuilds the valid forks' orphan metadata.
    pub fn refresh_stale_blocks(&self) -> Result<()> {
        let client = self.client();
        let mut tips = client.get_chain_tips()?;
        // Preserve Core's complete response and order before the orphan-only sort/filter.
        // Header hydration can fail afterward without hiding this successful tip observation.
        self.0.chain_tips.publish(&tips)?;
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

#[cfg(test)]
mod tests {
    use super::*;

    fn identity(id: RepresentationId) -> u64 {
        match id {
            RepresentationId::Content(hash) => hash,
            RepresentationId::Block(_) => panic!("tips must identify their complete content"),
        }
    }

    #[test]
    fn chain_tip_observations_keep_all_records_and_their_identity() {
        let cache = ChainTips::default();
        assert!(matches!(cache.json(), Err(Error::StateUpdating)));
        cache.publish(&[]).unwrap();
        assert!(matches!(cache.json(), Err(Error::StateUpdating)));

        let mut tips: Vec<_> = [
            "headers-only", "active", "valid-fork", "valid-headers", "invalid", "unknown",
            "future-status",
        ]
        .into_iter()
        .enumerate()
        .map(|(i, status)| ChainTip {
            height: 100 - i as u32 / 2,
            hash: format!("{:064x}", i + 1).parse().unwrap(),
            branchlen: (i % 3) as u32,
            status: status.into(),
        })
        .collect();
        let _: &brk_rpc::GetChainTip = &tips[0];
        cache.publish(&tips).unwrap();
        let (old_bytes, old_id) = cache.json().unwrap();
        assert_eq!(serde_json::from_slice::<Vec<ChainTip>>(&old_bytes).unwrap(), tips);
        assert_eq!(identity(old_id), RepresentationId::content_hash(&old_bytes));

        cache.publish(&[]).unwrap();
        let (same_bytes, same_id) = cache.json().unwrap();
        assert!(Arc::ptr_eq(&old_bytes, &same_bytes));
        assert_eq!(identity(old_id), identity(same_id));

        tips[2].status = "invalid".into();
        cache.publish(&tips).unwrap();
        let (changed, changed_id) = cache.json().unwrap();
        assert_ne!(identity(old_id), identity(changed_id));
        assert_ne!(&*old_bytes, &*changed);
        assert_eq!(serde_json::from_slice::<Vec<ChainTip>>(&changed).unwrap(), tips);
        assert_eq!(identity(old_id), RepresentationId::content_hash(&old_bytes));
        tips.swap(0, 1);
        cache.publish(&tips).unwrap();
        let (reordered, reordered_id) = cache.json().unwrap();
        assert_ne!(identity(changed_id), identity(reordered_id));
        assert_eq!(serde_json::from_slice::<Vec<ChainTip>>(&reordered).unwrap(), tips);
    }
}
