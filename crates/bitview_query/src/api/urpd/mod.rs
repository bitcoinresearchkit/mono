use std::sync::Arc;

use bitview_cohort::{AgeRange, AgeRangeId};
use bitview_plugin_indexer::SafeLengths;
use bitview_primitives::{Date, Day1};
use bitview_types::{Urpd, UrpdAggregation, UrpdCohort, UrpdWeight};
use bitview_urpd::OriginUrpd;
use brk_types::{BlockHash, Cents, Height, Sats, Version};
use vecdb::{AnyVec, ReadableVec};

use crate::{Error, OptionData, Query, ReadPin, Result, check_bound};

mod entries;

use entries::UrpdHeader;

/// The price × age grid of the block a distribution was asked for last, which every cohort,
/// aggregation and weight of that block projects.
pub(crate) struct UrpdGrid {
    hash: BlockHash,
    /// The summed versions of its sources.
    version: Version,
    source: OriginUrpd,
    date: Date,
    close: Cents,
}

/// A URPD request resolved under one pin to the block it answers for: its identity, without
/// the distribution, which [`Query::urpd_resolved`] projects from its block's grid under the
/// same pin.
pub struct ResolvedUrpd {
    pin: SafeLengths,
    cohort: UrpdCohort,
    ages: &'static [AgeRangeId],
    height: Height,
    hash: BlockHash,
    aggregation: UrpdAggregation,
    weight: UrpdWeight,
    /// The summed versions of the sources the distribution reads.
    version: Version,
    settled: bool,
}

impl ResolvedUrpd {
    /// Identifies the response: the request, the block (its hash commits to the chain the
    /// distribution reconstructs) and the versions of the sources it reads.
    pub fn representation(&self) -> String {
        format!(
            "{}-{}-{}-{}-{}",
            self.version, self.cohort, self.aggregation, self.weight, self.hash
        )
    }

    /// Whether the URL pins the point below the reorg window: appends never change it.
    pub fn is_settled(&self) -> bool {
        self.settled
    }
}

impl Query {
    /// Every cohort once a UTXO set is published (none before): the aggregates, then the age
    /// bands from the youngest.
    pub fn urpd_cohorts(&self) -> Result<Vec<UrpdCohort>> {
        let pin = self.pin_safe_lengths()?;
        let view = self.plugins().utxo_set.view()?;
        let published = usize::from(pin.lengths().height);
        if published == 0 || view.reader(published)?.is_empty() {
            return Ok(Vec::new());
        }
        Ok(UrpdCohort::all().collect())
    }

    pub fn resolve_urpd_at(
        &self,
        cohort: UrpdCohort,
        date: Date,
        aggregation: UrpdAggregation,
        weight: UrpdWeight,
    ) -> Result<ResolvedUrpd> {
        let pin = self.pin_safe_lengths()?;
        let end = usize::from(pin.lengths().height);
        let day = Day1::try_from(date)?;
        let (heights, settled) = self.day_heights(day, end, &pin)?;
        let height = Height::from(heights.end - 1);
        self.resolve_urpd_inner(cohort, height, aggregation, weight, pin, settled)
    }

    pub fn resolve_urpd_height(
        &self,
        cohort: UrpdCohort,
        height: Height,
        aggregation: UrpdAggregation,
        weight: UrpdWeight,
    ) -> Result<ResolvedUrpd> {
        let pin = self.pin_safe_lengths()?;
        let settled = pin.lengths().is_settled(height);
        self.resolve_urpd_inner(cohort, height, aggregation, weight, pin, settled)
    }

    pub fn resolve_urpd_latest(
        &self,
        cohort: UrpdCohort,
        aggregation: UrpdAggregation,
        weight: UrpdWeight,
    ) -> Result<ResolvedUrpd> {
        let pin = self.pin_safe_lengths()?;
        let height = pin
            .lengths()
            .last_height()
            .ok_or_else(|| Error::NotFound("No published UTXO set".into()))?;
        self.resolve_urpd_inner(cohort, height, aggregation, weight, pin, false)
    }

    fn urpd_weights(
        &self,
        ages: &[AgeRangeId],
        height: Height,
        weight: UrpdWeight,
    ) -> Result<AgeRange<f64>> {
        AgeRange::try_from_fn(|age| {
            if weight == UrpdWeight::None || !ages.contains(&age) {
                return Ok(1.0);
            }
            let plugins = self.plugins();
            let supply = age
                .select(&plugins.age.ranges)
                .supply
                .total
                .value
                .sats
                .height
                .collect_one(height)
                .data()?;
            match weight {
                UrpdWeight::None => Some(1.0),
                UrpdWeight::Wakefulness => plugins.cointime.urpd_weight(age, height, supply),
                UrpdWeight::Mobility => plugins.coinflow.urpd_weight(age, height, supply),
            }
            .ok_or_else(|| Error::NotFound(format!("No {weight} weight at block {height}")))
        })
    }

