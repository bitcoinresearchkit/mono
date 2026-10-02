use brk_error::Result;
use brk_types::{Height, Lengths};
use tracing::{debug, warn};
use vecdb::{
    AnyStoredVec, CachePolicy, PcoVec, PcoVecValue, ReadableVec, VecIndex, VecValue, WritableVec,
};

use crate::{Stores, Vecs};

pub trait IndexerLengths: Sized {
    fn push(&self, vecs: &mut Vecs);
    fn from_local(vecs: &Vecs, stores: &Stores) -> Result<Option<Self>>;
    fn resume_at(required_height: Height, vecs: &Vecs, stores: &Stores) -> Result<Option<Self>>;
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
            .empty
            .first_index
            .debug_checked_push(height, self.empty_output_index);
        vecs.scripts
            .p2ms
            .first_index
            .debug_checked_push(height, self.p2ms_output_index);
        vecs.op_return
            .first_index
            .debug_checked_push(height, self.op_return_index);
        vecs.addrs
            .p2a
            .first_index
            .debug_checked_push(height, self.p2a_addr_index);
        vecs.scripts
            .unknown
            .first_index
            .debug_checked_push(height, self.unknown_output_index);
        vecs.addrs
            .p2pk33
            .first_index
            .debug_checked_push(height, self.p2pk33_addr_index);
        vecs.addrs
            .p2pk65
            .first_index
            .debug_checked_push(height, self.p2pk65_addr_index);
        vecs.addrs
            .p2pkh
            .first_index
            .debug_checked_push(height, self.p2pkh_addr_index);
        vecs.addrs
            .p2sh
            .first_index
            .debug_checked_push(height, self.p2sh_addr_index);
        vecs.addrs
            .p2tr
            .first_index
            .debug_checked_push(height, self.p2tr_addr_index);
        vecs.addrs
            .p2wpkh
            .first_index
            .debug_checked_push(height, self.p2wpkh_addr_index);
        vecs.addrs
            .p2wsh
            .first_index
            .debug_checked_push(height, self.p2wsh_addr_index);
    }

    /// Read current local lengths. `None` pre-genesis.
    fn from_local(vecs: &Vecs, stores: &Stores) -> Result<Option<Self>> {
        let Some(height) = matching_height(vecs.next_height(), stores.next_height()?) else {
            return Ok(None);
        };
        Ok(collect_at(height, vecs))
    }

    /// Read lengths to resume at `required_height`. Reorg-aware:
    /// - if vector and store checkpoints differ, return `None` (full reset);
    /// - if local is ahead, clamp down to `required_height`;
    /// - if local is behind, return `None` (caller must full-reset).
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
}

fn collect_at(height: Height, vecs: &Vecs) -> Option<Lengths> {
    Some(Lengths {
        empty_output_index: next_index(
            &vecs.scripts.empty.first_index,
            &vecs.scripts.empty.to_tx_index,
            height,
        )?,
        height,
        p2ms_output_index: next_index(
            &vecs.scripts.p2ms.first_index,
            &vecs.scripts.p2ms.to_tx_index,
            height,
        )?,
        op_return_index: next_index(
            &vecs.op_return.first_index,
            &vecs.op_return.to_tx_index,
            height,
        )?,
        p2pk33_addr_index: next_index(
            &vecs.addrs.p2pk33.first_index,
            &vecs.addrs.p2pk33.bytes,
            height,
        )?,
        p2pk65_addr_index: next_index(
            &vecs.addrs.p2pk65.first_index,
            &vecs.addrs.p2pk65.bytes,
            height,
        )?,
        p2pkh_addr_index: next_index(
            &vecs.addrs.p2pkh.first_index,
            &vecs.addrs.p2pkh.bytes,
            height,
        )?,
        p2sh_addr_index: next_index(&vecs.addrs.p2sh.first_index, &vecs.addrs.p2sh.bytes, height)?,
        p2tr_addr_index: next_index(&vecs.addrs.p2tr.first_index, &vecs.addrs.p2tr.bytes, height)?,
        p2wpkh_addr_index: next_index(
            &vecs.addrs.p2wpkh.first_index,
            &vecs.addrs.p2wpkh.bytes,
            height,
        )?,
        p2wsh_addr_index: next_index(
            &vecs.addrs.p2wsh.first_index,
            &vecs.addrs.p2wsh.bytes,
            height,
        )?,
        p2a_addr_index: next_index(&vecs.addrs.p2a.first_index, &vecs.addrs.p2a.bytes, height)?,
        tx_index: next_index(
            &vecs.transactions.first_tx_index,
            &vecs.transactions.txid,
            height,
        )?,
        txin_index: next_index(&vecs.inputs.first_txin_index, &vecs.inputs.outpoint, height)?,
        txout_index: next_index(&vecs.outputs.first_txout_index, &vecs.outputs.value, height)?,
        unknown_output_index: next_index(
            &vecs.scripts.unknown.first_index,
            &vecs.scripts.unknown.to_tx_index,
            height,
        )?,
    })
}

fn matching_height(vec_height: Height, store_height: Option<Height>) -> Option<Height> {
    let store_height = store_height?;
    if vec_height == store_height {
        Some(vec_height)
    } else {
        debug!(
            "Indexer checkpoint mismatch: vectors at {}, stores at {}; full reset required",
            vec_height, store_height
        );
        None
    }
}

/// Per-type next-to-write counter at `next_height`. `None` pre-genesis.
fn next_index<I, T>(
    height_to_index: &PcoVec<Height, I, impl CachePolicy>,
    index_to_else: &impl ReadableVec<I, T>,
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
        Some(I::from(index_to_else.len()))
    } else {
        height_to_index.collect_one(next_height)
    }
}
