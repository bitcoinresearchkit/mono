# Auto-generated BRK Python client
# Do not edit manually

from __future__ import annotations
from dataclasses import dataclass
from typing import TypeVar, Generic, Any, Dict, Optional, List, Iterator, Literal, TypedDict, Union, Protocol, overload, Tuple, TYPE_CHECKING
from http.client import HTTPSConnection, HTTPConnection
from urllib.parse import urlparse
from datetime import date, datetime, timedelta, timezone
import json

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
# Unified index for any address type (funded or empty)
AnyAddrIndex = TypeIndex
# Unsigned basis points stored as u16.
# 1 bp = 0.0001. Range: 0–6.5535.
# Use for bounded 0–1 ratios (dominance, adoption, liveliness, etc.).
# `u16::MAX` is reserved as a NaN sentinel.
BasisPoints16 = int
# Unsigned basis points stored as u32.
# 1 bp = 0.0001. Range: 0–429,496.7295.
# Use for unbounded unsigned ratios (MVRV, NVT, SOPR, etc.).
# `u32::MAX` is reserved as a NaN sentinel.
BasisPoints32 = int
# Signed basis points stored as i16.
# 1 bp = 0.0001. Range: -3.2767 to +3.2767.
# Use for signed bounded ratios (NUPL, net PnL ratios, etc.).
# `i16::MIN` is reserved as a NaN sentinel.
BasisPointsSigned16 = int
# Signed basis points stored as i32.
# 1 bp = 0.0001. Range: -214,748.3647 to +214,748.3647.
# Use for unbounded signed values (returns, growth rates, volatility, z-scores, etc.).
# `i32::MIN` is reserved as a NaN sentinel.
BasisPointsSigned32 = int
# Bitcoin amount as floating point (1 BTC = 100,000,000 satoshis)
Bitcoin = float
# URL-friendly mining pool identifier
PoolSlug = Literal["unknown", "blockfills", "ultimuspool", "terrapool", "luxor", "1thash", "btccom", "bitfarms", "huobipool", "wayicn", "canoepool", "btctop", "bitcoincom", "175btc", "gbminers", "axbt", "asicminer", "bitminter", "bitcoinrussia", "btcserv", "simplecoinus", "btcguild", "eligius", "ozcoin", "eclipsemc", "maxbtc", "triplemining", "coinlab", "50btc", "ghashio", "stminingcorp", "bitparking", "mmpool", "polmine", "kncminer", "bitalo", "f2pool", "hhtt", "megabigpower", "mtred", "nmcbit", "yourbtcnet", "givemecoins", "braiinspool", "antpool", "multicoinco", "bcpoolio", "cointerra", "kanopool", "solock", "ckpool", "nicehash", "bitclub", "bitcoinaffiliatenetwork", "btcc", "bwpool", "exxbw", "bitsolo", "bitfury", "21inc", "digitalbtc", "8baochi", "mybtccoinpool", "tbdice", "hashpool", "nexious", "bravomining", "hotpool", "okexpool", "bcmonster", "1hash", "bixin", "tatmaspool", "viabtc", "connectbtc", "batpool", "waterhole", "dcexploration", "dcex", "btpool", "58coin", "bitcoinindia", "shawnp0wers", "phashio", "rigpool", "haozhuzhu", "7pool", "miningkings", "hashbx", "dpool", "rawpool", "haominer", "helix", "bitcoinukraine", "poolin", "secretsuperstar", "tigerpoolnet", "sigmapoolcom", "okpooltop", "hummerpool", "tangpool", "bytepool", "spiderpool", "novablock", "miningcity", "binancepool", "minerium", "lubiancom", "okkong", "aaopool", "emcdpool", "foundryusa", "sbicrypto", "arkpool", "purebtccom", "marapool", "kucoinpool", "entrustcharitypool", "okminer", "titan", "pegapool", "btcnuggets", "cloudhashing", "digitalxmintsy", "telco214", "btcpoolparty", "multipool", "transactioncoinmining", "btcdig", "trickysbtcpool", "btcmp", "eobot", "unomp", "patels", "gogreenlight", "bitcoinindiapool", "ekanembtc", "canoe", "tiger", "1m1x", "zulupool", "secpool", "ocean", "whitepool", "wiz", "wk057", "futurebitapollosolo", "carbonnegative", "portlandhodl", "phoenix", "neopool", "maxipool", "bitfufupool", "gdpool", "miningdutch", "publicpool", "miningsquared", "innopolistech", "btclab", "parasite", "redrockpool", "est3lar", "braiinssolo", "solopoolcom", "noderunners"]
# Fee rate in sat/vB
FeeRate = float
# Weight in weight units (WU). Max block weight is 4,000,000 WU.
Weight = int
# Block height
Height = int
# UNIX timestamp in seconds
Timestamp = int
# Block hash
BlockHash = str
# Position of a transaction within a single block (0 = coinbase).
# Distinct from `TxIndex`, which is the chain-wide global tx index.
BlockTxIndex = int
# Content hash of the projected next block (block 0 of the mempool
# snapshot). Same value as the mempool ETag. Opaque token: pass back
# as `since` on `/api/v1/mempool/block-template/diff/{hash}` to fetch
# deltas.
NextBlockHash = int
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
#   not in the prior template and must be added at this position.
# 
# Reconstruction is a single pass: for each entry, either copy
# `prior[idx]` or append the inline body.
BlockTemplateDiffEntry = Union[int, "Transaction"]
# Unsigned cents (u64) - for values that should never be negative.
# Used for invested capital, realized cap, etc.
Cents = int
# Cents × Sats (u128) - price in cents multiplied by amount in sats.
# Uses u128 because large amounts at any price can overflow u64.
CentsSats = int
# Signed cents (i64) - for values that can be negative.
# Used for profit/loss calculations, deltas, etc.
CentsSigned = int
# Raw cents squared (u128) - stores cents² × sats without division.
# Used for precise accumulation of capitalized cap values: Σ(price² × sats).
# capitalized_price = capitalized_cap_raw / realized_cap_raw
CentsSquaredSats = int
# Closing price value for a time period
Close = Dollars
# URPD cohort identifier. Use `GET /api/urpd` to list available cohorts.
# 
# Validated at construction: non-empty, ASCII `[a-z0-9_]+`. Matches the
# schemars enum value set; the type therefore proves "this is a valid
# cohort name" wherever a `Cohort` is held.
Cohort = Literal["all", "sth", "lth", "utxos_under_1h_old", "utxos_1h_to_1d_old", "utxos_1d_to_1w_old", "utxos_1w_to_1m_old", "utxos_1m_to_2m_old", "utxos_2m_to_3m_old", "utxos_3m_to_4m_old", "utxos_4m_to_5m_old", "utxos_5m_to_6m_old", "utxos_6m_to_1y_old", "utxos_1y_to_2y_old", "utxos_2y_to_3y_old", "utxos_3y_to_4y_old", "utxos_4y_to_5y_old", "utxos_5y_to_6y_old", "utxos_6y_to_7y_old", "utxos_7y_to_8y_old", "utxos_8y_to_10y_old", "utxos_10y_to_12y_old", "utxos_12y_to_15y_old", "utxos_over_15y_old"]
# Coinbase scriptSig tag for pool identification.
# 
# Stored as a fixed 101-byte record (1 byte length + 100 bytes data).
# Uses `[u8; 101]` internally so that `size_of::<CoinbaseTag>()` matches
# the serialized `Bytes::Array` size (vecdb requires this for alignment).
# 
# Bitcoin consensus limits coinbase scriptSig to 2-100 bytes.
CoinbaseTag = str
# Value type for the deprecated cost-basis distribution output.
CostBasisValue = Literal["supply", "realized", "unrealized"]
# Aggregation strategy for URPD buckets.
# Options: raw (no aggregation), lin200/lin500/lin1000 (linear $200/$500/$1000),
# log10/log50/log100/log200/log500/log1000/log2000 (logarithmic with 10/50/100/200/500/1000/2000 buckets per decade).
UrpdAggregation = Literal["raw", "lin200", "lin500", "lin1000", "log10", "log50", "log100", "log200", "log500", "log1000", "log2000"]
# Position of a transaction inside a `CpfpCluster.txs` array. Cluster-local,
# has no meaning outside the enclosing cluster.
CpfpClusterTxIndex = int
# Virtual size in vbytes (weight / 4, rounded up). Max block vsize is ~1,000,000 vB.
VSize = int
# Date in YYYYMMDD format stored as u32
Date = int
# Output format for API responses
Format = Literal["json", "csv"]
# Maximum number of results to return. Defaults to 100 if not specified.
Limit = int
# A range boundary: integer index, date, or timestamp.
RangeIndex = Union[int, Date, Timestamp]
Day1 = int
Day3 = int
EmptyAddrIndex = TypeIndex
EmptyOutputIndex = TypeIndex
Epoch = int
# Exchange rates (USD base, on-chain only — no fiat pairs available)
ExchangeRates = dict
FundedAddrIndex = TypeIndex
Halving = int
# Hex-encoded string. Transparent wrapper over `String`: serializes
# as a plain JSON string and derefs to `str`, so anywhere `&str` or
# `AsRef<[u8]>` is expected the `Hex` "just works".
Hex = str
# Highest price value for a time period
High = Dollars
Hour1 = int
Hour12 = int
Hour4 = int
# Aggregation dimension for querying series. Includes time-based (date, week, month, year),
# block-based (height, tx_index), and address/output type indexes.
Index = Literal["minute10", "minute30", "hour1", "hour4", "hour12", "day1", "day3", "week1", "month1", "month3", "month6", "year1", "year10", "halving", "epoch", "height", "tx_index", "txin_index", "txout_index", "empty_output_index", "op_return_index", "p2a_addr_index", "p2ms_output_index", "p2pk33_addr_index", "p2pk65_addr_index", "p2pkh_addr_index", "p2sh_addr_index", "p2tr_addr_index", "p2wpkh_addr_index", "p2wsh_addr_index", "unknown_output_index", "funded_addr_index", "empty_addr_index"]
# Series name
SeriesName = str
# Lowest price value for a time period
Low = Dollars
Minute10 = int
Minute30 = int
Month1 = int
Month3 = int
Month6 = int
# Opening price value for a time period
Open = Dollars
OpReturnIndex = TypeIndex
OutPoint = int
P2AAddrIndex = TypeIndex
U8x2 = List[int]
P2ABytes = U8x2
P2MSOutputIndex = TypeIndex
P2PK33AddrIndex = TypeIndex
U8x33 = List[int]
P2PK33Bytes = U8x33
P2PK65AddrIndex = TypeIndex
U8x65 = List[int]
P2PK65Bytes = U8x65
P2PKHAddrIndex = TypeIndex
U8x20 = List[int]
P2PKHBytes = U8x20
P2SHAddrIndex = TypeIndex
P2SHBytes = U8x20
P2TRAddrIndex = TypeIndex
U8x32 = List[int]
P2TRBytes = U8x32
P2WPKHAddrIndex = TypeIndex
P2WPKHBytes = U8x20
P2WSHAddrIndex = TypeIndex
P2WSHBytes = U8x32
# Fractional satoshis (f64) - for representing USD prices in sats
# 
# Formula: `sats_fract = usd_value * 100_000_000 / btc_price`
# 
# When BTC is $100,000:
# - $1 = 1,000 sats
# - $0.001 = 1 sat
# - $0.0001 = 0.1 sats (fractional)
SatsFract = float
# Signed satoshis (i64) - for values that can be negative.
# Used for changes, deltas, profit/loss calculations, etc.
SatsSigned = int
# Version tracking for data schema and computed values.
# 
# Used to detect when stored data needs to be recomputed due to changes
# in computation logic or source data versions. Supports validation
# against persisted versions to ensure compatibility.
Version = int
# Comma-separated list of series names
SeriesList = str
StoredBool = bool
# Stored 32-bit floating point value
StoredF32 = float
# Fixed-size 64-bit floating point value optimized for on-disk storage
StoredF64 = float
# Fixed-size 64-bit signed integer optimized for on-disk storage
StoredI64 = int
StoredI8 = int
StoredU16 = int
# Fixed-size 32-bit unsigned integer optimized for on-disk storage
StoredU32 = int
# Fixed-size 64-bit unsigned integer optimized for on-disk storage
StoredU64 = int
# Time period for mining statistics.
# 
# Used to specify the lookback window for pool statistics, hashrate calculations,
# and other time-based mining series.
TimePeriod = Literal["24h", "3d", "1w", "1m", "3m", "6m", "1y", "2y", "3y", "all"]
# Hierarchical tree node for organizing series into categories
TreeNode = Union[dict[str, "TreeNode"], "SeriesLeafWithSchema"]
TxInIndex = int
TxOutIndex = int
# Input index in the spending transaction
Vin = int
# Transaction version number
TxVersion = int
UnknownOutputIndex = TypeIndex
Week1 = int
Year1 = int
Year10 = int
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
        funded_txo_count: Total number of transaction outputs that funded this address
        funded_txo_sum: Total amount in satoshis received by this address across all funded outputs
        spent_txo_count: Total number of transaction outputs spent from this address
        spent_txo_sum: Total amount in satoshis spent from this address
        tx_count: Total number of confirmed transactions involving this address
        type_index: Index of this address within its type on the blockchain
        realized_price: Realized price (average cost basis) in USD
    """
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
    addr_type: OutputType
    prefix: str

class AddrMempoolStats(TypedDict):
    """
    Address statistics in the mempool (unconfirmed transactions only)
    
    Based on mempool.space's format.

    Attributes:
        funded_txo_count: Number of unconfirmed transaction outputs funding this address
        funded_txo_sum: Total amount in satoshis being received in unconfirmed transactions
        spent_txo_count: Number of unconfirmed transaction inputs spending from this address
        spent_txo_sum: Total amount in satoshis being spent in unconfirmed transactions
        tx_count: Number of unconfirmed transactions involving this address
    """
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
    Address information compatible with mempool.space API format

    Attributes:
        address: Bitcoin address string
        addr_type: BRK address type (p2pk33, p2pk65, p2pkh, p2sh, p2wpkh, p2wsh, p2tr, etc.)
        chain_stats: Statistics for confirmed transactions on the blockchain
        mempool_stats: Statistics for unconfirmed transactions in the mempool
    """
    address: Addr
    addr_type: OutputType
    chain_stats: AddrChainStats
    mempool_stats: AddrMempoolStats

class AddrValidation(TypedDict):
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
    isvalid: bool
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

class BlockPool(TypedDict):
    """
    Mining pool identification for a block

    Attributes:
        id: Unique pool identifier
        name: Pool name
        slug: URL-friendly pool identifier
        blockNumber: This block's ordinal among blocks attributed to this pool
        minerNames: Miner name tags found in coinbase scriptsig
    """
    id: int
    name: str
    slug: PoolSlug
    blockNumber: int
    minerNames: Optional[List[str]]

class BlockExtras(TypedDict):
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
        utxoSetChange: UTXO set change (total outputs - total inputs, includes unspendable like OP_RETURN).
Note: intentionally differs from utxo_set_size diff which excludes unspendable outputs.
Matches mempool.space/bitcoin-cli behavior.
        utxoSetSize: Total spendable UTXO set size at this height (excludes OP_RETURN and other unspendable outputs)
        totalInputAmt: Total input amount in satoshis
        virtualSize: Virtual size in vbytes
        firstSeen: Timestamp when the block was first seen (always null, not yet supported)
        orphans: Orphaned blocks (always empty)
        price: USD price at block height
    """
    totalFees: Sats
    medianFee: FeeRate
    feeRange: List[FeeRate]
    reward: Sats
    pool: BlockPool
    avgFee: Sats
    avgFeeRate: FeeRate
    coinbaseRaw: str
    coinbaseAddress: Optional[str]
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
    firstSeen: Optional[int]
    orphans: List[str]
    price: Dollars

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

class BlockInfoV1(TypedDict):
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
    stale: bool
    extras: BlockExtras

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

class BlockStatus(TypedDict):
    """
    Block status indicating whether block is in the best chain

    Attributes:
        in_best_chain: Whether this block is in the best chain
        height: Block height (only if in best chain)
        next_best: Hash of the next block in the best chain (null if tip)
    """
    in_best_chain: bool
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

class TxOut(TypedDict):
    """
    Transaction output

    Attributes:
        scriptpubkey: Script pubkey (locking script)
        value: Value of the output in satoshis
    """
    scriptpubkey: str
    value: Sats

class TxIn(TypedDict):
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
    txid: Txid
    vout: Vout
    prevout: Union[TxOut, None]
    scriptsig: str
    scriptsig_asm: str
    witness: Witness
    is_coinbase: bool
    sequence: int
    inner_redeemscript_asm: str
    inner_witnessscript_asm: str

class TxStatus(TypedDict):
    """
    Transaction confirmation status

    Attributes:
        confirmed: Whether the transaction is confirmed
        block_height: Block height (only present if confirmed)
        block_hash: Block hash (only present if confirmed)
        block_time: Block timestamp (only present if confirmed)
    """
    confirmed: bool
    block_height: Union[Height, None]
    block_hash: Union[BlockHash, None]
    block_time: Union[Timestamp, None]

class Transaction(TypedDict):
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

class BlockTemplate(TypedDict):
    """
    Projected next-block contents from Bitcoin Core's `getblocktemplate`
    (block 0 of the snapshot). Returned by
    `GET /api/v1/mempool/block-template`.

    Attributes:
        hash: Pass back as `<hash>` on
`/api/v1/mempool/block-template/diff/{hash}` to fetch deltas.
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
    
    `removed` is redundant (computable from `order` by collecting prior
    indices that don't appear) but shipped for cache-eviction ergonomics.

    Attributes:
        hash: Current next-block hash. Use as `since` on the next diff call.
        since: Echoed prior hash the diff was computed against.
        order: New template in order. Each entry is either an index into the
prior template's transactions or a full transaction body.
        removed: Txids that left the projected next block since `since`
(confirmed, evicted, replaced, or pushed past block 0).
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

class CostBasisCohortParam(TypedDict):
    cohort: Cohort

class CostBasisParams(TypedDict):
    cohort: Cohort
    date: str

class CostBasisQuery(TypedDict):
    bucket: UrpdAggregation
    value: CostBasisValue

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

class CpfpInfo(TypedDict):
    """
    CPFP (Child Pays For Parent) information for a transaction.

    Attributes:
        ancestors: Ancestor transactions in the CPFP chain.
        bestDescendant: Best (highest fee rate) descendant, if any.
        descendants: Descendant transactions in the CPFP chain.
        effectiveFeePerVsize: Effective fee rate considering CPFP relationships (sat/vB).
This is the seed's chunk feerate after lift-merging, i.e. the
rate Core/mempool.space would surface for this tx.
        sigops: BIP-141 sigop cost for the seed tx (witness sigops count as 1,
legacy and P2SH-redeem sigops count as 4).
        fee: Transaction fee (sats).
        vsize: Virtual size of the seed tx (vbytes).
        adjustedVsize: Policy-adjusted virtual size: `max(vsize, sigops * 5)`.
        cluster: Cluster the seed belongs to: full tx list, SFL-linearized chunks,
and the seed's chunk index. Omitted when the seed has no
ancestors and no descendants (matches mempool.space).
    """
    ancestors: List[CpfpEntry]
    bestDescendant: Union[CpfpEntry, None]
    descendants: List[CpfpEntry]
    effectiveFeePerVsize: FeeRate
    sigops: SigOps
    fee: Sats
    vsize: VSize
    adjustedVsize: VSize
    cluster: Union[CpfpCluster, None]

class DataRangeFormat(TypedDict):
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
    Series count statistics - distinct series and total series-index combinations

    Attributes:
        distinct_series: Number of unique series available (e.g., realized_price, market_cap)
        total_endpoints: Total number of series-index combinations across all timeframes
        lazy_endpoints: Number of lazy (computed on-the-fly) series-index combinations
        stored_endpoints: Number of eager (stored on disk) series-index combinations
    """
    distinct_series: int
    total_endpoints: int
    lazy_endpoints: int
    stored_endpoints: int

class DetailedSeriesCount(TypedDict):
    """
    Detailed series count with per-database breakdown

    Attributes:
        distinct_series: Number of unique series available (e.g., realized_price, market_cap)
        total_endpoints: Total number of series-index combinations across all timeframes
        lazy_endpoints: Number of lazy (computed on-the-fly) series-index combinations
        stored_endpoints: Number of eager (stored on disk) series-index combinations
        by_db: Per-database breakdown of counts
    """
    distinct_series: int
    total_endpoints: int
    lazy_endpoints: int
    stored_endpoints: int
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

class DifficultyAdjustmentEntry(TypedDict):
    """
    A single difficulty adjustment entry.
    Serializes as array: [timestamp, height, difficulty, change_percent]

    Attributes:
        timestamp: Unix timestamp of the adjustment
        height: Block height of the adjustment
        difficulty: Difficulty value
        change_percent: Adjustment ratio (new/previous, e.g. 1.068 = +6.8%)
    """
    timestamp: Timestamp
    height: Height
    difficulty: float
    change_percent: float

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
        ratio: brk as percentage of Bitcoin data
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
        type: Error category: "invalid_request", "forbidden", "not_found", "unavailable", or "internal"
        code: Machine-readable error code (e.g. "invalid_addr", "series_not_found")
        message: Human-readable description
        doc_url: Link to API documentation
    """
    type: str
    code: str
    message: str
    doc_url: str

class ErrorBody(TypedDict):
    error: ErrorDetail

class FundedAddrData(TypedDict):
    """
    Data for a funded (non-empty) address with current balance

    Attributes:
        tx_count: Total transaction count
        funded_txo_count: Number of transaction outputs funded to this address
        spent_txo_count: Number of transaction outputs spent by this address
        received: Satoshis received by this address
        sent: Satoshis sent by this address
        realized_cap_raw: The realized capitalization: Σ(price × sats)
        capitalized_cap_raw: The capitalized cap: Σ(price² × sats)
    """
    tx_count: int
    funded_txo_count: int
    spent_txo_count: int
    received: Sats
    sent: Sats
    realized_cap_raw: CentsSats
    capitalized_cap_raw: CentsSquaredSats

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

class LegacySeriesParam(TypedDict):
    """
    Legacy path parameter for `/api/metric/{metric}`
    """
    metric: SeriesName

class LegacySeriesWithIndex(TypedDict):
    """
    Legacy path parameters for `/api/metric/{metric}/{index}`
    """
    metric: SeriesName
    index: Index

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
    fee_histogram: dict[str, VSize]

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
    `since` hash for `/api/v1/mempool/block-template/diff/{hash}`.
    """
    hash: NextBlockHash

class OHLCCents(TypedDict):
    """
    OHLC (Open, High, Low, Close) data in cents
    """
    open: Open
    high: High
    low: Low
    close: Close

class OHLCDollars(TypedDict):
    """
    OHLC (Open, High, Low, Close) data in dollars
    """
    open: Open
    high: High
    low: Low
    close: Close

class OHLCSats(TypedDict):
    """
    OHLC (Open, High, Low, Close) data in satoshis
    """
    open: Open
    high: High
    low: Low
    close: Close

class OptionalTimestampParam(TypedDict):
    """
    Optional UNIX timestamp query parameter
    """
    timestamp: Union[Timestamp, None]

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

class Pagination(TypedDict):
    """
    Pagination parameters for paginated API endpoints

    Attributes:
        page: Pagination index
        per_page: Results per page (default: 1000, max: 1000)
    """
    page: Optional[int]
    per_page: Optional[int]

class PoolBlockCounts(TypedDict):
    """
    Block counts for different time periods

    Attributes:
        all: Total blocks mined (all time)
        _24h: Blocks mined in last 24 hours
        _1w: Blocks mined in last week
    """
    all: int
    _24h: int
    _1w: int

class PoolBlockShares(TypedDict):
    """
    Pool's share of total blocks for different time periods

    Attributes:
        all: Share of all blocks (0.0 - 1.0)
        _24h: Share of blocks in last 24 hours (0.0 - 1.0)
        _1w: Share of blocks in last week (0.0 - 1.0)
    """
    all: float
    _24h: float
    _1w: float

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

class PoolDetail(TypedDict):
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
    pool: PoolDetailInfo
    blockCount: PoolBlockCounts
    blockShare: PoolBlockShares
    estimatedHashrate: int
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

class RbfTx(TypedDict):
    """
    Transaction summary carried inside an RBF replacement node. Shape
    matches mempool.space's `/api/v1/tx/:txid/rbf` and
    `/api/v1/replacements` responses.

    Attributes:
        value: Sum of output amounts.
        rbf: BIP-125 signaling: at least one input has sequence < 0xffffffff-1.
        fullRbf: Only populated on the root `tx` of an RBF response. `true` iff
this tx displaced at least one non-signaling predecessor.
    """
    txid: Txid
    fee: Sats
    vsize: VSize
    value: Sats
    rate: FeeRate
    time: Timestamp
    rbf: bool
    fullRbf: Optional[bool]

class ReplacementNode(TypedDict):
    """
    One node in an RBF replacement tree. The node's `tx` replaced each
    entry in `replaces`, recursively.

    Attributes:
        time: First-seen timestamp, duplicated here to match mempool.space's
on-the-wire shape.
        fullRbf: Any predecessor in this subtree was non-signaling.
        interval: Seconds between this node's `time` and the successor that
replaced it. Omitted on the root of an RBF response.
        mined: `Some(true)` iff this node's tx is currently confirmed. Absent
on serialization otherwise.
    """
    tx: RbfTx
    time: Timestamp
    fullRbf: bool
    interval: Optional[int]
    mined: Optional[bool]
    replaces: List["ReplacementNode"]

class RbfResponse(TypedDict):
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
    totalReward: Sats
    totalFee: Sats
    totalTx: int

class SearchQuery(TypedDict):
    """
    Attributes:
        q: Search query string
        limit: Maximum number of results
    """
    q: SeriesName
    limit: Limit

class SeriesInfo(TypedDict):
    """
    Metadata about a series

    Attributes:
        indexes: Available indexes
        type: Value type (e.g. "f32", "u64", "Sats")
    """
    indexes: List[Index]
    type: str

class SeriesLeafWithSchema(TypedDict):
    """
    SeriesLeaf with JSON Schema for client generation

    Attributes:
        name: The series name/identifier
        kind: The Rust type (e.g., "Sats", "StoredF64")
        indexes: Available indexes for this series
        type: JSON Schema type (e.g., "integer", "number", "string", "boolean", "array", "object")
    """
    name: str
    kind: str
    indexes: List[Index]
    type: str

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

class SeriesSelection(TypedDict):
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
    series: SeriesList
    index: Index
    start: Union[RangeIndex, None]
    end: Union[RangeIndex, None]
    limit: Union[Limit, None]
    format: Format

class SeriesSelectionLegacy(TypedDict):
    """
    Legacy series selection parameters (deprecated)

    Attributes:
        start: Inclusive start: integer index, date (YYYY-MM-DD), or timestamp (ISO 8601). Negative integers count from end. Aliases: `from`, `f`, `s`
        end: Exclusive end: integer index, date (YYYY-MM-DD), or timestamp (ISO 8601). Negative integers count from end. Aliases: `to`, `t`, `e`
        limit: Maximum number of values to return (ignored if `end` is set). Aliases: `count`, `c`, `l`
        format: Format of the output
    """
    index: Index
    ids: SeriesList
    start: Union[RangeIndex, None]
    end: Union[RangeIndex, None]
    limit: Union[Limit, None]
    format: Format

class SupplyState(TypedDict):
    """
    Current supply state tracking UTXO count and total value

    Attributes:
        utxo_count: Number of unspent transaction outputs
        value: Total value in satoshis
    """
    utxo_count: int
    value: Sats

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

class TxOutspend(TypedDict):
    """
    Status of an output indicating whether it has been spent

    Attributes:
        spent: Whether the output has been spent
        txid: Transaction ID of the spending transaction (only present if spent)
        vin: Input index in the spending transaction (only present if spent)
        status: Status of the spending transaction (only present if spent)
    """
    spent: bool
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

class TxidsParam(TypedDict):
    """
    Query parameter for transaction-times endpoint.
    
    Extracted manually because `serde_urlencoded` (and serde derive in general)
    doesn't support repeated keys like `txId[]=a&txId[]=b`. The schema is still
    declared via `JsonSchema` so the OpenAPI spec lists the parameter and the
    generated client SDKs see `txids: List[Txid]`.

    Attributes:
        txId: Transaction IDs to look up (max 250 per request).
    """
    txId: List[Txid]

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
    UTXO Realized Price Distribution for a cohort on a specific date.
    
    Supply is grouped by the close price at which each UTXO was last moved.
    Each bucket exposes three values: supply in BTC, realized cap contribution
    in USD (sum of `realized_price * supply` over the coins in the bucket), and
    unrealized P&L in USD (`close * supply - realized_cap`, can be negative).

    Attributes:
        aggregation: Aggregation strategy applied to the buckets.
        close: Close price on `date`, in USD. Anchor for `unrealized_pnl`.
        total_supply: Sum of `supply` across all buckets, in BTC.
    """
    cohort: Cohort
    date: Date
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
    Path parameters for `/api/urpd/{cohort}/{date}`.
    """
    cohort: Cohort
    date: str

class UrpdQuery(TypedDict):
    """
    Query parameters for URPD endpoints.

    Attributes:
        agg: Aggregation strategy. Default: raw (no aggregation). Accepts `bucket` as alias.
    """
    agg: UrpdAggregation

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


class BrkError(Exception):
    """Custom error class for BRK client errors."""

    def __init__(self, message: str, status: Optional[int] = None):
        super().__init__(message)
        self.status = status


class BrkClientBase:
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
                raise BrkError(f"HTTP error: {res.status}", res.status)
            return data
        except (ConnectionError, OSError, TimeoutError) as e:
            self._conn = None
            raise BrkError(str(e))

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
                raise BrkError(f"HTTP error: {res.status}", res.status)
            return data
        except (ConnectionError, OSError, TimeoutError) as e:
            self._conn = None
            raise BrkError(str(e))

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

    def __enter__(self) -> BrkClientBase:
        return self

    def __exit__(self, exc_type: Optional[type], exc_val: Optional[BaseException], exc_tb: Optional[Any]) -> None:
        self.close()


def _m(acc: str, s: str) -> str:
    """Build series name with suffix."""
    if not s: return acc
    return f"{acc}_{s}" if acc else s


def _p(prefix: str, acc: str) -> str:
    """Build series name with prefix."""
    return f"{prefix}_{acc}" if acc else prefix



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
_GENESIS = date(2009, 1, 3)  # day1 0, week1 0
_DAY_ONE = date(2009, 1, 9)  # day1 1 (6 day gap after genesis)
_EPOCH = datetime(2009, 1, 1, tzinfo=timezone.utc)
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
        return _GENESIS if i == 0 else _DAY_ONE + timedelta(days=i - 1)
    elif index == 'day3':
        return _EPOCH.date() - timedelta(days=1) + timedelta(days=i * 3)
    elif index == 'week1':
        return _GENESIS + timedelta(weeks=i)
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
        return date(2009 + i * 10, 1, 1)
    else:
        raise ValueError(f"{index} is not a date-based index")


def _date_to_index(index: str, d: Union[date, datetime]) -> int:
    """Convert a date/datetime to an index value for date-based indexes.

    Returns the floor index (latest index whose date is <= the given date).
    For sub-day indexes (minute*, hour*), a plain date is treated as midnight UTC.
    """
    if index in ('minute10', 'minute30', 'hour1', 'hour4', 'hour12'):
        if isinstance(d, datetime):
            dt = d if d.tzinfo else d.replace(tzinfo=timezone.utc)
        else:
            dt = datetime(d.year, d.month, d.day, tzinfo=timezone.utc)
        secs = int((dt - _EPOCH).total_seconds())
        div = {'minute10': 600, 'minute30': 1800,
               'hour1': 3600, 'hour4': 14400, 'hour12': 43200}
        return secs // div[index]
    dd = d.date() if isinstance(d, datetime) else d
    if index == 'day1':
        if dd < _DAY_ONE:
            return 0
        return 1 + (dd - _DAY_ONE).days
    elif index == 'day3':
        return (dd - date(2008, 12, 31)).days // 3
    elif index == 'week1':
        return (dd - _GENESIS).days // 7
    elif index == 'month1':
        return (dd.year - 2009) * 12 + (dd.month - 1)
    elif index == 'month3':
        return (dd.year - 2009) * 4 + (dd.month - 1) // 3
    elif index == 'month6':
        return (dd.year - 2009) * 2 + (dd.month - 1) // 6
    elif index == 'year1':
        return dd.year - 2009
    elif index == 'year10':
        return (dd.year - 2009) // 10
    else:
        raise ValueError(f"{index} is not a date-based index")


@dataclass
class SeriesData(Generic[T]):
    """Series data with range information. Always int-indexed."""
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
        return list(zip(self.indexes(), self.data))

    def to_dict(self) -> Dict[int, T]:
        """Return {index: value} dict."""
        return dict(zip(self.indexes(), self.data))

    def __iter__(self) -> Iterator[Tuple[int, T]]:
        """Iterate over (index, value) pairs."""
        return iter(zip(self.indexes(), self.data))

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


class _EndpointConfig:
    """Shared endpoint configuration."""
    client: BrkClient
    name: str
    index: Index
    start: Optional[int]
    end: Optional[int]

    def __init__(self, client: BrkClient, name: str, index: Index,
                 start: Optional[int] = None, end: Optional[int] = None):
        self.client = client
        self.name = name
        self.index = index
        self.start = start
        self.end = end

    def path(self) -> str:
        return f"/api/series/{self.name}/{self.index}"

    def _build_path(self, format: Optional[str] = None) -> str:
        params = []
        if self.start is not None:
            params.append(f"start={self.start}")
        if self.end is not None:
            params.append(f"end={self.end}")
        if format is not None:
            params.append(f"format={format}")
        query = "&".join(params)
        p = self.path()
        return f"{p}?{query}" if query else p

    def _new(self, start: Optional[int] = None, end: Optional[int] = None) -> _EndpointConfig:
        return _EndpointConfig(self.client, self.name, self.index, start, end)

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
        start = self._config.start or 0
        return RangeBuilder(self._config._new(start, start + n))

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
        start = self._config.start or 0
        return DateRangeBuilder(self._config._new(start, start + n))
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

    def __init__(self, client: BrkClient, name: str, index: Index):
        self._config = _EndpointConfig(client, name, index)

    @overload
    def __getitem__(self, key: int) -> SingleItemBuilder[T]: ...
    @overload
    def __getitem__(self, key: slice) -> RangeBuilder[T]: ...

    def __getitem__(self, key: Union[int, slice]) -> Union[SingleItemBuilder[T], RangeBuilder[T]]:
        """Access single item or slice by integer index."""
        if isinstance(key, int):
            return SingleItemBuilder(self._config._new(key, key + 1))
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

    def __init__(self, client: BrkClient, name: str, index: Index):
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
            return DateSingleItemBuilder(self._config._new(idx, idx + 1))
        if isinstance(key, int):
            return DateSingleItemBuilder(self._config._new(key, key + 1))
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
_i2 = ('minute10', 'minute30', 'hour1', 'hour4', 'hour12', 'day1', 'day3', 'week1', 'month1', 'month3', 'month6', 'year1', 'year10', 'halving', 'epoch')
_i3 = ('minute10',)
_i4 = ('minute30',)
_i5 = ('hour1',)
_i6 = ('hour4',)
_i7 = ('hour12',)
_i8 = ('day1',)
_i9 = ('day3',)
_i10 = ('week1',)
_i11 = ('month1',)
_i12 = ('month3',)
_i13 = ('month6',)
_i14 = ('year1',)
_i15 = ('year10',)
_i16 = ('halving',)
_i17 = ('epoch',)
_i18 = ('height',)
_i19 = ('tx_index',)
_i20 = ('txin_index',)
_i21 = ('txout_index',)
_i22 = ('empty_output_index',)
_i23 = ('op_return_index',)
_i24 = ('p2a_addr_index',)
_i25 = ('p2ms_output_index',)
_i26 = ('p2pk33_addr_index',)
_i27 = ('p2pk65_addr_index',)
_i28 = ('p2pkh_addr_index',)
_i29 = ('p2sh_addr_index',)
_i30 = ('p2tr_addr_index',)
_i31 = ('p2wpkh_addr_index',)
_i32 = ('p2wsh_addr_index',)
_i33 = ('unknown_output_index',)
_i34 = ('funded_addr_index',)
_i35 = ('empty_addr_index',)

def _ep(c: BrkClient, n: str, i: Index) -> SeriesEndpoint[Any]:
    return SeriesEndpoint(c, n, i)

def _dep(c: BrkClient, n: str, i: Index) -> DateSeriesEndpoint[Any]:
    return DateSeriesEndpoint(c, n, i)

# Index accessor classes

class _SeriesPattern1By(Generic[T]):
    def __init__(self, c: BrkClient, n: str): self._c, self._n = c, n
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
    def __init__(self, c: BrkClient, n: str): self._n, self.by = n, _SeriesPattern1By(c, n)
    @property
    def name(self) -> str: return self._n
    def indexes(self) -> List[str]: return list(_i1)
    def get(self, index: Index) -> Optional[SeriesEndpoint[T]]: return _ep(self.by._c, self._n, index) if index in _i1 else None

class _SeriesPattern2By(Generic[T]):
    def __init__(self, c: BrkClient, n: str): self._c, self._n = c, n
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

class SeriesPattern2(Generic[T]):
    by: _SeriesPattern2By[T]
    def __init__(self, c: BrkClient, n: str): self._n, self.by = n, _SeriesPattern2By(c, n)
    @property
    def name(self) -> str: return self._n
    def indexes(self) -> List[str]: return list(_i2)
    def get(self, index: Index) -> Optional[SeriesEndpoint[T]]: return _ep(self.by._c, self._n, index) if index in _i2 else None

class _SeriesPattern3By(Generic[T]):
    def __init__(self, c: BrkClient, n: str): self._c, self._n = c, n
    def minute10(self) -> DateSeriesEndpoint[T]: return _dep(self._c, self._n, 'minute10')

class SeriesPattern3(Generic[T]):
    by: _SeriesPattern3By[T]
    def __init__(self, c: BrkClient, n: str): self._n, self.by = n, _SeriesPattern3By(c, n)
    @property
    def name(self) -> str: return self._n
    def indexes(self) -> List[str]: return list(_i3)
    def get(self, index: Index) -> Optional[SeriesEndpoint[T]]: return _ep(self.by._c, self._n, index) if index in _i3 else None

class _SeriesPattern4By(Generic[T]):
    def __init__(self, c: BrkClient, n: str): self._c, self._n = c, n
    def minute30(self) -> DateSeriesEndpoint[T]: return _dep(self._c, self._n, 'minute30')

class SeriesPattern4(Generic[T]):
    by: _SeriesPattern4By[T]
    def __init__(self, c: BrkClient, n: str): self._n, self.by = n, _SeriesPattern4By(c, n)
    @property
    def name(self) -> str: return self._n
    def indexes(self) -> List[str]: return list(_i4)
    def get(self, index: Index) -> Optional[SeriesEndpoint[T]]: return _ep(self.by._c, self._n, index) if index in _i4 else None

class _SeriesPattern5By(Generic[T]):
    def __init__(self, c: BrkClient, n: str): self._c, self._n = c, n
    def hour1(self) -> DateSeriesEndpoint[T]: return _dep(self._c, self._n, 'hour1')

class SeriesPattern5(Generic[T]):
    by: _SeriesPattern5By[T]
    def __init__(self, c: BrkClient, n: str): self._n, self.by = n, _SeriesPattern5By(c, n)
    @property
    def name(self) -> str: return self._n
    def indexes(self) -> List[str]: return list(_i5)
    def get(self, index: Index) -> Optional[SeriesEndpoint[T]]: return _ep(self.by._c, self._n, index) if index in _i5 else None

class _SeriesPattern6By(Generic[T]):
    def __init__(self, c: BrkClient, n: str): self._c, self._n = c, n
    def hour4(self) -> DateSeriesEndpoint[T]: return _dep(self._c, self._n, 'hour4')

class SeriesPattern6(Generic[T]):
    by: _SeriesPattern6By[T]
    def __init__(self, c: BrkClient, n: str): self._n, self.by = n, _SeriesPattern6By(c, n)
    @property
    def name(self) -> str: return self._n
    def indexes(self) -> List[str]: return list(_i6)
    def get(self, index: Index) -> Optional[SeriesEndpoint[T]]: return _ep(self.by._c, self._n, index) if index in _i6 else None

class _SeriesPattern7By(Generic[T]):
    def __init__(self, c: BrkClient, n: str): self._c, self._n = c, n
    def hour12(self) -> DateSeriesEndpoint[T]: return _dep(self._c, self._n, 'hour12')

class SeriesPattern7(Generic[T]):
    by: _SeriesPattern7By[T]
    def __init__(self, c: BrkClient, n: str): self._n, self.by = n, _SeriesPattern7By(c, n)
    @property
    def name(self) -> str: return self._n
    def indexes(self) -> List[str]: return list(_i7)
    def get(self, index: Index) -> Optional[SeriesEndpoint[T]]: return _ep(self.by._c, self._n, index) if index in _i7 else None

class _SeriesPattern8By(Generic[T]):
    def __init__(self, c: BrkClient, n: str): self._c, self._n = c, n
    def day1(self) -> DateSeriesEndpoint[T]: return _dep(self._c, self._n, 'day1')

class SeriesPattern8(Generic[T]):
    by: _SeriesPattern8By[T]
    def __init__(self, c: BrkClient, n: str): self._n, self.by = n, _SeriesPattern8By(c, n)
    @property
    def name(self) -> str: return self._n
    def indexes(self) -> List[str]: return list(_i8)
    def get(self, index: Index) -> Optional[SeriesEndpoint[T]]: return _ep(self.by._c, self._n, index) if index in _i8 else None

class _SeriesPattern9By(Generic[T]):
    def __init__(self, c: BrkClient, n: str): self._c, self._n = c, n
    def day3(self) -> DateSeriesEndpoint[T]: return _dep(self._c, self._n, 'day3')

class SeriesPattern9(Generic[T]):
    by: _SeriesPattern9By[T]
    def __init__(self, c: BrkClient, n: str): self._n, self.by = n, _SeriesPattern9By(c, n)
    @property
    def name(self) -> str: return self._n
    def indexes(self) -> List[str]: return list(_i9)
    def get(self, index: Index) -> Optional[SeriesEndpoint[T]]: return _ep(self.by._c, self._n, index) if index in _i9 else None

class _SeriesPattern10By(Generic[T]):
    def __init__(self, c: BrkClient, n: str): self._c, self._n = c, n
    def week1(self) -> DateSeriesEndpoint[T]: return _dep(self._c, self._n, 'week1')

class SeriesPattern10(Generic[T]):
    by: _SeriesPattern10By[T]
    def __init__(self, c: BrkClient, n: str): self._n, self.by = n, _SeriesPattern10By(c, n)
    @property
    def name(self) -> str: return self._n
    def indexes(self) -> List[str]: return list(_i10)
    def get(self, index: Index) -> Optional[SeriesEndpoint[T]]: return _ep(self.by._c, self._n, index) if index in _i10 else None

class _SeriesPattern11By(Generic[T]):
    def __init__(self, c: BrkClient, n: str): self._c, self._n = c, n
    def month1(self) -> DateSeriesEndpoint[T]: return _dep(self._c, self._n, 'month1')

class SeriesPattern11(Generic[T]):
    by: _SeriesPattern11By[T]
    def __init__(self, c: BrkClient, n: str): self._n, self.by = n, _SeriesPattern11By(c, n)
    @property
    def name(self) -> str: return self._n
    def indexes(self) -> List[str]: return list(_i11)
    def get(self, index: Index) -> Optional[SeriesEndpoint[T]]: return _ep(self.by._c, self._n, index) if index in _i11 else None

class _SeriesPattern12By(Generic[T]):
    def __init__(self, c: BrkClient, n: str): self._c, self._n = c, n
    def month3(self) -> DateSeriesEndpoint[T]: return _dep(self._c, self._n, 'month3')

class SeriesPattern12(Generic[T]):
    by: _SeriesPattern12By[T]
    def __init__(self, c: BrkClient, n: str): self._n, self.by = n, _SeriesPattern12By(c, n)
    @property
    def name(self) -> str: return self._n
    def indexes(self) -> List[str]: return list(_i12)
    def get(self, index: Index) -> Optional[SeriesEndpoint[T]]: return _ep(self.by._c, self._n, index) if index in _i12 else None

class _SeriesPattern13By(Generic[T]):
    def __init__(self, c: BrkClient, n: str): self._c, self._n = c, n
    def month6(self) -> DateSeriesEndpoint[T]: return _dep(self._c, self._n, 'month6')

class SeriesPattern13(Generic[T]):
    by: _SeriesPattern13By[T]
    def __init__(self, c: BrkClient, n: str): self._n, self.by = n, _SeriesPattern13By(c, n)
    @property
    def name(self) -> str: return self._n
    def indexes(self) -> List[str]: return list(_i13)
    def get(self, index: Index) -> Optional[SeriesEndpoint[T]]: return _ep(self.by._c, self._n, index) if index in _i13 else None

class _SeriesPattern14By(Generic[T]):
    def __init__(self, c: BrkClient, n: str): self._c, self._n = c, n
    def year1(self) -> DateSeriesEndpoint[T]: return _dep(self._c, self._n, 'year1')

class SeriesPattern14(Generic[T]):
    by: _SeriesPattern14By[T]
    def __init__(self, c: BrkClient, n: str): self._n, self.by = n, _SeriesPattern14By(c, n)
    @property
    def name(self) -> str: return self._n
    def indexes(self) -> List[str]: return list(_i14)
    def get(self, index: Index) -> Optional[SeriesEndpoint[T]]: return _ep(self.by._c, self._n, index) if index in _i14 else None

class _SeriesPattern15By(Generic[T]):
    def __init__(self, c: BrkClient, n: str): self._c, self._n = c, n
    def year10(self) -> DateSeriesEndpoint[T]: return _dep(self._c, self._n, 'year10')

class SeriesPattern15(Generic[T]):
    by: _SeriesPattern15By[T]
    def __init__(self, c: BrkClient, n: str): self._n, self.by = n, _SeriesPattern15By(c, n)
    @property
    def name(self) -> str: return self._n
    def indexes(self) -> List[str]: return list(_i15)
    def get(self, index: Index) -> Optional[SeriesEndpoint[T]]: return _ep(self.by._c, self._n, index) if index in _i15 else None

class _SeriesPattern16By(Generic[T]):
    def __init__(self, c: BrkClient, n: str): self._c, self._n = c, n
    def halving(self) -> SeriesEndpoint[T]: return _ep(self._c, self._n, 'halving')

class SeriesPattern16(Generic[T]):
    by: _SeriesPattern16By[T]
    def __init__(self, c: BrkClient, n: str): self._n, self.by = n, _SeriesPattern16By(c, n)
    @property
    def name(self) -> str: return self._n
    def indexes(self) -> List[str]: return list(_i16)
    def get(self, index: Index) -> Optional[SeriesEndpoint[T]]: return _ep(self.by._c, self._n, index) if index in _i16 else None

class _SeriesPattern17By(Generic[T]):
    def __init__(self, c: BrkClient, n: str): self._c, self._n = c, n
    def epoch(self) -> SeriesEndpoint[T]: return _ep(self._c, self._n, 'epoch')

class SeriesPattern17(Generic[T]):
    by: _SeriesPattern17By[T]
    def __init__(self, c: BrkClient, n: str): self._n, self.by = n, _SeriesPattern17By(c, n)
    @property
    def name(self) -> str: return self._n
    def indexes(self) -> List[str]: return list(_i17)
    def get(self, index: Index) -> Optional[SeriesEndpoint[T]]: return _ep(self.by._c, self._n, index) if index in _i17 else None

class _SeriesPattern18By(Generic[T]):
    def __init__(self, c: BrkClient, n: str): self._c, self._n = c, n
    def height(self) -> SeriesEndpoint[T]: return _ep(self._c, self._n, 'height')

class SeriesPattern18(Generic[T]):
    by: _SeriesPattern18By[T]
    def __init__(self, c: BrkClient, n: str): self._n, self.by = n, _SeriesPattern18By(c, n)
    @property
    def name(self) -> str: return self._n
    def indexes(self) -> List[str]: return list(_i18)
    def get(self, index: Index) -> Optional[SeriesEndpoint[T]]: return _ep(self.by._c, self._n, index) if index in _i18 else None

class _SeriesPattern19By(Generic[T]):
    def __init__(self, c: BrkClient, n: str): self._c, self._n = c, n
    def tx_index(self) -> SeriesEndpoint[T]: return _ep(self._c, self._n, 'tx_index')

class SeriesPattern19(Generic[T]):
    by: _SeriesPattern19By[T]
    def __init__(self, c: BrkClient, n: str): self._n, self.by = n, _SeriesPattern19By(c, n)
    @property
    def name(self) -> str: return self._n
    def indexes(self) -> List[str]: return list(_i19)
    def get(self, index: Index) -> Optional[SeriesEndpoint[T]]: return _ep(self.by._c, self._n, index) if index in _i19 else None

class _SeriesPattern20By(Generic[T]):
    def __init__(self, c: BrkClient, n: str): self._c, self._n = c, n
    def txin_index(self) -> SeriesEndpoint[T]: return _ep(self._c, self._n, 'txin_index')

class SeriesPattern20(Generic[T]):
    by: _SeriesPattern20By[T]
    def __init__(self, c: BrkClient, n: str): self._n, self.by = n, _SeriesPattern20By(c, n)
    @property
    def name(self) -> str: return self._n
    def indexes(self) -> List[str]: return list(_i20)
    def get(self, index: Index) -> Optional[SeriesEndpoint[T]]: return _ep(self.by._c, self._n, index) if index in _i20 else None

class _SeriesPattern21By(Generic[T]):
    def __init__(self, c: BrkClient, n: str): self._c, self._n = c, n
    def txout_index(self) -> SeriesEndpoint[T]: return _ep(self._c, self._n, 'txout_index')

class SeriesPattern21(Generic[T]):
    by: _SeriesPattern21By[T]
    def __init__(self, c: BrkClient, n: str): self._n, self.by = n, _SeriesPattern21By(c, n)
    @property
    def name(self) -> str: return self._n
    def indexes(self) -> List[str]: return list(_i21)
    def get(self, index: Index) -> Optional[SeriesEndpoint[T]]: return _ep(self.by._c, self._n, index) if index in _i21 else None

class _SeriesPattern22By(Generic[T]):
    def __init__(self, c: BrkClient, n: str): self._c, self._n = c, n
    def empty_output_index(self) -> SeriesEndpoint[T]: return _ep(self._c, self._n, 'empty_output_index')

class SeriesPattern22(Generic[T]):
    by: _SeriesPattern22By[T]
    def __init__(self, c: BrkClient, n: str): self._n, self.by = n, _SeriesPattern22By(c, n)
    @property
    def name(self) -> str: return self._n
    def indexes(self) -> List[str]: return list(_i22)
    def get(self, index: Index) -> Optional[SeriesEndpoint[T]]: return _ep(self.by._c, self._n, index) if index in _i22 else None

class _SeriesPattern23By(Generic[T]):
    def __init__(self, c: BrkClient, n: str): self._c, self._n = c, n
    def op_return_index(self) -> SeriesEndpoint[T]: return _ep(self._c, self._n, 'op_return_index')

class SeriesPattern23(Generic[T]):
    by: _SeriesPattern23By[T]
    def __init__(self, c: BrkClient, n: str): self._n, self.by = n, _SeriesPattern23By(c, n)
    @property
    def name(self) -> str: return self._n
    def indexes(self) -> List[str]: return list(_i23)
    def get(self, index: Index) -> Optional[SeriesEndpoint[T]]: return _ep(self.by._c, self._n, index) if index in _i23 else None

class _SeriesPattern24By(Generic[T]):
    def __init__(self, c: BrkClient, n: str): self._c, self._n = c, n
    def p2a_addr_index(self) -> SeriesEndpoint[T]: return _ep(self._c, self._n, 'p2a_addr_index')

class SeriesPattern24(Generic[T]):
    by: _SeriesPattern24By[T]
    def __init__(self, c: BrkClient, n: str): self._n, self.by = n, _SeriesPattern24By(c, n)
    @property
    def name(self) -> str: return self._n
    def indexes(self) -> List[str]: return list(_i24)
    def get(self, index: Index) -> Optional[SeriesEndpoint[T]]: return _ep(self.by._c, self._n, index) if index in _i24 else None

class _SeriesPattern25By(Generic[T]):
    def __init__(self, c: BrkClient, n: str): self._c, self._n = c, n
    def p2ms_output_index(self) -> SeriesEndpoint[T]: return _ep(self._c, self._n, 'p2ms_output_index')

class SeriesPattern25(Generic[T]):
    by: _SeriesPattern25By[T]
    def __init__(self, c: BrkClient, n: str): self._n, self.by = n, _SeriesPattern25By(c, n)
    @property
    def name(self) -> str: return self._n
    def indexes(self) -> List[str]: return list(_i25)
    def get(self, index: Index) -> Optional[SeriesEndpoint[T]]: return _ep(self.by._c, self._n, index) if index in _i25 else None

class _SeriesPattern26By(Generic[T]):
    def __init__(self, c: BrkClient, n: str): self._c, self._n = c, n
    def p2pk33_addr_index(self) -> SeriesEndpoint[T]: return _ep(self._c, self._n, 'p2pk33_addr_index')

class SeriesPattern26(Generic[T]):
    by: _SeriesPattern26By[T]
    def __init__(self, c: BrkClient, n: str): self._n, self.by = n, _SeriesPattern26By(c, n)
    @property
    def name(self) -> str: return self._n
    def indexes(self) -> List[str]: return list(_i26)
    def get(self, index: Index) -> Optional[SeriesEndpoint[T]]: return _ep(self.by._c, self._n, index) if index in _i26 else None

class _SeriesPattern27By(Generic[T]):
    def __init__(self, c: BrkClient, n: str): self._c, self._n = c, n
    def p2pk65_addr_index(self) -> SeriesEndpoint[T]: return _ep(self._c, self._n, 'p2pk65_addr_index')

class SeriesPattern27(Generic[T]):
    by: _SeriesPattern27By[T]
    def __init__(self, c: BrkClient, n: str): self._n, self.by = n, _SeriesPattern27By(c, n)
    @property
    def name(self) -> str: return self._n
    def indexes(self) -> List[str]: return list(_i27)
    def get(self, index: Index) -> Optional[SeriesEndpoint[T]]: return _ep(self.by._c, self._n, index) if index in _i27 else None

class _SeriesPattern28By(Generic[T]):
    def __init__(self, c: BrkClient, n: str): self._c, self._n = c, n
    def p2pkh_addr_index(self) -> SeriesEndpoint[T]: return _ep(self._c, self._n, 'p2pkh_addr_index')

class SeriesPattern28(Generic[T]):
    by: _SeriesPattern28By[T]
    def __init__(self, c: BrkClient, n: str): self._n, self.by = n, _SeriesPattern28By(c, n)
    @property
    def name(self) -> str: return self._n
    def indexes(self) -> List[str]: return list(_i28)
    def get(self, index: Index) -> Optional[SeriesEndpoint[T]]: return _ep(self.by._c, self._n, index) if index in _i28 else None

class _SeriesPattern29By(Generic[T]):
    def __init__(self, c: BrkClient, n: str): self._c, self._n = c, n
    def p2sh_addr_index(self) -> SeriesEndpoint[T]: return _ep(self._c, self._n, 'p2sh_addr_index')

class SeriesPattern29(Generic[T]):
    by: _SeriesPattern29By[T]
    def __init__(self, c: BrkClient, n: str): self._n, self.by = n, _SeriesPattern29By(c, n)
    @property
    def name(self) -> str: return self._n
    def indexes(self) -> List[str]: return list(_i29)
    def get(self, index: Index) -> Optional[SeriesEndpoint[T]]: return _ep(self.by._c, self._n, index) if index in _i29 else None

class _SeriesPattern30By(Generic[T]):
    def __init__(self, c: BrkClient, n: str): self._c, self._n = c, n
    def p2tr_addr_index(self) -> SeriesEndpoint[T]: return _ep(self._c, self._n, 'p2tr_addr_index')

class SeriesPattern30(Generic[T]):
    by: _SeriesPattern30By[T]
    def __init__(self, c: BrkClient, n: str): self._n, self.by = n, _SeriesPattern30By(c, n)
    @property
    def name(self) -> str: return self._n
    def indexes(self) -> List[str]: return list(_i30)
    def get(self, index: Index) -> Optional[SeriesEndpoint[T]]: return _ep(self.by._c, self._n, index) if index in _i30 else None

class _SeriesPattern31By(Generic[T]):
    def __init__(self, c: BrkClient, n: str): self._c, self._n = c, n
    def p2wpkh_addr_index(self) -> SeriesEndpoint[T]: return _ep(self._c, self._n, 'p2wpkh_addr_index')

class SeriesPattern31(Generic[T]):
    by: _SeriesPattern31By[T]
    def __init__(self, c: BrkClient, n: str): self._n, self.by = n, _SeriesPattern31By(c, n)
    @property
    def name(self) -> str: return self._n
    def indexes(self) -> List[str]: return list(_i31)
    def get(self, index: Index) -> Optional[SeriesEndpoint[T]]: return _ep(self.by._c, self._n, index) if index in _i31 else None

class _SeriesPattern32By(Generic[T]):
    def __init__(self, c: BrkClient, n: str): self._c, self._n = c, n
    def p2wsh_addr_index(self) -> SeriesEndpoint[T]: return _ep(self._c, self._n, 'p2wsh_addr_index')

class SeriesPattern32(Generic[T]):
    by: _SeriesPattern32By[T]
    def __init__(self, c: BrkClient, n: str): self._n, self.by = n, _SeriesPattern32By(c, n)
    @property
    def name(self) -> str: return self._n
    def indexes(self) -> List[str]: return list(_i32)
    def get(self, index: Index) -> Optional[SeriesEndpoint[T]]: return _ep(self.by._c, self._n, index) if index in _i32 else None

class _SeriesPattern33By(Generic[T]):
    def __init__(self, c: BrkClient, n: str): self._c, self._n = c, n
    def unknown_output_index(self) -> SeriesEndpoint[T]: return _ep(self._c, self._n, 'unknown_output_index')

class SeriesPattern33(Generic[T]):
    by: _SeriesPattern33By[T]
    def __init__(self, c: BrkClient, n: str): self._n, self.by = n, _SeriesPattern33By(c, n)
    @property
    def name(self) -> str: return self._n
    def indexes(self) -> List[str]: return list(_i33)
    def get(self, index: Index) -> Optional[SeriesEndpoint[T]]: return _ep(self.by._c, self._n, index) if index in _i33 else None

class _SeriesPattern34By(Generic[T]):
    def __init__(self, c: BrkClient, n: str): self._c, self._n = c, n
    def funded_addr_index(self) -> SeriesEndpoint[T]: return _ep(self._c, self._n, 'funded_addr_index')

class SeriesPattern34(Generic[T]):
    by: _SeriesPattern34By[T]
    def __init__(self, c: BrkClient, n: str): self._n, self.by = n, _SeriesPattern34By(c, n)
    @property
    def name(self) -> str: return self._n
    def indexes(self) -> List[str]: return list(_i34)
    def get(self, index: Index) -> Optional[SeriesEndpoint[T]]: return _ep(self.by._c, self._n, index) if index in _i34 else None

class _SeriesPattern35By(Generic[T]):
    def __init__(self, c: BrkClient, n: str): self._c, self._n = c, n
    def empty_addr_index(self) -> SeriesEndpoint[T]: return _ep(self._c, self._n, 'empty_addr_index')

class SeriesPattern35(Generic[T]):
    by: _SeriesPattern35By[T]
    def __init__(self, c: BrkClient, n: str): self._n, self.by = n, _SeriesPattern35By(c, n)
    @property
    def name(self) -> str: return self._n
    def indexes(self) -> List[str]: return list(_i35)
    def get(self, index: Index) -> Optional[SeriesEndpoint[T]]: return _ep(self.by._c, self._n, index) if index in _i35 else None

# Reusable structural pattern classes

class Pct05Pct10Pct15Pct20Pct25Pct30Pct35Pct40Pct45Pct50Pct55Pct60Pct65Pct70Pct75Pct80Pct85Pct90Pct95Pattern:
    """Pattern struct for repeated tree structure."""
    
    def __init__(self, client: BrkClient, acc: str):
        """Create pattern node with accumulated series name."""
        self.pct05: CentsSatsUsdPattern = CentsSatsUsdPattern(client, _m(acc, 'pct05'))
        self.pct10: CentsSatsUsdPattern = CentsSatsUsdPattern(client, _m(acc, 'pct10'))
        self.pct15: CentsSatsUsdPattern = CentsSatsUsdPattern(client, _m(acc, 'pct15'))
        self.pct20: CentsSatsUsdPattern = CentsSatsUsdPattern(client, _m(acc, 'pct20'))
        self.pct25: CentsSatsUsdPattern = CentsSatsUsdPattern(client, _m(acc, 'pct25'))
        self.pct30: CentsSatsUsdPattern = CentsSatsUsdPattern(client, _m(acc, 'pct30'))
        self.pct35: CentsSatsUsdPattern = CentsSatsUsdPattern(client, _m(acc, 'pct35'))
        self.pct40: CentsSatsUsdPattern = CentsSatsUsdPattern(client, _m(acc, 'pct40'))
        self.pct45: CentsSatsUsdPattern = CentsSatsUsdPattern(client, _m(acc, 'pct45'))
        self.pct50: CentsSatsUsdPattern = CentsSatsUsdPattern(client, _m(acc, 'pct50'))
        self.pct55: CentsSatsUsdPattern = CentsSatsUsdPattern(client, _m(acc, 'pct55'))
        self.pct60: CentsSatsUsdPattern = CentsSatsUsdPattern(client, _m(acc, 'pct60'))
        self.pct65: CentsSatsUsdPattern = CentsSatsUsdPattern(client, _m(acc, 'pct65'))
        self.pct70: CentsSatsUsdPattern = CentsSatsUsdPattern(client, _m(acc, 'pct70'))
        self.pct75: CentsSatsUsdPattern = CentsSatsUsdPattern(client, _m(acc, 'pct75'))
        self.pct80: CentsSatsUsdPattern = CentsSatsUsdPattern(client, _m(acc, 'pct80'))
        self.pct85: CentsSatsUsdPattern = CentsSatsUsdPattern(client, _m(acc, 'pct85'))
        self.pct90: CentsSatsUsdPattern = CentsSatsUsdPattern(client, _m(acc, 'pct90'))
        self.pct95: CentsSatsUsdPattern = CentsSatsUsdPattern(client, _m(acc, 'pct95'))

class _0sdM0M1M1sdM2M2sdM3sdP0P1P1sdP2P2sdP3sdSdZscorePattern:
    """Pattern struct for repeated tree structure."""
    pass

class AllEmptyOpP2aP2msP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshUnknownPattern:
    """Pattern struct for repeated tree structure."""
    
    def __init__(self, client: BrkClient, acc: str):
        """Create pattern node with accumulated series name."""
        self.all: AverageBlockCumulativeSumPattern[StoredU64] = AverageBlockCumulativeSumPattern(client, _m(acc, 'bis'))
        self.empty: AverageBlockCumulativeSumPattern[StoredU64] = AverageBlockCumulativeSumPattern(client, _m(acc, 'with_empty_outputs_output'))
        self.op_return: AverageBlockCumulativeSumPattern[StoredU64] = AverageBlockCumulativeSumPattern(client, _m(acc, 'with_op_return_output'))
        self.p2a: AverageBlockCumulativeSumPattern[StoredU64] = AverageBlockCumulativeSumPattern(client, _m(acc, 'with_p2a_output'))
        self.p2ms: AverageBlockCumulativeSumPattern[StoredU64] = AverageBlockCumulativeSumPattern(client, _m(acc, 'with_p2ms_output'))
        self.p2pk33: AverageBlockCumulativeSumPattern[StoredU64] = AverageBlockCumulativeSumPattern(client, _m(acc, 'with_p2pk33_output'))
        self.p2pk65: AverageBlockCumulativeSumPattern[StoredU64] = AverageBlockCumulativeSumPattern(client, _m(acc, 'with_p2pk65_output'))
        self.p2pkh: AverageBlockCumulativeSumPattern[StoredU64] = AverageBlockCumulativeSumPattern(client, _m(acc, 'with_p2pkh_output'))
        self.p2sh: AverageBlockCumulativeSumPattern[StoredU64] = AverageBlockCumulativeSumPattern(client, _m(acc, 'with_p2sh_output'))
        self.p2tr: AverageBlockCumulativeSumPattern[StoredU64] = AverageBlockCumulativeSumPattern(client, _m(acc, 'with_p2tr_output'))
        self.p2wpkh: AverageBlockCumulativeSumPattern[StoredU64] = AverageBlockCumulativeSumPattern(client, _m(acc, 'with_p2wpkh_output'))
        self.p2wsh: AverageBlockCumulativeSumPattern[StoredU64] = AverageBlockCumulativeSumPattern(client, _m(acc, 'with_p2wsh_output'))
        self.unknown: AverageBlockCumulativeSumPattern[StoredU64] = AverageBlockCumulativeSumPattern(client, _m(acc, 'with_unknown_outputs_output'))

class _10y1m1w1y2y3m3y4y5y6m6y8yPattern2:
    """Pattern struct for repeated tree structure."""
    
    def __init__(self, client: BrkClient, acc: str):
        """Create pattern node with accumulated series name."""
        self._10y: BpsPercentRatioPattern = BpsPercentRatioPattern(client, _m(acc, '10y'))
        self._1m: BpsPercentRatioPattern = BpsPercentRatioPattern(client, _m(acc, '1m'))
        self._1w: BpsPercentRatioPattern = BpsPercentRatioPattern(client, _m(acc, '1w'))
        self._1y: BpsPercentRatioPattern = BpsPercentRatioPattern(client, _m(acc, '1y'))
        self._2y: BpsPercentRatioPattern = BpsPercentRatioPattern(client, _m(acc, '2y'))
        self._3m: BpsPercentRatioPattern = BpsPercentRatioPattern(client, _m(acc, '3m'))
        self._3y: BpsPercentRatioPattern = BpsPercentRatioPattern(client, _m(acc, '3y'))
        self._4y: BpsPercentRatioPattern = BpsPercentRatioPattern(client, _m(acc, '4y'))
        self._5y: BpsPercentRatioPattern = BpsPercentRatioPattern(client, _m(acc, '5y'))
        self._6m: BpsPercentRatioPattern = BpsPercentRatioPattern(client, _m(acc, '6m'))
        self._6y: BpsPercentRatioPattern = BpsPercentRatioPattern(client, _m(acc, '6y'))
        self._8y: BpsPercentRatioPattern = BpsPercentRatioPattern(client, _m(acc, '8y'))

class _10y1m1w1y2y3m3y4y5y6m6y8yPattern3:
    """Pattern struct for repeated tree structure."""
    
    def __init__(self, client: BrkClient, acc: str):
        """Create pattern node with accumulated series name."""
        self._10y: BtcCentsSatsUsdPattern = BtcCentsSatsUsdPattern(client, _m(acc, '10y'))
        self._1m: BtcCentsSatsUsdPattern = BtcCentsSatsUsdPattern(client, _m(acc, '1m'))
        self._1w: BtcCentsSatsUsdPattern = BtcCentsSatsUsdPattern(client, _m(acc, '1w'))
        self._1y: BtcCentsSatsUsdPattern = BtcCentsSatsUsdPattern(client, _m(acc, '1y'))
        self._2y: BtcCentsSatsUsdPattern = BtcCentsSatsUsdPattern(client, _m(acc, '2y'))
        self._3m: BtcCentsSatsUsdPattern = BtcCentsSatsUsdPattern(client, _m(acc, '3m'))
        self._3y: BtcCentsSatsUsdPattern = BtcCentsSatsUsdPattern(client, _m(acc, '3y'))
        self._4y: BtcCentsSatsUsdPattern = BtcCentsSatsUsdPattern(client, _m(acc, '4y'))
        self._5y: BtcCentsSatsUsdPattern = BtcCentsSatsUsdPattern(client, _m(acc, '5y'))
        self._6m: BtcCentsSatsUsdPattern = BtcCentsSatsUsdPattern(client, _m(acc, '6m'))
        self._6y: BtcCentsSatsUsdPattern = BtcCentsSatsUsdPattern(client, _m(acc, '6y'))
        self._8y: BtcCentsSatsUsdPattern = BtcCentsSatsUsdPattern(client, _m(acc, '8y'))

class AllEmptyP2aP2msP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshUnknownPattern:
    """Pattern struct for repeated tree structure."""
    pass

class CapCapitalizedGrossLossMvrvNetPeakPriceProfitSellSoprPattern:
    """Pattern struct for repeated tree structure."""
    pass

class CapCapitalizedGrossLossMvrvNetPeakPriceProfitSellSoprPattern2:
    """Pattern struct for repeated tree structure."""
    pass

class EmptyOpP2aP2msP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshUnknownPattern2:
    """Pattern struct for repeated tree structure."""
    
    def __init__(self, client: BrkClient, acc: str):
        """Create pattern node with accumulated series name."""
        self.empty: _1m1w1y24hBpsPercentRatioPattern = _1m1w1y24hBpsPercentRatioPattern(client, _m(acc, 'empty_outputs_output'))
        self.op_return: _1m1w1y24hBpsPercentRatioPattern = _1m1w1y24hBpsPercentRatioPattern(client, _m(acc, 'op_return_output'))
        self.p2a: _1m1w1y24hBpsPercentRatioPattern = _1m1w1y24hBpsPercentRatioPattern(client, _m(acc, 'p2a_output'))
        self.p2ms: _1m1w1y24hBpsPercentRatioPattern = _1m1w1y24hBpsPercentRatioPattern(client, _m(acc, 'p2ms_output'))
        self.p2pk33: _1m1w1y24hBpsPercentRatioPattern = _1m1w1y24hBpsPercentRatioPattern(client, _m(acc, 'p2pk33_output'))
        self.p2pk65: _1m1w1y24hBpsPercentRatioPattern = _1m1w1y24hBpsPercentRatioPattern(client, _m(acc, 'p2pk65_output'))
        self.p2pkh: _1m1w1y24hBpsPercentRatioPattern = _1m1w1y24hBpsPercentRatioPattern(client, _m(acc, 'p2pkh_output'))
        self.p2sh: _1m1w1y24hBpsPercentRatioPattern = _1m1w1y24hBpsPercentRatioPattern(client, _m(acc, 'p2sh_output'))
        self.p2tr: _1m1w1y24hBpsPercentRatioPattern = _1m1w1y24hBpsPercentRatioPattern(client, _m(acc, 'p2tr_output'))
        self.p2wpkh: _1m1w1y24hBpsPercentRatioPattern = _1m1w1y24hBpsPercentRatioPattern(client, _m(acc, 'p2wpkh_output'))
        self.p2wsh: _1m1w1y24hBpsPercentRatioPattern = _1m1w1y24hBpsPercentRatioPattern(client, _m(acc, 'p2wsh_output'))
        self.unknown: _1m1w1y24hBpsPercentRatioPattern = _1m1w1y24hBpsPercentRatioPattern(client, _m(acc, 'unknown_outputs_output'))

class AverageBlockCumulativeMaxMedianMinPct10Pct25Pct75Pct90SumPattern:
    """Pattern struct for repeated tree structure."""
    
    def __init__(self, client: BrkClient, acc: str):
        """Create pattern node with accumulated series name."""
        self.average: _1m1w1y24hPattern[StoredF32] = _1m1w1y24hPattern(client, _m(acc, 'average'))
        self.block: SeriesPattern18[StoredU64] = SeriesPattern18(client, acc)
        self.cumulative: SeriesPattern1[StoredU64] = SeriesPattern1(client, _m(acc, 'cumulative'))
        self.max: _1m1w1y24hPattern[StoredU64] = _1m1w1y24hPattern(client, _m(acc, 'max'))
        self.median: _1m1w1y24hPattern[StoredU64] = _1m1w1y24hPattern(client, _m(acc, 'median'))
        self.min: _1m1w1y24hPattern[StoredU64] = _1m1w1y24hPattern(client, _m(acc, 'min'))
        self.pct10: _1m1w1y24hPattern[StoredU64] = _1m1w1y24hPattern(client, _m(acc, 'pct10'))
        self.pct25: _1m1w1y24hPattern[StoredU64] = _1m1w1y24hPattern(client, _m(acc, 'pct25'))
        self.pct75: _1m1w1y24hPattern[StoredU64] = _1m1w1y24hPattern(client, _m(acc, 'pct75'))
        self.pct90: _1m1w1y24hPattern[StoredU64] = _1m1w1y24hPattern(client, _m(acc, 'pct90'))
        self.sum: _1m1w1y24hPattern[StoredU64] = _1m1w1y24hPattern(client, _m(acc, 'sum'))

class EmptyP2aP2msP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshUnknownPattern2:
    """Pattern struct for repeated tree structure."""
    
    def __init__(self, client: BrkClient, acc: str):
        """Create pattern node with accumulated series name."""
        self.empty: _1m1w1y24hBpsPercentRatioPattern = _1m1w1y24hBpsPercentRatioPattern(client, _m(acc, 'empty_outputs_prevout'))
        self.p2a: _1m1w1y24hBpsPercentRatioPattern = _1m1w1y24hBpsPercentRatioPattern(client, _m(acc, 'p2a_prevout'))
        self.p2ms: _1m1w1y24hBpsPercentRatioPattern = _1m1w1y24hBpsPercentRatioPattern(client, _m(acc, 'p2ms_prevout'))
        self.p2pk33: _1m1w1y24hBpsPercentRatioPattern = _1m1w1y24hBpsPercentRatioPattern(client, _m(acc, 'p2pk33_prevout'))
        self.p2pk65: _1m1w1y24hBpsPercentRatioPattern = _1m1w1y24hBpsPercentRatioPattern(client, _m(acc, 'p2pk65_prevout'))
        self.p2pkh: _1m1w1y24hBpsPercentRatioPattern = _1m1w1y24hBpsPercentRatioPattern(client, _m(acc, 'p2pkh_prevout'))
        self.p2sh: _1m1w1y24hBpsPercentRatioPattern = _1m1w1y24hBpsPercentRatioPattern(client, _m(acc, 'p2sh_prevout'))
        self.p2tr: _1m1w1y24hBpsPercentRatioPattern = _1m1w1y24hBpsPercentRatioPattern(client, _m(acc, 'p2tr_prevout'))
        self.p2wpkh: _1m1w1y24hBpsPercentRatioPattern = _1m1w1y24hBpsPercentRatioPattern(client, _m(acc, 'p2wpkh_prevout'))
        self.p2wsh: _1m1w1y24hBpsPercentRatioPattern = _1m1w1y24hBpsPercentRatioPattern(client, _m(acc, 'p2wsh_prevout'))
        self.unknown: _1m1w1y24hBpsPercentRatioPattern = _1m1w1y24hBpsPercentRatioPattern(client, _m(acc, 'unknown_outputs_prevout'))

class AverageBaseCumulativeMaxMedianMinPct10Pct25Pct75Pct90SumPattern(Generic[T]):
    """Pattern struct for repeated tree structure."""
    
    def __init__(self, client: BrkClient, acc: str):
        """Create pattern node with accumulated series name."""
        self.average: _1m1w1y24hPattern[T] = _1m1w1y24hPattern(client, _m(acc, 'average'))
        self.base: SeriesPattern18[T] = SeriesPattern18(client, acc)
        self.cumulative: SeriesPattern1[T] = SeriesPattern1(client, _m(acc, 'cumulative'))
        self.max: _1m1w1y24hPattern[T] = _1m1w1y24hPattern(client, _m(acc, 'max'))
        self.median: _1m1w1y24hPattern[T] = _1m1w1y24hPattern(client, _m(acc, 'median'))
        self.min: _1m1w1y24hPattern[T] = _1m1w1y24hPattern(client, _m(acc, 'min'))
        self.pct10: _1m1w1y24hPattern[T] = _1m1w1y24hPattern(client, _m(acc, 'pct10'))
        self.pct25: _1m1w1y24hPattern[T] = _1m1w1y24hPattern(client, _m(acc, 'pct25'))
        self.pct75: _1m1w1y24hPattern[T] = _1m1w1y24hPattern(client, _m(acc, 'pct75'))
        self.pct90: _1m1w1y24hPattern[T] = _1m1w1y24hPattern(client, _m(acc, 'pct90'))
        self.sum: _1m1w1y24hPattern[T] = _1m1w1y24hPattern(client, _m(acc, 'sum'))

class AllP2aP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshSharePattern:
    """Pattern struct for repeated tree structure."""
    pass

class IndexPct0Pct1Pct2Pct5Pct95Pct98Pct99ScorePattern:
    """Pattern struct for repeated tree structure."""
    
    def __init__(self, client: BrkClient, acc: str):
        """Create pattern node with accumulated series name."""
        self.index: SeriesPattern1[StoredI8] = SeriesPattern1(client, _m(acc, 'index'))
        self.pct0_5: CentsSatsUsdPattern = CentsSatsUsdPattern(client, _m(acc, 'pct0_5'))
        self.pct1: CentsSatsUsdPattern = CentsSatsUsdPattern(client, _m(acc, 'pct01'))
        self.pct2: CentsSatsUsdPattern = CentsSatsUsdPattern(client, _m(acc, 'pct02'))
        self.pct5: CentsSatsUsdPattern = CentsSatsUsdPattern(client, _m(acc, 'pct05'))
        self.pct95: CentsSatsUsdPattern = CentsSatsUsdPattern(client, _m(acc, 'pct95'))
        self.pct98: CentsSatsUsdPattern = CentsSatsUsdPattern(client, _m(acc, 'pct98'))
        self.pct99: CentsSatsUsdPattern = CentsSatsUsdPattern(client, _m(acc, 'pct99'))
        self.pct99_5: CentsSatsUsdPattern = CentsSatsUsdPattern(client, _m(acc, 'pct99_5'))
        self.score: SeriesPattern1[StoredI8] = SeriesPattern1(client, _m(acc, 'score'))

class AllP2aP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshPattern6:
    """Pattern struct for repeated tree structure."""
    
    def __init__(self, client: BrkClient, acc: str):
        """Create pattern node with accumulated series name."""
        self.all: AverageBlockCumulativeSumPattern[StoredU64] = AverageBlockCumulativeSumPattern(client, acc)
        self.p2a: AverageBlockCumulativeSumPattern[StoredU64] = AverageBlockCumulativeSumPattern(client, _p('p2a', acc))
        self.p2pk33: AverageBlockCumulativeSumPattern[StoredU64] = AverageBlockCumulativeSumPattern(client, _p('p2pk33', acc))
        self.p2pk65: AverageBlockCumulativeSumPattern[StoredU64] = AverageBlockCumulativeSumPattern(client, _p('p2pk65', acc))
        self.p2pkh: AverageBlockCumulativeSumPattern[StoredU64] = AverageBlockCumulativeSumPattern(client, _p('p2pkh', acc))
        self.p2sh: AverageBlockCumulativeSumPattern[StoredU64] = AverageBlockCumulativeSumPattern(client, _p('p2sh', acc))
        self.p2tr: AverageBlockCumulativeSumPattern[StoredU64] = AverageBlockCumulativeSumPattern(client, _p('p2tr', acc))
        self.p2wpkh: AverageBlockCumulativeSumPattern[StoredU64] = AverageBlockCumulativeSumPattern(client, _p('p2wpkh', acc))
        self.p2wsh: AverageBlockCumulativeSumPattern[StoredU64] = AverageBlockCumulativeSumPattern(client, _p('p2wsh', acc))

class AllP2aP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshPattern5:
    """Pattern struct for repeated tree structure."""
    
    def __init__(self, client: BrkClient, acc: str):
        """Create pattern node with accumulated series name."""
        self.all: BpsPercentRatioPattern2 = BpsPercentRatioPattern2(client, acc)
        self.p2a: BpsPercentRatioPattern2 = BpsPercentRatioPattern2(client, _p('p2a', acc))
        self.p2pk33: BpsPercentRatioPattern2 = BpsPercentRatioPattern2(client, _p('p2pk33', acc))
        self.p2pk65: BpsPercentRatioPattern2 = BpsPercentRatioPattern2(client, _p('p2pk65', acc))
        self.p2pkh: BpsPercentRatioPattern2 = BpsPercentRatioPattern2(client, _p('p2pkh', acc))
        self.p2sh: BpsPercentRatioPattern2 = BpsPercentRatioPattern2(client, _p('p2sh', acc))
        self.p2tr: BpsPercentRatioPattern2 = BpsPercentRatioPattern2(client, _p('p2tr', acc))
        self.p2wpkh: BpsPercentRatioPattern2 = BpsPercentRatioPattern2(client, _p('p2wpkh', acc))
        self.p2wsh: BpsPercentRatioPattern2 = BpsPercentRatioPattern2(client, _p('p2wsh', acc))

class AllP2aP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshPattern4:
    """Pattern struct for repeated tree structure."""
    
    def __init__(self, client: BrkClient, acc: str):
        """Create pattern node with accumulated series name."""
        self.all: SeriesPattern1[StoredU64] = SeriesPattern1(client, acc)
        self.p2a: SeriesPattern1[StoredU64] = SeriesPattern1(client, _p('p2a', acc))
        self.p2pk33: SeriesPattern1[StoredU64] = SeriesPattern1(client, _p('p2pk33', acc))
        self.p2pk65: SeriesPattern1[StoredU64] = SeriesPattern1(client, _p('p2pk65', acc))
        self.p2pkh: SeriesPattern1[StoredU64] = SeriesPattern1(client, _p('p2pkh', acc))
        self.p2sh: SeriesPattern1[StoredU64] = SeriesPattern1(client, _p('p2sh', acc))
        self.p2tr: SeriesPattern1[StoredU64] = SeriesPattern1(client, _p('p2tr', acc))
        self.p2wpkh: SeriesPattern1[StoredU64] = SeriesPattern1(client, _p('p2wpkh', acc))
        self.p2wsh: SeriesPattern1[StoredU64] = SeriesPattern1(client, _p('p2wsh', acc))

class AllP2aP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshPattern7:
    """Pattern struct for repeated tree structure."""
    
    def __init__(self, client: BrkClient, acc: str):
        """Create pattern node with accumulated series name."""
        self.all: _1m1w1y24hBpsPercentRatioPattern = _1m1w1y24hBpsPercentRatioPattern(client, acc)
        self.p2a: _1m1w1y24hBpsPercentRatioPattern = _1m1w1y24hBpsPercentRatioPattern(client, _p('p2a', acc))
        self.p2pk33: _1m1w1y24hBpsPercentRatioPattern = _1m1w1y24hBpsPercentRatioPattern(client, _p('p2pk33', acc))
        self.p2pk65: _1m1w1y24hBpsPercentRatioPattern = _1m1w1y24hBpsPercentRatioPattern(client, _p('p2pk65', acc))
        self.p2pkh: _1m1w1y24hBpsPercentRatioPattern = _1m1w1y24hBpsPercentRatioPattern(client, _p('p2pkh', acc))
        self.p2sh: _1m1w1y24hBpsPercentRatioPattern = _1m1w1y24hBpsPercentRatioPattern(client, _p('p2sh', acc))
        self.p2tr: _1m1w1y24hBpsPercentRatioPattern = _1m1w1y24hBpsPercentRatioPattern(client, _p('p2tr', acc))
        self.p2wpkh: _1m1w1y24hBpsPercentRatioPattern = _1m1w1y24hBpsPercentRatioPattern(client, _p('p2wpkh', acc))
        self.p2wsh: _1m1w1y24hBpsPercentRatioPattern = _1m1w1y24hBpsPercentRatioPattern(client, _p('p2wsh', acc))

class AverageMaxMedianMinPct10Pct25Pct75Pct90SumPattern:
    """Pattern struct for repeated tree structure."""
    
    def __init__(self, client: BrkClient, acc: str):
        """Create pattern node with accumulated series name."""
        self.average: _1m1w1y24hPattern[StoredF32] = _1m1w1y24hPattern(client, _m(acc, 'average'))
        self.max: _1m1w1y24hPattern[StoredU64] = _1m1w1y24hPattern(client, _m(acc, 'max'))
        self.median: _1m1w1y24hPattern[StoredU64] = _1m1w1y24hPattern(client, _m(acc, 'median'))
        self.min: _1m1w1y24hPattern[StoredU64] = _1m1w1y24hPattern(client, _m(acc, 'min'))
        self.pct10: _1m1w1y24hPattern[StoredU64] = _1m1w1y24hPattern(client, _m(acc, 'pct10'))
        self.pct25: _1m1w1y24hPattern[StoredU64] = _1m1w1y24hPattern(client, _m(acc, 'pct25'))
        self.pct75: _1m1w1y24hPattern[StoredU64] = _1m1w1y24hPattern(client, _m(acc, 'pct75'))
        self.pct90: _1m1w1y24hPattern[StoredU64] = _1m1w1y24hPattern(client, _m(acc, 'pct90'))
        self.sum: _1m1w1y24hPattern[StoredU64] = _1m1w1y24hPattern(client, _m(acc, 'sum'))

class CapitalizedGrossInvestedLossNetNuplProfitSentimentPattern2:
    """Pattern struct for repeated tree structure."""
    
    def __init__(self, client: BrkClient, acc: str):
        """Create pattern node with accumulated series name."""
        self.capitalized_cap_in_loss_raw: SeriesPattern18[CentsSquaredSats] = SeriesPattern18(client, _m(acc, 'capitalized_cap_in_loss_raw'))
        self.capitalized_cap_in_profit_raw: SeriesPattern18[CentsSquaredSats] = SeriesPattern18(client, _m(acc, 'capitalized_cap_in_profit_raw'))
        self.gross_pnl: CentsUsdPattern3 = CentsUsdPattern3(client, _m(acc, 'unrealized_gross_pnl'))
        self.invested_capital: InPattern2 = InPattern2(client, _m(acc, 'invested_capital_in'))
        self.loss: CentsNegativeToUsdPattern2 = CentsNegativeToUsdPattern2(client, _m(acc, 'unrealized_loss'))
        self.net_pnl: CentsToUsdPattern3 = CentsToUsdPattern3(client, _m(acc, 'net_unrealized_pnl'))
        self.nupl: BpsRatioPattern = BpsRatioPattern(client, _m(acc, 'nupl'))
        self.profit: CentsToUsdPattern4 = CentsToUsdPattern4(client, _m(acc, 'unrealized_profit'))
        self.sentiment: GreedNetPainPattern = GreedNetPainPattern(client, acc)

class BpsCentsPercentilesRatioSatsSmaStdUsdPattern:
    """Pattern struct for repeated tree structure."""
    pass

class Pct0Pct1Pct2Pct5Pct95Pct98Pct99Pattern:
    """Pattern struct for repeated tree structure."""
    
    def __init__(self, client: BrkClient, acc: str):
        """Create pattern node with accumulated series name."""
        self.pct0_5: BpsPriceRatioPattern = BpsPriceRatioPattern(client, acc, 'pct0_5')
        self.pct1: BpsPriceRatioPattern = BpsPriceRatioPattern(client, acc, 'pct1')
        self.pct2: BpsPriceRatioPattern = BpsPriceRatioPattern(client, acc, 'pct2')
        self.pct5: BpsPriceRatioPattern = BpsPriceRatioPattern(client, acc, 'pct5')
        self.pct95: BpsPriceRatioPattern = BpsPriceRatioPattern(client, acc, 'pct95')
        self.pct98: BpsPriceRatioPattern = BpsPriceRatioPattern(client, acc, 'pct98')
        self.pct99: BpsPriceRatioPattern = BpsPriceRatioPattern(client, acc, 'pct99')
        self.pct99_5: BpsPriceRatioPattern = BpsPriceRatioPattern(client, acc, 'pct99_5')

class _10y2y3y4y5y6y8yPattern:
    """Pattern struct for repeated tree structure."""
    
    def __init__(self, client: BrkClient, acc: str):
        """Create pattern node with accumulated series name."""
        self._10y: BpsPercentRatioPattern = BpsPercentRatioPattern(client, _m(acc, '10y'))
        self._2y: BpsPercentRatioPattern = BpsPercentRatioPattern(client, _m(acc, '2y'))
        self._3y: BpsPercentRatioPattern = BpsPercentRatioPattern(client, _m(acc, '3y'))
        self._4y: BpsPercentRatioPattern = BpsPercentRatioPattern(client, _m(acc, '4y'))
        self._5y: BpsPercentRatioPattern = BpsPercentRatioPattern(client, _m(acc, '5y'))
        self._6y: BpsPercentRatioPattern = BpsPercentRatioPattern(client, _m(acc, '6y'))
        self._8y: BpsPercentRatioPattern = BpsPercentRatioPattern(client, _m(acc, '8y'))

class _1m1w1y24hBpsPercentRatioPattern:
    """Pattern struct for repeated tree structure."""
    
    def __init__(self, client: BrkClient, acc: str):
        """Create pattern node with accumulated series name."""
        self._1m: BpsPercentRatioPattern2 = BpsPercentRatioPattern2(client, _m(acc, '1m'))
        self._1w: BpsPercentRatioPattern2 = BpsPercentRatioPattern2(client, _m(acc, '1w'))
        self._1y: BpsPercentRatioPattern2 = BpsPercentRatioPattern2(client, _m(acc, '1y'))
        self._24h: BpsPercentRatioPattern2 = BpsPercentRatioPattern2(client, _m(acc, '24h'))
        self.bps: SeriesPattern1[BasisPoints16] = SeriesPattern1(client, _m(acc, 'bps'))
        self.percent: SeriesPattern1[StoredF32] = SeriesPattern1(client, acc)
        self.ratio: SeriesPattern1[StoredF32] = SeriesPattern1(client, _m(acc, 'ratio'))

class ActiveInputOutputSpendablePattern:
    """Pattern struct for repeated tree structure."""
    pass

class ActivityCostInvestedOutputsRealizedSupplyUnrealizedPattern2:
    """Pattern struct for repeated tree structure."""
    pass

class CapLossMvrvNetPriceProfitSoprPattern:
    """Pattern struct for repeated tree structure."""
    
    def __init__(self, client: BrkClient, acc: str):
        """Create pattern node with accumulated series name."""
        self.cap: CentsDeltaUsdPattern = CentsDeltaUsdPattern(client, _m(acc, 'realized_cap'))
        self.loss: BlockCumulativeNegativeSumPattern = BlockCumulativeNegativeSumPattern(client, _m(acc, 'realized_loss'))
        self.mvrv: SeriesPattern1[StoredF32] = SeriesPattern1(client, _m(acc, 'mvrv'))
        self.net_pnl: BlockCumulativeDeltaSumPattern = BlockCumulativeDeltaSumPattern(client, _m(acc, 'net_realized_pnl'))
        self.price: BpsCentsRatioSatsUsdPattern = BpsCentsRatioSatsUsdPattern(client, _m(acc, 'realized_price'))
        self.profit: BlockCumulativeSumPattern = BlockCumulativeSumPattern(client, _m(acc, 'realized_profit'))
        self.sopr: RatioValuePattern = RatioValuePattern(client, acc)

class InMaxMinPerSupplyPattern:
    """Pattern struct for repeated tree structure."""
    
    def __init__(self, client: BrkClient, acc: str):
        """Create pattern node with accumulated series name."""
        self.in_loss: PerPattern = PerPattern(client, _m(acc, 'cost_basis_in_loss_per'))
        self.in_profit: PerPattern = PerPattern(client, _m(acc, 'cost_basis_in_profit_per'))
        self.max: CentsSatsUsdPattern = CentsSatsUsdPattern(client, _m(acc, 'cost_basis_max'))
        self.min: CentsSatsUsdPattern = CentsSatsUsdPattern(client, _m(acc, 'cost_basis_min'))
        self.per_coin: Pct05Pct10Pct15Pct20Pct25Pct30Pct35Pct40Pct45Pct50Pct55Pct60Pct65Pct70Pct75Pct80Pct85Pct90Pct95Pattern = Pct05Pct10Pct15Pct20Pct25Pct30Pct35Pct40Pct45Pct50Pct55Pct60Pct65Pct70Pct75Pct80Pct85Pct90Pct95Pattern(client, _m(acc, 'cost_basis_per_coin'))
        self.per_dollar: Pct05Pct10Pct15Pct20Pct25Pct30Pct35Pct40Pct45Pct50Pct55Pct60Pct65Pct70Pct75Pct80Pct85Pct90Pct95Pattern = Pct05Pct10Pct15Pct20Pct25Pct30Pct35Pct40Pct45Pct50Pct55Pct60Pct65Pct70Pct75Pct80Pct85Pct90Pct95Pattern(client, _m(acc, 'cost_basis_per_dollar'))
        self.supply_density: BpsPercentRatioPattern2 = BpsPercentRatioPattern2(client, _m(acc, 'supply_density'))

class MaxMedianMinPct10Pct25Pct75Pct90Pattern2:
    """Pattern struct for repeated tree structure."""
    
    def __init__(self, client: BrkClient, acc: str):
        """Create pattern node with accumulated series name."""
        self.max: SeriesPattern18[Weight] = SeriesPattern18(client, _m(acc, 'max'))
        self.median: SeriesPattern18[Weight] = SeriesPattern18(client, _m(acc, 'median'))
        self.min: SeriesPattern18[Weight] = SeriesPattern18(client, _m(acc, 'min'))
        self.pct10: SeriesPattern18[Weight] = SeriesPattern18(client, _m(acc, 'pct10'))
        self.pct25: SeriesPattern18[Weight] = SeriesPattern18(client, _m(acc, 'pct25'))
        self.pct75: SeriesPattern18[Weight] = SeriesPattern18(client, _m(acc, 'pct75'))
        self.pct90: SeriesPattern18[Weight] = SeriesPattern18(client, _m(acc, 'pct90'))

class MaxMedianMinPct10Pct25Pct75Pct90Pattern(Generic[T]):
    """Pattern struct for repeated tree structure."""
    
    def __init__(self, client: BrkClient, acc: str):
        """Create pattern node with accumulated series name."""
        self.max: SeriesPattern1[T] = SeriesPattern1(client, _m(acc, 'max'))
        self.median: SeriesPattern1[T] = SeriesPattern1(client, _m(acc, 'median'))
        self.min: SeriesPattern1[T] = SeriesPattern1(client, _m(acc, 'min'))
        self.pct10: SeriesPattern1[T] = SeriesPattern1(client, _m(acc, 'pct10'))
        self.pct25: SeriesPattern1[T] = SeriesPattern1(client, _m(acc, 'pct25'))
        self.pct75: SeriesPattern1[T] = SeriesPattern1(client, _m(acc, 'pct75'))
        self.pct90: SeriesPattern1[T] = SeriesPattern1(client, _m(acc, 'pct90'))

class _1m1w1y2y4yAllPattern:
    """Pattern struct for repeated tree structure."""
    
    def __init__(self, client: BrkClient, acc: str):
        """Create pattern node with accumulated series name."""
        self._1m: BpsRatioPattern2 = BpsRatioPattern2(client, _m(acc, '1m'))
        self._1w: BpsRatioPattern2 = BpsRatioPattern2(client, _m(acc, '1w'))
        self._1y: BpsRatioPattern2 = BpsRatioPattern2(client, _m(acc, '1y'))
        self._2y: BpsRatioPattern2 = BpsRatioPattern2(client, _m(acc, '2y'))
        self._4y: BpsRatioPattern2 = BpsRatioPattern2(client, _m(acc, '4y'))
        self.all: BpsRatioPattern2 = BpsRatioPattern2(client, _m(acc, 'all'))

class ActivityAddrOutputsRealizedSupplyUnrealizedPattern:
    """Pattern struct for repeated tree structure."""
    
    def __init__(self, client: BrkClient, acc: str):
        """Create pattern node with accumulated series name."""
        self.activity: TransferPattern = TransferPattern(client, _m(acc, 'transfer_volume'))
        self.addr_count: BaseDeltaPattern = BaseDeltaPattern(client, _m(acc, 'addr_count'))
        self.outputs: SpendingSpentUnspentPattern = SpendingSpentUnspentPattern(client, acc)
        self.realized: CapLossMvrvPriceProfitPattern = CapLossMvrvPriceProfitPattern(client, acc)
        self.supply: DeltaDominanceTotalPattern = DeltaDominanceTotalPattern(client, _m(acc, 'supply'))
        self.unrealized: NuplPattern = NuplPattern(client, _m(acc, 'nupl'))

class AverageBlockCumulativeInSumPattern:
    """Pattern struct for repeated tree structure."""
    
    def __init__(self, client: BrkClient, acc: str):
        """Create pattern node with accumulated series name."""
        self.average: _1m1w1y24hPattern3 = _1m1w1y24hPattern3(client, _m(acc, 'average'))
        self.block: BtcCentsSatsUsdPattern3 = BtcCentsSatsUsdPattern3(client, acc)
        self.cumulative: BtcCentsSatsUsdPattern = BtcCentsSatsUsdPattern(client, _m(acc, 'cumulative'))
        self.in_loss: AverageBlockCumulativeSumPattern3 = AverageBlockCumulativeSumPattern3(client, _m(acc, 'in_loss'))
        self.in_profit: AverageBlockCumulativeSumPattern3 = AverageBlockCumulativeSumPattern3(client, _m(acc, 'in_profit'))
        self.sum: _1m1w1y24hPattern4 = _1m1w1y24hPattern4(client, _m(acc, 'sum'))

class BpsCentsPercentilesRatioSatsUsdPattern:
    """Pattern struct for repeated tree structure."""
    
    def __init__(self, client: BrkClient, acc: str):
        """Create pattern node with accumulated series name."""
        self.bps: SeriesPattern1[BasisPoints32] = SeriesPattern1(client, _m(acc, 'ratio_bps'))
        self.cents: SeriesPattern1[Cents] = SeriesPattern1(client, _m(acc, 'cents'))
        self.percentiles: Pct0Pct1Pct2Pct5Pct95Pct98Pct99Pattern = Pct0Pct1Pct2Pct5Pct95Pct98Pct99Pattern(client, acc)
        self.ratio: SeriesPattern1[StoredF32] = SeriesPattern1(client, _m(acc, 'ratio'))
        self.sats: SeriesPattern1[SatsFract] = SeriesPattern1(client, _m(acc, 'sats'))
        self.usd: SeriesPattern1[Dollars] = SeriesPattern1(client, acc)

class CentsNegativeToUsdPattern2:
    """Pattern struct for repeated tree structure."""
    
    def __init__(self, client: BrkClient, acc: str):
        """Create pattern node with accumulated series name."""
        self.cents: SeriesPattern1[Cents] = SeriesPattern1(client, _m(acc, 'cents'))
        self.negative: SeriesPattern1[Dollars] = SeriesPattern1(client, _m(acc, 'neg'))
        self.to_mcap: BpsPercentRatioPattern2 = BpsPercentRatioPattern2(client, _m(acc, 'to_mcap'))
        self.to_own_gross_pnl: BpsPercentRatioPattern2 = BpsPercentRatioPattern2(client, _m(acc, 'to_own_gross_pnl'))
        self.to_own_mcap: BpsPercentRatioPattern4 = BpsPercentRatioPattern4(client, _m(acc, 'to_own_mcap'))
        self.usd: SeriesPattern1[Dollars] = SeriesPattern1(client, acc)

class DeltaDominanceHalfInTotalPattern2:
    """Pattern struct for repeated tree structure."""
    
    def __init__(self, client: BrkClient, acc: str):
        """Create pattern node with accumulated series name."""
        self.delta: AbsoluteRatePattern3 = AbsoluteRatePattern3(client, _m(acc, 'delta'))
        self.dominance: BpsPercentRatioPattern2 = BpsPercentRatioPattern2(client, _m(acc, 'dominance'))
        self.half: BtcCentsSatsUsdPattern = BtcCentsSatsUsdPattern(client, _m(acc, 'half'))
        self.in_loss: BtcCentsSatsShareUsdPattern = BtcCentsSatsShareUsdPattern(client, _m(acc, 'in_loss'))
        self.in_profit: BtcCentsSatsShareUsdPattern = BtcCentsSatsShareUsdPattern(client, _m(acc, 'in_profit'))
        self.total: BtcCentsSatsUsdPattern = BtcCentsSatsUsdPattern(client, acc)

class DeltaDominanceHalfInTotalPattern:
    """Pattern struct for repeated tree structure."""
    
    def __init__(self, client: BrkClient, acc: str):
        """Create pattern node with accumulated series name."""
        self.delta: AbsoluteRatePattern3 = AbsoluteRatePattern3(client, _m(acc, 'delta'))
        self.dominance: BpsPercentRatioPattern2 = BpsPercentRatioPattern2(client, _m(acc, 'dominance'))
        self.half: BtcCentsSatsUsdPattern = BtcCentsSatsUsdPattern(client, _m(acc, 'half'))
        self.in_loss: BtcCentsSatsUsdPattern = BtcCentsSatsUsdPattern(client, _m(acc, 'in_loss'))
        self.in_profit: BtcCentsSatsUsdPattern = BtcCentsSatsUsdPattern(client, _m(acc, 'in_profit'))
        self.total: BtcCentsSatsUsdPattern = BtcCentsSatsUsdPattern(client, acc)

class _1m1w1y24hBlockPattern2:
    """Pattern struct for repeated tree structure."""
    
    def __init__(self, client: BrkClient, acc: str):
        """Create pattern node with accumulated series name."""
        self._1m: SeriesPattern1[StoredF32] = SeriesPattern1(client, _m(acc, 'average_1m'))
        self._1w: SeriesPattern1[StoredF32] = SeriesPattern1(client, _m(acc, 'average_1w'))
        self._1y: SeriesPattern1[StoredF32] = SeriesPattern1(client, _m(acc, 'average_1y'))
        self._24h: SeriesPattern1[StoredF32] = SeriesPattern1(client, _m(acc, 'average_24h'))
        self.block: SeriesPattern18[StoredF32] = SeriesPattern18(client, acc)

class _1m1w1y24hBlockPattern:
    """Pattern struct for repeated tree structure."""
    
    def __init__(self, client: BrkClient, acc: str):
        """Create pattern node with accumulated series name."""
        self._1m: SeriesPattern1[StoredF32] = SeriesPattern1(client, _m(acc, 'average_1m'))
        self._1w: SeriesPattern1[StoredF32] = SeriesPattern1(client, _m(acc, 'average_1w'))
        self._1y: SeriesPattern1[StoredF32] = SeriesPattern1(client, _m(acc, 'average_1y'))
        self._24h: SeriesPattern1[StoredF32] = SeriesPattern1(client, _m(acc, 'average_24h'))
        self.block: SeriesPattern18[StoredU32] = SeriesPattern18(client, acc)

class ActiveBidirectionalReactivatedReceivingSendingPattern:
    """Pattern struct for repeated tree structure."""
    
    def __init__(self, client: BrkClient, acc: str):
        """Create pattern node with accumulated series name."""
        self.active: _1m1w1y24hBlockPattern = _1m1w1y24hBlockPattern(client, _m(acc, 'active_addrs'))
        self.bidirectional: _1m1w1y24hBlockPattern = _1m1w1y24hBlockPattern(client, _m(acc, 'bidirectional_addrs'))
        self.reactivated: _1m1w1y24hBlockPattern = _1m1w1y24hBlockPattern(client, _m(acc, 'reactivated_addrs'))
        self.receiving: _1m1w1y24hBlockPattern = _1m1w1y24hBlockPattern(client, _m(acc, 'receiving_addrs'))
        self.sending: _1m1w1y24hBlockPattern = _1m1w1y24hBlockPattern(client, _m(acc, 'sending_addrs'))

class ActivityOutputsRealizedSupplyUnrealizedPattern:
    """Pattern struct for repeated tree structure."""
    
    def __init__(self, client: BrkClient, acc: str):
        """Create pattern node with accumulated series name."""
        self.activity: CoindaysTransferPattern = CoindaysTransferPattern(client, acc)
        self.outputs: SpendingSpentUnspentPattern = SpendingSpentUnspentPattern(client, acc)
        self.realized: CapLossMvrvNetPriceProfitSoprPattern = CapLossMvrvNetPriceProfitSoprPattern(client, acc)
        self.supply: DeltaDominanceHalfInTotalPattern = DeltaDominanceHalfInTotalPattern(client, _m(acc, 'supply'))
        self.unrealized: LossNetNuplProfitPattern = LossNetNuplProfitPattern(client, acc)

class ActivityOutputsRealizedSupplyUnrealizedPattern3:
    """Pattern struct for repeated tree structure."""
    
    def __init__(self, client: BrkClient, acc: str):
        """Create pattern node with accumulated series name."""
        self.activity: TransferPattern = TransferPattern(client, _m(acc, 'transfer_volume'))
        self.outputs: SpendingSpentUnspentPattern = SpendingSpentUnspentPattern(client, acc)
        self.realized: CapLossMvrvPriceProfitPattern = CapLossMvrvPriceProfitPattern(client, acc)
        self.supply: DeltaDominanceHalfInTotalPattern = DeltaDominanceHalfInTotalPattern(client, _m(acc, 'supply'))
        self.unrealized: LossNuplProfitPattern = LossNuplProfitPattern(client, acc)

class ActivityOutputsRealizedSupplyUnrealizedPattern2:
    """Pattern struct for repeated tree structure."""
    
    def __init__(self, client: BrkClient, acc: str):
        """Create pattern node with accumulated series name."""
        self.activity: TransferPattern = TransferPattern(client, _m(acc, 'transfer_volume'))
        self.outputs: SpendingSpentUnspentPattern = SpendingSpentUnspentPattern(client, acc)
        self.realized: CapLossMvrvPriceProfitPattern = CapLossMvrvPriceProfitPattern(client, acc)
        self.supply: DeltaDominanceTotalPattern = DeltaDominanceTotalPattern(client, _m(acc, 'supply'))
        self.unrealized: NuplPattern = NuplPattern(client, _m(acc, 'nupl'))

class BlockChangeCumulativeDeltaSumPattern:
    """Pattern struct for repeated tree structure."""
    
    def __init__(self, client: BrkClient, acc: str):
        """Create pattern node with accumulated series name."""
        self.block: CentsUsdPattern4 = CentsUsdPattern4(client, _m(acc, 'realized_pnl'))
        self.change_1m: ToPattern = ToPattern(client, _m(acc, 'pnl_change_1m_to'))
        self.cumulative: CentsUsdPattern = CentsUsdPattern(client, _m(acc, 'realized_pnl_cumulative'))
        self.delta: AbsoluteRatePattern2 = AbsoluteRatePattern2(client, _m(acc, 'realized_pnl_delta'))
        self.sum: _1m1w1y24hPattern5 = _1m1w1y24hPattern5(client, _m(acc, 'realized_pnl_sum'))

class BpsCentsRatioSatsUsdPattern:
    """Pattern struct for repeated tree structure."""
    
    def __init__(self, client: BrkClient, acc: str):
        """Create pattern node with accumulated series name."""
        self.bps: SeriesPattern1[BasisPoints32] = SeriesPattern1(client, _m(acc, 'ratio_bps'))
        self.cents: SeriesPattern1[Cents] = SeriesPattern1(client, _m(acc, 'cents'))
        self.ratio: SeriesPattern1[StoredF32] = SeriesPattern1(client, _m(acc, 'ratio'))
        self.sats: SeriesPattern1[SatsFract] = SeriesPattern1(client, _m(acc, 'sats'))
        self.usd: SeriesPattern1[Dollars] = SeriesPattern1(client, acc)

class BtcCentsDeltaSatsUsdPattern:
    """Pattern struct for repeated tree structure."""
    
    def __init__(self, client: BrkClient, acc: str):
        """Create pattern node with accumulated series name."""
        self.btc: SeriesPattern1[Bitcoin] = SeriesPattern1(client, acc)
        self.cents: SeriesPattern1[Cents] = SeriesPattern1(client, _m(acc, 'cents'))
        self.delta: AbsoluteRatePattern3 = AbsoluteRatePattern3(client, _m(acc, 'delta'))
        self.sats: SeriesPattern1[Sats] = SeriesPattern1(client, _m(acc, 'sats'))
        self.usd: SeriesPattern1[Dollars] = SeriesPattern1(client, _m(acc, 'usd'))

class BtcCentsSatsShareUsdPattern:
    """Pattern struct for repeated tree structure."""
    
    def __init__(self, client: BrkClient, acc: str):
        """Create pattern node with accumulated series name."""
        self.btc: SeriesPattern1[Bitcoin] = SeriesPattern1(client, acc)
        self.cents: SeriesPattern1[Cents] = SeriesPattern1(client, _m(acc, 'cents'))
        self.sats: SeriesPattern1[Sats] = SeriesPattern1(client, _m(acc, 'sats'))
        self.share: BpsPercentRatioPattern2 = BpsPercentRatioPattern2(client, _m(acc, 'share'))
        self.usd: SeriesPattern1[Dollars] = SeriesPattern1(client, _m(acc, 'usd'))

class CapLossMvrvPriceProfitPattern:
    """Pattern struct for repeated tree structure."""
    
    def __init__(self, client: BrkClient, acc: str):
        """Create pattern node with accumulated series name."""
        self.cap: CentsDeltaUsdPattern = CentsDeltaUsdPattern(client, _m(acc, 'realized_cap'))
        self.loss: BlockCumulativeSumPattern = BlockCumulativeSumPattern(client, _m(acc, 'realized_loss'))
        self.mvrv: SeriesPattern1[StoredF32] = SeriesPattern1(client, _m(acc, 'mvrv'))
        self.price: BpsCentsRatioSatsUsdPattern = BpsCentsRatioSatsUsdPattern(client, _m(acc, 'realized_price'))
        self.profit: BlockCumulativeSumPattern = BlockCumulativeSumPattern(client, _m(acc, 'realized_profit'))

class CentsToUsdPattern4:
    """Pattern struct for repeated tree structure."""
    
    def __init__(self, client: BrkClient, acc: str):
        """Create pattern node with accumulated series name."""
        self.cents: SeriesPattern1[Cents] = SeriesPattern1(client, _m(acc, 'cents'))
        self.to_mcap: BpsPercentRatioPattern2 = BpsPercentRatioPattern2(client, _m(acc, 'to_mcap'))
        self.to_own_gross_pnl: BpsPercentRatioPattern2 = BpsPercentRatioPattern2(client, _m(acc, 'to_own_gross_pnl'))
        self.to_own_mcap: BpsPercentRatioPattern2 = BpsPercentRatioPattern2(client, _m(acc, 'to_own_mcap'))
        self.usd: SeriesPattern1[Dollars] = SeriesPattern1(client, acc)

class EmaHistogramLineSignalPattern:
    """Pattern struct for repeated tree structure."""
    pass

class PhsReboundThsPattern:
    """Pattern struct for repeated tree structure."""
    
    def __init__(self, client: BrkClient, acc: str):
        """Create pattern node with accumulated series name."""
        self.phs: SeriesPattern1[StoredF32] = SeriesPattern1(client, _m(acc, 'phs'))
        self.phs_min: SeriesPattern1[StoredF32] = SeriesPattern1(client, _m(acc, 'phs_min'))
        self.rebound: BpsPercentRatioPattern = BpsPercentRatioPattern(client, _m(acc, 'rebound'))
        self.ths: SeriesPattern1[StoredF32] = SeriesPattern1(client, _m(acc, 'ths'))
        self.ths_min: SeriesPattern1[StoredF32] = SeriesPattern1(client, _m(acc, 'ths_min'))

class _1m1w1y24hPattern2:
    """Pattern struct for repeated tree structure."""
    
    def __init__(self, client: BrkClient, acc: str):
        """Create pattern node with accumulated series name."""
        self._1m: BpsPercentRatioPattern = BpsPercentRatioPattern(client, _m(acc, '1m_rate'))
        self._1w: BpsPercentRatioPattern = BpsPercentRatioPattern(client, _m(acc, '1w_rate'))
        self._1y: BpsPercentRatioPattern = BpsPercentRatioPattern(client, _m(acc, '1y_rate'))
        self._24h: BpsPercentRatioPattern = BpsPercentRatioPattern(client, _m(acc, '24h_rate'))

class _1m1w1y24hPattern8:
    """Pattern struct for repeated tree structure."""
    
    def __init__(self, client: BrkClient, acc: str):
        """Create pattern node with accumulated series name."""
        self._1m: BpsPercentRatioPattern4 = BpsPercentRatioPattern4(client, _m(acc, '1m'))
        self._1w: BpsPercentRatioPattern4 = BpsPercentRatioPattern4(client, _m(acc, '1w'))
        self._1y: BpsPercentRatioPattern4 = BpsPercentRatioPattern4(client, _m(acc, '1y'))
        self._24h: BpsPercentRatioPattern4 = BpsPercentRatioPattern4(client, _m(acc, '24h'))

class _1m1w1y24hPattern4:
    """Pattern struct for repeated tree structure."""
    
    def __init__(self, client: BrkClient, acc: str):
        """Create pattern node with accumulated series name."""
        self._1m: BtcCentsSatsUsdPattern = BtcCentsSatsUsdPattern(client, _m(acc, '1m'))
        self._1w: BtcCentsSatsUsdPattern = BtcCentsSatsUsdPattern(client, _m(acc, '1w'))
        self._1y: BtcCentsSatsUsdPattern = BtcCentsSatsUsdPattern(client, _m(acc, '1y'))
        self._24h: BtcCentsSatsUsdPattern = BtcCentsSatsUsdPattern(client, _m(acc, '24h'))

class _1m1w1y24hPattern3:
    """Pattern struct for repeated tree structure."""
    
    def __init__(self, client: BrkClient, acc: str):
        """Create pattern node with accumulated series name."""
        self._1m: BtcCentsSatsUsdPattern2 = BtcCentsSatsUsdPattern2(client, _m(acc, '1m'))
        self._1w: BtcCentsSatsUsdPattern2 = BtcCentsSatsUsdPattern2(client, _m(acc, '1w'))
        self._1y: BtcCentsSatsUsdPattern2 = BtcCentsSatsUsdPattern2(client, _m(acc, '1y'))
        self._24h: BtcCentsSatsUsdPattern2 = BtcCentsSatsUsdPattern2(client, _m(acc, '24h'))

class _1m1w1y24hPattern7:
    """Pattern struct for repeated tree structure."""
    
    def __init__(self, client: BrkClient, acc: str):
        """Create pattern node with accumulated series name."""
        self._1m: BtcSatsPattern = BtcSatsPattern(client, _m(acc, '1m'))
        self._1w: BtcSatsPattern = BtcSatsPattern(client, _m(acc, '1w'))
        self._1y: BtcSatsPattern = BtcSatsPattern(client, _m(acc, '1y'))
        self._24h: BtcSatsPattern = BtcSatsPattern(client, _m(acc, '24h'))

class _1m1w1y2wPattern:
    """Pattern struct for repeated tree structure."""
    
    def __init__(self, client: BrkClient, acc: str):
        """Create pattern node with accumulated series name."""
        self._1m: CentsSatsUsdPattern = CentsSatsUsdPattern(client, _m(acc, '1m'))
        self._1w: CentsSatsUsdPattern = CentsSatsUsdPattern(client, _m(acc, '1w'))
        self._1y: CentsSatsUsdPattern = CentsSatsUsdPattern(client, _m(acc, '1y'))
        self._2w: CentsSatsUsdPattern = CentsSatsUsdPattern(client, _m(acc, '2w'))

class _1m1w1y24hPattern5:
    """Pattern struct for repeated tree structure."""
    
    def __init__(self, client: BrkClient, acc: str):
        """Create pattern node with accumulated series name."""
        self._1m: CentsUsdPattern = CentsUsdPattern(client, _m(acc, '1m'))
        self._1w: CentsUsdPattern = CentsUsdPattern(client, _m(acc, '1w'))
        self._1y: CentsUsdPattern = CentsUsdPattern(client, _m(acc, '1y'))
        self._24h: CentsUsdPattern = CentsUsdPattern(client, _m(acc, '24h'))

class _1m1w1y24hPattern6:
    """Pattern struct for repeated tree structure."""
    
    def __init__(self, client: BrkClient, acc: str):
        """Create pattern node with accumulated series name."""
        self._1m: CentsUsdPattern3 = CentsUsdPattern3(client, _m(acc, '1m'))
        self._1w: CentsUsdPattern3 = CentsUsdPattern3(client, _m(acc, '1w'))
        self._1y: CentsUsdPattern3 = CentsUsdPattern3(client, _m(acc, '1y'))
        self._24h: CentsUsdPattern3 = CentsUsdPattern3(client, _m(acc, '24h'))

class _1y2y4yAllPattern:
    """Pattern struct for repeated tree structure."""
    pass

class AverageBlockCumulativeSumPattern2:
    """Pattern struct for repeated tree structure."""
    
    def __init__(self, client: BrkClient, acc: str):
        """Create pattern node with accumulated series name."""
        self.average: _1m1w1y24hPattern[StoredF32] = _1m1w1y24hPattern(client, _m(acc, 'average'))
        self.block: SeriesPattern18[StoredU32] = SeriesPattern18(client, acc)
        self.cumulative: SeriesPattern1[StoredU64] = SeriesPattern1(client, _m(acc, 'cumulative'))
        self.sum: _1m1w1y24hPattern[StoredU64] = _1m1w1y24hPattern(client, _m(acc, 'sum'))

class AverageBlockCumulativeSumPattern3:
    """Pattern struct for repeated tree structure."""
    
    def __init__(self, client: BrkClient, acc: str):
        """Create pattern node with accumulated series name."""
        self.average: _1m1w1y24hPattern3 = _1m1w1y24hPattern3(client, _m(acc, 'average'))
        self.block: BtcCentsSatsUsdPattern3 = BtcCentsSatsUsdPattern3(client, acc)
        self.cumulative: BtcCentsSatsUsdPattern = BtcCentsSatsUsdPattern(client, _m(acc, 'cumulative'))
        self.sum: _1m1w1y24hPattern4 = _1m1w1y24hPattern4(client, _m(acc, 'sum'))

class BlockCumulativeNegativeSumPattern:
    """Pattern struct for repeated tree structure."""
    
    def __init__(self, client: BrkClient, acc: str):
        """Create pattern node with accumulated series name."""
        self.block: CentsUsdPattern2 = CentsUsdPattern2(client, acc)
        self.cumulative: CentsUsdPattern3 = CentsUsdPattern3(client, _m(acc, 'cumulative'))
        self.negative: BaseSumPattern = BaseSumPattern(client, _m(acc, 'neg'))
        self.sum: _1m1w1y24hPattern6 = _1m1w1y24hPattern6(client, _m(acc, 'sum'))

class BlockCumulativeDeltaSumPattern:
    """Pattern struct for repeated tree structure."""
    
    def __init__(self, client: BrkClient, acc: str):
        """Create pattern node with accumulated series name."""
        self.block: CentsUsdPattern4 = CentsUsdPattern4(client, acc)
        self.cumulative: CentsUsdPattern = CentsUsdPattern(client, _m(acc, 'cumulative'))
        self.delta: AbsoluteRatePattern2 = AbsoluteRatePattern2(client, _m(acc, 'delta'))
        self.sum: _1m1w1y24hPattern5 = _1m1w1y24hPattern5(client, _m(acc, 'sum'))

class BtcCentsSatsUsdPattern:
    """Pattern struct for repeated tree structure."""
    
    def __init__(self, client: BrkClient, acc: str):
        """Create pattern node with accumulated series name."""
        self.btc: SeriesPattern1[Bitcoin] = SeriesPattern1(client, acc)
        self.cents: SeriesPattern1[Cents] = SeriesPattern1(client, _m(acc, 'cents'))
        self.sats: SeriesPattern1[Sats] = SeriesPattern1(client, _m(acc, 'sats'))
        self.usd: SeriesPattern1[Dollars] = SeriesPattern1(client, _m(acc, 'usd'))

class BtcCentsSatsUsdPattern2:
    """Pattern struct for repeated tree structure."""
    
    def __init__(self, client: BrkClient, acc: str):
        """Create pattern node with accumulated series name."""
        self.btc: SeriesPattern1[Bitcoin] = SeriesPattern1(client, acc)
        self.cents: SeriesPattern1[StoredF32] = SeriesPattern1(client, _m(acc, 'cents'))
        self.sats: SeriesPattern1[StoredF32] = SeriesPattern1(client, _m(acc, 'sats'))
        self.usd: SeriesPattern1[Dollars] = SeriesPattern1(client, _m(acc, 'usd'))

class BtcCentsSatsUsdPattern3:
    """Pattern struct for repeated tree structure."""
    
    def __init__(self, client: BrkClient, acc: str):
        """Create pattern node with accumulated series name."""
        self.btc: SeriesPattern18[Bitcoin] = SeriesPattern18(client, acc)
        self.cents: SeriesPattern18[Cents] = SeriesPattern18(client, _m(acc, 'cents'))
        self.sats: SeriesPattern18[Sats] = SeriesPattern18(client, _m(acc, 'sats'))
        self.usd: SeriesPattern18[Dollars] = SeriesPattern18(client, _m(acc, 'usd'))

class CentsDeltaToUsdPattern:
    """Pattern struct for repeated tree structure."""
    
    def __init__(self, client: BrkClient, acc: str):
        """Create pattern node with accumulated series name."""
        self.cents: SeriesPattern1[Cents] = SeriesPattern1(client, _m(acc, 'cents'))
        self.delta: AbsoluteRatePattern2 = AbsoluteRatePattern2(client, _m(acc, 'delta'))
        self.to_own_mcap: BpsPercentRatioPattern4 = BpsPercentRatioPattern4(client, _m(acc, 'to_own_mcap'))
        self.usd: SeriesPattern1[Dollars] = SeriesPattern1(client, acc)

class CentsToUsdPattern3:
    """Pattern struct for repeated tree structure."""
    
    def __init__(self, client: BrkClient, acc: str):
        """Create pattern node with accumulated series name."""
        self.cents: SeriesPattern1[CentsSigned] = SeriesPattern1(client, _m(acc, 'cents'))
        self.to_own_gross_pnl: BpsPercentRatioPattern = BpsPercentRatioPattern(client, _m(acc, 'to_own_gross_pnl'))
        self.to_own_mcap: BpsPercentRatioPattern = BpsPercentRatioPattern(client, _m(acc, 'to_own_mcap'))
        self.usd: SeriesPattern1[Dollars] = SeriesPattern1(client, acc)

class CoindaysCoinyearsDormancyTransferPattern:
    """Pattern struct for repeated tree structure."""
    
    def __init__(self, client: BrkClient, acc: str):
        """Create pattern node with accumulated series name."""
        self.coindays_destroyed: AverageBlockCumulativeSumPattern[StoredF64] = AverageBlockCumulativeSumPattern(client, _m(acc, 'coindays_destroyed'))
        self.coinyears_destroyed: SeriesPattern1[StoredF64] = SeriesPattern1(client, _m(acc, 'coinyears_destroyed'))
        self.dormancy: _1m1w1y24hPattern[StoredF32] = _1m1w1y24hPattern(client, _m(acc, 'dormancy'))
        self.transfer_volume: AverageBlockCumulativeInSumPattern = AverageBlockCumulativeInSumPattern(client, _m(acc, 'transfer_volume'))

class LossNetNuplProfitPattern:
    """Pattern struct for repeated tree structure."""
    
    def __init__(self, client: BrkClient, acc: str):
        """Create pattern node with accumulated series name."""
        self.loss: CentsNegativeUsdPattern = CentsNegativeUsdPattern(client, _m(acc, 'unrealized_loss'))
        self.net_pnl: CentsUsdPattern = CentsUsdPattern(client, _m(acc, 'net_unrealized_pnl'))
        self.nupl: BpsRatioPattern = BpsRatioPattern(client, _m(acc, 'nupl'))
        self.profit: CentsUsdPattern3 = CentsUsdPattern3(client, _m(acc, 'unrealized_profit'))

class NuplRealizedSupplyUnrealizedPattern:
    """Pattern struct for repeated tree structure."""
    
    def __init__(self, client: BrkClient, acc: str):
        """Create pattern node with accumulated series name."""
        self.nupl: BpsRatioPattern = BpsRatioPattern(client, _m(acc, 'nupl'))
        self.realized_cap: AllSthPattern = AllSthPattern(client, acc, 'realized_cap')
        self.supply: AllSthPattern2 = AllSthPattern2(client, acc)
        self.unrealized_pnl: AllSthPattern = AllSthPattern(client, acc, 'unrealized_pnl')

class _1m1w1y24hPattern(Generic[T]):
    """Pattern struct for repeated tree structure."""
    
    def __init__(self, client: BrkClient, acc: str):
        """Create pattern node with accumulated series name."""
        self._1m: SeriesPattern1[T] = SeriesPattern1(client, _m(acc, '1m'))
        self._1w: SeriesPattern1[T] = SeriesPattern1(client, _m(acc, '1w'))
        self._1y: SeriesPattern1[T] = SeriesPattern1(client, _m(acc, '1y'))
        self._24h: SeriesPattern1[T] = SeriesPattern1(client, _m(acc, '24h'))

class AverageBlockCumulativeSumPattern(Generic[T]):
    """Pattern struct for repeated tree structure."""
    
    def __init__(self, client: BrkClient, acc: str):
        """Create pattern node with accumulated series name."""
        self.average: _1m1w1y24hPattern[T] = _1m1w1y24hPattern(client, _m(acc, 'average'))
        self.block: SeriesPattern18[T] = SeriesPattern18(client, acc)
        self.cumulative: SeriesPattern1[T] = SeriesPattern1(client, _m(acc, 'cumulative'))
        self.sum: _1m1w1y24hPattern[T] = _1m1w1y24hPattern(client, _m(acc, 'sum'))

class AdjustedRatioValuePattern:
    """Pattern struct for repeated tree structure."""
    
    def __init__(self, client: BrkClient, acc: str):
        """Create pattern node with accumulated series name."""
        self.adjusted: RatioTransferValuePattern = RatioTransferValuePattern(client, acc)
        self.ratio: _1m1w1y24hPattern[StoredF64] = _1m1w1y24hPattern(client, _m(acc, 'sopr'))
        self.value_destroyed: AverageBlockCumulativeSumPattern[Cents] = AverageBlockCumulativeSumPattern(client, _m(acc, 'value_destroyed'))

class BlockCumulativeSumPattern:
    """Pattern struct for repeated tree structure."""
    
    def __init__(self, client: BrkClient, acc: str):
        """Create pattern node with accumulated series name."""
        self.block: CentsUsdPattern2 = CentsUsdPattern2(client, acc)
        self.cumulative: CentsUsdPattern3 = CentsUsdPattern3(client, _m(acc, 'cumulative'))
        self.sum: _1m1w1y24hPattern6 = _1m1w1y24hPattern6(client, _m(acc, 'sum'))

class BlocksDominanceRewardsPattern:
    """Pattern struct for repeated tree structure."""
    
    def __init__(self, client: BrkClient, acc: str):
        """Create pattern node with accumulated series name."""
        self.blocks_mined: AverageBlockCumulativeSumPattern2 = AverageBlockCumulativeSumPattern2(client, _m(acc, 'blocks_mined'))
        self.dominance: _1m1w1y24hBpsPercentRatioPattern = _1m1w1y24hBpsPercentRatioPattern(client, _m(acc, 'dominance'))
        self.rewards: AverageBlockCumulativeSumPattern3 = AverageBlockCumulativeSumPattern3(client, _m(acc, 'rewards'))

class BpsPercentRatioPattern2:
    """Pattern struct for repeated tree structure."""
    
    def __init__(self, client: BrkClient, acc: str):
        """Create pattern node with accumulated series name."""
        self.bps: SeriesPattern1[BasisPoints16] = SeriesPattern1(client, _m(acc, 'bps'))
        self.percent: SeriesPattern1[StoredF32] = SeriesPattern1(client, acc)
        self.ratio: SeriesPattern1[StoredF32] = SeriesPattern1(client, _m(acc, 'ratio'))

class BpsPercentRatioPattern4:
    """Pattern struct for repeated tree structure."""
    
    def __init__(self, client: BrkClient, acc: str):
        """Create pattern node with accumulated series name."""
        self.bps: SeriesPattern1[BasisPoints32] = SeriesPattern1(client, _m(acc, 'bps'))
        self.percent: SeriesPattern1[StoredF32] = SeriesPattern1(client, acc)
        self.ratio: SeriesPattern1[StoredF32] = SeriesPattern1(client, _m(acc, 'ratio'))

class BpsPriceRatioPattern:
    """Pattern struct for repeated tree structure."""
    
    def __init__(self, client: BrkClient, acc: str, disc: str):
        """Create pattern node with accumulated series name."""
        self.bps: SeriesPattern1[BasisPoints32] = SeriesPattern1(client, _m(acc, f'ratio_{disc}_bps'))
        self.price: CentsSatsUsdPattern = CentsSatsUsdPattern(client, _m(acc, disc))
        self.ratio: SeriesPattern1[StoredF32] = SeriesPattern1(client, _m(acc, f'ratio_{disc}'))

class BpsPercentRatioPattern5:
    """Pattern struct for repeated tree structure."""
    
    def __init__(self, client: BrkClient, acc: str):
        """Create pattern node with accumulated series name."""
        self.bps: SeriesPattern1[BasisPointsSigned16] = SeriesPattern1(client, _m(acc, 'bps'))
        self.percent: SeriesPattern1[StoredF32] = SeriesPattern1(client, acc)
        self.ratio: SeriesPattern1[StoredF32] = SeriesPattern1(client, _m(acc, 'ratio'))

class BpsPercentRatioPattern:
    """Pattern struct for repeated tree structure."""
    
    def __init__(self, client: BrkClient, acc: str):
        """Create pattern node with accumulated series name."""
        self.bps: SeriesPattern1[BasisPointsSigned32] = SeriesPattern1(client, _m(acc, 'bps'))
        self.percent: SeriesPattern1[StoredF32] = SeriesPattern1(client, acc)
        self.ratio: SeriesPattern1[StoredF32] = SeriesPattern1(client, _m(acc, 'ratio'))

class CentsSatsUsdPattern3:
    """Pattern struct for repeated tree structure."""
    
    def __init__(self, client: BrkClient, acc: str):
        """Create pattern node with accumulated series name."""
        self.cents: SeriesPattern2[Cents] = SeriesPattern2(client, _m(acc, 'cents'))
        self.sats: SeriesPattern2[Sats] = SeriesPattern2(client, _m(acc, 'sats'))
        self.usd: SeriesPattern2[Dollars] = SeriesPattern2(client, acc)

class CentsDeltaUsdPattern:
    """Pattern struct for repeated tree structure."""
    
    def __init__(self, client: BrkClient, acc: str):
        """Create pattern node with accumulated series name."""
        self.cents: SeriesPattern1[Cents] = SeriesPattern1(client, _m(acc, 'cents'))
        self.delta: AbsoluteRatePattern2 = AbsoluteRatePattern2(client, _m(acc, 'delta'))
        self.usd: SeriesPattern1[Dollars] = SeriesPattern1(client, acc)

class CentsNegativeUsdPattern:
    """Pattern struct for repeated tree structure."""
    
    def __init__(self, client: BrkClient, acc: str):
        """Create pattern node with accumulated series name."""
        self.cents: SeriesPattern1[Cents] = SeriesPattern1(client, _m(acc, 'cents'))
        self.negative: SeriesPattern1[Dollars] = SeriesPattern1(client, _m(acc, 'neg'))
        self.usd: SeriesPattern1[Dollars] = SeriesPattern1(client, acc)

class CentsSatsUsdPattern:
    """Pattern struct for repeated tree structure."""
    
    def __init__(self, client: BrkClient, acc: str):
        """Create pattern node with accumulated series name."""
        self.cents: SeriesPattern1[Cents] = SeriesPattern1(client, _m(acc, 'cents'))
        self.sats: SeriesPattern1[SatsFract] = SeriesPattern1(client, _m(acc, 'sats'))
        self.usd: SeriesPattern1[Dollars] = SeriesPattern1(client, acc)

class CountEventsSupplyPattern:
    """Pattern struct for repeated tree structure."""
    pass

class CumulativeRollingSumPattern:
    """Pattern struct for repeated tree structure."""
    
    def __init__(self, client: BrkClient, acc: str):
        """Create pattern node with accumulated series name."""
        self.cumulative: SeriesPattern1[StoredU64] = SeriesPattern1(client, _m(acc, 'cumulative'))
        self.rolling: AverageMaxMedianMinPct10Pct25Pct75Pct90SumPattern = AverageMaxMedianMinPct10Pct25Pct75Pct90SumPattern(client, acc)
        self.sum: SeriesPattern18[StoredU64] = SeriesPattern18(client, _m(acc, 'sum'))

class DeltaDominanceTotalPattern:
    """Pattern struct for repeated tree structure."""
    
    def __init__(self, client: BrkClient, acc: str):
        """Create pattern node with accumulated series name."""
        self.delta: AbsoluteRatePattern3 = AbsoluteRatePattern3(client, _m(acc, 'delta'))
        self.dominance: BpsPercentRatioPattern2 = BpsPercentRatioPattern2(client, _m(acc, 'dominance'))
        self.total: BtcCentsSatsUsdPattern = BtcCentsSatsUsdPattern(client, acc)

class GreedNetPainPattern:
    """Pattern struct for repeated tree structure."""
    
    def __init__(self, client: BrkClient, acc: str):
        """Create pattern node with accumulated series name."""
        self.greed_index: CentsUsdPattern3 = CentsUsdPattern3(client, _m(acc, 'greed_index'))
        self.net: CentsUsdPattern = CentsUsdPattern(client, _m(acc, 'net_sentiment'))
        self.pain_index: CentsUsdPattern3 = CentsUsdPattern3(client, _m(acc, 'pain_index'))

class LossNuplProfitPattern:
    """Pattern struct for repeated tree structure."""
    
    def __init__(self, client: BrkClient, acc: str):
        """Create pattern node with accumulated series name."""
        self.loss: CentsNegativeUsdPattern = CentsNegativeUsdPattern(client, _m(acc, 'unrealized_loss'))
        self.nupl: BpsRatioPattern = BpsRatioPattern(client, _m(acc, 'nupl'))
        self.profit: CentsUsdPattern3 = CentsUsdPattern3(client, _m(acc, 'unrealized_profit'))

class RatioTransferValuePattern:
    """Pattern struct for repeated tree structure."""
    
    def __init__(self, client: BrkClient, acc: str):
        """Create pattern node with accumulated series name."""
        self.ratio: _1m1w1y24hPattern[StoredF64] = _1m1w1y24hPattern(client, _m(acc, 'asopr'))
        self.transfer_volume: AverageBlockCumulativeSumPattern[Cents] = AverageBlockCumulativeSumPattern(client, _m(acc, 'adj_value_created'))
        self.value_destroyed: AverageBlockCumulativeSumPattern[Cents] = AverageBlockCumulativeSumPattern(client, _m(acc, 'adj_value_destroyed'))

class RsiStochPattern:
    """Pattern struct for repeated tree structure."""
    
    def __init__(self, client: BrkClient, acc: str, disc: str):
        """Create pattern node with accumulated series name."""
        self.rsi: BpsPercentRatioPattern2 = BpsPercentRatioPattern2(client, _m(acc, disc))
        self.stoch_rsi_d: BpsPercentRatioPattern2 = BpsPercentRatioPattern2(client, _m(acc, f'stoch_d_{disc}'))
        self.stoch_rsi_k: BpsPercentRatioPattern2 = BpsPercentRatioPattern2(client, _m(acc, f'stoch_k_{disc}'))

class SpendingSpentUnspentPattern:
    """Pattern struct for repeated tree structure."""
    
    def __init__(self, client: BrkClient, acc: str):
        """Create pattern node with accumulated series name."""
        self.spending_rate: SeriesPattern1[StoredF32] = SeriesPattern1(client, _m(acc, 'spending_rate'))
        self.spent_count: AverageBlockCumulativeSumPattern2 = AverageBlockCumulativeSumPattern2(client, _m(acc, 'spent_utxo_count'))
        self.unspent_count: BaseDeltaPattern = BaseDeltaPattern(client, _m(acc, 'utxo_count'))

class _6bBlockTxPattern(Generic[T]):
    """Pattern struct for repeated tree structure."""
    
    def __init__(self, client: BrkClient, acc: str):
        """Create pattern node with accumulated series name."""
        self._6b: MaxMedianMinPct10Pct25Pct75Pct90Pattern[T] = MaxMedianMinPct10Pct25Pct75Pct90Pattern(client, _m(acc, '6b'))
        self.block: MaxMedianMinPct10Pct25Pct75Pct90Pattern[T] = MaxMedianMinPct10Pct25Pct75Pct90Pattern(client, acc)
        self.tx_index: SeriesPattern19[T] = SeriesPattern19(client, acc)

class AbsoluteRatePattern:
    """Pattern struct for repeated tree structure."""
    
    def __init__(self, client: BrkClient, acc: str):
        """Create pattern node with accumulated series name."""
        self.absolute: _1m1w1y24hPattern[StoredI64] = _1m1w1y24hPattern(client, acc)
        self.rate: _1m1w1y24hPattern2 = _1m1w1y24hPattern2(client, acc)

class AbsoluteRatePattern2:
    """Pattern struct for repeated tree structure."""
    
    def __init__(self, client: BrkClient, acc: str):
        """Create pattern node with accumulated series name."""
        self.absolute: _1m1w1y24hPattern5 = _1m1w1y24hPattern5(client, acc)
        self.rate: _1m1w1y24hPattern2 = _1m1w1y24hPattern2(client, acc)

class AbsoluteRatePattern3:
    """Pattern struct for repeated tree structure."""
    
    def __init__(self, client: BrkClient, acc: str):
        """Create pattern node with accumulated series name."""
        self.absolute: _1m1w1y24hPattern7 = _1m1w1y24hPattern7(client, acc)
        self.rate: _1m1w1y24hPattern2 = _1m1w1y24hPattern2(client, acc)

class AddrUtxoPattern:
    """Pattern struct for repeated tree structure."""
    
    def __init__(self, client: BrkClient, acc: str):
        """Create pattern node with accumulated series name."""
        self.addr: BtcCentsSatsUsdPattern = BtcCentsSatsUsdPattern(client, _m(acc, 'addr_amount'))
        self.utxo: BtcCentsSatsUsdPattern = BtcCentsSatsUsdPattern(client, _m(acc, 'utxo_amount'))

class AllSthPattern2:
    """Pattern struct for repeated tree structure."""
    
    def __init__(self, client: BrkClient, acc: str):
        """Create pattern node with accumulated series name."""
        self.all: BtcCentsDeltaSatsUsdPattern = BtcCentsDeltaSatsUsdPattern(client, _m(acc, 'supply'))
        self.sth: BtcCentsSatsUsdPattern = BtcCentsSatsUsdPattern(client, _m(acc, 'sth_supply'))

class AllSthPattern:
    """Pattern struct for repeated tree structure."""
    
    def __init__(self, client: BrkClient, acc: str, disc: str):
        """Create pattern node with accumulated series name."""
        self.all: SeriesPattern1[Dollars] = SeriesPattern1(client, _m(acc, disc))
        self.sth: SeriesPattern1[Dollars] = SeriesPattern1(client, _m(acc, f'sth_{disc}'))

class BaseSumPattern:
    """Pattern struct for repeated tree structure."""
    
    def __init__(self, client: BrkClient, acc: str):
        """Create pattern node with accumulated series name."""
        self.base: SeriesPattern18[Dollars] = SeriesPattern18(client, acc)
        self.sum: _1m1w1y24hPattern[Dollars] = _1m1w1y24hPattern(client, _m(acc, 'sum'))

class BaseDeltaPattern:
    """Pattern struct for repeated tree structure."""
    
    def __init__(self, client: BrkClient, acc: str):
        """Create pattern node with accumulated series name."""
        self.base: SeriesPattern1[StoredU64] = SeriesPattern1(client, acc)
        self.delta: AbsoluteRatePattern = AbsoluteRatePattern(client, _m(acc, 'delta'))

class BlockCumulativePattern:
    """Pattern struct for repeated tree structure."""
    
    def __init__(self, client: BrkClient, acc: str):
        """Create pattern node with accumulated series name."""
        self.block: BtcCentsSatsUsdPattern3 = BtcCentsSatsUsdPattern3(client, acc)
        self.cumulative: BtcCentsSatsUsdPattern = BtcCentsSatsUsdPattern(client, _m(acc, 'cumulative'))

class BlocksDominancePattern:
    """Pattern struct for repeated tree structure."""
    
    def __init__(self, client: BrkClient, acc: str):
        """Create pattern node with accumulated series name."""
        self.blocks_mined: AverageBlockCumulativeSumPattern2 = AverageBlockCumulativeSumPattern2(client, _m(acc, 'blocks_mined'))
        self.dominance: BpsPercentRatioPattern2 = BpsPercentRatioPattern2(client, _m(acc, 'dominance'))

class BpsRatioPattern2:
    """Pattern struct for repeated tree structure."""
    
    def __init__(self, client: BrkClient, acc: str):
        """Create pattern node with accumulated series name."""
        self.bps: SeriesPattern1[BasisPoints32] = SeriesPattern1(client, _m(acc, 'bps'))
        self.ratio: SeriesPattern1[StoredF32] = SeriesPattern1(client, acc)

class BpsRatioPattern:
    """Pattern struct for repeated tree structure."""
    
    def __init__(self, client: BrkClient, acc: str):
        """Create pattern node with accumulated series name."""
        self.bps: SeriesPattern1[BasisPointsSigned32] = SeriesPattern1(client, _m(acc, 'bps'))
        self.ratio: SeriesPattern1[StoredF32] = SeriesPattern1(client, acc)

class BtcSatsPattern:
    """Pattern struct for repeated tree structure."""
    
    def __init__(self, client: BrkClient, acc: str):
        """Create pattern node with accumulated series name."""
        self.btc: SeriesPattern1[Bitcoin] = SeriesPattern1(client, acc)
        self.sats: SeriesPattern1[SatsSigned] = SeriesPattern1(client, _m(acc, 'sats'))

class CentsUsdPattern3:
    """Pattern struct for repeated tree structure."""
    
    def __init__(self, client: BrkClient, acc: str):
        """Create pattern node with accumulated series name."""
        self.cents: SeriesPattern1[Cents] = SeriesPattern1(client, _m(acc, 'cents'))
        self.usd: SeriesPattern1[Dollars] = SeriesPattern1(client, acc)

class CentsUsdPattern2:
    """Pattern struct for repeated tree structure."""
    
    def __init__(self, client: BrkClient, acc: str):
        """Create pattern node with accumulated series name."""
        self.cents: SeriesPattern18[Cents] = SeriesPattern18(client, _m(acc, 'cents'))
        self.usd: SeriesPattern18[Dollars] = SeriesPattern18(client, acc)

class CentsUsdPattern:
    """Pattern struct for repeated tree structure."""
    
    def __init__(self, client: BrkClient, acc: str):
        """Create pattern node with accumulated series name."""
        self.cents: SeriesPattern1[CentsSigned] = SeriesPattern1(client, _m(acc, 'cents'))
        self.usd: SeriesPattern1[Dollars] = SeriesPattern1(client, acc)

class CentsUsdPattern4:
    """Pattern struct for repeated tree structure."""
    
    def __init__(self, client: BrkClient, acc: str):
        """Create pattern node with accumulated series name."""
        self.cents: SeriesPattern18[CentsSigned] = SeriesPattern18(client, _m(acc, 'cents'))
        self.usd: SeriesPattern18[Dollars] = SeriesPattern18(client, acc)

class CoindaysTransferPattern:
    """Pattern struct for repeated tree structure."""
    
    def __init__(self, client: BrkClient, acc: str):
        """Create pattern node with accumulated series name."""
        self.coindays_destroyed: AverageBlockCumulativeSumPattern[StoredF64] = AverageBlockCumulativeSumPattern(client, _m(acc, 'coindays_destroyed'))
        self.transfer_volume: AverageBlockCumulativeInSumPattern = AverageBlockCumulativeInSumPattern(client, _m(acc, 'transfer_volume'))

class FundedTotalPattern:
    """Pattern struct for repeated tree structure."""
    
    def __init__(self, client: BrkClient, acc: str):
        """Create pattern node with accumulated series name."""
        self.funded: AllP2aP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshPattern4 = AllP2aP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshPattern4(client, acc)
        self.total: AllP2aP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshPattern4 = AllP2aP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshPattern4(client, _p('total', acc))

class InPattern2:
    """Pattern struct for repeated tree structure."""
    
    def __init__(self, client: BrkClient, acc: str):
        """Create pattern node with accumulated series name."""
        self.in_loss: CentsUsdPattern3 = CentsUsdPattern3(client, _m(acc, 'loss'))
        self.in_profit: CentsUsdPattern3 = CentsUsdPattern3(client, _m(acc, 'profit'))

class InPattern:
    """Pattern struct for repeated tree structure."""
    
    def __init__(self, client: BrkClient, acc: str):
        """Create pattern node with accumulated series name."""
        self.in_loss: SharePattern = SharePattern(client, _m(acc, 'loss_share'))
        self.in_profit: SharePattern = SharePattern(client, _m(acc, 'profit_share'))

class PerPattern:
    """Pattern struct for repeated tree structure."""
    
    def __init__(self, client: BrkClient, acc: str):
        """Create pattern node with accumulated series name."""
        self.per_coin: CentsSatsUsdPattern = CentsSatsUsdPattern(client, _m(acc, 'coin'))
        self.per_dollar: CentsSatsUsdPattern = CentsSatsUsdPattern(client, _m(acc, 'dollar'))

class PriceRatioPattern:
    """Pattern struct for repeated tree structure."""
    
    def __init__(self, client: BrkClient, acc: str, disc: str):
        """Create pattern node with accumulated series name."""
        self.price: CentsSatsUsdPattern = CentsSatsUsdPattern(client, _m(acc, disc))
        self.ratio: SeriesPattern1[StoredF32] = SeriesPattern1(client, _m(acc, f'ratio_{disc}'))

class RatioValuePattern2:
    """Pattern struct for repeated tree structure."""
    
    def __init__(self, client: BrkClient, acc: str):
        """Create pattern node with accumulated series name."""
        self.ratio: _1m1w1y24hPattern[StoredF64] = _1m1w1y24hPattern(client, _m(acc, 'sopr'))
        self.value_destroyed: AverageBlockCumulativeSumPattern[Cents] = AverageBlockCumulativeSumPattern(client, _m(acc, 'value_destroyed'))

class RatioValuePattern:
    """Pattern struct for repeated tree structure."""
    
    def __init__(self, client: BrkClient, acc: str):
        """Create pattern node with accumulated series name."""
        self.ratio: _24hPattern = _24hPattern(client, _m(acc, 'sopr_24h'))
        self.value_destroyed: AverageBlockCumulativeSumPattern[Cents] = AverageBlockCumulativeSumPattern(client, _m(acc, 'value_destroyed'))

class SdSmaPattern:
    """Pattern struct for repeated tree structure."""
    pass

class ToPattern:
    """Pattern struct for repeated tree structure."""
    
    def __init__(self, client: BrkClient, acc: str):
        """Create pattern node with accumulated series name."""
        self.to_mcap: BpsPercentRatioPattern = BpsPercentRatioPattern(client, _m(acc, 'mcap'))
        self.to_rcap: BpsPercentRatioPattern = BpsPercentRatioPattern(client, _m(acc, 'rcap'))

class _24hPattern:
    """Pattern struct for repeated tree structure."""
    
    def __init__(self, client: BrkClient, acc: str):
        """Create pattern node with accumulated series name."""
        self._24h: SeriesPattern1[StoredF64] = SeriesPattern1(client, acc)

class NuplPattern:
    """Pattern struct for repeated tree structure."""
    
    def __init__(self, client: BrkClient, acc: str):
        """Create pattern node with accumulated series name."""
        self.nupl: BpsRatioPattern = BpsRatioPattern(client, acc)

class PricePattern:
    """Pattern struct for repeated tree structure."""
    
    def __init__(self, client: BrkClient, acc: str):
        """Create pattern node with accumulated series name."""
        self.price: BpsCentsPercentilesRatioSatsUsdPattern = BpsCentsPercentilesRatioSatsUsdPattern(client, acc)

class SharePattern:
    """Pattern struct for repeated tree structure."""
    
    def __init__(self, client: BrkClient, acc: str):
        """Create pattern node with accumulated series name."""
        self.share: BpsPercentRatioPattern2 = BpsPercentRatioPattern2(client, acc)

class TransferPattern:
    """Pattern struct for repeated tree structure."""
    
    def __init__(self, client: BrkClient, acc: str):
        """Create pattern node with accumulated series name."""
        self.transfer_volume: AverageBlockCumulativeSumPattern3 = AverageBlockCumulativeSumPattern3(client, acc)

# Series tree classes

class SeriesTree_Blocks_Difficulty:
    """Series tree node."""
    
    def __init__(self, client: BrkClient, base_path: str = ''):
        self.value: SeriesPattern1[StoredF64] = SeriesPattern1(client, 'difficulty')
        self.hashrate: SeriesPattern1[StoredF64] = SeriesPattern1(client, 'difficulty_hashrate')
        self.adjustment: BpsPercentRatioPattern = BpsPercentRatioPattern(client, 'difficulty_adjustment')
        self.epoch: SeriesPattern1[Epoch] = SeriesPattern1(client, 'difficulty_epoch')
        self.blocks_to_retarget: SeriesPattern1[StoredU32] = SeriesPattern1(client, 'blocks_to_retarget')
        self.days_to_retarget: SeriesPattern1[StoredF32] = SeriesPattern1(client, 'days_to_retarget')

class SeriesTree_Blocks_Time:
    """Series tree node."""
    
    def __init__(self, client: BrkClient, base_path: str = ''):
        self.timestamp: SeriesPattern18[Timestamp] = SeriesPattern18(client, 'timestamp')

class SeriesTree_Blocks_Size:
    """Series tree node."""
    
    def __init__(self, client: BrkClient, base_path: str = ''):
        self.base: SeriesPattern18[StoredU64] = SeriesPattern18(client, 'total_size')
        self.cumulative: SeriesPattern1[StoredU64] = SeriesPattern1(client, 'block_size_cumulative')
        self.sum: _1m1w1y24hPattern[StoredU64] = _1m1w1y24hPattern(client, 'block_size_sum')
        self.average: _1m1w1y24hPattern[StoredF32] = _1m1w1y24hPattern(client, 'block_size_average')
        self.min: _1m1w1y24hPattern[StoredU64] = _1m1w1y24hPattern(client, 'block_size_min')
        self.max: _1m1w1y24hPattern[StoredU64] = _1m1w1y24hPattern(client, 'block_size_max')
        self.pct10: _1m1w1y24hPattern[StoredU64] = _1m1w1y24hPattern(client, 'block_size_pct10')
        self.pct25: _1m1w1y24hPattern[StoredU64] = _1m1w1y24hPattern(client, 'block_size_pct25')
        self.median: _1m1w1y24hPattern[StoredU64] = _1m1w1y24hPattern(client, 'block_size_median')
        self.pct75: _1m1w1y24hPattern[StoredU64] = _1m1w1y24hPattern(client, 'block_size_pct75')
        self.pct90: _1m1w1y24hPattern[StoredU64] = _1m1w1y24hPattern(client, 'block_size_pct90')

class SeriesTree_Blocks_Count:
    """Series tree node."""
    
    def __init__(self, client: BrkClient, base_path: str = ''):
        self.target: _1m1w1y24hPattern[StoredU64] = _1m1w1y24hPattern(client, 'block_count_target')
        self.total: AverageBlockCumulativeSumPattern2 = AverageBlockCumulativeSumPattern2(client, 'block_count')

class SeriesTree_Blocks_Lookback:
    """Series tree node."""
    
    def __init__(self, client: BrkClient, base_path: str = ''):
        self._1h: SeriesPattern18[Height] = SeriesPattern18(client, 'height_1h_ago')
        self._24h: SeriesPattern18[Height] = SeriesPattern18(client, 'height_24h_ago')
        self._3d: SeriesPattern18[Height] = SeriesPattern18(client, 'height_3d_ago')
        self._1w: SeriesPattern18[Height] = SeriesPattern18(client, 'height_1w_ago')
        self._8d: SeriesPattern18[Height] = SeriesPattern18(client, 'height_8d_ago')
        self._9d: SeriesPattern18[Height] = SeriesPattern18(client, 'height_9d_ago')
        self._12d: SeriesPattern18[Height] = SeriesPattern18(client, 'height_12d_ago')
        self._13d: SeriesPattern18[Height] = SeriesPattern18(client, 'height_13d_ago')
        self._2w: SeriesPattern18[Height] = SeriesPattern18(client, 'height_2w_ago')
        self._21d: SeriesPattern18[Height] = SeriesPattern18(client, 'height_21d_ago')
        self._26d: SeriesPattern18[Height] = SeriesPattern18(client, 'height_26d_ago')
        self._1m: SeriesPattern18[Height] = SeriesPattern18(client, 'height_1m_ago')
        self._34d: SeriesPattern18[Height] = SeriesPattern18(client, 'height_34d_ago')
        self._55d: SeriesPattern18[Height] = SeriesPattern18(client, 'height_55d_ago')
        self._2m: SeriesPattern18[Height] = SeriesPattern18(client, 'height_2m_ago')
        self._9w: SeriesPattern18[Height] = SeriesPattern18(client, 'height_9w_ago')
        self._12w: SeriesPattern18[Height] = SeriesPattern18(client, 'height_12w_ago')
        self._89d: SeriesPattern18[Height] = SeriesPattern18(client, 'height_89d_ago')
        self._3m: SeriesPattern18[Height] = SeriesPattern18(client, 'height_3m_ago')
        self._14w: SeriesPattern18[Height] = SeriesPattern18(client, 'height_14w_ago')
        self._111d: SeriesPattern18[Height] = SeriesPattern18(client, 'height_111d_ago')
        self._144d: SeriesPattern18[Height] = SeriesPattern18(client, 'height_144d_ago')
        self._6m: SeriesPattern18[Height] = SeriesPattern18(client, 'height_6m_ago')
        self._26w: SeriesPattern18[Height] = SeriesPattern18(client, 'height_26w_ago')
        self._200d: SeriesPattern18[Height] = SeriesPattern18(client, 'height_200d_ago')
        self._9m: SeriesPattern18[Height] = SeriesPattern18(client, 'height_9m_ago')
        self._350d: SeriesPattern18[Height] = SeriesPattern18(client, 'height_350d_ago')
        self._12m: SeriesPattern18[Height] = SeriesPattern18(client, 'height_12m_ago')
        self._1y: SeriesPattern18[Height] = SeriesPattern18(client, 'height_1y_ago')
        self._14m: SeriesPattern18[Height] = SeriesPattern18(client, 'height_14m_ago')
        self._2y: SeriesPattern18[Height] = SeriesPattern18(client, 'height_2y_ago')
        self._26m: SeriesPattern18[Height] = SeriesPattern18(client, 'height_26m_ago')
        self._3y: SeriesPattern18[Height] = SeriesPattern18(client, 'height_3y_ago')
        self._200w: SeriesPattern18[Height] = SeriesPattern18(client, 'height_200w_ago')
        self._4y: SeriesPattern18[Height] = SeriesPattern18(client, 'height_4y_ago')
        self._5y: SeriesPattern18[Height] = SeriesPattern18(client, 'height_5y_ago')
        self._6y: SeriesPattern18[Height] = SeriesPattern18(client, 'height_6y_ago')
        self._8y: SeriesPattern18[Height] = SeriesPattern18(client, 'height_8y_ago')
        self._9y: SeriesPattern18[Height] = SeriesPattern18(client, 'height_9y_ago')
        self._10y: SeriesPattern18[Height] = SeriesPattern18(client, 'height_10y_ago')
        self._12y: SeriesPattern18[Height] = SeriesPattern18(client, 'height_12y_ago')
        self._14y: SeriesPattern18[Height] = SeriesPattern18(client, 'height_14y_ago')
        self._26y: SeriesPattern18[Height] = SeriesPattern18(client, 'height_26y_ago')

class SeriesTree_Blocks_Interval:
    """Series tree node."""
    
    def __init__(self, client: BrkClient, base_path: str = ''):
        self.block: SeriesPattern18[Timestamp] = SeriesPattern18(client, 'block_interval')
        self._24h: SeriesPattern1[StoredF32] = SeriesPattern1(client, 'block_interval_average_24h')
        self._1w: SeriesPattern1[StoredF32] = SeriesPattern1(client, 'block_interval_average_1w')
        self._1m: SeriesPattern1[StoredF32] = SeriesPattern1(client, 'block_interval_average_1m')
        self._1y: SeriesPattern1[StoredF32] = SeriesPattern1(client, 'block_interval_average_1y')

class SeriesTree_Blocks_Fullness:
    """Series tree node."""
    
    def __init__(self, client: BrkClient, base_path: str = ''):
        self.bps: SeriesPattern18[BasisPoints16] = SeriesPattern18(client, 'block_fullness_bps')
        self.ratio: SeriesPattern18[StoredF32] = SeriesPattern18(client, 'block_fullness_ratio')
        self.percent: SeriesPattern18[StoredF32] = SeriesPattern18(client, 'block_fullness')

class SeriesTree_Blocks_Halving:
    """Series tree node."""
    
    def __init__(self, client: BrkClient, base_path: str = ''):
        self.epoch: SeriesPattern1[Halving] = SeriesPattern1(client, 'halving_epoch')
        self.blocks_to_halving: SeriesPattern1[StoredU32] = SeriesPattern1(client, 'blocks_to_halving')
        self.days_to_halving: SeriesPattern1[StoredF32] = SeriesPattern1(client, 'days_to_halving')

class SeriesTree_Blocks:
    """Series tree node."""
    
    def __init__(self, client: BrkClient, base_path: str = ''):
        self.blockhash: SeriesPattern18[BlockHash] = SeriesPattern18(client, 'blockhash')
        self.coinbase_tag: SeriesPattern18[CoinbaseTag] = SeriesPattern18(client, 'coinbase_tag')
        self.difficulty: SeriesTree_Blocks_Difficulty = SeriesTree_Blocks_Difficulty(client)
        self.time: SeriesTree_Blocks_Time = SeriesTree_Blocks_Time(client)
        self.size: SeriesTree_Blocks_Size = SeriesTree_Blocks_Size(client)
        self.weight: AverageBaseCumulativeMaxMedianMinPct10Pct25Pct75Pct90SumPattern[Weight] = AverageBaseCumulativeMaxMedianMinPct10Pct25Pct75Pct90SumPattern(client, 'block_weight')
        self.segwit_txs: SeriesPattern18[StoredU32] = SeriesPattern18(client, 'segwit_txs')
        self.segwit_size: SeriesPattern18[StoredU64] = SeriesPattern18(client, 'segwit_size')
        self.segwit_weight: SeriesPattern18[Weight] = SeriesPattern18(client, 'segwit_weight')
        self.count: SeriesTree_Blocks_Count = SeriesTree_Blocks_Count(client)
        self.lookback: SeriesTree_Blocks_Lookback = SeriesTree_Blocks_Lookback(client)
        self.interval: SeriesTree_Blocks_Interval = SeriesTree_Blocks_Interval(client)
        self.vbytes: AverageBlockCumulativeMaxMedianMinPct10Pct25Pct75Pct90SumPattern = AverageBlockCumulativeMaxMedianMinPct10Pct25Pct75Pct90SumPattern(client, 'block_vbytes')
        self.fullness: SeriesTree_Blocks_Fullness = SeriesTree_Blocks_Fullness(client)
        self.halving: SeriesTree_Blocks_Halving = SeriesTree_Blocks_Halving(client)

class SeriesTree_Transactions_Raw:
    """Series tree node."""
    
    def __init__(self, client: BrkClient, base_path: str = ''):
        self.first_tx_index: SeriesPattern18[TxIndex] = SeriesPattern18(client, 'first_tx_index')
        self.txid: SeriesPattern19[Txid] = SeriesPattern19(client, 'txid')
        self.tx_version: SeriesPattern19[TxVersion] = SeriesPattern19(client, 'tx_version')
        self.raw_locktime: SeriesPattern19[RawLockTime] = SeriesPattern19(client, 'raw_locktime')
        self.base_size: SeriesPattern19[StoredU32] = SeriesPattern19(client, 'base_size')
        self.total_size: SeriesPattern19[StoredU32] = SeriesPattern19(client, 'total_size')
        self.total_sigop_cost: SeriesPattern19[SigOps] = SeriesPattern19(client, 'total_sigop_cost')
        self.is_explicitly_rbf: SeriesPattern19[StoredBool] = SeriesPattern19(client, 'is_explicitly_rbf')
        self.first_txin_index: SeriesPattern19[TxInIndex] = SeriesPattern19(client, 'first_txin_index')
        self.first_txout_index: SeriesPattern19[TxOutIndex] = SeriesPattern19(client, 'first_txout_index')

class SeriesTree_Transactions_Count:
    """Series tree node."""
    
    def __init__(self, client: BrkClient, base_path: str = ''):
        self.total: AverageBlockCumulativeMaxMedianMinPct10Pct25Pct75Pct90SumPattern = AverageBlockCumulativeMaxMedianMinPct10Pct25Pct75Pct90SumPattern(client, 'tx_count')

class SeriesTree_Transactions_Size_Weight:
    """Series tree node."""
    
    def __init__(self, client: BrkClient, base_path: str = ''):
        self.tx_index: SeriesPattern19[Weight] = SeriesPattern19(client, 'tx_weight')
        self.block: MaxMedianMinPct10Pct25Pct75Pct90Pattern2 = MaxMedianMinPct10Pct25Pct75Pct90Pattern2(client, 'tx_weight')
        self._6b: MaxMedianMinPct10Pct25Pct75Pct90Pattern2 = MaxMedianMinPct10Pct25Pct75Pct90Pattern2(client, 'tx_weight_6b')

class SeriesTree_Transactions_Size:
    """Series tree node."""
    
    def __init__(self, client: BrkClient, base_path: str = ''):
        self.vsize: _6bBlockTxPattern[VSize] = _6bBlockTxPattern(client, 'tx_vsize')
        self.weight: SeriesTree_Transactions_Size_Weight = SeriesTree_Transactions_Size_Weight(client)

class SeriesTree_Transactions_Fees:
    """Series tree node."""
    
    def __init__(self, client: BrkClient, base_path: str = ''):
        self.input_value: SeriesPattern19[Sats] = SeriesPattern19(client, 'input_value')
        self.output_value: SeriesPattern19[Sats] = SeriesPattern19(client, 'output_value')
        self.fee: _6bBlockTxPattern[Sats] = _6bBlockTxPattern(client, 'fee')
        self.fee_rate: SeriesPattern19[FeeRate] = SeriesPattern19(client, 'fee_rate')
        self.effective_fee_rate: _6bBlockTxPattern[FeeRate] = _6bBlockTxPattern(client, 'effective_fee_rate')

class SeriesTree_Transactions_Versions:
    """Series tree node."""
    
    def __init__(self, client: BrkClient, base_path: str = ''):
        self.v1: AverageBlockCumulativeSumPattern[StoredU64] = AverageBlockCumulativeSumPattern(client, 'tx_v1')
        self.v2: AverageBlockCumulativeSumPattern[StoredU64] = AverageBlockCumulativeSumPattern(client, 'tx_v2')
        self.v3: AverageBlockCumulativeSumPattern[StoredU64] = AverageBlockCumulativeSumPattern(client, 'tx_v3')

class SeriesTree_Transactions_Volume:
    """Series tree node."""
    
    def __init__(self, client: BrkClient, base_path: str = ''):
        self.transfer_volume: AverageBlockCumulativeSumPattern3 = AverageBlockCumulativeSumPattern3(client, 'transfer_volume_bis')
        self.tx_per_sec: _1m1w1y24hPattern[StoredF32] = _1m1w1y24hPattern(client, 'tx_per_sec')

class SeriesTree_Transactions:
    """Series tree node."""
    
    def __init__(self, client: BrkClient, base_path: str = ''):
        self.raw: SeriesTree_Transactions_Raw = SeriesTree_Transactions_Raw(client)
        self.count: SeriesTree_Transactions_Count = SeriesTree_Transactions_Count(client)
        self.size: SeriesTree_Transactions_Size = SeriesTree_Transactions_Size(client)
        self.fees: SeriesTree_Transactions_Fees = SeriesTree_Transactions_Fees(client)
        self.versions: SeriesTree_Transactions_Versions = SeriesTree_Transactions_Versions(client)
        self.volume: SeriesTree_Transactions_Volume = SeriesTree_Transactions_Volume(client)

class SeriesTree_Inputs_Raw:
    """Series tree node."""
    
    def __init__(self, client: BrkClient, base_path: str = ''):
        self.first_txin_index: SeriesPattern18[TxInIndex] = SeriesPattern18(client, 'first_txin_index')
        self.outpoint: SeriesPattern20[OutPoint] = SeriesPattern20(client, 'outpoint')
        self.tx_index: SeriesPattern20[TxIndex] = SeriesPattern20(client, 'tx_index')
        self.output_type: SeriesPattern20[OutputType] = SeriesPattern20(client, 'output_type')
        self.type_index: SeriesPattern20[TypeIndex] = SeriesPattern20(client, 'type_index')

class SeriesTree_Inputs_Spent:
    """Series tree node."""
    
    def __init__(self, client: BrkClient, base_path: str = ''):
        self.txout_index: SeriesPattern20[TxOutIndex] = SeriesPattern20(client, 'txout_index')
        self.value: SeriesPattern20[Sats] = SeriesPattern20(client, 'value')

class SeriesTree_Inputs_ByType_InputCount:
    """Series tree node."""
    
    def __init__(self, client: BrkClient, base_path: str = ''):
        self.all: AverageBlockCumulativeSumPattern[StoredU64] = AverageBlockCumulativeSumPattern(client, 'input_count_bis')
        self.p2pk65: AverageBlockCumulativeSumPattern[StoredU64] = AverageBlockCumulativeSumPattern(client, 'p2pk65_prevout_count')
        self.p2pk33: AverageBlockCumulativeSumPattern[StoredU64] = AverageBlockCumulativeSumPattern(client, 'p2pk33_prevout_count')
        self.p2pkh: AverageBlockCumulativeSumPattern[StoredU64] = AverageBlockCumulativeSumPattern(client, 'p2pkh_prevout_count')
        self.p2ms: AverageBlockCumulativeSumPattern[StoredU64] = AverageBlockCumulativeSumPattern(client, 'p2ms_prevout_count')
        self.p2sh: AverageBlockCumulativeSumPattern[StoredU64] = AverageBlockCumulativeSumPattern(client, 'p2sh_prevout_count')
        self.p2wpkh: AverageBlockCumulativeSumPattern[StoredU64] = AverageBlockCumulativeSumPattern(client, 'p2wpkh_prevout_count')
        self.p2wsh: AverageBlockCumulativeSumPattern[StoredU64] = AverageBlockCumulativeSumPattern(client, 'p2wsh_prevout_count')
        self.p2tr: AverageBlockCumulativeSumPattern[StoredU64] = AverageBlockCumulativeSumPattern(client, 'p2tr_prevout_count')
        self.p2a: AverageBlockCumulativeSumPattern[StoredU64] = AverageBlockCumulativeSumPattern(client, 'p2a_prevout_count')
        self.unknown: AverageBlockCumulativeSumPattern[StoredU64] = AverageBlockCumulativeSumPattern(client, 'unknown_outputs_prevout_count')
        self.empty: AverageBlockCumulativeSumPattern[StoredU64] = AverageBlockCumulativeSumPattern(client, 'empty_outputs_prevout_count')

class SeriesTree_Inputs_ByType_InputShare:
    """Series tree node."""
    
    def __init__(self, client: BrkClient, base_path: str = ''):
        self.p2pk65: _1m1w1y24hBpsPercentRatioPattern = _1m1w1y24hBpsPercentRatioPattern(client, 'p2pk65_prevout_share')
        self.p2pk33: _1m1w1y24hBpsPercentRatioPattern = _1m1w1y24hBpsPercentRatioPattern(client, 'p2pk33_prevout_share')
        self.p2pkh: _1m1w1y24hBpsPercentRatioPattern = _1m1w1y24hBpsPercentRatioPattern(client, 'p2pkh_prevout_share')
        self.p2ms: _1m1w1y24hBpsPercentRatioPattern = _1m1w1y24hBpsPercentRatioPattern(client, 'p2ms_prevout_share')
        self.p2sh: _1m1w1y24hBpsPercentRatioPattern = _1m1w1y24hBpsPercentRatioPattern(client, 'p2sh_prevout_share')
        self.p2wpkh: _1m1w1y24hBpsPercentRatioPattern = _1m1w1y24hBpsPercentRatioPattern(client, 'p2wpkh_prevout_share')
        self.p2wsh: _1m1w1y24hBpsPercentRatioPattern = _1m1w1y24hBpsPercentRatioPattern(client, 'p2wsh_prevout_share')
        self.p2tr: _1m1w1y24hBpsPercentRatioPattern = _1m1w1y24hBpsPercentRatioPattern(client, 'p2tr_prevout_share')
        self.p2a: _1m1w1y24hBpsPercentRatioPattern = _1m1w1y24hBpsPercentRatioPattern(client, 'p2a_prevout_share')
        self.unknown: _1m1w1y24hBpsPercentRatioPattern = _1m1w1y24hBpsPercentRatioPattern(client, 'unknown_outputs_prevout_share')
        self.empty: _1m1w1y24hBpsPercentRatioPattern = _1m1w1y24hBpsPercentRatioPattern(client, 'empty_outputs_prevout_share')

class SeriesTree_Inputs_ByType_TxCount:
    """Series tree node."""
    
    def __init__(self, client: BrkClient, base_path: str = ''):
        self.all: AverageBlockCumulativeSumPattern[StoredU64] = AverageBlockCumulativeSumPattern(client, 'non_coinbase_tx_count')
        self.p2pk65: AverageBlockCumulativeSumPattern[StoredU64] = AverageBlockCumulativeSumPattern(client, 'tx_count_with_p2pk65_prevout')
        self.p2pk33: AverageBlockCumulativeSumPattern[StoredU64] = AverageBlockCumulativeSumPattern(client, 'tx_count_with_p2pk33_prevout')
        self.p2pkh: AverageBlockCumulativeSumPattern[StoredU64] = AverageBlockCumulativeSumPattern(client, 'tx_count_with_p2pkh_prevout')
        self.p2ms: AverageBlockCumulativeSumPattern[StoredU64] = AverageBlockCumulativeSumPattern(client, 'tx_count_with_p2ms_prevout')
        self.p2sh: AverageBlockCumulativeSumPattern[StoredU64] = AverageBlockCumulativeSumPattern(client, 'tx_count_with_p2sh_prevout')
        self.p2wpkh: AverageBlockCumulativeSumPattern[StoredU64] = AverageBlockCumulativeSumPattern(client, 'tx_count_with_p2wpkh_prevout')
        self.p2wsh: AverageBlockCumulativeSumPattern[StoredU64] = AverageBlockCumulativeSumPattern(client, 'tx_count_with_p2wsh_prevout')
        self.p2tr: AverageBlockCumulativeSumPattern[StoredU64] = AverageBlockCumulativeSumPattern(client, 'tx_count_with_p2tr_prevout')
        self.p2a: AverageBlockCumulativeSumPattern[StoredU64] = AverageBlockCumulativeSumPattern(client, 'tx_count_with_p2a_prevout')
        self.unknown: AverageBlockCumulativeSumPattern[StoredU64] = AverageBlockCumulativeSumPattern(client, 'tx_count_with_unknown_outputs_prevout')
        self.empty: AverageBlockCumulativeSumPattern[StoredU64] = AverageBlockCumulativeSumPattern(client, 'tx_count_with_empty_outputs_prevout')

class SeriesTree_Inputs_ByType:
    """Series tree node."""
    
    def __init__(self, client: BrkClient, base_path: str = ''):
        self.input_count: SeriesTree_Inputs_ByType_InputCount = SeriesTree_Inputs_ByType_InputCount(client)
        self.input_share: SeriesTree_Inputs_ByType_InputShare = SeriesTree_Inputs_ByType_InputShare(client)
        self.tx_count: SeriesTree_Inputs_ByType_TxCount = SeriesTree_Inputs_ByType_TxCount(client)
        self.tx_share: EmptyP2aP2msP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshUnknownPattern2 = EmptyP2aP2msP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshUnknownPattern2(client, 'tx_share_with')

class SeriesTree_Inputs:
    """Series tree node."""
    
    def __init__(self, client: BrkClient, base_path: str = ''):
        self.raw: SeriesTree_Inputs_Raw = SeriesTree_Inputs_Raw(client)
        self.spent: SeriesTree_Inputs_Spent = SeriesTree_Inputs_Spent(client)
        self.count: CumulativeRollingSumPattern = CumulativeRollingSumPattern(client, 'input_count')
        self.per_sec: _1m1w1y24hPattern[StoredF32] = _1m1w1y24hPattern(client, 'inputs_per_sec')
        self.by_type: SeriesTree_Inputs_ByType = SeriesTree_Inputs_ByType(client)

class SeriesTree_Outputs_Raw:
    """Series tree node."""
    
    def __init__(self, client: BrkClient, base_path: str = ''):
        self.first_txout_index: SeriesPattern18[TxOutIndex] = SeriesPattern18(client, 'first_txout_index')
        self.value: SeriesPattern21[Sats] = SeriesPattern21(client, 'value')
        self.output_type: SeriesPattern21[OutputType] = SeriesPattern21(client, 'output_type')
        self.type_index: SeriesPattern21[TypeIndex] = SeriesPattern21(client, 'type_index')
        self.tx_index: SeriesPattern21[TxIndex] = SeriesPattern21(client, 'tx_index')

class SeriesTree_Outputs_Spent:
    """Series tree node."""
    
    def __init__(self, client: BrkClient, base_path: str = ''):
        self.txin_index: SeriesPattern21[TxInIndex] = SeriesPattern21(client, 'txin_index')

class SeriesTree_Outputs_Count:
    """Series tree node."""
    
    def __init__(self, client: BrkClient, base_path: str = ''):
        self.total: CumulativeRollingSumPattern = CumulativeRollingSumPattern(client, 'output_count')

class SeriesTree_Outputs_Unspent:
    """Series tree node."""
    
    def __init__(self, client: BrkClient, base_path: str = ''):
        self.count: SeriesPattern1[StoredU64] = SeriesPattern1(client, 'utxo_count_bis')

class SeriesTree_Outputs_ByType_OutputCount:
    """Series tree node."""
    
    def __init__(self, client: BrkClient, base_path: str = ''):
        self.all: AverageBlockCumulativeSumPattern[StoredU64] = AverageBlockCumulativeSumPattern(client, 'output_count_bis')
        self.p2pk65: AverageBlockCumulativeSumPattern[StoredU64] = AverageBlockCumulativeSumPattern(client, 'p2pk65_output_count')
        self.p2pk33: AverageBlockCumulativeSumPattern[StoredU64] = AverageBlockCumulativeSumPattern(client, 'p2pk33_output_count')
        self.p2pkh: AverageBlockCumulativeSumPattern[StoredU64] = AverageBlockCumulativeSumPattern(client, 'p2pkh_output_count')
        self.p2ms: AverageBlockCumulativeSumPattern[StoredU64] = AverageBlockCumulativeSumPattern(client, 'p2ms_output_count')
        self.p2sh: AverageBlockCumulativeSumPattern[StoredU64] = AverageBlockCumulativeSumPattern(client, 'p2sh_output_count')
        self.p2wpkh: AverageBlockCumulativeSumPattern[StoredU64] = AverageBlockCumulativeSumPattern(client, 'p2wpkh_output_count')
        self.p2wsh: AverageBlockCumulativeSumPattern[StoredU64] = AverageBlockCumulativeSumPattern(client, 'p2wsh_output_count')
        self.p2tr: AverageBlockCumulativeSumPattern[StoredU64] = AverageBlockCumulativeSumPattern(client, 'p2tr_output_count')
        self.p2a: AverageBlockCumulativeSumPattern[StoredU64] = AverageBlockCumulativeSumPattern(client, 'p2a_output_count')
        self.unknown: AverageBlockCumulativeSumPattern[StoredU64] = AverageBlockCumulativeSumPattern(client, 'unknown_outputs_output_count')
        self.empty: AverageBlockCumulativeSumPattern[StoredU64] = AverageBlockCumulativeSumPattern(client, 'empty_outputs_output_count')
        self.op_return: AverageBlockCumulativeSumPattern[StoredU64] = AverageBlockCumulativeSumPattern(client, 'op_return_output_count')

class SeriesTree_Outputs_ByType_OutputShare:
    """Series tree node."""
    
    def __init__(self, client: BrkClient, base_path: str = ''):
        self.p2pk65: _1m1w1y24hBpsPercentRatioPattern = _1m1w1y24hBpsPercentRatioPattern(client, 'p2pk65_output_share')
        self.p2pk33: _1m1w1y24hBpsPercentRatioPattern = _1m1w1y24hBpsPercentRatioPattern(client, 'p2pk33_output_share')
        self.p2pkh: _1m1w1y24hBpsPercentRatioPattern = _1m1w1y24hBpsPercentRatioPattern(client, 'p2pkh_output_share')
        self.p2ms: _1m1w1y24hBpsPercentRatioPattern = _1m1w1y24hBpsPercentRatioPattern(client, 'p2ms_output_share')
        self.p2sh: _1m1w1y24hBpsPercentRatioPattern = _1m1w1y24hBpsPercentRatioPattern(client, 'p2sh_output_share')
        self.p2wpkh: _1m1w1y24hBpsPercentRatioPattern = _1m1w1y24hBpsPercentRatioPattern(client, 'p2wpkh_output_share')
        self.p2wsh: _1m1w1y24hBpsPercentRatioPattern = _1m1w1y24hBpsPercentRatioPattern(client, 'p2wsh_output_share')
        self.p2tr: _1m1w1y24hBpsPercentRatioPattern = _1m1w1y24hBpsPercentRatioPattern(client, 'p2tr_output_share')
        self.p2a: _1m1w1y24hBpsPercentRatioPattern = _1m1w1y24hBpsPercentRatioPattern(client, 'p2a_output_share')
        self.unknown: _1m1w1y24hBpsPercentRatioPattern = _1m1w1y24hBpsPercentRatioPattern(client, 'unknown_outputs_output_share')
        self.empty: _1m1w1y24hBpsPercentRatioPattern = _1m1w1y24hBpsPercentRatioPattern(client, 'empty_outputs_output_share')
        self.op_return: _1m1w1y24hBpsPercentRatioPattern = _1m1w1y24hBpsPercentRatioPattern(client, 'op_return_output_share')

class SeriesTree_Outputs_ByType:
    """Series tree node."""
    
    def __init__(self, client: BrkClient, base_path: str = ''):
        self.output_count: SeriesTree_Outputs_ByType_OutputCount = SeriesTree_Outputs_ByType_OutputCount(client)
        self.spendable_output_count: AverageBlockCumulativeSumPattern[StoredU64] = AverageBlockCumulativeSumPattern(client, 'spendable_output_count')
        self.output_share: SeriesTree_Outputs_ByType_OutputShare = SeriesTree_Outputs_ByType_OutputShare(client)
        self.tx_count: AllEmptyOpP2aP2msP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshUnknownPattern = AllEmptyOpP2aP2msP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshUnknownPattern(client, 'tx_count')
        self.tx_share: EmptyOpP2aP2msP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshUnknownPattern2 = EmptyOpP2aP2msP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshUnknownPattern2(client, 'tx_share_with')

class SeriesTree_Outputs_Value:
    """Series tree node."""
    
    def __init__(self, client: BrkClient, base_path: str = ''):
        self.op_return: BlockCumulativePattern = BlockCumulativePattern(client, 'op_return_value')

class SeriesTree_Outputs:
    """Series tree node."""
    
    def __init__(self, client: BrkClient, base_path: str = ''):
        self.raw: SeriesTree_Outputs_Raw = SeriesTree_Outputs_Raw(client)
        self.spent: SeriesTree_Outputs_Spent = SeriesTree_Outputs_Spent(client)
        self.count: SeriesTree_Outputs_Count = SeriesTree_Outputs_Count(client)
        self.per_sec: _1m1w1y24hPattern[StoredF32] = _1m1w1y24hPattern(client, 'outputs_per_sec')
        self.unspent: SeriesTree_Outputs_Unspent = SeriesTree_Outputs_Unspent(client)
        self.by_type: SeriesTree_Outputs_ByType = SeriesTree_Outputs_ByType(client)
        self.value: SeriesTree_Outputs_Value = SeriesTree_Outputs_Value(client)

class SeriesTree_Addrs_Raw_P2pk65:
    """Series tree node."""
    
    def __init__(self, client: BrkClient, base_path: str = ''):
        self.first_index: SeriesPattern18[P2PK65AddrIndex] = SeriesPattern18(client, 'first_p2pk65_addr_index')
        self.bytes: SeriesPattern27[P2PK65Bytes] = SeriesPattern27(client, 'p2pk65_bytes')

class SeriesTree_Addrs_Raw_P2pk33:
    """Series tree node."""
    
    def __init__(self, client: BrkClient, base_path: str = ''):
        self.first_index: SeriesPattern18[P2PK33AddrIndex] = SeriesPattern18(client, 'first_p2pk33_addr_index')
        self.bytes: SeriesPattern26[P2PK33Bytes] = SeriesPattern26(client, 'p2pk33_bytes')

class SeriesTree_Addrs_Raw_P2pkh:
    """Series tree node."""
    
    def __init__(self, client: BrkClient, base_path: str = ''):
        self.first_index: SeriesPattern18[P2PKHAddrIndex] = SeriesPattern18(client, 'first_p2pkh_addr_index')
        self.bytes: SeriesPattern28[P2PKHBytes] = SeriesPattern28(client, 'p2pkh_bytes')

class SeriesTree_Addrs_Raw_P2sh:
    """Series tree node."""
    
    def __init__(self, client: BrkClient, base_path: str = ''):
        self.first_index: SeriesPattern18[P2SHAddrIndex] = SeriesPattern18(client, 'first_p2sh_addr_index')
        self.bytes: SeriesPattern29[P2SHBytes] = SeriesPattern29(client, 'p2sh_bytes')

class SeriesTree_Addrs_Raw_P2wpkh:
    """Series tree node."""
    
    def __init__(self, client: BrkClient, base_path: str = ''):
        self.first_index: SeriesPattern18[P2WPKHAddrIndex] = SeriesPattern18(client, 'first_p2wpkh_addr_index')
        self.bytes: SeriesPattern31[P2WPKHBytes] = SeriesPattern31(client, 'p2wpkh_bytes')

class SeriesTree_Addrs_Raw_P2wsh:
    """Series tree node."""
    
    def __init__(self, client: BrkClient, base_path: str = ''):
        self.first_index: SeriesPattern18[P2WSHAddrIndex] = SeriesPattern18(client, 'first_p2wsh_addr_index')
        self.bytes: SeriesPattern32[P2WSHBytes] = SeriesPattern32(client, 'p2wsh_bytes')

class SeriesTree_Addrs_Raw_P2tr:
    """Series tree node."""
    
    def __init__(self, client: BrkClient, base_path: str = ''):
        self.first_index: SeriesPattern18[P2TRAddrIndex] = SeriesPattern18(client, 'first_p2tr_addr_index')
        self.bytes: SeriesPattern30[P2TRBytes] = SeriesPattern30(client, 'p2tr_bytes')

class SeriesTree_Addrs_Raw_P2a:
    """Series tree node."""
    
    def __init__(self, client: BrkClient, base_path: str = ''):
        self.first_index: SeriesPattern18[P2AAddrIndex] = SeriesPattern18(client, 'first_p2a_addr_index')
        self.bytes: SeriesPattern24[P2ABytes] = SeriesPattern24(client, 'p2a_bytes')

class SeriesTree_Addrs_Raw:
    """Series tree node."""
    
    def __init__(self, client: BrkClient, base_path: str = ''):
        self.p2pk65: SeriesTree_Addrs_Raw_P2pk65 = SeriesTree_Addrs_Raw_P2pk65(client)
        self.p2pk33: SeriesTree_Addrs_Raw_P2pk33 = SeriesTree_Addrs_Raw_P2pk33(client)
        self.p2pkh: SeriesTree_Addrs_Raw_P2pkh = SeriesTree_Addrs_Raw_P2pkh(client)
        self.p2sh: SeriesTree_Addrs_Raw_P2sh = SeriesTree_Addrs_Raw_P2sh(client)
        self.p2wpkh: SeriesTree_Addrs_Raw_P2wpkh = SeriesTree_Addrs_Raw_P2wpkh(client)
        self.p2wsh: SeriesTree_Addrs_Raw_P2wsh = SeriesTree_Addrs_Raw_P2wsh(client)
        self.p2tr: SeriesTree_Addrs_Raw_P2tr = SeriesTree_Addrs_Raw_P2tr(client)
        self.p2a: SeriesTree_Addrs_Raw_P2a = SeriesTree_Addrs_Raw_P2a(client)

class SeriesTree_Addrs_Indexes:
    """Series tree node."""
    
    def __init__(self, client: BrkClient, base_path: str = ''):
        self.p2a: SeriesPattern24[AnyAddrIndex] = SeriesPattern24(client, 'any_addr_index')
        self.p2pk33: SeriesPattern26[AnyAddrIndex] = SeriesPattern26(client, 'any_addr_index')
        self.p2pk65: SeriesPattern27[AnyAddrIndex] = SeriesPattern27(client, 'any_addr_index')
        self.p2pkh: SeriesPattern28[AnyAddrIndex] = SeriesPattern28(client, 'any_addr_index')
        self.p2sh: SeriesPattern29[AnyAddrIndex] = SeriesPattern29(client, 'any_addr_index')
        self.p2tr: SeriesPattern30[AnyAddrIndex] = SeriesPattern30(client, 'any_addr_index')
        self.p2wpkh: SeriesPattern31[AnyAddrIndex] = SeriesPattern31(client, 'any_addr_index')
        self.p2wsh: SeriesPattern32[AnyAddrIndex] = SeriesPattern32(client, 'any_addr_index')
        self.funded: SeriesPattern34[FundedAddrIndex] = SeriesPattern34(client, 'funded_addr_index')
        self.empty: SeriesPattern35[EmptyAddrIndex] = SeriesPattern35(client, 'empty_addr_index')

class SeriesTree_Addrs_Data:
    """Series tree node."""
    
    def __init__(self, client: BrkClient, base_path: str = ''):
        self.funded: SeriesPattern34[FundedAddrData] = SeriesPattern34(client, 'funded_addr_data')
        self.empty: SeriesPattern35[EmptyAddrData] = SeriesPattern35(client, 'empty_addr_data')

class SeriesTree_Addrs_Activity_All:
    """Series tree node."""
    
    def __init__(self, client: BrkClient, base_path: str = ''):
        self.reactivated: _1m1w1y24hBlockPattern = _1m1w1y24hBlockPattern(client, 'reactivated_addrs')
        self.sending: _1m1w1y24hBlockPattern = _1m1w1y24hBlockPattern(client, 'sending_addrs')
        self.receiving: _1m1w1y24hBlockPattern = _1m1w1y24hBlockPattern(client, 'receiving_addrs')
        self.bidirectional: _1m1w1y24hBlockPattern = _1m1w1y24hBlockPattern(client, 'bidirectional_addrs')
        self.active: _1m1w1y24hBlockPattern = _1m1w1y24hBlockPattern(client, 'active_addrs')

class SeriesTree_Addrs_Activity:
    """Series tree node."""
    
    def __init__(self, client: BrkClient, base_path: str = ''):
        self.all: SeriesTree_Addrs_Activity_All = SeriesTree_Addrs_Activity_All(client)
        self.p2pk65: ActiveBidirectionalReactivatedReceivingSendingPattern = ActiveBidirectionalReactivatedReceivingSendingPattern(client, 'p2pk65')
        self.p2pk33: ActiveBidirectionalReactivatedReceivingSendingPattern = ActiveBidirectionalReactivatedReceivingSendingPattern(client, 'p2pk33')
        self.p2pkh: ActiveBidirectionalReactivatedReceivingSendingPattern = ActiveBidirectionalReactivatedReceivingSendingPattern(client, 'p2pkh')
        self.p2sh: ActiveBidirectionalReactivatedReceivingSendingPattern = ActiveBidirectionalReactivatedReceivingSendingPattern(client, 'p2sh')
        self.p2wpkh: ActiveBidirectionalReactivatedReceivingSendingPattern = ActiveBidirectionalReactivatedReceivingSendingPattern(client, 'p2wpkh')
        self.p2wsh: ActiveBidirectionalReactivatedReceivingSendingPattern = ActiveBidirectionalReactivatedReceivingSendingPattern(client, 'p2wsh')
        self.p2tr: ActiveBidirectionalReactivatedReceivingSendingPattern = ActiveBidirectionalReactivatedReceivingSendingPattern(client, 'p2tr')
        self.p2a: ActiveBidirectionalReactivatedReceivingSendingPattern = ActiveBidirectionalReactivatedReceivingSendingPattern(client, 'p2a')

class SeriesTree_Addrs_Reused_Events:
    """Series tree node."""
    
    def __init__(self, client: BrkClient, base_path: str = ''):
        self.output_to_reused_addr_count: AllP2aP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshPattern6 = AllP2aP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshPattern6(client, 'output_to_reused_addr_count')
        self.output_to_reused_addr_share: AllP2aP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshPattern7 = AllP2aP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshPattern7(client, 'output_to_reused_addr_share')
        self.spendable_output_to_reused_addr_share: _1m1w1y24hBpsPercentRatioPattern = _1m1w1y24hBpsPercentRatioPattern(client, 'spendable_output_to_reused_addr_share')
        self.input_from_reused_addr_count: AllP2aP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshPattern6 = AllP2aP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshPattern6(client, 'input_from_reused_addr_count')
        self.input_from_reused_addr_share: AllP2aP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshPattern7 = AllP2aP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshPattern7(client, 'input_from_reused_addr_share')
        self.active_reused_addr_count: _1m1w1y24hBlockPattern = _1m1w1y24hBlockPattern(client, 'active_reused_addr_count')
        self.active_reused_addr_share: _1m1w1y24hBlockPattern2 = _1m1w1y24hBlockPattern2(client, 'active_reused_addr_share')

class SeriesTree_Addrs_Reused_Supply:
    """Series tree node."""
    
    def __init__(self, client: BrkClient, base_path: str = ''):
        self.all: BtcCentsSatsUsdPattern = BtcCentsSatsUsdPattern(client, 'reused_addr_supply')
        self.p2pk65: BtcCentsSatsUsdPattern = BtcCentsSatsUsdPattern(client, 'p2pk65_reused_addr_supply')
        self.p2pk33: BtcCentsSatsUsdPattern = BtcCentsSatsUsdPattern(client, 'p2pk33_reused_addr_supply')
        self.p2pkh: BtcCentsSatsUsdPattern = BtcCentsSatsUsdPattern(client, 'p2pkh_reused_addr_supply')
        self.p2sh: BtcCentsSatsUsdPattern = BtcCentsSatsUsdPattern(client, 'p2sh_reused_addr_supply')
        self.p2wpkh: BtcCentsSatsUsdPattern = BtcCentsSatsUsdPattern(client, 'p2wpkh_reused_addr_supply')
        self.p2wsh: BtcCentsSatsUsdPattern = BtcCentsSatsUsdPattern(client, 'p2wsh_reused_addr_supply')
        self.p2tr: BtcCentsSatsUsdPattern = BtcCentsSatsUsdPattern(client, 'p2tr_reused_addr_supply')
        self.p2a: BtcCentsSatsUsdPattern = BtcCentsSatsUsdPattern(client, 'p2a_reused_addr_supply')
        self.share: AllP2aP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshPattern5 = AllP2aP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshPattern5(client, 'reused_addr_supply_share')

class SeriesTree_Addrs_Reused:
    """Series tree node."""
    
    def __init__(self, client: BrkClient, base_path: str = ''):
        self.count: FundedTotalPattern = FundedTotalPattern(client, 'reused_addr_count')
        self.events: SeriesTree_Addrs_Reused_Events = SeriesTree_Addrs_Reused_Events(client)
        self.supply: SeriesTree_Addrs_Reused_Supply = SeriesTree_Addrs_Reused_Supply(client)

class SeriesTree_Addrs_Respent_Events:
    """Series tree node."""
    
    def __init__(self, client: BrkClient, base_path: str = ''):
        self.output_to_reused_addr_count: AllP2aP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshPattern6 = AllP2aP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshPattern6(client, 'output_to_respent_addr_count')
        self.output_to_reused_addr_share: AllP2aP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshPattern7 = AllP2aP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshPattern7(client, 'output_to_respent_addr_share')
        self.spendable_output_to_reused_addr_share: _1m1w1y24hBpsPercentRatioPattern = _1m1w1y24hBpsPercentRatioPattern(client, 'spendable_output_to_respent_addr_share')
        self.input_from_reused_addr_count: AllP2aP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshPattern6 = AllP2aP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshPattern6(client, 'input_from_respent_addr_count')
        self.input_from_reused_addr_share: AllP2aP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshPattern7 = AllP2aP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshPattern7(client, 'input_from_respent_addr_share')
        self.active_reused_addr_count: _1m1w1y24hBlockPattern = _1m1w1y24hBlockPattern(client, 'active_respent_addr_count')
        self.active_reused_addr_share: _1m1w1y24hBlockPattern2 = _1m1w1y24hBlockPattern2(client, 'active_respent_addr_share')

class SeriesTree_Addrs_Respent_Supply:
    """Series tree node."""
    
    def __init__(self, client: BrkClient, base_path: str = ''):
        self.all: BtcCentsSatsUsdPattern = BtcCentsSatsUsdPattern(client, 'respent_addr_supply')
        self.p2pk65: BtcCentsSatsUsdPattern = BtcCentsSatsUsdPattern(client, 'p2pk65_respent_addr_supply')
        self.p2pk33: BtcCentsSatsUsdPattern = BtcCentsSatsUsdPattern(client, 'p2pk33_respent_addr_supply')
        self.p2pkh: BtcCentsSatsUsdPattern = BtcCentsSatsUsdPattern(client, 'p2pkh_respent_addr_supply')
        self.p2sh: BtcCentsSatsUsdPattern = BtcCentsSatsUsdPattern(client, 'p2sh_respent_addr_supply')
        self.p2wpkh: BtcCentsSatsUsdPattern = BtcCentsSatsUsdPattern(client, 'p2wpkh_respent_addr_supply')
        self.p2wsh: BtcCentsSatsUsdPattern = BtcCentsSatsUsdPattern(client, 'p2wsh_respent_addr_supply')
        self.p2tr: BtcCentsSatsUsdPattern = BtcCentsSatsUsdPattern(client, 'p2tr_respent_addr_supply')
        self.p2a: BtcCentsSatsUsdPattern = BtcCentsSatsUsdPattern(client, 'p2a_respent_addr_supply')
        self.share: AllP2aP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshPattern5 = AllP2aP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshPattern5(client, 'respent_addr_supply_share')

class SeriesTree_Addrs_Respent:
    """Series tree node."""
    
    def __init__(self, client: BrkClient, base_path: str = ''):
        self.count: FundedTotalPattern = FundedTotalPattern(client, 'respent_addr_count')
        self.events: SeriesTree_Addrs_Respent_Events = SeriesTree_Addrs_Respent_Events(client)
        self.supply: SeriesTree_Addrs_Respent_Supply = SeriesTree_Addrs_Respent_Supply(client)

class SeriesTree_Addrs_Exposed_Supply:
    """Series tree node."""
    
    def __init__(self, client: BrkClient, base_path: str = ''):
        self.all: BtcCentsSatsUsdPattern = BtcCentsSatsUsdPattern(client, 'exposed_addr_supply')
        self.p2pk65: BtcCentsSatsUsdPattern = BtcCentsSatsUsdPattern(client, 'p2pk65_exposed_addr_supply')
        self.p2pk33: BtcCentsSatsUsdPattern = BtcCentsSatsUsdPattern(client, 'p2pk33_exposed_addr_supply')
        self.p2pkh: BtcCentsSatsUsdPattern = BtcCentsSatsUsdPattern(client, 'p2pkh_exposed_addr_supply')
        self.p2sh: BtcCentsSatsUsdPattern = BtcCentsSatsUsdPattern(client, 'p2sh_exposed_addr_supply')
        self.p2wpkh: BtcCentsSatsUsdPattern = BtcCentsSatsUsdPattern(client, 'p2wpkh_exposed_addr_supply')
        self.p2wsh: BtcCentsSatsUsdPattern = BtcCentsSatsUsdPattern(client, 'p2wsh_exposed_addr_supply')
        self.p2tr: BtcCentsSatsUsdPattern = BtcCentsSatsUsdPattern(client, 'p2tr_exposed_addr_supply')
        self.p2a: BtcCentsSatsUsdPattern = BtcCentsSatsUsdPattern(client, 'p2a_exposed_addr_supply')
        self.share: AllP2aP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshPattern5 = AllP2aP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshPattern5(client, 'exposed_addr_supply_share')

class SeriesTree_Addrs_Exposed:
    """Series tree node."""
    
    def __init__(self, client: BrkClient, base_path: str = ''):
        self.count: FundedTotalPattern = FundedTotalPattern(client, 'exposed_addr_count')
        self.supply: SeriesTree_Addrs_Exposed_Supply = SeriesTree_Addrs_Exposed_Supply(client)

class SeriesTree_Addrs_Delta:
    """Series tree node."""
    
    def __init__(self, client: BrkClient, base_path: str = ''):
        self.all: AbsoluteRatePattern = AbsoluteRatePattern(client, 'addr_count')
        self.p2pk65: AbsoluteRatePattern = AbsoluteRatePattern(client, 'p2pk65_addr_count')
        self.p2pk33: AbsoluteRatePattern = AbsoluteRatePattern(client, 'p2pk33_addr_count')
        self.p2pkh: AbsoluteRatePattern = AbsoluteRatePattern(client, 'p2pkh_addr_count')
        self.p2sh: AbsoluteRatePattern = AbsoluteRatePattern(client, 'p2sh_addr_count')
        self.p2wpkh: AbsoluteRatePattern = AbsoluteRatePattern(client, 'p2wpkh_addr_count')
        self.p2wsh: AbsoluteRatePattern = AbsoluteRatePattern(client, 'p2wsh_addr_count')
        self.p2tr: AbsoluteRatePattern = AbsoluteRatePattern(client, 'p2tr_addr_count')
        self.p2a: AbsoluteRatePattern = AbsoluteRatePattern(client, 'p2a_addr_count')

class SeriesTree_Addrs_AvgAmount:
    """Series tree node."""
    
    def __init__(self, client: BrkClient, base_path: str = ''):
        self.all: AddrUtxoPattern = AddrUtxoPattern(client, 'avg')
        self.p2pk65: AddrUtxoPattern = AddrUtxoPattern(client, 'p2pk65_avg')
        self.p2pk33: AddrUtxoPattern = AddrUtxoPattern(client, 'p2pk33_avg')
        self.p2pkh: AddrUtxoPattern = AddrUtxoPattern(client, 'p2pkh_avg')
        self.p2sh: AddrUtxoPattern = AddrUtxoPattern(client, 'p2sh_avg')
        self.p2wpkh: AddrUtxoPattern = AddrUtxoPattern(client, 'p2wpkh_avg')
        self.p2wsh: AddrUtxoPattern = AddrUtxoPattern(client, 'p2wsh_avg')
        self.p2tr: AddrUtxoPattern = AddrUtxoPattern(client, 'p2tr_avg')
        self.p2a: AddrUtxoPattern = AddrUtxoPattern(client, 'p2a_avg')

class SeriesTree_Addrs:
    """Series tree node."""
    
    def __init__(self, client: BrkClient, base_path: str = ''):
        self.raw: SeriesTree_Addrs_Raw = SeriesTree_Addrs_Raw(client)
        self.indexes: SeriesTree_Addrs_Indexes = SeriesTree_Addrs_Indexes(client)
        self.data: SeriesTree_Addrs_Data = SeriesTree_Addrs_Data(client)
        self.funded: AllP2aP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshPattern4 = AllP2aP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshPattern4(client, 'addr_count')
        self.empty: AllP2aP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshPattern4 = AllP2aP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshPattern4(client, 'empty_addr_count')
        self.activity: SeriesTree_Addrs_Activity = SeriesTree_Addrs_Activity(client)
        self.total: AllP2aP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshPattern4 = AllP2aP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshPattern4(client, 'total_addr_count')
        self.new: AllP2aP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshPattern6 = AllP2aP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshPattern6(client, 'new_addr_count')
        self.reused: SeriesTree_Addrs_Reused = SeriesTree_Addrs_Reused(client)
        self.respent: SeriesTree_Addrs_Respent = SeriesTree_Addrs_Respent(client)
        self.exposed: SeriesTree_Addrs_Exposed = SeriesTree_Addrs_Exposed(client)
        self.delta: SeriesTree_Addrs_Delta = SeriesTree_Addrs_Delta(client)
        self.avg_amount: SeriesTree_Addrs_AvgAmount = SeriesTree_Addrs_AvgAmount(client)

class SeriesTree_Scripts_Raw_Empty:
    """Series tree node."""
    
    def __init__(self, client: BrkClient, base_path: str = ''):
        self.first_index: SeriesPattern18[EmptyOutputIndex] = SeriesPattern18(client, 'first_empty_output_index')
        self.to_tx_index: SeriesPattern22[TxIndex] = SeriesPattern22(client, 'tx_index')

class SeriesTree_Scripts_Raw_OpReturn:
    """Series tree node."""
    
    def __init__(self, client: BrkClient, base_path: str = ''):
        self.first_index: SeriesPattern18[OpReturnIndex] = SeriesPattern18(client, 'first_op_return_index')
        self.to_tx_index: SeriesPattern23[TxIndex] = SeriesPattern23(client, 'tx_index')

class SeriesTree_Scripts_Raw_P2ms:
    """Series tree node."""
    
    def __init__(self, client: BrkClient, base_path: str = ''):
        self.first_index: SeriesPattern18[P2MSOutputIndex] = SeriesPattern18(client, 'first_p2ms_output_index')
        self.to_tx_index: SeriesPattern25[TxIndex] = SeriesPattern25(client, 'tx_index')

class SeriesTree_Scripts_Raw_Unknown:
    """Series tree node."""
    
    def __init__(self, client: BrkClient, base_path: str = ''):
        self.first_index: SeriesPattern18[UnknownOutputIndex] = SeriesPattern18(client, 'first_unknown_output_index')
        self.to_tx_index: SeriesPattern33[TxIndex] = SeriesPattern33(client, 'tx_index')

class SeriesTree_Scripts_Raw:
    """Series tree node."""
    
    def __init__(self, client: BrkClient, base_path: str = ''):
        self.empty: SeriesTree_Scripts_Raw_Empty = SeriesTree_Scripts_Raw_Empty(client)
        self.op_return: SeriesTree_Scripts_Raw_OpReturn = SeriesTree_Scripts_Raw_OpReturn(client)
        self.p2ms: SeriesTree_Scripts_Raw_P2ms = SeriesTree_Scripts_Raw_P2ms(client)
        self.unknown: SeriesTree_Scripts_Raw_Unknown = SeriesTree_Scripts_Raw_Unknown(client)

class SeriesTree_Scripts:
    """Series tree node."""
    
    def __init__(self, client: BrkClient, base_path: str = ''):
        self.raw: SeriesTree_Scripts_Raw = SeriesTree_Scripts_Raw(client)

class SeriesTree_Mining_Rewards_Subsidy:
    """Series tree node."""
    
    def __init__(self, client: BrkClient, base_path: str = ''):
        self.block: BtcCentsSatsUsdPattern3 = BtcCentsSatsUsdPattern3(client, 'subsidy')
        self.cumulative: BtcCentsSatsUsdPattern = BtcCentsSatsUsdPattern(client, 'subsidy_cumulative')
        self.sum: _1m1w1y24hPattern4 = _1m1w1y24hPattern4(client, 'subsidy_sum')
        self.average: _1m1w1y24hPattern3 = _1m1w1y24hPattern3(client, 'subsidy_average')
        self.dominance: _1m1w1y24hBpsPercentRatioPattern = _1m1w1y24hBpsPercentRatioPattern(client, 'subsidy_dominance')

class SeriesTree_Mining_Rewards_Fees_ToSubsidyRatio:
    """Series tree node."""
    
    def __init__(self, client: BrkClient, base_path: str = ''):
        self._24h: BpsRatioPattern2 = BpsRatioPattern2(client, 'fee_to_subsidy_ratio_24h')
        self._1w: BpsRatioPattern2 = BpsRatioPattern2(client, 'fee_to_subsidy_ratio_1w')
        self._1m: BpsRatioPattern2 = BpsRatioPattern2(client, 'fee_to_subsidy_ratio_1m')
        self._1y: BpsRatioPattern2 = BpsRatioPattern2(client, 'fee_to_subsidy_ratio_1y')

class SeriesTree_Mining_Rewards_Fees:
    """Series tree node."""
    
    def __init__(self, client: BrkClient, base_path: str = ''):
        self.block: BtcCentsSatsUsdPattern3 = BtcCentsSatsUsdPattern3(client, 'fees')
        self.cumulative: BtcCentsSatsUsdPattern = BtcCentsSatsUsdPattern(client, 'fees_cumulative')
        self.sum: _1m1w1y24hPattern4 = _1m1w1y24hPattern4(client, 'fees_sum')
        self.average: _1m1w1y24hPattern3 = _1m1w1y24hPattern3(client, 'fees_average')
        self.min: _1m1w1y24hPattern4 = _1m1w1y24hPattern4(client, 'fees_min')
        self.max: _1m1w1y24hPattern4 = _1m1w1y24hPattern4(client, 'fees_max')
        self.pct10: _1m1w1y24hPattern4 = _1m1w1y24hPattern4(client, 'fees_pct10')
        self.pct25: _1m1w1y24hPattern4 = _1m1w1y24hPattern4(client, 'fees_pct25')
        self.median: _1m1w1y24hPattern4 = _1m1w1y24hPattern4(client, 'fees_median')
        self.pct75: _1m1w1y24hPattern4 = _1m1w1y24hPattern4(client, 'fees_pct75')
        self.pct90: _1m1w1y24hPattern4 = _1m1w1y24hPattern4(client, 'fees_pct90')
        self.dominance: _1m1w1y24hBpsPercentRatioPattern = _1m1w1y24hBpsPercentRatioPattern(client, 'fee_dominance')
        self.to_subsidy_ratio: SeriesTree_Mining_Rewards_Fees_ToSubsidyRatio = SeriesTree_Mining_Rewards_Fees_ToSubsidyRatio(client)

class SeriesTree_Mining_Rewards:
    """Series tree node."""
    
    def __init__(self, client: BrkClient, base_path: str = ''):
        self.coinbase: AverageBlockCumulativeSumPattern3 = AverageBlockCumulativeSumPattern3(client, 'coinbase')
        self.subsidy: SeriesTree_Mining_Rewards_Subsidy = SeriesTree_Mining_Rewards_Subsidy(client)
        self.fees: SeriesTree_Mining_Rewards_Fees = SeriesTree_Mining_Rewards_Fees(client)
        self.output_volume: SeriesPattern18[Sats] = SeriesPattern18(client, 'output_volume')
        self.unclaimed: BlockCumulativePattern = BlockCumulativePattern(client, 'unclaimed_rewards')

class SeriesTree_Mining_Hashrate_Rate_Sma:
    """Series tree node."""
    
    def __init__(self, client: BrkClient, base_path: str = ''):
        self._1w: SeriesPattern1[StoredF64] = SeriesPattern1(client, 'hash_rate_sma_1w')
        self._1m: SeriesPattern1[StoredF64] = SeriesPattern1(client, 'hash_rate_sma_1m')
        self._2m: SeriesPattern1[StoredF64] = SeriesPattern1(client, 'hash_rate_sma_2m')
        self._1y: SeriesPattern1[StoredF64] = SeriesPattern1(client, 'hash_rate_sma_1y')

class SeriesTree_Mining_Hashrate_Rate:
    """Series tree node."""
    
    def __init__(self, client: BrkClient, base_path: str = ''):
        self.base: SeriesPattern1[StoredF64] = SeriesPattern1(client, 'hash_rate')
        self.sma: SeriesTree_Mining_Hashrate_Rate_Sma = SeriesTree_Mining_Hashrate_Rate_Sma(client)
        self.ath: SeriesPattern1[StoredF64] = SeriesPattern1(client, 'hash_rate_ath')
        self.drawdown: BpsPercentRatioPattern5 = BpsPercentRatioPattern5(client, 'hash_rate_drawdown')

class SeriesTree_Mining_Hashrate:
    """Series tree node."""
    
    def __init__(self, client: BrkClient, base_path: str = ''):
        self.rate: SeriesTree_Mining_Hashrate_Rate = SeriesTree_Mining_Hashrate_Rate(client)
        self.price: PhsReboundThsPattern = PhsReboundThsPattern(client, 'hash_price')
        self.value: PhsReboundThsPattern = PhsReboundThsPattern(client, 'hash_value')

class SeriesTree_Mining:
    """Series tree node."""
    
    def __init__(self, client: BrkClient, base_path: str = ''):
        self.rewards: SeriesTree_Mining_Rewards = SeriesTree_Mining_Rewards(client)
        self.hashrate: SeriesTree_Mining_Hashrate = SeriesTree_Mining_Hashrate(client)

class SeriesTree_Cointime_Activity:
    """Series tree node."""
    
    def __init__(self, client: BrkClient, base_path: str = ''):
        self.coinblocks_created: AverageBlockCumulativeSumPattern[StoredF64] = AverageBlockCumulativeSumPattern(client, 'coinblocks_created')
        self.coinblocks_stored: AverageBlockCumulativeSumPattern[StoredF64] = AverageBlockCumulativeSumPattern(client, 'coinblocks_stored')
        self.liveliness: SeriesPattern1[StoredF64] = SeriesPattern1(client, 'liveliness')
        self.vaultedness: SeriesPattern1[StoredF64] = SeriesPattern1(client, 'vaultedness')
        self.ratio: SeriesPattern1[StoredF64] = SeriesPattern1(client, 'activity_to_vaultedness')
        self.coinblocks_destroyed: AverageBlockCumulativeSumPattern[StoredF64] = AverageBlockCumulativeSumPattern(client, 'coinblocks_destroyed')

class SeriesTree_Cointime_Supply:
    """Series tree node."""
    
    def __init__(self, client: BrkClient, base_path: str = ''):
        self.vaulted: BtcCentsSatsUsdPattern = BtcCentsSatsUsdPattern(client, 'vaulted_supply')
        self.active: BtcCentsSatsUsdPattern = BtcCentsSatsUsdPattern(client, 'active_supply')

class SeriesTree_Cointime_Value:
    """Series tree node."""
    
    def __init__(self, client: BrkClient, base_path: str = ''):
        self.destroyed: AverageBlockCumulativeSumPattern[StoredF64] = AverageBlockCumulativeSumPattern(client, 'cointime_value_destroyed')
        self.created: AverageBlockCumulativeSumPattern[StoredF64] = AverageBlockCumulativeSumPattern(client, 'cointime_value_created')
        self.stored: AverageBlockCumulativeSumPattern[StoredF64] = AverageBlockCumulativeSumPattern(client, 'cointime_value_stored')
        self.vocdd: AverageBlockCumulativeSumPattern[StoredF64] = AverageBlockCumulativeSumPattern(client, 'vocdd')

class SeriesTree_Cointime_Cap:
    """Series tree node."""
    
    def __init__(self, client: BrkClient, base_path: str = ''):
        self.thermo: CentsUsdPattern3 = CentsUsdPattern3(client, 'thermo_cap')
        self.investor: CentsUsdPattern3 = CentsUsdPattern3(client, 'investor_cap')
        self.vaulted: CentsUsdPattern3 = CentsUsdPattern3(client, 'vaulted_cap')
        self.active: CentsUsdPattern3 = CentsUsdPattern3(client, 'active_cap')
        self.cointime: CentsUsdPattern3 = CentsUsdPattern3(client, 'cointime_cap')
        self.aviv: BpsRatioPattern2 = BpsRatioPattern2(client, 'aviv_ratio')

class SeriesTree_Cointime_Prices:
    """Series tree node."""
    
    def __init__(self, client: BrkClient, base_path: str = ''):
        self.vaulted: BpsCentsPercentilesRatioSatsUsdPattern = BpsCentsPercentilesRatioSatsUsdPattern(client, 'vaulted_price')
        self.active: BpsCentsPercentilesRatioSatsUsdPattern = BpsCentsPercentilesRatioSatsUsdPattern(client, 'active_price')
        self.true_market_mean: BpsCentsPercentilesRatioSatsUsdPattern = BpsCentsPercentilesRatioSatsUsdPattern(client, 'true_market_mean')
        self.cointime: BpsCentsPercentilesRatioSatsUsdPattern = BpsCentsPercentilesRatioSatsUsdPattern(client, 'cointime_price')

class SeriesTree_Cointime_Adjusted:
    """Series tree node."""
    
    def __init__(self, client: BrkClient, base_path: str = ''):
        self.inflation_rate: BpsPercentRatioPattern = BpsPercentRatioPattern(client, 'cointime_adj_inflation_rate')
        self.tx_velocity_native: SeriesPattern1[StoredF64] = SeriesPattern1(client, 'cointime_adj_tx_velocity_btc')
        self.tx_velocity_fiat: SeriesPattern1[StoredF64] = SeriesPattern1(client, 'cointime_adj_tx_velocity_usd')

class SeriesTree_Cointime_ReserveRisk:
    """Series tree node."""
    
    def __init__(self, client: BrkClient, base_path: str = ''):
        self.value: SeriesPattern1[StoredF64] = SeriesPattern1(client, 'reserve_risk')
        self.vocdd_median_1y: SeriesPattern18[StoredF64] = SeriesPattern18(client, 'vocdd_median_1y')
        self.hodl_bank: SeriesPattern18[StoredF64] = SeriesPattern18(client, 'hodl_bank')

class SeriesTree_Cointime:
    """Series tree node."""
    
    def __init__(self, client: BrkClient, base_path: str = ''):
        self.activity: SeriesTree_Cointime_Activity = SeriesTree_Cointime_Activity(client)
        self.supply: SeriesTree_Cointime_Supply = SeriesTree_Cointime_Supply(client)
        self.value: SeriesTree_Cointime_Value = SeriesTree_Cointime_Value(client)
        self.cap: SeriesTree_Cointime_Cap = SeriesTree_Cointime_Cap(client)
        self.prices: SeriesTree_Cointime_Prices = SeriesTree_Cointime_Prices(client)
        self.adjusted: SeriesTree_Cointime_Adjusted = SeriesTree_Cointime_Adjusted(client)
        self.reserve_risk: SeriesTree_Cointime_ReserveRisk = SeriesTree_Cointime_ReserveRisk(client)

class SeriesTree_Constants:
    """Series tree node."""
    
    def __init__(self, client: BrkClient, base_path: str = ''):
        self._0: SeriesPattern1[StoredU16] = SeriesPattern1(client, 'constant_0')
        self._1: SeriesPattern1[StoredU16] = SeriesPattern1(client, 'constant_1')
        self._2: SeriesPattern1[StoredU16] = SeriesPattern1(client, 'constant_2')
        self._3: SeriesPattern1[StoredU16] = SeriesPattern1(client, 'constant_3')
        self._4: SeriesPattern1[StoredU16] = SeriesPattern1(client, 'constant_4')
        self._20: SeriesPattern1[StoredU16] = SeriesPattern1(client, 'constant_20')
        self._30: SeriesPattern1[StoredU16] = SeriesPattern1(client, 'constant_30')
        self._38_2: SeriesPattern1[StoredF32] = SeriesPattern1(client, 'constant_38_2')
        self._50: SeriesPattern1[StoredU16] = SeriesPattern1(client, 'constant_50')
        self._61_8: SeriesPattern1[StoredF32] = SeriesPattern1(client, 'constant_61_8')
        self._70: SeriesPattern1[StoredU16] = SeriesPattern1(client, 'constant_70')
        self._80: SeriesPattern1[StoredU16] = SeriesPattern1(client, 'constant_80')
        self._100: SeriesPattern1[StoredU16] = SeriesPattern1(client, 'constant_100')
        self._600: SeriesPattern1[StoredU16] = SeriesPattern1(client, 'constant_600')
        self.minus_1: SeriesPattern1[StoredI8] = SeriesPattern1(client, 'constant_minus_1')
        self.minus_2: SeriesPattern1[StoredI8] = SeriesPattern1(client, 'constant_minus_2')
        self.minus_3: SeriesPattern1[StoredI8] = SeriesPattern1(client, 'constant_minus_3')
        self.minus_4: SeriesPattern1[StoredI8] = SeriesPattern1(client, 'constant_minus_4')

class SeriesTree_Indexes_Addr_P2pk33:
    """Series tree node."""
    
    def __init__(self, client: BrkClient, base_path: str = ''):
        self.identity: SeriesPattern26[P2PK33AddrIndex] = SeriesPattern26(client, 'p2pk33_addr_index')
        self.addr: SeriesPattern26[Addr] = SeriesPattern26(client, 'p2pk33_addr')

class SeriesTree_Indexes_Addr_P2pk65:
    """Series tree node."""
    
    def __init__(self, client: BrkClient, base_path: str = ''):
        self.identity: SeriesPattern27[P2PK65AddrIndex] = SeriesPattern27(client, 'p2pk65_addr_index')
        self.addr: SeriesPattern27[Addr] = SeriesPattern27(client, 'p2pk65_addr')

class SeriesTree_Indexes_Addr_P2pkh:
    """Series tree node."""
    
    def __init__(self, client: BrkClient, base_path: str = ''):
        self.identity: SeriesPattern28[P2PKHAddrIndex] = SeriesPattern28(client, 'p2pkh_addr_index')
        self.addr: SeriesPattern28[Addr] = SeriesPattern28(client, 'p2pkh_addr')

class SeriesTree_Indexes_Addr_P2sh:
    """Series tree node."""
    
    def __init__(self, client: BrkClient, base_path: str = ''):
        self.identity: SeriesPattern29[P2SHAddrIndex] = SeriesPattern29(client, 'p2sh_addr_index')
        self.addr: SeriesPattern29[Addr] = SeriesPattern29(client, 'p2sh_addr')

class SeriesTree_Indexes_Addr_P2tr:
    """Series tree node."""
    
    def __init__(self, client: BrkClient, base_path: str = ''):
        self.identity: SeriesPattern30[P2TRAddrIndex] = SeriesPattern30(client, 'p2tr_addr_index')
        self.addr: SeriesPattern30[Addr] = SeriesPattern30(client, 'p2tr_addr')

class SeriesTree_Indexes_Addr_P2wpkh:
    """Series tree node."""
    
    def __init__(self, client: BrkClient, base_path: str = ''):
        self.identity: SeriesPattern31[P2WPKHAddrIndex] = SeriesPattern31(client, 'p2wpkh_addr_index')
        self.addr: SeriesPattern31[Addr] = SeriesPattern31(client, 'p2wpkh_addr')

class SeriesTree_Indexes_Addr_P2wsh:
    """Series tree node."""
    
    def __init__(self, client: BrkClient, base_path: str = ''):
        self.identity: SeriesPattern32[P2WSHAddrIndex] = SeriesPattern32(client, 'p2wsh_addr_index')
        self.addr: SeriesPattern32[Addr] = SeriesPattern32(client, 'p2wsh_addr')

class SeriesTree_Indexes_Addr_P2a:
    """Series tree node."""
    
    def __init__(self, client: BrkClient, base_path: str = ''):
        self.identity: SeriesPattern24[P2AAddrIndex] = SeriesPattern24(client, 'p2a_addr_index')
        self.addr: SeriesPattern24[Addr] = SeriesPattern24(client, 'p2a_addr')

class SeriesTree_Indexes_Addr_P2ms:
    """Series tree node."""
    
    def __init__(self, client: BrkClient, base_path: str = ''):
        self.identity: SeriesPattern25[P2MSOutputIndex] = SeriesPattern25(client, 'p2ms_output_index')

class SeriesTree_Indexes_Addr_Empty:
    """Series tree node."""
    
    def __init__(self, client: BrkClient, base_path: str = ''):
        self.identity: SeriesPattern22[EmptyOutputIndex] = SeriesPattern22(client, 'empty_output_index')

class SeriesTree_Indexes_Addr_Unknown:
    """Series tree node."""
    
    def __init__(self, client: BrkClient, base_path: str = ''):
        self.identity: SeriesPattern33[UnknownOutputIndex] = SeriesPattern33(client, 'unknown_output_index')

class SeriesTree_Indexes_Addr_OpReturn:
    """Series tree node."""
    
    def __init__(self, client: BrkClient, base_path: str = ''):
        self.identity: SeriesPattern23[OpReturnIndex] = SeriesPattern23(client, 'op_return_index')

class SeriesTree_Indexes_Addr:
    """Series tree node."""
    
    def __init__(self, client: BrkClient, base_path: str = ''):
        self.p2pk33: SeriesTree_Indexes_Addr_P2pk33 = SeriesTree_Indexes_Addr_P2pk33(client)
        self.p2pk65: SeriesTree_Indexes_Addr_P2pk65 = SeriesTree_Indexes_Addr_P2pk65(client)
        self.p2pkh: SeriesTree_Indexes_Addr_P2pkh = SeriesTree_Indexes_Addr_P2pkh(client)
        self.p2sh: SeriesTree_Indexes_Addr_P2sh = SeriesTree_Indexes_Addr_P2sh(client)
        self.p2tr: SeriesTree_Indexes_Addr_P2tr = SeriesTree_Indexes_Addr_P2tr(client)
        self.p2wpkh: SeriesTree_Indexes_Addr_P2wpkh = SeriesTree_Indexes_Addr_P2wpkh(client)
        self.p2wsh: SeriesTree_Indexes_Addr_P2wsh = SeriesTree_Indexes_Addr_P2wsh(client)
        self.p2a: SeriesTree_Indexes_Addr_P2a = SeriesTree_Indexes_Addr_P2a(client)
        self.p2ms: SeriesTree_Indexes_Addr_P2ms = SeriesTree_Indexes_Addr_P2ms(client)
        self.empty: SeriesTree_Indexes_Addr_Empty = SeriesTree_Indexes_Addr_Empty(client)
        self.unknown: SeriesTree_Indexes_Addr_Unknown = SeriesTree_Indexes_Addr_Unknown(client)
        self.op_return: SeriesTree_Indexes_Addr_OpReturn = SeriesTree_Indexes_Addr_OpReturn(client)

class SeriesTree_Indexes_Height:
    """Series tree node."""
    
    def __init__(self, client: BrkClient, base_path: str = ''):
        self.minute10: SeriesPattern18[Minute10] = SeriesPattern18(client, 'minute10')
        self.minute30: SeriesPattern18[Minute30] = SeriesPattern18(client, 'minute30')
        self.hour1: SeriesPattern18[Hour1] = SeriesPattern18(client, 'hour1')
        self.hour4: SeriesPattern18[Hour4] = SeriesPattern18(client, 'hour4')
        self.hour12: SeriesPattern18[Hour12] = SeriesPattern18(client, 'hour12')
        self.day1: SeriesPattern18[Day1] = SeriesPattern18(client, 'day1')
        self.day3: SeriesPattern18[Day3] = SeriesPattern18(client, 'day3')
        self.epoch: SeriesPattern18[Epoch] = SeriesPattern18(client, 'epoch')
        self.halving: SeriesPattern18[Halving] = SeriesPattern18(client, 'halving')
        self.week1: SeriesPattern18[Week1] = SeriesPattern18(client, 'week1')
        self.month1: SeriesPattern18[Month1] = SeriesPattern18(client, 'month1')
        self.month3: SeriesPattern18[Month3] = SeriesPattern18(client, 'month3')
        self.month6: SeriesPattern18[Month6] = SeriesPattern18(client, 'month6')
        self.year1: SeriesPattern18[Year1] = SeriesPattern18(client, 'year1')
        self.year10: SeriesPattern18[Year10] = SeriesPattern18(client, 'year10')
        self.tx_index_count: SeriesPattern18[StoredU64] = SeriesPattern18(client, 'tx_index_count')

class SeriesTree_Indexes_Epoch:
    """Series tree node."""
    
    def __init__(self, client: BrkClient, base_path: str = ''):
        self.first_height: SeriesPattern17[Height] = SeriesPattern17(client, 'first_height')

class SeriesTree_Indexes_Halving:
    """Series tree node."""
    
    def __init__(self, client: BrkClient, base_path: str = ''):
        self.first_height: SeriesPattern16[Height] = SeriesPattern16(client, 'first_height')

class SeriesTree_Indexes_Minute10:
    """Series tree node."""
    
    def __init__(self, client: BrkClient, base_path: str = ''):
        self.first_height: SeriesPattern3[Height] = SeriesPattern3(client, 'first_height')

class SeriesTree_Indexes_Minute30:
    """Series tree node."""
    
    def __init__(self, client: BrkClient, base_path: str = ''):
        self.first_height: SeriesPattern4[Height] = SeriesPattern4(client, 'first_height')

class SeriesTree_Indexes_Hour1:
    """Series tree node."""
    
    def __init__(self, client: BrkClient, base_path: str = ''):
        self.first_height: SeriesPattern5[Height] = SeriesPattern5(client, 'first_height')

class SeriesTree_Indexes_Hour4:
    """Series tree node."""
    
    def __init__(self, client: BrkClient, base_path: str = ''):
        self.first_height: SeriesPattern6[Height] = SeriesPattern6(client, 'first_height')

class SeriesTree_Indexes_Hour12:
    """Series tree node."""
    
    def __init__(self, client: BrkClient, base_path: str = ''):
        self.first_height: SeriesPattern7[Height] = SeriesPattern7(client, 'first_height')

class SeriesTree_Indexes_Day1:
    """Series tree node."""
    
    def __init__(self, client: BrkClient, base_path: str = ''):
        self.date: SeriesPattern8[Date] = SeriesPattern8(client, 'date')
        self.first_height: SeriesPattern8[Height] = SeriesPattern8(client, 'first_height')

class SeriesTree_Indexes_Day3:
    """Series tree node."""
    
    def __init__(self, client: BrkClient, base_path: str = ''):
        self.date: SeriesPattern9[Date] = SeriesPattern9(client, 'date')
        self.first_height: SeriesPattern9[Height] = SeriesPattern9(client, 'first_height')

class SeriesTree_Indexes_Week1:
    """Series tree node."""
    
    def __init__(self, client: BrkClient, base_path: str = ''):
        self.date: SeriesPattern10[Date] = SeriesPattern10(client, 'date')
        self.first_height: SeriesPattern10[Height] = SeriesPattern10(client, 'first_height')

class SeriesTree_Indexes_Month1:
    """Series tree node."""
    
    def __init__(self, client: BrkClient, base_path: str = ''):
        self.date: SeriesPattern11[Date] = SeriesPattern11(client, 'date')
        self.first_height: SeriesPattern11[Height] = SeriesPattern11(client, 'first_height')

class SeriesTree_Indexes_Month3:
    """Series tree node."""
    
    def __init__(self, client: BrkClient, base_path: str = ''):
        self.date: SeriesPattern12[Date] = SeriesPattern12(client, 'date')
        self.first_height: SeriesPattern12[Height] = SeriesPattern12(client, 'first_height')

class SeriesTree_Indexes_Month6:
    """Series tree node."""
    
    def __init__(self, client: BrkClient, base_path: str = ''):
        self.date: SeriesPattern13[Date] = SeriesPattern13(client, 'date')
        self.first_height: SeriesPattern13[Height] = SeriesPattern13(client, 'first_height')

class SeriesTree_Indexes_Year1:
    """Series tree node."""
    
    def __init__(self, client: BrkClient, base_path: str = ''):
        self.date: SeriesPattern14[Date] = SeriesPattern14(client, 'date')
        self.first_height: SeriesPattern14[Height] = SeriesPattern14(client, 'first_height')

class SeriesTree_Indexes_Year10:
    """Series tree node."""
    
    def __init__(self, client: BrkClient, base_path: str = ''):
        self.date: SeriesPattern15[Date] = SeriesPattern15(client, 'date')
        self.first_height: SeriesPattern15[Height] = SeriesPattern15(client, 'first_height')

class SeriesTree_Indexes_TxIndex:
    """Series tree node."""
    
    def __init__(self, client: BrkClient, base_path: str = ''):
        self.identity: SeriesPattern19[TxIndex] = SeriesPattern19(client, 'tx_index')
        self.input_count: SeriesPattern19[StoredU64] = SeriesPattern19(client, 'input_count')
        self.output_count: SeriesPattern19[StoredU64] = SeriesPattern19(client, 'output_count')

class SeriesTree_Indexes_TxinIndex:
    """Series tree node."""
    
    def __init__(self, client: BrkClient, base_path: str = ''):
        self.identity: SeriesPattern20[TxInIndex] = SeriesPattern20(client, 'txin_index')

class SeriesTree_Indexes_TxoutIndex:
    """Series tree node."""
    
    def __init__(self, client: BrkClient, base_path: str = ''):
        self.identity: SeriesPattern21[TxOutIndex] = SeriesPattern21(client, 'txout_index')

class SeriesTree_Indexes_Timestamp:
    """Series tree node."""
    
    def __init__(self, client: BrkClient, base_path: str = ''):
        self.monotonic: SeriesPattern18[Timestamp] = SeriesPattern18(client, 'timestamp_monotonic')
        self.resolutions: SeriesPattern2[Timestamp] = SeriesPattern2(client, 'timestamp')

class SeriesTree_Indexes:
    """Series tree node."""
    
    def __init__(self, client: BrkClient, base_path: str = ''):
        self.addr: SeriesTree_Indexes_Addr = SeriesTree_Indexes_Addr(client)
        self.height: SeriesTree_Indexes_Height = SeriesTree_Indexes_Height(client)
        self.epoch: SeriesTree_Indexes_Epoch = SeriesTree_Indexes_Epoch(client)
        self.halving: SeriesTree_Indexes_Halving = SeriesTree_Indexes_Halving(client)
        self.minute10: SeriesTree_Indexes_Minute10 = SeriesTree_Indexes_Minute10(client)
        self.minute30: SeriesTree_Indexes_Minute30 = SeriesTree_Indexes_Minute30(client)
        self.hour1: SeriesTree_Indexes_Hour1 = SeriesTree_Indexes_Hour1(client)
        self.hour4: SeriesTree_Indexes_Hour4 = SeriesTree_Indexes_Hour4(client)
        self.hour12: SeriesTree_Indexes_Hour12 = SeriesTree_Indexes_Hour12(client)
        self.day1: SeriesTree_Indexes_Day1 = SeriesTree_Indexes_Day1(client)
        self.day3: SeriesTree_Indexes_Day3 = SeriesTree_Indexes_Day3(client)
        self.week1: SeriesTree_Indexes_Week1 = SeriesTree_Indexes_Week1(client)
        self.month1: SeriesTree_Indexes_Month1 = SeriesTree_Indexes_Month1(client)
        self.month3: SeriesTree_Indexes_Month3 = SeriesTree_Indexes_Month3(client)
        self.month6: SeriesTree_Indexes_Month6 = SeriesTree_Indexes_Month6(client)
        self.year1: SeriesTree_Indexes_Year1 = SeriesTree_Indexes_Year1(client)
        self.year10: SeriesTree_Indexes_Year10 = SeriesTree_Indexes_Year10(client)
        self.tx_index: SeriesTree_Indexes_TxIndex = SeriesTree_Indexes_TxIndex(client)
        self.txin_index: SeriesTree_Indexes_TxinIndex = SeriesTree_Indexes_TxinIndex(client)
        self.txout_index: SeriesTree_Indexes_TxoutIndex = SeriesTree_Indexes_TxoutIndex(client)
        self.timestamp: SeriesTree_Indexes_Timestamp = SeriesTree_Indexes_Timestamp(client)

class SeriesTree_Indicators_Dormancy:
    """Series tree node."""
    
    def __init__(self, client: BrkClient, base_path: str = ''):
        self.supply_adj: SeriesPattern1[StoredF32] = SeriesPattern1(client, 'dormancy_supply_adj')
        self.flow: SeriesPattern1[StoredF32] = SeriesPattern1(client, 'dormancy_flow')

class SeriesTree_Indicators_RarityMeter:
    """Series tree node."""
    
    def __init__(self, client: BrkClient, base_path: str = ''):
        self.full: IndexPct0Pct1Pct2Pct5Pct95Pct98Pct99ScorePattern = IndexPct0Pct1Pct2Pct5Pct95Pct98Pct99ScorePattern(client, 'rarity_meter')
        self.local: IndexPct0Pct1Pct2Pct5Pct95Pct98Pct99ScorePattern = IndexPct0Pct1Pct2Pct5Pct95Pct98Pct99ScorePattern(client, 'local_rarity_meter')
        self.cycle: IndexPct0Pct1Pct2Pct5Pct95Pct98Pct99ScorePattern = IndexPct0Pct1Pct2Pct5Pct95Pct98Pct99ScorePattern(client, 'cycle_rarity_meter')

class SeriesTree_Indicators:
    """Series tree node."""
    
    def __init__(self, client: BrkClient, base_path: str = ''):
        self.puell_multiple: BpsRatioPattern2 = BpsRatioPattern2(client, 'puell_multiple')
        self.nvt: BpsRatioPattern2 = BpsRatioPattern2(client, 'nvt')
        self.gini: BpsPercentRatioPattern2 = BpsPercentRatioPattern2(client, 'gini')
        self.rhodl_ratio: BpsRatioPattern2 = BpsRatioPattern2(client, 'rhodl_ratio')
        self.thermo_cap_multiple: BpsRatioPattern2 = BpsRatioPattern2(client, 'thermo_cap_multiple')
        self.coindays_destroyed_supply_adj: SeriesPattern1[StoredF32] = SeriesPattern1(client, 'coindays_destroyed_supply_adj')
        self.coinyears_destroyed_supply_adj: SeriesPattern1[StoredF32] = SeriesPattern1(client, 'coinyears_destroyed_supply_adj')
        self.dormancy: SeriesTree_Indicators_Dormancy = SeriesTree_Indicators_Dormancy(client)
        self.stock_to_flow: SeriesPattern1[StoredF32] = SeriesPattern1(client, 'stock_to_flow')
        self.seller_exhaustion: SeriesPattern1[StoredF32] = SeriesPattern1(client, 'seller_exhaustion')
        self.rarity_meter: SeriesTree_Indicators_RarityMeter = SeriesTree_Indicators_RarityMeter(client)

class SeriesTree_Investing_Period_DcaCostBasis:
    """Series tree node."""
    
    def __init__(self, client: BrkClient, base_path: str = ''):
        self._1w: CentsSatsUsdPattern = CentsSatsUsdPattern(client, 'dca_cost_basis_1w')
        self._1m: CentsSatsUsdPattern = CentsSatsUsdPattern(client, 'dca_cost_basis_1m')
        self._3m: CentsSatsUsdPattern = CentsSatsUsdPattern(client, 'dca_cost_basis_3m')
        self._6m: CentsSatsUsdPattern = CentsSatsUsdPattern(client, 'dca_cost_basis_6m')
        self._1y: CentsSatsUsdPattern = CentsSatsUsdPattern(client, 'dca_cost_basis_1y')
        self._2y: CentsSatsUsdPattern = CentsSatsUsdPattern(client, 'dca_cost_basis_2y')
        self._3y: CentsSatsUsdPattern = CentsSatsUsdPattern(client, 'dca_cost_basis_3y')
        self._4y: CentsSatsUsdPattern = CentsSatsUsdPattern(client, 'dca_cost_basis_4y')
        self._5y: CentsSatsUsdPattern = CentsSatsUsdPattern(client, 'dca_cost_basis_5y')
        self._6y: CentsSatsUsdPattern = CentsSatsUsdPattern(client, 'dca_cost_basis_6y')
        self._8y: CentsSatsUsdPattern = CentsSatsUsdPattern(client, 'dca_cost_basis_8y')
        self._10y: CentsSatsUsdPattern = CentsSatsUsdPattern(client, 'dca_cost_basis_10y')

class SeriesTree_Investing_Period:
    """Series tree node."""
    
    def __init__(self, client: BrkClient, base_path: str = ''):
        self.dca_stack: _10y1m1w1y2y3m3y4y5y6m6y8yPattern3 = _10y1m1w1y2y3m3y4y5y6m6y8yPattern3(client, 'dca_stack')
        self.dca_cost_basis: SeriesTree_Investing_Period_DcaCostBasis = SeriesTree_Investing_Period_DcaCostBasis(client)
        self.dca_return: _10y1m1w1y2y3m3y4y5y6m6y8yPattern2 = _10y1m1w1y2y3m3y4y5y6m6y8yPattern2(client, 'dca_return')
        self.dca_cagr: _10y2y3y4y5y6y8yPattern = _10y2y3y4y5y6y8yPattern(client, 'dca_cagr')
        self.lump_sum_stack: _10y1m1w1y2y3m3y4y5y6m6y8yPattern3 = _10y1m1w1y2y3m3y4y5y6m6y8yPattern3(client, 'lump_sum_stack')
        self.lump_sum_return: _10y1m1w1y2y3m3y4y5y6m6y8yPattern2 = _10y1m1w1y2y3m3y4y5y6m6y8yPattern2(client, 'lump_sum_return')

class SeriesTree_Investing_Class_DcaStack:
    """Series tree node."""
    
    def __init__(self, client: BrkClient, base_path: str = ''):
        self.from_2015: BtcCentsSatsUsdPattern = BtcCentsSatsUsdPattern(client, 'dca_stack_from_2015')
        self.from_2016: BtcCentsSatsUsdPattern = BtcCentsSatsUsdPattern(client, 'dca_stack_from_2016')
        self.from_2017: BtcCentsSatsUsdPattern = BtcCentsSatsUsdPattern(client, 'dca_stack_from_2017')
        self.from_2018: BtcCentsSatsUsdPattern = BtcCentsSatsUsdPattern(client, 'dca_stack_from_2018')
        self.from_2019: BtcCentsSatsUsdPattern = BtcCentsSatsUsdPattern(client, 'dca_stack_from_2019')
        self.from_2020: BtcCentsSatsUsdPattern = BtcCentsSatsUsdPattern(client, 'dca_stack_from_2020')
        self.from_2021: BtcCentsSatsUsdPattern = BtcCentsSatsUsdPattern(client, 'dca_stack_from_2021')
        self.from_2022: BtcCentsSatsUsdPattern = BtcCentsSatsUsdPattern(client, 'dca_stack_from_2022')
        self.from_2023: BtcCentsSatsUsdPattern = BtcCentsSatsUsdPattern(client, 'dca_stack_from_2023')
        self.from_2024: BtcCentsSatsUsdPattern = BtcCentsSatsUsdPattern(client, 'dca_stack_from_2024')
        self.from_2025: BtcCentsSatsUsdPattern = BtcCentsSatsUsdPattern(client, 'dca_stack_from_2025')
        self.from_2026: BtcCentsSatsUsdPattern = BtcCentsSatsUsdPattern(client, 'dca_stack_from_2026')

class SeriesTree_Investing_Class_DcaCostBasis:
    """Series tree node."""
    
    def __init__(self, client: BrkClient, base_path: str = ''):
        self.from_2015: CentsSatsUsdPattern = CentsSatsUsdPattern(client, 'dca_cost_basis_from_2015')
        self.from_2016: CentsSatsUsdPattern = CentsSatsUsdPattern(client, 'dca_cost_basis_from_2016')
        self.from_2017: CentsSatsUsdPattern = CentsSatsUsdPattern(client, 'dca_cost_basis_from_2017')
        self.from_2018: CentsSatsUsdPattern = CentsSatsUsdPattern(client, 'dca_cost_basis_from_2018')
        self.from_2019: CentsSatsUsdPattern = CentsSatsUsdPattern(client, 'dca_cost_basis_from_2019')
        self.from_2020: CentsSatsUsdPattern = CentsSatsUsdPattern(client, 'dca_cost_basis_from_2020')
        self.from_2021: CentsSatsUsdPattern = CentsSatsUsdPattern(client, 'dca_cost_basis_from_2021')
        self.from_2022: CentsSatsUsdPattern = CentsSatsUsdPattern(client, 'dca_cost_basis_from_2022')
        self.from_2023: CentsSatsUsdPattern = CentsSatsUsdPattern(client, 'dca_cost_basis_from_2023')
        self.from_2024: CentsSatsUsdPattern = CentsSatsUsdPattern(client, 'dca_cost_basis_from_2024')
        self.from_2025: CentsSatsUsdPattern = CentsSatsUsdPattern(client, 'dca_cost_basis_from_2025')
        self.from_2026: CentsSatsUsdPattern = CentsSatsUsdPattern(client, 'dca_cost_basis_from_2026')

class SeriesTree_Investing_Class_DcaReturn:
    """Series tree node."""
    
    def __init__(self, client: BrkClient, base_path: str = ''):
        self.from_2015: BpsPercentRatioPattern = BpsPercentRatioPattern(client, 'dca_return_from_2015')
        self.from_2016: BpsPercentRatioPattern = BpsPercentRatioPattern(client, 'dca_return_from_2016')
        self.from_2017: BpsPercentRatioPattern = BpsPercentRatioPattern(client, 'dca_return_from_2017')
        self.from_2018: BpsPercentRatioPattern = BpsPercentRatioPattern(client, 'dca_return_from_2018')
        self.from_2019: BpsPercentRatioPattern = BpsPercentRatioPattern(client, 'dca_return_from_2019')
        self.from_2020: BpsPercentRatioPattern = BpsPercentRatioPattern(client, 'dca_return_from_2020')
        self.from_2021: BpsPercentRatioPattern = BpsPercentRatioPattern(client, 'dca_return_from_2021')
        self.from_2022: BpsPercentRatioPattern = BpsPercentRatioPattern(client, 'dca_return_from_2022')
        self.from_2023: BpsPercentRatioPattern = BpsPercentRatioPattern(client, 'dca_return_from_2023')
        self.from_2024: BpsPercentRatioPattern = BpsPercentRatioPattern(client, 'dca_return_from_2024')
        self.from_2025: BpsPercentRatioPattern = BpsPercentRatioPattern(client, 'dca_return_from_2025')
        self.from_2026: BpsPercentRatioPattern = BpsPercentRatioPattern(client, 'dca_return_from_2026')

class SeriesTree_Investing_Class:
    """Series tree node."""
    
    def __init__(self, client: BrkClient, base_path: str = ''):
        self.dca_stack: SeriesTree_Investing_Class_DcaStack = SeriesTree_Investing_Class_DcaStack(client)
        self.dca_cost_basis: SeriesTree_Investing_Class_DcaCostBasis = SeriesTree_Investing_Class_DcaCostBasis(client)
        self.dca_return: SeriesTree_Investing_Class_DcaReturn = SeriesTree_Investing_Class_DcaReturn(client)

class SeriesTree_Investing:
    """Series tree node."""
    
    def __init__(self, client: BrkClient, base_path: str = ''):
        self.sats_per_day: SeriesPattern18[Sats] = SeriesPattern18(client, 'dca_sats_per_day')
        self.period: SeriesTree_Investing_Period = SeriesTree_Investing_Period(client)
        self.class_: SeriesTree_Investing_Class = SeriesTree_Investing_Class(client)

class SeriesTree_Market_Ath:
    """Series tree node."""
    
    def __init__(self, client: BrkClient, base_path: str = ''):
        self.high: CentsSatsUsdPattern = CentsSatsUsdPattern(client, 'price_ath')
        self.drawdown: BpsPercentRatioPattern5 = BpsPercentRatioPattern5(client, 'price_drawdown')
        self.days_since: SeriesPattern1[StoredF32] = SeriesPattern1(client, 'days_since_price_ath')
        self.years_since: SeriesPattern1[StoredF32] = SeriesPattern1(client, 'years_since_price_ath')
        self.max_days_between: SeriesPattern1[StoredF32] = SeriesPattern1(client, 'max_days_between_price_ath')
        self.max_years_between: SeriesPattern1[StoredF32] = SeriesPattern1(client, 'max_years_between_price_ath')

class SeriesTree_Market_Lookback:
    """Series tree node."""
    
    def __init__(self, client: BrkClient, base_path: str = ''):
        self._24h: CentsSatsUsdPattern = CentsSatsUsdPattern(client, 'price_past_24h')
        self._1w: CentsSatsUsdPattern = CentsSatsUsdPattern(client, 'price_past_1w')
        self._1m: CentsSatsUsdPattern = CentsSatsUsdPattern(client, 'price_past_1m')
        self._3m: CentsSatsUsdPattern = CentsSatsUsdPattern(client, 'price_past_3m')
        self._6m: CentsSatsUsdPattern = CentsSatsUsdPattern(client, 'price_past_6m')
        self._1y: CentsSatsUsdPattern = CentsSatsUsdPattern(client, 'price_past_1y')
        self._2y: CentsSatsUsdPattern = CentsSatsUsdPattern(client, 'price_past_2y')
        self._3y: CentsSatsUsdPattern = CentsSatsUsdPattern(client, 'price_past_3y')
        self._4y: CentsSatsUsdPattern = CentsSatsUsdPattern(client, 'price_past_4y')
        self._5y: CentsSatsUsdPattern = CentsSatsUsdPattern(client, 'price_past_5y')
        self._6y: CentsSatsUsdPattern = CentsSatsUsdPattern(client, 'price_past_6y')
        self._8y: CentsSatsUsdPattern = CentsSatsUsdPattern(client, 'price_past_8y')
        self._10y: CentsSatsUsdPattern = CentsSatsUsdPattern(client, 'price_past_10y')

class SeriesTree_Market_Returns_Periods:
    """Series tree node."""
    
    def __init__(self, client: BrkClient, base_path: str = ''):
        self._24h: BpsPercentRatioPattern = BpsPercentRatioPattern(client, 'price_return_24h')
        self._1w: BpsPercentRatioPattern = BpsPercentRatioPattern(client, 'price_return_1w')
        self._1m: BpsPercentRatioPattern = BpsPercentRatioPattern(client, 'price_return_1m')
        self._3m: BpsPercentRatioPattern = BpsPercentRatioPattern(client, 'price_return_3m')
        self._6m: BpsPercentRatioPattern = BpsPercentRatioPattern(client, 'price_return_6m')
        self._1y: BpsPercentRatioPattern = BpsPercentRatioPattern(client, 'price_return_1y')
        self._2y: BpsPercentRatioPattern = BpsPercentRatioPattern(client, 'price_return_2y')
        self._3y: BpsPercentRatioPattern = BpsPercentRatioPattern(client, 'price_return_3y')
        self._4y: BpsPercentRatioPattern = BpsPercentRatioPattern(client, 'price_return_4y')
        self._5y: BpsPercentRatioPattern = BpsPercentRatioPattern(client, 'price_return_5y')
        self._6y: BpsPercentRatioPattern = BpsPercentRatioPattern(client, 'price_return_6y')
        self._8y: BpsPercentRatioPattern = BpsPercentRatioPattern(client, 'price_return_8y')
        self._10y: BpsPercentRatioPattern = BpsPercentRatioPattern(client, 'price_return_10y')

class SeriesTree_Market_Returns_Sd24h_24h:
    """Series tree node."""
    
    def __init__(self, client: BrkClient, base_path: str = ''):
        self.sma: SeriesPattern1[StoredF32] = SeriesPattern1(client, 'price_return_24h_sma_24h')
        self.sd: SeriesPattern1[StoredF32] = SeriesPattern1(client, 'price_return_24h_sd_24h')

class SeriesTree_Market_Returns_Sd24h_1w:
    """Series tree node."""
    
    def __init__(self, client: BrkClient, base_path: str = ''):
        self.sma: SeriesPattern1[StoredF32] = SeriesPattern1(client, 'price_return_24h_sma_1w')
        self.sd: SeriesPattern1[StoredF32] = SeriesPattern1(client, 'price_return_24h_sd_1w')

class SeriesTree_Market_Returns_Sd24h_1m:
    """Series tree node."""
    
    def __init__(self, client: BrkClient, base_path: str = ''):
        self.sma: SeriesPattern1[StoredF32] = SeriesPattern1(client, 'price_return_24h_sma_1m')
        self.sd: SeriesPattern1[StoredF32] = SeriesPattern1(client, 'price_return_24h_sd_1m')

class SeriesTree_Market_Returns_Sd24h_1y:
    """Series tree node."""
    
    def __init__(self, client: BrkClient, base_path: str = ''):
        self.sma: SeriesPattern1[StoredF32] = SeriesPattern1(client, 'price_return_24h_sma_1y')
        self.sd: SeriesPattern1[StoredF32] = SeriesPattern1(client, 'price_return_24h_sd_1y')

class SeriesTree_Market_Returns_Sd24h:
    """Series tree node."""
    
    def __init__(self, client: BrkClient, base_path: str = ''):
        self._24h: SeriesTree_Market_Returns_Sd24h_24h = SeriesTree_Market_Returns_Sd24h_24h(client)
        self._1w: SeriesTree_Market_Returns_Sd24h_1w = SeriesTree_Market_Returns_Sd24h_1w(client)
        self._1m: SeriesTree_Market_Returns_Sd24h_1m = SeriesTree_Market_Returns_Sd24h_1m(client)
        self._1y: SeriesTree_Market_Returns_Sd24h_1y = SeriesTree_Market_Returns_Sd24h_1y(client)

class SeriesTree_Market_Returns:
    """Series tree node."""
    
    def __init__(self, client: BrkClient, base_path: str = ''):
        self.periods: SeriesTree_Market_Returns_Periods = SeriesTree_Market_Returns_Periods(client)
        self.cagr: _10y2y3y4y5y6y8yPattern = _10y2y3y4y5y6y8yPattern(client, 'price_cagr')
        self.sd_24h: SeriesTree_Market_Returns_Sd24h = SeriesTree_Market_Returns_Sd24h(client)

class SeriesTree_Market_Range:
    """Series tree node."""
    
    def __init__(self, client: BrkClient, base_path: str = ''):
        self.min: _1m1w1y2wPattern = _1m1w1y2wPattern(client, 'price_min')
        self.max: _1m1w1y2wPattern = _1m1w1y2wPattern(client, 'price_max')
        self.true_range: SeriesPattern1[StoredF32] = SeriesPattern1(client, 'price_true_range')
        self.true_range_sum_2w: SeriesPattern1[StoredF32] = SeriesPattern1(client, 'price_true_range_sum_2w')
        self.choppiness_index_2w: BpsPercentRatioPattern2 = BpsPercentRatioPattern2(client, 'price_choppiness_index_2w')

class SeriesTree_Market_MovingAverage_Sma_200d:
    """Series tree node."""
    
    def __init__(self, client: BrkClient, base_path: str = ''):
        self.usd: SeriesPattern1[Dollars] = SeriesPattern1(client, 'price_sma_200d')
        self.cents: SeriesPattern1[Cents] = SeriesPattern1(client, 'price_sma_200d_cents')
        self.sats: SeriesPattern1[SatsFract] = SeriesPattern1(client, 'price_sma_200d_sats')
        self.bps: SeriesPattern1[BasisPoints32] = SeriesPattern1(client, 'price_sma_200d_ratio_bps')
        self.ratio: SeriesPattern1[StoredF32] = SeriesPattern1(client, 'price_sma_200d_ratio')
        self.x2_4: CentsSatsUsdPattern = CentsSatsUsdPattern(client, 'price_sma_200d_x2_4')
        self.x0_8: CentsSatsUsdPattern = CentsSatsUsdPattern(client, 'price_sma_200d_x0_8')

class SeriesTree_Market_MovingAverage_Sma_350d:
    """Series tree node."""
    
    def __init__(self, client: BrkClient, base_path: str = ''):
        self.usd: SeriesPattern1[Dollars] = SeriesPattern1(client, 'price_sma_350d')
        self.cents: SeriesPattern1[Cents] = SeriesPattern1(client, 'price_sma_350d_cents')
        self.sats: SeriesPattern1[SatsFract] = SeriesPattern1(client, 'price_sma_350d_sats')
        self.bps: SeriesPattern1[BasisPoints32] = SeriesPattern1(client, 'price_sma_350d_ratio_bps')
        self.ratio: SeriesPattern1[StoredF32] = SeriesPattern1(client, 'price_sma_350d_ratio')
        self.x2: CentsSatsUsdPattern = CentsSatsUsdPattern(client, 'price_sma_350d_x2')

class SeriesTree_Market_MovingAverage_Sma:
    """Series tree node."""
    
    def __init__(self, client: BrkClient, base_path: str = ''):
        self._1w: BpsCentsRatioSatsUsdPattern = BpsCentsRatioSatsUsdPattern(client, 'price_sma_1w')
        self._8d: BpsCentsRatioSatsUsdPattern = BpsCentsRatioSatsUsdPattern(client, 'price_sma_8d')
        self._13d: BpsCentsRatioSatsUsdPattern = BpsCentsRatioSatsUsdPattern(client, 'price_sma_13d')
        self._21d: BpsCentsRatioSatsUsdPattern = BpsCentsRatioSatsUsdPattern(client, 'price_sma_21d')
        self._1m: BpsCentsRatioSatsUsdPattern = BpsCentsRatioSatsUsdPattern(client, 'price_sma_1m')
        self._34d: BpsCentsRatioSatsUsdPattern = BpsCentsRatioSatsUsdPattern(client, 'price_sma_34d')
        self._55d: BpsCentsRatioSatsUsdPattern = BpsCentsRatioSatsUsdPattern(client, 'price_sma_55d')
        self._89d: BpsCentsRatioSatsUsdPattern = BpsCentsRatioSatsUsdPattern(client, 'price_sma_89d')
        self._111d: BpsCentsRatioSatsUsdPattern = BpsCentsRatioSatsUsdPattern(client, 'price_sma_111d')
        self._144d: BpsCentsRatioSatsUsdPattern = BpsCentsRatioSatsUsdPattern(client, 'price_sma_144d')
        self._200d: SeriesTree_Market_MovingAverage_Sma_200d = SeriesTree_Market_MovingAverage_Sma_200d(client)
        self._350d: SeriesTree_Market_MovingAverage_Sma_350d = SeriesTree_Market_MovingAverage_Sma_350d(client)
        self._1y: BpsCentsRatioSatsUsdPattern = BpsCentsRatioSatsUsdPattern(client, 'price_sma_1y')
        self._2y: BpsCentsRatioSatsUsdPattern = BpsCentsRatioSatsUsdPattern(client, 'price_sma_2y')
        self._200w: BpsCentsRatioSatsUsdPattern = BpsCentsRatioSatsUsdPattern(client, 'price_sma_200w')
        self._4y: BpsCentsRatioSatsUsdPattern = BpsCentsRatioSatsUsdPattern(client, 'price_sma_4y')

class SeriesTree_Market_MovingAverage_Ema:
    """Series tree node."""
    
    def __init__(self, client: BrkClient, base_path: str = ''):
        self._1w: BpsCentsRatioSatsUsdPattern = BpsCentsRatioSatsUsdPattern(client, 'price_ema_1w')
        self._8d: BpsCentsRatioSatsUsdPattern = BpsCentsRatioSatsUsdPattern(client, 'price_ema_8d')
        self._12d: BpsCentsRatioSatsUsdPattern = BpsCentsRatioSatsUsdPattern(client, 'price_ema_12d')
        self._13d: BpsCentsRatioSatsUsdPattern = BpsCentsRatioSatsUsdPattern(client, 'price_ema_13d')
        self._21d: BpsCentsRatioSatsUsdPattern = BpsCentsRatioSatsUsdPattern(client, 'price_ema_21d')
        self._26d: BpsCentsRatioSatsUsdPattern = BpsCentsRatioSatsUsdPattern(client, 'price_ema_26d')
        self._1m: BpsCentsRatioSatsUsdPattern = BpsCentsRatioSatsUsdPattern(client, 'price_ema_1m')
        self._34d: BpsCentsRatioSatsUsdPattern = BpsCentsRatioSatsUsdPattern(client, 'price_ema_34d')
        self._55d: BpsCentsRatioSatsUsdPattern = BpsCentsRatioSatsUsdPattern(client, 'price_ema_55d')
        self._89d: BpsCentsRatioSatsUsdPattern = BpsCentsRatioSatsUsdPattern(client, 'price_ema_89d')
        self._144d: BpsCentsRatioSatsUsdPattern = BpsCentsRatioSatsUsdPattern(client, 'price_ema_144d')
        self._200d: BpsCentsRatioSatsUsdPattern = BpsCentsRatioSatsUsdPattern(client, 'price_ema_200d')
        self._1y: BpsCentsRatioSatsUsdPattern = BpsCentsRatioSatsUsdPattern(client, 'price_ema_1y')
        self._2y: BpsCentsRatioSatsUsdPattern = BpsCentsRatioSatsUsdPattern(client, 'price_ema_2y')
        self._200w: BpsCentsRatioSatsUsdPattern = BpsCentsRatioSatsUsdPattern(client, 'price_ema_200w')
        self._4y: BpsCentsRatioSatsUsdPattern = BpsCentsRatioSatsUsdPattern(client, 'price_ema_4y')

class SeriesTree_Market_MovingAverage:
    """Series tree node."""
    
    def __init__(self, client: BrkClient, base_path: str = ''):
        self.sma: SeriesTree_Market_MovingAverage_Sma = SeriesTree_Market_MovingAverage_Sma(client)
        self.ema: SeriesTree_Market_MovingAverage_Ema = SeriesTree_Market_MovingAverage_Ema(client)

class SeriesTree_Market_Technical_Rsi:
    """Series tree node."""
    
    def __init__(self, client: BrkClient, base_path: str = ''):
        self._24h: RsiStochPattern = RsiStochPattern(client, 'rsi', '24h')
        self._1w: RsiStochPattern = RsiStochPattern(client, 'rsi', '1w')
        self._1m: RsiStochPattern = RsiStochPattern(client, 'rsi', '1m')

class SeriesTree_Market_Technical_Macd_24h:
    """Series tree node."""
    
    def __init__(self, client: BrkClient, base_path: str = ''):
        self.ema_fast: SeriesPattern1[StoredF32] = SeriesPattern1(client, 'macd_ema_fast_24h')
        self.ema_slow: SeriesPattern1[StoredF32] = SeriesPattern1(client, 'macd_ema_slow_24h')
        self.line: SeriesPattern1[StoredF32] = SeriesPattern1(client, 'macd_line_24h')
        self.signal: SeriesPattern1[StoredF32] = SeriesPattern1(client, 'macd_signal_24h')
        self.histogram: SeriesPattern1[StoredF32] = SeriesPattern1(client, 'macd_histogram_24h')

class SeriesTree_Market_Technical_Macd_1w:
    """Series tree node."""
    
    def __init__(self, client: BrkClient, base_path: str = ''):
        self.ema_fast: SeriesPattern1[StoredF32] = SeriesPattern1(client, 'macd_ema_fast_1w')
        self.ema_slow: SeriesPattern1[StoredF32] = SeriesPattern1(client, 'macd_ema_slow_1w')
        self.line: SeriesPattern1[StoredF32] = SeriesPattern1(client, 'macd_line_1w')
        self.signal: SeriesPattern1[StoredF32] = SeriesPattern1(client, 'macd_signal_1w')
        self.histogram: SeriesPattern1[StoredF32] = SeriesPattern1(client, 'macd_histogram_1w')

class SeriesTree_Market_Technical_Macd_1m:
    """Series tree node."""
    
    def __init__(self, client: BrkClient, base_path: str = ''):
        self.ema_fast: SeriesPattern1[StoredF32] = SeriesPattern1(client, 'macd_ema_fast_1m')
        self.ema_slow: SeriesPattern1[StoredF32] = SeriesPattern1(client, 'macd_ema_slow_1m')
        self.line: SeriesPattern1[StoredF32] = SeriesPattern1(client, 'macd_line_1m')
        self.signal: SeriesPattern1[StoredF32] = SeriesPattern1(client, 'macd_signal_1m')
        self.histogram: SeriesPattern1[StoredF32] = SeriesPattern1(client, 'macd_histogram_1m')

class SeriesTree_Market_Technical_Macd:
    """Series tree node."""
    
    def __init__(self, client: BrkClient, base_path: str = ''):
        self._24h: SeriesTree_Market_Technical_Macd_24h = SeriesTree_Market_Technical_Macd_24h(client)
        self._1w: SeriesTree_Market_Technical_Macd_1w = SeriesTree_Market_Technical_Macd_1w(client)
        self._1m: SeriesTree_Market_Technical_Macd_1m = SeriesTree_Market_Technical_Macd_1m(client)

class SeriesTree_Market_Technical:
    """Series tree node."""
    
    def __init__(self, client: BrkClient, base_path: str = ''):
        self.rsi: SeriesTree_Market_Technical_Rsi = SeriesTree_Market_Technical_Rsi(client)
        self.pi_cycle: BpsRatioPattern2 = BpsRatioPattern2(client, 'pi_cycle')
        self.macd: SeriesTree_Market_Technical_Macd = SeriesTree_Market_Technical_Macd(client)

class SeriesTree_Market:
    """Series tree node."""
    
    def __init__(self, client: BrkClient, base_path: str = ''):
        self.ath: SeriesTree_Market_Ath = SeriesTree_Market_Ath(client)
        self.lookback: SeriesTree_Market_Lookback = SeriesTree_Market_Lookback(client)
        self.returns: SeriesTree_Market_Returns = SeriesTree_Market_Returns(client)
        self.volatility: _1m1w1y24hPattern[StoredF32] = _1m1w1y24hPattern(client, 'price_volatility')
        self.range: SeriesTree_Market_Range = SeriesTree_Market_Range(client)
        self.moving_average: SeriesTree_Market_MovingAverage = SeriesTree_Market_MovingAverage(client)
        self.technical: SeriesTree_Market_Technical = SeriesTree_Market_Technical(client)

class SeriesTree_Pools_Major:
    """Series tree node."""
    
    def __init__(self, client: BrkClient, base_path: str = ''):
        self.unknown: BlocksDominanceRewardsPattern = BlocksDominanceRewardsPattern(client, 'unknown')
        self.luxor: BlocksDominanceRewardsPattern = BlocksDominanceRewardsPattern(client, 'luxor')
        self.btccom: BlocksDominanceRewardsPattern = BlocksDominanceRewardsPattern(client, 'btccom')
        self.btctop: BlocksDominanceRewardsPattern = BlocksDominanceRewardsPattern(client, 'btctop')
        self.btcguild: BlocksDominanceRewardsPattern = BlocksDominanceRewardsPattern(client, 'btcguild')
        self.eligius: BlocksDominanceRewardsPattern = BlocksDominanceRewardsPattern(client, 'eligius')
        self.f2pool: BlocksDominanceRewardsPattern = BlocksDominanceRewardsPattern(client, 'f2pool')
        self.braiinspool: BlocksDominanceRewardsPattern = BlocksDominanceRewardsPattern(client, 'braiinspool')
        self.antpool: BlocksDominanceRewardsPattern = BlocksDominanceRewardsPattern(client, 'antpool')
        self.btcc: BlocksDominanceRewardsPattern = BlocksDominanceRewardsPattern(client, 'btcc')
        self.bwpool: BlocksDominanceRewardsPattern = BlocksDominanceRewardsPattern(client, 'bwpool')
        self.bitfury: BlocksDominanceRewardsPattern = BlocksDominanceRewardsPattern(client, 'bitfury')
        self.viabtc: BlocksDominanceRewardsPattern = BlocksDominanceRewardsPattern(client, 'viabtc')
        self.poolin: BlocksDominanceRewardsPattern = BlocksDominanceRewardsPattern(client, 'poolin')
        self.spiderpool: BlocksDominanceRewardsPattern = BlocksDominanceRewardsPattern(client, 'spiderpool')
        self.binancepool: BlocksDominanceRewardsPattern = BlocksDominanceRewardsPattern(client, 'binancepool')
        self.foundryusa: BlocksDominanceRewardsPattern = BlocksDominanceRewardsPattern(client, 'foundryusa')
        self.sbicrypto: BlocksDominanceRewardsPattern = BlocksDominanceRewardsPattern(client, 'sbicrypto')
        self.marapool: BlocksDominanceRewardsPattern = BlocksDominanceRewardsPattern(client, 'marapool')
        self.secpool: BlocksDominanceRewardsPattern = BlocksDominanceRewardsPattern(client, 'secpool')
        self.ocean: BlocksDominanceRewardsPattern = BlocksDominanceRewardsPattern(client, 'ocean')
        self.whitepool: BlocksDominanceRewardsPattern = BlocksDominanceRewardsPattern(client, 'whitepool')

class SeriesTree_Pools_Minor:
    """Series tree node."""
    
    def __init__(self, client: BrkClient, base_path: str = ''):
        self.blockfills: BlocksDominancePattern = BlocksDominancePattern(client, 'blockfills')
        self.ultimuspool: BlocksDominancePattern = BlocksDominancePattern(client, 'ultimuspool')
        self.terrapool: BlocksDominancePattern = BlocksDominancePattern(client, 'terrapool')
        self.onethash: BlocksDominancePattern = BlocksDominancePattern(client, 'onethash')
        self.bitfarms: BlocksDominancePattern = BlocksDominancePattern(client, 'bitfarms')
        self.huobipool: BlocksDominancePattern = BlocksDominancePattern(client, 'huobipool')
        self.wayicn: BlocksDominancePattern = BlocksDominancePattern(client, 'wayicn')
        self.canoepool: BlocksDominancePattern = BlocksDominancePattern(client, 'canoepool')
        self.bitcoincom: BlocksDominancePattern = BlocksDominancePattern(client, 'bitcoincom')
        self.pool175btc: BlocksDominancePattern = BlocksDominancePattern(client, 'pool175btc')
        self.gbminers: BlocksDominancePattern = BlocksDominancePattern(client, 'gbminers')
        self.axbt: BlocksDominancePattern = BlocksDominancePattern(client, 'axbt')
        self.asicminer: BlocksDominancePattern = BlocksDominancePattern(client, 'asicminer')
        self.bitminter: BlocksDominancePattern = BlocksDominancePattern(client, 'bitminter')
        self.bitcoinrussia: BlocksDominancePattern = BlocksDominancePattern(client, 'bitcoinrussia')
        self.btcserv: BlocksDominancePattern = BlocksDominancePattern(client, 'btcserv')
        self.simplecoinus: BlocksDominancePattern = BlocksDominancePattern(client, 'simplecoinus')
        self.ozcoin: BlocksDominancePattern = BlocksDominancePattern(client, 'ozcoin')
        self.eclipsemc: BlocksDominancePattern = BlocksDominancePattern(client, 'eclipsemc')
        self.maxbtc: BlocksDominancePattern = BlocksDominancePattern(client, 'maxbtc')
        self.triplemining: BlocksDominancePattern = BlocksDominancePattern(client, 'triplemining')
        self.coinlab: BlocksDominancePattern = BlocksDominancePattern(client, 'coinlab')
        self.pool50btc: BlocksDominancePattern = BlocksDominancePattern(client, 'pool50btc')
        self.ghashio: BlocksDominancePattern = BlocksDominancePattern(client, 'ghashio')
        self.stminingcorp: BlocksDominancePattern = BlocksDominancePattern(client, 'stminingcorp')
        self.bitparking: BlocksDominancePattern = BlocksDominancePattern(client, 'bitparking')
        self.mmpool: BlocksDominancePattern = BlocksDominancePattern(client, 'mmpool')
        self.polmine: BlocksDominancePattern = BlocksDominancePattern(client, 'polmine')
        self.kncminer: BlocksDominancePattern = BlocksDominancePattern(client, 'kncminer')
        self.bitalo: BlocksDominancePattern = BlocksDominancePattern(client, 'bitalo')
        self.hhtt: BlocksDominancePattern = BlocksDominancePattern(client, 'hhtt')
        self.megabigpower: BlocksDominancePattern = BlocksDominancePattern(client, 'megabigpower')
        self.mtred: BlocksDominancePattern = BlocksDominancePattern(client, 'mtred')
        self.nmcbit: BlocksDominancePattern = BlocksDominancePattern(client, 'nmcbit')
        self.yourbtcnet: BlocksDominancePattern = BlocksDominancePattern(client, 'yourbtcnet')
        self.givemecoins: BlocksDominancePattern = BlocksDominancePattern(client, 'givemecoins')
        self.multicoinco: BlocksDominancePattern = BlocksDominancePattern(client, 'multicoinco')
        self.bcpoolio: BlocksDominancePattern = BlocksDominancePattern(client, 'bcpoolio')
        self.cointerra: BlocksDominancePattern = BlocksDominancePattern(client, 'cointerra')
        self.kanopool: BlocksDominancePattern = BlocksDominancePattern(client, 'kanopool')
        self.solock: BlocksDominancePattern = BlocksDominancePattern(client, 'solock')
        self.ckpool: BlocksDominancePattern = BlocksDominancePattern(client, 'ckpool')
        self.nicehash: BlocksDominancePattern = BlocksDominancePattern(client, 'nicehash')
        self.bitclub: BlocksDominancePattern = BlocksDominancePattern(client, 'bitclub')
        self.bitcoinaffiliatenetwork: BlocksDominancePattern = BlocksDominancePattern(client, 'bitcoinaffiliatenetwork')
        self.exxbw: BlocksDominancePattern = BlocksDominancePattern(client, 'exxbw')
        self.bitsolo: BlocksDominancePattern = BlocksDominancePattern(client, 'bitsolo')
        self.twentyoneinc: BlocksDominancePattern = BlocksDominancePattern(client, 'twentyoneinc')
        self.digitalbtc: BlocksDominancePattern = BlocksDominancePattern(client, 'digitalbtc')
        self.eightbaochi: BlocksDominancePattern = BlocksDominancePattern(client, 'eightbaochi')
        self.mybtccoinpool: BlocksDominancePattern = BlocksDominancePattern(client, 'mybtccoinpool')
        self.tbdice: BlocksDominancePattern = BlocksDominancePattern(client, 'tbdice')
        self.hashpool: BlocksDominancePattern = BlocksDominancePattern(client, 'hashpool')
        self.nexious: BlocksDominancePattern = BlocksDominancePattern(client, 'nexious')
        self.bravomining: BlocksDominancePattern = BlocksDominancePattern(client, 'bravomining')
        self.hotpool: BlocksDominancePattern = BlocksDominancePattern(client, 'hotpool')
        self.okexpool: BlocksDominancePattern = BlocksDominancePattern(client, 'okexpool')
        self.bcmonster: BlocksDominancePattern = BlocksDominancePattern(client, 'bcmonster')
        self.onehash: BlocksDominancePattern = BlocksDominancePattern(client, 'onehash')
        self.bixin: BlocksDominancePattern = BlocksDominancePattern(client, 'bixin')
        self.tatmaspool: BlocksDominancePattern = BlocksDominancePattern(client, 'tatmaspool')
        self.connectbtc: BlocksDominancePattern = BlocksDominancePattern(client, 'connectbtc')
        self.batpool: BlocksDominancePattern = BlocksDominancePattern(client, 'batpool')
        self.waterhole: BlocksDominancePattern = BlocksDominancePattern(client, 'waterhole')
        self.dcexploration: BlocksDominancePattern = BlocksDominancePattern(client, 'dcexploration')
        self.dcex: BlocksDominancePattern = BlocksDominancePattern(client, 'dcex')
        self.btpool: BlocksDominancePattern = BlocksDominancePattern(client, 'btpool')
        self.fiftyeightcoin: BlocksDominancePattern = BlocksDominancePattern(client, 'fiftyeightcoin')
        self.bitcoinindia: BlocksDominancePattern = BlocksDominancePattern(client, 'bitcoinindia')
        self.shawnp0wers: BlocksDominancePattern = BlocksDominancePattern(client, 'shawnp0wers')
        self.phashio: BlocksDominancePattern = BlocksDominancePattern(client, 'phashio')
        self.rigpool: BlocksDominancePattern = BlocksDominancePattern(client, 'rigpool')
        self.haozhuzhu: BlocksDominancePattern = BlocksDominancePattern(client, 'haozhuzhu')
        self.sevenpool: BlocksDominancePattern = BlocksDominancePattern(client, 'sevenpool')
        self.miningkings: BlocksDominancePattern = BlocksDominancePattern(client, 'miningkings')
        self.hashbx: BlocksDominancePattern = BlocksDominancePattern(client, 'hashbx')
        self.dpool: BlocksDominancePattern = BlocksDominancePattern(client, 'dpool')
        self.rawpool: BlocksDominancePattern = BlocksDominancePattern(client, 'rawpool')
        self.haominer: BlocksDominancePattern = BlocksDominancePattern(client, 'haominer')
        self.helix: BlocksDominancePattern = BlocksDominancePattern(client, 'helix')
        self.bitcoinukraine: BlocksDominancePattern = BlocksDominancePattern(client, 'bitcoinukraine')
        self.secretsuperstar: BlocksDominancePattern = BlocksDominancePattern(client, 'secretsuperstar')
        self.tigerpoolnet: BlocksDominancePattern = BlocksDominancePattern(client, 'tigerpoolnet')
        self.sigmapoolcom: BlocksDominancePattern = BlocksDominancePattern(client, 'sigmapoolcom')
        self.okpooltop: BlocksDominancePattern = BlocksDominancePattern(client, 'okpooltop')
        self.hummerpool: BlocksDominancePattern = BlocksDominancePattern(client, 'hummerpool')
        self.tangpool: BlocksDominancePattern = BlocksDominancePattern(client, 'tangpool')
        self.bytepool: BlocksDominancePattern = BlocksDominancePattern(client, 'bytepool')
        self.novablock: BlocksDominancePattern = BlocksDominancePattern(client, 'novablock')
        self.miningcity: BlocksDominancePattern = BlocksDominancePattern(client, 'miningcity')
        self.minerium: BlocksDominancePattern = BlocksDominancePattern(client, 'minerium')
        self.lubiancom: BlocksDominancePattern = BlocksDominancePattern(client, 'lubiancom')
        self.okkong: BlocksDominancePattern = BlocksDominancePattern(client, 'okkong')
        self.aaopool: BlocksDominancePattern = BlocksDominancePattern(client, 'aaopool')
        self.emcdpool: BlocksDominancePattern = BlocksDominancePattern(client, 'emcdpool')
        self.arkpool: BlocksDominancePattern = BlocksDominancePattern(client, 'arkpool')
        self.purebtccom: BlocksDominancePattern = BlocksDominancePattern(client, 'purebtccom')
        self.kucoinpool: BlocksDominancePattern = BlocksDominancePattern(client, 'kucoinpool')
        self.entrustcharitypool: BlocksDominancePattern = BlocksDominancePattern(client, 'entrustcharitypool')
        self.okminer: BlocksDominancePattern = BlocksDominancePattern(client, 'okminer')
        self.titan: BlocksDominancePattern = BlocksDominancePattern(client, 'titan')
        self.pegapool: BlocksDominancePattern = BlocksDominancePattern(client, 'pegapool')
        self.btcnuggets: BlocksDominancePattern = BlocksDominancePattern(client, 'btcnuggets')
        self.cloudhashing: BlocksDominancePattern = BlocksDominancePattern(client, 'cloudhashing')
        self.digitalxmintsy: BlocksDominancePattern = BlocksDominancePattern(client, 'digitalxmintsy')
        self.telco214: BlocksDominancePattern = BlocksDominancePattern(client, 'telco214')
        self.btcpoolparty: BlocksDominancePattern = BlocksDominancePattern(client, 'btcpoolparty')
        self.multipool: BlocksDominancePattern = BlocksDominancePattern(client, 'multipool')
        self.transactioncoinmining: BlocksDominancePattern = BlocksDominancePattern(client, 'transactioncoinmining')
        self.btcdig: BlocksDominancePattern = BlocksDominancePattern(client, 'btcdig')
        self.trickysbtcpool: BlocksDominancePattern = BlocksDominancePattern(client, 'trickysbtcpool')
        self.btcmp: BlocksDominancePattern = BlocksDominancePattern(client, 'btcmp')
        self.eobot: BlocksDominancePattern = BlocksDominancePattern(client, 'eobot')
        self.unomp: BlocksDominancePattern = BlocksDominancePattern(client, 'unomp')
        self.patels: BlocksDominancePattern = BlocksDominancePattern(client, 'patels')
        self.gogreenlight: BlocksDominancePattern = BlocksDominancePattern(client, 'gogreenlight')
        self.bitcoinindiapool: BlocksDominancePattern = BlocksDominancePattern(client, 'bitcoinindiapool')
        self.ekanembtc: BlocksDominancePattern = BlocksDominancePattern(client, 'ekanembtc')
        self.canoe: BlocksDominancePattern = BlocksDominancePattern(client, 'canoe')
        self.tiger: BlocksDominancePattern = BlocksDominancePattern(client, 'tiger')
        self.onem1x: BlocksDominancePattern = BlocksDominancePattern(client, 'onem1x')
        self.zulupool: BlocksDominancePattern = BlocksDominancePattern(client, 'zulupool')
        self.wiz: BlocksDominancePattern = BlocksDominancePattern(client, 'wiz')
        self.wk057: BlocksDominancePattern = BlocksDominancePattern(client, 'wk057')
        self.futurebitapollosolo: BlocksDominancePattern = BlocksDominancePattern(client, 'futurebitapollosolo')
        self.carbonnegative: BlocksDominancePattern = BlocksDominancePattern(client, 'carbonnegative')
        self.portlandhodl: BlocksDominancePattern = BlocksDominancePattern(client, 'portlandhodl')
        self.phoenix: BlocksDominancePattern = BlocksDominancePattern(client, 'phoenix')
        self.neopool: BlocksDominancePattern = BlocksDominancePattern(client, 'neopool')
        self.maxipool: BlocksDominancePattern = BlocksDominancePattern(client, 'maxipool')
        self.bitfufupool: BlocksDominancePattern = BlocksDominancePattern(client, 'bitfufupool')
        self.gdpool: BlocksDominancePattern = BlocksDominancePattern(client, 'gdpool')
        self.miningdutch: BlocksDominancePattern = BlocksDominancePattern(client, 'miningdutch')
        self.publicpool: BlocksDominancePattern = BlocksDominancePattern(client, 'publicpool')
        self.miningsquared: BlocksDominancePattern = BlocksDominancePattern(client, 'miningsquared')
        self.innopolistech: BlocksDominancePattern = BlocksDominancePattern(client, 'innopolistech')
        self.btclab: BlocksDominancePattern = BlocksDominancePattern(client, 'btclab')
        self.parasite: BlocksDominancePattern = BlocksDominancePattern(client, 'parasite')
        self.redrockpool: BlocksDominancePattern = BlocksDominancePattern(client, 'redrockpool')
        self.est3lar: BlocksDominancePattern = BlocksDominancePattern(client, 'est3lar')
        self.braiinssolo: BlocksDominancePattern = BlocksDominancePattern(client, 'braiinssolo')
        self.solopool: BlocksDominancePattern = BlocksDominancePattern(client, 'solopool')
        self.noderunners: BlocksDominancePattern = BlocksDominancePattern(client, 'noderunners')

class SeriesTree_Pools:
    """Series tree node."""
    
    def __init__(self, client: BrkClient, base_path: str = ''):
        self.pool: SeriesPattern18[PoolSlug] = SeriesPattern18(client, 'pool')
        self.major: SeriesTree_Pools_Major = SeriesTree_Pools_Major(client)
        self.minor: SeriesTree_Pools_Minor = SeriesTree_Pools_Minor(client)

class SeriesTree_Price_Split:
    """Series tree node."""
    
    def __init__(self, client: BrkClient, base_path: str = ''):
        self.open: CentsSatsUsdPattern3 = CentsSatsUsdPattern3(client, 'price_open')
        self.high: CentsSatsUsdPattern3 = CentsSatsUsdPattern3(client, 'price_high')
        self.low: CentsSatsUsdPattern3 = CentsSatsUsdPattern3(client, 'price_low')
        self.close: CentsSatsUsdPattern3 = CentsSatsUsdPattern3(client, 'price_close')

class SeriesTree_Price_Ohlc:
    """Series tree node."""
    
    def __init__(self, client: BrkClient, base_path: str = ''):
        self.usd: SeriesPattern2[OHLCDollars] = SeriesPattern2(client, 'price_ohlc')
        self.cents: SeriesPattern2[OHLCCents] = SeriesPattern2(client, 'price_ohlc_cents')
        self.sats: SeriesPattern2[OHLCSats] = SeriesPattern2(client, 'price_ohlc_sats')

class SeriesTree_Price_Spot:
    """Series tree node."""
    
    def __init__(self, client: BrkClient, base_path: str = ''):
        self.usd: SeriesPattern1[Dollars] = SeriesPattern1(client, 'price')
        self.cents: SeriesPattern1[Cents] = SeriesPattern1(client, 'price_cents')
        self.sats: SeriesPattern1[Sats] = SeriesPattern1(client, 'price_sats')

class SeriesTree_Price:
    """Series tree node."""
    
    def __init__(self, client: BrkClient, base_path: str = ''):
        self.split: SeriesTree_Price_Split = SeriesTree_Price_Split(client)
        self.ohlc: SeriesTree_Price_Ohlc = SeriesTree_Price_Ohlc(client)
        self.spot: SeriesTree_Price_Spot = SeriesTree_Price_Spot(client)

class SeriesTree_Supply_Velocity:
    """Series tree node."""
    
    def __init__(self, client: BrkClient, base_path: str = ''):
        self.native: SeriesPattern1[StoredF64] = SeriesPattern1(client, 'velocity_btc')
        self.fiat: SeriesPattern1[StoredF64] = SeriesPattern1(client, 'velocity_usd')

class SeriesTree_Supply:
    """Series tree node."""
    
    def __init__(self, client: BrkClient, base_path: str = ''):
        self.state: SeriesPattern18[SupplyState] = SeriesPattern18(client, 'supply_state')
        self.circulating: BtcCentsSatsUsdPattern = BtcCentsSatsUsdPattern(client, 'circulating_supply')
        self.burned: BlockCumulativePattern = BlockCumulativePattern(client, 'unspendable_supply')
        self.inflation_rate: BpsPercentRatioPattern = BpsPercentRatioPattern(client, 'inflation_rate')
        self.velocity: SeriesTree_Supply_Velocity = SeriesTree_Supply_Velocity(client)
        self.market_cap: CentsDeltaUsdPattern = CentsDeltaUsdPattern(client, 'market_cap')
        self.market_minus_realized_cap_growth_rate: _1m1w1y24hPattern[BasisPointsSigned32] = _1m1w1y24hPattern(client, 'market_minus_realized_cap_growth_rate')
        self.hodled_or_lost: BtcCentsSatsUsdPattern = BtcCentsSatsUsdPattern(client, 'hodled_or_lost_supply')

class SeriesTree_Cohorts_Utxo_All_Outputs:
    """Series tree node."""
    
    def __init__(self, client: BrkClient, base_path: str = ''):
        self.unspent_count: BaseDeltaPattern = BaseDeltaPattern(client, 'utxo_count')
        self.spent_count: AverageBlockCumulativeSumPattern2 = AverageBlockCumulativeSumPattern2(client, 'spent_utxo_count')
        self.spending_rate: SeriesPattern1[StoredF32] = SeriesPattern1(client, 'spending_rate')

class SeriesTree_Cohorts_Utxo_All_Activity:
    """Series tree node."""
    
    def __init__(self, client: BrkClient, base_path: str = ''):
        self.transfer_volume: AverageBlockCumulativeInSumPattern = AverageBlockCumulativeInSumPattern(client, 'transfer_volume')
        self.coindays_destroyed: AverageBlockCumulativeSumPattern[StoredF64] = AverageBlockCumulativeSumPattern(client, 'coindays_destroyed')
        self.coinyears_destroyed: SeriesPattern1[StoredF64] = SeriesPattern1(client, 'coinyears_destroyed')
        self.dormancy: _1m1w1y24hPattern[StoredF32] = _1m1w1y24hPattern(client, 'dormancy')

class SeriesTree_Cohorts_Utxo_All_Realized_Price_StdDev_All:
    """Series tree node."""
    
    def __init__(self, client: BrkClient, base_path: str = ''):
        self.sd: SeriesPattern1[StoredF32] = SeriesPattern1(client, 'realized_price_ratio_sd')
        self.zscore: SeriesPattern1[StoredF32] = SeriesPattern1(client, 'realized_price_ratio_zscore')
        self._0sd: CentsSatsUsdPattern = CentsSatsUsdPattern(client, 'realized_price_0sd')
        self.p0_5sd: PriceRatioPattern = PriceRatioPattern(client, 'realized_price', 'p0_5sd')
        self.p1sd: PriceRatioPattern = PriceRatioPattern(client, 'realized_price', 'p1sd')
        self.p1_5sd: PriceRatioPattern = PriceRatioPattern(client, 'realized_price', 'p1_5sd')
        self.p2sd: PriceRatioPattern = PriceRatioPattern(client, 'realized_price', 'p2sd')
        self.p2_5sd: PriceRatioPattern = PriceRatioPattern(client, 'realized_price', 'p2_5sd')
        self.p3sd: PriceRatioPattern = PriceRatioPattern(client, 'realized_price', 'p3sd')
        self.m0_5sd: PriceRatioPattern = PriceRatioPattern(client, 'realized_price', 'm0_5sd')
        self.m1sd: PriceRatioPattern = PriceRatioPattern(client, 'realized_price', 'm1sd')
        self.m1_5sd: PriceRatioPattern = PriceRatioPattern(client, 'realized_price', 'm1_5sd')
        self.m2sd: PriceRatioPattern = PriceRatioPattern(client, 'realized_price', 'm2sd')
        self.m2_5sd: PriceRatioPattern = PriceRatioPattern(client, 'realized_price', 'm2_5sd')
        self.m3sd: PriceRatioPattern = PriceRatioPattern(client, 'realized_price', 'm3sd')

class SeriesTree_Cohorts_Utxo_All_Realized_Price_StdDev_4y:
    """Series tree node."""
    
    def __init__(self, client: BrkClient, base_path: str = ''):
        self.sd: SeriesPattern1[StoredF32] = SeriesPattern1(client, 'realized_price_ratio_sd_4y')
        self.zscore: SeriesPattern1[StoredF32] = SeriesPattern1(client, 'realized_price_ratio_zscore_4y')
        self._0sd: CentsSatsUsdPattern = CentsSatsUsdPattern(client, 'realized_price_0sd_4y')
        self.p0_5sd: PriceRatioPattern = PriceRatioPattern(client, 'realized_price', 'p0_5sd_4y')
        self.p1sd: PriceRatioPattern = PriceRatioPattern(client, 'realized_price', 'p1sd_4y')
        self.p1_5sd: PriceRatioPattern = PriceRatioPattern(client, 'realized_price', 'p1_5sd_4y')
        self.p2sd: PriceRatioPattern = PriceRatioPattern(client, 'realized_price', 'p2sd_4y')
        self.p2_5sd: PriceRatioPattern = PriceRatioPattern(client, 'realized_price', 'p2_5sd_4y')
        self.p3sd: PriceRatioPattern = PriceRatioPattern(client, 'realized_price', 'p3sd_4y')
        self.m0_5sd: PriceRatioPattern = PriceRatioPattern(client, 'realized_price', 'm0_5sd_4y')
        self.m1sd: PriceRatioPattern = PriceRatioPattern(client, 'realized_price', 'm1sd_4y')
        self.m1_5sd: PriceRatioPattern = PriceRatioPattern(client, 'realized_price', 'm1_5sd_4y')
        self.m2sd: PriceRatioPattern = PriceRatioPattern(client, 'realized_price', 'm2sd_4y')
        self.m2_5sd: PriceRatioPattern = PriceRatioPattern(client, 'realized_price', 'm2_5sd_4y')
        self.m3sd: PriceRatioPattern = PriceRatioPattern(client, 'realized_price', 'm3sd_4y')

class SeriesTree_Cohorts_Utxo_All_Realized_Price_StdDev_2y:
    """Series tree node."""
    
    def __init__(self, client: BrkClient, base_path: str = ''):
        self.sd: SeriesPattern1[StoredF32] = SeriesPattern1(client, 'realized_price_ratio_sd_2y')
        self.zscore: SeriesPattern1[StoredF32] = SeriesPattern1(client, 'realized_price_ratio_zscore_2y')
        self._0sd: CentsSatsUsdPattern = CentsSatsUsdPattern(client, 'realized_price_0sd_2y')
        self.p0_5sd: PriceRatioPattern = PriceRatioPattern(client, 'realized_price', 'p0_5sd_2y')
        self.p1sd: PriceRatioPattern = PriceRatioPattern(client, 'realized_price', 'p1sd_2y')
        self.p1_5sd: PriceRatioPattern = PriceRatioPattern(client, 'realized_price', 'p1_5sd_2y')
        self.p2sd: PriceRatioPattern = PriceRatioPattern(client, 'realized_price', 'p2sd_2y')
        self.p2_5sd: PriceRatioPattern = PriceRatioPattern(client, 'realized_price', 'p2_5sd_2y')
        self.p3sd: PriceRatioPattern = PriceRatioPattern(client, 'realized_price', 'p3sd_2y')
        self.m0_5sd: PriceRatioPattern = PriceRatioPattern(client, 'realized_price', 'm0_5sd_2y')
        self.m1sd: PriceRatioPattern = PriceRatioPattern(client, 'realized_price', 'm1sd_2y')
        self.m1_5sd: PriceRatioPattern = PriceRatioPattern(client, 'realized_price', 'm1_5sd_2y')
        self.m2sd: PriceRatioPattern = PriceRatioPattern(client, 'realized_price', 'm2sd_2y')
        self.m2_5sd: PriceRatioPattern = PriceRatioPattern(client, 'realized_price', 'm2_5sd_2y')
        self.m3sd: PriceRatioPattern = PriceRatioPattern(client, 'realized_price', 'm3sd_2y')

class SeriesTree_Cohorts_Utxo_All_Realized_Price_StdDev_1y:
    """Series tree node."""
    
    def __init__(self, client: BrkClient, base_path: str = ''):
        self.sd: SeriesPattern1[StoredF32] = SeriesPattern1(client, 'realized_price_ratio_sd_1y')
        self.zscore: SeriesPattern1[StoredF32] = SeriesPattern1(client, 'realized_price_ratio_zscore_1y')
        self._0sd: CentsSatsUsdPattern = CentsSatsUsdPattern(client, 'realized_price_0sd_1y')
        self.p0_5sd: PriceRatioPattern = PriceRatioPattern(client, 'realized_price', 'p0_5sd_1y')
        self.p1sd: PriceRatioPattern = PriceRatioPattern(client, 'realized_price', 'p1sd_1y')
        self.p1_5sd: PriceRatioPattern = PriceRatioPattern(client, 'realized_price', 'p1_5sd_1y')
        self.p2sd: PriceRatioPattern = PriceRatioPattern(client, 'realized_price', 'p2sd_1y')
        self.p2_5sd: PriceRatioPattern = PriceRatioPattern(client, 'realized_price', 'p2_5sd_1y')
        self.p3sd: PriceRatioPattern = PriceRatioPattern(client, 'realized_price', 'p3sd_1y')
        self.m0_5sd: PriceRatioPattern = PriceRatioPattern(client, 'realized_price', 'm0_5sd_1y')
        self.m1sd: PriceRatioPattern = PriceRatioPattern(client, 'realized_price', 'm1sd_1y')
        self.m1_5sd: PriceRatioPattern = PriceRatioPattern(client, 'realized_price', 'm1_5sd_1y')
        self.m2sd: PriceRatioPattern = PriceRatioPattern(client, 'realized_price', 'm2sd_1y')
        self.m2_5sd: PriceRatioPattern = PriceRatioPattern(client, 'realized_price', 'm2_5sd_1y')
        self.m3sd: PriceRatioPattern = PriceRatioPattern(client, 'realized_price', 'm3sd_1y')

class SeriesTree_Cohorts_Utxo_All_Realized_Price_StdDev:
    """Series tree node."""
    
    def __init__(self, client: BrkClient, base_path: str = ''):
        self.all: SeriesTree_Cohorts_Utxo_All_Realized_Price_StdDev_All = SeriesTree_Cohorts_Utxo_All_Realized_Price_StdDev_All(client)
        self._4y: SeriesTree_Cohorts_Utxo_All_Realized_Price_StdDev_4y = SeriesTree_Cohorts_Utxo_All_Realized_Price_StdDev_4y(client)
        self._2y: SeriesTree_Cohorts_Utxo_All_Realized_Price_StdDev_2y = SeriesTree_Cohorts_Utxo_All_Realized_Price_StdDev_2y(client)
        self._1y: SeriesTree_Cohorts_Utxo_All_Realized_Price_StdDev_1y = SeriesTree_Cohorts_Utxo_All_Realized_Price_StdDev_1y(client)

class SeriesTree_Cohorts_Utxo_All_Realized_Price:
    """Series tree node."""
    
    def __init__(self, client: BrkClient, base_path: str = ''):
        self.usd: SeriesPattern1[Dollars] = SeriesPattern1(client, 'realized_price')
        self.cents: SeriesPattern1[Cents] = SeriesPattern1(client, 'realized_price_cents')
        self.sats: SeriesPattern1[SatsFract] = SeriesPattern1(client, 'realized_price_sats')
        self.bps: SeriesPattern1[BasisPoints32] = SeriesPattern1(client, 'realized_price_ratio_bps')
        self.ratio: SeriesPattern1[StoredF32] = SeriesPattern1(client, 'realized_price_ratio')
        self.percentiles: Pct0Pct1Pct2Pct5Pct95Pct98Pct99Pattern = Pct0Pct1Pct2Pct5Pct95Pct98Pct99Pattern(client, 'realized_price')
        self.sma: _1m1w1y2y4yAllPattern = _1m1w1y2y4yAllPattern(client, 'realized_price_ratio_sma')
        self.std_dev: SeriesTree_Cohorts_Utxo_All_Realized_Price_StdDev = SeriesTree_Cohorts_Utxo_All_Realized_Price_StdDev(client)

class SeriesTree_Cohorts_Utxo_All_Realized_Sopr_Adjusted:
    """Series tree node."""
    
    def __init__(self, client: BrkClient, base_path: str = ''):
        self.ratio: _1m1w1y24hPattern[StoredF64] = _1m1w1y24hPattern(client, 'asopr')
        self.transfer_volume: AverageBlockCumulativeSumPattern[Cents] = AverageBlockCumulativeSumPattern(client, 'adj_value_created')
        self.value_destroyed: AverageBlockCumulativeSumPattern[Cents] = AverageBlockCumulativeSumPattern(client, 'adj_value_destroyed')

class SeriesTree_Cohorts_Utxo_All_Realized_Sopr:
    """Series tree node."""
    
    def __init__(self, client: BrkClient, base_path: str = ''):
        self.value_destroyed: AverageBlockCumulativeSumPattern[Cents] = AverageBlockCumulativeSumPattern(client, 'value_destroyed')
        self.ratio: _1m1w1y24hPattern[StoredF64] = _1m1w1y24hPattern(client, 'sopr')
        self.adjusted: SeriesTree_Cohorts_Utxo_All_Realized_Sopr_Adjusted = SeriesTree_Cohorts_Utxo_All_Realized_Sopr_Adjusted(client)

class SeriesTree_Cohorts_Utxo_All_Realized:
    """Series tree node."""
    
    def __init__(self, client: BrkClient, base_path: str = ''):
        self.cap: CentsDeltaToUsdPattern = CentsDeltaToUsdPattern(client, 'realized_cap')
        self.profit: BlockCumulativeSumPattern = BlockCumulativeSumPattern(client, 'realized_profit')
        self.loss: BlockCumulativeNegativeSumPattern = BlockCumulativeNegativeSumPattern(client, 'realized_loss')
        self.price: SeriesTree_Cohorts_Utxo_All_Realized_Price = SeriesTree_Cohorts_Utxo_All_Realized_Price(client)
        self.mvrv: SeriesPattern1[StoredF32] = SeriesPattern1(client, 'mvrv')
        self.net_pnl: BlockChangeCumulativeDeltaSumPattern = BlockChangeCumulativeDeltaSumPattern(client, 'net')
        self.sopr: SeriesTree_Cohorts_Utxo_All_Realized_Sopr = SeriesTree_Cohorts_Utxo_All_Realized_Sopr(client)
        self.gross_pnl: BlockCumulativeSumPattern = BlockCumulativeSumPattern(client, 'realized_gross_pnl')
        self.sell_side_risk_ratio: _1m1w1y24hPattern8 = _1m1w1y24hPattern8(client, 'sell_side_risk_ratio')
        self.peak_regret: BlockCumulativeSumPattern = BlockCumulativeSumPattern(client, 'realized_peak_regret')
        self.capitalized: PricePattern = PricePattern(client, 'capitalized_price')
        self.profit_to_loss_ratio: _1m1w1y24hPattern[StoredF64] = _1m1w1y24hPattern(client, 'realized_profit_to_loss_ratio')

class SeriesTree_Cohorts_Utxo_All_CostBasis:
    """Series tree node."""
    
    def __init__(self, client: BrkClient, base_path: str = ''):
        self.in_profit: PerPattern = PerPattern(client, 'cost_basis_in_profit_per')
        self.in_loss: PerPattern = PerPattern(client, 'cost_basis_in_loss_per')
        self.min: CentsSatsUsdPattern = CentsSatsUsdPattern(client, 'cost_basis_min')
        self.max: CentsSatsUsdPattern = CentsSatsUsdPattern(client, 'cost_basis_max')
        self.per_coin: Pct05Pct10Pct15Pct20Pct25Pct30Pct35Pct40Pct45Pct50Pct55Pct60Pct65Pct70Pct75Pct80Pct85Pct90Pct95Pattern = Pct05Pct10Pct15Pct20Pct25Pct30Pct35Pct40Pct45Pct50Pct55Pct60Pct65Pct70Pct75Pct80Pct85Pct90Pct95Pattern(client, 'cost_basis_per_coin')
        self.per_dollar: Pct05Pct10Pct15Pct20Pct25Pct30Pct35Pct40Pct45Pct50Pct55Pct60Pct65Pct70Pct75Pct80Pct85Pct90Pct95Pattern = Pct05Pct10Pct15Pct20Pct25Pct30Pct35Pct40Pct45Pct50Pct55Pct60Pct65Pct70Pct75Pct80Pct85Pct90Pct95Pattern(client, 'cost_basis_per_dollar')
        self.supply_density: BpsPercentRatioPattern2 = BpsPercentRatioPattern2(client, 'supply_density')

class SeriesTree_Cohorts_Utxo_All_Unrealized_Profit:
    """Series tree node."""
    
    def __init__(self, client: BrkClient, base_path: str = ''):
        self.usd: SeriesPattern1[Dollars] = SeriesPattern1(client, 'unrealized_profit')
        self.cents: SeriesPattern1[Cents] = SeriesPattern1(client, 'unrealized_profit_cents')
        self.to_mcap: BpsPercentRatioPattern2 = BpsPercentRatioPattern2(client, 'unrealized_profit_to_mcap')
        self.to_own_gross_pnl: BpsPercentRatioPattern2 = BpsPercentRatioPattern2(client, 'unrealized_profit_to_own_gross_pnl')

class SeriesTree_Cohorts_Utxo_All_Unrealized_Loss:
    """Series tree node."""
    
    def __init__(self, client: BrkClient, base_path: str = ''):
        self.usd: SeriesPattern1[Dollars] = SeriesPattern1(client, 'unrealized_loss')
        self.cents: SeriesPattern1[Cents] = SeriesPattern1(client, 'unrealized_loss_cents')
        self.negative: SeriesPattern1[Dollars] = SeriesPattern1(client, 'unrealized_loss_neg')
        self.to_mcap: BpsPercentRatioPattern2 = BpsPercentRatioPattern2(client, 'unrealized_loss_to_mcap')
        self.to_own_gross_pnl: BpsPercentRatioPattern2 = BpsPercentRatioPattern2(client, 'unrealized_loss_to_own_gross_pnl')

class SeriesTree_Cohorts_Utxo_All_Unrealized_NetPnl:
    """Series tree node."""
    
    def __init__(self, client: BrkClient, base_path: str = ''):
        self.usd: SeriesPattern1[Dollars] = SeriesPattern1(client, 'net_unrealized_pnl')
        self.cents: SeriesPattern1[CentsSigned] = SeriesPattern1(client, 'net_unrealized_pnl_cents')
        self.to_own_gross_pnl: BpsPercentRatioPattern = BpsPercentRatioPattern(client, 'net_unrealized_pnl_to_own_gross_pnl')

class SeriesTree_Cohorts_Utxo_All_Unrealized_Sentiment:
    """Series tree node."""
    
    def __init__(self, client: BrkClient, base_path: str = ''):
        self.pain_index: CentsUsdPattern3 = CentsUsdPattern3(client, 'pain_index')
        self.greed_index: CentsUsdPattern3 = CentsUsdPattern3(client, 'greed_index')
        self.net: CentsUsdPattern = CentsUsdPattern(client, 'net_sentiment')

class SeriesTree_Cohorts_Utxo_All_Unrealized:
    """Series tree node."""
    
    def __init__(self, client: BrkClient, base_path: str = ''):
        self.nupl: BpsRatioPattern = BpsRatioPattern(client, 'nupl')
        self.profit: SeriesTree_Cohorts_Utxo_All_Unrealized_Profit = SeriesTree_Cohorts_Utxo_All_Unrealized_Profit(client)
        self.loss: SeriesTree_Cohorts_Utxo_All_Unrealized_Loss = SeriesTree_Cohorts_Utxo_All_Unrealized_Loss(client)
        self.net_pnl: SeriesTree_Cohorts_Utxo_All_Unrealized_NetPnl = SeriesTree_Cohorts_Utxo_All_Unrealized_NetPnl(client)
        self.gross_pnl: CentsUsdPattern3 = CentsUsdPattern3(client, 'unrealized_gross_pnl')
        self.invested_capital: InPattern2 = InPattern2(client, 'invested_capital_in')
        self.capitalized_cap_in_profit_raw: SeriesPattern18[CentsSquaredSats] = SeriesPattern18(client, 'capitalized_cap_in_profit_raw')
        self.capitalized_cap_in_loss_raw: SeriesPattern18[CentsSquaredSats] = SeriesPattern18(client, 'capitalized_cap_in_loss_raw')
        self.sentiment: SeriesTree_Cohorts_Utxo_All_Unrealized_Sentiment = SeriesTree_Cohorts_Utxo_All_Unrealized_Sentiment(client)

class SeriesTree_Cohorts_Utxo_All:
    """Series tree node."""
    
    def __init__(self, client: BrkClient, base_path: str = ''):
        self.supply: DeltaDominanceHalfInTotalPattern2 = DeltaDominanceHalfInTotalPattern2(client, 'supply')
        self.outputs: SeriesTree_Cohorts_Utxo_All_Outputs = SeriesTree_Cohorts_Utxo_All_Outputs(client)
        self.activity: SeriesTree_Cohorts_Utxo_All_Activity = SeriesTree_Cohorts_Utxo_All_Activity(client)
        self.realized: SeriesTree_Cohorts_Utxo_All_Realized = SeriesTree_Cohorts_Utxo_All_Realized(client)
        self.cost_basis: SeriesTree_Cohorts_Utxo_All_CostBasis = SeriesTree_Cohorts_Utxo_All_CostBasis(client)
        self.unrealized: SeriesTree_Cohorts_Utxo_All_Unrealized = SeriesTree_Cohorts_Utxo_All_Unrealized(client)
        self.invested_capital: InPattern = InPattern(client, 'invested_capital_in')

class SeriesTree_Cohorts_Utxo_Sth_Realized_Price_StdDev_All:
    """Series tree node."""
    
    def __init__(self, client: BrkClient, base_path: str = ''):
        self.sd: SeriesPattern1[StoredF32] = SeriesPattern1(client, 'sth_realized_price_ratio_sd')
        self.zscore: SeriesPattern1[StoredF32] = SeriesPattern1(client, 'sth_realized_price_ratio_zscore')
        self._0sd: CentsSatsUsdPattern = CentsSatsUsdPattern(client, 'sth_realized_price_0sd')
        self.p0_5sd: PriceRatioPattern = PriceRatioPattern(client, 'sth_realized_price', 'p0_5sd')
        self.p1sd: PriceRatioPattern = PriceRatioPattern(client, 'sth_realized_price', 'p1sd')
        self.p1_5sd: PriceRatioPattern = PriceRatioPattern(client, 'sth_realized_price', 'p1_5sd')
        self.p2sd: PriceRatioPattern = PriceRatioPattern(client, 'sth_realized_price', 'p2sd')
        self.p2_5sd: PriceRatioPattern = PriceRatioPattern(client, 'sth_realized_price', 'p2_5sd')
        self.p3sd: PriceRatioPattern = PriceRatioPattern(client, 'sth_realized_price', 'p3sd')
        self.m0_5sd: PriceRatioPattern = PriceRatioPattern(client, 'sth_realized_price', 'm0_5sd')
        self.m1sd: PriceRatioPattern = PriceRatioPattern(client, 'sth_realized_price', 'm1sd')
        self.m1_5sd: PriceRatioPattern = PriceRatioPattern(client, 'sth_realized_price', 'm1_5sd')
        self.m2sd: PriceRatioPattern = PriceRatioPattern(client, 'sth_realized_price', 'm2sd')
        self.m2_5sd: PriceRatioPattern = PriceRatioPattern(client, 'sth_realized_price', 'm2_5sd')
        self.m3sd: PriceRatioPattern = PriceRatioPattern(client, 'sth_realized_price', 'm3sd')

class SeriesTree_Cohorts_Utxo_Sth_Realized_Price_StdDev_4y:
    """Series tree node."""
    
    def __init__(self, client: BrkClient, base_path: str = ''):
        self.sd: SeriesPattern1[StoredF32] = SeriesPattern1(client, 'sth_realized_price_ratio_sd_4y')
        self.zscore: SeriesPattern1[StoredF32] = SeriesPattern1(client, 'sth_realized_price_ratio_zscore_4y')
        self._0sd: CentsSatsUsdPattern = CentsSatsUsdPattern(client, 'sth_realized_price_0sd_4y')
        self.p0_5sd: PriceRatioPattern = PriceRatioPattern(client, 'sth_realized_price', 'p0_5sd_4y')
        self.p1sd: PriceRatioPattern = PriceRatioPattern(client, 'sth_realized_price', 'p1sd_4y')
        self.p1_5sd: PriceRatioPattern = PriceRatioPattern(client, 'sth_realized_price', 'p1_5sd_4y')
        self.p2sd: PriceRatioPattern = PriceRatioPattern(client, 'sth_realized_price', 'p2sd_4y')
        self.p2_5sd: PriceRatioPattern = PriceRatioPattern(client, 'sth_realized_price', 'p2_5sd_4y')
        self.p3sd: PriceRatioPattern = PriceRatioPattern(client, 'sth_realized_price', 'p3sd_4y')
        self.m0_5sd: PriceRatioPattern = PriceRatioPattern(client, 'sth_realized_price', 'm0_5sd_4y')
        self.m1sd: PriceRatioPattern = PriceRatioPattern(client, 'sth_realized_price', 'm1sd_4y')
        self.m1_5sd: PriceRatioPattern = PriceRatioPattern(client, 'sth_realized_price', 'm1_5sd_4y')
        self.m2sd: PriceRatioPattern = PriceRatioPattern(client, 'sth_realized_price', 'm2sd_4y')
        self.m2_5sd: PriceRatioPattern = PriceRatioPattern(client, 'sth_realized_price', 'm2_5sd_4y')
        self.m3sd: PriceRatioPattern = PriceRatioPattern(client, 'sth_realized_price', 'm3sd_4y')

class SeriesTree_Cohorts_Utxo_Sth_Realized_Price_StdDev_2y:
    """Series tree node."""
    
    def __init__(self, client: BrkClient, base_path: str = ''):
        self.sd: SeriesPattern1[StoredF32] = SeriesPattern1(client, 'sth_realized_price_ratio_sd_2y')
        self.zscore: SeriesPattern1[StoredF32] = SeriesPattern1(client, 'sth_realized_price_ratio_zscore_2y')
        self._0sd: CentsSatsUsdPattern = CentsSatsUsdPattern(client, 'sth_realized_price_0sd_2y')
        self.p0_5sd: PriceRatioPattern = PriceRatioPattern(client, 'sth_realized_price', 'p0_5sd_2y')
        self.p1sd: PriceRatioPattern = PriceRatioPattern(client, 'sth_realized_price', 'p1sd_2y')
        self.p1_5sd: PriceRatioPattern = PriceRatioPattern(client, 'sth_realized_price', 'p1_5sd_2y')
        self.p2sd: PriceRatioPattern = PriceRatioPattern(client, 'sth_realized_price', 'p2sd_2y')
        self.p2_5sd: PriceRatioPattern = PriceRatioPattern(client, 'sth_realized_price', 'p2_5sd_2y')
        self.p3sd: PriceRatioPattern = PriceRatioPattern(client, 'sth_realized_price', 'p3sd_2y')
        self.m0_5sd: PriceRatioPattern = PriceRatioPattern(client, 'sth_realized_price', 'm0_5sd_2y')
        self.m1sd: PriceRatioPattern = PriceRatioPattern(client, 'sth_realized_price', 'm1sd_2y')
        self.m1_5sd: PriceRatioPattern = PriceRatioPattern(client, 'sth_realized_price', 'm1_5sd_2y')
        self.m2sd: PriceRatioPattern = PriceRatioPattern(client, 'sth_realized_price', 'm2sd_2y')
        self.m2_5sd: PriceRatioPattern = PriceRatioPattern(client, 'sth_realized_price', 'm2_5sd_2y')
        self.m3sd: PriceRatioPattern = PriceRatioPattern(client, 'sth_realized_price', 'm3sd_2y')

class SeriesTree_Cohorts_Utxo_Sth_Realized_Price_StdDev_1y:
    """Series tree node."""
    
    def __init__(self, client: BrkClient, base_path: str = ''):
        self.sd: SeriesPattern1[StoredF32] = SeriesPattern1(client, 'sth_realized_price_ratio_sd_1y')
        self.zscore: SeriesPattern1[StoredF32] = SeriesPattern1(client, 'sth_realized_price_ratio_zscore_1y')
        self._0sd: CentsSatsUsdPattern = CentsSatsUsdPattern(client, 'sth_realized_price_0sd_1y')
        self.p0_5sd: PriceRatioPattern = PriceRatioPattern(client, 'sth_realized_price', 'p0_5sd_1y')
        self.p1sd: PriceRatioPattern = PriceRatioPattern(client, 'sth_realized_price', 'p1sd_1y')
        self.p1_5sd: PriceRatioPattern = PriceRatioPattern(client, 'sth_realized_price', 'p1_5sd_1y')
        self.p2sd: PriceRatioPattern = PriceRatioPattern(client, 'sth_realized_price', 'p2sd_1y')
        self.p2_5sd: PriceRatioPattern = PriceRatioPattern(client, 'sth_realized_price', 'p2_5sd_1y')
        self.p3sd: PriceRatioPattern = PriceRatioPattern(client, 'sth_realized_price', 'p3sd_1y')
        self.m0_5sd: PriceRatioPattern = PriceRatioPattern(client, 'sth_realized_price', 'm0_5sd_1y')
        self.m1sd: PriceRatioPattern = PriceRatioPattern(client, 'sth_realized_price', 'm1sd_1y')
        self.m1_5sd: PriceRatioPattern = PriceRatioPattern(client, 'sth_realized_price', 'm1_5sd_1y')
        self.m2sd: PriceRatioPattern = PriceRatioPattern(client, 'sth_realized_price', 'm2sd_1y')
        self.m2_5sd: PriceRatioPattern = PriceRatioPattern(client, 'sth_realized_price', 'm2_5sd_1y')
        self.m3sd: PriceRatioPattern = PriceRatioPattern(client, 'sth_realized_price', 'm3sd_1y')

class SeriesTree_Cohorts_Utxo_Sth_Realized_Price_StdDev:
    """Series tree node."""
    
    def __init__(self, client: BrkClient, base_path: str = ''):
        self.all: SeriesTree_Cohorts_Utxo_Sth_Realized_Price_StdDev_All = SeriesTree_Cohorts_Utxo_Sth_Realized_Price_StdDev_All(client)
        self._4y: SeriesTree_Cohorts_Utxo_Sth_Realized_Price_StdDev_4y = SeriesTree_Cohorts_Utxo_Sth_Realized_Price_StdDev_4y(client)
        self._2y: SeriesTree_Cohorts_Utxo_Sth_Realized_Price_StdDev_2y = SeriesTree_Cohorts_Utxo_Sth_Realized_Price_StdDev_2y(client)
        self._1y: SeriesTree_Cohorts_Utxo_Sth_Realized_Price_StdDev_1y = SeriesTree_Cohorts_Utxo_Sth_Realized_Price_StdDev_1y(client)

class SeriesTree_Cohorts_Utxo_Sth_Realized_Price:
    """Series tree node."""
    
    def __init__(self, client: BrkClient, base_path: str = ''):
        self.usd: SeriesPattern1[Dollars] = SeriesPattern1(client, 'sth_realized_price')
        self.cents: SeriesPattern1[Cents] = SeriesPattern1(client, 'sth_realized_price_cents')
        self.sats: SeriesPattern1[SatsFract] = SeriesPattern1(client, 'sth_realized_price_sats')
        self.bps: SeriesPattern1[BasisPoints32] = SeriesPattern1(client, 'sth_realized_price_ratio_bps')
        self.ratio: SeriesPattern1[StoredF32] = SeriesPattern1(client, 'sth_realized_price_ratio')
        self.percentiles: Pct0Pct1Pct2Pct5Pct95Pct98Pct99Pattern = Pct0Pct1Pct2Pct5Pct95Pct98Pct99Pattern(client, 'sth_realized_price')
        self.sma: _1m1w1y2y4yAllPattern = _1m1w1y2y4yAllPattern(client, 'sth_realized_price_ratio_sma')
        self.std_dev: SeriesTree_Cohorts_Utxo_Sth_Realized_Price_StdDev = SeriesTree_Cohorts_Utxo_Sth_Realized_Price_StdDev(client)

class SeriesTree_Cohorts_Utxo_Sth_Realized:
    """Series tree node."""
    
    def __init__(self, client: BrkClient, base_path: str = ''):
        self.cap: CentsDeltaToUsdPattern = CentsDeltaToUsdPattern(client, 'sth_realized_cap')
        self.profit: BlockCumulativeSumPattern = BlockCumulativeSumPattern(client, 'sth_realized_profit')
        self.loss: BlockCumulativeNegativeSumPattern = BlockCumulativeNegativeSumPattern(client, 'sth_realized_loss')
        self.price: SeriesTree_Cohorts_Utxo_Sth_Realized_Price = SeriesTree_Cohorts_Utxo_Sth_Realized_Price(client)
        self.mvrv: SeriesPattern1[StoredF32] = SeriesPattern1(client, 'sth_mvrv')
        self.net_pnl: BlockChangeCumulativeDeltaSumPattern = BlockChangeCumulativeDeltaSumPattern(client, 'sth_net')
        self.sopr: AdjustedRatioValuePattern = AdjustedRatioValuePattern(client, 'sth')
        self.gross_pnl: BlockCumulativeSumPattern = BlockCumulativeSumPattern(client, 'sth_realized_gross_pnl')
        self.sell_side_risk_ratio: _1m1w1y24hPattern8 = _1m1w1y24hPattern8(client, 'sth_sell_side_risk_ratio')
        self.peak_regret: BlockCumulativeSumPattern = BlockCumulativeSumPattern(client, 'sth_realized_peak_regret')
        self.capitalized: PricePattern = PricePattern(client, 'sth_capitalized_price')
        self.profit_to_loss_ratio: _1m1w1y24hPattern[StoredF64] = _1m1w1y24hPattern(client, 'sth_realized_profit_to_loss_ratio')

class SeriesTree_Cohorts_Utxo_Sth:
    """Series tree node."""
    
    def __init__(self, client: BrkClient, base_path: str = ''):
        self.supply: DeltaDominanceHalfInTotalPattern2 = DeltaDominanceHalfInTotalPattern2(client, 'sth_supply')
        self.outputs: SpendingSpentUnspentPattern = SpendingSpentUnspentPattern(client, 'sth')
        self.activity: CoindaysCoinyearsDormancyTransferPattern = CoindaysCoinyearsDormancyTransferPattern(client, 'sth')
        self.realized: SeriesTree_Cohorts_Utxo_Sth_Realized = SeriesTree_Cohorts_Utxo_Sth_Realized(client)
        self.cost_basis: InMaxMinPerSupplyPattern = InMaxMinPerSupplyPattern(client, 'sth')
        self.unrealized: CapitalizedGrossInvestedLossNetNuplProfitSentimentPattern2 = CapitalizedGrossInvestedLossNetNuplProfitSentimentPattern2(client, 'sth')
        self.invested_capital: InPattern = InPattern(client, 'sth_invested_capital_in')

class SeriesTree_Cohorts_Utxo_Lth_Realized_Price_StdDev_All:
    """Series tree node."""
    
    def __init__(self, client: BrkClient, base_path: str = ''):
        self.sd: SeriesPattern1[StoredF32] = SeriesPattern1(client, 'lth_realized_price_ratio_sd')
        self.zscore: SeriesPattern1[StoredF32] = SeriesPattern1(client, 'lth_realized_price_ratio_zscore')
        self._0sd: CentsSatsUsdPattern = CentsSatsUsdPattern(client, 'lth_realized_price_0sd')
        self.p0_5sd: PriceRatioPattern = PriceRatioPattern(client, 'lth_realized_price', 'p0_5sd')
        self.p1sd: PriceRatioPattern = PriceRatioPattern(client, 'lth_realized_price', 'p1sd')
        self.p1_5sd: PriceRatioPattern = PriceRatioPattern(client, 'lth_realized_price', 'p1_5sd')
        self.p2sd: PriceRatioPattern = PriceRatioPattern(client, 'lth_realized_price', 'p2sd')
        self.p2_5sd: PriceRatioPattern = PriceRatioPattern(client, 'lth_realized_price', 'p2_5sd')
        self.p3sd: PriceRatioPattern = PriceRatioPattern(client, 'lth_realized_price', 'p3sd')
        self.m0_5sd: PriceRatioPattern = PriceRatioPattern(client, 'lth_realized_price', 'm0_5sd')
        self.m1sd: PriceRatioPattern = PriceRatioPattern(client, 'lth_realized_price', 'm1sd')
        self.m1_5sd: PriceRatioPattern = PriceRatioPattern(client, 'lth_realized_price', 'm1_5sd')
        self.m2sd: PriceRatioPattern = PriceRatioPattern(client, 'lth_realized_price', 'm2sd')
        self.m2_5sd: PriceRatioPattern = PriceRatioPattern(client, 'lth_realized_price', 'm2_5sd')
        self.m3sd: PriceRatioPattern = PriceRatioPattern(client, 'lth_realized_price', 'm3sd')

class SeriesTree_Cohorts_Utxo_Lth_Realized_Price_StdDev_4y:
    """Series tree node."""
    
    def __init__(self, client: BrkClient, base_path: str = ''):
        self.sd: SeriesPattern1[StoredF32] = SeriesPattern1(client, 'lth_realized_price_ratio_sd_4y')
        self.zscore: SeriesPattern1[StoredF32] = SeriesPattern1(client, 'lth_realized_price_ratio_zscore_4y')
        self._0sd: CentsSatsUsdPattern = CentsSatsUsdPattern(client, 'lth_realized_price_0sd_4y')
        self.p0_5sd: PriceRatioPattern = PriceRatioPattern(client, 'lth_realized_price', 'p0_5sd_4y')
        self.p1sd: PriceRatioPattern = PriceRatioPattern(client, 'lth_realized_price', 'p1sd_4y')
        self.p1_5sd: PriceRatioPattern = PriceRatioPattern(client, 'lth_realized_price', 'p1_5sd_4y')
        self.p2sd: PriceRatioPattern = PriceRatioPattern(client, 'lth_realized_price', 'p2sd_4y')
        self.p2_5sd: PriceRatioPattern = PriceRatioPattern(client, 'lth_realized_price', 'p2_5sd_4y')
        self.p3sd: PriceRatioPattern = PriceRatioPattern(client, 'lth_realized_price', 'p3sd_4y')
        self.m0_5sd: PriceRatioPattern = PriceRatioPattern(client, 'lth_realized_price', 'm0_5sd_4y')
        self.m1sd: PriceRatioPattern = PriceRatioPattern(client, 'lth_realized_price', 'm1sd_4y')
        self.m1_5sd: PriceRatioPattern = PriceRatioPattern(client, 'lth_realized_price', 'm1_5sd_4y')
        self.m2sd: PriceRatioPattern = PriceRatioPattern(client, 'lth_realized_price', 'm2sd_4y')
        self.m2_5sd: PriceRatioPattern = PriceRatioPattern(client, 'lth_realized_price', 'm2_5sd_4y')
        self.m3sd: PriceRatioPattern = PriceRatioPattern(client, 'lth_realized_price', 'm3sd_4y')

class SeriesTree_Cohorts_Utxo_Lth_Realized_Price_StdDev_2y:
    """Series tree node."""
    
    def __init__(self, client: BrkClient, base_path: str = ''):
        self.sd: SeriesPattern1[StoredF32] = SeriesPattern1(client, 'lth_realized_price_ratio_sd_2y')
        self.zscore: SeriesPattern1[StoredF32] = SeriesPattern1(client, 'lth_realized_price_ratio_zscore_2y')
        self._0sd: CentsSatsUsdPattern = CentsSatsUsdPattern(client, 'lth_realized_price_0sd_2y')
        self.p0_5sd: PriceRatioPattern = PriceRatioPattern(client, 'lth_realized_price', 'p0_5sd_2y')
        self.p1sd: PriceRatioPattern = PriceRatioPattern(client, 'lth_realized_price', 'p1sd_2y')
        self.p1_5sd: PriceRatioPattern = PriceRatioPattern(client, 'lth_realized_price', 'p1_5sd_2y')
        self.p2sd: PriceRatioPattern = PriceRatioPattern(client, 'lth_realized_price', 'p2sd_2y')
        self.p2_5sd: PriceRatioPattern = PriceRatioPattern(client, 'lth_realized_price', 'p2_5sd_2y')
        self.p3sd: PriceRatioPattern = PriceRatioPattern(client, 'lth_realized_price', 'p3sd_2y')
        self.m0_5sd: PriceRatioPattern = PriceRatioPattern(client, 'lth_realized_price', 'm0_5sd_2y')
        self.m1sd: PriceRatioPattern = PriceRatioPattern(client, 'lth_realized_price', 'm1sd_2y')
        self.m1_5sd: PriceRatioPattern = PriceRatioPattern(client, 'lth_realized_price', 'm1_5sd_2y')
        self.m2sd: PriceRatioPattern = PriceRatioPattern(client, 'lth_realized_price', 'm2sd_2y')
        self.m2_5sd: PriceRatioPattern = PriceRatioPattern(client, 'lth_realized_price', 'm2_5sd_2y')
        self.m3sd: PriceRatioPattern = PriceRatioPattern(client, 'lth_realized_price', 'm3sd_2y')

class SeriesTree_Cohorts_Utxo_Lth_Realized_Price_StdDev_1y:
    """Series tree node."""
    
    def __init__(self, client: BrkClient, base_path: str = ''):
        self.sd: SeriesPattern1[StoredF32] = SeriesPattern1(client, 'lth_realized_price_ratio_sd_1y')
        self.zscore: SeriesPattern1[StoredF32] = SeriesPattern1(client, 'lth_realized_price_ratio_zscore_1y')
        self._0sd: CentsSatsUsdPattern = CentsSatsUsdPattern(client, 'lth_realized_price_0sd_1y')
        self.p0_5sd: PriceRatioPattern = PriceRatioPattern(client, 'lth_realized_price', 'p0_5sd_1y')
        self.p1sd: PriceRatioPattern = PriceRatioPattern(client, 'lth_realized_price', 'p1sd_1y')
        self.p1_5sd: PriceRatioPattern = PriceRatioPattern(client, 'lth_realized_price', 'p1_5sd_1y')
        self.p2sd: PriceRatioPattern = PriceRatioPattern(client, 'lth_realized_price', 'p2sd_1y')
        self.p2_5sd: PriceRatioPattern = PriceRatioPattern(client, 'lth_realized_price', 'p2_5sd_1y')
        self.p3sd: PriceRatioPattern = PriceRatioPattern(client, 'lth_realized_price', 'p3sd_1y')
        self.m0_5sd: PriceRatioPattern = PriceRatioPattern(client, 'lth_realized_price', 'm0_5sd_1y')
        self.m1sd: PriceRatioPattern = PriceRatioPattern(client, 'lth_realized_price', 'm1sd_1y')
        self.m1_5sd: PriceRatioPattern = PriceRatioPattern(client, 'lth_realized_price', 'm1_5sd_1y')
        self.m2sd: PriceRatioPattern = PriceRatioPattern(client, 'lth_realized_price', 'm2sd_1y')
        self.m2_5sd: PriceRatioPattern = PriceRatioPattern(client, 'lth_realized_price', 'm2_5sd_1y')
        self.m3sd: PriceRatioPattern = PriceRatioPattern(client, 'lth_realized_price', 'm3sd_1y')

class SeriesTree_Cohorts_Utxo_Lth_Realized_Price_StdDev:
    """Series tree node."""
    
    def __init__(self, client: BrkClient, base_path: str = ''):
        self.all: SeriesTree_Cohorts_Utxo_Lth_Realized_Price_StdDev_All = SeriesTree_Cohorts_Utxo_Lth_Realized_Price_StdDev_All(client)
        self._4y: SeriesTree_Cohorts_Utxo_Lth_Realized_Price_StdDev_4y = SeriesTree_Cohorts_Utxo_Lth_Realized_Price_StdDev_4y(client)
        self._2y: SeriesTree_Cohorts_Utxo_Lth_Realized_Price_StdDev_2y = SeriesTree_Cohorts_Utxo_Lth_Realized_Price_StdDev_2y(client)
        self._1y: SeriesTree_Cohorts_Utxo_Lth_Realized_Price_StdDev_1y = SeriesTree_Cohorts_Utxo_Lth_Realized_Price_StdDev_1y(client)

class SeriesTree_Cohorts_Utxo_Lth_Realized_Price:
    """Series tree node."""
    
    def __init__(self, client: BrkClient, base_path: str = ''):
        self.usd: SeriesPattern1[Dollars] = SeriesPattern1(client, 'lth_realized_price')
        self.cents: SeriesPattern1[Cents] = SeriesPattern1(client, 'lth_realized_price_cents')
        self.sats: SeriesPattern1[SatsFract] = SeriesPattern1(client, 'lth_realized_price_sats')
        self.bps: SeriesPattern1[BasisPoints32] = SeriesPattern1(client, 'lth_realized_price_ratio_bps')
        self.ratio: SeriesPattern1[StoredF32] = SeriesPattern1(client, 'lth_realized_price_ratio')
        self.percentiles: Pct0Pct1Pct2Pct5Pct95Pct98Pct99Pattern = Pct0Pct1Pct2Pct5Pct95Pct98Pct99Pattern(client, 'lth_realized_price')
        self.sma: _1m1w1y2y4yAllPattern = _1m1w1y2y4yAllPattern(client, 'lth_realized_price_ratio_sma')
        self.std_dev: SeriesTree_Cohorts_Utxo_Lth_Realized_Price_StdDev = SeriesTree_Cohorts_Utxo_Lth_Realized_Price_StdDev(client)

class SeriesTree_Cohorts_Utxo_Lth_Realized:
    """Series tree node."""
    
    def __init__(self, client: BrkClient, base_path: str = ''):
        self.cap: CentsDeltaToUsdPattern = CentsDeltaToUsdPattern(client, 'lth_realized_cap')
        self.profit: BlockCumulativeSumPattern = BlockCumulativeSumPattern(client, 'lth_realized_profit')
        self.loss: BlockCumulativeNegativeSumPattern = BlockCumulativeNegativeSumPattern(client, 'lth_realized_loss')
        self.price: SeriesTree_Cohorts_Utxo_Lth_Realized_Price = SeriesTree_Cohorts_Utxo_Lth_Realized_Price(client)
        self.mvrv: SeriesPattern1[StoredF32] = SeriesPattern1(client, 'lth_mvrv')
        self.net_pnl: BlockChangeCumulativeDeltaSumPattern = BlockChangeCumulativeDeltaSumPattern(client, 'lth_net')
        self.sopr: RatioValuePattern2 = RatioValuePattern2(client, 'lth')
        self.gross_pnl: BlockCumulativeSumPattern = BlockCumulativeSumPattern(client, 'lth_realized_gross_pnl')
        self.sell_side_risk_ratio: _1m1w1y24hPattern8 = _1m1w1y24hPattern8(client, 'lth_sell_side_risk_ratio')
        self.peak_regret: BlockCumulativeSumPattern = BlockCumulativeSumPattern(client, 'lth_realized_peak_regret')
        self.capitalized: PricePattern = PricePattern(client, 'lth_capitalized_price')
        self.profit_to_loss_ratio: _1m1w1y24hPattern[StoredF64] = _1m1w1y24hPattern(client, 'lth_realized_profit_to_loss_ratio')

class SeriesTree_Cohorts_Utxo_Lth:
    """Series tree node."""
    
    def __init__(self, client: BrkClient, base_path: str = ''):
        self.supply: DeltaDominanceHalfInTotalPattern2 = DeltaDominanceHalfInTotalPattern2(client, 'lth_supply')
        self.outputs: SpendingSpentUnspentPattern = SpendingSpentUnspentPattern(client, 'lth')
        self.activity: CoindaysCoinyearsDormancyTransferPattern = CoindaysCoinyearsDormancyTransferPattern(client, 'lth')
        self.realized: SeriesTree_Cohorts_Utxo_Lth_Realized = SeriesTree_Cohorts_Utxo_Lth_Realized(client)
        self.cost_basis: InMaxMinPerSupplyPattern = InMaxMinPerSupplyPattern(client, 'lth')
        self.unrealized: CapitalizedGrossInvestedLossNetNuplProfitSentimentPattern2 = CapitalizedGrossInvestedLossNetNuplProfitSentimentPattern2(client, 'lth')
        self.invested_capital: InPattern = InPattern(client, 'lth_invested_capital_in')

class SeriesTree_Cohorts_Utxo_AgeRange:
    """Series tree node."""
    
    def __init__(self, client: BrkClient, base_path: str = ''):
        self.under_1h: ActivityOutputsRealizedSupplyUnrealizedPattern = ActivityOutputsRealizedSupplyUnrealizedPattern(client, 'utxos_under_1h_old')
        self._1h_to_1d: ActivityOutputsRealizedSupplyUnrealizedPattern = ActivityOutputsRealizedSupplyUnrealizedPattern(client, 'utxos_1h_to_1d_old')
        self._1d_to_1w: ActivityOutputsRealizedSupplyUnrealizedPattern = ActivityOutputsRealizedSupplyUnrealizedPattern(client, 'utxos_1d_to_1w_old')
        self._1w_to_1m: ActivityOutputsRealizedSupplyUnrealizedPattern = ActivityOutputsRealizedSupplyUnrealizedPattern(client, 'utxos_1w_to_1m_old')
        self._1m_to_2m: ActivityOutputsRealizedSupplyUnrealizedPattern = ActivityOutputsRealizedSupplyUnrealizedPattern(client, 'utxos_1m_to_2m_old')
        self._2m_to_3m: ActivityOutputsRealizedSupplyUnrealizedPattern = ActivityOutputsRealizedSupplyUnrealizedPattern(client, 'utxos_2m_to_3m_old')
        self._3m_to_4m: ActivityOutputsRealizedSupplyUnrealizedPattern = ActivityOutputsRealizedSupplyUnrealizedPattern(client, 'utxos_3m_to_4m_old')
        self._4m_to_5m: ActivityOutputsRealizedSupplyUnrealizedPattern = ActivityOutputsRealizedSupplyUnrealizedPattern(client, 'utxos_4m_to_5m_old')
        self._5m_to_6m: ActivityOutputsRealizedSupplyUnrealizedPattern = ActivityOutputsRealizedSupplyUnrealizedPattern(client, 'utxos_5m_to_6m_old')
        self._6m_to_1y: ActivityOutputsRealizedSupplyUnrealizedPattern = ActivityOutputsRealizedSupplyUnrealizedPattern(client, 'utxos_6m_to_1y_old')
        self._1y_to_2y: ActivityOutputsRealizedSupplyUnrealizedPattern = ActivityOutputsRealizedSupplyUnrealizedPattern(client, 'utxos_1y_to_2y_old')
        self._2y_to_3y: ActivityOutputsRealizedSupplyUnrealizedPattern = ActivityOutputsRealizedSupplyUnrealizedPattern(client, 'utxos_2y_to_3y_old')
        self._3y_to_4y: ActivityOutputsRealizedSupplyUnrealizedPattern = ActivityOutputsRealizedSupplyUnrealizedPattern(client, 'utxos_3y_to_4y_old')
        self._4y_to_5y: ActivityOutputsRealizedSupplyUnrealizedPattern = ActivityOutputsRealizedSupplyUnrealizedPattern(client, 'utxos_4y_to_5y_old')
        self._5y_to_6y: ActivityOutputsRealizedSupplyUnrealizedPattern = ActivityOutputsRealizedSupplyUnrealizedPattern(client, 'utxos_5y_to_6y_old')
        self._6y_to_7y: ActivityOutputsRealizedSupplyUnrealizedPattern = ActivityOutputsRealizedSupplyUnrealizedPattern(client, 'utxos_6y_to_7y_old')
        self._7y_to_8y: ActivityOutputsRealizedSupplyUnrealizedPattern = ActivityOutputsRealizedSupplyUnrealizedPattern(client, 'utxos_7y_to_8y_old')
        self._8y_to_10y: ActivityOutputsRealizedSupplyUnrealizedPattern = ActivityOutputsRealizedSupplyUnrealizedPattern(client, 'utxos_8y_to_10y_old')
        self._10y_to_12y: ActivityOutputsRealizedSupplyUnrealizedPattern = ActivityOutputsRealizedSupplyUnrealizedPattern(client, 'utxos_10y_to_12y_old')
        self._12y_to_15y: ActivityOutputsRealizedSupplyUnrealizedPattern = ActivityOutputsRealizedSupplyUnrealizedPattern(client, 'utxos_12y_to_15y_old')
        self.over_15y: ActivityOutputsRealizedSupplyUnrealizedPattern = ActivityOutputsRealizedSupplyUnrealizedPattern(client, 'utxos_over_15y_old')

class SeriesTree_Cohorts_Utxo_UnderAge:
    """Series tree node."""
    
    def __init__(self, client: BrkClient, base_path: str = ''):
        self._1w: ActivityOutputsRealizedSupplyUnrealizedPattern = ActivityOutputsRealizedSupplyUnrealizedPattern(client, 'utxos_under_1w_old')
        self._1m: ActivityOutputsRealizedSupplyUnrealizedPattern = ActivityOutputsRealizedSupplyUnrealizedPattern(client, 'utxos_under_1m_old')
        self._2m: ActivityOutputsRealizedSupplyUnrealizedPattern = ActivityOutputsRealizedSupplyUnrealizedPattern(client, 'utxos_under_2m_old')
        self._3m: ActivityOutputsRealizedSupplyUnrealizedPattern = ActivityOutputsRealizedSupplyUnrealizedPattern(client, 'utxos_under_3m_old')
        self._4m: ActivityOutputsRealizedSupplyUnrealizedPattern = ActivityOutputsRealizedSupplyUnrealizedPattern(client, 'utxos_under_4m_old')
        self._5m: ActivityOutputsRealizedSupplyUnrealizedPattern = ActivityOutputsRealizedSupplyUnrealizedPattern(client, 'utxos_under_5m_old')
        self._6m: ActivityOutputsRealizedSupplyUnrealizedPattern = ActivityOutputsRealizedSupplyUnrealizedPattern(client, 'utxos_under_6m_old')
        self._1y: ActivityOutputsRealizedSupplyUnrealizedPattern = ActivityOutputsRealizedSupplyUnrealizedPattern(client, 'utxos_under_1y_old')
        self._2y: ActivityOutputsRealizedSupplyUnrealizedPattern = ActivityOutputsRealizedSupplyUnrealizedPattern(client, 'utxos_under_2y_old')
        self._3y: ActivityOutputsRealizedSupplyUnrealizedPattern = ActivityOutputsRealizedSupplyUnrealizedPattern(client, 'utxos_under_3y_old')
        self._4y: ActivityOutputsRealizedSupplyUnrealizedPattern = ActivityOutputsRealizedSupplyUnrealizedPattern(client, 'utxos_under_4y_old')
        self._5y: ActivityOutputsRealizedSupplyUnrealizedPattern = ActivityOutputsRealizedSupplyUnrealizedPattern(client, 'utxos_under_5y_old')
        self._6y: ActivityOutputsRealizedSupplyUnrealizedPattern = ActivityOutputsRealizedSupplyUnrealizedPattern(client, 'utxos_under_6y_old')
        self._7y: ActivityOutputsRealizedSupplyUnrealizedPattern = ActivityOutputsRealizedSupplyUnrealizedPattern(client, 'utxos_under_7y_old')
        self._8y: ActivityOutputsRealizedSupplyUnrealizedPattern = ActivityOutputsRealizedSupplyUnrealizedPattern(client, 'utxos_under_8y_old')
        self._10y: ActivityOutputsRealizedSupplyUnrealizedPattern = ActivityOutputsRealizedSupplyUnrealizedPattern(client, 'utxos_under_10y_old')
        self._12y: ActivityOutputsRealizedSupplyUnrealizedPattern = ActivityOutputsRealizedSupplyUnrealizedPattern(client, 'utxos_under_12y_old')
        self._15y: ActivityOutputsRealizedSupplyUnrealizedPattern = ActivityOutputsRealizedSupplyUnrealizedPattern(client, 'utxos_under_15y_old')

class SeriesTree_Cohorts_Utxo_OverAge:
    """Series tree node."""
    
    def __init__(self, client: BrkClient, base_path: str = ''):
        self._1d: ActivityOutputsRealizedSupplyUnrealizedPattern = ActivityOutputsRealizedSupplyUnrealizedPattern(client, 'utxos_over_1d_old')
        self._1w: ActivityOutputsRealizedSupplyUnrealizedPattern = ActivityOutputsRealizedSupplyUnrealizedPattern(client, 'utxos_over_1w_old')
        self._1m: ActivityOutputsRealizedSupplyUnrealizedPattern = ActivityOutputsRealizedSupplyUnrealizedPattern(client, 'utxos_over_1m_old')
        self._2m: ActivityOutputsRealizedSupplyUnrealizedPattern = ActivityOutputsRealizedSupplyUnrealizedPattern(client, 'utxos_over_2m_old')
        self._3m: ActivityOutputsRealizedSupplyUnrealizedPattern = ActivityOutputsRealizedSupplyUnrealizedPattern(client, 'utxos_over_3m_old')
        self._4m: ActivityOutputsRealizedSupplyUnrealizedPattern = ActivityOutputsRealizedSupplyUnrealizedPattern(client, 'utxos_over_4m_old')
        self._5m: ActivityOutputsRealizedSupplyUnrealizedPattern = ActivityOutputsRealizedSupplyUnrealizedPattern(client, 'utxos_over_5m_old')
        self._6m: ActivityOutputsRealizedSupplyUnrealizedPattern = ActivityOutputsRealizedSupplyUnrealizedPattern(client, 'utxos_over_6m_old')
        self._1y: ActivityOutputsRealizedSupplyUnrealizedPattern = ActivityOutputsRealizedSupplyUnrealizedPattern(client, 'utxos_over_1y_old')
        self._2y: ActivityOutputsRealizedSupplyUnrealizedPattern = ActivityOutputsRealizedSupplyUnrealizedPattern(client, 'utxos_over_2y_old')
        self._3y: ActivityOutputsRealizedSupplyUnrealizedPattern = ActivityOutputsRealizedSupplyUnrealizedPattern(client, 'utxos_over_3y_old')
        self._4y: ActivityOutputsRealizedSupplyUnrealizedPattern = ActivityOutputsRealizedSupplyUnrealizedPattern(client, 'utxos_over_4y_old')
        self._5y: ActivityOutputsRealizedSupplyUnrealizedPattern = ActivityOutputsRealizedSupplyUnrealizedPattern(client, 'utxos_over_5y_old')
        self._6y: ActivityOutputsRealizedSupplyUnrealizedPattern = ActivityOutputsRealizedSupplyUnrealizedPattern(client, 'utxos_over_6y_old')
        self._7y: ActivityOutputsRealizedSupplyUnrealizedPattern = ActivityOutputsRealizedSupplyUnrealizedPattern(client, 'utxos_over_7y_old')
        self._8y: ActivityOutputsRealizedSupplyUnrealizedPattern = ActivityOutputsRealizedSupplyUnrealizedPattern(client, 'utxos_over_8y_old')
        self._10y: ActivityOutputsRealizedSupplyUnrealizedPattern = ActivityOutputsRealizedSupplyUnrealizedPattern(client, 'utxos_over_10y_old')
        self._12y: ActivityOutputsRealizedSupplyUnrealizedPattern = ActivityOutputsRealizedSupplyUnrealizedPattern(client, 'utxos_over_12y_old')

class SeriesTree_Cohorts_Utxo_Epoch:
    """Series tree node."""
    
    def __init__(self, client: BrkClient, base_path: str = ''):
        self._0: ActivityOutputsRealizedSupplyUnrealizedPattern = ActivityOutputsRealizedSupplyUnrealizedPattern(client, 'epoch_0')
        self._1: ActivityOutputsRealizedSupplyUnrealizedPattern = ActivityOutputsRealizedSupplyUnrealizedPattern(client, 'epoch_1')
        self._2: ActivityOutputsRealizedSupplyUnrealizedPattern = ActivityOutputsRealizedSupplyUnrealizedPattern(client, 'epoch_2')
        self._3: ActivityOutputsRealizedSupplyUnrealizedPattern = ActivityOutputsRealizedSupplyUnrealizedPattern(client, 'epoch_3')
        self._4: ActivityOutputsRealizedSupplyUnrealizedPattern = ActivityOutputsRealizedSupplyUnrealizedPattern(client, 'epoch_4')

class SeriesTree_Cohorts_Utxo_Class:
    """Series tree node."""
    
    def __init__(self, client: BrkClient, base_path: str = ''):
        self._2009: ActivityOutputsRealizedSupplyUnrealizedPattern = ActivityOutputsRealizedSupplyUnrealizedPattern(client, 'class_2009')
        self._2010: ActivityOutputsRealizedSupplyUnrealizedPattern = ActivityOutputsRealizedSupplyUnrealizedPattern(client, 'class_2010')
        self._2011: ActivityOutputsRealizedSupplyUnrealizedPattern = ActivityOutputsRealizedSupplyUnrealizedPattern(client, 'class_2011')
        self._2012: ActivityOutputsRealizedSupplyUnrealizedPattern = ActivityOutputsRealizedSupplyUnrealizedPattern(client, 'class_2012')
        self._2013: ActivityOutputsRealizedSupplyUnrealizedPattern = ActivityOutputsRealizedSupplyUnrealizedPattern(client, 'class_2013')
        self._2014: ActivityOutputsRealizedSupplyUnrealizedPattern = ActivityOutputsRealizedSupplyUnrealizedPattern(client, 'class_2014')
        self._2015: ActivityOutputsRealizedSupplyUnrealizedPattern = ActivityOutputsRealizedSupplyUnrealizedPattern(client, 'class_2015')
        self._2016: ActivityOutputsRealizedSupplyUnrealizedPattern = ActivityOutputsRealizedSupplyUnrealizedPattern(client, 'class_2016')
        self._2017: ActivityOutputsRealizedSupplyUnrealizedPattern = ActivityOutputsRealizedSupplyUnrealizedPattern(client, 'class_2017')
        self._2018: ActivityOutputsRealizedSupplyUnrealizedPattern = ActivityOutputsRealizedSupplyUnrealizedPattern(client, 'class_2018')
        self._2019: ActivityOutputsRealizedSupplyUnrealizedPattern = ActivityOutputsRealizedSupplyUnrealizedPattern(client, 'class_2019')
        self._2020: ActivityOutputsRealizedSupplyUnrealizedPattern = ActivityOutputsRealizedSupplyUnrealizedPattern(client, 'class_2020')
        self._2021: ActivityOutputsRealizedSupplyUnrealizedPattern = ActivityOutputsRealizedSupplyUnrealizedPattern(client, 'class_2021')
        self._2022: ActivityOutputsRealizedSupplyUnrealizedPattern = ActivityOutputsRealizedSupplyUnrealizedPattern(client, 'class_2022')
        self._2023: ActivityOutputsRealizedSupplyUnrealizedPattern = ActivityOutputsRealizedSupplyUnrealizedPattern(client, 'class_2023')
        self._2024: ActivityOutputsRealizedSupplyUnrealizedPattern = ActivityOutputsRealizedSupplyUnrealizedPattern(client, 'class_2024')
        self._2025: ActivityOutputsRealizedSupplyUnrealizedPattern = ActivityOutputsRealizedSupplyUnrealizedPattern(client, 'class_2025')
        self._2026: ActivityOutputsRealizedSupplyUnrealizedPattern = ActivityOutputsRealizedSupplyUnrealizedPattern(client, 'class_2026')

class SeriesTree_Cohorts_Utxo_Entry_Discount_Realized_Price_StdDev_All:
    """Series tree node."""
    
    def __init__(self, client: BrkClient, base_path: str = ''):
        self.sd: SeriesPattern1[StoredF32] = SeriesPattern1(client, 'veteran_realized_price_ratio_sd')
        self.zscore: SeriesPattern1[StoredF32] = SeriesPattern1(client, 'veteran_realized_price_ratio_zscore')
        self._0sd: CentsSatsUsdPattern = CentsSatsUsdPattern(client, 'veteran_realized_price_0sd')
        self.p0_5sd: PriceRatioPattern = PriceRatioPattern(client, 'veteran_realized_price', 'p0_5sd')
        self.p1sd: PriceRatioPattern = PriceRatioPattern(client, 'veteran_realized_price', 'p1sd')
        self.p1_5sd: PriceRatioPattern = PriceRatioPattern(client, 'veteran_realized_price', 'p1_5sd')
        self.p2sd: PriceRatioPattern = PriceRatioPattern(client, 'veteran_realized_price', 'p2sd')
        self.p2_5sd: PriceRatioPattern = PriceRatioPattern(client, 'veteran_realized_price', 'p2_5sd')
        self.p3sd: PriceRatioPattern = PriceRatioPattern(client, 'veteran_realized_price', 'p3sd')
        self.m0_5sd: PriceRatioPattern = PriceRatioPattern(client, 'veteran_realized_price', 'm0_5sd')
        self.m1sd: PriceRatioPattern = PriceRatioPattern(client, 'veteran_realized_price', 'm1sd')
        self.m1_5sd: PriceRatioPattern = PriceRatioPattern(client, 'veteran_realized_price', 'm1_5sd')
        self.m2sd: PriceRatioPattern = PriceRatioPattern(client, 'veteran_realized_price', 'm2sd')
        self.m2_5sd: PriceRatioPattern = PriceRatioPattern(client, 'veteran_realized_price', 'm2_5sd')
        self.m3sd: PriceRatioPattern = PriceRatioPattern(client, 'veteran_realized_price', 'm3sd')

class SeriesTree_Cohorts_Utxo_Entry_Discount_Realized_Price_StdDev_4y:
    """Series tree node."""
    
    def __init__(self, client: BrkClient, base_path: str = ''):
        self.sd: SeriesPattern1[StoredF32] = SeriesPattern1(client, 'veteran_realized_price_ratio_sd_4y')
        self.zscore: SeriesPattern1[StoredF32] = SeriesPattern1(client, 'veteran_realized_price_ratio_zscore_4y')
        self._0sd: CentsSatsUsdPattern = CentsSatsUsdPattern(client, 'veteran_realized_price_0sd_4y')
        self.p0_5sd: PriceRatioPattern = PriceRatioPattern(client, 'veteran_realized_price', 'p0_5sd_4y')
        self.p1sd: PriceRatioPattern = PriceRatioPattern(client, 'veteran_realized_price', 'p1sd_4y')
        self.p1_5sd: PriceRatioPattern = PriceRatioPattern(client, 'veteran_realized_price', 'p1_5sd_4y')
        self.p2sd: PriceRatioPattern = PriceRatioPattern(client, 'veteran_realized_price', 'p2sd_4y')
        self.p2_5sd: PriceRatioPattern = PriceRatioPattern(client, 'veteran_realized_price', 'p2_5sd_4y')
        self.p3sd: PriceRatioPattern = PriceRatioPattern(client, 'veteran_realized_price', 'p3sd_4y')
        self.m0_5sd: PriceRatioPattern = PriceRatioPattern(client, 'veteran_realized_price', 'm0_5sd_4y')
        self.m1sd: PriceRatioPattern = PriceRatioPattern(client, 'veteran_realized_price', 'm1sd_4y')
        self.m1_5sd: PriceRatioPattern = PriceRatioPattern(client, 'veteran_realized_price', 'm1_5sd_4y')
        self.m2sd: PriceRatioPattern = PriceRatioPattern(client, 'veteran_realized_price', 'm2sd_4y')
        self.m2_5sd: PriceRatioPattern = PriceRatioPattern(client, 'veteran_realized_price', 'm2_5sd_4y')
        self.m3sd: PriceRatioPattern = PriceRatioPattern(client, 'veteran_realized_price', 'm3sd_4y')

class SeriesTree_Cohorts_Utxo_Entry_Discount_Realized_Price_StdDev_2y:
    """Series tree node."""
    
    def __init__(self, client: BrkClient, base_path: str = ''):
        self.sd: SeriesPattern1[StoredF32] = SeriesPattern1(client, 'veteran_realized_price_ratio_sd_2y')
        self.zscore: SeriesPattern1[StoredF32] = SeriesPattern1(client, 'veteran_realized_price_ratio_zscore_2y')
        self._0sd: CentsSatsUsdPattern = CentsSatsUsdPattern(client, 'veteran_realized_price_0sd_2y')
        self.p0_5sd: PriceRatioPattern = PriceRatioPattern(client, 'veteran_realized_price', 'p0_5sd_2y')
        self.p1sd: PriceRatioPattern = PriceRatioPattern(client, 'veteran_realized_price', 'p1sd_2y')
        self.p1_5sd: PriceRatioPattern = PriceRatioPattern(client, 'veteran_realized_price', 'p1_5sd_2y')
        self.p2sd: PriceRatioPattern = PriceRatioPattern(client, 'veteran_realized_price', 'p2sd_2y')
        self.p2_5sd: PriceRatioPattern = PriceRatioPattern(client, 'veteran_realized_price', 'p2_5sd_2y')
        self.p3sd: PriceRatioPattern = PriceRatioPattern(client, 'veteran_realized_price', 'p3sd_2y')
        self.m0_5sd: PriceRatioPattern = PriceRatioPattern(client, 'veteran_realized_price', 'm0_5sd_2y')
        self.m1sd: PriceRatioPattern = PriceRatioPattern(client, 'veteran_realized_price', 'm1sd_2y')
        self.m1_5sd: PriceRatioPattern = PriceRatioPattern(client, 'veteran_realized_price', 'm1_5sd_2y')
        self.m2sd: PriceRatioPattern = PriceRatioPattern(client, 'veteran_realized_price', 'm2sd_2y')
        self.m2_5sd: PriceRatioPattern = PriceRatioPattern(client, 'veteran_realized_price', 'm2_5sd_2y')
        self.m3sd: PriceRatioPattern = PriceRatioPattern(client, 'veteran_realized_price', 'm3sd_2y')

class SeriesTree_Cohorts_Utxo_Entry_Discount_Realized_Price_StdDev_1y:
    """Series tree node."""
    
    def __init__(self, client: BrkClient, base_path: str = ''):
        self.sd: SeriesPattern1[StoredF32] = SeriesPattern1(client, 'veteran_realized_price_ratio_sd_1y')
        self.zscore: SeriesPattern1[StoredF32] = SeriesPattern1(client, 'veteran_realized_price_ratio_zscore_1y')
        self._0sd: CentsSatsUsdPattern = CentsSatsUsdPattern(client, 'veteran_realized_price_0sd_1y')
        self.p0_5sd: PriceRatioPattern = PriceRatioPattern(client, 'veteran_realized_price', 'p0_5sd_1y')
        self.p1sd: PriceRatioPattern = PriceRatioPattern(client, 'veteran_realized_price', 'p1sd_1y')
        self.p1_5sd: PriceRatioPattern = PriceRatioPattern(client, 'veteran_realized_price', 'p1_5sd_1y')
        self.p2sd: PriceRatioPattern = PriceRatioPattern(client, 'veteran_realized_price', 'p2sd_1y')
        self.p2_5sd: PriceRatioPattern = PriceRatioPattern(client, 'veteran_realized_price', 'p2_5sd_1y')
        self.p3sd: PriceRatioPattern = PriceRatioPattern(client, 'veteran_realized_price', 'p3sd_1y')
        self.m0_5sd: PriceRatioPattern = PriceRatioPattern(client, 'veteran_realized_price', 'm0_5sd_1y')
        self.m1sd: PriceRatioPattern = PriceRatioPattern(client, 'veteran_realized_price', 'm1sd_1y')
        self.m1_5sd: PriceRatioPattern = PriceRatioPattern(client, 'veteran_realized_price', 'm1_5sd_1y')
        self.m2sd: PriceRatioPattern = PriceRatioPattern(client, 'veteran_realized_price', 'm2sd_1y')
        self.m2_5sd: PriceRatioPattern = PriceRatioPattern(client, 'veteran_realized_price', 'm2_5sd_1y')
        self.m3sd: PriceRatioPattern = PriceRatioPattern(client, 'veteran_realized_price', 'm3sd_1y')

class SeriesTree_Cohorts_Utxo_Entry_Discount_Realized_Price_StdDev:
    """Series tree node."""
    
    def __init__(self, client: BrkClient, base_path: str = ''):
        self.all: SeriesTree_Cohorts_Utxo_Entry_Discount_Realized_Price_StdDev_All = SeriesTree_Cohorts_Utxo_Entry_Discount_Realized_Price_StdDev_All(client)
        self._4y: SeriesTree_Cohorts_Utxo_Entry_Discount_Realized_Price_StdDev_4y = SeriesTree_Cohorts_Utxo_Entry_Discount_Realized_Price_StdDev_4y(client)
        self._2y: SeriesTree_Cohorts_Utxo_Entry_Discount_Realized_Price_StdDev_2y = SeriesTree_Cohorts_Utxo_Entry_Discount_Realized_Price_StdDev_2y(client)
        self._1y: SeriesTree_Cohorts_Utxo_Entry_Discount_Realized_Price_StdDev_1y = SeriesTree_Cohorts_Utxo_Entry_Discount_Realized_Price_StdDev_1y(client)

class SeriesTree_Cohorts_Utxo_Entry_Discount_Realized_Price:
    """Series tree node."""
    
    def __init__(self, client: BrkClient, base_path: str = ''):
        self.usd: SeriesPattern1[Dollars] = SeriesPattern1(client, 'veteran_realized_price')
        self.cents: SeriesPattern1[Cents] = SeriesPattern1(client, 'veteran_realized_price_cents')
        self.sats: SeriesPattern1[SatsFract] = SeriesPattern1(client, 'veteran_realized_price_sats')
        self.bps: SeriesPattern1[BasisPoints32] = SeriesPattern1(client, 'veteran_realized_price_ratio_bps')
        self.ratio: SeriesPattern1[StoredF32] = SeriesPattern1(client, 'veteran_realized_price_ratio')
        self.percentiles: Pct0Pct1Pct2Pct5Pct95Pct98Pct99Pattern = Pct0Pct1Pct2Pct5Pct95Pct98Pct99Pattern(client, 'veteran_realized_price')
        self.sma: _1m1w1y2y4yAllPattern = _1m1w1y2y4yAllPattern(client, 'veteran_realized_price_ratio_sma')
        self.std_dev: SeriesTree_Cohorts_Utxo_Entry_Discount_Realized_Price_StdDev = SeriesTree_Cohorts_Utxo_Entry_Discount_Realized_Price_StdDev(client)

class SeriesTree_Cohorts_Utxo_Entry_Discount_Realized:
    """Series tree node."""
    
    def __init__(self, client: BrkClient, base_path: str = ''):
        self.cap: CentsDeltaToUsdPattern = CentsDeltaToUsdPattern(client, 'veteran_realized_cap')
        self.profit: BlockCumulativeSumPattern = BlockCumulativeSumPattern(client, 'veteran_realized_profit')
        self.loss: BlockCumulativeNegativeSumPattern = BlockCumulativeNegativeSumPattern(client, 'veteran_realized_loss')
        self.price: SeriesTree_Cohorts_Utxo_Entry_Discount_Realized_Price = SeriesTree_Cohorts_Utxo_Entry_Discount_Realized_Price(client)
        self.mvrv: SeriesPattern1[StoredF32] = SeriesPattern1(client, 'veteran_mvrv')
        self.net_pnl: BlockChangeCumulativeDeltaSumPattern = BlockChangeCumulativeDeltaSumPattern(client, 'veteran_net')
        self.sopr: RatioValuePattern2 = RatioValuePattern2(client, 'veteran')
        self.gross_pnl: BlockCumulativeSumPattern = BlockCumulativeSumPattern(client, 'veteran_realized_gross_pnl')
        self.sell_side_risk_ratio: _1m1w1y24hPattern8 = _1m1w1y24hPattern8(client, 'veteran_sell_side_risk_ratio')
        self.peak_regret: BlockCumulativeSumPattern = BlockCumulativeSumPattern(client, 'veteran_realized_peak_regret')
        self.capitalized: PricePattern = PricePattern(client, 'veteran_capitalized_price')
        self.profit_to_loss_ratio: _1m1w1y24hPattern[StoredF64] = _1m1w1y24hPattern(client, 'veteran_realized_profit_to_loss_ratio')

class SeriesTree_Cohorts_Utxo_Entry_Discount:
    """Series tree node."""
    
    def __init__(self, client: BrkClient, base_path: str = ''):
        self.supply: DeltaDominanceHalfInTotalPattern2 = DeltaDominanceHalfInTotalPattern2(client, 'veteran_supply')
        self.outputs: SpendingSpentUnspentPattern = SpendingSpentUnspentPattern(client, 'veteran')
        self.activity: CoindaysCoinyearsDormancyTransferPattern = CoindaysCoinyearsDormancyTransferPattern(client, 'veteran')
        self.realized: SeriesTree_Cohorts_Utxo_Entry_Discount_Realized = SeriesTree_Cohorts_Utxo_Entry_Discount_Realized(client)
        self.cost_basis: InMaxMinPerSupplyPattern = InMaxMinPerSupplyPattern(client, 'veteran')
        self.unrealized: CapitalizedGrossInvestedLossNetNuplProfitSentimentPattern2 = CapitalizedGrossInvestedLossNetNuplProfitSentimentPattern2(client, 'veteran')
        self.invested_capital: InPattern = InPattern(client, 'veteran_invested_capital_in')

class SeriesTree_Cohorts_Utxo_Entry_Premium_Realized_Price_StdDev_All:
    """Series tree node."""
    
    def __init__(self, client: BrkClient, base_path: str = ''):
        self.sd: SeriesPattern1[StoredF32] = SeriesPattern1(client, 'rookie_realized_price_ratio_sd')
        self.zscore: SeriesPattern1[StoredF32] = SeriesPattern1(client, 'rookie_realized_price_ratio_zscore')
        self._0sd: CentsSatsUsdPattern = CentsSatsUsdPattern(client, 'rookie_realized_price_0sd')
        self.p0_5sd: PriceRatioPattern = PriceRatioPattern(client, 'rookie_realized_price', 'p0_5sd')
        self.p1sd: PriceRatioPattern = PriceRatioPattern(client, 'rookie_realized_price', 'p1sd')
        self.p1_5sd: PriceRatioPattern = PriceRatioPattern(client, 'rookie_realized_price', 'p1_5sd')
        self.p2sd: PriceRatioPattern = PriceRatioPattern(client, 'rookie_realized_price', 'p2sd')
        self.p2_5sd: PriceRatioPattern = PriceRatioPattern(client, 'rookie_realized_price', 'p2_5sd')
        self.p3sd: PriceRatioPattern = PriceRatioPattern(client, 'rookie_realized_price', 'p3sd')
        self.m0_5sd: PriceRatioPattern = PriceRatioPattern(client, 'rookie_realized_price', 'm0_5sd')
        self.m1sd: PriceRatioPattern = PriceRatioPattern(client, 'rookie_realized_price', 'm1sd')
        self.m1_5sd: PriceRatioPattern = PriceRatioPattern(client, 'rookie_realized_price', 'm1_5sd')
        self.m2sd: PriceRatioPattern = PriceRatioPattern(client, 'rookie_realized_price', 'm2sd')
        self.m2_5sd: PriceRatioPattern = PriceRatioPattern(client, 'rookie_realized_price', 'm2_5sd')
        self.m3sd: PriceRatioPattern = PriceRatioPattern(client, 'rookie_realized_price', 'm3sd')

class SeriesTree_Cohorts_Utxo_Entry_Premium_Realized_Price_StdDev_4y:
    """Series tree node."""
    
    def __init__(self, client: BrkClient, base_path: str = ''):
        self.sd: SeriesPattern1[StoredF32] = SeriesPattern1(client, 'rookie_realized_price_ratio_sd_4y')
        self.zscore: SeriesPattern1[StoredF32] = SeriesPattern1(client, 'rookie_realized_price_ratio_zscore_4y')
        self._0sd: CentsSatsUsdPattern = CentsSatsUsdPattern(client, 'rookie_realized_price_0sd_4y')
        self.p0_5sd: PriceRatioPattern = PriceRatioPattern(client, 'rookie_realized_price', 'p0_5sd_4y')
        self.p1sd: PriceRatioPattern = PriceRatioPattern(client, 'rookie_realized_price', 'p1sd_4y')
        self.p1_5sd: PriceRatioPattern = PriceRatioPattern(client, 'rookie_realized_price', 'p1_5sd_4y')
        self.p2sd: PriceRatioPattern = PriceRatioPattern(client, 'rookie_realized_price', 'p2sd_4y')
        self.p2_5sd: PriceRatioPattern = PriceRatioPattern(client, 'rookie_realized_price', 'p2_5sd_4y')
        self.p3sd: PriceRatioPattern = PriceRatioPattern(client, 'rookie_realized_price', 'p3sd_4y')
        self.m0_5sd: PriceRatioPattern = PriceRatioPattern(client, 'rookie_realized_price', 'm0_5sd_4y')
        self.m1sd: PriceRatioPattern = PriceRatioPattern(client, 'rookie_realized_price', 'm1sd_4y')
        self.m1_5sd: PriceRatioPattern = PriceRatioPattern(client, 'rookie_realized_price', 'm1_5sd_4y')
        self.m2sd: PriceRatioPattern = PriceRatioPattern(client, 'rookie_realized_price', 'm2sd_4y')
        self.m2_5sd: PriceRatioPattern = PriceRatioPattern(client, 'rookie_realized_price', 'm2_5sd_4y')
        self.m3sd: PriceRatioPattern = PriceRatioPattern(client, 'rookie_realized_price', 'm3sd_4y')

class SeriesTree_Cohorts_Utxo_Entry_Premium_Realized_Price_StdDev_2y:
    """Series tree node."""
    
    def __init__(self, client: BrkClient, base_path: str = ''):
        self.sd: SeriesPattern1[StoredF32] = SeriesPattern1(client, 'rookie_realized_price_ratio_sd_2y')
        self.zscore: SeriesPattern1[StoredF32] = SeriesPattern1(client, 'rookie_realized_price_ratio_zscore_2y')
        self._0sd: CentsSatsUsdPattern = CentsSatsUsdPattern(client, 'rookie_realized_price_0sd_2y')
        self.p0_5sd: PriceRatioPattern = PriceRatioPattern(client, 'rookie_realized_price', 'p0_5sd_2y')
        self.p1sd: PriceRatioPattern = PriceRatioPattern(client, 'rookie_realized_price', 'p1sd_2y')
        self.p1_5sd: PriceRatioPattern = PriceRatioPattern(client, 'rookie_realized_price', 'p1_5sd_2y')
        self.p2sd: PriceRatioPattern = PriceRatioPattern(client, 'rookie_realized_price', 'p2sd_2y')
        self.p2_5sd: PriceRatioPattern = PriceRatioPattern(client, 'rookie_realized_price', 'p2_5sd_2y')
        self.p3sd: PriceRatioPattern = PriceRatioPattern(client, 'rookie_realized_price', 'p3sd_2y')
        self.m0_5sd: PriceRatioPattern = PriceRatioPattern(client, 'rookie_realized_price', 'm0_5sd_2y')
        self.m1sd: PriceRatioPattern = PriceRatioPattern(client, 'rookie_realized_price', 'm1sd_2y')
        self.m1_5sd: PriceRatioPattern = PriceRatioPattern(client, 'rookie_realized_price', 'm1_5sd_2y')
        self.m2sd: PriceRatioPattern = PriceRatioPattern(client, 'rookie_realized_price', 'm2sd_2y')
        self.m2_5sd: PriceRatioPattern = PriceRatioPattern(client, 'rookie_realized_price', 'm2_5sd_2y')
        self.m3sd: PriceRatioPattern = PriceRatioPattern(client, 'rookie_realized_price', 'm3sd_2y')

class SeriesTree_Cohorts_Utxo_Entry_Premium_Realized_Price_StdDev_1y:
    """Series tree node."""
    
    def __init__(self, client: BrkClient, base_path: str = ''):
        self.sd: SeriesPattern1[StoredF32] = SeriesPattern1(client, 'rookie_realized_price_ratio_sd_1y')
        self.zscore: SeriesPattern1[StoredF32] = SeriesPattern1(client, 'rookie_realized_price_ratio_zscore_1y')
        self._0sd: CentsSatsUsdPattern = CentsSatsUsdPattern(client, 'rookie_realized_price_0sd_1y')
        self.p0_5sd: PriceRatioPattern = PriceRatioPattern(client, 'rookie_realized_price', 'p0_5sd_1y')
        self.p1sd: PriceRatioPattern = PriceRatioPattern(client, 'rookie_realized_price', 'p1sd_1y')
        self.p1_5sd: PriceRatioPattern = PriceRatioPattern(client, 'rookie_realized_price', 'p1_5sd_1y')
        self.p2sd: PriceRatioPattern = PriceRatioPattern(client, 'rookie_realized_price', 'p2sd_1y')
        self.p2_5sd: PriceRatioPattern = PriceRatioPattern(client, 'rookie_realized_price', 'p2_5sd_1y')
        self.p3sd: PriceRatioPattern = PriceRatioPattern(client, 'rookie_realized_price', 'p3sd_1y')
        self.m0_5sd: PriceRatioPattern = PriceRatioPattern(client, 'rookie_realized_price', 'm0_5sd_1y')
        self.m1sd: PriceRatioPattern = PriceRatioPattern(client, 'rookie_realized_price', 'm1sd_1y')
        self.m1_5sd: PriceRatioPattern = PriceRatioPattern(client, 'rookie_realized_price', 'm1_5sd_1y')
        self.m2sd: PriceRatioPattern = PriceRatioPattern(client, 'rookie_realized_price', 'm2sd_1y')
        self.m2_5sd: PriceRatioPattern = PriceRatioPattern(client, 'rookie_realized_price', 'm2_5sd_1y')
        self.m3sd: PriceRatioPattern = PriceRatioPattern(client, 'rookie_realized_price', 'm3sd_1y')

class SeriesTree_Cohorts_Utxo_Entry_Premium_Realized_Price_StdDev:
    """Series tree node."""
    
    def __init__(self, client: BrkClient, base_path: str = ''):
        self.all: SeriesTree_Cohorts_Utxo_Entry_Premium_Realized_Price_StdDev_All = SeriesTree_Cohorts_Utxo_Entry_Premium_Realized_Price_StdDev_All(client)
        self._4y: SeriesTree_Cohorts_Utxo_Entry_Premium_Realized_Price_StdDev_4y = SeriesTree_Cohorts_Utxo_Entry_Premium_Realized_Price_StdDev_4y(client)
        self._2y: SeriesTree_Cohorts_Utxo_Entry_Premium_Realized_Price_StdDev_2y = SeriesTree_Cohorts_Utxo_Entry_Premium_Realized_Price_StdDev_2y(client)
        self._1y: SeriesTree_Cohorts_Utxo_Entry_Premium_Realized_Price_StdDev_1y = SeriesTree_Cohorts_Utxo_Entry_Premium_Realized_Price_StdDev_1y(client)

class SeriesTree_Cohorts_Utxo_Entry_Premium_Realized_Price:
    """Series tree node."""
    
    def __init__(self, client: BrkClient, base_path: str = ''):
        self.usd: SeriesPattern1[Dollars] = SeriesPattern1(client, 'rookie_realized_price')
        self.cents: SeriesPattern1[Cents] = SeriesPattern1(client, 'rookie_realized_price_cents')
        self.sats: SeriesPattern1[SatsFract] = SeriesPattern1(client, 'rookie_realized_price_sats')
        self.bps: SeriesPattern1[BasisPoints32] = SeriesPattern1(client, 'rookie_realized_price_ratio_bps')
        self.ratio: SeriesPattern1[StoredF32] = SeriesPattern1(client, 'rookie_realized_price_ratio')
        self.percentiles: Pct0Pct1Pct2Pct5Pct95Pct98Pct99Pattern = Pct0Pct1Pct2Pct5Pct95Pct98Pct99Pattern(client, 'rookie_realized_price')
        self.sma: _1m1w1y2y4yAllPattern = _1m1w1y2y4yAllPattern(client, 'rookie_realized_price_ratio_sma')
        self.std_dev: SeriesTree_Cohorts_Utxo_Entry_Premium_Realized_Price_StdDev = SeriesTree_Cohorts_Utxo_Entry_Premium_Realized_Price_StdDev(client)

class SeriesTree_Cohorts_Utxo_Entry_Premium_Realized:
    """Series tree node."""
    
    def __init__(self, client: BrkClient, base_path: str = ''):
        self.cap: CentsDeltaToUsdPattern = CentsDeltaToUsdPattern(client, 'rookie_realized_cap')
        self.profit: BlockCumulativeSumPattern = BlockCumulativeSumPattern(client, 'rookie_realized_profit')
        self.loss: BlockCumulativeNegativeSumPattern = BlockCumulativeNegativeSumPattern(client, 'rookie_realized_loss')
        self.price: SeriesTree_Cohorts_Utxo_Entry_Premium_Realized_Price = SeriesTree_Cohorts_Utxo_Entry_Premium_Realized_Price(client)
        self.mvrv: SeriesPattern1[StoredF32] = SeriesPattern1(client, 'rookie_mvrv')
        self.net_pnl: BlockChangeCumulativeDeltaSumPattern = BlockChangeCumulativeDeltaSumPattern(client, 'rookie_net')
        self.sopr: RatioValuePattern2 = RatioValuePattern2(client, 'rookie')
        self.gross_pnl: BlockCumulativeSumPattern = BlockCumulativeSumPattern(client, 'rookie_realized_gross_pnl')
        self.sell_side_risk_ratio: _1m1w1y24hPattern8 = _1m1w1y24hPattern8(client, 'rookie_sell_side_risk_ratio')
        self.peak_regret: BlockCumulativeSumPattern = BlockCumulativeSumPattern(client, 'rookie_realized_peak_regret')
        self.capitalized: PricePattern = PricePattern(client, 'rookie_capitalized_price')
        self.profit_to_loss_ratio: _1m1w1y24hPattern[StoredF64] = _1m1w1y24hPattern(client, 'rookie_realized_profit_to_loss_ratio')

class SeriesTree_Cohorts_Utxo_Entry_Premium:
    """Series tree node."""
    
    def __init__(self, client: BrkClient, base_path: str = ''):
        self.supply: DeltaDominanceHalfInTotalPattern2 = DeltaDominanceHalfInTotalPattern2(client, 'rookie_supply')
        self.outputs: SpendingSpentUnspentPattern = SpendingSpentUnspentPattern(client, 'rookie')
        self.activity: CoindaysCoinyearsDormancyTransferPattern = CoindaysCoinyearsDormancyTransferPattern(client, 'rookie')
        self.realized: SeriesTree_Cohorts_Utxo_Entry_Premium_Realized = SeriesTree_Cohorts_Utxo_Entry_Premium_Realized(client)
        self.cost_basis: InMaxMinPerSupplyPattern = InMaxMinPerSupplyPattern(client, 'rookie')
        self.unrealized: CapitalizedGrossInvestedLossNetNuplProfitSentimentPattern2 = CapitalizedGrossInvestedLossNetNuplProfitSentimentPattern2(client, 'rookie')
        self.invested_capital: InPattern = InPattern(client, 'rookie_invested_capital_in')

class SeriesTree_Cohorts_Utxo_Entry:
    """Series tree node."""
    
    def __init__(self, client: BrkClient, base_path: str = ''):
        self.discount: SeriesTree_Cohorts_Utxo_Entry_Discount = SeriesTree_Cohorts_Utxo_Entry_Discount(client)
        self.premium: SeriesTree_Cohorts_Utxo_Entry_Premium = SeriesTree_Cohorts_Utxo_Entry_Premium(client)

class SeriesTree_Cohorts_Utxo_OverAmount:
    """Series tree node."""
    
    def __init__(self, client: BrkClient, base_path: str = ''):
        self._1sat: ActivityOutputsRealizedSupplyUnrealizedPattern2 = ActivityOutputsRealizedSupplyUnrealizedPattern2(client, 'utxos_over_1sat')
        self._10sats: ActivityOutputsRealizedSupplyUnrealizedPattern2 = ActivityOutputsRealizedSupplyUnrealizedPattern2(client, 'utxos_over_10sats')
        self._100sats: ActivityOutputsRealizedSupplyUnrealizedPattern2 = ActivityOutputsRealizedSupplyUnrealizedPattern2(client, 'utxos_over_100sats')
        self._1k_sats: ActivityOutputsRealizedSupplyUnrealizedPattern2 = ActivityOutputsRealizedSupplyUnrealizedPattern2(client, 'utxos_over_1k_sats')
        self._10k_sats: ActivityOutputsRealizedSupplyUnrealizedPattern2 = ActivityOutputsRealizedSupplyUnrealizedPattern2(client, 'utxos_over_10k_sats')
        self._100k_sats: ActivityOutputsRealizedSupplyUnrealizedPattern2 = ActivityOutputsRealizedSupplyUnrealizedPattern2(client, 'utxos_over_100k_sats')
        self._1m_sats: ActivityOutputsRealizedSupplyUnrealizedPattern2 = ActivityOutputsRealizedSupplyUnrealizedPattern2(client, 'utxos_over_1m_sats')
        self._10m_sats: ActivityOutputsRealizedSupplyUnrealizedPattern2 = ActivityOutputsRealizedSupplyUnrealizedPattern2(client, 'utxos_over_10m_sats')
        self._1btc: ActivityOutputsRealizedSupplyUnrealizedPattern2 = ActivityOutputsRealizedSupplyUnrealizedPattern2(client, 'utxos_over_1btc')
        self._10btc: ActivityOutputsRealizedSupplyUnrealizedPattern2 = ActivityOutputsRealizedSupplyUnrealizedPattern2(client, 'utxos_over_10btc')
        self._100btc: ActivityOutputsRealizedSupplyUnrealizedPattern2 = ActivityOutputsRealizedSupplyUnrealizedPattern2(client, 'utxos_over_100btc')
        self._1k_btc: ActivityOutputsRealizedSupplyUnrealizedPattern2 = ActivityOutputsRealizedSupplyUnrealizedPattern2(client, 'utxos_over_1k_btc')
        self._10k_btc: ActivityOutputsRealizedSupplyUnrealizedPattern2 = ActivityOutputsRealizedSupplyUnrealizedPattern2(client, 'utxos_over_10k_btc')

class SeriesTree_Cohorts_Utxo_AmountRange:
    """Series tree node."""
    
    def __init__(self, client: BrkClient, base_path: str = ''):
        self._0sats: ActivityOutputsRealizedSupplyUnrealizedPattern2 = ActivityOutputsRealizedSupplyUnrealizedPattern2(client, 'utxos_0sats')
        self._1sat_to_10sats: ActivityOutputsRealizedSupplyUnrealizedPattern2 = ActivityOutputsRealizedSupplyUnrealizedPattern2(client, 'utxos_1sat_to_10sats')
        self._10sats_to_100sats: ActivityOutputsRealizedSupplyUnrealizedPattern2 = ActivityOutputsRealizedSupplyUnrealizedPattern2(client, 'utxos_10sats_to_100sats')
        self._100sats_to_1k_sats: ActivityOutputsRealizedSupplyUnrealizedPattern2 = ActivityOutputsRealizedSupplyUnrealizedPattern2(client, 'utxos_100sats_to_1k_sats')
        self._1k_sats_to_10k_sats: ActivityOutputsRealizedSupplyUnrealizedPattern2 = ActivityOutputsRealizedSupplyUnrealizedPattern2(client, 'utxos_1k_sats_to_10k_sats')
        self._10k_sats_to_100k_sats: ActivityOutputsRealizedSupplyUnrealizedPattern2 = ActivityOutputsRealizedSupplyUnrealizedPattern2(client, 'utxos_10k_sats_to_100k_sats')
        self._100k_sats_to_1m_sats: ActivityOutputsRealizedSupplyUnrealizedPattern2 = ActivityOutputsRealizedSupplyUnrealizedPattern2(client, 'utxos_100k_sats_to_1m_sats')
        self._1m_sats_to_10m_sats: ActivityOutputsRealizedSupplyUnrealizedPattern2 = ActivityOutputsRealizedSupplyUnrealizedPattern2(client, 'utxos_1m_sats_to_10m_sats')
        self._10m_sats_to_1btc: ActivityOutputsRealizedSupplyUnrealizedPattern2 = ActivityOutputsRealizedSupplyUnrealizedPattern2(client, 'utxos_10m_sats_to_1btc')
        self._1btc_to_10btc: ActivityOutputsRealizedSupplyUnrealizedPattern2 = ActivityOutputsRealizedSupplyUnrealizedPattern2(client, 'utxos_1btc_to_10btc')
        self._10btc_to_100btc: ActivityOutputsRealizedSupplyUnrealizedPattern2 = ActivityOutputsRealizedSupplyUnrealizedPattern2(client, 'utxos_10btc_to_100btc')
        self._100btc_to_1k_btc: ActivityOutputsRealizedSupplyUnrealizedPattern2 = ActivityOutputsRealizedSupplyUnrealizedPattern2(client, 'utxos_100btc_to_1k_btc')
        self._1k_btc_to_10k_btc: ActivityOutputsRealizedSupplyUnrealizedPattern2 = ActivityOutputsRealizedSupplyUnrealizedPattern2(client, 'utxos_1k_btc_to_10k_btc')
        self._10k_btc_to_100k_btc: ActivityOutputsRealizedSupplyUnrealizedPattern2 = ActivityOutputsRealizedSupplyUnrealizedPattern2(client, 'utxos_10k_btc_to_100k_btc')
        self.over_100k_btc: ActivityOutputsRealizedSupplyUnrealizedPattern2 = ActivityOutputsRealizedSupplyUnrealizedPattern2(client, 'utxos_over_100k_btc')

class SeriesTree_Cohorts_Utxo_UnderAmount:
    """Series tree node."""
    
    def __init__(self, client: BrkClient, base_path: str = ''):
        self._10sats: ActivityOutputsRealizedSupplyUnrealizedPattern2 = ActivityOutputsRealizedSupplyUnrealizedPattern2(client, 'utxos_under_10sats')
        self._100sats: ActivityOutputsRealizedSupplyUnrealizedPattern2 = ActivityOutputsRealizedSupplyUnrealizedPattern2(client, 'utxos_under_100sats')
        self._1k_sats: ActivityOutputsRealizedSupplyUnrealizedPattern2 = ActivityOutputsRealizedSupplyUnrealizedPattern2(client, 'utxos_under_1k_sats')
        self._10k_sats: ActivityOutputsRealizedSupplyUnrealizedPattern2 = ActivityOutputsRealizedSupplyUnrealizedPattern2(client, 'utxos_under_10k_sats')
        self._100k_sats: ActivityOutputsRealizedSupplyUnrealizedPattern2 = ActivityOutputsRealizedSupplyUnrealizedPattern2(client, 'utxos_under_100k_sats')
        self._1m_sats: ActivityOutputsRealizedSupplyUnrealizedPattern2 = ActivityOutputsRealizedSupplyUnrealizedPattern2(client, 'utxos_under_1m_sats')
        self._10m_sats: ActivityOutputsRealizedSupplyUnrealizedPattern2 = ActivityOutputsRealizedSupplyUnrealizedPattern2(client, 'utxos_under_10m_sats')
        self._1btc: ActivityOutputsRealizedSupplyUnrealizedPattern2 = ActivityOutputsRealizedSupplyUnrealizedPattern2(client, 'utxos_under_1btc')
        self._10btc: ActivityOutputsRealizedSupplyUnrealizedPattern2 = ActivityOutputsRealizedSupplyUnrealizedPattern2(client, 'utxos_under_10btc')
        self._100btc: ActivityOutputsRealizedSupplyUnrealizedPattern2 = ActivityOutputsRealizedSupplyUnrealizedPattern2(client, 'utxos_under_100btc')
        self._1k_btc: ActivityOutputsRealizedSupplyUnrealizedPattern2 = ActivityOutputsRealizedSupplyUnrealizedPattern2(client, 'utxos_under_1k_btc')
        self._10k_btc: ActivityOutputsRealizedSupplyUnrealizedPattern2 = ActivityOutputsRealizedSupplyUnrealizedPattern2(client, 'utxos_under_10k_btc')
        self._100k_btc: ActivityOutputsRealizedSupplyUnrealizedPattern2 = ActivityOutputsRealizedSupplyUnrealizedPattern2(client, 'utxos_under_100k_btc')

class SeriesTree_Cohorts_Utxo_Type:
    """Series tree node."""
    
    def __init__(self, client: BrkClient, base_path: str = ''):
        self.p2pk65: ActivityOutputsRealizedSupplyUnrealizedPattern3 = ActivityOutputsRealizedSupplyUnrealizedPattern3(client, 'p2pk65')
        self.p2pk33: ActivityOutputsRealizedSupplyUnrealizedPattern3 = ActivityOutputsRealizedSupplyUnrealizedPattern3(client, 'p2pk33')
        self.p2pkh: ActivityOutputsRealizedSupplyUnrealizedPattern3 = ActivityOutputsRealizedSupplyUnrealizedPattern3(client, 'p2pkh')
        self.p2ms: ActivityOutputsRealizedSupplyUnrealizedPattern3 = ActivityOutputsRealizedSupplyUnrealizedPattern3(client, 'p2ms')
        self.p2sh: ActivityOutputsRealizedSupplyUnrealizedPattern3 = ActivityOutputsRealizedSupplyUnrealizedPattern3(client, 'p2sh')
        self.p2wpkh: ActivityOutputsRealizedSupplyUnrealizedPattern3 = ActivityOutputsRealizedSupplyUnrealizedPattern3(client, 'p2wpkh')
        self.p2wsh: ActivityOutputsRealizedSupplyUnrealizedPattern3 = ActivityOutputsRealizedSupplyUnrealizedPattern3(client, 'p2wsh')
        self.p2tr: ActivityOutputsRealizedSupplyUnrealizedPattern3 = ActivityOutputsRealizedSupplyUnrealizedPattern3(client, 'p2tr')
        self.p2a: ActivityOutputsRealizedSupplyUnrealizedPattern3 = ActivityOutputsRealizedSupplyUnrealizedPattern3(client, 'p2a')
        self.unknown: ActivityOutputsRealizedSupplyUnrealizedPattern3 = ActivityOutputsRealizedSupplyUnrealizedPattern3(client, 'unknown_outputs')
        self.empty: ActivityOutputsRealizedSupplyUnrealizedPattern3 = ActivityOutputsRealizedSupplyUnrealizedPattern3(client, 'empty_outputs')

class SeriesTree_Cohorts_Utxo_Profitability_Range:
    """Series tree node."""
    
    def __init__(self, client: BrkClient, base_path: str = ''):
        self.over_1000pct_in_profit: NuplRealizedSupplyUnrealizedPattern = NuplRealizedSupplyUnrealizedPattern(client, 'utxos_over_1000pct_in_profit')
        self._500pct_to_1000pct_in_profit: NuplRealizedSupplyUnrealizedPattern = NuplRealizedSupplyUnrealizedPattern(client, 'utxos_500pct_to_1000pct_in_profit')
        self._300pct_to_500pct_in_profit: NuplRealizedSupplyUnrealizedPattern = NuplRealizedSupplyUnrealizedPattern(client, 'utxos_300pct_to_500pct_in_profit')
        self._200pct_to_300pct_in_profit: NuplRealizedSupplyUnrealizedPattern = NuplRealizedSupplyUnrealizedPattern(client, 'utxos_200pct_to_300pct_in_profit')
        self._100pct_to_200pct_in_profit: NuplRealizedSupplyUnrealizedPattern = NuplRealizedSupplyUnrealizedPattern(client, 'utxos_100pct_to_200pct_in_profit')
        self._90pct_to_100pct_in_profit: NuplRealizedSupplyUnrealizedPattern = NuplRealizedSupplyUnrealizedPattern(client, 'utxos_90pct_to_100pct_in_profit')
        self._80pct_to_90pct_in_profit: NuplRealizedSupplyUnrealizedPattern = NuplRealizedSupplyUnrealizedPattern(client, 'utxos_80pct_to_90pct_in_profit')
        self._70pct_to_80pct_in_profit: NuplRealizedSupplyUnrealizedPattern = NuplRealizedSupplyUnrealizedPattern(client, 'utxos_70pct_to_80pct_in_profit')
        self._60pct_to_70pct_in_profit: NuplRealizedSupplyUnrealizedPattern = NuplRealizedSupplyUnrealizedPattern(client, 'utxos_60pct_to_70pct_in_profit')
        self._50pct_to_60pct_in_profit: NuplRealizedSupplyUnrealizedPattern = NuplRealizedSupplyUnrealizedPattern(client, 'utxos_50pct_to_60pct_in_profit')
        self._40pct_to_50pct_in_profit: NuplRealizedSupplyUnrealizedPattern = NuplRealizedSupplyUnrealizedPattern(client, 'utxos_40pct_to_50pct_in_profit')
        self._30pct_to_40pct_in_profit: NuplRealizedSupplyUnrealizedPattern = NuplRealizedSupplyUnrealizedPattern(client, 'utxos_30pct_to_40pct_in_profit')
        self._20pct_to_30pct_in_profit: NuplRealizedSupplyUnrealizedPattern = NuplRealizedSupplyUnrealizedPattern(client, 'utxos_20pct_to_30pct_in_profit')
        self._10pct_to_20pct_in_profit: NuplRealizedSupplyUnrealizedPattern = NuplRealizedSupplyUnrealizedPattern(client, 'utxos_10pct_to_20pct_in_profit')
        self._0pct_to_10pct_in_profit: NuplRealizedSupplyUnrealizedPattern = NuplRealizedSupplyUnrealizedPattern(client, 'utxos_0pct_to_10pct_in_profit')
        self._0pct_to_10pct_in_loss: NuplRealizedSupplyUnrealizedPattern = NuplRealizedSupplyUnrealizedPattern(client, 'utxos_0pct_to_10pct_in_loss')
        self._10pct_to_20pct_in_loss: NuplRealizedSupplyUnrealizedPattern = NuplRealizedSupplyUnrealizedPattern(client, 'utxos_10pct_to_20pct_in_loss')
        self._20pct_to_30pct_in_loss: NuplRealizedSupplyUnrealizedPattern = NuplRealizedSupplyUnrealizedPattern(client, 'utxos_20pct_to_30pct_in_loss')
        self._30pct_to_40pct_in_loss: NuplRealizedSupplyUnrealizedPattern = NuplRealizedSupplyUnrealizedPattern(client, 'utxos_30pct_to_40pct_in_loss')
        self._40pct_to_50pct_in_loss: NuplRealizedSupplyUnrealizedPattern = NuplRealizedSupplyUnrealizedPattern(client, 'utxos_40pct_to_50pct_in_loss')
        self._50pct_to_60pct_in_loss: NuplRealizedSupplyUnrealizedPattern = NuplRealizedSupplyUnrealizedPattern(client, 'utxos_50pct_to_60pct_in_loss')
        self._60pct_to_70pct_in_loss: NuplRealizedSupplyUnrealizedPattern = NuplRealizedSupplyUnrealizedPattern(client, 'utxos_60pct_to_70pct_in_loss')
        self._70pct_to_80pct_in_loss: NuplRealizedSupplyUnrealizedPattern = NuplRealizedSupplyUnrealizedPattern(client, 'utxos_70pct_to_80pct_in_loss')
        self._80pct_to_90pct_in_loss: NuplRealizedSupplyUnrealizedPattern = NuplRealizedSupplyUnrealizedPattern(client, 'utxos_80pct_to_90pct_in_loss')
        self._90pct_to_100pct_in_loss: NuplRealizedSupplyUnrealizedPattern = NuplRealizedSupplyUnrealizedPattern(client, 'utxos_90pct_to_100pct_in_loss')

class SeriesTree_Cohorts_Utxo_Profitability_Profit:
    """Series tree node."""
    
    def __init__(self, client: BrkClient, base_path: str = ''):
        self.all: NuplRealizedSupplyUnrealizedPattern = NuplRealizedSupplyUnrealizedPattern(client, 'utxos_in_profit')
        self._10pct: NuplRealizedSupplyUnrealizedPattern = NuplRealizedSupplyUnrealizedPattern(client, 'utxos_over_10pct_in_profit')
        self._20pct: NuplRealizedSupplyUnrealizedPattern = NuplRealizedSupplyUnrealizedPattern(client, 'utxos_over_20pct_in_profit')
        self._30pct: NuplRealizedSupplyUnrealizedPattern = NuplRealizedSupplyUnrealizedPattern(client, 'utxos_over_30pct_in_profit')
        self._40pct: NuplRealizedSupplyUnrealizedPattern = NuplRealizedSupplyUnrealizedPattern(client, 'utxos_over_40pct_in_profit')
        self._50pct: NuplRealizedSupplyUnrealizedPattern = NuplRealizedSupplyUnrealizedPattern(client, 'utxos_over_50pct_in_profit')
        self._60pct: NuplRealizedSupplyUnrealizedPattern = NuplRealizedSupplyUnrealizedPattern(client, 'utxos_over_60pct_in_profit')
        self._70pct: NuplRealizedSupplyUnrealizedPattern = NuplRealizedSupplyUnrealizedPattern(client, 'utxos_over_70pct_in_profit')
        self._80pct: NuplRealizedSupplyUnrealizedPattern = NuplRealizedSupplyUnrealizedPattern(client, 'utxos_over_80pct_in_profit')
        self._90pct: NuplRealizedSupplyUnrealizedPattern = NuplRealizedSupplyUnrealizedPattern(client, 'utxos_over_90pct_in_profit')
        self._100pct: NuplRealizedSupplyUnrealizedPattern = NuplRealizedSupplyUnrealizedPattern(client, 'utxos_over_100pct_in_profit')
        self._200pct: NuplRealizedSupplyUnrealizedPattern = NuplRealizedSupplyUnrealizedPattern(client, 'utxos_over_200pct_in_profit')
        self._300pct: NuplRealizedSupplyUnrealizedPattern = NuplRealizedSupplyUnrealizedPattern(client, 'utxos_over_300pct_in_profit')
        self._500pct: NuplRealizedSupplyUnrealizedPattern = NuplRealizedSupplyUnrealizedPattern(client, 'utxos_over_500pct_in_profit')

class SeriesTree_Cohorts_Utxo_Profitability_Loss:
    """Series tree node."""
    
    def __init__(self, client: BrkClient, base_path: str = ''):
        self.all: NuplRealizedSupplyUnrealizedPattern = NuplRealizedSupplyUnrealizedPattern(client, 'utxos_in_loss')
        self._10pct: NuplRealizedSupplyUnrealizedPattern = NuplRealizedSupplyUnrealizedPattern(client, 'utxos_over_10pct_in_loss')
        self._20pct: NuplRealizedSupplyUnrealizedPattern = NuplRealizedSupplyUnrealizedPattern(client, 'utxos_over_20pct_in_loss')
        self._30pct: NuplRealizedSupplyUnrealizedPattern = NuplRealizedSupplyUnrealizedPattern(client, 'utxos_over_30pct_in_loss')
        self._40pct: NuplRealizedSupplyUnrealizedPattern = NuplRealizedSupplyUnrealizedPattern(client, 'utxos_over_40pct_in_loss')
        self._50pct: NuplRealizedSupplyUnrealizedPattern = NuplRealizedSupplyUnrealizedPattern(client, 'utxos_over_50pct_in_loss')
        self._60pct: NuplRealizedSupplyUnrealizedPattern = NuplRealizedSupplyUnrealizedPattern(client, 'utxos_over_60pct_in_loss')
        self._70pct: NuplRealizedSupplyUnrealizedPattern = NuplRealizedSupplyUnrealizedPattern(client, 'utxos_over_70pct_in_loss')
        self._80pct: NuplRealizedSupplyUnrealizedPattern = NuplRealizedSupplyUnrealizedPattern(client, 'utxos_over_80pct_in_loss')

class SeriesTree_Cohorts_Utxo_Profitability:
    """Series tree node."""
    
    def __init__(self, client: BrkClient, base_path: str = ''):
        self.range: SeriesTree_Cohorts_Utxo_Profitability_Range = SeriesTree_Cohorts_Utxo_Profitability_Range(client)
        self.profit: SeriesTree_Cohorts_Utxo_Profitability_Profit = SeriesTree_Cohorts_Utxo_Profitability_Profit(client)
        self.loss: SeriesTree_Cohorts_Utxo_Profitability_Loss = SeriesTree_Cohorts_Utxo_Profitability_Loss(client)

class SeriesTree_Cohorts_Utxo_Matured:
    """Series tree node."""
    
    def __init__(self, client: BrkClient, base_path: str = ''):
        self.under_1h: AverageBlockCumulativeSumPattern3 = AverageBlockCumulativeSumPattern3(client, 'utxos_under_1h_old_matured_supply')
        self._1h_to_1d: AverageBlockCumulativeSumPattern3 = AverageBlockCumulativeSumPattern3(client, 'utxos_1h_to_1d_old_matured_supply')
        self._1d_to_1w: AverageBlockCumulativeSumPattern3 = AverageBlockCumulativeSumPattern3(client, 'utxos_1d_to_1w_old_matured_supply')
        self._1w_to_1m: AverageBlockCumulativeSumPattern3 = AverageBlockCumulativeSumPattern3(client, 'utxos_1w_to_1m_old_matured_supply')
        self._1m_to_2m: AverageBlockCumulativeSumPattern3 = AverageBlockCumulativeSumPattern3(client, 'utxos_1m_to_2m_old_matured_supply')
        self._2m_to_3m: AverageBlockCumulativeSumPattern3 = AverageBlockCumulativeSumPattern3(client, 'utxos_2m_to_3m_old_matured_supply')
        self._3m_to_4m: AverageBlockCumulativeSumPattern3 = AverageBlockCumulativeSumPattern3(client, 'utxos_3m_to_4m_old_matured_supply')
        self._4m_to_5m: AverageBlockCumulativeSumPattern3 = AverageBlockCumulativeSumPattern3(client, 'utxos_4m_to_5m_old_matured_supply')
        self._5m_to_6m: AverageBlockCumulativeSumPattern3 = AverageBlockCumulativeSumPattern3(client, 'utxos_5m_to_6m_old_matured_supply')
        self._6m_to_1y: AverageBlockCumulativeSumPattern3 = AverageBlockCumulativeSumPattern3(client, 'utxos_6m_to_1y_old_matured_supply')
        self._1y_to_2y: AverageBlockCumulativeSumPattern3 = AverageBlockCumulativeSumPattern3(client, 'utxos_1y_to_2y_old_matured_supply')
        self._2y_to_3y: AverageBlockCumulativeSumPattern3 = AverageBlockCumulativeSumPattern3(client, 'utxos_2y_to_3y_old_matured_supply')
        self._3y_to_4y: AverageBlockCumulativeSumPattern3 = AverageBlockCumulativeSumPattern3(client, 'utxos_3y_to_4y_old_matured_supply')
        self._4y_to_5y: AverageBlockCumulativeSumPattern3 = AverageBlockCumulativeSumPattern3(client, 'utxos_4y_to_5y_old_matured_supply')
        self._5y_to_6y: AverageBlockCumulativeSumPattern3 = AverageBlockCumulativeSumPattern3(client, 'utxos_5y_to_6y_old_matured_supply')
        self._6y_to_7y: AverageBlockCumulativeSumPattern3 = AverageBlockCumulativeSumPattern3(client, 'utxos_6y_to_7y_old_matured_supply')
        self._7y_to_8y: AverageBlockCumulativeSumPattern3 = AverageBlockCumulativeSumPattern3(client, 'utxos_7y_to_8y_old_matured_supply')
        self._8y_to_10y: AverageBlockCumulativeSumPattern3 = AverageBlockCumulativeSumPattern3(client, 'utxos_8y_to_10y_old_matured_supply')
        self._10y_to_12y: AverageBlockCumulativeSumPattern3 = AverageBlockCumulativeSumPattern3(client, 'utxos_10y_to_12y_old_matured_supply')
        self._12y_to_15y: AverageBlockCumulativeSumPattern3 = AverageBlockCumulativeSumPattern3(client, 'utxos_12y_to_15y_old_matured_supply')
        self.over_15y: AverageBlockCumulativeSumPattern3 = AverageBlockCumulativeSumPattern3(client, 'utxos_over_15y_old_matured_supply')

class SeriesTree_Cohorts_Utxo:
    """Series tree node."""
    
    def __init__(self, client: BrkClient, base_path: str = ''):
        self.all: SeriesTree_Cohorts_Utxo_All = SeriesTree_Cohorts_Utxo_All(client)
        self.sth: SeriesTree_Cohorts_Utxo_Sth = SeriesTree_Cohorts_Utxo_Sth(client)
        self.lth: SeriesTree_Cohorts_Utxo_Lth = SeriesTree_Cohorts_Utxo_Lth(client)
        self.age_range: SeriesTree_Cohorts_Utxo_AgeRange = SeriesTree_Cohorts_Utxo_AgeRange(client)
        self.under_age: SeriesTree_Cohorts_Utxo_UnderAge = SeriesTree_Cohorts_Utxo_UnderAge(client)
        self.over_age: SeriesTree_Cohorts_Utxo_OverAge = SeriesTree_Cohorts_Utxo_OverAge(client)
        self.epoch: SeriesTree_Cohorts_Utxo_Epoch = SeriesTree_Cohorts_Utxo_Epoch(client)
        self.class_: SeriesTree_Cohorts_Utxo_Class = SeriesTree_Cohorts_Utxo_Class(client)
        self.entry: SeriesTree_Cohorts_Utxo_Entry = SeriesTree_Cohorts_Utxo_Entry(client)
        self.over_amount: SeriesTree_Cohorts_Utxo_OverAmount = SeriesTree_Cohorts_Utxo_OverAmount(client)
        self.amount_range: SeriesTree_Cohorts_Utxo_AmountRange = SeriesTree_Cohorts_Utxo_AmountRange(client)
        self.under_amount: SeriesTree_Cohorts_Utxo_UnderAmount = SeriesTree_Cohorts_Utxo_UnderAmount(client)
        self.type: SeriesTree_Cohorts_Utxo_Type = SeriesTree_Cohorts_Utxo_Type(client)
        self.profitability: SeriesTree_Cohorts_Utxo_Profitability = SeriesTree_Cohorts_Utxo_Profitability(client)
        self.matured: SeriesTree_Cohorts_Utxo_Matured = SeriesTree_Cohorts_Utxo_Matured(client)

class SeriesTree_Cohorts_Addr_OverAmount:
    """Series tree node."""
    
    def __init__(self, client: BrkClient, base_path: str = ''):
        self._1sat: ActivityAddrOutputsRealizedSupplyUnrealizedPattern = ActivityAddrOutputsRealizedSupplyUnrealizedPattern(client, 'addrs_over_1sat')
        self._10sats: ActivityAddrOutputsRealizedSupplyUnrealizedPattern = ActivityAddrOutputsRealizedSupplyUnrealizedPattern(client, 'addrs_over_10sats')
        self._100sats: ActivityAddrOutputsRealizedSupplyUnrealizedPattern = ActivityAddrOutputsRealizedSupplyUnrealizedPattern(client, 'addrs_over_100sats')
        self._1k_sats: ActivityAddrOutputsRealizedSupplyUnrealizedPattern = ActivityAddrOutputsRealizedSupplyUnrealizedPattern(client, 'addrs_over_1k_sats')
        self._10k_sats: ActivityAddrOutputsRealizedSupplyUnrealizedPattern = ActivityAddrOutputsRealizedSupplyUnrealizedPattern(client, 'addrs_over_10k_sats')
        self._100k_sats: ActivityAddrOutputsRealizedSupplyUnrealizedPattern = ActivityAddrOutputsRealizedSupplyUnrealizedPattern(client, 'addrs_over_100k_sats')
        self._1m_sats: ActivityAddrOutputsRealizedSupplyUnrealizedPattern = ActivityAddrOutputsRealizedSupplyUnrealizedPattern(client, 'addrs_over_1m_sats')
        self._10m_sats: ActivityAddrOutputsRealizedSupplyUnrealizedPattern = ActivityAddrOutputsRealizedSupplyUnrealizedPattern(client, 'addrs_over_10m_sats')
        self._1btc: ActivityAddrOutputsRealizedSupplyUnrealizedPattern = ActivityAddrOutputsRealizedSupplyUnrealizedPattern(client, 'addrs_over_1btc')
        self._10btc: ActivityAddrOutputsRealizedSupplyUnrealizedPattern = ActivityAddrOutputsRealizedSupplyUnrealizedPattern(client, 'addrs_over_10btc')
        self._100btc: ActivityAddrOutputsRealizedSupplyUnrealizedPattern = ActivityAddrOutputsRealizedSupplyUnrealizedPattern(client, 'addrs_over_100btc')
        self._1k_btc: ActivityAddrOutputsRealizedSupplyUnrealizedPattern = ActivityAddrOutputsRealizedSupplyUnrealizedPattern(client, 'addrs_over_1k_btc')
        self._10k_btc: ActivityAddrOutputsRealizedSupplyUnrealizedPattern = ActivityAddrOutputsRealizedSupplyUnrealizedPattern(client, 'addrs_over_10k_btc')

class SeriesTree_Cohorts_Addr_AmountRange:
    """Series tree node."""
    
    def __init__(self, client: BrkClient, base_path: str = ''):
        self._0sats: ActivityAddrOutputsRealizedSupplyUnrealizedPattern = ActivityAddrOutputsRealizedSupplyUnrealizedPattern(client, 'addrs_0sats')
        self._1sat_to_10sats: ActivityAddrOutputsRealizedSupplyUnrealizedPattern = ActivityAddrOutputsRealizedSupplyUnrealizedPattern(client, 'addrs_1sat_to_10sats')
        self._10sats_to_100sats: ActivityAddrOutputsRealizedSupplyUnrealizedPattern = ActivityAddrOutputsRealizedSupplyUnrealizedPattern(client, 'addrs_10sats_to_100sats')
        self._100sats_to_1k_sats: ActivityAddrOutputsRealizedSupplyUnrealizedPattern = ActivityAddrOutputsRealizedSupplyUnrealizedPattern(client, 'addrs_100sats_to_1k_sats')
        self._1k_sats_to_10k_sats: ActivityAddrOutputsRealizedSupplyUnrealizedPattern = ActivityAddrOutputsRealizedSupplyUnrealizedPattern(client, 'addrs_1k_sats_to_10k_sats')
        self._10k_sats_to_100k_sats: ActivityAddrOutputsRealizedSupplyUnrealizedPattern = ActivityAddrOutputsRealizedSupplyUnrealizedPattern(client, 'addrs_10k_sats_to_100k_sats')
        self._100k_sats_to_1m_sats: ActivityAddrOutputsRealizedSupplyUnrealizedPattern = ActivityAddrOutputsRealizedSupplyUnrealizedPattern(client, 'addrs_100k_sats_to_1m_sats')
        self._1m_sats_to_10m_sats: ActivityAddrOutputsRealizedSupplyUnrealizedPattern = ActivityAddrOutputsRealizedSupplyUnrealizedPattern(client, 'addrs_1m_sats_to_10m_sats')
        self._10m_sats_to_1btc: ActivityAddrOutputsRealizedSupplyUnrealizedPattern = ActivityAddrOutputsRealizedSupplyUnrealizedPattern(client, 'addrs_10m_sats_to_1btc')
        self._1btc_to_10btc: ActivityAddrOutputsRealizedSupplyUnrealizedPattern = ActivityAddrOutputsRealizedSupplyUnrealizedPattern(client, 'addrs_1btc_to_10btc')
        self._10btc_to_100btc: ActivityAddrOutputsRealizedSupplyUnrealizedPattern = ActivityAddrOutputsRealizedSupplyUnrealizedPattern(client, 'addrs_10btc_to_100btc')
        self._100btc_to_1k_btc: ActivityAddrOutputsRealizedSupplyUnrealizedPattern = ActivityAddrOutputsRealizedSupplyUnrealizedPattern(client, 'addrs_100btc_to_1k_btc')
        self._1k_btc_to_10k_btc: ActivityAddrOutputsRealizedSupplyUnrealizedPattern = ActivityAddrOutputsRealizedSupplyUnrealizedPattern(client, 'addrs_1k_btc_to_10k_btc')
        self._10k_btc_to_100k_btc: ActivityAddrOutputsRealizedSupplyUnrealizedPattern = ActivityAddrOutputsRealizedSupplyUnrealizedPattern(client, 'addrs_10k_btc_to_100k_btc')
        self.over_100k_btc: ActivityAddrOutputsRealizedSupplyUnrealizedPattern = ActivityAddrOutputsRealizedSupplyUnrealizedPattern(client, 'addrs_over_100k_btc')

class SeriesTree_Cohorts_Addr_UnderAmount:
    """Series tree node."""
    
    def __init__(self, client: BrkClient, base_path: str = ''):
        self._10sats: ActivityAddrOutputsRealizedSupplyUnrealizedPattern = ActivityAddrOutputsRealizedSupplyUnrealizedPattern(client, 'addrs_under_10sats')
        self._100sats: ActivityAddrOutputsRealizedSupplyUnrealizedPattern = ActivityAddrOutputsRealizedSupplyUnrealizedPattern(client, 'addrs_under_100sats')
        self._1k_sats: ActivityAddrOutputsRealizedSupplyUnrealizedPattern = ActivityAddrOutputsRealizedSupplyUnrealizedPattern(client, 'addrs_under_1k_sats')
        self._10k_sats: ActivityAddrOutputsRealizedSupplyUnrealizedPattern = ActivityAddrOutputsRealizedSupplyUnrealizedPattern(client, 'addrs_under_10k_sats')
        self._100k_sats: ActivityAddrOutputsRealizedSupplyUnrealizedPattern = ActivityAddrOutputsRealizedSupplyUnrealizedPattern(client, 'addrs_under_100k_sats')
        self._1m_sats: ActivityAddrOutputsRealizedSupplyUnrealizedPattern = ActivityAddrOutputsRealizedSupplyUnrealizedPattern(client, 'addrs_under_1m_sats')
        self._10m_sats: ActivityAddrOutputsRealizedSupplyUnrealizedPattern = ActivityAddrOutputsRealizedSupplyUnrealizedPattern(client, 'addrs_under_10m_sats')
        self._1btc: ActivityAddrOutputsRealizedSupplyUnrealizedPattern = ActivityAddrOutputsRealizedSupplyUnrealizedPattern(client, 'addrs_under_1btc')
        self._10btc: ActivityAddrOutputsRealizedSupplyUnrealizedPattern = ActivityAddrOutputsRealizedSupplyUnrealizedPattern(client, 'addrs_under_10btc')
        self._100btc: ActivityAddrOutputsRealizedSupplyUnrealizedPattern = ActivityAddrOutputsRealizedSupplyUnrealizedPattern(client, 'addrs_under_100btc')
        self._1k_btc: ActivityAddrOutputsRealizedSupplyUnrealizedPattern = ActivityAddrOutputsRealizedSupplyUnrealizedPattern(client, 'addrs_under_1k_btc')
        self._10k_btc: ActivityAddrOutputsRealizedSupplyUnrealizedPattern = ActivityAddrOutputsRealizedSupplyUnrealizedPattern(client, 'addrs_under_10k_btc')
        self._100k_btc: ActivityAddrOutputsRealizedSupplyUnrealizedPattern = ActivityAddrOutputsRealizedSupplyUnrealizedPattern(client, 'addrs_under_100k_btc')

class SeriesTree_Cohorts_Addr:
    """Series tree node."""
    
    def __init__(self, client: BrkClient, base_path: str = ''):
        self.over_amount: SeriesTree_Cohorts_Addr_OverAmount = SeriesTree_Cohorts_Addr_OverAmount(client)
        self.amount_range: SeriesTree_Cohorts_Addr_AmountRange = SeriesTree_Cohorts_Addr_AmountRange(client)
        self.under_amount: SeriesTree_Cohorts_Addr_UnderAmount = SeriesTree_Cohorts_Addr_UnderAmount(client)

class SeriesTree_Cohorts:
    """Series tree node."""
    
    def __init__(self, client: BrkClient, base_path: str = ''):
        self.utxo: SeriesTree_Cohorts_Utxo = SeriesTree_Cohorts_Utxo(client)
        self.addr: SeriesTree_Cohorts_Addr = SeriesTree_Cohorts_Addr(client)

class SeriesTree:
    """Series tree node."""
    
    def __init__(self, client: BrkClient, base_path: str = ''):
        self.blocks: SeriesTree_Blocks = SeriesTree_Blocks(client)
        self.transactions: SeriesTree_Transactions = SeriesTree_Transactions(client)
        self.inputs: SeriesTree_Inputs = SeriesTree_Inputs(client)
        self.outputs: SeriesTree_Outputs = SeriesTree_Outputs(client)
        self.addrs: SeriesTree_Addrs = SeriesTree_Addrs(client)
        self.scripts: SeriesTree_Scripts = SeriesTree_Scripts(client)
        self.mining: SeriesTree_Mining = SeriesTree_Mining(client)
        self.cointime: SeriesTree_Cointime = SeriesTree_Cointime(client)
        self.constants: SeriesTree_Constants = SeriesTree_Constants(client)
        self.indexes: SeriesTree_Indexes = SeriesTree_Indexes(client)
        self.indicators: SeriesTree_Indicators = SeriesTree_Indicators(client)
        self.investing: SeriesTree_Investing = SeriesTree_Investing(client)
        self.market: SeriesTree_Market = SeriesTree_Market(client)
        self.pools: SeriesTree_Pools = SeriesTree_Pools(client)
        self.price: SeriesTree_Price = SeriesTree_Price(client)
        self.supply: SeriesTree_Supply = SeriesTree_Supply(client)
        self.cohorts: SeriesTree_Cohorts = SeriesTree_Cohorts(client)

class BrkClient(BrkClientBase):
    """Main BRK client with series tree and API methods."""

    VERSION = "v0.3.6"

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
      "empty_addr_index"
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
      "_6m_to_1y": {
        "id": "6m_to_1y_old",
        "short": "6m-1y",
        "long": "6 Months to 1 Year Old"
      },
      "_1y_to_2y": {
        "id": "1y_to_2y_old",
        "short": "1y-2y",
        "long": "1 to 2 Years Old"
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

    UNDER_AGE_NAMES = {
      "_1w": {
        "id": "under_1w_old",
        "short": "<1w",
        "long": "Under 1 Week Old"
      },
      "_1m": {
        "id": "under_1m_old",
        "short": "<1m",
        "long": "Under 1 Month Old"
      },
      "_2m": {
        "id": "under_2m_old",
        "short": "<2m",
        "long": "Under 2 Months Old"
      },
      "_3m": {
        "id": "under_3m_old",
        "short": "<3m",
        "long": "Under 3 Months Old"
      },
      "_4m": {
        "id": "under_4m_old",
        "short": "<4m",
        "long": "Under 4 Months Old"
      },
      "_5m": {
        "id": "under_5m_old",
        "short": "<5m",
        "long": "Under 5 Months Old"
      },
      "_6m": {
        "id": "under_6m_old",
        "short": "<6m",
        "long": "Under 6 Months Old"
      },
      "_1y": {
        "id": "under_1y_old",
        "short": "<1y",
        "long": "Under 1 Year Old"
      },
      "_2y": {
        "id": "under_2y_old",
        "short": "<2y",
        "long": "Under 2 Years Old"
      },
      "_3y": {
        "id": "under_3y_old",
        "short": "<3y",
        "long": "Under 3 Years Old"
      },
      "_4y": {
        "id": "under_4y_old",
        "short": "<4y",
        "long": "Under 4 Years Old"
      },
      "_5y": {
        "id": "under_5y_old",
        "short": "<5y",
        "long": "Under 5 Years Old"
      },
      "_6y": {
        "id": "under_6y_old",
        "short": "<6y",
        "long": "Under 6 Years Old"
      },
      "_7y": {
        "id": "under_7y_old",
        "short": "<7y",
        "long": "Under 7 Years Old"
      },
      "_8y": {
        "id": "under_8y_old",
        "short": "<8y",
        "long": "Under 8 Years Old"
      },
      "_10y": {
        "id": "under_10y_old",
        "short": "<10y",
        "long": "Under 10 Years Old"
      },
      "_12y": {
        "id": "under_12y_old",
        "short": "<12y",
        "long": "Under 12 Years Old"
      },
      "_15y": {
        "id": "under_15y_old",
        "short": "<15y",
        "long": "Under 15 Years Old"
      }
    }

    OVER_AGE_NAMES = {
      "_1d": {
        "id": "over_1d_old",
        "short": "1d+",
        "long": "Over 1 Day Old"
      },
      "_1w": {
        "id": "over_1w_old",
        "short": "1w+",
        "long": "Over 1 Week Old"
      },
      "_1m": {
        "id": "over_1m_old",
        "short": "1m+",
        "long": "Over 1 Month Old"
      },
      "_2m": {
        "id": "over_2m_old",
        "short": "2m+",
        "long": "Over 2 Months Old"
      },
      "_3m": {
        "id": "over_3m_old",
        "short": "3m+",
        "long": "Over 3 Months Old"
      },
      "_4m": {
        "id": "over_4m_old",
        "short": "4m+",
        "long": "Over 4 Months Old"
      },
      "_5m": {
        "id": "over_5m_old",
        "short": "5m+",
        "long": "Over 5 Months Old"
      },
      "_6m": {
        "id": "over_6m_old",
        "short": "6m+",
        "long": "Over 6 Months Old"
      },
      "_1y": {
        "id": "over_1y_old",
        "short": "1y+",
        "long": "Over 1 Year Old"
      },
      "_2y": {
        "id": "over_2y_old",
        "short": "2y+",
        "long": "Over 2 Years Old"
      },
      "_3y": {
        "id": "over_3y_old",
        "short": "3y+",
        "long": "Over 3 Years Old"
      },
      "_4y": {
        "id": "over_4y_old",
        "short": "4y+",
        "long": "Over 4 Years Old"
      },
      "_5y": {
        "id": "over_5y_old",
        "short": "5y+",
        "long": "Over 5 Years Old"
      },
      "_6y": {
        "id": "over_6y_old",
        "short": "6y+",
        "long": "Over 6 Years Old"
      },
      "_7y": {
        "id": "over_7y_old",
        "short": "7y+",
        "long": "Over 7 Years Old"
      },
      "_8y": {
        "id": "over_8y_old",
        "short": "8y+",
        "long": "Over 8 Years Old"
      },
      "_10y": {
        "id": "over_10y_old",
        "short": "10y+",
        "long": "Over 10 Years Old"
      },
      "_12y": {
        "id": "over_12y_old",
        "short": "12y+",
        "long": "Over 12 Years Old"
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

    OVER_AMOUNT_NAMES = {
      "_1sat": {
        "id": "over_1sat",
        "short": "1+ sats",
        "long": "Over 1 Sat"
      },
      "_10sats": {
        "id": "over_10sats",
        "short": "10+ sats",
        "long": "Over 10 Sats"
      },
      "_100sats": {
        "id": "over_100sats",
        "short": "100+ sats",
        "long": "Over 100 Sats"
      },
      "_1k_sats": {
        "id": "over_1k_sats",
        "short": "1k+ sats",
        "long": "Over 1K Sats"
      },
      "_10k_sats": {
        "id": "over_10k_sats",
        "short": "10k+ sats",
        "long": "Over 10K Sats"
      },
      "_100k_sats": {
        "id": "over_100k_sats",
        "short": "100k+ sats",
        "long": "Over 100K Sats"
      },
      "_1m_sats": {
        "id": "over_1m_sats",
        "short": "1M+ sats",
        "long": "Over 1M Sats"
      },
      "_10m_sats": {
        "id": "over_10m_sats",
        "short": "0.1+ BTC",
        "long": "Over 0.1 BTC"
      },
      "_1btc": {
        "id": "over_1btc",
        "short": "1+ BTC",
        "long": "Over 1 BTC"
      },
      "_10btc": {
        "id": "over_10btc",
        "short": "10+ BTC",
        "long": "Over 10 BTC"
      },
      "_100btc": {
        "id": "over_100btc",
        "short": "100+ BTC",
        "long": "Over 100 BTC"
      },
      "_1k_btc": {
        "id": "over_1k_btc",
        "short": "1k+ BTC",
        "long": "Over 1K BTC"
      },
      "_10k_btc": {
        "id": "over_10k_btc",
        "short": "10k+ BTC",
        "long": "Over 10K BTC"
      }
    }

    UNDER_AMOUNT_NAMES = {
      "_10sats": {
        "id": "under_10sats",
        "short": "<10 sats",
        "long": "Under 10 Sats"
      },
      "_100sats": {
        "id": "under_100sats",
        "short": "<100 sats",
        "long": "Under 100 Sats"
      },
      "_1k_sats": {
        "id": "under_1k_sats",
        "short": "<1k sats",
        "long": "Under 1K Sats"
      },
      "_10k_sats": {
        "id": "under_10k_sats",
        "short": "<10k sats",
        "long": "Under 10K Sats"
      },
      "_100k_sats": {
        "id": "under_100k_sats",
        "short": "<100k sats",
        "long": "Under 100K Sats"
      },
      "_1m_sats": {
        "id": "under_1m_sats",
        "short": "<1M sats",
        "long": "Under 1M Sats"
      },
      "_10m_sats": {
        "id": "under_10m_sats",
        "short": "<0.1 BTC",
        "long": "Under 0.1 BTC"
      },
      "_1btc": {
        "id": "under_1btc",
        "short": "<1 BTC",
        "long": "Under 1 BTC"
      },
      "_10btc": {
        "id": "under_10btc",
        "short": "<10 BTC",
        "long": "Under 10 BTC"
      },
      "_100btc": {
        "id": "under_100btc",
        "short": "<100 BTC",
        "long": "Under 100 BTC"
      },
      "_1k_btc": {
        "id": "under_1k_btc",
        "short": "<1k BTC",
        "long": "Under 1K BTC"
      },
      "_10k_btc": {
        "id": "under_10k_btc",
        "short": "<10k BTC",
        "long": "Under 10K BTC"
      },
      "_100k_btc": {
        "id": "under_100k_btc",
        "short": "<100k BTC",
        "long": "Under 100K BTC"
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

    PROFIT_NAMES = {
      "all": {
        "id": "utxos_in_profit",
        "short": "All",
        "long": "In Profit"
      },
      "_10pct": {
        "id": "utxos_over_10pct_in_profit",
        "short": ">=10%",
        "long": "Over 10% in Profit"
      },
      "_20pct": {
        "id": "utxos_over_20pct_in_profit",
        "short": ">=20%",
        "long": "Over 20% in Profit"
      },
      "_30pct": {
        "id": "utxos_over_30pct_in_profit",
        "short": ">=30%",
        "long": "Over 30% in Profit"
      },
      "_40pct": {
        "id": "utxos_over_40pct_in_profit",
        "short": ">=40%",
        "long": "Over 40% in Profit"
      },
      "_50pct": {
        "id": "utxos_over_50pct_in_profit",
        "short": ">=50%",
        "long": "Over 50% in Profit"
      },
      "_60pct": {
        "id": "utxos_over_60pct_in_profit",
        "short": ">=60%",
        "long": "Over 60% in Profit"
      },
      "_70pct": {
        "id": "utxos_over_70pct_in_profit",
        "short": ">=70%",
        "long": "Over 70% in Profit"
      },
      "_80pct": {
        "id": "utxos_over_80pct_in_profit",
        "short": ">=80%",
        "long": "Over 80% in Profit"
      },
      "_90pct": {
        "id": "utxos_over_90pct_in_profit",
        "short": ">=90%",
        "long": "Over 90% in Profit"
      },
      "_100pct": {
        "id": "utxos_over_100pct_in_profit",
        "short": ">=100%",
        "long": "Over 100% in Profit"
      },
      "_200pct": {
        "id": "utxos_over_200pct_in_profit",
        "short": ">=200%",
        "long": "Over 200% in Profit"
      },
      "_300pct": {
        "id": "utxos_over_300pct_in_profit",
        "short": ">=300%",
        "long": "Over 300% in Profit"
      },
      "_500pct": {
        "id": "utxos_over_500pct_in_profit",
        "short": ">=500%",
        "long": "Over 500% in Profit"
      }
    }

    LOSS_NAMES = {
      "all": {
        "id": "utxos_in_loss",
        "short": "All",
        "long": "In Loss"
      },
      "_10pct": {
        "id": "utxos_over_10pct_in_loss",
        "short": ">=10%",
        "long": "Over 10% in Loss"
      },
      "_20pct": {
        "id": "utxos_over_20pct_in_loss",
        "short": ">=20%",
        "long": "Over 20% in Loss"
      },
      "_30pct": {
        "id": "utxos_over_30pct_in_loss",
        "short": ">=30%",
        "long": "Over 30% in Loss"
      },
      "_40pct": {
        "id": "utxos_over_40pct_in_loss",
        "short": ">=40%",
        "long": "Over 40% in Loss"
      },
      "_50pct": {
        "id": "utxos_over_50pct_in_loss",
        "short": ">=50%",
        "long": "Over 50% in Loss"
      },
      "_60pct": {
        "id": "utxos_over_60pct_in_loss",
        "short": ">=60%",
        "long": "Over 60% in Loss"
      },
      "_70pct": {
        "id": "utxos_over_70pct_in_loss",
        "short": ">=70%",
        "long": "Over 70% in Loss"
      },
      "_80pct": {
        "id": "utxos_over_80pct_in_loss",
        "short": ">=80%",
        "long": "Over 80% in Loss"
      }
    }

    def __init__(self, base_url: str = 'http://localhost:3000', timeout: float = 30.0):
        super().__init__(base_url, timeout)
        self.series = SeriesTree(self)

    def series_endpoint(self, series: str, index: Index) -> SeriesEndpoint[Any]:
        """Create a dynamic series endpoint builder for any series/index combination.

        Use this for programmatic access when the series name is determined at runtime.
        For type-safe access, use the `series` tree instead.
        """
        return SeriesEndpoint(self, series, index)

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

        Liveness probe. Returns server identity, uptime, and indexed/computed heights from local state only (no bitcoind round-trip). For real chain-tip catch-up, see `/api/server/sync`.

        Endpoint: `GET /health`"""
        return self.get_json('/health')

    def get_version(self) -> str:
        """API version.

        Returns the current version of the API server

        Endpoint: `GET /version`"""
        return self.get_json('/version')

    def get_sync_status(self) -> SyncStatus:
        """Sync status.

        Returns the sync status of the indexer, including indexed height, tip height, blocks behind, and last indexed timestamp.

        Endpoint: `GET /api/server/sync`"""
        return self.get_json('/api/server/sync')

    def get_disk_usage(self) -> DiskUsage:
        """Disk usage.

        Returns the disk space used by BRK and Bitcoin data.

        Endpoint: `GET /api/server/disk`"""
        return self.get_json('/api/server/disk')

    def get_series_tree(self) -> TreeNode:
        """Series catalog.

        Returns the complete hierarchical catalog of available series organized as a tree structure. Series are grouped by categories and subcategories.

        Endpoint: `GET /api/series`"""
        return self.get_json('/api/series')

    def get_series_count(self) -> List[SeriesCount]:
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

        Fuzzy search for series by name. Supports partial matches and typos.

        Endpoint: `GET /api/series/search`"""
        params = []
        params.append(f'q={q}')
        if limit is not None: params.append(f'limit={limit}')
        query = '&'.join(params)
        path = f'/api/series/search{"?" + query if query else ""}'
        return self.get_json(path)

    def get_series_info(self, series: SeriesName) -> SeriesInfo:
        """Get series info.

        Returns the supported indexes and value type for the specified series.

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
        return self.get_json(path)

    def get_series_data(self, series: SeriesName, index: Index, start: Optional[RangeIndex] = None, end: Optional[RangeIndex] = None, limit: Optional[Limit] = None, format: Optional[Format] = None) -> Union[List[bool], str]:
        """Get raw series data.

        Returns just the data array without the SeriesData wrapper. Supports the same range and format parameters as the standard endpoint.

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

        Returns the current version of a series. Changes when the series data is updated.

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
        return self.get_json(path)

    def list_urpd_cohorts(self) -> List[Cohort]:
        """Available URPD cohorts.

        Cohorts for which URPD data is available. Returns names like `all`, `sth`, `lth`, `utxos_under_1h_old`.

        Endpoint: `GET /api/urpd`"""
        return self.get_json('/api/urpd')

    def list_urpd_dates(self, cohort: Cohort) -> List[Date]:
        """Available URPD dates.

        Dates for which a URPD snapshot is available for the cohort. One entry per UTC day, sorted ascending.

        Endpoint: `GET /api/urpd/{cohort}/dates`"""
        return self.get_json(f'/api/urpd/{cohort}/dates')

    def get_urpd(self, cohort: Cohort, agg: Optional[UrpdAggregation] = None) -> Urpd:
        """Latest URPD.

        URPD for the most recent available date in the cohort. The response's `date` field echoes which date was served.

        See the URPD tag description for the response shape and `agg` options.

        Endpoint: `GET /api/urpd/{cohort}`"""
        params = []
        if agg is not None: params.append(f'agg={agg}')
        query = '&'.join(params)
        path = f'/api/urpd/{cohort}{"?" + query if query else ""}'
        return self.get_json(path)

    def get_urpd_at(self, cohort: Cohort, date: str, agg: Optional[UrpdAggregation] = None) -> Urpd:
        """URPD at date.

        URPD for a (cohort, date) pair. Returns `{ cohort, date, aggregation, close, total_supply, buckets }` where each bucket is `{ price_floor, supply, realized_cap, unrealized_pnl }`.

        See the URPD tag description for unit conventions and `agg` options.

        Endpoint: `GET /api/urpd/{cohort}/{date}`"""
        params = []
        if agg is not None: params.append(f'agg={agg}')
        query = '&'.join(params)
        path = f'/api/urpd/{cohort}/{date}{"?" + query if query else ""}'
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

        Get historical BTC/USD price. Optionally specify a UNIX timestamp to get the price at that time.

        *[Mempool.space docs](https://mempool.space/docs/api/rest#get-historical-price)*

        Endpoint: `GET /api/v1/historical-price`"""
        params = []
        if timestamp is not None: params.append(f'timestamp={timestamp}')
        query = '&'.join(params)
        path = f'/api/v1/historical-price{"?" + query if query else ""}'
        return self.get_json(path)

    def get_address_hash_prefix_matches(self, addr_type: OutputType, prefix: str) -> AddrHashPrefixMatches:
        """Address hash-prefix matches.

        Find addresses by address type and by the first 1-16 hex nibbles of RapidHash v3 over the raw address payload bytes. Intended for privacy-preserving client-side wallet discovery without sending raw addresses or xpubs. Fetch metadata for the returned addresses through `/api/address/{address}`.

        Endpoint: `GET /api/address/hash-prefix/{addr_type}/{prefix}`"""
        return self.get_json(f'/api/address/hash-prefix/{addr_type}/{prefix}')

    def get_address(self, address: Addr) -> AddrStats:
        """Address information.

        Retrieve address information including balance and transaction counts. Supports all standard Bitcoin address types (P2PKH, P2SH, P2WPKH, P2WSH, P2TR).

        *[Mempool.space docs](https://mempool.space/docs/api/rest#get-address)*

        Endpoint: `GET /api/address/{address}`"""
        return self.get_json(f'/api/address/{address}')

    def get_address_txs(self, address: Addr) -> List[Transaction]:
        """Address transactions.

        Get transaction history for an address, newest first. Returns up to 50 mempool transactions plus a confirmed page sized to fill the response to 50 total (chain floor of 25, so 25-50 confirmed depending on mempool weight). To paginate further confirmed history, use `/address/{address}/txs/chain/{last_seen_txid}`.

        *[Mempool.space docs](https://mempool.space/docs/api/rest#get-address-transactions)*

        Endpoint: `GET /api/address/{address}/txs`"""
        return self.get_json(f'/api/address/{address}/txs')

    def get_address_confirmed_txs(self, address: Addr) -> List[Transaction]:
        """Address confirmed transactions.

        Get the first 25 confirmed transactions for an address. For pagination, use the path-style form `/txs/chain/{last_seen_txid}`.

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

        Find the block closest to a given UNIX timestamp.

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

        Get the 10 most recent blocks mined by a specific pool.

        *[Mempool.space docs](https://mempool.space/docs/api/rest#get-mining-pool-blocks)*

        Endpoint: `GET /api/v1/mining/pool/{slug}/blocks`"""
        return self.get_json(f'/api/v1/mining/pool/{slug}/blocks')

    def get_pool_blocks_from(self, slug: PoolSlug, height: Height) -> List[BlockInfoV1]:
        """Mining pool blocks from height.

        Get 10 blocks mined by a specific pool before (and including) the given height.

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
        """Precise recommended fees.

        Recommended fee rates with sub-integer precision.

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

        Returns an opaque hash that changes whenever the projected next block changes. Same value as the mempool ETag. Useful as a freshness/liveness signal: if it stays constant for tens of seconds on a live network, the mempool sync loop has stalled.

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

        Like `/api/v1/replacements`, but limited to trees where at least one predecessor was non-signaling (full-RBF).

        *[Mempool.space docs](https://mempool.space/docs/api/rest#get-fullrbf-replacements)*

        Endpoint: `GET /api/v1/fullrbf/replacements`"""
        return self.get_json('/api/v1/fullrbf/replacements')

    def get_block_template(self) -> BlockTemplate:
        """Projected next block template.

        Bitcoin Core's `getblocktemplate` selection: full transaction bodies in GBT order with aggregate stats. The returned `hash` is an opaque content token; pass it as `<hash>` on `/api/v1/mempool/block-template/diff/{hash}` to fetch deltas instead of refetching the whole template.

        Endpoint: `GET /api/v1/mempool/block-template`"""
        return self.get_json('/api/v1/mempool/block-template')

    def get_block_template_diff(self, hash: NextBlockHash) -> BlockTemplateDiff:
        """Block template diff since hash.

        Delta of the projected next block since `<hash>`. `order` is the full new template in order: each entry is either a number (index into the prior template the client cached at `<hash>`) or a transaction object (new body to insert at this position). Walk `order` once to rebuild; `removed` is a convenience list of txids that left so clients can evict cached bodies. After applying, use the response `hash` as `<hash>` on the next call to keep iterating. Returns `404` when `<hash>` has aged out of server history; clients should fall back to `/api/v1/mempool/block-template`.

        Endpoint: `GET /api/v1/mempool/block-template/diff/{hash}`"""
        return self.get_json(f'/api/v1/mempool/block-template/diff/{hash}')

    def get_live_price(self) -> Dollars:
        """Live BTC/USD price.

        Returns the current BTC/USD price in dollars, derived from on-chain round-dollar output patterns in the last 12 blocks plus mempool.

        Endpoint: `GET /api/mempool/price`"""
        return self.get_json('/api/mempool/price')

    def get_oracle_price(self) -> Dollars:
        """Live BTC/USD price.

        Current BTC/USD price in dollars. Same value as `/api/mempool/price`. Confirmed per-height history is available at `/api/vecs/height-to-price`.

        Endpoint: `GET /api/oracle/price`"""
        return self.get_json('/api/oracle/price')

    def get_oracle_histogram_payments_live(self) -> List[int]:
        """Live payment output histogram.

        Live smoothed histogram of oracle-eligible payment outputs, binned by output value on the oracle log scale. It combines the committed oracle window with the forming mempool block. A flat array of log-scale bins.

        Endpoint: `GET /api/oracle/histogram/payments/live`"""
        return self.get_json('/api/oracle/histogram/payments/live')

    def get_oracle_histogram_payments(self, point: str) -> List[int]:
        """Payment output histogram at height or day.

        Smoothed histogram of oracle-eligible payment outputs for a confirmed point. A block height (`840000`) gives that block's oracle payment histogram; a calendar date (`YYYY-MM-DD`) gives the average of that day's per-block payment histograms. A flat array of log-scale bins.

        Endpoint: `GET /api/oracle/histogram/payments/{point}`"""
        return self.get_json(f'/api/oracle/histogram/payments/{point}')

    def get_oracle_histogram_outputs_live(self) -> List[int]:
        """Live output value histogram.

        Live unfiltered output value histogram for the forming mempool block. Every live output is binned by value on the oracle log scale; no oracle payment filters are applied. A flat array of log-scale bins, all zero when no mempool is configured.

        Endpoint: `GET /api/oracle/histogram/outputs/live`"""
        return self.get_json('/api/oracle/histogram/outputs/live')

    def get_oracle_histogram_outputs(self, point: str) -> List[int]:
        """Output value histogram at height or day.

        Unfiltered output value histogram for a confirmed point. A block height (`840000`) gives every output in that block, coinbase included, binned by value on the oracle log scale; a calendar date (`YYYY-MM-DD`) sums every block that day. A flat array of log-scale bins.

        Endpoint: `GET /api/oracle/histogram/outputs/{point}`"""
        return self.get_json(f'/api/oracle/histogram/outputs/{point}')

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

        Broadcast a raw transaction to the network. The transaction should be provided as hex in the request body. The txid will be returned on success.

        *[Mempool.space docs](https://mempool.space/docs/api/rest#post-transaction)*

        Endpoint: `POST /api/tx`"""
        return self.post_json('/api/tx', body)

    def get_openapi(self) -> str:
        """OpenAPI specification.

        Full OpenAPI 3.1 specification for this API.

        Endpoint: `GET /openapi.json`"""
        return self.get_text('/openapi.json')

    def get_api(self) -> Any:
        """Compact OpenAPI specification.

        Compact OpenAPI specification optimized for LLM consumption. Removes redundant fields while preserving essential API information. Full spec available at `/openapi.json`.

        Endpoint: `GET /api.json`"""
        return self.get_json('/api.json')

