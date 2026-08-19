use brk_types::OutputType;

#[derive(Clone, Copy, Default)]
pub struct TxFeatureFlags(u32);

macro_rules! define_flags {
    ($($(#[$attribute:meta])* $vector:ident: $flag:ident = $bit:literal $(, count: $count:ident $(, count_attr: $count_attr:meta)?)?;)+) => {
        impl TxFeatureFlags {
            $(pub const $flag: u32 = 1 << $bit;)+
        }
    };
}

with_transaction_features!(define_flags);

impl TxFeatureFlags {
    pub fn insert_type(&mut self, output_type: OutputType) {
        self.0 |= match output_type {
            OutputType::P2PK65 | OutputType::P2PK33 => Self::P2PK,
            OutputType::P2MS => Self::P2MS,
            OutputType::P2PKH => Self::P2PKH,
            OutputType::P2SH => Self::P2SH,
            OutputType::P2WPKH => Self::P2WPKH,
            OutputType::P2WSH => Self::P2WSH,
            OutputType::P2TR => Self::P2TR,
            OutputType::P2A => Self::P2A,
            OutputType::OpReturn => Self::OP_RETURN,
            OutputType::Empty => Self::EMPTY,
            OutputType::Unknown => Self::UNKNOWN,
        };
    }

    #[inline]
    pub fn insert(&mut self, flag: u32) {
        self.0 |= flag;
    }

    #[inline]
    pub fn contains_all(self, flags: u32) -> bool {
        self.0 & flags == flags
    }

    #[inline]
    pub fn is_set(self, flag: u32) -> bool {
        self.0 & flag != 0
    }
}

#[cfg(test)]
mod tests {
    use brk_types::OutputType;

    use super::TxFeatureFlags;

    #[test]
    fn type_flags_union_inputs_and_outputs() {
        let mut flags = TxFeatureFlags::default();
        flags.insert_type(OutputType::P2PKH);
        flags.insert_type(OutputType::P2TR);

        assert!(flags.is_set(TxFeatureFlags::P2PKH));
        assert!(flags.is_set(TxFeatureFlags::P2TR));
    }
}
