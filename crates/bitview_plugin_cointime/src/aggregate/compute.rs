use std::iter;

use bitview_cohort::{AgeRange, AgeRangeId, ByTerm, Term};
use bitview_compute::WeightedCohortState;
use bitview_plugin_distribution::Vecs as DistributionVecs;
use bitview_plugin_indexer::Indexer;
use bitview_vecs::PerBlock;
use brk_error::Result;
use brk_exit::Exit;
use brk_types::{BoundedRatio, Cents, CentsSats, CentsSquaredSats, Height, Sats, Version};
use vecdb::{AnyStoredVec, AnyVec, CachePolicy, EagerVec, PcoVec, ReadableVec, WritableVec};

use super::{super::AgeRangeVecs, Sources, Vecs};

const WRITE_INTERVAL: usize = 10_000;

pub fn compute(
    vecs: &mut Vecs,
    indexer: &Indexer,
    distribution: &DistributionVecs,
    age_range: &mut AgeRangeVecs,
    all_supply_in_loss_share: &mut PerBlock<BoundedRatio>,
    exit: &Exit,
) -> Result<()> {
    let starting_height = indexer.safe_lengths().height;
    let supplies = AgeRange::from_fn(|id| {
        &id.select(&distribution.cohorts.supply.total.cohorts.utxo.age)
            .sats
            .height
    });
    let loss_supplies = AgeRange::from_fn(|id| {
        &id.select(&distribution.cohorts.supply.in_loss.cohorts.age)
            .sats
            .height
    });
    let realized_caps = AgeRange::from_fn(|id| {
        &id.select(&distribution.cohorts.realized.cap.cohorts.utxo.age)
            .cents
            .height
    });
    let cap_raw = AgeRange::from_fn(|id| id.select(&distribution.cohorts.realized.cap_raw.age));
    let capitalized_cap_raw =
        AgeRange::from_fn(|id| id.select(&distribution.cohorts.realized.capitalized_cap_raw.age));
    let weights = AgeRange::from_fn(|id| id.select(&age_range.activity_sources));

    vecs.sources.compute_primary(
        starting_height,
        &supplies,
        &loss_supplies,
        &realized_caps,
        &cap_raw,
        &capitalized_cap_raw,
        &weights,
        &mut all_supply_in_loss_share.height,
        exit,
    )
}