    fn resolve_urpd_inner(
        &self,
        cohort: UrpdCohort,
        height: Height,
        aggregation: UrpdAggregation,
        weight: UrpdWeight,
        pin: ReadPin,
        settled: bool,
    ) -> Result<ResolvedUrpd> {
        let ages = cohort.age_range_ids();
        check_bound(&pin, height, pin.lengths().height)?;
        if height >= pin.lengths().height {
            return Err(Error::NotFound("Block is not published".into()));
        }
        let hash = self
            .indexer()
            .vecs()
            .blocks
            .blockhash
            .collect_one(height)
            .data()?;
        Ok(ResolvedUrpd {
            version: self.urpd_version(ages, weight),
            pin: pin.into_inner(),
            cohort,
            ages,
            height,
            hash,
            aggregation,
            weight,
            settled,
        })
    }

    /// The summed versions of the sources a block's grid reads: prices, timestamps and the UTXO
    /// set history.
    fn urpd_grid_version(&self) -> Version {
        let plugins = self.plugins();
        plugins.price.spot.cents.height.version()
            + plugins.mappings.timestamp.monotonic.version()
            + plugins.utxo_set.supply.version()
    }

    /// The summed versions of the sources a distribution reads: its grid's, and for a weighted one
    /// the cohort's supplies and weights.
    fn urpd_version(&self, ages: &[AgeRangeId], weight: UrpdWeight) -> Version {
        let plugins = self.plugins();
        let base = self.urpd_grid_version();
        let weights = |age: &AgeRangeId| match weight {
            UrpdWeight::None => Version::ZERO,
            UrpdWeight::Wakefulness => age
                .select(&plugins.cointime.age_ranges.urpd_weight_sources())
                .version(),
            UrpdWeight::Mobility => age
                .select(&plugins.coinflow.age_ranges.urpd_weight_sources())
                .version(),
        };
        if weight == UrpdWeight::None {
            return base;
        }
        base + ages
            .iter()
            .map(|age| {
                let supply = &age
                    .select(&plugins.age.ranges)
                    .supply
                    .total
                    .value
                    .sats
                    .height;
                supply.version() + weights(age)
            })
            .sum::<Version>()
    }

    /// Projects the distribution `resolved` selected from its block's grid, under its pin.
    pub fn urpd_resolved(&self, resolved: ResolvedUrpd) -> Result<Urpd> {
        let ResolvedUrpd {
            pin,
            cohort,
            ages,
            height,
            hash,
            aggregation,
            weight,
            ..
        } = resolved;
        let pin = self.read_pin(pin);
        let weights = self.urpd_weights(ages, height, weight)?;
        let version = self.urpd_grid_version();
        let last = self.0.urpd_grid.lock().clone();
        let grid = match last {
            Some(grid) if grid.hash == hash && grid.version == version => grid,
            // Built unlocked so different blocks build at once; concurrent misses on one block
            // each build it, as many as the route admits.
            _ => {
                let grid = Arc::new(self.urpd_grid(&pin, height, hash, version)?);
                *self.0.urpd_grid.lock() = Some(Arc::clone(&grid));
                grid
            }
        };
        let header = UrpdHeader {
            cohort,
            height,
            hash,
            date: grid.date,
            weight,
            aggregation,
            close: grid.close,
        };
        let source = &grid.source;
        Ok(if weight == UrpdWeight::None {
            header.build(source.project(&[], [ages]).map(|b| (b.price, b.raw[0])))
        } else {
            header.build(source.project(&[Some(&weights)], [ages]).filter_map(|b| {
                let sats = b.weighted[0][0];
                (sats != Sats::ZERO).then_some((b.price, sats))
            }))
        })
    }

    /// Reconstructs the grid of the block at `height`, which `hash` names, under `pin`.
    fn urpd_grid(
        &self,
        pin: &ReadPin,
        height: Height,
        hash: BlockHash,
        version: Version,
    ) -> Result<UrpdGrid> {
        let end = usize::from(height) + 1;
        let plugins = self.plugins();
        let view = plugins.utxo_set.view()?;
        let reader = view.reader(usize::from(pin.lengths().height))?;
        if end > reader.len() {
            return Err(Error::NotFound(
                "Block is outside the published UTXO set".into(),
            ));
        }
        let state = reader.state_at(end)?;
        if state.hash() != *hash {
            return Err(Error::Internal(
                "UTXO set history does not match the published chain",
            ));
        }
        let prices = plugins.price.spot.cents.height.collect_range_at(0, end);
        let timestamps = plugins
            .mappings
            .timestamp
            .monotonic
            .collect_range_at(0, end);
        Ok(UrpdGrid {
            hash,
            version,
            source: OriginUrpd::new(&state, &prices, &timestamps)?,
            date: Date::from(timestamps[end - 1]),
            close: prices[end - 1],
        })
    }
}
