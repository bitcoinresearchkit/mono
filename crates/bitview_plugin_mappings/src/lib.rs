#![allow(clippy::type_complexity)]

mod addr;
mod chain_counts;
mod dependencies;
mod has;
mod height;
mod resolution;
mod timestamp;
mod tx_heights;
mod tx_index;
mod txin_index;
mod txout_index;

use std::ops::Deref;

use addr::Vecs as AddrVecs;
use bitview_collections::PerResolution;
use bitview_plugin::{
    ComputePlugin, ImportContext, Plugin, PluginId, PluginStorage, UpdateContext,
};
use bitview_plugin_indexer::Indexer;
use bitview_traversable::Traversable;
use bitview_vecs::{IndexSources, LazyCumulativeIndexVec, LazyPreviousDeltaVec};
use brk_error::Result;
use brk_types::{
    Day1, Day3, Epoch, Halving, Height, Hour1, Hour4, Hour12, Minute10, Minute30, Month1, Month3,
    Month6, StoredU64, TxInIndex, TxIndex, TxOutIndex, Version, Week1, Year1, Year10,
};
use chain_counts::ChainCounts;
use height::Vecs as HeightVecs;
use resolution::{DatedResolutionVecs, ResolutionVecs};
use timestamp::Timestamps;
use tx_heights::TxHeights;
use tx_index::Vecs as TxIndexVecs;
use txin_index::Vecs as TxInIndexVecs;
use txout_index::Vecs as TxOutIndexVecs;
use vecdb::{Database, IndexVec, ReadableBoxedVec, ReadableCloneableVec, Rw, StorageMode};

pub use dependencies::Dependencies;
pub use has::HasMappings;
pub use tx_heights::TxHeightMap;

const STORAGE: PluginStorage = PluginStorage::new(PluginId::new("mappings"), Version::new(9));
pub const ID: PluginId = STORAGE.id();

#[derive(Traversable)]
pub struct Vecs<M: StorageMode = Rw> {
    #[traversable(skip)]
    db: Database,
    chain_counts: M::WriteOnly<ChainCounts>,
    #[traversable(skip)]
    sources: IndexSources,
    #[traversable(skip)]
    pub tx_heights: TxHeights,
    pub addr: AddrVecs,
    pub height: HeightVecs,
    pub epoch: ResolutionVecs<Epoch>,
    pub halving: ResolutionVecs<Halving>,
    pub minute10: ResolutionVecs<Minute10>,
    pub minute30: ResolutionVecs<Minute30>,
    pub hour1: ResolutionVecs<Hour1>,
    pub hour4: ResolutionVecs<Hour4>,
    pub hour12: ResolutionVecs<Hour12>,
    pub day1: DatedResolutionVecs<Day1>,
    pub day3: DatedResolutionVecs<Day3>,
    pub week1: DatedResolutionVecs<Week1>,
    pub month1: DatedResolutionVecs<Month1>,
    pub month3: DatedResolutionVecs<Month3>,
    pub month6: DatedResolutionVecs<Month6>,
    pub year1: DatedResolutionVecs<Year1>,
    pub year10: DatedResolutionVecs<Year10>,
    pub tx_index: TxIndexVecs,
    pub txin_index: TxInIndexVecs,
    pub txout_index: TxOutIndexVecs,
    pub timestamp: Timestamps<M>,
}

impl<M: StorageMode> Deref for Vecs<M> {
    type Target = IndexSources;

    fn deref(&self) -> &Self::Target {
        &self.sources
    }
}

impl<M: StorageMode> Plugin for Vecs<M>
where
    Self: Traversable + Send + Sync,
{
    fn storage(&self) -> PluginStorage {
        STORAGE
    }
}

