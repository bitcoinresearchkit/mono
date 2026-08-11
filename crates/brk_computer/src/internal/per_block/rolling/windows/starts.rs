use derive_more::{Deref, DerefMut};

use crate::{blocks::lookback::LazyWindowStartVec, internal::Windows};

#[derive(Deref, DerefMut)]
pub struct WindowStarts<'a>(pub Windows<&'a LazyWindowStartVec>);
