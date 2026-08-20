//! Experimental oracle accuracy report from an arbitrary start height with many
//! scoring/filter knobs.
//!
//! Use `report.rs` for the canonical README/documentation report.
//!
//! Run with: cargo run -p bitview_plugin_price --example report_from --release

use std::path::PathBuf;

use brk_oracle::{
    Config, HistogramEma, HistogramRaw, NUM_BINS, PaymentFilter, START_HEIGHT_FAST, bin_to_cents,
    cents_to_bin, pre_oracle_price_cents,
};
use brk_types::{OutputType, Sats, TxIndex, TxOutIndex};
use vecdb::{AnyVec, ReadableVec, VecIndex};

mod common;

/// Day1 1 = Jan 9, 2009 (block 1). For dates after genesis week:
/// day1 = floor(timestamp / 86400) - 14252.
const GENESIS_DAY: u32 = 14252;

const BINS_5PCT: f64 = 4.24;
const BINS_10PCT: f64 = 8.28;
const BINS_20PCT: f64 = 15.84;

/// Local copy of the oracle's 19 round-USD stencil offsets (private in lib.rs),
/// used here only for per-block alias diagnostics.
const STENCIL_OFFSETS: [i32; 19] = [
    -400, -340, -305, -260, -200, -165, -140, -120, -105, -60, 0, 35, 60, 95, 140, 200, 260, 340,
    400,
];
const N_ARMS: usize = STENCIL_OFFSETS.len();

/// Canonical L1-normalized payment shape across the 19 stencil arms, estimated
/// from true-center arm vectors over a validated block range (~$1.8k era).
/// The real price center reproduces this profile; a ½×/2× alias distorts it
/// (dark holes at no-ladder-partner arms, spurious mass from between-rung
/// payments), so correlation against it discriminates octaves the raw stencil
/// sum cannot. Order matches STENCIL_OFFSETS / the $1..$10k ladder.
const ARM_PROFILE: [f64; N_ARMS] = [
    0.022, 0.029, 0.021, 0.045, 0.060, 0.053, 0.092, 0.066, 0.077, 0.075, 0.105, 0.052, 0.075,
    0.049, 0.059, 0.043, 0.044, 0.021, 0.014,
];

/// Raw EMA arm vector at `center` (mass on each of the 19 stencil offsets).
fn arms_at(ema: &HistogramEma, center: i64) -> [f64; N_ARMS] {
    let mut arms = [0.0f64; N_ARMS];
    for (i, &off) in STENCIL_OFFSETS.iter().enumerate() {
        let idx = center + off as i64;
        if idx >= 0 && (idx as usize) < NUM_BINS {
            arms[i] = ema[idx as usize];
        }
    }
    arms
}

/// Pearson correlation between the raw EMA arm vector at `center` and a payment
/// shape `profile`. High when the local shape matches real payments, low at a
/// ½×/2× alias whose holes and spurious arms distort the shape.
fn arm_profile_corr(ema: &HistogramEma, center: i64, profile: &[f64; N_ARMS]) -> f64 {
    let arms = arms_at(ema, center);
    let n = N_ARMS as f64;
    let ma = arms.iter().sum::<f64>() / n;
    let mb = profile.iter().sum::<f64>() / n;
    let (mut num, mut da, mut db) = (0.0, 0.0, 0.0);
    for i in 0..N_ARMS {
        let (xa, xb) = (arms[i] - ma, profile[i] - mb);
        num += xa * xb;
        da += xa * xa;
        db += xb * xb;
    }
    if da > 0.0 && db > 0.0 {
        num / (da * db).sqrt()
    } else {
        0.0
    }
}

/// Shape-match via negative L1 distance between the candidate's L1-normalized arm
/// vector and the L1-normalized `profile`. 1.0 = identical shape, lower as the
/// shapes diverge. A covariance-free alternative to arm_profile_corr.
fn arm_profile_l1(ema: &HistogramEma, center: i64, profile: &[f64; N_ARMS]) -> f64 {
    let arms = arms_at(ema, center);
    let s: f64 = arms.iter().sum();
    if s <= 0.0 {
        return 0.0;
    }
    let mut dist = 0.0;
    for i in 0..N_ARMS {
        dist += (arms[i] / s - profile[i]).abs();
    }
    1.0 - dist
}

/// Shape-match via the dot product of the candidate's L1-normalized arm vector
/// with the L1-normalized `profile`. The minimal matched-filter form: the same
/// multiply-accumulate the stencil sum already does, but profile-weighted instead
/// of uniform. No covariance, no abs. Rewards mass on profile-heavy arms but
/// (unlike L1/Pearson) does NOT penalize missing mass elsewhere.
fn arm_profile_dot(ema: &HistogramEma, center: i64, profile: &[f64; N_ARMS]) -> f64 {
    let arms = arms_at(ema, center);
    let s: f64 = arms.iter().sum();
    if s <= 0.0 {
        return 0.0;
    }
    let mut dot = 0.0;
    for i in 0..N_ARMS {
        dot += (arms[i] / s) * profile[i];
    }
    dot
}

/// Stencil-arm indices whose value v has 2v NOT on the round-USD ladder
/// ($2 $3 $20 $30 $200 $300 $2000 $10000). A half-price hypothesis shifts the
/// center +60 bins; an arm is lit there only if 2v is itself a round-USD amount
/// people pay, so these eight are the only arms that fall dark at the ½x alias.
/// They carry the entire octave discrimination; the other eleven alias cleanly.
const DISC_ARMS: [usize; 8] = [1, 2, 6, 8, 12, 13, 16, 18];

/// The four "decade-anchor" arms ($10 $50 $100 $1000) whose value has BOTH 2v
/// and v/2 on the round-USD ladder, so they alias across the octave in either
/// direction and carry zero up/down information. Down-weighting them is the
/// symmetric counterpart to up-weighting the half-only DISC_ARMS, meant to
/// resist the 2x climb as well as the 1/2x slide.
const ALIAS_ARMS: [usize; 4] = [4, 9, 10, 15];