impl Vecs {
    pub fn import(context: ImportContext<'_>, indexer: &Indexer) -> Result<Self> {
        let db = STORAGE.open_database(context, 1_000_000)?;
        let version = STORAGE.schema_version();

        let addr = AddrVecs::forced_import(version, indexer);
        let monotonic = Timestamps::forced_import_monotonic(&db, version)?;
        let chain_counts = ChainCounts::new(version, indexer);
        let monotonic_source = monotonic.read_only_boxed_clone();
        let epoch_source = IndexVec::new(
            "epoch",
            Version::ZERO,
            monotonic.read_only_boxed_clone(),
            Epoch::from,
        );
        let halving_source = IndexVec::new(
            "halving",
            Version::ZERO,
            monotonic_source.clone(),
            Halving::from,
        );
        let epoch = ResolutionVecs::new(&epoch_source);
        let halving = ResolutionVecs::new(&halving_source);
        // Fixed-duration buckets are cheaper to read with timestamp arithmetic;
        // calendar conversions below use the shared resident reverse lookups.
        let minute10_source =
            HeightVecs::from_timestamps("minute10", monotonic_source.clone(), |_, t| {
                Minute10::from_timestamp(t)
            });
        let minute30_source =
            HeightVecs::from_timestamps("minute30", monotonic_source.clone(), |_, t| {
                Minute30::from_timestamp(t)
            });
        let hour1_source =
            HeightVecs::from_timestamps("hour1", monotonic_source.clone(), |_, t| {
                Hour1::from_timestamp(t)
            });
        let hour4_source =
            HeightVecs::from_timestamps("hour4", monotonic_source.clone(), |_, t| {
                Hour4::from_timestamp(t)
            });
        let hour12_source =
            HeightVecs::from_timestamps("hour12", monotonic_source.clone(), |_, t| {
                Hour12::from_timestamp(t)
            });
        let day3_source = HeightVecs::from_timestamps("day3", monotonic_source.clone(), |_, t| {
            Day3::from_timestamp(t)
        });
        let minute10 = ResolutionVecs::new(&minute10_source);
        let minute30 = ResolutionVecs::new(&minute30_source);
        let hour1 = ResolutionVecs::new(&hour1_source);
        let hour4 = ResolutionVecs::new(&hour4_source);
        let hour12 = ResolutionVecs::new(&hour12_source);
        let day1 = DatedResolutionVecs::from_period_date(&HeightVecs::from_timestamps(
            "day1",
            monotonic_source.clone(),
            |_, t| HeightVecs::day1_from_timestamp(t),
        ));
        let day3 = DatedResolutionVecs::from_first_timestamp(&day3_source, &monotonic_source);
        let week1 = DatedResolutionVecs::from_first_timestamp(
            &HeightVecs::from_timestamps("week1", monotonic_source.clone(), |_, t| {
                HeightVecs::week1_from_timestamp(t)
            }),
            &monotonic_source,
        );
        let month1 = DatedResolutionVecs::from_first_timestamp(
            &HeightVecs::from_timestamps("month1", monotonic_source.clone(), |_, t| {
                HeightVecs::month1_from_timestamp(t)
            }),
            &monotonic_source,
        );
        let month3 = DatedResolutionVecs::from_first_timestamp(
            &HeightVecs::from_timestamps("month3", monotonic_source.clone(), |_, t| {
                HeightVecs::month3_from_timestamp(t)
            }),
            &monotonic_source,
        );
        let month6 = DatedResolutionVecs::from_first_timestamp(
            &HeightVecs::from_timestamps("month6", monotonic_source.clone(), |_, t| {
                HeightVecs::month6_from_timestamp(t)
            }),
            &monotonic_source,
        );
        let year1 = DatedResolutionVecs::from_first_timestamp(
            &HeightVecs::from_timestamps("year1", monotonic_source.clone(), |_, t| {
                HeightVecs::year1_from_timestamp(t)
            }),
            &monotonic_source,
        );
        let year10 = DatedResolutionVecs::from_first_timestamp(
            &HeightVecs::from_timestamps("year10", monotonic_source.clone(), |_, t| {
                HeightVecs::year10_from_timestamp(t)
            }),
            &monotonic_source,
        );
        let height = HeightVecs {
            minute10: minute10_source,
            minute30: minute30_source,
            hour1: hour1_source,
            hour4: hour4_source,
            hour12: hour12_source,
            day1: day1.height_lookup(),
            day3: day3_source,
            week1: week1.height_lookup(),
            month1: month1.height_lookup(),
            month3: month3.height_lookup(),
            month6: month6.height_lookup(),
            year1: year1.height_lookup(),
            year10: year10.height_lookup(),
            epoch: epoch_source,
            halving: halving_source,
            tx_index_count: LazyPreviousDeltaVec::new(
                "tx_index_count",
                version,
                &chain_counts.transaction_source(),
            ),
        };
        let tx_index = TxIndexVecs::new(version, indexer);
        let txin_index = TxInIndexVecs::forced_import(version, indexer);
        let txout_index = TxOutIndexVecs::forced_import(version, indexer);

        let timestamp = Timestamps::from_locals(
            version,
            monotonic,
            indexer.vecs().blocks.timestamp.read_only_boxed_clone(),
            &minute10,
            &minute30,
            &hour1,
            &hour4,
            &hour12,
            &day1,
            &day3,
            &week1,
            &month1,
            &month3,
            &month6,
            &year1,
            &year10,
        );

        let sources = IndexSources {
            first_height: PerResolution {
                minute10: minute10.first_height.clone(),
                minute30: minute30.first_height.clone(),
                hour1: hour1.first_height.clone(),
                hour4: hour4.first_height.clone(),
                hour12: hour12.first_height.clone(),
                day1: day1.first_height.clone(),
                day3: day3.first_height.clone(),
                week1: week1.first_height.clone(),
                month1: month1.first_height.clone(),
                month3: month3.first_height.clone(),
                month6: month6.first_height.clone(),
                year1: year1.first_height.clone(),
                year10: year10.first_height.clone(),
                halving: halving.first_height.clone(),
                epoch: epoch.first_height.clone(),
            },
            timestamp: PerResolution {
                minute10: timestamp.minute10.read_only_boxed_clone(),
                minute30: timestamp.minute30.read_only_boxed_clone(),
                hour1: timestamp.hour1.read_only_boxed_clone(),
                hour4: timestamp.hour4.read_only_boxed_clone(),
                hour12: timestamp.hour12.read_only_boxed_clone(),
                day1: timestamp.day1.read_only_boxed_clone(),
                day3: timestamp.day3.read_only_boxed_clone(),
                week1: timestamp.week1.read_only_boxed_clone(),
                month1: timestamp.month1.read_only_boxed_clone(),
                month3: timestamp.month3.read_only_boxed_clone(),
                month6: timestamp.month6.read_only_boxed_clone(),
                year1: timestamp.year1.read_only_boxed_clone(),
                year10: timestamp.year10.read_only_boxed_clone(),
                halving: timestamp.halving.read_only_boxed_clone(),
                epoch: timestamp.epoch.read_only_boxed_clone(),
            },
            height_minute10: height.minute10.read_only_boxed_clone(),
            height_day1: height.day1.read_only_boxed_clone(),
            height_tx_index_count: height.tx_index_count.clone(),
            day3_date: day3.date.read_only_boxed_clone(),
            week1_date: week1.date.read_only_boxed_clone(),
            month1_date: month1.date.read_only_boxed_clone(),
            month3_date: month3.date.read_only_boxed_clone(),
            month6_date: month6.date.read_only_boxed_clone(),
            year1_date: year1.date.read_only_boxed_clone(),
            year10_date: year10.date.read_only_boxed_clone(),
        };

        let this = Self {
            chain_counts,
            sources,
            tx_heights: TxHeights::init(indexer),
            addr,
            height,
            epoch,
            halving,
            minute10,
            minute30,
            hour1,
            hour4,
            hour12,
            day1,
            day3,
            week1,
            month1,
            month3,
            month6,
            year1,
            year10,
            tx_index,
            txin_index,
            txout_index,
            timestamp,
            db,
        };

        STORAGE.finalize_database(&this.db)?;
        Ok(this)
    }

