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
 * Yes or no.
 *
 * @typedef {boolean} Boolean
 */
/**
 * A size in bytes.
 *
 * @typedef {number} Bytes
 */
/**
 * A size in bytes that fits 32 bits (under 4 GiB), such as a block or transaction size.
 *
 * @typedef {number} Bytes32
 */
/**
 * A mean size in bytes.
 *
 * @typedef {number} BytesFract
 */
/**
 * Investor phase from the Capital Sentiment model.
 *
 * Codes are explicit because phase values are persisted. Code `0` represents
 * unavailable model inputs and is therefore not a phase.
 *
 * @typedef {("raging_bull"|"bull"|"cautious_bull"|"hopeful_bull"|"early_bull"|"weak_bull"|"limbo"|"deep_bear"|"bear"|"early_bear")} CapitalSentimentPhase
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
 * Bitcoin multiplied by the blocks it was held.
 *
 * @typedef {number} CoinBlocks
 */
/**
 * Bitcoin multiplied by the days it was held.
 *
 * @typedef {number} CoinDays
 */
/**
 * Bitcoin multiplied by the years it was held.
 *
 * @typedef {number} CoinYears
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
 * A number of things.
 *
 * @typedef {number} Count
 */
/**
 * A number of things that fits 16 bits (at most 65,535), such as transactions per block.
 *
 * @typedef {number} Count16
 */
/**
 * A number of things that fits 32 bits, such as outputs per block.
 *
 * @typedef {number} Count32
 */
/**
 * A mean number of things.
 *
 * @typedef {number} CountFract
 */
/**
 * A change in a number of things.
 *
 * @typedef {number} CountSigned
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
/**
 * Index of a day.
 *
 * @typedef {number} Day1
 */
/**
 * Index of a 3-day period.
 *
 * @typedef {number} Day3
 */
/**
 * A duration in days.
 *
 * @typedef {number} Days
 */
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
 * Proof-of-work difficulty.
 *
 * @typedef {number} Difficulty
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
/**
 * Index of an output with an empty script.
 *
 * @typedef {TypeIndex} EmptyOutputIndex
 */
/**
 * Index of a difficulty epoch (2,016 blocks).
 *
 * @typedef {number} Epoch
 */
/**
 * The JSON body of every API error (`application/problem+json`).
 *
 * @typedef {Object} ErrorBody
 * @property {ErrorDetail} error
 */
/**
 * Machine-readable error code.
 *
 * @typedef {("not_found"|"invalid_addr"|"invalid_network"|"unsupported_type"|"no_series"|"series_unsupported_index"|"weight_exceeded"|"too_many_utxos"|"unknown_addr"|"unknown_txid"|"out_of_range"|"unindexable_date"|"no_data"|"series_not_found"|"mempool_not_available"|"state_updating"|"internal_error"|"bad_request"|"overloaded"|"timeout"|"method_not_allowed")} ErrorCode
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
 * A 32-bit floating-point value without a specific unit.
 *
 * @typedef {number} Float32
 */
/**
 * A 64-bit floating-point value without a specific unit.
 *
 * @typedef {number} Float64
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
/**
 * Index of a halving epoch (210,000 blocks).
 *
 * @typedef {number} Halving
 */
/**
 * Hashes per second.
 *
 * @typedef {number} Hashrate
 */
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
/**
 * Index of an hour.
 *
 * @typedef {number} Hour1
 */
/**
 * Index of a 12-hour period.
 *
 * @typedef {number} Hour12
 */
/**
 * Index of a 4-hour period.
 *
 * @typedef {number} Hour4
 */
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
/**
 * Index of a 10-minute period.
 *
 * @typedef {number} Minute10
 */
/**
 * Index of a 30-minute period.
 *
 * @typedef {number} Minute30
 */
/**
 * Index of a month.
 *
 * @typedef {number} Month1
 */
/**
 * Index of a quarter (3 months).
 *
 * @typedef {number} Month3
 */
/**
 * Index of a half-year (6 months).
 *
 * @typedef {number} Month6
 */
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
 * Open, high, low and close prices in US dollars: [open, high, low, close].
 *
 * @typedef {Dollars[]} OHLCDollars
 */
/**
 * Index of an OP_RETURN output.
 *
 * @typedef {TypeIndex} OpReturnIndex
 */
/**
 * Protocol or format detected in an OP_RETURN output.
 *
 * @typedef {("runes"|"veri_block"|"omni"|"stacks"|"blockstack"|"colu"|"open_assets"|"komodo"|"coin_spark"|"poet"|"docproof"|"open_timestamps"|"factom"|"eternity_wall"|"memo"|"bitproof"|"ascribe"|"stampery"|"epobc"|"bare_hash"|"text"|"empty"|"unknown")} OpReturnKind
 */
/**
 * Optional UNIX timestamp query parameter
 *
 * @typedef {Object} OptionalTimestampParam
 * @property {(Timestamp|null)=} timestamp
 */
/**
 * The output a transaction input spends: its transaction index and output position, written as
 * `{"tx_index": N, "vout": M}` (coinbase inputs: `{"tx_index": 4294967295, "vout": 65535}`).
 *
 * @typedef {Object} OutPoint
 * @property {TxIndex} txIndex
 * @property {Vout} vout
 */
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
/**
 * Index of a P2A (pay-to-anchor) address.
 *
 * @typedef {TypeIndex} P2AAddrIndex
 */
/**
 * The 2-byte witness program of a P2A (pay-to-anchor) output.
 *
 * @typedef {string} P2ABytes
 */
/**
 * Index of a P2MS (bare multisig) output.
 *
 * @typedef {TypeIndex} P2MSOutputIndex
 */
/**
 * Index of a P2PK address with a compressed (33-byte) public key.
 *
 * @typedef {TypeIndex} P2PK33AddrIndex
 */
/**
 * A compressed (33-byte) public key.
 *
 * @typedef {string} P2PK33Bytes
 */
/**
 * Index of a P2PK address with an uncompressed (65-byte) public key.
 *
 * @typedef {TypeIndex} P2PK65AddrIndex
 */
/**
 * An uncompressed (65-byte) public key.
 *
 * @typedef {string} P2PK65Bytes
 */
/**
 * Index of a P2PKH address.
 *
 * @typedef {TypeIndex} P2PKHAddrIndex
 */
/**
 * The 20-byte public key hash of a P2PKH output.
 *
 * @typedef {string} P2PKHBytes
 */
/**
 * Index of a P2SH address.
 *
 * @typedef {TypeIndex} P2SHAddrIndex
 */
/**
 * The 20-byte script hash of a P2SH output.
 *
 * @typedef {string} P2SHBytes
 */
/**
 * Index of a P2TR (taproot) address.
 *
 * @typedef {TypeIndex} P2TRAddrIndex
 */
/**
 * The 32-byte output key of a P2TR output.
 *
 * @typedef {string} P2TRBytes
 */
/**
 * Index of a P2WPKH address.
 *
 * @typedef {TypeIndex} P2WPKHAddrIndex
 */
/**
 * The 20-byte public key hash of a P2WPKH output.
 *
 * @typedef {string} P2WPKHBytes
 */
/**
 * Index of a P2WSH address.
 *
 * @typedef {TypeIndex} P2WSHAddrIndex
 */
/**
 * The 32-byte script hash of a P2WSH output.
 *
 * @typedef {string} P2WSHBytes
 */
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
 * A fraction per day.
 *
 * @typedef {number} PerDay
 */
/**
 * Events per second.
 *
 * @typedef {number} PerSecond
 */
/**
 * A percentage: a ratio times 100.
 *
 * @typedef {number} Percent
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
 * A discrete rank.
 *
 * @typedef {number} Rank
 */
/**
 * A dimensionless ratio: a quotient, share or multiple.
 *
 * @typedef {number} Ratio
 */
/**
 * A dimensionless ratio at double precision.
 *
 * @typedef {number} Ratio64
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
 * Signed satoshis (i64) - for values that can be negative.
 * Used for changes, deltas, profit/loss calculations, etc.
 *
 * @typedef {number} SatsSigned
 */
/**
 * A signed model score.
 *
 * @typedef {number} Score
 */
/**
 * @typedef {Object} SearchQuery
 * @property {SeriesName} q - Search query string
 * @property {Limit=} limit - Maximum number of results
 */
/**
 * A duration in seconds (at most about 136 years).
 *
 * @typedef {number} Seconds
 */
/**
 * A mean duration in seconds.
 *
 * @typedef {number} SecondsFract
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
 * @property {Index[]} nullable - Indexes whose values can be null: a missing value (e.g. a period without blocks) or an
undefined one (e.g. NaN)
 * @property {string} type - Value type (e.g. "Ratio", "Sats", "Cents")
 * @property {?string=} unit - What the value type measures, when documented (e.g. "A duration in days.")
 */
/**
 * Series leaf with JSON Schema for client generation.
 *
 * @typedef {Object} SeriesLeafWithSchema
 * @property {string} name - The series name/identifier.
 * @property {string} kind - The Rust type (e.g., "Sats", "Ratio").
 * @property {Index[]} indexes - Available indexes for this series.
 * @property {Index[]} nullable - Indexes whose values can be null: a missing value (e.g. a period without blocks) or an
undefined one (e.g. NaN).
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
/**
 * BIP-141 signature-operation cost with enough range for cumulative and rolling totals.
 *
 * @typedef {number} SigOps64
 */
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
/**
 * Index of a transaction input.
 *
 * @typedef {number} TxInIndex
 */
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
/**
 * Index of a transaction output.
 *
 * @typedef {number} TxOutIndex
 */
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
/**
 * Index of an output with an unrecognized script.
 *
 * @typedef {TypeIndex} UnknownOutputIndex
 */
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
 * A mean virtual size in vbytes.
 *
 * @typedef {number} VSizeFract
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
/**
 * Index of a week.
 *
 * @typedef {number} Week1
 */
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
 * A mean weight in weight units.
 *
 * @typedef {number} WeightFract
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
/**
 * Index of a year.
 *
 * @typedef {number} Year1
 */
/**
 * Index of a decade.
 *
 * @typedef {number} Year10
 */
/**
 * A duration in years.
 *
 * @typedef {number} Years
 */

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
 * A failed request: the HTTP status and the server's error code and message, when it sent them.
 */
class BitviewError extends Error {
  /**
   * @param {string} message
   * @param {number} [status]
   * @param {ErrorCode} [code]
   */
  constructor(message, status, code) {
    super(message);
    this.name = 'BitviewError';
    this.status = status;
    this.code = code;
  }
}

/**
 * The server's error body as a `BitviewError`; a body this client can't read keeps its text.
 * @param {Response} res
 * @returns {Promise<BitviewError>}
 */
const _responseError = async (res) => {
  const text = await res.text().catch(() => '');
  /** @type {Partial<ErrorDetail> | undefined} */
  let detail;
  try {
    detail = JSON.parse(text)?.error;
  } catch {}
  const message = typeof detail?.message === 'string' ? detail.message : text || `HTTP ${res.status}`;
  return new BitviewError(message, res.status, detail?.code);
};

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
    case 'week1': return new Date(_EPOCH_MS - 3 * _MS_PER_DAY + i * _MS_PER_WEEK);
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
 * Throws for dates before the index's first date (2009-01-01; day3's is 2008-12-31, week1's 2008-12-29).
 * @param {Index} index - The index type
 * @param {globalThis.Date} d - The date to convert
 * @returns {number}
 */
