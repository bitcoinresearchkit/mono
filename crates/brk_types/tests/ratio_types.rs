use brk_types::BoundedRatio;

#[test]
fn bounded_ratios_floor_and_complement_exactly() {
    assert_eq!(BoundedRatio::from(0.0), BoundedRatio::ZERO);
    assert_eq!(BoundedRatio::from(1.0), BoundedRatio::ONE);
    assert_eq!(BoundedRatio::ZERO.complement(), BoundedRatio::ONE);
    assert_eq!(BoundedRatio::ONE.complement(), BoundedRatio::ZERO);
    assert!(BoundedRatio::NAN.complement().is_nan());
    assert_eq!(BoundedRatio::from(0.1).inner(), 429_496_729);
    for i in 0..=100_000 {
        let original = i as f64 / 100_000.0;
        let encoded = BoundedRatio::from(original);
        let decoded = f64::from(encoded);
        assert!(decoded <= original + f64::EPSILON);
        assert!(original - decoded < 1.0 / BoundedRatio::SCALE as f64 + f64::EPSILON);
        assert_eq!(
            encoded.inner() + encoded.complement().inner(),
            BoundedRatio::SCALE
        );
        assert_eq!(encoded.complement().complement(), encoded);
    }
}
