use std::path::Path;

use bitview_primitives::Index;
use bitview_traversable::Traversable;
use brk_error::Result;
use brk_types::{Height, Version};
use rayon::prelude::*;
use tracing::debug;
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
        debug!("Opening vecs database...");
        let db = Database::open(&parent.join("vecs"))?;
        debug!("Setting min len...");
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
        let saved_height = starting_lengths.last_height().unwrap_or_default();
        let stamp = Stamp::from(u64::from(saved_height));
        self.blocks.reset_median_window();
        for vec in self.iter_any_stored_mut() {
            let len = Index::try_from(vec.index_type_to_string())
                .ok()
                .and_then(|index| starting_lengths.get(index))
                .expect("an index the indexer counts");
            vec.update_stamp(stamp);
            vec.any_truncate_if_needed_at(len)?;
        }
        Ok(())
    }

    pub fn flush(&mut self, height: Height) -> Result<()> {
        self.stamped_write(height)?;
        self.db.flush();
        Ok(())
    }

    pub fn stamped_write(&mut self, height: Height) -> Result<()> {
        self.blocks.compute_median_times()?;
        let stamp = Stamp::from(height);
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
        let min_stamp = self
            .iter_any_stored_vec()
            .map(|vec| vec.stamp())
            .min()
            .unwrap();

        next_height_from_min_stamp(min_stamp, !self.blocks.blockhash.is_empty())
    }

    pub fn all_empty(&self) -> bool {
        self.iter_any_stored_vec().all(|vec| vec.len() == 0)
    }

    /// Whether every vector was flushed at the same checkpoint.
    pub fn checkpoints_match(&self) -> bool {
        let mut stamps = self.iter_any_stored_vec().map(|vec| vec.stamp());
        let first = stamps.next();
        stamps.all(|stamp| Some(stamp) == first)
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

fn next_height_from_min_stamp(min_stamp: Stamp, has_blocks: bool) -> Height {
    if has_blocks {
        Height::from(min_stamp).incremented()
    } else {
        Height::ZERO
    }
}
