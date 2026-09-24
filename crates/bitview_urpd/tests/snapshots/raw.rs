use std::{collections::BTreeMap, env, fs, iter, process};

use bitcoin::Amount;
use brk_types::Date;

use super::*;

#[test]
fn malformed_snapshot_returns_errors() {
    let valid =
        UrpdRaw::serialize_iter([(CentsCompact::new(100), Sats::from(1_u64))].into_iter()).unwrap();
    for (offset, value) in [
        (0, 2_usize),
        (0, UrpdRaw::MAX_ENTRIES + 1),
        (0, usize::MAX),
        (8, usize::MAX),
        (16, usize::MAX),
    ] {
        let mut bytes = valid.clone();
        bytes[offset..offset + 8].copy_from_slice(value.to_le_bytes().as_ref());
        assert!(UrpdRaw::deserialize_entries(&bytes).is_err());
    }
    for prices in [[100, 100], [200, 100]] {
        let bytes = UrpdRaw::serialize_iter(
            prices
                .into_iter()
                .map(|price| (CentsCompact::new(price), Sats::from(1_u64))),
        )
        .unwrap();
        assert!(UrpdRaw::deserialize_entries(&bytes).is_err());
    }
    for end in [0, 23, valid.len() - 1] {
        assert!(UrpdRaw::deserialize_entries(&valid[..end]).is_err());
    }
}

#[test]
fn reserved_price_is_rejected_without_constructing_a_nan_price() {
    for (price, supply) in [(u32::MAX, 1_u64), (100, u64::MAX)] {
        let keys = simple_compress(&[price], &ChunkConfig::default()).unwrap();
        let values = simple_compress(&[supply], &ChunkConfig::default()).unwrap();
        let mut bytes = Vec::new();
        for count in [1_usize, keys.len(), values.len()] {
            bytes.extend(count.to_le_bytes());
        }
        bytes.extend(keys);
        bytes.extend(values);
        assert!(UrpdRaw::deserialize_entries(&bytes).is_err());
    }
}

#[test]
fn file_roundtrip() {
    let root = env::temp_dir().join(format!("brk-urpd-file-{}", process::id()));
    let date = Date::new(2026, 8, 4);
    let expected = BTreeMap::from([
        (CentsCompact::new(100), Sats::from(21_u64)),
        (CentsCompact::new(200), Sats::from(34_u64)),
    ]);

    UrpdRaw::write(
        &root,
        "test",
        date,
        expected.iter().map(|(&price, &sats)| (price, sats)),
    )
    .unwrap();
    let encoded = UrpdRaw::read_bytes(&root, "test", date).unwrap();
    let expected_entries = expected
        .iter()
        .map(|(&price, &sats)| (price, sats))
        .collect::<Vec<_>>();
    assert_eq!(
        UrpdRaw::deserialize_entries(&encoded).unwrap(),
        expected_entries
    );

    UrpdRaw::write(
        &root,
        "test",
        date,
        expected_entries
            .iter()
            .map(|&(price, _)| (price, Sats::ZERO)),
    )
    .unwrap();
    assert_eq!(
        UrpdRaw::deserialize_entries(&encoded).unwrap(),
        expected_entries
    );
    let rewritten = UrpdRaw::read_bytes(&root, "test", date).unwrap();
    assert_ne!(
        UrpdRaw::deserialize_entries(&rewritten).unwrap(),
        expected_entries
    );
    let mut trailing = encoded;
    trailing.push(0);
    assert!(UrpdRaw::deserialize_entries(&trailing).is_err());

    UrpdRaw::write(&root, "empty", date, iter::empty()).unwrap();
    let empty = UrpdRaw::read_bytes(&root, "empty", date).unwrap();
    assert!(UrpdRaw::deserialize_entries(&empty).unwrap().is_empty());

    let oversized = UrpdRaw::path(&root, "empty", date);
    fs::OpenOptions::new()
        .write(true)
        .open(&oversized)
        .unwrap()
        .set_len(UrpdRaw::MAX_ENCODED_BYTES as u64 + 1)
        .unwrap();
    assert!(UrpdRaw::read_bytes(&root, "empty", date).is_err());

    fs::remove_dir_all(root).unwrap();
}

#[test]
fn invalid_supplies_and_non_finite_writer_prices_are_errors() {
    let raw = UrpdRaw {
        map: BTreeMap::from([
            (
                CentsCompact::new(100),
                Sats::from(Amount::MAX_MONEY.to_sat()),
            ),
            (CentsCompact::new(200), Sats::from(1_u64)),
        ]),
    };
    assert!(raw.serialize().is_err());
    assert!(UrpdRaw::serialize_iter([(CentsCompact::NAN, Sats::ZERO)].into_iter()).is_err());
}
