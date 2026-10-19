use bitview_cohort::{AddressType, AddressTypeId};
use brk_types::Height;

use super::{MemberState, delta::ShardDelta};
use crate::addr::AddressVecs;

#[derive(Default, Clone)]
pub struct AddrMetricsState(pub AddressType<MemberState>);

impl AddrMetricsState {
    /// The counters at the start of `height`, from the members' stored series; `None` without
    /// the block before.
    pub fn restore(types: &AddressType<AddressVecs>, height: Height) -> Option<Self> {
        let Some(previous) = height.decremented() else {
            return Some(Self::default());
        };
        AddressType::try_from_fn(|id| id.select(types).restore(previous).ok_or(()))
            .ok()
            .map(Self)
    }

    #[inline]
    pub fn reset_per_block(&mut self) {
        self.0.iter_mut().for_each(MemberState::reset_per_block);
    }

    /// Adds the change of member `id` in a shard that started from `base`.
    pub fn add_member_delta(&mut self, id: AddressTypeId, shard: MemberState, base: &Self) {
        id.select_mut(&mut self.0)
            .add_delta(shard, *id.select(&base.0));
    }

    pub fn all(&self) -> MemberState {
        let mut all = MemberState::default();
        for member in self.0.iter() {
            all += *member;
        }
        all
    }
}
