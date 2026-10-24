use bitview_primitives::Lengths;
use brk_error::Result;
use brk_types::Height;
use tracing::{debug, warn};
use vecdb::{
    AnyStoredVec, CachePolicy, PcoVec, PcoVecValue, ReadableVec, VecIndex, VecValue, WritableVec,
};

use crate::{Stores, Vecs};

pub trait IndexerLengths: Sized {
    fn push(&self, vecs: &mut Vecs);
    fn from_local(vecs: &Vecs, stores: &Stores) -> Result<Option<Self>>;
    fn resume_at(required_height: Height, vecs: &Vecs, stores: &Stores) -> Result<Option<Self>>;
    /// Local lengths before `height`; `None` at height zero or when a boundary is missing.
    fn at(height: Height, vecs: &Vecs) -> Option<Self>;
}

impl IndexerLengths for Lengths {
    fn push(&self, vecs: &mut Vecs) {
        let height = self.height;
        vecs.transactions
            .first_tx_index
            .debug_checked_push(height, self.tx_index);
        vecs.inputs
            .first_txin_index
            .debug_checked_push(height, self.txin_index);
        vecs.outputs
            .first_txout_index
            .debug_checked_push(height, self.txout_index);
        vecs.scripts
            .empty_script
            .first_index
            .debug_checked_push(height, self.empty_script_output_index);
        vecs.scripts
            .p2ms
            .first_index
            .debug_checked_push(height, self.p2ms_output_index);
        vecs.op_return
            .first_index
            .debug_checked_push(height, self.op_return_output_index);
        vecs.scripts
            .unknown_script
            .first_index
            .debug_checked_push(height, self.unknown_script_output_index);
        push_addr_first_indexes(self, vecs);
    }

    /// Current local lengths; `None` at height zero, for absent or mismatched checkpoints, or when
    /// a boundary is missing.
    fn from_local(vecs: &Vecs, stores: &Stores) -> Result<Option<Self>> {
        let Some(height) = matching_height(vecs.next_height(), stores.next_height()?) else {
            return Ok(None);
        };
        Ok(collect_at(height, vecs))
    }

    /// Lengths before `required_height`, which a reorg puts below the local height; `None` for
    /// absent or mismatched checkpoints, a local height below `required_height`, height zero, or a
    /// missing boundary.
    fn resume_at(required_height: Height, vecs: &Vecs, stores: &Stores) -> Result<Option<Self>> {
        let Some(local) = matching_height(vecs.next_height(), stores.next_height()?) else {
            return Ok(None);
        };
        if local < required_height {
            return Ok(None);
        }
        if local > required_height {
            warn!(
                "Reorg detected: rolling back from {} to {}",
                local, required_height
            );
        }
        Ok(collect_at(required_height, vecs))
    }

    fn at(height: Height, vecs: &Vecs) -> Option<Self> {
        collect_at(height, vecs)
    }
}

fn collect_at(height: Height, vecs: &Vecs) -> Option<Lengths> {
    let mut lengths = Lengths {
        empty_script_output_index: next_index(
            &vecs.scripts.empty_script.first_index,
            &vecs.scripts.empty_script.to_tx_index,
            height,
        )?,
        height,
        p2ms_output_index: next_index(
            &vecs.scripts.p2ms.first_index,
            &vecs.scripts.p2ms.to_tx_index,
            height,
        )?,
        op_return_output_index: next_index(
            &vecs.op_return.first_index,
            &vecs.op_return.to_tx_index,
            height,
        )?,
        tx_index: next_index(
            &vecs.transactions.first_tx_index,
            &vecs.transactions.txid,
            height,
        )?,
        txin_index: next_index(
            &vecs.inputs.first_txin_index,
            &*vecs.inputs.outpoint,
            height,
        )?,
        txout_index: next_index(&vecs.outputs.first_txout_index, &vecs.outputs.value, height)?,
        unknown_script_output_index: next_index(
            &vecs.scripts.unknown_script.first_index,
            &vecs.scripts.unknown_script.to_tx_index,
            height,
        )?,
        ..Lengths::default()
    };
    collect_addr_indexes(&mut lengths, vecs)?;
    Some(lengths)
}

macro_rules! addr_lengths {
    ($(($variant:ident, $field:ident, $length:ident, $index:path, $bytes:path)),+ $(,)?) => {
        fn push_addr_first_indexes(lengths: &Lengths, vecs: &mut Vecs) {
            $(vecs.addrs
                .$field
                .first_index
                .debug_checked_push(lengths.height, lengths.$length);)+
        }

        fn collect_addr_indexes(lengths: &mut Lengths, vecs: &Vecs) -> Option<()> {
            $(lengths.$length = next_index(
                &vecs.addrs.$field.first_index,
                &vecs.addrs.$field.bytes,
                lengths.height,
            )?;)+
            Some(())
        }
    };
}

bitview_primitives::with_address_storage_types!(addr_lengths);

fn matching_height(vec_height: Height, store_height: Option<Height>) -> Option<Height> {
    let store_height = store_height?;
    if vec_height == store_height {
        Some(vec_height)
    } else {
        debug!(
            "Indexer checkpoint mismatch: vectors at {}, stores at {}",
            vec_height, store_height
        );
        None
    }
}

/// Per-type next index before `next_height`; `None` at height zero or when the boundary is
/// missing.
fn next_index<I, T>(
    height_to_index: &PcoVec<Height, I, impl CachePolicy>,
    values: &impl ReadableVec<I, T>,
    next_height: Height,
) -> Option<I>
where
    I: VecIndex + PcoVecValue + From<usize>,
    T: VecValue,
{
    let h = Height::from(height_to_index.stamp());
    if next_height.is_zero() {
        None
    } else if h.incremented() == next_height {
        Some(I::from(values.len()))
    } else {
        height_to_index.collect_one(next_height)
    }
}
