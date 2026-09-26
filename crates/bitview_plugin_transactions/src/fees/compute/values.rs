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

#[cfg(test)]
mod tests {
    use brk_types::{Sats, TxInIndex, Version};
    use tempfile::tempdir;
    use vecdb::{AnyStoredVec, Database, ImportableVec, PcoVec, ReadableVec, WritableVec};

    use super::sum;

    #[test]
    fn sums_complete_transactions_with_coinbase_and_reused_scratch() {
        let directory = tempdir().unwrap();
        let db = Database::open(directory.path()).unwrap();
        let mut source =
            PcoVec::<TxInIndex, Sats>::forced_import(&db, "values", Version::ONE).unwrap();
        for value in [7, u64::MAX, 2, 3, 0, 9] {
            source.push(Sats::from(value));
        }
        source.write().unwrap();
        let mut sums = vec![Sats::MAX; 10];
        source.for_each_range_at(
            1,
            6,
            sum(&[1usize, 2, 4].map(TxInIndex::from), 6, &mut sums),
        );
        assert_eq!(sums, [Sats::MAX, Sats::new(5), Sats::new(9)]);
        source.for_each_range_at(4, 6, sum(&[TxInIndex::from(4usize)], 6, &mut sums));
        assert_eq!(sums, [Sats::new(9)]);
    }

    #[test]
    fn retained_decoder_sums_adjacent_blocks_across_compressed_pages() {
        let directory = tempdir().unwrap();
        let db = Database::open(directory.path()).unwrap();
        let mut source =
            PcoVec::<TxInIndex, Sats>::forced_import(&db, "values", Version::ONE).unwrap();
        let values: Vec<_> = (0..3_001)
            .map(|index| match index {
                123 | 1_025 | 2_010 => Sats::MAX,
                _ => Sats::new(index % 17),
            })
            .collect();
        for &value in &values {
            source.push(value);
        }
        source.write().unwrap();

        let mut cursor = source.range_cursor_at(123, values.len());
        let mut actual = Vec::new();
        for bounds in [
            &[123, 124, 1_021, 1_025][..],
            &[1_025, 1_040, 1_999, 2_010],
            &[2_010, 2_500, 3_001],
        ] {
            let starts: Vec<_> = bounds[..bounds.len() - 1]
                .iter()
                .copied()
                .map(TxInIndex::from)
                .collect();
            let end = *bounds.last().unwrap();
            cursor.for_each(end - bounds[0], sum(&starts, end, &mut actual));
            let expected: Vec<_> = bounds
                .windows(2)
                .map(|bounds| {
                    Sats::new(
                        values[bounds[0]..bounds[1]]
                            .iter()
                            .fold(0_u64, |sum, value| sum.saturating_add(u64::from(*value))),
                    )
                })
                .collect();
            assert_eq!(actual, expected);
            assert_eq!(cursor.position(), end);
        }
        assert_eq!(cursor.remaining(), 0);
    }
}
