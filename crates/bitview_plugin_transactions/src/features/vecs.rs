use bitview_plugin_indexer::FlagView;
use bitview_traversable::Traversable;
use vecdb::{Rw, StorageMode};

use crate::flagged::{Flagged, TxCount};

type Windowed<M> = TxCount<M>;

/// Transaction features: each one's per-transaction flag beside its per-block count, with its
/// cumulative and window sums and its share of all transactions.
#[derive(Traversable)]
pub struct Vecs<M: StorageMode = Rw> {
    // By the locking scripts transactions create or spend.
    pub scripts: Scripts<M>,
    // By their signatures' hash types, and the Taproot annex.
    pub signatures: Signatures<M>,
    /// Non-coinbase transactions using SegWit serialization.
    #[traversable(lead = "SegWit transactions.")]
    pub segwit: Windowed<M>,
    /// Transactions with at least one input sequence number below `0xfffffffe`, the explicit opt-in RBF signal
    /// defined by BIP 125. This is a mechanical sequence signal: it does not prove the transaction was replaceable or
    /// replaced, does not include inherited signaling, and does not account for full-RBF policy. Coinbase
    /// transactions are evaluated by the same sequence rule.
    #[traversable(lead = "Replace-by-fee transactions: ones that say they may be replaced by one paying more.")]
    pub rbf_signaling: Flagged<FlagView, Windowed<M>>,
    /// Non-coinbase transactions with at least one output below the type-specific dust threshold: 672 sats for
    /// P2PK65, 576 for P2PK33, 546 for P2PKH, 540 for P2SH, 294 for P2WPKH, 330 for P2WSH or P2TR, 240 for P2A, and
    /// 471 for an empty script. P2MS and unknown scripts use their computed minimal non-dust value; OP_RETURN is
    /// excluded.
    #[traversable(lead = "Transactions with dust: outputs below Bitcoin Core's dust threshold.")]
    pub dust_output: Flagged<FlagView, Windowed<M>>,
    /// Version-1 transactions tagged as EPOBC colored-coin transfers or issuances: the first
    /// input's sequence number is below 4,096 (bits 12 to 31 zero) and its low six bits are 51
    /// (transfer) or 37 (genesis), the remaining bits giving the padding, as ChromaWay's reference
    /// implementation writes them, and the first input is not a Taproot spend (no EPOBC wallet made
    /// one; small sequence numbers there carry the tag by chance). EPOBC carries no `OP_RETURN`
    /// payload.
    #[traversable(lead = "EPOBC transactions: colored coins, an early way of issuing assets on Bitcoin.")]
    pub epobc: Flagged<FlagView, Windowed<M>>,
    /// Transactions with exactly one input, including the coinbase transaction.
    #[traversable(lead = "Transactions with a single input.")]
    pub one_input: Windowed<M>,
    /// Transactions with exactly one output, including the coinbase transaction.
    #[traversable(lead = "Transactions with a single output.")]
    pub one_output: Windowed<M>,
}

