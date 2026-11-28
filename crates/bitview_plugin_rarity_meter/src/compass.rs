use bitview_compute::{FenwickTree, Retained, prepare_days, write_locked};
use bitview_plugin_coinflow::Vecs as CoinflowVecs;
use bitview_plugin_cointime::Vecs as CointimeVecs;
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_plugin_price::Vecs as PriceVecs;
use bitview_primitives::{Day1, Days, Flag, Float, Mood};
use bitview_traversable::Traversable;
use bitview_vecs::CostBasisByDollar;
use brk_error::Result;
use brk_exit::Exit;
use brk_types::{Dollars, Height, Version};
use vecdb::{
    AnyStoredVec, AnyVec, Database, ImportableVec, PcoVec, ReadableBoxedVec, ReadableCloneableVec,
    ReadableVec, Rw, StorageMode, WritableVec,
};

/// Days before the index and the extremes start: with fewer behind it, a place outside a cloud
/// would be ranked against next to nothing.
const INDEXED_AFTER: usize = 365;
/// A halving cycle, in days: a day's weight doubles each, so recent cycles count more. The page's
/// run shares weigh days the same way (website/compass/market.js, `weight`).
const HALF_LIFE: f64 = 1461.0;
/// Log distances outside a cloud, ranked by bucket.
const BUCKETS: usize = 4096;
const WIDTH: f64 = 0.002;
/// An extreme day's place has under this share of the earlier places' weight below it, or above it.
const EXTREME: f64 = 0.01;

fn weight(day: usize) -> f64 {
    2f64.powf(day as f64 / HALF_LIFE)
}

/// Where the price sits against what young and old coins cost (bitview.space/compass): two clouds,
/// each spanning its three holder groups' 40th and 60th percentiles and average cost basis by
/// dollar, and the price above, inside or below each. Read at each day's close from that day and
/// the days before it alone; the current day's moves with its blocks.
#[derive(Traversable)]
pub struct Compass<M: StorageMode = Rw> {
    /// Each coin counted by how likely it is ever to be spent, so lost coins weigh little.
    #[traversable(
        lead = "Compass, mobile coins: each coin counted by how likely it is ever to be spent."
    )]
    mobile: CompassSeries<M>,
    /// Each coin counted by how much of its holding time has been spent.
    #[traversable(
        lead = "Compass, awake coins: each coin counted by how much of its holding time has been spent."
    )]
    awake: CompassSeries<M>,
    #[traversable(skip)]
    close: ReadableBoxedVec<Day1, Option<Dollars>>,
    #[traversable(skip)]
    day_of_height: ReadableBoxedVec<Height, Day1>,
}

/// One weighting's Compass.
#[derive(Traversable)]
pub struct CompassSeries<M: StorageMode = Rw> {
    /// Where the day sits on the compass, from its two cloud positions: 0 is its Fear corner, 100
    /// its Optimism corner. Null for the first year, ranked against too few days.
    #[traversable(
        lead = "Compass index: 0 is the compass's Fear corner, 100 its Optimism corner."
    )]
    index: M::Stored<PcoVec<Day1, Float>>,
    /// The price against the short-term cloud (coins held under 4, 5 and 6 months), -1 to 1:
    /// inside, its log place between the cloud's edges, the middle third; outside, the weighted
    /// rank of its log distance among the earlier days on that side, a day counting half as much
    /// every four years back, the outer thirds.
    #[traversable(
        lead = "Position against the short-term cloud, what young coins cost: -1 far below, 1 far above."
    )]
    short_term_cloud_position: M::Stored<PcoVec<Day1, Float>>,
    /// The price against the long-term cloud (coins held over 4, 5 and 6 months), as the
    /// short-term position.
    #[traversable(
        lead = "Position against the long-term cloud, what old coins cost: -1 far below, 1 far above."
    )]
    long_term_cloud_position: M::Stored<PcoVec<Day1, Float>>,
    /// The day's mood, the price above, inside or below each cloud: 0 Optimism, 1 Calm, 2 Anxiety
    /// (above the long-term cloud; above, inside, below the short-term one), 3 Hope, 4 Doubt,
    /// 5 Worry (inside the long-term one), 6 Relief, 7 Apathy, 8 Fear (below it). A day missing
    /// its inputs keeps the day before's.
    #[traversable(
        lead = "Compass mood: 0 Optimism to 8 Fear, by the price above, inside or below each cloud."
    )]
    mood: M::Stored<PcoVec<Day1, Mood>>,
    /// 1 when the day's position against either cloud is in the outer 1% of the earlier days'
    /// positions, either way: those below it, or those above it, hold under 1% of all their
    /// weight, days weighted as the positions weigh them. Null for the first year.
    #[traversable(
        lead = "Compass extreme: 1 when under 1% of the earlier days, by weight, sat lower than the day, or higher, against either cloud."
    )]
    extreme: M::Stored<PcoVec<Day1, Flag>>,
    /// Days into the mood's current run so far, the day itself included; never the run's final
    /// length.
    #[traversable(lead = "Days in the mood so far: the current run's days up to the day.")]
    mood_days: M::Stored<PcoVec<Day1, Days>>,
    /// The short-term cloud's nine lines, then the long-term cloud's.
    #[traversable(skip)]
    lines: [[ReadableBoxedVec<Day1, Option<Dollars>>; 9]; 2],
    /// The closed days folded so far, kept between updates.
    folded: M::WriteOnly<Retained<Folded>>,
}

