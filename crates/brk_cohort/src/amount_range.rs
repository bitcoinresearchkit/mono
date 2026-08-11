use std::ops::{Add, AddAssign, Range};

use brk_traversable::Traversable;
use brk_types::Sats;
use rayon::prelude::*;
use schemars::JsonSchema;
use serde::Serialize;
use vecdb::ColumnId;

use super::{AmountBucket, AmountFilter, CohortName, Filter};

/// Amount range bounds
pub const AMOUNT_RANGE_BOUNDS: AmountRange<Range<Sats>> = AmountRange {
    _0sats: Sats::ZERO..Sats::_1,
    _1sat_to_10sats: Sats::_1..Sats::_10,
    _10sats_to_100sats: Sats::_10..Sats::_100,
    _100sats_to_1k_sats: Sats::_100..Sats::_1K,
    _1k_sats_to_10k_sats: Sats::_1K..Sats::_10K,
    _10k_sats_to_100k_sats: Sats::_10K..Sats::_100K,
    _100k_sats_to_1m_sats: Sats::_100K..Sats::_1M,
    _1m_sats_to_10m_sats: Sats::_1M..Sats::_10M,
    _10m_sats_to_1btc: Sats::_10M..Sats::_1BTC,
    _1btc_to_10btc: Sats::_1BTC..Sats::_10BTC,
    _10btc_to_100btc: Sats::_10BTC..Sats::_100BTC,
    _100btc_to_1k_btc: Sats::_100BTC..Sats::_1K_BTC,
    _1k_btc_to_10k_btc: Sats::_1K_BTC..Sats::_10K_BTC,
    _10k_btc_to_100k_btc: Sats::_10K_BTC..Sats::_100K_BTC,
    over_100k_btc: Sats::_100K_BTC..Sats::MAX,
};

/// Amount range names
pub const AMOUNT_RANGE_NAMES: AmountRange<CohortName> = AmountRange {
    _0sats: CohortName::new("0sats", "0 sats", "0 Sats"),
    _1sat_to_10sats: CohortName::new("1sat_to_10sats", "1-10 sats", "1-10 Sats"),
    _10sats_to_100sats: CohortName::new("10sats_to_100sats", "10-100 sats", "10-100 Sats"),
    _100sats_to_1k_sats: CohortName::new("100sats_to_1k_sats", "100-1k sats", "100-1K Sats"),
    _1k_sats_to_10k_sats: CohortName::new("1k_sats_to_10k_sats", "1k-10k sats", "1K-10K Sats"),
    _10k_sats_to_100k_sats: CohortName::new(
        "10k_sats_to_100k_sats",
        "10k-100k sats",
        "10K-100K Sats",
    ),
    _100k_sats_to_1m_sats: CohortName::new("100k_sats_to_1m_sats", "100k-1M sats", "100K-1M Sats"),
    _1m_sats_to_10m_sats: CohortName::new("1m_sats_to_10m_sats", "1M-10M sats", "1M-10M Sats"),
    _10m_sats_to_1btc: CohortName::new("10m_sats_to_1btc", "0.1-1 BTC", "0.1-1 BTC"),
    _1btc_to_10btc: CohortName::new("1btc_to_10btc", "1-10 BTC", "1-10 BTC"),
    _10btc_to_100btc: CohortName::new("10btc_to_100btc", "10-100 BTC", "10-100 BTC"),
    _100btc_to_1k_btc: CohortName::new("100btc_to_1k_btc", "100-1k BTC", "100-1K BTC"),
    _1k_btc_to_10k_btc: CohortName::new("1k_btc_to_10k_btc", "1k-10k BTC", "1K-10K BTC"),
    _10k_btc_to_100k_btc: CohortName::new("10k_btc_to_100k_btc", "10k-100k BTC", "10K-100K BTC"),
    over_100k_btc: CohortName::new("over_100k_btc", "100k+ BTC", "100K+ BTC"),
};

