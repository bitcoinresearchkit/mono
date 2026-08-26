use bitview_cohort::UTXOAggregateId;
use brk_types::UrpdRaw;

use super::AgeRangeUrpds;

impl AgeRangeUrpds {
    pub fn aggregate(&self, id: UTXOAggregateId) -> UrpdRaw {
        let entries = id
            .age_range_ids()
            .iter()
            .map(|&id| self.get(id))
            .filter(|entries| !entries.is_empty())
            .fold(Vec::new(), |left, right| Self::merge_sorted(&left, right));
        UrpdRaw {
            map: entries.into_iter().collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use bitview_cohort::{AGE_RANGE_COUNT, AgeRange, STH_AGE_RANGE_COUNT, UTXOAggregateId};
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
            AgeRangeUrpds::merge_sorted(&left, &right),
            vec![
                (CentsCompact::new(100), Sats::from(1_u64)),
                (CentsCompact::new(200), Sats::from(2_u64)),
                (CentsCompact::new(300), Sats::from(7_u64)),
                (CentsCompact::new(400), Sats::from(5_u64)),
            ]
        );
        assert_eq!(AgeRangeUrpds::merge_sorted(&[], &right), right);
        assert_eq!(AgeRangeUrpds::merge_sorted(&left, &[]), left);
    }

    #[test]
    fn aggregates_select_the_exact_age_ranges() {
        let urpds = AgeRangeUrpds {
            entries: AgeRange::from_fn(|_| vec![(CentsCompact::new(100), Sats::from(1_u64))]),
        };

        assert_eq!(
            urpds.aggregate(UTXOAggregateId::All).map[&CentsCompact::new(100)],
            Sats::from(AGE_RANGE_COUNT as u64)
        );
        assert_eq!(
            urpds.aggregate(UTXOAggregateId::Sth).map[&CentsCompact::new(100)],
            Sats::from(STH_AGE_RANGE_COUNT as u64)
        );
        assert_eq!(
            urpds.aggregate(UTXOAggregateId::Lth).map[&CentsCompact::new(100)],
            Sats::from((AGE_RANGE_COUNT - STH_AGE_RANGE_COUNT) as u64)
        );
    }
}
