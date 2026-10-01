use brk_error::Result;
use brk_types::{BlockHash, Height, SupplyState, Version};
use rustc_hash::FxHashMap;
use statedb::{Amount, Spends};
use std::path::Path;
use vecdb::Bytes;

/// Inputs owns spend production; statedb owns its files and encoding.
pub struct OriginSpends {
    store: Spends,
}
impl OriginSpends {
    pub fn spends(&self) -> &Spends {
        &self.store
    }

    pub(crate) fn import(path: &Path) -> Result<Self> {
        Ok(Self {
            store: Spends::open(path)?,
        })
    }
    pub(super) fn validate_sources(&mut self, version: Version) -> Result<()> {
        self.store.validate_version(u32::from(version).into())?;
        Ok(())
    }
    pub(crate) fn start(&self) -> usize {
        self.store.start()
    }
    pub(crate) fn len(&self) -> usize {
        self.store.len()
    }
    pub(super) fn hash(&self, height: usize) -> Result<BlockHash> {
        Ok(BlockHash::from_bytes(&self.store.hash(height)?)?)
    }
    pub(super) fn push(
        &mut self,
        height: Height,
        hash: BlockHash,
        spent: &FxHashMap<Height, SupplyState>,
    ) -> Result<()> {
        assert_eq!(usize::from(height), self.len());
        self.store.push(
            hash.to_bytes(),
            spent.iter().map(|(h, v)| {
                (
                    u32::from(*h),
                    Amount {
                        sats: u64::from(v.value),
                        count: v.utxo_count,
                    },
                )
            }),
        )?;
        Ok(())
    }
    pub(super) fn commit(&mut self) -> Result<()> {
        self.store.commit()?;
        Ok(())
    }
    pub(crate) fn truncate(&mut self, prefix: usize) -> Result<()> {
        self.store.truncate(prefix)?;
        Ok(())
    }
}
