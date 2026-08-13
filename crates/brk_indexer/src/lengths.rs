use brk_error::Result;
use brk_types::{
    EmptyOutputIndex, Height, OpReturnIndex, OutputType, P2AAddrIndex, P2MSOutputIndex,
    P2PK33AddrIndex, P2PK65AddrIndex, P2PKHAddrIndex, P2SHAddrIndex, P2TRAddrIndex,
    P2WPKHAddrIndex, P2WSHAddrIndex, TxInIndex, TxIndex, TxOutIndex, TypeIndex, UnknownOutputIndex,
};
use tracing::info;
use vecdb::{AnyStoredVec, PcoVec, PcoVecValue, ReadableVec, VecIndex, VecValue, WritableVec};

use crate::{Stores, Vecs, stores::IndexerStores as _};

/// Pipeline-wide length/count snapshot. Lengths semantics:
/// `bound.f = N` means positions `0..N` are fully written; readers
/// reject `pos >= bound.f`.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Lengths {
    pub empty_output_index: EmptyOutputIndex,
    pub height: Height,
    pub op_return_index: OpReturnIndex,
    pub p2ms_output_index: P2MSOutputIndex,
    pub p2pk33_addr_index: P2PK33AddrIndex,
    pub p2pk65_addr_index: P2PK65AddrIndex,
    pub p2pkh_addr_index: P2PKHAddrIndex,
    pub p2sh_addr_index: P2SHAddrIndex,
    pub p2tr_addr_index: P2TRAddrIndex,
    pub p2wpkh_addr_index: P2WPKHAddrIndex,
    pub p2wsh_addr_index: P2WSHAddrIndex,
    pub p2a_addr_index: P2AAddrIndex,
    pub tx_index: TxIndex,
    pub txin_index: TxInIndex,
    pub txout_index: TxOutIndex,
    pub unknown_output_index: UnknownOutputIndex,
}

pub trait IndexerLengths: Sized {
    fn clamp_to(&mut self, other: &Self);
    fn from_local(vecs: &Vecs, stores: &Stores) -> Result<Option<Self>>;
    fn resume_at(required_height: Height, vecs: &Vecs, stores: &Stores) -> Result<Option<Self>>;
}

impl Lengths {
    pub fn to_type_index(&self, output_type: OutputType) -> TypeIndex {
        match output_type {
            OutputType::Empty => *self.empty_output_index,
            OutputType::OpReturn => *self.op_return_index,
            OutputType::P2A => *self.p2a_addr_index,
            OutputType::P2MS => *self.p2ms_output_index,
            OutputType::P2PK33 => *self.p2pk33_addr_index,
            OutputType::P2PK65 => *self.p2pk65_addr_index,
            OutputType::P2PKH => *self.p2pkh_addr_index,
            OutputType::P2SH => *self.p2sh_addr_index,
            OutputType::P2TR => *self.p2tr_addr_index,
            OutputType::P2WPKH => *self.p2wpkh_addr_index,
            OutputType::P2WSH => *self.p2wsh_addr_index,
            OutputType::Unknown => *self.unknown_output_index,
        }
    }

    /// Bump per-block totals after processing a block.
    pub fn add_block(&mut self, tx_count: usize, input_count: usize, output_count: usize) {
        self.tx_index += TxIndex::from(tx_count);
        self.txin_index += TxInIndex::from(input_count);
        self.txout_index += TxOutIndex::from(output_count);
    }

    /// Increments the address index for the given address type and returns the previous value.
    /// Only call this for address types (P2PK65, P2PK33, P2PKH, P2SH, P2WPKH, P2WSH, P2TR, P2A).
    #[inline]
    pub fn increment_addr_index(&mut self, addr_type: OutputType) -> TypeIndex {
        match addr_type {
            OutputType::P2PK65 => self.p2pk65_addr_index.copy_then_increment(),
            OutputType::P2PK33 => self.p2pk33_addr_index.copy_then_increment(),
            OutputType::P2PKH => self.p2pkh_addr_index.copy_then_increment(),
            OutputType::P2SH => self.p2sh_addr_index.copy_then_increment(),
            OutputType::P2WPKH => self.p2wpkh_addr_index.copy_then_increment(),
            OutputType::P2WSH => self.p2wsh_addr_index.copy_then_increment(),
            OutputType::P2TR => self.p2tr_addr_index.copy_then_increment(),
            OutputType::P2A => self.p2a_addr_index.copy_then_increment(),
            _ => unreachable!(),
        }
    }

