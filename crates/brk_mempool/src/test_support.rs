//! Tiny tx fixtures shared across the crate's unit tests. Keeps
//! constructor noise out of the test bodies so each test reads as
//! "set up, mutate, assert" without 20 lines of struct literals.

use bitcoin::{
    Amount, OutPoint as BitcoinOutPoint, ScriptBuf, Sequence, Transaction as BitcoinTransaction,
    TxIn as BitcoinTxIn, TxOut as BitcoinTxOut, Txid as BitcoinTxid, WPubkeyHash,
    Witness as BitcoinWitness, absolute::LockTime, hashes::Hash, transaction::Version,
};
use brk_types::{
    MempoolEntryInfo, RawLockTime, Sats, SigOps, Timestamp, Transaction, TxIn, TxOut, TxStatus,
    TxVersionRaw, Txid, VSize, Vout, Weight, Witness,
};

/// Deterministic `Txid` from a single seed byte. The first byte of the
/// hash is `seed`, the rest is zero, so tests can identify txs by eye
/// in debug output.
pub(crate) fn fake_txid(seed: u8) -> Txid {
    let mut bytes = [0u8; 32];
    bytes[0] = seed;
    Txid::from(BitcoinTxid::from_byte_array(bytes))
}

/// Minimal P2WPKH `script_pubkey` keyed off `seed` so distinct inputs
/// or outputs in the same test don't collide on `addr_bytes()`.
pub(crate) fn p2wpkh_script(seed: u8) -> ScriptBuf {
    let mut bytes = [0u8; 20];
    bytes[0] = seed;
    ScriptBuf::new_p2wpkh(&WPubkeyHash::from_byte_array(bytes))
}

/// Build a `Transaction` with one input per entry in `prevouts` (each
/// either resolved or `None`) and one output per `(script, value)`
/// pair. Counterparty txids on the inputs are derived from the seed so
/// `addr_bytes` extraction sees distinct prev txids per input.
pub(crate) fn fake_tx(
    seed: u8,
    prevouts: &[Option<TxOut>],
    outputs: &[(ScriptBuf, u64)],
) -> Transaction {
    let input = prevouts
        .iter()
        .enumerate()
        .map(|(i, p)| TxIn {
            is_coinbase: false,
            prevout: p.clone(),
            txid: fake_txid(seed.wrapping_add(100 + i as u8)),
            vout: Vout::ZERO,
            script_sig: ScriptBuf::new(),
            script_sig_asm: (),
            witness: Witness::default(),
            sequence: 0xffff_fffe,
            inner_redeem_script_asm: (),
            inner_witness_script_asm: (),
        })
        .collect();

    let output = outputs
        .iter()
        .map(|(script, value)| TxOut::from((script.clone(), Sats::from(*value))))
        .collect();

    let mut tx = Transaction {
        index: None,
        txid: fake_txid(seed),
        version: TxVersionRaw::from(Version::TWO),
        lock_time: RawLockTime::from(LockTime::ZERO),
        input,
        output,
        total_size: 200,
        weight: Weight::from(800u64),
        total_sigop_cost: SigOps::ZERO,
        fee: Sats::ZERO,
        status: TxStatus::UNCONFIRMED,
    };
    tx.refresh_sigops();
    tx
}

/// Plain `MempoolEntryInfo` keyed off `txid`. Test bodies usually
/// already have the txid from `fake_tx`, so this just fills in the
/// non-essential fields with deterministic placeholders.
pub(crate) fn fake_entry_info(txid: Txid, fee: u64, vsize: u64) -> MempoolEntryInfo {
    MempoolEntryInfo {
        txid,
        vsize: VSize::from(vsize),
        weight: Weight::from(vsize * 4),
        fee: Sats::from(fee),
        first_seen: Timestamp::from(0u32),
        depends: vec![],
    }
}

/// Bitcoin-protocol `Transaction` matching `fake_tx`. Round-trippable
/// against a brk `Transaction`, lets the Preparer's `Fresh` path decode
/// it without a real RPC payload.
pub(crate) fn fake_bitcoin_tx(
    prev_txid_seed: u8,
    outputs: &[(ScriptBuf, u64)],
) -> BitcoinTransaction {
    let input = vec![BitcoinTxIn {
        previous_output: BitcoinOutPoint {
            txid: BitcoinTxid::from_byte_array({
                let mut b = [0u8; 32];
                b[0] = prev_txid_seed;
                b
            }),
            vout: 0,
        },
        script_sig: ScriptBuf::new(),
        sequence: Sequence(0xffff_fffe),
        witness: BitcoinWitness::new(),
    }];
    let output = outputs
        .iter()
        .map(|(script, value)| BitcoinTxOut {
            value: Amount::from_sat(*value),
            script_pubkey: script.clone(),
        })
        .collect();
    BitcoinTransaction {
        version: Version::TWO,
        lock_time: LockTime::ZERO,
        input,
        output,
    }
}
