use bitview_cohort::{AGE_RANGE_COUNT, Age, AgeRangeId, for_each_age_crossing};
use brk_error::Result;
use brk_types::Timestamp;
use statedb::Cursor;

/// Applies the cursor's next block and hands `change` its effect on the price/age distribution,
/// in the order every URPD applies it: the age crossings of the origins as they stood before
/// the block, then its new outputs (under an hour old), then its spends at their origins' ages.
/// Each change is (age, origin, signed sats), never zero; `crossings` is the crossing scan's
/// position, kept between blocks.
pub(crate) fn block_changes(
    cursor: &mut Cursor<'_>,
    timestamps: &[Timestamp],
    crossings: &mut [usize; AGE_RANGE_COUNT - 1],
    mut change: impl FnMut(AgeRangeId, usize, i64),
) -> Result<()> {
    let height = cursor.state().len();
    let current = timestamps[height];
    for_each_age_crossing(&timestamps[..height], current, crossings, |origin, younger, older, _| {
        let sats = cursor.state().amounts()[origin].sats as i64;
        if sats != 0 {
            change(younger, origin, -sats);
            change(older, origin, sats);
        }
    });
    let diff = cursor.advance()?.expect("history covers the replayed range");
    if diff.created.sats != 0 {
        change(AgeRangeId::Under1H, height, diff.created.sats as i64);
    }
    for (origin, amount) in diff.removed() {
        if amount.sats != 0 {
            let origin = origin as usize;
            let age = AgeRangeId::from(Age::new(current, timestamps[origin]));
            change(age, origin, -(amount.sats as i64));
        }
    }
    Ok(())
}
