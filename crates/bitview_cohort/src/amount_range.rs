use std::ops::Range;

#[cfg(feature = "storage")]
use bitview_traversable::Traversable;
use brk_types::Sats;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::{CohortId, CohortName};

const AMOUNT_RANGE_BOUNDS: AmountRange<Range<Sats>> = AmountRange {
    _0_sats: Sats::ZERO..Sats::_1,
    _1_to_10_sats: Sats::_1..Sats::_10,
    _10_to_100_sats: Sats::_10..Sats::_100,
    _100_to_1k_sats: Sats::_100..Sats::_1K,
    _1k_to_10k_sats: Sats::_1K..Sats::_10K,
    _10k_to_100k_sats: Sats::_10K..Sats::_100K,
    _100k_to_1m_sats: Sats::_100K..Sats::_1M,
    _1m_to_10m_sats: Sats::_1M..Sats::_10M,
    _10m_to_100m_sats: Sats::_10M..Sats::_1BTC,
    _1_to_10_btc: Sats::_1BTC..Sats::_10BTC,
    _10_to_100_btc: Sats::_10BTC..Sats::_100BTC,
    _100_to_1k_btc: Sats::_100BTC..Sats::_1K_BTC,
    _1k_to_10k_btc: Sats::_1K_BTC..Sats::_10K_BTC,
    _10k_to_100k_btc: Sats::_10K_BTC..Sats::_100K_BTC,
    over_100k_btc: Sats::_100K_BTC..Sats::MAX,
};

pub const AMOUNT_RANGE_NAMES: AmountRange<CohortName> = AmountRange {
    _0_sats: CohortName::new("0_sats", "0 sats", "0 Sats"),
    _1_to_10_sats: CohortName::new("1_to_10_sats", "1-10 sats", "1-10 Sats"),
    _10_to_100_sats: CohortName::new("10_to_100_sats", "10-100 sats", "10-100 Sats"),
    _100_to_1k_sats: CohortName::new("100_to_1k_sats", "100-1k sats", "100-1K Sats"),
    _1k_to_10k_sats: CohortName::new("1k_to_10k_sats", "1k-10k sats", "1K-10K Sats"),
    _10k_to_100k_sats: CohortName::new("10k_to_100k_sats", "10k-100k sats", "10K-100K Sats"),
    _100k_to_1m_sats: CohortName::new("100k_to_1m_sats", "100k-1M sats", "100K-1M Sats"),
    _1m_to_10m_sats: CohortName::new("1m_to_10m_sats", "1M-10M sats", "1M-10M Sats"),
    _10m_to_100m_sats: CohortName::new("10m_to_100m_sats", "0.1-1 BTC", "0.1-1 BTC"),
    _1_to_10_btc: CohortName::new("1_to_10_btc", "1-10 BTC", "1-10 BTC"),
    _10_to_100_btc: CohortName::new("10_to_100_btc", "10-100 BTC", "10-100 BTC"),
    _100_to_1k_btc: CohortName::new("100_to_1k_btc", "100-1k BTC", "100-1K BTC"),
    _1k_to_10k_btc: CohortName::new("1k_to_10k_btc", "1k-10k BTC", "1K-10K BTC"),
    _10k_to_100k_btc: CohortName::new("10k_to_100k_btc", "10k-100k BTC", "10K-100K BTC"),
    over_100k_btc: CohortName::new("over_100k_btc", "100k+ BTC", "100K+ BTC"),
};

#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema)]
#[cfg_attr(feature = "storage", derive(Traversable))]
/// The 15 amount bands. `Z`, the zero band's type, is the others' unless a family's zero band
/// holds less (its values all zero).
pub struct AmountRange<T, Z = T> {
    /// Uses zero-satoshi values.
    #[cfg_attr(feature = "storage", traversable(lead = "0 sats."))]
    _0_sats: Z,
    /// Uses values of at least 1 and less than 10 satoshis.
    #[cfg_attr(feature = "storage", traversable(lead = "1 to 10 sats."))]
    _1_to_10_sats: T,
    /// Uses values of at least 10 and less than 100 satoshis.
    #[cfg_attr(feature = "storage", traversable(lead = "10 to 100 sats."))]
    _10_to_100_sats: T,
    /// Uses values of at least 100 and less than 1,000 satoshis.
    #[cfg_attr(feature = "storage", traversable(lead = "100 to 1,000 sats."))]
    _100_to_1k_sats: T,
    /// Uses values of at least 1,000 and less than 10,000 satoshis.
    #[cfg_attr(feature = "storage", traversable(lead = "1,000 to 10,000 sats."))]
    _1k_to_10k_sats: T,
    /// Uses values of at least 10,000 and less than 100,000 satoshis.
    #[cfg_attr(feature = "storage", traversable(lead = "10,000 to 100,000 sats."))]
    _10k_to_100k_sats: T,
    /// Uses values of at least 100,000 and less than 1,000,000 satoshis.
    #[cfg_attr(feature = "storage", traversable(lead = "100,000 sats to 0.01 BTC."))]
    _100k_to_1m_sats: T,
    /// Uses values of at least 1,000,000 and less than 10,000,000 satoshis.
    #[cfg_attr(feature = "storage", traversable(lead = "0.01 to 0.1 BTC."))]
    _1m_to_10m_sats: T,
    /// Uses values of at least 10,000,000 satoshis and less than 1 BTC.
    #[cfg_attr(feature = "storage", traversable(lead = "0.1 to 1 BTC."))]
    _10m_to_100m_sats: T,
    /// Uses values of at least 1 and less than 10 BTC.
    #[cfg_attr(feature = "storage", traversable(lead = "1 to 10 BTC."))]
    _1_to_10_btc: T,
    /// Uses values of at least 10 and less than 100 BTC.
    #[cfg_attr(feature = "storage", traversable(lead = "10 to 100 BTC."))]
    _10_to_100_btc: T,
    /// Uses values of at least 100 and less than 1,000 BTC.
    #[cfg_attr(feature = "storage", traversable(lead = "100 to 1,000 BTC."))]
    _100_to_1k_btc: T,
    /// Uses values of at least 1,000 and less than 10,000 BTC.
    #[cfg_attr(feature = "storage", traversable(lead = "1,000 to 10,000 BTC."))]
    _1k_to_10k_btc: T,
    /// Uses values of at least 10,000 and less than 100,000 BTC.
    #[cfg_attr(feature = "storage", traversable(lead = "10,000 to 100,000 BTC."))]
    _10k_to_100k_btc: T,
    /// Uses values of at least 100,000 BTC.
    #[cfg_attr(feature = "storage", traversable(lead = "100,000 BTC or more."))]
    over_100k_btc: T,
}

