use brk_error::Result;

use brk_error::Error;
use brk_types::{
    Bitcoin, Cents, Date, Day1, Dollars, Height, PartsPerMillionSigned64, Sats, Version,
};
use vecdb::{
    BinaryTransform, CachedBoxedVec, CheckedSub, ReadableCloneableVec, ReadableVec, TypedVec,
    VecIndex,
};

use super::Vecs;
use crate::{DCA_AMOUNT, by_class, cached_dca_sats::CachedDcaSats};
use crate::{
    class_vecs::ClassVecs, dca_stack::DcaStack, lump_sum_stack::LumpSumStack,
    period_vecs::PeriodVecs,
};
use bitview_compute::{
    ByDcaCagr, ByDcaPeriod, CentsUnsignedToDollars, Identity, LazyIndexedVec, LazyPerBlock,
    LazyPercentPerBlock, LazyPreviousDeltaVec, LazySinceDayVec, LazyWindowVec, Price,
    RatioDiffCents, SatsToBitcoin, SatsToCents,
};

impl Vecs {
    pub fn forced_import(
        parent_version: Version,
        indexes: &bitview_plugin_indexes::Vecs,
        blocks: &bitview_plugin_blocks::Vecs,
        prices: &bitview_plugin_price::Vecs,
    ) -> Result<Self> {
        let version = parent_version;

        let cached_days = indexes.height.day1_cached_boxed_clone();
        let cached_dca_sats = CachedDcaSats::new(
            prices.split.close.usd.day1.read_only_boxed_clone(),
            cached_days.clone(),
        );
        let sats_cumulative = cached_dca_sats.read_only_boxed_clone();
        let sats_per_day =
            LazyPreviousDeltaVec::new("dca_sats_per_day", version, sats_cumulative.clone());

        let cached_starts = ByDcaPeriod::try_new(|_, days| {
            Ok::<_, Error>(
                blocks
                    .lookback
                    .cached_start_vec(days as usize)
                    .read_only_cached_boxed_clone(),
            )
        })?;
        let spot_price = prices.spot.cents.height.read_only_cached_boxed_clone();

        let dca_stack =
            ByDcaPeriod::try_from_period(&cached_starts, |name, _days, window_starts| {
                let metric_name = format!("dca_stack_{name}");
                let source = LazyWindowVec::<Height, Sats, Sats>::new(
                    &format!("{metric_name}_sats_source"),
                    version,
                    sats_cumulative.clone(),
                    window_starts.clone(),
                    true,
                    |current, before, _| current.checked_sub(before).unwrap_or_default(),
                );
                dca_stack_from_source(&metric_name, version, indexes, source, &spot_price)
            })?;

        let first_price_day = Day1::try_from(Date::new(2010, 7, 12)).unwrap();
        let dca_cost_basis = ByDcaPeriod::try_from_period(&dca_stack, |name, days, stack| {
            let metric_name = format!("dca_cost_basis_{name}");
            let source = LazyIndexedVec::new(
                &format!("{metric_name}_cents_source"),
                version,
                stack.sats.height.read_only_boxed_clone(),
                cached_days.clone(),
                move |_, stack_sats, day| {
                    if day <= first_price_day {
                        return Cents::ZERO;
                    }
                    let num_days =
                        (days as usize).min(day.to_usize() + 1 - first_price_day.to_usize());
                    Cents::from(DCA_AMOUNT * num_days / Bitcoin::from(stack_sats))
                },
            );
            Ok::<_, Error>(Price::from_height_source(
                &metric_name,
                version,
                source,
                indexes,
            ))
        })?;

        let dca_return =
            ByDcaPeriod::try_from_period(&dca_cost_basis, |name, _days, cost_basis| {
                let metric_name = format!("dca_return_{name}");
                let source = LazyIndexedVec::new(
                    &format!("{metric_name}_ppm_source"),
                    version,
                    cost_basis.cents.height.read_only_boxed_clone(),
                    spot_price.clone(),
                    |_, cost_basis, spot| {
                        RatioDiffCents::<PartsPerMillionSigned64>::apply(spot, cost_basis)
                    },
                );
                Ok::<_, Error>(LazyPercentPerBlock::from_height_source(
                    &metric_name,
                    version,
                    source,
                    indexes,
                ))
            })?;

        let dca_cagr = ByDcaCagr::try_new(&dca_return, |name, days, source| {
            Ok::<_, Error>(LazyPercentPerBlock::from_lazy_cagr(
                &format!("dca_cagr_{name}"),
                version,
                (days / 365) as u8,
                source,
            ))
        })?;

        let lump_sum_stack =
            ByDcaPeriod::try_from_period(&cached_starts, |name, days, window_starts| {
                lump_sum_stack(
                    &format!("lump_sum_stack_{name}"),
                    days,
                    version,
                    indexes,
                    window_starts,
                    prices,
                )
            })?;

        let lump_sum_return =
            ByDcaPeriod::try_from_period(&cached_starts, |name, _days, window_starts| {
                let metric_name = format!("lump_sum_return_{name}");
                let source = LazyWindowVec::<Height, Cents, PartsPerMillionSigned64>::new(
                    &format!("{metric_name}_ppm_source"),
                    version,
                    prices.spot.cents.height.read_only_boxed_clone(),
                    window_starts.clone(),
                    false,
                    |current, past, _| {
                        RatioDiffCents::<PartsPerMillionSigned64>::apply(current, past)
                    },
                );
                Ok::<_, Error>(LazyPercentPerBlock::from_height_source(
                    &metric_name,
                    version,
                    source,
                    indexes,
                ))
            })?;

        let class_stack = by_class::try_new(|name, _year, day| {
            let metric_name = format!("dca_stack_{name}");
            let source = LazySinceDayVec::new(
                &format!("{metric_name}_sats_source"),
                version,
                sats_cumulative.clone(),
                cached_days.clone(),
                day,
                |current, before| current.checked_sub(before).unwrap_or_default(),
            );
            dca_stack_from_source(&metric_name, version, indexes, source, &spot_price)
        })?;

        let class_cost_basis =
            by_class::try_from_class(&class_stack, |name, _year, from, stack| {
                let metric_name = format!("dca_cost_basis_{name}");
                let source = LazyIndexedVec::new(
                    &format!("{metric_name}_cents_source"),
                    version,
                    stack.sats.height.read_only_boxed_clone(),
                    cached_days.clone(),
                    move |_, stack_sats, day| {
                        if day < from {
                            return Cents::ZERO;
                        }
                        let num_days = day.to_usize() + 1 - from.to_usize();
                        Cents::from(DCA_AMOUNT * num_days / Bitcoin::from(stack_sats))
                    },
                );
                Ok::<_, Error>(Price::from_height_source(
                    &metric_name,
                    version,
                    source,
                    indexes,
                ))
            })?;

        let class_return =
            by_class::try_from_class(&class_cost_basis, |name, _year, _from, cost_basis| {
                let metric_name = format!("dca_return_{name}");
                let source = LazyIndexedVec::new(
                    &format!("{metric_name}_ppm_source"),
                    version,
                    cost_basis.cents.height.read_only_boxed_clone(),
                    spot_price.clone(),
                    |_, cost_basis, spot| {
                        RatioDiffCents::<PartsPerMillionSigned64>::apply(spot, cost_basis)
                    },
                );
                Ok::<_, Error>(LazyPercentPerBlock::from_height_source(
                    &metric_name,
                    version,
                    source,
                    indexes,
                ))
            })?;

        Ok(Self {
            plugin_gate: Default::default(),
            cached_dca_sats,
            sats_per_day,
            period: PeriodVecs {
                dca_stack,
                dca_cost_basis,
                dca_return,
                dca_cagr,
                lump_sum_stack,
                lump_sum_return,
            },
            class: ClassVecs {
                dca_stack: class_stack,
                dca_cost_basis: class_cost_basis,
                dca_return: class_return,
            },
        })
    }
}

