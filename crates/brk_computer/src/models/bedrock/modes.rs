use brk_traversable::Traversable;
use derive_more::{Deref, DerefMut};

use super::{ModeId, WeightedModes};

#[derive(Deref, DerefMut, Traversable)]
pub struct Modes<T> {
    /// Uses the unweighted all-chain URPD and raw all-chain supply-in-loss share.
    pub raw: T,
    #[deref]
    #[deref_mut]
    #[traversable(flatten)]
    pub weighted: WeightedModes<T>,
}

impl<T> Modes<T> {
    pub(super) fn from_fn(mut create: impl FnMut(ModeId) -> T) -> Self {
        Self {
            raw: create(ModeId::Raw),
            weighted: WeightedModes::from_fn(|id| create(id.mode())),
        }
    }

    pub(super) fn try_from_fn<E>(
        mut create: impl FnMut(ModeId) -> Result<T, E>,
    ) -> Result<Self, E> {
        Ok(Self {
            raw: create(ModeId::Raw)?,
            weighted: WeightedModes::try_from_fn(|id| create(id.mode()))?,
        })
    }

    pub(super) fn select_mut(&mut self, id: ModeId) -> &mut T {
        match id {
            ModeId::Raw => &mut self.raw,
            _ => self
                .weighted
                .select_mut(id.weighted().expect("weighted mode")),
        }
    }

    pub(super) fn select(&self, id: ModeId) -> &T {
        match id {
            ModeId::Raw => &self.raw,
            _ => self.weighted.select(id.weighted().expect("weighted mode")),
        }
    }

    pub(super) fn iter(&self) -> impl Iterator<Item = &T> {
        std::iter::once(&self.raw).chain(self.weighted.iter())
    }

    pub(super) fn iter_mut(&mut self) -> impl Iterator<Item = &mut T> {
        std::iter::once(&mut self.raw).chain(self.weighted.iter_mut())
    }
}