define_cohort_id!(
    AmountRangeId for AmountRange {
        Zero => _0_sats,
        From1SatTo10Sats => _1_to_10_sats,
        From10SatsTo100Sats => _10_to_100_sats,
        From100SatsTo1KSats => _100_to_1k_sats,
        From1KSatsTo10KSats => _1k_to_10k_sats,
        From10KSatsTo100KSats => _10k_to_100k_sats,
        From100KSatsTo1MSats => _100k_to_1m_sats,
        From1MSatsTo10MSats => _1m_to_10m_sats,
        From10MTo100MSats => _10m_to_100m_sats,
        From1BtcTo10Btc => _1_to_10_btc,
        From10BtcTo100Btc => _10_to_100_btc,
        From100BtcTo1KBtc => _100_to_1k_btc,
        From1KBtcTo10KBtc => _1k_to_10k_btc,
        From10KBtcTo100KBtc => _10k_to_100k_btc,
        Over100kBtc => over_100k_btc,
    }
);

impl<T> AmountRange<T> {
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

impl<T, Z> AmountRange<T, Z> {
    /// The zero band and the 14 others, apart.
    pub fn split(&self) -> (&Z, [&T; 14]) {
        (
            &self._0_sats,
            [
                &self._1_to_10_sats,
                &self._10_to_100_sats,
                &self._100_to_1k_sats,
                &self._1k_to_10k_sats,
                &self._10k_to_100k_sats,
                &self._100k_to_1m_sats,
                &self._1m_to_10m_sats,
                &self._10m_to_100m_sats,
                &self._1_to_10_btc,
                &self._10_to_100_btc,
                &self._100_to_1k_btc,
                &self._1k_to_10k_btc,
                &self._10k_to_100k_btc,
                &self.over_100k_btc,
            ],
        )
    }

    pub fn split_mut(&mut self) -> (&mut Z, [&mut T; 14]) {
        (
            &mut self._0_sats,
            [
                &mut self._1_to_10_sats,
                &mut self._10_to_100_sats,
                &mut self._100_to_1k_sats,
                &mut self._1k_to_10k_sats,
                &mut self._10k_to_100k_sats,
                &mut self._100k_to_1m_sats,
                &mut self._1m_to_10m_sats,
                &mut self._10m_to_100m_sats,
                &mut self._1_to_10_btc,
                &mut self._10_to_100_btc,
                &mut self._100_to_1k_btc,
                &mut self._1k_to_10k_btc,
                &mut self._10k_to_100k_btc,
                &mut self.over_100k_btc,
            ],
        )
    }

    /// Builds the zero band with `zero` and the others with `band`.
    pub fn try_from_fns<E>(
        zero: impl FnOnce(AmountRangeId) -> Result<Z, E>,
        mut band: impl FnMut(AmountRangeId) -> Result<T, E>,
    ) -> Result<Self, E> {
        Ok(Self {
            _0_sats: zero(AmountRangeId::Zero)?,
            _1_to_10_sats: band(AmountRangeId::From1SatTo10Sats)?,
            _10_to_100_sats: band(AmountRangeId::From10SatsTo100Sats)?,
            _100_to_1k_sats: band(AmountRangeId::From100SatsTo1KSats)?,
            _1k_to_10k_sats: band(AmountRangeId::From1KSatsTo10KSats)?,
            _10k_to_100k_sats: band(AmountRangeId::From10KSatsTo100KSats)?,
            _100k_to_1m_sats: band(AmountRangeId::From100KSatsTo1MSats)?,
            _1m_to_10m_sats: band(AmountRangeId::From1MSatsTo10MSats)?,
            _10m_to_100m_sats: band(AmountRangeId::From10MTo100MSats)?,
            _1_to_10_btc: band(AmountRangeId::From1BtcTo10Btc)?,
            _10_to_100_btc: band(AmountRangeId::From10BtcTo100Btc)?,
            _100_to_1k_btc: band(AmountRangeId::From100BtcTo1KBtc)?,
            _1k_to_10k_btc: band(AmountRangeId::From1KBtcTo10KBtc)?,
            _10k_to_100k_btc: band(AmountRangeId::From10KBtcTo100KBtc)?,
            over_100k_btc: band(AmountRangeId::Over100kBtc)?,
        })
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
