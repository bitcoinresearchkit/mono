use bitview_traversable::Traversable;
use brk_types::Sats;
use rayon::prelude::*;
use schemars::JsonSchema;
use serde::Serialize;

use super::{AmountFilter, CohortName, Filter};

/// Over-amount thresholds
pub const OVER_AMOUNT_THRESHOLDS: OverAmount<Sats> = OverAmount {
    _1sat: Sats::_1,
    _10sats: Sats::_10,
    _100sats: Sats::_100,
    _1k_sats: Sats::_1K,
    _10k_sats: Sats::_10K,
    _100k_sats: Sats::_100K,
    _1m_sats: Sats::_1M,
    _10m_sats: Sats::_10M,
    _1btc: Sats::_1BTC,
    _10btc: Sats::_10BTC,
    _100btc: Sats::_100BTC,
    _1k_btc: Sats::_1K_BTC,
    _10k_btc: Sats::_10K_BTC,
};

/// Over-amount names
pub const OVER_AMOUNT_NAMES: OverAmount<CohortName> = OverAmount {
    _1sat: CohortName::new("over_1sat", "1+ sats", "Over 1 Sat"),
    _10sats: CohortName::new("over_10sats", "10+ sats", "Over 10 Sats"),
    _100sats: CohortName::new("over_100sats", "100+ sats", "Over 100 Sats"),
    _1k_sats: CohortName::new("over_1k_sats", "1k+ sats", "Over 1K Sats"),
    _10k_sats: CohortName::new("over_10k_sats", "10k+ sats", "Over 10K Sats"),
    _100k_sats: CohortName::new("over_100k_sats", "100k+ sats", "Over 100K Sats"),
    _1m_sats: CohortName::new("over_1m_sats", "1M+ sats", "Over 1M Sats"),
    _10m_sats: CohortName::new("over_10m_sats", "0.1+ BTC", "Over 0.1 BTC"),
    _1btc: CohortName::new("over_1btc", "1+ BTC", "Over 1 BTC"),
    _10btc: CohortName::new("over_10btc", "10+ BTC", "Over 10 BTC"),
    _100btc: CohortName::new("over_100btc", "100+ BTC", "Over 100 BTC"),
    _1k_btc: CohortName::new("over_1k_btc", "1k+ BTC", "Over 1K BTC"),
    _10k_btc: CohortName::new("over_10k_btc", "10k+ BTC", "Over 10K BTC"),
};

/// Over-amount filters
pub const OVER_AMOUNT_FILTERS: OverAmount<Filter> = OverAmount {
    _1sat: Filter::Amount(AmountFilter::GreaterOrEqual(OVER_AMOUNT_THRESHOLDS._1sat)),
    _10sats: Filter::Amount(AmountFilter::GreaterOrEqual(OVER_AMOUNT_THRESHOLDS._10sats)),
    _100sats: Filter::Amount(AmountFilter::GreaterOrEqual(
        OVER_AMOUNT_THRESHOLDS._100sats,
    )),
    _1k_sats: Filter::Amount(AmountFilter::GreaterOrEqual(
        OVER_AMOUNT_THRESHOLDS._1k_sats,
    )),
    _10k_sats: Filter::Amount(AmountFilter::GreaterOrEqual(
        OVER_AMOUNT_THRESHOLDS._10k_sats,
    )),
    _100k_sats: Filter::Amount(AmountFilter::GreaterOrEqual(
        OVER_AMOUNT_THRESHOLDS._100k_sats,
    )),
    _1m_sats: Filter::Amount(AmountFilter::GreaterOrEqual(
        OVER_AMOUNT_THRESHOLDS._1m_sats,
    )),
    _10m_sats: Filter::Amount(AmountFilter::GreaterOrEqual(
        OVER_AMOUNT_THRESHOLDS._10m_sats,
    )),
    _1btc: Filter::Amount(AmountFilter::GreaterOrEqual(OVER_AMOUNT_THRESHOLDS._1btc)),
    _10btc: Filter::Amount(AmountFilter::GreaterOrEqual(OVER_AMOUNT_THRESHOLDS._10btc)),
    _100btc: Filter::Amount(AmountFilter::GreaterOrEqual(OVER_AMOUNT_THRESHOLDS._100btc)),
    _1k_btc: Filter::Amount(AmountFilter::GreaterOrEqual(OVER_AMOUNT_THRESHOLDS._1k_btc)),
    _10k_btc: Filter::Amount(AmountFilter::GreaterOrEqual(
        OVER_AMOUNT_THRESHOLDS._10k_btc,
    )),
};

#[derive(Debug, Default, Clone, Traversable, Serialize, JsonSchema)]
pub struct OverAmount<T> {
    /// Uses values of at least 1 satoshi.
    pub _1sat: T,
    /// Uses values of at least 10 satoshis.
    pub _10sats: T,
    /// Uses values of at least 100 satoshis.
    pub _100sats: T,
    /// Uses values of at least 1,000 satoshis.
    pub _1k_sats: T,
    /// Uses values of at least 10,000 satoshis.
    pub _10k_sats: T,
    /// Uses values of at least 100,000 satoshis.
    pub _100k_sats: T,
    /// Uses values of at least 1,000,000 satoshis.
    pub _1m_sats: T,
    /// Uses values of at least 10,000,000 satoshis.
    pub _10m_sats: T,
    /// Uses values of at least 1 BTC.
    pub _1btc: T,
    /// Uses values of at least 10 BTC.
    pub _10btc: T,
    /// Uses values of at least 100 BTC.
    pub _100btc: T,
    /// Uses values of at least 1,000 BTC.
    pub _1k_btc: T,
    /// Uses values of at least 10,000 BTC.
    pub _10k_btc: T,
}

