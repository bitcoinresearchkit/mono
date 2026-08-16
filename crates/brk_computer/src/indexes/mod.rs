mod addr;
mod chain_counts;
mod height;
mod resolution;
pub mod timestamp;
mod tx_heights;
mod tx_index;
mod txin_index;
mod txout_index;

use std::path::Path;

use brk_error::Result;
use brk_indexer::Indexer;
use brk_plugin::{Plugin, PluginGate};
use brk_traversable::Traversable;
use brk_types::{
    Day1, Day3, Epoch, Halving, Height, Hour1, Hour4, Hour12, Minute10, Minute30, Month1, Month3,
    Month6, StoredU64, TxInIndex, TxIndex, TxOutIndex, Version, Week1, Year1, Year10,
};
use vecdb::{AnyVec, CachedBoxedVec, CachedVec, Database, Exit, Rw, StorageMode, VecIndex};

use crate::internal::{
    LazyCumulativeIndexVec,
    db_utils::{finalize_db, open_db},
};

pub use addr::Vecs as AddrVecs;
use chain_counts::CachedChainCounts;
pub use height::Vecs as HeightVecs;
pub use resolution::{CachedDateVec, CachedFirstHeightVec, DatedResolutionVecs, ResolutionVecs};
pub use timestamp::Timestamps;
pub use tx_heights::TxHeights;
pub use tx_index::Vecs as TxIndexVecs;
pub use txin_index::Vecs as TxInIndexVecs;
pub use txout_index::Vecs as TxOutIndexVecs;

pub const DB_NAME: &str = "indexes";

#[derive(Traversable)]
pub struct Vecs<M: StorageMode = Rw> {
    #[traversable(skip)]
    pub(crate) plugin_gate: PluginGate,
    db: Database,
    #[traversable(skip)]
    chain_counts: CachedChainCounts,
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

impl<M: StorageMode> Plugin for Vecs<M>
where
    Self: Send + Sync,
{
    fn id(&self) -> &'static str {
        DB_NAME
    }

    fn gate(&self) -> &PluginGate {
        &self.plugin_gate
    }
}

impl Vecs {
    pub(crate) fn forced_import(
        parent: &Path,
        parent_version: Version,
        indexer: &Indexer,
    ) -> Result<Self> {
        let db = open_db(parent, DB_NAME, 1_000_000)?;

        let version = parent_version;

        let addr = AddrVecs::forced_import(version, indexer);
        let monotonic = Timestamps::forced_import_monotonic(&db, version)?;
        let chain_counts = CachedChainCounts::new(version, indexer);
        let height = HeightVecs::new(
            version,
            monotonic.read_only_boxed_clone(),
            Box::new(chain_counts.transaction_source()),
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

        let this = Self {
            plugin_gate: Default::default(),
            chain_counts,
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

        finalize_db(&this.db, &this)?;
        Ok(this)
    }

    pub(crate) fn compute(&mut self, indexer: &Indexer, exit: &Exit) -> Result<()> {
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

    pub(crate) fn transaction_count_source(
        &self,
    ) -> CachedVec<LazyCumulativeIndexVec<Height, TxIndex>> {
        self.chain_counts.transaction_source()
    }

    pub(crate) fn input_count_source(
        &self,
    ) -> CachedVec<LazyCumulativeIndexVec<Height, TxInIndex>> {
        self.chain_counts.input_source()
    }

    pub(crate) fn output_count(&self) -> CachedBoxedVec<Height, StoredU64> {
        self.chain_counts.output()
    }

    pub(crate) fn output_count_source(
        &self,
    ) -> CachedVec<LazyCumulativeIndexVec<Height, TxOutIndex>> {
        self.chain_counts.output_source()
    }
}
