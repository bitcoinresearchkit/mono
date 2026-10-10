use bitview_cohort::{AddressType, AddressTypeId};
use brk_types::{Height, OutputType};

use super::{MemberState, delta::ShardDelta};
use crate::addr::AddressVecs;

/// The address metrics' running state, one member per address type.
#[derive(Debug, Default, Clone)]
pub struct AddrMetricsState(pub AddressType<MemberState>);

impl AddrMetricsState {
    /// The counters at the start of `height`, from the members' stored series.
    pub fn restore(types: &AddressType<AddressVecs>, height: Height) -> Self {
        let Some(previous) = height.decremented() else {
            return Self::default();
        };
        Self(AddressType::from_fn(|id| {
            id.select(types)
                .restore(previous)
                .expect("address metrics stored through the resume height")
        }))
    }

    #[inline]
    pub fn reset_per_block(&mut self) {
        self.0.iter_mut().for_each(MemberState::reset_per_block);
    }

    /// The member `output_type`'s addresses update; `None` for P2A, not an address type.
    #[inline]
    pub fn member_mut(&mut self, output_type: OutputType) -> Option<&mut MemberState> {
        self.0.get_mut(output_type)
    }

    /// Adds the change to one member's values of a shard that started from `base`.
    pub fn add_member_delta(&mut self, shard: &Self, base: &Self, id: AddressTypeId) {
        id.select_mut(&mut self.0)
            .add_delta(*id.select(&shard.0), *id.select(&base.0));
    }

    /// Every address type together.
    pub fn all(&self) -> MemberState {
        let mut all = MemberState::default();
        for member in self.0.iter() {
            all += *member;
        }
        all
    }
}