/// Amount range filters
pub const AMOUNT_RANGE_FILTERS: AmountRange<Filter> = AmountRange {
    _0sats: Filter::Amount(AmountFilter::Range(AMOUNT_RANGE_BOUNDS._0sats)),
    _1sat_to_10sats: Filter::Amount(AmountFilter::Range(AMOUNT_RANGE_BOUNDS._1sat_to_10sats)),
    _10sats_to_100sats: Filter::Amount(AmountFilter::Range(AMOUNT_RANGE_BOUNDS._10sats_to_100sats)),
    _100sats_to_1k_sats: Filter::Amount(AmountFilter::Range(
        AMOUNT_RANGE_BOUNDS._100sats_to_1k_sats,
    )),
    _1k_sats_to_10k_sats: Filter::Amount(AmountFilter::Range(
        AMOUNT_RANGE_BOUNDS._1k_sats_to_10k_sats,
    )),
    _10k_sats_to_100k_sats: Filter::Amount(AmountFilter::Range(
        AMOUNT_RANGE_BOUNDS._10k_sats_to_100k_sats,
    )),
    _100k_sats_to_1m_sats: Filter::Amount(AmountFilter::Range(
        AMOUNT_RANGE_BOUNDS._100k_sats_to_1m_sats,
    )),
    _1m_sats_to_10m_sats: Filter::Amount(AmountFilter::Range(
        AMOUNT_RANGE_BOUNDS._1m_sats_to_10m_sats,
    )),
    _10m_sats_to_1btc: Filter::Amount(AmountFilter::Range(AMOUNT_RANGE_BOUNDS._10m_sats_to_1btc)),
    _1btc_to_10btc: Filter::Amount(AmountFilter::Range(AMOUNT_RANGE_BOUNDS._1btc_to_10btc)),
    _10btc_to_100btc: Filter::Amount(AmountFilter::Range(AMOUNT_RANGE_BOUNDS._10btc_to_100btc)),
    _100btc_to_1k_btc: Filter::Amount(AmountFilter::Range(AMOUNT_RANGE_BOUNDS._100btc_to_1k_btc)),
    _1k_btc_to_10k_btc: Filter::Amount(AmountFilter::Range(AMOUNT_RANGE_BOUNDS._1k_btc_to_10k_btc)),
    _10k_btc_to_100k_btc: Filter::Amount(AmountFilter::Range(
        AMOUNT_RANGE_BOUNDS._10k_btc_to_100k_btc,
    )),
    over_100k_btc: Filter::Amount(AmountFilter::Range(AMOUNT_RANGE_BOUNDS.over_100k_btc)),
};

#[derive(Debug, Default, Clone, Traversable, Serialize, JsonSchema)]
pub struct AmountRange<T> {
    pub _0sats: T,
    pub _1sat_to_10sats: T,
    pub _10sats_to_100sats: T,
    pub _100sats_to_1k_sats: T,
    pub _1k_sats_to_10k_sats: T,
    pub _10k_sats_to_100k_sats: T,
    pub _100k_sats_to_1m_sats: T,
    pub _1m_sats_to_10m_sats: T,
    pub _10m_sats_to_1btc: T,
    pub _1btc_to_10btc: T,
    pub _10btc_to_100btc: T,
    pub _100btc_to_1k_btc: T,
    pub _1k_btc_to_10k_btc: T,
    pub _10k_btc_to_100k_btc: T,
    pub over_100k_btc: T,
}

define_column_id!(
    AmountRangeId for AmountRange, version = 1 {
        Zero => _0sats,
        From1SatTo10Sats => _1sat_to_10sats,
        From10SatsTo100Sats => _10sats_to_100sats,
        From100SatsTo1KSats => _100sats_to_1k_sats,
        From1KSatsTo10KSats => _1k_sats_to_10k_sats,
        From10KSatsTo100KSats => _10k_sats_to_100k_sats,
        From100KSatsTo1MSats => _100k_sats_to_1m_sats,
        From1MSatsTo10MSats => _1m_sats_to_10m_sats,
        From10MSatsTo1Btc => _10m_sats_to_1btc,
        From1BtcTo10Btc => _1btc_to_10btc,
        From10BtcTo100Btc => _10btc_to_100btc,
        From100BtcTo1KBtc => _100btc_to_1k_btc,
        From1KBtcTo10KBtc => _1k_btc_to_10k_btc,
        From10KBtcTo100KBtc => _10k_btc_to_100k_btc,
        Over100kBtc => over_100k_btc,
    }
);

impl AmountRangeId {
    #[inline]
    pub fn filter(self) -> &'static Filter {
        self.select(&AMOUNT_RANGE_FILTERS)
    }

    pub fn matching(filter: &Filter) -> Option<Self> {
        Self::ALL
            .iter()
            .copied()
            .find(|column| column.filter() == filter)
    }

    pub fn included_by(filter: &Filter) -> impl Iterator<Item = Self> + '_ {
        Self::ALL
            .iter()
            .copied()
            .filter(|column| filter.includes(column.filter()))
    }
}

