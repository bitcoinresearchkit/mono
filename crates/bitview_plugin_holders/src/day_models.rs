use std::collections::VecDeque;

use bitview_compute::{prepare_computed, write_chunks, write_locked};
use bitview_primitives::{Day1, Float};
use bitview_traversable::Traversable;
use brk_error::Result;
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
    days: ReadableBoxedVec<Height, Day1>,
    /// The closed days folded so far, kept between updates.
    folded: M::WriteOnly<Option<Folded>>,
}

/// What the models keep of the closed days before `day`, whose first block is `height`.
#[derive(Clone, Default)]
struct Folded {
    version: Version,
    day: usize,
    height: usize,
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
        days: ReadableBoxedVec<Height, Day1>,
    ) -> Result<Self> {
        Ok(Self {
            mvrv_z: PcoVec::import(db, "mvrv_z", version)?,
            seller_exhaustion: PcoVec::import(db, "seller_exhaustion", version)?,
            days,
            folded: Default::default(),
        })
    }

    /// From the day of block `from` on.
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
            self.days.len(),
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
            + self.days.version();
        // From the last day written at the latest: it was written open, and a stop between the
        // inputs crossing its midnight and this update would otherwise leave it at an older tip.
        let from_day = self
            .days
            .collect_one_at(from.to_usize().min(end - 1))
            .unwrap()
            .to_usize()
            .min(self.mvrv_z.len().saturating_sub(1));
        let start_day = prepare_computed(
            [
                &mut self.mvrv_z as &mut dyn AnyStoredVec,
                &mut self.seller_exhaustion,
            ],
            version,
            from_day,
            exit,
        )?;
        // Continued from the open day it stopped in (its blocks walked again), else rebuilt from the
        // first block (restart, a reorg into an earlier day, new version).
        let mut folded = self
            .folded
            .take()
            .filter(|kept| kept.version == version && kept.day <= start_day)
            .unwrap_or(Folded {
                version,
                ..Default::default()
            });
        let mut open: Option<Close> = None;
        for chunk in write_chunks(folded.height..end) {
            let days = self.days.collect_range_at(chunk.start, chunk.end);
            let supply = inputs.supply.collect_range_at(chunk.start, chunk.end);
            let profit = inputs
                .supply_profit
                .collect_range_at(chunk.start, chunk.end);
            let realized = inputs.realized_cap.collect_range_at(chunk.start, chunk.end);
            let spot = inputs.spot.collect_range_at(chunk.start, chunk.end);
            for at in 0..days.len() {
                let day = days[at].to_usize();
                if let Some(closed) = open.take_if(|close| close.day != day) {
                    self.push(&folded, &closed, start_day);
                    folded.fold(&closed);
                    folded.day = day;
                    folded.height = chunk.start + at;
                }
                let supply_btc = f64::from(supply[at]) / 100_000_000.0;
                open = Some(Close {
                    day,
                    market_cap: supply_btc * f64::from(spot[at]),
                    realized_cap: f64::from(realized[at]),
                    price: f64::from(spot[at]),
                    profit_share: f64::from(profit[at]) / f64::from(supply[at]),
                });
            }
            if chunk.end < end {
                write_locked(self.outputs(), exit)?;
            }
        }
        // The last day may have more blocks to come: written, not folded.
        if let Some(last) = open {
            self.push(&folded, &last, start_day);
        }
        write_locked(self.outputs(), exit)?;
        self.folded = Some(folded);
        Ok(())
    }

    fn push(&mut self, folded: &Folded, close: &Close, start_day: usize) {
        if close.day < start_day {
            return;
        }
        let (mvrv_z, seller_exhaustion) = folded.models(close);
        // A day without blocks (January 2009) has neither.
        while self.mvrv_z.len() < close.day {
            self.mvrv_z.push(Float::from(f64::NAN));
            self.seller_exhaustion.push(Float::from(f64::NAN));
        }
        self.mvrv_z.push(mvrv_z);
        self.seller_exhaustion.push(seller_exhaustion);
    }

    fn outputs(&mut self) -> [&mut dyn AnyStoredVec; 2] {
        [&mut self.mvrv_z, &mut self.seller_exhaustion]
    }
}
