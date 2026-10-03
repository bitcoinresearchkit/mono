use bitview_cohort::{
    AgeAggregateId, AgeRangeId, HOURS_4M, HOURS_5M, HOURS_6M, for_each_age_cutoff,
};
use bitview_primitives::CentsCompact;
use brk_error::{Error, Result};
use brk_types::{Age, Cents, Height, Timestamp};
use statedb::{Cursor, Reader, State};
use vecdb::ReadableVec;

use super::{PriceIndex, PriceTotals};

/// Writer-owned age-filtered price index, retained only after a complete update
/// succeeds. Canonical origins remain owned by the history.
pub struct AgeIndexLive<V> {
    pub origins: State,
    pub index: PriceIndex<4>,
    pub prices: Vec<CentsCompact>,
    pub timestamps: Vec<Timestamp>,
    pub crossings: [usize; 3],
    version: V,
}

impl<V: PartialEq> AgeIndexLive<V> {
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
            Some(live)
                if live.origins.len() == start
                    && live.version == version
                    && history.matches(&live.origins)? =>
            {
                (live, true)
            }
            _ => (
                Self {
                    origins: history.state_at(start)?,
                    index: PriceIndex::default(),
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
        live.prices
            .extend(spot.iter().copied().map(CentsCompact::from));
        live.timestamps
            .extend(timestamps.collect_range_at(live.timestamps.len(), end));
        if live.prices.len() != end || live.timestamps.len() != end {
            return Err(Error::NotFound(
                "incomplete age-index price or timestamp history".into(),
            ));
        }
        if !reuse {
            restore(
                &mut live.index,
                &live.origins,
                &live.prices,
                &live.timestamps,
            );
        }
        Ok((live, spot.split_off(start - price_start)))
    }
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
    origins: &State,
    prices: &[CentsCompact],
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
}
pub fn advance(
    index: &mut PriceIndex<4>,
    cursor: &mut Cursor<'_>,
    prices: &[CentsCompact],
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
                mask[[2, 1, 3][cutoff]] = true;
                index.add(prices[origin], -(sats as i64), mask);
            }
        },
    );
    let diff = cursor
        .advance()?
        .expect("validated age-filter history range");
    index.add(prices[h], diff.created.sats as i64, [true; 4]);
    for (origin, amount) in diff.removed() {
        let origin = origin as usize;
        index.add(
            prices[origin],
            -(amount.sats as i64),
            filters(current, timestamps[origin]),
        );
    }
    Ok(())
}

/// The complementary filters share the same four independent index totals.
pub fn selected(id: AgeAggregateId, totals: &PriceTotals<4>) -> (i64, i128) {
    let (positive, negative) = match id {
        AgeAggregateId::All => (0, None),
        AgeAggregateId::Sth => (1, None),
        AgeAggregateId::Lth => (0, Some(1)),
        AgeAggregateId::Under4M => (2, None),
        AgeAggregateId::Under6M => (3, None),
        AgeAggregateId::Over4M => (0, Some(2)),
        AgeAggregateId::Over6M => (0, Some(3)),
    };
    (
        totals.sats[positive] - negative.map_or(0, |i| totals.sats[i]),
        totals.cap[positive] - negative.map_or(0, |i| totals.cap[i]),
    )
}
