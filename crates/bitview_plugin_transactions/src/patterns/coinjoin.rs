use brk_types::{OutputType, Sats, TypeIndex};
use rustc_hash::FxHashSet;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
struct Address(OutputType, TypeIndex);

#[derive(Default)]
pub struct Candidate {
    values: FxHashSet<Sats>,
    input_addresses: FxHashSet<Address>,
    output_addresses: FxHashSet<Address>,
    zero_values: usize,
    address_reuse: bool,
}

impl Candidate {
    pub fn clear(&mut self) {
        self.values.clear();
        self.input_addresses.clear();
        self.output_addresses.clear();
        self.zero_values = 0;
        self.address_reuse = false;
    }

    pub fn add_input(&mut self, value: Sats, output_type: OutputType, type_index: TypeIndex) {
        self.add_value(value);
        if has_script_address(output_type) {
            self.address_reuse |= !self
                .input_addresses
                .insert(Address(output_type, type_index));
        }
    }

    pub fn add_output(&mut self, value: Sats, output_type: OutputType, type_index: TypeIndex) {
        self.add_value(value);
        if has_script_address(output_type) {
            let address = Address(output_type, type_index);
            self.address_reuse |=
                self.input_addresses.contains(&address) || !self.output_addresses.insert(address);
        }
    }

    pub fn is_match(&self, input_count: usize, output_count: usize) -> bool {
        !self.address_reuse
            && self.values.len() + self.zero_values <= (input_count + output_count) / 2
    }

    fn add_value(&mut self, value: Sats) {
        if value.is_zero() {
            self.zero_values += 1;
        } else {
            self.values.insert(value);
        }
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

#[cfg(test)]
mod tests {
    use brk_types::{OutputType, Sats, TypeIndex};

    use super::Candidate;

    #[test]
    fn repeated_values_match_without_address_reuse() {
        let mut candidate = Candidate::default();
        for index in 0usize..5 {
            candidate.add_input(
                Sats::new(10_000),
                OutputType::P2WPKH,
                TypeIndex::from(index),
            );
            candidate.add_output(
                Sats::new(9_000),
                OutputType::P2WPKH,
                TypeIndex::from(index + 10),
            );
        }
        assert!(candidate.is_match(5, 5));
    }

    #[test]
    fn reused_output_address_does_not_match() {
        let mut candidate = Candidate::default();
        for _ in 0..5 {
            candidate.add_output(
                Sats::new(9_000),
                OutputType::P2WPKH,
                TypeIndex::from(1usize),
            );
        }
        assert!(!candidate.is_match(5, 5));
    }

    #[test]
    fn reused_input_address_does_not_match() {
        let mut candidate = Candidate::default();
        for _ in 0..5 {
            candidate.add_input(
                Sats::new(10_000),
                OutputType::P2PK33,
                TypeIndex::from(1usize),
            );
        }
        assert!(!candidate.is_match(5, 5));
    }
}
