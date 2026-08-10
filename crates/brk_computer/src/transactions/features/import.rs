use brk_error::Result;
use brk_types::Version;
use vecdb::{Database, ReadOnlyClone};

use super::{CountVecs, FeatureId, Vecs};
use crate::{
    indexes,
    internal::{
        CachedWindowStartVec, ColumnarPerBlockCumulativeRolling,
        LazyColumnPerBlockCumulativeRolling, Windows,
    },
};

impl Vecs {
    pub(crate) fn forced_import(
        db: &Database,
        version: Version,
        indexes: &indexes::Vecs,
        cached_starts: &Windows<&CachedWindowStartVec>,
    ) -> Result<Self> {
        let source = ColumnarPerBlockCumulativeRolling::forced_import(
            db,
            "tx_feature_count_cumulative",
            version,
            |_| (),
        )?;
        let counts = source.cumulative.read_only_clone();
        let import = |name, feature| {
            LazyColumnPerBlockCumulativeRolling::new(
                name,
                version,
                &counts,
                feature,
                indexes,
                cached_starts,
            )
        };

        Ok(Self {
            count: CountVecs {
                inscription: import("tx_count_inscription", FeatureId::Inscription),
                annex: import("tx_count_annex", FeatureId::Annex),
                sighash_all: import("tx_count_sighash_all", FeatureId::SighashAll),
                sighash_none: import("tx_count_sighash_none", FeatureId::SighashNone),
                sighash_single: import("tx_count_sighash_single", FeatureId::SighashSingle),
                sighash_default: import("tx_count_sighash_default", FeatureId::SighashDefault),
                sighash_anyone_can_pay: import(
                    "tx_count_sighash_anyone_can_pay",
                    FeatureId::SighashAnyoneCanPay,
                ),
                dust_output: import("tx_count_dust_output", FeatureId::DustOutput),
                source,
            },
        })
    }
}
