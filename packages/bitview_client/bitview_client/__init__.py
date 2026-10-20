# Auto-generated Bitview Python client
# Do not edit manually

from __future__ import annotations
from dataclasses import dataclass
from functools import cached_property
from typing import TypeVar, Generic, Any, Callable, Dict, Optional, List, Iterator, Literal, TypedDict, Union, Protocol, Tuple, TYPE_CHECKING
from http.client import HTTPException, HTTPSConnection, HTTPConnection
from urllib.parse import quote, urlencode, urlparse
from datetime import date, datetime, timedelta, timezone
import json
import re

if TYPE_CHECKING:
    import pandas as pd  # type: ignore[import-not-found]
    import polars as pl  # type: ignore[import-not-found]

T = TypeVar('T')

# Index of a decade.
Year10 = int
# Index of a year.
Year1 = int
# A mean weight in weight units.
WeightFract = float
# Weight in weight units (WU): non-witness bytes count four and witness bytes one. A block holds
# at most 4,000,000 WU; sums over many blocks use the full 64 bits.
Weight = int
# Index of a week.
Week1 = int
# Index of the output being spent in the previous transaction (coinbase inputs: 4294967295)
Vout = int
# Input index in the spending transaction
Vin = int
# Schema or computation version of a stored vector: when it changes, the data is reset and
# recomputed.
Version = int
# Virtual size in vbytes (weight / 4, rounded up). Max block vsize is ~1,000,000 vB.
VSize = int
# The weight applied to a URPD's age bands: `none` (every satoshi counts once), `wakefulness` (each band
# weighted by the share of its coin days already consumed, Cointime's; the distribution behind the
# `*_awake_*` series) or `mobility` (each band weighted by the estimated probability its coins are ever
# spent, Coinflow's; behind `*_mobile_*`).
UrpdWeight = Literal["none", "wakefulness", "mobility"]
# Aggregation strategy for URPD buckets.
# Options: raw (no aggregation), lin200/lin500/lin1000 (linear $200/$500/$1000),
# log10/log50/log100/log200/log500/log1000/log2000 (logarithmic with 10/50/100/200/500/1000/2000 buckets per decade).
UrpdAggregation = Literal["raw", "lin200", "lin500", "lin1000", "log10", "log50", "log100", "log200", "log500", "log1000", "log2000"]
# Index within its type (e.g., 0 for first P2WPKH address)
TypeIndex = int
# Index of an output with an unrecognized script.
UnknownOutputIndex = TypeIndex
# Index of a P2WSH address.
P2WSHAddressIndex = TypeIndex
# Index of a P2WPKH address.
P2WPKHAddressIndex = TypeIndex
# Index of a P2TR (taproot) address.
P2TRAddressIndex = TypeIndex
# Index of a P2SH address.
P2SHAddressIndex = TypeIndex
# Index of a P2PKH address.
P2PKHAddressIndex = TypeIndex
# Index of a P2PK address with an uncompressed (65-byte) public key.
P2PK65AddressIndex = TypeIndex
# Index of a P2PK address with a compressed (33-byte) public key.
P2PK33AddressIndex = TypeIndex
# Index of a P2MS (bare multisig) output.
P2MSOutputIndex = TypeIndex
# Index of a P2A (pay-to-anchor) address.
P2AAddressIndex = TypeIndex
# Index of an OP_RETURN output.
OpReturnIndex = TypeIndex
# Index of an output with an empty script.
EmptyOutputIndex = TypeIndex
# Transaction ID (hash)
Txid = str
# Raw transaction version (i32) from Bitcoin protocol.
# Unlike TxVersion (u8, indexed), this preserves non-standard values
# used in coinbase txs for miner signaling/branding.
TxVersionRaw = int
# Compact indexed transaction-version category. Values 1, 2, and 3 preserve
# those exact signed 32-bit Bitcoin transaction versions; 255 represents every
# other version.
TxVersion = int
# Index of a transaction output.
TxOutIndex = int
# Chain-wide transaction index (0 = the genesis coinbase). For an
# in-block position, use `BlockTxIndex` instead.
TxIndex = int
# Index of a transaction input.
TxInIndex = int
# UNIX timestamp in seconds
Timestamp = int
# Lookback window for mining statistics.
TimePeriod = Literal["24h", "3d", "1w", "1m", "3m", "6m", "1y", "2y", "3y", "all"]
# BIP-141 signature-operation cost: legacy and P2SH-redeem sigops count four and witness sigops
# one. Sums over many transactions or blocks use the full 64 bits.
SigOps = int
SeriesName = str
# Comma-separated list of series names
#
# Deserialization permits at most 32 normalized names and 2,048 decoded input
# string bytes. For arrays, the byte budget is shared by their string values.
SeriesList = str
# A mean duration in seconds.
SecondsFract = float
# A duration in seconds (at most about 136 years).
Seconds = int
# A signed model score; undefined (null) where the model has no calibration yet.
Score = int
# A signed amount in satoshis.
SatsSigned = int
# Amount in satoshis (1 BTC = 100,000,000 sats)
Sats = int
# Transaction locktime. Values below 500,000,000 are interpreted as block heights; values at or above are Unix timestamps.
RawLockTime = int
# A dimensionless ratio: a quotient or share, read as a plain number.
Ratio = float
# A positional index, YYYY-MM-DD date, or ISO 8601 timestamp.
RangeIndex = Union[int, str, str]
# URL-friendly mining pool identifier
PoolSlug = Literal["unknown", "blockfills", "ultimuspool", "terrapool", "luxor", "1thash", "btccom", "bitfarms", "huobipool", "wayicn", "canoepool", "btctop", "bitcoincom", "175btc", "gbminers", "axbt", "asicminer", "bitminter", "bitcoinrussia", "btcserv", "simplecoinus", "btcguild", "eligius", "ozcoin", "eclipsemc", "maxbtc", "triplemining", "coinlab", "50btc", "ghashio", "stminingcorp", "bitparking", "mmpool", "polmine", "kncminer", "bitalo", "f2pool", "hhtt", "megabigpower", "mtred", "nmcbit", "yourbtcnet", "givemecoins", "braiinspool", "antpool", "multicoinco", "bcpoolio", "cointerra", "kanopool", "solock", "ckpool", "nicehash", "bitclub", "bitcoinaffiliatenetwork", "btcc", "bwpool", "exxbw", "bitsolo", "bitfury", "21inc", "digitalbtc", "8baochi", "mybtccoinpool", "tbdice", "hashpool", "nexious", "bravomining", "hotpool", "okexpool", "bcmonster", "1hash", "bixin", "tatmaspool", "viabtc", "connectbtc", "batpool", "waterhole", "dcexploration", "dcex", "btpool", "58coin", "bitcoinindia", "shawnp0wers", "phashio", "rigpool", "haozhuzhu", "7pool", "miningkings", "hashbx", "dpool", "rawpool", "haominer", "helix", "bitcoinukraine", "poolin", "secretsuperstar", "tigerpoolnet", "sigmapoolcom", "okpooltop", "hummerpool", "tangpool", "bytepool", "spiderpool", "novablock", "miningcity", "binancepool", "minerium", "lubiancom", "okkong", "aaopool", "emcdpool", "foundryusa", "sbicrypto", "arkpool", "purebtccom", "marapool", "kucoinpool", "entrustcharitypool", "okminer", "titan", "pegapool", "btcnuggets", "cloudhashing", "digitalxmintsy", "telco214", "btcpoolparty", "multipool", "transactioncoinmining", "btcdig", "trickysbtcpool", "btcmp", "eobot", "unomp", "patels", "gogreenlight", "bitcoinindiapool", "ekanembtc", "canoe", "tiger", "1m1x", "zulupool", "secpool", "ocean", "whitepool", "wiz", "wk057", "futurebitapollosolo", "carbonnegative", "portlandhodl", "phoenix", "neopool", "maxipool", "bitfufupool", "gdpool", "miningdutch", "publicpool", "miningsquared", "innopolistech", "btclab", "parasite", "redrockpool", "est3lar", "braiinssolo", "solopoolcom", "noderunners", "dmnd"]
# A percentage: a ratio times 100.
Percent = float
# Events per second.
PerSecond = float
# A fraction per day.
PerDay = float
# The 32-byte script hash of a P2WSH output.
P2WSHBytes = str
# The 20-byte public key hash of a P2WPKH output.
P2WPKHBytes = str
# The 32-byte output key of a P2TR output.
P2TRBytes = str
# The 20-byte script hash of a P2SH output.
P2SHBytes = str
# The 20-byte public key hash of a P2PKH output.
P2PKHBytes = str
# An uncompressed (65-byte) public key.
P2PK65Bytes = str
# A compressed (33-byte) public key.
P2PK33Bytes = str
# The 2-byte witness program of a P2A (pay-to-anchor) output.
P2ABytes = str
# Output type names used by Esplora and mempool.space.
OutputTypeNormalized = Literal["p2pk", "p2pkh", "multisig", "p2sh", "op_return", "v0_p2wpkh", "v0_p2wsh", "v1_p2tr", "anchor", "empty", "unknown"]
# Type (P2PKH, P2WPKH, P2SH, P2TR, etc.)
OutputType = Literal["p2pk65", "p2pk33", "p2pkh", "p2ms", "p2sh", "opreturn", "p2wpkh", "p2wsh", "p2tr", "p2a", "empty", "unknown"]
# Protocol or format detected in an OP_RETURN output.
OpReturnKind = Literal["runes", "witness_commitment", "veri_block", "omni", "stacks", "blockstack", "colu", "open_assets", "komodo", "coin_spark", "poet", "docproof", "factom", "eternity_wall", "memo", "bitproof", "ascribe", "stampery", "bare_hash", "text", "empty", "unknown"]
# Content hash of the projected next block (block 0 of the mempool
# snapshot), including its statistics and complete transaction bodies.
# Opaque token, distinct from HTTP ETag formatting: pass back
# to `GET /api/v1/mempool/block-template/diff/{hash}` to fetch deltas.
# A decimal string, since JavaScript numbers can't hold every 64-bit value.
NextBlockHash = str
# A multiple: one value over another of its kind, 1 at parity (MVRV, SOPR, a price over its
# average).
Multiple = float
# Index of a half-year (6 months).
Month6 = int
# Index of a quarter (3 months).
Month3 = int
# Index of a month.
Month1 = int
# Index of a 30-minute period.
Minute30 = int
# Index of a 10-minute period.
Minute10 = int
# Maximum number of results to return; each parameter states what omitting it means.
Limit = int
# Aggregation dimension for querying series. Includes time-based (date, week, month, year),
# block-based (height, tx_index), and address/output type indexes.
Index = Literal["minute10", "minute30", "hour1", "hour4", "hour12", "day1", "day3", "week1", "month1", "month3", "month6", "year1", "year10", "halving", "epoch", "height", "tx_index", "txin_index", "txout_index", "empty_output_index", "op_return_index", "p2a_address_index", "p2ms_output_index", "p2pk33_address_index", "p2pk65_address_index", "p2pkh_address_index", "p2sh_address_index", "p2tr_address_index", "p2wpkh_address_index", "p2wsh_address_index", "unknown_output_index"]
TreeNode = Union[dict[str, "TreeNode"], "SeriesLeafWithSchema"]
# Index of a 4-hour period.
Hour4 = int
# Index of a 12-hour period.
Hour12 = int
# Index of an hour.
Hour1 = int
# Hex-encoded data as a JSON string.
Hex = str
# Block height
Height = int
# Hashes per second.
Hashrate = float
# Satoshis per PH/s of hash rate per day.
HashValue = float
# US dollars per PH/s of hash rate per day.
HashPrice = float
# Index of a halving epoch (210,000 blocks).
Halving = int
Format = Literal["json", "csv"]
# A floating-point value without a specific unit.
Float = float
# Fee rate stored in milli-sat/vB and exposed as sat/vB.
FeeRate = float
# Always empty: prices are quoted in USD only.
ExchangeRates = dict
# Error category, following the HTTP status: `invalid_request` (4xx other than 404),
# `not_found` (404), `unavailable` (503; `Retry-After` when transient), `timeout` (504), `internal`
# (other 5xx).
ErrorType = Literal["invalid_request", "not_found", "unavailable", "timeout", "internal"]
ErrorCode = Literal["not_found", "invalid_address", "invalid_network", "unsupported_type", "no_series", "series_unsupported_index", "weight_exceeded", "too_many_utxos", "unknown_address", "unknown_txid", "out_of_range", "unindexable_date", "no_data", "series_not_found", "mempool_not_available", "state_updating", "internal_error", "bad_request", "overloaded", "timeout", "method_not_allowed"]
# Index of a difficulty epoch (2,016 blocks).
Epoch = int
# US Dollar amount
Dollars = float
# Open, high, low and close prices in US dollars: [open, high, low, close].
OHLCDollars = List[Dollars]
# US dollars multiplied by days, per bitcoin: coin days valued at the spot price, per
# bitcoin of supply.
DollarDaysPerCoin = float
# US dollars multiplied by blocks: coinblocks valued at the spot price.
DollarBlocks = float
# Serializes as an array: [timestamp, height, difficulty, adjustment ratio].
DifficultyAdjustmentEntry = List[float]
# Proof-of-work difficulty.
Difficulty = float
# A duration in days.
Days = float
# Index of a 3-day period.
Day3 = int
# Index of a day.
Day1 = int
# Calendar date in YYYY-MM-DD format.
Date = str
# A quote currency prices are served in: USD only.
Currency = Literal["USD"]
# Position of a transaction inside a `CpfpCluster.txs` array. Cluster-local,
# has no meaning outside the enclosing cluster.
CpfpClusterTxIndex = int
# A change in a number of things.
CountSigned = int
# A mean number of things.
CountFract = float
# A number of things.
Count = int
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
# Bitcoin multiplied by the days it was held.
CoinDays = float
# Bitcoin multiplied by the blocks it was held.
CoinBlocks = float
# URPD cohort identifier. Use `GET /api/urpd` to list available cohorts.
#
# Names are non-empty ASCII `[a-z0-9_]+`. Availability is determined by
# supported age filters and the published UTXO set.
Cohort = str
# A mean size in bytes.
BytesFract = float
# A size in bytes.
Bytes = int
# Yes or no.
Boolean = bool
# Position of a transaction within a single block (0 = coinbase).
# Distinct from `TxIndex`, which is the chain-wide global tx index.
BlockTxIndex = int
# Double-SHA256 block-header hash, serialized in Bitcoin's conventional
# hexadecimal byte order.
BlockHash = str
# Bitcoin amount as floating point (1 BTC = 100,000,000 satoshis)
Bitcoin = float
# Bitcoin address, or the hex public key of a P2PK output.
Address = str
# One slot of the new template in a `BlockTemplateDiff`.
#
# Untagged on the wire so JSON type disambiguates the variants:
# - `Retained(idx)` serializes as a bare integer - index into the
#   transactions of the prior template (which the client cached at
#   `since`).
# - `New(tx)` serializes as a transaction object - a body that was
#   new or changed since the prior template and must replace this position.
BlockTemplateDiffEntry = Union[int, "Transaction"]
class ValidateAddressParam(TypedDict):
    """
    Attributes:
        address: Bitcoin address to validate (can be any string)
    """
    address: str

class UrpdQuery(TypedDict, total=False):
    """
    Attributes:
        agg: Aggregation strategy. Default: raw (no aggregation). Accepts `bucket` as alias.
        weight: Supply weighting. Default: none (unweighted).
    """
    agg: UrpdAggregation
    weight: UrpdWeight

# Transaction IDs as repeated `txId[]` query parameters.
TxidsParam = TypedDict("TxidsParam", {"txId[]": List[Txid]})

class TxidVout(TypedDict):
    """
    Attributes:
        vout: Output index
    """
    txid: Txid
    vout: Vout

class TxidParam(TypedDict):
    txid: Txid

class TxIndexParam(TypedDict):
    """
    Attributes:
        index: Global transaction index across the chain
    """
    index: TxIndex

class OutPoint(TypedDict):
    """
    The output a transaction input spends: its transaction index and output position, written as
    `{"tx_index": N, "vout": M}` (coinbase inputs: `{"tx_index": 4294967295, "vout": 4294967295}`).
    """
    tx_index: TxIndex
    vout: Vout

class TimestampParam(TypedDict):
    timestamp: Timestamp

class PoolHashrateEntry(TypedDict):
    """
    Attributes:
        timestamp: Unix timestamp
        avgHashrate: Average hashrate (H/s)
        share: Pool's share of total network hashrate (0.0 - 1.0)
    """
    timestamp: Timestamp
    avgHashrate: int
    share: float
    poolName: str

class HashrateEntry(TypedDict):
    """
    Attributes:
        timestamp: End of the day the hashrate covers (Unix timestamp), as mempool.space stamps it
        avgHashrate: Average hashrate (H/s)
    """
    timestamp: Timestamp
    avgHashrate: int

class TimePeriodParam(TypedDict):
    """
    Attributes:
        time_period: Lookback period: `24h`, `3d`, `1w`, `1m`, `3m`, `6m`, `1y`, `2y`, `3y` or `all`.
    """
    time_period: TimePeriod

class SeriesParam(TypedDict):
    series: SeriesName

class SeriesCount(TypedDict):
    """
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
    """
    distinct: int
    total: int
    lazy: int
    stored: int
    by_db: dict[str, SeriesCount]

class MempoolRecentTx(TypedDict):
    """
    Simplified mempool transaction for the `/api/mempool/recent` endpoint.

    Attributes:
        fee: Transaction fee (sats)
        vsize: Virtual size (vbytes)
        value: Total output value (sats)
    """
    txid: Txid
    fee: Sats
    vsize: VSize
    value: Sats

class MempoolInfo(TypedDict):
    """
    Mempool statistics with incrementally maintained fee histogram.

    Attributes:
        count: Number of transactions in the mempool
        vsize: Total virtual size of all transactions in the mempool (vbytes)
        total_fee: Total fees of all transactions in the mempool (satoshis)
        fee_histogram: Fee histogram as Esplora bins it: `[[fee_rate, vsize], ...]` by descending fee rate, each entry the vsize of the transactions paying from its fee rate up to the previous entry's, a bin closing at the first fee rate change past 50,000 vB
    """
    count: int
    vsize: VSize
    total_fee: Sats
    fee_histogram: List[List[float]]

class CpfpEntry(TypedDict):
    """
    A transaction in a CPFP relationship.
    """
    txid: Txid
    weight: Weight
    fee: Sats

class AddressMempoolStats(TypedDict):
    """
    Address statistics in the mempool (unconfirmed transactions only), in mempool.space's format.

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

class RecommendedFees(TypedDict):
    """
    Recommended fee rates in whole sat/vB (`/api/v1/fees/recommended`)

    Attributes:
        fastestFee: Fee rate for fastest confirmation (next block)
        halfHourFee: Fee rate for confirmation within ~30 minutes (second projected block)
        hourFee: Fee rate for confirmation within ~1 hour (third projected block)
        economyFee: Smoothed, fullness-adjusted third-block fee estimate, bounded by `minimumFee` and twice it; `minimumFee` when no third projected block exists
        minimumFee: The node's current mempool minimum fee rate, rounded up to a whole sat/vB (at least 1)
    """
    fastestFee: int
    halfHourFee: int
    hourFee: int
    economyFee: int
    minimumFee: int

class PoolStats(TypedDict):
    """
    Mining pool with block statistics for a time period

    Attributes:
        poolId: One-based pool ID, matching mempool.space's `id`
        link: Pool website URL
        rank: Pool ranking by block count (1 = most blocks)
        emptyBlocks: Number of empty (coinbase-only) blocks mined in the time period
        slug: URL-friendly pool identifier
        share: Pool's share of total blocks (0.0 - 1.0)
        poolUniqueId: Zero-based pool ID, matching mempool.space's `unique_id`
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

class PoolSlugParam(TypedDict):
    slug: PoolSlug

class PoolInfo(TypedDict):
    """
    Attributes:
        slug: URL-friendly pool identifier
        unique_id: Zero-based pool ID, matching mempool.space's `unique_id`
    """
    name: str
    slug: PoolSlug
    unique_id: int

class PoolDetailInfo(TypedDict):
    """
    Attributes:
        id: One-based pool ID, matching mempool.space's `id`
        link: Pool website URL
        addresses: Known payout addresses
        regexes: Coinbase tags, matched as case-insensitive substrings
        slug: URL-friendly pool identifier
        unique_id: Zero-based pool ID, matching mempool.space's `unique_id`
    """
    id: int
    name: str
    link: str
    addresses: List[str]
    regexes: List[str]
    slug: PoolSlug
    unique_id: int

class _BlockPoolRequired(TypedDict):
    id: int
    name: str
    slug: PoolSlug
    blockNumber: int

class BlockPool(_BlockPoolRequired, total=False):
    """
    Attributes:
        id: Zero-based pool ID, matching mempool.space's `unique_id`
        slug: URL-friendly pool identifier
        blockNumber: This block's ordinal among blocks attributed to this pool
        minerNames: Miner name tags found in coinbase scriptsig
    """
    minerNames: Optional[List[str]]

# Pool's share of total blocks for different time periods
PoolBlockShares = TypedDict("PoolBlockShares", {"all": float, "24h": float, "1w": float})

# Block counts for different time periods
PoolBlockCounts = TypedDict("PoolBlockCounts", {"all": int, "24h": int, "1w": int})

class _PoolDetailRequired(TypedDict):
    pool: PoolDetailInfo
    blockCount: PoolBlockCounts
    blockShare: PoolBlockShares
    estimatedHashrate: int
    totalReward: Sats

class PoolDetail(_PoolDetailRequired, total=False):
    """
    Attributes:
        estimatedHashrate: Estimated hashrate based on blocks mined (H/s)
        reportedHashrate: Self-reported hashrate (if available, H/s)
        totalReward: Total reward earned by this pool: coinbase output value of every block it mined (sats)
    """
    reportedHashrate: Optional[int]

class Pagination(TypedDict, total=False):
    """
    Attributes:
        page: Zero-based page number (default: 0)
        per_page: Results per page (default: 1000, max: 1000)
    """
    page: Optional[int]
    per_page: Optional[int]

class PaginatedSeries(TypedDict):
    """
    A paginated list of available series names (up to 1,000 per page)

    Attributes:
        current_page: Current page number (0-indexed)
        max_page: Maximum valid page index (0-indexed)
    """
    current_page: int
    max_page: int
    total_count: int
    per_page: int
    has_more: bool
    series: List[str]

class AddressHashPrefixParam(TypedDict):
    """
    Attributes:
        prefix: First 1–16 hexadecimal nibbles of the RapidHash v3 hash over the raw address payload bytes.
    """
    address_type: OutputType
    prefix: str

class NextBlockHashParam(TypedDict):
    """
    Prior-template hash for `GET /api/v1/mempool/block-template/diff/{hash}`.
    """
    hash: NextBlockHash

class _SearchQueryRequired(TypedDict):
    q: SeriesName

class SearchQuery(_SearchQueryRequired, total=False):
    """
    Attributes:
        q: Search query string
        limit: Maximum number of results, 100 when omitted
    """
    limit: Limit

class SeriesNameWithIndex(TypedDict):
    series: SeriesName
    index: Index

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
        kind: The Rust type (e.g., "Sats", "Ratio").
        nullable: Indexes whose values can be null: a missing value (e.g. a period without blocks) or an undefined one (e.g. NaN).
        description: Human-readable metric definition, when documented.
        type: JSON Schema type (e.g., "integer", "number", "string", "boolean", "array", "object").
    """
    description: Optional[str]

class _SeriesInfoRequired(TypedDict):
    indexes: List[Index]
    nullable: List[Index]
    type: str

class SeriesInfo(_SeriesInfoRequired, total=False):
    """
    Attributes:
        description: Human-readable metric definition, when documented
        nullable: Indexes whose values can be null: a missing value (e.g. a period without blocks) or an undefined one (e.g. NaN)
        type: Value type (e.g. "Ratio", "Sats", "Cents")
        unit: What the value type measures, when documented (e.g. "A duration in days.")
    """
    description: Optional[str]
    unit: Optional[str]

class IndexInfo(TypedDict):
    """
    Attributes:
        index: The canonical index name
        aliases: Every name REST queries accept for it; MCP tools take the canonical `index` only
    """
    index: Index
    aliases: List[str]

class HeightOrDateParam(TypedDict):
    """
    Attributes:
        point: Confirmed block height as decimal digits (`840000`) or calendar date in `YYYY-MM-DD` format.
    """
    point: str

class SyncStatus(TypedDict):
    """
    Attributes:
        indexed_height: Height of the last indexed block
        computed_height: Height of the last computed block (series)
        tip_height: Height of the Bitcoin node's chain tip; `/health` reports `indexed_height` instead, without asking the node
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

class RewardStats(TypedDict):
    """
    Block reward statistics over a range of blocks

    Attributes:
        startBlock: First block in the range
        endBlock: Last block in the range
        totalReward: Total coinbase rewards (subsidy + fees) in sats
        totalFee: Total transaction fees in sats
    """
    startBlock: Height
    endBlock: Height
    totalReward: str
    totalFee: str
    totalTx: str

class PoolSlugAndHeightParam(TypedDict):
    slug: PoolSlug
    height: Height

class MerkleProof(TypedDict):
    """
    Merkle inclusion proof for a transaction

    Attributes:
        merkle: Merkle proof path (hex-encoded hashes)
        pos: Transaction position in the block (0-indexed)
    """
    block_height: Height
    merkle: List[str]
    pos: int

class HeightParam(TypedDict):
    height: Height

class Health(TypedDict):
    """
    Attributes:
        status: Health status ("healthy")
        timestamp: Current server time (ISO 8601)
        started_at: Server start time (ISO 8601)
        uptime_seconds: Uptime in seconds
        indexed_height: Height of the last indexed block
        computed_height: Height of the last computed block (series)
        tip_height: Height of the Bitcoin node's chain tip; `/health` reports `indexed_height` instead, without asking the node
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

class DifficultyEntry(TypedDict):
    """
    A single difficulty data point in the hashrate summary.

    Attributes:
        time: Unix timestamp of the difficulty adjustment
        adjustment: Adjustment ratio (new/previous, e.g. 1.068 = +6.8%)
    """
    time: Timestamp
    height: Height
    difficulty: float
    adjustment: float

class HashrateSummary(TypedDict):
    """
    Attributes:
        currentHashrate: Current network hashrate over the trailing 7 days (H/s)
    """
    hashrates: List[HashrateEntry]
    difficulty: List[DifficultyEntry]
    currentHashrate: int
    currentDifficulty: float

class DifficultyAdjustment(TypedDict):
    """
    Attributes:
        progressPercent: Progress through current difficulty epoch (0-100%)
        difficultyChange: Estimated difficulty change at next retarget at the `adjustedTimeAvg` pace (%)
        estimatedRetargetDate: Estimated timestamp of next retarget (milliseconds)
        remainingBlocks: Blocks remaining until retarget
        remainingTime: Estimated time until retarget at `adjustedTimeAvg` (milliseconds)
        previousRetarget: Previous difficulty adjustment (%)
        previousTime: Timestamp of most recent retarget (seconds)
        timeAvg: Average block time in current epoch (milliseconds)
        adjustedTimeAvg: Block time the projection uses (milliseconds): `timeAvg`, except in an epoch's first 503 blocks, where it spans the last 503 blocks, those before the retarget scaled by it
        timeOffset: Testnet's offset for its 20-minute minimum-difficulty rule (milliseconds): always 0 on mainnet, as mempool.space
        expectedBlocks: Blocks expected at the 10-minute target between the epoch's first block and the tip block
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

class BlockWeightEntry(TypedDict):
    """
    One time bucket of blocks.

    Attributes:
        avgHeight: Average block height in this bucket
        timestamp: Average block timestamp in this bucket
        avgWeight: Mean block weight in this bucket, rounded half up (weight units)
    """
    avgHeight: Height
    timestamp: Timestamp
    avgWeight: Weight

class BlockSizeEntry(TypedDict):
    """
    One time bucket of blocks.

    Attributes:
        avgHeight: Average block height in this bucket
        timestamp: Average block timestamp in this bucket
        avgSize: Mean block size in this bucket, rounded half up (bytes)
    """
    avgHeight: Height
    timestamp: Timestamp
    avgSize: int

class BlockSizesWeights(TypedDict):
    sizes: List[BlockSizeEntry]
    weights: List[BlockWeightEntry]

class _SeriesSelectionRequired(TypedDict):
    series: SeriesList
    index: Index

class SeriesSelection(_SeriesSelectionRequired, total=False):
    """
    Attributes:
        start: Inclusive start: integer index, date (YYYY-MM-DD), or timestamp (ISO 8601). Negative integers count from end. REST aliases: `from`, `f`, `s`
        end: Exclusive end: integer index, date (YYYY-MM-DD), or timestamp (ISO 8601). Negative integers count from end. REST aliases: `to`, `t`, `e`
        limit: Maximum number of values to return (ignored if `end` is set); omitted, the rest of the range. REST aliases: `count`, `c`, `l`
    """
    start: Union[RangeIndex, None]
    end: Union[RangeIndex, None]
    limit: Union[Limit, None]
    format: Format

class DataRangeFormat(TypedDict, total=False):
    """
    Attributes:
        start: Inclusive start: integer index, date (YYYY-MM-DD), or timestamp (ISO 8601). Negative integers count from end. REST aliases: `from`, `f`, `s`
        end: Exclusive end: integer index, date (YYYY-MM-DD), or timestamp (ISO 8601). Negative integers count from end. REST aliases: `to`, `t`, `e`
        limit: Maximum number of values to return (ignored if `end` is set); omitted, the rest of the range. REST aliases: `count`, `c`, `l`
    """
    start: Union[RangeIndex, None]
    end: Union[RangeIndex, None]
    limit: Union[Limit, None]
    format: Format

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

class PreciseRecommendedFees(TypedDict):
    """
    Recommended fee rates in sat/vB, to the thousandth (`/api/v1/fees/precise`)

    Attributes:
        fastestFee: Fee rate for fastest confirmation (next block)
        halfHourFee: Fee rate for confirmation within ~30 minutes (second projected block)
        hourFee: Fee rate for confirmation within ~1 hour (third projected block)
        economyFee: Smoothed, fullness-adjusted third-block fee estimate, bounded by `minimumFee` and twice it; `minimumFee` when no third projected block exists
        minimumFee: The node's current mempool minimum fee rate, rounded up to 0.001 sat/vB (at least 0.001)
    """
    fastestFee: FeeRate
    halfHourFee: FeeRate
    hourFee: FeeRate
    economyFee: FeeRate
    minimumFee: FeeRate

class MempoolBlock(TypedDict):
    """
    Block info in a mempool.space like format for fee estimation.

    Attributes:
        blockSize: Sum of the transactions' serialized sizes in bytes (witness + non-witness), without header or coinbase.
        blockVSize: Sum of the transactions' virtual sizes in vbytes
        nTx: Number of transactions in the projected block
        totalFees: Total fees in satoshis
        medianFee: Median fee rate in sat/vB
        feeRange: Vsize-weighted percentiles of the transactions' CPFP chunk fee rates, in sat/vB: [0, 10, 25, 50, 75, 90, 100] for the first block, [5, 10, 25, 50, 75, 90, 95] for the later projected ones
    """
    blockSize: int
    blockVSize: float
    nTx: int
    totalFees: Sats
    medianFee: FeeRate
    feeRange: List[FeeRate]

class BlockFeeRatesEntry(TypedDict):
    """
    One time bucket of blocks. Each `avgFee_*` field is the
    bucket's mean of its blocks' own percentile of their non-coinbase transactions' effective fee
    rates, weighted by vsize (a block without such transactions counts as 0).

    Attributes:
        avgHeight: Average block height in this bucket
        timestamp: Average block timestamp in this bucket
        avgFee_0: Mean of the blocks' minimum fee rate (sat/vB)
        avgFee_10: Mean of the blocks' 10th percentile fee rate (sat/vB)
        avgFee_25: Mean of the blocks' 25th percentile fee rate (sat/vB)
        avgFee_50: Mean of the blocks' median fee rate (sat/vB)
        avgFee_75: Mean of the blocks' 75th percentile fee rate (sat/vB)
        avgFee_90: Mean of the blocks' 90th percentile fee rate (sat/vB)
        avgFee_100: Mean of the blocks' maximum fee rate (sat/vB)
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

class PricePoint(TypedDict):
    """
    A BTC/USD price at a time: the current price (`/api/v1/prices`) or one historical close

    Attributes:
        time: Unix timestamp
        USD: BTC/USD price
    """
    time: Timestamp
    USD: Dollars

class HistoricalPrice(TypedDict):
    """
    Attributes:
        exchangeRates: Exchange rates (currently empty)
    """
    prices: List[PricePoint]
    exchangeRates: ExchangeRates

class BlockRewardsEntry(TypedDict):
    """
    One time bucket of blocks.

    Attributes:
        avgHeight: Average block height in this bucket
        timestamp: Average block timestamp in this bucket
        avgRewards: Mean coinbase reward (subsidy + fees) per block in this bucket, rounded half up (sats)
        USD: Mean BTC/USD spot price over the bucket's blocks (the price, not the rewards in USD)
    """
    avgHeight: Height
    timestamp: Timestamp
    avgRewards: Sats
    USD: Dollars

class BlockFeesEntry(TypedDict):
    """
    One time bucket of blocks.

    Attributes:
        avgHeight: Average block height in this bucket
        timestamp: Average block timestamp in this bucket
        avgFees: Mean fees per block in this bucket, rounded half up (sats)
        USD: Mean BTC/USD spot price over the bucket's blocks (the price, not the fees in USD)
    """
    avgHeight: Height
    timestamp: Timestamp
    avgFees: Sats
    USD: Dollars

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
        avgFee: Mean fee per non-coinbase transaction in satoshis, rounded down (0 without any)
        avgFeeRate: Average fee rate in sat/vB
        coinbaseRaw: Raw coinbase transaction scriptsig as hex
        coinbaseAddress: Primary coinbase output address
        coinbaseAddresses: All coinbase output addresses
        coinbaseSignature: Coinbase output script in ASM format
        coinbaseSignatureAscii: Coinbase scriptsig with each byte read as the Unicode code point of the same value
        avgTxSize: Non-coinbase transaction bytes over the transaction count (coinbase included), rounded to 2 decimals
        totalInputs: Total number of inputs (excluding coinbase)
        totalOutputAmt: Total output amount in satoshis, coinbase excluded
        medianFeeAmt: Median fee amount in satoshis
        feePercentiles: Fee amount percentiles in satoshis: [min, 10%, 25%, 50%, 75%, 90%, max]
        segwitTotalSize: Total size of segwit transactions in bytes
        header: Raw 80-byte block header as hex
        utxoSetChange: UTXO set change as mempool.space and bitcoin-cli report it: total outputs minus total inputs, unspendable outputs (OP_RETURN) included, unlike `utxoSetSize`
        utxoSetSize: Total spendable UTXO set size at this height (excludes OP_RETURN and other unspendable outputs)
        totalInputAmt: Total input amount in satoshis
        virtualSize: Virtual size in vbytes: weight / 4, unrounded
        firstSeen: Timestamp when the block was first seen (always null, not yet supported)
        orphans: Orphaned blocks (always empty)
        price: USD price at block height
    """
    coinbaseAddress: Optional[str]
    firstSeen: Optional[int]

