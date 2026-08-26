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

use brk_error::Result;

use std::ops::Deref;

use bitview_compute::{IndexSources, LazyCumulativeIndexVec, PerResolution};
use bitview_plugin::{
    ComputePlugin, ImportContext, Plugin, PluginGate, PluginId, PluginStorage, UpdateContext,
};
use bitview_plugin_indexer::Indexer;
use bitview_traversable::Traversable;
use brk_types::{
    Day1, Day3, Epoch, Halving, Height, Hour1, Hour4, Hour12, Minute10, Minute30, Month1, Month3,
    Month6, StoredU64, TxInIndex, TxIndex, TxOutIndex, Version, Week1, Year1, Year10,
};
use vecdb::{
    AnyVec, CachedBoxedVec, CachedVec, Database, Exit, ReadableBoxedVec, ReadableCloneableVec, Rw,
    StorageMode, VecIndex,
};

use addr::Vecs as AddrVecs;
use chain_counts::CachedChainCounts;
pub use dependencies::Dependencies;
pub use has::HasMappings;
use height::Vecs as HeightVecs;
pub use resolution::CachedFirstHeightVec;
use resolution::{DatedResolutionVecs, ResolutionVecs};
use timestamp::Timestamps;
use tx_heights::TxHeights;
use tx_index::Vecs as TxIndexVecs;
use txin_index::Vecs as TxInIndexVecs;
use txout_index::Vecs as TxOutIndexVecs;

const STORAGE: PluginStorage = PluginStorage::new(PluginId::new("mappings"), Version::new(9));
pub const ID: PluginId = STORAGE.id();

#[derive(Traversable)]
pub struct Vecs<M: StorageMode = Rw> {
    #[traversable(skip)]
    plugin_gate: PluginGate,
    #[traversable(skip)]
    db: Database,
    #[traversable(skip)]
    chain_counts: CachedChainCounts,
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

    fn gate(&self) -> &PluginGate {
        &self.plugin_gate
    }
}

