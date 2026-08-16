use std::collections::BTreeMap;

use brk_types::Index;
use derive_more::{Deref, DerefMut};

use super::SeriesEntry;

#[derive(Default, Deref, DerefMut)]
pub struct IndexToVec<'a> {
    #[deref]
    #[deref_mut]
    vecs: BTreeMap<Index, SeriesEntry<'a>>,
    description: Option<&'static str>,
}

impl IndexToVec<'_> {
    pub(super) fn description(&self) -> Option<&'static str> {
        self.description
    }

    pub(super) fn set_description(&mut self, description: &'static str) {
        assert!(
            self.description.replace(description).is_none(),
            "series description set more than once"
        );
    }
}