impl Sources {
    #[allow(clippy::too_many_arguments)]
    fn compute_primary<S, L, C, W>(
        &mut self,
        starting_height: Height,
        supplies: &AgeRange<&S>,
        loss_supplies: &AgeRange<&L>,
        realized_caps: &AgeRange<&C>,
        cap_raw: &AgeRange<&impl ReadableVec<Height, CentsSats>>,
        capitalized_cap_raw: &AgeRange<&impl ReadableVec<Height, CentsSquaredSats>>,
        weights: &AgeRange<&W>,
        all_supply_in_loss_share: &mut EagerVec<PcoVec<Height, BoundedRatio, impl CachePolicy>>,
        exit: &Exit,
    ) -> Result<()>
    where
        S: ReadableVec<Height, Sats>,
        L: ReadableVec<Height, Sats>,
        C: ReadableVec<Height, Cents>,
        W: ReadableVec<Height, BoundedRatio>,
    {
        let source_version = Version::combine_all(
            supplies
                .iter()
                .map(|vec| vec.version())
                .chain(loss_supplies.iter().map(|vec| vec.version()))
                .chain(realized_caps.iter().map(|vec| vec.version()))
                .chain(cap_raw.iter().map(|vec| vec.version()))
                .chain(capitalized_cap_raw.iter().map(|vec| vec.version()))
                .chain(weights.iter().map(|vec| vec.version())),
        );

        for vec in self.primary_vecs_mut() {
            vec.any_validate_computed_version_or_reset(source_version)?;
        }
        all_supply_in_loss_share.any_validate_computed_version_or_reset(source_version)?;

        let start = self
            .primary_vecs_mut()
            .into_iter()
            .map(|vec| vec.len())
            .chain(iter::once(all_supply_in_loss_share.len()))
            .min()
            .unwrap_or_default()
            .min(usize::from(starting_height));

        for vec in self.primary_vecs_mut() {
            vec.any_truncate_if_needed_at(start)?;
        }
        all_supply_in_loss_share.truncate_if_needed_at(start)?;
        let aggregate_end = supplies
            .iter()
            .map(|vec| vec.len())
            .chain(loss_supplies.iter().map(|vec| vec.len()))
            .chain(realized_caps.iter().map(|vec| vec.len()))
            .chain(cap_raw.iter().map(|vec| vec.len()))
            .chain(capitalized_cap_raw.iter().map(|vec| vec.len()))
            .chain(weights.iter().map(|vec| vec.len()))
            .min()
            .unwrap_or_default();

        let mut chunk_start = start;
        while chunk_start < aggregate_end {
            let chunk_end = (chunk_start + WRITE_INTERVAL).min(aggregate_end);
            let aggregate_start = chunk_start.min(aggregate_end);
            let aggregate_chunk_end = chunk_end.min(aggregate_end);
            let supply_batches = AgeRange::from_fn(|id| {
                id.select(supplies).collect_range_at(chunk_start, chunk_end)
            });
            let loss_batches = AgeRange::from_fn(|id| {
                id.select(loss_supplies)
                    .collect_range_at(aggregate_start, aggregate_chunk_end)
            });
            let cap_batches = AgeRange::from_fn(|id| {
                id.select(realized_caps)
                    .collect_range_at(aggregate_start, aggregate_chunk_end)
            });
            let raw_batches =
                AgeRange::from_fn(|id| id.select(cap_raw).collect_range_at(chunk_start, chunk_end));
            let capitalized_batches = AgeRange::from_fn(|id| {
                id.select(capitalized_cap_raw)
                    .collect_range_at(chunk_start, chunk_end)
            });
            let weight_batch =
                AgeRange::from_fn(|id| id.select(weights).collect_range_at(chunk_start, chunk_end));

            for offset in 0..chunk_end - chunk_start {
                let mut terms = ByTerm::<WeightedCohortState>::default();
                let mut under_4m = WeightedCohortState::default();
                let mut under_6m = WeightedCohortState::default();
                let mut over_4m = WeightedCohortState::default();
                let mut over_6m = WeightedCohortState::default();
                for &id in AgeRangeId::ALL {
                    let term = if id.term() == Term::Sth {
                        &mut terms.short
                    } else {
                        &mut terms.long
                    };
                    let mut contribution = WeightedCohortState::default();
                    contribution.capitalized_price.add(
                        id.select(&raw_batches)[offset],
                        id.select(&capitalized_batches)[offset],
                        id.select(&weight_batch)[offset],
                    );
                    contribution.add(
                        id.select(&supply_batches)[offset],
                        id.select(&loss_batches)[offset],
                        id.select(&cap_batches)[offset],
                        id.select(&weight_batch)[offset],
                    );
                    *term = term.merged(contribution);
                    if id >= AgeRangeId::From4MTo5M {
                        over_4m = over_4m.merged(contribution);
                    }
                    if id >= AgeRangeId::From6MTo9M {
                        over_6m = over_6m.merged(contribution);
                    }
                    // AgeRangeId::ALL is ordered youngest to oldest. Capture
                    // exact aggregate states at the two additional cutoffs.
                    if id == AgeRangeId::From3MTo4M {
                        under_4m = terms.short;
                    } else if id == AgeRangeId::From5MTo6M {
                        under_6m = terms.short.merged(terms.long);
                    }
                }
                self.under_4m_awake_price.push(under_4m.realized_price());
                self.under_4m_awake_capitalized_price
                    .push(under_4m.capitalized_price.value());
                self.under_6m_awake_price.push(under_6m.realized_price());
                self.under_6m_awake_capitalized_price
                    .push(under_6m.capitalized_price.value());
                self.over_4m_awake_price.push(over_4m.realized_price());
                self.over_4m_awake_capitalized_price
                    .push(over_4m.capitalized_price.value());
                self.over_6m_awake_price.push(over_6m.realized_price());
                self.over_6m_awake_capitalized_price
                    .push(over_6m.capitalized_price.value());
                let all = terms.short.merged(terms.long);
                all_supply_in_loss_share.push(all.supply_in_loss.value());
                self.push(terms, all);
            }

            {
                let _lock = exit.lock();
                for vec in self.primary_vecs_mut() {
                    vec.write()?;
                }
                all_supply_in_loss_share.write()?;
            }
            chunk_start = chunk_end;
        }

        Ok(())
    }

