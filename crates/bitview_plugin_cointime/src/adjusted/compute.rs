use bitview_distribution::AllChainSources;
use bitview_plugin_indexer::Indexer;
use bitview_primitives::{BoundedRatio, PartsPerMillionSigned32, PartsPerMillionSigned64, Ratio};
use bitview_vecs::{LazyRollingSumAmountFromHeight, ops::BoundedOdds};
use brk_error::Result;
use brk_exit::Exit;
use brk_types::Height;
use vecdb::{ComputeVec, ReadableVec, UnaryTransform};

use super::{super::activity, Vecs};

impl Vecs {
    pub(crate) fn compute(
        &mut self,
        indexer: &Indexer,
        inflation_rate: &impl ReadableVec<Height, PartsPerMillionSigned64>,
        volume: &LazyRollingSumAmountFromHeight,
        all_chain: &AllChainSources,
        activity: &activity::Vecs,
        exit: &Exit,
    ) -> Result<()> {
        let starting_height = indexer.compute_start().height;

        self.inflation_rate.fixed.height.compute_transform2(
            starting_height,
            &activity.liveliness_source.height,
            inflation_rate,
            |(_, liveliness, inflation, ..)| {
                let active_to_vaulted: f64 =
                    <BoundedOdds as UnaryTransform<BoundedRatio, f64>>::apply(liveliness);
                PartsPerMillionSigned32::from(active_to_vaulted * f64::from(inflation))
            },
            exit,
        )?;

        // Velocity of the active supply: the year's volume over supply times liveliness, at f64.
        let liveliness = &activity.liveliness_source.height;
        self.velocity.btc.height.compute_transform3(
            starting_height,
            &volume.sats.height,
            &all_chain.supply,
            liveliness,
            |(_, volume, supply, liveliness, ..)| {
                Ratio::from(f64::from(volume) / (f64::from(supply) * f64::from(liveliness)))
            },
            exit,
        )?;
        self.velocity.usd.height.compute_transform3(
            starting_height,
            &volume.cents.height,
            &all_chain.market_cap,
            liveliness,
            |(_, volume, market_cap, liveliness, ..)| {
                Ratio::from(f64::from(volume) / (f64::from(market_cap) * f64::from(liveliness)))
            },
            exit,
        )?;

        Ok(())
    }
}
