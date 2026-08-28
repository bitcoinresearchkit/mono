use brk_error::Result;

use std::path::Path;

use bitview_traversable::Traversable;
use brk_types::{AddrHash, Height, OutputType, Version};
use rayon::prelude::*;
use tracing::debug;
use vecdb::{AnyStoredVec, AnyVec, Database, RawDBError, Rw, Stamp, StorageMode};

const PAGE_SIZE: usize = 4096;

use crate::Lengths;

#[macro_use]
mod macros;
mod addrs;
mod blocks;
mod inputs;
mod op_return;
mod outputs;
mod scripts;
mod transactions;

pub use addrs::*;
pub use blocks::*;
pub use inputs::*;
pub use op_return::*;
pub use outputs::*;
pub use scripts::*;
pub use transactions::*;

#[derive(Traversable)]
pub struct Vecs<M: StorageMode = Rw> {
    #[traversable(skip)]
    db: Database,
    pub blocks: BlocksVecs<M>,
    #[traversable(wrap = "transactions", rename = "raw")]
    pub transactions: TransactionsVecs<M>,
    #[traversable(wrap = "transactions", rename = "features")]
    pub transaction_features: TransactionFeaturesVecs<M>,
    #[traversable(wrap = "inputs", rename = "raw")]
    pub inputs: InputsVecs<M>,
    #[traversable(wrap = "outputs", rename = "raw")]
    pub outputs: OutputsVecs<M>,
    #[traversable(wrap = "addrs", rename = "raw")]
    pub addrs: AddrsVecs<M>,
    #[traversable(wrap = "scripts", rename = "raw")]
    pub scripts: ScriptsVecs<M>,
    #[traversable(wrap = "op_return", rename = "raw")]
    pub op_return: OpReturnVecs<M>,
}

pub trait IndexerVecs: Sized {
    fn forced_import(parent: &Path, version: Version) -> Result<Self>;
    fn rollback_if_needed(&mut self, starting_lengths: &Lengths) -> Result<()>;
    fn flush(&mut self, height: Height) -> Result<()>;
    fn stamped_write(&mut self, height: Height) -> Result<()>;
    fn sync_bg_tasks(&self) -> Result<()>;
    fn run_bg(
        &self,
        f: impl FnOnce(&Database) -> std::result::Result<(), RawDBError> + Send + 'static,
    );
    fn iter_addr_hashes_from(
        &self,
        addr_type: OutputType,
        height: Height,
    ) -> Result<Box<dyn Iterator<Item = AddrHash> + '_>>;
}

impl IndexerVecs for Vecs {
    fn forced_import(parent: &Path, version: Version) -> Result<Self> {
        debug!("Opening vecs database...");
        let db = Database::open(&parent.join("vecs"))?;
        debug!("Setting min len...");
        db.set_min_len(PAGE_SIZE * 60_000_000)?;

        let (
            blocks,
            transactions,
            transaction_features,
            inputs,
            outputs,
            addrs,
            scripts,
            op_return,
        ) = parallel_import! {
            blocks = BlocksVecs::forced_import(&db, version),
            transactions = TransactionsVecs::forced_import(&db, version),
            transaction_features = TransactionFeaturesVecs::forced_import(&db, version),
            inputs = InputsVecs::forced_import(&db, version),
            outputs = OutputsVecs::forced_import(&db, version),
            addrs = AddrsVecs::forced_import(&db, version),
            scripts = ScriptsVecs::forced_import(&db, version),
            op_return = OpReturnVecs::forced_import(&db, version),
        };

        let this = Self {
            db,
            blocks,
            transactions,
            transaction_features,
            inputs,
            outputs,
            addrs,
            scripts,
            op_return,
        };

        this.db.retain_accessed_regions()?;
        this.db.compact()?;

        Ok(this)
    }

