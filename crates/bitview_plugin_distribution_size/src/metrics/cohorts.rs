use super::{ActivityVecs, OutputsVecs, RealizedVecs, SupplyVecs};
use crate::{
    state::{AddrCohortState, RealizedOps, UTXOStates},
    values::SizeValues,
};
use bitview_cohort::{AmountRange, SpendableType};
use bitview_collections::Windows;
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_transforms::SatsToCents;
use bitview_traversable::Traversable;
use bitview_vecs::LazyWindowStartVec;
use brk_error::Result;
use brk_types::{Cents, Height, Sats, StoredU64, Version};
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
            outputs: OutputsVecs::forced_import(db, version, mappings, windows)?,
            activity: ActivityVecs::forced_import(db, version, mappings, windows)?,
            realized: RealizedVecs::forced_import(db, version, mappings, windows, spot)?,
        })
    }
    pub fn push(&mut self, states: &UTXOStates, price: Cents) {
        let supply = SizeValues {
            amount_range: AmountRange::from_fn(|id| id.select(&states.amount_range).supply_value()),
            type_: SpendableType::from_fn(|id| id.select(&states.type_).supply_value()),
        };
        let counts = SizeValues {
            amount_range: AmountRange::from_fn(|id| {
                id.select(&states.amount_range).output_counts()
            }),
            type_: SpendableType::from_fn(|id| id.select(&states.type_).output_counts()),
        };
        let sent = SizeValues {
            amount_range: AmountRange::from_fn(|id| {
                id.select(&states.amount_range).transfer_volume()
            }),
            type_: SpendableType::from_fn(|id| id.select(&states.type_).transfer_volume()),
        };
        let realized = SizeValues {
            amount_range: AmountRange::from_fn(|id| {
                id.select(&states.amount_range).realized_block_data()
            }),
            type_: SpendableType::from_fn(|id| id.select(&states.type_).realized_block_data()),
        };
        self.supply.total.stored.push(supply);
        self.outputs.push(counts.map(|v| v.0), counts.map(|v| v.1));
        self.activity
            .transfer_volume
            .push_block(sent.clone(), sent.map(|v| SatsToCents::apply(*v, price)));
        self.realized.cap.stored.push(realized.map(|v| v.cap));
        self.realized.price.stored.push(realized.map(|v| v.price));
        self.realized
            .profit
            .stored
            .push_block(realized.map(|v| v.profit));
        self.realized
            .loss
            .stored
            .push_block(realized.map(|v| v.loss));
    }
    pub fn push_addr_balance(
        &mut self,
        states: &AmountRange<AddrCohortState>,
        height_price: Cents,
    ) {
        let supply = AmountRange::from_fn(|amount| amount.select(states).inner.supply.value);
        let output_count = AmountRange::from_fn(|amount| {
            StoredU64::from(amount.select(states).inner.supply.utxo_count)
        });
        let transfer_volume = AmountRange::from_fn(|amount| amount.select(states).inner.sent);
        let realized_cap =
            AmountRange::from_fn(|amount| amount.select(states).inner.realized.cap());
        let realized_profit =
            AmountRange::from_fn(|amount| amount.select(states).inner.realized.profit());
        let realized_loss =
            AmountRange::from_fn(|amount| amount.select(states).inner.realized.loss());

        self.supply.total.push_addr_balance(supply);
        self.outputs.push_addr_balance(output_count);
        self.activity
            .push_addr_balance(height_price, &transfer_volume);
        self.realized
            .push_addr_balance(realized_cap, &realized_profit, &realized_loss);
    }
    pub fn min_resume_len(&self) -> Height {
        Height::from(
            self.supply
                .min_resume_len()
                .min(self.outputs.min_resume_len())
                .min(self.activity.min_resume_len())
                .min(self.realized.min_resume_len()),
        )
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
