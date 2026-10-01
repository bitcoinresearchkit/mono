use bitview_compute::compute_rolling_extrema_from_starts;
use bitview_plugin_blocks::Vecs as BlocksVecs;
use bitview_plugin_indexer::Indexer;
use bitview_plugin_price::Vecs as PriceVecs;
use brk_error::Result;
use brk_exit::Exit;
use brk_types::PartsPerMillion32;
use vecdb::VecIndex;

use super::Vecs;

pub fn compute(
    vecs: &mut Vecs,
    indexer: &Indexer,
    prices: &PriceVecs,
    blocks: &BlocksVecs,
    exit: &Exit,
) -> Result<()> {
    let starting_height = indexer.safe_lengths().height;
    let price = &prices.spot.cents.height;

    for (min_vec, max_vec, starts) in [
        (
            &mut vecs.min._1w.cents.height,
            &mut vecs.max._1w.cents.height,
            blocks.lookback.start_vec(7),
        ),
        (
            &mut vecs.min._2w.cents.height,
            &mut vecs.max._2w.cents.height,
            blocks.lookback.start_vec(14),
        ),
        (
            &mut vecs.min._1m.cents.height,
            &mut vecs.max._1m.cents.height,
            blocks.lookback.start_vec(30),
        ),
        (
            &mut vecs.min._1y.cents.height,
            &mut vecs.max._1y.cents.height,
            blocks.lookback.start_vec(365),
        ),
    ] {
        compute_rolling_extrema_from_starts(
            min_vec,
            max_vec,
            starting_height,
            starts,
            price,
            exit,
        )?;
    }

    // 2w rolling sum of true range
    vecs.true_range_sum_2w.height.compute_rolling_sum(
        starting_height,
        blocks.lookback.start_vec(14),
        &vecs.true_range.height,
        exit,
    )?;

    vecs.choppiness_index_2w.ppm.height.compute_transform4(
        starting_height,
        &vecs.true_range_sum_2w.height,
        &vecs.max._2w.cents.height,
        &vecs.min._2w.cents.height,
        blocks.lookback.start_vec(14),
        |(h, tr_sum, max, min, window_start, ..)| {
            let range = f64::from(max) - f64::from(min);
            let n = (h.to_usize() - window_start.to_usize() + 1) as f32;
            let ci = if range > 0.0 && n > 1.0 {
                PartsPerMillion32::from((*tr_sum / range as f32).log10() as f64 / n.log10() as f64)
            } else {
                PartsPerMillion32::ZERO
            };
            (h, ci)
        },
        exit,
    )?;

    Ok(())
}