/// Sum of EMA mass on a chosen subset of stencil arms at `center`.
fn arm_subset_sum(ema: &HistogramEma, center: i64, arms: &[usize]) -> f64 {
    arms.iter()
        .map(|&i| {
            let idx = center + STENCIL_OFFSETS[i] as i64;
            if idx >= 0 && (idx as usize) < NUM_BINS {
                ema[idx as usize]
            } else {
                0.0
            }
        })
        .sum()
}

/// Raw sum of EMA mass landing on the 19 stencil arms when centered at `center`.
fn ema_stencil_sum(ema: &HistogramEma, center: i64) -> f64 {
    STENCIL_OFFSETS
        .iter()
        .map(|&off| {
            let idx = center + off as i64;
            if idx >= 0 && (idx as usize) < brk_oracle::NUM_BINS {
                ema[idx as usize]
            } else {
                0.0
            }
        })
        .sum()
}

/// log10(2) * 200 = one price octave (½× / 2×) in bins.
const OCTAVE_BINS: i64 = 60;

/// Tunable octave-guard thresholds (env-overridable for sweeping).
struct GuardCfg {
    enabled: bool,
    tau: f64,        // arm "lit" if >= tau * peak arm
    raw_margin: f64, // octave neighbor raw mass must be >= raw_margin * current
    q_margin: usize, // neighbor must have >= q_margin MORE lit arms than current
    q_min: usize,    // neighbor must have at least this many lit arms (looks full)
    // Lever 2: global re-acquire. Instead of only checking the +-60 octave
    // neighbors, scan a wide band beyond the local search window for the
    // strongest true-price peak (most lit arms, raw mass as tiebreak) and snap
    // to it when it clearly beats the locally-trapped pick. Escapes any
    // local-max trap, not just the octave alias.
    global: bool,
    global_radius: i64, // bins scanned on each side of the local pick
}

impl GuardCfg {
    fn from_env() -> Self {
        let g = |k: &str, d: f64| -> f64 {
            std::env::var(k)
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(d)
        };
        Self {
            enabled: std::env::var("OCTAVE_GUARD")
                .ok()
                .map(|v| v != "0")
                .unwrap_or(false),
            tau: g("GUARD_TAU", 0.15),
            raw_margin: g("GUARD_RAW", 1.0),
            q_margin: g("GUARD_QMARGIN", 4.0) as usize,
            q_min: g("GUARD_QMIN", 14.0) as usize,
            global: std::env::var("GLOBAL_REACQUIRE")
                .ok()
                .map(|v| v != "0")
                .unwrap_or(false),
            global_radius: g("GLOBAL_RADIUS", 600.0) as i64,
        }
    }
}

/// Number of stencil arms carrying real mass at `center`. The true price lights
/// up ~all 19; a ½×/2× alias leaves ~8 structural holes (amounts with no ladder
/// partner one octave away), so this count separates truth from alias even when
/// the normalized score-sum cannot.
fn arm_count(ema: &HistogramEma, center: i64, tau: f64) -> usize {
    let mut arms = [0.0f64; N_ARMS];
    let mut peak = 0.0f64;
    for (i, &off) in STENCIL_OFFSETS.iter().enumerate() {
        let idx = center + off as i64;
        let v = if idx >= 0 && (idx as usize) < brk_oracle::NUM_BINS {
            ema[idx as usize]
        } else {
            0.0
        };
        arms[i] = v;
        if v > peak {
            peak = v;
        }
    }
    if peak <= 0.0 {
        return 0;
    }
    arms.iter().filter(|&&v| v >= tau * peak).count()
}

/// 19-char lit/dark pattern of the stencil arms at `center` (arm i lit if its
/// EMA mass >= tau * peak arm). Order: $1 $2 $3 $5 $10 $15 $20 $25 $30 $50 $100
/// $150 $200 $300 $500 $1k $2k $5k $10k. Reveals WHICH amounts are present.
fn arm_pattern(ema: &HistogramEma, center: i64, tau: f64) -> String {
    let mut arms = [0.0f64; N_ARMS];
    let mut peak = 0.0f64;
    for (i, &off) in STENCIL_OFFSETS.iter().enumerate() {
        let idx = center + off as i64;
        let v = if idx >= 0 && (idx as usize) < brk_oracle::NUM_BINS {
            ema[idx as usize]
        } else {
            0.0
        };
        arms[i] = v;
        if v > peak {
            peak = v;
        }
    }
    arms.iter()
        .map(|&v| {
            if peak > 0.0 && v >= tau * peak {
                'L'
            } else {
                '.'
            }
        })
        .collect()
}

