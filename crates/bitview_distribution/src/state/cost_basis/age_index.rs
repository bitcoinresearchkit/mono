use bitview_cohort::{
    Age, AgeAggregateId, AgeRangeId, HOURS_4M, HOURS_5M, HOURS_6M, for_each_age_cutoff,
};
use bitview_primitives::CentsCompact;
use brk_error::{Error, Result};
use brk_types::{Cents, Height, Timestamp, Version};
use statedb::{Cursor, Reader, State};
use vecdb::ReadableVec;

use super::{BucketedPrice, HeldExtremes, HeldOrigins, PriceIndex, PriceTotals};

/// Writer-owned age-filtered price index, retained only after a complete update
/// succeeds. Canonical origins remain owned by the history. `H`: what it keeps of the
/// origins holding coins beside it (their extremes, or nothing).
pub struct AgeIndexLive<V, H = ()> {
    pub origins: State,
    pub index: PriceIndex<4>,
    pub held: H,
    pub prices: Vec<BucketedPrice>,
    pub timestamps: Vec<Timestamp>,
    pub crossings: [usize; 3],
    version: V,
}

impl<V: PartialEq, H: HeldOrigins> AgeIndexLive<V, H> {
    /// Reuses `live` if it ends at `start` under `version` and the same history,
    /// else rebuilds it from the history state at `start`. Then extends the price
    /// and timestamp history through `end` and returns the spot prices of
    /// `start..end`.
    pub fn resume(
        live: Option<Self>,
        version: V,
        history: &Reader<'_>,
        start: usize,
        end: usize,
        prices: &impl ReadableVec<Height, Cents>,
        timestamps: &impl ReadableVec<Height, Timestamp>,
    ) -> Result<(Self, Vec<Cents>)> {
        let (mut live, reuse) = match live {
            Some(live) if live.version == version && history.resumes(&live.origins, start)? => {
                (live, true)
            }
            _ => (
                Self {
                    origins: history.state_at(start)?,
                    index: PriceIndex::default(),
                    held: H::default(),
                    prices: Vec::new(),
                    timestamps: Vec::new(),
                    crossings: [0; 3],
                    version,
                },
                false,
            ),
        };
        let price_start = live.prices.len();
        let mut spot = prices.collect_range_at(price_start, end);
        if spot.iter().any(|p| p.is_nan()) {
            return Err(Error::NotFound("invalid age-index price history".into()));
        }
        live.prices.extend(
            spot.iter()
                .map(|&price| BucketedPrice::from(CentsCompact::from(price))),
        );
        live.timestamps
            .extend(timestamps.collect_range_at(live.timestamps.len(), end));
        if live.prices.len() != end || live.timestamps.len() != end {
            return Err(Error::NotFound(
                "incomplete age-index price or timestamp history".into(),
            ));
        }
        live.held.reserve(end);
        if !reuse {
            restore(
                &mut live.index,
                &mut live.held,
                &live.origins,
                &live.prices,
                &live.timestamps,
            );
        }
        Ok((live, spot.split_off(start - price_start)))
    }
}

/// The history's spends and creations versions as one `Version`, folding in each u64's high half
/// instead of truncating it.
pub fn history_version(history: &Reader<'_>) -> Version {
    let (spends, creations) = history.versions();
    let fold = |version: u64| Version::new(version as u32) + Version::new((version >> 32) as u32);
    fold(spends) + fold(creations)
}

fn filters(current: Timestamp, origin: Timestamp) -> [bool; 4] {
    let age = AgeRangeId::from(Age::new(current, origin));
    [
        true,
        AgeAggregateId::Sth.contains(age),
        AgeAggregateId::Under4M.contains(age),
        AgeAggregateId::Under6M.contains(age),
    ]
}

