use bitcoin::{
    Script, Transaction as BitcoinTransaction, TxIn as BitcoinTxIn, TxOut as BitcoinTxOut,
    consensus::Encodable, constants::WITNESS_SCALE_FACTOR,
};
#[cfg(feature = "schemars")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    CheckedSub, RawLockTime, Sats, SigOps, TxIn, TxIndex, TxOut, TxStatus, TxVersionRaw, Txid,
    VSize, Weight, Witness,
};

/// Transaction information compatible with mempool.space API format
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
pub struct Transaction {
    /// Internal transaction index (brk-specific, not in mempool.space)
    #[cfg_attr(feature = "schemars", schemars(example = TxIndex::new(0)))]
    pub index: Option<TxIndex>,

    /// Transaction ID
    #[cfg_attr(
        feature = "schemars",
        schemars(example = "4a5e1e4baab89f3a32518a88c31bc87f618f76673e2cc77ab2127b7afdeda33b")
    )]
    pub txid: Txid,

    /// Transaction version (raw i32 from Bitcoin protocol, may contain non-standard values in coinbase txs)
    #[cfg_attr(feature = "schemars", schemars(example = 2))]
    pub version: TxVersionRaw,

    /// Transaction lock time
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

    /// Transaction weight
    #[cfg_attr(feature = "schemars", schemars(example = 558))]
    pub weight: Weight,

    /// Number of signature operations
    #[cfg_attr(feature = "schemars", schemars(example = SigOps::new(1)))]
    #[serde(rename = "sigops")]
    pub total_sigop_cost: SigOps,

    /// Transaction fee in satoshis
    #[cfg_attr(feature = "schemars", schemars(example = Sats::new(31)))]
    pub fee: Sats,

    /// Confirmation status (confirmed, block height/hash/time)
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

    /// Re-encode to canonical Bitcoin protocol bytes via
    /// `bitcoin::Transaction`. Lossless for mempool/confirmed txs
    /// (verified bytewise round-trip against Core over a 1000-tx live
    /// sample). Coinbase txs don't round-trip because brk's `Vout` is
    /// `u16` while the protocol's coinbase vout is `0xFFFFFFFF` -
    /// callers that may see coinbase shouldn't rely on this.
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

    /// Recompute `total_sigop_cost` from current inputs/outputs and
    /// write it back into the field. Cheaper than rebuilding the
    /// transaction when prevouts arrive; the only mutator paired with
    /// [`Self::total_sigop_cost`].
    pub fn refresh_sigops(&mut self) {
        self.total_sigop_cost = self.total_sigop_cost();
    }

    /// Total sigop cost (BIP-141 weight units).
    ///
    /// Mirrors `bitcoin::Transaction::total_sigop_cost`, but reads
    /// prevouts from `TxIn.prevout` and uses bitcoin's public
    /// `Script::redeem_script` (push-only check + last-push extraction
    /// in one). Inputs whose `prevout` is `None` skip the P2SH and
    /// witness components - legacy script-sig sigops are still counted.
    fn total_sigop_cost(&self) -> SigOps {
        let mut legacy: usize = 0;
        let mut redeem: usize = 0;
        let mut witness: usize = 0;

        for input in &self.input {
            legacy = legacy.saturating_add(input.script_sig.count_sigops_legacy());

            let Some(prevout) = input.prevout.as_ref() else {
                continue;
            };
            let spk: &Script = &prevout.script_pubkey;

            let redeem_script = spk
                .is_p2sh()
                .then(|| input.script_sig.redeem_script())
                .flatten();

            if let Some(rs) = redeem_script {
                redeem = redeem.saturating_add(rs.count_sigops());
            }

            let witness_program: Option<&Script> = if spk.is_witness_program() {
                Some(spk)
            } else {
                redeem_script
            };

            if let Some(wp) = witness_program {
                witness =
                    witness.saturating_add(count_sigops_with_witness_program(&input.witness, wp));
            }
        }

        for output in &self.output {
            legacy = legacy.saturating_add(output.script_pubkey.count_sigops_legacy());
        }

        SigOps::from(
            legacy
                .saturating_mul(WITNESS_SCALE_FACTOR)
                .saturating_add(redeem.saturating_mul(WITNESS_SCALE_FACTOR))
                .saturating_add(witness),
        )
    }
}

fn count_sigops_with_witness_program(witness: &Witness, witness_program: &Script) -> usize {
    if witness_program.is_p2wpkh() {
        1
    } else if witness_program.is_p2wsh() {
        witness
            .last()
            .map(|bytes| Script::from_bytes(bytes).count_sigops())
            .unwrap_or(0)
    } else {
        0
    }
}

/// Re-encode a brk `Transaction` to a canonical `bitcoin::Transaction`.
/// Lossless for mempool/confirmed txs (verified bytewise round-trip
/// against Core's `getrawtransaction` over a 1000-tx live sample).
///
/// Coinbase round-trip is **not** byte-perfect because brk's `Vout` is
/// a `u16` and coinbase encodes `vout = 0xFFFFFFFF` in the protocol;
/// the reconstructed value is `u16::MAX` (65535). Mempool txs are
/// never coinbase, and confirmed-tx callers don't go through this path.
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
