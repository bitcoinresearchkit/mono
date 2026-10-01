use std::array;

use bitview_compute::prepare_computed;
use bitview_plugin_distribution_age::RealizedTotals;
use bitview_traversable::Traversable;
use bitview_vecs::IndexSources;
use brk_error::Result;
use brk_exit::Exit;
use brk_types::{Cents, CentsSats, CentsSquaredSats, Height, Sats, Version};
use vecdb::{AnyStoredVec, Database, ReadableVec, Rw, StorageMode, WritableVec};

use crate::{COMPUTE_BATCH_SIZE, reference_price::ReferencePrice};

#[cfg(test)]
#[path = "reference_prices_tests.rs"]
mod tests;

/// Age-threshold prices reconstructed from raw totals before rounding once to cents.
#[derive(Traversable)]
pub struct ReferencePrices<M: StorageMode = Rw> {
    /// Realized price of UTXOs younger than 120 days.
    pub under_4m: ReferencePrice<M>,
    /// Realized price of UTXOs younger than 180 days.
    pub under_6m: ReferencePrice<M>,
    /// Realized price of UTXOs at least 120 days old.
    pub over_4m: ReferencePrice<M>,
    /// Realized price of UTXOs at least 180 days old.
    pub over_6m: ReferencePrice<M>,
    /// Capitalized price of UTXOs younger than 120 days.
    pub under_4m_capitalized_price: ReferencePrice<M>,
    /// Capitalized price of UTXOs younger than 180 days.
    pub under_6m_capitalized_price: ReferencePrice<M>,
}

impl ReferencePrices {
    pub fn forced_import(db: &Database, version: Version, indexes: &IndexSources) -> Result<Self> {
        let import = |name| {
            ReferencePrice::forced_import(
                db,
                &format!("rarity_meter_{name}"),
                version + Version::ONE,
                indexes,
            )
        };
        Ok(Self {
            under_4m: import("under_4m_realized_price")?,
            under_6m: import("under_6m_realized_price")?,
            over_4m: import("over_4m_realized_price")?,
            over_6m: import("over_6m_realized_price")?,
            under_4m_capitalized_price: import("under_4m_capitalized_price")?,
            under_6m_capitalized_price: import("under_6m_capitalized_price")?,
        })
    }

    /// Cap and supply inputs are STH, LTH, 4–5m, and 5–6m, in that order.
    /// Capitalized-cap inputs are STH, 4–5m, and 5–6m.
    pub fn compute(
        &mut self,
        starting_height: Height,
        cap_raw: [&impl ReadableVec<Height, CentsSats>; 4],
        supply: [&impl ReadableVec<Height, Sats>; 4],
        capitalized_cap_raw: [&impl ReadableVec<Height, CentsSquaredSats>; 3],
        spot: &impl ReadableVec<Height, Cents>,
        exit: &Exit,
    ) -> Result<()> {
        let mut source_end = usize::MAX;
        let mut version = Version::ZERO;
        for (cap, supply) in cap_raw.iter().zip(supply.iter()) {
            source_end = source_end.min(cap.len()).min(supply.len());
            version = version.combine(cap.version()).combine(supply.version());
        }
        for cap in capitalized_cap_raw {
            source_end = source_end.min(cap.len());
            version = version.combine(cap.version());
        }
        let mut targets = [
            &mut self.under_4m.cents.height,
            &mut self.under_6m.cents.height,
            &mut self.over_4m.cents.height,
            &mut self.over_6m.cents.height,
            &mut self.under_4m_capitalized_price.cents.height,
            &mut self.under_6m_capitalized_price.cents.height,
        ];
        let mut start = prepare_computed(
            &mut targets,
            version,
            usize::from(starting_height).min(source_end),
            exit,
        )?;

        while start < source_end {
            let end = (start + COMPUTE_BATCH_SIZE).min(source_end);
            let caps = cap_raw.map(|source| source.collect_range_at(start, end));
            let supplies = supply.map(|source| source.collect_range_at(start, end));
            let capitalized_caps =
                capitalized_cap_raw.map(|source| source.collect_range_at(start, end));
            for row in 0..end - start {
                let [sth, lth, band_4m_to_5m, band_5m_to_6m] = array::from_fn(|i| RealizedTotals {
                    cap_raw: caps[i][row],
                    supply: supplies[i][row],
                });
                // STH/LTH split at 5m. Adjust their exact totals before dividing.
                let values = [
                    sth - band_4m_to_5m,
                    sth + band_5m_to_6m,
                    lth + band_4m_to_5m,
                    lth - band_5m_to_6m,
                ];
                let [sth, band_4m_to_5m, band_5m_to_6m] =
                    array::from_fn(|i| capitalized_caps[i][row]);
                let capitalized_prices = [sth - band_4m_to_5m, sth + band_5m_to_6m]
                    .into_iter()
                    .zip(&values)
                    .map(|(cap, totals)| {
                        Cents::new(
                            cap.inner()
                                .checked_div(totals.cap_raw.as_u128())
                                .unwrap_or(0) as u64,
                        )
                    });
                for (target, price) in targets.iter_mut().zip(
                    values
                        .iter()
                        .map(|value| value.price())
                        .chain(capitalized_prices),
                ) {
                    target.push(price);
                }
            }
            let _lock = exit.lock();
            for target in &mut targets {
                target.write()?;
            }
            start = end;
        }
        for price in [
            &mut self.under_4m,
            &mut self.under_6m,
            &mut self.over_4m,
            &mut self.over_6m,
            &mut self.under_4m_capitalized_price,
            &mut self.under_6m_capitalized_price,
        ] {
            price.compute_ratio(starting_height, spot, exit)?;
        }
        Ok(())
    }
}
