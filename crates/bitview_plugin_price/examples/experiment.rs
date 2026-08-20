//! Compare oracle filter/EMA variants against the historical OHLC set.
//!
//! This is a diagnostic harness, not production code. It mirrors the production
//! state machine closely enough to compare candidate changes in one pass over
//! the indexed chain while recording the recent bad-lock heights.
//!
//! Run:
//!   cargo run -p bitview_plugin_price --example experiment --release

use std::{cmp::Ordering, env, path::PathBuf};

use brk_oracle::{
    BINS_PER_DECADE, Config, NUM_BINS, PaymentFilter, START_HEIGHT_FAST, START_HEIGHT_SLOW,
    bin_to_cents, cents_to_bin, seed_bin as oracle_seed_bin,
};
use brk_types::{OutputType, Sats, TxIndex, TxOutIndex};
use vecdb::{AnyVec, ReadableVec, VecIndex};

mod common;

const GENESIS_DAY: u32 = 14252;
const BINS_5PCT: f64 = 4.24;
const BINS_10PCT: f64 = 8.28;
const BINS_20PCT: f64 = 15.84;
const STENCIL_OFFSETS: [i32; 19] = [
    -400, -340, -305, -260, -200, -165, -140, -120, -105, -60, 0, 35, 60, 95, 140, 200, 260, 340,
    400,
];
const N_ARMS: usize = STENCIL_OFFSETS.len();
const TARGET_HEIGHTS: &[usize] = &[952_286, 952_287, 952_288, 952_289, 952_290];

fn bins_to_pct(bins: f64) -> f64 {
    (10.0_f64.powf(bins / BINS_PER_DECADE as f64) - 1.0) * 100.0
}

fn timestamp_to_year(ts: u32) -> u16 {
    let years_since_1970 = ts as f64 / 31_557_600.0;
    (1970.0 + years_since_1970) as u16
}

#[derive(Clone)]
struct ShapeAnchor {
    weight: f64,
    profile: [f64; N_ARMS],
}

impl ShapeAnchor {
    fn new(weight: f64) -> Self {
        Self {
            weight,
            profile: [1.0 / N_ARMS as f64; N_ARMS],
        }
    }

    fn score(&self, state: &OracleState, center: i64) -> f64 {
        if self.weight == 0.0 {
            return 0.0;
        }
        self.weight
            * normalized_arms_at(state, center)
                .map(|arms| {
                    1.0 - (0..N_ARMS)
                        .map(|i| (arms[i] - self.profile[i]).abs())
                        .sum::<f64>()
                })
                .unwrap_or(0.0)
    }

    fn update(&mut self, state: &OracleState, pick: i64) {
        const BETA: f64 = 0.004;
        if self.weight == 0.0 {
            return;
        }
        if let Some(arms) = normalized_arms_at(state, pick) {
            for (p, arm) in self.profile.iter_mut().zip(arms) {
                *p = (1.0 - BETA) * *p + BETA * arm;
            }
        }
    }
}

#[derive(Clone)]
struct OracleState {
    config: Config,
    ring: Vec<Vec<f64>>,
    nonzero: Vec<Vec<usize>>,
    weights: Vec<f64>,
    cursor: usize,
    filled: usize,
    ref_bin: f64,
    warmup: bool,
    shape: ShapeAnchor,
}

impl OracleState {
    fn new(ref_bin: f64, config: Config) -> Self {
        let weights = weights(config.window_size, config.alpha);
        Self {
            ring: vec![vec![0.0; NUM_BINS]; config.window_size],
            nonzero: vec![Vec::new(); config.window_size],
            weights,
            cursor: 0,
            filled: 0,
            ref_bin,
            warmup: false,
            shape: ShapeAnchor::new(config.shape_weight),
            config,
        }
    }

    fn reconfigure(&mut self, config: Config) {
        let kept = self.recent(config.window_size);
        let mut next = Self::new(self.ref_bin, config);
        next.warmup = true;
        for hist in kept {
            next.push_existing(hist);
        }
        next.warmup = false;
        *self = next;
    }

    fn recent(&self, n: usize) -> Vec<Vec<f64>> {
        (0..self.filled.min(n))
            .rev()
            .map(|age| self.ring[self.index_at_age(age)].clone())
            .collect()
    }

    fn index_at_age(&self, age: usize) -> usize {
        (self.cursor + self.ring.len() - 1 - age) % self.ring.len()
    }

