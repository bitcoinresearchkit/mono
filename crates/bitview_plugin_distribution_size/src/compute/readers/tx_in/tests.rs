use tempfile::tempdir;
use vecdb::{AnyStoredVec, Database, ImportableVec, Version, WritableVec};

use super::*;

#[test]
fn block_inputs_match_across_pages_pending_values_and_backward_reads() -> Result<()> {
    let directory = tempdir()?;
    let db = Database::open(directory.path())?;
    let mut values = PcoVec::<TxInIndex, Sats>::import(&db, "values", Version::ONE)?;
    let mut references = PcoVec::<TxInIndex, TxOutIndex>::import(&db, "references", Version::ONE)?;
    let mut types = PcoVec::<TxInIndex, OutputType>::import(&db, "types", Version::ONE)?;
    let mut indexes = PcoVec::<TxInIndex, TypeIndex>::import(&db, "indexes", Version::ONE)?;
    for index in 0usize..32_000 {
        if index == 20_000 {
            values.write()?;
            references.write()?;
            types.write()?;
            indexes.write()?;
        }
        values.push(Sats::new(index as u64 + 1));
        references.push(if index % 101 == 0 {
            TxOutIndex::COINBASE
        } else {
            TxOutIndex::from(index % 900)
        });
        types.push(OutputType::P2WPKH);
        indexes.push(TypeIndex::from(index));
    }
    let heights = HeightMap::from([0usize, 100, 500].map(TxOutIndex::from).to_vec());
    let mut reader = TxInReaders::new(&values, &references, &types, &indexes, &heights);
    let current = Height::new(900);
    for (from, to) in [
        (0, 1025),
        (1025, 17_000),
        (19_995, 20_010),
        (31_000, 32_000),
        (20, 31),
        (5, 5),
    ] {
        let (actual_values, actual_heights, actual_types, actual_indexes) =
            reader.collect_inputs(from, to - from, current)?;
        assert_eq!(actual_values, values.collect_range_at(from, to));
        assert_eq!(actual_types, types.collect_range_at(from, to));
        assert_eq!(actual_indexes, indexes.collect_range_at(from, to));
        let expected: Vec<_> = (from..to)
            .map(|index| {
                if index % 101 == 0 {
                    current
                } else if index % 900 < 100 {
                    Height::ZERO
                } else if index % 900 < 500 {
                    Height::new(1)
                } else {
                    Height::new(2)
                }
            })
            .collect();
        assert_eq!(actual_heights, expected);
    }
    let missing = HeightMap::from([100usize].map(TxOutIndex::from).to_vec());
    let mut reader = TxInReaders::new(&values, &references, &types, &indexes, &missing);
    assert!(reader.collect_inputs(1, 1, current).is_err());
    // A coinbase has no creation height to resolve and retains its explicit rule.
    assert_eq!(reader.collect_inputs(0, 1, current)?.1, &[current]);
    Ok(())
}