/// In-window stencil search (mirrors `Oracle::find_best_bin`) plus an octave
/// guard: if the half- or double-price bin lights up strictly more stencil arms
/// and carries comparable mass, snap to it. This escapes a ½×/2× alias lock that
/// the ±window can never climb the 60 bins out of on its own.
#[allow(clippy::too_many_arguments)]
fn guarded_best_bin(
    ema: &HistogramEma,
    prev_bin: f64,
    search_below: usize,
    search_above: usize,
    guard: &GuardCfg,
    arm_weights: &[f64; N_ARMS],
    corr_weight: f64,
    profile: &[f64; N_ARMS],
    metric: u8,
    stencil_weight: f64,
) -> f64 {
    let center = prev_bin.round() as usize;
    let search_start = center.saturating_sub(search_below);
    let search_end = (center + search_above + 1).min(brk_oracle::NUM_BINS);
    if search_start >= search_end {
        return prev_bin;
    }

    let mut track_norm = [0.0f64; N_ARMS];
    for (i, &off) in STENCIL_OFFSETS.iter().enumerate() {
        for bin in search_start..search_end {
            let idx = bin as i32 + off;
            if idx >= 0 && (idx as usize) < brk_oracle::NUM_BINS {
                track_norm[i] = track_norm[i].max(ema[idx as usize]);
            }
        }
    }
    let score = |bin: usize| -> f64 {
        let mut total = 0.0;
        if stencil_weight != 0.0 {
            for (i, &off) in STENCIL_OFFSETS.iter().enumerate() {
                let idx = bin as i32 + off;
                if idx >= 0 && (idx as usize) < brk_oracle::NUM_BINS && track_norm[i] > 0.0 {
                    total += stencil_weight * arm_weights[i] * ema[idx as usize] / track_norm[i];
                }
            }
        }
        if corr_weight != 0.0 {
            let shape = match metric {
                1 => arm_profile_l1(ema, bin as i64, profile),
                2 => arm_profile_dot(ema, bin as i64, profile),
                _ => arm_profile_corr(ema, bin as i64, profile),
            };
            total += corr_weight * shape;
        }
        total
    };

    let mut best_bin = search_start;
    let mut best_score = score(search_start);
    for bin in (search_start + 1)..search_end {
        let c = score(bin);
        if c > best_score {
            best_score = c;
            best_bin = bin;
        }
    }

    if guard.enabled {
        let b = best_bin as i64;
        let qb = arm_count(ema, b, guard.tau);
        let raw_b = ema_stencil_sum(ema, b);
        let mut target = b;
        if guard.global {
            // Scan beyond the local window for the strongest peak by lit-arm
            // count (raw mass as tiebreak), considering only bins carrying at
            // least the local pick's raw mass. Snap to it when it lights up
            // q_margin more arms and looks full (>= q_min), regardless of how
            // many bins away it sits.
            let lo = (b - guard.global_radius).max(0);
            let hi = (b + guard.global_radius).min(brk_oracle::NUM_BINS as i64 - 1);
            let mut best: Option<(i64, usize, f64)> = None;
            for n in lo..=hi {
                if n >= search_start as i64 && n < search_end as i64 {
                    continue; // window interior is owned by the local search
                }
                let raw_n = ema_stencil_sum(ema, n);
                if raw_n < guard.raw_margin * raw_b {
                    continue;
                }
                let qn = arm_count(ema, n, guard.tau);
                let better = best.is_none_or(|(_, sq, sr)| qn > sq || (qn == sq && raw_n > sr));
                if better {
                    best = Some((n, qn, raw_n));
                }
            }
            if let Some((n, qn, _)) = best
                && qn >= qb + guard.q_margin
                && qn >= guard.q_min
            {
                target = n;
            }
        } else {
            let mut best: Option<(usize, f64)> = None;
            for &delta in &[-OCTAVE_BINS, OCTAVE_BINS] {
                let n = b + delta;
                if n < 0 || n as usize >= brk_oracle::NUM_BINS {
                    continue;
                }
                let qn = arm_count(ema, n, guard.tau);
                let raw_n = ema_stencil_sum(ema, n);
                if qn >= qb + guard.q_margin
                    && qn >= guard.q_min
                    && raw_n >= guard.raw_margin * raw_b
                {
                    let better = best.is_none_or(|(sq, sr)| qn > sq || (qn == sq && raw_n > sr));
                    if better {
                        best = Some((qn, raw_n));
                        target = n;
                    }
                }
            }
        }
        if target != b {
            return target as f64;
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

fn bins_to_pct(bins: f64) -> f64 {
    (10.0_f64.powf(bins / 200.0) - 1.0) * 100.0
}

/// Per-block EMA contribution weighting. `Off` keeps the raw count sum (a flood
/// block dominates the window); `Unit` rescales every block to the same total
/// mass (one block = one vote); `Cap` only scales down blocks above a ceiling.
#[derive(Clone, Copy, PartialEq)]
enum NormMode {
    Off,
    Unit,
    Cap,
}

/// Scale factor applied to a block's bin counts before folding into the EMA.
fn norm_scale(total: u64, mode: NormMode, cap: f64, target: f64) -> f64 {
    if total == 0 {
        return 0.0;
    }
    match mode {
        NormMode::Off => 1.0,
        NormMode::Unit => target / total as f64,
        NormMode::Cap => (cap / total as f64).min(1.0),
    }
}

fn timestamp_to_year(ts: u32) -> u16 {
    let years_since_1970 = ts as f64 / 31557600.0;
    (1970.0 + years_since_1970) as u16
}

struct YearStats {
    year: u16,
    total_sq_err: f64,
    max_err: f64,
    total_blocks: u64,
    gt_5pct: u64,
    gt_10pct: u64,
    gt_20pct: u64,
    min_price: f64,
    max_price: f64,
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
            min_price: f64::MAX,
            max_price: 0.0,
            errors: Vec::new(),
        }
    }

    fn update(&mut self, err: f64, exchange_high: f64, exchange_low: f64) {
        let abs_err = err.abs();
        self.total_sq_err += err * err;
        self.total_blocks += 1;
        self.errors.push(bins_to_pct(abs_err));
        if abs_err > self.max_err {
            self.max_err = abs_err;
        }
        if abs_err > BINS_5PCT {
            self.gt_5pct += 1;
        }
        if abs_err > BINS_10PCT {
            self.gt_10pct += 1;
        }
        if abs_err > BINS_20PCT {
            self.gt_20pct += 1;
        }
        if exchange_high > self.max_price {
            self.max_price = exchange_high;
        }
        if exchange_low > 0.0 && exchange_low < self.min_price {
            self.min_price = exchange_low;
        }
    }

    fn rmse_pct(&self) -> f64 {
        bins_to_pct((self.total_sq_err / self.total_blocks as f64).sqrt())
    }

    fn max_pct(&self) -> f64 {
        bins_to_pct(self.max_err)
    }

    fn median_pct(&mut self) -> f64 {
        self.errors.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let n = self.errors.len();
        if n == 0 { 0.0 } else { self.errors[n / 2] }
    }

    fn percentile(&self, p: f64) -> f64 {
        let n = self.errors.len();
        if n == 0 {
            return 0.0;
        }
        let idx = ((p / 100.0) * (n - 1) as f64).round() as usize;
        self.errors[idx.min(n - 1)]
    }
}

/// Oracle OHLC for a single day, built from per-block prices.
struct DayCandle {
    day1: usize,
    open: f64,
    high: f64,
    low: f64,
    close: f64,
}

struct BlockError {
    height: usize,
    oracle_price: f64,
    exchange_low: f64,
    exchange_high: f64,
    error_pct: f64,
}

fn main() {
    let data_dir = std::env::var("BITVIEW_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            let home = std::env::var("HOME").unwrap();
            PathBuf::from(home).join(".bitview")
        });

    let start = std::env::var("ORACLE_START")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(START_HEIGHT_FAST);
    let end_override = std::env::var("ORACLE_END")
        .ok()
        .and_then(|s| s.parse::<usize>().ok());
    let trace_every: usize = std::env::var("TRACE_EVERY")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(5000);

    let indexer = common::import_indexer(&data_dir);
    let total_heights = indexer.vecs().blocks.timestamp.len();
    let manifest_dir = env!("CARGO_MANIFEST_DIR");

    let height_ohlc: Vec<[f64; 4]> = serde_json::from_str(
        &std::fs::read_to_string(format!("{manifest_dir}/examples/height_price_ohlc.json"))
            .expect("Failed to read height_price_ohlc.json"),
    )
    .expect("Failed to parse height OHLC");

    let daily_ohlc: Vec<[f64; 4]> = serde_json::from_str(
        &std::fs::read_to_string(format!("{manifest_dir}/examples/date_price_ohlc.json"))
            .expect("Failed to read date_price_ohlc.json"),
    )
    .expect("Failed to parse daily OHLC");

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

    // Read block timestamps for year + day1 mapping.
    let timestamps: Vec<brk_types::Timestamp> = indexer.vecs().blocks.timestamp.collect();
    let height_years: Vec<u16> = timestamps
        .iter()
        .map(|ts| timestamp_to_year(**ts))
        .collect();
    let height_day1s: Vec<usize> = timestamps
        .iter()
        .map(|ts| (**ts / 86400).saturating_sub(GENESIS_DAY) as usize)
        .collect();

    // Seed price at height `start - 1`. The baked prices.txt only covers the
    // pre-oracle seed range; past it this experimental harness warm-starts from
    // exchange OHLC so arbitrary later starts get a primed ref_bin.
    let seed_height = start.saturating_sub(1);
    let start_price: f64 = pre_oracle_price_cents(seed_height)
        .map(|cents| cents.inner() as f64 / 100.0)
        .unwrap_or_else(|| {
            let o = height_ohlc.get(seed_height).copied().unwrap_or([0.0; 4]);
            if o[3] > 0.0 {
                o[3]
            } else {
                (o[1] + o[2]) / 2.0
            }
        });
    // Exact seed override (reproduce the committed prices.txt seed at a start the
    // truncated working-tree prices.txt no longer covers).
    let start_price = std::env::var("SEED")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(start_price);

    let mut config = Config::default();
    if let Some(w) = std::env::var("EMA_WINDOW")
        .ok()
        .and_then(|s| s.parse().ok())
    {
        config.window_size = w;
    }
    if let Some(a) = std::env::var("EMA_ALPHA").ok().and_then(|s| s.parse().ok()) {
        config.alpha = a;
    }
    // Investigation default: widened up-reach (9 -> 12) to survive fast rallies
    // like the 2018-04-12 candle. Kept here only; config.rs is untouched.
    config.search_below = std::env::var("SEARCH_BELOW")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(12);
    if let Some(sa) = std::env::var("SEARCH_ABOVE")
        .ok()
        .and_then(|s| s.parse().ok())
    {
        config.search_above = sa;
    }
    let guard = GuardCfg::from_env();
    // Lever 3: up-weight the 8 octave-discriminating arms (2v not on the ladder)
    // in the stencil score. They alone separate a center from its half-price
    // alias; the other 11 alias cleanly and only dilute the up/down decision.
    let disc_weight: f64 = std::env::var("DISC_WEIGHT")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(1.0);
    let alias_weight: f64 = std::env::var("ALIAS_WEIGHT")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(1.0);
    // Shape-correlation restoring force: add corr_weight * Pearson(arms, profile)
    // to each candidate bin's stencil score. Pulls the ±window pick toward the
    // octave whose arm-shape matches real payments, resisting the ½×/2× slide
    // without a hard continuity clamp. 0 = off (bit-identical to baseline).
    let corr_weight: f64 = std::env::var("CORR_WEIGHT")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(0.0);
    // EMA rate for the adaptive shape template. The profile tracks the current
    // price regime (which arms are tall) so correlation stays meaningful as the
    // price moves an octave over months, while remaining slow enough to ride
    // through a transient ½×/2× slide (tens of blocks) without adapting to it.
    let corr_beta: f64 = std::env::var("CORR_BETA")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(0.002);
    // Apply the corr term only below this height. Lets the pre-X (slow) leg use
    // corr while the post-X (fast) leg stays bit-identical to the no-corr baseline.
    // Default = always on (global corr).
    let corr_until: usize = std::env::var("CORR_UNTIL")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(usize::MAX);
    // Shape-match metric: "l1" = negative L1 distance, "dot" = matched-filter dot
    // product (both covariance-free), else Pearson.
    let metric: u8 = match std::env::var("PROFILE_METRIC").as_deref() {
        Ok("l1") => 1,
        Ok("dot") => 2,
        _ => 0,
    };
    let metric_name = ["pearson", "l1", "dot"][metric as usize];
    // Profile seed: "bootstrap" = seed from the first warm-up pick's shape (no magic
    // constant), "uniform"/"flat" = every arm equal (1/N_ARMS), else the static
    // ARM_PROFILE.
    let profile_seed = std::env::var("PROFILE_SEED").ok();
    let bootstrap_profile = profile_seed.as_deref() == Some("bootstrap");
    let uniform_profile = matches!(profile_seed.as_deref(), Some("uniform") | Some("flat"));
    // Stencil-sum weight (default 1). Set 0 for SHAPE-ONLY scoring: the shape match
    // does both within-octave localization and octave discrimination, no stencil
    // term and no cw balance to tune.
    let stencil_weight: f64 = std::env::var("STENCIL_WEIGHT")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(1.0);
    eprintln!(
        "  shape: metric={} seed={} stencil_weight={}",
        metric_name,
        if bootstrap_profile {
            "bootstrap"
        } else if uniform_profile {
            "uniform"
        } else {
            "static"
        },
        stencil_weight,
    );
    // Mid-run regime switch, mirrors production Oracle::reconfigure at START_HEIGHT_FAST:
    // at SWITCH_AT rebuild the EMA to SWITCH_WINDOW/SWITCH_ALPHA and warm-start fresh
    // (ring reset, ref_bin kept) - the same state as a fresh warm-up. Search window
    // is unchanged (both regimes share it). 0 = no switch (single-config baseline).
    let switch_at: usize = std::env::var("SWITCH_AT")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);
    let switch_window: usize = std::env::var("SWITCH_WINDOW")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(12);
    let switch_alpha: f64 = std::env::var("SWITCH_ALPHA")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(2.0 / 7.0);
    let mut arm_weights = [1.0f64; N_ARMS];
    for &i in &DISC_ARMS {
        arm_weights[i] = disc_weight;
    }
    for &i in &ALIAS_ARMS {
        arm_weights[i] = alias_weight;
    }
    eprintln!(
        "  disc_weight={disc_weight} on {DISC_ARMS:?}; alias_weight={alias_weight} on {ALIAS_ARMS:?}; corr_weight={corr_weight}"
    );
    let anom_thresh: f64 = std::env::var("ANOM_THRESH")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(0.0);
    let norm_mode = match std::env::var("NORM_MODE").as_deref() {
        Ok("unit") => NormMode::Unit,
        Ok("cap") => NormMode::Cap,
        _ => NormMode::Off,
    };
    let norm_cap: f64 = std::env::var("NORM_CAP")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(8000.0);
    let norm_target: f64 = std::env::var("NORM_TARGET")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(4000.0);
    // Drop batch-payout txs (UTXOracle uses exactly-2-output; we cap instead).
    // 0 = disabled. A flood block's 591-output txs are dropped at 100.
    let max_outputs: usize = std::env::var("MAX_OUTPUTS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(100);
    // Apply the output-count filter only below this height (it helps the thin
    // 2018-2020 era, mildly hurts high-volume years). Default = always on.
    let max_outputs_until: usize = std::env::var("MAX_OUTPUTS_UNTIL")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(usize::MAX);
    eprintln!(
        "  norm: mode={} cap={} target={} max_outputs={}",
        match norm_mode {
            NormMode::Off => "off",
            NormMode::Unit => "unit",
            NormMode::Cap => "cap",
        },
        norm_cap,
        norm_target,
        max_outputs,
    );
    eprintln!(
        "  cfg: window_size={} alpha={:.5} (~{:.0}-block span) search -{}/+{} guard={} (tau={} raw={} qm={} qmin={}) global={} radius={}",
        config.window_size,
        config.alpha,
        2.0 / config.alpha - 1.0,
        config.search_below,
        config.search_above,
        guard.enabled,
        guard.tau,
        guard.raw_margin,
        guard.q_margin,
        guard.q_min,
        guard.global,
        guard.global_radius,
    );
    if switch_at != 0 {
        eprintln!(
            "  switch: at height {switch_at} -> window={switch_window} alpha={switch_alpha:.5}"
        );
    }
    let (sb, sa) = (config.search_below, config.search_above);
    let mut window_size = config.window_size;
    let alpha = config.alpha;
    let mut weights: Vec<f64> = (0..window_size)
        .map(|i| alpha * (1.0 - alpha).powi(i as i32))
        .collect();
    let mut ring: Vec<Vec<f64>> = vec![vec![0.0; NUM_BINS]; window_size];
    let mut ring_cursor = 0usize;
    let mut filled = 0usize;
    let mut ema = HistogramEma::zeros();
    let mut ref_bin = cents_to_bin(start_price * 100.0);
    // Adaptive shape template, re-estimated each block from the L1-normalized arm
    // vector at the pick. Static seed = ARM_PROFILE; bootstrap = filled from the
    // first warm-up pick (zeros until then, so corr contributes nothing yet).
    let mut profile = if bootstrap_profile {
        [0.0f64; N_ARMS]
    } else if uniform_profile {
        [1.0 / N_ARMS as f64; N_ARMS]
    } else {
        ARM_PROFILE
    };
    let mut profile_seeded = !bootstrap_profile;

    // Parity check (VERIFY_PROD=1): drive the PRODUCTION Oracle (lib.rs) over the
    // same per-block histograms and confirm its ref_bin matches this harness pick
    // bit-for-bit. Only meaningful under the shipped slow config (EMA_ALPHA=0.10
    // EMA_WINDOW=40 search 12/11, metric=l1, cw=8, norm off, ORACLE_END<=508000 so
    // corr stays on the whole run).
    let verify_prod = std::env::var("VERIFY_PROD").as_deref() == Ok("1");
    let mut prod_oracle = brk_oracle::Oracle::new(ref_bin, brk_oracle::Config::slow());
    let mut prod_max_diff = 0.0f64;
    let mut prod_diff_blocks = 0usize;

    // Lever 4: a parallel "sharp" detection EMA (fast span, short window) folded
    // from the same per-block hists. The slow EMA above still sets the price; this
    // is diagnostic only, used to check whether the true-price stencil holes (the
    // arm-count contrast that the smeared slow EMA flattens during a crash) survive
    // when the histogram is not smoothed.
    let sharp_span: f64 = std::env::var("SHARP_SPAN")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(3.0);
    let sharp_window: usize = std::env::var("SHARP_WINDOW")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(6);
    let sharp_alpha = 2.0 / (sharp_span + 1.0);
    let sharp_weights: Vec<f64> = (0..sharp_window)
        .map(|i| sharp_alpha * (1.0 - sharp_alpha).powi(i as i32))
        .collect();
    let mut sharp_ring: Vec<Vec<f64>> = vec![vec![0.0; NUM_BINS]; sharp_window];
    let mut sharp_cursor = 0usize;
    let mut sharp_filled = 0usize;
    let mut sharp_ema = HistogramEma::zeros();
    eprintln!("  sharp: span={sharp_span:.0} window={sharp_window} alpha={sharp_alpha:.5}");

    let total_txs = indexer.vecs().transactions.txid.len();
    let total_outputs = indexer.vecs().outputs.value.len();

    // Pre-collect height-indexed vecs (small). Transaction-indexed vecs are too
    // large, so the tx-indexed first_txout_index is read through a forward cursor.
    let first_tx_index: Vec<TxIndex> = indexer.vecs().transactions.first_tx_index.collect();
    let out_first: Vec<TxOutIndex> = indexer.vecs().outputs.first_txout_index.collect();
    let mut txout_cursor = indexer
        .vecs()
        .transactions
        .first_txout_index
        .reader()
        .cursor();
    let mut tx_starts: Vec<usize> = Vec::new();

    let mut year_stats: Vec<YearStats> = Vec::new();
    let mut overall = YearStats::new(0);
    let mut worst_blocks: Vec<BlockError> = Vec::new();
    let mut total_bias = 0.0f64;

    // Track oracle daily candles.
    let mut oracle_candles: Vec<DayCandle> = Vec::new();
    let mut current_di: Option<usize> = None;

    let loop_end = end_override.unwrap_or(total_heights).min(total_heights);
    for h in start..loop_end {
        if switch_at != 0 && h == switch_at {
            window_size = switch_window;
            weights = (0..window_size)
                .map(|i| switch_alpha * (1.0 - switch_alpha).powi(i as i32))
                .collect();
            ring = vec![vec![0.0; NUM_BINS]; window_size];
            ring_cursor = 0;
            filled = 0;
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

        // First txout index of each non-coinbase tx, for per-tx grouping.
        txout_cursor.advance(block_first_tx - txout_cursor.position());
        tx_starts.clear();
        for _ in 0..tx_count {
            tx_starts.push(txout_cursor.next().unwrap().to_usize());
        }
        let out_start = tx_starts.first().copied().unwrap_or(out_end);

        let values: Vec<Sats> = indexer
            .vecs()
            .outputs
            .value
            .collect_range_at(out_start, out_end);
        let output_types: Vec<OutputType> = indexer
            .vecs()
            .outputs
            .output_type
            .collect_range_at(out_start, out_end);

        // Drop every output of a tx carrying an OP_RETURN (protocol machinery).
        let mut hist = HistogramRaw::zeros();
        for tx in 0..tx_count {
            let lo = tx_starts[tx] - out_start;
            let hi = tx_starts
                .get(tx + 1)
                .map(|s| s - out_start)
                .unwrap_or(out_end - out_start);
            if output_types[lo..hi].contains(&OutputType::OpReturn) {
                continue;
            }
            if max_outputs > 0 && h < max_outputs_until && (hi - lo) > max_outputs {
                continue;
            }
            for i in lo..hi {
                if let Some(bin) = PaymentFilter::eligible_bin(values[i], output_types[i]) {
                    hist.increment(bin as usize);
                }
            }
        }

        let total: u64 = (0..NUM_BINS).map(|b| hist[b] as u64).sum();
        let scale = norm_scale(total, norm_mode, norm_cap, norm_target);
        {
            let slot = &mut ring[ring_cursor];
            for b in 0..NUM_BINS {
                slot[b] = hist[b] as f64 * scale;
            }
        }
        ring_cursor = (ring_cursor + 1) % window_size;
        if filled < window_size {
            filled += 1;
        }
        ema.fill(0.0);
        (0..filled).for_each(|age| {
            let idx = (ring_cursor + window_size - 1 - age) % window_size;
            let w = weights[age];
            let block = &ring[idx];
            for b in 0..NUM_BINS {
                ema[b] += w * block[b];
            }
        });
        // Sharp detection EMA (diagnostic only - does not drive the price).
        {
            let slot = &mut sharp_ring[sharp_cursor];
            for b in 0..NUM_BINS {
                slot[b] = hist[b] as f64 * scale;
            }
        }
        sharp_cursor = (sharp_cursor + 1) % sharp_window;
        if sharp_filled < sharp_window {
            sharp_filled += 1;
        }
        sharp_ema.fill(0.0);
        (0..sharp_filled).for_each(|age| {
            let idx = (sharp_cursor + sharp_window - 1 - age) % sharp_window;
            let w = sharp_weights[age];
            let block = &sharp_ring[idx];
            for b in 0..NUM_BINS {
                sharp_ema[b] += w * block[b];
            }
        });
        let cw = if h < corr_until { corr_weight } else { 0.0 };
        ref_bin = guarded_best_bin(
            &ema,
            ref_bin,
            sb,
            sa,
            &guard,
            &arm_weights,
            cw,
            &profile,
            metric,
            stencil_weight,
        );
        let oracle_price = bin_to_cents(ref_bin) as f64 / 100.0;

        if verify_prod {
            let prod_bin = prod_oracle.process_histogram(&hist);
            let d = (prod_bin - ref_bin).abs();
            prod_max_diff = prod_max_diff.max(d);
            if prod_bin != ref_bin {
                prod_diff_blocks += 1;
            }
        }

        // Re-estimate the shape template from the L1-normalized arm vector at the
        // new pick, blended in slowly so a transient octave slide cannot corrupt it.
        if cw != 0.0 {
            let arms = arms_at(&ema, ref_bin.round() as i64);
            let s: f64 = arms.iter().sum();
            if s > 0.0 {
                if !profile_seeded {
                    for i in 0..N_ARMS {
                        profile[i] = arms[i] / s;
                    }
                    profile_seeded = true;
                } else {
                    for i in 0..N_ARMS {
                        profile[i] = (1.0 - corr_beta) * profile[i] + corr_beta * (arms[i] / s);
                    }
                }
            }
        }

        let o = height_ohlc.get(h).copied().unwrap_or([0.0; 4]);
        let (ex_high, ex_low, ex_close) = (o[1], o[2], o[3]);
        let band_err = if ex_high > 0.0 && ex_low > 0.0 {
            if oracle_price > ex_high {
                (oracle_price - ex_high) / ex_high * 100.0
            } else if oracle_price < ex_low {
                (oracle_price - ex_low) / ex_low * 100.0
            } else {
                0.0
            }
        } else {
            0.0
        };
        let do_print = h % trace_every == 0 || (anom_thresh > 0.0 && band_err.abs() >= anom_thresh);
        if do_print {
            let eligible: u32 = (0..brk_oracle::NUM_BINS).map(|b| hist[b]).sum();
            // true_bin centered on exchange close; +60 bins = half price, -60 = double.
            let true_bin = if ex_close > 0.0 {
                cents_to_bin(ex_close * 100.0).round() as i64
            } else {
                ref_bin.round() as i64
            };
            let s_true = ema_stencil_sum(&ema, true_bin);
            let s_half = ema_stencil_sum(&ema, true_bin + 60);
            let s_dbl = ema_stencil_sum(&ema, true_bin - 60);
            let qt = arm_count(&ema, true_bin, guard.tau);
            let qh = arm_count(&ema, true_bin + 60, guard.tau);
            let qd = arm_count(&ema, true_bin - 60, guard.tau);
            let pat = arm_pattern(&ema, true_bin, guard.tau);
            // Octave-discriminating subset only: mass at true vs half center.
            let dt = arm_subset_sum(&ema, true_bin, &DISC_ARMS);
            let dh = arm_subset_sum(&ema, true_bin + 60, &DISC_ARMS);
            // Same arm-count contrast measured on the sharp detection EMA.
            let qst = arm_count(&sharp_ema, true_bin, guard.tau);
            let qsh = arm_count(&sharp_ema, true_bin + 60, guard.tau);
            let qsd = arm_count(&sharp_ema, true_bin - 60, guard.tau);
            let spat = arm_pattern(&sharp_ema, true_bin, guard.tau);
            let ts_secs: u32 = *timestamps[h];
            eprintln!(
                "{h}\t{ts_secs}\t{oracle_price:.0}\t{ex_close:.0}\t{band_err:+.2}\t{eligible}\tT={s_true:.1}\tH={s_half:.1}\tD={s_dbl:.1}\tQt={qt}\tQh={qh}\tQd={qd}\tDt={dt:.1}\tDh={dh:.1}\t{pat}\t|sharp Qt={qst} Qh={qsh} Qd={qsd}\t{spat}"
            );
        }

        // Build oracle daily candle.
        let di = height_day1s[h];
        if current_di != Some(di) {
            current_di = Some(di);
            oracle_candles.push(DayCandle {
                day1: di,
                open: oracle_price,
                high: oracle_price,
                low: oracle_price,
                close: oracle_price,
            });
        } else {
            let candle = oracle_candles.last_mut().unwrap();
            if oracle_price > candle.high {
                candle.high = oracle_price;
            }
            if oracle_price < candle.low {
                candle.low = oracle_price;
            }
            candle.close = oracle_price;
        }

        // Per-block error stats.
        if h < height_bands.len() {
            let (high_bin, low_bin) = height_bands[h];
            if high_bin > 0.0 && low_bin > 0.0 {
                let err = if ref_bin < high_bin {
                    ref_bin - high_bin
                } else if ref_bin > low_bin {
                    ref_bin - low_bin
                } else {
                    0.0
                };

                let exchange_high = height_ohlc[h][1];
                let exchange_low = height_ohlc[h][2];

                overall.update(err, exchange_high, exchange_low);
                total_bias += err;

                let year = height_years[h];
                if year_stats.is_empty() || year_stats.last().unwrap().year != year {
                    year_stats.push(YearStats::new(year));
                }
                year_stats
                    .last_mut()
                    .unwrap()
                    .update(err, exchange_high, exchange_low);

                if err.abs() > BINS_5PCT {
                    worst_blocks.push(BlockError {
                        height: h,
                        oracle_price,
                        exchange_low,
                        exchange_high,
                        error_pct: if err < 0.0 {
                            -bins_to_pct(err.abs())
                        } else {
                            bins_to_pct(err.abs())
                        },
                    });
                }
            }
        }
    }

    if verify_prod {
        eprintln!(
            "  VERIFY_PROD: production Oracle vs harness - max ref_bin diff {prod_max_diff:.6}, {prod_diff_blocks} blocks differ"
        );
    }

    worst_blocks.sort_by(|a, b| b.error_pct.abs().partial_cmp(&a.error_pct.abs()).unwrap());
    overall.errors.sort_by(|a, b| a.partial_cmp(b).unwrap());

    // Daily candle comparison: oracle OHLC vs exchange OHLC.
    let mut daily_open_errors: Vec<f64> = Vec::new();
    let mut daily_high_errors: Vec<f64> = Vec::new();
    let mut daily_low_errors: Vec<f64> = Vec::new();
    let mut daily_close_errors: Vec<f64> = Vec::new();
    let mut daily_days = 0u64;

    for candle in &oracle_candles {
        let di = candle.day1;
        if di >= daily_ohlc.len() {
            continue;
        }
        let ex = &daily_ohlc[di];
        if ex[0] <= 0.0 || ex[3] <= 0.0 {
            continue;
        }
        let ex_open = ex[0];
        let ex_high = ex[1];
        let ex_low = ex[2];
        let ex_close = ex[3];

        // Error as percentage: (oracle - exchange) / exchange * 100
        daily_open_errors.push((candle.open - ex_open) / ex_open * 100.0);
        daily_high_errors.push((candle.high - ex_high) / ex_high * 100.0);
        daily_low_errors.push((candle.low - ex_low) / ex_low * 100.0);
        daily_close_errors.push((candle.close - ex_close) / ex_close * 100.0);
        daily_days += 1;
    }

    fn daily_stats(errors: &mut [f64]) -> (f64, f64, f64) {
        let n = errors.len() as f64;
        let rmse = (errors.iter().map(|e| e * e).sum::<f64>() / n).sqrt();
        errors.sort_by(|a, b| a.abs().partial_cmp(&b.abs()).unwrap());
        let max = errors.last().map(|e| e.abs()).unwrap_or(0.0);
        let median = errors[errors.len() / 2].abs();
        (median, rmse, max)
    }

    let (open_med, open_rmse, open_max) = daily_stats(&mut daily_open_errors);
    let (high_med, high_rmse, high_max) = daily_stats(&mut daily_high_errors);
    let (low_med, low_rmse, low_max) = daily_stats(&mut daily_low_errors);
    let (close_med, close_rmse, close_max) = daily_stats(&mut daily_close_errors);

    // Print report.
    println!();
    println!("  brk_oracle accuracy report");
    println!("  ══════════════════════════");
    println!();
    println!(
        "  Config:       w{}, alpha={:.5}, search -{}/+{}, experimental knobs",
        window_size, config.alpha, config.search_below, config.search_above
    );
    println!(
        "  Test range:   height {} .. {} ({} blocks), seed ${:.2}",
        start,
        loop_end - 1,
        overall.total_blocks,
        start_price,
    );
    println!(
        "  Price range:  ${:.0} .. ${:.0}",
        overall.min_price, overall.max_price
    );

    println!();
    println!("  Per-block accuracy (vs per-height exchange OHLC):");
    println!("    Median:      {:.3}%", overall.percentile(50.0));
    println!("    95th pct:    {:.3}%", overall.percentile(95.0));
    println!("    99th pct:    {:.3}%", overall.percentile(99.0));
    println!("    99.9th pct:  {:.3}%", overall.percentile(99.9));
    println!("    RMSE:        {:.3}%", overall.rmse_pct());
    println!("    Max:         {:.1}%", overall.max_pct());
    println!(
        "    Bias:        {:+.2} bins",
        total_bias / overall.total_blocks as f64
    );
    println!(
        "    > 5%:        {} blocks ({:.3}%)",
        overall.gt_5pct,
        overall.gt_5pct as f64 / overall.total_blocks as f64 * 100.0
    );
    println!("    > 10%:       {} blocks", overall.gt_10pct);
    println!("    > 20%:       {} blocks", overall.gt_20pct);

    println!();
    println!(
        "  Daily candle accuracy ({} days, vs exchange daily OHLC):",
        daily_days
    );
    println!(
        "    {:>8} {:>10} {:>10} {:>10}",
        "", "Median", "RMSE", "Max"
    );
    println!(
        "    {:>8} {:>9.2}% {:>9.2}% {:>9.1}%",
        "Open", open_med, open_rmse, open_max
    );
    println!(
        "    {:>8} {:>9.2}% {:>9.2}% {:>9.1}%",
        "High", high_med, high_rmse, high_max
    );
    println!(
        "    {:>8} {:>9.2}% {:>9.2}% {:>9.1}%",
        "Low", low_med, low_rmse, low_max
    );
    println!(
        "    {:>8} {:>9.2}% {:>9.2}% {:>9.1}%",
        "Close", close_med, close_rmse, close_max
    );

    println!();
    println!("  By year:");
    println!(
        "    {:<6} {:>7} {:>9} {:>9} {:>9} {:>6} {:>5} {:>5} {:>14}",
        "Year", "Blocks", "Median", "RMSE", "Max", ">5%", ">10%", ">20%", "Price range"
    );
    println!("    {}", "-".repeat(80));
    for ys in &mut year_stats {
        let median = ys.median_pct();
        println!(
            "    {:<6} {:>7} {:>8.3}% {:>8.3}% {:>8.1}% {:>6} {:>5} {:>5}   ${:.0}..${:.0}",
            ys.year,
            ys.total_blocks,
            median,
            ys.rmse_pct(),
            ys.max_pct(),
            ys.gt_5pct,
            ys.gt_10pct,
            ys.gt_20pct,
            ys.min_price,
            ys.max_price,
        );
    }

    if !worst_blocks.is_empty() {
        println!();
        println!("  Worst blocks:");
        let show = worst_blocks.len().min(10);
        for wb in &worst_blocks[..show] {
            let dir = if wb.error_pct < 0.0 { "above" } else { "below" };
            println!(
                "    height {:>7}: oracle ${:>9.0}, exchange ${:.0}..${:.0} ({:+.1}%, {})",
                wb.height, wb.oracle_price, wb.exchange_low, wb.exchange_high, wb.error_pct, dir
            );
        }
        if worst_blocks.len() > show {
            println!("    ... and {} more", worst_blocks.len() - show);
        }
    }

    println!();
}
