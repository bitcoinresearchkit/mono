use brk_types::{FeeRate, Sats, Txid, VSize};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// A canonical transaction's chain-derived summary. Vsize is BIP-141 weight/4 rounded up;
/// rate is Bitview's stored policy-size chunk rate, rounded up to 0.001 sat/vB.
/// Flags use mempool's bit positions, with the native classification policy described below.
/// Observer-only time/acceleration/replacement facts are unavailable and are omitted. In
/// particular, unset observation bits do not assert that the transaction was never observed.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TransactionSummary {
    pub txid: Txid,
    pub fee: Sats,
    pub vsize: VSize,
    /// Sum of every output, including change, unspendable outputs and coinbase outputs.
    pub value: Sats,
    pub rate: FeeRate,
    /// Native standardness, key/signature and CoinJoin heuristics; broad opcode-envelope
    /// detection; structural same-block CPFP roles; payout ratios include coinbase.
    /// Bits 18 (replacement) and 19 (acceleration) carry no observation here.
    pub flags: u64,
}

/// The wire bit positions, independent of the indexer's internal flag layout.
pub mod summary_flags {
    pub const RBF: u64 = 1;
    pub const NO_RBF: u64 = 1 << 1;
    pub const V1: u64 = 1 << 2;
    pub const V2: u64 = 1 << 3;
    pub const V3: u64 = 1 << 4;
    pub const NONSTANDARD: u64 = 1 << 5;
    pub const P2PK: u64 = 1 << 8;
    pub const P2MS: u64 = 1 << 9;
    pub const P2PKH: u64 = 1 << 10;
    pub const P2SH: u64 = 1 << 11;
    pub const P2WPKH: u64 = 1 << 12;
    pub const P2WSH: u64 = 1 << 13;
    pub const P2TR: u64 = 1 << 14;
    pub const CPFP_PARENT: u64 = 1 << 16;
    pub const CPFP_CHILD: u64 = 1 << 17;
    pub const OP_RETURN: u64 = 1 << 24;
    pub const FAKE_PUBKEY: u64 = 1 << 25;
    pub const INSCRIPTION: u64 = 1 << 26;
    pub const FAKE_SCRIPTHASH: u64 = 1 << 27;
    pub const ANNEX: u64 = 1 << 28;
    pub const COINJOIN: u64 = 1 << 32;
    pub const CONSOLIDATION: u64 = 1 << 33;
    pub const BATCH_PAYOUT: u64 = 1 << 34;
    pub const SIGHASH_ALL: u64 = 1 << 40;
    pub const SIGHASH_NONE: u64 = 1 << 41;
    pub const SIGHASH_SINGLE: u64 = 1 << 42;
    pub const SIGHASH_DEFAULT: u64 = 1 << 43;
    pub const SIGHASH_ACP: u64 = 1 << 44;
}