impl Compass {
    pub(crate) fn import(
        db: &Database,
        version: Version,
        mappings: &MappingsVecs,
        cointime: &CointimeVecs,
        coinflow: &CoinflowVecs,
        price: &PriceVecs,
    ) -> Result<Self> {
        let (mobile, awake) = (&coinflow.aggregate.cohorts, &cointime.aggregate.cohorts);
        let mobile = [
            [&mobile.under_4m, &mobile.sth, &mobile.under_6m]
                .map(|c| &c.mobile.cost_basis.by_dollar),
            [&mobile.over_4m, &mobile.lth, &mobile.over_6m].map(|c| &c.mobile.cost_basis.by_dollar),
        ];
        let awake = [
            [&awake.under_4m, &awake.sth, &awake.under_6m].map(|c| &c.awake.cost_basis.by_dollar),
            [&awake.over_4m, &awake.lth, &awake.over_6m].map(|c| &c.awake.cost_basis.by_dollar),
        ];
        Ok(Self {
            mobile: CompassSeries::import(db, "mobile", version, mobile)?,
            awake: CompassSeries::import(db, "awake", version, awake)?,
            close: price.spot.usd.day1.read_only_boxed_clone(),
            day_of_height: mappings.height.day1.read_only_boxed_clone(),
        })
    }

    /// Writes the days from the one before block `from`'s on.
    pub(crate) fn compute(&mut self, from: Height, exit: &Exit) -> Result<()> {
        let version = Version::ONE + self.close.version() + self.day_of_height.version();
        for series in [&mut self.mobile, &mut self.awake] {
            series.compute(from, &self.day_of_height, &self.close, version, exit)?;
        }
        Ok(())
    }
}

impl CompassSeries {
    fn import(
        db: &Database,
        weighting: &str,
        version: Version,
        clouds: [[&CostBasisByDollar; 3]; 2],
    ) -> Result<Self> {
        let name = |metric: &str| format!("{weighting}_compass_{metric}");
        let lines = clouds.map(|groups| {
            let mut lines = groups.iter().flat_map(|by_dollar| {
                [
                    by_dollar.percentiles.p40.usd.day1.read_only_boxed_clone(),
                    by_dollar.percentiles.p60.usd.day1.read_only_boxed_clone(),
                    by_dollar.avg.usd.day1.read_only_boxed_clone(),
                ]
            });
            std::array::from_fn(|_| lines.next().unwrap())
        });
        Ok(Self {
            index: PcoVec::import(db, &name("index"), version)?,
            short_term_cloud_position: PcoVec::import(
                db,
                &name("short_term_cloud_position"),
                version,
            )?,
            long_term_cloud_position: PcoVec::import(
                db,
                &name("long_term_cloud_position"),
                version,
            )?,
            mood: PcoVec::import(db, &name("mood"), version)?,
            extreme: PcoVec::import(db, &name("extreme"), version)?,
            mood_days: PcoVec::import(db, &name("mood_days"), version)?,
            lines,
            folded: Default::default(),
        })
    }

