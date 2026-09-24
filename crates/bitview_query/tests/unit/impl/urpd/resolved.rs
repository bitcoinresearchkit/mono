use std::collections::BTreeMap;

use bitcoin::Amount;
use bitview_urpd::UrpdRaw;
use brk_error::Error;
use brk_types::{Cents, CentsCompact, Cohort, Date, Sats, UrpdAggregation, UrpdWeight};
use serde_json::to_value;

use super::*;

fn captured(bytes: Vec<u8>, close: Cents, scalar: f64) -> ResolvedUrpd {
    ResolvedUrpd {
        cohort: Cohort::new("all").unwrap(),
        date: Date::new(2026, 9, 1),
        weight: UrpdWeight::Cointime,
        aggregation: UrpdAggregation::Raw,
        scalar,
        close,
        input: UrpdInput::Raw(bytes),
    }
}

#[test]
fn captured_inputs_preserve_weighted_output() {
    let map = BTreeMap::from([
        (CentsCompact::new(100), Sats::from(3_u64)),
        (CentsCompact::new(200), Sats::from(1_u64)),
    ]);
    for scalar in [0.0, 0.5, 1.0] {
        let raw = UrpdRaw { map: map.clone() };
        let bytes = raw.serialize().unwrap();
        let close = Cents::from(300_u64);
        assert!(captured(bytes.clone(), close, scalar).validate().is_ok());
        let captured = captured(bytes.clone(), close, scalar);
        let mut count = 0;
        captured.for_each_section(|section| {
            assert_eq!(section, bytes);
            count += 1;
        });
        assert_eq!(count, 1);
        let expected = build_response(
            captured.cohort.clone(),
            captured.date,
            captured.weight,
            close,
            weighted_entries(raw.map.into_iter(), scalar),
            captured.aggregation,
        );
        assert_eq!(
            to_value(captured.build().unwrap()).unwrap(),
            to_value(expected).unwrap(),
        );
    }
}

#[test]
fn captured_input_still_requires_decoding() {
    let invalid = captured(Vec::new(), Cents::ZERO, 1.0);
    assert!(matches!(invalid.build(), Err(Error::Deserialization(_))));
    assert!(captured(Vec::new(), Cents::ZERO, 1.0).validate().is_err());
}

#[test]
fn invalid_weights_and_market_values_return_errors() {
    let bytes = UrpdRaw {
        map: BTreeMap::from([(
            CentsCompact::new(100),
            Sats::from(Amount::MAX_MONEY.to_sat()),
        )]),
    }
    .serialize()
    .unwrap();
    for scalar in [f64::NAN, f64::INFINITY, -1.0, 1.1] {
        assert!(
            captured(bytes.clone(), Cents::ZERO, scalar)
                .validate()
                .is_err()
        );
        assert!(
            captured(bytes.clone(), Cents::ZERO, scalar)
                .build()
                .is_err()
        );
    }
    for close in [Cents::NAN, Cents::new(u64::MAX - 1)] {
        assert!(captured(bytes.clone(), close, 1.0).validate().is_err());
        assert!(captured(bytes.clone(), close, 1.0).build().is_err());
    }
}
