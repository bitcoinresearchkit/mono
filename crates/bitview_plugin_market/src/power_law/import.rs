use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_primitives::Day1;
use bitview_vecs::Price;
use brk_error::Result;
use brk_types::{Cents, Height, Version};
use vecdb::{Database, LazyVec, ReadableCloneableVec};

use super::{Vecs, published};

impl Vecs {
    pub(crate) fn import(db: &Database, version: Version, mappings: &MappingsVecs) -> Result<Self> {
        // A published line, lazily from the block's day.
        let line = |name: &str, compute: fn(Height, Day1) -> Cents| {
            let source = LazyVec::new(
                &format!("{name}_cents_source"),
                version,
                mappings.height.day1.read_only_boxed_clone(),
                compute,
            );
            Price::from_height_source(name, version, &source, mappings)
        };
        Ok(Self {
            burger: line("burger_power_law", published::burger),
            burger_floor: line("burger_power_law_floor", published::burger_floor),
            mooij_floor: line("mooij_power_law_floor", published::mooij),
            rainbow_lowest_band: line("rainbow_lowest_band", published::rainbow_lowest_band),
            sminston_floor: Price::import(db, "sminston_power_law_floor", version, mappings)?,
            sminston_state: Default::default(),
        })
    }
}