class _AddressChainStatsRequired(TypedDict):
    balance: Sats
    funded_txo_count: int
    funded_txo_sum: Sats
    spent_txo_count: int
    spent_txo_sum: Sats
    tx_count: int
    realized_price: Dollars

class AddressChainStats(_AddressChainStatsRequired, total=False):
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
        type_index: Index of this address within its type on the blockchain, null for an address without confirmed activity
        realized_price: Realized price (average cost basis) in USD
    """
    type_index: Union[TypeIndex, None]

class DiskUsage(TypedDict):
    """
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

class HistoricalPriceParam(TypedDict, total=False):
    """
    Historical price query parameters, as mempool.space's

    Attributes:
        timestamp: UNIX timestamp: the latest close at or before it
        currency: Quote currency: USD, the only one served
    """
    timestamp: Union[Timestamp, None]
    currency: Union[Currency, None]

class CpfpClusterTx(TypedDict):
    """
    Attributes:
        parents: In-cluster parents of this tx.
    """
    txid: Txid
    weight: Weight
    fee: Sats
    parents: List[CpfpClusterTxIndex]

class CpfpClusterChunk(TypedDict):
    """
    One linearized chunk inside a `CpfpCluster`. `txs` follows `CpfpCluster.txs` order (topological);
    `feerate` is the chunk's, shared by all its transactions.
    """
    txs: List[CpfpClusterTxIndex]
    feerate: FeeRate

class CpfpCluster(TypedDict):
    """
    CPFP cluster: the connected component the seed belongs to, plus its
    linearization.

    Attributes:
        txs: All txs in the cluster, in topological order (parents before children).
        chunks: Chunks ordered by descending feerate.
        chunkIndex: Index into `chunks` of the chunk containing the seed tx.
    """
    txs: List[CpfpClusterTx]
    chunks: List[CpfpClusterChunk]
    chunkIndex: int

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
        bestDescendant: Best (highest fee rate) descendant, if any.
        effectiveFeePerVsize: Effective fee rate considering CPFP relationships (sat/vB). This is the seed's chunk feerate after lift-merging, i.e. the rate Core/mempool.space would surface for this tx.
        sigops: BIP-141 sigop cost for the seed tx (witness sigops count as 1, legacy and P2SH-redeem sigops count as 4).
        fee: Transaction fee (sats).
        vsize: Virtual size of the seed tx (vbytes).
        adjustedVsize: Policy-adjusted virtual size: `max(vsize, sigops * 5)`.
        cluster: Cluster the seed belongs to; omitted when the seed has no ancestors and no descendants (matches mempool.space).
    """
    bestDescendant: Union[CpfpEntry, None]
    cluster: Union[CpfpCluster, None]

class UrpdParams(TypedDict):
    """
    A URPD cohort and exact block height or UTC calendar-day alias.
    """
    cohort: Cohort
    point: str

class UrpdCohortParam(TypedDict):
    cohort: Cohort

class _TxStatusRequired(TypedDict):
    confirmed: bool

class TxStatus(_TxStatusRequired, total=False):
    """
    Attributes:
        block_height: Block height (only present if confirmed)
        block_hash: Block hash (only present if confirmed)
        block_time: Block timestamp (only present if confirmed)
    """
    block_height: Union[Height, None]
    block_hash: Union[BlockHash, None]
    block_time: Union[Timestamp, None]

class Utxo(TypedDict):
    """
    Unspent transaction output

    Attributes:
        value: Output value in satoshis
    """
    txid: Txid
    vout: Vout
    status: TxStatus
    value: Sats

class _TxOutspendRequired(TypedDict):
    spent: bool

class TxOutspend(_TxOutspendRequired, total=False):
    """
    Attributes:
        txid: Transaction ID of the spending transaction (only present if spent)
        vin: Input index in the spending transaction (only present if spent)
        status: Status of the spending transaction (only present if spent)
    """
    txid: Union[Txid, None]
    vin: Union[Vin, None]
    status: Union[TxStatus, None]

class BlockTimestamp(TypedDict):
    """
    Block information returned for timestamp queries

    Attributes:
        timestamp: Block timestamp in ISO 8601 format
    """
    height: Height
    hash: BlockHash
    timestamp: str

class _BlockStatusRequired(TypedDict):
    in_best_chain: bool

class BlockStatus(_BlockStatusRequired, total=False):
    """
    Attributes:
        height: Block height (only if in best chain)
        next_best: Hash of the next block in the best chain (null if tip)
    """
    height: Union[Height, None]
    next_best: Union[BlockHash, None]

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
        timestamp: Block timestamp (Unix time)
        bits: Compact target (bits)
        size: Block size in bytes
        weight: Block weight in weight units
        mediantime: Median time of the last 11 blocks
        stale: Whether this block has been replaced by a longer chain
    """
    stale: bool

class BlockInfo(TypedDict):
    """
    Block information matching mempool.space /api/block/{hash}

    Attributes:
        id: Block hash
        timestamp: Block timestamp (Unix time)
        bits: Compact target (bits)
        size: Block size in bytes
        weight: Block weight in weight units
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

class BlockHashTxIndex(TypedDict):
    """
    Attributes:
        index: Transaction index within the block (0-based)
    """
    hash: BlockHash
    index: BlockTxIndex

class BlockHashStartIndex(TypedDict):
    """
    Attributes:
        start_index: Starting transaction index within the block (0-based)
    """
    hash: BlockHash
    start_index: BlockTxIndex

class BlockHashParam(TypedDict):
    hash: BlockHash

class BlockCountParam(TypedDict):
    """
    Attributes:
        block_count: Number of recent blocks to include
    """
    block_count: int

class UtxoOrigins(TypedDict):
    """
    Columnar unspent amounts indexed by creation height.

    Attributes:
        count: Number of unspent outputs per creation height.
        supply: BTC held by the unspent outputs per creation height.
    """
    count: List[int]
    supply: List[Bitcoin]

class UtxoSet(TypedDict):
    """
    The UTXO set after a block, grouped by the height of the block that created each output.
    
    The set excludes `OP_RETURN` outputs and the genesis coinbase, which are unspendable, and the
    coinbases of blocks 91812 and 91722 once duplicates at 91842 and 91880 overwrite them; it
    includes zero-value outputs. The set after block `first - 1` plus a diff's `created` minus its
    `spent` gives the set after block `last`; rounding each supply to 8 decimals keeps it exact.

    Attributes:
        date: UTC date of the represented block's monotonic timestamp (the running maximum of block times).
        count: Number of unspent outputs.
        supply: BTC held by the unspent outputs.
        origins: Unspent outputs by creation height: entry `i` is what remains of block `i`'s outputs.
    """
    height: Height
    hash: BlockHash
    date: Date
    count: int
    supply: Bitcoin
    origins: UtxoOrigins

class UtxoChanges(TypedDict):
    """
    Columnar counts and supplies by creation height.

    Attributes:
        height: Creation height of each row.
        count: Number of outputs in each row.
        supply: BTC held by the outputs in each row.
    """
    height: List[Height]
    count: List[int]
    supply: List[Bitcoin]

class UtxoSetDiff(TypedDict):
    """
    What blocks `first` through `last` changed in the UTXO set.

    Attributes:
        first: First block of the diff; it applies to the set after block `first - 1` (the empty set for block 0).
        last: Last block of the diff, described by `hash` and `date`.
        hash: Hash of block `last`.
        date: UTC date of block `last`'s monotonic timestamp.
        created: Outputs created, one row per block, including those spent again within the range (which `spent` lists too).
        spent: Outputs removed, one row per creation height in ascending order: spent outputs, plus the coinbase outputs of blocks 91812 and 91722, which duplicates at 91842 and 91880 overwrote.
    """
    first: Height
    last: Height
    hash: BlockHash
    date: Date
    created: UtxoChanges
    spent: UtxoChanges

class UrpdBuckets(TypedDict):
    """
    Columnar buckets in ascending price order: entry `i` of each column is bucket `i`.

    Attributes:
        price: Each bucket's price, in USD: its lower bound when aggregated, and for `raw` the price its coins' creation prices round to (four significant digits of the price in cents).
        supply: Supply held with a last-move price inside each bucket, in BTC. With a `wakefulness` or `mobility` weight, the weighted supply, floored to whole satoshis at each grid price.
        realized_cap: Realized cap contribution in USD: `price * supply` summed over the coins in each bucket, each coin's creation price taken on the price grid (the `raw` bucket price, within 0.05% of the exact price), rounded to the cent: a bucket worth under half a cent shows 0. The buckets therefore need not sum to the cohort's realized cap series.
        unrealized_pnl: Unrealized P&L in USD at the represented block: `close * supply` minus each bucket's grid capital, rounded to the cent once. Can be negative.
    """
    price: List[Dollars]
    supply: List[Bitcoin]
    realized_cap: List[Dollars]
    unrealized_pnl: List[Dollars]

class Urpd(TypedDict):
    """
    UTXO Realized Price Distribution for a cohort at a specific block.
    
    Supply is grouped by the price at the block in which each UTXO was last moved; each bucket
    holds its supply, realized cap and unrealized P&L. The same block header as `UtxoSet`
    (`height`, `hash`, `date`), then its own fields and its buckets as an object of columns.

    Attributes:
        height: Exact published block represented by this distribution.
        hash: Hash of block `height`.
        date: UTC date of the represented block's monotonic timestamp.
        close: Price at `height`, in USD. Anchor for `unrealized_pnl`.
        supply: Sum of `buckets.supply`, in BTC.
    """
    height: Height
    hash: BlockHash
    date: Date
    cohort: Cohort
    weight: UrpdWeight
    aggregation: UrpdAggregation
    close: Dollars
    supply: Bitcoin
    buckets: UrpdBuckets

class _AddressValidationRequired(TypedDict):
    isvalid: bool

class AddressValidation(_AddressValidationRequired, total=False):
    """
    Attributes:
        address: The validated address
        scriptPubKey: The scriptPubKey in hex
        isscript: Whether the address is P2SH, P2WSH or P2TR
        witness_version: Witness version (0 for P2WPKH/P2WSH, 1 for P2TR and P2A)
        witness_program: Witness program in hex
        error_locations: Error locations: an empty array for invalid addresses, omitted for valid ones
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
        scriptpubkey_address: Bitcoin address, omitted for scripts without an address (P2PK included, as Esplora).
        value: Value of the output in satoshis.
    """
    scriptpubkey_address: Address

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
    Attributes:
        txid: Transaction ID of the output being spent
        vout: Output index being spent (coinbase: 4294967295)
        prevout: The output being spent
        scriptsig: Signature script in hex (empty for native SegWit inputs)
        scriptsig_asm: Signature script in assembly format
        witness: Witness data (stack items, present for SegWit inputs; hex-encoded on the wire)
        inner_redeemscript_asm: Inner redeemscript in assembly (P2SH inputs)
        inner_witnessscript_asm: Inner witnessscript in assembly (P2WSH and P2SH-P2WSH: the witness script; P2TR script path: the leaf script)
    """
    witness: List[str]
    inner_redeemscript_asm: str
    inner_witnessscript_asm: str

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
        version: Transaction version (raw i32 from Bitcoin protocol, may contain non-standard values in coinbase txs)
        vin: Transaction inputs
        vout: Transaction outputs
        size: Transaction size in bytes
        sigops: BIP-141 signature-operation cost
        fee: Transaction fee in satoshis
    """
    index: Union[TxIndex, None]

class BlockTemplateDiff(TypedDict):
    """
    Delta between the current `getblocktemplate` projection and a prior
    one identified by `since`. Returned by
    `GET /api/v1/mempool/block-template/diff/{hash}`.
    
    `removed` lists txids no longer present. A changed body can be emitted as
    `New` without removing its txid; absence of a retained index alone does not
    imply removal.

    Attributes:
        hash: Current next-block hash. Use as `since` on the next diff call.
        since: Echoed prior hash the diff was computed against.
        stats: Stats of the new template; they can change while every transaction is retained.
        order: New template in order. Each entry is either an index into the prior template's transactions or a full transaction body.
        removed: Txids that left the projected next block since `since` (confirmed, evicted, replaced, or pushed past block 0).
    """
    hash: NextBlockHash
    since: NextBlockHash
    stats: MempoolBlock
    order: List[BlockTemplateDiffEntry]
    removed: List[Txid]

class BlockTemplate(TypedDict):
    """
    Projected next-block contents from Bitcoin Core's `getblocktemplate`
    (block 0 of the snapshot). Returned by
    `GET /api/v1/mempool/block-template`.

    Attributes:
        hash: Pass to `GET /api/v1/mempool/block-template/diff/{hash}` to fetch deltas.
        transactions: Full transaction bodies in `getblocktemplate` order.
    """
    hash: NextBlockHash
    stats: MempoolBlock
    transactions: List[Transaction]

class AddressStats(TypedDict):
    """
    Address information compatible with mempool.space API format.

    Attributes:
        address: Bitcoin address, or the hex public key of a P2PK output
        address_type: Output script type
        chain_stats: Statistics for confirmed transactions on the blockchain
        mempool_stats: Statistics for unconfirmed transactions in the mempool
        balance: Total current balance in satoshis, including pending (unconfirmed) mempool changes
    """
    address: Address
    address_type: OutputType
    chain_stats: AddressChainStats
    mempool_stats: AddressMempoolStats
    balance: Sats

class AddressParam(TypedDict):
    address: Address

class AddressHashPrefixMatches(TypedDict):
    address_type: OutputType
    prefix: str
    truncated: bool
    addresses: List[Address]

class AddressAfterTxidParam(TypedDict):
    """
    Attributes:
        after_txid: Last txid from the previous page (return transactions strictly older than this)
    """
    address: Address
    after_txid: Txid


# Each date-based index's buckets as (unit, size, phase): bucket i starts i * size - phase units
# after 2009-01-01 UTC, except that a month calendar's bucket 0 starts at January 2009.
_CALENDARS: Dict[str, Tuple[str, int, int]] = {
    'minute10': ('second', 600, 0),
    'minute30': ('second', 1800, 0),
    'hour1': ('second', 3600, 0),
    'hour4': ('second', 14400, 0),
    'hour12': ('second', 43200, 0),
    'day1': ('day', 1, 0),
    'day3': ('day', 3, 1),
    'week1': ('day', 7, 3),
    'month1': ('month', 1, 0),
    'month3': ('month', 3, 0),
    'month6': ('month', 6, 0),
    'year1': ('month', 12, 0),
    'year10': ('month', 120, 108),
}

# Each address type's payload length in bytes.
_ADDRESS_PAYLOAD_LENGTHS: Dict[str, int] = {
    'p2pk65': 65,
    'p2pk33': 33,
    'p2pkh': 20,
    'p2sh': 20,
    'p2wpkh': 20,
    'p2wsh': 32,
    'p2tr': 32,
    'p2a': 2,
}


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


def _segment(value: Any) -> str:
    """A path segment, percent-encoded so `/`, `?` or `#` in a value stay inside it."""
    return quote(str(value), safe='')


class BitviewClientBase:
    def __init__(self, base_url: str, timeout: float = 30.0):
        parsed = urlparse(base_url)
        self._host = parsed.netloc
        self._prefix = parsed.path.rstrip('/')
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

    def _request(self, method: str, path: str, body: Optional[str] = None) -> bytes:
        """A failed exchange closes the connection, which `http.client` would otherwise leave
        mid-request for every later call."""
        try:
            conn = self._connect()
            conn.request(method, self._prefix + path, body=body)
            res = conn.getresponse()
            data = res.read()
        except (HTTPException, OSError) as e:
            self.close()
            raise BitviewError(str(e))
        if res.status >= 400:
            raise _response_error(res.status, data)
        return data

    def get(self, path: str) -> bytes:
        return self._request("GET", path)

    def get_json(self, path: str) -> Any:
        return json.loads(self.get(path))

    def get_text(self, path: str) -> str:
        return self.get(path).decode()

    def post(self, path: str, body: str) -> bytes:
        return self._request("POST", path, body)

    def post_json(self, path: str, body: str) -> Any:
        return json.loads(self.post(path, body))

    def post_text(self, path: str, body: str) -> str:
        return self.post(path, body).decode()

    def close(self) -> None:
        if self._conn:
            self._conn.close()
            self._conn = None

    def __enter__(self) -> BitviewClientBase:
        return self

    def __exit__(self, exc_type: Optional[type], exc_val: Optional[BaseException], exc_tb: Optional[Any]) -> None:
        self.close()


_MASK_64 = (1 << 64) - 1
# RapidHash v3's secrets 1 and 2: the only ones payloads of at most 65 bytes reach.
_RAPIDHASH_SECRET_1 = 0x8bb84b93962eacc9
_RAPIDHASH_SECRET_2 = 0x4b33a62ed433d4a3


def _u64(value: int) -> int:
    return value & _MASK_64


def _rapid_mix(left: int, right: int) -> int:
    result = _u64(left) * _u64(right)
    return _u64(result) ^ _u64(result >> 64)


def _rapid_mum(left: int, right: int) -> Tuple[int, int]:
    result = _u64(left) * _u64(right)
    return _u64(result), _u64(result >> 64)


def _rapid_hash_seed(seed: int) -> int:
    return _u64(seed ^ _rapid_mix(seed ^ _RAPIDHASH_SECRET_2, _RAPIDHASH_SECRET_1))


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
        else:
            a ^= (data[0] << 45) | data[length - 1]
            b ^= data[length >> 1]
        remainder = length
    else:
        seed = _rapid_mix(_read_u64(data, 0) ^ _RAPIDHASH_SECRET_2, _read_u64(data, 8) ^ seed)
        if length > 32:
            seed = _rapid_mix(_read_u64(data, 16) ^ _RAPIDHASH_SECRET_2, _read_u64(data, 24) ^ seed)
            if length > 48:
                seed = _rapid_mix(_read_u64(data, 32) ^ _RAPIDHASH_SECRET_1, _read_u64(data, 40) ^ seed)
                if length > 64:
                    seed = _rapid_mix(_read_u64(data, 48) ^ _RAPIDHASH_SECRET_1, _read_u64(data, 56) ^ seed)
        remainder = length
        a ^= _read_u64(data, length - 16) ^ remainder
        b ^= _read_u64(data, length - 8)

    a ^= _RAPIDHASH_SECRET_1
    b ^= seed
    a, b = _rapid_mum(a, b)
    return _rapid_mix(a ^ 0xaaaaaaaaaaaaaaaa, b ^ _RAPIDHASH_SECRET_1 ^ remainder)


def _validate_hash_prefix_nibbles(nibbles: int) -> None:
    if isinstance(nibbles, bool) or not isinstance(nibbles, int) or nibbles < 1 or nibbles > 16:
        raise ValueError("Expected hash-prefix length from 1 to 16 hex nibbles")


def _validate_address_payload_for_type(address_type: OutputType, payload: Union[bytes, bytearray, memoryview]) -> None:
    length = len(bytes(payload))
    expected = _ADDRESS_PAYLOAD_LENGTHS.get(address_type)
    if expected is None:
        raise ValueError(f"Unsupported address type for address payload hash-prefix: {address_type}")
    if length != expected:
        raise ValueError(f"Expected {address_type} address payload length {expected} bytes")


def address_payload_hash_prefix(payload: Union[bytes, bytearray, memoryview], nibbles: int) -> str:
    """Compute the RapidHash v3 hash-prefix used by `/api/address/hash-prefix/{address_type}/{prefix}`."""
    _validate_hash_prefix_nibbles(nibbles)
    return f"{_rapid_hash_v3(payload):016x}"[:nibbles]


# Date conversion, mirroring the server's indexes (`_CALENDARS`): UTC, from 2009-01-01
_EPOCH = datetime(2009, 1, 1, tzinfo=timezone.utc)
_EPOCH_DATE = _EPOCH.date()


def _calendar(index: str) -> Tuple[str, int, int]:
    calendar = _CALENDARS.get(index)
    if calendar is None:
        raise ValueError(f"{index} is not a date-based index")
    return calendar


def _index_to_date(index: str, i: int) -> Union[date, datetime]:
    """The start of bucket `i`: a UTC datetime for sub-daily indexes, a date otherwise. Raises
    ValueError for an index without a date calendar."""
    unit, size, phase = _calendar(index)
    if unit == 'second':
        return _EPOCH + timedelta(seconds=i * size)
    if unit == 'day':
        return _EPOCH_DATE + timedelta(days=i * size - phase)
    months = i * size - phase if i else 0
    return date(2009 + months // 12, months % 12 + 1, 1)


def _date_to_index(index: str, d: Union[date, datetime]) -> int:
    """Returns the floor index (latest index whose date is <= the given date).
    For sub-day indexes (minute*, hour*), a plain date is treated as midnight UTC.
    Raises ValueError for dates before the index's first date (2009-01-01; day3's is 2008-12-31, week1's 2008-12-29).
    """
    unit, size, phase = _calendar(index)
    if unit == 'second':
        if isinstance(d, datetime):
            dt = d if d.tzinfo else d.replace(tzinfo=timezone.utc)
        else:
            dt = datetime(d.year, d.month, d.day, tzinfo=timezone.utc)
        if dt < _EPOCH:
            raise ValueError("date is before the first index (2009-01-01)")
        return int((dt - _EPOCH).total_seconds()) // size
    if isinstance(d, datetime):
        d = (d.astimezone(timezone.utc) if d.tzinfo else d).date()
    first = _index_to_date(index, 0)
    if d < first:
        raise ValueError(f"date is before the first {index} date ({first})")
    if unit == 'day':
        return ((d - _EPOCH_DATE).days + phase) // size
    return ((d.year - 2009) * 12 + d.month - 1 + phase) // size


@dataclass
class SeriesData(Generic[T]):
    """Always int-indexed; a value is None where it is missing or undefined."""
    version: int
    index: Index
    type: str
    start: int
    end: int
    data: List[T]

    @property
    def is_date_based(self) -> bool:
        return self.index in _CALENDARS

    def indexes(self) -> List[int]:
        return list(range(self.start, self.end))

    def keys(self) -> List[int]:
        """Alias for indexes()."""
        return self.indexes()

    def items(self) -> List[Tuple[int, T]]:
        return list(zip(range(self.start, self.end), self.data))

    def to_dict(self) -> Dict[int, T]:
        return dict(zip(range(self.start, self.end), self.data))

    def __iter__(self) -> Iterator[Tuple[int, T]]:
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
    def dates(self) -> List[Union[date, datetime]]:
        """Get dates for the index range. Returns datetime for sub-daily indexes, date for daily+."""
        return [_index_to_date(self.index, i) for i in range(self.start, self.end)]

    def date_items(self) -> List[Tuple[Union[date, datetime], T]]:
        return list(zip(self.dates(), self.data))

    def to_date_dict(self) -> Dict[Union[date, datetime], T]:
        return dict(zip(self.dates(), self.data))

    def to_polars(self, with_dates: bool = True) -> pl.DataFrame:
        """Columns 'date' and 'value', or 'index' and 'value' with with_dates=False."""
        try:
            import polars as pl  # type: ignore[import-not-found]
        except ImportError:
            raise ImportError("polars is required: pip install polars")
        if with_dates:
            return pl.DataFrame({"date": self.dates(), "value": self.data})
        return pl.DataFrame({"index": self.indexes(), "value": self.data})

    def to_pandas(self, with_dates: bool = True) -> pd.DataFrame:
        """Columns 'date' and 'value', or 'index' and 'value' with with_dates=False."""
        try:
            import pandas as pd  # type: ignore[import-not-found]
        except ImportError:
            raise ImportError("pandas is required: pip install pandas")
        if with_dates:
            return pd.DataFrame({"date": self.dates(), "value": self.data})
        return pd.DataFrame({"index": self.indexes(), "value": self.data})


AnySeriesData = SeriesData[Any]
AnyDateSeriesData = DateSeriesData[Any]


def _series_data(raw: Dict[str, Any]) -> AnySeriesData:
    """Series data with its helpers (and dates, for a date-based index)."""
    return (DateSeriesData if raw['index'] in _CALENDARS else SeriesData)(**raw)


def _contiguous(key: slice) -> slice:
    """Series slices select a contiguous range: a step is rejected, not ignored."""
    if key.step not in (None, 1):
        raise ValueError("series slices do not support a step")
    return key


class _EndpointConfig:
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
        return f"/api/series/{_segment(self.name)}/{self.index}"

    def _build_path(self, format: Optional[str] = None) -> str:
        params: List[Tuple[str, Any]] = []
        if self.start is not None:
            params.append(("start", self.start))
        if self.end is not None:
            params.append(("end", self.end))
        if self.limit is not None:
            params.append(("limit", self.limit))
        if format is not None:
            params.append(("format", format))
        query = urlencode(params)
        path = self.path()
        return f"{path}?{query}" if query else path

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
    """Builder with a selection (one item or a range) specified."""

    def __init__(self, config: _EndpointConfig):
        self._config = config

    def fetch(self) -> SeriesData[T]:
        return self._config.get_series()

    def fetch_csv(self) -> str:
        return self._config.get_csv()


class SkippedBuilder(RangeBuilder[T]):
    """After skip(n): take() sets a count; fetch() reads from the skipped position to the end."""

    def take(self, n: int) -> RangeBuilder[T]:
        return RangeBuilder(self._config._new(self._config.start, limit=n))


class DateRangeBuilder(RangeBuilder[T]):
    def fetch(self) -> DateSeriesData[T]:
        return self._config.get_date_series()


class DateSkippedBuilder(SkippedBuilder[T]):
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

    def __getitem__(self, key: Union[int, slice]) -> RangeBuilder[T]:
        if isinstance(key, int):
            return RangeBuilder(self._config._new(key, limit=1))
        key = _contiguous(key)
        return RangeBuilder(self._config._new(key.start, key.stop))

    def head(self, n: int = 10) -> RangeBuilder[T]:
        return RangeBuilder(self._config._new(end=n))

    def tail(self, n: int = 10) -> RangeBuilder[T]:
        return RangeBuilder(self._config._new(end=0) if n == 0 else self._config._new(start=-n))

    def skip(self, n: int) -> SkippedBuilder[T]:
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
        return self._config.get_version()

    def path(self) -> str:
        return self._config.path()


class DateSeriesEndpoint(SeriesEndpoint[T]):
    """Builder for series endpoint queries with date-based indexing.

    Accepts dates in __getitem__ and returns DateSeriesData from fetch().

    Examples:
        data = endpoint[date(2020, 1, 1)].fetch()
        data = endpoint[date(2020, 1, 1):date(2023, 1, 1)].fetch()
    """

    def __getitem__(self, key: Union[int, slice, date, datetime]) -> DateRangeBuilder[T]:
        if isinstance(key, (date, datetime)):
            idx = _date_to_index(self._config.index, key)
            return DateRangeBuilder(self._config._new(idx, limit=1))
        if isinstance(key, int):
            return DateRangeBuilder(self._config._new(key, limit=1))
        key = _contiguous(key)
        start, stop = key.start, key.stop
        if isinstance(start, (date, datetime)):
            start = _date_to_index(self._config.index, start)
        if isinstance(stop, (date, datetime)):
            stop = _date_to_index(self._config.index, stop)
        return DateRangeBuilder(self._config._new(start, stop))

    def head(self, n: int = 10) -> DateRangeBuilder[T]:
        return DateRangeBuilder(self._config._new(end=n))

    def tail(self, n: int = 10) -> DateRangeBuilder[T]:
        return DateRangeBuilder(self._config._new(end=0) if n == 0 else self._config._new(start=-n))

    def skip(self, n: int) -> DateSkippedBuilder[T]:
        return DateSkippedBuilder(self._config._new(start=n))

    def fetch(self) -> DateSeriesData[T]:
        """Fetch all data."""
        return self._config.get_date_series()


AnySeriesEndpoint = SeriesEndpoint[Any]
AnyDateSeriesEndpoint = DateSeriesEndpoint[Any]


class SeriesPattern(Protocol[T]):
    @property
    def name(self) -> str:
        ...

    def indexes(self) -> List[str]:
        ...

    def get(self, index: Index) -> Optional[SeriesEndpoint[T]]:
        """Get an endpoint builder for a specific index, if supported."""
        ...


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
_i21 = ('txin_index',)
_i22 = ('txout_index',)
_i23 = ('txout_index',)
_i24 = ('empty_output_index',)
_i25 = ('op_return_index',)
_i26 = ('p2a_address_index',)
_i27 = ('p2ms_output_index',)
_i28 = ('p2pk33_address_index',)
_i29 = ('p2pk65_address_index',)
_i30 = ('p2pkh_address_index',)
_i31 = ('p2sh_address_index',)
_i32 = ('p2tr_address_index',)
_i33 = ('p2wpkh_address_index',)
_i34 = ('p2wsh_address_index',)
_i35 = ('unknown_output_index',)

def _endpoint(c: BitviewClient, n: str, i: Index) -> Union[SeriesEndpoint[Any], DateSeriesEndpoint[Any]]:
    """The endpoint for any index: with date helpers for a date-based one."""
    return DateSeriesEndpoint(c, n, i) if i in _CALENDARS else SeriesEndpoint(c, n, i)

class _SeriesPattern1By(Generic[T]):
    def __init__(self, c: BitviewClient, n: str): self._c, self._n = c, n
    def minute10(self) -> DateSeriesEndpoint[Optional[T]]: return DateSeriesEndpoint(self._c, self._n, 'minute10')
    def minute30(self) -> DateSeriesEndpoint[Optional[T]]: return DateSeriesEndpoint(self._c, self._n, 'minute30')
    def hour1(self) -> DateSeriesEndpoint[Optional[T]]: return DateSeriesEndpoint(self._c, self._n, 'hour1')
    def hour4(self) -> DateSeriesEndpoint[Optional[T]]: return DateSeriesEndpoint(self._c, self._n, 'hour4')
    def hour12(self) -> DateSeriesEndpoint[Optional[T]]: return DateSeriesEndpoint(self._c, self._n, 'hour12')
    def day1(self) -> DateSeriesEndpoint[Optional[T]]: return DateSeriesEndpoint(self._c, self._n, 'day1')
    def day3(self) -> DateSeriesEndpoint[Optional[T]]: return DateSeriesEndpoint(self._c, self._n, 'day3')
    def week1(self) -> DateSeriesEndpoint[Optional[T]]: return DateSeriesEndpoint(self._c, self._n, 'week1')
    def month1(self) -> DateSeriesEndpoint[Optional[T]]: return DateSeriesEndpoint(self._c, self._n, 'month1')
    def month3(self) -> DateSeriesEndpoint[Optional[T]]: return DateSeriesEndpoint(self._c, self._n, 'month3')
    def month6(self) -> DateSeriesEndpoint[Optional[T]]: return DateSeriesEndpoint(self._c, self._n, 'month6')
    def year1(self) -> DateSeriesEndpoint[Optional[T]]: return DateSeriesEndpoint(self._c, self._n, 'year1')
    def year10(self) -> DateSeriesEndpoint[Optional[T]]: return DateSeriesEndpoint(self._c, self._n, 'year10')
    def halving(self) -> SeriesEndpoint[T]: return SeriesEndpoint(self._c, self._n, 'halving')
    def epoch(self) -> SeriesEndpoint[T]: return SeriesEndpoint(self._c, self._n, 'epoch')
    def height(self) -> SeriesEndpoint[T]: return SeriesEndpoint(self._c, self._n, 'height')

class SeriesPattern1(Generic[T]):
    by: _SeriesPattern1By[T]
    def __init__(self, c: BitviewClient, n: str): self._n, self.by = n, _SeriesPattern1By(c, n)
    @property
    def name(self) -> str: return self._n
    def indexes(self) -> List[str]: return list(_i1)
    def get(self, index: Index) -> Optional[Union[SeriesEndpoint[Optional[T]], DateSeriesEndpoint[Optional[T]]]]: return _endpoint(self.by._c, self._n, index) if index in _i1 else None

class _SeriesPattern2By(Generic[T]):
    def __init__(self, c: BitviewClient, n: str): self._c, self._n = c, n
    def minute10(self) -> DateSeriesEndpoint[T]: return DateSeriesEndpoint(self._c, self._n, 'minute10')
    def minute30(self) -> DateSeriesEndpoint[T]: return DateSeriesEndpoint(self._c, self._n, 'minute30')
    def hour1(self) -> DateSeriesEndpoint[T]: return DateSeriesEndpoint(self._c, self._n, 'hour1')
    def hour4(self) -> DateSeriesEndpoint[T]: return DateSeriesEndpoint(self._c, self._n, 'hour4')
    def hour12(self) -> DateSeriesEndpoint[T]: return DateSeriesEndpoint(self._c, self._n, 'hour12')
    def day1(self) -> DateSeriesEndpoint[T]: return DateSeriesEndpoint(self._c, self._n, 'day1')
    def day3(self) -> DateSeriesEndpoint[T]: return DateSeriesEndpoint(self._c, self._n, 'day3')
    def week1(self) -> DateSeriesEndpoint[T]: return DateSeriesEndpoint(self._c, self._n, 'week1')
    def month1(self) -> DateSeriesEndpoint[T]: return DateSeriesEndpoint(self._c, self._n, 'month1')
    def month3(self) -> DateSeriesEndpoint[T]: return DateSeriesEndpoint(self._c, self._n, 'month3')
    def month6(self) -> DateSeriesEndpoint[T]: return DateSeriesEndpoint(self._c, self._n, 'month6')
    def year1(self) -> DateSeriesEndpoint[T]: return DateSeriesEndpoint(self._c, self._n, 'year1')
    def year10(self) -> DateSeriesEndpoint[T]: return DateSeriesEndpoint(self._c, self._n, 'year10')
    def halving(self) -> SeriesEndpoint[T]: return SeriesEndpoint(self._c, self._n, 'halving')
    def epoch(self) -> SeriesEndpoint[T]: return SeriesEndpoint(self._c, self._n, 'epoch')

class SeriesPattern2(Generic[T]):
    by: _SeriesPattern2By[T]
    def __init__(self, c: BitviewClient, n: str): self._n, self.by = n, _SeriesPattern2By(c, n)
    @property
    def name(self) -> str: return self._n
    def indexes(self) -> List[str]: return list(_i2)
    def get(self, index: Index) -> Optional[Union[SeriesEndpoint[T], DateSeriesEndpoint[T]]]: return _endpoint(self.by._c, self._n, index) if index in _i2 else None

class _SeriesPattern3By(Generic[T]):
    def __init__(self, c: BitviewClient, n: str): self._c, self._n = c, n
    def minute10(self) -> DateSeriesEndpoint[T]: return DateSeriesEndpoint(self._c, self._n, 'minute10')

class SeriesPattern3(Generic[T]):
    by: _SeriesPattern3By[T]
    def __init__(self, c: BitviewClient, n: str): self._n, self.by = n, _SeriesPattern3By(c, n)
    @property
    def name(self) -> str: return self._n
    def indexes(self) -> List[str]: return list(_i3)
    def get(self, index: Index) -> Optional[Union[SeriesEndpoint[T], DateSeriesEndpoint[T]]]: return _endpoint(self.by._c, self._n, index) if index in _i3 else None

class _SeriesPattern4By(Generic[T]):
    def __init__(self, c: BitviewClient, n: str): self._c, self._n = c, n
    def minute30(self) -> DateSeriesEndpoint[T]: return DateSeriesEndpoint(self._c, self._n, 'minute30')

class SeriesPattern4(Generic[T]):
    by: _SeriesPattern4By[T]
    def __init__(self, c: BitviewClient, n: str): self._n, self.by = n, _SeriesPattern4By(c, n)
    @property
    def name(self) -> str: return self._n
    def indexes(self) -> List[str]: return list(_i4)
    def get(self, index: Index) -> Optional[Union[SeriesEndpoint[T], DateSeriesEndpoint[T]]]: return _endpoint(self.by._c, self._n, index) if index in _i4 else None

class _SeriesPattern5By(Generic[T]):
    def __init__(self, c: BitviewClient, n: str): self._c, self._n = c, n
    def hour1(self) -> DateSeriesEndpoint[T]: return DateSeriesEndpoint(self._c, self._n, 'hour1')

class SeriesPattern5(Generic[T]):
    by: _SeriesPattern5By[T]
    def __init__(self, c: BitviewClient, n: str): self._n, self.by = n, _SeriesPattern5By(c, n)
    @property
    def name(self) -> str: return self._n
    def indexes(self) -> List[str]: return list(_i5)
    def get(self, index: Index) -> Optional[Union[SeriesEndpoint[T], DateSeriesEndpoint[T]]]: return _endpoint(self.by._c, self._n, index) if index in _i5 else None

class _SeriesPattern6By(Generic[T]):
    def __init__(self, c: BitviewClient, n: str): self._c, self._n = c, n
    def hour4(self) -> DateSeriesEndpoint[T]: return DateSeriesEndpoint(self._c, self._n, 'hour4')

class SeriesPattern6(Generic[T]):
    by: _SeriesPattern6By[T]
    def __init__(self, c: BitviewClient, n: str): self._n, self.by = n, _SeriesPattern6By(c, n)
    @property
    def name(self) -> str: return self._n
    def indexes(self) -> List[str]: return list(_i6)
    def get(self, index: Index) -> Optional[Union[SeriesEndpoint[T], DateSeriesEndpoint[T]]]: return _endpoint(self.by._c, self._n, index) if index in _i6 else None

class _SeriesPattern7By(Generic[T]):
    def __init__(self, c: BitviewClient, n: str): self._c, self._n = c, n
    def hour12(self) -> DateSeriesEndpoint[T]: return DateSeriesEndpoint(self._c, self._n, 'hour12')

class SeriesPattern7(Generic[T]):
    by: _SeriesPattern7By[T]
    def __init__(self, c: BitviewClient, n: str): self._n, self.by = n, _SeriesPattern7By(c, n)
    @property
    def name(self) -> str: return self._n
    def indexes(self) -> List[str]: return list(_i7)
    def get(self, index: Index) -> Optional[Union[SeriesEndpoint[T], DateSeriesEndpoint[T]]]: return _endpoint(self.by._c, self._n, index) if index in _i7 else None

class _SeriesPattern8By(Generic[T]):
    def __init__(self, c: BitviewClient, n: str): self._c, self._n = c, n
    def day1(self) -> DateSeriesEndpoint[T]: return DateSeriesEndpoint(self._c, self._n, 'day1')

class SeriesPattern8(Generic[T]):
    by: _SeriesPattern8By[T]
    def __init__(self, c: BitviewClient, n: str): self._n, self.by = n, _SeriesPattern8By(c, n)
    @property
    def name(self) -> str: return self._n
    def indexes(self) -> List[str]: return list(_i8)
    def get(self, index: Index) -> Optional[Union[SeriesEndpoint[T], DateSeriesEndpoint[T]]]: return _endpoint(self.by._c, self._n, index) if index in _i8 else None

class _SeriesPattern9By(Generic[T]):
    def __init__(self, c: BitviewClient, n: str): self._c, self._n = c, n
    def day3(self) -> DateSeriesEndpoint[T]: return DateSeriesEndpoint(self._c, self._n, 'day3')

class SeriesPattern9(Generic[T]):
    by: _SeriesPattern9By[T]
    def __init__(self, c: BitviewClient, n: str): self._n, self.by = n, _SeriesPattern9By(c, n)
    @property
    def name(self) -> str: return self._n
    def indexes(self) -> List[str]: return list(_i9)
    def get(self, index: Index) -> Optional[Union[SeriesEndpoint[T], DateSeriesEndpoint[T]]]: return _endpoint(self.by._c, self._n, index) if index in _i9 else None

class _SeriesPattern10By(Generic[T]):
    def __init__(self, c: BitviewClient, n: str): self._c, self._n = c, n
    def week1(self) -> DateSeriesEndpoint[T]: return DateSeriesEndpoint(self._c, self._n, 'week1')

class SeriesPattern10(Generic[T]):
    by: _SeriesPattern10By[T]
    def __init__(self, c: BitviewClient, n: str): self._n, self.by = n, _SeriesPattern10By(c, n)
    @property
    def name(self) -> str: return self._n
    def indexes(self) -> List[str]: return list(_i10)
    def get(self, index: Index) -> Optional[Union[SeriesEndpoint[T], DateSeriesEndpoint[T]]]: return _endpoint(self.by._c, self._n, index) if index in _i10 else None

class _SeriesPattern11By(Generic[T]):
    def __init__(self, c: BitviewClient, n: str): self._c, self._n = c, n
    def month1(self) -> DateSeriesEndpoint[T]: return DateSeriesEndpoint(self._c, self._n, 'month1')

class SeriesPattern11(Generic[T]):
    by: _SeriesPattern11By[T]
    def __init__(self, c: BitviewClient, n: str): self._n, self.by = n, _SeriesPattern11By(c, n)
    @property
    def name(self) -> str: return self._n
    def indexes(self) -> List[str]: return list(_i11)
    def get(self, index: Index) -> Optional[Union[SeriesEndpoint[T], DateSeriesEndpoint[T]]]: return _endpoint(self.by._c, self._n, index) if index in _i11 else None

class _SeriesPattern12By(Generic[T]):
    def __init__(self, c: BitviewClient, n: str): self._c, self._n = c, n
    def month3(self) -> DateSeriesEndpoint[T]: return DateSeriesEndpoint(self._c, self._n, 'month3')

class SeriesPattern12(Generic[T]):
    by: _SeriesPattern12By[T]
    def __init__(self, c: BitviewClient, n: str): self._n, self.by = n, _SeriesPattern12By(c, n)
    @property
    def name(self) -> str: return self._n
    def indexes(self) -> List[str]: return list(_i12)
    def get(self, index: Index) -> Optional[Union[SeriesEndpoint[T], DateSeriesEndpoint[T]]]: return _endpoint(self.by._c, self._n, index) if index in _i12 else None

class _SeriesPattern13By(Generic[T]):
    def __init__(self, c: BitviewClient, n: str): self._c, self._n = c, n
    def month6(self) -> DateSeriesEndpoint[T]: return DateSeriesEndpoint(self._c, self._n, 'month6')

class SeriesPattern13(Generic[T]):
    by: _SeriesPattern13By[T]
    def __init__(self, c: BitviewClient, n: str): self._n, self.by = n, _SeriesPattern13By(c, n)
    @property
    def name(self) -> str: return self._n
    def indexes(self) -> List[str]: return list(_i13)
    def get(self, index: Index) -> Optional[Union[SeriesEndpoint[T], DateSeriesEndpoint[T]]]: return _endpoint(self.by._c, self._n, index) if index in _i13 else None

class _SeriesPattern14By(Generic[T]):
    def __init__(self, c: BitviewClient, n: str): self._c, self._n = c, n
    def year1(self) -> DateSeriesEndpoint[T]: return DateSeriesEndpoint(self._c, self._n, 'year1')

class SeriesPattern14(Generic[T]):
    by: _SeriesPattern14By[T]
    def __init__(self, c: BitviewClient, n: str): self._n, self.by = n, _SeriesPattern14By(c, n)
    @property
    def name(self) -> str: return self._n
    def indexes(self) -> List[str]: return list(_i14)
    def get(self, index: Index) -> Optional[Union[SeriesEndpoint[T], DateSeriesEndpoint[T]]]: return _endpoint(self.by._c, self._n, index) if index in _i14 else None

class _SeriesPattern15By(Generic[T]):
    def __init__(self, c: BitviewClient, n: str): self._c, self._n = c, n
    def year10(self) -> DateSeriesEndpoint[T]: return DateSeriesEndpoint(self._c, self._n, 'year10')

class SeriesPattern15(Generic[T]):
    by: _SeriesPattern15By[T]
    def __init__(self, c: BitviewClient, n: str): self._n, self.by = n, _SeriesPattern15By(c, n)
    @property
    def name(self) -> str: return self._n
    def indexes(self) -> List[str]: return list(_i15)
    def get(self, index: Index) -> Optional[Union[SeriesEndpoint[T], DateSeriesEndpoint[T]]]: return _endpoint(self.by._c, self._n, index) if index in _i15 else None

class _SeriesPattern16By(Generic[T]):
    def __init__(self, c: BitviewClient, n: str): self._c, self._n = c, n
    def halving(self) -> SeriesEndpoint[T]: return SeriesEndpoint(self._c, self._n, 'halving')

class SeriesPattern16(Generic[T]):
    by: _SeriesPattern16By[T]
    def __init__(self, c: BitviewClient, n: str): self._n, self.by = n, _SeriesPattern16By(c, n)
    @property
    def name(self) -> str: return self._n
    def indexes(self) -> List[str]: return list(_i16)
    def get(self, index: Index) -> Optional[Union[SeriesEndpoint[T], DateSeriesEndpoint[T]]]: return _endpoint(self.by._c, self._n, index) if index in _i16 else None

class _SeriesPattern17By(Generic[T]):
    def __init__(self, c: BitviewClient, n: str): self._c, self._n = c, n
    def epoch(self) -> SeriesEndpoint[T]: return SeriesEndpoint(self._c, self._n, 'epoch')

class SeriesPattern17(Generic[T]):
    by: _SeriesPattern17By[T]
    def __init__(self, c: BitviewClient, n: str): self._n, self.by = n, _SeriesPattern17By(c, n)
    @property
    def name(self) -> str: return self._n
    def indexes(self) -> List[str]: return list(_i17)
    def get(self, index: Index) -> Optional[Union[SeriesEndpoint[T], DateSeriesEndpoint[T]]]: return _endpoint(self.by._c, self._n, index) if index in _i17 else None

class _SeriesPattern18By(Generic[T]):
    def __init__(self, c: BitviewClient, n: str): self._c, self._n = c, n
    def height(self) -> SeriesEndpoint[T]: return SeriesEndpoint(self._c, self._n, 'height')

class SeriesPattern18(Generic[T]):
    by: _SeriesPattern18By[T]
    def __init__(self, c: BitviewClient, n: str): self._n, self.by = n, _SeriesPattern18By(c, n)
    @property
    def name(self) -> str: return self._n
    def indexes(self) -> List[str]: return list(_i18)
    def get(self, index: Index) -> Optional[Union[SeriesEndpoint[T], DateSeriesEndpoint[T]]]: return _endpoint(self.by._c, self._n, index) if index in _i18 else None

class _SeriesPattern19By(Generic[T]):
    def __init__(self, c: BitviewClient, n: str): self._c, self._n = c, n
    def tx_index(self) -> SeriesEndpoint[T]: return SeriesEndpoint(self._c, self._n, 'tx_index')

class SeriesPattern19(Generic[T]):
    by: _SeriesPattern19By[T]
    def __init__(self, c: BitviewClient, n: str): self._n, self.by = n, _SeriesPattern19By(c, n)
    @property
    def name(self) -> str: return self._n
    def indexes(self) -> List[str]: return list(_i19)
    def get(self, index: Index) -> Optional[Union[SeriesEndpoint[T], DateSeriesEndpoint[T]]]: return _endpoint(self.by._c, self._n, index) if index in _i19 else None

class _SeriesPattern20By(Generic[T]):
    def __init__(self, c: BitviewClient, n: str): self._c, self._n = c, n
    def txin_index(self) -> SeriesEndpoint[T]: return SeriesEndpoint(self._c, self._n, 'txin_index')

class SeriesPattern20(Generic[T]):
    by: _SeriesPattern20By[T]
    def __init__(self, c: BitviewClient, n: str): self._n, self.by = n, _SeriesPattern20By(c, n)
    @property
    def name(self) -> str: return self._n
    def indexes(self) -> List[str]: return list(_i20)
    def get(self, index: Index) -> Optional[Union[SeriesEndpoint[T], DateSeriesEndpoint[T]]]: return _endpoint(self.by._c, self._n, index) if index in _i20 else None

class _SeriesPattern21By(Generic[T]):
    def __init__(self, c: BitviewClient, n: str): self._c, self._n = c, n
    def txin_index(self) -> SeriesEndpoint[Optional[T]]: return SeriesEndpoint(self._c, self._n, 'txin_index')

class SeriesPattern21(Generic[T]):
    by: _SeriesPattern21By[T]
    def __init__(self, c: BitviewClient, n: str): self._n, self.by = n, _SeriesPattern21By(c, n)
    @property
    def name(self) -> str: return self._n
    def indexes(self) -> List[str]: return list(_i21)
    def get(self, index: Index) -> Optional[Union[SeriesEndpoint[Optional[T]], DateSeriesEndpoint[Optional[T]]]]: return _endpoint(self.by._c, self._n, index) if index in _i21 else None

class _SeriesPattern22By(Generic[T]):
    def __init__(self, c: BitviewClient, n: str): self._c, self._n = c, n
    def txout_index(self) -> SeriesEndpoint[T]: return SeriesEndpoint(self._c, self._n, 'txout_index')

class SeriesPattern22(Generic[T]):
    by: _SeriesPattern22By[T]
    def __init__(self, c: BitviewClient, n: str): self._n, self.by = n, _SeriesPattern22By(c, n)
    @property
    def name(self) -> str: return self._n
    def indexes(self) -> List[str]: return list(_i22)
    def get(self, index: Index) -> Optional[Union[SeriesEndpoint[T], DateSeriesEndpoint[T]]]: return _endpoint(self.by._c, self._n, index) if index in _i22 else None

class _SeriesPattern23By(Generic[T]):
    def __init__(self, c: BitviewClient, n: str): self._c, self._n = c, n
    def txout_index(self) -> SeriesEndpoint[Optional[T]]: return SeriesEndpoint(self._c, self._n, 'txout_index')

class SeriesPattern23(Generic[T]):
    by: _SeriesPattern23By[T]
    def __init__(self, c: BitviewClient, n: str): self._n, self.by = n, _SeriesPattern23By(c, n)
    @property
    def name(self) -> str: return self._n
    def indexes(self) -> List[str]: return list(_i23)
    def get(self, index: Index) -> Optional[Union[SeriesEndpoint[Optional[T]], DateSeriesEndpoint[Optional[T]]]]: return _endpoint(self.by._c, self._n, index) if index in _i23 else None

class _SeriesPattern24By(Generic[T]):
    def __init__(self, c: BitviewClient, n: str): self._c, self._n = c, n
    def empty_output_index(self) -> SeriesEndpoint[T]: return SeriesEndpoint(self._c, self._n, 'empty_output_index')

class SeriesPattern24(Generic[T]):
    by: _SeriesPattern24By[T]
    def __init__(self, c: BitviewClient, n: str): self._n, self.by = n, _SeriesPattern24By(c, n)
    @property
    def name(self) -> str: return self._n
    def indexes(self) -> List[str]: return list(_i24)
    def get(self, index: Index) -> Optional[Union[SeriesEndpoint[T], DateSeriesEndpoint[T]]]: return _endpoint(self.by._c, self._n, index) if index in _i24 else None

class _SeriesPattern25By(Generic[T]):
    def __init__(self, c: BitviewClient, n: str): self._c, self._n = c, n
    def op_return_index(self) -> SeriesEndpoint[T]: return SeriesEndpoint(self._c, self._n, 'op_return_index')

class SeriesPattern25(Generic[T]):
    by: _SeriesPattern25By[T]
    def __init__(self, c: BitviewClient, n: str): self._n, self.by = n, _SeriesPattern25By(c, n)
    @property
    def name(self) -> str: return self._n
    def indexes(self) -> List[str]: return list(_i25)
    def get(self, index: Index) -> Optional[Union[SeriesEndpoint[T], DateSeriesEndpoint[T]]]: return _endpoint(self.by._c, self._n, index) if index in _i25 else None

class _SeriesPattern26By(Generic[T]):
    def __init__(self, c: BitviewClient, n: str): self._c, self._n = c, n
    def p2a_address_index(self) -> SeriesEndpoint[T]: return SeriesEndpoint(self._c, self._n, 'p2a_address_index')

class SeriesPattern26(Generic[T]):
    by: _SeriesPattern26By[T]
    def __init__(self, c: BitviewClient, n: str): self._n, self.by = n, _SeriesPattern26By(c, n)
    @property
    def name(self) -> str: return self._n
    def indexes(self) -> List[str]: return list(_i26)
    def get(self, index: Index) -> Optional[Union[SeriesEndpoint[T], DateSeriesEndpoint[T]]]: return _endpoint(self.by._c, self._n, index) if index in _i26 else None

class _SeriesPattern27By(Generic[T]):
    def __init__(self, c: BitviewClient, n: str): self._c, self._n = c, n
    def p2ms_output_index(self) -> SeriesEndpoint[T]: return SeriesEndpoint(self._c, self._n, 'p2ms_output_index')

class SeriesPattern27(Generic[T]):
    by: _SeriesPattern27By[T]
    def __init__(self, c: BitviewClient, n: str): self._n, self.by = n, _SeriesPattern27By(c, n)
    @property
    def name(self) -> str: return self._n
    def indexes(self) -> List[str]: return list(_i27)
    def get(self, index: Index) -> Optional[Union[SeriesEndpoint[T], DateSeriesEndpoint[T]]]: return _endpoint(self.by._c, self._n, index) if index in _i27 else None

class _SeriesPattern28By(Generic[T]):
    def __init__(self, c: BitviewClient, n: str): self._c, self._n = c, n
    def p2pk33_address_index(self) -> SeriesEndpoint[T]: return SeriesEndpoint(self._c, self._n, 'p2pk33_address_index')

class SeriesPattern28(Generic[T]):
    by: _SeriesPattern28By[T]
    def __init__(self, c: BitviewClient, n: str): self._n, self.by = n, _SeriesPattern28By(c, n)
    @property
    def name(self) -> str: return self._n
    def indexes(self) -> List[str]: return list(_i28)
    def get(self, index: Index) -> Optional[Union[SeriesEndpoint[T], DateSeriesEndpoint[T]]]: return _endpoint(self.by._c, self._n, index) if index in _i28 else None

class _SeriesPattern29By(Generic[T]):
    def __init__(self, c: BitviewClient, n: str): self._c, self._n = c, n
    def p2pk65_address_index(self) -> SeriesEndpoint[T]: return SeriesEndpoint(self._c, self._n, 'p2pk65_address_index')

class SeriesPattern29(Generic[T]):
    by: _SeriesPattern29By[T]
    def __init__(self, c: BitviewClient, n: str): self._n, self.by = n, _SeriesPattern29By(c, n)
    @property
    def name(self) -> str: return self._n
    def indexes(self) -> List[str]: return list(_i29)
    def get(self, index: Index) -> Optional[Union[SeriesEndpoint[T], DateSeriesEndpoint[T]]]: return _endpoint(self.by._c, self._n, index) if index in _i29 else None

class _SeriesPattern30By(Generic[T]):
    def __init__(self, c: BitviewClient, n: str): self._c, self._n = c, n
    def p2pkh_address_index(self) -> SeriesEndpoint[T]: return SeriesEndpoint(self._c, self._n, 'p2pkh_address_index')

class SeriesPattern30(Generic[T]):
    by: _SeriesPattern30By[T]
    def __init__(self, c: BitviewClient, n: str): self._n, self.by = n, _SeriesPattern30By(c, n)
    @property
    def name(self) -> str: return self._n
    def indexes(self) -> List[str]: return list(_i30)
    def get(self, index: Index) -> Optional[Union[SeriesEndpoint[T], DateSeriesEndpoint[T]]]: return _endpoint(self.by._c, self._n, index) if index in _i30 else None

class _SeriesPattern31By(Generic[T]):
    def __init__(self, c: BitviewClient, n: str): self._c, self._n = c, n
    def p2sh_address_index(self) -> SeriesEndpoint[T]: return SeriesEndpoint(self._c, self._n, 'p2sh_address_index')

class SeriesPattern31(Generic[T]):
    by: _SeriesPattern31By[T]
    def __init__(self, c: BitviewClient, n: str): self._n, self.by = n, _SeriesPattern31By(c, n)
    @property
    def name(self) -> str: return self._n
    def indexes(self) -> List[str]: return list(_i31)
    def get(self, index: Index) -> Optional[Union[SeriesEndpoint[T], DateSeriesEndpoint[T]]]: return _endpoint(self.by._c, self._n, index) if index in _i31 else None

class _SeriesPattern32By(Generic[T]):
    def __init__(self, c: BitviewClient, n: str): self._c, self._n = c, n
    def p2tr_address_index(self) -> SeriesEndpoint[T]: return SeriesEndpoint(self._c, self._n, 'p2tr_address_index')

class SeriesPattern32(Generic[T]):
    by: _SeriesPattern32By[T]
    def __init__(self, c: BitviewClient, n: str): self._n, self.by = n, _SeriesPattern32By(c, n)
    @property
    def name(self) -> str: return self._n
    def indexes(self) -> List[str]: return list(_i32)
    def get(self, index: Index) -> Optional[Union[SeriesEndpoint[T], DateSeriesEndpoint[T]]]: return _endpoint(self.by._c, self._n, index) if index in _i32 else None

class _SeriesPattern33By(Generic[T]):
    def __init__(self, c: BitviewClient, n: str): self._c, self._n = c, n
    def p2wpkh_address_index(self) -> SeriesEndpoint[T]: return SeriesEndpoint(self._c, self._n, 'p2wpkh_address_index')

class SeriesPattern33(Generic[T]):
    by: _SeriesPattern33By[T]
    def __init__(self, c: BitviewClient, n: str): self._n, self.by = n, _SeriesPattern33By(c, n)
    @property
    def name(self) -> str: return self._n
    def indexes(self) -> List[str]: return list(_i33)
    def get(self, index: Index) -> Optional[Union[SeriesEndpoint[T], DateSeriesEndpoint[T]]]: return _endpoint(self.by._c, self._n, index) if index in _i33 else None

class _SeriesPattern34By(Generic[T]):
    def __init__(self, c: BitviewClient, n: str): self._c, self._n = c, n
    def p2wsh_address_index(self) -> SeriesEndpoint[T]: return SeriesEndpoint(self._c, self._n, 'p2wsh_address_index')

class SeriesPattern34(Generic[T]):
    by: _SeriesPattern34By[T]
    def __init__(self, c: BitviewClient, n: str): self._n, self.by = n, _SeriesPattern34By(c, n)
    @property
    def name(self) -> str: return self._n
    def indexes(self) -> List[str]: return list(_i34)
    def get(self, index: Index) -> Optional[Union[SeriesEndpoint[T], DateSeriesEndpoint[T]]]: return _endpoint(self.by._c, self._n, index) if index in _i34 else None

class _SeriesPattern35By(Generic[T]):
    def __init__(self, c: BitviewClient, n: str): self._c, self._n = c, n
    def unknown_output_index(self) -> SeriesEndpoint[T]: return SeriesEndpoint(self._c, self._n, 'unknown_output_index')

class SeriesPattern35(Generic[T]):
    by: _SeriesPattern35By[T]
    def __init__(self, c: BitviewClient, n: str): self._n, self.by = n, _SeriesPattern35By(c, n)
    @property
    def name(self) -> str: return self._n
    def indexes(self) -> List[str]: return list(_i35)
    def get(self, index: Index) -> Optional[Union[SeriesEndpoint[T], DateSeriesEndpoint[T]]]: return _endpoint(self.by._c, self._n, index) if index in _i35 else None

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
    """A `_Child`, typed `Any` so that each child's annotation can name its type."""
    return _Child(make, template)


