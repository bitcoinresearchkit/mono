use brk_types::{CentsCompact, Sats};

use super::AgeRangeUrpds;

#[test]
fn sorted_merge_preserves_order_and_sums_equal_prices() {
    let left = [
        (CentsCompact::new(100), Sats::from(1_u64)),
        (CentsCompact::new(300), Sats::from(3_u64)),
    ];
    let right = [
        (CentsCompact::new(200), Sats::from(2_u64)),
        (CentsCompact::new(300), Sats::from(4_u64)),
        (CentsCompact::new(400), Sats::from(5_u64)),
    ];

    assert_eq!(
        AgeRangeUrpds::merge_sorted(&left, &right).unwrap(),
        vec![
            (CentsCompact::new(100), Sats::from(1_u64)),
            (CentsCompact::new(200), Sats::from(2_u64)),
            (CentsCompact::new(300), Sats::from(7_u64)),
            (CentsCompact::new(400), Sats::from(5_u64)),
        ]
    );
    assert_eq!(AgeRangeUrpds::merge_sorted(&[], &right).unwrap(), right);
    assert_eq!(AgeRangeUrpds::merge_sorted(&left, &[]).unwrap(), left);
    let huge = [(CentsCompact::new(100), Sats::from(u64::MAX))];
    assert!(AgeRangeUrpds::merge_sorted(&huge, &huge).is_err());
}