/// Features of the locking scripts a transaction creates or spends.
#[derive(Traversable)]
pub struct Scripts<M: StorageMode = Rw> {
    /// Transactions that create or, outside coinbase, spend at least one P2PK-shaped output with a 33- or 65-byte key
    /// field.
    #[traversable(lead = "Transactions with pay-to-public-key (P2PK) outputs, made or spent.")]
    pub p2pk: Flagged<FlagView, Windowed<M>>,
    /// Transactions that create or, outside coinbase, spend at least one pay-to-public-key-hash output.
    #[traversable(lead = "Transactions with pay-to-public-key-hash (P2PKH) outputs, made or spent.")]
    pub p2pkh: Flagged<FlagView, Windowed<M>>,
    /// Transactions that create or, outside coinbase, spend at least one bare multisig output recognized by Bitcoin
    /// script parsing.
    #[traversable(lead = "Transactions with bare multisig (P2MS) outputs, made or spent.")]
    pub p2ms: Flagged<FlagView, Windowed<M>>,
    /// Transactions that create or, outside coinbase, spend at least one pay-to-script-hash output.
    #[traversable(lead = "Transactions with pay-to-script-hash (P2SH) outputs, made or spent.")]
    pub p2sh: Flagged<FlagView, Windowed<M>>,
    /// Transactions that create or, outside coinbase, spend at least one version-0 pay-to-witness-public-key-hash
    /// output.
    #[traversable(lead = "Transactions with SegWit P2WPKH outputs, made or spent.")]
    pub p2wpkh: Flagged<FlagView, Windowed<M>>,
    /// Transactions that create or, outside coinbase, spend at least one version-0 pay-to-witness-script-hash output.
    #[traversable(lead = "Transactions with SegWit P2WSH outputs, made or spent.")]
    pub p2wsh: Flagged<FlagView, Windowed<M>>,
    /// Transactions that create or, outside coinbase, spend at least one pay-to-Taproot output.
    #[traversable(lead = "Transactions with Taproot (P2TR) outputs, made or spent.")]
    pub p2tr: Flagged<FlagView, Windowed<M>>,
    /// Transactions that create or, outside coinbase, spend at least one pay-to-Anchor output matching `OP_1
    /// PUSHBYTES_2 0x4e73`.
    #[traversable(lead = "Transactions with pay-to-anchor (P2A) outputs, made or spent.")]
    pub p2a: Flagged<FlagView, Windowed<M>>,
    /// Transactions that create or, outside coinbase, spend at least one output with an empty locking script.
    #[traversable(lead = "Transactions with empty-script outputs, made or spent.")]
    pub empty_script: Flagged<FlagView, Windowed<M>>,
    /// Transactions that create or, outside coinbase, spend at least one output not matching another recognized
    /// locking-script type.
    #[traversable(lead = "Transactions with unrecognized outputs, made or spent.")]
    pub unknown_script: Flagged<FlagView, Windowed<M>>,
    /// Transactions that create a P2PK output whose shaped public key is invalid, or a bare multisig output
    /// containing an invalid or recognized burn public key.
    #[traversable(lead = "Transactions with invalid or burn public keys, often data written as keys.")]
    pub fake_pubkey: Flagged<FlagView, Windowed<M>>,
    /// Transactions containing a consecutive run of P2WSH outputs whose 32-byte programs encode a big-endian two-byte
    /// payload length and the required zero padding in the final program.
    #[traversable(lead = "Transactions with fake script hashes, encoding data.")]
    pub fake_scripthash: Flagged<FlagView, Windowed<M>>,
}

/// Features of a transaction's signatures and witnesses.
#[derive(Traversable)]
pub struct Signatures<M: StorageMode = Rw> {
    /// Transactions where at least one Taproot input with more than one witness element ends in an annex whose first
    /// byte is `0x50`.
    #[traversable(lead = "Transactions with a Taproot annex: an extra witness element, kept for future use.")]
    pub annex: Flagged<FlagView, Windowed<M>>,
    /// Transactions containing at least one detected ECDSA or Schnorr signature using the `SIGHASH_ALL` base type,
    /// which commits to every output. `SIGHASH_ANYONECANPAY`, tracked separately, can narrow the input commitment.
    #[traversable(lead = "Transactions with SIGHASH_ALL signatures, the usual kind: committing to every output.")]
    pub sighash_all: Flagged<FlagView, Windowed<M>>,
    /// Transactions containing at least one detected ECDSA or Schnorr signature using the `SIGHASH_NONE` base type,
    /// which commits to no transaction outputs.
    #[traversable(lead = "Transactions with SIGHASH_NONE signatures: committing to none of the outputs.")]
    pub sighash_none: Flagged<FlagView, Windowed<M>>,
    /// Transactions containing at least one detected ECDSA or Schnorr signature using the `SIGHASH_SINGLE` base type,
    /// which normally commits only to the output at the signing input's position. Legacy signatures without a
    /// corresponding output retain Bitcoin's historical `SIGHASH_SINGLE` bug.
    #[traversable(lead = "Transactions with SIGHASH_SINGLE signatures: committing, normally, to one output.")]
    pub sighash_single: Flagged<FlagView, Windowed<M>>,
    /// Transactions containing at least one detected Taproot `SIGHASH_DEFAULT` signature. Taproot's omitted hash-type
    /// byte has the same commitments as `SIGHASH_ALL` without `SIGHASH_ANYONECANPAY`.
    #[traversable(lead = "Transactions with Taproot's default signatures: committing to every input and output.")]
    pub sighash_default: Flagged<FlagView, Windowed<M>>,
    /// Transactions containing at least one detected ECDSA or Schnorr signature with the `SIGHASH_ANYONECANPAY`
    /// modifier, which commits only to the signing input rather than every input. This is independent of ALL, NONE,
    /// and SINGLE.
    #[traversable(lead = "Transactions with ANYONECANPAY signatures: committing to their own input alone, so others can add theirs.")]
    pub sighash_anyone_can_pay: Flagged<FlagView, Windowed<M>>,
}
