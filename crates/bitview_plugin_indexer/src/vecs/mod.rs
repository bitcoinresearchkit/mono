use std::path::Path;

use bitview_primitives::Index;
use bitview_traversable::Traversable;
use brk_error::Result;
use brk_types::{Height, Version};
use rayon::prelude::*;
use vecdb::{AnyStoredVec, AnyVec, Database, Rw, Stamp, StorageMode};

use crate::Lengths;

const PAGE_SIZE: usize = 4096;

pub mod addrs;
pub mod blocks;
pub mod inputs;
pub mod op_return;
pub mod outputs;
pub mod scripts;
pub mod transactions;

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
    pub transactions: TransactionsVecs<M>,
    pub inputs: InputsVecs<M>,
    pub outputs: OutputsVecs<M>,
    #[traversable(rename = "addresses")]
    pub addrs: AddrsVecs<M>,
    pub scripts: ScriptsVecs<M>,
    pub op_return: OpReturnVecs<M>,
}

impl Vecs {
    pub fn import(parent: &Path, version: Version) -> Result<Self> {
        let path = parent.join("vecs");
        Self::open(&path, version).map_err(|error| error.opening_storage(&path))
    }

    fn open(path: &Path, version: Version) -> Result<Self> {
        let db = Database::open(path)?;
        db.set_min_len(PAGE_SIZE * 60_000_000)?;

        let (blocks, transactions, inputs, outputs, addrs, scripts, op_return) = parallel_import! {
            blocks = BlocksVecs::import(&db, version),
            transactions = TransactionsVecs::import(&db, version),
            inputs = InputsVecs::import(&db, version),
            outputs = OutputsVecs::import(&db, version),
            addrs = AddrsVecs::import(&db, version),
            scripts = ScriptsVecs::import(&db, version),
            op_return = OpReturnVecs::import(&db, version),
        };

        let this = Self {
            db,
            blocks,
            transactions,
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

    /// Truncates every stored vec to its index's length in `starting_lengths`, stamped as of the
    /// block before them.
    pub fn rollback_if_needed(&mut self, starting_lengths: &Lengths) -> Result<()> {
        let stamp = stamp_of(starting_lengths);
        self.blocks.reset_median_window();
        for vec in self.iter_any_stored_mut() {
            let len = len_in(starting_lengths, vec.index_type_name());
            vec.update_stamp(stamp);
            vec.any_truncate_if_needed_at(len)?;
        }
        Ok(())
    }

    /// Whether every stored vec holds exactly `lengths`: its index's rows, stamped as of the block
    /// before them.
    pub fn holds(&self, lengths: &Lengths) -> bool {
        let stamp = stamp_of(lengths);
        self.iter_any_stored_vec().all(|vec| {
            vec.stamp() == stamp && vec.len() == len_in(lengths, vec.index_type_name())
        })
    }

    pub fn flush(&mut self, lengths: &Lengths) -> Result<()> {
        self.write(lengths)?;
        self.db.flush();
        Ok(())
    }

    /// Writes every stored vec, stamped as of the block before `lengths`.
    pub fn write(&mut self, lengths: &Lengths) -> Result<()> {
        self.blocks.compute_median_times();
        let stamp = stamp_of(lengths);
        let vecs = self.iter_any_stored_mut().collect::<Vec<_>>();
        let count = vecs.len();
        vecs.into_par_iter()
            .try_for_each(|vec| vec.stamped_write(stamp))?;
        debug_assert_eq!(
            self.iter_any_stored_vec().count(),
            count,
            "the stored vecs listed by reference differ from the struct's"
        );
        Ok(())
    }

    pub fn db(&self) -> &Database {
        &self.db
    }

    pub fn next_height(&self) -> Height {
        if self.blocks.blockhash.is_empty() {
            return Height::ZERO;
        }
        let min_stamp = self
            .iter_any_stored_vec()
            .map(|vec| vec.stamp())
            .min()
            .unwrap();
        Height::from(min_stamp).incremented()
    }

    fn iter_any_stored_vec(&self) -> impl Iterator<Item = &dyn AnyStoredVec> {
        self.blocks
            .iter_any()
            .chain(self.transactions.iter_any())
            .chain(self.inputs.iter_any())
            .chain(self.outputs.iter_any())
            .chain(self.addrs.iter_any())
            .chain(self.scripts.iter_any())
            .chain(self.op_return.iter_any())
    }
}

/// The stamp of vectors holding `lengths`: the block before them (zero when empty).
fn stamp_of(lengths: &Lengths) -> Stamp {
    Stamp::from(lengths.last_height().unwrap_or_default())
}

/// A stored vec's length in `lengths`, by the name of its index.
fn len_in(lengths: &Lengths, index_type_name: &str) -> usize {
    Index::try_from(index_type_name)
        .ok()
        .and_then(|index| lengths.get(index))
        .expect("an index the indexer counts")
}