class _Node:
    """A series-tree node over base `_b`, built with one builder per shape parameter."""

    def __init__(self, c: BitviewClient, b: str, *f: Callable[[BitviewClient, str], Any]):
        self._c, self._b, self._f = c, b, f

A = TypeVar('A')
B = TypeVar('B')
C = TypeVar('C')
D = TypeVar('D')
E = TypeVar('E')
F = TypeVar('F')

class Meter(_Node):
    pct0_1: SeriesPattern1[Optional[Dollars]] = _at(SeriesPattern1, '*_pct0_1')
    pct0_5: SeriesPattern1[Optional[Dollars]] = _at(SeriesPattern1, '*_pct0_5')
    pct1: SeriesPattern1[Optional[Dollars]] = _at(SeriesPattern1, '*_pct1')
    pct2: SeriesPattern1[Optional[Dollars]] = _at(SeriesPattern1, '*_pct2')
    pct5: SeriesPattern1[Optional[Dollars]] = _at(SeriesPattern1, '*_pct5')
    pct10: SeriesPattern1[Optional[Dollars]] = _at(SeriesPattern1, '*_pct10')
    pct20: SeriesPattern1[Optional[Dollars]] = _at(SeriesPattern1, '*_pct20')
    pct30: SeriesPattern1[Optional[Dollars]] = _at(SeriesPattern1, '*_pct30')
    pct40: SeriesPattern1[Optional[Dollars]] = _at(SeriesPattern1, '*_pct40')
    median: SeriesPattern1[Optional[Dollars]] = _at(SeriesPattern1, '*_median')
    pct60: SeriesPattern1[Optional[Dollars]] = _at(SeriesPattern1, '*_pct60')
    pct70: SeriesPattern1[Optional[Dollars]] = _at(SeriesPattern1, '*_pct70')
    pct80: SeriesPattern1[Optional[Dollars]] = _at(SeriesPattern1, '*_pct80')
    pct90: SeriesPattern1[Optional[Dollars]] = _at(SeriesPattern1, '*_pct90')
    pct95: SeriesPattern1[Optional[Dollars]] = _at(SeriesPattern1, '*_pct95')
    pct98: SeriesPattern1[Optional[Dollars]] = _at(SeriesPattern1, '*_pct98')
    pct99: SeriesPattern1[Optional[Dollars]] = _at(SeriesPattern1, '*_pct99')
    pct99_5: SeriesPattern1[Optional[Dollars]] = _at(SeriesPattern1, '*_pct99_5')
    pct99_9: SeriesPattern1[Optional[Dollars]] = _at(SeriesPattern1, '*_pct99_9')
    level: SeriesPattern1[Optional[Score]] = _at(SeriesPattern1, '*_level')
    score: SeriesPattern1[Optional[Score]] = _at(SeriesPattern1, '*_score')


class Threshold(_Node, Generic[A]):
    tail0_1: SeriesPattern1[A] = _at(SeriesPattern1, '*_1')
    tail0_05: SeriesPattern1[A] = _at(SeriesPattern1, '*_05')
    tail0_025: SeriesPattern1[A] = _at(SeriesPattern1, '*_025')


class Extreme(_Node, Generic[A]):
    thresholds: Threshold[A] = _at(Threshold, '*_threshold_tail0')
    tail_share: SeriesPattern1[Optional[Percent]] = _at(SeriesPattern1, '*_tail_share')


class Extremes(_Node):
    supply_in_loss: Extreme[Optional[Bitcoin]] = _at(Extreme, '*_supply_in_loss')
    realized_profit_24h: Extreme[Optional[Dollars]] = _at(Extreme, '*_realized_profit_24h')
    realized_loss_24h: Extreme[Optional[Dollars]] = _at(Extreme, '*_realized_loss_24h')
    realized_peak_regret_24h: Extreme[Optional[Dollars]] = _at(Extreme, '*_realized_peak_regret_24h')
    sell_side_risk_ratio_24h: Extreme[Optional[Ratio]] = _at(Extreme, '*_sell_side_risk_ratio_24h')


class MobileMedianCapitalizedPriceMedian(_Node):
    ratio: SeriesPattern1[Optional[Multiple]] = _at(SeriesPattern1, 'mobile_median_capitalized_price_rarity_ratio_*')
    band: SeriesPattern1[Optional[Dollars]] = _at(SeriesPattern1, 'mobile_median_capitalized_price_rarity_*')


class MobileMedianRealizedPriceMedian(_Node):
    ratio: SeriesPattern1[Optional[Multiple]] = _at(SeriesPattern1, 'mobile_median_realized_price_rarity_ratio_*')
    band: SeriesPattern1[Optional[Dollars]] = _at(SeriesPattern1, 'mobile_median_realized_price_rarity_*')


class MobileRealizedPriceMedian(_Node):
    ratio: SeriesPattern1[Optional[Multiple]] = _at(SeriesPattern1, 'mobile_realized_price_rarity_ratio_*')
    band: SeriesPattern1[Optional[Dollars]] = _at(SeriesPattern1, 'mobile_realized_price_rarity_*')


class AwakeRealizedPriceMedian(_Node):
    ratio: SeriesPattern1[Optional[Multiple]] = _at(SeriesPattern1, 'awake_realized_price_rarity_ratio_*')
    band: SeriesPattern1[Optional[Dollars]] = _at(SeriesPattern1, 'awake_realized_price_rarity_*')


class Under6mCapitalizedPriceMedian(_Node):
    ratio: SeriesPattern1[Optional[Multiple]] = _at(SeriesPattern1, 'age_under_6m_capitalized_price_rarity_ratio_*')
    band: SeriesPattern1[Optional[Dollars]] = _at(SeriesPattern1, 'age_under_6m_capitalized_price_rarity_*')


class Under6mRealizedPriceMedian(_Node):
    ratio: SeriesPattern1[Optional[Multiple]] = _at(SeriesPattern1, 'age_under_6m_realized_price_rarity_ratio_*')
    band: SeriesPattern1[Optional[Dollars]] = _at(SeriesPattern1, 'age_under_6m_realized_price_rarity_*')


class SthMedianCapitalizedPriceMedian(_Node):
    ratio: SeriesPattern1[Optional[Multiple]] = _at(SeriesPattern1, 'sth_median_capitalized_price_rarity_ratio_*')
    band: SeriesPattern1[Optional[Dollars]] = _at(SeriesPattern1, 'sth_median_capitalized_price_rarity_*')


class SthMedianRealizedPriceMedian(_Node):
    ratio: SeriesPattern1[Optional[Multiple]] = _at(SeriesPattern1, 'sth_median_realized_price_rarity_ratio_*')
    band: SeriesPattern1[Optional[Dollars]] = _at(SeriesPattern1, 'sth_median_realized_price_rarity_*')


class SthCapitalizedPriceMedian(_Node):
    ratio: SeriesPattern1[Optional[Multiple]] = _at(SeriesPattern1, 'sth_capitalized_price_rarity_ratio_*')
    band: SeriesPattern1[Optional[Dollars]] = _at(SeriesPattern1, 'sth_capitalized_price_rarity_*')


class SthRealizedPriceMedian(_Node):
    ratio: SeriesPattern1[Optional[Multiple]] = _at(SeriesPattern1, 'sth_realized_price_rarity_ratio_*')
    band: SeriesPattern1[Optional[Dollars]] = _at(SeriesPattern1, 'sth_realized_price_rarity_*')


class TrueMarketMeanMedian(_Node):
    ratio: SeriesPattern1[Optional[Multiple]] = _at(SeriesPattern1, 'true_market_mean_rarity_ratio_*')
    band: SeriesPattern1[Optional[Dollars]] = _at(SeriesPattern1, 'true_market_mean_rarity_*')


class VaultedPriceMedian(_Node):
    ratio: SeriesPattern1[Optional[Multiple]] = _at(SeriesPattern1, 'vaulted_price_rarity_ratio_*')
    band: SeriesPattern1[Optional[Dollars]] = _at(SeriesPattern1, 'vaulted_price_rarity_*')


class AllMedianCapitalizedPriceMedian(_Node):
    ratio: SeriesPattern1[Optional[Multiple]] = _at(SeriesPattern1, 'median_capitalized_price_rarity_ratio_*')
    band: SeriesPattern1[Optional[Dollars]] = _at(SeriesPattern1, 'median_capitalized_price_rarity_*')


class AllMedianRealizedPriceMedian(_Node):
    ratio: SeriesPattern1[Optional[Multiple]] = _at(SeriesPattern1, 'median_realized_price_rarity_ratio_*')
    band: SeriesPattern1[Optional[Dollars]] = _at(SeriesPattern1, 'median_realized_price_rarity_*')


class MedianComponent(_Node, Generic[A]):
    block: SeriesPattern1[Optional[Dollars]] = _at(SeriesPattern1, '*')
    ratio: SeriesPattern1[Optional[Multiple]] = _at(SeriesPattern1, '*_ratio')
    pct0_1: A = _at(0, 'pct0_1')
    pct0_5: A = _at(0, 'pct0_5')
    pct1: A = _at(0, 'pct1')
    pct2: A = _at(0, 'pct2')
    pct5: A = _at(0, 'pct5')
    pct10: A = _at(0, 'pct10')
    pct20: A = _at(0, 'pct20')
    pct30: A = _at(0, 'pct30')
    pct40: A = _at(0, 'pct40')
    median: A = _at(0, 'median')
    pct60: A = _at(0, 'pct60')
    pct70: A = _at(0, 'pct70')
    pct80: A = _at(0, 'pct80')
    pct90: A = _at(0, 'pct90')
    pct95: A = _at(0, 'pct95')
    pct98: A = _at(0, 'pct98')
    pct99: A = _at(0, 'pct99')
    pct99_5: A = _at(0, 'pct99_5')
    pct99_9: A = _at(0, 'pct99_9')


