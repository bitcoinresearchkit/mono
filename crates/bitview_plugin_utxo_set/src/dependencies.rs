use statedb::{Creations, Spends};

pub struct Dependencies<'a> {
    pub spends: &'a Spends,
    pub creations: &'a Creations,
    pub end: usize,
}
