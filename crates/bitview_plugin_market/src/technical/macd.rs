use brk_error::Result;

use brk_indexer::Indexer;
use vecdb::Exit;

use super::MacdChain;
#[allow(clippy::too_many_arguments)]
pub fn compute(
    chain: &mut MacdChain,
    indexer: &Indexer,
    blocks: &bitview_plugin_blocks::Vecs,
    prices: &bitview_plugin_price::Vecs,
    fast_days: usize,
    slow_days: usize,
    signal_days: usize,
    exit: &Exit,
) -> Result<()> {
    let starting_height = indexer.safe_lengths().height;
    let close = &prices.spot.usd.height;
    let ws_fast = blocks.lookback.start_vec(fast_days);
    let ws_slow = blocks.lookback.start_vec(slow_days);
    let ws_signal = blocks.lookback.start_vec(signal_days);

    chain
        .ema_fast
        .height
        .compute_rolling_ema(starting_height, ws_fast, close, exit)?;

    chain
        .ema_slow
        .height
        .compute_rolling_ema(starting_height, ws_slow, close, exit)?;

    chain.line.height.compute_subtract(
        starting_height,
        &chain.ema_fast.height,
        &chain.ema_slow.height,
        exit,
    )?;

    chain.signal.height.compute_rolling_ema(
        starting_height,
        ws_signal,
        &chain.line.height,
        exit,
    )?;

    chain.histogram.height.compute_subtract(
        starting_height,
        &chain.line.height,
        &chain.signal.height,
        exit,
    )?;

    Ok(())
}