    fn start_block(&mut self) {
        for bin in self.nonzero[self.cursor].drain(..) {
            self.ring[self.cursor][bin] = 0.0;
        }
    }

    fn add(&mut self, bin: usize, weight: f64) {
        let slot = &mut self.ring[self.cursor];
        if slot[bin] == 0.0 {
            self.nonzero[self.cursor].push(bin);
        }
        slot[bin] += weight;
    }

    fn push_existing(&mut self, hist: Vec<f64>) {
        self.start_block();
        for (bin, value) in hist
            .into_iter()
            .enumerate()
            .filter(|(_, value)| *value != 0.0)
        {
            self.add(bin, value);
        }
        self.finish_block();
    }

    fn finish_block(&mut self) {
        self.cursor = (self.cursor + 1) % self.ring.len();
        self.filled = (self.filled + 1).min(self.ring.len());
        if self.warmup {
            return;
        }
        self.ref_bin = find_best_bin(self);
        let mut shape = self.shape.clone();
        shape.update(self, self.ref_bin.round() as i64);
        self.shape = shape;
    }

    fn value_at(&self, bin: i64) -> f64 {
        if bin < 0 || bin as usize >= NUM_BINS {
            return 0.0;
        }
        let bin = bin as usize;
        (0..self.filled)
            .map(|age| self.weights[age] * self.ring[self.index_at_age(age)][bin])
            .sum()
    }
}

fn weights(window_size: usize, alpha: f64) -> Vec<f64> {
    let decay = 1.0 - alpha;
    (0..window_size)
        .map(|i| alpha * decay.powi(i as i32))
        .collect()
}

fn normalized_arms_at(state: &OracleState, center: i64) -> Option<[f64; N_ARMS]> {
    let mut arms = STENCIL_OFFSETS.map(|offset| state.value_at(center + offset as i64));
    let sum: f64 = arms.iter().sum();
    if sum <= 0.0 {
        return None;
    }
    for arm in &mut arms {
        *arm /= sum;
    }
    Some(arms)
}

fn find_best_bin(state: &OracleState) -> f64 {
    let center = state.ref_bin.round() as usize;
    let search_start = center.saturating_sub(state.config.search_below);
    let search_end = (center + state.config.search_above + 1).min(NUM_BINS);
    if search_start >= search_end {
        return state.ref_bin;
    }

    let mut arm_peaks = [0.0f64; N_ARMS];
    for (i, &offset) in STENCIL_OFFSETS.iter().enumerate() {
        for bin in search_start..search_end {
            arm_peaks[i] = arm_peaks[i].max(state.value_at(bin as i64 + offset as i64));
        }
    }

    let score = |bin: usize| -> f64 {
        let mut total = 0.0;
        for (i, &offset) in STENCIL_OFFSETS.iter().enumerate() {
            if arm_peaks[i] > 0.0 {
                total += state.value_at(bin as i64 + offset as i64) / arm_peaks[i];
            }
        }
        total + state.shape.score(state, bin as i64)
    };

    let mut best_bin = search_start;
    let mut best_score = score(search_start);
    for bin in (search_start + 1)..search_end {
        let candidate = score(bin);
        if candidate > best_score {
            best_score = candidate;
            best_bin = bin;
        }
    }

    let score_center = best_score;
    let score_left = if best_bin > search_start {
        score(best_bin - 1)
    } else {
        score_center
    };
    let score_right = if best_bin + 1 < search_end {
        score(best_bin + 1)
    } else {
        score_center
    };
    let denom = score_left - 2.0 * score_center + score_right;
    let sub_bin = if denom.abs() > 1e-10 {
        (0.5 * (score_left - score_right) / denom).clamp(-0.5, 0.5)
    } else {
        0.0
    };
    best_bin as f64 + sub_bin
}

#[derive(Clone)]
struct VariantCfg {
    name: String,
    fast_alpha: f64,
    fast_window: usize,
    max_outputs: Option<usize>,
    max_outputs_until: usize,
    max_outputs_after: Option<usize>,
}

struct Variant {
    cfg: VariantCfg,
    state: OracleState,
    overall: YearStats,
    years: Vec<YearStats>,
    bias: f64,
    target_prices: Vec<(usize, f64)>,
}

fn cap_label(cap: Option<usize>) -> String {
    cap.map(|cap| cap.to_string())
        .unwrap_or_else(|| "none".to_string())
}

fn target_price(target_prices: &[(usize, f64)], height: usize) -> Option<f64> {
    target_prices
        .iter()
        .find(|(h, _)| *h == height)
        .map(|(_, price)| *price)
}

