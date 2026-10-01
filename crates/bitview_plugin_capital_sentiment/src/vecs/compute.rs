use bitview_compute::prepare_computed;
use bitview_plugin::{ComputePlugin, UpdateContext};
use bitview_vecs::CachedSeries;
use brk_error::Result;
use brk_exit::Exit;
use brk_types::{CapitalSentimentPhase as Phase, Cents, Height, StoredBool, StoredU8, Version};
use vecdb::{AnyStoredVec, Cursor, ReadableVec, WritableVec};

use super::Vecs;
use crate::Dependencies;

const WRITE_INTERVAL_BLOCKS: usize = 1_000;

impl ComputePlugin for Vecs {
    type Dependencies<'a> = Dependencies<'a>;

    fn compute(
        &mut self,
        dependencies: Self::Dependencies<'_>,
        context: UpdateContext<'_>,
    ) -> Result<()> {
        let Dependencies {
            indexer,
            price: prices,
            distribution_aggregated,
            moving_average,
        } = dependencies;
        let exit = context.exit();

        self.db.sync_bg_tasks()?;

        let spot = &prices.spot.cents.height;
        let sma = &moving_average.sma._1y.cents.height;
        let all = &distribution_aggregated
            .cohorts
            .all
            .realized
            .capitalized_price
            .cents
            .height;
        let sth = &distribution_aggregated
            .cohorts
            .sth
            .realized
            .capitalized_price
            .cents
            .height;
        let lth = &distribution_aggregated
            .cohorts
            .lth
            .realized
            .capitalized_price
            .cents
            .height;

        compute_series(
            &mut self.phase_code.height,
            &mut self.is_long.height,
            [spot, all, sth, lth, sma],
            usize::from(indexer.safe_lengths().height),
            exit,
        )?;

        context.compact_database(&self.db);

        Ok(())
    }
}

fn compute_series(
    phase: &mut CachedSeries<Height, StoredU8>,
    position: &mut CachedSeries<Height, StoredBool>,
    sources: [&dyn ReadableVec<Height, Cents>; 5],
    recompute_from: usize,
    exit: &Exit,
) -> Result<()> {
    let source_version: Version = sources.iter().map(|source| source.version()).sum();
    let source_end = sources
        .iter()
        .map(|source| source.len())
        .min()
        .unwrap_or_default();
    let start = prepare_computed(
        [&mut *phase as &mut dyn AnyStoredVec, &mut *position],
        source_version,
        recompute_from.min(source_end),
        exit,
    )?;

    let mut is_long = start
        .checked_sub(1)
        .map(Height::from)
        .and_then(|height| position.collect_one(height))
        .is_some_and(|value| value.is_true());
    let mut sources = sources.map(Cursor::new);
    let mut previous_over_sth = start
        .checked_sub(1)
        .map(|index| is_over_sth(sources[0].get(index), sources[2].get(index)));
    for block_index in start..source_end {
        let [price, all, sth, lth, sma] = sources.each_mut().map(|source| source.get(block_index));
        let over_sth = is_over_sth(price, sth);
        let code = classify_phase_code(price, all, sth, lth, sma);
        is_long = next_is_long(is_long, previous_over_sth, over_sth, code);

        phase.push(code);
        position.push(StoredBool::from(is_long));
        previous_over_sth = Some(over_sth);

        if (block_index + 1).is_multiple_of(WRITE_INTERVAL_BLOCKS) || block_index + 1 == source_end
        {
            let _lock = exit.lock();
            phase.write()?;
            position.write()?;
        }
    }

    Ok(())
}

#[cfg(test)]
#[path = "recovery_tests.rs"]
mod recovery_tests;

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
    if is_long && Phase::from_code(*phase_code).is_some_and(|phase| phase.is_sell()) {
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
    let (Some(price), Some(all), Some(sth), Some(lth)) = (price, all, sth, lth) else {
        return StoredU8::ZERO;
    };
    if ![price, all, sth, lth].into_iter().all(is_finite_positive) {
        return StoredU8::ZERO;
    }

    StoredU8::new(classify_phase(price, all, sth, lth, sma).code())
}

/// Classify investor sentiment from the three capitalized-price references,
/// using the one-year price SMA only as confirmation and disambiguation.
fn classify_phase(price: Cents, all: Cents, sth: Cents, lth: Cents, sma: Option<Cents>) -> Phase {
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

#[cfg(test)]
mod tests {
    use std::{cmp::Reverse, collections::BTreeSet};

    use super::*;

    fn cents(value: u64) -> Cents {
        Cents::new(value)
    }

    fn classify(price: u64, all: u64, sth: u64, lth: u64, sma: u64) -> Phase {
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
        assert_eq!(classify(100, 70, 80, 60, 90), Phase::Bull);
        assert_eq!(classify(100, 70, 80, 60, 50), Phase::RagingBull);
    }

    #[test]
    fn equal_sth_and_lth_is_not_a_bull_structure() {
        assert_eq!(classify(70, 50, 50, 50, 100), Phase::CautiousBull);
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
    fn every_capitalized_price_reference_must_be_present_and_positive() {
        let valid = [
            Some(cents(100)),
            Some(cents(70)),
            Some(cents(80)),
            Some(cents(60)),
        ];
        for index in 0..valid.len() {
            for invalid in [None, Some(Cents::ZERO), Some(Cents::NAN)] {
                let mut references = valid;
                references[index] = invalid;
                let [price, all, sth, lth] = references;
                assert_eq!(
                    classify_phase_code(price, all, sth, lth, Some(cents(50))),
                    StoredU8::ZERO,
                );
            }
        }
    }

    #[test]
    fn phase_is_available_without_sma() {
        assert_eq!(
            classify_phase(cents(100), cents(70), cents(80), cents(60), None),
            Phase::Bull
        );
    }

    #[test]
    fn signal_enters_only_on_an_sth_cross_and_exits_on_a_sell_phase() {
        let bull = StoredU8::new(Phase::Bull.code());
        let bear = StoredU8::new(Phase::Bear.code());

        assert!(!next_is_long(false, None, true, bull));
        assert!(next_is_long(false, Some(false), true, bull));
        assert!(!next_is_long(false, Some(true), true, bull));
        assert!(next_is_long(true, Some(true), true, bull));
        assert!(!next_is_long(true, Some(true), true, bear));
    }
}
