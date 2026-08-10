use brk_cohort::{AddrTypeId, ByAddrType};
use brk_error::Result;
use brk_traversable::Traversable;
use brk_types::{Cents, Height, Sats, StoredU64, Version};
use derive_more::{Deref, DerefMut};
use rayon::prelude::*;
use vecdb::{
    AnyStoredVec, CachedBoxedVec, Database, Exit, ReadOnlyClone, ReadableVec, Rw, StorageMode,
    WritableVec,
};

use crate::{
    distribution::AllChainCache,
    indexes,
    internal::{
        ColumnarPerBlock, LazyColumnSpotValuePerBlock, LazySpotValuePerBlock, WithAddrTypes,
    },
};

/// Average amount held per UTXO and per funded address.
///
/// `utxo = supply / utxo_count`, `addr = supply / funded_addr_count`.
#[derive(Clone, Traversable)]
pub struct AvgAmountMetrics<V> {
    pub utxo: V,
    pub addr: V,
}

#[derive(Deref, DerefMut, Traversable)]
pub struct AvgAmountVecs<M: StorageMode = Rw> {
    #[deref]
    #[deref_mut]
    #[traversable(flatten)]
    pub series: WithAddrTypes<
        AvgAmountMetrics<LazyColumnSpotValuePerBlock<AddrTypeId>>,
        AvgAmountMetrics<LazySpotValuePerBlock>,
    >,
    #[traversable(hidden)]
    utxo_source: ColumnarPerBlock<Sats, AddrTypeId, (), M>,
    #[traversable(hidden)]
    addr_source: ColumnarPerBlock<Sats, AddrTypeId, (), M>,
}

impl AvgAmountVecs {
    pub(crate) fn forced_import(
        db: &Database,
        version: Version,
        indexes: &indexes::Vecs,
        spot_price: &CachedBoxedVec<Height, Cents>,
        all_chain: &AllChainCache,
        utxo_count: &(impl vecdb::ReadableCloneableVec<Height, StoredU64> + 'static),
        funded_addr_count: &(impl vecdb::ReadableCloneableVec<Height, StoredU64> + 'static),
    ) -> Result<Self> {
        let avg_utxo = all_chain.with_supply(
            "avg_utxo_amount_sats_source",
            Version::ZERO,
            utxo_count,
            |_, count, supply| supply / count,
        );
        let avg_addr = all_chain.with_supply(
            "avg_addr_amount_sats_source",
            Version::ZERO,
            funded_addr_count,
            |_, count, supply| supply / count,
        );
        let all = AvgAmountMetrics {
            utxo: LazySpotValuePerBlock::from_sats_source(
                "avg_utxo_amount",
                version,
                avg_utxo,
                indexes,
                spot_price,
            ),
            addr: LazySpotValuePerBlock::from_sats_source(
                "avg_addr_amount",
                version,
                avg_addr,
                indexes,
                spot_price,
            ),
        };

        let utxo_source =
            ColumnarPerBlock::forced_import(db, "avg_utxo_amount_sats_by_type", version, |_| ())?;
        let addr_source =
            ColumnarPerBlock::forced_import(db, "avg_addr_amount_sats_by_type", version, |_| ())?;
        let utxo = utxo_source.height.read_only_clone();
        let addr = addr_source.height.read_only_clone();
        let by_addr_type = AddrTypeId::series(|column, type_name| AvgAmountMetrics {
            utxo: LazyColumnSpotValuePerBlock::new(
                &format!("{type_name}_avg_utxo_amount"),
                version,
                &utxo,
                column,
                indexes,
                spot_price,
            ),
            addr: LazyColumnSpotValuePerBlock::new(
                &format!("{type_name}_avg_addr_amount"),
                version,
                &addr,
                column,
                indexes,
                spot_price,
            ),
        });

        Ok(Self {
            series: WithAddrTypes { all, by_addr_type },
            utxo_source,
            addr_source,
        })
    }

    pub(crate) fn par_iter_height_mut(
        &mut self,
    ) -> impl ParallelIterator<Item = &mut dyn AnyStoredVec> {
        rayon::iter::once(self.utxo_source.stored_mut())
            .chain(rayon::iter::once(self.addr_source.stored_mut()))
    }

    pub(crate) fn reset_height(&mut self) -> Result<()> {
        self.utxo_source.height.reset()?;
        self.addr_source.height.reset()?;
        Ok(())
    }

    pub(crate) fn compute(
        &mut self,
        supply_sats: &ByAddrType<&impl ReadableVec<Height, Sats>>,
        utxo_count: &ByAddrType<&impl ReadableVec<Height, StoredU64>>,
        funded_addr_count: &ByAddrType<&impl ReadableVec<Height, StoredU64>>,
        max_from: Height,
        exit: &Exit,
    ) -> Result<()> {
        self.utxo_source.compute_columns2(
            max_from,
            |column| *column.select(supply_sats),
            |column| *column.select(utxo_count),
            |_, supply, count| supply / count,
            exit,
        )?;
        self.addr_source.compute_columns2(
            max_from,
            |column| *column.select(supply_sats),
            |column| *column.select(funded_addr_count),
            |_, supply, count| supply / count,
            exit,
        )?;

        Ok(())
    }
}
