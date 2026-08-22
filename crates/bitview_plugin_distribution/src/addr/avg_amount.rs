use brk_error::Result;

use bitview_cohort::{AddrTypeId, ByAddrType};
use bitview_traversable::Traversable;
use brk_types::{Cents, Height, Sats, StoredU64, Version};
use rayon::prelude::*;
use vecdb::{
    AnyStoredVec, CachedBoxedVec, Database, Exit, ReadOnlyClone, ReadableCloneableVec, ReadableVec,
    Rw, StorageMode, WritableVec,
};

use crate::AllChainSources;
use bitview_compute::{
    ColumnarPerBlock, LazyColumnSpotValuePerBlock, LazySpotValuePerBlock, WithAddrTypes,
};

#[derive(Traversable)]
pub struct AvgAmountVecs<M: StorageMode = Rw> {
    /// Mean value of an output unspent at the represented block: unspent supply
    /// divided by unspent output count.
    pub utxo: WithAddrTypes<LazyColumnSpotValuePerBlock<AddrTypeId>, LazySpotValuePerBlock>,
    /// Mean balance of a funded address: unspent supply divided by funded
    /// address count.
    pub addr: WithAddrTypes<LazyColumnSpotValuePerBlock<AddrTypeId>, LazySpotValuePerBlock>,
    #[traversable(hidden)]
    utxo_source: ColumnarPerBlock<Sats, AddrTypeId, (), M>,
    #[traversable(hidden)]
    addr_source: ColumnarPerBlock<Sats, AddrTypeId, (), M>,
}

impl AvgAmountVecs {
    pub fn forced_import(
        db: &Database,
        version: Version,
        mappings: &bitview_plugin_mappings::Vecs,
        spot_price: &CachedBoxedVec<Height, Cents>,
        all_chain: &AllChainSources,
        utxo_count: &(impl ReadableCloneableVec<Height, StoredU64> + 'static),
        funded_addr_count: &(impl ReadableCloneableVec<Height, StoredU64> + 'static),
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
        let utxo_source =
            ColumnarPerBlock::forced_import(db, "avg_utxo_amount_sats_by_type", version, |_| ())?;
        let addr_source =
            ColumnarPerBlock::forced_import(db, "avg_addr_amount_sats_by_type", version, |_| ())?;
        let utxo_columns = utxo_source.height.read_only_clone();
        let addr_columns = addr_source.height.read_only_clone();
        let utxo = WithAddrTypes {
            all: LazySpotValuePerBlock::from_sats_source(
                "avg_utxo_amount",
                version,
                avg_utxo,
                mappings,
                spot_price,
            ),
            by_addr_type: AddrTypeId::series(|column, type_name| {
                LazyColumnSpotValuePerBlock::new(
                    &format!("{type_name}_avg_utxo_amount"),
                    version,
                    &utxo_columns,
                    column,
                    mappings,
                    spot_price,
                )
            }),
        };
        let addr = WithAddrTypes {
            all: LazySpotValuePerBlock::from_sats_source(
                "avg_addr_amount",
                version,
                avg_addr,
                mappings,
                spot_price,
            ),
            by_addr_type: AddrTypeId::series(|column, type_name| {
                LazyColumnSpotValuePerBlock::new(
                    &format!("{type_name}_avg_addr_amount"),
                    version,
                    &addr_columns,
                    column,
                    mappings,
                    spot_price,
                )
            }),
        };

        Ok(Self {
            utxo,
            addr,
            utxo_source,
            addr_source,
        })
    }

    pub fn par_iter_height_mut(&mut self) -> impl ParallelIterator<Item = &mut dyn AnyStoredVec> {
        rayon::iter::once(self.utxo_source.stored_mut())
            .chain(rayon::iter::once(self.addr_source.stored_mut()))
    }

    pub fn reset_height(&mut self) -> Result<()> {
        self.utxo_source.height.reset()?;
        self.addr_source.height.reset()?;
        Ok(())
    }

    pub fn compute(
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
