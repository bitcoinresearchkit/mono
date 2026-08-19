use std::collections::BTreeMap;

use derive_more::{Deref, DerefMut};

use super::SeriesEntry;

#[derive(Default, Deref, DerefMut)]
pub struct SeriesToVec<'a>(BTreeMap<&'a str, SeriesEntry<'a>>);
