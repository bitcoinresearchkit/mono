use super::{ActivityVecs, OutputsVecs, RealizedVecs, SupplyVecs};
use crate::state::UTXOStates;
use bitview_cohort::UtxoGroups;
use bitview_cohort::{AmountRange, SpendableType};
use bitview_collections::Windows;
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_transforms::SatsToCents;
use bitview_traversable::Traversable;
use bitview_vecs::LazyWindowStartVec;
use brk_error::Result;
use brk_types::{Cents, Height, Sats, Version};
use rayon::prelude::*;
use vecdb::BinaryTransform;
use vecdb::{AnyStoredVec, Database, ReadableBoxedVec, Rw, StorageMode};
#[derive(Traversable)]
pub struct CohortMetrics<M: StorageMode = Rw> {
    pub supply: Box<SupplyVecs<M>>,
    pub outputs: Box<OutputsVecs<M>>,
    pub activity: Box<ActivityVecs<M>>,
    pub realized: Box<RealizedVecs<M>>,
}
impl CohortMetrics {
    pub fn forced_import(
        db: &Database,
        version: Version,
        mappings: &MappingsVecs,
        windows: &Windows<&LazyWindowStartVec>,
        spot: &ReadableBoxedVec<Height, Cents>,
        all_supply: &ReadableBoxedVec<Height, Sats>,
    ) -> Result<Self> {
        Ok(Self {
            supply: SupplyVecs::forced_import(db, version, mappings, windows, spot, all_supply)?,
            outputs: OutputsVecs::forced_import(db, version, mappings, windows, spot)?,
            activity: ActivityVecs::forced_import(db, version, mappings, windows)?,
            realized: RealizedVecs::forced_import(db, version, mappings, windows)?,
        })
    }
    pub fn push(&mut self, states: &UTXOStates, price: Cents) {
        let supply = UtxoGroups {
            utxo_amount: AmountRange::from_fn(|id| id.select(&states.amount_range).supply_value()),
            type_: SpendableType::from_fn(|id| id.select(&states.type_).supply_value()),
        };
        let counts = UtxoGroups {
            utxo_amount: AmountRange::from_fn(|id| id.select(&states.amount_range).output_counts()),
            type_: SpendableType::from_fn(|id| id.select(&states.type_).output_counts()),
        };
        let sent = UtxoGroups {
            utxo_amount: AmountRange::from_fn(|id| {
                id.select(&states.amount_range).transfer_volume()
            }),
            type_: SpendableType::from_fn(|id| id.select(&states.type_).transfer_volume()),
        };
        let realized = UtxoGroups {
            utxo_amount: AmountRange::from_fn(|id| {
                id.select(&states.amount_range).realized_block_data()
            }),
            type_: SpendableType::from_fn(|id| id.select(&states.type_).realized_block_data()),
        };
        self.outputs.avg_amount.push(&supply.type_, &counts.type_);
        self.supply.total.stored.push(&supply);
        self.outputs.push(counts.map(|v| v.0), counts.map(|v| v.1));
        self.activity
            .transfer_volume
            .push_block(&sent, &sent.map(|v| SatsToCents::apply(*v, price)));
        self.realized.cap.stored.push(&realized.map(|v| v.cap));
        self.realized
            .price
            .stored
            .push(&realized.map(|v| v.price()));
        self.realized
            .profit
            .stored
            .push_block(realized.map(|v| v.profit));
        self.realized
            .loss
            .stored
            .push_block(realized.map(|v| v.loss));
    }

    pub fn par_iter_vecs_mut(&mut self) -> impl ParallelIterator<Item = &mut dyn AnyStoredVec> {
        self.supply
            .stored_vecs_mut()
            .chain(self.outputs.stored_vecs_mut())
            .chain(self.activity.stored_vecs_mut())
            .chain(self.realized.stored_vecs_mut())
            .collect::<Vec<_>>()
            .into_par_iter()
    }
}
