# Auto-generated Bitview Python client
# Do not edit manually

from __future__ import annotations
from dataclasses import dataclass
from functools import cached_property
from typing import TypeVar, Generic, Any, Callable, Dict, Optional, List, Iterator, Literal, TypedDict, Union, Protocol, overload, Tuple, TYPE_CHECKING
from http.client import HTTPSConnection, HTTPConnection
from urllib.parse import urlparse
from datetime import date, datetime, timedelta, timezone
import json
import re

if TYPE_CHECKING:
    import pandas as pd  # type: ignore[import-not-found]
    import polars as pl  # type: ignore[import-not-found]

T = TypeVar('T')

# Type definitions

# Bitcoin address string
Addr = str
# Transaction ID (hash)
Txid = str
# US Dollar amount
Dollars = float
# Amount in satoshis (1 BTC = 100,000,000 sats)
Sats = int
# Index within its type (e.g., 0 for first P2WPKH address)
TypeIndex = int
# Type (P2PKH, P2WPKH, P2SH, P2TR, etc.)
OutputType = Literal["p2pk65", "p2pk33", "p2pkh", "p2ms", "p2sh", "opreturn", "p2wpkh", "p2wsh", "p2tr", "p2a", "empty", "unknown"]
# Signed satoshis (i64) - for values that can be negative.
# Used for changes, deltas, profit/loss calculations, etc.
SatsSigned = int
# Four-byte primary state stored for every address.
#
# Empty addresses with small lifetime totals are stored inline. The upper two
# bits select an inline layout or a sidecar, whose index occupies the lower 30
# bits.
AddrState = int
# A ratio in basis points: 10,000 represents 1.
#
# Maximum finite ratio: 429,496.7294. u32::MAX represents undefined.
# Finite input range is a debug-checked precondition, not a saturation policy.
# Serde preserves raw encoded bits; vector JSON emits null for undefined.
BasisPoints32 = int
# Bitcoin amount as floating point (1 BTC = 100,000,000 satoshis)
Bitcoin = float
# URL-friendly mining pool identifier
PoolSlug = Literal["unknown", "blockfills", "ultimuspool", "terrapool", "luxor", "1thash", "btccom", "bitfarms", "huobipool", "wayicn", "canoepool", "btctop", "bitcoincom", "175btc", "gbminers", "axbt", "asicminer", "bitminter", "bitcoinrussia", "btcserv", "simplecoinus", "btcguild", "eligius", "ozcoin", "eclipsemc", "maxbtc", "triplemining", "coinlab", "50btc", "ghashio", "stminingcorp", "bitparking", "mmpool", "polmine", "kncminer", "bitalo", "f2pool", "hhtt", "megabigpower", "mtred", "nmcbit", "yourbtcnet", "givemecoins", "braiinspool", "antpool", "multicoinco", "bcpoolio", "cointerra", "kanopool", "solock", "ckpool", "nicehash", "bitclub", "bitcoinaffiliatenetwork", "btcc", "bwpool", "exxbw", "bitsolo", "bitfury", "21inc", "digitalbtc", "8baochi", "mybtccoinpool", "tbdice", "hashpool", "nexious", "bravomining", "hotpool", "okexpool", "bcmonster", "1hash", "bixin", "tatmaspool", "viabtc", "connectbtc", "batpool", "waterhole", "dcexploration", "dcex", "btpool", "58coin", "bitcoinindia", "shawnp0wers", "phashio", "rigpool", "haozhuzhu", "7pool", "miningkings", "hashbx", "dpool", "rawpool", "haominer", "helix", "bitcoinukraine", "poolin", "secretsuperstar", "tigerpoolnet", "sigmapoolcom", "okpooltop", "hummerpool", "tangpool", "bytepool", "spiderpool", "novablock", "miningcity", "binancepool", "minerium", "lubiancom", "okkong", "aaopool", "emcdpool", "foundryusa", "sbicrypto", "arkpool", "purebtccom", "marapool", "kucoinpool", "entrustcharitypool", "okminer", "titan", "pegapool", "btcnuggets", "cloudhashing", "digitalxmintsy", "telco214", "btcpoolparty", "multipool", "transactioncoinmining", "btcdig", "trickysbtcpool", "btcmp", "eobot", "unomp", "patels", "gogreenlight", "bitcoinindiapool", "ekanembtc", "canoe", "tiger", "1m1x", "zulupool", "secpool", "ocean", "whitepool", "wiz", "wk057", "futurebitapollosolo", "carbonnegative", "portlandhodl", "phoenix", "neopool", "maxipool", "bitfufupool", "gdpool", "miningdutch", "publicpool", "miningsquared", "innopolistech", "btclab", "parasite", "redrockpool", "est3lar", "braiinssolo", "solopoolcom", "noderunners", "dmnd"]
# Fee rate stored in milli-sat/vB and exposed as sat/vB.
FeeRate = float
# Weight in weight units (WU). Max block weight is 4,000,000 WU.
Weight = int
# Block height
Height = int
# UNIX timestamp in seconds
Timestamp = int
# Double-SHA256 block-header hash, serialized in Bitcoin's conventional
# hexadecimal byte order.
BlockHash = str
# Position of a transaction within a single block (0 = coinbase).
# Distinct from `TxIndex`, which is the chain-wide global tx index.
BlockTxIndex = int
# Content hash of the projected next block (block 0 of the mempool
# snapshot), including its statistics and complete transaction bodies.
# Opaque token, distinct from HTTP ETag formatting: pass back
# to `GET /api/v1/mempool/block-template/diff/{hash}` to fetch deltas.
NextBlockHash = int
# Output type names used by Esplora and mempool.space.
OutputTypeNormalized = Literal["p2pk", "p2pkh", "multisig", "p2sh", "op_return", "v0_p2wpkh", "v0_p2wsh", "v1_p2tr", "anchor", "empty", "unknown"]
# Transaction locktime. Values below 500,000,000 are interpreted as block heights; values at or above are Unix timestamps.
RawLockTime = int
# BIP-141 sigop cost. The block-level budget is 80,000, so a `u32`
# fits a single tx's count with room to spare.
#
# Witness sigops count as 1; legacy and P2SH-redeem sigops count as 4.
# Five vbytes per sigop is the policy adjustment Core applies in
# `nSigOpCost` to discourage sigop-heavy txs (`max(weight/4, sigops*5)`).
SigOps = int
# Index of the output being spent in the previous transaction
Vout = int
# Transaction witness: a stack of byte arrays, one per witness item.
#
# Wraps `bitcoin::Witness` (single-buffer layout with offsets, much
# more compact than `Vec<Vec<u8>>`). Serializes as a JSON array of
# hex strings - the format used by Bitcoin Core REST and mempool.space
# and matching brk's `script_sig: ScriptBuf` (bytes internally, hex
# on the wire).
Witness = List[str]
# Chain-wide transaction index (0 = the genesis coinbase). For an
# in-block position, use `BlockTxIndex` instead.
TxIndex = int
# Raw transaction version (i32) from Bitcoin protocol.
# Unlike TxVersion (u8, indexed), this preserves non-standard values
# used in coinbase txs for miner signaling/branding.
TxVersionRaw = int
# One slot of the new template in a `BlockTemplateDiff`.
#
# Untagged on the wire so JSON type disambiguates the variants:
# - `Retained(idx)` serializes as a bare integer - index into the
#   transactions of the prior template (which the client cached at
#   `since`).
# - `New(tx)` serializes as a transaction object - a body that was
#   new or changed since the prior template and must replace this position.
#
# Reconstruction is a single pass: for each entry, either copy
# `prior[idx]` or append the inline body.
BlockTemplateDiffEntry = Union[int, "Transaction"]
# Yes or no.
Boolean = bool
# A ratio between 0 and 1.
#
# Floored at scale u32::MAX - 1. Zero and one are exact; finite quantization error is less than 1 / SCALE
# apart from floating-point arithmetic error. u32::MAX represents undefined.
# Non-finite inputs become undefined. Finite inputs must lie in [0, 1]; this
# precondition is checked only in debug builds. Keep cumulative state unrounded.
# Serde preserves raw encoded bits; vector JSON emits null for undefined.
BoundedRatio = int
# A size in bytes.
Bytes = int
# A size in bytes that fits 32 bits (under 4 GiB), such as a block or transaction size.
Bytes32 = int
# A mean size in bytes.
BytesFract = float
# Investor phase from the Capital Sentiment model.
#
# Codes are explicit because phase values are persisted. Code `0` represents
# unavailable model inputs and is therefore not a phase.
CapitalSentimentPhase = Literal["raging_bull", "bull", "cautious_bull", "hopeful_bull", "early_bull", "weak_bull", "limbo", "deep_bear", "bear", "early_bear"]
# An amount in US cents (100 cents = 1 USD).
#
# Unsigned, for values that are never negative (invested capital, realized cap, ...). `u64::MAX` is
# reserved as a NaN sentinel.
Cents = int
# A mean amount in cents.
CentsFract = float
# Signed cents (i64) - for values that can be negative.
# Used for profit/loss calculations, deltas, etc.
CentsSigned = int
# URPD cohort identifier. Use `GET /api/urpd` to list available cohorts.
#
# Names are non-empty ASCII `[a-z0-9_]+`. Availability is determined by
# supported age filters and published UTXO history.
Cohort = str
# Bitcoin multiplied by the blocks it was held.
CoinBlocks = float
# Bitcoin multiplied by the days it was held.
CoinDays = float
# Bitcoin multiplied by the years it was held.
CoinYears = float
# Up to the first 100 bytes of a coinbase transaction's first-input
# `scriptSig`. Bytes are preserved for storage and exposed as a string by
# mapping each byte to the same-valued Unicode code point. Pool attribution
# may search this raw value, but the value itself is not a normalized pool
# label.
#
# Stored as a fixed 101-byte record (1 byte length + 100 bytes data).
# Uses `[u8; 101]` internally so that `size_of::<CoinbaseTag>()` matches
# the serialized `Bytes::Array` size (vecdb requires this for alignment).
#
# Bitcoin consensus limits coinbase scriptSig to 2-100 bytes.
CoinbaseTag = str
# A number of things.
Count = int
# A number of things that fits 16 bits (at most 65,535), such as transactions per block.
Count16 = int
# A number of things that fits 32 bits, such as outputs per block.
Count32 = int
# A mean number of things.
CountFract = float
# A change in a number of things.
CountSigned = int
# Position of a transaction inside a `CpfpCluster.txs` array. Cluster-local,
# has no meaning outside the enclosing cluster.
CpfpClusterTxIndex = int
# Virtual size in vbytes (weight / 4, rounded up). Max block vsize is ~1,000,000 vB.
VSize = int
# Output format for API responses
Format = Literal["json", "csv"]
# Maximum number of results to return. Defaults to 100 if not specified.
Limit = int
# A positional index, YYYY-MM-DD date, or ISO 8601 timestamp.
RangeIndex = Union[int, str, str]
# Calendar date in YYYY-MM-DD format.
Date = str
# Index of a day.
Day1 = int
# Index of a 3-day period.
Day3 = int
# A duration in days.
Days = float
# Proof-of-work difficulty.
Difficulty = float
# A single difficulty adjustment entry.
# Serializes as array: [timestamp, height, difficulty, change_percent]
DifficultyAdjustmentEntry = List[float]
# Index of an output with an empty script.
EmptyOutputIndex = TypeIndex
# Index of a difficulty epoch (2,016 blocks).
Epoch = int
# Machine-readable error code.
ErrorCode = Literal["not_found", "invalid_addr", "invalid_network", "unsupported_type", "no_series", "series_unsupported_index", "weight_exceeded", "too_many_utxos", "unknown_addr", "unknown_txid", "out_of_range", "unindexable_date", "no_data", "series_not_found", "mempool_not_available", "state_updating", "internal_error", "bad_request", "overloaded", "timeout", "method_not_allowed"]
# Error category, following the HTTP status: `invalid_request` (4xx other than 404),
# `not_found` (404), `unavailable` (503; `Retry-After` when transient), `timeout` (504), `internal`
# (other 5xx).
ErrorType = Literal["invalid_request", "not_found", "unavailable", "timeout", "internal"]
# Exchange rates (USD base, on-chain only — no fiat pairs available)
ExchangeRates = dict
# A 32-bit floating-point value without a specific unit.
Float32 = float
# A 64-bit floating-point value without a specific unit.
Float64 = float
# Index of a halving epoch (210,000 blocks).
Halving = int
# Hashes per second.
Hashrate = float
# Hex-encoded string. Transparent wrapper over `String`: serializes
# as a plain JSON string and derefs to `str`, so anywhere `&str` or
# `AsRef<[u8]>` is expected the `Hex` "just works".
Hex = str
# Index of an hour.
Hour1 = int
# Index of a 12-hour period.
Hour12 = int
# Index of a 4-hour period.
Hour4 = int
# Aggregation dimension for querying series. Includes time-based (date, week, month, year),
# block-based (height, tx_index), and address/output type indexes.
Index = Literal["minute10", "minute30", "hour1", "hour4", "hour12", "day1", "day3", "week1", "month1", "month3", "month6", "year1", "year10", "halving", "epoch", "height", "tx_index", "txin_index", "txout_index", "empty_output_index", "op_return_index", "p2a_addr_index", "p2ms_output_index", "p2pk33_addr_index", "p2pk65_addr_index", "p2pkh_addr_index", "p2sh_addr_index", "p2tr_addr_index", "p2wpkh_addr_index", "p2wsh_addr_index", "unknown_output_index", "funded_addr_index", "empty_addr_index", "extended_empty_addr_index"]
# Index of a 10-minute period.
Minute10 = int
# Index of a 30-minute period.
Minute30 = int
# Index of a month.
Month1 = int
# Index of a quarter (3 months).
Month3 = int
# Index of a half-year (6 months).
Month6 = int
# Open, high, low and close prices in cents: [open, high, low, close].
OHLCCents = List[Cents]
# Open, high, low and close prices in US dollars: [open, high, low, close].
OHLCDollars = List[Dollars]
# Open, high, low and close prices in sats: [open, high, low, close].
OHLCSats = List[Sats]
# Index of an OP_RETURN output.
OpReturnIndex = TypeIndex
# Protocol or format detected in an OP_RETURN output.
OpReturnKind = Literal["runes", "veri_block", "omni", "stacks", "blockstack", "colu", "open_assets", "komodo", "coin_spark", "poet", "docproof", "open_timestamps", "factom", "eternity_wall", "memo", "bitproof", "ascribe", "stampery", "epobc", "bare_hash", "text", "empty", "unknown"]
# Index of a P2A (pay-to-anchor) address.
P2AAddrIndex = TypeIndex
# The 2-byte witness program of a P2A (pay-to-anchor) output.
P2ABytes = str
# Index of a P2MS (bare multisig) output.
P2MSOutputIndex = TypeIndex
# Index of a P2PK address with a compressed (33-byte) public key.
P2PK33AddrIndex = TypeIndex
# A compressed (33-byte) public key.
P2PK33Bytes = str
# Index of a P2PK address with an uncompressed (65-byte) public key.
P2PK65AddrIndex = TypeIndex
# An uncompressed (65-byte) public key.
P2PK65Bytes = str
# Index of a P2PKH address.
P2PKHAddrIndex = TypeIndex
# The 20-byte public key hash of a P2PKH output.
P2PKHBytes = str
# Index of a P2SH address.
P2SHAddrIndex = TypeIndex
# The 20-byte script hash of a P2SH output.
P2SHBytes = str
# Index of a P2TR (taproot) address.
P2TRAddrIndex = TypeIndex
# The 32-byte output key of a P2TR output.
P2TRBytes = str
# Index of a P2WPKH address.
P2WPKHAddrIndex = TypeIndex
# The 20-byte public key hash of a P2WPKH output.
P2WPKHBytes = str
# Index of a P2WSH address.
P2WSHAddrIndex = TypeIndex
# The 32-byte script hash of a P2WSH output.
P2WSHBytes = str
# Unsigned parts per million stored as u32.
# One unit is 0.000001. Range: 0–4,294.967294.
# Use for precise bounded ratios and percentages.
# `u32::MAX` is reserved as a NaN sentinel.
PartsPerMillion32 = int
# Unsigned parts per million stored as u64.
# One unit is 0.000001. Range: 0–18,446,744,073,709.551614.
# Use for precise wide-range ratios.
# `u64::MAX` is reserved as a NaN sentinel.
PartsPerMillion64 = int
# Signed parts per million stored as i32.
# One unit is 0.000001. Range: -2,147.483647 to +2,147.483647.
# Use for precise bounded signed ratios and percentages.
# `i32::MIN` is reserved as a NaN sentinel.
PartsPerMillionSigned32 = int
# Signed parts per million stored as i64.
# One unit is 0.000001. Range: -9,223,372,036,854.775807 to +9,223,372,036,854.775807.
# Use for precise wide-range signed ratios and percentages.
# `i64::MIN` is reserved as a NaN sentinel.
PartsPerMillionSigned64 = int
# A fraction per day.
PerDay = float
# Events per second.
PerSecond = float
# A percentage: a ratio times 100.
Percent = float
# A price divided by a reference price, in parts per million (1,000,000 represents 1).
#
# Finite values saturate at 4,294.967294; u32::MAX represents undefined.
# Saturation is deliberately specific to price ratios, across all cohorts.
# Non-finite inputs become undefined; finite inputs must be nonnegative
# (a debug-checked precondition). PPM conversion rounds to nearest, matching
# the existing price ratios.
# Serde preserves raw encoded bits; vector JSON emits null for undefined.
PriceRatio = int
# A discrete rank.
Rank = int
# A dimensionless ratio: a quotient, share or multiple.
Ratio = float
# A dimensionless ratio at double precision.
Ratio64 = float
# Fractional satoshis (f64): mean amounts in sats, and USD prices expressed in sats.
#
# A USD price in sats is `usd_value * 100_000_000 / btc_price`
#
# When BTC is $100,000:
# - $1 = 1,000 sats
# - $0.001 = 1 sat
# - $0.0001 = 0.1 sats (fractional)
SatsFract = float
# A signed model score.
Score = int
# Series name
SeriesName = str
# A duration in seconds (at most about 136 years).
Seconds = int
# A mean duration in seconds.
SecondsFract = float
# Comma-separated list of series names
#
# Deserialization permits at most 32 normalized names and 2,048 decoded input
# string bytes. For arrays, the byte budget is shared by their string values.
SeriesList = str
# BIP-141 signature-operation cost with enough range for cumulative and rolling totals.
SigOps64 = int
# A mean BIP-141 signature-operation cost.
SigOpsFract = float
# Time period for mining statistics.
#
# Used to specify the lookback window for pool statistics, hashrate calculations,
# and other time-based mining series.
TimePeriod = Literal["24h", "3d", "1w", "1m", "3m", "6m", "1y", "2y", "3y", "all"]
# Hierarchical tree node for organizing series into categories
TreeNode = Union[dict[str, "TreeNode"], "SeriesLeafWithSchema"]
# Index of a transaction input.
TxInIndex = int
# Index of a transaction output.
TxOutIndex = int
# Input index in the spending transaction
Vin = int
# Compact indexed transaction-version category. Values 1, 2, and 3 preserve
# those exact signed 32-bit Bitcoin transaction versions; 255 represents every
# other version.
TxVersion = int
# Index of an output with an unrecognized script.
UnknownOutputIndex = TypeIndex
# Aggregation strategy for URPD buckets.
# Options: raw (no aggregation), lin200/lin500/lin1000 (linear $200/$500/$1000),
# log10/log50/log100/log200/log500/log1000/log2000 (logarithmic with 10/50/100/200/500/1000/2000 buckets per decade).
UrpdAggregation = Literal["raw", "lin200", "lin500", "lin1000", "log10", "log50", "log100", "log200", "log500", "log1000", "log2000"]
# Weighting applied to a URPD: raw (unweighted), cointime, or coinflow.
UrpdWeight = Literal["raw", "cointime", "coinflow"]
# A mean virtual size in vbytes.
VSizeFract = float
# Version tracking for data schema and computed values.
#
# Used to detect when stored data needs to be recomputed due to changes
# in computation logic or source data versions. Supports validation
# against persisted versions to ensure compatibility.
Version = int
# Index of a week.
Week1 = int
# Weight in weight units with enough range for cumulative and rolling totals.
Weight64 = int
# A mean weight in weight units.
WeightFract = float
# Index of a year.
Year1 = int
# Index of a decade.
Year10 = int
# A duration in years.
Years = float
class AddrAfterTxidParam(TypedDict):
    """
    Bitcoin address + last-seen txid path parameters (Esplora-style pagination)

    Attributes:
        after_txid: Last txid from the previous page (return transactions strictly older than this)
    """
    address: Addr
    after_txid: Txid

class AddrChainStats(TypedDict):
    """
    Address statistics on the blockchain (confirmed transactions only)
    
    Based on mempool.space's format with type_index extension.

    Attributes:
        balance: Current confirmed balance in satoshis
        funded_txo_count: Total number of transaction outputs that funded this address
        funded_txo_sum: Total amount in satoshis received by this address across all funded outputs
        spent_txo_count: Total number of transaction outputs spent from this address
        spent_txo_sum: Total amount in satoshis spent from this address
        tx_count: Total number of confirmed transactions involving this address
        type_index: Index of this address within its type on the blockchain
        realized_price: Realized price (average cost basis) in USD
    """
    balance: Sats
    funded_txo_count: int
    funded_txo_sum: Sats
    spent_txo_count: int
    spent_txo_sum: Sats
    tx_count: int
    type_index: TypeIndex
    realized_price: Dollars

class AddrHashPrefixMatches(TypedDict):
    addr_type: OutputType
    prefix: str
    truncated: bool
    addresses: List[Addr]

class AddrHashPrefixParam(TypedDict):
    """
    Attributes:
        prefix: First 1–16 hexadecimal nibbles of the RapidHash v3 hash over the raw address payload bytes.
    """
    addr_type: OutputType
    prefix: str

class AddrMempoolStats(TypedDict):
    """
    Address statistics in the mempool (unconfirmed transactions only)
    
    Based on mempool.space's format.

    Attributes:
        balance_delta: Net pending (unconfirmed) balance change in satoshis; negative when pending spends exceed receipts
        funded_txo_count: Number of unconfirmed transaction outputs funding this address
        funded_txo_sum: Total amount in satoshis being received in unconfirmed transactions
        spent_txo_count: Number of unconfirmed transaction inputs spending from this address
        spent_txo_sum: Total amount in satoshis being spent in unconfirmed transactions
        tx_count: Number of unconfirmed transactions involving this address
    """
    balance_delta: SatsSigned
    funded_txo_count: int
    funded_txo_sum: Sats
    spent_txo_count: int
    spent_txo_sum: Sats
    tx_count: int

class AddrParam(TypedDict):
    """
    Bitcoin address path parameter
    """
    address: Addr

class AddrStats(TypedDict):
    """
    Address information compatible with mempool.space API format.

    Attributes:
        address: Bitcoin address string
        addr_type: BRK address type (p2pk33, p2pk65, p2pkh, p2sh, p2wpkh, p2wsh, p2tr, etc.)
        chain_stats: Statistics for confirmed transactions on the blockchain
        mempool_stats: Statistics for unconfirmed transactions in the mempool
        balance: Total current balance in satoshis, including pending (unconfirmed) mempool changes
    """
    address: Addr
    addr_type: OutputType
    chain_stats: AddrChainStats
    mempool_stats: AddrMempoolStats
    balance: Sats

class _AddrValidationRequired(TypedDict):
    isvalid: bool

class AddrValidation(_AddrValidationRequired, total=False):
    """
    Address validation result

    Attributes:
        isvalid: Whether the address is valid
        address: The validated address
        scriptPubKey: The scriptPubKey in hex
        isscript: Whether this is a script address (P2SH)
        iswitness: Whether this is a witness address
        witness_version: Witness version (0 for P2WPKH/P2WSH, 1 for P2TR)
        witness_program: Witness program in hex
        error_locations: Error locations (empty array for most errors)
        error: Error message for invalid addresses
    """
    address: Optional[str]
    scriptPubKey: Optional[str]
    isscript: Optional[bool]
    iswitness: Optional[bool]
    witness_version: Optional[int]
    witness_program: Optional[str]
    error_locations: Optional[List[int]]
    error: Optional[str]

class BlockCountParam(TypedDict):
    """
    Block count path parameter

    Attributes:
        block_count: Number of recent blocks to include
    """
    block_count: int

class _BlockPoolRequired(TypedDict):
    id: int
    name: str
    slug: PoolSlug
    blockNumber: int

class BlockPool(_BlockPoolRequired, total=False):
    """
    Mining pool identification for a block

    Attributes:
        id: Unique pool identifier
        name: Pool name
        slug: URL-friendly pool identifier
        blockNumber: This block's ordinal among blocks attributed to this pool
        minerNames: Miner name tags found in coinbase scriptsig
    """
    minerNames: Optional[List[str]]

class _BlockExtrasRequired(TypedDict):
    totalFees: Sats
    medianFee: FeeRate
    feeRange: List[FeeRate]
    reward: Sats
    pool: BlockPool
    avgFee: Sats
    avgFeeRate: FeeRate
    coinbaseRaw: str
    coinbaseAddresses: List[str]
    coinbaseSignature: str
    coinbaseSignatureAscii: str
    avgTxSize: float
    totalInputs: int
    totalOutputs: int
    totalOutputAmt: Sats
    medianFeeAmt: Sats
    feePercentiles: List[Sats]
    segwitTotalTxs: int
    segwitTotalSize: int
    segwitTotalWeight: Weight
    header: str
    utxoSetChange: int
    utxoSetSize: int
    totalInputAmt: Sats
    virtualSize: float
    orphans: List[str]
    price: Dollars

class BlockExtras(_BlockExtrasRequired, total=False):
    """
    Extended block data matching mempool.space /api/v1/blocks extras

    Attributes:
        totalFees: Total fees in satoshis
        medianFee: Median fee rate in sat/vB
        feeRange: Fee rate range: [min, 10%, 25%, 50%, 75%, 90%, max]
        reward: Total block reward (subsidy + fees) in satoshis
        pool: Mining pool that mined this block
        avgFee: Average fee per transaction in satoshis
        avgFeeRate: Average fee rate in sat/vB
        coinbaseRaw: Raw coinbase transaction scriptsig as hex
        coinbaseAddress: Primary coinbase output address
        coinbaseAddresses: All coinbase output addresses
        coinbaseSignature: Coinbase output script in ASM format
        coinbaseSignatureAscii: Coinbase scriptsig decoded as ASCII
        avgTxSize: Average transaction size in bytes
        totalInputs: Total number of inputs (excluding coinbase)
        totalOutputs: Total number of outputs
        totalOutputAmt: Total output amount in satoshis
        medianFeeAmt: Median fee amount in satoshis
        feePercentiles: Fee amount percentiles in satoshis: [min, 10%, 25%, 50%, 75%, 90%, max]
        segwitTotalTxs: Number of segwit transactions
        segwitTotalSize: Total size of segwit transactions in bytes
        segwitTotalWeight: Total weight of segwit transactions
        header: Raw 80-byte block header as hex
        utxoSetChange: UTXO set change (total outputs - total inputs, includes unspendable like OP_RETURN). Note: intentionally differs from utxo_set_size diff which excludes unspendable outputs. Matches mempool.space/bitcoin-cli behavior.
        utxoSetSize: Total spendable UTXO set size at this height (excludes OP_RETURN and other unspendable outputs)
        totalInputAmt: Total input amount in satoshis
        virtualSize: Virtual size in vbytes
        firstSeen: Timestamp when the block was first seen (always null, not yet supported)
        orphans: Orphaned blocks (always empty)
        price: USD price at block height
    """
    coinbaseAddress: Optional[str]
    firstSeen: Optional[int]

class BlockFeeRatesEntry(TypedDict):
    """
    A single block fee rates data point with percentiles.

    Attributes:
        avgHeight: Average block height in this window
        timestamp: Unix timestamp at the window midpoint
        avgFee_0: Minimum fee rate (sat/vB)
        avgFee_10: 10th percentile fee rate (sat/vB)
        avgFee_25: 25th percentile fee rate (sat/vB)
        avgFee_50: Median fee rate (sat/vB)
        avgFee_75: 75th percentile fee rate (sat/vB)
        avgFee_90: 90th percentile fee rate (sat/vB)
        avgFee_100: Maximum fee rate (sat/vB)
    """
    avgHeight: Height
    timestamp: Timestamp
    avgFee_0: FeeRate
    avgFee_10: FeeRate
    avgFee_25: FeeRate
    avgFee_50: FeeRate
    avgFee_75: FeeRate
    avgFee_90: FeeRate
    avgFee_100: FeeRate

class BlockFeesEntry(TypedDict):
    """
    A single block fees data point.

    Attributes:
        avgHeight: Average block height in this window
        timestamp: Unix timestamp at the window midpoint
        avgFees: Average fees per block in this window (sats)
        USD: BTC/USD price at this height
    """
    avgHeight: Height
    timestamp: Timestamp
    avgFees: Sats
    USD: Dollars

class BlockHashParam(TypedDict):
    """
    Block hash path parameter
    """
    hash: BlockHash

class BlockHashStartIndex(TypedDict):
    """
    Block hash + starting transaction index path parameters

    Attributes:
        hash: Bitcoin block hash
        start_index: Starting transaction index within the block (0-based)
    """
    hash: BlockHash
    start_index: BlockTxIndex

class BlockHashTxIndex(TypedDict):
    """
    Block hash + transaction index path parameters

    Attributes:
        hash: Bitcoin block hash
        index: Transaction index within the block (0-based)
    """
    hash: BlockHash
    index: BlockTxIndex

class BlockInfo(TypedDict):
    """
    Block information matching mempool.space /api/block/{hash}

    Attributes:
        id: Block hash
        height: Block height
        version: Block version
        timestamp: Block timestamp (Unix time)
        bits: Compact target (bits)
        nonce: Nonce
        difficulty: Block difficulty
        merkle_root: Merkle root of the transaction tree
        tx_count: Number of transactions
        size: Block size in bytes
        weight: Block weight in weight units
        previousblockhash: Previous block hash
        mediantime: Median time of the last 11 blocks
    """
    id: BlockHash
    height: Height
    version: int
    timestamp: Timestamp
    bits: int
    nonce: int
    difficulty: float
    merkle_root: str
    tx_count: int
    size: int
    weight: Weight
    previousblockhash: BlockHash
    mediantime: Timestamp

class _BlockInfoV1Required(TypedDict):
    id: BlockHash
    height: Height
    version: int
    timestamp: Timestamp
    bits: int
    nonce: int
    difficulty: float
    merkle_root: str
    tx_count: int
    size: int
    weight: Weight
    previousblockhash: BlockHash
    mediantime: Timestamp
    extras: BlockExtras

class BlockInfoV1(_BlockInfoV1Required, total=False):
    """
    Block information with extras, matching mempool.space /api/v1/blocks

    Attributes:
        id: Block hash
        height: Block height
        version: Block version
        timestamp: Block timestamp (Unix time)
        bits: Compact target (bits)
        nonce: Nonce
        difficulty: Block difficulty
        merkle_root: Merkle root of the transaction tree
        tx_count: Number of transactions
        size: Block size in bytes
        weight: Block weight in weight units
        previousblockhash: Previous block hash
        mediantime: Median time of the last 11 blocks
        stale: Whether this block has been replaced by a longer chain
        extras: Extended block data
    """
    stale: bool

class BlockRewardsEntry(TypedDict):
    """
    A single block rewards data point.

    Attributes:
        avgHeight: Average block height in this window
        timestamp: Unix timestamp at the window midpoint
        avgRewards: Average coinbase reward per block (subsidy + fees, sats)
        USD: BTC/USD price at this height
    """
    avgHeight: Height
    timestamp: Timestamp
    avgRewards: Sats
    USD: Dollars

class BlockSizeEntry(TypedDict):
    """
    A single block size data point.

    Attributes:
        avgHeight: Average block height in this window
        timestamp: Unix timestamp at the window midpoint
        avgSize: Rolling 24h median block size (bytes)
    """
    avgHeight: Height
    timestamp: Timestamp
    avgSize: int

class BlockWeightEntry(TypedDict):
    """
    A single block weight data point.

    Attributes:
        avgHeight: Average block height in this window
        timestamp: Unix timestamp at the window midpoint
        avgWeight: Rolling 24h median block weight (weight units)
    """
    avgHeight: Height
    timestamp: Timestamp
    avgWeight: Weight

class BlockSizesWeights(TypedDict):
    """
    Combined block sizes and weights response.

    Attributes:
        sizes: Block size data points
        weights: Block weight data points
    """
    sizes: List[BlockSizeEntry]
    weights: List[BlockWeightEntry]

class _BlockStatusRequired(TypedDict):
    in_best_chain: bool

class BlockStatus(_BlockStatusRequired, total=False):
    """
    Block status indicating whether block is in the best chain

    Attributes:
        in_best_chain: Whether this block is in the best chain
        height: Block height (only if in best chain)
        next_best: Hash of the next block in the best chain (null if tip)
    """
    height: Union[Height, None]
    next_best: Union[BlockHash, None]

class MempoolBlock(TypedDict):
    """
    Block info in a mempool.space like format for fee estimation.

    Attributes:
        blockSize: Total serialized block size in bytes (witness + non-witness).
        blockVSize: Total block virtual size in vbytes
        nTx: Number of transactions in the projected block
        totalFees: Total fees in satoshis
        medianFee: Median fee rate in sat/vB
        feeRange: Fee rate range: [min, 10%, 25%, 50%, 75%, 90%, max]
    """
    blockSize: int
    blockVSize: float
    nTx: int
    totalFees: Sats
    medianFee: FeeRate
    feeRange: List[FeeRate]

class _TxOutRequired(TypedDict):
    scriptpubkey: str
    scriptpubkey_asm: str
    scriptpubkey_type: OutputTypeNormalized
    value: Sats

class TxOut(_TxOutRequired, total=False):
    """
    Attributes:
        scriptpubkey: Script pubkey (locking script), encoded as hexadecimal.
        scriptpubkey_asm: Script pubkey in assembly format.
        scriptpubkey_type: Esplora/mempool.space script type.
        scriptpubkey_address: Bitcoin address, omitted for scripts without an address.
        value: Value of the output in satoshis.
    """
    scriptpubkey_address: Addr

class _TxInRequired(TypedDict):
    txid: Txid
    vout: Vout
    prevout: Union[TxOut, None]
    scriptsig: str
    scriptsig_asm: str
    is_coinbase: bool
    sequence: int

class TxIn(_TxInRequired, total=False):
    """
    Transaction input

    Attributes:
        txid: Transaction ID of the output being spent
        vout: Output index being spent (u16: coinbase is 65535, mempool.space uses u32: 4294967295)
        prevout: Information about the previous output being spent
        scriptsig: Signature script (hex, for non-SegWit inputs)
        scriptsig_asm: Signature script in assembly format
        witness: Witness data (stack items, present for SegWit inputs; hex-encoded on the wire)
        is_coinbase: Whether this input is a coinbase (block reward) input
        sequence: Input sequence number
        inner_redeemscript_asm: Inner redeemscript in assembly (for P2SH-wrapped SegWit: scriptsig + witness both present)
        inner_witnessscript_asm: Inner witnessscript in assembly (for P2WSH: last witness item decoded as script)
    """
    witness: Witness
    inner_redeemscript_asm: str
    inner_witnessscript_asm: str

class _TxStatusRequired(TypedDict):
    confirmed: bool

class TxStatus(_TxStatusRequired, total=False):
    """
    Transaction confirmation status

    Attributes:
        confirmed: Whether the transaction is confirmed
        block_height: Block height (only present if confirmed)
        block_hash: Block hash (only present if confirmed)
        block_time: Block timestamp (only present if confirmed)
    """
    block_height: Union[Height, None]
    block_hash: Union[BlockHash, None]
    block_time: Union[Timestamp, None]

class _TransactionRequired(TypedDict):
    txid: Txid
    version: TxVersionRaw
    locktime: RawLockTime
    vin: List[TxIn]
    vout: List[TxOut]
    size: int
    weight: Weight
    sigops: SigOps
    fee: Sats
    status: TxStatus

class Transaction(_TransactionRequired, total=False):
    """
    Transaction information compatible with mempool.space API format

    Attributes:
        index: Internal transaction index (brk-specific, not in mempool.space)
        txid: Transaction ID
        version: Transaction version (raw i32 from Bitcoin protocol, may contain non-standard values in coinbase txs)
        locktime: Transaction lock time
        vin: Transaction inputs
        vout: Transaction outputs
        size: Transaction size in bytes
        weight: Transaction weight
        sigops: Number of signature operations
        fee: Transaction fee in satoshis
        status: Confirmation status (confirmed, block height/hash/time)
    """
    index: Union[TxIndex, None]

class BlockTemplate(TypedDict):
    """
    Projected next-block contents from Bitcoin Core's `getblocktemplate`
    (block 0 of the snapshot). Returned by
    `GET /api/v1/mempool/block-template`.

    Attributes:
        hash: Pass to `GET /api/v1/mempool/block-template/diff/{hash}` to fetch deltas.
        stats: Aggregate stats for this block (size, vsize, fee range, ...).
        transactions: Full transaction bodies in `getblocktemplate` order.
    """
    hash: NextBlockHash
    stats: MempoolBlock
    transactions: List[Transaction]

class BlockTemplateDiff(TypedDict):
    """
    Delta between the current `getblocktemplate` projection and a prior
    one identified by `since`. Returned by
    `GET /api/v1/mempool/block-template/diff/{hash}`.
    
    `order` carries the full new template in template order: each entry
    is either a `Retained(idx)` pointing into the prior template (which
    the client cached at `since`) or a `New(tx)` inline body. Walk it
    once to rebuild the new template; no separate `added` array to
    cross-reference.
    
    `removed` lists txids no longer present. A changed body can be emitted as
    `New` without removing its txid; absence of a retained index alone does not
    imply removal.

    Attributes:
        hash: Current next-block hash. Use as `since` on the next diff call.
        since: Echoed prior hash the diff was computed against.
        order: New template in order. Each entry is either an index into the prior template's transactions or a full transaction body.
        removed: Txids that left the projected next block since `since` (confirmed, evicted, replaced, or pushed past block 0).
    """
    hash: NextBlockHash
    since: NextBlockHash
    order: List[BlockTemplateDiffEntry]
    removed: List[Txid]

class BlockTimestamp(TypedDict):
    """
    Block information returned for timestamp queries

    Attributes:
        height: Block height
        hash: Block hash
        timestamp: Block timestamp in ISO 8601 format
    """
    height: Height
    hash: BlockHash
    timestamp: str

class CpfpClusterChunk(TypedDict):
    """
    One SFL chunk inside a `CpfpCluster`. `txs` is in topological order
    (matches `CpfpCluster.txs` ordering); the chunk's `feerate` is the
    per-chunk SFL feerate and is the same for every tx in this chunk.
    """
    txs: List[CpfpClusterTxIndex]
    feerate: FeeRate

class CpfpClusterTx(TypedDict):
    """
    One entry in a `CpfpCluster.txs` array.

    Attributes:
        parents: In-cluster parents of this tx.
    """
    txid: Txid
    weight: Weight
    fee: Sats
    parents: List[CpfpClusterTxIndex]

class CpfpCluster(TypedDict):
    """
    CPFP cluster: the connected component the seed belongs to, plus its
    SFL linearization.

    Attributes:
        txs: All txs in the cluster, in topological order (parents before children).
        chunks: SFL-emitted chunks ordered by descending feerate.
        chunkIndex: Index into `chunks` of the chunk containing the seed tx.
    """
    txs: List[CpfpClusterTx]
    chunks: List[CpfpClusterChunk]
    chunkIndex: int

class CpfpEntry(TypedDict):
    """
    A transaction in a CPFP relationship.
    """
    txid: Txid
    weight: Weight
    fee: Sats

class _CpfpInfoRequired(TypedDict):
    ancestors: List[CpfpEntry]
    descendants: List[CpfpEntry]
    effectiveFeePerVsize: FeeRate
    sigops: SigOps
    fee: Sats
    vsize: VSize
    adjustedVsize: VSize

class CpfpInfo(_CpfpInfoRequired, total=False):
    """
    CPFP (Child Pays For Parent) information for a transaction.

    Attributes:
        ancestors: Ancestor transactions in the CPFP chain.
        bestDescendant: Best (highest fee rate) descendant, if any.
        descendants: Descendant transactions in the CPFP chain.
        effectiveFeePerVsize: Effective fee rate considering CPFP relationships (sat/vB). This is the seed's chunk feerate after lift-merging, i.e. the rate Core/mempool.space would surface for this tx.
        sigops: BIP-141 sigop cost for the seed tx (witness sigops count as 1, legacy and P2SH-redeem sigops count as 4).
        fee: Transaction fee (sats).
        vsize: Virtual size of the seed tx (vbytes).
        adjustedVsize: Policy-adjusted virtual size: `max(vsize, sigops * 5)`.
        cluster: Cluster the seed belongs to: full tx list, SFL-linearized chunks, and the seed's chunk index. Omitted when the seed has no ancestors and no descendants (matches mempool.space).
    """
    bestDescendant: Union[CpfpEntry, None]
    cluster: Union[CpfpCluster, None]

class DataRangeFormat(TypedDict, total=False):
    """
    Range parameters with output format for API query parameters.

    Attributes:
        start: Inclusive start: integer index, date (YYYY-MM-DD), or timestamp (ISO 8601). Negative integers count from end. Aliases: `from`, `f`, `s`
        end: Exclusive end: integer index, date (YYYY-MM-DD), or timestamp (ISO 8601). Negative integers count from end. Aliases: `to`, `t`, `e`
        limit: Maximum number of values to return (ignored if `end` is set). Aliases: `count`, `c`, `l`
        format: Format of the output
    """
    start: Union[RangeIndex, None]
    end: Union[RangeIndex, None]
    limit: Union[Limit, None]
    format: Format

class SeriesCount(TypedDict):
    """
    Series count statistics

    Attributes:
        distinct: Number of unique series available (e.g., realized_price, market_cap)
        total: Total number of series-index combinations across all timeframes
        lazy: Number of lazy (computed on-the-fly) series-index combinations
        stored: Number of eager (stored on disk) series-index combinations
    """
    distinct: int
    total: int
    lazy: int
    stored: int

class DetailedSeriesCount(TypedDict):
    """
    Detailed series count with per-database breakdown.

    Attributes:
        distinct: Number of unique series available (e.g., realized_price, market_cap)
        total: Total number of series-index combinations across all timeframes
        lazy: Number of lazy (computed on-the-fly) series-index combinations
        stored: Number of eager (stored on disk) series-index combinations
        by_db: Per-database breakdown of counts.
    """
    distinct: int
    total: int
    lazy: int
    stored: int
    by_db: dict[str, SeriesCount]

class DifficultyAdjustment(TypedDict):
    """
    Difficulty adjustment information.

    Attributes:
        progressPercent: Progress through current difficulty epoch (0-100%)
        difficultyChange: Estimated difficulty change at next retarget (%)
        estimatedRetargetDate: Estimated timestamp of next retarget (milliseconds)
        remainingBlocks: Blocks remaining until retarget
        remainingTime: Estimated time until retarget (milliseconds)
        previousRetarget: Previous difficulty adjustment (%)
        previousTime: Timestamp of most recent retarget (seconds)
        nextRetargetHeight: Height of next retarget
        timeAvg: Average block time in current epoch (milliseconds)
        adjustedTimeAvg: Time-adjusted average (milliseconds)
        timeOffset: Time offset from expected schedule (seconds)
        expectedBlocks: Expected blocks based on wall clock time since epoch start
    """
    progressPercent: float
    difficultyChange: float
    estimatedRetargetDate: int
    remainingBlocks: int
    remainingTime: int
    previousRetarget: float
    previousTime: Timestamp
    nextRetargetHeight: Height
    timeAvg: int
    adjustedTimeAvg: int
    timeOffset: int
    expectedBlocks: float

class DifficultyEntry(TypedDict):
    """
    A single difficulty data point in the hashrate summary.

    Attributes:
        time: Unix timestamp of the difficulty adjustment
        height: Block height of the adjustment
        difficulty: Difficulty value
        adjustment: Adjustment ratio (new/previous, e.g. 1.068 = +6.8%)
    """
    time: Timestamp
    height: Height
    difficulty: float
    adjustment: float

class DiskUsage(TypedDict):
    """
    Disk usage of the indexed data

    Attributes:
        brk: Human-readable brk data size (e.g., "48.8 GiB")
        brk_bytes: brk data size in bytes
        bitcoin: Human-readable Bitcoin blocks directory size
        bitcoin_bytes: Bitcoin blocks directory size in bytes
        ratio: Ratio of BRK bytes to Bitcoin bytes; zero when Bitcoin bytes are zero.
    """
    brk: str
    brk_bytes: int
    bitcoin: str
    bitcoin_bytes: int
    ratio: float

class EmptyAddrData(TypedDict):
    """
    Data of an empty address

    Attributes:
        tx_count: Total transaction count
        funded_txo_count: Total funded/spent transaction output count (equal since address is empty)
        transfered: Total satoshis transferred
    """
    tx_count: int
    funded_txo_count: int
    transfered: Sats

class ErrorDetail(TypedDict):
    """
    Attributes:
        type: Error category, following the HTTP status
        code: Machine-readable error code
        message: Human-readable description
        doc_url: Link to API documentation
    """
    type: ErrorType
    code: ErrorCode
    message: str
    doc_url: str

class ErrorBody(TypedDict):
    """
    The JSON body of every API error (`application/problem+json`).
    """
    error: ErrorDetail

class FundedAddrData(TypedDict):
    """
    Data for a funded (non-empty) address with current balance.
    
    Kept compact because one value is stored for every funded address.

    Attributes:
        received: Satoshis received by this address
        sent: Satoshis sent by this address
        realized_cap_raw: The realized capitalization: Σ(price × sats)
        tx_count: Total transaction count
        funded_txo_count: Number of transaction outputs funded to this address
        spent_txo_count: Number of transaction outputs spent by this address
    """
    received: Sats
    sent: Sats
    realized_cap_raw: int
    tx_count: int
    funded_txo_count: int
    spent_txo_count: int

class HashrateEntry(TypedDict):
    """
    A single hashrate data point.

    Attributes:
        timestamp: Unix timestamp
        avgHashrate: Average hashrate (H/s)
    """
    timestamp: Timestamp
    avgHashrate: int

class HashrateSummary(TypedDict):
    """
    Summary of network hashrate and difficulty data.

    Attributes:
        hashrates: Historical hashrate data points
        difficulty: Historical difficulty adjustments
        currentHashrate: Current network hashrate (H/s)
        currentDifficulty: Current network difficulty
    """
    hashrates: List[HashrateEntry]
    difficulty: List[DifficultyEntry]
    currentHashrate: int
    currentDifficulty: float

class Health(TypedDict):
    """
    Server health status

    Attributes:
        status: Health status ("healthy")
        service: Service name
        version: Server version
        timestamp: Current server time (ISO 8601)
        started_at: Server start time (ISO 8601)
        uptime_seconds: Uptime in seconds
        indexed_height: Height of the last indexed block
        computed_height: Height of the last computed block (series)
        tip_height: Height of the chain tip (from Bitcoin node)
        blocks_behind: Number of blocks behind the tip
        last_indexed_at: Human-readable timestamp of the last indexed block (ISO 8601)
        last_indexed_at_unix: Unix timestamp of the last indexed block
    """
    status: str
    service: str
    version: str
    timestamp: str
    started_at: str
    uptime_seconds: int
    indexed_height: Height
    computed_height: Height
    tip_height: Height
    blocks_behind: Height
    last_indexed_at: str
    last_indexed_at_unix: Timestamp

class HeightOrDateParam(TypedDict):
    """
    Path parameter accepting either a block height (`840000`) or a calendar date
    (`YYYY-MM-DD`). The handler resolves it and dispatches to the per-height or
    per-day variant, choosing the matching cache strategy.

    Attributes:
        point: Confirmed block height as decimal digits (`840000`) or calendar date in `YYYY-MM-DD` format.
    """
    point: str

class HeightParam(TypedDict):
    """
    Block height path parameter
    """
    height: Height

class HistoricalPriceEntry(TypedDict):
    """
    A single price data point

    Attributes:
        time: Unix timestamp
        USD: BTC/USD price
    """
    time: Timestamp
    USD: Dollars

class HistoricalPrice(TypedDict):
    """
    Historical price response

    Attributes:
        prices: Price data points
        exchangeRates: Exchange rates (currently empty)
    """
    prices: List[HistoricalPriceEntry]
    exchangeRates: ExchangeRates

class IndexInfo(TypedDict):
    """
    Information about an available index and its query aliases

    Attributes:
        index: The canonical index name
        aliases: All Accepted query aliases
    """
    index: Index
    aliases: List[str]

class MempoolInfo(TypedDict):
    """
    Mempool statistics with incrementally maintained fee histogram.

    Attributes:
        count: Number of transactions in the mempool
        vsize: Total virtual size of all transactions in the mempool (vbytes)
        total_fee: Total fees of all transactions in the mempool (satoshis)
        fee_histogram: Fee histogram: `[[fee_rate, vsize], ...]` sorted by descending fee rate
    """
    count: int
    vsize: VSize
    total_fee: Sats
    fee_histogram: List[List[float]]

class MempoolRecentTx(TypedDict):
    """
    Simplified mempool transaction for the `/api/mempool/recent` endpoint.

    Attributes:
        txid: Transaction ID
        fee: Transaction fee (sats)
        vsize: Virtual size (vbytes)
        value: Total output value (sats)
    """
    txid: Txid
    fee: Sats
    vsize: VSize
    value: Sats

class MerkleProof(TypedDict):
    """
    Merkle inclusion proof for a transaction

    Attributes:
        block_height: Block height containing the transaction
        merkle: Merkle proof path (hex-encoded hashes)
        pos: Transaction position in the block (0-indexed)
    """
    block_height: Height
    merkle: List[str]
    pos: int

class NextBlockHashParam(TypedDict):
    """
    Prior-template hash for `GET /api/v1/mempool/block-template/diff/{hash}`.
    """
    hash: NextBlockHash

class OptionalTimestampParam(TypedDict, total=False):
    """
    Optional UNIX timestamp query parameter
    """
    timestamp: Union[Timestamp, None]

class OutPoint(TypedDict):
    """
    The output a transaction input spends: its transaction index and output position, written as
    `{"tx_index": N, "vout": M}` (coinbase inputs: `{"tx_index": 4294967295, "vout": 65535}`).
    """
    tx_index: TxIndex
    vout: Vout

class PaginatedSeries(TypedDict):
    """
    A paginated list of available series names (1000 per page)

    Attributes:
        current_page: Current page number (0-indexed)
        max_page: Maximum valid page index (0-indexed)
        total_count: Total number of series
        per_page: Results per page
        has_more: Whether more pages are available after the current one
        series: List of series names
    """
    current_page: int
    max_page: int
    total_count: int
    per_page: int
    has_more: bool
    series: List[str]

class Pagination(TypedDict, total=False):
    """
    Pagination parameters for paginated API endpoints

    Attributes:
        page: Pagination index
        per_page: Results per page (default: 1000, max: 1000)
    """
    page: Optional[int]
    per_page: Optional[int]

# Block counts for different time periods
PoolBlockCounts = TypedDict("PoolBlockCounts", {"all": int, "24h": int, "1w": int})

# Pool's share of total blocks for different time periods
PoolBlockShares = TypedDict("PoolBlockShares", {"all": float, "24h": float, "1w": float})

class PoolDetailInfo(TypedDict):
    """
    Pool information for detail view

    Attributes:
        id: Pool identifier
        name: Pool name
        link: Pool website URL
        addresses: Known payout addresses
        regexes: Coinbase tag patterns (regexes)
        slug: URL-friendly pool identifier
        unique_id: Unique pool identifier
    """
    id: int
    name: str
    link: str
    addresses: List[str]
    regexes: List[str]
    slug: PoolSlug
    unique_id: int

class _PoolDetailRequired(TypedDict):
    pool: PoolDetailInfo
    blockCount: PoolBlockCounts
    blockShare: PoolBlockShares
    estimatedHashrate: int

class PoolDetail(_PoolDetailRequired, total=False):
    """
    Detailed pool information with statistics across time periods

    Attributes:
        pool: Pool information
        blockCount: Block counts for different time periods
        blockShare: Pool's share of total blocks for different time periods
        estimatedHashrate: Estimated hashrate based on blocks mined (H/s)
        reportedHashrate: Self-reported hashrate (if available, H/s)
        totalReward: Total reward earned by this pool (sats, all time; None for minor pools)
    """
    reportedHashrate: Optional[int]
    totalReward: Union[Sats, None]

class PoolHashrateEntry(TypedDict):
    """
    A single pool hashrate data point.

    Attributes:
        timestamp: Unix timestamp
        avgHashrate: Average hashrate (H/s)
        share: Pool's share of total network hashrate (0.0 - 1.0)
        poolName: Pool name
    """
    timestamp: Timestamp
    avgHashrate: int
    share: float
    poolName: str

class PoolInfo(TypedDict):
    """
    Basic pool information for listing all pools

    Attributes:
        name: Pool name
        slug: URL-friendly pool identifier
        unique_id: Unique numeric pool identifier
    """
    name: str
    slug: PoolSlug
    unique_id: int

class PoolSlugAndHeightParam(TypedDict):
    """
    Mining pool slug + block height path parameters
    """
    slug: PoolSlug
    height: Height

class PoolSlugParam(TypedDict):
    """
    Mining pool slug path parameter
    """
    slug: PoolSlug

class PoolStats(TypedDict):
    """
    Mining pool with block statistics for a time period

    Attributes:
        poolId: Unique pool identifier
        name: Pool name
        link: Pool website URL
        blockCount: Number of blocks mined in the time period
        rank: Pool ranking by block count (1 = most blocks)
        emptyBlocks: Number of empty blocks mined
        slug: URL-friendly pool identifier
        share: Pool's share of total blocks (0.0 - 1.0)
        poolUniqueId: Unique pool identifier
    """
    poolId: int
    name: str
    link: str
    blockCount: int
    rank: int
    emptyBlocks: int
    slug: PoolSlug
    share: float
    poolUniqueId: int

class PoolsSummary(TypedDict):
    """
    Mining pools response for a time period

    Attributes:
        pools: List of pools sorted by block count descending
        blockCount: Total blocks in the time period
        lastEstimatedHashrate: Estimated network hashrate (H/s)
        lastEstimatedHashrate3d: Estimated network hashrate over last 3 days (H/s)
        lastEstimatedHashrate1w: Estimated network hashrate over last 1 week (H/s)
    """
    pools: List[PoolStats]
    blockCount: int
    lastEstimatedHashrate: int
    lastEstimatedHashrate3d: int
    lastEstimatedHashrate1w: int

class Prices(TypedDict):
    """
    Current price response matching mempool.space /api/v1/prices format

    Attributes:
        time: Unix timestamp
        USD: BTC/USD price
    """
    time: Timestamp
    USD: Dollars

class _RbfTxRequired(TypedDict):
    txid: Txid
    fee: Sats
    vsize: VSize
    value: Sats
    rate: FeeRate
    time: Timestamp
    rbf: bool

class RbfTx(_RbfTxRequired, total=False):
    """
    Transaction summary carried inside an RBF replacement node. Shape
    matches mempool.space's `/api/v1/tx/:txid/rbf` and
    `/api/v1/replacements` responses.

    Attributes:
        value: Sum of output amounts.
        rbf: BIP-125 signaling: at least one input has sequence < 0xffffffff-1.
        fullRbf: Only populated on the root `tx` of an RBF response. `true` iff this tx displaced at least one non-signaling predecessor.
    """
    fullRbf: Optional[bool]

class _ReplacementNodeRequired(TypedDict):
    tx: RbfTx
    time: Timestamp
    fullRbf: bool
    replaces: List["ReplacementNode"]

class ReplacementNode(_ReplacementNodeRequired, total=False):
    """
    One node in an RBF replacement tree. The node's `tx` replaced each
    entry in `replaces`, recursively.

    Attributes:
        time: First-seen timestamp, duplicated here to match mempool.space's on-the-wire shape.
        fullRbf: Any predecessor in this subtree was non-signaling.
        interval: Seconds between this node's `time` and the successor that replaced it. Omitted on the root of an RBF response.
        mined: `Some(true)` iff this node's tx is currently confirmed. Absent on serialization otherwise.
    """
    interval: Optional[int]
    mined: Optional[bool]

class RbfResponse(TypedDict, total=False):
    """
    Response body for `GET /api/v1/tx/:txid/rbf`. Both fields are null
    when the tx has no known RBF history within the mempool monitor's
    graveyard retention window.
    """
    replacements: Union[ReplacementNode, None]
    replaces: Optional[List[Txid]]

class RecommendedFees(TypedDict):
    """
    Recommended fee rates in sat/vB

    Attributes:
        fastestFee: Fee rate for fastest confirmation (next block)
        halfHourFee: Fee rate for confirmation within ~30 minutes (3 blocks)
        hourFee: Fee rate for confirmation within ~1 hour (6 blocks)
        economyFee: Fee rate for economical confirmation
        minimumFee: Minimum relay fee rate
    """
    fastestFee: FeeRate
    halfHourFee: FeeRate
    hourFee: FeeRate
    economyFee: FeeRate
    minimumFee: FeeRate

class RewardStats(TypedDict):
    """
    Block reward statistics over a range of blocks

    Attributes:
        startBlock: First block in the range
        endBlock: Last block in the range
        totalReward: Total coinbase rewards (subsidy + fees) in sats
        totalFee: Total transaction fees in sats
        totalTx: Total number of transactions
    """
    startBlock: Height
    endBlock: Height
    totalReward: str
    totalFee: str
    totalTx: str

class _SearchQueryRequired(TypedDict):
    q: SeriesName

class SearchQuery(_SearchQueryRequired, total=False):
    """
    Attributes:
        q: Search query string
        limit: Maximum number of results
    """
    limit: Limit

class _SeriesInfoRequired(TypedDict):
    indexes: List[Index]
    nullable: List[Index]
    type: str

class SeriesInfo(_SeriesInfoRequired, total=False):
    """
    Metadata about a series

    Attributes:
        description: Human-readable metric definition, when documented
        indexes: Available indexes
        nullable: Indexes whose values can be null: a missing value (e.g. a period without blocks) or an undefined one (e.g. NaN)
        type: Value type (e.g. "Ratio", "Sats", "Cents")
        unit: What the value type measures, when documented (e.g. "A duration in days.")
    """
    description: Optional[str]
    unit: Optional[str]

class _SeriesLeafWithSchemaRequired(TypedDict):
    name: str
    kind: str
    indexes: List[Index]
    nullable: List[Index]
    type: str

class SeriesLeafWithSchema(_SeriesLeafWithSchemaRequired, total=False):
    """
    Series leaf with JSON Schema for client generation.

    Attributes:
        name: The series name/identifier.
        kind: The Rust type (e.g., "Sats", "Ratio").
        indexes: Available indexes for this series.
        nullable: Indexes whose values can be null: a missing value (e.g. a period without blocks) or an undefined one (e.g. NaN).
        description: Human-readable metric definition, when documented.
        type: JSON Schema type (e.g., "integer", "number", "string", "boolean", "array", "object").
    """
    description: Optional[str]

class SeriesNameWithIndex(TypedDict):
    """
    Attributes:
        series: Series name
        index: Aggregation index
    """
    series: SeriesName
    index: Index

class SeriesParam(TypedDict):
    series: SeriesName

class _SeriesSelectionRequired(TypedDict):
    series: SeriesList
    index: Index

class SeriesSelection(_SeriesSelectionRequired, total=False):
    """
    Selection of series to query

    Attributes:
        series: Requested series
        index: Index to query
        start: Inclusive start: integer index, date (YYYY-MM-DD), or timestamp (ISO 8601). Negative integers count from end. Aliases: `from`, `f`, `s`
        end: Exclusive end: integer index, date (YYYY-MM-DD), or timestamp (ISO 8601). Negative integers count from end. Aliases: `to`, `t`, `e`
        limit: Maximum number of values to return (ignored if `end` is set). Aliases: `count`, `c`, `l`
        format: Format of the output
    """
    start: Union[RangeIndex, None]
    end: Union[RangeIndex, None]
    limit: Union[Limit, None]
    format: Format

class SyncStatus(TypedDict):
    """
    Sync status of the indexer

    Attributes:
        indexed_height: Height of the last indexed block
        computed_height: Height of the last computed block (series)
        tip_height: Height of the chain tip (from Bitcoin node)
        blocks_behind: Number of blocks behind the tip
        last_indexed_at: Human-readable timestamp of the last indexed block (ISO 8601)
        last_indexed_at_unix: Unix timestamp of the last indexed block
    """
    indexed_height: Height
    computed_height: Height
    tip_height: Height
    blocks_behind: Height
    last_indexed_at: str
    last_indexed_at_unix: Timestamp

class TimePeriodParam(TypedDict):
    """
    Time period path parameter (24h, 3d, 1w, 1m, 3m, 6m, 1y, 2y, 3y)
    """
    time_period: TimePeriod

class TimestampParam(TypedDict):
    """
    UNIX timestamp path parameter
    """
    timestamp: Timestamp

class TxIndexParam(TypedDict):
    """
    Transaction index path parameter
    """
    index: TxIndex

class _TxOutspendRequired(TypedDict):
    spent: bool

class TxOutspend(_TxOutspendRequired, total=False):
    """
    Status of an output indicating whether it has been spent

    Attributes:
        spent: Whether the output has been spent
        txid: Transaction ID of the spending transaction (only present if spent)
        vin: Input index in the spending transaction (only present if spent)
        status: Status of the spending transaction (only present if spent)
    """
    txid: Union[Txid, None]
    vin: Union[Vin, None]
    status: Union[TxStatus, None]

class TxidParam(TypedDict):
    """
    Transaction ID path parameter
    """
    txid: Txid

class TxidVout(TypedDict):
    """
    Transaction output reference (txid + output index)

    Attributes:
        txid: Transaction ID
        vout: Output index
    """
    txid: Txid
    vout: Vout

# Query parameter for transaction-times endpoint.
#
# Extracted manually because `serde_urlencoded` (and serde derive in general)
# doesn't support repeated keys like `txId[]=a&txId[]=b`. The schema is still
# declared via `JsonSchema` so the OpenAPI spec lists the parameter and the
# generated client SDKs see `txids: List[Txid]`.
TxidsParam = TypedDict("TxidsParam", {"txId[]": List[Txid]})

class UrpdBucket(TypedDict):
    """
    A single bucket in a URPD snapshot.

    Attributes:
        price_floor: Lower bound of the bucket, in USD. Equals the exact realized price for `Raw`.
        supply: Supply held with a last-move price inside this bucket, in BTC.
        realized_cap: Realized cap contribution in USD: sum of `realized_price * supply` over the coins in this bucket.
        unrealized_pnl: Unrealized P&L in USD against the close on the snapshot date: `close * supply - realized_cap`. Can be negative.
    """
    price_floor: Dollars
    supply: Bitcoin
    realized_cap: Dollars
    unrealized_pnl: Dollars

class Urpd(TypedDict):
    """
    UTXO Realized Price Distribution for a cohort at a specific block.
    
    Supply is grouped by the price at the block in which each UTXO was last moved.
    Each bucket exposes three values: supply in BTC, realized cap contribution
    in USD (sum of `realized_price * supply` over the coins in the bucket), and
    unrealized P&L in USD (`close * supply - realized_cap`, can be negative).

    Attributes:
        date: UTC date of the represented block.
        height: Exact published block represented by this distribution.
        weight: Weighting applied to the source supply.
        aggregation: Aggregation strategy applied to the buckets.
        close: Price at `height`, in USD. Anchor for `unrealized_pnl`.
        total_supply: Sum of `supply` across all buckets, in BTC.
    """
    cohort: Cohort
    date: Date
    height: Height
    weight: UrpdWeight
    aggregation: UrpdAggregation
    close: Dollars
    total_supply: Bitcoin
    buckets: List[UrpdBucket]

class UrpdCohortParam(TypedDict):
    """
    Path parameters for per-cohort URPD endpoints.
    """
    cohort: Cohort

class UrpdParams(TypedDict):
    """
    A URPD cohort and exact block height or UTC calendar-day alias.
    """
    cohort: Cohort
    point: str

class UrpdQuery(TypedDict, total=False):
    """
    Query parameters for URPD endpoints.

    Attributes:
        agg: Aggregation strategy. Default: raw (no aggregation). Accepts `bucket` as alias.
        weight: Supply weighting. Default: raw (unweighted).
    """
    agg: UrpdAggregation
    weight: UrpdWeight

class UrpdWeightQuery(TypedDict, total=False):
    """
    Query parameters for URPD date discovery.

    Attributes:
        weight: Supply weighting. Default: raw (unweighted).
    """
    weight: UrpdWeight

class Utxo(TypedDict):
    """
    Unspent transaction output

    Attributes:
        txid: Transaction ID of the UTXO
        vout: Output index
        status: Confirmation status
        value: Output value in satoshis
    """
    txid: Txid
    vout: Vout
    status: TxStatus
    value: Sats

class ValidateAddrParam(TypedDict):
    """
    Attributes:
        address: Bitcoin address to validate (can be any string)
    """
    address: str


class BitviewError(Exception):
    """A failed request: the HTTP status and the server's error code and message, when it sent them."""

    def __init__(self, message: str, status: Optional[int] = None, code: Optional[ErrorCode] = None):
        super().__init__(message)
        self.status = status
        self.code = code


def _response_error(status: int, data: bytes) -> BitviewError:
    """The server's error body as a BitviewError; a body this client can't read keeps its text."""
    text = data.decode(errors="replace")
    try:
        detail = json.loads(text).get("error")
    except (ValueError, AttributeError):
        detail = None
    if not isinstance(detail, dict):
        detail = {}
    message = detail.get("message")
    return BitviewError(
        message if isinstance(message, str) else (text or f"HTTP {status}"),
        status,
        detail.get("code"),
    )


class BitviewClientBase:
    """Base HTTP client for making requests."""

    def __init__(self, base_url: str, timeout: float = 30.0):
        parsed = urlparse(base_url)
        self._host = parsed.netloc
        self._secure = parsed.scheme == 'https'
        self._timeout = timeout
        self._conn: Optional[Union[HTTPSConnection, HTTPConnection]] = None

    def _connect(self) -> Union[HTTPSConnection, HTTPConnection]:
        """Get or create HTTP connection."""
        if self._conn is None:
            if self._secure:
                self._conn = HTTPSConnection(self._host, timeout=self._timeout)
            else:
                self._conn = HTTPConnection(self._host, timeout=self._timeout)
        return self._conn

    def get(self, path: str) -> bytes:
        """Make a GET request and return raw bytes."""
        try:
            conn = self._connect()
            conn.request("GET", path)
            res = conn.getresponse()
            data = res.read()
            if res.status >= 400:
                raise _response_error(res.status, data)
            return data
        except (ConnectionError, OSError, TimeoutError) as e:
            self._conn = None
            raise BitviewError(str(e))

    def get_json(self, path: str) -> Any:
        """Make a GET request and return JSON."""
        return json.loads(self.get(path))

    def get_text(self, path: str) -> str:
        """Make a GET request and return text."""
        return self.get(path).decode()

    def post(self, path: str, body: str) -> bytes:
        """Make a POST request with a string body and return raw bytes."""
        try:
            conn = self._connect()
            conn.request("POST", path, body=body)
            res = conn.getresponse()
            data = res.read()
            if res.status >= 400:
                raise _response_error(res.status, data)
            return data
        except (ConnectionError, OSError, TimeoutError) as e:
            self._conn = None
            raise BitviewError(str(e))

    def post_json(self, path: str, body: str) -> Any:
        """Make a POST request and return JSON."""
        return json.loads(self.post(path, body))

    def post_text(self, path: str, body: str) -> str:
        """Make a POST request and return text."""
        return self.post(path, body).decode()

    def close(self) -> None:
        """Close the HTTP client."""
        if self._conn:
            self._conn.close()
            self._conn = None

    def __enter__(self) -> BitviewClientBase:
        return self

    def __exit__(self, exc_type: Optional[type], exc_val: Optional[BaseException], exc_tb: Optional[Any]) -> None:
        self.close()




_MASK_64 = (1 << 64) - 1
_RAPIDHASH_SECRETS = (
    0x2d358dccaa6c78a5,
    0x8bb84b93962eacc9,
    0x4b33a62ed433d4a3,
    0x4d5a2da51de1aa47,
    0xa0761d6478bd642f,
    0xe7037ed1a0b428db,
    0x90ed1765281c388c,
)
_RAPIDHASH_SEED = 0


def _u64(value: int) -> int:
    return value & _MASK_64


def _rapid_mix(left: int, right: int) -> int:
    result = _u64(left) * _u64(right)
    return _u64(result) ^ _u64(result >> 64)


def _rapid_mum(left: int, right: int) -> Tuple[int, int]:
    result = _u64(left) * _u64(right)
    return _u64(result), _u64(result >> 64)


def _rapid_hash_seed(seed: int) -> int:
    return _u64(seed ^ _rapid_mix(seed ^ _RAPIDHASH_SECRETS[2], _RAPIDHASH_SECRETS[1]))


_RAPIDHASH_SEED = _rapid_hash_seed(0)


def _read_u32(data: bytes, offset: int) -> int:
    return int.from_bytes(data[offset:offset + 4], "little")


def _read_u64(data: bytes, offset: int) -> int:
    return int.from_bytes(data[offset:offset + 8], "little")


def _rapid_hash_v3(payload: Union[bytes, bytearray, memoryview]) -> int:
    data = bytes(payload)
    length = len(data)
    if length == 0:
        raise ValueError("Expected a non-empty address payload")
    if length > 65:
        raise ValueError("Expected at most 65 address payload bytes")

    seed = _RAPIDHASH_SEED
    a = 0
    b = 0

    if length <= 16:
        if length >= 4:
            seed ^= length
            if length >= 8:
                a ^= _read_u64(data, 0)
                b ^= _read_u64(data, length - 8)
            else:
                a ^= _read_u32(data, 0)
                b ^= _read_u32(data, length - 4)
        elif length > 0:
            a ^= (data[0] << 45) | data[length - 1]
            b ^= data[length >> 1]
        remainder = length
    else:
        if length > 16:
            seed = _rapid_mix(_read_u64(data, 0) ^ _RAPIDHASH_SECRETS[2], _read_u64(data, 8) ^ seed)
            if length > 32:
                seed = _rapid_mix(_read_u64(data, 16) ^ _RAPIDHASH_SECRETS[2], _read_u64(data, 24) ^ seed)
                if length > 48:
                    seed = _rapid_mix(_read_u64(data, 32) ^ _RAPIDHASH_SECRETS[1], _read_u64(data, 40) ^ seed)
                    if length > 64:
                        seed = _rapid_mix(_read_u64(data, 48) ^ _RAPIDHASH_SECRETS[1], _read_u64(data, 56) ^ seed)
        remainder = length
        a ^= _read_u64(data, length - 16) ^ remainder
        b ^= _read_u64(data, length - 8)

    a ^= _RAPIDHASH_SECRETS[1]
    b ^= seed
    a, b = _rapid_mum(a, b)
    return _rapid_mix(a ^ 0xaaaaaaaaaaaaaaaa, b ^ _RAPIDHASH_SECRETS[1] ^ remainder)


def _validate_hash_prefix_nibbles(nibbles: int) -> None:
    if isinstance(nibbles, bool) or not isinstance(nibbles, int) or nibbles < 1 or nibbles > 16:
        raise ValueError("Expected hash-prefix length from 1 to 16 hex nibbles")


def _address_payload_lengths(addr_type: OutputType) -> Tuple[int, ...]:
    if addr_type == "p2a":
        return (2,)
    if addr_type == "p2pk33":
        return (33,)
    if addr_type == "p2pk65":
        return (65,)
    if addr_type in ("p2pkh", "p2sh", "p2wpkh"):
        return (20,)
    if addr_type in ("p2wsh", "p2tr"):
        return (32,)
    raise ValueError(f"Unsupported address type for address payload hash-prefix: {addr_type}")


def _validate_address_payload_for_type(addr_type: OutputType, payload: Union[bytes, bytearray, memoryview]) -> None:
    length = len(bytes(payload))
    expected = _address_payload_lengths(addr_type)
    if length not in expected:
        joined = " or ".join(str(value) for value in expected)
        raise ValueError(f"Expected {addr_type} address payload length {joined} bytes")


def address_payload_hash_prefix(payload: Union[bytes, bytearray, memoryview], nibbles: int) -> str:
    """Compute the RapidHash v3 hash-prefix used by `/api/address/hash-prefix/{addr_type}/{prefix}`."""
    _validate_hash_prefix_nibbles(nibbles)
    return f"{_rapid_hash_v3(payload):016x}"[:nibbles]


# Date conversion constants
# Mirrors the server's indexes: UTC, from 2009-01-01 (week1 buckets ISO weeks, which start three days earlier;
# year10 buckets calendar decades, 2009 alone in the first)
_EPOCH = datetime(2009, 1, 1, tzinfo=timezone.utc)
_EPOCH_DATE = _EPOCH.date()
_DATE_INDEXES = frozenset([
    'minute10', 'minute30',
    'hour1', 'hour4', 'hour12',
    'day1', 'day3', 'week1',
    'month1', 'month3', 'month6',
    'year1', 'year10',
])

def _index_to_date(index: str, i: int) -> Union[date, datetime]:
    """Convert an index value to a date/datetime for date-based indexes."""
    if index == 'minute10':
        return _EPOCH + timedelta(minutes=i * 10)
    elif index == 'minute30':
        return _EPOCH + timedelta(minutes=i * 30)
    elif index == 'hour1':
        return _EPOCH + timedelta(hours=i)
    elif index == 'hour4':
        return _EPOCH + timedelta(hours=i * 4)
    elif index == 'hour12':
        return _EPOCH + timedelta(hours=i * 12)
    elif index == 'day1':
        return _EPOCH_DATE + timedelta(days=i)
    elif index == 'day3':
        return _EPOCH_DATE - timedelta(days=1) + timedelta(days=i * 3)
    elif index == 'week1':
        return _EPOCH_DATE - timedelta(days=3) + timedelta(weeks=i)
    elif index == 'month1':
        return date(2009 + i // 12, i % 12 + 1, 1)
    elif index == 'month3':
        m = i * 3
        return date(2009 + m // 12, m % 12 + 1, 1)
    elif index == 'month6':
        m = i * 6
        return date(2009 + m // 12, m % 12 + 1, 1)
    elif index == 'year1':
        return date(2009 + i, 1, 1)
    elif index == 'year10':
        return date(2009 if i == 0 else 2000 + i * 10, 1, 1)
    else:
        raise ValueError(f"{index} is not a date-based index")


def _date_to_index(index: str, d: Union[date, datetime]) -> int:
    """Convert a date/datetime to an index value for date-based indexes.

    Returns the floor index (latest index whose date is <= the given date).
    For sub-day indexes (minute*, hour*), a plain date is treated as midnight UTC.
    Raises ValueError for dates before the index's first date (2009-01-01; day3's is 2008-12-31, week1's 2008-12-29).
    """
    if index in ('minute10', 'minute30', 'hour1', 'hour4', 'hour12'):
        if isinstance(d, datetime):
            dt = d if d.tzinfo else d.replace(tzinfo=timezone.utc)
        else:
            dt = datetime(d.year, d.month, d.day, tzinfo=timezone.utc)
        if dt < _EPOCH:
            raise ValueError("date is before the first index (2009-01-01)")
        secs = int((dt - _EPOCH).total_seconds())
        div = {'minute10': 600, 'minute30': 1800,
               'hour1': 3600, 'hour4': 14400, 'hour12': 43200}
        return secs // div[index]
    if isinstance(d, datetime):
        d = (d.astimezone(timezone.utc) if d.tzinfo else d).date()
    dd = d
    first = _index_to_date(index, 0)
    if dd < first:
        raise ValueError(f"date is before the first {index} date ({first})")
    if index == 'day1':
        return (dd - _EPOCH_DATE).days
    elif index == 'day3':
        return (dd - date(2008, 12, 31)).days // 3
    elif index == 'week1':
        return ((dd - _EPOCH_DATE).days + 3) // 7
    elif index == 'month1':
        return (dd.year - 2009) * 12 + (dd.month - 1)
    elif index == 'month3':
        return (dd.year - 2009) * 4 + (dd.month - 1) // 3
    elif index == 'month6':
        return (dd.year - 2009) * 2 + (dd.month - 1) // 6
    elif index == 'year1':
        return dd.year - 2009
    elif index == 'year10':
        return (dd.year - 2000) // 10
    else:
        raise ValueError(f"{index} is not a date-based index")


@dataclass
class SeriesData(Generic[T]):
    """Series data with range information. Always int-indexed; a value is None where it is
    missing or undefined."""
    version: int
    index: Index
    type: str
    start: int
    end: int
    stamp: str
    data: List[T]

    @property
    def is_date_based(self) -> bool:
        """Whether this series uses a date-based index."""
        return self.index in _DATE_INDEXES

    def indexes(self) -> List[int]:
        """Get raw index numbers."""
        return list(range(self.start, self.end))

    def keys(self) -> List[int]:
        """Get keys as index numbers."""
        return self.indexes()

    def items(self) -> List[Tuple[int, T]]:
        """Get (index, value) pairs."""
        return list(zip(range(self.start, self.end), self.data))

    def to_dict(self) -> Dict[int, T]:
        """Return {index: value} dict."""
        return dict(zip(range(self.start, self.end), self.data))

    def __iter__(self) -> Iterator[Tuple[int, T]]:
        """Iterate over (index, value) pairs."""
        return zip(range(self.start, self.end), self.data)

    def __len__(self) -> int:
        return len(self.data)

    def to_polars(self) -> pl.DataFrame:
        """Convert to Polars DataFrame with 'index' and 'value' columns."""
        try:
            import polars as pl  # type: ignore[import-not-found]
        except ImportError:
            raise ImportError("polars is required: pip install polars")
        return pl.DataFrame({"index": self.indexes(), "value": self.data})

    def to_pandas(self) -> pd.DataFrame:
        """Convert to Pandas DataFrame with 'index' and 'value' columns."""
        try:
            import pandas as pd  # type: ignore[import-not-found]
        except ImportError:
            raise ImportError("pandas is required: pip install pandas")
        return pd.DataFrame({"index": self.indexes(), "value": self.data})


@dataclass
class DateSeriesData(SeriesData[T]):
    """Series data with date-based index. Extends SeriesData with date methods."""

    def dates(self) -> List[Union[date, datetime]]:
        """Get dates for the index range. Returns datetime for sub-daily indexes, date for daily+."""
        return [_index_to_date(self.index, i) for i in range(self.start, self.end)]

    def date_items(self) -> List[Tuple[Union[date, datetime], T]]:
        """Get (date, value) pairs."""
        return list(zip(self.dates(), self.data))

    def to_date_dict(self) -> Dict[Union[date, datetime], T]:
        """Return {date: value} dict."""
        return dict(zip(self.dates(), self.data))

    def to_polars(self, with_dates: bool = True) -> pl.DataFrame:
        """Convert to Polars DataFrame.

        Returns a DataFrame with columns:
        - 'date' and 'value' if with_dates=True (default)
        - 'index' and 'value' otherwise
        """
        try:
            import polars as pl  # type: ignore[import-not-found]
        except ImportError:
            raise ImportError("polars is required: pip install polars")
        if with_dates:
            return pl.DataFrame({"date": self.dates(), "value": self.data})
        return pl.DataFrame({"index": self.indexes(), "value": self.data})

    def to_pandas(self, with_dates: bool = True) -> pd.DataFrame:
        """Convert to Pandas DataFrame.

        Returns a DataFrame with columns:
        - 'date' and 'value' if with_dates=True (default)
        - 'index' and 'value' otherwise
        """
        try:
            import pandas as pd  # type: ignore[import-not-found]
        except ImportError:
            raise ImportError("pandas is required: pip install pandas")
        if with_dates:
            return pd.DataFrame({"date": self.dates(), "value": self.data})
        return pd.DataFrame({"index": self.indexes(), "value": self.data})


# Type aliases for non-generic usage
AnySeriesData = SeriesData[Any]
AnyDateSeriesData = DateSeriesData[Any]


def _series_data(raw: Dict[str, Any]) -> AnySeriesData:
    """Series data with its helpers (and dates, for a date-based index)."""
    return (DateSeriesData if raw['index'] in _DATE_INDEXES else SeriesData)(**raw)


class _EndpointConfig:
    """Shared endpoint configuration."""
    client: BitviewClient
    name: str
    index: Index
    start: Optional[int]
    end: Optional[int]
    limit: Optional[int]

    def __init__(self, client: BitviewClient, name: str, index: Index,
                 start: Optional[int] = None, end: Optional[int] = None,
                 limit: Optional[int] = None):
        self.client = client
        self.name = name
        self.index = index
        self.start = start
        self.end = end
        self.limit = limit

    def path(self) -> str:
        return f"/api/series/{self.name}/{self.index}"

    def _build_path(self, format: Optional[str] = None) -> str:
        params = []
        if self.start is not None:
            params.append(f"start={self.start}")
        if self.end is not None:
            params.append(f"end={self.end}")
        if self.limit is not None:
            params.append(f"limit={self.limit}")
        if format is not None:
            params.append(f"format={format}")
        query = "&".join(params)
        p = self.path()
        return f"{p}?{query}" if query else p

    def _new(self, start: Optional[int] = None, end: Optional[int] = None,
             limit: Optional[int] = None) -> _EndpointConfig:
        """Counted selections pass `limit`: the server ends them after resolving a negative start."""
        return _EndpointConfig(self.client, self.name, self.index, start, end, limit)

    def get_series(self) -> SeriesData[Any]:
        return SeriesData(**self.client.get_json(self._build_path()))

    def get_date_series(self) -> DateSeriesData[Any]:
        return DateSeriesData(**self.client.get_json(self._build_path()))

    def get_csv(self) -> str:
        return self.client.get_text(self._build_path(format='csv'))

    def get_len(self) -> int:
        return self.client.get_series_len(self.name, self.index)

    def get_version(self) -> Version:
        return self.client.get_series_version(self.name, self.index)


class RangeBuilder(Generic[T]):
    """Builder with range specified."""

    def __init__(self, config: _EndpointConfig):
        self._config = config

    def fetch(self) -> SeriesData[T]:
        """Fetch the range as parsed JSON."""
        return self._config.get_series()

    def fetch_csv(self) -> str:
        """Fetch the range as CSV string."""
        return self._config.get_csv()


class SingleItemBuilder(Generic[T]):
    """Builder for single item access."""

    def __init__(self, config: _EndpointConfig):
        self._config = config

    def fetch(self) -> SeriesData[T]:
        """Fetch the single item."""
        return self._config.get_series()

    def fetch_csv(self) -> str:
        """Fetch as CSV."""
        return self._config.get_csv()


class SkippedBuilder(Generic[T]):
    """Builder after calling skip(n). Chain with take() to specify count."""

    def __init__(self, config: _EndpointConfig):
        self._config = config

    def take(self, n: int) -> RangeBuilder[T]:
        """Take n items after the skipped position."""
        return RangeBuilder(self._config._new(self._config.start, limit=n))

    def fetch(self) -> SeriesData[T]:
        """Fetch from skipped position to end."""
        return self._config.get_series()

    def fetch_csv(self) -> str:
        """Fetch as CSV."""
        return self._config.get_csv()


class DateRangeBuilder(RangeBuilder[T]):
    """Range builder that returns DateSeriesData."""
    def fetch(self) -> DateSeriesData[T]:
        return self._config.get_date_series()


class DateSingleItemBuilder(SingleItemBuilder[T]):
    """Single item builder that returns DateSeriesData."""
    def fetch(self) -> DateSeriesData[T]:
        return self._config.get_date_series()


class DateSkippedBuilder(SkippedBuilder[T]):
    """Skipped builder that returns DateSeriesData."""
    def take(self, n: int) -> DateRangeBuilder[T]:
        return DateRangeBuilder(self._config._new(self._config.start, limit=n))
    def fetch(self) -> DateSeriesData[T]:
        return self._config.get_date_series()


class SeriesEndpoint(Generic[T]):
    """Builder for series endpoint queries with int-based indexing.

    Examples:
        data = endpoint.fetch()
        data = endpoint[5].fetch()
        data = endpoint[:10].fetch()
        data = endpoint.head(20).fetch()
        data = endpoint.skip(100).take(10).fetch()
    """

    def __init__(self, client: BitviewClient, name: str, index: Index):
        self._config = _EndpointConfig(client, name, index)

    @overload
    def __getitem__(self, key: int) -> SingleItemBuilder[T]: ...
    @overload
    def __getitem__(self, key: slice) -> RangeBuilder[T]: ...

    def __getitem__(self, key: Union[int, slice]) -> Union[SingleItemBuilder[T], RangeBuilder[T]]:
        """Access single item or slice by integer index."""
        if isinstance(key, int):
            return SingleItemBuilder(self._config._new(key, limit=1))
        return RangeBuilder(self._config._new(key.start, key.stop))

    def head(self, n: int = 10) -> RangeBuilder[T]:
        """Get the first n items."""
        return RangeBuilder(self._config._new(end=n))

    def tail(self, n: int = 10) -> RangeBuilder[T]:
        """Get the last n items."""
        return RangeBuilder(self._config._new(end=0) if n == 0 else self._config._new(start=-n))

    def skip(self, n: int) -> SkippedBuilder[T]:
        """Skip the first n items."""
        return SkippedBuilder(self._config._new(start=n))

    def fetch(self) -> SeriesData[T]:
        """Fetch all data."""
        return self._config.get_series()

    def fetch_csv(self) -> str:
        """Fetch all data as CSV."""
        return self._config.get_csv()

    def len(self) -> int:
        """Total number of data points for this series."""
        return self._config.get_len()

    def version(self) -> Version:
        """Current version of the series."""
        return self._config.get_version()

    def path(self) -> str:
        """Get the base endpoint path."""
        return self._config.path()


class DateSeriesEndpoint(Generic[T]):
    """Builder for series endpoint queries with date-based indexing.

    Accepts dates in __getitem__ and returns DateSeriesData from fetch().

    Examples:
        data = endpoint.fetch()
        data = endpoint[date(2020, 1, 1)].fetch()
        data = endpoint[date(2020, 1, 1):date(2023, 1, 1)].fetch()
        data = endpoint[:10].fetch()
    """

    def __init__(self, client: BitviewClient, name: str, index: Index):
        self._config = _EndpointConfig(client, name, index)

    @overload
    def __getitem__(self, key: int) -> DateSingleItemBuilder[T]: ...
    @overload
    def __getitem__(self, key: datetime) -> DateSingleItemBuilder[T]: ...
    @overload
    def __getitem__(self, key: date) -> DateSingleItemBuilder[T]: ...
    @overload
    def __getitem__(self, key: slice) -> DateRangeBuilder[T]: ...

    def __getitem__(self, key: Union[int, slice, date, datetime]) -> Union[DateSingleItemBuilder[T], DateRangeBuilder[T]]:
        """Access single item or slice. Accepts int, date, or datetime."""
        if isinstance(key, (date, datetime)):
            idx = _date_to_index(self._config.index, key)
            return DateSingleItemBuilder(self._config._new(idx, limit=1))
        if isinstance(key, int):
            return DateSingleItemBuilder(self._config._new(key, limit=1))
        start, stop = key.start, key.stop
        if isinstance(start, (date, datetime)):
            start = _date_to_index(self._config.index, start)
        if isinstance(stop, (date, datetime)):
            stop = _date_to_index(self._config.index, stop)
        return DateRangeBuilder(self._config._new(start, stop))

    def head(self, n: int = 10) -> DateRangeBuilder[T]:
        """Get the first n items."""
        return DateRangeBuilder(self._config._new(end=n))

    def tail(self, n: int = 10) -> DateRangeBuilder[T]:
        """Get the last n items."""
        return DateRangeBuilder(self._config._new(end=0) if n == 0 else self._config._new(start=-n))

    def skip(self, n: int) -> DateSkippedBuilder[T]:
        """Skip the first n items."""
        return DateSkippedBuilder(self._config._new(start=n))

    def fetch(self) -> DateSeriesData[T]:
        """Fetch all data."""
        return self._config.get_date_series()

    def fetch_csv(self) -> str:
        """Fetch all data as CSV."""
        return self._config.get_csv()

    def len(self) -> int:
        """Total number of data points for this series."""
        return self._config.get_len()

    def version(self) -> Version:
        """Current version of the series."""
        return self._config.get_version()

    def path(self) -> str:
        """Get the base endpoint path."""
        return self._config.path()


# Type aliases for non-generic usage
AnySeriesEndpoint = SeriesEndpoint[Any]
AnyDateSeriesEndpoint = DateSeriesEndpoint[Any]


class SeriesPattern(Protocol[T]):
    """Protocol for series patterns with different index sets."""

    @property
    def name(self) -> str:
        """Get the series name."""
        ...

    def indexes(self) -> List[str]:
        """Get the list of available indexes for this series."""
        ...

    def get(self, index: Index) -> Optional[SeriesEndpoint[T]]:
        """Get an endpoint builder for a specific index, if supported."""
        ...


# Static index tuples
_i1 = ('minute10', 'minute30', 'hour1', 'hour4', 'hour12', 'day1', 'day3', 'week1', 'month1', 'month3', 'month6', 'year1', 'year10', 'halving', 'epoch', 'height')
_i2 = ('minute10', 'minute30', 'hour1', 'hour4', 'hour12', 'day1', 'day3', 'week1', 'month1', 'month3', 'month6', 'year1', 'year10', 'halving', 'epoch', 'height')
_i3 = ('minute10', 'minute30', 'hour1', 'hour4', 'hour12', 'day1', 'day3', 'week1', 'month1', 'month3', 'month6', 'year1', 'year10', 'halving', 'epoch', 'height')
_i4 = ('minute10', 'minute30', 'hour1', 'hour4', 'hour12', 'day1', 'day3', 'week1', 'month1', 'month3', 'month6', 'year1', 'year10', 'halving', 'epoch')
_i5 = ('minute10', 'minute30', 'hour1', 'hour4', 'hour12', 'day1', 'day3', 'week1', 'month1', 'month3', 'month6', 'year1', 'year10', 'halving', 'epoch')
_i6 = ('minute10',)
_i7 = ('minute30',)
_i8 = ('hour1',)
_i9 = ('hour4',)
_i10 = ('hour12',)
_i11 = ('day1',)
_i12 = ('day3',)
_i13 = ('week1',)
_i14 = ('month1',)
_i15 = ('month3',)
_i16 = ('month6',)
_i17 = ('year1',)
_i18 = ('year10',)
_i19 = ('halving',)
_i20 = ('epoch',)
_i21 = ('height',)
_i22 = ('tx_index',)
_i23 = ('txin_index',)
_i24 = ('txout_index',)
_i25 = ('empty_output_index',)
_i26 = ('op_return_index',)
_i27 = ('p2a_addr_index',)
_i28 = ('p2ms_output_index',)
_i29 = ('p2pk33_addr_index',)
_i30 = ('p2pk65_addr_index',)
_i31 = ('p2pkh_addr_index',)
_i32 = ('p2sh_addr_index',)
_i33 = ('p2tr_addr_index',)
_i34 = ('p2wpkh_addr_index',)
_i35 = ('p2wsh_addr_index',)
_i36 = ('unknown_output_index',)
_i37 = ('funded_addr_index',)
_i38 = ('extended_empty_addr_index',)

def _ep(c: BitviewClient, n: str, i: Index) -> SeriesEndpoint[Any]:
    return SeriesEndpoint(c, n, i)

def _dep(c: BitviewClient, n: str, i: Index) -> DateSeriesEndpoint[Any]:
    return DateSeriesEndpoint(c, n, i)

def _endpoint(c: BitviewClient, n: str, i: Index) -> Union[SeriesEndpoint[Any], DateSeriesEndpoint[Any]]:
    """The endpoint for any index: with date helpers for a date-based one."""
    return _dep(c, n, i) if i in _DATE_INDEXES else _ep(c, n, i)

# Index accessor classes

class _SeriesPattern1By(Generic[T]):
    def __init__(self, c: BitviewClient, n: str): self._c, self._n = c, n
    def minute10(self) -> DateSeriesEndpoint[T]: return _dep(self._c, self._n, 'minute10')
    def minute30(self) -> DateSeriesEndpoint[T]: return _dep(self._c, self._n, 'minute30')
    def hour1(self) -> DateSeriesEndpoint[T]: return _dep(self._c, self._n, 'hour1')
    def hour4(self) -> DateSeriesEndpoint[T]: return _dep(self._c, self._n, 'hour4')
    def hour12(self) -> DateSeriesEndpoint[T]: return _dep(self._c, self._n, 'hour12')
    def day1(self) -> DateSeriesEndpoint[T]: return _dep(self._c, self._n, 'day1')
    def day3(self) -> DateSeriesEndpoint[T]: return _dep(self._c, self._n, 'day3')
    def week1(self) -> DateSeriesEndpoint[T]: return _dep(self._c, self._n, 'week1')
    def month1(self) -> DateSeriesEndpoint[T]: return _dep(self._c, self._n, 'month1')
    def month3(self) -> DateSeriesEndpoint[T]: return _dep(self._c, self._n, 'month3')
    def month6(self) -> DateSeriesEndpoint[T]: return _dep(self._c, self._n, 'month6')
    def year1(self) -> DateSeriesEndpoint[T]: return _dep(self._c, self._n, 'year1')
    def year10(self) -> DateSeriesEndpoint[T]: return _dep(self._c, self._n, 'year10')
    def halving(self) -> SeriesEndpoint[T]: return _ep(self._c, self._n, 'halving')
    def epoch(self) -> SeriesEndpoint[T]: return _ep(self._c, self._n, 'epoch')
    def height(self) -> SeriesEndpoint[T]: return _ep(self._c, self._n, 'height')

class SeriesPattern1(Generic[T]):
    by: _SeriesPattern1By[T]
    def __init__(self, c: BitviewClient, n: str): self._n, self.by = n, _SeriesPattern1By(c, n)
    @property
    def name(self) -> str: return self._n
    def indexes(self) -> List[str]: return list(_i1)
    def get(self, index: Index) -> Optional[Union[SeriesEndpoint[T], DateSeriesEndpoint[T]]]: return _endpoint(self.by._c, self._n, index) if index in _i1 else None

class _SeriesPattern2By(Generic[T]):
    def __init__(self, c: BitviewClient, n: str): self._c, self._n = c, n
    def minute10(self) -> DateSeriesEndpoint[Optional[T]]: return _dep(self._c, self._n, 'minute10')
    def minute30(self) -> DateSeriesEndpoint[Optional[T]]: return _dep(self._c, self._n, 'minute30')
    def hour1(self) -> DateSeriesEndpoint[Optional[T]]: return _dep(self._c, self._n, 'hour1')
    def hour4(self) -> DateSeriesEndpoint[Optional[T]]: return _dep(self._c, self._n, 'hour4')
    def hour12(self) -> DateSeriesEndpoint[Optional[T]]: return _dep(self._c, self._n, 'hour12')
    def day1(self) -> DateSeriesEndpoint[Optional[T]]: return _dep(self._c, self._n, 'day1')
    def day3(self) -> DateSeriesEndpoint[Optional[T]]: return _dep(self._c, self._n, 'day3')
    def week1(self) -> DateSeriesEndpoint[Optional[T]]: return _dep(self._c, self._n, 'week1')
    def month1(self) -> DateSeriesEndpoint[Optional[T]]: return _dep(self._c, self._n, 'month1')
    def month3(self) -> DateSeriesEndpoint[Optional[T]]: return _dep(self._c, self._n, 'month3')
    def month6(self) -> DateSeriesEndpoint[Optional[T]]: return _dep(self._c, self._n, 'month6')
    def year1(self) -> DateSeriesEndpoint[Optional[T]]: return _dep(self._c, self._n, 'year1')
    def year10(self) -> DateSeriesEndpoint[Optional[T]]: return _dep(self._c, self._n, 'year10')
    def halving(self) -> SeriesEndpoint[T]: return _ep(self._c, self._n, 'halving')
    def epoch(self) -> SeriesEndpoint[T]: return _ep(self._c, self._n, 'epoch')
    def height(self) -> SeriesEndpoint[T]: return _ep(self._c, self._n, 'height')

class SeriesPattern2(Generic[T]):
    by: _SeriesPattern2By[T]
    def __init__(self, c: BitviewClient, n: str): self._n, self.by = n, _SeriesPattern2By(c, n)
    @property
    def name(self) -> str: return self._n
    def indexes(self) -> List[str]: return list(_i2)
    def get(self, index: Index) -> Optional[Union[SeriesEndpoint[Optional[T]], DateSeriesEndpoint[Optional[T]]]]: return _endpoint(self.by._c, self._n, index) if index in _i2 else None

class _SeriesPattern3By(Generic[T]):
    def __init__(self, c: BitviewClient, n: str): self._c, self._n = c, n
    def minute10(self) -> DateSeriesEndpoint[Optional[T]]: return _dep(self._c, self._n, 'minute10')
    def minute30(self) -> DateSeriesEndpoint[Optional[T]]: return _dep(self._c, self._n, 'minute30')
    def hour1(self) -> DateSeriesEndpoint[Optional[T]]: return _dep(self._c, self._n, 'hour1')
    def hour4(self) -> DateSeriesEndpoint[Optional[T]]: return _dep(self._c, self._n, 'hour4')
    def hour12(self) -> DateSeriesEndpoint[Optional[T]]: return _dep(self._c, self._n, 'hour12')
    def day1(self) -> DateSeriesEndpoint[Optional[T]]: return _dep(self._c, self._n, 'day1')
    def day3(self) -> DateSeriesEndpoint[Optional[T]]: return _dep(self._c, self._n, 'day3')
    def week1(self) -> DateSeriesEndpoint[Optional[T]]: return _dep(self._c, self._n, 'week1')
    def month1(self) -> DateSeriesEndpoint[Optional[T]]: return _dep(self._c, self._n, 'month1')
    def month3(self) -> DateSeriesEndpoint[Optional[T]]: return _dep(self._c, self._n, 'month3')
    def month6(self) -> DateSeriesEndpoint[Optional[T]]: return _dep(self._c, self._n, 'month6')
    def year1(self) -> DateSeriesEndpoint[Optional[T]]: return _dep(self._c, self._n, 'year1')
    def year10(self) -> DateSeriesEndpoint[Optional[T]]: return _dep(self._c, self._n, 'year10')
    def halving(self) -> SeriesEndpoint[Optional[T]]: return _ep(self._c, self._n, 'halving')
    def epoch(self) -> SeriesEndpoint[Optional[T]]: return _ep(self._c, self._n, 'epoch')
    def height(self) -> SeriesEndpoint[Optional[T]]: return _ep(self._c, self._n, 'height')

class SeriesPattern3(Generic[T]):
    by: _SeriesPattern3By[T]
    def __init__(self, c: BitviewClient, n: str): self._n, self.by = n, _SeriesPattern3By(c, n)
    @property
    def name(self) -> str: return self._n
    def indexes(self) -> List[str]: return list(_i3)
    def get(self, index: Index) -> Optional[Union[SeriesEndpoint[Optional[T]], DateSeriesEndpoint[Optional[T]]]]: return _endpoint(self.by._c, self._n, index) if index in _i3 else None

class _SeriesPattern4By(Generic[T]):
    def __init__(self, c: BitviewClient, n: str): self._c, self._n = c, n
    def minute10(self) -> DateSeriesEndpoint[T]: return _dep(self._c, self._n, 'minute10')
    def minute30(self) -> DateSeriesEndpoint[T]: return _dep(self._c, self._n, 'minute30')
    def hour1(self) -> DateSeriesEndpoint[T]: return _dep(self._c, self._n, 'hour1')
    def hour4(self) -> DateSeriesEndpoint[T]: return _dep(self._c, self._n, 'hour4')
    def hour12(self) -> DateSeriesEndpoint[T]: return _dep(self._c, self._n, 'hour12')
    def day1(self) -> DateSeriesEndpoint[T]: return _dep(self._c, self._n, 'day1')
    def day3(self) -> DateSeriesEndpoint[T]: return _dep(self._c, self._n, 'day3')
    def week1(self) -> DateSeriesEndpoint[T]: return _dep(self._c, self._n, 'week1')
    def month1(self) -> DateSeriesEndpoint[T]: return _dep(self._c, self._n, 'month1')
    def month3(self) -> DateSeriesEndpoint[T]: return _dep(self._c, self._n, 'month3')
    def month6(self) -> DateSeriesEndpoint[T]: return _dep(self._c, self._n, 'month6')
    def year1(self) -> DateSeriesEndpoint[T]: return _dep(self._c, self._n, 'year1')
    def year10(self) -> DateSeriesEndpoint[T]: return _dep(self._c, self._n, 'year10')
    def halving(self) -> SeriesEndpoint[T]: return _ep(self._c, self._n, 'halving')
    def epoch(self) -> SeriesEndpoint[T]: return _ep(self._c, self._n, 'epoch')

class SeriesPattern4(Generic[T]):
    by: _SeriesPattern4By[T]
    def __init__(self, c: BitviewClient, n: str): self._n, self.by = n, _SeriesPattern4By(c, n)
    @property
    def name(self) -> str: return self._n
    def indexes(self) -> List[str]: return list(_i4)
    def get(self, index: Index) -> Optional[Union[SeriesEndpoint[T], DateSeriesEndpoint[T]]]: return _endpoint(self.by._c, self._n, index) if index in _i4 else None

class _SeriesPattern5By(Generic[T]):
    def __init__(self, c: BitviewClient, n: str): self._c, self._n = c, n
    def minute10(self) -> DateSeriesEndpoint[Optional[T]]: return _dep(self._c, self._n, 'minute10')
    def minute30(self) -> DateSeriesEndpoint[Optional[T]]: return _dep(self._c, self._n, 'minute30')
    def hour1(self) -> DateSeriesEndpoint[Optional[T]]: return _dep(self._c, self._n, 'hour1')
    def hour4(self) -> DateSeriesEndpoint[Optional[T]]: return _dep(self._c, self._n, 'hour4')
    def hour12(self) -> DateSeriesEndpoint[Optional[T]]: return _dep(self._c, self._n, 'hour12')
    def day1(self) -> DateSeriesEndpoint[Optional[T]]: return _dep(self._c, self._n, 'day1')
    def day3(self) -> DateSeriesEndpoint[Optional[T]]: return _dep(self._c, self._n, 'day3')
    def week1(self) -> DateSeriesEndpoint[Optional[T]]: return _dep(self._c, self._n, 'week1')
    def month1(self) -> DateSeriesEndpoint[Optional[T]]: return _dep(self._c, self._n, 'month1')
    def month3(self) -> DateSeriesEndpoint[Optional[T]]: return _dep(self._c, self._n, 'month3')
    def month6(self) -> DateSeriesEndpoint[Optional[T]]: return _dep(self._c, self._n, 'month6')
    def year1(self) -> DateSeriesEndpoint[Optional[T]]: return _dep(self._c, self._n, 'year1')
    def year10(self) -> DateSeriesEndpoint[Optional[T]]: return _dep(self._c, self._n, 'year10')
    def halving(self) -> SeriesEndpoint[T]: return _ep(self._c, self._n, 'halving')
    def epoch(self) -> SeriesEndpoint[T]: return _ep(self._c, self._n, 'epoch')

class SeriesPattern5(Generic[T]):
    by: _SeriesPattern5By[T]
    def __init__(self, c: BitviewClient, n: str): self._n, self.by = n, _SeriesPattern5By(c, n)
    @property
    def name(self) -> str: return self._n
    def indexes(self) -> List[str]: return list(_i5)
    def get(self, index: Index) -> Optional[Union[SeriesEndpoint[Optional[T]], DateSeriesEndpoint[Optional[T]]]]: return _endpoint(self.by._c, self._n, index) if index in _i5 else None

class _SeriesPattern6By(Generic[T]):
    def __init__(self, c: BitviewClient, n: str): self._c, self._n = c, n
    def minute10(self) -> DateSeriesEndpoint[T]: return _dep(self._c, self._n, 'minute10')

class SeriesPattern6(Generic[T]):
    by: _SeriesPattern6By[T]
    def __init__(self, c: BitviewClient, n: str): self._n, self.by = n, _SeriesPattern6By(c, n)
    @property
    def name(self) -> str: return self._n
    def indexes(self) -> List[str]: return list(_i6)
    def get(self, index: Index) -> Optional[Union[SeriesEndpoint[T], DateSeriesEndpoint[T]]]: return _endpoint(self.by._c, self._n, index) if index in _i6 else None

class _SeriesPattern7By(Generic[T]):
    def __init__(self, c: BitviewClient, n: str): self._c, self._n = c, n
    def minute30(self) -> DateSeriesEndpoint[T]: return _dep(self._c, self._n, 'minute30')

class SeriesPattern7(Generic[T]):
    by: _SeriesPattern7By[T]
    def __init__(self, c: BitviewClient, n: str): self._n, self.by = n, _SeriesPattern7By(c, n)
    @property
    def name(self) -> str: return self._n
    def indexes(self) -> List[str]: return list(_i7)
    def get(self, index: Index) -> Optional[Union[SeriesEndpoint[T], DateSeriesEndpoint[T]]]: return _endpoint(self.by._c, self._n, index) if index in _i7 else None

class _SeriesPattern8By(Generic[T]):
    def __init__(self, c: BitviewClient, n: str): self._c, self._n = c, n
    def hour1(self) -> DateSeriesEndpoint[T]: return _dep(self._c, self._n, 'hour1')

class SeriesPattern8(Generic[T]):
    by: _SeriesPattern8By[T]
    def __init__(self, c: BitviewClient, n: str): self._n, self.by = n, _SeriesPattern8By(c, n)
    @property
    def name(self) -> str: return self._n
    def indexes(self) -> List[str]: return list(_i8)
    def get(self, index: Index) -> Optional[Union[SeriesEndpoint[T], DateSeriesEndpoint[T]]]: return _endpoint(self.by._c, self._n, index) if index in _i8 else None

class _SeriesPattern9By(Generic[T]):
    def __init__(self, c: BitviewClient, n: str): self._c, self._n = c, n
    def hour4(self) -> DateSeriesEndpoint[T]: return _dep(self._c, self._n, 'hour4')

class SeriesPattern9(Generic[T]):
    by: _SeriesPattern9By[T]
    def __init__(self, c: BitviewClient, n: str): self._n, self.by = n, _SeriesPattern9By(c, n)
    @property
    def name(self) -> str: return self._n
    def indexes(self) -> List[str]: return list(_i9)
    def get(self, index: Index) -> Optional[Union[SeriesEndpoint[T], DateSeriesEndpoint[T]]]: return _endpoint(self.by._c, self._n, index) if index in _i9 else None

class _SeriesPattern10By(Generic[T]):
    def __init__(self, c: BitviewClient, n: str): self._c, self._n = c, n
    def hour12(self) -> DateSeriesEndpoint[T]: return _dep(self._c, self._n, 'hour12')

class SeriesPattern10(Generic[T]):
    by: _SeriesPattern10By[T]
    def __init__(self, c: BitviewClient, n: str): self._n, self.by = n, _SeriesPattern10By(c, n)
    @property
    def name(self) -> str: return self._n
    def indexes(self) -> List[str]: return list(_i10)
    def get(self, index: Index) -> Optional[Union[SeriesEndpoint[T], DateSeriesEndpoint[T]]]: return _endpoint(self.by._c, self._n, index) if index in _i10 else None

class _SeriesPattern11By(Generic[T]):
    def __init__(self, c: BitviewClient, n: str): self._c, self._n = c, n
    def day1(self) -> DateSeriesEndpoint[T]: return _dep(self._c, self._n, 'day1')

class SeriesPattern11(Generic[T]):
    by: _SeriesPattern11By[T]
    def __init__(self, c: BitviewClient, n: str): self._n, self.by = n, _SeriesPattern11By(c, n)
    @property
    def name(self) -> str: return self._n
    def indexes(self) -> List[str]: return list(_i11)
    def get(self, index: Index) -> Optional[Union[SeriesEndpoint[T], DateSeriesEndpoint[T]]]: return _endpoint(self.by._c, self._n, index) if index in _i11 else None

class _SeriesPattern12By(Generic[T]):
    def __init__(self, c: BitviewClient, n: str): self._c, self._n = c, n
    def day3(self) -> DateSeriesEndpoint[T]: return _dep(self._c, self._n, 'day3')

class SeriesPattern12(Generic[T]):
    by: _SeriesPattern12By[T]
    def __init__(self, c: BitviewClient, n: str): self._n, self.by = n, _SeriesPattern12By(c, n)
    @property
    def name(self) -> str: return self._n
    def indexes(self) -> List[str]: return list(_i12)
    def get(self, index: Index) -> Optional[Union[SeriesEndpoint[T], DateSeriesEndpoint[T]]]: return _endpoint(self.by._c, self._n, index) if index in _i12 else None

class _SeriesPattern13By(Generic[T]):
    def __init__(self, c: BitviewClient, n: str): self._c, self._n = c, n
    def week1(self) -> DateSeriesEndpoint[T]: return _dep(self._c, self._n, 'week1')

class SeriesPattern13(Generic[T]):
    by: _SeriesPattern13By[T]
    def __init__(self, c: BitviewClient, n: str): self._n, self.by = n, _SeriesPattern13By(c, n)
    @property
    def name(self) -> str: return self._n
    def indexes(self) -> List[str]: return list(_i13)
    def get(self, index: Index) -> Optional[Union[SeriesEndpoint[T], DateSeriesEndpoint[T]]]: return _endpoint(self.by._c, self._n, index) if index in _i13 else None

class _SeriesPattern14By(Generic[T]):
    def __init__(self, c: BitviewClient, n: str): self._c, self._n = c, n
    def month1(self) -> DateSeriesEndpoint[T]: return _dep(self._c, self._n, 'month1')

class SeriesPattern14(Generic[T]):
    by: _SeriesPattern14By[T]
    def __init__(self, c: BitviewClient, n: str): self._n, self.by = n, _SeriesPattern14By(c, n)
    @property
    def name(self) -> str: return self._n
    def indexes(self) -> List[str]: return list(_i14)
    def get(self, index: Index) -> Optional[Union[SeriesEndpoint[T], DateSeriesEndpoint[T]]]: return _endpoint(self.by._c, self._n, index) if index in _i14 else None

class _SeriesPattern15By(Generic[T]):
    def __init__(self, c: BitviewClient, n: str): self._c, self._n = c, n
    def month3(self) -> DateSeriesEndpoint[T]: return _dep(self._c, self._n, 'month3')

class SeriesPattern15(Generic[T]):
    by: _SeriesPattern15By[T]
    def __init__(self, c: BitviewClient, n: str): self._n, self.by = n, _SeriesPattern15By(c, n)
    @property
    def name(self) -> str: return self._n
    def indexes(self) -> List[str]: return list(_i15)
    def get(self, index: Index) -> Optional[Union[SeriesEndpoint[T], DateSeriesEndpoint[T]]]: return _endpoint(self.by._c, self._n, index) if index in _i15 else None

class _SeriesPattern16By(Generic[T]):
    def __init__(self, c: BitviewClient, n: str): self._c, self._n = c, n
    def month6(self) -> DateSeriesEndpoint[T]: return _dep(self._c, self._n, 'month6')

class SeriesPattern16(Generic[T]):
    by: _SeriesPattern16By[T]
    def __init__(self, c: BitviewClient, n: str): self._n, self.by = n, _SeriesPattern16By(c, n)
    @property
    def name(self) -> str: return self._n
    def indexes(self) -> List[str]: return list(_i16)
    def get(self, index: Index) -> Optional[Union[SeriesEndpoint[T], DateSeriesEndpoint[T]]]: return _endpoint(self.by._c, self._n, index) if index in _i16 else None

class _SeriesPattern17By(Generic[T]):
    def __init__(self, c: BitviewClient, n: str): self._c, self._n = c, n
    def year1(self) -> DateSeriesEndpoint[T]: return _dep(self._c, self._n, 'year1')

class SeriesPattern17(Generic[T]):
    by: _SeriesPattern17By[T]
    def __init__(self, c: BitviewClient, n: str): self._n, self.by = n, _SeriesPattern17By(c, n)
    @property
    def name(self) -> str: return self._n
    def indexes(self) -> List[str]: return list(_i17)
    def get(self, index: Index) -> Optional[Union[SeriesEndpoint[T], DateSeriesEndpoint[T]]]: return _endpoint(self.by._c, self._n, index) if index in _i17 else None

class _SeriesPattern18By(Generic[T]):
    def __init__(self, c: BitviewClient, n: str): self._c, self._n = c, n
    def year10(self) -> DateSeriesEndpoint[T]: return _dep(self._c, self._n, 'year10')

class SeriesPattern18(Generic[T]):
    by: _SeriesPattern18By[T]
    def __init__(self, c: BitviewClient, n: str): self._n, self.by = n, _SeriesPattern18By(c, n)
    @property
    def name(self) -> str: return self._n
    def indexes(self) -> List[str]: return list(_i18)
    def get(self, index: Index) -> Optional[Union[SeriesEndpoint[T], DateSeriesEndpoint[T]]]: return _endpoint(self.by._c, self._n, index) if index in _i18 else None

class _SeriesPattern19By(Generic[T]):
    def __init__(self, c: BitviewClient, n: str): self._c, self._n = c, n
    def halving(self) -> SeriesEndpoint[T]: return _ep(self._c, self._n, 'halving')

class SeriesPattern19(Generic[T]):
    by: _SeriesPattern19By[T]
    def __init__(self, c: BitviewClient, n: str): self._n, self.by = n, _SeriesPattern19By(c, n)
    @property
    def name(self) -> str: return self._n
    def indexes(self) -> List[str]: return list(_i19)
    def get(self, index: Index) -> Optional[Union[SeriesEndpoint[T], DateSeriesEndpoint[T]]]: return _endpoint(self.by._c, self._n, index) if index in _i19 else None

class _SeriesPattern20By(Generic[T]):
    def __init__(self, c: BitviewClient, n: str): self._c, self._n = c, n
    def epoch(self) -> SeriesEndpoint[T]: return _ep(self._c, self._n, 'epoch')

class SeriesPattern20(Generic[T]):
    by: _SeriesPattern20By[T]
    def __init__(self, c: BitviewClient, n: str): self._n, self.by = n, _SeriesPattern20By(c, n)
    @property
    def name(self) -> str: return self._n
    def indexes(self) -> List[str]: return list(_i20)
    def get(self, index: Index) -> Optional[Union[SeriesEndpoint[T], DateSeriesEndpoint[T]]]: return _endpoint(self.by._c, self._n, index) if index in _i20 else None

class _SeriesPattern21By(Generic[T]):
    def __init__(self, c: BitviewClient, n: str): self._c, self._n = c, n
    def height(self) -> SeriesEndpoint[T]: return _ep(self._c, self._n, 'height')

class SeriesPattern21(Generic[T]):
    by: _SeriesPattern21By[T]
    def __init__(self, c: BitviewClient, n: str): self._n, self.by = n, _SeriesPattern21By(c, n)
    @property
    def name(self) -> str: return self._n
    def indexes(self) -> List[str]: return list(_i21)
    def get(self, index: Index) -> Optional[Union[SeriesEndpoint[T], DateSeriesEndpoint[T]]]: return _endpoint(self.by._c, self._n, index) if index in _i21 else None

class _SeriesPattern22By(Generic[T]):
    def __init__(self, c: BitviewClient, n: str): self._c, self._n = c, n
    def tx_index(self) -> SeriesEndpoint[T]: return _ep(self._c, self._n, 'tx_index')

class SeriesPattern22(Generic[T]):
    by: _SeriesPattern22By[T]
    def __init__(self, c: BitviewClient, n: str): self._n, self.by = n, _SeriesPattern22By(c, n)
    @property
    def name(self) -> str: return self._n
    def indexes(self) -> List[str]: return list(_i22)
    def get(self, index: Index) -> Optional[Union[SeriesEndpoint[T], DateSeriesEndpoint[T]]]: return _endpoint(self.by._c, self._n, index) if index in _i22 else None

class _SeriesPattern23By(Generic[T]):
    def __init__(self, c: BitviewClient, n: str): self._c, self._n = c, n
    def txin_index(self) -> SeriesEndpoint[T]: return _ep(self._c, self._n, 'txin_index')

class SeriesPattern23(Generic[T]):
    by: _SeriesPattern23By[T]
    def __init__(self, c: BitviewClient, n: str): self._n, self.by = n, _SeriesPattern23By(c, n)
    @property
    def name(self) -> str: return self._n
    def indexes(self) -> List[str]: return list(_i23)
    def get(self, index: Index) -> Optional[Union[SeriesEndpoint[T], DateSeriesEndpoint[T]]]: return _endpoint(self.by._c, self._n, index) if index in _i23 else None

class _SeriesPattern24By(Generic[T]):
    def __init__(self, c: BitviewClient, n: str): self._c, self._n = c, n
    def txout_index(self) -> SeriesEndpoint[T]: return _ep(self._c, self._n, 'txout_index')

class SeriesPattern24(Generic[T]):
    by: _SeriesPattern24By[T]
    def __init__(self, c: BitviewClient, n: str): self._n, self.by = n, _SeriesPattern24By(c, n)
    @property
    def name(self) -> str: return self._n
    def indexes(self) -> List[str]: return list(_i24)
    def get(self, index: Index) -> Optional[Union[SeriesEndpoint[T], DateSeriesEndpoint[T]]]: return _endpoint(self.by._c, self._n, index) if index in _i24 else None

class _SeriesPattern25By(Generic[T]):
    def __init__(self, c: BitviewClient, n: str): self._c, self._n = c, n
    def empty_output_index(self) -> SeriesEndpoint[T]: return _ep(self._c, self._n, 'empty_output_index')

class SeriesPattern25(Generic[T]):
    by: _SeriesPattern25By[T]
    def __init__(self, c: BitviewClient, n: str): self._n, self.by = n, _SeriesPattern25By(c, n)
    @property
    def name(self) -> str: return self._n
    def indexes(self) -> List[str]: return list(_i25)
    def get(self, index: Index) -> Optional[Union[SeriesEndpoint[T], DateSeriesEndpoint[T]]]: return _endpoint(self.by._c, self._n, index) if index in _i25 else None

class _SeriesPattern26By(Generic[T]):
    def __init__(self, c: BitviewClient, n: str): self._c, self._n = c, n
    def op_return_index(self) -> SeriesEndpoint[T]: return _ep(self._c, self._n, 'op_return_index')

class SeriesPattern26(Generic[T]):
    by: _SeriesPattern26By[T]
    def __init__(self, c: BitviewClient, n: str): self._n, self.by = n, _SeriesPattern26By(c, n)
    @property
    def name(self) -> str: return self._n
    def indexes(self) -> List[str]: return list(_i26)
    def get(self, index: Index) -> Optional[Union[SeriesEndpoint[T], DateSeriesEndpoint[T]]]: return _endpoint(self.by._c, self._n, index) if index in _i26 else None

class _SeriesPattern27By(Generic[T]):
    def __init__(self, c: BitviewClient, n: str): self._c, self._n = c, n
    def p2a_addr_index(self) -> SeriesEndpoint[T]: return _ep(self._c, self._n, 'p2a_addr_index')

class SeriesPattern27(Generic[T]):
    by: _SeriesPattern27By[T]
    def __init__(self, c: BitviewClient, n: str): self._n, self.by = n, _SeriesPattern27By(c, n)
    @property
    def name(self) -> str: return self._n
    def indexes(self) -> List[str]: return list(_i27)
    def get(self, index: Index) -> Optional[Union[SeriesEndpoint[T], DateSeriesEndpoint[T]]]: return _endpoint(self.by._c, self._n, index) if index in _i27 else None

class _SeriesPattern28By(Generic[T]):
    def __init__(self, c: BitviewClient, n: str): self._c, self._n = c, n
    def p2ms_output_index(self) -> SeriesEndpoint[T]: return _ep(self._c, self._n, 'p2ms_output_index')

class SeriesPattern28(Generic[T]):
    by: _SeriesPattern28By[T]
    def __init__(self, c: BitviewClient, n: str): self._n, self.by = n, _SeriesPattern28By(c, n)
    @property
    def name(self) -> str: return self._n
    def indexes(self) -> List[str]: return list(_i28)
    def get(self, index: Index) -> Optional[Union[SeriesEndpoint[T], DateSeriesEndpoint[T]]]: return _endpoint(self.by._c, self._n, index) if index in _i28 else None

class _SeriesPattern29By(Generic[T]):
    def __init__(self, c: BitviewClient, n: str): self._c, self._n = c, n
    def p2pk33_addr_index(self) -> SeriesEndpoint[T]: return _ep(self._c, self._n, 'p2pk33_addr_index')

class SeriesPattern29(Generic[T]):
    by: _SeriesPattern29By[T]
    def __init__(self, c: BitviewClient, n: str): self._n, self.by = n, _SeriesPattern29By(c, n)
    @property
    def name(self) -> str: return self._n
    def indexes(self) -> List[str]: return list(_i29)
    def get(self, index: Index) -> Optional[Union[SeriesEndpoint[T], DateSeriesEndpoint[T]]]: return _endpoint(self.by._c, self._n, index) if index in _i29 else None

class _SeriesPattern30By(Generic[T]):
    def __init__(self, c: BitviewClient, n: str): self._c, self._n = c, n
    def p2pk65_addr_index(self) -> SeriesEndpoint[T]: return _ep(self._c, self._n, 'p2pk65_addr_index')

class SeriesPattern30(Generic[T]):
    by: _SeriesPattern30By[T]
    def __init__(self, c: BitviewClient, n: str): self._n, self.by = n, _SeriesPattern30By(c, n)
    @property
    def name(self) -> str: return self._n
    def indexes(self) -> List[str]: return list(_i30)
    def get(self, index: Index) -> Optional[Union[SeriesEndpoint[T], DateSeriesEndpoint[T]]]: return _endpoint(self.by._c, self._n, index) if index in _i30 else None

class _SeriesPattern31By(Generic[T]):
    def __init__(self, c: BitviewClient, n: str): self._c, self._n = c, n
    def p2pkh_addr_index(self) -> SeriesEndpoint[T]: return _ep(self._c, self._n, 'p2pkh_addr_index')

class SeriesPattern31(Generic[T]):
    by: _SeriesPattern31By[T]
    def __init__(self, c: BitviewClient, n: str): self._n, self.by = n, _SeriesPattern31By(c, n)
    @property
    def name(self) -> str: return self._n
    def indexes(self) -> List[str]: return list(_i31)
    def get(self, index: Index) -> Optional[Union[SeriesEndpoint[T], DateSeriesEndpoint[T]]]: return _endpoint(self.by._c, self._n, index) if index in _i31 else None

class _SeriesPattern32By(Generic[T]):
    def __init__(self, c: BitviewClient, n: str): self._c, self._n = c, n
    def p2sh_addr_index(self) -> SeriesEndpoint[T]: return _ep(self._c, self._n, 'p2sh_addr_index')

class SeriesPattern32(Generic[T]):
    by: _SeriesPattern32By[T]
    def __init__(self, c: BitviewClient, n: str): self._n, self.by = n, _SeriesPattern32By(c, n)
    @property
    def name(self) -> str: return self._n
    def indexes(self) -> List[str]: return list(_i32)
    def get(self, index: Index) -> Optional[Union[SeriesEndpoint[T], DateSeriesEndpoint[T]]]: return _endpoint(self.by._c, self._n, index) if index in _i32 else None

class _SeriesPattern33By(Generic[T]):
    def __init__(self, c: BitviewClient, n: str): self._c, self._n = c, n
    def p2tr_addr_index(self) -> SeriesEndpoint[T]: return _ep(self._c, self._n, 'p2tr_addr_index')

class SeriesPattern33(Generic[T]):
    by: _SeriesPattern33By[T]
    def __init__(self, c: BitviewClient, n: str): self._n, self.by = n, _SeriesPattern33By(c, n)
    @property
    def name(self) -> str: return self._n
    def indexes(self) -> List[str]: return list(_i33)
    def get(self, index: Index) -> Optional[Union[SeriesEndpoint[T], DateSeriesEndpoint[T]]]: return _endpoint(self.by._c, self._n, index) if index in _i33 else None

class _SeriesPattern34By(Generic[T]):
    def __init__(self, c: BitviewClient, n: str): self._c, self._n = c, n
    def p2wpkh_addr_index(self) -> SeriesEndpoint[T]: return _ep(self._c, self._n, 'p2wpkh_addr_index')

class SeriesPattern34(Generic[T]):
    by: _SeriesPattern34By[T]
    def __init__(self, c: BitviewClient, n: str): self._n, self.by = n, _SeriesPattern34By(c, n)
    @property
    def name(self) -> str: return self._n
    def indexes(self) -> List[str]: return list(_i34)
    def get(self, index: Index) -> Optional[Union[SeriesEndpoint[T], DateSeriesEndpoint[T]]]: return _endpoint(self.by._c, self._n, index) if index in _i34 else None

class _SeriesPattern35By(Generic[T]):
    def __init__(self, c: BitviewClient, n: str): self._c, self._n = c, n
    def p2wsh_addr_index(self) -> SeriesEndpoint[T]: return _ep(self._c, self._n, 'p2wsh_addr_index')

class SeriesPattern35(Generic[T]):
    by: _SeriesPattern35By[T]
    def __init__(self, c: BitviewClient, n: str): self._n, self.by = n, _SeriesPattern35By(c, n)
    @property
    def name(self) -> str: return self._n
    def indexes(self) -> List[str]: return list(_i35)
    def get(self, index: Index) -> Optional[Union[SeriesEndpoint[T], DateSeriesEndpoint[T]]]: return _endpoint(self.by._c, self._n, index) if index in _i35 else None

class _SeriesPattern36By(Generic[T]):
    def __init__(self, c: BitviewClient, n: str): self._c, self._n = c, n
    def unknown_output_index(self) -> SeriesEndpoint[T]: return _ep(self._c, self._n, 'unknown_output_index')

class SeriesPattern36(Generic[T]):
    by: _SeriesPattern36By[T]
    def __init__(self, c: BitviewClient, n: str): self._n, self.by = n, _SeriesPattern36By(c, n)
    @property
    def name(self) -> str: return self._n
    def indexes(self) -> List[str]: return list(_i36)
    def get(self, index: Index) -> Optional[Union[SeriesEndpoint[T], DateSeriesEndpoint[T]]]: return _endpoint(self.by._c, self._n, index) if index in _i36 else None

class _SeriesPattern37By(Generic[T]):
    def __init__(self, c: BitviewClient, n: str): self._c, self._n = c, n
    def funded_addr_index(self) -> SeriesEndpoint[T]: return _ep(self._c, self._n, 'funded_addr_index')

class SeriesPattern37(Generic[T]):
    by: _SeriesPattern37By[T]
    def __init__(self, c: BitviewClient, n: str): self._n, self.by = n, _SeriesPattern37By(c, n)
    @property
    def name(self) -> str: return self._n
    def indexes(self) -> List[str]: return list(_i37)
    def get(self, index: Index) -> Optional[Union[SeriesEndpoint[T], DateSeriesEndpoint[T]]]: return _endpoint(self.by._c, self._n, index) if index in _i37 else None

class _SeriesPattern38By(Generic[T]):
    def __init__(self, c: BitviewClient, n: str): self._c, self._n = c, n
    def extended_empty_addr_index(self) -> SeriesEndpoint[T]: return _ep(self._c, self._n, 'extended_empty_addr_index')

class SeriesPattern38(Generic[T]):
    by: _SeriesPattern38By[T]
    def __init__(self, c: BitviewClient, n: str): self._n, self.by = n, _SeriesPattern38By(c, n)
    @property
    def name(self) -> str: return self._n
    def indexes(self) -> List[str]: return list(_i38)
    def get(self, index: Index) -> Optional[Union[SeriesEndpoint[T], DateSeriesEndpoint[T]]]: return _endpoint(self.by._c, self._n, index) if index in _i38 else None

# Series tree

class _Child:
    """A series-tree child, materialized on first access. Its series name (a leaf) or base (a node)
    is the template with `*` replaced by the parent's base, dropping the joining `_` when the base
    is empty. It is made from (client, name) by an accessor or node class, by the index of one of
    the parent's builders, or by a tuple applying a node class to builders."""

    def __init__(self, make: Any, template: str):
        self._make, self._template = make, template

    def __set_name__(self, owner: type, key: str) -> None:
        self._key = key

    def __get__(self, node: Any, owner: Any = None) -> Any:
        if node is None:
            return self
        base = node._b
        if base:
            name = self._template.replace('*', base)
        else:
            name = re.sub(r'^\*_|_?\*', '', self._template, count=1)
        value = _builder(node, self._make)(node._c, name)
        node.__dict__[self._key] = value
        return value


def _builder(node: _Node, make: Any) -> Callable[[BitviewClient, str], Any]:
    if isinstance(make, int):
        return node._f[make]
    if isinstance(make, tuple):
        cls, *args = make
        builders = [_builder(node, arg) for arg in args]
        return lambda c, b: cls(c, b, *builders)
    return make


def _at(make: Any, template: str) -> Any:
    return _Child(make, template)


class _Node:
    """A series-tree node over base `_b`, built with one builder per shape parameter."""

    def __init__(self, c: BitviewClient, b: str, *f: Callable[[BitviewClient, str], Any]):
        self._c, self._b, self._f = c, b, f

A = TypeVar('A')
B = TypeVar('B')
C = TypeVar('C')
D = TypeVar('D')

class UtxoHistory(_Node):
    supply: SeriesPattern21[Sats] = _at(SeriesPattern21, '*')
    count: SeriesPattern2[Count] = _at(SeriesPattern2, 'utxo_count_bis')


class Velocity(_Node):
    native: SeriesPattern2[Optional[Ratio64]] = _at(SeriesPattern2, '*_btc')
    fiat: SeriesPattern2[Optional[Ratio64]] = _at(SeriesPattern2, '*_usd')


class SoprRatioExtended(_Node):
    _1w: SeriesPattern2[Optional[Ratio]] = _at(SeriesPattern2, '*_1w')
    _1m: SeriesPattern2[Optional[Ratio]] = _at(SeriesPattern2, '*_1m')
    _1y: SeriesPattern2[Optional[Ratio]] = _at(SeriesPattern2, '*_1y')


class Close(_Node):
    usd: SeriesPattern5[Optional[Dollars]] = _at(SeriesPattern5, '*')
    cents: SeriesPattern5[Optional[Cents]] = _at(SeriesPattern5, '*_cents')
    sats: SeriesPattern5[Sats] = _at(SeriesPattern5, '*_sats')


class Ohlc(_Node, Generic[A, B, C]):
    usd: SeriesPattern4[A] = _at(SeriesPattern4, '*')
    cents: SeriesPattern4[B] = _at(SeriesPattern4, '*_cents')
    sats: SeriesPattern4[C] = _at(SeriesPattern4, '*_sats')


class Split(_Node):
    open: Ohlc[Optional[Dollars], Optional[Cents], Sats] = _at(Ohlc, '*_open')
    high: Ohlc[Optional[Dollars], Optional[Cents], Sats] = _at(Ohlc, '*_high')
    low: Ohlc[Optional[Dollars], Optional[Cents], Sats] = _at(Ohlc, '*_low')
    close: Close = _at(Close, '*_close')


class Macd1m(_Node):
    ema_fast: SeriesPattern2[Optional[Dollars]] = _at(SeriesPattern2, 'macd_ema_fast_*')
    ema_slow: SeriesPattern2[Optional[Dollars]] = _at(SeriesPattern2, 'macd_ema_slow_*')
    line: SeriesPattern2[Optional[Dollars]] = _at(SeriesPattern2, 'macd_line_*')
    signal: SeriesPattern2[Optional[Dollars]] = _at(SeriesPattern2, 'macd_signal_*')
    histogram: SeriesPattern2[Optional[Dollars]] = _at(SeriesPattern2, 'macd_histogram_*')


class Sd24h1m(_Node):
    sma: SeriesPattern2[Optional[Ratio]] = _at(SeriesPattern2, 'price_return_24h_sma_*')
    sd: SeriesPattern2[Optional[Ratio]] = _at(SeriesPattern2, 'price_return_24h_sd_*')


class Dormancy(_Node):
    supply_adj: SeriesPattern2[Optional[Float32]] = _at(SeriesPattern2, '*_supply_adj')
    flow: SeriesPattern2[Optional[Float32]] = _at(SeriesPattern2, '*_flow')


class Nvt(_Node):
    bps: SeriesPattern2[Optional[BasisPoints32]] = _at(SeriesPattern2, '*_bps')
    ratio: SeriesPattern2[Optional[Ratio]] = _at(SeriesPattern2, '*')


class MappingsTimestamp(_Node):
    monotonic: SeriesPattern21[Timestamp] = _at(SeriesPattern21, '*_monotonic')
    resolutions: SeriesPattern4[Timestamp] = _at(SeriesPattern4, '*')


class TxoutIndex(_Node):
    identity: SeriesPattern24[TxOutIndex] = _at(SeriesPattern24, '*')


class TxinIndex(_Node):
    identity: SeriesPattern23[TxInIndex] = _at(SeriesPattern23, '*')


class MappingsTxIndex(_Node):
    identity: SeriesPattern22[TxIndex] = _at(SeriesPattern22, 'tx_index')
    input_count: SeriesPattern22[Count] = _at(SeriesPattern22, 'input_*')
    output_count: SeriesPattern22[Count] = _at(SeriesPattern22, 'output_*')


class MappingsYear10(_Node):
    date: SeriesPattern18[Date] = _at(SeriesPattern18, '*')
    first_height: SeriesPattern18[Height] = _at(SeriesPattern18, 'first_height')


class MappingsYear1(_Node):
    date: SeriesPattern17[Date] = _at(SeriesPattern17, '*')
    first_height: SeriesPattern17[Height] = _at(SeriesPattern17, 'first_height')


class MappingsMonth6(_Node):
    date: SeriesPattern16[Date] = _at(SeriesPattern16, '*')
    first_height: SeriesPattern16[Height] = _at(SeriesPattern16, 'first_height')


class MappingsMonth3(_Node):
    date: SeriesPattern15[Date] = _at(SeriesPattern15, '*')
    first_height: SeriesPattern15[Height] = _at(SeriesPattern15, 'first_height')


class MappingsMonth1(_Node):
    date: SeriesPattern14[Date] = _at(SeriesPattern14, '*')
    first_height: SeriesPattern14[Height] = _at(SeriesPattern14, 'first_height')


class MappingsWeek1(_Node):
    date: SeriesPattern13[Date] = _at(SeriesPattern13, '*')
    first_height: SeriesPattern13[Height] = _at(SeriesPattern13, 'first_height')


class MappingsDay3(_Node):
    date: SeriesPattern12[Date] = _at(SeriesPattern12, '*')
    first_height: SeriesPattern12[Height] = _at(SeriesPattern12, 'first_height')


class MappingsDay1(_Node):
    date: SeriesPattern11[Date] = _at(SeriesPattern11, '*')
    first_height: SeriesPattern11[Height] = _at(SeriesPattern11, 'first_height')


class MappingsHour12(_Node):
    first_height: SeriesPattern10[Height] = _at(SeriesPattern10, '*')


class MappingsHour4(_Node):
    first_height: SeriesPattern9[Height] = _at(SeriesPattern9, '*')


class MappingsHour1(_Node):
    first_height: SeriesPattern8[Height] = _at(SeriesPattern8, '*')


class MappingsMinute30(_Node):
    first_height: SeriesPattern7[Height] = _at(SeriesPattern7, '*')


class MappingsMinute10(_Node):
    first_height: SeriesPattern6[Height] = _at(SeriesPattern6, '*')


class MappingsHalving(_Node):
    first_height: SeriesPattern19[Height] = _at(SeriesPattern19, '*')


class MappingsEpoch(_Node):
    first_height: SeriesPattern20[Height] = _at(SeriesPattern20, '*')


class MappingsHeight(_Node):
    minute10: SeriesPattern21[Minute10] = _at(SeriesPattern21, '*')
    minute30: SeriesPattern21[Minute30] = _at(SeriesPattern21, 'minute30')
    hour1: SeriesPattern21[Hour1] = _at(SeriesPattern21, 'hour1')
    hour4: SeriesPattern21[Hour4] = _at(SeriesPattern21, 'hour4')
    hour12: SeriesPattern21[Hour12] = _at(SeriesPattern21, 'hour12')
    day1: SeriesPattern21[Day1] = _at(SeriesPattern21, 'day1')
    day3: SeriesPattern21[Day3] = _at(SeriesPattern21, 'day3')
    epoch: SeriesPattern21[Epoch] = _at(SeriesPattern21, 'epoch')
    halving: SeriesPattern21[Halving] = _at(SeriesPattern21, 'halving')
    week1: SeriesPattern21[Week1] = _at(SeriesPattern21, 'week1')
    month1: SeriesPattern21[Month1] = _at(SeriesPattern21, 'month1')
    month3: SeriesPattern21[Month3] = _at(SeriesPattern21, 'month3')
    month6: SeriesPattern21[Month6] = _at(SeriesPattern21, 'month6')
    year1: SeriesPattern21[Year1] = _at(SeriesPattern21, 'year1')
    year10: SeriesPattern21[Year10] = _at(SeriesPattern21, 'year10')
    tx_index_count: SeriesPattern21[Count] = _at(SeriesPattern21, 'tx_index_count')


class AddrOpReturn(_Node):
    identity: SeriesPattern26[OpReturnIndex] = _at(SeriesPattern26, '*')


class AddrUnknown(_Node):
    identity: SeriesPattern36[UnknownOutputIndex] = _at(SeriesPattern36, '*')


class AddrEmpty(_Node):
    identity: SeriesPattern25[EmptyOutputIndex] = _at(SeriesPattern25, '*')


class AddrP2ms(_Node):
    identity: SeriesPattern28[P2MSOutputIndex] = _at(SeriesPattern28, '*')


class AddrP2a(_Node):
    identity: SeriesPattern27[P2AAddrIndex] = _at(SeriesPattern27, '*_index')
    addr: SeriesPattern27[Addr] = _at(SeriesPattern27, '*')


class AddrP2wsh(_Node):
    identity: SeriesPattern35[P2WSHAddrIndex] = _at(SeriesPattern35, '*_index')
    addr: SeriesPattern35[Addr] = _at(SeriesPattern35, '*')


class AddrP2wpkh(_Node):
    identity: SeriesPattern34[P2WPKHAddrIndex] = _at(SeriesPattern34, '*_index')
    addr: SeriesPattern34[Addr] = _at(SeriesPattern34, '*')


class AddrP2tr(_Node):
    identity: SeriesPattern33[P2TRAddrIndex] = _at(SeriesPattern33, '*_index')
    addr: SeriesPattern33[Addr] = _at(SeriesPattern33, '*')


class AddrP2sh(_Node):
    identity: SeriesPattern32[P2SHAddrIndex] = _at(SeriesPattern32, '*_index')
    addr: SeriesPattern32[Addr] = _at(SeriesPattern32, '*')


class AddrP2pkh(_Node):
    identity: SeriesPattern31[P2PKHAddrIndex] = _at(SeriesPattern31, '*_index')
    addr: SeriesPattern31[Addr] = _at(SeriesPattern31, '*')


class AddrP2pk65(_Node):
    identity: SeriesPattern30[P2PK65AddrIndex] = _at(SeriesPattern30, '*_index')
    addr: SeriesPattern30[Addr] = _at(SeriesPattern30, '*')


class AddrP2pk33(_Node):
    identity: SeriesPattern29[P2PK33AddrIndex] = _at(SeriesPattern29, '*_index')
    addr: SeriesPattern29[Addr] = _at(SeriesPattern29, '*')


class MappingsAddr(_Node):
    p2pk33: AddrP2pk33 = _at(AddrP2pk33, 'p2pk33_*')
    p2pk65: AddrP2pk65 = _at(AddrP2pk65, 'p2pk65_*')
    p2pkh: AddrP2pkh = _at(AddrP2pkh, 'p2pkh_*')
    p2sh: AddrP2sh = _at(AddrP2sh, 'p2sh_*')
    p2tr: AddrP2tr = _at(AddrP2tr, 'p2tr_*')
    p2wpkh: AddrP2wpkh = _at(AddrP2wpkh, 'p2wpkh_*')
    p2wsh: AddrP2wsh = _at(AddrP2wsh, 'p2wsh_*')
    p2a: AddrP2a = _at(AddrP2a, 'p2a_*')
    p2ms: AddrP2ms = _at(AddrP2ms, 'p2ms_output_index')
    empty: AddrEmpty = _at(AddrEmpty, 'empty_output_index')
    unknown: AddrUnknown = _at(AddrUnknown, 'unknown_output_index')
    op_return: AddrOpReturn = _at(AddrOpReturn, 'op_return_index')


class Mappings(_Node):
    addr: MappingsAddr = _at(MappingsAddr, 'addr')
    height: MappingsHeight = _at(MappingsHeight, 'minute10')
    epoch: MappingsEpoch = _at(MappingsEpoch, 'first_height')
    halving: MappingsHalving = _at(MappingsHalving, 'first_height')
    minute10: MappingsMinute10 = _at(MappingsMinute10, 'first_height')
    minute30: MappingsMinute30 = _at(MappingsMinute30, 'first_height')
    hour1: MappingsHour1 = _at(MappingsHour1, 'first_height')
    hour4: MappingsHour4 = _at(MappingsHour4, 'first_height')
    hour12: MappingsHour12 = _at(MappingsHour12, 'first_height')
    day1: MappingsDay1 = _at(MappingsDay1, '*')
    day3: MappingsDay3 = _at(MappingsDay3, '*')
    week1: MappingsWeek1 = _at(MappingsWeek1, '*')
    month1: MappingsMonth1 = _at(MappingsMonth1, '*')
    month3: MappingsMonth3 = _at(MappingsMonth3, '*')
    month6: MappingsMonth6 = _at(MappingsMonth6, '*')
    year1: MappingsYear1 = _at(MappingsYear1, '*')
    year10: MappingsYear10 = _at(MappingsYear10, '*')
    tx_index: MappingsTxIndex = _at(MappingsTxIndex, 'count')
    txin_index: TxinIndex = _at(TxinIndex, 'txin_index')
    txout_index: TxoutIndex = _at(TxoutIndex, 'txout_index')
    timestamp: MappingsTimestamp = _at(MappingsTimestamp, 'timestamp')


class Constants(_Node):
    _0: SeriesPattern1[Optional[Float32]] = _at(SeriesPattern1, '*_0')
    _1: SeriesPattern1[Optional[Float32]] = _at(SeriesPattern1, '*_1')
    _2: SeriesPattern1[Optional[Float32]] = _at(SeriesPattern1, '*_2')
    _3: SeriesPattern1[Optional[Float32]] = _at(SeriesPattern1, '*_3')
    _4: SeriesPattern1[Optional[Float32]] = _at(SeriesPattern1, '*_4')
    _20: SeriesPattern1[Optional[Float32]] = _at(SeriesPattern1, '*_20')
    _30: SeriesPattern1[Optional[Float32]] = _at(SeriesPattern1, '*_30')
    _38_2: SeriesPattern1[Optional[Float32]] = _at(SeriesPattern1, '*_38_2')
    _50: SeriesPattern1[Optional[Float32]] = _at(SeriesPattern1, '*_50')
    _61_8: SeriesPattern1[Optional[Float32]] = _at(SeriesPattern1, '*_61_8')
    _70: SeriesPattern1[Optional[Float32]] = _at(SeriesPattern1, '*_70')
    _80: SeriesPattern1[Optional[Float32]] = _at(SeriesPattern1, '*_80')
    _100: SeriesPattern1[Optional[Float32]] = _at(SeriesPattern1, '*_100')
    _600: SeriesPattern1[Optional[Float32]] = _at(SeriesPattern1, '*_600')
    minus_1: SeriesPattern1[Optional[Float32]] = _at(SeriesPattern1, '*_minus_1')
    minus_2: SeriesPattern1[Optional[Float32]] = _at(SeriesPattern1, '*_minus_2')
    minus_3: SeriesPattern1[Optional[Float32]] = _at(SeriesPattern1, '*_minus_3')
    minus_4: SeriesPattern1[Optional[Float32]] = _at(SeriesPattern1, '*_minus_4')


class CapitalSentiment(_Node):
    is_long: SeriesPattern2[Boolean] = _at(SeriesPattern2, '*_is_long')
    is_short: SeriesPattern2[Boolean] = _at(SeriesPattern2, '*_is_short')
    phase: SeriesPattern3[CapitalSentimentPhase] = _at(SeriesPattern3, '*_phase')
    score: SeriesPattern3[Score] = _at(SeriesPattern3, '*_score')


class SupplyInLossThreshold(_Node):
    pct95: SeriesPattern2[Optional[Ratio64]] = _at(SeriesPattern2, '*_pct95_ratio')
    pct98: SeriesPattern2[Optional[Ratio64]] = _at(SeriesPattern2, '*_pct98_ratio')
    pct99: SeriesPattern2[Optional[Ratio64]] = _at(SeriesPattern2, '*_pct99_ratio')
    pct99_5: SeriesPattern2[Optional[Ratio64]] = _at(SeriesPattern2, '*_pct99_5_ratio')
    pct99_9: SeriesPattern2[Optional[Ratio64]] = _at(SeriesPattern2, '*_pct99_9_ratio')


class ReserveRisk(_Node):
    value: SeriesPattern2[Optional[Float64]] = _at(SeriesPattern2, '*')
    vocdd_median_1y: SeriesPattern21[Optional[Float64]] = _at(SeriesPattern21, 'vocdd_median_1y')
    hodl_bank: SeriesPattern21[Optional[Float64]] = _at(SeriesPattern21, 'hodl_bank')


class RhodlRatio(_Node, Generic[A]):
    ppm: SeriesPattern2[A] = _at(SeriesPattern2, '*_ppm')
    ratio: SeriesPattern2[Optional[Ratio]] = _at(SeriesPattern2, '*')


class Share(_Node):
    bounded: SeriesPattern2[Optional[BoundedRatio]] = _at(SeriesPattern2, '*_bounded')
    ratio: SeriesPattern2[Optional[Ratio64]] = _at(SeriesPattern2, '*')


class ActiveInLoss(_Node):
    share: Share = _at(Share, '*')


class Active(_Node):
    btc: SeriesPattern2[Optional[Bitcoin]] = _at(SeriesPattern2, 'active_*')
    sats: SeriesPattern2[Sats] = _at(SeriesPattern2, 'active_*_sats')
    usd: SeriesPattern2[Optional[Dollars]] = _at(SeriesPattern2, 'active_*_usd')
    cents: SeriesPattern2[Optional[Cents]] = _at(SeriesPattern2, 'active_*_cents')
    in_loss: ActiveInLoss = _at(ActiveInLoss, 'cointime_*_in_loss_share')


class MobileInLoss(_Node):
    share: SeriesPattern2[Optional[Ratio64]] = _at(SeriesPattern2, '*')


class Mobile(_Node):
    btc: SeriesPattern2[Optional[Bitcoin]] = _at(SeriesPattern2, '*_mobile_supply')
    sats: SeriesPattern2[Sats] = _at(SeriesPattern2, '*_mobile_supply_sats')
    usd: SeriesPattern2[Optional[Dollars]] = _at(SeriesPattern2, '*_mobile_supply_usd')
    cents: SeriesPattern2[Optional[Cents]] = _at(SeriesPattern2, '*_mobile_supply_cents')
    in_loss: MobileInLoss = _at(MobileInLoss, '*_coinflow_supply_in_loss_share')


class AwakeSupply(_Node):
    btc: SeriesPattern2[Optional[Bitcoin]] = _at(SeriesPattern2, '*')
    sats: SeriesPattern2[Sats] = _at(SeriesPattern2, '*_sats')
    usd: SeriesPattern2[Optional[Dollars]] = _at(SeriesPattern2, '*_usd')
    cents: SeriesPattern2[Optional[Cents]] = _at(SeriesPattern2, '*_cents')
    in_loss: MobileInLoss = _at(MobileInLoss, '*_in_loss_share')


class CapitalizedPrice(_Node):
    usd: SeriesPattern2[Optional[Dollars]] = _at(SeriesPattern2, '*')
    cents: SeriesPattern2[Optional[Cents]] = _at(SeriesPattern2, '*_cents')
    sats: SeriesPattern2[Optional[SatsFract]] = _at(SeriesPattern2, '*_sats')
    ppm: SeriesPattern2[Optional[PriceRatio]] = _at(SeriesPattern2, '*_ratio_ppm')
    ratio: SeriesPattern2[Optional[Ratio]] = _at(SeriesPattern2, '*_ratio')


class Ema(_Node):
    _1w: CapitalizedPrice = _at(CapitalizedPrice, '*_1w')
    _8d: CapitalizedPrice = _at(CapitalizedPrice, '*_8d')
    _12d: CapitalizedPrice = _at(CapitalizedPrice, '*_12d')
    _13d: CapitalizedPrice = _at(CapitalizedPrice, '*_13d')
    _21d: CapitalizedPrice = _at(CapitalizedPrice, '*_21d')
    _26d: CapitalizedPrice = _at(CapitalizedPrice, '*_26d')
    _1m: CapitalizedPrice = _at(CapitalizedPrice, '*_1m')
    _34d: CapitalizedPrice = _at(CapitalizedPrice, '*_34d')
    _55d: CapitalizedPrice = _at(CapitalizedPrice, '*_55d')
    _89d: CapitalizedPrice = _at(CapitalizedPrice, '*_89d')
    _144d: CapitalizedPrice = _at(CapitalizedPrice, '*_144d')
    _200d: CapitalizedPrice = _at(CapitalizedPrice, '*_200d')
    _1y: CapitalizedPrice = _at(CapitalizedPrice, '*_1y')
    _2y: CapitalizedPrice = _at(CapitalizedPrice, '*_2y')
    _200w: CapitalizedPrice = _at(CapitalizedPrice, '*_200w')
    _4y: CapitalizedPrice = _at(CapitalizedPrice, '*_4y')


class CointimePrices(_Node):
    vaulted: CapitalizedPrice = _at(CapitalizedPrice, 'vaulted_*')
    active: CapitalizedPrice = _at(CapitalizedPrice, 'active_*')
    true_market_mean: CapitalizedPrice = _at(CapitalizedPrice, 'true_market_mean')
    cointime: CapitalizedPrice = _at(CapitalizedPrice, 'cointime_*')


class Spot(_Node, Generic[A]):
    usd: SeriesPattern2[Optional[Dollars]] = _at(SeriesPattern2, '*')
    cents: SeriesPattern2[Optional[Cents]] = _at(SeriesPattern2, '*_cents')
    sats: SeriesPattern2[A] = _at(SeriesPattern2, '*_sats')


class AgeBoundsAll(_Node):
    min: Spot[Optional[SatsFract]] = _at(Spot, '*_min')
    max: Spot[Optional[SatsFract]] = _at(Spot, '*_max')


class AgeBounds(_Node):
    all: AgeBoundsAll = _at(AgeBoundsAll, '*_all_cost_basis')
    sth: AgeBoundsAll = _at(AgeBoundsAll, '*_sth_cost_basis')
    lth: AgeBoundsAll = _at(AgeBoundsAll, '*_lth_cost_basis')
    under_4m: AgeBoundsAll = _at(AgeBoundsAll, '*_under_4m_cost_basis')
    under_6m: AgeBoundsAll = _at(AgeBoundsAll, '*_under_6m_cost_basis')
    over_4m: AgeBoundsAll = _at(AgeBoundsAll, '*_over_4m_cost_basis')
    over_6m: AgeBoundsAll = _at(AgeBoundsAll, '*_over_6m_cost_basis')


class CohortsUrpd(_Node):
    age_bounds: AgeBounds = _at(AgeBounds, '*')


class Price(_Node):
    split: Split = _at(Split, '*')
    ohlc: Ohlc[OHLCDollars, OHLCCents, OHLCSats] = _at(Ohlc, '*_ohlc')
    spot: Spot[Sats] = _at(Spot, '*')


class Sma350d(_Node):
    usd: SeriesPattern2[Optional[Dollars]] = _at(SeriesPattern2, '*')
    cents: SeriesPattern2[Optional[Cents]] = _at(SeriesPattern2, '*_cents')
    sats: SeriesPattern2[Optional[SatsFract]] = _at(SeriesPattern2, '*_sats')
    ppm: SeriesPattern2[Optional[PriceRatio]] = _at(SeriesPattern2, '*_ratio_ppm')
    ratio: SeriesPattern2[Optional[Ratio]] = _at(SeriesPattern2, '*_ratio')
    x2: Spot[Optional[SatsFract]] = _at(Spot, '*_x2')


class Sma200d(_Node):
    usd: SeriesPattern2[Optional[Dollars]] = _at(SeriesPattern2, '*')
    cents: SeriesPattern2[Optional[Cents]] = _at(SeriesPattern2, '*_cents')
    sats: SeriesPattern2[Optional[SatsFract]] = _at(SeriesPattern2, '*_sats')
    ppm: SeriesPattern2[Optional[PriceRatio]] = _at(SeriesPattern2, '*_ratio_ppm')
    ratio: SeriesPattern2[Optional[Ratio]] = _at(SeriesPattern2, '*_ratio')
    x2_4: Spot[Optional[SatsFract]] = _at(Spot, '*_x2_4')
    x0_8: Spot[Optional[SatsFract]] = _at(Spot, '*_x0_8')


class MovingAverageSma(_Node):
    _1w: CapitalizedPrice = _at(CapitalizedPrice, '*_1w')
    _8d: CapitalizedPrice = _at(CapitalizedPrice, '*_8d')
    _13d: CapitalizedPrice = _at(CapitalizedPrice, '*_13d')
    _21d: CapitalizedPrice = _at(CapitalizedPrice, '*_21d')
    _1m: CapitalizedPrice = _at(CapitalizedPrice, '*_1m')
    _34d: CapitalizedPrice = _at(CapitalizedPrice, '*_34d')
    _50d: CapitalizedPrice = _at(CapitalizedPrice, '*_50d')
    _55d: CapitalizedPrice = _at(CapitalizedPrice, '*_55d')
    _89d: CapitalizedPrice = _at(CapitalizedPrice, '*_89d')
    _111d: CapitalizedPrice = _at(CapitalizedPrice, '*_111d')
    _144d: CapitalizedPrice = _at(CapitalizedPrice, '*_144d')
    _200d: Sma200d = _at(Sma200d, '*_200d')
    _350d: Sma350d = _at(Sma350d, '*_350d')
    _1y: CapitalizedPrice = _at(CapitalizedPrice, '*_1y')
    _2y: CapitalizedPrice = _at(CapitalizedPrice, '*_2y')
    _200w: CapitalizedPrice = _at(CapitalizedPrice, '*_200w')
    _4y: CapitalizedPrice = _at(CapitalizedPrice, '*_4y')


class MovingAverage(_Node):
    sma: MovingAverageSma = _at(MovingAverageSma, '*_sma')
    ema: Ema = _at(Ema, '*_ema')


class Max(_Node):
    _1w: Spot[Optional[SatsFract]] = _at(Spot, '*_1w')
    _2w: Spot[Optional[SatsFract]] = _at(Spot, '*_2w')
    _1m: Spot[Optional[SatsFract]] = _at(Spot, '*_1m')
    _1y: Spot[Optional[SatsFract]] = _at(Spot, '*_1y')


class Cycle(_Node):
    pct0_1: Spot[Optional[SatsFract]] = _at(Spot, '*_pct0_1')
    pct0_5: Spot[Optional[SatsFract]] = _at(Spot, '*_pct0_5')
    pct1: Spot[Optional[SatsFract]] = _at(Spot, '*_pct01')
    pct2: Spot[Optional[SatsFract]] = _at(Spot, '*_pct02')
    pct5: Spot[Optional[SatsFract]] = _at(Spot, '*_pct05')
    pct10: Spot[Optional[SatsFract]] = _at(Spot, '*_pct10')
    pct20: Spot[Optional[SatsFract]] = _at(Spot, '*_pct20')
    pct30: Spot[Optional[SatsFract]] = _at(Spot, '*_pct30')
    pct40: Spot[Optional[SatsFract]] = _at(Spot, '*_pct40')
    pct50: Spot[Optional[SatsFract]] = _at(Spot, '*_pct50')
    pct60: Spot[Optional[SatsFract]] = _at(Spot, '*_pct60')
    pct70: Spot[Optional[SatsFract]] = _at(Spot, '*_pct70')
    pct80: Spot[Optional[SatsFract]] = _at(Spot, '*_pct80')
    pct90: Spot[Optional[SatsFract]] = _at(Spot, '*_pct90')
    pct95: Spot[Optional[SatsFract]] = _at(Spot, '*_pct95')
    pct98: Spot[Optional[SatsFract]] = _at(Spot, '*_pct98')
    pct99: Spot[Optional[SatsFract]] = _at(Spot, '*_pct99')
    pct99_5: Spot[Optional[SatsFract]] = _at(Spot, '*_pct99_5')
    pct99_9: Spot[Optional[SatsFract]] = _at(Spot, '*_pct99_9')
    level: SeriesPattern2[Score] = _at(SeriesPattern2, '*_level')
    score: SeriesPattern2[Score] = _at(SeriesPattern2, '*_score')


class Pct999(_Node):
    ppm: SeriesPattern2[Optional[PartsPerMillion32]] = _at(SeriesPattern2, '*_ratio_pct99_9_ppm')
    ratio: SeriesPattern2[Optional[Ratio]] = _at(SeriesPattern2, '*_ratio_pct99_9')
    price: Spot[Optional[SatsFract]] = _at(Spot, '*_pct99_9')


class Pct995(_Node):
    ppm: SeriesPattern2[Optional[PartsPerMillion32]] = _at(SeriesPattern2, '*_ratio_pct99_5_ppm')
    ratio: SeriesPattern2[Optional[Ratio]] = _at(SeriesPattern2, '*_ratio_pct99_5')
    price: Spot[Optional[SatsFract]] = _at(Spot, '*_pct99_5')


class Pct99(_Node):
    ppm: SeriesPattern2[Optional[PartsPerMillion32]] = _at(SeriesPattern2, '*_ratio_pct99_ppm')
    ratio: SeriesPattern2[Optional[Ratio]] = _at(SeriesPattern2, '*_ratio_pct99')
    price: Spot[Optional[SatsFract]] = _at(Spot, '*_pct99')


class Pct98(_Node):
    ppm: SeriesPattern2[Optional[PartsPerMillion32]] = _at(SeriesPattern2, '*_ratio_pct98_ppm')
    ratio: SeriesPattern2[Optional[Ratio]] = _at(SeriesPattern2, '*_ratio_pct98')
    price: Spot[Optional[SatsFract]] = _at(Spot, '*_pct98')


class Pct95(_Node):
    ppm: SeriesPattern2[Optional[PartsPerMillion32]] = _at(SeriesPattern2, '*_ratio_pct95_ppm')
    ratio: SeriesPattern2[Optional[Ratio]] = _at(SeriesPattern2, '*_ratio_pct95')
    price: Spot[Optional[SatsFract]] = _at(Spot, '*_pct95')


class Pct90(_Node):
    ppm: SeriesPattern2[Optional[PartsPerMillion32]] = _at(SeriesPattern2, '*_ratio_pct90_ppm')
    ratio: SeriesPattern2[Optional[Ratio]] = _at(SeriesPattern2, '*_ratio_pct90')
    price: Spot[Optional[SatsFract]] = _at(Spot, '*_pct90')


class Pct80(_Node):
    ppm: SeriesPattern2[Optional[PartsPerMillion32]] = _at(SeriesPattern2, '*_ratio_pct80_ppm')
    ratio: SeriesPattern2[Optional[Ratio]] = _at(SeriesPattern2, '*_ratio_pct80')
    price: Spot[Optional[SatsFract]] = _at(Spot, '*_pct80')


class Pct70(_Node):
    ppm: SeriesPattern2[Optional[PartsPerMillion32]] = _at(SeriesPattern2, '*_ratio_pct70_ppm')
    ratio: SeriesPattern2[Optional[Ratio]] = _at(SeriesPattern2, '*_ratio_pct70')
    price: Spot[Optional[SatsFract]] = _at(Spot, '*_pct70')


class Pct60(_Node):
    ppm: SeriesPattern2[Optional[PartsPerMillion32]] = _at(SeriesPattern2, '*_ratio_pct60_ppm')
    ratio: SeriesPattern2[Optional[Ratio]] = _at(SeriesPattern2, '*_ratio_pct60')
    price: Spot[Optional[SatsFract]] = _at(Spot, '*_pct60')


class Pct50(_Node):
    ppm: SeriesPattern2[Optional[PartsPerMillion32]] = _at(SeriesPattern2, '*_ratio_pct50_ppm')
    ratio: SeriesPattern2[Optional[Ratio]] = _at(SeriesPattern2, '*_ratio_pct50')
    price: Spot[Optional[SatsFract]] = _at(Spot, '*_pct50')


class Pct40(_Node):
    ppm: SeriesPattern2[Optional[PartsPerMillion32]] = _at(SeriesPattern2, '*_ratio_pct40_ppm')
    ratio: SeriesPattern2[Optional[Ratio]] = _at(SeriesPattern2, '*_ratio_pct40')
    price: Spot[Optional[SatsFract]] = _at(Spot, '*_pct40')


class Pct30(_Node):
    ppm: SeriesPattern2[Optional[PartsPerMillion32]] = _at(SeriesPattern2, '*_ratio_pct30_ppm')
    ratio: SeriesPattern2[Optional[Ratio]] = _at(SeriesPattern2, '*_ratio_pct30')
    price: Spot[Optional[SatsFract]] = _at(Spot, '*_pct30')


class Pct20(_Node):
    ppm: SeriesPattern2[Optional[PartsPerMillion32]] = _at(SeriesPattern2, '*_ratio_pct20_ppm')
    ratio: SeriesPattern2[Optional[Ratio]] = _at(SeriesPattern2, '*_ratio_pct20')
    price: Spot[Optional[SatsFract]] = _at(Spot, '*_pct20')


class Pct10(_Node):
    ppm: SeriesPattern2[Optional[PartsPerMillion32]] = _at(SeriesPattern2, '*_ratio_pct10_ppm')
    ratio: SeriesPattern2[Optional[Ratio]] = _at(SeriesPattern2, '*_ratio_pct10')
    price: Spot[Optional[SatsFract]] = _at(Spot, '*_pct10')


class Pct5(_Node):
    ppm: SeriesPattern2[Optional[PartsPerMillion32]] = _at(SeriesPattern2, '*_ratio_pct5_ppm')
    ratio: SeriesPattern2[Optional[Ratio]] = _at(SeriesPattern2, '*_ratio_pct5')
    price: Spot[Optional[SatsFract]] = _at(Spot, '*_pct5')


class Pct2(_Node):
    ppm: SeriesPattern2[Optional[PartsPerMillion32]] = _at(SeriesPattern2, '*_ratio_pct2_ppm')
    ratio: SeriesPattern2[Optional[Ratio]] = _at(SeriesPattern2, '*_ratio_pct2')
    price: Spot[Optional[SatsFract]] = _at(Spot, '*_pct2')


class Pct1(_Node):
    ppm: SeriesPattern2[Optional[PartsPerMillion32]] = _at(SeriesPattern2, '*_ratio_pct1_ppm')
    ratio: SeriesPattern2[Optional[Ratio]] = _at(SeriesPattern2, '*_ratio_pct1')
    price: Spot[Optional[SatsFract]] = _at(Spot, '*_pct1')


class Pct05(_Node):
    ppm: SeriesPattern2[Optional[PartsPerMillion32]] = _at(SeriesPattern2, '*_ratio_pct0_5_ppm')
    ratio: SeriesPattern2[Optional[Ratio]] = _at(SeriesPattern2, '*_ratio_pct0_5')
    price: Spot[Optional[SatsFract]] = _at(Spot, '*_pct0_5')


class Pct01(_Node):
    ppm: SeriesPattern2[Optional[PartsPerMillion32]] = _at(SeriesPattern2, '*_ratio_pct0_1_ppm')
    ratio: SeriesPattern2[Optional[Ratio]] = _at(SeriesPattern2, '*_ratio_pct0_1')
    price: Spot[Optional[SatsFract]] = _at(Spot, '*_pct0_1')


class CoinflowMedianPriceBtcWeighted(_Node):
    usd: SeriesPattern2[Optional[Dollars]] = _at(SeriesPattern2, '*')
    cents: SeriesPattern2[Optional[Cents]] = _at(SeriesPattern2, '*_cents')
    sats: SeriesPattern2[Optional[SatsFract]] = _at(SeriesPattern2, '*_sats')
    ppm: SeriesPattern2[Optional[PriceRatio]] = _at(SeriesPattern2, '*_ratio_ppm')
    ratio: SeriesPattern2[Optional[Ratio]] = _at(SeriesPattern2, '*_ratio')
    pct0_1: Pct01 = _at(Pct01, '*')
    pct0_5: Pct05 = _at(Pct05, '*')
    pct1: Pct1 = _at(Pct1, '*')
    pct2: Pct2 = _at(Pct2, '*')
    pct5: Pct5 = _at(Pct5, '*')
    pct10: Pct10 = _at(Pct10, '*')
    pct20: Pct20 = _at(Pct20, '*')
    pct30: Pct30 = _at(Pct30, '*')
    pct40: Pct40 = _at(Pct40, '*')
    pct50: Pct50 = _at(Pct50, '*')
    pct60: Pct60 = _at(Pct60, '*')
    pct70: Pct70 = _at(Pct70, '*')
    pct80: Pct80 = _at(Pct80, '*')
    pct90: Pct90 = _at(Pct90, '*')
    pct95: Pct95 = _at(Pct95, '*')
    pct98: Pct98 = _at(Pct98, '*')
    pct99: Pct99 = _at(Pct99, '*')
    pct99_5: Pct995 = _at(Pct995, '*')
    pct99_9: Pct999 = _at(Pct999, '*')


class ActivePrice(_Node):
    pct0_1: Pct01 = _at(Pct01, '*')
    pct0_5: Pct05 = _at(Pct05, '*')
    pct1: Pct1 = _at(Pct1, '*')
    pct2: Pct2 = _at(Pct2, '*')
    pct5: Pct5 = _at(Pct5, '*')
    pct10: Pct10 = _at(Pct10, '*')
    pct20: Pct20 = _at(Pct20, '*')
    pct30: Pct30 = _at(Pct30, '*')
    pct40: Pct40 = _at(Pct40, '*')
    pct50: Pct50 = _at(Pct50, '*')
    pct60: Pct60 = _at(Pct60, '*')
    pct70: Pct70 = _at(Pct70, '*')
    pct80: Pct80 = _at(Pct80, '*')
    pct90: Pct90 = _at(Pct90, '*')
    pct95: Pct95 = _at(Pct95, '*')
    pct98: Pct98 = _at(Pct98, '*')
    pct99: Pct99 = _at(Pct99, '*')
    pct99_5: Pct995 = _at(Pct995, '*')
    pct99_9: Pct999 = _at(Pct999, '*')


class Components(_Node):
    realized_price: ActivePrice = _at(ActivePrice, 'realized_*')
    capitalized_price: ActivePrice = _at(ActivePrice, 'capitalized_*')
    median_price_btc_weighted: CoinflowMedianPriceBtcWeighted = _at(CoinflowMedianPriceBtcWeighted, 'median_*_btc_weighted')
    median_price_usd_weighted: CoinflowMedianPriceBtcWeighted = _at(CoinflowMedianPriceBtcWeighted, 'median_*_usd_weighted')
    sth_median_price_btc_weighted: CoinflowMedianPriceBtcWeighted = _at(CoinflowMedianPriceBtcWeighted, 'sth_median_*_btc_weighted')
    sth_median_price_usd_weighted: CoinflowMedianPriceBtcWeighted = _at(CoinflowMedianPriceBtcWeighted, 'sth_median_*_usd_weighted')
    lth_median_price_btc_weighted: CoinflowMedianPriceBtcWeighted = _at(CoinflowMedianPriceBtcWeighted, 'lth_median_*_btc_weighted')
    lth_median_price_usd_weighted: CoinflowMedianPriceBtcWeighted = _at(CoinflowMedianPriceBtcWeighted, 'lth_median_*_usd_weighted')
    cointime_median_price_btc_weighted: CoinflowMedianPriceBtcWeighted = _at(CoinflowMedianPriceBtcWeighted, 'cointime_median_*_btc_weighted')
    cointime_median_price_usd_weighted: CoinflowMedianPriceBtcWeighted = _at(CoinflowMedianPriceBtcWeighted, 'cointime_median_*_usd_weighted')
    coinflow_median_price_btc_weighted: CoinflowMedianPriceBtcWeighted = _at(CoinflowMedianPriceBtcWeighted, 'coinflow_median_*_btc_weighted')
    coinflow_median_price_usd_weighted: CoinflowMedianPriceBtcWeighted = _at(CoinflowMedianPriceBtcWeighted, 'coinflow_median_*_usd_weighted')
    sth_cointime_median_price_btc_weighted: CoinflowMedianPriceBtcWeighted = _at(CoinflowMedianPriceBtcWeighted, 'sth_cointime_median_*_btc_weighted')
    sth_cointime_median_price_usd_weighted: CoinflowMedianPriceBtcWeighted = _at(CoinflowMedianPriceBtcWeighted, 'sth_cointime_median_*_usd_weighted')
    lth_cointime_median_price_btc_weighted: CoinflowMedianPriceBtcWeighted = _at(CoinflowMedianPriceBtcWeighted, 'lth_cointime_median_*_btc_weighted')
    lth_cointime_median_price_usd_weighted: CoinflowMedianPriceBtcWeighted = _at(CoinflowMedianPriceBtcWeighted, 'lth_cointime_median_*_usd_weighted')
    sth_coinflow_median_price_btc_weighted: CoinflowMedianPriceBtcWeighted = _at(CoinflowMedianPriceBtcWeighted, 'sth_coinflow_median_*_btc_weighted')
    sth_coinflow_median_price_usd_weighted: CoinflowMedianPriceBtcWeighted = _at(CoinflowMedianPriceBtcWeighted, 'sth_coinflow_median_*_usd_weighted')
    lth_coinflow_median_price_btc_weighted: CoinflowMedianPriceBtcWeighted = _at(CoinflowMedianPriceBtcWeighted, 'lth_coinflow_median_*_btc_weighted')
    lth_coinflow_median_price_usd_weighted: CoinflowMedianPriceBtcWeighted = _at(CoinflowMedianPriceBtcWeighted, 'lth_coinflow_median_*_usd_weighted')
    sth_realized_price: ActivePrice = _at(ActivePrice, 'sth_realized_*')
    sth_capitalized_price: ActivePrice = _at(ActivePrice, 'sth_capitalized_*')
    lth_realized_price: ActivePrice = _at(ActivePrice, 'lth_realized_*')
    lth_capitalized_price: ActivePrice = _at(ActivePrice, 'lth_capitalized_*')
    over_6m_realized_price: ActivePrice = _at(ActivePrice, 'over_6m_realized_*')
    over_4m_realized_price: ActivePrice = _at(ActivePrice, 'over_4m_realized_*')
    under_4m_realized_price: ActivePrice = _at(ActivePrice, 'under_4m_realized_*')
    under_6m_realized_price: ActivePrice = _at(ActivePrice, 'under_6m_realized_*')
    under_4m_capitalized_price: ActivePrice = _at(ActivePrice, 'under_4m_capitalized_*')
    under_6m_capitalized_price: ActivePrice = _at(ActivePrice, 'under_6m_capitalized_*')
    vaulted_price: ActivePrice = _at(ActivePrice, 'vaulted_*')
    active_price: ActivePrice = _at(ActivePrice, 'active_*')
    true_market_mean_price: ActivePrice = _at(ActivePrice, 'true_market_mean_*')
    cointime_price: ActivePrice = _at(ActivePrice, 'cointime_*')
    awake_price: ActivePrice = _at(ActivePrice, 'awake_*')
    coinflow_price: ActivePrice = _at(ActivePrice, 'coinflow_*')


class Level(_Node):
    pct10: Spot[Optional[SatsFract]] = _at(Spot, '*_pct10')
    pct20: Spot[Optional[SatsFract]] = _at(Spot, '*_pct20')
    pct30: Spot[Optional[SatsFract]] = _at(Spot, '*_pct30')
    pct40: Spot[Optional[SatsFract]] = _at(Spot, '*_pct40')
    pct50: Spot[Optional[SatsFract]] = _at(Spot, '*_pct50')
    pct60: Spot[Optional[SatsFract]] = _at(Spot, '*_pct60')
    pct70: Spot[Optional[SatsFract]] = _at(Spot, '*_pct70')
    pct80: Spot[Optional[SatsFract]] = _at(Spot, '*_pct80')
    pct90: Spot[Optional[SatsFract]] = _at(Spot, '*_pct90')


class Floor(_Node):
    pct95: Spot[Optional[SatsFract]] = _at(Spot, '*_pct95')
    pct98: Spot[Optional[SatsFract]] = _at(Spot, '*_pct98')
    pct99: Spot[Optional[SatsFract]] = _at(Spot, '*_pct99')
    pct99_5: Spot[Optional[SatsFract]] = _at(Spot, '*_pct99_5')
    pct99_9: Spot[Optional[SatsFract]] = _at(Spot, '*_pct99_9')


class BedrockCoinflow(_Node):
    supply_in_loss_threshold: SupplyInLossThreshold = _at(SupplyInLossThreshold, '*_supply_in_loss_threshold')
    floor: Floor = _at(Floor, '*_floor')
    level: Level = _at(Level, '*_level')


class Bedrock(_Node):
    raw: BedrockCoinflow = _at(BedrockCoinflow, '*_raw')
    cointime: BedrockCoinflow = _at(BedrockCoinflow, '*_cointime')
    coinflow: BedrockCoinflow = _at(BedrockCoinflow, '*_coinflow')


class PerCoin(_Node):
    pct05: Spot[Optional[SatsFract]] = _at(Spot, '*_pct05')
    pct10: Spot[Optional[SatsFract]] = _at(Spot, '*_pct10')
    pct15: Spot[Optional[SatsFract]] = _at(Spot, '*_pct15')
    pct20: Spot[Optional[SatsFract]] = _at(Spot, '*_pct20')
    pct25: Spot[Optional[SatsFract]] = _at(Spot, '*_pct25')
    pct30: Spot[Optional[SatsFract]] = _at(Spot, '*_pct30')
    pct35: Spot[Optional[SatsFract]] = _at(Spot, '*_pct35')
    pct40: Spot[Optional[SatsFract]] = _at(Spot, '*_pct40')
    pct45: Spot[Optional[SatsFract]] = _at(Spot, '*_pct45')
    pct50: Spot[Optional[SatsFract]] = _at(Spot, '*_pct50')
    pct55: Spot[Optional[SatsFract]] = _at(Spot, '*_pct55')
    pct60: Spot[Optional[SatsFract]] = _at(Spot, '*_pct60')
    pct65: Spot[Optional[SatsFract]] = _at(Spot, '*_pct65')
    pct70: Spot[Optional[SatsFract]] = _at(Spot, '*_pct70')
    pct75: Spot[Optional[SatsFract]] = _at(Spot, '*_pct75')
    pct80: Spot[Optional[SatsFract]] = _at(Spot, '*_pct80')
    pct85: Spot[Optional[SatsFract]] = _at(Spot, '*_pct85')
    pct90: Spot[Optional[SatsFract]] = _at(Spot, '*_pct90')
    pct95: Spot[Optional[SatsFract]] = _at(Spot, '*_pct95')


class UrpdAllCostBasis(_Node, Generic[A]):
    per_coin: A = _at(0, '*_coin')
    per_dollar: A = _at(0, '*_dollar')


class SpendingRate(_Node, Generic[A]):
    under_1h: SeriesPattern2[A] = _at(SeriesPattern2, 'utxos_under_1h_*')
    _1h_to_1d: SeriesPattern2[A] = _at(SeriesPattern2, 'utxos_1h_to_1d_*')
    _1d_to_1w: SeriesPattern2[A] = _at(SeriesPattern2, 'utxos_1d_to_1w_*')
    _1w_to_1m: SeriesPattern2[A] = _at(SeriesPattern2, 'utxos_1w_to_1m_*')
    _1m_to_2m: SeriesPattern2[A] = _at(SeriesPattern2, 'utxos_1m_to_2m_*')
    _2m_to_3m: SeriesPattern2[A] = _at(SeriesPattern2, 'utxos_2m_to_3m_*')
    _3m_to_4m: SeriesPattern2[A] = _at(SeriesPattern2, 'utxos_3m_to_4m_*')
    _4m_to_5m: SeriesPattern2[A] = _at(SeriesPattern2, 'utxos_4m_to_5m_*')
    _5m_to_6m: SeriesPattern2[A] = _at(SeriesPattern2, 'utxos_5m_to_6m_*')
    _6m_to_9m: SeriesPattern2[A] = _at(SeriesPattern2, 'utxos_6m_to_9m_*')
    _9m_to_1y: SeriesPattern2[A] = _at(SeriesPattern2, 'utxos_9m_to_1y_*')
    _1y_to_18m: SeriesPattern2[A] = _at(SeriesPattern2, 'utxos_1y_to_18m_*')
    _18m_to_2y: SeriesPattern2[A] = _at(SeriesPattern2, 'utxos_18m_to_2y_*')
    _2y_to_3y: SeriesPattern2[A] = _at(SeriesPattern2, 'utxos_2y_to_3y_*')
    _3y_to_4y: SeriesPattern2[A] = _at(SeriesPattern2, 'utxos_3y_to_4y_*')
    _4y_to_5y: SeriesPattern2[A] = _at(SeriesPattern2, 'utxos_4y_to_5y_*')
    _5y_to_6y: SeriesPattern2[A] = _at(SeriesPattern2, 'utxos_5y_to_6y_*')
    _6y_to_7y: SeriesPattern2[A] = _at(SeriesPattern2, 'utxos_6y_to_7y_*')
    _7y_to_8y: SeriesPattern2[A] = _at(SeriesPattern2, 'utxos_7y_to_8y_*')
    _8y_to_10y: SeriesPattern2[A] = _at(SeriesPattern2, 'utxos_8y_to_10y_*')
    _10y_to_12y: SeriesPattern2[A] = _at(SeriesPattern2, 'utxos_10y_to_12y_*')
    _12y_to_15y: SeriesPattern2[A] = _at(SeriesPattern2, 'utxos_12y_to_15y_*')
    over_15y: SeriesPattern2[A] = _at(SeriesPattern2, 'utxos_over_15y_*')


class SpendingExposure(_Node):
    under_1h: SeriesPattern2[Optional[Float64]] = _at(SeriesPattern2, 'utxos_under_1h_*_spending_exposure')
    _1h_to_1d: SeriesPattern2[Optional[Float64]] = _at(SeriesPattern2, 'utxos_1h_to_1d_*_spending_exposure')
    _1d_to_1w: SeriesPattern2[Optional[Float64]] = _at(SeriesPattern2, 'utxos_1d_to_1w_*_spending_exposure')
    _1w_to_1m: SeriesPattern2[Optional[Float64]] = _at(SeriesPattern2, 'utxos_1w_to_1m_*_spending_exposure')
    _1m_to_2m: SeriesPattern2[Optional[Float64]] = _at(SeriesPattern2, 'utxos_1m_to_2m_*_spending_exposure')
    _2m_to_3m: SeriesPattern2[Optional[Float64]] = _at(SeriesPattern2, 'utxos_2m_to_3m_*_spending_exposure')
    _3m_to_4m: SeriesPattern2[Optional[Float64]] = _at(SeriesPattern2, 'utxos_3m_to_4m_*_spending_exposure')
    _4m_to_5m: SeriesPattern2[Optional[Float64]] = _at(SeriesPattern2, 'utxos_4m_to_5m_*_spending_exposure')
    _5m_to_6m: SeriesPattern2[Optional[Float64]] = _at(SeriesPattern2, 'utxos_5m_to_6m_*_spending_exposure')
    _6m_to_9m: SeriesPattern2[Optional[Float64]] = _at(SeriesPattern2, 'utxos_6m_to_9m_*_spending_exposure')
    _9m_to_1y: SeriesPattern2[Optional[Float64]] = _at(SeriesPattern2, 'utxos_9m_to_1y_*_spending_exposure')
    _1y_to_18m: SeriesPattern2[Optional[Float64]] = _at(SeriesPattern2, 'utxos_1y_to_18m_*_spending_exposure')
    _18m_to_2y: SeriesPattern2[Optional[Float64]] = _at(SeriesPattern2, 'utxos_18m_to_2y_*_spending_exposure')
    _2y_to_3y: SeriesPattern2[Optional[Float64]] = _at(SeriesPattern2, 'utxos_2y_to_3y_*_spending_exposure')
    _3y_to_4y: SeriesPattern2[Optional[Float64]] = _at(SeriesPattern2, 'utxos_3y_to_4y_*_spending_exposure')
    _4y_to_5y: SeriesPattern2[Optional[Float64]] = _at(SeriesPattern2, 'utxos_4y_to_5y_*_spending_exposure')
    _5y_to_6y: SeriesPattern2[Optional[Float64]] = _at(SeriesPattern2, 'utxos_5y_to_6y_*_spending_exposure')
    _6y_to_7y: SeriesPattern2[Optional[Float64]] = _at(SeriesPattern2, 'utxos_6y_to_7y_*_spending_exposure')
    _7y_to_8y: SeriesPattern2[Optional[Float64]] = _at(SeriesPattern2, 'utxos_7y_to_8y_*_spending_exposure')
    _8y_to_10y: SeriesPattern2[Optional[Float64]] = _at(SeriesPattern2, 'utxos_8y_to_10y_*_spending_exposure')
    _10y_to_12y: SeriesPattern2[Optional[Float64]] = _at(SeriesPattern2, 'utxos_10y_to_12y_*_spending_exposure')
    _12y_to_15y: SeriesPattern2[Optional[Float64]] = _at(SeriesPattern2, 'utxos_12y_to_15y_*_spending_exposure')
    over_15y: SeriesPattern2[Optional[Float64]] = _at(SeriesPattern2, 'utxos_over_15y_*_spending_exposure')
    mobility: SpendingRate[Optional[Ratio64]] = _at(SpendingRate, '*_mobility')


class AgeRangeActivity(_Node):
    wakefulness: SpendingRate[Optional[Ratio64]] = _at(SpendingRate, '*_wakefulness')
    dormancy: SpendingRate[Optional[Ratio64]] = _at(SpendingRate, '*_dormancy')
    wakefulness_to_dormancy: SpendingRate[Optional[Ratio64]] = _at(SpendingRate, '*_wakefulness_to_dormancy')


class RateSma(_Node):
    _1w: SeriesPattern2[Optional[Hashrate]] = _at(SeriesPattern2, '*_1w')
    _1m: SeriesPattern2[Optional[Hashrate]] = _at(SeriesPattern2, '*_1m')
    _2m: SeriesPattern2[Optional[Hashrate]] = _at(SeriesPattern2, '*_2m')
    _1y: SeriesPattern2[Optional[Hashrate]] = _at(SeriesPattern2, '*_1y')


class OpReturnRaw(_Node):
    first_index: SeriesPattern21[OpReturnIndex] = _at(SeriesPattern21, 'first_op_return_*')
    to_tx_index: SeriesPattern26[TxIndex] = _at(SeriesPattern26, 'tx_*')
    kind: SeriesPattern26[OpReturnKind] = _at(SeriesPattern26, 'kind')
    post_op_return_bytes: SeriesPattern26[Bytes32] = _at(SeriesPattern26, 'op_return_post_op_return_bytes')


class RawUnknown(_Node):
    first_index: SeriesPattern21[UnknownOutputIndex] = _at(SeriesPattern21, 'first_unknown_output_*')
    to_tx_index: SeriesPattern36[TxIndex] = _at(SeriesPattern36, 'tx_*')
    legacy_sigops: SeriesPattern36[SigOps] = _at(SeriesPattern36, 'unknown_legacy_sigops')


class RawP2ms(_Node):
    first_index: SeriesPattern21[P2MSOutputIndex] = _at(SeriesPattern21, 'first_p2ms_output_*')
    to_tx_index: SeriesPattern28[TxIndex] = _at(SeriesPattern28, 'tx_*')
    legacy_sigops: SeriesPattern28[SigOps] = _at(SeriesPattern28, 'p2ms_legacy_sigops')


class RawEmpty(_Node):
    first_index: SeriesPattern21[EmptyOutputIndex] = _at(SeriesPattern21, 'first_empty_output_*')
    to_tx_index: SeriesPattern25[TxIndex] = _at(SeriesPattern25, 'tx_*')


class ScriptsRaw(_Node):
    empty: RawEmpty = _at(RawEmpty, '*')
    p2ms: RawP2ms = _at(RawP2ms, '*')
    unknown: RawUnknown = _at(RawUnknown, '*')


class Scripts(_Node):
    raw: ScriptsRaw = _at(ScriptsRaw, '*')


class AddrsEmpty(_Node):
    all: SeriesPattern2[Count] = _at(SeriesPattern2, '*')
    p2pk65: SeriesPattern2[Count] = _at(SeriesPattern2, 'p2pk65_*')
    p2pk33: SeriesPattern2[Count] = _at(SeriesPattern2, 'p2pk33_*')
    p2pkh: SeriesPattern2[Count] = _at(SeriesPattern2, 'p2pkh_*')
    p2sh: SeriesPattern2[Count] = _at(SeriesPattern2, 'p2sh_*')
    p2wpkh: SeriesPattern2[Count] = _at(SeriesPattern2, 'p2wpkh_*')
    p2wsh: SeriesPattern2[Count] = _at(SeriesPattern2, 'p2wsh_*')
    p2tr: SeriesPattern2[Count] = _at(SeriesPattern2, 'p2tr_*')
    p2a: SeriesPattern2[Count] = _at(SeriesPattern2, 'p2a_*')


class ExposedCount(_Node):
    funded: AddrsEmpty = _at(AddrsEmpty, '*')
    total: AddrsEmpty = _at(AddrsEmpty, 'total_*')


class RealizedLoss0satsBlock(_Node, Generic[A]):
    usd: SeriesPattern21[Optional[Dollars]] = _at(SeriesPattern21, '*')
    cents: SeriesPattern21[A] = _at(SeriesPattern21, '*_cents')


class CoinflowCap(_Node, Generic[A]):
    usd: SeriesPattern2[Optional[Dollars]] = _at(SeriesPattern2, '*')
    cents: SeriesPattern2[A] = _at(SeriesPattern2, '*_cents')


class AllUnrealized(_Node):
    profit: CoinflowCap[Optional[Cents]] = _at(CoinflowCap, '*_unrealized_profit')
    loss: CoinflowCap[Optional[Cents]] = _at(CoinflowCap, '*_unrealized_loss')
    net_pnl: CoinflowCap[CentsSigned] = _at(CoinflowCap, '*_net_unrealized_pnl')
    gross_pnl: CoinflowCap[Optional[Cents]] = _at(CoinflowCap, '*_unrealized_gross_pnl')
    invested_capital_in_profit: CoinflowCap[Optional[Cents]] = _at(CoinflowCap, '*_invested_capital_in_profit')
    invested_capital_in_loss: CoinflowCap[Optional[Cents]] = _at(CoinflowCap, '*_invested_capital_in_loss')
    pain_index: CoinflowCap[Optional[Cents]] = _at(CoinflowCap, '*_pain_index')
    greed_index: CoinflowCap[Optional[Cents]] = _at(CoinflowCap, '*_greed_index')
    net_sentiment: CoinflowCap[CentsSigned] = _at(CoinflowCap, '*_net_sentiment')
    nupl: RhodlRatio[Optional[PartsPerMillionSigned32]] = _at(RhodlRatio, '*_nupl')


class CointimeCap(_Node):
    thermo: CoinflowCap[Optional[Cents]] = _at(CoinflowCap, 'thermo_*')
    investor: CoinflowCap[Optional[Cents]] = _at(CoinflowCap, 'investor_*')
    vaulted: CoinflowCap[Optional[Cents]] = _at(CoinflowCap, 'vaulted_*')
    active: CoinflowCap[Optional[Cents]] = _at(CoinflowCap, 'active_*')
    cointime: CoinflowCap[Optional[Cents]] = _at(CoinflowCap, 'cointime_*')
    aviv: RhodlRatio[Optional[PartsPerMillion32]] = _at(RhodlRatio, 'aviv_ratio')


class Awake(_Node):
    supply: AwakeSupply = _at(AwakeSupply, '*_supply')
    cap: CoinflowCap[Optional[Cents]] = _at(CoinflowCap, '*_cap')
    price: CapitalizedPrice = _at(CapitalizedPrice, '*_price')
    capitalized_price: CapitalizedPrice = _at(CapitalizedPrice, '*_capitalized_price')


class Absolute1m(_Node):
    btc: SeriesPattern2[Optional[Bitcoin]] = _at(SeriesPattern2, '*')
    sats: SeriesPattern2[SatsSigned] = _at(SeriesPattern2, '*_sats')


class State(_Node):
    p2a: SeriesPattern27[AddrState] = _at(SeriesPattern27, '*_state')
    p2pk33: SeriesPattern29[AddrState] = _at(SeriesPattern29, '*_state')
    p2pk65: SeriesPattern30[AddrState] = _at(SeriesPattern30, '*_state')
    p2pkh: SeriesPattern31[AddrState] = _at(SeriesPattern31, '*_state')
    p2sh: SeriesPattern32[AddrState] = _at(SeriesPattern32, '*_state')
    p2tr: SeriesPattern33[AddrState] = _at(SeriesPattern33, '*_state')
    p2wpkh: SeriesPattern34[AddrState] = _at(SeriesPattern34, '*_state')
    p2wsh: SeriesPattern35[AddrState] = _at(SeriesPattern35, '*_state')
    funded: SeriesPattern37[FundedAddrData] = _at(SeriesPattern37, 'funded_*_data')
    extended_empty: SeriesPattern38[EmptyAddrData] = _at(SeriesPattern38, 'extended_empty_*_data')


class RawP2a(_Node):
    first_index: SeriesPattern21[P2AAddrIndex] = _at(SeriesPattern21, 'first_*_addr_index')
    bytes: SeriesPattern27[P2ABytes] = _at(SeriesPattern27, '*_bytes')


class RawP2tr(_Node):
    first_index: SeriesPattern21[P2TRAddrIndex] = _at(SeriesPattern21, 'first_*_addr_index')
    bytes: SeriesPattern33[P2TRBytes] = _at(SeriesPattern33, '*_bytes')


class RawP2wsh(_Node):
    first_index: SeriesPattern21[P2WSHAddrIndex] = _at(SeriesPattern21, 'first_*_addr_index')
    bytes: SeriesPattern35[P2WSHBytes] = _at(SeriesPattern35, '*_bytes')


class RawP2wpkh(_Node):
    first_index: SeriesPattern21[P2WPKHAddrIndex] = _at(SeriesPattern21, 'first_*_addr_index')
    bytes: SeriesPattern34[P2WPKHBytes] = _at(SeriesPattern34, '*_bytes')


class RawP2sh(_Node):
    first_index: SeriesPattern21[P2SHAddrIndex] = _at(SeriesPattern21, 'first_*_addr_index')
    bytes: SeriesPattern32[P2SHBytes] = _at(SeriesPattern32, '*_bytes')


class RawP2pkh(_Node):
    first_index: SeriesPattern21[P2PKHAddrIndex] = _at(SeriesPattern21, 'first_*_addr_index')
    bytes: SeriesPattern31[P2PKHBytes] = _at(SeriesPattern31, '*_bytes')


class RawP2pk33(_Node):
    first_index: SeriesPattern21[P2PK33AddrIndex] = _at(SeriesPattern21, 'first_*_addr_index')
    bytes: SeriesPattern29[P2PK33Bytes] = _at(SeriesPattern29, '*_bytes')


class RawP2pk65(_Node):
    first_index: SeriesPattern21[P2PK65AddrIndex] = _at(SeriesPattern21, 'first_*_addr_index')
    bytes: SeriesPattern30[P2PK65Bytes] = _at(SeriesPattern30, '*_bytes')


class AddrsRaw(_Node):
    p2pk65: RawP2pk65 = _at(RawP2pk65, '*')
    p2pk33: RawP2pk33 = _at(RawP2pk33, 'p2pk33')
    p2pkh: RawP2pkh = _at(RawP2pkh, 'p2pkh')
    p2sh: RawP2sh = _at(RawP2sh, 'p2sh')
    p2wpkh: RawP2wpkh = _at(RawP2wpkh, 'p2wpkh')
    p2wsh: RawP2wsh = _at(RawP2wsh, 'p2wsh')
    p2tr: RawP2tr = _at(RawP2tr, 'p2tr')
    p2a: RawP2a = _at(RawP2a, 'p2a')


class Spent(_Node):
    txin_index: SeriesPattern24[TxInIndex] = _at(SeriesPattern24, '*')


class OutputsRaw(_Node):
    first_txout_index: SeriesPattern21[TxOutIndex] = _at(SeriesPattern21, 'first_txout_index')
    value: SeriesPattern24[Sats] = _at(SeriesPattern24, 'value')
    output_type: SeriesPattern24[OutputType] = _at(SeriesPattern24, 'output_*')
    type_index: SeriesPattern24[TypeIndex] = _at(SeriesPattern24, '*_index')


class InputsRaw(_Node):
    first_txin_index: SeriesPattern21[TxInIndex] = _at(SeriesPattern21, 'first_txin_*')
    outpoint: SeriesPattern23[OutPoint] = _at(SeriesPattern23, 'outpoint')
    txout_index: SeriesPattern23[TxOutIndex] = _at(SeriesPattern23, 'txout_*')
    tx_index: SeriesPattern23[TxIndex] = _at(SeriesPattern23, 'tx_*')
    output_type: SeriesPattern23[OutputType] = _at(SeriesPattern23, 'output_type')
    type_index: SeriesPattern23[TypeIndex] = _at(SeriesPattern23, 'type_*')


class Circulating(_Node, Generic[A, B]):
    btc: SeriesPattern2[Optional[Bitcoin]] = _at(SeriesPattern2, '*')
    sats: SeriesPattern2[A] = _at(SeriesPattern2, '*_sats')
    usd: SeriesPattern2[Optional[Dollars]] = _at(SeriesPattern2, '*_usd')
    cents: SeriesPattern2[B] = _at(SeriesPattern2, '*_cents')


class CoinflowSupply(_Node):
    mobile: Mobile = _at(Mobile, '*')
    immobile: Circulating[Sats, Optional[Cents]] = _at(Circulating, '*_immobile_supply')


class CoinflowLth(_Node):
    supply: CoinflowSupply = _at(CoinflowSupply, '*')
    cap: CoinflowCap[Optional[Cents]] = _at(CoinflowCap, '*_coinflow_cap')
    price: CapitalizedPrice = _at(CapitalizedPrice, '*_coinflow_price')
    capitalized_price: CapitalizedPrice = _at(CapitalizedPrice, '*_coinflow_capitalized_price')


class CointimeSupply(_Node):
    vaulted: Circulating[Sats, Optional[Cents]] = _at(Circulating, 'vaulted_*')
    active: Active = _at(Active, '*')


class Dormant(_Node):
    supply: Circulating[Sats, Optional[Cents]] = _at(Circulating, '*')


class CointimeLth(_Node):
    awake: Awake = _at(Awake, '*_awake')
    dormant: Dormant = _at(Dormant, '*_dormant_supply')


class BurnedBlock(_Node):
    btc: SeriesPattern21[Optional[Bitcoin]] = _at(SeriesPattern21, '*')
    sats: SeriesPattern21[Sats] = _at(SeriesPattern21, '*_sats')
    usd: SeriesPattern21[Optional[Dollars]] = _at(SeriesPattern21, '*_usd')
    cents: SeriesPattern21[Optional[Cents]] = _at(SeriesPattern21, '*_cents')


class Burned(_Node):
    block: BurnedBlock = _at(BurnedBlock, '*')
    cumulative: Circulating[Sats, Optional[Cents]] = _at(Circulating, '*_cumulative')


class OutputsValue(_Node):
    op_return: Burned = _at(Burned, '*')


class EffectiveFeeRate6b(_Node, Generic[A]):
    min: SeriesPattern2[A] = _at(SeriesPattern2, '*_min')
    max: SeriesPattern2[A] = _at(SeriesPattern2, '*_max')
    pct10: SeriesPattern2[A] = _at(SeriesPattern2, '*_pct10')
    pct25: SeriesPattern2[A] = _at(SeriesPattern2, '*_pct25')
    median: SeriesPattern2[A] = _at(SeriesPattern2, '*_median')
    pct75: SeriesPattern2[A] = _at(SeriesPattern2, '*_pct75')
    pct90: SeriesPattern2[A] = _at(SeriesPattern2, '*_pct90')


class SizeWeight(_Node):
    block: EffectiveFeeRate6b[Weight] = _at(EffectiveFeeRate6b, '*')
    _6b: EffectiveFeeRate6b[Weight] = _at(EffectiveFeeRate6b, '*_6b')


class Vsize6b(_Node):
    min: SeriesPattern21[VSize] = _at(SeriesPattern21, '*_min')
    max: SeriesPattern21[VSize] = _at(SeriesPattern21, '*_max')
    pct10: SeriesPattern21[VSize] = _at(SeriesPattern21, '*_pct10')
    pct25: SeriesPattern21[VSize] = _at(SeriesPattern21, '*_pct25')
    median: SeriesPattern21[VSize] = _at(SeriesPattern21, '*_median')
    pct75: SeriesPattern21[VSize] = _at(SeriesPattern21, '*_pct75')
    pct90: SeriesPattern21[VSize] = _at(SeriesPattern21, '*_pct90')


class EffectiveFeeRate(_Node, Generic[A, B]):
    tx_index: SeriesPattern22[A] = _at(SeriesPattern22, '*')
    block: B = _at(0, '*')
    _6b: B = _at(0, '*_6b')


class TransactionsSize(_Node):
    vsize: EffectiveFeeRate[VSize, Vsize6b] = _at((EffectiveFeeRate, Vsize6b), '*_vsize')
    weight: SizeWeight = _at(SizeWeight, '*_weight')


class TransactionsRaw(_Node):
    first_tx_index: SeriesPattern21[TxIndex] = _at(SeriesPattern21, 'first_*_index')
    txid: SeriesPattern22[Txid] = _at(SeriesPattern22, 'txid')
    tx_version: SeriesPattern22[TxVersion] = _at(SeriesPattern22, '*_version')
    raw_locktime: SeriesPattern22[RawLockTime] = _at(SeriesPattern22, 'raw_locktime')
    weight: SeriesPattern22[Weight] = _at(SeriesPattern22, '*_weight')
    total_size: SeriesPattern22[Bytes32] = _at(SeriesPattern22, 'total_size')
    total_sigop_cost: SeriesPattern22[SigOps] = _at(SeriesPattern22, 'total_sigop_cost')
    is_explicitly_rbf: SeriesPattern22[Boolean] = _at(SeriesPattern22, 'is_explicitly_rbf')
    first_txin_index: SeriesPattern22[TxInIndex] = _at(SeriesPattern22, 'first_txin_index')
    first_txout_index: SeriesPattern22[TxOutIndex] = _at(SeriesPattern22, 'first_txout_index')


class BlocksHalving(_Node):
    epoch: SeriesPattern2[Halving] = _at(SeriesPattern2, '*_epoch')
    blocks_to_halving: SeriesPattern2[Count] = _at(SeriesPattern2, 'blocks_to_*')
    days_to_halving: SeriesPattern2[Optional[Days]] = _at(SeriesPattern2, 'days_to_*')


class Fullness(_Node):
    ppm: SeriesPattern21[Optional[PartsPerMillion32]] = _at(SeriesPattern21, '*_ppm')
    ratio: SeriesPattern21[Optional[Ratio]] = _at(SeriesPattern21, '*_ratio')
    percent: SeriesPattern21[Optional[Percent]] = _at(SeriesPattern21, '*')


class Interval(_Node, Generic[A, B]):
    block: SeriesPattern21[A] = _at(SeriesPattern21, '*')
    _24h: SeriesPattern2[B] = _at(SeriesPattern2, '*_average_24h')
    _1w: SeriesPattern2[B] = _at(SeriesPattern2, '*_average_1w')
    _1m: SeriesPattern2[B] = _at(SeriesPattern2, '*_average_1m')
    _1y: SeriesPattern2[B] = _at(SeriesPattern2, '*_average_1y')


class BlocksLookback(_Node):
    _1h: SeriesPattern21[Height] = _at(SeriesPattern21, '*_1h_ago')
    _24h: SeriesPattern21[Height] = _at(SeriesPattern21, '*_24h_ago')
    _3d: SeriesPattern21[Height] = _at(SeriesPattern21, '*_3d_ago')
    _1w: SeriesPattern21[Height] = _at(SeriesPattern21, '*_1w_ago')
    _8d: SeriesPattern21[Height] = _at(SeriesPattern21, '*_8d_ago')
    _9d: SeriesPattern21[Height] = _at(SeriesPattern21, '*_9d_ago')
    _12d: SeriesPattern21[Height] = _at(SeriesPattern21, '*_12d_ago')
    _13d: SeriesPattern21[Height] = _at(SeriesPattern21, '*_13d_ago')
    _2w: SeriesPattern21[Height] = _at(SeriesPattern21, '*_2w_ago')
    _21d: SeriesPattern21[Height] = _at(SeriesPattern21, '*_21d_ago')
    _26d: SeriesPattern21[Height] = _at(SeriesPattern21, '*_26d_ago')
    _1m: SeriesPattern21[Height] = _at(SeriesPattern21, '*_1m_ago')
    _34d: SeriesPattern21[Height] = _at(SeriesPattern21, '*_34d_ago')
    _50d: SeriesPattern21[Height] = _at(SeriesPattern21, '*_50d_ago')
    _55d: SeriesPattern21[Height] = _at(SeriesPattern21, '*_55d_ago')
    _2m: SeriesPattern21[Height] = _at(SeriesPattern21, '*_2m_ago')
    _9w: SeriesPattern21[Height] = _at(SeriesPattern21, '*_9w_ago')
    _12w: SeriesPattern21[Height] = _at(SeriesPattern21, '*_12w_ago')
    _89d: SeriesPattern21[Height] = _at(SeriesPattern21, '*_89d_ago')
    _3m: SeriesPattern21[Height] = _at(SeriesPattern21, '*_3m_ago')
    _14w: SeriesPattern21[Height] = _at(SeriesPattern21, '*_14w_ago')
    _111d: SeriesPattern21[Height] = _at(SeriesPattern21, '*_111d_ago')
    _144d: SeriesPattern21[Height] = _at(SeriesPattern21, '*_144d_ago')
    _6m: SeriesPattern21[Height] = _at(SeriesPattern21, '*_6m_ago')
    _26w: SeriesPattern21[Height] = _at(SeriesPattern21, '*_26w_ago')
    _200d: SeriesPattern21[Height] = _at(SeriesPattern21, '*_200d_ago')
    _9m: SeriesPattern21[Height] = _at(SeriesPattern21, '*_9m_ago')
    _350d: SeriesPattern21[Height] = _at(SeriesPattern21, '*_350d_ago')
    _12m: SeriesPattern21[Height] = _at(SeriesPattern21, '*_12m_ago')
    _1y: SeriesPattern21[Height] = _at(SeriesPattern21, '*_1y_ago')
    _14m: SeriesPattern21[Height] = _at(SeriesPattern21, '*_14m_ago')
    _2y: SeriesPattern21[Height] = _at(SeriesPattern21, '*_2y_ago')
    _26m: SeriesPattern21[Height] = _at(SeriesPattern21, '*_26m_ago')
    _3y: SeriesPattern21[Height] = _at(SeriesPattern21, '*_3y_ago')
    _200w: SeriesPattern21[Height] = _at(SeriesPattern21, '*_200w_ago')
    _4y: SeriesPattern21[Height] = _at(SeriesPattern21, '*_4y_ago')
    _5y: SeriesPattern21[Height] = _at(SeriesPattern21, '*_5y_ago')
    _6y: SeriesPattern21[Height] = _at(SeriesPattern21, '*_6y_ago')
    _8y: SeriesPattern21[Height] = _at(SeriesPattern21, '*_8y_ago')
    _9y: SeriesPattern21[Height] = _at(SeriesPattern21, '*_9y_ago')
    _10y: SeriesPattern21[Height] = _at(SeriesPattern21, '*_10y_ago')
    _12y: SeriesPattern21[Height] = _at(SeriesPattern21, '*_12y_ago')
    _14y: SeriesPattern21[Height] = _at(SeriesPattern21, '*_14y_ago')
    _26y: SeriesPattern21[Height] = _at(SeriesPattern21, '*_26y_ago')


class Target(_Node):
    _24h: SeriesPattern1[Count] = _at(SeriesPattern1, '*_24h')
    _1w: SeriesPattern1[Count] = _at(SeriesPattern1, '*_1w')
    _1m: SeriesPattern1[Count] = _at(SeriesPattern1, '*_1m')
    _1y: SeriesPattern1[Count] = _at(SeriesPattern1, '*_1y')


class PerSec(_Node, Generic[A]):
    _24h: SeriesPattern2[A] = _at(SeriesPattern2, '*_24h')
    _1w: SeriesPattern2[A] = _at(SeriesPattern2, '*_1w')
    _1m: SeriesPattern2[A] = _at(SeriesPattern2, '*_1m')
    _1y: SeriesPattern2[A] = _at(SeriesPattern2, '*_1y')


class BlocksMined(_Node):
    block: SeriesPattern21[Count] = _at(SeriesPattern21, '*')
    cumulative: SeriesPattern2[Count] = _at(SeriesPattern2, '*_cumulative')
    sum: PerSec[Count] = _at(PerSec, '*_sum')


class Rolling(_Node, Generic[A]):
    sum: PerSec[Count] = _at(PerSec, '*_sum')
    average: PerSec[Optional[CountFract]] = _at(PerSec, '*_average')
    min: PerSec[A] = _at(PerSec, '*_min')
    max: PerSec[A] = _at(PerSec, '*_max')
    pct10: PerSec[A] = _at(PerSec, '*_pct10')
    pct25: PerSec[A] = _at(PerSec, '*_pct25')
    median: PerSec[A] = _at(PerSec, '*_median')
    pct75: PerSec[A] = _at(PerSec, '*_pct75')
    pct90: PerSec[A] = _at(PerSec, '*_pct90')


class InputsCount(_Node, Generic[A]):
    sum: SeriesPattern21[Count] = _at(SeriesPattern21, '*_sum')
    cumulative: SeriesPattern2[Count] = _at(SeriesPattern2, '*_cumulative')
    rolling: Rolling[A] = _at(Rolling, '*')


class Vbytes(_Node, Generic[A, B, C]):
    block: SeriesPattern21[A] = _at(SeriesPattern21, '*')
    cumulative: SeriesPattern2[A] = _at(SeriesPattern2, '*_cumulative')
    sum: PerSec[A] = _at(PerSec, '*_sum')
    average: PerSec[B] = _at(PerSec, '*_average')
    min: PerSec[C] = _at(PerSec, '*_min')
    max: PerSec[C] = _at(PerSec, '*_max')
    pct10: PerSec[C] = _at(PerSec, '*_pct10')
    pct25: PerSec[C] = _at(PerSec, '*_pct25')
    median: PerSec[C] = _at(PerSec, '*_median')
    pct75: PerSec[C] = _at(PerSec, '*_pct75')
    pct90: PerSec[C] = _at(PerSec, '*_pct90')


class NewAll(_Node, Generic[A, B]):
    block: SeriesPattern21[A] = _at(SeriesPattern21, '*')
    cumulative: SeriesPattern2[A] = _at(SeriesPattern2, '*_cumulative')
    sum: PerSec[A] = _at(PerSec, '*_sum')
    average: PerSec[B] = _at(PerSec, '*_average')


class CointimeValue(_Node):
    destroyed: NewAll[Optional[Float64], Optional[Float64]] = _at(NewAll, '*_destroyed')
    created: NewAll[Optional[Float64], Optional[Float64]] = _at(NewAll, '*_created')
    stored: NewAll[Optional[Float64], Optional[Float64]] = _at(NewAll, '*_stored')
    vocdd: NewAll[Optional[Float64], Optional[Float64]] = _at(NewAll, 'vocdd')


class CointimeActivity(_Node):
    coinblocks_created: NewAll[Optional[CoinBlocks], Optional[CoinBlocks]] = _at(NewAll, '*_created')
    coinblocks_stored: NewAll[Optional[CoinBlocks], Optional[CoinBlocks]] = _at(NewAll, '*_stored')
    liveliness: SeriesPattern2[Optional[Ratio64]] = _at(SeriesPattern2, 'liveliness')
    vaultedness: SeriesPattern2[Optional[Ratio64]] = _at(SeriesPattern2, 'vaultedness')
    ratio: SeriesPattern2[Optional[Ratio64]] = _at(SeriesPattern2, 'activity_to_vaultedness')
    coinblocks_destroyed: NewAll[Optional[CoinBlocks], Optional[CoinBlocks]] = _at(NewAll, '*_destroyed')


class OutputsByTypeTxCount(_Node):
    all: NewAll[Count, Optional[CountFract]] = _at(NewAll, '*_bis')
    p2pk65: NewAll[Count, Optional[CountFract]] = _at(NewAll, '*_with_p2pk65_output')
    p2pk33: NewAll[Count, Optional[CountFract]] = _at(NewAll, '*_with_p2pk33_output')
    p2pkh: NewAll[Count, Optional[CountFract]] = _at(NewAll, '*_with_p2pkh_output')
    p2ms: NewAll[Count, Optional[CountFract]] = _at(NewAll, '*_with_p2ms_output')
    p2sh: NewAll[Count, Optional[CountFract]] = _at(NewAll, '*_with_p2sh_output')
    p2wpkh: NewAll[Count, Optional[CountFract]] = _at(NewAll, '*_with_p2wpkh_output')
    p2wsh: NewAll[Count, Optional[CountFract]] = _at(NewAll, '*_with_p2wsh_output')
    p2tr: NewAll[Count, Optional[CountFract]] = _at(NewAll, '*_with_p2tr_output')
    p2a: NewAll[Count, Optional[CountFract]] = _at(NewAll, '*_with_p2a_output')
    unknown: NewAll[Count, Optional[CountFract]] = _at(NewAll, '*_with_unknown_outputs_output')
    empty: NewAll[Count, Optional[CountFract]] = _at(NewAll, '*_with_empty_outputs_output')
    op_return: NewAll[Count, Optional[CountFract]] = _at(NewAll, '*_with_op_return_output')


class OutputCount(_Node):
    all: NewAll[Count, Optional[CountFract]] = _at(NewAll, '*_bis')
    p2pk65: NewAll[Count, Optional[CountFract]] = _at(NewAll, 'p2pk65_*')
    p2pk33: NewAll[Count, Optional[CountFract]] = _at(NewAll, 'p2pk33_*')
    p2pkh: NewAll[Count, Optional[CountFract]] = _at(NewAll, 'p2pkh_*')
    p2ms: NewAll[Count, Optional[CountFract]] = _at(NewAll, 'p2ms_*')
    p2sh: NewAll[Count, Optional[CountFract]] = _at(NewAll, 'p2sh_*')
    p2wpkh: NewAll[Count, Optional[CountFract]] = _at(NewAll, 'p2wpkh_*')
    p2wsh: NewAll[Count, Optional[CountFract]] = _at(NewAll, 'p2wsh_*')
    p2tr: NewAll[Count, Optional[CountFract]] = _at(NewAll, 'p2tr_*')
    p2a: NewAll[Count, Optional[CountFract]] = _at(NewAll, 'p2a_*')
    unknown: NewAll[Count, Optional[CountFract]] = _at(NewAll, 'unknown_outputs_*')
    empty: NewAll[Count, Optional[CountFract]] = _at(NewAll, 'empty_outputs_*')
    op_return: NewAll[Count, Optional[CountFract]] = _at(NewAll, 'op_return_*')


class InputsByTypeTxCount(_Node):
    all: NewAll[Count, Optional[CountFract]] = _at(NewAll, 'non_coinbase_*')
    p2pk65: NewAll[Count, Optional[CountFract]] = _at(NewAll, '*_with_p2pk65_prevout')
    p2pk33: NewAll[Count, Optional[CountFract]] = _at(NewAll, '*_with_p2pk33_prevout')
    p2pkh: NewAll[Count, Optional[CountFract]] = _at(NewAll, '*_with_p2pkh_prevout')
    p2ms: NewAll[Count, Optional[CountFract]] = _at(NewAll, '*_with_p2ms_prevout')
    p2sh: NewAll[Count, Optional[CountFract]] = _at(NewAll, '*_with_p2sh_prevout')
    p2wpkh: NewAll[Count, Optional[CountFract]] = _at(NewAll, '*_with_p2wpkh_prevout')
    p2wsh: NewAll[Count, Optional[CountFract]] = _at(NewAll, '*_with_p2wsh_prevout')
    p2tr: NewAll[Count, Optional[CountFract]] = _at(NewAll, '*_with_p2tr_prevout')
    p2a: NewAll[Count, Optional[CountFract]] = _at(NewAll, '*_with_p2a_prevout')
    unknown: NewAll[Count, Optional[CountFract]] = _at(NewAll, '*_with_unknown_outputs_prevout')
    empty: NewAll[Count, Optional[CountFract]] = _at(NewAll, '*_with_empty_outputs_prevout')


class InputCount(_Node):
    all: NewAll[Count, Optional[CountFract]] = _at(NewAll, 'input_*_bis')
    p2pk65: NewAll[Count, Optional[CountFract]] = _at(NewAll, 'p2pk65_prevout_*')
    p2pk33: NewAll[Count, Optional[CountFract]] = _at(NewAll, 'p2pk33_prevout_*')
    p2pkh: NewAll[Count, Optional[CountFract]] = _at(NewAll, 'p2pkh_prevout_*')
    p2ms: NewAll[Count, Optional[CountFract]] = _at(NewAll, 'p2ms_prevout_*')
    p2sh: NewAll[Count, Optional[CountFract]] = _at(NewAll, 'p2sh_prevout_*')
    p2wpkh: NewAll[Count, Optional[CountFract]] = _at(NewAll, 'p2wpkh_prevout_*')
    p2wsh: NewAll[Count, Optional[CountFract]] = _at(NewAll, 'p2wsh_prevout_*')
    p2tr: NewAll[Count, Optional[CountFract]] = _at(NewAll, 'p2tr_prevout_*')
    p2a: NewAll[Count, Optional[CountFract]] = _at(NewAll, 'p2a_prevout_*')
    unknown: NewAll[Count, Optional[CountFract]] = _at(NewAll, 'unknown_outputs_prevout_*')
    empty: NewAll[Count, Optional[CountFract]] = _at(NewAll, 'empty_outputs_prevout_*')


class Versions(_Node):
    v1: NewAll[Count, Optional[CountFract]] = _at(NewAll, '*_v1')
    v2: NewAll[Count, Optional[CountFract]] = _at(NewAll, '*_v2')
    v3: NewAll[Count, Optional[CountFract]] = _at(NewAll, '*_v3')
    other: NewAll[Count, Optional[CountFract]] = _at(NewAll, '*_other_version')


class PolicyCount(_Node):
    nonstandard: NewAll[Count, Optional[CountFract]] = _at(NewAll, '*')


class Policy(_Node):
    count: PolicyCount = _at(PolicyCount, '*_count')
    is_nonstandard: SeriesPattern22[Boolean] = _at(SeriesPattern22, 'is_*')


class PatternsCount(_Node):
    coinjoin: NewAll[Count, Optional[CountFract]] = _at(NewAll, 'coinjoin_*')
    consolidation: NewAll[Count, Optional[CountFract]] = _at(NewAll, 'consolidation_*')
    batch_payout: NewAll[Count, Optional[CountFract]] = _at(NewAll, 'batch_payout_*')


class Patterns(_Node):
    count: PatternsCount = _at(PatternsCount, 'count')
    is_coinjoin: SeriesPattern22[Boolean] = _at(SeriesPattern22, '*_coinjoin')
    is_consolidation: SeriesPattern22[Boolean] = _at(SeriesPattern22, '*_consolidation')
    is_batch_payout: SeriesPattern22[Boolean] = _at(SeriesPattern22, '*_batch_payout')


class FeesCount(_Node):
    cpfp_parent: NewAll[Count, Optional[CountFract]] = _at(NewAll, '*_parent_count')
    cpfp_child: NewAll[Count, Optional[CountFract]] = _at(NewAll, '*_child_count')


class TransactionsFees(_Node):
    count: FeesCount = _at(FeesCount, 'cpfp')
    fee: EffectiveFeeRate[Sats, EffectiveFeeRate6b[Sats]] = _at((EffectiveFeeRate, EffectiveFeeRate6b), '*')
    fee_rate: SeriesPattern22[Optional[FeeRate]] = _at(SeriesPattern22, '*_rate')
    effective_fee_rate: EffectiveFeeRate[Optional[FeeRate], EffectiveFeeRate6b[Optional[FeeRate]]] = _at((EffectiveFeeRate, EffectiveFeeRate6b), 'effective_*_rate')
    is_cpfp_parent: SeriesPattern22[Boolean] = _at(SeriesPattern22, 'is_cpfp_parent')
    is_cpfp_child: SeriesPattern22[Boolean] = _at(SeriesPattern22, 'is_cpfp_child')


class OutputsCount(_Node, Generic[A]):
    total: A = _at(0, '*')


class FeaturesCount(_Node):
    v1: SeriesPattern21[Count16] = _at(SeriesPattern21, '*_v1')
    v2: SeriesPattern21[Count16] = _at(SeriesPattern21, '*_v2')
    v3: SeriesPattern21[Count16] = _at(SeriesPattern21, '*_v3')
    other_version: SeriesPattern21[Count16] = _at(SeriesPattern21, '*_other_version')
    explicitly_rbf: SeriesPattern21[Count16] = _at(SeriesPattern21, '*_explicitly_rbf')
    one_input: SeriesPattern21[Count16] = _at(SeriesPattern21, '*_one_input')
    one_output: SeriesPattern21[Count16] = _at(SeriesPattern21, '*_one_output')
    p2pk: SeriesPattern21[Count16] = _at(SeriesPattern21, '*_p2pk')
    p2ms: SeriesPattern21[Count16] = _at(SeriesPattern21, '*_p2ms')
    p2pkh: SeriesPattern21[Count16] = _at(SeriesPattern21, '*_p2pkh')
    p2sh: SeriesPattern21[Count16] = _at(SeriesPattern21, '*_p2sh')
    p2wpkh: SeriesPattern21[Count16] = _at(SeriesPattern21, '*_p2wpkh')
    p2wsh: SeriesPattern21[Count16] = _at(SeriesPattern21, '*_p2wsh')
    p2tr: SeriesPattern21[Count16] = _at(SeriesPattern21, '*_p2tr')
    p2a: SeriesPattern21[Count16] = _at(SeriesPattern21, '*_p2a')
    op_return: SeriesPattern21[Count16] = _at(SeriesPattern21, '*_op_return')
    empty: SeriesPattern21[Count16] = _at(SeriesPattern21, '*_empty')
    unknown: SeriesPattern21[Count16] = _at(SeriesPattern21, '*_unknown')
    fake_pubkey: SeriesPattern21[Count16] = _at(SeriesPattern21, '*_fake_pubkey')
    fake_scripthash: SeriesPattern21[Count16] = _at(SeriesPattern21, '*_fake_scripthash')
    annex: NewAll[Count, Optional[CountFract]] = _at(NewAll, '*_annex')
    sighash_all: NewAll[Count, Optional[CountFract]] = _at(NewAll, '*_sighash_all')
    sighash_none: NewAll[Count, Optional[CountFract]] = _at(NewAll, '*_sighash_none')
    sighash_single: NewAll[Count, Optional[CountFract]] = _at(NewAll, '*_sighash_single')
    sighash_default: NewAll[Count, Optional[CountFract]] = _at(NewAll, '*_sighash_default')
    sighash_anyone_can_pay: NewAll[Count, Optional[CountFract]] = _at(NewAll, '*_sighash_anyone_can_pay')
    dust_output: NewAll[Count, Optional[CountFract]] = _at(NewAll, '*_dust_output')


class Features(_Node):
    count: FeaturesCount = _at(FeaturesCount, 'tx_count')
    has_p2pk: SeriesPattern22[Boolean] = _at(SeriesPattern22, '*_p2pk')
    has_p2ms: SeriesPattern22[Boolean] = _at(SeriesPattern22, '*_p2ms')
    has_p2pkh: SeriesPattern22[Boolean] = _at(SeriesPattern22, '*_p2pkh')
    has_p2sh: SeriesPattern22[Boolean] = _at(SeriesPattern22, '*_p2sh')
    has_p2wpkh: SeriesPattern22[Boolean] = _at(SeriesPattern22, '*_p2wpkh')
    has_p2wsh: SeriesPattern22[Boolean] = _at(SeriesPattern22, '*_p2wsh')
    has_p2tr: SeriesPattern22[Boolean] = _at(SeriesPattern22, '*_p2tr')
    has_p2a: SeriesPattern22[Boolean] = _at(SeriesPattern22, '*_p2a')
    has_op_return: SeriesPattern22[Boolean] = _at(SeriesPattern22, '*_op_return')
    has_empty: SeriesPattern22[Boolean] = _at(SeriesPattern22, '*_empty')
    has_unknown: SeriesPattern22[Boolean] = _at(SeriesPattern22, '*_unknown')
    has_fake_pubkey: SeriesPattern22[Boolean] = _at(SeriesPattern22, '*_fake_pubkey')
    has_fake_scripthash: SeriesPattern22[Boolean] = _at(SeriesPattern22, '*_fake_scripthash')
    has_inscription: SeriesPattern22[Boolean] = _at(SeriesPattern22, '*_inscription')
    has_annex: SeriesPattern22[Boolean] = _at(SeriesPattern22, '*_annex')
    has_sighash_all: SeriesPattern22[Boolean] = _at(SeriesPattern22, '*_sighash_all')
    has_sighash_none: SeriesPattern22[Boolean] = _at(SeriesPattern22, '*_sighash_none')
    has_sighash_single: SeriesPattern22[Boolean] = _at(SeriesPattern22, '*_sighash_single')
    has_sighash_default: SeriesPattern22[Boolean] = _at(SeriesPattern22, '*_sighash_default')
    has_sighash_anyone_can_pay: SeriesPattern22[Boolean] = _at(SeriesPattern22, '*_sighash_anyone_can_pay')
    has_dust_output: SeriesPattern22[Boolean] = _at(SeriesPattern22, '*_dust_output')


class BlocksCount(_Node):
    target: Target = _at(Target, '*_target')
    total: NewAll[Count, Optional[CountFract]] = _at(NewAll, '*')


class BlocksWeight(_Node):
    base: SeriesPattern21[Weight] = _at(SeriesPattern21, '*')
    cumulative: SeriesPattern2[Weight64] = _at(SeriesPattern2, '*_cumulative')
    sum: PerSec[Weight64] = _at(PerSec, '*_sum')
    average: PerSec[Optional[WeightFract]] = _at(PerSec, '*_average')
    min: PerSec[Weight] = _at(PerSec, '*_min')
    max: PerSec[Weight] = _at(PerSec, '*_max')
    pct10: PerSec[Weight] = _at(PerSec, '*_pct10')
    pct25: PerSec[Weight] = _at(PerSec, '*_pct25')
    median: PerSec[Weight] = _at(PerSec, '*_median')
    pct75: PerSec[Weight] = _at(PerSec, '*_pct75')
    pct90: PerSec[Weight] = _at(PerSec, '*_pct90')


class BlocksSize(_Node):
    base: SeriesPattern21[Bytes32] = _at(SeriesPattern21, 'total_*')
    cumulative: SeriesPattern2[Bytes] = _at(SeriesPattern2, 'block_*_cumulative')
    sum: PerSec[Bytes] = _at(PerSec, 'block_*_sum')
    average: PerSec[Optional[BytesFract]] = _at(PerSec, 'block_*_average')
    min: PerSec[Bytes32] = _at(PerSec, 'block_*_min')
    max: PerSec[Bytes32] = _at(PerSec, 'block_*_max')
    pct10: PerSec[Bytes32] = _at(PerSec, 'block_*_pct10')
    pct25: PerSec[Bytes32] = _at(PerSec, 'block_*_pct25')
    median: PerSec[Bytes32] = _at(PerSec, 'block_*_median')
    pct75: PerSec[Bytes32] = _at(PerSec, 'block_*_pct75')
    pct90: PerSec[Bytes32] = _at(PerSec, 'block_*_pct90')


class Time(_Node):
    timestamp: SeriesPattern21[Timestamp] = _at(SeriesPattern21, '*')


class Gini(_Node, Generic[A]):
    ppm: SeriesPattern2[A] = _at(SeriesPattern2, '*_ppm')
    ratio: SeriesPattern2[Optional[Ratio]] = _at(SeriesPattern2, '*_ratio')
    percent: SeriesPattern2[Optional[Percent]] = _at(SeriesPattern2, '*')


class Relative(_Node):
    supply_dominance: Gini[Optional[PartsPerMillion32]] = _at(Gini, '*_supply_dominance')
    supply_in_profit_share: Gini[Optional[PartsPerMillion32]] = _at(Gini, '*_supply_in_profit_share')
    supply_in_loss_share: Gini[Optional[PartsPerMillion32]] = _at(Gini, '*_supply_in_loss_share')
    unrealized_profit_to_mcap: Gini[Optional[PartsPerMillion32]] = _at(Gini, '*_unrealized_profit_to_mcap')
    unrealized_loss_to_mcap: Gini[Optional[PartsPerMillion32]] = _at(Gini, '*_unrealized_loss_to_mcap')
    unrealized_profit_to_own_mcap: Gini[Optional[PartsPerMillion32]] = _at(Gini, '*_unrealized_profit_to_own_mcap')
    unrealized_loss_to_own_mcap: Gini[Optional[PartsPerMillion32]] = _at(Gini, '*_unrealized_loss_to_own_mcap')
    unrealized_profit_to_own_gross_pnl: Gini[Optional[PartsPerMillion32]] = _at(Gini, '*_unrealized_profit_to_own_gross_pnl')
    unrealized_loss_to_own_gross_pnl: Gini[Optional[PartsPerMillion32]] = _at(Gini, '*_unrealized_loss_to_own_gross_pnl')
    net_unrealized_pnl_to_own_gross_pnl: Gini[Optional[PartsPerMillionSigned32]] = _at(Gini, '*_net_unrealized_pnl_to_own_gross_pnl')
    invested_capital_in_profit_share: Gini[Optional[PartsPerMillion32]] = _at(Gini, '*_invested_capital_in_profit_share')
    invested_capital_in_loss_share: Gini[Optional[PartsPerMillion32]] = _at(Gini, '*_invested_capital_in_loss_share')
    realized_cap_to_own_mcap: Gini[Optional[PartsPerMillion32]] = _at(Gini, '*_realized_cap_to_own_mcap')
    net_pnl_change_1m_to_mcap: Gini[Optional[PartsPerMillionSigned64]] = _at(Gini, '*_net_pnl_change_1m_to_mcap')
    net_pnl_change_1m_to_rcap: Gini[Optional[PartsPerMillionSigned64]] = _at(Gini, '*_net_pnl_change_1m_to_rcap')


class CohortsAllCostBasis(_Node):
    in_profit: UrpdAllCostBasis[Spot[Optional[SatsFract]]] = _at((UrpdAllCostBasis, Spot), '*_cost_basis_in_profit_per')
    in_loss: UrpdAllCostBasis[Spot[Optional[SatsFract]]] = _at((UrpdAllCostBasis, Spot), '*_cost_basis_in_loss_per')
    min: Spot[Optional[SatsFract]] = _at(Spot, '*_cost_basis_min')
    max: Spot[Optional[SatsFract]] = _at(Spot, '*_cost_basis_max')
    per_coin: PerCoin = _at(PerCoin, '*_cost_basis_per_coin')
    per_dollar: PerCoin = _at(PerCoin, '*_cost_basis_per_dollar')
    supply_density: Gini[Optional[PartsPerMillion32]] = _at(Gini, '*_supply_density')


class Aaopool(_Node):
    blocks_mined: BlocksMined = _at(BlocksMined, '*_blocks_mined')
    dominance: Gini[Optional[PartsPerMillion32]] = _at(Gini, '*_dominance')


class Minor(_Node):
    blockfills: Aaopool = _at(Aaopool, '*')
    ultimuspool: Aaopool = _at(Aaopool, 'ultimuspool')
    terrapool: Aaopool = _at(Aaopool, 'terrapool')
    onethash: Aaopool = _at(Aaopool, 'onethash')
    bitfarms: Aaopool = _at(Aaopool, 'bitfarms')
    huobipool: Aaopool = _at(Aaopool, 'huobipool')
    wayicn: Aaopool = _at(Aaopool, 'wayicn')
    canoepool: Aaopool = _at(Aaopool, 'canoepool')
    bitcoincom: Aaopool = _at(Aaopool, 'bitcoincom')
    pool175btc: Aaopool = _at(Aaopool, 'pool175btc')
    gbminers: Aaopool = _at(Aaopool, 'gbminers')
    axbt: Aaopool = _at(Aaopool, 'axbt')
    asicminer: Aaopool = _at(Aaopool, 'asicminer')
    bitminter: Aaopool = _at(Aaopool, 'bitminter')
    bitcoinrussia: Aaopool = _at(Aaopool, 'bitcoinrussia')
    btcserv: Aaopool = _at(Aaopool, 'btcserv')
    simplecoinus: Aaopool = _at(Aaopool, 'simplecoinus')
    ozcoin: Aaopool = _at(Aaopool, 'ozcoin')
    eclipsemc: Aaopool = _at(Aaopool, 'eclipsemc')
    maxbtc: Aaopool = _at(Aaopool, 'maxbtc')
    triplemining: Aaopool = _at(Aaopool, 'triplemining')
    coinlab: Aaopool = _at(Aaopool, 'coinlab')
    pool50btc: Aaopool = _at(Aaopool, 'pool50btc')
    ghashio: Aaopool = _at(Aaopool, 'ghashio')
    stminingcorp: Aaopool = _at(Aaopool, 'stminingcorp')
    bitparking: Aaopool = _at(Aaopool, 'bitparking')
    mmpool: Aaopool = _at(Aaopool, 'mmpool')
    polmine: Aaopool = _at(Aaopool, 'polmine')
    kncminer: Aaopool = _at(Aaopool, 'kncminer')
    bitalo: Aaopool = _at(Aaopool, 'bitalo')
    hhtt: Aaopool = _at(Aaopool, 'hhtt')
    megabigpower: Aaopool = _at(Aaopool, 'megabigpower')
    mtred: Aaopool = _at(Aaopool, 'mtred')
    nmcbit: Aaopool = _at(Aaopool, 'nmcbit')
    yourbtcnet: Aaopool = _at(Aaopool, 'yourbtcnet')
    givemecoins: Aaopool = _at(Aaopool, 'givemecoins')
    multicoinco: Aaopool = _at(Aaopool, 'multicoinco')
    bcpoolio: Aaopool = _at(Aaopool, 'bcpoolio')
    cointerra: Aaopool = _at(Aaopool, 'cointerra')
    kanopool: Aaopool = _at(Aaopool, 'kanopool')
    solock: Aaopool = _at(Aaopool, 'solock')
    ckpool: Aaopool = _at(Aaopool, 'ckpool')
    nicehash: Aaopool = _at(Aaopool, 'nicehash')
    bitclub: Aaopool = _at(Aaopool, 'bitclub')
    bitcoinaffiliatenetwork: Aaopool = _at(Aaopool, 'bitcoinaffiliatenetwork')
    exxbw: Aaopool = _at(Aaopool, 'exxbw')
    bitsolo: Aaopool = _at(Aaopool, 'bitsolo')
    twentyoneinc: Aaopool = _at(Aaopool, 'twentyoneinc')
    digitalbtc: Aaopool = _at(Aaopool, 'digitalbtc')
    eightbaochi: Aaopool = _at(Aaopool, 'eightbaochi')
    mybtccoinpool: Aaopool = _at(Aaopool, 'mybtccoinpool')
    tbdice: Aaopool = _at(Aaopool, 'tbdice')
    hashpool: Aaopool = _at(Aaopool, 'hashpool')
    nexious: Aaopool = _at(Aaopool, 'nexious')
    bravomining: Aaopool = _at(Aaopool, 'bravomining')
    hotpool: Aaopool = _at(Aaopool, 'hotpool')
    okexpool: Aaopool = _at(Aaopool, 'okexpool')
    bcmonster: Aaopool = _at(Aaopool, 'bcmonster')
    onehash: Aaopool = _at(Aaopool, 'onehash')
    bixin: Aaopool = _at(Aaopool, 'bixin')
    tatmaspool: Aaopool = _at(Aaopool, 'tatmaspool')
    connectbtc: Aaopool = _at(Aaopool, 'connectbtc')
    batpool: Aaopool = _at(Aaopool, 'batpool')
    waterhole: Aaopool = _at(Aaopool, 'waterhole')
    dcexploration: Aaopool = _at(Aaopool, 'dcexploration')
    dcex: Aaopool = _at(Aaopool, 'dcex')
    btpool: Aaopool = _at(Aaopool, 'btpool')
    fiftyeightcoin: Aaopool = _at(Aaopool, 'fiftyeightcoin')
    bitcoinindia: Aaopool = _at(Aaopool, 'bitcoinindia')
    shawnp0wers: Aaopool = _at(Aaopool, 'shawnp0wers')
    phashio: Aaopool = _at(Aaopool, 'phashio')
    rigpool: Aaopool = _at(Aaopool, 'rigpool')
    haozhuzhu: Aaopool = _at(Aaopool, 'haozhuzhu')
    sevenpool: Aaopool = _at(Aaopool, 'sevenpool')
    miningkings: Aaopool = _at(Aaopool, 'miningkings')
    hashbx: Aaopool = _at(Aaopool, 'hashbx')
    dpool: Aaopool = _at(Aaopool, 'dpool')
    rawpool: Aaopool = _at(Aaopool, 'rawpool')
    haominer: Aaopool = _at(Aaopool, 'haominer')
    helix: Aaopool = _at(Aaopool, 'helix')
    bitcoinukraine: Aaopool = _at(Aaopool, 'bitcoinukraine')
    secretsuperstar: Aaopool = _at(Aaopool, 'secretsuperstar')
    tigerpoolnet: Aaopool = _at(Aaopool, 'tigerpoolnet')
    sigmapoolcom: Aaopool = _at(Aaopool, 'sigmapoolcom')
    okpooltop: Aaopool = _at(Aaopool, 'okpooltop')
    hummerpool: Aaopool = _at(Aaopool, 'hummerpool')
    tangpool: Aaopool = _at(Aaopool, 'tangpool')
    bytepool: Aaopool = _at(Aaopool, 'bytepool')
    novablock: Aaopool = _at(Aaopool, 'novablock')
    miningcity: Aaopool = _at(Aaopool, 'miningcity')
    minerium: Aaopool = _at(Aaopool, 'minerium')
    lubiancom: Aaopool = _at(Aaopool, 'lubiancom')
    okkong: Aaopool = _at(Aaopool, 'okkong')
    aaopool: Aaopool = _at(Aaopool, 'aaopool')
    emcdpool: Aaopool = _at(Aaopool, 'emcdpool')
    arkpool: Aaopool = _at(Aaopool, 'arkpool')
    purebtccom: Aaopool = _at(Aaopool, 'purebtccom')
    kucoinpool: Aaopool = _at(Aaopool, 'kucoinpool')
    entrustcharitypool: Aaopool = _at(Aaopool, 'entrustcharitypool')
    okminer: Aaopool = _at(Aaopool, 'okminer')
    titan: Aaopool = _at(Aaopool, 'titan')
    pegapool: Aaopool = _at(Aaopool, 'pegapool')
    btcnuggets: Aaopool = _at(Aaopool, 'btcnuggets')
    cloudhashing: Aaopool = _at(Aaopool, 'cloudhashing')
    digitalxmintsy: Aaopool = _at(Aaopool, 'digitalxmintsy')
    telco214: Aaopool = _at(Aaopool, 'telco214')
    btcpoolparty: Aaopool = _at(Aaopool, 'btcpoolparty')
    multipool: Aaopool = _at(Aaopool, 'multipool')
    transactioncoinmining: Aaopool = _at(Aaopool, 'transactioncoinmining')
    btcdig: Aaopool = _at(Aaopool, 'btcdig')
    trickysbtcpool: Aaopool = _at(Aaopool, 'trickysbtcpool')
    btcmp: Aaopool = _at(Aaopool, 'btcmp')
    eobot: Aaopool = _at(Aaopool, 'eobot')
    unomp: Aaopool = _at(Aaopool, 'unomp')
    patels: Aaopool = _at(Aaopool, 'patels')
    gogreenlight: Aaopool = _at(Aaopool, 'gogreenlight')
    bitcoinindiapool: Aaopool = _at(Aaopool, 'bitcoinindiapool')
    ekanembtc: Aaopool = _at(Aaopool, 'ekanembtc')
    canoe: Aaopool = _at(Aaopool, 'canoe')
    tiger: Aaopool = _at(Aaopool, 'tiger')
    onem1x: Aaopool = _at(Aaopool, 'onem1x')
    zulupool: Aaopool = _at(Aaopool, 'zulupool')
    wiz: Aaopool = _at(Aaopool, 'wiz')
    wk057: Aaopool = _at(Aaopool, 'wk057')
    futurebitapollosolo: Aaopool = _at(Aaopool, 'futurebitapollosolo')
    carbonnegative: Aaopool = _at(Aaopool, 'carbonnegative')
    portlandhodl: Aaopool = _at(Aaopool, 'portlandhodl')
    phoenix: Aaopool = _at(Aaopool, 'phoenix')
    neopool: Aaopool = _at(Aaopool, 'neopool')
    maxipool: Aaopool = _at(Aaopool, 'maxipool')
    bitfufupool: Aaopool = _at(Aaopool, 'bitfufupool')
    gdpool: Aaopool = _at(Aaopool, 'gdpool')
    miningdutch: Aaopool = _at(Aaopool, 'miningdutch')
    publicpool: Aaopool = _at(Aaopool, 'publicpool')
    miningsquared: Aaopool = _at(Aaopool, 'miningsquared')
    innopolistech: Aaopool = _at(Aaopool, 'innopolistech')
    btclab: Aaopool = _at(Aaopool, 'btclab')
    parasite: Aaopool = _at(Aaopool, 'parasite')
    redrockpool: Aaopool = _at(Aaopool, 'redrockpool')
    est3lar: Aaopool = _at(Aaopool, 'est3lar')
    braiinssolo: Aaopool = _at(Aaopool, 'braiinssolo')
    solopool: Aaopool = _at(Aaopool, 'solopool')
    noderunners: Aaopool = _at(Aaopool, 'noderunners')
    dmnd: Aaopool = _at(Aaopool, 'dmnd')


class Rsi1m(_Node):
    rsi: Gini[Optional[PartsPerMillion32]] = _at(Gini, 'rsi_*')
    stoch_rsi_k: Gini[Optional[PartsPerMillion32]] = _at(Gini, 'rsi_stoch_k_*')
    stoch_rsi_d: Gini[Optional[PartsPerMillion32]] = _at(Gini, 'rsi_stoch_d_*')


class Macd(_Node, Generic[A]):
    _24h: A = _at(0, '*')
    _1w: A = _at(0, '1w')
    _1m: A = _at(0, '1m')


class Technical(_Node):
    rsi: Macd[Rsi1m] = _at((Macd, Rsi1m), '*')
    pi_cycle: RhodlRatio[Optional[PartsPerMillion32]] = _at(RhodlRatio, 'pi_cycle')
    macd: Macd[Macd1m] = _at((Macd, Macd1m), '*')


class Range(_Node):
    min: Max = _at(Max, '*_min')
    max: Max = _at(Max, '*_max')
    true_range: SeriesPattern2[Optional[Cents]] = _at(SeriesPattern2, '*_true_range')
    true_range_sum_2w: SeriesPattern2[Optional[Cents]] = _at(SeriesPattern2, '*_true_range_sum_2w')
    choppiness_index_2w: Gini[Optional[PartsPerMillion32]] = _at(Gini, '*_choppiness_index_2w')


class Cagr(_Node):
    _2y: Gini[Optional[PartsPerMillionSigned64]] = _at(Gini, '*_2y')
    _3y: Gini[Optional[PartsPerMillionSigned64]] = _at(Gini, '*_3y')
    _4y: Gini[Optional[PartsPerMillionSigned64]] = _at(Gini, '*_4y')
    _5y: Gini[Optional[PartsPerMillionSigned64]] = _at(Gini, '*_5y')
    _6y: Gini[Optional[PartsPerMillionSigned64]] = _at(Gini, '*_6y')
    _8y: Gini[Optional[PartsPerMillionSigned64]] = _at(Gini, '*_8y')
    _10y: Gini[Optional[PartsPerMillionSigned64]] = _at(Gini, '*_10y')


class MarketLookback(_Node, Generic[A]):
    _24h: A = _at(0, '*_24h')
    _1w: A = _at(0, '*_1w')
    _1m: A = _at(0, '*_1m')
    _3m: A = _at(0, '*_3m')
    _6m: A = _at(0, '*_6m')
    _1y: A = _at(0, '*_1y')
    _2y: A = _at(0, '*_2y')
    _3y: A = _at(0, '*_3y')
    _4y: A = _at(0, '*_4y')
    _5y: A = _at(0, '*_5y')
    _6y: A = _at(0, '*_6y')
    _8y: A = _at(0, '*_8y')
    _10y: A = _at(0, '*_10y')


class Ath(_Node):
    high: Spot[Optional[SatsFract]] = _at(Spot, '*_ath')
    drawdown: Gini[Optional[PartsPerMillionSigned32]] = _at(Gini, '*_drawdown')
    days_since: SeriesPattern2[Optional[Days]] = _at(SeriesPattern2, 'days_since_*_ath')
    years_since: SeriesPattern2[Optional[Years]] = _at(SeriesPattern2, 'years_since_*_ath')
    max_days_between: SeriesPattern2[Optional[Days]] = _at(SeriesPattern2, 'max_days_between_*_ath')
    max_years_between: SeriesPattern2[Optional[Years]] = _at(SeriesPattern2, 'max_years_between_*_ath')


class Indicators(_Node):
    puell_multiple: Nvt = _at(Nvt, 'puell_multiple')
    nvt: Nvt = _at(Nvt, 'nvt')
    gini: Gini[Optional[PartsPerMillion32]] = _at(Gini, 'gini')
    rhodl_ratio: RhodlRatio[Optional[PartsPerMillion64]] = _at(RhodlRatio, 'rhodl_ratio')
    thermo_cap_multiple: Nvt = _at(Nvt, 'thermo_cap_multiple')
    coindays_destroyed_supply_adj: SeriesPattern2[Optional[Days]] = _at(SeriesPattern2, 'coindays_*')
    coinyears_destroyed_supply_adj: SeriesPattern2[Optional[Years]] = _at(SeriesPattern2, 'coinyears_*')
    dormancy: Dormancy = _at(Dormancy, 'dormancy')
    stock_to_flow: SeriesPattern2[Optional[Years]] = _at(SeriesPattern2, 'stock_to_flow')
    seller_exhaustion: SeriesPattern2[Optional[Ratio]] = _at(SeriesPattern2, 'seller_exhaustion')


class Capitulation(_Node, Generic[A]):
    threshold_pct0_1: SeriesPattern2[A] = _at(SeriesPattern2, '*_threshold_pct0_1')
    threshold_pct0_05: SeriesPattern2[A] = _at(SeriesPattern2, '*_threshold_pct0_05')
    threshold_pct0_025: SeriesPattern2[A] = _at(SeriesPattern2, '*_threshold')
    tail: Gini[Optional[PartsPerMillion32]] = _at(Gini, '*_tail')
    rank: SeriesPattern2[Rank] = _at(SeriesPattern2, '*_rank')


class Extremes(_Node):
    coins_in_loss: Capitulation[Optional[Bitcoin]] = _at(Capitulation, '*_coins_in_loss')
    profit_taking: Capitulation[Optional[Dollars]] = _at(Capitulation, '*_profit_taking')
    capitulation: Capitulation[Optional[Dollars]] = _at(Capitulation, '*_capitulation')
    peak_regret: Capitulation[Optional[Dollars]] = _at(Capitulation, '*_peak_regret')
    seller_exhaustion: Capitulation[Optional[Percent]] = _at(Capitulation, '*_seller_exhaustion')


class RarityMeter(_Node):
    components: Components = _at(Components, 'price')
    extremes: Extremes = _at(Extremes, '*')
    full: Cycle = _at(Cycle, '*')
    full_v2: Cycle = _at(Cycle, '*_v2')
    local: Cycle = _at(Cycle, 'local_*')
    local_v2: Cycle = _at(Cycle, 'local_*_v2')
    cycle: Cycle = _at(Cycle, 'cycle_*')
    cycle_v2: Cycle = _at(Cycle, 'cycle_*_v2')


class Adjusted(_Node):
    inflation_rate: Gini[Optional[PartsPerMillionSigned32]] = _at(Gini, '*_inflation_rate')
    tx_velocity_native: SeriesPattern2[Optional[Ratio64]] = _at(SeriesPattern2, '*_tx_velocity_btc')
    tx_velocity_fiat: SeriesPattern2[Optional[Ratio64]] = _at(SeriesPattern2, '*_tx_velocity_usd')


class SupplyDensity(_Node):
    total: Gini[Optional[PartsPerMillion32]] = _at(Gini, '*_total')
    in_profit: Gini[Optional[PartsPerMillion32]] = _at(Gini, '*_in_profit')
    in_loss: Gini[Optional[PartsPerMillion32]] = _at(Gini, '*_in_loss')


class CoinflowUrpdLth(_Node):
    cost_basis: UrpdAllCostBasis[PerCoin] = _at((UrpdAllCostBasis, PerCoin), '*_coinflow_cost_basis_per')
    capitalized_price: CapitalizedPrice = _at(CapitalizedPrice, 'coinflow_urpd_*_capitalized_price')
    supply_density: SupplyDensity = _at(SupplyDensity, 'coinflow_urpd_*_supply_density')


class CointimeUrpdLth(_Node):
    cost_basis: UrpdAllCostBasis[PerCoin] = _at((UrpdAllCostBasis, PerCoin), '*_cointime_cost_basis_per')
    capitalized_price: CapitalizedPrice = _at(CapitalizedPrice, 'cointime_urpd_*_capitalized_price')
    supply_density: SupplyDensity = _at(SupplyDensity, 'cointime_urpd_*_supply_density')


class UrpdAll(_Node):
    cost_basis: UrpdAllCostBasis[PerCoin] = _at((UrpdAllCostBasis, PerCoin), '*_cost_basis_per')
    capitalized_price: CapitalizedPrice = _at(CapitalizedPrice, '*_urpd_all_capitalized_price')
    supply_density: SupplyDensity = _at(SupplyDensity, '*_urpd_all_supply_density')


class CoinflowUrpd(_Node, Generic[A]):
    all: UrpdAll = _at(UrpdAll, '*')
    sth: A = _at(0, 'sth')
    lth: A = _at(0, 'lth')
    under_4m: A = _at(0, 'under_4m')
    under_6m: A = _at(0, 'under_6m')
    over_4m: A = _at(0, 'over_4m')
    over_6m: A = _at(0, 'over_6m')


class HashratePrice(_Node):
    ths: SeriesPattern2[Optional[Float32]] = _at(SeriesPattern2, '*_ths')
    ths_min: SeriesPattern2[Optional[Float32]] = _at(SeriesPattern2, '*_ths_min')
    phs: SeriesPattern2[Optional[Float32]] = _at(SeriesPattern2, '*_phs')
    phs_min: SeriesPattern2[Optional[Float32]] = _at(SeriesPattern2, '*_phs_min')
    rebound: Gini[Optional[PartsPerMillionSigned32]] = _at(Gini, '*_rebound')


class HashrateRate(_Node):
    base: SeriesPattern2[Optional[Hashrate]] = _at(SeriesPattern2, '*')
    sma: RateSma = _at(RateSma, '*_sma')
    ath: SeriesPattern2[Optional[Hashrate]] = _at(SeriesPattern2, '*_ath')
    drawdown: Gini[Optional[PartsPerMillionSigned32]] = _at(Gini, '*_drawdown')


class MiningHashrate(_Node):
    rate: HashrateRate = _at(HashrateRate, '*_rate')
    price: HashratePrice = _at(HashratePrice, '*_price')
    value: HashratePrice = _at(HashratePrice, '*_value')


class DataBytesAscribe(_Node):
    block: SeriesPattern21[Bytes] = _at(SeriesPattern21, '*_data_bytes')
    cumulative: SeriesPattern2[Bytes] = _at(SeriesPattern2, '*_data_bytes_cumulative')
    sum: PerSec[Bytes] = _at(PerSec, '*_data_bytes_sum')
    average: PerSec[Optional[BytesFract]] = _at(PerSec, '*_data_bytes_average')
    data_share: Gini[Optional[PartsPerMillion32]] = _at(Gini, '*_data_share')
    chain_share: Gini[Optional[PartsPerMillion32]] = _at(Gini, '*_chain_share')


class PolicyDataBytes(_Node, Generic[A]):
    pre_v30_standard: A = _at(0, 'op_return_policy_pre_v30_standard_*')
    pre_v30_nonstandard: A = _at(0, 'op_return_policy_pre_v30_nonstandard_*')
    oversized: A = _at(0, 'op_return_policy_oversized_*')
    multiple: A = _at(0, 'op_return_policy_multiple_*')


class ByKindDataBytes(_Node, Generic[A]):
    runes: A = _at(0, 'op_return_runes_*')
    veri_block: A = _at(0, 'op_return_veri_block_*')
    omni: A = _at(0, 'op_return_omni_*')
    stacks: A = _at(0, 'op_return_stacks_*')
    blockstack: A = _at(0, 'op_return_blockstack_*')
    colu: A = _at(0, 'op_return_colu_*')
    open_assets: A = _at(0, 'op_return_open_assets_*')
    komodo: A = _at(0, 'op_return_komodo_*')
    coin_spark: A = _at(0, 'op_return_coin_spark_*')
    poet: A = _at(0, 'op_return_poet_*')
    docproof: A = _at(0, 'op_return_docproof_*')
    open_timestamps: A = _at(0, 'op_return_open_timestamps_*')
    factom: A = _at(0, 'op_return_factom_*')
    eternity_wall: A = _at(0, 'op_return_eternity_wall_*')
    memo: A = _at(0, 'op_return_memo_*')
    bitproof: A = _at(0, 'op_return_bitproof_*')
    ascribe: A = _at(0, 'op_return_ascribe_*')
    stampery: A = _at(0, 'op_return_stampery_*')
    epobc: A = _at(0, 'op_return_epobc_*')
    bare_hash: A = _at(0, 'op_return_bare_hash_*')
    text: A = _at(0, 'op_return_text_*')
    empty: A = _at(0, 'op_return_empty_*')
    unknown: A = _at(0, 'op_return_unknown_*')


class AllRate(_Node):
    _24h: Gini[Optional[PartsPerMillionSigned64]] = _at(Gini, '*_24h_rate')
    _1w: Gini[Optional[PartsPerMillionSigned64]] = _at(Gini, '*_1w_rate')
    _1m: Gini[Optional[PartsPerMillionSigned64]] = _at(Gini, '*_1m_rate')
    _1y: Gini[Optional[PartsPerMillionSigned64]] = _at(Gini, '*_1y_rate')


class FeeShare(_Node):
    ppm: SeriesPattern2[Optional[PartsPerMillion32]] = _at(SeriesPattern2, '*_ppm')
    ratio: SeriesPattern2[Optional[Ratio]] = _at(SeriesPattern2, '*_ratio')
    percent: SeriesPattern2[Optional[Percent]] = _at(SeriesPattern2, '*')
    _24h: Gini[Optional[PartsPerMillion32]] = _at(Gini, '*_24h')
    _1w: Gini[Optional[PartsPerMillion32]] = _at(Gini, '*_1w')
    _1m: Gini[Optional[PartsPerMillion32]] = _at(Gini, '*_1m')
    _1y: Gini[Optional[PartsPerMillion32]] = _at(Gini, '*_1y')


class FeesAscribe(_Node):
    block: SeriesPattern21[Sats] = _at(SeriesPattern21, '*_fees')
    cumulative: SeriesPattern2[Sats] = _at(SeriesPattern2, '*_fees_cumulative')
    sum: PerSec[Sats] = _at(PerSec, '*_fees_sum')
    average: PerSec[Optional[SatsFract]] = _at(PerSec, '*_fees_average')
    fee_share: FeeShare = _at(FeeShare, '*_fee_share')


class PolicyFees(_Node):
    pre_v30_standard: FeesAscribe = _at(FeesAscribe, '*_pre_v30_standard')
    pre_v30_nonstandard: FeesAscribe = _at(FeesAscribe, '*_pre_v30_nonstandard')
    oversized: FeesAscribe = _at(FeesAscribe, '*_oversized')
    multiple: FeesAscribe = _at(FeesAscribe, '*_multiple')


class ByKindFees(_Node):
    runes: FeesAscribe = _at(FeesAscribe, '*_runes')
    veri_block: FeesAscribe = _at(FeesAscribe, '*_veri_block')
    omni: FeesAscribe = _at(FeesAscribe, '*_omni')
    stacks: FeesAscribe = _at(FeesAscribe, '*_stacks')
    blockstack: FeesAscribe = _at(FeesAscribe, '*_blockstack')
    colu: FeesAscribe = _at(FeesAscribe, '*_colu')
    open_assets: FeesAscribe = _at(FeesAscribe, '*_open_assets')
    komodo: FeesAscribe = _at(FeesAscribe, '*_komodo')
    coin_spark: FeesAscribe = _at(FeesAscribe, '*_coin_spark')
    poet: FeesAscribe = _at(FeesAscribe, '*_poet')
    docproof: FeesAscribe = _at(FeesAscribe, '*_docproof')
    open_timestamps: FeesAscribe = _at(FeesAscribe, '*_open_timestamps')
    factom: FeesAscribe = _at(FeesAscribe, '*_factom')
    eternity_wall: FeesAscribe = _at(FeesAscribe, '*_eternity_wall')
    memo: FeesAscribe = _at(FeesAscribe, '*_memo')
    bitproof: FeesAscribe = _at(FeesAscribe, '*_bitproof')
    ascribe: FeesAscribe = _at(FeesAscribe, '*_ascribe')
    stampery: FeesAscribe = _at(FeesAscribe, '*_stampery')
    epobc: FeesAscribe = _at(FeesAscribe, '*_epobc')
    bare_hash: FeesAscribe = _at(FeesAscribe, '*_bare_hash')
    text: FeesAscribe = _at(FeesAscribe, '*_text')
    empty: FeesAscribe = _at(FeesAscribe, '*_empty')
    unknown: FeesAscribe = _at(FeesAscribe, '*_unknown')


class ByKind(_Node, Generic[A, B, C, D]):
    output_count: A = _at(0, 'output_count')
    data_bytes: B = _at(1, '')
    tx_count: A = _at(0, 'tx_count')
    tx_vsize: C = _at(2, 'tx_vsize')
    fees: D = _at(3, '*')


class Total(_Node):
    data_bytes: NewAll[Bytes, Optional[BytesFract]] = _at(NewAll, '*_data_bytes')
    tx_count: NewAll[Count, Optional[CountFract]] = _at(NewAll, '*_tx_count')
    tx_vsize: NewAll[VSize, Optional[VSizeFract]] = _at(NewAll, '*_tx_vsize')
    fees: NewAll[Sats, Optional[SatsFract]] = _at(NewAll, '*_fees')
    chain_share: Gini[Optional[PartsPerMillion32]] = _at(Gini, '*_chain_share')
    fee_share: FeeShare = _at(FeeShare, '*_fee_share')


class OpReturn(_Node):
    raw: OpReturnRaw = _at(OpReturnRaw, 'index')
    total: Total = _at(Total, '*')
    by_kind: ByKind[ByKindDataBytes[NewAll[Count, Optional[CountFract]]], ByKindDataBytes[DataBytesAscribe], ByKindDataBytes[NewAll[VSize, Optional[VSizeFract]]], ByKindFees] = _at((ByKind, (ByKindDataBytes, NewAll), (ByKindDataBytes, DataBytesAscribe), (ByKindDataBytes, NewAll), ByKindFees), '*')
    policy: ByKind[PolicyDataBytes[NewAll[Count, Optional[CountFract]]], PolicyDataBytes[DataBytesAscribe], PolicyDataBytes[NewAll[VSize, Optional[VSizeFract]]], PolicyFees] = _at((ByKind, (PolicyDataBytes, NewAll), (PolicyDataBytes, DataBytesAscribe), (PolicyDataBytes, NewAll), PolicyFees), '*_policy')


class OutputsByTypeTxShare(_Node):
    p2pk65: FeeShare = _at(FeeShare, '*_p2pk65_output')
    p2pk33: FeeShare = _at(FeeShare, '*_p2pk33_output')
    p2pkh: FeeShare = _at(FeeShare, '*_p2pkh_output')
    p2ms: FeeShare = _at(FeeShare, '*_p2ms_output')
    p2sh: FeeShare = _at(FeeShare, '*_p2sh_output')
    p2wpkh: FeeShare = _at(FeeShare, '*_p2wpkh_output')
    p2wsh: FeeShare = _at(FeeShare, '*_p2wsh_output')
    p2tr: FeeShare = _at(FeeShare, '*_p2tr_output')
    p2a: FeeShare = _at(FeeShare, '*_p2a_output')
    unknown: FeeShare = _at(FeeShare, '*_unknown_outputs_output')
    empty: FeeShare = _at(FeeShare, '*_empty_outputs_output')
    op_return: FeeShare = _at(FeeShare, '*_op_return_output')


class OutputShare(_Node):
    p2pk65: FeeShare = _at(FeeShare, 'p2pk65_*')
    p2pk33: FeeShare = _at(FeeShare, 'p2pk33_*')
    p2pkh: FeeShare = _at(FeeShare, 'p2pkh_*')
    p2ms: FeeShare = _at(FeeShare, 'p2ms_*')
    p2sh: FeeShare = _at(FeeShare, 'p2sh_*')
    p2wpkh: FeeShare = _at(FeeShare, 'p2wpkh_*')
    p2wsh: FeeShare = _at(FeeShare, 'p2wsh_*')
    p2tr: FeeShare = _at(FeeShare, 'p2tr_*')
    p2a: FeeShare = _at(FeeShare, 'p2a_*')
    unknown: FeeShare = _at(FeeShare, 'unknown_outputs_*')
    empty: FeeShare = _at(FeeShare, 'empty_outputs_*')
    op_return: FeeShare = _at(FeeShare, 'op_return_*')


class OutputsByType(_Node):
    output_count: OutputCount = _at(OutputCount, '*_count')
    spendable_output_count: NewAll[Count, Optional[CountFract]] = _at(NewAll, 'spendable_*_count')
    output_share: OutputShare = _at(OutputShare, '*_share')
    tx_count: OutputsByTypeTxCount = _at(OutputsByTypeTxCount, 'tx_count')
    tx_share: OutputsByTypeTxShare = _at(OutputsByTypeTxShare, 'tx_share_with')


class Outputs(_Node):
    raw: OutputsRaw = _at(OutputsRaw, 'type')
    spent: Spent = _at(Spent, 'txin_index')
    count: OutputsCount[InputsCount[Count32]] = _at((OutputsCount, InputsCount), '*_count')
    per_sec: PerSec[Optional[PerSecond]] = _at(PerSec, 'outputs_per_sec')
    by_type: OutputsByType = _at(OutputsByType, '*')
    value: OutputsValue = _at(OutputsValue, 'op_return_value')


class InputsByTypeTxShare(_Node):
    p2pk65: FeeShare = _at(FeeShare, '*_p2pk65_prevout')
    p2pk33: FeeShare = _at(FeeShare, '*_p2pk33_prevout')
    p2pkh: FeeShare = _at(FeeShare, '*_p2pkh_prevout')
    p2ms: FeeShare = _at(FeeShare, '*_p2ms_prevout')
    p2sh: FeeShare = _at(FeeShare, '*_p2sh_prevout')
    p2wpkh: FeeShare = _at(FeeShare, '*_p2wpkh_prevout')
    p2wsh: FeeShare = _at(FeeShare, '*_p2wsh_prevout')
    p2tr: FeeShare = _at(FeeShare, '*_p2tr_prevout')
    p2a: FeeShare = _at(FeeShare, '*_p2a_prevout')
    unknown: FeeShare = _at(FeeShare, '*_unknown_outputs_prevout')
    empty: FeeShare = _at(FeeShare, '*_empty_outputs_prevout')


class Sd24h(_Node, Generic[A]):
    _24h: A = _at(0, '*_24h')
    _1w: A = _at(0, '*_1w')
    _1m: A = _at(0, '*_1m')
    _1y: A = _at(0, '*_1y')


class Returns(_Node):
    periods: MarketLookback[Gini[Optional[PartsPerMillionSigned64]]] = _at((MarketLookback, Gini), '*_return')
    cagr: Cagr = _at(Cagr, '*_cagr')
    sd_24h: Sd24h[Sd24h1m] = _at((Sd24h, Sd24h1m), '')


class Market(_Node):
    ath: Ath = _at(Ath, '*')
    lookback: MarketLookback[Spot[Optional[SatsFract]]] = _at((MarketLookback, Spot), '*_past')
    returns: Returns = _at(Returns, '*')
    volatility: PerSec[Optional[Ratio]] = _at(PerSec, '*_volatility')
    range: Range = _at(Range, '*')
    moving_average: MovingAverage = _at(MovingAverage, '*')
    technical: Technical = _at(Technical, '24h')


class RewardsFees(_Node):
    block: BurnedBlock = _at(BurnedBlock, '*')
    cumulative: Circulating[Sats, Optional[Cents]] = _at(Circulating, '*_cumulative')
    sum: Sd24h[Circulating[Sats, Optional[Cents]]] = _at((Sd24h, Circulating), '*_sum')
    average: Sd24h[Circulating[Optional[SatsFract], Optional[CentsFract]]] = _at((Sd24h, Circulating), '*_average')
    min: Sd24h[Circulating[Sats, Optional[Cents]]] = _at((Sd24h, Circulating), '*_min')
    max: Sd24h[Circulating[Sats, Optional[Cents]]] = _at((Sd24h, Circulating), '*_max')
    pct10: Sd24h[Circulating[Sats, Optional[Cents]]] = _at((Sd24h, Circulating), '*_pct10')
    pct25: Sd24h[Circulating[Sats, Optional[Cents]]] = _at((Sd24h, Circulating), '*_pct25')
    median: Sd24h[Circulating[Sats, Optional[Cents]]] = _at((Sd24h, Circulating), '*_median')
    pct75: Sd24h[Circulating[Sats, Optional[Cents]]] = _at((Sd24h, Circulating), '*_pct75')
    pct90: Sd24h[Circulating[Sats, Optional[Cents]]] = _at((Sd24h, Circulating), '*_pct90')
    dominance: FeeShare = _at(FeeShare, 'fee_dominance')
    to_subsidy: Sd24h[Gini[Optional[PartsPerMillion64]]] = _at((Sd24h, Gini), 'fee_to_subsidy')


class Subsidy(_Node):
    block: BurnedBlock = _at(BurnedBlock, '*')
    cumulative: Circulating[Sats, Optional[Cents]] = _at(Circulating, '*_cumulative')
    sum: Sd24h[Circulating[Sats, Optional[Cents]]] = _at((Sd24h, Circulating), '*_sum')
    average: Sd24h[Circulating[Optional[SatsFract], Optional[CentsFract]]] = _at((Sd24h, Circulating), '*_average')
    dominance: FeeShare = _at(FeeShare, '*_dominance')


class RealizedLoss0sats(_Node):
    block: RealizedLoss0satsBlock[Optional[Cents]] = _at(RealizedLoss0satsBlock, '*')
    cumulative: CoinflowCap[Optional[Cents]] = _at(CoinflowCap, '*_cumulative')
    sum: Sd24h[CoinflowCap[Optional[Cents]]] = _at((Sd24h, CoinflowCap), '*_sum')


class DeltaAll(_Node, Generic[A]):
    absolute: A = _at(0, '*')
    rate: AllRate = _at(AllRate, '*')


class MarketCap(_Node):
    usd: SeriesPattern2[Optional[Dollars]] = _at(SeriesPattern2, '*')
    cents: SeriesPattern2[Optional[Cents]] = _at(SeriesPattern2, '*_cents')
    delta: DeltaAll[Sd24h[CoinflowCap[CentsSigned]]] = _at((DeltaAll, (Sd24h, CoinflowCap)), '*_delta')


class Supply(_Node):
    circulating: Circulating[Sats, Optional[Cents]] = _at(Circulating, 'circulating_*')
    burned: Burned = _at(Burned, 'unspendable_*')
    inflation_rate: Gini[Optional[PartsPerMillionSigned64]] = _at(Gini, 'inflation_rate')
    velocity: Velocity = _at(Velocity, 'velocity')
    market_cap: MarketCap = _at(MarketCap, 'market_cap')
    market_minus_realized_cap_growth_rate: PerSec[Optional[PartsPerMillionSigned64]] = _at(PerSec, 'market_minus_realized_cap_growth_rate')
    hodled_or_lost: Circulating[Sats, Optional[Cents]] = _at(Circulating, 'hodled_or_lost_*')


class AllSupply(_Node):
    total: Circulating[Sats, Optional[Cents]] = _at(Circulating, '*')
    in_profit: Circulating[Sats, Optional[Cents]] = _at(Circulating, '*_in_profit')
    in_loss: Circulating[Sats, Optional[Cents]] = _at(Circulating, '*_in_loss')
    delta: DeltaAll[Sd24h[Absolute1m]] = _at((DeltaAll, (Sd24h, Absolute1m)), '*_delta')


class Age10yTo12y(_Node):
    block: RealizedLoss0satsBlock[CentsSigned] = _at(RealizedLoss0satsBlock, '*')
    cumulative: CoinflowCap[CentsSigned] = _at(CoinflowCap, '*_cumulative')
    sum: Sd24h[CoinflowCap[CentsSigned]] = _at((Sd24h, CoinflowCap), '*_sum')
    delta: DeltaAll[Sd24h[CoinflowCap[CentsSigned]]] = _at((DeltaAll, (Sd24h, CoinflowCap)), '*_delta')


class AvgBalance(_Node, Generic[A]):
    all: A = _at(0, '*')
    p2pk65: A = _at(0, 'p2pk65_*')
    p2pk33: A = _at(0, 'p2pk33_*')
    p2pkh: A = _at(0, 'p2pkh_*')
    p2sh: A = _at(0, 'p2sh_*')
    p2wpkh: A = _at(0, 'p2wpkh_*')
    p2wsh: A = _at(0, 'p2wsh_*')
    p2tr: A = _at(0, 'p2tr_*')
    p2a: A = _at(0, 'p2a_*')


class ExposedSupply(_Node):
    all: Circulating[Sats, Optional[Cents]] = _at(Circulating, '*')
    p2pk65: Circulating[Sats, Optional[Cents]] = _at(Circulating, 'p2pk65_*')
    p2pk33: Circulating[Sats, Optional[Cents]] = _at(Circulating, 'p2pk33_*')
    p2pkh: Circulating[Sats, Optional[Cents]] = _at(Circulating, 'p2pkh_*')
    p2sh: Circulating[Sats, Optional[Cents]] = _at(Circulating, 'p2sh_*')
    p2wpkh: Circulating[Sats, Optional[Cents]] = _at(Circulating, 'p2wpkh_*')
    p2wsh: Circulating[Sats, Optional[Cents]] = _at(Circulating, 'p2wsh_*')
    p2tr: Circulating[Sats, Optional[Cents]] = _at(Circulating, 'p2tr_*')
    p2a: Circulating[Sats, Optional[Cents]] = _at(Circulating, 'p2a_*')
    share: AvgBalance[Gini[Optional[PartsPerMillion32]]] = _at((AvgBalance, Gini), '*_share')


class Exposed(_Node):
    count: ExposedCount = _at(ExposedCount, '*_count')
    supply: ExposedSupply = _at(ExposedSupply, '*_supply')


class Events(_Node):
    output_to_reused_addr_count: AvgBalance[NewAll[Count, Optional[CountFract]]] = _at((AvgBalance, NewAll), 'output_to_*_count')
    output_to_reused_addr_share: AvgBalance[FeeShare] = _at((AvgBalance, FeeShare), 'output_to_*_share')
    spendable_output_to_reused_addr_share: FeeShare = _at(FeeShare, 'spendable_output_to_*_share')
    input_from_reused_addr_count: AvgBalance[NewAll[Count, Optional[CountFract]]] = _at((AvgBalance, NewAll), 'input_from_*_count')
    input_from_reused_addr_share: AvgBalance[FeeShare] = _at((AvgBalance, FeeShare), 'input_from_*_share')
    active_reused_addr_count: Interval[Count, Optional[CountFract]] = _at(Interval, 'active_*_count')
    active_reused_addr_share: FeeShare = _at(FeeShare, 'active_*_share')


class Respent(_Node):
    count: ExposedCount = _at(ExposedCount, '*_count')
    events: Events = _at(Events, '*')
    supply: ExposedSupply = _at(ExposedSupply, '*_supply')


class AddrsActivity(_Node):
    reactivated: AvgBalance[Interval[Count, Optional[CountFract]]] = _at((AvgBalance, Interval), 'reactivated_*')
    sending: AvgBalance[Interval[Count, Optional[CountFract]]] = _at((AvgBalance, Interval), 'sending_*')
    receiving: AvgBalance[Interval[Count, Optional[CountFract]]] = _at((AvgBalance, Interval), 'receiving_*')
    bidirectional: AvgBalance[Interval[Count, Optional[CountFract]]] = _at((AvgBalance, Interval), 'bidirectional_*')
    active: AvgBalance[Interval[Count, Optional[CountFract]]] = _at((AvgBalance, Interval), 'active_*')


class UtxoCount0sats(_Node):
    base: SeriesPattern2[Count] = _at(SeriesPattern2, '*')
    delta: DeltaAll[PerSec[CountSigned]] = _at((DeltaAll, PerSec), '*_delta')


class AllOutputs(_Node):
    unspent_count: UtxoCount0sats = _at(UtxoCount0sats, '*_utxo_count')
    spent_count: NewAll[Count, Optional[CountFract]] = _at(NewAll, '*_spent_utxo_count')


class Supply0sats(_Node):
    total: Circulating[Sats, Optional[Cents]] = _at(Circulating, '*')
    delta: DeltaAll[Sd24h[Absolute1m]] = _at((DeltaAll, (Sd24h, Absolute1m)), '*_delta')
    dominance: Gini[Optional[PartsPerMillion32]] = _at(Gini, '*_dominance')


class Coinbase(_Node, Generic[A, B, C]):
    block: A = _at(0, '*')
    cumulative: B = _at(1, '*_cumulative')
    sum: Sd24h[B] = _at((Sd24h, 1), '*_sum')
    average: Sd24h[C] = _at((Sd24h, 2), '*_average')


class AdjustedSopr(_Node):
    ratio: PerSec[Optional[Ratio]] = _at(PerSec, '*_adjusted_sopr')
    transfer_volume: Coinbase[RealizedLoss0satsBlock[Optional[Cents]], CoinflowCap[Optional[Cents]], CoinflowCap[Optional[CentsFract]]] = _at((Coinbase, RealizedLoss0satsBlock, CoinflowCap, CoinflowCap), '*_adj_value_created')
    value_destroyed: Coinbase[RealizedLoss0satsBlock[Optional[Cents]], CoinflowCap[Optional[Cents]], CoinflowCap[Optional[CentsFract]]] = _at((Coinbase, RealizedLoss0satsBlock, CoinflowCap, CoinflowCap), '*_adj_value_destroyed')


class Ratios(_Node):
    adjusted_sopr: AdjustedSopr = _at(AdjustedSopr, '*')
    dormancy: PerSec[Optional[Days]] = _at(PerSec, '*_dormancy')
    sopr: SeriesPattern2[Optional[Ratio]] = _at(SeriesPattern2, '*_sopr_24h')
    sopr_ratio_extended: SoprRatioExtended = _at(SoprRatioExtended, '*_sopr')
    sell_side_risk_ratio: Sd24h[Gini[Optional[PartsPerMillion32]]] = _at((Sd24h, Gini), '*_sell_side_risk_ratio')
    profit_to_loss_ratio: PerSec[Optional[Ratio]] = _at(PerSec, '*_realized_profit_to_loss_ratio')


class AllRealized(_Node):
    cap: MarketCap = _at(MarketCap, '*_realized_cap')
    price: Spot[Optional[SatsFract]] = _at(Spot, '*_realized_price')
    capitalized_price: CapitalizedPrice = _at(CapitalizedPrice, '*_capitalized_price')
    profit: RealizedLoss0sats = _at(RealizedLoss0sats, '*_realized_profit')
    loss: RealizedLoss0sats = _at(RealizedLoss0sats, '*_realized_loss')
    net_pnl: Age10yTo12y = _at(Age10yTo12y, '*_net_realized_pnl')
    value_destroyed: Coinbase[RealizedLoss0satsBlock[Optional[Cents]], CoinflowCap[Optional[Cents]], CoinflowCap[Optional[CentsFract]]] = _at((Coinbase, RealizedLoss0satsBlock, CoinflowCap, CoinflowCap), '*_value_destroyed')
    gross_pnl: RealizedLoss0sats = _at(RealizedLoss0sats, '*_realized_gross_pnl')
    peak_regret: RealizedLoss0sats = _at(RealizedLoss0sats, '*_realized_peak_regret')
    mvrv: RhodlRatio[Optional[PriceRatio]] = _at(RhodlRatio, '*_mvrv')


class AllActivity(_Node):
    transfer_volume: Coinbase[BurnedBlock, Circulating[Sats, Optional[Cents]], Circulating[Optional[SatsFract], Optional[CentsFract]]] = _at((Coinbase, BurnedBlock, Circulating, Circulating), '*_transfer_volume')
    transfer_volume_in_profit: Coinbase[BurnedBlock, Circulating[Sats, Optional[Cents]], Circulating[Optional[SatsFract], Optional[CentsFract]]] = _at((Coinbase, BurnedBlock, Circulating, Circulating), '*_transfer_volume_in_profit')
    transfer_volume_in_loss: Coinbase[BurnedBlock, Circulating[Sats, Optional[Cents]], Circulating[Optional[SatsFract], Optional[CentsFract]]] = _at((Coinbase, BurnedBlock, Circulating, Circulating), '*_transfer_volume_in_loss')
    coindays_destroyed: NewAll[Optional[CoinDays], Optional[CoinDays]] = _at(NewAll, '*_coindays_destroyed')
    coinyears_destroyed: SeriesPattern2[Optional[CoinYears]] = _at(SeriesPattern2, '*_coinyears_destroyed')


class CohortsAll(_Node):
    supply: AllSupply = _at(AllSupply, '*_supply')
    outputs: AllOutputs = _at(AllOutputs, '*')
    activity: AllActivity = _at(AllActivity, '*')
    realized: AllRealized = _at(AllRealized, '*')
    unrealized: AllUnrealized = _at(AllUnrealized, '*')
    cost_basis: CohortsAllCostBasis = _at(CohortsAllCostBasis, '*')
    ratios: Ratios = _at(Ratios, '*')
    relative: Relative = _at(Relative, '*')


class DistributionAggregatedCohorts(_Node):
    all: CohortsAll = _at(CohortsAll, '')
    sth: CohortsAll = _at(CohortsAll, 'sth')
    lth: CohortsAll = _at(CohortsAll, 'lth')
    under_4m: CohortsAll = _at(CohortsAll, '*_4m')
    under_6m: CohortsAll = _at(CohortsAll, '*_6m')
    over_4m: CohortsAll = _at(CohortsAll, 'over_4m')
    over_6m: CohortsAll = _at(CohortsAll, 'over_6m')


class DistributionAggregated(_Node):
    cohorts: DistributionAggregatedCohorts = _at(DistributionAggregatedCohorts, '*')


class UtxoAmount(_Node, Generic[A]):
    _0sats: A = _at(0, 'utxos_0sats_*')
    _1sat_to_10sats: A = _at(0, 'utxos_1sat_to_10sats_*')
    _10sats_to_100sats: A = _at(0, 'utxos_10sats_to_100sats_*')
    _100sats_to_1k_sats: A = _at(0, 'utxos_100sats_to_1k_sats_*')
    _1k_sats_to_10k_sats: A = _at(0, 'utxos_1k_sats_to_10k_sats_*')
    _10k_sats_to_100k_sats: A = _at(0, 'utxos_10k_sats_to_100k_sats_*')
    _100k_sats_to_1m_sats: A = _at(0, 'utxos_100k_sats_to_1m_sats_*')
    _1m_sats_to_10m_sats: A = _at(0, 'utxos_1m_sats_to_10m_sats_*')
    _10m_sats_to_1btc: A = _at(0, 'utxos_10m_sats_to_1btc_*')
    _1btc_to_10btc: A = _at(0, 'utxos_1btc_to_10btc_*')
    _10btc_to_100btc: A = _at(0, 'utxos_10btc_to_100btc_*')
    _100btc_to_1k_btc: A = _at(0, 'utxos_100btc_to_1k_btc_*')
    _1k_btc_to_10k_btc: A = _at(0, 'utxos_1k_btc_to_10k_btc_*')
    _10k_btc_to_100k_btc: A = _at(0, 'utxos_10k_btc_to_100k_btc_*')
    over_100k_btc: A = _at(0, 'utxos_over_100k_btc_*')


class Class(_Node, Generic[A]):
    _2009: A = _at(0, 'class_2009_*')
    _2010: A = _at(0, 'class_2010_*')
    _2011: A = _at(0, 'class_2011_*')
    _2012: A = _at(0, 'class_2012_*')
    _2013: A = _at(0, 'class_2013_*')
    _2014: A = _at(0, 'class_2014_*')
    _2015: A = _at(0, 'class_2015_*')
    _2016: A = _at(0, 'class_2016_*')
    _2017: A = _at(0, 'class_2017_*')
    _2018: A = _at(0, 'class_2018_*')
    _2019: A = _at(0, 'class_2019_*')
    _2020: A = _at(0, 'class_2020_*')
    _2021: A = _at(0, 'class_2021_*')
    _2022: A = _at(0, 'class_2022_*')
    _2023: A = _at(0, 'class_2023_*')
    _2024: A = _at(0, 'class_2024_*')
    _2025: A = _at(0, 'class_2025_*')
    _2026: A = _at(0, 'class_2026_*')


class CoindaysDestroyedEpoch(_Node, Generic[A]):
    _0: A = _at(0, 'epoch_0_*')
    _1: A = _at(0, 'epoch_1_*')
    _2: A = _at(0, 'epoch_2_*')
    _3: A = _at(0, 'epoch_3_*')
    _4: A = _at(0, 'epoch_4_*')


class Antpool(_Node):
    blocks_mined: BlocksMined = _at(BlocksMined, '*_blocks_mined')
    dominance: FeeShare = _at(FeeShare, '*_dominance')
    rewards: Coinbase[BurnedBlock, Circulating[Sats, Optional[Cents]], Circulating[Optional[SatsFract], Optional[CentsFract]]] = _at((Coinbase, BurnedBlock, Circulating, Circulating), '*_rewards')


class Major(_Node):
    unknown: Antpool = _at(Antpool, '*')
    luxor: Antpool = _at(Antpool, 'luxor')
    btccom: Antpool = _at(Antpool, 'btccom')
    btctop: Antpool = _at(Antpool, 'btctop')
    btcguild: Antpool = _at(Antpool, 'btcguild')
    eligius: Antpool = _at(Antpool, 'eligius')
    f2pool: Antpool = _at(Antpool, 'f2pool')
    braiinspool: Antpool = _at(Antpool, 'braiinspool')
    antpool: Antpool = _at(Antpool, 'antpool')
    btcc: Antpool = _at(Antpool, 'btcc')
    bwpool: Antpool = _at(Antpool, 'bwpool')
    bitfury: Antpool = _at(Antpool, 'bitfury')
    viabtc: Antpool = _at(Antpool, 'viabtc')
    poolin: Antpool = _at(Antpool, 'poolin')
    spiderpool: Antpool = _at(Antpool, 'spiderpool')
    binancepool: Antpool = _at(Antpool, 'binancepool')
    foundryusa: Antpool = _at(Antpool, 'foundryusa')
    sbicrypto: Antpool = _at(Antpool, 'sbicrypto')
    marapool: Antpool = _at(Antpool, 'marapool')
    secpool: Antpool = _at(Antpool, 'secpool')
    ocean: Antpool = _at(Antpool, 'ocean')
    whitepool: Antpool = _at(Antpool, 'whitepool')


class Pools(_Node):
    pool: SeriesPattern21[PoolSlug] = _at(SeriesPattern21, '*')
    major: Major = _at(Major, 'unknown')
    minor: Minor = _at(Minor, 'blockfills')


class Matured(_Node, Generic[A]):
    under_1h: A = _at(0, 'utxos_under_1h_*')
    _1h_to_1d: A = _at(0, 'utxos_1h_to_1d_*')
    _1d_to_1w: A = _at(0, 'utxos_1d_to_1w_*')
    _1w_to_1m: A = _at(0, 'utxos_1w_to_1m_*')
    _1m_to_2m: A = _at(0, 'utxos_1m_to_2m_*')
    _2m_to_3m: A = _at(0, 'utxos_2m_to_3m_*')
    _3m_to_4m: A = _at(0, 'utxos_3m_to_4m_*')
    _4m_to_5m: A = _at(0, 'utxos_4m_to_5m_*')
    _5m_to_6m: A = _at(0, 'utxos_5m_to_6m_*')
    _6m_to_9m: A = _at(0, 'utxos_6m_to_9m_*')
    _9m_to_1y: A = _at(0, 'utxos_9m_to_1y_*')
    _1y_to_18m: A = _at(0, 'utxos_1y_to_18m_*')
    _18m_to_2y: A = _at(0, 'utxos_18m_to_2y_*')
    _2y_to_3y: A = _at(0, 'utxos_2y_to_3y_*')
    _3y_to_4y: A = _at(0, 'utxos_3y_to_4y_*')
    _4y_to_5y: A = _at(0, 'utxos_4y_to_5y_*')
    _5y_to_6y: A = _at(0, 'utxos_5y_to_6y_*')
    _6y_to_7y: A = _at(0, 'utxos_6y_to_7y_*')
    _7y_to_8y: A = _at(0, 'utxos_7y_to_8y_*')
    _8y_to_10y: A = _at(0, 'utxos_8y_to_10y_*')
    _10y_to_12y: A = _at(0, 'utxos_10y_to_12y_*')
    _12y_to_15y: A = _at(0, 'utxos_12y_to_15y_*')
    over_15y: A = _at(0, 'utxos_over_15y_*')


class CoindaysDestroyed(_Node, Generic[A]):
    age: Matured[A] = _at((Matured, 0), 'old_*')
    epoch: CoindaysDestroyedEpoch[A] = _at((CoindaysDestroyedEpoch, 0), '*')
    class_: Class[A] = _at((Class, 0), '*')


class CohortsUnrealized(_Node):
    profit: CoindaysDestroyed[CoinflowCap[Optional[Cents]]] = _at((CoindaysDestroyed, CoinflowCap), '*_profit')
    loss: CoindaysDestroyed[CoinflowCap[Optional[Cents]]] = _at((CoindaysDestroyed, CoinflowCap), '*_loss')
    net_pnl: CoindaysDestroyed[CoinflowCap[CentsSigned]] = _at((CoindaysDestroyed, CoinflowCap), 'net_*_pnl')


class CoinflowAgeRangeSupply(_Node):
    mobile: Matured[Circulating[Sats, Optional[Cents]]] = _at((Matured, Circulating), '*_mobile_supply')
    immobile: Matured[Circulating[Sats, Optional[Cents]]] = _at((Matured, Circulating), '*_immobile_supply')


class CoinflowAgeRange(_Node):
    spending_rate: SpendingRate[Optional[PerDay]] = _at(SpendingRate, '*_spending_rate')
    spending_exposure: SpendingExposure = _at(SpendingExposure, '*')
    supply: CoinflowAgeRangeSupply = _at(CoinflowAgeRangeSupply, '*')


class Coinflow(_Node):
    age_range: CoinflowAgeRange = _at(CoinflowAgeRange, 'old')
    urpd: CoinflowUrpd[CoinflowUrpdLth] = _at((CoinflowUrpd, CoinflowUrpdLth), '*')
    supply: CoinflowSupply = _at(CoinflowSupply, '')
    cap: CoinflowCap[Optional[Cents]] = _at(CoinflowCap, '*_cap')
    price: CapitalizedPrice = _at(CapitalizedPrice, '*_price')
    capitalized_price: CapitalizedPrice = _at(CapitalizedPrice, '*_capitalized_price')
    sth: CoinflowLth = _at(CoinflowLth, 'sth')
    lth: CoinflowLth = _at(CoinflowLth, 'lth')
    under_4m_price: CapitalizedPrice = _at(CapitalizedPrice, 'under_4m_*_price')
    under_4m_capitalized_price: CapitalizedPrice = _at(CapitalizedPrice, 'under_4m_*_capitalized_price')
    under_6m_price: CapitalizedPrice = _at(CapitalizedPrice, 'under_6m_*_price')
    under_6m_capitalized_price: CapitalizedPrice = _at(CapitalizedPrice, 'under_6m_*_capitalized_price')
    over_4m_price: CapitalizedPrice = _at(CapitalizedPrice, 'over_4m_*_price')
    over_4m_capitalized_price: CapitalizedPrice = _at(CapitalizedPrice, 'over_4m_*_capitalized_price')
    over_6m_price: CapitalizedPrice = _at(CapitalizedPrice, 'over_6m_*_price')
    over_6m_capitalized_price: CapitalizedPrice = _at(CapitalizedPrice, 'over_6m_*_capitalized_price')


class CointimeAgeRangeSupply(_Node):
    awake: Matured[Circulating[Sats, Optional[Cents]]] = _at((Matured, Circulating), '*_awake_supply')
    dormant: Matured[Circulating[Sats, Optional[Cents]]] = _at((Matured, Circulating), '*_dormant_supply')


class CointimeAgeRange(_Node):
    coindays_consumed: Matured[NewAll[Optional[CoinDays], Optional[CoinDays]]] = _at((Matured, NewAll), '*_coindays_consumed')
    coindays_stored: Matured[NewAll[Optional[CoinDays], Optional[CoinDays]]] = _at((Matured, NewAll), '*_coindays_stored')
    activity: AgeRangeActivity = _at(AgeRangeActivity, '*')
    supply: CointimeAgeRangeSupply = _at(CointimeAgeRangeSupply, '*')
    coindays_created: Matured[NewAll[Optional[CoinDays], Optional[CoinDays]]] = _at((Matured, NewAll), '*_coindays_created')


class Cointime(_Node):
    activity: CointimeActivity = _at(CointimeActivity, 'coinblocks')
    age_range: CointimeAgeRange = _at(CointimeAgeRange, 'old')
    urpd: CoinflowUrpd[CointimeUrpdLth] = _at((CoinflowUrpd, CointimeUrpdLth), 'cointime')
    awake: Awake = _at(Awake, '*')
    dormant: Dormant = _at(Dormant, 'dormant_supply')
    sth: CointimeLth = _at(CointimeLth, 'sth')
    lth: CointimeLth = _at(CointimeLth, 'lth')
    under_4m_awake_price: CapitalizedPrice = _at(CapitalizedPrice, 'under_4m_*_price')
    under_4m_awake_capitalized_price: CapitalizedPrice = _at(CapitalizedPrice, 'under_4m_*_capitalized_price')
    under_6m_awake_price: CapitalizedPrice = _at(CapitalizedPrice, 'under_6m_*_price')
    under_6m_awake_capitalized_price: CapitalizedPrice = _at(CapitalizedPrice, 'under_6m_*_capitalized_price')
    over_4m_awake_price: CapitalizedPrice = _at(CapitalizedPrice, 'over_4m_*_price')
    over_4m_awake_capitalized_price: CapitalizedPrice = _at(CapitalizedPrice, 'over_4m_*_capitalized_price')
    over_6m_awake_price: CapitalizedPrice = _at(CapitalizedPrice, 'over_6m_*_price')
    over_6m_awake_capitalized_price: CapitalizedPrice = _at(CapitalizedPrice, 'over_6m_*_capitalized_price')
    supply: CointimeSupply = _at(CointimeSupply, 'supply')
    value: CointimeValue = _at(CointimeValue, 'cointime_value')
    cap: CointimeCap = _at(CointimeCap, 'cap')
    prices: CointimePrices = _at(CointimePrices, 'price')
    adjusted: Adjusted = _at(Adjusted, 'cointime_adj')
    reserve_risk: ReserveRisk = _at(ReserveRisk, 'reserve_risk')


class Rewards(_Node):
    coinbase: Coinbase[BurnedBlock, Circulating[Sats, Optional[Cents]], Circulating[Optional[SatsFract], Optional[CentsFract]]] = _at((Coinbase, BurnedBlock, Circulating, Circulating), '*')
    subsidy: Subsidy = _at(Subsidy, 'subsidy')
    fees: RewardsFees = _at(RewardsFees, 'fees')
    output_volume: SeriesPattern21[Sats] = _at(SeriesPattern21, 'output_volume')
    unclaimed: Burned = _at(Burned, 'unclaimed_rewards')


class Mining(_Node):
    rewards: Rewards = _at(Rewards, '*')
    hashrate: MiningHashrate = _at(MiningHashrate, 'hash')


class RealizedCap(_Node, Generic[A]):
    _0sats: A = _at(0, 'addrs_0sats_*')
    _1sat_to_10sats: A = _at(0, 'addrs_1sat_to_10sats_*')
    _10sats_to_100sats: A = _at(0, 'addrs_10sats_to_100sats_*')
    _100sats_to_1k_sats: A = _at(0, 'addrs_100sats_to_1k_sats_*')
    _1k_sats_to_10k_sats: A = _at(0, 'addrs_1k_sats_to_10k_sats_*')
    _10k_sats_to_100k_sats: A = _at(0, 'addrs_10k_sats_to_100k_sats_*')
    _100k_sats_to_1m_sats: A = _at(0, 'addrs_100k_sats_to_1m_sats_*')
    _1m_sats_to_10m_sats: A = _at(0, 'addrs_1m_sats_to_10m_sats_*')
    _10m_sats_to_1btc: A = _at(0, 'addrs_10m_sats_to_1btc_*')
    _1btc_to_10btc: A = _at(0, 'addrs_1btc_to_10btc_*')
    _10btc_to_100btc: A = _at(0, 'addrs_10btc_to_100btc_*')
    _100btc_to_1k_btc: A = _at(0, 'addrs_100btc_to_1k_btc_*')
    _1k_btc_to_10k_btc: A = _at(0, 'addrs_1k_btc_to_10k_btc_*')
    _10k_btc_to_100k_btc: A = _at(0, 'addrs_10k_btc_to_100k_btc_*')
    over_100k_btc: A = _at(0, 'addrs_over_100k_btc_*')


class Funded(_Node):
    all: SeriesPattern2[Count] = _at(SeriesPattern2, '*')
    p2pk65: SeriesPattern2[Count] = _at(SeriesPattern2, 'p2pk65_*')
    p2pk33: SeriesPattern2[Count] = _at(SeriesPattern2, 'p2pk33_*')
    p2pkh: SeriesPattern2[Count] = _at(SeriesPattern2, 'p2pkh_*')
    p2sh: SeriesPattern2[Count] = _at(SeriesPattern2, 'p2sh_*')
    p2wpkh: SeriesPattern2[Count] = _at(SeriesPattern2, 'p2wpkh_*')
    p2wsh: SeriesPattern2[Count] = _at(SeriesPattern2, 'p2wsh_*')
    p2tr: SeriesPattern2[Count] = _at(SeriesPattern2, 'p2tr_*')
    p2a: SeriesPattern2[Count] = _at(SeriesPattern2, 'p2a_*')
    balance: RealizedCap[UtxoCount0sats] = _at((RealizedCap, UtxoCount0sats), '*')


class ByBalance(_Node):
    supply: RealizedCap[Supply0sats] = _at((RealizedCap, Supply0sats), 'supply')
    utxo_count: RealizedCap[UtxoCount0sats] = _at((RealizedCap, UtxoCount0sats), 'utxo_count')
    transfer_volume: RealizedCap[Coinbase[BurnedBlock, Circulating[Sats, Optional[Cents]], Circulating[Optional[SatsFract], Optional[CentsFract]]]] = _at((RealizedCap, (Coinbase, BurnedBlock, Circulating, Circulating)), 'transfer_volume')
    realized_cap: RealizedCap[CoinflowCap[Optional[Cents]]] = _at((RealizedCap, CoinflowCap), '*_cap')
    realized_profit: RealizedCap[RealizedLoss0sats] = _at((RealizedCap, RealizedLoss0sats), '*_profit')
    realized_loss: RealizedCap[RealizedLoss0sats] = _at((RealizedCap, RealizedLoss0sats), '*_loss')


class Addrs(_Node):
    raw: AddrsRaw = _at(AddrsRaw, 'p2pk65')
    state: State = _at(State, '*')
    by_balance: ByBalance = _at(ByBalance, 'realized')
    funded: Funded = _at(Funded, '*_count')
    empty: AddrsEmpty = _at(AddrsEmpty, 'empty_*_count')
    activity: AddrsActivity = _at(AddrsActivity, 'addrs')
    total: AddrsEmpty = _at(AddrsEmpty, 'total_*_count')
    new: AvgBalance[NewAll[Count, Optional[CountFract]]] = _at((AvgBalance, NewAll), 'new_*_count')
    reused: Respent = _at(Respent, 'reused_*')
    respent: Respent = _at(Respent, 'respent_*')
    exposed: Exposed = _at(Exposed, 'exposed_*')
    delta: AvgBalance[DeltaAll[PerSec[CountSigned]]] = _at((AvgBalance, (DeltaAll, PerSec)), '*_count')
    avg_balance: AvgBalance[Circulating[Sats, Optional[Cents]]] = _at((AvgBalance, Circulating), 'avg_*_amount')


class InputShare(_Node, Generic[A]):
    p2pk65: A = _at(0, 'p2pk65_*')
    p2pk33: A = _at(0, 'p2pk33_*')
    p2pkh: A = _at(0, 'p2pkh_*')
    p2ms: A = _at(0, 'p2ms_*')
    p2sh: A = _at(0, 'p2sh_*')
    p2wpkh: A = _at(0, 'p2wpkh_*')
    p2wsh: A = _at(0, 'p2wsh_*')
    p2tr: A = _at(0, 'p2tr_*')
    p2a: A = _at(0, 'p2a_*')
    unknown: A = _at(0, 'unknown_outputs_*')
    empty: A = _at(0, 'empty_outputs_*')


class RealizedPrice(_Node):
    utxo_amount: UtxoAmount[Spot[Optional[SatsFract]]] = _at((UtxoAmount, Spot), '*')
    type_: InputShare[Spot[Optional[SatsFract]]] = _at((InputShare, Spot), '*')


class TransferVolume(_Node):
    age: Matured[Coinbase[BurnedBlock, Circulating[Sats, Optional[Cents]], Circulating[Optional[SatsFract], Optional[CentsFract]]]] = _at((Matured, (Coinbase, BurnedBlock, Circulating, Circulating)), 'old_*')
    epoch: CoindaysDestroyedEpoch[Coinbase[BurnedBlock, Circulating[Sats, Optional[Cents]], Circulating[Optional[SatsFract], Optional[CentsFract]]]] = _at((CoindaysDestroyedEpoch, (Coinbase, BurnedBlock, Circulating, Circulating)), '*')
    class_: Class[Coinbase[BurnedBlock, Circulating[Sats, Optional[Cents]], Circulating[Optional[SatsFract], Optional[CentsFract]]]] = _at((Class, (Coinbase, BurnedBlock, Circulating, Circulating)), '*')
    in_profit: CoindaysDestroyed[Coinbase[BurnedBlock, Circulating[Sats, Optional[Cents]], Circulating[Optional[SatsFract], Optional[CentsFract]]]] = _at((CoindaysDestroyed, (Coinbase, BurnedBlock, Circulating, Circulating)), '*_in_profit')
    in_loss: CoindaysDestroyed[Coinbase[BurnedBlock, Circulating[Sats, Optional[Cents]], Circulating[Optional[SatsFract], Optional[CentsFract]]]] = _at((CoindaysDestroyed, (Coinbase, BurnedBlock, Circulating, Circulating)), '*_in_loss')
    utxo_amount: UtxoAmount[Coinbase[BurnedBlock, Circulating[Sats, Optional[Cents]], Circulating[Optional[SatsFract], Optional[CentsFract]]]] = _at((UtxoAmount, (Coinbase, BurnedBlock, Circulating, Circulating)), '*')
    type_: InputShare[Coinbase[BurnedBlock, Circulating[Sats, Optional[Cents]], Circulating[Optional[SatsFract], Optional[CentsFract]]]] = _at((InputShare, (Coinbase, BurnedBlock, Circulating, Circulating)), '*')


class CohortsActivity(_Node):
    transfer_volume: TransferVolume = _at(TransferVolume, '*')
    coindays_destroyed: CoindaysDestroyed[NewAll[Optional[CoinDays], Optional[CoinDays]]] = _at((CoindaysDestroyed, NewAll), 'coindays_destroyed')


class AvgAmount(_Node):
    all: Circulating[Sats, Optional[Cents]] = _at(Circulating, '*')
    by_type: InputShare[Circulating[Sats, Optional[Cents]]] = _at((InputShare, Circulating), '*')


class SpentCount(_Node, Generic[A]):
    age: Matured[A] = _at((Matured, 0), 'old_*')
    epoch: CoindaysDestroyedEpoch[A] = _at((CoindaysDestroyedEpoch, 0), '*')
    class_: Class[A] = _at((Class, 0), '*')
    utxo_amount: UtxoAmount[A] = _at((UtxoAmount, 0), '*')
    type_: InputShare[A] = _at((InputShare, 0), '*')


class CohortsRealized(_Node):
    cap: SpentCount[CoinflowCap[Optional[Cents]]] = _at((SpentCount, CoinflowCap), '*_cap')
    profit: SpentCount[RealizedLoss0sats] = _at((SpentCount, RealizedLoss0sats), '*_profit')
    loss: SpentCount[RealizedLoss0sats] = _at((SpentCount, RealizedLoss0sats), '*_loss')
    net_pnl: CoindaysDestroyed[Age10yTo12y] = _at((CoindaysDestroyed, Age10yTo12y), 'net_*_pnl')
    value_destroyed: CoindaysDestroyed[Coinbase[RealizedLoss0satsBlock[Optional[Cents]], CoinflowCap[Optional[Cents]], CoinflowCap[Optional[CentsFract]]]] = _at((CoindaysDestroyed, (Coinbase, RealizedLoss0satsBlock, CoinflowCap, CoinflowCap)), 'value_destroyed')
    price: RealizedPrice = _at(RealizedPrice, '*_price')


class CohortsOutputs(_Node):
    unspent_count: SpentCount[UtxoCount0sats] = _at((SpentCount, UtxoCount0sats), '*_count')
    spent_count: SpentCount[NewAll[Count, Optional[CountFract]]] = _at((SpentCount, NewAll), 'spent_*_count')
    avg_amount: AvgAmount = _at(AvgAmount, 'avg_*_amount')


class CohortsSupply(_Node):
    total: SpentCount[Circulating[Sats, Optional[Cents]]] = _at((SpentCount, Circulating), '*')
    matured: Matured[Coinbase[BurnedBlock, Circulating[Sats, Optional[Cents]], Circulating[Optional[SatsFract], Optional[CentsFract]]]] = _at((Matured, (Coinbase, BurnedBlock, Circulating, Circulating)), 'old_matured_*')
    in_profit: CoindaysDestroyed[Circulating[Sats, Optional[Cents]]] = _at((CoindaysDestroyed, Circulating), '*_in_profit')
    in_loss: CoindaysDestroyed[Circulating[Sats, Optional[Cents]]] = _at((CoindaysDestroyed, Circulating), '*_in_loss')
    delta: SpentCount[DeltaAll[Sd24h[Absolute1m]]] = _at((SpentCount, (DeltaAll, (Sd24h, Absolute1m))), '*_delta')
    dominance: SpentCount[Gini[Optional[PartsPerMillion32]]] = _at((SpentCount, Gini), '*_dominance')


class Cohorts(_Node):
    supply: CohortsSupply = _at(CohortsSupply, '*')
    outputs: CohortsOutputs = _at(CohortsOutputs, 'utxo')
    activity: CohortsActivity = _at(CohortsActivity, 'transfer_volume')
    realized: CohortsRealized = _at(CohortsRealized, 'realized')
    unrealized: CohortsUnrealized = _at(CohortsUnrealized, 'unrealized')
    urpd: CohortsUrpd = _at(CohortsUrpd, 'utxos_urpd')


class InputsByType(_Node):
    input_count: InputCount = _at(InputCount, '*')
    input_share: InputShare[FeeShare] = _at((InputShare, FeeShare), 'prevout_share')
    tx_count: InputsByTypeTxCount = _at(InputsByTypeTxCount, 'tx_*')
    tx_share: InputsByTypeTxShare = _at(InputsByTypeTxShare, 'tx_share_with')


class Inputs(_Node):
    raw: InputsRaw = _at(InputsRaw, 'index')
    value: SeriesPattern23[Sats] = _at(SeriesPattern23, 'value')
    count: InputsCount[Count16] = _at(InputsCount, 'input_*')
    per_sec: PerSec[Optional[PerSecond]] = _at(PerSec, 'inputs_per_sec')
    by_type: InputsByType = _at(InputsByType, '*')


class Volume(_Node):
    transfer_volume: Coinbase[BurnedBlock, Circulating[Sats, Optional[Cents]], Circulating[Optional[SatsFract], Optional[CentsFract]]] = _at((Coinbase, BurnedBlock, Circulating, Circulating), '*')
    tx_per_sec: PerSec[Optional[PerSecond]] = _at(PerSec, 'tx_per_sec')


class Inscription(_Node):
    count: NewAll[Count, Optional[CountFract]] = _at(NewAll, 'tx_count_*')
    fees: NewAll[Sats, Optional[SatsFract]] = _at(NewAll, '*_fees')
    fee_share: Gini[Optional[PartsPerMillion32]] = _at(Gini, '*_fee_share')


class Transactions(_Node):
    raw: TransactionsRaw = _at(TransactionsRaw, '*')
    features: Features = _at(Features, 'has')
    count: OutputsCount[Vbytes[Count, Optional[CountFract], Count16]] = _at((OutputsCount, Vbytes), '*_count')
    size: TransactionsSize = _at(TransactionsSize, '*')
    fees: TransactionsFees = _at(TransactionsFees, 'fee')
    inscription: Inscription = _at(Inscription, 'inscription')
    patterns: Patterns = _at(Patterns, 'is')
    policy: Policy = _at(Policy, 'nonstandard')
    sigops: OutputsCount[NewAll[SigOps64, Optional[SigOpsFract]]] = _at((OutputsCount, NewAll), 'total_sigop_cost')
    versions: Versions = _at(Versions, '*')
    volume: Volume = _at(Volume, 'transfer_volume_bis')


class BlocksDifficulty(_Node):
    value: SeriesPattern2[Optional[Difficulty]] = _at(SeriesPattern2, '*')
    hashrate: SeriesPattern2[Optional[Hashrate]] = _at(SeriesPattern2, '*_hashrate')
    adjustment: Gini[Optional[PartsPerMillionSigned32]] = _at(Gini, '*_adjustment')
    epoch: SeriesPattern2[Epoch] = _at(SeriesPattern2, '*_epoch')
    blocks_to_retarget: SeriesPattern2[Count] = _at(SeriesPattern2, 'blocks_to_retarget')
    days_to_retarget: SeriesPattern2[Optional[Days]] = _at(SeriesPattern2, 'days_to_retarget')


class Blocks(_Node):
    blockhash: SeriesPattern21[BlockHash] = _at(SeriesPattern21, 'blockhash')
    coinbase_tag: SeriesPattern21[CoinbaseTag] = _at(SeriesPattern21, 'coinbase_tag')
    difficulty: BlocksDifficulty = _at(BlocksDifficulty, 'difficulty')
    time: Time = _at(Time, 'timestamp')
    size: BlocksSize = _at(BlocksSize, 'size')
    weight: BlocksWeight = _at(BlocksWeight, '*_weight')
    segwit_txs: SeriesPattern21[Count16] = _at(SeriesPattern21, 'segwit_txs')
    segwit_size: SeriesPattern21[Bytes32] = _at(SeriesPattern21, 'segwit_size')
    segwit_weight: SeriesPattern21[Weight] = _at(SeriesPattern21, 'segwit_weight')
    count: BlocksCount = _at(BlocksCount, '*_count')
    lookback: BlocksLookback = _at(BlocksLookback, 'height')
    interval: Interval[Seconds, Optional[SecondsFract]] = _at(Interval, '*_interval')
    vbytes: Vbytes[VSize, Optional[VSizeFract], VSize] = _at(Vbytes, '*_vbytes')
    fullness: Fullness = _at(Fullness, '*_fullness')
    halving: BlocksHalving = _at(BlocksHalving, 'halving')


class SeriesTree(_Node):
    blocks: Blocks = _at(Blocks, 'block')
    transactions: Transactions = _at(Transactions, 'tx')
    inputs: Inputs = _at(Inputs, 'count')
    outputs: Outputs = _at(Outputs, 'output')
    addrs: Addrs = _at(Addrs, 'addr')
    scripts: Scripts = _at(Scripts, 'index')
    op_return: OpReturn = _at(OpReturn, 'op_return')
    mining: Mining = _at(Mining, 'coinbase')
    cointime: Cointime = _at(Cointime, 'awake')
    coinflow: Coinflow = _at(Coinflow, 'coinflow')
    bedrock: Bedrock = _at(Bedrock, 'bedrock')
    capital_sentiment: CapitalSentiment = _at(CapitalSentiment, 'capital_sentiment')
    rarity_meter: RarityMeter = _at(RarityMeter, 'rarity_meter')
    constants: Constants = _at(Constants, 'constant')
    mappings: Mappings = _at(Mappings, 'date')
    indicators: Indicators = _at(Indicators, 'destroyed_supply_adj')
    market: Market = _at(Market, 'price')
    pools: Pools = _at(Pools, 'pool')
    price: Price = _at(Price, 'price')
    cohorts: Cohorts = _at(Cohorts, 'supply')
    distribution_aggregated: DistributionAggregated = _at(DistributionAggregated, 'under')
    supply: Supply = _at(Supply, 'supply')
    utxo_history: UtxoHistory = _at(UtxoHistory, 'unspent_sats')


class BitviewClient(BitviewClientBase):
    """Main Bitview client with series tree and API methods."""

    VERSION = "v0.12.2"

    INDEXES = [
      "minute10",
      "minute30",
      "hour1",
      "hour4",
      "hour12",
      "day1",
      "day3",
      "week1",
      "month1",
      "month3",
      "month6",
      "year1",
      "year10",
      "halving",
      "epoch",
      "height",
      "tx_index",
      "txin_index",
      "txout_index",
      "empty_output_index",
      "op_return_index",
      "p2a_addr_index",
      "p2ms_output_index",
      "p2pk33_addr_index",
      "p2pk65_addr_index",
      "p2pkh_addr_index",
      "p2sh_addr_index",
      "p2tr_addr_index",
      "p2wpkh_addr_index",
      "p2wsh_addr_index",
      "unknown_output_index",
      "funded_addr_index",
      "empty_addr_index",
      "extended_empty_addr_index"
    ]

    POOL_ID_TO_POOL_NAME = {
      "aaopool": "AAO Pool",
      "antpool": "AntPool",
      "arkpool": "ArkPool",
      "asicminer": "ASICMiner",
      "axbt": "A-XBT",
      "batpool": "BATPOOL",
      "bcmonster": "BCMonster",
      "bcpoolio": "bcpool.io",
      "binancepool": "Binance Pool",
      "bitalo": "Bitalo",
      "bitclub": "BitClub",
      "bitcoinaffiliatenetwork": "Bitcoin Affiliate Network",
      "bitcoincom": "Bitcoin.com",
      "bitcoinindia": "Bitcoin India",
      "bitcoinindiapool": "BitcoinIndia",
      "bitcoinrussia": "BitcoinRussia",
      "bitcoinukraine": "Bitcoin-Ukraine",
      "bitfarms": "Bitfarms",
      "bitfufupool": "BitFuFuPool",
      "bitfury": "BitFury",
      "bitminter": "BitMinter",
      "bitparking": "Bitparking",
      "bitsolo": "Bitsolo",
      "bixin": "Bixin",
      "blockfills": "BlockFills",
      "braiinspool": "Braiins Pool",
      "braiinssolo": "Braiins Solo",
      "bravomining": "Bravo Mining",
      "btcc": "BTCC",
      "btccom": "BTC.com",
      "btcdig": "BTCDig",
      "btcguild": "BTC Guild",
      "btclab": "BTCLab",
      "btcmp": "BTCMP",
      "btcnuggets": "BTC Nuggets",
      "btcpoolparty": "BTC Pool Party",
      "btcserv": "BTCServ",
      "btctop": "BTC.TOP",
      "btpool": "BTPOOL",
      "bwpool": "BWPool",
      "bytepool": "BytePool",
      "canoe": "CANOE",
      "canoepool": "CanoePool",
      "carbonnegative": "Carbon Negative",
      "ckpool": "CKPool",
      "cloudhashing": "CloudHashing",
      "coinlab": "CoinLab",
      "cointerra": "Cointerra",
      "connectbtc": "ConnectBTC",
      "dcex": "DCEX",
      "dcexploration": "DCExploration",
      "digitalbtc": "digitalBTC",
      "digitalxmintsy": "digitalX Mintsy",
      "dmnd": "DMND",
      "dpool": "DPOOL",
      "eclipsemc": "EclipseMC",
      "eightbaochi": "8baochi",
      "ekanembtc": "EkanemBTC",
      "eligius": "Eligius",
      "emcdpool": "EMCDPool",
      "entrustcharitypool": "Entrust Charity Pool",
      "eobot": "Eobot",
      "est3lar": "Est3lar",
      "exxbw": "EXX&BW",
      "f2pool": "F2Pool",
      "fiftyeightcoin": "58COIN",
      "foundryusa": "Foundry USA",
      "futurebitapollosolo": "FutureBit Apollo Solo",
      "gbminers": "GBMiners",
      "gdpool": "GDPool",
      "ghashio": "GHash.IO",
      "givemecoins": "Give Me Coins",
      "gogreenlight": "GoGreenLight",
      "haominer": "haominer",
      "haozhuzhu": "HAOZHUZHU",
      "hashbx": "HashBX",
      "hashpool": "HASHPOOL",
      "helix": "Helix",
      "hhtt": "HHTT",
      "hotpool": "HotPool",
      "hummerpool": "Hummerpool",
      "huobipool": "Huobi.pool",
      "innopolistech": "Innopolis Tech",
      "kanopool": "KanoPool",
      "kncminer": "KnCMiner",
      "kucoinpool": "KuCoinPool",
      "lubiancom": "Lubian.com",
      "luxor": "Luxor",
      "marapool": "MARA Pool",
      "maxbtc": "MaxBTC",
      "maxipool": "MaxiPool",
      "megabigpower": "MegaBigPower",
      "minerium": "Minerium",
      "miningcity": "MiningCity",
      "miningdutch": "Mining-Dutch",
      "miningkings": "MiningKings",
      "miningsquared": "Mining Squared",
      "mmpool": "mmpool",
      "mtred": "Mt Red",
      "multicoinco": "MultiCoin.co",
      "multipool": "Multipool",
      "mybtccoinpool": "myBTCcoin Pool",
      "neopool": "Neopool",
      "nexious": "Nexious",
      "nicehash": "NiceHash",
      "nmcbit": "NMCbit",
      "noderunners": "Noderunners",
      "novablock": "NovaBlock",
      "ocean": "OCEAN",
      "okexpool": "OKExPool",
      "okkong": "OKKONG",
      "okminer": "OKMINER",
      "okpooltop": "okpool.top",
      "onehash": "1Hash",
      "onem1x": "1M1X",
      "onethash": "1THash",
      "ozcoin": "OzCoin",
      "parasite": "Parasite",
      "patels": "Patels",
      "pegapool": "PEGA Pool",
      "phashio": "PHash.IO",
      "phoenix": "Phoenix",
      "polmine": "Polmine",
      "pool175btc": "175btc",
      "pool50btc": "50BTC",
      "poolin": "Poolin",
      "portlandhodl": "Portland.HODL",
      "publicpool": "Public Pool",
      "purebtccom": "PureBTC.COM",
      "rawpool": "Rawpool",
      "redrockpool": "RedRock Pool",
      "rigpool": "RigPool",
      "sbicrypto": "SBI Crypto",
      "secpool": "SECPOOL",
      "secretsuperstar": "SecretSuperstar",
      "sevenpool": "7pool",
      "shawnp0wers": "shawnp0wers",
      "sigmapoolcom": "Sigmapool.com",
      "simplecoinus": "simplecoin.us",
      "solock": "Solo CK",
      "solopool": "SoloPool.com",
      "spiderpool": "SpiderPool",
      "stminingcorp": "ST Mining Corp",
      "tangpool": "Tangpool",
      "tatmaspool": "TATMAS Pool",
      "tbdice": "TBDice",
      "telco214": "Telco 214",
      "terrapool": "Terra Pool",
      "tiger": "tiger",
      "tigerpoolnet": "tigerpool.net",
      "titan": "Titan",
      "transactioncoinmining": "transactioncoinmining",
      "trickysbtcpool": "Tricky's BTC Pool",
      "triplemining": "TripleMining",
      "twentyoneinc": "21 Inc.",
      "ultimuspool": "ULTIMUSPOOL",
      "unknown": "Unknown",
      "unomp": "UNOMP",
      "viabtc": "ViaBTC",
      "waterhole": "Waterhole",
      "wayicn": "WAYI.CN",
      "whitepool": "WhitePool",
      "wiz": "wiz",
      "wk057": "wk057",
      "yourbtcnet": "Yourbtc.net",
      "zulupool": "Zulupool"
    }

    TERM_NAMES = {
      "short": {
        "id": "sth",
        "short": "STH",
        "long": "Short Term Holders"
      },
      "long": {
        "id": "lth",
        "short": "LTH",
        "long": "Long Term Holders"
      }
    }

    EPOCH_NAMES = {
      "_0": {
        "id": "epoch_0",
        "short": "0",
        "long": "Epoch 0"
      },
      "_1": {
        "id": "epoch_1",
        "short": "1",
        "long": "Epoch 1"
      },
      "_2": {
        "id": "epoch_2",
        "short": "2",
        "long": "Epoch 2"
      },
      "_3": {
        "id": "epoch_3",
        "short": "3",
        "long": "Epoch 3"
      },
      "_4": {
        "id": "epoch_4",
        "short": "4",
        "long": "Epoch 4"
      }
    }

    CLASS_NAMES = {
      "_2009": {
        "id": "class_2009",
        "short": "2009",
        "long": "Class 2009"
      },
      "_2010": {
        "id": "class_2010",
        "short": "2010",
        "long": "Class 2010"
      },
      "_2011": {
        "id": "class_2011",
        "short": "2011",
        "long": "Class 2011"
      },
      "_2012": {
        "id": "class_2012",
        "short": "2012",
        "long": "Class 2012"
      },
      "_2013": {
        "id": "class_2013",
        "short": "2013",
        "long": "Class 2013"
      },
      "_2014": {
        "id": "class_2014",
        "short": "2014",
        "long": "Class 2014"
      },
      "_2015": {
        "id": "class_2015",
        "short": "2015",
        "long": "Class 2015"
      },
      "_2016": {
        "id": "class_2016",
        "short": "2016",
        "long": "Class 2016"
      },
      "_2017": {
        "id": "class_2017",
        "short": "2017",
        "long": "Class 2017"
      },
      "_2018": {
        "id": "class_2018",
        "short": "2018",
        "long": "Class 2018"
      },
      "_2019": {
        "id": "class_2019",
        "short": "2019",
        "long": "Class 2019"
      },
      "_2020": {
        "id": "class_2020",
        "short": "2020",
        "long": "Class 2020"
      },
      "_2021": {
        "id": "class_2021",
        "short": "2021",
        "long": "Class 2021"
      },
      "_2022": {
        "id": "class_2022",
        "short": "2022",
        "long": "Class 2022"
      },
      "_2023": {
        "id": "class_2023",
        "short": "2023",
        "long": "Class 2023"
      },
      "_2024": {
        "id": "class_2024",
        "short": "2024",
        "long": "Class 2024"
      },
      "_2025": {
        "id": "class_2025",
        "short": "2025",
        "long": "Class 2025"
      },
      "_2026": {
        "id": "class_2026",
        "short": "2026",
        "long": "Class 2026"
      }
    }

    ENTRY_NAMES = {
      "discount": {
        "id": "veteran",
        "short": "Veteran",
        "long": "Veteran Coins"
      },
      "premium": {
        "id": "rookie",
        "short": "Rookie",
        "long": "Rookie Coins"
      }
    }

    SPENDABLE_TYPE_NAMES = {
      "p2pk65": {
        "id": "p2pk65",
        "short": "P2PK65",
        "long": "Pay to Public Key (65 bytes)"
      },
      "p2pk33": {
        "id": "p2pk33",
        "short": "P2PK33",
        "long": "Pay to Public Key (33 bytes)"
      },
      "p2pkh": {
        "id": "p2pkh",
        "short": "P2PKH",
        "long": "Pay to Public Key Hash"
      },
      "p2ms": {
        "id": "p2ms",
        "short": "P2MS",
        "long": "Pay to Multisig"
      },
      "p2sh": {
        "id": "p2sh",
        "short": "P2SH",
        "long": "Pay to Script Hash"
      },
      "p2wpkh": {
        "id": "p2wpkh",
        "short": "P2WPKH",
        "long": "Pay to Witness Public Key Hash"
      },
      "p2wsh": {
        "id": "p2wsh",
        "short": "P2WSH",
        "long": "Pay to Witness Script Hash"
      },
      "p2tr": {
        "id": "p2tr",
        "short": "P2TR",
        "long": "Pay to Taproot"
      },
      "p2a": {
        "id": "p2a",
        "short": "P2A",
        "long": "Pay to Anchor"
      },
      "unknown": {
        "id": "unknown_outputs",
        "short": "Unknown",
        "long": "Unknown Output Type"
      },
      "empty": {
        "id": "empty_outputs",
        "short": "Empty",
        "long": "Empty Output"
      }
    }

    AGE_RANGE_NAMES = {
      "under_1h": {
        "id": "under_1h_old",
        "short": "<1h",
        "long": "Under 1 Hour Old"
      },
      "_1h_to_1d": {
        "id": "1h_to_1d_old",
        "short": "1h-1d",
        "long": "1 Hour to 1 Day Old"
      },
      "_1d_to_1w": {
        "id": "1d_to_1w_old",
        "short": "1d-1w",
        "long": "1 Day to 1 Week Old"
      },
      "_1w_to_1m": {
        "id": "1w_to_1m_old",
        "short": "1w-1m",
        "long": "1 Week to 1 Month Old"
      },
      "_1m_to_2m": {
        "id": "1m_to_2m_old",
        "short": "1m-2m",
        "long": "1 to 2 Months Old"
      },
      "_2m_to_3m": {
        "id": "2m_to_3m_old",
        "short": "2m-3m",
        "long": "2 to 3 Months Old"
      },
      "_3m_to_4m": {
        "id": "3m_to_4m_old",
        "short": "3m-4m",
        "long": "3 to 4 Months Old"
      },
      "_4m_to_5m": {
        "id": "4m_to_5m_old",
        "short": "4m-5m",
        "long": "4 to 5 Months Old"
      },
      "_5m_to_6m": {
        "id": "5m_to_6m_old",
        "short": "5m-6m",
        "long": "5 to 6 Months Old"
      },
      "_6m_to_9m": {
        "id": "6m_to_9m_old",
        "short": "6m-9m",
        "long": "6 to 9 Months Old"
      },
      "_9m_to_1y": {
        "id": "9m_to_1y_old",
        "short": "9m-1y",
        "long": "9 Months to 1 Year Old"
      },
      "_1y_to_18m": {
        "id": "1y_to_18m_old",
        "short": "1y-18m",
        "long": "1 Year to 18 Months Old"
      },
      "_18m_to_2y": {
        "id": "18m_to_2y_old",
        "short": "18m-2y",
        "long": "18 Months to 2 Years Old"
      },
      "_2y_to_3y": {
        "id": "2y_to_3y_old",
        "short": "2y-3y",
        "long": "2 to 3 Years Old"
      },
      "_3y_to_4y": {
        "id": "3y_to_4y_old",
        "short": "3y-4y",
        "long": "3 to 4 Years Old"
      },
      "_4y_to_5y": {
        "id": "4y_to_5y_old",
        "short": "4y-5y",
        "long": "4 to 5 Years Old"
      },
      "_5y_to_6y": {
        "id": "5y_to_6y_old",
        "short": "5y-6y",
        "long": "5 to 6 Years Old"
      },
      "_6y_to_7y": {
        "id": "6y_to_7y_old",
        "short": "6y-7y",
        "long": "6 to 7 Years Old"
      },
      "_7y_to_8y": {
        "id": "7y_to_8y_old",
        "short": "7y-8y",
        "long": "7 to 8 Years Old"
      },
      "_8y_to_10y": {
        "id": "8y_to_10y_old",
        "short": "8y-10y",
        "long": "8 to 10 Years Old"
      },
      "_10y_to_12y": {
        "id": "10y_to_12y_old",
        "short": "10y-12y",
        "long": "10 to 12 Years Old"
      },
      "_12y_to_15y": {
        "id": "12y_to_15y_old",
        "short": "12y-15y",
        "long": "12 to 15 Years Old"
      },
      "over_15y": {
        "id": "over_15y_old",
        "short": "15y+",
        "long": "15+ Years Old"
      }
    }

    AMOUNT_RANGE_NAMES = {
      "_0sats": {
        "id": "0sats",
        "short": "0 sats",
        "long": "0 Sats"
      },
      "_1sat_to_10sats": {
        "id": "1sat_to_10sats",
        "short": "1-10 sats",
        "long": "1-10 Sats"
      },
      "_10sats_to_100sats": {
        "id": "10sats_to_100sats",
        "short": "10-100 sats",
        "long": "10-100 Sats"
      },
      "_100sats_to_1k_sats": {
        "id": "100sats_to_1k_sats",
        "short": "100-1k sats",
        "long": "100-1K Sats"
      },
      "_1k_sats_to_10k_sats": {
        "id": "1k_sats_to_10k_sats",
        "short": "1k-10k sats",
        "long": "1K-10K Sats"
      },
      "_10k_sats_to_100k_sats": {
        "id": "10k_sats_to_100k_sats",
        "short": "10k-100k sats",
        "long": "10K-100K Sats"
      },
      "_100k_sats_to_1m_sats": {
        "id": "100k_sats_to_1m_sats",
        "short": "100k-1M sats",
        "long": "100K-1M Sats"
      },
      "_1m_sats_to_10m_sats": {
        "id": "1m_sats_to_10m_sats",
        "short": "1M-10M sats",
        "long": "1M-10M Sats"
      },
      "_10m_sats_to_1btc": {
        "id": "10m_sats_to_1btc",
        "short": "0.1-1 BTC",
        "long": "0.1-1 BTC"
      },
      "_1btc_to_10btc": {
        "id": "1btc_to_10btc",
        "short": "1-10 BTC",
        "long": "1-10 BTC"
      },
      "_10btc_to_100btc": {
        "id": "10btc_to_100btc",
        "short": "10-100 BTC",
        "long": "10-100 BTC"
      },
      "_100btc_to_1k_btc": {
        "id": "100btc_to_1k_btc",
        "short": "100-1k BTC",
        "long": "100-1K BTC"
      },
      "_1k_btc_to_10k_btc": {
        "id": "1k_btc_to_10k_btc",
        "short": "1k-10k BTC",
        "long": "1K-10K BTC"
      },
      "_10k_btc_to_100k_btc": {
        "id": "10k_btc_to_100k_btc",
        "short": "10k-100k BTC",
        "long": "10K-100K BTC"
      },
      "over_100k_btc": {
        "id": "over_100k_btc",
        "short": "100k+ BTC",
        "long": "100K+ BTC"
      }
    }

    PROFITABILITY_RANGE_NAMES = {
      "over_1000pct_in_profit": {
        "id": "utxos_over_1000pct_in_profit",
        "short": "+>1000%",
        "long": "Over 1000% in Profit"
      },
      "_500pct_to_1000pct_in_profit": {
        "id": "utxos_500pct_to_1000pct_in_profit",
        "short": "+500-1000%",
        "long": "500-1000% in Profit"
      },
      "_300pct_to_500pct_in_profit": {
        "id": "utxos_300pct_to_500pct_in_profit",
        "short": "+300-500%",
        "long": "300-500% in Profit"
      },
      "_200pct_to_300pct_in_profit": {
        "id": "utxos_200pct_to_300pct_in_profit",
        "short": "+200-300%",
        "long": "200-300% in Profit"
      },
      "_100pct_to_200pct_in_profit": {
        "id": "utxos_100pct_to_200pct_in_profit",
        "short": "+100-200%",
        "long": "100-200% in Profit"
      },
      "_90pct_to_100pct_in_profit": {
        "id": "utxos_90pct_to_100pct_in_profit",
        "short": "+90-100%",
        "long": "90-100% in Profit"
      },
      "_80pct_to_90pct_in_profit": {
        "id": "utxos_80pct_to_90pct_in_profit",
        "short": "+80-90%",
        "long": "80-90% in Profit"
      },
      "_70pct_to_80pct_in_profit": {
        "id": "utxos_70pct_to_80pct_in_profit",
        "short": "+70-80%",
        "long": "70-80% in Profit"
      },
      "_60pct_to_70pct_in_profit": {
        "id": "utxos_60pct_to_70pct_in_profit",
        "short": "+60-70%",
        "long": "60-70% in Profit"
      },
      "_50pct_to_60pct_in_profit": {
        "id": "utxos_50pct_to_60pct_in_profit",
        "short": "+50-60%",
        "long": "50-60% in Profit"
      },
      "_40pct_to_50pct_in_profit": {
        "id": "utxos_40pct_to_50pct_in_profit",
        "short": "+40-50%",
        "long": "40-50% in Profit"
      },
      "_30pct_to_40pct_in_profit": {
        "id": "utxos_30pct_to_40pct_in_profit",
        "short": "+30-40%",
        "long": "30-40% in Profit"
      },
      "_20pct_to_30pct_in_profit": {
        "id": "utxos_20pct_to_30pct_in_profit",
        "short": "+20-30%",
        "long": "20-30% in Profit"
      },
      "_10pct_to_20pct_in_profit": {
        "id": "utxos_10pct_to_20pct_in_profit",
        "short": "+10-20%",
        "long": "10-20% in Profit"
      },
      "_0pct_to_10pct_in_profit": {
        "id": "utxos_0pct_to_10pct_in_profit",
        "short": "+0-10%",
        "long": "0-10% in Profit"
      },
      "_0pct_to_10pct_in_loss": {
        "id": "utxos_0pct_to_10pct_in_loss",
        "short": "-0-10%",
        "long": "0-10% in Loss"
      },
      "_10pct_to_20pct_in_loss": {
        "id": "utxos_10pct_to_20pct_in_loss",
        "short": "-10-20%",
        "long": "10-20% in Loss"
      },
      "_20pct_to_30pct_in_loss": {
        "id": "utxos_20pct_to_30pct_in_loss",
        "short": "-20-30%",
        "long": "20-30% in Loss"
      },
      "_30pct_to_40pct_in_loss": {
        "id": "utxos_30pct_to_40pct_in_loss",
        "short": "-30-40%",
        "long": "30-40% in Loss"
      },
      "_40pct_to_50pct_in_loss": {
        "id": "utxos_40pct_to_50pct_in_loss",
        "short": "-40-50%",
        "long": "40-50% in Loss"
      },
      "_50pct_to_60pct_in_loss": {
        "id": "utxos_50pct_to_60pct_in_loss",
        "short": "-50-60%",
        "long": "50-60% in Loss"
      },
      "_60pct_to_70pct_in_loss": {
        "id": "utxos_60pct_to_70pct_in_loss",
        "short": "-60-70%",
        "long": "60-70% in Loss"
      },
      "_70pct_to_80pct_in_loss": {
        "id": "utxos_70pct_to_80pct_in_loss",
        "short": "-70-80%",
        "long": "70-80% in Loss"
      },
      "_80pct_to_90pct_in_loss": {
        "id": "utxos_80pct_to_90pct_in_loss",
        "short": "-80-90%",
        "long": "80-90% in Loss"
      },
      "_90pct_to_100pct_in_loss": {
        "id": "utxos_90pct_to_100pct_in_loss",
        "short": "-90-100%",
        "long": "90-100% in Loss"
      }
    }

    def __init__(self, base_url: str = 'http://localhost:3110', timeout: float = 30.0):
        super().__init__(base_url, timeout)

    @cached_property
    def series(self) -> SeriesTree:
        return SeriesTree(self, '')

    def series_endpoint(self, series: str, index: Index) -> Union[SeriesEndpoint[Any], DateSeriesEndpoint[Any]]:
        """Create a dynamic series endpoint builder for any series/index combination.

        Use this for programmatic access when the series name is determined at runtime.
        For type-safe access, use the `series` tree instead.
        """
        return _endpoint(self, series, index)

    def index_to_date(self, index: Index, i: int) -> Union[date, datetime]:
        """Convert an index value to a date/datetime for date-based indexes."""
        return _index_to_date(index, i)

    def date_to_index(self, index: Index, d: Union[date, datetime]) -> int:
        """Convert a date/datetime to an index value for date-based indexes."""
        return _date_to_index(index, d)

    @staticmethod
    def address_payload_hash_prefix(payload: Union[bytes, bytearray, memoryview], nibbles: int) -> str:
        """Compute the RapidHash v3 hash-prefix for raw address payload bytes."""
        return address_payload_hash_prefix(payload, nibbles)

    def get_address_payload_hash_prefix_matches(
        self,
        addr_type: OutputType,
        payload: Union[bytes, bytearray, memoryview],
        nibbles: int,
    ) -> AddrHashPrefixMatches:
        """Fetch address hash-prefix matches from raw payload bytes matching addr_type length."""
        _validate_address_payload_for_type(addr_type, payload)
        return self.get_address_hash_prefix_matches(addr_type, address_payload_hash_prefix(payload, nibbles))

    def get_health(self) -> Health:
        """Health check.

        Local health and query-readiness check. Returns server identity, uptime, and a coherent local sync snapshot without a bitcoind round-trip. Reads the published prefix during processing; an empty index waits until the request deadline, then returns 504. Responses are not cached. For chain-tip catch-up, request `GET /api/server/sync`.

        Endpoint: `GET /health`"""
        return self.get_json('/health')

    def get_version(self) -> str:
        """API version.

        Returns the current version of the API server

        Endpoint: `GET /version`"""
        return self.get_json('/version')

    def get_sync_status(self) -> SyncStatus:
        """Sync status.

        Returns a coherent local index snapshot and a separately observed Bitcoin Core tip height. The two heights can differ during indexing or a reorg. Conditional requests refresh these observations before validation.

        Endpoint: `GET /api/server/sync`"""
        return self.get_json('/api/server/sync')

    def get_disk_usage(self) -> DiskUsage:
        """Disk usage.

        Returns allocated file bytes for BRK and Bitcoin data. Each request scans both trees; these are independent observations, not an atomic filesystem snapshot. Conditional requests validate the newly observed totals. Directory-link cycles and excessive nesting fail without returning partial totals.

        Endpoint: `GET /api/server/disk`"""
        return self.get_json('/api/server/disk')

    def get_series_tree(self) -> TreeNode:
        """Series catalog.

        Returns the complete hierarchical catalog of available series organized as a tree structure. Series are grouped by categories and subcategories.

        Endpoint: `GET /api/series`"""
        return self.get_json('/api/series')

    def get_series_count(self) -> DetailedSeriesCount:
        """Series count.

        Returns the number of series available per index type.

        Endpoint: `GET /api/series/count`"""
        return self.get_json('/api/series/count')

    def get_indexes(self) -> List[IndexInfo]:
        """List available indexes.

        Returns all available indexes with their accepted query aliases. Use any alias when querying series.

        Endpoint: `GET /api/series/indexes`"""
        return self.get_json('/api/series/indexes')

    def list_series(self, page: Optional[int] = None, per_page: Optional[int] = None) -> PaginatedSeries:
        """Series list.

        Paginated flat list of all available series names. Use `page` query param for pagination.

        Endpoint: `GET /api/series/list`"""
        params = []
        if page is not None: params.append(f'page={page}')
        if per_page is not None: params.append(f'per_page={per_page}')
        query = '&'.join(params)
        path = f'/api/series/list{"?" + query if query else ""}'
        return self.get_json(path)

    def search_series(self, q: SeriesName, limit: Optional[Limit] = None) -> List[str]:
        """Search series.

        Search series by name or descriptive terms. Results prioritize whole query words in names, then descriptions, then fuzzy names, then fuzzy descriptions. Word order does not matter. Descriptions provide cohort terminology and formulas. The decoded q parameter is limited to 1024 UTF-8 bytes.

        Endpoint: `GET /api/series/search`"""
        params = []
        params.append(f'q={q}')
        if limit is not None: params.append(f'limit={limit}')
        query = '&'.join(params)
        path = f'/api/series/search{"?" + query if query else ""}'
        return self.get_json(path)

    def get_series_info(self, series: SeriesName) -> SeriesInfo:
        """Get series info.

        Returns the optional description, supported indexes, the indexes whose values can be null, the value type, and the optional unit (what the value type measures) for the specified series. The decoded series name is limited to 1024 UTF-8 bytes.

        Endpoint: `GET /api/series/{series}`"""
        return self.get_json(f'/api/series/{series}')

    def get_series(self, series: SeriesName, index: Index, start: Optional[RangeIndex] = None, end: Optional[RangeIndex] = None, limit: Optional[Limit] = None, format: Optional[Format] = None) -> Union[AnySeriesData, str]:
        """Get series data.

        Fetch data for a specific series at the given index. Use query parameters to filter by date range and format (json/csv).

        Endpoint: `GET /api/series/{series}/{index}`"""
        params = []
        if start is not None: params.append(f'start={start}')
        if end is not None: params.append(f'end={end}')
        if limit is not None: params.append(f'limit={limit}')
        if format is not None: params.append(f'format={format}')
        query = '&'.join(params)
        path = f'/api/series/{series}/{index}{"?" + query if query else ""}'
        if format == 'csv':
            return self.get_text(path)
        return _series_data(self.get_json(path))

    def get_series_data(self, series: SeriesName, index: Index, start: Optional[RangeIndex] = None, end: Optional[RangeIndex] = None, limit: Optional[Limit] = None, format: Optional[Format] = None) -> Union[List[Any], str]:
        """Get raw series data.

        Returns just the data array without the SeriesData wrapper. Supports the same range and format parameters as `GET /api/series/{series}/{index}`.

        Endpoint: `GET /api/series/{series}/{index}/data`"""
        params = []
        if start is not None: params.append(f'start={start}')
        if end is not None: params.append(f'end={end}')
        if limit is not None: params.append(f'limit={limit}')
        if format is not None: params.append(f'format={format}')
        query = '&'.join(params)
        path = f'/api/series/{series}/{index}/data{"?" + query if query else ""}'
        if format == 'csv':
            return self.get_text(path)
        return self.get_json(path)

    def get_series_latest(self, series: SeriesName, index: Index) -> Any:
        """Get latest series value.

        Returns the single most recent value for a series, unwrapped (not inside a SeriesData object).

        Endpoint: `GET /api/series/{series}/{index}/latest`"""
        return self.get_json(f'/api/series/{series}/{index}/latest')

    def get_series_len(self, series: SeriesName, index: Index) -> int:
        """Get series data length.

        Returns the total number of data points for a series at the given index.

        Endpoint: `GET /api/series/{series}/{index}/len`"""
        return self.get_json(f'/api/series/{series}/{index}/len')

    def get_series_version(self, series: SeriesName, index: Index) -> Version:
        """Get series version.

        Returns the vector's schema/computation version, not its length or latest update. Appends and reorgs do not by themselves change this version.

        Endpoint: `GET /api/series/{series}/{index}/version`"""
        return self.get_json(f'/api/series/{series}/{index}/version')

    def get_series_bulk(self, series: SeriesList, index: Index, start: Optional[RangeIndex] = None, end: Optional[RangeIndex] = None, limit: Optional[Limit] = None, format: Optional[Format] = None) -> Union[List[AnySeriesData], str]:
        """Bulk series data.

        Fetch multiple series in a single request. Supports filtering by index and date range. Returns an array of SeriesData objects. For a single series, use `get_series` instead.

        Endpoint: `GET /api/series/bulk`"""
        params = []
        params.append(f'series={series}')
        params.append(f'index={index}')
        if start is not None: params.append(f'start={start}')
        if end is not None: params.append(f'end={end}')
        if limit is not None: params.append(f'limit={limit}')
        if format is not None: params.append(f'format={format}')
        query = '&'.join(params)
        path = f'/api/series/bulk{"?" + query if query else ""}'
        if format == 'csv':
            return self.get_text(path)
        return [_series_data(raw) for raw in self.get_json(path)]

    def list_urpd_cohorts(self) -> List[Cohort]:
        """Available URPD cohorts.

        Cohorts for which URPD data is available. Returns names like `all`, `sth`, `lth`, `under_4m`, `under_6m`, `over_4m`, `over_6m`, `utxos_under_1h_old`.

        Endpoint: `GET /api/urpd`"""
        return self.get_json('/api/urpd')

    def list_urpd_dates(self, cohort: Cohort, weight: Optional[UrpdWeight] = None) -> List[Date]:
        """Available URPD dates.

        Dates for which a published block is available for the cohort and selected `weight`. One entry per UTC day, sorted ascending.

        Endpoint: `GET /api/urpd/{cohort}/dates`"""
        params = []
        if weight is not None: params.append(f'weight={weight}')
        query = '&'.join(params)
        path = f'/api/urpd/{cohort}/dates{"?" + query if query else ""}'
        return self.get_json(path)

    def get_urpd(self, cohort: Cohort, agg: Optional[UrpdAggregation] = None, weight: Optional[UrpdWeight] = None) -> Urpd:
        """Latest URPD.

        URPD for the latest published block. The response's `date` field echoes which date was served. Returns `{ cohort, height, date, weight, aggregation, close, total_supply, buckets }`. `close` and each bucket's `price_floor`, `realized_cap`, and `unrealized_pnl` are USD; `total_supply` and bucket `supply` are BTC. `unrealized_pnl` can be negative.

        Endpoint: `GET /api/urpd/{cohort}`"""
        params = []
        if agg is not None: params.append(f'agg={agg}')
        if weight is not None: params.append(f'weight={weight}')
        query = '&'.join(params)
        path = f'/api/urpd/{cohort}{"?" + query if query else ""}'
        return self.get_json(path)

    def get_urpd_at(self, cohort: Cohort, point: str, agg: Optional[UrpdAggregation] = None, weight: Optional[UrpdWeight] = None) -> Urpd:
        """URPD at block height or date.

        URPD for a cohort at a block height or the last block of a UTC day. Returns `{ cohort, height, date, weight, aggregation, close, total_supply, buckets }` where each bucket is `{ price_floor, supply, realized_cap, unrealized_pnl }`. `close`, `price_floor`, `realized_cap`, and `unrealized_pnl` are USD; `total_supply` and `supply` are BTC. `unrealized_pnl` can be negative.

        Endpoint: `GET /api/urpd/{cohort}/{point}`"""
        params = []
        if agg is not None: params.append(f'agg={agg}')
        if weight is not None: params.append(f'weight={weight}')
        query = '&'.join(params)
        path = f'/api/urpd/{cohort}/{point}{"?" + query if query else ""}'
        return self.get_json(path)

    def get_difficulty_adjustment(self) -> DifficultyAdjustment:
        """Difficulty adjustment.

        Get current difficulty adjustment progress and estimates.

        *[Mempool.space docs](https://mempool.space/docs/api/rest#get-difficulty-adjustment)*

        Endpoint: `GET /api/v1/difficulty-adjustment`"""
        return self.get_json('/api/v1/difficulty-adjustment')

    def get_prices(self) -> Prices:
        """Current BTC price.

        Returns bitcoin latest price (on-chain derived, USD only).

        *[Mempool.space docs](https://mempool.space/docs/api/rest#get-price)*

        Endpoint: `GET /api/v1/prices`"""
        return self.get_json('/api/v1/prices')

    def get_historical_price(self, timestamp: Optional[Timestamp] = None) -> HistoricalPrice:
        """Historical price.

        Completed four-hour BTC/USD closes, oldest first, labeled by interval end. With a UNIX timestamp, returns the latest nonempty completed close at or before it; before the first close returns an empty list. The current partial interval is excluded. USD only; exchangeRates is empty.

        *[Mempool.space docs](https://mempool.space/docs/api/rest#get-historical-price)*

        Endpoint: `GET /api/v1/historical-price`"""
        params = []
        if timestamp is not None: params.append(f'timestamp={timestamp}')
        query = '&'.join(params)
        path = f'/api/v1/historical-price{"?" + query if query else ""}'
        return self.get_json(path)

    def get_address_hash_prefix_matches(self, addr_type: OutputType, prefix: str) -> AddrHashPrefixMatches:
        """Address hash-prefix matches.

        Find addresses by address type and by the first 1-16 hex nibbles of RapidHash v3 over the raw address payload bytes. Intended for privacy-preserving client-side wallet discovery without sending raw addresses or xpubs. Fetch metadata with `GET /api/address/{address}`.

        Endpoint: `GET /api/address/hash-prefix/{addr_type}/{prefix}`"""
        return self.get_json(f'/api/address/hash-prefix/{addr_type}/{prefix}')

    def get_address(self, address: Addr) -> AddrStats:
        """Address information.

        Retrieve address information including current balance and transaction counts. Supports all standard Bitcoin address types (P2PKH, P2SH, P2WPKH, P2WSH, P2TR).

        *[Mempool.space docs](https://mempool.space/docs/api/rest#get-address)*

        Endpoint: `GET /api/address/{address}`"""
        return self.get_json(f'/api/address/{address}')

    def get_address_txs(self, address: Addr) -> List[Transaction]:
        """Address transactions.

        Get transaction history for an address, newest first. Returns up to 50 mempool transactions plus a confirmed page sized to fill the response to 50 total (chain floor of 25, so 25-50 confirmed depending on mempool weight). To paginate further confirmed history, request `GET /api/address/{address}/txs/chain/{after_txid}` with the last returned txid.

        *[Mempool.space docs](https://mempool.space/docs/api/rest#get-address-transactions)*

        Endpoint: `GET /api/address/{address}/txs`"""
        return self.get_json(f'/api/address/{address}/txs')

    def get_address_confirmed_txs(self, address: Addr) -> List[Transaction]:
        """Address confirmed transactions.

        Get the first 25 confirmed transactions for an address. For pagination, request `GET /api/address/{address}/txs/chain/{after_txid}` with the last returned txid.

        *[Mempool.space docs](https://mempool.space/docs/api/rest#get-address-transactions-chain)*

        Endpoint: `GET /api/address/{address}/txs/chain`"""
        return self.get_json(f'/api/address/{address}/txs/chain')

    def get_address_confirmed_txs_after(self, address: Addr, after_txid: Txid) -> List[Transaction]:
        """Address confirmed transactions (paginated).

        Get the next 25 confirmed transactions strictly older than `after_txid` (Esplora-canonical pagination form, matches mempool.space).

        *[Mempool.space docs](https://mempool.space/docs/api/rest#get-address-transactions-chain)*

        Endpoint: `GET /api/address/{address}/txs/chain/{after_txid}`"""
        return self.get_json(f'/api/address/{address}/txs/chain/{after_txid}')

    def get_address_mempool_txs(self, address: Addr) -> List[Transaction]:
        """Address mempool transactions.

        Get unconfirmed transactions for an address from the mempool, newest first (up to 50).

        *[Mempool.space docs](https://mempool.space/docs/api/rest#get-address-transactions-mempool)*

        Endpoint: `GET /api/address/{address}/txs/mempool`"""
        return self.get_json(f'/api/address/{address}/txs/mempool')

    def get_address_utxos(self, address: Addr) -> List[Utxo]:
        """Address UTXOs.

        Get unspent transaction outputs (UTXOs) for an address. Returns txid, vout, value, and confirmation status for each UTXO.

        *[Mempool.space docs](https://mempool.space/docs/api/rest#get-address-utxo)*

        Endpoint: `GET /api/address/{address}/utxo`"""
        return self.get_json(f'/api/address/{address}/utxo')

    def validate_address(self, address: str) -> AddrValidation:
        """Validate address.

        Validate a Bitcoin address and get information about its type and scriptPubKey. Returns `isvalid: false` with an error message for invalid addresses.

        *[Mempool.space docs](https://mempool.space/docs/api/rest#get-address-validate)*

        Endpoint: `GET /api/v1/validate-address/{address}`"""
        return self.get_json(f'/api/v1/validate-address/{address}')

    def get_block(self, hash: BlockHash) -> BlockInfo:
        """Block information.

        Retrieve block information by block hash. Returns block metadata including height, timestamp, difficulty, size, weight, and transaction count.

        *[Mempool.space docs](https://mempool.space/docs/api/rest#get-block)*

        Endpoint: `GET /api/block/{hash}`"""
        return self.get_json(f'/api/block/{hash}')

    def get_block_v1(self, hash: BlockHash) -> BlockInfoV1:
        """Block (v1).

        Returns block details with extras by hash.

        *[Mempool.space docs](https://mempool.space/docs/api/rest#get-block-v1)*

        Endpoint: `GET /api/v1/block/{hash}`"""
        return self.get_json(f'/api/v1/block/{hash}')

    def get_block_header(self, hash: BlockHash) -> Hex:
        """Block header.

        Returns the hex-encoded 80-byte block header.

        *[Mempool.space docs](https://mempool.space/docs/api/rest#get-block-header)*

        Endpoint: `GET /api/block/{hash}/header`"""
        return self.get_text(f'/api/block/{hash}/header')

    def get_block_by_height(self, height: Height) -> BlockHash:
        """Block hash by height.

        Retrieve the block hash at a given height. Returns the hash as plain text.

        *[Mempool.space docs](https://mempool.space/docs/api/rest#get-block-height)*

        Endpoint: `GET /api/block-height/{height}`"""
        return self.get_text(f'/api/block-height/{height}')

    def get_block_by_timestamp(self, timestamp: Timestamp) -> BlockTimestamp:
        """Block by timestamp.

        Find the block with the greatest header timestamp at or before the given UNIX timestamp, choosing the earliest height on ties.

        *[Mempool.space docs](https://mempool.space/docs/api/rest#get-block-timestamp)*

        Endpoint: `GET /api/v1/mining/blocks/timestamp/{timestamp}`"""
        return self.get_json(f'/api/v1/mining/blocks/timestamp/{timestamp}')

    def get_block_raw(self, hash: BlockHash) -> bytes:
        """Raw block.

        Returns the raw block data in binary format.

        *[Mempool.space docs](https://mempool.space/docs/api/rest#get-block-raw)*

        Endpoint: `GET /api/block/{hash}/raw`"""
        return self.get(f'/api/block/{hash}/raw')

    def get_block_status(self, hash: BlockHash) -> BlockStatus:
        """Block status.

        Retrieve the status of a block. Returns whether the block is in the best chain and, if so, its height and the hash of the next block.

        *[Mempool.space docs](https://mempool.space/docs/api/rest#get-block-status)*

        Endpoint: `GET /api/block/{hash}/status`"""
        return self.get_json(f'/api/block/{hash}/status')

    def get_block_tip_height(self) -> Height:
        """Block tip height.

        Returns the height of the last block.

        *[Mempool.space docs](https://mempool.space/docs/api/rest#get-block-tip-height)*

        Endpoint: `GET /api/blocks/tip/height`"""
        return int(self.get_text('/api/blocks/tip/height'))

    def get_block_tip_hash(self) -> BlockHash:
        """Block tip hash.

        Returns the hash of the last block.

        *[Mempool.space docs](https://mempool.space/docs/api/rest#get-block-tip-hash)*

        Endpoint: `GET /api/blocks/tip/hash`"""
        return self.get_text('/api/blocks/tip/hash')

    def get_block_txid(self, hash: BlockHash, index: BlockTxIndex) -> Txid:
        """Transaction ID at index.

        Retrieve a single transaction ID at a specific index within a block. Returns plain text txid.

        *[Mempool.space docs](https://mempool.space/docs/api/rest#get-block-transaction-id)*

        Endpoint: `GET /api/block/{hash}/txid/{index}`"""
        return self.get_text(f'/api/block/{hash}/txid/{index}')

    def get_block_txids(self, hash: BlockHash) -> List[Txid]:
        """Block transaction IDs.

        Retrieve all transaction IDs in a block. Returns an array of txids in block order.

        *[Mempool.space docs](https://mempool.space/docs/api/rest#get-block-transaction-ids)*

        Endpoint: `GET /api/block/{hash}/txids`"""
        return self.get_json(f'/api/block/{hash}/txids')

    def get_block_txs(self, hash: BlockHash) -> List[Transaction]:
        """Block transactions.

        Retrieve transactions in a block by block hash. Returns up to 25 transactions starting from index 0.

        *[Mempool.space docs](https://mempool.space/docs/api/rest#get-block-transactions)*

        Endpoint: `GET /api/block/{hash}/txs`"""
        return self.get_json(f'/api/block/{hash}/txs')

    def get_block_txs_from_index(self, hash: BlockHash, start_index: BlockTxIndex) -> List[Transaction]:
        """Block transactions (paginated).

        Retrieve transactions in a block by block hash, starting from the specified index. Returns up to 25 transactions at a time.

        *[Mempool.space docs](https://mempool.space/docs/api/rest#get-block-transactions)*

        Endpoint: `GET /api/block/{hash}/txs/{start_index}`"""
        return self.get_json(f'/api/block/{hash}/txs/{start_index}')

    def get_blocks(self) -> List[BlockInfo]:
        """Recent blocks.

        Retrieve the last 10 blocks. Returns block metadata for each block.

        *[Mempool.space docs](https://mempool.space/docs/api/rest#get-blocks)*

        Endpoint: `GET /api/blocks`"""
        return self.get_json('/api/blocks')

    def get_blocks_from_height(self, height: Height) -> List[BlockInfo]:
        """Blocks from height.

        Retrieve up to 10 blocks going backwards from the given height. For example, height=100 returns blocks 100, 99, 98, ..., 91. Height=0 returns only block 0.

        *[Mempool.space docs](https://mempool.space/docs/api/rest#get-blocks)*

        Endpoint: `GET /api/blocks/{height}`"""
        return self.get_json(f'/api/blocks/{height}')

    def get_blocks_v1(self) -> List[BlockInfoV1]:
        """Recent blocks with extras.

        Retrieve the last 15 blocks with extended data including pool identification and fee statistics.

        *[Mempool.space docs](https://mempool.space/docs/api/rest#get-blocks-v1)*

        Endpoint: `GET /api/v1/blocks`"""
        return self.get_json('/api/v1/blocks')

    def get_blocks_v1_from_height(self, height: Height) -> List[BlockInfoV1]:
        """Blocks from height with extras.

        Retrieve up to 15 blocks with extended data going backwards from the given height.

        *[Mempool.space docs](https://mempool.space/docs/api/rest#get-blocks-v1)*

        Endpoint: `GET /api/v1/blocks/{height}`"""
        return self.get_json(f'/api/v1/blocks/{height}')

    def get_pools(self) -> List[PoolInfo]:
        """List all mining pools.

        Get list of all known mining pools with their identifiers.

        *[Mempool.space docs](https://mempool.space/docs/api/rest#get-mining-pools)*

        Endpoint: `GET /api/v1/mining/pools`"""
        return self.get_json('/api/v1/mining/pools')

    def get_pool_stats(self, time_period: TimePeriod) -> PoolsSummary:
        """Mining pool statistics.

        Get mining pool statistics for a time period. Valid periods: `24h`, `3d`, `1w`, `1m`, `3m`, `6m`, `1y`, `2y`, `3y`.

        *[Mempool.space docs](https://mempool.space/docs/api/rest#get-mining-pools)*

        Endpoint: `GET /api/v1/mining/pools/{time_period}`"""
        return self.get_json(f'/api/v1/mining/pools/{time_period}')

    def get_pool(self, slug: PoolSlug) -> PoolDetail:
        """Mining pool details.

        Get detailed information about a specific mining pool including block counts and shares for different time periods.

        *[Mempool.space docs](https://mempool.space/docs/api/rest#get-mining-pool)*

        Endpoint: `GET /api/v1/mining/pool/{slug}`"""
        return self.get_json(f'/api/v1/mining/pool/{slug}')

    def get_pools_hashrate(self) -> List[PoolHashrateEntry]:
        """All pools hashrate (all time).

        Get hashrate data for all mining pools.

        *[Mempool.space docs](https://mempool.space/docs/api/rest#get-mining-pool-hashrates)*

        Endpoint: `GET /api/v1/mining/hashrate/pools`"""
        return self.get_json('/api/v1/mining/hashrate/pools')

    def get_pools_hashrate_by_period(self, time_period: TimePeriod) -> List[PoolHashrateEntry]:
        """All pools hashrate.

        Get hashrate data for all mining pools for a time period. Valid periods: `1m`, `3m`, `6m`, `1y`, `2y`, `3y`.

        *[Mempool.space docs](https://mempool.space/docs/api/rest#get-mining-pool-hashrates)*

        Endpoint: `GET /api/v1/mining/hashrate/pools/{time_period}`"""
        return self.get_json(f'/api/v1/mining/hashrate/pools/{time_period}')

    def get_pool_hashrate(self, slug: PoolSlug) -> List[PoolHashrateEntry]:
        """Mining pool hashrate.

        Get hashrate history for a specific mining pool.

        *[Mempool.space docs](https://mempool.space/docs/api/rest#get-mining-pool-hashrate)*

        Endpoint: `GET /api/v1/mining/pool/{slug}/hashrate`"""
        return self.get_json(f'/api/v1/mining/pool/{slug}/hashrate')

    def get_pool_blocks(self, slug: PoolSlug) -> List[BlockInfoV1]:
        """Mining pool blocks.

        Get up to 100 recent blocks mined by a specific pool.

        *[Mempool.space docs](https://mempool.space/docs/api/rest#get-mining-pool-blocks)*

        Endpoint: `GET /api/v1/mining/pool/{slug}/blocks`"""
        return self.get_json(f'/api/v1/mining/pool/{slug}/blocks')

    def get_pool_blocks_from(self, slug: PoolSlug, height: Height) -> List[BlockInfoV1]:
        """Mining pool blocks from height.

        Get up to 100 blocks mined by a specific pool before (and including) the given height.

        *[Mempool.space docs](https://mempool.space/docs/api/rest#get-mining-pool-blocks)*

        Endpoint: `GET /api/v1/mining/pool/{slug}/blocks/{height}`"""
        return self.get_json(f'/api/v1/mining/pool/{slug}/blocks/{height}')

    def get_hashrate(self) -> HashrateSummary:
        """Network hashrate (all time).

        Get network hashrate and difficulty data for all time.

        *[Mempool.space docs](https://mempool.space/docs/api/rest#get-hashrate)*

        Endpoint: `GET /api/v1/mining/hashrate`"""
        return self.get_json('/api/v1/mining/hashrate')

    def get_hashrate_by_period(self, time_period: TimePeriod) -> HashrateSummary:
        """Network hashrate.

        Get network hashrate and difficulty data for a time period. Valid periods: `24h`, `3d`, `1w`, `1m`, `3m`, `6m`, `1y`, `2y`, `3y`.

        *[Mempool.space docs](https://mempool.space/docs/api/rest#get-hashrate)*

        Endpoint: `GET /api/v1/mining/hashrate/{time_period}`"""
        return self.get_json(f'/api/v1/mining/hashrate/{time_period}')

    def get_difficulty_adjustments(self) -> List[DifficultyAdjustmentEntry]:
        """Difficulty adjustments (all time).

        Get historical difficulty adjustments including timestamp, block height, difficulty value, and percentage change.

        *[Mempool.space docs](https://mempool.space/docs/api/rest#get-difficulty-adjustments)*

        Endpoint: `GET /api/v1/mining/difficulty-adjustments`"""
        return self.get_json('/api/v1/mining/difficulty-adjustments')

    def get_difficulty_adjustments_by_period(self, time_period: TimePeriod) -> List[DifficultyAdjustmentEntry]:
        """Difficulty adjustments.

        Get historical difficulty adjustments for a time period. Valid periods: `24h`, `3d`, `1w`, `1m`, `3m`, `6m`, `1y`, `2y`, `3y`.

        *[Mempool.space docs](https://mempool.space/docs/api/rest#get-difficulty-adjustments)*

        Endpoint: `GET /api/v1/mining/difficulty-adjustments/{time_period}`"""
        return self.get_json(f'/api/v1/mining/difficulty-adjustments/{time_period}')

    def get_reward_stats(self, block_count: int) -> RewardStats:
        """Mining reward statistics.

        Get mining reward statistics for the last N blocks including total rewards, fees, and transaction count.

        *[Mempool.space docs](https://mempool.space/docs/api/rest#get-reward-stats)*

        Endpoint: `GET /api/v1/mining/reward-stats/{block_count}`"""
        return self.get_json(f'/api/v1/mining/reward-stats/{block_count}')

    def get_block_fees(self, time_period: TimePeriod) -> List[BlockFeesEntry]:
        """Block fees.

        Get average total fees per block for a time period. Valid periods: `24h`, `3d`, `1w`, `1m`, `3m`, `6m`, `1y`, `2y`, `3y`.

        *[Mempool.space docs](https://mempool.space/docs/api/rest#get-block-fees)*

        Endpoint: `GET /api/v1/mining/blocks/fees/{time_period}`"""
        return self.get_json(f'/api/v1/mining/blocks/fees/{time_period}')

    def get_block_rewards(self, time_period: TimePeriod) -> List[BlockRewardsEntry]:
        """Block rewards.

        Get average coinbase reward (subsidy + fees) per block for a time period. Valid periods: `24h`, `3d`, `1w`, `1m`, `3m`, `6m`, `1y`, `2y`, `3y`.

        *[Mempool.space docs](https://mempool.space/docs/api/rest#get-block-rewards)*

        Endpoint: `GET /api/v1/mining/blocks/rewards/{time_period}`"""
        return self.get_json(f'/api/v1/mining/blocks/rewards/{time_period}')

    def get_block_fee_rates(self, time_period: TimePeriod) -> List[BlockFeeRatesEntry]:
        """Block fee rates.

        Get block fee rate percentiles (min, 10th, 25th, median, 75th, 90th, max) for a time period. Valid periods: `24h`, `3d`, `1w`, `1m`, `3m`, `6m`, `1y`, `2y`, `3y`.

        *[Mempool.space docs](https://mempool.space/docs/api/rest#get-block-feerates)*

        Endpoint: `GET /api/v1/mining/blocks/fee-rates/{time_period}`"""
        return self.get_json(f'/api/v1/mining/blocks/fee-rates/{time_period}')

    def get_block_sizes_weights(self, time_period: TimePeriod) -> BlockSizesWeights:
        """Block sizes and weights.

        Get average block sizes and weights for a time period. Valid periods: `24h`, `3d`, `1w`, `1m`, `3m`, `6m`, `1y`, `2y`, `3y`.

        *[Mempool.space docs](https://mempool.space/docs/api/rest#get-sizes-weights)*

        Endpoint: `GET /api/v1/mining/blocks/sizes-weights/{time_period}`"""
        return self.get_json(f'/api/v1/mining/blocks/sizes-weights/{time_period}')

    def get_mempool_blocks(self) -> List[MempoolBlock]:
        """Projected mempool blocks.

        Projected blocks for fee estimation. Block 0 reflects Bitcoin Core's actual next-block selection; blocks 1+ are a fee-tier approximation.

        *[Mempool.space docs](https://mempool.space/docs/api/rest#get-mempool-blocks-fees)*

        Endpoint: `GET /api/v1/fees/mempool-blocks`"""
        return self.get_json('/api/v1/fees/mempool-blocks')

    def get_recommended_fees(self) -> RecommendedFees:
        """Recommended fees.

        Recommended fee rates by confirmation target.

        *[Mempool.space docs](https://mempool.space/docs/api/rest#get-recommended-fees)*

        Endpoint: `GET /api/v1/fees/recommended`"""
        return self.get_json('/api/v1/fees/recommended')

    def get_precise_fees(self) -> RecommendedFees:
        """Recommended fee rates (precise).

        Recommended fee rates by confirmation target, with up to three decimal places and support for sub-sat/vB rates.

        *[Mempool.space docs](https://mempool.space/docs/api/rest#get-recommended-fees-precise)*

        Endpoint: `GET /api/v1/fees/precise`"""
        return self.get_json('/api/v1/fees/precise')

    def get_mempool(self) -> MempoolInfo:
        """Mempool statistics.

        Get current mempool statistics including transaction count, total vsize, total fees, and fee histogram.

        *[Mempool.space docs](https://mempool.space/docs/api/rest#get-mempool)*

        Endpoint: `GET /api/mempool`"""
        return self.get_json('/api/mempool')

    def get_mempool_hash(self) -> NextBlockHash:
        """Mempool content hash.

        Returns an opaque content token for the published projected next block, including statistics and transaction bodies. This is not the HTTP ETag. An unchanged token means unchanged content, not necessarily a stalled sync loop.

        Endpoint: `GET /api/mempool/hash`"""
        return self.get_json('/api/mempool/hash')

    def get_mempool_txids(self) -> List[Txid]:
        """Mempool transaction IDs.

        Get all transaction IDs currently in the mempool.

        *[Mempool.space docs](https://mempool.space/docs/api/rest#get-mempool-transaction-ids)*

        Endpoint: `GET /api/mempool/txids`"""
        return self.get_json('/api/mempool/txids')

    def get_mempool_recent(self) -> List[MempoolRecentTx]:
        """Recent mempool transactions.

        Get the last 10 transactions to enter the mempool.

        *[Mempool.space docs](https://mempool.space/docs/api/rest#get-mempool-recent)*

        Endpoint: `GET /api/mempool/recent`"""
        return self.get_json('/api/mempool/recent')

    def get_replacements(self) -> List[ReplacementNode]:
        """Recent RBF replacements.

        Returns up to 25 most-recent RBF replacement trees across the whole mempool. Each entry has the same shape as `tx_rbf().replacements`.

        *[Mempool.space docs](https://mempool.space/docs/api/rest#get-replacements)*

        Endpoint: `GET /api/v1/replacements`"""
        return self.get_json('/api/v1/replacements')

    def get_fullrbf_replacements(self) -> List[ReplacementNode]:
        """Recent full-RBF replacements.

        Same response shape as `GET /api/v1/replacements`, but limited to trees where at least one predecessor was non-signaling (full-RBF).

        *[Mempool.space docs](https://mempool.space/docs/api/rest#get-fullrbf-replacements)*

        Endpoint: `GET /api/v1/fullrbf/replacements`"""
        return self.get_json('/api/v1/fullrbf/replacements')

    def get_block_template(self) -> BlockTemplate:
        """Projected next block template.

        Bitcoin Core's `getblocktemplate` selection: full transaction bodies in GBT order with aggregate stats. The returned `hash` is an opaque content token; pass it to `GET /api/v1/mempool/block-template/diff/{hash}` to fetch deltas instead of refetching the whole template.

        Endpoint: `GET /api/v1/mempool/block-template`"""
        return self.get_json('/api/v1/mempool/block-template')

    def get_block_template_diff(self, hash: NextBlockHash) -> BlockTemplateDiff:
        """Block template diff since hash.

        Delta of the projected next block since `<hash>`. `order` is the full new template in order: each entry is either a number (index into the prior template the client cached at `<hash>`) or a transaction object (new body to insert at this position). Walk `order` once to rebuild; `removed` is a convenience list of txids that left so clients can evict cached bodies. After applying, use the response `hash` as `<hash>` on the next call to keep iterating. Returns `404` when `<hash>` has aged out of server history; clients should fall back to `GET /api/v1/mempool/block-template`.

        Endpoint: `GET /api/v1/mempool/block-template/diff/{hash}`"""
        return self.get_json(f'/api/v1/mempool/block-template/diff/{hash}')

    def get_live_price(self) -> Dollars:
        """Live BTC/USD price.

        Returns the current BTC/USD price in dollars, derived from on-chain round-dollar output patterns in the last 12 blocks plus mempool.

        Endpoint: `GET /api/mempool/price`"""
        return self.get_json('/api/mempool/price')

    def get_tx_by_index(self, index: TxIndex) -> Txid:
        """Txid by index.

        Retrieve the transaction ID (txid) at a given global transaction index. Returns the txid as plain text.

        Endpoint: `GET /api/tx-index/{index}`"""
        return self.get_text(f'/api/tx-index/{index}')

    def get_cpfp(self, txid: Txid) -> CpfpInfo:
        """CPFP info.

        Returns ancestors and descendants for a CPFP (Child Pays For Parent) transaction, including the effective fee rate of the package.

        *[Mempool.space docs](https://mempool.space/docs/api/rest#get-children-pay-for-parent)*

        Endpoint: `GET /api/v1/cpfp/{txid}`"""
        return self.get_json(f'/api/v1/cpfp/{txid}')

    def get_tx_rbf(self, txid: Txid) -> RbfResponse:
        """RBF replacement history.

        Returns the RBF replacement tree for a transaction, if any. Both `replacements` and `replaces` are null when the tx has no known RBF history within the mempool monitor's retention window.

        *[Mempool.space docs](https://mempool.space/docs/api/rest#get-transaction-rbf-history)*

        Endpoint: `GET /api/v1/tx/{txid}/rbf`"""
        return self.get_json(f'/api/v1/tx/{txid}/rbf')

    def get_tx(self, txid: Txid) -> Transaction:
        """Transaction information.

        Retrieve complete transaction data by transaction ID (txid). Returns inputs, outputs, fee, size, and confirmation status.

        *[Mempool.space docs](https://mempool.space/docs/api/rest#get-transaction)*

        Endpoint: `GET /api/tx/{txid}`"""
        return self.get_json(f'/api/tx/{txid}')

    def get_tx_hex(self, txid: Txid) -> Hex:
        """Transaction hex.

        Retrieve the raw transaction as a hex-encoded string. Returns the serialized transaction in hexadecimal format.

        *[Mempool.space docs](https://mempool.space/docs/api/rest#get-transaction-hex)*

        Endpoint: `GET /api/tx/{txid}/hex`"""
        return self.get_text(f'/api/tx/{txid}/hex')

    def get_tx_merkleblock_proof(self, txid: Txid) -> Hex:
        """Transaction merkleblock proof.

        Get the merkleblock proof for a transaction (BIP37 format, hex encoded).

        *[Mempool.space docs](https://mempool.space/docs/api/rest#get-transaction-merkleblock-proof)*

        Endpoint: `GET /api/tx/{txid}/merkleblock-proof`"""
        return self.get_text(f'/api/tx/{txid}/merkleblock-proof')

    def get_tx_merkle_proof(self, txid: Txid) -> MerkleProof:
        """Transaction merkle proof.

        Get the merkle inclusion proof for a transaction.

        *[Mempool.space docs](https://mempool.space/docs/api/rest#get-transaction-merkle-proof)*

        Endpoint: `GET /api/tx/{txid}/merkle-proof`"""
        return self.get_json(f'/api/tx/{txid}/merkle-proof')

    def get_tx_outspend(self, txid: Txid, vout: Vout) -> TxOutspend:
        """Output spend status.

        Get the spending status of a transaction output. Returns whether the output has been spent and, if so, the spending transaction details.

        *[Mempool.space docs](https://mempool.space/docs/api/rest#get-transaction-outspend)*

        Endpoint: `GET /api/tx/{txid}/outspend/{vout}`"""
        return self.get_json(f'/api/tx/{txid}/outspend/{vout}')

    def get_tx_outspends(self, txid: Txid) -> List[TxOutspend]:
        """All output spend statuses.

        Get the spending status of all outputs in a transaction. Returns an array with the spend status for each output.

        *[Mempool.space docs](https://mempool.space/docs/api/rest#get-transaction-outspends)*

        Endpoint: `GET /api/tx/{txid}/outspends`"""
        return self.get_json(f'/api/tx/{txid}/outspends')

    def get_tx_raw(self, txid: Txid) -> bytes:
        """Transaction raw.

        Returns a transaction as binary data.

        *[Mempool.space docs](https://mempool.space/docs/api/rest#get-transaction-raw)*

        Endpoint: `GET /api/tx/{txid}/raw`"""
        return self.get(f'/api/tx/{txid}/raw')

    def get_tx_status(self, txid: Txid) -> TxStatus:
        """Transaction status.

        Retrieve the confirmation status of a transaction. Returns whether the transaction is confirmed and, if so, the block height, hash, and timestamp.

        *[Mempool.space docs](https://mempool.space/docs/api/rest#get-transaction-status)*

        Endpoint: `GET /api/tx/{txid}/status`"""
        return self.get_json(f'/api/tx/{txid}/status')

    def get_transaction_times(self, txId: List[Txid]) -> List[int]:
        """Transaction first-seen times.

        Returns timestamps when transactions were first seen in the mempool. Returns 0 for mined or unknown transactions.

        *[Mempool.space docs](https://mempool.space/docs/api/rest#get-transaction-times)*

        Endpoint: `GET /api/v1/transaction-times`"""
        params = []
        for _v in txId: params.append(f'txId[]={_v}')
        query = '&'.join(params)
        path = f'/api/v1/transaction-times{"?" + query if query else ""}'
        return self.get_json(path)

    def post_tx(self, body: str) -> Txid:
        """Broadcast transaction.

        Submit a raw transaction as hexadecimal text (at most 8,000,000 request bytes, including whitespace). Returns its txid as plain text. No responses are cached. Cancellation or a transport error after dispatch may leave the submission outcome unknown; do not automatically retry.

        *[Mempool.space docs](https://mempool.space/docs/api/rest#post-transaction)*

        Endpoint: `POST /api/tx`"""
        return self.post_text('/api/tx', body)

    def get_oracle_price(self) -> Dollars:
        """Live BTC/USD price.

        Current BTC/USD price in dollars. Same value as `GET /api/mempool/price`. Confirmed per-height history is available at `GET /api/series/price/height`.

        Endpoint: `GET /api/oracle/price`"""
        return self.get_json('/api/oracle/price')

    def get_oracle_histogram_payments_live(self) -> List[int]:
        """Live payment output histogram.

        Live smoothed histogram of oracle-eligible payment outputs, binned by output value on the oracle log scale. It combines the committed oracle window with the complete mempool's eligible outputs from a matching chain publication. A flat array of log-scale bins.

        Endpoint: `GET /api/oracle/histogram/payments/live`"""
        return self.get_json('/api/oracle/histogram/payments/live')

    def get_oracle_histogram_payments(self, point: str) -> List[int]:
        """Payment output histogram at height or day.

        Smoothed histogram of oracle-eligible payment outputs for a confirmed point. A block height (`840000`) gives that block's oracle payment histogram; a calendar date (`YYYY-MM-DD`) gives the average of that day's per-block payment histograms. A flat array of log-scale bins.

        Endpoint: `GET /api/oracle/histogram/payments/{point}`"""
        return self.get_json(f'/api/oracle/histogram/payments/{point}')

    def get_oracle_histogram_outputs_live(self) -> List[int]:
        """Live output value histogram.

        Live unfiltered output value histogram for the complete published mempool. Every live output is binned by value on the oracle log scale; no oracle payment filters are applied. A flat array of log-scale bins, all zero when no mempool is configured.

        Endpoint: `GET /api/oracle/histogram/outputs/live`"""
        return self.get_json('/api/oracle/histogram/outputs/live')

    def get_oracle_histogram_outputs(self, point: str) -> List[int]:
        """Output value histogram at height or day.

        Unfiltered output value histogram for a confirmed point. A block height (`840000`) gives every output in that block, coinbase included, binned by value on the oracle log scale; a calendar date (`YYYY-MM-DD`) sums every block that day. A flat array of log-scale bins.

        Endpoint: `GET /api/oracle/histogram/outputs/{point}`"""
        return self.get_json(f'/api/oracle/histogram/outputs/{point}')

    def get_openapi(self) -> str:
        """OpenAPI specification.

        Full OpenAPI 3.1 specification for this API.

        Endpoint: `GET /openapi.json`"""
        return self.get_text('/openapi.json')

    def get_api(self) -> Any:
        """Compact OpenAPI specification.

        Compact OpenAPI specification optimized for LLM consumption. Removes redundant fields while preserving essential API information. The full specification is available at `GET /openapi.json`.

        Endpoint: `GET /api.json`"""
        return self.get_json('/api.json')
