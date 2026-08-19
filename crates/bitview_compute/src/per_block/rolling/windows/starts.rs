use derive_more::{Deref, DerefMut};

use crate::{LazyWindowStartVec, Windows};

#[derive(Deref, DerefMut)]
pub struct WindowStarts<'a>(pub Windows<&'a LazyWindowStartVec>);
