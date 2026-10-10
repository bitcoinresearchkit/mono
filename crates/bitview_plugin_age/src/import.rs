use std::thread;

use bitview_cohort::{AgeRange, ByEpoch, Class};
use bitview_collections::Windows;
use bitview_plugin::ImportContext;
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_plugin_price::Vecs as PriceVecs;
use bitview_vecs::{LazyWindowStartVec, PerBlockCumulativeRolling};
use brk_error::Result;
use brk_types::{Height, Sats, Version};
use vecdb::{ReadableBoxedVec, ReadableCloneableVec};

use crate::{CohortVecs, RangeVecs, STORAGE, Vecs};

const IMPORT_STACK_SIZE: usize = 8 * 1024 * 1024;

impl Vecs {
    pub fn import(
        context: ImportContext<'_>,
        mappings: &MappingsVecs,
        windows: &Windows<&LazyWindowStartVec>,
        prices: &PriceVecs,
        all_supply: &ReadableBoxedVec<Height, Sats>,
    ) -> Result<Self> {
        let db = STORAGE.open_database(context, 20_000_000)?;
        let version = STORAGE.schema_version();
        let spot = prices.spot.cents.height.read_only_boxed_clone();
        let cohort =
            |id| CohortVecs::import(&db, id, version, mappings, windows, &spot, all_supply);
        // Age ranges and the other families import independently.
        let (ranges, (epochs, classes)) = thread::scope(|scope| -> Result<_> {
            let others = thread::Builder::new()
                .stack_size(IMPORT_STACK_SIZE)
                .spawn_scoped(scope, || -> Result<_> {
                    Ok((
                        Box::new(ByEpoch::try_new(cohort)?),
                        Box::new(Class::try_new(cohort)?),
                    ))
                })?;
            let ranges = Box::new(AgeRange::try_from_fn(|id| {
                RangeVecs::import(
                    &db,
                    id.cohort(),
                    version,
                    mappings,
                    windows,
                    &spot,
                    all_supply,
                )
            })?);
            Ok((ranges, others.join().unwrap()?))
        })?;
        let coinblocks_destroyed = PerBlockCumulativeRolling::import(
            &db,
            "coinblocks_destroyed",
            version + Version::TWO,
            mappings,
            windows,
        )?;
        STORAGE.finalize_database(&db)?;
        Ok(Self {
            db,
            live: None,
            all_supply: all_supply.read_only_boxed_clone(),
            ranges,
            epochs,
            classes,
            coinblocks_destroyed,
        })
    }
}
