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
    pub fn description(&self) -> Option<&'static str> {
        self.description
    }
}

pub trait IndexToVecInternal {
    fn set_description(&mut self, description: &'static str);
}

impl IndexToVecInternal for IndexToVec<'_> {
    fn set_description(&mut self, description: &'static str) {
        assert!(
            self.description.replace(description).is_none(),
            "series description set more than once"
        );
    }
}