fn fixes_bad_lock(target_prices: &[(usize, f64)]) -> bool {
    target_price(target_prices, 952_287).is_some_and(|price| price > 62_000.0)
        && target_price(target_prices, 952_288).is_some_and(|price| price > 62_000.0)
}

impl Variant {
    fn new(cfg: VariantCfg, seed_bin: f64) -> Self {
        Self {
            cfg,
            state: OracleState::new(seed_bin, Config::slow()),
            overall: YearStats::new(0),
            years: Vec::new(),
            bias: 0.0,
            target_prices: Vec::new(),
        }
    }

    fn fast_config(&self) -> Config {
        Config {
            alpha: self.cfg.fast_alpha,
            window_size: self.cfg.fast_window,
            ..Config::default()
        }
    }

    fn maybe_reconfigure(&mut self, height: usize) {
        if height == START_HEIGHT_FAST {
            self.state.reconfigure(self.fast_config());
        }
    }

    fn should_drop_tx(&self, height: usize, output_count: usize) -> bool {
        if height < self.cfg.max_outputs_until {
            self.cfg.max_outputs.is_some_and(|max| output_count > max)
        } else {
            self.cfg
                .max_outputs_after
                .is_some_and(|max| output_count > max)
        }
    }

    fn add_tx(&mut self, bins: &[u16], height: usize, output_count: usize) {
        if bins.is_empty() || self.should_drop_tx(height, output_count) {
            return;
        }
        for &bin in bins {
            self.state.add(bin as usize, 1.0);
        }
    }

    fn finish_block(&mut self, height: usize) {
        self.state.finish_block();
        if TARGET_HEIGHTS.contains(&height) {
            self.target_prices
                .push((height, bin_to_cents(self.state.ref_bin) as f64 / 100.0));
        }
    }

    fn update_stats(
        &mut self,
        height: usize,
        height_bands: &[(f64, f64)],
        height_ohlc: &[[f64; 4]],
        height_years: &[u16],
    ) {
        if height >= height_bands.len() {
            return;
        }
        let (high_bin, low_bin) = height_bands[height];
        if high_bin <= 0.0 || low_bin <= 0.0 {
            return;
        }
        let err = if self.state.ref_bin < high_bin {
            self.state.ref_bin - high_bin
        } else if self.state.ref_bin > low_bin {
            self.state.ref_bin - low_bin
        } else {
            0.0
        };
        let exchange_high = height_ohlc[height][1];
        let exchange_low = height_ohlc[height][2];
        self.overall.update(err, exchange_high, exchange_low);
        self.bias += err;

        let year = height_years[height];
        if self.years.last().is_none_or(|stats| stats.year != year) {
            self.years.push(YearStats::new(year));
        }
        self.years
            .last_mut()
            .unwrap()
            .update(err, exchange_high, exchange_low);
    }
}

struct YearStats {
    year: u16,
    total_sq_err: f64,
    max_err: f64,
    total_blocks: u64,
    gt_5pct: u64,
    gt_10pct: u64,
    gt_20pct: u64,
    errors: Vec<f64>,
}

impl YearStats {
    fn new(year: u16) -> Self {
        Self {
            year,
            total_sq_err: 0.0,
            max_err: 0.0,
            total_blocks: 0,
            gt_5pct: 0,
            gt_10pct: 0,
            gt_20pct: 0,
            errors: Vec::new(),
        }
    }

    fn update(&mut self, err: f64, _exchange_high: f64, _exchange_low: f64) {
        let abs_err = err.abs();
        self.total_sq_err += err * err;
        self.total_blocks += 1;
        self.errors.push(bins_to_pct(abs_err));
        self.max_err = self.max_err.max(abs_err);
        if abs_err > BINS_5PCT {
            self.gt_5pct += 1;
        }
        if abs_err > BINS_10PCT {
            self.gt_10pct += 1;
        }
        if abs_err > BINS_20PCT {
            self.gt_20pct += 1;
        }
    }

    fn rmse_pct(&self) -> f64 {
        if self.total_blocks == 0 {
            return 0.0;
        }
        bins_to_pct((self.total_sq_err / self.total_blocks as f64).sqrt())
    }

    fn max_pct(&self) -> f64 {
        bins_to_pct(self.max_err)
    }

