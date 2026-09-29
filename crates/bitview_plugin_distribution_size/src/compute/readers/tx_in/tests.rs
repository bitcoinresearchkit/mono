use brk_types::{TxIndex, Vout};
use tempfile::tempdir;
use vecdb::{AnyStoredVec, Database, ImportableVec, Result, Version, WritableVec};

use super::*;

#[test]
fn block_inputs_match_across_pages_pending_values_and_backward_reads() -> Result<()> {
    let directory = tempdir()?;
    let db = Database::open(directory.path())?;
    let mut values = PcoVec::<TxInIndex, Sats>::import(&db, "values", Version::ONE)?;
    let mut outpoints = PcoVec::<TxInIndex, OutPoint>::import(&db, "outpoints", Version::ONE)?;
    let mut types = PcoVec::<TxInIndex, OutputType>::import(&db, "types", Version::ONE)?;
    let mut indexes = PcoVec::<TxInIndex, TypeIndex>::import(&db, "indexes", Version::ONE)?;
    for index in 0..32_000 {
        if index == 20_000 {
            values.write()?;
            outpoints.write()?;
            types.write()?;
            indexes.write()?;
        }
        values.push(Sats::new(index as u64 + 1));
        outpoints.push(if index % 101 == 0 {
            OutPoint::COINBASE
        } else {
            OutPoint::new(TxIndex::new(index % 900), Vout::ZERO)
        });
        types.push(OutputType::P2WPKH);
        indexes.push(TypeIndex::from(index));
    }
    let heights = TxHeightMap::from([10, 100, 500].map(TxIndex::new).to_vec());
    let mut reader = TxInReaders::new(&values, &outpoints, &types, &indexes, &heights);
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
            reader.collect_block_inputs(from, to - from, current);
        assert_eq!(actual_values, values.collect_range_at(from, to));
        assert_eq!(actual_types, types.collect_range_at(from, to));
        assert_eq!(actual_indexes, indexes.collect_range_at(from, to));
        let expected: Vec<_> = (from..to)
            .map(|index| {
                if index % 101 == 0 || index % 900 < 10 {
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
    Ok(())
}
