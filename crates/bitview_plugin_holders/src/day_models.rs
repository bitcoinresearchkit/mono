use std::collections::VecDeque;

use bitview_compute::{Retained, day_closes, prepare_days, write_locked};
use bitview_primitives::{Day1, Float};
use bitview_traversable::Traversable;
use brk_error::{OptionData, Result};
use brk_exit::Exit;
use brk_types::{Cents, Height, Sats, Version};
use vecdb::{
    AnyStoredVec, AnyVec, Database, ImportableVec, PcoVec, ReadableBoxedVec, ReadableVec, Rw,
    StorageMode, VecIndex, WritableVec,
};

/// Seller exhaustion's volatility window, in daily returns.
const RETURNS: usize = 30;

/// Two published models read at each day's close (the day's last block, UTC), from the whole
/// market's coins and the price. Each day's value uses only the days up to it, and is written
/// once its day ends; the current day's moves with its blocks, as every day value does.
#[derive(Traversable)]
pub struct DayModels<M: StorageMode = Rw> {
    /// MVRV Z-score (Awe & Wonder, 2018, on the MVRV of Mahmudov and Puell, 2018): market cap
    /// minus realized cap, divided by the (population) standard deviation of the daily market
    /// cap. The deviation runs from the first priced day (block 74,641) through the represented
    /// day, never over all history, so early values differ from sites that recompute it over
    /// every day.
    /// Null before two priced days.
    #[traversable(
        lead = "MVRV Z-score: market cap less realized cap, in standard deviations of the market cap so far."
    )]
    mvrv_z: M::Stored<PcoVec<Day1, Float>>,
    /// Seller exhaustion constant (ARK Invest, 2021): the share of supply in profit (as a
    /// fraction) times the price's 30-day volatility, the sample standard deviation of the
    /// day's and the 29 days before's daily log returns (drift removed) scaled by √30. Null
    /// until 31 consecutive priced daily closes.
    #[traversable(
        lead = "Seller exhaustion: the share of coins in profit times the price's 30-day volatility; low marks capitulation."
    )]
    seller_exhaustion: M::Stored<PcoVec<Day1, Float>>,
    #[traversable(skip)]
    day_of_height: ReadableBoxedVec<Height, Day1>,
    #[traversable(skip)]
    first_height: ReadableBoxedVec<Day1, Height>,
    /// The closed days folded so far, kept between updates.
    folded: M::WriteOnly<Retained<Folded>>,
}

/// What the models keep of the closed days.
#[derive(Clone, Default)]
struct Folded {
    caps: f64,
    squares: f64,
    priced_days: u32,
    /// The last closes, oldest first, by day.
    closes: VecDeque<(usize, f64)>,
}

/// A day's close: the values at its last block.
struct Close {
    day: usize,
    market_cap: f64,
    realized_cap: f64,
    price: f64,
    profit_share: f64,
}

impl Folded {
    fn fold(&mut self, close: &Close) {
        if close.market_cap > 0.0 {
            self.caps += close.market_cap;
            self.squares += close.market_cap * close.market_cap;
            self.priced_days += 1;
        }
        if close.price > 0.0 {
            self.closes.push_back((close.day, close.price));
            if self.closes.len() > RETURNS + 1 {
                self.closes.pop_front();
            }
        }
    }

    /// Both models at `close`'s day, with the days folded before it.
    fn models(&self, close: &Close) -> (Float, Float) {
        let mut through = self.clone();
        through.fold(close);
        let count = f64::from(through.priced_days);
        let mean = through.caps / count;
        let deviation = (through.squares / count - mean * mean).max(0.0).sqrt();
        let mvrv_z = if through.priced_days >= 2 && close.market_cap > 0.0 {
            (close.market_cap - close.realized_cap) / deviation
        } else {
            f64::NAN
        };
        (
            Float::from(mvrv_z),
            Float::from(close.profit_share * through.volatility(close.day)),
        )
    }

    /// The 30 daily log returns ending at `day`, each from the day before: their sample deviation
    /// with the drift removed, over 30 days.
    fn volatility(&self, day: usize) -> f64 {
        let closes = &self.closes;
        let consecutive = closes.len() == RETURNS + 1
            && closes.back().is_some_and(|&(last, _)| last == day)
            && closes[0].0 + RETURNS == day;
        if !consecutive {
            return f64::NAN;
        }
        let (mut sum, mut squares) = (0.0, 0.0);
        for pair in closes.iter().zip(closes.iter().skip(1)) {
            let ((_, before), (_, after)) = pair;
            let r = (after / before).ln();
            sum += r;
            squares += r * r;
        }
        let n = RETURNS as f64;
        ((squares - sum * sum / n) / (n - 1.0)).max(0.0).sqrt() * n.sqrt()
    }
}