    fn percentile(&self, p: f64) -> f64 {
        if self.errors.is_empty() {
            return 0.0;
        }
        let mut errors = self.errors.clone();
        errors.sort_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal));
        let idx = ((p / 100.0) * (errors.len() - 1) as f64).round() as usize;
        errors[idx.min(errors.len() - 1)]
    }
}

fn main() {
    let data_dir = std::env::var("BITVIEW_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from(std::env::var("HOME").unwrap()).join(".bitview"));
    let end_override = std::env::var("ORACLE_END")
        .ok()
        .and_then(|s| s.parse::<usize>().ok());
    let stats_start = std::env::var("ORACLE_STATS_START")
        .ok()
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(START_HEIGHT_SLOW)
        .max(START_HEIGHT_SLOW);

    let indexer = common::import_indexer(&data_dir);
    let total_heights = indexer.vecs().blocks.timestamp.len();
    let end = end_override.unwrap_or(total_heights).min(total_heights);
    let manifest_dir = env!("CARGO_MANIFEST_DIR");

    let height_ohlc: Vec<[f64; 4]> = serde_json::from_str(
        &std::fs::read_to_string(format!("{manifest_dir}/examples/height_price_ohlc.json"))
            .expect("read height_price_ohlc.json"),
    )
    .expect("parse height OHLC");
    let height_bands: Vec<(f64, f64)> = height_ohlc
        .iter()
        .map(|ohlc| {
            let high = ohlc[1];
            let low = ohlc[2];
            if high > 0.0 && low > 0.0 {
                (cents_to_bin(high * 100.0), cents_to_bin(low * 100.0))
            } else {
                (0.0, 0.0)
            }
        })
        .collect();

    let timestamps: Vec<brk_types::Timestamp> = indexer.vecs().blocks.timestamp.collect();
    let height_years: Vec<u16> = timestamps
        .iter()
        .map(|ts| timestamp_to_year(**ts))
        .collect();
    let _height_day1s: Vec<usize> = timestamps
        .iter()
        .map(|ts| (**ts / 86_400).saturating_sub(GENESIS_DAY) as usize)
        .collect();

    let seed_bin = oracle_seed_bin();

    let current_alpha = 2.0 / 7.0;
    let current_window = 12;
    let mut cfgs = Vec::<VariantCfg>::new();
    let mut add_cfg = |name: String,
                       max_outputs: Option<usize>,
                       max_outputs_until: usize,
                       max_outputs_after: Option<usize>| {
        cfgs.push(VariantCfg {
            name,
            fast_alpha: current_alpha,
            fast_window: current_window,
            max_outputs,
            max_outputs_until,
            max_outputs_after,
        });
    };

    for post in [200, 250] {
        add_cfg(
            format!("pre100_post{post}"),
            Some(100),
            PaymentFilter::MODERN_TX_OUTPUT_FANOUT_CAP_START_HEIGHT,
            Some(post),
        );
    }

    cfgs.dedup_by(|a, b| a.name == b.name);
    if let Ok(only) = env::var("BRK_ORACLE_EXPERIMENT_ONLY") {
        let names = only
            .split(',')
            .map(str::trim)
            .filter(|name| !name.is_empty())
            .collect::<Vec<_>>();
        cfgs.retain(|cfg| names.iter().any(|name| *name == cfg.name));
    }
    let mut variants: Vec<Variant> = cfgs
        .into_iter()
        .map(|cfg| Variant::new(cfg, seed_bin))
        .collect();

    let total_txs = indexer.vecs().transactions.txid.len();
    let total_outputs = indexer.vecs().outputs.value.len();
    let first_tx_index: Vec<TxIndex> = indexer.vecs().transactions.first_tx_index.collect();
    let out_first: Vec<TxOutIndex> = indexer.vecs().outputs.first_txout_index.collect();
    let mut txout_cursor = indexer
        .vecs()
        .transactions
        .first_txout_index
        .reader()
        .cursor();
    let mut tx_starts: Vec<usize> = Vec::new();
    let mut values: Vec<Sats> = Vec::new();
    let mut output_types: Vec<OutputType> = Vec::new();
    let mut bins: Vec<u16> = Vec::new();

    eprintln!(
        "running {} variants over heights {START_HEIGHT_SLOW}..{end}; stats from {stats_start}",
        variants.len()
    );

    for h in START_HEIGHT_SLOW..end {
        if h % 25_000 == 0 {
            eprintln!("height {h}");
        }
        for variant in &mut variants {
            variant.maybe_reconfigure(h);
            variant.state.start_block();
        }

        let ft = first_tx_index[h];
        let next_ft = first_tx_index
            .get(h + 1)
            .copied()
            .unwrap_or(TxIndex::from(total_txs));
        let block_first_tx = ft.to_usize() + 1;
        let tx_count = next_ft.to_usize() - block_first_tx;
        let out_end = out_first
            .get(h + 1)
            .copied()
            .unwrap_or(TxOutIndex::from(total_outputs))
            .to_usize();

        txout_cursor.advance(block_first_tx - txout_cursor.position());
        tx_starts.clear();
        for _ in 0..tx_count {
            tx_starts.push(txout_cursor.next().unwrap().to_usize());
        }
        let out_start = tx_starts.first().copied().unwrap_or(out_end);

        indexer
            .vecs()
            .outputs
            .value
            .collect_range_into_at(out_start, out_end, &mut values);
        indexer.vecs().outputs.output_type.collect_range_into_at(
            out_start,
            out_end,
            &mut output_types,
        );

        for tx in 0..tx_count {
            let lo = tx_starts[tx] - out_start;
            let hi = tx_starts
                .get(tx + 1)
                .map(|s| s - out_start)
                .unwrap_or(out_end - out_start);
            if output_types[lo..hi].contains(&OutputType::OpReturn) {
                continue;
            }
            bins.clear();
            for i in lo..hi {
                if let Some(bin) = PaymentFilter::eligible_bin(values[i], output_types[i]) {
                    bins.push(bin);
                }
            }
            for variant in &mut variants {
                variant.add_tx(&bins, h, hi - lo);
            }
        }

        for variant in &mut variants {
            variant.finish_block(h);
            if h >= stats_start {
                variant.update_stats(h, &height_bands, &height_ohlc, &height_years);
            }
        }
    }

    variants.sort_by(|a, b| {
        fixes_bad_lock(&b.target_prices)
            .cmp(&fixes_bad_lock(&a.target_prices))
            .then_with(|| {
                a.overall
                    .rmse_pct()
                    .partial_cmp(&b.overall.rmse_pct())
                    .unwrap_or(Ordering::Equal)
            })
            .then_with(|| a.overall.gt_5pct.cmp(&b.overall.gt_5pct))
    });

    println!(
        "variant\tpre_cap\tpost_cap\tfixed\tmedian\tp95\tp99\tp999\trmse\tmax\tbias_bins\tgt5\tgt10\tgt20\tp952287\tp952288\ttarget_prices\trmse_by_year\tgt5_by_year"
    );
    for variant in &variants {
        let overall = &variant.overall;
        let bias = if overall.total_blocks > 0 {
            variant.bias / overall.total_blocks as f64
        } else {
            0.0
        };
        let rmse_by_year = (2015..=2026)
            .map(|year| {
                let rmse = variant
                    .years
                    .iter()
                    .find(|stats| stats.year == year)
                    .map(YearStats::rmse_pct)
                    .unwrap_or(0.0);
                format!("{year}:{rmse:.3}")
            })
            .collect::<Vec<_>>()
            .join(",");
        let gt5_by_year = (2015..=2026)
            .map(|year| {
                let gt5 = variant
                    .years
                    .iter()
                    .find(|stats| stats.year == year)
                    .map(|stats| stats.gt_5pct)
                    .unwrap_or(0);
                format!("{year}:{gt5}")
            })
            .collect::<Vec<_>>()
            .join(",");
        println!(
            "{}\t{}\t{}\t{}\t{:.3}\t{:.3}\t{:.3}\t{:.3}\t{:.3}\t{:.3}\t{:.3}\t{}\t{}\t{}\t{:.2}\t{:.2}\t{}\t{}\t{}",
            variant.cfg.name,
            cap_label(variant.cfg.max_outputs),
            cap_label(variant.cfg.max_outputs_after),
            fixes_bad_lock(&variant.target_prices),
            overall.percentile(50.0),
            overall.percentile(95.0),
            overall.percentile(99.0),
            overall.percentile(99.9),
            overall.rmse_pct(),
            overall.max_pct(),
            bias,
            overall.gt_5pct,
            overall.gt_10pct,
            overall.gt_20pct,
            target_price(&variant.target_prices, 952_287).unwrap_or(0.0),
            target_price(&variant.target_prices, 952_288).unwrap_or(0.0),
            variant
                .target_prices
                .iter()
                .map(|(height, price)| format!("{height}:{price:.2}"))
                .collect::<Vec<_>>()
                .join(","),
            rmse_by_year,
            gt5_by_year
        );
    }
}
