use brk_types::{
    BasisPoints32, BoundedRatio, CheckedSub, PartsPerMillion64, PartsPerMillionSigned32, PriceRatio,
};

#[cfg(feature = "storage")]
use vecdb::{Formattable, Pco};

#[test]
fn price_ratio_numeric_traits_preserve_saturation_and_undefined() {
    assert_eq!(PriceRatio::ONE + PriceRatio::ONE, PriceRatio::from(2.0));
    assert_eq!(PriceRatio::MAX + PriceRatio::ONE, PriceRatio::MAX);
    assert_eq!(PriceRatio::MAX + PriceRatio::MAX, PriceRatio::MAX);
    assert_eq!(PriceRatio::from(2.0) / 2, PriceRatio::ONE);
    assert!((PriceRatio::ONE / 0).is_nan());
    assert!((PriceRatio::NAN / 2).is_nan());
    assert!((PriceRatio::ONE + PriceRatio::NAN).is_nan());
    assert_eq!(
        PriceRatio::ONE.checked_sub(PriceRatio::ONE),
        Some(PriceRatio::ZERO)
    );
    assert_eq!(PriceRatio::ZERO.checked_sub(PriceRatio::ONE), None);
    assert_eq!(
        PriceRatio::NAN.checked_sub(PriceRatio::ONE),
        Some(PriceRatio::NAN)
    );
    for raw in [0u32, 123_457, 16_777_217, u32::MAX - 1] {
        assert_eq!(
            f32::from(PriceRatio::from_raw(raw)),
            PartsPerMillion64::new(raw as u64).to_f32()
        );
    }
}

#[test]
fn price_ratio_preserves_ppm_and_saturates_finite_overflow() {
    assert_eq!(PriceRatio::from(0.0), PriceRatio::ZERO);
    assert_eq!(PriceRatio::from(1.0), PriceRatio::ONE);
    assert_eq!(PriceRatio::from(0.123_456_6).inner(), 123_457);
    assert_eq!(PriceRatio::from(4_294.967_294), PriceRatio::MAX);
    assert_eq!(PriceRatio::from(10_000.0), PriceRatio::MAX);
    assert_eq!(PriceRatio::from(f64::MAX), PriceRatio::MAX);
    assert!(!PriceRatio::MAX.is_nan());
    assert!(PriceRatio::MAX.is_saturated());
    assert!(!PriceRatio::NAN.is_saturated());
    for raw in [
        0,
        1,
        1_000_000,
        u32::MAX as u64 - 2,
        u32::MAX as u64 - 1,
        u32::MAX as u64,
        u64::MAX - 1,
    ] {
        assert_eq!(
            PriceRatio::from(PartsPerMillion64::new(raw)).inner(),
            raw.min(u32::MAX as u64 - 1) as u32
        );
    }
    assert!(PriceRatio::from(PartsPerMillion64::NAN).is_nan());
}

#[test]
fn undefined_float_values_stay_undefined() {
    for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        assert!(PriceRatio::from(value).is_nan());
        assert!(BoundedRatio::from(value).is_nan());
        assert!(BasisPoints32::from(value).is_nan());
    }
    assert!(f64::from(PriceRatio::NAN).is_nan());
    assert!(f64::from(BoundedRatio::NAN).is_nan());
    assert!(f64::from(BasisPoints32::NAN).is_nan());
}

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

#[test]
fn basis_points_floor_instead_of_rounding() {
    assert_eq!(BasisPoints32::from(0.0), BasisPoints32::ZERO);
    assert_eq!(BasisPoints32::from(1.0), BasisPoints32::ONE);
    assert_eq!(BasisPoints32::from(0.123_499).inner(), 1_234);
    assert_eq!(BasisPoints32::from(429_496.729_4), BasisPoints32::MAX);
    assert_eq!(BasisPoints32::from(429_496.729_499), BasisPoints32::MAX);
    for raw in [0, 1, 99, 100, 199, 1_000_000, 429_496_729_499] {
        assert_eq!(
            BasisPoints32::from(PartsPerMillion64::new(raw)).inner() as u64,
            raw / 100
        );
    }
    assert!(BasisPoints32::from(PartsPerMillion64::NAN).is_nan());
}

#[test]
fn basis_point_arithmetic_preserves_units_and_undefined() {
    let mut value = BasisPoints32::ONE;
    value += BasisPoints32::ONE;
    assert_eq!(value, BasisPoints32::from(2.0));
    assert_eq!(value / 2, BasisPoints32::ONE);
    assert_eq!(BasisPoints32::from(10_000usize), BasisPoints32::ONE);
    assert!((value / 0).is_nan());
    assert!((value + BasisPoints32::NAN).is_nan());
    assert_eq!(
        value.checked_sub(BasisPoints32::ONE),
        Some(BasisPoints32::ONE)
    );
    assert_eq!(BasisPoints32::ZERO.checked_sub(BasisPoints32::ONE), None);
    assert_eq!(
        BasisPoints32::NAN.checked_sub(value),
        Some(BasisPoints32::NAN)
    );
}

#[test]
fn saturation_keeps_nupl_finite_with_the_agreed_error_bound() {
    let capped = f64::from(PriceRatio::from(10_000.0));
    let nupl = 1.0 - 1.0 / capped;
    assert!(nupl.is_finite());
    assert!((1.0 - nupl) * 100.0 < 0.023_284);
    assert_eq!(PartsPerMillionSigned32::from(nupl).inner(), 999_767);
}

#[test]
#[cfg(feature = "storage")]
fn encoded_serialization_schema_and_pco_preserve_all_bit_patterns() {
    macro_rules! check {
        ($ty:ty) => {{
            assert_eq!(std::mem::size_of::<$ty>(), 4);
            assert!(<$ty as Pco>::IS_TRANSPARENT);
            let bits = [0, 1, 1_000_000, u32::MAX - 1, u32::MAX];
            let values: Vec<$ty> = bits.into_iter().map(<$ty>::from_raw).collect();
            for value in &values {
                let encoded = serde_json::to_string(value).unwrap();
                assert_eq!(serde_json::from_str::<$ty>(&encoded).unwrap(), *value);
                assert_eq!(encoded, value.inner().to_string());
                let mut json = Vec::new();
                value.fmt_json(&mut json);
                assert_eq!(
                    json,
                    if value.is_nan() {
                        b"null".to_vec()
                    } else {
                        encoded.into_bytes()
                    }
                );
                assert_eq!(
                    <$ty as Pco>::from_number(value.to_number()).unwrap(),
                    *value
                );
            }
            let compressed =
                pco::standalone::simple_compress(&bits, &pco::ChunkConfig::default()).unwrap();
            let decoded = pco::standalone::simple_decompress::<u32>(&compressed).unwrap();
            let restored: Vec<$ty> = decoded
                .into_iter()
                .map(|x| <$ty as Pco>::from_number(x).unwrap())
                .collect();
            assert_eq!(restored, values);
            let schema = serde_json::to_value(schemars::schema_for!($ty)).unwrap();
            assert_eq!(schema["type"], "integer");
        }};
    }
    check!(PriceRatio);
    check!(BoundedRatio);
    check!(BasisPoints32);
}
