//! What the analysis finds in one transaction: its feature flags, and its signatures, keys,
//! scripts and shape counted into its worker's block totals. The input analysis walks each script
//! and witness once and hands every argument, key and opcode it meets to the transaction's
//! `TxScan`, which reads the rest from the facts. An argument that is a signature, by the one rule
//! in `signature`, sets its sighash flag and is counted: a flag tells whether a transaction has
//! one, a count how many.

use bitcoin::{
    Sequence, Transaction, TxIn,
    opcodes::{
        Opcode,
        all::{
            OP_CHECKMULTISIG, OP_CHECKSIG, OP_CHECKSIGADD, OP_CHECKSIGVERIFY, OP_CLTV, OP_CSV,
            OP_HASH160, OP_IF, OP_NOTIF, OP_SHA256,
        },
    },
    sighash::EcdsaSighashType,
};
use brk_types::OutputType;

use super::{
    features, input, output,
    signature::{self, Ecdsa},
};
use crate::{BlockCounters, TxFeatureFlags};

/// One transaction's flags, and its counts, added to its worker's block totals as the analysis
/// meets them.
pub struct TxScan<'a> {
    pub flags: TxFeatureFlags,
    counters: &'a mut BlockCounters,
    spends_segwit: bool,
    spends_legacy: bool,
    relative_height_lock: bool,
    relative_time_lock: bool,
    /// An input's sequence isn't final, so the locktime applies.
    locktime_enabled: bool,
}

impl<'a> TxScan<'a> {
    pub fn new(counters: &'a mut BlockCounters) -> Self {
        Self {
            flags: TxFeatureFlags::default(),
            counters,
            spends_segwit: false,
            spends_legacy: false,
            relative_height_lock: false,
            relative_time_lock: false,
            locktime_enabled: false,
        }
    }

    /// An input's sequence, the coinbase's too: whether it signals replacement, and whether it
    /// lets the locktime apply.
    #[inline]
    pub fn sequence(&mut self, sequence: Sequence) {
        self.flags
            .insert(TxFeatureFlags::RBF_SIGNALING * u32::from(sequence.is_rbf()));
        self.locktime_enabled |= sequence.enables_absolute_lock_time();
    }

    /// What an input spends, from its facts once its scripts are walked.
    pub fn scan_input(
        &mut self,
        input: &TxIn,
        output_type: OutputType,
        facts: &input::Facts<'_>,
        counts_relative_locks: bool,
    ) {
        if spends_segwit(!input.witness.is_empty(), output_type) {
            self.spends_segwit = true;
        } else {
            self.spends_legacy = true;
        }
        if counts_relative_locks {
            self.relative_height_lock |= input.sequence.is_height_locked();
            self.relative_time_lock |= input.sequence.is_time_locked();
        }

        let counters = &mut *self.counters;
        let witness_multisig = u32::from(facts.witness.script.multisig);
        match output_type {
            OutputType::P2SH => {
                counters.p2sh_multisig_input_count += u32::from(facts.redeem.is_multisig());
                match facts
                    .redeem
                    .effective_output_type(output_type, input.witness.len())
                {
                    OutputType::P2WPKH => counters.p2sh_p2wpkh_input_count += 1,
                    OutputType::P2WSH => {
                        counters.p2sh_p2wsh_input_count += 1;
                        counters.p2sh_p2wsh_multisig_input_count += witness_multisig;
                    }
                    _ => {}
                }
            }
            OutputType::P2WSH => counters.p2wsh_multisig_input_count += witness_multisig,
            OutputType::P2TR if facts.witness.stack_items >= 2 => {
                counters.p2tr_script_path_input_count += 1;
                counters.inscription_input_count += u32::from(facts.witness.inscription);
            }
            _ => {}
        }
    }

    #[inline]
    pub fn opcode(&mut self, op: Opcode) {
        let counters = &mut *self.counters;
        *match op {
            OP_CHECKSIG => &mut counters.op_checksig_in_input_count,
            OP_CHECKSIGVERIFY => &mut counters.op_checksigverify_in_input_count,
            OP_CHECKMULTISIG => &mut counters.op_checkmultisig_in_input_count,
            OP_CHECKSIGADD => &mut counters.op_checksigadd_in_input_count,
            OP_CLTV => &mut counters.op_checklocktimeverify_in_input_count,
            OP_CSV => &mut counters.op_checksequenceverify_in_input_count,
            OP_HASH160 => &mut counters.op_hash160_in_input_count,
            OP_SHA256 => &mut counters.op_sha256_in_input_count,
            OP_IF => &mut counters.op_if_in_input_count,
            OP_NOTIF => &mut counters.op_notif_in_input_count,
            _ => return,
        } += 1;
    }

    /// Shape alone: a spend's own keys were checked by consensus, and a redeem script's keys are
    /// rarely not curve points, which this hot path doesn't spend a curve operation per key on.
    #[inline]
    pub fn input_pubkey(&mut self, bytes: &[u8]) {
        match (bytes.len(), bytes.first()) {
            (33, Some(2 | 3)) => self.counters.compressed_pubkey_in_input_count += 1,
            // Uncompressed, or hybrid (0x06, 0x07), which consensus takes as well.
            (65, Some(4 | 6 | 7)) => self.counters.uncompressed_pubkey_in_input_count += 1,
            _ => {}
        }
    }