    fn compute(
        &mut self,
        from: Height,
        day_of_height: &ReadableBoxedVec<Height, Day1>,
        close: &ReadableBoxedVec<Day1, Option<Dollars>>,
        version: Version,
        exit: &Exit,
    ) -> Result<()> {
        let end = self
            .lines
            .as_flattened()
            .iter()
            .map(AnyVec::len)
            .fold(close.len(), usize::min);
        if end == 0 {
            return Ok(());
        }
        let version = version
            + self
                .lines
                .as_flattened()
                .iter()
                .map(AnyVec::version)
                .sum::<Version>();
        let start_day = prepare_days(self.outputs(), version, day_of_height, from, end, exit)?;
        // Continued from the open day it stopped at, else rebuilt from the first day (restart, a
        // rewrite into an earlier day, new version).
        let (first, mut folded) = self.folded.resume(start_day, version, Folded::new);
        let closes = close.collect_range_at(first, end);
        let lines = self
            .lines
            .as_flattened()
            .iter()
            .map(|line| line.collect_range_at(first, end))
            .collect::<Vec<_>>();
        for (at, day) in (first..end).enumerate() {
            let inputs = Inputs::of(closes[at], |k| lines[k][at]);
            let today = folded.read(day, inputs);
            // A day's extreme scans every earlier day: only for the days written.
            if day >= start_day {
                self.push(today.as_ref().map(|today| (today, folded.extreme(today))));
            }
            // The last day may have more blocks to come: written, not folded.
            if day + 1 < end
                && let Some(today) = &today
            {
                folded.fold(today);
            }
        }
        write_locked(self.outputs(), exit)?;
        self.folded.keep(end - 1, version, folded);
        Ok(())
    }

    fn push(&mut self, day: Option<(&Day, bool)>) {
        let Some((day, extreme)) = day else {
            self.index.push(Float::from(f64::NAN));
            self.short_term_cloud_position.push(Float::from(f64::NAN));
            self.long_term_cloud_position.push(Float::from(f64::NAN));
            self.mood.push(Mood::NAN);
            self.extreme.push(Flag::NAN);
            self.mood_days.push(Days::from(f64::NAN));
            return;
        };
        let [x, y] = day.places.map(|place| place.value);
        let index = if day.indexed {
            25.0 * (f64::from(x) + f64::from(y) + 2.0)
        } else {
            f64::NAN
        };
        self.index.push(Float::from(index));
        self.short_term_cloud_position
            .push(Float::from(f64::from(x)));
        self.long_term_cloud_position
            .push(Float::from(f64::from(y)));
        self.mood.push(Mood::new(day.mood));
        self.extreme.push(if day.indexed {
            Flag::new(i8::from(extreme))
        } else {
            Flag::NAN
        });
        self.mood_days
            .push(Days::from((day.day + 1 - day.run_start) as f64));
    }

    fn outputs(&mut self) -> [&mut dyn AnyStoredVec; 6] {
        [
            &mut self.index,
            &mut self.short_term_cloud_position,
            &mut self.long_term_cloud_position,
            &mut self.mood,
            &mut self.extreme,
            &mut self.mood_days,
        ]
    }
}

/// A day's close and its clouds' edges, bottom then top: the short-term cloud, then the long-term.
#[derive(Clone, Copy)]
struct Inputs {
    close: f64,
    clouds: [[f64; 2]; 2],
}

impl Inputs {
    /// `None` without a price or with a line missing: no mood that day.
    fn of(close: Option<Dollars>, line: impl Fn(usize) -> Option<Dollars>) -> Option<Self> {
        let close = Some(f64::from(close?)).filter(|&close| close > 0.0)?;
        let cloud = |c: usize| {
            (0..9).try_fold([f64::INFINITY, f64::NEG_INFINITY], |[low, high], k| {
                let value = f64::from(line(9 * c + k)?);
                (value > 0.0).then_some([low.min(value), high.max(value)])
            })
        };
        Some(Self {
            close,
            clouds: [cloud(0)?, cloud(1)?],
        })
    }

    /// 1 above the cloud, 0 inside, -1 below.
    fn side(&self, cloud: usize) -> i8 {
        let [low, high] = self.clouds[cloud];
        if self.close > high {
            1
        } else if self.close < low {
            -1
        } else {
            0
        }
    }
}