    fn push(&mut self, terms: ByTerm<WeightedCohortState>, all: WeightedCohortState) {
        for (target, value) in [
            (&mut self.awake_supply.all, all.weighted_supply),
            (&mut self.awake_supply.sth, terms.short.weighted_supply),
            (&mut self.awake_supply.lth, terms.long.weighted_supply),
            (&mut self.dormant_supply.all, all.complement_supply),
            (&mut self.dormant_supply.sth, terms.short.complement_supply),
            (&mut self.dormant_supply.lth, terms.long.complement_supply),
        ] {
            target.push(value);
        }
        for (target, value) in [
            (&mut self.awake_cap.all, all.weighted_cap),
            (&mut self.awake_cap.sth, terms.short.weighted_cap),
            (&mut self.awake_cap.lth, terms.long.weighted_cap),
            (&mut self.awake_price.all, all.realized_price()),
            (&mut self.awake_price.sth, terms.short.realized_price()),
            (&mut self.awake_price.lth, terms.long.realized_price()),
            (
                &mut self.awake_capitalized_price.all,
                all.capitalized_price.value(),
            ),
            (
                &mut self.awake_capitalized_price.sth,
                terms.short.capitalized_price.value(),
            ),
            (
                &mut self.awake_capitalized_price.lth,
                terms.long.capitalized_price.value(),
            ),
        ] {
            target.push(value);
        }
        self.supply_in_loss_share
            .short
            .push(terms.short.supply_in_loss.value());
        self.supply_in_loss_share
            .long
            .push(terms.long.supply_in_loss.value());
    }

    fn primary_vecs_mut(&mut self) -> Vec<&mut dyn AnyStoredVec> {
        vec![
            &mut self.over_4m_awake_price,
            &mut self.over_4m_awake_capitalized_price,
            &mut self.over_6m_awake_price,
            &mut self.over_6m_awake_capitalized_price,
            &mut self.under_4m_awake_price,
            &mut self.under_4m_awake_capitalized_price,
            &mut self.under_6m_awake_price,
            &mut self.under_6m_awake_capitalized_price,
            &mut self.awake_supply.all,
            &mut self.awake_supply.sth,
            &mut self.awake_supply.lth,
            &mut self.dormant_supply.all,
            &mut self.dormant_supply.sth,
            &mut self.dormant_supply.lth,
            &mut self.awake_cap.all,
            &mut self.awake_cap.sth,
            &mut self.awake_cap.lth,
            &mut self.awake_price.all,
            &mut self.awake_price.sth,
            &mut self.awake_price.lth,
            &mut self.awake_capitalized_price.all,
            &mut self.awake_capitalized_price.sth,
            &mut self.awake_capitalized_price.lth,
            &mut self.supply_in_loss_share.short,
            &mut self.supply_in_loss_share.long,
        ]
    }
}

#[cfg(test)]
mod tests {
    use crate::test_cache::init_cache;

    use tempfile::tempdir;
    use vecdb::{BytesVec, Database, ImportableVec};

    use super::*;

