use brk_types::TxVersion;

use super::schema::with_transaction_features;

macro_rules! define_transaction_counts {
    ($($(#[$attribute:meta])* $vector:ident: $flag:ident = $bit:literal $(, count: $count:ident $(, count_attr: $count_attr:meta)?)?;)+) => {
        #[derive(Default)]
        pub(crate) struct TransactionCounts {
            pub(super) v1: u64,
            pub(super) v2: u64,
            pub(super) v3: u64,
            pub(super) other_version: u64,
            pub(super) explicitly_rbf: u64,
            pub(super) one_input: u64,
            pub(super) one_output: u64,
            $($(pub(super) $count: u64,)?) +
        }

        impl TransactionCounts {
            pub(crate) fn add_base(
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

#[cfg(test)]
mod tests {
    use brk_types::TxVersion;

    use super::TransactionCounts;

    #[test]
    fn counts_every_version_category_and_base_property() {
        let mut counts = TransactionCounts::default();
        counts.add_base(1, 2, TxVersion::ONE, false);
        counts.add_base(2, 1, TxVersion::TWO, true);
        counts.add_base(3, 3, TxVersion::THREE, false);
        counts.add_base(4, 4, TxVersion::NON_STANDARD, false);

        assert_eq!(counts.v1, 1);
        assert_eq!(counts.v2, 1);
        assert_eq!(counts.v3, 1);
        assert_eq!(counts.other_version, 1);
        assert_eq!(counts.explicitly_rbf, 1);
        assert_eq!(counts.one_input, 1);
        assert_eq!(counts.one_output, 1);
    }
}
