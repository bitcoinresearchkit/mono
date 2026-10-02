use super::*;

#[test]
fn matches_direct_utxo_formula_and_term_merge() {
    // (creation price in cents, sats, encoded cohort weight, is STH).
    let outputs = [
        (10_001_u128, 137_u128, 321_u32, true),
        (80_009, 911, 321, true),
        (30_007, 271, 9_876, false),
        (90_011, 31, 9_876, false),
    ];
    let mut sth = WeightedCapitalizedPrice::default();
    let mut lth = WeightedCapitalizedPrice::default();
    let mut direct = WeightedCapitalizedPrice::default();
    let mut numerator = 0;
    let mut denominator = 0;
    for (price, sats, weight, short) in outputs {
        let cap = CentsSats::new(price * sats);
        let second = CentsSquaredSats::new(price * price * sats);
        let encoded = BoundedRatio::from_raw(weight);
        direct.add(cap, second, encoded);
        let term = if short { &mut sth } else { &mut lth };
        term.add(cap, second, encoded);
        numerator += price * price * sats * u128::from(weight);
        denominator += price * sats * u128::from(weight);
    }
    let expected = Cents::from(numerator / denominator);
    assert_eq!(direct.value(), expected);
    sth.merge(lth);
    assert_eq!(sth.value(), expected);

    // Aggregating UTXOs into age-band moments before weighting is equivalent.
    let mut grouped = WeightedCapitalizedPrice::default();
    for short in [true, false] {
        let mut cap = 0;
        let mut second = 0;
        let mut weight = 0;
        for (price, sats, w, is_short) in outputs {
            if short == is_short {
                cap += price * sats;
                second += price * price * sats;
                weight = w;
            }
        }
        grouped.add(
            CentsSats::new(cap),
            CentsSquaredSats::new(second),
            BoundedRatio::from_raw(weight),
        );
    }
    assert_eq!(grouped.value(), expected);
}

#[test]
fn preserves_sub_raw_unit_weights_and_integer_price_boundaries() {
    for raw_weight in [1, 3, 321, BoundedRatio::SCALE / 2, BoundedRatio::SCALE] {
        for price in [1_u128, 101, 7_718_423] {
            let mut state = WeightedCapitalizedPrice::default();
            state.add(
                CentsSats::new(price),
                CentsSquaredSats::new(price * price),
                BoundedRatio::from_raw(raw_weight),
            );
            assert_eq!(state.value(), Cents::from(price));
        }
    }
}

#[test]
fn large_moments_do_not_overflow_when_weighted() {
    let price = 20_000_000_u128;
    let cap = price * 2_100_000_000_000_000;
    let second = cap * price;
    assert!(
        second
            .checked_mul(u128::from(BoundedRatio::SCALE))
            .is_none()
    );
    for weight in [
        BoundedRatio::ONE,
        BoundedRatio::from_raw(1),
        BoundedRatio::from(0.321),
    ] {
        let mut state = WeightedCapitalizedPrice::default();
        state.add(CentsSats::new(cap), CentsSquaredSats::new(second), weight);
        assert_eq!(state.value(), Cents::from(price));
    }
}

#[test]
fn zero_and_undefined_weights_have_explicit_behavior() {
    let mut state = WeightedCapitalizedPrice::default();
    assert_eq!(state.value(), Cents::ZERO);
    state.add(
        CentsSats::new(1),
        CentsSquaredSats::new(123),
        BoundedRatio::ZERO,
    );
    state.add(CentsSats::ZERO, CentsSquaredSats::ZERO, BoundedRatio::NAN);
    assert_eq!(state.value(), Cents::ZERO);
    state.add(
        CentsSats::new(1),
        CentsSquaredSats::new(123),
        BoundedRatio::NAN,
    );
    assert!(state.value().is_nan());
    let mut merged = WeightedCapitalizedPrice::default();
    merged.merge(state);
    assert!(merged.value().is_nan());
}

#[test]
fn mixed_moments_match_integer_reference_across_weight_range() {
    let mut seed = 12345_u64;
    for _ in 0..1_000 {
        let mut state = WeightedCapitalizedPrice::default();
        let mut numerator = 0_u128;
        let mut denominator = 0_u128;
        for _ in 0..8 {
            seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
            let weight = (seed % u64::from(BoundedRatio::SCALE)) as u32;
            let price = u128::from(seed % 20_000_000 + 1);
            let sats = u128::from(seed % 1_000_000 + 1);
            state.add(
                CentsSats::new(price * sats),
                CentsSquaredSats::new(price * price * sats),
                BoundedRatio::from_raw(weight),
            );
            numerator += price * price * sats * u128::from(weight);
            denominator += price * sats * u128::from(weight);
        }
        assert_eq!(state.value(), Cents::from(numerator / denominator));
    }
}