class AllCapitalizedPriceMedian(_Node):
    ratio: SeriesPattern1[Optional[Multiple]] = _at(SeriesPattern1, 'capitalized_price_rarity_ratio_*')
    band: SeriesPattern1[Optional[Dollars]] = _at(SeriesPattern1, 'capitalized_price_rarity_*')


class AllRealizedPriceMedian(_Node):
    ratio: SeriesPattern1[Optional[Multiple]] = _at(SeriesPattern1, 'realized_price_rarity_ratio_*')
    band: SeriesPattern1[Optional[Dollars]] = _at(SeriesPattern1, 'realized_price_rarity_*')


class RarityPercentiles(_Node, Generic[A]):
    pct0_1: A = _at(0, 'pct0_1')
    pct0_5: A = _at(0, 'pct0_5')
    pct1: A = _at(0, 'pct1')
    pct2: A = _at(0, 'pct2')
    pct5: A = _at(0, 'pct5')
    pct10: A = _at(0, 'pct10')
    pct20: A = _at(0, 'pct20')
    pct30: A = _at(0, 'pct30')
    pct40: A = _at(0, 'pct40')
    median: A = _at(0, 'median')
    pct60: A = _at(0, 'pct60')
    pct70: A = _at(0, 'pct70')
    pct80: A = _at(0, 'pct80')
    pct90: A = _at(0, 'pct90')
    pct95: A = _at(0, 'pct95')
    pct98: A = _at(0, 'pct98')
    pct99: A = _at(0, '*')
    pct99_5: A = _at(0, '*_5')
    pct99_9: A = _at(0, '*_9')


class MobileComponents(_Node):
    realized_price: RarityPercentiles[MobileRealizedPriceMedian] = _at((RarityPercentiles, MobileRealizedPriceMedian), 'pct99')
    median_realized_price: MedianComponent[MobileMedianRealizedPriceMedian] = _at((MedianComponent, MobileMedianRealizedPriceMedian), '*_realized_price')
    median_capitalized_price: MedianComponent[MobileMedianCapitalizedPriceMedian] = _at((MedianComponent, MobileMedianCapitalizedPriceMedian), '*_capitalized_price')


class AwakeComponents(_Node):
    realized_price: RarityPercentiles[AwakeRealizedPriceMedian] = _at((RarityPercentiles, AwakeRealizedPriceMedian), '*')


class MeanComponents(_Node):
    realized_price: RarityPercentiles[Under6mRealizedPriceMedian] = _at((RarityPercentiles, Under6mRealizedPriceMedian), '*')
    capitalized_price: RarityPercentiles[Under6mCapitalizedPriceMedian] = _at((RarityPercentiles, Under6mCapitalizedPriceMedian), '*')


class CohortComponents(_Node):
    realized_price: RarityPercentiles[SthRealizedPriceMedian] = _at((RarityPercentiles, SthRealizedPriceMedian), '*')
    capitalized_price: RarityPercentiles[SthCapitalizedPriceMedian] = _at((RarityPercentiles, SthCapitalizedPriceMedian), '*')
    median_realized_price: MedianComponent[SthMedianRealizedPriceMedian] = _at((MedianComponent, SthMedianRealizedPriceMedian), 'sth_median_realized_price')
    median_capitalized_price: MedianComponent[SthMedianCapitalizedPriceMedian] = _at((MedianComponent, SthMedianCapitalizedPriceMedian), 'sth_median_capitalized_price')


class AllComponents(_Node):
    realized_price: RarityPercentiles[AllRealizedPriceMedian] = _at((RarityPercentiles, AllRealizedPriceMedian), '*')
    capitalized_price: RarityPercentiles[AllCapitalizedPriceMedian] = _at((RarityPercentiles, AllCapitalizedPriceMedian), '*')
    median_realized_price: MedianComponent[AllMedianRealizedPriceMedian] = _at((MedianComponent, AllMedianRealizedPriceMedian), 'median_realized_price')
    median_capitalized_price: MedianComponent[AllMedianCapitalizedPriceMedian] = _at((MedianComponent, AllMedianCapitalizedPriceMedian), 'median_capitalized_price')
    vaulted_price: RarityPercentiles[VaultedPriceMedian] = _at((RarityPercentiles, VaultedPriceMedian), '*')
    true_market_mean: RarityPercentiles[TrueMarketMeanMedian] = _at((RarityPercentiles, TrueMarketMeanMedian), '*')


class Components(_Node):
    all: AllComponents = _at(AllComponents, '*')
    sth: CohortComponents = _at(CohortComponents, '*')
    under_6m: MeanComponents = _at(MeanComponents, '*')
    awake: AwakeComponents = _at(AwakeComponents, '*')
    mobile: MobileComponents = _at(MobileComponents, 'mobile_median')


class RarityMeter(_Node):
    components: Components = _at(Components, 'pct99')
    extremes: Extremes = _at(Extremes, '*')
    full: Meter = _at(Meter, '*')
    local: Meter = _at(Meter, 'local_*')
    cycle: Meter = _at(Meter, 'cycle_*')


class Levels(_Node):
    pct10: SeriesPattern1[Optional[Dollars]] = _at(SeriesPattern1, '*_pct10')
    pct20: SeriesPattern1[Optional[Dollars]] = _at(SeriesPattern1, '*_pct20')
    pct30: SeriesPattern1[Optional[Dollars]] = _at(SeriesPattern1, '*_pct30')
    pct40: SeriesPattern1[Optional[Dollars]] = _at(SeriesPattern1, '*_pct40')
    median: SeriesPattern1[Optional[Dollars]] = _at(SeriesPattern1, '*_median')
    pct60: SeriesPattern1[Optional[Dollars]] = _at(SeriesPattern1, '*_pct60')
    pct70: SeriesPattern1[Optional[Dollars]] = _at(SeriesPattern1, '*_pct70')
    pct80: SeriesPattern1[Optional[Dollars]] = _at(SeriesPattern1, '*_pct80')
    pct90: SeriesPattern1[Optional[Dollars]] = _at(SeriesPattern1, '*_pct90')


class Percentiles(_Node, Generic[A]):
    pct95: SeriesPattern1[A] = _at(SeriesPattern1, '*_pct95')
    pct98: SeriesPattern1[A] = _at(SeriesPattern1, '*_pct98')
    pct99: SeriesPattern1[A] = _at(SeriesPattern1, '*_pct99')
    pct99_5: SeriesPattern1[A] = _at(SeriesPattern1, '*_pct99_5')
    pct99_9: SeriesPattern1[A] = _at(SeriesPattern1, '*_pct99_9')


class Mode(_Node):
    supply_in_loss_share_threshold: Percentiles[Optional[Percent]] = _at(Percentiles, '*_supply_in_loss_share_threshold')
    floor: Percentiles[Optional[Dollars]] = _at(Percentiles, '*_floor')
    cost_basis_above_floor: Levels = _at(Levels, '*_cost_basis_above_floor')


class Bedrock(_Node):
    awake: Mode = _at(Mode, '*_awake')
    mobile: Mode = _at(Mode, '*_mobile')
    unweighted: Mode = _at(Mode, '*_unweighted')


class CointimePriceWithRatio(_Node):
    block: SeriesPattern1[Optional[Dollars]] = _at(SeriesPattern1, '*')
    ratio: SeriesPattern1[Optional[Multiple]] = _at(SeriesPattern1, '*_ratio')
    aviv: SeriesPattern1[Optional[Multiple]] = _at(SeriesPattern1, 'aviv')
    aviv_nupl: SeriesPattern1[Optional[Ratio]] = _at(SeriesPattern1, 'aviv_nupl')


class CointimePriceWithMvrv(_Node):
    block: SeriesPattern1[Optional[Dollars]] = _at(SeriesPattern1, '*_price')
    ratio: SeriesPattern1[Optional[Multiple]] = _at(SeriesPattern1, '*_price_ratio')
    mvrv: SeriesPattern1[Optional[Multiple]] = _at(SeriesPattern1, '*_mvrv')


class Prices(_Node):
    vaulted: CointimePriceWithMvrv = _at(CointimePriceWithMvrv, '*')
    active: CointimePriceWithMvrv = _at(CointimePriceWithMvrv, 'active')
    true_market_mean: CointimePriceWithRatio = _at(CointimePriceWithRatio, 'true_market_mean')
    cointime: CointimePriceWithMvrv = _at(CointimePriceWithMvrv, 'cointime')


class Caps(_Node):
    thermo: SeriesPattern1[Optional[Dollars]] = _at(SeriesPattern1, 'thermo_*')
    investor: SeriesPattern1[Optional[Dollars]] = _at(SeriesPattern1, 'investor_*')
    active: SeriesPattern1[Optional[Dollars]] = _at(SeriesPattern1, 'active_*')
    vaulted: SeriesPattern1[Optional[Dollars]] = _at(SeriesPattern1, 'vaulted_*')
    cointime: SeriesPattern1[Optional[Dollars]] = _at(SeriesPattern1, 'cointime_*')
    investorness: SeriesPattern1[Optional[Percent]] = _at(SeriesPattern1, 'investorness')
    producerness: SeriesPattern1[Optional[Percent]] = _at(SeriesPattern1, 'producerness')


class MobileCapital(_Node):
    total: SeriesPattern1[Optional[Dollars]] = _at(SeriesPattern1, '*_capital')
    realized_cap: SeriesPattern1[Optional[Dollars]] = _at(SeriesPattern1, '*_realized_cap')


class InLoss(_Node):
    share: SeriesPattern1[Optional[Percent]] = _at(SeriesPattern1, '*')


class Velocity(_Node):
    btc: SeriesPattern1[Optional[Ratio]] = _at(SeriesPattern1, '*_btc')
    usd: SeriesPattern1[Optional[Ratio]] = _at(SeriesPattern1, '*_usd')


class Adjusted(_Node):
    inflation_rate: SeriesPattern1[Optional[Percent]] = _at(SeriesPattern1, '*_inflation_rate')
    velocity: Velocity = _at(Velocity, '*_velocity')


class BandCapital(_Node):
    total: SeriesPattern1[Optional[Dollars]] = _at(SeriesPattern1, '*_capital')
    realized_cap: SeriesPattern1[Optional[Dollars]] = _at(SeriesPattern1, '*_realized_cap')
    share: SeriesPattern1[Optional[Percent]] = _at(SeriesPattern1, '*_capital_share')


class Density(_Node):
    total: SeriesPattern1[Optional[Percent]] = _at(SeriesPattern1, '*')
    in_profit: SeriesPattern1[Optional[Percent]] = _at(SeriesPattern1, '*_in_profit')
    in_loss: SeriesPattern1[Optional[Percent]] = _at(SeriesPattern1, '*_in_loss')


class PriceWithMvrv(_Node):
    block: SeriesPattern1[Optional[Dollars]] = _at(SeriesPattern1, '*_cost_basis_per_coin_avg')
    ratio: SeriesPattern1[Optional[Multiple]] = _at(SeriesPattern1, '*_cost_basis_per_coin_avg_ratio')
    mvrv: SeriesPattern1[Optional[Multiple]] = _at(SeriesPattern1, '*_mvrv')


class CostBasisPerCoin(_Node):
    avg: PriceWithMvrv = _at(PriceWithMvrv, '*')
    realized_price: SeriesPattern1[Optional[Dollars]] = _at(SeriesPattern1, '*_realized_price')
    pct5: SeriesPattern1[Optional[Dollars]] = _at(SeriesPattern1, '*_cost_basis_per_coin_pct5')
    pct10: SeriesPattern1[Optional[Dollars]] = _at(SeriesPattern1, '*_cost_basis_per_coin_pct10')
    pct15: SeriesPattern1[Optional[Dollars]] = _at(SeriesPattern1, '*_cost_basis_per_coin_pct15')
    pct20: SeriesPattern1[Optional[Dollars]] = _at(SeriesPattern1, '*_cost_basis_per_coin_pct20')
    pct25: SeriesPattern1[Optional[Dollars]] = _at(SeriesPattern1, '*_cost_basis_per_coin_pct25')
    pct30: SeriesPattern1[Optional[Dollars]] = _at(SeriesPattern1, '*_cost_basis_per_coin_pct30')
    pct35: SeriesPattern1[Optional[Dollars]] = _at(SeriesPattern1, '*_cost_basis_per_coin_pct35')
    pct40: SeriesPattern1[Optional[Dollars]] = _at(SeriesPattern1, '*_cost_basis_per_coin_pct40')
    pct45: SeriesPattern1[Optional[Dollars]] = _at(SeriesPattern1, '*_cost_basis_per_coin_pct45')
    median: SeriesPattern1[Optional[Dollars]] = _at(SeriesPattern1, '*_cost_basis_per_coin_median')
    pct55: SeriesPattern1[Optional[Dollars]] = _at(SeriesPattern1, '*_cost_basis_per_coin_pct55')
    pct60: SeriesPattern1[Optional[Dollars]] = _at(SeriesPattern1, '*_cost_basis_per_coin_pct60')
    pct65: SeriesPattern1[Optional[Dollars]] = _at(SeriesPattern1, '*_cost_basis_per_coin_pct65')
    pct70: SeriesPattern1[Optional[Dollars]] = _at(SeriesPattern1, '*_cost_basis_per_coin_pct70')
    pct75: SeriesPattern1[Optional[Dollars]] = _at(SeriesPattern1, '*_cost_basis_per_coin_pct75')
    pct80: SeriesPattern1[Optional[Dollars]] = _at(SeriesPattern1, '*_cost_basis_per_coin_pct80')
    pct85: SeriesPattern1[Optional[Dollars]] = _at(SeriesPattern1, '*_cost_basis_per_coin_pct85')
    pct90: SeriesPattern1[Optional[Dollars]] = _at(SeriesPattern1, '*_cost_basis_per_coin_pct90')
    pct95: SeriesPattern1[Optional[Dollars]] = _at(SeriesPattern1, '*_cost_basis_per_coin_pct95')
    median_realized_price: SeriesPattern1[Optional[Dollars]] = _at(SeriesPattern1, '*_median_realized_price')


class Unrealized(_Node):
    profit: SeriesPattern1[Optional[Dollars]] = _at(SeriesPattern1, '*_unrealized_profit')
    loss: SeriesPattern1[Optional[Dollars]] = _at(SeriesPattern1, '*_unrealized_loss')
    net_pnl: SeriesPattern1[Optional[Dollars]] = _at(SeriesPattern1, '*_net_unrealized_pnl')
    gross_pnl: SeriesPattern1[Optional[Dollars]] = _at(SeriesPattern1, '*_gross_unrealized_pnl')
    nupl: SeriesPattern1[Optional[Ratio]] = _at(SeriesPattern1, '*_nupl')


class Part(_Node):
    block: SeriesPattern1[Optional[Dollars]] = _at(SeriesPattern1, '*')
    share: SeriesPattern1[Optional[Percent]] = _at(SeriesPattern1, '*_share')


class SpotValue(_Node):
    btc: SeriesPattern1[Optional[Bitcoin]] = _at(SeriesPattern1, '*')
    usd: SeriesPattern1[Optional[Dollars]] = _at(SeriesPattern1, 'market_cap')


class FundedTotal(_Node):
    funded: SeriesPattern1[Count] = _at(SeriesPattern1, '*_funded_exposed_address_count')
    total: SeriesPattern1[Count] = _at(SeriesPattern1, '*_total_exposed_address_count')


class RespentCount(_Node):
    funded: SeriesPattern1[Count] = _at(SeriesPattern1, '*_funded_respent_address_count')
    total: SeriesPattern1[Count] = _at(SeriesPattern1, '*_total_respent_address_count')


class PredicateSupply(_Node):
    btc: SeriesPattern1[Optional[Bitcoin]] = _at(SeriesPattern1, '*')
    usd: SeriesPattern1[Optional[Dollars]] = _at(SeriesPattern1, '*_usd')
    share: SeriesPattern1[Optional[Percent]] = _at(SeriesPattern1, '*_share')


class Exposed(_Node):
    count: FundedTotal = _at(FundedTotal, '*')
    supply: PredicateSupply = _at(PredicateSupply, '*_exposed_address_supply')


class ReusedCount(_Node):
    funded: SeriesPattern1[Count] = _at(SeriesPattern1, '*_funded_reused_address_count')
    total: SeriesPattern1[Count] = _at(SeriesPattern1, '*_total_reused_address_count')


class PerCoin(_Node):
    avg: SeriesPattern1[Optional[Dollars]] = _at(SeriesPattern1, '*_cost_basis_per_coin_avg')
    realized_price: SeriesPattern1[Optional[Dollars]] = _at(SeriesPattern1, '*_realized_price')


class CohortCostBasis(_Node):
    per_coin: PerCoin = _at(PerCoin, '*')


class CreationUnrealized(_Node):
    profit: SeriesPattern1[Optional[Dollars]] = _at(SeriesPattern1, '*_unrealized_profit')
    loss: SeriesPattern1[Optional[Dollars]] = _at(SeriesPattern1, '*_unrealized_loss')
    net_pnl: SeriesPattern1[Optional[Dollars]] = _at(SeriesPattern1, '*_net_unrealized_pnl')


class AddressesWindows(_Node):
    _24h: SeriesPattern1[Optional[Percent]] = _at(SeriesPattern1, '*_24h_rate')
    _1w: SeriesPattern1[Optional[Percent]] = _at(SeriesPattern1, '*_1w_rate')
    _1m: SeriesPattern1[Optional[Percent]] = _at(SeriesPattern1, '*_1m_rate')
    _1y: SeriesPattern1[Optional[Percent]] = _at(SeriesPattern1, '*_1y_rate')


class MarketPriceWithRatio(_Node):
    block: SeriesPattern1[Optional[Dollars]] = _at(SeriesPattern1, '*')
    ratio: SeriesPattern1[Optional[Multiple]] = _at(SeriesPattern1, '*_ratio')
    mayer_multiple: SeriesPattern1[Optional[Multiple]] = _at(SeriesPattern1, 'mayer_multiple')


class PriceWithRatio(_Node):
    block: SeriesPattern1[Optional[Dollars]] = _at(SeriesPattern1, '*')
    ratio: SeriesPattern1[Optional[Multiple]] = _at(SeriesPattern1, '*_ratio')


class CostBasisPerDollar(_Node):
    avg: PriceWithRatio = _at(PriceWithRatio, '*_cost_basis_per_dollar_avg')
    capitalized_price: SeriesPattern1[Optional[Dollars]] = _at(SeriesPattern1, '*_capitalized_price')
    pct5: SeriesPattern1[Optional[Dollars]] = _at(SeriesPattern1, '*_cost_basis_per_dollar_pct5')
    pct10: SeriesPattern1[Optional[Dollars]] = _at(SeriesPattern1, '*_cost_basis_per_dollar_pct10')
    pct15: SeriesPattern1[Optional[Dollars]] = _at(SeriesPattern1, '*_cost_basis_per_dollar_pct15')
    pct20: SeriesPattern1[Optional[Dollars]] = _at(SeriesPattern1, '*_cost_basis_per_dollar_pct20')
    pct25: SeriesPattern1[Optional[Dollars]] = _at(SeriesPattern1, '*_cost_basis_per_dollar_pct25')
    pct30: SeriesPattern1[Optional[Dollars]] = _at(SeriesPattern1, '*_cost_basis_per_dollar_pct30')
    pct35: SeriesPattern1[Optional[Dollars]] = _at(SeriesPattern1, '*_cost_basis_per_dollar_pct35')
    pct40: SeriesPattern1[Optional[Dollars]] = _at(SeriesPattern1, '*_cost_basis_per_dollar_pct40')
    pct45: SeriesPattern1[Optional[Dollars]] = _at(SeriesPattern1, '*_cost_basis_per_dollar_pct45')
    median: SeriesPattern1[Optional[Dollars]] = _at(SeriesPattern1, '*_cost_basis_per_dollar_median')
    pct55: SeriesPattern1[Optional[Dollars]] = _at(SeriesPattern1, '*_cost_basis_per_dollar_pct55')
    pct60: SeriesPattern1[Optional[Dollars]] = _at(SeriesPattern1, '*_cost_basis_per_dollar_pct60')
    pct65: SeriesPattern1[Optional[Dollars]] = _at(SeriesPattern1, '*_cost_basis_per_dollar_pct65')
    pct70: SeriesPattern1[Optional[Dollars]] = _at(SeriesPattern1, '*_cost_basis_per_dollar_pct70')
    pct75: SeriesPattern1[Optional[Dollars]] = _at(SeriesPattern1, '*_cost_basis_per_dollar_pct75')
    pct80: SeriesPattern1[Optional[Dollars]] = _at(SeriesPattern1, '*_cost_basis_per_dollar_pct80')
    pct85: SeriesPattern1[Optional[Dollars]] = _at(SeriesPattern1, '*_cost_basis_per_dollar_pct85')
    pct90: SeriesPattern1[Optional[Dollars]] = _at(SeriesPattern1, '*_cost_basis_per_dollar_pct90')
    pct95: SeriesPattern1[Optional[Dollars]] = _at(SeriesPattern1, '*_cost_basis_per_dollar_pct95')
    median_capitalized_price: SeriesPattern1[Optional[Dollars]] = _at(SeriesPattern1, '*_median_capitalized_price')


class CostBasis(_Node):
    per_coin: CostBasisPerCoin = _at(CostBasisPerCoin, '*')
    per_dollar: CostBasisPerDollar = _at(CostBasisPerDollar, '*')
    supply_density: Density = _at(Density, '*_supply_density')
    capital_density: Density = _at(Density, '*_capital_density')


class HoldersCostBasis(_Node):
    per_coin: CostBasisPerCoin = _at(CostBasisPerCoin, '*')
    per_dollar: CostBasisPerDollar = _at(CostBasisPerDollar, '*')
    min: SeriesPattern1[Optional[Dollars]] = _at(SeriesPattern1, '*_cost_basis_min')
    max: SeriesPattern1[Optional[Dollars]] = _at(SeriesPattern1, '*_cost_basis_max')
    supply_density: Density = _at(Density, '*_supply_density')
    capital_density: Density = _at(Density, '*_capital_density')


class Ema(_Node):
    _1w: PriceWithRatio = _at(PriceWithRatio, '*_1w')
    _8d: PriceWithRatio = _at(PriceWithRatio, '*_8d')
    _13d: PriceWithRatio = _at(PriceWithRatio, '*_13d')
    _21d: PriceWithRatio = _at(PriceWithRatio, '*_21d')
    _1m: PriceWithRatio = _at(PriceWithRatio, '*_1m')
    _34d: PriceWithRatio = _at(PriceWithRatio, '*_34d')
    _55d: PriceWithRatio = _at(PriceWithRatio, '*_55d')
    _89d: PriceWithRatio = _at(PriceWithRatio, '*_89d')
    _144d: PriceWithRatio = _at(PriceWithRatio, '*_144d')
    _200d: PriceWithRatio = _at(PriceWithRatio, '*_200d')
    _1y: PriceWithRatio = _at(PriceWithRatio, '*_1y')
    _2y: PriceWithRatio = _at(PriceWithRatio, '*_2y')
    _200w: PriceWithRatio = _at(PriceWithRatio, '*_200w')
    _4y: PriceWithRatio = _at(PriceWithRatio, '*_4y')


class Sma(_Node):
    _1w: PriceWithRatio = _at(PriceWithRatio, '*_1w')
    _8d: PriceWithRatio = _at(PriceWithRatio, '*_8d')
    _13d: PriceWithRatio = _at(PriceWithRatio, '*_13d')
    _21d: PriceWithRatio = _at(PriceWithRatio, '*_21d')
    _1m: PriceWithRatio = _at(PriceWithRatio, '*_1m')
    _34d: PriceWithRatio = _at(PriceWithRatio, '*_34d')
    _50d: PriceWithRatio = _at(PriceWithRatio, '*_50d')
    _55d: PriceWithRatio = _at(PriceWithRatio, '*_55d')
    _89d: PriceWithRatio = _at(PriceWithRatio, '*_89d')
    _144d: PriceWithRatio = _at(PriceWithRatio, '*_144d')
    _200d: MarketPriceWithRatio = _at(MarketPriceWithRatio, '*_200d')
    _1y: PriceWithRatio = _at(PriceWithRatio, '*_1y')
    _2y: PriceWithRatio = _at(PriceWithRatio, '*_2y')
    _200w: PriceWithRatio = _at(PriceWithRatio, '*_200w')
    _4y: PriceWithRatio = _at(PriceWithRatio, '*_4y')


class PriceMinMax(_Node):
    _1w: SeriesPattern1[Optional[Dollars]] = _at(SeriesPattern1, '*_1w')
    _1m: SeriesPattern1[Optional[Dollars]] = _at(SeriesPattern1, '*_1m')
    _1y: SeriesPattern1[Optional[Dollars]] = _at(SeriesPattern1, '*_1y')


class MarketRange(_Node):
    min: PriceMinMax = _at(PriceMinMax, '*_min')
    max: PriceMinMax = _at(PriceMinMax, '*_max')
    true_range: SeriesPattern1[Optional[Dollars]] = _at(SeriesPattern1, '*_true_range')


class ByLookbackPeriod(_Node):
    _24h: SeriesPattern1[Optional[Percent]] = _at(SeriesPattern1, '*_24h')
    _1w: SeriesPattern1[Optional[Percent]] = _at(SeriesPattern1, '*_1w')
    _1m: SeriesPattern1[Optional[Percent]] = _at(SeriesPattern1, '*_1m')
    _3m: SeriesPattern1[Optional[Percent]] = _at(SeriesPattern1, '*_3m')
    _6m: SeriesPattern1[Optional[Percent]] = _at(SeriesPattern1, '*_6m')
    _1y: SeriesPattern1[Optional[Percent]] = _at(SeriesPattern1, '*_1y')
    _2y: SeriesPattern1[Optional[Percent]] = _at(SeriesPattern1, '*_2y')
    _3y: SeriesPattern1[Optional[Percent]] = _at(SeriesPattern1, '*_3y')
    _4y: SeriesPattern1[Optional[Percent]] = _at(SeriesPattern1, '*_4y')
    _5y: SeriesPattern1[Optional[Percent]] = _at(SeriesPattern1, '*_5y')
    _6y: SeriesPattern1[Optional[Percent]] = _at(SeriesPattern1, '*_6y')
    _8y: SeriesPattern1[Optional[Percent]] = _at(SeriesPattern1, '*_8y')
    _10y: SeriesPattern1[Optional[Percent]] = _at(SeriesPattern1, '*_10y')


class Ath(_Node):
    high: SeriesPattern1[Optional[Dollars]] = _at(SeriesPattern1, '*_ath')
    drawdown: SeriesPattern1[Optional[Percent]] = _at(SeriesPattern1, '*_drawdown')
    days_since: SeriesPattern1[Optional[Days]] = _at(SeriesPattern1, 'days_since_*_ath')
    max_days_between: SeriesPattern1[Optional[Days]] = _at(SeriesPattern1, 'max_days_between_*_ath')


class Market(_Node):
    ath: Ath = _at(Ath, '*')
    returns: ByLookbackPeriod = _at(ByLookbackPeriod, '*_return')
    range: MarketRange = _at(MarketRange, '*')
    sma: Sma = _at(Sma, '*_sma')
    ema: Ema = _at(Ema, '*_ema')


class UtxoSetSeries(_Node):
    supply: SeriesPattern1[Optional[Bitcoin]] = _at(SeriesPattern1, '*')
    count: SeriesPattern1[Count] = _at(SeriesPattern1, 'utxo_count')


class HashPriceValue(_Node, Generic[A]):
    block: SeriesPattern1[A] = _at(SeriesPattern1, '*')
    atl: SeriesPattern1[A] = _at(SeriesPattern1, '*_atl')
    rebound: SeriesPattern1[Optional[Percent]] = _at(SeriesPattern1, '*_rebound')


class HashRateSma(_Node):
    _1w: SeriesPattern1[Optional[Hashrate]] = _at(SeriesPattern1, '*_1w')
    _1m: SeriesPattern1[Optional[Hashrate]] = _at(SeriesPattern1, '*_1m')
    _2m: SeriesPattern1[Optional[Hashrate]] = _at(SeriesPattern1, '*_2m')
    _1y: SeriesPattern1[Optional[Hashrate]] = _at(SeriesPattern1, '*_1y')


class Rate(_Node):
    block: SeriesPattern1[Optional[Hashrate]] = _at(SeriesPattern1, '*')
    sma: HashRateSma = _at(HashRateSma, '*_sma')
    ath: SeriesPattern1[Optional[Hashrate]] = _at(SeriesPattern1, '*_ath')
    drawdown: SeriesPattern1[Optional[Percent]] = _at(SeriesPattern1, '*_drawdown')


class MiningHashrate(_Node):
    rate: Rate = _at(Rate, 'hashrate')
    price: HashPriceValue[Optional[HashPrice]] = _at(HashPriceValue, '*_price')
    value: HashPriceValue[Optional[HashValue]] = _at(HashPriceValue, '*_value')


class PerTxDistribution(_Node, Generic[A]):
    tx: SeriesPattern19[A] = _at(SeriesPattern19, '*')
    min: SeriesPattern1[A] = _at(SeriesPattern1, '*_min')
    max: SeriesPattern1[A] = _at(SeriesPattern1, '*_max')
    pct10: SeriesPattern1[A] = _at(SeriesPattern1, '*_pct10')
    pct25: SeriesPattern1[A] = _at(SeriesPattern1, '*_pct25')
    median: SeriesPattern1[A] = _at(SeriesPattern1, '*_median')
    pct75: SeriesPattern1[A] = _at(SeriesPattern1, '*_pct75')
    pct90: SeriesPattern1[A] = _at(SeriesPattern1, '*_pct90')


class Value(_Node):
    btc: SeriesPattern1[Optional[Bitcoin]] = _at(SeriesPattern1, '*')
    usd: SeriesPattern1[Optional[Dollars]] = _at(SeriesPattern1, '*_usd')


class CointimeSupply(_Node):
    vaulted: Value = _at(Value, 'vaulted_*')
    hodled_or_lost: Value = _at(Value, 'hodled_or_lost_*')
    active: Value = _at(Value, 'active_*')


class MobileSupply(_Node):
    total: Value = _at(Value, '*')
    in_loss: InLoss = _at(InLoss, '*_in_loss_share')


class Awake(_Node):
    supply: MobileSupply = _at(MobileSupply, '*_supply')
    capital: MobileCapital = _at(MobileCapital, '*')
    cost_basis: CostBasis = _at(CostBasis, '*')


class Side(_Node):
    supply: Value = _at(Value, '*')


class CoinflowCohortSeries(_Node):
    mobile: Awake = _at(Awake, '*_mobile')
    immobile: Side = _at(Side, '*_immobile_supply')


class CoinflowRange(_Node):
    spending_rate: SeriesPattern1[Optional[PerDay]] = _at(SeriesPattern1, '*_spending_rate')
    spending_exposure: SeriesPattern1[Optional[Float]] = _at(SeriesPattern1, '*_spending_exposure')
    mobility: SeriesPattern1[Optional[Ratio]] = _at(SeriesPattern1, '*_mobility')
    mobile: Side = _at(Side, '*_mobile_supply')
    immobile: Side = _at(Side, '*_immobile_supply')


class CointimeCohortSeries(_Node):
    awake: Awake = _at(Awake, '*_awake')
    dormant: Side = _at(Side, '*_dormant_supply')


class BandSupply(_Node):
    total: Value = _at(Value, '*')
    share: SeriesPattern1[Optional[Percent]] = _at(SeriesPattern1, '*_share')


class Band(_Node):
    supply: BandSupply = _at(BandSupply, '*_supply')
    capital: BandCapital = _at(BandCapital, '*')


class ProfitabilityRange(_Node):
    profit_over_1000_percent: Band = _at(Band, '*_profit_over_1000_percent')
    profit_500_to_1000_percent: Band = _at(Band, '*_profit_500_to_1000_percent')
    profit_300_to_500_percent: Band = _at(Band, '*_profit_300_to_500_percent')
    profit_200_to_300_percent: Band = _at(Band, '*_profit_200_to_300_percent')
    profit_100_to_200_percent: Band = _at(Band, '*_profit_100_to_200_percent')
    profit_90_to_100_percent: Band = _at(Band, '*_profit_90_to_100_percent')
    profit_80_to_90_percent: Band = _at(Band, '*_profit_80_to_90_percent')
    profit_70_to_80_percent: Band = _at(Band, '*_profit_70_to_80_percent')
    profit_60_to_70_percent: Band = _at(Band, '*_profit_60_to_70_percent')
    profit_50_to_60_percent: Band = _at(Band, '*_profit_50_to_60_percent')
    profit_40_to_50_percent: Band = _at(Band, '*_profit_40_to_50_percent')
    profit_30_to_40_percent: Band = _at(Band, '*_profit_30_to_40_percent')
    profit_20_to_30_percent: Band = _at(Band, '*_profit_20_to_30_percent')
    profit_10_to_20_percent: Band = _at(Band, '*_profit_10_to_20_percent')
    profit_0_to_10_percent: Band = _at(Band, '*_profit_0_to_10_percent')
    loss_0_to_10_percent: Band = _at(Band, '*_loss_0_to_10_percent')
    loss_10_to_20_percent: Band = _at(Band, '*_loss_10_to_20_percent')
    loss_20_to_30_percent: Band = _at(Band, '*_loss_20_to_30_percent')
    loss_30_to_40_percent: Band = _at(Band, '*_loss_30_to_40_percent')
    loss_40_to_50_percent: Band = _at(Band, '*_loss_40_to_50_percent')
    loss_50_to_60_percent: Band = _at(Band, '*_loss_50_to_60_percent')
    loss_60_to_70_percent: Band = _at(Band, '*_loss_60_to_70_percent')
    loss_70_to_80_percent: Band = _at(Band, '*_loss_70_to_80_percent')
    loss_80_to_90_percent: Band = _at(Band, '*_loss_80_to_90_percent')
    loss_90_to_100_percent: Band = _at(Band, '*_loss_90_to_100_percent')


class Profitability(_Node):
    all: ProfitabilityRange = _at(ProfitabilityRange, '')
    sth: ProfitabilityRange = _at(ProfitabilityRange, '*')
    lth: ProfitabilityRange = _at(ProfitabilityRange, 'lth')


class OpReturnWindows(_Node):
    _24h: Value = _at(Value, '*_24h')
    _1w: Value = _at(Value, '*_1w')
    _1m: Value = _at(Value, '*_1m')
    _1y: Value = _at(Value, '*_1y')


class Rewards(_Node):
    cumulative: Value = _at(Value, '*_cumulative')
    sum: OpReturnWindows = _at(OpReturnWindows, '*_sum')


class OpReturnValue(_Node):
    btc: SeriesPattern18[Optional[Bitcoin]] = _at(SeriesPattern18, '*')
    usd: SeriesPattern18[Optional[Dollars]] = _at(SeriesPattern18, '*_usd')


