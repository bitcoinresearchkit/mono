use std::fs::OpenOptions;

use bitview_cohort::{AgeRange, AgeRangeId, UTXOAggregateId};
use brk_types::{CentsCompact, Date, Sats};
use tempfile::tempdir;

use super::AgeRangeUrpds;
use crate::UrpdRaw;

#[test]
fn packed_file_reads_all_or_one_age_range() {
    let root = tempdir().unwrap();
    let date = Date::new(2026, 8, 23);
    let expected = AgeRangeUrpds::from_sorted_entries(|id| {
        let price = CentsCompact::new((id.index() as u32 + 1) * 100);
        [
            (price, Sats::from(id.index() as u64 + 1)),
            (price, Sats::from(1_u64)),
        ]
    });
    for id in AgeRangeId::ALL.iter().copied() {
        assert_eq!(
            expected.get(id),
            &[(
                CentsCompact::new((id.index() as u32 + 1) * 100),
                Sats::from(id.index() as u64 + 2),
            )]
        );
    }
    expected.write(root.path(), date).unwrap();

    let actual = AgeRangeUrpds::read(root.path(), date).unwrap();
    for id in AgeRangeId::ALL.iter().copied() {
        assert_eq!(actual.get(id), expected.get(id));
    }

    let id = AgeRangeId::From2YTo3Y;
    let one = AgeRangeUrpds::read_one_bytes(root.path(), id, date).unwrap();
    assert_eq!(
        UrpdRaw::deserialize_entries(&one).unwrap(),
        expected.get(id)
    );

    let mut captured = Vec::new();
    for id in UTXOAggregateId::ALL.iter().copied() {
        let encoded = AgeRangeUrpds::read_aggregate_encoded(root.path(), id, date).unwrap();
        assert_eq!(
            encoded.decode_entries().unwrap(),
            id.age_range_ids()
                .iter()
                .flat_map(|&age| expected.get(age).iter().copied())
                .collect::<Vec<_>>()
        );
        assert_eq!(encoded.sections().count(), id.age_range_ids().len());
        for (section, age) in encoded.sections().zip(id.age_range_ids()) {
            assert_eq!(
                UrpdRaw::deserialize_entries(section).unwrap(),
                expected.get(*age)
            );
        }
        captured.push((id, encoded));
    }

    let bytes = AgeRangeUrpds::read_one_bytes(root.path(), id, date).unwrap();
    assert_eq!(
        UrpdRaw::deserialize_entries(&bytes).unwrap(),
        expected.get(id)
    );
    let replacement = AgeRangeUrpds {
        entries: AgeRange::from_fn(|_| Vec::new()),
    };
    replacement.write(root.path(), date).unwrap();
    for (id, encoded) in captured {
        assert_eq!(
            encoded.decode_entries().unwrap(),
            id.age_range_ids()
                .iter()
                .flat_map(|&age| expected.get(age).iter().copied())
                .collect::<Vec<_>>()
        );
    }
    assert!(
        UrpdRaw::deserialize_entries(
            &AgeRangeUrpds::read_one_bytes(root.path(), id, date).unwrap()
        )
        .unwrap()
        .is_empty()
    );
    assert_eq!(
        UrpdRaw::deserialize_entries(&bytes).unwrap(),
        expected.get(id)
    );
    OpenOptions::new()
        .write(true)
        .open(AgeRangeUrpds::path(root.path(), date))
        .unwrap()
        .set_len(UrpdRaw::MAX_ENCODED_BYTES as u64 + 1)
        .unwrap();
    assert!(AgeRangeUrpds::read(root.path(), date).is_err());
    assert!(AgeRangeUrpds::read_one_bytes(root.path(), id, date).is_err());
    assert!(
        AgeRangeUrpds::read_aggregate_encoded(root.path(), UTXOAggregateId::All, date).is_err()
    );
    assert!(
        AgeRangeUrpds::read_aggregate_encoded(root.path(), UTXOAggregateId::Sth, date).is_err()
    );
}