    fn rollback_if_needed(&mut self, starting_lengths: &Lengths) -> Result<()> {
        let saved_height = starting_lengths.last_height().unwrap_or_default();
        let stamp = Stamp::from(u64::from(saved_height));

        self.blocks.truncate(starting_lengths.height, stamp)?;

        self.transactions
            .truncate(starting_lengths.height, starting_lengths.tx_index, stamp)?;

        self.transaction_features.truncate(
            starting_lengths.height,
            starting_lengths.tx_index,
            stamp,
        )?;

        self.inputs
            .truncate(starting_lengths.height, starting_lengths.txin_index, stamp)?;

        self.outputs
            .truncate(starting_lengths.height, starting_lengths.txout_index, stamp)?;

        self.addrs.truncate(
            starting_lengths.height,
            starting_lengths.p2pk65_addr_index,
            starting_lengths.p2pk33_addr_index,
            starting_lengths.p2pkh_addr_index,
            starting_lengths.p2sh_addr_index,
            starting_lengths.p2wpkh_addr_index,
            starting_lengths.p2wsh_addr_index,
            starting_lengths.p2tr_addr_index,
            starting_lengths.p2a_addr_index,
            stamp,
        )?;

        self.scripts.truncate(
            starting_lengths.height,
            starting_lengths.empty_output_index,
            starting_lengths.p2ms_output_index,
            starting_lengths.unknown_output_index,
            stamp,
        )?;

        self.op_return.truncate(
            starting_lengths.height,
            starting_lengths.op_return_index,
            stamp,
        )?;

        Ok(())
    }

    fn flush(&mut self, height: Height) -> Result<()> {
        self.stamped_write(height)?;
        self.db.flush()?;
        Ok(())
    }

    fn stamped_write(&mut self, height: Height) -> Result<()> {
        self.par_iter_mut_any_stored_vec()
            .try_for_each(|vec| vec.stamped_write(Stamp::from(height)))?;
        Ok(())
    }

    fn sync_bg_tasks(&self) -> Result<()> {
        self.db.sync_bg_tasks()?;
        Ok(())
    }

    fn run_bg(
        &self,
        f: impl FnOnce(&Database) -> std::result::Result<(), RawDBError> + Send + 'static,
    ) {
        self.db.run_bg(f);
    }

    fn iter_addr_hashes_from(
        &self,
        addr_type: OutputType,
        height: Height,
    ) -> Result<Box<dyn Iterator<Item = AddrHash> + '_>> {
        self.addrs.iter_hashes_from(addr_type, height)
    }
}

impl Vecs {
    pub fn next_height(&self) -> Height {
        let min_stamp = self
            .iter_any_stored_vec()
            .map(|vec| vec.stamp())
            .min()
            .unwrap();

        next_height_from_min_stamp(min_stamp, !self.blocks.blockhash.is_empty())
    }

    fn par_iter_mut_any_stored_vec(
        &mut self,
    ) -> impl ParallelIterator<Item = &mut dyn AnyStoredVec> {
        self.blocks
            .par_iter_mut_any()
            .chain(self.transactions.par_iter_mut_any())
            .chain(self.transaction_features.par_iter_mut_any())
            .chain(self.inputs.par_iter_mut_any())
            .chain(self.outputs.par_iter_mut_any())
            .chain(self.addrs.par_iter_mut_any())
            .chain(self.scripts.par_iter_mut_any())
            .chain(self.op_return.par_iter_mut_any())
    }

    fn iter_any_stored_vec(&self) -> impl Iterator<Item = &dyn AnyStoredVec> {
        self.blocks
            .iter_any()
            .chain(self.transactions.iter_any())
            .chain(self.transaction_features.iter_any())
            .chain(self.inputs.iter_any())
            .chain(self.outputs.iter_any())
            .chain(self.addrs.iter_any())
            .chain(self.scripts.iter_any())
            .chain(self.op_return.iter_any())
    }
}

fn next_height_from_min_stamp(min_stamp: Stamp, has_blocks: bool) -> Height {
    if has_blocks {
        Height::from(min_stamp).incremented()
    } else {
        Height::ZERO
    }
}

#[cfg(test)]
mod checkpoint_tests {
    use super::*;

    #[test]
    fn zero_stamp_distinguishes_empty_from_genesis() {
        let zero = Stamp::from(0_u64);

        assert_eq!(next_height_from_min_stamp(zero, false), Height::ZERO);
        assert_eq!(next_height_from_min_stamp(zero, true), Height::new(1));
    }

    #[test]
    fn nonzero_stamp_advances_to_next_height() {
        assert_eq!(
            next_height_from_min_stamp(Stamp::from(41_u64), true),
            Height::new(42)
        );
    }
}
