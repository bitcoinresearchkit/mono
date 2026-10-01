use super::{PriceIndex, PriceTotals};
use bitview_cohort::{
    AgeAggregateId, AgeRangeId, HOURS_4M, HOURS_5M, HOURS_6M, for_each_age_cutoff,
};
use brk_error::Result;
use brk_types::{Age, CentsCompact, Timestamp};
use statedb::{Cursor, State};

fn filters(current: Timestamp, origin: Timestamp) -> [bool; 4] {
    let age = AgeRangeId::from(Age::new(current, origin));
    [
        true,
        AgeAggregateId::Sth.contains(age),
        AgeAggregateId::Under4M.contains(age),
        AgeAggregateId::Under6M.contains(age),
    ]
}

pub fn restore(
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
