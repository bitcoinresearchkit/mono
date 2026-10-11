use bitview_primitives::TxVersion;

macro_rules! define_transaction_counts {
    (
        features { $($(#[$doc:meta])* $feature:ident: $vector:ident, $flag:ident = $bit:literal;)+ }
        flags { $($(#[$flag_attribute:meta])* $flag_vector:ident: $flag_only:ident = $flag_bit:literal;)+ }
    ) => {
        /// One block's transaction counts while it is indexed.
        #[derive(Default)]
        pub struct TransactionCounts {
            pub v1: u16,
            pub v2: u16,
            pub v3: u16,
            pub other_version: u16,
            pub one_input: u16,
            pub one_output: u16,
            $(pub $feature: u16,)+
        }

        impl TransactionCounts {
            pub fn add_base(&mut self, input_count: usize, output_count: usize, version: TxVersion) {
                match version {
                    TxVersion::ONE => self.v1 += 1,
                    TxVersion::TWO => self.v2 += 1,
                    TxVersion::THREE => self.v3 += 1,
                    _ => self.other_version += 1,
                }
                self.one_input += u16::from(input_count == 1);
                self.one_output += u16::from(output_count == 1);
            }
        }
    };
}

with_transaction_features!(define_transaction_counts);
