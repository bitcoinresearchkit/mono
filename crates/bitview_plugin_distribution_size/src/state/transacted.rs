use std::ops::{Add, AddAssign};

use bitview_cohort::{AmountRange, ByType};
use brk_types::{OutputType, Sats, SupplyState};
use vecdb::unlikely;

#[derive(Default, Debug)]
pub struct Transacted {
    pub spendable_supply: SupplyState,
    pub by_type: ByType<SupplyState>,
    pub by_size_group: AmountRange<SupplyState>,
}

impl Transacted {
    #[allow(clippy::inconsistent_digit_grouping)]
    pub fn iterate(&mut self, value: Sats, _type: OutputType) {
        let supply = SupplyState {
            utxo_count: 1,
            value,
        };

        *self.by_type.get_mut(_type) += &supply;

        if unlikely(_type.is_unspendable()) {
            return;
        }

        self.spendable_supply += &supply;

        *self.by_size_group.get_mut(value) += &supply;
    }
}

impl Add for Transacted {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Self {
            spendable_supply: self.spendable_supply + rhs.spendable_supply,
            by_type: self.by_type + rhs.by_type,
            by_size_group: self.by_size_group + rhs.by_size_group,
        }
    }
}

impl AddAssign for Transacted {
    fn add_assign(&mut self, rhs: Self) {
        self.by_size_group += rhs.by_size_group;
        self.spendable_supply += &rhs.spendable_supply;
        self.by_type += rhs.by_type;
    }
}
