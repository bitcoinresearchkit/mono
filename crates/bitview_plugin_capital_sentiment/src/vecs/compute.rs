use brk_error::Result;

use bitview_plugin::ComputePlugin;
use brk_indexer::Indexer;
use brk_types::{CapitalSentimentPhase, Cents, Day1, Height, StoredBool, StoredU8, Version};
use vecdb::{AnyStoredVec, AnyVec, Exit, ReadableVec, VecIndex, WritableVec};

use super::Vecs;
use bitview_compute::db_utils::validate_any_computed_version_or_reset;

const WRITE_INTERVAL_DAYS: usize = 1_000;

impl Vecs {
    fn compute_inner(
        &mut self,
        indexer: &Indexer,
        indexes: &bitview_plugin_indexes::Vecs,
        prices: &bitview_plugin_price::Vecs,
        distribution: &bitview_plugin_distribution::Vecs,
        moving_average: &bitview_plugin_market::MovingAverageVecs,
        exit: &Exit,
    ) -> Result<()> {
        self.db.sync_bg_tasks()?;

        let spot = &prices.spot.cents.height;
        let sma = &moving_average.sma._1y.cents.height;
        let all = &distribution
            .cohorts
            .realized
            .capitalized_price
            .series
            .all
            .cents
            .height;
        let sth = &distribution
            .cohorts
            .realized
            .capitalized_price
            .series
            .sth
            .cents
            .height;
        let lth = &distribution
            .cohorts
            .realized
            .capitalized_price
            .series
            .lth
            .cents
            .height;
        let first_height = &indexes.day1.first_height;

        let source_version: Version = [
            spot.version(),
            sma.version(),
            all.version(),
            sth.version(),
            lth.version(),
            first_height.version(),
        ]
        .into_iter()
        .sum();
        validate_any_computed_version_or_reset(&mut self.phase_code.day1, source_version)?;
        validate_any_computed_version_or_reset(&mut self.is_long.day1, source_version)?;

        let height_end = [spot.len(), sma.len(), all.len(), sth.len(), lth.len()]
            .into_iter()
            .min()
            .unwrap_or_default();
        let first_heights = first_height.collect();
        let source_end = indexes.day1.date.len().min(first_heights.len());
        let recompute_from = recompute_day(indexer, indexes)
            .map(usize::from)
            .unwrap_or_default();
        let start = self
            .phase_code
            .day1
            .len()
            .min(self.is_long.day1.len())
            .min(recompute_from)
            .min(source_end);
        self.phase_code.day1.any_truncate_if_needed_at(start)?;
        self.is_long.day1.any_truncate_if_needed_at(start)?;

        let mut is_long = start
            .checked_sub(1)
            .map(Day1::from)
            .and_then(|day| self.is_long.day1.collect_one(day))
            .is_some_and(|value| value.is_true());
        let mut previous_over_sth = start.checked_sub(1).map(|day| {
            let height = last_height_of_day(&first_heights, day, height_end);
            is_over_sth(
                height.and_then(|height| spot.collect_one(height)),
                height.and_then(|height| sth.collect_one(height)),
            )
        });
        for day_index in start..source_end {
            let height = last_height_of_day(&first_heights, day_index, height_end);
            let price = height.and_then(|height| spot.collect_one(height));
            let sth_price = height.and_then(|height| sth.collect_one(height));
            let over_sth = is_over_sth(price, sth_price);
            let code = classify_phase_code(
                price,
                height.and_then(|height| all.collect_one(height)),
                sth_price,
                height.and_then(|height| lth.collect_one(height)),
                height.and_then(|height| sma.collect_one(height)),
            );
            is_long = next_is_long(is_long, previous_over_sth, over_sth, code);

            self.phase_code.day1.push(code);
            self.is_long.day1.push(StoredBool::from(is_long));
            previous_over_sth = Some(over_sth);

            if (day_index + 1).is_multiple_of(WRITE_INTERVAL_DAYS) || day_index + 1 == source_end {
                let _lock = exit.lock();
                self.phase_code.day1.write()?;
                self.is_long.day1.write()?;
            }
        }

        let exit = exit.clone();
        self.db.run_bg(move |db| {
            let _lock = exit.lock();
            db.compact_deferred_default()
        });

        Ok(())
    }
}

impl ComputePlugin for Vecs {
    type Dependencies<'a> = crate::Dependencies<'a>;
    type Output = ();

    fn compute(
        &mut self,
        dependencies: Self::Dependencies<'_>,
        exit: &Exit,
    ) -> Result<Self::Output> {
        self.compute_inner(
            dependencies.indexer,
            dependencies.indexes,
            dependencies.price,
            dependencies.distribution,
            dependencies.moving_average,
            exit,
        )
    }
}