/// A day's place against one cloud, and where an outside one is ranked: its side (0 above, 1
/// below) and bucket.
#[derive(Clone, Copy)]
struct Place {
    value: f32,
    outside: Option<(usize, usize)>,
}

/// A day as read: its inputs (the day before's when it has none), mood, places and run.
struct Day {
    day: usize,
    inputs: Inputs,
    mood: i8,
    places: [Place; 2],
    indexed: bool,
    run_start: usize,
}

/// What the Compass keeps of the closed days before the next one it reads.
#[derive(Clone)]
struct Folded {
    /// The first day with a mood.
    first: Option<usize>,
    last: Option<Inputs>,
    /// Each cloud's outside distances by bucket, above and below, and their total weights.
    ranks: [[FenwickTree<f64>; 2]; 2],
    totals: [[f64; 2]; 2],
    /// Each cloud's earlier places and their weights, for the extremes.
    places: [Vec<(f32, f64)>; 2],
    mass: f64,
    /// The last mood and the first day of its run.
    run: (i8, usize),
}

impl Folded {
    fn new() -> Self {
        Self {
            first: None,
            last: None,
            ranks: std::array::from_fn(|_| std::array::from_fn(|_| FenwickTree::new(BUCKETS))),
            totals: [[0.0; 2]; 2],
            places: Default::default(),
            mass: 0.0,
            run: (0, 0),
        }
    }

    /// `day` against the days folded before it; `None` before the first day with a mood.
    fn read(&self, day: usize, inputs: Option<Inputs>) -> Option<Day> {
        let inputs = inputs.or(self.last)?;
        let first = self.first.unwrap_or(day);
        let mood = (1 - inputs.side(1)) * 3 + (1 - inputs.side(0));
        let places = [0, 1].map(|cloud| self.place(&inputs, cloud));
        let indexed = day >= first + INDEXED_AFTER;
        let run_start = if self.first.is_some() && self.run.0 == mood {
            self.run.1
        } else {
            day
        };
        Some(Day {
            day,
            inputs,
            mood,
            places,
            indexed,
            run_start,
        })
    }

    /// Whether either of `today`'s places is in the outer share of the earlier days' places.
    fn extreme(&self, today: &Day) -> bool {
        today.indexed
            && today
                .places
                .iter()
                .zip(&self.places)
                .any(|(place, earlier)| {
                    let (below, above) =
                        earlier.iter().fold((0.0, 0.0), |(below, above), &(v, w)| {
                            (
                                below + if v < place.value { w } else { 0.0 },
                                above + if v > place.value { w } else { 0.0 },
                            )
                        });
                    below / self.mass < EXTREME || above / self.mass < EXTREME
                })
    }

    /// Inside a cloud the middle third by log place between its edges; outside, the outer third
    /// by weighted rank among the earlier days on that side.
    fn place(&self, inputs: &Inputs, cloud: usize) -> Place {
        let (p, [low, high]) = (inputs.close, inputs.clouds[cloud]);
        if p >= low && p <= high {
            let value = if high > low {
                -1.0 / 3.0 + ((2.0 / 3.0) * (p / low).ln()) / (high / low).ln()
            } else {
                0.0
            };
            return Place {
                value: value as f32,
                outside: None,
            };
        }
        let side = usize::from(p < low);
        let distance = if side == 1 { low / p } else { p / high };
        let bucket = ((distance.ln() / WIDTH).floor() as usize).min(BUCKETS - 1);
        let total = self.totals[cloud][side];
        let rank = if total > 0.0 {
            self.ranks[cloud][side].prefix_sum(bucket) / total
        } else {
            0.5
        };
        let sign = if side == 1 { -1.0 } else { 1.0 };
        Place {
            value: (sign * (1.0 / 3.0 + (2.0 / 3.0) * rank)) as f32,
            outside: Some((side, bucket)),
        }
    }

    fn fold(&mut self, today: &Day) {
        let w = weight(today.day);
        self.first.get_or_insert(today.day);
        self.last = Some(today.inputs);
        for (cloud, place) in today.places.iter().enumerate() {
            if let Some((side, bucket)) = place.outside {
                self.ranks[cloud][side].add(bucket, &w);
                self.totals[cloud][side] += w;
            }
            self.places[cloud].push((place.value, w));
        }
        self.mass += w;
        self.run = (today.mood, today.run_start);
    }
}
