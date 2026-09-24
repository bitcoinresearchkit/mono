use bitview_cohort::{AgeRange, AgeRangeId, Term, UTXOAggregateId};
use brk_types::{CentsCompact, Sats};

use super::{AgeCutoffs, accumulate_masses, collect_mass};
use crate::UrpdRaw;

#[derive(Default)]
struct Masses {
    all: f64,
    age: AgeCutoffs<f64>,
    long: f64,
}

pub struct DailyUrpds {
    pub all: UrpdRaw,
    pub age: AgeCutoffs<UrpdRaw>,
    pub long: UrpdRaw,
}

impl DailyUrpds {
    pub fn from_age_entries(
        entries: impl IntoIterator<Item = (AgeRangeId, CentsCompact, Sats)>,
        weights: &AgeRange<f64>,
    ) -> Self {
        let buckets = accumulate_masses(entries, |bucket: &mut Masses, age, _, mass| {
            let mass = u64::from(mass) as f64 * *age.select(weights);
            bucket.all += mass;
            for value in bucket.age.containing_mut(age) {
                *value += mass;
            }
            if age.term() == Term::Lth {
                bucket.long += mass;
            }
        });
        Self {
            all: collect_mass(&buckets, |b| b.all),
            age: AgeCutoffs {
                under_4m: collect_mass(&buckets, |b| b.age.under_4m),
                under_5m: collect_mass(&buckets, |b| b.age.under_5m),
                under_6m: collect_mass(&buckets, |b| b.age.under_6m),
            },
            long: collect_mass(&buckets, |b| b.long),
        }
    }

    pub fn aggregate(&self, id: UTXOAggregateId) -> &UrpdRaw {
        match id {
            UTXOAggregateId::All => &self.all,
            UTXOAggregateId::Sth => &self.age.under_5m,
            UTXOAggregateId::Lth => &self.long,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use brk_types::Cents;

    use crate::metrics::capitalized_price;

    #[test]
    fn cohorts_share_rounding_but_keep_their_exact_age_boundaries() {
        let entries = [
            (AgeRangeId::Under1H, 100, 1),
            (AgeRangeId::From3MTo4M, 100, 1),
            (AgeRangeId::From4MTo5M, 200, 3),
            (AgeRangeId::From5MTo6M, 300, 4),
            (AgeRangeId::From6MTo9M, 400, 2),
        ]
        .map(|(age, p, s)| (age, CentsCompact::new(p), Sats::from(s as u64)));
        let urpds = DailyUrpds::from_age_entries(entries, &AgeRange::from_fn(|_| 0.5));
        let values = |urpd: &UrpdRaw| {
            urpd.map
                .iter()
                .map(|(p, s)| (p.inner(), u64::from(*s)))
                .collect::<Vec<_>>()
        };
        assert_eq!(values(&urpds.all), [(100, 1), (200, 1), (300, 2), (400, 1)]);
        assert_eq!(values(&urpds.age.under_4m), [(100, 1)]);
        assert_eq!(values(&urpds.age.under_5m), [(100, 1), (200, 1)]);
        assert_eq!(values(&urpds.age.under_6m), [(100, 1), (200, 1), (300, 2)]);
        assert_eq!(values(&urpds.long), [(300, 2), (400, 1)]);
        assert_eq!(
            capitalized_price(urpds.all.map.iter().map(|(&p, &s)| (p, s))),
            Cents::new(300)
        );
        let bounds = AgeCutoffs::from_age_entries(entries);
        assert_eq!(bounds.under_4m.max, Cents::new(100));
        assert_eq!(bounds.under_5m.max, Cents::new(200));
        assert_eq!(bounds.under_6m.max, Cents::new(300));
    }

    #[test]
    fn zero_weights_leave_no_occupied_price_buckets() {
        let urpds = DailyUrpds::from_age_entries(
            [(
                AgeRangeId::Under1H,
                CentsCompact::new(100),
                Sats::from(1_u64),
            )],
            &AgeRange::from_fn(|_| 0.0),
        );
        assert!(urpds.all.map.is_empty());
        assert!(urpds.age.iter().all(|u| u.map.is_empty()));
        assert!(urpds.long.map.is_empty());
    }
}