fn last_height_of_day(first_heights: &[Height], day: usize, height_end: usize) -> Option<Height> {
    let first = first_heights.get(day)?.to_usize().min(height_end);
    let end = first_heights
        .get(day + 1)
        .map(|height| height.to_usize())
        .unwrap_or(height_end)
        .min(height_end);
    (first < end).then(|| Height::from(end - 1))
}

/// Advance the stateful short/long strategy used by BRK Signal.
fn next_is_long(
    is_long: bool,
    previous_over_sth: Option<bool>,
    over_sth: bool,
    phase_code: StoredU8,
) -> bool {
    let crossed_above_sth = previous_over_sth.is_some_and(|previous| !previous && over_sth);

    if !is_long && crossed_above_sth {
        return true;
    }
    if is_long && CapitalSentimentPhase::from_code(*phase_code).is_some_and(|phase| phase.is_sell())
    {
        return false;
    }
    is_long
}

#[inline]
fn is_finite_positive(value: Cents) -> bool {
    !value.is_nan() && value > Cents::ZERO
}

#[inline]
fn is_over_sth(price: Option<Cents>, sth: Option<Cents>) -> bool {
    price.zip(sth).is_some_and(|(price, sth)| {
        is_finite_positive(price) && is_finite_positive(sth) && price >= sth
    })
}

/// Code `0` means the capitalized-price references are not all available yet.
fn classify_phase_code(
    price: Option<Cents>,
    all: Option<Cents>,
    sth: Option<Cents>,
    lth: Option<Cents>,
    sma: Option<Cents>,
) -> StoredU8 {
    let Some((price, all, sth, lth)) = price
        .zip(all)
        .zip(sth)
        .zip(lth)
        .map(|(((price, all), sth), lth)| (price, all, sth, lth))
        .filter(|values| {
            [values.0, values.1, values.2, values.3]
                .into_iter()
                .all(is_finite_positive)
        })
    else {
        return StoredU8::ZERO;
    };

    StoredU8::new(classify_phase(price, all, sth, lth, sma).code())
}

/// Classify investor sentiment from the three capitalized-price references,
/// using the one-year price SMA only as confirmation and disambiguation.
fn classify_phase(
    price: Cents,
    all: Cents,
    sth: Cents,
    lth: Cents,
    sma: Option<Cents>,
) -> CapitalSentimentPhase {
    use CapitalSentimentPhase as Phase;

    let above_all = price >= all;
    let above_sth = price >= sth;
    let above_lth = price >= lth;
    let above_sma = sma.is_some_and(|sma| price >= sma);
    let bull_structure = sth > lth;
    let above_slow_refs = above_all && above_lth;
    let above_any_slow_ref = above_all || above_lth;
    let references_above_price = [all, sth, lth]
        .into_iter()
        .filter(|reference| *reference > price)
        .count()
        + usize::from(sma.is_some_and(|sma| sma > price));
    let price_in_middle = references_above_price == 2;
    let core_bull_phase = if sma.is_some_and(|sma| all > sma) {
        Phase::RagingBull
    } else {
        Phase::Bull
    };
    let core_bear_phase = if lth > all {
        Phase::DeepBear
    } else {
        Phase::Bear
    };

    if sma.is_none() {
        if bull_structure {
            if above_sth {
                return if above_slow_refs {
                    core_bull_phase
                } else {
                    Phase::EarlyBull
                };
            }
            if above_slow_refs {
                return Phase::WeakBull;
            }
            return if above_any_slow_ref {
                Phase::EarlyBear
            } else {
                core_bear_phase
            };
        }

        if !above_sth {
            return core_bear_phase;
        }
        if above_slow_refs {
            return core_bull_phase;
        }
        return if above_any_slow_ref {
            Phase::EarlyBull
        } else {
            Phase::CautiousBull
        };
    }

    if !above_all && !above_sth && !above_lth && !above_sma {
        return core_bear_phase;
    }
    if !above_sth && price_in_middle {
        return Phase::Limbo;
    }
    if bull_structure && (!above_slow_refs || !above_sma) {
        return Phase::EarlyBear;
    }
    if above_sth && above_sma {
        return if above_slow_refs {
            core_bull_phase
        } else {
            Phase::EarlyBull
        };
    }
    if !above_sth && above_slow_refs && above_sma {
        return Phase::WeakBull;
    }
    if above_sth {
        return Phase::CautiousBull;
    }
    if above_sma {
        return Phase::HopefulBull;
    }
    Phase::EarlyBear
}

fn recompute_day(indexer: &Indexer, indexes: &bitview_plugin_indexes::Vecs) -> Option<Day1> {
    let starting_height = indexer.safe_lengths().height;
    indexes
        .height
        .day1
        .collect_one(starting_height)
        .or_else(|| {
            starting_height
                .decremented()
                .and_then(|height| indexes.height.day1.collect_one(height))
        })
}

