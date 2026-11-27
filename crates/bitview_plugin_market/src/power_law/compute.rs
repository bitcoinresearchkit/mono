use bitview_plugin_indexer::Indexer;
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_plugin_price::Vecs as PriceVecs;
use bitview_primitives::Day1;
use brk_error::Result;
use brk_exit::Exit;
use brk_types::Cents;
use vecdb::{AnyVec, ComputeVec, ReadableVec, VecIndex};

use super::{Vecs, fit::Points, published::days_since_genesis};

/// Sminston With's quantile: the line under 5% of the closes.
const QUANTILE: f64 = 0.05;
/// The closes the floor waits for: a year's. On fewer, the fitted slope swings by over ten either way.
const LEAST: usize = 365;

impl Vecs {
    pub(crate) fn compute(
        &mut self,
        indexer: &Indexer,
        prices: &PriceVecs,
        mappings: &MappingsVecs,
        exit: &Exit,
    ) -> Result<()> {
        let Self { sminston_floor, sminston_state, .. } = self;
        let spot = &prices.spot.cents;
        let days = &mappings.height.day1;
        let version = spot.height.version() + days.version();
        // Continued from the last update when it ended where this one starts; otherwise (restart,
        // reorg) rebuilt from the stored closes of the days before, to the same bits.
        let mut floor: Option<Floor> = None;
        sminston_floor.cents.height.compute_transform2(
            indexer.compute_start().height,
            &spot.height,
            days,
            |(height, price, day, _)| {
                let floor = floor.get_or_insert_with(|| {
                    sminston_state
                        .take(height.to_usize(), version)
                        .unwrap_or_else(|| Floor::before(day, &spot.resolutions.day1))
                });
                floor.next = height.to_usize() + 1;
                floor.at(day, price)
            },
            exit,
        )?;
        if let Some(floor) = floor {
            sminston_state.keep(floor.next, version, floor);
        }
        Ok(())
    }
}

/// Sminston With's floor through the blocks so far: the closes of the days before the current one,
/// and their quantile line.
pub(super) struct Floor {
    points: Points,
    /// The block after the last one seen.
    next: usize,
    day: Day1,
    /// The last block's price: its day's close once the day ends.
    last: Cents,
    line: Option<(f64, f64)>,
}

impl Floor {
    /// The floor for blocks of `day`, from the closes of every day before it.
    fn before(day: Day1, closes: &impl ReadableVec<Day1, Option<Cents>>) -> Self {
        let mut floor = Self { points: Points::new(QUANTILE), next: 0, day, last: Cents::NAN, line: None };
        for (at, close) in closes.collect_range(Day1::from(0_usize), day).into_iter().enumerate() {
            if let Some(close) = close {
                floor.add(Day1::from(at), close);
            }
        }
        floor.refit();
        floor
    }

    /// A day's close as a point: log10 cents on log10 days since genesis. Unpriced days have none.
    fn add(&mut self, day: Day1, close: Cents) {
        let days = days_since_genesis(day);
        if days > 0.0 && !close.is_nan() && close > Cents::ZERO {
            self.points.push(days.log10(), f64::from(close).log10());
        }
    }

    fn refit(&mut self) {
        self.line = (self.points.len() >= LEAST).then(|| self.points.fit());
    }

    /// The floor at a block of `day` priced `price`. A block of a new day first adds the day
    /// before's close and refits: a block's value uses only closes of days before its own.
    fn at(&mut self, day: Day1, price: Cents) -> Cents {
        if day != self.day {
            self.add(self.day, self.last);
            self.refit();
            self.day = day;
        }
        self.last = price;
        self.line.map_or(Cents::NAN, |(intercept, slope)| {
            Cents::from(10f64.powf(intercept + slope * days_since_genesis(day).log10()))
        })
    }
}