impl AmountRange<CohortName> {
    pub const fn names() -> &'static Self {
        &AMOUNT_RANGE_NAMES
    }
}

impl<T> AmountRange<T> {
    pub fn new<F>(mut create: F) -> Self
    where
        F: FnMut(Filter, &'static str) -> T,
    {
        let f = AMOUNT_RANGE_FILTERS;
        let n = AMOUNT_RANGE_NAMES;
        Self {
            _0sats: create(f._0sats.clone(), n._0sats.id),
            _1sat_to_10sats: create(f._1sat_to_10sats.clone(), n._1sat_to_10sats.id),
            _10sats_to_100sats: create(f._10sats_to_100sats.clone(), n._10sats_to_100sats.id),
            _100sats_to_1k_sats: create(f._100sats_to_1k_sats.clone(), n._100sats_to_1k_sats.id),
            _1k_sats_to_10k_sats: create(f._1k_sats_to_10k_sats.clone(), n._1k_sats_to_10k_sats.id),
            _10k_sats_to_100k_sats: create(
                f._10k_sats_to_100k_sats.clone(),
                n._10k_sats_to_100k_sats.id,
            ),
            _100k_sats_to_1m_sats: create(
                f._100k_sats_to_1m_sats.clone(),
                n._100k_sats_to_1m_sats.id,
            ),
            _1m_sats_to_10m_sats: create(f._1m_sats_to_10m_sats.clone(), n._1m_sats_to_10m_sats.id),
            _10m_sats_to_1btc: create(f._10m_sats_to_1btc.clone(), n._10m_sats_to_1btc.id),
            _1btc_to_10btc: create(f._1btc_to_10btc.clone(), n._1btc_to_10btc.id),
            _10btc_to_100btc: create(f._10btc_to_100btc.clone(), n._10btc_to_100btc.id),
            _100btc_to_1k_btc: create(f._100btc_to_1k_btc.clone(), n._100btc_to_1k_btc.id),
            _1k_btc_to_10k_btc: create(f._1k_btc_to_10k_btc.clone(), n._1k_btc_to_10k_btc.id),
            _10k_btc_to_100k_btc: create(f._10k_btc_to_100k_btc.clone(), n._10k_btc_to_100k_btc.id),
            over_100k_btc: create(f.over_100k_btc.clone(), n.over_100k_btc.id),
        }
    }

    pub fn try_new<F, E>(mut create: F) -> Result<Self, E>
    where
        F: FnMut(Filter, &'static str) -> Result<T, E>,
    {
        let f = AMOUNT_RANGE_FILTERS;
        let n = AMOUNT_RANGE_NAMES;
        Ok(Self {
            _0sats: create(f._0sats.clone(), n._0sats.id)?,
            _1sat_to_10sats: create(f._1sat_to_10sats.clone(), n._1sat_to_10sats.id)?,
            _10sats_to_100sats: create(f._10sats_to_100sats.clone(), n._10sats_to_100sats.id)?,
            _100sats_to_1k_sats: create(f._100sats_to_1k_sats.clone(), n._100sats_to_1k_sats.id)?,
            _1k_sats_to_10k_sats: create(
                f._1k_sats_to_10k_sats.clone(),
                n._1k_sats_to_10k_sats.id,
            )?,
            _10k_sats_to_100k_sats: create(
                f._10k_sats_to_100k_sats.clone(),
                n._10k_sats_to_100k_sats.id,
            )?,
            _100k_sats_to_1m_sats: create(
                f._100k_sats_to_1m_sats.clone(),
                n._100k_sats_to_1m_sats.id,
            )?,
            _1m_sats_to_10m_sats: create(
                f._1m_sats_to_10m_sats.clone(),
                n._1m_sats_to_10m_sats.id,
            )?,
            _10m_sats_to_1btc: create(f._10m_sats_to_1btc.clone(), n._10m_sats_to_1btc.id)?,
            _1btc_to_10btc: create(f._1btc_to_10btc.clone(), n._1btc_to_10btc.id)?,
            _10btc_to_100btc: create(f._10btc_to_100btc.clone(), n._10btc_to_100btc.id)?,
            _100btc_to_1k_btc: create(f._100btc_to_1k_btc.clone(), n._100btc_to_1k_btc.id)?,
            _1k_btc_to_10k_btc: create(f._1k_btc_to_10k_btc.clone(), n._1k_btc_to_10k_btc.id)?,
            _10k_btc_to_100k_btc: create(
                f._10k_btc_to_100k_btc.clone(),
                n._10k_btc_to_100k_btc.id,
            )?,
            over_100k_btc: create(f.over_100k_btc.clone(), n.over_100k_btc.id)?,
        })
    }

    #[inline(always)]
    pub fn get(&self, value: Sats) -> &T {
        match AmountBucket::from(value).index() {
            0 => &self._0sats,
            1 => &self._1sat_to_10sats,
            2 => &self._10sats_to_100sats,
            3 => &self._100sats_to_1k_sats,
            4 => &self._1k_sats_to_10k_sats,
            5 => &self._10k_sats_to_100k_sats,
            6 => &self._100k_sats_to_1m_sats,
            7 => &self._1m_sats_to_10m_sats,
            8 => &self._10m_sats_to_1btc,
            9 => &self._1btc_to_10btc,
            10 => &self._10btc_to_100btc,
            11 => &self._100btc_to_1k_btc,
            12 => &self._1k_btc_to_10k_btc,
            13 => &self._10k_btc_to_100k_btc,
            _ => &self.over_100k_btc,
        }
    }

    #[inline(always)]
    pub fn get_mut(&mut self, value: Sats) -> &mut T {
        self.get_mut_by_bucket(AmountBucket::from(value))
    }

    /// Get mutable reference by pre-computed bucket index.
    /// Use with `AmountBucket::transition_to` to avoid recomputing bucket.
    #[inline(always)]
    pub fn get_mut_by_bucket(&mut self, bucket: AmountBucket) -> &mut T {
        match bucket.index() {
            0 => &mut self._0sats,
            1 => &mut self._1sat_to_10sats,
            2 => &mut self._10sats_to_100sats,
            3 => &mut self._100sats_to_1k_sats,
            4 => &mut self._1k_sats_to_10k_sats,
            5 => &mut self._10k_sats_to_100k_sats,
            6 => &mut self._100k_sats_to_1m_sats,
            7 => &mut self._1m_sats_to_10m_sats,
            8 => &mut self._10m_sats_to_1btc,
            9 => &mut self._1btc_to_10btc,
            10 => &mut self._10btc_to_100btc,
            11 => &mut self._100btc_to_1k_btc,
            12 => &mut self._1k_btc_to_10k_btc,
            13 => &mut self._10k_btc_to_100k_btc,
            _ => &mut self.over_100k_btc,
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = &T> {
        [
            &self._0sats,
            &self._1sat_to_10sats,
            &self._10sats_to_100sats,
            &self._100sats_to_1k_sats,
            &self._1k_sats_to_10k_sats,
            &self._10k_sats_to_100k_sats,
            &self._100k_sats_to_1m_sats,
            &self._1m_sats_to_10m_sats,
            &self._10m_sats_to_1btc,
            &self._1btc_to_10btc,
            &self._10btc_to_100btc,
            &self._100btc_to_1k_btc,
            &self._1k_btc_to_10k_btc,
            &self._10k_btc_to_100k_btc,
            &self.over_100k_btc,
        ]
        .into_iter()
    }

    pub fn iter_typed(&self) -> impl Iterator<Item = (Sats, &T)> {
        [
            (Sats::ZERO, &self._0sats),
            (Sats::_1, &self._1sat_to_10sats),
            (Sats::_10, &self._10sats_to_100sats),
            (Sats::_100, &self._100sats_to_1k_sats),
            (Sats::_1K, &self._1k_sats_to_10k_sats),
            (Sats::_10K, &self._10k_sats_to_100k_sats),
            (Sats::_100K, &self._100k_sats_to_1m_sats),
            (Sats::_1M, &self._1m_sats_to_10m_sats),
            (Sats::_10M, &self._10m_sats_to_1btc),
            (Sats::_1BTC, &self._1btc_to_10btc),
            (Sats::_10BTC, &self._10btc_to_100btc),
            (Sats::_100BTC, &self._100btc_to_1k_btc),
            (Sats::_1K_BTC, &self._1k_btc_to_10k_btc),
            (Sats::_10K_BTC, &self._10k_btc_to_100k_btc),
            (Sats::_100K_BTC, &self.over_100k_btc),
        ]
        .into_iter()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut T> {
        [
            &mut self._0sats,
            &mut self._1sat_to_10sats,
            &mut self._10sats_to_100sats,
            &mut self._100sats_to_1k_sats,
            &mut self._1k_sats_to_10k_sats,
            &mut self._10k_sats_to_100k_sats,
            &mut self._100k_sats_to_1m_sats,
            &mut self._1m_sats_to_10m_sats,
            &mut self._10m_sats_to_1btc,
            &mut self._1btc_to_10btc,
            &mut self._10btc_to_100btc,
            &mut self._100btc_to_1k_btc,
            &mut self._1k_btc_to_10k_btc,
            &mut self._10k_btc_to_100k_btc,
            &mut self.over_100k_btc,
        ]
        .into_iter()
    }

    pub fn par_iter_mut(&mut self) -> impl ParallelIterator<Item = &mut T>
    where
        T: Send + Sync,
    {
        [
            &mut self._0sats,
            &mut self._1sat_to_10sats,
            &mut self._10sats_to_100sats,
            &mut self._100sats_to_1k_sats,
            &mut self._1k_sats_to_10k_sats,
            &mut self._10k_sats_to_100k_sats,
            &mut self._100k_sats_to_1m_sats,
            &mut self._1m_sats_to_10m_sats,
            &mut self._10m_sats_to_1btc,
            &mut self._1btc_to_10btc,
            &mut self._10btc_to_100btc,
            &mut self._100btc_to_1k_btc,
            &mut self._1k_btc_to_10k_btc,
            &mut self._10k_btc_to_100k_btc,
            &mut self.over_100k_btc,
        ]
        .into_par_iter()
    }
}

impl<T> Add for AmountRange<T>
where
    T: Add<Output = T>,
{
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Self {
            _0sats: self._0sats + rhs._0sats,
            _1sat_to_10sats: self._1sat_to_10sats + rhs._1sat_to_10sats,
            _10sats_to_100sats: self._10sats_to_100sats + rhs._10sats_to_100sats,
            _100sats_to_1k_sats: self._100sats_to_1k_sats + rhs._100sats_to_1k_sats,
            _1k_sats_to_10k_sats: self._1k_sats_to_10k_sats + rhs._1k_sats_to_10k_sats,
            _10k_sats_to_100k_sats: self._10k_sats_to_100k_sats + rhs._10k_sats_to_100k_sats,
            _100k_sats_to_1m_sats: self._100k_sats_to_1m_sats + rhs._100k_sats_to_1m_sats,
            _1m_sats_to_10m_sats: self._1m_sats_to_10m_sats + rhs._1m_sats_to_10m_sats,
            _10m_sats_to_1btc: self._10m_sats_to_1btc + rhs._10m_sats_to_1btc,
            _1btc_to_10btc: self._1btc_to_10btc + rhs._1btc_to_10btc,
            _10btc_to_100btc: self._10btc_to_100btc + rhs._10btc_to_100btc,
            _100btc_to_1k_btc: self._100btc_to_1k_btc + rhs._100btc_to_1k_btc,
            _1k_btc_to_10k_btc: self._1k_btc_to_10k_btc + rhs._1k_btc_to_10k_btc,
            _10k_btc_to_100k_btc: self._10k_btc_to_100k_btc + rhs._10k_btc_to_100k_btc,
            over_100k_btc: self.over_100k_btc + rhs.over_100k_btc,
        }
    }
}

impl<T> AddAssign for AmountRange<T>
where
    T: AddAssign,
{
    fn add_assign(&mut self, rhs: Self) {
        self._0sats += rhs._0sats;
        self._1sat_to_10sats += rhs._1sat_to_10sats;
        self._10sats_to_100sats += rhs._10sats_to_100sats;
        self._100sats_to_1k_sats += rhs._100sats_to_1k_sats;
        self._1k_sats_to_10k_sats += rhs._1k_sats_to_10k_sats;
        self._10k_sats_to_100k_sats += rhs._10k_sats_to_100k_sats;
        self._100k_sats_to_1m_sats += rhs._100k_sats_to_1m_sats;
        self._1m_sats_to_10m_sats += rhs._1m_sats_to_10m_sats;
        self._10m_sats_to_1btc += rhs._10m_sats_to_1btc;
        self._1btc_to_10btc += rhs._1btc_to_10btc;
        self._10btc_to_100btc += rhs._10btc_to_100btc;
        self._100btc_to_1k_btc += rhs._100btc_to_1k_btc;
        self._1k_btc_to_10k_btc += rhs._1k_btc_to_10k_btc;
        self._10k_btc_to_100k_btc += rhs._10k_btc_to_100k_btc;
        self.over_100k_btc += rhs.over_100k_btc;
    }
}