#[cfg(test)]
mod tests {
    use std::{cmp::Reverse, collections::BTreeSet};

    use super::*;

    fn cents(value: u64) -> Cents {
        Cents::new(value)
    }

    #[test]
    fn samples_each_days_last_available_block() {
        let first_heights = [0_usize, 2, 2, 5].map(Height::from);

        assert_eq!(
            last_height_of_day(&first_heights, 0, 7),
            Some(Height::from(1_usize))
        );
        assert_eq!(last_height_of_day(&first_heights, 1, 7), None);
        assert_eq!(
            last_height_of_day(&first_heights, 2, 7),
            Some(Height::from(4_usize))
        );
        assert_eq!(
            last_height_of_day(&first_heights, 3, 7),
            Some(Height::from(6_usize))
        );
    }

    #[test]
    fn sampling_clamps_the_current_day_to_the_shared_source_length() {
        let first_heights = [0_usize, 2, 5].map(Height::from);

        assert_eq!(
            last_height_of_day(&first_heights, 1, 4),
            Some(Height::from(3_usize))
        );
        assert_eq!(last_height_of_day(&first_heights, 2, 4), None);
    }

    fn classify(price: u64, all: u64, sth: u64, lth: u64, sma: u64) -> CapitalSentimentPhase {
        classify_phase(
            cents(price),
            cents(all),
            cents(sth),
            cents(lth),
            Some(cents(sma)),
        )
    }

    #[test]
    fn classifies_all_ten_phases_across_all_eight_reference_orders() {
        use CapitalSentimentPhase as Phase;

        let cases = [
            ((100, 70, 80, 50, 60), Phase::RagingBull),
            ((100, 70, 80, 60, 90), Phase::Bull),
            ((90, 70, 60, 80, 100), Phase::CautiousBull),
            ((40, 80, 60, 100, 20), Phase::HopefulBull),
            ((90, 70, 60, 100, 80), Phase::EarlyBull),
            ((90, 70, 100, 60, 80), Phase::WeakBull),
            ((70, 80, 100, 60, 40), Phase::Limbo),
            ((40, 80, 60, 100, 70), Phase::DeepBear),
            ((40, 80, 100, 60, 50), Phase::Bear),
            ((60, 80, 100, 50, 70), Phase::EarlyBear),
        ];

        let mut reference_orders = BTreeSet::new();

        for ((price, all, sth, lth, sma), expected) in cases {
            assert!(
                (sth > all && all > lth) || (lth > all && all > sth),
                "All capitalized price must be between STH and LTH"
            );

            let mut references = [("SMA", sma), ("STH", sth), ("All", all), ("LTH", lth)];
            references.sort_unstable_by_key(|(_, value)| Reverse(*value));
            reference_orders.insert(references.map(|(name, _)| name));

            assert_eq!(classify(price, all, sth, lth, sma), expected);
        }

        assert_eq!(reference_orders.len(), 8);
    }

    #[test]
    fn sma_confirms_the_capitalized_price_structure() {
        use CapitalSentimentPhase as Phase;

        assert_eq!(classify(100, 70, 80, 60, 90), Phase::Bull);
        assert_eq!(classify(100, 70, 80, 60, 50), Phase::RagingBull);
    }

    #[test]
    fn equal_sth_and_lth_is_not_a_bull_structure() {
        assert_eq!(
            classify(70, 50, 50, 50, 100),
            CapitalSentimentPhase::CautiousBull
        );
    }

    #[test]
    fn missing_reference_has_no_phase() {
        assert_eq!(
            classify_phase_code(
                Some(cents(100)),
                Some(cents(70)),
                Some(cents(80)),
                None,
                Some(cents(50)),
            ),
            StoredU8::ZERO
        );
    }

    #[test]
    fn phase_is_available_without_sma() {
        assert_eq!(
            classify_phase(cents(100), cents(70), cents(80), cents(60), None),
            CapitalSentimentPhase::Bull
        );
    }

    #[test]
    fn signal_enters_only_on_an_sth_cross_and_exits_on_a_sell_phase() {
        use CapitalSentimentPhase as Phase;

        let bull = StoredU8::new(Phase::Bull.code());
        let bear = StoredU8::new(Phase::Bear.code());

        assert!(!next_is_long(false, None, true, bull));
        assert!(next_is_long(false, Some(false), true, bull));
        assert!(!next_is_long(false, Some(true), true, bull));
        assert!(next_is_long(true, Some(true), true, bull));
        assert!(!next_is_long(true, Some(true), true, bear));
    }
}
