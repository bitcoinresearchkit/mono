use crate::{COST_BASIS_PRICE_DIGITS, ProjectedBucket};
use bitview_cohort::{AGE_RANGE_COUNT, AGE_RANGE_IDS, AgeRange, AgeRangeId, for_each_age_crossing};
use brk_error::{Error, Result};
use brk_types::{Age, Cents, CentsCompact, Sats, Timestamp};
use statedb::{Cursor, State};

/// One in-memory price histogram for a sequential history pass. Origin amounts
/// remain owned by the history cursor; this stores only their price/age grouping.
pub struct OriginUrpd {
    prices: Vec<CentsCompact>,
    amounts: Vec<[u64; AGE_RANGE_COUNT]>,
    order: Vec<u32>,
    origin_buckets: Vec<u32>,
    crossings: [usize; AGE_RANGE_COUNT - 1],
}
impl OriginUrpd {
    pub fn new(state: &State, prices: &[Cents], timestamps: &[Timestamp]) -> Result<Self> {
        if state.len() > prices.len()
            || prices.len() != timestamps.len()
            || prices.iter().any(|p| p.is_nan())
        {
            return Err(Error::Internal("invalid origin price/timestamp coverage"));
        }
        let rounded = |p: Cents| CentsCompact::from(p).round_to_dollar(COST_BASIS_PRICE_DIGITS);
        let mut buckets: Vec<_> = prices.iter().copied().map(rounded).collect();
        buckets.sort_unstable();
        buckets.dedup();
        buckets.shrink_to_fit();
        let origin_buckets: Vec<_> = prices
            .iter()
            .map(|&p| buckets.binary_search(&rounded(p)).unwrap() as u32)
            .collect();
        let mut amounts = vec![[0; AGE_RANGE_COUNT]; buckets.len()];
        if let Some(&current) = timestamps.get(state.len().wrapping_sub(1)) {
            for (h, amount) in state.amounts().iter().enumerate() {
                let age = AgeRangeId::from(Age::new(current, timestamps[h]));
                amounts[origin_buckets[h] as usize][age.index()] += amount.sats;
            }
        }
        Ok(Self {
            order: (0..buckets.len() as u32).collect(),
            prices: buckets,
            amounts,
            origin_buckets,
            crossings: [0; AGE_RANGE_COUNT - 1],
        })
    }
    /// Append newly available creation prices without rebuilding the occupied histogram.
    pub fn extend_prices(&mut self, prices: &[Cents]) -> Result<()> {
        if prices.iter().any(|p| p.is_nan()) {
            return Err(Error::Internal("invalid origin price"));
        }
        for &price in prices {
            let price = CentsCompact::from(price).round_to_dollar(COST_BASIS_PRICE_DIGITS);
            let slot = match self.prices.binary_search(&price) {
                Ok(position) => self.order[position],
                Err(position) => {
                    let slot = self.amounts.len() as u32;
                    self.prices.insert(position, price);
                    self.order.insert(position, slot);
                    self.amounts.push([0; AGE_RANGE_COUNT]);
                    slot
                }
            };
            self.origin_buckets.push(slot);
        }
        Ok(())
    }
    /// Apply exactly one block, including all age crossings before its spends.
    pub fn advance(&mut self, cursor: &mut Cursor<'_>, timestamps: &[Timestamp]) -> Result<()> {
        let h = cursor.state().len();
        let current = *timestamps
            .get(h)
            .ok_or(Error::Internal("missing origin timestamp"))?;
        if h >= self.origin_buckets.len() {
            return Err(Error::Internal("missing origin price"));
        }
        for_each_age_crossing(
            &timestamps[..h],
            current,
            &mut self.crossings,
            |origin, younger, older, _| {
                let sats = cursor.state().amounts()[origin].sats;
                let row = &mut self.amounts[self.origin_buckets[origin] as usize];
                row[younger.index()] -= sats;
                row[older.index()] += sats;
            },
        );
        let diff = cursor
            .advance()?
            .ok_or(Error::Internal("incomplete origin history"))?;
        self.amounts[self.origin_buckets[h] as usize][AgeRangeId::Under1H.index()] +=
            diff.created.sats;
        for &(origin, amount) in diff.removed() {
            let origin = origin as usize;
            let age = AgeRangeId::from(Age::new(current, timestamps[origin]));
            self.amounts[self.origin_buckets[origin] as usize][age.index()] -= amount.sats;
        }
        Ok(())
    }
    pub fn buckets(&self) -> impl Iterator<Item = (CentsCompact, &[u64; AGE_RANGE_COUNT])> + Clone {
        self.prices
            .iter()
            .copied()
            .zip(&self.order)
            .map(|(price, &slot)| (price, &self.amounts[slot as usize]))
    }
    pub fn iter(&self) -> impl Iterator<Item = (AgeRangeId, CentsCompact, Sats)> + '_ {
        self.buckets().flat_map(|(price, values)| {
            AGE_RANGE_IDS
                .into_iter()
                .zip(values)
                .filter_map(move |(age, &sats)| {
                    (sats != 0).then_some((age, price, Sats::new(sats)))
                })
        })
    }

    /// Project all requested weights together, visiting each bucket's ages once.
    /// The caller can consume this view directly or retain its rows for repeated reads.
    /// Weighted entries retain input order; missing weights produce zero. Rounding
    /// happens after combining all ages in each price bucket.
    pub fn project<'a, const N: usize>(
        &'a self,
        weights: &'a [Option<&AgeRange<f64>>; N],
    ) -> impl Iterator<Item = ProjectedBucket<N>> + Clone + 'a {
        self.buckets().filter_map(move |(price, supplies)| {
            let mut raw = 0;
            let mut masses = [0.0_f64; N];
            for (&age, &sats) in AgeRangeId::ALL.iter().zip(supplies) {
                raw += sats;
                if sats != 0 {
                    for (mass, weights) in masses.iter_mut().zip(weights) {
                        if let Some(weights) = weights {
                            *mass += sats as f64 * *age.select(weights);
                        }
                    }
                }
            }
            (raw != 0).then(|| ProjectedBucket {
                price,
                raw: Sats::new(raw),
                weighted: masses.map(|mass| Sats::new(mass.floor() as u64)),
            })
        })
    }
}
