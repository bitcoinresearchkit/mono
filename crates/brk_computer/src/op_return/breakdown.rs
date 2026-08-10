use brk_error::Result;
use brk_traversable::Traversable;
use brk_types::{
    Bytes, Height, OpReturnKind, OpReturnPolicyId, PartsPerMillion32, Sats, StoredU64, VSize,
    Version,
};
use derive_more::{Deref, DerefMut};
use vecdb::{AnyVec, CachedBoxedVec, ColumnId, Database, Rw, StorageMode};

use super::{by_kind::ByKind, policy::Policy};
use crate::{
    indexes,
    internal::{
        CachedWindowStartVec, ColumnarPerBlockCumulativeRolling,
        LazyColumnPerBlockCumulativeRolling, LazyPercentCumulativeRolling, LazyPercentPerBlock,
        RatioBytes, RatioSats, Windows,
    },
};

#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct BlockMetrics {
    pub output_count: u64,
    pub data_bytes: Bytes,
    pub tx_count: u64,
    pub tx_vsize: VSize,
    pub fees: Sats,
}

pub trait BreakdownAxis: ColumnId {
    type Series<T>: Clone + Traversable
    where
        T: Clone + Traversable + Send + Sync + 'static;

    fn series<T>(create: impl FnMut(Self, &'static str) -> T) -> Self::Series<T>
    where
        T: Clone + Traversable + Send + Sync + 'static;
}

impl BreakdownAxis for OpReturnKind {
    type Series<T>
        = ByKind<T>
    where
        T: Clone + Traversable + Send + Sync + 'static;

    fn series<T>(create: impl FnMut(Self, &'static str) -> T) -> Self::Series<T>
    where
        T: Clone + Traversable + Send + Sync + 'static,
    {
        ByKind::new(create)
    }
}

impl BreakdownAxis for OpReturnPolicyId {
    type Series<T>
        = Policy<T>
    where
        T: Clone + Traversable + Send + Sync + 'static;

    fn series<T>(create: impl FnMut(Self, &'static str) -> T) -> Self::Series<T>
    where
        T: Clone + Traversable + Send + Sync + 'static,
    {
        Policy::new(create)
    }
}

#[derive(Clone, Deref, DerefMut, Traversable)]
pub struct DataBytesSeries<C: ColumnId> {
    #[deref]
    #[deref_mut]
    #[traversable(flatten)]
    pub data_bytes: LazyColumnPerBlockCumulativeRolling<Bytes, C>,
    pub data_share: LazyPercentPerBlock<PartsPerMillion32>,
    pub chain_share: LazyPercentPerBlock<PartsPerMillion32>,
}

#[derive(Clone, Deref, DerefMut, Traversable)]
pub struct FeesSeries<C: ColumnId> {
    #[deref]
    #[deref_mut]
    #[traversable(flatten)]
    pub fees: LazyColumnPerBlockCumulativeRolling<Sats, C>,
    pub fee_share: LazyPercentCumulativeRolling<PartsPerMillion32>,
}

impl<C: ColumnId> DataBytesSeries<C> {
    fn new(
        prefix: &str,
        version: Version,
        data_bytes: LazyColumnPerBlockCumulativeRolling<Bytes, C>,
        total_data: CachedBoxedVec<Height, Bytes>,
        block_size: CachedBoxedVec<Height, StoredU64>,
        indexes: &indexes::Vecs,
    ) -> Self {
        let data_share =
            LazyPercentPerBlock::from_cached_ratio::<Bytes, _, RatioBytes<PartsPerMillion32>>(
                &format!("{prefix}_data_share"),
                version,
                &data_bytes.cumulative.height,
                total_data,
                indexes,
            );
        let chain_share =
            LazyPercentPerBlock::from_cached_ratio::<Bytes, _, RatioBytes<PartsPerMillion32>>(
                &format!("{prefix}_chain_share"),
                version,
                &data_bytes.cumulative.height,
                block_size,
                indexes,
            );

        Self {
            data_bytes,
            data_share,
            chain_share,
        }
    }
}

impl<C: ColumnId> FeesSeries<C> {
    fn new(
        prefix: &str,
        version: Version,
        fees: LazyColumnPerBlockCumulativeRolling<Sats, C>,
        chain_fees: CachedBoxedVec<Height, Sats>,
        cached_starts: &Windows<&CachedWindowStartVec>,
        indexes: &indexes::Vecs,
    ) -> Self {
        let fee_share = LazyPercentCumulativeRolling::from_cumulative_ratio::<
            Sats,
            Sats,
            RatioSats<PartsPerMillion32>,
        >(
            &format!("{prefix}_fee_share"),
            version,
            &fees.cumulative.height,
            chain_fees,
            cached_starts,
            indexes,
        );

        Self { fees, fee_share }
    }
}

#[derive(Traversable)]
pub struct BreakdownVecs<C: BreakdownAxis, M: StorageMode = Rw> {
    pub output_count: ColumnarPerBlockCumulativeRolling<
        StoredU64,
        C,
        C::Series<LazyColumnPerBlockCumulativeRolling<StoredU64, C>>,
        M,
    >,
    pub data_bytes: ColumnarPerBlockCumulativeRolling<Bytes, C, C::Series<DataBytesSeries<C>>, M>,
    pub tx_count: ColumnarPerBlockCumulativeRolling<
        StoredU64,
        C,
        C::Series<LazyColumnPerBlockCumulativeRolling<StoredU64, C>>,
        M,
    >,
    pub tx_vsize: ColumnarPerBlockCumulativeRolling<
        VSize,
        C,
        C::Series<LazyColumnPerBlockCumulativeRolling<VSize, C>>,
        M,
    >,
    pub fees: ColumnarPerBlockCumulativeRolling<Sats, C, C::Series<FeesSeries<C>>, M>,
}

impl<C: BreakdownAxis> BreakdownVecs<C> {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn forced_import(
        db: &Database,
        source_prefix: &str,
        series_prefix: &str,
        version: Version,
        indexes: &indexes::Vecs,
        cached_starts: &Windows<&CachedWindowStartVec>,
        total_data: &CachedBoxedVec<Height, Bytes>,
        block_size: &CachedBoxedVec<Height, StoredU64>,
        chain_fees: &CachedBoxedVec<Height, Sats>,
    ) -> Result<Self> {
        let output_count = ColumnarPerBlockCumulativeRolling::forced_import(
            db,
            &format!("{source_prefix}_output_count"),
            version,
            |source| {
                C::series(|column, name| {
                    LazyColumnPerBlockCumulativeRolling::new(
                        &format!("{series_prefix}_{name}_output_count"),
                        version,
                        source,
                        column,
                        indexes,
                        cached_starts,
                    )
                })
            },
        )?;
        let data_bytes = ColumnarPerBlockCumulativeRolling::forced_import(
            db,
            &format!("{source_prefix}_data_bytes"),
            version,
            |source| {
                C::series(|column, name| {
                    let prefix = format!("{series_prefix}_{name}");
                    let data_bytes = LazyColumnPerBlockCumulativeRolling::new(
                        &format!("{prefix}_data_bytes"),
                        version,
                        source,
                        column,
                        indexes,
                        cached_starts,
                    );
                    DataBytesSeries::new(
                        &prefix,
                        version,
                        data_bytes,
                        total_data.clone(),
                        block_size.clone(),
                        indexes,
                    )
                })
            },
        )?;
        let tx_count = ColumnarPerBlockCumulativeRolling::forced_import(
            db,
            &format!("{source_prefix}_tx_count"),
            version,
            |source| {
                C::series(|column, name| {
                    LazyColumnPerBlockCumulativeRolling::new(
                        &format!("{series_prefix}_{name}_tx_count"),
                        version,
                        source,
                        column,
                        indexes,
                        cached_starts,
                    )
                })
            },
        )?;
        let tx_vsize = ColumnarPerBlockCumulativeRolling::forced_import(
            db,
            &format!("{source_prefix}_tx_vsize"),
            version,
            |source| {
                C::series(|column, name| {
                    LazyColumnPerBlockCumulativeRolling::new(
                        &format!("{series_prefix}_{name}_tx_vsize"),
                        version,
                        source,
                        column,
                        indexes,
                        cached_starts,
                    )
                })
            },
        )?;
        let fees = ColumnarPerBlockCumulativeRolling::forced_import(
            db,
            &format!("{source_prefix}_fees"),
            version,
            |source| {
                C::series(|column, name| {
                    let prefix = format!("{series_prefix}_{name}");
                    let fees = LazyColumnPerBlockCumulativeRolling::new(
                        &format!("{prefix}_fees"),
                        version,
                        source,
                        column,
                        indexes,
                        cached_starts,
                    );
                    FeesSeries::new(
                        &prefix,
                        version,
                        fees,
                        chain_fees.clone(),
                        cached_starts,
                        indexes,
                    )
                })
            },
        )?;

        Ok(Self {
            output_count,
            data_bytes,
            tx_count,
            tx_vsize,
            fees,
        })
    }

    pub(crate) fn len(&self) -> usize {
        self.output_count
            .cumulative
            .len()
            .min(self.data_bytes.cumulative.len())
            .min(self.tx_count.cumulative.len())
            .min(self.tx_vsize.cumulative.len())
            .min(self.fees.cumulative.len())
    }

    pub(crate) fn push(&mut self, row: C::Row<BlockMetrics>) {
        self.output_count.push_block(C::map(row.clone(), |metrics| {
            StoredU64::from(metrics.output_count)
        }));
        self.data_bytes
            .push_block(C::map(row.clone(), |metrics| metrics.data_bytes));
        self.tx_count.push_block(C::map(row.clone(), |metrics| {
            StoredU64::from(metrics.tx_count)
        }));
        self.tx_vsize
            .push_block(C::map(row.clone(), |metrics| metrics.tx_vsize));
        self.fees.push_block(C::map(row, |metrics| metrics.fees));
    }

    pub(crate) fn validate_and_truncate(&mut self, version: Version, height: Height) -> Result<()> {
        self.output_count.validate_and_truncate(version, height)?;
        self.data_bytes.validate_and_truncate(version, height)?;
        self.tx_count.validate_and_truncate(version, height)?;
        self.tx_vsize.validate_and_truncate(version, height)?;
        self.fees.validate_and_truncate(version, height)?;
        Ok(())
    }

    pub(crate) fn truncate_if_needed_at(&mut self, len: usize) -> Result<()> {
        self.output_count.truncate_if_needed_at(len)?;
        self.data_bytes.truncate_if_needed_at(len)?;
        self.tx_count.truncate_if_needed_at(len)?;
        self.tx_vsize.truncate_if_needed_at(len)?;
        self.fees.truncate_if_needed_at(len)?;
        Ok(())
    }

    pub(crate) fn write(&mut self) -> Result<()> {
        self.output_count.write()?;
        self.data_bytes.write()?;
        self.tx_count.write()?;
        self.tx_vsize.write()?;
        self.fees.write()?;
        Ok(())
    }
}