fn dca_stack_from_source<V>(
    name: &str,
    version: Version,
    indexes: &bitview_plugin_indexes::Vecs,
    source: V,
    spot_price: &CachedBoxedVec<Height, Cents>,
) -> Result<DcaStack>
where
    V: TypedVec<I = Height, T = Sats> + ReadableVec<Height, Sats> + Clone + 'static,
{
    let sats = LazyPerBlock::from_height_source::<Identity<Sats>>(
        &format!("{name}_sats"),
        version,
        source,
        indexes,
    );
    let btc = LazyPerBlock::from_lazy::<SatsToBitcoin, Sats>(name, version, &sats);
    let cents_source = LazyIndexedVec::new(
        &format!("{name}_cents_source"),
        version,
        sats.height.read_only_boxed_clone(),
        spot_price.clone(),
        |_, sats, spot| SatsToCents::apply(sats, spot),
    );
    let cents = LazyPerBlock::from_height_source::<Identity<Cents>>(
        &format!("{name}_cents"),
        version,
        cents_source,
        indexes,
    );
    let usd = LazyPerBlock::from_lazy::<CentsUnsignedToDollars, Cents>(
        &format!("{name}_usd"),
        version,
        &cents,
    );
    Ok(DcaStack {
        btc,
        sats,
        usd,
        cents,
    })
}

fn lump_sum_stack(
    name: &str,
    days: u32,
    version: Version,
    indexes: &bitview_plugin_indexes::Vecs,
    window_starts: &CachedBoxedVec<Height, Height>,
    prices: &bitview_plugin_price::Vecs,
) -> Result<LumpSumStack> {
    let total_invested = DCA_AMOUNT * days as usize;

    let sats_source = LazyWindowVec::<Height, Cents, Sats>::new(
        &format!("{name}_sats_source"),
        version,
        prices.spot.cents.height.read_only_boxed_clone(),
        window_starts.clone(),
        false,
        move |_, past, _| lump_sum_sats(total_invested, past),
    );
    let sats = LazyPerBlock::from_height_source::<Identity<Sats>>(
        &format!("{name}_sats"),
        version,
        sats_source,
        indexes,
    );
    let btc = LazyPerBlock::from_lazy::<SatsToBitcoin, Sats>(name, version, &sats);

    let cents_source = LazyWindowVec::<Height, Cents, Cents>::new(
        &format!("{name}_cents_source"),
        version,
        prices.spot.cents.height.read_only_boxed_clone(),
        window_starts.clone(),
        false,
        move |current, past, _| SatsToCents::apply(lump_sum_sats(total_invested, past), current),
    );
    let cents = LazyPerBlock::from_height_source::<Identity<Cents>>(
        &format!("{name}_cents"),
        version,
        cents_source,
        indexes,
    );
    let usd = LazyPerBlock::from_lazy::<CentsUnsignedToDollars, Cents>(
        &format!("{name}_usd"),
        version,
        &cents,
    );

    Ok(LumpSumStack {
        btc,
        sats,
        usd,
        cents,
    })
}

fn lump_sum_sats(total_invested: Dollars, past_price: Cents) -> Sats {
    if past_price == Cents::ZERO {
        Sats::ZERO
    } else {
        Sats::from(Bitcoin::from(total_invested / Dollars::from(past_price)))
    }
}
