use brk_error::Result;
use brk_traversable::Traversable;
use brk_types::{Height, OutPoint, OutputType, TxInIndex, TxIndex, TxOutIndex, TypeIndex, Version};
use rayon::prelude::*;
use vecdb::{AnyStoredVec, Database, ImportableVec, PcoVec, Rw, Stamp, StorageMode, WritableVec};

use crate::parallel_import;

#[derive(Traversable)]
pub struct InputsVecs<M: StorageMode = Rw> {
    /// Global zero-based transaction-input index in canonical blockchain order.
    /// At `height`, this is where the block begins and equals the number of
    /// inputs in preceding blocks; at `tx_index`, it identifies the
    /// transaction's first input.
    pub first_txin_index: M::Stored<PcoVec<Height, TxInIndex>>,
    /// Previous-output reference encoded as the global transaction index and
    /// zero-based output position within that transaction. Coinbase inputs use
    /// `u32::MAX` for both components.
    pub outpoint: M::Stored<PcoVec<TxInIndex, OutPoint>>,
    /// Global zero-based transaction-output index in canonical blockchain order.
    /// At `txout_index`, this is the identity value; at `txin_index`, it
    /// identifies the previous output spent by the input, with `u64::MAX`
    /// representing a coinbase input.
    pub txout_index: M::Stored<PcoVec<TxInIndex, TxOutIndex>>,
    /// Global zero-based index of a transaction in canonical blockchain order.
    /// At `tx_index`, this is the identity value; at `txin_index`, it identifies
    /// the transaction containing the input; at type-specific output indexes,
    /// it identifies the transaction containing that output.
    pub tx_index: M::Stored<PcoVec<TxInIndex, TxIndex>>,
    /// BRK locking-script classification of an output. At `txout_index`, this
    /// classifies the indexed output; at `txin_index`, it classifies the
    /// previous output spent by the input. Coinbase inputs use `unknown`.
    pub output_type: M::Stored<PcoVec<TxInIndex, OutputType>>,
    /// Zero-based index within the output's BRK type-specific collection. At
    /// `txout_index`, this identifies the indexed output; at `txin_index`, it
    /// identifies the previous output spent by the input. Address types index
    /// distinct addresses, while other types index outputs in canonical order.
    /// Coinbase inputs use `u32::MAX`.
    pub type_index: M::Stored<PcoVec<TxInIndex, TypeIndex>>,
}

impl InputsVecs {
    pub fn forced_import(db: &Database, version: Version) -> Result<Self> {
        let (first_txin_index, outpoint, txout_index, tx_index, output_type, type_index) = parallel_import! {
            first_txin_index = PcoVec::forced_import(db, "first_txin_index", version),
            outpoint = PcoVec::forced_import(db, "outpoint", version),
            txout_index = PcoVec::forced_import(db, "txout_index", version),
            tx_index = PcoVec::forced_import(db, "tx_index", version),
            output_type = PcoVec::forced_import(db, "output_type", version),
            type_index = PcoVec::forced_import(db, "type_index", version),
        };
        Ok(Self {
            first_txin_index,
            outpoint,
            txout_index,
            tx_index,
            output_type,
            type_index,
        })
    }

    pub fn truncate(&mut self, height: Height, txin_index: TxInIndex, stamp: Stamp) -> Result<()> {
        self.first_txin_index
            .truncate_if_needed_with_stamp(height, stamp)?;
        self.outpoint
            .truncate_if_needed_with_stamp(txin_index, stamp)?;
        self.txout_index
            .truncate_if_needed_with_stamp(txin_index, stamp)?;
        self.tx_index
            .truncate_if_needed_with_stamp(txin_index, stamp)?;
        self.output_type
            .truncate_if_needed_with_stamp(txin_index, stamp)?;
        self.type_index
            .truncate_if_needed_with_stamp(txin_index, stamp)?;
        Ok(())
    }

    pub fn par_iter_mut_any(&mut self) -> impl ParallelIterator<Item = &mut dyn AnyStoredVec> {
        [
            &mut self.first_txin_index as &mut dyn AnyStoredVec,
            &mut self.outpoint,
            &mut self.txout_index,
            &mut self.tx_index,
            &mut self.output_type,
            &mut self.type_index,
        ]
        .into_par_iter()
    }

    pub fn iter_any(&self) -> impl Iterator<Item = &dyn AnyStoredVec> {
        [
            &self.first_txin_index as &dyn AnyStoredVec,
            &self.outpoint,
            &self.txout_index,
            &self.tx_index,
            &self.output_type,
            &self.type_index,
        ]
        .into_iter()
    }
}

#[cfg(test)]
mod tests {
    use brk_types::{Version, Vout};
    use rayon::prelude::*;
    use vecdb::{AnyVec, ReadableVec};

    use super::*;

    #[test]
    fn rollback_keeps_all_input_facts_aligned() {
        let dir = tempfile::tempdir().unwrap();
        let db = Database::open(dir.path()).unwrap();
        let mut inputs = InputsVecs::forced_import(&db, Version::ONE).unwrap();

        for txin_index in [0_usize, 2, 4] {
            inputs.first_txin_index.push(TxInIndex::from(txin_index));
        }

        let facts = [
            (
                OutPoint::COINBASE,
                TxOutIndex::COINBASE,
                TxIndex::ZERO,
                OutputType::Unknown,
                TypeIndex::COINBASE,
            ),
            (
                OutPoint::new(TxIndex::from(1_usize), Vout::ZERO),
                TxOutIndex::from(2_usize),
                TxIndex::from(2_usize),
                OutputType::P2PKH,
                TypeIndex::from(3_usize),
            ),
            (
                OutPoint::new(TxIndex::from(2_usize), Vout::ZERO),
                TxOutIndex::from(4_usize),
                TxIndex::from(3_usize),
                OutputType::P2TR,
                TypeIndex::from(5_usize),
            ),
            (
                OutPoint::new(TxIndex::from(3_usize), Vout::ZERO),
                TxOutIndex::from(6_usize),
                TxIndex::from(4_usize),
                OutputType::P2WPKH,
                TypeIndex::from(8_usize),
            ),
        ];

        for &(outpoint, txout_index, tx_index, output_type, type_index) in &facts {
            inputs.outpoint.push(outpoint);
            inputs.txout_index.push(txout_index);
            inputs.tx_index.push(tx_index);
            inputs.output_type.push(output_type);
            inputs.type_index.push(type_index);
        }

        inputs
            .par_iter_mut_any()
            .try_for_each(|vec| vec.stamped_write(Stamp::from(2_u64)))
            .unwrap();

        inputs
            .truncate(
                Height::from(1_usize),
                TxInIndex::from(2_usize),
                Stamp::from(0_u64),
            )
            .unwrap();
        inputs
            .par_iter_mut_any()
            .try_for_each(|vec| vec.stamped_write(Stamp::from(0_u64)))
            .unwrap();

        drop(inputs);
        drop(db);

        let db = Database::open(dir.path()).unwrap();
        let inputs = InputsVecs::forced_import(&db, Version::ONE).unwrap();
        assert_eq!(inputs.first_txin_index.len(), 1);
        assert!(inputs.iter_any().skip(1).all(|vec| vec.len() == 2));
        assert_eq!(
            inputs.txout_index.collect_range_at(0, 2),
            [TxOutIndex::COINBASE, TxOutIndex::from(2_usize)]
        );
    }
}
