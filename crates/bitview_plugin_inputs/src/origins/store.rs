use brk_error::Result;
use brk_types::{BlockHash, Height, Sats, SupplyState, Version};
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
    pub fn start(&self) -> usize {
        self.store.start()
    }
    pub fn len(&self) -> usize {
        self.store.len()
    }
    pub fn is_empty(&self) -> bool {
        self.store.is_empty()
    }
    pub fn seed(&mut self, start: usize) -> Result<()> {
        self.store.seed(start)?;
        Ok(())
    }
    pub fn version(&self) -> Version {
        Version::ONE + Version::from(self.store.version() as u32)
    }
    pub fn total(&self, height: Height) -> Result<SupplyState> {
        let (_, v) = self
            .store
            .read(usize::from(height), &mut Vec::new(), &mut Vec::new())?;
        Ok(SupplyState {
            value: Sats::new(v.sats),
            utxo_count: v.count,
        })
    }
    pub(super) fn hash(&self, height: usize) -> Option<BlockHash> {
        self.store
            .hash(height)
            .ok()
            .and_then(|h| BlockHash::from_bytes(&h).ok())
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
    pub fn read(
        &self,
        height: Height,
        bytes: &mut Vec<u8>,
        spent: &mut Vec<(Height, SupplyState)>,
    ) -> Result<()> {
        let mut rows = Vec::new();
        self.store.read(usize::from(height), bytes, &mut rows)?;
        spent.clear();
        spent.extend(rows.into_iter().map(|(h, v)| {
            (
                Height::new(h),
                SupplyState {
                    value: Sats::new(v.sats),
                    utxo_count: v.count,
                },
            )
        }));
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