function dateToIndex(index, d) {
  const ms = d.getTime(), first = indexToDate(index, 0);
  if (!(ms >= first.getTime())) throw new RangeError(`Date is before the first ${index} date (${first.toISOString().slice(0, 10)})`);
  switch (index) {
    case 'minute10': return Math.floor((ms - _EPOCH_MS) / 600000);
    case 'minute30': return Math.floor((ms - _EPOCH_MS) / 1800000);
    case 'hour1': return Math.floor((ms - _EPOCH_MS) / 3600000);
    case 'hour4': return Math.floor((ms - _EPOCH_MS) / 14400000);
    case 'hour12': return Math.floor((ms - _EPOCH_MS) / 43200000);
    case 'day1': return Math.floor((ms - _EPOCH_MS) / _MS_PER_DAY);
    case 'day3': return Math.floor((ms - _EPOCH_MS + 86400000) / 259200000);
    case 'week1': return Math.floor((ms - _EPOCH_MS + 3 * _MS_PER_DAY) / _MS_PER_WEEK);
    case 'month1': return (d.getUTCFullYear() - 2009) * 12 + d.getUTCMonth();
    case 'month3': return (d.getUTCFullYear() - 2009) * 4 + Math.floor(d.getUTCMonth() / 3);
    case 'month6': return (d.getUTCFullYear() - 2009) * 2 + Math.floor(d.getUTCMonth() / 6);
    case 'year1': return d.getUTCFullYear() - 2009;
    case 'year10': return Math.floor((d.getUTCFullYear() - 2000) / 10);
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
 * @property {string} type - Value type (e.g. "Ratio", "Sats", "Cents")
 * @property {number} start - Start index (inclusive)
 * @property {number} end - End index (exclusive)
 * @property {string} stamp - ISO 8601 timestamp of when the response was generated
 * @property {T[]} data - The series data (`null` where a value is missing or undefined)
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
   * Counted selections send `limit`: the server ends them after resolving a negative start.
   * @param {{ start?: number, end?: number, limit?: number }} range
   * @param {string} [format]
   * @returns {string}
   */
  const buildPath = ({ start, end, limit }, format) => {
    const params = new URLSearchParams();
    if (start !== undefined) params.set('start', String(start));
    if (end !== undefined) params.set('end', String(end));
    if (limit !== undefined) params.set('limit', String(limit));
    if (format) params.set('format', format);
    const query = params.toString();
    return query ? `${p}?${query}` : p;
  };

  /**
   * @param {{ start?: number, end?: number, limit?: number }} range
   * @returns {DateRangeBuilder<T>}
   */
  const rangeBuilder = (range) => ({
    fetch(arg, options) { return client._fetchSeriesData(buildPath(range), arg, options); },
    fetchCsv(options) { return client.getText(buildPath(range, 'csv'), options); },
    then(resolve, reject) { return this.fetch().then(resolve, reject); },
  });

  /**
   * @param {number} start
   * @returns {DateSkippedBuilder<T>}
   */
  const skippedBuilder = (start) => ({
    take(n) { return rangeBuilder({ start, limit: n }); },
    fetch(arg, options) { return client._fetchSeriesData(buildPath({ start }), arg, options); },
    fetchCsv(options) { return client.getText(buildPath({ start }, 'csv'), options); },
    then(resolve, reject) { return this.fetch().then(resolve, reject); },
  });

  /** @type {DateSeriesEndpoint<T>} */
  const endpoint = {
    get(idx) {
      if (idx instanceof Date) idx = dateToIndex(index, idx);
      return rangeBuilder({ start: idx, limit: 1 });
    },
    slice(start, end) {
      if (start instanceof Date) start = dateToIndex(index, start);
      if (end instanceof Date) end = dateToIndex(index, end);
      return rangeBuilder({ start, end });
    },
    first(n) { return rangeBuilder({ end: n }); },
    last(n) { return rangeBuilder(n === 0 ? { end: 0 } : { start: -n }); },
    skip(n) { return skippedBuilder(n); },
    fetch(arg, options) { return client._fetchSeriesData(buildPath({}), arg, options); },
    fetchCsv(options) { return client.getText(buildPath({}, 'csv'), options); },
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
    if (!res.ok && !(revalidate && res.status === 304)) throw await _responseError(res);
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
    if (!res.ok) throw await _responseError(res);
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
const _i2 = /** @type {const} */ (["minute10", "minute30", "hour1", "hour4", "hour12", "day1", "day3", "week1", "month1", "month3", "month6", "year1", "year10", "halving", "epoch", "height"]);
const _i3 = /** @type {const} */ (["minute10", "minute30", "hour1", "hour4", "hour12", "day1", "day3", "week1", "month1", "month3", "month6", "year1", "year10", "halving", "epoch", "height"]);
const _i4 = /** @type {const} */ (["minute10", "minute30", "hour1", "hour4", "hour12", "day1", "day3", "week1", "month1", "month3", "month6", "year1", "year10", "halving", "epoch"]);
const _i5 = /** @type {const} */ (["minute10", "minute30", "hour1", "hour4", "hour12", "day1", "day3", "week1", "month1", "month3", "month6", "year1", "year10", "halving", "epoch"]);
const _i6 = /** @type {const} */ (["minute10"]);
const _i7 = /** @type {const} */ (["minute30"]);
const _i8 = /** @type {const} */ (["hour1"]);
const _i9 = /** @type {const} */ (["hour4"]);
const _i10 = /** @type {const} */ (["hour12"]);
const _i11 = /** @type {const} */ (["day1"]);
const _i12 = /** @type {const} */ (["day3"]);
const _i13 = /** @type {const} */ (["week1"]);
const _i14 = /** @type {const} */ (["month1"]);
const _i15 = /** @type {const} */ (["month3"]);
const _i16 = /** @type {const} */ (["month6"]);
const _i17 = /** @type {const} */ (["year1"]);
const _i18 = /** @type {const} */ (["year10"]);
const _i19 = /** @type {const} */ (["halving"]);
const _i20 = /** @type {const} */ (["epoch"]);
const _i21 = /** @type {const} */ (["height"]);
const _i22 = /** @type {const} */ (["tx_index"]);
const _i23 = /** @type {const} */ (["txin_index"]);
const _i24 = /** @type {const} */ (["txout_index"]);
const _i25 = /** @type {const} */ (["empty_output_index"]);
const _i26 = /** @type {const} */ (["op_return_index"]);
const _i27 = /** @type {const} */ (["p2a_addr_index"]);
const _i28 = /** @type {const} */ (["p2ms_output_index"]);
const _i29 = /** @type {const} */ (["p2pk33_addr_index"]);
const _i30 = /** @type {const} */ (["p2pk65_addr_index"]);
const _i31 = /** @type {const} */ (["p2pkh_addr_index"]);
const _i32 = /** @type {const} */ (["p2sh_addr_index"]);
const _i33 = /** @type {const} */ (["p2tr_addr_index"]);
const _i34 = /** @type {const} */ (["p2wpkh_addr_index"]);
const _i35 = /** @type {const} */ (["p2wsh_addr_index"]);
const _i36 = /** @type {const} */ (["unknown_output_index"]);
const _i37 = /** @type {const} */ (["funded_addr_index"]);
const _i38 = /** @type {const} */ (["extended_empty_addr_index"]);

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
/** @template T @typedef {{ name: string, by: { readonly minute10: DateSeriesEndpoint<T | null>, readonly minute30: DateSeriesEndpoint<T | null>, readonly hour1: DateSeriesEndpoint<T | null>, readonly hour4: DateSeriesEndpoint<T | null>, readonly hour12: DateSeriesEndpoint<T | null>, readonly day1: DateSeriesEndpoint<T | null>, readonly day3: DateSeriesEndpoint<T | null>, readonly week1: DateSeriesEndpoint<T | null>, readonly month1: DateSeriesEndpoint<T | null>, readonly month3: DateSeriesEndpoint<T | null>, readonly month6: DateSeriesEndpoint<T | null>, readonly year1: DateSeriesEndpoint<T | null>, readonly year10: DateSeriesEndpoint<T | null>, readonly halving: SeriesEndpoint<T>, readonly epoch: SeriesEndpoint<T>, readonly height: SeriesEndpoint<T> }, indexes: () => readonly Index[], get: (index: Index) => SeriesEndpoint<T | null>|undefined }} SeriesPattern2 */
/** @template T @typedef {{ name: string, by: { readonly minute10: DateSeriesEndpoint<T | null>, readonly minute30: DateSeriesEndpoint<T | null>, readonly hour1: DateSeriesEndpoint<T | null>, readonly hour4: DateSeriesEndpoint<T | null>, readonly hour12: DateSeriesEndpoint<T | null>, readonly day1: DateSeriesEndpoint<T | null>, readonly day3: DateSeriesEndpoint<T | null>, readonly week1: DateSeriesEndpoint<T | null>, readonly month1: DateSeriesEndpoint<T | null>, readonly month3: DateSeriesEndpoint<T | null>, readonly month6: DateSeriesEndpoint<T | null>, readonly year1: DateSeriesEndpoint<T | null>, readonly year10: DateSeriesEndpoint<T | null>, readonly halving: SeriesEndpoint<T | null>, readonly epoch: SeriesEndpoint<T | null>, readonly height: SeriesEndpoint<T | null> }, indexes: () => readonly Index[], get: (index: Index) => SeriesEndpoint<T | null>|undefined }} SeriesPattern3 */
/** @template T @typedef {{ name: string, by: { readonly minute10: DateSeriesEndpoint<T>, readonly minute30: DateSeriesEndpoint<T>, readonly hour1: DateSeriesEndpoint<T>, readonly hour4: DateSeriesEndpoint<T>, readonly hour12: DateSeriesEndpoint<T>, readonly day1: DateSeriesEndpoint<T>, readonly day3: DateSeriesEndpoint<T>, readonly week1: DateSeriesEndpoint<T>, readonly month1: DateSeriesEndpoint<T>, readonly month3: DateSeriesEndpoint<T>, readonly month6: DateSeriesEndpoint<T>, readonly year1: DateSeriesEndpoint<T>, readonly year10: DateSeriesEndpoint<T>, readonly halving: SeriesEndpoint<T>, readonly epoch: SeriesEndpoint<T> }, indexes: () => readonly Index[], get: (index: Index) => SeriesEndpoint<T>|undefined }} SeriesPattern4 */
/** @template T @typedef {{ name: string, by: { readonly minute10: DateSeriesEndpoint<T | null>, readonly minute30: DateSeriesEndpoint<T | null>, readonly hour1: DateSeriesEndpoint<T | null>, readonly hour4: DateSeriesEndpoint<T | null>, readonly hour12: DateSeriesEndpoint<T | null>, readonly day1: DateSeriesEndpoint<T | null>, readonly day3: DateSeriesEndpoint<T | null>, readonly week1: DateSeriesEndpoint<T | null>, readonly month1: DateSeriesEndpoint<T | null>, readonly month3: DateSeriesEndpoint<T | null>, readonly month6: DateSeriesEndpoint<T | null>, readonly year1: DateSeriesEndpoint<T | null>, readonly year10: DateSeriesEndpoint<T | null>, readonly halving: SeriesEndpoint<T>, readonly epoch: SeriesEndpoint<T> }, indexes: () => readonly Index[], get: (index: Index) => SeriesEndpoint<T | null>|undefined }} SeriesPattern5 */
/** @template T @typedef {{ name: string, by: { readonly minute10: DateSeriesEndpoint<T> }, indexes: () => readonly Index[], get: (index: Index) => SeriesEndpoint<T>|undefined }} SeriesPattern6 */
/** @template T @typedef {{ name: string, by: { readonly minute30: DateSeriesEndpoint<T> }, indexes: () => readonly Index[], get: (index: Index) => SeriesEndpoint<T>|undefined }} SeriesPattern7 */
/** @template T @typedef {{ name: string, by: { readonly hour1: DateSeriesEndpoint<T> }, indexes: () => readonly Index[], get: (index: Index) => SeriesEndpoint<T>|undefined }} SeriesPattern8 */
/** @template T @typedef {{ name: string, by: { readonly hour4: DateSeriesEndpoint<T> }, indexes: () => readonly Index[], get: (index: Index) => SeriesEndpoint<T>|undefined }} SeriesPattern9 */
/** @template T @typedef {{ name: string, by: { readonly hour12: DateSeriesEndpoint<T> }, indexes: () => readonly Index[], get: (index: Index) => SeriesEndpoint<T>|undefined }} SeriesPattern10 */
/** @template T @typedef {{ name: string, by: { readonly day1: DateSeriesEndpoint<T> }, indexes: () => readonly Index[], get: (index: Index) => SeriesEndpoint<T>|undefined }} SeriesPattern11 */
/** @template T @typedef {{ name: string, by: { readonly day3: DateSeriesEndpoint<T> }, indexes: () => readonly Index[], get: (index: Index) => SeriesEndpoint<T>|undefined }} SeriesPattern12 */
/** @template T @typedef {{ name: string, by: { readonly week1: DateSeriesEndpoint<T> }, indexes: () => readonly Index[], get: (index: Index) => SeriesEndpoint<T>|undefined }} SeriesPattern13 */
/** @template T @typedef {{ name: string, by: { readonly month1: DateSeriesEndpoint<T> }, indexes: () => readonly Index[], get: (index: Index) => SeriesEndpoint<T>|undefined }} SeriesPattern14 */
/** @template T @typedef {{ name: string, by: { readonly month3: DateSeriesEndpoint<T> }, indexes: () => readonly Index[], get: (index: Index) => SeriesEndpoint<T>|undefined }} SeriesPattern15 */
/** @template T @typedef {{ name: string, by: { readonly month6: DateSeriesEndpoint<T> }, indexes: () => readonly Index[], get: (index: Index) => SeriesEndpoint<T>|undefined }} SeriesPattern16 */
/** @template T @typedef {{ name: string, by: { readonly year1: DateSeriesEndpoint<T> }, indexes: () => readonly Index[], get: (index: Index) => SeriesEndpoint<T>|undefined }} SeriesPattern17 */
/** @template T @typedef {{ name: string, by: { readonly year10: DateSeriesEndpoint<T> }, indexes: () => readonly Index[], get: (index: Index) => SeriesEndpoint<T>|undefined }} SeriesPattern18 */
/** @template T @typedef {{ name: string, by: { readonly halving: SeriesEndpoint<T> }, indexes: () => readonly Index[], get: (index: Index) => SeriesEndpoint<T>|undefined }} SeriesPattern19 */
/** @template T @typedef {{ name: string, by: { readonly epoch: SeriesEndpoint<T> }, indexes: () => readonly Index[], get: (index: Index) => SeriesEndpoint<T>|undefined }} SeriesPattern20 */
/** @template T @typedef {{ name: string, by: { readonly height: SeriesEndpoint<T> }, indexes: () => readonly Index[], get: (index: Index) => SeriesEndpoint<T>|undefined }} SeriesPattern21 */
/** @template T @typedef {{ name: string, by: { readonly tx_index: SeriesEndpoint<T> }, indexes: () => readonly Index[], get: (index: Index) => SeriesEndpoint<T>|undefined }} SeriesPattern22 */
/** @template T @typedef {{ name: string, by: { readonly txin_index: SeriesEndpoint<T> }, indexes: () => readonly Index[], get: (index: Index) => SeriesEndpoint<T>|undefined }} SeriesPattern23 */
/** @template T @typedef {{ name: string, by: { readonly txout_index: SeriesEndpoint<T> }, indexes: () => readonly Index[], get: (index: Index) => SeriesEndpoint<T>|undefined }} SeriesPattern24 */
/** @template T @typedef {{ name: string, by: { readonly empty_output_index: SeriesEndpoint<T> }, indexes: () => readonly Index[], get: (index: Index) => SeriesEndpoint<T>|undefined }} SeriesPattern25 */
/** @template T @typedef {{ name: string, by: { readonly op_return_index: SeriesEndpoint<T> }, indexes: () => readonly Index[], get: (index: Index) => SeriesEndpoint<T>|undefined }} SeriesPattern26 */
/** @template T @typedef {{ name: string, by: { readonly p2a_addr_index: SeriesEndpoint<T> }, indexes: () => readonly Index[], get: (index: Index) => SeriesEndpoint<T>|undefined }} SeriesPattern27 */
/** @template T @typedef {{ name: string, by: { readonly p2ms_output_index: SeriesEndpoint<T> }, indexes: () => readonly Index[], get: (index: Index) => SeriesEndpoint<T>|undefined }} SeriesPattern28 */
/** @template T @typedef {{ name: string, by: { readonly p2pk33_addr_index: SeriesEndpoint<T> }, indexes: () => readonly Index[], get: (index: Index) => SeriesEndpoint<T>|undefined }} SeriesPattern29 */
/** @template T @typedef {{ name: string, by: { readonly p2pk65_addr_index: SeriesEndpoint<T> }, indexes: () => readonly Index[], get: (index: Index) => SeriesEndpoint<T>|undefined }} SeriesPattern30 */
/** @template T @typedef {{ name: string, by: { readonly p2pkh_addr_index: SeriesEndpoint<T> }, indexes: () => readonly Index[], get: (index: Index) => SeriesEndpoint<T>|undefined }} SeriesPattern31 */
/** @template T @typedef {{ name: string, by: { readonly p2sh_addr_index: SeriesEndpoint<T> }, indexes: () => readonly Index[], get: (index: Index) => SeriesEndpoint<T>|undefined }} SeriesPattern32 */
/** @template T @typedef {{ name: string, by: { readonly p2tr_addr_index: SeriesEndpoint<T> }, indexes: () => readonly Index[], get: (index: Index) => SeriesEndpoint<T>|undefined }} SeriesPattern33 */
/** @template T @typedef {{ name: string, by: { readonly p2wpkh_addr_index: SeriesEndpoint<T> }, indexes: () => readonly Index[], get: (index: Index) => SeriesEndpoint<T>|undefined }} SeriesPattern34 */
/** @template T @typedef {{ name: string, by: { readonly p2wsh_addr_index: SeriesEndpoint<T> }, indexes: () => readonly Index[], get: (index: Index) => SeriesEndpoint<T>|undefined }} SeriesPattern35 */
/** @template T @typedef {{ name: string, by: { readonly unknown_output_index: SeriesEndpoint<T> }, indexes: () => readonly Index[], get: (index: Index) => SeriesEndpoint<T>|undefined }} SeriesPattern36 */
/** @template T @typedef {{ name: string, by: { readonly funded_addr_index: SeriesEndpoint<T> }, indexes: () => readonly Index[], get: (index: Index) => SeriesEndpoint<T>|undefined }} SeriesPattern37 */
/** @template T @typedef {{ name: string, by: { readonly extended_empty_addr_index: SeriesEndpoint<T> }, indexes: () => readonly Index[], get: (index: Index) => SeriesEndpoint<T>|undefined }} SeriesPattern38 */

// Series tree

/**
 * Builds a series-tree node, or a leaf, from its base or series name.
 * @typedef {(c: BitviewClient, b: string, ...f: _Make[]) => any} _Make
 */

/**
 * A series-tree node over base `b`. Each child is `[make, template]`: its series name (a leaf, made
 * from its index list) or base (a node, made by its builder) is the template with `*` replaced by
 * `b`, dropping the joining `_` when `b` is empty. Children materialize on first access.
 * @param {BitviewClient} c
 * @param {string} b
 * @param {Record<string, [_Make | readonly Index[], string]>} children
 * @returns {any}
 */
function _n(c, b, children) {
  const node = {};
  for (const [key, [make, t]] of Object.entries(children)) {
    Object.defineProperty(node, key, {
      get() {
        const name = t.replace(b ? '*' : /^\*_|_?\*/, b);
        return _lazy(this, key, () => typeof make === 'function' ? make(c, name) : _mp(c, name, make));
      },
      enumerable: true,
      configurable: true,
    });
  }
  return node;
}

/**
 * The builder of a node whose children are fixed.
 * @param {Record<string, [_Make | readonly Index[], string]>} children
 * @returns {_Make}
 */
const _s = (children) => (c, b) => _n(c, b, children);

/**
 * @typedef {{
 *   supply: SeriesPattern2<?Bitcoin>,
 *   count: SeriesPattern2<Count>,
 * }} UtxoHistory
 */
const _UtxoHistory = _s({
  supply: [_i2, '*'],
  count: [_i2, 'utxo_count'],
});

/**
 * @typedef {{
 *   txinIndex: SeriesPattern24<TxInIndex>,
 * }} Spent
 */
const _Spent = _s({
  txinIndex: [_i24, '*'],
});

/**
 * @typedef {{
 *   native: SeriesPattern2<?Ratio64>,
 *   fiat: SeriesPattern2<?Ratio64>,
 * }} Velocity
 */
const _Velocity = _s({
  native: [_i2, '*_btc'],
  fiat: [_i2, '*_usd'],
});

/**
 * @template A
 * @typedef {{
 *   all: SeriesPattern2<A>,
 *   p2pk65: SeriesPattern2<A>,
 *   p2pk33: SeriesPattern2<A>,
 *   p2pkh: SeriesPattern2<A>,
 *   p2sh: SeriesPattern2<A>,
 *   p2wpkh: SeriesPattern2<A>,
 *   p2wsh: SeriesPattern2<A>,
 *   p2tr: SeriesPattern2<A>,
 *   p2a: SeriesPattern2<A>,
 * }} AddressesEmpty
 */
const _AddressesEmpty = _s({
  all: [_i2, '*'],
  p2pk65: [_i2, 'p2pk65_*'],
  p2pk33: [_i2, 'p2pk33_*'],
  p2pkh: [_i2, 'p2pkh_*'],
  p2sh: [_i2, 'p2sh_*'],
  p2wpkh: [_i2, 'p2wpkh_*'],
  p2wsh: [_i2, 'p2wsh_*'],
  p2tr: [_i2, 'p2tr_*'],
  p2a: [_i2, 'p2a_*'],
});

/**
 * @typedef {{
 *   funded: AddressesEmpty<Count>,
 *   total: AddressesEmpty<Count>,
 * }} ExposedCount
 */
const _ExposedCount = _s({
  funded: [_AddressesEmpty, '*'],
  total: [_AddressesEmpty, 'total_*'],
});

/**
 * @typedef {{
 *   _0sats: SeriesPattern2<?Dollars>,
 *   _1satTo10sats: SeriesPattern2<?Dollars>,
 *   _10satsTo100sats: SeriesPattern2<?Dollars>,
 *   _100satsTo1kSats: SeriesPattern2<?Dollars>,
 *   _1kSatsTo10kSats: SeriesPattern2<?Dollars>,
 *   _10kSatsTo100kSats: SeriesPattern2<?Dollars>,
 *   _100kSatsTo1mSats: SeriesPattern2<?Dollars>,
 *   _1mSatsTo10mSats: SeriesPattern2<?Dollars>,
 *   _10mSatsTo1btc: SeriesPattern2<?Dollars>,
 *   _1btcTo10btc: SeriesPattern2<?Dollars>,
 *   _10btcTo100btc: SeriesPattern2<?Dollars>,
 *   _100btcTo1kBtc: SeriesPattern2<?Dollars>,
 *   _1kBtcTo10kBtc: SeriesPattern2<?Dollars>,
 *   _10kBtcTo100kBtc: SeriesPattern2<?Dollars>,
 *   over100kBtc: SeriesPattern2<?Dollars>,
 * }} RealizedCap
 */
const _RealizedCap = _s({
  _0sats: [_i2, 'addrs_0sats_*'],
  _1satTo10sats: [_i2, 'addrs_1sat_to_10sats_*'],
  _10satsTo100sats: [_i2, 'addrs_10sats_to_100sats_*'],
  _100satsTo1kSats: [_i2, 'addrs_100sats_to_1k_sats_*'],
  _1kSatsTo10kSats: [_i2, 'addrs_1k_sats_to_10k_sats_*'],
  _10kSatsTo100kSats: [_i2, 'addrs_10k_sats_to_100k_sats_*'],
  _100kSatsTo1mSats: [_i2, 'addrs_100k_sats_to_1m_sats_*'],
  _1mSatsTo10mSats: [_i2, 'addrs_1m_sats_to_10m_sats_*'],
  _10mSatsTo1btc: [_i2, 'addrs_10m_sats_to_1btc_*'],
  _1btcTo10btc: [_i2, 'addrs_1btc_to_10btc_*'],
  _10btcTo100btc: [_i2, 'addrs_10btc_to_100btc_*'],
  _100btcTo1kBtc: [_i2, 'addrs_100btc_to_1k_btc_*'],
  _1kBtcTo10kBtc: [_i2, 'addrs_1k_btc_to_10k_btc_*'],
  _10kBtcTo100kBtc: [_i2, 'addrs_10k_btc_to_100k_btc_*'],
  over100kBtc: [_i2, 'addrs_over_100k_btc_*'],
});

/**
 * @typedef {{
 *   p2a: SeriesPattern27<AddrState>,
 *   p2pk33: SeriesPattern29<AddrState>,
 *   p2pk65: SeriesPattern30<AddrState>,
 *   p2pkh: SeriesPattern31<AddrState>,
 *   p2sh: SeriesPattern32<AddrState>,
 *   p2tr: SeriesPattern33<AddrState>,
 *   p2wpkh: SeriesPattern34<AddrState>,
 *   p2wsh: SeriesPattern35<AddrState>,
 *   funded: SeriesPattern37<FundedAddrData>,
 *   extendedEmpty: SeriesPattern38<EmptyAddrData>,
 * }} State
 */
const _State = _s({
  p2a: [_i27, '*_state'],
  p2pk33: [_i29, '*_state'],
  p2pk65: [_i30, '*_state'],
  p2pkh: [_i31, '*_state'],
  p2sh: [_i32, '*_state'],
  p2tr: [_i33, '*_state'],
  p2wpkh: [_i34, '*_state'],
  p2wsh: [_i35, '*_state'],
  funded: [_i37, 'funded_*_data'],
  extendedEmpty: [_i38, 'extended_empty_*_data'],
});

/**
 * @template A
 * @typedef {{
 *   p2pk65: SeriesPattern2<A>,
 *   p2pk33: SeriesPattern2<A>,
 *   p2pkh: SeriesPattern2<A>,
 *   p2ms: SeriesPattern2<A>,
 *   p2sh: SeriesPattern2<A>,
 *   p2wpkh: SeriesPattern2<A>,
 *   p2wsh: SeriesPattern2<A>,
 *   p2tr: SeriesPattern2<A>,
 *   p2a: SeriesPattern2<A>,
 *   unknown: SeriesPattern2<A>,
 *   empty: SeriesPattern2<A>,
 * }} CapType
 */
const _CapType = _s({
  p2pk65: [_i2, 'p2pk65_*'],
  p2pk33: [_i2, 'p2pk33_*'],
  p2pkh: [_i2, 'p2pkh_*'],
  p2ms: [_i2, 'p2ms_*'],
  p2sh: [_i2, 'p2sh_*'],
  p2wpkh: [_i2, 'p2wpkh_*'],
  p2wsh: [_i2, 'p2wsh_*'],
  p2tr: [_i2, 'p2tr_*'],
  p2a: [_i2, 'p2a_*'],
  unknown: [_i2, 'unknown_outputs_*'],
  empty: [_i2, 'empty_outputs_*'],
});

/**
 * @template A
 * @typedef {{
 *   _0sats: SeriesPattern2<A>,
 *   _1satTo10sats: SeriesPattern2<A>,
 *   _10satsTo100sats: SeriesPattern2<A>,
 *   _100satsTo1kSats: SeriesPattern2<A>,
 *   _1kSatsTo10kSats: SeriesPattern2<A>,
 *   _10kSatsTo100kSats: SeriesPattern2<A>,
 *   _100kSatsTo1mSats: SeriesPattern2<A>,
 *   _1mSatsTo10mSats: SeriesPattern2<A>,
 *   _10mSatsTo1btc: SeriesPattern2<A>,
 *   _1btcTo10btc: SeriesPattern2<A>,
 *   _10btcTo100btc: SeriesPattern2<A>,
 *   _100btcTo1kBtc: SeriesPattern2<A>,
 *   _1kBtcTo10kBtc: SeriesPattern2<A>,
 *   _10kBtcTo100kBtc: SeriesPattern2<A>,
 *   over100kBtc: SeriesPattern2<A>,
 * }} CapUtxoAmount
 */
const _CapUtxoAmount = _s({
  _0sats: [_i2, 'utxos_0sats_*'],
  _1satTo10sats: [_i2, 'utxos_1sat_to_10sats_*'],
  _10satsTo100sats: [_i2, 'utxos_10sats_to_100sats_*'],
  _100satsTo1kSats: [_i2, 'utxos_100sats_to_1k_sats_*'],
  _1kSatsTo10kSats: [_i2, 'utxos_1k_sats_to_10k_sats_*'],
  _10kSatsTo100kSats: [_i2, 'utxos_10k_sats_to_100k_sats_*'],
  _100kSatsTo1mSats: [_i2, 'utxos_100k_sats_to_1m_sats_*'],
  _1mSatsTo10mSats: [_i2, 'utxos_1m_sats_to_10m_sats_*'],
  _10mSatsTo1btc: [_i2, 'utxos_10m_sats_to_1btc_*'],
  _1btcTo10btc: [_i2, 'utxos_1btc_to_10btc_*'],
  _10btcTo100btc: [_i2, 'utxos_10btc_to_100btc_*'],
  _100btcTo1kBtc: [_i2, 'utxos_100btc_to_1k_btc_*'],
  _1kBtcTo10kBtc: [_i2, 'utxos_1k_btc_to_10k_btc_*'],
  _10kBtcTo100kBtc: [_i2, 'utxos_10k_btc_to_100k_btc_*'],
  over100kBtc: [_i2, 'utxos_over_100k_btc_*'],
});

/**
 * @typedef {{
 *   profit: SeriesPattern2<?Dollars>,
 *   loss: SeriesPattern2<?Dollars>,
 *   netPnl: SeriesPattern2<?Dollars>,
 *   nupl: SeriesPattern2<?Ratio>,
 * }} DiscountUnrealized
 */
const _DiscountUnrealized = _s({
  profit: [_i2, '*_unrealized_profit'],
  loss: [_i2, '*_unrealized_loss'],
  netPnl: [_i2, '*_net_unrealized_pnl'],
  nupl: [_i2, '*_nupl'],
});

/**
 * @typedef {{
 *   supplyDominance: SeriesPattern2<?Percent>,
 *   supplyInProfitShare: SeriesPattern2<?Percent>,
 *   supplyInLossShare: SeriesPattern2<?Percent>,
 *   unrealizedProfitToMcap: SeriesPattern2<?Ratio>,
 *   unrealizedLossToMcap: SeriesPattern2<?Ratio>,
 *   unrealizedProfitToOwnMcap: SeriesPattern2<?Ratio>,
 *   unrealizedLossToOwnMcap: SeriesPattern2<?Ratio>,
 *   unrealizedProfitToOwnGrossPnl: SeriesPattern2<?Ratio>,
 *   unrealizedLossToOwnGrossPnl: SeriesPattern2<?Ratio>,
 *   netUnrealizedPnlToOwnGrossPnl: SeriesPattern2<?Ratio>,
 *   investedCapitalInProfitShare: SeriesPattern2<?Percent>,
 *   investedCapitalInLossShare: SeriesPattern2<?Percent>,
 *   realizedCapToOwnMcap: SeriesPattern2<?Ratio>,
 *   netPnlChange1mToMcap: SeriesPattern2<?Ratio>,
 *   netPnlChange1mToRcap: SeriesPattern2<?Ratio>,
 * }} Relative
 */
const _Relative = _s({
  supplyDominance: [_i2, '*_supply_dominance'],
  supplyInProfitShare: [_i2, '*_supply_in_profit_share'],
  supplyInLossShare: [_i2, '*_supply_in_loss_share'],
  unrealizedProfitToMcap: [_i2, '*_unrealized_profit_to_mcap'],
  unrealizedLossToMcap: [_i2, '*_unrealized_loss_to_mcap'],
  unrealizedProfitToOwnMcap: [_i2, '*_unrealized_profit_to_own_mcap'],
  unrealizedLossToOwnMcap: [_i2, '*_unrealized_loss_to_own_mcap'],
  unrealizedProfitToOwnGrossPnl: [_i2, '*_unrealized_profit_to_own_gross_pnl'],
  unrealizedLossToOwnGrossPnl: [_i2, '*_unrealized_loss_to_own_gross_pnl'],
  netUnrealizedPnlToOwnGrossPnl: [_i2, '*_net_unrealized_pnl_to_own_gross_pnl'],
  investedCapitalInProfitShare: [_i2, '*_invested_capital_in_profit_share'],
  investedCapitalInLossShare: [_i2, '*_invested_capital_in_loss_share'],
  realizedCapToOwnMcap: [_i2, '*_realized_cap_to_own_mcap'],
  netPnlChange1mToMcap: [_i2, '*_net_pnl_change_1m_to_mcap'],
  netPnlChange1mToRcap: [_i2, '*_net_pnl_change_1m_to_rcap'],
});

/**
 * @typedef {{
 *   _1w: SeriesPattern2<?Ratio>,
 *   _1m: SeriesPattern2<?Ratio>,
 *   _1y: SeriesPattern2<?Ratio>,
 * }} SoprRatioExtended
 */
const _SoprRatioExtended = _s({
  _1w: [_i2, '*_1w'],
  _1m: [_i2, '*_1m'],
  _1y: [_i2, '*_1y'],
});

/**
 * @typedef {{
 *   perCoin: SeriesPattern2<?Dollars>,
 *   perDollar: SeriesPattern2<?Dollars>,
 * }} CostBasisInLoss
 */
const _CostBasisInLoss = _s({
  perCoin: [_i2, '*_coin'],
  perDollar: [_i2, '*_dollar'],
});

/**
 * @typedef {{
 *   profit: SeriesPattern2<?Dollars>,
 *   loss: SeriesPattern2<?Dollars>,
 *   netPnl: SeriesPattern2<?Dollars>,
 *   grossPnl: SeriesPattern2<?Dollars>,
 *   investedCapitalInProfit: SeriesPattern2<?Dollars>,
 *   investedCapitalInLoss: SeriesPattern2<?Dollars>,
 *   painIndex: SeriesPattern2<?Dollars>,
 *   greedIndex: SeriesPattern2<?Dollars>,
 *   netSentiment: SeriesPattern2<?Dollars>,
 *   nupl: SeriesPattern2<?Ratio>,
 * }} AllUnrealized
 */
const _AllUnrealized = _s({
  profit: [_i2, '*_unrealized_profit'],
  loss: [_i2, '*_unrealized_loss'],
  netPnl: [_i2, '*_net_unrealized_pnl'],
  grossPnl: [_i2, '*_unrealized_gross_pnl'],
  investedCapitalInProfit: [_i2, '*_invested_capital_in_profit'],
  investedCapitalInLoss: [_i2, '*_invested_capital_in_loss'],
  painIndex: [_i2, '*_pain_index'],
  greedIndex: [_i2, '*_greed_index'],
  netSentiment: [_i2, '*_net_sentiment'],
  nupl: [_i2, '*_nupl'],
});

/**
 * @typedef {{
 *   btc: SeriesPattern2<?Bitcoin>,
 *   usd: SeriesPattern2<?Dollars>,
 * }} AllSupplyTotal
 */
const _AllSupplyTotal = _s({
  btc: [_i2, '*'],
  usd: [_i2, 'market_cap'],
});

/**
 * @typedef {{
 *   min: SeriesPattern2<?Dollars>,
 *   max: SeriesPattern2<?Dollars>,
 * }} AgeBoundsAll
 */
const _AgeBoundsAll = _s({
  min: [_i2, '*_min'],
  max: [_i2, '*_max'],
});

/**
 * @typedef {{
 *   all: AgeBoundsAll,
 *   sth: AgeBoundsAll,
 *   lth: AgeBoundsAll,
 *   under4m: AgeBoundsAll,
 *   under6m: AgeBoundsAll,
 *   over4m: AgeBoundsAll,
 *   over6m: AgeBoundsAll,
 * }} AgeBounds
 */
const _AgeBounds = _s({
  all: [_AgeBoundsAll, '*_all_cost_basis'],
  sth: [_AgeBoundsAll, '*_sth_cost_basis'],
  lth: [_AgeBoundsAll, '*_lth_cost_basis'],
  under4m: [_AgeBoundsAll, '*_under_4m_cost_basis'],
  under6m: [_AgeBoundsAll, '*_under_6m_cost_basis'],
  over4m: [_AgeBoundsAll, '*_over_4m_cost_basis'],
  over6m: [_AgeBoundsAll, '*_over_6m_cost_basis'],
});

/**
 * @template A
 * @typedef {{
 *   _2009: SeriesPattern2<A>,
 *   _2010: SeriesPattern2<A>,
 *   _2011: SeriesPattern2<A>,
 *   _2012: SeriesPattern2<A>,
 *   _2013: SeriesPattern2<A>,
 *   _2014: SeriesPattern2<A>,
 *   _2015: SeriesPattern2<A>,
 *   _2016: SeriesPattern2<A>,
 *   _2017: SeriesPattern2<A>,
 *   _2018: SeriesPattern2<A>,
 *   _2019: SeriesPattern2<A>,
 *   _2020: SeriesPattern2<A>,
 *   _2021: SeriesPattern2<A>,
 *   _2022: SeriesPattern2<A>,
 *   _2023: SeriesPattern2<A>,
 *   _2024: SeriesPattern2<A>,
 *   _2025: SeriesPattern2<A>,
 *   _2026: SeriesPattern2<A>,
 * }} CapClass
 */
const _CapClass = _s({
  _2009: [_i2, 'class_2009_*'],
  _2010: [_i2, 'class_2010_*'],
  _2011: [_i2, 'class_2011_*'],
  _2012: [_i2, 'class_2012_*'],
  _2013: [_i2, 'class_2013_*'],
  _2014: [_i2, 'class_2014_*'],
  _2015: [_i2, 'class_2015_*'],
  _2016: [_i2, 'class_2016_*'],
  _2017: [_i2, 'class_2017_*'],
  _2018: [_i2, 'class_2018_*'],
  _2019: [_i2, 'class_2019_*'],
  _2020: [_i2, 'class_2020_*'],
  _2021: [_i2, 'class_2021_*'],
  _2022: [_i2, 'class_2022_*'],
  _2023: [_i2, 'class_2023_*'],
  _2024: [_i2, 'class_2024_*'],
  _2025: [_i2, 'class_2025_*'],
  _2026: [_i2, 'class_2026_*'],
});

/**
 * @template A
 * @typedef {{
 *   _0: SeriesPattern2<A>,
 *   _1: SeriesPattern2<A>,
 *   _2: SeriesPattern2<A>,
 *   _3: SeriesPattern2<A>,
 *   _4: SeriesPattern2<A>,
 * }} CapEpoch
 */
const _CapEpoch = _s({
  _0: [_i2, 'epoch_0_*'],
  _1: [_i2, 'epoch_1_*'],
  _2: [_i2, 'epoch_2_*'],
  _3: [_i2, 'epoch_3_*'],
  _4: [_i2, 'epoch_4_*'],
});

/**
 * @typedef {{
 *   _24h: SeriesPattern2<?Percent>,
 *   _1w: SeriesPattern2<?Percent>,
 *   _1m: SeriesPattern2<?Percent>,
 *   _1y: SeriesPattern2<?Percent>,
 * }} AllRate
 */
const _AllRate = _s({
  _24h: [_i2, '*_24h_rate'],
  _1w: [_i2, '*_1w_rate'],
  _1m: [_i2, '*_1m_rate'],
  _1y: [_i2, '*_1y_rate'],
});

/**
 * @typedef {{
 *   open: SeriesPattern4<?Dollars>,
 *   high: SeriesPattern4<?Dollars>,
 *   low: SeriesPattern4<?Dollars>,
 *   close: SeriesPattern5<?Dollars>,
 * }} Split
 */
const _Split = _s({
  open: [_i4, '*_open'],
  high: [_i4, '*_high'],
  low: [_i4, '*_low'],
  close: [_i5, '*_close'],
});

/**
 * @typedef {{
 *   split: Split,
 *   ohlc: SeriesPattern4<OHLCDollars>,
 *   spot: SeriesPattern2<?Dollars>,
 *   satsPerDollar: SeriesPattern2<Sats>,
 * }} Price
 */
const _Price = _s({
  split: [_Split, '*'],
  ohlc: [_i4, '*_ohlc'],
  spot: [_i2, '*'],
  satsPerDollar: [_i2, 'sats_per_dollar'],
});

/**
 * @typedef {{
 *   cumulative: SeriesPattern2<?Percent>,
 * }} Dominance
 */
const _Dominance = _s({
  cumulative: [_i2, '*'],
});

/**
 * @typedef {{
 *   emaFast: SeriesPattern2<?Dollars>,
 *   emaSlow: SeriesPattern2<?Dollars>,
 *   line: SeriesPattern2<?Dollars>,
 *   signal: SeriesPattern2<?Dollars>,
 *   histogram: SeriesPattern2<?Dollars>,
 * }} Macd1m
 */
const _Macd1m = _s({
  emaFast: [_i2, 'macd_ema_fast_*'],
  emaSlow: [_i2, 'macd_ema_slow_*'],
  line: [_i2, 'macd_line_*'],
  signal: [_i2, 'macd_signal_*'],
  histogram: [_i2, 'macd_histogram_*'],
});

/**
 * @typedef {{
 *   rsi: SeriesPattern2<?Percent>,
 *   stochRsiK: SeriesPattern2<?Percent>,
 *   stochRsiD: SeriesPattern2<?Percent>,
 * }} Rsi1m
 */
const _Rsi1m = _s({
  rsi: [_i2, 'rsi_*'],
  stochRsiK: [_i2, 'rsi_stoch_k_*'],
  stochRsiD: [_i2, 'rsi_stoch_d_*'],
});

/**
 * @template A
 * @typedef {{
 *   _24h: A,
 *   _1w: A,
 *   _1m: A,
 * }} Macd
 */
/** @type {_Make} */
const _Macd = (c, b, f0) => _n(c, b, {
  _24h: [f0, '*'],
  _1w: [f0, '1w'],
  _1m: [f0, '1m'],
});

/**
 * @typedef {{
 *   block: SeriesPattern2<?Dollars>,
 *   ratio: SeriesPattern2<?Ratio>,
 *   x2: SeriesPattern2<?Dollars>,
 * }} Sma350d
 */
const _Sma350d = _s({
  block: [_i2, '*'],
  ratio: [_i2, '*_ratio'],
  x2: [_i2, '*_x2'],
});

/**
 * @typedef {{
 *   block: SeriesPattern2<?Dollars>,
 *   ratio: SeriesPattern2<?Ratio>,
 *   x24: SeriesPattern2<?Dollars>,
 *   x08: SeriesPattern2<?Dollars>,
 *   mayerMultiple: SeriesPattern2<?Ratio>,
 * }} Sma200d
 */
const _Sma200d = _s({
  block: [_i2, '*'],
  ratio: [_i2, '*_ratio'],
  x24: [_i2, '*_x2_4'],
  x08: [_i2, '*_x0_8'],
  mayerMultiple: [_i2, 'mayer_multiple'],
});

/**
 * @typedef {{
 *   _1w: SeriesPattern2<?Dollars>,
 *   _2w: SeriesPattern2<?Dollars>,
 *   _1m: SeriesPattern2<?Dollars>,
 *   _1y: SeriesPattern2<?Dollars>,
 * }} Max
 */
const _Max = _s({
  _1w: [_i2, '*_1w'],
  _2w: [_i2, '*_2w'],
  _1m: [_i2, '*_1m'],
  _1y: [_i2, '*_1y'],
});

/**
 * @typedef {{
 *   min: Max,
 *   max: Max,
 *   trueRange: SeriesPattern2<?Dollars>,
 *   trueRangeSum2w: SeriesPattern2<?Dollars>,
 *   choppinessIndex2w: SeriesPattern2<?Percent>,
 * }} Range
 */
const _Range = _s({
  min: [_Max, '*_min'],
  max: [_Max, '*_max'],
  trueRange: [_i2, '*_true_range'],
  trueRangeSum2w: [_i2, '*_true_range_sum_2w'],
  choppinessIndex2w: [_i2, '*_choppiness_index_2w'],
});

/**
 * @typedef {{
 *   _2y: SeriesPattern2<?Percent>,
 *   _3y: SeriesPattern2<?Percent>,
 *   _4y: SeriesPattern2<?Percent>,
 *   _5y: SeriesPattern2<?Percent>,
 *   _6y: SeriesPattern2<?Percent>,
 *   _8y: SeriesPattern2<?Percent>,
 *   _10y: SeriesPattern2<?Percent>,
 * }} Cagr
 */
const _Cagr = _s({
  _2y: [_i2, '*_2y'],
  _3y: [_i2, '*_3y'],
  _4y: [_i2, '*_4y'],
  _5y: [_i2, '*_5y'],
  _6y: [_i2, '*_6y'],
  _8y: [_i2, '*_8y'],
  _10y: [_i2, '*_10y'],
});

/**
 * @template A
 * @typedef {{
 *   _24h: SeriesPattern2<A>,
 *   _1w: SeriesPattern2<A>,
 *   _1m: SeriesPattern2<A>,
 *   _3m: SeriesPattern2<A>,
 *   _6m: SeriesPattern2<A>,
 *   _1y: SeriesPattern2<A>,
 *   _2y: SeriesPattern2<A>,
 *   _3y: SeriesPattern2<A>,
 *   _4y: SeriesPattern2<A>,
 *   _5y: SeriesPattern2<A>,
 *   _6y: SeriesPattern2<A>,
 *   _8y: SeriesPattern2<A>,
 *   _10y: SeriesPattern2<A>,
 * }} MarketLookback
 */
const _MarketLookback = _s({
  _24h: [_i2, '*_24h'],
  _1w: [_i2, '*_1w'],
  _1m: [_i2, '*_1m'],
  _3m: [_i2, '*_3m'],
  _6m: [_i2, '*_6m'],
  _1y: [_i2, '*_1y'],
  _2y: [_i2, '*_2y'],
  _3y: [_i2, '*_3y'],
  _4y: [_i2, '*_4y'],
  _5y: [_i2, '*_5y'],
  _6y: [_i2, '*_6y'],
  _8y: [_i2, '*_8y'],
  _10y: [_i2, '*_10y'],
});

/**
 * @typedef {{
 *   high: SeriesPattern2<?Dollars>,
 *   drawdown: SeriesPattern2<?Percent>,
 *   daysSince: SeriesPattern2<?Days>,
 *   maxDaysBetween: SeriesPattern2<?Days>,
 * }} Ath
 */
const _Ath = _s({
  high: [_i2, '*_ath'],
  drawdown: [_i2, '*_drawdown'],
  daysSince: [_i2, 'days_since_*_ath'],
  maxDaysBetween: [_i2, 'max_days_between_*_ath'],
});

/**
 * @typedef {{
 *   supplyAdj: SeriesPattern2<?Float32>,
 *   flow: SeriesPattern2<?Float32>,
 * }} Dormancy
 */
const _Dormancy = _s({
  supplyAdj: [_i2, '*_supply_adj'],
  flow: [_i2, '*_flow'],
});

/**
 * @typedef {{
 *   puellMultiple: SeriesPattern2<?Ratio>,
 *   nvt: SeriesPattern2<?Ratio>,
 *   gini: SeriesPattern2<?Ratio>,
 *   rhodlRatio: SeriesPattern2<?Ratio>,
 *   thermoCapMultiple: SeriesPattern2<?Ratio>,
 *   coindaysDestroyedSupplyAdj: SeriesPattern2<?Days>,
 *   coinyearsDestroyedSupplyAdj: SeriesPattern2<?Years>,
 *   dormancy: Dormancy,
 *   stockToFlow: SeriesPattern2<?Years>,
 *   sellerExhaustion: SeriesPattern2<?Ratio>,
 * }} Indicators
 */
const _Indicators = _s({
  puellMultiple: [_i2, 'puell_multiple'],
  nvt: [_i2, 'nvt'],
  gini: [_i2, 'gini'],
  rhodlRatio: [_i2, 'rhodl_ratio'],
  thermoCapMultiple: [_i2, 'thermo_cap_multiple'],
  coindaysDestroyedSupplyAdj: [_i2, 'coindays_*'],
  coinyearsDestroyedSupplyAdj: [_i2, 'coinyears_*'],
  dormancy: [_Dormancy, 'dormancy'],
  stockToFlow: [_i2, 'stock_to_flow'],
  sellerExhaustion: [_i2, 'seller_exhaustion'],
});

/**
 * @typedef {{
 *   monotonic: SeriesPattern21<Timestamp>,
 *   resolutions: SeriesPattern4<Timestamp>,
 * }} MappingsTimestamp
 */
const _MappingsTimestamp = _s({
  monotonic: [_i21, '*_monotonic'],
  resolutions: [_i4, '*'],
});

/**
 * @typedef {{
 *   identity: SeriesPattern24<TxOutIndex>,
 * }} TxoutIndex
 */
const _TxoutIndex = _s({
  identity: [_i24, '*'],
});

/**
 * @typedef {{
 *   identity: SeriesPattern23<TxInIndex>,
 * }} TxinIndex
 */
const _TxinIndex = _s({
  identity: [_i23, '*'],
});

/**
 * @typedef {{
 *   identity: SeriesPattern22<TxIndex>,
 *   inputCount: SeriesPattern22<Count>,
 *   outputCount: SeriesPattern22<Count>,
 * }} MappingsTxIndex
 */
const _MappingsTxIndex = _s({
  identity: [_i22, '*_index'],
  inputCount: [_i22, '*_input_count'],
  outputCount: [_i22, '*_output_count'],
});

/**
 * @typedef {{
 *   date: SeriesPattern18<Date>,
 *   firstHeight: SeriesPattern18<Height>,
 * }} MappingsYear10
 */
const _MappingsYear10 = _s({
  date: [_i18, '*'],
  firstHeight: [_i18, 'first_height'],
});

/**
 * @typedef {{
 *   date: SeriesPattern17<Date>,
 *   firstHeight: SeriesPattern17<Height>,
 * }} MappingsYear1
 */
const _MappingsYear1 = _s({
  date: [_i17, '*'],
  firstHeight: [_i17, 'first_height'],
});

/**
 * @typedef {{
 *   date: SeriesPattern16<Date>,
 *   firstHeight: SeriesPattern16<Height>,
 * }} MappingsMonth6
 */
const _MappingsMonth6 = _s({
  date: [_i16, '*'],
  firstHeight: [_i16, 'first_height'],
});

/**
 * @typedef {{
 *   date: SeriesPattern15<Date>,
 *   firstHeight: SeriesPattern15<Height>,
 * }} MappingsMonth3
 */
const _MappingsMonth3 = _s({
  date: [_i15, '*'],
  firstHeight: [_i15, 'first_height'],
});

/**
 * @typedef {{
 *   date: SeriesPattern14<Date>,
 *   firstHeight: SeriesPattern14<Height>,
 * }} MappingsMonth1
 */
const _MappingsMonth1 = _s({
  date: [_i14, '*'],
  firstHeight: [_i14, 'first_height'],
});

/**
 * @typedef {{
 *   date: SeriesPattern13<Date>,
 *   firstHeight: SeriesPattern13<Height>,
 * }} MappingsWeek1
 */
const _MappingsWeek1 = _s({
  date: [_i13, '*'],
  firstHeight: [_i13, 'first_height'],
});

/**
 * @typedef {{
 *   date: SeriesPattern12<Date>,
 *   firstHeight: SeriesPattern12<Height>,
 * }} MappingsDay3
 */
const _MappingsDay3 = _s({
  date: [_i12, '*'],
  firstHeight: [_i12, 'first_height'],
});

/**
 * @typedef {{
 *   date: SeriesPattern11<Date>,
 *   firstHeight: SeriesPattern11<Height>,
 * }} MappingsDay1
 */
const _MappingsDay1 = _s({
  date: [_i11, '*'],
  firstHeight: [_i11, 'first_height'],
});

/**
 * @typedef {{
 *   firstHeight: SeriesPattern10<Height>,
 * }} MappingsHour12
 */
const _MappingsHour12 = _s({
  firstHeight: [_i10, '*'],
});

/**
 * @typedef {{
 *   firstHeight: SeriesPattern9<Height>,
 * }} MappingsHour4
 */
const _MappingsHour4 = _s({
  firstHeight: [_i9, '*'],
});

/**
 * @typedef {{
 *   firstHeight: SeriesPattern8<Height>,
 * }} MappingsHour1
 */
const _MappingsHour1 = _s({
  firstHeight: [_i8, '*'],
});

/**
 * @typedef {{
 *   firstHeight: SeriesPattern7<Height>,
 * }} MappingsMinute30
 */
const _MappingsMinute30 = _s({
  firstHeight: [_i7, '*'],
});

/**
 * @typedef {{
 *   firstHeight: SeriesPattern6<Height>,
 * }} MappingsMinute10
 */
const _MappingsMinute10 = _s({
  firstHeight: [_i6, '*'],
});

/**
 * @typedef {{
 *   firstHeight: SeriesPattern19<Height>,
 * }} MappingsHalving
 */
const _MappingsHalving = _s({
  firstHeight: [_i19, '*'],
});

/**
 * @typedef {{
 *   firstHeight: SeriesPattern20<Height>,
 * }} MappingsEpoch
 */
const _MappingsEpoch = _s({
  firstHeight: [_i20, '*'],
});

/**
 * @typedef {{
 *   minute10: SeriesPattern21<Minute10>,
 *   minute30: SeriesPattern21<Minute30>,
 *   hour1: SeriesPattern21<Hour1>,
 *   hour4: SeriesPattern21<Hour4>,
 *   hour12: SeriesPattern21<Hour12>,
 *   day1: SeriesPattern21<Day1>,
 *   day3: SeriesPattern21<Day3>,
 *   epoch: SeriesPattern21<Epoch>,
 *   halving: SeriesPattern21<Halving>,
 *   week1: SeriesPattern21<Week1>,
 *   month1: SeriesPattern21<Month1>,
 *   month3: SeriesPattern21<Month3>,
 *   month6: SeriesPattern21<Month6>,
 *   year1: SeriesPattern21<Year1>,
 *   year10: SeriesPattern21<Year10>,
 *   txIndexCount: SeriesPattern21<Count>,
 * }} MappingsHeight
 */
const _MappingsHeight = _s({
  minute10: [_i21, '*'],
  minute30: [_i21, 'minute30'],
  hour1: [_i21, 'hour1'],
  hour4: [_i21, 'hour4'],
  hour12: [_i21, 'hour12'],
  day1: [_i21, 'day1'],
  day3: [_i21, 'day3'],
  epoch: [_i21, 'epoch'],
  halving: [_i21, 'halving'],
  week1: [_i21, 'week1'],
  month1: [_i21, 'month1'],
  month3: [_i21, 'month3'],
  month6: [_i21, 'month6'],
  year1: [_i21, 'year1'],
  year10: [_i21, 'year10'],
  txIndexCount: [_i21, 'tx_index_count'],
});

/**
 * @typedef {{
 *   identity: SeriesPattern26<OpReturnIndex>,
 * }} AddrOpReturn
 */
const _AddrOpReturn = _s({
  identity: [_i26, '*'],
});

/**
 * @typedef {{
 *   identity: SeriesPattern36<UnknownOutputIndex>,
 * }} AddrUnknown
 */
const _AddrUnknown = _s({
  identity: [_i36, '*'],
});

/**
 * @typedef {{
 *   identity: SeriesPattern25<EmptyOutputIndex>,
 * }} AddrEmpty
 */
const _AddrEmpty = _s({
  identity: [_i25, '*'],
});

/**
 * @typedef {{
 *   identity: SeriesPattern28<P2MSOutputIndex>,
 * }} AddrP2ms
 */
const _AddrP2ms = _s({
  identity: [_i28, '*'],
});

/**
 * @typedef {{
 *   identity: SeriesPattern27<P2AAddrIndex>,
 *   addr: SeriesPattern27<Addr>,
 * }} AddrP2a
 */
const _AddrP2a = _s({
  identity: [_i27, '*_index'],
  addr: [_i27, '*'],
});

/**
 * @typedef {{
 *   identity: SeriesPattern35<P2WSHAddrIndex>,
 *   addr: SeriesPattern35<Addr>,
 * }} AddrP2wsh
 */
const _AddrP2wsh = _s({
  identity: [_i35, '*_index'],
  addr: [_i35, '*'],
});

/**
 * @typedef {{
 *   identity: SeriesPattern34<P2WPKHAddrIndex>,
 *   addr: SeriesPattern34<Addr>,
 * }} AddrP2wpkh
 */
const _AddrP2wpkh = _s({
  identity: [_i34, '*_index'],
  addr: [_i34, '*'],
});

/**
 * @typedef {{
 *   identity: SeriesPattern33<P2TRAddrIndex>,
 *   addr: SeriesPattern33<Addr>,
 * }} AddrP2tr
 */
const _AddrP2tr = _s({
  identity: [_i33, '*_index'],
  addr: [_i33, '*'],
});

/**
 * @typedef {{
 *   identity: SeriesPattern32<P2SHAddrIndex>,
 *   addr: SeriesPattern32<Addr>,
 * }} AddrP2sh
 */
const _AddrP2sh = _s({
  identity: [_i32, '*_index'],
  addr: [_i32, '*'],
});

/**
 * @typedef {{
 *   identity: SeriesPattern31<P2PKHAddrIndex>,
 *   addr: SeriesPattern31<Addr>,
 * }} AddrP2pkh
 */
const _AddrP2pkh = _s({
  identity: [_i31, '*_index'],
  addr: [_i31, '*'],
});

/**
 * @typedef {{
 *   identity: SeriesPattern30<P2PK65AddrIndex>,
 *   addr: SeriesPattern30<Addr>,
 * }} AddrP2pk65
 */
const _AddrP2pk65 = _s({
  identity: [_i30, '*_index'],
  addr: [_i30, '*'],
});

/**
 * @typedef {{
 *   identity: SeriesPattern29<P2PK33AddrIndex>,
 *   addr: SeriesPattern29<Addr>,
 * }} AddrP2pk33
 */
const _AddrP2pk33 = _s({
  identity: [_i29, '*_index'],
  addr: [_i29, '*'],
});

/**
 * @typedef {{
 *   p2pk33: AddrP2pk33,
 *   p2pk65: AddrP2pk65,
 *   p2pkh: AddrP2pkh,
 *   p2sh: AddrP2sh,
 *   p2tr: AddrP2tr,
 *   p2wpkh: AddrP2wpkh,
 *   p2wsh: AddrP2wsh,
 *   p2a: AddrP2a,
 *   p2ms: AddrP2ms,
 *   empty: AddrEmpty,
 *   unknown: AddrUnknown,
 *   opReturn: AddrOpReturn,
 * }} MappingsAddr
 */
const _MappingsAddr = _s({
  p2pk33: [_AddrP2pk33, 'p2pk33_*'],
  p2pk65: [_AddrP2pk65, 'p2pk65_*'],
  p2pkh: [_AddrP2pkh, 'p2pkh_*'],
  p2sh: [_AddrP2sh, 'p2sh_*'],
  p2tr: [_AddrP2tr, 'p2tr_*'],
  p2wpkh: [_AddrP2wpkh, 'p2wpkh_*'],
  p2wsh: [_AddrP2wsh, 'p2wsh_*'],
  p2a: [_AddrP2a, 'p2a_*'],
  p2ms: [_AddrP2ms, 'p2ms_output_index'],
  empty: [_AddrEmpty, 'empty_output_index'],
  unknown: [_AddrUnknown, 'unknown_output_index'],
  opReturn: [_AddrOpReturn, 'op_return_index'],
});

/**
 * @typedef {{
 *   addr: MappingsAddr,
 *   height: MappingsHeight,
 *   epoch: MappingsEpoch,
 *   halving: MappingsHalving,
 *   minute10: MappingsMinute10,
 *   minute30: MappingsMinute30,
 *   hour1: MappingsHour1,
 *   hour4: MappingsHour4,
 *   hour12: MappingsHour12,
 *   day1: MappingsDay1,
 *   day3: MappingsDay3,
 *   week1: MappingsWeek1,
 *   month1: MappingsMonth1,
 *   month3: MappingsMonth3,
 *   month6: MappingsMonth6,
 *   year1: MappingsYear1,
 *   year10: MappingsYear10,
 *   txIndex: MappingsTxIndex,
 *   txinIndex: TxinIndex,
 *   txoutIndex: TxoutIndex,
 *   timestamp: MappingsTimestamp,
 * }} Mappings
 */
const _Mappings = _s({
  addr: [_MappingsAddr, 'addr'],
  height: [_MappingsHeight, 'minute10'],
  epoch: [_MappingsEpoch, 'first_height'],
  halving: [_MappingsHalving, 'first_height'],
  minute10: [_MappingsMinute10, 'first_height'],
  minute30: [_MappingsMinute30, 'first_height'],
  hour1: [_MappingsHour1, 'first_height'],
  hour4: [_MappingsHour4, 'first_height'],
  hour12: [_MappingsHour12, 'first_height'],
  day1: [_MappingsDay1, '*'],
  day3: [_MappingsDay3, '*'],
  week1: [_MappingsWeek1, '*'],
  month1: [_MappingsMonth1, '*'],
  month3: [_MappingsMonth3, '*'],
  month6: [_MappingsMonth6, '*'],
  year1: [_MappingsYear1, '*'],
  year10: [_MappingsYear10, '*'],
  txIndex: [_MappingsTxIndex, 'tx'],
  txinIndex: [_TxinIndex, 'txin_index'],
  txoutIndex: [_TxoutIndex, 'txout_index'],
  timestamp: [_MappingsTimestamp, 'timestamp'],
});

/**
 * @typedef {{
 *   pct01: SeriesPattern2<?Dollars>,
 *   pct05: SeriesPattern2<?Dollars>,
 *   pct1: SeriesPattern2<?Dollars>,
 *   pct2: SeriesPattern2<?Dollars>,
 *   pct5: SeriesPattern2<?Dollars>,
 *   pct10: SeriesPattern2<?Dollars>,
 *   pct20: SeriesPattern2<?Dollars>,
 *   pct30: SeriesPattern2<?Dollars>,
 *   pct40: SeriesPattern2<?Dollars>,
 *   median: SeriesPattern2<?Dollars>,
 *   pct60: SeriesPattern2<?Dollars>,
 *   pct70: SeriesPattern2<?Dollars>,
 *   pct80: SeriesPattern2<?Dollars>,
 *   pct90: SeriesPattern2<?Dollars>,
 *   pct95: SeriesPattern2<?Dollars>,
 *   pct98: SeriesPattern2<?Dollars>,
 *   pct99: SeriesPattern2<?Dollars>,
 *   pct995: SeriesPattern2<?Dollars>,
 *   pct999: SeriesPattern2<?Dollars>,
 *   level: SeriesPattern2<Score>,
 *   score: SeriesPattern2<Score>,
 * }} Cycle
 */
const _Cycle = _s({
  pct01: [_i2, '*_pct0_1'],
  pct05: [_i2, '*_pct0_5'],
  pct1: [_i2, '*_pct1'],
  pct2: [_i2, '*_pct2'],
  pct5: [_i2, '*_pct5'],
  pct10: [_i2, '*_pct10'],
  pct20: [_i2, '*_pct20'],
  pct30: [_i2, '*_pct30'],
  pct40: [_i2, '*_pct40'],
  median: [_i2, '*_median'],
  pct60: [_i2, '*_pct60'],
  pct70: [_i2, '*_pct70'],
  pct80: [_i2, '*_pct80'],
  pct90: [_i2, '*_pct90'],
  pct95: [_i2, '*_pct95'],
  pct98: [_i2, '*_pct98'],
  pct99: [_i2, '*_pct99'],
  pct995: [_i2, '*_pct99_5'],
  pct999: [_i2, '*_pct99_9'],
  level: [_i2, '*_level'],
  score: [_i2, '*_score'],
});

/**
 * @template A
 * @typedef {{
 *   thresholdPct01: SeriesPattern2<A>,
 *   thresholdPct005: SeriesPattern2<A>,
 *   thresholdPct0025: SeriesPattern2<A>,
 *   tail: SeriesPattern2<?Percent>,
 *   rank: SeriesPattern2<Rank>,
 * }} Capitulation
 */
const _Capitulation = _s({
  thresholdPct01: [_i2, '*_threshold_pct0_1'],
  thresholdPct005: [_i2, '*_threshold_pct0_05'],
  thresholdPct0025: [_i2, '*_threshold'],
  tail: [_i2, '*_tail'],
  rank: [_i2, '*_rank'],
});

/**
 * @typedef {{
 *   coinsInLoss: Capitulation<?Bitcoin>,
 *   profitTaking: Capitulation<?Dollars>,
 *   capitulation: Capitulation<?Dollars>,
 *   peakRegret: Capitulation<?Dollars>,
 *   sellerExhaustion: Capitulation<?Ratio>,
 * }} Extremes
 */
const _Extremes = _s({
  coinsInLoss: [_Capitulation, '*_coins_in_loss'],
  profitTaking: [_Capitulation, '*_profit_taking'],
  capitulation: [_Capitulation, '*_capitulation'],
  peakRegret: [_Capitulation, '*_peak_regret'],
  sellerExhaustion: [_Capitulation, '*_seller_exhaustion'],
});

/**
 * @typedef {{
 *   ratio: SeriesPattern2<?Ratio>,
 *   price: SeriesPattern2<?Dollars>,
 * }} Pct999
 */
const _Pct999 = _s({
  ratio: [_i2, '*_ratio_pct99_9'],
  price: [_i2, '*_pct99_9'],
});

/**
 * @typedef {{
 *   ratio: SeriesPattern2<?Ratio>,
 *   price: SeriesPattern2<?Dollars>,
 * }} Pct995
 */
const _Pct995 = _s({
  ratio: [_i2, '*_ratio_pct99_5'],
  price: [_i2, '*_pct99_5'],
});

/**
 * @typedef {{
 *   ratio: SeriesPattern2<?Ratio>,
 *   price: SeriesPattern2<?Dollars>,
 * }} Pct99
 */
const _Pct99 = _s({
  ratio: [_i2, '*_ratio_pct99'],
  price: [_i2, '*_pct99'],
});

/**
 * @typedef {{
 *   ratio: SeriesPattern2<?Ratio>,
 *   price: SeriesPattern2<?Dollars>,
 * }} Pct98
 */
const _Pct98 = _s({
  ratio: [_i2, '*_ratio_pct98'],
  price: [_i2, '*_pct98'],
});

/**
 * @typedef {{
 *   ratio: SeriesPattern2<?Ratio>,
 *   price: SeriesPattern2<?Dollars>,
 * }} Pct95
 */
const _Pct95 = _s({
  ratio: [_i2, '*_ratio_pct95'],
  price: [_i2, '*_pct95'],
});

/**
 * @typedef {{
 *   ratio: SeriesPattern2<?Ratio>,
 *   price: SeriesPattern2<?Dollars>,
 * }} Pct90
 */
const _Pct90 = _s({
  ratio: [_i2, '*_ratio_pct90'],
  price: [_i2, '*_pct90'],
});

/**
 * @typedef {{
 *   ratio: SeriesPattern2<?Ratio>,
 *   price: SeriesPattern2<?Dollars>,
 * }} Pct80
 */
const _Pct80 = _s({
  ratio: [_i2, '*_ratio_pct80'],
  price: [_i2, '*_pct80'],
});

/**
 * @typedef {{
 *   ratio: SeriesPattern2<?Ratio>,
 *   price: SeriesPattern2<?Dollars>,
 * }} Pct70
 */
const _Pct70 = _s({
  ratio: [_i2, '*_ratio_pct70'],
  price: [_i2, '*_pct70'],
});

/**
 * @typedef {{
 *   ratio: SeriesPattern2<?Ratio>,
 *   price: SeriesPattern2<?Dollars>,
 * }} Pct60
 */
const _Pct60 = _s({
  ratio: [_i2, '*_ratio_pct60'],
  price: [_i2, '*_pct60'],
});

/**
 * @typedef {{
 *   ratio: SeriesPattern2<?Ratio>,
 *   price: SeriesPattern2<?Dollars>,
 * }} Median
 */
const _Median = _s({
  ratio: [_i2, '*_ratio_median'],
  price: [_i2, '*_median'],
});

/**
 * @typedef {{
 *   ratio: SeriesPattern2<?Ratio>,
 *   price: SeriesPattern2<?Dollars>,
 * }} Pct40
 */
const _Pct40 = _s({
  ratio: [_i2, '*_ratio_pct40'],
  price: [_i2, '*_pct40'],
});

/**
 * @typedef {{
 *   ratio: SeriesPattern2<?Ratio>,
 *   price: SeriesPattern2<?Dollars>,
 * }} Pct30
 */
const _Pct30 = _s({
  ratio: [_i2, '*_ratio_pct30'],
  price: [_i2, '*_pct30'],
});

/**
 * @typedef {{
 *   ratio: SeriesPattern2<?Ratio>,
 *   price: SeriesPattern2<?Dollars>,
 * }} Pct20
 */
const _Pct20 = _s({
  ratio: [_i2, '*_ratio_pct20'],
  price: [_i2, '*_pct20'],
});

/**
 * @typedef {{
 *   ratio: SeriesPattern2<?Ratio>,
 *   price: SeriesPattern2<?Dollars>,
 * }} Pct10
 */
const _Pct10 = _s({
  ratio: [_i2, '*_ratio_pct10'],
  price: [_i2, '*_pct10'],
});

/**
 * @typedef {{
 *   ratio: SeriesPattern2<?Ratio>,
 *   price: SeriesPattern2<?Dollars>,
 * }} Pct5
 */
const _Pct5 = _s({
  ratio: [_i2, '*_ratio_pct5'],
  price: [_i2, '*_pct5'],
});

/**
 * @typedef {{
 *   ratio: SeriesPattern2<?Ratio>,
 *   price: SeriesPattern2<?Dollars>,
 * }} Pct2
 */
const _Pct2 = _s({
  ratio: [_i2, '*_ratio_pct2'],
  price: [_i2, '*_pct2'],
});

/**
 * @typedef {{
 *   ratio: SeriesPattern2<?Ratio>,
 *   price: SeriesPattern2<?Dollars>,
 * }} Pct1
 */
const _Pct1 = _s({
  ratio: [_i2, '*_ratio_pct1'],
  price: [_i2, '*_pct1'],
});

/**
 * @typedef {{
 *   ratio: SeriesPattern2<?Ratio>,
 *   price: SeriesPattern2<?Dollars>,
 * }} Pct05
 */
const _Pct05 = _s({
  ratio: [_i2, '*_ratio_pct0_5'],
  price: [_i2, '*_pct0_5'],
});

/**
 * @typedef {{
 *   ratio: SeriesPattern2<?Ratio>,
 *   price: SeriesPattern2<?Dollars>,
 * }} Pct01
 */
const _Pct01 = _s({
  ratio: [_i2, '*_ratio_pct0_1'],
  price: [_i2, '*_pct0_1'],
});

/**
 * @typedef {{
 *   block: SeriesPattern2<?Dollars>,
 *   ratio: SeriesPattern2<?Ratio>,
 *   pct01: Pct01,
 *   pct05: Pct05,
 *   pct1: Pct1,
 *   pct2: Pct2,
 *   pct5: Pct5,
 *   pct10: Pct10,
 *   pct20: Pct20,
 *   pct30: Pct30,
 *   pct40: Pct40,
 *   median: Median,
 *   pct60: Pct60,
 *   pct70: Pct70,
 *   pct80: Pct80,
 *   pct90: Pct90,
 *   pct95: Pct95,
 *   pct98: Pct98,
 *   pct99: Pct99,
 *   pct995: Pct995,
 *   pct999: Pct999,
 * }} CoinflowMedianPriceBtcWeighted
 */
const _CoinflowMedianPriceBtcWeighted = _s({
  block: [_i2, '*'],
  ratio: [_i2, '*_ratio'],
  pct01: [_Pct01, '*'],
  pct05: [_Pct05, '*'],
  pct1: [_Pct1, '*'],
  pct2: [_Pct2, '*'],
  pct5: [_Pct5, '*'],
  pct10: [_Pct10, '*'],
  pct20: [_Pct20, '*'],
  pct30: [_Pct30, '*'],
  pct40: [_Pct40, '*'],
  median: [_Median, '*'],
  pct60: [_Pct60, '*'],
  pct70: [_Pct70, '*'],
  pct80: [_Pct80, '*'],
  pct90: [_Pct90, '*'],
  pct95: [_Pct95, '*'],
  pct98: [_Pct98, '*'],
  pct99: [_Pct99, '*'],
  pct995: [_Pct995, '*'],
  pct999: [_Pct999, '*'],
});

/**
 * @typedef {{
 *   pct01: Pct01,
 *   pct05: Pct05,
 *   pct1: Pct1,
 *   pct2: Pct2,
 *   pct5: Pct5,
 *   pct10: Pct10,
 *   pct20: Pct20,
 *   pct30: Pct30,
 *   pct40: Pct40,
 *   median: Median,
 *   pct60: Pct60,
 *   pct70: Pct70,
 *   pct80: Pct80,
 *   pct90: Pct90,
 *   pct95: Pct95,
 *   pct98: Pct98,
 *   pct99: Pct99,
 *   pct995: Pct995,
 *   pct999: Pct999,
 * }} ActivePrice
 */
const _ActivePrice = _s({
  pct01: [_Pct01, '*'],
  pct05: [_Pct05, '*'],
  pct1: [_Pct1, '*'],
  pct2: [_Pct2, '*'],
  pct5: [_Pct5, '*'],
  pct10: [_Pct10, '*'],
  pct20: [_Pct20, '*'],
  pct30: [_Pct30, '*'],
  pct40: [_Pct40, '*'],
  median: [_Median, '*'],
  pct60: [_Pct60, '*'],
  pct70: [_Pct70, '*'],
  pct80: [_Pct80, '*'],
  pct90: [_Pct90, '*'],
  pct95: [_Pct95, '*'],
  pct98: [_Pct98, '*'],
  pct99: [_Pct99, '*'],
  pct995: [_Pct995, '*'],
  pct999: [_Pct999, '*'],
});

/**
 * @typedef {{
 *   realizedPrice: ActivePrice,
 *   capitalizedPrice: ActivePrice,
 *   medianPriceBtcWeighted: CoinflowMedianPriceBtcWeighted,
 *   medianPriceUsdWeighted: CoinflowMedianPriceBtcWeighted,
 *   sthMedianPriceBtcWeighted: CoinflowMedianPriceBtcWeighted,
 *   sthMedianPriceUsdWeighted: CoinflowMedianPriceBtcWeighted,
 *   lthMedianPriceBtcWeighted: CoinflowMedianPriceBtcWeighted,
 *   lthMedianPriceUsdWeighted: CoinflowMedianPriceBtcWeighted,
 *   cointimeMedianPriceBtcWeighted: CoinflowMedianPriceBtcWeighted,
 *   cointimeMedianPriceUsdWeighted: CoinflowMedianPriceBtcWeighted,
 *   coinflowMedianPriceBtcWeighted: CoinflowMedianPriceBtcWeighted,
 *   coinflowMedianPriceUsdWeighted: CoinflowMedianPriceBtcWeighted,
 *   sthCointimeMedianPriceBtcWeighted: CoinflowMedianPriceBtcWeighted,
 *   sthCointimeMedianPriceUsdWeighted: CoinflowMedianPriceBtcWeighted,
 *   lthCointimeMedianPriceBtcWeighted: CoinflowMedianPriceBtcWeighted,
 *   lthCointimeMedianPriceUsdWeighted: CoinflowMedianPriceBtcWeighted,
 *   sthCoinflowMedianPriceBtcWeighted: CoinflowMedianPriceBtcWeighted,
 *   sthCoinflowMedianPriceUsdWeighted: CoinflowMedianPriceBtcWeighted,
 *   lthCoinflowMedianPriceBtcWeighted: CoinflowMedianPriceBtcWeighted,
 *   lthCoinflowMedianPriceUsdWeighted: CoinflowMedianPriceBtcWeighted,
 *   sthRealizedPrice: ActivePrice,
 *   sthCapitalizedPrice: ActivePrice,
 *   lthRealizedPrice: ActivePrice,
 *   lthCapitalizedPrice: ActivePrice,
 *   over6mRealizedPrice: ActivePrice,
 *   over4mRealizedPrice: ActivePrice,
 *   under4mRealizedPrice: ActivePrice,
 *   under6mRealizedPrice: ActivePrice,
 *   under4mCapitalizedPrice: ActivePrice,
 *   under6mCapitalizedPrice: ActivePrice,
 *   vaultedPrice: ActivePrice,
 *   activePrice: ActivePrice,
 *   trueMarketMeanPrice: ActivePrice,
 *   cointimePrice: ActivePrice,
 *   awakePrice: ActivePrice,
 *   coinflowPrice: ActivePrice,
 * }} Components
 */
const _Components = _s({
  realizedPrice: [_ActivePrice, 'realized_*'],
  capitalizedPrice: [_ActivePrice, 'capitalized_*'],
  medianPriceBtcWeighted: [_CoinflowMedianPriceBtcWeighted, 'median_*_btc_weighted'],
  medianPriceUsdWeighted: [_CoinflowMedianPriceBtcWeighted, 'median_*_usd_weighted'],
  sthMedianPriceBtcWeighted: [_CoinflowMedianPriceBtcWeighted, 'sth_median_*_btc_weighted'],
  sthMedianPriceUsdWeighted: [_CoinflowMedianPriceBtcWeighted, 'sth_median_*_usd_weighted'],
  lthMedianPriceBtcWeighted: [_CoinflowMedianPriceBtcWeighted, 'lth_median_*_btc_weighted'],
  lthMedianPriceUsdWeighted: [_CoinflowMedianPriceBtcWeighted, 'lth_median_*_usd_weighted'],
  cointimeMedianPriceBtcWeighted: [_CoinflowMedianPriceBtcWeighted, 'cointime_median_*_btc_weighted'],
  cointimeMedianPriceUsdWeighted: [_CoinflowMedianPriceBtcWeighted, 'cointime_median_*_usd_weighted'],
  coinflowMedianPriceBtcWeighted: [_CoinflowMedianPriceBtcWeighted, 'coinflow_median_*_btc_weighted'],
  coinflowMedianPriceUsdWeighted: [_CoinflowMedianPriceBtcWeighted, 'coinflow_median_*_usd_weighted'],
  sthCointimeMedianPriceBtcWeighted: [_CoinflowMedianPriceBtcWeighted, 'sth_cointime_median_*_btc_weighted'],
  sthCointimeMedianPriceUsdWeighted: [_CoinflowMedianPriceBtcWeighted, 'sth_cointime_median_*_usd_weighted'],
  lthCointimeMedianPriceBtcWeighted: [_CoinflowMedianPriceBtcWeighted, 'lth_cointime_median_*_btc_weighted'],
  lthCointimeMedianPriceUsdWeighted: [_CoinflowMedianPriceBtcWeighted, 'lth_cointime_median_*_usd_weighted'],
  sthCoinflowMedianPriceBtcWeighted: [_CoinflowMedianPriceBtcWeighted, 'sth_coinflow_median_*_btc_weighted'],
  sthCoinflowMedianPriceUsdWeighted: [_CoinflowMedianPriceBtcWeighted, 'sth_coinflow_median_*_usd_weighted'],
  lthCoinflowMedianPriceBtcWeighted: [_CoinflowMedianPriceBtcWeighted, 'lth_coinflow_median_*_btc_weighted'],
  lthCoinflowMedianPriceUsdWeighted: [_CoinflowMedianPriceBtcWeighted, 'lth_coinflow_median_*_usd_weighted'],
  sthRealizedPrice: [_ActivePrice, 'sth_realized_*'],
  sthCapitalizedPrice: [_ActivePrice, 'sth_capitalized_*'],
  lthRealizedPrice: [_ActivePrice, 'lth_realized_*'],
  lthCapitalizedPrice: [_ActivePrice, 'lth_capitalized_*'],
  over6mRealizedPrice: [_ActivePrice, 'over_6m_realized_*'],
  over4mRealizedPrice: [_ActivePrice, 'over_4m_realized_*'],
  under4mRealizedPrice: [_ActivePrice, 'under_4m_realized_*'],
  under6mRealizedPrice: [_ActivePrice, 'under_6m_realized_*'],
  under4mCapitalizedPrice: [_ActivePrice, 'under_4m_capitalized_*'],
  under6mCapitalizedPrice: [_ActivePrice, 'under_6m_capitalized_*'],
  vaultedPrice: [_ActivePrice, 'vaulted_*'],
  activePrice: [_ActivePrice, 'active_*'],
  trueMarketMeanPrice: [_ActivePrice, 'true_market_mean_*'],
  cointimePrice: [_ActivePrice, 'cointime_*'],
  awakePrice: [_ActivePrice, 'awake_*'],
  coinflowPrice: [_ActivePrice, 'coinflow_*'],
});

/**
 * @typedef {{
 *   components: Components,
 *   extremes: Extremes,
 *   full: Cycle,
 *   fullV2: Cycle,
 *   local: Cycle,
 *   localV2: Cycle,
 *   cycle: Cycle,
 *   cycleV2: Cycle,
 * }} RarityMeter
 */
const _RarityMeter = _s({
  components: [_Components, 'price'],
  extremes: [_Extremes, '*'],
  full: [_Cycle, '*'],
  fullV2: [_Cycle, '*_v2'],
  local: [_Cycle, 'local_*'],
  localV2: [_Cycle, 'local_*_v2'],
  cycle: [_Cycle, 'cycle_*'],
  cycleV2: [_Cycle, 'cycle_*_v2'],
});

/**
 * @typedef {{
 *   isLong: SeriesPattern2<Boolean>,
 *   isShort: SeriesPattern2<Boolean>,
 *   phase: SeriesPattern3<CapitalSentimentPhase>,
 *   score: SeriesPattern3<Score>,
 * }} CapitalSentiment
 */
const _CapitalSentiment = _s({
  isLong: [_i2, '*_is_long'],
  isShort: [_i2, '*_is_short'],
  phase: [_i3, '*_phase'],
  score: [_i3, '*_score'],
});

/**
 * @typedef {{
 *   pct10: SeriesPattern2<?Dollars>,
 *   pct20: SeriesPattern2<?Dollars>,
 *   pct30: SeriesPattern2<?Dollars>,
 *   pct40: SeriesPattern2<?Dollars>,
 *   median: SeriesPattern2<?Dollars>,
 *   pct60: SeriesPattern2<?Dollars>,
 *   pct70: SeriesPattern2<?Dollars>,
 *   pct80: SeriesPattern2<?Dollars>,
 *   pct90: SeriesPattern2<?Dollars>,
 * }} Level
 */
const _Level = _s({
  pct10: [_i2, '*_pct10'],
  pct20: [_i2, '*_pct20'],
  pct30: [_i2, '*_pct30'],
  pct40: [_i2, '*_pct40'],
  median: [_i2, '*_median'],
  pct60: [_i2, '*_pct60'],
  pct70: [_i2, '*_pct70'],
  pct80: [_i2, '*_pct80'],
  pct90: [_i2, '*_pct90'],
});

/**
 * @template A
 * @typedef {{
 *   pct95: SeriesPattern2<A>,
 *   pct98: SeriesPattern2<A>,
 *   pct99: SeriesPattern2<A>,
 *   pct995: SeriesPattern2<A>,
 *   pct999: SeriesPattern2<A>,
 * }} Floor
 */
const _Floor = _s({
  pct95: [_i2, '*_pct95'],
  pct98: [_i2, '*_pct98'],
  pct99: [_i2, '*_pct99'],
  pct995: [_i2, '*_pct99_5'],
  pct999: [_i2, '*_pct99_9'],
});

/**
 * @typedef {{
 *   supplyInLossThreshold: Floor<?Percent>,
 *   floor: Floor<?Dollars>,
 *   level: Level,
 * }} BedrockCoinflow
 */
const _BedrockCoinflow = _s({
  supplyInLossThreshold: [_Floor, '*_supply_in_loss_threshold'],
  floor: [_Floor, '*_floor'],
  level: [_Level, '*_level'],
});

/**
 * @typedef {{
 *   raw: BedrockCoinflow,
 *   cointime: BedrockCoinflow,
 *   coinflow: BedrockCoinflow,
 * }} Bedrock
 */
const _Bedrock = _s({
  raw: [_BedrockCoinflow, '*_raw'],
  cointime: [_BedrockCoinflow, '*_cointime'],
  coinflow: [_BedrockCoinflow, '*_coinflow'],
});

/**
 * @typedef {{
 *   value: SeriesPattern2<?Float64>,
 *   vocddMedian1y: SeriesPattern21<?Float64>,
 *   hodlBank: SeriesPattern21<?Float64>,
 * }} ReserveRisk
 */
const _ReserveRisk = _s({
  value: [_i2, '*'],
  vocddMedian1y: [_i21, 'vocdd_median_1y'],
  hodlBank: [_i21, 'hodl_bank'],
});

/**
 * @typedef {{
 *   inflationRate: SeriesPattern2<?Percent>,
 *   txVelocityNative: SeriesPattern2<?Ratio64>,
 *   txVelocityFiat: SeriesPattern2<?Ratio64>,
 * }} Adjusted
 */
const _Adjusted = _s({
  inflationRate: [_i2, '*_inflation_rate'],
  txVelocityNative: [_i2, '*_tx_velocity_btc'],
  txVelocityFiat: [_i2, '*_tx_velocity_usd'],
});

/**
 * @typedef {{
 *   thermo: SeriesPattern2<?Dollars>,
 *   investor: SeriesPattern2<?Dollars>,
 *   vaulted: SeriesPattern2<?Dollars>,
 *   active: SeriesPattern2<?Dollars>,
 *   cointime: SeriesPattern2<?Dollars>,
 *   aviv: SeriesPattern2<?Ratio>,
 * }} Cap
 */
const _Cap = _s({
  thermo: [_i2, 'thermo_*'],
  investor: [_i2, 'investor_*'],
  vaulted: [_i2, 'vaulted_*'],
  active: [_i2, 'active_*'],
  cointime: [_i2, 'cointime_*'],
  aviv: [_i2, 'aviv_ratio'],
});

/**
 * @typedef {{
 *   share: SeriesPattern2<?Percent>,
 * }} MobileInLoss
 */
const _MobileInLoss = _s({
  share: [_i2, '*'],
});

/**
 * @typedef {{
 *   btc: SeriesPattern2<?Bitcoin>,
 *   usd: SeriesPattern2<?Dollars>,
 *   inLoss: MobileInLoss,
 * }} Mobile
 */
const _Mobile = _s({
  btc: [_i2, '*_mobile_supply'],
  usd: [_i2, '*_mobile_supply_usd'],
  inLoss: [_MobileInLoss, '*_coinflow_supply_in_loss_share'],
});

/**
 * @typedef {{
 *   btc: SeriesPattern2<?Bitcoin>,
 *   usd: SeriesPattern2<?Dollars>,
 *   inLoss: MobileInLoss,
 * }} Active
 */
const _Active = _s({
  btc: [_i2, 'active_*'],
  usd: [_i2, 'active_*_usd'],
  inLoss: [_MobileInLoss, 'cointime_*_in_loss_share'],
});

/**
 * @typedef {{
 *   btc: SeriesPattern2<?Bitcoin>,
 *   usd: SeriesPattern2<?Dollars>,
 *   inLoss: MobileInLoss,
 * }} AwakeSupply
 */
const _AwakeSupply = _s({
  btc: [_i2, '*'],
  usd: [_i2, '*_usd'],
  inLoss: [_MobileInLoss, '*_in_loss_share'],
});

/**
 * @typedef {{
 *   total: SeriesPattern2<?Percent>,
 *   inProfit: SeriesPattern2<?Percent>,
 *   inLoss: SeriesPattern2<?Percent>,
 * }} SupplyDensity
 */
const _SupplyDensity = _s({
  total: [_i2, '*_total'],
  inProfit: [_i2, '*_in_profit'],
  inLoss: [_i2, '*_in_loss'],
});

/**
 * @typedef {{
 *   block: SeriesPattern2<?Dollars>,
 *   ratio: SeriesPattern2<?Ratio>,
 * }} CapitalizedPrice
 */
const _CapitalizedPrice = _s({
  block: [_i2, '*'],
  ratio: [_i2, '*_ratio'],
});

/**
 * @typedef {{
 *   _1w: CapitalizedPrice,
 *   _8d: CapitalizedPrice,
 *   _12d: CapitalizedPrice,
 *   _13d: CapitalizedPrice,
 *   _21d: CapitalizedPrice,
 *   _26d: CapitalizedPrice,
 *   _1m: CapitalizedPrice,
 *   _34d: CapitalizedPrice,
 *   _55d: CapitalizedPrice,
 *   _89d: CapitalizedPrice,
 *   _144d: CapitalizedPrice,
 *   _200d: CapitalizedPrice,
 *   _1y: CapitalizedPrice,
 *   _2y: CapitalizedPrice,
 *   _200w: CapitalizedPrice,
 *   _4y: CapitalizedPrice,
 * }} Ema
 */
const _Ema = _s({
  _1w: [_CapitalizedPrice, '*_1w'],
  _8d: [_CapitalizedPrice, '*_8d'],
  _12d: [_CapitalizedPrice, '*_12d'],
  _13d: [_CapitalizedPrice, '*_13d'],
  _21d: [_CapitalizedPrice, '*_21d'],
  _26d: [_CapitalizedPrice, '*_26d'],
  _1m: [_CapitalizedPrice, '*_1m'],
  _34d: [_CapitalizedPrice, '*_34d'],
  _55d: [_CapitalizedPrice, '*_55d'],
  _89d: [_CapitalizedPrice, '*_89d'],
  _144d: [_CapitalizedPrice, '*_144d'],
  _200d: [_CapitalizedPrice, '*_200d'],
  _1y: [_CapitalizedPrice, '*_1y'],
  _2y: [_CapitalizedPrice, '*_2y'],
  _200w: [_CapitalizedPrice, '*_200w'],
  _4y: [_CapitalizedPrice, '*_4y'],
});

/**
 * @typedef {{
 *   _1w: CapitalizedPrice,
 *   _8d: CapitalizedPrice,
 *   _13d: CapitalizedPrice,
 *   _21d: CapitalizedPrice,
 *   _1m: CapitalizedPrice,
 *   _34d: CapitalizedPrice,
 *   _50d: CapitalizedPrice,
 *   _55d: CapitalizedPrice,
 *   _89d: CapitalizedPrice,
 *   _111d: CapitalizedPrice,
 *   _144d: CapitalizedPrice,
 *   _200d: Sma200d,
 *   _350d: Sma350d,
 *   _1y: CapitalizedPrice,
 *   _2y: CapitalizedPrice,
 *   _200w: CapitalizedPrice,
 *   _4y: CapitalizedPrice,
 * }} MarketSma
 */
const _MarketSma = _s({
  _1w: [_CapitalizedPrice, '*_1w'],
  _8d: [_CapitalizedPrice, '*_8d'],
  _13d: [_CapitalizedPrice, '*_13d'],
  _21d: [_CapitalizedPrice, '*_21d'],
  _1m: [_CapitalizedPrice, '*_1m'],
  _34d: [_CapitalizedPrice, '*_34d'],
  _50d: [_CapitalizedPrice, '*_50d'],
  _55d: [_CapitalizedPrice, '*_55d'],
  _89d: [_CapitalizedPrice, '*_89d'],
  _111d: [_CapitalizedPrice, '*_111d'],
  _144d: [_CapitalizedPrice, '*_144d'],
  _200d: [_Sma200d, '*_200d'],
  _350d: [_Sma350d, '*_350d'],
  _1y: [_CapitalizedPrice, '*_1y'],
  _2y: [_CapitalizedPrice, '*_2y'],
  _200w: [_CapitalizedPrice, '*_200w'],
  _4y: [_CapitalizedPrice, '*_4y'],
});

/**
 * @typedef {{
 *   vaulted: CapitalizedPrice,
 *   active: CapitalizedPrice,
 *   trueMarketMean: CapitalizedPrice,
 *   cointime: CapitalizedPrice,
 * }} CointimePrices
 */
const _CointimePrices = _s({
  vaulted: [_CapitalizedPrice, 'vaulted_*'],
  active: [_CapitalizedPrice, 'active_*'],
  trueMarketMean: [_CapitalizedPrice, 'true_market_mean'],
  cointime: [_CapitalizedPrice, 'cointime_*'],
});

/**
 * @typedef {{
 *   supply: AwakeSupply,
 *   cap: SeriesPattern2<?Dollars>,
 *   price: CapitalizedPrice,
 *   capitalizedPrice: CapitalizedPrice,
 * }} Awake
 */
const _Awake = _s({
  supply: [_AwakeSupply, '*_supply'],
  cap: [_i2, '*_cap'],
  price: [_CapitalizedPrice, '*_price'],
  capitalizedPrice: [_CapitalizedPrice, '*_capitalized_price'],
});

/**
 * @typedef {{
 *   pct5: SeriesPattern2<?Dollars>,
 *   pct10: SeriesPattern2<?Dollars>,
 *   pct15: SeriesPattern2<?Dollars>,
 *   pct20: SeriesPattern2<?Dollars>,
 *   pct25: SeriesPattern2<?Dollars>,
 *   pct30: SeriesPattern2<?Dollars>,
 *   pct35: SeriesPattern2<?Dollars>,
 *   pct40: SeriesPattern2<?Dollars>,
 *   pct45: SeriesPattern2<?Dollars>,
 *   median: SeriesPattern2<?Dollars>,
 *   pct55: SeriesPattern2<?Dollars>,
 *   pct60: SeriesPattern2<?Dollars>,
 *   pct65: SeriesPattern2<?Dollars>,
 *   pct70: SeriesPattern2<?Dollars>,
 *   pct75: SeriesPattern2<?Dollars>,
 *   pct80: SeriesPattern2<?Dollars>,
 *   pct85: SeriesPattern2<?Dollars>,
 *   pct90: SeriesPattern2<?Dollars>,
 *   pct95: SeriesPattern2<?Dollars>,
 * }} PerCoin
 */
const _PerCoin = _s({
  pct5: [_i2, '*_pct5'],
  pct10: [_i2, '*_pct10'],
  pct15: [_i2, '*_pct15'],
  pct20: [_i2, '*_pct20'],
  pct25: [_i2, '*_pct25'],
  pct30: [_i2, '*_pct30'],
  pct35: [_i2, '*_pct35'],
  pct40: [_i2, '*_pct40'],
  pct45: [_i2, '*_pct45'],
  median: [_i2, '*_median'],
  pct55: [_i2, '*_pct55'],
  pct60: [_i2, '*_pct60'],
  pct65: [_i2, '*_pct65'],
  pct70: [_i2, '*_pct70'],
  pct75: [_i2, '*_pct75'],
  pct80: [_i2, '*_pct80'],
  pct85: [_i2, '*_pct85'],
  pct90: [_i2, '*_pct90'],
  pct95: [_i2, '*_pct95'],
});

/**
 * @typedef {{
 *   inProfit: CostBasisInLoss,
 *   inLoss: CostBasisInLoss,
 *   min: SeriesPattern2<?Dollars>,
 *   max: SeriesPattern2<?Dollars>,
 *   perCoin: PerCoin,
 *   perDollar: PerCoin,
 *   supplyDensity: SeriesPattern2<?Percent>,
 * }} HoldersAllCostBasis
 */
const _HoldersAllCostBasis = _s({
  inProfit: [_CostBasisInLoss, '*_cost_basis_in_profit_per'],
  inLoss: [_CostBasisInLoss, '*_cost_basis_in_loss_per'],
  min: [_i2, '*_cost_basis_min'],
  max: [_i2, '*_cost_basis_max'],
  perCoin: [_PerCoin, '*_cost_basis_per_coin'],
  perDollar: [_PerCoin, '*_cost_basis_per_dollar'],
  supplyDensity: [_i2, '*_supply_density'],
});

/**
 * @typedef {{
 *   perCoin: PerCoin,
 *   perDollar: PerCoin,
 * }} UrpdAllCostBasis
 */
const _UrpdAllCostBasis = _s({
  perCoin: [_PerCoin, '*_coin'],
  perDollar: [_PerCoin, '*_dollar'],
});

/**
 * @typedef {{
 *   costBasis: UrpdAllCostBasis,
 *   capitalizedPrice: CapitalizedPrice,
 *   supplyDensity: SupplyDensity,
 * }} CoinflowUrpdLth
 */
const _CoinflowUrpdLth = _s({
  costBasis: [_UrpdAllCostBasis, '*_coinflow_cost_basis_per'],
  capitalizedPrice: [_CapitalizedPrice, 'coinflow_urpd_*_capitalized_price'],
  supplyDensity: [_SupplyDensity, 'coinflow_urpd_*_supply_density'],
});

/**
 * @typedef {{
 *   costBasis: UrpdAllCostBasis,
 *   capitalizedPrice: CapitalizedPrice,
 *   supplyDensity: SupplyDensity,
 * }} CointimeUrpdLth
 */
const _CointimeUrpdLth = _s({
  costBasis: [_UrpdAllCostBasis, '*_cointime_cost_basis_per'],
  capitalizedPrice: [_CapitalizedPrice, 'cointime_urpd_*_capitalized_price'],
  supplyDensity: [_SupplyDensity, 'cointime_urpd_*_supply_density'],
});

/**
 * @typedef {{
 *   costBasis: UrpdAllCostBasis,
 *   capitalizedPrice: CapitalizedPrice,
 *   supplyDensity: SupplyDensity,
 * }} UrpdAll
 */
const _UrpdAll = _s({
  costBasis: [_UrpdAllCostBasis, '*_cost_basis_per'],
  capitalizedPrice: [_CapitalizedPrice, '*_urpd_all_capitalized_price'],
  supplyDensity: [_SupplyDensity, '*_urpd_all_supply_density'],
});

/**
 * @template A
 * @typedef {{
 *   all: UrpdAll,
 *   sth: A,
 *   lth: A,
 *   under4m: A,
 *   under6m: A,
 *   over4m: A,
 *   over6m: A,
 * }} CoinflowUrpd
 */
/** @type {_Make} */
const _CoinflowUrpd = (c, b, f0) => _n(c, b, {
  all: [_UrpdAll, '*'],
  sth: [f0, 'sth'],
  lth: [f0, 'lth'],
  under4m: [f0, 'under_4m'],
  under6m: [f0, 'under_6m'],
  over4m: [f0, 'over_4m'],
  over6m: [f0, 'over_6m'],
});

/**
 * @template A
 * @typedef {{
 *   under1h: SeriesPattern2<A>,
 *   _1hTo1d: SeriesPattern2<A>,
 *   _1dTo1w: SeriesPattern2<A>,
 *   _1wTo1m: SeriesPattern2<A>,
 *   _1mTo2m: SeriesPattern2<A>,
 *   _2mTo3m: SeriesPattern2<A>,
 *   _3mTo4m: SeriesPattern2<A>,
 *   _4mTo5m: SeriesPattern2<A>,
 *   _5mTo6m: SeriesPattern2<A>,
 *   _6mTo9m: SeriesPattern2<A>,
 *   _9mTo1y: SeriesPattern2<A>,
 *   _1yTo18m: SeriesPattern2<A>,
 *   _18mTo2y: SeriesPattern2<A>,
 *   _2yTo3y: SeriesPattern2<A>,
 *   _3yTo4y: SeriesPattern2<A>,
 *   _4yTo5y: SeriesPattern2<A>,
 *   _5yTo6y: SeriesPattern2<A>,
 *   _6yTo7y: SeriesPattern2<A>,
 *   _7yTo8y: SeriesPattern2<A>,
 *   _8yTo10y: SeriesPattern2<A>,
 *   _10yTo12y: SeriesPattern2<A>,
 *   _12yTo15y: SeriesPattern2<A>,
 *   over15y: SeriesPattern2<A>,
 * }} SpendingRate
 */
const _SpendingRate = _s({
  under1h: [_i2, 'utxos_under_1h_*'],
  _1hTo1d: [_i2, 'utxos_1h_to_1d_*'],
  _1dTo1w: [_i2, 'utxos_1d_to_1w_*'],
  _1wTo1m: [_i2, 'utxos_1w_to_1m_*'],
  _1mTo2m: [_i2, 'utxos_1m_to_2m_*'],
  _2mTo3m: [_i2, 'utxos_2m_to_3m_*'],
  _3mTo4m: [_i2, 'utxos_3m_to_4m_*'],
  _4mTo5m: [_i2, 'utxos_4m_to_5m_*'],
  _5mTo6m: [_i2, 'utxos_5m_to_6m_*'],
  _6mTo9m: [_i2, 'utxos_6m_to_9m_*'],
  _9mTo1y: [_i2, 'utxos_9m_to_1y_*'],
  _1yTo18m: [_i2, 'utxos_1y_to_18m_*'],
  _18mTo2y: [_i2, 'utxos_18m_to_2y_*'],
  _2yTo3y: [_i2, 'utxos_2y_to_3y_*'],
  _3yTo4y: [_i2, 'utxos_3y_to_4y_*'],
  _4yTo5y: [_i2, 'utxos_4y_to_5y_*'],
  _5yTo6y: [_i2, 'utxos_5y_to_6y_*'],
  _6yTo7y: [_i2, 'utxos_6y_to_7y_*'],
  _7yTo8y: [_i2, 'utxos_7y_to_8y_*'],
  _8yTo10y: [_i2, 'utxos_8y_to_10y_*'],
  _10yTo12y: [_i2, 'utxos_10y_to_12y_*'],
  _12yTo15y: [_i2, 'utxos_12y_to_15y_*'],
  over15y: [_i2, 'utxos_over_15y_*'],
});

/**
 * @typedef {{
 *   age: SpendingRate<?Dollars>,
 * }} RealizedPrice
 */
const _RealizedPrice = _s({
  age: [_SpendingRate, '*'],
});

/**
 * @typedef {{
 *   under1h: SeriesPattern2<?Float64>,
 *   _1hTo1d: SeriesPattern2<?Float64>,
 *   _1dTo1w: SeriesPattern2<?Float64>,
 *   _1wTo1m: SeriesPattern2<?Float64>,
 *   _1mTo2m: SeriesPattern2<?Float64>,
 *   _2mTo3m: SeriesPattern2<?Float64>,
 *   _3mTo4m: SeriesPattern2<?Float64>,
 *   _4mTo5m: SeriesPattern2<?Float64>,
 *   _5mTo6m: SeriesPattern2<?Float64>,
 *   _6mTo9m: SeriesPattern2<?Float64>,
 *   _9mTo1y: SeriesPattern2<?Float64>,
 *   _1yTo18m: SeriesPattern2<?Float64>,
 *   _18mTo2y: SeriesPattern2<?Float64>,
 *   _2yTo3y: SeriesPattern2<?Float64>,
 *   _3yTo4y: SeriesPattern2<?Float64>,
 *   _4yTo5y: SeriesPattern2<?Float64>,
 *   _5yTo6y: SeriesPattern2<?Float64>,
 *   _6yTo7y: SeriesPattern2<?Float64>,
 *   _7yTo8y: SeriesPattern2<?Float64>,
 *   _8yTo10y: SeriesPattern2<?Float64>,
 *   _10yTo12y: SeriesPattern2<?Float64>,
 *   _12yTo15y: SeriesPattern2<?Float64>,
 *   over15y: SeriesPattern2<?Float64>,
 *   mobility: SpendingRate<?Ratio64>,
 * }} SpendingExposure
 */
const _SpendingExposure = _s({
  under1h: [_i2, 'utxos_under_1h_*_spending_exposure'],
  _1hTo1d: [_i2, 'utxos_1h_to_1d_*_spending_exposure'],
  _1dTo1w: [_i2, 'utxos_1d_to_1w_*_spending_exposure'],
  _1wTo1m: [_i2, 'utxos_1w_to_1m_*_spending_exposure'],
  _1mTo2m: [_i2, 'utxos_1m_to_2m_*_spending_exposure'],
  _2mTo3m: [_i2, 'utxos_2m_to_3m_*_spending_exposure'],
  _3mTo4m: [_i2, 'utxos_3m_to_4m_*_spending_exposure'],
  _4mTo5m: [_i2, 'utxos_4m_to_5m_*_spending_exposure'],
  _5mTo6m: [_i2, 'utxos_5m_to_6m_*_spending_exposure'],
  _6mTo9m: [_i2, 'utxos_6m_to_9m_*_spending_exposure'],
  _9mTo1y: [_i2, 'utxos_9m_to_1y_*_spending_exposure'],
  _1yTo18m: [_i2, 'utxos_1y_to_18m_*_spending_exposure'],
  _18mTo2y: [_i2, 'utxos_18m_to_2y_*_spending_exposure'],
  _2yTo3y: [_i2, 'utxos_2y_to_3y_*_spending_exposure'],
  _3yTo4y: [_i2, 'utxos_3y_to_4y_*_spending_exposure'],
  _4yTo5y: [_i2, 'utxos_4y_to_5y_*_spending_exposure'],
  _5yTo6y: [_i2, 'utxos_5y_to_6y_*_spending_exposure'],
  _6yTo7y: [_i2, 'utxos_6y_to_7y_*_spending_exposure'],
  _7yTo8y: [_i2, 'utxos_7y_to_8y_*_spending_exposure'],
  _8yTo10y: [_i2, 'utxos_8y_to_10y_*_spending_exposure'],
  _10yTo12y: [_i2, 'utxos_10y_to_12y_*_spending_exposure'],
  _12yTo15y: [_i2, 'utxos_12y_to_15y_*_spending_exposure'],
  over15y: [_i2, 'utxos_over_15y_*_spending_exposure'],
  mobility: [_SpendingRate, '*_mobility'],
});

/**
 * @typedef {{
 *   wakefulness: SpendingRate<?Ratio64>,
 *   dormancy: SpendingRate<?Ratio64>,
 *   wakefulnessToDormancy: SpendingRate<?Ratio64>,
 * }} AgeRangeActivity
 */
const _AgeRangeActivity = _s({
  wakefulness: [_SpendingRate, '*_wakefulness'],
  dormancy: [_SpendingRate, '*_dormancy'],
  wakefulnessToDormancy: [_SpendingRate, '*_wakefulness_to_dormancy'],
});

/**
 * @template A
 * @typedef {{
 *   min: SeriesPattern2<A>,
 *   max: SeriesPattern2<A>,
 *   pct10: SeriesPattern2<A>,
 *   pct25: SeriesPattern2<A>,
 *   median: SeriesPattern2<A>,
 *   pct75: SeriesPattern2<A>,
 *   pct90: SeriesPattern2<A>,
 * }} EffectiveFeeRate6b
 */
const _EffectiveFeeRate6b = _s({
  min: [_i2, '*_min'],
  max: [_i2, '*_max'],
  pct10: [_i2, '*_pct10'],
  pct25: [_i2, '*_pct25'],
  median: [_i2, '*_median'],
  pct75: [_i2, '*_pct75'],
  pct90: [_i2, '*_pct90'],
});

/**
 * @typedef {{
 *   block: EffectiveFeeRate6b<Weight>,
 *   _6b: EffectiveFeeRate6b<Weight>,
 * }} SizeWeight
 */
const _SizeWeight = _s({
  block: [_EffectiveFeeRate6b, '*'],
  _6b: [_EffectiveFeeRate6b, '*_6b'],
});

/**
 * @typedef {{
 *   min: SeriesPattern21<VSize>,
 *   max: SeriesPattern21<VSize>,
 *   pct10: SeriesPattern21<VSize>,
 *   pct25: SeriesPattern21<VSize>,
 *   median: SeriesPattern21<VSize>,
 *   pct75: SeriesPattern21<VSize>,
 *   pct90: SeriesPattern21<VSize>,
 * }} Vsize6b
 */
const _Vsize6b = _s({
  min: [_i21, '*_min'],
  max: [_i21, '*_max'],
  pct10: [_i21, '*_pct10'],
  pct25: [_i21, '*_pct25'],
  median: [_i21, '*_median'],
  pct75: [_i21, '*_pct75'],
  pct90: [_i21, '*_pct90'],
});

/**
 * @template A, B
 * @typedef {{
 *   txIndex: SeriesPattern22<A>,
 *   block: B,
 *   _6b: B,
 * }} EffectiveFeeRate
 */
/** @type {_Make} */
const _EffectiveFeeRate = (c, b, f0) => _n(c, b, {
  txIndex: [_i22, '*'],
  block: [f0, '*'],
  _6b: [f0, '*_6b'],
});

/**
 * @typedef {{
 *   vsize: EffectiveFeeRate<VSize, Vsize6b>,
 *   weight: SizeWeight,
 * }} TransactionsSize
 */
const _TransactionsSize = _s({
  vsize: [(c, b) => _EffectiveFeeRate(c, b, _Vsize6b), '*_vsize'],
  weight: [_SizeWeight, '*_weight'],
});

/**
 * @typedef {{
 *   block: SeriesPattern2<?Float32>,
 *   atl: SeriesPattern2<?Float32>,
 *   rebound: SeriesPattern2<?Percent>,
 * }} HashratePrice
 */
const _HashratePrice = _s({
  block: [_i2, '*'],
  atl: [_i2, '*_atl'],
  rebound: [_i2, '*_rebound'],
});

/**
 * @typedef {{
 *   _1w: SeriesPattern2<?Hashrate>,
 *   _1m: SeriesPattern2<?Hashrate>,
 *   _2m: SeriesPattern2<?Hashrate>,
 *   _1y: SeriesPattern2<?Hashrate>,
 * }} RateSma
 */
const _RateSma = _s({
  _1w: [_i2, '*_1w'],
  _1m: [_i2, '*_1m'],
  _2m: [_i2, '*_2m'],
  _1y: [_i2, '*_1y'],
});

/**
 * @typedef {{
 *   block: SeriesPattern2<?Hashrate>,
 *   sma: RateSma,
 *   ath: SeriesPattern2<?Hashrate>,
 *   drawdown: SeriesPattern2<?Percent>,
 * }} HashrateRate
 */
const _HashrateRate = _s({
  block: [_i2, '*'],
  sma: [_RateSma, '*_sma'],
  ath: [_i2, '*_ath'],
  drawdown: [_i2, '*_drawdown'],
});

/**
 * @typedef {{
 *   rate: HashrateRate,
 *   price: HashratePrice,
 *   value: HashratePrice,
 * }} MiningHashrate
 */
const _MiningHashrate = _s({
  rate: [_HashrateRate, '*_rate'],
  price: [_HashratePrice, '*_price'],
  value: [_HashratePrice, '*_value'],
});

/**
 * @typedef {{
 *   cumulative: SeriesPattern2<?Percent>,
 *   _24h: SeriesPattern2<?Percent>,
 *   _1w: SeriesPattern2<?Percent>,
 *   _1m: SeriesPattern2<?Percent>,
 *   _1y: SeriesPattern2<?Percent>,
 * }} Share
 */
const _Share = _s({
  cumulative: [_i2, '*'],
  _24h: [_i2, '*_24h'],
  _1w: [_i2, '*_1w'],
  _1m: [_i2, '*_1m'],
  _1y: [_i2, '*_1y'],
});

/**
 * @typedef {{
 *   btc: SeriesPattern2<?Bitcoin>,
 *   usd: SeriesPattern2<?Dollars>,
 * }} HodledOrLost
 */
const _HodledOrLost = _s({
  btc: [_i2, '*'],
  usd: [_i2, '*_usd'],
});

/**
 * @typedef {{
 *   all: HodledOrLost,
 *   p2pk65: HodledOrLost,
 *   p2pk33: HodledOrLost,
 *   p2pkh: HodledOrLost,
 *   p2sh: HodledOrLost,
 *   p2wpkh: HodledOrLost,
 *   p2wsh: HodledOrLost,
 *   p2tr: HodledOrLost,
 *   p2a: HodledOrLost,
 *   share: AddressesEmpty<?Percent>,
 * }} ExposedSupply
 */
const _ExposedSupply = _s({
  all: [_HodledOrLost, '*'],
  p2pk65: [_HodledOrLost, 'p2pk65_*'],
  p2pk33: [_HodledOrLost, 'p2pk33_*'],
  p2pkh: [_HodledOrLost, 'p2pkh_*'],
  p2sh: [_HodledOrLost, 'p2sh_*'],
  p2wpkh: [_HodledOrLost, 'p2wpkh_*'],
  p2wsh: [_HodledOrLost, 'p2wsh_*'],
  p2tr: [_HodledOrLost, 'p2tr_*'],
  p2a: [_HodledOrLost, 'p2a_*'],
  share: [_AddressesEmpty, '*_share'],
});

/**
 * @typedef {{
 *   count: ExposedCount,
 *   supply: ExposedSupply,
 * }} Exposed
 */
const _Exposed = _s({
  count: [_ExposedCount, '*_count'],
  supply: [_ExposedSupply, '*_supply'],
});

/**
 * @typedef {{
 *   _0sats: HodledOrLost,
 *   _1satTo10sats: HodledOrLost,
 *   _10satsTo100sats: HodledOrLost,
 *   _100satsTo1kSats: HodledOrLost,
 *   _1kSatsTo10kSats: HodledOrLost,
 *   _10kSatsTo100kSats: HodledOrLost,
 *   _100kSatsTo1mSats: HodledOrLost,
 *   _1mSatsTo10mSats: HodledOrLost,
 *   _10mSatsTo1btc: HodledOrLost,
 *   _1btcTo10btc: HodledOrLost,
 *   _10btcTo100btc: HodledOrLost,
 *   _100btcTo1kBtc: HodledOrLost,
 *   _1kBtcTo10kBtc: HodledOrLost,
 *   _10kBtcTo100kBtc: HodledOrLost,
 *   over100kBtc: HodledOrLost,
 * }} TotalUtxoAmount
 */
const _TotalUtxoAmount = _s({
  _0sats: [_HodledOrLost, '*_0sats_supply'],
  _1satTo10sats: [_HodledOrLost, '*_1sat_to_10sats_supply'],
  _10satsTo100sats: [_HodledOrLost, '*_10sats_to_100sats_supply'],
  _100satsTo1kSats: [_HodledOrLost, '*_100sats_to_1k_sats_supply'],
  _1kSatsTo10kSats: [_HodledOrLost, '*_1k_sats_to_10k_sats_supply'],
  _10kSatsTo100kSats: [_HodledOrLost, '*_10k_sats_to_100k_sats_supply'],
  _100kSatsTo1mSats: [_HodledOrLost, '*_100k_sats_to_1m_sats_supply'],
  _1mSatsTo10mSats: [_HodledOrLost, '*_1m_sats_to_10m_sats_supply'],
  _10mSatsTo1btc: [_HodledOrLost, '*_10m_sats_to_1btc_supply'],
  _1btcTo10btc: [_HodledOrLost, '*_1btc_to_10btc_supply'],
  _10btcTo100btc: [_HodledOrLost, '*_10btc_to_100btc_supply'],
  _100btcTo1kBtc: [_HodledOrLost, '*_100btc_to_1k_btc_supply'],
  _1kBtcTo10kBtc: [_HodledOrLost, '*_1k_btc_to_10k_btc_supply'],
  _10kBtcTo100kBtc: [_HodledOrLost, '*_10k_btc_to_100k_btc_supply'],
  over100kBtc: [_HodledOrLost, '*_over_100k_btc_supply'],
});

/**
 * @typedef {{
 *   mobile: Mobile,
 *   immobile: HodledOrLost,
 * }} CoinflowSupply
 */
const _CoinflowSupply = _s({
  mobile: [_Mobile, '*'],
  immobile: [_HodledOrLost, '*_immobile_supply'],
});

/**
 * @typedef {{
 *   supply: CoinflowSupply,
 *   cap: SeriesPattern2<?Dollars>,
 *   price: CapitalizedPrice,
 *   capitalizedPrice: CapitalizedPrice,
 * }} CoinflowLth
 */
const _CoinflowLth = _s({
  supply: [_CoinflowSupply, '*'],
  cap: [_i2, '*_coinflow_cap'],
  price: [_CapitalizedPrice, '*_coinflow_price'],
  capitalizedPrice: [_CapitalizedPrice, '*_coinflow_capitalized_price'],
});

/**
 * @typedef {{
 *   vaulted: HodledOrLost,
 *   active: Active,
 * }} CointimeSupply
 */
const _CointimeSupply = _s({
  vaulted: [_HodledOrLost, 'vaulted_*'],
  active: [_Active, '*'],
});

/**
 * @typedef {{
 *   supply: HodledOrLost,
 * }} Dormant
 */
const _Dormant = _s({
  supply: [_HodledOrLost, '*'],
});

/**
 * @typedef {{
 *   awake: Awake,
 *   dormant: Dormant,
 * }} CointimeLth
 */
const _CointimeLth = _s({
  awake: [_Awake, '*_awake'],
  dormant: [_Dormant, '*_dormant_supply'],
});

/**
 * @typedef {{
 *   _24h: HodledOrLost,
 *   _1w: HodledOrLost,
 *   _1m: HodledOrLost,
 *   _1y: HodledOrLost,
 * }} Sum
 */
const _Sum = _s({
  _24h: [_HodledOrLost, '*_24h'],
  _1w: [_HodledOrLost, '*_1w'],
  _1m: [_HodledOrLost, '*_1m'],
  _1y: [_HodledOrLost, '*_1y'],
});

/**
 * @typedef {{
 *   btc: SeriesPattern21<?Bitcoin>,
 *   usd: SeriesPattern21<?Dollars>,
 * }} Block
 */
const _Block = _s({
  btc: [_i21, '*'],
  usd: [_i21, '*_usd'],
});

/**
 * @typedef {{
 *   block: Block,
 *   cumulative: HodledOrLost,
 *   sum: Sum,
 *   share: Share,
 * }} OpReturnFees
 */
const _OpReturnFees = _s({
  block: [_Block, '*_fees'],
  cumulative: [_HodledOrLost, '*_fees_cumulative'],
  sum: [_Sum, '*_fees_sum'],
  share: [_Share, '*_fee_share'],
});

/**
 * @typedef {{
 *   block: Block,
 *   cumulative: HodledOrLost,
 * }} OpReturnValue
 */
const _OpReturnValue = _s({
  block: [_Block, '*'],
  cumulative: [_HodledOrLost, '*_cumulative'],
});

/**
 * @typedef {{
 *   block: Block,
 *   cumulative: HodledOrLost,
 *   sum: Sum,
 *   share: Share,
 * }} Subsidy
 */
const _Subsidy = _s({
  block: [_Block, '*'],
  cumulative: [_HodledOrLost, '*_cumulative'],
  sum: [_Sum, '*_sum'],
  share: [_Share, '*_share'],
});

/**
 * @typedef {{
 *   block: Block,
 *   cumulative: HodledOrLost,
 *   sum: Sum,
 * }} TransferVolume0sats
 */
const _TransferVolume0sats = _s({
  block: [_Block, '*'],
  cumulative: [_HodledOrLost, '*_cumulative'],
  sum: [_Sum, '*_sum'],
});

/**
 * @typedef {{
 *   block: Block,
 *   cumulative: HodledOrLost,
 *   sum: Sum,
 *   inProfit: TransferVolume0sats,
 *   inLoss: TransferVolume0sats,
 * }} DiscountActivityTransferVolume
 */
const _DiscountActivityTransferVolume = _s({
  block: [_Block, '*'],
  cumulative: [_HodledOrLost, '*_cumulative'],
  sum: [_Sum, '*_sum'],
  inProfit: [_TransferVolume0sats, '*_in_profit'],
  inLoss: [_TransferVolume0sats, '*_in_loss'],
});

/**
 * @typedef {{
 *   epoch: SeriesPattern2<Halving>,
 *   blocksToHalving: SeriesPattern2<Count>,
 *   daysToHalving: SeriesPattern2<?Days>,
 * }} BlocksHalving
 */
const _BlocksHalving = _s({
  epoch: [_i2, '*_epoch'],
  blocksToHalving: [_i2, 'blocks_to_*'],
  daysToHalving: [_i2, 'days_to_*'],
});

/**
 * @typedef {{
 *   value: SeriesPattern2<?Difficulty>,
 *   hashrate: SeriesPattern2<?Hashrate>,
 *   adjustment: SeriesPattern2<?Percent>,
 *   epoch: SeriesPattern2<Epoch>,
 *   blocksToRetarget: SeriesPattern2<Count>,
 *   daysToRetarget: SeriesPattern2<?Days>,
 * }} BlocksDifficulty
 */
const _BlocksDifficulty = _s({
  value: [_i2, '*'],
  hashrate: [_i2, '*_hashrate'],
  adjustment: [_i2, '*_adjustment'],
  epoch: [_i2, '*_epoch'],
  blocksToRetarget: [_i2, 'blocks_to_retarget'],
  daysToRetarget: [_i2, 'days_to_retarget'],
});

/**
 * @typedef {{
 *   _1h: SeriesPattern21<Height>,
 *   _24h: SeriesPattern21<Height>,
 *   _3d: SeriesPattern21<Height>,
 *   _1w: SeriesPattern21<Height>,
 *   _8d: SeriesPattern21<Height>,
 *   _9d: SeriesPattern21<Height>,
 *   _12d: SeriesPattern21<Height>,
 *   _13d: SeriesPattern21<Height>,
 *   _2w: SeriesPattern21<Height>,
 *   _21d: SeriesPattern21<Height>,
 *   _26d: SeriesPattern21<Height>,
 *   _1m: SeriesPattern21<Height>,
 *   _34d: SeriesPattern21<Height>,
 *   _50d: SeriesPattern21<Height>,
 *   _55d: SeriesPattern21<Height>,
 *   _2m: SeriesPattern21<Height>,
 *   _9w: SeriesPattern21<Height>,
 *   _12w: SeriesPattern21<Height>,
 *   _89d: SeriesPattern21<Height>,
 *   _3m: SeriesPattern21<Height>,
 *   _14w: SeriesPattern21<Height>,
 *   _111d: SeriesPattern21<Height>,
 *   _144d: SeriesPattern21<Height>,
 *   _6m: SeriesPattern21<Height>,
 *   _26w: SeriesPattern21<Height>,
 *   _200d: SeriesPattern21<Height>,
 *   _9m: SeriesPattern21<Height>,
 *   _350d: SeriesPattern21<Height>,
 *   _12m: SeriesPattern21<Height>,
 *   _1y: SeriesPattern21<Height>,
 *   _14m: SeriesPattern21<Height>,
 *   _2y: SeriesPattern21<Height>,
 *   _26m: SeriesPattern21<Height>,
 *   _3y: SeriesPattern21<Height>,
 *   _200w: SeriesPattern21<Height>,
 *   _4y: SeriesPattern21<Height>,
 *   _5y: SeriesPattern21<Height>,
 *   _6y: SeriesPattern21<Height>,
 *   _8y: SeriesPattern21<Height>,
 *   _9y: SeriesPattern21<Height>,
 *   _10y: SeriesPattern21<Height>,
 *   _12y: SeriesPattern21<Height>,
 *   _14y: SeriesPattern21<Height>,
 *   _26y: SeriesPattern21<Height>,
 * }} BlocksLookback
 */
const _BlocksLookback = _s({
  _1h: [_i21, '*_1h_ago'],
  _24h: [_i21, '*_24h_ago'],
  _3d: [_i21, '*_3d_ago'],
  _1w: [_i21, '*_1w_ago'],
  _8d: [_i21, '*_8d_ago'],
  _9d: [_i21, '*_9d_ago'],
  _12d: [_i21, '*_12d_ago'],
  _13d: [_i21, '*_13d_ago'],
  _2w: [_i21, '*_2w_ago'],
  _21d: [_i21, '*_21d_ago'],
  _26d: [_i21, '*_26d_ago'],
  _1m: [_i21, '*_1m_ago'],
  _34d: [_i21, '*_34d_ago'],
  _50d: [_i21, '*_50d_ago'],
  _55d: [_i21, '*_55d_ago'],
  _2m: [_i21, '*_2m_ago'],
  _9w: [_i21, '*_9w_ago'],
  _12w: [_i21, '*_12w_ago'],
  _89d: [_i21, '*_89d_ago'],
  _3m: [_i21, '*_3m_ago'],
  _14w: [_i21, '*_14w_ago'],
  _111d: [_i21, '*_111d_ago'],
  _144d: [_i21, '*_144d_ago'],
  _6m: [_i21, '*_6m_ago'],
  _26w: [_i21, '*_26w_ago'],
  _200d: [_i21, '*_200d_ago'],
  _9m: [_i21, '*_9m_ago'],
  _350d: [_i21, '*_350d_ago'],
  _12m: [_i21, '*_12m_ago'],
  _1y: [_i21, '*_1y_ago'],
  _14m: [_i21, '*_14m_ago'],
  _2y: [_i21, '*_2y_ago'],
  _26m: [_i21, '*_26m_ago'],
  _3y: [_i21, '*_3y_ago'],
  _200w: [_i21, '*_200w_ago'],
  _4y: [_i21, '*_4y_ago'],
  _5y: [_i21, '*_5y_ago'],
  _6y: [_i21, '*_6y_ago'],
  _8y: [_i21, '*_8y_ago'],
  _9y: [_i21, '*_9y_ago'],
  _10y: [_i21, '*_10y_ago'],
  _12y: [_i21, '*_12y_ago'],
  _14y: [_i21, '*_14y_ago'],
  _26y: [_i21, '*_26y_ago'],
});

/**
 * @template A
 * @typedef {{
 *   _24h: SeriesPattern2<A>,
 *   _1w: SeriesPattern2<A>,
 *   _1m: SeriesPattern2<A>,
 *   _1y: SeriesPattern2<A>,
 * }} InputsPerSecond
 */
const _InputsPerSecond = _s({
  _24h: [_i2, '*_24h'],
  _1w: [_i2, '*_1w'],
  _1m: [_i2, '*_1m'],
  _1y: [_i2, '*_1y'],
});

/**
 * @typedef {{
 *   block: SeriesPattern21<Bytes>,
 *   cumulative: SeriesPattern2<Bytes>,
 *   sum: InputsPerSecond<Bytes>,
 *   share: SeriesPattern2<?Percent>,
 *   chainShare: SeriesPattern2<?Percent>,
 * }} MultipleDataBytes
 */
const _MultipleDataBytes = _s({
  block: [_i21, '*_data_bytes'],
  cumulative: [_i2, '*_data_bytes_cumulative'],
  sum: [_InputsPerSecond, '*_data_bytes_sum'],
  share: [_i2, '*_data_share'],
  chainShare: [_i2, '*_chain_share'],
});

/**
 * @typedef {{
 *   block: SeriesPattern21<Bytes>,
 *   cumulative: SeriesPattern2<Bytes>,
 *   sum: InputsPerSecond<Bytes>,
 *   chainShare: SeriesPattern2<?Percent>,
 * }} OpReturnDataBytes
 */
const _OpReturnDataBytes = _s({
  block: [_i21, '*_data_bytes'],
  cumulative: [_i2, '*_data_bytes_cumulative'],
  sum: [_InputsPerSecond, '*_data_bytes_sum'],
  chainShare: [_i2, '*_chain_share'],
});

/**
 * @template A
 * @typedef {{
 *   absolute: InputsPerSecond<A>,
 *   rate: AllRate,
 * }} DeltaAll
 */
const _DeltaAll = _s({
  absolute: [_InputsPerSecond, '*'],
  rate: [_AllRate, '*'],
});

/**
 * @typedef {{
 *   total: HodledOrLost,
 *   delta: DeltaAll<?Bitcoin>,
 *   dominance: SeriesPattern2<?Percent>,
 * }} Supply0sats
 */
const _Supply0sats = _s({
  total: [_HodledOrLost, '*'],
  delta: [_DeltaAll, '*_delta'],
  dominance: [_i2, '*_dominance'],
});

/**
 * @typedef {{
 *   total: HodledOrLost,
 *   delta: DeltaAll<?Bitcoin>,
 *   dominance: SeriesPattern2<?Percent>,
 *   inProfit: HodledOrLost,
 *   inLoss: HodledOrLost,
 * }} DiscountSupply
 */
const _DiscountSupply = _s({
  total: [_HodledOrLost, '*'],
  delta: [_DeltaAll, '*_delta'],
  dominance: [_i2, '*_dominance'],
  inProfit: [_HodledOrLost, '*_in_profit'],
  inLoss: [_HodledOrLost, '*_in_loss'],
});

/**
 * @typedef {{
 *   total: HodledOrLost,
 *   inProfit: HodledOrLost,
 *   inLoss: HodledOrLost,
 *   delta: DeltaAll<?Bitcoin>,
 * }} LthSupply
 */
const _LthSupply = _s({
  total: [_HodledOrLost, '*'],
  inProfit: [_HodledOrLost, '*_in_profit'],
  inLoss: [_HodledOrLost, '*_in_loss'],
  delta: [_DeltaAll, '*_delta'],
});

/**
 * @typedef {{
 *   total: AllSupplyTotal,
 *   inProfit: HodledOrLost,
 *   inLoss: HodledOrLost,
 *   delta: DeltaAll<?Bitcoin>,
 * }} AllSupply
 */
const _AllSupply = _s({
  total: [_AllSupplyTotal, 'circulating_*'],
  inProfit: [_HodledOrLost, '*_in_profit'],
  inLoss: [_HodledOrLost, '*_in_loss'],
  delta: [_DeltaAll, '*_delta'],
});

/**
 * @typedef {{
 *   block: SeriesPattern21<?Dollars>,
 *   cumulative: SeriesPattern2<?Dollars>,
 *   sum: InputsPerSecond<?Dollars>,
 *   delta: DeltaAll<?Dollars>,
 * }} NetPnl
 */
const _NetPnl = _s({
  block: [_i21, '*'],
  cumulative: [_i2, '*_cumulative'],
  sum: [_InputsPerSecond, '*_sum'],
  delta: [_DeltaAll, '*_delta'],
});

/**
 * @template A, B
 * @typedef {{
 *   block: SeriesPattern2<A>,
 *   delta: DeltaAll<B>,
 * }} MarketCap
 */
const _MarketCap = _s({
  block: [_i2, '*'],
  delta: [_DeltaAll, '*_delta'],
});

/**
 * @typedef {{
 *   circulating: SeriesPattern2<?Bitcoin>,
 *   burned: OpReturnValue,
 *   inflationRate: SeriesPattern2<?Percent>,
 *   velocity: Velocity,
 *   marketCap: MarketCap<?Dollars, ?Dollars>,
 *   marketMinusRealizedCapGrowthRate: InputsPerSecond<?Percent>,
 *   hodledOrLost: HodledOrLost,
 * }} Supply
 */
const _Supply = _s({
  circulating: [_i2, 'circulating_*'],
  burned: [_OpReturnValue, 'unspendable_*'],
  inflationRate: [_i2, 'inflation_rate'],
  velocity: [_Velocity, 'velocity'],
  marketCap: [_MarketCap, 'market_cap'],
  marketMinusRealizedCapGrowthRate: [_InputsPerSecond, 'market_minus_realized_cap_growth_rate'],
  hodledOrLost: [_HodledOrLost, 'hodled_or_lost_*'],
});

/**
 * @typedef {{
 *   avg: InputsPerSecond<?Percent>,
 *   sd: InputsPerSecond<?Percent>,
 * }} Daily
 */
const _Daily = _s({
  avg: [_InputsPerSecond, '*_avg'],
  sd: [_InputsPerSecond, '*_sd'],
});

/**
 * @typedef {{
 *   periods: MarketLookback<?Percent>,
 *   cagr: Cagr,
 *   daily: Daily,
 * }} Returns
 */
const _Returns = _s({
  periods: [_MarketLookback, '*_return'],
  cagr: [_Cagr, '*_cagr'],
  daily: [_Daily, '*_return_24h'],
});

/**
 * @typedef {{
 *   ath: Ath,
 *   lookback: MarketLookback<?Dollars>,
 *   returns: Returns,
 *   volatility: InputsPerSecond<?Percent>,
 *   range: Range,
 *   sma: MarketSma,
 *   ema: Ema,
 *   rsi: Macd<Rsi1m>,
 *   piCycle: SeriesPattern2<?Ratio>,
 *   macd: Macd<Macd1m>,
 * }} Market
 */
const _Market = _s({
  ath: [_Ath, '*'],
  lookback: [_MarketLookback, '*_past'],
  returns: [_Returns, '*'],
  volatility: [_InputsPerSecond, '*_volatility'],
  range: [_Range, '*'],
  sma: [_MarketSma, '*_sma'],
  ema: [_Ema, '*_ema'],
  rsi: [(c, b) => _Macd(c, b, _Rsi1m), '24h'],
  piCycle: [_i2, 'pi_cycle'],
  macd: [(c, b) => _Macd(c, b, _Macd1m), '24h'],
});

/**
 * @typedef {{
 *   value: TransferVolume0sats,
 *   txPerSecond: InputsPerSecond<?PerSecond>,
 * }} Volume
 */
const _Volume = _s({
  value: [_TransferVolume0sats, '*_volume'],
  txPerSecond: [_InputsPerSecond, '*_per_second'],
});

/**
 * @typedef {{
 *   block: Block,
 *   cumulative: HodledOrLost,
 *   sum: Sum,
 *   avg: Sum,
 *   min: Sum,
 *   max: Sum,
 *   pct10: Sum,
 *   pct25: Sum,
 *   median: Sum,
 *   pct75: Sum,
 *   pct90: Sum,
 *   share: Share,
 *   toSubsidy: InputsPerSecond<?Ratio>,
 * }} RewardsFees
 */
const _RewardsFees = _s({
  block: [_Block, '*'],
  cumulative: [_HodledOrLost, '*_cumulative'],
  sum: [_Sum, '*_sum'],
  avg: [_Sum, '*_avg'],
  min: [_Sum, '*_min'],
  max: [_Sum, '*_max'],
  pct10: [_Sum, '*_pct10'],
  pct25: [_Sum, '*_pct25'],
  median: [_Sum, '*_median'],
  pct75: [_Sum, '*_pct75'],
  pct90: [_Sum, '*_pct90'],
  share: [_Share, 'fee_share'],
  toSubsidy: [_InputsPerSecond, 'fee_to_subsidy'],
});

/**
 * @typedef {{
 *   coinbase: TransferVolume0sats,
 *   subsidy: Subsidy,
 *   fees: RewardsFees,
 *   outputVolume: TransferVolume0sats,
 *   unclaimed: OpReturnValue,
 * }} Rewards
 */
const _Rewards = _s({
  coinbase: [_TransferVolume0sats, '*'],
  subsidy: [_Subsidy, 'subsidy'],
  fees: [_RewardsFees, 'fees'],
  outputVolume: [_TransferVolume0sats, 'output_volume'],
  unclaimed: [_OpReturnValue, 'unclaimed_rewards'],
});

/**
 * @typedef {{
 *   rewards: Rewards,
 *   hashrate: MiningHashrate,
 * }} Mining
 */
const _Mining = _s({
  rewards: [_Rewards, '*'],
  hashrate: [_MiningHashrate, 'hash'],
});

/**
 * @template A, B, C
 * @typedef {{
 *   cumulative: SeriesPattern2<A>,
 *   sum: InputsPerSecond<A>,
 *   avg: InputsPerSecond<B>,
 *   min: InputsPerSecond<C>,
 *   max: InputsPerSecond<C>,
 *   pct10: InputsPerSecond<C>,
 *   pct25: InputsPerSecond<C>,
 *   median: InputsPerSecond<C>,
 *   pct75: InputsPerSecond<C>,
 *   pct90: InputsPerSecond<C>,
 * }} BlocksSize
 */
const _BlocksSize = _s({
  cumulative: [_i2, '*_cumulative'],
  sum: [_InputsPerSecond, '*_sum'],
  avg: [_InputsPerSecond, '*_avg'],
  min: [_InputsPerSecond, '*_min'],
  max: [_InputsPerSecond, '*_max'],
  pct10: [_InputsPerSecond, '*_pct10'],
  pct25: [_InputsPerSecond, '*_pct25'],
  median: [_InputsPerSecond, '*_median'],
  pct75: [_InputsPerSecond, '*_pct75'],
  pct90: [_InputsPerSecond, '*_pct90'],
});

/**
 * @template A, B, C
 * @typedef {{
 *   block: SeriesPattern21<A>,
 *   cumulative: SeriesPattern2<A>,
 *   sum: InputsPerSecond<A>,
 *   avg: InputsPerSecond<B>,
 *   min: InputsPerSecond<C>,
 *   max: InputsPerSecond<C>,
 *   pct10: InputsPerSecond<C>,
 *   pct25: InputsPerSecond<C>,
 *   median: InputsPerSecond<C>,
 *   pct75: InputsPerSecond<C>,
 *   pct90: InputsPerSecond<C>,
 * }} Vbytes
 */
const _Vbytes = _s({
  block: [_i21, '*'],
  cumulative: [_i2, '*_cumulative'],
  sum: [_InputsPerSecond, '*_sum'],
  avg: [_InputsPerSecond, '*_avg'],
  min: [_InputsPerSecond, '*_min'],
  max: [_InputsPerSecond, '*_max'],
  pct10: [_InputsPerSecond, '*_pct10'],
  pct25: [_InputsPerSecond, '*_pct25'],
  median: [_InputsPerSecond, '*_median'],
  pct75: [_InputsPerSecond, '*_pct75'],
  pct90: [_InputsPerSecond, '*_pct90'],
});

/**
 * @template A, B
 * @typedef {{
 *   block: SeriesPattern21<A>,
 *   avg: InputsPerSecond<B>,
 * }} Interval
 */
const _Interval = _s({
  block: [_i21, '*'],
  avg: [_InputsPerSecond, '*_avg'],
});

/**
 * @template A
 * @typedef {{
 *   block: SeriesPattern21<A>,
 *   cumulative: SeriesPattern2<A>,
 *   sum: InputsPerSecond<A>,
 * }} CoinblocksDestroyed
 */
const _CoinblocksDestroyed = _s({
  block: [_i21, '*'],
  cumulative: [_i2, '*_cumulative'],
  sum: [_InputsPerSecond, '*_sum'],
});

/**
 * @typedef {{
 *   outputCount: CoinblocksDestroyed<Count>,
 *   dataBytes: MultipleDataBytes,
 *   txCount: CoinblocksDestroyed<Count>,
 *   txVsize: CoinblocksDestroyed<VSize>,
 *   fees: OpReturnFees,
 * }} Multiple
 */
const _Multiple = _s({
  outputCount: [_CoinblocksDestroyed, '*_output_count'],
  dataBytes: [_MultipleDataBytes, '*'],
  txCount: [_CoinblocksDestroyed, '*_tx_count'],
  txVsize: [_CoinblocksDestroyed, '*_tx_vsize'],
  fees: [_OpReturnFees, '*'],
});

/**
 * @typedef {{
 *   preV30Standard: Multiple,
 *   preV30Nonstandard: Multiple,
 *   oversized: Multiple,
 *   multiple: Multiple,
 * }} Policies
 */
const _Policies = _s({
  preV30Standard: [_Multiple, '*_pre_v30_standard'],
  preV30Nonstandard: [_Multiple, '*_pre_v30_nonstandard'],
  oversized: [_Multiple, '*_oversized'],
  multiple: [_Multiple, '*_multiple'],
});

/**
 * @typedef {{
 *   runes: Multiple,
 *   veriBlock: Multiple,
 *   omni: Multiple,
 *   stacks: Multiple,
 *   blockstack: Multiple,
 *   colu: Multiple,
 *   openAssets: Multiple,
 *   komodo: Multiple,
 *   coinSpark: Multiple,
 *   poet: Multiple,
 *   docproof: Multiple,
 *   openTimestamps: Multiple,
 *   factom: Multiple,
 *   eternityWall: Multiple,
 *   memo: Multiple,
 *   bitproof: Multiple,
 *   ascribe: Multiple,
 *   stampery: Multiple,
 *   epobc: Multiple,
 *   bareHash: Multiple,
 *   text: Multiple,
 *   empty: Multiple,
 *   unknown: Multiple,
 * }} Protocols
 */
const _Protocols = _s({
  runes: [_Multiple, '*_runes'],
  veriBlock: [_Multiple, '*_veri_block'],
  omni: [_Multiple, '*_omni'],
  stacks: [_Multiple, '*_stacks'],
  blockstack: [_Multiple, '*_blockstack'],
  colu: [_Multiple, '*_colu'],
  openAssets: [_Multiple, '*_open_assets'],
  komodo: [_Multiple, '*_komodo'],
  coinSpark: [_Multiple, '*_coin_spark'],
  poet: [_Multiple, '*_poet'],
  docproof: [_Multiple, '*_docproof'],
  openTimestamps: [_Multiple, '*_open_timestamps'],
  factom: [_Multiple, '*_factom'],
  eternityWall: [_Multiple, '*_eternity_wall'],
  memo: [_Multiple, '*_memo'],
  bitproof: [_Multiple, '*_bitproof'],
  ascribe: [_Multiple, '*_ascribe'],
  stampery: [_Multiple, '*_stampery'],
  epobc: [_Multiple, '*_epobc'],
  bareHash: [_Multiple, '*_bare_hash'],
  text: [_Multiple, '*_text'],
  empty: [_Multiple, '*_empty'],
  unknown: [_Multiple, '*_unknown'],
});

/**
 * @typedef {{
 *   outputCount: CoinblocksDestroyed<Count>,
 *   dataBytes: OpReturnDataBytes,
 *   txCount: CoinblocksDestroyed<Count>,
 *   txVsize: CoinblocksDestroyed<VSize>,
 *   fees: OpReturnFees,
 *   protocols: Protocols,
 *   policies: Policies,
 * }} OpReturn
 */
const _OpReturn = _s({
  outputCount: [_CoinblocksDestroyed, '*_output_count'],
  dataBytes: [_OpReturnDataBytes, '*'],
  txCount: [_CoinblocksDestroyed, '*_tx_count'],
  txVsize: [_CoinblocksDestroyed, '*_tx_vsize'],
  fees: [_OpReturnFees, '*'],
  protocols: [_Protocols, '*'],
  policies: [_Policies, '*'],
});

/**
 * @typedef {{
 *   count: CoinblocksDestroyed<Count>,
 *   share: Share,
 *   txCount: CoinblocksDestroyed<Count>,
 *   txShare: Share,
 * }} TypesEmpty
 */
const _TypesEmpty = _s({
  count: [_CoinblocksDestroyed, '*_count'],
  share: [_Share, '*_share'],
  txCount: [_CoinblocksDestroyed, '*_tx_count'],
  txShare: [_Share, '*_tx_share'],
});

/**
 * @typedef {{
 *   p2pk65: TypesEmpty,
 *   p2pk33: TypesEmpty,
 *   p2pkh: TypesEmpty,
 *   p2ms: TypesEmpty,
 *   p2sh: TypesEmpty,
 *   p2wpkh: TypesEmpty,
 *   p2wsh: TypesEmpty,
 *   p2tr: TypesEmpty,
 *   p2a: TypesEmpty,
 *   unknown: TypesEmpty,
 *   empty: TypesEmpty,
 *   opReturn: TypesEmpty,
 * }} OutputsTypes
 */
const _OutputsTypes = _s({
  p2pk65: [_TypesEmpty, 'p2pk65_*'],
  p2pk33: [_TypesEmpty, 'p2pk33_*'],
  p2pkh: [_TypesEmpty, 'p2pkh_*'],
  p2ms: [_TypesEmpty, 'p2ms_*'],
  p2sh: [_TypesEmpty, 'p2sh_*'],
  p2wpkh: [_TypesEmpty, 'p2wpkh_*'],
  p2wsh: [_TypesEmpty, 'p2wsh_*'],
  p2tr: [_TypesEmpty, 'p2tr_*'],
  p2a: [_TypesEmpty, 'p2a_*'],
  unknown: [_TypesEmpty, 'unknown_*'],
  empty: [_TypesEmpty, 'empty_*'],
  opReturn: [_TypesEmpty, 'op_return_*'],
});

/**
 * @typedef {{
 *   spent: Spent,
 *   count: Vbytes<Count, ?CountFract, Count32>,
 *   perSecond: InputsPerSecond<?PerSecond>,
 *   spendableCount: CoinblocksDestroyed<Count>,
 *   types: OutputsTypes,
 *   opReturnValue: OpReturnValue,
 * }} Outputs
 */
const _Outputs = _s({
  spent: [_Spent, 'txin_index'],
  count: [_Vbytes, '*_count'],
  perSecond: [_InputsPerSecond, 'outputs_per_second'],
  spendableCount: [_CoinblocksDestroyed, 'spendable_*_count'],
  types: [_OutputsTypes, '*'],
  opReturnValue: [_OpReturnValue, 'op_return_value'],
});

/**
 * @typedef {{
 *   p2pk65: TypesEmpty,
 *   p2pk33: TypesEmpty,
 *   p2pkh: TypesEmpty,
 *   p2ms: TypesEmpty,
 *   p2sh: TypesEmpty,
 *   p2wpkh: TypesEmpty,
 *   p2wsh: TypesEmpty,
 *   p2tr: TypesEmpty,
 *   p2a: TypesEmpty,
 *   unknown: TypesEmpty,
 *   empty: TypesEmpty,
 * }} InputsTypes
 */
const _InputsTypes = _s({
  p2pk65: [_TypesEmpty, 'p2pk65_*'],
  p2pk33: [_TypesEmpty, 'p2pk33_*'],
  p2pkh: [_TypesEmpty, 'p2pkh_*'],
  p2ms: [_TypesEmpty, 'p2ms_*'],
  p2sh: [_TypesEmpty, 'p2sh_*'],
  p2wpkh: [_TypesEmpty, 'p2wpkh_*'],
  p2wsh: [_TypesEmpty, 'p2wsh_*'],
  p2tr: [_TypesEmpty, 'p2tr_*'],
  p2a: [_TypesEmpty, 'p2a_*'],
  unknown: [_TypesEmpty, 'unknown_*'],
  empty: [_TypesEmpty, 'empty_*'],
});

/**
 * @typedef {{
 *   value: SeriesPattern23<Sats>,
 *   count: Vbytes<Count, ?CountFract, Count16>,
 *   perSecond: InputsPerSecond<?PerSecond>,
 *   types: InputsTypes,
 * }} Inputs
 */
const _Inputs = _s({
  value: [_i23, 'value'],
  count: [_Vbytes, '*_count'],
  perSecond: [_InputsPerSecond, 'inputs_per_second'],
  types: [_InputsTypes, '*'],
});

/**
 * @template A
 * @typedef {{
 *   all: A,
 *   p2pk65: A,
 *   p2pk33: A,
 *   p2pkh: A,
 *   p2sh: A,
 *   p2wpkh: A,
 *   p2wsh: A,
 *   p2tr: A,
 *   p2a: A,
 * }} AvgBalance
 */
/** @type {_Make} */
const _AvgBalance = (c, b, f0) => _n(c, b, {
  all: [f0, '*'],
  p2pk65: [f0, 'p2pk65_*'],
  p2pk33: [f0, 'p2pk33_*'],
  p2pkh: [f0, 'p2pkh_*'],
  p2sh: [f0, 'p2sh_*'],
  p2wpkh: [f0, 'p2wpkh_*'],
  p2wsh: [f0, 'p2wsh_*'],
  p2tr: [f0, 'p2tr_*'],
  p2a: [f0, 'p2a_*'],
});

/**
 * @typedef {{
 *   outputToReusedAddrCount: AvgBalance<CoinblocksDestroyed<Count>>,
 *   outputToReusedAddrShare: AvgBalance<Share>,
 *   spendableOutputToReusedAddrShare: Share,
 *   inputFromReusedAddrCount: AvgBalance<CoinblocksDestroyed<Count>>,
 *   inputFromReusedAddrShare: AvgBalance<Share>,
 *   activeReusedAddrCount: Interval<Count, ?CountFract>,
 *   activeReusedAddrShare: Share,
 * }} Events
 */
const _Events = _s({
  outputToReusedAddrCount: [(c, b) => _AvgBalance(c, b, _CoinblocksDestroyed), 'output_to_*_count'],
  outputToReusedAddrShare: [(c, b) => _AvgBalance(c, b, _Share), 'output_to_*_share'],
  spendableOutputToReusedAddrShare: [_Share, 'spendable_output_to_*_share'],
  inputFromReusedAddrCount: [(c, b) => _AvgBalance(c, b, _CoinblocksDestroyed), 'input_from_*_count'],
  inputFromReusedAddrShare: [(c, b) => _AvgBalance(c, b, _Share), 'input_from_*_share'],
  activeReusedAddrCount: [_Interval, 'active_*_count'],
  activeReusedAddrShare: [_Share, 'active_*_share'],
});

/**
 * @typedef {{
 *   count: ExposedCount,
 *   events: Events,
 *   supply: ExposedSupply,
 * }} Respent
 */
const _Respent = _s({
  count: [_ExposedCount, '*_count'],
  events: [_Events, '*'],
  supply: [_ExposedSupply, '*_supply'],
});

/**
 * @typedef {{
 *   reactivated: AvgBalance<Interval<Count, ?CountFract>>,
 *   sending: AvgBalance<Interval<Count, ?CountFract>>,
 *   receiving: AvgBalance<Interval<Count, ?CountFract>>,
 *   bidirectional: AvgBalance<Interval<Count, ?CountFract>>,
 *   active: AvgBalance<Interval<Count, ?CountFract>>,
 * }} AddressesActivity
 */
const _AddressesActivity = _s({
  reactivated: [(c, b) => _AvgBalance(c, b, _Interval), 'reactivated_*'],
  sending: [(c, b) => _AvgBalance(c, b, _Interval), 'sending_*'],
  receiving: [(c, b) => _AvgBalance(c, b, _Interval), 'receiving_*'],
  bidirectional: [(c, b) => _AvgBalance(c, b, _Interval), 'bidirectional_*'],
  active: [(c, b) => _AvgBalance(c, b, _Interval), 'active_*'],
});

/**
 * @template A
 * @typedef {{
 *   _0sats: A,
 *   _1satTo10sats: A,
 *   _10satsTo100sats: A,
 *   _100satsTo1kSats: A,
 *   _1kSatsTo10kSats: A,
 *   _10kSatsTo100kSats: A,
 *   _100kSatsTo1mSats: A,
 *   _1mSatsTo10mSats: A,
 *   _10mSatsTo1btc: A,
 *   _1btcTo10btc: A,
 *   _10btcTo100btc: A,
 *   _100btcTo1kBtc: A,
 *   _1kBtcTo10kBtc: A,
 *   _10kBtcTo100kBtc: A,
 *   over100kBtc: A,
 * }} RealizedLoss
 */
/** @type {_Make} */
const _RealizedLoss = (c, b, f0) => _n(c, b, {
  _0sats: [f0, 'addrs_0sats_*'],
  _1satTo10sats: [f0, 'addrs_1sat_to_10sats_*'],
  _10satsTo100sats: [f0, 'addrs_10sats_to_100sats_*'],
  _100satsTo1kSats: [f0, 'addrs_100sats_to_1k_sats_*'],
  _1kSatsTo10kSats: [f0, 'addrs_1k_sats_to_10k_sats_*'],
  _10kSatsTo100kSats: [f0, 'addrs_10k_sats_to_100k_sats_*'],
  _100kSatsTo1mSats: [f0, 'addrs_100k_sats_to_1m_sats_*'],
  _1mSatsTo10mSats: [f0, 'addrs_1m_sats_to_10m_sats_*'],
  _10mSatsTo1btc: [f0, 'addrs_10m_sats_to_1btc_*'],
  _1btcTo10btc: [f0, 'addrs_1btc_to_10btc_*'],
  _10btcTo100btc: [f0, 'addrs_10btc_to_100btc_*'],
  _100btcTo1kBtc: [f0, 'addrs_100btc_to_1k_btc_*'],
  _1kBtcTo10kBtc: [f0, 'addrs_1k_btc_to_10k_btc_*'],
  _10kBtcTo100kBtc: [f0, 'addrs_10k_btc_to_100k_btc_*'],
  over100kBtc: [f0, 'addrs_over_100k_btc_*'],
});

/**
 * @typedef {{
 *   all: SeriesPattern2<Count>,
 *   p2pk65: SeriesPattern2<Count>,
 *   p2pk33: SeriesPattern2<Count>,
 *   p2pkh: SeriesPattern2<Count>,
 *   p2sh: SeriesPattern2<Count>,
 *   p2wpkh: SeriesPattern2<Count>,
 *   p2wsh: SeriesPattern2<Count>,
 *   p2tr: SeriesPattern2<Count>,
 *   p2a: SeriesPattern2<Count>,
 *   balance: RealizedLoss<MarketCap<Count, CountSigned>>,
 * }} Funded
 */
const _Funded = _s({
  all: [_i2, '*'],
  p2pk65: [_i2, 'p2pk65_*'],
  p2pk33: [_i2, 'p2pk33_*'],
  p2pkh: [_i2, 'p2pkh_*'],
  p2sh: [_i2, 'p2sh_*'],
  p2wpkh: [_i2, 'p2wpkh_*'],
  p2wsh: [_i2, 'p2wsh_*'],
  p2tr: [_i2, 'p2tr_*'],
  p2a: [_i2, 'p2a_*'],
  balance: [(c, b) => _RealizedLoss(c, b, _MarketCap), '*'],
});

/**
 * @typedef {{
 *   state: State,
 *   supply: RealizedLoss<Supply0sats>,
 *   utxoCount: RealizedLoss<MarketCap<Count, CountSigned>>,
 *   transferVolume: RealizedLoss<TransferVolume0sats>,
 *   realizedCap: RealizedCap,
 *   realizedProfit: RealizedLoss<CoinblocksDestroyed<?Dollars>>,
 *   realizedLoss: RealizedLoss<CoinblocksDestroyed<?Dollars>>,
 *   funded: Funded,
 *   empty: AddressesEmpty<Count>,
 *   activity: AddressesActivity,
 *   total: AddressesEmpty<Count>,
 *   new: AvgBalance<CoinblocksDestroyed<Count>>,
 *   reused: Respent,
 *   respent: Respent,
 *   exposed: Exposed,
 *   delta: AvgBalance<DeltaAll<CountSigned>>,
 *   avgBalance: AvgBalance<HodledOrLost>,
 * }} Addresses
 */
const _Addresses = _s({
  state: [_State, '*'],
  supply: [(c, b) => _RealizedLoss(c, b, _Supply0sats), 'supply'],
  utxoCount: [(c, b) => _RealizedLoss(c, b, _MarketCap), 'utxo_count'],
  transferVolume: [(c, b) => _RealizedLoss(c, b, _TransferVolume0sats), 'transfer_volume'],
  realizedCap: [_RealizedCap, 'realized_cap'],
  realizedProfit: [(c, b) => _RealizedLoss(c, b, _CoinblocksDestroyed), 'realized_profit'],
  realizedLoss: [(c, b) => _RealizedLoss(c, b, _CoinblocksDestroyed), 'realized_loss'],
  funded: [_Funded, '*_count'],
  empty: [_AddressesEmpty, 'empty_*_count'],
  activity: [_AddressesActivity, 'addrs'],
  total: [_AddressesEmpty, 'total_*_count'],
  new: [(c, b) => _AvgBalance(c, b, _CoinblocksDestroyed), 'new_*_count'],
  reused: [_Respent, 'reused_*'],
  respent: [_Respent, 'respent_*'],
  exposed: [_Exposed, 'exposed_*'],
  delta: [(c, b) => _AvgBalance(c, b, _DeltaAll), '*_count'],
  avgBalance: [(c, b) => _AvgBalance(c, b, _HodledOrLost), 'avg_*_amount'],
});

/**
 * @template A
 * @typedef {{
 *   _0sats: A,
 *   _1satTo10sats: A,
 *   _10satsTo100sats: A,
 *   _100satsTo1kSats: A,
 *   _1kSatsTo10kSats: A,
 *   _10kSatsTo100kSats: A,
 *   _100kSatsTo1mSats: A,
 *   _1mSatsTo10mSats: A,
 *   _10mSatsTo1btc: A,
 *   _1btcTo10btc: A,
 *   _10btcTo100btc: A,
 *   _100btcTo1kBtc: A,
 *   _1kBtcTo10kBtc: A,
 *   _10kBtcTo100kBtc: A,
 *   over100kBtc: A,
 * }} TransferVolumeUtxoAmount
 */
/** @type {_Make} */
const _TransferVolumeUtxoAmount = (c, b, f0) => _n(c, b, {
  _0sats: [f0, 'utxos_0sats_*'],
  _1satTo10sats: [f0, 'utxos_1sat_to_10sats_*'],
  _10satsTo100sats: [f0, 'utxos_10sats_to_100sats_*'],
  _100satsTo1kSats: [f0, 'utxos_100sats_to_1k_sats_*'],
  _1kSatsTo10kSats: [f0, 'utxos_1k_sats_to_10k_sats_*'],
  _10kSatsTo100kSats: [f0, 'utxos_10k_sats_to_100k_sats_*'],
  _100kSatsTo1mSats: [f0, 'utxos_100k_sats_to_1m_sats_*'],
  _1mSatsTo10mSats: [f0, 'utxos_1m_sats_to_10m_sats_*'],
  _10mSatsTo1btc: [f0, 'utxos_10m_sats_to_1btc_*'],
  _1btcTo10btc: [f0, 'utxos_1btc_to_10btc_*'],
  _10btcTo100btc: [f0, 'utxos_10btc_to_100btc_*'],
  _100btcTo1kBtc: [f0, 'utxos_100btc_to_1k_btc_*'],
  _1kBtcTo10kBtc: [f0, 'utxos_1k_btc_to_10k_btc_*'],
  _10kBtcTo100kBtc: [f0, 'utxos_10k_btc_to_100k_btc_*'],
  over100kBtc: [f0, 'utxos_over_100k_btc_*'],
});

/**
 * @template A
 * @typedef {{
 *   p2pk65: A,
 *   p2pk33: A,
 *   p2pkh: A,
 *   p2ms: A,
 *   p2sh: A,
 *   p2wpkh: A,
 *   p2wsh: A,
 *   p2tr: A,
 *   p2a: A,
 *   unknown: A,
 *   empty: A,
 * }} TransferVolumeType
 */
/** @type {_Make} */
const _TransferVolumeType = (c, b, f0) => _n(c, b, {
  p2pk65: [f0, 'p2pk65_*'],
  p2pk33: [f0, 'p2pk33_*'],
  p2pkh: [f0, 'p2pkh_*'],
  p2ms: [f0, 'p2ms_*'],
  p2sh: [f0, 'p2sh_*'],
  p2wpkh: [f0, 'p2wpkh_*'],
  p2wsh: [f0, 'p2wsh_*'],
  p2tr: [f0, 'p2tr_*'],
  p2a: [f0, 'p2a_*'],
  unknown: [f0, 'unknown_outputs_*'],
  empty: [f0, 'empty_outputs_*'],
});

/**
 * @typedef {{
 *   all: HodledOrLost,
 *   byType: TransferVolumeType<HodledOrLost>,
 * }} AvgAmount
 */
const _AvgAmount = _s({
  all: [_HodledOrLost, '*'],
  byType: [(c, b) => _TransferVolumeType(c, b, _HodledOrLost), '*'],
});

/**
 * @template A, B
 * @typedef {{
 *   utxoAmount: A,
 *   type: B,
 * }} UtxosActivityTransferVolume
 */
/** @type {_Make} */
const _UtxosActivityTransferVolume = (c, b, f0, f1) => _n(c, b, {
  utxoAmount: [f0, '*'],
  type: [f1, '*'],
});

/**
 * @typedef {{
 *   cap: UtxosActivityTransferVolume<CapUtxoAmount<?Dollars>, CapType<?Dollars>>,
 *   price: UtxosActivityTransferVolume<CapUtxoAmount<?Dollars>, CapType<?Dollars>>,
 *   profit: UtxosActivityTransferVolume<TransferVolumeUtxoAmount<CoinblocksDestroyed<?Dollars>>, TransferVolumeType<CoinblocksDestroyed<?Dollars>>>,
 *   loss: UtxosActivityTransferVolume<TransferVolumeUtxoAmount<CoinblocksDestroyed<?Dollars>>, TransferVolumeType<CoinblocksDestroyed<?Dollars>>>,
 * }} UtxosRealized
 */
const _UtxosRealized = _s({
  cap: [(c, b) => _UtxosActivityTransferVolume(c, b, _CapUtxoAmount, _CapType), '*_cap'],
  price: [(c, b) => _UtxosActivityTransferVolume(c, b, _CapUtxoAmount, _CapType), '*_price'],
  profit: [(c, b) => _UtxosActivityTransferVolume(c, b, (c, b) => _TransferVolumeUtxoAmount(c, b, _CoinblocksDestroyed), (c, b) => _TransferVolumeType(c, b, _CoinblocksDestroyed)), '*_profit'],
  loss: [(c, b) => _UtxosActivityTransferVolume(c, b, (c, b) => _TransferVolumeUtxoAmount(c, b, _CoinblocksDestroyed), (c, b) => _TransferVolumeType(c, b, _CoinblocksDestroyed)), '*_loss'],
});

/**
 * @typedef {{
 *   transferVolume: UtxosActivityTransferVolume<TransferVolumeUtxoAmount<TransferVolume0sats>, TransferVolumeType<TransferVolume0sats>>,
 * }} UtxosActivity
 */
const _UtxosActivity = _s({
  transferVolume: [(c, b) => _UtxosActivityTransferVolume(c, b, (c, b) => _TransferVolumeUtxoAmount(c, b, _TransferVolume0sats), (c, b) => _TransferVolumeType(c, b, _TransferVolume0sats)), '*'],
});

/**
 * @typedef {{
 *   avgAmount: AvgAmount,
 *   unspentCount: UtxosActivityTransferVolume<TransferVolumeUtxoAmount<MarketCap<Count, CountSigned>>, TransferVolumeType<MarketCap<Count, CountSigned>>>,
 *   spentCount: UtxosActivityTransferVolume<TransferVolumeUtxoAmount<CoinblocksDestroyed<Count>>, TransferVolumeType<CoinblocksDestroyed<Count>>>,
 * }} UtxosOutputs
 */
const _UtxosOutputs = _s({
  avgAmount: [_AvgAmount, 'avg_*_amount'],
  unspentCount: [(c, b) => _UtxosActivityTransferVolume(c, b, (c, b) => _TransferVolumeUtxoAmount(c, b, _MarketCap), (c, b) => _TransferVolumeType(c, b, _MarketCap)), '*_count'],
  spentCount: [(c, b) => _UtxosActivityTransferVolume(c, b, (c, b) => _TransferVolumeUtxoAmount(c, b, _CoinblocksDestroyed), (c, b) => _TransferVolumeType(c, b, _CoinblocksDestroyed)), 'spent_*_count'],
});

/**
 * @typedef {{
 *   utxoAmount: TotalUtxoAmount,
 *   type: TransferVolumeType<HodledOrLost>,
 * }} UtxosSupplyTotal
 */
const _UtxosSupplyTotal = _s({
  utxoAmount: [_TotalUtxoAmount, '*'],
  type: [(c, b) => _TransferVolumeType(c, b, _HodledOrLost), 'supply'],
});

/**
 * @typedef {{
 *   total: UtxosSupplyTotal,
 *   delta: UtxosActivityTransferVolume<TransferVolumeUtxoAmount<DeltaAll<?Bitcoin>>, TransferVolumeType<DeltaAll<?Bitcoin>>>,
 *   dominance: UtxosActivityTransferVolume<CapUtxoAmount<?Percent>, CapType<?Percent>>,
 * }} UtxosSupply
 */
const _UtxosSupply = _s({
  total: [_UtxosSupplyTotal, 'utxos'],
  delta: [(c, b) => _UtxosActivityTransferVolume(c, b, (c, b) => _TransferVolumeUtxoAmount(c, b, _DeltaAll), (c, b) => _TransferVolumeType(c, b, _DeltaAll)), '*_delta'],
  dominance: [(c, b) => _UtxosActivityTransferVolume(c, b, _CapUtxoAmount, _CapType), '*_dominance'],
});

/**
 * @typedef {{
 *   supply: UtxosSupply,
 *   outputs: UtxosOutputs,
 *   activity: UtxosActivity,
 *   realized: UtxosRealized,
 * }} Utxos
 */
const _Utxos = _s({
  supply: [_UtxosSupply, '*'],
  outputs: [_UtxosOutputs, 'utxo'],
  activity: [_UtxosActivity, 'transfer_volume'],
  realized: [_UtxosRealized, 'realized'],
});

/**
 * @typedef {{
 *   _24h: SeriesPattern2<?Ratio>,
 *   valueDestroyed: CoinblocksDestroyed<?Dollars>,
 * }} Sopr
 */
const _Sopr = _s({
  _24h: [_i2, '*_sopr_24h'],
  valueDestroyed: [_CoinblocksDestroyed, '*_value_destroyed'],
});

/**
 * @typedef {{
 *   cap: MarketCap<?Dollars, ?Dollars>,
 *   price: CapitalizedPrice,
 *   profit: CoinblocksDestroyed<?Dollars>,
 *   loss: CoinblocksDestroyed<?Dollars>,
 *   netPnl: NetPnl,
 *   sopr: Sopr,
 *   mvrv: SeriesPattern2<?Ratio>,
 * }} DiscountRealized
 */
const _DiscountRealized = _s({
  cap: [_MarketCap, '*_realized_cap'],
  price: [_CapitalizedPrice, '*_realized_price'],
  profit: [_CoinblocksDestroyed, '*_realized_profit'],
  loss: [_CoinblocksDestroyed, '*_realized_loss'],
  netPnl: [_NetPnl, '*_net_realized_pnl'],
  sopr: [_Sopr, '*'],
  mvrv: [_i2, '*_mvrv'],
});

/**
 * @typedef {{
 *   transferVolume: DiscountActivityTransferVolume,
 *   coindaysDestroyed: CoinblocksDestroyed<?CoinDays>,
 * }} DiscountActivity
 */
const _DiscountActivity = _s({
  transferVolume: [_DiscountActivityTransferVolume, '*_transfer_volume'],
  coindaysDestroyed: [_CoinblocksDestroyed, '*_coindays_destroyed'],
});

/**
 * @typedef {{
 *   ratio: InputsPerSecond<?Ratio>,
 *   transferVolume: CoinblocksDestroyed<?Dollars>,
 *   valueDestroyed: CoinblocksDestroyed<?Dollars>,
 * }} AdjustedSopr
 */
const _AdjustedSopr = _s({
  ratio: [_InputsPerSecond, '*_adjusted_sopr'],
  transferVolume: [_CoinblocksDestroyed, '*_adj_value_created'],
  valueDestroyed: [_CoinblocksDestroyed, '*_adj_value_destroyed'],
});

/**
 * @typedef {{
 *   adjustedSopr: AdjustedSopr,
 *   dormancy: InputsPerSecond<?Days>,
 *   sopr: SeriesPattern2<?Ratio>,
 *   soprRatioExtended: SoprRatioExtended,
 *   sellSideRiskRatio: InputsPerSecond<?Ratio>,
 *   profitToLossRatio: InputsPerSecond<?Ratio>,
 * }} Ratios
 */
const _Ratios = _s({
  adjustedSopr: [_AdjustedSopr, '*'],
  dormancy: [_InputsPerSecond, '*_dormancy'],
  sopr: [_i2, '*_sopr_24h'],
  soprRatioExtended: [_SoprRatioExtended, '*_sopr'],
  sellSideRiskRatio: [_InputsPerSecond, '*_sell_side_risk_ratio'],
  profitToLossRatio: [_InputsPerSecond, '*_realized_profit_to_loss_ratio'],
});

/**
 * @typedef {{
 *   cap: MarketCap<?Dollars, ?Dollars>,
 *   price: SeriesPattern2<?Dollars>,
 *   capitalizedPrice: CapitalizedPrice,
 *   profit: CoinblocksDestroyed<?Dollars>,
 *   loss: CoinblocksDestroyed<?Dollars>,
 *   netPnl: NetPnl,
 *   valueDestroyed: CoinblocksDestroyed<?Dollars>,
 *   grossPnl: CoinblocksDestroyed<?Dollars>,
 *   peakRegret: CoinblocksDestroyed<?Dollars>,
 *   mvrv: SeriesPattern2<?Ratio>,
 * }} AllRealized
 */
const _AllRealized = _s({
  cap: [_MarketCap, '*_realized_cap'],
  price: [_i2, '*_realized_price'],
  capitalizedPrice: [_CapitalizedPrice, '*_capitalized_price'],
  profit: [_CoinblocksDestroyed, '*_realized_profit'],
  loss: [_CoinblocksDestroyed, '*_realized_loss'],
  netPnl: [_NetPnl, '*_net_realized_pnl'],
  valueDestroyed: [_CoinblocksDestroyed, '*_value_destroyed'],
  grossPnl: [_CoinblocksDestroyed, '*_realized_gross_pnl'],
  peakRegret: [_CoinblocksDestroyed, '*_realized_peak_regret'],
  mvrv: [_i2, '*_mvrv'],
});

/**
 * @typedef {{
 *   transferVolume: TransferVolume0sats,
 *   transferVolumeInProfit: TransferVolume0sats,
 *   transferVolumeInLoss: TransferVolume0sats,
 *   coindaysDestroyed: CoinblocksDestroyed<?CoinDays>,
 *   coinyearsDestroyed: SeriesPattern2<?CoinYears>,
 * }} AllActivity
 */
const _AllActivity = _s({
  transferVolume: [_TransferVolume0sats, '*_transfer_volume'],
  transferVolumeInProfit: [_TransferVolume0sats, '*_transfer_volume_in_profit'],
  transferVolumeInLoss: [_TransferVolume0sats, '*_transfer_volume_in_loss'],
  coindaysDestroyed: [_CoinblocksDestroyed, '*_coindays_destroyed'],
  coinyearsDestroyed: [_i2, '*_coinyears_destroyed'],
});

/**
 * @typedef {{
 *   unspentCount: MarketCap<Count, CountSigned>,
 *   spentCount: CoinblocksDestroyed<Count>,
 * }} DiscountOutputs
 */
const _DiscountOutputs = _s({
  unspentCount: [_MarketCap, '*_utxo_count'],
  spentCount: [_CoinblocksDestroyed, '*_spent_utxo_count'],
});

/**
 * @typedef {{
 *   supply: DiscountSupply,
 *   outputs: DiscountOutputs,
 *   activity: DiscountActivity,
 *   realized: DiscountRealized,
 *   unrealized: DiscountUnrealized,
 * }} Discount
 */
const _Discount = _s({
  supply: [_DiscountSupply, '*_supply'],
  outputs: [_DiscountOutputs, '*'],
  activity: [_DiscountActivity, '*'],
  realized: [_DiscountRealized, '*'],
  unrealized: [_DiscountUnrealized, '*'],
});

/**
 * @typedef {{
 *   discount: Discount,
 *   premium: Discount,
 * }} Entry
 */
const _Entry = _s({
  discount: [_Discount, '*'],
  premium: [_Discount, 'rookie'],
});

/**
 * @template A
 * @typedef {{
 *   supply: A,
 *   outputs: DiscountOutputs,
 *   activity: AllActivity,
 *   realized: AllRealized,
 *   unrealized: AllUnrealized,
 *   costBasis: HoldersAllCostBasis,
 *   ratios: Ratios,
 *   relative: Relative,
 * }} HoldersAll
 */
/** @type {_Make} */
const _HoldersAll = (c, b, f0) => _n(c, b, {
  supply: [f0, '*_supply'],
  outputs: [_DiscountOutputs, '*'],
  activity: [_AllActivity, '*'],
  realized: [_AllRealized, '*'],
  unrealized: [_AllUnrealized, '*'],
  costBasis: [_HoldersAllCostBasis, '*'],
  ratios: [_Ratios, '*'],
  relative: [_Relative, '*'],
});

/**
 * @typedef {{
 *   all: HoldersAll<AllSupply>,
 *   sth: HoldersAll<LthSupply>,
 *   lth: HoldersAll<LthSupply>,
 *   under4m: HoldersAll<LthSupply>,
 *   under6m: HoldersAll<LthSupply>,
 *   over4m: HoldersAll<LthSupply>,
 *   over6m: HoldersAll<LthSupply>,
 * }} Holders
 */
const _Holders = _s({
  all: [(c, b) => _HoldersAll(c, b, _AllSupply), ''],
  sth: [(c, b) => _HoldersAll(c, b, _LthSupply), 'sth'],
  lth: [(c, b) => _HoldersAll(c, b, _LthSupply), 'lth'],
  under4m: [(c, b) => _HoldersAll(c, b, _LthSupply), '*_4m'],
  under6m: [(c, b) => _HoldersAll(c, b, _LthSupply), '*_6m'],
  over4m: [(c, b) => _HoldersAll(c, b, _LthSupply), 'over_4m'],
  over6m: [(c, b) => _HoldersAll(c, b, _LthSupply), 'over_6m'],
});

/**
 * @template A
 * @typedef {{
 *   _2009: A,
 *   _2010: A,
 *   _2011: A,
 *   _2012: A,
 *   _2013: A,
 *   _2014: A,
 *   _2015: A,
 *   _2016: A,
 *   _2017: A,
 *   _2018: A,
 *   _2019: A,
 *   _2020: A,
 *   _2021: A,
 *   _2022: A,
 *   _2023: A,
 *   _2024: A,
 *   _2025: A,
 *   _2026: A,
 * }} CoindaysDestroyedClass
 */
/** @type {_Make} */
const _CoindaysDestroyedClass = (c, b, f0) => _n(c, b, {
  _2009: [f0, 'class_2009_*'],
  _2010: [f0, 'class_2010_*'],
  _2011: [f0, 'class_2011_*'],
  _2012: [f0, 'class_2012_*'],
  _2013: [f0, 'class_2013_*'],
  _2014: [f0, 'class_2014_*'],
  _2015: [f0, 'class_2015_*'],
  _2016: [f0, 'class_2016_*'],
  _2017: [f0, 'class_2017_*'],
  _2018: [f0, 'class_2018_*'],
  _2019: [f0, 'class_2019_*'],
  _2020: [f0, 'class_2020_*'],
  _2021: [f0, 'class_2021_*'],
  _2022: [f0, 'class_2022_*'],
  _2023: [f0, 'class_2023_*'],
  _2024: [f0, 'class_2024_*'],
  _2025: [f0, 'class_2025_*'],
  _2026: [f0, 'class_2026_*'],
});

/**
 * @template A
 * @typedef {{
 *   _0: A,
 *   _1: A,
 *   _2: A,
 *   _3: A,
 *   _4: A,
 * }} CoindaysDestroyedEpoch
 */
/** @type {_Make} */
const _CoindaysDestroyedEpoch = (c, b, f0) => _n(c, b, {
  _0: [f0, 'epoch_0_*'],
  _1: [f0, 'epoch_1_*'],
  _2: [f0, 'epoch_2_*'],
  _3: [f0, 'epoch_3_*'],
  _4: [f0, 'epoch_4_*'],
});

/**
 * @typedef {{
 *   blocksMined: CoinblocksDestroyed<Count>,
 *   dominance: Dominance,
 * }} Aaopool
 */
const _Aaopool = _s({
  blocksMined: [_CoinblocksDestroyed, '*_blocks_mined'],
  dominance: [_Dominance, '*_dominance'],
});

/**
 * @typedef {{
 *   blockfills: Aaopool,
 *   ultimuspool: Aaopool,
 *   terrapool: Aaopool,
 *   onethash: Aaopool,
 *   bitfarms: Aaopool,
 *   huobipool: Aaopool,
 *   wayicn: Aaopool,
 *   canoepool: Aaopool,
 *   bitcoincom: Aaopool,
 *   pool175btc: Aaopool,
 *   gbminers: Aaopool,
 *   axbt: Aaopool,
 *   asicminer: Aaopool,
 *   bitminter: Aaopool,
 *   bitcoinrussia: Aaopool,
 *   btcserv: Aaopool,
 *   simplecoinus: Aaopool,
 *   ozcoin: Aaopool,
 *   eclipsemc: Aaopool,
 *   maxbtc: Aaopool,
 *   triplemining: Aaopool,
 *   coinlab: Aaopool,
 *   pool50btc: Aaopool,
 *   ghashio: Aaopool,
 *   stminingcorp: Aaopool,
 *   bitparking: Aaopool,
 *   mmpool: Aaopool,
 *   polmine: Aaopool,
 *   kncminer: Aaopool,
 *   bitalo: Aaopool,
 *   hhtt: Aaopool,
 *   megabigpower: Aaopool,
 *   mtred: Aaopool,
 *   nmcbit: Aaopool,
 *   yourbtcnet: Aaopool,
 *   givemecoins: Aaopool,
 *   multicoinco: Aaopool,
 *   bcpoolio: Aaopool,
 *   cointerra: Aaopool,
 *   kanopool: Aaopool,
 *   solock: Aaopool,
 *   ckpool: Aaopool,
 *   nicehash: Aaopool,
 *   bitclub: Aaopool,
 *   bitcoinaffiliatenetwork: Aaopool,
 *   exxbw: Aaopool,
 *   bitsolo: Aaopool,
 *   twentyoneinc: Aaopool,
 *   digitalbtc: Aaopool,
 *   eightbaochi: Aaopool,
 *   mybtccoinpool: Aaopool,
 *   tbdice: Aaopool,
 *   hashpool: Aaopool,
 *   nexious: Aaopool,
 *   bravomining: Aaopool,
 *   hotpool: Aaopool,
 *   okexpool: Aaopool,
 *   bcmonster: Aaopool,
 *   onehash: Aaopool,
 *   bixin: Aaopool,
 *   tatmaspool: Aaopool,
 *   connectbtc: Aaopool,
 *   batpool: Aaopool,
 *   waterhole: Aaopool,
 *   dcexploration: Aaopool,
 *   dcex: Aaopool,
 *   btpool: Aaopool,
 *   fiftyeightcoin: Aaopool,
 *   bitcoinindia: Aaopool,
 *   shawnp0wers: Aaopool,
 *   phashio: Aaopool,
 *   rigpool: Aaopool,
 *   haozhuzhu: Aaopool,
 *   sevenpool: Aaopool,
 *   miningkings: Aaopool,
 *   hashbx: Aaopool,
 *   dpool: Aaopool,
 *   rawpool: Aaopool,
 *   haominer: Aaopool,
 *   helix: Aaopool,
 *   bitcoinukraine: Aaopool,
 *   secretsuperstar: Aaopool,
 *   tigerpoolnet: Aaopool,
 *   sigmapoolcom: Aaopool,
 *   okpooltop: Aaopool,
 *   hummerpool: Aaopool,
 *   tangpool: Aaopool,
 *   bytepool: Aaopool,
 *   novablock: Aaopool,
 *   miningcity: Aaopool,
 *   minerium: Aaopool,
 *   lubiancom: Aaopool,
 *   okkong: Aaopool,
 *   aaopool: Aaopool,
 *   emcdpool: Aaopool,
 *   arkpool: Aaopool,
 *   purebtccom: Aaopool,
 *   kucoinpool: Aaopool,
 *   entrustcharitypool: Aaopool,
 *   okminer: Aaopool,
 *   titan: Aaopool,
 *   pegapool: Aaopool,
 *   btcnuggets: Aaopool,
 *   cloudhashing: Aaopool,
 *   digitalxmintsy: Aaopool,
 *   telco214: Aaopool,
 *   btcpoolparty: Aaopool,
 *   multipool: Aaopool,
 *   transactioncoinmining: Aaopool,
 *   btcdig: Aaopool,
 *   trickysbtcpool: Aaopool,
 *   btcmp: Aaopool,
 *   eobot: Aaopool,
 *   unomp: Aaopool,
 *   patels: Aaopool,
 *   gogreenlight: Aaopool,
 *   bitcoinindiapool: Aaopool,
 *   ekanembtc: Aaopool,
 *   canoe: Aaopool,
 *   tiger: Aaopool,
 *   onem1x: Aaopool,
 *   zulupool: Aaopool,
 *   wiz: Aaopool,
 *   wk057: Aaopool,
 *   futurebitapollosolo: Aaopool,
 *   carbonnegative: Aaopool,
 *   portlandhodl: Aaopool,
 *   phoenix: Aaopool,
 *   neopool: Aaopool,
 *   maxipool: Aaopool,
 *   bitfufupool: Aaopool,
 *   gdpool: Aaopool,
 *   miningdutch: Aaopool,
 *   publicpool: Aaopool,
 *   miningsquared: Aaopool,
 *   innopolistech: Aaopool,
 *   btclab: Aaopool,
 *   parasite: Aaopool,
 *   redrockpool: Aaopool,
 *   est3lar: Aaopool,
 *   braiinssolo: Aaopool,
 *   solopool: Aaopool,
 *   noderunners: Aaopool,
 *   dmnd: Aaopool,
 * }} Minor
 */
const _Minor = _s({
  blockfills: [_Aaopool, '*'],
  ultimuspool: [_Aaopool, 'ultimuspool'],
  terrapool: [_Aaopool, 'terrapool'],
  onethash: [_Aaopool, 'onethash'],
  bitfarms: [_Aaopool, 'bitfarms'],
  huobipool: [_Aaopool, 'huobipool'],
  wayicn: [_Aaopool, 'wayicn'],
  canoepool: [_Aaopool, 'canoepool'],
  bitcoincom: [_Aaopool, 'bitcoincom'],
  pool175btc: [_Aaopool, 'pool175btc'],
  gbminers: [_Aaopool, 'gbminers'],
  axbt: [_Aaopool, 'axbt'],
  asicminer: [_Aaopool, 'asicminer'],
  bitminter: [_Aaopool, 'bitminter'],
  bitcoinrussia: [_Aaopool, 'bitcoinrussia'],
  btcserv: [_Aaopool, 'btcserv'],
  simplecoinus: [_Aaopool, 'simplecoinus'],
  ozcoin: [_Aaopool, 'ozcoin'],
  eclipsemc: [_Aaopool, 'eclipsemc'],
  maxbtc: [_Aaopool, 'maxbtc'],
  triplemining: [_Aaopool, 'triplemining'],
  coinlab: [_Aaopool, 'coinlab'],
  pool50btc: [_Aaopool, 'pool50btc'],
  ghashio: [_Aaopool, 'ghashio'],
  stminingcorp: [_Aaopool, 'stminingcorp'],
  bitparking: [_Aaopool, 'bitparking'],
  mmpool: [_Aaopool, 'mmpool'],
  polmine: [_Aaopool, 'polmine'],
  kncminer: [_Aaopool, 'kncminer'],
  bitalo: [_Aaopool, 'bitalo'],
  hhtt: [_Aaopool, 'hhtt'],
  megabigpower: [_Aaopool, 'megabigpower'],
  mtred: [_Aaopool, 'mtred'],
  nmcbit: [_Aaopool, 'nmcbit'],
  yourbtcnet: [_Aaopool, 'yourbtcnet'],
  givemecoins: [_Aaopool, 'givemecoins'],
  multicoinco: [_Aaopool, 'multicoinco'],
  bcpoolio: [_Aaopool, 'bcpoolio'],
  cointerra: [_Aaopool, 'cointerra'],
  kanopool: [_Aaopool, 'kanopool'],
  solock: [_Aaopool, 'solock'],
  ckpool: [_Aaopool, 'ckpool'],
  nicehash: [_Aaopool, 'nicehash'],
  bitclub: [_Aaopool, 'bitclub'],
  bitcoinaffiliatenetwork: [_Aaopool, 'bitcoinaffiliatenetwork'],
  exxbw: [_Aaopool, 'exxbw'],
  bitsolo: [_Aaopool, 'bitsolo'],
  twentyoneinc: [_Aaopool, 'twentyoneinc'],
  digitalbtc: [_Aaopool, 'digitalbtc'],
  eightbaochi: [_Aaopool, 'eightbaochi'],
  mybtccoinpool: [_Aaopool, 'mybtccoinpool'],
  tbdice: [_Aaopool, 'tbdice'],
  hashpool: [_Aaopool, 'hashpool'],
  nexious: [_Aaopool, 'nexious'],
  bravomining: [_Aaopool, 'bravomining'],
  hotpool: [_Aaopool, 'hotpool'],
  okexpool: [_Aaopool, 'okexpool'],
  bcmonster: [_Aaopool, 'bcmonster'],
  onehash: [_Aaopool, 'onehash'],
  bixin: [_Aaopool, 'bixin'],
  tatmaspool: [_Aaopool, 'tatmaspool'],
  connectbtc: [_Aaopool, 'connectbtc'],
  batpool: [_Aaopool, 'batpool'],
  waterhole: [_Aaopool, 'waterhole'],
  dcexploration: [_Aaopool, 'dcexploration'],
  dcex: [_Aaopool, 'dcex'],
  btpool: [_Aaopool, 'btpool'],
  fiftyeightcoin: [_Aaopool, 'fiftyeightcoin'],
  bitcoinindia: [_Aaopool, 'bitcoinindia'],
  shawnp0wers: [_Aaopool, 'shawnp0wers'],
  phashio: [_Aaopool, 'phashio'],
  rigpool: [_Aaopool, 'rigpool'],
  haozhuzhu: [_Aaopool, 'haozhuzhu'],
  sevenpool: [_Aaopool, 'sevenpool'],
  miningkings: [_Aaopool, 'miningkings'],
  hashbx: [_Aaopool, 'hashbx'],
  dpool: [_Aaopool, 'dpool'],
  rawpool: [_Aaopool, 'rawpool'],
  haominer: [_Aaopool, 'haominer'],
  helix: [_Aaopool, 'helix'],
  bitcoinukraine: [_Aaopool, 'bitcoinukraine'],
  secretsuperstar: [_Aaopool, 'secretsuperstar'],
  tigerpoolnet: [_Aaopool, 'tigerpoolnet'],
  sigmapoolcom: [_Aaopool, 'sigmapoolcom'],
  okpooltop: [_Aaopool, 'okpooltop'],
  hummerpool: [_Aaopool, 'hummerpool'],
  tangpool: [_Aaopool, 'tangpool'],
  bytepool: [_Aaopool, 'bytepool'],
  novablock: [_Aaopool, 'novablock'],
  miningcity: [_Aaopool, 'miningcity'],
  minerium: [_Aaopool, 'minerium'],
  lubiancom: [_Aaopool, 'lubiancom'],
  okkong: [_Aaopool, 'okkong'],
  aaopool: [_Aaopool, 'aaopool'],
  emcdpool: [_Aaopool, 'emcdpool'],
  arkpool: [_Aaopool, 'arkpool'],
  purebtccom: [_Aaopool, 'purebtccom'],
  kucoinpool: [_Aaopool, 'kucoinpool'],
  entrustcharitypool: [_Aaopool, 'entrustcharitypool'],
  okminer: [_Aaopool, 'okminer'],
  titan: [_Aaopool, 'titan'],
  pegapool: [_Aaopool, 'pegapool'],
  btcnuggets: [_Aaopool, 'btcnuggets'],
  cloudhashing: [_Aaopool, 'cloudhashing'],
  digitalxmintsy: [_Aaopool, 'digitalxmintsy'],
  telco214: [_Aaopool, 'telco214'],
  btcpoolparty: [_Aaopool, 'btcpoolparty'],
  multipool: [_Aaopool, 'multipool'],
  transactioncoinmining: [_Aaopool, 'transactioncoinmining'],
  btcdig: [_Aaopool, 'btcdig'],
  trickysbtcpool: [_Aaopool, 'trickysbtcpool'],
  btcmp: [_Aaopool, 'btcmp'],
  eobot: [_Aaopool, 'eobot'],
  unomp: [_Aaopool, 'unomp'],
  patels: [_Aaopool, 'patels'],
  gogreenlight: [_Aaopool, 'gogreenlight'],
  bitcoinindiapool: [_Aaopool, 'bitcoinindiapool'],
  ekanembtc: [_Aaopool, 'ekanembtc'],
  canoe: [_Aaopool, 'canoe'],
  tiger: [_Aaopool, 'tiger'],
  onem1x: [_Aaopool, 'onem1x'],
  zulupool: [_Aaopool, 'zulupool'],
  wiz: [_Aaopool, 'wiz'],
  wk057: [_Aaopool, 'wk057'],
  futurebitapollosolo: [_Aaopool, 'futurebitapollosolo'],
  carbonnegative: [_Aaopool, 'carbonnegative'],
  portlandhodl: [_Aaopool, 'portlandhodl'],
  phoenix: [_Aaopool, 'phoenix'],
  neopool: [_Aaopool, 'neopool'],
  maxipool: [_Aaopool, 'maxipool'],
  bitfufupool: [_Aaopool, 'bitfufupool'],
  gdpool: [_Aaopool, 'gdpool'],
  miningdutch: [_Aaopool, 'miningdutch'],
  publicpool: [_Aaopool, 'publicpool'],
  miningsquared: [_Aaopool, 'miningsquared'],
  innopolistech: [_Aaopool, 'innopolistech'],
  btclab: [_Aaopool, 'btclab'],
  parasite: [_Aaopool, 'parasite'],
  redrockpool: [_Aaopool, 'redrockpool'],
  est3lar: [_Aaopool, 'est3lar'],
  braiinssolo: [_Aaopool, 'braiinssolo'],
  solopool: [_Aaopool, 'solopool'],
  noderunners: [_Aaopool, 'noderunners'],
  dmnd: [_Aaopool, 'dmnd'],
});

/**
 * @typedef {{
 *   blocksMined: CoinblocksDestroyed<Count>,
 *   dominance: Share,
 *   rewards: TransferVolume0sats,
 * }} Antpool
 */
const _Antpool = _s({
  blocksMined: [_CoinblocksDestroyed, '*_blocks_mined'],
  dominance: [_Share, '*_dominance'],
  rewards: [_TransferVolume0sats, '*_rewards'],
});

/**
 * @typedef {{
 *   unknown: Antpool,
 *   luxor: Antpool,
 *   btccom: Antpool,
 *   btctop: Antpool,
 *   btcguild: Antpool,
 *   eligius: Antpool,
 *   f2pool: Antpool,
 *   braiinspool: Antpool,
 *   antpool: Antpool,
 *   btcc: Antpool,
 *   bwpool: Antpool,
 *   bitfury: Antpool,
 *   viabtc: Antpool,
 *   poolin: Antpool,
 *   spiderpool: Antpool,
 *   binancepool: Antpool,
 *   foundryusa: Antpool,
 *   sbicrypto: Antpool,
 *   marapool: Antpool,
 *   secpool: Antpool,
 *   ocean: Antpool,
 *   whitepool: Antpool,
 * }} Major
 */
const _Major = _s({
  unknown: [_Antpool, '*'],
  luxor: [_Antpool, 'luxor'],
  btccom: [_Antpool, 'btccom'],
  btctop: [_Antpool, 'btctop'],
  btcguild: [_Antpool, 'btcguild'],
  eligius: [_Antpool, 'eligius'],
  f2pool: [_Antpool, 'f2pool'],
  braiinspool: [_Antpool, 'braiinspool'],
  antpool: [_Antpool, 'antpool'],
  btcc: [_Antpool, 'btcc'],
  bwpool: [_Antpool, 'bwpool'],
  bitfury: [_Antpool, 'bitfury'],
  viabtc: [_Antpool, 'viabtc'],
  poolin: [_Antpool, 'poolin'],
  spiderpool: [_Antpool, 'spiderpool'],
  binancepool: [_Antpool, 'binancepool'],
  foundryusa: [_Antpool, 'foundryusa'],
  sbicrypto: [_Antpool, 'sbicrypto'],
  marapool: [_Antpool, 'marapool'],
  secpool: [_Antpool, 'secpool'],
  ocean: [_Antpool, 'ocean'],
  whitepool: [_Antpool, 'whitepool'],
});

/**
 * @typedef {{
 *   pool: SeriesPattern21<PoolSlug>,
 *   major: Major,
 *   minor: Minor,
 * }} Pools
 */
const _Pools = _s({
  pool: [_i21, '*'],
  major: [_Major, 'unknown'],
  minor: [_Minor, 'blockfills'],
});

/**
 * @typedef {{
 *   destroyed: CoinblocksDestroyed<?Float64>,
 *   created: CoinblocksDestroyed<?Float64>,
 *   stored: CoinblocksDestroyed<?Float64>,
 *   vocdd: CoinblocksDestroyed<?Float64>,
 * }} Value
 */
const _Value = _s({
  destroyed: [_CoinblocksDestroyed, '*_destroyed'],
  created: [_CoinblocksDestroyed, '*_created'],
  stored: [_CoinblocksDestroyed, '*_stored'],
  vocdd: [_CoinblocksDestroyed, 'vocdd'],
});

/**
 * @template A
 * @typedef {{
 *   under1h: A,
 *   _1hTo1d: A,
 *   _1dTo1w: A,
 *   _1wTo1m: A,
 *   _1mTo2m: A,
 *   _2mTo3m: A,
 *   _3mTo4m: A,
 *   _4mTo5m: A,
 *   _5mTo6m: A,
 *   _6mTo9m: A,
 *   _9mTo1y: A,
 *   _1yTo18m: A,
 *   _18mTo2y: A,
 *   _2yTo3y: A,
 *   _3yTo4y: A,
 *   _4yTo5y: A,
 *   _5yTo6y: A,
 *   _6yTo7y: A,
 *   _7yTo8y: A,
 *   _8yTo10y: A,
 *   _10yTo12y: A,
 *   _12yTo15y: A,
 *   over15y: A,
 * }} CoindaysCreated
 */
/** @type {_Make} */
const _CoindaysCreated = (c, b, f0) => _n(c, b, {
  under1h: [f0, 'utxos_under_1h_*'],
  _1hTo1d: [f0, 'utxos_1h_to_1d_*'],
  _1dTo1w: [f0, 'utxos_1d_to_1w_*'],
  _1wTo1m: [f0, 'utxos_1w_to_1m_*'],
  _1mTo2m: [f0, 'utxos_1m_to_2m_*'],
  _2mTo3m: [f0, 'utxos_2m_to_3m_*'],
  _3mTo4m: [f0, 'utxos_3m_to_4m_*'],
  _4mTo5m: [f0, 'utxos_4m_to_5m_*'],
  _5mTo6m: [f0, 'utxos_5m_to_6m_*'],
  _6mTo9m: [f0, 'utxos_6m_to_9m_*'],
  _9mTo1y: [f0, 'utxos_9m_to_1y_*'],
  _1yTo18m: [f0, 'utxos_1y_to_18m_*'],
  _18mTo2y: [f0, 'utxos_18m_to_2y_*'],
  _2yTo3y: [f0, 'utxos_2y_to_3y_*'],
  _3yTo4y: [f0, 'utxos_3y_to_4y_*'],
  _4yTo5y: [f0, 'utxos_4y_to_5y_*'],
  _5yTo6y: [f0, 'utxos_5y_to_6y_*'],
  _6yTo7y: [f0, 'utxos_6y_to_7y_*'],
  _7yTo8y: [f0, 'utxos_7y_to_8y_*'],
  _8yTo10y: [f0, 'utxos_8y_to_10y_*'],
  _10yTo12y: [f0, 'utxos_10y_to_12y_*'],
  _12yTo15y: [f0, 'utxos_12y_to_15y_*'],
  over15y: [f0, 'utxos_over_15y_*'],
});

/**
 * @template A, B, C
 * @typedef {{
 *   age: A,
 *   epoch: B,
 *   class: C,
 * }} CoindaysDestroyed
 */
/** @type {_Make} */
const _CoindaysDestroyed = (c, b, f0, f1, f2) => _n(c, b, {
  age: [f0, 'old_*'],
  epoch: [f1, '*'],
  class: [f2, '*'],
});

/**
 * @typedef {{
 *   profit: CoindaysDestroyed<SpendingRate<?Dollars>, CapEpoch<?Dollars>, CapClass<?Dollars>>,
 *   loss: CoindaysDestroyed<SpendingRate<?Dollars>, CapEpoch<?Dollars>, CapClass<?Dollars>>,
 *   netPnl: CoindaysDestroyed<SpendingRate<?Dollars>, CapEpoch<?Dollars>, CapClass<?Dollars>>,
 * }} AgeUnrealized
 */
const _AgeUnrealized = _s({
  profit: [(c, b) => _CoindaysDestroyed(c, b, _SpendingRate, _CapEpoch, _CapClass), '*_profit'],
  loss: [(c, b) => _CoindaysDestroyed(c, b, _SpendingRate, _CapEpoch, _CapClass), '*_loss'],
  netPnl: [(c, b) => _CoindaysDestroyed(c, b, _SpendingRate, _CapEpoch, _CapClass), 'net_*_pnl'],
});

/**
 * @typedef {{
 *   cap: CoindaysDestroyed<SpendingRate<?Dollars>, CapEpoch<?Dollars>, CapClass<?Dollars>>,
 *   price: RealizedPrice,
 *   profit: CoindaysDestroyed<CoindaysCreated<CoinblocksDestroyed<?Dollars>>, CoindaysDestroyedEpoch<CoinblocksDestroyed<?Dollars>>, CoindaysDestroyedClass<CoinblocksDestroyed<?Dollars>>>,
 *   loss: CoindaysDestroyed<CoindaysCreated<CoinblocksDestroyed<?Dollars>>, CoindaysDestroyedEpoch<CoinblocksDestroyed<?Dollars>>, CoindaysDestroyedClass<CoinblocksDestroyed<?Dollars>>>,
 *   netPnl: CoindaysDestroyed<CoindaysCreated<NetPnl>, CoindaysDestroyedEpoch<NetPnl>, CoindaysDestroyedClass<NetPnl>>,
 *   valueDestroyed: CoindaysDestroyed<CoindaysCreated<CoinblocksDestroyed<?Dollars>>, CoindaysDestroyedEpoch<CoinblocksDestroyed<?Dollars>>, CoindaysDestroyedClass<CoinblocksDestroyed<?Dollars>>>,
 * }} AgeRealized
 */
const _AgeRealized = _s({
  cap: [(c, b) => _CoindaysDestroyed(c, b, _SpendingRate, _CapEpoch, _CapClass), '*_cap'],
  price: [_RealizedPrice, 'old_*_price'],
  profit: [(c, b) => _CoindaysDestroyed(c, b, (c, b) => _CoindaysCreated(c, b, _CoinblocksDestroyed), (c, b) => _CoindaysDestroyedEpoch(c, b, _CoinblocksDestroyed), (c, b) => _CoindaysDestroyedClass(c, b, _CoinblocksDestroyed)), '*_profit'],
  loss: [(c, b) => _CoindaysDestroyed(c, b, (c, b) => _CoindaysCreated(c, b, _CoinblocksDestroyed), (c, b) => _CoindaysDestroyedEpoch(c, b, _CoinblocksDestroyed), (c, b) => _CoindaysDestroyedClass(c, b, _CoinblocksDestroyed)), '*_loss'],
  netPnl: [(c, b) => _CoindaysDestroyed(c, b, (c, b) => _CoindaysCreated(c, b, _NetPnl), (c, b) => _CoindaysDestroyedEpoch(c, b, _NetPnl), (c, b) => _CoindaysDestroyedClass(c, b, _NetPnl)), 'net_*_pnl'],
  valueDestroyed: [(c, b) => _CoindaysDestroyed(c, b, (c, b) => _CoindaysCreated(c, b, _CoinblocksDestroyed), (c, b) => _CoindaysDestroyedEpoch(c, b, _CoinblocksDestroyed), (c, b) => _CoindaysDestroyedClass(c, b, _CoinblocksDestroyed)), 'value_destroyed'],
});

/**
 * @typedef {{
 *   age: CoindaysCreated<TransferVolume0sats>,
 *   epoch: CoindaysDestroyedEpoch<TransferVolume0sats>,
 *   class: CoindaysDestroyedClass<TransferVolume0sats>,
 *   inProfit: CoindaysDestroyed<CoindaysCreated<TransferVolume0sats>, CoindaysDestroyedEpoch<TransferVolume0sats>, CoindaysDestroyedClass<TransferVolume0sats>>,
 *   inLoss: CoindaysDestroyed<CoindaysCreated<TransferVolume0sats>, CoindaysDestroyedEpoch<TransferVolume0sats>, CoindaysDestroyedClass<TransferVolume0sats>>,
 * }} AgeActivityTransferVolume
 */
const _AgeActivityTransferVolume = _s({
  age: [(c, b) => _CoindaysCreated(c, b, _TransferVolume0sats), 'old_*'],
  epoch: [(c, b) => _CoindaysDestroyedEpoch(c, b, _TransferVolume0sats), '*'],
  class: [(c, b) => _CoindaysDestroyedClass(c, b, _TransferVolume0sats), '*'],
  inProfit: [(c, b) => _CoindaysDestroyed(c, b, (c, b) => _CoindaysCreated(c, b, _TransferVolume0sats), (c, b) => _CoindaysDestroyedEpoch(c, b, _TransferVolume0sats), (c, b) => _CoindaysDestroyedClass(c, b, _TransferVolume0sats)), '*_in_profit'],
  inLoss: [(c, b) => _CoindaysDestroyed(c, b, (c, b) => _CoindaysCreated(c, b, _TransferVolume0sats), (c, b) => _CoindaysDestroyedEpoch(c, b, _TransferVolume0sats), (c, b) => _CoindaysDestroyedClass(c, b, _TransferVolume0sats)), '*_in_loss'],
});

/**
 * @typedef {{
 *   transferVolume: AgeActivityTransferVolume,
 *   coindaysDestroyed: CoindaysDestroyed<CoindaysCreated<CoinblocksDestroyed<?CoinDays>>, CoindaysDestroyedEpoch<CoinblocksDestroyed<?CoinDays>>, CoindaysDestroyedClass<CoinblocksDestroyed<?CoinDays>>>,
 * }} AgeActivity
 */
const _AgeActivity = _s({
  transferVolume: [_AgeActivityTransferVolume, '*'],
  coindaysDestroyed: [(c, b) => _CoindaysDestroyed(c, b, (c, b) => _CoindaysCreated(c, b, _CoinblocksDestroyed), (c, b) => _CoindaysDestroyedEpoch(c, b, _CoinblocksDestroyed), (c, b) => _CoindaysDestroyedClass(c, b, _CoinblocksDestroyed)), 'coindays_destroyed'],
});

/**
 * @typedef {{
 *   unspentCount: CoindaysDestroyed<CoindaysCreated<MarketCap<Count, CountSigned>>, CoindaysDestroyedEpoch<MarketCap<Count, CountSigned>>, CoindaysDestroyedClass<MarketCap<Count, CountSigned>>>,
 *   spentCount: CoindaysDestroyed<CoindaysCreated<CoinblocksDestroyed<Count>>, CoindaysDestroyedEpoch<CoinblocksDestroyed<Count>>, CoindaysDestroyedClass<CoinblocksDestroyed<Count>>>,
 * }} AgeOutputs
 */
const _AgeOutputs = _s({
  unspentCount: [(c, b) => _CoindaysDestroyed(c, b, (c, b) => _CoindaysCreated(c, b, _MarketCap), (c, b) => _CoindaysDestroyedEpoch(c, b, _MarketCap), (c, b) => _CoindaysDestroyedClass(c, b, _MarketCap)), '*'],
  spentCount: [(c, b) => _CoindaysDestroyed(c, b, (c, b) => _CoindaysCreated(c, b, _CoinblocksDestroyed), (c, b) => _CoindaysDestroyedEpoch(c, b, _CoinblocksDestroyed), (c, b) => _CoindaysDestroyedClass(c, b, _CoinblocksDestroyed)), 'spent_*'],
});

/**
 * @typedef {{
 *   total: CoindaysDestroyed<CoindaysCreated<HodledOrLost>, CoindaysDestroyedEpoch<HodledOrLost>, CoindaysDestroyedClass<HodledOrLost>>,
 *   matured: CoindaysCreated<TransferVolume0sats>,
 *   inProfit: CoindaysDestroyed<CoindaysCreated<HodledOrLost>, CoindaysDestroyedEpoch<HodledOrLost>, CoindaysDestroyedClass<HodledOrLost>>,
 *   inLoss: CoindaysDestroyed<CoindaysCreated<HodledOrLost>, CoindaysDestroyedEpoch<HodledOrLost>, CoindaysDestroyedClass<HodledOrLost>>,
 *   delta: CoindaysDestroyed<CoindaysCreated<DeltaAll<?Bitcoin>>, CoindaysDestroyedEpoch<DeltaAll<?Bitcoin>>, CoindaysDestroyedClass<DeltaAll<?Bitcoin>>>,
 *   dominance: CoindaysDestroyed<SpendingRate<?Percent>, CapEpoch<?Percent>, CapClass<?Percent>>,
 * }} AgeSupply
 */
const _AgeSupply = _s({
  total: [(c, b) => _CoindaysDestroyed(c, b, (c, b) => _CoindaysCreated(c, b, _HodledOrLost), (c, b) => _CoindaysDestroyedEpoch(c, b, _HodledOrLost), (c, b) => _CoindaysDestroyedClass(c, b, _HodledOrLost)), '*'],
  matured: [(c, b) => _CoindaysCreated(c, b, _TransferVolume0sats), 'old_matured_*'],
  inProfit: [(c, b) => _CoindaysDestroyed(c, b, (c, b) => _CoindaysCreated(c, b, _HodledOrLost), (c, b) => _CoindaysDestroyedEpoch(c, b, _HodledOrLost), (c, b) => _CoindaysDestroyedClass(c, b, _HodledOrLost)), '*_in_profit'],
  inLoss: [(c, b) => _CoindaysDestroyed(c, b, (c, b) => _CoindaysCreated(c, b, _HodledOrLost), (c, b) => _CoindaysDestroyedEpoch(c, b, _HodledOrLost), (c, b) => _CoindaysDestroyedClass(c, b, _HodledOrLost)), '*_in_loss'],
  delta: [(c, b) => _CoindaysDestroyed(c, b, (c, b) => _CoindaysCreated(c, b, _DeltaAll), (c, b) => _CoindaysDestroyedEpoch(c, b, _DeltaAll), (c, b) => _CoindaysDestroyedClass(c, b, _DeltaAll)), '*_delta'],
  dominance: [(c, b) => _CoindaysDestroyed(c, b, _SpendingRate, _CapEpoch, _CapClass), '*_dominance'],
});

/**
 * @typedef {{
 *   supply: AgeSupply,
 *   outputs: AgeOutputs,
 *   activity: AgeActivity,
 *   realized: AgeRealized,
 *   unrealized: AgeUnrealized,
 *   ageBounds: AgeBounds,
 *   coindaysCreated: CoindaysCreated<CoinblocksDestroyed<?CoinDays>>,
 *   coinblocksDestroyed: CoinblocksDestroyed<?CoinBlocks>,
 * }} Age
 */
const _Age = _s({
  supply: [_AgeSupply, '*'],
  outputs: [_AgeOutputs, 'utxo_count'],
  activity: [_AgeActivity, 'transfer_volume'],
  realized: [_AgeRealized, 'realized'],
  unrealized: [_AgeUnrealized, 'unrealized'],
  ageBounds: [_AgeBounds, 'utxos_urpd'],
  coindaysCreated: [(c, b) => _CoindaysCreated(c, b, _CoinblocksDestroyed), 'old_coindays_created'],
  coinblocksDestroyed: [_CoinblocksDestroyed, 'coinblocks_destroyed'],
});

/**
 * @typedef {{
 *   mobile: CoindaysCreated<HodledOrLost>,
 *   immobile: CoindaysCreated<HodledOrLost>,
 * }} CoinflowAgeRangeSupply
 */
const _CoinflowAgeRangeSupply = _s({
  mobile: [(c, b) => _CoindaysCreated(c, b, _HodledOrLost), '*_mobile_supply'],
  immobile: [(c, b) => _CoindaysCreated(c, b, _HodledOrLost), '*_immobile_supply'],
});

/**
 * @typedef {{
 *   spendingRate: SpendingRate<?PerDay>,
 *   spendingExposure: SpendingExposure,
 *   supply: CoinflowAgeRangeSupply,
 * }} CoinflowAgeRange
 */
const _CoinflowAgeRange = _s({
  spendingRate: [_SpendingRate, '*_spending_rate'],
  spendingExposure: [_SpendingExposure, '*'],
  supply: [_CoinflowAgeRangeSupply, '*'],
});

/**
 * @typedef {{
 *   ageRange: CoinflowAgeRange,
 *   urpd: CoinflowUrpd<CoinflowUrpdLth>,
 *   supply: CoinflowSupply,
 *   cap: SeriesPattern2<?Dollars>,
 *   price: CapitalizedPrice,
 *   capitalizedPrice: CapitalizedPrice,
 *   sth: CoinflowLth,
 *   lth: CoinflowLth,
 *   under4mPrice: CapitalizedPrice,
 *   under4mCapitalizedPrice: CapitalizedPrice,
 *   under6mPrice: CapitalizedPrice,
 *   under6mCapitalizedPrice: CapitalizedPrice,
 *   over4mPrice: CapitalizedPrice,
 *   over4mCapitalizedPrice: CapitalizedPrice,
 *   over6mPrice: CapitalizedPrice,
 *   over6mCapitalizedPrice: CapitalizedPrice,
 * }} Coinflow
 */
const _Coinflow = _s({
  ageRange: [_CoinflowAgeRange, 'old'],
  urpd: [(c, b) => _CoinflowUrpd(c, b, _CoinflowUrpdLth), '*'],
  supply: [_CoinflowSupply, ''],
  cap: [_i2, '*_cap'],
  price: [_CapitalizedPrice, '*_price'],
  capitalizedPrice: [_CapitalizedPrice, '*_capitalized_price'],
  sth: [_CoinflowLth, 'sth'],
  lth: [_CoinflowLth, 'lth'],
  under4mPrice: [_CapitalizedPrice, 'under_4m_*_price'],
  under4mCapitalizedPrice: [_CapitalizedPrice, 'under_4m_*_capitalized_price'],
  under6mPrice: [_CapitalizedPrice, 'under_6m_*_price'],
  under6mCapitalizedPrice: [_CapitalizedPrice, 'under_6m_*_capitalized_price'],
  over4mPrice: [_CapitalizedPrice, 'over_4m_*_price'],
  over4mCapitalizedPrice: [_CapitalizedPrice, 'over_4m_*_capitalized_price'],
  over6mPrice: [_CapitalizedPrice, 'over_6m_*_price'],
  over6mCapitalizedPrice: [_CapitalizedPrice, 'over_6m_*_capitalized_price'],
});

/**
 * @typedef {{
 *   awake: CoindaysCreated<HodledOrLost>,
 *   dormant: CoindaysCreated<HodledOrLost>,
 * }} CointimeAgeRangeSupply
 */
const _CointimeAgeRangeSupply = _s({
  awake: [(c, b) => _CoindaysCreated(c, b, _HodledOrLost), '*_awake_supply'],
  dormant: [(c, b) => _CoindaysCreated(c, b, _HodledOrLost), '*_dormant_supply'],
});

/**
 * @typedef {{
 *   coindaysConsumed: CoindaysCreated<CoinblocksDestroyed<?CoinDays>>,
 *   coindaysStored: CoindaysCreated<CoinblocksDestroyed<?CoinDays>>,
 *   activity: AgeRangeActivity,
 *   supply: CointimeAgeRangeSupply,
 * }} CointimeAgeRange
 */
const _CointimeAgeRange = _s({
  coindaysConsumed: [(c, b) => _CoindaysCreated(c, b, _CoinblocksDestroyed), '*_coindays_consumed'],
  coindaysStored: [(c, b) => _CoindaysCreated(c, b, _CoinblocksDestroyed), '*_coindays_stored'],
  activity: [_AgeRangeActivity, '*'],
  supply: [_CointimeAgeRangeSupply, '*'],
});

/**
 * @typedef {{
 *   coinblocksCreated: CoinblocksDestroyed<?CoinBlocks>,
 *   coinblocksStored: CoinblocksDestroyed<?CoinBlocks>,
 *   liveliness: SeriesPattern2<?Ratio64>,
 *   vaultedness: SeriesPattern2<?Ratio64>,
 *   ratio: SeriesPattern2<?Ratio64>,
 * }} CointimeActivity
 */
const _CointimeActivity = _s({
  coinblocksCreated: [_CoinblocksDestroyed, '*_created'],
  coinblocksStored: [_CoinblocksDestroyed, '*_stored'],
  liveliness: [_i2, 'liveliness'],
  vaultedness: [_i2, 'vaultedness'],
  ratio: [_i2, 'activity_to_vaultedness'],
});

/**
 * @typedef {{
 *   activity: CointimeActivity,
 *   ageRange: CointimeAgeRange,
 *   urpd: CoinflowUrpd<CointimeUrpdLth>,
 *   awake: Awake,
 *   dormant: Dormant,
 *   sth: CointimeLth,
 *   lth: CointimeLth,
 *   under4mAwakePrice: CapitalizedPrice,
 *   under4mAwakeCapitalizedPrice: CapitalizedPrice,
 *   under6mAwakePrice: CapitalizedPrice,
 *   under6mAwakeCapitalizedPrice: CapitalizedPrice,
 *   over4mAwakePrice: CapitalizedPrice,
 *   over4mAwakeCapitalizedPrice: CapitalizedPrice,
 *   over6mAwakePrice: CapitalizedPrice,
 *   over6mAwakeCapitalizedPrice: CapitalizedPrice,
 *   supply: CointimeSupply,
 *   value: Value,
 *   cap: Cap,
 *   prices: CointimePrices,
 *   adjusted: Adjusted,
 *   reserveRisk: ReserveRisk,
 * }} Cointime
 */
const _Cointime = _s({
  activity: [_CointimeActivity, 'coinblocks'],
  ageRange: [_CointimeAgeRange, 'old'],
  urpd: [(c, b) => _CoinflowUrpd(c, b, _CointimeUrpdLth), 'cointime'],
  awake: [_Awake, '*'],
  dormant: [_Dormant, 'dormant_supply'],
  sth: [_CointimeLth, 'sth'],
  lth: [_CointimeLth, 'lth'],
  under4mAwakePrice: [_CapitalizedPrice, 'under_4m_*_price'],
  under4mAwakeCapitalizedPrice: [_CapitalizedPrice, 'under_4m_*_capitalized_price'],
  under6mAwakePrice: [_CapitalizedPrice, 'under_6m_*_price'],
  under6mAwakeCapitalizedPrice: [_CapitalizedPrice, 'under_6m_*_capitalized_price'],
  over4mAwakePrice: [_CapitalizedPrice, 'over_4m_*_price'],
  over4mAwakeCapitalizedPrice: [_CapitalizedPrice, 'over_4m_*_capitalized_price'],
  over6mAwakePrice: [_CapitalizedPrice, 'over_6m_*_price'],
  over6mAwakeCapitalizedPrice: [_CapitalizedPrice, 'over_6m_*_capitalized_price'],
  supply: [_CointimeSupply, 'supply'],
  value: [_Value, 'cointime_value'],
  cap: [_Cap, 'cap'],
  prices: [_CointimePrices, 'price'],
  adjusted: [_Adjusted, 'cointime_adj'],
  reserveRisk: [_ReserveRisk, 'reserve_risk'],
});

/**
 * @typedef {{
 *   v1: CoinblocksDestroyed<Count>,
 *   v2: CoinblocksDestroyed<Count>,
 *   v3: CoinblocksDestroyed<Count>,
 *   other: CoinblocksDestroyed<Count>,
 * }} Versions
 */
const _Versions = _s({
  v1: [_CoinblocksDestroyed, '*_v1'],
  v2: [_CoinblocksDestroyed, '*_v2'],
  v3: [_CoinblocksDestroyed, '*_v3'],
  other: [_CoinblocksDestroyed, '*_other_version'],
});

/**
 * @typedef {{
 *   total: CoinblocksDestroyed<SigOps64>,
 * }} Sigops
 */
const _Sigops = _s({
  total: [_CoinblocksDestroyed, '*'],
});

/**
 * @typedef {{
 *   nonstandard: CoinblocksDestroyed<Count>,
 * }} PolicyCount
 */
const _PolicyCount = _s({
  nonstandard: [_CoinblocksDestroyed, '*'],
});

/**
 * @typedef {{
 *   count: PolicyCount,
 *   isNonstandard: SeriesPattern22<Boolean>,
 * }} Policy
 */
const _Policy = _s({
  count: [_PolicyCount, '*_count'],
  isNonstandard: [_i22, 'is_*'],
});

/**
 * @typedef {{
 *   coinjoin: CoinblocksDestroyed<Count>,
 *   consolidation: CoinblocksDestroyed<Count>,
 *   batchPayout: CoinblocksDestroyed<Count>,
 * }} PatternsCount
 */
const _PatternsCount = _s({
  coinjoin: [_CoinblocksDestroyed, 'coinjoin_*'],
  consolidation: [_CoinblocksDestroyed, 'consolidation_*'],
  batchPayout: [_CoinblocksDestroyed, 'batch_payout_*'],
});

/**
 * @typedef {{
 *   count: PatternsCount,
 *   isCoinjoin: SeriesPattern22<Boolean>,
 *   isConsolidation: SeriesPattern22<Boolean>,
 *   isBatchPayout: SeriesPattern22<Boolean>,
 * }} Patterns
 */
const _Patterns = _s({
  count: [_PatternsCount, 'count'],
  isCoinjoin: [_i22, '*_coinjoin'],
  isConsolidation: [_i22, '*_consolidation'],
  isBatchPayout: [_i22, '*_batch_payout'],
});

/**
 * @typedef {{
 *   count: CoinblocksDestroyed<Count>,
 *   fees: TransferVolume0sats,
 *   feeShare: SeriesPattern2<?Percent>,
 * }} Inscription
 */
const _Inscription = _s({
  count: [_CoinblocksDestroyed, 'tx_count_*'],
  fees: [_TransferVolume0sats, '*_fees'],
  feeShare: [_i2, '*_fee_share'],
});

/**
 * @typedef {{
 *   cpfpParent: CoinblocksDestroyed<Count>,
 *   cpfpChild: CoinblocksDestroyed<Count>,
 * }} FeesCount
 */
const _FeesCount = _s({
  cpfpParent: [_CoinblocksDestroyed, '*_parent_count'],
  cpfpChild: [_CoinblocksDestroyed, '*_child_count'],
});

/**
 * @typedef {{
 *   count: FeesCount,
 *   fee: EffectiveFeeRate<Sats, EffectiveFeeRate6b<Sats>>,
 *   feeRate: SeriesPattern22<?FeeRate>,
 *   effectiveFeeRate: EffectiveFeeRate<?FeeRate, EffectiveFeeRate6b<?FeeRate>>,
 *   isCpfpParent: SeriesPattern22<Boolean>,
 *   isCpfpChild: SeriesPattern22<Boolean>,
 * }} TransactionsFees
 */
const _TransactionsFees = _s({
  count: [_FeesCount, 'cpfp'],
  fee: [(c, b) => _EffectiveFeeRate(c, b, _EffectiveFeeRate6b), '*'],
  feeRate: [_i22, '*_rate'],
  effectiveFeeRate: [(c, b) => _EffectiveFeeRate(c, b, _EffectiveFeeRate6b), 'effective_*_rate'],
  isCpfpParent: [_i22, 'is_cpfp_parent'],
  isCpfpChild: [_i22, 'is_cpfp_child'],
});

/**
 * @typedef {{
 *   annex: CoinblocksDestroyed<Count>,
 *   sighashAll: CoinblocksDestroyed<Count>,
 *   sighashNone: CoinblocksDestroyed<Count>,
 *   sighashSingle: CoinblocksDestroyed<Count>,
 *   sighashDefault: CoinblocksDestroyed<Count>,
 *   sighashAnyoneCanPay: CoinblocksDestroyed<Count>,
 *   dustOutput: CoinblocksDestroyed<Count>,
 * }} TransactionsFeaturesCount
 */
const _TransactionsFeaturesCount = _s({
  annex: [_CoinblocksDestroyed, '*_annex'],
  sighashAll: [_CoinblocksDestroyed, '*_sighash_all'],
  sighashNone: [_CoinblocksDestroyed, '*_sighash_none'],
  sighashSingle: [_CoinblocksDestroyed, '*_sighash_single'],
  sighashDefault: [_CoinblocksDestroyed, '*_sighash_default'],
  sighashAnyoneCanPay: [_CoinblocksDestroyed, '*_sighash_anyone_can_pay'],
  dustOutput: [_CoinblocksDestroyed, '*_dust_output'],
});

/**
 * @typedef {{
 *   count: TransactionsFeaturesCount,
 * }} TransactionsFeatures
 */
const _TransactionsFeatures = _s({
  count: [_TransactionsFeaturesCount, '*'],
});

/**
 * @typedef {{
 *   count: Vbytes<Count, ?CountFract, Count16>,
 *   features: TransactionsFeatures,
 *   size: TransactionsSize,
 *   fees: TransactionsFees,
 *   inscription: Inscription,
 *   patterns: Patterns,
 *   policy: Policy,
 *   sigops: Sigops,
 *   versions: Versions,
 *   volume: Volume,
 * }} Transactions
 */
const _Transactions = _s({
  count: [_Vbytes, '*_count'],
  features: [_TransactionsFeatures, '*_count'],
  size: [_TransactionsSize, '*'],
  fees: [_TransactionsFees, 'fee'],
  inscription: [_Inscription, 'inscription'],
  patterns: [_Patterns, 'is'],
  policy: [_Policy, 'nonstandard'],
  sigops: [_Sigops, 'total_sigop_cost'],
  versions: [_Versions, '*'],
  volume: [_Volume, '*'],
});

/**
 * @typedef {{
 *   _24h: SeriesPattern1<Count>,
 *   _1w: SeriesPattern1<Count>,
 *   _1m: SeriesPattern1<Count>,
 *   _1y: SeriesPattern1<Count>,
 * }} Target
 */
const _Target = _s({
  _24h: [_i1, '*_24h'],
  _1w: [_i1, '*_1w'],
  _1m: [_i1, '*_1m'],
  _1y: [_i1, '*_1y'],
});

/**
 * @typedef {{
 *   target: Target,
 *   total: CoinblocksDestroyed<Count>,
 * }} BlocksCount
 */
const _BlocksCount = _s({
  target: [_Target, '*_target'],
  total: [_CoinblocksDestroyed, '*'],
});

/**
 * @typedef {{
 *   count: BlocksCount,
 *   lookback: BlocksLookback,
 *   interval: Interval<Seconds, ?SecondsFract>,
 *   vbytes: Vbytes<VSize, ?VSizeFract, VSize>,
 *   size: BlocksSize<Bytes, ?BytesFract, Bytes32>,
 *   weight: BlocksSize<Weight64, ?WeightFract, Weight>,
 *   fullness: SeriesPattern21<?Percent>,
 *   difficulty: BlocksDifficulty,
 *   halving: BlocksHalving,
 * }} Blocks
 */
const _Blocks = _s({
  count: [_BlocksCount, '*_count'],
  lookback: [_BlocksLookback, 'height'],
  interval: [_Interval, '*_interval'],
  vbytes: [_Vbytes, '*_vbytes'],
  size: [_BlocksSize, '*_size'],
  weight: [_BlocksSize, '*_weight'],
  fullness: [_i21, '*_fullness'],
  difficulty: [_BlocksDifficulty, 'difficulty'],
  halving: [_BlocksHalving, 'halving'],
});

/**
 * @typedef {{
 *   firstIndex: SeriesPattern21<OpReturnIndex>,
 *   toTxIndex: SeriesPattern26<TxIndex>,
 *   kind: SeriesPattern26<OpReturnKind>,
 *   postOpReturnBytes: SeriesPattern26<Bytes32>,
 * }} IndexerOpReturn
 */
const _IndexerOpReturn = _s({
  firstIndex: [_i21, 'first_op_return_*'],
  toTxIndex: [_i26, 'tx_*'],
  kind: [_i26, 'kind'],
  postOpReturnBytes: [_i26, 'op_return_post_op_return_bytes'],
});

/**
 * @typedef {{
 *   firstIndex: SeriesPattern21<UnknownOutputIndex>,
 *   toTxIndex: SeriesPattern36<TxIndex>,
 *   legacySigops: SeriesPattern36<SigOps>,
 * }} ScriptsUnknown
 */
const _ScriptsUnknown = _s({
  firstIndex: [_i21, 'first_unknown_output_*'],
  toTxIndex: [_i36, 'tx_*'],
  legacySigops: [_i36, 'unknown_legacy_sigops'],
});

/**
 * @typedef {{
 *   firstIndex: SeriesPattern21<P2MSOutputIndex>,
 *   toTxIndex: SeriesPattern28<TxIndex>,
 *   legacySigops: SeriesPattern28<SigOps>,
 * }} ScriptsP2ms
 */
const _ScriptsP2ms = _s({
  firstIndex: [_i21, 'first_p2ms_output_*'],
  toTxIndex: [_i28, 'tx_*'],
  legacySigops: [_i28, 'p2ms_legacy_sigops'],
});

/**
 * @typedef {{
 *   firstIndex: SeriesPattern21<EmptyOutputIndex>,
 *   toTxIndex: SeriesPattern25<TxIndex>,
 * }} ScriptsEmpty
 */
const _ScriptsEmpty = _s({
  firstIndex: [_i21, 'first_empty_output_*'],
  toTxIndex: [_i25, 'tx_*'],
});

/**
 * @typedef {{
 *   empty: ScriptsEmpty,
 *   p2ms: ScriptsP2ms,
 *   unknown: ScriptsUnknown,
 * }} Scripts
 */
const _Scripts = _s({
  empty: [_ScriptsEmpty, '*'],
  p2ms: [_ScriptsP2ms, '*'],
  unknown: [_ScriptsUnknown, '*'],
});

/**
 * @typedef {{
 *   firstIndex: SeriesPattern21<P2AAddrIndex>,
 *   bytes: SeriesPattern27<P2ABytes>,
 * }} AddressesP2a
 */
const _AddressesP2a = _s({
  firstIndex: [_i21, 'first_*_addr_index'],
  bytes: [_i27, '*_bytes'],
});

/**
 * @typedef {{
 *   firstIndex: SeriesPattern21<P2TRAddrIndex>,
 *   bytes: SeriesPattern33<P2TRBytes>,
 * }} AddressesP2tr
 */
const _AddressesP2tr = _s({
  firstIndex: [_i21, 'first_*_addr_index'],
  bytes: [_i33, '*_bytes'],
});

/**
 * @typedef {{
 *   firstIndex: SeriesPattern21<P2WSHAddrIndex>,
 *   bytes: SeriesPattern35<P2WSHBytes>,
 * }} AddressesP2wsh
 */
const _AddressesP2wsh = _s({
  firstIndex: [_i21, 'first_*_addr_index'],
  bytes: [_i35, '*_bytes'],
});

/**
 * @typedef {{
 *   firstIndex: SeriesPattern21<P2WPKHAddrIndex>,
 *   bytes: SeriesPattern34<P2WPKHBytes>,
 * }} AddressesP2wpkh
 */
const _AddressesP2wpkh = _s({
  firstIndex: [_i21, 'first_*_addr_index'],
  bytes: [_i34, '*_bytes'],
});

/**
 * @typedef {{
 *   firstIndex: SeriesPattern21<P2SHAddrIndex>,
 *   bytes: SeriesPattern32<P2SHBytes>,
 * }} AddressesP2sh
 */
const _AddressesP2sh = _s({
  firstIndex: [_i21, 'first_*_addr_index'],
  bytes: [_i32, '*_bytes'],
});

/**
 * @typedef {{
 *   firstIndex: SeriesPattern21<P2PKHAddrIndex>,
 *   bytes: SeriesPattern31<P2PKHBytes>,
 * }} AddressesP2pkh
 */
const _AddressesP2pkh = _s({
  firstIndex: [_i21, 'first_*_addr_index'],
  bytes: [_i31, '*_bytes'],
});

/**
 * @typedef {{
 *   firstIndex: SeriesPattern21<P2PK33AddrIndex>,
 *   bytes: SeriesPattern29<P2PK33Bytes>,
 * }} AddressesP2pk33
 */
const _AddressesP2pk33 = _s({
  firstIndex: [_i21, 'first_*_addr_index'],
  bytes: [_i29, '*_bytes'],
});

/**
 * @typedef {{
 *   firstIndex: SeriesPattern21<P2PK65AddrIndex>,
 *   bytes: SeriesPattern30<P2PK65Bytes>,
 * }} AddressesP2pk65
 */
const _AddressesP2pk65 = _s({
  firstIndex: [_i21, 'first_*_addr_index'],
  bytes: [_i30, '*_bytes'],
});

/**
 * @typedef {{
 *   p2pk65: AddressesP2pk65,
 *   p2pk33: AddressesP2pk33,
 *   p2pkh: AddressesP2pkh,
 *   p2sh: AddressesP2sh,
 *   p2wpkh: AddressesP2wpkh,
 *   p2wsh: AddressesP2wsh,
 *   p2tr: AddressesP2tr,
 *   p2a: AddressesP2a,
 * }} IndexerAddresses
 */
const _IndexerAddresses = _s({
  p2pk65: [_AddressesP2pk65, '*'],
  p2pk33: [_AddressesP2pk33, 'p2pk33'],
  p2pkh: [_AddressesP2pkh, 'p2pkh'],
  p2sh: [_AddressesP2sh, 'p2sh'],
  p2wpkh: [_AddressesP2wpkh, 'p2wpkh'],
  p2wsh: [_AddressesP2wsh, 'p2wsh'],
  p2tr: [_AddressesP2tr, 'p2tr'],
  p2a: [_AddressesP2a, 'p2a'],
});

/**
 * @typedef {{
 *   firstTxoutIndex: SeriesPattern21<TxOutIndex>,
 *   value: SeriesPattern24<Sats>,
 *   outputType: SeriesPattern24<OutputType>,
 *   typeIndex: SeriesPattern24<TypeIndex>,
 * }} IndexerOutputs
 */
const _IndexerOutputs = _s({
  firstTxoutIndex: [_i21, 'first_txout_index'],
  value: [_i24, 'value'],
  outputType: [_i24, 'output_*'],
  typeIndex: [_i24, '*_index'],
});

/**
 * @typedef {{
 *   firstTxinIndex: SeriesPattern21<TxInIndex>,
 *   outpoint: SeriesPattern23<OutPoint>,
 *   txoutIndex: SeriesPattern23<TxOutIndex>,
 *   txIndex: SeriesPattern23<TxIndex>,
 *   outputType: SeriesPattern23<OutputType>,
 *   typeIndex: SeriesPattern23<TypeIndex>,
 * }} IndexerInputs
 */
const _IndexerInputs = _s({
  firstTxinIndex: [_i21, 'first_txin_*'],
  outpoint: [_i23, 'outpoint'],
  txoutIndex: [_i23, 'txout_*'],
  txIndex: [_i23, 'tx_*'],
  outputType: [_i23, 'output_type'],
  typeIndex: [_i23, 'type_*'],
});

/**
 * @typedef {{
 *   v1: SeriesPattern21<Count16>,
 *   v2: SeriesPattern21<Count16>,
 *   v3: SeriesPattern21<Count16>,
 *   otherVersion: SeriesPattern21<Count16>,
 *   explicitlyRbf: SeriesPattern21<Count16>,
 *   oneInput: SeriesPattern21<Count16>,
 *   oneOutput: SeriesPattern21<Count16>,
 *   p2pk: SeriesPattern21<Count16>,
 *   p2ms: SeriesPattern21<Count16>,
 *   p2pkh: SeriesPattern21<Count16>,
 *   p2sh: SeriesPattern21<Count16>,
 *   p2wpkh: SeriesPattern21<Count16>,
 *   p2wsh: SeriesPattern21<Count16>,
 *   p2tr: SeriesPattern21<Count16>,
 *   p2a: SeriesPattern21<Count16>,
 *   opReturn: SeriesPattern21<Count16>,
 *   empty: SeriesPattern21<Count16>,
 *   unknown: SeriesPattern21<Count16>,
 *   fakePubkey: SeriesPattern21<Count16>,
 *   fakeScripthash: SeriesPattern21<Count16>,
 * }} IndexerTransactionsFeaturesCount
 */
const _IndexerTransactionsFeaturesCount = _s({
  v1: [_i21, '*_v1'],
  v2: [_i21, '*_v2'],
  v3: [_i21, '*_v3'],
  otherVersion: [_i21, '*_other_version'],
  explicitlyRbf: [_i21, '*_explicitly_rbf'],
  oneInput: [_i21, '*_one_input'],
  oneOutput: [_i21, '*_one_output'],
  p2pk: [_i21, '*_p2pk'],
  p2ms: [_i21, '*_p2ms'],
  p2pkh: [_i21, '*_p2pkh'],
  p2sh: [_i21, '*_p2sh'],
  p2wpkh: [_i21, '*_p2wpkh'],
  p2wsh: [_i21, '*_p2wsh'],
  p2tr: [_i21, '*_p2tr'],
  p2a: [_i21, '*_p2a'],
  opReturn: [_i21, '*_op_return'],
  empty: [_i21, '*_empty'],
  unknown: [_i21, '*_unknown'],
  fakePubkey: [_i21, '*_fake_pubkey'],
  fakeScripthash: [_i21, '*_fake_scripthash'],
});

/**
 * @typedef {{
 *   count: IndexerTransactionsFeaturesCount,
 *   hasP2pk: SeriesPattern22<Boolean>,
 *   hasP2ms: SeriesPattern22<Boolean>,
 *   hasP2pkh: SeriesPattern22<Boolean>,
 *   hasP2sh: SeriesPattern22<Boolean>,
 *   hasP2wpkh: SeriesPattern22<Boolean>,
 *   hasP2wsh: SeriesPattern22<Boolean>,
 *   hasP2tr: SeriesPattern22<Boolean>,
 *   hasP2a: SeriesPattern22<Boolean>,
 *   hasOpReturn: SeriesPattern22<Boolean>,
 *   hasEmpty: SeriesPattern22<Boolean>,
 *   hasUnknown: SeriesPattern22<Boolean>,
 *   hasFakePubkey: SeriesPattern22<Boolean>,
 *   hasFakeScripthash: SeriesPattern22<Boolean>,
 *   hasInscription: SeriesPattern22<Boolean>,
 *   hasAnnex: SeriesPattern22<Boolean>,
 *   hasSighashAll: SeriesPattern22<Boolean>,
 *   hasSighashNone: SeriesPattern22<Boolean>,
 *   hasSighashSingle: SeriesPattern22<Boolean>,
 *   hasSighashDefault: SeriesPattern22<Boolean>,
 *   hasSighashAnyoneCanPay: SeriesPattern22<Boolean>,
 *   hasDustOutput: SeriesPattern22<Boolean>,
 * }} IndexerTransactionsFeatures
 */
const _IndexerTransactionsFeatures = _s({
  count: [_IndexerTransactionsFeaturesCount, 'tx_count'],
  hasP2pk: [_i22, '*_p2pk'],
  hasP2ms: [_i22, '*_p2ms'],
  hasP2pkh: [_i22, '*_p2pkh'],
  hasP2sh: [_i22, '*_p2sh'],
  hasP2wpkh: [_i22, '*_p2wpkh'],
  hasP2wsh: [_i22, '*_p2wsh'],
  hasP2tr: [_i22, '*_p2tr'],
  hasP2a: [_i22, '*_p2a'],
  hasOpReturn: [_i22, '*_op_return'],
  hasEmpty: [_i22, '*_empty'],
  hasUnknown: [_i22, '*_unknown'],
  hasFakePubkey: [_i22, '*_fake_pubkey'],
  hasFakeScripthash: [_i22, '*_fake_scripthash'],
  hasInscription: [_i22, '*_inscription'],
  hasAnnex: [_i22, '*_annex'],
  hasSighashAll: [_i22, '*_sighash_all'],
  hasSighashNone: [_i22, '*_sighash_none'],
  hasSighashSingle: [_i22, '*_sighash_single'],
  hasSighashDefault: [_i22, '*_sighash_default'],
  hasSighashAnyoneCanPay: [_i22, '*_sighash_anyone_can_pay'],
  hasDustOutput: [_i22, '*_dust_output'],
});

/**
 * @typedef {{
 *   firstTxIndex: SeriesPattern21<TxIndex>,
 *   txid: SeriesPattern22<Txid>,
 *   txVersion: SeriesPattern22<TxVersion>,
 *   rawLocktime: SeriesPattern22<RawLockTime>,
 *   weight: SeriesPattern22<Weight>,
 *   totalSize: SeriesPattern22<Bytes32>,
 *   totalSigopCost: SeriesPattern22<SigOps>,
 *   isExplicitlyRbf: SeriesPattern22<Boolean>,
 *   firstTxinIndex: SeriesPattern22<TxInIndex>,
 *   firstTxoutIndex: SeriesPattern22<TxOutIndex>,
 *   features: IndexerTransactionsFeatures,
 * }} IndexerTransactions
 */
const _IndexerTransactions = _s({
  firstTxIndex: [_i21, 'first_*_index'],
  txid: [_i22, 'txid'],
  txVersion: [_i22, '*_version'],
  rawLocktime: [_i22, 'raw_locktime'],
  weight: [_i22, '*_weight'],
  totalSize: [_i22, 'total_size'],
  totalSigopCost: [_i22, 'total_sigop_cost'],
  isExplicitlyRbf: [_i22, 'is_explicitly_rbf'],
  firstTxinIndex: [_i22, 'first_txin_index'],
  firstTxoutIndex: [_i22, 'first_txout_index'],
  features: [_IndexerTransactionsFeatures, 'has'],
});

/**
 * @template A
 * @typedef {{
 *   block: SeriesPattern21<A>,
 * }} IndexerBlocksSize
 */
const _IndexerBlocksSize = _s({
  block: [_i21, '*'],
});

/**
 * @typedef {{
 *   timestamp: SeriesPattern21<Timestamp>,
 * }} Time
 */
const _Time = _s({
  timestamp: [_i21, '*'],
});

/**
 * @typedef {{
 *   blockhash: SeriesPattern21<BlockHash>,
 *   coinbaseTag: SeriesPattern21<CoinbaseTag>,
 *   time: Time,
 *   size: IndexerBlocksSize<Bytes32>,
 *   weight: IndexerBlocksSize<Weight>,
 *   segwitTxs: SeriesPattern21<Count16>,
 *   segwitSize: SeriesPattern21<Bytes32>,
 *   segwitWeight: SeriesPattern21<Weight>,
 * }} IndexerBlocks
 */
const _IndexerBlocks = _s({
  blockhash: [_i21, 'blockhash'],
  coinbaseTag: [_i21, 'coinbase_tag'],
  time: [_Time, 'timestamp'],
  size: [_IndexerBlocksSize, 'total_size'],
  weight: [_IndexerBlocksSize, 'block_weight'],
  segwitTxs: [_i21, '*_txs'],
  segwitSize: [_i21, '*_size'],
  segwitWeight: [_i21, '*_weight'],
});

/**
 * @typedef {{
 *   blocks: IndexerBlocks,
 *   transactions: IndexerTransactions,
 *   inputs: IndexerInputs,
 *   outputs: IndexerOutputs,
 *   addresses: IndexerAddresses,
 *   scripts: Scripts,
 *   opReturn: IndexerOpReturn,
 * }} Indexer
 */
const _Indexer = _s({
  blocks: [_IndexerBlocks, 'segwit'],
  transactions: [_IndexerTransactions, 'tx'],
  inputs: [_IndexerInputs, '*'],
  outputs: [_IndexerOutputs, 'type'],
  addresses: [_IndexerAddresses, 'p2pk65'],
  scripts: [_Scripts, '*'],
  opReturn: [_IndexerOpReturn, '*'],
});

/**
 * @typedef {{
 *   indexer: Indexer,
 *   blocks: Blocks,
 *   mining: Mining,
 *   transactions: Transactions,
 *   cointime: Cointime,
 *   coinflow: Coinflow,
 *   bedrock: Bedrock,
 *   capitalSentiment: CapitalSentiment,
 *   rarityMeter: RarityMeter,
 *   mappings: Mappings,
 *   indicators: Indicators,
 *   market: Market,
 *   pools: Pools,
 *   price: Price,
 *   age: Age,
 *   holders: Holders,
 *   entry: Entry,
 *   utxos: Utxos,
 *   addresses: Addresses,
 *   supply: Supply,
 *   inputs: Inputs,
 *   outputs: Outputs,
 *   utxoHistory: UtxoHistory,
 *   opReturn: OpReturn,
 * }} SeriesTree
 */
const _SeriesTree = _s({
  indexer: [_Indexer, 'index'],
  blocks: [_Blocks, 'block'],
  mining: [_Mining, 'coinbase'],
  transactions: [_Transactions, 'tx'],
  cointime: [_Cointime, 'awake'],
  coinflow: [_Coinflow, 'coinflow'],
  bedrock: [_Bedrock, 'bedrock'],
  capitalSentiment: [_CapitalSentiment, 'capital_sentiment'],
  rarityMeter: [_RarityMeter, 'rarity_meter'],
  mappings: [_Mappings, 'date'],
  indicators: [_Indicators, 'destroyed_supply_adj'],
  market: [_Market, 'price'],
  pools: [_Pools, 'pool'],
  price: [_Price, 'price'],
  age: [_Age, 'supply'],
  holders: [_Holders, 'under'],
  entry: [_Entry, 'veteran'],
  utxos: [_Utxos, 'supply'],
  addresses: [_Addresses, 'addr'],
  supply: [_Supply, 'supply'],
  inputs: [_Inputs, 'input'],
  outputs: [_Outputs, 'output'],
  utxoHistory: [_UtxoHistory, 'circulating_supply'],
  opReturn: [_OpReturn, 'op_return'],
});

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
    return _lazy(this, 'series', () => _SeriesTree(this, ''));
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
   * Returns the optional description, supported indexes, the indexes whose values can be null, the value type, and the optional unit (what the value type measures) for the specified series. The decoded series name is limited to 1024 UTF-8 bytes.
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
    return _wrapSeriesData(await this.getJson(path, { signal, cache, memCache, onValue: onValue && ((v) => onValue(_wrapSeriesData(v))) }));
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
   * @param {{ signal?: AbortSignal, onValue?: (value: *[] | string) => void, cache?: boolean, memCache?: boolean }} [options]
   * @returns {Promise<*[] | string>}
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
    return (await this.getJson(path, { signal, cache, memCache, onValue: onValue && ((v) => onValue(v.map(_wrapSeriesData))) })).map(_wrapSeriesData);
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
