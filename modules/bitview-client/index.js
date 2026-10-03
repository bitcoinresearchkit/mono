// Auto-generated Bitview JavaScript client
// Do not edit manually

// Type definitions

/**
 * Bitcoin address string
 *
 * @typedef {string} Addr
 */
/**
 * Bitcoin address + last-seen txid path parameters (Esplora-style pagination)
 *
 * @typedef {Object} AddrAfterTxidParam
 * @property {Addr} address
 * @property {Txid} afterTxid - Last txid from the previous page (return transactions strictly older than this)
 */
/**
 * Address statistics on the blockchain (confirmed transactions only)
 *
 * Based on mempool.space's format with type_index extension.
 *
 * @typedef {Object} AddrChainStats
 * @property {Sats} balance - Current confirmed balance in satoshis
 * @property {number} fundedTxoCount - Total number of transaction outputs that funded this address
 * @property {Sats} fundedTxoSum - Total amount in satoshis received by this address across all funded outputs
 * @property {number} spentTxoCount - Total number of transaction outputs spent from this address
 * @property {Sats} spentTxoSum - Total amount in satoshis spent from this address
 * @property {number} txCount - Total number of confirmed transactions involving this address
 * @property {TypeIndex} typeIndex - Index of this address within its type on the blockchain
 * @property {Dollars} realizedPrice - Realized price (average cost basis) in USD
 */
/**
 * @typedef {Object} AddrHashPrefixMatches
 * @property {OutputType} addrType
 * @property {string} prefix
 * @property {boolean} truncated
 * @property {Addr[]} addresses
 */
/**
 * @typedef {Object} AddrHashPrefixParam
 * @property {OutputType} addrType
 * @property {string} prefix - First 1–16 hexadecimal nibbles of the RapidHash v3 hash over the raw
address payload bytes.
 */
/**
 * Address statistics in the mempool (unconfirmed transactions only)
 *
 * Based on mempool.space's format.
 *
 * @typedef {Object} AddrMempoolStats
 * @property {SatsSigned} balanceDelta - Net pending (unconfirmed) balance change in satoshis; negative when pending spends exceed receipts
 * @property {number} fundedTxoCount - Number of unconfirmed transaction outputs funding this address
 * @property {Sats} fundedTxoSum - Total amount in satoshis being received in unconfirmed transactions
 * @property {number} spentTxoCount - Number of unconfirmed transaction inputs spending from this address
 * @property {Sats} spentTxoSum - Total amount in satoshis being spent in unconfirmed transactions
 * @property {number} txCount - Number of unconfirmed transactions involving this address
 */
/**
 * Bitcoin address path parameter
 *
 * @typedef {Object} AddrParam
 * @property {Addr} address
 */
/**
 * Four-byte primary state stored for every address.
 *
 * Empty addresses with small lifetime totals are stored inline. The upper two
 * bits select an inline layout or a sidecar, whose index occupies the lower 30
 * bits.
 *
 * @typedef {number} AddrState
 */
/**
 * Address information compatible with mempool.space API format.
 *
 * @typedef {Object} AddrStats
 * @property {Addr} address - Bitcoin address string
 * @property {OutputType} addrType - BRK address type (p2pk33, p2pk65, p2pkh, p2sh, p2wpkh, p2wsh, p2tr, etc.)
 * @property {AddrChainStats} chainStats - Statistics for confirmed transactions on the blockchain
 * @property {AddrMempoolStats} mempoolStats - Statistics for unconfirmed transactions in the mempool
 * @property {Sats} balance - Total current balance in satoshis, including pending (unconfirmed) mempool changes
 */
/**
 * Address validation result
 *
 * @typedef {Object} AddrValidation
 * @property {boolean} isvalid - Whether the address is valid
 * @property {?string=} address - The validated address
 * @property {?string=} scriptPubKey - The scriptPubKey in hex
 * @property {?boolean=} isscript - Whether this is a script address (P2SH)
 * @property {?boolean=} iswitness - Whether this is a witness address
 * @property {?number=} witnessVersion - Witness version (0 for P2WPKH/P2WSH, 1 for P2TR)
 * @property {?string=} witnessProgram - Witness program in hex
 * @property {?number[]=} errorLocations - Error locations (empty array for most errors)
 * @property {?string=} error - Error message for invalid addresses
 */
/**
 * Unsigned basis points: 10,000 represents the ratio 1.
 * Maximum finite ratio: 429,496.7294. u32::MAX represents undefined.
 * Finite input range is a debug-checked precondition, not a saturation policy.
 * Serde preserves raw encoded bits; vector JSON emits null for undefined.
 *
 * @typedef {number} BasisPoints32
 */
/**
 * Bitcoin amount as floating point (1 BTC = 100,000,000 satoshis)
 *
 * @typedef {number} Bitcoin
 */
/**
 * Block count path parameter
 *
 * @typedef {Object} BlockCountParam
 * @property {number} blockCount - Number of recent blocks to include
 */
/**
 * Extended block data matching mempool.space /api/v1/blocks extras
 *
 * @typedef {Object} BlockExtras
 * @property {Sats} totalFees - Total fees in satoshis
 * @property {FeeRate} medianFee - Median fee rate in sat/vB
 * @property {FeeRate[]} feeRange - Fee rate range: [min, 10%, 25%, 50%, 75%, 90%, max]
 * @property {Sats} reward - Total block reward (subsidy + fees) in satoshis
 * @property {BlockPool} pool - Mining pool that mined this block
 * @property {Sats} avgFee - Average fee per transaction in satoshis
 * @property {FeeRate} avgFeeRate - Average fee rate in sat/vB
 * @property {string} coinbaseRaw - Raw coinbase transaction scriptsig as hex
 * @property {?string=} coinbaseAddress - Primary coinbase output address
 * @property {string[]} coinbaseAddresses - All coinbase output addresses
 * @property {string} coinbaseSignature - Coinbase output script in ASM format
 * @property {string} coinbaseSignatureAscii - Coinbase scriptsig decoded as ASCII
 * @property {number} avgTxSize - Average transaction size in bytes
 * @property {number} totalInputs - Total number of inputs (excluding coinbase)
 * @property {number} totalOutputs - Total number of outputs
 * @property {Sats} totalOutputAmt - Total output amount in satoshis
 * @property {Sats} medianFeeAmt - Median fee amount in satoshis
 * @property {Sats[]} feePercentiles - Fee amount percentiles in satoshis: [min, 10%, 25%, 50%, 75%, 90%, max]
 * @property {number} segwitTotalTxs - Number of segwit transactions
 * @property {number} segwitTotalSize - Total size of segwit transactions in bytes
 * @property {Weight} segwitTotalWeight - Total weight of segwit transactions
 * @property {string} header - Raw 80-byte block header as hex
 * @property {number} utxoSetChange - UTXO set change (total outputs - total inputs, includes unspendable like OP_RETURN).
Note: intentionally differs from utxo_set_size diff which excludes unspendable outputs.
Matches mempool.space/bitcoin-cli behavior.
 * @property {number} utxoSetSize - Total spendable UTXO set size at this height (excludes OP_RETURN and other unspendable outputs)
 * @property {Sats} totalInputAmt - Total input amount in satoshis
 * @property {number} virtualSize - Virtual size in vbytes
 * @property {?number=} firstSeen - Timestamp when the block was first seen (always null, not yet supported)
 * @property {string[]} orphans - Orphaned blocks (always empty)
 * @property {Dollars} price - USD price at block height
 */
/**
 * A single block fee rates data point with percentiles.
 *
 * @typedef {Object} BlockFeeRatesEntry
 * @property {Height} avgHeight - Average block height in this window
 * @property {Timestamp} timestamp - Unix timestamp at the window midpoint
 * @property {FeeRate} avgFee0 - Minimum fee rate (sat/vB)
 * @property {FeeRate} avgFee10 - 10th percentile fee rate (sat/vB)
 * @property {FeeRate} avgFee25 - 25th percentile fee rate (sat/vB)
 * @property {FeeRate} avgFee50 - Median fee rate (sat/vB)
 * @property {FeeRate} avgFee75 - 75th percentile fee rate (sat/vB)
 * @property {FeeRate} avgFee90 - 90th percentile fee rate (sat/vB)
 * @property {FeeRate} avgFee100 - Maximum fee rate (sat/vB)
 */
/**
 * A single block fees data point.
 *
 * @typedef {Object} BlockFeesEntry
 * @property {Height} avgHeight - Average block height in this window
 * @property {Timestamp} timestamp - Unix timestamp at the window midpoint
 * @property {Sats} avgFees - Average fees per block in this window (sats)
 * @property {Dollars} uSD - BTC/USD price at this height
 */
/**
 * Double-SHA256 block-header hash, serialized in Bitcoin's conventional
 * hexadecimal byte order.
 *
 * @typedef {string} BlockHash
 */
/**
 * Block hash path parameter
 *
 * @typedef {Object} BlockHashParam
 * @property {BlockHash} hash
 */
/**
 * Block hash + starting transaction index path parameters
 *
 * @typedef {Object} BlockHashStartIndex
 * @property {BlockHash} hash - Bitcoin block hash
 * @property {BlockTxIndex} startIndex - Starting transaction index within the block (0-based)
 */
/**
 * Block hash + transaction index path parameters
 *
 * @typedef {Object} BlockHashTxIndex
 * @property {BlockHash} hash - Bitcoin block hash
 * @property {BlockTxIndex} index - Transaction index within the block (0-based)
 */
/**
 * Block information matching mempool.space /api/block/{hash}
 *
 * @typedef {Object} BlockInfo
 * @property {BlockHash} id - Block hash
 * @property {Height} height - Block height
 * @property {number} version - Block version
 * @property {Timestamp} timestamp - Block timestamp (Unix time)
 * @property {number} bits - Compact target (bits)
 * @property {number} nonce - Nonce
 * @property {number} difficulty - Block difficulty
 * @property {string} merkleRoot - Merkle root of the transaction tree
 * @property {number} txCount - Number of transactions
 * @property {number} size - Block size in bytes
 * @property {Weight} weight - Block weight in weight units
 * @property {BlockHash} previousblockhash - Previous block hash
 * @property {Timestamp} mediantime - Median time of the last 11 blocks
 */
/**
 * Block information with extras, matching mempool.space /api/v1/blocks
 *
 * @typedef {Object} BlockInfoV1
 * @property {BlockHash} id - Block hash
 * @property {Height} height - Block height
 * @property {number} version - Block version
 * @property {Timestamp} timestamp - Block timestamp (Unix time)
 * @property {number} bits - Compact target (bits)
 * @property {number} nonce - Nonce
 * @property {number} difficulty - Block difficulty
 * @property {string} merkleRoot - Merkle root of the transaction tree
 * @property {number} txCount - Number of transactions
 * @property {number} size - Block size in bytes
 * @property {Weight} weight - Block weight in weight units
 * @property {BlockHash} previousblockhash - Previous block hash
 * @property {Timestamp} mediantime - Median time of the last 11 blocks
 * @property {boolean=} stale - Whether this block has been replaced by a longer chain
 * @property {BlockExtras} extras - Extended block data
 */
/**
 * Mining pool identification for a block
 *
 * @typedef {Object} BlockPool
 * @property {number} id - Unique pool identifier
 * @property {string} name - Pool name
 * @property {PoolSlug} slug - URL-friendly pool identifier
 * @property {number} blockNumber - This block's ordinal among blocks attributed to this pool
 * @property {?string[]=} minerNames - Miner name tags found in coinbase scriptsig
 */
/**
 * A single block rewards data point.
 *
 * @typedef {Object} BlockRewardsEntry
 * @property {Height} avgHeight - Average block height in this window
 * @property {Timestamp} timestamp - Unix timestamp at the window midpoint
 * @property {Sats} avgRewards - Average coinbase reward per block (subsidy + fees, sats)
 * @property {Dollars} uSD - BTC/USD price at this height
 */
/**
 * A single block size data point.
 *
 * @typedef {Object} BlockSizeEntry
 * @property {Height} avgHeight - Average block height in this window
 * @property {Timestamp} timestamp - Unix timestamp at the window midpoint
 * @property {number} avgSize - Rolling 24h median block size (bytes)
 */
/**
 * Combined block sizes and weights response.
 *
 * @typedef {Object} BlockSizesWeights
 * @property {BlockSizeEntry[]} sizes - Block size data points
 * @property {BlockWeightEntry[]} weights - Block weight data points
 */
/**
 * Block status indicating whether block is in the best chain
 *
 * @typedef {Object} BlockStatus
 * @property {boolean} inBestChain - Whether this block is in the best chain
 * @property {(Height|null)=} height - Block height (only if in best chain)
 * @property {(BlockHash|null)=} nextBest - Hash of the next block in the best chain (null if tip)
 */
/**
 * Projected next-block contents from Bitcoin Core's `getblocktemplate`
 * (block 0 of the snapshot). Returned by
 * `GET /api/v1/mempool/block-template`.
 *
 * @typedef {Object} BlockTemplate
 * @property {NextBlockHash} hash - Pass to `GET /api/v1/mempool/block-template/diff/{hash}` to fetch deltas.
 * @property {MempoolBlock} stats - Aggregate stats for this block (size, vsize, fee range, ...).
 * @property {Transaction[]} transactions - Full transaction bodies in `getblocktemplate` order.
 */
/**
 * Delta between the current `getblocktemplate` projection and a prior
 * one identified by `since`. Returned by
 * `GET /api/v1/mempool/block-template/diff/{hash}`.
 *
 * `order` carries the full new template in template order: each entry
 * is either a `Retained(idx)` pointing into the prior template (which
 * the client cached at `since`) or a `New(tx)` inline body. Walk it
 * once to rebuild the new template; no separate `added` array to
 * cross-reference.
 *
 * `removed` lists txids no longer present. A changed body can be emitted as
 * `New` without removing its txid; absence of a retained index alone does not
 * imply removal.
 *
 * @typedef {Object} BlockTemplateDiff
 * @property {NextBlockHash} hash - Current next-block hash. Use as `since` on the next diff call.
 * @property {NextBlockHash} since - Echoed prior hash the diff was computed against.
 * @property {BlockTemplateDiffEntry[]} order - New template in order. Each entry is either an index into the
prior template's transactions or a full transaction body.
 * @property {Txid[]} removed - Txids that left the projected next block since `since`
(confirmed, evicted, replaced, or pushed past block 0).
 */
/**
 * One slot of the new template in a `BlockTemplateDiff`.
 *
 * Untagged on the wire so JSON type disambiguates the variants:
 * - `Retained(idx)` serializes as a bare integer - index into the
 *   transactions of the prior template (which the client cached at
 *   `since`).
 * - `New(tx)` serializes as a transaction object - a body that was
 *   new or changed since the prior template and must replace this position.
 *
 * Reconstruction is a single pass: for each entry, either copy
 * `prior[idx]` or append the inline body.
 *
 * @typedef {(number|Transaction)} BlockTemplateDiffEntry
 */
/**
 * Block information returned for timestamp queries
 *
 * @typedef {Object} BlockTimestamp
 * @property {Height} height - Block height
 * @property {BlockHash} hash - Block hash
 * @property {string} timestamp - Block timestamp in ISO 8601 format
 */
/**
 * Position of a transaction within a single block (0 = coinbase).
 * Distinct from `TxIndex`, which is the chain-wide global tx index.
 *
 * @typedef {number} BlockTxIndex
 */
/**
 * A single block weight data point.
 *
 * @typedef {Object} BlockWeightEntry
 * @property {Height} avgHeight - Average block height in this window
 * @property {Timestamp} timestamp - Unix timestamp at the window midpoint
 * @property {Weight} avgWeight - Rolling 24h median block weight (weight units)
 */
/**
 * A ratio in [0, 1], floored at scale u32::MAX - 1.
 * Zero and one are exact; finite quantization error is less than 1 / SCALE
 * apart from floating-point arithmetic error. u32::MAX represents undefined.
 * Non-finite inputs become undefined. Finite inputs must lie in [0, 1]; this
 * precondition is checked only in debug builds. Keep cumulative state unrounded.
 * Serde preserves raw encoded bits; vector JSON emits null for undefined.
 *
 * @typedef {number} BoundedRatio
 */
/** @typedef {number} Bytes */
/**
 * Investor phase from the Capital Sentiment model.
 *
 * Codes are explicit because phase values are persisted. Code `0` represents
 * unavailable model inputs and is therefore not a phase.
 *
 * @typedef {("raging_bull"|"bull"|"cautious_bull"|"hopeful_bull"|"early_bull"|"weak_bull"|"limbo"|"deep_bear"|"bear"|"early_bear")} CapitalSentimentPhase
 */
/**
 * Unsigned cents (u64) - for values that should never be negative.
 * Used for invested capital, realized cap, etc.
 * `u64::MAX` is reserved as a NaN sentinel.
 *
 * @typedef {number} Cents
 */
/**
 * Signed cents (i64) - for values that can be negative.
 * Used for profit/loss calculations, deltas, etc.
 *
 * @typedef {number} CentsSigned
 */
/**
 * Closing price value for a time period
 *
 * @typedef {Dollars} Close
 */
/**
 * URPD cohort identifier. Use `GET /api/urpd` to list available cohorts.
 *
 * Names are non-empty ASCII `[a-z0-9_]+`. Availability is determined by
 * supported age filters and published UTXO history.
 *
 * @typedef {string} Cohort
 */
/**
 * Up to the first 100 bytes of a coinbase transaction's first-input
 * `scriptSig`. Bytes are preserved for storage and exposed as a string by
 * mapping each byte to the same-valued Unicode code point. Pool attribution
 * may search this raw value, but the value itself is not a normalized pool
 * label.
 *
 * Stored as a fixed 101-byte record (1 byte length + 100 bytes data).
 * Uses `[u8; 101]` internally so that `size_of::<CoinbaseTag>()` matches
 * the serialized `Bytes::Array` size (vecdb requires this for alignment).
 *
 * Bitcoin consensus limits coinbase scriptSig to 2-100 bytes.
 *
 * @typedef {string} CoinbaseTag
 */
/**
 * CPFP cluster: the connected component the seed belongs to, plus its
 * SFL linearization.
 *
 * @typedef {Object} CpfpCluster
 * @property {CpfpClusterTx[]} txs - All txs in the cluster, in topological order (parents before children).
 * @property {CpfpClusterChunk[]} chunks - SFL-emitted chunks ordered by descending feerate.
 * @property {number} chunkIndex - Index into `chunks` of the chunk containing the seed tx.
 */
/**
 * One SFL chunk inside a `CpfpCluster`. `txs` is in topological order
 * (matches `CpfpCluster.txs` ordering); the chunk's `feerate` is the
 * per-chunk SFL feerate and is the same for every tx in this chunk.
 *
 * @typedef {Object} CpfpClusterChunk
 * @property {CpfpClusterTxIndex[]} txs
 * @property {FeeRate} feerate
 */
/**
 * One entry in a `CpfpCluster.txs` array.
 *
 * @typedef {Object} CpfpClusterTx
 * @property {Txid} txid
 * @property {Weight} weight
 * @property {Sats} fee
 * @property {CpfpClusterTxIndex[]} parents - In-cluster parents of this tx.
 */
/**
 * Position of a transaction inside a `CpfpCluster.txs` array. Cluster-local,
 * has no meaning outside the enclosing cluster.
 *
 * @typedef {number} CpfpClusterTxIndex
 */
/**
 * A transaction in a CPFP relationship.
 *
 * @typedef {Object} CpfpEntry
 * @property {Txid} txid
 * @property {Weight} weight
 * @property {Sats} fee
 */
/**
 * CPFP (Child Pays For Parent) information for a transaction.
 *
 * @typedef {Object} CpfpInfo
 * @property {CpfpEntry[]} ancestors - Ancestor transactions in the CPFP chain.
 * @property {(CpfpEntry|null)=} bestDescendant - Best (highest fee rate) descendant, if any.
 * @property {CpfpEntry[]} descendants - Descendant transactions in the CPFP chain.
 * @property {FeeRate} effectiveFeePerVsize - Effective fee rate considering CPFP relationships (sat/vB).
This is the seed's chunk feerate after lift-merging, i.e. the
rate Core/mempool.space would surface for this tx.
 * @property {SigOps} sigops - BIP-141 sigop cost for the seed tx (witness sigops count as 1,
legacy and P2SH-redeem sigops count as 4).
 * @property {Sats} fee - Transaction fee (sats).
 * @property {VSize} vsize - Virtual size of the seed tx (vbytes).
 * @property {VSize} adjustedVsize - Policy-adjusted virtual size: `max(vsize, sigops * 5)`.
 * @property {(CpfpCluster|null)=} cluster - Cluster the seed belongs to: full tx list, SFL-linearized chunks,
and the seed's chunk index. Omitted when the seed has no
ancestors and no descendants (matches mempool.space).
 */
/**
 * Range parameters with output format for API query parameters.
 *
 * @typedef {Object} DataRangeFormat
 * @property {(RangeIndex|null)=} start - Inclusive start: integer index, date (YYYY-MM-DD), or timestamp (ISO 8601). Negative integers count from end. Aliases: `from`, `f`, `s`
 * @property {(RangeIndex|null)=} end - Exclusive end: integer index, date (YYYY-MM-DD), or timestamp (ISO 8601). Negative integers count from end. Aliases: `to`, `t`, `e`
 * @property {(Limit|null)=} limit - Maximum number of values to return (ignored if `end` is set). Aliases: `count`, `c`, `l`
 * @property {Format=} format - Format of the output
 */
/**
 * Calendar date in YYYY-MM-DD format.
 *
 * @typedef {string} Date
 */
/** @typedef {number} Day1 */
/** @typedef {number} Day3 */
/**
 * Detailed series count with per-database breakdown.
 *
 * @typedef {Object} DetailedSeriesCount
 * @property {number} distinct - Number of unique series available (e.g., realized_price, market_cap)
 * @property {number} total - Total number of series-index combinations across all timeframes
 * @property {number} lazy - Number of lazy (computed on-the-fly) series-index combinations
 * @property {number} stored - Number of eager (stored on disk) series-index combinations
 * @property {{ [key: string]: SeriesCount }} byDb - Per-database breakdown of counts.
 */
/**
 * Difficulty adjustment information.
 *
 * @typedef {Object} DifficultyAdjustment
 * @property {number} progressPercent - Progress through current difficulty epoch (0-100%)
 * @property {number} difficultyChange - Estimated difficulty change at next retarget (%)
 * @property {number} estimatedRetargetDate - Estimated timestamp of next retarget (milliseconds)
 * @property {number} remainingBlocks - Blocks remaining until retarget
 * @property {number} remainingTime - Estimated time until retarget (milliseconds)
 * @property {number} previousRetarget - Previous difficulty adjustment (%)
 * @property {Timestamp} previousTime - Timestamp of most recent retarget (seconds)
 * @property {Height} nextRetargetHeight - Height of next retarget
 * @property {number} timeAvg - Average block time in current epoch (milliseconds)
 * @property {number} adjustedTimeAvg - Time-adjusted average (milliseconds)
 * @property {number} timeOffset - Time offset from expected schedule (seconds)
 * @property {number} expectedBlocks - Expected blocks based on wall clock time since epoch start
 */
/**
 * A single difficulty adjustment entry.
 * Serializes as array: [timestamp, height, difficulty, change_percent]
 *
 * @typedef {number[]} DifficultyAdjustmentEntry
 */
/**
 * A single difficulty data point in the hashrate summary.
 *
 * @typedef {Object} DifficultyEntry
 * @property {Timestamp} time - Unix timestamp of the difficulty adjustment
 * @property {Height} height - Block height of the adjustment
 * @property {number} difficulty - Difficulty value
 * @property {number} adjustment - Adjustment ratio (new/previous, e.g. 1.068 = +6.8%)
 */
/**
 * Disk usage of the indexed data
 *
 * @typedef {Object} DiskUsage
 * @property {string} brk - Human-readable brk data size (e.g., "48.8 GiB")
 * @property {number} brkBytes - brk data size in bytes
 * @property {string} bitcoin - Human-readable Bitcoin blocks directory size
 * @property {number} bitcoinBytes - Bitcoin blocks directory size in bytes
 * @property {number} ratio - Ratio of BRK bytes to Bitcoin bytes; zero when Bitcoin bytes are zero.
 */
/**
 * US Dollar amount
 *
 * @typedef {number} Dollars
 */
/**
 * Data of an empty address
 *
 * @typedef {Object} EmptyAddrData
 * @property {number} txCount - Total transaction count
 * @property {number} fundedTxoCount - Total funded/spent transaction output count (equal since address is empty)
 * @property {Sats} transfered - Total satoshis transferred
 */
/** @typedef {TypeIndex} EmptyOutputIndex */
/** @typedef {number} Epoch */
/**
 * The JSON body of every API error (`application/problem+json`).
 *
 * @typedef {Object} ErrorBody
 * @property {ErrorDetail} error
 */
/**
 * Machine-readable error code.
 *
 * @typedef {("not_found"|"invalid_addr"|"invalid_network"|"unsupported_type"|"parse_error"|"no_series"|"series_unsupported_index"|"weight_exceeded"|"too_many_utxos"|"unknown_addr"|"unknown_txid"|"out_of_range"|"unindexable_date"|"no_data"|"series_not_found"|"mempool_not_available"|"state_updating"|"internal_error"|"bad_request"|"overloaded"|"timeout"|"method_not_allowed")} ErrorCode
 */
/**
 * @typedef {Object} ErrorDetail
 * @property {ErrorType} type - Error category, following the HTTP status
 * @property {ErrorCode} code - Machine-readable error code
 * @property {string} message - Human-readable description
 * @property {string} docUrl - Link to API documentation
 */
/**
 * Error category, following the HTTP status: `invalid_request` (4xx other than 404),
 * `not_found` (404), `unavailable` (503; `Retry-After` when transient), `timeout` (504), `internal`
 * (other 5xx).
 *
 * @typedef {("invalid_request"|"not_found"|"unavailable"|"timeout"|"internal")} ErrorType
 */
/**
 * Exchange rates (USD base, on-chain only — no fiat pairs available)
 *
 * @typedef {Object} ExchangeRates
 */
/**
 * Fee rate stored in milli-sat/vB and exposed as sat/vB.
 *
 * @typedef {number} FeeRate
 */
/**
 * Output format for API responses
 *
 * @typedef {("json"|"csv")} Format
 */
/**
 * Data for a funded (non-empty) address with current balance.
 *
 * Kept compact because one value is stored for every funded address.
 *
 * @typedef {Object} FundedAddrData
 * @property {Sats} received - Satoshis received by this address
 * @property {Sats} sent - Satoshis sent by this address
 * @property {number} realizedCapRaw - The realized capitalization: Σ(price × sats)
 * @property {number} txCount - Total transaction count
 * @property {number} fundedTxoCount - Number of transaction outputs funded to this address
 * @property {number} spentTxoCount - Number of transaction outputs spent by this address
 */
/** @typedef {number} Halving */
/**
 * A single hashrate data point.
 *
 * @typedef {Object} HashrateEntry
 * @property {Timestamp} timestamp - Unix timestamp
 * @property {number} avgHashrate - Average hashrate (H/s)
 */
/**
 * Summary of network hashrate and difficulty data.
 *
 * @typedef {Object} HashrateSummary
 * @property {HashrateEntry[]} hashrates - Historical hashrate data points
 * @property {DifficultyEntry[]} difficulty - Historical difficulty adjustments
 * @property {number} currentHashrate - Current network hashrate (H/s)
 * @property {number} currentDifficulty - Current network difficulty
 */
/**
 * Server health status
 *
 * @typedef {Object} Health
 * @property {string} status - Health status ("healthy")
 * @property {string} service - Service name
 * @property {string} version - Server version
 * @property {string} timestamp - Current server time (ISO 8601)
 * @property {string} startedAt - Server start time (ISO 8601)
 * @property {number} uptimeSeconds - Uptime in seconds
 * @property {Height} indexedHeight - Height of the last indexed block
 * @property {Height} computedHeight - Height of the last computed block (series)
 * @property {Height} tipHeight - Height of the chain tip (from Bitcoin node)
 * @property {Height} blocksBehind - Number of blocks behind the tip
 * @property {string} lastIndexedAt - Human-readable timestamp of the last indexed block (ISO 8601)
 * @property {Timestamp} lastIndexedAtUnix - Unix timestamp of the last indexed block
 */
/**
 * Block height
 *
 * @typedef {number} Height
 */
/**
 * Path parameter accepting either a block height (`840000`) or a calendar date
 * (`YYYY-MM-DD`). The handler resolves it and dispatches to the per-height or
 * per-day variant, choosing the matching cache strategy.
 *
 * @typedef {Object} HeightOrDateParam
 * @property {string} point - Confirmed block height as decimal digits (`840000`) or calendar date in
`YYYY-MM-DD` format.
 */
/**
 * Block height path parameter
 *
 * @typedef {Object} HeightParam
 * @property {Height} height
 */
/**
 * Hex-encoded string. Transparent wrapper over `String`: serializes
 * as a plain JSON string and derefs to `str`, so anywhere `&str` or
 * `AsRef<[u8]>` is expected the `Hex` "just works".
 *
 * @typedef {string} Hex
 */
/**
 * Highest price value for a time period
 *
 * @typedef {Dollars} High
 */
/**
 * Historical price response
 *
 * @typedef {Object} HistoricalPrice
 * @property {HistoricalPriceEntry[]} prices - Price data points
 * @property {ExchangeRates} exchangeRates - Exchange rates (currently empty)
 */
/**
 * A single price data point
 *
 * @typedef {Object} HistoricalPriceEntry
 * @property {Timestamp} time - Unix timestamp
 * @property {Dollars} uSD - BTC/USD price
 */
/** @typedef {number} Hour1 */
/** @typedef {number} Hour12 */
/** @typedef {number} Hour4 */
/**
 * Aggregation dimension for querying series. Includes time-based (date, week, month, year),
 * block-based (height, tx_index), and address/output type indexes.
 *
 * @typedef {("minute10"|"minute30"|"hour1"|"hour4"|"hour12"|"day1"|"day3"|"week1"|"month1"|"month3"|"month6"|"year1"|"year10"|"halving"|"epoch"|"height"|"tx_index"|"txin_index"|"txout_index"|"empty_output_index"|"op_return_index"|"p2a_addr_index"|"p2ms_output_index"|"p2pk33_addr_index"|"p2pk65_addr_index"|"p2pkh_addr_index"|"p2sh_addr_index"|"p2tr_addr_index"|"p2wpkh_addr_index"|"p2wsh_addr_index"|"unknown_output_index"|"funded_addr_index"|"empty_addr_index"|"extended_empty_addr_index")} Index
 */
/**
 * Information about an available index and its query aliases
 *
 * @typedef {Object} IndexInfo
 * @property {Index} index - The canonical index name
 * @property {string[]} aliases - All Accepted query aliases
 */
/**
 * Maximum number of results to return. Defaults to 100 if not specified.
 *
 * @typedef {number} Limit
 */
/**
 * Lowest price value for a time period
 *
 * @typedef {Dollars} Low
 */
/**
 * Block info in a mempool.space like format for fee estimation.
 *
 * @typedef {Object} MempoolBlock
 * @property {number} blockSize - Total serialized block size in bytes (witness + non-witness).
 * @property {number} blockVSize - Total block virtual size in vbytes
 * @property {number} nTx - Number of transactions in the projected block
 * @property {Sats} totalFees - Total fees in satoshis
 * @property {FeeRate} medianFee - Median fee rate in sat/vB
 * @property {FeeRate[]} feeRange - Fee rate range: [min, 10%, 25%, 50%, 75%, 90%, max]
 */
/**
 * Mempool statistics with incrementally maintained fee histogram.
 *
 * @typedef {Object} MempoolInfo
 * @property {number} count - Number of transactions in the mempool
 * @property {VSize} vsize - Total virtual size of all transactions in the mempool (vbytes)
 * @property {Sats} totalFee - Total fees of all transactions in the mempool (satoshis)
 * @property {number[][]} feeHistogram - Fee histogram: `[[fee_rate, vsize], ...]` sorted by descending fee rate
 */
/**
 * Simplified mempool transaction for the `/api/mempool/recent` endpoint.
 *
 * @typedef {Object} MempoolRecentTx
 * @property {Txid} txid - Transaction ID
 * @property {Sats} fee - Transaction fee (sats)
 * @property {VSize} vsize - Virtual size (vbytes)
 * @property {Sats} value - Total output value (sats)
 */
/**
 * Merkle inclusion proof for a transaction
 *
 * @typedef {Object} MerkleProof
 * @property {Height} blockHeight - Block height containing the transaction
 * @property {string[]} merkle - Merkle proof path (hex-encoded hashes)
 * @property {number} pos - Transaction position in the block (0-indexed)
 */
/** @typedef {number} Minute10 */
/** @typedef {number} Minute30 */
/** @typedef {number} Month1 */
/** @typedef {number} Month3 */
/** @typedef {number} Month6 */
/**
 * Content hash of the projected next block (block 0 of the mempool
 * snapshot), including its statistics and complete transaction bodies.
 * Opaque token, distinct from HTTP ETag formatting: pass back
 * to `GET /api/v1/mempool/block-template/diff/{hash}` to fetch deltas.
 *
 * @typedef {number} NextBlockHash
 */
/**
 * Prior-template hash for `GET /api/v1/mempool/block-template/diff/{hash}`.
 *
 * @typedef {Object} NextBlockHashParam
 * @property {NextBlockHash} hash
 */
/**
 * OHLC (Open, High, Low, Close) data in cents
 *
 * @typedef {Object} OHLCCents
 * @property {Open} open
 * @property {High} high
 * @property {Low} low
 * @property {Close} close
 */
/**
 * OHLC (Open, High, Low, Close) data in dollars
 *
 * @typedef {Object} OHLCDollars
 * @property {Open} open
 * @property {High} high
 * @property {Low} low
 * @property {Close} close
 */
/**
 * OHLC (Open, High, Low, Close) data in satoshis
 *
 * @typedef {Object} OHLCSats
 * @property {Open} open
 * @property {High} high
 * @property {Low} low
 * @property {Close} close
 */
/** @typedef {TypeIndex} OpReturnIndex */
/** @typedef {("runes"|"veri_block"|"omni"|"stacks"|"blockstack"|"colu"|"open_assets"|"komodo"|"coin_spark"|"poet"|"docproof"|"open_timestamps"|"factom"|"eternity_wall"|"memo"|"bitproof"|"ascribe"|"stampery"|"epobc"|"bare_hash"|"text"|"empty"|"unknown")} OpReturnKind */
/**
 * Opening price value for a time period
 *
 * @typedef {Dollars} Open
 */
/**
 * Optional UNIX timestamp query parameter
 *
 * @typedef {Object} OptionalTimestampParam
 * @property {(Timestamp|null)=} timestamp
 */
/** @typedef {number} OutPoint */
/**
 * Type (P2PKH, P2WPKH, P2SH, P2TR, etc.)
 *
 * @typedef {("p2pk65"|"p2pk33"|"p2pkh"|"p2ms"|"p2sh"|"opreturn"|"p2wpkh"|"p2wsh"|"p2tr"|"p2a"|"empty"|"unknown")} OutputType
 */
/**
 * Output type names used by Esplora and mempool.space.
 *
 * @typedef {("p2pk"|"p2pkh"|"multisig"|"p2sh"|"op_return"|"v0_p2wpkh"|"v0_p2wsh"|"v1_p2tr"|"anchor"|"empty"|"unknown")} OutputTypeNormalized
 */
/** @typedef {TypeIndex} P2AAddrIndex */
/** @typedef {U8x2} P2ABytes */
/** @typedef {TypeIndex} P2MSOutputIndex */
/** @typedef {TypeIndex} P2PK33AddrIndex */
/** @typedef {U8x33} P2PK33Bytes */
/** @typedef {TypeIndex} P2PK65AddrIndex */
/** @typedef {U8x65} P2PK65Bytes */
/** @typedef {TypeIndex} P2PKHAddrIndex */
/** @typedef {U8x20} P2PKHBytes */
/** @typedef {TypeIndex} P2SHAddrIndex */
/** @typedef {U8x20} P2SHBytes */
/** @typedef {TypeIndex} P2TRAddrIndex */
/** @typedef {U8x32} P2TRBytes */
/** @typedef {TypeIndex} P2WPKHAddrIndex */
/** @typedef {U8x20} P2WPKHBytes */
/** @typedef {TypeIndex} P2WSHAddrIndex */
/** @typedef {U8x32} P2WSHBytes */
/**
 * A paginated list of available series names (1000 per page)
 *
 * @typedef {Object} PaginatedSeries
 * @property {number} currentPage - Current page number (0-indexed)
 * @property {number} maxPage - Maximum valid page index (0-indexed)
 * @property {number} totalCount - Total number of series
 * @property {number} perPage - Results per page
 * @property {boolean} hasMore - Whether more pages are available after the current one
 * @property {string[]} series - List of series names
 */
/**
 * Pagination parameters for paginated API endpoints
 *
 * @typedef {Object} Pagination
 * @property {?number=} page - Pagination index
 * @property {?number=} perPage - Results per page (default: 1000, max: 1000)
 */
/**
 * Unsigned parts per million stored as u32.
 * One unit is 0.000001. Range: 0–4,294.967294.
 * Use for precise bounded ratios and percentages.
 * `u32::MAX` is reserved as a NaN sentinel.
 *
 * @typedef {number} PartsPerMillion32
 */
/**
 * Unsigned parts per million stored as u64.
 * One unit is 0.000001. Range: 0–18,446,744,073,709.551614.
 * Use for precise wide-range ratios.
 * `u64::MAX` is reserved as a NaN sentinel.
 *
 * @typedef {number} PartsPerMillion64
 */
/**
 * Signed parts per million stored as i32.
 * One unit is 0.000001. Range: -2,147.483647 to +2,147.483647.
 * Use for precise bounded signed ratios and percentages.
 * `i32::MIN` is reserved as a NaN sentinel.
 *
 * @typedef {number} PartsPerMillionSigned32
 */
/**
 * Signed parts per million stored as i64.
 * One unit is 0.000001. Range: -9,223,372,036,854.775807 to +9,223,372,036,854.775807.
 * Use for precise wide-range signed ratios and percentages.
 * `i64::MIN` is reserved as a NaN sentinel.
 *
 * @typedef {number} PartsPerMillionSigned64
 */
/**
 * Block counts for different time periods
 *
 * @typedef {Object} PoolBlockCounts
 * @property {number} all - Total blocks mined (all time)
 * @property {number} _24h - Blocks mined in last 24 hours
 * @property {number} _1w - Blocks mined in last week
 */
/**
 * Pool's share of total blocks for different time periods
 *
 * @typedef {Object} PoolBlockShares
 * @property {number} all - Share of all blocks (0.0 - 1.0)
 * @property {number} _24h - Share of blocks in last 24 hours (0.0 - 1.0)
 * @property {number} _1w - Share of blocks in last week (0.0 - 1.0)
 */
/**
 * Detailed pool information with statistics across time periods
 *
 * @typedef {Object} PoolDetail
 * @property {PoolDetailInfo} pool - Pool information
 * @property {PoolBlockCounts} blockCount - Block counts for different time periods
 * @property {PoolBlockShares} blockShare - Pool's share of total blocks for different time periods
 * @property {number} estimatedHashrate - Estimated hashrate based on blocks mined (H/s)
 * @property {?number=} reportedHashrate - Self-reported hashrate (if available, H/s)
 * @property {(Sats|null)=} totalReward - Total reward earned by this pool (sats, all time; None for minor pools)
 */
/**
 * Pool information for detail view
 *
 * @typedef {Object} PoolDetailInfo
 * @property {number} id - Pool identifier
 * @property {string} name - Pool name
 * @property {string} link - Pool website URL
 * @property {string[]} addresses - Known payout addresses
 * @property {string[]} regexes - Coinbase tag patterns (regexes)
 * @property {PoolSlug} slug - URL-friendly pool identifier
 * @property {number} uniqueId - Unique pool identifier
 */
/**
 * A single pool hashrate data point.
 *
 * @typedef {Object} PoolHashrateEntry
 * @property {Timestamp} timestamp - Unix timestamp
 * @property {number} avgHashrate - Average hashrate (H/s)
 * @property {number} share - Pool's share of total network hashrate (0.0 - 1.0)
 * @property {string} poolName - Pool name
 */
/**
 * Basic pool information for listing all pools
 *
 * @typedef {Object} PoolInfo
 * @property {string} name - Pool name
 * @property {PoolSlug} slug - URL-friendly pool identifier
 * @property {number} uniqueId - Unique numeric pool identifier
 */
/**
 * URL-friendly mining pool identifier
 *
 * @typedef {("unknown"|"blockfills"|"ultimuspool"|"terrapool"|"luxor"|"1thash"|"btccom"|"bitfarms"|"huobipool"|"wayicn"|"canoepool"|"btctop"|"bitcoincom"|"175btc"|"gbminers"|"axbt"|"asicminer"|"bitminter"|"bitcoinrussia"|"btcserv"|"simplecoinus"|"btcguild"|"eligius"|"ozcoin"|"eclipsemc"|"maxbtc"|"triplemining"|"coinlab"|"50btc"|"ghashio"|"stminingcorp"|"bitparking"|"mmpool"|"polmine"|"kncminer"|"bitalo"|"f2pool"|"hhtt"|"megabigpower"|"mtred"|"nmcbit"|"yourbtcnet"|"givemecoins"|"braiinspool"|"antpool"|"multicoinco"|"bcpoolio"|"cointerra"|"kanopool"|"solock"|"ckpool"|"nicehash"|"bitclub"|"bitcoinaffiliatenetwork"|"btcc"|"bwpool"|"exxbw"|"bitsolo"|"bitfury"|"21inc"|"digitalbtc"|"8baochi"|"mybtccoinpool"|"tbdice"|"hashpool"|"nexious"|"bravomining"|"hotpool"|"okexpool"|"bcmonster"|"1hash"|"bixin"|"tatmaspool"|"viabtc"|"connectbtc"|"batpool"|"waterhole"|"dcexploration"|"dcex"|"btpool"|"58coin"|"bitcoinindia"|"shawnp0wers"|"phashio"|"rigpool"|"haozhuzhu"|"7pool"|"miningkings"|"hashbx"|"dpool"|"rawpool"|"haominer"|"helix"|"bitcoinukraine"|"poolin"|"secretsuperstar"|"tigerpoolnet"|"sigmapoolcom"|"okpooltop"|"hummerpool"|"tangpool"|"bytepool"|"spiderpool"|"novablock"|"miningcity"|"binancepool"|"minerium"|"lubiancom"|"okkong"|"aaopool"|"emcdpool"|"foundryusa"|"sbicrypto"|"arkpool"|"purebtccom"|"marapool"|"kucoinpool"|"entrustcharitypool"|"okminer"|"titan"|"pegapool"|"btcnuggets"|"cloudhashing"|"digitalxmintsy"|"telco214"|"btcpoolparty"|"multipool"|"transactioncoinmining"|"btcdig"|"trickysbtcpool"|"btcmp"|"eobot"|"unomp"|"patels"|"gogreenlight"|"bitcoinindiapool"|"ekanembtc"|"canoe"|"tiger"|"1m1x"|"zulupool"|"secpool"|"ocean"|"whitepool"|"wiz"|"wk057"|"futurebitapollosolo"|"carbonnegative"|"portlandhodl"|"phoenix"|"neopool"|"maxipool"|"bitfufupool"|"gdpool"|"miningdutch"|"publicpool"|"miningsquared"|"innopolistech"|"btclab"|"parasite"|"redrockpool"|"est3lar"|"braiinssolo"|"solopoolcom"|"noderunners"|"dmnd")} PoolSlug
 */
/**
 * Mining pool slug + block height path parameters
 *
 * @typedef {Object} PoolSlugAndHeightParam
 * @property {PoolSlug} slug
 * @property {Height} height
 */
/**
 * Mining pool slug path parameter
 *
 * @typedef {Object} PoolSlugParam
 * @property {PoolSlug} slug
 */
/**
 * Mining pool with block statistics for a time period
 *
 * @typedef {Object} PoolStats
 * @property {number} poolId - Unique pool identifier
 * @property {string} name - Pool name
 * @property {string} link - Pool website URL
 * @property {number} blockCount - Number of blocks mined in the time period
 * @property {number} rank - Pool ranking by block count (1 = most blocks)
 * @property {number} emptyBlocks - Number of empty blocks mined
 * @property {PoolSlug} slug - URL-friendly pool identifier
 * @property {number} share - Pool's share of total blocks (0.0 - 1.0)
 * @property {number} poolUniqueId - Unique pool identifier
 */
/**
 * Mining pools response for a time period
 *
 * @typedef {Object} PoolsSummary
 * @property {PoolStats[]} pools - List of pools sorted by block count descending
 * @property {number} blockCount - Total blocks in the time period
 * @property {number} lastEstimatedHashrate - Estimated network hashrate (H/s)
 * @property {number} lastEstimatedHashrate3d - Estimated network hashrate over last 3 days (H/s)
 * @property {number} lastEstimatedHashrate1w - Estimated network hashrate over last 1 week (H/s)
 */
/**
 * Spot price divided by a reference price, encoded in parts per million.
 * Finite values saturate at 4,294.967294; u32::MAX represents undefined.
 * Saturation is deliberately specific to price ratios, across all cohorts.
 * Non-finite inputs become undefined; finite inputs must be nonnegative
 * (a debug-checked precondition). PPM conversion rounds to nearest, matching
 * the existing price ratios.
 * Serde preserves raw encoded bits; vector JSON emits null for undefined.
 *
 * @typedef {number} PriceRatio
 */
/**
 * Current price response matching mempool.space /api/v1/prices format
 *
 * @typedef {Object} Prices
 * @property {Timestamp} time - Unix timestamp
 * @property {Dollars} uSD - BTC/USD price
 */
/**
 * A positional index, YYYY-MM-DD date, or ISO 8601 timestamp.
 *
 * @typedef {(number|string|string)} RangeIndex
 */
/**
 * Transaction locktime. Values below 500,000,000 are interpreted as block heights; values at or above are Unix timestamps.
 *
 * @typedef {number} RawLockTime
 */
/**
 * Response body for `GET /api/v1/tx/:txid/rbf`. Both fields are null
 * when the tx has no known RBF history within the mempool monitor's
 * graveyard retention window.
 *
 * @typedef {Object} RbfResponse
 * @property {(ReplacementNode|null)=} replacements
 * @property {?Txid[]=} replaces
 */
/**
 * Transaction summary carried inside an RBF replacement node. Shape
 * matches mempool.space's `/api/v1/tx/:txid/rbf` and
 * `/api/v1/replacements` responses.
 *
 * @typedef {Object} RbfTx
 * @property {Txid} txid
 * @property {Sats} fee
 * @property {VSize} vsize
 * @property {Sats} value - Sum of output amounts.
 * @property {FeeRate} rate
 * @property {Timestamp} time
 * @property {boolean} rbf - BIP-125 signaling: at least one input has sequence < 0xffffffff-1.
 * @property {?boolean=} fullRbf - Only populated on the root `tx` of an RBF response. `true` iff
this tx displaced at least one non-signaling predecessor.
 */
/**
 * Recommended fee rates in sat/vB
 *
 * @typedef {Object} RecommendedFees
 * @property {FeeRate} fastestFee - Fee rate for fastest confirmation (next block)
 * @property {FeeRate} halfHourFee - Fee rate for confirmation within ~30 minutes (3 blocks)
 * @property {FeeRate} hourFee - Fee rate for confirmation within ~1 hour (6 blocks)
 * @property {FeeRate} economyFee - Fee rate for economical confirmation
 * @property {FeeRate} minimumFee - Minimum relay fee rate
 */
/**
 * One node in an RBF replacement tree. The node's `tx` replaced each
 * entry in `replaces`, recursively.
 *
 * @typedef {Object} ReplacementNode
 * @property {RbfTx} tx
 * @property {Timestamp} time - First-seen timestamp, duplicated here to match mempool.space's
on-the-wire shape.
 * @property {boolean} fullRbf - Any predecessor in this subtree was non-signaling.
 * @property {?number=} interval - Seconds between this node's `time` and the successor that
replaced it. Omitted on the root of an RBF response.
 * @property {?boolean=} mined - `Some(true)` iff this node's tx is currently confirmed. Absent
on serialization otherwise.
 * @property {ReplacementNode[]} replaces
 */
/**
 * Block reward statistics over a range of blocks
 *
 * @typedef {Object} RewardStats
 * @property {Height} startBlock - First block in the range
 * @property {Height} endBlock - Last block in the range
 * @property {string} totalReward - Total coinbase rewards (subsidy + fees) in sats
 * @property {string} totalFee - Total transaction fees in sats
 * @property {string} totalTx - Total number of transactions
 */
/**
 * Amount in satoshis (1 BTC = 100,000,000 sats)
 *
 * @typedef {number} Sats
 */
/**
 * Fractional satoshis (f64) - for representing USD prices in sats
 *
 * Formula: `sats_fract = usd_value * 100_000_000 / btc_price`
 *
 * When BTC is $100,000:
 * - $1 = 1,000 sats
 * - $0.001 = 1 sat
 * - $0.0001 = 0.1 sats (fractional)
 *
 * @typedef {number} SatsFract
 */
/**
 * Signed satoshis (i64) - for values that can be negative.
 * Used for changes, deltas, profit/loss calculations, etc.
 *
 * @typedef {number} SatsSigned
 */
/**
 * @typedef {Object} SearchQuery
 * @property {SeriesName} q - Search query string
 * @property {Limit=} limit - Maximum number of results
 */
/**
 * Series count statistics
 *
 * @typedef {Object} SeriesCount
 * @property {number} distinct - Number of unique series available (e.g., realized_price, market_cap)
 * @property {number} total - Total number of series-index combinations across all timeframes
 * @property {number} lazy - Number of lazy (computed on-the-fly) series-index combinations
 * @property {number} stored - Number of eager (stored on disk) series-index combinations
 */
/**
 * Metadata about a series
 *
 * @typedef {Object} SeriesInfo
 * @property {?string=} description - Human-readable metric definition, when documented
 * @property {Index[]} indexes - Available indexes
 * @property {string} type - Value type (e.g. "f32", "u64", "Sats")
 */
/**
 * Series leaf with JSON Schema for client generation.
 *
 * @typedef {Object} SeriesLeafWithSchema
 * @property {string} name - The series name/identifier.
 * @property {string} kind - The Rust type (e.g., "Sats", "StoredF64").
 * @property {Index[]} indexes - Available indexes for this series.
 * @property {?string=} description - Human-readable metric definition, when documented.
 * @property {string} type - JSON Schema type (e.g., "integer", "number", "string", "boolean", "array", "object").
 */
/**
 * Comma-separated list of series names
 *
 * Deserialization permits at most 32 normalized names and 2,048 decoded input
 * string bytes. For arrays, the byte budget is shared by their string values.
 *
 * @typedef {string} SeriesList
 */
/**
 * Series name
 *
 * @typedef {string} SeriesName
 */
/**
 * @typedef {Object} SeriesNameWithIndex
 * @property {SeriesName} series - Series name
 * @property {Index} index - Aggregation index
 */
/**
 * @typedef {Object} SeriesParam
 * @property {SeriesName} series
 */
/**
 * Selection of series to query
 *
 * @typedef {Object} SeriesSelection
 * @property {SeriesList} series - Requested series
 * @property {Index} index - Index to query
 * @property {(RangeIndex|null)=} start - Inclusive start: integer index, date (YYYY-MM-DD), or timestamp (ISO 8601). Negative integers count from end. Aliases: `from`, `f`, `s`
 * @property {(RangeIndex|null)=} end - Exclusive end: integer index, date (YYYY-MM-DD), or timestamp (ISO 8601). Negative integers count from end. Aliases: `to`, `t`, `e`
 * @property {(Limit|null)=} limit - Maximum number of values to return (ignored if `end` is set). Aliases: `count`, `c`, `l`
 * @property {Format=} format - Format of the output
 */
/**
 * BIP-141 sigop cost. The block-level budget is 80,000, so a `u32`
 * fits a single tx's count with room to spare.
 *
 * Witness sigops count as 1; legacy and P2SH-redeem sigops count as 4.
 * Five vbytes per sigop is the policy adjustment Core applies in
 * `nSigOpCost` to discourage sigop-heavy txs (`max(weight/4, sigops*5)`).
 *
 * @typedef {number} SigOps
 */
/** @typedef {boolean} StoredBool */
/**
 * Stored 32-bit floating point value
 *
 * @typedef {number} StoredF32
 */
/**
 * Fixed-size 64-bit floating point value optimized for on-disk storage
 *
 * @typedef {number} StoredF64
 */
/**
 * Fixed-size 64-bit signed integer optimized for on-disk storage
 *
 * @typedef {number} StoredI64
 */
/** @typedef {number} StoredI8 */
/** @typedef {number} StoredU16 */
/**
 * Fixed-size 32-bit unsigned integer optimized for on-disk storage
 *
 * @typedef {number} StoredU32
 */
/**
 * Fixed-size 64-bit unsigned integer optimized for on-disk storage
 *
 * @typedef {number} StoredU64
 */
/** @typedef {number} StoredU8 */
/**
 * Sync status of the indexer
 *
 * @typedef {Object} SyncStatus
 * @property {Height} indexedHeight - Height of the last indexed block
 * @property {Height} computedHeight - Height of the last computed block (series)
 * @property {Height} tipHeight - Height of the chain tip (from Bitcoin node)
 * @property {Height} blocksBehind - Number of blocks behind the tip
 * @property {string} lastIndexedAt - Human-readable timestamp of the last indexed block (ISO 8601)
 * @property {Timestamp} lastIndexedAtUnix - Unix timestamp of the last indexed block
 */
/**
 * Time period for mining statistics.
 *
 * Used to specify the lookback window for pool statistics, hashrate calculations,
 * and other time-based mining series.
 *
 * @typedef {("24h"|"3d"|"1w"|"1m"|"3m"|"6m"|"1y"|"2y"|"3y"|"all")} TimePeriod
 */
/**
 * Time period path parameter (24h, 3d, 1w, 1m, 3m, 6m, 1y, 2y, 3y)
 *
 * @typedef {Object} TimePeriodParam
 * @property {TimePeriod} timePeriod
 */
/**
 * UNIX timestamp in seconds
 *
 * @typedef {number} Timestamp
 */
/**
 * UNIX timestamp path parameter
 *
 * @typedef {Object} TimestampParam
 * @property {Timestamp} timestamp
 */
/**
 * Transaction information compatible with mempool.space API format
 *
 * @typedef {Object} Transaction
 * @property {(TxIndex|null)=} index - Internal transaction index (brk-specific, not in mempool.space)
 * @property {Txid} txid - Transaction ID
 * @property {TxVersionRaw} version - Transaction version (raw i32 from Bitcoin protocol, may contain non-standard values in coinbase txs)
 * @property {RawLockTime} locktime - Transaction lock time
 * @property {TxIn[]} vin - Transaction inputs
 * @property {TxOut[]} vout - Transaction outputs
 * @property {number} size - Transaction size in bytes
 * @property {Weight} weight - Transaction weight
 * @property {SigOps} sigops - Number of signature operations
 * @property {Sats} fee - Transaction fee in satoshis
 * @property {TxStatus} status - Confirmation status (confirmed, block height/hash/time)
 */
/**
 * Hierarchical tree node for organizing series into categories
 *
 * @typedef {({ [key: string]: TreeNode }|SeriesLeafWithSchema)} TreeNode
 */
/**
 * Transaction input
 *
 * @typedef {Object} TxIn
 * @property {Txid} txid - Transaction ID of the output being spent
 * @property {Vout} vout - Output index being spent (u16: coinbase is 65535, mempool.space uses u32: 4294967295)
 * @property {(TxOut|null)} prevout - Information about the previous output being spent
 * @property {string} scriptsig - Signature script (hex, for non-SegWit inputs)
 * @property {string} scriptsigAsm - Signature script in assembly format
 * @property {Witness=} witness - Witness data (stack items, present for SegWit inputs; hex-encoded on the wire)
 * @property {boolean} isCoinbase - Whether this input is a coinbase (block reward) input
 * @property {number} sequence - Input sequence number
 * @property {string=} innerRedeemscriptAsm - Inner redeemscript in assembly (for P2SH-wrapped SegWit: scriptsig + witness both present)
 * @property {string=} innerWitnessscriptAsm - Inner witnessscript in assembly (for P2WSH: last witness item decoded as script)
 */
/** @typedef {number} TxInIndex */
/**
 * Chain-wide transaction index (0 = the genesis coinbase). For an
 * in-block position, use `BlockTxIndex` instead.
 *
 * @typedef {number} TxIndex
 */
/**
 * Transaction index path parameter
 *
 * @typedef {Object} TxIndexParam
 * @property {TxIndex} index
 */
/**
 * @typedef {Object} TxOut
 * @property {string} scriptpubkey - Script pubkey (locking script), encoded as hexadecimal.
 * @property {string} scriptpubkeyAsm - Script pubkey in assembly format.
 * @property {OutputTypeNormalized} scriptpubkeyType - Esplora/mempool.space script type.
 * @property {Addr=} scriptpubkeyAddress - Bitcoin address, omitted for scripts without an address.
 * @property {Sats} value - Value of the output in satoshis.
 */
/** @typedef {number} TxOutIndex */
/**
 * Status of an output indicating whether it has been spent
 *
 * @typedef {Object} TxOutspend
 * @property {boolean} spent - Whether the output has been spent
 * @property {(Txid|null)=} txid - Transaction ID of the spending transaction (only present if spent)
 * @property {(Vin|null)=} vin - Input index in the spending transaction (only present if spent)
 * @property {(TxStatus|null)=} status - Status of the spending transaction (only present if spent)
 */
/**
 * Transaction confirmation status
 *
 * @typedef {Object} TxStatus
 * @property {boolean} confirmed - Whether the transaction is confirmed
 * @property {(Height|null)=} blockHeight - Block height (only present if confirmed)
 * @property {(BlockHash|null)=} blockHash - Block hash (only present if confirmed)
 * @property {(Timestamp|null)=} blockTime - Block timestamp (only present if confirmed)
 */
/**
 * Compact indexed transaction-version category. Values 1, 2, and 3 preserve
 * those exact signed 32-bit Bitcoin transaction versions; 255 represents every
 * other version.
 *
 * @typedef {number} TxVersion
 */
/**
 * Raw transaction version (i32) from Bitcoin protocol.
 * Unlike TxVersion (u8, indexed), this preserves non-standard values
 * used in coinbase txs for miner signaling/branding.
 *
 * @typedef {number} TxVersionRaw
 */
/**
 * Transaction ID (hash)
 *
 * @typedef {string} Txid
 */
/**
 * Transaction ID path parameter
 *
 * @typedef {Object} TxidParam
 * @property {Txid} txid
 */
/**
 * Transaction output reference (txid + output index)
 *
 * @typedef {Object} TxidVout
 * @property {Txid} txid - Transaction ID
 * @property {Vout} vout - Output index
 */
/**
 * Query parameter for transaction-times endpoint.
 *
 * Extracted manually because `serde_urlencoded` (and serde derive in general)
 * doesn't support repeated keys like `txId[]=a&txId[]=b`. The schema is still
 * declared via `JsonSchema` so the OpenAPI spec lists the parameter and the
 * generated client SDKs see `txids: List[Txid]`.
 *
 * @typedef {Object} TxidsParam
 * @property {Txid[]} txId - Transaction IDs to look up (max 250 per request).
 */
/**
 * Index within its type (e.g., 0 for first P2WPKH address)
 *
 * @typedef {number} TypeIndex
 */
/** @typedef {number[]} U8x2 */
/** @typedef {number[]} U8x20 */
/** @typedef {number[]} U8x32 */
/** @typedef {number[]} U8x33 */
/** @typedef {number[]} U8x65 */
/** @typedef {TypeIndex} UnknownOutputIndex */
/**
 * UTXO Realized Price Distribution for a cohort at a specific block.
 *
 * Supply is grouped by the price at the block in which each UTXO was last moved.
 * Each bucket exposes three values: supply in BTC, realized cap contribution
 * in USD (sum of `realized_price * supply` over the coins in the bucket), and
 * unrealized P&L in USD (`close * supply - realized_cap`, can be negative).
 *
 * @typedef {Object} Urpd
 * @property {Cohort} cohort
 * @property {Date} date - UTC date of the represented block.
 * @property {Height} height - Exact published block represented by this distribution.
 * @property {UrpdWeight} weight - Weighting applied to the source supply.
 * @property {UrpdAggregation} aggregation - Aggregation strategy applied to the buckets.
 * @property {Dollars} close - Price at `height`, in USD. Anchor for `unrealized_pnl`.
 * @property {Bitcoin} totalSupply - Sum of `supply` across all buckets, in BTC.
 * @property {UrpdBucket[]} buckets
 */
/**
 * Aggregation strategy for URPD buckets.
 * Options: raw (no aggregation), lin200/lin500/lin1000 (linear $200/$500/$1000),
 * log10/log50/log100/log200/log500/log1000/log2000 (logarithmic with 10/50/100/200/500/1000/2000 buckets per decade).
 *
 * @typedef {("raw"|"lin200"|"lin500"|"lin1000"|"log10"|"log50"|"log100"|"log200"|"log500"|"log1000"|"log2000")} UrpdAggregation
 */
/**
 * A single bucket in a URPD snapshot.
 *
 * @typedef {Object} UrpdBucket
 * @property {Dollars} priceFloor - Lower bound of the bucket, in USD. Equals the exact realized price for `Raw`.
 * @property {Bitcoin} supply - Supply held with a last-move price inside this bucket, in BTC.
 * @property {Dollars} realizedCap - Realized cap contribution in USD: sum of `realized_price * supply` over the coins in this bucket.
 * @property {Dollars} unrealizedPnl - Unrealized P&L in USD against the close on the snapshot date: `close * supply - realized_cap`. Can be negative.
 */
/**
 * Path parameters for per-cohort URPD endpoints.
 *
 * @typedef {Object} UrpdCohortParam
 * @property {Cohort} cohort
 */
/**
 * A URPD cohort and exact block height or UTC calendar-day alias.
 *
 * @typedef {Object} UrpdParams
 * @property {Cohort} cohort
 * @property {string} point
 */
/**
 * Query parameters for URPD endpoints.
 *
 * @typedef {Object} UrpdQuery
 * @property {UrpdAggregation=} agg - Aggregation strategy. Default: raw (no aggregation). Accepts `bucket` as alias.
 * @property {UrpdWeight=} weight - Supply weighting. Default: raw (unweighted).
 */
/**
 * Weighting applied to a URPD: raw (unweighted), cointime, or coinflow.
 *
 * @typedef {("raw"|"cointime"|"coinflow")} UrpdWeight
 */
/**
 * Query parameters for URPD date discovery.
 *
 * @typedef {Object} UrpdWeightQuery
 * @property {UrpdWeight=} weight - Supply weighting. Default: raw (unweighted).
 */
/**
 * Unspent transaction output
 *
 * @typedef {Object} Utxo
 * @property {Txid} txid - Transaction ID of the UTXO
 * @property {Vout} vout - Output index
 * @property {TxStatus} status - Confirmation status
 * @property {Sats} value - Output value in satoshis
 */
/**
 * Virtual size in vbytes (weight / 4, rounded up). Max block vsize is ~1,000,000 vB.
 *
 * @typedef {number} VSize
 */
/**
 * @typedef {Object} ValidateAddrParam
 * @property {string} address - Bitcoin address to validate (can be any string)
 */
/**
 * Version tracking for data schema and computed values.
 *
 * Used to detect when stored data needs to be recomputed due to changes
 * in computation logic or source data versions. Supports validation
 * against persisted versions to ensure compatibility.
 *
 * @typedef {number} Version
 */
/**
 * Input index in the spending transaction
 *
 * @typedef {number} Vin
 */
/**
 * Index of the output being spent in the previous transaction
 *
 * @typedef {number} Vout
 */
/** @typedef {number} Week1 */
/**
 * Weight in weight units (WU). Max block weight is 4,000,000 WU.
 *
 * @typedef {number} Weight
 */
/**
 * Weight in weight units with enough range for cumulative and rolling totals.
 *
 * @typedef {number} Weight64
 */
/**
 * Transaction witness: a stack of byte arrays, one per witness item.
 *
 * Wraps `bitcoin::Witness` (single-buffer layout with offsets, much
 * more compact than `Vec<Vec<u8>>`). Serializes as a JSON array of
 * hex strings - the format used by Bitcoin Core REST and mempool.space
 * and matching brk's `script_sig: ScriptBuf` (bytes internally, hex
 * on the wire).
 *
 * @typedef {string[]} Witness
 */
/** @typedef {number} Year1 */
/** @typedef {number} Year10 */

/**
 * @typedef {Object} BitviewClientOptions
 * @property {string} baseUrl - Base URL for the API
 * @property {number} [timeout] - Request timeout in milliseconds
 * @property {string|boolean} [browserCache] - Enable browser Cache API with default name (true), custom name (string), or disable (false). No effect in Node.js. Default: true
 * @property {number|boolean} [memCache] - In-memory parsed-response cache size (LRU). true/undefined → 1000, false/0 → disabled. Lets 304 responses skip the JSON parse entirely. Default: 1000
 */

const _isBrowser = typeof window !== 'undefined' && 'caches' in window;
const _runIdle = (/** @type {VoidFunction} */ fn) => (globalThis.requestIdleCallback ?? setTimeout)(fn);
const _defaultBrowserCacheName = '__BRK_CLIENT__';
const _DEFAULT_MEM_CACHE_SIZE = 1000;

/** @template T @typedef {{ etag: string | null, value: T }} _MemEntry */
/** @param {*} v */
const _addCamelGetters = (v) => {
  if (Array.isArray(v)) { v.forEach(_addCamelGetters); return v; }
  if (v && typeof v === 'object' && v.constructor === Object) {
    for (const k in v) {
      if (k.includes('_')) {
        const c = k.replace(/_([a-z])/g, (_, l) => l.toUpperCase());
        if (!(c in v)) Object.defineProperty(v, c, { get() { return this[k]; } });
      }
      _addCamelGetters(v[k]);
    }
  }
  return v;
};

/**
 * @param {string|boolean|undefined} option
 * @returns {Promise<Cache | null>}
 */
const _openBrowserCache = (option) => {
  if (!_isBrowser || option === false) return Promise.resolve(null);
  const name = typeof option === 'string' ? option : _defaultBrowserCacheName;
  return caches.open(name).catch(() => null);
};

/**
 * @param {string} url
 * @returns {URL}
 */
const _parseBaseUrl = (url) => new URL(url, typeof location === 'undefined' ? undefined : location.href);

/**
 * Custom error class for Bitview client errors
 */
class BitviewError extends Error {
  /**
   * @param {string} message
   * @param {number} [status]
   */
  constructor(message, status) {
    super(message);
    this.name = 'BitviewError';
    this.status = status;
  }
}

// Date conversion constants and helpers, mirroring the server's indexes: UTC, from 2009-01-01
// (week1 buckets ISO weeks, which start three days earlier; year10 buckets calendar decades, 2009 alone in the first).
const _MS_PER_DAY = 86400000;
const _MS_PER_WEEK = 7 * _MS_PER_DAY;
const _EPOCH_MS = 1230768000000;
/** @typedef {'minute10'|'minute30'|'hour1'|'hour4'|'hour12'|'day1'|'day3'|'week1'|'month1'|'month3'|'month6'|'year1'|'year10'} DateIndex */
const _DATE_INDEXES = new Set([
  'minute10', 'minute30',
  'hour1', 'hour4', 'hour12',
  'day1', 'day3', 'week1',
  'month1', 'month3', 'month6',
  'year1', 'year10',
]);

/** @param {number} months @returns {globalThis.Date} */
const _addMonths = (months) => new Date(Date.UTC(2009, months, 1));

/**
 * Convert an index value to a Date for date-based indexes.
 * @param {Index} index - The index type
 * @param {number} i - The index value
 * @returns {globalThis.Date}
 */
function indexToDate(index, i) {
  switch (index) {
    case 'minute10': return new Date(_EPOCH_MS + i * 600000);
    case 'minute30': return new Date(_EPOCH_MS + i * 1800000);
    case 'hour1': return new Date(_EPOCH_MS + i * 3600000);
    case 'hour4': return new Date(_EPOCH_MS + i * 14400000);
    case 'hour12': return new Date(_EPOCH_MS + i * 43200000);
    case 'day1': return new Date(_EPOCH_MS + i * _MS_PER_DAY);
    case 'day3': return new Date(_EPOCH_MS - 86400000 + i * 259200000);
    case 'week1': return new Date(_EPOCH_MS + i * _MS_PER_WEEK);
    case 'month1': return _addMonths(i);
    case 'month3': return _addMonths(i * 3);
    case 'month6': return _addMonths(i * 6);
    case 'year1': return new Date(Date.UTC(2009 + i, 0, 1));
    case 'year10': return new Date(Date.UTC(i === 0 ? 2009 : 2000 + i * 10, 0, 1));
    default: throw new Error(`${index} is not a date-based index`);
  }
}

/**
 * Convert a Date to an index value for date-based indexes.
 * Returns the floor index (latest index whose date is <= the given date).
 * @param {Index} index - The index type
 * @param {globalThis.Date} d - The date to convert
 * @returns {number}
 */
function dateToIndex(index, d) {
  const ms = d.getTime();
  switch (index) {
    case 'minute10': return Math.floor((ms - _EPOCH_MS) / 600000);
    case 'minute30': return Math.floor((ms - _EPOCH_MS) / 1800000);
    case 'hour1': return Math.floor((ms - _EPOCH_MS) / 3600000);
    case 'hour4': return Math.floor((ms - _EPOCH_MS) / 14400000);
    case 'hour12': return Math.floor((ms - _EPOCH_MS) / 43200000);
    case 'day1': return Math.max(0, Math.floor((ms - _EPOCH_MS) / _MS_PER_DAY));
    case 'day3': return Math.floor((ms - _EPOCH_MS + 86400000) / 259200000);
    case 'week1': return Math.max(0, Math.floor((ms - _EPOCH_MS + 3 * _MS_PER_DAY) / _MS_PER_WEEK));
    case 'month1': return (d.getUTCFullYear() - 2009) * 12 + d.getUTCMonth();
    case 'month3': return (d.getUTCFullYear() - 2009) * 4 + Math.floor(d.getUTCMonth() / 3);
    case 'month6': return (d.getUTCFullYear() - 2009) * 2 + Math.floor(d.getUTCMonth() / 6);
    case 'year1': return d.getUTCFullYear() - 2009;
    case 'year10': return Math.max(0, Math.floor((d.getUTCFullYear() - 2000) / 10));
    default: throw new Error(`${index} is not a date-based index`);
  }
}

/**
 * Wrap raw series data with helper methods.
 * @template T
 * @param {SeriesData<T>} raw - Raw JSON response
 * @returns {DateSeriesData<T>}
 */
function _wrapSeriesData(raw) {
  const { index, start, end, data } = raw;
  const _dateBased = _DATE_INDEXES.has(index);
  return /** @type {DateSeriesData<T>} */ ({
    ...raw,
    isDateBased: _dateBased,
    indexes() {
      /** @type {number[]} */
      const result = new Array(end - start);
      for (let i = 0; i < result.length; i++) result[i] = start + i;
      return result;
    },
    keys() {
      return this.indexes();
    },
    entries() {
      /** @type {Array<[number, T]>} */
      const result = new Array(data.length);
      for (let i = 0; i < data.length; i++) result[i] = [start + i, data[i]];
      return result;
    },
    toMap() {
      /** @type {Map<number, T>} */
      const map = new Map();
      for (let i = 0; i < data.length; i++) map.set(start + i, data[i]);
      return map;
    },
    *[Symbol.iterator]() {
      for (let i = 0; i < data.length; i++) yield /** @type {[number, T]} */ ([start + i, data[i]]);
    },
    // DateSeriesData methods (only meaningful for date-based indexes)
    dates() {
      /** @type {globalThis.Date[]} */
      const result = [];
      for (let i = start; i < end; i++) result.push(indexToDate(index, i));
      return result;
    },
    dateEntries() {
      /** @type {Array<[globalThis.Date, T]>} */
      const result = [];
      for (let i = 0; i < data.length; i++) result.push([indexToDate(index, start + i), data[i]]);
      return result;
    },
    toDateMap() {
      /** @type {Map<globalThis.Date, T>} */
      const map = new Map();
      for (let i = 0; i < data.length; i++) map.set(indexToDate(index, start + i), data[i]);
      return map;
    },
  });
}

/**
 * @template T
 * @typedef {Object} SeriesDataBase
 * @property {number} version - Version of the series data
 * @property {Index} index - The index type used for this query
 * @property {string} type - Value type (e.g. "f32", "u64", "Sats")
 * @property {number} start - Start index (inclusive)
 * @property {number} end - End index (exclusive)
 * @property {string} stamp - ISO 8601 timestamp of when the response was generated
 * @property {T[]} data - The series data
 * @property {boolean} isDateBased - Whether this series uses a date-based index
 * @property {() => number[]} indexes - Get index numbers
 * @property {() => number[]} keys - Get keys as index numbers (alias for indexes)
 * @property {() => Array<[number, T]>} entries - Get [index, value] pairs
 * @property {() => Map<number, T>} toMap - Convert to Map<index, value>
 */

/** @template T @typedef {SeriesDataBase<T> & Iterable<[number, T]>} SeriesData */

/**
 * @template T
 * @typedef {Object} DateSeriesDataExtras
 * @property {() => globalThis.Date[]} dates - Get dates for each data point
 * @property {() => Array<[globalThis.Date, T]>} dateEntries - Get [date, value] pairs
 * @property {() => Map<globalThis.Date, T>} toDateMap - Convert to Map<date, value>
 */

/** @template T @typedef {SeriesData<T> & DateSeriesDataExtras<T>} DateSeriesData */
/** @typedef {SeriesData<any>} AnySeriesData */

/** @template T @typedef {(onfulfilled?: (value: SeriesData<T>) => any, onrejected?: (reason: Error) => never) => Promise<SeriesData<T>>} Thenable */
/** @template T @typedef {(onfulfilled?: (value: DateSeriesData<T>) => any, onrejected?: (reason: Error) => never) => Promise<DateSeriesData<T>>} DateThenable */

/**
 * @template T
 * @typedef {Object} SeriesEndpoint
 * @property {(index: number) => SingleItemBuilder<T>} get - Get single item at index
 * @property {(start?: number, end?: number) => RangeBuilder<T>} slice - Slice by index
 * @property {(n: number) => RangeBuilder<T>} first - Get first n items
 * @property {(n: number) => RangeBuilder<T>} last - Get last n items
 * @property {(n: number) => SkippedBuilder<T>} skip - Skip first n items, chain with take()
 * @property {(arg?: SeriesFetchArg<T>, options?: ClientFetchOptions<SeriesData<T>>) => Promise<SeriesData<T>>} fetch - Fetch all data
 * @property {(options?: ClientFetchOptions<string>) => Promise<string>} fetchCsv - Fetch all data as CSV
 * @property {() => Promise<number>} len - Get total number of data points
 * @property {() => Promise<Version>} version - Get the current version of the series
 * @property {Thenable<T>} then - Thenable (await endpoint)
 * @property {string} path - The endpoint path
 */

/**
 * @template T
 * @typedef {Object} DateSeriesEndpoint
 * @property {(index: number | globalThis.Date) => DateSingleItemBuilder<T>} get - Get single item at index or Date
 * @property {(start?: number | globalThis.Date, end?: number | globalThis.Date) => DateRangeBuilder<T>} slice - Slice by index or Date
 * @property {(n: number) => DateRangeBuilder<T>} first - Get first n items
 * @property {(n: number) => DateRangeBuilder<T>} last - Get last n items
 * @property {(n: number) => DateSkippedBuilder<T>} skip - Skip first n items, chain with take()
 * @property {(arg?: DateSeriesFetchArg<T>, options?: ClientFetchOptions<DateSeriesData<T>>) => Promise<DateSeriesData<T>>} fetch - Fetch all data
 * @property {(options?: ClientFetchOptions<string>) => Promise<string>} fetchCsv - Fetch all data as CSV
 * @property {() => Promise<number>} len - Get total number of data points
 * @property {() => Promise<Version>} version - Get the current version of the series
 * @property {DateThenable<T>} then - Thenable (await endpoint)
 * @property {string} path - The endpoint path
 */

/** @typedef {SeriesEndpoint<any>} AnySeriesEndpoint */

/**
 * @template T
 * @typedef {Object} ClientFetchOptions
 * @property {AbortSignal} [signal] - Abort this request
 * @property {boolean} [cache] - Use HTTP/browser/client caches. Set false for a no-store network fetch.
 * @property {boolean} [memCache] - Use the parsed in-memory response cache. Set false for large one-shot reads.
 * @property {(value: T) => void} [onValue] - Receive stale/fresh values as they arrive
 */

/** @template T @typedef {ClientFetchOptions<SeriesData<T>> | ((value: SeriesData<T>) => void)} SeriesFetchArg */
/** @template T @typedef {ClientFetchOptions<DateSeriesData<T>> | ((value: DateSeriesData<T>) => void)} DateSeriesFetchArg */

/** @template T @typedef {Object} SingleItemBuilder
 * @property {(arg?: SeriesFetchArg<T>, options?: ClientFetchOptions<SeriesData<T>>) => Promise<SeriesData<T>>} fetch - Fetch the item
 * @property {(options?: ClientFetchOptions<string>) => Promise<string>} fetchCsv - Fetch as CSV
 * @property {Thenable<T>} then - Thenable
 */

/** @template T @typedef {Object} DateSingleItemBuilder
 * @property {(arg?: DateSeriesFetchArg<T>, options?: ClientFetchOptions<DateSeriesData<T>>) => Promise<DateSeriesData<T>>} fetch - Fetch the item
 * @property {(options?: ClientFetchOptions<string>) => Promise<string>} fetchCsv - Fetch as CSV
 * @property {DateThenable<T>} then - Thenable
 */

/** @template T @typedef {Object} SkippedBuilder
 * @property {(n: number) => RangeBuilder<T>} take - Take n items after skipped position
 * @property {(arg?: SeriesFetchArg<T>, options?: ClientFetchOptions<SeriesData<T>>) => Promise<SeriesData<T>>} fetch - Fetch from skipped position to end
 * @property {(options?: ClientFetchOptions<string>) => Promise<string>} fetchCsv - Fetch as CSV
 * @property {Thenable<T>} then - Thenable
 */

/** @template T @typedef {Object} DateSkippedBuilder
 * @property {(n: number) => DateRangeBuilder<T>} take - Take n items after skipped position
 * @property {(arg?: DateSeriesFetchArg<T>, options?: ClientFetchOptions<DateSeriesData<T>>) => Promise<DateSeriesData<T>>} fetch - Fetch from skipped position to end
 * @property {(options?: ClientFetchOptions<string>) => Promise<string>} fetchCsv - Fetch as CSV
 * @property {DateThenable<T>} then - Thenable
 */

/** @template T @typedef {Object} RangeBuilder
 * @property {(arg?: SeriesFetchArg<T>, options?: ClientFetchOptions<SeriesData<T>>) => Promise<SeriesData<T>>} fetch - Fetch the range
 * @property {(options?: ClientFetchOptions<string>) => Promise<string>} fetchCsv - Fetch as CSV
 * @property {Thenable<T>} then - Thenable
 */

/** @template T @typedef {Object} DateRangeBuilder
 * @property {(arg?: DateSeriesFetchArg<T>, options?: ClientFetchOptions<DateSeriesData<T>>) => Promise<DateSeriesData<T>>} fetch - Fetch the range
 * @property {(options?: ClientFetchOptions<string>) => Promise<string>} fetchCsv - Fetch as CSV
 * @property {DateThenable<T>} then - Thenable
 */

/**
 * @template T
 * @typedef {Object} SeriesPattern
 * @property {string} name - The series name
 * @property {Readonly<Partial<Record<Index, SeriesEndpoint<T>>>>} by - Index endpoints as lazy getters
 * @property {() => readonly Index[]} indexes - Get the list of available indexes
 * @property {(index: Index) => SeriesEndpoint<T>|undefined} get - Get an endpoint for a specific index
 */

/** @typedef {SeriesPattern<any>} AnySeriesPattern */

/**
 * Create a series endpoint builder with typestate pattern.
 * @template T
 * @param {BitviewClient} client
 * @param {string} name - The series vec name
 * @param {Index} index - The index name
 * @returns {DateSeriesEndpoint<T>}
 */
function _endpoint(client, name, index) {
  const p = `/api/series/${name}/${index}`;

  /**
   * @param {number} [start]
   * @param {number} [end]
   * @param {string} [format]
   * @returns {string}
   */
  const buildPath = (start, end, format) => {
    const params = new URLSearchParams();
    if (start !== undefined) params.set('start', String(start));
    if (end !== undefined) params.set('end', String(end));
    if (format) params.set('format', format);
    const query = params.toString();
    return query ? `${p}?${query}` : p;
  };

  /**
   * @param {number} [start]
   * @param {number} [end]
   * @returns {DateRangeBuilder<T>}
   */
  const rangeBuilder = (start, end) => ({
    fetch(arg, options) { return client._fetchSeriesData(buildPath(start, end), arg, options); },
    fetchCsv(options) { return client.getText(buildPath(start, end, 'csv'), options); },
    then(resolve, reject) { return this.fetch().then(resolve, reject); },
  });

  /**
   * @param {number} idx
   * @returns {DateSingleItemBuilder<T>}
   */
  const singleItemBuilder = (idx) => ({
    fetch(arg, options) { return client._fetchSeriesData(buildPath(idx, idx + 1), arg, options); },
    fetchCsv(options) { return client.getText(buildPath(idx, idx + 1, 'csv'), options); },
    then(resolve, reject) { return this.fetch().then(resolve, reject); },
  });

  /**
   * @param {number} start
   * @returns {DateSkippedBuilder<T>}
   */
  const skippedBuilder = (start) => ({
    take(n) { return rangeBuilder(start, start + n); },
    fetch(arg, options) { return client._fetchSeriesData(buildPath(start, undefined), arg, options); },
    fetchCsv(options) { return client.getText(buildPath(start, undefined, 'csv'), options); },
    then(resolve, reject) { return this.fetch().then(resolve, reject); },
  });

  /** @type {DateSeriesEndpoint<T>} */
  const endpoint = {
    get(idx) { if (idx instanceof Date) idx = dateToIndex(index, idx); return singleItemBuilder(idx); },
    slice(start, end) {
      if (start instanceof Date) start = dateToIndex(index, start);
      if (end instanceof Date) end = dateToIndex(index, end);
      return rangeBuilder(start, end);
    },
    first(n) { return rangeBuilder(undefined, n); },
    last(n) { return n === 0 ? rangeBuilder(undefined, 0) : rangeBuilder(-n, undefined); },
    skip(n) { return skippedBuilder(n); },
    fetch(arg, options) { return client._fetchSeriesData(buildPath(), arg, options); },
    fetchCsv(options) { return client.getText(buildPath(undefined, undefined, 'csv'), options); },
    len() { return client.getSeriesLen(name, index); },
    version() { return client.getSeriesVersion(name, index); },
    then(resolve, reject) { return this.fetch().then(resolve, reject); },
    get path() { return p; },
  };

  return endpoint;
}

/**
 * Base HTTP client for making requests with caching support
 */
class BitviewClientBase {
  /**
   * @param {BitviewClientOptions|string} options
   */
  constructor(options) {
    const isString = typeof options === 'string';
    const rawUrl = isString ? options : options.baseUrl;
    this.baseUrl = rawUrl.endsWith('/') ? rawUrl.slice(0, -1) : rawUrl;
    const url = _parseBaseUrl(this.baseUrl);
    this.url = url.href.endsWith('/') ? url.href.slice(0, -1) : url.href;
    this.domain = url.hostname;
    this.timeout = isString ? 5000 : (options.timeout ?? 5000);
    /** @type {Promise<Cache | null>} */
    this._browserCachePromise = _openBrowserCache(isString ? undefined : options.browserCache);
    /** @type {Cache | null} */
    this._browserCache = null;
    this._browserCachePromise.then(c => this._browserCache = c);
    const memOpt = isString ? undefined : options.memCache;
    this._memCacheMax = memOpt === false || memOpt === 0
      ? 0
      : (typeof memOpt === 'number' ? memOpt : _DEFAULT_MEM_CACHE_SIZE);
    /** @type {Map<string, _MemEntry<unknown>>} */
    this._memCache = new Map();
  }

  /**
   * @template T
   * @param {string} key
   * @returns {_MemEntry<T> | undefined}
   */
  _memGet(key) {
    if (!this._memCacheMax) return undefined;
    const hit = this._memCache.get(key);
    if (!hit) return undefined;
    this._memCache.delete(key);
    this._memCache.set(key, hit);
    return /** @type {_MemEntry<T>} */ (hit);
  }

  /**
   * @param {string} key
   * @param {string | null} etag
   * @param {unknown} value
   */
  _memSet(key, etag, value) {
    if (!this._memCacheMax) return;
    if (this._memCache.has(key)) this._memCache.delete(key);
    else if (this._memCache.size >= this._memCacheMax) {
      const oldest = this._memCache.keys().next().value;
      if (oldest !== undefined) this._memCache.delete(oldest);
    }
    this._memCache.set(key, { etag, value });
  }

  /**
   * @param {string} path
   * @param {{ signal?: AbortSignal, cache?: boolean, etag?: string | null }} [options]
   * @returns {Promise<Response>}
   */
  async get(path, { signal, cache = true, etag } = {}) {
    const url = `${this.baseUrl}${path}`;
    const signals = [AbortSignal.timeout(this.timeout)];
    if (signal) signals.push(signal);
    /** @type {RequestInit} */
    const init = { signal: AbortSignal.any(signals) };
    // Let browsers manage HTTP revalidation; explicit validators bypass their HTTP cache.
    const revalidate = cache && etag && typeof location === 'undefined';
    if (revalidate) init.headers = { 'If-None-Match': etag };
    if (!cache) init.cache = 'no-store';
    const res = await fetch(url, init);
    if (!res.ok && !(revalidate && res.status === 304)) {
      throw new BitviewError(`HTTP ${res.status}: ${url}`, res.status);
    }
    return res;
  }

  /**
   * Make a GET request with layered caching.
   *
   * Contract:
   * - The returned Promise resolves with the **freshest** value (post-revalidation).
   * - `onValue` fires once with the freshest value, or twice if a stale snapshot
   *   could be shown first (stale-while-revalidate). On a 304 there is no second fire.
   *
   * Layers:
   * - L1 (memCache): in-memory parsed values keyed by URL+ETag. Lets 304s skip the parse entirely.
   * - L2 (browserCache): Cache API, survives reload and feeds onValue fast on cold start.
   *
   * @template T
   * @param {string} path
   * @param {(res: Response) => Promise<T>} parse - Response body reader
   * @param {ClientFetchOptions<T>} [options]
   * @returns {Promise<T>}
   */
  async _getCached(path, parse, { onValue, signal, cache = true, memCache = true } = {}) {
    if (!cache) {
      const res = await this.get(path, { signal, cache });
      const value = await parse(res);
      if (onValue) onValue(value);
      return value;
    }

    const url = `${this.baseUrl}${path}`;
    const useMemCache = memCache !== false;
    /** @type {_MemEntry<T> | undefined} */
    const memHit = useMemCache ? this._memGet(url) : undefined;
    const browserCache = this._browserCache;

    // L1 fast path: deliver from memCache, revalidate via network.
    // ETag match → zero parse, zero clone, zero cache write, no second onValue fire.
    if (memHit) {
      if (onValue) onValue(memHit.value);
      try {
        const res = await this.get(path, { signal, etag: memHit.etag });
        const netEtag = res.headers.get('ETag');
        if (res.status === 304 || (netEtag && netEtag === memHit.etag)) {
          await res.body?.cancel();
          return memHit.value;
        }
        const cloned = browserCache ? res.clone() : null;
        const value = await parse(res);
        if (useMemCache) this._memSet(url, netEtag, value);
        if (onValue) onValue(value);
        if (cloned && browserCache) {
          const cacheStore = browserCache;
          _runIdle(() => cacheStore.put(url, cloned));
        }
        return value;
      } catch {
        return memHit.value;
      }
    }

    // L1 miss: race browserCache (stale snapshot) vs network (fresh).
    let networkSettled = false;
    const stalePromise = onValue && browserCache
      ? browserCache.match(url).then(async (res) => {
          if (!res || networkSettled) return null;
          const value = await parse(res);
          if (networkSettled) return value;
          if (useMemCache) this._memSet(url, res.headers.get('ETag'), value);
          onValue(value);
          return value;
        }).catch(() => null)
      : null;

    try {
      const res = await this.get(path, { signal });
      networkSettled = true;
      const netEtag = res.headers.get('ETag');
      // Stale won and populated memCache with matching ETag → reuse, skip parse + second onValue.
      const populated = useMemCache ? /** @type {_MemEntry<T> | undefined} */ (this._memGet(url)) : undefined;
      if (populated && netEtag && netEtag === populated.etag) {
        await res.body?.cancel();
        return populated.value;
      }
      const cloned = browserCache ? res.clone() : null;
      const value = await parse(res);
      if (useMemCache) this._memSet(url, netEtag, value);
      if (onValue) onValue(value);
      if (cloned && browserCache) {
        const cacheStore = browserCache;
        _runIdle(() => cacheStore.put(url, cloned));
      }
      return value;
    } catch (e) {
      const stale = await stalePromise;
      if (stale != null) return stale;
      throw e;
    }
  }

  /**
   * Make a GET request expecting a JSON response. Cached and supports `onValue`.
   * @template T
   * @param {string} path
   * @param {ClientFetchOptions<T>} [options]
   * @returns {Promise<T>}
   */
  getJson(path, options) {
    return this._getCached(path, async (res) => _addCamelGetters(await res.json()), options);
  }

  /**
   * Make a GET request expecting a text response (text/plain, text/csv, ...).
   * Cached and supports `onValue`, same as `getJson`.
   * @param {string} path
   * @param {ClientFetchOptions<string>} [options]
   * @returns {Promise<string>}
   */
  getText(path, options) {
    return this._getCached(path, (res) => res.text(), options);
  }

  /**
   * Make a GET request expecting binary data (application/octet-stream).
   * Cached and supports `onValue`, same as `getJson`.
   * @param {string} path
   * @param {ClientFetchOptions<Uint8Array>} [options]
   * @returns {Promise<Uint8Array>}
   */
  getBytes(path, options) {
    return this._getCached(path, async (res) => new Uint8Array(await res.arrayBuffer()), options);
  }

  /**
   * Make a POST request with a string body.
   *
   * POST responses are uncached and never invoke `onValue` — every call hits
   * the network with the same body and returns the upstream response.
   *
   * @param {string} path
   * @param {string} body
   * @param {{ signal?: AbortSignal }} [options]
   * @returns {Promise<Response>}
   */
  async post(path, body, { signal } = {}) {
    const url = `${this.baseUrl}${path}`;
    const signals = [AbortSignal.timeout(this.timeout)];
    if (signal) signals.push(signal);
    const res = await fetch(url, {
      method: 'POST',
      body,
      signal: AbortSignal.any(signals),
    });
    if (!res.ok) throw new BitviewError(`HTTP ${res.status}: ${url}`, res.status);
    return res;
  }

  /**
   * Make a POST request expecting a JSON response.
   * @template T
   * @param {string} path
   * @param {string} body
   * @param {{ signal?: AbortSignal }} [options]
   * @returns {Promise<T>}
   */
  async postJson(path, body, options) {
    const res = await this.post(path, body, options);
    return _addCamelGetters(await res.json());
  }

  /**
   * Make a POST request expecting a text response.
   * @param {string} path
   * @param {string} body
   * @param {{ signal?: AbortSignal }} [options]
   * @returns {Promise<string>}
   */
  async postText(path, body, options) {
    const res = await this.post(path, body, options);
    return res.text();
  }

  /**
   * Make a POST request expecting binary data (application/octet-stream).
   * @param {string} path
   * @param {string} body
   * @param {{ signal?: AbortSignal }} [options]
   * @returns {Promise<Uint8Array>}
   */
  async postBytes(path, body, options) {
    const res = await this.post(path, body, options);
    return new Uint8Array(await res.arrayBuffer());
  }

  /**
   * Fetch series data and wrap with helper methods (internal)
   * @template T
   * @param {string} path
   * @param {DateSeriesFetchArg<T>} [arg]
   * @param {ClientFetchOptions<DateSeriesData<T>>} [options]
   * @returns {Promise<DateSeriesData<T>>}
   */
  async _fetchSeriesData(path, arg, options) {
    const requestOptions = typeof arg === 'function'
      ? { ...(options ?? {}), onValue: arg }
      : { ...(arg ?? {}), ...(options ?? {}) };
    const onValue = requestOptions.onValue;
    const wrappedOnValue = onValue ? (/** @type {SeriesData<T>} */ raw) => onValue(_wrapSeriesData(raw)) : undefined;
    const raw = await this.getJson(path, { ...requestOptions, onValue: wrappedOnValue });
    return _wrapSeriesData(raw);
  }
}

/**
 * Build series name with suffix.
 * @param {string} acc - Accumulated prefix
 * @param {string} s - Series suffix
 * @returns {string}
 */
const _m = (acc, s) => s ? (acc ? `${acc}_${s}` : s) : acc;

/**
 * Build series name with prefix.
 * @param {string} prefix - Prefix to prepend
 * @param {string} acc - Accumulated name
 * @returns {string}
 */
const _p = (prefix, acc) => acc ? `${prefix}_${acc}` : prefix;

/**
 * Materialize and replace a lazy object property.
 * @template T
 * @param {object} owner
 * @param {string} name
 * @param {() => T} init
 * @returns {T}
 */
function _lazy(owner, name, init) {
  const value = init();
  Object.defineProperty(owner, name, { value, writable: true, enumerable: true, configurable: true });
  return value;
}



const _MASK_64 = 0xffffffffffffffffn;
const _RAPIDHASH_SECRETS = /** @type {const} */ ([
  0x2d358dccaa6c78a5n,
  0x8bb84b93962eacc9n,
  0x4b33a62ed433d4a3n,
  0x4d5a2da51de1aa47n,
  0xa0761d6478bd642fn,
  0xe7037ed1a0b428dbn,
  0x90ed1765281c388cn,
]);
const _RAPIDHASH_SEED = _rapidHashSeed(0n);

/** @param {bigint} value */
function _u64(value) {
  return value & _MASK_64;
}

/** @param {bigint} left @param {bigint} right */
function _rapidMix(left, right) {
  const result = _u64(left) * _u64(right);
  return _u64(result) ^ _u64(result >> 64n);
}

/** @param {bigint} left @param {bigint} right @returns {[bigint, bigint]} */
function _rapidMum(left, right) {
  const result = _u64(left) * _u64(right);
  return [_u64(result), _u64(result >> 64n)];
}

/** @param {bigint} seed */
function _rapidHashSeed(seed) {
  return _u64(seed ^ _rapidMix(seed ^ _RAPIDHASH_SECRETS[2], _RAPIDHASH_SECRETS[1]));
}

/** @param {Uint8Array} bytes @param {number} offset */
function _readU32(bytes, offset) {
  return (
    BigInt(bytes[offset]) |
    (BigInt(bytes[offset + 1]) << 8n) |
    (BigInt(bytes[offset + 2]) << 16n) |
    (BigInt(bytes[offset + 3]) << 24n)
  );
}

/** @param {Uint8Array} bytes @param {number} offset */
function _readU64(bytes, offset) {
  return _readU32(bytes, offset) | (_readU32(bytes, offset + 4) << 32n);
}

/** @param {Uint8Array | ArrayBuffer | ArrayBufferView | number[]} payload */
function _asUint8Array(payload) {
  if (payload instanceof Uint8Array) return payload;
  if (payload instanceof ArrayBuffer) return new Uint8Array(payload);
  if (ArrayBuffer.isView(payload)) return new Uint8Array(payload.buffer, payload.byteOffset, payload.byteLength);
  if (Array.isArray(payload)) return new Uint8Array(payload);
  throw new Error("Expected address payload bytes");
}

/** @param {Uint8Array | ArrayBuffer | ArrayBufferView | number[]} payload */
function _rapidHashV3(payload) {
  const bytes = _asUint8Array(payload);
  const length = bytes.length;
  if (length === 0) throw new Error("Expected a non-empty address payload");
  if (length > 65) throw new Error("Expected at most 65 address payload bytes");

  let seed = _RAPIDHASH_SEED;
  let a = 0n;
  let b = 0n;
  let remainder;

  if (length <= 16) {
    if (length >= 4) {
      seed ^= BigInt(length);
      if (length >= 8) {
        a ^= _readU64(bytes, 0);
        b ^= _readU64(bytes, length - 8);
      } else {
        a ^= _readU32(bytes, 0);
        b ^= _readU32(bytes, length - 4);
      }
    } else if (length > 0) {
      a ^= (BigInt(bytes[0]) << 45n) | BigInt(bytes[length - 1]);
      b ^= BigInt(bytes[length >> 1]);
    }
    remainder = BigInt(length);
  } else {
    seed = _rapidMix(_readU64(bytes, 0) ^ _RAPIDHASH_SECRETS[2], _readU64(bytes, 8) ^ seed);
    if (length > 32) {
      seed = _rapidMix(_readU64(bytes, 16) ^ _RAPIDHASH_SECRETS[2], _readU64(bytes, 24) ^ seed);
      if (length > 48) {
        seed = _rapidMix(_readU64(bytes, 32) ^ _RAPIDHASH_SECRETS[1], _readU64(bytes, 40) ^ seed);
        if (length > 64) {
          seed = _rapidMix(_readU64(bytes, 48) ^ _RAPIDHASH_SECRETS[1], _readU64(bytes, 56) ^ seed);
        }
      }
    }
    remainder = BigInt(length);
    a ^= _readU64(bytes, length - 16) ^ remainder;
    b ^= _readU64(bytes, length - 8);
  }

  a ^= _RAPIDHASH_SECRETS[1];
  b ^= seed;
  [a, b] = _rapidMum(a, b);
  return _rapidMix(a ^ 0xaaaaaaaaaaaaaaaan, b ^ _RAPIDHASH_SECRETS[1] ^ remainder);
}

/** @param {number} nibbles */
function _validateHashPrefixNibbles(nibbles) {
  if (!Number.isInteger(nibbles) || nibbles < 1 || nibbles > 16) {
    throw new Error("Expected hash-prefix length from 1 to 16 hex nibbles");
  }
}

/** @param {OutputType} addrType @returns {number[]} */
function _addressPayloadLengths(addrType) {
  switch (addrType) {
    case "p2a": return [2];
    case "p2pk33": return [33];
    case "p2pk65": return [65];
    case "p2pkh":
    case "p2sh":
    case "p2wpkh": return [20];
    case "p2wsh":
    case "p2tr": return [32];
    default:
      throw new Error(`Unsupported address type for address payload hash-prefix: ${addrType}`);
  }
}

/**
 * @param {OutputType} addrType
 * @param {Uint8Array | ArrayBuffer | ArrayBufferView | number[]} payload
 */
function _validateAddressPayloadForType(addrType, payload) {
  const length = _asUint8Array(payload).length;
  const expected = _addressPayloadLengths(addrType);
  if (!expected.includes(length)) {
    throw new Error(`Expected ${addrType} address payload length ${expected.join(" or ")} bytes`);
  }
}

/**
 * Compute the RapidHash v3 hash-prefix used by `/api/address/hash-prefix/{addr_type}/{prefix}`.
 * @param {Uint8Array | ArrayBuffer | ArrayBufferView | number[]} payload - Raw address payload bytes
 * @param {number} nibbles - Prefix length from 1 to 16 hex nibbles
 * @returns {string}
 */
function addressPayloadHashPrefix(payload, nibbles) {
  _validateHashPrefixNibbles(nibbles);
  return _rapidHashV3(payload).toString(16).padStart(16, "0").slice(0, nibbles);
}

// Index group constants and factory

const _i1 = /** @type {const} */ (["minute10", "minute30", "hour1", "hour4", "hour12", "day1", "day3", "week1", "month1", "month3", "month6", "year1", "year10", "halving", "epoch", "height"]);
const _i2 = /** @type {const} */ (["minute10", "minute30", "hour1", "hour4", "hour12", "day1", "day3", "week1", "month1", "month3", "month6", "year1", "year10", "halving", "epoch"]);
const _i3 = /** @type {const} */ (["minute10"]);
const _i4 = /** @type {const} */ (["minute30"]);
const _i5 = /** @type {const} */ (["hour1"]);
const _i6 = /** @type {const} */ (["hour4"]);
const _i7 = /** @type {const} */ (["hour12"]);
const _i8 = /** @type {const} */ (["day1"]);
const _i9 = /** @type {const} */ (["day3"]);
const _i10 = /** @type {const} */ (["week1"]);
const _i11 = /** @type {const} */ (["month1"]);
const _i12 = /** @type {const} */ (["month3"]);
const _i13 = /** @type {const} */ (["month6"]);
const _i14 = /** @type {const} */ (["year1"]);
const _i15 = /** @type {const} */ (["year10"]);
const _i16 = /** @type {const} */ (["halving"]);
const _i17 = /** @type {const} */ (["epoch"]);
const _i18 = /** @type {const} */ (["height"]);
const _i19 = /** @type {const} */ (["tx_index"]);
const _i20 = /** @type {const} */ (["txin_index"]);
const _i21 = /** @type {const} */ (["txout_index"]);
const _i22 = /** @type {const} */ (["empty_output_index"]);
const _i23 = /** @type {const} */ (["op_return_index"]);
const _i24 = /** @type {const} */ (["p2a_addr_index"]);
const _i25 = /** @type {const} */ (["p2ms_output_index"]);
const _i26 = /** @type {const} */ (["p2pk33_addr_index"]);
const _i27 = /** @type {const} */ (["p2pk65_addr_index"]);
const _i28 = /** @type {const} */ (["p2pkh_addr_index"]);
const _i29 = /** @type {const} */ (["p2sh_addr_index"]);
const _i30 = /** @type {const} */ (["p2tr_addr_index"]);
const _i31 = /** @type {const} */ (["p2wpkh_addr_index"]);
const _i32 = /** @type {const} */ (["p2wsh_addr_index"]);
const _i33 = /** @type {const} */ (["unknown_output_index"]);
const _i34 = /** @type {const} */ (["funded_addr_index"]);
const _i35 = /** @type {const} */ (["extended_empty_addr_index"]);

/**
 * Generic series pattern factory.
 * @template T
 * @param {BitviewClient} client
 * @param {string} name - The series vec name
 * @param {readonly Index[]} indexes - The supported indexes
 */
function _mp(client, name, indexes) {
  const by = {};
  for (const idx of indexes) {
    Object.defineProperty(by, idx, {
      get() { return _endpoint(client, name, idx); },
      enumerable: true,
      configurable: true
    });
  }
  return {
    name,
    by,
    /** @returns {readonly Index[]} */
    indexes() { return indexes; },
    /** @param {Index} index @returns {SeriesEndpoint<T>|undefined} */
    get(index) { return indexes.includes(index) ? _endpoint(client, name, index) : undefined; }
  };
}

/** @template T @typedef {{ name: string, by: { readonly minute10: DateSeriesEndpoint<T>, readonly minute30: DateSeriesEndpoint<T>, readonly hour1: DateSeriesEndpoint<T>, readonly hour4: DateSeriesEndpoint<T>, readonly hour12: DateSeriesEndpoint<T>, readonly day1: DateSeriesEndpoint<T>, readonly day3: DateSeriesEndpoint<T>, readonly week1: DateSeriesEndpoint<T>, readonly month1: DateSeriesEndpoint<T>, readonly month3: DateSeriesEndpoint<T>, readonly month6: DateSeriesEndpoint<T>, readonly year1: DateSeriesEndpoint<T>, readonly year10: DateSeriesEndpoint<T>, readonly halving: SeriesEndpoint<T>, readonly epoch: SeriesEndpoint<T>, readonly height: SeriesEndpoint<T> }, indexes: () => readonly Index[], get: (index: Index) => SeriesEndpoint<T>|undefined }} SeriesPattern1 */
/** @template T @param {BitviewClient} client @param {string} name @returns {SeriesPattern1<T>} */
function createSeriesPattern1(client, name) { return /** @type {SeriesPattern1<T>} */ (_mp(client, name, _i1)); }
/** @template T @typedef {{ name: string, by: { readonly minute10: DateSeriesEndpoint<T>, readonly minute30: DateSeriesEndpoint<T>, readonly hour1: DateSeriesEndpoint<T>, readonly hour4: DateSeriesEndpoint<T>, readonly hour12: DateSeriesEndpoint<T>, readonly day1: DateSeriesEndpoint<T>, readonly day3: DateSeriesEndpoint<T>, readonly week1: DateSeriesEndpoint<T>, readonly month1: DateSeriesEndpoint<T>, readonly month3: DateSeriesEndpoint<T>, readonly month6: DateSeriesEndpoint<T>, readonly year1: DateSeriesEndpoint<T>, readonly year10: DateSeriesEndpoint<T>, readonly halving: SeriesEndpoint<T>, readonly epoch: SeriesEndpoint<T> }, indexes: () => readonly Index[], get: (index: Index) => SeriesEndpoint<T>|undefined }} SeriesPattern2 */
/** @template T @param {BitviewClient} client @param {string} name @returns {SeriesPattern2<T>} */
function createSeriesPattern2(client, name) { return /** @type {SeriesPattern2<T>} */ (_mp(client, name, _i2)); }
/** @template T @typedef {{ name: string, by: { readonly minute10: DateSeriesEndpoint<T> }, indexes: () => readonly Index[], get: (index: Index) => SeriesEndpoint<T>|undefined }} SeriesPattern3 */
/** @template T @param {BitviewClient} client @param {string} name @returns {SeriesPattern3<T>} */
function createSeriesPattern3(client, name) { return /** @type {SeriesPattern3<T>} */ (_mp(client, name, _i3)); }
/** @template T @typedef {{ name: string, by: { readonly minute30: DateSeriesEndpoint<T> }, indexes: () => readonly Index[], get: (index: Index) => SeriesEndpoint<T>|undefined }} SeriesPattern4 */
/** @template T @param {BitviewClient} client @param {string} name @returns {SeriesPattern4<T>} */
function createSeriesPattern4(client, name) { return /** @type {SeriesPattern4<T>} */ (_mp(client, name, _i4)); }
/** @template T @typedef {{ name: string, by: { readonly hour1: DateSeriesEndpoint<T> }, indexes: () => readonly Index[], get: (index: Index) => SeriesEndpoint<T>|undefined }} SeriesPattern5 */
/** @template T @param {BitviewClient} client @param {string} name @returns {SeriesPattern5<T>} */
function createSeriesPattern5(client, name) { return /** @type {SeriesPattern5<T>} */ (_mp(client, name, _i5)); }
/** @template T @typedef {{ name: string, by: { readonly hour4: DateSeriesEndpoint<T> }, indexes: () => readonly Index[], get: (index: Index) => SeriesEndpoint<T>|undefined }} SeriesPattern6 */
/** @template T @param {BitviewClient} client @param {string} name @returns {SeriesPattern6<T>} */
function createSeriesPattern6(client, name) { return /** @type {SeriesPattern6<T>} */ (_mp(client, name, _i6)); }
/** @template T @typedef {{ name: string, by: { readonly hour12: DateSeriesEndpoint<T> }, indexes: () => readonly Index[], get: (index: Index) => SeriesEndpoint<T>|undefined }} SeriesPattern7 */
/** @template T @param {BitviewClient} client @param {string} name @returns {SeriesPattern7<T>} */
function createSeriesPattern7(client, name) { return /** @type {SeriesPattern7<T>} */ (_mp(client, name, _i7)); }
/** @template T @typedef {{ name: string, by: { readonly day1: DateSeriesEndpoint<T> }, indexes: () => readonly Index[], get: (index: Index) => SeriesEndpoint<T>|undefined }} SeriesPattern8 */
/** @template T @param {BitviewClient} client @param {string} name @returns {SeriesPattern8<T>} */
function createSeriesPattern8(client, name) { return /** @type {SeriesPattern8<T>} */ (_mp(client, name, _i8)); }
/** @template T @typedef {{ name: string, by: { readonly day3: DateSeriesEndpoint<T> }, indexes: () => readonly Index[], get: (index: Index) => SeriesEndpoint<T>|undefined }} SeriesPattern9 */
/** @template T @param {BitviewClient} client @param {string} name @returns {SeriesPattern9<T>} */
function createSeriesPattern9(client, name) { return /** @type {SeriesPattern9<T>} */ (_mp(client, name, _i9)); }
/** @template T @typedef {{ name: string, by: { readonly week1: DateSeriesEndpoint<T> }, indexes: () => readonly Index[], get: (index: Index) => SeriesEndpoint<T>|undefined }} SeriesPattern10 */
/** @template T @param {BitviewClient} client @param {string} name @returns {SeriesPattern10<T>} */
function createSeriesPattern10(client, name) { return /** @type {SeriesPattern10<T>} */ (_mp(client, name, _i10)); }
/** @template T @typedef {{ name: string, by: { readonly month1: DateSeriesEndpoint<T> }, indexes: () => readonly Index[], get: (index: Index) => SeriesEndpoint<T>|undefined }} SeriesPattern11 */
/** @template T @param {BitviewClient} client @param {string} name @returns {SeriesPattern11<T>} */
function createSeriesPattern11(client, name) { return /** @type {SeriesPattern11<T>} */ (_mp(client, name, _i11)); }
/** @template T @typedef {{ name: string, by: { readonly month3: DateSeriesEndpoint<T> }, indexes: () => readonly Index[], get: (index: Index) => SeriesEndpoint<T>|undefined }} SeriesPattern12 */
/** @template T @param {BitviewClient} client @param {string} name @returns {SeriesPattern12<T>} */
function createSeriesPattern12(client, name) { return /** @type {SeriesPattern12<T>} */ (_mp(client, name, _i12)); }
/** @template T @typedef {{ name: string, by: { readonly month6: DateSeriesEndpoint<T> }, indexes: () => readonly Index[], get: (index: Index) => SeriesEndpoint<T>|undefined }} SeriesPattern13 */
/** @template T @param {BitviewClient} client @param {string} name @returns {SeriesPattern13<T>} */
function createSeriesPattern13(client, name) { return /** @type {SeriesPattern13<T>} */ (_mp(client, name, _i13)); }
/** @template T @typedef {{ name: string, by: { readonly year1: DateSeriesEndpoint<T> }, indexes: () => readonly Index[], get: (index: Index) => SeriesEndpoint<T>|undefined }} SeriesPattern14 */
/** @template T @param {BitviewClient} client @param {string} name @returns {SeriesPattern14<T>} */
function createSeriesPattern14(client, name) { return /** @type {SeriesPattern14<T>} */ (_mp(client, name, _i14)); }
/** @template T @typedef {{ name: string, by: { readonly year10: DateSeriesEndpoint<T> }, indexes: () => readonly Index[], get: (index: Index) => SeriesEndpoint<T>|undefined }} SeriesPattern15 */
/** @template T @param {BitviewClient} client @param {string} name @returns {SeriesPattern15<T>} */
function createSeriesPattern15(client, name) { return /** @type {SeriesPattern15<T>} */ (_mp(client, name, _i15)); }
/** @template T @typedef {{ name: string, by: { readonly halving: SeriesEndpoint<T> }, indexes: () => readonly Index[], get: (index: Index) => SeriesEndpoint<T>|undefined }} SeriesPattern16 */
/** @template T @param {BitviewClient} client @param {string} name @returns {SeriesPattern16<T>} */
function createSeriesPattern16(client, name) { return /** @type {SeriesPattern16<T>} */ (_mp(client, name, _i16)); }
/** @template T @typedef {{ name: string, by: { readonly epoch: SeriesEndpoint<T> }, indexes: () => readonly Index[], get: (index: Index) => SeriesEndpoint<T>|undefined }} SeriesPattern17 */
/** @template T @param {BitviewClient} client @param {string} name @returns {SeriesPattern17<T>} */
function createSeriesPattern17(client, name) { return /** @type {SeriesPattern17<T>} */ (_mp(client, name, _i17)); }
/** @template T @typedef {{ name: string, by: { readonly height: SeriesEndpoint<T> }, indexes: () => readonly Index[], get: (index: Index) => SeriesEndpoint<T>|undefined }} SeriesPattern18 */
/** @template T @param {BitviewClient} client @param {string} name @returns {SeriesPattern18<T>} */
function createSeriesPattern18(client, name) { return /** @type {SeriesPattern18<T>} */ (_mp(client, name, _i18)); }
/** @template T @typedef {{ name: string, by: { readonly tx_index: SeriesEndpoint<T> }, indexes: () => readonly Index[], get: (index: Index) => SeriesEndpoint<T>|undefined }} SeriesPattern19 */
/** @template T @param {BitviewClient} client @param {string} name @returns {SeriesPattern19<T>} */
function createSeriesPattern19(client, name) { return /** @type {SeriesPattern19<T>} */ (_mp(client, name, _i19)); }
/** @template T @typedef {{ name: string, by: { readonly txin_index: SeriesEndpoint<T> }, indexes: () => readonly Index[], get: (index: Index) => SeriesEndpoint<T>|undefined }} SeriesPattern20 */
/** @template T @param {BitviewClient} client @param {string} name @returns {SeriesPattern20<T>} */
function createSeriesPattern20(client, name) { return /** @type {SeriesPattern20<T>} */ (_mp(client, name, _i20)); }
/** @template T @typedef {{ name: string, by: { readonly txout_index: SeriesEndpoint<T> }, indexes: () => readonly Index[], get: (index: Index) => SeriesEndpoint<T>|undefined }} SeriesPattern21 */
/** @template T @param {BitviewClient} client @param {string} name @returns {SeriesPattern21<T>} */
function createSeriesPattern21(client, name) { return /** @type {SeriesPattern21<T>} */ (_mp(client, name, _i21)); }
/** @template T @typedef {{ name: string, by: { readonly empty_output_index: SeriesEndpoint<T> }, indexes: () => readonly Index[], get: (index: Index) => SeriesEndpoint<T>|undefined }} SeriesPattern22 */
/** @template T @param {BitviewClient} client @param {string} name @returns {SeriesPattern22<T>} */
function createSeriesPattern22(client, name) { return /** @type {SeriesPattern22<T>} */ (_mp(client, name, _i22)); }
/** @template T @typedef {{ name: string, by: { readonly op_return_index: SeriesEndpoint<T> }, indexes: () => readonly Index[], get: (index: Index) => SeriesEndpoint<T>|undefined }} SeriesPattern23 */
/** @template T @param {BitviewClient} client @param {string} name @returns {SeriesPattern23<T>} */
function createSeriesPattern23(client, name) { return /** @type {SeriesPattern23<T>} */ (_mp(client, name, _i23)); }
/** @template T @typedef {{ name: string, by: { readonly p2a_addr_index: SeriesEndpoint<T> }, indexes: () => readonly Index[], get: (index: Index) => SeriesEndpoint<T>|undefined }} SeriesPattern24 */
/** @template T @param {BitviewClient} client @param {string} name @returns {SeriesPattern24<T>} */
function createSeriesPattern24(client, name) { return /** @type {SeriesPattern24<T>} */ (_mp(client, name, _i24)); }
/** @template T @typedef {{ name: string, by: { readonly p2ms_output_index: SeriesEndpoint<T> }, indexes: () => readonly Index[], get: (index: Index) => SeriesEndpoint<T>|undefined }} SeriesPattern25 */
/** @template T @param {BitviewClient} client @param {string} name @returns {SeriesPattern25<T>} */
function createSeriesPattern25(client, name) { return /** @type {SeriesPattern25<T>} */ (_mp(client, name, _i25)); }
/** @template T @typedef {{ name: string, by: { readonly p2pk33_addr_index: SeriesEndpoint<T> }, indexes: () => readonly Index[], get: (index: Index) => SeriesEndpoint<T>|undefined }} SeriesPattern26 */
/** @template T @param {BitviewClient} client @param {string} name @returns {SeriesPattern26<T>} */
function createSeriesPattern26(client, name) { return /** @type {SeriesPattern26<T>} */ (_mp(client, name, _i26)); }
/** @template T @typedef {{ name: string, by: { readonly p2pk65_addr_index: SeriesEndpoint<T> }, indexes: () => readonly Index[], get: (index: Index) => SeriesEndpoint<T>|undefined }} SeriesPattern27 */
/** @template T @param {BitviewClient} client @param {string} name @returns {SeriesPattern27<T>} */
function createSeriesPattern27(client, name) { return /** @type {SeriesPattern27<T>} */ (_mp(client, name, _i27)); }
/** @template T @typedef {{ name: string, by: { readonly p2pkh_addr_index: SeriesEndpoint<T> }, indexes: () => readonly Index[], get: (index: Index) => SeriesEndpoint<T>|undefined }} SeriesPattern28 */
/** @template T @param {BitviewClient} client @param {string} name @returns {SeriesPattern28<T>} */
function createSeriesPattern28(client, name) { return /** @type {SeriesPattern28<T>} */ (_mp(client, name, _i28)); }
/** @template T @typedef {{ name: string, by: { readonly p2sh_addr_index: SeriesEndpoint<T> }, indexes: () => readonly Index[], get: (index: Index) => SeriesEndpoint<T>|undefined }} SeriesPattern29 */
/** @template T @param {BitviewClient} client @param {string} name @returns {SeriesPattern29<T>} */
function createSeriesPattern29(client, name) { return /** @type {SeriesPattern29<T>} */ (_mp(client, name, _i29)); }
/** @template T @typedef {{ name: string, by: { readonly p2tr_addr_index: SeriesEndpoint<T> }, indexes: () => readonly Index[], get: (index: Index) => SeriesEndpoint<T>|undefined }} SeriesPattern30 */
/** @template T @param {BitviewClient} client @param {string} name @returns {SeriesPattern30<T>} */
function createSeriesPattern30(client, name) { return /** @type {SeriesPattern30<T>} */ (_mp(client, name, _i30)); }
/** @template T @typedef {{ name: string, by: { readonly p2wpkh_addr_index: SeriesEndpoint<T> }, indexes: () => readonly Index[], get: (index: Index) => SeriesEndpoint<T>|undefined }} SeriesPattern31 */
/** @template T @param {BitviewClient} client @param {string} name @returns {SeriesPattern31<T>} */
function createSeriesPattern31(client, name) { return /** @type {SeriesPattern31<T>} */ (_mp(client, name, _i31)); }
/** @template T @typedef {{ name: string, by: { readonly p2wsh_addr_index: SeriesEndpoint<T> }, indexes: () => readonly Index[], get: (index: Index) => SeriesEndpoint<T>|undefined }} SeriesPattern32 */
/** @template T @param {BitviewClient} client @param {string} name @returns {SeriesPattern32<T>} */
function createSeriesPattern32(client, name) { return /** @type {SeriesPattern32<T>} */ (_mp(client, name, _i32)); }
/** @template T @typedef {{ name: string, by: { readonly unknown_output_index: SeriesEndpoint<T> }, indexes: () => readonly Index[], get: (index: Index) => SeriesEndpoint<T>|undefined }} SeriesPattern33 */
/** @template T @param {BitviewClient} client @param {string} name @returns {SeriesPattern33<T>} */
function createSeriesPattern33(client, name) { return /** @type {SeriesPattern33<T>} */ (_mp(client, name, _i33)); }
/** @template T @typedef {{ name: string, by: { readonly funded_addr_index: SeriesEndpoint<T> }, indexes: () => readonly Index[], get: (index: Index) => SeriesEndpoint<T>|undefined }} SeriesPattern34 */
/** @template T @param {BitviewClient} client @param {string} name @returns {SeriesPattern34<T>} */
function createSeriesPattern34(client, name) { return /** @type {SeriesPattern34<T>} */ (_mp(client, name, _i34)); }
/** @template T @typedef {{ name: string, by: { readonly extended_empty_addr_index: SeriesEndpoint<T> }, indexes: () => readonly Index[], get: (index: Index) => SeriesEndpoint<T>|undefined }} SeriesPattern35 */
/** @template T @param {BitviewClient} client @param {string} name @returns {SeriesPattern35<T>} */
function createSeriesPattern35(client, name) { return /** @type {SeriesPattern35<T>} */ (_mp(client, name, _i35)); }

// Reusable structural pattern factories

/**
 * @typedef {Object} CentsPct0Pct1Pct10Pct2Pct20Pct30Pct40Pct5Pct50Pct60Pct70Pct80Pct90Pct95Pct98Pct99PpmRatioSatsUsdPattern
 * @property {SeriesPattern1<Cents>} cents
 * @property {PpmPriceRatioPattern} pct01
 * @property {PpmPriceRatioPattern} pct05
 * @property {PpmPriceRatioPattern} pct1
 * @property {PpmPriceRatioPattern} pct10
 * @property {PpmPriceRatioPattern} pct2
 * @property {PpmPriceRatioPattern} pct20
 * @property {PpmPriceRatioPattern} pct30
 * @property {PpmPriceRatioPattern} pct40
 * @property {PpmPriceRatioPattern} pct5
 * @property {PpmPriceRatioPattern} pct50
 * @property {PpmPriceRatioPattern} pct60
 * @property {PpmPriceRatioPattern} pct70
 * @property {PpmPriceRatioPattern} pct80
 * @property {PpmPriceRatioPattern} pct90
 * @property {PpmPriceRatioPattern} pct95
 * @property {PpmPriceRatioPattern} pct98
 * @property {PpmPriceRatioPattern} pct99
 * @property {PpmPriceRatioPattern} pct995
 * @property {PpmPriceRatioPattern} pct999
 * @property {SeriesPattern1<PriceRatio>} ppm
 * @property {SeriesPattern1<StoredF32>} ratio
 * @property {SeriesPattern1<SatsFract>} sats
 * @property {SeriesPattern1<Dollars>} usd
 */

/**
 * Create a CentsPct0Pct1Pct10Pct2Pct20Pct30Pct40Pct5Pct50Pct60Pct70Pct80Pct90Pct95Pct98Pct99PpmRatioSatsUsdPattern pattern node
 * @param {BitviewClient} client
 * @param {string} acc - Accumulated series name
 * @returns {CentsPct0Pct1Pct10Pct2Pct20Pct30Pct40Pct5Pct50Pct60Pct70Pct80Pct90Pct95Pct98Pct99PpmRatioSatsUsdPattern}
 */
function createCentsPct0Pct1Pct10Pct2Pct20Pct30Pct40Pct5Pct50Pct60Pct70Pct80Pct90Pct95Pct98Pct99PpmRatioSatsUsdPattern(client, acc) {
  return {
    cents: createSeriesPattern1(client, _m(acc, 'cents')),
    pct01: createPpmPriceRatioPattern(client, acc, 'pct0_1'),
    pct05: createPpmPriceRatioPattern(client, acc, 'pct0_5'),
    pct1: createPpmPriceRatioPattern(client, acc, 'pct1'),
    pct10: createPpmPriceRatioPattern(client, acc, 'pct10'),
    pct2: createPpmPriceRatioPattern(client, acc, 'pct2'),
    pct20: createPpmPriceRatioPattern(client, acc, 'pct20'),
    pct30: createPpmPriceRatioPattern(client, acc, 'pct30'),
    pct40: createPpmPriceRatioPattern(client, acc, 'pct40'),
    pct5: createPpmPriceRatioPattern(client, acc, 'pct5'),
    pct50: createPpmPriceRatioPattern(client, acc, 'pct50'),
    pct60: createPpmPriceRatioPattern(client, acc, 'pct60'),
    pct70: createPpmPriceRatioPattern(client, acc, 'pct70'),
    pct80: createPpmPriceRatioPattern(client, acc, 'pct80'),
    pct90: createPpmPriceRatioPattern(client, acc, 'pct90'),
    pct95: createPpmPriceRatioPattern(client, acc, 'pct95'),
    pct98: createPpmPriceRatioPattern(client, acc, 'pct98'),
    pct99: createPpmPriceRatioPattern(client, acc, 'pct99'),
    pct995: createPpmPriceRatioPattern(client, acc, 'pct99_5'),
    pct999: createPpmPriceRatioPattern(client, acc, 'pct99_9'),
    ppm: createSeriesPattern1(client, _m(acc, 'ratio_ppm')),
    ratio: createSeriesPattern1(client, _m(acc, 'ratio')),
    sats: createSeriesPattern1(client, _m(acc, 'sats')),
    usd: createSeriesPattern1(client, acc),
  };
}

/**
 * @typedef {Object} _10y12y18m1d1h1m1w1y2m2y3m3y4m4y5m5y6m6y7y8y9mOverUnderPattern2
 * @property {AverageBlockCumulativeSumPattern} _10yTo12y
 * @property {AverageBlockCumulativeSumPattern} _12yTo15y
 * @property {AverageBlockCumulativeSumPattern} _18mTo2y
 * @property {AverageBlockCumulativeSumPattern} _1dTo1w
 * @property {AverageBlockCumulativeSumPattern} _1hTo1d
 * @property {AverageBlockCumulativeSumPattern} _1mTo2m
 * @property {AverageBlockCumulativeSumPattern} _1wTo1m
 * @property {AverageBlockCumulativeSumPattern} _1yTo18m
 * @property {AverageBlockCumulativeSumPattern} _2mTo3m
 * @property {AverageBlockCumulativeSumPattern} _2yTo3y
 * @property {AverageBlockCumulativeSumPattern} _3mTo4m
 * @property {AverageBlockCumulativeSumPattern} _3yTo4y
 * @property {AverageBlockCumulativeSumPattern} _4mTo5m
 * @property {AverageBlockCumulativeSumPattern} _4yTo5y
 * @property {AverageBlockCumulativeSumPattern} _5mTo6m
 * @property {AverageBlockCumulativeSumPattern} _5yTo6y
 * @property {AverageBlockCumulativeSumPattern} _6mTo9m
 * @property {AverageBlockCumulativeSumPattern} _6yTo7y
 * @property {AverageBlockCumulativeSumPattern} _7yTo8y
 * @property {AverageBlockCumulativeSumPattern} _8yTo10y
 * @property {AverageBlockCumulativeSumPattern} _9mTo1y
 * @property {AverageBlockCumulativeSumPattern} over15y
 * @property {AverageBlockCumulativeSumPattern} under1h
 */

/**
 * @typedef {Object} _10y12y18m1d1h1m1w1y2m2y3m3y4m4y5m5y6m6y7y8y9mOverUnderPattern
 * @property {AverageBlockCumulativeSumPattern2<StoredF64>} _10yTo12y
 * @property {AverageBlockCumulativeSumPattern2<StoredF64>} _12yTo15y
 * @property {AverageBlockCumulativeSumPattern2<StoredF64>} _18mTo2y
 * @property {AverageBlockCumulativeSumPattern2<StoredF64>} _1dTo1w
 * @property {AverageBlockCumulativeSumPattern2<StoredF64>} _1hTo1d
 * @property {AverageBlockCumulativeSumPattern2<StoredF64>} _1mTo2m
 * @property {AverageBlockCumulativeSumPattern2<StoredF64>} _1wTo1m
 * @property {AverageBlockCumulativeSumPattern2<StoredF64>} _1yTo18m
 * @property {AverageBlockCumulativeSumPattern2<StoredF64>} _2mTo3m
 * @property {AverageBlockCumulativeSumPattern2<StoredF64>} _2yTo3y
 * @property {AverageBlockCumulativeSumPattern2<StoredF64>} _3mTo4m
 * @property {AverageBlockCumulativeSumPattern2<StoredF64>} _3yTo4y
 * @property {AverageBlockCumulativeSumPattern2<StoredF64>} _4mTo5m
 * @property {AverageBlockCumulativeSumPattern2<StoredF64>} _4yTo5y
 * @property {AverageBlockCumulativeSumPattern2<StoredF64>} _5mTo6m
 * @property {AverageBlockCumulativeSumPattern2<StoredF64>} _5yTo6y
 * @property {AverageBlockCumulativeSumPattern2<StoredF64>} _6mTo9m
 * @property {AverageBlockCumulativeSumPattern2<StoredF64>} _6yTo7y
 * @property {AverageBlockCumulativeSumPattern2<StoredF64>} _7yTo8y
 * @property {AverageBlockCumulativeSumPattern2<StoredF64>} _8yTo10y
 * @property {AverageBlockCumulativeSumPattern2<StoredF64>} _9mTo1y
 * @property {AverageBlockCumulativeSumPattern2<StoredF64>} over15y
 * @property {AverageBlockCumulativeSumPattern2<StoredF64>} under1h
 */

/**
 * @typedef {Object} _10y12y18m1d1h1m1w1y2m2y3m3y4m4y5m5y6m6y7y8y9mOverUnderPattern5
 * @property {BlockCumulativeSumPattern} _10yTo12y
 * @property {BlockCumulativeSumPattern} _12yTo15y
 * @property {BlockCumulativeSumPattern} _18mTo2y
 * @property {BlockCumulativeSumPattern} _1dTo1w
 * @property {BlockCumulativeSumPattern} _1hTo1d
 * @property {BlockCumulativeSumPattern} _1mTo2m
 * @property {BlockCumulativeSumPattern} _1wTo1m
 * @property {BlockCumulativeSumPattern} _1yTo18m
 * @property {BlockCumulativeSumPattern} _2mTo3m
 * @property {BlockCumulativeSumPattern} _2yTo3y
 * @property {BlockCumulativeSumPattern} _3mTo4m
 * @property {BlockCumulativeSumPattern} _3yTo4y
 * @property {BlockCumulativeSumPattern} _4mTo5m
 * @property {BlockCumulativeSumPattern} _4yTo5y
 * @property {BlockCumulativeSumPattern} _5mTo6m
 * @property {BlockCumulativeSumPattern} _5yTo6y
 * @property {BlockCumulativeSumPattern} _6mTo9m
 * @property {BlockCumulativeSumPattern} _6yTo7y
 * @property {BlockCumulativeSumPattern} _7yTo8y
 * @property {BlockCumulativeSumPattern} _8yTo10y
 * @property {BlockCumulativeSumPattern} _9mTo1y
 * @property {BlockCumulativeSumPattern} over15y
 * @property {BlockCumulativeSumPattern} under1h
 */

/**
 * @typedef {Object} _10y12y18m1d1h1m1w1y2m2y3m3y4m4y5m5y6m6y7y8y9mOverUnderPattern10
 * @property {BtcCentsSatsUsdPattern} _10yTo12y
 * @property {BtcCentsSatsUsdPattern} _12yTo15y
 * @property {BtcCentsSatsUsdPattern} _18mTo2y
 * @property {BtcCentsSatsUsdPattern} _1dTo1w
 * @property {BtcCentsSatsUsdPattern} _1hTo1d
 * @property {BtcCentsSatsUsdPattern} _1mTo2m
 * @property {BtcCentsSatsUsdPattern} _1wTo1m
 * @property {BtcCentsSatsUsdPattern} _1yTo18m
 * @property {BtcCentsSatsUsdPattern} _2mTo3m
 * @property {BtcCentsSatsUsdPattern} _2yTo3y
 * @property {BtcCentsSatsUsdPattern} _3mTo4m
 * @property {BtcCentsSatsUsdPattern} _3yTo4y
 * @property {BtcCentsSatsUsdPattern} _4mTo5m
 * @property {BtcCentsSatsUsdPattern} _4yTo5y
 * @property {BtcCentsSatsUsdPattern} _5mTo6m
 * @property {BtcCentsSatsUsdPattern} _5yTo6y
 * @property {BtcCentsSatsUsdPattern} _6mTo9m
 * @property {BtcCentsSatsUsdPattern} _6yTo7y
 * @property {BtcCentsSatsUsdPattern} _7yTo8y
 * @property {BtcCentsSatsUsdPattern} _8yTo10y
 * @property {BtcCentsSatsUsdPattern} _9mTo1y
 * @property {BtcCentsSatsUsdPattern} over15y
 * @property {BtcCentsSatsUsdPattern} under1h
 */

/**
 * @typedef {Object} _10y12y18m1d1h1m1w1y2m2y3m3y4m4y5m5y6m6y7y8y9mOverUnderPattern4
 * @property {CentsUsdPattern} _10yTo12y
 * @property {CentsUsdPattern} _12yTo15y
 * @property {CentsUsdPattern} _18mTo2y
 * @property {CentsUsdPattern} _1dTo1w
 * @property {CentsUsdPattern} _1hTo1d
 * @property {CentsUsdPattern} _1mTo2m
 * @property {CentsUsdPattern} _1wTo1m
 * @property {CentsUsdPattern} _1yTo18m
 * @property {CentsUsdPattern} _2mTo3m
 * @property {CentsUsdPattern} _2yTo3y
 * @property {CentsUsdPattern} _3mTo4m
 * @property {CentsUsdPattern} _3yTo4y
 * @property {CentsUsdPattern} _4mTo5m
 * @property {CentsUsdPattern} _4yTo5y
 * @property {CentsUsdPattern} _5mTo6m
 * @property {CentsUsdPattern} _5yTo6y
 * @property {CentsUsdPattern} _6mTo9m
 * @property {CentsUsdPattern} _6yTo7y
 * @property {CentsUsdPattern} _7yTo8y
 * @property {CentsUsdPattern} _8yTo10y
 * @property {CentsUsdPattern} _9mTo1y
 * @property {CentsUsdPattern} over15y
 * @property {CentsUsdPattern} under1h
 */

/**
 * @typedef {Object} _10y12y18m1d1h1m1w1y2m2y3m3y4m4y5m5y6m6y7y8y9mOverUnderPattern12
 * @property {SeriesPattern1<StoredF64>} _10yTo12y
 * @property {SeriesPattern1<StoredF64>} _12yTo15y
 * @property {SeriesPattern1<StoredF64>} _18mTo2y
 * @property {SeriesPattern1<StoredF64>} _1dTo1w
 * @property {SeriesPattern1<StoredF64>} _1hTo1d
 * @property {SeriesPattern1<StoredF64>} _1mTo2m
 * @property {SeriesPattern1<StoredF64>} _1wTo1m
 * @property {SeriesPattern1<StoredF64>} _1yTo18m
 * @property {SeriesPattern1<StoredF64>} _2mTo3m
 * @property {SeriesPattern1<StoredF64>} _2yTo3y
 * @property {SeriesPattern1<StoredF64>} _3mTo4m
 * @property {SeriesPattern1<StoredF64>} _3yTo4y
 * @property {SeriesPattern1<StoredF64>} _4mTo5m
 * @property {SeriesPattern1<StoredF64>} _4yTo5y
 * @property {SeriesPattern1<StoredF64>} _5mTo6m
 * @property {SeriesPattern1<StoredF64>} _5yTo6y
 * @property {SeriesPattern1<StoredF64>} _6mTo9m
 * @property {SeriesPattern1<StoredF64>} _6yTo7y
 * @property {SeriesPattern1<StoredF64>} _7yTo8y
 * @property {SeriesPattern1<StoredF64>} _8yTo10y
 * @property {SeriesPattern1<StoredF64>} _9mTo1y
 * @property {SeriesPattern1<StoredF64>} over15y
 * @property {SeriesPattern1<StoredF64>} under1h
 */

/**
 * @typedef {Object} AscribeBareBitproofBlockstackCoinColuDocproofEmptyEpobcEternityFactomKomodoMemoOmniOpenPoetRunesStacksStamperyTextUnknownVeriPattern3
 * @property {AverageBlockCumulativeSumPattern2<StoredU64>} ascribe
 * @property {AverageBlockCumulativeSumPattern2<StoredU64>} bareHash
 * @property {AverageBlockCumulativeSumPattern2<StoredU64>} bitproof
 * @property {AverageBlockCumulativeSumPattern2<StoredU64>} blockstack
 * @property {AverageBlockCumulativeSumPattern2<StoredU64>} coinSpark
 * @property {AverageBlockCumulativeSumPattern2<StoredU64>} colu
 * @property {AverageBlockCumulativeSumPattern2<StoredU64>} docproof
 * @property {AverageBlockCumulativeSumPattern2<StoredU64>} empty
 * @property {AverageBlockCumulativeSumPattern2<StoredU64>} epobc
 * @property {AverageBlockCumulativeSumPattern2<StoredU64>} eternityWall
 * @property {AverageBlockCumulativeSumPattern2<StoredU64>} factom
 * @property {AverageBlockCumulativeSumPattern2<StoredU64>} komodo
 * @property {AverageBlockCumulativeSumPattern2<StoredU64>} memo
 * @property {AverageBlockCumulativeSumPattern2<StoredU64>} omni
 * @property {AverageBlockCumulativeSumPattern2<StoredU64>} openAssets
 * @property {AverageBlockCumulativeSumPattern2<StoredU64>} openTimestamps
 * @property {AverageBlockCumulativeSumPattern2<StoredU64>} poet
 * @property {AverageBlockCumulativeSumPattern2<StoredU64>} runes
 * @property {AverageBlockCumulativeSumPattern2<StoredU64>} stacks
 * @property {AverageBlockCumulativeSumPattern2<StoredU64>} stampery
 * @property {AverageBlockCumulativeSumPattern2<StoredU64>} text
 * @property {AverageBlockCumulativeSumPattern2<StoredU64>} unknown
 * @property {AverageBlockCumulativeSumPattern2<StoredU64>} veriBlock
 */

/**
 * @typedef {Object} IndexPct0Pct1Pct10Pct2Pct20Pct30Pct40Pct5Pct50Pct60Pct70Pct80Pct90Pct95Pct98Pct99ScorePattern
 * @property {SeriesPattern1<StoredI8>} index
 * @property {CentsSatsUsdPattern} pct01
 * @property {CentsSatsUsdPattern} pct05
 * @property {CentsSatsUsdPattern} pct1
 * @property {CentsSatsUsdPattern} pct10
 * @property {CentsSatsUsdPattern} pct2
 * @property {CentsSatsUsdPattern} pct20
 * @property {CentsSatsUsdPattern} pct30
 * @property {CentsSatsUsdPattern} pct40
 * @property {CentsSatsUsdPattern} pct5
 * @property {CentsSatsUsdPattern} pct50
 * @property {CentsSatsUsdPattern} pct60
 * @property {CentsSatsUsdPattern} pct70
 * @property {CentsSatsUsdPattern} pct80
 * @property {CentsSatsUsdPattern} pct90
 * @property {CentsSatsUsdPattern} pct95
 * @property {CentsSatsUsdPattern} pct98
 * @property {CentsSatsUsdPattern} pct99
 * @property {CentsSatsUsdPattern} pct995
 * @property {CentsSatsUsdPattern} pct999
 * @property {SeriesPattern1<StoredI8>} score
 */

/**
 * Create a IndexPct0Pct1Pct10Pct2Pct20Pct30Pct40Pct5Pct50Pct60Pct70Pct80Pct90Pct95Pct98Pct99ScorePattern pattern node
 * @param {BitviewClient} client
 * @param {string} acc - Accumulated series name
 * @returns {IndexPct0Pct1Pct10Pct2Pct20Pct30Pct40Pct5Pct50Pct60Pct70Pct80Pct90Pct95Pct98Pct99ScorePattern}
 */
function createIndexPct0Pct1Pct10Pct2Pct20Pct30Pct40Pct5Pct50Pct60Pct70Pct80Pct90Pct95Pct98Pct99ScorePattern(client, acc) {
  return {
    index: createSeriesPattern1(client, _m(acc, 'index')),
    pct01: createCentsSatsUsdPattern(client, _m(acc, 'pct0_1')),
    pct05: createCentsSatsUsdPattern(client, _m(acc, 'pct0_5')),
    pct1: createCentsSatsUsdPattern(client, _m(acc, 'pct01')),
    pct10: createCentsSatsUsdPattern(client, _m(acc, 'pct10')),
    pct2: createCentsSatsUsdPattern(client, _m(acc, 'pct02')),
    pct20: createCentsSatsUsdPattern(client, _m(acc, 'pct20')),
    pct30: createCentsSatsUsdPattern(client, _m(acc, 'pct30')),
    pct40: createCentsSatsUsdPattern(client, _m(acc, 'pct40')),
    pct5: createCentsSatsUsdPattern(client, _m(acc, 'pct05')),
    pct50: createCentsSatsUsdPattern(client, _m(acc, 'pct50')),
    pct60: createCentsSatsUsdPattern(client, _m(acc, 'pct60')),
    pct70: createCentsSatsUsdPattern(client, _m(acc, 'pct70')),
    pct80: createCentsSatsUsdPattern(client, _m(acc, 'pct80')),
    pct90: createCentsSatsUsdPattern(client, _m(acc, 'pct90')),
    pct95: createCentsSatsUsdPattern(client, _m(acc, 'pct95')),
    pct98: createCentsSatsUsdPattern(client, _m(acc, 'pct98')),
    pct99: createCentsSatsUsdPattern(client, _m(acc, 'pct99')),
    pct995: createCentsSatsUsdPattern(client, _m(acc, 'pct99_5')),
    pct999: createCentsSatsUsdPattern(client, _m(acc, 'pct99_9')),
    score: createSeriesPattern1(client, _m(acc, 'score')),
  };
}

/**
 * @typedef {Object} Pct05Pct10Pct15Pct20Pct25Pct30Pct35Pct40Pct45Pct50Pct55Pct60Pct65Pct70Pct75Pct80Pct85Pct90Pct95Pattern
 * @property {CentsSatsUsdPattern} pct05
 * @property {CentsSatsUsdPattern} pct10
 * @property {CentsSatsUsdPattern} pct15
 * @property {CentsSatsUsdPattern} pct20
 * @property {CentsSatsUsdPattern} pct25
 * @property {CentsSatsUsdPattern} pct30
 * @property {CentsSatsUsdPattern} pct35
 * @property {CentsSatsUsdPattern} pct40
 * @property {CentsSatsUsdPattern} pct45
 * @property {CentsSatsUsdPattern} pct50
 * @property {CentsSatsUsdPattern} pct55
 * @property {CentsSatsUsdPattern} pct60
 * @property {CentsSatsUsdPattern} pct65
 * @property {CentsSatsUsdPattern} pct70
 * @property {CentsSatsUsdPattern} pct75
 * @property {CentsSatsUsdPattern} pct80
 * @property {CentsSatsUsdPattern} pct85
 * @property {CentsSatsUsdPattern} pct90
 * @property {CentsSatsUsdPattern} pct95
 */

/**
 * Create a Pct05Pct10Pct15Pct20Pct25Pct30Pct35Pct40Pct45Pct50Pct55Pct60Pct65Pct70Pct75Pct80Pct85Pct90Pct95Pattern pattern node
 * @param {BitviewClient} client
 * @param {string} acc - Accumulated series name
 * @returns {Pct05Pct10Pct15Pct20Pct25Pct30Pct35Pct40Pct45Pct50Pct55Pct60Pct65Pct70Pct75Pct80Pct85Pct90Pct95Pattern}
 */
function createPct05Pct10Pct15Pct20Pct25Pct30Pct35Pct40Pct45Pct50Pct55Pct60Pct65Pct70Pct75Pct80Pct85Pct90Pct95Pattern(client, acc) {
  return {
    pct05: createCentsSatsUsdPattern(client, _m(acc, 'pct05')),
    pct10: createCentsSatsUsdPattern(client, _m(acc, 'pct10')),
    pct15: createCentsSatsUsdPattern(client, _m(acc, 'pct15')),
    pct20: createCentsSatsUsdPattern(client, _m(acc, 'pct20')),
    pct25: createCentsSatsUsdPattern(client, _m(acc, 'pct25')),
    pct30: createCentsSatsUsdPattern(client, _m(acc, 'pct30')),
    pct35: createCentsSatsUsdPattern(client, _m(acc, 'pct35')),
    pct40: createCentsSatsUsdPattern(client, _m(acc, 'pct40')),
    pct45: createCentsSatsUsdPattern(client, _m(acc, 'pct45')),
    pct50: createCentsSatsUsdPattern(client, _m(acc, 'pct50')),
    pct55: createCentsSatsUsdPattern(client, _m(acc, 'pct55')),
    pct60: createCentsSatsUsdPattern(client, _m(acc, 'pct60')),
    pct65: createCentsSatsUsdPattern(client, _m(acc, 'pct65')),
    pct70: createCentsSatsUsdPattern(client, _m(acc, 'pct70')),
    pct75: createCentsSatsUsdPattern(client, _m(acc, 'pct75')),
    pct80: createCentsSatsUsdPattern(client, _m(acc, 'pct80')),
    pct85: createCentsSatsUsdPattern(client, _m(acc, 'pct85')),
    pct90: createCentsSatsUsdPattern(client, _m(acc, 'pct90')),
    pct95: createCentsSatsUsdPattern(client, _m(acc, 'pct95')),
  };
}

/**
 * @typedef {Object} Pct0Pct1Pct10Pct2Pct20Pct30Pct40Pct5Pct50Pct60Pct70Pct80Pct90Pct95Pct98Pct99Pattern
 * @property {PpmPriceRatioPattern} pct01
 * @property {PpmPriceRatioPattern} pct05
 * @property {PpmPriceRatioPattern} pct1
 * @property {PpmPriceRatioPattern} pct10
 * @property {PpmPriceRatioPattern} pct2
 * @property {PpmPriceRatioPattern} pct20
 * @property {PpmPriceRatioPattern} pct30
 * @property {PpmPriceRatioPattern} pct40
 * @property {PpmPriceRatioPattern} pct5
 * @property {PpmPriceRatioPattern} pct50
 * @property {PpmPriceRatioPattern} pct60
 * @property {PpmPriceRatioPattern} pct70
 * @property {PpmPriceRatioPattern} pct80
 * @property {PpmPriceRatioPattern} pct90
 * @property {PpmPriceRatioPattern} pct95
 * @property {PpmPriceRatioPattern} pct98
 * @property {PpmPriceRatioPattern} pct99
 * @property {PpmPriceRatioPattern} pct995
 * @property {PpmPriceRatioPattern} pct999
 */

/**
 * Create a Pct0Pct1Pct10Pct2Pct20Pct30Pct40Pct5Pct50Pct60Pct70Pct80Pct90Pct95Pct98Pct99Pattern pattern node
 * @param {BitviewClient} client
 * @param {string} acc - Accumulated series name
 * @returns {Pct0Pct1Pct10Pct2Pct20Pct30Pct40Pct5Pct50Pct60Pct70Pct80Pct90Pct95Pct98Pct99Pattern}
 */
function createPct0Pct1Pct10Pct2Pct20Pct30Pct40Pct5Pct50Pct60Pct70Pct80Pct90Pct95Pct98Pct99Pattern(client, acc) {
  return {
    pct01: createPpmPriceRatioPattern(client, acc, 'pct0_1'),
    pct05: createPpmPriceRatioPattern(client, acc, 'pct0_5'),
    pct1: createPpmPriceRatioPattern(client, acc, 'pct1'),
    pct10: createPpmPriceRatioPattern(client, acc, 'pct10'),
    pct2: createPpmPriceRatioPattern(client, acc, 'pct2'),
    pct20: createPpmPriceRatioPattern(client, acc, 'pct20'),
    pct30: createPpmPriceRatioPattern(client, acc, 'pct30'),
    pct40: createPpmPriceRatioPattern(client, acc, 'pct40'),
    pct5: createPpmPriceRatioPattern(client, acc, 'pct5'),
    pct50: createPpmPriceRatioPattern(client, acc, 'pct50'),
    pct60: createPpmPriceRatioPattern(client, acc, 'pct60'),
    pct70: createPpmPriceRatioPattern(client, acc, 'pct70'),
    pct80: createPpmPriceRatioPattern(client, acc, 'pct80'),
    pct90: createPpmPriceRatioPattern(client, acc, 'pct90'),
    pct95: createPpmPriceRatioPattern(client, acc, 'pct95'),
    pct98: createPpmPriceRatioPattern(client, acc, 'pct98'),
    pct99: createPpmPriceRatioPattern(client, acc, 'pct99'),
    pct995: createPpmPriceRatioPattern(client, acc, 'pct99_5'),
    pct999: createPpmPriceRatioPattern(client, acc, 'pct99_9'),
  };
}

/**
 * @typedef {Object} _200920102011201220132014201520162017201820192020202120222023202420252026Pattern2
 * @property {AverageBlockCumulativeSumPattern} _2009
 * @property {AverageBlockCumulativeSumPattern} _2010
 * @property {AverageBlockCumulativeSumPattern} _2011
 * @property {AverageBlockCumulativeSumPattern} _2012
 * @property {AverageBlockCumulativeSumPattern} _2013
 * @property {AverageBlockCumulativeSumPattern} _2014
 * @property {AverageBlockCumulativeSumPattern} _2015
 * @property {AverageBlockCumulativeSumPattern} _2016
 * @property {AverageBlockCumulativeSumPattern} _2017
 * @property {AverageBlockCumulativeSumPattern} _2018
 * @property {AverageBlockCumulativeSumPattern} _2019
 * @property {AverageBlockCumulativeSumPattern} _2020
 * @property {AverageBlockCumulativeSumPattern} _2021
 * @property {AverageBlockCumulativeSumPattern} _2022
 * @property {AverageBlockCumulativeSumPattern} _2023
 * @property {AverageBlockCumulativeSumPattern} _2024
 * @property {AverageBlockCumulativeSumPattern} _2025
 * @property {AverageBlockCumulativeSumPattern} _2026
 */

/**
 * Create a _200920102011201220132014201520162017201820192020202120222023202420252026Pattern2 pattern node
 * @param {BitviewClient} client
 * @param {string} acc - Accumulated series name
 * @param {string} disc - Discriminator suffix
 * @returns {_200920102011201220132014201520162017201820192020202120222023202420252026Pattern2}
 */
function create_200920102011201220132014201520162017201820192020202120222023202420252026Pattern2(client, acc, disc) {
  return {
    _2009: createAverageBlockCumulativeSumPattern(client, _m(_m(acc, '2009_transfer_volume'), disc)),
    _2010: createAverageBlockCumulativeSumPattern(client, _m(_m(acc, '2010_transfer_volume'), disc)),
    _2011: createAverageBlockCumulativeSumPattern(client, _m(_m(acc, '2011_transfer_volume'), disc)),
    _2012: createAverageBlockCumulativeSumPattern(client, _m(_m(acc, '2012_transfer_volume'), disc)),
    _2013: createAverageBlockCumulativeSumPattern(client, _m(_m(acc, '2013_transfer_volume'), disc)),
    _2014: createAverageBlockCumulativeSumPattern(client, _m(_m(acc, '2014_transfer_volume'), disc)),
    _2015: createAverageBlockCumulativeSumPattern(client, _m(_m(acc, '2015_transfer_volume'), disc)),
    _2016: createAverageBlockCumulativeSumPattern(client, _m(_m(acc, '2016_transfer_volume'), disc)),
    _2017: createAverageBlockCumulativeSumPattern(client, _m(_m(acc, '2017_transfer_volume'), disc)),
    _2018: createAverageBlockCumulativeSumPattern(client, _m(_m(acc, '2018_transfer_volume'), disc)),
    _2019: createAverageBlockCumulativeSumPattern(client, _m(_m(acc, '2019_transfer_volume'), disc)),
    _2020: createAverageBlockCumulativeSumPattern(client, _m(_m(acc, '2020_transfer_volume'), disc)),
    _2021: createAverageBlockCumulativeSumPattern(client, _m(_m(acc, '2021_transfer_volume'), disc)),
    _2022: createAverageBlockCumulativeSumPattern(client, _m(_m(acc, '2022_transfer_volume'), disc)),
    _2023: createAverageBlockCumulativeSumPattern(client, _m(_m(acc, '2023_transfer_volume'), disc)),
    _2024: createAverageBlockCumulativeSumPattern(client, _m(_m(acc, '2024_transfer_volume'), disc)),
    _2025: createAverageBlockCumulativeSumPattern(client, _m(_m(acc, '2025_transfer_volume'), disc)),
    _2026: createAverageBlockCumulativeSumPattern(client, _m(_m(acc, '2026_transfer_volume'), disc)),
  };
}

/**
 * @typedef {Object} _200920102011201220132014201520162017201820192020202120222023202420252026Pattern
 * @property {AverageBlockCumulativeSumPattern2<StoredF64>} _2009
 * @property {AverageBlockCumulativeSumPattern2<StoredF64>} _2010
 * @property {AverageBlockCumulativeSumPattern2<StoredF64>} _2011
 * @property {AverageBlockCumulativeSumPattern2<StoredF64>} _2012
 * @property {AverageBlockCumulativeSumPattern2<StoredF64>} _2013
 * @property {AverageBlockCumulativeSumPattern2<StoredF64>} _2014
 * @property {AverageBlockCumulativeSumPattern2<StoredF64>} _2015
 * @property {AverageBlockCumulativeSumPattern2<StoredF64>} _2016
 * @property {AverageBlockCumulativeSumPattern2<StoredF64>} _2017
 * @property {AverageBlockCumulativeSumPattern2<StoredF64>} _2018
 * @property {AverageBlockCumulativeSumPattern2<StoredF64>} _2019
 * @property {AverageBlockCumulativeSumPattern2<StoredF64>} _2020
 * @property {AverageBlockCumulativeSumPattern2<StoredF64>} _2021
 * @property {AverageBlockCumulativeSumPattern2<StoredF64>} _2022
 * @property {AverageBlockCumulativeSumPattern2<StoredF64>} _2023
 * @property {AverageBlockCumulativeSumPattern2<StoredF64>} _2024
 * @property {AverageBlockCumulativeSumPattern2<StoredF64>} _2025
 * @property {AverageBlockCumulativeSumPattern2<StoredF64>} _2026
 */

/**
 * @typedef {Object} _200920102011201220132014201520162017201820192020202120222023202420252026Pattern5
 * @property {BlockCumulativeSumPattern} _2009
 * @property {BlockCumulativeSumPattern} _2010
 * @property {BlockCumulativeSumPattern} _2011
 * @property {BlockCumulativeSumPattern} _2012
 * @property {BlockCumulativeSumPattern} _2013
 * @property {BlockCumulativeSumPattern} _2014
 * @property {BlockCumulativeSumPattern} _2015
 * @property {BlockCumulativeSumPattern} _2016
 * @property {BlockCumulativeSumPattern} _2017
 * @property {BlockCumulativeSumPattern} _2018
 * @property {BlockCumulativeSumPattern} _2019
 * @property {BlockCumulativeSumPattern} _2020
 * @property {BlockCumulativeSumPattern} _2021
 * @property {BlockCumulativeSumPattern} _2022
 * @property {BlockCumulativeSumPattern} _2023
 * @property {BlockCumulativeSumPattern} _2024
 * @property {BlockCumulativeSumPattern} _2025
 * @property {BlockCumulativeSumPattern} _2026
 */

/**
 * @typedef {Object} _200920102011201220132014201520162017201820192020202120222023202420252026Pattern10
 * @property {BtcCentsSatsUsdPattern} _2009
 * @property {BtcCentsSatsUsdPattern} _2010
 * @property {BtcCentsSatsUsdPattern} _2011
 * @property {BtcCentsSatsUsdPattern} _2012
 * @property {BtcCentsSatsUsdPattern} _2013
 * @property {BtcCentsSatsUsdPattern} _2014
 * @property {BtcCentsSatsUsdPattern} _2015
 * @property {BtcCentsSatsUsdPattern} _2016
 * @property {BtcCentsSatsUsdPattern} _2017
 * @property {BtcCentsSatsUsdPattern} _2018
 * @property {BtcCentsSatsUsdPattern} _2019
 * @property {BtcCentsSatsUsdPattern} _2020
 * @property {BtcCentsSatsUsdPattern} _2021
 * @property {BtcCentsSatsUsdPattern} _2022
 * @property {BtcCentsSatsUsdPattern} _2023
 * @property {BtcCentsSatsUsdPattern} _2024
 * @property {BtcCentsSatsUsdPattern} _2025
 * @property {BtcCentsSatsUsdPattern} _2026
 */

/**
 * @typedef {Object} _200920102011201220132014201520162017201820192020202120222023202420252026Pattern4
 * @property {CentsUsdPattern} _2009
 * @property {CentsUsdPattern} _2010
 * @property {CentsUsdPattern} _2011
 * @property {CentsUsdPattern} _2012
 * @property {CentsUsdPattern} _2013
 * @property {CentsUsdPattern} _2014
 * @property {CentsUsdPattern} _2015
 * @property {CentsUsdPattern} _2016
 * @property {CentsUsdPattern} _2017
 * @property {CentsUsdPattern} _2018
 * @property {CentsUsdPattern} _2019
 * @property {CentsUsdPattern} _2020
 * @property {CentsUsdPattern} _2021
 * @property {CentsUsdPattern} _2022
 * @property {CentsUsdPattern} _2023
 * @property {CentsUsdPattern} _2024
 * @property {CentsUsdPattern} _2025
 * @property {CentsUsdPattern} _2026
 */

/**
 * @typedef {Object} _0sats100btc100k100sats10btc10k10m10sats1btc1k1m1satOverPattern4
 * @property {AverageBlockCumulativeSumPattern} _0sats
 * @property {AverageBlockCumulativeSumPattern} _100btcTo1kBtc
 * @property {AverageBlockCumulativeSumPattern} _100kSatsTo1mSats
 * @property {AverageBlockCumulativeSumPattern} _100satsTo1kSats
 * @property {AverageBlockCumulativeSumPattern} _10btcTo100btc
 * @property {AverageBlockCumulativeSumPattern} _10kBtcTo100kBtc
 * @property {AverageBlockCumulativeSumPattern} _10kSatsTo100kSats
 * @property {AverageBlockCumulativeSumPattern} _10mSatsTo1btc
 * @property {AverageBlockCumulativeSumPattern} _10satsTo100sats
 * @property {AverageBlockCumulativeSumPattern} _1btcTo10btc
 * @property {AverageBlockCumulativeSumPattern} _1kBtcTo10kBtc
 * @property {AverageBlockCumulativeSumPattern} _1kSatsTo10kSats
 * @property {AverageBlockCumulativeSumPattern} _1mSatsTo10mSats
 * @property {AverageBlockCumulativeSumPattern} _1satTo10sats
 * @property {AverageBlockCumulativeSumPattern} over100kBtc
 */

/**
 * Create a _0sats100btc100k100sats10btc10k10m10sats1btc1k1m1satOverPattern4 pattern node
 * @param {BitviewClient} client
 * @param {string} acc - Accumulated series name
 * @returns {_0sats100btc100k100sats10btc10k10m10sats1btc1k1m1satOverPattern4}
 */
function create_0sats100btc100k100sats10btc10k10m10sats1btc1k1m1satOverPattern4(client, acc) {
  return {
    _0sats: createAverageBlockCumulativeSumPattern(client, _m(acc, '0sats_transfer_volume')),
    _100btcTo1kBtc: createAverageBlockCumulativeSumPattern(client, _m(acc, '100btc_to_1k_btc_transfer_volume')),
    _100kSatsTo1mSats: createAverageBlockCumulativeSumPattern(client, _m(acc, '100k_sats_to_1m_sats_transfer_volume')),
    _100satsTo1kSats: createAverageBlockCumulativeSumPattern(client, _m(acc, '100sats_to_1k_sats_transfer_volume')),
    _10btcTo100btc: createAverageBlockCumulativeSumPattern(client, _m(acc, '10btc_to_100btc_transfer_volume')),
    _10kBtcTo100kBtc: createAverageBlockCumulativeSumPattern(client, _m(acc, '10k_btc_to_100k_btc_transfer_volume')),
    _10kSatsTo100kSats: createAverageBlockCumulativeSumPattern(client, _m(acc, '10k_sats_to_100k_sats_transfer_volume')),
    _10mSatsTo1btc: createAverageBlockCumulativeSumPattern(client, _m(acc, '10m_sats_to_1btc_transfer_volume')),
    _10satsTo100sats: createAverageBlockCumulativeSumPattern(client, _m(acc, '10sats_to_100sats_transfer_volume')),
    _1btcTo10btc: createAverageBlockCumulativeSumPattern(client, _m(acc, '1btc_to_10btc_transfer_volume')),
    _1kBtcTo10kBtc: createAverageBlockCumulativeSumPattern(client, _m(acc, '1k_btc_to_10k_btc_transfer_volume')),
    _1kSatsTo10kSats: createAverageBlockCumulativeSumPattern(client, _m(acc, '1k_sats_to_10k_sats_transfer_volume')),
    _1mSatsTo10mSats: createAverageBlockCumulativeSumPattern(client, _m(acc, '1m_sats_to_10m_sats_transfer_volume')),
    _1satTo10sats: createAverageBlockCumulativeSumPattern(client, _m(acc, '1sat_to_10sats_transfer_volume')),
    over100kBtc: createAverageBlockCumulativeSumPattern(client, _m(acc, 'over_100k_btc_transfer_volume')),
  };
}

/**
 * @typedef {Object} _0sats100btc100k100sats10btc10k10m10sats1btc1k1m1satOverPattern5
 * @property {BaseDeltaPattern} _0sats
 * @property {BaseDeltaPattern} _100btcTo1kBtc
 * @property {BaseDeltaPattern} _100kSatsTo1mSats
 * @property {BaseDeltaPattern} _100satsTo1kSats
 * @property {BaseDeltaPattern} _10btcTo100btc
 * @property {BaseDeltaPattern} _10kBtcTo100kBtc
 * @property {BaseDeltaPattern} _10kSatsTo100kSats
 * @property {BaseDeltaPattern} _10mSatsTo1btc
 * @property {BaseDeltaPattern} _10satsTo100sats
 * @property {BaseDeltaPattern} _1btcTo10btc
 * @property {BaseDeltaPattern} _1kBtcTo10kBtc
 * @property {BaseDeltaPattern} _1kSatsTo10kSats
 * @property {BaseDeltaPattern} _1mSatsTo10mSats
 * @property {BaseDeltaPattern} _1satTo10sats
 * @property {BaseDeltaPattern} over100kBtc
 */

/**
 * @typedef {Object} _0sats100btc100k100sats10btc10k10m10sats1btc1k1m1satOverPattern2
 * @property {BlockCumulativeSumPattern} _0sats
 * @property {BlockCumulativeSumPattern} _100btcTo1kBtc
 * @property {BlockCumulativeSumPattern} _100kSatsTo1mSats
 * @property {BlockCumulativeSumPattern} _100satsTo1kSats
 * @property {BlockCumulativeSumPattern} _10btcTo100btc
 * @property {BlockCumulativeSumPattern} _10kBtcTo100kBtc
 * @property {BlockCumulativeSumPattern} _10kSatsTo100kSats
 * @property {BlockCumulativeSumPattern} _10mSatsTo1btc
 * @property {BlockCumulativeSumPattern} _10satsTo100sats
 * @property {BlockCumulativeSumPattern} _1btcTo10btc
 * @property {BlockCumulativeSumPattern} _1kBtcTo10kBtc
 * @property {BlockCumulativeSumPattern} _1kSatsTo10kSats
 * @property {BlockCumulativeSumPattern} _1mSatsTo10mSats
 * @property {BlockCumulativeSumPattern} _1satTo10sats
 * @property {BlockCumulativeSumPattern} over100kBtc
 */

/**
 * @typedef {Object} _0sats100btc100k100sats10btc10k10m10sats1btc1k1m1satOverPattern
 * @property {CentsUsdPattern} _0sats
 * @property {CentsUsdPattern} _100btcTo1kBtc
 * @property {CentsUsdPattern} _100kSatsTo1mSats
 * @property {CentsUsdPattern} _100satsTo1kSats
 * @property {CentsUsdPattern} _10btcTo100btc
 * @property {CentsUsdPattern} _10kBtcTo100kBtc
 * @property {CentsUsdPattern} _10kSatsTo100kSats
 * @property {CentsUsdPattern} _10mSatsTo1btc
 * @property {CentsUsdPattern} _10satsTo100sats
 * @property {CentsUsdPattern} _1btcTo10btc
 * @property {CentsUsdPattern} _1kBtcTo10kBtc
 * @property {CentsUsdPattern} _1kSatsTo10kSats
 * @property {CentsUsdPattern} _1mSatsTo10mSats
 * @property {CentsUsdPattern} _1satTo10sats
 * @property {CentsUsdPattern} over100kBtc
 */

/**
 * Create a _0sats100btc100k100sats10btc10k10m10sats1btc1k1m1satOverPattern pattern node
 * @param {BitviewClient} client
 * @param {string} acc - Accumulated series name
 * @returns {_0sats100btc100k100sats10btc10k10m10sats1btc1k1m1satOverPattern}
 */
function create_0sats100btc100k100sats10btc10k10m10sats1btc1k1m1satOverPattern(client, acc) {
  return {
    _0sats: createCentsUsdPattern(client, _m(acc, '0sats_realized_cap')),
    _100btcTo1kBtc: createCentsUsdPattern(client, _m(acc, '100btc_to_1k_btc_realized_cap')),
    _100kSatsTo1mSats: createCentsUsdPattern(client, _m(acc, '100k_sats_to_1m_sats_realized_cap')),
    _100satsTo1kSats: createCentsUsdPattern(client, _m(acc, '100sats_to_1k_sats_realized_cap')),
    _10btcTo100btc: createCentsUsdPattern(client, _m(acc, '10btc_to_100btc_realized_cap')),
    _10kBtcTo100kBtc: createCentsUsdPattern(client, _m(acc, '10k_btc_to_100k_btc_realized_cap')),
    _10kSatsTo100kSats: createCentsUsdPattern(client, _m(acc, '10k_sats_to_100k_sats_realized_cap')),
    _10mSatsTo1btc: createCentsUsdPattern(client, _m(acc, '10m_sats_to_1btc_realized_cap')),
    _10satsTo100sats: createCentsUsdPattern(client, _m(acc, '10sats_to_100sats_realized_cap')),
    _1btcTo10btc: createCentsUsdPattern(client, _m(acc, '1btc_to_10btc_realized_cap')),
    _1kBtcTo10kBtc: createCentsUsdPattern(client, _m(acc, '1k_btc_to_10k_btc_realized_cap')),
    _1kSatsTo10kSats: createCentsUsdPattern(client, _m(acc, '1k_sats_to_10k_sats_realized_cap')),
    _1mSatsTo10mSats: createCentsUsdPattern(client, _m(acc, '1m_sats_to_10m_sats_realized_cap')),
    _1satTo10sats: createCentsUsdPattern(client, _m(acc, '1sat_to_10sats_realized_cap')),
    over100kBtc: createCentsUsdPattern(client, _m(acc, 'over_100k_btc_realized_cap')),
  };
}

/**
 * @typedef {Object} InvestedNetRealizedSupplyUnrealizedPattern
 * @property {PercentPpmRatioPattern2} investedCapitalInLossShare
 * @property {PercentPpmRatioPattern2} investedCapitalInProfitShare
 * @property {PercentPpmRatioPattern} netPnlChange1mToMcap
 * @property {PercentPpmRatioPattern} netPnlChange1mToRcap
 * @property {PercentPpmRatioPattern3} netUnrealizedPnlToOwnGrossPnl
 * @property {PercentPpmRatioPattern2} realizedCapToOwnMcap
 * @property {PercentPpmRatioPattern2} supplyDominance
 * @property {PercentPpmRatioPattern2} supplyInLossShare
 * @property {PercentPpmRatioPattern2} supplyInProfitShare
 * @property {PercentPpmRatioPattern2} unrealizedLossToMcap
 * @property {PercentPpmRatioPattern2} unrealizedLossToOwnGrossPnl
 * @property {PercentPpmRatioPattern2} unrealizedLossToOwnMcap
 * @property {PercentPpmRatioPattern2} unrealizedProfitToMcap
 * @property {PercentPpmRatioPattern2} unrealizedProfitToOwnGrossPnl
 * @property {PercentPpmRatioPattern2} unrealizedProfitToOwnMcap
 */

/**
 * Create a InvestedNetRealizedSupplyUnrealizedPattern pattern node
 * @param {BitviewClient} client
 * @param {string} acc - Accumulated series name
 * @returns {InvestedNetRealizedSupplyUnrealizedPattern}
 */
function createInvestedNetRealizedSupplyUnrealizedPattern(client, acc) {
  return {
    investedCapitalInLossShare: createPercentPpmRatioPattern2(client, _m(acc, 'invested_capital_in_loss_share')),
    investedCapitalInProfitShare: createPercentPpmRatioPattern2(client, _m(acc, 'invested_capital_in_profit_share')),
    netPnlChange1mToMcap: createPercentPpmRatioPattern(client, _m(acc, 'net_pnl_change_1m_to_mcap')),
    netPnlChange1mToRcap: createPercentPpmRatioPattern(client, _m(acc, 'net_pnl_change_1m_to_rcap')),
    netUnrealizedPnlToOwnGrossPnl: createPercentPpmRatioPattern3(client, _m(acc, 'net_unrealized_pnl_to_own_gross_pnl')),
    realizedCapToOwnMcap: createPercentPpmRatioPattern2(client, _m(acc, 'realized_cap_to_own_mcap')),
    supplyDominance: createPercentPpmRatioPattern2(client, _m(acc, 'supply_dominance')),
    supplyInLossShare: createPercentPpmRatioPattern2(client, _m(acc, 'supply_in_loss_share')),
    supplyInProfitShare: createPercentPpmRatioPattern2(client, _m(acc, 'supply_in_profit_share')),
    unrealizedLossToMcap: createPercentPpmRatioPattern2(client, _m(acc, 'unrealized_loss_to_mcap')),
    unrealizedLossToOwnGrossPnl: createPercentPpmRatioPattern2(client, _m(acc, 'unrealized_loss_to_own_gross_pnl')),
    unrealizedLossToOwnMcap: createPercentPpmRatioPattern2(client, _m(acc, 'unrealized_loss_to_own_mcap')),
    unrealizedProfitToMcap: createPercentPpmRatioPattern2(client, _m(acc, 'unrealized_profit_to_mcap')),
    unrealizedProfitToOwnGrossPnl: createPercentPpmRatioPattern2(client, _m(acc, 'unrealized_profit_to_own_gross_pnl')),
    unrealizedProfitToOwnMcap: createPercentPpmRatioPattern2(client, _m(acc, 'unrealized_profit_to_own_mcap')),
  };
}

/**
 * @typedef {Object} AllEmptyOpP2aP2msP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshUnknownPattern
 * @property {AverageBlockCumulativeSumPattern2<StoredU64>} all
 * @property {AverageBlockCumulativeSumPattern2<StoredU64>} empty
 * @property {AverageBlockCumulativeSumPattern2<StoredU64>} opReturn
 * @property {AverageBlockCumulativeSumPattern2<StoredU64>} p2a
 * @property {AverageBlockCumulativeSumPattern2<StoredU64>} p2ms
 * @property {AverageBlockCumulativeSumPattern2<StoredU64>} p2pk33
 * @property {AverageBlockCumulativeSumPattern2<StoredU64>} p2pk65
 * @property {AverageBlockCumulativeSumPattern2<StoredU64>} p2pkh
 * @property {AverageBlockCumulativeSumPattern2<StoredU64>} p2sh
 * @property {AverageBlockCumulativeSumPattern2<StoredU64>} p2tr
 * @property {AverageBlockCumulativeSumPattern2<StoredU64>} p2wpkh
 * @property {AverageBlockCumulativeSumPattern2<StoredU64>} p2wsh
 * @property {AverageBlockCumulativeSumPattern2<StoredU64>} unknown
 */

/**
 * Create a AllEmptyOpP2aP2msP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshUnknownPattern pattern node
 * @param {BitviewClient} client
 * @param {string} acc - Accumulated series name
 * @returns {AllEmptyOpP2aP2msP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshUnknownPattern}
 */
function createAllEmptyOpP2aP2msP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshUnknownPattern(client, acc) {
  return {
    all: createAverageBlockCumulativeSumPattern2(client, _m(acc, 'bis')),
    empty: createAverageBlockCumulativeSumPattern2(client, _m(acc, 'with_empty_outputs_output')),
    opReturn: createAverageBlockCumulativeSumPattern2(client, _m(acc, 'with_op_return_output')),
    p2a: createAverageBlockCumulativeSumPattern2(client, _m(acc, 'with_p2a_output')),
    p2ms: createAverageBlockCumulativeSumPattern2(client, _m(acc, 'with_p2ms_output')),
    p2pk33: createAverageBlockCumulativeSumPattern2(client, _m(acc, 'with_p2pk33_output')),
    p2pk65: createAverageBlockCumulativeSumPattern2(client, _m(acc, 'with_p2pk65_output')),
    p2pkh: createAverageBlockCumulativeSumPattern2(client, _m(acc, 'with_p2pkh_output')),
    p2sh: createAverageBlockCumulativeSumPattern2(client, _m(acc, 'with_p2sh_output')),
    p2tr: createAverageBlockCumulativeSumPattern2(client, _m(acc, 'with_p2tr_output')),
    p2wpkh: createAverageBlockCumulativeSumPattern2(client, _m(acc, 'with_p2wpkh_output')),
    p2wsh: createAverageBlockCumulativeSumPattern2(client, _m(acc, 'with_p2wsh_output')),
    unknown: createAverageBlockCumulativeSumPattern2(client, _m(acc, 'with_unknown_outputs_output')),
  };
}

/**
 * @typedef {Object} AllEmptyP2aP2msP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshUnknownPattern
 * @property {AverageBlockCumulativeSumPattern2<StoredU64>} all
 * @property {AverageBlockCumulativeSumPattern2<StoredU64>} empty
 * @property {AverageBlockCumulativeSumPattern2<StoredU64>} p2a
 * @property {AverageBlockCumulativeSumPattern2<StoredU64>} p2ms
 * @property {AverageBlockCumulativeSumPattern2<StoredU64>} p2pk33
 * @property {AverageBlockCumulativeSumPattern2<StoredU64>} p2pk65
 * @property {AverageBlockCumulativeSumPattern2<StoredU64>} p2pkh
 * @property {AverageBlockCumulativeSumPattern2<StoredU64>} p2sh
 * @property {AverageBlockCumulativeSumPattern2<StoredU64>} p2tr
 * @property {AverageBlockCumulativeSumPattern2<StoredU64>} p2wpkh
 * @property {AverageBlockCumulativeSumPattern2<StoredU64>} p2wsh
 * @property {AverageBlockCumulativeSumPattern2<StoredU64>} unknown
 */

/**
 * @typedef {Object} EmptyOpP2aP2msP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshUnknownPattern2
 * @property {_1m1w1y24hPercentPpmRatioPattern} empty
 * @property {_1m1w1y24hPercentPpmRatioPattern} opReturn
 * @property {_1m1w1y24hPercentPpmRatioPattern} p2a
 * @property {_1m1w1y24hPercentPpmRatioPattern} p2ms
 * @property {_1m1w1y24hPercentPpmRatioPattern} p2pk33
 * @property {_1m1w1y24hPercentPpmRatioPattern} p2pk65
 * @property {_1m1w1y24hPercentPpmRatioPattern} p2pkh
 * @property {_1m1w1y24hPercentPpmRatioPattern} p2sh
 * @property {_1m1w1y24hPercentPpmRatioPattern} p2tr
 * @property {_1m1w1y24hPercentPpmRatioPattern} p2wpkh
 * @property {_1m1w1y24hPercentPpmRatioPattern} p2wsh
 * @property {_1m1w1y24hPercentPpmRatioPattern} unknown
 */

/**
 * Create a EmptyOpP2aP2msP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshUnknownPattern2 pattern node
 * @param {BitviewClient} client
 * @param {string} acc - Accumulated series name
 * @returns {EmptyOpP2aP2msP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshUnknownPattern2}
 */
function createEmptyOpP2aP2msP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshUnknownPattern2(client, acc) {
  return {
    empty: create_1m1w1y24hPercentPpmRatioPattern(client, _m(acc, 'empty_outputs_output')),
    opReturn: create_1m1w1y24hPercentPpmRatioPattern(client, _m(acc, 'op_return_output')),
    p2a: create_1m1w1y24hPercentPpmRatioPattern(client, _m(acc, 'p2a_output')),
    p2ms: create_1m1w1y24hPercentPpmRatioPattern(client, _m(acc, 'p2ms_output')),
    p2pk33: create_1m1w1y24hPercentPpmRatioPattern(client, _m(acc, 'p2pk33_output')),
    p2pk65: create_1m1w1y24hPercentPpmRatioPattern(client, _m(acc, 'p2pk65_output')),
    p2pkh: create_1m1w1y24hPercentPpmRatioPattern(client, _m(acc, 'p2pkh_output')),
    p2sh: create_1m1w1y24hPercentPpmRatioPattern(client, _m(acc, 'p2sh_output')),
    p2tr: create_1m1w1y24hPercentPpmRatioPattern(client, _m(acc, 'p2tr_output')),
    p2wpkh: create_1m1w1y24hPercentPpmRatioPattern(client, _m(acc, 'p2wpkh_output')),
    p2wsh: create_1m1w1y24hPercentPpmRatioPattern(client, _m(acc, 'p2wsh_output')),
    unknown: create_1m1w1y24hPercentPpmRatioPattern(client, _m(acc, 'unknown_outputs_output')),
  };
}

/**
 * @typedef {Object} AverageBlockCumulativeMaxMedianMinPct10Pct25Pct75Pct90SumPattern
 * @property {_1m1w1y24hPattern6<StoredF32>} average
 * @property {SeriesPattern18<StoredU64>} block
 * @property {SeriesPattern1<StoredU64>} cumulative
 * @property {_1m1w1y24hPattern6<StoredU64>} max
 * @property {_1m1w1y24hPattern6<StoredU64>} median
 * @property {_1m1w1y24hPattern6<StoredU64>} min
 * @property {_1m1w1y24hPattern6<StoredU64>} pct10
 * @property {_1m1w1y24hPattern6<StoredU64>} pct25
 * @property {_1m1w1y24hPattern6<StoredU64>} pct75
 * @property {_1m1w1y24hPattern6<StoredU64>} pct90
 * @property {_1m1w1y24hPattern6<StoredU64>} sum
 */

/**
 * Create a AverageBlockCumulativeMaxMedianMinPct10Pct25Pct75Pct90SumPattern pattern node
 * @param {BitviewClient} client
 * @param {string} acc - Accumulated series name
 * @returns {AverageBlockCumulativeMaxMedianMinPct10Pct25Pct75Pct90SumPattern}
 */
function createAverageBlockCumulativeMaxMedianMinPct10Pct25Pct75Pct90SumPattern(client, acc) {
  return {
    average: create_1m1w1y24hPattern6(client, _m(acc, 'average')),
    block: createSeriesPattern18(client, acc),
    cumulative: createSeriesPattern1(client, _m(acc, 'cumulative')),
    max: create_1m1w1y24hPattern6(client, _m(acc, 'max')),
    median: create_1m1w1y24hPattern6(client, _m(acc, 'median')),
    min: create_1m1w1y24hPattern6(client, _m(acc, 'min')),
    pct10: create_1m1w1y24hPattern6(client, _m(acc, 'pct10')),
    pct25: create_1m1w1y24hPattern6(client, _m(acc, 'pct25')),
    pct75: create_1m1w1y24hPattern6(client, _m(acc, 'pct75')),
    pct90: create_1m1w1y24hPattern6(client, _m(acc, 'pct90')),
    sum: create_1m1w1y24hPattern6(client, _m(acc, 'sum')),
  };
}

/**
 * @typedef {Object} EmptyP2aP2msP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshUnknownPattern6
 * @property {BlockCumulativeSumPattern} empty
 * @property {BlockCumulativeSumPattern} p2a
 * @property {BlockCumulativeSumPattern} p2ms
 * @property {BlockCumulativeSumPattern} p2pk33
 * @property {BlockCumulativeSumPattern} p2pk65
 * @property {BlockCumulativeSumPattern} p2pkh
 * @property {BlockCumulativeSumPattern} p2sh
 * @property {BlockCumulativeSumPattern} p2tr
 * @property {BlockCumulativeSumPattern} p2wpkh
 * @property {BlockCumulativeSumPattern} p2wsh
 * @property {BlockCumulativeSumPattern} unknown
 */

/**
 * Create a EmptyP2aP2msP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshUnknownPattern6 pattern node
 * @param {BitviewClient} client
 * @param {string} acc - Accumulated series name
 * @returns {EmptyP2aP2msP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshUnknownPattern6}
 */
function createEmptyP2aP2msP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshUnknownPattern6(client, acc) {
  return {
    empty: createBlockCumulativeSumPattern(client, _p('empty_outputs', acc)),
    p2a: createBlockCumulativeSumPattern(client, _p('p2a', acc)),
    p2ms: createBlockCumulativeSumPattern(client, _p('p2ms', acc)),
    p2pk33: createBlockCumulativeSumPattern(client, _p('p2pk33', acc)),
    p2pk65: createBlockCumulativeSumPattern(client, _p('p2pk65', acc)),
    p2pkh: createBlockCumulativeSumPattern(client, _p('p2pkh', acc)),
    p2sh: createBlockCumulativeSumPattern(client, _p('p2sh', acc)),
    p2tr: createBlockCumulativeSumPattern(client, _p('p2tr', acc)),
    p2wpkh: createBlockCumulativeSumPattern(client, _p('p2wpkh', acc)),
    p2wsh: createBlockCumulativeSumPattern(client, _p('p2wsh', acc)),
    unknown: createBlockCumulativeSumPattern(client, _p('unknown_outputs', acc)),
  };
}

/**
 * @typedef {Object} EmptyP2aP2msP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshUnknownPattern2
 * @property {BtcCentsSatsUsdPattern} empty
 * @property {BtcCentsSatsUsdPattern} p2a
 * @property {BtcCentsSatsUsdPattern} p2ms
 * @property {BtcCentsSatsUsdPattern} p2pk33
 * @property {BtcCentsSatsUsdPattern} p2pk65
 * @property {BtcCentsSatsUsdPattern} p2pkh
 * @property {BtcCentsSatsUsdPattern} p2sh
 * @property {BtcCentsSatsUsdPattern} p2tr
 * @property {BtcCentsSatsUsdPattern} p2wpkh
 * @property {BtcCentsSatsUsdPattern} p2wsh
 * @property {BtcCentsSatsUsdPattern} unknown
 */

/**
 * Create a EmptyP2aP2msP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshUnknownPattern2 pattern node
 * @param {BitviewClient} client
 * @param {string} acc - Accumulated series name
 * @returns {EmptyP2aP2msP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshUnknownPattern2}
 */
function createEmptyP2aP2msP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshUnknownPattern2(client, acc) {
  return {
    empty: createBtcCentsSatsUsdPattern(client, _p('empty_outputs', acc)),
    p2a: createBtcCentsSatsUsdPattern(client, _p('p2a', acc)),
    p2ms: createBtcCentsSatsUsdPattern(client, _p('p2ms', acc)),
    p2pk33: createBtcCentsSatsUsdPattern(client, _p('p2pk33', acc)),
    p2pk65: createBtcCentsSatsUsdPattern(client, _p('p2pk65', acc)),
    p2pkh: createBtcCentsSatsUsdPattern(client, _p('p2pkh', acc)),
    p2sh: createBtcCentsSatsUsdPattern(client, _p('p2sh', acc)),
    p2tr: createBtcCentsSatsUsdPattern(client, _p('p2tr', acc)),
    p2wpkh: createBtcCentsSatsUsdPattern(client, _p('p2wpkh', acc)),
    p2wsh: createBtcCentsSatsUsdPattern(client, _p('p2wsh', acc)),
    unknown: createBtcCentsSatsUsdPattern(client, _p('unknown_outputs', acc)),
  };
}

/**
 * @typedef {Object} EmptyP2aP2msP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshUnknownPattern10
 * @property {_1m1w1y24hPercentPpmRatioPattern} empty
 * @property {_1m1w1y24hPercentPpmRatioPattern} p2a
 * @property {_1m1w1y24hPercentPpmRatioPattern} p2ms
 * @property {_1m1w1y24hPercentPpmRatioPattern} p2pk33
 * @property {_1m1w1y24hPercentPpmRatioPattern} p2pk65
 * @property {_1m1w1y24hPercentPpmRatioPattern} p2pkh
 * @property {_1m1w1y24hPercentPpmRatioPattern} p2sh
 * @property {_1m1w1y24hPercentPpmRatioPattern} p2tr
 * @property {_1m1w1y24hPercentPpmRatioPattern} p2wpkh
 * @property {_1m1w1y24hPercentPpmRatioPattern} p2wsh
 * @property {_1m1w1y24hPercentPpmRatioPattern} unknown
 */

/**
 * Create a EmptyP2aP2msP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshUnknownPattern10 pattern node
 * @param {BitviewClient} client
 * @param {string} acc - Accumulated series name
 * @returns {EmptyP2aP2msP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshUnknownPattern10}
 */
function createEmptyP2aP2msP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshUnknownPattern10(client, acc) {
  return {
    empty: create_1m1w1y24hPercentPpmRatioPattern(client, _m(acc, 'empty_outputs_prevout')),
    p2a: create_1m1w1y24hPercentPpmRatioPattern(client, _m(acc, 'p2a_prevout')),
    p2ms: create_1m1w1y24hPercentPpmRatioPattern(client, _m(acc, 'p2ms_prevout')),
    p2pk33: create_1m1w1y24hPercentPpmRatioPattern(client, _m(acc, 'p2pk33_prevout')),
    p2pk65: create_1m1w1y24hPercentPpmRatioPattern(client, _m(acc, 'p2pk65_prevout')),
    p2pkh: create_1m1w1y24hPercentPpmRatioPattern(client, _m(acc, 'p2pkh_prevout')),
    p2sh: create_1m1w1y24hPercentPpmRatioPattern(client, _m(acc, 'p2sh_prevout')),
    p2tr: create_1m1w1y24hPercentPpmRatioPattern(client, _m(acc, 'p2tr_prevout')),
    p2wpkh: create_1m1w1y24hPercentPpmRatioPattern(client, _m(acc, 'p2wpkh_prevout')),
    p2wsh: create_1m1w1y24hPercentPpmRatioPattern(client, _m(acc, 'p2wsh_prevout')),
    unknown: create_1m1w1y24hPercentPpmRatioPattern(client, _m(acc, 'unknown_outputs_prevout')),
  };
}

/**
 * @typedef {Object} AllP2aP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshSharePattern
 * @property {BtcCentsSatsUsdPattern} all
 * @property {BtcCentsSatsUsdPattern} p2a
 * @property {BtcCentsSatsUsdPattern} p2pk33
 * @property {BtcCentsSatsUsdPattern} p2pk65
 * @property {BtcCentsSatsUsdPattern} p2pkh
 * @property {BtcCentsSatsUsdPattern} p2sh
 * @property {BtcCentsSatsUsdPattern} p2tr
 * @property {BtcCentsSatsUsdPattern} p2wpkh
 * @property {BtcCentsSatsUsdPattern} p2wsh
 * @property {AllP2aP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshPattern5} share
 */

/**
 * @typedef {Object} CapCapitalizedGrossLossMvrvNetPeakPriceProfitValuePattern
 * @property {CentsDeltaUsdPattern} cap
 * @property {CentsPpmRatioSatsUsdPattern} capitalizedPrice
 * @property {BlockCumulativeSumPattern} grossPnl
 * @property {BlockCumulativeSumPattern} loss
 * @property {PpmRatioPattern2} mvrv
 * @property {BlockCumulativeDeltaSumPattern} netPnl
 * @property {BlockCumulativeSumPattern} peakRegret
 * @property {CentsSatsUsdPattern} price
 * @property {BlockCumulativeSumPattern} profit
 * @property {AverageBlockCumulativeSumPattern3} valueDestroyed
 */

/**
 * Create a CapCapitalizedGrossLossMvrvNetPeakPriceProfitValuePattern pattern node
 * @param {BitviewClient} client
 * @param {string} acc - Accumulated series name
 * @returns {CapCapitalizedGrossLossMvrvNetPeakPriceProfitValuePattern}
 */
function createCapCapitalizedGrossLossMvrvNetPeakPriceProfitValuePattern(client, acc) {
  return {
    cap: createCentsDeltaUsdPattern(client, _m(acc, 'realized_cap')),
    capitalizedPrice: createCentsPpmRatioSatsUsdPattern(client, _m(acc, 'capitalized_price')),
    grossPnl: createBlockCumulativeSumPattern(client, _m(acc, 'realized_gross_pnl')),
    loss: createBlockCumulativeSumPattern(client, _m(acc, 'realized_loss')),
    mvrv: createPpmRatioPattern2(client, _m(acc, 'mvrv')),
    netPnl: createBlockCumulativeDeltaSumPattern(client, _m(acc, 'net_realized_pnl')),
    peakRegret: createBlockCumulativeSumPattern(client, _m(acc, 'realized_peak_regret')),
    price: createCentsSatsUsdPattern(client, _m(acc, 'realized_price')),
    profit: createBlockCumulativeSumPattern(client, _m(acc, 'realized_profit')),
    valueDestroyed: createAverageBlockCumulativeSumPattern3(client, _m(acc, 'value_destroyed')),
  };
}

/**
 * @typedef {Object} GreedGrossInvestedLossNetNuplPainProfitPattern
 * @property {CentsUsdPattern} greedIndex
 * @property {CentsUsdPattern} grossPnl
 * @property {CentsUsdPattern} investedCapitalInLoss
 * @property {CentsUsdPattern} investedCapitalInProfit
 * @property {CentsUsdPattern} loss
 * @property {CentsUsdPattern4} netPnl
 * @property {CentsUsdPattern4} netSentiment
 * @property {PpmRatioPattern3} nupl
 * @property {CentsUsdPattern} painIndex
 * @property {CentsUsdPattern} profit
 */

/**
 * Create a GreedGrossInvestedLossNetNuplPainProfitPattern pattern node
 * @param {BitviewClient} client
 * @param {string} acc - Accumulated series name
 * @returns {GreedGrossInvestedLossNetNuplPainProfitPattern}
 */
function createGreedGrossInvestedLossNetNuplPainProfitPattern(client, acc) {
  return {
    greedIndex: createCentsUsdPattern(client, _m(acc, 'greed_index')),
    grossPnl: createCentsUsdPattern(client, _m(acc, 'unrealized_gross_pnl')),
    investedCapitalInLoss: createCentsUsdPattern(client, _m(acc, 'invested_capital_in_loss')),
    investedCapitalInProfit: createCentsUsdPattern(client, _m(acc, 'invested_capital_in_profit')),
    loss: createCentsUsdPattern(client, _m(acc, 'unrealized_loss')),
    netPnl: createCentsUsdPattern4(client, _m(acc, 'net_unrealized_pnl')),
    netSentiment: createCentsUsdPattern4(client, _m(acc, 'net_sentiment')),
    nupl: createPpmRatioPattern3(client, _m(acc, 'nupl')),
    painIndex: createCentsUsdPattern(client, _m(acc, 'pain_index')),
    profit: createCentsUsdPattern(client, _m(acc, 'unrealized_profit')),
  };
}

/**
 * @typedef {Object} AllP2aP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshPattern6
 * @property {AverageBlockCumulativeSumPattern2<StoredU64>} all
 * @property {AverageBlockCumulativeSumPattern2<StoredU64>} p2a
 * @property {AverageBlockCumulativeSumPattern2<StoredU64>} p2pk33
 * @property {AverageBlockCumulativeSumPattern2<StoredU64>} p2pk65
 * @property {AverageBlockCumulativeSumPattern2<StoredU64>} p2pkh
 * @property {AverageBlockCumulativeSumPattern2<StoredU64>} p2sh
 * @property {AverageBlockCumulativeSumPattern2<StoredU64>} p2tr
 * @property {AverageBlockCumulativeSumPattern2<StoredU64>} p2wpkh
 * @property {AverageBlockCumulativeSumPattern2<StoredU64>} p2wsh
 */

/**
 * Create a AllP2aP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshPattern6 pattern node
 * @param {BitviewClient} client
 * @param {string} acc - Accumulated series name
 * @returns {AllP2aP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshPattern6}
 */
function createAllP2aP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshPattern6(client, acc) {
  return {
    all: createAverageBlockCumulativeSumPattern2(client, acc),
    p2a: createAverageBlockCumulativeSumPattern2(client, _p('p2a', acc)),
    p2pk33: createAverageBlockCumulativeSumPattern2(client, _p('p2pk33', acc)),
    p2pk65: createAverageBlockCumulativeSumPattern2(client, _p('p2pk65', acc)),
    p2pkh: createAverageBlockCumulativeSumPattern2(client, _p('p2pkh', acc)),
    p2sh: createAverageBlockCumulativeSumPattern2(client, _p('p2sh', acc)),
    p2tr: createAverageBlockCumulativeSumPattern2(client, _p('p2tr', acc)),
    p2wpkh: createAverageBlockCumulativeSumPattern2(client, _p('p2wpkh', acc)),
    p2wsh: createAverageBlockCumulativeSumPattern2(client, _p('p2wsh', acc)),
  };
}

/**
 * @typedef {Object} AllP2aP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshPattern5
 * @property {PercentPpmRatioPattern2} all
 * @property {PercentPpmRatioPattern2} p2a
 * @property {PercentPpmRatioPattern2} p2pk33
 * @property {PercentPpmRatioPattern2} p2pk65
 * @property {PercentPpmRatioPattern2} p2pkh
 * @property {PercentPpmRatioPattern2} p2sh
 * @property {PercentPpmRatioPattern2} p2tr
 * @property {PercentPpmRatioPattern2} p2wpkh
 * @property {PercentPpmRatioPattern2} p2wsh
 */

/**
 * Create a AllP2aP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshPattern5 pattern node
 * @param {BitviewClient} client
 * @param {string} acc - Accumulated series name
 * @returns {AllP2aP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshPattern5}
 */
function createAllP2aP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshPattern5(client, acc) {
  return {
    all: createPercentPpmRatioPattern2(client, acc),
    p2a: createPercentPpmRatioPattern2(client, _p('p2a', acc)),
    p2pk33: createPercentPpmRatioPattern2(client, _p('p2pk33', acc)),
    p2pk65: createPercentPpmRatioPattern2(client, _p('p2pk65', acc)),
    p2pkh: createPercentPpmRatioPattern2(client, _p('p2pkh', acc)),
    p2sh: createPercentPpmRatioPattern2(client, _p('p2sh', acc)),
    p2tr: createPercentPpmRatioPattern2(client, _p('p2tr', acc)),
    p2wpkh: createPercentPpmRatioPattern2(client, _p('p2wpkh', acc)),
    p2wsh: createPercentPpmRatioPattern2(client, _p('p2wsh', acc)),
  };
}

/**
 * @typedef {Object} AllP2aP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshPattern4
 * @property {SeriesPattern1<StoredU64>} all
 * @property {SeriesPattern1<StoredU64>} p2a
 * @property {SeriesPattern1<StoredU64>} p2pk33
 * @property {SeriesPattern1<StoredU64>} p2pk65
 * @property {SeriesPattern1<StoredU64>} p2pkh
 * @property {SeriesPattern1<StoredU64>} p2sh
 * @property {SeriesPattern1<StoredU64>} p2tr
 * @property {SeriesPattern1<StoredU64>} p2wpkh
 * @property {SeriesPattern1<StoredU64>} p2wsh
 */

/**
 * Create a AllP2aP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshPattern4 pattern node
 * @param {BitviewClient} client
 * @param {string} acc - Accumulated series name
 * @returns {AllP2aP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshPattern4}
 */
function createAllP2aP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshPattern4(client, acc) {
  return {
    all: createSeriesPattern1(client, acc),
    p2a: createSeriesPattern1(client, _p('p2a', acc)),
    p2pk33: createSeriesPattern1(client, _p('p2pk33', acc)),
    p2pk65: createSeriesPattern1(client, _p('p2pk65', acc)),
    p2pkh: createSeriesPattern1(client, _p('p2pkh', acc)),
    p2sh: createSeriesPattern1(client, _p('p2sh', acc)),
    p2tr: createSeriesPattern1(client, _p('p2tr', acc)),
    p2wpkh: createSeriesPattern1(client, _p('p2wpkh', acc)),
    p2wsh: createSeriesPattern1(client, _p('p2wsh', acc)),
  };
}

/**
 * @typedef {Object} AllP2aP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshPattern
 * @property {_1m1w1y24hBlockPattern} all
 * @property {_1m1w1y24hBlockPattern} p2a
 * @property {_1m1w1y24hBlockPattern} p2pk33
 * @property {_1m1w1y24hBlockPattern} p2pk65
 * @property {_1m1w1y24hBlockPattern} p2pkh
 * @property {_1m1w1y24hBlockPattern} p2sh
 * @property {_1m1w1y24hBlockPattern} p2tr
 * @property {_1m1w1y24hBlockPattern} p2wpkh
 * @property {_1m1w1y24hBlockPattern} p2wsh
 */

/**
 * Create a AllP2aP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshPattern pattern node
 * @param {BitviewClient} client
 * @param {string} acc - Accumulated series name
 * @returns {AllP2aP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshPattern}
 */
function createAllP2aP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshPattern(client, acc) {
  return {
    all: create_1m1w1y24hBlockPattern(client, acc),
    p2a: create_1m1w1y24hBlockPattern(client, _p('p2a', acc)),
    p2pk33: create_1m1w1y24hBlockPattern(client, _p('p2pk33', acc)),
    p2pk65: create_1m1w1y24hBlockPattern(client, _p('p2pk65', acc)),
    p2pkh: create_1m1w1y24hBlockPattern(client, _p('p2pkh', acc)),
    p2sh: create_1m1w1y24hBlockPattern(client, _p('p2sh', acc)),
    p2tr: create_1m1w1y24hBlockPattern(client, _p('p2tr', acc)),
    p2wpkh: create_1m1w1y24hBlockPattern(client, _p('p2wpkh', acc)),
    p2wsh: create_1m1w1y24hBlockPattern(client, _p('p2wsh', acc)),
  };
}

/**
 * @typedef {Object} AllP2aP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshPattern7
 * @property {_1m1w1y24hPercentPpmRatioPattern} all
 * @property {_1m1w1y24hPercentPpmRatioPattern} p2a
 * @property {_1m1w1y24hPercentPpmRatioPattern} p2pk33
 * @property {_1m1w1y24hPercentPpmRatioPattern} p2pk65
 * @property {_1m1w1y24hPercentPpmRatioPattern} p2pkh
 * @property {_1m1w1y24hPercentPpmRatioPattern} p2sh
 * @property {_1m1w1y24hPercentPpmRatioPattern} p2tr
 * @property {_1m1w1y24hPercentPpmRatioPattern} p2wpkh
 * @property {_1m1w1y24hPercentPpmRatioPattern} p2wsh
 */

/**
 * Create a AllP2aP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshPattern7 pattern node
 * @param {BitviewClient} client
 * @param {string} acc - Accumulated series name
 * @returns {AllP2aP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshPattern7}
 */
function createAllP2aP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshPattern7(client, acc) {
  return {
    all: create_1m1w1y24hPercentPpmRatioPattern(client, acc),
    p2a: create_1m1w1y24hPercentPpmRatioPattern(client, _p('p2a', acc)),
    p2pk33: create_1m1w1y24hPercentPpmRatioPattern(client, _p('p2pk33', acc)),
    p2pk65: create_1m1w1y24hPercentPpmRatioPattern(client, _p('p2pk65', acc)),
    p2pkh: create_1m1w1y24hPercentPpmRatioPattern(client, _p('p2pkh', acc)),
    p2sh: create_1m1w1y24hPercentPpmRatioPattern(client, _p('p2sh', acc)),
    p2tr: create_1m1w1y24hPercentPpmRatioPattern(client, _p('p2tr', acc)),
    p2wpkh: create_1m1w1y24hPercentPpmRatioPattern(client, _p('p2wpkh', acc)),
    p2wsh: create_1m1w1y24hPercentPpmRatioPattern(client, _p('p2wsh', acc)),
  };
}

/**
 * @typedef {Object} AverageMaxMedianMinPct10Pct25Pct75Pct90SumPattern
 * @property {_1m1w1y24hPattern6<StoredF32>} average
 * @property {_1m1w1y24hPattern6<StoredU64>} max
 * @property {_1m1w1y24hPattern6<StoredU64>} median
 * @property {_1m1w1y24hPattern6<StoredU64>} min
 * @property {_1m1w1y24hPattern6<StoredU64>} pct10
 * @property {_1m1w1y24hPattern6<StoredU64>} pct25
 * @property {_1m1w1y24hPattern6<StoredU64>} pct75
 * @property {_1m1w1y24hPattern6<StoredU64>} pct90
 * @property {_1m1w1y24hPattern6<StoredU64>} sum
 */

/**
 * Create a AverageMaxMedianMinPct10Pct25Pct75Pct90SumPattern pattern node
 * @param {BitviewClient} client
 * @param {string} acc - Accumulated series name
 * @returns {AverageMaxMedianMinPct10Pct25Pct75Pct90SumPattern}
 */
function createAverageMaxMedianMinPct10Pct25Pct75Pct90SumPattern(client, acc) {
  return {
    average: create_1m1w1y24hPattern6(client, _m(acc, 'average')),
    max: create_1m1w1y24hPattern6(client, _m(acc, 'max')),
    median: create_1m1w1y24hPattern6(client, _m(acc, 'median')),
    min: create_1m1w1y24hPattern6(client, _m(acc, 'min')),
    pct10: create_1m1w1y24hPattern6(client, _m(acc, 'pct10')),
    pct25: create_1m1w1y24hPattern6(client, _m(acc, 'pct25')),
    pct75: create_1m1w1y24hPattern6(client, _m(acc, 'pct75')),
    pct90: create_1m1w1y24hPattern6(client, _m(acc, 'pct90')),
    sum: create_1m1w1y24hPattern6(client, _m(acc, 'sum')),
  };
}

/**
 * @typedef {Object} Pct10Pct20Pct30Pct40Pct50Pct60Pct70Pct80Pct90Pattern
 * @property {CentsSatsUsdPattern} pct10
 * @property {CentsSatsUsdPattern} pct20
 * @property {CentsSatsUsdPattern} pct30
 * @property {CentsSatsUsdPattern} pct40
 * @property {CentsSatsUsdPattern} pct50
 * @property {CentsSatsUsdPattern} pct60
 * @property {CentsSatsUsdPattern} pct70
 * @property {CentsSatsUsdPattern} pct80
 * @property {CentsSatsUsdPattern} pct90
 */

/**
 * Create a Pct10Pct20Pct30Pct40Pct50Pct60Pct70Pct80Pct90Pattern pattern node
 * @param {BitviewClient} client
 * @param {string} acc - Accumulated series name
 * @returns {Pct10Pct20Pct30Pct40Pct50Pct60Pct70Pct80Pct90Pattern}
 */
function createPct10Pct20Pct30Pct40Pct50Pct60Pct70Pct80Pct90Pattern(client, acc) {
  return {
    pct10: createCentsSatsUsdPattern(client, _m(acc, 'pct10')),
    pct20: createCentsSatsUsdPattern(client, _m(acc, 'pct20')),
    pct30: createCentsSatsUsdPattern(client, _m(acc, 'pct30')),
    pct40: createCentsSatsUsdPattern(client, _m(acc, 'pct40')),
    pct50: createCentsSatsUsdPattern(client, _m(acc, 'pct50')),
    pct60: createCentsSatsUsdPattern(client, _m(acc, 'pct60')),
    pct70: createCentsSatsUsdPattern(client, _m(acc, 'pct70')),
    pct80: createCentsSatsUsdPattern(client, _m(acc, 'pct80')),
    pct90: createCentsSatsUsdPattern(client, _m(acc, 'pct90')),
  };
}

/**
 * @typedef {Object} ActivityCostOutputsRatiosRealizedRelativeSupplyUnrealizedPattern
 * @property {CoindaysCoinyearsTransferPattern} activity
 * @property {InMaxMinPerSupplyPattern} costBasis
 * @property {SpentUnspentPattern} outputs
 * @property {AdjustedDormancyProfitSellSoprPattern} ratios
 * @property {CapCapitalizedGrossLossMvrvNetPeakPriceProfitValuePattern} realized
 * @property {InvestedNetRealizedSupplyUnrealizedPattern} relative
 * @property {DeltaInTotalPattern} supply
 * @property {GreedGrossInvestedLossNetNuplPainProfitPattern} unrealized
 */

/**
 * Create a ActivityCostOutputsRatiosRealizedRelativeSupplyUnrealizedPattern pattern node
 * @param {BitviewClient} client
 * @param {string} acc - Accumulated series name
 * @returns {ActivityCostOutputsRatiosRealizedRelativeSupplyUnrealizedPattern}
 */
function createActivityCostOutputsRatiosRealizedRelativeSupplyUnrealizedPattern(client, acc) {
  return {
    activity: createCoindaysCoinyearsTransferPattern(client, acc),
    costBasis: createInMaxMinPerSupplyPattern(client, acc),
    outputs: createSpentUnspentPattern(client, acc),
    ratios: createAdjustedDormancyProfitSellSoprPattern(client, acc),
    realized: createCapCapitalizedGrossLossMvrvNetPeakPriceProfitValuePattern(client, acc),
    relative: createInvestedNetRealizedSupplyUnrealizedPattern(client, acc),
    supply: createDeltaInTotalPattern(client, _m(acc, 'supply')),
    unrealized: createGreedGrossInvestedLossNetNuplPainProfitPattern(client, acc),
  };
}

/**
 * @typedef {Object} _1m1w1y24hPercentPpmRatioPattern
 * @property {PercentPpmRatioPattern2} _1m
 * @property {PercentPpmRatioPattern2} _1w
 * @property {PercentPpmRatioPattern2} _1y
 * @property {PercentPpmRatioPattern2} _24h
 * @property {SeriesPattern1<StoredF32>} percent
 * @property {SeriesPattern1<PartsPerMillion32>} ppm
 * @property {SeriesPattern1<StoredF32>} ratio
 */

/**
 * Create a _1m1w1y24hPercentPpmRatioPattern pattern node
 * @param {BitviewClient} client
 * @param {string} acc - Accumulated series name
 * @returns {_1m1w1y24hPercentPpmRatioPattern}
 */
function create_1m1w1y24hPercentPpmRatioPattern(client, acc) {
  return {
    _1m: createPercentPpmRatioPattern2(client, _m(acc, '1m')),
    _1w: createPercentPpmRatioPattern2(client, _m(acc, '1w')),
    _1y: createPercentPpmRatioPattern2(client, _m(acc, '1y')),
    _24h: createPercentPpmRatioPattern2(client, _m(acc, '24h')),
    percent: createSeriesPattern1(client, acc),
    ppm: createSeriesPattern1(client, _m(acc, 'ppm')),
    ratio: createSeriesPattern1(client, _m(acc, 'ratio')),
  };
}

/**
 * @typedef {Object} ActiveInputOutputSpendablePattern
 * @property {_1m1w1y24hBlockPattern} activeReusedAddrCount
 * @property {_1m1w1y24hBlockPattern2} activeReusedAddrShare
 * @property {AllP2aP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshPattern6} inputFromReusedAddrCount
 * @property {AllP2aP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshPattern7} inputFromReusedAddrShare
 * @property {AllP2aP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshPattern6} outputToReusedAddrCount
 * @property {AllP2aP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshPattern7} outputToReusedAddrShare
 * @property {_1m1w1y24hPercentPpmRatioPattern} spendableOutputToReusedAddrShare
 */

/**
 * @typedef {Object} AllLthOverSthUnderPattern2
 * @property {CapitalizedCostSupplyPattern} all
 * @property {CapitalizedCostSupplyPattern} lth
 * @property {CapitalizedCostSupplyPattern} over4m
 * @property {CapitalizedCostSupplyPattern} over6m
 * @property {CapitalizedCostSupplyPattern} sth
 * @property {CapitalizedCostSupplyPattern} under4m
 * @property {CapitalizedCostSupplyPattern} under6m
 */

/**
 * @typedef {Object} InMaxMinPerSupplyPattern
 * @property {PerPattern2} inLoss
 * @property {PerPattern2} inProfit
 * @property {CentsSatsUsdPattern} max
 * @property {CentsSatsUsdPattern} min
 * @property {Pct05Pct10Pct15Pct20Pct25Pct30Pct35Pct40Pct45Pct50Pct55Pct60Pct65Pct70Pct75Pct80Pct85Pct90Pct95Pattern} perCoin
 * @property {Pct05Pct10Pct15Pct20Pct25Pct30Pct35Pct40Pct45Pct50Pct55Pct60Pct65Pct70Pct75Pct80Pct85Pct90Pct95Pattern} perDollar
 * @property {PercentPpmRatioPattern2} supplyDensity
 */

/**
 * Create a InMaxMinPerSupplyPattern pattern node
 * @param {BitviewClient} client
 * @param {string} acc - Accumulated series name
 * @returns {InMaxMinPerSupplyPattern}
 */
function createInMaxMinPerSupplyPattern(client, acc) {
  return {
    inLoss: createPerPattern2(client, _m(acc, 'cost_basis_in_loss_per')),
    inProfit: createPerPattern2(client, _m(acc, 'cost_basis_in_profit_per')),
    max: createCentsSatsUsdPattern(client, _m(acc, 'cost_basis_max')),
    min: createCentsSatsUsdPattern(client, _m(acc, 'cost_basis_min')),
    perCoin: createPct05Pct10Pct15Pct20Pct25Pct30Pct35Pct40Pct45Pct50Pct55Pct60Pct65Pct70Pct75Pct80Pct85Pct90Pct95Pattern(client, _m(acc, 'cost_basis_per_coin')),
    perDollar: createPct05Pct10Pct15Pct20Pct25Pct30Pct35Pct40Pct45Pct50Pct55Pct60Pct65Pct70Pct75Pct80Pct85Pct90Pct95Pattern(client, _m(acc, 'cost_basis_per_dollar')),
    supplyDensity: createPercentPpmRatioPattern2(client, _m(acc, 'supply_density')),
  };
}

/**
 * @typedef {Object} MaxMedianMinPct10Pct25Pct75Pct90Pattern2
 * @property {SeriesPattern18<VSize>} max
 * @property {SeriesPattern18<VSize>} median
 * @property {SeriesPattern18<VSize>} min
 * @property {SeriesPattern18<VSize>} pct10
 * @property {SeriesPattern18<VSize>} pct25
 * @property {SeriesPattern18<VSize>} pct75
 * @property {SeriesPattern18<VSize>} pct90
 */

/**
 * Create a MaxMedianMinPct10Pct25Pct75Pct90Pattern2 pattern node
 * @param {BitviewClient} client
 * @param {string} acc - Accumulated series name
 * @returns {MaxMedianMinPct10Pct25Pct75Pct90Pattern2}
 */
function createMaxMedianMinPct10Pct25Pct75Pct90Pattern2(client, acc) {
  return {
    max: createSeriesPattern18(client, _m(acc, 'max')),
    median: createSeriesPattern18(client, _m(acc, 'median')),
    min: createSeriesPattern18(client, _m(acc, 'min')),
    pct10: createSeriesPattern18(client, _m(acc, 'pct10')),
    pct25: createSeriesPattern18(client, _m(acc, 'pct25')),
    pct75: createSeriesPattern18(client, _m(acc, 'pct75')),
    pct90: createSeriesPattern18(client, _m(acc, 'pct90')),
  };
}

/**
 * @template T
 * @typedef {Object} MaxMedianMinPct10Pct25Pct75Pct90Pattern
 * @property {SeriesPattern1<T>} max
 * @property {SeriesPattern1<T>} median
 * @property {SeriesPattern1<T>} min
 * @property {SeriesPattern1<T>} pct10
 * @property {SeriesPattern1<T>} pct25
 * @property {SeriesPattern1<T>} pct75
 * @property {SeriesPattern1<T>} pct90
 */

/**
 * Create a MaxMedianMinPct10Pct25Pct75Pct90Pattern pattern node
 * @template T
 * @param {BitviewClient} client
 * @param {string} acc - Accumulated series name
 * @returns {MaxMedianMinPct10Pct25Pct75Pct90Pattern<T>}
 */
function createMaxMedianMinPct10Pct25Pct75Pct90Pattern(client, acc) {
  return {
    max: createSeriesPattern1(client, _m(acc, 'max')),
    median: createSeriesPattern1(client, _m(acc, 'median')),
    min: createSeriesPattern1(client, _m(acc, 'min')),
    pct10: createSeriesPattern1(client, _m(acc, 'pct10')),
    pct25: createSeriesPattern1(client, _m(acc, 'pct25')),
    pct75: createSeriesPattern1(client, _m(acc, 'pct75')),
    pct90: createSeriesPattern1(client, _m(acc, 'pct90')),
  };
}

/**
 * @typedef {Object} AdjustedDormancyProfitSellSoprPattern
 * @property {RatioTransferValuePattern} adjustedSopr
 * @property {_1m1w1y24hPattern6<StoredF32>} dormancy
 * @property {_1m1w1y24hPattern6<StoredF32>} profitToLossRatio
 * @property {_1m1w1y24hPattern9} sellSideRiskRatio
 * @property {SeriesPattern1<StoredF32>} sopr
 * @property {_1m1w1yPattern} soprRatioExtended
 */

/**
 * Create a AdjustedDormancyProfitSellSoprPattern pattern node
 * @param {BitviewClient} client
 * @param {string} acc - Accumulated series name
 * @returns {AdjustedDormancyProfitSellSoprPattern}
 */
function createAdjustedDormancyProfitSellSoprPattern(client, acc) {
  return {
    adjustedSopr: createRatioTransferValuePattern(client, acc),
    dormancy: create_1m1w1y24hPattern6(client, _m(acc, 'dormancy')),
    profitToLossRatio: create_1m1w1y24hPattern6(client, _m(acc, 'realized_profit_to_loss_ratio')),
    sellSideRiskRatio: create_1m1w1y24hPattern9(client, _m(acc, 'sell_side_risk_ratio')),
    sopr: createSeriesPattern1(client, _m(acc, 'sopr_24h')),
    soprRatioExtended: create_1m1w1yPattern(client, _m(acc, 'sopr')),
  };
}

/**
 * @typedef {Object} AverageBlockChainCumulativeDataSumPattern
 * @property {_1m1w1y24hPattern6<StoredF32>} average
 * @property {SeriesPattern18<Bytes>} block
 * @property {PercentPpmRatioPattern2} chainShare
 * @property {SeriesPattern1<Bytes>} cumulative
 * @property {PercentPpmRatioPattern2} dataShare
 * @property {_1m1w1y24hPattern6<Bytes>} sum
 */

/**
 * Create a AverageBlockChainCumulativeDataSumPattern pattern node
 * @param {BitviewClient} client
 * @param {string} acc - Accumulated series name
 * @returns {AverageBlockChainCumulativeDataSumPattern}
 */
function createAverageBlockChainCumulativeDataSumPattern(client, acc) {
  return {
    average: create_1m1w1y24hPattern6(client, _m(acc, 'data_bytes_average')),
    block: createSeriesPattern18(client, _m(acc, 'data_bytes')),
    chainShare: createPercentPpmRatioPattern2(client, _m(acc, 'chain_share')),
    cumulative: createSeriesPattern1(client, _m(acc, 'data_bytes_cumulative')),
    dataShare: createPercentPpmRatioPattern2(client, _m(acc, 'data_share')),
    sum: create_1m1w1y24hPattern6(client, _m(acc, 'data_bytes_sum')),
  };
}

/**
 * @typedef {Object} _01234Pattern2
 * @property {AverageBlockCumulativeSumPattern} _0
 * @property {AverageBlockCumulativeSumPattern} _1
 * @property {AverageBlockCumulativeSumPattern} _2
 * @property {AverageBlockCumulativeSumPattern} _3
 * @property {AverageBlockCumulativeSumPattern} _4
 */

/**
 * Create a _01234Pattern2 pattern node
 * @param {BitviewClient} client
 * @param {string} acc - Accumulated series name
 * @param {string} disc - Discriminator suffix
 * @returns {_01234Pattern2}
 */
function create_01234Pattern2(client, acc, disc) {
  return {
    _0: createAverageBlockCumulativeSumPattern(client, _m(_m(acc, '0_transfer_volume'), disc)),
    _1: createAverageBlockCumulativeSumPattern(client, _m(_m(acc, '1_transfer_volume'), disc)),
    _2: createAverageBlockCumulativeSumPattern(client, _m(_m(acc, '2_transfer_volume'), disc)),
    _3: createAverageBlockCumulativeSumPattern(client, _m(_m(acc, '3_transfer_volume'), disc)),
    _4: createAverageBlockCumulativeSumPattern(client, _m(_m(acc, '4_transfer_volume'), disc)),
  };
}

/**
 * @typedef {Object} _01234Pattern
 * @property {AverageBlockCumulativeSumPattern2<StoredF64>} _0
 * @property {AverageBlockCumulativeSumPattern2<StoredF64>} _1
 * @property {AverageBlockCumulativeSumPattern2<StoredF64>} _2
 * @property {AverageBlockCumulativeSumPattern2<StoredF64>} _3
 * @property {AverageBlockCumulativeSumPattern2<StoredF64>} _4
 */

/**
 * @typedef {Object} _01234Pattern5
 * @property {BlockCumulativeSumPattern} _0
 * @property {BlockCumulativeSumPattern} _1
 * @property {BlockCumulativeSumPattern} _2
 * @property {BlockCumulativeSumPattern} _3
 * @property {BlockCumulativeSumPattern} _4
 */

/**
 * @typedef {Object} _01234Pattern10
 * @property {BtcCentsSatsUsdPattern} _0
 * @property {BtcCentsSatsUsdPattern} _1
 * @property {BtcCentsSatsUsdPattern} _2
 * @property {BtcCentsSatsUsdPattern} _3
 * @property {BtcCentsSatsUsdPattern} _4
 */

/**
 * @typedef {Object} _01234Pattern4
 * @property {CentsUsdPattern} _0
 * @property {CentsUsdPattern} _1
 * @property {CentsUsdPattern} _2
 * @property {CentsUsdPattern} _3
 * @property {CentsUsdPattern} _4
 */

/**
 * @typedef {Object} _1m1w1y24hBlockPattern2
 * @property {SeriesPattern1<StoredF32>} _1m
 * @property {SeriesPattern1<StoredF32>} _1w
 * @property {SeriesPattern1<StoredF32>} _1y
 * @property {SeriesPattern1<StoredF32>} _24h
 * @property {SeriesPattern18<StoredF32>} block
 */

/**
 * Create a _1m1w1y24hBlockPattern2 pattern node
 * @param {BitviewClient} client
 * @param {string} acc - Accumulated series name
 * @returns {_1m1w1y24hBlockPattern2}
 */
function create_1m1w1y24hBlockPattern2(client, acc) {
  return {
    _1m: createSeriesPattern1(client, _m(acc, 'average_1m')),
    _1w: createSeriesPattern1(client, _m(acc, 'average_1w')),
    _1y: createSeriesPattern1(client, _m(acc, 'average_1y')),
    _24h: createSeriesPattern1(client, _m(acc, 'average_24h')),
    block: createSeriesPattern18(client, acc),
  };
}

/**
 * @typedef {Object} _1m1w1y24hBlockPattern
 * @property {SeriesPattern1<StoredF32>} _1m
 * @property {SeriesPattern1<StoredF32>} _1w
 * @property {SeriesPattern1<StoredF32>} _1y
 * @property {SeriesPattern1<StoredF32>} _24h
 * @property {SeriesPattern18<StoredU32>} block
 */

/**
 * Create a _1m1w1y24hBlockPattern pattern node
 * @param {BitviewClient} client
 * @param {string} acc - Accumulated series name
 * @returns {_1m1w1y24hBlockPattern}
 */
function create_1m1w1y24hBlockPattern(client, acc) {
  return {
    _1m: createSeriesPattern1(client, _m(acc, 'average_1m')),
    _1w: createSeriesPattern1(client, _m(acc, 'average_1w')),
    _1y: createSeriesPattern1(client, _m(acc, 'average_1y')),
    _24h: createSeriesPattern1(client, _m(acc, 'average_24h')),
    block: createSeriesPattern18(client, acc),
  };
}

/**
 * @typedef {Object} AgeClassEpochTypeUtxoPattern4
 * @property {_10y12y18m1d1h1m1w1y2m2y3m3y4m4y5m5y6m6y7y8y9mOverUnderPattern5} age
 * @property {_200920102011201220132014201520162017201820192020202120222023202420252026Pattern5} class
 * @property {_01234Pattern5} epoch
 * @property {EmptyP2aP2msP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshUnknownPattern6} type
 * @property {_0sats100btc100k100sats10btc10k10m10sats1btc1k1m1satOverPattern2} utxoAmount
 */

/**
 * @typedef {Object} AverageBlockCumulativeFeeSumPattern
 * @property {_1m1w1y24hPattern6<StoredF32>} average
 * @property {SeriesPattern18<Sats>} block
 * @property {SeriesPattern1<Sats>} cumulative
 * @property {_1m1w1y24hPercentPpmRatioPattern} feeShare
 * @property {_1m1w1y24hPattern6<Sats>} sum
 */

/**
 * Create a AverageBlockCumulativeFeeSumPattern pattern node
 * @param {BitviewClient} client
 * @param {string} acc - Accumulated series name
 * @returns {AverageBlockCumulativeFeeSumPattern}
 */
function createAverageBlockCumulativeFeeSumPattern(client, acc) {
  return {
    average: create_1m1w1y24hPattern6(client, _m(acc, 'fees_average')),
    block: createSeriesPattern18(client, _m(acc, 'fees')),
    cumulative: createSeriesPattern1(client, _m(acc, 'fees_cumulative')),
    feeShare: create_1m1w1y24hPercentPpmRatioPattern(client, _m(acc, 'fee_share')),
    sum: create_1m1w1y24hPattern6(client, _m(acc, 'fees_sum')),
  };
}

/**
 * @typedef {Object} BtcCentsInSatsUsdPattern
 * @property {SeriesPattern1<Bitcoin>} btc
 * @property {SeriesPattern1<Cents>} cents
 * @property {SharePattern} inLoss
 * @property {SeriesPattern1<Sats>} sats
 * @property {SeriesPattern1<Dollars>} usd
 */

/**
 * @typedef {Object} CentsPpmRatioSatsUsdPattern
 * @property {SeriesPattern1<Cents>} cents
 * @property {SeriesPattern1<PriceRatio>} ppm
 * @property {SeriesPattern1<StoredF32>} ratio
 * @property {SeriesPattern1<SatsFract>} sats
 * @property {SeriesPattern1<Dollars>} usd
 */

/**
 * Create a CentsPpmRatioSatsUsdPattern pattern node
 * @param {BitviewClient} client
 * @param {string} acc - Accumulated series name
 * @returns {CentsPpmRatioSatsUsdPattern}
 */
function createCentsPpmRatioSatsUsdPattern(client, acc) {
  return {
    cents: createSeriesPattern1(client, _m(acc, 'cents')),
    ppm: createSeriesPattern1(client, _m(acc, 'ratio_ppm')),
    ratio: createSeriesPattern1(client, _m(acc, 'ratio')),
    sats: createSeriesPattern1(client, _m(acc, 'sats')),
    usd: createSeriesPattern1(client, acc),
  };
}

/**
 * @typedef {Object} CoindaysCoinyearsTransferPattern
 * @property {AverageBlockCumulativeSumPattern2<StoredF64>} coindaysDestroyed
 * @property {SeriesPattern1<StoredF64>} coinyearsDestroyed
 * @property {AverageBlockCumulativeSumPattern} transferVolume
 * @property {AverageBlockCumulativeSumPattern} transferVolumeInLoss
 * @property {AverageBlockCumulativeSumPattern} transferVolumeInProfit
 */

/**
 * Create a CoindaysCoinyearsTransferPattern pattern node
 * @param {BitviewClient} client
 * @param {string} acc - Accumulated series name
 * @returns {CoindaysCoinyearsTransferPattern}
 */
function createCoindaysCoinyearsTransferPattern(client, acc) {
  return {
    coindaysDestroyed: createAverageBlockCumulativeSumPattern2(client, _m(acc, 'coindays_destroyed')),
    coinyearsDestroyed: createSeriesPattern1(client, _m(acc, 'coinyears_destroyed')),
    transferVolume: createAverageBlockCumulativeSumPattern(client, _m(acc, 'transfer_volume')),
    transferVolumeInLoss: createAverageBlockCumulativeSumPattern(client, _m(acc, 'transfer_volume_in_loss')),
    transferVolumeInProfit: createAverageBlockCumulativeSumPattern(client, _m(acc, 'transfer_volume_in_profit')),
  };
}

/**
 * @typedef {Object} EmaHistogramLineSignalPattern
 * @property {SeriesPattern1<StoredF32>} emaFast
 * @property {SeriesPattern1<StoredF32>} emaSlow
 * @property {SeriesPattern1<StoredF32>} histogram
 * @property {SeriesPattern1<StoredF32>} line
 * @property {SeriesPattern1<StoredF32>} signal
 */

/**
 * @typedef {Object} Pct95Pct98Pct99Pattern
 * @property {CentsSatsUsdPattern} pct95
 * @property {CentsSatsUsdPattern} pct98
 * @property {CentsSatsUsdPattern} pct99
 * @property {CentsSatsUsdPattern} pct995
 * @property {CentsSatsUsdPattern} pct999
 */

/**
 * Create a Pct95Pct98Pct99Pattern pattern node
 * @param {BitviewClient} client
 * @param {string} acc - Accumulated series name
 * @returns {Pct95Pct98Pct99Pattern}
 */
function createPct95Pct98Pct99Pattern(client, acc) {
  return {
    pct95: createCentsSatsUsdPattern(client, _m(acc, 'pct95')),
    pct98: createCentsSatsUsdPattern(client, _m(acc, 'pct98')),
    pct99: createCentsSatsUsdPattern(client, _m(acc, 'pct99')),
    pct995: createCentsSatsUsdPattern(client, _m(acc, 'pct99_5')),
    pct999: createCentsSatsUsdPattern(client, _m(acc, 'pct99_9')),
  };
}

/**
 * @typedef {Object} Pct95Pct98Pct99Pattern2
 * @property {SeriesPattern1<StoredF64>} pct95
 * @property {SeriesPattern1<StoredF64>} pct98
 * @property {SeriesPattern1<StoredF64>} pct99
 * @property {SeriesPattern1<StoredF64>} pct995
 * @property {SeriesPattern1<StoredF64>} pct999
 */

/**
 * Create a Pct95Pct98Pct99Pattern2 pattern node
 * @param {BitviewClient} client
 * @param {string} acc - Accumulated series name
 * @returns {Pct95Pct98Pct99Pattern2}
 */
function createPct95Pct98Pct99Pattern2(client, acc) {
  return {
    pct95: createSeriesPattern1(client, _m(acc, 'pct95_ratio')),
    pct98: createSeriesPattern1(client, _m(acc, 'pct98_ratio')),
    pct99: createSeriesPattern1(client, _m(acc, 'pct99_ratio')),
    pct995: createSeriesPattern1(client, _m(acc, 'pct99_5_ratio')),
    pct999: createSeriesPattern1(client, _m(acc, 'pct99_9_ratio')),
  };
}

/**
 * @typedef {Object} PhsReboundThsPattern
 * @property {SeriesPattern1<StoredF32>} phs
 * @property {SeriesPattern1<StoredF32>} phsMin
 * @property {PercentPpmRatioPattern3} rebound
 * @property {SeriesPattern1<StoredF32>} ths
 * @property {SeriesPattern1<StoredF32>} thsMin
 */

/**
 * Create a PhsReboundThsPattern pattern node
 * @param {BitviewClient} client
 * @param {string} acc - Accumulated series name
 * @returns {PhsReboundThsPattern}
 */
function createPhsReboundThsPattern(client, acc) {
  return {
    phs: createSeriesPattern1(client, _m(acc, 'phs')),
    phsMin: createSeriesPattern1(client, _m(acc, 'phs_min')),
    rebound: createPercentPpmRatioPattern3(client, _m(acc, 'rebound')),
    ths: createSeriesPattern1(client, _m(acc, 'ths')),
    thsMin: createSeriesPattern1(client, _m(acc, 'ths_min')),
  };
}

/**
 * @typedef {Object} RankTailThresholdPattern
 * @property {SeriesPattern1<StoredU8>} rank
 * @property {PercentPpmRatioPattern2} tail
 * @property {SeriesPattern1<Dollars>} thresholdPct0025
 * @property {SeriesPattern1<Dollars>} thresholdPct005
 * @property {SeriesPattern1<Dollars>} thresholdPct01
 */

/**
 * Create a RankTailThresholdPattern pattern node
 * @param {BitviewClient} client
 * @param {string} acc - Accumulated series name
 * @returns {RankTailThresholdPattern}
 */
function createRankTailThresholdPattern(client, acc) {
  return {
    rank: createSeriesPattern1(client, _m(acc, 'rank')),
    tail: createPercentPpmRatioPattern2(client, _m(acc, 'tail')),
    thresholdPct0025: createSeriesPattern1(client, _m(acc, 'threshold')),
    thresholdPct005: createSeriesPattern1(client, _m(acc, 'threshold_pct0_05')),
    thresholdPct01: createSeriesPattern1(client, _m(acc, 'threshold_pct0_1')),
  };
}

/**
 * @typedef {Object} _1m1w1y24hPattern5
 * @property {BtcCentsSatsUsdPattern} _1m
 * @property {BtcCentsSatsUsdPattern} _1w
 * @property {BtcCentsSatsUsdPattern} _1y
 * @property {BtcCentsSatsUsdPattern} _24h
 */

/**
 * Create a _1m1w1y24hPattern5 pattern node
 * @param {BitviewClient} client
 * @param {string} acc - Accumulated series name
 * @returns {_1m1w1y24hPattern5}
 */
function create_1m1w1y24hPattern5(client, acc) {
  return {
    _1m: createBtcCentsSatsUsdPattern(client, _m(acc, '1m')),
    _1w: createBtcCentsSatsUsdPattern(client, _m(acc, '1w')),
    _1y: createBtcCentsSatsUsdPattern(client, _m(acc, '1y')),
    _24h: createBtcCentsSatsUsdPattern(client, _m(acc, '24h')),
  };
}

/**
 * @typedef {Object} _1m1w1y24hPattern4
 * @property {BtcCentsSatsUsdPattern2} _1m
 * @property {BtcCentsSatsUsdPattern2} _1w
 * @property {BtcCentsSatsUsdPattern2} _1y
 * @property {BtcCentsSatsUsdPattern2} _24h
 */

/**
 * Create a _1m1w1y24hPattern4 pattern node
 * @param {BitviewClient} client
 * @param {string} acc - Accumulated series name
 * @returns {_1m1w1y24hPattern4}
 */
function create_1m1w1y24hPattern4(client, acc) {
  return {
    _1m: createBtcCentsSatsUsdPattern2(client, _m(acc, '1m')),
    _1w: createBtcCentsSatsUsdPattern2(client, _m(acc, '1w')),
    _1y: createBtcCentsSatsUsdPattern2(client, _m(acc, '1y')),
    _24h: createBtcCentsSatsUsdPattern2(client, _m(acc, '24h')),
  };
}

/**
 * @typedef {Object} _1m1w1y24hPattern2
 * @property {BtcSatsPattern} _1m
 * @property {BtcSatsPattern} _1w
 * @property {BtcSatsPattern} _1y
 * @property {BtcSatsPattern} _24h
 */

/**
 * Create a _1m1w1y24hPattern2 pattern node
 * @param {BitviewClient} client
 * @param {string} acc - Accumulated series name
 * @returns {_1m1w1y24hPattern2}
 */
function create_1m1w1y24hPattern2(client, acc) {
  return {
    _1m: createBtcSatsPattern(client, _m(acc, '1m')),
    _1w: createBtcSatsPattern(client, _m(acc, '1w')),
    _1y: createBtcSatsPattern(client, _m(acc, '1y')),
    _24h: createBtcSatsPattern(client, _m(acc, '24h')),
  };
}

/**
 * @typedef {Object} _1m1w1y2wPattern
 * @property {CentsSatsUsdPattern} _1m
 * @property {CentsSatsUsdPattern} _1w
 * @property {CentsSatsUsdPattern} _1y
 * @property {CentsSatsUsdPattern} _2w
 */

/**
 * Create a _1m1w1y2wPattern pattern node
 * @param {BitviewClient} client
 * @param {string} acc - Accumulated series name
 * @returns {_1m1w1y2wPattern}
 */
function create_1m1w1y2wPattern(client, acc) {
  return {
    _1m: createCentsSatsUsdPattern(client, _m(acc, '1m')),
    _1w: createCentsSatsUsdPattern(client, _m(acc, '1w')),
    _1y: createCentsSatsUsdPattern(client, _m(acc, '1y')),
    _2w: createCentsSatsUsdPattern(client, _m(acc, '2w')),
  };
}

/**
 * @typedef {Object} _1m1w1y24hPattern
 * @property {CentsUsdPattern} _1m
 * @property {CentsUsdPattern} _1w
 * @property {CentsUsdPattern} _1y
 * @property {CentsUsdPattern} _24h
 */

/**
 * Create a _1m1w1y24hPattern pattern node
 * @param {BitviewClient} client
 * @param {string} acc - Accumulated series name
 * @returns {_1m1w1y24hPattern}
 */
function create_1m1w1y24hPattern(client, acc) {
  return {
    _1m: createCentsUsdPattern(client, _m(acc, '1m')),
    _1w: createCentsUsdPattern(client, _m(acc, '1w')),
    _1y: createCentsUsdPattern(client, _m(acc, '1y')),
    _24h: createCentsUsdPattern(client, _m(acc, '24h')),
  };
}

/**
 * @typedef {Object} _1m1w1y24hPattern7
 * @property {CentsUsdPattern4} _1m
 * @property {CentsUsdPattern4} _1w
 * @property {CentsUsdPattern4} _1y
 * @property {CentsUsdPattern4} _24h
 */

/**
 * Create a _1m1w1y24hPattern7 pattern node
 * @param {BitviewClient} client
 * @param {string} acc - Accumulated series name
 * @returns {_1m1w1y24hPattern7}
 */
function create_1m1w1y24hPattern7(client, acc) {
  return {
    _1m: createCentsUsdPattern4(client, _m(acc, '1m')),
    _1w: createCentsUsdPattern4(client, _m(acc, '1w')),
    _1y: createCentsUsdPattern4(client, _m(acc, '1y')),
    _24h: createCentsUsdPattern4(client, _m(acc, '24h')),
  };
}

/**
 * @typedef {Object} _1m1w1y24hPattern8
 * @property {CentsUsdPattern5} _1m
 * @property {CentsUsdPattern5} _1w
 * @property {CentsUsdPattern5} _1y
 * @property {CentsUsdPattern5} _24h
 */

/**
 * Create a _1m1w1y24hPattern8 pattern node
 * @param {BitviewClient} client
 * @param {string} acc - Accumulated series name
 * @returns {_1m1w1y24hPattern8}
 */
function create_1m1w1y24hPattern8(client, acc) {
  return {
    _1m: createCentsUsdPattern5(client, _m(acc, '1m')),
    _1w: createCentsUsdPattern5(client, _m(acc, '1w')),
    _1y: createCentsUsdPattern5(client, _m(acc, '1y')),
    _24h: createCentsUsdPattern5(client, _m(acc, '24h')),
  };
}

/**
 * @typedef {Object} _1m1w1y24hPattern3
 * @property {PercentPpmRatioPattern} _1m
 * @property {PercentPpmRatioPattern} _1w
 * @property {PercentPpmRatioPattern} _1y
 * @property {PercentPpmRatioPattern} _24h
 */

/**
 * Create a _1m1w1y24hPattern3 pattern node
 * @param {BitviewClient} client
 * @param {string} acc - Accumulated series name
 * @returns {_1m1w1y24hPattern3}
 */
function create_1m1w1y24hPattern3(client, acc) {
  return {
    _1m: createPercentPpmRatioPattern(client, _m(acc, '1m_rate')),
    _1w: createPercentPpmRatioPattern(client, _m(acc, '1w_rate')),
    _1y: createPercentPpmRatioPattern(client, _m(acc, '1y_rate')),
    _24h: createPercentPpmRatioPattern(client, _m(acc, '24h_rate')),
  };
}

/**
 * @typedef {Object} _1m1w1y24hPattern9
 * @property {PercentPpmRatioPattern2} _1m
 * @property {PercentPpmRatioPattern2} _1w
 * @property {PercentPpmRatioPattern2} _1y
 * @property {PercentPpmRatioPattern2} _24h
 */

/**
 * Create a _1m1w1y24hPattern9 pattern node
 * @param {BitviewClient} client
 * @param {string} acc - Accumulated series name
 * @returns {_1m1w1y24hPattern9}
 */
function create_1m1w1y24hPattern9(client, acc) {
  return {
    _1m: createPercentPpmRatioPattern2(client, _m(acc, '1m')),
    _1w: createPercentPpmRatioPattern2(client, _m(acc, '1w')),
    _1y: createPercentPpmRatioPattern2(client, _m(acc, '1y')),
    _24h: createPercentPpmRatioPattern2(client, _m(acc, '24h')),
  };
}

/**
 * @typedef {Object} AverageBlockCumulativeSumPattern
 * @property {_1m1w1y24hPattern4} average
 * @property {BtcCentsSatsUsdPattern3} block
 * @property {BtcCentsSatsUsdPattern} cumulative
 * @property {_1m1w1y24hPattern5} sum
 */

/**
 * Create a AverageBlockCumulativeSumPattern pattern node
 * @param {BitviewClient} client
 * @param {string} acc - Accumulated series name
 * @returns {AverageBlockCumulativeSumPattern}
 */
function createAverageBlockCumulativeSumPattern(client, acc) {
  return {
    average: create_1m1w1y24hPattern4(client, _m(acc, 'average')),
    block: createBtcCentsSatsUsdPattern3(client, acc),
    cumulative: createBtcCentsSatsUsdPattern(client, _m(acc, 'cumulative')),
    sum: create_1m1w1y24hPattern5(client, _m(acc, 'sum')),
  };
}

/**
 * @typedef {Object} AverageBlockCumulativeSumPattern3
 * @property {_1m1w1y24hPattern8} average
 * @property {CentsUsdPattern2} block
 * @property {CentsUsdPattern} cumulative
 * @property {_1m1w1y24hPattern} sum
 */

/**
 * Create a AverageBlockCumulativeSumPattern3 pattern node
 * @param {BitviewClient} client
 * @param {string} acc - Accumulated series name
 * @returns {AverageBlockCumulativeSumPattern3}
 */
function createAverageBlockCumulativeSumPattern3(client, acc) {
  return {
    average: create_1m1w1y24hPattern8(client, _m(acc, 'average')),
    block: createCentsUsdPattern2(client, acc),
    cumulative: createCentsUsdPattern(client, _m(acc, 'cumulative')),
    sum: create_1m1w1y24hPattern(client, _m(acc, 'sum')),
  };
}

/**
 * @typedef {Object} BlockCumulativeDeltaSumPattern
 * @property {CentsUsdPattern3} block
 * @property {CentsUsdPattern4} cumulative
 * @property {AbsoluteRatePattern3} delta
 * @property {_1m1w1y24hPattern7} sum
 */

/**
 * Create a BlockCumulativeDeltaSumPattern pattern node
 * @param {BitviewClient} client
 * @param {string} acc - Accumulated series name
 * @returns {BlockCumulativeDeltaSumPattern}
 */
function createBlockCumulativeDeltaSumPattern(client, acc) {
  return {
    block: createCentsUsdPattern3(client, acc),
    cumulative: createCentsUsdPattern4(client, _m(acc, 'cumulative')),
    delta: createAbsoluteRatePattern3(client, _m(acc, 'delta')),
    sum: create_1m1w1y24hPattern7(client, _m(acc, 'sum')),
  };
}

/**
 * @typedef {Object} BtcCentsSatsUsdPattern
 * @property {SeriesPattern1<Bitcoin>} btc
 * @property {SeriesPattern1<Cents>} cents
 * @property {SeriesPattern1<Sats>} sats
 * @property {SeriesPattern1<Dollars>} usd
 */

/**
 * Create a BtcCentsSatsUsdPattern pattern node
 * @param {BitviewClient} client
 * @param {string} acc - Accumulated series name
 * @returns {BtcCentsSatsUsdPattern}
 */
function createBtcCentsSatsUsdPattern(client, acc) {
  return {
    btc: createSeriesPattern1(client, acc),
    cents: createSeriesPattern1(client, _m(acc, 'cents')),
    sats: createSeriesPattern1(client, _m(acc, 'sats')),
    usd: createSeriesPattern1(client, _m(acc, 'usd')),
  };
}

/**
 * @typedef {Object} BtcCentsSatsUsdPattern2
 * @property {SeriesPattern1<Bitcoin>} btc
 * @property {SeriesPattern1<StoredF32>} cents
 * @property {SeriesPattern1<StoredF32>} sats
 * @property {SeriesPattern1<Dollars>} usd
 */

/**
 * Create a BtcCentsSatsUsdPattern2 pattern node
 * @param {BitviewClient} client
 * @param {string} acc - Accumulated series name
 * @returns {BtcCentsSatsUsdPattern2}
 */
function createBtcCentsSatsUsdPattern2(client, acc) {
  return {
    btc: createSeriesPattern1(client, acc),
    cents: createSeriesPattern1(client, _m(acc, 'cents')),
    sats: createSeriesPattern1(client, _m(acc, 'sats')),
    usd: createSeriesPattern1(client, _m(acc, 'usd')),
  };
}

/**
 * @typedef {Object} BtcCentsSatsUsdPattern3
 * @property {SeriesPattern18<Bitcoin>} btc
 * @property {SeriesPattern18<Cents>} cents
 * @property {SeriesPattern18<Sats>} sats
 * @property {SeriesPattern18<Dollars>} usd
 */

/**
 * Create a BtcCentsSatsUsdPattern3 pattern node
 * @param {BitviewClient} client
 * @param {string} acc - Accumulated series name
 * @returns {BtcCentsSatsUsdPattern3}
 */
function createBtcCentsSatsUsdPattern3(client, acc) {
  return {
    btc: createSeriesPattern18(client, acc),
    cents: createSeriesPattern18(client, _m(acc, 'cents')),
    sats: createSeriesPattern18(client, _m(acc, 'sats')),
    usd: createSeriesPattern18(client, _m(acc, 'usd')),
  };
}

/**
 * @typedef {Object} CapCapitalizedPriceSupplyPattern2
 * @property {CentsUsdPattern} cap
 * @property {CentsPpmRatioSatsUsdPattern} capitalizedPrice
 * @property {CentsPpmRatioSatsUsdPattern} price
 * @property {BtcCentsInSatsUsdPattern} supply
 */

/**
 * @typedef {Object} CapCapitalizedPriceSupplyPattern
 * @property {CentsUsdPattern} cap
 * @property {CentsPpmRatioSatsUsdPattern} capitalizedPrice
 * @property {CentsPpmRatioSatsUsdPattern} price
 * @property {ImmobileMobilePattern2} supply
 */

/**
 * @typedef {Object} DeltaInTotalPattern
 * @property {AbsoluteRatePattern} delta
 * @property {BtcCentsSatsUsdPattern} inLoss
 * @property {BtcCentsSatsUsdPattern} inProfit
 * @property {BtcCentsSatsUsdPattern} total
 */

/**
 * Create a DeltaInTotalPattern pattern node
 * @param {BitviewClient} client
 * @param {string} acc - Accumulated series name
 * @returns {DeltaInTotalPattern}
 */
function createDeltaInTotalPattern(client, acc) {
  return {
    delta: createAbsoluteRatePattern(client, _m(acc, 'delta')),
    inLoss: createBtcCentsSatsUsdPattern(client, _m(acc, 'in_loss')),
    inProfit: createBtcCentsSatsUsdPattern(client, _m(acc, 'in_profit')),
    total: createBtcCentsSatsUsdPattern(client, acc),
  };
}

/**
 * @typedef {Object} MultipleOversizedPrePattern3
 * @property {AverageBlockCumulativeSumPattern2<StoredU64>} multiple
 * @property {AverageBlockCumulativeSumPattern2<StoredU64>} oversized
 * @property {AverageBlockCumulativeSumPattern2<StoredU64>} preV30Nonstandard
 * @property {AverageBlockCumulativeSumPattern2<StoredU64>} preV30Standard
 */

/**
 * @template T
 * @typedef {Object} _1m1w1y24hPattern6
 * @property {SeriesPattern1<T>} _1m
 * @property {SeriesPattern1<T>} _1w
 * @property {SeriesPattern1<T>} _1y
 * @property {SeriesPattern1<T>} _24h
 */

/**
 * Create a _1m1w1y24hPattern6 pattern node
 * @template T
 * @param {BitviewClient} client
 * @param {string} acc - Accumulated series name
 * @returns {_1m1w1y24hPattern6<T>}
 */
function create_1m1w1y24hPattern6(client, acc) {
  return {
    _1m: createSeriesPattern1(client, _m(acc, '1m')),
    _1w: createSeriesPattern1(client, _m(acc, '1w')),
    _1y: createSeriesPattern1(client, _m(acc, '1y')),
    _24h: createSeriesPattern1(client, _m(acc, '24h')),
  };
}

/**
 * @template T
 * @typedef {Object} AverageBlockCumulativeSumPattern2
 * @property {_1m1w1y24hPattern6<T>} average
 * @property {SeriesPattern18<T>} block
 * @property {SeriesPattern1<T>} cumulative
 * @property {_1m1w1y24hPattern6<T>} sum
 */

/**
 * Create a AverageBlockCumulativeSumPattern2 pattern node
 * @template T
 * @param {BitviewClient} client
 * @param {string} acc - Accumulated series name
 * @returns {AverageBlockCumulativeSumPattern2<T>}
 */
function createAverageBlockCumulativeSumPattern2(client, acc) {
  return {
    average: create_1m1w1y24hPattern6(client, _m(acc, 'average')),
    block: createSeriesPattern18(client, acc),
    cumulative: createSeriesPattern1(client, _m(acc, 'cumulative')),
    sum: create_1m1w1y24hPattern6(client, _m(acc, 'sum')),
  };
}

/**
 * @typedef {Object} _1m1w1yPattern
 * @property {SeriesPattern1<StoredF32>} _1m
 * @property {SeriesPattern1<StoredF32>} _1w
 * @property {SeriesPattern1<StoredF32>} _1y
 */

/**
 * Create a _1m1w1yPattern pattern node
 * @param {BitviewClient} client
 * @param {string} acc - Accumulated series name
 * @returns {_1m1w1yPattern}
 */
function create_1m1w1yPattern(client, acc) {
  return {
    _1m: createSeriesPattern1(client, _m(acc, '1m')),
    _1w: createSeriesPattern1(client, _m(acc, '1w')),
    _1y: createSeriesPattern1(client, _m(acc, '1y')),
  };
}

/**
 * @typedef {Object} AgeClassEpochPattern5
 * @property {_10y12y18m1d1h1m1w1y2m2y3m3y4m4y5m5y6m6y7y8y9mOverUnderPattern10} age
 * @property {_200920102011201220132014201520162017201820192020202120222023202420252026Pattern10} class
 * @property {_01234Pattern10} epoch
 */

/**
 * @typedef {Object} AgeClassEpochPattern2
 * @property {_10y12y18m1d1h1m1w1y2m2y3m3y4m4y5m5y6m6y7y8y9mOverUnderPattern2} age
 * @property {_200920102011201220132014201520162017201820192020202120222023202420252026Pattern2} class
 * @property {_01234Pattern2} epoch
 */

/**
 * @typedef {Object} AgeClassEpochPattern6
 * @property {_10y12y18m1d1h1m1w1y2m2y3m3y4m4y5m5y6m6y7y8y9mOverUnderPattern4} age
 * @property {_200920102011201220132014201520162017201820192020202120222023202420252026Pattern4} class
 * @property {_01234Pattern4} epoch
 */

/**
 * @typedef {Object} BlockCumulativeSumPattern
 * @property {CentsUsdPattern2} block
 * @property {CentsUsdPattern} cumulative
 * @property {_1m1w1y24hPattern} sum
 */

/**
 * Create a BlockCumulativeSumPattern pattern node
 * @param {BitviewClient} client
 * @param {string} acc - Accumulated series name
 * @returns {BlockCumulativeSumPattern}
 */
function createBlockCumulativeSumPattern(client, acc) {
  return {
    block: createCentsUsdPattern2(client, acc),
    cumulative: createCentsUsdPattern(client, _m(acc, 'cumulative')),
    sum: create_1m1w1y24hPattern(client, _m(acc, 'sum')),
  };
}

/**
 * @typedef {Object} BlockCumulativeSumPattern2
 * @property {SeriesPattern18<StoredU64>} block
 * @property {SeriesPattern1<StoredU64>} cumulative
 * @property {_1m1w1y24hPattern6<StoredU64>} sum
 */

/**
 * Create a BlockCumulativeSumPattern2 pattern node
 * @param {BitviewClient} client
 * @param {string} acc - Accumulated series name
 * @returns {BlockCumulativeSumPattern2}
 */
function createBlockCumulativeSumPattern2(client, acc) {
  return {
    block: createSeriesPattern18(client, acc),
    cumulative: createSeriesPattern1(client, _m(acc, 'cumulative')),
    sum: create_1m1w1y24hPattern6(client, _m(acc, 'sum')),
  };
}

/**
 * @typedef {Object} BlocksDominanceRewardsPattern
 * @property {BlockCumulativeSumPattern2} blocksMined
 * @property {_1m1w1y24hPercentPpmRatioPattern} dominance
 * @property {AverageBlockCumulativeSumPattern} rewards
 */

/**
 * Create a BlocksDominanceRewardsPattern pattern node
 * @param {BitviewClient} client
 * @param {string} acc - Accumulated series name
 * @returns {BlocksDominanceRewardsPattern}
 */
function createBlocksDominanceRewardsPattern(client, acc) {
  return {
    blocksMined: createBlockCumulativeSumPattern2(client, _m(acc, 'blocks_mined')),
    dominance: create_1m1w1y24hPercentPpmRatioPattern(client, _m(acc, 'dominance')),
    rewards: createAverageBlockCumulativeSumPattern(client, _m(acc, 'rewards')),
  };
}

/**
 * @typedef {Object} CapitalizedCostSupplyPattern
 * @property {CentsPpmRatioSatsUsdPattern} capitalizedPrice
 * @property {PerPattern} costBasis
 * @property {InTotalPattern} supplyDensity
 */

/**
 * Create a CapitalizedCostSupplyPattern pattern node
 * @param {BitviewClient} client
 * @param {string} acc - Accumulated series name
 * @returns {CapitalizedCostSupplyPattern}
 */
function createCapitalizedCostSupplyPattern(client, acc) {
  return {
    capitalizedPrice: createCentsPpmRatioSatsUsdPattern(client, _m(acc, 'urpd_all_capitalized_price')),
    costBasis: createPerPattern(client, _m(acc, 'cost_basis_per')),
    supplyDensity: createInTotalPattern(client, _m(acc, 'urpd_all_supply_density')),
  };
}

/**
 * @typedef {Object} CentsSatsUsdPattern3
 * @property {SeriesPattern2<Cents>} cents
 * @property {SeriesPattern2<Sats>} sats
 * @property {SeriesPattern2<Dollars>} usd
 */

/**
 * Create a CentsSatsUsdPattern3 pattern node
 * @param {BitviewClient} client
 * @param {string} acc - Accumulated series name
 * @returns {CentsSatsUsdPattern3}
 */
function createCentsSatsUsdPattern3(client, acc) {
  return {
    cents: createSeriesPattern2(client, _m(acc, 'cents')),
    sats: createSeriesPattern2(client, _m(acc, 'sats')),
    usd: createSeriesPattern2(client, acc),
  };
}

/**
 * @typedef {Object} CentsDeltaUsdPattern
 * @property {SeriesPattern1<Cents>} cents
 * @property {AbsoluteRatePattern3} delta
 * @property {SeriesPattern1<Dollars>} usd
 */

/**
 * Create a CentsDeltaUsdPattern pattern node
 * @param {BitviewClient} client
 * @param {string} acc - Accumulated series name
 * @returns {CentsDeltaUsdPattern}
 */
function createCentsDeltaUsdPattern(client, acc) {
  return {
    cents: createSeriesPattern1(client, _m(acc, 'cents')),
    delta: createAbsoluteRatePattern3(client, _m(acc, 'delta')),
    usd: createSeriesPattern1(client, acc),
  };
}

/**
 * @typedef {Object} CentsSatsUsdPattern
 * @property {SeriesPattern1<Cents>} cents
 * @property {SeriesPattern1<SatsFract>} sats
 * @property {SeriesPattern1<Dollars>} usd
 */

/**
 * Create a CentsSatsUsdPattern pattern node
 * @param {BitviewClient} client
 * @param {string} acc - Accumulated series name
 * @returns {CentsSatsUsdPattern}
 */
function createCentsSatsUsdPattern(client, acc) {
  return {
    cents: createSeriesPattern1(client, _m(acc, 'cents')),
    sats: createSeriesPattern1(client, _m(acc, 'sats')),
    usd: createSeriesPattern1(client, acc),
  };
}

/**
 * @typedef {Object} CountEventsSupplyPattern
 * @property {FundedTotalPattern} count
 * @property {ActiveInputOutputSpendablePattern} events
 * @property {AllP2aP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshSharePattern} supply
 */

/**
 * @typedef {Object} CumulativeRollingSumPattern
 * @property {SeriesPattern1<StoredU64>} cumulative
 * @property {AverageMaxMedianMinPct10Pct25Pct75Pct90SumPattern} rolling
 * @property {SeriesPattern18<StoredU64>} sum
 */

/**
 * Create a CumulativeRollingSumPattern pattern node
 * @param {BitviewClient} client
 * @param {string} acc - Accumulated series name
 * @returns {CumulativeRollingSumPattern}
 */
function createCumulativeRollingSumPattern(client, acc) {
  return {
    cumulative: createSeriesPattern1(client, _m(acc, 'cumulative')),
    rolling: createAverageMaxMedianMinPct10Pct25Pct75Pct90SumPattern(client, acc),
    sum: createSeriesPattern18(client, _m(acc, 'sum')),
  };
}

/**
 * @typedef {Object} DeltaDominanceTotalPattern
 * @property {AbsoluteRatePattern} delta
 * @property {PercentPpmRatioPattern2} dominance
 * @property {BtcCentsSatsUsdPattern} total
 */

/**
 * Create a DeltaDominanceTotalPattern pattern node
 * @param {BitviewClient} client
 * @param {string} acc - Accumulated series name
 * @returns {DeltaDominanceTotalPattern}
 */
function createDeltaDominanceTotalPattern(client, acc) {
  return {
    delta: createAbsoluteRatePattern(client, _m(acc, 'delta')),
    dominance: createPercentPpmRatioPattern2(client, _m(acc, 'dominance')),
    total: createBtcCentsSatsUsdPattern(client, acc),
  };
}

/**
 * @typedef {Object} FloorLevelSupplyPattern
 * @property {Pct95Pct98Pct99Pattern} floor
 * @property {Pct10Pct20Pct30Pct40Pct50Pct60Pct70Pct80Pct90Pattern} level
 * @property {Pct95Pct98Pct99Pattern2} supplyInLossThreshold
 */

/**
 * Create a FloorLevelSupplyPattern pattern node
 * @param {BitviewClient} client
 * @param {string} acc - Accumulated series name
 * @returns {FloorLevelSupplyPattern}
 */
function createFloorLevelSupplyPattern(client, acc) {
  return {
    floor: createPct95Pct98Pct99Pattern(client, _m(acc, 'floor')),
    level: createPct10Pct20Pct30Pct40Pct50Pct60Pct70Pct80Pct90Pattern(client, _m(acc, 'level')),
    supplyInLossThreshold: createPct95Pct98Pct99Pattern2(client, _m(acc, 'supply_in_loss_threshold')),
  };
}

/**
 * @typedef {Object} InTotalPattern
 * @property {PercentPpmRatioPattern2} inLoss
 * @property {PercentPpmRatioPattern2} inProfit
 * @property {PercentPpmRatioPattern2} total
 */

/**
 * Create a InTotalPattern pattern node
 * @param {BitviewClient} client
 * @param {string} acc - Accumulated series name
 * @returns {InTotalPattern}
 */
function createInTotalPattern(client, acc) {
  return {
    inLoss: createPercentPpmRatioPattern2(client, _m(acc, 'in_loss')),
    inProfit: createPercentPpmRatioPattern2(client, _m(acc, 'in_profit')),
    total: createPercentPpmRatioPattern2(client, _m(acc, 'total')),
  };
}

/**
 * @typedef {Object} PercentPpmRatioPattern2
 * @property {SeriesPattern1<StoredF32>} percent
 * @property {SeriesPattern1<PartsPerMillion32>} ppm
 * @property {SeriesPattern1<StoredF32>} ratio
 */

/**
 * Create a PercentPpmRatioPattern2 pattern node
 * @param {BitviewClient} client
 * @param {string} acc - Accumulated series name
 * @returns {PercentPpmRatioPattern2}
 */
function createPercentPpmRatioPattern2(client, acc) {
  return {
    percent: createSeriesPattern1(client, acc),
    ppm: createSeriesPattern1(client, _m(acc, 'ppm')),
    ratio: createSeriesPattern1(client, _m(acc, 'ratio')),
  };
}

/**
 * @typedef {Object} PercentPpmRatioPattern5
 * @property {SeriesPattern1<StoredF32>} percent
 * @property {SeriesPattern1<PartsPerMillion64>} ppm
 * @property {SeriesPattern1<StoredF32>} ratio
 */

/**
 * Create a PercentPpmRatioPattern5 pattern node
 * @param {BitviewClient} client
 * @param {string} acc - Accumulated series name
 * @returns {PercentPpmRatioPattern5}
 */
function createPercentPpmRatioPattern5(client, acc) {
  return {
    percent: createSeriesPattern1(client, acc),
    ppm: createSeriesPattern1(client, _m(acc, 'ppm')),
    ratio: createSeriesPattern1(client, _m(acc, 'ratio')),
  };
}

/**
 * @typedef {Object} PercentPpmRatioPattern3
 * @property {SeriesPattern1<StoredF32>} percent
 * @property {SeriesPattern1<PartsPerMillionSigned32>} ppm
 * @property {SeriesPattern1<StoredF32>} ratio
 */

/**
 * Create a PercentPpmRatioPattern3 pattern node
 * @param {BitviewClient} client
 * @param {string} acc - Accumulated series name
 * @returns {PercentPpmRatioPattern3}
 */
function createPercentPpmRatioPattern3(client, acc) {
  return {
    percent: createSeriesPattern1(client, acc),
    ppm: createSeriesPattern1(client, _m(acc, 'ppm')),
    ratio: createSeriesPattern1(client, _m(acc, 'ratio')),
  };
}

/**
 * @typedef {Object} PercentPpmRatioPattern
 * @property {SeriesPattern1<StoredF32>} percent
 * @property {SeriesPattern1<PartsPerMillionSigned64>} ppm
 * @property {SeriesPattern1<StoredF32>} ratio
 */

/**
 * Create a PercentPpmRatioPattern pattern node
 * @param {BitviewClient} client
 * @param {string} acc - Accumulated series name
 * @returns {PercentPpmRatioPattern}
 */
function createPercentPpmRatioPattern(client, acc) {
  return {
    percent: createSeriesPattern1(client, acc),
    ppm: createSeriesPattern1(client, _m(acc, 'ppm')),
    ratio: createSeriesPattern1(client, _m(acc, 'ratio')),
  };
}

/**
 * @typedef {Object} PpmPriceRatioPattern
 * @property {SeriesPattern1<PartsPerMillion32>} ppm
 * @property {CentsSatsUsdPattern} price
 * @property {SeriesPattern1<StoredF32>} ratio
 */

/**
 * Create a PpmPriceRatioPattern pattern node
 * @param {BitviewClient} client
 * @param {string} acc - Accumulated series name
 * @param {string} disc - Discriminator suffix
 * @returns {PpmPriceRatioPattern}
 */
function createPpmPriceRatioPattern(client, acc, disc) {
  return {
    ppm: createSeriesPattern1(client, _m(acc, `ratio_${disc}_ppm`)),
    price: createCentsSatsUsdPattern(client, _m(acc, disc)),
    ratio: createSeriesPattern1(client, _m(_m(acc, 'ratio'), disc)),
  };
}

/**
 * @typedef {Object} RatioTransferValuePattern
 * @property {_1m1w1y24hPattern6<StoredF32>} ratio
 * @property {AverageBlockCumulativeSumPattern3} transferVolume
 * @property {AverageBlockCumulativeSumPattern3} valueDestroyed
 */

/**
 * Create a RatioTransferValuePattern pattern node
 * @param {BitviewClient} client
 * @param {string} acc - Accumulated series name
 * @returns {RatioTransferValuePattern}
 */
function createRatioTransferValuePattern(client, acc) {
  return {
    ratio: create_1m1w1y24hPattern6(client, _m(acc, 'adjusted_sopr')),
    transferVolume: createAverageBlockCumulativeSumPattern3(client, _m(acc, 'adj_value_created')),
    valueDestroyed: createAverageBlockCumulativeSumPattern3(client, _m(acc, 'adj_value_destroyed')),
  };
}

/**
 * @typedef {Object} RsiStochPattern
 * @property {PercentPpmRatioPattern2} rsi
 * @property {PercentPpmRatioPattern2} stochRsiD
 * @property {PercentPpmRatioPattern2} stochRsiK
 */

/**
 * Create a RsiStochPattern pattern node
 * @param {BitviewClient} client
 * @param {string} acc - Accumulated series name
 * @param {string} disc - Discriminator suffix
 * @returns {RsiStochPattern}
 */
function createRsiStochPattern(client, acc, disc) {
  return {
    rsi: createPercentPpmRatioPattern2(client, _m(acc, disc)),
    stochRsiD: createPercentPpmRatioPattern2(client, _m(_m(acc, 'stoch_d'), disc)),
    stochRsiK: createPercentPpmRatioPattern2(client, _m(_m(acc, 'stoch_k'), disc)),
  };
}

/**
 * @template T
 * @typedef {Object} _6bBlockTxPattern
 * @property {MaxMedianMinPct10Pct25Pct75Pct90Pattern<T>} _6b
 * @property {MaxMedianMinPct10Pct25Pct75Pct90Pattern<T>} block
 * @property {SeriesPattern19<T>} txIndex
 */

/**
 * Create a _6bBlockTxPattern pattern node
 * @template T
 * @param {BitviewClient} client
 * @param {string} acc - Accumulated series name
 * @returns {_6bBlockTxPattern<T>}
 */
function create_6bBlockTxPattern(client, acc) {
  return {
    _6b: createMaxMedianMinPct10Pct25Pct75Pct90Pattern(client, _m(acc, '6b')),
    block: createMaxMedianMinPct10Pct25Pct75Pct90Pattern(client, acc),
    txIndex: createSeriesPattern19(client, acc),
  };
}

/**
 * @typedef {Object} AbsoluteRatePattern
 * @property {_1m1w1y24hPattern2} absolute
 * @property {_1m1w1y24hPattern3} rate
 */

/**
 * Create a AbsoluteRatePattern pattern node
 * @param {BitviewClient} client
 * @param {string} acc - Accumulated series name
 * @returns {AbsoluteRatePattern}
 */
function createAbsoluteRatePattern(client, acc) {
  return {
    absolute: create_1m1w1y24hPattern2(client, acc),
    rate: create_1m1w1y24hPattern3(client, acc),
  };
}

/**
 * @typedef {Object} AbsoluteRatePattern2
 * @property {_1m1w1y24hPattern6<StoredI64>} absolute
 * @property {_1m1w1y24hPattern3} rate
 */

/**
 * Create a AbsoluteRatePattern2 pattern node
 * @param {BitviewClient} client
 * @param {string} acc - Accumulated series name
 * @returns {AbsoluteRatePattern2}
 */
function createAbsoluteRatePattern2(client, acc) {
  return {
    absolute: create_1m1w1y24hPattern6(client, acc),
    rate: create_1m1w1y24hPattern3(client, acc),
  };
}

/**
 * @typedef {Object} AbsoluteRatePattern3
 * @property {_1m1w1y24hPattern7} absolute
 * @property {_1m1w1y24hPattern3} rate
 */

/**
 * Create a AbsoluteRatePattern3 pattern node
 * @param {BitviewClient} client
 * @param {string} acc - Accumulated series name
 * @returns {AbsoluteRatePattern3}
 */
function createAbsoluteRatePattern3(client, acc) {
  return {
    absolute: create_1m1w1y24hPattern7(client, acc),
    rate: create_1m1w1y24hPattern3(client, acc),
  };
}

/**
 * @typedef {Object} AwakeDormantPattern2
 * @property {CapCapitalizedPriceSupplyPattern2} awake
 * @property {SupplyPattern} dormant
 */

/**
 * @typedef {Object} BaseDeltaPattern
 * @property {SeriesPattern1<StoredU64>} base
 * @property {AbsoluteRatePattern2} delta
 */

/**
 * Create a BaseDeltaPattern pattern node
 * @param {BitviewClient} client
 * @param {string} acc - Accumulated series name
 * @returns {BaseDeltaPattern}
 */
function createBaseDeltaPattern(client, acc) {
  return {
    base: createSeriesPattern1(client, acc),
    delta: createAbsoluteRatePattern2(client, _m(acc, 'delta')),
  };
}

/**
 * @typedef {Object} BlockCumulativePattern
 * @property {BtcCentsSatsUsdPattern3} block
 * @property {BtcCentsSatsUsdPattern} cumulative
 */

/**
 * Create a BlockCumulativePattern pattern node
 * @param {BitviewClient} client
 * @param {string} acc - Accumulated series name
 * @returns {BlockCumulativePattern}
 */
function createBlockCumulativePattern(client, acc) {
  return {
    block: createBtcCentsSatsUsdPattern3(client, acc),
    cumulative: createBtcCentsSatsUsdPattern(client, _m(acc, 'cumulative')),
  };
}

/**
 * @typedef {Object} BlocksDominancePattern
 * @property {BlockCumulativeSumPattern2} blocksMined
 * @property {PercentPpmRatioPattern2} dominance
 */

/**
 * Create a BlocksDominancePattern pattern node
 * @param {BitviewClient} client
 * @param {string} acc - Accumulated series name
 * @returns {BlocksDominancePattern}
 */
function createBlocksDominancePattern(client, acc) {
  return {
    blocksMined: createBlockCumulativeSumPattern2(client, _m(acc, 'blocks_mined')),
    dominance: createPercentPpmRatioPattern2(client, _m(acc, 'dominance')),
  };
}

/**
 * @typedef {Object} BpsRatioPattern
 * @property {SeriesPattern1<BasisPoints32>} bps
 * @property {SeriesPattern1<StoredF32>} ratio
 */

/**
 * Create a BpsRatioPattern pattern node
 * @param {BitviewClient} client
 * @param {string} acc - Accumulated series name
 * @returns {BpsRatioPattern}
 */
function createBpsRatioPattern(client, acc) {
  return {
    bps: createSeriesPattern1(client, _m(acc, 'bps')),
    ratio: createSeriesPattern1(client, acc),
  };
}

/**
 * @typedef {Object} BtcSatsPattern
 * @property {SeriesPattern1<Bitcoin>} btc
 * @property {SeriesPattern1<SatsSigned>} sats
 */

/**
 * Create a BtcSatsPattern pattern node
 * @param {BitviewClient} client
 * @param {string} acc - Accumulated series name
 * @returns {BtcSatsPattern}
 */
function createBtcSatsPattern(client, acc) {
  return {
    btc: createSeriesPattern1(client, acc),
    sats: createSeriesPattern1(client, _m(acc, 'sats')),
  };
}

/**
 * @typedef {Object} CentsUsdPattern
 * @property {SeriesPattern1<Cents>} cents
 * @property {SeriesPattern1<Dollars>} usd
 */

/**
 * Create a CentsUsdPattern pattern node
 * @param {BitviewClient} client
 * @param {string} acc - Accumulated series name
 * @returns {CentsUsdPattern}
 */
function createCentsUsdPattern(client, acc) {
  return {
    cents: createSeriesPattern1(client, _m(acc, 'cents')),
    usd: createSeriesPattern1(client, acc),
  };
}

/**
 * @typedef {Object} CentsUsdPattern2
 * @property {SeriesPattern18<Cents>} cents
 * @property {SeriesPattern18<Dollars>} usd
 */

/**
 * Create a CentsUsdPattern2 pattern node
 * @param {BitviewClient} client
 * @param {string} acc - Accumulated series name
 * @returns {CentsUsdPattern2}
 */
function createCentsUsdPattern2(client, acc) {
  return {
    cents: createSeriesPattern18(client, _m(acc, 'cents')),
    usd: createSeriesPattern18(client, acc),
  };
}

/**
 * @typedef {Object} CentsUsdPattern4
 * @property {SeriesPattern1<CentsSigned>} cents
 * @property {SeriesPattern1<Dollars>} usd
 */

/**
 * Create a CentsUsdPattern4 pattern node
 * @param {BitviewClient} client
 * @param {string} acc - Accumulated series name
 * @returns {CentsUsdPattern4}
 */
function createCentsUsdPattern4(client, acc) {
  return {
    cents: createSeriesPattern1(client, _m(acc, 'cents')),
    usd: createSeriesPattern1(client, acc),
  };
}

/**
 * @typedef {Object} CentsUsdPattern3
 * @property {SeriesPattern18<CentsSigned>} cents
 * @property {SeriesPattern18<Dollars>} usd
 */

/**
 * Create a CentsUsdPattern3 pattern node
 * @param {BitviewClient} client
 * @param {string} acc - Accumulated series name
 * @returns {CentsUsdPattern3}
 */
function createCentsUsdPattern3(client, acc) {
  return {
    cents: createSeriesPattern18(client, _m(acc, 'cents')),
    usd: createSeriesPattern18(client, acc),
  };
}

/**
 * @typedef {Object} CentsUsdPattern5
 * @property {SeriesPattern1<StoredF32>} cents
 * @property {SeriesPattern1<Dollars>} usd
 */

/**
 * Create a CentsUsdPattern5 pattern node
 * @param {BitviewClient} client
 * @param {string} acc - Accumulated series name
 * @returns {CentsUsdPattern5}
 */
function createCentsUsdPattern5(client, acc) {
  return {
    cents: createSeriesPattern1(client, _m(acc, 'cents')),
    usd: createSeriesPattern1(client, acc),
  };
}

/**
 * @typedef {Object} FundedTotalPattern
 * @property {AllP2aP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshPattern4} funded
 * @property {AllP2aP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshPattern4} total
 */

/**
 * Create a FundedTotalPattern pattern node
 * @param {BitviewClient} client
 * @param {string} acc - Accumulated series name
 * @returns {FundedTotalPattern}
 */
function createFundedTotalPattern(client, acc) {
  return {
    funded: createAllP2aP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshPattern4(client, acc),
    total: createAllP2aP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshPattern4(client, _p('total', acc)),
  };
}

/**
 * @typedef {Object} ImmobileMobilePattern2
 * @property {BtcCentsSatsUsdPattern} immobile
 * @property {BtcCentsInSatsUsdPattern} mobile
 */

/**
 * @typedef {Object} MaxMinPattern
 * @property {CentsSatsUsdPattern} max
 * @property {CentsSatsUsdPattern} min
 */

/**
 * Create a MaxMinPattern pattern node
 * @param {BitviewClient} client
 * @param {string} acc - Accumulated series name
 * @returns {MaxMinPattern}
 */
function createMaxMinPattern(client, acc) {
  return {
    max: createCentsSatsUsdPattern(client, _m(acc, 'max')),
    min: createCentsSatsUsdPattern(client, _m(acc, 'min')),
  };
}

/**
 * @typedef {Object} PerPattern2
 * @property {CentsSatsUsdPattern} perCoin
 * @property {CentsSatsUsdPattern} perDollar
 */

/**
 * Create a PerPattern2 pattern node
 * @param {BitviewClient} client
 * @param {string} acc - Accumulated series name
 * @returns {PerPattern2}
 */
function createPerPattern2(client, acc) {
  return {
    perCoin: createCentsSatsUsdPattern(client, _m(acc, 'coin')),
    perDollar: createCentsSatsUsdPattern(client, _m(acc, 'dollar')),
  };
}

/**
 * @typedef {Object} PerPattern
 * @property {Pct05Pct10Pct15Pct20Pct25Pct30Pct35Pct40Pct45Pct50Pct55Pct60Pct65Pct70Pct75Pct80Pct85Pct90Pct95Pattern} perCoin
 * @property {Pct05Pct10Pct15Pct20Pct25Pct30Pct35Pct40Pct45Pct50Pct55Pct60Pct65Pct70Pct75Pct80Pct85Pct90Pct95Pattern} perDollar
 */

/**
 * Create a PerPattern pattern node
 * @param {BitviewClient} client
 * @param {string} acc - Accumulated series name
 * @returns {PerPattern}
 */
function createPerPattern(client, acc) {
  return {
    perCoin: createPct05Pct10Pct15Pct20Pct25Pct30Pct35Pct40Pct45Pct50Pct55Pct60Pct65Pct70Pct75Pct80Pct85Pct90Pct95Pattern(client, _m(acc, 'coin')),
    perDollar: createPct05Pct10Pct15Pct20Pct25Pct30Pct35Pct40Pct45Pct50Pct55Pct60Pct65Pct70Pct75Pct80Pct85Pct90Pct95Pattern(client, _m(acc, 'dollar')),
  };
}

/**
 * @typedef {Object} PpmRatioPattern
 * @property {SeriesPattern1<PartsPerMillion32>} ppm
 * @property {SeriesPattern1<StoredF32>} ratio
 */

/**
 * Create a PpmRatioPattern pattern node
 * @param {BitviewClient} client
 * @param {string} acc - Accumulated series name
 * @returns {PpmRatioPattern}
 */
function createPpmRatioPattern(client, acc) {
  return {
    ppm: createSeriesPattern1(client, _m(acc, 'ppm')),
    ratio: createSeriesPattern1(client, acc),
  };
}

/**
 * @typedef {Object} PpmRatioPattern3
 * @property {SeriesPattern1<PartsPerMillionSigned32>} ppm
 * @property {SeriesPattern1<StoredF32>} ratio
 */

/**
 * Create a PpmRatioPattern3 pattern node
 * @param {BitviewClient} client
 * @param {string} acc - Accumulated series name
 * @returns {PpmRatioPattern3}
 */
function createPpmRatioPattern3(client, acc) {
  return {
    ppm: createSeriesPattern1(client, _m(acc, 'ppm')),
    ratio: createSeriesPattern1(client, acc),
  };
}

/**
 * @typedef {Object} PpmRatioPattern2
 * @property {SeriesPattern1<PriceRatio>} ppm
 * @property {SeriesPattern1<StoredF32>} ratio
 */

/**
 * Create a PpmRatioPattern2 pattern node
 * @param {BitviewClient} client
 * @param {string} acc - Accumulated series name
 * @returns {PpmRatioPattern2}
 */
function createPpmRatioPattern2(client, acc) {
  return {
    ppm: createSeriesPattern1(client, _m(acc, 'ppm')),
    ratio: createSeriesPattern1(client, acc),
  };
}

/**
 * @typedef {Object} SdSmaPattern
 * @property {SeriesPattern1<StoredF32>} sd
 * @property {SeriesPattern1<StoredF32>} sma
 */

/**
 * @typedef {Object} SpentUnspentPattern
 * @property {AverageBlockCumulativeSumPattern2<StoredU64>} spentCount
 * @property {BaseDeltaPattern} unspentCount
 */

/**
 * Create a SpentUnspentPattern pattern node
 * @param {BitviewClient} client
 * @param {string} acc - Accumulated series name
 * @returns {SpentUnspentPattern}
 */
function createSpentUnspentPattern(client, acc) {
  return {
    spentCount: createAverageBlockCumulativeSumPattern2(client, _m(acc, 'spent_utxo_count')),
    unspentCount: createBaseDeltaPattern(client, _m(acc, 'utxo_count')),
  };
}

/**
 * @typedef {Object} SharePattern
 * @property {SeriesPattern1<StoredF64>} share
 */

/**
 * Create a SharePattern pattern node
 * @param {BitviewClient} client
 * @param {string} acc - Accumulated series name
 * @returns {SharePattern}
 */
function createSharePattern(client, acc) {
  return {
    share: createSeriesPattern1(client, acc),
  };
}

/**
 * @typedef {Object} SupplyPattern
 * @property {BtcCentsSatsUsdPattern} supply
 */

/**
 * Create a SupplyPattern pattern node
 * @param {BitviewClient} client
 * @param {string} acc - Accumulated series name
 * @returns {SupplyPattern}
 */
function createSupplyPattern(client, acc) {
  return {
    supply: createBtcCentsSatsUsdPattern(client, acc),
  };
}

// Catalog tree typedefs

/**
 * @typedef {Object} SeriesTree
 * @property {SeriesTree_Blocks} blocks
 * @property {SeriesTree_Transactions} transactions
 * @property {SeriesTree_Inputs} inputs
 * @property {SeriesTree_Outputs} outputs
 * @property {SeriesTree_Addrs} addrs
 * @property {SeriesTree_Scripts} scripts
 * @property {SeriesTree_OpReturn} opReturn
 * @property {SeriesTree_Mining} mining
 * @property {SeriesTree_Cointime} cointime
 * @property {SeriesTree_Coinflow} coinflow
 * @property {SeriesTree_Bedrock} bedrock
 * @property {SeriesTree_CapitalSentiment} capitalSentiment
 * @property {SeriesTree_RarityMeter} rarityMeter
 * @property {SeriesTree_Constants} constants
 * @property {SeriesTree_Mappings} mappings
 * @property {SeriesTree_Indicators} indicators
 * @property {SeriesTree_Market} market
 * @property {SeriesTree_Pools} pools
 * @property {SeriesTree_Price} price
 * @property {SeriesTree_Cohorts} cohorts
 * @property {SeriesTree_DistributionAggregated} distributionAggregated
 * @property {SeriesTree_Supply} supply
 * @property {SeriesTree_UtxoHistory} utxoHistory
 */

/**
 * @typedef {Object} SeriesTree_Blocks
 * @property {SeriesPattern18<BlockHash>} blockhash
 * @property {SeriesPattern18<CoinbaseTag>} coinbaseTag
 * @property {SeriesTree_Blocks_Difficulty} difficulty
 * @property {SeriesTree_Blocks_Time} time
 * @property {SeriesTree_Blocks_Size} size
 * @property {SeriesTree_Blocks_Weight} weight
 * @property {SeriesPattern18<StoredU32>} segwitTxs
 * @property {SeriesPattern18<StoredU64>} segwitSize
 * @property {SeriesPattern18<Weight>} segwitWeight
 * @property {SeriesTree_Blocks_Count} count
 * @property {SeriesTree_Blocks_Lookback} lookback
 * @property {SeriesTree_Blocks_Interval} interval
 * @property {AverageBlockCumulativeMaxMedianMinPct10Pct25Pct75Pct90SumPattern} vbytes
 * @property {SeriesTree_Blocks_Fullness} fullness
 * @property {SeriesTree_Blocks_Halving} halving
 */

/**
 * @typedef {Object} SeriesTree_Blocks_Difficulty
 * @property {SeriesPattern1<StoredF64>} value
 * @property {SeriesPattern1<StoredF64>} hashrate
 * @property {PercentPpmRatioPattern3} adjustment
 * @property {SeriesPattern1<Epoch>} epoch
 * @property {SeriesPattern1<StoredU32>} blocksToRetarget
 * @property {SeriesPattern1<StoredF32>} daysToRetarget
 */

/**
 * @typedef {Object} SeriesTree_Blocks_Time
 * @property {SeriesPattern18<Timestamp>} timestamp
 */

/**
 * @typedef {Object} SeriesTree_Blocks_Size
 * @property {SeriesPattern18<StoredU64>} base
 * @property {SeriesPattern1<StoredU64>} cumulative
 * @property {_1m1w1y24hPattern6<StoredU64>} sum
 * @property {_1m1w1y24hPattern6<StoredF32>} average
 * @property {_1m1w1y24hPattern6<StoredU64>} min
 * @property {_1m1w1y24hPattern6<StoredU64>} max
 * @property {_1m1w1y24hPattern6<StoredU64>} pct10
 * @property {_1m1w1y24hPattern6<StoredU64>} pct25
 * @property {_1m1w1y24hPattern6<StoredU64>} median
 * @property {_1m1w1y24hPattern6<StoredU64>} pct75
 * @property {_1m1w1y24hPattern6<StoredU64>} pct90
 */

/**
 * @typedef {Object} SeriesTree_Blocks_Weight
 * @property {SeriesPattern18<Weight>} base
 * @property {SeriesPattern1<Weight64>} cumulative
 * @property {_1m1w1y24hPattern6<Weight64>} sum
 * @property {_1m1w1y24hPattern6<StoredF32>} average
 * @property {_1m1w1y24hPattern6<Weight64>} min
 * @property {_1m1w1y24hPattern6<Weight64>} max
 * @property {_1m1w1y24hPattern6<Weight64>} pct10
 * @property {_1m1w1y24hPattern6<Weight64>} pct25
 * @property {_1m1w1y24hPattern6<Weight64>} median
 * @property {_1m1w1y24hPattern6<Weight64>} pct75
 * @property {_1m1w1y24hPattern6<Weight64>} pct90
 */

/**
 * @typedef {Object} SeriesTree_Blocks_Count
 * @property {_1m1w1y24hPattern6<StoredU64>} target
 * @property {AverageBlockCumulativeSumPattern2<StoredU64>} total
 */

/**
 * @typedef {Object} SeriesTree_Blocks_Lookback
 * @property {SeriesPattern18<Height>} _1h
 * @property {SeriesPattern18<Height>} _24h
 * @property {SeriesPattern18<Height>} _3d
 * @property {SeriesPattern18<Height>} _1w
 * @property {SeriesPattern18<Height>} _8d
 * @property {SeriesPattern18<Height>} _9d
 * @property {SeriesPattern18<Height>} _12d
 * @property {SeriesPattern18<Height>} _13d
 * @property {SeriesPattern18<Height>} _2w
 * @property {SeriesPattern18<Height>} _21d
 * @property {SeriesPattern18<Height>} _26d
 * @property {SeriesPattern18<Height>} _1m
 * @property {SeriesPattern18<Height>} _34d
 * @property {SeriesPattern18<Height>} _50d
 * @property {SeriesPattern18<Height>} _55d
 * @property {SeriesPattern18<Height>} _2m
 * @property {SeriesPattern18<Height>} _9w
 * @property {SeriesPattern18<Height>} _12w
 * @property {SeriesPattern18<Height>} _89d
 * @property {SeriesPattern18<Height>} _3m
 * @property {SeriesPattern18<Height>} _14w
 * @property {SeriesPattern18<Height>} _111d
 * @property {SeriesPattern18<Height>} _144d
 * @property {SeriesPattern18<Height>} _6m
 * @property {SeriesPattern18<Height>} _26w
 * @property {SeriesPattern18<Height>} _200d
 * @property {SeriesPattern18<Height>} _9m
 * @property {SeriesPattern18<Height>} _350d
 * @property {SeriesPattern18<Height>} _12m
 * @property {SeriesPattern18<Height>} _1y
 * @property {SeriesPattern18<Height>} _14m
 * @property {SeriesPattern18<Height>} _2y
 * @property {SeriesPattern18<Height>} _26m
 * @property {SeriesPattern18<Height>} _3y
 * @property {SeriesPattern18<Height>} _200w
 * @property {SeriesPattern18<Height>} _4y
 * @property {SeriesPattern18<Height>} _5y
 * @property {SeriesPattern18<Height>} _6y
 * @property {SeriesPattern18<Height>} _8y
 * @property {SeriesPattern18<Height>} _9y
 * @property {SeriesPattern18<Height>} _10y
 * @property {SeriesPattern18<Height>} _12y
 * @property {SeriesPattern18<Height>} _14y
 * @property {SeriesPattern18<Height>} _26y
 */

/**
 * @typedef {Object} SeriesTree_Blocks_Interval
 * @property {SeriesPattern18<Timestamp>} block
 * @property {SeriesPattern1<StoredF32>} _24h
 * @property {SeriesPattern1<StoredF32>} _1w
 * @property {SeriesPattern1<StoredF32>} _1m
 * @property {SeriesPattern1<StoredF32>} _1y
 */

/**
 * @typedef {Object} SeriesTree_Blocks_Fullness
 * @property {SeriesPattern18<PartsPerMillion32>} ppm
 * @property {SeriesPattern18<StoredF32>} ratio
 * @property {SeriesPattern18<StoredF32>} percent
 */

/**
 * @typedef {Object} SeriesTree_Blocks_Halving
 * @property {SeriesPattern1<Halving>} epoch
 * @property {SeriesPattern1<StoredU32>} blocksToHalving
 * @property {SeriesPattern1<StoredF32>} daysToHalving
 */

/**
 * @typedef {Object} SeriesTree_Transactions
 * @property {SeriesTree_Transactions_Raw} raw
 * @property {SeriesTree_Transactions_Features} features
 * @property {SeriesTree_Transactions_Count} count
 * @property {SeriesTree_Transactions_Size} size
 * @property {SeriesTree_Transactions_Fees} fees
 * @property {SeriesTree_Transactions_Inscription} inscription
 * @property {SeriesTree_Transactions_Patterns} patterns
 * @property {SeriesTree_Transactions_Policy} policy
 * @property {SeriesTree_Transactions_Sigops} sigops
 * @property {SeriesTree_Transactions_Versions} versions
 * @property {SeriesTree_Transactions_Volume} volume
 */

/**
 * @typedef {Object} SeriesTree_Transactions_Raw
 * @property {SeriesPattern18<TxIndex>} firstTxIndex
 * @property {SeriesPattern19<Txid>} txid
 * @property {SeriesPattern19<TxVersion>} txVersion
 * @property {SeriesPattern19<RawLockTime>} rawLocktime
 * @property {SeriesPattern19<Weight>} weight
 * @property {SeriesPattern19<StoredU32>} totalSize
 * @property {SeriesPattern19<SigOps>} totalSigopCost
 * @property {SeriesPattern19<StoredBool>} isExplicitlyRbf
 * @property {SeriesPattern19<TxInIndex>} firstTxinIndex
 * @property {SeriesPattern19<TxOutIndex>} firstTxoutIndex
 */

/**
 * @typedef {Object} SeriesTree_Transactions_Features
 * @property {SeriesTree_Transactions_Features_Count} count
 * @property {SeriesPattern19<StoredBool>} hasP2pk
 * @property {SeriesPattern19<StoredBool>} hasP2ms
 * @property {SeriesPattern19<StoredBool>} hasP2pkh
 * @property {SeriesPattern19<StoredBool>} hasP2sh
 * @property {SeriesPattern19<StoredBool>} hasP2wpkh
 * @property {SeriesPattern19<StoredBool>} hasP2wsh
 * @property {SeriesPattern19<StoredBool>} hasP2tr
 * @property {SeriesPattern19<StoredBool>} hasP2a
 * @property {SeriesPattern19<StoredBool>} hasOpReturn
 * @property {SeriesPattern19<StoredBool>} hasEmpty
 * @property {SeriesPattern19<StoredBool>} hasUnknown
 * @property {SeriesPattern19<StoredBool>} hasFakePubkey
 * @property {SeriesPattern19<StoredBool>} hasFakeScripthash
 * @property {SeriesPattern19<StoredBool>} hasInscription
 * @property {SeriesPattern19<StoredBool>} hasAnnex
 * @property {SeriesPattern19<StoredBool>} hasSighashAll
 * @property {SeriesPattern19<StoredBool>} hasSighashNone
 * @property {SeriesPattern19<StoredBool>} hasSighashSingle
 * @property {SeriesPattern19<StoredBool>} hasSighashDefault
 * @property {SeriesPattern19<StoredBool>} hasSighashAnyoneCanPay
 * @property {SeriesPattern19<StoredBool>} hasDustOutput
 */

/**
 * @typedef {Object} SeriesTree_Transactions_Features_Count
 * @property {SeriesPattern18<StoredU64>} v1
 * @property {SeriesPattern18<StoredU64>} v2
 * @property {SeriesPattern18<StoredU64>} v3
 * @property {SeriesPattern18<StoredU64>} otherVersion
 * @property {SeriesPattern18<StoredU64>} explicitlyRbf
 * @property {SeriesPattern18<StoredU64>} oneInput
 * @property {SeriesPattern18<StoredU64>} oneOutput
 * @property {SeriesPattern18<StoredU64>} p2pk
 * @property {SeriesPattern18<StoredU64>} p2ms
 * @property {SeriesPattern18<StoredU64>} p2pkh
 * @property {SeriesPattern18<StoredU64>} p2sh
 * @property {SeriesPattern18<StoredU64>} p2wpkh
 * @property {SeriesPattern18<StoredU64>} p2wsh
 * @property {SeriesPattern18<StoredU64>} p2tr
 * @property {SeriesPattern18<StoredU64>} p2a
 * @property {SeriesPattern18<StoredU64>} opReturn
 * @property {SeriesPattern18<StoredU64>} empty
 * @property {SeriesPattern18<StoredU64>} unknown
 * @property {SeriesPattern18<StoredU64>} fakePubkey
 * @property {SeriesPattern18<StoredU64>} fakeScripthash
 * @property {AverageBlockCumulativeSumPattern2<StoredU64>} annex
 * @property {AverageBlockCumulativeSumPattern2<StoredU64>} sighashAll
 * @property {AverageBlockCumulativeSumPattern2<StoredU64>} sighashNone
 * @property {AverageBlockCumulativeSumPattern2<StoredU64>} sighashSingle
 * @property {AverageBlockCumulativeSumPattern2<StoredU64>} sighashDefault
 * @property {AverageBlockCumulativeSumPattern2<StoredU64>} sighashAnyoneCanPay
 * @property {AverageBlockCumulativeSumPattern2<StoredU64>} dustOutput
 */

/**
 * @typedef {Object} SeriesTree_Transactions_Count
 * @property {AverageBlockCumulativeMaxMedianMinPct10Pct25Pct75Pct90SumPattern} total
 */

/**
 * @typedef {Object} SeriesTree_Transactions_Size
 * @property {SeriesTree_Transactions_Size_Vsize} vsize
 * @property {SeriesTree_Transactions_Size_Weight} weight
 */

/**
 * @typedef {Object} SeriesTree_Transactions_Size_Vsize
 * @property {SeriesPattern19<VSize>} txIndex
 * @property {MaxMedianMinPct10Pct25Pct75Pct90Pattern2} block
 * @property {MaxMedianMinPct10Pct25Pct75Pct90Pattern2} _6b
 */

/**
 * @typedef {Object} SeriesTree_Transactions_Size_Weight
 * @property {MaxMedianMinPct10Pct25Pct75Pct90Pattern<Weight>} block
 * @property {MaxMedianMinPct10Pct25Pct75Pct90Pattern<Weight>} _6b
 */

/**
 * @typedef {Object} SeriesTree_Transactions_Fees
 * @property {SeriesTree_Transactions_Fees_Count} count
 * @property {_6bBlockTxPattern<Sats>} fee
 * @property {SeriesPattern19<FeeRate>} feeRate
 * @property {_6bBlockTxPattern<FeeRate>} effectiveFeeRate
 * @property {SeriesPattern19<StoredBool>} isCpfpParent
 * @property {SeriesPattern19<StoredBool>} isCpfpChild
 */

/**
 * @typedef {Object} SeriesTree_Transactions_Fees_Count
 * @property {AverageBlockCumulativeSumPattern2<StoredU64>} cpfpParent
 * @property {AverageBlockCumulativeSumPattern2<StoredU64>} cpfpChild
 */

/**
 * @typedef {Object} SeriesTree_Transactions_Inscription
 * @property {AverageBlockCumulativeSumPattern2<StoredU64>} count
 * @property {AverageBlockCumulativeSumPattern2<Sats>} fees
 * @property {PercentPpmRatioPattern2} feeShare
 */

/**
 * @typedef {Object} SeriesTree_Transactions_Patterns
 * @property {SeriesTree_Transactions_Patterns_Count} count
 * @property {SeriesPattern19<StoredBool>} isCoinjoin
 * @property {SeriesPattern19<StoredBool>} isConsolidation
 * @property {SeriesPattern19<StoredBool>} isBatchPayout
 */

/**
 * @typedef {Object} SeriesTree_Transactions_Patterns_Count
 * @property {AverageBlockCumulativeSumPattern2<StoredU64>} coinjoin
 * @property {AverageBlockCumulativeSumPattern2<StoredU64>} consolidation
 * @property {AverageBlockCumulativeSumPattern2<StoredU64>} batchPayout
 */

/**
 * @typedef {Object} SeriesTree_Transactions_Policy
 * @property {SeriesTree_Transactions_Policy_Count} count
 * @property {SeriesPattern19<StoredBool>} isNonstandard
 */

/**
 * @typedef {Object} SeriesTree_Transactions_Policy_Count
 * @property {AverageBlockCumulativeSumPattern2<StoredU64>} nonstandard
 */

/**
 * @typedef {Object} SeriesTree_Transactions_Sigops
 * @property {AverageBlockCumulativeSumPattern2<StoredU64>} total
 */

/**
 * @typedef {Object} SeriesTree_Transactions_Versions
 * @property {AverageBlockCumulativeSumPattern2<StoredU64>} v1
 * @property {AverageBlockCumulativeSumPattern2<StoredU64>} v2
 * @property {AverageBlockCumulativeSumPattern2<StoredU64>} v3
 * @property {AverageBlockCumulativeSumPattern2<StoredU64>} other
 */

/**
 * @typedef {Object} SeriesTree_Transactions_Volume
 * @property {AverageBlockCumulativeSumPattern} transferVolume
 * @property {_1m1w1y24hPattern6<StoredF32>} txPerSec
 */

/**
 * @typedef {Object} SeriesTree_Inputs
 * @property {SeriesTree_Inputs_Raw} raw
 * @property {SeriesPattern20<Sats>} value
 * @property {CumulativeRollingSumPattern} count
 * @property {_1m1w1y24hPattern6<StoredF32>} perSec
 * @property {SeriesTree_Inputs_ByType} byType
 */

/**
 * @typedef {Object} SeriesTree_Inputs_Raw
 * @property {SeriesPattern18<TxInIndex>} firstTxinIndex
 * @property {SeriesPattern20<OutPoint>} outpoint
 * @property {SeriesPattern20<TxOutIndex>} txoutIndex
 * @property {SeriesPattern20<TxIndex>} txIndex
 * @property {SeriesPattern20<OutputType>} outputType
 * @property {SeriesPattern20<TypeIndex>} typeIndex
 */

/**
 * @typedef {Object} SeriesTree_Inputs_ByType
 * @property {SeriesTree_Inputs_ByType_InputCount} inputCount
 * @property {SeriesTree_Inputs_ByType_InputShare} inputShare
 * @property {SeriesTree_Inputs_ByType_TxCount} txCount
 * @property {EmptyP2aP2msP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshUnknownPattern10} txShare
 */

/**
 * @typedef {Object} SeriesTree_Inputs_ByType_InputCount
 * @property {AverageBlockCumulativeSumPattern2<StoredU64>} all
 * @property {AverageBlockCumulativeSumPattern2<StoredU64>} p2pk65
 * @property {AverageBlockCumulativeSumPattern2<StoredU64>} p2pk33
 * @property {AverageBlockCumulativeSumPattern2<StoredU64>} p2pkh
 * @property {AverageBlockCumulativeSumPattern2<StoredU64>} p2ms
 * @property {AverageBlockCumulativeSumPattern2<StoredU64>} p2sh
 * @property {AverageBlockCumulativeSumPattern2<StoredU64>} p2wpkh
 * @property {AverageBlockCumulativeSumPattern2<StoredU64>} p2wsh
 * @property {AverageBlockCumulativeSumPattern2<StoredU64>} p2tr
 * @property {AverageBlockCumulativeSumPattern2<StoredU64>} p2a
 * @property {AverageBlockCumulativeSumPattern2<StoredU64>} unknown
 * @property {AverageBlockCumulativeSumPattern2<StoredU64>} empty
 */

/**
 * @typedef {Object} SeriesTree_Inputs_ByType_InputShare
 * @property {_1m1w1y24hPercentPpmRatioPattern} p2pk65
 * @property {_1m1w1y24hPercentPpmRatioPattern} p2pk33
 * @property {_1m1w1y24hPercentPpmRatioPattern} p2pkh
 * @property {_1m1w1y24hPercentPpmRatioPattern} p2ms
 * @property {_1m1w1y24hPercentPpmRatioPattern} p2sh
 * @property {_1m1w1y24hPercentPpmRatioPattern} p2wpkh
 * @property {_1m1w1y24hPercentPpmRatioPattern} p2wsh
 * @property {_1m1w1y24hPercentPpmRatioPattern} p2tr
 * @property {_1m1w1y24hPercentPpmRatioPattern} p2a
 * @property {_1m1w1y24hPercentPpmRatioPattern} unknown
 * @property {_1m1w1y24hPercentPpmRatioPattern} empty
 */

/**
 * @typedef {Object} SeriesTree_Inputs_ByType_TxCount
 * @property {AverageBlockCumulativeSumPattern2<StoredU64>} all
 * @property {AverageBlockCumulativeSumPattern2<StoredU64>} p2pk65
 * @property {AverageBlockCumulativeSumPattern2<StoredU64>} p2pk33
 * @property {AverageBlockCumulativeSumPattern2<StoredU64>} p2pkh
 * @property {AverageBlockCumulativeSumPattern2<StoredU64>} p2ms
 * @property {AverageBlockCumulativeSumPattern2<StoredU64>} p2sh
 * @property {AverageBlockCumulativeSumPattern2<StoredU64>} p2wpkh
 * @property {AverageBlockCumulativeSumPattern2<StoredU64>} p2wsh
 * @property {AverageBlockCumulativeSumPattern2<StoredU64>} p2tr
 * @property {AverageBlockCumulativeSumPattern2<StoredU64>} p2a
 * @property {AverageBlockCumulativeSumPattern2<StoredU64>} unknown
 * @property {AverageBlockCumulativeSumPattern2<StoredU64>} empty
 */

/**
 * @typedef {Object} SeriesTree_Outputs
 * @property {SeriesTree_Outputs_Raw} raw
 * @property {SeriesTree_Outputs_Spent} spent
 * @property {SeriesTree_Outputs_Count} count
 * @property {_1m1w1y24hPattern6<StoredF32>} perSec
 * @property {SeriesTree_Outputs_ByType} byType
 * @property {SeriesTree_Outputs_Value} value
 */

/**
 * @typedef {Object} SeriesTree_Outputs_Raw
 * @property {SeriesPattern18<TxOutIndex>} firstTxoutIndex
 * @property {SeriesPattern21<Sats>} value
 * @property {SeriesPattern21<OutputType>} outputType
 * @property {SeriesPattern21<TypeIndex>} typeIndex
 */

/**
 * @typedef {Object} SeriesTree_Outputs_Spent
 * @property {SeriesPattern21<TxInIndex>} txinIndex
 */

/**
 * @typedef {Object} SeriesTree_Outputs_Count
 * @property {CumulativeRollingSumPattern} total
 */

/**
 * @typedef {Object} SeriesTree_Outputs_ByType
 * @property {SeriesTree_Outputs_ByType_OutputCount} outputCount
 * @property {AverageBlockCumulativeSumPattern2<StoredU64>} spendableOutputCount
 * @property {SeriesTree_Outputs_ByType_OutputShare} outputShare
 * @property {AllEmptyOpP2aP2msP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshUnknownPattern} txCount
 * @property {EmptyOpP2aP2msP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshUnknownPattern2} txShare
 */

/**
 * @typedef {Object} SeriesTree_Outputs_ByType_OutputCount
 * @property {AverageBlockCumulativeSumPattern2<StoredU64>} all
 * @property {AverageBlockCumulativeSumPattern2<StoredU64>} p2pk65
 * @property {AverageBlockCumulativeSumPattern2<StoredU64>} p2pk33
 * @property {AverageBlockCumulativeSumPattern2<StoredU64>} p2pkh
 * @property {AverageBlockCumulativeSumPattern2<StoredU64>} p2ms
 * @property {AverageBlockCumulativeSumPattern2<StoredU64>} p2sh
 * @property {AverageBlockCumulativeSumPattern2<StoredU64>} p2wpkh
 * @property {AverageBlockCumulativeSumPattern2<StoredU64>} p2wsh
 * @property {AverageBlockCumulativeSumPattern2<StoredU64>} p2tr
 * @property {AverageBlockCumulativeSumPattern2<StoredU64>} p2a
 * @property {AverageBlockCumulativeSumPattern2<StoredU64>} unknown
 * @property {AverageBlockCumulativeSumPattern2<StoredU64>} empty
 * @property {AverageBlockCumulativeSumPattern2<StoredU64>} opReturn
 */

/**
 * @typedef {Object} SeriesTree_Outputs_ByType_OutputShare
 * @property {_1m1w1y24hPercentPpmRatioPattern} p2pk65
 * @property {_1m1w1y24hPercentPpmRatioPattern} p2pk33
 * @property {_1m1w1y24hPercentPpmRatioPattern} p2pkh
 * @property {_1m1w1y24hPercentPpmRatioPattern} p2ms
 * @property {_1m1w1y24hPercentPpmRatioPattern} p2sh
 * @property {_1m1w1y24hPercentPpmRatioPattern} p2wpkh
 * @property {_1m1w1y24hPercentPpmRatioPattern} p2wsh
 * @property {_1m1w1y24hPercentPpmRatioPattern} p2tr
 * @property {_1m1w1y24hPercentPpmRatioPattern} p2a
 * @property {_1m1w1y24hPercentPpmRatioPattern} unknown
 * @property {_1m1w1y24hPercentPpmRatioPattern} empty
 * @property {_1m1w1y24hPercentPpmRatioPattern} opReturn
 */

/**
 * @typedef {Object} SeriesTree_Outputs_Value
 * @property {BlockCumulativePattern} opReturn
 */

/**
 * @typedef {Object} SeriesTree_Addrs
 * @property {SeriesTree_Addrs_Raw} raw
 * @property {SeriesTree_Addrs_State} state
 * @property {SeriesTree_Addrs_ByBalance} byBalance
 * @property {SeriesTree_Addrs_Funded} funded
 * @property {AllP2aP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshPattern4} empty
 * @property {SeriesTree_Addrs_Activity} activity
 * @property {AllP2aP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshPattern4} total
 * @property {AllP2aP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshPattern6} new
 * @property {SeriesTree_Addrs_Reused} reused
 * @property {SeriesTree_Addrs_Respent} respent
 * @property {SeriesTree_Addrs_Exposed} exposed
 * @property {SeriesTree_Addrs_Delta} delta
 * @property {SeriesTree_Addrs_AvgBalance} avgBalance
 */

/**
 * @typedef {Object} SeriesTree_Addrs_Raw
 * @property {SeriesTree_Addrs_Raw_P2pk65} p2pk65
 * @property {SeriesTree_Addrs_Raw_P2pk33} p2pk33
 * @property {SeriesTree_Addrs_Raw_P2pkh} p2pkh
 * @property {SeriesTree_Addrs_Raw_P2sh} p2sh
 * @property {SeriesTree_Addrs_Raw_P2wpkh} p2wpkh
 * @property {SeriesTree_Addrs_Raw_P2wsh} p2wsh
 * @property {SeriesTree_Addrs_Raw_P2tr} p2tr
 * @property {SeriesTree_Addrs_Raw_P2a} p2a
 */

/**
 * @typedef {Object} SeriesTree_Addrs_Raw_P2pk65
 * @property {SeriesPattern18<P2PK65AddrIndex>} firstIndex
 * @property {SeriesPattern27<P2PK65Bytes>} bytes
 */

/**
 * @typedef {Object} SeriesTree_Addrs_Raw_P2pk33
 * @property {SeriesPattern18<P2PK33AddrIndex>} firstIndex
 * @property {SeriesPattern26<P2PK33Bytes>} bytes
 */

/**
 * @typedef {Object} SeriesTree_Addrs_Raw_P2pkh
 * @property {SeriesPattern18<P2PKHAddrIndex>} firstIndex
 * @property {SeriesPattern28<P2PKHBytes>} bytes
 */

/**
 * @typedef {Object} SeriesTree_Addrs_Raw_P2sh
 * @property {SeriesPattern18<P2SHAddrIndex>} firstIndex
 * @property {SeriesPattern29<P2SHBytes>} bytes
 */

/**
 * @typedef {Object} SeriesTree_Addrs_Raw_P2wpkh
 * @property {SeriesPattern18<P2WPKHAddrIndex>} firstIndex
 * @property {SeriesPattern31<P2WPKHBytes>} bytes
 */

/**
 * @typedef {Object} SeriesTree_Addrs_Raw_P2wsh
 * @property {SeriesPattern18<P2WSHAddrIndex>} firstIndex
 * @property {SeriesPattern32<P2WSHBytes>} bytes
 */

/**
 * @typedef {Object} SeriesTree_Addrs_Raw_P2tr
 * @property {SeriesPattern18<P2TRAddrIndex>} firstIndex
 * @property {SeriesPattern30<P2TRBytes>} bytes
 */

/**
 * @typedef {Object} SeriesTree_Addrs_Raw_P2a
 * @property {SeriesPattern18<P2AAddrIndex>} firstIndex
 * @property {SeriesPattern24<P2ABytes>} bytes
 */

/**
 * @typedef {Object} SeriesTree_Addrs_State
 * @property {SeriesPattern24<AddrState>} p2a
 * @property {SeriesPattern26<AddrState>} p2pk33
 * @property {SeriesPattern27<AddrState>} p2pk65
 * @property {SeriesPattern28<AddrState>} p2pkh
 * @property {SeriesPattern29<AddrState>} p2sh
 * @property {SeriesPattern30<AddrState>} p2tr
 * @property {SeriesPattern31<AddrState>} p2wpkh
 * @property {SeriesPattern32<AddrState>} p2wsh
 * @property {SeriesPattern34<FundedAddrData>} funded
 * @property {SeriesPattern35<EmptyAddrData>} extendedEmpty
 */

/**
 * @typedef {Object} SeriesTree_Addrs_ByBalance
 * @property {SeriesTree_Addrs_ByBalance_Supply} supply
 * @property {SeriesTree_Addrs_ByBalance_UtxoCount} utxoCount
 * @property {_0sats100btc100k100sats10btc10k10m10sats1btc1k1m1satOverPattern4} transferVolume
 * @property {_0sats100btc100k100sats10btc10k10m10sats1btc1k1m1satOverPattern} realizedCap
 * @property {SeriesTree_Addrs_ByBalance_RealizedProfit} realizedProfit
 * @property {SeriesTree_Addrs_ByBalance_RealizedLoss} realizedLoss
 */

/**
 * @typedef {Object} SeriesTree_Addrs_ByBalance_Supply
 * @property {DeltaDominanceTotalPattern} _0sats
 * @property {DeltaDominanceTotalPattern} _1satTo10sats
 * @property {DeltaDominanceTotalPattern} _10satsTo100sats
 * @property {DeltaDominanceTotalPattern} _100satsTo1kSats
 * @property {DeltaDominanceTotalPattern} _1kSatsTo10kSats
 * @property {DeltaDominanceTotalPattern} _10kSatsTo100kSats
 * @property {DeltaDominanceTotalPattern} _100kSatsTo1mSats
 * @property {DeltaDominanceTotalPattern} _1mSatsTo10mSats
 * @property {DeltaDominanceTotalPattern} _10mSatsTo1btc
 * @property {DeltaDominanceTotalPattern} _1btcTo10btc
 * @property {DeltaDominanceTotalPattern} _10btcTo100btc
 * @property {DeltaDominanceTotalPattern} _100btcTo1kBtc
 * @property {DeltaDominanceTotalPattern} _1kBtcTo10kBtc
 * @property {DeltaDominanceTotalPattern} _10kBtcTo100kBtc
 * @property {DeltaDominanceTotalPattern} over100kBtc
 */

/**
 * @typedef {Object} SeriesTree_Addrs_ByBalance_UtxoCount
 * @property {BaseDeltaPattern} _0sats
 * @property {BaseDeltaPattern} _1satTo10sats
 * @property {BaseDeltaPattern} _10satsTo100sats
 * @property {BaseDeltaPattern} _100satsTo1kSats
 * @property {BaseDeltaPattern} _1kSatsTo10kSats
 * @property {BaseDeltaPattern} _10kSatsTo100kSats
 * @property {BaseDeltaPattern} _100kSatsTo1mSats
 * @property {BaseDeltaPattern} _1mSatsTo10mSats
 * @property {BaseDeltaPattern} _10mSatsTo1btc
 * @property {BaseDeltaPattern} _1btcTo10btc
 * @property {BaseDeltaPattern} _10btcTo100btc
 * @property {BaseDeltaPattern} _100btcTo1kBtc
 * @property {BaseDeltaPattern} _1kBtcTo10kBtc
 * @property {BaseDeltaPattern} _10kBtcTo100kBtc
 * @property {BaseDeltaPattern} over100kBtc
 */

/**
 * @typedef {Object} SeriesTree_Addrs_ByBalance_RealizedProfit
 * @property {BlockCumulativeSumPattern} _0sats
 * @property {BlockCumulativeSumPattern} _1satTo10sats
 * @property {BlockCumulativeSumPattern} _10satsTo100sats
 * @property {BlockCumulativeSumPattern} _100satsTo1kSats
 * @property {BlockCumulativeSumPattern} _1kSatsTo10kSats
 * @property {BlockCumulativeSumPattern} _10kSatsTo100kSats
 * @property {BlockCumulativeSumPattern} _100kSatsTo1mSats
 * @property {BlockCumulativeSumPattern} _1mSatsTo10mSats
 * @property {BlockCumulativeSumPattern} _10mSatsTo1btc
 * @property {BlockCumulativeSumPattern} _1btcTo10btc
 * @property {BlockCumulativeSumPattern} _10btcTo100btc
 * @property {BlockCumulativeSumPattern} _100btcTo1kBtc
 * @property {BlockCumulativeSumPattern} _1kBtcTo10kBtc
 * @property {BlockCumulativeSumPattern} _10kBtcTo100kBtc
 * @property {BlockCumulativeSumPattern} over100kBtc
 */

/**
 * @typedef {Object} SeriesTree_Addrs_ByBalance_RealizedLoss
 * @property {BlockCumulativeSumPattern} _0sats
 * @property {BlockCumulativeSumPattern} _1satTo10sats
 * @property {BlockCumulativeSumPattern} _10satsTo100sats
 * @property {BlockCumulativeSumPattern} _100satsTo1kSats
 * @property {BlockCumulativeSumPattern} _1kSatsTo10kSats
 * @property {BlockCumulativeSumPattern} _10kSatsTo100kSats
 * @property {BlockCumulativeSumPattern} _100kSatsTo1mSats
 * @property {BlockCumulativeSumPattern} _1mSatsTo10mSats
 * @property {BlockCumulativeSumPattern} _10mSatsTo1btc
 * @property {BlockCumulativeSumPattern} _1btcTo10btc
 * @property {BlockCumulativeSumPattern} _10btcTo100btc
 * @property {BlockCumulativeSumPattern} _100btcTo1kBtc
 * @property {BlockCumulativeSumPattern} _1kBtcTo10kBtc
 * @property {BlockCumulativeSumPattern} _10kBtcTo100kBtc
 * @property {BlockCumulativeSumPattern} over100kBtc
 */

/**
 * @typedef {Object} SeriesTree_Addrs_Funded
 * @property {SeriesPattern1<StoredU64>} all
 * @property {SeriesPattern1<StoredU64>} p2pk65
 * @property {SeriesPattern1<StoredU64>} p2pk33
 * @property {SeriesPattern1<StoredU64>} p2pkh
 * @property {SeriesPattern1<StoredU64>} p2sh
 * @property {SeriesPattern1<StoredU64>} p2wpkh
 * @property {SeriesPattern1<StoredU64>} p2wsh
 * @property {SeriesPattern1<StoredU64>} p2tr
 * @property {SeriesPattern1<StoredU64>} p2a
 * @property {SeriesTree_Addrs_Funded_Balance} balance
 */

/**
 * @typedef {Object} SeriesTree_Addrs_Funded_Balance
 * @property {BaseDeltaPattern} _0sats
 * @property {BaseDeltaPattern} _1satTo10sats
 * @property {BaseDeltaPattern} _10satsTo100sats
 * @property {BaseDeltaPattern} _100satsTo1kSats
 * @property {BaseDeltaPattern} _1kSatsTo10kSats
 * @property {BaseDeltaPattern} _10kSatsTo100kSats
 * @property {BaseDeltaPattern} _100kSatsTo1mSats
 * @property {BaseDeltaPattern} _1mSatsTo10mSats
 * @property {BaseDeltaPattern} _10mSatsTo1btc
 * @property {BaseDeltaPattern} _1btcTo10btc
 * @property {BaseDeltaPattern} _10btcTo100btc
 * @property {BaseDeltaPattern} _100btcTo1kBtc
 * @property {BaseDeltaPattern} _1kBtcTo10kBtc
 * @property {BaseDeltaPattern} _10kBtcTo100kBtc
 * @property {BaseDeltaPattern} over100kBtc
 */

/**
 * @typedef {Object} SeriesTree_Addrs_Activity
 * @property {AllP2aP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshPattern} reactivated
 * @property {AllP2aP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshPattern} sending
 * @property {AllP2aP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshPattern} receiving
 * @property {AllP2aP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshPattern} bidirectional
 * @property {AllP2aP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshPattern} active
 */

/**
 * @typedef {Object} SeriesTree_Addrs_Reused
 * @property {FundedTotalPattern} count
 * @property {SeriesTree_Addrs_Reused_Events} events
 * @property {SeriesTree_Addrs_Reused_Supply} supply
 */

/**
 * @typedef {Object} SeriesTree_Addrs_Reused_Events
 * @property {AllP2aP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshPattern6} outputToReusedAddrCount
 * @property {AllP2aP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshPattern7} outputToReusedAddrShare
 * @property {_1m1w1y24hPercentPpmRatioPattern} spendableOutputToReusedAddrShare
 * @property {AllP2aP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshPattern6} inputFromReusedAddrCount
 * @property {AllP2aP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshPattern7} inputFromReusedAddrShare
 * @property {_1m1w1y24hBlockPattern} activeReusedAddrCount
 * @property {_1m1w1y24hBlockPattern2} activeReusedAddrShare
 */

/**
 * @typedef {Object} SeriesTree_Addrs_Reused_Supply
 * @property {BtcCentsSatsUsdPattern} all
 * @property {BtcCentsSatsUsdPattern} p2pk65
 * @property {BtcCentsSatsUsdPattern} p2pk33
 * @property {BtcCentsSatsUsdPattern} p2pkh
 * @property {BtcCentsSatsUsdPattern} p2sh
 * @property {BtcCentsSatsUsdPattern} p2wpkh
 * @property {BtcCentsSatsUsdPattern} p2wsh
 * @property {BtcCentsSatsUsdPattern} p2tr
 * @property {BtcCentsSatsUsdPattern} p2a
 * @property {AllP2aP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshPattern5} share
 */

/**
 * @typedef {Object} SeriesTree_Addrs_Respent
 * @property {FundedTotalPattern} count
 * @property {SeriesTree_Addrs_Respent_Events} events
 * @property {SeriesTree_Addrs_Respent_Supply} supply
 */

/**
 * @typedef {Object} SeriesTree_Addrs_Respent_Events
 * @property {AllP2aP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshPattern6} outputToReusedAddrCount
 * @property {AllP2aP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshPattern7} outputToReusedAddrShare
 * @property {_1m1w1y24hPercentPpmRatioPattern} spendableOutputToReusedAddrShare
 * @property {AllP2aP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshPattern6} inputFromReusedAddrCount
 * @property {AllP2aP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshPattern7} inputFromReusedAddrShare
 * @property {_1m1w1y24hBlockPattern} activeReusedAddrCount
 * @property {_1m1w1y24hBlockPattern2} activeReusedAddrShare
 */

/**
 * @typedef {Object} SeriesTree_Addrs_Respent_Supply
 * @property {BtcCentsSatsUsdPattern} all
 * @property {BtcCentsSatsUsdPattern} p2pk65
 * @property {BtcCentsSatsUsdPattern} p2pk33
 * @property {BtcCentsSatsUsdPattern} p2pkh
 * @property {BtcCentsSatsUsdPattern} p2sh
 * @property {BtcCentsSatsUsdPattern} p2wpkh
 * @property {BtcCentsSatsUsdPattern} p2wsh
 * @property {BtcCentsSatsUsdPattern} p2tr
 * @property {BtcCentsSatsUsdPattern} p2a
 * @property {AllP2aP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshPattern5} share
 */

/**
 * @typedef {Object} SeriesTree_Addrs_Exposed
 * @property {FundedTotalPattern} count
 * @property {SeriesTree_Addrs_Exposed_Supply} supply
 */

/**
 * @typedef {Object} SeriesTree_Addrs_Exposed_Supply
 * @property {BtcCentsSatsUsdPattern} all
 * @property {BtcCentsSatsUsdPattern} p2pk65
 * @property {BtcCentsSatsUsdPattern} p2pk33
 * @property {BtcCentsSatsUsdPattern} p2pkh
 * @property {BtcCentsSatsUsdPattern} p2sh
 * @property {BtcCentsSatsUsdPattern} p2wpkh
 * @property {BtcCentsSatsUsdPattern} p2wsh
 * @property {BtcCentsSatsUsdPattern} p2tr
 * @property {BtcCentsSatsUsdPattern} p2a
 * @property {AllP2aP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshPattern5} share
 */

/**
 * @typedef {Object} SeriesTree_Addrs_Delta
 * @property {AbsoluteRatePattern2} all
 * @property {AbsoluteRatePattern2} p2pk65
 * @property {AbsoluteRatePattern2} p2pk33
 * @property {AbsoluteRatePattern2} p2pkh
 * @property {AbsoluteRatePattern2} p2sh
 * @property {AbsoluteRatePattern2} p2wpkh
 * @property {AbsoluteRatePattern2} p2wsh
 * @property {AbsoluteRatePattern2} p2tr
 * @property {AbsoluteRatePattern2} p2a
 */

/**
 * @typedef {Object} SeriesTree_Addrs_AvgBalance
 * @property {BtcCentsSatsUsdPattern} all
 * @property {BtcCentsSatsUsdPattern} p2pk65
 * @property {BtcCentsSatsUsdPattern} p2pk33
 * @property {BtcCentsSatsUsdPattern} p2pkh
 * @property {BtcCentsSatsUsdPattern} p2sh
 * @property {BtcCentsSatsUsdPattern} p2wpkh
 * @property {BtcCentsSatsUsdPattern} p2wsh
 * @property {BtcCentsSatsUsdPattern} p2tr
 * @property {BtcCentsSatsUsdPattern} p2a
 */

/**
 * @typedef {Object} SeriesTree_Scripts
 * @property {SeriesTree_Scripts_Raw} raw
 */

/**
 * @typedef {Object} SeriesTree_Scripts_Raw
 * @property {SeriesTree_Scripts_Raw_Empty} empty
 * @property {SeriesTree_Scripts_Raw_P2ms} p2ms
 * @property {SeriesTree_Scripts_Raw_Unknown} unknown
 */

/**
 * @typedef {Object} SeriesTree_Scripts_Raw_Empty
 * @property {SeriesPattern18<EmptyOutputIndex>} firstIndex
 * @property {SeriesPattern22<TxIndex>} toTxIndex
 */

/**
 * @typedef {Object} SeriesTree_Scripts_Raw_P2ms
 * @property {SeriesPattern18<P2MSOutputIndex>} firstIndex
 * @property {SeriesPattern25<TxIndex>} toTxIndex
 * @property {SeriesPattern25<SigOps>} legacySigops
 */

/**
 * @typedef {Object} SeriesTree_Scripts_Raw_Unknown
 * @property {SeriesPattern18<UnknownOutputIndex>} firstIndex
 * @property {SeriesPattern33<TxIndex>} toTxIndex
 * @property {SeriesPattern33<SigOps>} legacySigops
 */

/**
 * @typedef {Object} SeriesTree_OpReturn
 * @property {SeriesTree_OpReturn_Raw} raw
 * @property {SeriesTree_OpReturn_Total} total
 * @property {SeriesTree_OpReturn_ByKind} byKind
 * @property {SeriesTree_OpReturn_Policy} policy
 */

/**
 * @typedef {Object} SeriesTree_OpReturn_Raw
 * @property {SeriesPattern18<OpReturnIndex>} firstIndex
 * @property {SeriesPattern23<TxIndex>} toTxIndex
 * @property {SeriesPattern23<OpReturnKind>} kind
 * @property {SeriesPattern23<StoredU32>} postOpReturnBytes
 */

/**
 * @typedef {Object} SeriesTree_OpReturn_Total
 * @property {AverageBlockCumulativeSumPattern2<Bytes>} dataBytes
 * @property {AverageBlockCumulativeSumPattern2<StoredU64>} txCount
 * @property {AverageBlockCumulativeSumPattern2<VSize>} txVsize
 * @property {AverageBlockCumulativeSumPattern2<Sats>} fees
 * @property {PercentPpmRatioPattern2} chainShare
 * @property {_1m1w1y24hPercentPpmRatioPattern} feeShare
 */

/**
 * @typedef {Object} SeriesTree_OpReturn_ByKind
 * @property {SeriesTree_OpReturn_ByKind_OutputCount} outputCount
 * @property {SeriesTree_OpReturn_ByKind_DataBytes} dataBytes
 * @property {SeriesTree_OpReturn_ByKind_TxCount} txCount
 * @property {SeriesTree_OpReturn_ByKind_TxVsize} txVsize
 * @property {SeriesTree_OpReturn_ByKind_Fees} fees
 */

/**
 * @typedef {Object} SeriesTree_OpReturn_ByKind_OutputCount
 * @property {AverageBlockCumulativeSumPattern2<StoredU64>} runes
 * @property {AverageBlockCumulativeSumPattern2<StoredU64>} veriBlock
 * @property {AverageBlockCumulativeSumPattern2<StoredU64>} omni
 * @property {AverageBlockCumulativeSumPattern2<StoredU64>} stacks
 * @property {AverageBlockCumulativeSumPattern2<StoredU64>} blockstack
 * @property {AverageBlockCumulativeSumPattern2<StoredU64>} colu
 * @property {AverageBlockCumulativeSumPattern2<StoredU64>} openAssets
 * @property {AverageBlockCumulativeSumPattern2<StoredU64>} komodo
 * @property {AverageBlockCumulativeSumPattern2<StoredU64>} coinSpark
 * @property {AverageBlockCumulativeSumPattern2<StoredU64>} poet
 * @property {AverageBlockCumulativeSumPattern2<StoredU64>} docproof
 * @property {AverageBlockCumulativeSumPattern2<StoredU64>} openTimestamps
 * @property {AverageBlockCumulativeSumPattern2<StoredU64>} factom
 * @property {AverageBlockCumulativeSumPattern2<StoredU64>} eternityWall
 * @property {AverageBlockCumulativeSumPattern2<StoredU64>} memo
 * @property {AverageBlockCumulativeSumPattern2<StoredU64>} bitproof
 * @property {AverageBlockCumulativeSumPattern2<StoredU64>} ascribe
 * @property {AverageBlockCumulativeSumPattern2<StoredU64>} stampery
 * @property {AverageBlockCumulativeSumPattern2<StoredU64>} epobc
 * @property {AverageBlockCumulativeSumPattern2<StoredU64>} bareHash
 * @property {AverageBlockCumulativeSumPattern2<StoredU64>} text
 * @property {AverageBlockCumulativeSumPattern2<StoredU64>} empty
 * @property {AverageBlockCumulativeSumPattern2<StoredU64>} unknown
 */

/**
 * @typedef {Object} SeriesTree_OpReturn_ByKind_DataBytes
 * @property {AverageBlockChainCumulativeDataSumPattern} runes
 * @property {AverageBlockChainCumulativeDataSumPattern} veriBlock
 * @property {AverageBlockChainCumulativeDataSumPattern} omni
 * @property {AverageBlockChainCumulativeDataSumPattern} stacks
 * @property {AverageBlockChainCumulativeDataSumPattern} blockstack
 * @property {AverageBlockChainCumulativeDataSumPattern} colu
 * @property {AverageBlockChainCumulativeDataSumPattern} openAssets
 * @property {AverageBlockChainCumulativeDataSumPattern} komodo
 * @property {AverageBlockChainCumulativeDataSumPattern} coinSpark
 * @property {AverageBlockChainCumulativeDataSumPattern} poet
 * @property {AverageBlockChainCumulativeDataSumPattern} docproof
 * @property {AverageBlockChainCumulativeDataSumPattern} openTimestamps
 * @property {AverageBlockChainCumulativeDataSumPattern} factom
 * @property {AverageBlockChainCumulativeDataSumPattern} eternityWall
 * @property {AverageBlockChainCumulativeDataSumPattern} memo
 * @property {AverageBlockChainCumulativeDataSumPattern} bitproof
 * @property {AverageBlockChainCumulativeDataSumPattern} ascribe
 * @property {AverageBlockChainCumulativeDataSumPattern} stampery
 * @property {AverageBlockChainCumulativeDataSumPattern} epobc
 * @property {AverageBlockChainCumulativeDataSumPattern} bareHash
 * @property {AverageBlockChainCumulativeDataSumPattern} text
 * @property {AverageBlockChainCumulativeDataSumPattern} empty
 * @property {AverageBlockChainCumulativeDataSumPattern} unknown
 */

/**
 * @typedef {Object} SeriesTree_OpReturn_ByKind_TxCount
 * @property {AverageBlockCumulativeSumPattern2<StoredU64>} runes
 * @property {AverageBlockCumulativeSumPattern2<StoredU64>} veriBlock
 * @property {AverageBlockCumulativeSumPattern2<StoredU64>} omni
 * @property {AverageBlockCumulativeSumPattern2<StoredU64>} stacks
 * @property {AverageBlockCumulativeSumPattern2<StoredU64>} blockstack
 * @property {AverageBlockCumulativeSumPattern2<StoredU64>} colu
 * @property {AverageBlockCumulativeSumPattern2<StoredU64>} openAssets
 * @property {AverageBlockCumulativeSumPattern2<StoredU64>} komodo
 * @property {AverageBlockCumulativeSumPattern2<StoredU64>} coinSpark
 * @property {AverageBlockCumulativeSumPattern2<StoredU64>} poet
 * @property {AverageBlockCumulativeSumPattern2<StoredU64>} docproof
 * @property {AverageBlockCumulativeSumPattern2<StoredU64>} openTimestamps
 * @property {AverageBlockCumulativeSumPattern2<StoredU64>} factom
 * @property {AverageBlockCumulativeSumPattern2<StoredU64>} eternityWall
 * @property {AverageBlockCumulativeSumPattern2<StoredU64>} memo
 * @property {AverageBlockCumulativeSumPattern2<StoredU64>} bitproof
 * @property {AverageBlockCumulativeSumPattern2<StoredU64>} ascribe
 * @property {AverageBlockCumulativeSumPattern2<StoredU64>} stampery
 * @property {AverageBlockCumulativeSumPattern2<StoredU64>} epobc
 * @property {AverageBlockCumulativeSumPattern2<StoredU64>} bareHash
 * @property {AverageBlockCumulativeSumPattern2<StoredU64>} text
 * @property {AverageBlockCumulativeSumPattern2<StoredU64>} empty
 * @property {AverageBlockCumulativeSumPattern2<StoredU64>} unknown
 */

/**
 * @typedef {Object} SeriesTree_OpReturn_ByKind_TxVsize
 * @property {AverageBlockCumulativeSumPattern2<VSize>} runes
 * @property {AverageBlockCumulativeSumPattern2<VSize>} veriBlock
 * @property {AverageBlockCumulativeSumPattern2<VSize>} omni
 * @property {AverageBlockCumulativeSumPattern2<VSize>} stacks
 * @property {AverageBlockCumulativeSumPattern2<VSize>} blockstack
 * @property {AverageBlockCumulativeSumPattern2<VSize>} colu
 * @property {AverageBlockCumulativeSumPattern2<VSize>} openAssets
 * @property {AverageBlockCumulativeSumPattern2<VSize>} komodo
 * @property {AverageBlockCumulativeSumPattern2<VSize>} coinSpark
 * @property {AverageBlockCumulativeSumPattern2<VSize>} poet
 * @property {AverageBlockCumulativeSumPattern2<VSize>} docproof
 * @property {AverageBlockCumulativeSumPattern2<VSize>} openTimestamps
 * @property {AverageBlockCumulativeSumPattern2<VSize>} factom
 * @property {AverageBlockCumulativeSumPattern2<VSize>} eternityWall
 * @property {AverageBlockCumulativeSumPattern2<VSize>} memo
 * @property {AverageBlockCumulativeSumPattern2<VSize>} bitproof
 * @property {AverageBlockCumulativeSumPattern2<VSize>} ascribe
 * @property {AverageBlockCumulativeSumPattern2<VSize>} stampery
 * @property {AverageBlockCumulativeSumPattern2<VSize>} epobc
 * @property {AverageBlockCumulativeSumPattern2<VSize>} bareHash
 * @property {AverageBlockCumulativeSumPattern2<VSize>} text
 * @property {AverageBlockCumulativeSumPattern2<VSize>} empty
 * @property {AverageBlockCumulativeSumPattern2<VSize>} unknown
 */

/**
 * @typedef {Object} SeriesTree_OpReturn_ByKind_Fees
 * @property {AverageBlockCumulativeFeeSumPattern} runes
 * @property {AverageBlockCumulativeFeeSumPattern} veriBlock
 * @property {AverageBlockCumulativeFeeSumPattern} omni
 * @property {AverageBlockCumulativeFeeSumPattern} stacks
 * @property {AverageBlockCumulativeFeeSumPattern} blockstack
 * @property {AverageBlockCumulativeFeeSumPattern} colu
 * @property {AverageBlockCumulativeFeeSumPattern} openAssets
 * @property {AverageBlockCumulativeFeeSumPattern} komodo
 * @property {AverageBlockCumulativeFeeSumPattern} coinSpark
 * @property {AverageBlockCumulativeFeeSumPattern} poet
 * @property {AverageBlockCumulativeFeeSumPattern} docproof
 * @property {AverageBlockCumulativeFeeSumPattern} openTimestamps
 * @property {AverageBlockCumulativeFeeSumPattern} factom
 * @property {AverageBlockCumulativeFeeSumPattern} eternityWall
 * @property {AverageBlockCumulativeFeeSumPattern} memo
 * @property {AverageBlockCumulativeFeeSumPattern} bitproof
 * @property {AverageBlockCumulativeFeeSumPattern} ascribe
 * @property {AverageBlockCumulativeFeeSumPattern} stampery
 * @property {AverageBlockCumulativeFeeSumPattern} epobc
 * @property {AverageBlockCumulativeFeeSumPattern} bareHash
 * @property {AverageBlockCumulativeFeeSumPattern} text
 * @property {AverageBlockCumulativeFeeSumPattern} empty
 * @property {AverageBlockCumulativeFeeSumPattern} unknown
 */

/**
 * @typedef {Object} SeriesTree_OpReturn_Policy
 * @property {SeriesTree_OpReturn_Policy_OutputCount} outputCount
 * @property {SeriesTree_OpReturn_Policy_DataBytes} dataBytes
 * @property {SeriesTree_OpReturn_Policy_TxCount} txCount
 * @property {SeriesTree_OpReturn_Policy_TxVsize} txVsize
 * @property {SeriesTree_OpReturn_Policy_Fees} fees
 */

/**
 * @typedef {Object} SeriesTree_OpReturn_Policy_OutputCount
 * @property {AverageBlockCumulativeSumPattern2<StoredU64>} preV30Standard
 * @property {AverageBlockCumulativeSumPattern2<StoredU64>} preV30Nonstandard
 * @property {AverageBlockCumulativeSumPattern2<StoredU64>} oversized
 * @property {AverageBlockCumulativeSumPattern2<StoredU64>} multiple
 */

/**
 * @typedef {Object} SeriesTree_OpReturn_Policy_DataBytes
 * @property {AverageBlockChainCumulativeDataSumPattern} preV30Standard
 * @property {AverageBlockChainCumulativeDataSumPattern} preV30Nonstandard
 * @property {AverageBlockChainCumulativeDataSumPattern} oversized
 * @property {AverageBlockChainCumulativeDataSumPattern} multiple
 */

/**
 * @typedef {Object} SeriesTree_OpReturn_Policy_TxCount
 * @property {AverageBlockCumulativeSumPattern2<StoredU64>} preV30Standard
 * @property {AverageBlockCumulativeSumPattern2<StoredU64>} preV30Nonstandard
 * @property {AverageBlockCumulativeSumPattern2<StoredU64>} oversized
 * @property {AverageBlockCumulativeSumPattern2<StoredU64>} multiple
 */

/**
 * @typedef {Object} SeriesTree_OpReturn_Policy_TxVsize
 * @property {AverageBlockCumulativeSumPattern2<VSize>} preV30Standard
 * @property {AverageBlockCumulativeSumPattern2<VSize>} preV30Nonstandard
 * @property {AverageBlockCumulativeSumPattern2<VSize>} oversized
 * @property {AverageBlockCumulativeSumPattern2<VSize>} multiple
 */

/**
 * @typedef {Object} SeriesTree_OpReturn_Policy_Fees
 * @property {AverageBlockCumulativeFeeSumPattern} preV30Standard
 * @property {AverageBlockCumulativeFeeSumPattern} preV30Nonstandard
 * @property {AverageBlockCumulativeFeeSumPattern} oversized
 * @property {AverageBlockCumulativeFeeSumPattern} multiple
 */

/**
 * @typedef {Object} SeriesTree_Mining
 * @property {SeriesTree_Mining_Rewards} rewards
 * @property {SeriesTree_Mining_Hashrate} hashrate
 */

/**
 * @typedef {Object} SeriesTree_Mining_Rewards
 * @property {AverageBlockCumulativeSumPattern} coinbase
 * @property {SeriesTree_Mining_Rewards_Subsidy} subsidy
 * @property {SeriesTree_Mining_Rewards_Fees} fees
 * @property {SeriesPattern18<Sats>} outputVolume
 * @property {BlockCumulativePattern} unclaimed
 */

/**
 * @typedef {Object} SeriesTree_Mining_Rewards_Subsidy
 * @property {BtcCentsSatsUsdPattern3} block
 * @property {BtcCentsSatsUsdPattern} cumulative
 * @property {_1m1w1y24hPattern5} sum
 * @property {_1m1w1y24hPattern4} average
 * @property {_1m1w1y24hPercentPpmRatioPattern} dominance
 */

/**
 * @typedef {Object} SeriesTree_Mining_Rewards_Fees
 * @property {BtcCentsSatsUsdPattern3} block
 * @property {BtcCentsSatsUsdPattern} cumulative
 * @property {_1m1w1y24hPattern5} sum
 * @property {_1m1w1y24hPattern4} average
 * @property {_1m1w1y24hPattern5} min
 * @property {_1m1w1y24hPattern5} max
 * @property {_1m1w1y24hPattern5} pct10
 * @property {_1m1w1y24hPattern5} pct25
 * @property {_1m1w1y24hPattern5} median
 * @property {_1m1w1y24hPattern5} pct75
 * @property {_1m1w1y24hPattern5} pct90
 * @property {_1m1w1y24hPercentPpmRatioPattern} dominance
 * @property {SeriesTree_Mining_Rewards_Fees_ToSubsidy} toSubsidy
 */

/**
 * @typedef {Object} SeriesTree_Mining_Rewards_Fees_ToSubsidy
 * @property {PercentPpmRatioPattern5} _24h
 * @property {PercentPpmRatioPattern5} _1w
 * @property {PercentPpmRatioPattern5} _1m
 * @property {PercentPpmRatioPattern5} _1y
 */

/**
 * @typedef {Object} SeriesTree_Mining_Hashrate
 * @property {SeriesTree_Mining_Hashrate_Rate} rate
 * @property {PhsReboundThsPattern} price
 * @property {PhsReboundThsPattern} value
 */

/**
 * @typedef {Object} SeriesTree_Mining_Hashrate_Rate
 * @property {SeriesPattern1<StoredF64>} base
 * @property {SeriesTree_Mining_Hashrate_Rate_Sma} sma
 * @property {SeriesPattern1<StoredF64>} ath
 * @property {PercentPpmRatioPattern3} drawdown
 */

/**
 * @typedef {Object} SeriesTree_Mining_Hashrate_Rate_Sma
 * @property {SeriesPattern1<StoredF64>} _1w
 * @property {SeriesPattern1<StoredF64>} _1m
 * @property {SeriesPattern1<StoredF64>} _2m
 * @property {SeriesPattern1<StoredF64>} _1y
 */

/**
 * @typedef {Object} SeriesTree_Cointime
 * @property {SeriesTree_Cointime_Activity} activity
 * @property {SeriesTree_Cointime_AgeRange} ageRange
 * @property {SeriesTree_Cointime_Urpd} urpd
 * @property {SeriesTree_Cointime_Awake} awake
 * @property {SupplyPattern} dormant
 * @property {SeriesTree_Cointime_Sth} sth
 * @property {SeriesTree_Cointime_Lth} lth
 * @property {CentsPpmRatioSatsUsdPattern} under4mAwakePrice
 * @property {CentsPpmRatioSatsUsdPattern} under4mAwakeCapitalizedPrice
 * @property {CentsPpmRatioSatsUsdPattern} under6mAwakePrice
 * @property {CentsPpmRatioSatsUsdPattern} under6mAwakeCapitalizedPrice
 * @property {CentsPpmRatioSatsUsdPattern} over4mAwakePrice
 * @property {CentsPpmRatioSatsUsdPattern} over4mAwakeCapitalizedPrice
 * @property {CentsPpmRatioSatsUsdPattern} over6mAwakePrice
 * @property {CentsPpmRatioSatsUsdPattern} over6mAwakeCapitalizedPrice
 * @property {SeriesTree_Cointime_Supply} supply
 * @property {SeriesTree_Cointime_Value} value
 * @property {SeriesTree_Cointime_Cap} cap
 * @property {SeriesTree_Cointime_Prices} prices
 * @property {SeriesTree_Cointime_Adjusted} adjusted
 * @property {SeriesTree_Cointime_ReserveRisk} reserveRisk
 */

/**
 * @typedef {Object} SeriesTree_Cointime_Activity
 * @property {AverageBlockCumulativeSumPattern2<StoredF64>} coinblocksCreated
 * @property {AverageBlockCumulativeSumPattern2<StoredF64>} coinblocksStored
 * @property {SeriesPattern1<StoredF64>} liveliness
 * @property {SeriesPattern1<StoredF64>} vaultedness
 * @property {SeriesPattern1<StoredF64>} ratio
 * @property {AverageBlockCumulativeSumPattern2<StoredF64>} coinblocksDestroyed
 */

/**
 * @typedef {Object} SeriesTree_Cointime_AgeRange
 * @property {SeriesTree_Cointime_AgeRange_CoindaysConsumed} coindaysConsumed
 * @property {SeriesTree_Cointime_AgeRange_CoindaysStored} coindaysStored
 * @property {SeriesTree_Cointime_AgeRange_Activity} activity
 * @property {SeriesTree_Cointime_AgeRange_Supply} supply
 * @property {SeriesTree_Cointime_AgeRange_CoindaysCreated} coindaysCreated
 */

/**
 * @typedef {Object} SeriesTree_Cointime_AgeRange_CoindaysConsumed
 * @property {AverageBlockCumulativeSumPattern2<StoredF64>} under1h
 * @property {AverageBlockCumulativeSumPattern2<StoredF64>} _1hTo1d
 * @property {AverageBlockCumulativeSumPattern2<StoredF64>} _1dTo1w
 * @property {AverageBlockCumulativeSumPattern2<StoredF64>} _1wTo1m
 * @property {AverageBlockCumulativeSumPattern2<StoredF64>} _1mTo2m
 * @property {AverageBlockCumulativeSumPattern2<StoredF64>} _2mTo3m
 * @property {AverageBlockCumulativeSumPattern2<StoredF64>} _3mTo4m
 * @property {AverageBlockCumulativeSumPattern2<StoredF64>} _4mTo5m
 * @property {AverageBlockCumulativeSumPattern2<StoredF64>} _5mTo6m
 * @property {AverageBlockCumulativeSumPattern2<StoredF64>} _6mTo9m
 * @property {AverageBlockCumulativeSumPattern2<StoredF64>} _9mTo1y
 * @property {AverageBlockCumulativeSumPattern2<StoredF64>} _1yTo18m
 * @property {AverageBlockCumulativeSumPattern2<StoredF64>} _18mTo2y
 * @property {AverageBlockCumulativeSumPattern2<StoredF64>} _2yTo3y
 * @property {AverageBlockCumulativeSumPattern2<StoredF64>} _3yTo4y
 * @property {AverageBlockCumulativeSumPattern2<StoredF64>} _4yTo5y
 * @property {AverageBlockCumulativeSumPattern2<StoredF64>} _5yTo6y
 * @property {AverageBlockCumulativeSumPattern2<StoredF64>} _6yTo7y
 * @property {AverageBlockCumulativeSumPattern2<StoredF64>} _7yTo8y
 * @property {AverageBlockCumulativeSumPattern2<StoredF64>} _8yTo10y
 * @property {AverageBlockCumulativeSumPattern2<StoredF64>} _10yTo12y
 * @property {AverageBlockCumulativeSumPattern2<StoredF64>} _12yTo15y
 * @property {AverageBlockCumulativeSumPattern2<StoredF64>} over15y
 */

/**
 * @typedef {Object} SeriesTree_Cointime_AgeRange_CoindaysStored
 * @property {AverageBlockCumulativeSumPattern2<StoredF64>} under1h
 * @property {AverageBlockCumulativeSumPattern2<StoredF64>} _1hTo1d
 * @property {AverageBlockCumulativeSumPattern2<StoredF64>} _1dTo1w
 * @property {AverageBlockCumulativeSumPattern2<StoredF64>} _1wTo1m
 * @property {AverageBlockCumulativeSumPattern2<StoredF64>} _1mTo2m
 * @property {AverageBlockCumulativeSumPattern2<StoredF64>} _2mTo3m
 * @property {AverageBlockCumulativeSumPattern2<StoredF64>} _3mTo4m
 * @property {AverageBlockCumulativeSumPattern2<StoredF64>} _4mTo5m
 * @property {AverageBlockCumulativeSumPattern2<StoredF64>} _5mTo6m
 * @property {AverageBlockCumulativeSumPattern2<StoredF64>} _6mTo9m
 * @property {AverageBlockCumulativeSumPattern2<StoredF64>} _9mTo1y
 * @property {AverageBlockCumulativeSumPattern2<StoredF64>} _1yTo18m
 * @property {AverageBlockCumulativeSumPattern2<StoredF64>} _18mTo2y
 * @property {AverageBlockCumulativeSumPattern2<StoredF64>} _2yTo3y
 * @property {AverageBlockCumulativeSumPattern2<StoredF64>} _3yTo4y
 * @property {AverageBlockCumulativeSumPattern2<StoredF64>} _4yTo5y
 * @property {AverageBlockCumulativeSumPattern2<StoredF64>} _5yTo6y
 * @property {AverageBlockCumulativeSumPattern2<StoredF64>} _6yTo7y
 * @property {AverageBlockCumulativeSumPattern2<StoredF64>} _7yTo8y
 * @property {AverageBlockCumulativeSumPattern2<StoredF64>} _8yTo10y
 * @property {AverageBlockCumulativeSumPattern2<StoredF64>} _10yTo12y
 * @property {AverageBlockCumulativeSumPattern2<StoredF64>} _12yTo15y
 * @property {AverageBlockCumulativeSumPattern2<StoredF64>} over15y
 */

/**
 * @typedef {Object} SeriesTree_Cointime_AgeRange_Activity
 * @property {SeriesTree_Cointime_AgeRange_Activity_Wakefulness} wakefulness
 * @property {SeriesTree_Cointime_AgeRange_Activity_Dormancy} dormancy
 * @property {SeriesTree_Cointime_AgeRange_Activity_WakefulnessToDormancy} wakefulnessToDormancy
 */

/**
 * @typedef {Object} SeriesTree_Cointime_AgeRange_Activity_Wakefulness
 * @property {SeriesPattern1<StoredF64>} under1h
 * @property {SeriesPattern1<StoredF64>} _1hTo1d
 * @property {SeriesPattern1<StoredF64>} _1dTo1w
 * @property {SeriesPattern1<StoredF64>} _1wTo1m
 * @property {SeriesPattern1<StoredF64>} _1mTo2m
 * @property {SeriesPattern1<StoredF64>} _2mTo3m
 * @property {SeriesPattern1<StoredF64>} _3mTo4m
 * @property {SeriesPattern1<StoredF64>} _4mTo5m
 * @property {SeriesPattern1<StoredF64>} _5mTo6m
 * @property {SeriesPattern1<StoredF64>} _6mTo9m
 * @property {SeriesPattern1<StoredF64>} _9mTo1y
 * @property {SeriesPattern1<StoredF64>} _1yTo18m
 * @property {SeriesPattern1<StoredF64>} _18mTo2y
 * @property {SeriesPattern1<StoredF64>} _2yTo3y
 * @property {SeriesPattern1<StoredF64>} _3yTo4y
 * @property {SeriesPattern1<StoredF64>} _4yTo5y
 * @property {SeriesPattern1<StoredF64>} _5yTo6y
 * @property {SeriesPattern1<StoredF64>} _6yTo7y
 * @property {SeriesPattern1<StoredF64>} _7yTo8y
 * @property {SeriesPattern1<StoredF64>} _8yTo10y
 * @property {SeriesPattern1<StoredF64>} _10yTo12y
 * @property {SeriesPattern1<StoredF64>} _12yTo15y
 * @property {SeriesPattern1<StoredF64>} over15y
 */

/**
 * @typedef {Object} SeriesTree_Cointime_AgeRange_Activity_Dormancy
 * @property {SeriesPattern1<StoredF64>} under1h
 * @property {SeriesPattern1<StoredF64>} _1hTo1d
 * @property {SeriesPattern1<StoredF64>} _1dTo1w
 * @property {SeriesPattern1<StoredF64>} _1wTo1m
 * @property {SeriesPattern1<StoredF64>} _1mTo2m
 * @property {SeriesPattern1<StoredF64>} _2mTo3m
 * @property {SeriesPattern1<StoredF64>} _3mTo4m
 * @property {SeriesPattern1<StoredF64>} _4mTo5m
 * @property {SeriesPattern1<StoredF64>} _5mTo6m
 * @property {SeriesPattern1<StoredF64>} _6mTo9m
 * @property {SeriesPattern1<StoredF64>} _9mTo1y
 * @property {SeriesPattern1<StoredF64>} _1yTo18m
 * @property {SeriesPattern1<StoredF64>} _18mTo2y
 * @property {SeriesPattern1<StoredF64>} _2yTo3y
 * @property {SeriesPattern1<StoredF64>} _3yTo4y
 * @property {SeriesPattern1<StoredF64>} _4yTo5y
 * @property {SeriesPattern1<StoredF64>} _5yTo6y
 * @property {SeriesPattern1<StoredF64>} _6yTo7y
 * @property {SeriesPattern1<StoredF64>} _7yTo8y
 * @property {SeriesPattern1<StoredF64>} _8yTo10y
 * @property {SeriesPattern1<StoredF64>} _10yTo12y
 * @property {SeriesPattern1<StoredF64>} _12yTo15y
 * @property {SeriesPattern1<StoredF64>} over15y
 */

/**
 * @typedef {Object} SeriesTree_Cointime_AgeRange_Activity_WakefulnessToDormancy
 * @property {SeriesPattern1<StoredF64>} under1h
 * @property {SeriesPattern1<StoredF64>} _1hTo1d
 * @property {SeriesPattern1<StoredF64>} _1dTo1w
 * @property {SeriesPattern1<StoredF64>} _1wTo1m
 * @property {SeriesPattern1<StoredF64>} _1mTo2m
 * @property {SeriesPattern1<StoredF64>} _2mTo3m
 * @property {SeriesPattern1<StoredF64>} _3mTo4m
 * @property {SeriesPattern1<StoredF64>} _4mTo5m
 * @property {SeriesPattern1<StoredF64>} _5mTo6m
 * @property {SeriesPattern1<StoredF64>} _6mTo9m
 * @property {SeriesPattern1<StoredF64>} _9mTo1y
 * @property {SeriesPattern1<StoredF64>} _1yTo18m
 * @property {SeriesPattern1<StoredF64>} _18mTo2y
 * @property {SeriesPattern1<StoredF64>} _2yTo3y
 * @property {SeriesPattern1<StoredF64>} _3yTo4y
 * @property {SeriesPattern1<StoredF64>} _4yTo5y
 * @property {SeriesPattern1<StoredF64>} _5yTo6y
 * @property {SeriesPattern1<StoredF64>} _6yTo7y
 * @property {SeriesPattern1<StoredF64>} _7yTo8y
 * @property {SeriesPattern1<StoredF64>} _8yTo10y
 * @property {SeriesPattern1<StoredF64>} _10yTo12y
 * @property {SeriesPattern1<StoredF64>} _12yTo15y
 * @property {SeriesPattern1<StoredF64>} over15y
 */

/**
 * @typedef {Object} SeriesTree_Cointime_AgeRange_Supply
 * @property {SeriesTree_Cointime_AgeRange_Supply_Awake} awake
 * @property {SeriesTree_Cointime_AgeRange_Supply_Dormant} dormant
 */

/**
 * @typedef {Object} SeriesTree_Cointime_AgeRange_Supply_Awake
 * @property {BtcCentsSatsUsdPattern} under1h
 * @property {BtcCentsSatsUsdPattern} _1hTo1d
 * @property {BtcCentsSatsUsdPattern} _1dTo1w
 * @property {BtcCentsSatsUsdPattern} _1wTo1m
 * @property {BtcCentsSatsUsdPattern} _1mTo2m
 * @property {BtcCentsSatsUsdPattern} _2mTo3m
 * @property {BtcCentsSatsUsdPattern} _3mTo4m
 * @property {BtcCentsSatsUsdPattern} _4mTo5m
 * @property {BtcCentsSatsUsdPattern} _5mTo6m
 * @property {BtcCentsSatsUsdPattern} _6mTo9m
 * @property {BtcCentsSatsUsdPattern} _9mTo1y
 * @property {BtcCentsSatsUsdPattern} _1yTo18m
 * @property {BtcCentsSatsUsdPattern} _18mTo2y
 * @property {BtcCentsSatsUsdPattern} _2yTo3y
 * @property {BtcCentsSatsUsdPattern} _3yTo4y
 * @property {BtcCentsSatsUsdPattern} _4yTo5y
 * @property {BtcCentsSatsUsdPattern} _5yTo6y
 * @property {BtcCentsSatsUsdPattern} _6yTo7y
 * @property {BtcCentsSatsUsdPattern} _7yTo8y
 * @property {BtcCentsSatsUsdPattern} _8yTo10y
 * @property {BtcCentsSatsUsdPattern} _10yTo12y
 * @property {BtcCentsSatsUsdPattern} _12yTo15y
 * @property {BtcCentsSatsUsdPattern} over15y
 */

/**
 * @typedef {Object} SeriesTree_Cointime_AgeRange_Supply_Dormant
 * @property {BtcCentsSatsUsdPattern} under1h
 * @property {BtcCentsSatsUsdPattern} _1hTo1d
 * @property {BtcCentsSatsUsdPattern} _1dTo1w
 * @property {BtcCentsSatsUsdPattern} _1wTo1m
 * @property {BtcCentsSatsUsdPattern} _1mTo2m
 * @property {BtcCentsSatsUsdPattern} _2mTo3m
 * @property {BtcCentsSatsUsdPattern} _3mTo4m
 * @property {BtcCentsSatsUsdPattern} _4mTo5m
 * @property {BtcCentsSatsUsdPattern} _5mTo6m
 * @property {BtcCentsSatsUsdPattern} _6mTo9m
 * @property {BtcCentsSatsUsdPattern} _9mTo1y
 * @property {BtcCentsSatsUsdPattern} _1yTo18m
 * @property {BtcCentsSatsUsdPattern} _18mTo2y
 * @property {BtcCentsSatsUsdPattern} _2yTo3y
 * @property {BtcCentsSatsUsdPattern} _3yTo4y
 * @property {BtcCentsSatsUsdPattern} _4yTo5y
 * @property {BtcCentsSatsUsdPattern} _5yTo6y
 * @property {BtcCentsSatsUsdPattern} _6yTo7y
 * @property {BtcCentsSatsUsdPattern} _7yTo8y
 * @property {BtcCentsSatsUsdPattern} _8yTo10y
 * @property {BtcCentsSatsUsdPattern} _10yTo12y
 * @property {BtcCentsSatsUsdPattern} _12yTo15y
 * @property {BtcCentsSatsUsdPattern} over15y
 */

/**
 * @typedef {Object} SeriesTree_Cointime_AgeRange_CoindaysCreated
 * @property {AverageBlockCumulativeSumPattern2<StoredF64>} under1h
 * @property {AverageBlockCumulativeSumPattern2<StoredF64>} _1hTo1d
 * @property {AverageBlockCumulativeSumPattern2<StoredF64>} _1dTo1w
 * @property {AverageBlockCumulativeSumPattern2<StoredF64>} _1wTo1m
 * @property {AverageBlockCumulativeSumPattern2<StoredF64>} _1mTo2m
 * @property {AverageBlockCumulativeSumPattern2<StoredF64>} _2mTo3m
 * @property {AverageBlockCumulativeSumPattern2<StoredF64>} _3mTo4m
 * @property {AverageBlockCumulativeSumPattern2<StoredF64>} _4mTo5m
 * @property {AverageBlockCumulativeSumPattern2<StoredF64>} _5mTo6m
 * @property {AverageBlockCumulativeSumPattern2<StoredF64>} _6mTo9m
 * @property {AverageBlockCumulativeSumPattern2<StoredF64>} _9mTo1y
 * @property {AverageBlockCumulativeSumPattern2<StoredF64>} _1yTo18m
 * @property {AverageBlockCumulativeSumPattern2<StoredF64>} _18mTo2y
 * @property {AverageBlockCumulativeSumPattern2<StoredF64>} _2yTo3y
 * @property {AverageBlockCumulativeSumPattern2<StoredF64>} _3yTo4y
 * @property {AverageBlockCumulativeSumPattern2<StoredF64>} _4yTo5y
 * @property {AverageBlockCumulativeSumPattern2<StoredF64>} _5yTo6y
 * @property {AverageBlockCumulativeSumPattern2<StoredF64>} _6yTo7y
 * @property {AverageBlockCumulativeSumPattern2<StoredF64>} _7yTo8y
 * @property {AverageBlockCumulativeSumPattern2<StoredF64>} _8yTo10y
 * @property {AverageBlockCumulativeSumPattern2<StoredF64>} _10yTo12y
 * @property {AverageBlockCumulativeSumPattern2<StoredF64>} _12yTo15y
 * @property {AverageBlockCumulativeSumPattern2<StoredF64>} over15y
 */

/**
 * @typedef {Object} SeriesTree_Cointime_Urpd
 * @property {CapitalizedCostSupplyPattern} all
 * @property {SeriesTree_Cointime_Urpd_Sth} sth
 * @property {SeriesTree_Cointime_Urpd_Lth} lth
 * @property {SeriesTree_Cointime_Urpd_Under4m} under4m
 * @property {SeriesTree_Cointime_Urpd_Under6m} under6m
 * @property {SeriesTree_Cointime_Urpd_Over4m} over4m
 * @property {SeriesTree_Cointime_Urpd_Over6m} over6m
 */

/**
 * @typedef {Object} SeriesTree_Cointime_Urpd_Sth
 * @property {PerPattern} costBasis
 * @property {CentsPpmRatioSatsUsdPattern} capitalizedPrice
 * @property {InTotalPattern} supplyDensity
 */

/**
 * @typedef {Object} SeriesTree_Cointime_Urpd_Lth
 * @property {PerPattern} costBasis
 * @property {CentsPpmRatioSatsUsdPattern} capitalizedPrice
 * @property {InTotalPattern} supplyDensity
 */

/**
 * @typedef {Object} SeriesTree_Cointime_Urpd_Under4m
 * @property {PerPattern} costBasis
 * @property {CentsPpmRatioSatsUsdPattern} capitalizedPrice
 * @property {InTotalPattern} supplyDensity
 */

/**
 * @typedef {Object} SeriesTree_Cointime_Urpd_Under6m
 * @property {PerPattern} costBasis
 * @property {CentsPpmRatioSatsUsdPattern} capitalizedPrice
 * @property {InTotalPattern} supplyDensity
 */

/**
 * @typedef {Object} SeriesTree_Cointime_Urpd_Over4m
 * @property {PerPattern} costBasis
 * @property {CentsPpmRatioSatsUsdPattern} capitalizedPrice
 * @property {InTotalPattern} supplyDensity
 */

/**
 * @typedef {Object} SeriesTree_Cointime_Urpd_Over6m
 * @property {PerPattern} costBasis
 * @property {CentsPpmRatioSatsUsdPattern} capitalizedPrice
 * @property {InTotalPattern} supplyDensity
 */

/**
 * @typedef {Object} SeriesTree_Cointime_Awake
 * @property {SeriesTree_Cointime_Awake_Supply} supply
 * @property {CentsUsdPattern} cap
 * @property {CentsPpmRatioSatsUsdPattern} price
 * @property {CentsPpmRatioSatsUsdPattern} capitalizedPrice
 */

/**
 * @typedef {Object} SeriesTree_Cointime_Awake_Supply
 * @property {SeriesPattern1<Bitcoin>} btc
 * @property {SeriesPattern1<Sats>} sats
 * @property {SeriesPattern1<Dollars>} usd
 * @property {SeriesPattern1<Cents>} cents
 * @property {SharePattern} inLoss
 */

/**
 * @typedef {Object} SeriesTree_Cointime_Sth
 * @property {SeriesTree_Cointime_Sth_Awake} awake
 * @property {SupplyPattern} dormant
 */

/**
 * @typedef {Object} SeriesTree_Cointime_Sth_Awake
 * @property {SeriesTree_Cointime_Sth_Awake_Supply} supply
 * @property {CentsUsdPattern} cap
 * @property {CentsPpmRatioSatsUsdPattern} price
 * @property {CentsPpmRatioSatsUsdPattern} capitalizedPrice
 */

/**
 * @typedef {Object} SeriesTree_Cointime_Sth_Awake_Supply
 * @property {SeriesPattern1<Bitcoin>} btc
 * @property {SeriesPattern1<Sats>} sats
 * @property {SeriesPattern1<Dollars>} usd
 * @property {SeriesPattern1<Cents>} cents
 * @property {SharePattern} inLoss
 */

/**
 * @typedef {Object} SeriesTree_Cointime_Lth
 * @property {SeriesTree_Cointime_Lth_Awake} awake
 * @property {SupplyPattern} dormant
 */

/**
 * @typedef {Object} SeriesTree_Cointime_Lth_Awake
 * @property {SeriesTree_Cointime_Lth_Awake_Supply} supply
 * @property {CentsUsdPattern} cap
 * @property {CentsPpmRatioSatsUsdPattern} price
 * @property {CentsPpmRatioSatsUsdPattern} capitalizedPrice
 */

/**
 * @typedef {Object} SeriesTree_Cointime_Lth_Awake_Supply
 * @property {SeriesPattern1<Bitcoin>} btc
 * @property {SeriesPattern1<Sats>} sats
 * @property {SeriesPattern1<Dollars>} usd
 * @property {SeriesPattern1<Cents>} cents
 * @property {SharePattern} inLoss
 */

/**
 * @typedef {Object} SeriesTree_Cointime_Supply
 * @property {BtcCentsSatsUsdPattern} vaulted
 * @property {SeriesTree_Cointime_Supply_Active} active
 */

/**
 * @typedef {Object} SeriesTree_Cointime_Supply_Active
 * @property {SeriesPattern1<Bitcoin>} btc
 * @property {SeriesPattern1<Sats>} sats
 * @property {SeriesPattern1<Dollars>} usd
 * @property {SeriesPattern1<Cents>} cents
 * @property {SeriesTree_Cointime_Supply_Active_InLoss} inLoss
 */

/**
 * @typedef {Object} SeriesTree_Cointime_Supply_Active_InLoss
 * @property {SeriesTree_Cointime_Supply_Active_InLoss_Share} share
 */

/**
 * @typedef {Object} SeriesTree_Cointime_Supply_Active_InLoss_Share
 * @property {SeriesPattern1<BoundedRatio>} bounded
 * @property {SeriesPattern1<StoredF64>} ratio
 */

/**
 * @typedef {Object} SeriesTree_Cointime_Value
 * @property {AverageBlockCumulativeSumPattern2<StoredF64>} destroyed
 * @property {AverageBlockCumulativeSumPattern2<StoredF64>} created
 * @property {AverageBlockCumulativeSumPattern2<StoredF64>} stored
 * @property {AverageBlockCumulativeSumPattern2<StoredF64>} vocdd
 */

/**
 * @typedef {Object} SeriesTree_Cointime_Cap
 * @property {CentsUsdPattern} thermo
 * @property {CentsUsdPattern} investor
 * @property {CentsUsdPattern} vaulted
 * @property {CentsUsdPattern} active
 * @property {CentsUsdPattern} cointime
 * @property {PpmRatioPattern} aviv
 */

/**
 * @typedef {Object} SeriesTree_Cointime_Prices
 * @property {CentsPpmRatioSatsUsdPattern} vaulted
 * @property {CentsPpmRatioSatsUsdPattern} active
 * @property {CentsPpmRatioSatsUsdPattern} trueMarketMean
 * @property {CentsPpmRatioSatsUsdPattern} cointime
 */

/**
 * @typedef {Object} SeriesTree_Cointime_Adjusted
 * @property {PercentPpmRatioPattern3} inflationRate
 * @property {SeriesPattern1<StoredF64>} txVelocityNative
 * @property {SeriesPattern1<StoredF64>} txVelocityFiat
 */

/**
 * @typedef {Object} SeriesTree_Cointime_ReserveRisk
 * @property {SeriesPattern1<StoredF64>} value
 * @property {SeriesPattern18<StoredF64>} vocddMedian1y
 * @property {SeriesPattern18<StoredF64>} hodlBank
 */

/**
 * @typedef {Object} SeriesTree_Coinflow
 * @property {SeriesTree_Coinflow_AgeRange} ageRange
 * @property {SeriesTree_Coinflow_Urpd} urpd
 * @property {SeriesTree_Coinflow_Supply} supply
 * @property {CentsUsdPattern} cap
 * @property {CentsPpmRatioSatsUsdPattern} price
 * @property {CentsPpmRatioSatsUsdPattern} capitalizedPrice
 * @property {SeriesTree_Coinflow_Sth} sth
 * @property {SeriesTree_Coinflow_Lth} lth
 * @property {CentsPpmRatioSatsUsdPattern} under4mPrice
 * @property {CentsPpmRatioSatsUsdPattern} under4mCapitalizedPrice
 * @property {CentsPpmRatioSatsUsdPattern} under6mPrice
 * @property {CentsPpmRatioSatsUsdPattern} under6mCapitalizedPrice
 * @property {CentsPpmRatioSatsUsdPattern} over4mPrice
 * @property {CentsPpmRatioSatsUsdPattern} over4mCapitalizedPrice
 * @property {CentsPpmRatioSatsUsdPattern} over6mPrice
 * @property {CentsPpmRatioSatsUsdPattern} over6mCapitalizedPrice
 */

/**
 * @typedef {Object} SeriesTree_Coinflow_AgeRange
 * @property {SeriesTree_Coinflow_AgeRange_SpendingRate} spendingRate
 * @property {SeriesTree_Coinflow_AgeRange_SpendingExposure} spendingExposure
 * @property {SeriesTree_Coinflow_AgeRange_Supply} supply
 */

/**
 * @typedef {Object} SeriesTree_Coinflow_AgeRange_SpendingRate
 * @property {SeriesPattern1<StoredF64>} under1h
 * @property {SeriesPattern1<StoredF64>} _1hTo1d
 * @property {SeriesPattern1<StoredF64>} _1dTo1w
 * @property {SeriesPattern1<StoredF64>} _1wTo1m
 * @property {SeriesPattern1<StoredF64>} _1mTo2m
 * @property {SeriesPattern1<StoredF64>} _2mTo3m
 * @property {SeriesPattern1<StoredF64>} _3mTo4m
 * @property {SeriesPattern1<StoredF64>} _4mTo5m
 * @property {SeriesPattern1<StoredF64>} _5mTo6m
 * @property {SeriesPattern1<StoredF64>} _6mTo9m
 * @property {SeriesPattern1<StoredF64>} _9mTo1y
 * @property {SeriesPattern1<StoredF64>} _1yTo18m
 * @property {SeriesPattern1<StoredF64>} _18mTo2y
 * @property {SeriesPattern1<StoredF64>} _2yTo3y
 * @property {SeriesPattern1<StoredF64>} _3yTo4y
 * @property {SeriesPattern1<StoredF64>} _4yTo5y
 * @property {SeriesPattern1<StoredF64>} _5yTo6y
 * @property {SeriesPattern1<StoredF64>} _6yTo7y
 * @property {SeriesPattern1<StoredF64>} _7yTo8y
 * @property {SeriesPattern1<StoredF64>} _8yTo10y
 * @property {SeriesPattern1<StoredF64>} _10yTo12y
 * @property {SeriesPattern1<StoredF64>} _12yTo15y
 * @property {SeriesPattern1<StoredF64>} over15y
 */

/**
 * @typedef {Object} SeriesTree_Coinflow_AgeRange_SpendingExposure
 * @property {SeriesPattern1<StoredF64>} under1h
 * @property {SeriesPattern1<StoredF64>} _1hTo1d
 * @property {SeriesPattern1<StoredF64>} _1dTo1w
 * @property {SeriesPattern1<StoredF64>} _1wTo1m
 * @property {SeriesPattern1<StoredF64>} _1mTo2m
 * @property {SeriesPattern1<StoredF64>} _2mTo3m
 * @property {SeriesPattern1<StoredF64>} _3mTo4m
 * @property {SeriesPattern1<StoredF64>} _4mTo5m
 * @property {SeriesPattern1<StoredF64>} _5mTo6m
 * @property {SeriesPattern1<StoredF64>} _6mTo9m
 * @property {SeriesPattern1<StoredF64>} _9mTo1y
 * @property {SeriesPattern1<StoredF64>} _1yTo18m
 * @property {SeriesPattern1<StoredF64>} _18mTo2y
 * @property {SeriesPattern1<StoredF64>} _2yTo3y
 * @property {SeriesPattern1<StoredF64>} _3yTo4y
 * @property {SeriesPattern1<StoredF64>} _4yTo5y
 * @property {SeriesPattern1<StoredF64>} _5yTo6y
 * @property {SeriesPattern1<StoredF64>} _6yTo7y
 * @property {SeriesPattern1<StoredF64>} _7yTo8y
 * @property {SeriesPattern1<StoredF64>} _8yTo10y
 * @property {SeriesPattern1<StoredF64>} _10yTo12y
 * @property {SeriesPattern1<StoredF64>} _12yTo15y
 * @property {SeriesPattern1<StoredF64>} over15y
 * @property {SeriesTree_Coinflow_AgeRange_SpendingExposure_Mobility} mobility
 */

/**
 * @typedef {Object} SeriesTree_Coinflow_AgeRange_SpendingExposure_Mobility
 * @property {SeriesPattern1<StoredF64>} under1h
 * @property {SeriesPattern1<StoredF64>} _1hTo1d
 * @property {SeriesPattern1<StoredF64>} _1dTo1w
 * @property {SeriesPattern1<StoredF64>} _1wTo1m
 * @property {SeriesPattern1<StoredF64>} _1mTo2m
 * @property {SeriesPattern1<StoredF64>} _2mTo3m
 * @property {SeriesPattern1<StoredF64>} _3mTo4m
 * @property {SeriesPattern1<StoredF64>} _4mTo5m
 * @property {SeriesPattern1<StoredF64>} _5mTo6m
 * @property {SeriesPattern1<StoredF64>} _6mTo9m
 * @property {SeriesPattern1<StoredF64>} _9mTo1y
 * @property {SeriesPattern1<StoredF64>} _1yTo18m
 * @property {SeriesPattern1<StoredF64>} _18mTo2y
 * @property {SeriesPattern1<StoredF64>} _2yTo3y
 * @property {SeriesPattern1<StoredF64>} _3yTo4y
 * @property {SeriesPattern1<StoredF64>} _4yTo5y
 * @property {SeriesPattern1<StoredF64>} _5yTo6y
 * @property {SeriesPattern1<StoredF64>} _6yTo7y
 * @property {SeriesPattern1<StoredF64>} _7yTo8y
 * @property {SeriesPattern1<StoredF64>} _8yTo10y
 * @property {SeriesPattern1<StoredF64>} _10yTo12y
 * @property {SeriesPattern1<StoredF64>} _12yTo15y
 * @property {SeriesPattern1<StoredF64>} over15y
 */

/**
 * @typedef {Object} SeriesTree_Coinflow_AgeRange_Supply
 * @property {SeriesTree_Coinflow_AgeRange_Supply_Mobile} mobile
 * @property {SeriesTree_Coinflow_AgeRange_Supply_Immobile} immobile
 */

/**
 * @typedef {Object} SeriesTree_Coinflow_AgeRange_Supply_Mobile
 * @property {BtcCentsSatsUsdPattern} under1h
 * @property {BtcCentsSatsUsdPattern} _1hTo1d
 * @property {BtcCentsSatsUsdPattern} _1dTo1w
 * @property {BtcCentsSatsUsdPattern} _1wTo1m
 * @property {BtcCentsSatsUsdPattern} _1mTo2m
 * @property {BtcCentsSatsUsdPattern} _2mTo3m
 * @property {BtcCentsSatsUsdPattern} _3mTo4m
 * @property {BtcCentsSatsUsdPattern} _4mTo5m
 * @property {BtcCentsSatsUsdPattern} _5mTo6m
 * @property {BtcCentsSatsUsdPattern} _6mTo9m
 * @property {BtcCentsSatsUsdPattern} _9mTo1y
 * @property {BtcCentsSatsUsdPattern} _1yTo18m
 * @property {BtcCentsSatsUsdPattern} _18mTo2y
 * @property {BtcCentsSatsUsdPattern} _2yTo3y
 * @property {BtcCentsSatsUsdPattern} _3yTo4y
 * @property {BtcCentsSatsUsdPattern} _4yTo5y
 * @property {BtcCentsSatsUsdPattern} _5yTo6y
 * @property {BtcCentsSatsUsdPattern} _6yTo7y
 * @property {BtcCentsSatsUsdPattern} _7yTo8y
 * @property {BtcCentsSatsUsdPattern} _8yTo10y
 * @property {BtcCentsSatsUsdPattern} _10yTo12y
 * @property {BtcCentsSatsUsdPattern} _12yTo15y
 * @property {BtcCentsSatsUsdPattern} over15y
 */

/**
 * @typedef {Object} SeriesTree_Coinflow_AgeRange_Supply_Immobile
 * @property {BtcCentsSatsUsdPattern} under1h
 * @property {BtcCentsSatsUsdPattern} _1hTo1d
 * @property {BtcCentsSatsUsdPattern} _1dTo1w
 * @property {BtcCentsSatsUsdPattern} _1wTo1m
 * @property {BtcCentsSatsUsdPattern} _1mTo2m
 * @property {BtcCentsSatsUsdPattern} _2mTo3m
 * @property {BtcCentsSatsUsdPattern} _3mTo4m
 * @property {BtcCentsSatsUsdPattern} _4mTo5m
 * @property {BtcCentsSatsUsdPattern} _5mTo6m
 * @property {BtcCentsSatsUsdPattern} _6mTo9m
 * @property {BtcCentsSatsUsdPattern} _9mTo1y
 * @property {BtcCentsSatsUsdPattern} _1yTo18m
 * @property {BtcCentsSatsUsdPattern} _18mTo2y
 * @property {BtcCentsSatsUsdPattern} _2yTo3y
 * @property {BtcCentsSatsUsdPattern} _3yTo4y
 * @property {BtcCentsSatsUsdPattern} _4yTo5y
 * @property {BtcCentsSatsUsdPattern} _5yTo6y
 * @property {BtcCentsSatsUsdPattern} _6yTo7y
 * @property {BtcCentsSatsUsdPattern} _7yTo8y
 * @property {BtcCentsSatsUsdPattern} _8yTo10y
 * @property {BtcCentsSatsUsdPattern} _10yTo12y
 * @property {BtcCentsSatsUsdPattern} _12yTo15y
 * @property {BtcCentsSatsUsdPattern} over15y
 */

/**
 * @typedef {Object} SeriesTree_Coinflow_Urpd
 * @property {CapitalizedCostSupplyPattern} all
 * @property {SeriesTree_Coinflow_Urpd_Sth} sth
 * @property {SeriesTree_Coinflow_Urpd_Lth} lth
 * @property {SeriesTree_Coinflow_Urpd_Under4m} under4m
 * @property {SeriesTree_Coinflow_Urpd_Under6m} under6m
 * @property {SeriesTree_Coinflow_Urpd_Over4m} over4m
 * @property {SeriesTree_Coinflow_Urpd_Over6m} over6m
 */

/**
 * @typedef {Object} SeriesTree_Coinflow_Urpd_Sth
 * @property {PerPattern} costBasis
 * @property {CentsPpmRatioSatsUsdPattern} capitalizedPrice
 * @property {InTotalPattern} supplyDensity
 */

/**
 * @typedef {Object} SeriesTree_Coinflow_Urpd_Lth
 * @property {PerPattern} costBasis
 * @property {CentsPpmRatioSatsUsdPattern} capitalizedPrice
 * @property {InTotalPattern} supplyDensity
 */

/**
 * @typedef {Object} SeriesTree_Coinflow_Urpd_Under4m
 * @property {PerPattern} costBasis
 * @property {CentsPpmRatioSatsUsdPattern} capitalizedPrice
 * @property {InTotalPattern} supplyDensity
 */

/**
 * @typedef {Object} SeriesTree_Coinflow_Urpd_Under6m
 * @property {PerPattern} costBasis
 * @property {CentsPpmRatioSatsUsdPattern} capitalizedPrice
 * @property {InTotalPattern} supplyDensity
 */

/**
 * @typedef {Object} SeriesTree_Coinflow_Urpd_Over4m
 * @property {PerPattern} costBasis
 * @property {CentsPpmRatioSatsUsdPattern} capitalizedPrice
 * @property {InTotalPattern} supplyDensity
 */

/**
 * @typedef {Object} SeriesTree_Coinflow_Urpd_Over6m
 * @property {PerPattern} costBasis
 * @property {CentsPpmRatioSatsUsdPattern} capitalizedPrice
 * @property {InTotalPattern} supplyDensity
 */

/**
 * @typedef {Object} SeriesTree_Coinflow_Supply
 * @property {SeriesTree_Coinflow_Supply_Mobile} mobile
 * @property {BtcCentsSatsUsdPattern} immobile
 */

/**
 * @typedef {Object} SeriesTree_Coinflow_Supply_Mobile
 * @property {SeriesPattern1<Bitcoin>} btc
 * @property {SeriesPattern1<Sats>} sats
 * @property {SeriesPattern1<Dollars>} usd
 * @property {SeriesPattern1<Cents>} cents
 * @property {SharePattern} inLoss
 */

/**
 * @typedef {Object} SeriesTree_Coinflow_Sth
 * @property {SeriesTree_Coinflow_Sth_Supply} supply
 * @property {CentsUsdPattern} cap
 * @property {CentsPpmRatioSatsUsdPattern} price
 * @property {CentsPpmRatioSatsUsdPattern} capitalizedPrice
 */

/**
 * @typedef {Object} SeriesTree_Coinflow_Sth_Supply
 * @property {SeriesTree_Coinflow_Sth_Supply_Mobile} mobile
 * @property {BtcCentsSatsUsdPattern} immobile
 */

/**
 * @typedef {Object} SeriesTree_Coinflow_Sth_Supply_Mobile
 * @property {SeriesPattern1<Bitcoin>} btc
 * @property {SeriesPattern1<Sats>} sats
 * @property {SeriesPattern1<Dollars>} usd
 * @property {SeriesPattern1<Cents>} cents
 * @property {SharePattern} inLoss
 */

/**
 * @typedef {Object} SeriesTree_Coinflow_Lth
 * @property {SeriesTree_Coinflow_Lth_Supply} supply
 * @property {CentsUsdPattern} cap
 * @property {CentsPpmRatioSatsUsdPattern} price
 * @property {CentsPpmRatioSatsUsdPattern} capitalizedPrice
 */

/**
 * @typedef {Object} SeriesTree_Coinflow_Lth_Supply
 * @property {SeriesTree_Coinflow_Lth_Supply_Mobile} mobile
 * @property {BtcCentsSatsUsdPattern} immobile
 */

/**
 * @typedef {Object} SeriesTree_Coinflow_Lth_Supply_Mobile
 * @property {SeriesPattern1<Bitcoin>} btc
 * @property {SeriesPattern1<Sats>} sats
 * @property {SeriesPattern1<Dollars>} usd
 * @property {SeriesPattern1<Cents>} cents
 * @property {SharePattern} inLoss
 */

/**
 * @typedef {Object} SeriesTree_Bedrock
 * @property {FloorLevelSupplyPattern} raw
 * @property {FloorLevelSupplyPattern} cointime
 * @property {FloorLevelSupplyPattern} coinflow
 */

/**
 * @typedef {Object} SeriesTree_CapitalSentiment
 * @property {SeriesPattern1<StoredBool>} isLong
 * @property {SeriesPattern1<StoredBool>} isShort
 * @property {SeriesPattern1<CapitalSentimentPhase>} phase
 * @property {SeriesPattern1<StoredI8>} score
 */

/**
 * @typedef {Object} SeriesTree_RarityMeter
 * @property {SeriesTree_RarityMeter_Components} components
 * @property {SeriesTree_RarityMeter_Extremes} extremes
 * @property {IndexPct0Pct1Pct10Pct2Pct20Pct30Pct40Pct5Pct50Pct60Pct70Pct80Pct90Pct95Pct98Pct99ScorePattern} full
 * @property {IndexPct0Pct1Pct10Pct2Pct20Pct30Pct40Pct5Pct50Pct60Pct70Pct80Pct90Pct95Pct98Pct99ScorePattern} fullV2
 * @property {IndexPct0Pct1Pct10Pct2Pct20Pct30Pct40Pct5Pct50Pct60Pct70Pct80Pct90Pct95Pct98Pct99ScorePattern} local
 * @property {IndexPct0Pct1Pct10Pct2Pct20Pct30Pct40Pct5Pct50Pct60Pct70Pct80Pct90Pct95Pct98Pct99ScorePattern} localV2
 * @property {IndexPct0Pct1Pct10Pct2Pct20Pct30Pct40Pct5Pct50Pct60Pct70Pct80Pct90Pct95Pct98Pct99ScorePattern} cycle
 * @property {IndexPct0Pct1Pct10Pct2Pct20Pct30Pct40Pct5Pct50Pct60Pct70Pct80Pct90Pct95Pct98Pct99ScorePattern} cycleV2
 */

/**
 * @typedef {Object} SeriesTree_RarityMeter_Components
 * @property {Pct0Pct1Pct10Pct2Pct20Pct30Pct40Pct5Pct50Pct60Pct70Pct80Pct90Pct95Pct98Pct99Pattern} realizedPrice
 * @property {Pct0Pct1Pct10Pct2Pct20Pct30Pct40Pct5Pct50Pct60Pct70Pct80Pct90Pct95Pct98Pct99Pattern} capitalizedPrice
 * @property {CentsPct0Pct1Pct10Pct2Pct20Pct30Pct40Pct5Pct50Pct60Pct70Pct80Pct90Pct95Pct98Pct99PpmRatioSatsUsdPattern} medianPriceBtcWeighted
 * @property {CentsPct0Pct1Pct10Pct2Pct20Pct30Pct40Pct5Pct50Pct60Pct70Pct80Pct90Pct95Pct98Pct99PpmRatioSatsUsdPattern} medianPriceUsdWeighted
 * @property {CentsPct0Pct1Pct10Pct2Pct20Pct30Pct40Pct5Pct50Pct60Pct70Pct80Pct90Pct95Pct98Pct99PpmRatioSatsUsdPattern} sthMedianPriceBtcWeighted
 * @property {CentsPct0Pct1Pct10Pct2Pct20Pct30Pct40Pct5Pct50Pct60Pct70Pct80Pct90Pct95Pct98Pct99PpmRatioSatsUsdPattern} sthMedianPriceUsdWeighted
 * @property {CentsPct0Pct1Pct10Pct2Pct20Pct30Pct40Pct5Pct50Pct60Pct70Pct80Pct90Pct95Pct98Pct99PpmRatioSatsUsdPattern} lthMedianPriceBtcWeighted
 * @property {CentsPct0Pct1Pct10Pct2Pct20Pct30Pct40Pct5Pct50Pct60Pct70Pct80Pct90Pct95Pct98Pct99PpmRatioSatsUsdPattern} lthMedianPriceUsdWeighted
 * @property {CentsPct0Pct1Pct10Pct2Pct20Pct30Pct40Pct5Pct50Pct60Pct70Pct80Pct90Pct95Pct98Pct99PpmRatioSatsUsdPattern} cointimeMedianPriceBtcWeighted
 * @property {CentsPct0Pct1Pct10Pct2Pct20Pct30Pct40Pct5Pct50Pct60Pct70Pct80Pct90Pct95Pct98Pct99PpmRatioSatsUsdPattern} cointimeMedianPriceUsdWeighted
 * @property {CentsPct0Pct1Pct10Pct2Pct20Pct30Pct40Pct5Pct50Pct60Pct70Pct80Pct90Pct95Pct98Pct99PpmRatioSatsUsdPattern} coinflowMedianPriceBtcWeighted
 * @property {CentsPct0Pct1Pct10Pct2Pct20Pct30Pct40Pct5Pct50Pct60Pct70Pct80Pct90Pct95Pct98Pct99PpmRatioSatsUsdPattern} coinflowMedianPriceUsdWeighted
 * @property {CentsPct0Pct1Pct10Pct2Pct20Pct30Pct40Pct5Pct50Pct60Pct70Pct80Pct90Pct95Pct98Pct99PpmRatioSatsUsdPattern} sthCointimeMedianPriceBtcWeighted
 * @property {CentsPct0Pct1Pct10Pct2Pct20Pct30Pct40Pct5Pct50Pct60Pct70Pct80Pct90Pct95Pct98Pct99PpmRatioSatsUsdPattern} sthCointimeMedianPriceUsdWeighted
 * @property {CentsPct0Pct1Pct10Pct2Pct20Pct30Pct40Pct5Pct50Pct60Pct70Pct80Pct90Pct95Pct98Pct99PpmRatioSatsUsdPattern} lthCointimeMedianPriceBtcWeighted
 * @property {CentsPct0Pct1Pct10Pct2Pct20Pct30Pct40Pct5Pct50Pct60Pct70Pct80Pct90Pct95Pct98Pct99PpmRatioSatsUsdPattern} lthCointimeMedianPriceUsdWeighted
 * @property {CentsPct0Pct1Pct10Pct2Pct20Pct30Pct40Pct5Pct50Pct60Pct70Pct80Pct90Pct95Pct98Pct99PpmRatioSatsUsdPattern} sthCoinflowMedianPriceBtcWeighted
 * @property {CentsPct0Pct1Pct10Pct2Pct20Pct30Pct40Pct5Pct50Pct60Pct70Pct80Pct90Pct95Pct98Pct99PpmRatioSatsUsdPattern} sthCoinflowMedianPriceUsdWeighted
 * @property {CentsPct0Pct1Pct10Pct2Pct20Pct30Pct40Pct5Pct50Pct60Pct70Pct80Pct90Pct95Pct98Pct99PpmRatioSatsUsdPattern} lthCoinflowMedianPriceBtcWeighted
 * @property {CentsPct0Pct1Pct10Pct2Pct20Pct30Pct40Pct5Pct50Pct60Pct70Pct80Pct90Pct95Pct98Pct99PpmRatioSatsUsdPattern} lthCoinflowMedianPriceUsdWeighted
 * @property {Pct0Pct1Pct10Pct2Pct20Pct30Pct40Pct5Pct50Pct60Pct70Pct80Pct90Pct95Pct98Pct99Pattern} sthRealizedPrice
 * @property {Pct0Pct1Pct10Pct2Pct20Pct30Pct40Pct5Pct50Pct60Pct70Pct80Pct90Pct95Pct98Pct99Pattern} sthCapitalizedPrice
 * @property {Pct0Pct1Pct10Pct2Pct20Pct30Pct40Pct5Pct50Pct60Pct70Pct80Pct90Pct95Pct98Pct99Pattern} lthRealizedPrice
 * @property {Pct0Pct1Pct10Pct2Pct20Pct30Pct40Pct5Pct50Pct60Pct70Pct80Pct90Pct95Pct98Pct99Pattern} lthCapitalizedPrice
 * @property {Pct0Pct1Pct10Pct2Pct20Pct30Pct40Pct5Pct50Pct60Pct70Pct80Pct90Pct95Pct98Pct99Pattern} over6mRealizedPrice
 * @property {Pct0Pct1Pct10Pct2Pct20Pct30Pct40Pct5Pct50Pct60Pct70Pct80Pct90Pct95Pct98Pct99Pattern} over4mRealizedPrice
 * @property {Pct0Pct1Pct10Pct2Pct20Pct30Pct40Pct5Pct50Pct60Pct70Pct80Pct90Pct95Pct98Pct99Pattern} under4mRealizedPrice
 * @property {Pct0Pct1Pct10Pct2Pct20Pct30Pct40Pct5Pct50Pct60Pct70Pct80Pct90Pct95Pct98Pct99Pattern} under6mRealizedPrice
 * @property {Pct0Pct1Pct10Pct2Pct20Pct30Pct40Pct5Pct50Pct60Pct70Pct80Pct90Pct95Pct98Pct99Pattern} under4mCapitalizedPrice
 * @property {Pct0Pct1Pct10Pct2Pct20Pct30Pct40Pct5Pct50Pct60Pct70Pct80Pct90Pct95Pct98Pct99Pattern} under6mCapitalizedPrice
 * @property {Pct0Pct1Pct10Pct2Pct20Pct30Pct40Pct5Pct50Pct60Pct70Pct80Pct90Pct95Pct98Pct99Pattern} vaultedPrice
 * @property {Pct0Pct1Pct10Pct2Pct20Pct30Pct40Pct5Pct50Pct60Pct70Pct80Pct90Pct95Pct98Pct99Pattern} activePrice
 * @property {Pct0Pct1Pct10Pct2Pct20Pct30Pct40Pct5Pct50Pct60Pct70Pct80Pct90Pct95Pct98Pct99Pattern} trueMarketMeanPrice
 * @property {Pct0Pct1Pct10Pct2Pct20Pct30Pct40Pct5Pct50Pct60Pct70Pct80Pct90Pct95Pct98Pct99Pattern} cointimePrice
 * @property {Pct0Pct1Pct10Pct2Pct20Pct30Pct40Pct5Pct50Pct60Pct70Pct80Pct90Pct95Pct98Pct99Pattern} awakePrice
 * @property {Pct0Pct1Pct10Pct2Pct20Pct30Pct40Pct5Pct50Pct60Pct70Pct80Pct90Pct95Pct98Pct99Pattern} coinflowPrice
 */

/**
 * @typedef {Object} SeriesTree_RarityMeter_Extremes
 * @property {SeriesTree_RarityMeter_Extremes_CoinsInLoss} coinsInLoss
 * @property {RankTailThresholdPattern} profitTaking
 * @property {RankTailThresholdPattern} capitulation
 * @property {RankTailThresholdPattern} peakRegret
 * @property {SeriesTree_RarityMeter_Extremes_SellerExhaustion} sellerExhaustion
 */

/**
 * @typedef {Object} SeriesTree_RarityMeter_Extremes_CoinsInLoss
 * @property {SeriesPattern1<Bitcoin>} thresholdPct01
 * @property {SeriesPattern1<Bitcoin>} thresholdPct005
 * @property {SeriesPattern1<Bitcoin>} thresholdPct0025
 * @property {PercentPpmRatioPattern2} tail
 * @property {SeriesPattern1<StoredU8>} rank
 */

/**
 * @typedef {Object} SeriesTree_RarityMeter_Extremes_SellerExhaustion
 * @property {SeriesPattern1<StoredF32>} thresholdPct01
 * @property {SeriesPattern1<StoredF32>} thresholdPct005
 * @property {SeriesPattern1<StoredF32>} thresholdPct0025
 * @property {PercentPpmRatioPattern2} tail
 * @property {SeriesPattern1<StoredU8>} rank
 */

/**
 * @typedef {Object} SeriesTree_Constants
 * @property {SeriesPattern1<StoredU16>} _0
 * @property {SeriesPattern1<StoredU16>} _1
 * @property {SeriesPattern1<StoredU16>} _2
 * @property {SeriesPattern1<StoredU16>} _3
 * @property {SeriesPattern1<StoredU16>} _4
 * @property {SeriesPattern1<StoredU16>} _20
 * @property {SeriesPattern1<StoredU16>} _30
 * @property {SeriesPattern1<StoredF32>} _382
 * @property {SeriesPattern1<StoredU16>} _50
 * @property {SeriesPattern1<StoredF32>} _618
 * @property {SeriesPattern1<StoredU16>} _70
 * @property {SeriesPattern1<StoredU16>} _80
 * @property {SeriesPattern1<StoredU16>} _100
 * @property {SeriesPattern1<StoredU16>} _600
 * @property {SeriesPattern1<StoredI8>} minus1
 * @property {SeriesPattern1<StoredI8>} minus2
 * @property {SeriesPattern1<StoredI8>} minus3
 * @property {SeriesPattern1<StoredI8>} minus4
 */

/**
 * @typedef {Object} SeriesTree_Mappings
 * @property {SeriesTree_Mappings_Addr} addr
 * @property {SeriesTree_Mappings_Height} height
 * @property {SeriesTree_Mappings_Epoch} epoch
 * @property {SeriesTree_Mappings_Halving} halving
 * @property {SeriesTree_Mappings_Minute10} minute10
 * @property {SeriesTree_Mappings_Minute30} minute30
 * @property {SeriesTree_Mappings_Hour1} hour1
 * @property {SeriesTree_Mappings_Hour4} hour4
 * @property {SeriesTree_Mappings_Hour12} hour12
 * @property {SeriesTree_Mappings_Day1} day1
 * @property {SeriesTree_Mappings_Day3} day3
 * @property {SeriesTree_Mappings_Week1} week1
 * @property {SeriesTree_Mappings_Month1} month1
 * @property {SeriesTree_Mappings_Month3} month3
 * @property {SeriesTree_Mappings_Month6} month6
 * @property {SeriesTree_Mappings_Year1} year1
 * @property {SeriesTree_Mappings_Year10} year10
 * @property {SeriesTree_Mappings_TxIndex} txIndex
 * @property {SeriesTree_Mappings_TxinIndex} txinIndex
 * @property {SeriesTree_Mappings_TxoutIndex} txoutIndex
 * @property {SeriesTree_Mappings_Timestamp} timestamp
 */

/**
 * @typedef {Object} SeriesTree_Mappings_Addr
 * @property {SeriesTree_Mappings_Addr_P2pk33} p2pk33
 * @property {SeriesTree_Mappings_Addr_P2pk65} p2pk65
 * @property {SeriesTree_Mappings_Addr_P2pkh} p2pkh
 * @property {SeriesTree_Mappings_Addr_P2sh} p2sh
 * @property {SeriesTree_Mappings_Addr_P2tr} p2tr
 * @property {SeriesTree_Mappings_Addr_P2wpkh} p2wpkh
 * @property {SeriesTree_Mappings_Addr_P2wsh} p2wsh
 * @property {SeriesTree_Mappings_Addr_P2a} p2a
 * @property {SeriesTree_Mappings_Addr_P2ms} p2ms
 * @property {SeriesTree_Mappings_Addr_Empty} empty
 * @property {SeriesTree_Mappings_Addr_Unknown} unknown
 * @property {SeriesTree_Mappings_Addr_OpReturn} opReturn
 */

/**
 * @typedef {Object} SeriesTree_Mappings_Addr_P2pk33
 * @property {SeriesPattern26<P2PK33AddrIndex>} identity
 * @property {SeriesPattern26<Addr>} addr
 */

/**
 * @typedef {Object} SeriesTree_Mappings_Addr_P2pk65
 * @property {SeriesPattern27<P2PK65AddrIndex>} identity
 * @property {SeriesPattern27<Addr>} addr
 */

/**
 * @typedef {Object} SeriesTree_Mappings_Addr_P2pkh
 * @property {SeriesPattern28<P2PKHAddrIndex>} identity
 * @property {SeriesPattern28<Addr>} addr
 */

/**
 * @typedef {Object} SeriesTree_Mappings_Addr_P2sh
 * @property {SeriesPattern29<P2SHAddrIndex>} identity
 * @property {SeriesPattern29<Addr>} addr
 */

/**
 * @typedef {Object} SeriesTree_Mappings_Addr_P2tr
 * @property {SeriesPattern30<P2TRAddrIndex>} identity
 * @property {SeriesPattern30<Addr>} addr
 */

/**
 * @typedef {Object} SeriesTree_Mappings_Addr_P2wpkh
 * @property {SeriesPattern31<P2WPKHAddrIndex>} identity
 * @property {SeriesPattern31<Addr>} addr
 */

/**
 * @typedef {Object} SeriesTree_Mappings_Addr_P2wsh
 * @property {SeriesPattern32<P2WSHAddrIndex>} identity
 * @property {SeriesPattern32<Addr>} addr
 */

/**
 * @typedef {Object} SeriesTree_Mappings_Addr_P2a
 * @property {SeriesPattern24<P2AAddrIndex>} identity
 * @property {SeriesPattern24<Addr>} addr
 */

/**
 * @typedef {Object} SeriesTree_Mappings_Addr_P2ms
 * @property {SeriesPattern25<P2MSOutputIndex>} identity
 */

/**
 * @typedef {Object} SeriesTree_Mappings_Addr_Empty
 * @property {SeriesPattern22<EmptyOutputIndex>} identity
 */

/**
 * @typedef {Object} SeriesTree_Mappings_Addr_Unknown
 * @property {SeriesPattern33<UnknownOutputIndex>} identity
 */

/**
 * @typedef {Object} SeriesTree_Mappings_Addr_OpReturn
 * @property {SeriesPattern23<OpReturnIndex>} identity
 */

/**
 * @typedef {Object} SeriesTree_Mappings_Height
 * @property {SeriesPattern18<Minute10>} minute10
 * @property {SeriesPattern18<Minute30>} minute30
 * @property {SeriesPattern18<Hour1>} hour1
 * @property {SeriesPattern18<Hour4>} hour4
 * @property {SeriesPattern18<Hour12>} hour12
 * @property {SeriesPattern18<Day1>} day1
 * @property {SeriesPattern18<Day3>} day3
 * @property {SeriesPattern18<Epoch>} epoch
 * @property {SeriesPattern18<Halving>} halving
 * @property {SeriesPattern18<Week1>} week1
 * @property {SeriesPattern18<Month1>} month1
 * @property {SeriesPattern18<Month3>} month3
 * @property {SeriesPattern18<Month6>} month6
 * @property {SeriesPattern18<Year1>} year1
 * @property {SeriesPattern18<Year10>} year10
 * @property {SeriesPattern18<StoredU64>} txIndexCount
 */

/**
 * @typedef {Object} SeriesTree_Mappings_Epoch
 * @property {SeriesPattern17<Height>} firstHeight
 */

/**
 * @typedef {Object} SeriesTree_Mappings_Halving
 * @property {SeriesPattern16<Height>} firstHeight
 */

/**
 * @typedef {Object} SeriesTree_Mappings_Minute10
 * @property {SeriesPattern3<Height>} firstHeight
 */

/**
 * @typedef {Object} SeriesTree_Mappings_Minute30
 * @property {SeriesPattern4<Height>} firstHeight
 */

/**
 * @typedef {Object} SeriesTree_Mappings_Hour1
 * @property {SeriesPattern5<Height>} firstHeight
 */

/**
 * @typedef {Object} SeriesTree_Mappings_Hour4
 * @property {SeriesPattern6<Height>} firstHeight
 */

/**
 * @typedef {Object} SeriesTree_Mappings_Hour12
 * @property {SeriesPattern7<Height>} firstHeight
 */

/**
 * @typedef {Object} SeriesTree_Mappings_Day1
 * @property {SeriesPattern8<Date>} date
 * @property {SeriesPattern8<Height>} firstHeight
 */

/**
 * @typedef {Object} SeriesTree_Mappings_Day3
 * @property {SeriesPattern9<Date>} date
 * @property {SeriesPattern9<Height>} firstHeight
 */

/**
 * @typedef {Object} SeriesTree_Mappings_Week1
 * @property {SeriesPattern10<Date>} date
 * @property {SeriesPattern10<Height>} firstHeight
 */

/**
 * @typedef {Object} SeriesTree_Mappings_Month1
 * @property {SeriesPattern11<Date>} date
 * @property {SeriesPattern11<Height>} firstHeight
 */

/**
 * @typedef {Object} SeriesTree_Mappings_Month3
 * @property {SeriesPattern12<Date>} date
 * @property {SeriesPattern12<Height>} firstHeight
 */

/**
 * @typedef {Object} SeriesTree_Mappings_Month6
 * @property {SeriesPattern13<Date>} date
 * @property {SeriesPattern13<Height>} firstHeight
 */

/**
 * @typedef {Object} SeriesTree_Mappings_Year1
 * @property {SeriesPattern14<Date>} date
 * @property {SeriesPattern14<Height>} firstHeight
 */

/**
 * @typedef {Object} SeriesTree_Mappings_Year10
 * @property {SeriesPattern15<Date>} date
 * @property {SeriesPattern15<Height>} firstHeight
 */

/**
 * @typedef {Object} SeriesTree_Mappings_TxIndex
 * @property {SeriesPattern19<TxIndex>} identity
 * @property {SeriesPattern19<StoredU64>} inputCount
 * @property {SeriesPattern19<StoredU64>} outputCount
 */

/**
 * @typedef {Object} SeriesTree_Mappings_TxinIndex
 * @property {SeriesPattern20<TxInIndex>} identity
 */

/**
 * @typedef {Object} SeriesTree_Mappings_TxoutIndex
 * @property {SeriesPattern21<TxOutIndex>} identity
 */

/**
 * @typedef {Object} SeriesTree_Mappings_Timestamp
 * @property {SeriesPattern18<Timestamp>} monotonic
 * @property {SeriesPattern2<Timestamp>} resolutions
 */

/**
 * @typedef {Object} SeriesTree_Indicators
 * @property {BpsRatioPattern} puellMultiple
 * @property {BpsRatioPattern} nvt
 * @property {PercentPpmRatioPattern2} gini
 * @property {SeriesTree_Indicators_RhodlRatio} rhodlRatio
 * @property {BpsRatioPattern} thermoCapMultiple
 * @property {SeriesPattern1<StoredF32>} coindaysDestroyedSupplyAdj
 * @property {SeriesPattern1<StoredF32>} coinyearsDestroyedSupplyAdj
 * @property {SeriesTree_Indicators_Dormancy} dormancy
 * @property {SeriesPattern1<StoredF32>} stockToFlow
 * @property {SeriesPattern1<StoredF32>} sellerExhaustion
 */

/**
 * @typedef {Object} SeriesTree_Indicators_RhodlRatio
 * @property {SeriesPattern1<PartsPerMillion64>} ppm
 * @property {SeriesPattern1<StoredF32>} ratio
 */

/**
 * @typedef {Object} SeriesTree_Indicators_Dormancy
 * @property {SeriesPattern1<StoredF32>} supplyAdj
 * @property {SeriesPattern1<StoredF32>} flow
 */

/**
 * @typedef {Object} SeriesTree_Market
 * @property {SeriesTree_Market_Ath} ath
 * @property {SeriesTree_Market_Lookback} lookback
 * @property {SeriesTree_Market_Returns} returns
 * @property {_1m1w1y24hPattern6<StoredF32>} volatility
 * @property {SeriesTree_Market_Range} range
 * @property {SeriesTree_Market_MovingAverage} movingAverage
 * @property {SeriesTree_Market_Technical} technical
 */

/**
 * @typedef {Object} SeriesTree_Market_Ath
 * @property {CentsSatsUsdPattern} high
 * @property {PercentPpmRatioPattern3} drawdown
 * @property {SeriesPattern1<StoredF32>} daysSince
 * @property {SeriesPattern1<StoredF32>} yearsSince
 * @property {SeriesPattern1<StoredF32>} maxDaysBetween
 * @property {SeriesPattern1<StoredF32>} maxYearsBetween
 */

/**
 * @typedef {Object} SeriesTree_Market_Lookback
 * @property {CentsSatsUsdPattern} _24h
 * @property {CentsSatsUsdPattern} _1w
 * @property {CentsSatsUsdPattern} _1m
 * @property {CentsSatsUsdPattern} _3m
 * @property {CentsSatsUsdPattern} _6m
 * @property {CentsSatsUsdPattern} _1y
 * @property {CentsSatsUsdPattern} _2y
 * @property {CentsSatsUsdPattern} _3y
 * @property {CentsSatsUsdPattern} _4y
 * @property {CentsSatsUsdPattern} _5y
 * @property {CentsSatsUsdPattern} _6y
 * @property {CentsSatsUsdPattern} _8y
 * @property {CentsSatsUsdPattern} _10y
 */

/**
 * @typedef {Object} SeriesTree_Market_Returns
 * @property {SeriesTree_Market_Returns_Periods} periods
 * @property {SeriesTree_Market_Returns_Cagr} cagr
 * @property {SeriesTree_Market_Returns_Sd24h} sd24h
 */

/**
 * @typedef {Object} SeriesTree_Market_Returns_Periods
 * @property {PercentPpmRatioPattern} _24h
 * @property {PercentPpmRatioPattern} _1w
 * @property {PercentPpmRatioPattern} _1m
 * @property {PercentPpmRatioPattern} _3m
 * @property {PercentPpmRatioPattern} _6m
 * @property {PercentPpmRatioPattern} _1y
 * @property {PercentPpmRatioPattern} _2y
 * @property {PercentPpmRatioPattern} _3y
 * @property {PercentPpmRatioPattern} _4y
 * @property {PercentPpmRatioPattern} _5y
 * @property {PercentPpmRatioPattern} _6y
 * @property {PercentPpmRatioPattern} _8y
 * @property {PercentPpmRatioPattern} _10y
 */

/**
 * @typedef {Object} SeriesTree_Market_Returns_Cagr
 * @property {PercentPpmRatioPattern} _2y
 * @property {PercentPpmRatioPattern} _3y
 * @property {PercentPpmRatioPattern} _4y
 * @property {PercentPpmRatioPattern} _5y
 * @property {PercentPpmRatioPattern} _6y
 * @property {PercentPpmRatioPattern} _8y
 * @property {PercentPpmRatioPattern} _10y
 */

/**
 * @typedef {Object} SeriesTree_Market_Returns_Sd24h
 * @property {SeriesTree_Market_Returns_Sd24h_24h} _24h
 * @property {SeriesTree_Market_Returns_Sd24h_1w} _1w
 * @property {SeriesTree_Market_Returns_Sd24h_1m} _1m
 * @property {SeriesTree_Market_Returns_Sd24h_1y} _1y
 */

/**
 * @typedef {Object} SeriesTree_Market_Returns_Sd24h_24h
 * @property {SeriesPattern1<StoredF32>} sma
 * @property {SeriesPattern1<StoredF32>} sd
 */

/**
 * @typedef {Object} SeriesTree_Market_Returns_Sd24h_1w
 * @property {SeriesPattern1<StoredF32>} sma
 * @property {SeriesPattern1<StoredF32>} sd
 */

/**
 * @typedef {Object} SeriesTree_Market_Returns_Sd24h_1m
 * @property {SeriesPattern1<StoredF32>} sma
 * @property {SeriesPattern1<StoredF32>} sd
 */

/**
 * @typedef {Object} SeriesTree_Market_Returns_Sd24h_1y
 * @property {SeriesPattern1<StoredF32>} sma
 * @property {SeriesPattern1<StoredF32>} sd
 */

/**
 * @typedef {Object} SeriesTree_Market_Range
 * @property {_1m1w1y2wPattern} min
 * @property {_1m1w1y2wPattern} max
 * @property {SeriesPattern1<StoredF32>} trueRange
 * @property {SeriesPattern1<StoredF32>} trueRangeSum2w
 * @property {PercentPpmRatioPattern2} choppinessIndex2w
 */

/**
 * @typedef {Object} SeriesTree_Market_MovingAverage
 * @property {SeriesTree_Market_MovingAverage_Sma} sma
 * @property {SeriesTree_Market_MovingAverage_Ema} ema
 */

/**
 * @typedef {Object} SeriesTree_Market_MovingAverage_Sma
 * @property {CentsPpmRatioSatsUsdPattern} _1w
 * @property {CentsPpmRatioSatsUsdPattern} _8d
 * @property {CentsPpmRatioSatsUsdPattern} _13d
 * @property {CentsPpmRatioSatsUsdPattern} _21d
 * @property {CentsPpmRatioSatsUsdPattern} _1m
 * @property {CentsPpmRatioSatsUsdPattern} _34d
 * @property {CentsPpmRatioSatsUsdPattern} _50d
 * @property {CentsPpmRatioSatsUsdPattern} _55d
 * @property {CentsPpmRatioSatsUsdPattern} _89d
 * @property {CentsPpmRatioSatsUsdPattern} _111d
 * @property {CentsPpmRatioSatsUsdPattern} _144d
 * @property {SeriesTree_Market_MovingAverage_Sma_200d} _200d
 * @property {SeriesTree_Market_MovingAverage_Sma_350d} _350d
 * @property {CentsPpmRatioSatsUsdPattern} _1y
 * @property {CentsPpmRatioSatsUsdPattern} _2y
 * @property {CentsPpmRatioSatsUsdPattern} _200w
 * @property {CentsPpmRatioSatsUsdPattern} _4y
 */

/**
 * @typedef {Object} SeriesTree_Market_MovingAverage_Sma_200d
 * @property {SeriesPattern1<Dollars>} usd
 * @property {SeriesPattern1<Cents>} cents
 * @property {SeriesPattern1<SatsFract>} sats
 * @property {SeriesPattern1<PriceRatio>} ppm
 * @property {SeriesPattern1<StoredF32>} ratio
 * @property {CentsSatsUsdPattern} x24
 * @property {CentsSatsUsdPattern} x08
 */

/**
 * @typedef {Object} SeriesTree_Market_MovingAverage_Sma_350d
 * @property {SeriesPattern1<Dollars>} usd
 * @property {SeriesPattern1<Cents>} cents
 * @property {SeriesPattern1<SatsFract>} sats
 * @property {SeriesPattern1<PriceRatio>} ppm
 * @property {SeriesPattern1<StoredF32>} ratio
 * @property {CentsSatsUsdPattern} x2
 */

/**
 * @typedef {Object} SeriesTree_Market_MovingAverage_Ema
 * @property {CentsPpmRatioSatsUsdPattern} _1w
 * @property {CentsPpmRatioSatsUsdPattern} _8d
 * @property {CentsPpmRatioSatsUsdPattern} _12d
 * @property {CentsPpmRatioSatsUsdPattern} _13d
 * @property {CentsPpmRatioSatsUsdPattern} _21d
 * @property {CentsPpmRatioSatsUsdPattern} _26d
 * @property {CentsPpmRatioSatsUsdPattern} _1m
 * @property {CentsPpmRatioSatsUsdPattern} _34d
 * @property {CentsPpmRatioSatsUsdPattern} _55d
 * @property {CentsPpmRatioSatsUsdPattern} _89d
 * @property {CentsPpmRatioSatsUsdPattern} _144d
 * @property {CentsPpmRatioSatsUsdPattern} _200d
 * @property {CentsPpmRatioSatsUsdPattern} _1y
 * @property {CentsPpmRatioSatsUsdPattern} _2y
 * @property {CentsPpmRatioSatsUsdPattern} _200w
 * @property {CentsPpmRatioSatsUsdPattern} _4y
 */

/**
 * @typedef {Object} SeriesTree_Market_Technical
 * @property {SeriesTree_Market_Technical_Rsi} rsi
 * @property {PpmRatioPattern} piCycle
 * @property {SeriesTree_Market_Technical_Macd} macd
 */

/**
 * @typedef {Object} SeriesTree_Market_Technical_Rsi
 * @property {RsiStochPattern} _24h
 * @property {RsiStochPattern} _1w
 * @property {RsiStochPattern} _1m
 */

/**
 * @typedef {Object} SeriesTree_Market_Technical_Macd
 * @property {SeriesTree_Market_Technical_Macd_24h} _24h
 * @property {SeriesTree_Market_Technical_Macd_1w} _1w
 * @property {SeriesTree_Market_Technical_Macd_1m} _1m
 */

/**
 * @typedef {Object} SeriesTree_Market_Technical_Macd_24h
 * @property {SeriesPattern1<StoredF32>} emaFast
 * @property {SeriesPattern1<StoredF32>} emaSlow
 * @property {SeriesPattern1<StoredF32>} line
 * @property {SeriesPattern1<StoredF32>} signal
 * @property {SeriesPattern1<StoredF32>} histogram
 */

/**
 * @typedef {Object} SeriesTree_Market_Technical_Macd_1w
 * @property {SeriesPattern1<StoredF32>} emaFast
 * @property {SeriesPattern1<StoredF32>} emaSlow
 * @property {SeriesPattern1<StoredF32>} line
 * @property {SeriesPattern1<StoredF32>} signal
 * @property {SeriesPattern1<StoredF32>} histogram
 */

/**
 * @typedef {Object} SeriesTree_Market_Technical_Macd_1m
 * @property {SeriesPattern1<StoredF32>} emaFast
 * @property {SeriesPattern1<StoredF32>} emaSlow
 * @property {SeriesPattern1<StoredF32>} line
 * @property {SeriesPattern1<StoredF32>} signal
 * @property {SeriesPattern1<StoredF32>} histogram
 */

/**
 * @typedef {Object} SeriesTree_Pools
 * @property {SeriesPattern18<PoolSlug>} pool
 * @property {SeriesTree_Pools_Major} major
 * @property {SeriesTree_Pools_Minor} minor
 */

/**
 * @typedef {Object} SeriesTree_Pools_Major
 * @property {BlocksDominanceRewardsPattern} unknown
 * @property {BlocksDominanceRewardsPattern} luxor
 * @property {BlocksDominanceRewardsPattern} btccom
 * @property {BlocksDominanceRewardsPattern} btctop
 * @property {BlocksDominanceRewardsPattern} btcguild
 * @property {BlocksDominanceRewardsPattern} eligius
 * @property {BlocksDominanceRewardsPattern} f2pool
 * @property {BlocksDominanceRewardsPattern} braiinspool
 * @property {BlocksDominanceRewardsPattern} antpool
 * @property {BlocksDominanceRewardsPattern} btcc
 * @property {BlocksDominanceRewardsPattern} bwpool
 * @property {BlocksDominanceRewardsPattern} bitfury
 * @property {BlocksDominanceRewardsPattern} viabtc
 * @property {BlocksDominanceRewardsPattern} poolin
 * @property {BlocksDominanceRewardsPattern} spiderpool
 * @property {BlocksDominanceRewardsPattern} binancepool
 * @property {BlocksDominanceRewardsPattern} foundryusa
 * @property {BlocksDominanceRewardsPattern} sbicrypto
 * @property {BlocksDominanceRewardsPattern} marapool
 * @property {BlocksDominanceRewardsPattern} secpool
 * @property {BlocksDominanceRewardsPattern} ocean
 * @property {BlocksDominanceRewardsPattern} whitepool
 */

/**
 * @typedef {Object} SeriesTree_Pools_Minor
 * @property {BlocksDominancePattern} blockfills
 * @property {BlocksDominancePattern} ultimuspool
 * @property {BlocksDominancePattern} terrapool
 * @property {BlocksDominancePattern} onethash
 * @property {BlocksDominancePattern} bitfarms
 * @property {BlocksDominancePattern} huobipool
 * @property {BlocksDominancePattern} wayicn
 * @property {BlocksDominancePattern} canoepool
 * @property {BlocksDominancePattern} bitcoincom
 * @property {BlocksDominancePattern} pool175btc
 * @property {BlocksDominancePattern} gbminers
 * @property {BlocksDominancePattern} axbt
 * @property {BlocksDominancePattern} asicminer
 * @property {BlocksDominancePattern} bitminter
 * @property {BlocksDominancePattern} bitcoinrussia
 * @property {BlocksDominancePattern} btcserv
 * @property {BlocksDominancePattern} simplecoinus
 * @property {BlocksDominancePattern} ozcoin
 * @property {BlocksDominancePattern} eclipsemc
 * @property {BlocksDominancePattern} maxbtc
 * @property {BlocksDominancePattern} triplemining
 * @property {BlocksDominancePattern} coinlab
 * @property {BlocksDominancePattern} pool50btc
 * @property {BlocksDominancePattern} ghashio
 * @property {BlocksDominancePattern} stminingcorp
 * @property {BlocksDominancePattern} bitparking
 * @property {BlocksDominancePattern} mmpool
 * @property {BlocksDominancePattern} polmine
 * @property {BlocksDominancePattern} kncminer
 * @property {BlocksDominancePattern} bitalo
 * @property {BlocksDominancePattern} hhtt
 * @property {BlocksDominancePattern} megabigpower
 * @property {BlocksDominancePattern} mtred
 * @property {BlocksDominancePattern} nmcbit
 * @property {BlocksDominancePattern} yourbtcnet
 * @property {BlocksDominancePattern} givemecoins
 * @property {BlocksDominancePattern} multicoinco
 * @property {BlocksDominancePattern} bcpoolio
 * @property {BlocksDominancePattern} cointerra
 * @property {BlocksDominancePattern} kanopool
 * @property {BlocksDominancePattern} solock
 * @property {BlocksDominancePattern} ckpool
 * @property {BlocksDominancePattern} nicehash
 * @property {BlocksDominancePattern} bitclub
 * @property {BlocksDominancePattern} bitcoinaffiliatenetwork
 * @property {BlocksDominancePattern} exxbw
 * @property {BlocksDominancePattern} bitsolo
 * @property {BlocksDominancePattern} twentyoneinc
 * @property {BlocksDominancePattern} digitalbtc
 * @property {BlocksDominancePattern} eightbaochi
 * @property {BlocksDominancePattern} mybtccoinpool
 * @property {BlocksDominancePattern} tbdice
 * @property {BlocksDominancePattern} hashpool
 * @property {BlocksDominancePattern} nexious
 * @property {BlocksDominancePattern} bravomining
 * @property {BlocksDominancePattern} hotpool
 * @property {BlocksDominancePattern} okexpool
 * @property {BlocksDominancePattern} bcmonster
 * @property {BlocksDominancePattern} onehash
 * @property {BlocksDominancePattern} bixin
 * @property {BlocksDominancePattern} tatmaspool
 * @property {BlocksDominancePattern} connectbtc
 * @property {BlocksDominancePattern} batpool
 * @property {BlocksDominancePattern} waterhole
 * @property {BlocksDominancePattern} dcexploration
 * @property {BlocksDominancePattern} dcex
 * @property {BlocksDominancePattern} btpool
 * @property {BlocksDominancePattern} fiftyeightcoin
 * @property {BlocksDominancePattern} bitcoinindia
 * @property {BlocksDominancePattern} shawnp0wers
 * @property {BlocksDominancePattern} phashio
 * @property {BlocksDominancePattern} rigpool
 * @property {BlocksDominancePattern} haozhuzhu
 * @property {BlocksDominancePattern} sevenpool
 * @property {BlocksDominancePattern} miningkings
 * @property {BlocksDominancePattern} hashbx
 * @property {BlocksDominancePattern} dpool
 * @property {BlocksDominancePattern} rawpool
 * @property {BlocksDominancePattern} haominer
 * @property {BlocksDominancePattern} helix
 * @property {BlocksDominancePattern} bitcoinukraine
 * @property {BlocksDominancePattern} secretsuperstar
 * @property {BlocksDominancePattern} tigerpoolnet
 * @property {BlocksDominancePattern} sigmapoolcom
 * @property {BlocksDominancePattern} okpooltop
 * @property {BlocksDominancePattern} hummerpool
 * @property {BlocksDominancePattern} tangpool
 * @property {BlocksDominancePattern} bytepool
 * @property {BlocksDominancePattern} novablock
 * @property {BlocksDominancePattern} miningcity
 * @property {BlocksDominancePattern} minerium
 * @property {BlocksDominancePattern} lubiancom
 * @property {BlocksDominancePattern} okkong
 * @property {BlocksDominancePattern} aaopool
 * @property {BlocksDominancePattern} emcdpool
 * @property {BlocksDominancePattern} arkpool
 * @property {BlocksDominancePattern} purebtccom
 * @property {BlocksDominancePattern} kucoinpool
 * @property {BlocksDominancePattern} entrustcharitypool
 * @property {BlocksDominancePattern} okminer
 * @property {BlocksDominancePattern} titan
 * @property {BlocksDominancePattern} pegapool
 * @property {BlocksDominancePattern} btcnuggets
 * @property {BlocksDominancePattern} cloudhashing
 * @property {BlocksDominancePattern} digitalxmintsy
 * @property {BlocksDominancePattern} telco214
 * @property {BlocksDominancePattern} btcpoolparty
 * @property {BlocksDominancePattern} multipool
 * @property {BlocksDominancePattern} transactioncoinmining
 * @property {BlocksDominancePattern} btcdig
 * @property {BlocksDominancePattern} trickysbtcpool
 * @property {BlocksDominancePattern} btcmp
 * @property {BlocksDominancePattern} eobot
 * @property {BlocksDominancePattern} unomp
 * @property {BlocksDominancePattern} patels
 * @property {BlocksDominancePattern} gogreenlight
 * @property {BlocksDominancePattern} bitcoinindiapool
 * @property {BlocksDominancePattern} ekanembtc
 * @property {BlocksDominancePattern} canoe
 * @property {BlocksDominancePattern} tiger
 * @property {BlocksDominancePattern} onem1x
 * @property {BlocksDominancePattern} zulupool
 * @property {BlocksDominancePattern} wiz
 * @property {BlocksDominancePattern} wk057
 * @property {BlocksDominancePattern} futurebitapollosolo
 * @property {BlocksDominancePattern} carbonnegative
 * @property {BlocksDominancePattern} portlandhodl
 * @property {BlocksDominancePattern} phoenix
 * @property {BlocksDominancePattern} neopool
 * @property {BlocksDominancePattern} maxipool
 * @property {BlocksDominancePattern} bitfufupool
 * @property {BlocksDominancePattern} gdpool
 * @property {BlocksDominancePattern} miningdutch
 * @property {BlocksDominancePattern} publicpool
 * @property {BlocksDominancePattern} miningsquared
 * @property {BlocksDominancePattern} innopolistech
 * @property {BlocksDominancePattern} btclab
 * @property {BlocksDominancePattern} parasite
 * @property {BlocksDominancePattern} redrockpool
 * @property {BlocksDominancePattern} est3lar
 * @property {BlocksDominancePattern} braiinssolo
 * @property {BlocksDominancePattern} solopool
 * @property {BlocksDominancePattern} noderunners
 * @property {BlocksDominancePattern} dmnd
 */

/**
 * @typedef {Object} SeriesTree_Price
 * @property {SeriesTree_Price_Split} split
 * @property {SeriesTree_Price_Ohlc} ohlc
 * @property {SeriesTree_Price_Spot} spot
 */

/**
 * @typedef {Object} SeriesTree_Price_Split
 * @property {CentsSatsUsdPattern3} open
 * @property {CentsSatsUsdPattern3} high
 * @property {CentsSatsUsdPattern3} low
 * @property {CentsSatsUsdPattern3} close
 */

/**
 * @typedef {Object} SeriesTree_Price_Ohlc
 * @property {SeriesPattern2<OHLCDollars>} usd
 * @property {SeriesPattern2<OHLCCents>} cents
 * @property {SeriesPattern2<OHLCSats>} sats
 */

/**
 * @typedef {Object} SeriesTree_Price_Spot
 * @property {SeriesPattern1<Dollars>} usd
 * @property {SeriesPattern1<Cents>} cents
 * @property {SeriesPattern1<Sats>} sats
 */

/**
 * @typedef {Object} SeriesTree_Cohorts
 * @property {SeriesTree_Cohorts_Supply} supply
 * @property {SeriesTree_Cohorts_Outputs} outputs
 * @property {SeriesTree_Cohorts_Activity} activity
 * @property {SeriesTree_Cohorts_Realized} realized
 * @property {SeriesTree_Cohorts_Unrealized} unrealized
 * @property {SeriesTree_Cohorts_Urpd} urpd
 */

/**
 * @typedef {Object} SeriesTree_Cohorts_Supply
 * @property {SeriesTree_Cohorts_Supply_Total} total
 * @property {SeriesTree_Cohorts_Supply_Matured} matured
 * @property {SeriesTree_Cohorts_Supply_InProfit} inProfit
 * @property {SeriesTree_Cohorts_Supply_InLoss} inLoss
 * @property {SeriesTree_Cohorts_Supply_Delta} delta
 * @property {SeriesTree_Cohorts_Supply_Dominance} dominance
 */

/**
 * @typedef {Object} SeriesTree_Cohorts_Supply_Total
 * @property {SeriesTree_Cohorts_Supply_Total_Age} age
 * @property {SeriesTree_Cohorts_Supply_Total_Epoch} epoch
 * @property {SeriesTree_Cohorts_Supply_Total_Class} class
 * @property {SeriesTree_Cohorts_Supply_Total_UtxoAmount} utxoAmount
 * @property {EmptyP2aP2msP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshUnknownPattern2} type
 */

/**
 * @typedef {Object} SeriesTree_Cohorts_Supply_Total_Age
 * @property {BtcCentsSatsUsdPattern} under1h
 * @property {BtcCentsSatsUsdPattern} _1hTo1d
 * @property {BtcCentsSatsUsdPattern} _1dTo1w
 * @property {BtcCentsSatsUsdPattern} _1wTo1m
 * @property {BtcCentsSatsUsdPattern} _1mTo2m
 * @property {BtcCentsSatsUsdPattern} _2mTo3m
 * @property {BtcCentsSatsUsdPattern} _3mTo4m
 * @property {BtcCentsSatsUsdPattern} _4mTo5m
 * @property {BtcCentsSatsUsdPattern} _5mTo6m
 * @property {BtcCentsSatsUsdPattern} _6mTo9m
 * @property {BtcCentsSatsUsdPattern} _9mTo1y
 * @property {BtcCentsSatsUsdPattern} _1yTo18m
 * @property {BtcCentsSatsUsdPattern} _18mTo2y
 * @property {BtcCentsSatsUsdPattern} _2yTo3y
 * @property {BtcCentsSatsUsdPattern} _3yTo4y
 * @property {BtcCentsSatsUsdPattern} _4yTo5y
 * @property {BtcCentsSatsUsdPattern} _5yTo6y
 * @property {BtcCentsSatsUsdPattern} _6yTo7y
 * @property {BtcCentsSatsUsdPattern} _7yTo8y
 * @property {BtcCentsSatsUsdPattern} _8yTo10y
 * @property {BtcCentsSatsUsdPattern} _10yTo12y
 * @property {BtcCentsSatsUsdPattern} _12yTo15y
 * @property {BtcCentsSatsUsdPattern} over15y
 */

/**
 * @typedef {Object} SeriesTree_Cohorts_Supply_Total_Epoch
 * @property {BtcCentsSatsUsdPattern} _0
 * @property {BtcCentsSatsUsdPattern} _1
 * @property {BtcCentsSatsUsdPattern} _2
 * @property {BtcCentsSatsUsdPattern} _3
 * @property {BtcCentsSatsUsdPattern} _4
 */

/**
 * @typedef {Object} SeriesTree_Cohorts_Supply_Total_Class
 * @property {BtcCentsSatsUsdPattern} _2009
 * @property {BtcCentsSatsUsdPattern} _2010
 * @property {BtcCentsSatsUsdPattern} _2011
 * @property {BtcCentsSatsUsdPattern} _2012
 * @property {BtcCentsSatsUsdPattern} _2013
 * @property {BtcCentsSatsUsdPattern} _2014
 * @property {BtcCentsSatsUsdPattern} _2015
 * @property {BtcCentsSatsUsdPattern} _2016
 * @property {BtcCentsSatsUsdPattern} _2017
 * @property {BtcCentsSatsUsdPattern} _2018
 * @property {BtcCentsSatsUsdPattern} _2019
 * @property {BtcCentsSatsUsdPattern} _2020
 * @property {BtcCentsSatsUsdPattern} _2021
 * @property {BtcCentsSatsUsdPattern} _2022
 * @property {BtcCentsSatsUsdPattern} _2023
 * @property {BtcCentsSatsUsdPattern} _2024
 * @property {BtcCentsSatsUsdPattern} _2025
 * @property {BtcCentsSatsUsdPattern} _2026
 */

/**
 * @typedef {Object} SeriesTree_Cohorts_Supply_Total_UtxoAmount
 * @property {BtcCentsSatsUsdPattern} _0sats
 * @property {BtcCentsSatsUsdPattern} _1satTo10sats
 * @property {BtcCentsSatsUsdPattern} _10satsTo100sats
 * @property {BtcCentsSatsUsdPattern} _100satsTo1kSats
 * @property {BtcCentsSatsUsdPattern} _1kSatsTo10kSats
 * @property {BtcCentsSatsUsdPattern} _10kSatsTo100kSats
 * @property {BtcCentsSatsUsdPattern} _100kSatsTo1mSats
 * @property {BtcCentsSatsUsdPattern} _1mSatsTo10mSats
 * @property {BtcCentsSatsUsdPattern} _10mSatsTo1btc
 * @property {BtcCentsSatsUsdPattern} _1btcTo10btc
 * @property {BtcCentsSatsUsdPattern} _10btcTo100btc
 * @property {BtcCentsSatsUsdPattern} _100btcTo1kBtc
 * @property {BtcCentsSatsUsdPattern} _1kBtcTo10kBtc
 * @property {BtcCentsSatsUsdPattern} _10kBtcTo100kBtc
 * @property {BtcCentsSatsUsdPattern} over100kBtc
 */

/**
 * @typedef {Object} SeriesTree_Cohorts_Supply_Matured
 * @property {AverageBlockCumulativeSumPattern} under1h
 * @property {AverageBlockCumulativeSumPattern} _1hTo1d
 * @property {AverageBlockCumulativeSumPattern} _1dTo1w
 * @property {AverageBlockCumulativeSumPattern} _1wTo1m
 * @property {AverageBlockCumulativeSumPattern} _1mTo2m
 * @property {AverageBlockCumulativeSumPattern} _2mTo3m
 * @property {AverageBlockCumulativeSumPattern} _3mTo4m
 * @property {AverageBlockCumulativeSumPattern} _4mTo5m
 * @property {AverageBlockCumulativeSumPattern} _5mTo6m
 * @property {AverageBlockCumulativeSumPattern} _6mTo9m
 * @property {AverageBlockCumulativeSumPattern} _9mTo1y
 * @property {AverageBlockCumulativeSumPattern} _1yTo18m
 * @property {AverageBlockCumulativeSumPattern} _18mTo2y
 * @property {AverageBlockCumulativeSumPattern} _2yTo3y
 * @property {AverageBlockCumulativeSumPattern} _3yTo4y
 * @property {AverageBlockCumulativeSumPattern} _4yTo5y
 * @property {AverageBlockCumulativeSumPattern} _5yTo6y
 * @property {AverageBlockCumulativeSumPattern} _6yTo7y
 * @property {AverageBlockCumulativeSumPattern} _7yTo8y
 * @property {AverageBlockCumulativeSumPattern} _8yTo10y
 * @property {AverageBlockCumulativeSumPattern} _10yTo12y
 * @property {AverageBlockCumulativeSumPattern} _12yTo15y
 * @property {AverageBlockCumulativeSumPattern} over15y
 */

/**
 * @typedef {Object} SeriesTree_Cohorts_Supply_InProfit
 * @property {SeriesTree_Cohorts_Supply_InProfit_Age} age
 * @property {SeriesTree_Cohorts_Supply_InProfit_Epoch} epoch
 * @property {SeriesTree_Cohorts_Supply_InProfit_Class} class
 */

/**
 * @typedef {Object} SeriesTree_Cohorts_Supply_InProfit_Age
 * @property {BtcCentsSatsUsdPattern} under1h
 * @property {BtcCentsSatsUsdPattern} _1hTo1d
 * @property {BtcCentsSatsUsdPattern} _1dTo1w
 * @property {BtcCentsSatsUsdPattern} _1wTo1m
 * @property {BtcCentsSatsUsdPattern} _1mTo2m
 * @property {BtcCentsSatsUsdPattern} _2mTo3m
 * @property {BtcCentsSatsUsdPattern} _3mTo4m
 * @property {BtcCentsSatsUsdPattern} _4mTo5m
 * @property {BtcCentsSatsUsdPattern} _5mTo6m
 * @property {BtcCentsSatsUsdPattern} _6mTo9m
 * @property {BtcCentsSatsUsdPattern} _9mTo1y
 * @property {BtcCentsSatsUsdPattern} _1yTo18m
 * @property {BtcCentsSatsUsdPattern} _18mTo2y
 * @property {BtcCentsSatsUsdPattern} _2yTo3y
 * @property {BtcCentsSatsUsdPattern} _3yTo4y
 * @property {BtcCentsSatsUsdPattern} _4yTo5y
 * @property {BtcCentsSatsUsdPattern} _5yTo6y
 * @property {BtcCentsSatsUsdPattern} _6yTo7y
 * @property {BtcCentsSatsUsdPattern} _7yTo8y
 * @property {BtcCentsSatsUsdPattern} _8yTo10y
 * @property {BtcCentsSatsUsdPattern} _10yTo12y
 * @property {BtcCentsSatsUsdPattern} _12yTo15y
 * @property {BtcCentsSatsUsdPattern} over15y
 */

/**
 * @typedef {Object} SeriesTree_Cohorts_Supply_InProfit_Epoch
 * @property {BtcCentsSatsUsdPattern} _0
 * @property {BtcCentsSatsUsdPattern} _1
 * @property {BtcCentsSatsUsdPattern} _2
 * @property {BtcCentsSatsUsdPattern} _3
 * @property {BtcCentsSatsUsdPattern} _4
 */

/**
 * @typedef {Object} SeriesTree_Cohorts_Supply_InProfit_Class
 * @property {BtcCentsSatsUsdPattern} _2009
 * @property {BtcCentsSatsUsdPattern} _2010
 * @property {BtcCentsSatsUsdPattern} _2011
 * @property {BtcCentsSatsUsdPattern} _2012
 * @property {BtcCentsSatsUsdPattern} _2013
 * @property {BtcCentsSatsUsdPattern} _2014
 * @property {BtcCentsSatsUsdPattern} _2015
 * @property {BtcCentsSatsUsdPattern} _2016
 * @property {BtcCentsSatsUsdPattern} _2017
 * @property {BtcCentsSatsUsdPattern} _2018
 * @property {BtcCentsSatsUsdPattern} _2019
 * @property {BtcCentsSatsUsdPattern} _2020
 * @property {BtcCentsSatsUsdPattern} _2021
 * @property {BtcCentsSatsUsdPattern} _2022
 * @property {BtcCentsSatsUsdPattern} _2023
 * @property {BtcCentsSatsUsdPattern} _2024
 * @property {BtcCentsSatsUsdPattern} _2025
 * @property {BtcCentsSatsUsdPattern} _2026
 */

/**
 * @typedef {Object} SeriesTree_Cohorts_Supply_InLoss
 * @property {SeriesTree_Cohorts_Supply_InLoss_Age} age
 * @property {SeriesTree_Cohorts_Supply_InLoss_Epoch} epoch
 * @property {SeriesTree_Cohorts_Supply_InLoss_Class} class
 */

/**
 * @typedef {Object} SeriesTree_Cohorts_Supply_InLoss_Age
 * @property {BtcCentsSatsUsdPattern} under1h
 * @property {BtcCentsSatsUsdPattern} _1hTo1d
 * @property {BtcCentsSatsUsdPattern} _1dTo1w
 * @property {BtcCentsSatsUsdPattern} _1wTo1m
 * @property {BtcCentsSatsUsdPattern} _1mTo2m
 * @property {BtcCentsSatsUsdPattern} _2mTo3m
 * @property {BtcCentsSatsUsdPattern} _3mTo4m
 * @property {BtcCentsSatsUsdPattern} _4mTo5m
 * @property {BtcCentsSatsUsdPattern} _5mTo6m
 * @property {BtcCentsSatsUsdPattern} _6mTo9m
 * @property {BtcCentsSatsUsdPattern} _9mTo1y
 * @property {BtcCentsSatsUsdPattern} _1yTo18m
 * @property {BtcCentsSatsUsdPattern} _18mTo2y
 * @property {BtcCentsSatsUsdPattern} _2yTo3y
 * @property {BtcCentsSatsUsdPattern} _3yTo4y
 * @property {BtcCentsSatsUsdPattern} _4yTo5y
 * @property {BtcCentsSatsUsdPattern} _5yTo6y
 * @property {BtcCentsSatsUsdPattern} _6yTo7y
 * @property {BtcCentsSatsUsdPattern} _7yTo8y
 * @property {BtcCentsSatsUsdPattern} _8yTo10y
 * @property {BtcCentsSatsUsdPattern} _10yTo12y
 * @property {BtcCentsSatsUsdPattern} _12yTo15y
 * @property {BtcCentsSatsUsdPattern} over15y
 */

/**
 * @typedef {Object} SeriesTree_Cohorts_Supply_InLoss_Epoch
 * @property {BtcCentsSatsUsdPattern} _0
 * @property {BtcCentsSatsUsdPattern} _1
 * @property {BtcCentsSatsUsdPattern} _2
 * @property {BtcCentsSatsUsdPattern} _3
 * @property {BtcCentsSatsUsdPattern} _4
 */

/**
 * @typedef {Object} SeriesTree_Cohorts_Supply_InLoss_Class
 * @property {BtcCentsSatsUsdPattern} _2009
 * @property {BtcCentsSatsUsdPattern} _2010
 * @property {BtcCentsSatsUsdPattern} _2011
 * @property {BtcCentsSatsUsdPattern} _2012
 * @property {BtcCentsSatsUsdPattern} _2013
 * @property {BtcCentsSatsUsdPattern} _2014
 * @property {BtcCentsSatsUsdPattern} _2015
 * @property {BtcCentsSatsUsdPattern} _2016
 * @property {BtcCentsSatsUsdPattern} _2017
 * @property {BtcCentsSatsUsdPattern} _2018
 * @property {BtcCentsSatsUsdPattern} _2019
 * @property {BtcCentsSatsUsdPattern} _2020
 * @property {BtcCentsSatsUsdPattern} _2021
 * @property {BtcCentsSatsUsdPattern} _2022
 * @property {BtcCentsSatsUsdPattern} _2023
 * @property {BtcCentsSatsUsdPattern} _2024
 * @property {BtcCentsSatsUsdPattern} _2025
 * @property {BtcCentsSatsUsdPattern} _2026
 */

/**
 * @typedef {Object} SeriesTree_Cohorts_Supply_Delta
 * @property {SeriesTree_Cohorts_Supply_Delta_Age} age
 * @property {SeriesTree_Cohorts_Supply_Delta_Epoch} epoch
 * @property {SeriesTree_Cohorts_Supply_Delta_Class} class
 * @property {SeriesTree_Cohorts_Supply_Delta_UtxoAmount} utxoAmount
 * @property {SeriesTree_Cohorts_Supply_Delta_Type} type
 */

/**
 * @typedef {Object} SeriesTree_Cohorts_Supply_Delta_Age
 * @property {AbsoluteRatePattern} under1h
 * @property {AbsoluteRatePattern} _1hTo1d
 * @property {AbsoluteRatePattern} _1dTo1w
 * @property {AbsoluteRatePattern} _1wTo1m
 * @property {AbsoluteRatePattern} _1mTo2m
 * @property {AbsoluteRatePattern} _2mTo3m
 * @property {AbsoluteRatePattern} _3mTo4m
 * @property {AbsoluteRatePattern} _4mTo5m
 * @property {AbsoluteRatePattern} _5mTo6m
 * @property {AbsoluteRatePattern} _6mTo9m
 * @property {AbsoluteRatePattern} _9mTo1y
 * @property {AbsoluteRatePattern} _1yTo18m
 * @property {AbsoluteRatePattern} _18mTo2y
 * @property {AbsoluteRatePattern} _2yTo3y
 * @property {AbsoluteRatePattern} _3yTo4y
 * @property {AbsoluteRatePattern} _4yTo5y
 * @property {AbsoluteRatePattern} _5yTo6y
 * @property {AbsoluteRatePattern} _6yTo7y
 * @property {AbsoluteRatePattern} _7yTo8y
 * @property {AbsoluteRatePattern} _8yTo10y
 * @property {AbsoluteRatePattern} _10yTo12y
 * @property {AbsoluteRatePattern} _12yTo15y
 * @property {AbsoluteRatePattern} over15y
 */

/**
 * @typedef {Object} SeriesTree_Cohorts_Supply_Delta_Epoch
 * @property {AbsoluteRatePattern} _0
 * @property {AbsoluteRatePattern} _1
 * @property {AbsoluteRatePattern} _2
 * @property {AbsoluteRatePattern} _3
 * @property {AbsoluteRatePattern} _4
 */

/**
 * @typedef {Object} SeriesTree_Cohorts_Supply_Delta_Class
 * @property {AbsoluteRatePattern} _2009
 * @property {AbsoluteRatePattern} _2010
 * @property {AbsoluteRatePattern} _2011
 * @property {AbsoluteRatePattern} _2012
 * @property {AbsoluteRatePattern} _2013
 * @property {AbsoluteRatePattern} _2014
 * @property {AbsoluteRatePattern} _2015
 * @property {AbsoluteRatePattern} _2016
 * @property {AbsoluteRatePattern} _2017
 * @property {AbsoluteRatePattern} _2018
 * @property {AbsoluteRatePattern} _2019
 * @property {AbsoluteRatePattern} _2020
 * @property {AbsoluteRatePattern} _2021
 * @property {AbsoluteRatePattern} _2022
 * @property {AbsoluteRatePattern} _2023
 * @property {AbsoluteRatePattern} _2024
 * @property {AbsoluteRatePattern} _2025
 * @property {AbsoluteRatePattern} _2026
 */

/**
 * @typedef {Object} SeriesTree_Cohorts_Supply_Delta_UtxoAmount
 * @property {AbsoluteRatePattern} _0sats
 * @property {AbsoluteRatePattern} _1satTo10sats
 * @property {AbsoluteRatePattern} _10satsTo100sats
 * @property {AbsoluteRatePattern} _100satsTo1kSats
 * @property {AbsoluteRatePattern} _1kSatsTo10kSats
 * @property {AbsoluteRatePattern} _10kSatsTo100kSats
 * @property {AbsoluteRatePattern} _100kSatsTo1mSats
 * @property {AbsoluteRatePattern} _1mSatsTo10mSats
 * @property {AbsoluteRatePattern} _10mSatsTo1btc
 * @property {AbsoluteRatePattern} _1btcTo10btc
 * @property {AbsoluteRatePattern} _10btcTo100btc
 * @property {AbsoluteRatePattern} _100btcTo1kBtc
 * @property {AbsoluteRatePattern} _1kBtcTo10kBtc
 * @property {AbsoluteRatePattern} _10kBtcTo100kBtc
 * @property {AbsoluteRatePattern} over100kBtc
 */

/**
 * @typedef {Object} SeriesTree_Cohorts_Supply_Delta_Type
 * @property {AbsoluteRatePattern} p2pk65
 * @property {AbsoluteRatePattern} p2pk33
 * @property {AbsoluteRatePattern} p2pkh
 * @property {AbsoluteRatePattern} p2ms
 * @property {AbsoluteRatePattern} p2sh
 * @property {AbsoluteRatePattern} p2wpkh
 * @property {AbsoluteRatePattern} p2wsh
 * @property {AbsoluteRatePattern} p2tr
 * @property {AbsoluteRatePattern} p2a
 * @property {AbsoluteRatePattern} unknown
 * @property {AbsoluteRatePattern} empty
 */

/**
 * @typedef {Object} SeriesTree_Cohorts_Supply_Dominance
 * @property {SeriesTree_Cohorts_Supply_Dominance_Age} age
 * @property {SeriesTree_Cohorts_Supply_Dominance_Epoch} epoch
 * @property {SeriesTree_Cohorts_Supply_Dominance_Class} class
 * @property {SeriesTree_Cohorts_Supply_Dominance_UtxoAmount} utxoAmount
 * @property {SeriesTree_Cohorts_Supply_Dominance_Type} type
 */

/**
 * @typedef {Object} SeriesTree_Cohorts_Supply_Dominance_Age
 * @property {PercentPpmRatioPattern2} under1h
 * @property {PercentPpmRatioPattern2} _1hTo1d
 * @property {PercentPpmRatioPattern2} _1dTo1w
 * @property {PercentPpmRatioPattern2} _1wTo1m
 * @property {PercentPpmRatioPattern2} _1mTo2m
 * @property {PercentPpmRatioPattern2} _2mTo3m
 * @property {PercentPpmRatioPattern2} _3mTo4m
 * @property {PercentPpmRatioPattern2} _4mTo5m
 * @property {PercentPpmRatioPattern2} _5mTo6m
 * @property {PercentPpmRatioPattern2} _6mTo9m
 * @property {PercentPpmRatioPattern2} _9mTo1y
 * @property {PercentPpmRatioPattern2} _1yTo18m
 * @property {PercentPpmRatioPattern2} _18mTo2y
 * @property {PercentPpmRatioPattern2} _2yTo3y
 * @property {PercentPpmRatioPattern2} _3yTo4y
 * @property {PercentPpmRatioPattern2} _4yTo5y
 * @property {PercentPpmRatioPattern2} _5yTo6y
 * @property {PercentPpmRatioPattern2} _6yTo7y
 * @property {PercentPpmRatioPattern2} _7yTo8y
 * @property {PercentPpmRatioPattern2} _8yTo10y
 * @property {PercentPpmRatioPattern2} _10yTo12y
 * @property {PercentPpmRatioPattern2} _12yTo15y
 * @property {PercentPpmRatioPattern2} over15y
 */

/**
 * @typedef {Object} SeriesTree_Cohorts_Supply_Dominance_Epoch
 * @property {PercentPpmRatioPattern2} _0
 * @property {PercentPpmRatioPattern2} _1
 * @property {PercentPpmRatioPattern2} _2
 * @property {PercentPpmRatioPattern2} _3
 * @property {PercentPpmRatioPattern2} _4
 */

/**
 * @typedef {Object} SeriesTree_Cohorts_Supply_Dominance_Class
 * @property {PercentPpmRatioPattern2} _2009
 * @property {PercentPpmRatioPattern2} _2010
 * @property {PercentPpmRatioPattern2} _2011
 * @property {PercentPpmRatioPattern2} _2012
 * @property {PercentPpmRatioPattern2} _2013
 * @property {PercentPpmRatioPattern2} _2014
 * @property {PercentPpmRatioPattern2} _2015
 * @property {PercentPpmRatioPattern2} _2016
 * @property {PercentPpmRatioPattern2} _2017
 * @property {PercentPpmRatioPattern2} _2018
 * @property {PercentPpmRatioPattern2} _2019
 * @property {PercentPpmRatioPattern2} _2020
 * @property {PercentPpmRatioPattern2} _2021
 * @property {PercentPpmRatioPattern2} _2022
 * @property {PercentPpmRatioPattern2} _2023
 * @property {PercentPpmRatioPattern2} _2024
 * @property {PercentPpmRatioPattern2} _2025
 * @property {PercentPpmRatioPattern2} _2026
 */

/**
 * @typedef {Object} SeriesTree_Cohorts_Supply_Dominance_UtxoAmount
 * @property {PercentPpmRatioPattern2} _0sats
 * @property {PercentPpmRatioPattern2} _1satTo10sats
 * @property {PercentPpmRatioPattern2} _10satsTo100sats
 * @property {PercentPpmRatioPattern2} _100satsTo1kSats
 * @property {PercentPpmRatioPattern2} _1kSatsTo10kSats
 * @property {PercentPpmRatioPattern2} _10kSatsTo100kSats
 * @property {PercentPpmRatioPattern2} _100kSatsTo1mSats
 * @property {PercentPpmRatioPattern2} _1mSatsTo10mSats
 * @property {PercentPpmRatioPattern2} _10mSatsTo1btc
 * @property {PercentPpmRatioPattern2} _1btcTo10btc
 * @property {PercentPpmRatioPattern2} _10btcTo100btc
 * @property {PercentPpmRatioPattern2} _100btcTo1kBtc
 * @property {PercentPpmRatioPattern2} _1kBtcTo10kBtc
 * @property {PercentPpmRatioPattern2} _10kBtcTo100kBtc
 * @property {PercentPpmRatioPattern2} over100kBtc
 */

/**
 * @typedef {Object} SeriesTree_Cohorts_Supply_Dominance_Type
 * @property {PercentPpmRatioPattern2} p2pk65
 * @property {PercentPpmRatioPattern2} p2pk33
 * @property {PercentPpmRatioPattern2} p2pkh
 * @property {PercentPpmRatioPattern2} p2ms
 * @property {PercentPpmRatioPattern2} p2sh
 * @property {PercentPpmRatioPattern2} p2wpkh
 * @property {PercentPpmRatioPattern2} p2wsh
 * @property {PercentPpmRatioPattern2} p2tr
 * @property {PercentPpmRatioPattern2} p2a
 * @property {PercentPpmRatioPattern2} unknown
 * @property {PercentPpmRatioPattern2} empty
 */

/**
 * @typedef {Object} SeriesTree_Cohorts_Outputs
 * @property {SeriesTree_Cohorts_Outputs_UnspentCount} unspentCount
 * @property {SeriesTree_Cohorts_Outputs_SpentCount} spentCount
 * @property {SeriesTree_Cohorts_Outputs_AvgAmount} avgAmount
 */

/**
 * @typedef {Object} SeriesTree_Cohorts_Outputs_UnspentCount
 * @property {SeriesTree_Cohorts_Outputs_UnspentCount_Age} age
 * @property {SeriesTree_Cohorts_Outputs_UnspentCount_Epoch} epoch
 * @property {SeriesTree_Cohorts_Outputs_UnspentCount_Class} class
 * @property {SeriesTree_Cohorts_Outputs_UnspentCount_UtxoAmount} utxoAmount
 * @property {SeriesTree_Cohorts_Outputs_UnspentCount_Type} type
 */

/**
 * @typedef {Object} SeriesTree_Cohorts_Outputs_UnspentCount_Age
 * @property {BaseDeltaPattern} under1h
 * @property {BaseDeltaPattern} _1hTo1d
 * @property {BaseDeltaPattern} _1dTo1w
 * @property {BaseDeltaPattern} _1wTo1m
 * @property {BaseDeltaPattern} _1mTo2m
 * @property {BaseDeltaPattern} _2mTo3m
 * @property {BaseDeltaPattern} _3mTo4m
 * @property {BaseDeltaPattern} _4mTo5m
 * @property {BaseDeltaPattern} _5mTo6m
 * @property {BaseDeltaPattern} _6mTo9m
 * @property {BaseDeltaPattern} _9mTo1y
 * @property {BaseDeltaPattern} _1yTo18m
 * @property {BaseDeltaPattern} _18mTo2y
 * @property {BaseDeltaPattern} _2yTo3y
 * @property {BaseDeltaPattern} _3yTo4y
 * @property {BaseDeltaPattern} _4yTo5y
 * @property {BaseDeltaPattern} _5yTo6y
 * @property {BaseDeltaPattern} _6yTo7y
 * @property {BaseDeltaPattern} _7yTo8y
 * @property {BaseDeltaPattern} _8yTo10y
 * @property {BaseDeltaPattern} _10yTo12y
 * @property {BaseDeltaPattern} _12yTo15y
 * @property {BaseDeltaPattern} over15y
 */

/**
 * @typedef {Object} SeriesTree_Cohorts_Outputs_UnspentCount_Epoch
 * @property {BaseDeltaPattern} _0
 * @property {BaseDeltaPattern} _1
 * @property {BaseDeltaPattern} _2
 * @property {BaseDeltaPattern} _3
 * @property {BaseDeltaPattern} _4
 */

/**
 * @typedef {Object} SeriesTree_Cohorts_Outputs_UnspentCount_Class
 * @property {BaseDeltaPattern} _2009
 * @property {BaseDeltaPattern} _2010
 * @property {BaseDeltaPattern} _2011
 * @property {BaseDeltaPattern} _2012
 * @property {BaseDeltaPattern} _2013
 * @property {BaseDeltaPattern} _2014
 * @property {BaseDeltaPattern} _2015
 * @property {BaseDeltaPattern} _2016
 * @property {BaseDeltaPattern} _2017
 * @property {BaseDeltaPattern} _2018
 * @property {BaseDeltaPattern} _2019
 * @property {BaseDeltaPattern} _2020
 * @property {BaseDeltaPattern} _2021
 * @property {BaseDeltaPattern} _2022
 * @property {BaseDeltaPattern} _2023
 * @property {BaseDeltaPattern} _2024
 * @property {BaseDeltaPattern} _2025
 * @property {BaseDeltaPattern} _2026
 */

/**
 * @typedef {Object} SeriesTree_Cohorts_Outputs_UnspentCount_UtxoAmount
 * @property {BaseDeltaPattern} _0sats
 * @property {BaseDeltaPattern} _1satTo10sats
 * @property {BaseDeltaPattern} _10satsTo100sats
 * @property {BaseDeltaPattern} _100satsTo1kSats
 * @property {BaseDeltaPattern} _1kSatsTo10kSats
 * @property {BaseDeltaPattern} _10kSatsTo100kSats
 * @property {BaseDeltaPattern} _100kSatsTo1mSats
 * @property {BaseDeltaPattern} _1mSatsTo10mSats
 * @property {BaseDeltaPattern} _10mSatsTo1btc
 * @property {BaseDeltaPattern} _1btcTo10btc
 * @property {BaseDeltaPattern} _10btcTo100btc
 * @property {BaseDeltaPattern} _100btcTo1kBtc
 * @property {BaseDeltaPattern} _1kBtcTo10kBtc
 * @property {BaseDeltaPattern} _10kBtcTo100kBtc
 * @property {BaseDeltaPattern} over100kBtc
 */

/**
 * @typedef {Object} SeriesTree_Cohorts_Outputs_UnspentCount_Type
 * @property {BaseDeltaPattern} p2pk65
 * @property {BaseDeltaPattern} p2pk33
 * @property {BaseDeltaPattern} p2pkh
 * @property {BaseDeltaPattern} p2ms
 * @property {BaseDeltaPattern} p2sh
 * @property {BaseDeltaPattern} p2wpkh
 * @property {BaseDeltaPattern} p2wsh
 * @property {BaseDeltaPattern} p2tr
 * @property {BaseDeltaPattern} p2a
 * @property {BaseDeltaPattern} unknown
 * @property {BaseDeltaPattern} empty
 */

/**
 * @typedef {Object} SeriesTree_Cohorts_Outputs_SpentCount
 * @property {SeriesTree_Cohorts_Outputs_SpentCount_Age} age
 * @property {SeriesTree_Cohorts_Outputs_SpentCount_Epoch} epoch
 * @property {SeriesTree_Cohorts_Outputs_SpentCount_Class} class
 * @property {SeriesTree_Cohorts_Outputs_SpentCount_UtxoAmount} utxoAmount
 * @property {SeriesTree_Cohorts_Outputs_SpentCount_Type} type
 */

/**
 * @typedef {Object} SeriesTree_Cohorts_Outputs_SpentCount_Age
 * @property {AverageBlockCumulativeSumPattern2<StoredU64>} under1h
 * @property {AverageBlockCumulativeSumPattern2<StoredU64>} _1hTo1d
 * @property {AverageBlockCumulativeSumPattern2<StoredU64>} _1dTo1w
 * @property {AverageBlockCumulativeSumPattern2<StoredU64>} _1wTo1m
 * @property {AverageBlockCumulativeSumPattern2<StoredU64>} _1mTo2m
 * @property {AverageBlockCumulativeSumPattern2<StoredU64>} _2mTo3m
 * @property {AverageBlockCumulativeSumPattern2<StoredU64>} _3mTo4m
 * @property {AverageBlockCumulativeSumPattern2<StoredU64>} _4mTo5m
 * @property {AverageBlockCumulativeSumPattern2<StoredU64>} _5mTo6m
 * @property {AverageBlockCumulativeSumPattern2<StoredU64>} _6mTo9m
 * @property {AverageBlockCumulativeSumPattern2<StoredU64>} _9mTo1y
 * @property {AverageBlockCumulativeSumPattern2<StoredU64>} _1yTo18m
 * @property {AverageBlockCumulativeSumPattern2<StoredU64>} _18mTo2y
 * @property {AverageBlockCumulativeSumPattern2<StoredU64>} _2yTo3y
 * @property {AverageBlockCumulativeSumPattern2<StoredU64>} _3yTo4y
 * @property {AverageBlockCumulativeSumPattern2<StoredU64>} _4yTo5y
 * @property {AverageBlockCumulativeSumPattern2<StoredU64>} _5yTo6y
 * @property {AverageBlockCumulativeSumPattern2<StoredU64>} _6yTo7y
 * @property {AverageBlockCumulativeSumPattern2<StoredU64>} _7yTo8y
 * @property {AverageBlockCumulativeSumPattern2<StoredU64>} _8yTo10y
 * @property {AverageBlockCumulativeSumPattern2<StoredU64>} _10yTo12y
 * @property {AverageBlockCumulativeSumPattern2<StoredU64>} _12yTo15y
 * @property {AverageBlockCumulativeSumPattern2<StoredU64>} over15y
 */

/**
 * @typedef {Object} SeriesTree_Cohorts_Outputs_SpentCount_Epoch
 * @property {AverageBlockCumulativeSumPattern2<StoredU64>} _0
 * @property {AverageBlockCumulativeSumPattern2<StoredU64>} _1
 * @property {AverageBlockCumulativeSumPattern2<StoredU64>} _2
 * @property {AverageBlockCumulativeSumPattern2<StoredU64>} _3
 * @property {AverageBlockCumulativeSumPattern2<StoredU64>} _4
 */

/**
 * @typedef {Object} SeriesTree_Cohorts_Outputs_SpentCount_Class
 * @property {AverageBlockCumulativeSumPattern2<StoredU64>} _2009
 * @property {AverageBlockCumulativeSumPattern2<StoredU64>} _2010
 * @property {AverageBlockCumulativeSumPattern2<StoredU64>} _2011
 * @property {AverageBlockCumulativeSumPattern2<StoredU64>} _2012
 * @property {AverageBlockCumulativeSumPattern2<StoredU64>} _2013
 * @property {AverageBlockCumulativeSumPattern2<StoredU64>} _2014
 * @property {AverageBlockCumulativeSumPattern2<StoredU64>} _2015
 * @property {AverageBlockCumulativeSumPattern2<StoredU64>} _2016
 * @property {AverageBlockCumulativeSumPattern2<StoredU64>} _2017
 * @property {AverageBlockCumulativeSumPattern2<StoredU64>} _2018
 * @property {AverageBlockCumulativeSumPattern2<StoredU64>} _2019
 * @property {AverageBlockCumulativeSumPattern2<StoredU64>} _2020
 * @property {AverageBlockCumulativeSumPattern2<StoredU64>} _2021
 * @property {AverageBlockCumulativeSumPattern2<StoredU64>} _2022
 * @property {AverageBlockCumulativeSumPattern2<StoredU64>} _2023
 * @property {AverageBlockCumulativeSumPattern2<StoredU64>} _2024
 * @property {AverageBlockCumulativeSumPattern2<StoredU64>} _2025
 * @property {AverageBlockCumulativeSumPattern2<StoredU64>} _2026
 */

/**
 * @typedef {Object} SeriesTree_Cohorts_Outputs_SpentCount_UtxoAmount
 * @property {AverageBlockCumulativeSumPattern2<StoredU64>} _0sats
 * @property {AverageBlockCumulativeSumPattern2<StoredU64>} _1satTo10sats
 * @property {AverageBlockCumulativeSumPattern2<StoredU64>} _10satsTo100sats
 * @property {AverageBlockCumulativeSumPattern2<StoredU64>} _100satsTo1kSats
 * @property {AverageBlockCumulativeSumPattern2<StoredU64>} _1kSatsTo10kSats
 * @property {AverageBlockCumulativeSumPattern2<StoredU64>} _10kSatsTo100kSats
 * @property {AverageBlockCumulativeSumPattern2<StoredU64>} _100kSatsTo1mSats
 * @property {AverageBlockCumulativeSumPattern2<StoredU64>} _1mSatsTo10mSats
 * @property {AverageBlockCumulativeSumPattern2<StoredU64>} _10mSatsTo1btc
 * @property {AverageBlockCumulativeSumPattern2<StoredU64>} _1btcTo10btc
 * @property {AverageBlockCumulativeSumPattern2<StoredU64>} _10btcTo100btc
 * @property {AverageBlockCumulativeSumPattern2<StoredU64>} _100btcTo1kBtc
 * @property {AverageBlockCumulativeSumPattern2<StoredU64>} _1kBtcTo10kBtc
 * @property {AverageBlockCumulativeSumPattern2<StoredU64>} _10kBtcTo100kBtc
 * @property {AverageBlockCumulativeSumPattern2<StoredU64>} over100kBtc
 */

/**
 * @typedef {Object} SeriesTree_Cohorts_Outputs_SpentCount_Type
 * @property {AverageBlockCumulativeSumPattern2<StoredU64>} p2pk65
 * @property {AverageBlockCumulativeSumPattern2<StoredU64>} p2pk33
 * @property {AverageBlockCumulativeSumPattern2<StoredU64>} p2pkh
 * @property {AverageBlockCumulativeSumPattern2<StoredU64>} p2ms
 * @property {AverageBlockCumulativeSumPattern2<StoredU64>} p2sh
 * @property {AverageBlockCumulativeSumPattern2<StoredU64>} p2wpkh
 * @property {AverageBlockCumulativeSumPattern2<StoredU64>} p2wsh
 * @property {AverageBlockCumulativeSumPattern2<StoredU64>} p2tr
 * @property {AverageBlockCumulativeSumPattern2<StoredU64>} p2a
 * @property {AverageBlockCumulativeSumPattern2<StoredU64>} unknown
 * @property {AverageBlockCumulativeSumPattern2<StoredU64>} empty
 */

/**
 * @typedef {Object} SeriesTree_Cohorts_Outputs_AvgAmount
 * @property {BtcCentsSatsUsdPattern} all
 * @property {EmptyP2aP2msP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshUnknownPattern2} byType
 */

/**
 * @typedef {Object} SeriesTree_Cohorts_Activity
 * @property {SeriesTree_Cohorts_Activity_TransferVolume} transferVolume
 * @property {SeriesTree_Cohorts_Activity_CoindaysDestroyed} coindaysDestroyed
 */

/**
 * @typedef {Object} SeriesTree_Cohorts_Activity_TransferVolume
 * @property {SeriesTree_Cohorts_Activity_TransferVolume_Age} age
 * @property {SeriesTree_Cohorts_Activity_TransferVolume_Epoch} epoch
 * @property {SeriesTree_Cohorts_Activity_TransferVolume_Class} class
 * @property {SeriesTree_Cohorts_Activity_TransferVolume_InProfit} inProfit
 * @property {SeriesTree_Cohorts_Activity_TransferVolume_InLoss} inLoss
 * @property {_0sats100btc100k100sats10btc10k10m10sats1btc1k1m1satOverPattern4} utxoAmount
 * @property {SeriesTree_Cohorts_Activity_TransferVolume_Type} type
 */

/**
 * @typedef {Object} SeriesTree_Cohorts_Activity_TransferVolume_Age
 * @property {AverageBlockCumulativeSumPattern} under1h
 * @property {AverageBlockCumulativeSumPattern} _1hTo1d
 * @property {AverageBlockCumulativeSumPattern} _1dTo1w
 * @property {AverageBlockCumulativeSumPattern} _1wTo1m
 * @property {AverageBlockCumulativeSumPattern} _1mTo2m
 * @property {AverageBlockCumulativeSumPattern} _2mTo3m
 * @property {AverageBlockCumulativeSumPattern} _3mTo4m
 * @property {AverageBlockCumulativeSumPattern} _4mTo5m
 * @property {AverageBlockCumulativeSumPattern} _5mTo6m
 * @property {AverageBlockCumulativeSumPattern} _6mTo9m
 * @property {AverageBlockCumulativeSumPattern} _9mTo1y
 * @property {AverageBlockCumulativeSumPattern} _1yTo18m
 * @property {AverageBlockCumulativeSumPattern} _18mTo2y
 * @property {AverageBlockCumulativeSumPattern} _2yTo3y
 * @property {AverageBlockCumulativeSumPattern} _3yTo4y
 * @property {AverageBlockCumulativeSumPattern} _4yTo5y
 * @property {AverageBlockCumulativeSumPattern} _5yTo6y
 * @property {AverageBlockCumulativeSumPattern} _6yTo7y
 * @property {AverageBlockCumulativeSumPattern} _7yTo8y
 * @property {AverageBlockCumulativeSumPattern} _8yTo10y
 * @property {AverageBlockCumulativeSumPattern} _10yTo12y
 * @property {AverageBlockCumulativeSumPattern} _12yTo15y
 * @property {AverageBlockCumulativeSumPattern} over15y
 */

/**
 * @typedef {Object} SeriesTree_Cohorts_Activity_TransferVolume_Epoch
 * @property {AverageBlockCumulativeSumPattern} _0
 * @property {AverageBlockCumulativeSumPattern} _1
 * @property {AverageBlockCumulativeSumPattern} _2
 * @property {AverageBlockCumulativeSumPattern} _3
 * @property {AverageBlockCumulativeSumPattern} _4
 */

/**
 * @typedef {Object} SeriesTree_Cohorts_Activity_TransferVolume_Class
 * @property {AverageBlockCumulativeSumPattern} _2009
 * @property {AverageBlockCumulativeSumPattern} _2010
 * @property {AverageBlockCumulativeSumPattern} _2011
 * @property {AverageBlockCumulativeSumPattern} _2012
 * @property {AverageBlockCumulativeSumPattern} _2013
 * @property {AverageBlockCumulativeSumPattern} _2014
 * @property {AverageBlockCumulativeSumPattern} _2015
 * @property {AverageBlockCumulativeSumPattern} _2016
 * @property {AverageBlockCumulativeSumPattern} _2017
 * @property {AverageBlockCumulativeSumPattern} _2018
 * @property {AverageBlockCumulativeSumPattern} _2019
 * @property {AverageBlockCumulativeSumPattern} _2020
 * @property {AverageBlockCumulativeSumPattern} _2021
 * @property {AverageBlockCumulativeSumPattern} _2022
 * @property {AverageBlockCumulativeSumPattern} _2023
 * @property {AverageBlockCumulativeSumPattern} _2024
 * @property {AverageBlockCumulativeSumPattern} _2025
 * @property {AverageBlockCumulativeSumPattern} _2026
 */

/**
 * @typedef {Object} SeriesTree_Cohorts_Activity_TransferVolume_InProfit
 * @property {SeriesTree_Cohorts_Activity_TransferVolume_InProfit_Age} age
 * @property {_01234Pattern2} epoch
 * @property {_200920102011201220132014201520162017201820192020202120222023202420252026Pattern2} class
 */

/**
 * @typedef {Object} SeriesTree_Cohorts_Activity_TransferVolume_InProfit_Age
 * @property {AverageBlockCumulativeSumPattern} under1h
 * @property {AverageBlockCumulativeSumPattern} _1hTo1d
 * @property {AverageBlockCumulativeSumPattern} _1dTo1w
 * @property {AverageBlockCumulativeSumPattern} _1wTo1m
 * @property {AverageBlockCumulativeSumPattern} _1mTo2m
 * @property {AverageBlockCumulativeSumPattern} _2mTo3m
 * @property {AverageBlockCumulativeSumPattern} _3mTo4m
 * @property {AverageBlockCumulativeSumPattern} _4mTo5m
 * @property {AverageBlockCumulativeSumPattern} _5mTo6m
 * @property {AverageBlockCumulativeSumPattern} _6mTo9m
 * @property {AverageBlockCumulativeSumPattern} _9mTo1y
 * @property {AverageBlockCumulativeSumPattern} _1yTo18m
 * @property {AverageBlockCumulativeSumPattern} _18mTo2y
 * @property {AverageBlockCumulativeSumPattern} _2yTo3y
 * @property {AverageBlockCumulativeSumPattern} _3yTo4y
 * @property {AverageBlockCumulativeSumPattern} _4yTo5y
 * @property {AverageBlockCumulativeSumPattern} _5yTo6y
 * @property {AverageBlockCumulativeSumPattern} _6yTo7y
 * @property {AverageBlockCumulativeSumPattern} _7yTo8y
 * @property {AverageBlockCumulativeSumPattern} _8yTo10y
 * @property {AverageBlockCumulativeSumPattern} _10yTo12y
 * @property {AverageBlockCumulativeSumPattern} _12yTo15y
 * @property {AverageBlockCumulativeSumPattern} over15y
 */

/**
 * @typedef {Object} SeriesTree_Cohorts_Activity_TransferVolume_InLoss
 * @property {SeriesTree_Cohorts_Activity_TransferVolume_InLoss_Age} age
 * @property {_01234Pattern2} epoch
 * @property {_200920102011201220132014201520162017201820192020202120222023202420252026Pattern2} class
 */

/**
 * @typedef {Object} SeriesTree_Cohorts_Activity_TransferVolume_InLoss_Age
 * @property {AverageBlockCumulativeSumPattern} under1h
 * @property {AverageBlockCumulativeSumPattern} _1hTo1d
 * @property {AverageBlockCumulativeSumPattern} _1dTo1w
 * @property {AverageBlockCumulativeSumPattern} _1wTo1m
 * @property {AverageBlockCumulativeSumPattern} _1mTo2m
 * @property {AverageBlockCumulativeSumPattern} _2mTo3m
 * @property {AverageBlockCumulativeSumPattern} _3mTo4m
 * @property {AverageBlockCumulativeSumPattern} _4mTo5m
 * @property {AverageBlockCumulativeSumPattern} _5mTo6m
 * @property {AverageBlockCumulativeSumPattern} _6mTo9m
 * @property {AverageBlockCumulativeSumPattern} _9mTo1y
 * @property {AverageBlockCumulativeSumPattern} _1yTo18m
 * @property {AverageBlockCumulativeSumPattern} _18mTo2y
 * @property {AverageBlockCumulativeSumPattern} _2yTo3y
 * @property {AverageBlockCumulativeSumPattern} _3yTo4y
 * @property {AverageBlockCumulativeSumPattern} _4yTo5y
 * @property {AverageBlockCumulativeSumPattern} _5yTo6y
 * @property {AverageBlockCumulativeSumPattern} _6yTo7y
 * @property {AverageBlockCumulativeSumPattern} _7yTo8y
 * @property {AverageBlockCumulativeSumPattern} _8yTo10y
 * @property {AverageBlockCumulativeSumPattern} _10yTo12y
 * @property {AverageBlockCumulativeSumPattern} _12yTo15y
 * @property {AverageBlockCumulativeSumPattern} over15y
 */

/**
 * @typedef {Object} SeriesTree_Cohorts_Activity_TransferVolume_Type
 * @property {AverageBlockCumulativeSumPattern} p2pk65
 * @property {AverageBlockCumulativeSumPattern} p2pk33
 * @property {AverageBlockCumulativeSumPattern} p2pkh
 * @property {AverageBlockCumulativeSumPattern} p2ms
 * @property {AverageBlockCumulativeSumPattern} p2sh
 * @property {AverageBlockCumulativeSumPattern} p2wpkh
 * @property {AverageBlockCumulativeSumPattern} p2wsh
 * @property {AverageBlockCumulativeSumPattern} p2tr
 * @property {AverageBlockCumulativeSumPattern} p2a
 * @property {AverageBlockCumulativeSumPattern} unknown
 * @property {AverageBlockCumulativeSumPattern} empty
 */

/**
 * @typedef {Object} SeriesTree_Cohorts_Activity_CoindaysDestroyed
 * @property {SeriesTree_Cohorts_Activity_CoindaysDestroyed_Age} age
 * @property {SeriesTree_Cohorts_Activity_CoindaysDestroyed_Epoch} epoch
 * @property {SeriesTree_Cohorts_Activity_CoindaysDestroyed_Class} class
 */

/**
 * @typedef {Object} SeriesTree_Cohorts_Activity_CoindaysDestroyed_Age
 * @property {AverageBlockCumulativeSumPattern2<StoredF64>} under1h
 * @property {AverageBlockCumulativeSumPattern2<StoredF64>} _1hTo1d
 * @property {AverageBlockCumulativeSumPattern2<StoredF64>} _1dTo1w
 * @property {AverageBlockCumulativeSumPattern2<StoredF64>} _1wTo1m
 * @property {AverageBlockCumulativeSumPattern2<StoredF64>} _1mTo2m
 * @property {AverageBlockCumulativeSumPattern2<StoredF64>} _2mTo3m
 * @property {AverageBlockCumulativeSumPattern2<StoredF64>} _3mTo4m
 * @property {AverageBlockCumulativeSumPattern2<StoredF64>} _4mTo5m
 * @property {AverageBlockCumulativeSumPattern2<StoredF64>} _5mTo6m
 * @property {AverageBlockCumulativeSumPattern2<StoredF64>} _6mTo9m
 * @property {AverageBlockCumulativeSumPattern2<StoredF64>} _9mTo1y
 * @property {AverageBlockCumulativeSumPattern2<StoredF64>} _1yTo18m
 * @property {AverageBlockCumulativeSumPattern2<StoredF64>} _18mTo2y
 * @property {AverageBlockCumulativeSumPattern2<StoredF64>} _2yTo3y
 * @property {AverageBlockCumulativeSumPattern2<StoredF64>} _3yTo4y
 * @property {AverageBlockCumulativeSumPattern2<StoredF64>} _4yTo5y
 * @property {AverageBlockCumulativeSumPattern2<StoredF64>} _5yTo6y
 * @property {AverageBlockCumulativeSumPattern2<StoredF64>} _6yTo7y
 * @property {AverageBlockCumulativeSumPattern2<StoredF64>} _7yTo8y
 * @property {AverageBlockCumulativeSumPattern2<StoredF64>} _8yTo10y
 * @property {AverageBlockCumulativeSumPattern2<StoredF64>} _10yTo12y
 * @property {AverageBlockCumulativeSumPattern2<StoredF64>} _12yTo15y
 * @property {AverageBlockCumulativeSumPattern2<StoredF64>} over15y
 */

/**
 * @typedef {Object} SeriesTree_Cohorts_Activity_CoindaysDestroyed_Epoch
 * @property {AverageBlockCumulativeSumPattern2<StoredF64>} _0
 * @property {AverageBlockCumulativeSumPattern2<StoredF64>} _1
 * @property {AverageBlockCumulativeSumPattern2<StoredF64>} _2
 * @property {AverageBlockCumulativeSumPattern2<StoredF64>} _3
 * @property {AverageBlockCumulativeSumPattern2<StoredF64>} _4
 */

/**
 * @typedef {Object} SeriesTree_Cohorts_Activity_CoindaysDestroyed_Class
 * @property {AverageBlockCumulativeSumPattern2<StoredF64>} _2009
 * @property {AverageBlockCumulativeSumPattern2<StoredF64>} _2010
 * @property {AverageBlockCumulativeSumPattern2<StoredF64>} _2011
 * @property {AverageBlockCumulativeSumPattern2<StoredF64>} _2012
 * @property {AverageBlockCumulativeSumPattern2<StoredF64>} _2013
 * @property {AverageBlockCumulativeSumPattern2<StoredF64>} _2014
 * @property {AverageBlockCumulativeSumPattern2<StoredF64>} _2015
 * @property {AverageBlockCumulativeSumPattern2<StoredF64>} _2016
 * @property {AverageBlockCumulativeSumPattern2<StoredF64>} _2017
 * @property {AverageBlockCumulativeSumPattern2<StoredF64>} _2018
 * @property {AverageBlockCumulativeSumPattern2<StoredF64>} _2019
 * @property {AverageBlockCumulativeSumPattern2<StoredF64>} _2020
 * @property {AverageBlockCumulativeSumPattern2<StoredF64>} _2021
 * @property {AverageBlockCumulativeSumPattern2<StoredF64>} _2022
 * @property {AverageBlockCumulativeSumPattern2<StoredF64>} _2023
 * @property {AverageBlockCumulativeSumPattern2<StoredF64>} _2024
 * @property {AverageBlockCumulativeSumPattern2<StoredF64>} _2025
 * @property {AverageBlockCumulativeSumPattern2<StoredF64>} _2026
 */

/**
 * @typedef {Object} SeriesTree_Cohorts_Realized
 * @property {SeriesTree_Cohorts_Realized_Cap} cap
 * @property {SeriesTree_Cohorts_Realized_Profit} profit
 * @property {SeriesTree_Cohorts_Realized_Loss} loss
 * @property {SeriesTree_Cohorts_Realized_NetPnl} netPnl
 * @property {SeriesTree_Cohorts_Realized_ValueDestroyed} valueDestroyed
 * @property {SeriesTree_Cohorts_Realized_Price} price
 */

/**
 * @typedef {Object} SeriesTree_Cohorts_Realized_Cap
 * @property {SeriesTree_Cohorts_Realized_Cap_Age} age
 * @property {SeriesTree_Cohorts_Realized_Cap_Epoch} epoch
 * @property {SeriesTree_Cohorts_Realized_Cap_Class} class
 * @property {_0sats100btc100k100sats10btc10k10m10sats1btc1k1m1satOverPattern} utxoAmount
 * @property {SeriesTree_Cohorts_Realized_Cap_Type} type
 */

/**
 * @typedef {Object} SeriesTree_Cohorts_Realized_Cap_Age
 * @property {CentsUsdPattern} under1h
 * @property {CentsUsdPattern} _1hTo1d
 * @property {CentsUsdPattern} _1dTo1w
 * @property {CentsUsdPattern} _1wTo1m
 * @property {CentsUsdPattern} _1mTo2m
 * @property {CentsUsdPattern} _2mTo3m
 * @property {CentsUsdPattern} _3mTo4m
 * @property {CentsUsdPattern} _4mTo5m
 * @property {CentsUsdPattern} _5mTo6m
 * @property {CentsUsdPattern} _6mTo9m
 * @property {CentsUsdPattern} _9mTo1y
 * @property {CentsUsdPattern} _1yTo18m
 * @property {CentsUsdPattern} _18mTo2y
 * @property {CentsUsdPattern} _2yTo3y
 * @property {CentsUsdPattern} _3yTo4y
 * @property {CentsUsdPattern} _4yTo5y
 * @property {CentsUsdPattern} _5yTo6y
 * @property {CentsUsdPattern} _6yTo7y
 * @property {CentsUsdPattern} _7yTo8y
 * @property {CentsUsdPattern} _8yTo10y
 * @property {CentsUsdPattern} _10yTo12y
 * @property {CentsUsdPattern} _12yTo15y
 * @property {CentsUsdPattern} over15y
 */

/**
 * @typedef {Object} SeriesTree_Cohorts_Realized_Cap_Epoch
 * @property {CentsUsdPattern} _0
 * @property {CentsUsdPattern} _1
 * @property {CentsUsdPattern} _2
 * @property {CentsUsdPattern} _3
 * @property {CentsUsdPattern} _4
 */

/**
 * @typedef {Object} SeriesTree_Cohorts_Realized_Cap_Class
 * @property {CentsUsdPattern} _2009
 * @property {CentsUsdPattern} _2010
 * @property {CentsUsdPattern} _2011
 * @property {CentsUsdPattern} _2012
 * @property {CentsUsdPattern} _2013
 * @property {CentsUsdPattern} _2014
 * @property {CentsUsdPattern} _2015
 * @property {CentsUsdPattern} _2016
 * @property {CentsUsdPattern} _2017
 * @property {CentsUsdPattern} _2018
 * @property {CentsUsdPattern} _2019
 * @property {CentsUsdPattern} _2020
 * @property {CentsUsdPattern} _2021
 * @property {CentsUsdPattern} _2022
 * @property {CentsUsdPattern} _2023
 * @property {CentsUsdPattern} _2024
 * @property {CentsUsdPattern} _2025
 * @property {CentsUsdPattern} _2026
 */

/**
 * @typedef {Object} SeriesTree_Cohorts_Realized_Cap_Type
 * @property {CentsUsdPattern} p2pk65
 * @property {CentsUsdPattern} p2pk33
 * @property {CentsUsdPattern} p2pkh
 * @property {CentsUsdPattern} p2ms
 * @property {CentsUsdPattern} p2sh
 * @property {CentsUsdPattern} p2wpkh
 * @property {CentsUsdPattern} p2wsh
 * @property {CentsUsdPattern} p2tr
 * @property {CentsUsdPattern} p2a
 * @property {CentsUsdPattern} unknown
 * @property {CentsUsdPattern} empty
 */

/**
 * @typedef {Object} SeriesTree_Cohorts_Realized_Profit
 * @property {SeriesTree_Cohorts_Realized_Profit_Age} age
 * @property {SeriesTree_Cohorts_Realized_Profit_Epoch} epoch
 * @property {SeriesTree_Cohorts_Realized_Profit_Class} class
 * @property {SeriesTree_Cohorts_Realized_Profit_UtxoAmount} utxoAmount
 * @property {EmptyP2aP2msP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshUnknownPattern6} type
 */

/**
 * @typedef {Object} SeriesTree_Cohorts_Realized_Profit_Age
 * @property {BlockCumulativeSumPattern} under1h
 * @property {BlockCumulativeSumPattern} _1hTo1d
 * @property {BlockCumulativeSumPattern} _1dTo1w
 * @property {BlockCumulativeSumPattern} _1wTo1m
 * @property {BlockCumulativeSumPattern} _1mTo2m
 * @property {BlockCumulativeSumPattern} _2mTo3m
 * @property {BlockCumulativeSumPattern} _3mTo4m
 * @property {BlockCumulativeSumPattern} _4mTo5m
 * @property {BlockCumulativeSumPattern} _5mTo6m
 * @property {BlockCumulativeSumPattern} _6mTo9m
 * @property {BlockCumulativeSumPattern} _9mTo1y
 * @property {BlockCumulativeSumPattern} _1yTo18m
 * @property {BlockCumulativeSumPattern} _18mTo2y
 * @property {BlockCumulativeSumPattern} _2yTo3y
 * @property {BlockCumulativeSumPattern} _3yTo4y
 * @property {BlockCumulativeSumPattern} _4yTo5y
 * @property {BlockCumulativeSumPattern} _5yTo6y
 * @property {BlockCumulativeSumPattern} _6yTo7y
 * @property {BlockCumulativeSumPattern} _7yTo8y
 * @property {BlockCumulativeSumPattern} _8yTo10y
 * @property {BlockCumulativeSumPattern} _10yTo12y
 * @property {BlockCumulativeSumPattern} _12yTo15y
 * @property {BlockCumulativeSumPattern} over15y
 */

/**
 * @typedef {Object} SeriesTree_Cohorts_Realized_Profit_Epoch
 * @property {BlockCumulativeSumPattern} _0
 * @property {BlockCumulativeSumPattern} _1
 * @property {BlockCumulativeSumPattern} _2
 * @property {BlockCumulativeSumPattern} _3
 * @property {BlockCumulativeSumPattern} _4
 */

/**
 * @typedef {Object} SeriesTree_Cohorts_Realized_Profit_Class
 * @property {BlockCumulativeSumPattern} _2009
 * @property {BlockCumulativeSumPattern} _2010
 * @property {BlockCumulativeSumPattern} _2011
 * @property {BlockCumulativeSumPattern} _2012
 * @property {BlockCumulativeSumPattern} _2013
 * @property {BlockCumulativeSumPattern} _2014
 * @property {BlockCumulativeSumPattern} _2015
 * @property {BlockCumulativeSumPattern} _2016
 * @property {BlockCumulativeSumPattern} _2017
 * @property {BlockCumulativeSumPattern} _2018
 * @property {BlockCumulativeSumPattern} _2019
 * @property {BlockCumulativeSumPattern} _2020
 * @property {BlockCumulativeSumPattern} _2021
 * @property {BlockCumulativeSumPattern} _2022
 * @property {BlockCumulativeSumPattern} _2023
 * @property {BlockCumulativeSumPattern} _2024
 * @property {BlockCumulativeSumPattern} _2025
 * @property {BlockCumulativeSumPattern} _2026
 */

/**
 * @typedef {Object} SeriesTree_Cohorts_Realized_Profit_UtxoAmount
 * @property {BlockCumulativeSumPattern} _0sats
 * @property {BlockCumulativeSumPattern} _1satTo10sats
 * @property {BlockCumulativeSumPattern} _10satsTo100sats
 * @property {BlockCumulativeSumPattern} _100satsTo1kSats
 * @property {BlockCumulativeSumPattern} _1kSatsTo10kSats
 * @property {BlockCumulativeSumPattern} _10kSatsTo100kSats
 * @property {BlockCumulativeSumPattern} _100kSatsTo1mSats
 * @property {BlockCumulativeSumPattern} _1mSatsTo10mSats
 * @property {BlockCumulativeSumPattern} _10mSatsTo1btc
 * @property {BlockCumulativeSumPattern} _1btcTo10btc
 * @property {BlockCumulativeSumPattern} _10btcTo100btc
 * @property {BlockCumulativeSumPattern} _100btcTo1kBtc
 * @property {BlockCumulativeSumPattern} _1kBtcTo10kBtc
 * @property {BlockCumulativeSumPattern} _10kBtcTo100kBtc
 * @property {BlockCumulativeSumPattern} over100kBtc
 */

/**
 * @typedef {Object} SeriesTree_Cohorts_Realized_Loss
 * @property {SeriesTree_Cohorts_Realized_Loss_Age} age
 * @property {SeriesTree_Cohorts_Realized_Loss_Epoch} epoch
 * @property {SeriesTree_Cohorts_Realized_Loss_Class} class
 * @property {SeriesTree_Cohorts_Realized_Loss_UtxoAmount} utxoAmount
 * @property {EmptyP2aP2msP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshUnknownPattern6} type
 */

/**
 * @typedef {Object} SeriesTree_Cohorts_Realized_Loss_Age
 * @property {BlockCumulativeSumPattern} under1h
 * @property {BlockCumulativeSumPattern} _1hTo1d
 * @property {BlockCumulativeSumPattern} _1dTo1w
 * @property {BlockCumulativeSumPattern} _1wTo1m
 * @property {BlockCumulativeSumPattern} _1mTo2m
 * @property {BlockCumulativeSumPattern} _2mTo3m
 * @property {BlockCumulativeSumPattern} _3mTo4m
 * @property {BlockCumulativeSumPattern} _4mTo5m
 * @property {BlockCumulativeSumPattern} _5mTo6m
 * @property {BlockCumulativeSumPattern} _6mTo9m
 * @property {BlockCumulativeSumPattern} _9mTo1y
 * @property {BlockCumulativeSumPattern} _1yTo18m
 * @property {BlockCumulativeSumPattern} _18mTo2y
 * @property {BlockCumulativeSumPattern} _2yTo3y
 * @property {BlockCumulativeSumPattern} _3yTo4y
 * @property {BlockCumulativeSumPattern} _4yTo5y
 * @property {BlockCumulativeSumPattern} _5yTo6y
 * @property {BlockCumulativeSumPattern} _6yTo7y
 * @property {BlockCumulativeSumPattern} _7yTo8y
 * @property {BlockCumulativeSumPattern} _8yTo10y
 * @property {BlockCumulativeSumPattern} _10yTo12y
 * @property {BlockCumulativeSumPattern} _12yTo15y
 * @property {BlockCumulativeSumPattern} over15y
 */

/**
 * @typedef {Object} SeriesTree_Cohorts_Realized_Loss_Epoch
 * @property {BlockCumulativeSumPattern} _0
 * @property {BlockCumulativeSumPattern} _1
 * @property {BlockCumulativeSumPattern} _2
 * @property {BlockCumulativeSumPattern} _3
 * @property {BlockCumulativeSumPattern} _4
 */

/**
 * @typedef {Object} SeriesTree_Cohorts_Realized_Loss_Class
 * @property {BlockCumulativeSumPattern} _2009
 * @property {BlockCumulativeSumPattern} _2010
 * @property {BlockCumulativeSumPattern} _2011
 * @property {BlockCumulativeSumPattern} _2012
 * @property {BlockCumulativeSumPattern} _2013
 * @property {BlockCumulativeSumPattern} _2014
 * @property {BlockCumulativeSumPattern} _2015
 * @property {BlockCumulativeSumPattern} _2016
 * @property {BlockCumulativeSumPattern} _2017
 * @property {BlockCumulativeSumPattern} _2018
 * @property {BlockCumulativeSumPattern} _2019
 * @property {BlockCumulativeSumPattern} _2020
 * @property {BlockCumulativeSumPattern} _2021
 * @property {BlockCumulativeSumPattern} _2022
 * @property {BlockCumulativeSumPattern} _2023
 * @property {BlockCumulativeSumPattern} _2024
 * @property {BlockCumulativeSumPattern} _2025
 * @property {BlockCumulativeSumPattern} _2026
 */

/**
 * @typedef {Object} SeriesTree_Cohorts_Realized_Loss_UtxoAmount
 * @property {BlockCumulativeSumPattern} _0sats
 * @property {BlockCumulativeSumPattern} _1satTo10sats
 * @property {BlockCumulativeSumPattern} _10satsTo100sats
 * @property {BlockCumulativeSumPattern} _100satsTo1kSats
 * @property {BlockCumulativeSumPattern} _1kSatsTo10kSats
 * @property {BlockCumulativeSumPattern} _10kSatsTo100kSats
 * @property {BlockCumulativeSumPattern} _100kSatsTo1mSats
 * @property {BlockCumulativeSumPattern} _1mSatsTo10mSats
 * @property {BlockCumulativeSumPattern} _10mSatsTo1btc
 * @property {BlockCumulativeSumPattern} _1btcTo10btc
 * @property {BlockCumulativeSumPattern} _10btcTo100btc
 * @property {BlockCumulativeSumPattern} _100btcTo1kBtc
 * @property {BlockCumulativeSumPattern} _1kBtcTo10kBtc
 * @property {BlockCumulativeSumPattern} _10kBtcTo100kBtc
 * @property {BlockCumulativeSumPattern} over100kBtc
 */

/**
 * @typedef {Object} SeriesTree_Cohorts_Realized_NetPnl
 * @property {SeriesTree_Cohorts_Realized_NetPnl_Age} age
 * @property {SeriesTree_Cohorts_Realized_NetPnl_Epoch} epoch
 * @property {SeriesTree_Cohorts_Realized_NetPnl_Class} class
 */

/**
 * @typedef {Object} SeriesTree_Cohorts_Realized_NetPnl_Age
 * @property {BlockCumulativeDeltaSumPattern} under1h
 * @property {BlockCumulativeDeltaSumPattern} _1hTo1d
 * @property {BlockCumulativeDeltaSumPattern} _1dTo1w
 * @property {BlockCumulativeDeltaSumPattern} _1wTo1m
 * @property {BlockCumulativeDeltaSumPattern} _1mTo2m
 * @property {BlockCumulativeDeltaSumPattern} _2mTo3m
 * @property {BlockCumulativeDeltaSumPattern} _3mTo4m
 * @property {BlockCumulativeDeltaSumPattern} _4mTo5m
 * @property {BlockCumulativeDeltaSumPattern} _5mTo6m
 * @property {BlockCumulativeDeltaSumPattern} _6mTo9m
 * @property {BlockCumulativeDeltaSumPattern} _9mTo1y
 * @property {BlockCumulativeDeltaSumPattern} _1yTo18m
 * @property {BlockCumulativeDeltaSumPattern} _18mTo2y
 * @property {BlockCumulativeDeltaSumPattern} _2yTo3y
 * @property {BlockCumulativeDeltaSumPattern} _3yTo4y
 * @property {BlockCumulativeDeltaSumPattern} _4yTo5y
 * @property {BlockCumulativeDeltaSumPattern} _5yTo6y
 * @property {BlockCumulativeDeltaSumPattern} _6yTo7y
 * @property {BlockCumulativeDeltaSumPattern} _7yTo8y
 * @property {BlockCumulativeDeltaSumPattern} _8yTo10y
 * @property {BlockCumulativeDeltaSumPattern} _10yTo12y
 * @property {BlockCumulativeDeltaSumPattern} _12yTo15y
 * @property {BlockCumulativeDeltaSumPattern} over15y
 */

/**
 * @typedef {Object} SeriesTree_Cohorts_Realized_NetPnl_Epoch
 * @property {BlockCumulativeDeltaSumPattern} _0
 * @property {BlockCumulativeDeltaSumPattern} _1
 * @property {BlockCumulativeDeltaSumPattern} _2
 * @property {BlockCumulativeDeltaSumPattern} _3
 * @property {BlockCumulativeDeltaSumPattern} _4
 */

/**
 * @typedef {Object} SeriesTree_Cohorts_Realized_NetPnl_Class
 * @property {BlockCumulativeDeltaSumPattern} _2009
 * @property {BlockCumulativeDeltaSumPattern} _2010
 * @property {BlockCumulativeDeltaSumPattern} _2011
 * @property {BlockCumulativeDeltaSumPattern} _2012
 * @property {BlockCumulativeDeltaSumPattern} _2013
 * @property {BlockCumulativeDeltaSumPattern} _2014
 * @property {BlockCumulativeDeltaSumPattern} _2015
 * @property {BlockCumulativeDeltaSumPattern} _2016
 * @property {BlockCumulativeDeltaSumPattern} _2017
 * @property {BlockCumulativeDeltaSumPattern} _2018
 * @property {BlockCumulativeDeltaSumPattern} _2019
 * @property {BlockCumulativeDeltaSumPattern} _2020
 * @property {BlockCumulativeDeltaSumPattern} _2021
 * @property {BlockCumulativeDeltaSumPattern} _2022
 * @property {BlockCumulativeDeltaSumPattern} _2023
 * @property {BlockCumulativeDeltaSumPattern} _2024
 * @property {BlockCumulativeDeltaSumPattern} _2025
 * @property {BlockCumulativeDeltaSumPattern} _2026
 */

/**
 * @typedef {Object} SeriesTree_Cohorts_Realized_ValueDestroyed
 * @property {SeriesTree_Cohorts_Realized_ValueDestroyed_Age} age
 * @property {SeriesTree_Cohorts_Realized_ValueDestroyed_Epoch} epoch
 * @property {SeriesTree_Cohorts_Realized_ValueDestroyed_Class} class
 */

/**
 * @typedef {Object} SeriesTree_Cohorts_Realized_ValueDestroyed_Age
 * @property {AverageBlockCumulativeSumPattern3} under1h
 * @property {AverageBlockCumulativeSumPattern3} _1hTo1d
 * @property {AverageBlockCumulativeSumPattern3} _1dTo1w
 * @property {AverageBlockCumulativeSumPattern3} _1wTo1m
 * @property {AverageBlockCumulativeSumPattern3} _1mTo2m
 * @property {AverageBlockCumulativeSumPattern3} _2mTo3m
 * @property {AverageBlockCumulativeSumPattern3} _3mTo4m
 * @property {AverageBlockCumulativeSumPattern3} _4mTo5m
 * @property {AverageBlockCumulativeSumPattern3} _5mTo6m
 * @property {AverageBlockCumulativeSumPattern3} _6mTo9m
 * @property {AverageBlockCumulativeSumPattern3} _9mTo1y
 * @property {AverageBlockCumulativeSumPattern3} _1yTo18m
 * @property {AverageBlockCumulativeSumPattern3} _18mTo2y
 * @property {AverageBlockCumulativeSumPattern3} _2yTo3y
 * @property {AverageBlockCumulativeSumPattern3} _3yTo4y
 * @property {AverageBlockCumulativeSumPattern3} _4yTo5y
 * @property {AverageBlockCumulativeSumPattern3} _5yTo6y
 * @property {AverageBlockCumulativeSumPattern3} _6yTo7y
 * @property {AverageBlockCumulativeSumPattern3} _7yTo8y
 * @property {AverageBlockCumulativeSumPattern3} _8yTo10y
 * @property {AverageBlockCumulativeSumPattern3} _10yTo12y
 * @property {AverageBlockCumulativeSumPattern3} _12yTo15y
 * @property {AverageBlockCumulativeSumPattern3} over15y
 */

/**
 * @typedef {Object} SeriesTree_Cohorts_Realized_ValueDestroyed_Epoch
 * @property {AverageBlockCumulativeSumPattern3} _0
 * @property {AverageBlockCumulativeSumPattern3} _1
 * @property {AverageBlockCumulativeSumPattern3} _2
 * @property {AverageBlockCumulativeSumPattern3} _3
 * @property {AverageBlockCumulativeSumPattern3} _4
 */

/**
 * @typedef {Object} SeriesTree_Cohorts_Realized_ValueDestroyed_Class
 * @property {AverageBlockCumulativeSumPattern3} _2009
 * @property {AverageBlockCumulativeSumPattern3} _2010
 * @property {AverageBlockCumulativeSumPattern3} _2011
 * @property {AverageBlockCumulativeSumPattern3} _2012
 * @property {AverageBlockCumulativeSumPattern3} _2013
 * @property {AverageBlockCumulativeSumPattern3} _2014
 * @property {AverageBlockCumulativeSumPattern3} _2015
 * @property {AverageBlockCumulativeSumPattern3} _2016
 * @property {AverageBlockCumulativeSumPattern3} _2017
 * @property {AverageBlockCumulativeSumPattern3} _2018
 * @property {AverageBlockCumulativeSumPattern3} _2019
 * @property {AverageBlockCumulativeSumPattern3} _2020
 * @property {AverageBlockCumulativeSumPattern3} _2021
 * @property {AverageBlockCumulativeSumPattern3} _2022
 * @property {AverageBlockCumulativeSumPattern3} _2023
 * @property {AverageBlockCumulativeSumPattern3} _2024
 * @property {AverageBlockCumulativeSumPattern3} _2025
 * @property {AverageBlockCumulativeSumPattern3} _2026
 */

/**
 * @typedef {Object} SeriesTree_Cohorts_Realized_Price
 * @property {SeriesTree_Cohorts_Realized_Price_UtxoAmount} utxoAmount
 * @property {SeriesTree_Cohorts_Realized_Price_Type} type
 */

/**
 * @typedef {Object} SeriesTree_Cohorts_Realized_Price_UtxoAmount
 * @property {CentsSatsUsdPattern} _0sats
 * @property {CentsSatsUsdPattern} _1satTo10sats
 * @property {CentsSatsUsdPattern} _10satsTo100sats
 * @property {CentsSatsUsdPattern} _100satsTo1kSats
 * @property {CentsSatsUsdPattern} _1kSatsTo10kSats
 * @property {CentsSatsUsdPattern} _10kSatsTo100kSats
 * @property {CentsSatsUsdPattern} _100kSatsTo1mSats
 * @property {CentsSatsUsdPattern} _1mSatsTo10mSats
 * @property {CentsSatsUsdPattern} _10mSatsTo1btc
 * @property {CentsSatsUsdPattern} _1btcTo10btc
 * @property {CentsSatsUsdPattern} _10btcTo100btc
 * @property {CentsSatsUsdPattern} _100btcTo1kBtc
 * @property {CentsSatsUsdPattern} _1kBtcTo10kBtc
 * @property {CentsSatsUsdPattern} _10kBtcTo100kBtc
 * @property {CentsSatsUsdPattern} over100kBtc
 */

/**
 * @typedef {Object} SeriesTree_Cohorts_Realized_Price_Type
 * @property {CentsSatsUsdPattern} p2pk65
 * @property {CentsSatsUsdPattern} p2pk33
 * @property {CentsSatsUsdPattern} p2pkh
 * @property {CentsSatsUsdPattern} p2ms
 * @property {CentsSatsUsdPattern} p2sh
 * @property {CentsSatsUsdPattern} p2wpkh
 * @property {CentsSatsUsdPattern} p2wsh
 * @property {CentsSatsUsdPattern} p2tr
 * @property {CentsSatsUsdPattern} p2a
 * @property {CentsSatsUsdPattern} unknown
 * @property {CentsSatsUsdPattern} empty
 */

/**
 * @typedef {Object} SeriesTree_Cohorts_Unrealized
 * @property {SeriesTree_Cohorts_Unrealized_Profit} profit
 * @property {SeriesTree_Cohorts_Unrealized_Loss} loss
 * @property {SeriesTree_Cohorts_Unrealized_NetPnl} netPnl
 */

/**
 * @typedef {Object} SeriesTree_Cohorts_Unrealized_Profit
 * @property {SeriesTree_Cohorts_Unrealized_Profit_Age} age
 * @property {SeriesTree_Cohorts_Unrealized_Profit_Epoch} epoch
 * @property {SeriesTree_Cohorts_Unrealized_Profit_Class} class
 */

/**
 * @typedef {Object} SeriesTree_Cohorts_Unrealized_Profit_Age
 * @property {CentsUsdPattern} under1h
 * @property {CentsUsdPattern} _1hTo1d
 * @property {CentsUsdPattern} _1dTo1w
 * @property {CentsUsdPattern} _1wTo1m
 * @property {CentsUsdPattern} _1mTo2m
 * @property {CentsUsdPattern} _2mTo3m
 * @property {CentsUsdPattern} _3mTo4m
 * @property {CentsUsdPattern} _4mTo5m
 * @property {CentsUsdPattern} _5mTo6m
 * @property {CentsUsdPattern} _6mTo9m
 * @property {CentsUsdPattern} _9mTo1y
 * @property {CentsUsdPattern} _1yTo18m
 * @property {CentsUsdPattern} _18mTo2y
 * @property {CentsUsdPattern} _2yTo3y
 * @property {CentsUsdPattern} _3yTo4y
 * @property {CentsUsdPattern} _4yTo5y
 * @property {CentsUsdPattern} _5yTo6y
 * @property {CentsUsdPattern} _6yTo7y
 * @property {CentsUsdPattern} _7yTo8y
 * @property {CentsUsdPattern} _8yTo10y
 * @property {CentsUsdPattern} _10yTo12y
 * @property {CentsUsdPattern} _12yTo15y
 * @property {CentsUsdPattern} over15y
 */

/**
 * @typedef {Object} SeriesTree_Cohorts_Unrealized_Profit_Epoch
 * @property {CentsUsdPattern} _0
 * @property {CentsUsdPattern} _1
 * @property {CentsUsdPattern} _2
 * @property {CentsUsdPattern} _3
 * @property {CentsUsdPattern} _4
 */

/**
 * @typedef {Object} SeriesTree_Cohorts_Unrealized_Profit_Class
 * @property {CentsUsdPattern} _2009
 * @property {CentsUsdPattern} _2010
 * @property {CentsUsdPattern} _2011
 * @property {CentsUsdPattern} _2012
 * @property {CentsUsdPattern} _2013
 * @property {CentsUsdPattern} _2014
 * @property {CentsUsdPattern} _2015
 * @property {CentsUsdPattern} _2016
 * @property {CentsUsdPattern} _2017
 * @property {CentsUsdPattern} _2018
 * @property {CentsUsdPattern} _2019
 * @property {CentsUsdPattern} _2020
 * @property {CentsUsdPattern} _2021
 * @property {CentsUsdPattern} _2022
 * @property {CentsUsdPattern} _2023
 * @property {CentsUsdPattern} _2024
 * @property {CentsUsdPattern} _2025
 * @property {CentsUsdPattern} _2026
 */

/**
 * @typedef {Object} SeriesTree_Cohorts_Unrealized_Loss
 * @property {SeriesTree_Cohorts_Unrealized_Loss_Age} age
 * @property {SeriesTree_Cohorts_Unrealized_Loss_Epoch} epoch
 * @property {SeriesTree_Cohorts_Unrealized_Loss_Class} class
 */

/**
 * @typedef {Object} SeriesTree_Cohorts_Unrealized_Loss_Age
 * @property {CentsUsdPattern} under1h
 * @property {CentsUsdPattern} _1hTo1d
 * @property {CentsUsdPattern} _1dTo1w
 * @property {CentsUsdPattern} _1wTo1m
 * @property {CentsUsdPattern} _1mTo2m
 * @property {CentsUsdPattern} _2mTo3m
 * @property {CentsUsdPattern} _3mTo4m
 * @property {CentsUsdPattern} _4mTo5m
 * @property {CentsUsdPattern} _5mTo6m
 * @property {CentsUsdPattern} _6mTo9m
 * @property {CentsUsdPattern} _9mTo1y
 * @property {CentsUsdPattern} _1yTo18m
 * @property {CentsUsdPattern} _18mTo2y
 * @property {CentsUsdPattern} _2yTo3y
 * @property {CentsUsdPattern} _3yTo4y
 * @property {CentsUsdPattern} _4yTo5y
 * @property {CentsUsdPattern} _5yTo6y
 * @property {CentsUsdPattern} _6yTo7y
 * @property {CentsUsdPattern} _7yTo8y
 * @property {CentsUsdPattern} _8yTo10y
 * @property {CentsUsdPattern} _10yTo12y
 * @property {CentsUsdPattern} _12yTo15y
 * @property {CentsUsdPattern} over15y
 */

/**
 * @typedef {Object} SeriesTree_Cohorts_Unrealized_Loss_Epoch
 * @property {CentsUsdPattern} _0
 * @property {CentsUsdPattern} _1
 * @property {CentsUsdPattern} _2
 * @property {CentsUsdPattern} _3
 * @property {CentsUsdPattern} _4
 */

/**
 * @typedef {Object} SeriesTree_Cohorts_Unrealized_Loss_Class
 * @property {CentsUsdPattern} _2009
 * @property {CentsUsdPattern} _2010
 * @property {CentsUsdPattern} _2011
 * @property {CentsUsdPattern} _2012
 * @property {CentsUsdPattern} _2013
 * @property {CentsUsdPattern} _2014
 * @property {CentsUsdPattern} _2015
 * @property {CentsUsdPattern} _2016
 * @property {CentsUsdPattern} _2017
 * @property {CentsUsdPattern} _2018
 * @property {CentsUsdPattern} _2019
 * @property {CentsUsdPattern} _2020
 * @property {CentsUsdPattern} _2021
 * @property {CentsUsdPattern} _2022
 * @property {CentsUsdPattern} _2023
 * @property {CentsUsdPattern} _2024
 * @property {CentsUsdPattern} _2025
 * @property {CentsUsdPattern} _2026
 */

/**
 * @typedef {Object} SeriesTree_Cohorts_Unrealized_NetPnl
 * @property {SeriesTree_Cohorts_Unrealized_NetPnl_Age} age
 * @property {SeriesTree_Cohorts_Unrealized_NetPnl_Epoch} epoch
 * @property {SeriesTree_Cohorts_Unrealized_NetPnl_Class} class
 */

/**
 * @typedef {Object} SeriesTree_Cohorts_Unrealized_NetPnl_Age
 * @property {CentsUsdPattern4} under1h
 * @property {CentsUsdPattern4} _1hTo1d
 * @property {CentsUsdPattern4} _1dTo1w
 * @property {CentsUsdPattern4} _1wTo1m
 * @property {CentsUsdPattern4} _1mTo2m
 * @property {CentsUsdPattern4} _2mTo3m
 * @property {CentsUsdPattern4} _3mTo4m
 * @property {CentsUsdPattern4} _4mTo5m
 * @property {CentsUsdPattern4} _5mTo6m
 * @property {CentsUsdPattern4} _6mTo9m
 * @property {CentsUsdPattern4} _9mTo1y
 * @property {CentsUsdPattern4} _1yTo18m
 * @property {CentsUsdPattern4} _18mTo2y
 * @property {CentsUsdPattern4} _2yTo3y
 * @property {CentsUsdPattern4} _3yTo4y
 * @property {CentsUsdPattern4} _4yTo5y
 * @property {CentsUsdPattern4} _5yTo6y
 * @property {CentsUsdPattern4} _6yTo7y
 * @property {CentsUsdPattern4} _7yTo8y
 * @property {CentsUsdPattern4} _8yTo10y
 * @property {CentsUsdPattern4} _10yTo12y
 * @property {CentsUsdPattern4} _12yTo15y
 * @property {CentsUsdPattern4} over15y
 */

/**
 * @typedef {Object} SeriesTree_Cohorts_Unrealized_NetPnl_Epoch
 * @property {CentsUsdPattern4} _0
 * @property {CentsUsdPattern4} _1
 * @property {CentsUsdPattern4} _2
 * @property {CentsUsdPattern4} _3
 * @property {CentsUsdPattern4} _4
 */

/**
 * @typedef {Object} SeriesTree_Cohorts_Unrealized_NetPnl_Class
 * @property {CentsUsdPattern4} _2009
 * @property {CentsUsdPattern4} _2010
 * @property {CentsUsdPattern4} _2011
 * @property {CentsUsdPattern4} _2012
 * @property {CentsUsdPattern4} _2013
 * @property {CentsUsdPattern4} _2014
 * @property {CentsUsdPattern4} _2015
 * @property {CentsUsdPattern4} _2016
 * @property {CentsUsdPattern4} _2017
 * @property {CentsUsdPattern4} _2018
 * @property {CentsUsdPattern4} _2019
 * @property {CentsUsdPattern4} _2020
 * @property {CentsUsdPattern4} _2021
 * @property {CentsUsdPattern4} _2022
 * @property {CentsUsdPattern4} _2023
 * @property {CentsUsdPattern4} _2024
 * @property {CentsUsdPattern4} _2025
 * @property {CentsUsdPattern4} _2026
 */

/**
 * @typedef {Object} SeriesTree_Cohorts_Urpd
 * @property {SeriesTree_Cohorts_Urpd_AgeBounds} ageBounds
 */

/**
 * @typedef {Object} SeriesTree_Cohorts_Urpd_AgeBounds
 * @property {MaxMinPattern} all
 * @property {MaxMinPattern} sth
 * @property {MaxMinPattern} lth
 * @property {MaxMinPattern} under4m
 * @property {MaxMinPattern} under6m
 * @property {MaxMinPattern} over4m
 * @property {MaxMinPattern} over6m
 */

/**
 * @typedef {Object} SeriesTree_DistributionAggregated
 * @property {SeriesTree_DistributionAggregated_Cohorts} cohorts
 */

/**
 * @typedef {Object} SeriesTree_DistributionAggregated_Cohorts
 * @property {SeriesTree_DistributionAggregated_Cohorts_All} all
 * @property {ActivityCostOutputsRatiosRealizedRelativeSupplyUnrealizedPattern} sth
 * @property {ActivityCostOutputsRatiosRealizedRelativeSupplyUnrealizedPattern} lth
 * @property {ActivityCostOutputsRatiosRealizedRelativeSupplyUnrealizedPattern} under4m
 * @property {ActivityCostOutputsRatiosRealizedRelativeSupplyUnrealizedPattern} under6m
 * @property {ActivityCostOutputsRatiosRealizedRelativeSupplyUnrealizedPattern} over4m
 * @property {ActivityCostOutputsRatiosRealizedRelativeSupplyUnrealizedPattern} over6m
 */

/**
 * @typedef {Object} SeriesTree_DistributionAggregated_Cohorts_All
 * @property {DeltaInTotalPattern} supply
 * @property {SeriesTree_DistributionAggregated_Cohorts_All_Outputs} outputs
 * @property {SeriesTree_DistributionAggregated_Cohorts_All_Activity} activity
 * @property {SeriesTree_DistributionAggregated_Cohorts_All_Realized} realized
 * @property {SeriesTree_DistributionAggregated_Cohorts_All_Unrealized} unrealized
 * @property {SeriesTree_DistributionAggregated_Cohorts_All_CostBasis} costBasis
 * @property {SeriesTree_DistributionAggregated_Cohorts_All_Ratios} ratios
 * @property {SeriesTree_DistributionAggregated_Cohorts_All_Relative} relative
 */

/**
 * @typedef {Object} SeriesTree_DistributionAggregated_Cohorts_All_Outputs
 * @property {BaseDeltaPattern} unspentCount
 * @property {AverageBlockCumulativeSumPattern2<StoredU64>} spentCount
 */

/**
 * @typedef {Object} SeriesTree_DistributionAggregated_Cohorts_All_Activity
 * @property {AverageBlockCumulativeSumPattern} transferVolume
 * @property {AverageBlockCumulativeSumPattern} transferVolumeInProfit
 * @property {AverageBlockCumulativeSumPattern} transferVolumeInLoss
 * @property {AverageBlockCumulativeSumPattern2<StoredF64>} coindaysDestroyed
 * @property {SeriesPattern1<StoredF64>} coinyearsDestroyed
 */

/**
 * @typedef {Object} SeriesTree_DistributionAggregated_Cohorts_All_Realized
 * @property {CentsDeltaUsdPattern} cap
 * @property {CentsSatsUsdPattern} price
 * @property {CentsPpmRatioSatsUsdPattern} capitalizedPrice
 * @property {BlockCumulativeSumPattern} profit
 * @property {BlockCumulativeSumPattern} loss
 * @property {BlockCumulativeDeltaSumPattern} netPnl
 * @property {AverageBlockCumulativeSumPattern3} valueDestroyed
 * @property {BlockCumulativeSumPattern} grossPnl
 * @property {BlockCumulativeSumPattern} peakRegret
 * @property {PpmRatioPattern2} mvrv
 */

/**
 * @typedef {Object} SeriesTree_DistributionAggregated_Cohorts_All_Unrealized
 * @property {CentsUsdPattern} profit
 * @property {CentsUsdPattern} loss
 * @property {CentsUsdPattern4} netPnl
 * @property {CentsUsdPattern} grossPnl
 * @property {CentsUsdPattern} investedCapitalInProfit
 * @property {CentsUsdPattern} investedCapitalInLoss
 * @property {CentsUsdPattern} painIndex
 * @property {CentsUsdPattern} greedIndex
 * @property {CentsUsdPattern4} netSentiment
 * @property {PpmRatioPattern3} nupl
 */

/**
 * @typedef {Object} SeriesTree_DistributionAggregated_Cohorts_All_CostBasis
 * @property {PerPattern2} inProfit
 * @property {PerPattern2} inLoss
 * @property {CentsSatsUsdPattern} min
 * @property {CentsSatsUsdPattern} max
 * @property {Pct05Pct10Pct15Pct20Pct25Pct30Pct35Pct40Pct45Pct50Pct55Pct60Pct65Pct70Pct75Pct80Pct85Pct90Pct95Pattern} perCoin
 * @property {Pct05Pct10Pct15Pct20Pct25Pct30Pct35Pct40Pct45Pct50Pct55Pct60Pct65Pct70Pct75Pct80Pct85Pct90Pct95Pattern} perDollar
 * @property {PercentPpmRatioPattern2} supplyDensity
 */

/**
 * @typedef {Object} SeriesTree_DistributionAggregated_Cohorts_All_Ratios
 * @property {SeriesTree_DistributionAggregated_Cohorts_All_Ratios_AdjustedSopr} adjustedSopr
 * @property {_1m1w1y24hPattern6<StoredF32>} dormancy
 * @property {SeriesPattern1<StoredF32>} sopr
 * @property {_1m1w1yPattern} soprRatioExtended
 * @property {_1m1w1y24hPattern9} sellSideRiskRatio
 * @property {_1m1w1y24hPattern6<StoredF32>} profitToLossRatio
 */

/**
 * @typedef {Object} SeriesTree_DistributionAggregated_Cohorts_All_Ratios_AdjustedSopr
 * @property {_1m1w1y24hPattern6<StoredF32>} ratio
 * @property {AverageBlockCumulativeSumPattern3} transferVolume
 * @property {AverageBlockCumulativeSumPattern3} valueDestroyed
 */

/**
 * @typedef {Object} SeriesTree_DistributionAggregated_Cohorts_All_Relative
 * @property {PercentPpmRatioPattern2} supplyDominance
 * @property {PercentPpmRatioPattern2} supplyInProfitShare
 * @property {PercentPpmRatioPattern2} supplyInLossShare
 * @property {PercentPpmRatioPattern2} unrealizedProfitToMcap
 * @property {PercentPpmRatioPattern2} unrealizedLossToMcap
 * @property {PercentPpmRatioPattern2} unrealizedProfitToOwnMcap
 * @property {PercentPpmRatioPattern2} unrealizedLossToOwnMcap
 * @property {PercentPpmRatioPattern2} unrealizedProfitToOwnGrossPnl
 * @property {PercentPpmRatioPattern2} unrealizedLossToOwnGrossPnl
 * @property {PercentPpmRatioPattern3} netUnrealizedPnlToOwnGrossPnl
 * @property {PercentPpmRatioPattern2} investedCapitalInProfitShare
 * @property {PercentPpmRatioPattern2} investedCapitalInLossShare
 * @property {PercentPpmRatioPattern2} realizedCapToOwnMcap
 * @property {PercentPpmRatioPattern} netPnlChange1mToMcap
 * @property {PercentPpmRatioPattern} netPnlChange1mToRcap
 */

/**
 * @typedef {Object} SeriesTree_Supply
 * @property {BtcCentsSatsUsdPattern} circulating
 * @property {BlockCumulativePattern} burned
 * @property {PercentPpmRatioPattern} inflationRate
 * @property {SeriesTree_Supply_Velocity} velocity
 * @property {CentsDeltaUsdPattern} marketCap
 * @property {_1m1w1y24hPattern6<PartsPerMillionSigned64>} marketMinusRealizedCapGrowthRate
 * @property {BtcCentsSatsUsdPattern} hodledOrLost
 */

/**
 * @typedef {Object} SeriesTree_Supply_Velocity
 * @property {SeriesPattern1<StoredF64>} native
 * @property {SeriesPattern1<StoredF64>} fiat
 */

/**
 * @typedef {Object} SeriesTree_UtxoHistory
 * @property {SeriesPattern18<Sats>} supply
 * @property {SeriesPattern1<StoredU64>} count
 */

/**
 * Main Bitview client with series tree and API methods
 * @extends BitviewClientBase
 */
class BitviewClient extends BitviewClientBase {
  VERSION = "v0.12.2";

  INDEXES = /** @type {const} */ ([
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
  ]);

  POOL_ID_TO_POOL_NAME = /** @type {const} */ ({
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
  });

  TERM_NAMES = /** @type {const} */ ({
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
  });

  EPOCH_NAMES = /** @type {const} */ ({
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
  });

  CLASS_NAMES = /** @type {const} */ ({
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
  });

  ENTRY_NAMES = /** @type {const} */ ({
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
  });

  SPENDABLE_TYPE_NAMES = /** @type {const} */ ({
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
  });

  AGE_RANGE_NAMES = /** @type {const} */ ({
    "under1h": {
      "id": "under_1h_old",
      "short": "<1h",
      "long": "Under 1 Hour Old"
    },
    "_1hTo1d": {
      "id": "1h_to_1d_old",
      "short": "1h-1d",
      "long": "1 Hour to 1 Day Old"
    },
    "_1dTo1w": {
      "id": "1d_to_1w_old",
      "short": "1d-1w",
      "long": "1 Day to 1 Week Old"
    },
    "_1wTo1m": {
      "id": "1w_to_1m_old",
      "short": "1w-1m",
      "long": "1 Week to 1 Month Old"
    },
    "_1mTo2m": {
      "id": "1m_to_2m_old",
      "short": "1m-2m",
      "long": "1 to 2 Months Old"
    },
    "_2mTo3m": {
      "id": "2m_to_3m_old",
      "short": "2m-3m",
      "long": "2 to 3 Months Old"
    },
    "_3mTo4m": {
      "id": "3m_to_4m_old",
      "short": "3m-4m",
      "long": "3 to 4 Months Old"
    },
    "_4mTo5m": {
      "id": "4m_to_5m_old",
      "short": "4m-5m",
      "long": "4 to 5 Months Old"
    },
    "_5mTo6m": {
      "id": "5m_to_6m_old",
      "short": "5m-6m",
      "long": "5 to 6 Months Old"
    },
    "_6mTo9m": {
      "id": "6m_to_9m_old",
      "short": "6m-9m",
      "long": "6 to 9 Months Old"
    },
    "_9mTo1y": {
      "id": "9m_to_1y_old",
      "short": "9m-1y",
      "long": "9 Months to 1 Year Old"
    },
    "_1yTo18m": {
      "id": "1y_to_18m_old",
      "short": "1y-18m",
      "long": "1 Year to 18 Months Old"
    },
    "_18mTo2y": {
      "id": "18m_to_2y_old",
      "short": "18m-2y",
      "long": "18 Months to 2 Years Old"
    },
    "_2yTo3y": {
      "id": "2y_to_3y_old",
      "short": "2y-3y",
      "long": "2 to 3 Years Old"
    },
    "_3yTo4y": {
      "id": "3y_to_4y_old",
      "short": "3y-4y",
      "long": "3 to 4 Years Old"
    },
    "_4yTo5y": {
      "id": "4y_to_5y_old",
      "short": "4y-5y",
      "long": "4 to 5 Years Old"
    },
    "_5yTo6y": {
      "id": "5y_to_6y_old",
      "short": "5y-6y",
      "long": "5 to 6 Years Old"
    },
    "_6yTo7y": {
      "id": "6y_to_7y_old",
      "short": "6y-7y",
      "long": "6 to 7 Years Old"
    },
    "_7yTo8y": {
      "id": "7y_to_8y_old",
      "short": "7y-8y",
      "long": "7 to 8 Years Old"
    },
    "_8yTo10y": {
      "id": "8y_to_10y_old",
      "short": "8y-10y",
      "long": "8 to 10 Years Old"
    },
    "_10yTo12y": {
      "id": "10y_to_12y_old",
      "short": "10y-12y",
      "long": "10 to 12 Years Old"
    },
    "_12yTo15y": {
      "id": "12y_to_15y_old",
      "short": "12y-15y",
      "long": "12 to 15 Years Old"
    },
    "over15y": {
      "id": "over_15y_old",
      "short": "15y+",
      "long": "15+ Years Old"
    }
  });

  AMOUNT_RANGE_NAMES = /** @type {const} */ ({
    "_0sats": {
      "id": "0sats",
      "short": "0 sats",
      "long": "0 Sats"
    },
    "_1satTo10sats": {
      "id": "1sat_to_10sats",
      "short": "1-10 sats",
      "long": "1-10 Sats"
    },
    "_10satsTo100sats": {
      "id": "10sats_to_100sats",
      "short": "10-100 sats",
      "long": "10-100 Sats"
    },
    "_100satsTo1kSats": {
      "id": "100sats_to_1k_sats",
      "short": "100-1k sats",
      "long": "100-1K Sats"
    },
    "_1kSatsTo10kSats": {
      "id": "1k_sats_to_10k_sats",
      "short": "1k-10k sats",
      "long": "1K-10K Sats"
    },
    "_10kSatsTo100kSats": {
      "id": "10k_sats_to_100k_sats",
      "short": "10k-100k sats",
      "long": "10K-100K Sats"
    },
    "_100kSatsTo1mSats": {
      "id": "100k_sats_to_1m_sats",
      "short": "100k-1M sats",
      "long": "100K-1M Sats"
    },
    "_1mSatsTo10mSats": {
      "id": "1m_sats_to_10m_sats",
      "short": "1M-10M sats",
      "long": "1M-10M Sats"
    },
    "_10mSatsTo1btc": {
      "id": "10m_sats_to_1btc",
      "short": "0.1-1 BTC",
      "long": "0.1-1 BTC"
    },
    "_1btcTo10btc": {
      "id": "1btc_to_10btc",
      "short": "1-10 BTC",
      "long": "1-10 BTC"
    },
    "_10btcTo100btc": {
      "id": "10btc_to_100btc",
      "short": "10-100 BTC",
      "long": "10-100 BTC"
    },
    "_100btcTo1kBtc": {
      "id": "100btc_to_1k_btc",
      "short": "100-1k BTC",
      "long": "100-1K BTC"
    },
    "_1kBtcTo10kBtc": {
      "id": "1k_btc_to_10k_btc",
      "short": "1k-10k BTC",
      "long": "1K-10K BTC"
    },
    "_10kBtcTo100kBtc": {
      "id": "10k_btc_to_100k_btc",
      "short": "10k-100k BTC",
      "long": "10K-100K BTC"
    },
    "over100kBtc": {
      "id": "over_100k_btc",
      "short": "100k+ BTC",
      "long": "100K+ BTC"
    }
  });

  PROFITABILITY_RANGE_NAMES = /** @type {const} */ ({
    "over1000pctInProfit": {
      "id": "utxos_over_1000pct_in_profit",
      "short": "+>1000%",
      "long": "Over 1000% in Profit"
    },
    "_500pctTo1000pctInProfit": {
      "id": "utxos_500pct_to_1000pct_in_profit",
      "short": "+500-1000%",
      "long": "500-1000% in Profit"
    },
    "_300pctTo500pctInProfit": {
      "id": "utxos_300pct_to_500pct_in_profit",
      "short": "+300-500%",
      "long": "300-500% in Profit"
    },
    "_200pctTo300pctInProfit": {
      "id": "utxos_200pct_to_300pct_in_profit",
      "short": "+200-300%",
      "long": "200-300% in Profit"
    },
    "_100pctTo200pctInProfit": {
      "id": "utxos_100pct_to_200pct_in_profit",
      "short": "+100-200%",
      "long": "100-200% in Profit"
    },
    "_90pctTo100pctInProfit": {
      "id": "utxos_90pct_to_100pct_in_profit",
      "short": "+90-100%",
      "long": "90-100% in Profit"
    },
    "_80pctTo90pctInProfit": {
      "id": "utxos_80pct_to_90pct_in_profit",
      "short": "+80-90%",
      "long": "80-90% in Profit"
    },
    "_70pctTo80pctInProfit": {
      "id": "utxos_70pct_to_80pct_in_profit",
      "short": "+70-80%",
      "long": "70-80% in Profit"
    },
    "_60pctTo70pctInProfit": {
      "id": "utxos_60pct_to_70pct_in_profit",
      "short": "+60-70%",
      "long": "60-70% in Profit"
    },
    "_50pctTo60pctInProfit": {
      "id": "utxos_50pct_to_60pct_in_profit",
      "short": "+50-60%",
      "long": "50-60% in Profit"
    },
    "_40pctTo50pctInProfit": {
      "id": "utxos_40pct_to_50pct_in_profit",
      "short": "+40-50%",
      "long": "40-50% in Profit"
    },
    "_30pctTo40pctInProfit": {
      "id": "utxos_30pct_to_40pct_in_profit",
      "short": "+30-40%",
      "long": "30-40% in Profit"
    },
    "_20pctTo30pctInProfit": {
      "id": "utxos_20pct_to_30pct_in_profit",
      "short": "+20-30%",
      "long": "20-30% in Profit"
    },
    "_10pctTo20pctInProfit": {
      "id": "utxos_10pct_to_20pct_in_profit",
      "short": "+10-20%",
      "long": "10-20% in Profit"
    },
    "_0pctTo10pctInProfit": {
      "id": "utxos_0pct_to_10pct_in_profit",
      "short": "+0-10%",
      "long": "0-10% in Profit"
    },
    "_0pctTo10pctInLoss": {
      "id": "utxos_0pct_to_10pct_in_loss",
      "short": "-0-10%",
      "long": "0-10% in Loss"
    },
    "_10pctTo20pctInLoss": {
      "id": "utxos_10pct_to_20pct_in_loss",
      "short": "-10-20%",
      "long": "10-20% in Loss"
    },
    "_20pctTo30pctInLoss": {
      "id": "utxos_20pct_to_30pct_in_loss",
      "short": "-20-30%",
      "long": "20-30% in Loss"
    },
    "_30pctTo40pctInLoss": {
      "id": "utxos_30pct_to_40pct_in_loss",
      "short": "-30-40%",
      "long": "30-40% in Loss"
    },
    "_40pctTo50pctInLoss": {
      "id": "utxos_40pct_to_50pct_in_loss",
      "short": "-40-50%",
      "long": "40-50% in Loss"
    },
    "_50pctTo60pctInLoss": {
      "id": "utxos_50pct_to_60pct_in_loss",
      "short": "-50-60%",
      "long": "50-60% in Loss"
    },
    "_60pctTo70pctInLoss": {
      "id": "utxos_60pct_to_70pct_in_loss",
      "short": "-60-70%",
      "long": "60-70% in Loss"
    },
    "_70pctTo80pctInLoss": {
      "id": "utxos_70pct_to_80pct_in_loss",
      "short": "-70-80%",
      "long": "70-80% in Loss"
    },
    "_80pctTo90pctInLoss": {
      "id": "utxos_80pct_to_90pct_in_loss",
      "short": "-80-90%",
      "long": "80-90% in Loss"
    },
    "_90pctTo100pctInLoss": {
      "id": "utxos_90pct_to_100pct_in_loss",
      "short": "-90-100%",
      "long": "90-100% in Loss"
    }
  });

  /**
   * Convert an index value to a Date for date-based indexes.
   * @param {Index} index - The index type
   * @param {number} i - The index value
   * @returns {globalThis.Date}
   */
  indexToDate(index, i) {
    return indexToDate(index, i);
  }

  /**
   * Convert a Date to an index value for date-based indexes.
   * @param {Index} index - The index type
   * @param {globalThis.Date} d - The date to convert
   * @returns {number}
   */
  dateToIndex(index, d) {
    return dateToIndex(index, d);
  }


  /**
   * @param {BitviewClientOptions|string} options
   */
  constructor(options) {
    super(options);
  }

  /** @returns {SeriesTree} */
  get series() {
    return _lazy(this, 'series', () => this._buildTree());
  }

  /**
   * Compute the RapidHash v3 hash-prefix for raw address payload bytes.
   * @param {Uint8Array | ArrayBuffer | ArrayBufferView | number[]} payload
   * @param {number} nibbles
   * @returns {string}
   */
  static addressPayloadHashPrefix(payload, nibbles) {
    return addressPayloadHashPrefix(payload, nibbles);
  }

  /**
   * Fetch address hash-prefix matches from raw address payload bytes.
   * @param {OutputType} addrType
   * @param {Uint8Array | ArrayBuffer | ArrayBufferView | number[]} payload - Raw payload bytes matching addrType length
   * @param {number} nibbles
   * @param {{ signal?: AbortSignal, onValue?: (value: AddrHashPrefixMatches) => void, cache?: boolean, memCache?: boolean }} [options]
   * @returns {Promise<AddrHashPrefixMatches>}
   */
  getAddressPayloadHashPrefixMatches(addrType, payload, nibbles, options = {}) {
    _validateAddressPayloadForType(addrType, payload);
    const prefix = addressPayloadHashPrefix(payload, nibbles);
    return this.getAddressHashPrefixMatches(addrType, prefix, options);
  }

  /**
   * @private
   * @returns {SeriesTree}
   */
  _buildTree() {
    const client = this;
    return {
      get blocks() { return _lazy(this, 'blocks', () => ({
        blockhash: createSeriesPattern18(client, 'blockhash'),
        coinbaseTag: createSeriesPattern18(client, 'coinbase_tag'),
        get difficulty() { return _lazy(this, 'difficulty', () => ({
          value: createSeriesPattern1(client, 'difficulty'),
          hashrate: createSeriesPattern1(client, 'difficulty_hashrate'),
          adjustment: createPercentPpmRatioPattern3(client, 'difficulty_adjustment'),
          epoch: createSeriesPattern1(client, 'difficulty_epoch'),
          blocksToRetarget: createSeriesPattern1(client, 'blocks_to_retarget'),
          daysToRetarget: createSeriesPattern1(client, 'days_to_retarget'),
        })); },
        get time() { return _lazy(this, 'time', () => ({
          timestamp: createSeriesPattern18(client, 'timestamp'),
        })); },
        get size() { return _lazy(this, 'size', () => ({
          base: createSeriesPattern18(client, 'total_size'),
          cumulative: createSeriesPattern1(client, 'block_size_cumulative'),
          sum: create_1m1w1y24hPattern6(client, 'block_size_sum'),
          average: create_1m1w1y24hPattern6(client, 'block_size_average'),
          min: create_1m1w1y24hPattern6(client, 'block_size_min'),
          max: create_1m1w1y24hPattern6(client, 'block_size_max'),
          pct10: create_1m1w1y24hPattern6(client, 'block_size_pct10'),
          pct25: create_1m1w1y24hPattern6(client, 'block_size_pct25'),
          median: create_1m1w1y24hPattern6(client, 'block_size_median'),
          pct75: create_1m1w1y24hPattern6(client, 'block_size_pct75'),
          pct90: create_1m1w1y24hPattern6(client, 'block_size_pct90'),
        })); },
        get weight() { return _lazy(this, 'weight', () => ({
          base: createSeriesPattern18(client, 'block_weight'),
          cumulative: createSeriesPattern1(client, 'block_weight_cumulative'),
          sum: create_1m1w1y24hPattern6(client, 'block_weight_sum'),
          average: create_1m1w1y24hPattern6(client, 'block_weight_average'),
          min: create_1m1w1y24hPattern6(client, 'block_weight_min'),
          max: create_1m1w1y24hPattern6(client, 'block_weight_max'),
          pct10: create_1m1w1y24hPattern6(client, 'block_weight_pct10'),
          pct25: create_1m1w1y24hPattern6(client, 'block_weight_pct25'),
          median: create_1m1w1y24hPattern6(client, 'block_weight_median'),
          pct75: create_1m1w1y24hPattern6(client, 'block_weight_pct75'),
          pct90: create_1m1w1y24hPattern6(client, 'block_weight_pct90'),
        })); },
        segwitTxs: createSeriesPattern18(client, 'segwit_txs'),
        segwitSize: createSeriesPattern18(client, 'segwit_size'),
        segwitWeight: createSeriesPattern18(client, 'segwit_weight'),
        get count() { return _lazy(this, 'count', () => ({
          target: create_1m1w1y24hPattern6(client, 'block_count_target'),
          total: createAverageBlockCumulativeSumPattern2(client, 'block_count'),
        })); },
        get lookback() { return _lazy(this, 'lookback', () => ({
          _1h: createSeriesPattern18(client, 'height_1h_ago'),
          _24h: createSeriesPattern18(client, 'height_24h_ago'),
          _3d: createSeriesPattern18(client, 'height_3d_ago'),
          _1w: createSeriesPattern18(client, 'height_1w_ago'),
          _8d: createSeriesPattern18(client, 'height_8d_ago'),
          _9d: createSeriesPattern18(client, 'height_9d_ago'),
          _12d: createSeriesPattern18(client, 'height_12d_ago'),
          _13d: createSeriesPattern18(client, 'height_13d_ago'),
          _2w: createSeriesPattern18(client, 'height_2w_ago'),
          _21d: createSeriesPattern18(client, 'height_21d_ago'),
          _26d: createSeriesPattern18(client, 'height_26d_ago'),
          _1m: createSeriesPattern18(client, 'height_1m_ago'),
          _34d: createSeriesPattern18(client, 'height_34d_ago'),
          _50d: createSeriesPattern18(client, 'height_50d_ago'),
          _55d: createSeriesPattern18(client, 'height_55d_ago'),
          _2m: createSeriesPattern18(client, 'height_2m_ago'),
          _9w: createSeriesPattern18(client, 'height_9w_ago'),
          _12w: createSeriesPattern18(client, 'height_12w_ago'),
          _89d: createSeriesPattern18(client, 'height_89d_ago'),
          _3m: createSeriesPattern18(client, 'height_3m_ago'),
          _14w: createSeriesPattern18(client, 'height_14w_ago'),
          _111d: createSeriesPattern18(client, 'height_111d_ago'),
          _144d: createSeriesPattern18(client, 'height_144d_ago'),
          _6m: createSeriesPattern18(client, 'height_6m_ago'),
          _26w: createSeriesPattern18(client, 'height_26w_ago'),
          _200d: createSeriesPattern18(client, 'height_200d_ago'),
          _9m: createSeriesPattern18(client, 'height_9m_ago'),
          _350d: createSeriesPattern18(client, 'height_350d_ago'),
          _12m: createSeriesPattern18(client, 'height_12m_ago'),
          _1y: createSeriesPattern18(client, 'height_1y_ago'),
          _14m: createSeriesPattern18(client, 'height_14m_ago'),
          _2y: createSeriesPattern18(client, 'height_2y_ago'),
          _26m: createSeriesPattern18(client, 'height_26m_ago'),
          _3y: createSeriesPattern18(client, 'height_3y_ago'),
          _200w: createSeriesPattern18(client, 'height_200w_ago'),
          _4y: createSeriesPattern18(client, 'height_4y_ago'),
          _5y: createSeriesPattern18(client, 'height_5y_ago'),
          _6y: createSeriesPattern18(client, 'height_6y_ago'),
          _8y: createSeriesPattern18(client, 'height_8y_ago'),
          _9y: createSeriesPattern18(client, 'height_9y_ago'),
          _10y: createSeriesPattern18(client, 'height_10y_ago'),
          _12y: createSeriesPattern18(client, 'height_12y_ago'),
          _14y: createSeriesPattern18(client, 'height_14y_ago'),
          _26y: createSeriesPattern18(client, 'height_26y_ago'),
        })); },
        get interval() { return _lazy(this, 'interval', () => ({
          block: createSeriesPattern18(client, 'block_interval'),
          _24h: createSeriesPattern1(client, 'block_interval_average_24h'),
          _1w: createSeriesPattern1(client, 'block_interval_average_1w'),
          _1m: createSeriesPattern1(client, 'block_interval_average_1m'),
          _1y: createSeriesPattern1(client, 'block_interval_average_1y'),
        })); },
        vbytes: createAverageBlockCumulativeMaxMedianMinPct10Pct25Pct75Pct90SumPattern(client, 'block_vbytes'),
        get fullness() { return _lazy(this, 'fullness', () => ({
          ppm: createSeriesPattern18(client, 'block_fullness_ppm'),
          ratio: createSeriesPattern18(client, 'block_fullness_ratio'),
          percent: createSeriesPattern18(client, 'block_fullness'),
        })); },
        get halving() { return _lazy(this, 'halving', () => ({
          epoch: createSeriesPattern1(client, 'halving_epoch'),
          blocksToHalving: createSeriesPattern1(client, 'blocks_to_halving'),
          daysToHalving: createSeriesPattern1(client, 'days_to_halving'),
        })); },
      })); },
      get transactions() { return _lazy(this, 'transactions', () => ({
        get raw() { return _lazy(this, 'raw', () => ({
          firstTxIndex: createSeriesPattern18(client, 'first_tx_index'),
          txid: createSeriesPattern19(client, 'txid'),
          txVersion: createSeriesPattern19(client, 'tx_version'),
          rawLocktime: createSeriesPattern19(client, 'raw_locktime'),
          weight: createSeriesPattern19(client, 'tx_weight'),
          totalSize: createSeriesPattern19(client, 'total_size'),
          totalSigopCost: createSeriesPattern19(client, 'total_sigop_cost'),
          isExplicitlyRbf: createSeriesPattern19(client, 'is_explicitly_rbf'),
          firstTxinIndex: createSeriesPattern19(client, 'first_txin_index'),
          firstTxoutIndex: createSeriesPattern19(client, 'first_txout_index'),
        })); },
        get features() { return _lazy(this, 'features', () => ({
          get count() { return _lazy(this, 'count', () => ({
            v1: createSeriesPattern18(client, 'tx_count_v1'),
            v2: createSeriesPattern18(client, 'tx_count_v2'),
            v3: createSeriesPattern18(client, 'tx_count_v3'),
            otherVersion: createSeriesPattern18(client, 'tx_count_other_version'),
            explicitlyRbf: createSeriesPattern18(client, 'tx_count_explicitly_rbf'),
            oneInput: createSeriesPattern18(client, 'tx_count_one_input'),
            oneOutput: createSeriesPattern18(client, 'tx_count_one_output'),
            p2pk: createSeriesPattern18(client, 'tx_count_p2pk'),
            p2ms: createSeriesPattern18(client, 'tx_count_p2ms'),
            p2pkh: createSeriesPattern18(client, 'tx_count_p2pkh'),
            p2sh: createSeriesPattern18(client, 'tx_count_p2sh'),
            p2wpkh: createSeriesPattern18(client, 'tx_count_p2wpkh'),
            p2wsh: createSeriesPattern18(client, 'tx_count_p2wsh'),
            p2tr: createSeriesPattern18(client, 'tx_count_p2tr'),
            p2a: createSeriesPattern18(client, 'tx_count_p2a'),
            opReturn: createSeriesPattern18(client, 'tx_count_op_return'),
            empty: createSeriesPattern18(client, 'tx_count_empty'),
            unknown: createSeriesPattern18(client, 'tx_count_unknown'),
            fakePubkey: createSeriesPattern18(client, 'tx_count_fake_pubkey'),
            fakeScripthash: createSeriesPattern18(client, 'tx_count_fake_scripthash'),
            annex: createAverageBlockCumulativeSumPattern2(client, 'tx_count_annex'),
            sighashAll: createAverageBlockCumulativeSumPattern2(client, 'tx_count_sighash_all'),
            sighashNone: createAverageBlockCumulativeSumPattern2(client, 'tx_count_sighash_none'),
            sighashSingle: createAverageBlockCumulativeSumPattern2(client, 'tx_count_sighash_single'),
            sighashDefault: createAverageBlockCumulativeSumPattern2(client, 'tx_count_sighash_default'),
            sighashAnyoneCanPay: createAverageBlockCumulativeSumPattern2(client, 'tx_count_sighash_anyone_can_pay'),
            dustOutput: createAverageBlockCumulativeSumPattern2(client, 'tx_count_dust_output'),
          })); },
          hasP2pk: createSeriesPattern19(client, 'has_p2pk'),
          hasP2ms: createSeriesPattern19(client, 'has_p2ms'),
          hasP2pkh: createSeriesPattern19(client, 'has_p2pkh'),
          hasP2sh: createSeriesPattern19(client, 'has_p2sh'),
          hasP2wpkh: createSeriesPattern19(client, 'has_p2wpkh'),
          hasP2wsh: createSeriesPattern19(client, 'has_p2wsh'),
          hasP2tr: createSeriesPattern19(client, 'has_p2tr'),
          hasP2a: createSeriesPattern19(client, 'has_p2a'),
          hasOpReturn: createSeriesPattern19(client, 'has_op_return'),
          hasEmpty: createSeriesPattern19(client, 'has_empty'),
          hasUnknown: createSeriesPattern19(client, 'has_unknown'),
          hasFakePubkey: createSeriesPattern19(client, 'has_fake_pubkey'),
          hasFakeScripthash: createSeriesPattern19(client, 'has_fake_scripthash'),
          hasInscription: createSeriesPattern19(client, 'has_inscription'),
          hasAnnex: createSeriesPattern19(client, 'has_annex'),
          hasSighashAll: createSeriesPattern19(client, 'has_sighash_all'),
          hasSighashNone: createSeriesPattern19(client, 'has_sighash_none'),
          hasSighashSingle: createSeriesPattern19(client, 'has_sighash_single'),
          hasSighashDefault: createSeriesPattern19(client, 'has_sighash_default'),
          hasSighashAnyoneCanPay: createSeriesPattern19(client, 'has_sighash_anyone_can_pay'),
          hasDustOutput: createSeriesPattern19(client, 'has_dust_output'),
        })); },
        get count() { return _lazy(this, 'count', () => ({
          total: createAverageBlockCumulativeMaxMedianMinPct10Pct25Pct75Pct90SumPattern(client, 'tx_count'),
        })); },
        get size() { return _lazy(this, 'size', () => ({
          get vsize() { return _lazy(this, 'vsize', () => ({
            txIndex: createSeriesPattern19(client, 'tx_vsize'),
            block: createMaxMedianMinPct10Pct25Pct75Pct90Pattern2(client, 'tx_vsize'),
            _6b: createMaxMedianMinPct10Pct25Pct75Pct90Pattern2(client, 'tx_vsize_6b'),
          })); },
          get weight() { return _lazy(this, 'weight', () => ({
            block: createMaxMedianMinPct10Pct25Pct75Pct90Pattern(client, 'tx_weight'),
            _6b: createMaxMedianMinPct10Pct25Pct75Pct90Pattern(client, 'tx_weight_6b'),
          })); },
        })); },
        get fees() { return _lazy(this, 'fees', () => ({
          get count() { return _lazy(this, 'count', () => ({
            cpfpParent: createAverageBlockCumulativeSumPattern2(client, 'cpfp_parent_count'),
            cpfpChild: createAverageBlockCumulativeSumPattern2(client, 'cpfp_child_count'),
          })); },
          fee: create_6bBlockTxPattern(client, 'fee'),
          feeRate: createSeriesPattern19(client, 'fee_rate'),
          effectiveFeeRate: create_6bBlockTxPattern(client, 'effective_fee_rate'),
          isCpfpParent: createSeriesPattern19(client, 'is_cpfp_parent'),
          isCpfpChild: createSeriesPattern19(client, 'is_cpfp_child'),
        })); },
        get inscription() { return _lazy(this, 'inscription', () => ({
          count: createAverageBlockCumulativeSumPattern2(client, 'tx_count_inscription'),
          fees: createAverageBlockCumulativeSumPattern2(client, 'inscription_fees'),
          feeShare: createPercentPpmRatioPattern2(client, 'inscription_fee_share'),
        })); },
        get patterns() { return _lazy(this, 'patterns', () => ({
          get count() { return _lazy(this, 'count', () => ({
            coinjoin: createAverageBlockCumulativeSumPattern2(client, 'coinjoin_count'),
            consolidation: createAverageBlockCumulativeSumPattern2(client, 'consolidation_count'),
            batchPayout: createAverageBlockCumulativeSumPattern2(client, 'batch_payout_count'),
          })); },
          isCoinjoin: createSeriesPattern19(client, 'is_coinjoin'),
          isConsolidation: createSeriesPattern19(client, 'is_consolidation'),
          isBatchPayout: createSeriesPattern19(client, 'is_batch_payout'),
        })); },
        get policy() { return _lazy(this, 'policy', () => ({
          get count() { return _lazy(this, 'count', () => ({
            nonstandard: createAverageBlockCumulativeSumPattern2(client, 'nonstandard_count'),
          })); },
          isNonstandard: createSeriesPattern19(client, 'is_nonstandard'),
        })); },
        get sigops() { return _lazy(this, 'sigops', () => ({
          total: createAverageBlockCumulativeSumPattern2(client, 'total_sigop_cost'),
        })); },
        get versions() { return _lazy(this, 'versions', () => ({
          v1: createAverageBlockCumulativeSumPattern2(client, 'tx_v1'),
          v2: createAverageBlockCumulativeSumPattern2(client, 'tx_v2'),
          v3: createAverageBlockCumulativeSumPattern2(client, 'tx_v3'),
          other: createAverageBlockCumulativeSumPattern2(client, 'tx_other_version'),
        })); },
        get volume() { return _lazy(this, 'volume', () => ({
          transferVolume: createAverageBlockCumulativeSumPattern(client, 'transfer_volume_bis'),
          txPerSec: create_1m1w1y24hPattern6(client, 'tx_per_sec'),
        })); },
      })); },
      get inputs() { return _lazy(this, 'inputs', () => ({
        get raw() { return _lazy(this, 'raw', () => ({
          firstTxinIndex: createSeriesPattern18(client, 'first_txin_index'),
          outpoint: createSeriesPattern20(client, 'outpoint'),
          txoutIndex: createSeriesPattern20(client, 'txout_index'),
          txIndex: createSeriesPattern20(client, 'tx_index'),
          outputType: createSeriesPattern20(client, 'output_type'),
          typeIndex: createSeriesPattern20(client, 'type_index'),
        })); },
        value: createSeriesPattern20(client, 'value'),
        count: createCumulativeRollingSumPattern(client, 'input_count'),
        perSec: create_1m1w1y24hPattern6(client, 'inputs_per_sec'),
        get byType() { return _lazy(this, 'byType', () => ({
          get inputCount() { return _lazy(this, 'inputCount', () => ({
            all: createAverageBlockCumulativeSumPattern2(client, 'input_count_bis'),
            p2pk65: createAverageBlockCumulativeSumPattern2(client, 'p2pk65_prevout_count'),
            p2pk33: createAverageBlockCumulativeSumPattern2(client, 'p2pk33_prevout_count'),
            p2pkh: createAverageBlockCumulativeSumPattern2(client, 'p2pkh_prevout_count'),
            p2ms: createAverageBlockCumulativeSumPattern2(client, 'p2ms_prevout_count'),
            p2sh: createAverageBlockCumulativeSumPattern2(client, 'p2sh_prevout_count'),
            p2wpkh: createAverageBlockCumulativeSumPattern2(client, 'p2wpkh_prevout_count'),
            p2wsh: createAverageBlockCumulativeSumPattern2(client, 'p2wsh_prevout_count'),
            p2tr: createAverageBlockCumulativeSumPattern2(client, 'p2tr_prevout_count'),
            p2a: createAverageBlockCumulativeSumPattern2(client, 'p2a_prevout_count'),
            unknown: createAverageBlockCumulativeSumPattern2(client, 'unknown_outputs_prevout_count'),
            empty: createAverageBlockCumulativeSumPattern2(client, 'empty_outputs_prevout_count'),
          })); },
          get inputShare() { return _lazy(this, 'inputShare', () => ({
            p2pk65: create_1m1w1y24hPercentPpmRatioPattern(client, 'p2pk65_prevout_share'),
            p2pk33: create_1m1w1y24hPercentPpmRatioPattern(client, 'p2pk33_prevout_share'),
            p2pkh: create_1m1w1y24hPercentPpmRatioPattern(client, 'p2pkh_prevout_share'),
            p2ms: create_1m1w1y24hPercentPpmRatioPattern(client, 'p2ms_prevout_share'),
            p2sh: create_1m1w1y24hPercentPpmRatioPattern(client, 'p2sh_prevout_share'),
            p2wpkh: create_1m1w1y24hPercentPpmRatioPattern(client, 'p2wpkh_prevout_share'),
            p2wsh: create_1m1w1y24hPercentPpmRatioPattern(client, 'p2wsh_prevout_share'),
            p2tr: create_1m1w1y24hPercentPpmRatioPattern(client, 'p2tr_prevout_share'),
            p2a: create_1m1w1y24hPercentPpmRatioPattern(client, 'p2a_prevout_share'),
            unknown: create_1m1w1y24hPercentPpmRatioPattern(client, 'unknown_outputs_prevout_share'),
            empty: create_1m1w1y24hPercentPpmRatioPattern(client, 'empty_outputs_prevout_share'),
          })); },
          get txCount() { return _lazy(this, 'txCount', () => ({
            all: createAverageBlockCumulativeSumPattern2(client, 'non_coinbase_tx_count'),
            p2pk65: createAverageBlockCumulativeSumPattern2(client, 'tx_count_with_p2pk65_prevout'),
            p2pk33: createAverageBlockCumulativeSumPattern2(client, 'tx_count_with_p2pk33_prevout'),
            p2pkh: createAverageBlockCumulativeSumPattern2(client, 'tx_count_with_p2pkh_prevout'),
            p2ms: createAverageBlockCumulativeSumPattern2(client, 'tx_count_with_p2ms_prevout'),
            p2sh: createAverageBlockCumulativeSumPattern2(client, 'tx_count_with_p2sh_prevout'),
            p2wpkh: createAverageBlockCumulativeSumPattern2(client, 'tx_count_with_p2wpkh_prevout'),
            p2wsh: createAverageBlockCumulativeSumPattern2(client, 'tx_count_with_p2wsh_prevout'),
            p2tr: createAverageBlockCumulativeSumPattern2(client, 'tx_count_with_p2tr_prevout'),
            p2a: createAverageBlockCumulativeSumPattern2(client, 'tx_count_with_p2a_prevout'),
            unknown: createAverageBlockCumulativeSumPattern2(client, 'tx_count_with_unknown_outputs_prevout'),
            empty: createAverageBlockCumulativeSumPattern2(client, 'tx_count_with_empty_outputs_prevout'),
          })); },
          txShare: createEmptyP2aP2msP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshUnknownPattern10(client, 'tx_share_with'),
        })); },
      })); },
      get outputs() { return _lazy(this, 'outputs', () => ({
        get raw() { return _lazy(this, 'raw', () => ({
          firstTxoutIndex: createSeriesPattern18(client, 'first_txout_index'),
          value: createSeriesPattern21(client, 'value'),
          outputType: createSeriesPattern21(client, 'output_type'),
          typeIndex: createSeriesPattern21(client, 'type_index'),
        })); },
        get spent() { return _lazy(this, 'spent', () => ({
          txinIndex: createSeriesPattern21(client, 'txin_index'),
        })); },
        get count() { return _lazy(this, 'count', () => ({
          total: createCumulativeRollingSumPattern(client, 'output_count'),
        })); },
        perSec: create_1m1w1y24hPattern6(client, 'outputs_per_sec'),
        get byType() { return _lazy(this, 'byType', () => ({
          get outputCount() { return _lazy(this, 'outputCount', () => ({
            all: createAverageBlockCumulativeSumPattern2(client, 'output_count_bis'),
            p2pk65: createAverageBlockCumulativeSumPattern2(client, 'p2pk65_output_count'),
            p2pk33: createAverageBlockCumulativeSumPattern2(client, 'p2pk33_output_count'),
            p2pkh: createAverageBlockCumulativeSumPattern2(client, 'p2pkh_output_count'),
            p2ms: createAverageBlockCumulativeSumPattern2(client, 'p2ms_output_count'),
            p2sh: createAverageBlockCumulativeSumPattern2(client, 'p2sh_output_count'),
            p2wpkh: createAverageBlockCumulativeSumPattern2(client, 'p2wpkh_output_count'),
            p2wsh: createAverageBlockCumulativeSumPattern2(client, 'p2wsh_output_count'),
            p2tr: createAverageBlockCumulativeSumPattern2(client, 'p2tr_output_count'),
            p2a: createAverageBlockCumulativeSumPattern2(client, 'p2a_output_count'),
            unknown: createAverageBlockCumulativeSumPattern2(client, 'unknown_outputs_output_count'),
            empty: createAverageBlockCumulativeSumPattern2(client, 'empty_outputs_output_count'),
            opReturn: createAverageBlockCumulativeSumPattern2(client, 'op_return_output_count'),
          })); },
          spendableOutputCount: createAverageBlockCumulativeSumPattern2(client, 'spendable_output_count'),
          get outputShare() { return _lazy(this, 'outputShare', () => ({
            p2pk65: create_1m1w1y24hPercentPpmRatioPattern(client, 'p2pk65_output_share'),
            p2pk33: create_1m1w1y24hPercentPpmRatioPattern(client, 'p2pk33_output_share'),
            p2pkh: create_1m1w1y24hPercentPpmRatioPattern(client, 'p2pkh_output_share'),
            p2ms: create_1m1w1y24hPercentPpmRatioPattern(client, 'p2ms_output_share'),
            p2sh: create_1m1w1y24hPercentPpmRatioPattern(client, 'p2sh_output_share'),
            p2wpkh: create_1m1w1y24hPercentPpmRatioPattern(client, 'p2wpkh_output_share'),
            p2wsh: create_1m1w1y24hPercentPpmRatioPattern(client, 'p2wsh_output_share'),
            p2tr: create_1m1w1y24hPercentPpmRatioPattern(client, 'p2tr_output_share'),
            p2a: create_1m1w1y24hPercentPpmRatioPattern(client, 'p2a_output_share'),
            unknown: create_1m1w1y24hPercentPpmRatioPattern(client, 'unknown_outputs_output_share'),
            empty: create_1m1w1y24hPercentPpmRatioPattern(client, 'empty_outputs_output_share'),
            opReturn: create_1m1w1y24hPercentPpmRatioPattern(client, 'op_return_output_share'),
          })); },
          txCount: createAllEmptyOpP2aP2msP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshUnknownPattern(client, 'tx_count'),
          txShare: createEmptyOpP2aP2msP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshUnknownPattern2(client, 'tx_share_with'),
        })); },
        get value() { return _lazy(this, 'value', () => ({
          opReturn: createBlockCumulativePattern(client, 'op_return_value'),
        })); },
      })); },
      get addrs() { return _lazy(this, 'addrs', () => ({
        get raw() { return _lazy(this, 'raw', () => ({
          get p2pk65() { return _lazy(this, 'p2pk65', () => ({
            firstIndex: createSeriesPattern18(client, 'first_p2pk65_addr_index'),
            bytes: createSeriesPattern27(client, 'p2pk65_bytes'),
          })); },
          get p2pk33() { return _lazy(this, 'p2pk33', () => ({
            firstIndex: createSeriesPattern18(client, 'first_p2pk33_addr_index'),
            bytes: createSeriesPattern26(client, 'p2pk33_bytes'),
          })); },
          get p2pkh() { return _lazy(this, 'p2pkh', () => ({
            firstIndex: createSeriesPattern18(client, 'first_p2pkh_addr_index'),
            bytes: createSeriesPattern28(client, 'p2pkh_bytes'),
          })); },
          get p2sh() { return _lazy(this, 'p2sh', () => ({
            firstIndex: createSeriesPattern18(client, 'first_p2sh_addr_index'),
            bytes: createSeriesPattern29(client, 'p2sh_bytes'),
          })); },
          get p2wpkh() { return _lazy(this, 'p2wpkh', () => ({
            firstIndex: createSeriesPattern18(client, 'first_p2wpkh_addr_index'),
            bytes: createSeriesPattern31(client, 'p2wpkh_bytes'),
          })); },
          get p2wsh() { return _lazy(this, 'p2wsh', () => ({
            firstIndex: createSeriesPattern18(client, 'first_p2wsh_addr_index'),
            bytes: createSeriesPattern32(client, 'p2wsh_bytes'),
          })); },
          get p2tr() { return _lazy(this, 'p2tr', () => ({
            firstIndex: createSeriesPattern18(client, 'first_p2tr_addr_index'),
            bytes: createSeriesPattern30(client, 'p2tr_bytes'),
          })); },
          get p2a() { return _lazy(this, 'p2a', () => ({
            firstIndex: createSeriesPattern18(client, 'first_p2a_addr_index'),
            bytes: createSeriesPattern24(client, 'p2a_bytes'),
          })); },
        })); },
        get state() { return _lazy(this, 'state', () => ({
          p2a: createSeriesPattern24(client, 'addr_state'),
          p2pk33: createSeriesPattern26(client, 'addr_state'),
          p2pk65: createSeriesPattern27(client, 'addr_state'),
          p2pkh: createSeriesPattern28(client, 'addr_state'),
          p2sh: createSeriesPattern29(client, 'addr_state'),
          p2tr: createSeriesPattern30(client, 'addr_state'),
          p2wpkh: createSeriesPattern31(client, 'addr_state'),
          p2wsh: createSeriesPattern32(client, 'addr_state'),
          funded: createSeriesPattern34(client, 'funded_addr_data'),
          extendedEmpty: createSeriesPattern35(client, 'extended_empty_addr_data'),
        })); },
        get byBalance() { return _lazy(this, 'byBalance', () => ({
          get supply() { return _lazy(this, 'supply', () => ({
            _0sats: createDeltaDominanceTotalPattern(client, 'addrs_0sats_supply'),
            _1satTo10sats: createDeltaDominanceTotalPattern(client, 'addrs_1sat_to_10sats_supply'),
            _10satsTo100sats: createDeltaDominanceTotalPattern(client, 'addrs_10sats_to_100sats_supply'),
            _100satsTo1kSats: createDeltaDominanceTotalPattern(client, 'addrs_100sats_to_1k_sats_supply'),
            _1kSatsTo10kSats: createDeltaDominanceTotalPattern(client, 'addrs_1k_sats_to_10k_sats_supply'),
            _10kSatsTo100kSats: createDeltaDominanceTotalPattern(client, 'addrs_10k_sats_to_100k_sats_supply'),
            _100kSatsTo1mSats: createDeltaDominanceTotalPattern(client, 'addrs_100k_sats_to_1m_sats_supply'),
            _1mSatsTo10mSats: createDeltaDominanceTotalPattern(client, 'addrs_1m_sats_to_10m_sats_supply'),
            _10mSatsTo1btc: createDeltaDominanceTotalPattern(client, 'addrs_10m_sats_to_1btc_supply'),
            _1btcTo10btc: createDeltaDominanceTotalPattern(client, 'addrs_1btc_to_10btc_supply'),
            _10btcTo100btc: createDeltaDominanceTotalPattern(client, 'addrs_10btc_to_100btc_supply'),
            _100btcTo1kBtc: createDeltaDominanceTotalPattern(client, 'addrs_100btc_to_1k_btc_supply'),
            _1kBtcTo10kBtc: createDeltaDominanceTotalPattern(client, 'addrs_1k_btc_to_10k_btc_supply'),
            _10kBtcTo100kBtc: createDeltaDominanceTotalPattern(client, 'addrs_10k_btc_to_100k_btc_supply'),
            over100kBtc: createDeltaDominanceTotalPattern(client, 'addrs_over_100k_btc_supply'),
          })); },
          get utxoCount() { return _lazy(this, 'utxoCount', () => ({
            _0sats: createBaseDeltaPattern(client, 'addrs_0sats_utxo_count'),
            _1satTo10sats: createBaseDeltaPattern(client, 'addrs_1sat_to_10sats_utxo_count'),
            _10satsTo100sats: createBaseDeltaPattern(client, 'addrs_10sats_to_100sats_utxo_count'),
            _100satsTo1kSats: createBaseDeltaPattern(client, 'addrs_100sats_to_1k_sats_utxo_count'),
            _1kSatsTo10kSats: createBaseDeltaPattern(client, 'addrs_1k_sats_to_10k_sats_utxo_count'),
            _10kSatsTo100kSats: createBaseDeltaPattern(client, 'addrs_10k_sats_to_100k_sats_utxo_count'),
            _100kSatsTo1mSats: createBaseDeltaPattern(client, 'addrs_100k_sats_to_1m_sats_utxo_count'),
            _1mSatsTo10mSats: createBaseDeltaPattern(client, 'addrs_1m_sats_to_10m_sats_utxo_count'),
            _10mSatsTo1btc: createBaseDeltaPattern(client, 'addrs_10m_sats_to_1btc_utxo_count'),
            _1btcTo10btc: createBaseDeltaPattern(client, 'addrs_1btc_to_10btc_utxo_count'),
            _10btcTo100btc: createBaseDeltaPattern(client, 'addrs_10btc_to_100btc_utxo_count'),
            _100btcTo1kBtc: createBaseDeltaPattern(client, 'addrs_100btc_to_1k_btc_utxo_count'),
            _1kBtcTo10kBtc: createBaseDeltaPattern(client, 'addrs_1k_btc_to_10k_btc_utxo_count'),
            _10kBtcTo100kBtc: createBaseDeltaPattern(client, 'addrs_10k_btc_to_100k_btc_utxo_count'),
            over100kBtc: createBaseDeltaPattern(client, 'addrs_over_100k_btc_utxo_count'),
          })); },
          transferVolume: create_0sats100btc100k100sats10btc10k10m10sats1btc1k1m1satOverPattern4(client, 'addrs'),
          realizedCap: create_0sats100btc100k100sats10btc10k10m10sats1btc1k1m1satOverPattern(client, 'addrs'),
          get realizedProfit() { return _lazy(this, 'realizedProfit', () => ({
            _0sats: createBlockCumulativeSumPattern(client, 'addrs_0sats_realized_profit'),
            _1satTo10sats: createBlockCumulativeSumPattern(client, 'addrs_1sat_to_10sats_realized_profit'),
            _10satsTo100sats: createBlockCumulativeSumPattern(client, 'addrs_10sats_to_100sats_realized_profit'),
            _100satsTo1kSats: createBlockCumulativeSumPattern(client, 'addrs_100sats_to_1k_sats_realized_profit'),
            _1kSatsTo10kSats: createBlockCumulativeSumPattern(client, 'addrs_1k_sats_to_10k_sats_realized_profit'),
            _10kSatsTo100kSats: createBlockCumulativeSumPattern(client, 'addrs_10k_sats_to_100k_sats_realized_profit'),
            _100kSatsTo1mSats: createBlockCumulativeSumPattern(client, 'addrs_100k_sats_to_1m_sats_realized_profit'),
            _1mSatsTo10mSats: createBlockCumulativeSumPattern(client, 'addrs_1m_sats_to_10m_sats_realized_profit'),
            _10mSatsTo1btc: createBlockCumulativeSumPattern(client, 'addrs_10m_sats_to_1btc_realized_profit'),
            _1btcTo10btc: createBlockCumulativeSumPattern(client, 'addrs_1btc_to_10btc_realized_profit'),
            _10btcTo100btc: createBlockCumulativeSumPattern(client, 'addrs_10btc_to_100btc_realized_profit'),
            _100btcTo1kBtc: createBlockCumulativeSumPattern(client, 'addrs_100btc_to_1k_btc_realized_profit'),
            _1kBtcTo10kBtc: createBlockCumulativeSumPattern(client, 'addrs_1k_btc_to_10k_btc_realized_profit'),
            _10kBtcTo100kBtc: createBlockCumulativeSumPattern(client, 'addrs_10k_btc_to_100k_btc_realized_profit'),
            over100kBtc: createBlockCumulativeSumPattern(client, 'addrs_over_100k_btc_realized_profit'),
          })); },
          get realizedLoss() { return _lazy(this, 'realizedLoss', () => ({
            _0sats: createBlockCumulativeSumPattern(client, 'addrs_0sats_realized_loss'),
            _1satTo10sats: createBlockCumulativeSumPattern(client, 'addrs_1sat_to_10sats_realized_loss'),
            _10satsTo100sats: createBlockCumulativeSumPattern(client, 'addrs_10sats_to_100sats_realized_loss'),
            _100satsTo1kSats: createBlockCumulativeSumPattern(client, 'addrs_100sats_to_1k_sats_realized_loss'),
            _1kSatsTo10kSats: createBlockCumulativeSumPattern(client, 'addrs_1k_sats_to_10k_sats_realized_loss'),
            _10kSatsTo100kSats: createBlockCumulativeSumPattern(client, 'addrs_10k_sats_to_100k_sats_realized_loss'),
            _100kSatsTo1mSats: createBlockCumulativeSumPattern(client, 'addrs_100k_sats_to_1m_sats_realized_loss'),
            _1mSatsTo10mSats: createBlockCumulativeSumPattern(client, 'addrs_1m_sats_to_10m_sats_realized_loss'),
            _10mSatsTo1btc: createBlockCumulativeSumPattern(client, 'addrs_10m_sats_to_1btc_realized_loss'),
            _1btcTo10btc: createBlockCumulativeSumPattern(client, 'addrs_1btc_to_10btc_realized_loss'),
            _10btcTo100btc: createBlockCumulativeSumPattern(client, 'addrs_10btc_to_100btc_realized_loss'),
            _100btcTo1kBtc: createBlockCumulativeSumPattern(client, 'addrs_100btc_to_1k_btc_realized_loss'),
            _1kBtcTo10kBtc: createBlockCumulativeSumPattern(client, 'addrs_1k_btc_to_10k_btc_realized_loss'),
            _10kBtcTo100kBtc: createBlockCumulativeSumPattern(client, 'addrs_10k_btc_to_100k_btc_realized_loss'),
            over100kBtc: createBlockCumulativeSumPattern(client, 'addrs_over_100k_btc_realized_loss'),
          })); },
        })); },
        get funded() { return _lazy(this, 'funded', () => ({
          all: createSeriesPattern1(client, 'addr_count'),
          p2pk65: createSeriesPattern1(client, 'p2pk65_addr_count'),
          p2pk33: createSeriesPattern1(client, 'p2pk33_addr_count'),
          p2pkh: createSeriesPattern1(client, 'p2pkh_addr_count'),
          p2sh: createSeriesPattern1(client, 'p2sh_addr_count'),
          p2wpkh: createSeriesPattern1(client, 'p2wpkh_addr_count'),
          p2wsh: createSeriesPattern1(client, 'p2wsh_addr_count'),
          p2tr: createSeriesPattern1(client, 'p2tr_addr_count'),
          p2a: createSeriesPattern1(client, 'p2a_addr_count'),
          get balance() { return _lazy(this, 'balance', () => ({
            _0sats: createBaseDeltaPattern(client, 'addrs_0sats_addr_count'),
            _1satTo10sats: createBaseDeltaPattern(client, 'addrs_1sat_to_10sats_addr_count'),
            _10satsTo100sats: createBaseDeltaPattern(client, 'addrs_10sats_to_100sats_addr_count'),
            _100satsTo1kSats: createBaseDeltaPattern(client, 'addrs_100sats_to_1k_sats_addr_count'),
            _1kSatsTo10kSats: createBaseDeltaPattern(client, 'addrs_1k_sats_to_10k_sats_addr_count'),
            _10kSatsTo100kSats: createBaseDeltaPattern(client, 'addrs_10k_sats_to_100k_sats_addr_count'),
            _100kSatsTo1mSats: createBaseDeltaPattern(client, 'addrs_100k_sats_to_1m_sats_addr_count'),
            _1mSatsTo10mSats: createBaseDeltaPattern(client, 'addrs_1m_sats_to_10m_sats_addr_count'),
            _10mSatsTo1btc: createBaseDeltaPattern(client, 'addrs_10m_sats_to_1btc_addr_count'),
            _1btcTo10btc: createBaseDeltaPattern(client, 'addrs_1btc_to_10btc_addr_count'),
            _10btcTo100btc: createBaseDeltaPattern(client, 'addrs_10btc_to_100btc_addr_count'),
            _100btcTo1kBtc: createBaseDeltaPattern(client, 'addrs_100btc_to_1k_btc_addr_count'),
            _1kBtcTo10kBtc: createBaseDeltaPattern(client, 'addrs_1k_btc_to_10k_btc_addr_count'),
            _10kBtcTo100kBtc: createBaseDeltaPattern(client, 'addrs_10k_btc_to_100k_btc_addr_count'),
            over100kBtc: createBaseDeltaPattern(client, 'addrs_over_100k_btc_addr_count'),
          })); },
        })); },
        empty: createAllP2aP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshPattern4(client, 'empty_addr_count'),
        get activity() { return _lazy(this, 'activity', () => ({
          reactivated: createAllP2aP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshPattern(client, 'reactivated_addrs'),
          sending: createAllP2aP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshPattern(client, 'sending_addrs'),
          receiving: createAllP2aP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshPattern(client, 'receiving_addrs'),
          bidirectional: createAllP2aP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshPattern(client, 'bidirectional_addrs'),
          active: createAllP2aP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshPattern(client, 'active_addrs'),
        })); },
        total: createAllP2aP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshPattern4(client, 'total_addr_count'),
        new: createAllP2aP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshPattern6(client, 'new_addr_count'),
        get reused() { return _lazy(this, 'reused', () => ({
          count: createFundedTotalPattern(client, 'reused_addr_count'),
          get events() { return _lazy(this, 'events', () => ({
            outputToReusedAddrCount: createAllP2aP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshPattern6(client, 'output_to_reused_addr_count'),
            outputToReusedAddrShare: createAllP2aP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshPattern7(client, 'output_to_reused_addr_share'),
            spendableOutputToReusedAddrShare: create_1m1w1y24hPercentPpmRatioPattern(client, 'spendable_output_to_reused_addr_share'),
            inputFromReusedAddrCount: createAllP2aP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshPattern6(client, 'input_from_reused_addr_count'),
            inputFromReusedAddrShare: createAllP2aP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshPattern7(client, 'input_from_reused_addr_share'),
            activeReusedAddrCount: create_1m1w1y24hBlockPattern(client, 'active_reused_addr_count'),
            activeReusedAddrShare: create_1m1w1y24hBlockPattern2(client, 'active_reused_addr_share'),
          })); },
          get supply() { return _lazy(this, 'supply', () => ({
            all: createBtcCentsSatsUsdPattern(client, 'reused_addr_supply'),
            p2pk65: createBtcCentsSatsUsdPattern(client, 'p2pk65_reused_addr_supply'),
            p2pk33: createBtcCentsSatsUsdPattern(client, 'p2pk33_reused_addr_supply'),
            p2pkh: createBtcCentsSatsUsdPattern(client, 'p2pkh_reused_addr_supply'),
            p2sh: createBtcCentsSatsUsdPattern(client, 'p2sh_reused_addr_supply'),
            p2wpkh: createBtcCentsSatsUsdPattern(client, 'p2wpkh_reused_addr_supply'),
            p2wsh: createBtcCentsSatsUsdPattern(client, 'p2wsh_reused_addr_supply'),
            p2tr: createBtcCentsSatsUsdPattern(client, 'p2tr_reused_addr_supply'),
            p2a: createBtcCentsSatsUsdPattern(client, 'p2a_reused_addr_supply'),
            share: createAllP2aP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshPattern5(client, 'reused_addr_supply_share'),
          })); },
        })); },
        get respent() { return _lazy(this, 'respent', () => ({
          count: createFundedTotalPattern(client, 'respent_addr_count'),
          get events() { return _lazy(this, 'events', () => ({
            outputToReusedAddrCount: createAllP2aP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshPattern6(client, 'output_to_respent_addr_count'),
            outputToReusedAddrShare: createAllP2aP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshPattern7(client, 'output_to_respent_addr_share'),
            spendableOutputToReusedAddrShare: create_1m1w1y24hPercentPpmRatioPattern(client, 'spendable_output_to_respent_addr_share'),
            inputFromReusedAddrCount: createAllP2aP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshPattern6(client, 'input_from_respent_addr_count'),
            inputFromReusedAddrShare: createAllP2aP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshPattern7(client, 'input_from_respent_addr_share'),
            activeReusedAddrCount: create_1m1w1y24hBlockPattern(client, 'active_respent_addr_count'),
            activeReusedAddrShare: create_1m1w1y24hBlockPattern2(client, 'active_respent_addr_share'),
          })); },
          get supply() { return _lazy(this, 'supply', () => ({
            all: createBtcCentsSatsUsdPattern(client, 'respent_addr_supply'),
            p2pk65: createBtcCentsSatsUsdPattern(client, 'p2pk65_respent_addr_supply'),
            p2pk33: createBtcCentsSatsUsdPattern(client, 'p2pk33_respent_addr_supply'),
            p2pkh: createBtcCentsSatsUsdPattern(client, 'p2pkh_respent_addr_supply'),
            p2sh: createBtcCentsSatsUsdPattern(client, 'p2sh_respent_addr_supply'),
            p2wpkh: createBtcCentsSatsUsdPattern(client, 'p2wpkh_respent_addr_supply'),
            p2wsh: createBtcCentsSatsUsdPattern(client, 'p2wsh_respent_addr_supply'),
            p2tr: createBtcCentsSatsUsdPattern(client, 'p2tr_respent_addr_supply'),
            p2a: createBtcCentsSatsUsdPattern(client, 'p2a_respent_addr_supply'),
            share: createAllP2aP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshPattern5(client, 'respent_addr_supply_share'),
          })); },
        })); },
        get exposed() { return _lazy(this, 'exposed', () => ({
          count: createFundedTotalPattern(client, 'exposed_addr_count'),
          get supply() { return _lazy(this, 'supply', () => ({
            all: createBtcCentsSatsUsdPattern(client, 'exposed_addr_supply'),
            p2pk65: createBtcCentsSatsUsdPattern(client, 'p2pk65_exposed_addr_supply'),
            p2pk33: createBtcCentsSatsUsdPattern(client, 'p2pk33_exposed_addr_supply'),
            p2pkh: createBtcCentsSatsUsdPattern(client, 'p2pkh_exposed_addr_supply'),
            p2sh: createBtcCentsSatsUsdPattern(client, 'p2sh_exposed_addr_supply'),
            p2wpkh: createBtcCentsSatsUsdPattern(client, 'p2wpkh_exposed_addr_supply'),
            p2wsh: createBtcCentsSatsUsdPattern(client, 'p2wsh_exposed_addr_supply'),
            p2tr: createBtcCentsSatsUsdPattern(client, 'p2tr_exposed_addr_supply'),
            p2a: createBtcCentsSatsUsdPattern(client, 'p2a_exposed_addr_supply'),
            share: createAllP2aP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshPattern5(client, 'exposed_addr_supply_share'),
          })); },
        })); },
        get delta() { return _lazy(this, 'delta', () => ({
          all: createAbsoluteRatePattern2(client, 'addr_count'),
          p2pk65: createAbsoluteRatePattern2(client, 'p2pk65_addr_count'),
          p2pk33: createAbsoluteRatePattern2(client, 'p2pk33_addr_count'),
          p2pkh: createAbsoluteRatePattern2(client, 'p2pkh_addr_count'),
          p2sh: createAbsoluteRatePattern2(client, 'p2sh_addr_count'),
          p2wpkh: createAbsoluteRatePattern2(client, 'p2wpkh_addr_count'),
          p2wsh: createAbsoluteRatePattern2(client, 'p2wsh_addr_count'),
          p2tr: createAbsoluteRatePattern2(client, 'p2tr_addr_count'),
          p2a: createAbsoluteRatePattern2(client, 'p2a_addr_count'),
        })); },
        get avgBalance() { return _lazy(this, 'avgBalance', () => ({
          all: createBtcCentsSatsUsdPattern(client, 'avg_addr_amount'),
          p2pk65: createBtcCentsSatsUsdPattern(client, 'p2pk65_avg_addr_amount'),
          p2pk33: createBtcCentsSatsUsdPattern(client, 'p2pk33_avg_addr_amount'),
          p2pkh: createBtcCentsSatsUsdPattern(client, 'p2pkh_avg_addr_amount'),
          p2sh: createBtcCentsSatsUsdPattern(client, 'p2sh_avg_addr_amount'),
          p2wpkh: createBtcCentsSatsUsdPattern(client, 'p2wpkh_avg_addr_amount'),
          p2wsh: createBtcCentsSatsUsdPattern(client, 'p2wsh_avg_addr_amount'),
          p2tr: createBtcCentsSatsUsdPattern(client, 'p2tr_avg_addr_amount'),
          p2a: createBtcCentsSatsUsdPattern(client, 'p2a_avg_addr_amount'),
        })); },
      })); },
      get scripts() { return _lazy(this, 'scripts', () => ({
        get raw() { return _lazy(this, 'raw', () => ({
          get empty() { return _lazy(this, 'empty', () => ({
            firstIndex: createSeriesPattern18(client, 'first_empty_output_index'),
            toTxIndex: createSeriesPattern22(client, 'tx_index'),
          })); },
          get p2ms() { return _lazy(this, 'p2ms', () => ({
            firstIndex: createSeriesPattern18(client, 'first_p2ms_output_index'),
            toTxIndex: createSeriesPattern25(client, 'tx_index'),
            legacySigops: createSeriesPattern25(client, 'p2ms_legacy_sigops'),
          })); },
          get unknown() { return _lazy(this, 'unknown', () => ({
            firstIndex: createSeriesPattern18(client, 'first_unknown_output_index'),
            toTxIndex: createSeriesPattern33(client, 'tx_index'),
            legacySigops: createSeriesPattern33(client, 'unknown_legacy_sigops'),
          })); },
        })); },
      })); },
      get opReturn() { return _lazy(this, 'opReturn', () => ({
        get raw() { return _lazy(this, 'raw', () => ({
          firstIndex: createSeriesPattern18(client, 'first_op_return_index'),
          toTxIndex: createSeriesPattern23(client, 'tx_index'),
          kind: createSeriesPattern23(client, 'kind'),
          postOpReturnBytes: createSeriesPattern23(client, 'op_return_post_op_return_bytes'),
        })); },
        get total() { return _lazy(this, 'total', () => ({
          dataBytes: createAverageBlockCumulativeSumPattern2(client, 'op_return_data_bytes'),
          txCount: createAverageBlockCumulativeSumPattern2(client, 'op_return_tx_count'),
          txVsize: createAverageBlockCumulativeSumPattern2(client, 'op_return_tx_vsize'),
          fees: createAverageBlockCumulativeSumPattern2(client, 'op_return_fees'),
          chainShare: createPercentPpmRatioPattern2(client, 'op_return_chain_share'),
          feeShare: create_1m1w1y24hPercentPpmRatioPattern(client, 'op_return_fee_share'),
        })); },
        get byKind() { return _lazy(this, 'byKind', () => ({
          get outputCount() { return _lazy(this, 'outputCount', () => ({
            runes: createAverageBlockCumulativeSumPattern2(client, 'op_return_runes_output_count'),
            veriBlock: createAverageBlockCumulativeSumPattern2(client, 'op_return_veri_block_output_count'),
            omni: createAverageBlockCumulativeSumPattern2(client, 'op_return_omni_output_count'),
            stacks: createAverageBlockCumulativeSumPattern2(client, 'op_return_stacks_output_count'),
            blockstack: createAverageBlockCumulativeSumPattern2(client, 'op_return_blockstack_output_count'),
            colu: createAverageBlockCumulativeSumPattern2(client, 'op_return_colu_output_count'),
            openAssets: createAverageBlockCumulativeSumPattern2(client, 'op_return_open_assets_output_count'),
            komodo: createAverageBlockCumulativeSumPattern2(client, 'op_return_komodo_output_count'),
            coinSpark: createAverageBlockCumulativeSumPattern2(client, 'op_return_coin_spark_output_count'),
            poet: createAverageBlockCumulativeSumPattern2(client, 'op_return_poet_output_count'),
            docproof: createAverageBlockCumulativeSumPattern2(client, 'op_return_docproof_output_count'),
            openTimestamps: createAverageBlockCumulativeSumPattern2(client, 'op_return_open_timestamps_output_count'),
            factom: createAverageBlockCumulativeSumPattern2(client, 'op_return_factom_output_count'),
            eternityWall: createAverageBlockCumulativeSumPattern2(client, 'op_return_eternity_wall_output_count'),
            memo: createAverageBlockCumulativeSumPattern2(client, 'op_return_memo_output_count'),
            bitproof: createAverageBlockCumulativeSumPattern2(client, 'op_return_bitproof_output_count'),
            ascribe: createAverageBlockCumulativeSumPattern2(client, 'op_return_ascribe_output_count'),
            stampery: createAverageBlockCumulativeSumPattern2(client, 'op_return_stampery_output_count'),
            epobc: createAverageBlockCumulativeSumPattern2(client, 'op_return_epobc_output_count'),
            bareHash: createAverageBlockCumulativeSumPattern2(client, 'op_return_bare_hash_output_count'),
            text: createAverageBlockCumulativeSumPattern2(client, 'op_return_text_output_count'),
            empty: createAverageBlockCumulativeSumPattern2(client, 'op_return_empty_output_count'),
            unknown: createAverageBlockCumulativeSumPattern2(client, 'op_return_unknown_output_count'),
          })); },
          get dataBytes() { return _lazy(this, 'dataBytes', () => ({
            runes: createAverageBlockChainCumulativeDataSumPattern(client, 'op_return_runes'),
            veriBlock: createAverageBlockChainCumulativeDataSumPattern(client, 'op_return_veri_block'),
            omni: createAverageBlockChainCumulativeDataSumPattern(client, 'op_return_omni'),
            stacks: createAverageBlockChainCumulativeDataSumPattern(client, 'op_return_stacks'),
            blockstack: createAverageBlockChainCumulativeDataSumPattern(client, 'op_return_blockstack'),
            colu: createAverageBlockChainCumulativeDataSumPattern(client, 'op_return_colu'),
            openAssets: createAverageBlockChainCumulativeDataSumPattern(client, 'op_return_open_assets'),
            komodo: createAverageBlockChainCumulativeDataSumPattern(client, 'op_return_komodo'),
            coinSpark: createAverageBlockChainCumulativeDataSumPattern(client, 'op_return_coin_spark'),
            poet: createAverageBlockChainCumulativeDataSumPattern(client, 'op_return_poet'),
            docproof: createAverageBlockChainCumulativeDataSumPattern(client, 'op_return_docproof'),
            openTimestamps: createAverageBlockChainCumulativeDataSumPattern(client, 'op_return_open_timestamps'),
            factom: createAverageBlockChainCumulativeDataSumPattern(client, 'op_return_factom'),
            eternityWall: createAverageBlockChainCumulativeDataSumPattern(client, 'op_return_eternity_wall'),
            memo: createAverageBlockChainCumulativeDataSumPattern(client, 'op_return_memo'),
            bitproof: createAverageBlockChainCumulativeDataSumPattern(client, 'op_return_bitproof'),
            ascribe: createAverageBlockChainCumulativeDataSumPattern(client, 'op_return_ascribe'),
            stampery: createAverageBlockChainCumulativeDataSumPattern(client, 'op_return_stampery'),
            epobc: createAverageBlockChainCumulativeDataSumPattern(client, 'op_return_epobc'),
            bareHash: createAverageBlockChainCumulativeDataSumPattern(client, 'op_return_bare_hash'),
            text: createAverageBlockChainCumulativeDataSumPattern(client, 'op_return_text'),
            empty: createAverageBlockChainCumulativeDataSumPattern(client, 'op_return_empty'),
            unknown: createAverageBlockChainCumulativeDataSumPattern(client, 'op_return_unknown'),
          })); },
          get txCount() { return _lazy(this, 'txCount', () => ({
            runes: createAverageBlockCumulativeSumPattern2(client, 'op_return_runes_tx_count'),
            veriBlock: createAverageBlockCumulativeSumPattern2(client, 'op_return_veri_block_tx_count'),
            omni: createAverageBlockCumulativeSumPattern2(client, 'op_return_omni_tx_count'),
            stacks: createAverageBlockCumulativeSumPattern2(client, 'op_return_stacks_tx_count'),
            blockstack: createAverageBlockCumulativeSumPattern2(client, 'op_return_blockstack_tx_count'),
            colu: createAverageBlockCumulativeSumPattern2(client, 'op_return_colu_tx_count'),
            openAssets: createAverageBlockCumulativeSumPattern2(client, 'op_return_open_assets_tx_count'),
            komodo: createAverageBlockCumulativeSumPattern2(client, 'op_return_komodo_tx_count'),
            coinSpark: createAverageBlockCumulativeSumPattern2(client, 'op_return_coin_spark_tx_count'),
            poet: createAverageBlockCumulativeSumPattern2(client, 'op_return_poet_tx_count'),
            docproof: createAverageBlockCumulativeSumPattern2(client, 'op_return_docproof_tx_count'),
            openTimestamps: createAverageBlockCumulativeSumPattern2(client, 'op_return_open_timestamps_tx_count'),
            factom: createAverageBlockCumulativeSumPattern2(client, 'op_return_factom_tx_count'),
            eternityWall: createAverageBlockCumulativeSumPattern2(client, 'op_return_eternity_wall_tx_count'),
            memo: createAverageBlockCumulativeSumPattern2(client, 'op_return_memo_tx_count'),
            bitproof: createAverageBlockCumulativeSumPattern2(client, 'op_return_bitproof_tx_count'),
            ascribe: createAverageBlockCumulativeSumPattern2(client, 'op_return_ascribe_tx_count'),
            stampery: createAverageBlockCumulativeSumPattern2(client, 'op_return_stampery_tx_count'),
            epobc: createAverageBlockCumulativeSumPattern2(client, 'op_return_epobc_tx_count'),
            bareHash: createAverageBlockCumulativeSumPattern2(client, 'op_return_bare_hash_tx_count'),
            text: createAverageBlockCumulativeSumPattern2(client, 'op_return_text_tx_count'),
            empty: createAverageBlockCumulativeSumPattern2(client, 'op_return_empty_tx_count'),
            unknown: createAverageBlockCumulativeSumPattern2(client, 'op_return_unknown_tx_count'),
          })); },
          get txVsize() { return _lazy(this, 'txVsize', () => ({
            runes: createAverageBlockCumulativeSumPattern2(client, 'op_return_runes_tx_vsize'),
            veriBlock: createAverageBlockCumulativeSumPattern2(client, 'op_return_veri_block_tx_vsize'),
            omni: createAverageBlockCumulativeSumPattern2(client, 'op_return_omni_tx_vsize'),
            stacks: createAverageBlockCumulativeSumPattern2(client, 'op_return_stacks_tx_vsize'),
            blockstack: createAverageBlockCumulativeSumPattern2(client, 'op_return_blockstack_tx_vsize'),
            colu: createAverageBlockCumulativeSumPattern2(client, 'op_return_colu_tx_vsize'),
            openAssets: createAverageBlockCumulativeSumPattern2(client, 'op_return_open_assets_tx_vsize'),
            komodo: createAverageBlockCumulativeSumPattern2(client, 'op_return_komodo_tx_vsize'),
            coinSpark: createAverageBlockCumulativeSumPattern2(client, 'op_return_coin_spark_tx_vsize'),
            poet: createAverageBlockCumulativeSumPattern2(client, 'op_return_poet_tx_vsize'),
            docproof: createAverageBlockCumulativeSumPattern2(client, 'op_return_docproof_tx_vsize'),
            openTimestamps: createAverageBlockCumulativeSumPattern2(client, 'op_return_open_timestamps_tx_vsize'),
            factom: createAverageBlockCumulativeSumPattern2(client, 'op_return_factom_tx_vsize'),
            eternityWall: createAverageBlockCumulativeSumPattern2(client, 'op_return_eternity_wall_tx_vsize'),
            memo: createAverageBlockCumulativeSumPattern2(client, 'op_return_memo_tx_vsize'),
            bitproof: createAverageBlockCumulativeSumPattern2(client, 'op_return_bitproof_tx_vsize'),
            ascribe: createAverageBlockCumulativeSumPattern2(client, 'op_return_ascribe_tx_vsize'),
            stampery: createAverageBlockCumulativeSumPattern2(client, 'op_return_stampery_tx_vsize'),
            epobc: createAverageBlockCumulativeSumPattern2(client, 'op_return_epobc_tx_vsize'),
            bareHash: createAverageBlockCumulativeSumPattern2(client, 'op_return_bare_hash_tx_vsize'),
            text: createAverageBlockCumulativeSumPattern2(client, 'op_return_text_tx_vsize'),
            empty: createAverageBlockCumulativeSumPattern2(client, 'op_return_empty_tx_vsize'),
            unknown: createAverageBlockCumulativeSumPattern2(client, 'op_return_unknown_tx_vsize'),
          })); },
          get fees() { return _lazy(this, 'fees', () => ({
            runes: createAverageBlockCumulativeFeeSumPattern(client, 'op_return_runes'),
            veriBlock: createAverageBlockCumulativeFeeSumPattern(client, 'op_return_veri_block'),
            omni: createAverageBlockCumulativeFeeSumPattern(client, 'op_return_omni'),
            stacks: createAverageBlockCumulativeFeeSumPattern(client, 'op_return_stacks'),
            blockstack: createAverageBlockCumulativeFeeSumPattern(client, 'op_return_blockstack'),
            colu: createAverageBlockCumulativeFeeSumPattern(client, 'op_return_colu'),
            openAssets: createAverageBlockCumulativeFeeSumPattern(client, 'op_return_open_assets'),
            komodo: createAverageBlockCumulativeFeeSumPattern(client, 'op_return_komodo'),
            coinSpark: createAverageBlockCumulativeFeeSumPattern(client, 'op_return_coin_spark'),
            poet: createAverageBlockCumulativeFeeSumPattern(client, 'op_return_poet'),
            docproof: createAverageBlockCumulativeFeeSumPattern(client, 'op_return_docproof'),
            openTimestamps: createAverageBlockCumulativeFeeSumPattern(client, 'op_return_open_timestamps'),
            factom: createAverageBlockCumulativeFeeSumPattern(client, 'op_return_factom'),
            eternityWall: createAverageBlockCumulativeFeeSumPattern(client, 'op_return_eternity_wall'),
            memo: createAverageBlockCumulativeFeeSumPattern(client, 'op_return_memo'),
            bitproof: createAverageBlockCumulativeFeeSumPattern(client, 'op_return_bitproof'),
            ascribe: createAverageBlockCumulativeFeeSumPattern(client, 'op_return_ascribe'),
            stampery: createAverageBlockCumulativeFeeSumPattern(client, 'op_return_stampery'),
            epobc: createAverageBlockCumulativeFeeSumPattern(client, 'op_return_epobc'),
            bareHash: createAverageBlockCumulativeFeeSumPattern(client, 'op_return_bare_hash'),
            text: createAverageBlockCumulativeFeeSumPattern(client, 'op_return_text'),
            empty: createAverageBlockCumulativeFeeSumPattern(client, 'op_return_empty'),
            unknown: createAverageBlockCumulativeFeeSumPattern(client, 'op_return_unknown'),
          })); },
        })); },
        get policy() { return _lazy(this, 'policy', () => ({
          get outputCount() { return _lazy(this, 'outputCount', () => ({
            preV30Standard: createAverageBlockCumulativeSumPattern2(client, 'op_return_policy_pre_v30_standard_output_count'),
            preV30Nonstandard: createAverageBlockCumulativeSumPattern2(client, 'op_return_policy_pre_v30_nonstandard_output_count'),
            oversized: createAverageBlockCumulativeSumPattern2(client, 'op_return_policy_oversized_output_count'),
            multiple: createAverageBlockCumulativeSumPattern2(client, 'op_return_policy_multiple_output_count'),
          })); },
          get dataBytes() { return _lazy(this, 'dataBytes', () => ({
            preV30Standard: createAverageBlockChainCumulativeDataSumPattern(client, 'op_return_policy_pre_v30_standard'),
            preV30Nonstandard: createAverageBlockChainCumulativeDataSumPattern(client, 'op_return_policy_pre_v30_nonstandard'),
            oversized: createAverageBlockChainCumulativeDataSumPattern(client, 'op_return_policy_oversized'),
            multiple: createAverageBlockChainCumulativeDataSumPattern(client, 'op_return_policy_multiple'),
          })); },
          get txCount() { return _lazy(this, 'txCount', () => ({
            preV30Standard: createAverageBlockCumulativeSumPattern2(client, 'op_return_policy_pre_v30_standard_tx_count'),
            preV30Nonstandard: createAverageBlockCumulativeSumPattern2(client, 'op_return_policy_pre_v30_nonstandard_tx_count'),
            oversized: createAverageBlockCumulativeSumPattern2(client, 'op_return_policy_oversized_tx_count'),
            multiple: createAverageBlockCumulativeSumPattern2(client, 'op_return_policy_multiple_tx_count'),
          })); },
          get txVsize() { return _lazy(this, 'txVsize', () => ({
            preV30Standard: createAverageBlockCumulativeSumPattern2(client, 'op_return_policy_pre_v30_standard_tx_vsize'),
            preV30Nonstandard: createAverageBlockCumulativeSumPattern2(client, 'op_return_policy_pre_v30_nonstandard_tx_vsize'),
            oversized: createAverageBlockCumulativeSumPattern2(client, 'op_return_policy_oversized_tx_vsize'),
            multiple: createAverageBlockCumulativeSumPattern2(client, 'op_return_policy_multiple_tx_vsize'),
          })); },
          get fees() { return _lazy(this, 'fees', () => ({
            preV30Standard: createAverageBlockCumulativeFeeSumPattern(client, 'op_return_policy_pre_v30_standard'),
            preV30Nonstandard: createAverageBlockCumulativeFeeSumPattern(client, 'op_return_policy_pre_v30_nonstandard'),
            oversized: createAverageBlockCumulativeFeeSumPattern(client, 'op_return_policy_oversized'),
            multiple: createAverageBlockCumulativeFeeSumPattern(client, 'op_return_policy_multiple'),
          })); },
        })); },
      })); },
      get mining() { return _lazy(this, 'mining', () => ({
        get rewards() { return _lazy(this, 'rewards', () => ({
          coinbase: createAverageBlockCumulativeSumPattern(client, 'coinbase'),
          get subsidy() { return _lazy(this, 'subsidy', () => ({
            block: createBtcCentsSatsUsdPattern3(client, 'subsidy'),
            cumulative: createBtcCentsSatsUsdPattern(client, 'subsidy_cumulative'),
            sum: create_1m1w1y24hPattern5(client, 'subsidy_sum'),
            average: create_1m1w1y24hPattern4(client, 'subsidy_average'),
            dominance: create_1m1w1y24hPercentPpmRatioPattern(client, 'subsidy_dominance'),
          })); },
          get fees() { return _lazy(this, 'fees', () => ({
            block: createBtcCentsSatsUsdPattern3(client, 'fees'),
            cumulative: createBtcCentsSatsUsdPattern(client, 'fees_cumulative'),
            sum: create_1m1w1y24hPattern5(client, 'fees_sum'),
            average: create_1m1w1y24hPattern4(client, 'fees_average'),
            min: create_1m1w1y24hPattern5(client, 'fees_min'),
            max: create_1m1w1y24hPattern5(client, 'fees_max'),
            pct10: create_1m1w1y24hPattern5(client, 'fees_pct10'),
            pct25: create_1m1w1y24hPattern5(client, 'fees_pct25'),
            median: create_1m1w1y24hPattern5(client, 'fees_median'),
            pct75: create_1m1w1y24hPattern5(client, 'fees_pct75'),
            pct90: create_1m1w1y24hPattern5(client, 'fees_pct90'),
            dominance: create_1m1w1y24hPercentPpmRatioPattern(client, 'fee_dominance'),
            get toSubsidy() { return _lazy(this, 'toSubsidy', () => ({
              _24h: createPercentPpmRatioPattern5(client, 'fee_to_subsidy_24h'),
              _1w: createPercentPpmRatioPattern5(client, 'fee_to_subsidy_1w'),
              _1m: createPercentPpmRatioPattern5(client, 'fee_to_subsidy_1m'),
              _1y: createPercentPpmRatioPattern5(client, 'fee_to_subsidy_1y'),
            })); },
          })); },
          outputVolume: createSeriesPattern18(client, 'output_volume'),
          unclaimed: createBlockCumulativePattern(client, 'unclaimed_rewards'),
        })); },
        get hashrate() { return _lazy(this, 'hashrate', () => ({
          get rate() { return _lazy(this, 'rate', () => ({
            base: createSeriesPattern1(client, 'hash_rate'),
            get sma() { return _lazy(this, 'sma', () => ({
              _1w: createSeriesPattern1(client, 'hash_rate_sma_1w'),
              _1m: createSeriesPattern1(client, 'hash_rate_sma_1m'),
              _2m: createSeriesPattern1(client, 'hash_rate_sma_2m'),
              _1y: createSeriesPattern1(client, 'hash_rate_sma_1y'),
            })); },
            ath: createSeriesPattern1(client, 'hash_rate_ath'),
            drawdown: createPercentPpmRatioPattern3(client, 'hash_rate_drawdown'),
          })); },
          price: createPhsReboundThsPattern(client, 'hash_price'),
          value: createPhsReboundThsPattern(client, 'hash_value'),
        })); },
      })); },
      get cointime() { return _lazy(this, 'cointime', () => ({
        get activity() { return _lazy(this, 'activity', () => ({
          coinblocksCreated: createAverageBlockCumulativeSumPattern2(client, 'coinblocks_created'),
          coinblocksStored: createAverageBlockCumulativeSumPattern2(client, 'coinblocks_stored'),
          liveliness: createSeriesPattern1(client, 'liveliness'),
          vaultedness: createSeriesPattern1(client, 'vaultedness'),
          ratio: createSeriesPattern1(client, 'activity_to_vaultedness'),
          coinblocksDestroyed: createAverageBlockCumulativeSumPattern2(client, 'coinblocks_destroyed'),
        })); },
        get ageRange() { return _lazy(this, 'ageRange', () => ({
          get coindaysConsumed() { return _lazy(this, 'coindaysConsumed', () => ({
            under1h: createAverageBlockCumulativeSumPattern2(client, 'utxos_under_1h_old_coindays_consumed'),
            _1hTo1d: createAverageBlockCumulativeSumPattern2(client, 'utxos_1h_to_1d_old_coindays_consumed'),
            _1dTo1w: createAverageBlockCumulativeSumPattern2(client, 'utxos_1d_to_1w_old_coindays_consumed'),
            _1wTo1m: createAverageBlockCumulativeSumPattern2(client, 'utxos_1w_to_1m_old_coindays_consumed'),
            _1mTo2m: createAverageBlockCumulativeSumPattern2(client, 'utxos_1m_to_2m_old_coindays_consumed'),
            _2mTo3m: createAverageBlockCumulativeSumPattern2(client, 'utxos_2m_to_3m_old_coindays_consumed'),
            _3mTo4m: createAverageBlockCumulativeSumPattern2(client, 'utxos_3m_to_4m_old_coindays_consumed'),
            _4mTo5m: createAverageBlockCumulativeSumPattern2(client, 'utxos_4m_to_5m_old_coindays_consumed'),
            _5mTo6m: createAverageBlockCumulativeSumPattern2(client, 'utxos_5m_to_6m_old_coindays_consumed'),
            _6mTo9m: createAverageBlockCumulativeSumPattern2(client, 'utxos_6m_to_9m_old_coindays_consumed'),
            _9mTo1y: createAverageBlockCumulativeSumPattern2(client, 'utxos_9m_to_1y_old_coindays_consumed'),
            _1yTo18m: createAverageBlockCumulativeSumPattern2(client, 'utxos_1y_to_18m_old_coindays_consumed'),
            _18mTo2y: createAverageBlockCumulativeSumPattern2(client, 'utxos_18m_to_2y_old_coindays_consumed'),
            _2yTo3y: createAverageBlockCumulativeSumPattern2(client, 'utxos_2y_to_3y_old_coindays_consumed'),
            _3yTo4y: createAverageBlockCumulativeSumPattern2(client, 'utxos_3y_to_4y_old_coindays_consumed'),
            _4yTo5y: createAverageBlockCumulativeSumPattern2(client, 'utxos_4y_to_5y_old_coindays_consumed'),
            _5yTo6y: createAverageBlockCumulativeSumPattern2(client, 'utxos_5y_to_6y_old_coindays_consumed'),
            _6yTo7y: createAverageBlockCumulativeSumPattern2(client, 'utxos_6y_to_7y_old_coindays_consumed'),
            _7yTo8y: createAverageBlockCumulativeSumPattern2(client, 'utxos_7y_to_8y_old_coindays_consumed'),
            _8yTo10y: createAverageBlockCumulativeSumPattern2(client, 'utxos_8y_to_10y_old_coindays_consumed'),
            _10yTo12y: createAverageBlockCumulativeSumPattern2(client, 'utxos_10y_to_12y_old_coindays_consumed'),
            _12yTo15y: createAverageBlockCumulativeSumPattern2(client, 'utxos_12y_to_15y_old_coindays_consumed'),
            over15y: createAverageBlockCumulativeSumPattern2(client, 'utxos_over_15y_old_coindays_consumed'),
          })); },
          get coindaysStored() { return _lazy(this, 'coindaysStored', () => ({
            under1h: createAverageBlockCumulativeSumPattern2(client, 'utxos_under_1h_old_coindays_stored'),
            _1hTo1d: createAverageBlockCumulativeSumPattern2(client, 'utxos_1h_to_1d_old_coindays_stored'),
            _1dTo1w: createAverageBlockCumulativeSumPattern2(client, 'utxos_1d_to_1w_old_coindays_stored'),
            _1wTo1m: createAverageBlockCumulativeSumPattern2(client, 'utxos_1w_to_1m_old_coindays_stored'),
            _1mTo2m: createAverageBlockCumulativeSumPattern2(client, 'utxos_1m_to_2m_old_coindays_stored'),
            _2mTo3m: createAverageBlockCumulativeSumPattern2(client, 'utxos_2m_to_3m_old_coindays_stored'),
            _3mTo4m: createAverageBlockCumulativeSumPattern2(client, 'utxos_3m_to_4m_old_coindays_stored'),
            _4mTo5m: createAverageBlockCumulativeSumPattern2(client, 'utxos_4m_to_5m_old_coindays_stored'),
            _5mTo6m: createAverageBlockCumulativeSumPattern2(client, 'utxos_5m_to_6m_old_coindays_stored'),
            _6mTo9m: createAverageBlockCumulativeSumPattern2(client, 'utxos_6m_to_9m_old_coindays_stored'),
            _9mTo1y: createAverageBlockCumulativeSumPattern2(client, 'utxos_9m_to_1y_old_coindays_stored'),
            _1yTo18m: createAverageBlockCumulativeSumPattern2(client, 'utxos_1y_to_18m_old_coindays_stored'),
            _18mTo2y: createAverageBlockCumulativeSumPattern2(client, 'utxos_18m_to_2y_old_coindays_stored'),
            _2yTo3y: createAverageBlockCumulativeSumPattern2(client, 'utxos_2y_to_3y_old_coindays_stored'),
            _3yTo4y: createAverageBlockCumulativeSumPattern2(client, 'utxos_3y_to_4y_old_coindays_stored'),
            _4yTo5y: createAverageBlockCumulativeSumPattern2(client, 'utxos_4y_to_5y_old_coindays_stored'),
            _5yTo6y: createAverageBlockCumulativeSumPattern2(client, 'utxos_5y_to_6y_old_coindays_stored'),
            _6yTo7y: createAverageBlockCumulativeSumPattern2(client, 'utxos_6y_to_7y_old_coindays_stored'),
            _7yTo8y: createAverageBlockCumulativeSumPattern2(client, 'utxos_7y_to_8y_old_coindays_stored'),
            _8yTo10y: createAverageBlockCumulativeSumPattern2(client, 'utxos_8y_to_10y_old_coindays_stored'),
            _10yTo12y: createAverageBlockCumulativeSumPattern2(client, 'utxos_10y_to_12y_old_coindays_stored'),
            _12yTo15y: createAverageBlockCumulativeSumPattern2(client, 'utxos_12y_to_15y_old_coindays_stored'),
            over15y: createAverageBlockCumulativeSumPattern2(client, 'utxos_over_15y_old_coindays_stored'),
          })); },
          get activity() { return _lazy(this, 'activity', () => ({
            get wakefulness() { return _lazy(this, 'wakefulness', () => ({
              under1h: createSeriesPattern1(client, 'utxos_under_1h_old_wakefulness'),
              _1hTo1d: createSeriesPattern1(client, 'utxos_1h_to_1d_old_wakefulness'),
              _1dTo1w: createSeriesPattern1(client, 'utxos_1d_to_1w_old_wakefulness'),
              _1wTo1m: createSeriesPattern1(client, 'utxos_1w_to_1m_old_wakefulness'),
              _1mTo2m: createSeriesPattern1(client, 'utxos_1m_to_2m_old_wakefulness'),
              _2mTo3m: createSeriesPattern1(client, 'utxos_2m_to_3m_old_wakefulness'),
              _3mTo4m: createSeriesPattern1(client, 'utxos_3m_to_4m_old_wakefulness'),
              _4mTo5m: createSeriesPattern1(client, 'utxos_4m_to_5m_old_wakefulness'),
              _5mTo6m: createSeriesPattern1(client, 'utxos_5m_to_6m_old_wakefulness'),
              _6mTo9m: createSeriesPattern1(client, 'utxos_6m_to_9m_old_wakefulness'),
              _9mTo1y: createSeriesPattern1(client, 'utxos_9m_to_1y_old_wakefulness'),
              _1yTo18m: createSeriesPattern1(client, 'utxos_1y_to_18m_old_wakefulness'),
              _18mTo2y: createSeriesPattern1(client, 'utxos_18m_to_2y_old_wakefulness'),
              _2yTo3y: createSeriesPattern1(client, 'utxos_2y_to_3y_old_wakefulness'),
              _3yTo4y: createSeriesPattern1(client, 'utxos_3y_to_4y_old_wakefulness'),
              _4yTo5y: createSeriesPattern1(client, 'utxos_4y_to_5y_old_wakefulness'),
              _5yTo6y: createSeriesPattern1(client, 'utxos_5y_to_6y_old_wakefulness'),
              _6yTo7y: createSeriesPattern1(client, 'utxos_6y_to_7y_old_wakefulness'),
              _7yTo8y: createSeriesPattern1(client, 'utxos_7y_to_8y_old_wakefulness'),
              _8yTo10y: createSeriesPattern1(client, 'utxos_8y_to_10y_old_wakefulness'),
              _10yTo12y: createSeriesPattern1(client, 'utxos_10y_to_12y_old_wakefulness'),
              _12yTo15y: createSeriesPattern1(client, 'utxos_12y_to_15y_old_wakefulness'),
              over15y: createSeriesPattern1(client, 'utxos_over_15y_old_wakefulness'),
            })); },
            get dormancy() { return _lazy(this, 'dormancy', () => ({
              under1h: createSeriesPattern1(client, 'utxos_under_1h_old_dormancy'),
              _1hTo1d: createSeriesPattern1(client, 'utxos_1h_to_1d_old_dormancy'),
              _1dTo1w: createSeriesPattern1(client, 'utxos_1d_to_1w_old_dormancy'),
              _1wTo1m: createSeriesPattern1(client, 'utxos_1w_to_1m_old_dormancy'),
              _1mTo2m: createSeriesPattern1(client, 'utxos_1m_to_2m_old_dormancy'),
              _2mTo3m: createSeriesPattern1(client, 'utxos_2m_to_3m_old_dormancy'),
              _3mTo4m: createSeriesPattern1(client, 'utxos_3m_to_4m_old_dormancy'),
              _4mTo5m: createSeriesPattern1(client, 'utxos_4m_to_5m_old_dormancy'),
              _5mTo6m: createSeriesPattern1(client, 'utxos_5m_to_6m_old_dormancy'),
              _6mTo9m: createSeriesPattern1(client, 'utxos_6m_to_9m_old_dormancy'),
              _9mTo1y: createSeriesPattern1(client, 'utxos_9m_to_1y_old_dormancy'),
              _1yTo18m: createSeriesPattern1(client, 'utxos_1y_to_18m_old_dormancy'),
              _18mTo2y: createSeriesPattern1(client, 'utxos_18m_to_2y_old_dormancy'),
              _2yTo3y: createSeriesPattern1(client, 'utxos_2y_to_3y_old_dormancy'),
              _3yTo4y: createSeriesPattern1(client, 'utxos_3y_to_4y_old_dormancy'),
              _4yTo5y: createSeriesPattern1(client, 'utxos_4y_to_5y_old_dormancy'),
              _5yTo6y: createSeriesPattern1(client, 'utxos_5y_to_6y_old_dormancy'),
              _6yTo7y: createSeriesPattern1(client, 'utxos_6y_to_7y_old_dormancy'),
              _7yTo8y: createSeriesPattern1(client, 'utxos_7y_to_8y_old_dormancy'),
              _8yTo10y: createSeriesPattern1(client, 'utxos_8y_to_10y_old_dormancy'),
              _10yTo12y: createSeriesPattern1(client, 'utxos_10y_to_12y_old_dormancy'),
              _12yTo15y: createSeriesPattern1(client, 'utxos_12y_to_15y_old_dormancy'),
              over15y: createSeriesPattern1(client, 'utxos_over_15y_old_dormancy'),
            })); },
            get wakefulnessToDormancy() { return _lazy(this, 'wakefulnessToDormancy', () => ({
              under1h: createSeriesPattern1(client, 'utxos_under_1h_old_wakefulness_to_dormancy'),
              _1hTo1d: createSeriesPattern1(client, 'utxos_1h_to_1d_old_wakefulness_to_dormancy'),
              _1dTo1w: createSeriesPattern1(client, 'utxos_1d_to_1w_old_wakefulness_to_dormancy'),
              _1wTo1m: createSeriesPattern1(client, 'utxos_1w_to_1m_old_wakefulness_to_dormancy'),
              _1mTo2m: createSeriesPattern1(client, 'utxos_1m_to_2m_old_wakefulness_to_dormancy'),
              _2mTo3m: createSeriesPattern1(client, 'utxos_2m_to_3m_old_wakefulness_to_dormancy'),
              _3mTo4m: createSeriesPattern1(client, 'utxos_3m_to_4m_old_wakefulness_to_dormancy'),
              _4mTo5m: createSeriesPattern1(client, 'utxos_4m_to_5m_old_wakefulness_to_dormancy'),
              _5mTo6m: createSeriesPattern1(client, 'utxos_5m_to_6m_old_wakefulness_to_dormancy'),
              _6mTo9m: createSeriesPattern1(client, 'utxos_6m_to_9m_old_wakefulness_to_dormancy'),
              _9mTo1y: createSeriesPattern1(client, 'utxos_9m_to_1y_old_wakefulness_to_dormancy'),
              _1yTo18m: createSeriesPattern1(client, 'utxos_1y_to_18m_old_wakefulness_to_dormancy'),
              _18mTo2y: createSeriesPattern1(client, 'utxos_18m_to_2y_old_wakefulness_to_dormancy'),
              _2yTo3y: createSeriesPattern1(client, 'utxos_2y_to_3y_old_wakefulness_to_dormancy'),
              _3yTo4y: createSeriesPattern1(client, 'utxos_3y_to_4y_old_wakefulness_to_dormancy'),
              _4yTo5y: createSeriesPattern1(client, 'utxos_4y_to_5y_old_wakefulness_to_dormancy'),
              _5yTo6y: createSeriesPattern1(client, 'utxos_5y_to_6y_old_wakefulness_to_dormancy'),
              _6yTo7y: createSeriesPattern1(client, 'utxos_6y_to_7y_old_wakefulness_to_dormancy'),
              _7yTo8y: createSeriesPattern1(client, 'utxos_7y_to_8y_old_wakefulness_to_dormancy'),
              _8yTo10y: createSeriesPattern1(client, 'utxos_8y_to_10y_old_wakefulness_to_dormancy'),
              _10yTo12y: createSeriesPattern1(client, 'utxos_10y_to_12y_old_wakefulness_to_dormancy'),
              _12yTo15y: createSeriesPattern1(client, 'utxos_12y_to_15y_old_wakefulness_to_dormancy'),
              over15y: createSeriesPattern1(client, 'utxos_over_15y_old_wakefulness_to_dormancy'),
            })); },
          })); },
          get supply() { return _lazy(this, 'supply', () => ({
            get awake() { return _lazy(this, 'awake', () => ({
              under1h: createBtcCentsSatsUsdPattern(client, 'utxos_under_1h_old_awake_supply'),
              _1hTo1d: createBtcCentsSatsUsdPattern(client, 'utxos_1h_to_1d_old_awake_supply'),
              _1dTo1w: createBtcCentsSatsUsdPattern(client, 'utxos_1d_to_1w_old_awake_supply'),
              _1wTo1m: createBtcCentsSatsUsdPattern(client, 'utxos_1w_to_1m_old_awake_supply'),
              _1mTo2m: createBtcCentsSatsUsdPattern(client, 'utxos_1m_to_2m_old_awake_supply'),
              _2mTo3m: createBtcCentsSatsUsdPattern(client, 'utxos_2m_to_3m_old_awake_supply'),
              _3mTo4m: createBtcCentsSatsUsdPattern(client, 'utxos_3m_to_4m_old_awake_supply'),
              _4mTo5m: createBtcCentsSatsUsdPattern(client, 'utxos_4m_to_5m_old_awake_supply'),
              _5mTo6m: createBtcCentsSatsUsdPattern(client, 'utxos_5m_to_6m_old_awake_supply'),
              _6mTo9m: createBtcCentsSatsUsdPattern(client, 'utxos_6m_to_9m_old_awake_supply'),
              _9mTo1y: createBtcCentsSatsUsdPattern(client, 'utxos_9m_to_1y_old_awake_supply'),
              _1yTo18m: createBtcCentsSatsUsdPattern(client, 'utxos_1y_to_18m_old_awake_supply'),
              _18mTo2y: createBtcCentsSatsUsdPattern(client, 'utxos_18m_to_2y_old_awake_supply'),
              _2yTo3y: createBtcCentsSatsUsdPattern(client, 'utxos_2y_to_3y_old_awake_supply'),
              _3yTo4y: createBtcCentsSatsUsdPattern(client, 'utxos_3y_to_4y_old_awake_supply'),
              _4yTo5y: createBtcCentsSatsUsdPattern(client, 'utxos_4y_to_5y_old_awake_supply'),
              _5yTo6y: createBtcCentsSatsUsdPattern(client, 'utxos_5y_to_6y_old_awake_supply'),
              _6yTo7y: createBtcCentsSatsUsdPattern(client, 'utxos_6y_to_7y_old_awake_supply'),
              _7yTo8y: createBtcCentsSatsUsdPattern(client, 'utxos_7y_to_8y_old_awake_supply'),
              _8yTo10y: createBtcCentsSatsUsdPattern(client, 'utxos_8y_to_10y_old_awake_supply'),
              _10yTo12y: createBtcCentsSatsUsdPattern(client, 'utxos_10y_to_12y_old_awake_supply'),
              _12yTo15y: createBtcCentsSatsUsdPattern(client, 'utxos_12y_to_15y_old_awake_supply'),
              over15y: createBtcCentsSatsUsdPattern(client, 'utxos_over_15y_old_awake_supply'),
            })); },
            get dormant() { return _lazy(this, 'dormant', () => ({
              under1h: createBtcCentsSatsUsdPattern(client, 'utxos_under_1h_old_dormant_supply'),
              _1hTo1d: createBtcCentsSatsUsdPattern(client, 'utxos_1h_to_1d_old_dormant_supply'),
              _1dTo1w: createBtcCentsSatsUsdPattern(client, 'utxos_1d_to_1w_old_dormant_supply'),
              _1wTo1m: createBtcCentsSatsUsdPattern(client, 'utxos_1w_to_1m_old_dormant_supply'),
              _1mTo2m: createBtcCentsSatsUsdPattern(client, 'utxos_1m_to_2m_old_dormant_supply'),
              _2mTo3m: createBtcCentsSatsUsdPattern(client, 'utxos_2m_to_3m_old_dormant_supply'),
              _3mTo4m: createBtcCentsSatsUsdPattern(client, 'utxos_3m_to_4m_old_dormant_supply'),
              _4mTo5m: createBtcCentsSatsUsdPattern(client, 'utxos_4m_to_5m_old_dormant_supply'),
              _5mTo6m: createBtcCentsSatsUsdPattern(client, 'utxos_5m_to_6m_old_dormant_supply'),
              _6mTo9m: createBtcCentsSatsUsdPattern(client, 'utxos_6m_to_9m_old_dormant_supply'),
              _9mTo1y: createBtcCentsSatsUsdPattern(client, 'utxos_9m_to_1y_old_dormant_supply'),
              _1yTo18m: createBtcCentsSatsUsdPattern(client, 'utxos_1y_to_18m_old_dormant_supply'),
              _18mTo2y: createBtcCentsSatsUsdPattern(client, 'utxos_18m_to_2y_old_dormant_supply'),
              _2yTo3y: createBtcCentsSatsUsdPattern(client, 'utxos_2y_to_3y_old_dormant_supply'),
              _3yTo4y: createBtcCentsSatsUsdPattern(client, 'utxos_3y_to_4y_old_dormant_supply'),
              _4yTo5y: createBtcCentsSatsUsdPattern(client, 'utxos_4y_to_5y_old_dormant_supply'),
              _5yTo6y: createBtcCentsSatsUsdPattern(client, 'utxos_5y_to_6y_old_dormant_supply'),
              _6yTo7y: createBtcCentsSatsUsdPattern(client, 'utxos_6y_to_7y_old_dormant_supply'),
              _7yTo8y: createBtcCentsSatsUsdPattern(client, 'utxos_7y_to_8y_old_dormant_supply'),
              _8yTo10y: createBtcCentsSatsUsdPattern(client, 'utxos_8y_to_10y_old_dormant_supply'),
              _10yTo12y: createBtcCentsSatsUsdPattern(client, 'utxos_10y_to_12y_old_dormant_supply'),
              _12yTo15y: createBtcCentsSatsUsdPattern(client, 'utxos_12y_to_15y_old_dormant_supply'),
              over15y: createBtcCentsSatsUsdPattern(client, 'utxos_over_15y_old_dormant_supply'),
            })); },
          })); },
          get coindaysCreated() { return _lazy(this, 'coindaysCreated', () => ({
            under1h: createAverageBlockCumulativeSumPattern2(client, 'utxos_under_1h_old_coindays_created'),
            _1hTo1d: createAverageBlockCumulativeSumPattern2(client, 'utxos_1h_to_1d_old_coindays_created'),
            _1dTo1w: createAverageBlockCumulativeSumPattern2(client, 'utxos_1d_to_1w_old_coindays_created'),
            _1wTo1m: createAverageBlockCumulativeSumPattern2(client, 'utxos_1w_to_1m_old_coindays_created'),
            _1mTo2m: createAverageBlockCumulativeSumPattern2(client, 'utxos_1m_to_2m_old_coindays_created'),
            _2mTo3m: createAverageBlockCumulativeSumPattern2(client, 'utxos_2m_to_3m_old_coindays_created'),
            _3mTo4m: createAverageBlockCumulativeSumPattern2(client, 'utxos_3m_to_4m_old_coindays_created'),
            _4mTo5m: createAverageBlockCumulativeSumPattern2(client, 'utxos_4m_to_5m_old_coindays_created'),
            _5mTo6m: createAverageBlockCumulativeSumPattern2(client, 'utxos_5m_to_6m_old_coindays_created'),
            _6mTo9m: createAverageBlockCumulativeSumPattern2(client, 'utxos_6m_to_9m_old_coindays_created'),
            _9mTo1y: createAverageBlockCumulativeSumPattern2(client, 'utxos_9m_to_1y_old_coindays_created'),
            _1yTo18m: createAverageBlockCumulativeSumPattern2(client, 'utxos_1y_to_18m_old_coindays_created'),
            _18mTo2y: createAverageBlockCumulativeSumPattern2(client, 'utxos_18m_to_2y_old_coindays_created'),
            _2yTo3y: createAverageBlockCumulativeSumPattern2(client, 'utxos_2y_to_3y_old_coindays_created'),
            _3yTo4y: createAverageBlockCumulativeSumPattern2(client, 'utxos_3y_to_4y_old_coindays_created'),
            _4yTo5y: createAverageBlockCumulativeSumPattern2(client, 'utxos_4y_to_5y_old_coindays_created'),
            _5yTo6y: createAverageBlockCumulativeSumPattern2(client, 'utxos_5y_to_6y_old_coindays_created'),
            _6yTo7y: createAverageBlockCumulativeSumPattern2(client, 'utxos_6y_to_7y_old_coindays_created'),
            _7yTo8y: createAverageBlockCumulativeSumPattern2(client, 'utxos_7y_to_8y_old_coindays_created'),
            _8yTo10y: createAverageBlockCumulativeSumPattern2(client, 'utxos_8y_to_10y_old_coindays_created'),
            _10yTo12y: createAverageBlockCumulativeSumPattern2(client, 'utxos_10y_to_12y_old_coindays_created'),
            _12yTo15y: createAverageBlockCumulativeSumPattern2(client, 'utxos_12y_to_15y_old_coindays_created'),
            over15y: createAverageBlockCumulativeSumPattern2(client, 'utxos_over_15y_old_coindays_created'),
          })); },
        })); },
        get urpd() { return _lazy(this, 'urpd', () => ({
          all: createCapitalizedCostSupplyPattern(client, 'cointime'),
          get sth() { return _lazy(this, 'sth', () => ({
            costBasis: createPerPattern(client, 'sth_cointime_cost_basis_per'),
            capitalizedPrice: createCentsPpmRatioSatsUsdPattern(client, 'cointime_urpd_sth_capitalized_price'),
            supplyDensity: createInTotalPattern(client, 'cointime_urpd_sth_supply_density'),
          })); },
          get lth() { return _lazy(this, 'lth', () => ({
            costBasis: createPerPattern(client, 'lth_cointime_cost_basis_per'),
            capitalizedPrice: createCentsPpmRatioSatsUsdPattern(client, 'cointime_urpd_lth_capitalized_price'),
            supplyDensity: createInTotalPattern(client, 'cointime_urpd_lth_supply_density'),
          })); },
          get under4m() { return _lazy(this, 'under4m', () => ({
            costBasis: createPerPattern(client, 'under_4m_cointime_cost_basis_per'),
            capitalizedPrice: createCentsPpmRatioSatsUsdPattern(client, 'cointime_urpd_under_4m_capitalized_price'),
            supplyDensity: createInTotalPattern(client, 'cointime_urpd_under_4m_supply_density'),
          })); },
          get under6m() { return _lazy(this, 'under6m', () => ({
            costBasis: createPerPattern(client, 'under_6m_cointime_cost_basis_per'),
            capitalizedPrice: createCentsPpmRatioSatsUsdPattern(client, 'cointime_urpd_under_6m_capitalized_price'),
            supplyDensity: createInTotalPattern(client, 'cointime_urpd_under_6m_supply_density'),
          })); },
          get over4m() { return _lazy(this, 'over4m', () => ({
            costBasis: createPerPattern(client, 'over_4m_cointime_cost_basis_per'),
            capitalizedPrice: createCentsPpmRatioSatsUsdPattern(client, 'cointime_urpd_over_4m_capitalized_price'),
            supplyDensity: createInTotalPattern(client, 'cointime_urpd_over_4m_supply_density'),
          })); },
          get over6m() { return _lazy(this, 'over6m', () => ({
            costBasis: createPerPattern(client, 'over_6m_cointime_cost_basis_per'),
            capitalizedPrice: createCentsPpmRatioSatsUsdPattern(client, 'cointime_urpd_over_6m_capitalized_price'),
            supplyDensity: createInTotalPattern(client, 'cointime_urpd_over_6m_supply_density'),
          })); },
        })); },
        get awake() { return _lazy(this, 'awake', () => ({
          get supply() { return _lazy(this, 'supply', () => ({
            btc: createSeriesPattern1(client, 'awake_supply'),
            sats: createSeriesPattern1(client, 'awake_supply_sats'),
            usd: createSeriesPattern1(client, 'awake_supply_usd'),
            cents: createSeriesPattern1(client, 'awake_supply_cents'),
            inLoss: createSharePattern(client, 'awake_supply_in_loss_share'),
          })); },
          cap: createCentsUsdPattern(client, 'awake_cap'),
          price: createCentsPpmRatioSatsUsdPattern(client, 'awake_price'),
          capitalizedPrice: createCentsPpmRatioSatsUsdPattern(client, 'awake_capitalized_price'),
        })); },
        dormant: createSupplyPattern(client, 'dormant_supply'),
        get sth() { return _lazy(this, 'sth', () => ({
          get awake() { return _lazy(this, 'awake', () => ({
            get supply() { return _lazy(this, 'supply', () => ({
              btc: createSeriesPattern1(client, 'sth_awake_supply'),
              sats: createSeriesPattern1(client, 'sth_awake_supply_sats'),
              usd: createSeriesPattern1(client, 'sth_awake_supply_usd'),
              cents: createSeriesPattern1(client, 'sth_awake_supply_cents'),
              inLoss: createSharePattern(client, 'sth_awake_supply_in_loss_share'),
            })); },
            cap: createCentsUsdPattern(client, 'sth_awake_cap'),
            price: createCentsPpmRatioSatsUsdPattern(client, 'sth_awake_price'),
            capitalizedPrice: createCentsPpmRatioSatsUsdPattern(client, 'sth_awake_capitalized_price'),
          })); },
          dormant: createSupplyPattern(client, 'sth_dormant_supply'),
        })); },
        get lth() { return _lazy(this, 'lth', () => ({
          get awake() { return _lazy(this, 'awake', () => ({
            get supply() { return _lazy(this, 'supply', () => ({
              btc: createSeriesPattern1(client, 'lth_awake_supply'),
              sats: createSeriesPattern1(client, 'lth_awake_supply_sats'),
              usd: createSeriesPattern1(client, 'lth_awake_supply_usd'),
              cents: createSeriesPattern1(client, 'lth_awake_supply_cents'),
              inLoss: createSharePattern(client, 'lth_awake_supply_in_loss_share'),
            })); },
            cap: createCentsUsdPattern(client, 'lth_awake_cap'),
            price: createCentsPpmRatioSatsUsdPattern(client, 'lth_awake_price'),
            capitalizedPrice: createCentsPpmRatioSatsUsdPattern(client, 'lth_awake_capitalized_price'),
          })); },
          dormant: createSupplyPattern(client, 'lth_dormant_supply'),
        })); },
        under4mAwakePrice: createCentsPpmRatioSatsUsdPattern(client, 'under_4m_awake_price'),
        under4mAwakeCapitalizedPrice: createCentsPpmRatioSatsUsdPattern(client, 'under_4m_awake_capitalized_price'),
        under6mAwakePrice: createCentsPpmRatioSatsUsdPattern(client, 'under_6m_awake_price'),
        under6mAwakeCapitalizedPrice: createCentsPpmRatioSatsUsdPattern(client, 'under_6m_awake_capitalized_price'),
        over4mAwakePrice: createCentsPpmRatioSatsUsdPattern(client, 'over_4m_awake_price'),
        over4mAwakeCapitalizedPrice: createCentsPpmRatioSatsUsdPattern(client, 'over_4m_awake_capitalized_price'),
        over6mAwakePrice: createCentsPpmRatioSatsUsdPattern(client, 'over_6m_awake_price'),
        over6mAwakeCapitalizedPrice: createCentsPpmRatioSatsUsdPattern(client, 'over_6m_awake_capitalized_price'),
        get supply() { return _lazy(this, 'supply', () => ({
          vaulted: createBtcCentsSatsUsdPattern(client, 'vaulted_supply'),
          get active() { return _lazy(this, 'active', () => ({
            btc: createSeriesPattern1(client, 'active_supply'),
            sats: createSeriesPattern1(client, 'active_supply_sats'),
            usd: createSeriesPattern1(client, 'active_supply_usd'),
            cents: createSeriesPattern1(client, 'active_supply_cents'),
            get inLoss() { return _lazy(this, 'inLoss', () => ({
              get share() { return _lazy(this, 'share', () => ({
                bounded: createSeriesPattern1(client, 'cointime_supply_in_loss_share_bounded'),
                ratio: createSeriesPattern1(client, 'cointime_supply_in_loss_share'),
              })); },
            })); },
          })); },
        })); },
        get value() { return _lazy(this, 'value', () => ({
          destroyed: createAverageBlockCumulativeSumPattern2(client, 'cointime_value_destroyed'),
          created: createAverageBlockCumulativeSumPattern2(client, 'cointime_value_created'),
          stored: createAverageBlockCumulativeSumPattern2(client, 'cointime_value_stored'),
          vocdd: createAverageBlockCumulativeSumPattern2(client, 'vocdd'),
        })); },
        get cap() { return _lazy(this, 'cap', () => ({
          thermo: createCentsUsdPattern(client, 'thermo_cap'),
          investor: createCentsUsdPattern(client, 'investor_cap'),
          vaulted: createCentsUsdPattern(client, 'vaulted_cap'),
          active: createCentsUsdPattern(client, 'active_cap'),
          cointime: createCentsUsdPattern(client, 'cointime_cap'),
          aviv: createPpmRatioPattern(client, 'aviv_ratio'),
        })); },
        get prices() { return _lazy(this, 'prices', () => ({
          vaulted: createCentsPpmRatioSatsUsdPattern(client, 'vaulted_price'),
          active: createCentsPpmRatioSatsUsdPattern(client, 'active_price'),
          trueMarketMean: createCentsPpmRatioSatsUsdPattern(client, 'true_market_mean'),
          cointime: createCentsPpmRatioSatsUsdPattern(client, 'cointime_price'),
        })); },
        get adjusted() { return _lazy(this, 'adjusted', () => ({
          inflationRate: createPercentPpmRatioPattern3(client, 'cointime_adj_inflation_rate'),
          txVelocityNative: createSeriesPattern1(client, 'cointime_adj_tx_velocity_btc'),
          txVelocityFiat: createSeriesPattern1(client, 'cointime_adj_tx_velocity_usd'),
        })); },
        get reserveRisk() { return _lazy(this, 'reserveRisk', () => ({
          value: createSeriesPattern1(client, 'reserve_risk'),
          vocddMedian1y: createSeriesPattern18(client, 'vocdd_median_1y'),
          hodlBank: createSeriesPattern18(client, 'hodl_bank'),
        })); },
      })); },
      get coinflow() { return _lazy(this, 'coinflow', () => ({
        get ageRange() { return _lazy(this, 'ageRange', () => ({
          get spendingRate() { return _lazy(this, 'spendingRate', () => ({
            under1h: createSeriesPattern1(client, 'utxos_under_1h_old_spending_rate'),
            _1hTo1d: createSeriesPattern1(client, 'utxos_1h_to_1d_old_spending_rate'),
            _1dTo1w: createSeriesPattern1(client, 'utxos_1d_to_1w_old_spending_rate'),
            _1wTo1m: createSeriesPattern1(client, 'utxos_1w_to_1m_old_spending_rate'),
            _1mTo2m: createSeriesPattern1(client, 'utxos_1m_to_2m_old_spending_rate'),
            _2mTo3m: createSeriesPattern1(client, 'utxos_2m_to_3m_old_spending_rate'),
            _3mTo4m: createSeriesPattern1(client, 'utxos_3m_to_4m_old_spending_rate'),
            _4mTo5m: createSeriesPattern1(client, 'utxos_4m_to_5m_old_spending_rate'),
            _5mTo6m: createSeriesPattern1(client, 'utxos_5m_to_6m_old_spending_rate'),
            _6mTo9m: createSeriesPattern1(client, 'utxos_6m_to_9m_old_spending_rate'),
            _9mTo1y: createSeriesPattern1(client, 'utxos_9m_to_1y_old_spending_rate'),
            _1yTo18m: createSeriesPattern1(client, 'utxos_1y_to_18m_old_spending_rate'),
            _18mTo2y: createSeriesPattern1(client, 'utxos_18m_to_2y_old_spending_rate'),
            _2yTo3y: createSeriesPattern1(client, 'utxos_2y_to_3y_old_spending_rate'),
            _3yTo4y: createSeriesPattern1(client, 'utxos_3y_to_4y_old_spending_rate'),
            _4yTo5y: createSeriesPattern1(client, 'utxos_4y_to_5y_old_spending_rate'),
            _5yTo6y: createSeriesPattern1(client, 'utxos_5y_to_6y_old_spending_rate'),
            _6yTo7y: createSeriesPattern1(client, 'utxos_6y_to_7y_old_spending_rate'),
            _7yTo8y: createSeriesPattern1(client, 'utxos_7y_to_8y_old_spending_rate'),
            _8yTo10y: createSeriesPattern1(client, 'utxos_8y_to_10y_old_spending_rate'),
            _10yTo12y: createSeriesPattern1(client, 'utxos_10y_to_12y_old_spending_rate'),
            _12yTo15y: createSeriesPattern1(client, 'utxos_12y_to_15y_old_spending_rate'),
            over15y: createSeriesPattern1(client, 'utxos_over_15y_old_spending_rate'),
          })); },
          get spendingExposure() { return _lazy(this, 'spendingExposure', () => ({
            under1h: createSeriesPattern1(client, 'utxos_under_1h_old_spending_exposure'),
            _1hTo1d: createSeriesPattern1(client, 'utxos_1h_to_1d_old_spending_exposure'),
            _1dTo1w: createSeriesPattern1(client, 'utxos_1d_to_1w_old_spending_exposure'),
            _1wTo1m: createSeriesPattern1(client, 'utxos_1w_to_1m_old_spending_exposure'),
            _1mTo2m: createSeriesPattern1(client, 'utxos_1m_to_2m_old_spending_exposure'),
            _2mTo3m: createSeriesPattern1(client, 'utxos_2m_to_3m_old_spending_exposure'),
            _3mTo4m: createSeriesPattern1(client, 'utxos_3m_to_4m_old_spending_exposure'),
            _4mTo5m: createSeriesPattern1(client, 'utxos_4m_to_5m_old_spending_exposure'),
            _5mTo6m: createSeriesPattern1(client, 'utxos_5m_to_6m_old_spending_exposure'),
            _6mTo9m: createSeriesPattern1(client, 'utxos_6m_to_9m_old_spending_exposure'),
            _9mTo1y: createSeriesPattern1(client, 'utxos_9m_to_1y_old_spending_exposure'),
            _1yTo18m: createSeriesPattern1(client, 'utxos_1y_to_18m_old_spending_exposure'),
            _18mTo2y: createSeriesPattern1(client, 'utxos_18m_to_2y_old_spending_exposure'),
            _2yTo3y: createSeriesPattern1(client, 'utxos_2y_to_3y_old_spending_exposure'),
            _3yTo4y: createSeriesPattern1(client, 'utxos_3y_to_4y_old_spending_exposure'),
            _4yTo5y: createSeriesPattern1(client, 'utxos_4y_to_5y_old_spending_exposure'),
            _5yTo6y: createSeriesPattern1(client, 'utxos_5y_to_6y_old_spending_exposure'),
            _6yTo7y: createSeriesPattern1(client, 'utxos_6y_to_7y_old_spending_exposure'),
            _7yTo8y: createSeriesPattern1(client, 'utxos_7y_to_8y_old_spending_exposure'),
            _8yTo10y: createSeriesPattern1(client, 'utxos_8y_to_10y_old_spending_exposure'),
            _10yTo12y: createSeriesPattern1(client, 'utxos_10y_to_12y_old_spending_exposure'),
            _12yTo15y: createSeriesPattern1(client, 'utxos_12y_to_15y_old_spending_exposure'),
            over15y: createSeriesPattern1(client, 'utxos_over_15y_old_spending_exposure'),
            get mobility() { return _lazy(this, 'mobility', () => ({
              under1h: createSeriesPattern1(client, 'utxos_under_1h_old_mobility'),
              _1hTo1d: createSeriesPattern1(client, 'utxos_1h_to_1d_old_mobility'),
              _1dTo1w: createSeriesPattern1(client, 'utxos_1d_to_1w_old_mobility'),
              _1wTo1m: createSeriesPattern1(client, 'utxos_1w_to_1m_old_mobility'),
              _1mTo2m: createSeriesPattern1(client, 'utxos_1m_to_2m_old_mobility'),
              _2mTo3m: createSeriesPattern1(client, 'utxos_2m_to_3m_old_mobility'),
              _3mTo4m: createSeriesPattern1(client, 'utxos_3m_to_4m_old_mobility'),
              _4mTo5m: createSeriesPattern1(client, 'utxos_4m_to_5m_old_mobility'),
              _5mTo6m: createSeriesPattern1(client, 'utxos_5m_to_6m_old_mobility'),
              _6mTo9m: createSeriesPattern1(client, 'utxos_6m_to_9m_old_mobility'),
              _9mTo1y: createSeriesPattern1(client, 'utxos_9m_to_1y_old_mobility'),
              _1yTo18m: createSeriesPattern1(client, 'utxos_1y_to_18m_old_mobility'),
              _18mTo2y: createSeriesPattern1(client, 'utxos_18m_to_2y_old_mobility'),
              _2yTo3y: createSeriesPattern1(client, 'utxos_2y_to_3y_old_mobility'),
              _3yTo4y: createSeriesPattern1(client, 'utxos_3y_to_4y_old_mobility'),
              _4yTo5y: createSeriesPattern1(client, 'utxos_4y_to_5y_old_mobility'),
              _5yTo6y: createSeriesPattern1(client, 'utxos_5y_to_6y_old_mobility'),
              _6yTo7y: createSeriesPattern1(client, 'utxos_6y_to_7y_old_mobility'),
              _7yTo8y: createSeriesPattern1(client, 'utxos_7y_to_8y_old_mobility'),
              _8yTo10y: createSeriesPattern1(client, 'utxos_8y_to_10y_old_mobility'),
              _10yTo12y: createSeriesPattern1(client, 'utxos_10y_to_12y_old_mobility'),
              _12yTo15y: createSeriesPattern1(client, 'utxos_12y_to_15y_old_mobility'),
              over15y: createSeriesPattern1(client, 'utxos_over_15y_old_mobility'),
            })); },
          })); },
          get supply() { return _lazy(this, 'supply', () => ({
            get mobile() { return _lazy(this, 'mobile', () => ({
              under1h: createBtcCentsSatsUsdPattern(client, 'utxos_under_1h_old_mobile_supply'),
              _1hTo1d: createBtcCentsSatsUsdPattern(client, 'utxos_1h_to_1d_old_mobile_supply'),
              _1dTo1w: createBtcCentsSatsUsdPattern(client, 'utxos_1d_to_1w_old_mobile_supply'),
              _1wTo1m: createBtcCentsSatsUsdPattern(client, 'utxos_1w_to_1m_old_mobile_supply'),
              _1mTo2m: createBtcCentsSatsUsdPattern(client, 'utxos_1m_to_2m_old_mobile_supply'),
              _2mTo3m: createBtcCentsSatsUsdPattern(client, 'utxos_2m_to_3m_old_mobile_supply'),
              _3mTo4m: createBtcCentsSatsUsdPattern(client, 'utxos_3m_to_4m_old_mobile_supply'),
              _4mTo5m: createBtcCentsSatsUsdPattern(client, 'utxos_4m_to_5m_old_mobile_supply'),
              _5mTo6m: createBtcCentsSatsUsdPattern(client, 'utxos_5m_to_6m_old_mobile_supply'),
              _6mTo9m: createBtcCentsSatsUsdPattern(client, 'utxos_6m_to_9m_old_mobile_supply'),
              _9mTo1y: createBtcCentsSatsUsdPattern(client, 'utxos_9m_to_1y_old_mobile_supply'),
              _1yTo18m: createBtcCentsSatsUsdPattern(client, 'utxos_1y_to_18m_old_mobile_supply'),
              _18mTo2y: createBtcCentsSatsUsdPattern(client, 'utxos_18m_to_2y_old_mobile_supply'),
              _2yTo3y: createBtcCentsSatsUsdPattern(client, 'utxos_2y_to_3y_old_mobile_supply'),
              _3yTo4y: createBtcCentsSatsUsdPattern(client, 'utxos_3y_to_4y_old_mobile_supply'),
              _4yTo5y: createBtcCentsSatsUsdPattern(client, 'utxos_4y_to_5y_old_mobile_supply'),
              _5yTo6y: createBtcCentsSatsUsdPattern(client, 'utxos_5y_to_6y_old_mobile_supply'),
              _6yTo7y: createBtcCentsSatsUsdPattern(client, 'utxos_6y_to_7y_old_mobile_supply'),
              _7yTo8y: createBtcCentsSatsUsdPattern(client, 'utxos_7y_to_8y_old_mobile_supply'),
              _8yTo10y: createBtcCentsSatsUsdPattern(client, 'utxos_8y_to_10y_old_mobile_supply'),
              _10yTo12y: createBtcCentsSatsUsdPattern(client, 'utxos_10y_to_12y_old_mobile_supply'),
              _12yTo15y: createBtcCentsSatsUsdPattern(client, 'utxos_12y_to_15y_old_mobile_supply'),
              over15y: createBtcCentsSatsUsdPattern(client, 'utxos_over_15y_old_mobile_supply'),
            })); },
            get immobile() { return _lazy(this, 'immobile', () => ({
              under1h: createBtcCentsSatsUsdPattern(client, 'utxos_under_1h_old_immobile_supply'),
              _1hTo1d: createBtcCentsSatsUsdPattern(client, 'utxos_1h_to_1d_old_immobile_supply'),
              _1dTo1w: createBtcCentsSatsUsdPattern(client, 'utxos_1d_to_1w_old_immobile_supply'),
              _1wTo1m: createBtcCentsSatsUsdPattern(client, 'utxos_1w_to_1m_old_immobile_supply'),
              _1mTo2m: createBtcCentsSatsUsdPattern(client, 'utxos_1m_to_2m_old_immobile_supply'),
              _2mTo3m: createBtcCentsSatsUsdPattern(client, 'utxos_2m_to_3m_old_immobile_supply'),
              _3mTo4m: createBtcCentsSatsUsdPattern(client, 'utxos_3m_to_4m_old_immobile_supply'),
              _4mTo5m: createBtcCentsSatsUsdPattern(client, 'utxos_4m_to_5m_old_immobile_supply'),
              _5mTo6m: createBtcCentsSatsUsdPattern(client, 'utxos_5m_to_6m_old_immobile_supply'),
              _6mTo9m: createBtcCentsSatsUsdPattern(client, 'utxos_6m_to_9m_old_immobile_supply'),
              _9mTo1y: createBtcCentsSatsUsdPattern(client, 'utxos_9m_to_1y_old_immobile_supply'),
              _1yTo18m: createBtcCentsSatsUsdPattern(client, 'utxos_1y_to_18m_old_immobile_supply'),
              _18mTo2y: createBtcCentsSatsUsdPattern(client, 'utxos_18m_to_2y_old_immobile_supply'),
              _2yTo3y: createBtcCentsSatsUsdPattern(client, 'utxos_2y_to_3y_old_immobile_supply'),
              _3yTo4y: createBtcCentsSatsUsdPattern(client, 'utxos_3y_to_4y_old_immobile_supply'),
              _4yTo5y: createBtcCentsSatsUsdPattern(client, 'utxos_4y_to_5y_old_immobile_supply'),
              _5yTo6y: createBtcCentsSatsUsdPattern(client, 'utxos_5y_to_6y_old_immobile_supply'),
              _6yTo7y: createBtcCentsSatsUsdPattern(client, 'utxos_6y_to_7y_old_immobile_supply'),
              _7yTo8y: createBtcCentsSatsUsdPattern(client, 'utxos_7y_to_8y_old_immobile_supply'),
              _8yTo10y: createBtcCentsSatsUsdPattern(client, 'utxos_8y_to_10y_old_immobile_supply'),
              _10yTo12y: createBtcCentsSatsUsdPattern(client, 'utxos_10y_to_12y_old_immobile_supply'),
              _12yTo15y: createBtcCentsSatsUsdPattern(client, 'utxos_12y_to_15y_old_immobile_supply'),
              over15y: createBtcCentsSatsUsdPattern(client, 'utxos_over_15y_old_immobile_supply'),
            })); },
          })); },
        })); },
        get urpd() { return _lazy(this, 'urpd', () => ({
          all: createCapitalizedCostSupplyPattern(client, 'coinflow'),
          get sth() { return _lazy(this, 'sth', () => ({
            costBasis: createPerPattern(client, 'sth_coinflow_cost_basis_per'),
            capitalizedPrice: createCentsPpmRatioSatsUsdPattern(client, 'coinflow_urpd_sth_capitalized_price'),
            supplyDensity: createInTotalPattern(client, 'coinflow_urpd_sth_supply_density'),
          })); },
          get lth() { return _lazy(this, 'lth', () => ({
            costBasis: createPerPattern(client, 'lth_coinflow_cost_basis_per'),
            capitalizedPrice: createCentsPpmRatioSatsUsdPattern(client, 'coinflow_urpd_lth_capitalized_price'),
            supplyDensity: createInTotalPattern(client, 'coinflow_urpd_lth_supply_density'),
          })); },
          get under4m() { return _lazy(this, 'under4m', () => ({
            costBasis: createPerPattern(client, 'under_4m_coinflow_cost_basis_per'),
            capitalizedPrice: createCentsPpmRatioSatsUsdPattern(client, 'coinflow_urpd_under_4m_capitalized_price'),
            supplyDensity: createInTotalPattern(client, 'coinflow_urpd_under_4m_supply_density'),
          })); },
          get under6m() { return _lazy(this, 'under6m', () => ({
            costBasis: createPerPattern(client, 'under_6m_coinflow_cost_basis_per'),
            capitalizedPrice: createCentsPpmRatioSatsUsdPattern(client, 'coinflow_urpd_under_6m_capitalized_price'),
            supplyDensity: createInTotalPattern(client, 'coinflow_urpd_under_6m_supply_density'),
          })); },
          get over4m() { return _lazy(this, 'over4m', () => ({
            costBasis: createPerPattern(client, 'over_4m_coinflow_cost_basis_per'),
            capitalizedPrice: createCentsPpmRatioSatsUsdPattern(client, 'coinflow_urpd_over_4m_capitalized_price'),
            supplyDensity: createInTotalPattern(client, 'coinflow_urpd_over_4m_supply_density'),
          })); },
          get over6m() { return _lazy(this, 'over6m', () => ({
            costBasis: createPerPattern(client, 'over_6m_coinflow_cost_basis_per'),
            capitalizedPrice: createCentsPpmRatioSatsUsdPattern(client, 'coinflow_urpd_over_6m_capitalized_price'),
            supplyDensity: createInTotalPattern(client, 'coinflow_urpd_over_6m_supply_density'),
          })); },
        })); },
        get supply() { return _lazy(this, 'supply', () => ({
          get mobile() { return _lazy(this, 'mobile', () => ({
            btc: createSeriesPattern1(client, 'mobile_supply'),
            sats: createSeriesPattern1(client, 'mobile_supply_sats'),
            usd: createSeriesPattern1(client, 'mobile_supply_usd'),
            cents: createSeriesPattern1(client, 'mobile_supply_cents'),
            inLoss: createSharePattern(client, 'coinflow_supply_in_loss_share'),
          })); },
          immobile: createBtcCentsSatsUsdPattern(client, 'immobile_supply'),
        })); },
        cap: createCentsUsdPattern(client, 'coinflow_cap'),
        price: createCentsPpmRatioSatsUsdPattern(client, 'coinflow_price'),
        capitalizedPrice: createCentsPpmRatioSatsUsdPattern(client, 'coinflow_capitalized_price'),
        get sth() { return _lazy(this, 'sth', () => ({
          get supply() { return _lazy(this, 'supply', () => ({
            get mobile() { return _lazy(this, 'mobile', () => ({
              btc: createSeriesPattern1(client, 'sth_mobile_supply'),
              sats: createSeriesPattern1(client, 'sth_mobile_supply_sats'),
              usd: createSeriesPattern1(client, 'sth_mobile_supply_usd'),
              cents: createSeriesPattern1(client, 'sth_mobile_supply_cents'),
              inLoss: createSharePattern(client, 'sth_coinflow_supply_in_loss_share'),
            })); },
            immobile: createBtcCentsSatsUsdPattern(client, 'sth_immobile_supply'),
          })); },
          cap: createCentsUsdPattern(client, 'sth_coinflow_cap'),
          price: createCentsPpmRatioSatsUsdPattern(client, 'sth_coinflow_price'),
          capitalizedPrice: createCentsPpmRatioSatsUsdPattern(client, 'sth_coinflow_capitalized_price'),
        })); },
        get lth() { return _lazy(this, 'lth', () => ({
          get supply() { return _lazy(this, 'supply', () => ({
            get mobile() { return _lazy(this, 'mobile', () => ({
              btc: createSeriesPattern1(client, 'lth_mobile_supply'),
              sats: createSeriesPattern1(client, 'lth_mobile_supply_sats'),
              usd: createSeriesPattern1(client, 'lth_mobile_supply_usd'),
              cents: createSeriesPattern1(client, 'lth_mobile_supply_cents'),
              inLoss: createSharePattern(client, 'lth_coinflow_supply_in_loss_share'),
            })); },
            immobile: createBtcCentsSatsUsdPattern(client, 'lth_immobile_supply'),
          })); },
          cap: createCentsUsdPattern(client, 'lth_coinflow_cap'),
          price: createCentsPpmRatioSatsUsdPattern(client, 'lth_coinflow_price'),
          capitalizedPrice: createCentsPpmRatioSatsUsdPattern(client, 'lth_coinflow_capitalized_price'),
        })); },
        under4mPrice: createCentsPpmRatioSatsUsdPattern(client, 'under_4m_coinflow_price'),
        under4mCapitalizedPrice: createCentsPpmRatioSatsUsdPattern(client, 'under_4m_coinflow_capitalized_price'),
        under6mPrice: createCentsPpmRatioSatsUsdPattern(client, 'under_6m_coinflow_price'),
        under6mCapitalizedPrice: createCentsPpmRatioSatsUsdPattern(client, 'under_6m_coinflow_capitalized_price'),
        over4mPrice: createCentsPpmRatioSatsUsdPattern(client, 'over_4m_coinflow_price'),
        over4mCapitalizedPrice: createCentsPpmRatioSatsUsdPattern(client, 'over_4m_coinflow_capitalized_price'),
        over6mPrice: createCentsPpmRatioSatsUsdPattern(client, 'over_6m_coinflow_price'),
        over6mCapitalizedPrice: createCentsPpmRatioSatsUsdPattern(client, 'over_6m_coinflow_capitalized_price'),
      })); },
      get bedrock() { return _lazy(this, 'bedrock', () => ({
        raw: createFloorLevelSupplyPattern(client, 'bedrock_raw'),
        cointime: createFloorLevelSupplyPattern(client, 'bedrock_cointime'),
        coinflow: createFloorLevelSupplyPattern(client, 'bedrock_coinflow'),
      })); },
      get capitalSentiment() { return _lazy(this, 'capitalSentiment', () => ({
        isLong: createSeriesPattern1(client, 'capital_sentiment_is_long'),
        isShort: createSeriesPattern1(client, 'capital_sentiment_is_short'),
        phase: createSeriesPattern1(client, 'capital_sentiment_phase'),
        score: createSeriesPattern1(client, 'capital_sentiment_score'),
      })); },
      get rarityMeter() { return _lazy(this, 'rarityMeter', () => ({
        get components() { return _lazy(this, 'components', () => ({
          realizedPrice: createPct0Pct1Pct10Pct2Pct20Pct30Pct40Pct5Pct50Pct60Pct70Pct80Pct90Pct95Pct98Pct99Pattern(client, 'realized_price'),
          capitalizedPrice: createPct0Pct1Pct10Pct2Pct20Pct30Pct40Pct5Pct50Pct60Pct70Pct80Pct90Pct95Pct98Pct99Pattern(client, 'capitalized_price'),
          medianPriceBtcWeighted: createCentsPct0Pct1Pct10Pct2Pct20Pct30Pct40Pct5Pct50Pct60Pct70Pct80Pct90Pct95Pct98Pct99PpmRatioSatsUsdPattern(client, 'median_price_btc_weighted'),
          medianPriceUsdWeighted: createCentsPct0Pct1Pct10Pct2Pct20Pct30Pct40Pct5Pct50Pct60Pct70Pct80Pct90Pct95Pct98Pct99PpmRatioSatsUsdPattern(client, 'median_price_usd_weighted'),
          sthMedianPriceBtcWeighted: createCentsPct0Pct1Pct10Pct2Pct20Pct30Pct40Pct5Pct50Pct60Pct70Pct80Pct90Pct95Pct98Pct99PpmRatioSatsUsdPattern(client, 'sth_median_price_btc_weighted'),
          sthMedianPriceUsdWeighted: createCentsPct0Pct1Pct10Pct2Pct20Pct30Pct40Pct5Pct50Pct60Pct70Pct80Pct90Pct95Pct98Pct99PpmRatioSatsUsdPattern(client, 'sth_median_price_usd_weighted'),
          lthMedianPriceBtcWeighted: createCentsPct0Pct1Pct10Pct2Pct20Pct30Pct40Pct5Pct50Pct60Pct70Pct80Pct90Pct95Pct98Pct99PpmRatioSatsUsdPattern(client, 'lth_median_price_btc_weighted'),
          lthMedianPriceUsdWeighted: createCentsPct0Pct1Pct10Pct2Pct20Pct30Pct40Pct5Pct50Pct60Pct70Pct80Pct90Pct95Pct98Pct99PpmRatioSatsUsdPattern(client, 'lth_median_price_usd_weighted'),
          cointimeMedianPriceBtcWeighted: createCentsPct0Pct1Pct10Pct2Pct20Pct30Pct40Pct5Pct50Pct60Pct70Pct80Pct90Pct95Pct98Pct99PpmRatioSatsUsdPattern(client, 'cointime_median_price_btc_weighted'),
          cointimeMedianPriceUsdWeighted: createCentsPct0Pct1Pct10Pct2Pct20Pct30Pct40Pct5Pct50Pct60Pct70Pct80Pct90Pct95Pct98Pct99PpmRatioSatsUsdPattern(client, 'cointime_median_price_usd_weighted'),
          coinflowMedianPriceBtcWeighted: createCentsPct0Pct1Pct10Pct2Pct20Pct30Pct40Pct5Pct50Pct60Pct70Pct80Pct90Pct95Pct98Pct99PpmRatioSatsUsdPattern(client, 'coinflow_median_price_btc_weighted'),
          coinflowMedianPriceUsdWeighted: createCentsPct0Pct1Pct10Pct2Pct20Pct30Pct40Pct5Pct50Pct60Pct70Pct80Pct90Pct95Pct98Pct99PpmRatioSatsUsdPattern(client, 'coinflow_median_price_usd_weighted'),
          sthCointimeMedianPriceBtcWeighted: createCentsPct0Pct1Pct10Pct2Pct20Pct30Pct40Pct5Pct50Pct60Pct70Pct80Pct90Pct95Pct98Pct99PpmRatioSatsUsdPattern(client, 'sth_cointime_median_price_btc_weighted'),
          sthCointimeMedianPriceUsdWeighted: createCentsPct0Pct1Pct10Pct2Pct20Pct30Pct40Pct5Pct50Pct60Pct70Pct80Pct90Pct95Pct98Pct99PpmRatioSatsUsdPattern(client, 'sth_cointime_median_price_usd_weighted'),
          lthCointimeMedianPriceBtcWeighted: createCentsPct0Pct1Pct10Pct2Pct20Pct30Pct40Pct5Pct50Pct60Pct70Pct80Pct90Pct95Pct98Pct99PpmRatioSatsUsdPattern(client, 'lth_cointime_median_price_btc_weighted'),
          lthCointimeMedianPriceUsdWeighted: createCentsPct0Pct1Pct10Pct2Pct20Pct30Pct40Pct5Pct50Pct60Pct70Pct80Pct90Pct95Pct98Pct99PpmRatioSatsUsdPattern(client, 'lth_cointime_median_price_usd_weighted'),
          sthCoinflowMedianPriceBtcWeighted: createCentsPct0Pct1Pct10Pct2Pct20Pct30Pct40Pct5Pct50Pct60Pct70Pct80Pct90Pct95Pct98Pct99PpmRatioSatsUsdPattern(client, 'sth_coinflow_median_price_btc_weighted'),
          sthCoinflowMedianPriceUsdWeighted: createCentsPct0Pct1Pct10Pct2Pct20Pct30Pct40Pct5Pct50Pct60Pct70Pct80Pct90Pct95Pct98Pct99PpmRatioSatsUsdPattern(client, 'sth_coinflow_median_price_usd_weighted'),
          lthCoinflowMedianPriceBtcWeighted: createCentsPct0Pct1Pct10Pct2Pct20Pct30Pct40Pct5Pct50Pct60Pct70Pct80Pct90Pct95Pct98Pct99PpmRatioSatsUsdPattern(client, 'lth_coinflow_median_price_btc_weighted'),
          lthCoinflowMedianPriceUsdWeighted: createCentsPct0Pct1Pct10Pct2Pct20Pct30Pct40Pct5Pct50Pct60Pct70Pct80Pct90Pct95Pct98Pct99PpmRatioSatsUsdPattern(client, 'lth_coinflow_median_price_usd_weighted'),
          sthRealizedPrice: createPct0Pct1Pct10Pct2Pct20Pct30Pct40Pct5Pct50Pct60Pct70Pct80Pct90Pct95Pct98Pct99Pattern(client, 'sth_realized_price'),
          sthCapitalizedPrice: createPct0Pct1Pct10Pct2Pct20Pct30Pct40Pct5Pct50Pct60Pct70Pct80Pct90Pct95Pct98Pct99Pattern(client, 'sth_capitalized_price'),
          lthRealizedPrice: createPct0Pct1Pct10Pct2Pct20Pct30Pct40Pct5Pct50Pct60Pct70Pct80Pct90Pct95Pct98Pct99Pattern(client, 'lth_realized_price'),
          lthCapitalizedPrice: createPct0Pct1Pct10Pct2Pct20Pct30Pct40Pct5Pct50Pct60Pct70Pct80Pct90Pct95Pct98Pct99Pattern(client, 'lth_capitalized_price'),
          over6mRealizedPrice: createPct0Pct1Pct10Pct2Pct20Pct30Pct40Pct5Pct50Pct60Pct70Pct80Pct90Pct95Pct98Pct99Pattern(client, 'over_6m_realized_price'),
          over4mRealizedPrice: createPct0Pct1Pct10Pct2Pct20Pct30Pct40Pct5Pct50Pct60Pct70Pct80Pct90Pct95Pct98Pct99Pattern(client, 'over_4m_realized_price'),
          under4mRealizedPrice: createPct0Pct1Pct10Pct2Pct20Pct30Pct40Pct5Pct50Pct60Pct70Pct80Pct90Pct95Pct98Pct99Pattern(client, 'under_4m_realized_price'),
          under6mRealizedPrice: createPct0Pct1Pct10Pct2Pct20Pct30Pct40Pct5Pct50Pct60Pct70Pct80Pct90Pct95Pct98Pct99Pattern(client, 'under_6m_realized_price'),
          under4mCapitalizedPrice: createPct0Pct1Pct10Pct2Pct20Pct30Pct40Pct5Pct50Pct60Pct70Pct80Pct90Pct95Pct98Pct99Pattern(client, 'under_4m_capitalized_price'),
          under6mCapitalizedPrice: createPct0Pct1Pct10Pct2Pct20Pct30Pct40Pct5Pct50Pct60Pct70Pct80Pct90Pct95Pct98Pct99Pattern(client, 'under_6m_capitalized_price'),
          vaultedPrice: createPct0Pct1Pct10Pct2Pct20Pct30Pct40Pct5Pct50Pct60Pct70Pct80Pct90Pct95Pct98Pct99Pattern(client, 'vaulted_price'),
          activePrice: createPct0Pct1Pct10Pct2Pct20Pct30Pct40Pct5Pct50Pct60Pct70Pct80Pct90Pct95Pct98Pct99Pattern(client, 'active_price'),
          trueMarketMeanPrice: createPct0Pct1Pct10Pct2Pct20Pct30Pct40Pct5Pct50Pct60Pct70Pct80Pct90Pct95Pct98Pct99Pattern(client, 'true_market_mean_price'),
          cointimePrice: createPct0Pct1Pct10Pct2Pct20Pct30Pct40Pct5Pct50Pct60Pct70Pct80Pct90Pct95Pct98Pct99Pattern(client, 'cointime_price'),
          awakePrice: createPct0Pct1Pct10Pct2Pct20Pct30Pct40Pct5Pct50Pct60Pct70Pct80Pct90Pct95Pct98Pct99Pattern(client, 'awake_price'),
          coinflowPrice: createPct0Pct1Pct10Pct2Pct20Pct30Pct40Pct5Pct50Pct60Pct70Pct80Pct90Pct95Pct98Pct99Pattern(client, 'coinflow_price'),
        })); },
        get extremes() { return _lazy(this, 'extremes', () => ({
          get coinsInLoss() { return _lazy(this, 'coinsInLoss', () => ({
            thresholdPct01: createSeriesPattern1(client, 'rarity_meter_coins_in_loss_threshold_pct0_1'),
            thresholdPct005: createSeriesPattern1(client, 'rarity_meter_coins_in_loss_threshold_pct0_05'),
            thresholdPct0025: createSeriesPattern1(client, 'rarity_meter_coins_in_loss_threshold'),
            tail: createPercentPpmRatioPattern2(client, 'rarity_meter_coins_in_loss_tail'),
            rank: createSeriesPattern1(client, 'rarity_meter_coins_in_loss_rank'),
          })); },
          profitTaking: createRankTailThresholdPattern(client, 'rarity_meter_profit_taking'),
          capitulation: createRankTailThresholdPattern(client, 'rarity_meter_capitulation'),
          peakRegret: createRankTailThresholdPattern(client, 'rarity_meter_peak_regret'),
          get sellerExhaustion() { return _lazy(this, 'sellerExhaustion', () => ({
            thresholdPct01: createSeriesPattern1(client, 'rarity_meter_seller_exhaustion_threshold_pct0_1'),
            thresholdPct005: createSeriesPattern1(client, 'rarity_meter_seller_exhaustion_threshold_pct0_05'),
            thresholdPct0025: createSeriesPattern1(client, 'rarity_meter_seller_exhaustion_threshold'),
            tail: createPercentPpmRatioPattern2(client, 'rarity_meter_seller_exhaustion_tail'),
            rank: createSeriesPattern1(client, 'rarity_meter_seller_exhaustion_rank'),
          })); },
        })); },
        full: createIndexPct0Pct1Pct10Pct2Pct20Pct30Pct40Pct5Pct50Pct60Pct70Pct80Pct90Pct95Pct98Pct99ScorePattern(client, 'rarity_meter'),
        fullV2: createIndexPct0Pct1Pct10Pct2Pct20Pct30Pct40Pct5Pct50Pct60Pct70Pct80Pct90Pct95Pct98Pct99ScorePattern(client, 'rarity_meter_v2'),
        local: createIndexPct0Pct1Pct10Pct2Pct20Pct30Pct40Pct5Pct50Pct60Pct70Pct80Pct90Pct95Pct98Pct99ScorePattern(client, 'local_rarity_meter'),
        localV2: createIndexPct0Pct1Pct10Pct2Pct20Pct30Pct40Pct5Pct50Pct60Pct70Pct80Pct90Pct95Pct98Pct99ScorePattern(client, 'local_rarity_meter_v2'),
        cycle: createIndexPct0Pct1Pct10Pct2Pct20Pct30Pct40Pct5Pct50Pct60Pct70Pct80Pct90Pct95Pct98Pct99ScorePattern(client, 'cycle_rarity_meter'),
        cycleV2: createIndexPct0Pct1Pct10Pct2Pct20Pct30Pct40Pct5Pct50Pct60Pct70Pct80Pct90Pct95Pct98Pct99ScorePattern(client, 'cycle_rarity_meter_v2'),
      })); },
      get constants() { return _lazy(this, 'constants', () => ({
        _0: createSeriesPattern1(client, 'constant_0'),
        _1: createSeriesPattern1(client, 'constant_1'),
        _2: createSeriesPattern1(client, 'constant_2'),
        _3: createSeriesPattern1(client, 'constant_3'),
        _4: createSeriesPattern1(client, 'constant_4'),
        _20: createSeriesPattern1(client, 'constant_20'),
        _30: createSeriesPattern1(client, 'constant_30'),
        _382: createSeriesPattern1(client, 'constant_38_2'),
        _50: createSeriesPattern1(client, 'constant_50'),
        _618: createSeriesPattern1(client, 'constant_61_8'),
        _70: createSeriesPattern1(client, 'constant_70'),
        _80: createSeriesPattern1(client, 'constant_80'),
        _100: createSeriesPattern1(client, 'constant_100'),
        _600: createSeriesPattern1(client, 'constant_600'),
        minus1: createSeriesPattern1(client, 'constant_minus_1'),
        minus2: createSeriesPattern1(client, 'constant_minus_2'),
        minus3: createSeriesPattern1(client, 'constant_minus_3'),
        minus4: createSeriesPattern1(client, 'constant_minus_4'),
      })); },
      get mappings() { return _lazy(this, 'mappings', () => ({
        get addr() { return _lazy(this, 'addr', () => ({
          get p2pk33() { return _lazy(this, 'p2pk33', () => ({
            identity: createSeriesPattern26(client, 'p2pk33_addr_index'),
            addr: createSeriesPattern26(client, 'p2pk33_addr'),
          })); },
          get p2pk65() { return _lazy(this, 'p2pk65', () => ({
            identity: createSeriesPattern27(client, 'p2pk65_addr_index'),
            addr: createSeriesPattern27(client, 'p2pk65_addr'),
          })); },
          get p2pkh() { return _lazy(this, 'p2pkh', () => ({
            identity: createSeriesPattern28(client, 'p2pkh_addr_index'),
            addr: createSeriesPattern28(client, 'p2pkh_addr'),
          })); },
          get p2sh() { return _lazy(this, 'p2sh', () => ({
            identity: createSeriesPattern29(client, 'p2sh_addr_index'),
            addr: createSeriesPattern29(client, 'p2sh_addr'),
          })); },
          get p2tr() { return _lazy(this, 'p2tr', () => ({
            identity: createSeriesPattern30(client, 'p2tr_addr_index'),
            addr: createSeriesPattern30(client, 'p2tr_addr'),
          })); },
          get p2wpkh() { return _lazy(this, 'p2wpkh', () => ({
            identity: createSeriesPattern31(client, 'p2wpkh_addr_index'),
            addr: createSeriesPattern31(client, 'p2wpkh_addr'),
          })); },
          get p2wsh() { return _lazy(this, 'p2wsh', () => ({
            identity: createSeriesPattern32(client, 'p2wsh_addr_index'),
            addr: createSeriesPattern32(client, 'p2wsh_addr'),
          })); },
          get p2a() { return _lazy(this, 'p2a', () => ({
            identity: createSeriesPattern24(client, 'p2a_addr_index'),
            addr: createSeriesPattern24(client, 'p2a_addr'),
          })); },
          get p2ms() { return _lazy(this, 'p2ms', () => ({
            identity: createSeriesPattern25(client, 'p2ms_output_index'),
          })); },
          get empty() { return _lazy(this, 'empty', () => ({
            identity: createSeriesPattern22(client, 'empty_output_index'),
          })); },
          get unknown() { return _lazy(this, 'unknown', () => ({
            identity: createSeriesPattern33(client, 'unknown_output_index'),
          })); },
          get opReturn() { return _lazy(this, 'opReturn', () => ({
            identity: createSeriesPattern23(client, 'op_return_index'),
          })); },
        })); },
        get height() { return _lazy(this, 'height', () => ({
          minute10: createSeriesPattern18(client, 'minute10'),
          minute30: createSeriesPattern18(client, 'minute30'),
          hour1: createSeriesPattern18(client, 'hour1'),
          hour4: createSeriesPattern18(client, 'hour4'),
          hour12: createSeriesPattern18(client, 'hour12'),
          day1: createSeriesPattern18(client, 'day1'),
          day3: createSeriesPattern18(client, 'day3'),
          epoch: createSeriesPattern18(client, 'epoch'),
          halving: createSeriesPattern18(client, 'halving'),
          week1: createSeriesPattern18(client, 'week1'),
          month1: createSeriesPattern18(client, 'month1'),
          month3: createSeriesPattern18(client, 'month3'),
          month6: createSeriesPattern18(client, 'month6'),
          year1: createSeriesPattern18(client, 'year1'),
          year10: createSeriesPattern18(client, 'year10'),
          txIndexCount: createSeriesPattern18(client, 'tx_index_count'),
        })); },
        get epoch() { return _lazy(this, 'epoch', () => ({
          firstHeight: createSeriesPattern17(client, 'first_height'),
        })); },
        get halving() { return _lazy(this, 'halving', () => ({
          firstHeight: createSeriesPattern16(client, 'first_height'),
        })); },
        get minute10() { return _lazy(this, 'minute10', () => ({
          firstHeight: createSeriesPattern3(client, 'first_height'),
        })); },
        get minute30() { return _lazy(this, 'minute30', () => ({
          firstHeight: createSeriesPattern4(client, 'first_height'),
        })); },
        get hour1() { return _lazy(this, 'hour1', () => ({
          firstHeight: createSeriesPattern5(client, 'first_height'),
        })); },
        get hour4() { return _lazy(this, 'hour4', () => ({
          firstHeight: createSeriesPattern6(client, 'first_height'),
        })); },
        get hour12() { return _lazy(this, 'hour12', () => ({
          firstHeight: createSeriesPattern7(client, 'first_height'),
        })); },
        get day1() { return _lazy(this, 'day1', () => ({
          date: createSeriesPattern8(client, 'date'),
          firstHeight: createSeriesPattern8(client, 'first_height'),
        })); },
        get day3() { return _lazy(this, 'day3', () => ({
          date: createSeriesPattern9(client, 'date'),
          firstHeight: createSeriesPattern9(client, 'first_height'),
        })); },
        get week1() { return _lazy(this, 'week1', () => ({
          date: createSeriesPattern10(client, 'date'),
          firstHeight: createSeriesPattern10(client, 'first_height'),
        })); },
        get month1() { return _lazy(this, 'month1', () => ({
          date: createSeriesPattern11(client, 'date'),
          firstHeight: createSeriesPattern11(client, 'first_height'),
        })); },
        get month3() { return _lazy(this, 'month3', () => ({
          date: createSeriesPattern12(client, 'date'),
          firstHeight: createSeriesPattern12(client, 'first_height'),
        })); },
        get month6() { return _lazy(this, 'month6', () => ({
          date: createSeriesPattern13(client, 'date'),
          firstHeight: createSeriesPattern13(client, 'first_height'),
        })); },
        get year1() { return _lazy(this, 'year1', () => ({
          date: createSeriesPattern14(client, 'date'),
          firstHeight: createSeriesPattern14(client, 'first_height'),
        })); },
        get year10() { return _lazy(this, 'year10', () => ({
          date: createSeriesPattern15(client, 'date'),
          firstHeight: createSeriesPattern15(client, 'first_height'),
        })); },
        get txIndex() { return _lazy(this, 'txIndex', () => ({
          identity: createSeriesPattern19(client, 'tx_index'),
          inputCount: createSeriesPattern19(client, 'input_count'),
          outputCount: createSeriesPattern19(client, 'output_count'),
        })); },
        get txinIndex() { return _lazy(this, 'txinIndex', () => ({
          identity: createSeriesPattern20(client, 'txin_index'),
        })); },
        get txoutIndex() { return _lazy(this, 'txoutIndex', () => ({
          identity: createSeriesPattern21(client, 'txout_index'),
        })); },
        get timestamp() { return _lazy(this, 'timestamp', () => ({
          monotonic: createSeriesPattern18(client, 'timestamp_monotonic'),
          resolutions: createSeriesPattern2(client, 'timestamp'),
        })); },
      })); },
      get indicators() { return _lazy(this, 'indicators', () => ({
        puellMultiple: createBpsRatioPattern(client, 'puell_multiple'),
        nvt: createBpsRatioPattern(client, 'nvt'),
        gini: createPercentPpmRatioPattern2(client, 'gini'),
        get rhodlRatio() { return _lazy(this, 'rhodlRatio', () => ({
          ppm: createSeriesPattern1(client, 'rhodl_ratio_ppm'),
          ratio: createSeriesPattern1(client, 'rhodl_ratio'),
        })); },
        thermoCapMultiple: createBpsRatioPattern(client, 'thermo_cap_multiple'),
        coindaysDestroyedSupplyAdj: createSeriesPattern1(client, 'coindays_destroyed_supply_adj'),
        coinyearsDestroyedSupplyAdj: createSeriesPattern1(client, 'coinyears_destroyed_supply_adj'),
        get dormancy() { return _lazy(this, 'dormancy', () => ({
          supplyAdj: createSeriesPattern1(client, 'dormancy_supply_adj'),
          flow: createSeriesPattern1(client, 'dormancy_flow'),
        })); },
        stockToFlow: createSeriesPattern1(client, 'stock_to_flow'),
        sellerExhaustion: createSeriesPattern1(client, 'seller_exhaustion'),
      })); },
      get market() { return _lazy(this, 'market', () => ({
        get ath() { return _lazy(this, 'ath', () => ({
          high: createCentsSatsUsdPattern(client, 'price_ath'),
          drawdown: createPercentPpmRatioPattern3(client, 'price_drawdown'),
          daysSince: createSeriesPattern1(client, 'days_since_price_ath'),
          yearsSince: createSeriesPattern1(client, 'years_since_price_ath'),
          maxDaysBetween: createSeriesPattern1(client, 'max_days_between_price_ath'),
          maxYearsBetween: createSeriesPattern1(client, 'max_years_between_price_ath'),
        })); },
        get lookback() { return _lazy(this, 'lookback', () => ({
          _24h: createCentsSatsUsdPattern(client, 'price_past_24h'),
          _1w: createCentsSatsUsdPattern(client, 'price_past_1w'),
          _1m: createCentsSatsUsdPattern(client, 'price_past_1m'),
          _3m: createCentsSatsUsdPattern(client, 'price_past_3m'),
          _6m: createCentsSatsUsdPattern(client, 'price_past_6m'),
          _1y: createCentsSatsUsdPattern(client, 'price_past_1y'),
          _2y: createCentsSatsUsdPattern(client, 'price_past_2y'),
          _3y: createCentsSatsUsdPattern(client, 'price_past_3y'),
          _4y: createCentsSatsUsdPattern(client, 'price_past_4y'),
          _5y: createCentsSatsUsdPattern(client, 'price_past_5y'),
          _6y: createCentsSatsUsdPattern(client, 'price_past_6y'),
          _8y: createCentsSatsUsdPattern(client, 'price_past_8y'),
          _10y: createCentsSatsUsdPattern(client, 'price_past_10y'),
        })); },
        get returns() { return _lazy(this, 'returns', () => ({
          get periods() { return _lazy(this, 'periods', () => ({
            _24h: createPercentPpmRatioPattern(client, 'price_return_24h'),
            _1w: createPercentPpmRatioPattern(client, 'price_return_1w'),
            _1m: createPercentPpmRatioPattern(client, 'price_return_1m'),
            _3m: createPercentPpmRatioPattern(client, 'price_return_3m'),
            _6m: createPercentPpmRatioPattern(client, 'price_return_6m'),
            _1y: createPercentPpmRatioPattern(client, 'price_return_1y'),
            _2y: createPercentPpmRatioPattern(client, 'price_return_2y'),
            _3y: createPercentPpmRatioPattern(client, 'price_return_3y'),
            _4y: createPercentPpmRatioPattern(client, 'price_return_4y'),
            _5y: createPercentPpmRatioPattern(client, 'price_return_5y'),
            _6y: createPercentPpmRatioPattern(client, 'price_return_6y'),
            _8y: createPercentPpmRatioPattern(client, 'price_return_8y'),
            _10y: createPercentPpmRatioPattern(client, 'price_return_10y'),
          })); },
          get cagr() { return _lazy(this, 'cagr', () => ({
            _2y: createPercentPpmRatioPattern(client, 'price_cagr_2y'),
            _3y: createPercentPpmRatioPattern(client, 'price_cagr_3y'),
            _4y: createPercentPpmRatioPattern(client, 'price_cagr_4y'),
            _5y: createPercentPpmRatioPattern(client, 'price_cagr_5y'),
            _6y: createPercentPpmRatioPattern(client, 'price_cagr_6y'),
            _8y: createPercentPpmRatioPattern(client, 'price_cagr_8y'),
            _10y: createPercentPpmRatioPattern(client, 'price_cagr_10y'),
          })); },
          get sd24h() { return _lazy(this, 'sd24h', () => ({
            get _24h() { return _lazy(this, '_24h', () => ({
              sma: createSeriesPattern1(client, 'price_return_24h_sma_24h'),
              sd: createSeriesPattern1(client, 'price_return_24h_sd_24h'),
            })); },
            get _1w() { return _lazy(this, '_1w', () => ({
              sma: createSeriesPattern1(client, 'price_return_24h_sma_1w'),
              sd: createSeriesPattern1(client, 'price_return_24h_sd_1w'),
            })); },
            get _1m() { return _lazy(this, '_1m', () => ({
              sma: createSeriesPattern1(client, 'price_return_24h_sma_1m'),
              sd: createSeriesPattern1(client, 'price_return_24h_sd_1m'),
            })); },
            get _1y() { return _lazy(this, '_1y', () => ({
              sma: createSeriesPattern1(client, 'price_return_24h_sma_1y'),
              sd: createSeriesPattern1(client, 'price_return_24h_sd_1y'),
            })); },
          })); },
        })); },
        volatility: create_1m1w1y24hPattern6(client, 'price_volatility'),
        get range() { return _lazy(this, 'range', () => ({
          min: create_1m1w1y2wPattern(client, 'price_min'),
          max: create_1m1w1y2wPattern(client, 'price_max'),
          trueRange: createSeriesPattern1(client, 'price_true_range'),
          trueRangeSum2w: createSeriesPattern1(client, 'price_true_range_sum_2w'),
          choppinessIndex2w: createPercentPpmRatioPattern2(client, 'price_choppiness_index_2w'),
        })); },
        get movingAverage() { return _lazy(this, 'movingAverage', () => ({
          get sma() { return _lazy(this, 'sma', () => ({
            _1w: createCentsPpmRatioSatsUsdPattern(client, 'price_sma_1w'),
            _8d: createCentsPpmRatioSatsUsdPattern(client, 'price_sma_8d'),
            _13d: createCentsPpmRatioSatsUsdPattern(client, 'price_sma_13d'),
            _21d: createCentsPpmRatioSatsUsdPattern(client, 'price_sma_21d'),
            _1m: createCentsPpmRatioSatsUsdPattern(client, 'price_sma_1m'),
            _34d: createCentsPpmRatioSatsUsdPattern(client, 'price_sma_34d'),
            _50d: createCentsPpmRatioSatsUsdPattern(client, 'price_sma_50d'),
            _55d: createCentsPpmRatioSatsUsdPattern(client, 'price_sma_55d'),
            _89d: createCentsPpmRatioSatsUsdPattern(client, 'price_sma_89d'),
            _111d: createCentsPpmRatioSatsUsdPattern(client, 'price_sma_111d'),
            _144d: createCentsPpmRatioSatsUsdPattern(client, 'price_sma_144d'),
            get _200d() { return _lazy(this, '_200d', () => ({
              usd: createSeriesPattern1(client, 'price_sma_200d'),
              cents: createSeriesPattern1(client, 'price_sma_200d_cents'),
              sats: createSeriesPattern1(client, 'price_sma_200d_sats'),
              ppm: createSeriesPattern1(client, 'price_sma_200d_ratio_ppm'),
              ratio: createSeriesPattern1(client, 'price_sma_200d_ratio'),
              x24: createCentsSatsUsdPattern(client, 'price_sma_200d_x2_4'),
              x08: createCentsSatsUsdPattern(client, 'price_sma_200d_x0_8'),
            })); },
            get _350d() { return _lazy(this, '_350d', () => ({
              usd: createSeriesPattern1(client, 'price_sma_350d'),
              cents: createSeriesPattern1(client, 'price_sma_350d_cents'),
              sats: createSeriesPattern1(client, 'price_sma_350d_sats'),
              ppm: createSeriesPattern1(client, 'price_sma_350d_ratio_ppm'),
              ratio: createSeriesPattern1(client, 'price_sma_350d_ratio'),
              x2: createCentsSatsUsdPattern(client, 'price_sma_350d_x2'),
            })); },
            _1y: createCentsPpmRatioSatsUsdPattern(client, 'price_sma_1y'),
            _2y: createCentsPpmRatioSatsUsdPattern(client, 'price_sma_2y'),
            _200w: createCentsPpmRatioSatsUsdPattern(client, 'price_sma_200w'),
            _4y: createCentsPpmRatioSatsUsdPattern(client, 'price_sma_4y'),
          })); },
          get ema() { return _lazy(this, 'ema', () => ({
            _1w: createCentsPpmRatioSatsUsdPattern(client, 'price_ema_1w'),
            _8d: createCentsPpmRatioSatsUsdPattern(client, 'price_ema_8d'),
            _12d: createCentsPpmRatioSatsUsdPattern(client, 'price_ema_12d'),
            _13d: createCentsPpmRatioSatsUsdPattern(client, 'price_ema_13d'),
            _21d: createCentsPpmRatioSatsUsdPattern(client, 'price_ema_21d'),
            _26d: createCentsPpmRatioSatsUsdPattern(client, 'price_ema_26d'),
            _1m: createCentsPpmRatioSatsUsdPattern(client, 'price_ema_1m'),
            _34d: createCentsPpmRatioSatsUsdPattern(client, 'price_ema_34d'),
            _55d: createCentsPpmRatioSatsUsdPattern(client, 'price_ema_55d'),
            _89d: createCentsPpmRatioSatsUsdPattern(client, 'price_ema_89d'),
            _144d: createCentsPpmRatioSatsUsdPattern(client, 'price_ema_144d'),
            _200d: createCentsPpmRatioSatsUsdPattern(client, 'price_ema_200d'),
            _1y: createCentsPpmRatioSatsUsdPattern(client, 'price_ema_1y'),
            _2y: createCentsPpmRatioSatsUsdPattern(client, 'price_ema_2y'),
            _200w: createCentsPpmRatioSatsUsdPattern(client, 'price_ema_200w'),
            _4y: createCentsPpmRatioSatsUsdPattern(client, 'price_ema_4y'),
          })); },
        })); },
        get technical() { return _lazy(this, 'technical', () => ({
          get rsi() { return _lazy(this, 'rsi', () => ({
            _24h: createRsiStochPattern(client, 'rsi', '24h'),
            _1w: createRsiStochPattern(client, 'rsi', '1w'),
            _1m: createRsiStochPattern(client, 'rsi', '1m'),
          })); },
          piCycle: createPpmRatioPattern(client, 'pi_cycle'),
          get macd() { return _lazy(this, 'macd', () => ({
            get _24h() { return _lazy(this, '_24h', () => ({
              emaFast: createSeriesPattern1(client, 'macd_ema_fast_24h'),
              emaSlow: createSeriesPattern1(client, 'macd_ema_slow_24h'),
              line: createSeriesPattern1(client, 'macd_line_24h'),
              signal: createSeriesPattern1(client, 'macd_signal_24h'),
              histogram: createSeriesPattern1(client, 'macd_histogram_24h'),
            })); },
            get _1w() { return _lazy(this, '_1w', () => ({
              emaFast: createSeriesPattern1(client, 'macd_ema_fast_1w'),
              emaSlow: createSeriesPattern1(client, 'macd_ema_slow_1w'),
              line: createSeriesPattern1(client, 'macd_line_1w'),
              signal: createSeriesPattern1(client, 'macd_signal_1w'),
              histogram: createSeriesPattern1(client, 'macd_histogram_1w'),
            })); },
            get _1m() { return _lazy(this, '_1m', () => ({
              emaFast: createSeriesPattern1(client, 'macd_ema_fast_1m'),
              emaSlow: createSeriesPattern1(client, 'macd_ema_slow_1m'),
              line: createSeriesPattern1(client, 'macd_line_1m'),
              signal: createSeriesPattern1(client, 'macd_signal_1m'),
              histogram: createSeriesPattern1(client, 'macd_histogram_1m'),
            })); },
          })); },
        })); },
      })); },
      get pools() { return _lazy(this, 'pools', () => ({
        pool: createSeriesPattern18(client, 'pool'),
        get major() { return _lazy(this, 'major', () => ({
          unknown: createBlocksDominanceRewardsPattern(client, 'unknown'),
          luxor: createBlocksDominanceRewardsPattern(client, 'luxor'),
          btccom: createBlocksDominanceRewardsPattern(client, 'btccom'),
          btctop: createBlocksDominanceRewardsPattern(client, 'btctop'),
          btcguild: createBlocksDominanceRewardsPattern(client, 'btcguild'),
          eligius: createBlocksDominanceRewardsPattern(client, 'eligius'),
          f2pool: createBlocksDominanceRewardsPattern(client, 'f2pool'),
          braiinspool: createBlocksDominanceRewardsPattern(client, 'braiinspool'),
          antpool: createBlocksDominanceRewardsPattern(client, 'antpool'),
          btcc: createBlocksDominanceRewardsPattern(client, 'btcc'),
          bwpool: createBlocksDominanceRewardsPattern(client, 'bwpool'),
          bitfury: createBlocksDominanceRewardsPattern(client, 'bitfury'),
          viabtc: createBlocksDominanceRewardsPattern(client, 'viabtc'),
          poolin: createBlocksDominanceRewardsPattern(client, 'poolin'),
          spiderpool: createBlocksDominanceRewardsPattern(client, 'spiderpool'),
          binancepool: createBlocksDominanceRewardsPattern(client, 'binancepool'),
          foundryusa: createBlocksDominanceRewardsPattern(client, 'foundryusa'),
          sbicrypto: createBlocksDominanceRewardsPattern(client, 'sbicrypto'),
          marapool: createBlocksDominanceRewardsPattern(client, 'marapool'),
          secpool: createBlocksDominanceRewardsPattern(client, 'secpool'),
          ocean: createBlocksDominanceRewardsPattern(client, 'ocean'),
          whitepool: createBlocksDominanceRewardsPattern(client, 'whitepool'),
        })); },
        get minor() { return _lazy(this, 'minor', () => ({
          blockfills: createBlocksDominancePattern(client, 'blockfills'),
          ultimuspool: createBlocksDominancePattern(client, 'ultimuspool'),
          terrapool: createBlocksDominancePattern(client, 'terrapool'),
          onethash: createBlocksDominancePattern(client, 'onethash'),
          bitfarms: createBlocksDominancePattern(client, 'bitfarms'),
          huobipool: createBlocksDominancePattern(client, 'huobipool'),
          wayicn: createBlocksDominancePattern(client, 'wayicn'),
          canoepool: createBlocksDominancePattern(client, 'canoepool'),
          bitcoincom: createBlocksDominancePattern(client, 'bitcoincom'),
          pool175btc: createBlocksDominancePattern(client, 'pool175btc'),
          gbminers: createBlocksDominancePattern(client, 'gbminers'),
          axbt: createBlocksDominancePattern(client, 'axbt'),
          asicminer: createBlocksDominancePattern(client, 'asicminer'),
          bitminter: createBlocksDominancePattern(client, 'bitminter'),
          bitcoinrussia: createBlocksDominancePattern(client, 'bitcoinrussia'),
          btcserv: createBlocksDominancePattern(client, 'btcserv'),
          simplecoinus: createBlocksDominancePattern(client, 'simplecoinus'),
          ozcoin: createBlocksDominancePattern(client, 'ozcoin'),
          eclipsemc: createBlocksDominancePattern(client, 'eclipsemc'),
          maxbtc: createBlocksDominancePattern(client, 'maxbtc'),
          triplemining: createBlocksDominancePattern(client, 'triplemining'),
          coinlab: createBlocksDominancePattern(client, 'coinlab'),
          pool50btc: createBlocksDominancePattern(client, 'pool50btc'),
          ghashio: createBlocksDominancePattern(client, 'ghashio'),
          stminingcorp: createBlocksDominancePattern(client, 'stminingcorp'),
          bitparking: createBlocksDominancePattern(client, 'bitparking'),
          mmpool: createBlocksDominancePattern(client, 'mmpool'),
          polmine: createBlocksDominancePattern(client, 'polmine'),
          kncminer: createBlocksDominancePattern(client, 'kncminer'),
          bitalo: createBlocksDominancePattern(client, 'bitalo'),
          hhtt: createBlocksDominancePattern(client, 'hhtt'),
          megabigpower: createBlocksDominancePattern(client, 'megabigpower'),
          mtred: createBlocksDominancePattern(client, 'mtred'),
          nmcbit: createBlocksDominancePattern(client, 'nmcbit'),
          yourbtcnet: createBlocksDominancePattern(client, 'yourbtcnet'),
          givemecoins: createBlocksDominancePattern(client, 'givemecoins'),
          multicoinco: createBlocksDominancePattern(client, 'multicoinco'),
          bcpoolio: createBlocksDominancePattern(client, 'bcpoolio'),
          cointerra: createBlocksDominancePattern(client, 'cointerra'),
          kanopool: createBlocksDominancePattern(client, 'kanopool'),
          solock: createBlocksDominancePattern(client, 'solock'),
          ckpool: createBlocksDominancePattern(client, 'ckpool'),
          nicehash: createBlocksDominancePattern(client, 'nicehash'),
          bitclub: createBlocksDominancePattern(client, 'bitclub'),
          bitcoinaffiliatenetwork: createBlocksDominancePattern(client, 'bitcoinaffiliatenetwork'),
          exxbw: createBlocksDominancePattern(client, 'exxbw'),
          bitsolo: createBlocksDominancePattern(client, 'bitsolo'),
          twentyoneinc: createBlocksDominancePattern(client, 'twentyoneinc'),
          digitalbtc: createBlocksDominancePattern(client, 'digitalbtc'),
          eightbaochi: createBlocksDominancePattern(client, 'eightbaochi'),
          mybtccoinpool: createBlocksDominancePattern(client, 'mybtccoinpool'),
          tbdice: createBlocksDominancePattern(client, 'tbdice'),
          hashpool: createBlocksDominancePattern(client, 'hashpool'),
          nexious: createBlocksDominancePattern(client, 'nexious'),
          bravomining: createBlocksDominancePattern(client, 'bravomining'),
          hotpool: createBlocksDominancePattern(client, 'hotpool'),
          okexpool: createBlocksDominancePattern(client, 'okexpool'),
          bcmonster: createBlocksDominancePattern(client, 'bcmonster'),
          onehash: createBlocksDominancePattern(client, 'onehash'),
          bixin: createBlocksDominancePattern(client, 'bixin'),
          tatmaspool: createBlocksDominancePattern(client, 'tatmaspool'),
          connectbtc: createBlocksDominancePattern(client, 'connectbtc'),
          batpool: createBlocksDominancePattern(client, 'batpool'),
          waterhole: createBlocksDominancePattern(client, 'waterhole'),
          dcexploration: createBlocksDominancePattern(client, 'dcexploration'),
          dcex: createBlocksDominancePattern(client, 'dcex'),
          btpool: createBlocksDominancePattern(client, 'btpool'),
          fiftyeightcoin: createBlocksDominancePattern(client, 'fiftyeightcoin'),
          bitcoinindia: createBlocksDominancePattern(client, 'bitcoinindia'),
          shawnp0wers: createBlocksDominancePattern(client, 'shawnp0wers'),
          phashio: createBlocksDominancePattern(client, 'phashio'),
          rigpool: createBlocksDominancePattern(client, 'rigpool'),
          haozhuzhu: createBlocksDominancePattern(client, 'haozhuzhu'),
          sevenpool: createBlocksDominancePattern(client, 'sevenpool'),
          miningkings: createBlocksDominancePattern(client, 'miningkings'),
          hashbx: createBlocksDominancePattern(client, 'hashbx'),
          dpool: createBlocksDominancePattern(client, 'dpool'),
          rawpool: createBlocksDominancePattern(client, 'rawpool'),
          haominer: createBlocksDominancePattern(client, 'haominer'),
          helix: createBlocksDominancePattern(client, 'helix'),
          bitcoinukraine: createBlocksDominancePattern(client, 'bitcoinukraine'),
          secretsuperstar: createBlocksDominancePattern(client, 'secretsuperstar'),
          tigerpoolnet: createBlocksDominancePattern(client, 'tigerpoolnet'),
          sigmapoolcom: createBlocksDominancePattern(client, 'sigmapoolcom'),
          okpooltop: createBlocksDominancePattern(client, 'okpooltop'),
          hummerpool: createBlocksDominancePattern(client, 'hummerpool'),
          tangpool: createBlocksDominancePattern(client, 'tangpool'),
          bytepool: createBlocksDominancePattern(client, 'bytepool'),
          novablock: createBlocksDominancePattern(client, 'novablock'),
          miningcity: createBlocksDominancePattern(client, 'miningcity'),
          minerium: createBlocksDominancePattern(client, 'minerium'),
          lubiancom: createBlocksDominancePattern(client, 'lubiancom'),
          okkong: createBlocksDominancePattern(client, 'okkong'),
          aaopool: createBlocksDominancePattern(client, 'aaopool'),
          emcdpool: createBlocksDominancePattern(client, 'emcdpool'),
          arkpool: createBlocksDominancePattern(client, 'arkpool'),
          purebtccom: createBlocksDominancePattern(client, 'purebtccom'),
          kucoinpool: createBlocksDominancePattern(client, 'kucoinpool'),
          entrustcharitypool: createBlocksDominancePattern(client, 'entrustcharitypool'),
          okminer: createBlocksDominancePattern(client, 'okminer'),
          titan: createBlocksDominancePattern(client, 'titan'),
          pegapool: createBlocksDominancePattern(client, 'pegapool'),
          btcnuggets: createBlocksDominancePattern(client, 'btcnuggets'),
          cloudhashing: createBlocksDominancePattern(client, 'cloudhashing'),
          digitalxmintsy: createBlocksDominancePattern(client, 'digitalxmintsy'),
          telco214: createBlocksDominancePattern(client, 'telco214'),
          btcpoolparty: createBlocksDominancePattern(client, 'btcpoolparty'),
          multipool: createBlocksDominancePattern(client, 'multipool'),
          transactioncoinmining: createBlocksDominancePattern(client, 'transactioncoinmining'),
          btcdig: createBlocksDominancePattern(client, 'btcdig'),
          trickysbtcpool: createBlocksDominancePattern(client, 'trickysbtcpool'),
          btcmp: createBlocksDominancePattern(client, 'btcmp'),
          eobot: createBlocksDominancePattern(client, 'eobot'),
          unomp: createBlocksDominancePattern(client, 'unomp'),
          patels: createBlocksDominancePattern(client, 'patels'),
          gogreenlight: createBlocksDominancePattern(client, 'gogreenlight'),
          bitcoinindiapool: createBlocksDominancePattern(client, 'bitcoinindiapool'),
          ekanembtc: createBlocksDominancePattern(client, 'ekanembtc'),
          canoe: createBlocksDominancePattern(client, 'canoe'),
          tiger: createBlocksDominancePattern(client, 'tiger'),
          onem1x: createBlocksDominancePattern(client, 'onem1x'),
          zulupool: createBlocksDominancePattern(client, 'zulupool'),
          wiz: createBlocksDominancePattern(client, 'wiz'),
          wk057: createBlocksDominancePattern(client, 'wk057'),
          futurebitapollosolo: createBlocksDominancePattern(client, 'futurebitapollosolo'),
          carbonnegative: createBlocksDominancePattern(client, 'carbonnegative'),
          portlandhodl: createBlocksDominancePattern(client, 'portlandhodl'),
          phoenix: createBlocksDominancePattern(client, 'phoenix'),
          neopool: createBlocksDominancePattern(client, 'neopool'),
          maxipool: createBlocksDominancePattern(client, 'maxipool'),
          bitfufupool: createBlocksDominancePattern(client, 'bitfufupool'),
          gdpool: createBlocksDominancePattern(client, 'gdpool'),
          miningdutch: createBlocksDominancePattern(client, 'miningdutch'),
          publicpool: createBlocksDominancePattern(client, 'publicpool'),
          miningsquared: createBlocksDominancePattern(client, 'miningsquared'),
          innopolistech: createBlocksDominancePattern(client, 'innopolistech'),
          btclab: createBlocksDominancePattern(client, 'btclab'),
          parasite: createBlocksDominancePattern(client, 'parasite'),
          redrockpool: createBlocksDominancePattern(client, 'redrockpool'),
          est3lar: createBlocksDominancePattern(client, 'est3lar'),
          braiinssolo: createBlocksDominancePattern(client, 'braiinssolo'),
          solopool: createBlocksDominancePattern(client, 'solopool'),
          noderunners: createBlocksDominancePattern(client, 'noderunners'),
          dmnd: createBlocksDominancePattern(client, 'dmnd'),
        })); },
      })); },
      get price() { return _lazy(this, 'price', () => ({
        get split() { return _lazy(this, 'split', () => ({
          open: createCentsSatsUsdPattern3(client, 'price_open'),
          high: createCentsSatsUsdPattern3(client, 'price_high'),
          low: createCentsSatsUsdPattern3(client, 'price_low'),
          close: createCentsSatsUsdPattern3(client, 'price_close'),
        })); },
        get ohlc() { return _lazy(this, 'ohlc', () => ({
          usd: createSeriesPattern2(client, 'price_ohlc'),
          cents: createSeriesPattern2(client, 'price_ohlc_cents'),
          sats: createSeriesPattern2(client, 'price_ohlc_sats'),
        })); },
        get spot() { return _lazy(this, 'spot', () => ({
          usd: createSeriesPattern1(client, 'price'),
          cents: createSeriesPattern1(client, 'price_cents'),
          sats: createSeriesPattern1(client, 'price_sats'),
        })); },
      })); },
      get cohorts() { return _lazy(this, 'cohorts', () => ({
        get supply() { return _lazy(this, 'supply', () => ({
          get total() { return _lazy(this, 'total', () => ({
            get age() { return _lazy(this, 'age', () => ({
              under1h: createBtcCentsSatsUsdPattern(client, 'utxos_under_1h_old_supply'),
              _1hTo1d: createBtcCentsSatsUsdPattern(client, 'utxos_1h_to_1d_old_supply'),
              _1dTo1w: createBtcCentsSatsUsdPattern(client, 'utxos_1d_to_1w_old_supply'),
              _1wTo1m: createBtcCentsSatsUsdPattern(client, 'utxos_1w_to_1m_old_supply'),
              _1mTo2m: createBtcCentsSatsUsdPattern(client, 'utxos_1m_to_2m_old_supply'),
              _2mTo3m: createBtcCentsSatsUsdPattern(client, 'utxos_2m_to_3m_old_supply'),
              _3mTo4m: createBtcCentsSatsUsdPattern(client, 'utxos_3m_to_4m_old_supply'),
              _4mTo5m: createBtcCentsSatsUsdPattern(client, 'utxos_4m_to_5m_old_supply'),
              _5mTo6m: createBtcCentsSatsUsdPattern(client, 'utxos_5m_to_6m_old_supply'),
              _6mTo9m: createBtcCentsSatsUsdPattern(client, 'utxos_6m_to_9m_old_supply'),
              _9mTo1y: createBtcCentsSatsUsdPattern(client, 'utxos_9m_to_1y_old_supply'),
              _1yTo18m: createBtcCentsSatsUsdPattern(client, 'utxos_1y_to_18m_old_supply'),
              _18mTo2y: createBtcCentsSatsUsdPattern(client, 'utxos_18m_to_2y_old_supply'),
              _2yTo3y: createBtcCentsSatsUsdPattern(client, 'utxos_2y_to_3y_old_supply'),
              _3yTo4y: createBtcCentsSatsUsdPattern(client, 'utxos_3y_to_4y_old_supply'),
              _4yTo5y: createBtcCentsSatsUsdPattern(client, 'utxos_4y_to_5y_old_supply'),
              _5yTo6y: createBtcCentsSatsUsdPattern(client, 'utxos_5y_to_6y_old_supply'),
              _6yTo7y: createBtcCentsSatsUsdPattern(client, 'utxos_6y_to_7y_old_supply'),
              _7yTo8y: createBtcCentsSatsUsdPattern(client, 'utxos_7y_to_8y_old_supply'),
              _8yTo10y: createBtcCentsSatsUsdPattern(client, 'utxos_8y_to_10y_old_supply'),
              _10yTo12y: createBtcCentsSatsUsdPattern(client, 'utxos_10y_to_12y_old_supply'),
              _12yTo15y: createBtcCentsSatsUsdPattern(client, 'utxos_12y_to_15y_old_supply'),
              over15y: createBtcCentsSatsUsdPattern(client, 'utxos_over_15y_old_supply'),
            })); },
            get epoch() { return _lazy(this, 'epoch', () => ({
              _0: createBtcCentsSatsUsdPattern(client, 'epoch_0_supply'),
              _1: createBtcCentsSatsUsdPattern(client, 'epoch_1_supply'),
              _2: createBtcCentsSatsUsdPattern(client, 'epoch_2_supply'),
              _3: createBtcCentsSatsUsdPattern(client, 'epoch_3_supply'),
              _4: createBtcCentsSatsUsdPattern(client, 'epoch_4_supply'),
            })); },
            get class() { return _lazy(this, 'class', () => ({
              _2009: createBtcCentsSatsUsdPattern(client, 'class_2009_supply'),
              _2010: createBtcCentsSatsUsdPattern(client, 'class_2010_supply'),
              _2011: createBtcCentsSatsUsdPattern(client, 'class_2011_supply'),
              _2012: createBtcCentsSatsUsdPattern(client, 'class_2012_supply'),
              _2013: createBtcCentsSatsUsdPattern(client, 'class_2013_supply'),
              _2014: createBtcCentsSatsUsdPattern(client, 'class_2014_supply'),
              _2015: createBtcCentsSatsUsdPattern(client, 'class_2015_supply'),
              _2016: createBtcCentsSatsUsdPattern(client, 'class_2016_supply'),
              _2017: createBtcCentsSatsUsdPattern(client, 'class_2017_supply'),
              _2018: createBtcCentsSatsUsdPattern(client, 'class_2018_supply'),
              _2019: createBtcCentsSatsUsdPattern(client, 'class_2019_supply'),
              _2020: createBtcCentsSatsUsdPattern(client, 'class_2020_supply'),
              _2021: createBtcCentsSatsUsdPattern(client, 'class_2021_supply'),
              _2022: createBtcCentsSatsUsdPattern(client, 'class_2022_supply'),
              _2023: createBtcCentsSatsUsdPattern(client, 'class_2023_supply'),
              _2024: createBtcCentsSatsUsdPattern(client, 'class_2024_supply'),
              _2025: createBtcCentsSatsUsdPattern(client, 'class_2025_supply'),
              _2026: createBtcCentsSatsUsdPattern(client, 'class_2026_supply'),
            })); },
            get utxoAmount() { return _lazy(this, 'utxoAmount', () => ({
              _0sats: createBtcCentsSatsUsdPattern(client, 'utxos_0sats_supply'),
              _1satTo10sats: createBtcCentsSatsUsdPattern(client, 'utxos_1sat_to_10sats_supply'),
              _10satsTo100sats: createBtcCentsSatsUsdPattern(client, 'utxos_10sats_to_100sats_supply'),
              _100satsTo1kSats: createBtcCentsSatsUsdPattern(client, 'utxos_100sats_to_1k_sats_supply'),
              _1kSatsTo10kSats: createBtcCentsSatsUsdPattern(client, 'utxos_1k_sats_to_10k_sats_supply'),
              _10kSatsTo100kSats: createBtcCentsSatsUsdPattern(client, 'utxos_10k_sats_to_100k_sats_supply'),
              _100kSatsTo1mSats: createBtcCentsSatsUsdPattern(client, 'utxos_100k_sats_to_1m_sats_supply'),
              _1mSatsTo10mSats: createBtcCentsSatsUsdPattern(client, 'utxos_1m_sats_to_10m_sats_supply'),
              _10mSatsTo1btc: createBtcCentsSatsUsdPattern(client, 'utxos_10m_sats_to_1btc_supply'),
              _1btcTo10btc: createBtcCentsSatsUsdPattern(client, 'utxos_1btc_to_10btc_supply'),
              _10btcTo100btc: createBtcCentsSatsUsdPattern(client, 'utxos_10btc_to_100btc_supply'),
              _100btcTo1kBtc: createBtcCentsSatsUsdPattern(client, 'utxos_100btc_to_1k_btc_supply'),
              _1kBtcTo10kBtc: createBtcCentsSatsUsdPattern(client, 'utxos_1k_btc_to_10k_btc_supply'),
              _10kBtcTo100kBtc: createBtcCentsSatsUsdPattern(client, 'utxos_10k_btc_to_100k_btc_supply'),
              over100kBtc: createBtcCentsSatsUsdPattern(client, 'utxos_over_100k_btc_supply'),
            })); },
            type: createEmptyP2aP2msP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshUnknownPattern2(client, 'supply'),
          })); },
          get matured() { return _lazy(this, 'matured', () => ({
            under1h: createAverageBlockCumulativeSumPattern(client, 'utxos_under_1h_old_matured_supply'),
            _1hTo1d: createAverageBlockCumulativeSumPattern(client, 'utxos_1h_to_1d_old_matured_supply'),
            _1dTo1w: createAverageBlockCumulativeSumPattern(client, 'utxos_1d_to_1w_old_matured_supply'),
            _1wTo1m: createAverageBlockCumulativeSumPattern(client, 'utxos_1w_to_1m_old_matured_supply'),
            _1mTo2m: createAverageBlockCumulativeSumPattern(client, 'utxos_1m_to_2m_old_matured_supply'),
            _2mTo3m: createAverageBlockCumulativeSumPattern(client, 'utxos_2m_to_3m_old_matured_supply'),
            _3mTo4m: createAverageBlockCumulativeSumPattern(client, 'utxos_3m_to_4m_old_matured_supply'),
            _4mTo5m: createAverageBlockCumulativeSumPattern(client, 'utxos_4m_to_5m_old_matured_supply'),
            _5mTo6m: createAverageBlockCumulativeSumPattern(client, 'utxos_5m_to_6m_old_matured_supply'),
            _6mTo9m: createAverageBlockCumulativeSumPattern(client, 'utxos_6m_to_9m_old_matured_supply'),
            _9mTo1y: createAverageBlockCumulativeSumPattern(client, 'utxos_9m_to_1y_old_matured_supply'),
            _1yTo18m: createAverageBlockCumulativeSumPattern(client, 'utxos_1y_to_18m_old_matured_supply'),
            _18mTo2y: createAverageBlockCumulativeSumPattern(client, 'utxos_18m_to_2y_old_matured_supply'),
            _2yTo3y: createAverageBlockCumulativeSumPattern(client, 'utxos_2y_to_3y_old_matured_supply'),
            _3yTo4y: createAverageBlockCumulativeSumPattern(client, 'utxos_3y_to_4y_old_matured_supply'),
            _4yTo5y: createAverageBlockCumulativeSumPattern(client, 'utxos_4y_to_5y_old_matured_supply'),
            _5yTo6y: createAverageBlockCumulativeSumPattern(client, 'utxos_5y_to_6y_old_matured_supply'),
            _6yTo7y: createAverageBlockCumulativeSumPattern(client, 'utxos_6y_to_7y_old_matured_supply'),
            _7yTo8y: createAverageBlockCumulativeSumPattern(client, 'utxos_7y_to_8y_old_matured_supply'),
            _8yTo10y: createAverageBlockCumulativeSumPattern(client, 'utxos_8y_to_10y_old_matured_supply'),
            _10yTo12y: createAverageBlockCumulativeSumPattern(client, 'utxos_10y_to_12y_old_matured_supply'),
            _12yTo15y: createAverageBlockCumulativeSumPattern(client, 'utxos_12y_to_15y_old_matured_supply'),
            over15y: createAverageBlockCumulativeSumPattern(client, 'utxos_over_15y_old_matured_supply'),
          })); },
          get inProfit() { return _lazy(this, 'inProfit', () => ({
            get age() { return _lazy(this, 'age', () => ({
              under1h: createBtcCentsSatsUsdPattern(client, 'utxos_under_1h_old_supply_in_profit'),
              _1hTo1d: createBtcCentsSatsUsdPattern(client, 'utxos_1h_to_1d_old_supply_in_profit'),
              _1dTo1w: createBtcCentsSatsUsdPattern(client, 'utxos_1d_to_1w_old_supply_in_profit'),
              _1wTo1m: createBtcCentsSatsUsdPattern(client, 'utxos_1w_to_1m_old_supply_in_profit'),
              _1mTo2m: createBtcCentsSatsUsdPattern(client, 'utxos_1m_to_2m_old_supply_in_profit'),
              _2mTo3m: createBtcCentsSatsUsdPattern(client, 'utxos_2m_to_3m_old_supply_in_profit'),
              _3mTo4m: createBtcCentsSatsUsdPattern(client, 'utxos_3m_to_4m_old_supply_in_profit'),
              _4mTo5m: createBtcCentsSatsUsdPattern(client, 'utxos_4m_to_5m_old_supply_in_profit'),
              _5mTo6m: createBtcCentsSatsUsdPattern(client, 'utxos_5m_to_6m_old_supply_in_profit'),
              _6mTo9m: createBtcCentsSatsUsdPattern(client, 'utxos_6m_to_9m_old_supply_in_profit'),
              _9mTo1y: createBtcCentsSatsUsdPattern(client, 'utxos_9m_to_1y_old_supply_in_profit'),
              _1yTo18m: createBtcCentsSatsUsdPattern(client, 'utxos_1y_to_18m_old_supply_in_profit'),
              _18mTo2y: createBtcCentsSatsUsdPattern(client, 'utxos_18m_to_2y_old_supply_in_profit'),
              _2yTo3y: createBtcCentsSatsUsdPattern(client, 'utxos_2y_to_3y_old_supply_in_profit'),
              _3yTo4y: createBtcCentsSatsUsdPattern(client, 'utxos_3y_to_4y_old_supply_in_profit'),
              _4yTo5y: createBtcCentsSatsUsdPattern(client, 'utxos_4y_to_5y_old_supply_in_profit'),
              _5yTo6y: createBtcCentsSatsUsdPattern(client, 'utxos_5y_to_6y_old_supply_in_profit'),
              _6yTo7y: createBtcCentsSatsUsdPattern(client, 'utxos_6y_to_7y_old_supply_in_profit'),
              _7yTo8y: createBtcCentsSatsUsdPattern(client, 'utxos_7y_to_8y_old_supply_in_profit'),
              _8yTo10y: createBtcCentsSatsUsdPattern(client, 'utxos_8y_to_10y_old_supply_in_profit'),
              _10yTo12y: createBtcCentsSatsUsdPattern(client, 'utxos_10y_to_12y_old_supply_in_profit'),
              _12yTo15y: createBtcCentsSatsUsdPattern(client, 'utxos_12y_to_15y_old_supply_in_profit'),
              over15y: createBtcCentsSatsUsdPattern(client, 'utxos_over_15y_old_supply_in_profit'),
            })); },
            get epoch() { return _lazy(this, 'epoch', () => ({
              _0: createBtcCentsSatsUsdPattern(client, 'epoch_0_supply_in_profit'),
              _1: createBtcCentsSatsUsdPattern(client, 'epoch_1_supply_in_profit'),
              _2: createBtcCentsSatsUsdPattern(client, 'epoch_2_supply_in_profit'),
              _3: createBtcCentsSatsUsdPattern(client, 'epoch_3_supply_in_profit'),
              _4: createBtcCentsSatsUsdPattern(client, 'epoch_4_supply_in_profit'),
            })); },
            get class() { return _lazy(this, 'class', () => ({
              _2009: createBtcCentsSatsUsdPattern(client, 'class_2009_supply_in_profit'),
              _2010: createBtcCentsSatsUsdPattern(client, 'class_2010_supply_in_profit'),
              _2011: createBtcCentsSatsUsdPattern(client, 'class_2011_supply_in_profit'),
              _2012: createBtcCentsSatsUsdPattern(client, 'class_2012_supply_in_profit'),
              _2013: createBtcCentsSatsUsdPattern(client, 'class_2013_supply_in_profit'),
              _2014: createBtcCentsSatsUsdPattern(client, 'class_2014_supply_in_profit'),
              _2015: createBtcCentsSatsUsdPattern(client, 'class_2015_supply_in_profit'),
              _2016: createBtcCentsSatsUsdPattern(client, 'class_2016_supply_in_profit'),
              _2017: createBtcCentsSatsUsdPattern(client, 'class_2017_supply_in_profit'),
              _2018: createBtcCentsSatsUsdPattern(client, 'class_2018_supply_in_profit'),
              _2019: createBtcCentsSatsUsdPattern(client, 'class_2019_supply_in_profit'),
              _2020: createBtcCentsSatsUsdPattern(client, 'class_2020_supply_in_profit'),
              _2021: createBtcCentsSatsUsdPattern(client, 'class_2021_supply_in_profit'),
              _2022: createBtcCentsSatsUsdPattern(client, 'class_2022_supply_in_profit'),
              _2023: createBtcCentsSatsUsdPattern(client, 'class_2023_supply_in_profit'),
              _2024: createBtcCentsSatsUsdPattern(client, 'class_2024_supply_in_profit'),
              _2025: createBtcCentsSatsUsdPattern(client, 'class_2025_supply_in_profit'),
              _2026: createBtcCentsSatsUsdPattern(client, 'class_2026_supply_in_profit'),
            })); },
          })); },
          get inLoss() { return _lazy(this, 'inLoss', () => ({
            get age() { return _lazy(this, 'age', () => ({
              under1h: createBtcCentsSatsUsdPattern(client, 'utxos_under_1h_old_supply_in_loss'),
              _1hTo1d: createBtcCentsSatsUsdPattern(client, 'utxos_1h_to_1d_old_supply_in_loss'),
              _1dTo1w: createBtcCentsSatsUsdPattern(client, 'utxos_1d_to_1w_old_supply_in_loss'),
              _1wTo1m: createBtcCentsSatsUsdPattern(client, 'utxos_1w_to_1m_old_supply_in_loss'),
              _1mTo2m: createBtcCentsSatsUsdPattern(client, 'utxos_1m_to_2m_old_supply_in_loss'),
              _2mTo3m: createBtcCentsSatsUsdPattern(client, 'utxos_2m_to_3m_old_supply_in_loss'),
              _3mTo4m: createBtcCentsSatsUsdPattern(client, 'utxos_3m_to_4m_old_supply_in_loss'),
              _4mTo5m: createBtcCentsSatsUsdPattern(client, 'utxos_4m_to_5m_old_supply_in_loss'),
              _5mTo6m: createBtcCentsSatsUsdPattern(client, 'utxos_5m_to_6m_old_supply_in_loss'),
              _6mTo9m: createBtcCentsSatsUsdPattern(client, 'utxos_6m_to_9m_old_supply_in_loss'),
              _9mTo1y: createBtcCentsSatsUsdPattern(client, 'utxos_9m_to_1y_old_supply_in_loss'),
              _1yTo18m: createBtcCentsSatsUsdPattern(client, 'utxos_1y_to_18m_old_supply_in_loss'),
              _18mTo2y: createBtcCentsSatsUsdPattern(client, 'utxos_18m_to_2y_old_supply_in_loss'),
              _2yTo3y: createBtcCentsSatsUsdPattern(client, 'utxos_2y_to_3y_old_supply_in_loss'),
              _3yTo4y: createBtcCentsSatsUsdPattern(client, 'utxos_3y_to_4y_old_supply_in_loss'),
              _4yTo5y: createBtcCentsSatsUsdPattern(client, 'utxos_4y_to_5y_old_supply_in_loss'),
              _5yTo6y: createBtcCentsSatsUsdPattern(client, 'utxos_5y_to_6y_old_supply_in_loss'),
              _6yTo7y: createBtcCentsSatsUsdPattern(client, 'utxos_6y_to_7y_old_supply_in_loss'),
              _7yTo8y: createBtcCentsSatsUsdPattern(client, 'utxos_7y_to_8y_old_supply_in_loss'),
              _8yTo10y: createBtcCentsSatsUsdPattern(client, 'utxos_8y_to_10y_old_supply_in_loss'),
              _10yTo12y: createBtcCentsSatsUsdPattern(client, 'utxos_10y_to_12y_old_supply_in_loss'),
              _12yTo15y: createBtcCentsSatsUsdPattern(client, 'utxos_12y_to_15y_old_supply_in_loss'),
              over15y: createBtcCentsSatsUsdPattern(client, 'utxos_over_15y_old_supply_in_loss'),
            })); },
            get epoch() { return _lazy(this, 'epoch', () => ({
              _0: createBtcCentsSatsUsdPattern(client, 'epoch_0_supply_in_loss'),
              _1: createBtcCentsSatsUsdPattern(client, 'epoch_1_supply_in_loss'),
              _2: createBtcCentsSatsUsdPattern(client, 'epoch_2_supply_in_loss'),
              _3: createBtcCentsSatsUsdPattern(client, 'epoch_3_supply_in_loss'),
              _4: createBtcCentsSatsUsdPattern(client, 'epoch_4_supply_in_loss'),
            })); },
            get class() { return _lazy(this, 'class', () => ({
              _2009: createBtcCentsSatsUsdPattern(client, 'class_2009_supply_in_loss'),
              _2010: createBtcCentsSatsUsdPattern(client, 'class_2010_supply_in_loss'),
              _2011: createBtcCentsSatsUsdPattern(client, 'class_2011_supply_in_loss'),
              _2012: createBtcCentsSatsUsdPattern(client, 'class_2012_supply_in_loss'),
              _2013: createBtcCentsSatsUsdPattern(client, 'class_2013_supply_in_loss'),
              _2014: createBtcCentsSatsUsdPattern(client, 'class_2014_supply_in_loss'),
              _2015: createBtcCentsSatsUsdPattern(client, 'class_2015_supply_in_loss'),
              _2016: createBtcCentsSatsUsdPattern(client, 'class_2016_supply_in_loss'),
              _2017: createBtcCentsSatsUsdPattern(client, 'class_2017_supply_in_loss'),
              _2018: createBtcCentsSatsUsdPattern(client, 'class_2018_supply_in_loss'),
              _2019: createBtcCentsSatsUsdPattern(client, 'class_2019_supply_in_loss'),
              _2020: createBtcCentsSatsUsdPattern(client, 'class_2020_supply_in_loss'),
              _2021: createBtcCentsSatsUsdPattern(client, 'class_2021_supply_in_loss'),
              _2022: createBtcCentsSatsUsdPattern(client, 'class_2022_supply_in_loss'),
              _2023: createBtcCentsSatsUsdPattern(client, 'class_2023_supply_in_loss'),
              _2024: createBtcCentsSatsUsdPattern(client, 'class_2024_supply_in_loss'),
              _2025: createBtcCentsSatsUsdPattern(client, 'class_2025_supply_in_loss'),
              _2026: createBtcCentsSatsUsdPattern(client, 'class_2026_supply_in_loss'),
            })); },
          })); },
          get delta() { return _lazy(this, 'delta', () => ({
            get age() { return _lazy(this, 'age', () => ({
              under1h: createAbsoluteRatePattern(client, 'utxos_under_1h_old_supply_delta'),
              _1hTo1d: createAbsoluteRatePattern(client, 'utxos_1h_to_1d_old_supply_delta'),
              _1dTo1w: createAbsoluteRatePattern(client, 'utxos_1d_to_1w_old_supply_delta'),
              _1wTo1m: createAbsoluteRatePattern(client, 'utxos_1w_to_1m_old_supply_delta'),
              _1mTo2m: createAbsoluteRatePattern(client, 'utxos_1m_to_2m_old_supply_delta'),
              _2mTo3m: createAbsoluteRatePattern(client, 'utxos_2m_to_3m_old_supply_delta'),
              _3mTo4m: createAbsoluteRatePattern(client, 'utxos_3m_to_4m_old_supply_delta'),
              _4mTo5m: createAbsoluteRatePattern(client, 'utxos_4m_to_5m_old_supply_delta'),
              _5mTo6m: createAbsoluteRatePattern(client, 'utxos_5m_to_6m_old_supply_delta'),
              _6mTo9m: createAbsoluteRatePattern(client, 'utxos_6m_to_9m_old_supply_delta'),
              _9mTo1y: createAbsoluteRatePattern(client, 'utxos_9m_to_1y_old_supply_delta'),
              _1yTo18m: createAbsoluteRatePattern(client, 'utxos_1y_to_18m_old_supply_delta'),
              _18mTo2y: createAbsoluteRatePattern(client, 'utxos_18m_to_2y_old_supply_delta'),
              _2yTo3y: createAbsoluteRatePattern(client, 'utxos_2y_to_3y_old_supply_delta'),
              _3yTo4y: createAbsoluteRatePattern(client, 'utxos_3y_to_4y_old_supply_delta'),
              _4yTo5y: createAbsoluteRatePattern(client, 'utxos_4y_to_5y_old_supply_delta'),
              _5yTo6y: createAbsoluteRatePattern(client, 'utxos_5y_to_6y_old_supply_delta'),
              _6yTo7y: createAbsoluteRatePattern(client, 'utxos_6y_to_7y_old_supply_delta'),
              _7yTo8y: createAbsoluteRatePattern(client, 'utxos_7y_to_8y_old_supply_delta'),
              _8yTo10y: createAbsoluteRatePattern(client, 'utxos_8y_to_10y_old_supply_delta'),
              _10yTo12y: createAbsoluteRatePattern(client, 'utxos_10y_to_12y_old_supply_delta'),
              _12yTo15y: createAbsoluteRatePattern(client, 'utxos_12y_to_15y_old_supply_delta'),
              over15y: createAbsoluteRatePattern(client, 'utxos_over_15y_old_supply_delta'),
            })); },
            get epoch() { return _lazy(this, 'epoch', () => ({
              _0: createAbsoluteRatePattern(client, 'epoch_0_supply_delta'),
              _1: createAbsoluteRatePattern(client, 'epoch_1_supply_delta'),
              _2: createAbsoluteRatePattern(client, 'epoch_2_supply_delta'),
              _3: createAbsoluteRatePattern(client, 'epoch_3_supply_delta'),
              _4: createAbsoluteRatePattern(client, 'epoch_4_supply_delta'),
            })); },
            get class() { return _lazy(this, 'class', () => ({
              _2009: createAbsoluteRatePattern(client, 'class_2009_supply_delta'),
              _2010: createAbsoluteRatePattern(client, 'class_2010_supply_delta'),
              _2011: createAbsoluteRatePattern(client, 'class_2011_supply_delta'),
              _2012: createAbsoluteRatePattern(client, 'class_2012_supply_delta'),
              _2013: createAbsoluteRatePattern(client, 'class_2013_supply_delta'),
              _2014: createAbsoluteRatePattern(client, 'class_2014_supply_delta'),
              _2015: createAbsoluteRatePattern(client, 'class_2015_supply_delta'),
              _2016: createAbsoluteRatePattern(client, 'class_2016_supply_delta'),
              _2017: createAbsoluteRatePattern(client, 'class_2017_supply_delta'),
              _2018: createAbsoluteRatePattern(client, 'class_2018_supply_delta'),
              _2019: createAbsoluteRatePattern(client, 'class_2019_supply_delta'),
              _2020: createAbsoluteRatePattern(client, 'class_2020_supply_delta'),
              _2021: createAbsoluteRatePattern(client, 'class_2021_supply_delta'),
              _2022: createAbsoluteRatePattern(client, 'class_2022_supply_delta'),
              _2023: createAbsoluteRatePattern(client, 'class_2023_supply_delta'),
              _2024: createAbsoluteRatePattern(client, 'class_2024_supply_delta'),
              _2025: createAbsoluteRatePattern(client, 'class_2025_supply_delta'),
              _2026: createAbsoluteRatePattern(client, 'class_2026_supply_delta'),
            })); },
            get utxoAmount() { return _lazy(this, 'utxoAmount', () => ({
              _0sats: createAbsoluteRatePattern(client, 'utxos_0sats_supply_delta'),
              _1satTo10sats: createAbsoluteRatePattern(client, 'utxos_1sat_to_10sats_supply_delta'),
              _10satsTo100sats: createAbsoluteRatePattern(client, 'utxos_10sats_to_100sats_supply_delta'),
              _100satsTo1kSats: createAbsoluteRatePattern(client, 'utxos_100sats_to_1k_sats_supply_delta'),
              _1kSatsTo10kSats: createAbsoluteRatePattern(client, 'utxos_1k_sats_to_10k_sats_supply_delta'),
              _10kSatsTo100kSats: createAbsoluteRatePattern(client, 'utxos_10k_sats_to_100k_sats_supply_delta'),
              _100kSatsTo1mSats: createAbsoluteRatePattern(client, 'utxos_100k_sats_to_1m_sats_supply_delta'),
              _1mSatsTo10mSats: createAbsoluteRatePattern(client, 'utxos_1m_sats_to_10m_sats_supply_delta'),
              _10mSatsTo1btc: createAbsoluteRatePattern(client, 'utxos_10m_sats_to_1btc_supply_delta'),
              _1btcTo10btc: createAbsoluteRatePattern(client, 'utxos_1btc_to_10btc_supply_delta'),
              _10btcTo100btc: createAbsoluteRatePattern(client, 'utxos_10btc_to_100btc_supply_delta'),
              _100btcTo1kBtc: createAbsoluteRatePattern(client, 'utxos_100btc_to_1k_btc_supply_delta'),
              _1kBtcTo10kBtc: createAbsoluteRatePattern(client, 'utxos_1k_btc_to_10k_btc_supply_delta'),
              _10kBtcTo100kBtc: createAbsoluteRatePattern(client, 'utxos_10k_btc_to_100k_btc_supply_delta'),
              over100kBtc: createAbsoluteRatePattern(client, 'utxos_over_100k_btc_supply_delta'),
            })); },
            get type() { return _lazy(this, 'type', () => ({
              p2pk65: createAbsoluteRatePattern(client, 'p2pk65_supply_delta'),
              p2pk33: createAbsoluteRatePattern(client, 'p2pk33_supply_delta'),
              p2pkh: createAbsoluteRatePattern(client, 'p2pkh_supply_delta'),
              p2ms: createAbsoluteRatePattern(client, 'p2ms_supply_delta'),
              p2sh: createAbsoluteRatePattern(client, 'p2sh_supply_delta'),
              p2wpkh: createAbsoluteRatePattern(client, 'p2wpkh_supply_delta'),
              p2wsh: createAbsoluteRatePattern(client, 'p2wsh_supply_delta'),
              p2tr: createAbsoluteRatePattern(client, 'p2tr_supply_delta'),
              p2a: createAbsoluteRatePattern(client, 'p2a_supply_delta'),
              unknown: createAbsoluteRatePattern(client, 'unknown_outputs_supply_delta'),
              empty: createAbsoluteRatePattern(client, 'empty_outputs_supply_delta'),
            })); },
          })); },
          get dominance() { return _lazy(this, 'dominance', () => ({
            get age() { return _lazy(this, 'age', () => ({
              under1h: createPercentPpmRatioPattern2(client, 'utxos_under_1h_old_supply_dominance'),
              _1hTo1d: createPercentPpmRatioPattern2(client, 'utxos_1h_to_1d_old_supply_dominance'),
              _1dTo1w: createPercentPpmRatioPattern2(client, 'utxos_1d_to_1w_old_supply_dominance'),
              _1wTo1m: createPercentPpmRatioPattern2(client, 'utxos_1w_to_1m_old_supply_dominance'),
              _1mTo2m: createPercentPpmRatioPattern2(client, 'utxos_1m_to_2m_old_supply_dominance'),
              _2mTo3m: createPercentPpmRatioPattern2(client, 'utxos_2m_to_3m_old_supply_dominance'),
              _3mTo4m: createPercentPpmRatioPattern2(client, 'utxos_3m_to_4m_old_supply_dominance'),
              _4mTo5m: createPercentPpmRatioPattern2(client, 'utxos_4m_to_5m_old_supply_dominance'),
              _5mTo6m: createPercentPpmRatioPattern2(client, 'utxos_5m_to_6m_old_supply_dominance'),
              _6mTo9m: createPercentPpmRatioPattern2(client, 'utxos_6m_to_9m_old_supply_dominance'),
              _9mTo1y: createPercentPpmRatioPattern2(client, 'utxos_9m_to_1y_old_supply_dominance'),
              _1yTo18m: createPercentPpmRatioPattern2(client, 'utxos_1y_to_18m_old_supply_dominance'),
              _18mTo2y: createPercentPpmRatioPattern2(client, 'utxos_18m_to_2y_old_supply_dominance'),
              _2yTo3y: createPercentPpmRatioPattern2(client, 'utxos_2y_to_3y_old_supply_dominance'),
              _3yTo4y: createPercentPpmRatioPattern2(client, 'utxos_3y_to_4y_old_supply_dominance'),
              _4yTo5y: createPercentPpmRatioPattern2(client, 'utxos_4y_to_5y_old_supply_dominance'),
              _5yTo6y: createPercentPpmRatioPattern2(client, 'utxos_5y_to_6y_old_supply_dominance'),
              _6yTo7y: createPercentPpmRatioPattern2(client, 'utxos_6y_to_7y_old_supply_dominance'),
              _7yTo8y: createPercentPpmRatioPattern2(client, 'utxos_7y_to_8y_old_supply_dominance'),
              _8yTo10y: createPercentPpmRatioPattern2(client, 'utxos_8y_to_10y_old_supply_dominance'),
              _10yTo12y: createPercentPpmRatioPattern2(client, 'utxos_10y_to_12y_old_supply_dominance'),
              _12yTo15y: createPercentPpmRatioPattern2(client, 'utxos_12y_to_15y_old_supply_dominance'),
              over15y: createPercentPpmRatioPattern2(client, 'utxos_over_15y_old_supply_dominance'),
            })); },
            get epoch() { return _lazy(this, 'epoch', () => ({
              _0: createPercentPpmRatioPattern2(client, 'epoch_0_supply_dominance'),
              _1: createPercentPpmRatioPattern2(client, 'epoch_1_supply_dominance'),
              _2: createPercentPpmRatioPattern2(client, 'epoch_2_supply_dominance'),
              _3: createPercentPpmRatioPattern2(client, 'epoch_3_supply_dominance'),
              _4: createPercentPpmRatioPattern2(client, 'epoch_4_supply_dominance'),
            })); },
            get class() { return _lazy(this, 'class', () => ({
              _2009: createPercentPpmRatioPattern2(client, 'class_2009_supply_dominance'),
              _2010: createPercentPpmRatioPattern2(client, 'class_2010_supply_dominance'),
              _2011: createPercentPpmRatioPattern2(client, 'class_2011_supply_dominance'),
              _2012: createPercentPpmRatioPattern2(client, 'class_2012_supply_dominance'),
              _2013: createPercentPpmRatioPattern2(client, 'class_2013_supply_dominance'),
              _2014: createPercentPpmRatioPattern2(client, 'class_2014_supply_dominance'),
              _2015: createPercentPpmRatioPattern2(client, 'class_2015_supply_dominance'),
              _2016: createPercentPpmRatioPattern2(client, 'class_2016_supply_dominance'),
              _2017: createPercentPpmRatioPattern2(client, 'class_2017_supply_dominance'),
              _2018: createPercentPpmRatioPattern2(client, 'class_2018_supply_dominance'),
              _2019: createPercentPpmRatioPattern2(client, 'class_2019_supply_dominance'),
              _2020: createPercentPpmRatioPattern2(client, 'class_2020_supply_dominance'),
              _2021: createPercentPpmRatioPattern2(client, 'class_2021_supply_dominance'),
              _2022: createPercentPpmRatioPattern2(client, 'class_2022_supply_dominance'),
              _2023: createPercentPpmRatioPattern2(client, 'class_2023_supply_dominance'),
              _2024: createPercentPpmRatioPattern2(client, 'class_2024_supply_dominance'),
              _2025: createPercentPpmRatioPattern2(client, 'class_2025_supply_dominance'),
              _2026: createPercentPpmRatioPattern2(client, 'class_2026_supply_dominance'),
            })); },
            get utxoAmount() { return _lazy(this, 'utxoAmount', () => ({
              _0sats: createPercentPpmRatioPattern2(client, 'utxos_0sats_supply_dominance'),
              _1satTo10sats: createPercentPpmRatioPattern2(client, 'utxos_1sat_to_10sats_supply_dominance'),
              _10satsTo100sats: createPercentPpmRatioPattern2(client, 'utxos_10sats_to_100sats_supply_dominance'),
              _100satsTo1kSats: createPercentPpmRatioPattern2(client, 'utxos_100sats_to_1k_sats_supply_dominance'),
              _1kSatsTo10kSats: createPercentPpmRatioPattern2(client, 'utxos_1k_sats_to_10k_sats_supply_dominance'),
              _10kSatsTo100kSats: createPercentPpmRatioPattern2(client, 'utxos_10k_sats_to_100k_sats_supply_dominance'),
              _100kSatsTo1mSats: createPercentPpmRatioPattern2(client, 'utxos_100k_sats_to_1m_sats_supply_dominance'),
              _1mSatsTo10mSats: createPercentPpmRatioPattern2(client, 'utxos_1m_sats_to_10m_sats_supply_dominance'),
              _10mSatsTo1btc: createPercentPpmRatioPattern2(client, 'utxos_10m_sats_to_1btc_supply_dominance'),
              _1btcTo10btc: createPercentPpmRatioPattern2(client, 'utxos_1btc_to_10btc_supply_dominance'),
              _10btcTo100btc: createPercentPpmRatioPattern2(client, 'utxos_10btc_to_100btc_supply_dominance'),
              _100btcTo1kBtc: createPercentPpmRatioPattern2(client, 'utxos_100btc_to_1k_btc_supply_dominance'),
              _1kBtcTo10kBtc: createPercentPpmRatioPattern2(client, 'utxos_1k_btc_to_10k_btc_supply_dominance'),
              _10kBtcTo100kBtc: createPercentPpmRatioPattern2(client, 'utxos_10k_btc_to_100k_btc_supply_dominance'),
              over100kBtc: createPercentPpmRatioPattern2(client, 'utxos_over_100k_btc_supply_dominance'),
            })); },
            get type() { return _lazy(this, 'type', () => ({
              p2pk65: createPercentPpmRatioPattern2(client, 'p2pk65_supply_dominance'),
              p2pk33: createPercentPpmRatioPattern2(client, 'p2pk33_supply_dominance'),
              p2pkh: createPercentPpmRatioPattern2(client, 'p2pkh_supply_dominance'),
              p2ms: createPercentPpmRatioPattern2(client, 'p2ms_supply_dominance'),
              p2sh: createPercentPpmRatioPattern2(client, 'p2sh_supply_dominance'),
              p2wpkh: createPercentPpmRatioPattern2(client, 'p2wpkh_supply_dominance'),
              p2wsh: createPercentPpmRatioPattern2(client, 'p2wsh_supply_dominance'),
              p2tr: createPercentPpmRatioPattern2(client, 'p2tr_supply_dominance'),
              p2a: createPercentPpmRatioPattern2(client, 'p2a_supply_dominance'),
              unknown: createPercentPpmRatioPattern2(client, 'unknown_outputs_supply_dominance'),
              empty: createPercentPpmRatioPattern2(client, 'empty_outputs_supply_dominance'),
            })); },
          })); },
        })); },
        get outputs() { return _lazy(this, 'outputs', () => ({
          get unspentCount() { return _lazy(this, 'unspentCount', () => ({
            get age() { return _lazy(this, 'age', () => ({
              under1h: createBaseDeltaPattern(client, 'utxos_under_1h_old_utxo_count'),
              _1hTo1d: createBaseDeltaPattern(client, 'utxos_1h_to_1d_old_utxo_count'),
              _1dTo1w: createBaseDeltaPattern(client, 'utxos_1d_to_1w_old_utxo_count'),
              _1wTo1m: createBaseDeltaPattern(client, 'utxos_1w_to_1m_old_utxo_count'),
              _1mTo2m: createBaseDeltaPattern(client, 'utxos_1m_to_2m_old_utxo_count'),
              _2mTo3m: createBaseDeltaPattern(client, 'utxos_2m_to_3m_old_utxo_count'),
              _3mTo4m: createBaseDeltaPattern(client, 'utxos_3m_to_4m_old_utxo_count'),
              _4mTo5m: createBaseDeltaPattern(client, 'utxos_4m_to_5m_old_utxo_count'),
              _5mTo6m: createBaseDeltaPattern(client, 'utxos_5m_to_6m_old_utxo_count'),
              _6mTo9m: createBaseDeltaPattern(client, 'utxos_6m_to_9m_old_utxo_count'),
              _9mTo1y: createBaseDeltaPattern(client, 'utxos_9m_to_1y_old_utxo_count'),
              _1yTo18m: createBaseDeltaPattern(client, 'utxos_1y_to_18m_old_utxo_count'),
              _18mTo2y: createBaseDeltaPattern(client, 'utxos_18m_to_2y_old_utxo_count'),
              _2yTo3y: createBaseDeltaPattern(client, 'utxos_2y_to_3y_old_utxo_count'),
              _3yTo4y: createBaseDeltaPattern(client, 'utxos_3y_to_4y_old_utxo_count'),
              _4yTo5y: createBaseDeltaPattern(client, 'utxos_4y_to_5y_old_utxo_count'),
              _5yTo6y: createBaseDeltaPattern(client, 'utxos_5y_to_6y_old_utxo_count'),
              _6yTo7y: createBaseDeltaPattern(client, 'utxos_6y_to_7y_old_utxo_count'),
              _7yTo8y: createBaseDeltaPattern(client, 'utxos_7y_to_8y_old_utxo_count'),
              _8yTo10y: createBaseDeltaPattern(client, 'utxos_8y_to_10y_old_utxo_count'),
              _10yTo12y: createBaseDeltaPattern(client, 'utxos_10y_to_12y_old_utxo_count'),
              _12yTo15y: createBaseDeltaPattern(client, 'utxos_12y_to_15y_old_utxo_count'),
              over15y: createBaseDeltaPattern(client, 'utxos_over_15y_old_utxo_count'),
            })); },
            get epoch() { return _lazy(this, 'epoch', () => ({
              _0: createBaseDeltaPattern(client, 'epoch_0_utxo_count'),
              _1: createBaseDeltaPattern(client, 'epoch_1_utxo_count'),
              _2: createBaseDeltaPattern(client, 'epoch_2_utxo_count'),
              _3: createBaseDeltaPattern(client, 'epoch_3_utxo_count'),
              _4: createBaseDeltaPattern(client, 'epoch_4_utxo_count'),
            })); },
            get class() { return _lazy(this, 'class', () => ({
              _2009: createBaseDeltaPattern(client, 'class_2009_utxo_count'),
              _2010: createBaseDeltaPattern(client, 'class_2010_utxo_count'),
              _2011: createBaseDeltaPattern(client, 'class_2011_utxo_count'),
              _2012: createBaseDeltaPattern(client, 'class_2012_utxo_count'),
              _2013: createBaseDeltaPattern(client, 'class_2013_utxo_count'),
              _2014: createBaseDeltaPattern(client, 'class_2014_utxo_count'),
              _2015: createBaseDeltaPattern(client, 'class_2015_utxo_count'),
              _2016: createBaseDeltaPattern(client, 'class_2016_utxo_count'),
              _2017: createBaseDeltaPattern(client, 'class_2017_utxo_count'),
              _2018: createBaseDeltaPattern(client, 'class_2018_utxo_count'),
              _2019: createBaseDeltaPattern(client, 'class_2019_utxo_count'),
              _2020: createBaseDeltaPattern(client, 'class_2020_utxo_count'),
              _2021: createBaseDeltaPattern(client, 'class_2021_utxo_count'),
              _2022: createBaseDeltaPattern(client, 'class_2022_utxo_count'),
              _2023: createBaseDeltaPattern(client, 'class_2023_utxo_count'),
              _2024: createBaseDeltaPattern(client, 'class_2024_utxo_count'),
              _2025: createBaseDeltaPattern(client, 'class_2025_utxo_count'),
              _2026: createBaseDeltaPattern(client, 'class_2026_utxo_count'),
            })); },
            get utxoAmount() { return _lazy(this, 'utxoAmount', () => ({
              _0sats: createBaseDeltaPattern(client, 'utxos_0sats_utxo_count'),
              _1satTo10sats: createBaseDeltaPattern(client, 'utxos_1sat_to_10sats_utxo_count'),
              _10satsTo100sats: createBaseDeltaPattern(client, 'utxos_10sats_to_100sats_utxo_count'),
              _100satsTo1kSats: createBaseDeltaPattern(client, 'utxos_100sats_to_1k_sats_utxo_count'),
              _1kSatsTo10kSats: createBaseDeltaPattern(client, 'utxos_1k_sats_to_10k_sats_utxo_count'),
              _10kSatsTo100kSats: createBaseDeltaPattern(client, 'utxos_10k_sats_to_100k_sats_utxo_count'),
              _100kSatsTo1mSats: createBaseDeltaPattern(client, 'utxos_100k_sats_to_1m_sats_utxo_count'),
              _1mSatsTo10mSats: createBaseDeltaPattern(client, 'utxos_1m_sats_to_10m_sats_utxo_count'),
              _10mSatsTo1btc: createBaseDeltaPattern(client, 'utxos_10m_sats_to_1btc_utxo_count'),
              _1btcTo10btc: createBaseDeltaPattern(client, 'utxos_1btc_to_10btc_utxo_count'),
              _10btcTo100btc: createBaseDeltaPattern(client, 'utxos_10btc_to_100btc_utxo_count'),
              _100btcTo1kBtc: createBaseDeltaPattern(client, 'utxos_100btc_to_1k_btc_utxo_count'),
              _1kBtcTo10kBtc: createBaseDeltaPattern(client, 'utxos_1k_btc_to_10k_btc_utxo_count'),
              _10kBtcTo100kBtc: createBaseDeltaPattern(client, 'utxos_10k_btc_to_100k_btc_utxo_count'),
              over100kBtc: createBaseDeltaPattern(client, 'utxos_over_100k_btc_utxo_count'),
            })); },
            get type() { return _lazy(this, 'type', () => ({
              p2pk65: createBaseDeltaPattern(client, 'p2pk65_utxo_count'),
              p2pk33: createBaseDeltaPattern(client, 'p2pk33_utxo_count'),
              p2pkh: createBaseDeltaPattern(client, 'p2pkh_utxo_count'),
              p2ms: createBaseDeltaPattern(client, 'p2ms_utxo_count'),
              p2sh: createBaseDeltaPattern(client, 'p2sh_utxo_count'),
              p2wpkh: createBaseDeltaPattern(client, 'p2wpkh_utxo_count'),
              p2wsh: createBaseDeltaPattern(client, 'p2wsh_utxo_count'),
              p2tr: createBaseDeltaPattern(client, 'p2tr_utxo_count'),
              p2a: createBaseDeltaPattern(client, 'p2a_utxo_count'),
              unknown: createBaseDeltaPattern(client, 'unknown_outputs_utxo_count'),
              empty: createBaseDeltaPattern(client, 'empty_outputs_utxo_count'),
            })); },
          })); },
          get spentCount() { return _lazy(this, 'spentCount', () => ({
            get age() { return _lazy(this, 'age', () => ({
              under1h: createAverageBlockCumulativeSumPattern2(client, 'utxos_under_1h_old_spent_utxo_count'),
              _1hTo1d: createAverageBlockCumulativeSumPattern2(client, 'utxos_1h_to_1d_old_spent_utxo_count'),
              _1dTo1w: createAverageBlockCumulativeSumPattern2(client, 'utxos_1d_to_1w_old_spent_utxo_count'),
              _1wTo1m: createAverageBlockCumulativeSumPattern2(client, 'utxos_1w_to_1m_old_spent_utxo_count'),
              _1mTo2m: createAverageBlockCumulativeSumPattern2(client, 'utxos_1m_to_2m_old_spent_utxo_count'),
              _2mTo3m: createAverageBlockCumulativeSumPattern2(client, 'utxos_2m_to_3m_old_spent_utxo_count'),
              _3mTo4m: createAverageBlockCumulativeSumPattern2(client, 'utxos_3m_to_4m_old_spent_utxo_count'),
              _4mTo5m: createAverageBlockCumulativeSumPattern2(client, 'utxos_4m_to_5m_old_spent_utxo_count'),
              _5mTo6m: createAverageBlockCumulativeSumPattern2(client, 'utxos_5m_to_6m_old_spent_utxo_count'),
              _6mTo9m: createAverageBlockCumulativeSumPattern2(client, 'utxos_6m_to_9m_old_spent_utxo_count'),
              _9mTo1y: createAverageBlockCumulativeSumPattern2(client, 'utxos_9m_to_1y_old_spent_utxo_count'),
              _1yTo18m: createAverageBlockCumulativeSumPattern2(client, 'utxos_1y_to_18m_old_spent_utxo_count'),
              _18mTo2y: createAverageBlockCumulativeSumPattern2(client, 'utxos_18m_to_2y_old_spent_utxo_count'),
              _2yTo3y: createAverageBlockCumulativeSumPattern2(client, 'utxos_2y_to_3y_old_spent_utxo_count'),
              _3yTo4y: createAverageBlockCumulativeSumPattern2(client, 'utxos_3y_to_4y_old_spent_utxo_count'),
              _4yTo5y: createAverageBlockCumulativeSumPattern2(client, 'utxos_4y_to_5y_old_spent_utxo_count'),
              _5yTo6y: createAverageBlockCumulativeSumPattern2(client, 'utxos_5y_to_6y_old_spent_utxo_count'),
              _6yTo7y: createAverageBlockCumulativeSumPattern2(client, 'utxos_6y_to_7y_old_spent_utxo_count'),
              _7yTo8y: createAverageBlockCumulativeSumPattern2(client, 'utxos_7y_to_8y_old_spent_utxo_count'),
              _8yTo10y: createAverageBlockCumulativeSumPattern2(client, 'utxos_8y_to_10y_old_spent_utxo_count'),
              _10yTo12y: createAverageBlockCumulativeSumPattern2(client, 'utxos_10y_to_12y_old_spent_utxo_count'),
              _12yTo15y: createAverageBlockCumulativeSumPattern2(client, 'utxos_12y_to_15y_old_spent_utxo_count'),
              over15y: createAverageBlockCumulativeSumPattern2(client, 'utxos_over_15y_old_spent_utxo_count'),
            })); },
            get epoch() { return _lazy(this, 'epoch', () => ({
              _0: createAverageBlockCumulativeSumPattern2(client, 'epoch_0_spent_utxo_count'),
              _1: createAverageBlockCumulativeSumPattern2(client, 'epoch_1_spent_utxo_count'),
              _2: createAverageBlockCumulativeSumPattern2(client, 'epoch_2_spent_utxo_count'),
              _3: createAverageBlockCumulativeSumPattern2(client, 'epoch_3_spent_utxo_count'),
              _4: createAverageBlockCumulativeSumPattern2(client, 'epoch_4_spent_utxo_count'),
            })); },
            get class() { return _lazy(this, 'class', () => ({
              _2009: createAverageBlockCumulativeSumPattern2(client, 'class_2009_spent_utxo_count'),
              _2010: createAverageBlockCumulativeSumPattern2(client, 'class_2010_spent_utxo_count'),
              _2011: createAverageBlockCumulativeSumPattern2(client, 'class_2011_spent_utxo_count'),
              _2012: createAverageBlockCumulativeSumPattern2(client, 'class_2012_spent_utxo_count'),
              _2013: createAverageBlockCumulativeSumPattern2(client, 'class_2013_spent_utxo_count'),
              _2014: createAverageBlockCumulativeSumPattern2(client, 'class_2014_spent_utxo_count'),
              _2015: createAverageBlockCumulativeSumPattern2(client, 'class_2015_spent_utxo_count'),
              _2016: createAverageBlockCumulativeSumPattern2(client, 'class_2016_spent_utxo_count'),
              _2017: createAverageBlockCumulativeSumPattern2(client, 'class_2017_spent_utxo_count'),
              _2018: createAverageBlockCumulativeSumPattern2(client, 'class_2018_spent_utxo_count'),
              _2019: createAverageBlockCumulativeSumPattern2(client, 'class_2019_spent_utxo_count'),
              _2020: createAverageBlockCumulativeSumPattern2(client, 'class_2020_spent_utxo_count'),
              _2021: createAverageBlockCumulativeSumPattern2(client, 'class_2021_spent_utxo_count'),
              _2022: createAverageBlockCumulativeSumPattern2(client, 'class_2022_spent_utxo_count'),
              _2023: createAverageBlockCumulativeSumPattern2(client, 'class_2023_spent_utxo_count'),
              _2024: createAverageBlockCumulativeSumPattern2(client, 'class_2024_spent_utxo_count'),
              _2025: createAverageBlockCumulativeSumPattern2(client, 'class_2025_spent_utxo_count'),
              _2026: createAverageBlockCumulativeSumPattern2(client, 'class_2026_spent_utxo_count'),
            })); },
            get utxoAmount() { return _lazy(this, 'utxoAmount', () => ({
              _0sats: createAverageBlockCumulativeSumPattern2(client, 'utxos_0sats_spent_utxo_count'),
              _1satTo10sats: createAverageBlockCumulativeSumPattern2(client, 'utxos_1sat_to_10sats_spent_utxo_count'),
              _10satsTo100sats: createAverageBlockCumulativeSumPattern2(client, 'utxos_10sats_to_100sats_spent_utxo_count'),
              _100satsTo1kSats: createAverageBlockCumulativeSumPattern2(client, 'utxos_100sats_to_1k_sats_spent_utxo_count'),
              _1kSatsTo10kSats: createAverageBlockCumulativeSumPattern2(client, 'utxos_1k_sats_to_10k_sats_spent_utxo_count'),
              _10kSatsTo100kSats: createAverageBlockCumulativeSumPattern2(client, 'utxos_10k_sats_to_100k_sats_spent_utxo_count'),
              _100kSatsTo1mSats: createAverageBlockCumulativeSumPattern2(client, 'utxos_100k_sats_to_1m_sats_spent_utxo_count'),
              _1mSatsTo10mSats: createAverageBlockCumulativeSumPattern2(client, 'utxos_1m_sats_to_10m_sats_spent_utxo_count'),
              _10mSatsTo1btc: createAverageBlockCumulativeSumPattern2(client, 'utxos_10m_sats_to_1btc_spent_utxo_count'),
              _1btcTo10btc: createAverageBlockCumulativeSumPattern2(client, 'utxos_1btc_to_10btc_spent_utxo_count'),
              _10btcTo100btc: createAverageBlockCumulativeSumPattern2(client, 'utxos_10btc_to_100btc_spent_utxo_count'),
              _100btcTo1kBtc: createAverageBlockCumulativeSumPattern2(client, 'utxos_100btc_to_1k_btc_spent_utxo_count'),
              _1kBtcTo10kBtc: createAverageBlockCumulativeSumPattern2(client, 'utxos_1k_btc_to_10k_btc_spent_utxo_count'),
              _10kBtcTo100kBtc: createAverageBlockCumulativeSumPattern2(client, 'utxos_10k_btc_to_100k_btc_spent_utxo_count'),
              over100kBtc: createAverageBlockCumulativeSumPattern2(client, 'utxos_over_100k_btc_spent_utxo_count'),
            })); },
            get type() { return _lazy(this, 'type', () => ({
              p2pk65: createAverageBlockCumulativeSumPattern2(client, 'p2pk65_spent_utxo_count'),
              p2pk33: createAverageBlockCumulativeSumPattern2(client, 'p2pk33_spent_utxo_count'),
              p2pkh: createAverageBlockCumulativeSumPattern2(client, 'p2pkh_spent_utxo_count'),
              p2ms: createAverageBlockCumulativeSumPattern2(client, 'p2ms_spent_utxo_count'),
              p2sh: createAverageBlockCumulativeSumPattern2(client, 'p2sh_spent_utxo_count'),
              p2wpkh: createAverageBlockCumulativeSumPattern2(client, 'p2wpkh_spent_utxo_count'),
              p2wsh: createAverageBlockCumulativeSumPattern2(client, 'p2wsh_spent_utxo_count'),
              p2tr: createAverageBlockCumulativeSumPattern2(client, 'p2tr_spent_utxo_count'),
              p2a: createAverageBlockCumulativeSumPattern2(client, 'p2a_spent_utxo_count'),
              unknown: createAverageBlockCumulativeSumPattern2(client, 'unknown_outputs_spent_utxo_count'),
              empty: createAverageBlockCumulativeSumPattern2(client, 'empty_outputs_spent_utxo_count'),
            })); },
          })); },
          get avgAmount() { return _lazy(this, 'avgAmount', () => ({
            all: createBtcCentsSatsUsdPattern(client, 'avg_utxo_amount'),
            byType: createEmptyP2aP2msP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshUnknownPattern2(client, 'avg_utxo_amount'),
          })); },
        })); },
        get activity() { return _lazy(this, 'activity', () => ({
          get transferVolume() { return _lazy(this, 'transferVolume', () => ({
            get age() { return _lazy(this, 'age', () => ({
              under1h: createAverageBlockCumulativeSumPattern(client, 'utxos_under_1h_old_transfer_volume'),
              _1hTo1d: createAverageBlockCumulativeSumPattern(client, 'utxos_1h_to_1d_old_transfer_volume'),
              _1dTo1w: createAverageBlockCumulativeSumPattern(client, 'utxos_1d_to_1w_old_transfer_volume'),
              _1wTo1m: createAverageBlockCumulativeSumPattern(client, 'utxos_1w_to_1m_old_transfer_volume'),
              _1mTo2m: createAverageBlockCumulativeSumPattern(client, 'utxos_1m_to_2m_old_transfer_volume'),
              _2mTo3m: createAverageBlockCumulativeSumPattern(client, 'utxos_2m_to_3m_old_transfer_volume'),
              _3mTo4m: createAverageBlockCumulativeSumPattern(client, 'utxos_3m_to_4m_old_transfer_volume'),
              _4mTo5m: createAverageBlockCumulativeSumPattern(client, 'utxos_4m_to_5m_old_transfer_volume'),
              _5mTo6m: createAverageBlockCumulativeSumPattern(client, 'utxos_5m_to_6m_old_transfer_volume'),
              _6mTo9m: createAverageBlockCumulativeSumPattern(client, 'utxos_6m_to_9m_old_transfer_volume'),
              _9mTo1y: createAverageBlockCumulativeSumPattern(client, 'utxos_9m_to_1y_old_transfer_volume'),
              _1yTo18m: createAverageBlockCumulativeSumPattern(client, 'utxos_1y_to_18m_old_transfer_volume'),
              _18mTo2y: createAverageBlockCumulativeSumPattern(client, 'utxos_18m_to_2y_old_transfer_volume'),
              _2yTo3y: createAverageBlockCumulativeSumPattern(client, 'utxos_2y_to_3y_old_transfer_volume'),
              _3yTo4y: createAverageBlockCumulativeSumPattern(client, 'utxos_3y_to_4y_old_transfer_volume'),
              _4yTo5y: createAverageBlockCumulativeSumPattern(client, 'utxos_4y_to_5y_old_transfer_volume'),
              _5yTo6y: createAverageBlockCumulativeSumPattern(client, 'utxos_5y_to_6y_old_transfer_volume'),
              _6yTo7y: createAverageBlockCumulativeSumPattern(client, 'utxos_6y_to_7y_old_transfer_volume'),
              _7yTo8y: createAverageBlockCumulativeSumPattern(client, 'utxos_7y_to_8y_old_transfer_volume'),
              _8yTo10y: createAverageBlockCumulativeSumPattern(client, 'utxos_8y_to_10y_old_transfer_volume'),
              _10yTo12y: createAverageBlockCumulativeSumPattern(client, 'utxos_10y_to_12y_old_transfer_volume'),
              _12yTo15y: createAverageBlockCumulativeSumPattern(client, 'utxos_12y_to_15y_old_transfer_volume'),
              over15y: createAverageBlockCumulativeSumPattern(client, 'utxos_over_15y_old_transfer_volume'),
            })); },
            get epoch() { return _lazy(this, 'epoch', () => ({
              _0: createAverageBlockCumulativeSumPattern(client, 'epoch_0_transfer_volume'),
              _1: createAverageBlockCumulativeSumPattern(client, 'epoch_1_transfer_volume'),
              _2: createAverageBlockCumulativeSumPattern(client, 'epoch_2_transfer_volume'),
              _3: createAverageBlockCumulativeSumPattern(client, 'epoch_3_transfer_volume'),
              _4: createAverageBlockCumulativeSumPattern(client, 'epoch_4_transfer_volume'),
            })); },
            get class() { return _lazy(this, 'class', () => ({
              _2009: createAverageBlockCumulativeSumPattern(client, 'class_2009_transfer_volume'),
              _2010: createAverageBlockCumulativeSumPattern(client, 'class_2010_transfer_volume'),
              _2011: createAverageBlockCumulativeSumPattern(client, 'class_2011_transfer_volume'),
              _2012: createAverageBlockCumulativeSumPattern(client, 'class_2012_transfer_volume'),
              _2013: createAverageBlockCumulativeSumPattern(client, 'class_2013_transfer_volume'),
              _2014: createAverageBlockCumulativeSumPattern(client, 'class_2014_transfer_volume'),
              _2015: createAverageBlockCumulativeSumPattern(client, 'class_2015_transfer_volume'),
              _2016: createAverageBlockCumulativeSumPattern(client, 'class_2016_transfer_volume'),
              _2017: createAverageBlockCumulativeSumPattern(client, 'class_2017_transfer_volume'),
              _2018: createAverageBlockCumulativeSumPattern(client, 'class_2018_transfer_volume'),
              _2019: createAverageBlockCumulativeSumPattern(client, 'class_2019_transfer_volume'),
              _2020: createAverageBlockCumulativeSumPattern(client, 'class_2020_transfer_volume'),
              _2021: createAverageBlockCumulativeSumPattern(client, 'class_2021_transfer_volume'),
              _2022: createAverageBlockCumulativeSumPattern(client, 'class_2022_transfer_volume'),
              _2023: createAverageBlockCumulativeSumPattern(client, 'class_2023_transfer_volume'),
              _2024: createAverageBlockCumulativeSumPattern(client, 'class_2024_transfer_volume'),
              _2025: createAverageBlockCumulativeSumPattern(client, 'class_2025_transfer_volume'),
              _2026: createAverageBlockCumulativeSumPattern(client, 'class_2026_transfer_volume'),
            })); },
            get inProfit() { return _lazy(this, 'inProfit', () => ({
              get age() { return _lazy(this, 'age', () => ({
                under1h: createAverageBlockCumulativeSumPattern(client, 'utxos_under_1h_old_transfer_volume_in_profit'),
                _1hTo1d: createAverageBlockCumulativeSumPattern(client, 'utxos_1h_to_1d_old_transfer_volume_in_profit'),
                _1dTo1w: createAverageBlockCumulativeSumPattern(client, 'utxos_1d_to_1w_old_transfer_volume_in_profit'),
                _1wTo1m: createAverageBlockCumulativeSumPattern(client, 'utxos_1w_to_1m_old_transfer_volume_in_profit'),
                _1mTo2m: createAverageBlockCumulativeSumPattern(client, 'utxos_1m_to_2m_old_transfer_volume_in_profit'),
                _2mTo3m: createAverageBlockCumulativeSumPattern(client, 'utxos_2m_to_3m_old_transfer_volume_in_profit'),
                _3mTo4m: createAverageBlockCumulativeSumPattern(client, 'utxos_3m_to_4m_old_transfer_volume_in_profit'),
                _4mTo5m: createAverageBlockCumulativeSumPattern(client, 'utxos_4m_to_5m_old_transfer_volume_in_profit'),
                _5mTo6m: createAverageBlockCumulativeSumPattern(client, 'utxos_5m_to_6m_old_transfer_volume_in_profit'),
                _6mTo9m: createAverageBlockCumulativeSumPattern(client, 'utxos_6m_to_9m_old_transfer_volume_in_profit'),
                _9mTo1y: createAverageBlockCumulativeSumPattern(client, 'utxos_9m_to_1y_old_transfer_volume_in_profit'),
                _1yTo18m: createAverageBlockCumulativeSumPattern(client, 'utxos_1y_to_18m_old_transfer_volume_in_profit'),
                _18mTo2y: createAverageBlockCumulativeSumPattern(client, 'utxos_18m_to_2y_old_transfer_volume_in_profit'),
                _2yTo3y: createAverageBlockCumulativeSumPattern(client, 'utxos_2y_to_3y_old_transfer_volume_in_profit'),
                _3yTo4y: createAverageBlockCumulativeSumPattern(client, 'utxos_3y_to_4y_old_transfer_volume_in_profit'),
                _4yTo5y: createAverageBlockCumulativeSumPattern(client, 'utxos_4y_to_5y_old_transfer_volume_in_profit'),
                _5yTo6y: createAverageBlockCumulativeSumPattern(client, 'utxos_5y_to_6y_old_transfer_volume_in_profit'),
                _6yTo7y: createAverageBlockCumulativeSumPattern(client, 'utxos_6y_to_7y_old_transfer_volume_in_profit'),
                _7yTo8y: createAverageBlockCumulativeSumPattern(client, 'utxos_7y_to_8y_old_transfer_volume_in_profit'),
                _8yTo10y: createAverageBlockCumulativeSumPattern(client, 'utxos_8y_to_10y_old_transfer_volume_in_profit'),
                _10yTo12y: createAverageBlockCumulativeSumPattern(client, 'utxos_10y_to_12y_old_transfer_volume_in_profit'),
                _12yTo15y: createAverageBlockCumulativeSumPattern(client, 'utxos_12y_to_15y_old_transfer_volume_in_profit'),
                over15y: createAverageBlockCumulativeSumPattern(client, 'utxos_over_15y_old_transfer_volume_in_profit'),
              })); },
              epoch: create_01234Pattern2(client, 'epoch', 'in_profit'),
              class: create_200920102011201220132014201520162017201820192020202120222023202420252026Pattern2(client, 'class', 'in_profit'),
            })); },
            get inLoss() { return _lazy(this, 'inLoss', () => ({
              get age() { return _lazy(this, 'age', () => ({
                under1h: createAverageBlockCumulativeSumPattern(client, 'utxos_under_1h_old_transfer_volume_in_loss'),
                _1hTo1d: createAverageBlockCumulativeSumPattern(client, 'utxos_1h_to_1d_old_transfer_volume_in_loss'),
                _1dTo1w: createAverageBlockCumulativeSumPattern(client, 'utxos_1d_to_1w_old_transfer_volume_in_loss'),
                _1wTo1m: createAverageBlockCumulativeSumPattern(client, 'utxos_1w_to_1m_old_transfer_volume_in_loss'),
                _1mTo2m: createAverageBlockCumulativeSumPattern(client, 'utxos_1m_to_2m_old_transfer_volume_in_loss'),
                _2mTo3m: createAverageBlockCumulativeSumPattern(client, 'utxos_2m_to_3m_old_transfer_volume_in_loss'),
                _3mTo4m: createAverageBlockCumulativeSumPattern(client, 'utxos_3m_to_4m_old_transfer_volume_in_loss'),
                _4mTo5m: createAverageBlockCumulativeSumPattern(client, 'utxos_4m_to_5m_old_transfer_volume_in_loss'),
                _5mTo6m: createAverageBlockCumulativeSumPattern(client, 'utxos_5m_to_6m_old_transfer_volume_in_loss'),
                _6mTo9m: createAverageBlockCumulativeSumPattern(client, 'utxos_6m_to_9m_old_transfer_volume_in_loss'),
                _9mTo1y: createAverageBlockCumulativeSumPattern(client, 'utxos_9m_to_1y_old_transfer_volume_in_loss'),
                _1yTo18m: createAverageBlockCumulativeSumPattern(client, 'utxos_1y_to_18m_old_transfer_volume_in_loss'),
                _18mTo2y: createAverageBlockCumulativeSumPattern(client, 'utxos_18m_to_2y_old_transfer_volume_in_loss'),
                _2yTo3y: createAverageBlockCumulativeSumPattern(client, 'utxos_2y_to_3y_old_transfer_volume_in_loss'),
                _3yTo4y: createAverageBlockCumulativeSumPattern(client, 'utxos_3y_to_4y_old_transfer_volume_in_loss'),
                _4yTo5y: createAverageBlockCumulativeSumPattern(client, 'utxos_4y_to_5y_old_transfer_volume_in_loss'),
                _5yTo6y: createAverageBlockCumulativeSumPattern(client, 'utxos_5y_to_6y_old_transfer_volume_in_loss'),
                _6yTo7y: createAverageBlockCumulativeSumPattern(client, 'utxos_6y_to_7y_old_transfer_volume_in_loss'),
                _7yTo8y: createAverageBlockCumulativeSumPattern(client, 'utxos_7y_to_8y_old_transfer_volume_in_loss'),
                _8yTo10y: createAverageBlockCumulativeSumPattern(client, 'utxos_8y_to_10y_old_transfer_volume_in_loss'),
                _10yTo12y: createAverageBlockCumulativeSumPattern(client, 'utxos_10y_to_12y_old_transfer_volume_in_loss'),
                _12yTo15y: createAverageBlockCumulativeSumPattern(client, 'utxos_12y_to_15y_old_transfer_volume_in_loss'),
                over15y: createAverageBlockCumulativeSumPattern(client, 'utxos_over_15y_old_transfer_volume_in_loss'),
              })); },
              epoch: create_01234Pattern2(client, 'epoch', 'in_loss'),
              class: create_200920102011201220132014201520162017201820192020202120222023202420252026Pattern2(client, 'class', 'in_loss'),
            })); },
            utxoAmount: create_0sats100btc100k100sats10btc10k10m10sats1btc1k1m1satOverPattern4(client, 'utxos'),
            get type() { return _lazy(this, 'type', () => ({
              p2pk65: createAverageBlockCumulativeSumPattern(client, 'p2pk65_transfer_volume'),
              p2pk33: createAverageBlockCumulativeSumPattern(client, 'p2pk33_transfer_volume'),
              p2pkh: createAverageBlockCumulativeSumPattern(client, 'p2pkh_transfer_volume'),
              p2ms: createAverageBlockCumulativeSumPattern(client, 'p2ms_transfer_volume'),
              p2sh: createAverageBlockCumulativeSumPattern(client, 'p2sh_transfer_volume'),
              p2wpkh: createAverageBlockCumulativeSumPattern(client, 'p2wpkh_transfer_volume'),
              p2wsh: createAverageBlockCumulativeSumPattern(client, 'p2wsh_transfer_volume'),
              p2tr: createAverageBlockCumulativeSumPattern(client, 'p2tr_transfer_volume'),
              p2a: createAverageBlockCumulativeSumPattern(client, 'p2a_transfer_volume'),
              unknown: createAverageBlockCumulativeSumPattern(client, 'unknown_outputs_transfer_volume'),
              empty: createAverageBlockCumulativeSumPattern(client, 'empty_outputs_transfer_volume'),
            })); },
          })); },
          get coindaysDestroyed() { return _lazy(this, 'coindaysDestroyed', () => ({
            get age() { return _lazy(this, 'age', () => ({
              under1h: createAverageBlockCumulativeSumPattern2(client, 'utxos_under_1h_old_coindays_destroyed'),
              _1hTo1d: createAverageBlockCumulativeSumPattern2(client, 'utxos_1h_to_1d_old_coindays_destroyed'),
              _1dTo1w: createAverageBlockCumulativeSumPattern2(client, 'utxos_1d_to_1w_old_coindays_destroyed'),
              _1wTo1m: createAverageBlockCumulativeSumPattern2(client, 'utxos_1w_to_1m_old_coindays_destroyed'),
              _1mTo2m: createAverageBlockCumulativeSumPattern2(client, 'utxos_1m_to_2m_old_coindays_destroyed'),
              _2mTo3m: createAverageBlockCumulativeSumPattern2(client, 'utxos_2m_to_3m_old_coindays_destroyed'),
              _3mTo4m: createAverageBlockCumulativeSumPattern2(client, 'utxos_3m_to_4m_old_coindays_destroyed'),
              _4mTo5m: createAverageBlockCumulativeSumPattern2(client, 'utxos_4m_to_5m_old_coindays_destroyed'),
              _5mTo6m: createAverageBlockCumulativeSumPattern2(client, 'utxos_5m_to_6m_old_coindays_destroyed'),
              _6mTo9m: createAverageBlockCumulativeSumPattern2(client, 'utxos_6m_to_9m_old_coindays_destroyed'),
              _9mTo1y: createAverageBlockCumulativeSumPattern2(client, 'utxos_9m_to_1y_old_coindays_destroyed'),
              _1yTo18m: createAverageBlockCumulativeSumPattern2(client, 'utxos_1y_to_18m_old_coindays_destroyed'),
              _18mTo2y: createAverageBlockCumulativeSumPattern2(client, 'utxos_18m_to_2y_old_coindays_destroyed'),
              _2yTo3y: createAverageBlockCumulativeSumPattern2(client, 'utxos_2y_to_3y_old_coindays_destroyed'),
              _3yTo4y: createAverageBlockCumulativeSumPattern2(client, 'utxos_3y_to_4y_old_coindays_destroyed'),
              _4yTo5y: createAverageBlockCumulativeSumPattern2(client, 'utxos_4y_to_5y_old_coindays_destroyed'),
              _5yTo6y: createAverageBlockCumulativeSumPattern2(client, 'utxos_5y_to_6y_old_coindays_destroyed'),
              _6yTo7y: createAverageBlockCumulativeSumPattern2(client, 'utxos_6y_to_7y_old_coindays_destroyed'),
              _7yTo8y: createAverageBlockCumulativeSumPattern2(client, 'utxos_7y_to_8y_old_coindays_destroyed'),
              _8yTo10y: createAverageBlockCumulativeSumPattern2(client, 'utxos_8y_to_10y_old_coindays_destroyed'),
              _10yTo12y: createAverageBlockCumulativeSumPattern2(client, 'utxos_10y_to_12y_old_coindays_destroyed'),
              _12yTo15y: createAverageBlockCumulativeSumPattern2(client, 'utxos_12y_to_15y_old_coindays_destroyed'),
              over15y: createAverageBlockCumulativeSumPattern2(client, 'utxos_over_15y_old_coindays_destroyed'),
            })); },
            get epoch() { return _lazy(this, 'epoch', () => ({
              _0: createAverageBlockCumulativeSumPattern2(client, 'epoch_0_coindays_destroyed'),
              _1: createAverageBlockCumulativeSumPattern2(client, 'epoch_1_coindays_destroyed'),
              _2: createAverageBlockCumulativeSumPattern2(client, 'epoch_2_coindays_destroyed'),
              _3: createAverageBlockCumulativeSumPattern2(client, 'epoch_3_coindays_destroyed'),
              _4: createAverageBlockCumulativeSumPattern2(client, 'epoch_4_coindays_destroyed'),
            })); },
            get class() { return _lazy(this, 'class', () => ({
              _2009: createAverageBlockCumulativeSumPattern2(client, 'class_2009_coindays_destroyed'),
              _2010: createAverageBlockCumulativeSumPattern2(client, 'class_2010_coindays_destroyed'),
              _2011: createAverageBlockCumulativeSumPattern2(client, 'class_2011_coindays_destroyed'),
              _2012: createAverageBlockCumulativeSumPattern2(client, 'class_2012_coindays_destroyed'),
              _2013: createAverageBlockCumulativeSumPattern2(client, 'class_2013_coindays_destroyed'),
              _2014: createAverageBlockCumulativeSumPattern2(client, 'class_2014_coindays_destroyed'),
              _2015: createAverageBlockCumulativeSumPattern2(client, 'class_2015_coindays_destroyed'),
              _2016: createAverageBlockCumulativeSumPattern2(client, 'class_2016_coindays_destroyed'),
              _2017: createAverageBlockCumulativeSumPattern2(client, 'class_2017_coindays_destroyed'),
              _2018: createAverageBlockCumulativeSumPattern2(client, 'class_2018_coindays_destroyed'),
              _2019: createAverageBlockCumulativeSumPattern2(client, 'class_2019_coindays_destroyed'),
              _2020: createAverageBlockCumulativeSumPattern2(client, 'class_2020_coindays_destroyed'),
              _2021: createAverageBlockCumulativeSumPattern2(client, 'class_2021_coindays_destroyed'),
              _2022: createAverageBlockCumulativeSumPattern2(client, 'class_2022_coindays_destroyed'),
              _2023: createAverageBlockCumulativeSumPattern2(client, 'class_2023_coindays_destroyed'),
              _2024: createAverageBlockCumulativeSumPattern2(client, 'class_2024_coindays_destroyed'),
              _2025: createAverageBlockCumulativeSumPattern2(client, 'class_2025_coindays_destroyed'),
              _2026: createAverageBlockCumulativeSumPattern2(client, 'class_2026_coindays_destroyed'),
            })); },
          })); },
        })); },
        get realized() { return _lazy(this, 'realized', () => ({
          get cap() { return _lazy(this, 'cap', () => ({
            get age() { return _lazy(this, 'age', () => ({
              under1h: createCentsUsdPattern(client, 'utxos_under_1h_old_realized_cap'),
              _1hTo1d: createCentsUsdPattern(client, 'utxos_1h_to_1d_old_realized_cap'),
              _1dTo1w: createCentsUsdPattern(client, 'utxos_1d_to_1w_old_realized_cap'),
              _1wTo1m: createCentsUsdPattern(client, 'utxos_1w_to_1m_old_realized_cap'),
              _1mTo2m: createCentsUsdPattern(client, 'utxos_1m_to_2m_old_realized_cap'),
              _2mTo3m: createCentsUsdPattern(client, 'utxos_2m_to_3m_old_realized_cap'),
              _3mTo4m: createCentsUsdPattern(client, 'utxos_3m_to_4m_old_realized_cap'),
              _4mTo5m: createCentsUsdPattern(client, 'utxos_4m_to_5m_old_realized_cap'),
              _5mTo6m: createCentsUsdPattern(client, 'utxos_5m_to_6m_old_realized_cap'),
              _6mTo9m: createCentsUsdPattern(client, 'utxos_6m_to_9m_old_realized_cap'),
              _9mTo1y: createCentsUsdPattern(client, 'utxos_9m_to_1y_old_realized_cap'),
              _1yTo18m: createCentsUsdPattern(client, 'utxos_1y_to_18m_old_realized_cap'),
              _18mTo2y: createCentsUsdPattern(client, 'utxos_18m_to_2y_old_realized_cap'),
              _2yTo3y: createCentsUsdPattern(client, 'utxos_2y_to_3y_old_realized_cap'),
              _3yTo4y: createCentsUsdPattern(client, 'utxos_3y_to_4y_old_realized_cap'),
              _4yTo5y: createCentsUsdPattern(client, 'utxos_4y_to_5y_old_realized_cap'),
              _5yTo6y: createCentsUsdPattern(client, 'utxos_5y_to_6y_old_realized_cap'),
              _6yTo7y: createCentsUsdPattern(client, 'utxos_6y_to_7y_old_realized_cap'),
              _7yTo8y: createCentsUsdPattern(client, 'utxos_7y_to_8y_old_realized_cap'),
              _8yTo10y: createCentsUsdPattern(client, 'utxos_8y_to_10y_old_realized_cap'),
              _10yTo12y: createCentsUsdPattern(client, 'utxos_10y_to_12y_old_realized_cap'),
              _12yTo15y: createCentsUsdPattern(client, 'utxos_12y_to_15y_old_realized_cap'),
              over15y: createCentsUsdPattern(client, 'utxos_over_15y_old_realized_cap'),
            })); },
            get epoch() { return _lazy(this, 'epoch', () => ({
              _0: createCentsUsdPattern(client, 'epoch_0_realized_cap'),
              _1: createCentsUsdPattern(client, 'epoch_1_realized_cap'),
              _2: createCentsUsdPattern(client, 'epoch_2_realized_cap'),
              _3: createCentsUsdPattern(client, 'epoch_3_realized_cap'),
              _4: createCentsUsdPattern(client, 'epoch_4_realized_cap'),
            })); },
            get class() { return _lazy(this, 'class', () => ({
              _2009: createCentsUsdPattern(client, 'class_2009_realized_cap'),
              _2010: createCentsUsdPattern(client, 'class_2010_realized_cap'),
              _2011: createCentsUsdPattern(client, 'class_2011_realized_cap'),
              _2012: createCentsUsdPattern(client, 'class_2012_realized_cap'),
              _2013: createCentsUsdPattern(client, 'class_2013_realized_cap'),
              _2014: createCentsUsdPattern(client, 'class_2014_realized_cap'),
              _2015: createCentsUsdPattern(client, 'class_2015_realized_cap'),
              _2016: createCentsUsdPattern(client, 'class_2016_realized_cap'),
              _2017: createCentsUsdPattern(client, 'class_2017_realized_cap'),
              _2018: createCentsUsdPattern(client, 'class_2018_realized_cap'),
              _2019: createCentsUsdPattern(client, 'class_2019_realized_cap'),
              _2020: createCentsUsdPattern(client, 'class_2020_realized_cap'),
              _2021: createCentsUsdPattern(client, 'class_2021_realized_cap'),
              _2022: createCentsUsdPattern(client, 'class_2022_realized_cap'),
              _2023: createCentsUsdPattern(client, 'class_2023_realized_cap'),
              _2024: createCentsUsdPattern(client, 'class_2024_realized_cap'),
              _2025: createCentsUsdPattern(client, 'class_2025_realized_cap'),
              _2026: createCentsUsdPattern(client, 'class_2026_realized_cap'),
            })); },
            utxoAmount: create_0sats100btc100k100sats10btc10k10m10sats1btc1k1m1satOverPattern(client, 'utxos'),
            get type() { return _lazy(this, 'type', () => ({
              p2pk65: createCentsUsdPattern(client, 'p2pk65_realized_cap'),
              p2pk33: createCentsUsdPattern(client, 'p2pk33_realized_cap'),
              p2pkh: createCentsUsdPattern(client, 'p2pkh_realized_cap'),
              p2ms: createCentsUsdPattern(client, 'p2ms_realized_cap'),
              p2sh: createCentsUsdPattern(client, 'p2sh_realized_cap'),
              p2wpkh: createCentsUsdPattern(client, 'p2wpkh_realized_cap'),
              p2wsh: createCentsUsdPattern(client, 'p2wsh_realized_cap'),
              p2tr: createCentsUsdPattern(client, 'p2tr_realized_cap'),
              p2a: createCentsUsdPattern(client, 'p2a_realized_cap'),
              unknown: createCentsUsdPattern(client, 'unknown_outputs_realized_cap'),
              empty: createCentsUsdPattern(client, 'empty_outputs_realized_cap'),
            })); },
          })); },
          get profit() { return _lazy(this, 'profit', () => ({
            get age() { return _lazy(this, 'age', () => ({
              under1h: createBlockCumulativeSumPattern(client, 'utxos_under_1h_old_realized_profit'),
              _1hTo1d: createBlockCumulativeSumPattern(client, 'utxos_1h_to_1d_old_realized_profit'),
              _1dTo1w: createBlockCumulativeSumPattern(client, 'utxos_1d_to_1w_old_realized_profit'),
              _1wTo1m: createBlockCumulativeSumPattern(client, 'utxos_1w_to_1m_old_realized_profit'),
              _1mTo2m: createBlockCumulativeSumPattern(client, 'utxos_1m_to_2m_old_realized_profit'),
              _2mTo3m: createBlockCumulativeSumPattern(client, 'utxos_2m_to_3m_old_realized_profit'),
              _3mTo4m: createBlockCumulativeSumPattern(client, 'utxos_3m_to_4m_old_realized_profit'),
              _4mTo5m: createBlockCumulativeSumPattern(client, 'utxos_4m_to_5m_old_realized_profit'),
              _5mTo6m: createBlockCumulativeSumPattern(client, 'utxos_5m_to_6m_old_realized_profit'),
              _6mTo9m: createBlockCumulativeSumPattern(client, 'utxos_6m_to_9m_old_realized_profit'),
              _9mTo1y: createBlockCumulativeSumPattern(client, 'utxos_9m_to_1y_old_realized_profit'),
              _1yTo18m: createBlockCumulativeSumPattern(client, 'utxos_1y_to_18m_old_realized_profit'),
              _18mTo2y: createBlockCumulativeSumPattern(client, 'utxos_18m_to_2y_old_realized_profit'),
              _2yTo3y: createBlockCumulativeSumPattern(client, 'utxos_2y_to_3y_old_realized_profit'),
              _3yTo4y: createBlockCumulativeSumPattern(client, 'utxos_3y_to_4y_old_realized_profit'),
              _4yTo5y: createBlockCumulativeSumPattern(client, 'utxos_4y_to_5y_old_realized_profit'),
              _5yTo6y: createBlockCumulativeSumPattern(client, 'utxos_5y_to_6y_old_realized_profit'),
              _6yTo7y: createBlockCumulativeSumPattern(client, 'utxos_6y_to_7y_old_realized_profit'),
              _7yTo8y: createBlockCumulativeSumPattern(client, 'utxos_7y_to_8y_old_realized_profit'),
              _8yTo10y: createBlockCumulativeSumPattern(client, 'utxos_8y_to_10y_old_realized_profit'),
              _10yTo12y: createBlockCumulativeSumPattern(client, 'utxos_10y_to_12y_old_realized_profit'),
              _12yTo15y: createBlockCumulativeSumPattern(client, 'utxos_12y_to_15y_old_realized_profit'),
              over15y: createBlockCumulativeSumPattern(client, 'utxos_over_15y_old_realized_profit'),
            })); },
            get epoch() { return _lazy(this, 'epoch', () => ({
              _0: createBlockCumulativeSumPattern(client, 'epoch_0_realized_profit'),
              _1: createBlockCumulativeSumPattern(client, 'epoch_1_realized_profit'),
              _2: createBlockCumulativeSumPattern(client, 'epoch_2_realized_profit'),
              _3: createBlockCumulativeSumPattern(client, 'epoch_3_realized_profit'),
              _4: createBlockCumulativeSumPattern(client, 'epoch_4_realized_profit'),
            })); },
            get class() { return _lazy(this, 'class', () => ({
              _2009: createBlockCumulativeSumPattern(client, 'class_2009_realized_profit'),
              _2010: createBlockCumulativeSumPattern(client, 'class_2010_realized_profit'),
              _2011: createBlockCumulativeSumPattern(client, 'class_2011_realized_profit'),
              _2012: createBlockCumulativeSumPattern(client, 'class_2012_realized_profit'),
              _2013: createBlockCumulativeSumPattern(client, 'class_2013_realized_profit'),
              _2014: createBlockCumulativeSumPattern(client, 'class_2014_realized_profit'),
              _2015: createBlockCumulativeSumPattern(client, 'class_2015_realized_profit'),
              _2016: createBlockCumulativeSumPattern(client, 'class_2016_realized_profit'),
              _2017: createBlockCumulativeSumPattern(client, 'class_2017_realized_profit'),
              _2018: createBlockCumulativeSumPattern(client, 'class_2018_realized_profit'),
              _2019: createBlockCumulativeSumPattern(client, 'class_2019_realized_profit'),
              _2020: createBlockCumulativeSumPattern(client, 'class_2020_realized_profit'),
              _2021: createBlockCumulativeSumPattern(client, 'class_2021_realized_profit'),
              _2022: createBlockCumulativeSumPattern(client, 'class_2022_realized_profit'),
              _2023: createBlockCumulativeSumPattern(client, 'class_2023_realized_profit'),
              _2024: createBlockCumulativeSumPattern(client, 'class_2024_realized_profit'),
              _2025: createBlockCumulativeSumPattern(client, 'class_2025_realized_profit'),
              _2026: createBlockCumulativeSumPattern(client, 'class_2026_realized_profit'),
            })); },
            get utxoAmount() { return _lazy(this, 'utxoAmount', () => ({
              _0sats: createBlockCumulativeSumPattern(client, 'utxos_0sats_realized_profit'),
              _1satTo10sats: createBlockCumulativeSumPattern(client, 'utxos_1sat_to_10sats_realized_profit'),
              _10satsTo100sats: createBlockCumulativeSumPattern(client, 'utxos_10sats_to_100sats_realized_profit'),
              _100satsTo1kSats: createBlockCumulativeSumPattern(client, 'utxos_100sats_to_1k_sats_realized_profit'),
              _1kSatsTo10kSats: createBlockCumulativeSumPattern(client, 'utxos_1k_sats_to_10k_sats_realized_profit'),
              _10kSatsTo100kSats: createBlockCumulativeSumPattern(client, 'utxos_10k_sats_to_100k_sats_realized_profit'),
              _100kSatsTo1mSats: createBlockCumulativeSumPattern(client, 'utxos_100k_sats_to_1m_sats_realized_profit'),
              _1mSatsTo10mSats: createBlockCumulativeSumPattern(client, 'utxos_1m_sats_to_10m_sats_realized_profit'),
              _10mSatsTo1btc: createBlockCumulativeSumPattern(client, 'utxos_10m_sats_to_1btc_realized_profit'),
              _1btcTo10btc: createBlockCumulativeSumPattern(client, 'utxos_1btc_to_10btc_realized_profit'),
              _10btcTo100btc: createBlockCumulativeSumPattern(client, 'utxos_10btc_to_100btc_realized_profit'),
              _100btcTo1kBtc: createBlockCumulativeSumPattern(client, 'utxos_100btc_to_1k_btc_realized_profit'),
              _1kBtcTo10kBtc: createBlockCumulativeSumPattern(client, 'utxos_1k_btc_to_10k_btc_realized_profit'),
              _10kBtcTo100kBtc: createBlockCumulativeSumPattern(client, 'utxos_10k_btc_to_100k_btc_realized_profit'),
              over100kBtc: createBlockCumulativeSumPattern(client, 'utxos_over_100k_btc_realized_profit'),
            })); },
            type: createEmptyP2aP2msP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshUnknownPattern6(client, 'realized_profit'),
          })); },
          get loss() { return _lazy(this, 'loss', () => ({
            get age() { return _lazy(this, 'age', () => ({
              under1h: createBlockCumulativeSumPattern(client, 'utxos_under_1h_old_realized_loss'),
              _1hTo1d: createBlockCumulativeSumPattern(client, 'utxos_1h_to_1d_old_realized_loss'),
              _1dTo1w: createBlockCumulativeSumPattern(client, 'utxos_1d_to_1w_old_realized_loss'),
              _1wTo1m: createBlockCumulativeSumPattern(client, 'utxos_1w_to_1m_old_realized_loss'),
              _1mTo2m: createBlockCumulativeSumPattern(client, 'utxos_1m_to_2m_old_realized_loss'),
              _2mTo3m: createBlockCumulativeSumPattern(client, 'utxos_2m_to_3m_old_realized_loss'),
              _3mTo4m: createBlockCumulativeSumPattern(client, 'utxos_3m_to_4m_old_realized_loss'),
              _4mTo5m: createBlockCumulativeSumPattern(client, 'utxos_4m_to_5m_old_realized_loss'),
              _5mTo6m: createBlockCumulativeSumPattern(client, 'utxos_5m_to_6m_old_realized_loss'),
              _6mTo9m: createBlockCumulativeSumPattern(client, 'utxos_6m_to_9m_old_realized_loss'),
              _9mTo1y: createBlockCumulativeSumPattern(client, 'utxos_9m_to_1y_old_realized_loss'),
              _1yTo18m: createBlockCumulativeSumPattern(client, 'utxos_1y_to_18m_old_realized_loss'),
              _18mTo2y: createBlockCumulativeSumPattern(client, 'utxos_18m_to_2y_old_realized_loss'),
              _2yTo3y: createBlockCumulativeSumPattern(client, 'utxos_2y_to_3y_old_realized_loss'),
              _3yTo4y: createBlockCumulativeSumPattern(client, 'utxos_3y_to_4y_old_realized_loss'),
              _4yTo5y: createBlockCumulativeSumPattern(client, 'utxos_4y_to_5y_old_realized_loss'),
              _5yTo6y: createBlockCumulativeSumPattern(client, 'utxos_5y_to_6y_old_realized_loss'),
              _6yTo7y: createBlockCumulativeSumPattern(client, 'utxos_6y_to_7y_old_realized_loss'),
              _7yTo8y: createBlockCumulativeSumPattern(client, 'utxos_7y_to_8y_old_realized_loss'),
              _8yTo10y: createBlockCumulativeSumPattern(client, 'utxos_8y_to_10y_old_realized_loss'),
              _10yTo12y: createBlockCumulativeSumPattern(client, 'utxos_10y_to_12y_old_realized_loss'),
              _12yTo15y: createBlockCumulativeSumPattern(client, 'utxos_12y_to_15y_old_realized_loss'),
              over15y: createBlockCumulativeSumPattern(client, 'utxos_over_15y_old_realized_loss'),
            })); },
            get epoch() { return _lazy(this, 'epoch', () => ({
              _0: createBlockCumulativeSumPattern(client, 'epoch_0_realized_loss'),
              _1: createBlockCumulativeSumPattern(client, 'epoch_1_realized_loss'),
              _2: createBlockCumulativeSumPattern(client, 'epoch_2_realized_loss'),
              _3: createBlockCumulativeSumPattern(client, 'epoch_3_realized_loss'),
              _4: createBlockCumulativeSumPattern(client, 'epoch_4_realized_loss'),
            })); },
            get class() { return _lazy(this, 'class', () => ({
              _2009: createBlockCumulativeSumPattern(client, 'class_2009_realized_loss'),
              _2010: createBlockCumulativeSumPattern(client, 'class_2010_realized_loss'),
              _2011: createBlockCumulativeSumPattern(client, 'class_2011_realized_loss'),
              _2012: createBlockCumulativeSumPattern(client, 'class_2012_realized_loss'),
              _2013: createBlockCumulativeSumPattern(client, 'class_2013_realized_loss'),
              _2014: createBlockCumulativeSumPattern(client, 'class_2014_realized_loss'),
              _2015: createBlockCumulativeSumPattern(client, 'class_2015_realized_loss'),
              _2016: createBlockCumulativeSumPattern(client, 'class_2016_realized_loss'),
              _2017: createBlockCumulativeSumPattern(client, 'class_2017_realized_loss'),
              _2018: createBlockCumulativeSumPattern(client, 'class_2018_realized_loss'),
              _2019: createBlockCumulativeSumPattern(client, 'class_2019_realized_loss'),
              _2020: createBlockCumulativeSumPattern(client, 'class_2020_realized_loss'),
              _2021: createBlockCumulativeSumPattern(client, 'class_2021_realized_loss'),
              _2022: createBlockCumulativeSumPattern(client, 'class_2022_realized_loss'),
              _2023: createBlockCumulativeSumPattern(client, 'class_2023_realized_loss'),
              _2024: createBlockCumulativeSumPattern(client, 'class_2024_realized_loss'),
              _2025: createBlockCumulativeSumPattern(client, 'class_2025_realized_loss'),
              _2026: createBlockCumulativeSumPattern(client, 'class_2026_realized_loss'),
            })); },
            get utxoAmount() { return _lazy(this, 'utxoAmount', () => ({
              _0sats: createBlockCumulativeSumPattern(client, 'utxos_0sats_realized_loss'),
              _1satTo10sats: createBlockCumulativeSumPattern(client, 'utxos_1sat_to_10sats_realized_loss'),
              _10satsTo100sats: createBlockCumulativeSumPattern(client, 'utxos_10sats_to_100sats_realized_loss'),
              _100satsTo1kSats: createBlockCumulativeSumPattern(client, 'utxos_100sats_to_1k_sats_realized_loss'),
              _1kSatsTo10kSats: createBlockCumulativeSumPattern(client, 'utxos_1k_sats_to_10k_sats_realized_loss'),
              _10kSatsTo100kSats: createBlockCumulativeSumPattern(client, 'utxos_10k_sats_to_100k_sats_realized_loss'),
              _100kSatsTo1mSats: createBlockCumulativeSumPattern(client, 'utxos_100k_sats_to_1m_sats_realized_loss'),
              _1mSatsTo10mSats: createBlockCumulativeSumPattern(client, 'utxos_1m_sats_to_10m_sats_realized_loss'),
              _10mSatsTo1btc: createBlockCumulativeSumPattern(client, 'utxos_10m_sats_to_1btc_realized_loss'),
              _1btcTo10btc: createBlockCumulativeSumPattern(client, 'utxos_1btc_to_10btc_realized_loss'),
              _10btcTo100btc: createBlockCumulativeSumPattern(client, 'utxos_10btc_to_100btc_realized_loss'),
              _100btcTo1kBtc: createBlockCumulativeSumPattern(client, 'utxos_100btc_to_1k_btc_realized_loss'),
              _1kBtcTo10kBtc: createBlockCumulativeSumPattern(client, 'utxos_1k_btc_to_10k_btc_realized_loss'),
              _10kBtcTo100kBtc: createBlockCumulativeSumPattern(client, 'utxos_10k_btc_to_100k_btc_realized_loss'),
              over100kBtc: createBlockCumulativeSumPattern(client, 'utxos_over_100k_btc_realized_loss'),
            })); },
            type: createEmptyP2aP2msP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshUnknownPattern6(client, 'realized_loss'),
          })); },
          get netPnl() { return _lazy(this, 'netPnl', () => ({
            get age() { return _lazy(this, 'age', () => ({
              under1h: createBlockCumulativeDeltaSumPattern(client, 'utxos_under_1h_old_net_realized_pnl'),
              _1hTo1d: createBlockCumulativeDeltaSumPattern(client, 'utxos_1h_to_1d_old_net_realized_pnl'),
              _1dTo1w: createBlockCumulativeDeltaSumPattern(client, 'utxos_1d_to_1w_old_net_realized_pnl'),
              _1wTo1m: createBlockCumulativeDeltaSumPattern(client, 'utxos_1w_to_1m_old_net_realized_pnl'),
              _1mTo2m: createBlockCumulativeDeltaSumPattern(client, 'utxos_1m_to_2m_old_net_realized_pnl'),
              _2mTo3m: createBlockCumulativeDeltaSumPattern(client, 'utxos_2m_to_3m_old_net_realized_pnl'),
              _3mTo4m: createBlockCumulativeDeltaSumPattern(client, 'utxos_3m_to_4m_old_net_realized_pnl'),
              _4mTo5m: createBlockCumulativeDeltaSumPattern(client, 'utxos_4m_to_5m_old_net_realized_pnl'),
              _5mTo6m: createBlockCumulativeDeltaSumPattern(client, 'utxos_5m_to_6m_old_net_realized_pnl'),
              _6mTo9m: createBlockCumulativeDeltaSumPattern(client, 'utxos_6m_to_9m_old_net_realized_pnl'),
              _9mTo1y: createBlockCumulativeDeltaSumPattern(client, 'utxos_9m_to_1y_old_net_realized_pnl'),
              _1yTo18m: createBlockCumulativeDeltaSumPattern(client, 'utxos_1y_to_18m_old_net_realized_pnl'),
              _18mTo2y: createBlockCumulativeDeltaSumPattern(client, 'utxos_18m_to_2y_old_net_realized_pnl'),
              _2yTo3y: createBlockCumulativeDeltaSumPattern(client, 'utxos_2y_to_3y_old_net_realized_pnl'),
              _3yTo4y: createBlockCumulativeDeltaSumPattern(client, 'utxos_3y_to_4y_old_net_realized_pnl'),
              _4yTo5y: createBlockCumulativeDeltaSumPattern(client, 'utxos_4y_to_5y_old_net_realized_pnl'),
              _5yTo6y: createBlockCumulativeDeltaSumPattern(client, 'utxos_5y_to_6y_old_net_realized_pnl'),
              _6yTo7y: createBlockCumulativeDeltaSumPattern(client, 'utxos_6y_to_7y_old_net_realized_pnl'),
              _7yTo8y: createBlockCumulativeDeltaSumPattern(client, 'utxos_7y_to_8y_old_net_realized_pnl'),
              _8yTo10y: createBlockCumulativeDeltaSumPattern(client, 'utxos_8y_to_10y_old_net_realized_pnl'),
              _10yTo12y: createBlockCumulativeDeltaSumPattern(client, 'utxos_10y_to_12y_old_net_realized_pnl'),
              _12yTo15y: createBlockCumulativeDeltaSumPattern(client, 'utxos_12y_to_15y_old_net_realized_pnl'),
              over15y: createBlockCumulativeDeltaSumPattern(client, 'utxos_over_15y_old_net_realized_pnl'),
            })); },
            get epoch() { return _lazy(this, 'epoch', () => ({
              _0: createBlockCumulativeDeltaSumPattern(client, 'epoch_0_net_realized_pnl'),
              _1: createBlockCumulativeDeltaSumPattern(client, 'epoch_1_net_realized_pnl'),
              _2: createBlockCumulativeDeltaSumPattern(client, 'epoch_2_net_realized_pnl'),
              _3: createBlockCumulativeDeltaSumPattern(client, 'epoch_3_net_realized_pnl'),
              _4: createBlockCumulativeDeltaSumPattern(client, 'epoch_4_net_realized_pnl'),
            })); },
            get class() { return _lazy(this, 'class', () => ({
              _2009: createBlockCumulativeDeltaSumPattern(client, 'class_2009_net_realized_pnl'),
              _2010: createBlockCumulativeDeltaSumPattern(client, 'class_2010_net_realized_pnl'),
              _2011: createBlockCumulativeDeltaSumPattern(client, 'class_2011_net_realized_pnl'),
              _2012: createBlockCumulativeDeltaSumPattern(client, 'class_2012_net_realized_pnl'),
              _2013: createBlockCumulativeDeltaSumPattern(client, 'class_2013_net_realized_pnl'),
              _2014: createBlockCumulativeDeltaSumPattern(client, 'class_2014_net_realized_pnl'),
              _2015: createBlockCumulativeDeltaSumPattern(client, 'class_2015_net_realized_pnl'),
              _2016: createBlockCumulativeDeltaSumPattern(client, 'class_2016_net_realized_pnl'),
              _2017: createBlockCumulativeDeltaSumPattern(client, 'class_2017_net_realized_pnl'),
              _2018: createBlockCumulativeDeltaSumPattern(client, 'class_2018_net_realized_pnl'),
              _2019: createBlockCumulativeDeltaSumPattern(client, 'class_2019_net_realized_pnl'),
              _2020: createBlockCumulativeDeltaSumPattern(client, 'class_2020_net_realized_pnl'),
              _2021: createBlockCumulativeDeltaSumPattern(client, 'class_2021_net_realized_pnl'),
              _2022: createBlockCumulativeDeltaSumPattern(client, 'class_2022_net_realized_pnl'),
              _2023: createBlockCumulativeDeltaSumPattern(client, 'class_2023_net_realized_pnl'),
              _2024: createBlockCumulativeDeltaSumPattern(client, 'class_2024_net_realized_pnl'),
              _2025: createBlockCumulativeDeltaSumPattern(client, 'class_2025_net_realized_pnl'),
              _2026: createBlockCumulativeDeltaSumPattern(client, 'class_2026_net_realized_pnl'),
            })); },
          })); },
          get valueDestroyed() { return _lazy(this, 'valueDestroyed', () => ({
            get age() { return _lazy(this, 'age', () => ({
              under1h: createAverageBlockCumulativeSumPattern3(client, 'utxos_under_1h_old_value_destroyed'),
              _1hTo1d: createAverageBlockCumulativeSumPattern3(client, 'utxos_1h_to_1d_old_value_destroyed'),
              _1dTo1w: createAverageBlockCumulativeSumPattern3(client, 'utxos_1d_to_1w_old_value_destroyed'),
              _1wTo1m: createAverageBlockCumulativeSumPattern3(client, 'utxos_1w_to_1m_old_value_destroyed'),
              _1mTo2m: createAverageBlockCumulativeSumPattern3(client, 'utxos_1m_to_2m_old_value_destroyed'),
              _2mTo3m: createAverageBlockCumulativeSumPattern3(client, 'utxos_2m_to_3m_old_value_destroyed'),
              _3mTo4m: createAverageBlockCumulativeSumPattern3(client, 'utxos_3m_to_4m_old_value_destroyed'),
              _4mTo5m: createAverageBlockCumulativeSumPattern3(client, 'utxos_4m_to_5m_old_value_destroyed'),
              _5mTo6m: createAverageBlockCumulativeSumPattern3(client, 'utxos_5m_to_6m_old_value_destroyed'),
              _6mTo9m: createAverageBlockCumulativeSumPattern3(client, 'utxos_6m_to_9m_old_value_destroyed'),
              _9mTo1y: createAverageBlockCumulativeSumPattern3(client, 'utxos_9m_to_1y_old_value_destroyed'),
              _1yTo18m: createAverageBlockCumulativeSumPattern3(client, 'utxos_1y_to_18m_old_value_destroyed'),
              _18mTo2y: createAverageBlockCumulativeSumPattern3(client, 'utxos_18m_to_2y_old_value_destroyed'),
              _2yTo3y: createAverageBlockCumulativeSumPattern3(client, 'utxos_2y_to_3y_old_value_destroyed'),
              _3yTo4y: createAverageBlockCumulativeSumPattern3(client, 'utxos_3y_to_4y_old_value_destroyed'),
              _4yTo5y: createAverageBlockCumulativeSumPattern3(client, 'utxos_4y_to_5y_old_value_destroyed'),
              _5yTo6y: createAverageBlockCumulativeSumPattern3(client, 'utxos_5y_to_6y_old_value_destroyed'),
              _6yTo7y: createAverageBlockCumulativeSumPattern3(client, 'utxos_6y_to_7y_old_value_destroyed'),
              _7yTo8y: createAverageBlockCumulativeSumPattern3(client, 'utxos_7y_to_8y_old_value_destroyed'),
              _8yTo10y: createAverageBlockCumulativeSumPattern3(client, 'utxos_8y_to_10y_old_value_destroyed'),
              _10yTo12y: createAverageBlockCumulativeSumPattern3(client, 'utxos_10y_to_12y_old_value_destroyed'),
              _12yTo15y: createAverageBlockCumulativeSumPattern3(client, 'utxos_12y_to_15y_old_value_destroyed'),
              over15y: createAverageBlockCumulativeSumPattern3(client, 'utxos_over_15y_old_value_destroyed'),
            })); },
            get epoch() { return _lazy(this, 'epoch', () => ({
              _0: createAverageBlockCumulativeSumPattern3(client, 'epoch_0_value_destroyed'),
              _1: createAverageBlockCumulativeSumPattern3(client, 'epoch_1_value_destroyed'),
              _2: createAverageBlockCumulativeSumPattern3(client, 'epoch_2_value_destroyed'),
              _3: createAverageBlockCumulativeSumPattern3(client, 'epoch_3_value_destroyed'),
              _4: createAverageBlockCumulativeSumPattern3(client, 'epoch_4_value_destroyed'),
            })); },
            get class() { return _lazy(this, 'class', () => ({
              _2009: createAverageBlockCumulativeSumPattern3(client, 'class_2009_value_destroyed'),
              _2010: createAverageBlockCumulativeSumPattern3(client, 'class_2010_value_destroyed'),
              _2011: createAverageBlockCumulativeSumPattern3(client, 'class_2011_value_destroyed'),
              _2012: createAverageBlockCumulativeSumPattern3(client, 'class_2012_value_destroyed'),
              _2013: createAverageBlockCumulativeSumPattern3(client, 'class_2013_value_destroyed'),
              _2014: createAverageBlockCumulativeSumPattern3(client, 'class_2014_value_destroyed'),
              _2015: createAverageBlockCumulativeSumPattern3(client, 'class_2015_value_destroyed'),
              _2016: createAverageBlockCumulativeSumPattern3(client, 'class_2016_value_destroyed'),
              _2017: createAverageBlockCumulativeSumPattern3(client, 'class_2017_value_destroyed'),
              _2018: createAverageBlockCumulativeSumPattern3(client, 'class_2018_value_destroyed'),
              _2019: createAverageBlockCumulativeSumPattern3(client, 'class_2019_value_destroyed'),
              _2020: createAverageBlockCumulativeSumPattern3(client, 'class_2020_value_destroyed'),
              _2021: createAverageBlockCumulativeSumPattern3(client, 'class_2021_value_destroyed'),
              _2022: createAverageBlockCumulativeSumPattern3(client, 'class_2022_value_destroyed'),
              _2023: createAverageBlockCumulativeSumPattern3(client, 'class_2023_value_destroyed'),
              _2024: createAverageBlockCumulativeSumPattern3(client, 'class_2024_value_destroyed'),
              _2025: createAverageBlockCumulativeSumPattern3(client, 'class_2025_value_destroyed'),
              _2026: createAverageBlockCumulativeSumPattern3(client, 'class_2026_value_destroyed'),
            })); },
          })); },
          get price() { return _lazy(this, 'price', () => ({
            get utxoAmount() { return _lazy(this, 'utxoAmount', () => ({
              _0sats: createCentsSatsUsdPattern(client, 'utxos_0sats_realized_price'),
              _1satTo10sats: createCentsSatsUsdPattern(client, 'utxos_1sat_to_10sats_realized_price'),
              _10satsTo100sats: createCentsSatsUsdPattern(client, 'utxos_10sats_to_100sats_realized_price'),
              _100satsTo1kSats: createCentsSatsUsdPattern(client, 'utxos_100sats_to_1k_sats_realized_price'),
              _1kSatsTo10kSats: createCentsSatsUsdPattern(client, 'utxos_1k_sats_to_10k_sats_realized_price'),
              _10kSatsTo100kSats: createCentsSatsUsdPattern(client, 'utxos_10k_sats_to_100k_sats_realized_price'),
              _100kSatsTo1mSats: createCentsSatsUsdPattern(client, 'utxos_100k_sats_to_1m_sats_realized_price'),
              _1mSatsTo10mSats: createCentsSatsUsdPattern(client, 'utxos_1m_sats_to_10m_sats_realized_price'),
              _10mSatsTo1btc: createCentsSatsUsdPattern(client, 'utxos_10m_sats_to_1btc_realized_price'),
              _1btcTo10btc: createCentsSatsUsdPattern(client, 'utxos_1btc_to_10btc_realized_price'),
              _10btcTo100btc: createCentsSatsUsdPattern(client, 'utxos_10btc_to_100btc_realized_price'),
              _100btcTo1kBtc: createCentsSatsUsdPattern(client, 'utxos_100btc_to_1k_btc_realized_price'),
              _1kBtcTo10kBtc: createCentsSatsUsdPattern(client, 'utxos_1k_btc_to_10k_btc_realized_price'),
              _10kBtcTo100kBtc: createCentsSatsUsdPattern(client, 'utxos_10k_btc_to_100k_btc_realized_price'),
              over100kBtc: createCentsSatsUsdPattern(client, 'utxos_over_100k_btc_realized_price'),
            })); },
            get type() { return _lazy(this, 'type', () => ({
              p2pk65: createCentsSatsUsdPattern(client, 'p2pk65_realized_price'),
              p2pk33: createCentsSatsUsdPattern(client, 'p2pk33_realized_price'),
              p2pkh: createCentsSatsUsdPattern(client, 'p2pkh_realized_price'),
              p2ms: createCentsSatsUsdPattern(client, 'p2ms_realized_price'),
              p2sh: createCentsSatsUsdPattern(client, 'p2sh_realized_price'),
              p2wpkh: createCentsSatsUsdPattern(client, 'p2wpkh_realized_price'),
              p2wsh: createCentsSatsUsdPattern(client, 'p2wsh_realized_price'),
              p2tr: createCentsSatsUsdPattern(client, 'p2tr_realized_price'),
              p2a: createCentsSatsUsdPattern(client, 'p2a_realized_price'),
              unknown: createCentsSatsUsdPattern(client, 'unknown_outputs_realized_price'),
              empty: createCentsSatsUsdPattern(client, 'empty_outputs_realized_price'),
            })); },
          })); },
        })); },
        get unrealized() { return _lazy(this, 'unrealized', () => ({
          get profit() { return _lazy(this, 'profit', () => ({
            get age() { return _lazy(this, 'age', () => ({
              under1h: createCentsUsdPattern(client, 'utxos_under_1h_old_unrealized_profit'),
              _1hTo1d: createCentsUsdPattern(client, 'utxos_1h_to_1d_old_unrealized_profit'),
              _1dTo1w: createCentsUsdPattern(client, 'utxos_1d_to_1w_old_unrealized_profit'),
              _1wTo1m: createCentsUsdPattern(client, 'utxos_1w_to_1m_old_unrealized_profit'),
              _1mTo2m: createCentsUsdPattern(client, 'utxos_1m_to_2m_old_unrealized_profit'),
              _2mTo3m: createCentsUsdPattern(client, 'utxos_2m_to_3m_old_unrealized_profit'),
              _3mTo4m: createCentsUsdPattern(client, 'utxos_3m_to_4m_old_unrealized_profit'),
              _4mTo5m: createCentsUsdPattern(client, 'utxos_4m_to_5m_old_unrealized_profit'),
              _5mTo6m: createCentsUsdPattern(client, 'utxos_5m_to_6m_old_unrealized_profit'),
              _6mTo9m: createCentsUsdPattern(client, 'utxos_6m_to_9m_old_unrealized_profit'),
              _9mTo1y: createCentsUsdPattern(client, 'utxos_9m_to_1y_old_unrealized_profit'),
              _1yTo18m: createCentsUsdPattern(client, 'utxos_1y_to_18m_old_unrealized_profit'),
              _18mTo2y: createCentsUsdPattern(client, 'utxos_18m_to_2y_old_unrealized_profit'),
              _2yTo3y: createCentsUsdPattern(client, 'utxos_2y_to_3y_old_unrealized_profit'),
              _3yTo4y: createCentsUsdPattern(client, 'utxos_3y_to_4y_old_unrealized_profit'),
              _4yTo5y: createCentsUsdPattern(client, 'utxos_4y_to_5y_old_unrealized_profit'),
              _5yTo6y: createCentsUsdPattern(client, 'utxos_5y_to_6y_old_unrealized_profit'),
              _6yTo7y: createCentsUsdPattern(client, 'utxos_6y_to_7y_old_unrealized_profit'),
              _7yTo8y: createCentsUsdPattern(client, 'utxos_7y_to_8y_old_unrealized_profit'),
              _8yTo10y: createCentsUsdPattern(client, 'utxos_8y_to_10y_old_unrealized_profit'),
              _10yTo12y: createCentsUsdPattern(client, 'utxos_10y_to_12y_old_unrealized_profit'),
              _12yTo15y: createCentsUsdPattern(client, 'utxos_12y_to_15y_old_unrealized_profit'),
              over15y: createCentsUsdPattern(client, 'utxos_over_15y_old_unrealized_profit'),
            })); },
            get epoch() { return _lazy(this, 'epoch', () => ({
              _0: createCentsUsdPattern(client, 'epoch_0_unrealized_profit'),
              _1: createCentsUsdPattern(client, 'epoch_1_unrealized_profit'),
              _2: createCentsUsdPattern(client, 'epoch_2_unrealized_profit'),
              _3: createCentsUsdPattern(client, 'epoch_3_unrealized_profit'),
              _4: createCentsUsdPattern(client, 'epoch_4_unrealized_profit'),
            })); },
            get class() { return _lazy(this, 'class', () => ({
              _2009: createCentsUsdPattern(client, 'class_2009_unrealized_profit'),
              _2010: createCentsUsdPattern(client, 'class_2010_unrealized_profit'),
              _2011: createCentsUsdPattern(client, 'class_2011_unrealized_profit'),
              _2012: createCentsUsdPattern(client, 'class_2012_unrealized_profit'),
              _2013: createCentsUsdPattern(client, 'class_2013_unrealized_profit'),
              _2014: createCentsUsdPattern(client, 'class_2014_unrealized_profit'),
              _2015: createCentsUsdPattern(client, 'class_2015_unrealized_profit'),
              _2016: createCentsUsdPattern(client, 'class_2016_unrealized_profit'),
              _2017: createCentsUsdPattern(client, 'class_2017_unrealized_profit'),
              _2018: createCentsUsdPattern(client, 'class_2018_unrealized_profit'),
              _2019: createCentsUsdPattern(client, 'class_2019_unrealized_profit'),
              _2020: createCentsUsdPattern(client, 'class_2020_unrealized_profit'),
              _2021: createCentsUsdPattern(client, 'class_2021_unrealized_profit'),
              _2022: createCentsUsdPattern(client, 'class_2022_unrealized_profit'),
              _2023: createCentsUsdPattern(client, 'class_2023_unrealized_profit'),
              _2024: createCentsUsdPattern(client, 'class_2024_unrealized_profit'),
              _2025: createCentsUsdPattern(client, 'class_2025_unrealized_profit'),
              _2026: createCentsUsdPattern(client, 'class_2026_unrealized_profit'),
            })); },
          })); },
          get loss() { return _lazy(this, 'loss', () => ({
            get age() { return _lazy(this, 'age', () => ({
              under1h: createCentsUsdPattern(client, 'utxos_under_1h_old_unrealized_loss'),
              _1hTo1d: createCentsUsdPattern(client, 'utxos_1h_to_1d_old_unrealized_loss'),
              _1dTo1w: createCentsUsdPattern(client, 'utxos_1d_to_1w_old_unrealized_loss'),
              _1wTo1m: createCentsUsdPattern(client, 'utxos_1w_to_1m_old_unrealized_loss'),
              _1mTo2m: createCentsUsdPattern(client, 'utxos_1m_to_2m_old_unrealized_loss'),
              _2mTo3m: createCentsUsdPattern(client, 'utxos_2m_to_3m_old_unrealized_loss'),
              _3mTo4m: createCentsUsdPattern(client, 'utxos_3m_to_4m_old_unrealized_loss'),
              _4mTo5m: createCentsUsdPattern(client, 'utxos_4m_to_5m_old_unrealized_loss'),
              _5mTo6m: createCentsUsdPattern(client, 'utxos_5m_to_6m_old_unrealized_loss'),
              _6mTo9m: createCentsUsdPattern(client, 'utxos_6m_to_9m_old_unrealized_loss'),
              _9mTo1y: createCentsUsdPattern(client, 'utxos_9m_to_1y_old_unrealized_loss'),
              _1yTo18m: createCentsUsdPattern(client, 'utxos_1y_to_18m_old_unrealized_loss'),
              _18mTo2y: createCentsUsdPattern(client, 'utxos_18m_to_2y_old_unrealized_loss'),
              _2yTo3y: createCentsUsdPattern(client, 'utxos_2y_to_3y_old_unrealized_loss'),
              _3yTo4y: createCentsUsdPattern(client, 'utxos_3y_to_4y_old_unrealized_loss'),
              _4yTo5y: createCentsUsdPattern(client, 'utxos_4y_to_5y_old_unrealized_loss'),
              _5yTo6y: createCentsUsdPattern(client, 'utxos_5y_to_6y_old_unrealized_loss'),
              _6yTo7y: createCentsUsdPattern(client, 'utxos_6y_to_7y_old_unrealized_loss'),
              _7yTo8y: createCentsUsdPattern(client, 'utxos_7y_to_8y_old_unrealized_loss'),
              _8yTo10y: createCentsUsdPattern(client, 'utxos_8y_to_10y_old_unrealized_loss'),
              _10yTo12y: createCentsUsdPattern(client, 'utxos_10y_to_12y_old_unrealized_loss'),
              _12yTo15y: createCentsUsdPattern(client, 'utxos_12y_to_15y_old_unrealized_loss'),
              over15y: createCentsUsdPattern(client, 'utxos_over_15y_old_unrealized_loss'),
            })); },
            get epoch() { return _lazy(this, 'epoch', () => ({
              _0: createCentsUsdPattern(client, 'epoch_0_unrealized_loss'),
              _1: createCentsUsdPattern(client, 'epoch_1_unrealized_loss'),
              _2: createCentsUsdPattern(client, 'epoch_2_unrealized_loss'),
              _3: createCentsUsdPattern(client, 'epoch_3_unrealized_loss'),
              _4: createCentsUsdPattern(client, 'epoch_4_unrealized_loss'),
            })); },
            get class() { return _lazy(this, 'class', () => ({
              _2009: createCentsUsdPattern(client, 'class_2009_unrealized_loss'),
              _2010: createCentsUsdPattern(client, 'class_2010_unrealized_loss'),
              _2011: createCentsUsdPattern(client, 'class_2011_unrealized_loss'),
              _2012: createCentsUsdPattern(client, 'class_2012_unrealized_loss'),
              _2013: createCentsUsdPattern(client, 'class_2013_unrealized_loss'),
              _2014: createCentsUsdPattern(client, 'class_2014_unrealized_loss'),
              _2015: createCentsUsdPattern(client, 'class_2015_unrealized_loss'),
              _2016: createCentsUsdPattern(client, 'class_2016_unrealized_loss'),
              _2017: createCentsUsdPattern(client, 'class_2017_unrealized_loss'),
              _2018: createCentsUsdPattern(client, 'class_2018_unrealized_loss'),
              _2019: createCentsUsdPattern(client, 'class_2019_unrealized_loss'),
              _2020: createCentsUsdPattern(client, 'class_2020_unrealized_loss'),
              _2021: createCentsUsdPattern(client, 'class_2021_unrealized_loss'),
              _2022: createCentsUsdPattern(client, 'class_2022_unrealized_loss'),
              _2023: createCentsUsdPattern(client, 'class_2023_unrealized_loss'),
              _2024: createCentsUsdPattern(client, 'class_2024_unrealized_loss'),
              _2025: createCentsUsdPattern(client, 'class_2025_unrealized_loss'),
              _2026: createCentsUsdPattern(client, 'class_2026_unrealized_loss'),
            })); },
          })); },
          get netPnl() { return _lazy(this, 'netPnl', () => ({
            get age() { return _lazy(this, 'age', () => ({
              under1h: createCentsUsdPattern4(client, 'utxos_under_1h_old_net_unrealized_pnl'),
              _1hTo1d: createCentsUsdPattern4(client, 'utxos_1h_to_1d_old_net_unrealized_pnl'),
              _1dTo1w: createCentsUsdPattern4(client, 'utxos_1d_to_1w_old_net_unrealized_pnl'),
              _1wTo1m: createCentsUsdPattern4(client, 'utxos_1w_to_1m_old_net_unrealized_pnl'),
              _1mTo2m: createCentsUsdPattern4(client, 'utxos_1m_to_2m_old_net_unrealized_pnl'),
              _2mTo3m: createCentsUsdPattern4(client, 'utxos_2m_to_3m_old_net_unrealized_pnl'),
              _3mTo4m: createCentsUsdPattern4(client, 'utxos_3m_to_4m_old_net_unrealized_pnl'),
              _4mTo5m: createCentsUsdPattern4(client, 'utxos_4m_to_5m_old_net_unrealized_pnl'),
              _5mTo6m: createCentsUsdPattern4(client, 'utxos_5m_to_6m_old_net_unrealized_pnl'),
              _6mTo9m: createCentsUsdPattern4(client, 'utxos_6m_to_9m_old_net_unrealized_pnl'),
              _9mTo1y: createCentsUsdPattern4(client, 'utxos_9m_to_1y_old_net_unrealized_pnl'),
              _1yTo18m: createCentsUsdPattern4(client, 'utxos_1y_to_18m_old_net_unrealized_pnl'),
              _18mTo2y: createCentsUsdPattern4(client, 'utxos_18m_to_2y_old_net_unrealized_pnl'),
              _2yTo3y: createCentsUsdPattern4(client, 'utxos_2y_to_3y_old_net_unrealized_pnl'),
              _3yTo4y: createCentsUsdPattern4(client, 'utxos_3y_to_4y_old_net_unrealized_pnl'),
              _4yTo5y: createCentsUsdPattern4(client, 'utxos_4y_to_5y_old_net_unrealized_pnl'),
              _5yTo6y: createCentsUsdPattern4(client, 'utxos_5y_to_6y_old_net_unrealized_pnl'),
              _6yTo7y: createCentsUsdPattern4(client, 'utxos_6y_to_7y_old_net_unrealized_pnl'),
              _7yTo8y: createCentsUsdPattern4(client, 'utxos_7y_to_8y_old_net_unrealized_pnl'),
              _8yTo10y: createCentsUsdPattern4(client, 'utxos_8y_to_10y_old_net_unrealized_pnl'),
              _10yTo12y: createCentsUsdPattern4(client, 'utxos_10y_to_12y_old_net_unrealized_pnl'),
              _12yTo15y: createCentsUsdPattern4(client, 'utxos_12y_to_15y_old_net_unrealized_pnl'),
              over15y: createCentsUsdPattern4(client, 'utxos_over_15y_old_net_unrealized_pnl'),
            })); },
            get epoch() { return _lazy(this, 'epoch', () => ({
              _0: createCentsUsdPattern4(client, 'epoch_0_net_unrealized_pnl'),
              _1: createCentsUsdPattern4(client, 'epoch_1_net_unrealized_pnl'),
              _2: createCentsUsdPattern4(client, 'epoch_2_net_unrealized_pnl'),
              _3: createCentsUsdPattern4(client, 'epoch_3_net_unrealized_pnl'),
              _4: createCentsUsdPattern4(client, 'epoch_4_net_unrealized_pnl'),
            })); },
            get class() { return _lazy(this, 'class', () => ({
              _2009: createCentsUsdPattern4(client, 'class_2009_net_unrealized_pnl'),
              _2010: createCentsUsdPattern4(client, 'class_2010_net_unrealized_pnl'),
              _2011: createCentsUsdPattern4(client, 'class_2011_net_unrealized_pnl'),
              _2012: createCentsUsdPattern4(client, 'class_2012_net_unrealized_pnl'),
              _2013: createCentsUsdPattern4(client, 'class_2013_net_unrealized_pnl'),
              _2014: createCentsUsdPattern4(client, 'class_2014_net_unrealized_pnl'),
              _2015: createCentsUsdPattern4(client, 'class_2015_net_unrealized_pnl'),
              _2016: createCentsUsdPattern4(client, 'class_2016_net_unrealized_pnl'),
              _2017: createCentsUsdPattern4(client, 'class_2017_net_unrealized_pnl'),
              _2018: createCentsUsdPattern4(client, 'class_2018_net_unrealized_pnl'),
              _2019: createCentsUsdPattern4(client, 'class_2019_net_unrealized_pnl'),
              _2020: createCentsUsdPattern4(client, 'class_2020_net_unrealized_pnl'),
              _2021: createCentsUsdPattern4(client, 'class_2021_net_unrealized_pnl'),
              _2022: createCentsUsdPattern4(client, 'class_2022_net_unrealized_pnl'),
              _2023: createCentsUsdPattern4(client, 'class_2023_net_unrealized_pnl'),
              _2024: createCentsUsdPattern4(client, 'class_2024_net_unrealized_pnl'),
              _2025: createCentsUsdPattern4(client, 'class_2025_net_unrealized_pnl'),
              _2026: createCentsUsdPattern4(client, 'class_2026_net_unrealized_pnl'),
            })); },
          })); },
        })); },
        get urpd() { return _lazy(this, 'urpd', () => ({
          get ageBounds() { return _lazy(this, 'ageBounds', () => ({
            all: createMaxMinPattern(client, 'utxos_urpd_all_cost_basis'),
            sth: createMaxMinPattern(client, 'utxos_urpd_sth_cost_basis'),
            lth: createMaxMinPattern(client, 'utxos_urpd_lth_cost_basis'),
            under4m: createMaxMinPattern(client, 'utxos_urpd_under_4m_cost_basis'),
            under6m: createMaxMinPattern(client, 'utxos_urpd_under_6m_cost_basis'),
            over4m: createMaxMinPattern(client, 'utxos_urpd_over_4m_cost_basis'),
            over6m: createMaxMinPattern(client, 'utxos_urpd_over_6m_cost_basis'),
          })); },
        })); },
      })); },
      get distributionAggregated() { return _lazy(this, 'distributionAggregated', () => ({
        get cohorts() { return _lazy(this, 'cohorts', () => ({
          get all() { return _lazy(this, 'all', () => ({
            supply: createDeltaInTotalPattern(client, 'supply'),
            get outputs() { return _lazy(this, 'outputs', () => ({
              unspentCount: createBaseDeltaPattern(client, 'utxo_count'),
              spentCount: createAverageBlockCumulativeSumPattern2(client, 'spent_utxo_count'),
            })); },
            get activity() { return _lazy(this, 'activity', () => ({
              transferVolume: createAverageBlockCumulativeSumPattern(client, 'transfer_volume'),
              transferVolumeInProfit: createAverageBlockCumulativeSumPattern(client, 'transfer_volume_in_profit'),
              transferVolumeInLoss: createAverageBlockCumulativeSumPattern(client, 'transfer_volume_in_loss'),
              coindaysDestroyed: createAverageBlockCumulativeSumPattern2(client, 'coindays_destroyed'),
              coinyearsDestroyed: createSeriesPattern1(client, 'coinyears_destroyed'),
            })); },
            get realized() { return _lazy(this, 'realized', () => ({
              cap: createCentsDeltaUsdPattern(client, 'realized_cap'),
              price: createCentsSatsUsdPattern(client, 'realized_price'),
              capitalizedPrice: createCentsPpmRatioSatsUsdPattern(client, 'capitalized_price'),
              profit: createBlockCumulativeSumPattern(client, 'realized_profit'),
              loss: createBlockCumulativeSumPattern(client, 'realized_loss'),
              netPnl: createBlockCumulativeDeltaSumPattern(client, 'net_realized_pnl'),
              valueDestroyed: createAverageBlockCumulativeSumPattern3(client, 'value_destroyed'),
              grossPnl: createBlockCumulativeSumPattern(client, 'realized_gross_pnl'),
              peakRegret: createBlockCumulativeSumPattern(client, 'realized_peak_regret'),
              mvrv: createPpmRatioPattern2(client, 'mvrv'),
            })); },
            get unrealized() { return _lazy(this, 'unrealized', () => ({
              profit: createCentsUsdPattern(client, 'unrealized_profit'),
              loss: createCentsUsdPattern(client, 'unrealized_loss'),
              netPnl: createCentsUsdPattern4(client, 'net_unrealized_pnl'),
              grossPnl: createCentsUsdPattern(client, 'unrealized_gross_pnl'),
              investedCapitalInProfit: createCentsUsdPattern(client, 'invested_capital_in_profit'),
              investedCapitalInLoss: createCentsUsdPattern(client, 'invested_capital_in_loss'),
              painIndex: createCentsUsdPattern(client, 'pain_index'),
              greedIndex: createCentsUsdPattern(client, 'greed_index'),
              netSentiment: createCentsUsdPattern4(client, 'net_sentiment'),
              nupl: createPpmRatioPattern3(client, 'nupl'),
            })); },
            get costBasis() { return _lazy(this, 'costBasis', () => ({
              inProfit: createPerPattern2(client, 'cost_basis_in_profit_per'),
              inLoss: createPerPattern2(client, 'cost_basis_in_loss_per'),
              min: createCentsSatsUsdPattern(client, 'cost_basis_min'),
              max: createCentsSatsUsdPattern(client, 'cost_basis_max'),
              perCoin: createPct05Pct10Pct15Pct20Pct25Pct30Pct35Pct40Pct45Pct50Pct55Pct60Pct65Pct70Pct75Pct80Pct85Pct90Pct95Pattern(client, 'cost_basis_per_coin'),
              perDollar: createPct05Pct10Pct15Pct20Pct25Pct30Pct35Pct40Pct45Pct50Pct55Pct60Pct65Pct70Pct75Pct80Pct85Pct90Pct95Pattern(client, 'cost_basis_per_dollar'),
              supplyDensity: createPercentPpmRatioPattern2(client, 'supply_density'),
            })); },
            get ratios() { return _lazy(this, 'ratios', () => ({
              get adjustedSopr() { return _lazy(this, 'adjustedSopr', () => ({
                ratio: create_1m1w1y24hPattern6(client, 'adjusted_sopr'),
                transferVolume: createAverageBlockCumulativeSumPattern3(client, 'adj_value_created'),
                valueDestroyed: createAverageBlockCumulativeSumPattern3(client, 'adj_value_destroyed'),
              })); },
              dormancy: create_1m1w1y24hPattern6(client, 'dormancy'),
              sopr: createSeriesPattern1(client, 'sopr_24h'),
              soprRatioExtended: create_1m1w1yPattern(client, 'sopr'),
              sellSideRiskRatio: create_1m1w1y24hPattern9(client, 'sell_side_risk_ratio'),
              profitToLossRatio: create_1m1w1y24hPattern6(client, 'realized_profit_to_loss_ratio'),
            })); },
            get relative() { return _lazy(this, 'relative', () => ({
              supplyDominance: createPercentPpmRatioPattern2(client, 'supply_dominance'),
              supplyInProfitShare: createPercentPpmRatioPattern2(client, 'supply_in_profit_share'),
              supplyInLossShare: createPercentPpmRatioPattern2(client, 'supply_in_loss_share'),
              unrealizedProfitToMcap: createPercentPpmRatioPattern2(client, 'unrealized_profit_to_mcap'),
              unrealizedLossToMcap: createPercentPpmRatioPattern2(client, 'unrealized_loss_to_mcap'),
              unrealizedProfitToOwnMcap: createPercentPpmRatioPattern2(client, 'unrealized_profit_to_own_mcap'),
              unrealizedLossToOwnMcap: createPercentPpmRatioPattern2(client, 'unrealized_loss_to_own_mcap'),
              unrealizedProfitToOwnGrossPnl: createPercentPpmRatioPattern2(client, 'unrealized_profit_to_own_gross_pnl'),
              unrealizedLossToOwnGrossPnl: createPercentPpmRatioPattern2(client, 'unrealized_loss_to_own_gross_pnl'),
              netUnrealizedPnlToOwnGrossPnl: createPercentPpmRatioPattern3(client, 'net_unrealized_pnl_to_own_gross_pnl'),
              investedCapitalInProfitShare: createPercentPpmRatioPattern2(client, 'invested_capital_in_profit_share'),
              investedCapitalInLossShare: createPercentPpmRatioPattern2(client, 'invested_capital_in_loss_share'),
              realizedCapToOwnMcap: createPercentPpmRatioPattern2(client, 'realized_cap_to_own_mcap'),
              netPnlChange1mToMcap: createPercentPpmRatioPattern(client, 'net_pnl_change_1m_to_mcap'),
              netPnlChange1mToRcap: createPercentPpmRatioPattern(client, 'net_pnl_change_1m_to_rcap'),
            })); },
          })); },
          sth: createActivityCostOutputsRatiosRealizedRelativeSupplyUnrealizedPattern(client, 'sth'),
          lth: createActivityCostOutputsRatiosRealizedRelativeSupplyUnrealizedPattern(client, 'lth'),
          under4m: createActivityCostOutputsRatiosRealizedRelativeSupplyUnrealizedPattern(client, 'under_4m'),
          under6m: createActivityCostOutputsRatiosRealizedRelativeSupplyUnrealizedPattern(client, 'under_6m'),
          over4m: createActivityCostOutputsRatiosRealizedRelativeSupplyUnrealizedPattern(client, 'over_4m'),
          over6m: createActivityCostOutputsRatiosRealizedRelativeSupplyUnrealizedPattern(client, 'over_6m'),
        })); },
      })); },
      get supply() { return _lazy(this, 'supply', () => ({
        circulating: createBtcCentsSatsUsdPattern(client, 'circulating_supply'),
        burned: createBlockCumulativePattern(client, 'unspendable_supply'),
        inflationRate: createPercentPpmRatioPattern(client, 'inflation_rate'),
        get velocity() { return _lazy(this, 'velocity', () => ({
          native: createSeriesPattern1(client, 'velocity_btc'),
          fiat: createSeriesPattern1(client, 'velocity_usd'),
        })); },
        marketCap: createCentsDeltaUsdPattern(client, 'market_cap'),
        marketMinusRealizedCapGrowthRate: create_1m1w1y24hPattern6(client, 'market_minus_realized_cap_growth_rate'),
        hodledOrLost: createBtcCentsSatsUsdPattern(client, 'hodled_or_lost_supply'),
      })); },
      get utxoHistory() { return _lazy(this, 'utxoHistory', () => ({
        supply: createSeriesPattern18(client, 'unspent_sats'),
        count: createSeriesPattern1(client, 'utxo_count_bis'),
      })); },
    };
  }

  /**
   * Create a dynamic series endpoint builder for any series/index combination.
   *
   * Use this for programmatic access when the series name is determined at runtime.
   * For type-safe access, use the `series` tree instead.
   *
   * @template {Index} I
   * @param {string} series - The series name
   * @param {I} index - The index name; date indexes also slice by Date
   * @returns {I extends DateIndex ? DateSeriesEndpoint<unknown> : SeriesEndpoint<unknown>}
   */
  seriesEndpoint(series, index) {
    return /** @type {any} */ (_endpoint(this, series, index));
  }

  /**
   * Health check
   *
   * Local health and query-readiness check. Returns server identity, uptime, and a coherent local sync snapshot without a bitcoind round-trip. Reads the published prefix during processing; an empty index waits until the request deadline, then returns 504. Responses are not cached. For chain-tip catch-up, request `GET /api/server/sync`.
   *
   * Endpoint: `GET /health`
   * @param {{ signal?: AbortSignal, onValue?: (value: Health) => void, cache?: boolean, memCache?: boolean }} [options]
   * @returns {Promise<Health>}
   */
  async getHealth({ signal, onValue, cache, memCache } = {}) {
    const path = `/health`;
    return this.getJson(path, { signal, onValue, cache, memCache });
  }

  /**
   * API version
   *
   * Returns the current version of the API server
   *
   * Endpoint: `GET /version`
   * @param {{ signal?: AbortSignal, onValue?: (value: string) => void, cache?: boolean, memCache?: boolean }} [options]
   * @returns {Promise<string>}
   */
  async getVersion({ signal, onValue, cache, memCache } = {}) {
    const path = `/version`;
    return this.getJson(path, { signal, onValue, cache, memCache });
  }

  /**
   * Sync status
   *
   * Returns a coherent local index snapshot and a separately observed Bitcoin Core tip height. The two heights can differ during indexing or a reorg. Conditional requests refresh these observations before validation.
   *
   * Endpoint: `GET /api/server/sync`
   * @param {{ signal?: AbortSignal, onValue?: (value: SyncStatus) => void, cache?: boolean, memCache?: boolean }} [options]
   * @returns {Promise<SyncStatus>}
   */
  async getSyncStatus({ signal, onValue, cache, memCache } = {}) {
    const path = `/api/server/sync`;
    return this.getJson(path, { signal, onValue, cache, memCache });
  }

  /**
   * Disk usage
   *
   * Returns allocated file bytes for BRK and Bitcoin data. Each request scans both trees; these are independent observations, not an atomic filesystem snapshot. Conditional requests validate the newly observed totals. Directory-link cycles and excessive nesting fail without returning partial totals.
   *
   * Endpoint: `GET /api/server/disk`
   * @param {{ signal?: AbortSignal, onValue?: (value: DiskUsage) => void, cache?: boolean, memCache?: boolean }} [options]
   * @returns {Promise<DiskUsage>}
   */
  async getDiskUsage({ signal, onValue, cache, memCache } = {}) {
    const path = `/api/server/disk`;
    return this.getJson(path, { signal, onValue, cache, memCache });
  }

  /**
   * Series catalog
   *
   * Returns the complete hierarchical catalog of available series organized as a tree structure. Series are grouped by categories and subcategories.
   *
   * Endpoint: `GET /api/series`
   * @param {{ signal?: AbortSignal, onValue?: (value: TreeNode) => void, cache?: boolean, memCache?: boolean }} [options]
   * @returns {Promise<TreeNode>}
   */
  async getSeriesTree({ signal, onValue, cache, memCache } = {}) {
    const path = `/api/series`;
    return this.getJson(path, { signal, onValue, cache, memCache });
  }

  /**
   * Series count
   *
   * Returns the number of series available per index type.
   *
   * Endpoint: `GET /api/series/count`
   * @param {{ signal?: AbortSignal, onValue?: (value: DetailedSeriesCount) => void, cache?: boolean, memCache?: boolean }} [options]
   * @returns {Promise<DetailedSeriesCount>}
   */
  async getSeriesCount({ signal, onValue, cache, memCache } = {}) {
    const path = `/api/series/count`;
    return this.getJson(path, { signal, onValue, cache, memCache });
  }

  /**
   * List available indexes
   *
   * Returns all available indexes with their accepted query aliases. Use any alias when querying series.
   *
   * Endpoint: `GET /api/series/indexes`
   * @param {{ signal?: AbortSignal, onValue?: (value: IndexInfo[]) => void, cache?: boolean, memCache?: boolean }} [options]
   * @returns {Promise<IndexInfo[]>}
   */
  async getIndexes({ signal, onValue, cache, memCache } = {}) {
    const path = `/api/series/indexes`;
    return this.getJson(path, { signal, onValue, cache, memCache });
  }

  /**
   * Series list
   *
   * Paginated flat list of all available series names. Use `page` query param for pagination.
   *
   * Endpoint: `GET /api/series/list`
   *
   * @param {number=} [page] - Pagination index
   * @param {number=} [per_page] - Results per page (default: 1000, max: 1000)
   * @param {{ signal?: AbortSignal, onValue?: (value: PaginatedSeries) => void, cache?: boolean, memCache?: boolean }} [options]
   * @returns {Promise<PaginatedSeries>}
   */
  async listSeries(page, per_page, { signal, onValue, cache, memCache } = {}) {
    const params = new URLSearchParams();
    if (page !== undefined) params.set('page', String(page));
    if (per_page !== undefined) params.set('per_page', String(per_page));
    const query = params.toString();
    const path = `/api/series/list${query ? '?' + query : ''}`;
    return this.getJson(path, { signal, onValue, cache, memCache });
  }

  /**
   * Search series
   *
   * Search series by name or descriptive terms. Results prioritize whole query words in names, then descriptions, then fuzzy names, then fuzzy descriptions. Word order does not matter. Descriptions provide cohort terminology and formulas. The decoded q parameter is limited to 1024 UTF-8 bytes.
   *
   * Endpoint: `GET /api/series/search`
   *
   * @param {SeriesName} q - Search query string
   * @param {Limit=} [limit] - Maximum number of results
   * @param {{ signal?: AbortSignal, onValue?: (value: string[]) => void, cache?: boolean, memCache?: boolean }} [options]
   * @returns {Promise<string[]>}
   */
  async searchSeries(q, limit, { signal, onValue, cache, memCache } = {}) {
    const params = new URLSearchParams();
    params.set('q', String(q));
    if (limit !== undefined) params.set('limit', String(limit));
    const query = params.toString();
    const path = `/api/series/search${query ? '?' + query : ''}`;
    return this.getJson(path, { signal, onValue, cache, memCache });
  }

  /**
   * Get series info
   *
   * Returns the optional description, supported indexes, and value type for the specified series. The decoded series name is limited to 1024 UTF-8 bytes.
   *
   * Endpoint: `GET /api/series/{series}`
   *
   * @param {SeriesName} series
   * @param {{ signal?: AbortSignal, onValue?: (value: SeriesInfo) => void, cache?: boolean, memCache?: boolean }} [options]
   * @returns {Promise<SeriesInfo>}
   */
  async getSeriesInfo(series, { signal, onValue, cache, memCache } = {}) {
    const path = `/api/series/${series}`;
    return this.getJson(path, { signal, onValue, cache, memCache });
  }

  /**
   * Get series data
   *
   * Fetch data for a specific series at the given index. Use query parameters to filter by date range and format (json/csv).
   *
   * Endpoint: `GET /api/series/{series}/{index}`
   *
   * @param {SeriesName} series - Series name
   * @param {Index} index - Aggregation index
   * @param {RangeIndex=} [start] - Inclusive start: integer index, date (YYYY-MM-DD), or timestamp (ISO 8601). Negative integers count from end. Aliases: `from`, `f`, `s`
   * @param {RangeIndex=} [end] - Exclusive end: integer index, date (YYYY-MM-DD), or timestamp (ISO 8601). Negative integers count from end. Aliases: `to`, `t`, `e`
   * @param {Limit=} [limit] - Maximum number of values to return (ignored if `end` is set). Aliases: `count`, `c`, `l`
   * @param {Format=} [format] - Format of the output
   * @param {{ signal?: AbortSignal, onValue?: (value: AnySeriesData | string) => void, cache?: boolean, memCache?: boolean }} [options]
   * @returns {Promise<AnySeriesData | string>}
   */
  async getSeries(series, index, start, end, limit, format, { signal, onValue, cache, memCache } = {}) {
    const params = new URLSearchParams();
    if (start !== undefined) params.set('start', String(start));
    if (end !== undefined) params.set('end', String(end));
    if (limit !== undefined) params.set('limit', String(limit));
    if (format !== undefined) params.set('format', String(format));
    const query = params.toString();
    const path = `/api/series/${series}/${index}${query ? '?' + query : ''}`;
    if (format === 'csv') return this.getText(path, { signal, onValue, cache, memCache });
    return this.getJson(path, { signal, onValue, cache, memCache });
  }

  /**
   * Get raw series data
   *
   * Returns just the data array without the SeriesData wrapper. Supports the same range and format parameters as `GET /api/series/{series}/{index}`.
   *
   * Endpoint: `GET /api/series/{series}/{index}/data`
   *
   * @param {SeriesName} series - Series name
   * @param {Index} index - Aggregation index
   * @param {RangeIndex=} [start] - Inclusive start: integer index, date (YYYY-MM-DD), or timestamp (ISO 8601). Negative integers count from end. Aliases: `from`, `f`, `s`
   * @param {RangeIndex=} [end] - Exclusive end: integer index, date (YYYY-MM-DD), or timestamp (ISO 8601). Negative integers count from end. Aliases: `to`, `t`, `e`
   * @param {Limit=} [limit] - Maximum number of values to return (ignored if `end` is set). Aliases: `count`, `c`, `l`
   * @param {Format=} [format] - Format of the output
   * @param {{ signal?: AbortSignal, onValue?: (value: boolean[] | string) => void, cache?: boolean, memCache?: boolean }} [options]
   * @returns {Promise<boolean[] | string>}
   */
  async getSeriesData(series, index, start, end, limit, format, { signal, onValue, cache, memCache } = {}) {
    const params = new URLSearchParams();
    if (start !== undefined) params.set('start', String(start));
    if (end !== undefined) params.set('end', String(end));
    if (limit !== undefined) params.set('limit', String(limit));
    if (format !== undefined) params.set('format', String(format));
    const query = params.toString();
    const path = `/api/series/${series}/${index}/data${query ? '?' + query : ''}`;
    if (format === 'csv') return this.getText(path, { signal, onValue, cache, memCache });
    return this.getJson(path, { signal, onValue, cache, memCache });
  }

  /**
   * Get latest series value
   *
   * Returns the single most recent value for a series, unwrapped (not inside a SeriesData object).
   *
   * Endpoint: `GET /api/series/{series}/{index}/latest`
   *
   * @param {SeriesName} series - Series name
   * @param {Index} index - Aggregation index
   * @param {{ signal?: AbortSignal, onValue?: (value: *) => void, cache?: boolean, memCache?: boolean }} [options]
   * @returns {Promise<*>}
   */
  async getSeriesLatest(series, index, { signal, onValue, cache, memCache } = {}) {
    const path = `/api/series/${series}/${index}/latest`;
    return this.getJson(path, { signal, onValue, cache, memCache });
  }

  /**
   * Get series data length
   *
   * Returns the total number of data points for a series at the given index.
   *
   * Endpoint: `GET /api/series/{series}/{index}/len`
   *
   * @param {SeriesName} series - Series name
   * @param {Index} index - Aggregation index
   * @param {{ signal?: AbortSignal, onValue?: (value: number) => void, cache?: boolean, memCache?: boolean }} [options]
   * @returns {Promise<number>}
   */
  async getSeriesLen(series, index, { signal, onValue, cache, memCache } = {}) {
    const path = `/api/series/${series}/${index}/len`;
    return this.getJson(path, { signal, onValue, cache, memCache });
  }

  /**
   * Get series version
   *
   * Returns the vector's schema/computation version, not its length or latest update. Appends and reorgs do not by themselves change this version.
   *
   * Endpoint: `GET /api/series/{series}/{index}/version`
   *
   * @param {SeriesName} series - Series name
   * @param {Index} index - Aggregation index
   * @param {{ signal?: AbortSignal, onValue?: (value: Version) => void, cache?: boolean, memCache?: boolean }} [options]
   * @returns {Promise<Version>}
   */
  async getSeriesVersion(series, index, { signal, onValue, cache, memCache } = {}) {
    const path = `/api/series/${series}/${index}/version`;
    return this.getJson(path, { signal, onValue, cache, memCache });
  }

  /**
   * Bulk series data
   *
   * Fetch multiple series in a single request. Supports filtering by index and date range. Returns an array of SeriesData objects. For a single series, use `get_series` instead.
   *
   * Endpoint: `GET /api/series/bulk`
   *
   * @param {SeriesList} series - Requested series
   * @param {Index} index - Index to query
   * @param {RangeIndex=} [start] - Inclusive start: integer index, date (YYYY-MM-DD), or timestamp (ISO 8601). Negative integers count from end. Aliases: `from`, `f`, `s`
   * @param {RangeIndex=} [end] - Exclusive end: integer index, date (YYYY-MM-DD), or timestamp (ISO 8601). Negative integers count from end. Aliases: `to`, `t`, `e`
   * @param {Limit=} [limit] - Maximum number of values to return (ignored if `end` is set). Aliases: `count`, `c`, `l`
   * @param {Format=} [format] - Format of the output
   * @param {{ signal?: AbortSignal, onValue?: (value: AnySeriesData[] | string) => void, cache?: boolean, memCache?: boolean }} [options]
   * @returns {Promise<AnySeriesData[] | string>}
   */
  async getSeriesBulk(series, index, start, end, limit, format, { signal, onValue, cache, memCache } = {}) {
    const params = new URLSearchParams();
    params.set('series', String(series));
    params.set('index', String(index));
    if (start !== undefined) params.set('start', String(start));
    if (end !== undefined) params.set('end', String(end));
    if (limit !== undefined) params.set('limit', String(limit));
    if (format !== undefined) params.set('format', String(format));
    const query = params.toString();
    const path = `/api/series/bulk${query ? '?' + query : ''}`;
    if (format === 'csv') return this.getText(path, { signal, onValue, cache, memCache });
    return this.getJson(path, { signal, onValue, cache, memCache });
  }

  /**
   * Available URPD cohorts
   *
   * Cohorts for which URPD data is available. Returns names like `all`, `sth`, `lth`, `under_4m`, `under_6m`, `over_4m`, `over_6m`, `utxos_under_1h_old`.
   *
   * Endpoint: `GET /api/urpd`
   * @param {{ signal?: AbortSignal, onValue?: (value: Cohort[]) => void, cache?: boolean, memCache?: boolean }} [options]
   * @returns {Promise<Cohort[]>}
   */
  async listUrpdCohorts({ signal, onValue, cache, memCache } = {}) {
    const path = `/api/urpd`;
    return this.getJson(path, { signal, onValue, cache, memCache });
  }

  /**
   * Available URPD dates
   *
   * Dates for which a published block is available for the cohort and selected `weight`. One entry per UTC day, sorted ascending.
   *
   * Endpoint: `GET /api/urpd/{cohort}/dates`
   *
   * @param {Cohort} cohort
   * @param {UrpdWeight=} [weight] - Supply weighting. Default: raw (unweighted).
   * @param {{ signal?: AbortSignal, onValue?: (value: Date[]) => void, cache?: boolean, memCache?: boolean }} [options]
   * @returns {Promise<Date[]>}
   */
  async listUrpdDates(cohort, weight, { signal, onValue, cache, memCache } = {}) {
    const params = new URLSearchParams();
    if (weight !== undefined) params.set('weight', String(weight));
    const query = params.toString();
    const path = `/api/urpd/${cohort}/dates${query ? '?' + query : ''}`;
    return this.getJson(path, { signal, onValue, cache, memCache });
  }

  /**
   * Latest URPD
   *
   * URPD for the latest published block. The response's `date` field echoes which date was served. Returns `{ cohort, height, date, weight, aggregation, close, total_supply, buckets }`. `close` and each bucket's `price_floor`, `realized_cap`, and `unrealized_pnl` are USD; `total_supply` and bucket `supply` are BTC. `unrealized_pnl` can be negative.
   *
   * Endpoint: `GET /api/urpd/{cohort}`
   *
   * @param {Cohort} cohort
   * @param {UrpdAggregation=} [agg] - Aggregation strategy. Default: raw (no aggregation). Accepts `bucket` as alias.
   * @param {UrpdWeight=} [weight] - Supply weighting. Default: raw (unweighted).
   * @param {{ signal?: AbortSignal, onValue?: (value: Urpd) => void, cache?: boolean, memCache?: boolean }} [options]
   * @returns {Promise<Urpd>}
   */
  async getUrpd(cohort, agg, weight, { signal, onValue, cache, memCache } = {}) {
    const params = new URLSearchParams();
    if (agg !== undefined) params.set('agg', String(agg));
    if (weight !== undefined) params.set('weight', String(weight));
    const query = params.toString();
    const path = `/api/urpd/${cohort}${query ? '?' + query : ''}`;
    return this.getJson(path, { signal, onValue, cache, memCache });
  }

  /**
   * URPD at block height or date
   *
   * URPD for a cohort at a block height or the last block of a UTC day. Returns `{ cohort, height, date, weight, aggregation, close, total_supply, buckets }` where each bucket is `{ price_floor, supply, realized_cap, unrealized_pnl }`. `close`, `price_floor`, `realized_cap`, and `unrealized_pnl` are USD; `total_supply` and `supply` are BTC. `unrealized_pnl` can be negative.
   *
   * Endpoint: `GET /api/urpd/{cohort}/{point}`
   *
   * @param {Cohort} cohort
   * @param {string} point
   * @param {UrpdAggregation=} [agg] - Aggregation strategy. Default: raw (no aggregation). Accepts `bucket` as alias.
   * @param {UrpdWeight=} [weight] - Supply weighting. Default: raw (unweighted).
   * @param {{ signal?: AbortSignal, onValue?: (value: Urpd) => void, cache?: boolean, memCache?: boolean }} [options]
   * @returns {Promise<Urpd>}
   */
  async getUrpdAt(cohort, point, agg, weight, { signal, onValue, cache, memCache } = {}) {
    const params = new URLSearchParams();
    if (agg !== undefined) params.set('agg', String(agg));
    if (weight !== undefined) params.set('weight', String(weight));
    const query = params.toString();
    const path = `/api/urpd/${cohort}/${point}${query ? '?' + query : ''}`;
    return this.getJson(path, { signal, onValue, cache, memCache });
  }

  /**
   * Difficulty adjustment
   *
   * Get current difficulty adjustment progress and estimates.
   *
   * *[Mempool.space docs](https://mempool.space/docs/api/rest#get-difficulty-adjustment)*
   *
   * Endpoint: `GET /api/v1/difficulty-adjustment`
   * @param {{ signal?: AbortSignal, onValue?: (value: DifficultyAdjustment) => void, cache?: boolean, memCache?: boolean }} [options]
   * @returns {Promise<DifficultyAdjustment>}
   */
  async getDifficultyAdjustment({ signal, onValue, cache, memCache } = {}) {
    const path = `/api/v1/difficulty-adjustment`;
    return this.getJson(path, { signal, onValue, cache, memCache });
  }

  /**
   * Current BTC price
   *
   * Returns bitcoin latest price (on-chain derived, USD only).
   *
   * *[Mempool.space docs](https://mempool.space/docs/api/rest#get-price)*
   *
   * Endpoint: `GET /api/v1/prices`
   * @param {{ signal?: AbortSignal, onValue?: (value: Prices) => void, cache?: boolean, memCache?: boolean }} [options]
   * @returns {Promise<Prices>}
   */
  async getPrices({ signal, onValue, cache, memCache } = {}) {
    const path = `/api/v1/prices`;
    return this.getJson(path, { signal, onValue, cache, memCache });
  }

  /**
   * Historical price
   *
   * Completed four-hour BTC/USD closes, oldest first, labeled by interval end. With a UNIX timestamp, returns the latest nonempty completed close at or before it; before the first close returns an empty list. The current partial interval is excluded. USD only; exchangeRates is empty.
   *
   * *[Mempool.space docs](https://mempool.space/docs/api/rest#get-historical-price)*
   *
   * Endpoint: `GET /api/v1/historical-price`
   *
   * @param {Timestamp=} [timestamp]
   * @param {{ signal?: AbortSignal, onValue?: (value: HistoricalPrice) => void, cache?: boolean, memCache?: boolean }} [options]
   * @returns {Promise<HistoricalPrice>}
   */
  async getHistoricalPrice(timestamp, { signal, onValue, cache, memCache } = {}) {
    const params = new URLSearchParams();
    if (timestamp !== undefined) params.set('timestamp', String(timestamp));
    const query = params.toString();
    const path = `/api/v1/historical-price${query ? '?' + query : ''}`;
    return this.getJson(path, { signal, onValue, cache, memCache });
  }

  /**
   * Address hash-prefix matches
   *
   * Find addresses by address type and by the first 1-16 hex nibbles of RapidHash v3 over the raw address payload bytes. Intended for privacy-preserving client-side wallet discovery without sending raw addresses or xpubs. Fetch metadata with `GET /api/address/{address}`.
   *
   * Endpoint: `GET /api/address/hash-prefix/{addr_type}/{prefix}`
   *
   * @param {OutputType} addr_type
   * @param {string} prefix - First 1–16 hexadecimal nibbles of the RapidHash v3 hash over the raw
address payload bytes.
   * @param {{ signal?: AbortSignal, onValue?: (value: AddrHashPrefixMatches) => void, cache?: boolean, memCache?: boolean }} [options]
   * @returns {Promise<AddrHashPrefixMatches>}
   */
  async getAddressHashPrefixMatches(addr_type, prefix, { signal, onValue, cache, memCache } = {}) {
    const path = `/api/address/hash-prefix/${addr_type}/${prefix}`;
    return this.getJson(path, { signal, onValue, cache, memCache });
  }

  /**
   * Address information
   *
   * Retrieve address information including current balance and transaction counts. Supports all standard Bitcoin address types (P2PKH, P2SH, P2WPKH, P2WSH, P2TR).
   *
   * *[Mempool.space docs](https://mempool.space/docs/api/rest#get-address)*
   *
   * Endpoint: `GET /api/address/{address}`
   *
   * @param {Addr} address
   * @param {{ signal?: AbortSignal, onValue?: (value: AddrStats) => void, cache?: boolean, memCache?: boolean }} [options]
   * @returns {Promise<AddrStats>}
   */
  async getAddress(address, { signal, onValue, cache, memCache } = {}) {
    const path = `/api/address/${address}`;
    return this.getJson(path, { signal, onValue, cache, memCache });
  }

  /**
   * Address transactions
   *
   * Get transaction history for an address, newest first. Returns up to 50 mempool transactions plus a confirmed page sized to fill the response to 50 total (chain floor of 25, so 25-50 confirmed depending on mempool weight). To paginate further confirmed history, request `GET /api/address/{address}/txs/chain/{after_txid}` with the last returned txid.
   *
   * *[Mempool.space docs](https://mempool.space/docs/api/rest#get-address-transactions)*
   *
   * Endpoint: `GET /api/address/{address}/txs`
   *
   * @param {Addr} address
   * @param {{ signal?: AbortSignal, onValue?: (value: Transaction[]) => void, cache?: boolean, memCache?: boolean }} [options]
   * @returns {Promise<Transaction[]>}
   */
  async getAddressTxs(address, { signal, onValue, cache, memCache } = {}) {
    const path = `/api/address/${address}/txs`;
    return this.getJson(path, { signal, onValue, cache, memCache });
  }

  /**
   * Address confirmed transactions
   *
   * Get the first 25 confirmed transactions for an address. For pagination, request `GET /api/address/{address}/txs/chain/{after_txid}` with the last returned txid.
   *
   * *[Mempool.space docs](https://mempool.space/docs/api/rest#get-address-transactions-chain)*
   *
   * Endpoint: `GET /api/address/{address}/txs/chain`
   *
   * @param {Addr} address
   * @param {{ signal?: AbortSignal, onValue?: (value: Transaction[]) => void, cache?: boolean, memCache?: boolean }} [options]
   * @returns {Promise<Transaction[]>}
   */
  async getAddressConfirmedTxs(address, { signal, onValue, cache, memCache } = {}) {
    const path = `/api/address/${address}/txs/chain`;
    return this.getJson(path, { signal, onValue, cache, memCache });
  }

  /**
   * Address confirmed transactions (paginated)
   *
   * Get the next 25 confirmed transactions strictly older than `after_txid` (Esplora-canonical pagination form, matches mempool.space).
   *
   * *[Mempool.space docs](https://mempool.space/docs/api/rest#get-address-transactions-chain)*
   *
   * Endpoint: `GET /api/address/{address}/txs/chain/{after_txid}`
   *
   * @param {Addr} address
   * @param {Txid} after_txid - Last txid from the previous page (return transactions strictly older than this)
   * @param {{ signal?: AbortSignal, onValue?: (value: Transaction[]) => void, cache?: boolean, memCache?: boolean }} [options]
   * @returns {Promise<Transaction[]>}
   */
  async getAddressConfirmedTxsAfter(address, after_txid, { signal, onValue, cache, memCache } = {}) {
    const path = `/api/address/${address}/txs/chain/${after_txid}`;
    return this.getJson(path, { signal, onValue, cache, memCache });
  }

  /**
   * Address mempool transactions
   *
   * Get unconfirmed transactions for an address from the mempool, newest first (up to 50).
   *
   * *[Mempool.space docs](https://mempool.space/docs/api/rest#get-address-transactions-mempool)*
   *
   * Endpoint: `GET /api/address/{address}/txs/mempool`
   *
   * @param {Addr} address
   * @param {{ signal?: AbortSignal, onValue?: (value: Transaction[]) => void, cache?: boolean, memCache?: boolean }} [options]
   * @returns {Promise<Transaction[]>}
   */
  async getAddressMempoolTxs(address, { signal, onValue, cache, memCache } = {}) {
    const path = `/api/address/${address}/txs/mempool`;
    return this.getJson(path, { signal, onValue, cache, memCache });
  }

  /**
   * Address UTXOs
   *
   * Get unspent transaction outputs (UTXOs) for an address. Returns txid, vout, value, and confirmation status for each UTXO.
   *
   * *[Mempool.space docs](https://mempool.space/docs/api/rest#get-address-utxo)*
   *
   * Endpoint: `GET /api/address/{address}/utxo`
   *
   * @param {Addr} address
   * @param {{ signal?: AbortSignal, onValue?: (value: Utxo[]) => void, cache?: boolean, memCache?: boolean }} [options]
   * @returns {Promise<Utxo[]>}
   */
  async getAddressUtxos(address, { signal, onValue, cache, memCache } = {}) {
    const path = `/api/address/${address}/utxo`;
    return this.getJson(path, { signal, onValue, cache, memCache });
  }

  /**
   * Validate address
   *
   * Validate a Bitcoin address and get information about its type and scriptPubKey. Returns `isvalid: false` with an error message for invalid addresses.
   *
   * *[Mempool.space docs](https://mempool.space/docs/api/rest#get-address-validate)*
   *
   * Endpoint: `GET /api/v1/validate-address/{address}`
   *
   * @param {string} address - Bitcoin address to validate (can be any string)
   * @param {{ signal?: AbortSignal, onValue?: (value: AddrValidation) => void, cache?: boolean, memCache?: boolean }} [options]
   * @returns {Promise<AddrValidation>}
   */
  async validateAddress(address, { signal, onValue, cache, memCache } = {}) {
    const path = `/api/v1/validate-address/${address}`;
    return this.getJson(path, { signal, onValue, cache, memCache });
  }

  /**
   * Block information
   *
   * Retrieve block information by block hash. Returns block metadata including height, timestamp, difficulty, size, weight, and transaction count.
   *
   * *[Mempool.space docs](https://mempool.space/docs/api/rest#get-block)*
   *
   * Endpoint: `GET /api/block/{hash}`
   *
   * @param {BlockHash} hash
   * @param {{ signal?: AbortSignal, onValue?: (value: BlockInfo) => void, cache?: boolean, memCache?: boolean }} [options]
   * @returns {Promise<BlockInfo>}
   */
  async getBlock(hash, { signal, onValue, cache, memCache } = {}) {
    const path = `/api/block/${hash}`;
    return this.getJson(path, { signal, onValue, cache, memCache });
  }

  /**
   * Block (v1)
   *
   * Returns block details with extras by hash.
   *
   * *[Mempool.space docs](https://mempool.space/docs/api/rest#get-block-v1)*
   *
   * Endpoint: `GET /api/v1/block/{hash}`
   *
   * @param {BlockHash} hash
   * @param {{ signal?: AbortSignal, onValue?: (value: BlockInfoV1) => void, cache?: boolean, memCache?: boolean }} [options]
   * @returns {Promise<BlockInfoV1>}
   */
  async getBlockV1(hash, { signal, onValue, cache, memCache } = {}) {
    const path = `/api/v1/block/${hash}`;
    return this.getJson(path, { signal, onValue, cache, memCache });
  }

  /**
   * Block header
   *
   * Returns the hex-encoded 80-byte block header.
   *
   * *[Mempool.space docs](https://mempool.space/docs/api/rest#get-block-header)*
   *
   * Endpoint: `GET /api/block/{hash}/header`
   *
   * @param {BlockHash} hash
   * @param {{ signal?: AbortSignal, onValue?: (value: Hex) => void, cache?: boolean, memCache?: boolean }} [options]
   * @returns {Promise<Hex>}
   */
  async getBlockHeader(hash, { signal, onValue, cache, memCache } = {}) {
    const path = `/api/block/${hash}/header`;
    return this.getText(path, { signal, onValue, cache, memCache });
  }

  /**
   * Block hash by height
   *
   * Retrieve the block hash at a given height. Returns the hash as plain text.
   *
   * *[Mempool.space docs](https://mempool.space/docs/api/rest#get-block-height)*
   *
   * Endpoint: `GET /api/block-height/{height}`
   *
   * @param {Height} height
   * @param {{ signal?: AbortSignal, onValue?: (value: BlockHash) => void, cache?: boolean, memCache?: boolean }} [options]
   * @returns {Promise<BlockHash>}
   */
  async getBlockByHeight(height, { signal, onValue, cache, memCache } = {}) {
    const path = `/api/block-height/${height}`;
    return this.getText(path, { signal, onValue, cache, memCache });
  }

  /**
   * Block by timestamp
   *
   * Find the block with the greatest header timestamp at or before the given UNIX timestamp, choosing the earliest height on ties.
   *
   * *[Mempool.space docs](https://mempool.space/docs/api/rest#get-block-timestamp)*
   *
   * Endpoint: `GET /api/v1/mining/blocks/timestamp/{timestamp}`
   *
   * @param {Timestamp} timestamp
   * @param {{ signal?: AbortSignal, onValue?: (value: BlockTimestamp) => void, cache?: boolean, memCache?: boolean }} [options]
   * @returns {Promise<BlockTimestamp>}
   */
  async getBlockByTimestamp(timestamp, { signal, onValue, cache, memCache } = {}) {
    const path = `/api/v1/mining/blocks/timestamp/${timestamp}`;
    return this.getJson(path, { signal, onValue, cache, memCache });
  }

  /**
   * Raw block
   *
   * Returns the raw block data in binary format.
   *
   * *[Mempool.space docs](https://mempool.space/docs/api/rest#get-block-raw)*
   *
   * Endpoint: `GET /api/block/{hash}/raw`
   *
   * @param {BlockHash} hash
   * @param {{ signal?: AbortSignal, onValue?: (value: Uint8Array) => void, cache?: boolean, memCache?: boolean }} [options]
   * @returns {Promise<Uint8Array>}
   */
  async getBlockRaw(hash, { signal, onValue, cache, memCache } = {}) {
    const path = `/api/block/${hash}/raw`;
    return this.getBytes(path, { signal, onValue, cache, memCache });
  }

  /**
   * Block status
   *
   * Retrieve the status of a block. Returns whether the block is in the best chain and, if so, its height and the hash of the next block.
   *
   * *[Mempool.space docs](https://mempool.space/docs/api/rest#get-block-status)*
   *
   * Endpoint: `GET /api/block/{hash}/status`
   *
   * @param {BlockHash} hash
   * @param {{ signal?: AbortSignal, onValue?: (value: BlockStatus) => void, cache?: boolean, memCache?: boolean }} [options]
   * @returns {Promise<BlockStatus>}
   */
  async getBlockStatus(hash, { signal, onValue, cache, memCache } = {}) {
    const path = `/api/block/${hash}/status`;
    return this.getJson(path, { signal, onValue, cache, memCache });
  }

  /**
   * Block tip height
   *
   * Returns the height of the last block.
   *
   * *[Mempool.space docs](https://mempool.space/docs/api/rest#get-block-tip-height)*
   *
   * Endpoint: `GET /api/blocks/tip/height`
   * @param {{ signal?: AbortSignal, onValue?: (value: Height) => void, cache?: boolean, memCache?: boolean }} [options]
   * @returns {Promise<Height>}
   */
  async getBlockTipHeight({ signal, onValue, cache, memCache } = {}) {
    const path = `/api/blocks/tip/height`;
    return Number(await this.getText(path, { signal, cache, memCache, onValue: onValue ? (v) => onValue(Number(v)) : undefined }));
  }

  /**
   * Block tip hash
   *
   * Returns the hash of the last block.
   *
   * *[Mempool.space docs](https://mempool.space/docs/api/rest#get-block-tip-hash)*
   *
   * Endpoint: `GET /api/blocks/tip/hash`
   * @param {{ signal?: AbortSignal, onValue?: (value: BlockHash) => void, cache?: boolean, memCache?: boolean }} [options]
   * @returns {Promise<BlockHash>}
   */
  async getBlockTipHash({ signal, onValue, cache, memCache } = {}) {
    const path = `/api/blocks/tip/hash`;
    return this.getText(path, { signal, onValue, cache, memCache });
  }

  /**
   * Transaction ID at index
   *
   * Retrieve a single transaction ID at a specific index within a block. Returns plain text txid.
   *
   * *[Mempool.space docs](https://mempool.space/docs/api/rest#get-block-transaction-id)*
   *
   * Endpoint: `GET /api/block/{hash}/txid/{index}`
   *
   * @param {BlockHash} hash - Bitcoin block hash
   * @param {BlockTxIndex} index - Transaction index within the block (0-based)
   * @param {{ signal?: AbortSignal, onValue?: (value: Txid) => void, cache?: boolean, memCache?: boolean }} [options]
   * @returns {Promise<Txid>}
   */
  async getBlockTxid(hash, index, { signal, onValue, cache, memCache } = {}) {
    const path = `/api/block/${hash}/txid/${index}`;
    return this.getText(path, { signal, onValue, cache, memCache });
  }

  /**
   * Block transaction IDs
   *
   * Retrieve all transaction IDs in a block. Returns an array of txids in block order.
   *
   * *[Mempool.space docs](https://mempool.space/docs/api/rest#get-block-transaction-ids)*
   *
   * Endpoint: `GET /api/block/{hash}/txids`
   *
   * @param {BlockHash} hash
   * @param {{ signal?: AbortSignal, onValue?: (value: Txid[]) => void, cache?: boolean, memCache?: boolean }} [options]
   * @returns {Promise<Txid[]>}
   */
  async getBlockTxids(hash, { signal, onValue, cache, memCache } = {}) {
    const path = `/api/block/${hash}/txids`;
    return this.getJson(path, { signal, onValue, cache, memCache });
  }

  /**
   * Block transactions
   *
   * Retrieve transactions in a block by block hash. Returns up to 25 transactions starting from index 0.
   *
   * *[Mempool.space docs](https://mempool.space/docs/api/rest#get-block-transactions)*
   *
   * Endpoint: `GET /api/block/{hash}/txs`
   *
   * @param {BlockHash} hash
   * @param {{ signal?: AbortSignal, onValue?: (value: Transaction[]) => void, cache?: boolean, memCache?: boolean }} [options]
   * @returns {Promise<Transaction[]>}
   */
  async getBlockTxs(hash, { signal, onValue, cache, memCache } = {}) {
    const path = `/api/block/${hash}/txs`;
    return this.getJson(path, { signal, onValue, cache, memCache });
  }

  /**
   * Block transactions (paginated)
   *
   * Retrieve transactions in a block by block hash, starting from the specified index. Returns up to 25 transactions at a time.
   *
   * *[Mempool.space docs](https://mempool.space/docs/api/rest#get-block-transactions)*
   *
   * Endpoint: `GET /api/block/{hash}/txs/{start_index}`
   *
   * @param {BlockHash} hash - Bitcoin block hash
   * @param {BlockTxIndex} start_index - Starting transaction index within the block (0-based)
   * @param {{ signal?: AbortSignal, onValue?: (value: Transaction[]) => void, cache?: boolean, memCache?: boolean }} [options]
   * @returns {Promise<Transaction[]>}
   */
  async getBlockTxsFromIndex(hash, start_index, { signal, onValue, cache, memCache } = {}) {
    const path = `/api/block/${hash}/txs/${start_index}`;
    return this.getJson(path, { signal, onValue, cache, memCache });
  }

  /**
   * Recent blocks
   *
   * Retrieve the last 10 blocks. Returns block metadata for each block.
   *
   * *[Mempool.space docs](https://mempool.space/docs/api/rest#get-blocks)*
   *
   * Endpoint: `GET /api/blocks`
   * @param {{ signal?: AbortSignal, onValue?: (value: BlockInfo[]) => void, cache?: boolean, memCache?: boolean }} [options]
   * @returns {Promise<BlockInfo[]>}
   */
  async getBlocks({ signal, onValue, cache, memCache } = {}) {
    const path = `/api/blocks`;
    return this.getJson(path, { signal, onValue, cache, memCache });
  }

  /**
   * Blocks from height
   *
   * Retrieve up to 10 blocks going backwards from the given height. For example, height=100 returns blocks 100, 99, 98, ..., 91. Height=0 returns only block 0.
   *
   * *[Mempool.space docs](https://mempool.space/docs/api/rest#get-blocks)*
   *
   * Endpoint: `GET /api/blocks/{height}`
   *
   * @param {Height} height
   * @param {{ signal?: AbortSignal, onValue?: (value: BlockInfo[]) => void, cache?: boolean, memCache?: boolean }} [options]
   * @returns {Promise<BlockInfo[]>}
   */
  async getBlocksFromHeight(height, { signal, onValue, cache, memCache } = {}) {
    const path = `/api/blocks/${height}`;
    return this.getJson(path, { signal, onValue, cache, memCache });
  }

  /**
   * Recent blocks with extras
   *
   * Retrieve the last 15 blocks with extended data including pool identification and fee statistics.
   *
   * *[Mempool.space docs](https://mempool.space/docs/api/rest#get-blocks-v1)*
   *
   * Endpoint: `GET /api/v1/blocks`
   * @param {{ signal?: AbortSignal, onValue?: (value: BlockInfoV1[]) => void, cache?: boolean, memCache?: boolean }} [options]
   * @returns {Promise<BlockInfoV1[]>}
   */
  async getBlocksV1({ signal, onValue, cache, memCache } = {}) {
    const path = `/api/v1/blocks`;
    return this.getJson(path, { signal, onValue, cache, memCache });
  }

  /**
   * Blocks from height with extras
   *
   * Retrieve up to 15 blocks with extended data going backwards from the given height.
   *
   * *[Mempool.space docs](https://mempool.space/docs/api/rest#get-blocks-v1)*
   *
   * Endpoint: `GET /api/v1/blocks/{height}`
   *
   * @param {Height} height
   * @param {{ signal?: AbortSignal, onValue?: (value: BlockInfoV1[]) => void, cache?: boolean, memCache?: boolean }} [options]
   * @returns {Promise<BlockInfoV1[]>}
   */
  async getBlocksV1FromHeight(height, { signal, onValue, cache, memCache } = {}) {
    const path = `/api/v1/blocks/${height}`;
    return this.getJson(path, { signal, onValue, cache, memCache });
  }

  /**
   * List all mining pools
   *
   * Get list of all known mining pools with their identifiers.
   *
   * *[Mempool.space docs](https://mempool.space/docs/api/rest#get-mining-pools)*
   *
   * Endpoint: `GET /api/v1/mining/pools`
   * @param {{ signal?: AbortSignal, onValue?: (value: PoolInfo[]) => void, cache?: boolean, memCache?: boolean }} [options]
   * @returns {Promise<PoolInfo[]>}
   */
  async getPools({ signal, onValue, cache, memCache } = {}) {
    const path = `/api/v1/mining/pools`;
    return this.getJson(path, { signal, onValue, cache, memCache });
  }

  /**
   * Mining pool statistics
   *
   * Get mining pool statistics for a time period. Valid periods: `24h`, `3d`, `1w`, `1m`, `3m`, `6m`, `1y`, `2y`, `3y`.
   *
   * *[Mempool.space docs](https://mempool.space/docs/api/rest#get-mining-pools)*
   *
   * Endpoint: `GET /api/v1/mining/pools/{time_period}`
   *
   * @param {TimePeriod} time_period
   * @param {{ signal?: AbortSignal, onValue?: (value: PoolsSummary) => void, cache?: boolean, memCache?: boolean }} [options]
   * @returns {Promise<PoolsSummary>}
   */
  async getPoolStats(time_period, { signal, onValue, cache, memCache } = {}) {
    const path = `/api/v1/mining/pools/${time_period}`;
    return this.getJson(path, { signal, onValue, cache, memCache });
  }

  /**
   * Mining pool details
   *
   * Get detailed information about a specific mining pool including block counts and shares for different time periods.
   *
   * *[Mempool.space docs](https://mempool.space/docs/api/rest#get-mining-pool)*
   *
   * Endpoint: `GET /api/v1/mining/pool/{slug}`
   *
   * @param {PoolSlug} slug
   * @param {{ signal?: AbortSignal, onValue?: (value: PoolDetail) => void, cache?: boolean, memCache?: boolean }} [options]
   * @returns {Promise<PoolDetail>}
   */
  async getPool(slug, { signal, onValue, cache, memCache } = {}) {
    const path = `/api/v1/mining/pool/${slug}`;
    return this.getJson(path, { signal, onValue, cache, memCache });
  }

  /**
   * All pools hashrate (all time)
   *
   * Get hashrate data for all mining pools.
   *
   * *[Mempool.space docs](https://mempool.space/docs/api/rest#get-mining-pool-hashrates)*
   *
   * Endpoint: `GET /api/v1/mining/hashrate/pools`
   * @param {{ signal?: AbortSignal, onValue?: (value: PoolHashrateEntry[]) => void, cache?: boolean, memCache?: boolean }} [options]
   * @returns {Promise<PoolHashrateEntry[]>}
   */
  async getPoolsHashrate({ signal, onValue, cache, memCache } = {}) {
    const path = `/api/v1/mining/hashrate/pools`;
    return this.getJson(path, { signal, onValue, cache, memCache });
  }

  /**
   * All pools hashrate
   *
   * Get hashrate data for all mining pools for a time period. Valid periods: `1m`, `3m`, `6m`, `1y`, `2y`, `3y`.
   *
   * *[Mempool.space docs](https://mempool.space/docs/api/rest#get-mining-pool-hashrates)*
   *
   * Endpoint: `GET /api/v1/mining/hashrate/pools/{time_period}`
   *
   * @param {TimePeriod} time_period
   * @param {{ signal?: AbortSignal, onValue?: (value: PoolHashrateEntry[]) => void, cache?: boolean, memCache?: boolean }} [options]
   * @returns {Promise<PoolHashrateEntry[]>}
   */
  async getPoolsHashrateByPeriod(time_period, { signal, onValue, cache, memCache } = {}) {
    const path = `/api/v1/mining/hashrate/pools/${time_period}`;
    return this.getJson(path, { signal, onValue, cache, memCache });
  }

  /**
   * Mining pool hashrate
   *
   * Get hashrate history for a specific mining pool.
   *
   * *[Mempool.space docs](https://mempool.space/docs/api/rest#get-mining-pool-hashrate)*
   *
   * Endpoint: `GET /api/v1/mining/pool/{slug}/hashrate`
   *
   * @param {PoolSlug} slug
   * @param {{ signal?: AbortSignal, onValue?: (value: PoolHashrateEntry[]) => void, cache?: boolean, memCache?: boolean }} [options]
   * @returns {Promise<PoolHashrateEntry[]>}
   */
  async getPoolHashrate(slug, { signal, onValue, cache, memCache } = {}) {
    const path = `/api/v1/mining/pool/${slug}/hashrate`;
    return this.getJson(path, { signal, onValue, cache, memCache });
  }

  /**
   * Mining pool blocks
   *
   * Get up to 100 recent blocks mined by a specific pool.
   *
   * *[Mempool.space docs](https://mempool.space/docs/api/rest#get-mining-pool-blocks)*
   *
   * Endpoint: `GET /api/v1/mining/pool/{slug}/blocks`
   *
   * @param {PoolSlug} slug
   * @param {{ signal?: AbortSignal, onValue?: (value: BlockInfoV1[]) => void, cache?: boolean, memCache?: boolean }} [options]
   * @returns {Promise<BlockInfoV1[]>}
   */
  async getPoolBlocks(slug, { signal, onValue, cache, memCache } = {}) {
    const path = `/api/v1/mining/pool/${slug}/blocks`;
    return this.getJson(path, { signal, onValue, cache, memCache });
  }

  /**
   * Mining pool blocks from height
   *
   * Get up to 100 blocks mined by a specific pool before (and including) the given height.
   *
   * *[Mempool.space docs](https://mempool.space/docs/api/rest#get-mining-pool-blocks)*
   *
   * Endpoint: `GET /api/v1/mining/pool/{slug}/blocks/{height}`
   *
   * @param {PoolSlug} slug
   * @param {Height} height
   * @param {{ signal?: AbortSignal, onValue?: (value: BlockInfoV1[]) => void, cache?: boolean, memCache?: boolean }} [options]
   * @returns {Promise<BlockInfoV1[]>}
   */
  async getPoolBlocksFrom(slug, height, { signal, onValue, cache, memCache } = {}) {
    const path = `/api/v1/mining/pool/${slug}/blocks/${height}`;
    return this.getJson(path, { signal, onValue, cache, memCache });
  }

  /**
   * Network hashrate (all time)
   *
   * Get network hashrate and difficulty data for all time.
   *
   * *[Mempool.space docs](https://mempool.space/docs/api/rest#get-hashrate)*
   *
   * Endpoint: `GET /api/v1/mining/hashrate`
   * @param {{ signal?: AbortSignal, onValue?: (value: HashrateSummary) => void, cache?: boolean, memCache?: boolean }} [options]
   * @returns {Promise<HashrateSummary>}
   */
  async getHashrate({ signal, onValue, cache, memCache } = {}) {
    const path = `/api/v1/mining/hashrate`;
    return this.getJson(path, { signal, onValue, cache, memCache });
  }

  /**
   * Network hashrate
   *
   * Get network hashrate and difficulty data for a time period. Valid periods: `24h`, `3d`, `1w`, `1m`, `3m`, `6m`, `1y`, `2y`, `3y`.
   *
   * *[Mempool.space docs](https://mempool.space/docs/api/rest#get-hashrate)*
   *
   * Endpoint: `GET /api/v1/mining/hashrate/{time_period}`
   *
   * @param {TimePeriod} time_period
   * @param {{ signal?: AbortSignal, onValue?: (value: HashrateSummary) => void, cache?: boolean, memCache?: boolean }} [options]
   * @returns {Promise<HashrateSummary>}
   */
  async getHashrateByPeriod(time_period, { signal, onValue, cache, memCache } = {}) {
    const path = `/api/v1/mining/hashrate/${time_period}`;
    return this.getJson(path, { signal, onValue, cache, memCache });
  }

  /**
   * Difficulty adjustments (all time)
   *
   * Get historical difficulty adjustments including timestamp, block height, difficulty value, and percentage change.
   *
   * *[Mempool.space docs](https://mempool.space/docs/api/rest#get-difficulty-adjustments)*
   *
   * Endpoint: `GET /api/v1/mining/difficulty-adjustments`
   * @param {{ signal?: AbortSignal, onValue?: (value: DifficultyAdjustmentEntry[]) => void, cache?: boolean, memCache?: boolean }} [options]
   * @returns {Promise<DifficultyAdjustmentEntry[]>}
   */
  async getDifficultyAdjustments({ signal, onValue, cache, memCache } = {}) {
    const path = `/api/v1/mining/difficulty-adjustments`;
    return this.getJson(path, { signal, onValue, cache, memCache });
  }

  /**
   * Difficulty adjustments
   *
   * Get historical difficulty adjustments for a time period. Valid periods: `24h`, `3d`, `1w`, `1m`, `3m`, `6m`, `1y`, `2y`, `3y`.
   *
   * *[Mempool.space docs](https://mempool.space/docs/api/rest#get-difficulty-adjustments)*
   *
   * Endpoint: `GET /api/v1/mining/difficulty-adjustments/{time_period}`
   *
   * @param {TimePeriod} time_period
   * @param {{ signal?: AbortSignal, onValue?: (value: DifficultyAdjustmentEntry[]) => void, cache?: boolean, memCache?: boolean }} [options]
   * @returns {Promise<DifficultyAdjustmentEntry[]>}
   */
  async getDifficultyAdjustmentsByPeriod(time_period, { signal, onValue, cache, memCache } = {}) {
    const path = `/api/v1/mining/difficulty-adjustments/${time_period}`;
    return this.getJson(path, { signal, onValue, cache, memCache });
  }

  /**
   * Mining reward statistics
   *
   * Get mining reward statistics for the last N blocks including total rewards, fees, and transaction count.
   *
   * *[Mempool.space docs](https://mempool.space/docs/api/rest#get-reward-stats)*
   *
   * Endpoint: `GET /api/v1/mining/reward-stats/{block_count}`
   *
   * @param {number} block_count - Number of recent blocks to include
   * @param {{ signal?: AbortSignal, onValue?: (value: RewardStats) => void, cache?: boolean, memCache?: boolean }} [options]
   * @returns {Promise<RewardStats>}
   */
  async getRewardStats(block_count, { signal, onValue, cache, memCache } = {}) {
    const path = `/api/v1/mining/reward-stats/${block_count}`;
    return this.getJson(path, { signal, onValue, cache, memCache });
  }

  /**
   * Block fees
   *
   * Get average total fees per block for a time period. Valid periods: `24h`, `3d`, `1w`, `1m`, `3m`, `6m`, `1y`, `2y`, `3y`.
   *
   * *[Mempool.space docs](https://mempool.space/docs/api/rest#get-block-fees)*
   *
   * Endpoint: `GET /api/v1/mining/blocks/fees/{time_period}`
   *
   * @param {TimePeriod} time_period
   * @param {{ signal?: AbortSignal, onValue?: (value: BlockFeesEntry[]) => void, cache?: boolean, memCache?: boolean }} [options]
   * @returns {Promise<BlockFeesEntry[]>}
   */
  async getBlockFees(time_period, { signal, onValue, cache, memCache } = {}) {
    const path = `/api/v1/mining/blocks/fees/${time_period}`;
    return this.getJson(path, { signal, onValue, cache, memCache });
  }

  /**
   * Block rewards
   *
   * Get average coinbase reward (subsidy + fees) per block for a time period. Valid periods: `24h`, `3d`, `1w`, `1m`, `3m`, `6m`, `1y`, `2y`, `3y`.
   *
   * *[Mempool.space docs](https://mempool.space/docs/api/rest#get-block-rewards)*
   *
   * Endpoint: `GET /api/v1/mining/blocks/rewards/{time_period}`
   *
   * @param {TimePeriod} time_period
   * @param {{ signal?: AbortSignal, onValue?: (value: BlockRewardsEntry[]) => void, cache?: boolean, memCache?: boolean }} [options]
   * @returns {Promise<BlockRewardsEntry[]>}
   */
  async getBlockRewards(time_period, { signal, onValue, cache, memCache } = {}) {
    const path = `/api/v1/mining/blocks/rewards/${time_period}`;
    return this.getJson(path, { signal, onValue, cache, memCache });
  }

  /**
   * Block fee rates
   *
   * Get block fee rate percentiles (min, 10th, 25th, median, 75th, 90th, max) for a time period. Valid periods: `24h`, `3d`, `1w`, `1m`, `3m`, `6m`, `1y`, `2y`, `3y`.
   *
   * *[Mempool.space docs](https://mempool.space/docs/api/rest#get-block-feerates)*
   *
   * Endpoint: `GET /api/v1/mining/blocks/fee-rates/{time_period}`
   *
   * @param {TimePeriod} time_period
   * @param {{ signal?: AbortSignal, onValue?: (value: BlockFeeRatesEntry[]) => void, cache?: boolean, memCache?: boolean }} [options]
   * @returns {Promise<BlockFeeRatesEntry[]>}
   */
  async getBlockFeeRates(time_period, { signal, onValue, cache, memCache } = {}) {
    const path = `/api/v1/mining/blocks/fee-rates/${time_period}`;
    return this.getJson(path, { signal, onValue, cache, memCache });
  }

  /**
   * Block sizes and weights
   *
   * Get average block sizes and weights for a time period. Valid periods: `24h`, `3d`, `1w`, `1m`, `3m`, `6m`, `1y`, `2y`, `3y`.
   *
   * *[Mempool.space docs](https://mempool.space/docs/api/rest#get-sizes-weights)*
   *
   * Endpoint: `GET /api/v1/mining/blocks/sizes-weights/{time_period}`
   *
   * @param {TimePeriod} time_period
   * @param {{ signal?: AbortSignal, onValue?: (value: BlockSizesWeights) => void, cache?: boolean, memCache?: boolean }} [options]
   * @returns {Promise<BlockSizesWeights>}
   */
  async getBlockSizesWeights(time_period, { signal, onValue, cache, memCache } = {}) {
    const path = `/api/v1/mining/blocks/sizes-weights/${time_period}`;
    return this.getJson(path, { signal, onValue, cache, memCache });
  }

  /**
   * Projected mempool blocks
   *
   * Projected blocks for fee estimation. Block 0 reflects Bitcoin Core's actual next-block selection; blocks 1+ are a fee-tier approximation.
   *
   * *[Mempool.space docs](https://mempool.space/docs/api/rest#get-mempool-blocks-fees)*
   *
   * Endpoint: `GET /api/v1/fees/mempool-blocks`
   * @param {{ signal?: AbortSignal, onValue?: (value: MempoolBlock[]) => void, cache?: boolean, memCache?: boolean }} [options]
   * @returns {Promise<MempoolBlock[]>}
   */
  async getMempoolBlocks({ signal, onValue, cache, memCache } = {}) {
    const path = `/api/v1/fees/mempool-blocks`;
    return this.getJson(path, { signal, onValue, cache, memCache });
  }

  /**
   * Recommended fees
   *
   * Recommended fee rates by confirmation target.
   *
   * *[Mempool.space docs](https://mempool.space/docs/api/rest#get-recommended-fees)*
   *
   * Endpoint: `GET /api/v1/fees/recommended`
   * @param {{ signal?: AbortSignal, onValue?: (value: RecommendedFees) => void, cache?: boolean, memCache?: boolean }} [options]
   * @returns {Promise<RecommendedFees>}
   */
  async getRecommendedFees({ signal, onValue, cache, memCache } = {}) {
    const path = `/api/v1/fees/recommended`;
    return this.getJson(path, { signal, onValue, cache, memCache });
  }

  /**
   * Recommended fee rates (precise)
   *
   * Recommended fee rates by confirmation target, with up to three decimal places and support for sub-sat/vB rates.
   *
   * *[Mempool.space docs](https://mempool.space/docs/api/rest#get-recommended-fees-precise)*
   *
   * Endpoint: `GET /api/v1/fees/precise`
   * @param {{ signal?: AbortSignal, onValue?: (value: RecommendedFees) => void, cache?: boolean, memCache?: boolean }} [options]
   * @returns {Promise<RecommendedFees>}
   */
  async getPreciseFees({ signal, onValue, cache, memCache } = {}) {
    const path = `/api/v1/fees/precise`;
    return this.getJson(path, { signal, onValue, cache, memCache });
  }

  /**
   * Mempool statistics
   *
   * Get current mempool statistics including transaction count, total vsize, total fees, and fee histogram.
   *
   * *[Mempool.space docs](https://mempool.space/docs/api/rest#get-mempool)*
   *
   * Endpoint: `GET /api/mempool`
   * @param {{ signal?: AbortSignal, onValue?: (value: MempoolInfo) => void, cache?: boolean, memCache?: boolean }} [options]
   * @returns {Promise<MempoolInfo>}
   */
  async getMempool({ signal, onValue, cache, memCache } = {}) {
    const path = `/api/mempool`;
    return this.getJson(path, { signal, onValue, cache, memCache });
  }

  /**
   * Mempool content hash
   *
   * Returns an opaque content token for the published projected next block, including statistics and transaction bodies. This is not the HTTP ETag. An unchanged token means unchanged content, not necessarily a stalled sync loop.
   *
   * Endpoint: `GET /api/mempool/hash`
   * @param {{ signal?: AbortSignal, onValue?: (value: NextBlockHash) => void, cache?: boolean, memCache?: boolean }} [options]
   * @returns {Promise<NextBlockHash>}
   */
  async getMempoolHash({ signal, onValue, cache, memCache } = {}) {
    const path = `/api/mempool/hash`;
    return this.getJson(path, { signal, onValue, cache, memCache });
  }

  /**
   * Mempool transaction IDs
   *
   * Get all transaction IDs currently in the mempool.
   *
   * *[Mempool.space docs](https://mempool.space/docs/api/rest#get-mempool-transaction-ids)*
   *
   * Endpoint: `GET /api/mempool/txids`
   * @param {{ signal?: AbortSignal, onValue?: (value: Txid[]) => void, cache?: boolean, memCache?: boolean }} [options]
   * @returns {Promise<Txid[]>}
   */
  async getMempoolTxids({ signal, onValue, cache, memCache } = {}) {
    const path = `/api/mempool/txids`;
    return this.getJson(path, { signal, onValue, cache, memCache });
  }

  /**
   * Recent mempool transactions
   *
   * Get the last 10 transactions to enter the mempool.
   *
   * *[Mempool.space docs](https://mempool.space/docs/api/rest#get-mempool-recent)*
   *
   * Endpoint: `GET /api/mempool/recent`
   * @param {{ signal?: AbortSignal, onValue?: (value: MempoolRecentTx[]) => void, cache?: boolean, memCache?: boolean }} [options]
   * @returns {Promise<MempoolRecentTx[]>}
   */
  async getMempoolRecent({ signal, onValue, cache, memCache } = {}) {
    const path = `/api/mempool/recent`;
    return this.getJson(path, { signal, onValue, cache, memCache });
  }

  /**
   * Recent RBF replacements
   *
   * Returns up to 25 most-recent RBF replacement trees across the whole mempool. Each entry has the same shape as `tx_rbf().replacements`.
   *
   * *[Mempool.space docs](https://mempool.space/docs/api/rest#get-replacements)*
   *
   * Endpoint: `GET /api/v1/replacements`
   * @param {{ signal?: AbortSignal, onValue?: (value: ReplacementNode[]) => void, cache?: boolean, memCache?: boolean }} [options]
   * @returns {Promise<ReplacementNode[]>}
   */
  async getReplacements({ signal, onValue, cache, memCache } = {}) {
    const path = `/api/v1/replacements`;
    return this.getJson(path, { signal, onValue, cache, memCache });
  }

  /**
   * Recent full-RBF replacements
   *
   * Same response shape as `GET /api/v1/replacements`, but limited to trees where at least one predecessor was non-signaling (full-RBF).
   *
   * *[Mempool.space docs](https://mempool.space/docs/api/rest#get-fullrbf-replacements)*
   *
   * Endpoint: `GET /api/v1/fullrbf/replacements`
   * @param {{ signal?: AbortSignal, onValue?: (value: ReplacementNode[]) => void, cache?: boolean, memCache?: boolean }} [options]
   * @returns {Promise<ReplacementNode[]>}
   */
  async getFullrbfReplacements({ signal, onValue, cache, memCache } = {}) {
    const path = `/api/v1/fullrbf/replacements`;
    return this.getJson(path, { signal, onValue, cache, memCache });
  }

  /**
   * Projected next block template
   *
   * Bitcoin Core's `getblocktemplate` selection: full transaction bodies in GBT order with aggregate stats. The returned `hash` is an opaque content token; pass it to `GET /api/v1/mempool/block-template/diff/{hash}` to fetch deltas instead of refetching the whole template.
   *
   * Endpoint: `GET /api/v1/mempool/block-template`
   * @param {{ signal?: AbortSignal, onValue?: (value: BlockTemplate) => void, cache?: boolean, memCache?: boolean }} [options]
   * @returns {Promise<BlockTemplate>}
   */
  async getBlockTemplate({ signal, onValue, cache, memCache } = {}) {
    const path = `/api/v1/mempool/block-template`;
    return this.getJson(path, { signal, onValue, cache, memCache });
  }

  /**
   * Block template diff since hash
   *
   * Delta of the projected next block since `<hash>`. `order` is the full new template in order: each entry is either a number (index into the prior template the client cached at `<hash>`) or a transaction object (new body to insert at this position). Walk `order` once to rebuild; `removed` is a convenience list of txids that left so clients can evict cached bodies. After applying, use the response `hash` as `<hash>` on the next call to keep iterating. Returns `404` when `<hash>` has aged out of server history; clients should fall back to `GET /api/v1/mempool/block-template`.
   *
   * Endpoint: `GET /api/v1/mempool/block-template/diff/{hash}`
   *
   * @param {NextBlockHash} hash
   * @param {{ signal?: AbortSignal, onValue?: (value: BlockTemplateDiff) => void, cache?: boolean, memCache?: boolean }} [options]
   * @returns {Promise<BlockTemplateDiff>}
   */
  async getBlockTemplateDiff(hash, { signal, onValue, cache, memCache } = {}) {
    const path = `/api/v1/mempool/block-template/diff/${hash}`;
    return this.getJson(path, { signal, onValue, cache, memCache });
  }

  /**
   * Live BTC/USD price
   *
   * Returns the current BTC/USD price in dollars, derived from on-chain round-dollar output patterns in the last 12 blocks plus mempool.
   *
   * Endpoint: `GET /api/mempool/price`
   * @param {{ signal?: AbortSignal, onValue?: (value: Dollars) => void, cache?: boolean, memCache?: boolean }} [options]
   * @returns {Promise<Dollars>}
   */
  async getLivePrice({ signal, onValue, cache, memCache } = {}) {
    const path = `/api/mempool/price`;
    return this.getJson(path, { signal, onValue, cache, memCache });
  }

  /**
   * Txid by index
   *
   * Retrieve the transaction ID (txid) at a given global transaction index. Returns the txid as plain text.
   *
   * Endpoint: `GET /api/tx-index/{index}`
   *
   * @param {TxIndex} index
   * @param {{ signal?: AbortSignal, onValue?: (value: Txid) => void, cache?: boolean, memCache?: boolean }} [options]
   * @returns {Promise<Txid>}
   */
  async getTxByIndex(index, { signal, onValue, cache, memCache } = {}) {
    const path = `/api/tx-index/${index}`;
    return this.getText(path, { signal, onValue, cache, memCache });
  }

  /**
   * CPFP info
   *
   * Returns ancestors and descendants for a CPFP (Child Pays For Parent) transaction, including the effective fee rate of the package.
   *
   * *[Mempool.space docs](https://mempool.space/docs/api/rest#get-children-pay-for-parent)*
   *
   * Endpoint: `GET /api/v1/cpfp/{txid}`
   *
   * @param {Txid} txid
   * @param {{ signal?: AbortSignal, onValue?: (value: CpfpInfo) => void, cache?: boolean, memCache?: boolean }} [options]
   * @returns {Promise<CpfpInfo>}
   */
  async getCpfp(txid, { signal, onValue, cache, memCache } = {}) {
    const path = `/api/v1/cpfp/${txid}`;
    return this.getJson(path, { signal, onValue, cache, memCache });
  }

  /**
   * RBF replacement history
   *
   * Returns the RBF replacement tree for a transaction, if any. Both `replacements` and `replaces` are null when the tx has no known RBF history within the mempool monitor's retention window.
   *
   * *[Mempool.space docs](https://mempool.space/docs/api/rest#get-transaction-rbf-history)*
   *
   * Endpoint: `GET /api/v1/tx/{txid}/rbf`
   *
   * @param {Txid} txid
   * @param {{ signal?: AbortSignal, onValue?: (value: RbfResponse) => void, cache?: boolean, memCache?: boolean }} [options]
   * @returns {Promise<RbfResponse>}
   */
  async getTxRbf(txid, { signal, onValue, cache, memCache } = {}) {
    const path = `/api/v1/tx/${txid}/rbf`;
    return this.getJson(path, { signal, onValue, cache, memCache });
  }

  /**
   * Transaction information
   *
   * Retrieve complete transaction data by transaction ID (txid). Returns inputs, outputs, fee, size, and confirmation status.
   *
   * *[Mempool.space docs](https://mempool.space/docs/api/rest#get-transaction)*
   *
   * Endpoint: `GET /api/tx/{txid}`
   *
   * @param {Txid} txid
   * @param {{ signal?: AbortSignal, onValue?: (value: Transaction) => void, cache?: boolean, memCache?: boolean }} [options]
   * @returns {Promise<Transaction>}
   */
  async getTx(txid, { signal, onValue, cache, memCache } = {}) {
    const path = `/api/tx/${txid}`;
    return this.getJson(path, { signal, onValue, cache, memCache });
  }

  /**
   * Transaction hex
   *
   * Retrieve the raw transaction as a hex-encoded string. Returns the serialized transaction in hexadecimal format.
   *
   * *[Mempool.space docs](https://mempool.space/docs/api/rest#get-transaction-hex)*
   *
   * Endpoint: `GET /api/tx/{txid}/hex`
   *
   * @param {Txid} txid
   * @param {{ signal?: AbortSignal, onValue?: (value: Hex) => void, cache?: boolean, memCache?: boolean }} [options]
   * @returns {Promise<Hex>}
   */
  async getTxHex(txid, { signal, onValue, cache, memCache } = {}) {
    const path = `/api/tx/${txid}/hex`;
    return this.getText(path, { signal, onValue, cache, memCache });
  }

  /**
   * Transaction merkleblock proof
   *
   * Get the merkleblock proof for a transaction (BIP37 format, hex encoded).
   *
   * *[Mempool.space docs](https://mempool.space/docs/api/rest#get-transaction-merkleblock-proof)*
   *
   * Endpoint: `GET /api/tx/{txid}/merkleblock-proof`
   *
   * @param {Txid} txid
   * @param {{ signal?: AbortSignal, onValue?: (value: Hex) => void, cache?: boolean, memCache?: boolean }} [options]
   * @returns {Promise<Hex>}
   */
  async getTxMerkleblockProof(txid, { signal, onValue, cache, memCache } = {}) {
    const path = `/api/tx/${txid}/merkleblock-proof`;
    return this.getText(path, { signal, onValue, cache, memCache });
  }

  /**
   * Transaction merkle proof
   *
   * Get the merkle inclusion proof for a transaction.
   *
   * *[Mempool.space docs](https://mempool.space/docs/api/rest#get-transaction-merkle-proof)*
   *
   * Endpoint: `GET /api/tx/{txid}/merkle-proof`
   *
   * @param {Txid} txid
   * @param {{ signal?: AbortSignal, onValue?: (value: MerkleProof) => void, cache?: boolean, memCache?: boolean }} [options]
   * @returns {Promise<MerkleProof>}
   */
  async getTxMerkleProof(txid, { signal, onValue, cache, memCache } = {}) {
    const path = `/api/tx/${txid}/merkle-proof`;
    return this.getJson(path, { signal, onValue, cache, memCache });
  }

  /**
   * Output spend status
   *
   * Get the spending status of a transaction output. Returns whether the output has been spent and, if so, the spending transaction details.
   *
   * *[Mempool.space docs](https://mempool.space/docs/api/rest#get-transaction-outspend)*
   *
   * Endpoint: `GET /api/tx/{txid}/outspend/{vout}`
   *
   * @param {Txid} txid - Transaction ID
   * @param {Vout} vout - Output index
   * @param {{ signal?: AbortSignal, onValue?: (value: TxOutspend) => void, cache?: boolean, memCache?: boolean }} [options]
   * @returns {Promise<TxOutspend>}
   */
  async getTxOutspend(txid, vout, { signal, onValue, cache, memCache } = {}) {
    const path = `/api/tx/${txid}/outspend/${vout}`;
    return this.getJson(path, { signal, onValue, cache, memCache });
  }

  /**
   * All output spend statuses
   *
   * Get the spending status of all outputs in a transaction. Returns an array with the spend status for each output.
   *
   * *[Mempool.space docs](https://mempool.space/docs/api/rest#get-transaction-outspends)*
   *
   * Endpoint: `GET /api/tx/{txid}/outspends`
   *
   * @param {Txid} txid
   * @param {{ signal?: AbortSignal, onValue?: (value: TxOutspend[]) => void, cache?: boolean, memCache?: boolean }} [options]
   * @returns {Promise<TxOutspend[]>}
   */
  async getTxOutspends(txid, { signal, onValue, cache, memCache } = {}) {
    const path = `/api/tx/${txid}/outspends`;
    return this.getJson(path, { signal, onValue, cache, memCache });
  }

  /**
   * Transaction raw
   *
   * Returns a transaction as binary data.
   *
   * *[Mempool.space docs](https://mempool.space/docs/api/rest#get-transaction-raw)*
   *
   * Endpoint: `GET /api/tx/{txid}/raw`
   *
   * @param {Txid} txid
   * @param {{ signal?: AbortSignal, onValue?: (value: Uint8Array) => void, cache?: boolean, memCache?: boolean }} [options]
   * @returns {Promise<Uint8Array>}
   */
  async getTxRaw(txid, { signal, onValue, cache, memCache } = {}) {
    const path = `/api/tx/${txid}/raw`;
    return this.getBytes(path, { signal, onValue, cache, memCache });
  }

  /**
   * Transaction status
   *
   * Retrieve the confirmation status of a transaction. Returns whether the transaction is confirmed and, if so, the block height, hash, and timestamp.
   *
   * *[Mempool.space docs](https://mempool.space/docs/api/rest#get-transaction-status)*
   *
   * Endpoint: `GET /api/tx/{txid}/status`
   *
   * @param {Txid} txid
   * @param {{ signal?: AbortSignal, onValue?: (value: TxStatus) => void, cache?: boolean, memCache?: boolean }} [options]
   * @returns {Promise<TxStatus>}
   */
  async getTxStatus(txid, { signal, onValue, cache, memCache } = {}) {
    const path = `/api/tx/${txid}/status`;
    return this.getJson(path, { signal, onValue, cache, memCache });
  }

  /**
   * Transaction first-seen times
   *
   * Returns timestamps when transactions were first seen in the mempool. Returns 0 for mined or unknown transactions.
   *
   * *[Mempool.space docs](https://mempool.space/docs/api/rest#get-transaction-times)*
   *
   * Endpoint: `GET /api/v1/transaction-times`
   *
   * @param {Txid[]} txId - Transaction IDs to look up (max 250 per request).
   * @param {{ signal?: AbortSignal, onValue?: (value: number[]) => void, cache?: boolean, memCache?: boolean }} [options]
   * @returns {Promise<number[]>}
   */
  async getTransactionTimes(txId, { signal, onValue, cache, memCache } = {}) {
    const params = new URLSearchParams();
    for (const _v of txId) params.append('txId[]', String(_v));
    const query = params.toString();
    const path = `/api/v1/transaction-times${query ? '?' + query : ''}`;
    return this.getJson(path, { signal, onValue, cache, memCache });
  }

  /**
   * Broadcast transaction
   *
   * Submit a raw transaction as hexadecimal text (at most 8,000,000 request bytes, including whitespace). Returns its txid as plain text. No responses are cached. Cancellation or a transport error after dispatch may leave the submission outcome unknown; do not automatically retry.
   *
   * *[Mempool.space docs](https://mempool.space/docs/api/rest#post-transaction)*
   *
   * Endpoint: `POST /api/tx`
   *
   * @param {string} body - Request body
   * @param {{ signal?: AbortSignal }} [options]
   * @returns {Promise<Txid>}
   */
  async postTx(body, { signal } = {}) {
    const path = `/api/tx`;
    return this.postText(path, body, { signal });
  }

  /**
   * Live BTC/USD price
   *
   * Current BTC/USD price in dollars. Same value as `GET /api/mempool/price`. Confirmed per-height history is available at `GET /api/series/price/height`.
   *
   * Endpoint: `GET /api/oracle/price`
   * @param {{ signal?: AbortSignal, onValue?: (value: Dollars) => void, cache?: boolean, memCache?: boolean }} [options]
   * @returns {Promise<Dollars>}
   */
  async getOraclePrice({ signal, onValue, cache, memCache } = {}) {
    const path = `/api/oracle/price`;
    return this.getJson(path, { signal, onValue, cache, memCache });
  }

  /**
   * Live payment output histogram
   *
   * Live smoothed histogram of oracle-eligible payment outputs, binned by output value on the oracle log scale. It combines the committed oracle window with the complete mempool's eligible outputs from a matching chain publication. A flat array of log-scale bins.
   *
   * Endpoint: `GET /api/oracle/histogram/payments/live`
   * @param {{ signal?: AbortSignal, onValue?: (value: number[]) => void, cache?: boolean, memCache?: boolean }} [options]
   * @returns {Promise<number[]>}
   */
  async getOracleHistogramPaymentsLive({ signal, onValue, cache, memCache } = {}) {
    const path = `/api/oracle/histogram/payments/live`;
    return this.getJson(path, { signal, onValue, cache, memCache });
  }

  /**
   * Payment output histogram at height or day
   *
   * Smoothed histogram of oracle-eligible payment outputs for a confirmed point. A block height (`840000`) gives that block's oracle payment histogram; a calendar date (`YYYY-MM-DD`) gives the average of that day's per-block payment histograms. A flat array of log-scale bins.
   *
   * Endpoint: `GET /api/oracle/histogram/payments/{point}`
   *
   * @param {string} point - Confirmed block height as decimal digits (`840000`) or calendar date in
`YYYY-MM-DD` format.
   * @param {{ signal?: AbortSignal, onValue?: (value: number[]) => void, cache?: boolean, memCache?: boolean }} [options]
   * @returns {Promise<number[]>}
   */
  async getOracleHistogramPayments(point, { signal, onValue, cache, memCache } = {}) {
    const path = `/api/oracle/histogram/payments/${point}`;
    return this.getJson(path, { signal, onValue, cache, memCache });
  }

  /**
   * Live output value histogram
   *
   * Live unfiltered output value histogram for the complete published mempool. Every live output is binned by value on the oracle log scale; no oracle payment filters are applied. A flat array of log-scale bins, all zero when no mempool is configured.
   *
   * Endpoint: `GET /api/oracle/histogram/outputs/live`
   * @param {{ signal?: AbortSignal, onValue?: (value: number[]) => void, cache?: boolean, memCache?: boolean }} [options]
   * @returns {Promise<number[]>}
   */
  async getOracleHistogramOutputsLive({ signal, onValue, cache, memCache } = {}) {
    const path = `/api/oracle/histogram/outputs/live`;
    return this.getJson(path, { signal, onValue, cache, memCache });
  }

  /**
   * Output value histogram at height or day
   *
   * Unfiltered output value histogram for a confirmed point. A block height (`840000`) gives every output in that block, coinbase included, binned by value on the oracle log scale; a calendar date (`YYYY-MM-DD`) sums every block that day. A flat array of log-scale bins.
   *
   * Endpoint: `GET /api/oracle/histogram/outputs/{point}`
   *
   * @param {string} point - Confirmed block height as decimal digits (`840000`) or calendar date in
`YYYY-MM-DD` format.
   * @param {{ signal?: AbortSignal, onValue?: (value: number[]) => void, cache?: boolean, memCache?: boolean }} [options]
   * @returns {Promise<number[]>}
   */
  async getOracleHistogramOutputs(point, { signal, onValue, cache, memCache } = {}) {
    const path = `/api/oracle/histogram/outputs/${point}`;
    return this.getJson(path, { signal, onValue, cache, memCache });
  }

  /**
   * OpenAPI specification
   *
   * Full OpenAPI 3.1 specification for this API.
   *
   * Endpoint: `GET /openapi.json`
   * @param {{ signal?: AbortSignal, onValue?: (value: *) => void, cache?: boolean, memCache?: boolean }} [options]
   * @returns {Promise<*>}
   */
  async getOpenapi({ signal, onValue, cache, memCache } = {}) {
    const path = `/openapi.json`;
    return this.getText(path, { signal, onValue, cache, memCache });
  }

  /**
   * Compact OpenAPI specification
   *
   * Compact OpenAPI specification optimized for LLM consumption. Removes redundant fields while preserving essential API information. The full specification is available at `GET /openapi.json`.
   *
   * Endpoint: `GET /api.json`
   * @param {{ signal?: AbortSignal, onValue?: (value: *) => void, cache?: boolean, memCache?: boolean }} [options]
   * @returns {Promise<*>}
   */
  async getApi({ signal, onValue, cache, memCache } = {}) {
    const path = `/api.json`;
    return this.getJson(path, { signal, onValue, cache, memCache });
  }

}

export { BitviewClient, BitviewError, addressPayloadHashPrefix };
