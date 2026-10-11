use bitview_plugin_indexer::FlagView;
use bitview_primitives::{Count, Count16};
use bitview_traversable::Traversable;
use bitview_vecs::PerBlockCumulativeRolling;
use brk_types::Height;
use vecdb::{LazyVec, Rw, StorageMode};

use crate::flagged::{Block, Flagged};

pub(super) type BlockView = Block<LazyVec<Height, Count, Height, Count16>>;
type Windowed<M> = PerBlockCumulativeRolling<Count, M>;

/// Transaction features: each one's per-transaction flag beside its per-block count, with
/// cumulative and window sums where they are computed here.
#[derive(Traversable)]
pub struct Vecs<M: StorageMode = Rw> {
    /// Transactions that create or, outside coinbase, spend at least one P2PK-shaped output with a 33- or 65-byte key
    /// field.
    pub p2pk: Flagged<FlagView, BlockView>,
    /// Transactions that create or, outside coinbase, spend at least one bare multisig output recognized by Bitcoin
    /// script parsing.
    pub p2ms: Flagged<FlagView, BlockView>,
    /// Transactions that create or, outside coinbase, spend at least one pay-to-public-key-hash output.
    pub p2pkh: Flagged<FlagView, BlockView>,
    /// Transactions that create or, outside coinbase, spend at least one pay-to-script-hash output.
    pub p2sh: Flagged<FlagView, BlockView>,
    /// Transactions that create or, outside coinbase, spend at least one version-0 pay-to-witness-public-key-hash
    /// output.
    pub p2wpkh: Flagged<FlagView, BlockView>,
    /// Transactions that create or, outside coinbase, spend at least one version-0 pay-to-witness-script-hash output.
    pub p2wsh: Flagged<FlagView, BlockView>,
    /// Transactions that create or, outside coinbase, spend at least one pay-to-Taproot output.
    pub p2tr: Flagged<FlagView, Windowed<M>>,
    /// Transactions that create or, outside coinbase, spend at least one pay-to-Anchor output matching `OP_1
    /// PUSHBYTES_2 0x4e73`.
    pub p2a: Flagged<FlagView, BlockView>,
    /// Transactions that create or, outside coinbase, spend at least one output with an empty locking script.
    pub empty: Flagged<FlagView, BlockView>,
    /// Transactions that create or, outside coinbase, spend at least one output not matching another recognized
    /// locking-script type.
    pub unknown: Flagged<FlagView, BlockView>,
    /// Transactions that create a P2PK output whose shaped public key is invalid, or a bare multisig output
    /// containing an invalid or recognized burn public key.
    pub fake_pubkey: Flagged<FlagView, BlockView>,
    /// Transactions containing a consecutive run of P2WSH outputs whose 32-byte programs encode a big-endian two-byte
    /// payload length and the required zero padding in the final program.
    pub fake_scripthash: Flagged<FlagView, BlockView>,
    /// Non-coinbase transactions using SegWit serialization.
    pub segwit: Windowed<M>,
    /// Transactions where at least one Taproot input with more than one witness element ends in an annex whose first
    /// byte is `0x50`.
    pub annex: Flagged<FlagView, Windowed<M>>,
    /// Transactions containing at least one detected ECDSA or Schnorr signature using the `SIGHASH_ALL` base type,
    /// which commits to every output. `SIGHASH_ANYONECANPAY`, tracked separately, can narrow the input commitment.
    pub sighash_all: Flagged<FlagView, Windowed<M>>,
    /// Transactions containing at least one detected ECDSA or Schnorr signature using the `SIGHASH_NONE` base type,
    /// which commits to no transaction outputs.
    pub sighash_none: Flagged<FlagView, Windowed<M>>,
    /// Transactions containing at least one detected ECDSA or Schnorr signature using the `SIGHASH_SINGLE` base type,
    /// which normally commits only to the output at the signing input's position. Legacy signatures without a
    /// corresponding output retain Bitcoin's historical `SIGHASH_SINGLE` bug.
    pub sighash_single: Flagged<FlagView, Windowed<M>>,
    /// Transactions containing at least one detected Taproot `SIGHASH_DEFAULT` signature. Taproot's omitted hash-type
    /// byte has the same commitments as `SIGHASH_ALL` without `SIGHASH_ANYONECANPAY`.
    pub sighash_default: Flagged<FlagView, Windowed<M>>,
    /// Transactions containing at least one detected ECDSA or Schnorr signature with the `SIGHASH_ANYONECANPAY`
    /// modifier, which commits only to the signing input rather than every input. This is independent of ALL, NONE,
    /// and SINGLE.
    pub sighash_anyone_can_pay: Flagged<FlagView, Windowed<M>>,
    /// Transactions with at least one input sequence number below `0xfffffffe`, the explicit opt-in RBF signal
    /// defined by BIP 125. This is a mechanical sequence signal: it does not prove the transaction was replaceable or
    /// replaced, does not include inherited signaling, and does not account for full-RBF policy. Coinbase
    /// transactions are evaluated by the same sequence rule.
    pub explicitly_rbf: Flagged<FlagView, Windowed<M>>,
    /// Non-coinbase transactions with at least one output below the type-specific dust threshold: 672 sats for
    /// P2PK65, 576 for P2PK33, 546 for P2PKH, 540 for P2SH, 294 for P2WPKH, 330 for P2WSH or P2TR, 240 for P2A, and
    /// 471 for an empty script. P2MS and unknown scripts use their computed minimal non-dust value; OP_RETURN is
    /// excluded.
    pub dust_output: Flagged<FlagView, Windowed<M>>,
    /// Transactions with exactly one input, including the coinbase transaction.
    pub one_input: BlockView,
    /// Transactions with exactly one output, including the coinbase transaction.
    pub one_output: BlockView,
}
