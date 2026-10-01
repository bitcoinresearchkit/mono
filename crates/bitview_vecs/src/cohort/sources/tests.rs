use crate::test_cache::init_cache;

use bitview_cohort::{AgeRange, AgeRangeId, CohortId, Term, UTXOCoreValues};
use brk_types::{Cents, Height, OutputType, Sats, Version};
use tempfile::tempdir;
use vecdb::{Database, ReadableVec};

use super::{CreationSources, CumulativeCreationValueSources};
use crate::SatsCents;

#[test]
fn disjoint_and_cumulative_sources_share_selection_without_aggregate_storage() {
    init_cache();
    let directory = tempdir().unwrap();
    let db = Database::open(directory.path()).unwrap();
    let mut direct = CreationSources::<Sats>::forced_import(&db, "direct", Version::ONE).unwrap();
    let mut cumulative =
        CumulativeCreationValueSources::forced_import(&db, "cumulative", Version::ONE).unwrap();
    let sats = UTXOCoreValues {
        age_range: AgeRange::from_fn(|id| Sats::from(id.index() as u64 + 1)),
        ..Default::default()
    };
    let cents = sats.map(|value| Cents::from(u64::from(*value) * 10));
    for _ in 0..2 {
        direct.push(sats.clone());
        cumulative.push_block(sats.clone(), cents.clone());
    }
    for vector in direct
        .collect_vecs_mut()
        .into_iter()
        .chain(cumulative.collect_vecs_mut())
    {
        vector.write().unwrap();
    }
    for &id in AgeRangeId::ALL {
        let value = id.index() as u64 + 1;
        assert_eq!(
            direct.get(id.cohort()).unwrap().collect_range_at(0, 2),
            [Sats::from(value); 2]
        );
        let SatsCents { sats, cents } = cumulative
            .sources(id.cohort(), "sum", Version::ONE)
            .unwrap();
        assert_eq!(
            sats.collect_one(Height::from(1_usize)),
            Some(Sats::from(value * 2))
        );
        assert_eq!(
            cents.collect_one(Height::from(1_usize)),
            Some(Cents::from(value * 20))
        );
    }
    for unsupported in [
        CohortId::All,
        CohortId::Term(Term::Sth),
        CohortId::Term(Term::Lth),
        CohortId::Type(OutputType::OpReturn),
    ] {
        assert!(direct.get(unsupported).is_none());
        assert!(
            cumulative
                .sources(unsupported, "unsupported", Version::ONE)
                .is_none()
        );
    }
    // Mutable access invalidates the cumulative checkpoint before a rollback.
    for vector in cumulative.collect_vecs_mut() {
        vector.any_truncate_if_needed_at(1).unwrap();
    }
    cumulative.push_block(sats, cents);
    assert_eq!(
        cumulative
            .sats
            .stored
            .get(AgeRangeId::Under1H.cohort())
            .unwrap()
            .collect_one_at(1)
            .map(u64::from)
            .unwrap(),
        2
    );
}
