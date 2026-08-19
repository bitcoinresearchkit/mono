use brk_types::{Cents, Height, RangeMap, Timestamp, TxIndex};
use vecdb::Database;

use super::{compute::PriceRangeMax, state::BlockState};

/// Private storage and transient computation state for distribution.
#[derive(Clone)]
pub struct Inner {
    pub db: Database,
    pub chain_state: Vec<BlockState>,
    pub tx_index_to_height: RangeMap<TxIndex, Height>,
    pub prices: Vec<Cents>,
    pub timestamps: Vec<Timestamp>,
    pub price_range_max: PriceRangeMax,
}

impl Inner {
    pub fn new(db: Database) -> Self {
        Self {
            db,
            chain_state: Vec::new(),
            tx_index_to_height: RangeMap::default(),
            prices: Vec::new(),
            timestamps: Vec::new(),
            price_range_max: PriceRangeMax::default(),
        }
    }

    pub fn reset(&mut self) {
        self.chain_state = Vec::new();
        self.tx_index_to_height = RangeMap::default();
        self.prices = Vec::new();
        self.timestamps = Vec::new();
        self.price_range_max = PriceRangeMax::default();
    }
}
