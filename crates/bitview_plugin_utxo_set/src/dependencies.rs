use bitview_plugin_mappings::Vecs as Mappings;
use bitview_plugin_price::Vecs as Price;
use brk_types::Height;
use statedb::{Creations, Spends};

pub struct Dependencies<'a> {
    pub spends: &'a Spends,
    pub creations: &'a Creations,
    /// Creation prices and timestamps, which place the outputs in the URPD history.
    pub price: &'a Price,
    pub mappings: &'a Mappings,
    pub from: Height,
    pub end: usize,
}