impl Vecs {
    pub fn import(context: ImportContext<'_>, indexer: &Indexer) -> Result<Self> {
        let db = STORAGE.open_database(context, 1_000_000)?;
        let version = STORAGE.schema_version();

        let addr = AddrVecs::forced_import(version, indexer);
        let monotonic = Timestamps::forced_import_monotonic(&db, version)?;
        let chain_counts = CachedChainCounts::new(version, indexer);
        let height = HeightVecs::new(
            version,
            monotonic.read_only_boxed_clone(),
            ReadableBoxedVec::new(chain_counts.transaction_source()),
        );
        let epoch = ResolutionVecs::new(&height.epoch);
        let halving = ResolutionVecs::new(&height.halving);
        let minute10 = ResolutionVecs::new(&height.minute10);
        let minute30 = ResolutionVecs::new(&height.minute30);
        let hour1 = ResolutionVecs::new(&height.hour1);
        let hour4 = ResolutionVecs::new(&height.hour4);
        let hour12 = ResolutionVecs::new(&height.hour12);
        let monotonic_source = monotonic.read_only_boxed_clone();
        let day1 = DatedResolutionVecs::from_period_date(
            &height.day1,
            monotonic_source.clone(),
            HeightVecs::day1_from_timestamp,
        );
        let day3 = DatedResolutionVecs::from_first_timestamp(
            &height.day3,
            monotonic_source.clone(),
            Day3::from_timestamp,
        );
        let week1 = DatedResolutionVecs::from_first_timestamp(
            &height.week1,
            monotonic_source.clone(),
            HeightVecs::week1_from_timestamp,
        );
        let month1 = DatedResolutionVecs::from_first_timestamp(
            &height.month1,
            monotonic_source.clone(),
            HeightVecs::month1_from_timestamp,
        );
        let month3 = DatedResolutionVecs::from_first_timestamp(
            &height.month3,
            monotonic_source.clone(),
            HeightVecs::month3_from_timestamp,
        );
        let month6 = DatedResolutionVecs::from_first_timestamp(
            &height.month6,
            monotonic_source.clone(),
            HeightVecs::month6_from_timestamp,
        );
        let year1 = DatedResolutionVecs::from_first_timestamp(
            &height.year1,
            monotonic_source.clone(),
            HeightVecs::year1_from_timestamp,
        );
        let year10 = DatedResolutionVecs::from_first_timestamp(
            &height.year10,
            monotonic_source,
            HeightVecs::year10_from_timestamp,
        );
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
                minute10: minute10.first_height.read_only_boxed_clone(),
                minute30: minute30.first_height.read_only_boxed_clone(),
                hour1: hour1.first_height.read_only_boxed_clone(),
                hour4: hour4.first_height.read_only_boxed_clone(),
                hour12: hour12.first_height.read_only_boxed_clone(),
                day1: day1.first_height.read_only_boxed_clone(),
                day3: day3.first_height.read_only_boxed_clone(),
                week1: week1.first_height.read_only_boxed_clone(),
                month1: month1.first_height.read_only_boxed_clone(),
                month3: month3.first_height.read_only_boxed_clone(),
                month6: month6.first_height.read_only_boxed_clone(),
                year1: year1.first_height.read_only_boxed_clone(),
                year10: year10.first_height.read_only_boxed_clone(),
                halving: halving.first_height.read_only_boxed_clone(),
                epoch: epoch.first_height.read_only_boxed_clone(),
            },
            cached_first_height: PerResolution {
                minute10: minute10.first_height.read_only_cached_boxed_clone(),
                minute30: minute30.first_height.read_only_cached_boxed_clone(),
                hour1: hour1.first_height.read_only_cached_boxed_clone(),
                hour4: hour4.first_height.read_only_cached_boxed_clone(),
                hour12: hour12.first_height.read_only_cached_boxed_clone(),
                day1: day1.first_height.read_only_cached_boxed_clone(),
                day3: day3.first_height.read_only_cached_boxed_clone(),
                week1: week1.first_height.read_only_cached_boxed_clone(),
                month1: month1.first_height.read_only_cached_boxed_clone(),
                month3: month3.first_height.read_only_cached_boxed_clone(),
                month6: month6.first_height.read_only_cached_boxed_clone(),
                year1: year1.first_height.read_only_cached_boxed_clone(),
                year10: year10.first_height.read_only_cached_boxed_clone(),
                halving: halving.first_height.read_only_cached_boxed_clone(),
                epoch: epoch.first_height.read_only_cached_boxed_clone(),
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
            height_day1: height.day1_read_only_boxed_clone(),
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
            plugin_gate: Default::default(),
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

    fn compute_inner(&mut self, indexer: &Indexer, exit: &Exit) -> Result<()> {
        self.db.sync_bg_tasks()?;

        let starting_height = indexer.safe_lengths().height;

        self.tx_heights.update(indexer, starting_height);
        if starting_height.to_usize() < indexer.vecs().transactions.first_tx_index.len() {
            self.chain_counts.invalidate();
        }

        // timestamp_monotonic must be computed first — other mappings read it
        let rewrote_existing = self
            .timestamp
            .compute_monotonic(indexer, starting_height, exit)?;
        if rewrote_existing {
            self.invalidate_timestamp_dependents();
        }

        let exit = exit.clone();
        self.db.run_bg(move |db| {
            let _lock = exit.lock();
            db.compact_deferred_default()
        });
        Ok(())
    }

    fn invalidate_timestamp_dependents(&self) {
        self.height.invalidate_timestamp_caches();

        macro_rules! period {
            ($($field:ident),+ $(,)?) => {
                $(self.$field.invalidate_timestamp_caches();)+
            };
        }

        period!(
            minute10, minute30, hour1, hour4, hour12, day1, day3, week1, month1, month3, month6,
            year1, year10, halving, epoch,
        );
    }

    pub fn transaction_count_source(&self) -> CachedVec<LazyCumulativeIndexVec<Height, TxIndex>> {
        self.chain_counts.transaction_source()
    }

    pub fn input_count_source(&self) -> CachedVec<LazyCumulativeIndexVec<Height, TxInIndex>> {
        self.chain_counts.input_source()
    }

    pub fn output_count(&self) -> CachedBoxedVec<Height, StoredU64> {
        self.chain_counts.output()
    }

    pub fn output_count_source(&self) -> CachedVec<LazyCumulativeIndexVec<Height, TxOutIndex>> {
        self.chain_counts.output_source()
    }
}

impl ComputePlugin for Vecs {
    type Dependencies<'a> = Dependencies<'a>;
    type Output = ();

    fn compute(
        &mut self,
        dependencies: Self::Dependencies<'_>,
        context: UpdateContext<'_>,
    ) -> Result<Self::Output> {
        self.compute_inner(dependencies.indexer, context.exit())
    }
}
