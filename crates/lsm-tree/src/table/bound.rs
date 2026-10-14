use crate::Slice;

pub enum Bound {
    Included(Slice),
    Excluded(Slice),
}

impl Bound {
    pub fn key(&self) -> &Slice {
        match self {
            Self::Included(key) | Self::Excluded(key) => key,
        }
    }
}