    #[test]
    fn age_threshold_prices_use_exact_membership_and_backfill() {
        init_cache();
        let directory = tempdir().unwrap();
        let db = Database::open(directory.path()).unwrap();
        let mut sources = Sources::forced_import(&db, Version::ONE).unwrap();
        let mut loss_share = EagerVec::<PcoVec<Height, BoundedRatio>>::forced_import(
            &db,
            "loss_share",
            Version::ONE,
        )
        .unwrap();
        let mut supply =
            PcoVec::<Height, Sats>::forced_import(&db, "supply", Version::ONE).unwrap();
        let mut weight =
            PcoVec::<Height, BoundedRatio>::forced_import(&db, "weight", Version::ONE).unwrap();
        supply.push(Sats::from(100_000_000_u64));
        weight.push(BoundedRatio::ONE);
        supply.write().unwrap();
        weight.write().unwrap();
        let caps = AgeRange::from_fn(|id| {
            let mut value = PcoVec::<Height, Cents>::forced_import(
                &db,
                &format!("cap_{}", id.index()),
                Version::ONE,
            )
            .unwrap();
            value.push(Cents::new((id.index() as u64 + 1) * 100));
            value.write().unwrap();
            value
        });
        let raw = AgeRange::from_fn(|id| {
            let mut value = BytesVec::<Height, CentsSats>::forced_import(
                &db,
                &format!("raw_{}", id.index()),
                Version::ONE,
            )
            .unwrap();
            value.push(CentsSats::new(
                (id.index() as u128 + 1) * 100 * Sats::ONE_BTC_U128,
            ));
            value.write().unwrap();
            value
        });
        let squared = AgeRange::from_fn(|id| {
            let mut value = BytesVec::<Height, CentsSquaredSats>::forced_import(
                &db,
                &format!("squared_{}", id.index()),
                Version::ONE,
            )
            .unwrap();
            value.push(CentsSquaredSats::new(
                ((id.index() as u128 + 1) * 100).pow(2) * Sats::ONE_BTC_U128,
            ));
            value.write().unwrap();
            value
        });
        for start in [0_usize, 1] {
            sources
                .over_6m_awake_price
                .truncate_if_needed_at(0)
                .unwrap();
            sources
                .compute_primary(
                    Height::from(start),
                    &AgeRange::from_fn(|_| &supply),
                    &AgeRange::from_fn(|_| &supply),
                    &AgeRange::from_fn(|id| id.select(&caps)),
                    &AgeRange::from_fn(|id| id.select(&raw)),
                    &AgeRange::from_fn(|id| id.select(&squared)),
                    &AgeRange::from_fn(|_| &weight),
                    &mut loss_share,
                    &Exit::default(),
                )
                .unwrap();
            // Seven, eight, and nine disjoint age ranges below 120/150/180 days.
            for (price, capitalized, count) in [
                (
                    &sources.under_4m_awake_price,
                    &sources.under_4m_awake_capitalized_price,
                    7_u64,
                ),
                (
                    &sources.awake_price.sth,
                    &sources.awake_capitalized_price.sth,
                    8,
                ),
                (
                    &sources.under_6m_awake_price,
                    &sources.under_6m_awake_capitalized_price,
                    9,
                ),
            ] {
                assert_eq!(price.len(), 1);
                assert_eq!(
                    price.collect_one_at(0),
                    Some(Cents::new(100 * (count + 1) / 2))
                );
                assert_eq!(
                    capitalized.collect_one_at(0),
                    Some(Cents::new(100 * (2 * count + 1) / 3))
                );
            }
            for (price, capitalized, first) in [
                (
                    &sources.over_4m_awake_price,
                    &sources.over_4m_awake_capitalized_price,
                    8_u64,
                ),
                (
                    &sources.awake_price.lth,
                    &sources.awake_capitalized_price.lth,
                    9,
                ),
                (
                    &sources.over_6m_awake_price,
                    &sources.over_6m_awake_capitalized_price,
                    10,
                ),
            ] {
                let last = AgeRangeId::ALL.len() as u64;
                let sum: u64 = (first..=last).sum();
                let squared: u64 = (first..=last).map(|value| value * value).sum();
                assert_eq!(price.len(), 1);
                assert_eq!(capitalized.len(), 1);
                assert_eq!(
                    price.collect_one_at(0),
                    Some(Cents::new(100 * sum / (last - first + 1)))
                );
                assert_eq!(
                    capitalized.collect_one_at(0),
                    Some(Cents::new(100 * squared / sum))
                );
            }
        }
    }