/// The whole market's per-block inputs.
pub(crate) struct Inputs<'a, S> {
    pub supply: &'a ReadableBoxedVec<Height, Sats>,
    pub supply_profit: &'a ReadableBoxedVec<Height, Sats>,
    pub realized_cap: &'a ReadableBoxedVec<Height, Cents>,
    pub spot: &'a S,
}

impl DayModels {
    pub(crate) fn import(
        db: &Database,
        version: Version,
        day_of_height: ReadableBoxedVec<Height, Day1>,
        first_height: ReadableBoxedVec<Day1, Height>,
    ) -> Result<Self> {
        Ok(Self {
            mvrv_z: PcoVec::import(db, "mvrv_z", version)?,
            seller_exhaustion: PcoVec::import(db, "seller_exhaustion", version)?,
            day_of_height,
            first_height,
            folded: Default::default(),
        })
    }

    /// Writes the days from the one before block `from`'s on.
    pub(crate) fn compute(
        &mut self,
        from: Height,
        inputs: &Inputs<impl ReadableVec<Height, Cents>>,
        exit: &Exit,
    ) -> Result<()> {
        let end = [
            inputs.supply.len(),
            inputs.supply_profit.len(),
            inputs.realized_cap.len(),
            inputs.spot.len(),
            self.day_of_height.len(),
        ]
        .into_iter()
        .min()
        .unwrap_or(0);
        if end == 0 {
            return Ok(());
        }
        let version = Version::ONE
            + inputs.supply.version()
            + inputs.supply_profit.version()
            + inputs.realized_cap.version()
            + inputs.spot.version()
            + self.day_of_height.version()
            + self.first_height.version();
        let last_day = self.day_of_height.collect_one_at(end - 1).data()?.to_usize();
        let start_day = prepare_days(
            [
                &mut self.mvrv_z as &mut dyn AnyStoredVec,
                &mut self.seller_exhaustion,
            ],
            version,
            &self.day_of_height,
            from,
            last_day + 1,
            exit,
        )?;
        // Only each day's close is read, so a rebuild (restart, a rewrite into an earlier day,
        // new version) is cheap.
        let (first, mut folded) = self.folded.resume(start_day, version, Folded::default);
        let closes = day_closes(&self.first_height, first..last_day + 1, end);
        let heights = closes.iter().flatten().copied().collect::<Vec<_>>();
        let supply = inputs.supply.read_sorted_at(&heights);
        let profit = inputs.supply_profit.read_sorted_at(&heights);
        let realized = inputs.realized_cap.read_sorted_at(&heights);
        let spot = inputs.spot.read_sorted_at(&heights);
        let mut rows = 0..heights.len();
        for (day, close) in (first..).zip(closes) {
            let close = close.map(|_| {
                let at = rows.next().expect("a row per close");
                let supply_btc = f64::from(supply[at]) / 100_000_000.0;
                Close {
                    day,
                    market_cap: supply_btc * f64::from(spot[at]),
                    realized_cap: f64::from(realized[at]),
                    price: f64::from(spot[at]),
                    profit_share: f64::from(profit[at]) / f64::from(supply[at]),
                }
            });
            if day >= start_day {
                // A day without blocks (January 2009) has neither.
                let (mvrv_z, seller_exhaustion) = close.as_ref().map_or_else(
                    || (Float::from(f64::NAN), Float::from(f64::NAN)),
                    |close| folded.models(close),
                );
                self.mvrv_z.push(mvrv_z);
                self.seller_exhaustion.push(seller_exhaustion);
            }
            // The last day may have more blocks to come: written, not folded.
            if day < last_day
                && let Some(close) = &close
            {
                folded.fold(close);
            }
        }
        write_locked(self.outputs(), exit)?;
        self.folded.keep(last_day, version, folded);
        Ok(())
    }

    fn outputs(&mut self) -> [&mut dyn AnyStoredVec; 2] {
        [&mut self.mvrv_z, &mut self.seller_exhaustion]
    }
}