class ValueFlow(_Node):
    block: OpReturnValue = _at(OpReturnValue, '*')
    cumulative: Value = _at(Value, '*_cumulative')
    sum: OpReturnWindows = _at(OpReturnWindows, '*_sum')


class Activity(_Node):
    transfer_volume: ValueFlow = _at(ValueFlow, '*')


class HoldersValueFlow(_Node):
    block: OpReturnValue = _at(OpReturnValue, '*')
    cumulative: Value = _at(Value, '*_cumulative')
    sum: OpReturnWindows = _at(OpReturnWindows, '*_sum')
    in_profit: ValueFlow = _at(ValueFlow, '*_in_profit')
    in_loss: ValueFlow = _at(ValueFlow, '*_in_loss')


class Share(_Node):
    cumulative: SeriesPattern1[Optional[Percent]] = _at(SeriesPattern1, '*_cumulative')
    _24h: SeriesPattern1[Optional[Percent]] = _at(SeriesPattern1, '*_24h')
    _1w: SeriesPattern1[Optional[Percent]] = _at(SeriesPattern1, '*_1w')
    _1m: SeriesPattern1[Optional[Percent]] = _at(SeriesPattern1, '*_1m')
    _1y: SeriesPattern1[Optional[Percent]] = _at(SeriesPattern1, '*_1y')


class ValueFlowStats(_Node):
    block: OpReturnValue = _at(OpReturnValue, '*')
    cumulative: Value = _at(Value, '*_cumulative')
    sum: OpReturnWindows = _at(OpReturnWindows, '*_sum')
    avg: OpReturnWindows = _at(OpReturnWindows, '*_avg')
    min: OpReturnWindows = _at(OpReturnWindows, '*_min')
    max: OpReturnWindows = _at(OpReturnWindows, '*_max')
    pct10: OpReturnWindows = _at(OpReturnWindows, '*_pct10')
    pct25: OpReturnWindows = _at(OpReturnWindows, '*_pct25')
    median: OpReturnWindows = _at(OpReturnWindows, '*_median')
    pct75: OpReturnWindows = _at(OpReturnWindows, '*_pct75')
    pct90: OpReturnWindows = _at(OpReturnWindows, '*_pct90')
    share: Share = _at(Share, 'fee_share')


class MiningValueFlow(_Node):
    block: OpReturnValue = _at(OpReturnValue, '*')
    cumulative: Value = _at(Value, '*_cumulative')
    sum: OpReturnWindows = _at(OpReturnWindows, '*_sum')
    share: Share = _at(Share, '*_share')


class MiningRewards(_Node):
    coinbase: ValueFlow = _at(ValueFlow, '*')
    subsidy: MiningValueFlow = _at(MiningValueFlow, 'subsidy')
    fees: ValueFlowStats = _at(ValueFlowStats, 'fees')
    unclaimed: ValueFlow = _at(ValueFlow, 'unclaimed_rewards')


class Mining(_Node):
    rewards: MiningRewards = _at(MiningRewards, '*')
    hashrate: MiningHashrate = _at(MiningHashrate, 'hash')


class FeesSeries(_Node):
    block: OpReturnValue = _at(OpReturnValue, '*_fees')
    cumulative: Value = _at(Value, '*_fees_cumulative')
    sum: OpReturnWindows = _at(OpReturnWindows, '*_fees_sum')
    chain_share: Share = _at(Share, '*_fee_chain_share')


class BlocksHalving(_Node):
    epoch: SeriesPattern1[Halving] = _at(SeriesPattern1, '*_epoch')
    blocks_to_halving: SeriesPattern1[Count] = _at(SeriesPattern1, 'blocks_to_*')
    days_to_halving: SeriesPattern1[Optional[Days]] = _at(SeriesPattern1, 'days_to_*')


class BlocksDifficulty(_Node):
    block: SeriesPattern1[Optional[Difficulty]] = _at(SeriesPattern1, '*')
    hashrate: SeriesPattern1[Optional[Hashrate]] = _at(SeriesPattern1, '*_hashrate')
    adjustment: SeriesPattern1[Optional[Percent]] = _at(SeriesPattern1, '*_adjustment')
    epoch: SeriesPattern1[Epoch] = _at(SeriesPattern1, '*_epoch')
    blocks_to_retarget: SeriesPattern1[Count] = _at(SeriesPattern1, 'blocks_to_retarget')
    days_to_retarget: SeriesPattern1[Optional[Days]] = _at(SeriesPattern1, 'days_to_retarget')


class Windows(_Node, Generic[A]):
    _24h: SeriesPattern1[A] = _at(SeriesPattern1, '*_24h')
    _1w: SeriesPattern1[A] = _at(SeriesPattern1, '*_1w')
    _1m: SeriesPattern1[A] = _at(SeriesPattern1, '*_1m')
    _1y: SeriesPattern1[A] = _at(SeriesPattern1, '*_1y')


class NewAddressCount(_Node):
    block: SeriesPattern18[Count] = _at(SeriesPattern18, '*')
    sum: Windows[Count] = _at(Windows, '*_sum')


class Delta(_Node, Generic[A]):
    absolute: Windows[A] = _at(Windows, '*')
    rate: AddressesWindows = _at(AddressesWindows, '*')


class LthSupply(_Node):
    total: Value = _at(Value, '*')
    share: SeriesPattern1[Optional[Percent]] = _at(SeriesPattern1, '*_share')
    in_profit: PredicateSupply = _at(PredicateSupply, '*_in_profit')
    in_loss: PredicateSupply = _at(PredicateSupply, '*_in_loss')
    delta: Delta[Optional[Bitcoin]] = _at(Delta, '*_delta')


class Capital(_Node):
    total: SeriesPattern1[Optional[Dollars]] = _at(SeriesPattern1, '*_capital')
    realized_cap: SeriesPattern1[Optional[Dollars]] = _at(SeriesPattern1, '*_realized_cap')
    share: SeriesPattern1[Optional[Percent]] = _at(SeriesPattern1, '*_capital_share')
    in_profit: Part = _at(Part, '*_capital_in_profit')
    in_loss: Part = _at(Part, '*_capital_in_loss')
    delta: Delta[Optional[Dollars]] = _at(Delta, '*_capital_delta')


class AllSupply(_Node):
    total: SpotValue = _at(SpotValue, 'circulating_*')
    share: SeriesPattern1[Optional[Percent]] = _at(SeriesPattern1, '*_share')
    in_profit: PredicateSupply = _at(PredicateSupply, '*_in_profit')
    in_loss: PredicateSupply = _at(PredicateSupply, '*_in_loss')
    delta: Delta[Optional[Bitcoin]] = _at(Delta, '*_delta')


class CapitalViews(_Node):
    total: SeriesPattern1[Optional[Dollars]] = _at(SeriesPattern1, '*_capital')
    realized_cap: SeriesPattern1[Optional[Dollars]] = _at(SeriesPattern1, '*_realized_cap')
    delta: Delta[Optional[Dollars]] = _at(Delta, '*_capital_delta')
    share: SeriesPattern1[Optional[Percent]] = _at(SeriesPattern1, '*_capital_share')


class CohortSupply(_Node):
    total: Value = _at(Value, '*')
    delta: Delta[Optional[Bitcoin]] = _at(Delta, '*_delta')
    share: SeriesPattern1[Optional[Percent]] = _at(SeriesPattern1, '*_share')


class CreationSupply(_Node):
    total: Value = _at(Value, '*')
    delta: Delta[Optional[Bitcoin]] = _at(Delta, '*_delta')
    share: SeriesPattern1[Optional[Percent]] = _at(SeriesPattern1, '*_share')
    in_profit: Value = _at(Value, '*_in_profit')
    in_loss: Value = _at(Value, '*_in_loss')


class WithDelta(_Node, Generic[A, B]):
    block: SeriesPattern1[A] = _at(SeriesPattern1, '*')
    delta: Delta[B] = _at(Delta, '*_delta')


class Supply(_Node):
    circulating: SeriesPattern1[Optional[Bitcoin]] = _at(SeriesPattern1, '*')
    burned: ValueFlow = _at(ValueFlow, 'burned')
    inflation_rate: SeriesPattern1[Optional[Percent]] = _at(SeriesPattern1, 'inflation_rate')
    velocity: Velocity = _at(Velocity, 'velocity')
    market_cap: WithDelta[Optional[Dollars], Optional[Dollars]] = _at(WithDelta, 'market_cap')


class BalanceUtxos(_Node):
    count: WithDelta[Count, CountSigned] = _at(WithDelta, '*')


class ZeroBalance(_Node):
    funded: WithDelta[Count, CountSigned] = _at(WithDelta, '*_address_count')
    utxos: BalanceUtxos = _at(BalanceUtxos, '*_utxo_count')


class CreationCapital(_Node):
    total: SeriesPattern1[Optional[Dollars]] = _at(SeriesPattern1, '*_capital')
    realized_cap: SeriesPattern1[Optional[Dollars]] = _at(SeriesPattern1, '*_realized_cap')
    delta: Delta[Optional[Dollars]] = _at(Delta, '*_capital_delta')
    share: SeriesPattern1[Optional[Percent]] = _at(SeriesPattern1, '*_capital_share')
    in_profit: SeriesPattern1[Optional[Dollars]] = _at(SeriesPattern1, '*_capital_in_profit')
    in_loss: SeriesPattern1[Optional[Dollars]] = _at(SeriesPattern1, '*_capital_in_loss')


class AgeCreationSupply(_Node):
    total: Value = _at(Value, '*_supply')
    delta: Delta[Optional[Bitcoin]] = _at(Delta, '*_supply_delta')
    share: SeriesPattern1[Optional[Percent]] = _at(SeriesPattern1, '*_supply_share')
    in_profit: Value = _at(Value, '*_supply_in_profit')
    in_loss: Value = _at(Value, '*_supply_in_loss')
    matured: ValueFlow = _at(ValueFlow, '*_matured_supply')


class BlockCount(_Node):
    cumulative: SeriesPattern1[Count] = _at(SeriesPattern1, '*_cumulative')
    sum: Windows[Count] = _at(Windows, '*_sum')


class Pool(_Node):
    block_count: BlockCount = _at(BlockCount, '*_block_count')
    block_share: Share = _at(Share, '*_block_share')
    rewards: Rewards = _at(Rewards, '*_rewards')
    fees_per_block: OpReturnWindows = _at(OpReturnWindows, '*_fees_per_block')
    hashrate: Windows[Optional[Hashrate]] = _at(Windows, '*_hashrate')


class Pools(_Node):
    block_pool: SeriesPattern18[PoolSlug] = _at(SeriesPattern18, 'block_*')
    unknown: Pool = _at(Pool, '*_unknown')
    blockfills: Pool = _at(Pool, '*_blockfills')
    ultimuspool: Pool = _at(Pool, '*_ultimuspool')
    terrapool: Pool = _at(Pool, '*_terrapool')
    luxor: Pool = _at(Pool, '*_luxor')
    _1thash: Pool = _at(Pool, '*_1thash')
    btccom: Pool = _at(Pool, '*_btccom')
    bitfarms: Pool = _at(Pool, '*_bitfarms')
    huobipool: Pool = _at(Pool, '*_huobipool')
    wayicn: Pool = _at(Pool, '*_wayicn')
    canoepool: Pool = _at(Pool, '*_canoepool')
    btctop: Pool = _at(Pool, '*_btctop')
    bitcoincom: Pool = _at(Pool, '*_bitcoincom')
    _175btc: Pool = _at(Pool, '*_175btc')
    gbminers: Pool = _at(Pool, '*_gbminers')
    axbt: Pool = _at(Pool, '*_axbt')
    asicminer: Pool = _at(Pool, '*_asicminer')
    bitminter: Pool = _at(Pool, '*_bitminter')
    bitcoinrussia: Pool = _at(Pool, '*_bitcoinrussia')
    btcserv: Pool = _at(Pool, '*_btcserv')
    simplecoinus: Pool = _at(Pool, '*_simplecoinus')
    btcguild: Pool = _at(Pool, '*_btcguild')
    eligius: Pool = _at(Pool, '*_eligius')
    ozcoin: Pool = _at(Pool, '*_ozcoin')
    eclipsemc: Pool = _at(Pool, '*_eclipsemc')
    maxbtc: Pool = _at(Pool, '*_maxbtc')
    triplemining: Pool = _at(Pool, '*_triplemining')
    coinlab: Pool = _at(Pool, '*_coinlab')
    _50btc: Pool = _at(Pool, '*_50btc')
    ghashio: Pool = _at(Pool, '*_ghashio')
    stminingcorp: Pool = _at(Pool, '*_stminingcorp')
    bitparking: Pool = _at(Pool, '*_bitparking')
    mmpool: Pool = _at(Pool, '*_mmpool')
    polmine: Pool = _at(Pool, '*_polmine')
    kncminer: Pool = _at(Pool, '*_kncminer')
    bitalo: Pool = _at(Pool, '*_bitalo')
    f2pool: Pool = _at(Pool, '*_f2pool')
    hhtt: Pool = _at(Pool, '*_hhtt')
    megabigpower: Pool = _at(Pool, '*_megabigpower')
    mtred: Pool = _at(Pool, '*_mtred')
    nmcbit: Pool = _at(Pool, '*_nmcbit')
    yourbtcnet: Pool = _at(Pool, '*_yourbtcnet')
    givemecoins: Pool = _at(Pool, '*_givemecoins')
    braiinspool: Pool = _at(Pool, '*_braiinspool')
    antpool: Pool = _at(Pool, '*_antpool')
    multicoinco: Pool = _at(Pool, '*_multicoinco')
    bcpoolio: Pool = _at(Pool, '*_bcpoolio')
    cointerra: Pool = _at(Pool, '*_cointerra')
    kanopool: Pool = _at(Pool, '*_kanopool')
    solock: Pool = _at(Pool, '*_solock')
    ckpool: Pool = _at(Pool, '*_ckpool')
    nicehash: Pool = _at(Pool, '*_nicehash')
    bitclub: Pool = _at(Pool, '*_bitclub')
    bitcoinaffiliatenetwork: Pool = _at(Pool, '*_bitcoinaffiliatenetwork')
    btcc: Pool = _at(Pool, '*_btcc')
    bwpool: Pool = _at(Pool, '*_bwpool')
    exxbw: Pool = _at(Pool, '*_exxbw')
    bitsolo: Pool = _at(Pool, '*_bitsolo')
    bitfury: Pool = _at(Pool, '*_bitfury')
    _21inc: Pool = _at(Pool, '*_21inc')
    digitalbtc: Pool = _at(Pool, '*_digitalbtc')
    _8baochi: Pool = _at(Pool, '*_8baochi')
    mybtccoinpool: Pool = _at(Pool, '*_mybtccoinpool')
    tbdice: Pool = _at(Pool, '*_tbdice')
    hashpool: Pool = _at(Pool, '*_hashpool')
    nexious: Pool = _at(Pool, '*_nexious')
    bravomining: Pool = _at(Pool, '*_bravomining')
    hotpool: Pool = _at(Pool, '*_hotpool')
    okexpool: Pool = _at(Pool, '*_okexpool')
    bcmonster: Pool = _at(Pool, '*_bcmonster')
    _1hash: Pool = _at(Pool, '*_1hash')
    bixin: Pool = _at(Pool, '*_bixin')
    tatmaspool: Pool = _at(Pool, '*_tatmaspool')
    viabtc: Pool = _at(Pool, '*_viabtc')
    connectbtc: Pool = _at(Pool, '*_connectbtc')
    batpool: Pool = _at(Pool, '*_batpool')
    waterhole: Pool = _at(Pool, '*_waterhole')
    dcexploration: Pool = _at(Pool, '*_dcexploration')
    dcex: Pool = _at(Pool, '*_dcex')
    btpool: Pool = _at(Pool, '*_btpool')
    _58coin: Pool = _at(Pool, '*_58coin')
    bitcoinindia: Pool = _at(Pool, '*_bitcoinindia')
    shawnp0wers: Pool = _at(Pool, '*_shawnp0wers')
    phashio: Pool = _at(Pool, '*_phashio')
    rigpool: Pool = _at(Pool, '*_rigpool')
    haozhuzhu: Pool = _at(Pool, '*_haozhuzhu')
    _7pool: Pool = _at(Pool, '*_7pool')
    miningkings: Pool = _at(Pool, '*_miningkings')
    hashbx: Pool = _at(Pool, '*_hashbx')
    dpool: Pool = _at(Pool, '*_dpool')
    rawpool: Pool = _at(Pool, '*_rawpool')
    haominer: Pool = _at(Pool, '*_haominer')
    helix: Pool = _at(Pool, '*_helix')
    bitcoinukraine: Pool = _at(Pool, '*_bitcoinukraine')
    poolin: Pool = _at(Pool, '*_poolin')
    secretsuperstar: Pool = _at(Pool, '*_secretsuperstar')
    tigerpoolnet: Pool = _at(Pool, '*_tigerpoolnet')
    sigmapoolcom: Pool = _at(Pool, '*_sigmapoolcom')
    okpooltop: Pool = _at(Pool, '*_okpooltop')
    hummerpool: Pool = _at(Pool, '*_hummerpool')
    tangpool: Pool = _at(Pool, '*_tangpool')
    bytepool: Pool = _at(Pool, '*_bytepool')
    spiderpool: Pool = _at(Pool, '*_spiderpool')
    novablock: Pool = _at(Pool, '*_novablock')
    miningcity: Pool = _at(Pool, '*_miningcity')
    binancepool: Pool = _at(Pool, '*_binancepool')
    minerium: Pool = _at(Pool, '*_minerium')
    lubiancom: Pool = _at(Pool, '*_lubiancom')
    okkong: Pool = _at(Pool, '*_okkong')
    aaopool: Pool = _at(Pool, '*_aaopool')
    emcdpool: Pool = _at(Pool, '*_emcdpool')
    foundryusa: Pool = _at(Pool, '*_foundryusa')
    sbicrypto: Pool = _at(Pool, '*_sbicrypto')
    arkpool: Pool = _at(Pool, '*_arkpool')
    purebtccom: Pool = _at(Pool, '*_purebtccom')
    marapool: Pool = _at(Pool, '*_marapool')
    kucoinpool: Pool = _at(Pool, '*_kucoinpool')
    entrustcharitypool: Pool = _at(Pool, '*_entrustcharitypool')
    okminer: Pool = _at(Pool, '*_okminer')
    titan: Pool = _at(Pool, '*_titan')
    pegapool: Pool = _at(Pool, '*_pegapool')
    btcnuggets: Pool = _at(Pool, '*_btcnuggets')
    cloudhashing: Pool = _at(Pool, '*_cloudhashing')
    digitalxmintsy: Pool = _at(Pool, '*_digitalxmintsy')
    telco214: Pool = _at(Pool, '*_telco214')
    btcpoolparty: Pool = _at(Pool, '*_btcpoolparty')
    multipool: Pool = _at(Pool, '*_multipool')
    transactioncoinmining: Pool = _at(Pool, '*_transactioncoinmining')
    btcdig: Pool = _at(Pool, '*_btcdig')
    trickysbtcpool: Pool = _at(Pool, '*_trickysbtcpool')
    btcmp: Pool = _at(Pool, '*_btcmp')
    eobot: Pool = _at(Pool, '*_eobot')
    unomp: Pool = _at(Pool, '*_unomp')
    patels: Pool = _at(Pool, '*_patels')
    gogreenlight: Pool = _at(Pool, '*_gogreenlight')
    bitcoinindiapool: Pool = _at(Pool, '*_bitcoinindiapool')
    ekanembtc: Pool = _at(Pool, '*_ekanembtc')
    canoe: Pool = _at(Pool, '*_canoe')
    tiger: Pool = _at(Pool, '*_tiger')
    _1m1x: Pool = _at(Pool, '*_1m1x')
    zulupool: Pool = _at(Pool, '*_zulupool')
    secpool: Pool = _at(Pool, '*_secpool')
    ocean: Pool = _at(Pool, '*_ocean')
    whitepool: Pool = _at(Pool, '*_whitepool')
    wiz: Pool = _at(Pool, '*_wiz')
    wk057: Pool = _at(Pool, '*_wk057')
    futurebitapollosolo: Pool = _at(Pool, '*_futurebitapollosolo')
    carbonnegative: Pool = _at(Pool, '*_carbonnegative')
    portlandhodl: Pool = _at(Pool, '*_portlandhodl')
    phoenix: Pool = _at(Pool, '*_phoenix')
    neopool: Pool = _at(Pool, '*_neopool')
    maxipool: Pool = _at(Pool, '*_maxipool')
    bitfufupool: Pool = _at(Pool, '*_bitfufupool')
    gdpool: Pool = _at(Pool, '*_gdpool')
    miningdutch: Pool = _at(Pool, '*_miningdutch')
    publicpool: Pool = _at(Pool, '*_publicpool')
    miningsquared: Pool = _at(Pool, '*_miningsquared')
    innopolistech: Pool = _at(Pool, '*_innopolistech')
    btclab: Pool = _at(Pool, '*_btclab')
    parasite: Pool = _at(Pool, '*_parasite')
    redrockpool: Pool = _at(Pool, '*_redrockpool')
    est3lar: Pool = _at(Pool, '*_est3lar')
    braiinssolo: Pool = _at(Pool, '*_braiinssolo')
    solopoolcom: Pool = _at(Pool, '*_solopoolcom')
    noderunners: Pool = _at(Pool, '*_noderunners')
    dmnd: Pool = _at(Pool, '*_dmnd')


class DataBytesSeries(_Node):
    block: SeriesPattern18[Bytes] = _at(SeriesPattern18, '*_bytes')
    cumulative: SeriesPattern1[Bytes] = _at(SeriesPattern1, '*_bytes_cumulative')
    sum: Windows[Bytes] = _at(Windows, '*_bytes_sum')
    chain_share: Share = _at(Share, '*_chain_share')


class Inscription(_Node):
    flag: SeriesPattern19[Boolean] = _at(SeriesPattern19, 'has_*')
    block: SeriesPattern18[Count] = _at(SeriesPattern18, '*_tx_count')
    cumulative: SeriesPattern1[Count] = _at(SeriesPattern1, '*_tx_count_cumulative')
    sum: Windows[Count] = _at(Windows, '*_tx_count_sum')
    share: Share = _at(Share, '*_tx_share')
    fees: FeesSeries = _at(FeesSeries, '*')


class TransactionsFlagged(_Node):
    flag: SeriesPattern19[Boolean] = _at(SeriesPattern19, 'is_*')
    block: SeriesPattern18[Count] = _at(SeriesPattern18, '*_tx_count')
    cumulative: SeriesPattern1[Count] = _at(SeriesPattern1, '*_tx_count_cumulative')
    sum: Windows[Count] = _at(Windows, '*_tx_count_sum')
    share: Share = _at(Share, '*_tx_share')


class Patterns(_Node):
    coinjoin: TransactionsFlagged = _at(TransactionsFlagged, '*')
    consolidation: TransactionsFlagged = _at(TransactionsFlagged, 'consolidation')
    batch_payout: TransactionsFlagged = _at(TransactionsFlagged, 'batch_payout')


class Fees(_Node):
    fee: PerTxDistribution[Sats] = _at(PerTxDistribution, 'tx_*')
    fee_rate: SeriesPattern19[Optional[FeeRate]] = _at(SeriesPattern19, '*_rate')
    effective_fee_rate: PerTxDistribution[Optional[FeeRate]] = _at(PerTxDistribution, 'effective_*_rate')
    cpfp_parent: TransactionsFlagged = _at(TransactionsFlagged, 'cpfp_parent')
    cpfp_child: TransactionsFlagged = _at(TransactionsFlagged, 'cpfp_child')


class TxCount(_Node):
    block: SeriesPattern18[Count] = _at(SeriesPattern18, '*_count')
    cumulative: SeriesPattern1[Count] = _at(SeriesPattern1, '*_count_cumulative')
    sum: Windows[Count] = _at(Windows, '*_count_sum')
    share: Share = _at(Share, '*_share')


class Flagged(_Node):
    flag: SeriesPattern19[Boolean] = _at(SeriesPattern19, 'has_*')
    block: SeriesPattern18[Count] = _at(SeriesPattern18, '*_tx_count')
    cumulative: SeriesPattern1[Count] = _at(SeriesPattern1, '*_tx_count_cumulative')
    sum: Windows[Count] = _at(Windows, '*_tx_count_sum')
    share: Share = _at(Share, '*_tx_share')


class Signatures(_Node):
    annex: Flagged = _at(Flagged, 'annex')
    sighash_all: Flagged = _at(Flagged, '*_all')
    sighash_none: Flagged = _at(Flagged, '*_none')
    sighash_single: Flagged = _at(Flagged, '*_single')
    sighash_default: Flagged = _at(Flagged, '*_default')
    sighash_anyone_can_pay: Flagged = _at(Flagged, '*_anyone_can_pay')


class TransactionsScripts(_Node):
    p2pk: Flagged = _at(Flagged, 'p2pk')
    p2ms: Flagged = _at(Flagged, 'p2ms')
    p2pkh: Flagged = _at(Flagged, 'p2pkh')
    p2sh: Flagged = _at(Flagged, 'p2sh')
    p2wpkh: Flagged = _at(Flagged, 'p2wpkh')
    p2wsh: Flagged = _at(Flagged, 'p2wsh')
    p2tr: Flagged = _at(Flagged, 'p2tr')
    p2a: Flagged = _at(Flagged, 'p2a')
    empty_script: Flagged = _at(Flagged, 'empty_*')
    unknown_script: Flagged = _at(Flagged, 'unknown_*')
    fake_pubkey: Flagged = _at(Flagged, 'fake_pubkey')
    fake_scripthash: Flagged = _at(Flagged, 'fake_scripthash')


class Features(_Node):
    scripts: TransactionsScripts = _at(TransactionsScripts, 'script')
    signatures: Signatures = _at(Signatures, 'sighash')
    segwit: TxCount = _at(TxCount, 'segwit_*')
    explicitly_rbf: TransactionsFlagged = _at(TransactionsFlagged, 'explicitly_rbf')
    dust_output: Flagged = _at(Flagged, 'dust_output')
    epobc: TransactionsFlagged = _at(TransactionsFlagged, 'epobc')
    one_input: TxCount = _at(TxCount, 'one_input_*')
    one_output: TxCount = _at(TxCount, 'one_output_*')


class FlowStats(_Node, Generic[A, B]):
    block: SeriesPattern18[A] = _at(SeriesPattern18, '*')
    cumulative: SeriesPattern1[A] = _at(SeriesPattern1, '*_cumulative')
    sum: Windows[A] = _at(Windows, '*_sum')
    avg: Windows[B] = _at(Windows, '*_avg')
    min: Windows[A] = _at(Windows, '*_min')
    max: Windows[A] = _at(Windows, '*_max')
    pct10: Windows[A] = _at(Windows, '*_pct10')
    pct25: Windows[A] = _at(Windows, '*_pct25')
    median: Windows[A] = _at(Windows, '*_median')
    pct75: Windows[A] = _at(Windows, '*_pct75')
    pct90: Windows[A] = _at(Windows, '*_pct90')


class Averaged(_Node, Generic[A, B]):
    block: SeriesPattern18[A] = _at(SeriesPattern18, '*')
    avg: Windows[B] = _at(Windows, '*_avg')


class AddressesActivity(_Node):
    reactivated: Averaged[Count, Optional[CountFract]] = _at(Averaged, '*_reactivated_address_count')
    sending: Averaged[Count, Optional[CountFract]] = _at(Averaged, '*_sending_address_count')
    receiving: Averaged[Count, Optional[CountFract]] = _at(Averaged, '*_receiving_address_count')
    bidirectional: Averaged[Count, Optional[CountFract]] = _at(Averaged, '*_bidirectional_address_count')
    active: Averaged[Count, Optional[CountFract]] = _at(Averaged, '*_active_address_count')


class Flow(_Node, Generic[A]):
    block: SeriesPattern18[A] = _at(SeriesPattern18, '*')
    cumulative: SeriesPattern1[A] = _at(SeriesPattern1, '*_cumulative')
    sum: Windows[A] = _at(Windows, '*_sum')


class ReserveRisk(_Node):
    block: SeriesPattern1[Optional[Float]] = _at(SeriesPattern1, '*')
    hodl_bank: SeriesPattern1[Optional[Float]] = _at(SeriesPattern1, 'hodl_bank')
    vocdd: Flow[Optional[DollarDaysPerCoin]] = _at(Flow, 'vocdd')


class CointimeValue(_Node):
    destroyed: Flow[Optional[DollarBlocks]] = _at(Flow, '*_destroyed')
    created: Flow[Optional[DollarBlocks]] = _at(Flow, '*_created')
    stored: Flow[Optional[DollarBlocks]] = _at(Flow, '*_stored')


class CointimeRange(_Node):
    coindays_created: Flow[Optional[CoinDays]] = _at(Flow, '*_coindays_created')
    coindays_consumed: Flow[Optional[CoinDays]] = _at(Flow, '*_coindays_consumed')
    wakefulness: SeriesPattern1[Optional[Ratio]] = _at(SeriesPattern1, '*_wakefulness')
    awake_to_dormant: SeriesPattern1[Optional[Multiple]] = _at(SeriesPattern1, '*_awake_to_dormant')
    awake: Side = _at(Side, '*_awake_supply')
    dormant: Side = _at(Side, '*_dormant_supply')


class CointimeActivity(_Node):
    coinblocks_created: Flow[Optional[CoinBlocks]] = _at(Flow, '*_created')
    coinblocks_destroyed: Flow[Optional[CoinBlocks]] = _at(Flow, '*_destroyed')
    coinblocks_stored: Flow[Optional[CoinBlocks]] = _at(Flow, '*_stored')
    liveliness: SeriesPattern1[Optional[Ratio]] = _at(SeriesPattern1, 'liveliness')
    vaultedness: SeriesPattern1[Optional[Ratio]] = _at(SeriesPattern1, 'vaultedness')
    liveliness_to_vaultedness: SeriesPattern1[Optional[Multiple]] = _at(SeriesPattern1, 'liveliness_to_vaultedness')
    concurrent_liveliness: Windows[Optional[Ratio]] = _at(Windows, 'concurrent_liveliness')


class AdjustedSopr(_Node):
    _24h: SeriesPattern1[Optional[Multiple]] = _at(SeriesPattern1, '*_sopr_24h')
    _1w: SeriesPattern1[Optional[Multiple]] = _at(SeriesPattern1, '*_sopr_1w')
    _1m: SeriesPattern1[Optional[Multiple]] = _at(SeriesPattern1, '*_sopr_1m')
    _1y: SeriesPattern1[Optional[Multiple]] = _at(SeriesPattern1, '*_sopr_1y')
    value_created: Flow[Optional[Dollars]] = _at(Flow, '*_value_created')
    value_destroyed: Flow[Optional[Dollars]] = _at(Flow, '*_value_destroyed')


class HoldersRealized(_Node):
    profit: Flow[Optional[Dollars]] = _at(Flow, '*_realized_profit')
    loss: Flow[Optional[Dollars]] = _at(Flow, '*_realized_loss')
    net_pnl: Flow[Optional[Dollars]] = _at(Flow, '*_net_realized_pnl')
    gross_pnl: Flow[Optional[Dollars]] = _at(Flow, '*_gross_realized_pnl')
    profit_to_loss: Windows[Optional[Multiple]] = _at(Windows, '*_realized_profit_to_loss')
    value_destroyed: Flow[Optional[Dollars]] = _at(Flow, '*_value_destroyed')
    peak_regret: Flow[Optional[Dollars]] = _at(Flow, '*_realized_peak_regret')
    sopr: Windows[Optional[Multiple]] = _at(Windows, '*_sopr')
    adjusted_sopr: AdjustedSopr = _at(AdjustedSopr, '*_adjusted')
    sell_side_risk_ratio: Windows[Optional[Ratio]] = _at(Windows, '*_sell_side_risk_ratio')


class HoldersActivity(_Node):
    transfer_volume: HoldersValueFlow = _at(HoldersValueFlow, '*_transfer_volume')
    coindays_destroyed: Flow[Optional[CoinDays]] = _at(Flow, '*_coindays_destroyed')
    dormancy: Windows[Optional[Days]] = _at(Windows, '*_dormancy')


class EventCount(_Node, Generic[A]):
    count: A = _at(0, '*_count')
    share: Share = _at(Share, '*_share')


class ReuseEvents(_Node):
    outputs: EventCount[Flow[Count]] = _at((EventCount, Flow), '*_output_to_respent_address')
    inputs: EventCount[Flow[Count]] = _at((EventCount, Flow), '*_input_from_respent_address')
    active: EventCount[Averaged[Count, Optional[CountFract]]] = _at((EventCount, Averaged), '*_active_respent_address')


class Reuse(_Node):
    count: RespentCount = _at(RespentCount, '*')
    supply: PredicateSupply = _at(PredicateSupply, '*_respent_address_supply')
    events: ReuseEvents = _at(ReuseEvents, '*')


class AddressesReuseEvents(_Node):
    outputs: EventCount[Flow[Count]] = _at((EventCount, Flow), '*_output_to_reused_address')
    inputs: EventCount[Flow[Count]] = _at((EventCount, Flow), '*_input_from_reused_address')
    active: EventCount[Averaged[Count, Optional[CountFract]]] = _at((EventCount, Averaged), '*_active_reused_address')


class AddressesReuse(_Node):
    count: ReusedCount = _at(ReusedCount, '*')
    supply: PredicateSupply = _at(PredicateSupply, '*_reused_address_supply')
    events: AddressesReuseEvents = _at(AddressesReuseEvents, '*')


class AddressSeries(_Node):
    funded: WithDelta[Count, CountSigned] = _at(WithDelta, '*_funded_address_count')
    empty: SeriesPattern1[Count] = _at(SeriesPattern1, '*_empty_address_count')
    total: SeriesPattern1[Count] = _at(SeriesPattern1, '*_total_address_count')
    new: NewAddressCount = _at(NewAddressCount, '*_new_address_count')
    activity: AddressesActivity = _at(AddressesActivity, '*')
    avg_balance: Value = _at(Value, '*_address_balance_avg')
    reused: AddressesReuse = _at(AddressesReuse, '*')
    respent: Reuse = _at(Reuse, '*')
    exposed: Exposed = _at(Exposed, '*')


class AddressType(_Node):
    p2pk: AddressSeries = _at(AddressSeries, '*')
    p2pkh: AddressSeries = _at(AddressSeries, 'p2pkh')
    p2sh: AddressSeries = _at(AddressSeries, 'p2sh')
    p2wpkh: AddressSeries = _at(AddressSeries, 'p2wpkh')
    p2wsh: AddressSeries = _at(AddressSeries, 'p2wsh')
    p2tr: AddressSeries = _at(AddressSeries, 'p2tr')