    #[test]
    fn shared_batches_cover_boundaries_short_inputs_and_rewrites() {
        init_cache();

        let directory = tempdir().unwrap();
        let db = Database::open(directory.path()).unwrap();
        let mut sources = Sources::forced_import(&db, Version::ONE).unwrap();
        let mut loss_share = EagerVec::<PcoVec<Height, BoundedRatio>>::forced_import(
            &db,
            "loss_share",
            Version::ONE,
        )
        .unwrap();
        let mut supply =
            PcoVec::<Height, Sats>::forced_import(&db, "supply", Version::ONE).unwrap();
        let mut loss = PcoVec::<Height, Sats>::forced_import(&db, "loss", Version::ONE).unwrap();
        let mut cap = PcoVec::<Height, Cents>::forced_import(&db, "cap", Version::ONE).unwrap();
        let mut raw =
            BytesVec::<Height, CentsSats>::forced_import(&db, "raw", Version::ONE).unwrap();
        let mut capitalized =
            BytesVec::<Height, CentsSquaredSats>::forced_import(&db, "capitalized", Version::ONE)
                .unwrap();
        let mut weights =
            PcoVec::<Height, BoundedRatio>::forced_import(&db, "weights", Version::ONE).unwrap();
        let length = WRITE_INTERVAL + 3;
        for height in 0..length {
            supply.push(Sats::from(100_u64));
            weights.push(BoundedRatio::from(0.5));
            if height < length - 1 {
                loss.push(Sats::from(20_u64));
                cap.push(Cents::from(1_000_u64));
                raw.push(CentsSats::new(1_000 * Sats::ONE_BTC_U128));
                capitalized.push(CentsSquaredSats::new(3_000 * 1_000 * Sats::ONE_BTC_U128));
            }
        }
        supply.write().unwrap();
        loss.write().unwrap();
        cap.write().unwrap();
        raw.write().unwrap();
        capitalized.write().unwrap();
        weights.write().unwrap();

        for rewrite in [false, true] {
            let start = if rewrite { WRITE_INTERVAL } else { 0 };
            if rewrite {
                weights.truncate_if_needed_at(start).unwrap();
                for _ in start..length {
                    weights.push(BoundedRatio::ONE);
                }
                weights.write().unwrap();
                capitalized.truncate_if_needed_at(start).unwrap();
                for _ in start..length - 1 {
                    capitalized.push(CentsSquaredSats::new(4_000 * 1_000 * Sats::ONE_BTC_U128));
                }
                capitalized.write().unwrap();
                // A missing new series must backfill even when the existing
                // supply/price sources and the indexer are already ahead.
                sources
                    .awake_capitalized_price
                    .sth
                    .truncate_if_needed_at(0)
                    .unwrap();
            }
            sources
                .compute_primary(
                    Height::from(start),
                    &AgeRange::from_fn(|_| &supply),
                    &AgeRange::from_fn(|_| &loss),
                    &AgeRange::from_fn(|_| &cap),
                    &AgeRange::from_fn(|_| &raw),
                    &AgeRange::from_fn(|_| &capitalized),
                    &AgeRange::from_fn(|_| &weights),
                    &mut loss_share,
                    &Exit::default(),
                )
                .unwrap();
            assert_eq!(loss_share.len(), length - 1);
            for height in [
                0,
                WRITE_INTERVAL - 1,
                WRITE_INTERVAL,
                length - 2,
                length - 1,
            ] {
                let weight = if rewrite && height >= start {
                    BoundedRatio::ONE
                } else {
                    BoundedRatio::from(0.5)
                };
                let (awake, _) = WeightedCohortState::split_supply(Sats::from(100_u64), weight);
                if height < length - 1 {
                    for price in sources.awake_capitalized_price.iter() {
                        assert_eq!(price.len(), length - 1);
                        let expected = if rewrite && height >= start {
                            4_000
                        } else {
                            3_000
                        };
                        assert_eq!(price.collect_one_at(height).unwrap(), Cents::new(expected));
                    }
                    let short = sources.awake_supply.sth.collect_one_at(height).unwrap();
                    let long = sources.awake_supply.lth.collect_one_at(height).unwrap();
                    assert_eq!(
                        short + long,
                        Sats::from(u64::from(awake) * AgeRangeId::ALL.len() as u64)
                    );
                }
            }
        }
    }

    #[test]
    fn term_states_merge_into_all() {
        let mut sth = WeightedCohortState::default();
        sth.add(
            Sats::from(100_u64),
            Sats::from(20_u64),
            Cents::from(1_000_u64),
            BoundedRatio::from(0.3),
        );
        let mut lth = WeightedCohortState::default();
        lth.add(
            Sats::from(200_u64),
            Sats::from(50_u64),
            Cents::from(3_000_u64),
            BoundedRatio::from(0.4),
        );

        let all = sth.merged(lth);

        assert_eq!(
            all.weighted_supply,
            sth.weighted_supply + lth.weighted_supply
        );
        assert_eq!(
            all.complement_supply,
            sth.complement_supply + lth.complement_supply
        );
        assert_eq!(all.weighted_cap, sth.weighted_cap + lth.weighted_cap);
    }

    #[test]
    fn awake_and_dormant_supply_are_independently_floored() {
        let supply = Sats::from(123_456_789_u64);
        let weight = BoundedRatio::from(0.321);
        let mut state = WeightedCohortState::default();

        state.add(supply, Sats::ZERO, Cents::ZERO, weight);

        let sum = state.weighted_supply + state.complement_supply;
        assert!(sum <= supply);
        assert!(supply - sum <= Sats::from(1_u64));
    }
}