    pub fn push(&self, vecs: &mut Vecs) {
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
    fn read_local(vecs: &Vecs, stores: &Stores) -> Result<Option<Self>> {
        let Some(height) = matching_height(vecs.next_height(), stores.next_height()?) else {
            return Ok(None);
        };
        Ok(Self::collect_at(height, vecs))
    }

    /// Read lengths to resume at `required_height`. Reorg-aware:
    /// - if vector and store checkpoints differ, return `None` (full reset);
    /// - if local is ahead, clamp down to `required_height`;
    /// - if local is behind, return `None` (caller must full-reset).
    fn read_resume(required_height: Height, vecs: &Vecs, stores: &Stores) -> Result<Option<Self>> {
        let Some(local) = matching_height(vecs.next_height(), stores.next_height()?) else {
            return Ok(None);
        };
        if local < required_height {
            return Ok(None);
        }
        let height = if local > required_height {
            info!(
                "Reorg detected: rolling back from {} to {}",
                local, required_height
            );
            required_height
        } else {
            local
        };
        Ok(Self::collect_at(height, vecs))
    }

    fn collect_at(height: Height, vecs: &Vecs) -> Option<Self> {
        Some(Self {
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
            p2sh_addr_index: next_index(
                &vecs.addrs.p2sh.first_index,
                &vecs.addrs.p2sh.bytes,
                height,
            )?,
            p2tr_addr_index: next_index(
                &vecs.addrs.p2tr.first_index,
                &vecs.addrs.p2tr.bytes,
                height,
            )?,
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
}

impl IndexerLengths for Lengths {
    fn clamp_to(&mut self, other: &Self) {
        self.height = self.height.min(other.height);
        self.tx_index = self.tx_index.min(other.tx_index);
        self.txin_index = self.txin_index.min(other.txin_index);
        self.txout_index = self.txout_index.min(other.txout_index);
        self.empty_output_index = self.empty_output_index.min(other.empty_output_index);
        self.op_return_index = self.op_return_index.min(other.op_return_index);
        self.p2ms_output_index = self.p2ms_output_index.min(other.p2ms_output_index);
        self.p2pk33_addr_index = self.p2pk33_addr_index.min(other.p2pk33_addr_index);
        self.p2pk65_addr_index = self.p2pk65_addr_index.min(other.p2pk65_addr_index);
        self.p2pkh_addr_index = self.p2pkh_addr_index.min(other.p2pkh_addr_index);
        self.p2sh_addr_index = self.p2sh_addr_index.min(other.p2sh_addr_index);
        self.p2tr_addr_index = self.p2tr_addr_index.min(other.p2tr_addr_index);
        self.p2wpkh_addr_index = self.p2wpkh_addr_index.min(other.p2wpkh_addr_index);
        self.p2wsh_addr_index = self.p2wsh_addr_index.min(other.p2wsh_addr_index);
        self.p2a_addr_index = self.p2a_addr_index.min(other.p2a_addr_index);
        self.unknown_output_index = self.unknown_output_index.min(other.unknown_output_index);
    }

    fn from_local(vecs: &Vecs, stores: &Stores) -> Result<Option<Self>> {
        Self::read_local(vecs, stores)
    }

    fn resume_at(required_height: Height, vecs: &Vecs, stores: &Stores) -> Result<Option<Self>> {
        Self::read_resume(required_height, vecs, stores)
    }
}

fn matching_height(vec_height: Height, store_height: Option<Height>) -> Option<Height> {
    let store_height = store_height?;
    if vec_height == store_height {
        Some(vec_height)
    } else {
        info!(
            "Indexer checkpoint mismatch: vectors at {}, stores at {}; full reset required",
            vec_height, store_height
        );
        None
    }
}

/// Per-type next-to-write counter at `next_height`. `None` pre-genesis.
fn next_index<I, T>(
    height_to_index: &PcoVec<Height, I>,
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

#[cfg(test)]
mod checkpoint_tests {
    use super::*;
    use brk_types::StoredU32;
    use vecdb::{Database, ImportableVec, Stamp, Version};

    #[test]
    fn matching_checkpoint_is_accepted() {
        let height = Height::new(42);
        assert_eq!(matching_height(height, Some(height)), Some(height));
    }

    #[test]
    fn mismatched_checkpoint_requires_reset() {
        assert_eq!(
            matching_height(Height::new(42), Some(Height::new(41))),
            None
        );
        assert_eq!(
            matching_height(Height::new(41), Some(Height::new(42))),
            None
        );
        assert_eq!(matching_height(Height::ZERO, None), None);
    }

    #[test]
    fn componentwise_clamping_covers_every_field() {
        let value = u32::MAX as usize;
        let max = Lengths {
            empty_output_index: EmptyOutputIndex::from(value),
            height: Height::from(value),
            op_return_index: OpReturnIndex::from(value),
            p2ms_output_index: P2MSOutputIndex::from(value),
            p2pk33_addr_index: P2PK33AddrIndex::from(value),
            p2pk65_addr_index: P2PK65AddrIndex::from(value),
            p2pkh_addr_index: P2PKHAddrIndex::from(value),
            p2sh_addr_index: P2SHAddrIndex::from(value),
            p2tr_addr_index: P2TRAddrIndex::from(value),
            p2wpkh_addr_index: P2WPKHAddrIndex::from(value),
            p2wsh_addr_index: P2WSHAddrIndex::from(value),
            p2a_addr_index: P2AAddrIndex::from(value),
            tx_index: TxIndex::from(value),
            txin_index: TxInIndex::from(value),
            txout_index: TxOutIndex::from(value),
            unknown_output_index: UnknownOutputIndex::from(value),
        };
        let min = Lengths::default();

        let mut lengths = max;
        lengths.clamp_to(&min);
        assert_eq!(lengths, min);
    }

    #[test]
    fn genesis_stamp_uses_current_length() {
        let dir = tempfile::tempdir().unwrap();
        let db = Database::open(dir.path()).unwrap();
        let mut first_index =
            PcoVec::<Height, TxIndex>::forced_import(&db, "first_index", Version::ONE).unwrap();
        let mut values =
            PcoVec::<TxIndex, StoredU32>::forced_import(&db, "values", Version::ONE).unwrap();

        first_index.push(TxIndex::ZERO);
        values.push(StoredU32::from(1_u32));
        values.push(StoredU32::from(2_u32));
        first_index.stamped_write(Stamp::from(0_u64)).unwrap();

        assert_eq!(
            next_index(&first_index, &values, Height::new(1)),
            Some(TxIndex::new(2))
        );
        assert_eq!(next_index(&first_index, &values, Height::ZERO), None);
    }
}
