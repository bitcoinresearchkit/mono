use std::ops::{Add, AddAssign, Range};

#[cfg(feature = "storage")]
use bitview_traversable::Traversable;
use brk_types::Sats;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::{CohortId, CohortName};

/// Amount range bounds
const AMOUNT_RANGE_BOUNDS: AmountRange<Range<Sats>> = AmountRange {
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

#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema)]
#[cfg_attr(feature = "storage", derive(Traversable))]
pub struct AmountRange<T> {
    /// Uses zero-satoshi values.
    _0sats: T,
    /// Uses values of at least 1 and less than 10 satoshis.
    _1sat_to_10sats: T,
    /// Uses values of at least 10 and less than 100 satoshis.
    _10sats_to_100sats: T,
    /// Uses values of at least 100 and less than 1,000 satoshis.
    _100sats_to_1k_sats: T,
    /// Uses values of at least 1,000 and less than 10,000 satoshis.
    _1k_sats_to_10k_sats: T,
    /// Uses values of at least 10,000 and less than 100,000 satoshis.
    _10k_sats_to_100k_sats: T,
    /// Uses values of at least 100,000 and less than 1,000,000 satoshis.
    _100k_sats_to_1m_sats: T,
    /// Uses values of at least 1,000,000 and less than 10,000,000 satoshis.
    _1m_sats_to_10m_sats: T,
    /// Uses values of at least 10,000,000 satoshis and less than 1 BTC.
    _10m_sats_to_1btc: T,
    /// Uses values of at least 1 and less than 10 BTC.
    _1btc_to_10btc: T,
    /// Uses values of at least 10 and less than 100 BTC.
    _10btc_to_100btc: T,
    /// Uses values of at least 100 and less than 1,000 BTC.
    _100btc_to_1k_btc: T,
    /// Uses values of at least 1,000 and less than 10,000 BTC.
    _1k_btc_to_10k_btc: T,
    /// Uses values of at least 10,000 and less than 100,000 BTC.
    _10k_btc_to_100k_btc: T,
    /// Uses values of at least 100,000 BTC.
    over_100k_btc: T,
}

define_cohort_id!(
    AmountRangeId for AmountRange {
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

impl<T> AmountRange<T> {
    pub fn new(mut create: impl FnMut(CohortId) -> T) -> Self {
        Self::from_fn(|id| create(id.cohort()))
    }

    pub fn try_new<E>(mut create: impl FnMut(CohortId) -> Result<T, E>) -> Result<Self, E> {
        Self::try_from_fn(|id| create(id.cohort()))
    }

    #[inline(always)]
    pub fn get_mut(&mut self, value: Sats) -> &mut T {
        AmountRangeId::from(value).select_mut(self)
    }

    pub fn iter_typed(&self) -> impl Iterator<Item = (Sats, &T)> {
        AMOUNT_RANGE_BOUNDS
            .iter()
            .zip(self.iter())
            .map(|(bounds, value)| (bounds.start, value))
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

impl AmountRangeId {
    pub const fn cohort(self) -> CohortId {
        CohortId::Amount(self)
    }

    pub(crate) fn name(self) -> &'static CohortName {
        self.select(&AMOUNT_RANGE_NAMES)
    }
}

impl From<Sats> for AmountRangeId {
    #[inline(always)]
    fn from(value: Sats) -> Self {
        let value = u64::from(value);
        let index = if value == 0 {
            0
        } else {
            (value.ilog10() as usize + 1).min(Self::ALL.len() - 1)
        };
        Self::ALL[index]
    }
}
