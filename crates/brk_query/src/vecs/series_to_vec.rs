use std::collections::BTreeMap;

use derive_more::{Deref, DerefMut};
use vecdb::AnyExportableVec;

#[derive(Default, Deref, DerefMut)]
pub struct SeriesToVec<'a>(BTreeMap<&'a str, &'a dyn AnyExportableVec>);