class Realized(_Node):
    profit: Flow[Optional[Dollars]] = _at(Flow, '*_profit')
    loss: Flow[Optional[Dollars]] = _at(Flow, '*_loss')


class Balance(_Node):
    funded: WithDelta[Count, CountSigned] = _at(WithDelta, '*_address_count')
    supply: CohortSupply = _at(CohortSupply, '*_supply')
    capital: CapitalViews = _at(CapitalViews, '*')
    utxos: BalanceUtxos = _at(BalanceUtxos, '*_utxo_count')
    activity: Activity = _at(Activity, '*_transfer_volume')
    realized: Realized = _at(Realized, '*_realized')


class CreationActivity(_Node):
    transfer_volume: ValueFlow = _at(ValueFlow, '*_transfer_volume')
    coindays_destroyed: Flow[Optional[CoinDays]] = _at(Flow, '*_coindays_destroyed')


class CreationRealized(_Node):
    profit: Flow[Optional[Dollars]] = _at(Flow, '*_realized_profit')
    loss: Flow[Optional[Dollars]] = _at(Flow, '*_realized_loss')
    net_pnl: Flow[Optional[Dollars]] = _at(Flow, '*_net_realized_pnl')
    value_destroyed: Flow[Optional[Dollars]] = _at(Flow, '*_value_destroyed')


class AgeCreationActivity(_Node):
    transfer_volume: HoldersValueFlow = _at(HoldersValueFlow, '*_transfer_volume')
    coindays_destroyed: Flow[Optional[CoinDays]] = _at(Flow, '*_coindays_destroyed')
    coindays_created: Flow[Optional[CoinDays]] = _at(Flow, '*_coindays_created')


class CohortUtxos(_Node):
    count: WithDelta[Count, CountSigned] = _at(WithDelta, '*_utxo_count')
    spent_count: Flow[Count] = _at(Flow, '*_spent_utxo_count')


class CohortSeries(_Node):
    supply: CohortSupply = _at(CohortSupply, '*_supply')
    capital: CapitalViews = _at(CapitalViews, '*')
    utxos: CohortUtxos = _at(CohortUtxos, '*')
    activity: Activity = _at(Activity, '*_transfer_volume')
    realized: Realized = _at(Realized, '*_realized')
    cost_basis: CohortCostBasis = _at(CohortCostBasis, '*')


class UtxosSpendableType(_Node):
    p2pk: CohortSeries = _at(CohortSeries, 'p2pk')
    p2pkh: CohortSeries = _at(CohortSeries, 'p2pkh')
    p2ms: CohortSeries = _at(CohortSeries, 'p2ms')
    p2sh: CohortSeries = _at(CohortSeries, 'p2sh')
    p2wpkh: CohortSeries = _at(CohortSeries, 'p2wpkh')
    p2wsh: CohortSeries = _at(CohortSeries, 'p2wsh')
    p2tr: CohortSeries = _at(CohortSeries, 'p2tr')
    p2a: CohortSeries = _at(CohortSeries, 'p2a')
    unknown: CohortSeries = _at(CohortSeries, 'unknown_*')
    empty: CohortSeries = _at(CohortSeries, 'empty_*')


class Zero(_Node):
    utxos: CohortUtxos = _at(CohortUtxos, '*')


class AmountRange(_Node, Generic[A, B]):
    _0_sats: A = _at(0, '*_0_sats')
    _1_to_10_sats: B = _at(1, '*_1_to_10_sats')
    _10_to_100_sats: B = _at(1, '*_10_to_100_sats')
    _100_to_1k_sats: B = _at(1, '*_100_to_1k_sats')
    _1k_to_10k_sats: B = _at(1, '*_1k_to_10k_sats')
    _10k_to_100k_sats: B = _at(1, '*_10k_to_100k_sats')
    _100k_to_1m_sats: B = _at(1, '*_100k_to_1m_sats')
    _1m_to_10m_sats: B = _at(1, '*_1m_to_10m_sats')
    _10m_to_100m_sats: B = _at(1, '*_10m_to_100m_sats')
    _1_to_10_btc: B = _at(1, '*_1_to_10_btc')
    _10_to_100_btc: B = _at(1, '*_10_to_100_btc')
    _100_to_1k_btc: B = _at(1, '*_100_to_1k_btc')
    _1k_to_10k_btc: B = _at(1, '*_1k_to_10k_btc')
    _10k_to_100k_btc: B = _at(1, '*_10k_to_100k_btc')
    over_100k_btc: B = _at(1, '*_over_100k_btc')


class Addresses(_Node):
    funded: WithDelta[Count, CountSigned] = _at(WithDelta, 'funded_*_count')
    empty: SeriesPattern1[Count] = _at(SeriesPattern1, 'empty_*_count')
    total: SeriesPattern1[Count] = _at(SeriesPattern1, 'total_*_count')
    new: NewAddressCount = _at(NewAddressCount, 'new_*_count')
    activity: AddressesActivity = _at(AddressesActivity, '')
    avg_balance: Value = _at(Value, '*_balance_avg')
    reused: AddressesReuse = _at(AddressesReuse, '')
    respent: Reuse = _at(Reuse, '')
    exposed: Exposed = _at(Exposed, '')
    supply: CohortSupply = _at(CohortSupply, '*_supply')
    capital: CapitalViews = _at(CapitalViews, '*')
    types: AddressType = _at(AddressType, 'p2pk')
    balances: AmountRange[ZeroBalance, Balance] = _at((AmountRange, ZeroBalance, Balance), 'balance')


class Utxos(_Node):
    amounts: AmountRange[Zero, CohortSeries] = _at((AmountRange, Zero, CohortSeries), '*')
    types: UtxosSpendableType = _at(UtxosSpendableType, 'output')


class CreationCohort(_Node, Generic[A, B, C, D, E, F]):
    supply: A = _at(0, '*_supply')
    capital: B = _at(1, '*')
    utxos: CohortUtxos = _at(CohortUtxos, '*')
    activity: C = _at(2, '*')
    realized: D = _at(3, '*')
    unrealized: E = _at(4, '*')
    cost_basis: F = _at(5, '*')


class Entry(_Node):
    veteran: CreationCohort[CreationSupply, CreationCapital, CreationActivity, CreationRealized, CreationUnrealized, CohortCostBasis] = _at((CreationCohort, CreationSupply, CreationCapital, CreationActivity, CreationRealized, CreationUnrealized, CohortCostBasis), '*')
    rookie: CreationCohort[CreationSupply, CreationCapital, CreationActivity, CreationRealized, CreationUnrealized, CohortCostBasis] = _at((CreationCohort, CreationSupply, CreationCapital, CreationActivity, CreationRealized, CreationUnrealized, CohortCostBasis), 'rookie')


class Holders(_Node, Generic[A, B]):
    all: A = _at(0, '')
    sth: B = _at(1, 'sth')
    lth: B = _at(1, 'lth')
    under_4m: B = _at(1, '*_under_4m')
    under_6m: B = _at(1, '*_under_6m')
    over_4m: B = _at(1, '*_over_4m')
    over_6m: B = _at(1, '*_over_6m')


class Vintage(_Node):
    _2009: CreationCohort[CreationSupply, CreationCapital, CreationActivity, CreationRealized, CreationUnrealized, CohortCostBasis] = _at((CreationCohort, CreationSupply, CreationCapital, CreationActivity, CreationRealized, CreationUnrealized, CohortCostBasis), '*_2009')
    _2010: CreationCohort[CreationSupply, CreationCapital, CreationActivity, CreationRealized, CreationUnrealized, CohortCostBasis] = _at((CreationCohort, CreationSupply, CreationCapital, CreationActivity, CreationRealized, CreationUnrealized, CohortCostBasis), '*_2010')
    _2011: CreationCohort[CreationSupply, CreationCapital, CreationActivity, CreationRealized, CreationUnrealized, CohortCostBasis] = _at((CreationCohort, CreationSupply, CreationCapital, CreationActivity, CreationRealized, CreationUnrealized, CohortCostBasis), '*_2011')
    _2012: CreationCohort[CreationSupply, CreationCapital, CreationActivity, CreationRealized, CreationUnrealized, CohortCostBasis] = _at((CreationCohort, CreationSupply, CreationCapital, CreationActivity, CreationRealized, CreationUnrealized, CohortCostBasis), '*_2012')
    _2013: CreationCohort[CreationSupply, CreationCapital, CreationActivity, CreationRealized, CreationUnrealized, CohortCostBasis] = _at((CreationCohort, CreationSupply, CreationCapital, CreationActivity, CreationRealized, CreationUnrealized, CohortCostBasis), '*_2013')
    _2014: CreationCohort[CreationSupply, CreationCapital, CreationActivity, CreationRealized, CreationUnrealized, CohortCostBasis] = _at((CreationCohort, CreationSupply, CreationCapital, CreationActivity, CreationRealized, CreationUnrealized, CohortCostBasis), '*_2014')
    _2015: CreationCohort[CreationSupply, CreationCapital, CreationActivity, CreationRealized, CreationUnrealized, CohortCostBasis] = _at((CreationCohort, CreationSupply, CreationCapital, CreationActivity, CreationRealized, CreationUnrealized, CohortCostBasis), '*_2015')
    _2016: CreationCohort[CreationSupply, CreationCapital, CreationActivity, CreationRealized, CreationUnrealized, CohortCostBasis] = _at((CreationCohort, CreationSupply, CreationCapital, CreationActivity, CreationRealized, CreationUnrealized, CohortCostBasis), '*_2016')
    _2017: CreationCohort[CreationSupply, CreationCapital, CreationActivity, CreationRealized, CreationUnrealized, CohortCostBasis] = _at((CreationCohort, CreationSupply, CreationCapital, CreationActivity, CreationRealized, CreationUnrealized, CohortCostBasis), '*_2017')
    _2018: CreationCohort[CreationSupply, CreationCapital, CreationActivity, CreationRealized, CreationUnrealized, CohortCostBasis] = _at((CreationCohort, CreationSupply, CreationCapital, CreationActivity, CreationRealized, CreationUnrealized, CohortCostBasis), '*_2018')
    _2019: CreationCohort[CreationSupply, CreationCapital, CreationActivity, CreationRealized, CreationUnrealized, CohortCostBasis] = _at((CreationCohort, CreationSupply, CreationCapital, CreationActivity, CreationRealized, CreationUnrealized, CohortCostBasis), '*_2019')
    _2020: CreationCohort[CreationSupply, CreationCapital, CreationActivity, CreationRealized, CreationUnrealized, CohortCostBasis] = _at((CreationCohort, CreationSupply, CreationCapital, CreationActivity, CreationRealized, CreationUnrealized, CohortCostBasis), '*_2020')
    _2021: CreationCohort[CreationSupply, CreationCapital, CreationActivity, CreationRealized, CreationUnrealized, CohortCostBasis] = _at((CreationCohort, CreationSupply, CreationCapital, CreationActivity, CreationRealized, CreationUnrealized, CohortCostBasis), '*_2021')
    _2022: CreationCohort[CreationSupply, CreationCapital, CreationActivity, CreationRealized, CreationUnrealized, CohortCostBasis] = _at((CreationCohort, CreationSupply, CreationCapital, CreationActivity, CreationRealized, CreationUnrealized, CohortCostBasis), '*_2022')
    _2023: CreationCohort[CreationSupply, CreationCapital, CreationActivity, CreationRealized, CreationUnrealized, CohortCostBasis] = _at((CreationCohort, CreationSupply, CreationCapital, CreationActivity, CreationRealized, CreationUnrealized, CohortCostBasis), '*_2023')
    _2024: CreationCohort[CreationSupply, CreationCapital, CreationActivity, CreationRealized, CreationUnrealized, CohortCostBasis] = _at((CreationCohort, CreationSupply, CreationCapital, CreationActivity, CreationRealized, CreationUnrealized, CohortCostBasis), '*_2024')
    _2025: CreationCohort[CreationSupply, CreationCapital, CreationActivity, CreationRealized, CreationUnrealized, CohortCostBasis] = _at((CreationCohort, CreationSupply, CreationCapital, CreationActivity, CreationRealized, CreationUnrealized, CohortCostBasis), '*_2025')
    _2026: CreationCohort[CreationSupply, CreationCapital, CreationActivity, CreationRealized, CreationUnrealized, CohortCostBasis] = _at((CreationCohort, CreationSupply, CreationCapital, CreationActivity, CreationRealized, CreationUnrealized, CohortCostBasis), '*_2026')
    _2027: CreationCohort[CreationSupply, CreationCapital, CreationActivity, CreationRealized, CreationUnrealized, CohortCostBasis] = _at((CreationCohort, CreationSupply, CreationCapital, CreationActivity, CreationRealized, CreationUnrealized, CohortCostBasis), '*_2027')


class ByHalvingEpoch(_Node):
    _0: CreationCohort[CreationSupply, CreationCapital, CreationActivity, CreationRealized, CreationUnrealized, CohortCostBasis] = _at((CreationCohort, CreationSupply, CreationCapital, CreationActivity, CreationRealized, CreationUnrealized, CohortCostBasis), '*_0')
    _1: CreationCohort[CreationSupply, CreationCapital, CreationActivity, CreationRealized, CreationUnrealized, CohortCostBasis] = _at((CreationCohort, CreationSupply, CreationCapital, CreationActivity, CreationRealized, CreationUnrealized, CohortCostBasis), '*_1')
    _2: CreationCohort[CreationSupply, CreationCapital, CreationActivity, CreationRealized, CreationUnrealized, CohortCostBasis] = _at((CreationCohort, CreationSupply, CreationCapital, CreationActivity, CreationRealized, CreationUnrealized, CohortCostBasis), '*_2')
    _3: CreationCohort[CreationSupply, CreationCapital, CreationActivity, CreationRealized, CreationUnrealized, CohortCostBasis] = _at((CreationCohort, CreationSupply, CreationCapital, CreationActivity, CreationRealized, CreationUnrealized, CohortCostBasis), '*_3')
    _4: CreationCohort[CreationSupply, CreationCapital, CreationActivity, CreationRealized, CreationUnrealized, CohortCostBasis] = _at((CreationCohort, CreationSupply, CreationCapital, CreationActivity, CreationRealized, CreationUnrealized, CohortCostBasis), '*_4')
    _5: CreationCohort[CreationSupply, CreationCapital, CreationActivity, CreationRealized, CreationUnrealized, CohortCostBasis] = _at((CreationCohort, CreationSupply, CreationCapital, CreationActivity, CreationRealized, CreationUnrealized, CohortCostBasis), '*_5')


class Range(_Node):
    supply: AgeCreationSupply = _at(AgeCreationSupply, '*')
    capital: CreationCapital = _at(CreationCapital, '*')
    utxos: CohortUtxos = _at(CohortUtxos, '*')
    activity: AgeCreationActivity = _at(AgeCreationActivity, '*')
    realized: CreationRealized = _at(CreationRealized, '*')
    unrealized: CreationUnrealized = _at(CreationUnrealized, '*')
    cost_basis: CohortCostBasis = _at(CohortCostBasis, '*')


class AgeRange(_Node, Generic[A]):
    under_1h: A = _at(0, '*_under_1h')
    _1h_to_1d: A = _at(0, '*_1h_to_1d')
    _1d_to_1w: A = _at(0, '*_1d_to_1w')
    _1w_to_1m: A = _at(0, '*_1w_to_1m')
    _1m_to_2m: A = _at(0, '*_1m_to_2m')
    _2m_to_3m: A = _at(0, '*_2m_to_3m')
    _3m_to_4m: A = _at(0, '*_3m_to_4m')
    _4m_to_5m: A = _at(0, '*_4m_to_5m')
    _5m_to_6m: A = _at(0, '*_5m_to_6m')
    _6m_to_9m: A = _at(0, '*_6m_to_9m')
    _9m_to_1y: A = _at(0, '*_9m_to_1y')
    _1y_to_18m: A = _at(0, '*_1y_to_18m')
    _18m_to_2y: A = _at(0, '*_18m_to_2y')
    _2y_to_3y: A = _at(0, '*_2y_to_3y')
    _3y_to_4y: A = _at(0, '*_3y_to_4y')
    _4y_to_5y: A = _at(0, '*_4y_to_5y')
    _5y_to_6y: A = _at(0, '*_5y_to_6y')
    _6y_to_7y: A = _at(0, '*_6y_to_7y')
    _7y_to_8y: A = _at(0, '*_7y_to_8y')
    _8y_to_10y: A = _at(0, '*_8y_to_10y')
    _10y_to_12y: A = _at(0, '*_10y_to_12y')
    _12y_to_15y: A = _at(0, '*_12y_to_15y')
    over_15y: A = _at(0, '*_over_15y')


class Coinflow(_Node):
    age_ranges: AgeRange[CoinflowRange] = _at((AgeRange, CoinflowRange), '*')
    cohorts: Holders[CoinflowCohortSeries, CoinflowCohortSeries] = _at((Holders, CoinflowCohortSeries, CoinflowCohortSeries), '*')


class Cointime(_Node):
    activity: CointimeActivity = _at(CointimeActivity, 'coinblocks')
    age_ranges: AgeRange[CointimeRange] = _at((AgeRange, CointimeRange), '*')
    cohorts: Holders[CointimeCohortSeries, CointimeCohortSeries] = _at((Holders, CointimeCohortSeries, CointimeCohortSeries), '*')
    supply: CointimeSupply = _at(CointimeSupply, 'supply')
    value: CointimeValue = _at(CointimeValue, 'cointime_value')
    caps: Caps = _at(Caps, 'cap')
    prices: Prices = _at(Prices, 'vaulted')
    adjusted: Adjusted = _at(Adjusted, 'cointime_adjusted')
    reserve_risk: ReserveRisk = _at(ReserveRisk, 'reserve_risk')


class Age(_Node):
    ranges: AgeRange[Range] = _at((AgeRange, Range), '*')
    halving_epochs: ByHalvingEpoch = _at(ByHalvingEpoch, 'halving_epoch')
    vintages: Vintage = _at(Vintage, 'vintage')


class Bucket(_Node):
    output_count: Flow[Count] = _at(Flow, '*_output_count')
    data_bytes: DataBytesSeries = _at(DataBytesSeries, '*_data')
    tx_count: Flow[Count] = _at(Flow, '*_tx_count')
    tx_share: Share = _at(Share, '*_tx_share')
    tx_vsize: Flow[VSize] = _at(Flow, '*_tx_vsize')
    fees: FeesSeries = _at(FeesSeries, '*')


class Policy(_Node):
    pre_v30_standard: Bucket = _at(Bucket, '*_pre_v30_standard')
    pre_v30_nonstandard: Bucket = _at(Bucket, '*_pre_v30_nonstandard')
    oversized: Bucket = _at(Bucket, '*_oversized')
    multiple: Bucket = _at(Bucket, '*_multiple')


class ByKind(_Node):
    runes: Bucket = _at(Bucket, '*_runes')
    witness_commitment: Bucket = _at(Bucket, '*_witness_commitment')
    veri_block: Bucket = _at(Bucket, '*_veri_block')
    omni: Bucket = _at(Bucket, '*_omni')
    stacks: Bucket = _at(Bucket, '*_stacks')
    blockstack: Bucket = _at(Bucket, '*_blockstack')
    colu: Bucket = _at(Bucket, '*_colu')
    open_assets: Bucket = _at(Bucket, '*_open_assets')
    komodo: Bucket = _at(Bucket, '*_komodo')
    coin_spark: Bucket = _at(Bucket, '*_coin_spark')
    poet: Bucket = _at(Bucket, '*_poet')
    docproof: Bucket = _at(Bucket, '*_docproof')
    factom: Bucket = _at(Bucket, '*_factom')
    eternity_wall: Bucket = _at(Bucket, '*_eternity_wall')
    memo: Bucket = _at(Bucket, '*_memo')
    bitproof: Bucket = _at(Bucket, '*_bitproof')
    ascribe: Bucket = _at(Bucket, '*_ascribe')
    stampery: Bucket = _at(Bucket, '*_stampery')
    bare_hash: Bucket = _at(Bucket, '*_bare_hash')
    text: Bucket = _at(Bucket, '*_text')
    empty: Bucket = _at(Bucket, '*_empty')
    unknown: Bucket = _at(Bucket, '*_unknown')


class OpReturn(_Node):
    flag: SeriesPattern19[Boolean] = _at(SeriesPattern19, 'has_*')
    output_count: Flow[Count] = _at(Flow, '*_output_count')
    data_bytes: DataBytesSeries = _at(DataBytesSeries, '*_data')
    tx_count: Flow[Count] = _at(Flow, '*_tx_count')
    tx_share: Share = _at(Share, '*_tx_share')
    tx_vsize: Flow[VSize] = _at(Flow, '*_tx_vsize')
    fees: FeesSeries = _at(FeesSeries, '*')
    protocols: ByKind = _at(ByKind, '*')
    policies: Policy = _at(Policy, '*')


class OutputTypeSeries(_Node):
    count: Flow[Count] = _at(Flow, '*_count')
    share: Share = _at(Share, '*_share')
    tx_count: Flow[Count] = _at(Flow, '*_tx_count')
    tx_share: Share = _at(Share, '*_tx_share')


class ByType(_Node):
    p2pk: OutputTypeSeries = _at(OutputTypeSeries, 'p2pk_*')
    p2pkh: OutputTypeSeries = _at(OutputTypeSeries, 'p2pkh_*')
    p2ms: OutputTypeSeries = _at(OutputTypeSeries, 'p2ms_*')
    p2sh: OutputTypeSeries = _at(OutputTypeSeries, 'p2sh_*')
    p2wpkh: OutputTypeSeries = _at(OutputTypeSeries, 'p2wpkh_*')
    p2wsh: OutputTypeSeries = _at(OutputTypeSeries, 'p2wsh_*')
    p2tr: OutputTypeSeries = _at(OutputTypeSeries, 'p2tr_*')
    p2a: OutputTypeSeries = _at(OutputTypeSeries, 'p2a_*')
    unknown: OutputTypeSeries = _at(OutputTypeSeries, 'unknown_*')
    empty: OutputTypeSeries = _at(OutputTypeSeries, 'empty_*')
    op_return: OutputTypeSeries = _at(OutputTypeSeries, 'op_return_*')


class Outputs(_Node):
    spending_txin_index: SeriesPattern23[TxInIndex] = _at(SeriesPattern23, 'spending_txin_index')
    count: FlowStats[Count, Optional[CountFract]] = _at(FlowStats, '*_count')
    per_second: Windows[Optional[PerSecond]] = _at(Windows, 'outputs_per_second')
    spendable_count: Flow[Count] = _at(Flow, 'spendable_*_count')
    types: ByType = _at(ByType, '*')
    op_return_value: ValueFlow = _at(ValueFlow, 'op_return_value')


class SpendableType(_Node):
    p2pk: OutputTypeSeries = _at(OutputTypeSeries, 'p2pk_*')
    p2pkh: OutputTypeSeries = _at(OutputTypeSeries, 'p2pkh_*')
    p2ms: OutputTypeSeries = _at(OutputTypeSeries, 'p2ms_*')
    p2sh: OutputTypeSeries = _at(OutputTypeSeries, 'p2sh_*')
    p2wpkh: OutputTypeSeries = _at(OutputTypeSeries, 'p2wpkh_*')
    p2wsh: OutputTypeSeries = _at(OutputTypeSeries, 'p2wsh_*')
    p2tr: OutputTypeSeries = _at(OutputTypeSeries, 'p2tr_*')
    p2a: OutputTypeSeries = _at(OutputTypeSeries, 'p2a_*')
    unknown: OutputTypeSeries = _at(OutputTypeSeries, 'unknown_*')
    empty: OutputTypeSeries = _at(OutputTypeSeries, 'empty_*')


class Inputs(_Node):
    count: FlowStats[Count, Optional[CountFract]] = _at(FlowStats, '*_count')
    per_second: Windows[Optional[PerSecond]] = _at(Windows, 'inputs_per_second')
    types: SpendableType = _at(SpendableType, '*')


class Blocks(_Node):
    count: Flow[Count] = _at(Flow, '*_count')
    interval: Averaged[Seconds, Optional[SecondsFract]] = _at(Averaged, '*_interval')
    size: FlowStats[Bytes, Optional[BytesFract]] = _at(FlowStats, '*_size')
    weight: FlowStats[Weight, Optional[WeightFract]] = _at(FlowStats, '*_weight')
    fullness: SeriesPattern18[Optional[Percent]] = _at(SeriesPattern18, '*_fullness')
    difficulty: BlocksDifficulty = _at(BlocksDifficulty, 'difficulty')
    halving: BlocksHalving = _at(BlocksHalving, 'halving')


class Price(_Node):
    open: SeriesPattern2[Optional[Dollars]] = _at(SeriesPattern2, '*_open')
    high: SeriesPattern2[Optional[Dollars]] = _at(SeriesPattern2, '*_high')
    low: SeriesPattern2[Optional[Dollars]] = _at(SeriesPattern2, '*_low')
    close: SeriesPattern2[Optional[Dollars]] = _at(SeriesPattern2, '*_close')
    ohlc: SeriesPattern2[OHLCDollars] = _at(SeriesPattern2, '*_ohlc')
    spot: SeriesPattern1[Optional[Dollars]] = _at(SeriesPattern1, '*')
    sats_per_dollar: SeriesPattern1[Sats] = _at(SeriesPattern1, 'sats_per_dollar')


class Timestamps(_Node):
    monotonic: SeriesPattern18[Timestamp] = _at(SeriesPattern18, '*_monotonic')
    period_start: SeriesPattern2[Timestamp] = _at(SeriesPattern2, '*')


class TxoutIndex(_Node):
    identity: SeriesPattern22[TxOutIndex] = _at(SeriesPattern22, '*')


class Identity(_Node):
    identity: SeriesPattern20[TxInIndex] = _at(SeriesPattern20, '*')


class MappingsTxIndex(_Node):
    identity: SeriesPattern19[TxIndex] = _at(SeriesPattern19, '*_index')
    input_count: SeriesPattern19[Count] = _at(SeriesPattern19, '*_input_count')
    output_count: SeriesPattern19[Count] = _at(SeriesPattern19, '*_output_count')


class MappingsYear10(_Node):
    date: SeriesPattern15[Date] = _at(SeriesPattern15, '*')
    first_height: SeriesPattern15[Height] = _at(SeriesPattern15, 'first_height')


class MappingsYear1(_Node):
    date: SeriesPattern14[Date] = _at(SeriesPattern14, '*')
    first_height: SeriesPattern14[Height] = _at(SeriesPattern14, 'first_height')


class MappingsMonth6(_Node):
    date: SeriesPattern13[Date] = _at(SeriesPattern13, '*')
    first_height: SeriesPattern13[Height] = _at(SeriesPattern13, 'first_height')


class MappingsMonth3(_Node):
    date: SeriesPattern12[Date] = _at(SeriesPattern12, '*')
    first_height: SeriesPattern12[Height] = _at(SeriesPattern12, 'first_height')


class MappingsMonth1(_Node):
    date: SeriesPattern11[Date] = _at(SeriesPattern11, '*')
    first_height: SeriesPattern11[Height] = _at(SeriesPattern11, 'first_height')


class MappingsWeek1(_Node):
    date: SeriesPattern10[Date] = _at(SeriesPattern10, '*')
    first_height: SeriesPattern10[Height] = _at(SeriesPattern10, 'first_height')


class MappingsDay3(_Node):
    date: SeriesPattern9[Date] = _at(SeriesPattern9, '*')
    first_height: SeriesPattern9[Height] = _at(SeriesPattern9, 'first_height')


class DatedResolution(_Node):
    date: SeriesPattern8[Date] = _at(SeriesPattern8, '*')
    first_height: SeriesPattern8[Height] = _at(SeriesPattern8, 'first_height')


class MappingsHour12(_Node):
    first_height: SeriesPattern7[Height] = _at(SeriesPattern7, '*')


class MappingsHour4(_Node):
    first_height: SeriesPattern6[Height] = _at(SeriesPattern6, '*')


class MappingsHour1(_Node):
    first_height: SeriesPattern5[Height] = _at(SeriesPattern5, '*')


class MappingsMinute30(_Node):
    first_height: SeriesPattern4[Height] = _at(SeriesPattern4, '*')


class MappingsMinute10(_Node):
    first_height: SeriesPattern3[Height] = _at(SeriesPattern3, '*')


class MappingsHalving(_Node):
    first_height: SeriesPattern16[Height] = _at(SeriesPattern16, '*')


class Resolution(_Node):
    first_height: SeriesPattern17[Height] = _at(SeriesPattern17, '*')


class MappingsHeight(_Node):
    minute10: SeriesPattern18[Minute10] = _at(SeriesPattern18, '*_minute10')
    minute30: SeriesPattern18[Minute30] = _at(SeriesPattern18, '*_minute30')
    hour1: SeriesPattern18[Hour1] = _at(SeriesPattern18, '*_hour1')
    hour4: SeriesPattern18[Hour4] = _at(SeriesPattern18, '*_hour4')
    hour12: SeriesPattern18[Hour12] = _at(SeriesPattern18, '*_hour12')
    day1: SeriesPattern18[Day1] = _at(SeriesPattern18, '*_day1')
    day3: SeriesPattern18[Day3] = _at(SeriesPattern18, '*_day3')
    epoch: SeriesPattern18[Epoch] = _at(SeriesPattern18, 'difficulty_epoch')
    halving: SeriesPattern18[Halving] = _at(SeriesPattern18, 'halving_epoch')
    week1: SeriesPattern18[Week1] = _at(SeriesPattern18, '*_week1')
    month1: SeriesPattern18[Month1] = _at(SeriesPattern18, '*_month1')
    month3: SeriesPattern18[Month3] = _at(SeriesPattern18, '*_month3')
    month6: SeriesPattern18[Month6] = _at(SeriesPattern18, '*_month6')
    year1: SeriesPattern18[Year1] = _at(SeriesPattern18, '*_year1')
    year10: SeriesPattern18[Year10] = _at(SeriesPattern18, '*_year10')


class OutputsOpReturn(_Node):
    identity: SeriesPattern25[OpReturnIndex] = _at(SeriesPattern25, '*')


class Unknown(_Node):
    identity: SeriesPattern35[UnknownOutputIndex] = _at(SeriesPattern35, '*')


class Empty(_Node):
    identity: SeriesPattern24[EmptyOutputIndex] = _at(SeriesPattern24, '*')


class P2ms(_Node):
    identity: SeriesPattern27[P2MSOutputIndex] = _at(SeriesPattern27, '*')


class Output(_Node):
    p2ms: P2ms = _at(P2ms, 'p2ms_output_*')
    empty: Empty = _at(Empty, 'empty_output_*')
    unknown: Unknown = _at(Unknown, 'unknown_output_*')
    op_return: OutputsOpReturn = _at(OutputsOpReturn, 'op_return_*')


class P2a(_Node):
    identity: SeriesPattern26[P2AAddressIndex] = _at(SeriesPattern26, '*_index')
    address: SeriesPattern26[Address] = _at(SeriesPattern26, '*')


class MappingsAddressesP2wsh(_Node):
    identity: SeriesPattern34[P2WSHAddressIndex] = _at(SeriesPattern34, '*_index')
    address: SeriesPattern34[Address] = _at(SeriesPattern34, '*')


class MappingsAddressesP2wpkh(_Node):
    identity: SeriesPattern33[P2WPKHAddressIndex] = _at(SeriesPattern33, '*_index')
    address: SeriesPattern33[Address] = _at(SeriesPattern33, '*')


class MappingsAddressesP2tr(_Node):
    identity: SeriesPattern32[P2TRAddressIndex] = _at(SeriesPattern32, '*_index')
    address: SeriesPattern32[Address] = _at(SeriesPattern32, '*')


class MappingsAddressesP2sh(_Node):
    identity: SeriesPattern31[P2SHAddressIndex] = _at(SeriesPattern31, '*_index')
    address: SeriesPattern31[Address] = _at(SeriesPattern31, '*')


class MappingsAddressesP2pkh(_Node):
    identity: SeriesPattern30[P2PKHAddressIndex] = _at(SeriesPattern30, '*_index')
    address: SeriesPattern30[Address] = _at(SeriesPattern30, '*')


class MappingsAddressesP2pk65(_Node):
    identity: SeriesPattern29[P2PK65AddressIndex] = _at(SeriesPattern29, '*_index')
    address: SeriesPattern29[Address] = _at(SeriesPattern29, '*')


class MappingsAddressesP2pk33(_Node):
    identity: SeriesPattern28[P2PK33AddressIndex] = _at(SeriesPattern28, '*_index')
    address: SeriesPattern28[Address] = _at(SeriesPattern28, '*')


class MappingsAddresses(_Node):
    p2pk33: MappingsAddressesP2pk33 = _at(MappingsAddressesP2pk33, 'p2pk33_*')
    p2pk65: MappingsAddressesP2pk65 = _at(MappingsAddressesP2pk65, 'p2pk65_*')
    p2pkh: MappingsAddressesP2pkh = _at(MappingsAddressesP2pkh, 'p2pkh_*')
    p2sh: MappingsAddressesP2sh = _at(MappingsAddressesP2sh, 'p2sh_*')
    p2tr: MappingsAddressesP2tr = _at(MappingsAddressesP2tr, 'p2tr_*')
    p2wpkh: MappingsAddressesP2wpkh = _at(MappingsAddressesP2wpkh, 'p2wpkh_*')
    p2wsh: MappingsAddressesP2wsh = _at(MappingsAddressesP2wsh, 'p2wsh_*')
    p2a: P2a = _at(P2a, 'p2a_*')


class Mappings(_Node):
    addresses: MappingsAddresses = _at(MappingsAddresses, 'address')
    outputs: Output = _at(Output, 'index')
    height: MappingsHeight = _at(MappingsHeight, 'block')
    epoch: Resolution = _at(Resolution, 'first_height')
    halving: MappingsHalving = _at(MappingsHalving, 'first_height')
    minute10: MappingsMinute10 = _at(MappingsMinute10, 'first_height')
    minute30: MappingsMinute30 = _at(MappingsMinute30, 'first_height')
    hour1: MappingsHour1 = _at(MappingsHour1, 'first_height')
    hour4: MappingsHour4 = _at(MappingsHour4, 'first_height')
    hour12: MappingsHour12 = _at(MappingsHour12, 'first_height')
    day1: DatedResolution = _at(DatedResolution, '*')
    day3: MappingsDay3 = _at(MappingsDay3, '*')
    week1: MappingsWeek1 = _at(MappingsWeek1, '*')
    month1: MappingsMonth1 = _at(MappingsMonth1, '*')
    month3: MappingsMonth3 = _at(MappingsMonth3, '*')
    month6: MappingsMonth6 = _at(MappingsMonth6, '*')
    year1: MappingsYear1 = _at(MappingsYear1, '*')
    year10: MappingsYear10 = _at(MappingsYear10, '*')
    tx_index: MappingsTxIndex = _at(MappingsTxIndex, 'tx')
    txin_index: Identity = _at(Identity, 'txin_index')
    txout_index: TxoutIndex = _at(TxoutIndex, 'txout_index')
    timestamp: Timestamps = _at(Timestamps, 'timestamp')


