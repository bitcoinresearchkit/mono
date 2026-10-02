use brk_types::TxVersion;

macro_rules! define_transaction_counts {
    ($($(#[$attribute:meta])* $vector:ident: $flag:ident = $bit:literal $(, count: $count:ident $(, count_attr: $count_attr:meta)?)?;)+) => {
        #[derive(Default)]
        pub struct TransactionCounts {
            pub v1: u64,
            pub v2: u64,
            pub v3: u64,
            pub other_version: u64,
            pub explicitly_rbf: u64,
            pub one_input: u64,
            pub one_output: u64,
            $($(pub $count: u64,)?) +
        }

        impl TransactionCounts {
            pub fn add_base(
                &mut self,
                input_count: usize,
                output_count: usize,
                version: TxVersion,
                explicitly_rbf: bool,
            ) {
                match version {
                    TxVersion::ONE => self.v1 += 1,
                    TxVersion::TWO => self.v2 += 1,
                    TxVersion::THREE => self.v3 += 1,
                    _ => self.other_version += 1,
                }
                self.explicitly_rbf += explicitly_rbf as u64;
                self.one_input += (input_count == 1) as u64;
                self.one_output += (output_count == 1) as u64;
            }
        }
    };
}

with_transaction_features!(define_transaction_counts);