fn restore(
    index: &mut PriceIndex<4>,
    held: &mut impl HeldOrigins,
    origins: &State,
    prices: &[BucketedPrice],
    timestamps: &[Timestamp],
) {
    index.reset();
    if let Some(last) = origins.len().checked_sub(1) {
        let current = timestamps[last];
        for (h, amount) in origins.amounts().iter().enumerate() {
            if amount.sats != 0 {
                index.add_raw(
                    prices[h],
                    amount.sats as i64,
                    filters(current, timestamps[h]),
                );
            }
        }
    }
    index.build();
    held.restore(origins.amounts(), prices);
}
pub fn advance(
    index: &mut PriceIndex<4>,
    held: &mut impl HeldOrigins,
    cursor: &mut Cursor<'_>,
    prices: &[BucketedPrice],
    timestamps: &[Timestamp],
    crossings: &mut [usize; 3],
) -> Result<()> {
    let h = cursor.state().len();
    let current = timestamps[h];
    for_each_age_cutoff(
        &timestamps[..h],
        current,
        &[HOURS_4M, HOURS_5M, HOURS_6M],
        crossings,
        |cutoff, origin, _| {
            let sats = cursor.state().amounts()[origin].sats;
            if sats != 0 {
                let mut mask = [false; 4];
                mask[CUTOFF_FILTERS[cutoff]] = true;
                index.add(prices[origin], -(sats as i64), mask);
            }
        },
    );
    let diff = cursor
        .advance()?
        .expect("validated age-filter history range");
    index.add(prices[h], diff.created.sats as i64, [true; 4]);
    for (origin, amount) in diff.removed() {
        held.spend_from(origin);
        let origin = origin as usize;
        index.add(
            prices[origin],
            -(amount.sats as i64),
            filters(current, timestamps[origin]),
        );
    }
    held.settle(h, cursor.state().amounts(), prices);
    Ok(())
}

/// The index's filter of each age cutoff, in `for_each_age_cutoff`'s order (4, 5 and 6 months).
const CUTOFF_FILTERS: [usize; 3] = [2, 1, 3];

/// A cohort as an age cut: its cutoff and whether it's the young side of it; all of them, none.
fn cut_of(id: AgeAggregateId) -> Option<(usize, bool)> {
    match id {
        AgeAggregateId::All => None,
        AgeAggregateId::Under4M => Some((0, true)),
        AgeAggregateId::Over4M => Some((0, false)),
        AgeAggregateId::Sth => Some((1, true)),
        AgeAggregateId::Lth => Some((1, false)),
        AgeAggregateId::Under6M => Some((2, true)),
        AgeAggregateId::Over6M => Some((2, false)),
    }
}

/// A cohort's lowest and highest cost basis, exactly: of its origins still holding coins, the
/// lowest and highest price, at the state's last height (`len` origins). An age cut's young side
/// is its origins from the first its filter holds (the filter turns once: origins come in age
/// order), its old side those before.
pub fn extremes(
    extremes: &HeldExtremes,
    id: AgeAggregateId,
    timestamps: &[Timestamp],
    len: usize,
) -> (Cents, Cents) {
    let Some(last) = len.checked_sub(1) else {
        return (Cents::NAN, Cents::NAN);
    };
    let current = timestamps[last];
    let cut = |cutoff: usize| {
        timestamps[..len].partition_point(|&t| !filters(current, t)[CUTOFF_FILTERS[cutoff]])
    };
    let (from, to) = match cut_of(id) {
        None => (0, len),
        Some((cutoff, true)) => (cut(cutoff), len),
        Some((cutoff, false)) => (0, cut(cutoff)),
    };
    extremes.range(from, to)
}

/// The complementary filters share the same four independent index totals.
pub fn selected(id: AgeAggregateId, totals: &PriceTotals<4>) -> (i64, i128) {
    let (positive, negative) = match cut_of(id) {
        None => (0, None),
        Some((cutoff, true)) => (CUTOFF_FILTERS[cutoff], None),
        Some((cutoff, false)) => (0, Some(CUTOFF_FILTERS[cutoff])),
    };
    (
        totals.sats[positive] - negative.map_or(0, |i| totals.sats[i]),
        totals.cap[positive] - negative.map_or(0, |i| totals.cap[i]),
    )
}
