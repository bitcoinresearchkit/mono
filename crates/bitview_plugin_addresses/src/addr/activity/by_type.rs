use bitview_cohort::ByAddrType;
use derive_more::{Deref, DerefMut};

use super::BlockActivityCounts;

/// Activity counts accumulated during block processing for each address type.
#[derive(Debug, Default, Clone, Deref, DerefMut)]
pub struct AddrTypeToActivityCounts(pub ByAddrType<BlockActivityCounts>);

impl AddrTypeToActivityCounts {
    pub fn reset(&mut self) {
        self.0.values_mut().for_each(BlockActivityCounts::reset);
    }
}
