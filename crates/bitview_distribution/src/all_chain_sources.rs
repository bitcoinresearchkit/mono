use brk_types::{Cents, Height, Sats};
use vecdb::{ReadableBoxedVec, ReadableCloneableVec};

/// Shared views of all-chain inputs. Retention belongs to their stored sources.
#[derive(Clone)]
pub struct AllChainSources {
    pub supply: ReadableBoxedVec<Height, Sats>,
    pub market_cap: ReadableBoxedVec<Height, Cents>,
    pub realized_cap: ReadableBoxedVec<Height, Cents>,
}

impl AllChainSources {
    pub fn new(
        supply: &impl ReadableCloneableVec<Height, Sats>,
        market_cap: &impl ReadableCloneableVec<Height, Cents>,
        realized_cap: &impl ReadableCloneableVec<Height, Cents>,
    ) -> Self {
        Self {
            supply: supply.read_only_boxed_clone(),
            market_cap: market_cap.read_only_boxed_clone(),
            realized_cap: realized_cap.read_only_boxed_clone(),
        }
    }
}
