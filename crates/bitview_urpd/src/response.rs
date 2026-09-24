use brk_types::{
    Bitcoin, Cents, CentsCompact, CentsSats, CentsSigned, Cohort, Date, Dollars, Sats, Urpd,
    UrpdAggregation, UrpdBucket, UrpdWeight,
};

/// Build response buckets directly from validated, sorted source entries.
pub fn build_response(
    cohort: Cohort,
    date: Date,
    weight: UrpdWeight,
    close: Cents,
    entries: impl IntoIterator<Item = (CentsCompact, Sats)>,
    aggregation: UrpdAggregation,
) -> Urpd {
    let mut buckets = Vec::new();
    let mut total = Sats::ZERO;
    let mut current: Option<(Cents, Sats, CentsSats)> = None;
    let finish = |(price, supply, capital): (Cents, Sats, CentsSats)| {
        let realized_cap = capital.to_cents();
        let market_cap = CentsSats::from_price_sats(close, supply).to_cents();
        UrpdBucket {
            price_floor: Dollars::from(price),
            supply: Bitcoin::from(supply),
            realized_cap: Dollars::from(realized_cap),
            unrealized_pnl: Dollars::from(
                CentsSigned::from(market_cap.inner()) - CentsSigned::from(realized_cap.inner()),
            ),
        }
    };
    for (price, supply) in entries {
        let price = Cents::from(price);
        let floor = aggregation.bucket_floor(price);
        let capital = CentsSats::from_price_sats(price, supply);
        total += supply;
        if let Some((last, sats, cap)) = &mut current
            && *last == floor
        {
            *sats += supply;
            *cap += capital;
        } else if let Some(previous) = current.replace((floor, supply, capital)) {
            buckets.push(finish(previous));
        }
    }
    if let Some(last) = current {
        buckets.push(finish(last));
    }
    Urpd {
        cohort,
        date,
        weight,
        aggregation,
        close: Dollars::from(close),
        total_supply: Bitcoin::from(total),
        buckets,
    }
}
