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