class IndexerOpReturn(_Node):
    first_index: SeriesPattern18[OpReturnIndex] = _at(SeriesPattern18, 'first_*_index')
    tx_index: SeriesPattern25[TxIndex] = _at(SeriesPattern25, '*_tx_index')
    protocol: SeriesPattern25[OpReturnKind] = _at(SeriesPattern25, '*_protocol')
    post_op_return_bytes: SeriesPattern25[Bytes] = _at(SeriesPattern25, 'post_*_bytes')


class IndexerScriptTypeWithSigOps(_Node):
    first_index: SeriesPattern18[UnknownOutputIndex] = _at(SeriesPattern18, 'first_*_output_index')
    tx_index: SeriesPattern35[TxIndex] = _at(SeriesPattern35, '*_output_tx_index')
    legacy_sigop_cost: SeriesPattern35[SigOps] = _at(SeriesPattern35, '*_legacy_sigop_cost')


class ScriptTypeWithSigOps(_Node):
    first_index: SeriesPattern18[P2MSOutputIndex] = _at(SeriesPattern18, 'first_*_output_index')
    tx_index: SeriesPattern27[TxIndex] = _at(SeriesPattern27, '*_output_tx_index')
    legacy_sigop_cost: SeriesPattern27[SigOps] = _at(SeriesPattern27, '*_legacy_sigop_cost')


class ScriptType(_Node):
    first_index: SeriesPattern18[EmptyOutputIndex] = _at(SeriesPattern18, 'first_*_index')
    tx_index: SeriesPattern24[TxIndex] = _at(SeriesPattern24, '*_tx_index')


class Scripts(_Node):
    empty: ScriptType = _at(ScriptType, '*')
    p2ms: ScriptTypeWithSigOps = _at(ScriptTypeWithSigOps, 'p2ms')
    unknown: IndexerScriptTypeWithSigOps = _at(IndexerScriptTypeWithSigOps, 'unknown')


class AddrType(_Node):
    first_index: SeriesPattern18[P2AAddressIndex] = _at(SeriesPattern18, 'first_*_address_index')
    bytes: SeriesPattern26[P2ABytes] = _at(SeriesPattern26, '*_bytes')


class IndexerAddressesP2tr(_Node):
    first_index: SeriesPattern18[P2TRAddressIndex] = _at(SeriesPattern18, 'first_*_address_index')
    bytes: SeriesPattern32[P2TRBytes] = _at(SeriesPattern32, '*_bytes')


class IndexerAddressesP2wsh(_Node):
    first_index: SeriesPattern18[P2WSHAddressIndex] = _at(SeriesPattern18, 'first_*_address_index')
    bytes: SeriesPattern34[P2WSHBytes] = _at(SeriesPattern34, '*_bytes')


class IndexerAddressesP2wpkh(_Node):
    first_index: SeriesPattern18[P2WPKHAddressIndex] = _at(SeriesPattern18, 'first_*_address_index')
    bytes: SeriesPattern33[P2WPKHBytes] = _at(SeriesPattern33, '*_bytes')


class IndexerAddressesP2sh(_Node):
    first_index: SeriesPattern18[P2SHAddressIndex] = _at(SeriesPattern18, 'first_*_address_index')
    bytes: SeriesPattern31[P2SHBytes] = _at(SeriesPattern31, '*_bytes')


class IndexerAddressesP2pkh(_Node):
    first_index: SeriesPattern18[P2PKHAddressIndex] = _at(SeriesPattern18, 'first_*_address_index')
    bytes: SeriesPattern30[P2PKHBytes] = _at(SeriesPattern30, '*_bytes')


class IndexerAddressesP2pk33(_Node):
    first_index: SeriesPattern18[P2PK33AddressIndex] = _at(SeriesPattern18, 'first_*_address_index')
    bytes: SeriesPattern28[P2PK33Bytes] = _at(SeriesPattern28, '*_bytes')


class IndexerAddressesP2pk65(_Node):
    first_index: SeriesPattern18[P2PK65AddressIndex] = _at(SeriesPattern18, 'first_*_address_index')
    bytes: SeriesPattern29[P2PK65Bytes] = _at(SeriesPattern29, '*_bytes')


class Addrs(_Node):
    p2pk65: IndexerAddressesP2pk65 = _at(IndexerAddressesP2pk65, '*')
    p2pk33: IndexerAddressesP2pk33 = _at(IndexerAddressesP2pk33, 'p2pk33')
    p2pkh: IndexerAddressesP2pkh = _at(IndexerAddressesP2pkh, 'p2pkh')
    p2sh: IndexerAddressesP2sh = _at(IndexerAddressesP2sh, 'p2sh')
    p2wpkh: IndexerAddressesP2wpkh = _at(IndexerAddressesP2wpkh, 'p2wpkh')
    p2wsh: IndexerAddressesP2wsh = _at(IndexerAddressesP2wsh, 'p2wsh')
    p2tr: IndexerAddressesP2tr = _at(IndexerAddressesP2tr, 'p2tr')
    p2a: AddrType = _at(AddrType, 'p2a')


class IndexerOutputs(_Node):
    first_txout_index: SeriesPattern18[TxOutIndex] = _at(SeriesPattern18, 'first_txout_index')
    value: SeriesPattern22[Sats] = _at(SeriesPattern22, '*_value')
    output_type: SeriesPattern22[OutputType] = _at(SeriesPattern22, '*_type')
    type_index: SeriesPattern22[TypeIndex] = _at(SeriesPattern22, 'type_index')


class IndexerInputs(_Node):
    first_txin_index: SeriesPattern18[TxInIndex] = _at(SeriesPattern18, 'first_txin_*')
    outpoint: SeriesPattern21[OutPoint] = _at(SeriesPattern21, 'outpoint')
    txout_index: SeriesPattern21[TxOutIndex] = _at(SeriesPattern21, 'spent_txout_*')
    tx_index: SeriesPattern20[TxIndex] = _at(SeriesPattern20, 'input_tx_*')
    output_type: SeriesPattern20[OutputType] = _at(SeriesPattern20, 'output_type')
    type_index: SeriesPattern21[TypeIndex] = _at(SeriesPattern21, 'type_*')
    value: SeriesPattern20[Sats] = _at(SeriesPattern20, 'input_value')


class CountSeries(_Node):
    block: SeriesPattern18[Count] = _at(SeriesPattern18, '*')


class VersionCount(_Node, Generic[A]):
    v1: A = _at(0, 'v1_*')
    v2: A = _at(0, 'v2_*')
    v3: A = _at(0, 'v3_*')
    other: A = _at(0, 'other_version_*')


class Transactions(_Node):
    count: FlowStats[Count, Optional[CountFract]] = _at(FlowStats, '*_count')
    features: Features = _at(Features, '*')
    vsize: PerTxDistribution[VSize] = _at(PerTxDistribution, '*_vsize')
    fees: Fees = _at(Fees, 'fee')
    inscription: Inscription = _at(Inscription, 'inscription')
    patterns: Patterns = _at(Patterns, 'coinjoin')
    nonstandard: TransactionsFlagged = _at(TransactionsFlagged, 'nonstandard')
    sigop_cost: Flow[SigOps] = _at(Flow, 'block_sigop_cost')
    versions: VersionCount[TxCount] = _at((VersionCount, TxCount), '*')
    volume: ValueFlow = _at(ValueFlow, '*_volume')
    per_second: Windows[Optional[PerSecond]] = _at(Windows, '*_per_second')


class IndexerFeature(_Node):
    flag: SeriesPattern19[Boolean] = _at(SeriesPattern19, 'is_*')
    block: SeriesPattern18[Count] = _at(SeriesPattern18, '*_tx_count')


class Feature(_Node):
    flag: SeriesPattern19[Boolean] = _at(SeriesPattern19, 'has_*')
    block: SeriesPattern18[Count] = _at(SeriesPattern18, '*_tx_count')


class TransactionFeatures(_Node):
    p2pk: Feature = _at(Feature, 'p2pk')
    p2ms: Feature = _at(Feature, 'p2ms')
    p2pkh: Feature = _at(Feature, 'p2pkh')
    p2sh: Feature = _at(Feature, 'p2sh')
    p2wpkh: Feature = _at(Feature, 'p2wpkh')
    p2wsh: Feature = _at(Feature, 'p2wsh')
    p2tr: Feature = _at(Feature, 'p2tr')
    p2a: Feature = _at(Feature, 'p2a')
    op_return: Feature = _at(Feature, 'op_return')
    empty_script: Feature = _at(Feature, 'empty_script')
    unknown_script: Feature = _at(Feature, 'unknown_script')
    fake_pubkey: Feature = _at(Feature, 'fake_pubkey')
    fake_scripthash: Feature = _at(Feature, 'fake_scripthash')
    inscription: Feature = _at(Feature, 'inscription')
    annex: Feature = _at(Feature, 'annex')
    sighash_all: Feature = _at(Feature, '*_all')
    sighash_none: Feature = _at(Feature, '*_none')
    sighash_single: Feature = _at(Feature, '*_single')
    sighash_default: Feature = _at(Feature, '*_default')
    sighash_anyone_can_pay: Feature = _at(Feature, '*_anyone_can_pay')
    explicitly_rbf: IndexerFeature = _at(IndexerFeature, 'explicitly_rbf')
    dust_output: Feature = _at(Feature, 'dust_output')
    epobc: IndexerFeature = _at(IndexerFeature, 'epobc')
    one_input: CountSeries = _at(CountSeries, 'one_input_tx_count')
    one_output: CountSeries = _at(CountSeries, 'one_output_tx_count')


class IndexerTransactions(_Node):
    first_tx_index: SeriesPattern18[TxIndex] = _at(SeriesPattern18, 'first_*_index')
    txid: SeriesPattern19[Txid] = _at(SeriesPattern19, 'txid')
    version: SeriesPattern19[TxVersion] = _at(SeriesPattern19, '*_version')
    locktime: SeriesPattern19[RawLockTime] = _at(SeriesPattern19, '*_locktime')
    weight: SeriesPattern19[Weight] = _at(SeriesPattern19, '*_weight')
    size: SeriesPattern19[Bytes] = _at(SeriesPattern19, '*_size')
    sigop_cost: SeriesPattern19[SigOps] = _at(SeriesPattern19, '*_sigop_cost')
    first_txin_index: SeriesPattern19[TxInIndex] = _at(SeriesPattern19, 'first_txin_index')
    first_txout_index: SeriesPattern19[TxOutIndex] = _at(SeriesPattern19, 'first_txout_index')
    features: TransactionFeatures = _at(TransactionFeatures, 'sighash')
    versions: VersionCount[CountSeries] = _at((VersionCount, CountSeries), '*_count')


class IndexerBlocks(_Node):
    blockhash: SeriesPattern18[BlockHash] = _at(SeriesPattern18, 'blockhash')
    coinbase_tag: SeriesPattern18[CoinbaseTag] = _at(SeriesPattern18, 'coinbase_tag')
    timestamp: SeriesPattern18[Timestamp] = _at(SeriesPattern18, 'timestamp')
    median_time: SeriesPattern18[Timestamp] = _at(SeriesPattern18, 'median_time')
    size: SeriesPattern18[Bytes] = _at(SeriesPattern18, 'block_size')
    weight: SeriesPattern18[Weight] = _at(SeriesPattern18, 'block_weight')
    segwit_tx_count: SeriesPattern18[Count] = _at(SeriesPattern18, '*_count')
    segwit_tx_size: SeriesPattern18[Bytes] = _at(SeriesPattern18, '*_size')
    segwit_tx_weight: SeriesPattern18[Weight] = _at(SeriesPattern18, '*_weight')


class Indexer(_Node):
    blocks: IndexerBlocks = _at(IndexerBlocks, 'segwit_*')
    transactions: IndexerTransactions = _at(IndexerTransactions, '*')
    inputs: IndexerInputs = _at(IndexerInputs, 'index')
    outputs: IndexerOutputs = _at(IndexerOutputs, 'output')
    addresses: Addrs = _at(Addrs, 'p2pk65')
    scripts: Scripts = _at(Scripts, 'empty_output')
    op_return: IndexerOpReturn = _at(IndexerOpReturn, 'op_return')


class SeriesTree(_Node):
    indexer: Indexer = _at(Indexer, 'tx')
    mappings: Mappings = _at(Mappings, 'date')
    price: Price = _at(Price, 'price')
    blocks: Blocks = _at(Blocks, 'block')
    inputs: Inputs = _at(Inputs, 'input')
    outputs: Outputs = _at(Outputs, 'output')
    transactions: Transactions = _at(Transactions, 'tx')
    mining: Mining = _at(Mining, 'coinbase')
    op_return: OpReturn = _at(OpReturn, 'op_return')
    pools: Pools = _at(Pools, 'pool')
    utxo_set: UtxoSetSeries = _at(UtxoSetSeries, 'circulating_supply')
    market: Market = _at(Market, 'price')
    age: Age = _at(Age, 'age')
    utxos: Utxos = _at(Utxos, 'amount')
    addresses: Addresses = _at(Addresses, 'address')
    holders: Holders[CreationCohort[AllSupply, Capital, HoldersActivity, HoldersRealized, Unrealized, HoldersCostBasis], CreationCohort[LthSupply, Capital, HoldersActivity, HoldersRealized, Unrealized, HoldersCostBasis]] = _at((Holders, (CreationCohort, AllSupply, Capital, HoldersActivity, HoldersRealized, Unrealized, HoldersCostBasis), (CreationCohort, LthSupply, Capital, HoldersActivity, HoldersRealized, Unrealized, HoldersCostBasis)), 'age')
    entry: Entry = _at(Entry, 'veteran')
    profitability: Profitability = _at(Profitability, 'sth')
    supply: Supply = _at(Supply, 'circulating_supply')
    cointime: Cointime = _at(Cointime, 'age')
    coinflow: Coinflow = _at(Coinflow, 'age')
    bedrock: Bedrock = _at(Bedrock, 'bedrock')
    rarity_meter: RarityMeter = _at(RarityMeter, 'rarity_meter')


