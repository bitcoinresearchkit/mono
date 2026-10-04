use bitview_compute::prepare_computed;
use bitview_plugin::{ComputePlugin, UpdateContext};
use bitview_primitives::{Boolean, CapitalSentimentPhase as Phase, StoredU8};
use bitview_vecs::CachedSeries;
use brk_error::Result;
use brk_exit::Exit;
use brk_types::{Cents, Height, Version};
use vecdb::{AnyStoredVec, Cursor, Database, ReadableVec, WritableVec};

use crate::{Dependencies, Vecs};

const WRITE_INTERVAL_BLOCKS: usize = 1_000;

impl ComputePlugin for Vecs {
    type Dependencies<'a> = Dependencies<'a>;

    fn database(&self) -> Option<&Database> {
        Some(&self.db)
    }

    fn compute_state(
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

        Ok(())
    }
}

fn compute_series(
    phase: &mut CachedSeries<Height, StoredU8>,
    position: &mut CachedSeries<Height, Boolean>,
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
        position.push(Boolean::from(is_long));
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