    /// A legacy or v0 witness spend's argument: an ECDSA signature or not.
    pub fn ecdsa(&mut self, bytes: &[u8]) {
        let Some(signature) = Ecdsa::of(bytes) else {
            return;
        };
        let counters = &mut *self.counters;
        counters.ecdsa_signature_count += 1;
        counters.ecdsa_signature_bytes += bytes.len() as u32;
        counters.strict_der_ecdsa_signature_count += u32::from(signature.strict_der);
        counters.low_r_ecdsa_signature_count += u32::from(signature.low_r);
        counters.low_s_ecdsa_signature_count += u32::from(signature.low_s);
        counters.low_r_low_s_ecdsa_signature_count += u32::from(signature.low_r && signature.low_s);
        if let Some(length) = match bytes.len() {
            70 => Some(&mut counters.ecdsa_70_bytes_signature_count),
            71 => Some(&mut counters.ecdsa_71_bytes_signature_count),
            72 => Some(&mut counters.ecdsa_72_bytes_signature_count),
            73 => Some(&mut counters.ecdsa_73_bytes_signature_count),
            74 => Some(&mut counters.ecdsa_74_bytes_signature_count),
            75.. => Some(&mut counters.ecdsa_over_74_bytes_signature_count),
            _ => None,
        } {
            *length += 1;
        }
        let Some(sighash) = signature.sighash else {
            return;
        };
        self.flags.insert(features::ecdsa_sighash(sighash));
        *match sighash {
            EcdsaSighashType::All => &mut counters.sighash_all_ecdsa_signature_count,
            EcdsaSighashType::None => &mut counters.sighash_none_ecdsa_signature_count,
            EcdsaSighashType::Single => &mut counters.sighash_single_ecdsa_signature_count,
            EcdsaSighashType::AllPlusAnyoneCanPay => {
                &mut counters.sighash_all_anyone_can_pay_ecdsa_signature_count
            }
            EcdsaSighashType::NonePlusAnyoneCanPay => {
                &mut counters.sighash_none_anyone_can_pay_ecdsa_signature_count
            }
            EcdsaSighashType::SinglePlusAnyoneCanPay => {
                &mut counters.sighash_single_anyone_can_pay_ecdsa_signature_count
            }
        } += 1;
    }

    /// A Taproot key path's argument, or a tapscript's: a Schnorr signature or not.
    #[inline]
    pub fn schnorr(&mut self, bytes: &[u8]) {
        let Some(sighash) = signature::schnorr_sighash(bytes) else {
            return;
        };
        self.flags.insert(features::taproot_sighash(sighash));
        self.counters.schnorr_signature_count += 1;
        self.counters.schnorr_signature_bytes += bytes.len() as u32;
    }

    /// An output's keys, as the output scanner parsed them.
    pub fn scan_output(&mut self, keys: output::Keys) {
        self.counters.compressed_pubkey_in_output_count += keys.compressed;
        self.counters.uncompressed_pubkey_in_output_count += keys.uncompressed;
    }

    /// What the whole transaction is, once its inputs and outputs are walked: its flags.
    pub fn finish(self, tx: &Transaction, is_coinbase: bool) -> TxFeatureFlags {
        let counters = self.counters;
        counters.bip69_tx_count += u32::from(is_bip69_sorted(tx));
        counters.unenforced_locktime_tx_count +=
            u32::from(tx.lock_time.to_consensus_u32() > 0 && !self.locktime_enabled);

        if !is_coinbase {
            counters.relative_height_locktime_tx_count += u32::from(self.relative_height_lock);
            counters.relative_time_locktime_tx_count += u32::from(self.relative_time_lock);
            match (self.spends_segwit, self.spends_legacy) {
                (true, false) => counters.only_segwit_spending_tx_count += 1,
                (false, _) => counters.only_legacy_spending_tx_count += 1,
                (true, true) => counters.segwit_and_legacy_spending_tx_count += 1,
            }
        }
        self.flags
    }
}

/// Whether an input spends SegWit, a witness program native or nested: every such spend carries a
/// witness but pay-to-anchor's, which is empty.
fn spends_segwit(has_witness: bool, spent: OutputType) -> bool {
    has_witness || spent == OutputType::P2A
}

/// Inputs and outputs in BIP-69's order: by reversed txid then index, by value then script.
fn is_bip69_sorted(tx: &Transaction) -> bool {
    let inputs = tx.input.windows(2).all(|pair| {
        let (a, b) = (pair[0].previous_output, pair[1].previous_output);
        let (a_txid, b_txid) = (a.txid.to_raw_hash(), b.txid.to_raw_hash());
        let (a_txid, b_txid): (&[u8; 32], &[u8; 32]) = (a_txid.as_ref(), b_txid.as_ref());
        a_txid
            .iter()
            .rev()
            .cmp(b_txid.iter().rev())
            .then(a.vout.cmp(&b.vout))
            .is_le()
    });
    inputs
        && tx.output.windows(2).all(|pair| {
            pair[0]
                .value
                .cmp(&pair[1].value)
                .then_with(|| pair[0].script_pubkey.cmp(&pair[1].script_pubkey))
                .is_le()
        })
}