define_column_id!(
    OverAmountId for OverAmount, version = 1 {
        Over1Sat => _1sat,
        Over10Sats => _10sats,
        Over100Sats => _100sats,
        Over1KSats => _1k_sats,
        Over10KSats => _10k_sats,
        Over100KSats => _100k_sats,
        Over1MSats => _1m_sats,
        Over10MSats => _10m_sats,
        Over1Btc => _1btc,
        Over10Btc => _10btc,
        Over100Btc => _100btc,
        Over1KBtc => _1k_btc,
        Over10KBtc => _10k_btc,
    }
);

impl OverAmount<CohortName> {
    pub const fn names() -> &'static Self {
        &OVER_AMOUNT_NAMES
    }
}

impl<T> OverAmount<T> {
    pub fn new<F>(mut create: F) -> Self
    where
        F: FnMut(Filter, &'static str) -> T,
    {
        let f = OVER_AMOUNT_FILTERS;
        let n = OVER_AMOUNT_NAMES;
        Self {
            _1sat: create(f._1sat.clone(), n._1sat.id),
            _10sats: create(f._10sats.clone(), n._10sats.id),
            _100sats: create(f._100sats.clone(), n._100sats.id),
            _1k_sats: create(f._1k_sats.clone(), n._1k_sats.id),
            _10k_sats: create(f._10k_sats.clone(), n._10k_sats.id),
            _100k_sats: create(f._100k_sats.clone(), n._100k_sats.id),
            _1m_sats: create(f._1m_sats.clone(), n._1m_sats.id),
            _10m_sats: create(f._10m_sats.clone(), n._10m_sats.id),
            _1btc: create(f._1btc.clone(), n._1btc.id),
            _10btc: create(f._10btc.clone(), n._10btc.id),
            _100btc: create(f._100btc.clone(), n._100btc.id),
            _1k_btc: create(f._1k_btc.clone(), n._1k_btc.id),
            _10k_btc: create(f._10k_btc.clone(), n._10k_btc.id),
        }
    }

    pub fn try_new<F, E>(mut create: F) -> Result<Self, E>
    where
        F: FnMut(Filter, &'static str) -> Result<T, E>,
    {
        let f = OVER_AMOUNT_FILTERS;
        let n = OVER_AMOUNT_NAMES;
        Ok(Self {
            _1sat: create(f._1sat.clone(), n._1sat.id)?,
            _10sats: create(f._10sats.clone(), n._10sats.id)?,
            _100sats: create(f._100sats.clone(), n._100sats.id)?,
            _1k_sats: create(f._1k_sats.clone(), n._1k_sats.id)?,
            _10k_sats: create(f._10k_sats.clone(), n._10k_sats.id)?,
            _100k_sats: create(f._100k_sats.clone(), n._100k_sats.id)?,
            _1m_sats: create(f._1m_sats.clone(), n._1m_sats.id)?,
            _10m_sats: create(f._10m_sats.clone(), n._10m_sats.id)?,
            _1btc: create(f._1btc.clone(), n._1btc.id)?,
            _10btc: create(f._10btc.clone(), n._10btc.id)?,
            _100btc: create(f._100btc.clone(), n._100btc.id)?,
            _1k_btc: create(f._1k_btc.clone(), n._1k_btc.id)?,
            _10k_btc: create(f._10k_btc.clone(), n._10k_btc.id)?,
        })
    }

    pub fn iter(&self) -> impl Iterator<Item = &T> {
        [
            &self._1sat,
            &self._10sats,
            &self._100sats,
            &self._1k_sats,
            &self._10k_sats,
            &self._100k_sats,
            &self._1m_sats,
            &self._10m_sats,
            &self._1btc,
            &self._10btc,
            &self._100btc,
            &self._1k_btc,
            &self._10k_btc,
        ]
        .into_iter()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut T> {
        [
            &mut self._1sat,
            &mut self._10sats,
            &mut self._100sats,
            &mut self._1k_sats,
            &mut self._10k_sats,
            &mut self._100k_sats,
            &mut self._1m_sats,
            &mut self._10m_sats,
            &mut self._1btc,
            &mut self._10btc,
            &mut self._100btc,
            &mut self._1k_btc,
            &mut self._10k_btc,
        ]
        .into_iter()
    }

    pub fn par_iter_mut(&mut self) -> impl ParallelIterator<Item = &mut T>
    where
        T: Send + Sync,
    {
        [
            &mut self._1sat,
            &mut self._10sats,
            &mut self._100sats,
            &mut self._1k_sats,
            &mut self._10k_sats,
            &mut self._100k_sats,
            &mut self._1m_sats,
            &mut self._10m_sats,
            &mut self._1btc,
            &mut self._10btc,
            &mut self._100btc,
            &mut self._1k_btc,
            &mut self._10k_btc,
        ]
        .into_par_iter()
    }
}
