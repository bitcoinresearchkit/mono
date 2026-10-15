use bitview_primitives::TypeIndex;
use brk_types::{OutputType, Sats};
use rustc_hash::FxHashSet;

#[derive(Default)]
pub struct Candidate {
    values: FxHashSet<Sats>,
    addresses: FxHashSet<(OutputType, TypeIndex)>,
    remaining_values: usize,
}

impl Candidate {
    pub fn clear(&mut self, value_limit: usize) {
        self.values.clear();
        self.addresses.clear();
        self.remaining_values = value_limit;
    }

    /// Stop at the first rejection: more entries cannot undo address reuse or
    /// bring the distinct-value count back below the candidate's limit.
    pub fn add(&mut self, value: Sats, output_type: OutputType, type_index: TypeIndex) -> bool {
        if output_type.is_addr() && !self.addresses.insert((output_type, type_index)) {
            return false;
        }
        // Each zero counts separately, even when other values are repeated.
        if value.is_zero() || self.values.insert(value) {
            if self.remaining_values == 0 {
                return false;
            }
            self.remaining_values -= 1;
        }
        true
    }
}
