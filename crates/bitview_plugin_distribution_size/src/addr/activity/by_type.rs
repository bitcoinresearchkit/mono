use bitview_cohort::ByAddrType;
use derive_more::{Deref, DerefMut};

use super::BlockActivityCounts;

/// Activity counts accumulated during block processing for each address type.
#[derive(Debug, Default, Deref, DerefMut)]
pub struct AddrTypeToActivityCounts(pub ByAddrType<BlockActivityCounts>);

impl AddrTypeToActivityCounts {
    pub fn reset(&mut self) {
        self.0.values_mut().for_each(BlockActivityCounts::reset);
    }

    pub fn active(&self) -> u32 {
        self.0.values().map(BlockActivityCounts::active).sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn active_count_sums_distinct_addresses_across_types() {
        let counts = AddrTypeToActivityCounts(ByAddrType {
            p2pkh: BlockActivityCounts {
                sending: 5,
                receiving: 4,
                bidirectional: 2,
                ..BlockActivityCounts::default()
            },
            p2tr: BlockActivityCounts {
                sending: 3,
                receiving: 2,
                bidirectional: 1,
                ..BlockActivityCounts::default()
            },
            ..ByAddrType::default()
        });

        assert_eq!(counts.active(), 11);
    }
}
