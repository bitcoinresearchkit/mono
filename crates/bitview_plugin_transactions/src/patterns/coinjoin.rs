use bitview_primitives::TypeIndex;
use brk_types::{OutputType, Sats};
use rustc_hash::FxHashSet;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
struct Address(OutputType, TypeIndex);

#[derive(Default)]
pub struct Candidate {
    values: FxHashSet<Sats>,
    addresses: FxHashSet<Address>,
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
        if has_script_address(output_type)
            && !self.addresses.insert(Address(output_type, type_index))
        {
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

fn has_script_address(output_type: OutputType) -> bool {
    matches!(
        output_type,
        OutputType::P2PK65
            | OutputType::P2PK33
            | OutputType::P2PKH
            | OutputType::P2SH
            | OutputType::P2WPKH
            | OutputType::P2WSH
            | OutputType::P2TR
            | OutputType::P2A
    )
}
