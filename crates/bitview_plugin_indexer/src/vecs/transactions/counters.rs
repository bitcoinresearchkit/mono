//! What only reading a block can tell, counted per block while its transactions are analyzed:
//! signatures, keys, witnesses, sequences and script bytes, none of which the other columns keep.
//! One 32-bit column per measure (a count, or bytes where it says so), each the source of a public
//! cumulative series in the plugin that owns it (see `bitview_vecs::counter_group!`). Anything the
//! stored columns can answer is computed there instead.

use bitview_primitives::Count32;
use bitview_traversable::Traversable;
use brk_error::Result;
use brk_types::{Height, Version};
use vecdb::{AnyStoredVec, Database, ImportableVec, PcoVec, Rw, StorageMode, WritableVec};

macro_rules! define_block_counters {
    ($($counter:ident,)+) => {
        /// One block's counts while it is analyzed. 32 bits each: opcode and key occurrences can
        /// exceed 65,535 in a block.
        #[derive(Clone, Copy, Default)]
        pub struct BlockCounters {
            $(pub $counter: u32,)+
        }

        impl BlockCounters {
            /// Two partial totals as one, as parallel analysis workers reduce into the block's.
            pub fn merged(mut self, other: Self) -> Self {
                $(self.$counter += other.$counter;)+
                self
            }
        }

        #[derive(Traversable)]
        pub struct BlockCounterVecs<M: StorageMode = Rw> {
            $(pub $counter: M::Stored<PcoVec<Height, Count32>>,)+
        }

        impl BlockCounterVecs {
            pub(crate) fn import(db: &Database, version: Version) -> Result<Self> {
                Ok(Self {
                    $($counter: PcoVec::import(db, stringify!($counter), version)?,)+
                })
            }

            pub(crate) fn push(&mut self, height: Height, counters: &BlockCounters) {
                $(self.$counter.debug_checked_push(height, counters.$counter.into());)+
            }

            pub(crate) fn iter_any(&self) -> impl Iterator<Item = &dyn AnyStoredVec> {
                [$(&self.$counter as &dyn AnyStoredVec,)+].into_iter()
            }
        }
    };
}

define_block_counters! {
    compressed_pubkey_in_output_count,
    uncompressed_pubkey_in_output_count,
    ecdsa_signature_count,
    ecdsa_signature_bytes,
    schnorr_signature_count,
    schnorr_signature_bytes,
    strict_der_ecdsa_signature_count,
    ecdsa_70_bytes_signature_count,
    ecdsa_71_bytes_signature_count,
    ecdsa_72_bytes_signature_count,
    ecdsa_73_bytes_signature_count,
    ecdsa_74_bytes_signature_count,
    ecdsa_over_74_bytes_signature_count,
    low_r_ecdsa_signature_count,
    low_s_ecdsa_signature_count,
    low_r_low_s_ecdsa_signature_count,
    sighash_all_ecdsa_signature_count,
    sighash_none_ecdsa_signature_count,
    sighash_single_ecdsa_signature_count,
    sighash_all_anyone_can_pay_ecdsa_signature_count,
    sighash_none_anyone_can_pay_ecdsa_signature_count,
    sighash_single_anyone_can_pay_ecdsa_signature_count,
    compressed_pubkey_in_input_count,
    uncompressed_pubkey_in_input_count,
    p2tr_script_path_input_count,
    p2sh_p2wpkh_input_count,
    p2sh_p2wsh_input_count,
    p2sh_multisig_input_count,
    p2sh_p2wsh_multisig_input_count,
    p2wsh_multisig_input_count,
    inscription_input_count,
    op_checksig_in_input_count,
    op_checksigverify_in_input_count,
    op_checkmultisig_in_input_count,
    op_checksigadd_in_input_count,
    op_checklocktimeverify_in_input_count,
    op_checksequenceverify_in_input_count,
    op_hash160_in_input_count,
    op_sha256_in_input_count,
    op_if_in_input_count,
    op_notif_in_input_count,
    bip69_tx_count,
    unenforced_locktime_tx_count,
    relative_height_locktime_tx_count,
    relative_time_locktime_tx_count,
    only_segwit_spending_tx_count,
    only_legacy_spending_tx_count,
    segwit_and_legacy_spending_tx_count,
}
