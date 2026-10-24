use bitcoin::{
    Script, Transaction as BitcoinTransaction, TxIn as BitcoinTxIn, TxOut as BitcoinTxOut, Witness,
    consensus::Encodable, constants::WITNESS_SCALE_FACTOR, opcodes::all::OP_PUSHNUM_16,
    script::Instruction,
};
#[cfg(feature = "schemars")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    CheckedSub, RawLockTime, Sats, SigOpCount, SigOps, TxIn, TxIndex, TxOut, TxStatus,
    TxVersionRaw, Txid, VSize, Weight,
};

/// Transaction information compatible with mempool.space API format
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
pub struct Transaction {
    /// Internal transaction index (brk-specific, not in mempool.space)
    #[cfg_attr(feature = "schemars", schemars(example = TxIndex::new(0)))]
    pub index: Option<TxIndex>,

    #[cfg_attr(
        feature = "schemars",
        schemars(example = "4a5e1e4baab89f3a32518a88c31bc87f618f76673e2cc77ab2127b7afdeda33b")
    )]
    pub txid: Txid,

    /// Transaction version (the protocol's 32 bits read unsigned, as Esplora reports them; may contain non-standard values in coinbase txs)
    #[cfg_attr(feature = "schemars", schemars(example = 2))]
    pub version: TxVersionRaw,

    #[cfg_attr(feature = "schemars", schemars(example = 0))]
    #[serde(rename = "locktime")]
    pub lock_time: RawLockTime,

    /// Transaction inputs
    #[serde(rename = "vin")]
    pub input: Vec<TxIn>,

    /// Transaction outputs
    #[serde(rename = "vout")]
    pub output: Vec<TxOut>,

    /// Transaction size in bytes
    #[cfg_attr(feature = "schemars", schemars(example = 222))]
    #[serde(rename = "size")]
    pub total_size: usize,

    #[cfg_attr(feature = "schemars", schemars(example = 558))]
    pub weight: Weight,

    /// BIP-141 signature-operation cost
    #[cfg_attr(feature = "schemars", schemars(example = SigOps::new(1)))]
    #[serde(rename = "sigops")]
    pub total_sigop_cost: SigOps,

    /// Transaction fee in satoshis
    #[cfg_attr(feature = "schemars", schemars(example = Sats::new(31)))]
    pub fee: Sats,

    pub status: TxStatus,
}

impl Transaction {
    fn fee(tx: &Transaction) -> Option<Sats> {
        let in_ = tx
            .input
            .iter()
            .map(|txin| txin.prevout.as_ref().map(|txout| txout.value))
            .sum::<Option<Sats>>()?;
        let out = tx.output.iter().map(|txout| txout.value).sum::<Sats>();
        Some(in_.checked_sub(out).unwrap())
    }

    pub fn compute_fee(&mut self) {
        self.fee = Self::fee(self).unwrap_or_default();
    }

    /// Canonical Bitcoin protocol bytes, via `bitcoin::Transaction`.
    pub fn encode_bytes(&self) -> Vec<u8> {
        let bitcoin_tx: BitcoinTransaction = self.into();
        let mut buf = Vec::with_capacity(self.total_size);
        Encodable::consensus_encode(&bitcoin_tx, &mut buf)
            .expect("in-memory consensus_encode is infallible");
        buf
    }

    /// Virtual size in vbytes (weight / 4, rounded up)
    #[inline]
    pub(crate) fn vsize(&self) -> VSize {
        VSize::from(self.weight)
    }

    /// Recompute `total_sigop_cost` once prevouts are filled in, without rebuilding the
    /// transaction.
    pub fn refresh_sigops(&mut self) {
        self.total_sigop_cost = self.total_sigop_cost();
    }

    /// Total sigop cost (BIP-141 weight units), counted as Core's `GetTransactionSigOpCost`,
    /// reading prevouts from `TxIn.prevout`. Inputs whose `prevout` is `None` skip the P2SH and
    /// witness components - legacy script-sig sigops are still counted.
    fn total_sigop_cost(&self) -> SigOps {
        let mut legacy: usize = 0;
        let mut redeem: usize = 0;
        let mut witness: usize = 0;

        for input in &self.input {
            legacy += SigOpCount::of(&input.script_sig).legacy;

            let Some(prevout) = input.prevout.as_ref() else {
                continue;
            };
            let spk: &Script = &prevout.script_pubkey;

            let redeem_script = spk
                .is_p2sh()
                .then(|| p2sh_redeem_script(&input.script_sig))
                .flatten();

            if let Some(rs) = redeem_script {
                redeem += SigOpCount::of(rs).accurate;
            }

            let witness_program: Option<&Script> = if spk.is_witness_program() {
                Some(spk)
            } else {
                redeem_script
            };

            if let Some(wp) = witness_program {
                witness += count_sigops_with_witness_program(&input.witness, wp);
            }
        }

        for output in &self.output {
            legacy += SigOpCount::of(&output.script_pubkey).legacy;
        }

        SigOps::from((legacy + redeem) * WITNESS_SCALE_FACTOR + witness)
    }
}

/// Core's P2SH redeem script: the last push of a push-only script sig, whose `OP_n`s push no
/// script (rust-bitcoin's `redeem_script` rejects them, missing spends like `<sig> OP_1 <redeem>`).
pub(crate) fn p2sh_redeem_script(script_sig: &Script) -> Option<&Script> {
    let mut last = None;
    for instruction in script_sig.instructions() {
        match instruction.ok()? {
            Instruction::PushBytes(bytes) => last = Some(bytes.as_bytes()),
            Instruction::Op(opcode) if opcode.to_u8() <= OP_PUSHNUM_16.to_u8() => last = None,
            Instruction::Op(_) => return None,
        }
    }
    last.map(Script::from_bytes)
}

fn count_sigops_with_witness_program(witness: &Witness, witness_program: &Script) -> usize {
    if witness_program.is_p2wpkh() {
        1
    } else if witness_program.is_p2wsh() {
        witness
            .witness_script()
            .map_or(0, |script| SigOpCount::of(script).accurate)
    } else {
        0
    }
}

/// Lossless: the canonical `bitcoin::Transaction`.
impl From<&Transaction> for BitcoinTransaction {
    #[inline]
    fn from(tx: &Transaction) -> Self {
        Self {
            version: tx.version.into(),
            lock_time: tx.lock_time.into(),
            input: tx.input.iter().map(BitcoinTxIn::from).collect(),
            output: tx.output.iter().map(BitcoinTxOut::from).collect(),
        }
    }
}