class BitviewClient(BitviewClientBase):
    """A method per API endpoint, and the typed series tree."""

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
      "p2a_address_index",
      "p2ms_output_index",
      "p2pk33_address_index",
      "p2pk65_address_index",
      "p2pkh_address_index",
      "p2sh_address_index",
      "p2tr_address_index",
      "p2wpkh_address_index",
      "p2wsh_address_index",
      "unknown_output_index"
    ]

    POOL_ID_TO_POOL_NAME = {
      "175btc": "175btc",
      "1hash": "1Hash",
      "1m1x": "1M1X",
      "1thash": "1THash",
      "21inc": "21 Inc.",
      "50btc": "50BTC",
      "58coin": "58COIN",
      "7pool": "7pool",
      "8baochi": "8baochi",
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
      "ekanembtc": "EkanemBTC",
      "eligius": "Eligius",
      "emcdpool": "EMCDPool",
      "entrustcharitypool": "Entrust Charity Pool",
      "eobot": "Eobot",
      "est3lar": "Est3lar",
      "exxbw": "EXX&BW",
      "f2pool": "F2Pool",
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
      "ozcoin": "OzCoin",
      "parasite": "Parasite",
      "patels": "Patels",
      "pegapool": "PEGA Pool",
      "phashio": "PHash.IO",
      "phoenix": "Phoenix",
      "polmine": "Polmine",
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
      "shawnp0wers": "shawnp0wers",
      "sigmapoolcom": "Sigmapool.com",
      "simplecoinus": "simplecoin.us",
      "solock": "Solo CK",
      "solopoolcom": "SoloPool.com",
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

    HALVING_EPOCH_NAMES = {
      "_0": {
        "id": "halving_epoch_0",
        "short": "0",
        "long": "Halving epoch 0"
      },
      "_1": {
        "id": "halving_epoch_1",
        "short": "1",
        "long": "Halving epoch 1"
      },
      "_2": {
        "id": "halving_epoch_2",
        "short": "2",
        "long": "Halving epoch 2"
      },
      "_3": {
        "id": "halving_epoch_3",
        "short": "3",
        "long": "Halving epoch 3"
      },
      "_4": {
        "id": "halving_epoch_4",
        "short": "4",
        "long": "Halving epoch 4"
      },
      "_5": {
        "id": "halving_epoch_5",
        "short": "5",
        "long": "Halving epoch 5"
      }
    }

    VINTAGE_NAMES = {
      "_2009": {
        "id": "vintage_2009",
        "short": "2009",
        "long": "Vintage 2009"
      },
      "_2010": {
        "id": "vintage_2010",
        "short": "2010",
        "long": "Vintage 2010"
      },
      "_2011": {
        "id": "vintage_2011",
        "short": "2011",
        "long": "Vintage 2011"
      },
      "_2012": {
        "id": "vintage_2012",
        "short": "2012",
        "long": "Vintage 2012"
      },
      "_2013": {
        "id": "vintage_2013",
        "short": "2013",
        "long": "Vintage 2013"
      },
      "_2014": {
        "id": "vintage_2014",
        "short": "2014",
        "long": "Vintage 2014"
      },
      "_2015": {
        "id": "vintage_2015",
        "short": "2015",
        "long": "Vintage 2015"
      },
      "_2016": {
        "id": "vintage_2016",
        "short": "2016",
        "long": "Vintage 2016"
      },
      "_2017": {
        "id": "vintage_2017",
        "short": "2017",
        "long": "Vintage 2017"
      },
      "_2018": {
        "id": "vintage_2018",
        "short": "2018",
        "long": "Vintage 2018"
      },
      "_2019": {
        "id": "vintage_2019",
        "short": "2019",
        "long": "Vintage 2019"
      },
      "_2020": {
        "id": "vintage_2020",
        "short": "2020",
        "long": "Vintage 2020"
      },
      "_2021": {
        "id": "vintage_2021",
        "short": "2021",
        "long": "Vintage 2021"
      },
      "_2022": {
        "id": "vintage_2022",
        "short": "2022",
        "long": "Vintage 2022"
      },
      "_2023": {
        "id": "vintage_2023",
        "short": "2023",
        "long": "Vintage 2023"
      },
      "_2024": {
        "id": "vintage_2024",
        "short": "2024",
        "long": "Vintage 2024"
      },
      "_2025": {
        "id": "vintage_2025",
        "short": "2025",
        "long": "Vintage 2025"
      },
      "_2026": {
        "id": "vintage_2026",
        "short": "2026",
        "long": "Vintage 2026"
      },
      "_2027": {
        "id": "vintage_2027",
        "short": "2027",
        "long": "Vintage 2027"
      }
    }

    ENTRY_NAMES = {
      "veteran": {
        "id": "veteran",
        "short": "Veteran",
        "long": "Veteran Coins"
      },
      "rookie": {
        "id": "rookie",
        "short": "Rookie",
        "long": "Rookie Coins"
      }
    }

    SPENDABLE_TYPE_NAMES = {
      "p2pk": {
        "id": "p2pk",
        "short": "P2PK",
        "long": "Pay to Public Key"
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
        "id": "unknown_output",
        "short": "Unknown",
        "long": "Unknown Output Type"
      },
      "empty": {
        "id": "empty_output",
        "short": "Empty",
        "long": "Empty Output"
      }
    }

    AGE_RANGE_NAMES = {
      "under_1h": {
        "id": "under_1h",
        "short": "<1h",
        "long": "Under 1 Hour Old"
      },
      "_1h_to_1d": {
        "id": "1h_to_1d",
        "short": "1h-1d",
        "long": "1 Hour to 1 Day Old"
      },
      "_1d_to_1w": {
        "id": "1d_to_1w",
        "short": "1d-1w",
        "long": "1 Day to 1 Week Old"
      },
      "_1w_to_1m": {
        "id": "1w_to_1m",
        "short": "1w-1m",
        "long": "1 Week to 1 Month Old"
      },
      "_1m_to_2m": {
        "id": "1m_to_2m",
        "short": "1m-2m",
        "long": "1 to 2 Months Old"
      },
      "_2m_to_3m": {
        "id": "2m_to_3m",
        "short": "2m-3m",
        "long": "2 to 3 Months Old"
      },
      "_3m_to_4m": {
        "id": "3m_to_4m",
        "short": "3m-4m",
        "long": "3 to 4 Months Old"
      },
      "_4m_to_5m": {
        "id": "4m_to_5m",
        "short": "4m-5m",
        "long": "4 to 5 Months Old"
      },
      "_5m_to_6m": {
        "id": "5m_to_6m",
        "short": "5m-6m",
        "long": "5 to 6 Months Old"
      },
      "_6m_to_9m": {
        "id": "6m_to_9m",
        "short": "6m-9m",
        "long": "6 to 9 Months Old"
      },
      "_9m_to_1y": {
        "id": "9m_to_1y",
        "short": "9m-1y",
        "long": "9 Months to 1 Year Old"
      },
      "_1y_to_18m": {
        "id": "1y_to_18m",
        "short": "1y-18m",
        "long": "1 Year to 18 Months Old"
      },
      "_18m_to_2y": {
        "id": "18m_to_2y",
        "short": "18m-2y",
        "long": "18 Months to 2 Years Old"
      },
      "_2y_to_3y": {
        "id": "2y_to_3y",
        "short": "2y-3y",
        "long": "2 to 3 Years Old"
      },
      "_3y_to_4y": {
        "id": "3y_to_4y",
        "short": "3y-4y",
        "long": "3 to 4 Years Old"
      },
      "_4y_to_5y": {
        "id": "4y_to_5y",
        "short": "4y-5y",
        "long": "4 to 5 Years Old"
      },
      "_5y_to_6y": {
        "id": "5y_to_6y",
        "short": "5y-6y",
        "long": "5 to 6 Years Old"
      },
      "_6y_to_7y": {
        "id": "6y_to_7y",
        "short": "6y-7y",
        "long": "6 to 7 Years Old"
      },
      "_7y_to_8y": {
        "id": "7y_to_8y",
        "short": "7y-8y",
        "long": "7 to 8 Years Old"
      },
      "_8y_to_10y": {
        "id": "8y_to_10y",
        "short": "8y-10y",
        "long": "8 to 10 Years Old"
      },
      "_10y_to_12y": {
        "id": "10y_to_12y",
        "short": "10y-12y",
        "long": "10 to 12 Years Old"
      },
      "_12y_to_15y": {
        "id": "12y_to_15y",
        "short": "12y-15y",
        "long": "12 to 15 Years Old"
      },
      "over_15y": {
        "id": "over_15y",
        "short": "15y+",
        "long": "15+ Years Old"
      }
    }

    AMOUNT_RANGE_NAMES = {
      "_0_sats": {
        "id": "0_sats",
        "short": "0 sats",
        "long": "0 Sats"
      },
      "_1_to_10_sats": {
        "id": "1_to_10_sats",
        "short": "1-10 sats",
        "long": "1-10 Sats"
      },
      "_10_to_100_sats": {
        "id": "10_to_100_sats",
        "short": "10-100 sats",
        "long": "10-100 Sats"
      },
      "_100_to_1k_sats": {
        "id": "100_to_1k_sats",
        "short": "100-1k sats",
        "long": "100-1K Sats"
      },
      "_1k_to_10k_sats": {
        "id": "1k_to_10k_sats",
        "short": "1k-10k sats",
        "long": "1K-10K Sats"
      },
      "_10k_to_100k_sats": {
        "id": "10k_to_100k_sats",
        "short": "10k-100k sats",
        "long": "10K-100K Sats"
      },
      "_100k_to_1m_sats": {
        "id": "100k_to_1m_sats",
        "short": "100k-1M sats",
        "long": "100K-1M Sats"
      },
      "_1m_to_10m_sats": {
        "id": "1m_to_10m_sats",
        "short": "1M-10M sats",
        "long": "1M-10M Sats"
      },
      "_10m_to_100m_sats": {
        "id": "10m_to_100m_sats",
        "short": "0.1-1 BTC",
        "long": "0.1-1 BTC"
      },
      "_1_to_10_btc": {
        "id": "1_to_10_btc",
        "short": "1-10 BTC",
        "long": "1-10 BTC"
      },
      "_10_to_100_btc": {
        "id": "10_to_100_btc",
        "short": "10-100 BTC",
        "long": "10-100 BTC"
      },
      "_100_to_1k_btc": {
        "id": "100_to_1k_btc",
        "short": "100-1k BTC",
        "long": "100-1K BTC"
      },
      "_1k_to_10k_btc": {
        "id": "1k_to_10k_btc",
        "short": "1k-10k BTC",
        "long": "1K-10K BTC"
      },
      "_10k_to_100k_btc": {
        "id": "10k_to_100k_btc",
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
      "profit_over_1000_percent": {
        "id": "profit_over_1000_percent",
        "short": "+>1000%",
        "long": "Over 1000% in Profit"
      },
      "profit_500_to_1000_percent": {
        "id": "profit_500_to_1000_percent",
        "short": "+500-1000%",
        "long": "500-1000% in Profit"
      },
      "profit_300_to_500_percent": {
        "id": "profit_300_to_500_percent",
        "short": "+300-500%",
        "long": "300-500% in Profit"
      },
      "profit_200_to_300_percent": {
        "id": "profit_200_to_300_percent",
        "short": "+200-300%",
        "long": "200-300% in Profit"
      },
      "profit_100_to_200_percent": {
        "id": "profit_100_to_200_percent",
        "short": "+100-200%",
        "long": "100-200% in Profit"
      },
      "profit_90_to_100_percent": {
        "id": "profit_90_to_100_percent",
        "short": "+90-100%",
        "long": "90-100% in Profit"
      },
      "profit_80_to_90_percent": {
        "id": "profit_80_to_90_percent",
        "short": "+80-90%",
        "long": "80-90% in Profit"
      },
      "profit_70_to_80_percent": {
        "id": "profit_70_to_80_percent",
        "short": "+70-80%",
        "long": "70-80% in Profit"
      },
      "profit_60_to_70_percent": {
        "id": "profit_60_to_70_percent",
        "short": "+60-70%",
        "long": "60-70% in Profit"
      },
      "profit_50_to_60_percent": {
        "id": "profit_50_to_60_percent",
        "short": "+50-60%",
        "long": "50-60% in Profit"
      },
      "profit_40_to_50_percent": {
        "id": "profit_40_to_50_percent",
        "short": "+40-50%",
        "long": "40-50% in Profit"
      },
      "profit_30_to_40_percent": {
        "id": "profit_30_to_40_percent",
        "short": "+30-40%",
        "long": "30-40% in Profit"
      },
      "profit_20_to_30_percent": {
        "id": "profit_20_to_30_percent",
        "short": "+20-30%",
        "long": "20-30% in Profit"
      },
      "profit_10_to_20_percent": {
        "id": "profit_10_to_20_percent",
        "short": "+10-20%",
        "long": "10-20% in Profit"
      },
      "profit_0_to_10_percent": {
        "id": "profit_0_to_10_percent",
        "short": "+0-10%",
        "long": "0-10% in Profit"
      },
      "loss_0_to_10_percent": {
        "id": "loss_0_to_10_percent",
        "short": "-0-10%",
        "long": "0-10% in Loss"
      },
      "loss_10_to_20_percent": {
        "id": "loss_10_to_20_percent",
        "short": "-10-20%",
        "long": "10-20% in Loss"
      },
      "loss_20_to_30_percent": {
        "id": "loss_20_to_30_percent",
        "short": "-20-30%",
        "long": "20-30% in Loss"
      },
      "loss_30_to_40_percent": {
        "id": "loss_30_to_40_percent",
        "short": "-30-40%",
        "long": "30-40% in Loss"
      },
      "loss_40_to_50_percent": {
        "id": "loss_40_to_50_percent",
        "short": "-40-50%",
        "long": "40-50% in Loss"
      },
      "loss_50_to_60_percent": {
        "id": "loss_50_to_60_percent",
        "short": "-50-60%",
        "long": "50-60% in Loss"
      },
      "loss_60_to_70_percent": {
        "id": "loss_60_to_70_percent",
        "short": "-60-70%",
        "long": "60-70% in Loss"
      },
      "loss_70_to_80_percent": {
        "id": "loss_70_to_80_percent",
        "short": "-70-80%",
        "long": "70-80% in Loss"
      },
      "loss_80_to_90_percent": {
        "id": "loss_80_to_90_percent",
        "short": "-80-90%",
        "long": "80-90% in Loss"
      },
      "loss_90_to_100_percent": {
        "id": "loss_90_to_100_percent",
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
        """A series by a name known only at runtime; `series` has the typed paths."""
        return _endpoint(self, series, index)

    def index_to_date(self, index: Index, i: int) -> Union[date, datetime]:
        """The start of bucket `i`: a UTC datetime for sub-daily indexes, a date otherwise."""
        return _index_to_date(index, i)

    def date_to_index(self, index: Index, d: Union[date, datetime]) -> int:
        """The floor index (latest index whose date is <= the given date)."""
        return _date_to_index(index, d)

    @staticmethod
    def address_payload_hash_prefix(payload: Union[bytes, bytearray, memoryview], nibbles: int) -> str:
        """Compute the RapidHash v3 hash-prefix for raw address payload bytes."""
        return address_payload_hash_prefix(payload, nibbles)

    def get_address_payload_hash_prefix_matches(
        self,
        address_type: OutputType,
        payload: Union[bytes, bytearray, memoryview],
        nibbles: int,
    ) -> AddressHashPrefixMatches:
        """Fetch address hash-prefix matches from raw payload bytes matching address_type length."""
        _validate_address_payload_for_type(address_type, payload)
        return self.get_address_hash_prefix_matches(address_type, address_payload_hash_prefix(payload, nibbles))

    def get_health(self) -> Health:
        """Health check.

        Local health and query-readiness check. Returns server identity, uptime, and a coherent local sync snapshot without a bitcoind round-trip. Reads the published prefix during processing; an empty index returns 503 `state_updating` at once. Responses are not cached. For chain-tip catch-up, request `GET /api/server/sync`.

        Endpoint: `GET /health`"""
        return self.get_json('/health')

    def get_version(self) -> str:
        """API version

        Endpoint: `GET /version`"""
        return self.get_json('/version')

    def get_sync_status(self) -> SyncStatus:
        """Sync status.

        Returns a coherent local index snapshot and a separately observed Bitcoin Core tip height. The two heights can differ during indexing or a reorg. Conditional requests refresh these observations before validation.

        Endpoint: `GET /api/server/sync`"""
        return self.get_json('/api/server/sync')

    def get_disk_usage(self) -> DiskUsage:
        """Disk usage.

        Returns allocated file bytes (file lengths on non-Unix platforms) for BRK and Bitcoin data. Each request scans both trees; these are independent observations, not an atomic filesystem snapshot. Conditional requests validate the newly observed totals. Directory-link cycles and excessive nesting fail without returning partial totals.

        Endpoint: `GET /api/server/disk`"""
        return self.get_json('/api/server/disk')

    def get_series_tree(self) -> TreeNode:
        """Series catalog.

        The available series as a tree of categories and subcategories.

        Endpoint: `GET /api/series`"""
        return self.get_json('/api/series')

    def get_series_count(self) -> DetailedSeriesCount:
        """Series count.

        Returns the number of series available per index type.

        Endpoint: `GET /api/series/count`"""
        return self.get_json('/api/series/count')

    def get_indexes(self) -> List[IndexInfo]:
        """List available indexes.

        Returns all available indexes with their accepted REST query aliases. REST series queries accept any alias; MCP tools take the canonical `index` name only.

        Endpoint: `GET /api/series/indexes`"""
        return self.get_json('/api/series/indexes')

    def list_series(self, page: Optional[int] = None, per_page: Optional[int] = None) -> PaginatedSeries:
        """Series list.

        Paginated flat list of all available series names.

        Endpoint: `GET /api/series/list`"""
        params: List[Tuple[str, Any]] = []
        if page is not None: params.append(('page', page))
        if per_page is not None: params.append(('per_page', per_page))
        query = urlencode(params)
        path = f'/api/series/list{"?" + query if query else ""}'
        return self.get_json(path)

    def search_series(self, q: SeriesName, limit: Optional[Limit] = None) -> List[str]:
        """Search series.

        Search series by name or descriptive terms. Results prioritize whole query words in names, then descriptions, then fuzzy names, then fuzzy descriptions. Word order does not matter. Descriptions provide cohort terminology and formulas. The decoded q parameter is limited to 1024 UTF-8 bytes.

        Endpoint: `GET /api/series/search`"""
        params: List[Tuple[str, Any]] = []
        params.append(('q', q))
        if limit is not None: params.append(('limit', limit))
        query = urlencode(params)
        path = f'/api/series/search{"?" + query if query else ""}'
        return self.get_json(path)

    def get_series_info(self, series: SeriesName) -> SeriesInfo:
        """Get series info.

        Returns the optional description, supported indexes, the indexes whose values can be null, the value type, and the optional unit (what the value type measures) for the specified series. The decoded series name is limited to 1024 UTF-8 bytes.

        Endpoint: `GET /api/series/{series}`"""
        return self.get_json(f'/api/series/{_segment(series)}')

    def get_series(self, series: SeriesName, index: Index, start: Optional[RangeIndex] = None, end: Optional[RangeIndex] = None, limit: Optional[Limit] = None, format: Optional[Format] = None) -> Union[AnySeriesData, str]:
        """Get series data.

        Rows selected by a range of positions, dates or timestamps, as JSON or CSV.

        Endpoint: `GET /api/series/{series}/{index}`"""
        params: List[Tuple[str, Any]] = []
        if start is not None: params.append(('start', start))
        if end is not None: params.append(('end', end))
        if limit is not None: params.append(('limit', limit))
        if format is not None: params.append(('format', format))
        query = urlencode(params)
        path = f'/api/series/{_segment(series)}/{_segment(index)}{"?" + query if query else ""}'
        if format == 'csv':
            return self.get_text(path)
        return _series_data(self.get_json(path))

    def get_series_data(self, series: SeriesName, index: Index, start: Optional[RangeIndex] = None, end: Optional[RangeIndex] = None, limit: Optional[Limit] = None, format: Optional[Format] = None) -> Union[List[Any], str]:
        """Get raw series data.

        The data array without the `SeriesData` wrapper. Supports the same range and format parameters as `GET /api/series/{series}/{index}`.

        Endpoint: `GET /api/series/{series}/{index}/data`"""
        params: List[Tuple[str, Any]] = []
        if start is not None: params.append(('start', start))
        if end is not None: params.append(('end', end))
        if limit is not None: params.append(('limit', limit))
        if format is not None: params.append(('format', format))
        query = urlencode(params)
        path = f'/api/series/{_segment(series)}/{_segment(index)}/data{"?" + query if query else ""}'
        if format == 'csv':
            return self.get_text(path)
        return self.get_json(path)

    def get_series_latest(self, series: SeriesName, index: Index) -> Any:
        """Get latest series value.

        Returns the single most recent value for a series, unwrapped (not inside a SeriesData object).

        Endpoint: `GET /api/series/{series}/{index}/latest`"""
        return self.get_json(f'/api/series/{_segment(series)}/{_segment(index)}/latest')

    def get_series_len(self, series: SeriesName, index: Index) -> int:
        """Get series data length

        Endpoint: `GET /api/series/{series}/{index}/len`"""
        return self.get_json(f'/api/series/{_segment(series)}/{_segment(index)}/len')

    def get_series_version(self, series: SeriesName, index: Index) -> Version:
        """Get series version.

        Returns the vector's schema/computation version, not its length or latest update. Appends and reorgs do not by themselves change this version.

        Endpoint: `GET /api/series/{series}/{index}/version`"""
        return self.get_json(f'/api/series/{_segment(series)}/{_segment(index)}/version')

    def get_series_bulk(self, series: SeriesList, index: Index, start: Optional[RangeIndex] = None, end: Optional[RangeIndex] = None, limit: Optional[Limit] = None, format: Optional[Format] = None) -> Union[List[AnySeriesData], str]:
        """Bulk series data.

        Fetch multiple series in a single request. Supports filtering by index and date range. Returns an array of SeriesData objects. For a single series, use `get_series` instead.

        Endpoint: `GET /api/series/bulk`"""
        params: List[Tuple[str, Any]] = []
        params.append(('series', series))
        params.append(('index', index))
        if start is not None: params.append(('start', start))
        if end is not None: params.append(('end', end))
        if limit is not None: params.append(('limit', limit))
        if format is not None: params.append(('format', format))
        query = urlencode(params)
        path = f'/api/series/bulk{"?" + query if query else ""}'
        if format == 'csv':
            return self.get_text(path)
        return [_series_data(raw) for raw in self.get_json(path)]

    def list_urpd_cohorts(self) -> List[Cohort]:
        """Available URPD cohorts.

        Cohorts for which URPD data is available. Returns names like `all`, `sth`, `lth`, `age_under_4m`, `age_under_6m`, `age_over_4m`, `age_over_6m`, `age_under_1h`.

        Endpoint: `GET /api/urpd`"""
        return self.get_json('/api/urpd')

    def get_urpd(self, cohort: Cohort, agg: Optional[UrpdAggregation] = None, weight: Optional[UrpdWeight] = None) -> Urpd:
        """Latest URPD.

        URPD for the latest published block; `date` echoes the date served. Fields, units, price-grid rounding and weighting as in `GET /api/urpd/{cohort}/{point}`.

        Endpoint: `GET /api/urpd/{cohort}`"""
        params: List[Tuple[str, Any]] = []
        if agg is not None: params.append(('agg', agg))
        if weight is not None: params.append(('weight', weight))
        query = urlencode(params)
        path = f'/api/urpd/{_segment(cohort)}{"?" + query if query else ""}'
        return self.get_json(path)

    def get_urpd_at(self, cohort: Cohort, point: str, agg: Optional[UrpdAggregation] = None, weight: Optional[UrpdWeight] = None) -> Urpd:
        """URPD at block height or date.

        URPD for a cohort at a block height or the last block of a UTC day. Returns `{ height, hash, date, cohort, weight, aggregation, close, supply, buckets }` where `buckets` is columns, `{ price, supply, realized_cap, unrealized_pnl }`, one entry per bucket in ascending price: the same block header and columnar layout as `GET /api/utxo-set/{point}`. `close`, `price`, `realized_cap`, and `unrealized_pnl` are USD; `supply` is BTC. `unrealized_pnl` can be negative. Bucket `realized_cap` and `unrealized_pnl` value each coin at its creation price on the price grid (four significant digits of the price in cents, within 0.05% of the exact price), rounded to the cent per bucket, so the buckets need not sum to the realized cap series. With a `wakefulness` or `mobility` `weight`, `supply` is the weighted supply (whole satoshis) and the bucket values follow it.

        Endpoint: `GET /api/urpd/{cohort}/{point}`"""
        params: List[Tuple[str, Any]] = []
        if agg is not None: params.append(('agg', agg))
        if weight is not None: params.append(('weight', weight))
        query = urlencode(params)
        path = f'/api/urpd/{_segment(cohort)}/{_segment(point)}{"?" + query if query else ""}'
        return self.get_json(path)

    def get_difficulty_adjustment(self) -> DifficultyAdjustment:
        """Difficulty adjustment.

        Get current difficulty adjustment progress and estimates.

        *[Mempool.space docs](https://mempool.space/docs/api/rest#get-difficulty-adjustment)*

        Endpoint: `GET /api/v1/difficulty-adjustment`"""
        return self.get_json('/api/v1/difficulty-adjustment')

    def get_prices(self) -> PricePoint:
        """Current BTC price.

        Returns bitcoin latest price (on-chain derived, USD only).

        *[Mempool.space docs](https://mempool.space/docs/api/rest#get-price)*

        Endpoint: `GET /api/v1/prices`"""
        return self.get_json('/api/v1/prices')

    def get_historical_price(self, timestamp: Optional[Timestamp] = None, currency: Optional[Currency] = None) -> HistoricalPrice:
        """Historical price.

        Completed four-hour BTC/USD closes, oldest first, labeled by interval end, from the first priced interval (2010-08-16). With a UNIX timestamp, returns the latest nonempty completed close at or before it; before the first close returns an empty list. The current partial interval is excluded. USD only (`currency` accepts `USD`); exchangeRates is empty.

        *[Mempool.space docs](https://mempool.space/docs/api/rest#get-historical-price)*

        Endpoint: `GET /api/v1/historical-price`"""
        params: List[Tuple[str, Any]] = []
        if timestamp is not None: params.append(('timestamp', timestamp))
        if currency is not None: params.append(('currency', currency))
        query = urlencode(params)
        path = f'/api/v1/historical-price{"?" + query if query else ""}'
        return self.get_json(path)

    def get_address_hash_prefix_matches(self, address_type: OutputType, prefix: str) -> AddressHashPrefixMatches:
        """Address hash-prefix matches.

        Find addresses by address type and by the first 1-16 hex nibbles of RapidHash v3 over the raw address payload bytes. Intended for privacy-preserving client-side wallet discovery without sending raw addresses or xpubs. Fetch metadata with `GET /api/address/{address}`.

        Endpoint: `GET /api/address/hash-prefix/{address_type}/{prefix}`"""
        return self.get_json(f'/api/address/hash-prefix/{_segment(address_type)}/{_segment(prefix)}')

    def get_address(self, address: Address) -> AddressStats:
        """Address information.

        Balance and transaction counts. Supports P2PKH, P2SH, P2WPKH, P2WSH, P2TR and P2A addresses, and P2PK public keys in hex (33 or 65 bytes).

        *[Mempool.space docs](https://mempool.space/docs/api/rest#get-address)*

        Endpoint: `GET /api/address/{address}`"""
        return self.get_json(f'/api/address/{_segment(address)}')

    def get_address_txs(self, address: Address) -> List[Transaction]:
        """Address transactions.

        Get transaction history for an address, newest first. Returns up to 50 mempool transactions plus a confirmed page sized to fill the response to 50 total (chain floor of 25, so 25-50 confirmed depending on mempool weight). To paginate further confirmed history, request `GET /api/address/{address}/txs/chain/{after_txid}` with the last returned txid.

        *[Mempool.space docs](https://mempool.space/docs/api/rest#get-address-transactions)*

        Endpoint: `GET /api/address/{address}/txs`"""
        return self.get_json(f'/api/address/{_segment(address)}/txs')

    def get_address_confirmed_txs(self, address: Address) -> List[Transaction]:
        """Address confirmed transactions.

        Get the first 25 confirmed transactions for an address. For pagination, request `GET /api/address/{address}/txs/chain/{after_txid}` with the last returned txid.

        *[Mempool.space docs](https://mempool.space/docs/api/rest#get-address-transactions-chain)*

        Endpoint: `GET /api/address/{address}/txs/chain`"""
        return self.get_json(f'/api/address/{_segment(address)}/txs/chain')

    def get_address_confirmed_txs_after(self, address: Address, after_txid: Txid) -> List[Transaction]:
        """Address confirmed transactions (paginated).

        Get the next 25 confirmed transactions strictly older than `after_txid` (Esplora-canonical pagination form, matches mempool.space).

        *[Mempool.space docs](https://mempool.space/docs/api/rest#get-address-transactions-chain)*

        Endpoint: `GET /api/address/{address}/txs/chain/{after_txid}`"""
        return self.get_json(f'/api/address/{_segment(address)}/txs/chain/{_segment(after_txid)}')

    def get_address_mempool_txs(self, address: Address) -> List[Transaction]:
        """Address mempool transactions.

        Get unconfirmed transactions for an address from the mempool, newest first (up to 50).

        *[Mempool.space docs](https://mempool.space/docs/api/rest#get-address-transactions-mempool)*

        Endpoint: `GET /api/address/{address}/txs/mempool`"""
        return self.get_json(f'/api/address/{_segment(address)}/txs/mempool')

    def get_address_utxos(self, address: Address) -> List[Utxo]:
        """Address UTXOs.

        Get unspent transaction outputs (UTXOs) for an address, including unconfirmed ones; outputs that mempool transactions spend are left out.

        *[Mempool.space docs](https://mempool.space/docs/api/rest#get-address-utxo)*

        Endpoint: `GET /api/address/{address}/utxo`"""
        return self.get_json(f'/api/address/{_segment(address)}/utxo')

    def validate_address(self, address: str) -> AddressValidation:
        """Validate address.

        Validate a Bitcoin address and get information about its type and scriptPubKey. Returns `isvalid: false` with an error message for invalid addresses.

        *[Mempool.space docs](https://mempool.space/docs/api/rest#get-address-validate)*

        Endpoint: `GET /api/v1/validate-address/{address}`"""
        return self.get_json(f'/api/v1/validate-address/{_segment(address)}')

    def get_block(self, hash: BlockHash) -> BlockInfo:
        """Block information.

        *[Mempool.space docs](https://mempool.space/docs/api/rest#get-block)*

        Endpoint: `GET /api/block/{hash}`"""
        return self.get_json(f'/api/block/{_segment(hash)}')

    def get_block_v1(self, hash: BlockHash) -> BlockInfoV1:
        """Block (v1).

        Returns block details with extras by hash.

        *[Mempool.space docs](https://mempool.space/docs/api/rest#get-block-v1)*

        Endpoint: `GET /api/v1/block/{hash}`"""
        return self.get_json(f'/api/v1/block/{_segment(hash)}')

    def get_block_header(self, hash: BlockHash) -> Hex:
        """Block header.

        Returns the hex-encoded 80-byte block header.

        *[Mempool.space docs](https://mempool.space/docs/api/rest#get-block-header)*

        Endpoint: `GET /api/block/{hash}/header`"""
        return self.get_text(f'/api/block/{_segment(hash)}/header')

    def get_block_by_height(self, height: Height) -> BlockHash:
        """Block hash by height.

        Block hash as plain text.

        *[Mempool.space docs](https://mempool.space/docs/api/rest#get-block-height)*

        Endpoint: `GET /api/block-height/{height}`"""
        return self.get_text(f'/api/block-height/{_segment(height)}')

    def get_block_by_timestamp(self, timestamp: Timestamp) -> BlockTimestamp:
        """Block by timestamp.

        Find the block with the greatest header timestamp at or before the given UNIX timestamp, choosing the earliest height on ties.

        *[Mempool.space docs](https://mempool.space/docs/api/rest#get-block-timestamp)*

        Endpoint: `GET /api/v1/mining/blocks/timestamp/{timestamp}`"""
        return self.get_json(f'/api/v1/mining/blocks/timestamp/{_segment(timestamp)}')

    def get_block_raw(self, hash: BlockHash) -> bytes:
        """Raw block.

        Returns the raw block data in binary format.

        *[Mempool.space docs](https://mempool.space/docs/api/rest#get-block-raw)*

        Endpoint: `GET /api/block/{hash}/raw`"""
        return self.get(f'/api/block/{_segment(hash)}/raw')

    def get_block_status(self, hash: BlockHash) -> BlockStatus:
        """Block status.

        Whether the block is in the best chain and, if so, its height and the hash of the next block. A block outside the best chain (orphaned or unknown) reports `in_best_chain: false`.

        *[Mempool.space docs](https://mempool.space/docs/api/rest#get-block-status)*

        Endpoint: `GET /api/block/{hash}/status`"""
        return self.get_json(f'/api/block/{_segment(hash)}/status')

    def get_block_tip_height(self) -> Height:
        """Block tip height.

        *[Mempool.space docs](https://mempool.space/docs/api/rest#get-block-tip-height)*

        Endpoint: `GET /api/blocks/tip/height`"""
        return int(self.get_text('/api/blocks/tip/height'))

    def get_block_tip_hash(self) -> BlockHash:
        """Block tip hash.

        *[Mempool.space docs](https://mempool.space/docs/api/rest#get-block-tip-hash)*

        Endpoint: `GET /api/blocks/tip/hash`"""
        return self.get_text('/api/blocks/tip/hash')

    def get_block_txid(self, hash: BlockHash, index: BlockTxIndex) -> Txid:
        """Transaction ID at index.

        Retrieve a single transaction ID at a specific index within a block. Returns plain text txid.

        *[Mempool.space docs](https://mempool.space/docs/api/rest#get-block-transaction-id)*

        Endpoint: `GET /api/block/{hash}/txid/{index}`"""
        return self.get_text(f'/api/block/{_segment(hash)}/txid/{_segment(index)}')

    def get_block_txids(self, hash: BlockHash) -> List[Txid]:
        """Block transaction IDs.

        Transaction IDs in block order.

        *[Mempool.space docs](https://mempool.space/docs/api/rest#get-block-transaction-ids)*

        Endpoint: `GET /api/block/{hash}/txids`"""
        return self.get_json(f'/api/block/{_segment(hash)}/txids')

    def get_block_txs(self, hash: BlockHash) -> List[Transaction]:
        """Block transactions.

        Up to 25 transactions starting at index 0.

        *[Mempool.space docs](https://mempool.space/docs/api/rest#get-block-transactions)*

        Endpoint: `GET /api/block/{hash}/txs`"""
        return self.get_json(f'/api/block/{_segment(hash)}/txs')

    def get_block_txs_from_index(self, hash: BlockHash, start_index: BlockTxIndex) -> List[Transaction]:
        """Block transactions (paginated).

        Up to 25 transactions starting at `start_index`.

        *[Mempool.space docs](https://mempool.space/docs/api/rest#get-block-transactions)*

        Endpoint: `GET /api/block/{hash}/txs/{start_index}`"""
        return self.get_json(f'/api/block/{_segment(hash)}/txs/{_segment(start_index)}')

    def get_blocks(self) -> List[BlockInfo]:
        """Recent blocks.

        The last 10 blocks.

        *[Mempool.space docs](https://mempool.space/docs/api/rest#get-blocks)*

        Endpoint: `GET /api/blocks`"""
        return self.get_json('/api/blocks')

    def get_blocks_from_height(self, height: Height) -> List[BlockInfo]:
        """Blocks from height.

        Retrieve up to 10 blocks going backwards from the given height. For example, height=100 returns blocks 100, 99, 98, ..., 91. Height=0 returns only block 0. A height past the tip is not found.

        *[Mempool.space docs](https://mempool.space/docs/api/rest#get-blocks)*

        Endpoint: `GET /api/blocks/{height}`"""
        return self.get_json(f'/api/blocks/{_segment(height)}')

    def get_blocks_v1(self) -> List[BlockInfoV1]:
        """Recent blocks with extras.

        Retrieve the last 15 blocks with extended data including pool identification and fee statistics.

        *[Mempool.space docs](https://mempool.space/docs/api/rest#get-blocks-v1)*

        Endpoint: `GET /api/v1/blocks`"""
        return self.get_json('/api/v1/blocks')

    def get_blocks_v1_from_height(self, height: Height) -> List[BlockInfoV1]:
        """Blocks from height with extras.

        Retrieve up to 15 blocks with extended data going backwards from the given height: the published blocks among the 15 heights ending at it, so fewer (or none) past the tip.

        *[Mempool.space docs](https://mempool.space/docs/api/rest#get-blocks-v1)*

        Endpoint: `GET /api/v1/blocks/{height}`"""
        return self.get_json(f'/api/v1/blocks/{_segment(height)}')

    def get_pools(self) -> List[PoolInfo]:
        """List all mining pools.

        *[Mempool.space docs](https://mempool.space/docs/api/rest#get-mining-pools)*

        Endpoint: `GET /api/v1/mining/pools`"""
        return self.get_json('/api/v1/mining/pools')

    def get_pool_stats(self, time_period: TimePeriod) -> PoolsSummary:
        """Mining pool statistics.

        Get mining pool statistics for a time period.

        *[Mempool.space docs](https://mempool.space/docs/api/rest#get-mining-pools)*

        Endpoint: `GET /api/v1/mining/pools/{time_period}`"""
        return self.get_json(f'/api/v1/mining/pools/{_segment(time_period)}')

    def get_pool(self, slug: PoolSlug) -> PoolDetail:
        """Mining pool details.

        Block counts and mining shares by time period for one pool.

        *[Mempool.space docs](https://mempool.space/docs/api/rest#get-mining-pool)*

        Endpoint: `GET /api/v1/mining/pool/{slug}`"""
        return self.get_json(f'/api/v1/mining/pool/{_segment(slug)}')

    def get_pools_hashrate(self) -> List[PoolHashrateEntry]:
        """All pools hashrate (all time).

        *[Mempool.space docs](https://mempool.space/docs/api/rest#get-mining-pool-hashrates)*

        Endpoint: `GET /api/v1/mining/hashrate/pools`"""
        return self.get_json('/api/v1/mining/hashrate/pools')

    def get_pools_hashrate_by_period(self, time_period: TimePeriod) -> List[PoolHashrateEntry]:
        """All pools hashrate.

        Get hashrate data for all mining pools for a time period.

        *[Mempool.space docs](https://mempool.space/docs/api/rest#get-mining-pool-hashrates)*

        Endpoint: `GET /api/v1/mining/hashrate/pools/{time_period}`"""
        return self.get_json(f'/api/v1/mining/hashrate/pools/{_segment(time_period)}')

    def get_pool_hashrate(self, slug: PoolSlug) -> List[PoolHashrateEntry]:
        """Mining pool hashrate.

        Get hashrate history for a specific mining pool.

        *[Mempool.space docs](https://mempool.space/docs/api/rest#get-mining-pool-hashrate)*

        Endpoint: `GET /api/v1/mining/pool/{slug}/hashrate`"""
        return self.get_json(f'/api/v1/mining/pool/{_segment(slug)}/hashrate')

    def get_pool_blocks(self, slug: PoolSlug) -> List[BlockInfoV1]:
        """Mining pool blocks.

        Get up to 100 recent blocks mined by a specific pool.

        *[Mempool.space docs](https://mempool.space/docs/api/rest#get-mining-pool-blocks)*

        Endpoint: `GET /api/v1/mining/pool/{slug}/blocks`"""
        return self.get_json(f'/api/v1/mining/pool/{_segment(slug)}/blocks')

    def get_pool_blocks_from(self, slug: PoolSlug, height: Height) -> List[BlockInfoV1]:
        """Mining pool blocks before height.

        Get up to 100 blocks mined by a specific pool strictly below the given height (pass the last height of a page to get the next one).

        *[Mempool.space docs](https://mempool.space/docs/api/rest#get-mining-pool-blocks)*

        Endpoint: `GET /api/v1/mining/pool/{slug}/blocks/{height}`"""
        return self.get_json(f'/api/v1/mining/pool/{_segment(slug)}/blocks/{_segment(height)}')

    def get_hashrate(self) -> HashrateSummary:
        """Network hashrate (all time).

        Get network hashrate and difficulty data for all time.

        *[Mempool.space docs](https://mempool.space/docs/api/rest#get-hashrate)*

        Endpoint: `GET /api/v1/mining/hashrate`"""
        return self.get_json('/api/v1/mining/hashrate')

    def get_hashrate_by_period(self, time_period: TimePeriod) -> HashrateSummary:
        """Network hashrate.

        Get network hashrate and difficulty data for a time period.

        *[Mempool.space docs](https://mempool.space/docs/api/rest#get-hashrate)*

        Endpoint: `GET /api/v1/mining/hashrate/{time_period}`"""
        return self.get_json(f'/api/v1/mining/hashrate/{_segment(time_period)}')

    def get_difficulty_adjustments(self) -> List[DifficultyAdjustmentEntry]:
        """Difficulty adjustments (all time).

        *[Mempool.space docs](https://mempool.space/docs/api/rest#get-difficulty-adjustments)*

        Endpoint: `GET /api/v1/mining/difficulty-adjustments`"""
        return self.get_json('/api/v1/mining/difficulty-adjustments')

    def get_difficulty_adjustments_by_period(self, time_period: TimePeriod) -> List[DifficultyAdjustmentEntry]:
        """Difficulty adjustments.

        Get historical difficulty adjustments for a time period.

        *[Mempool.space docs](https://mempool.space/docs/api/rest#get-difficulty-adjustments)*

        Endpoint: `GET /api/v1/mining/difficulty-adjustments/{time_period}`"""
        return self.get_json(f'/api/v1/mining/difficulty-adjustments/{_segment(time_period)}')

    def get_reward_stats(self, block_count: int) -> RewardStats:
        """Mining reward statistics.

        Get mining reward statistics for the last N blocks including total rewards, fees, and transaction count.

        *[Mempool.space docs](https://mempool.space/docs/api/rest#get-reward-stats)*

        Endpoint: `GET /api/v1/mining/reward-stats/{block_count}`"""
        return self.get_json(f'/api/v1/mining/reward-stats/{_segment(block_count)}')

    def get_block_fees(self, time_period: TimePeriod) -> List[BlockFeesEntry]:
        """Block fees.

        Get average total fees per block for a time period.

        *[Mempool.space docs](https://mempool.space/docs/api/rest#get-block-fees)*

        Endpoint: `GET /api/v1/mining/blocks/fees/{time_period}`"""
        return self.get_json(f'/api/v1/mining/blocks/fees/{_segment(time_period)}')

    def get_block_rewards(self, time_period: TimePeriod) -> List[BlockRewardsEntry]:
        """Block rewards.

        Get average coinbase reward (subsidy + fees) per block for a time period.

        *[Mempool.space docs](https://mempool.space/docs/api/rest#get-block-rewards)*

        Endpoint: `GET /api/v1/mining/blocks/rewards/{time_period}`"""
        return self.get_json(f'/api/v1/mining/blocks/rewards/{_segment(time_period)}')

    def get_block_fee_rates(self, time_period: TimePeriod) -> List[BlockFeeRatesEntry]:
        """Block fee rates.

        Get block fee rate percentiles (min, 10th, 25th, median, 75th, 90th, max) for a time period.

        *[Mempool.space docs](https://mempool.space/docs/api/rest#get-block-feerates)*

        Endpoint: `GET /api/v1/mining/blocks/fee-rates/{time_period}`"""
        return self.get_json(f'/api/v1/mining/blocks/fee-rates/{_segment(time_period)}')

    def get_block_sizes_weights(self, time_period: TimePeriod) -> BlockSizesWeights:
        """Block sizes and weights.

        Get average block sizes and weights for a time period.

        *[Mempool.space docs](https://mempool.space/docs/api/rest#get-sizes-weights)*

        Endpoint: `GET /api/v1/mining/blocks/sizes-weights/{time_period}`"""
        return self.get_json(f'/api/v1/mining/blocks/sizes-weights/{_segment(time_period)}')

    def get_mempool_blocks(self) -> List[MempoolBlock]:
        """Projected mempool blocks.

        Projected blocks for fee estimation. Block 0 reflects Bitcoin Core's actual next-block selection; blocks 1+ are a fee-tier approximation.

        *[Mempool.space docs](https://mempool.space/docs/api/rest#get-mempool-blocks-fees)*

        Endpoint: `GET /api/v1/fees/mempool-blocks`"""
        return self.get_json('/api/v1/fees/mempool-blocks')

    def get_recommended_fees(self) -> RecommendedFees:
        """Recommended fees.

        Recommended fee rates by confirmation target, in whole sat/vB.

        *[Mempool.space docs](https://mempool.space/docs/api/rest#get-recommended-fees)*

        Endpoint: `GET /api/v1/fees/recommended`"""
        return self.get_json('/api/v1/fees/recommended')

    def get_precise_fees(self) -> PreciseRecommendedFees:
        """Recommended fee rates (precise).

        Recommended fee rates by confirmation target, with up to three decimal places and support for sub-sat/vB rates.

        *[Mempool.space docs](https://mempool.space/docs/api/rest#get-recommended-fees-precise)*

        Endpoint: `GET /api/v1/fees/precise`"""
        return self.get_json('/api/v1/fees/precise')

    def get_mempool(self) -> MempoolInfo:
        """Mempool statistics.

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

        Returns up to 25 most-recent RBF replacement trees across the whole mempool. Each entry has the shape of `replacements` in `GET /api/v1/tx/{txid}/rbf`.

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

        Delta of the projected next block since `<hash>`. `order` is the full new template in order: each entry is either a number (index into the prior template the client cached at `<hash>`) or a transaction object (new body to insert at this position). Walk `order` once to rebuild and replace the cached `stats` with the response's; `removed` is a convenience list of txids that left so clients can evict cached bodies. After applying, use the response `hash` as `<hash>` on the next call to keep iterating. Returns `404` when `<hash>` has aged out of server history; clients should fall back to `GET /api/v1/mempool/block-template`.

        Endpoint: `GET /api/v1/mempool/block-template/diff/{hash}`"""
        return self.get_json(f'/api/v1/mempool/block-template/diff/{_segment(hash)}')

    def get_live_price(self) -> Dollars:
        """Live BTC/USD price.

        Current BTC/USD price in dollars, derived from on-chain round-dollar output patterns in the oracle's recent-block window, with the mempool's outputs blended in when available.

        Endpoint: `GET /api/mempool/price`"""
        return self.get_json('/api/mempool/price')

    def get_tx_by_index(self, index: TxIndex) -> Txid:
        """Txid by index.

        Retrieve the transaction ID (txid) at a given global transaction index. Returns the txid as plain text.

        Endpoint: `GET /api/tx-index/{index}`"""
        return self.get_text(f'/api/tx-index/{_segment(index)}')

    def get_cpfp(self, txid: Txid) -> CpfpInfo:
        """CPFP info.

        Returns ancestors and descendants for a CPFP (Child Pays For Parent) transaction, including the effective fee rate of the package.

        *[Mempool.space docs](https://mempool.space/docs/api/rest#get-children-pay-for-parent)*

        Endpoint: `GET /api/v1/cpfp/{txid}`"""
        return self.get_json(f'/api/v1/cpfp/{_segment(txid)}')

    def get_tx_rbf(self, txid: Txid) -> RbfResponse:
        """RBF replacement history.

        Returns the RBF replacement tree for a transaction, if any. Both `replacements` and `replaces` are null when the tx has no known RBF history within the mempool monitor's retention window.

        *[Mempool.space docs](https://mempool.space/docs/api/rest#get-transaction-rbf-history)*

        Endpoint: `GET /api/v1/tx/{txid}/rbf`"""
        return self.get_json(f'/api/v1/tx/{_segment(txid)}/rbf')

    def get_tx(self, txid: Txid) -> Transaction:
        """Transaction information.

        *[Mempool.space docs](https://mempool.space/docs/api/rest#get-transaction)*

        Endpoint: `GET /api/tx/{txid}`"""
        return self.get_json(f'/api/tx/{_segment(txid)}')

    def get_tx_hex(self, txid: Txid) -> Hex:
        """Transaction hex.

        Hex-encoded serialized transaction.

        *[Mempool.space docs](https://mempool.space/docs/api/rest#get-transaction-hex)*

        Endpoint: `GET /api/tx/{txid}/hex`"""
        return self.get_text(f'/api/tx/{_segment(txid)}/hex')

    def get_tx_merkleblock_proof(self, txid: Txid) -> Hex:
        """Transaction merkleblock proof.

        Get the merkleblock proof for a transaction (BIP37 format, hex encoded).

        *[Mempool.space docs](https://mempool.space/docs/api/rest#get-transaction-merkleblock-proof)*

        Endpoint: `GET /api/tx/{txid}/merkleblock-proof`"""
        return self.get_text(f'/api/tx/{_segment(txid)}/merkleblock-proof')

    def get_tx_merkle_proof(self, txid: Txid) -> MerkleProof:
        """Transaction merkle proof.

        *[Mempool.space docs](https://mempool.space/docs/api/rest#get-transaction-merkle-proof)*

        Endpoint: `GET /api/tx/{txid}/merkle-proof`"""
        return self.get_json(f'/api/tx/{_segment(txid)}/merkle-proof')

    def get_tx_outspend(self, txid: Txid, vout: Vout) -> TxOutspend:
        """Output spend status.

        Whether the output has been spent and, if so, the spending transaction details.

        *[Mempool.space docs](https://mempool.space/docs/api/rest#get-transaction-outspend)*

        Endpoint: `GET /api/tx/{txid}/outspend/{vout}`"""
        return self.get_json(f'/api/tx/{_segment(txid)}/outspend/{_segment(vout)}')

    def get_tx_outspends(self, txid: Txid) -> List[TxOutspend]:
        """All output spend statuses.

        Spend statuses in output-index order.

        *[Mempool.space docs](https://mempool.space/docs/api/rest#get-transaction-outspends)*

        Endpoint: `GET /api/tx/{txid}/outspends`"""
        return self.get_json(f'/api/tx/{_segment(txid)}/outspends')

    def get_tx_raw(self, txid: Txid) -> bytes:
        """Transaction raw.

        Returns a transaction as binary data.

        *[Mempool.space docs](https://mempool.space/docs/api/rest#get-transaction-raw)*

        Endpoint: `GET /api/tx/{txid}/raw`"""
        return self.get(f'/api/tx/{_segment(txid)}/raw')

    def get_tx_status(self, txid: Txid) -> TxStatus:
        """Transaction status.

        Retrieve the confirmation status of a transaction. Returns whether the transaction is confirmed and, if so, the block height, hash, and timestamp.

        *[Mempool.space docs](https://mempool.space/docs/api/rest#get-transaction-status)*

        Endpoint: `GET /api/tx/{txid}/status`"""
        return self.get_json(f'/api/tx/{_segment(txid)}/status')

    def get_transaction_times(self, txId: List[Txid]) -> List[int]:
        """Transaction first-seen times.

        First-seen UNIX timestamps in seconds, in request order, of transactions in the mempool or that recently left it other than by replacement (mined ones included); 0 otherwise.

        *[Mempool.space docs](https://mempool.space/docs/api/rest#get-transaction-times)*

        Endpoint: `GET /api/v1/transaction-times`"""
        params: List[Tuple[str, Any]] = []
        for _v in txId: params.append(('txId[]', _v))
        query = urlencode(params)
        path = f'/api/v1/transaction-times{"?" + query if query else ""}'
        return self.get_json(path)

    def post_tx(self, body: str) -> Txid:
        """Broadcast transaction.

        Submit a raw transaction as hexadecimal text (at most 8,000,000 request bytes, including whitespace). Returns its txid as plain text. No responses are cached. Cancellation or a transport error after dispatch may leave the submission outcome unknown; do not automatically retry.

        *[Mempool.space docs](https://mempool.space/docs/api/rest#post-transaction)*

        Endpoint: `POST /api/tx`"""
        return self.post_text('/api/tx', body)

    def get_utxo_set_latest(self) -> UtxoSet:
        """Latest UTXO set.

        The UTXO set after the latest published block, in the layout of `GET /api/utxo-set/{point}`.

        Endpoint: `GET /api/utxo-set`"""
        return self.get_json('/api/utxo-set')

    def get_utxo_set(self, point: str) -> UtxoSet:
        """UTXO set at block height or date.

        The UTXO set after a block (`840000`) or after the last published block of a UTC day (`YYYY-MM-DD`), by creation height. Returns `{ height, hash, date, count, supply, origins }`; entry `i` of `origins.count` and `origins.supply` (BTC) is what remains unspent of block `i`'s outputs.

        Endpoint: `GET /api/utxo-set/{point}`"""
        return self.get_json(f'/api/utxo-set/{_segment(point)}')

    def get_utxo_set_diff(self, point: str) -> UtxoSetDiff:
        """UTXO set changes of a block or date.

        What a block (`840000`) or a UTC day's published blocks (`YYYY-MM-DD`) changed in the UTXO set. Returns `{ first, last, hash, date, created, spent }`: `created` has one row per block, `spent` one row per creation height, each as columnar `height`, `count` and `supply` (BTC). The set after block `first - 1` plus `created` minus `spent` is the set after block `last`.

        Endpoint: `GET /api/utxo-set/{point}/diff`"""
        return self.get_json(f'/api/utxo-set/{_segment(point)}/diff')

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
        return self.get_json(f'/api/oracle/histogram/payments/{_segment(point)}')

    def get_oracle_histogram_outputs_live(self) -> List[int]:
        """Live output value histogram.

        Live unfiltered output value histogram for the complete published mempool. Nonzero outputs within the histogram's bin range are counted by value on the oracle log scale; no oracle payment filters are applied. A flat array of log-scale bins, all zero when no mempool is configured.

        Endpoint: `GET /api/oracle/histogram/outputs/live`"""
        return self.get_json('/api/oracle/histogram/outputs/live')

    def get_oracle_histogram_outputs(self, point: str) -> List[int]:
        """Output value histogram at height or day.

        Unfiltered output value histogram for a confirmed point. A block height (`840000`) counts that block's nonzero outputs within the histogram's bin range, coinbase included, by value on the oracle log scale; a calendar date (`YYYY-MM-DD`) sums every block that day. A flat array of log-scale bins.

        Endpoint: `GET /api/oracle/histogram/outputs/{point}`"""
        return self.get_json(f'/api/oracle/histogram/outputs/{_segment(point)}')

    def get_openapi(self) -> Any:
        """OpenAPI specification.

        Full OpenAPI 3.1 specification for this API. Build tools from this document.

        Endpoint: `GET /openapi.json`"""
        return self.get_json('/openapi.json')

    def get_api(self) -> Any:
        """Compact API summary.

        Compact custom summary of the OpenAPI specification for LLM reading: types instead of schemas, `returns` instead of responses, no examples; parameters keep their enum, default and bounds. Not valid OpenAPI: build tools from `GET /openapi.json`.

        Endpoint: `GET /api.json`"""
        return self.get_json('/api.json')
