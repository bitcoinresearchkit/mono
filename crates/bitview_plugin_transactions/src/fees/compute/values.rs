use brk_types::Sats;
use vecdb::{SaturatingAdd, VecIndex};

/// Visit a complete block's inputs or outputs to fill per-transaction sums.
/// Indexed Bitcoin transactions have at least one input and one output.
pub(super) fn sum<'a, I: VecIndex>(
    starts: &'a [I],
    end: usize,
    target: &'a mut Vec<Sats>,
) -> impl FnMut(Sats) + 'a {
    target.clear();
    let mut boundaries = starts
        .iter()
        .skip(1)
        .map(|index| index.to_usize())
        .chain([end]);
    let mut boundary = boundaries.next().unwrap();
    let mut pos = starts[0].to_usize();
    let mut sum = Sats::ZERO;
    move |value| {
        sum = sum.saturating_add(value);
        pos += 1;
        if pos == boundary {
            target.push(sum);
            boundary = boundaries.next().unwrap_or(end);
            sum = Sats::ZERO;
        }
    }
}
