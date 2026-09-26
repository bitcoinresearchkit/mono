use brk_types::{OutputType, Sats, TypeIndex};

use super::Candidate;

type Entry = (Sats, OutputType, TypeIndex);

fn entry(value: u64, kind: OutputType, index: usize) -> Entry {
    (Sats::new(value), kind, TypeIndex::from(index))
}

fn classify(candidate: &mut Candidate, entries: &[Entry]) -> bool {
    candidate.clear(entries.len() / 2);
    entries
        .iter()
        .all(|&(value, kind, index)| candidate.add(value, kind, index))
}

#[test]
fn repeated_values_match_but_any_script_address_reuse_rejects() {
    let mut candidate = Candidate::default();
    let original: Vec<_> = (0..10)
        .map(|i| entry(if i < 5 { 10_000 } else { 9_000 }, OutputType::P2WPKH, i))
        .collect();
    assert!(classify(&mut candidate, &original));
    // Reuse within inputs, within outputs, and across the input/output boundary.
    for (from, to) in [(0, 1), (5, 6), (0, 5)] {
        let mut reused = original.clone();
        reused[to].2 = reused[from].2;
        assert!(!classify(&mut candidate, &reused));
        assert!(classify(&mut candidate, &original));
    }
}

#[test]
fn zeros_count_separately_and_distinct_value_limit_is_inclusive() {
    let mut candidate = Candidate::default();
    let mut entries: Vec<_> = (0..10)
        .map(|i| entry((i % 5 + 1) as u64, OutputType::P2TR, i))
        .collect();
    assert!(classify(&mut candidate, &entries));
    entries[9].0 = Sats::new(6);
    assert!(!classify(&mut candidate, &entries));

    for zeros in 0..=10 {
        for (i, item) in entries.iter_mut().enumerate() {
            item.0 = Sats::new(if i < zeros { 0 } else { 1 });
        }
        assert_eq!(classify(&mut candidate, &entries), zeros <= 4);
    }
}

#[test]
fn address_identity_includes_type_and_excludes_non_address_scripts() {
    let mut candidate = Candidate::default();
    assert!(classify(
        &mut candidate,
        &[
            entry(7, OutputType::P2WPKH, 0),
            entry(7, OutputType::P2TR, 0),
            entry(7, OutputType::OpReturn, 0),
            entry(7, OutputType::OpReturn, 0),
        ],
    ));
}

#[test]
fn early_rejection_matches_an_independent_complete_scan() {
    let mut candidate = Candidate::default();
    let mut seed = 7u64;
    let mut accepted = 0;
    for case in 0..2_000 {
        let len = 10 + case % 51;
        let entries: Vec<_> = (0..len)
            .map(|i| {
                seed = seed.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1);
                let kind = [OutputType::P2WPKH, OutputType::P2TR, OutputType::OpReturn]
                    [(seed >> 32) as usize % 3];
                let index = if case % 2 == 0 { i } else { i / 2 };
                entry(seed % (1 + case as u64 % 32), kind, index)
            })
            .collect();
        let zeros = entries.iter().filter(|item| item.0.is_zero()).count();
        let mut values: Vec<_> = entries
            .iter()
            .filter_map(|item| (!item.0.is_zero()).then_some(item.0))
            .collect();
        values.sort_unstable();
        values.dedup();
        let mut addresses: Vec<_> = entries
            .iter()
            .filter(|item| item.1 != OutputType::OpReturn)
            .map(|item| (item.1 as u8, item.2))
            .collect();
        let address_count = addresses.len();
        addresses.sort_unstable();
        addresses.dedup();
        let expected = addresses.len() == address_count && values.len() + zeros <= len / 2;
        assert_eq!(classify(&mut candidate, &entries), expected, "case {case}");
        accepted += usize::from(expected);
    }
    assert!(accepted > 0 && accepted < 2_000);
}
