use std::collections::BTreeMap;

use brk_types::Index;
use derive_more::{Deref, DerefMut};

use super::SeriesEntry;

#[derive(Default, Deref, DerefMut)]
pub struct IndexToVec<'a> {
    #[deref]
    #[deref_mut]
    vecs: BTreeMap<Index, SeriesEntry<'a>>,
}
