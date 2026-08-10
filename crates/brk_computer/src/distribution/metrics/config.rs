use brk_cohort::Filter;
use brk_error::Result;
use brk_types::{
    Cents, Height, PartsPerMillion32, PartsPerMillionSigned32, PartsPerMillionSigned64, Version,
};
use schemars::JsonSchema;
use vecdb::{BytesVec, BytesVecValue, CachedBoxedVec, Database, ImportableVec};

use crate::{
    indexes,
    internal::{
        CachedWindowStartVec, ColumnarPercentRollingWindows, ColumnarRollingWindows,
        ColumnarRollingWindowsFrom1w, FiatPerBlock, FiatPerBlockCumulativeWithSums, FiatType,
        NumericValue, PerBlock, PerBlockCumulativeRolling, PercentPerBlock, Price,
        PriceWithRatioPerBlock, RatioPerBlock, RollingWindow24hPerBlock, RollingWindows,
        SpotValuePerBlock, ValuePerBlock, ValuePerBlockCumulative, ValuePerBlockCumulativeRolling,
        Windows,
    },
};

/// Trait for types importable via `ImportConfig::import`.
pub(crate) trait ConfigImport: Sized {
    fn config_import(cfg: &ImportConfig, suffix: &str, offset: Version) -> Result<Self>;
}

/// Implement `ConfigImport` for types whose `forced_import` takes `(db, name, version, indexes)`.
macro_rules! impl_config_import {
    ($($type:ty),+ $(,)?) => {
        $(
            impl ConfigImport for $type {
                fn config_import(cfg: &ImportConfig, suffix: &str, offset: Version) -> Result<Self> {
                    Self::forced_import(cfg.db, &cfg.name(suffix), cfg.version + offset, cfg.indexes)
                }
            }
        )+
    };
}

// Non-generic types
impl_config_import!(
    ValuePerBlock,
    ValuePerBlockCumulative,
    RatioPerBlock<PartsPerMillionSigned32>,
    PercentPerBlock<PartsPerMillion32>,
    PercentPerBlock<PartsPerMillionSigned32>,
    PercentPerBlock<PartsPerMillionSigned64>,
    ColumnarPercentRollingWindows<PartsPerMillion32>,
    Price<PerBlock<Cents>>,
);

impl ConfigImport for PriceWithRatioPerBlock {
    fn config_import(cfg: &ImportConfig, suffix: &str, offset: Version) -> Result<Self> {
        Self::forced_import(
            cfg.db,
            &cfg.name(suffix),
            cfg.version + offset,
            cfg.indexes,
            cfg.spot_price,
        )
    }
}

impl ConfigImport for SpotValuePerBlock {
    fn config_import(cfg: &ImportConfig, suffix: &str, offset: Version) -> Result<Self> {
        Self::forced_import(
            cfg.db,
            &cfg.name(suffix),
            cfg.version + offset,
            cfg.indexes,
            cfg.spot_price,
        )
    }
}

// Generic types (macro_rules can't parse generic bounds, so written out)
impl<T: NumericValue + JsonSchema> ConfigImport for PerBlock<T> {
    fn config_import(cfg: &ImportConfig, suffix: &str, offset: Version) -> Result<Self> {
        Self::forced_import(cfg.db, &cfg.name(suffix), cfg.version + offset, cfg.indexes)
    }
}
impl<T> ConfigImport for PerBlockCumulativeRolling<T>
where
    T: NumericValue + JsonSchema,
{
    fn config_import(cfg: &ImportConfig, suffix: &str, offset: Version) -> Result<Self> {
        Self::forced_import(
            cfg.db,
            &cfg.name(suffix),
            cfg.version + offset,
            cfg.indexes,
            cfg.cached_starts,
        )
    }
}
impl<T: NumericValue + JsonSchema> ConfigImport for RollingWindows<T> {
    fn config_import(cfg: &ImportConfig, suffix: &str, offset: Version) -> Result<Self> {
        Self::forced_import(cfg.db, &cfg.name(suffix), cfg.version + offset, cfg.indexes)
    }
}
impl<T: NumericValue + JsonSchema> ConfigImport for ColumnarRollingWindows<T> {
    fn config_import(cfg: &ImportConfig, suffix: &str, offset: Version) -> Result<Self> {
        Self::forced_import(cfg.db, &cfg.name(suffix), cfg.version + offset, cfg.indexes)
    }
}
impl<T: NumericValue + JsonSchema> ConfigImport for RollingWindow24hPerBlock<T> {
    fn config_import(cfg: &ImportConfig, suffix: &str, offset: Version) -> Result<Self> {
        Self::forced_import(cfg.db, &cfg.name(suffix), cfg.version + offset, cfg.indexes)
    }
}
impl ConfigImport for ValuePerBlockCumulativeRolling {
    fn config_import(cfg: &ImportConfig, suffix: &str, offset: Version) -> Result<Self> {
        Self::forced_import(
            cfg.db,
            &cfg.name(suffix),
            cfg.version + offset,
            cfg.indexes,
            cfg.cached_starts,
        )
    }
}
impl<C: FiatType> ConfigImport for FiatPerBlockCumulativeWithSums<C> {
    fn config_import(cfg: &ImportConfig, suffix: &str, offset: Version) -> Result<Self> {
        Self::forced_import(
            cfg.db,
            &cfg.name(suffix),
            cfg.version + offset,
            cfg.indexes,
            cfg.cached_starts,
        )
    }
}
impl<T: NumericValue + JsonSchema> ConfigImport for ColumnarRollingWindowsFrom1w<T> {
    fn config_import(cfg: &ImportConfig, suffix: &str, offset: Version) -> Result<Self> {
        Self::forced_import(cfg.db, &cfg.name(suffix), cfg.version + offset, cfg.indexes)
    }
}
impl<C: FiatType> ConfigImport for FiatPerBlock<C> {
    fn config_import(cfg: &ImportConfig, suffix: &str, offset: Version) -> Result<Self> {
        Self::forced_import(cfg.db, &cfg.name(suffix), cfg.version + offset, cfg.indexes)
    }
}
impl<T: BytesVecValue> ConfigImport for BytesVec<Height, T> {
    fn config_import(cfg: &ImportConfig, suffix: &str, offset: Version) -> Result<Self> {
        Ok(Self::forced_import(
            cfg.db,
            &cfg.name(suffix),
            cfg.version + offset,
        )?)
    }
}

#[derive(Clone, Copy)]
pub struct ImportConfig<'a> {
    pub db: &'a Database,
    pub filter: &'a Filter,
    pub full_name: &'a str,
    pub version: Version,
    pub indexes: &'a indexes::Vecs,
    pub cached_starts: &'a Windows<&'a CachedWindowStartVec>,
    pub spot_price: &'a CachedBoxedVec<Height, Cents>,
}

impl<'a> ImportConfig<'a> {
    pub(crate) fn name(&self, suffix: &str) -> String {
        if self.full_name.is_empty() {
            suffix.to_string()
        } else if suffix.is_empty() {
            self.full_name.to_string()
        } else {
            format!("{}_{suffix}", self.full_name)
        }
    }

    pub(crate) fn import<T: ConfigImport>(&self, suffix: &str, offset: Version) -> Result<T> {
        T::config_import(self, suffix, offset)
    }
}
