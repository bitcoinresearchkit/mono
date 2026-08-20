//! Generate detailed oracle accuracy report for README / documentation.
//!
//! Run with: cargo run -p bitview_plugin_price --example report --release

use std::path::PathBuf;

use brk_oracle::{
    Config, Oracle, PaymentFilter, START_HEIGHT_FAST, START_HEIGHT_SLOW, bin_to_cents, cents_to_bin,
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

fn bins_to_pct(bins: f64) -> f64 {
    (10.0_f64.powf(bins / 200.0) - 1.0) * 100.0
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

    let mut oracle = Oracle::from_seed();

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

    for h in START_HEIGHT_SLOW..total_heights {
        if h == START_HEIGHT_FAST {
            oracle.reconfigure(Config::default());
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

        let tx_outputs = (0..tx_count).map(|tx| {
            let lo = tx_starts[tx] - out_start;
            let hi = tx_starts
                .get(tx + 1)
                .map(|s| s - out_start)
                .unwrap_or(out_end - out_start);
            values[lo..hi]
                .iter()
                .copied()
                .zip(output_types[lo..hi].iter().copied())
        });
        let hist = PaymentFilter::for_height(h).histogram(tx_outputs);

        let ref_bin = oracle.process_histogram(&hist);
        let oracle_price = bin_to_cents(ref_bin) as f64 / 100.0;

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
        "  Config:       slow w40 alpha=0.10 until {START_HEIGHT_FAST}, then w12 alpha=2/7; shared payment filter"
    );
    println!(
        "  Test range:   height {} .. {} ({} exchange-covered blocks)",
        START_HEIGHT_SLOW,
        total_heights - 1,
        overall.total_blocks
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