    pub fn transaction_count_source(&self) -> LazyCumulativeIndexVec<Height, TxIndex> {
        self.chain_counts.transaction_source()
    }

    pub fn input_count_source(&self) -> LazyCumulativeIndexVec<Height, TxInIndex> {
        self.chain_counts.input_source()
    }

    pub fn output_count(&self) -> ReadableBoxedVec<Height, StoredU64> {
        self.chain_counts.output()
    }

    pub fn output_count_source(&self) -> LazyCumulativeIndexVec<Height, TxOutIndex> {
        self.chain_counts.output_source()
    }
}

impl ComputePlugin for Vecs {
    type Dependencies<'a> = Dependencies<'a>;

    fn compute(
        &mut self,
        dependencies: Self::Dependencies<'_>,
        context: UpdateContext<'_>,
    ) -> Result<()> {
        let Dependencies { indexer } = dependencies;
        let exit = context.exit();
        self.db.sync_bg_tasks()?;

        let starting_height = indexer.safe_lengths().height;

        self.tx_heights.update(indexer, starting_height);

        // timestamp_monotonic must be computed first — other mappings read it
        self.timestamp
            .compute_monotonic(indexer, starting_height, exit)?;

        self.epoch.update(starting_height);
        self.halving.update(starting_height);
        self.minute10.update(starting_height);
        self.minute30.update(starting_height);
        self.hour1.update(starting_height);
        self.hour4.update(starting_height);
        self.hour12.update(starting_height);
        self.day1.update(starting_height);
        self.day3.update(starting_height);
        self.week1.update(starting_height);
        self.month1.update(starting_height);
        self.month3.update(starting_height);
        self.month6.update(starting_height);
        self.year1.update(starting_height);
        self.year10.update(starting_height);

        context.compact_database(&self.db);
        Ok(())
    }
}
