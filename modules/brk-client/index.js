// Auto-generated BRK JavaScript client
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
 * @property {string} prefix
 */
/**
 * Address statistics in the mempool (unconfirmed transactions only)
 *
 * Based on mempool.space's format.
 *
 * @typedef {Object} AddrMempoolStats
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
 * Address information compatible with mempool.space API format
 *
 * @typedef {Object} AddrStats
 * @property {Addr} address - Bitcoin address string
 * @property {OutputType} addrType - BRK address type (p2pk33, p2pk65, p2pkh, p2sh, p2wpkh, p2wsh, p2tr, etc.)
 * @property {AddrChainStats} chainStats - Statistics for confirmed transactions on the blockchain
 * @property {AddrMempoolStats} mempoolStats - Statistics for unconfirmed transactions in the mempool
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
 * Unified index for any address type (funded or empty)
 *
 * @typedef {TypeIndex} AnyAddrIndex
 */
/**
 * Unsigned basis points stored as u16.
 * 1 bp = 0.0001. Range: 0–6.5535.
 * Use for bounded 0–1 ratios (dominance, adoption, liveliness, etc.).
 * `u16::MAX` is reserved as a NaN sentinel.
 *
 * @typedef {number} BasisPoints16
 */
/**
 * Unsigned basis points stored as u32.
 * 1 bp = 0.0001. Range: 0–429,496.7295.
 * Use for unbounded unsigned ratios (MVRV, NVT, SOPR, etc.).
 * `u32::MAX` is reserved as a NaN sentinel.
 *
 * @typedef {number} BasisPoints32
 */
/**
 * Signed basis points stored as i16.
 * 1 bp = 0.0001. Range: -3.2767 to +3.2767.
 * Use for signed bounded ratios (NUPL, net PnL ratios, etc.).
 * `i16::MIN` is reserved as a NaN sentinel.
 *
 * @typedef {number} BasisPointsSigned16
 */
/**
 * Signed basis points stored as i32.
 * 1 bp = 0.0001. Range: -214,748.3647 to +214,748.3647.
 * Use for unbounded signed values (returns, growth rates, volatility, z-scores, etc.).
 * `i32::MIN` is reserved as a NaN sentinel.
 *
 * @typedef {number} BasisPointsSigned32
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
 * Block hash
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
 * @property {NextBlockHash} hash - Pass back as `<hash>` on
`/api/v1/mempool/block-template/diff/{hash}` to fetch deltas.
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
 * `removed` is redundant (computable from `order` by collecting prior
 * indices that don't appear) but shipped for cache-eviction ergonomics.
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
 *   not in the prior template and must be added at this position.
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
 * Unsigned cents (u64) - for values that should never be negative.
 * Used for invested capital, realized cap, etc.
 *
 * @typedef {number} Cents
 */
/**
 * Cents × Sats (u128) - price in cents multiplied by amount in sats.
 * Uses u128 because large amounts at any price can overflow u64.
 *
 * @typedef {number} CentsSats
 */
/**
 * Signed cents (i64) - for values that can be negative.
 * Used for profit/loss calculations, deltas, etc.
 *
 * @typedef {number} CentsSigned
 */
/**
 * Raw cents squared (u128) - stores cents² × sats without division.
 * Used for precise accumulation of capitalized cap values: Σ(price² × sats).
 * capitalized_price = capitalized_cap_raw / realized_cap_raw
 *
 * @typedef {number} CentsSquaredSats
 */
/**
 * Closing price value for a time period
 *
 * @typedef {Dollars} Close
 */
/**
 * URPD cohort identifier. Use `GET /api/urpd` to list available cohorts.
 *
 * Validated at construction: non-empty, ASCII `[a-z0-9_]+`. Matches the
 * schemars enum value set; the type therefore proves "this is a valid
 * cohort name" wherever a `Cohort` is held.
 *
 * @typedef {("all"|"sth"|"lth"|"utxos_under_1h_old"|"utxos_1h_to_1d_old"|"utxos_1d_to_1w_old"|"utxos_1w_to_1m_old"|"utxos_1m_to_2m_old"|"utxos_2m_to_3m_old"|"utxos_3m_to_4m_old"|"utxos_4m_to_5m_old"|"utxos_5m_to_6m_old"|"utxos_6m_to_1y_old"|"utxos_1y_to_2y_old"|"utxos_2y_to_3y_old"|"utxos_3y_to_4y_old"|"utxos_4y_to_5y_old"|"utxos_5y_to_6y_old"|"utxos_6y_to_7y_old"|"utxos_7y_to_8y_old"|"utxos_8y_to_10y_old"|"utxos_10y_to_12y_old"|"utxos_12y_to_15y_old"|"utxos_over_15y_old")} Cohort
 */
/**
 * Coinbase scriptSig tag for pool identification.
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
 * @typedef {Object} CostBasisCohortParam
 * @property {Cohort} cohort
 */
/**
 * @typedef {Object} CostBasisParams
 * @property {Cohort} cohort
 * @property {string} date
 */
/**
 * @typedef {Object} CostBasisQuery
 * @property {UrpdAggregation=} bucket
 * @property {CostBasisValue=} value
 */
/**
 * Value type for the deprecated cost-basis distribution output.
 *
 * @typedef {("supply"|"realized"|"unrealized")} CostBasisValue
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
 * Date in YYYYMMDD format stored as u32
 *
 * @typedef {number} Date
 */
/** @typedef {number} Day1 */
/** @typedef {number} Day3 */
/**
 * Detailed series count with per-database breakdown
 *
 * @typedef {Object} DetailedSeriesCount
 * @property {number} distinctSeries - Number of unique series available (e.g., realized_price, market_cap)
 * @property {number} totalEndpoints - Total number of series-index combinations across all timeframes
 * @property {number} lazyEndpoints - Number of lazy (computed on-the-fly) series-index combinations
 * @property {number} storedEndpoints - Number of eager (stored on disk) series-index combinations
 * @property {{ [key: string]: SeriesCount }} byDb - Per-database breakdown of counts
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
 * @typedef {Object} DifficultyAdjustmentEntry
 * @property {Timestamp} timestamp - Unix timestamp of the adjustment
 * @property {Height} height - Block height of the adjustment
 * @property {number} difficulty - Difficulty value
 * @property {number} changePercent - Adjustment ratio (new/previous, e.g. 1.068 = +6.8%)
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
 * @property {number} ratio - brk as percentage of Bitcoin data
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
/** @typedef {TypeIndex} EmptyAddrIndex */
/** @typedef {TypeIndex} EmptyOutputIndex */
/** @typedef {number} Epoch */
/**
 * @typedef {Object} ErrorBody
 * @property {ErrorDetail} error
 */
/**
 * @typedef {Object} ErrorDetail
 * @property {string} type - Error category: "invalid_request", "forbidden", "not_found", "unavailable", or "internal"
 * @property {string} code - Machine-readable error code (e.g. "invalid_addr", "series_not_found")
 * @property {string} message - Human-readable description
 * @property {string} docUrl - Link to API documentation
 */
/**
 * Exchange rates (USD base, on-chain only — no fiat pairs available)
 *
 * @typedef {Object} ExchangeRates
 */
/**
 * Fee rate in sat/vB
 *
 * @typedef {number} FeeRate
 */
/**
 * Output format for API responses
 *
 * @typedef {("json"|"csv")} Format
 */
/**
 * Data for a funded (non-empty) address with current balance
 *
 * @typedef {Object} FundedAddrData
 * @property {number} txCount - Total transaction count
 * @property {number} fundedTxoCount - Number of transaction outputs funded to this address
 * @property {number} spentTxoCount - Number of transaction outputs spent by this address
 * @property {Sats} received - Satoshis received by this address
 * @property {Sats} sent - Satoshis sent by this address
 * @property {CentsSats} realizedCapRaw - The realized capitalization: Σ(price × sats)
 * @property {CentsSquaredSats} capitalizedCapRaw - The capitalized cap: Σ(price² × sats)
 */
/** @typedef {TypeIndex} FundedAddrIndex */
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
 * @property {string} point
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
 * @typedef {("minute10"|"minute30"|"hour1"|"hour4"|"hour12"|"day1"|"day3"|"week1"|"month1"|"month3"|"month6"|"year1"|"year10"|"halving"|"epoch"|"height"|"tx_index"|"txin_index"|"txout_index"|"empty_output_index"|"op_return_index"|"p2a_addr_index"|"p2ms_output_index"|"p2pk33_addr_index"|"p2pk65_addr_index"|"p2pkh_addr_index"|"p2sh_addr_index"|"p2tr_addr_index"|"p2wpkh_addr_index"|"p2wsh_addr_index"|"unknown_output_index"|"funded_addr_index"|"empty_addr_index")} Index
 */
/**
 * Information about an available index and its query aliases
 *
 * @typedef {Object} IndexInfo
 * @property {Index} index - The canonical index name
 * @property {string[]} aliases - All Accepted query aliases
 */
/**
 * Legacy path parameter for `/api/metric/{metric}`
 *
 * @typedef {Object} LegacySeriesParam
 * @property {SeriesName} metric
 */
/**
 * Legacy path parameters for `/api/metric/{metric}/{index}`
 *
 * @typedef {Object} LegacySeriesWithIndex
 * @property {SeriesName} metric
 * @property {Index} index
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
 * @property {{ [key: string]: VSize }} feeHistogram - Fee histogram: `[[fee_rate, vsize], ...]` sorted by descending fee rate
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
 * snapshot). Same value as the mempool ETag. Opaque token: pass back
 * as `since` on `/api/v1/mempool/block-template/diff/{hash}` to fetch
 * deltas.
 *
 * @typedef {number} NextBlockHash
 */
/**
 * `since` hash for `/api/v1/mempool/block-template/diff/{hash}`.
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
 * @typedef {("unknown"|"blockfills"|"ultimuspool"|"terrapool"|"luxor"|"1thash"|"btccom"|"bitfarms"|"huobipool"|"wayicn"|"canoepool"|"btctop"|"bitcoincom"|"175btc"|"gbminers"|"axbt"|"asicminer"|"bitminter"|"bitcoinrussia"|"btcserv"|"simplecoinus"|"btcguild"|"eligius"|"ozcoin"|"eclipsemc"|"maxbtc"|"triplemining"|"coinlab"|"50btc"|"ghashio"|"stminingcorp"|"bitparking"|"mmpool"|"polmine"|"kncminer"|"bitalo"|"f2pool"|"hhtt"|"megabigpower"|"mtred"|"nmcbit"|"yourbtcnet"|"givemecoins"|"braiinspool"|"antpool"|"multicoinco"|"bcpoolio"|"cointerra"|"kanopool"|"solock"|"ckpool"|"nicehash"|"bitclub"|"bitcoinaffiliatenetwork"|"btcc"|"bwpool"|"exxbw"|"bitsolo"|"bitfury"|"21inc"|"digitalbtc"|"8baochi"|"mybtccoinpool"|"tbdice"|"hashpool"|"nexious"|"bravomining"|"hotpool"|"okexpool"|"bcmonster"|"1hash"|"bixin"|"tatmaspool"|"viabtc"|"connectbtc"|"batpool"|"waterhole"|"dcexploration"|"dcex"|"btpool"|"58coin"|"bitcoinindia"|"shawnp0wers"|"phashio"|"rigpool"|"haozhuzhu"|"7pool"|"miningkings"|"hashbx"|"dpool"|"rawpool"|"haominer"|"helix"|"bitcoinukraine"|"poolin"|"secretsuperstar"|"tigerpoolnet"|"sigmapoolcom"|"okpooltop"|"hummerpool"|"tangpool"|"bytepool"|"spiderpool"|"novablock"|"miningcity"|"binancepool"|"minerium"|"lubiancom"|"okkong"|"aaopool"|"emcdpool"|"foundryusa"|"sbicrypto"|"arkpool"|"purebtccom"|"marapool"|"kucoinpool"|"entrustcharitypool"|"okminer"|"titan"|"pegapool"|"btcnuggets"|"cloudhashing"|"digitalxmintsy"|"telco214"|"btcpoolparty"|"multipool"|"transactioncoinmining"|"btcdig"|"trickysbtcpool"|"btcmp"|"eobot"|"unomp"|"patels"|"gogreenlight"|"bitcoinindiapool"|"ekanembtc"|"canoe"|"tiger"|"1m1x"|"zulupool"|"secpool"|"ocean"|"whitepool"|"wiz"|"wk057"|"futurebitapollosolo"|"carbonnegative"|"portlandhodl"|"phoenix"|"neopool"|"maxipool"|"bitfufupool"|"gdpool"|"miningdutch"|"publicpool"|"miningsquared"|"innopolistech"|"btclab"|"parasite"|"redrockpool"|"est3lar"|"braiinssolo"|"solopoolcom"|"noderunners")} PoolSlug
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
 * A range boundary: integer index, date, or timestamp.
 *
 * @typedef {(number|Date|Timestamp)} RangeIndex
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
 * @property {Sats} totalReward - Total coinbase rewards (subsidy + fees) in sats
 * @property {Sats} totalFee - Total transaction fees in sats
 * @property {number} totalTx - Total number of transactions
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
 * Series count statistics - distinct series and total series-index combinations
 *
 * @typedef {Object} SeriesCount
 * @property {number} distinctSeries - Number of unique series available (e.g., realized_price, market_cap)
 * @property {number} totalEndpoints - Total number of series-index combinations across all timeframes
 * @property {number} lazyEndpoints - Number of lazy (computed on-the-fly) series-index combinations
 * @property {number} storedEndpoints - Number of eager (stored on disk) series-index combinations
 */
/**
 * Metadata about a series
 *
 * @typedef {Object} SeriesInfo
 * @property {Index[]} indexes - Available indexes
 * @property {string} type - Value type (e.g. "f32", "u64", "Sats")
 */
/**
 * SeriesLeaf with JSON Schema for client generation
 *
 * @typedef {Object} SeriesLeafWithSchema
 * @property {string} name - The series name/identifier
 * @property {string} kind - The Rust type (e.g., "Sats", "StoredF64")
 * @property {Index[]} indexes - Available indexes for this series
 * @property {string} type - JSON Schema type (e.g., "integer", "number", "string", "boolean", "array", "object")
 */
/**
 * Comma-separated list of series names
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
 * Legacy series selection parameters (deprecated)
 *
 * @typedef {Object} SeriesSelectionLegacy
 * @property {Index} index
 * @property {SeriesList} ids
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
/**
 * Current supply state tracking UTXO count and total value
 *
 * @typedef {Object} SupplyState
 * @property {number} utxoCount - Number of unspent transaction outputs
 * @property {Sats} value - Total value in satoshis
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
 * @property {(TxOut|null)=} prevout - Information about the previous output being spent
 * @property {string} scriptsig - Signature script (hex, for non-SegWit inputs)
 * @property {string} scriptsigAsm - Signature script in assembly format
 * @property {Witness} witness - Witness data (stack items, present for SegWit inputs; hex-encoded on the wire)
 * @property {boolean} isCoinbase - Whether this input is a coinbase (block reward) input
 * @property {number} sequence - Input sequence number
 * @property {string} innerRedeemscriptAsm - Inner redeemscript in assembly (for P2SH-wrapped SegWit: scriptsig + witness both present)
 * @property {string} innerWitnessscriptAsm - Inner witnessscript in assembly (for P2WSH: last witness item decoded as script)
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
 * Transaction output
 *
 * @typedef {Object} TxOut
 * @property {string} scriptpubkey - Script pubkey (locking script)
 * @property {Sats} value - Value of the output in satoshis
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
 * Transaction version number
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
 * UTXO Realized Price Distribution for a cohort on a specific date.
 *
 * Supply is grouped by the close price at which each UTXO was last moved.
 * Each bucket exposes three values: supply in BTC, realized cap contribution
 * in USD (sum of `realized_price * supply` over the coins in the bucket), and
 * unrealized P&L in USD (`close * supply - realized_cap`, can be negative).
 *
 * @typedef {Object} Urpd
 * @property {Cohort} cohort
 * @property {Date} date
 * @property {UrpdAggregation} aggregation - Aggregation strategy applied to the buckets.
 * @property {Dollars} close - Close price on `date`, in USD. Anchor for `unrealized_pnl`.
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
 * Path parameters for `/api/urpd/{cohort}/{date}`.
 *
 * @typedef {Object} UrpdParams
 * @property {Cohort} cohort
 * @property {string} date
 */
/**
 * Query parameters for URPD endpoints.
 *
 * @typedef {Object} UrpdQuery
 * @property {UrpdAggregation=} agg - Aggregation strategy. Default: raw (no aggregation). Accepts `bucket` as alias.
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
 * @typedef {Object} BrkClientOptions
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
 * Custom error class for BRK client errors
 */
class BrkError extends Error {
  /**
   * @param {string} message
   * @param {number} [status]
   */
  constructor(message, status) {
    super(message);
    this.name = 'BrkError';
    this.status = status;
  }
}

// Date conversion constants and helpers
const _GENESIS = new Date(2009, 0, 3);  // day1 0, week1 0
const _DAY_ONE = new Date(2009, 0, 9);  // day1 1 (6 day gap after genesis)
const _MS_PER_DAY = 86400000;
const _MS_PER_WEEK = 7 * _MS_PER_DAY;
const _EPOCH_MS = 1230768000000;
const _DATE_INDEXES = new Set([
  'minute10', 'minute30',
  'hour1', 'hour4', 'hour12',
  'day1', 'day3', 'week1',
  'month1', 'month3', 'month6',
  'year1', 'year10',
]);

/** @param {number} months @returns {globalThis.Date} */
const _addMonths = (months) => new Date(2009, months, 1);

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
    case 'day1': return i === 0 ? _GENESIS : new Date(_DAY_ONE.getTime() + (i - 1) * _MS_PER_DAY);
    case 'day3': return new Date(_EPOCH_MS - 86400000 + i * 259200000);
    case 'week1': return new Date(_GENESIS.getTime() + i * _MS_PER_WEEK);
    case 'month1': return _addMonths(i);
    case 'month3': return _addMonths(i * 3);
    case 'month6': return _addMonths(i * 6);
    case 'year1': return new Date(2009 + i, 0, 1);
    case 'year10': return new Date(2009 + i * 10, 0, 1);
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
    case 'day1': {
      if (ms < _DAY_ONE.getTime()) return 0;
      return 1 + Math.floor((ms - _DAY_ONE.getTime()) / _MS_PER_DAY);
    }
    case 'day3': return Math.floor((ms - _EPOCH_MS + 86400000) / 259200000);
    case 'week1': return Math.floor((ms - _GENESIS.getTime()) / _MS_PER_WEEK);
    case 'month1': return (d.getFullYear() - 2009) * 12 + d.getMonth();
    case 'month3': return (d.getFullYear() - 2009) * 4 + Math.floor(d.getMonth() / 3);
    case 'month6': return (d.getFullYear() - 2009) * 2 + Math.floor(d.getMonth() / 6);
    case 'year1': return d.getFullYear() - 2009;
    case 'year10': return Math.floor((d.getFullYear() - 2009) / 10);
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
      const result = [];
      for (let i = start; i < end; i++) result.push(i);
      return result;
    },
    keys() {
      return this.indexes();
    },
    entries() {
      /** @type {Array<[number, T]>} */
      const result = [];
      for (let i = 0; i < data.length; i++) result.push([start + i, data[i]]);
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
 * @param {BrkClient} client
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
class BrkClientBase {
  /**
   * @param {BrkClientOptions|string} options
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
   * @param {{ signal?: AbortSignal, cache?: boolean }} [options]
   * @returns {Promise<Response>}
   */
  async get(path, { signal, cache = true } = {}) {
    const url = `${this.baseUrl}${path}`;
    const signals = [AbortSignal.timeout(this.timeout)];
    if (signal) signals.push(signal);
    /** @type {RequestInit} */
    const init = { signal: AbortSignal.any(signals) };
    if (!cache) init.cache = 'no-store';
    const res = await fetch(url, init);
    if (!res.ok) throw new BrkError(`HTTP ${res.status}: ${url}`, res.status);
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
        const res = await this.get(path, { signal });
        const netEtag = res.headers.get('ETag');
        if (netEtag && netEtag === memHit.etag) return memHit.value;
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
      if (populated && netEtag && netEtag === populated.etag) return populated.value;
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
    if (!res.ok) throw new BrkError(`HTTP ${res.status}: ${url}`, res.status);
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
const _i35 = /** @type {const} */ (["empty_addr_index"]);

/**
 * Generic series pattern factory.
 * @template T
 * @param {BrkClient} client
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
/** @template T @param {BrkClient} client @param {string} name @returns {SeriesPattern1<T>} */
function createSeriesPattern1(client, name) { return /** @type {SeriesPattern1<T>} */ (_mp(client, name, _i1)); }
/** @template T @typedef {{ name: string, by: { readonly minute10: DateSeriesEndpoint<T>, readonly minute30: DateSeriesEndpoint<T>, readonly hour1: DateSeriesEndpoint<T>, readonly hour4: DateSeriesEndpoint<T>, readonly hour12: DateSeriesEndpoint<T>, readonly day1: DateSeriesEndpoint<T>, readonly day3: DateSeriesEndpoint<T>, readonly week1: DateSeriesEndpoint<T>, readonly month1: DateSeriesEndpoint<T>, readonly month3: DateSeriesEndpoint<T>, readonly month6: DateSeriesEndpoint<T>, readonly year1: DateSeriesEndpoint<T>, readonly year10: DateSeriesEndpoint<T>, readonly halving: SeriesEndpoint<T>, readonly epoch: SeriesEndpoint<T> }, indexes: () => readonly Index[], get: (index: Index) => SeriesEndpoint<T>|undefined }} SeriesPattern2 */
/** @template T @param {BrkClient} client @param {string} name @returns {SeriesPattern2<T>} */
function createSeriesPattern2(client, name) { return /** @type {SeriesPattern2<T>} */ (_mp(client, name, _i2)); }
/** @template T @typedef {{ name: string, by: { readonly minute10: DateSeriesEndpoint<T> }, indexes: () => readonly Index[], get: (index: Index) => SeriesEndpoint<T>|undefined }} SeriesPattern3 */
/** @template T @param {BrkClient} client @param {string} name @returns {SeriesPattern3<T>} */
function createSeriesPattern3(client, name) { return /** @type {SeriesPattern3<T>} */ (_mp(client, name, _i3)); }
/** @template T @typedef {{ name: string, by: { readonly minute30: DateSeriesEndpoint<T> }, indexes: () => readonly Index[], get: (index: Index) => SeriesEndpoint<T>|undefined }} SeriesPattern4 */
/** @template T @param {BrkClient} client @param {string} name @returns {SeriesPattern4<T>} */
function createSeriesPattern4(client, name) { return /** @type {SeriesPattern4<T>} */ (_mp(client, name, _i4)); }
/** @template T @typedef {{ name: string, by: { readonly hour1: DateSeriesEndpoint<T> }, indexes: () => readonly Index[], get: (index: Index) => SeriesEndpoint<T>|undefined }} SeriesPattern5 */
/** @template T @param {BrkClient} client @param {string} name @returns {SeriesPattern5<T>} */
function createSeriesPattern5(client, name) { return /** @type {SeriesPattern5<T>} */ (_mp(client, name, _i5)); }
/** @template T @typedef {{ name: string, by: { readonly hour4: DateSeriesEndpoint<T> }, indexes: () => readonly Index[], get: (index: Index) => SeriesEndpoint<T>|undefined }} SeriesPattern6 */
/** @template T @param {BrkClient} client @param {string} name @returns {SeriesPattern6<T>} */
function createSeriesPattern6(client, name) { return /** @type {SeriesPattern6<T>} */ (_mp(client, name, _i6)); }
/** @template T @typedef {{ name: string, by: { readonly hour12: DateSeriesEndpoint<T> }, indexes: () => readonly Index[], get: (index: Index) => SeriesEndpoint<T>|undefined }} SeriesPattern7 */
/** @template T @param {BrkClient} client @param {string} name @returns {SeriesPattern7<T>} */
function createSeriesPattern7(client, name) { return /** @type {SeriesPattern7<T>} */ (_mp(client, name, _i7)); }
/** @template T @typedef {{ name: string, by: { readonly day1: DateSeriesEndpoint<T> }, indexes: () => readonly Index[], get: (index: Index) => SeriesEndpoint<T>|undefined }} SeriesPattern8 */
/** @template T @param {BrkClient} client @param {string} name @returns {SeriesPattern8<T>} */
function createSeriesPattern8(client, name) { return /** @type {SeriesPattern8<T>} */ (_mp(client, name, _i8)); }
/** @template T @typedef {{ name: string, by: { readonly day3: DateSeriesEndpoint<T> }, indexes: () => readonly Index[], get: (index: Index) => SeriesEndpoint<T>|undefined }} SeriesPattern9 */
/** @template T @param {BrkClient} client @param {string} name @returns {SeriesPattern9<T>} */
function createSeriesPattern9(client, name) { return /** @type {SeriesPattern9<T>} */ (_mp(client, name, _i9)); }
/** @template T @typedef {{ name: string, by: { readonly week1: DateSeriesEndpoint<T> }, indexes: () => readonly Index[], get: (index: Index) => SeriesEndpoint<T>|undefined }} SeriesPattern10 */
/** @template T @param {BrkClient} client @param {string} name @returns {SeriesPattern10<T>} */
function createSeriesPattern10(client, name) { return /** @type {SeriesPattern10<T>} */ (_mp(client, name, _i10)); }
/** @template T @typedef {{ name: string, by: { readonly month1: DateSeriesEndpoint<T> }, indexes: () => readonly Index[], get: (index: Index) => SeriesEndpoint<T>|undefined }} SeriesPattern11 */
/** @template T @param {BrkClient} client @param {string} name @returns {SeriesPattern11<T>} */
function createSeriesPattern11(client, name) { return /** @type {SeriesPattern11<T>} */ (_mp(client, name, _i11)); }
/** @template T @typedef {{ name: string, by: { readonly month3: DateSeriesEndpoint<T> }, indexes: () => readonly Index[], get: (index: Index) => SeriesEndpoint<T>|undefined }} SeriesPattern12 */
/** @template T @param {BrkClient} client @param {string} name @returns {SeriesPattern12<T>} */
function createSeriesPattern12(client, name) { return /** @type {SeriesPattern12<T>} */ (_mp(client, name, _i12)); }
/** @template T @typedef {{ name: string, by: { readonly month6: DateSeriesEndpoint<T> }, indexes: () => readonly Index[], get: (index: Index) => SeriesEndpoint<T>|undefined }} SeriesPattern13 */
/** @template T @param {BrkClient} client @param {string} name @returns {SeriesPattern13<T>} */
function createSeriesPattern13(client, name) { return /** @type {SeriesPattern13<T>} */ (_mp(client, name, _i13)); }
/** @template T @typedef {{ name: string, by: { readonly year1: DateSeriesEndpoint<T> }, indexes: () => readonly Index[], get: (index: Index) => SeriesEndpoint<T>|undefined }} SeriesPattern14 */
/** @template T @param {BrkClient} client @param {string} name @returns {SeriesPattern14<T>} */
function createSeriesPattern14(client, name) { return /** @type {SeriesPattern14<T>} */ (_mp(client, name, _i14)); }
/** @template T @typedef {{ name: string, by: { readonly year10: DateSeriesEndpoint<T> }, indexes: () => readonly Index[], get: (index: Index) => SeriesEndpoint<T>|undefined }} SeriesPattern15 */
/** @template T @param {BrkClient} client @param {string} name @returns {SeriesPattern15<T>} */
function createSeriesPattern15(client, name) { return /** @type {SeriesPattern15<T>} */ (_mp(client, name, _i15)); }
/** @template T @typedef {{ name: string, by: { readonly halving: SeriesEndpoint<T> }, indexes: () => readonly Index[], get: (index: Index) => SeriesEndpoint<T>|undefined }} SeriesPattern16 */
/** @template T @param {BrkClient} client @param {string} name @returns {SeriesPattern16<T>} */
function createSeriesPattern16(client, name) { return /** @type {SeriesPattern16<T>} */ (_mp(client, name, _i16)); }
/** @template T @typedef {{ name: string, by: { readonly epoch: SeriesEndpoint<T> }, indexes: () => readonly Index[], get: (index: Index) => SeriesEndpoint<T>|undefined }} SeriesPattern17 */
/** @template T @param {BrkClient} client @param {string} name @returns {SeriesPattern17<T>} */
function createSeriesPattern17(client, name) { return /** @type {SeriesPattern17<T>} */ (_mp(client, name, _i17)); }
/** @template T @typedef {{ name: string, by: { readonly height: SeriesEndpoint<T> }, indexes: () => readonly Index[], get: (index: Index) => SeriesEndpoint<T>|undefined }} SeriesPattern18 */
/** @template T @param {BrkClient} client @param {string} name @returns {SeriesPattern18<T>} */
function createSeriesPattern18(client, name) { return /** @type {SeriesPattern18<T>} */ (_mp(client, name, _i18)); }
/** @template T @typedef {{ name: string, by: { readonly tx_index: SeriesEndpoint<T> }, indexes: () => readonly Index[], get: (index: Index) => SeriesEndpoint<T>|undefined }} SeriesPattern19 */
/** @template T @param {BrkClient} client @param {string} name @returns {SeriesPattern19<T>} */
function createSeriesPattern19(client, name) { return /** @type {SeriesPattern19<T>} */ (_mp(client, name, _i19)); }
/** @template T @typedef {{ name: string, by: { readonly txin_index: SeriesEndpoint<T> }, indexes: () => readonly Index[], get: (index: Index) => SeriesEndpoint<T>|undefined }} SeriesPattern20 */
/** @template T @param {BrkClient} client @param {string} name @returns {SeriesPattern20<T>} */
function createSeriesPattern20(client, name) { return /** @type {SeriesPattern20<T>} */ (_mp(client, name, _i20)); }
/** @template T @typedef {{ name: string, by: { readonly txout_index: SeriesEndpoint<T> }, indexes: () => readonly Index[], get: (index: Index) => SeriesEndpoint<T>|undefined }} SeriesPattern21 */
/** @template T @param {BrkClient} client @param {string} name @returns {SeriesPattern21<T>} */
function createSeriesPattern21(client, name) { return /** @type {SeriesPattern21<T>} */ (_mp(client, name, _i21)); }
/** @template T @typedef {{ name: string, by: { readonly empty_output_index: SeriesEndpoint<T> }, indexes: () => readonly Index[], get: (index: Index) => SeriesEndpoint<T>|undefined }} SeriesPattern22 */
/** @template T @param {BrkClient} client @param {string} name @returns {SeriesPattern22<T>} */
function createSeriesPattern22(client, name) { return /** @type {SeriesPattern22<T>} */ (_mp(client, name, _i22)); }
/** @template T @typedef {{ name: string, by: { readonly op_return_index: SeriesEndpoint<T> }, indexes: () => readonly Index[], get: (index: Index) => SeriesEndpoint<T>|undefined }} SeriesPattern23 */
/** @template T @param {BrkClient} client @param {string} name @returns {SeriesPattern23<T>} */
function createSeriesPattern23(client, name) { return /** @type {SeriesPattern23<T>} */ (_mp(client, name, _i23)); }
/** @template T @typedef {{ name: string, by: { readonly p2a_addr_index: SeriesEndpoint<T> }, indexes: () => readonly Index[], get: (index: Index) => SeriesEndpoint<T>|undefined }} SeriesPattern24 */
/** @template T @param {BrkClient} client @param {string} name @returns {SeriesPattern24<T>} */
function createSeriesPattern24(client, name) { return /** @type {SeriesPattern24<T>} */ (_mp(client, name, _i24)); }
/** @template T @typedef {{ name: string, by: { readonly p2ms_output_index: SeriesEndpoint<T> }, indexes: () => readonly Index[], get: (index: Index) => SeriesEndpoint<T>|undefined }} SeriesPattern25 */
/** @template T @param {BrkClient} client @param {string} name @returns {SeriesPattern25<T>} */
function createSeriesPattern25(client, name) { return /** @type {SeriesPattern25<T>} */ (_mp(client, name, _i25)); }
/** @template T @typedef {{ name: string, by: { readonly p2pk33_addr_index: SeriesEndpoint<T> }, indexes: () => readonly Index[], get: (index: Index) => SeriesEndpoint<T>|undefined }} SeriesPattern26 */
/** @template T @param {BrkClient} client @param {string} name @returns {SeriesPattern26<T>} */
function createSeriesPattern26(client, name) { return /** @type {SeriesPattern26<T>} */ (_mp(client, name, _i26)); }
/** @template T @typedef {{ name: string, by: { readonly p2pk65_addr_index: SeriesEndpoint<T> }, indexes: () => readonly Index[], get: (index: Index) => SeriesEndpoint<T>|undefined }} SeriesPattern27 */
/** @template T @param {BrkClient} client @param {string} name @returns {SeriesPattern27<T>} */
function createSeriesPattern27(client, name) { return /** @type {SeriesPattern27<T>} */ (_mp(client, name, _i27)); }
/** @template T @typedef {{ name: string, by: { readonly p2pkh_addr_index: SeriesEndpoint<T> }, indexes: () => readonly Index[], get: (index: Index) => SeriesEndpoint<T>|undefined }} SeriesPattern28 */
/** @template T @param {BrkClient} client @param {string} name @returns {SeriesPattern28<T>} */
function createSeriesPattern28(client, name) { return /** @type {SeriesPattern28<T>} */ (_mp(client, name, _i28)); }
/** @template T @typedef {{ name: string, by: { readonly p2sh_addr_index: SeriesEndpoint<T> }, indexes: () => readonly Index[], get: (index: Index) => SeriesEndpoint<T>|undefined }} SeriesPattern29 */
/** @template T @param {BrkClient} client @param {string} name @returns {SeriesPattern29<T>} */
function createSeriesPattern29(client, name) { return /** @type {SeriesPattern29<T>} */ (_mp(client, name, _i29)); }
/** @template T @typedef {{ name: string, by: { readonly p2tr_addr_index: SeriesEndpoint<T> }, indexes: () => readonly Index[], get: (index: Index) => SeriesEndpoint<T>|undefined }} SeriesPattern30 */
/** @template T @param {BrkClient} client @param {string} name @returns {SeriesPattern30<T>} */
function createSeriesPattern30(client, name) { return /** @type {SeriesPattern30<T>} */ (_mp(client, name, _i30)); }
/** @template T @typedef {{ name: string, by: { readonly p2wpkh_addr_index: SeriesEndpoint<T> }, indexes: () => readonly Index[], get: (index: Index) => SeriesEndpoint<T>|undefined }} SeriesPattern31 */
/** @template T @param {BrkClient} client @param {string} name @returns {SeriesPattern31<T>} */
function createSeriesPattern31(client, name) { return /** @type {SeriesPattern31<T>} */ (_mp(client, name, _i31)); }
/** @template T @typedef {{ name: string, by: { readonly p2wsh_addr_index: SeriesEndpoint<T> }, indexes: () => readonly Index[], get: (index: Index) => SeriesEndpoint<T>|undefined }} SeriesPattern32 */
/** @template T @param {BrkClient} client @param {string} name @returns {SeriesPattern32<T>} */
function createSeriesPattern32(client, name) { return /** @type {SeriesPattern32<T>} */ (_mp(client, name, _i32)); }
/** @template T @typedef {{ name: string, by: { readonly unknown_output_index: SeriesEndpoint<T> }, indexes: () => readonly Index[], get: (index: Index) => SeriesEndpoint<T>|undefined }} SeriesPattern33 */
/** @template T @param {BrkClient} client @param {string} name @returns {SeriesPattern33<T>} */
function createSeriesPattern33(client, name) { return /** @type {SeriesPattern33<T>} */ (_mp(client, name, _i33)); }
/** @template T @typedef {{ name: string, by: { readonly funded_addr_index: SeriesEndpoint<T> }, indexes: () => readonly Index[], get: (index: Index) => SeriesEndpoint<T>|undefined }} SeriesPattern34 */
/** @template T @param {BrkClient} client @param {string} name @returns {SeriesPattern34<T>} */
function createSeriesPattern34(client, name) { return /** @type {SeriesPattern34<T>} */ (_mp(client, name, _i34)); }
/** @template T @typedef {{ name: string, by: { readonly empty_addr_index: SeriesEndpoint<T> }, indexes: () => readonly Index[], get: (index: Index) => SeriesEndpoint<T>|undefined }} SeriesPattern35 */
/** @template T @param {BrkClient} client @param {string} name @returns {SeriesPattern35<T>} */
function createSeriesPattern35(client, name) { return /** @type {SeriesPattern35<T>} */ (_mp(client, name, _i35)); }

// Reusable structural pattern factories

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
 * @param {BrkClient} client
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
 * @typedef {Object} _0sdM0M1M1sdM2M2sdM3sdP0P1P1sdP2P2sdP3sdSdZscorePattern
 * @property {CentsSatsUsdPattern} _0sd
 * @property {PriceRatioPattern} m05sd
 * @property {PriceRatioPattern} m15sd
 * @property {PriceRatioPattern} m1sd
 * @property {PriceRatioPattern} m25sd
 * @property {PriceRatioPattern} m2sd
 * @property {PriceRatioPattern} m3sd
 * @property {PriceRatioPattern} p05sd
 * @property {PriceRatioPattern} p15sd
 * @property {PriceRatioPattern} p1sd
 * @property {PriceRatioPattern} p25sd
 * @property {PriceRatioPattern} p2sd
 * @property {PriceRatioPattern} p3sd
 * @property {SeriesPattern1<StoredF32>} sd
 * @property {SeriesPattern1<StoredF32>} zscore
 */

/**
 * @typedef {Object} AllEmptyOpP2aP2msP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshUnknownPattern
 * @property {AverageBlockCumulativeSumPattern<StoredU64>} all
 * @property {AverageBlockCumulativeSumPattern<StoredU64>} empty
 * @property {AverageBlockCumulativeSumPattern<StoredU64>} opReturn
 * @property {AverageBlockCumulativeSumPattern<StoredU64>} p2a
 * @property {AverageBlockCumulativeSumPattern<StoredU64>} p2ms
 * @property {AverageBlockCumulativeSumPattern<StoredU64>} p2pk33
 * @property {AverageBlockCumulativeSumPattern<StoredU64>} p2pk65
 * @property {AverageBlockCumulativeSumPattern<StoredU64>} p2pkh
 * @property {AverageBlockCumulativeSumPattern<StoredU64>} p2sh
 * @property {AverageBlockCumulativeSumPattern<StoredU64>} p2tr
 * @property {AverageBlockCumulativeSumPattern<StoredU64>} p2wpkh
 * @property {AverageBlockCumulativeSumPattern<StoredU64>} p2wsh
 * @property {AverageBlockCumulativeSumPattern<StoredU64>} unknown
 */

/**
 * Create a AllEmptyOpP2aP2msP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshUnknownPattern pattern node
 * @param {BrkClient} client
 * @param {string} acc - Accumulated series name
 * @returns {AllEmptyOpP2aP2msP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshUnknownPattern}
 */
function createAllEmptyOpP2aP2msP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshUnknownPattern(client, acc) {
  return {
    all: createAverageBlockCumulativeSumPattern(client, _m(acc, 'bis')),
    empty: createAverageBlockCumulativeSumPattern(client, _m(acc, 'with_empty_outputs_output')),
    opReturn: createAverageBlockCumulativeSumPattern(client, _m(acc, 'with_op_return_output')),
    p2a: createAverageBlockCumulativeSumPattern(client, _m(acc, 'with_p2a_output')),
    p2ms: createAverageBlockCumulativeSumPattern(client, _m(acc, 'with_p2ms_output')),
    p2pk33: createAverageBlockCumulativeSumPattern(client, _m(acc, 'with_p2pk33_output')),
    p2pk65: createAverageBlockCumulativeSumPattern(client, _m(acc, 'with_p2pk65_output')),
    p2pkh: createAverageBlockCumulativeSumPattern(client, _m(acc, 'with_p2pkh_output')),
    p2sh: createAverageBlockCumulativeSumPattern(client, _m(acc, 'with_p2sh_output')),
    p2tr: createAverageBlockCumulativeSumPattern(client, _m(acc, 'with_p2tr_output')),
    p2wpkh: createAverageBlockCumulativeSumPattern(client, _m(acc, 'with_p2wpkh_output')),
    p2wsh: createAverageBlockCumulativeSumPattern(client, _m(acc, 'with_p2wsh_output')),
    unknown: createAverageBlockCumulativeSumPattern(client, _m(acc, 'with_unknown_outputs_output')),
  };
}

/**
 * @typedef {Object} _10y1m1w1y2y3m3y4y5y6m6y8yPattern2
 * @property {BpsPercentRatioPattern} _10y
 * @property {BpsPercentRatioPattern} _1m
 * @property {BpsPercentRatioPattern} _1w
 * @property {BpsPercentRatioPattern} _1y
 * @property {BpsPercentRatioPattern} _2y
 * @property {BpsPercentRatioPattern} _3m
 * @property {BpsPercentRatioPattern} _3y
 * @property {BpsPercentRatioPattern} _4y
 * @property {BpsPercentRatioPattern} _5y
 * @property {BpsPercentRatioPattern} _6m
 * @property {BpsPercentRatioPattern} _6y
 * @property {BpsPercentRatioPattern} _8y
 */

/**
 * Create a _10y1m1w1y2y3m3y4y5y6m6y8yPattern2 pattern node
 * @param {BrkClient} client
 * @param {string} acc - Accumulated series name
 * @returns {_10y1m1w1y2y3m3y4y5y6m6y8yPattern2}
 */
function create_10y1m1w1y2y3m3y4y5y6m6y8yPattern2(client, acc) {
  return {
    _10y: createBpsPercentRatioPattern(client, _m(acc, '10y')),
    _1m: createBpsPercentRatioPattern(client, _m(acc, '1m')),
    _1w: createBpsPercentRatioPattern(client, _m(acc, '1w')),
    _1y: createBpsPercentRatioPattern(client, _m(acc, '1y')),
    _2y: createBpsPercentRatioPattern(client, _m(acc, '2y')),
    _3m: createBpsPercentRatioPattern(client, _m(acc, '3m')),
    _3y: createBpsPercentRatioPattern(client, _m(acc, '3y')),
    _4y: createBpsPercentRatioPattern(client, _m(acc, '4y')),
    _5y: createBpsPercentRatioPattern(client, _m(acc, '5y')),
    _6m: createBpsPercentRatioPattern(client, _m(acc, '6m')),
    _6y: createBpsPercentRatioPattern(client, _m(acc, '6y')),
    _8y: createBpsPercentRatioPattern(client, _m(acc, '8y')),
  };
}

/**
 * @typedef {Object} _10y1m1w1y2y3m3y4y5y6m6y8yPattern3
 * @property {BtcCentsSatsUsdPattern} _10y
 * @property {BtcCentsSatsUsdPattern} _1m
 * @property {BtcCentsSatsUsdPattern} _1w
 * @property {BtcCentsSatsUsdPattern} _1y
 * @property {BtcCentsSatsUsdPattern} _2y
 * @property {BtcCentsSatsUsdPattern} _3m
 * @property {BtcCentsSatsUsdPattern} _3y
 * @property {BtcCentsSatsUsdPattern} _4y
 * @property {BtcCentsSatsUsdPattern} _5y
 * @property {BtcCentsSatsUsdPattern} _6m
 * @property {BtcCentsSatsUsdPattern} _6y
 * @property {BtcCentsSatsUsdPattern} _8y
 */

/**
 * Create a _10y1m1w1y2y3m3y4y5y6m6y8yPattern3 pattern node
 * @param {BrkClient} client
 * @param {string} acc - Accumulated series name
 * @returns {_10y1m1w1y2y3m3y4y5y6m6y8yPattern3}
 */
function create_10y1m1w1y2y3m3y4y5y6m6y8yPattern3(client, acc) {
  return {
    _10y: createBtcCentsSatsUsdPattern(client, _m(acc, '10y')),
    _1m: createBtcCentsSatsUsdPattern(client, _m(acc, '1m')),
    _1w: createBtcCentsSatsUsdPattern(client, _m(acc, '1w')),
    _1y: createBtcCentsSatsUsdPattern(client, _m(acc, '1y')),
    _2y: createBtcCentsSatsUsdPattern(client, _m(acc, '2y')),
    _3m: createBtcCentsSatsUsdPattern(client, _m(acc, '3m')),
    _3y: createBtcCentsSatsUsdPattern(client, _m(acc, '3y')),
    _4y: createBtcCentsSatsUsdPattern(client, _m(acc, '4y')),
    _5y: createBtcCentsSatsUsdPattern(client, _m(acc, '5y')),
    _6m: createBtcCentsSatsUsdPattern(client, _m(acc, '6m')),
    _6y: createBtcCentsSatsUsdPattern(client, _m(acc, '6y')),
    _8y: createBtcCentsSatsUsdPattern(client, _m(acc, '8y')),
  };
}

/**
 * @typedef {Object} AllEmptyP2aP2msP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshUnknownPattern
 * @property {AverageBlockCumulativeSumPattern<StoredU64>} all
 * @property {AverageBlockCumulativeSumPattern<StoredU64>} empty
 * @property {AverageBlockCumulativeSumPattern<StoredU64>} p2a
 * @property {AverageBlockCumulativeSumPattern<StoredU64>} p2ms
 * @property {AverageBlockCumulativeSumPattern<StoredU64>} p2pk33
 * @property {AverageBlockCumulativeSumPattern<StoredU64>} p2pk65
 * @property {AverageBlockCumulativeSumPattern<StoredU64>} p2pkh
 * @property {AverageBlockCumulativeSumPattern<StoredU64>} p2sh
 * @property {AverageBlockCumulativeSumPattern<StoredU64>} p2tr
 * @property {AverageBlockCumulativeSumPattern<StoredU64>} p2wpkh
 * @property {AverageBlockCumulativeSumPattern<StoredU64>} p2wsh
 * @property {AverageBlockCumulativeSumPattern<StoredU64>} unknown
 */

/**
 * @typedef {Object} CapCapitalizedGrossLossMvrvNetPeakPriceProfitSellSoprPattern
 * @property {CentsDeltaToUsdPattern} cap
 * @property {PricePattern} capitalized
 * @property {BlockCumulativeSumPattern} grossPnl
 * @property {BlockCumulativeNegativeSumPattern} loss
 * @property {SeriesPattern1<StoredF32>} mvrv
 * @property {BlockChangeCumulativeDeltaSumPattern} netPnl
 * @property {BlockCumulativeSumPattern} peakRegret
 * @property {BpsCentsPercentilesRatioSatsSmaStdUsdPattern} price
 * @property {BlockCumulativeSumPattern} profit
 * @property {_1m1w1y24hPattern<StoredF64>} profitToLossRatio
 * @property {_1m1w1y24hPattern8} sellSideRiskRatio
 * @property {AdjustedRatioValuePattern} sopr
 */

/**
 * @typedef {Object} CapCapitalizedGrossLossMvrvNetPeakPriceProfitSellSoprPattern2
 * @property {CentsDeltaToUsdPattern} cap
 * @property {PricePattern} capitalized
 * @property {BlockCumulativeSumPattern} grossPnl
 * @property {BlockCumulativeNegativeSumPattern} loss
 * @property {SeriesPattern1<StoredF32>} mvrv
 * @property {BlockChangeCumulativeDeltaSumPattern} netPnl
 * @property {BlockCumulativeSumPattern} peakRegret
 * @property {BpsCentsPercentilesRatioSatsSmaStdUsdPattern} price
 * @property {BlockCumulativeSumPattern} profit
 * @property {_1m1w1y24hPattern<StoredF64>} profitToLossRatio
 * @property {_1m1w1y24hPattern8} sellSideRiskRatio
 * @property {RatioValuePattern2} sopr
 */

/**
 * @typedef {Object} EmptyOpP2aP2msP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshUnknownPattern2
 * @property {_1m1w1y24hBpsPercentRatioPattern} empty
 * @property {_1m1w1y24hBpsPercentRatioPattern} opReturn
 * @property {_1m1w1y24hBpsPercentRatioPattern} p2a
 * @property {_1m1w1y24hBpsPercentRatioPattern} p2ms
 * @property {_1m1w1y24hBpsPercentRatioPattern} p2pk33
 * @property {_1m1w1y24hBpsPercentRatioPattern} p2pk65
 * @property {_1m1w1y24hBpsPercentRatioPattern} p2pkh
 * @property {_1m1w1y24hBpsPercentRatioPattern} p2sh
 * @property {_1m1w1y24hBpsPercentRatioPattern} p2tr
 * @property {_1m1w1y24hBpsPercentRatioPattern} p2wpkh
 * @property {_1m1w1y24hBpsPercentRatioPattern} p2wsh
 * @property {_1m1w1y24hBpsPercentRatioPattern} unknown
 */

/**
 * Create a EmptyOpP2aP2msP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshUnknownPattern2 pattern node
 * @param {BrkClient} client
 * @param {string} acc - Accumulated series name
 * @returns {EmptyOpP2aP2msP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshUnknownPattern2}
 */
function createEmptyOpP2aP2msP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshUnknownPattern2(client, acc) {
  return {
    empty: create_1m1w1y24hBpsPercentRatioPattern(client, _m(acc, 'empty_outputs_output')),
    opReturn: create_1m1w1y24hBpsPercentRatioPattern(client, _m(acc, 'op_return_output')),
    p2a: create_1m1w1y24hBpsPercentRatioPattern(client, _m(acc, 'p2a_output')),
    p2ms: create_1m1w1y24hBpsPercentRatioPattern(client, _m(acc, 'p2ms_output')),
    p2pk33: create_1m1w1y24hBpsPercentRatioPattern(client, _m(acc, 'p2pk33_output')),
    p2pk65: create_1m1w1y24hBpsPercentRatioPattern(client, _m(acc, 'p2pk65_output')),
    p2pkh: create_1m1w1y24hBpsPercentRatioPattern(client, _m(acc, 'p2pkh_output')),
    p2sh: create_1m1w1y24hBpsPercentRatioPattern(client, _m(acc, 'p2sh_output')),
    p2tr: create_1m1w1y24hBpsPercentRatioPattern(client, _m(acc, 'p2tr_output')),
    p2wpkh: create_1m1w1y24hBpsPercentRatioPattern(client, _m(acc, 'p2wpkh_output')),
    p2wsh: create_1m1w1y24hBpsPercentRatioPattern(client, _m(acc, 'p2wsh_output')),
    unknown: create_1m1w1y24hBpsPercentRatioPattern(client, _m(acc, 'unknown_outputs_output')),
  };
}

/**
 * @typedef {Object} AverageBlockCumulativeMaxMedianMinPct10Pct25Pct75Pct90SumPattern
 * @property {_1m1w1y24hPattern<StoredF32>} average
 * @property {SeriesPattern18<StoredU64>} block
 * @property {SeriesPattern1<StoredU64>} cumulative
 * @property {_1m1w1y24hPattern<StoredU64>} max
 * @property {_1m1w1y24hPattern<StoredU64>} median
 * @property {_1m1w1y24hPattern<StoredU64>} min
 * @property {_1m1w1y24hPattern<StoredU64>} pct10
 * @property {_1m1w1y24hPattern<StoredU64>} pct25
 * @property {_1m1w1y24hPattern<StoredU64>} pct75
 * @property {_1m1w1y24hPattern<StoredU64>} pct90
 * @property {_1m1w1y24hPattern<StoredU64>} sum
 */

/**
 * Create a AverageBlockCumulativeMaxMedianMinPct10Pct25Pct75Pct90SumPattern pattern node
 * @param {BrkClient} client
 * @param {string} acc - Accumulated series name
 * @returns {AverageBlockCumulativeMaxMedianMinPct10Pct25Pct75Pct90SumPattern}
 */
function createAverageBlockCumulativeMaxMedianMinPct10Pct25Pct75Pct90SumPattern(client, acc) {
  return {
    average: create_1m1w1y24hPattern(client, _m(acc, 'average')),
    block: createSeriesPattern18(client, acc),
    cumulative: createSeriesPattern1(client, _m(acc, 'cumulative')),
    max: create_1m1w1y24hPattern(client, _m(acc, 'max')),
    median: create_1m1w1y24hPattern(client, _m(acc, 'median')),
    min: create_1m1w1y24hPattern(client, _m(acc, 'min')),
    pct10: create_1m1w1y24hPattern(client, _m(acc, 'pct10')),
    pct25: create_1m1w1y24hPattern(client, _m(acc, 'pct25')),
    pct75: create_1m1w1y24hPattern(client, _m(acc, 'pct75')),
    pct90: create_1m1w1y24hPattern(client, _m(acc, 'pct90')),
    sum: create_1m1w1y24hPattern(client, _m(acc, 'sum')),
  };
}

/**
 * @typedef {Object} EmptyP2aP2msP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshUnknownPattern2
 * @property {_1m1w1y24hBpsPercentRatioPattern} empty
 * @property {_1m1w1y24hBpsPercentRatioPattern} p2a
 * @property {_1m1w1y24hBpsPercentRatioPattern} p2ms
 * @property {_1m1w1y24hBpsPercentRatioPattern} p2pk33
 * @property {_1m1w1y24hBpsPercentRatioPattern} p2pk65
 * @property {_1m1w1y24hBpsPercentRatioPattern} p2pkh
 * @property {_1m1w1y24hBpsPercentRatioPattern} p2sh
 * @property {_1m1w1y24hBpsPercentRatioPattern} p2tr
 * @property {_1m1w1y24hBpsPercentRatioPattern} p2wpkh
 * @property {_1m1w1y24hBpsPercentRatioPattern} p2wsh
 * @property {_1m1w1y24hBpsPercentRatioPattern} unknown
 */

/**
 * Create a EmptyP2aP2msP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshUnknownPattern2 pattern node
 * @param {BrkClient} client
 * @param {string} acc - Accumulated series name
 * @returns {EmptyP2aP2msP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshUnknownPattern2}
 */
function createEmptyP2aP2msP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshUnknownPattern2(client, acc) {
  return {
    empty: create_1m1w1y24hBpsPercentRatioPattern(client, _m(acc, 'empty_outputs_prevout')),
    p2a: create_1m1w1y24hBpsPercentRatioPattern(client, _m(acc, 'p2a_prevout')),
    p2ms: create_1m1w1y24hBpsPercentRatioPattern(client, _m(acc, 'p2ms_prevout')),
    p2pk33: create_1m1w1y24hBpsPercentRatioPattern(client, _m(acc, 'p2pk33_prevout')),
    p2pk65: create_1m1w1y24hBpsPercentRatioPattern(client, _m(acc, 'p2pk65_prevout')),
    p2pkh: create_1m1w1y24hBpsPercentRatioPattern(client, _m(acc, 'p2pkh_prevout')),
    p2sh: create_1m1w1y24hBpsPercentRatioPattern(client, _m(acc, 'p2sh_prevout')),
    p2tr: create_1m1w1y24hBpsPercentRatioPattern(client, _m(acc, 'p2tr_prevout')),
    p2wpkh: create_1m1w1y24hBpsPercentRatioPattern(client, _m(acc, 'p2wpkh_prevout')),
    p2wsh: create_1m1w1y24hBpsPercentRatioPattern(client, _m(acc, 'p2wsh_prevout')),
    unknown: create_1m1w1y24hBpsPercentRatioPattern(client, _m(acc, 'unknown_outputs_prevout')),
  };
}

/**
 * @template T
 * @typedef {Object} AverageBaseCumulativeMaxMedianMinPct10Pct25Pct75Pct90SumPattern
 * @property {_1m1w1y24hPattern<T>} average
 * @property {SeriesPattern18<T>} base
 * @property {SeriesPattern1<T>} cumulative
 * @property {_1m1w1y24hPattern<T>} max
 * @property {_1m1w1y24hPattern<T>} median
 * @property {_1m1w1y24hPattern<T>} min
 * @property {_1m1w1y24hPattern<T>} pct10
 * @property {_1m1w1y24hPattern<T>} pct25
 * @property {_1m1w1y24hPattern<T>} pct75
 * @property {_1m1w1y24hPattern<T>} pct90
 * @property {_1m1w1y24hPattern<T>} sum
 */

/**
 * Create a AverageBaseCumulativeMaxMedianMinPct10Pct25Pct75Pct90SumPattern pattern node
 * @template T
 * @param {BrkClient} client
 * @param {string} acc - Accumulated series name
 * @returns {AverageBaseCumulativeMaxMedianMinPct10Pct25Pct75Pct90SumPattern<T>}
 */
function createAverageBaseCumulativeMaxMedianMinPct10Pct25Pct75Pct90SumPattern(client, acc) {
  return {
    average: create_1m1w1y24hPattern(client, _m(acc, 'average')),
    base: createSeriesPattern18(client, acc),
    cumulative: createSeriesPattern1(client, _m(acc, 'cumulative')),
    max: create_1m1w1y24hPattern(client, _m(acc, 'max')),
    median: create_1m1w1y24hPattern(client, _m(acc, 'median')),
    min: create_1m1w1y24hPattern(client, _m(acc, 'min')),
    pct10: create_1m1w1y24hPattern(client, _m(acc, 'pct10')),
    pct25: create_1m1w1y24hPattern(client, _m(acc, 'pct25')),
    pct75: create_1m1w1y24hPattern(client, _m(acc, 'pct75')),
    pct90: create_1m1w1y24hPattern(client, _m(acc, 'pct90')),
    sum: create_1m1w1y24hPattern(client, _m(acc, 'sum')),
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
 * @typedef {Object} IndexPct0Pct1Pct2Pct5Pct95Pct98Pct99ScorePattern
 * @property {SeriesPattern1<StoredI8>} index
 * @property {CentsSatsUsdPattern} pct05
 * @property {CentsSatsUsdPattern} pct1
 * @property {CentsSatsUsdPattern} pct2
 * @property {CentsSatsUsdPattern} pct5
 * @property {CentsSatsUsdPattern} pct95
 * @property {CentsSatsUsdPattern} pct98
 * @property {CentsSatsUsdPattern} pct99
 * @property {CentsSatsUsdPattern} pct995
 * @property {SeriesPattern1<StoredI8>} score
 */

/**
 * Create a IndexPct0Pct1Pct2Pct5Pct95Pct98Pct99ScorePattern pattern node
 * @param {BrkClient} client
 * @param {string} acc - Accumulated series name
 * @returns {IndexPct0Pct1Pct2Pct5Pct95Pct98Pct99ScorePattern}
 */
function createIndexPct0Pct1Pct2Pct5Pct95Pct98Pct99ScorePattern(client, acc) {
  return {
    index: createSeriesPattern1(client, _m(acc, 'index')),
    pct05: createCentsSatsUsdPattern(client, _m(acc, 'pct0_5')),
    pct1: createCentsSatsUsdPattern(client, _m(acc, 'pct01')),
    pct2: createCentsSatsUsdPattern(client, _m(acc, 'pct02')),
    pct5: createCentsSatsUsdPattern(client, _m(acc, 'pct05')),
    pct95: createCentsSatsUsdPattern(client, _m(acc, 'pct95')),
    pct98: createCentsSatsUsdPattern(client, _m(acc, 'pct98')),
    pct99: createCentsSatsUsdPattern(client, _m(acc, 'pct99')),
    pct995: createCentsSatsUsdPattern(client, _m(acc, 'pct99_5')),
    score: createSeriesPattern1(client, _m(acc, 'score')),
  };
}

/**
 * @typedef {Object} AllP2aP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshPattern6
 * @property {AverageBlockCumulativeSumPattern<StoredU64>} all
 * @property {AverageBlockCumulativeSumPattern<StoredU64>} p2a
 * @property {AverageBlockCumulativeSumPattern<StoredU64>} p2pk33
 * @property {AverageBlockCumulativeSumPattern<StoredU64>} p2pk65
 * @property {AverageBlockCumulativeSumPattern<StoredU64>} p2pkh
 * @property {AverageBlockCumulativeSumPattern<StoredU64>} p2sh
 * @property {AverageBlockCumulativeSumPattern<StoredU64>} p2tr
 * @property {AverageBlockCumulativeSumPattern<StoredU64>} p2wpkh
 * @property {AverageBlockCumulativeSumPattern<StoredU64>} p2wsh
 */

/**
 * Create a AllP2aP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshPattern6 pattern node
 * @param {BrkClient} client
 * @param {string} acc - Accumulated series name
 * @returns {AllP2aP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshPattern6}
 */
function createAllP2aP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshPattern6(client, acc) {
  return {
    all: createAverageBlockCumulativeSumPattern(client, acc),
    p2a: createAverageBlockCumulativeSumPattern(client, _p('p2a', acc)),
    p2pk33: createAverageBlockCumulativeSumPattern(client, _p('p2pk33', acc)),
    p2pk65: createAverageBlockCumulativeSumPattern(client, _p('p2pk65', acc)),
    p2pkh: createAverageBlockCumulativeSumPattern(client, _p('p2pkh', acc)),
    p2sh: createAverageBlockCumulativeSumPattern(client, _p('p2sh', acc)),
    p2tr: createAverageBlockCumulativeSumPattern(client, _p('p2tr', acc)),
    p2wpkh: createAverageBlockCumulativeSumPattern(client, _p('p2wpkh', acc)),
    p2wsh: createAverageBlockCumulativeSumPattern(client, _p('p2wsh', acc)),
  };
}

/**
 * @typedef {Object} AllP2aP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshPattern5
 * @property {BpsPercentRatioPattern2} all
 * @property {BpsPercentRatioPattern2} p2a
 * @property {BpsPercentRatioPattern2} p2pk33
 * @property {BpsPercentRatioPattern2} p2pk65
 * @property {BpsPercentRatioPattern2} p2pkh
 * @property {BpsPercentRatioPattern2} p2sh
 * @property {BpsPercentRatioPattern2} p2tr
 * @property {BpsPercentRatioPattern2} p2wpkh
 * @property {BpsPercentRatioPattern2} p2wsh
 */

/**
 * Create a AllP2aP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshPattern5 pattern node
 * @param {BrkClient} client
 * @param {string} acc - Accumulated series name
 * @returns {AllP2aP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshPattern5}
 */
function createAllP2aP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshPattern5(client, acc) {
  return {
    all: createBpsPercentRatioPattern2(client, acc),
    p2a: createBpsPercentRatioPattern2(client, _p('p2a', acc)),
    p2pk33: createBpsPercentRatioPattern2(client, _p('p2pk33', acc)),
    p2pk65: createBpsPercentRatioPattern2(client, _p('p2pk65', acc)),
    p2pkh: createBpsPercentRatioPattern2(client, _p('p2pkh', acc)),
    p2sh: createBpsPercentRatioPattern2(client, _p('p2sh', acc)),
    p2tr: createBpsPercentRatioPattern2(client, _p('p2tr', acc)),
    p2wpkh: createBpsPercentRatioPattern2(client, _p('p2wpkh', acc)),
    p2wsh: createBpsPercentRatioPattern2(client, _p('p2wsh', acc)),
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
 * @param {BrkClient} client
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
 * @typedef {Object} AllP2aP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshPattern7
 * @property {_1m1w1y24hBpsPercentRatioPattern} all
 * @property {_1m1w1y24hBpsPercentRatioPattern} p2a
 * @property {_1m1w1y24hBpsPercentRatioPattern} p2pk33
 * @property {_1m1w1y24hBpsPercentRatioPattern} p2pk65
 * @property {_1m1w1y24hBpsPercentRatioPattern} p2pkh
 * @property {_1m1w1y24hBpsPercentRatioPattern} p2sh
 * @property {_1m1w1y24hBpsPercentRatioPattern} p2tr
 * @property {_1m1w1y24hBpsPercentRatioPattern} p2wpkh
 * @property {_1m1w1y24hBpsPercentRatioPattern} p2wsh
 */

/**
 * Create a AllP2aP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshPattern7 pattern node
 * @param {BrkClient} client
 * @param {string} acc - Accumulated series name
 * @returns {AllP2aP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshPattern7}
 */
function createAllP2aP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshPattern7(client, acc) {
  return {
    all: create_1m1w1y24hBpsPercentRatioPattern(client, acc),
    p2a: create_1m1w1y24hBpsPercentRatioPattern(client, _p('p2a', acc)),
    p2pk33: create_1m1w1y24hBpsPercentRatioPattern(client, _p('p2pk33', acc)),
    p2pk65: create_1m1w1y24hBpsPercentRatioPattern(client, _p('p2pk65', acc)),
    p2pkh: create_1m1w1y24hBpsPercentRatioPattern(client, _p('p2pkh', acc)),
    p2sh: create_1m1w1y24hBpsPercentRatioPattern(client, _p('p2sh', acc)),
    p2tr: create_1m1w1y24hBpsPercentRatioPattern(client, _p('p2tr', acc)),
    p2wpkh: create_1m1w1y24hBpsPercentRatioPattern(client, _p('p2wpkh', acc)),
    p2wsh: create_1m1w1y24hBpsPercentRatioPattern(client, _p('p2wsh', acc)),
  };
}

/**
 * @typedef {Object} AverageMaxMedianMinPct10Pct25Pct75Pct90SumPattern
 * @property {_1m1w1y24hPattern<StoredF32>} average
 * @property {_1m1w1y24hPattern<StoredU64>} max
 * @property {_1m1w1y24hPattern<StoredU64>} median
 * @property {_1m1w1y24hPattern<StoredU64>} min
 * @property {_1m1w1y24hPattern<StoredU64>} pct10
 * @property {_1m1w1y24hPattern<StoredU64>} pct25
 * @property {_1m1w1y24hPattern<StoredU64>} pct75
 * @property {_1m1w1y24hPattern<StoredU64>} pct90
 * @property {_1m1w1y24hPattern<StoredU64>} sum
 */

/**
 * Create a AverageMaxMedianMinPct10Pct25Pct75Pct90SumPattern pattern node
 * @param {BrkClient} client
 * @param {string} acc - Accumulated series name
 * @returns {AverageMaxMedianMinPct10Pct25Pct75Pct90SumPattern}
 */
function createAverageMaxMedianMinPct10Pct25Pct75Pct90SumPattern(client, acc) {
  return {
    average: create_1m1w1y24hPattern(client, _m(acc, 'average')),
    max: create_1m1w1y24hPattern(client, _m(acc, 'max')),
    median: create_1m1w1y24hPattern(client, _m(acc, 'median')),
    min: create_1m1w1y24hPattern(client, _m(acc, 'min')),
    pct10: create_1m1w1y24hPattern(client, _m(acc, 'pct10')),
    pct25: create_1m1w1y24hPattern(client, _m(acc, 'pct25')),
    pct75: create_1m1w1y24hPattern(client, _m(acc, 'pct75')),
    pct90: create_1m1w1y24hPattern(client, _m(acc, 'pct90')),
    sum: create_1m1w1y24hPattern(client, _m(acc, 'sum')),
  };
}

/**
 * @typedef {Object} CapitalizedGrossInvestedLossNetNuplProfitSentimentPattern2
 * @property {SeriesPattern18<CentsSquaredSats>} capitalizedCapInLossRaw
 * @property {SeriesPattern18<CentsSquaredSats>} capitalizedCapInProfitRaw
 * @property {CentsUsdPattern3} grossPnl
 * @property {InPattern2} investedCapital
 * @property {CentsNegativeToUsdPattern2} loss
 * @property {CentsToUsdPattern3} netPnl
 * @property {BpsRatioPattern} nupl
 * @property {CentsToUsdPattern4} profit
 * @property {GreedNetPainPattern} sentiment
 */

/**
 * Create a CapitalizedGrossInvestedLossNetNuplProfitSentimentPattern2 pattern node
 * @param {BrkClient} client
 * @param {string} acc - Accumulated series name
 * @returns {CapitalizedGrossInvestedLossNetNuplProfitSentimentPattern2}
 */
function createCapitalizedGrossInvestedLossNetNuplProfitSentimentPattern2(client, acc) {
  return {
    capitalizedCapInLossRaw: createSeriesPattern18(client, _m(acc, 'capitalized_cap_in_loss_raw')),
    capitalizedCapInProfitRaw: createSeriesPattern18(client, _m(acc, 'capitalized_cap_in_profit_raw')),
    grossPnl: createCentsUsdPattern3(client, _m(acc, 'unrealized_gross_pnl')),
    investedCapital: createInPattern2(client, _m(acc, 'invested_capital_in')),
    loss: createCentsNegativeToUsdPattern2(client, _m(acc, 'unrealized_loss')),
    netPnl: createCentsToUsdPattern3(client, _m(acc, 'net_unrealized_pnl')),
    nupl: createBpsRatioPattern(client, _m(acc, 'nupl')),
    profit: createCentsToUsdPattern4(client, _m(acc, 'unrealized_profit')),
    sentiment: createGreedNetPainPattern(client, acc),
  };
}

/**
 * @typedef {Object} BpsCentsPercentilesRatioSatsSmaStdUsdPattern
 * @property {SeriesPattern1<BasisPoints32>} bps
 * @property {SeriesPattern1<Cents>} cents
 * @property {Pct0Pct1Pct2Pct5Pct95Pct98Pct99Pattern} percentiles
 * @property {SeriesPattern1<StoredF32>} ratio
 * @property {SeriesPattern1<SatsFract>} sats
 * @property {_1m1w1y2y4yAllPattern} sma
 * @property {_1y2y4yAllPattern} stdDev
 * @property {SeriesPattern1<Dollars>} usd
 */

/**
 * @typedef {Object} Pct0Pct1Pct2Pct5Pct95Pct98Pct99Pattern
 * @property {BpsPriceRatioPattern} pct05
 * @property {BpsPriceRatioPattern} pct1
 * @property {BpsPriceRatioPattern} pct2
 * @property {BpsPriceRatioPattern} pct5
 * @property {BpsPriceRatioPattern} pct95
 * @property {BpsPriceRatioPattern} pct98
 * @property {BpsPriceRatioPattern} pct99
 * @property {BpsPriceRatioPattern} pct995
 */

/**
 * Create a Pct0Pct1Pct2Pct5Pct95Pct98Pct99Pattern pattern node
 * @param {BrkClient} client
 * @param {string} acc - Accumulated series name
 * @returns {Pct0Pct1Pct2Pct5Pct95Pct98Pct99Pattern}
 */
function createPct0Pct1Pct2Pct5Pct95Pct98Pct99Pattern(client, acc) {
  return {
    pct05: createBpsPriceRatioPattern(client, acc, 'pct0_5'),
    pct1: createBpsPriceRatioPattern(client, acc, 'pct1'),
    pct2: createBpsPriceRatioPattern(client, acc, 'pct2'),
    pct5: createBpsPriceRatioPattern(client, acc, 'pct5'),
    pct95: createBpsPriceRatioPattern(client, acc, 'pct95'),
    pct98: createBpsPriceRatioPattern(client, acc, 'pct98'),
    pct99: createBpsPriceRatioPattern(client, acc, 'pct99'),
    pct995: createBpsPriceRatioPattern(client, acc, 'pct99_5'),
  };
}

/**
 * @typedef {Object} _10y2y3y4y5y6y8yPattern
 * @property {BpsPercentRatioPattern} _10y
 * @property {BpsPercentRatioPattern} _2y
 * @property {BpsPercentRatioPattern} _3y
 * @property {BpsPercentRatioPattern} _4y
 * @property {BpsPercentRatioPattern} _5y
 * @property {BpsPercentRatioPattern} _6y
 * @property {BpsPercentRatioPattern} _8y
 */

/**
 * Create a _10y2y3y4y5y6y8yPattern pattern node
 * @param {BrkClient} client
 * @param {string} acc - Accumulated series name
 * @returns {_10y2y3y4y5y6y8yPattern}
 */
function create_10y2y3y4y5y6y8yPattern(client, acc) {
  return {
    _10y: createBpsPercentRatioPattern(client, _m(acc, '10y')),
    _2y: createBpsPercentRatioPattern(client, _m(acc, '2y')),
    _3y: createBpsPercentRatioPattern(client, _m(acc, '3y')),
    _4y: createBpsPercentRatioPattern(client, _m(acc, '4y')),
    _5y: createBpsPercentRatioPattern(client, _m(acc, '5y')),
    _6y: createBpsPercentRatioPattern(client, _m(acc, '6y')),
    _8y: createBpsPercentRatioPattern(client, _m(acc, '8y')),
  };
}

/**
 * @typedef {Object} _1m1w1y24hBpsPercentRatioPattern
 * @property {BpsPercentRatioPattern2} _1m
 * @property {BpsPercentRatioPattern2} _1w
 * @property {BpsPercentRatioPattern2} _1y
 * @property {BpsPercentRatioPattern2} _24h
 * @property {SeriesPattern1<BasisPoints16>} bps
 * @property {SeriesPattern1<StoredF32>} percent
 * @property {SeriesPattern1<StoredF32>} ratio
 */

/**
 * Create a _1m1w1y24hBpsPercentRatioPattern pattern node
 * @param {BrkClient} client
 * @param {string} acc - Accumulated series name
 * @returns {_1m1w1y24hBpsPercentRatioPattern}
 */
function create_1m1w1y24hBpsPercentRatioPattern(client, acc) {
  return {
    _1m: createBpsPercentRatioPattern2(client, _m(acc, '1m')),
    _1w: createBpsPercentRatioPattern2(client, _m(acc, '1w')),
    _1y: createBpsPercentRatioPattern2(client, _m(acc, '1y')),
    _24h: createBpsPercentRatioPattern2(client, _m(acc, '24h')),
    bps: createSeriesPattern1(client, _m(acc, 'bps')),
    percent: createSeriesPattern1(client, acc),
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
 * @property {_1m1w1y24hBpsPercentRatioPattern} spendableOutputToReusedAddrShare
 */

/**
 * @typedef {Object} ActivityCostInvestedOutputsRealizedSupplyUnrealizedPattern2
 * @property {CoindaysCoinyearsDormancyTransferPattern} activity
 * @property {InMaxMinPerSupplyPattern} costBasis
 * @property {InPattern} investedCapital
 * @property {SpendingSpentUnspentPattern} outputs
 * @property {CapCapitalizedGrossLossMvrvNetPeakPriceProfitSellSoprPattern2} realized
 * @property {DeltaDominanceHalfInTotalPattern2} supply
 * @property {CapitalizedGrossInvestedLossNetNuplProfitSentimentPattern2} unrealized
 */

/**
 * @typedef {Object} CapLossMvrvNetPriceProfitSoprPattern
 * @property {CentsDeltaUsdPattern} cap
 * @property {BlockCumulativeNegativeSumPattern} loss
 * @property {SeriesPattern1<StoredF32>} mvrv
 * @property {BlockCumulativeDeltaSumPattern} netPnl
 * @property {BpsCentsRatioSatsUsdPattern} price
 * @property {BlockCumulativeSumPattern} profit
 * @property {RatioValuePattern} sopr
 */

/**
 * Create a CapLossMvrvNetPriceProfitSoprPattern pattern node
 * @param {BrkClient} client
 * @param {string} acc - Accumulated series name
 * @returns {CapLossMvrvNetPriceProfitSoprPattern}
 */
function createCapLossMvrvNetPriceProfitSoprPattern(client, acc) {
  return {
    cap: createCentsDeltaUsdPattern(client, _m(acc, 'realized_cap')),
    loss: createBlockCumulativeNegativeSumPattern(client, _m(acc, 'realized_loss')),
    mvrv: createSeriesPattern1(client, _m(acc, 'mvrv')),
    netPnl: createBlockCumulativeDeltaSumPattern(client, _m(acc, 'net_realized_pnl')),
    price: createBpsCentsRatioSatsUsdPattern(client, _m(acc, 'realized_price')),
    profit: createBlockCumulativeSumPattern(client, _m(acc, 'realized_profit')),
    sopr: createRatioValuePattern(client, acc),
  };
}

/**
 * @typedef {Object} InMaxMinPerSupplyPattern
 * @property {PerPattern} inLoss
 * @property {PerPattern} inProfit
 * @property {CentsSatsUsdPattern} max
 * @property {CentsSatsUsdPattern} min
 * @property {Pct05Pct10Pct15Pct20Pct25Pct30Pct35Pct40Pct45Pct50Pct55Pct60Pct65Pct70Pct75Pct80Pct85Pct90Pct95Pattern} perCoin
 * @property {Pct05Pct10Pct15Pct20Pct25Pct30Pct35Pct40Pct45Pct50Pct55Pct60Pct65Pct70Pct75Pct80Pct85Pct90Pct95Pattern} perDollar
 * @property {BpsPercentRatioPattern2} supplyDensity
 */

/**
 * Create a InMaxMinPerSupplyPattern pattern node
 * @param {BrkClient} client
 * @param {string} acc - Accumulated series name
 * @returns {InMaxMinPerSupplyPattern}
 */
function createInMaxMinPerSupplyPattern(client, acc) {
  return {
    inLoss: createPerPattern(client, _m(acc, 'cost_basis_in_loss_per')),
    inProfit: createPerPattern(client, _m(acc, 'cost_basis_in_profit_per')),
    max: createCentsSatsUsdPattern(client, _m(acc, 'cost_basis_max')),
    min: createCentsSatsUsdPattern(client, _m(acc, 'cost_basis_min')),
    perCoin: createPct05Pct10Pct15Pct20Pct25Pct30Pct35Pct40Pct45Pct50Pct55Pct60Pct65Pct70Pct75Pct80Pct85Pct90Pct95Pattern(client, _m(acc, 'cost_basis_per_coin')),
    perDollar: createPct05Pct10Pct15Pct20Pct25Pct30Pct35Pct40Pct45Pct50Pct55Pct60Pct65Pct70Pct75Pct80Pct85Pct90Pct95Pattern(client, _m(acc, 'cost_basis_per_dollar')),
    supplyDensity: createBpsPercentRatioPattern2(client, _m(acc, 'supply_density')),
  };
}

/**
 * @typedef {Object} MaxMedianMinPct10Pct25Pct75Pct90Pattern2
 * @property {SeriesPattern18<Weight>} max
 * @property {SeriesPattern18<Weight>} median
 * @property {SeriesPattern18<Weight>} min
 * @property {SeriesPattern18<Weight>} pct10
 * @property {SeriesPattern18<Weight>} pct25
 * @property {SeriesPattern18<Weight>} pct75
 * @property {SeriesPattern18<Weight>} pct90
 */

/**
 * Create a MaxMedianMinPct10Pct25Pct75Pct90Pattern2 pattern node
 * @param {BrkClient} client
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
 * @param {BrkClient} client
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
 * @typedef {Object} _1m1w1y2y4yAllPattern
 * @property {BpsRatioPattern2} _1m
 * @property {BpsRatioPattern2} _1w
 * @property {BpsRatioPattern2} _1y
 * @property {BpsRatioPattern2} _2y
 * @property {BpsRatioPattern2} _4y
 * @property {BpsRatioPattern2} all
 */

/**
 * Create a _1m1w1y2y4yAllPattern pattern node
 * @param {BrkClient} client
 * @param {string} acc - Accumulated series name
 * @returns {_1m1w1y2y4yAllPattern}
 */
function create_1m1w1y2y4yAllPattern(client, acc) {
  return {
    _1m: createBpsRatioPattern2(client, _m(acc, '1m')),
    _1w: createBpsRatioPattern2(client, _m(acc, '1w')),
    _1y: createBpsRatioPattern2(client, _m(acc, '1y')),
    _2y: createBpsRatioPattern2(client, _m(acc, '2y')),
    _4y: createBpsRatioPattern2(client, _m(acc, '4y')),
    all: createBpsRatioPattern2(client, _m(acc, 'all')),
  };
}

/**
 * @typedef {Object} ActivityAddrOutputsRealizedSupplyUnrealizedPattern
 * @property {TransferPattern} activity
 * @property {BaseDeltaPattern} addrCount
 * @property {SpendingSpentUnspentPattern} outputs
 * @property {CapLossMvrvPriceProfitPattern} realized
 * @property {DeltaDominanceTotalPattern} supply
 * @property {NuplPattern} unrealized
 */

/**
 * Create a ActivityAddrOutputsRealizedSupplyUnrealizedPattern pattern node
 * @param {BrkClient} client
 * @param {string} acc - Accumulated series name
 * @returns {ActivityAddrOutputsRealizedSupplyUnrealizedPattern}
 */
function createActivityAddrOutputsRealizedSupplyUnrealizedPattern(client, acc) {
  return {
    activity: createTransferPattern(client, _m(acc, 'transfer_volume')),
    addrCount: createBaseDeltaPattern(client, _m(acc, 'addr_count')),
    outputs: createSpendingSpentUnspentPattern(client, acc),
    realized: createCapLossMvrvPriceProfitPattern(client, acc),
    supply: createDeltaDominanceTotalPattern(client, _m(acc, 'supply')),
    unrealized: createNuplPattern(client, _m(acc, 'nupl')),
  };
}

/**
 * @typedef {Object} AverageBlockCumulativeInSumPattern
 * @property {_1m1w1y24hPattern3} average
 * @property {BtcCentsSatsUsdPattern3} block
 * @property {BtcCentsSatsUsdPattern} cumulative
 * @property {AverageBlockCumulativeSumPattern3} inLoss
 * @property {AverageBlockCumulativeSumPattern3} inProfit
 * @property {_1m1w1y24hPattern4} sum
 */

/**
 * Create a AverageBlockCumulativeInSumPattern pattern node
 * @param {BrkClient} client
 * @param {string} acc - Accumulated series name
 * @returns {AverageBlockCumulativeInSumPattern}
 */
function createAverageBlockCumulativeInSumPattern(client, acc) {
  return {
    average: create_1m1w1y24hPattern3(client, _m(acc, 'average')),
    block: createBtcCentsSatsUsdPattern3(client, acc),
    cumulative: createBtcCentsSatsUsdPattern(client, _m(acc, 'cumulative')),
    inLoss: createAverageBlockCumulativeSumPattern3(client, _m(acc, 'in_loss')),
    inProfit: createAverageBlockCumulativeSumPattern3(client, _m(acc, 'in_profit')),
    sum: create_1m1w1y24hPattern4(client, _m(acc, 'sum')),
  };
}

/**
 * @typedef {Object} BpsCentsPercentilesRatioSatsUsdPattern
 * @property {SeriesPattern1<BasisPoints32>} bps
 * @property {SeriesPattern1<Cents>} cents
 * @property {Pct0Pct1Pct2Pct5Pct95Pct98Pct99Pattern} percentiles
 * @property {SeriesPattern1<StoredF32>} ratio
 * @property {SeriesPattern1<SatsFract>} sats
 * @property {SeriesPattern1<Dollars>} usd
 */

/**
 * Create a BpsCentsPercentilesRatioSatsUsdPattern pattern node
 * @param {BrkClient} client
 * @param {string} acc - Accumulated series name
 * @returns {BpsCentsPercentilesRatioSatsUsdPattern}
 */
function createBpsCentsPercentilesRatioSatsUsdPattern(client, acc) {
  return {
    bps: createSeriesPattern1(client, _m(acc, 'ratio_bps')),
    cents: createSeriesPattern1(client, _m(acc, 'cents')),
    percentiles: createPct0Pct1Pct2Pct5Pct95Pct98Pct99Pattern(client, acc),
    ratio: createSeriesPattern1(client, _m(acc, 'ratio')),
    sats: createSeriesPattern1(client, _m(acc, 'sats')),
    usd: createSeriesPattern1(client, acc),
  };
}

/**
 * @typedef {Object} CentsNegativeToUsdPattern2
 * @property {SeriesPattern1<Cents>} cents
 * @property {SeriesPattern1<Dollars>} negative
 * @property {BpsPercentRatioPattern2} toMcap
 * @property {BpsPercentRatioPattern2} toOwnGrossPnl
 * @property {BpsPercentRatioPattern4} toOwnMcap
 * @property {SeriesPattern1<Dollars>} usd
 */

/**
 * Create a CentsNegativeToUsdPattern2 pattern node
 * @param {BrkClient} client
 * @param {string} acc - Accumulated series name
 * @returns {CentsNegativeToUsdPattern2}
 */
function createCentsNegativeToUsdPattern2(client, acc) {
  return {
    cents: createSeriesPattern1(client, _m(acc, 'cents')),
    negative: createSeriesPattern1(client, _m(acc, 'neg')),
    toMcap: createBpsPercentRatioPattern2(client, _m(acc, 'to_mcap')),
    toOwnGrossPnl: createBpsPercentRatioPattern2(client, _m(acc, 'to_own_gross_pnl')),
    toOwnMcap: createBpsPercentRatioPattern4(client, _m(acc, 'to_own_mcap')),
    usd: createSeriesPattern1(client, acc),
  };
}

/**
 * @typedef {Object} DeltaDominanceHalfInTotalPattern2
 * @property {AbsoluteRatePattern3} delta
 * @property {BpsPercentRatioPattern2} dominance
 * @property {BtcCentsSatsUsdPattern} half
 * @property {BtcCentsSatsShareUsdPattern} inLoss
 * @property {BtcCentsSatsShareUsdPattern} inProfit
 * @property {BtcCentsSatsUsdPattern} total
 */

/**
 * Create a DeltaDominanceHalfInTotalPattern2 pattern node
 * @param {BrkClient} client
 * @param {string} acc - Accumulated series name
 * @returns {DeltaDominanceHalfInTotalPattern2}
 */
function createDeltaDominanceHalfInTotalPattern2(client, acc) {
  return {
    delta: createAbsoluteRatePattern3(client, _m(acc, 'delta')),
    dominance: createBpsPercentRatioPattern2(client, _m(acc, 'dominance')),
    half: createBtcCentsSatsUsdPattern(client, _m(acc, 'half')),
    inLoss: createBtcCentsSatsShareUsdPattern(client, _m(acc, 'in_loss')),
    inProfit: createBtcCentsSatsShareUsdPattern(client, _m(acc, 'in_profit')),
    total: createBtcCentsSatsUsdPattern(client, acc),
  };
}

/**
 * @typedef {Object} DeltaDominanceHalfInTotalPattern
 * @property {AbsoluteRatePattern3} delta
 * @property {BpsPercentRatioPattern2} dominance
 * @property {BtcCentsSatsUsdPattern} half
 * @property {BtcCentsSatsUsdPattern} inLoss
 * @property {BtcCentsSatsUsdPattern} inProfit
 * @property {BtcCentsSatsUsdPattern} total
 */

/**
 * Create a DeltaDominanceHalfInTotalPattern pattern node
 * @param {BrkClient} client
 * @param {string} acc - Accumulated series name
 * @returns {DeltaDominanceHalfInTotalPattern}
 */
function createDeltaDominanceHalfInTotalPattern(client, acc) {
  return {
    delta: createAbsoluteRatePattern3(client, _m(acc, 'delta')),
    dominance: createBpsPercentRatioPattern2(client, _m(acc, 'dominance')),
    half: createBtcCentsSatsUsdPattern(client, _m(acc, 'half')),
    inLoss: createBtcCentsSatsUsdPattern(client, _m(acc, 'in_loss')),
    inProfit: createBtcCentsSatsUsdPattern(client, _m(acc, 'in_profit')),
    total: createBtcCentsSatsUsdPattern(client, acc),
  };
}

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
 * @param {BrkClient} client
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
 * @param {BrkClient} client
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
 * @typedef {Object} ActiveBidirectionalReactivatedReceivingSendingPattern
 * @property {_1m1w1y24hBlockPattern} active
 * @property {_1m1w1y24hBlockPattern} bidirectional
 * @property {_1m1w1y24hBlockPattern} reactivated
 * @property {_1m1w1y24hBlockPattern} receiving
 * @property {_1m1w1y24hBlockPattern} sending
 */

/**
 * Create a ActiveBidirectionalReactivatedReceivingSendingPattern pattern node
 * @param {BrkClient} client
 * @param {string} acc - Accumulated series name
 * @returns {ActiveBidirectionalReactivatedReceivingSendingPattern}
 */
function createActiveBidirectionalReactivatedReceivingSendingPattern(client, acc) {
  return {
    active: create_1m1w1y24hBlockPattern(client, _m(acc, 'active_addrs')),
    bidirectional: create_1m1w1y24hBlockPattern(client, _m(acc, 'bidirectional_addrs')),
    reactivated: create_1m1w1y24hBlockPattern(client, _m(acc, 'reactivated_addrs')),
    receiving: create_1m1w1y24hBlockPattern(client, _m(acc, 'receiving_addrs')),
    sending: create_1m1w1y24hBlockPattern(client, _m(acc, 'sending_addrs')),
  };
}

/**
 * @typedef {Object} ActivityOutputsRealizedSupplyUnrealizedPattern
 * @property {CoindaysTransferPattern} activity
 * @property {SpendingSpentUnspentPattern} outputs
 * @property {CapLossMvrvNetPriceProfitSoprPattern} realized
 * @property {DeltaDominanceHalfInTotalPattern} supply
 * @property {LossNetNuplProfitPattern} unrealized
 */

/**
 * Create a ActivityOutputsRealizedSupplyUnrealizedPattern pattern node
 * @param {BrkClient} client
 * @param {string} acc - Accumulated series name
 * @returns {ActivityOutputsRealizedSupplyUnrealizedPattern}
 */
function createActivityOutputsRealizedSupplyUnrealizedPattern(client, acc) {
  return {
    activity: createCoindaysTransferPattern(client, acc),
    outputs: createSpendingSpentUnspentPattern(client, acc),
    realized: createCapLossMvrvNetPriceProfitSoprPattern(client, acc),
    supply: createDeltaDominanceHalfInTotalPattern(client, _m(acc, 'supply')),
    unrealized: createLossNetNuplProfitPattern(client, acc),
  };
}

/**
 * @typedef {Object} ActivityOutputsRealizedSupplyUnrealizedPattern3
 * @property {TransferPattern} activity
 * @property {SpendingSpentUnspentPattern} outputs
 * @property {CapLossMvrvPriceProfitPattern} realized
 * @property {DeltaDominanceHalfInTotalPattern} supply
 * @property {LossNuplProfitPattern} unrealized
 */

/**
 * Create a ActivityOutputsRealizedSupplyUnrealizedPattern3 pattern node
 * @param {BrkClient} client
 * @param {string} acc - Accumulated series name
 * @returns {ActivityOutputsRealizedSupplyUnrealizedPattern3}
 */
function createActivityOutputsRealizedSupplyUnrealizedPattern3(client, acc) {
  return {
    activity: createTransferPattern(client, _m(acc, 'transfer_volume')),
    outputs: createSpendingSpentUnspentPattern(client, acc),
    realized: createCapLossMvrvPriceProfitPattern(client, acc),
    supply: createDeltaDominanceHalfInTotalPattern(client, _m(acc, 'supply')),
    unrealized: createLossNuplProfitPattern(client, acc),
  };
}

/**
 * @typedef {Object} ActivityOutputsRealizedSupplyUnrealizedPattern2
 * @property {TransferPattern} activity
 * @property {SpendingSpentUnspentPattern} outputs
 * @property {CapLossMvrvPriceProfitPattern} realized
 * @property {DeltaDominanceTotalPattern} supply
 * @property {NuplPattern} unrealized
 */

/**
 * Create a ActivityOutputsRealizedSupplyUnrealizedPattern2 pattern node
 * @param {BrkClient} client
 * @param {string} acc - Accumulated series name
 * @returns {ActivityOutputsRealizedSupplyUnrealizedPattern2}
 */
function createActivityOutputsRealizedSupplyUnrealizedPattern2(client, acc) {
  return {
    activity: createTransferPattern(client, _m(acc, 'transfer_volume')),
    outputs: createSpendingSpentUnspentPattern(client, acc),
    realized: createCapLossMvrvPriceProfitPattern(client, acc),
    supply: createDeltaDominanceTotalPattern(client, _m(acc, 'supply')),
    unrealized: createNuplPattern(client, _m(acc, 'nupl')),
  };
}

/**
 * @typedef {Object} BlockChangeCumulativeDeltaSumPattern
 * @property {CentsUsdPattern4} block
 * @property {ToPattern} change1m
 * @property {CentsUsdPattern} cumulative
 * @property {AbsoluteRatePattern2} delta
 * @property {_1m1w1y24hPattern5} sum
 */

/**
 * Create a BlockChangeCumulativeDeltaSumPattern pattern node
 * @param {BrkClient} client
 * @param {string} acc - Accumulated series name
 * @returns {BlockChangeCumulativeDeltaSumPattern}
 */
function createBlockChangeCumulativeDeltaSumPattern(client, acc) {
  return {
    block: createCentsUsdPattern4(client, _m(acc, 'realized_pnl')),
    change1m: createToPattern(client, _m(acc, 'pnl_change_1m_to')),
    cumulative: createCentsUsdPattern(client, _m(acc, 'realized_pnl_cumulative')),
    delta: createAbsoluteRatePattern2(client, _m(acc, 'realized_pnl_delta')),
    sum: create_1m1w1y24hPattern5(client, _m(acc, 'realized_pnl_sum')),
  };
}

/**
 * @typedef {Object} BpsCentsRatioSatsUsdPattern
 * @property {SeriesPattern1<BasisPoints32>} bps
 * @property {SeriesPattern1<Cents>} cents
 * @property {SeriesPattern1<StoredF32>} ratio
 * @property {SeriesPattern1<SatsFract>} sats
 * @property {SeriesPattern1<Dollars>} usd
 */

/**
 * Create a BpsCentsRatioSatsUsdPattern pattern node
 * @param {BrkClient} client
 * @param {string} acc - Accumulated series name
 * @returns {BpsCentsRatioSatsUsdPattern}
 */
function createBpsCentsRatioSatsUsdPattern(client, acc) {
  return {
    bps: createSeriesPattern1(client, _m(acc, 'ratio_bps')),
    cents: createSeriesPattern1(client, _m(acc, 'cents')),
    ratio: createSeriesPattern1(client, _m(acc, 'ratio')),
    sats: createSeriesPattern1(client, _m(acc, 'sats')),
    usd: createSeriesPattern1(client, acc),
  };
}

/**
 * @typedef {Object} BtcCentsDeltaSatsUsdPattern
 * @property {SeriesPattern1<Bitcoin>} btc
 * @property {SeriesPattern1<Cents>} cents
 * @property {AbsoluteRatePattern3} delta
 * @property {SeriesPattern1<Sats>} sats
 * @property {SeriesPattern1<Dollars>} usd
 */

/**
 * Create a BtcCentsDeltaSatsUsdPattern pattern node
 * @param {BrkClient} client
 * @param {string} acc - Accumulated series name
 * @returns {BtcCentsDeltaSatsUsdPattern}
 */
function createBtcCentsDeltaSatsUsdPattern(client, acc) {
  return {
    btc: createSeriesPattern1(client, acc),
    cents: createSeriesPattern1(client, _m(acc, 'cents')),
    delta: createAbsoluteRatePattern3(client, _m(acc, 'delta')),
    sats: createSeriesPattern1(client, _m(acc, 'sats')),
    usd: createSeriesPattern1(client, _m(acc, 'usd')),
  };
}

/**
 * @typedef {Object} BtcCentsSatsShareUsdPattern
 * @property {SeriesPattern1<Bitcoin>} btc
 * @property {SeriesPattern1<Cents>} cents
 * @property {SeriesPattern1<Sats>} sats
 * @property {BpsPercentRatioPattern2} share
 * @property {SeriesPattern1<Dollars>} usd
 */

/**
 * Create a BtcCentsSatsShareUsdPattern pattern node
 * @param {BrkClient} client
 * @param {string} acc - Accumulated series name
 * @returns {BtcCentsSatsShareUsdPattern}
 */
function createBtcCentsSatsShareUsdPattern(client, acc) {
  return {
    btc: createSeriesPattern1(client, acc),
    cents: createSeriesPattern1(client, _m(acc, 'cents')),
    sats: createSeriesPattern1(client, _m(acc, 'sats')),
    share: createBpsPercentRatioPattern2(client, _m(acc, 'share')),
    usd: createSeriesPattern1(client, _m(acc, 'usd')),
  };
}

/**
 * @typedef {Object} CapLossMvrvPriceProfitPattern
 * @property {CentsDeltaUsdPattern} cap
 * @property {BlockCumulativeSumPattern} loss
 * @property {SeriesPattern1<StoredF32>} mvrv
 * @property {BpsCentsRatioSatsUsdPattern} price
 * @property {BlockCumulativeSumPattern} profit
 */

/**
 * Create a CapLossMvrvPriceProfitPattern pattern node
 * @param {BrkClient} client
 * @param {string} acc - Accumulated series name
 * @returns {CapLossMvrvPriceProfitPattern}
 */
function createCapLossMvrvPriceProfitPattern(client, acc) {
  return {
    cap: createCentsDeltaUsdPattern(client, _m(acc, 'realized_cap')),
    loss: createBlockCumulativeSumPattern(client, _m(acc, 'realized_loss')),
    mvrv: createSeriesPattern1(client, _m(acc, 'mvrv')),
    price: createBpsCentsRatioSatsUsdPattern(client, _m(acc, 'realized_price')),
    profit: createBlockCumulativeSumPattern(client, _m(acc, 'realized_profit')),
  };
}

/**
 * @typedef {Object} CentsToUsdPattern4
 * @property {SeriesPattern1<Cents>} cents
 * @property {BpsPercentRatioPattern2} toMcap
 * @property {BpsPercentRatioPattern2} toOwnGrossPnl
 * @property {BpsPercentRatioPattern2} toOwnMcap
 * @property {SeriesPattern1<Dollars>} usd
 */

/**
 * Create a CentsToUsdPattern4 pattern node
 * @param {BrkClient} client
 * @param {string} acc - Accumulated series name
 * @returns {CentsToUsdPattern4}
 */
function createCentsToUsdPattern4(client, acc) {
  return {
    cents: createSeriesPattern1(client, _m(acc, 'cents')),
    toMcap: createBpsPercentRatioPattern2(client, _m(acc, 'to_mcap')),
    toOwnGrossPnl: createBpsPercentRatioPattern2(client, _m(acc, 'to_own_gross_pnl')),
    toOwnMcap: createBpsPercentRatioPattern2(client, _m(acc, 'to_own_mcap')),
    usd: createSeriesPattern1(client, acc),
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
 * @typedef {Object} PhsReboundThsPattern
 * @property {SeriesPattern1<StoredF32>} phs
 * @property {SeriesPattern1<StoredF32>} phsMin
 * @property {BpsPercentRatioPattern} rebound
 * @property {SeriesPattern1<StoredF32>} ths
 * @property {SeriesPattern1<StoredF32>} thsMin
 */

/**
 * Create a PhsReboundThsPattern pattern node
 * @param {BrkClient} client
 * @param {string} acc - Accumulated series name
 * @returns {PhsReboundThsPattern}
 */
function createPhsReboundThsPattern(client, acc) {
  return {
    phs: createSeriesPattern1(client, _m(acc, 'phs')),
    phsMin: createSeriesPattern1(client, _m(acc, 'phs_min')),
    rebound: createBpsPercentRatioPattern(client, _m(acc, 'rebound')),
    ths: createSeriesPattern1(client, _m(acc, 'ths')),
    thsMin: createSeriesPattern1(client, _m(acc, 'ths_min')),
  };
}

/**
 * @typedef {Object} _1m1w1y24hPattern2
 * @property {BpsPercentRatioPattern} _1m
 * @property {BpsPercentRatioPattern} _1w
 * @property {BpsPercentRatioPattern} _1y
 * @property {BpsPercentRatioPattern} _24h
 */

/**
 * Create a _1m1w1y24hPattern2 pattern node
 * @param {BrkClient} client
 * @param {string} acc - Accumulated series name
 * @returns {_1m1w1y24hPattern2}
 */
function create_1m1w1y24hPattern2(client, acc) {
  return {
    _1m: createBpsPercentRatioPattern(client, _m(acc, '1m_rate')),
    _1w: createBpsPercentRatioPattern(client, _m(acc, '1w_rate')),
    _1y: createBpsPercentRatioPattern(client, _m(acc, '1y_rate')),
    _24h: createBpsPercentRatioPattern(client, _m(acc, '24h_rate')),
  };
}

/**
 * @typedef {Object} _1m1w1y24hPattern8
 * @property {BpsPercentRatioPattern4} _1m
 * @property {BpsPercentRatioPattern4} _1w
 * @property {BpsPercentRatioPattern4} _1y
 * @property {BpsPercentRatioPattern4} _24h
 */

/**
 * Create a _1m1w1y24hPattern8 pattern node
 * @param {BrkClient} client
 * @param {string} acc - Accumulated series name
 * @returns {_1m1w1y24hPattern8}
 */
function create_1m1w1y24hPattern8(client, acc) {
  return {
    _1m: createBpsPercentRatioPattern4(client, _m(acc, '1m')),
    _1w: createBpsPercentRatioPattern4(client, _m(acc, '1w')),
    _1y: createBpsPercentRatioPattern4(client, _m(acc, '1y')),
    _24h: createBpsPercentRatioPattern4(client, _m(acc, '24h')),
  };
}

/**
 * @typedef {Object} _1m1w1y24hPattern4
 * @property {BtcCentsSatsUsdPattern} _1m
 * @property {BtcCentsSatsUsdPattern} _1w
 * @property {BtcCentsSatsUsdPattern} _1y
 * @property {BtcCentsSatsUsdPattern} _24h
 */

/**
 * Create a _1m1w1y24hPattern4 pattern node
 * @param {BrkClient} client
 * @param {string} acc - Accumulated series name
 * @returns {_1m1w1y24hPattern4}
 */
function create_1m1w1y24hPattern4(client, acc) {
  return {
    _1m: createBtcCentsSatsUsdPattern(client, _m(acc, '1m')),
    _1w: createBtcCentsSatsUsdPattern(client, _m(acc, '1w')),
    _1y: createBtcCentsSatsUsdPattern(client, _m(acc, '1y')),
    _24h: createBtcCentsSatsUsdPattern(client, _m(acc, '24h')),
  };
}

/**
 * @typedef {Object} _1m1w1y24hPattern3
 * @property {BtcCentsSatsUsdPattern2} _1m
 * @property {BtcCentsSatsUsdPattern2} _1w
 * @property {BtcCentsSatsUsdPattern2} _1y
 * @property {BtcCentsSatsUsdPattern2} _24h
 */

/**
 * Create a _1m1w1y24hPattern3 pattern node
 * @param {BrkClient} client
 * @param {string} acc - Accumulated series name
 * @returns {_1m1w1y24hPattern3}
 */
function create_1m1w1y24hPattern3(client, acc) {
  return {
    _1m: createBtcCentsSatsUsdPattern2(client, _m(acc, '1m')),
    _1w: createBtcCentsSatsUsdPattern2(client, _m(acc, '1w')),
    _1y: createBtcCentsSatsUsdPattern2(client, _m(acc, '1y')),
    _24h: createBtcCentsSatsUsdPattern2(client, _m(acc, '24h')),
  };
}

/**
 * @typedef {Object} _1m1w1y24hPattern7
 * @property {BtcSatsPattern} _1m
 * @property {BtcSatsPattern} _1w
 * @property {BtcSatsPattern} _1y
 * @property {BtcSatsPattern} _24h
 */

/**
 * Create a _1m1w1y24hPattern7 pattern node
 * @param {BrkClient} client
 * @param {string} acc - Accumulated series name
 * @returns {_1m1w1y24hPattern7}
 */
function create_1m1w1y24hPattern7(client, acc) {
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
 * @param {BrkClient} client
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
 * @typedef {Object} _1m1w1y24hPattern5
 * @property {CentsUsdPattern} _1m
 * @property {CentsUsdPattern} _1w
 * @property {CentsUsdPattern} _1y
 * @property {CentsUsdPattern} _24h
 */

/**
 * Create a _1m1w1y24hPattern5 pattern node
 * @param {BrkClient} client
 * @param {string} acc - Accumulated series name
 * @returns {_1m1w1y24hPattern5}
 */
function create_1m1w1y24hPattern5(client, acc) {
  return {
    _1m: createCentsUsdPattern(client, _m(acc, '1m')),
    _1w: createCentsUsdPattern(client, _m(acc, '1w')),
    _1y: createCentsUsdPattern(client, _m(acc, '1y')),
    _24h: createCentsUsdPattern(client, _m(acc, '24h')),
  };
}

/**
 * @typedef {Object} _1m1w1y24hPattern6
 * @property {CentsUsdPattern3} _1m
 * @property {CentsUsdPattern3} _1w
 * @property {CentsUsdPattern3} _1y
 * @property {CentsUsdPattern3} _24h
 */

/**
 * Create a _1m1w1y24hPattern6 pattern node
 * @param {BrkClient} client
 * @param {string} acc - Accumulated series name
 * @returns {_1m1w1y24hPattern6}
 */
function create_1m1w1y24hPattern6(client, acc) {
  return {
    _1m: createCentsUsdPattern3(client, _m(acc, '1m')),
    _1w: createCentsUsdPattern3(client, _m(acc, '1w')),
    _1y: createCentsUsdPattern3(client, _m(acc, '1y')),
    _24h: createCentsUsdPattern3(client, _m(acc, '24h')),
  };
}

/**
 * @typedef {Object} _1y2y4yAllPattern
 * @property {_0sdM0M1M1sdM2M2sdM3sdP0P1P1sdP2P2sdP3sdSdZscorePattern} _1y
 * @property {_0sdM0M1M1sdM2M2sdM3sdP0P1P1sdP2P2sdP3sdSdZscorePattern} _2y
 * @property {_0sdM0M1M1sdM2M2sdM3sdP0P1P1sdP2P2sdP3sdSdZscorePattern} _4y
 * @property {_0sdM0M1M1sdM2M2sdM3sdP0P1P1sdP2P2sdP3sdSdZscorePattern} all
 */

/**
 * @typedef {Object} AverageBlockCumulativeSumPattern2
 * @property {_1m1w1y24hPattern<StoredF32>} average
 * @property {SeriesPattern18<StoredU32>} block
 * @property {SeriesPattern1<StoredU64>} cumulative
 * @property {_1m1w1y24hPattern<StoredU64>} sum
 */

/**
 * Create a AverageBlockCumulativeSumPattern2 pattern node
 * @param {BrkClient} client
 * @param {string} acc - Accumulated series name
 * @returns {AverageBlockCumulativeSumPattern2}
 */
function createAverageBlockCumulativeSumPattern2(client, acc) {
  return {
    average: create_1m1w1y24hPattern(client, _m(acc, 'average')),
    block: createSeriesPattern18(client, acc),
    cumulative: createSeriesPattern1(client, _m(acc, 'cumulative')),
    sum: create_1m1w1y24hPattern(client, _m(acc, 'sum')),
  };
}

/**
 * @typedef {Object} AverageBlockCumulativeSumPattern3
 * @property {_1m1w1y24hPattern3} average
 * @property {BtcCentsSatsUsdPattern3} block
 * @property {BtcCentsSatsUsdPattern} cumulative
 * @property {_1m1w1y24hPattern4} sum
 */

/**
 * Create a AverageBlockCumulativeSumPattern3 pattern node
 * @param {BrkClient} client
 * @param {string} acc - Accumulated series name
 * @returns {AverageBlockCumulativeSumPattern3}
 */
function createAverageBlockCumulativeSumPattern3(client, acc) {
  return {
    average: create_1m1w1y24hPattern3(client, _m(acc, 'average')),
    block: createBtcCentsSatsUsdPattern3(client, acc),
    cumulative: createBtcCentsSatsUsdPattern(client, _m(acc, 'cumulative')),
    sum: create_1m1w1y24hPattern4(client, _m(acc, 'sum')),
  };
}

/**
 * @typedef {Object} BlockCumulativeNegativeSumPattern
 * @property {CentsUsdPattern2} block
 * @property {CentsUsdPattern3} cumulative
 * @property {BaseSumPattern} negative
 * @property {_1m1w1y24hPattern6} sum
 */

/**
 * Create a BlockCumulativeNegativeSumPattern pattern node
 * @param {BrkClient} client
 * @param {string} acc - Accumulated series name
 * @returns {BlockCumulativeNegativeSumPattern}
 */
function createBlockCumulativeNegativeSumPattern(client, acc) {
  return {
    block: createCentsUsdPattern2(client, acc),
    cumulative: createCentsUsdPattern3(client, _m(acc, 'cumulative')),
    negative: createBaseSumPattern(client, _m(acc, 'neg')),
    sum: create_1m1w1y24hPattern6(client, _m(acc, 'sum')),
  };
}

/**
 * @typedef {Object} BlockCumulativeDeltaSumPattern
 * @property {CentsUsdPattern4} block
 * @property {CentsUsdPattern} cumulative
 * @property {AbsoluteRatePattern2} delta
 * @property {_1m1w1y24hPattern5} sum
 */

/**
 * Create a BlockCumulativeDeltaSumPattern pattern node
 * @param {BrkClient} client
 * @param {string} acc - Accumulated series name
 * @returns {BlockCumulativeDeltaSumPattern}
 */
function createBlockCumulativeDeltaSumPattern(client, acc) {
  return {
    block: createCentsUsdPattern4(client, acc),
    cumulative: createCentsUsdPattern(client, _m(acc, 'cumulative')),
    delta: createAbsoluteRatePattern2(client, _m(acc, 'delta')),
    sum: create_1m1w1y24hPattern5(client, _m(acc, 'sum')),
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
 * @param {BrkClient} client
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
 * @param {BrkClient} client
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
 * @param {BrkClient} client
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
 * @typedef {Object} CentsDeltaToUsdPattern
 * @property {SeriesPattern1<Cents>} cents
 * @property {AbsoluteRatePattern2} delta
 * @property {BpsPercentRatioPattern4} toOwnMcap
 * @property {SeriesPattern1<Dollars>} usd
 */

/**
 * Create a CentsDeltaToUsdPattern pattern node
 * @param {BrkClient} client
 * @param {string} acc - Accumulated series name
 * @returns {CentsDeltaToUsdPattern}
 */
function createCentsDeltaToUsdPattern(client, acc) {
  return {
    cents: createSeriesPattern1(client, _m(acc, 'cents')),
    delta: createAbsoluteRatePattern2(client, _m(acc, 'delta')),
    toOwnMcap: createBpsPercentRatioPattern4(client, _m(acc, 'to_own_mcap')),
    usd: createSeriesPattern1(client, acc),
  };
}

/**
 * @typedef {Object} CentsToUsdPattern3
 * @property {SeriesPattern1<CentsSigned>} cents
 * @property {BpsPercentRatioPattern} toOwnGrossPnl
 * @property {BpsPercentRatioPattern} toOwnMcap
 * @property {SeriesPattern1<Dollars>} usd
 */

/**
 * Create a CentsToUsdPattern3 pattern node
 * @param {BrkClient} client
 * @param {string} acc - Accumulated series name
 * @returns {CentsToUsdPattern3}
 */
function createCentsToUsdPattern3(client, acc) {
  return {
    cents: createSeriesPattern1(client, _m(acc, 'cents')),
    toOwnGrossPnl: createBpsPercentRatioPattern(client, _m(acc, 'to_own_gross_pnl')),
    toOwnMcap: createBpsPercentRatioPattern(client, _m(acc, 'to_own_mcap')),
    usd: createSeriesPattern1(client, acc),
  };
}

/**
 * @typedef {Object} CoindaysCoinyearsDormancyTransferPattern
 * @property {AverageBlockCumulativeSumPattern<StoredF64>} coindaysDestroyed
 * @property {SeriesPattern1<StoredF64>} coinyearsDestroyed
 * @property {_1m1w1y24hPattern<StoredF32>} dormancy
 * @property {AverageBlockCumulativeInSumPattern} transferVolume
 */

/**
 * Create a CoindaysCoinyearsDormancyTransferPattern pattern node
 * @param {BrkClient} client
 * @param {string} acc - Accumulated series name
 * @returns {CoindaysCoinyearsDormancyTransferPattern}
 */
function createCoindaysCoinyearsDormancyTransferPattern(client, acc) {
  return {
    coindaysDestroyed: createAverageBlockCumulativeSumPattern(client, _m(acc, 'coindays_destroyed')),
    coinyearsDestroyed: createSeriesPattern1(client, _m(acc, 'coinyears_destroyed')),
    dormancy: create_1m1w1y24hPattern(client, _m(acc, 'dormancy')),
    transferVolume: createAverageBlockCumulativeInSumPattern(client, _m(acc, 'transfer_volume')),
  };
}

/**
 * @typedef {Object} LossNetNuplProfitPattern
 * @property {CentsNegativeUsdPattern} loss
 * @property {CentsUsdPattern} netPnl
 * @property {BpsRatioPattern} nupl
 * @property {CentsUsdPattern3} profit
 */

/**
 * Create a LossNetNuplProfitPattern pattern node
 * @param {BrkClient} client
 * @param {string} acc - Accumulated series name
 * @returns {LossNetNuplProfitPattern}
 */
function createLossNetNuplProfitPattern(client, acc) {
  return {
    loss: createCentsNegativeUsdPattern(client, _m(acc, 'unrealized_loss')),
    netPnl: createCentsUsdPattern(client, _m(acc, 'net_unrealized_pnl')),
    nupl: createBpsRatioPattern(client, _m(acc, 'nupl')),
    profit: createCentsUsdPattern3(client, _m(acc, 'unrealized_profit')),
  };
}

/**
 * @typedef {Object} NuplRealizedSupplyUnrealizedPattern
 * @property {BpsRatioPattern} nupl
 * @property {AllSthPattern} realizedCap
 * @property {AllSthPattern2} supply
 * @property {AllSthPattern} unrealizedPnl
 */

/**
 * Create a NuplRealizedSupplyUnrealizedPattern pattern node
 * @param {BrkClient} client
 * @param {string} acc - Accumulated series name
 * @returns {NuplRealizedSupplyUnrealizedPattern}
 */
function createNuplRealizedSupplyUnrealizedPattern(client, acc) {
  return {
    nupl: createBpsRatioPattern(client, _m(acc, 'nupl')),
    realizedCap: createAllSthPattern(client, acc, 'realized_cap'),
    supply: createAllSthPattern2(client, acc),
    unrealizedPnl: createAllSthPattern(client, acc, 'unrealized_pnl'),
  };
}

/**
 * @template T
 * @typedef {Object} _1m1w1y24hPattern
 * @property {SeriesPattern1<T>} _1m
 * @property {SeriesPattern1<T>} _1w
 * @property {SeriesPattern1<T>} _1y
 * @property {SeriesPattern1<T>} _24h
 */

/**
 * Create a _1m1w1y24hPattern pattern node
 * @template T
 * @param {BrkClient} client
 * @param {string} acc - Accumulated series name
 * @returns {_1m1w1y24hPattern<T>}
 */
function create_1m1w1y24hPattern(client, acc) {
  return {
    _1m: createSeriesPattern1(client, _m(acc, '1m')),
    _1w: createSeriesPattern1(client, _m(acc, '1w')),
    _1y: createSeriesPattern1(client, _m(acc, '1y')),
    _24h: createSeriesPattern1(client, _m(acc, '24h')),
  };
}

/**
 * @template T
 * @typedef {Object} AverageBlockCumulativeSumPattern
 * @property {_1m1w1y24hPattern<T>} average
 * @property {SeriesPattern18<T>} block
 * @property {SeriesPattern1<T>} cumulative
 * @property {_1m1w1y24hPattern<T>} sum
 */

/**
 * Create a AverageBlockCumulativeSumPattern pattern node
 * @template T
 * @param {BrkClient} client
 * @param {string} acc - Accumulated series name
 * @returns {AverageBlockCumulativeSumPattern<T>}
 */
function createAverageBlockCumulativeSumPattern(client, acc) {
  return {
    average: create_1m1w1y24hPattern(client, _m(acc, 'average')),
    block: createSeriesPattern18(client, acc),
    cumulative: createSeriesPattern1(client, _m(acc, 'cumulative')),
    sum: create_1m1w1y24hPattern(client, _m(acc, 'sum')),
  };
}

/**
 * @typedef {Object} AdjustedRatioValuePattern
 * @property {RatioTransferValuePattern} adjusted
 * @property {_1m1w1y24hPattern<StoredF64>} ratio
 * @property {AverageBlockCumulativeSumPattern<Cents>} valueDestroyed
 */

/**
 * Create a AdjustedRatioValuePattern pattern node
 * @param {BrkClient} client
 * @param {string} acc - Accumulated series name
 * @returns {AdjustedRatioValuePattern}
 */
function createAdjustedRatioValuePattern(client, acc) {
  return {
    adjusted: createRatioTransferValuePattern(client, acc),
    ratio: create_1m1w1y24hPattern(client, _m(acc, 'sopr')),
    valueDestroyed: createAverageBlockCumulativeSumPattern(client, _m(acc, 'value_destroyed')),
  };
}

/**
 * @typedef {Object} BlockCumulativeSumPattern
 * @property {CentsUsdPattern2} block
 * @property {CentsUsdPattern3} cumulative
 * @property {_1m1w1y24hPattern6} sum
 */

/**
 * Create a BlockCumulativeSumPattern pattern node
 * @param {BrkClient} client
 * @param {string} acc - Accumulated series name
 * @returns {BlockCumulativeSumPattern}
 */
function createBlockCumulativeSumPattern(client, acc) {
  return {
    block: createCentsUsdPattern2(client, acc),
    cumulative: createCentsUsdPattern3(client, _m(acc, 'cumulative')),
    sum: create_1m1w1y24hPattern6(client, _m(acc, 'sum')),
  };
}

/**
 * @typedef {Object} BlocksDominanceRewardsPattern
 * @property {AverageBlockCumulativeSumPattern2} blocksMined
 * @property {_1m1w1y24hBpsPercentRatioPattern} dominance
 * @property {AverageBlockCumulativeSumPattern3} rewards
 */

/**
 * Create a BlocksDominanceRewardsPattern pattern node
 * @param {BrkClient} client
 * @param {string} acc - Accumulated series name
 * @returns {BlocksDominanceRewardsPattern}
 */
function createBlocksDominanceRewardsPattern(client, acc) {
  return {
    blocksMined: createAverageBlockCumulativeSumPattern2(client, _m(acc, 'blocks_mined')),
    dominance: create_1m1w1y24hBpsPercentRatioPattern(client, _m(acc, 'dominance')),
    rewards: createAverageBlockCumulativeSumPattern3(client, _m(acc, 'rewards')),
  };
}

/**
 * @typedef {Object} BpsPercentRatioPattern2
 * @property {SeriesPattern1<BasisPoints16>} bps
 * @property {SeriesPattern1<StoredF32>} percent
 * @property {SeriesPattern1<StoredF32>} ratio
 */

/**
 * Create a BpsPercentRatioPattern2 pattern node
 * @param {BrkClient} client
 * @param {string} acc - Accumulated series name
 * @returns {BpsPercentRatioPattern2}
 */
function createBpsPercentRatioPattern2(client, acc) {
  return {
    bps: createSeriesPattern1(client, _m(acc, 'bps')),
    percent: createSeriesPattern1(client, acc),
    ratio: createSeriesPattern1(client, _m(acc, 'ratio')),
  };
}

/**
 * @typedef {Object} BpsPercentRatioPattern4
 * @property {SeriesPattern1<BasisPoints32>} bps
 * @property {SeriesPattern1<StoredF32>} percent
 * @property {SeriesPattern1<StoredF32>} ratio
 */

/**
 * Create a BpsPercentRatioPattern4 pattern node
 * @param {BrkClient} client
 * @param {string} acc - Accumulated series name
 * @returns {BpsPercentRatioPattern4}
 */
function createBpsPercentRatioPattern4(client, acc) {
  return {
    bps: createSeriesPattern1(client, _m(acc, 'bps')),
    percent: createSeriesPattern1(client, acc),
    ratio: createSeriesPattern1(client, _m(acc, 'ratio')),
  };
}

/**
 * @typedef {Object} BpsPriceRatioPattern
 * @property {SeriesPattern1<BasisPoints32>} bps
 * @property {CentsSatsUsdPattern} price
 * @property {SeriesPattern1<StoredF32>} ratio
 */

/**
 * Create a BpsPriceRatioPattern pattern node
 * @param {BrkClient} client
 * @param {string} acc - Accumulated series name
 * @param {string} disc - Discriminator suffix
 * @returns {BpsPriceRatioPattern}
 */
function createBpsPriceRatioPattern(client, acc, disc) {
  return {
    bps: createSeriesPattern1(client, _m(acc, `ratio_${disc}_bps`)),
    price: createCentsSatsUsdPattern(client, _m(acc, disc)),
    ratio: createSeriesPattern1(client, _m(_m(acc, 'ratio'), disc)),
  };
}

/**
 * @typedef {Object} BpsPercentRatioPattern5
 * @property {SeriesPattern1<BasisPointsSigned16>} bps
 * @property {SeriesPattern1<StoredF32>} percent
 * @property {SeriesPattern1<StoredF32>} ratio
 */

/**
 * Create a BpsPercentRatioPattern5 pattern node
 * @param {BrkClient} client
 * @param {string} acc - Accumulated series name
 * @returns {BpsPercentRatioPattern5}
 */
function createBpsPercentRatioPattern5(client, acc) {
  return {
    bps: createSeriesPattern1(client, _m(acc, 'bps')),
    percent: createSeriesPattern1(client, acc),
    ratio: createSeriesPattern1(client, _m(acc, 'ratio')),
  };
}

/**
 * @typedef {Object} BpsPercentRatioPattern
 * @property {SeriesPattern1<BasisPointsSigned32>} bps
 * @property {SeriesPattern1<StoredF32>} percent
 * @property {SeriesPattern1<StoredF32>} ratio
 */

/**
 * Create a BpsPercentRatioPattern pattern node
 * @param {BrkClient} client
 * @param {string} acc - Accumulated series name
 * @returns {BpsPercentRatioPattern}
 */
function createBpsPercentRatioPattern(client, acc) {
  return {
    bps: createSeriesPattern1(client, _m(acc, 'bps')),
    percent: createSeriesPattern1(client, acc),
    ratio: createSeriesPattern1(client, _m(acc, 'ratio')),
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
 * @param {BrkClient} client
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
 * @property {AbsoluteRatePattern2} delta
 * @property {SeriesPattern1<Dollars>} usd
 */

/**
 * Create a CentsDeltaUsdPattern pattern node
 * @param {BrkClient} client
 * @param {string} acc - Accumulated series name
 * @returns {CentsDeltaUsdPattern}
 */
function createCentsDeltaUsdPattern(client, acc) {
  return {
    cents: createSeriesPattern1(client, _m(acc, 'cents')),
    delta: createAbsoluteRatePattern2(client, _m(acc, 'delta')),
    usd: createSeriesPattern1(client, acc),
  };
}

/**
 * @typedef {Object} CentsNegativeUsdPattern
 * @property {SeriesPattern1<Cents>} cents
 * @property {SeriesPattern1<Dollars>} negative
 * @property {SeriesPattern1<Dollars>} usd
 */

/**
 * Create a CentsNegativeUsdPattern pattern node
 * @param {BrkClient} client
 * @param {string} acc - Accumulated series name
 * @returns {CentsNegativeUsdPattern}
 */
function createCentsNegativeUsdPattern(client, acc) {
  return {
    cents: createSeriesPattern1(client, _m(acc, 'cents')),
    negative: createSeriesPattern1(client, _m(acc, 'neg')),
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
 * @param {BrkClient} client
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
 * @param {BrkClient} client
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
 * @property {AbsoluteRatePattern3} delta
 * @property {BpsPercentRatioPattern2} dominance
 * @property {BtcCentsSatsUsdPattern} total
 */

/**
 * Create a DeltaDominanceTotalPattern pattern node
 * @param {BrkClient} client
 * @param {string} acc - Accumulated series name
 * @returns {DeltaDominanceTotalPattern}
 */
function createDeltaDominanceTotalPattern(client, acc) {
  return {
    delta: createAbsoluteRatePattern3(client, _m(acc, 'delta')),
    dominance: createBpsPercentRatioPattern2(client, _m(acc, 'dominance')),
    total: createBtcCentsSatsUsdPattern(client, acc),
  };
}

/**
 * @typedef {Object} GreedNetPainPattern
 * @property {CentsUsdPattern3} greedIndex
 * @property {CentsUsdPattern} net
 * @property {CentsUsdPattern3} painIndex
 */

/**
 * Create a GreedNetPainPattern pattern node
 * @param {BrkClient} client
 * @param {string} acc - Accumulated series name
 * @returns {GreedNetPainPattern}
 */
function createGreedNetPainPattern(client, acc) {
  return {
    greedIndex: createCentsUsdPattern3(client, _m(acc, 'greed_index')),
    net: createCentsUsdPattern(client, _m(acc, 'net_sentiment')),
    painIndex: createCentsUsdPattern3(client, _m(acc, 'pain_index')),
  };
}

/**
 * @typedef {Object} LossNuplProfitPattern
 * @property {CentsNegativeUsdPattern} loss
 * @property {BpsRatioPattern} nupl
 * @property {CentsUsdPattern3} profit
 */

/**
 * Create a LossNuplProfitPattern pattern node
 * @param {BrkClient} client
 * @param {string} acc - Accumulated series name
 * @returns {LossNuplProfitPattern}
 */
function createLossNuplProfitPattern(client, acc) {
  return {
    loss: createCentsNegativeUsdPattern(client, _m(acc, 'unrealized_loss')),
    nupl: createBpsRatioPattern(client, _m(acc, 'nupl')),
    profit: createCentsUsdPattern3(client, _m(acc, 'unrealized_profit')),
  };
}

/**
 * @typedef {Object} RatioTransferValuePattern
 * @property {_1m1w1y24hPattern<StoredF64>} ratio
 * @property {AverageBlockCumulativeSumPattern<Cents>} transferVolume
 * @property {AverageBlockCumulativeSumPattern<Cents>} valueDestroyed
 */

/**
 * Create a RatioTransferValuePattern pattern node
 * @param {BrkClient} client
 * @param {string} acc - Accumulated series name
 * @returns {RatioTransferValuePattern}
 */
function createRatioTransferValuePattern(client, acc) {
  return {
    ratio: create_1m1w1y24hPattern(client, _m(acc, 'asopr')),
    transferVolume: createAverageBlockCumulativeSumPattern(client, _m(acc, 'adj_value_created')),
    valueDestroyed: createAverageBlockCumulativeSumPattern(client, _m(acc, 'adj_value_destroyed')),
  };
}

/**
 * @typedef {Object} RsiStochPattern
 * @property {BpsPercentRatioPattern2} rsi
 * @property {BpsPercentRatioPattern2} stochRsiD
 * @property {BpsPercentRatioPattern2} stochRsiK
 */

/**
 * Create a RsiStochPattern pattern node
 * @param {BrkClient} client
 * @param {string} acc - Accumulated series name
 * @param {string} disc - Discriminator suffix
 * @returns {RsiStochPattern}
 */
function createRsiStochPattern(client, acc, disc) {
  return {
    rsi: createBpsPercentRatioPattern2(client, _m(acc, disc)),
    stochRsiD: createBpsPercentRatioPattern2(client, _m(_m(acc, 'stoch_d'), disc)),
    stochRsiK: createBpsPercentRatioPattern2(client, _m(_m(acc, 'stoch_k'), disc)),
  };
}

/**
 * @typedef {Object} SpendingSpentUnspentPattern
 * @property {SeriesPattern1<StoredF32>} spendingRate
 * @property {AverageBlockCumulativeSumPattern2} spentCount
 * @property {BaseDeltaPattern} unspentCount
 */

/**
 * Create a SpendingSpentUnspentPattern pattern node
 * @param {BrkClient} client
 * @param {string} acc - Accumulated series name
 * @returns {SpendingSpentUnspentPattern}
 */
function createSpendingSpentUnspentPattern(client, acc) {
  return {
    spendingRate: createSeriesPattern1(client, _m(acc, 'spending_rate')),
    spentCount: createAverageBlockCumulativeSumPattern2(client, _m(acc, 'spent_utxo_count')),
    unspentCount: createBaseDeltaPattern(client, _m(acc, 'utxo_count')),
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
 * @param {BrkClient} client
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
 * @property {_1m1w1y24hPattern<StoredI64>} absolute
 * @property {_1m1w1y24hPattern2} rate
 */

/**
 * Create a AbsoluteRatePattern pattern node
 * @param {BrkClient} client
 * @param {string} acc - Accumulated series name
 * @returns {AbsoluteRatePattern}
 */
function createAbsoluteRatePattern(client, acc) {
  return {
    absolute: create_1m1w1y24hPattern(client, acc),
    rate: create_1m1w1y24hPattern2(client, acc),
  };
}

/**
 * @typedef {Object} AbsoluteRatePattern2
 * @property {_1m1w1y24hPattern5} absolute
 * @property {_1m1w1y24hPattern2} rate
 */

/**
 * Create a AbsoluteRatePattern2 pattern node
 * @param {BrkClient} client
 * @param {string} acc - Accumulated series name
 * @returns {AbsoluteRatePattern2}
 */
function createAbsoluteRatePattern2(client, acc) {
  return {
    absolute: create_1m1w1y24hPattern5(client, acc),
    rate: create_1m1w1y24hPattern2(client, acc),
  };
}

/**
 * @typedef {Object} AbsoluteRatePattern3
 * @property {_1m1w1y24hPattern7} absolute
 * @property {_1m1w1y24hPattern2} rate
 */

/**
 * Create a AbsoluteRatePattern3 pattern node
 * @param {BrkClient} client
 * @param {string} acc - Accumulated series name
 * @returns {AbsoluteRatePattern3}
 */
function createAbsoluteRatePattern3(client, acc) {
  return {
    absolute: create_1m1w1y24hPattern7(client, acc),
    rate: create_1m1w1y24hPattern2(client, acc),
  };
}

/**
 * @typedef {Object} AddrUtxoPattern
 * @property {BtcCentsSatsUsdPattern} addr
 * @property {BtcCentsSatsUsdPattern} utxo
 */

/**
 * Create a AddrUtxoPattern pattern node
 * @param {BrkClient} client
 * @param {string} acc - Accumulated series name
 * @returns {AddrUtxoPattern}
 */
function createAddrUtxoPattern(client, acc) {
  return {
    addr: createBtcCentsSatsUsdPattern(client, _m(acc, 'addr_amount')),
    utxo: createBtcCentsSatsUsdPattern(client, _m(acc, 'utxo_amount')),
  };
}

/**
 * @typedef {Object} AllSthPattern2
 * @property {BtcCentsDeltaSatsUsdPattern} all
 * @property {BtcCentsSatsUsdPattern} sth
 */

/**
 * Create a AllSthPattern2 pattern node
 * @param {BrkClient} client
 * @param {string} acc - Accumulated series name
 * @returns {AllSthPattern2}
 */
function createAllSthPattern2(client, acc) {
  return {
    all: createBtcCentsDeltaSatsUsdPattern(client, _m(acc, 'supply')),
    sth: createBtcCentsSatsUsdPattern(client, _m(acc, 'sth_supply')),
  };
}

/**
 * @typedef {Object} AllSthPattern
 * @property {SeriesPattern1<Dollars>} all
 * @property {SeriesPattern1<Dollars>} sth
 */

/**
 * Create a AllSthPattern pattern node
 * @param {BrkClient} client
 * @param {string} acc - Accumulated series name
 * @param {string} disc - Discriminator suffix
 * @returns {AllSthPattern}
 */
function createAllSthPattern(client, acc, disc) {
  return {
    all: createSeriesPattern1(client, _m(acc, disc)),
    sth: createSeriesPattern1(client, _m(_m(acc, 'sth'), disc)),
  };
}

/**
 * @typedef {Object} BaseSumPattern
 * @property {SeriesPattern18<Dollars>} base
 * @property {_1m1w1y24hPattern<Dollars>} sum
 */

/**
 * Create a BaseSumPattern pattern node
 * @param {BrkClient} client
 * @param {string} acc - Accumulated series name
 * @returns {BaseSumPattern}
 */
function createBaseSumPattern(client, acc) {
  return {
    base: createSeriesPattern18(client, acc),
    sum: create_1m1w1y24hPattern(client, _m(acc, 'sum')),
  };
}

/**
 * @typedef {Object} BaseDeltaPattern
 * @property {SeriesPattern1<StoredU64>} base
 * @property {AbsoluteRatePattern} delta
 */

/**
 * Create a BaseDeltaPattern pattern node
 * @param {BrkClient} client
 * @param {string} acc - Accumulated series name
 * @returns {BaseDeltaPattern}
 */
function createBaseDeltaPattern(client, acc) {
  return {
    base: createSeriesPattern1(client, acc),
    delta: createAbsoluteRatePattern(client, _m(acc, 'delta')),
  };
}

/**
 * @typedef {Object} BlockCumulativePattern
 * @property {BtcCentsSatsUsdPattern3} block
 * @property {BtcCentsSatsUsdPattern} cumulative
 */

/**
 * Create a BlockCumulativePattern pattern node
 * @param {BrkClient} client
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
 * @property {AverageBlockCumulativeSumPattern2} blocksMined
 * @property {BpsPercentRatioPattern2} dominance
 */

/**
 * Create a BlocksDominancePattern pattern node
 * @param {BrkClient} client
 * @param {string} acc - Accumulated series name
 * @returns {BlocksDominancePattern}
 */
function createBlocksDominancePattern(client, acc) {
  return {
    blocksMined: createAverageBlockCumulativeSumPattern2(client, _m(acc, 'blocks_mined')),
    dominance: createBpsPercentRatioPattern2(client, _m(acc, 'dominance')),
  };
}

/**
 * @typedef {Object} BpsRatioPattern2
 * @property {SeriesPattern1<BasisPoints32>} bps
 * @property {SeriesPattern1<StoredF32>} ratio
 */

/**
 * Create a BpsRatioPattern2 pattern node
 * @param {BrkClient} client
 * @param {string} acc - Accumulated series name
 * @returns {BpsRatioPattern2}
 */
function createBpsRatioPattern2(client, acc) {
  return {
    bps: createSeriesPattern1(client, _m(acc, 'bps')),
    ratio: createSeriesPattern1(client, acc),
  };
}

/**
 * @typedef {Object} BpsRatioPattern
 * @property {SeriesPattern1<BasisPointsSigned32>} bps
 * @property {SeriesPattern1<StoredF32>} ratio
 */

/**
 * Create a BpsRatioPattern pattern node
 * @param {BrkClient} client
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
 * @param {BrkClient} client
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
 * @typedef {Object} CentsUsdPattern3
 * @property {SeriesPattern1<Cents>} cents
 * @property {SeriesPattern1<Dollars>} usd
 */

/**
 * Create a CentsUsdPattern3 pattern node
 * @param {BrkClient} client
 * @param {string} acc - Accumulated series name
 * @returns {CentsUsdPattern3}
 */
function createCentsUsdPattern3(client, acc) {
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
 * @param {BrkClient} client
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
 * @typedef {Object} CentsUsdPattern
 * @property {SeriesPattern1<CentsSigned>} cents
 * @property {SeriesPattern1<Dollars>} usd
 */

/**
 * Create a CentsUsdPattern pattern node
 * @param {BrkClient} client
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
 * @typedef {Object} CentsUsdPattern4
 * @property {SeriesPattern18<CentsSigned>} cents
 * @property {SeriesPattern18<Dollars>} usd
 */

/**
 * Create a CentsUsdPattern4 pattern node
 * @param {BrkClient} client
 * @param {string} acc - Accumulated series name
 * @returns {CentsUsdPattern4}
 */
function createCentsUsdPattern4(client, acc) {
  return {
    cents: createSeriesPattern18(client, _m(acc, 'cents')),
    usd: createSeriesPattern18(client, acc),
  };
}

/**
 * @typedef {Object} CoindaysTransferPattern
 * @property {AverageBlockCumulativeSumPattern<StoredF64>} coindaysDestroyed
 * @property {AverageBlockCumulativeInSumPattern} transferVolume
 */

/**
 * Create a CoindaysTransferPattern pattern node
 * @param {BrkClient} client
 * @param {string} acc - Accumulated series name
 * @returns {CoindaysTransferPattern}
 */
function createCoindaysTransferPattern(client, acc) {
  return {
    coindaysDestroyed: createAverageBlockCumulativeSumPattern(client, _m(acc, 'coindays_destroyed')),
    transferVolume: createAverageBlockCumulativeInSumPattern(client, _m(acc, 'transfer_volume')),
  };
}

/**
 * @typedef {Object} FundedTotalPattern
 * @property {AllP2aP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshPattern4} funded
 * @property {AllP2aP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshPattern4} total
 */

/**
 * Create a FundedTotalPattern pattern node
 * @param {BrkClient} client
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
 * @typedef {Object} InPattern2
 * @property {CentsUsdPattern3} inLoss
 * @property {CentsUsdPattern3} inProfit
 */

/**
 * Create a InPattern2 pattern node
 * @param {BrkClient} client
 * @param {string} acc - Accumulated series name
 * @returns {InPattern2}
 */
function createInPattern2(client, acc) {
  return {
    inLoss: createCentsUsdPattern3(client, _m(acc, 'loss')),
    inProfit: createCentsUsdPattern3(client, _m(acc, 'profit')),
  };
}

/**
 * @typedef {Object} InPattern
 * @property {SharePattern} inLoss
 * @property {SharePattern} inProfit
 */

/**
 * Create a InPattern pattern node
 * @param {BrkClient} client
 * @param {string} acc - Accumulated series name
 * @returns {InPattern}
 */
function createInPattern(client, acc) {
  return {
    inLoss: createSharePattern(client, _m(acc, 'loss_share')),
    inProfit: createSharePattern(client, _m(acc, 'profit_share')),
  };
}

/**
 * @typedef {Object} PerPattern
 * @property {CentsSatsUsdPattern} perCoin
 * @property {CentsSatsUsdPattern} perDollar
 */

/**
 * Create a PerPattern pattern node
 * @param {BrkClient} client
 * @param {string} acc - Accumulated series name
 * @returns {PerPattern}
 */
function createPerPattern(client, acc) {
  return {
    perCoin: createCentsSatsUsdPattern(client, _m(acc, 'coin')),
    perDollar: createCentsSatsUsdPattern(client, _m(acc, 'dollar')),
  };
}

/**
 * @typedef {Object} PriceRatioPattern
 * @property {CentsSatsUsdPattern} price
 * @property {SeriesPattern1<StoredF32>} ratio
 */

/**
 * Create a PriceRatioPattern pattern node
 * @param {BrkClient} client
 * @param {string} acc - Accumulated series name
 * @param {string} disc - Discriminator suffix
 * @returns {PriceRatioPattern}
 */
function createPriceRatioPattern(client, acc, disc) {
  return {
    price: createCentsSatsUsdPattern(client, _m(acc, disc)),
    ratio: createSeriesPattern1(client, _m(_m(acc, 'ratio'), disc)),
  };
}

/**
 * @typedef {Object} RatioValuePattern2
 * @property {_1m1w1y24hPattern<StoredF64>} ratio
 * @property {AverageBlockCumulativeSumPattern<Cents>} valueDestroyed
 */

/**
 * Create a RatioValuePattern2 pattern node
 * @param {BrkClient} client
 * @param {string} acc - Accumulated series name
 * @returns {RatioValuePattern2}
 */
function createRatioValuePattern2(client, acc) {
  return {
    ratio: create_1m1w1y24hPattern(client, _m(acc, 'sopr')),
    valueDestroyed: createAverageBlockCumulativeSumPattern(client, _m(acc, 'value_destroyed')),
  };
}

/**
 * @typedef {Object} RatioValuePattern
 * @property {_24hPattern} ratio
 * @property {AverageBlockCumulativeSumPattern<Cents>} valueDestroyed
 */

/**
 * Create a RatioValuePattern pattern node
 * @param {BrkClient} client
 * @param {string} acc - Accumulated series name
 * @returns {RatioValuePattern}
 */
function createRatioValuePattern(client, acc) {
  return {
    ratio: create_24hPattern(client, _m(acc, 'sopr_24h')),
    valueDestroyed: createAverageBlockCumulativeSumPattern(client, _m(acc, 'value_destroyed')),
  };
}

/**
 * @typedef {Object} SdSmaPattern
 * @property {SeriesPattern1<StoredF32>} sd
 * @property {SeriesPattern1<StoredF32>} sma
 */

/**
 * @typedef {Object} ToPattern
 * @property {BpsPercentRatioPattern} toMcap
 * @property {BpsPercentRatioPattern} toRcap
 */

/**
 * Create a ToPattern pattern node
 * @param {BrkClient} client
 * @param {string} acc - Accumulated series name
 * @returns {ToPattern}
 */
function createToPattern(client, acc) {
  return {
    toMcap: createBpsPercentRatioPattern(client, _m(acc, 'mcap')),
    toRcap: createBpsPercentRatioPattern(client, _m(acc, 'rcap')),
  };
}

/**
 * @typedef {Object} _24hPattern
 * @property {SeriesPattern1<StoredF64>} _24h
 */

/**
 * Create a _24hPattern pattern node
 * @param {BrkClient} client
 * @param {string} acc - Accumulated series name
 * @returns {_24hPattern}
 */
function create_24hPattern(client, acc) {
  return {
    _24h: createSeriesPattern1(client, acc),
  };
}

/**
 * @typedef {Object} NuplPattern
 * @property {BpsRatioPattern} nupl
 */

/**
 * Create a NuplPattern pattern node
 * @param {BrkClient} client
 * @param {string} acc - Accumulated series name
 * @returns {NuplPattern}
 */
function createNuplPattern(client, acc) {
  return {
    nupl: createBpsRatioPattern(client, acc),
  };
}

/**
 * @typedef {Object} PricePattern
 * @property {BpsCentsPercentilesRatioSatsUsdPattern} price
 */

/**
 * Create a PricePattern pattern node
 * @param {BrkClient} client
 * @param {string} acc - Accumulated series name
 * @returns {PricePattern}
 */
function createPricePattern(client, acc) {
  return {
    price: createBpsCentsPercentilesRatioSatsUsdPattern(client, acc),
  };
}

/**
 * @typedef {Object} SharePattern
 * @property {BpsPercentRatioPattern2} share
 */

/**
 * Create a SharePattern pattern node
 * @param {BrkClient} client
 * @param {string} acc - Accumulated series name
 * @returns {SharePattern}
 */
function createSharePattern(client, acc) {
  return {
    share: createBpsPercentRatioPattern2(client, acc),
  };
}

/**
 * @typedef {Object} TransferPattern
 * @property {AverageBlockCumulativeSumPattern3} transferVolume
 */

/**
 * Create a TransferPattern pattern node
 * @param {BrkClient} client
 * @param {string} acc - Accumulated series name
 * @returns {TransferPattern}
 */
function createTransferPattern(client, acc) {
  return {
    transferVolume: createAverageBlockCumulativeSumPattern3(client, acc),
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
 * @property {SeriesTree_Mining} mining
 * @property {SeriesTree_Cointime} cointime
 * @property {SeriesTree_Constants} constants
 * @property {SeriesTree_Indexes} indexes
 * @property {SeriesTree_Indicators} indicators
 * @property {SeriesTree_Investing} investing
 * @property {SeriesTree_Market} market
 * @property {SeriesTree_Pools} pools
 * @property {SeriesTree_Price} price
 * @property {SeriesTree_Supply} supply
 * @property {SeriesTree_Cohorts} cohorts
 */

/**
 * @typedef {Object} SeriesTree_Blocks
 * @property {SeriesPattern18<BlockHash>} blockhash
 * @property {SeriesPattern18<CoinbaseTag>} coinbaseTag
 * @property {SeriesTree_Blocks_Difficulty} difficulty
 * @property {SeriesTree_Blocks_Time} time
 * @property {SeriesTree_Blocks_Size} size
 * @property {AverageBaseCumulativeMaxMedianMinPct10Pct25Pct75Pct90SumPattern<Weight>} weight
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
 * @property {BpsPercentRatioPattern} adjustment
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
 * @property {_1m1w1y24hPattern<StoredU64>} sum
 * @property {_1m1w1y24hPattern<StoredF32>} average
 * @property {_1m1w1y24hPattern<StoredU64>} min
 * @property {_1m1w1y24hPattern<StoredU64>} max
 * @property {_1m1w1y24hPattern<StoredU64>} pct10
 * @property {_1m1w1y24hPattern<StoredU64>} pct25
 * @property {_1m1w1y24hPattern<StoredU64>} median
 * @property {_1m1w1y24hPattern<StoredU64>} pct75
 * @property {_1m1w1y24hPattern<StoredU64>} pct90
 */

/**
 * @typedef {Object} SeriesTree_Blocks_Count
 * @property {_1m1w1y24hPattern<StoredU64>} target
 * @property {AverageBlockCumulativeSumPattern2} total
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
 * @property {SeriesPattern18<BasisPoints16>} bps
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
 * @property {SeriesTree_Transactions_Count} count
 * @property {SeriesTree_Transactions_Size} size
 * @property {SeriesTree_Transactions_Fees} fees
 * @property {SeriesTree_Transactions_Versions} versions
 * @property {SeriesTree_Transactions_Volume} volume
 */

/**
 * @typedef {Object} SeriesTree_Transactions_Raw
 * @property {SeriesPattern18<TxIndex>} firstTxIndex
 * @property {SeriesPattern19<Txid>} txid
 * @property {SeriesPattern19<TxVersion>} txVersion
 * @property {SeriesPattern19<RawLockTime>} rawLocktime
 * @property {SeriesPattern19<StoredU32>} baseSize
 * @property {SeriesPattern19<StoredU32>} totalSize
 * @property {SeriesPattern19<SigOps>} totalSigopCost
 * @property {SeriesPattern19<StoredBool>} isExplicitlyRbf
 * @property {SeriesPattern19<TxInIndex>} firstTxinIndex
 * @property {SeriesPattern19<TxOutIndex>} firstTxoutIndex
 */

/**
 * @typedef {Object} SeriesTree_Transactions_Count
 * @property {AverageBlockCumulativeMaxMedianMinPct10Pct25Pct75Pct90SumPattern} total
 */

/**
 * @typedef {Object} SeriesTree_Transactions_Size
 * @property {_6bBlockTxPattern<VSize>} vsize
 * @property {SeriesTree_Transactions_Size_Weight} weight
 */

/**
 * @typedef {Object} SeriesTree_Transactions_Size_Weight
 * @property {SeriesPattern19<Weight>} txIndex
 * @property {MaxMedianMinPct10Pct25Pct75Pct90Pattern2} block
 * @property {MaxMedianMinPct10Pct25Pct75Pct90Pattern2} _6b
 */

/**
 * @typedef {Object} SeriesTree_Transactions_Fees
 * @property {SeriesPattern19<Sats>} inputValue
 * @property {SeriesPattern19<Sats>} outputValue
 * @property {_6bBlockTxPattern<Sats>} fee
 * @property {SeriesPattern19<FeeRate>} feeRate
 * @property {_6bBlockTxPattern<FeeRate>} effectiveFeeRate
 */

/**
 * @typedef {Object} SeriesTree_Transactions_Versions
 * @property {AverageBlockCumulativeSumPattern<StoredU64>} v1
 * @property {AverageBlockCumulativeSumPattern<StoredU64>} v2
 * @property {AverageBlockCumulativeSumPattern<StoredU64>} v3
 */

/**
 * @typedef {Object} SeriesTree_Transactions_Volume
 * @property {AverageBlockCumulativeSumPattern3} transferVolume
 * @property {_1m1w1y24hPattern<StoredF32>} txPerSec
 */

/**
 * @typedef {Object} SeriesTree_Inputs
 * @property {SeriesTree_Inputs_Raw} raw
 * @property {SeriesTree_Inputs_Spent} spent
 * @property {CumulativeRollingSumPattern} count
 * @property {_1m1w1y24hPattern<StoredF32>} perSec
 * @property {SeriesTree_Inputs_ByType} byType
 */

/**
 * @typedef {Object} SeriesTree_Inputs_Raw
 * @property {SeriesPattern18<TxInIndex>} firstTxinIndex
 * @property {SeriesPattern20<OutPoint>} outpoint
 * @property {SeriesPattern20<TxIndex>} txIndex
 * @property {SeriesPattern20<OutputType>} outputType
 * @property {SeriesPattern20<TypeIndex>} typeIndex
 */

/**
 * @typedef {Object} SeriesTree_Inputs_Spent
 * @property {SeriesPattern20<TxOutIndex>} txoutIndex
 * @property {SeriesPattern20<Sats>} value
 */

/**
 * @typedef {Object} SeriesTree_Inputs_ByType
 * @property {SeriesTree_Inputs_ByType_InputCount} inputCount
 * @property {SeriesTree_Inputs_ByType_InputShare} inputShare
 * @property {SeriesTree_Inputs_ByType_TxCount} txCount
 * @property {EmptyP2aP2msP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshUnknownPattern2} txShare
 */

/**
 * @typedef {Object} SeriesTree_Inputs_ByType_InputCount
 * @property {AverageBlockCumulativeSumPattern<StoredU64>} all
 * @property {AverageBlockCumulativeSumPattern<StoredU64>} p2pk65
 * @property {AverageBlockCumulativeSumPattern<StoredU64>} p2pk33
 * @property {AverageBlockCumulativeSumPattern<StoredU64>} p2pkh
 * @property {AverageBlockCumulativeSumPattern<StoredU64>} p2ms
 * @property {AverageBlockCumulativeSumPattern<StoredU64>} p2sh
 * @property {AverageBlockCumulativeSumPattern<StoredU64>} p2wpkh
 * @property {AverageBlockCumulativeSumPattern<StoredU64>} p2wsh
 * @property {AverageBlockCumulativeSumPattern<StoredU64>} p2tr
 * @property {AverageBlockCumulativeSumPattern<StoredU64>} p2a
 * @property {AverageBlockCumulativeSumPattern<StoredU64>} unknown
 * @property {AverageBlockCumulativeSumPattern<StoredU64>} empty
 */

/**
 * @typedef {Object} SeriesTree_Inputs_ByType_InputShare
 * @property {_1m1w1y24hBpsPercentRatioPattern} p2pk65
 * @property {_1m1w1y24hBpsPercentRatioPattern} p2pk33
 * @property {_1m1w1y24hBpsPercentRatioPattern} p2pkh
 * @property {_1m1w1y24hBpsPercentRatioPattern} p2ms
 * @property {_1m1w1y24hBpsPercentRatioPattern} p2sh
 * @property {_1m1w1y24hBpsPercentRatioPattern} p2wpkh
 * @property {_1m1w1y24hBpsPercentRatioPattern} p2wsh
 * @property {_1m1w1y24hBpsPercentRatioPattern} p2tr
 * @property {_1m1w1y24hBpsPercentRatioPattern} p2a
 * @property {_1m1w1y24hBpsPercentRatioPattern} unknown
 * @property {_1m1w1y24hBpsPercentRatioPattern} empty
 */

/**
 * @typedef {Object} SeriesTree_Inputs_ByType_TxCount
 * @property {AverageBlockCumulativeSumPattern<StoredU64>} all
 * @property {AverageBlockCumulativeSumPattern<StoredU64>} p2pk65
 * @property {AverageBlockCumulativeSumPattern<StoredU64>} p2pk33
 * @property {AverageBlockCumulativeSumPattern<StoredU64>} p2pkh
 * @property {AverageBlockCumulativeSumPattern<StoredU64>} p2ms
 * @property {AverageBlockCumulativeSumPattern<StoredU64>} p2sh
 * @property {AverageBlockCumulativeSumPattern<StoredU64>} p2wpkh
 * @property {AverageBlockCumulativeSumPattern<StoredU64>} p2wsh
 * @property {AverageBlockCumulativeSumPattern<StoredU64>} p2tr
 * @property {AverageBlockCumulativeSumPattern<StoredU64>} p2a
 * @property {AverageBlockCumulativeSumPattern<StoredU64>} unknown
 * @property {AverageBlockCumulativeSumPattern<StoredU64>} empty
 */

/**
 * @typedef {Object} SeriesTree_Outputs
 * @property {SeriesTree_Outputs_Raw} raw
 * @property {SeriesTree_Outputs_Spent} spent
 * @property {SeriesTree_Outputs_Count} count
 * @property {_1m1w1y24hPattern<StoredF32>} perSec
 * @property {SeriesTree_Outputs_Unspent} unspent
 * @property {SeriesTree_Outputs_ByType} byType
 * @property {SeriesTree_Outputs_Value} value
 */

/**
 * @typedef {Object} SeriesTree_Outputs_Raw
 * @property {SeriesPattern18<TxOutIndex>} firstTxoutIndex
 * @property {SeriesPattern21<Sats>} value
 * @property {SeriesPattern21<OutputType>} outputType
 * @property {SeriesPattern21<TypeIndex>} typeIndex
 * @property {SeriesPattern21<TxIndex>} txIndex
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
 * @typedef {Object} SeriesTree_Outputs_Unspent
 * @property {SeriesPattern1<StoredU64>} count
 */

/**
 * @typedef {Object} SeriesTree_Outputs_ByType
 * @property {SeriesTree_Outputs_ByType_OutputCount} outputCount
 * @property {AverageBlockCumulativeSumPattern<StoredU64>} spendableOutputCount
 * @property {SeriesTree_Outputs_ByType_OutputShare} outputShare
 * @property {AllEmptyOpP2aP2msP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshUnknownPattern} txCount
 * @property {EmptyOpP2aP2msP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshUnknownPattern2} txShare
 */

/**
 * @typedef {Object} SeriesTree_Outputs_ByType_OutputCount
 * @property {AverageBlockCumulativeSumPattern<StoredU64>} all
 * @property {AverageBlockCumulativeSumPattern<StoredU64>} p2pk65
 * @property {AverageBlockCumulativeSumPattern<StoredU64>} p2pk33
 * @property {AverageBlockCumulativeSumPattern<StoredU64>} p2pkh
 * @property {AverageBlockCumulativeSumPattern<StoredU64>} p2ms
 * @property {AverageBlockCumulativeSumPattern<StoredU64>} p2sh
 * @property {AverageBlockCumulativeSumPattern<StoredU64>} p2wpkh
 * @property {AverageBlockCumulativeSumPattern<StoredU64>} p2wsh
 * @property {AverageBlockCumulativeSumPattern<StoredU64>} p2tr
 * @property {AverageBlockCumulativeSumPattern<StoredU64>} p2a
 * @property {AverageBlockCumulativeSumPattern<StoredU64>} unknown
 * @property {AverageBlockCumulativeSumPattern<StoredU64>} empty
 * @property {AverageBlockCumulativeSumPattern<StoredU64>} opReturn
 */

/**
 * @typedef {Object} SeriesTree_Outputs_ByType_OutputShare
 * @property {_1m1w1y24hBpsPercentRatioPattern} p2pk65
 * @property {_1m1w1y24hBpsPercentRatioPattern} p2pk33
 * @property {_1m1w1y24hBpsPercentRatioPattern} p2pkh
 * @property {_1m1w1y24hBpsPercentRatioPattern} p2ms
 * @property {_1m1w1y24hBpsPercentRatioPattern} p2sh
 * @property {_1m1w1y24hBpsPercentRatioPattern} p2wpkh
 * @property {_1m1w1y24hBpsPercentRatioPattern} p2wsh
 * @property {_1m1w1y24hBpsPercentRatioPattern} p2tr
 * @property {_1m1w1y24hBpsPercentRatioPattern} p2a
 * @property {_1m1w1y24hBpsPercentRatioPattern} unknown
 * @property {_1m1w1y24hBpsPercentRatioPattern} empty
 * @property {_1m1w1y24hBpsPercentRatioPattern} opReturn
 */

/**
 * @typedef {Object} SeriesTree_Outputs_Value
 * @property {BlockCumulativePattern} opReturn
 */

/**
 * @typedef {Object} SeriesTree_Addrs
 * @property {SeriesTree_Addrs_Raw} raw
 * @property {SeriesTree_Addrs_Indexes} indexes
 * @property {SeriesTree_Addrs_Data} data
 * @property {AllP2aP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshPattern4} funded
 * @property {AllP2aP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshPattern4} empty
 * @property {SeriesTree_Addrs_Activity} activity
 * @property {AllP2aP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshPattern4} total
 * @property {AllP2aP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshPattern6} new
 * @property {SeriesTree_Addrs_Reused} reused
 * @property {SeriesTree_Addrs_Respent} respent
 * @property {SeriesTree_Addrs_Exposed} exposed
 * @property {SeriesTree_Addrs_Delta} delta
 * @property {SeriesTree_Addrs_AvgAmount} avgAmount
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
 * @typedef {Object} SeriesTree_Addrs_Indexes
 * @property {SeriesPattern24<AnyAddrIndex>} p2a
 * @property {SeriesPattern26<AnyAddrIndex>} p2pk33
 * @property {SeriesPattern27<AnyAddrIndex>} p2pk65
 * @property {SeriesPattern28<AnyAddrIndex>} p2pkh
 * @property {SeriesPattern29<AnyAddrIndex>} p2sh
 * @property {SeriesPattern30<AnyAddrIndex>} p2tr
 * @property {SeriesPattern31<AnyAddrIndex>} p2wpkh
 * @property {SeriesPattern32<AnyAddrIndex>} p2wsh
 * @property {SeriesPattern34<FundedAddrIndex>} funded
 * @property {SeriesPattern35<EmptyAddrIndex>} empty
 */

/**
 * @typedef {Object} SeriesTree_Addrs_Data
 * @property {SeriesPattern34<FundedAddrData>} funded
 * @property {SeriesPattern35<EmptyAddrData>} empty
 */

/**
 * @typedef {Object} SeriesTree_Addrs_Activity
 * @property {SeriesTree_Addrs_Activity_All} all
 * @property {ActiveBidirectionalReactivatedReceivingSendingPattern} p2pk65
 * @property {ActiveBidirectionalReactivatedReceivingSendingPattern} p2pk33
 * @property {ActiveBidirectionalReactivatedReceivingSendingPattern} p2pkh
 * @property {ActiveBidirectionalReactivatedReceivingSendingPattern} p2sh
 * @property {ActiveBidirectionalReactivatedReceivingSendingPattern} p2wpkh
 * @property {ActiveBidirectionalReactivatedReceivingSendingPattern} p2wsh
 * @property {ActiveBidirectionalReactivatedReceivingSendingPattern} p2tr
 * @property {ActiveBidirectionalReactivatedReceivingSendingPattern} p2a
 */

/**
 * @typedef {Object} SeriesTree_Addrs_Activity_All
 * @property {_1m1w1y24hBlockPattern} reactivated
 * @property {_1m1w1y24hBlockPattern} sending
 * @property {_1m1w1y24hBlockPattern} receiving
 * @property {_1m1w1y24hBlockPattern} bidirectional
 * @property {_1m1w1y24hBlockPattern} active
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
 * @property {_1m1w1y24hBpsPercentRatioPattern} spendableOutputToReusedAddrShare
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
 * @property {_1m1w1y24hBpsPercentRatioPattern} spendableOutputToReusedAddrShare
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
 * @property {AbsoluteRatePattern} all
 * @property {AbsoluteRatePattern} p2pk65
 * @property {AbsoluteRatePattern} p2pk33
 * @property {AbsoluteRatePattern} p2pkh
 * @property {AbsoluteRatePattern} p2sh
 * @property {AbsoluteRatePattern} p2wpkh
 * @property {AbsoluteRatePattern} p2wsh
 * @property {AbsoluteRatePattern} p2tr
 * @property {AbsoluteRatePattern} p2a
 */

/**
 * @typedef {Object} SeriesTree_Addrs_AvgAmount
 * @property {AddrUtxoPattern} all
 * @property {AddrUtxoPattern} p2pk65
 * @property {AddrUtxoPattern} p2pk33
 * @property {AddrUtxoPattern} p2pkh
 * @property {AddrUtxoPattern} p2sh
 * @property {AddrUtxoPattern} p2wpkh
 * @property {AddrUtxoPattern} p2wsh
 * @property {AddrUtxoPattern} p2tr
 * @property {AddrUtxoPattern} p2a
 */

/**
 * @typedef {Object} SeriesTree_Scripts
 * @property {SeriesTree_Scripts_Raw} raw
 */

/**
 * @typedef {Object} SeriesTree_Scripts_Raw
 * @property {SeriesTree_Scripts_Raw_Empty} empty
 * @property {SeriesTree_Scripts_Raw_OpReturn} opReturn
 * @property {SeriesTree_Scripts_Raw_P2ms} p2ms
 * @property {SeriesTree_Scripts_Raw_Unknown} unknown
 */

/**
 * @typedef {Object} SeriesTree_Scripts_Raw_Empty
 * @property {SeriesPattern18<EmptyOutputIndex>} firstIndex
 * @property {SeriesPattern22<TxIndex>} toTxIndex
 */

/**
 * @typedef {Object} SeriesTree_Scripts_Raw_OpReturn
 * @property {SeriesPattern18<OpReturnIndex>} firstIndex
 * @property {SeriesPattern23<TxIndex>} toTxIndex
 */

/**
 * @typedef {Object} SeriesTree_Scripts_Raw_P2ms
 * @property {SeriesPattern18<P2MSOutputIndex>} firstIndex
 * @property {SeriesPattern25<TxIndex>} toTxIndex
 */

/**
 * @typedef {Object} SeriesTree_Scripts_Raw_Unknown
 * @property {SeriesPattern18<UnknownOutputIndex>} firstIndex
 * @property {SeriesPattern33<TxIndex>} toTxIndex
 */

/**
 * @typedef {Object} SeriesTree_Mining
 * @property {SeriesTree_Mining_Rewards} rewards
 * @property {SeriesTree_Mining_Hashrate} hashrate
 */

/**
 * @typedef {Object} SeriesTree_Mining_Rewards
 * @property {AverageBlockCumulativeSumPattern3} coinbase
 * @property {SeriesTree_Mining_Rewards_Subsidy} subsidy
 * @property {SeriesTree_Mining_Rewards_Fees} fees
 * @property {SeriesPattern18<Sats>} outputVolume
 * @property {BlockCumulativePattern} unclaimed
 */

/**
 * @typedef {Object} SeriesTree_Mining_Rewards_Subsidy
 * @property {BtcCentsSatsUsdPattern3} block
 * @property {BtcCentsSatsUsdPattern} cumulative
 * @property {_1m1w1y24hPattern4} sum
 * @property {_1m1w1y24hPattern3} average
 * @property {_1m1w1y24hBpsPercentRatioPattern} dominance
 */

/**
 * @typedef {Object} SeriesTree_Mining_Rewards_Fees
 * @property {BtcCentsSatsUsdPattern3} block
 * @property {BtcCentsSatsUsdPattern} cumulative
 * @property {_1m1w1y24hPattern4} sum
 * @property {_1m1w1y24hPattern3} average
 * @property {_1m1w1y24hPattern4} min
 * @property {_1m1w1y24hPattern4} max
 * @property {_1m1w1y24hPattern4} pct10
 * @property {_1m1w1y24hPattern4} pct25
 * @property {_1m1w1y24hPattern4} median
 * @property {_1m1w1y24hPattern4} pct75
 * @property {_1m1w1y24hPattern4} pct90
 * @property {_1m1w1y24hBpsPercentRatioPattern} dominance
 * @property {SeriesTree_Mining_Rewards_Fees_ToSubsidyRatio} toSubsidyRatio
 */

/**
 * @typedef {Object} SeriesTree_Mining_Rewards_Fees_ToSubsidyRatio
 * @property {BpsRatioPattern2} _24h
 * @property {BpsRatioPattern2} _1w
 * @property {BpsRatioPattern2} _1m
 * @property {BpsRatioPattern2} _1y
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
 * @property {BpsPercentRatioPattern5} drawdown
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
 * @property {SeriesTree_Cointime_Supply} supply
 * @property {SeriesTree_Cointime_Value} value
 * @property {SeriesTree_Cointime_Cap} cap
 * @property {SeriesTree_Cointime_Prices} prices
 * @property {SeriesTree_Cointime_Adjusted} adjusted
 * @property {SeriesTree_Cointime_ReserveRisk} reserveRisk
 */

/**
 * @typedef {Object} SeriesTree_Cointime_Activity
 * @property {AverageBlockCumulativeSumPattern<StoredF64>} coinblocksCreated
 * @property {AverageBlockCumulativeSumPattern<StoredF64>} coinblocksStored
 * @property {SeriesPattern1<StoredF64>} liveliness
 * @property {SeriesPattern1<StoredF64>} vaultedness
 * @property {SeriesPattern1<StoredF64>} ratio
 * @property {AverageBlockCumulativeSumPattern<StoredF64>} coinblocksDestroyed
 */

/**
 * @typedef {Object} SeriesTree_Cointime_Supply
 * @property {BtcCentsSatsUsdPattern} vaulted
 * @property {BtcCentsSatsUsdPattern} active
 */

/**
 * @typedef {Object} SeriesTree_Cointime_Value
 * @property {AverageBlockCumulativeSumPattern<StoredF64>} destroyed
 * @property {AverageBlockCumulativeSumPattern<StoredF64>} created
 * @property {AverageBlockCumulativeSumPattern<StoredF64>} stored
 * @property {AverageBlockCumulativeSumPattern<StoredF64>} vocdd
 */

/**
 * @typedef {Object} SeriesTree_Cointime_Cap
 * @property {CentsUsdPattern3} thermo
 * @property {CentsUsdPattern3} investor
 * @property {CentsUsdPattern3} vaulted
 * @property {CentsUsdPattern3} active
 * @property {CentsUsdPattern3} cointime
 * @property {BpsRatioPattern2} aviv
 */

/**
 * @typedef {Object} SeriesTree_Cointime_Prices
 * @property {BpsCentsPercentilesRatioSatsUsdPattern} vaulted
 * @property {BpsCentsPercentilesRatioSatsUsdPattern} active
 * @property {BpsCentsPercentilesRatioSatsUsdPattern} trueMarketMean
 * @property {BpsCentsPercentilesRatioSatsUsdPattern} cointime
 */

/**
 * @typedef {Object} SeriesTree_Cointime_Adjusted
 * @property {BpsPercentRatioPattern} inflationRate
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
 * @typedef {Object} SeriesTree_Indexes
 * @property {SeriesTree_Indexes_Addr} addr
 * @property {SeriesTree_Indexes_Height} height
 * @property {SeriesTree_Indexes_Epoch} epoch
 * @property {SeriesTree_Indexes_Halving} halving
 * @property {SeriesTree_Indexes_Minute10} minute10
 * @property {SeriesTree_Indexes_Minute30} minute30
 * @property {SeriesTree_Indexes_Hour1} hour1
 * @property {SeriesTree_Indexes_Hour4} hour4
 * @property {SeriesTree_Indexes_Hour12} hour12
 * @property {SeriesTree_Indexes_Day1} day1
 * @property {SeriesTree_Indexes_Day3} day3
 * @property {SeriesTree_Indexes_Week1} week1
 * @property {SeriesTree_Indexes_Month1} month1
 * @property {SeriesTree_Indexes_Month3} month3
 * @property {SeriesTree_Indexes_Month6} month6
 * @property {SeriesTree_Indexes_Year1} year1
 * @property {SeriesTree_Indexes_Year10} year10
 * @property {SeriesTree_Indexes_TxIndex} txIndex
 * @property {SeriesTree_Indexes_TxinIndex} txinIndex
 * @property {SeriesTree_Indexes_TxoutIndex} txoutIndex
 * @property {SeriesTree_Indexes_Timestamp} timestamp
 */

/**
 * @typedef {Object} SeriesTree_Indexes_Addr
 * @property {SeriesTree_Indexes_Addr_P2pk33} p2pk33
 * @property {SeriesTree_Indexes_Addr_P2pk65} p2pk65
 * @property {SeriesTree_Indexes_Addr_P2pkh} p2pkh
 * @property {SeriesTree_Indexes_Addr_P2sh} p2sh
 * @property {SeriesTree_Indexes_Addr_P2tr} p2tr
 * @property {SeriesTree_Indexes_Addr_P2wpkh} p2wpkh
 * @property {SeriesTree_Indexes_Addr_P2wsh} p2wsh
 * @property {SeriesTree_Indexes_Addr_P2a} p2a
 * @property {SeriesTree_Indexes_Addr_P2ms} p2ms
 * @property {SeriesTree_Indexes_Addr_Empty} empty
 * @property {SeriesTree_Indexes_Addr_Unknown} unknown
 * @property {SeriesTree_Indexes_Addr_OpReturn} opReturn
 */

/**
 * @typedef {Object} SeriesTree_Indexes_Addr_P2pk33
 * @property {SeriesPattern26<P2PK33AddrIndex>} identity
 * @property {SeriesPattern26<Addr>} addr
 */

/**
 * @typedef {Object} SeriesTree_Indexes_Addr_P2pk65
 * @property {SeriesPattern27<P2PK65AddrIndex>} identity
 * @property {SeriesPattern27<Addr>} addr
 */

/**
 * @typedef {Object} SeriesTree_Indexes_Addr_P2pkh
 * @property {SeriesPattern28<P2PKHAddrIndex>} identity
 * @property {SeriesPattern28<Addr>} addr
 */

/**
 * @typedef {Object} SeriesTree_Indexes_Addr_P2sh
 * @property {SeriesPattern29<P2SHAddrIndex>} identity
 * @property {SeriesPattern29<Addr>} addr
 */

/**
 * @typedef {Object} SeriesTree_Indexes_Addr_P2tr
 * @property {SeriesPattern30<P2TRAddrIndex>} identity
 * @property {SeriesPattern30<Addr>} addr
 */

/**
 * @typedef {Object} SeriesTree_Indexes_Addr_P2wpkh
 * @property {SeriesPattern31<P2WPKHAddrIndex>} identity
 * @property {SeriesPattern31<Addr>} addr
 */

/**
 * @typedef {Object} SeriesTree_Indexes_Addr_P2wsh
 * @property {SeriesPattern32<P2WSHAddrIndex>} identity
 * @property {SeriesPattern32<Addr>} addr
 */

/**
 * @typedef {Object} SeriesTree_Indexes_Addr_P2a
 * @property {SeriesPattern24<P2AAddrIndex>} identity
 * @property {SeriesPattern24<Addr>} addr
 */

/**
 * @typedef {Object} SeriesTree_Indexes_Addr_P2ms
 * @property {SeriesPattern25<P2MSOutputIndex>} identity
 */

/**
 * @typedef {Object} SeriesTree_Indexes_Addr_Empty
 * @property {SeriesPattern22<EmptyOutputIndex>} identity
 */

/**
 * @typedef {Object} SeriesTree_Indexes_Addr_Unknown
 * @property {SeriesPattern33<UnknownOutputIndex>} identity
 */

/**
 * @typedef {Object} SeriesTree_Indexes_Addr_OpReturn
 * @property {SeriesPattern23<OpReturnIndex>} identity
 */

/**
 * @typedef {Object} SeriesTree_Indexes_Height
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
 * @typedef {Object} SeriesTree_Indexes_Epoch
 * @property {SeriesPattern17<Height>} firstHeight
 */

/**
 * @typedef {Object} SeriesTree_Indexes_Halving
 * @property {SeriesPattern16<Height>} firstHeight
 */

/**
 * @typedef {Object} SeriesTree_Indexes_Minute10
 * @property {SeriesPattern3<Height>} firstHeight
 */

/**
 * @typedef {Object} SeriesTree_Indexes_Minute30
 * @property {SeriesPattern4<Height>} firstHeight
 */

/**
 * @typedef {Object} SeriesTree_Indexes_Hour1
 * @property {SeriesPattern5<Height>} firstHeight
 */

/**
 * @typedef {Object} SeriesTree_Indexes_Hour4
 * @property {SeriesPattern6<Height>} firstHeight
 */

/**
 * @typedef {Object} SeriesTree_Indexes_Hour12
 * @property {SeriesPattern7<Height>} firstHeight
 */

/**
 * @typedef {Object} SeriesTree_Indexes_Day1
 * @property {SeriesPattern8<Date>} date
 * @property {SeriesPattern8<Height>} firstHeight
 */

/**
 * @typedef {Object} SeriesTree_Indexes_Day3
 * @property {SeriesPattern9<Date>} date
 * @property {SeriesPattern9<Height>} firstHeight
 */

/**
 * @typedef {Object} SeriesTree_Indexes_Week1
 * @property {SeriesPattern10<Date>} date
 * @property {SeriesPattern10<Height>} firstHeight
 */

/**
 * @typedef {Object} SeriesTree_Indexes_Month1
 * @property {SeriesPattern11<Date>} date
 * @property {SeriesPattern11<Height>} firstHeight
 */

/**
 * @typedef {Object} SeriesTree_Indexes_Month3
 * @property {SeriesPattern12<Date>} date
 * @property {SeriesPattern12<Height>} firstHeight
 */

/**
 * @typedef {Object} SeriesTree_Indexes_Month6
 * @property {SeriesPattern13<Date>} date
 * @property {SeriesPattern13<Height>} firstHeight
 */

/**
 * @typedef {Object} SeriesTree_Indexes_Year1
 * @property {SeriesPattern14<Date>} date
 * @property {SeriesPattern14<Height>} firstHeight
 */

/**
 * @typedef {Object} SeriesTree_Indexes_Year10
 * @property {SeriesPattern15<Date>} date
 * @property {SeriesPattern15<Height>} firstHeight
 */

/**
 * @typedef {Object} SeriesTree_Indexes_TxIndex
 * @property {SeriesPattern19<TxIndex>} identity
 * @property {SeriesPattern19<StoredU64>} inputCount
 * @property {SeriesPattern19<StoredU64>} outputCount
 */

/**
 * @typedef {Object} SeriesTree_Indexes_TxinIndex
 * @property {SeriesPattern20<TxInIndex>} identity
 */

/**
 * @typedef {Object} SeriesTree_Indexes_TxoutIndex
 * @property {SeriesPattern21<TxOutIndex>} identity
 */

/**
 * @typedef {Object} SeriesTree_Indexes_Timestamp
 * @property {SeriesPattern18<Timestamp>} monotonic
 * @property {SeriesPattern2<Timestamp>} resolutions
 */

/**
 * @typedef {Object} SeriesTree_Indicators
 * @property {BpsRatioPattern2} puellMultiple
 * @property {BpsRatioPattern2} nvt
 * @property {BpsPercentRatioPattern2} gini
 * @property {BpsRatioPattern2} rhodlRatio
 * @property {BpsRatioPattern2} thermoCapMultiple
 * @property {SeriesPattern1<StoredF32>} coindaysDestroyedSupplyAdj
 * @property {SeriesPattern1<StoredF32>} coinyearsDestroyedSupplyAdj
 * @property {SeriesTree_Indicators_Dormancy} dormancy
 * @property {SeriesPattern1<StoredF32>} stockToFlow
 * @property {SeriesPattern1<StoredF32>} sellerExhaustion
 * @property {SeriesTree_Indicators_RarityMeter} rarityMeter
 */

/**
 * @typedef {Object} SeriesTree_Indicators_Dormancy
 * @property {SeriesPattern1<StoredF32>} supplyAdj
 * @property {SeriesPattern1<StoredF32>} flow
 */

/**
 * @typedef {Object} SeriesTree_Indicators_RarityMeter
 * @property {IndexPct0Pct1Pct2Pct5Pct95Pct98Pct99ScorePattern} full
 * @property {IndexPct0Pct1Pct2Pct5Pct95Pct98Pct99ScorePattern} local
 * @property {IndexPct0Pct1Pct2Pct5Pct95Pct98Pct99ScorePattern} cycle
 */

/**
 * @typedef {Object} SeriesTree_Investing
 * @property {SeriesPattern18<Sats>} satsPerDay
 * @property {SeriesTree_Investing_Period} period
 * @property {SeriesTree_Investing_Class} class
 */

/**
 * @typedef {Object} SeriesTree_Investing_Period
 * @property {_10y1m1w1y2y3m3y4y5y6m6y8yPattern3} dcaStack
 * @property {SeriesTree_Investing_Period_DcaCostBasis} dcaCostBasis
 * @property {_10y1m1w1y2y3m3y4y5y6m6y8yPattern2} dcaReturn
 * @property {_10y2y3y4y5y6y8yPattern} dcaCagr
 * @property {_10y1m1w1y2y3m3y4y5y6m6y8yPattern3} lumpSumStack
 * @property {_10y1m1w1y2y3m3y4y5y6m6y8yPattern2} lumpSumReturn
 */

/**
 * @typedef {Object} SeriesTree_Investing_Period_DcaCostBasis
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
 * @typedef {Object} SeriesTree_Investing_Class
 * @property {SeriesTree_Investing_Class_DcaStack} dcaStack
 * @property {SeriesTree_Investing_Class_DcaCostBasis} dcaCostBasis
 * @property {SeriesTree_Investing_Class_DcaReturn} dcaReturn
 */

/**
 * @typedef {Object} SeriesTree_Investing_Class_DcaStack
 * @property {BtcCentsSatsUsdPattern} from2015
 * @property {BtcCentsSatsUsdPattern} from2016
 * @property {BtcCentsSatsUsdPattern} from2017
 * @property {BtcCentsSatsUsdPattern} from2018
 * @property {BtcCentsSatsUsdPattern} from2019
 * @property {BtcCentsSatsUsdPattern} from2020
 * @property {BtcCentsSatsUsdPattern} from2021
 * @property {BtcCentsSatsUsdPattern} from2022
 * @property {BtcCentsSatsUsdPattern} from2023
 * @property {BtcCentsSatsUsdPattern} from2024
 * @property {BtcCentsSatsUsdPattern} from2025
 * @property {BtcCentsSatsUsdPattern} from2026
 */

/**
 * @typedef {Object} SeriesTree_Investing_Class_DcaCostBasis
 * @property {CentsSatsUsdPattern} from2015
 * @property {CentsSatsUsdPattern} from2016
 * @property {CentsSatsUsdPattern} from2017
 * @property {CentsSatsUsdPattern} from2018
 * @property {CentsSatsUsdPattern} from2019
 * @property {CentsSatsUsdPattern} from2020
 * @property {CentsSatsUsdPattern} from2021
 * @property {CentsSatsUsdPattern} from2022
 * @property {CentsSatsUsdPattern} from2023
 * @property {CentsSatsUsdPattern} from2024
 * @property {CentsSatsUsdPattern} from2025
 * @property {CentsSatsUsdPattern} from2026
 */

/**
 * @typedef {Object} SeriesTree_Investing_Class_DcaReturn
 * @property {BpsPercentRatioPattern} from2015
 * @property {BpsPercentRatioPattern} from2016
 * @property {BpsPercentRatioPattern} from2017
 * @property {BpsPercentRatioPattern} from2018
 * @property {BpsPercentRatioPattern} from2019
 * @property {BpsPercentRatioPattern} from2020
 * @property {BpsPercentRatioPattern} from2021
 * @property {BpsPercentRatioPattern} from2022
 * @property {BpsPercentRatioPattern} from2023
 * @property {BpsPercentRatioPattern} from2024
 * @property {BpsPercentRatioPattern} from2025
 * @property {BpsPercentRatioPattern} from2026
 */

/**
 * @typedef {Object} SeriesTree_Market
 * @property {SeriesTree_Market_Ath} ath
 * @property {SeriesTree_Market_Lookback} lookback
 * @property {SeriesTree_Market_Returns} returns
 * @property {_1m1w1y24hPattern<StoredF32>} volatility
 * @property {SeriesTree_Market_Range} range
 * @property {SeriesTree_Market_MovingAverage} movingAverage
 * @property {SeriesTree_Market_Technical} technical
 */

/**
 * @typedef {Object} SeriesTree_Market_Ath
 * @property {CentsSatsUsdPattern} high
 * @property {BpsPercentRatioPattern5} drawdown
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
 * @property {_10y2y3y4y5y6y8yPattern} cagr
 * @property {SeriesTree_Market_Returns_Sd24h} sd24h
 */

/**
 * @typedef {Object} SeriesTree_Market_Returns_Periods
 * @property {BpsPercentRatioPattern} _24h
 * @property {BpsPercentRatioPattern} _1w
 * @property {BpsPercentRatioPattern} _1m
 * @property {BpsPercentRatioPattern} _3m
 * @property {BpsPercentRatioPattern} _6m
 * @property {BpsPercentRatioPattern} _1y
 * @property {BpsPercentRatioPattern} _2y
 * @property {BpsPercentRatioPattern} _3y
 * @property {BpsPercentRatioPattern} _4y
 * @property {BpsPercentRatioPattern} _5y
 * @property {BpsPercentRatioPattern} _6y
 * @property {BpsPercentRatioPattern} _8y
 * @property {BpsPercentRatioPattern} _10y
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
 * @property {BpsPercentRatioPattern2} choppinessIndex2w
 */

/**
 * @typedef {Object} SeriesTree_Market_MovingAverage
 * @property {SeriesTree_Market_MovingAverage_Sma} sma
 * @property {SeriesTree_Market_MovingAverage_Ema} ema
 */

/**
 * @typedef {Object} SeriesTree_Market_MovingAverage_Sma
 * @property {BpsCentsRatioSatsUsdPattern} _1w
 * @property {BpsCentsRatioSatsUsdPattern} _8d
 * @property {BpsCentsRatioSatsUsdPattern} _13d
 * @property {BpsCentsRatioSatsUsdPattern} _21d
 * @property {BpsCentsRatioSatsUsdPattern} _1m
 * @property {BpsCentsRatioSatsUsdPattern} _34d
 * @property {BpsCentsRatioSatsUsdPattern} _55d
 * @property {BpsCentsRatioSatsUsdPattern} _89d
 * @property {BpsCentsRatioSatsUsdPattern} _111d
 * @property {BpsCentsRatioSatsUsdPattern} _144d
 * @property {SeriesTree_Market_MovingAverage_Sma_200d} _200d
 * @property {SeriesTree_Market_MovingAverage_Sma_350d} _350d
 * @property {BpsCentsRatioSatsUsdPattern} _1y
 * @property {BpsCentsRatioSatsUsdPattern} _2y
 * @property {BpsCentsRatioSatsUsdPattern} _200w
 * @property {BpsCentsRatioSatsUsdPattern} _4y
 */

/**
 * @typedef {Object} SeriesTree_Market_MovingAverage_Sma_200d
 * @property {SeriesPattern1<Dollars>} usd
 * @property {SeriesPattern1<Cents>} cents
 * @property {SeriesPattern1<SatsFract>} sats
 * @property {SeriesPattern1<BasisPoints32>} bps
 * @property {SeriesPattern1<StoredF32>} ratio
 * @property {CentsSatsUsdPattern} x24
 * @property {CentsSatsUsdPattern} x08
 */

/**
 * @typedef {Object} SeriesTree_Market_MovingAverage_Sma_350d
 * @property {SeriesPattern1<Dollars>} usd
 * @property {SeriesPattern1<Cents>} cents
 * @property {SeriesPattern1<SatsFract>} sats
 * @property {SeriesPattern1<BasisPoints32>} bps
 * @property {SeriesPattern1<StoredF32>} ratio
 * @property {CentsSatsUsdPattern} x2
 */

/**
 * @typedef {Object} SeriesTree_Market_MovingAverage_Ema
 * @property {BpsCentsRatioSatsUsdPattern} _1w
 * @property {BpsCentsRatioSatsUsdPattern} _8d
 * @property {BpsCentsRatioSatsUsdPattern} _12d
 * @property {BpsCentsRatioSatsUsdPattern} _13d
 * @property {BpsCentsRatioSatsUsdPattern} _21d
 * @property {BpsCentsRatioSatsUsdPattern} _26d
 * @property {BpsCentsRatioSatsUsdPattern} _1m
 * @property {BpsCentsRatioSatsUsdPattern} _34d
 * @property {BpsCentsRatioSatsUsdPattern} _55d
 * @property {BpsCentsRatioSatsUsdPattern} _89d
 * @property {BpsCentsRatioSatsUsdPattern} _144d
 * @property {BpsCentsRatioSatsUsdPattern} _200d
 * @property {BpsCentsRatioSatsUsdPattern} _1y
 * @property {BpsCentsRatioSatsUsdPattern} _2y
 * @property {BpsCentsRatioSatsUsdPattern} _200w
 * @property {BpsCentsRatioSatsUsdPattern} _4y
 */

/**
 * @typedef {Object} SeriesTree_Market_Technical
 * @property {SeriesTree_Market_Technical_Rsi} rsi
 * @property {BpsRatioPattern2} piCycle
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
 * @typedef {Object} SeriesTree_Supply
 * @property {SeriesPattern18<SupplyState>} state
 * @property {BtcCentsSatsUsdPattern} circulating
 * @property {BlockCumulativePattern} burned
 * @property {BpsPercentRatioPattern} inflationRate
 * @property {SeriesTree_Supply_Velocity} velocity
 * @property {CentsDeltaUsdPattern} marketCap
 * @property {_1m1w1y24hPattern<BasisPointsSigned32>} marketMinusRealizedCapGrowthRate
 * @property {BtcCentsSatsUsdPattern} hodledOrLost
 */

/**
 * @typedef {Object} SeriesTree_Supply_Velocity
 * @property {SeriesPattern1<StoredF64>} native
 * @property {SeriesPattern1<StoredF64>} fiat
 */

/**
 * @typedef {Object} SeriesTree_Cohorts
 * @property {SeriesTree_Cohorts_Utxo} utxo
 * @property {SeriesTree_Cohorts_Addr} addr
 */

/**
 * @typedef {Object} SeriesTree_Cohorts_Utxo
 * @property {SeriesTree_Cohorts_Utxo_All} all
 * @property {SeriesTree_Cohorts_Utxo_Sth} sth
 * @property {SeriesTree_Cohorts_Utxo_Lth} lth
 * @property {SeriesTree_Cohorts_Utxo_AgeRange} ageRange
 * @property {SeriesTree_Cohorts_Utxo_UnderAge} underAge
 * @property {SeriesTree_Cohorts_Utxo_OverAge} overAge
 * @property {SeriesTree_Cohorts_Utxo_Epoch} epoch
 * @property {SeriesTree_Cohorts_Utxo_Class} class
 * @property {SeriesTree_Cohorts_Utxo_Entry} entry
 * @property {SeriesTree_Cohorts_Utxo_OverAmount} overAmount
 * @property {SeriesTree_Cohorts_Utxo_AmountRange} amountRange
 * @property {SeriesTree_Cohorts_Utxo_UnderAmount} underAmount
 * @property {SeriesTree_Cohorts_Utxo_Type} type
 * @property {SeriesTree_Cohorts_Utxo_Profitability} profitability
 * @property {SeriesTree_Cohorts_Utxo_Matured} matured
 */

/**
 * @typedef {Object} SeriesTree_Cohorts_Utxo_All
 * @property {DeltaDominanceHalfInTotalPattern2} supply
 * @property {SeriesTree_Cohorts_Utxo_All_Outputs} outputs
 * @property {SeriesTree_Cohorts_Utxo_All_Activity} activity
 * @property {SeriesTree_Cohorts_Utxo_All_Realized} realized
 * @property {SeriesTree_Cohorts_Utxo_All_CostBasis} costBasis
 * @property {SeriesTree_Cohorts_Utxo_All_Unrealized} unrealized
 * @property {InPattern} investedCapital
 */

/**
 * @typedef {Object} SeriesTree_Cohorts_Utxo_All_Outputs
 * @property {BaseDeltaPattern} unspentCount
 * @property {AverageBlockCumulativeSumPattern2} spentCount
 * @property {SeriesPattern1<StoredF32>} spendingRate
 */

/**
 * @typedef {Object} SeriesTree_Cohorts_Utxo_All_Activity
 * @property {AverageBlockCumulativeInSumPattern} transferVolume
 * @property {AverageBlockCumulativeSumPattern<StoredF64>} coindaysDestroyed
 * @property {SeriesPattern1<StoredF64>} coinyearsDestroyed
 * @property {_1m1w1y24hPattern<StoredF32>} dormancy
 */

/**
 * @typedef {Object} SeriesTree_Cohorts_Utxo_All_Realized
 * @property {CentsDeltaToUsdPattern} cap
 * @property {BlockCumulativeSumPattern} profit
 * @property {BlockCumulativeNegativeSumPattern} loss
 * @property {SeriesTree_Cohorts_Utxo_All_Realized_Price} price
 * @property {SeriesPattern1<StoredF32>} mvrv
 * @property {BlockChangeCumulativeDeltaSumPattern} netPnl
 * @property {SeriesTree_Cohorts_Utxo_All_Realized_Sopr} sopr
 * @property {BlockCumulativeSumPattern} grossPnl
 * @property {_1m1w1y24hPattern8} sellSideRiskRatio
 * @property {BlockCumulativeSumPattern} peakRegret
 * @property {PricePattern} capitalized
 * @property {_1m1w1y24hPattern<StoredF64>} profitToLossRatio
 */

/**
 * @typedef {Object} SeriesTree_Cohorts_Utxo_All_Realized_Price
 * @property {SeriesPattern1<Dollars>} usd
 * @property {SeriesPattern1<Cents>} cents
 * @property {SeriesPattern1<SatsFract>} sats
 * @property {SeriesPattern1<BasisPoints32>} bps
 * @property {SeriesPattern1<StoredF32>} ratio
 * @property {Pct0Pct1Pct2Pct5Pct95Pct98Pct99Pattern} percentiles
 * @property {_1m1w1y2y4yAllPattern} sma
 * @property {SeriesTree_Cohorts_Utxo_All_Realized_Price_StdDev} stdDev
 */

/**
 * @typedef {Object} SeriesTree_Cohorts_Utxo_All_Realized_Price_StdDev
 * @property {SeriesTree_Cohorts_Utxo_All_Realized_Price_StdDev_All} all
 * @property {SeriesTree_Cohorts_Utxo_All_Realized_Price_StdDev_4y} _4y
 * @property {SeriesTree_Cohorts_Utxo_All_Realized_Price_StdDev_2y} _2y
 * @property {SeriesTree_Cohorts_Utxo_All_Realized_Price_StdDev_1y} _1y
 */

/**
 * @typedef {Object} SeriesTree_Cohorts_Utxo_All_Realized_Price_StdDev_All
 * @property {SeriesPattern1<StoredF32>} sd
 * @property {SeriesPattern1<StoredF32>} zscore
 * @property {CentsSatsUsdPattern} _0sd
 * @property {PriceRatioPattern} p05sd
 * @property {PriceRatioPattern} p1sd
 * @property {PriceRatioPattern} p15sd
 * @property {PriceRatioPattern} p2sd
 * @property {PriceRatioPattern} p25sd
 * @property {PriceRatioPattern} p3sd
 * @property {PriceRatioPattern} m05sd
 * @property {PriceRatioPattern} m1sd
 * @property {PriceRatioPattern} m15sd
 * @property {PriceRatioPattern} m2sd
 * @property {PriceRatioPattern} m25sd
 * @property {PriceRatioPattern} m3sd
 */

/**
 * @typedef {Object} SeriesTree_Cohorts_Utxo_All_Realized_Price_StdDev_4y
 * @property {SeriesPattern1<StoredF32>} sd
 * @property {SeriesPattern1<StoredF32>} zscore
 * @property {CentsSatsUsdPattern} _0sd
 * @property {PriceRatioPattern} p05sd
 * @property {PriceRatioPattern} p1sd
 * @property {PriceRatioPattern} p15sd
 * @property {PriceRatioPattern} p2sd
 * @property {PriceRatioPattern} p25sd
 * @property {PriceRatioPattern} p3sd
 * @property {PriceRatioPattern} m05sd
 * @property {PriceRatioPattern} m1sd
 * @property {PriceRatioPattern} m15sd
 * @property {PriceRatioPattern} m2sd
 * @property {PriceRatioPattern} m25sd
 * @property {PriceRatioPattern} m3sd
 */

/**
 * @typedef {Object} SeriesTree_Cohorts_Utxo_All_Realized_Price_StdDev_2y
 * @property {SeriesPattern1<StoredF32>} sd
 * @property {SeriesPattern1<StoredF32>} zscore
 * @property {CentsSatsUsdPattern} _0sd
 * @property {PriceRatioPattern} p05sd
 * @property {PriceRatioPattern} p1sd
 * @property {PriceRatioPattern} p15sd
 * @property {PriceRatioPattern} p2sd
 * @property {PriceRatioPattern} p25sd
 * @property {PriceRatioPattern} p3sd
 * @property {PriceRatioPattern} m05sd
 * @property {PriceRatioPattern} m1sd
 * @property {PriceRatioPattern} m15sd
 * @property {PriceRatioPattern} m2sd
 * @property {PriceRatioPattern} m25sd
 * @property {PriceRatioPattern} m3sd
 */

/**
 * @typedef {Object} SeriesTree_Cohorts_Utxo_All_Realized_Price_StdDev_1y
 * @property {SeriesPattern1<StoredF32>} sd
 * @property {SeriesPattern1<StoredF32>} zscore
 * @property {CentsSatsUsdPattern} _0sd
 * @property {PriceRatioPattern} p05sd
 * @property {PriceRatioPattern} p1sd
 * @property {PriceRatioPattern} p15sd
 * @property {PriceRatioPattern} p2sd
 * @property {PriceRatioPattern} p25sd
 * @property {PriceRatioPattern} p3sd
 * @property {PriceRatioPattern} m05sd
 * @property {PriceRatioPattern} m1sd
 * @property {PriceRatioPattern} m15sd
 * @property {PriceRatioPattern} m2sd
 * @property {PriceRatioPattern} m25sd
 * @property {PriceRatioPattern} m3sd
 */

/**
 * @typedef {Object} SeriesTree_Cohorts_Utxo_All_Realized_Sopr
 * @property {AverageBlockCumulativeSumPattern<Cents>} valueDestroyed
 * @property {_1m1w1y24hPattern<StoredF64>} ratio
 * @property {SeriesTree_Cohorts_Utxo_All_Realized_Sopr_Adjusted} adjusted
 */

/**
 * @typedef {Object} SeriesTree_Cohorts_Utxo_All_Realized_Sopr_Adjusted
 * @property {_1m1w1y24hPattern<StoredF64>} ratio
 * @property {AverageBlockCumulativeSumPattern<Cents>} transferVolume
 * @property {AverageBlockCumulativeSumPattern<Cents>} valueDestroyed
 */

/**
 * @typedef {Object} SeriesTree_Cohorts_Utxo_All_CostBasis
 * @property {PerPattern} inProfit
 * @property {PerPattern} inLoss
 * @property {CentsSatsUsdPattern} min
 * @property {CentsSatsUsdPattern} max
 * @property {Pct05Pct10Pct15Pct20Pct25Pct30Pct35Pct40Pct45Pct50Pct55Pct60Pct65Pct70Pct75Pct80Pct85Pct90Pct95Pattern} perCoin
 * @property {Pct05Pct10Pct15Pct20Pct25Pct30Pct35Pct40Pct45Pct50Pct55Pct60Pct65Pct70Pct75Pct80Pct85Pct90Pct95Pattern} perDollar
 * @property {BpsPercentRatioPattern2} supplyDensity
 */

/**
 * @typedef {Object} SeriesTree_Cohorts_Utxo_All_Unrealized
 * @property {BpsRatioPattern} nupl
 * @property {SeriesTree_Cohorts_Utxo_All_Unrealized_Profit} profit
 * @property {SeriesTree_Cohorts_Utxo_All_Unrealized_Loss} loss
 * @property {SeriesTree_Cohorts_Utxo_All_Unrealized_NetPnl} netPnl
 * @property {CentsUsdPattern3} grossPnl
 * @property {InPattern2} investedCapital
 * @property {SeriesPattern18<CentsSquaredSats>} capitalizedCapInProfitRaw
 * @property {SeriesPattern18<CentsSquaredSats>} capitalizedCapInLossRaw
 * @property {SeriesTree_Cohorts_Utxo_All_Unrealized_Sentiment} sentiment
 */

/**
 * @typedef {Object} SeriesTree_Cohorts_Utxo_All_Unrealized_Profit
 * @property {SeriesPattern1<Dollars>} usd
 * @property {SeriesPattern1<Cents>} cents
 * @property {BpsPercentRatioPattern2} toMcap
 * @property {BpsPercentRatioPattern2} toOwnGrossPnl
 */

/**
 * @typedef {Object} SeriesTree_Cohorts_Utxo_All_Unrealized_Loss
 * @property {SeriesPattern1<Dollars>} usd
 * @property {SeriesPattern1<Cents>} cents
 * @property {SeriesPattern1<Dollars>} negative
 * @property {BpsPercentRatioPattern2} toMcap
 * @property {BpsPercentRatioPattern2} toOwnGrossPnl
 */

/**
 * @typedef {Object} SeriesTree_Cohorts_Utxo_All_Unrealized_NetPnl
 * @property {SeriesPattern1<Dollars>} usd
 * @property {SeriesPattern1<CentsSigned>} cents
 * @property {BpsPercentRatioPattern} toOwnGrossPnl
 */

/**
 * @typedef {Object} SeriesTree_Cohorts_Utxo_All_Unrealized_Sentiment
 * @property {CentsUsdPattern3} painIndex
 * @property {CentsUsdPattern3} greedIndex
 * @property {CentsUsdPattern} net
 */

/**
 * @typedef {Object} SeriesTree_Cohorts_Utxo_Sth
 * @property {DeltaDominanceHalfInTotalPattern2} supply
 * @property {SpendingSpentUnspentPattern} outputs
 * @property {CoindaysCoinyearsDormancyTransferPattern} activity
 * @property {SeriesTree_Cohorts_Utxo_Sth_Realized} realized
 * @property {InMaxMinPerSupplyPattern} costBasis
 * @property {CapitalizedGrossInvestedLossNetNuplProfitSentimentPattern2} unrealized
 * @property {InPattern} investedCapital
 */

/**
 * @typedef {Object} SeriesTree_Cohorts_Utxo_Sth_Realized
 * @property {CentsDeltaToUsdPattern} cap
 * @property {BlockCumulativeSumPattern} profit
 * @property {BlockCumulativeNegativeSumPattern} loss
 * @property {SeriesTree_Cohorts_Utxo_Sth_Realized_Price} price
 * @property {SeriesPattern1<StoredF32>} mvrv
 * @property {BlockChangeCumulativeDeltaSumPattern} netPnl
 * @property {AdjustedRatioValuePattern} sopr
 * @property {BlockCumulativeSumPattern} grossPnl
 * @property {_1m1w1y24hPattern8} sellSideRiskRatio
 * @property {BlockCumulativeSumPattern} peakRegret
 * @property {PricePattern} capitalized
 * @property {_1m1w1y24hPattern<StoredF64>} profitToLossRatio
 */

/**
 * @typedef {Object} SeriesTree_Cohorts_Utxo_Sth_Realized_Price
 * @property {SeriesPattern1<Dollars>} usd
 * @property {SeriesPattern1<Cents>} cents
 * @property {SeriesPattern1<SatsFract>} sats
 * @property {SeriesPattern1<BasisPoints32>} bps
 * @property {SeriesPattern1<StoredF32>} ratio
 * @property {Pct0Pct1Pct2Pct5Pct95Pct98Pct99Pattern} percentiles
 * @property {_1m1w1y2y4yAllPattern} sma
 * @property {SeriesTree_Cohorts_Utxo_Sth_Realized_Price_StdDev} stdDev
 */

/**
 * @typedef {Object} SeriesTree_Cohorts_Utxo_Sth_Realized_Price_StdDev
 * @property {SeriesTree_Cohorts_Utxo_Sth_Realized_Price_StdDev_All} all
 * @property {SeriesTree_Cohorts_Utxo_Sth_Realized_Price_StdDev_4y} _4y
 * @property {SeriesTree_Cohorts_Utxo_Sth_Realized_Price_StdDev_2y} _2y
 * @property {SeriesTree_Cohorts_Utxo_Sth_Realized_Price_StdDev_1y} _1y
 */

/**
 * @typedef {Object} SeriesTree_Cohorts_Utxo_Sth_Realized_Price_StdDev_All
 * @property {SeriesPattern1<StoredF32>} sd
 * @property {SeriesPattern1<StoredF32>} zscore
 * @property {CentsSatsUsdPattern} _0sd
 * @property {PriceRatioPattern} p05sd
 * @property {PriceRatioPattern} p1sd
 * @property {PriceRatioPattern} p15sd
 * @property {PriceRatioPattern} p2sd
 * @property {PriceRatioPattern} p25sd
 * @property {PriceRatioPattern} p3sd
 * @property {PriceRatioPattern} m05sd
 * @property {PriceRatioPattern} m1sd
 * @property {PriceRatioPattern} m15sd
 * @property {PriceRatioPattern} m2sd
 * @property {PriceRatioPattern} m25sd
 * @property {PriceRatioPattern} m3sd
 */

/**
 * @typedef {Object} SeriesTree_Cohorts_Utxo_Sth_Realized_Price_StdDev_4y
 * @property {SeriesPattern1<StoredF32>} sd
 * @property {SeriesPattern1<StoredF32>} zscore
 * @property {CentsSatsUsdPattern} _0sd
 * @property {PriceRatioPattern} p05sd
 * @property {PriceRatioPattern} p1sd
 * @property {PriceRatioPattern} p15sd
 * @property {PriceRatioPattern} p2sd
 * @property {PriceRatioPattern} p25sd
 * @property {PriceRatioPattern} p3sd
 * @property {PriceRatioPattern} m05sd
 * @property {PriceRatioPattern} m1sd
 * @property {PriceRatioPattern} m15sd
 * @property {PriceRatioPattern} m2sd
 * @property {PriceRatioPattern} m25sd
 * @property {PriceRatioPattern} m3sd
 */

/**
 * @typedef {Object} SeriesTree_Cohorts_Utxo_Sth_Realized_Price_StdDev_2y
 * @property {SeriesPattern1<StoredF32>} sd
 * @property {SeriesPattern1<StoredF32>} zscore
 * @property {CentsSatsUsdPattern} _0sd
 * @property {PriceRatioPattern} p05sd
 * @property {PriceRatioPattern} p1sd
 * @property {PriceRatioPattern} p15sd
 * @property {PriceRatioPattern} p2sd
 * @property {PriceRatioPattern} p25sd
 * @property {PriceRatioPattern} p3sd
 * @property {PriceRatioPattern} m05sd
 * @property {PriceRatioPattern} m1sd
 * @property {PriceRatioPattern} m15sd
 * @property {PriceRatioPattern} m2sd
 * @property {PriceRatioPattern} m25sd
 * @property {PriceRatioPattern} m3sd
 */

/**
 * @typedef {Object} SeriesTree_Cohorts_Utxo_Sth_Realized_Price_StdDev_1y
 * @property {SeriesPattern1<StoredF32>} sd
 * @property {SeriesPattern1<StoredF32>} zscore
 * @property {CentsSatsUsdPattern} _0sd
 * @property {PriceRatioPattern} p05sd
 * @property {PriceRatioPattern} p1sd
 * @property {PriceRatioPattern} p15sd
 * @property {PriceRatioPattern} p2sd
 * @property {PriceRatioPattern} p25sd
 * @property {PriceRatioPattern} p3sd
 * @property {PriceRatioPattern} m05sd
 * @property {PriceRatioPattern} m1sd
 * @property {PriceRatioPattern} m15sd
 * @property {PriceRatioPattern} m2sd
 * @property {PriceRatioPattern} m25sd
 * @property {PriceRatioPattern} m3sd
 */

/**
 * @typedef {Object} SeriesTree_Cohorts_Utxo_Lth
 * @property {DeltaDominanceHalfInTotalPattern2} supply
 * @property {SpendingSpentUnspentPattern} outputs
 * @property {CoindaysCoinyearsDormancyTransferPattern} activity
 * @property {SeriesTree_Cohorts_Utxo_Lth_Realized} realized
 * @property {InMaxMinPerSupplyPattern} costBasis
 * @property {CapitalizedGrossInvestedLossNetNuplProfitSentimentPattern2} unrealized
 * @property {InPattern} investedCapital
 */

/**
 * @typedef {Object} SeriesTree_Cohorts_Utxo_Lth_Realized
 * @property {CentsDeltaToUsdPattern} cap
 * @property {BlockCumulativeSumPattern} profit
 * @property {BlockCumulativeNegativeSumPattern} loss
 * @property {SeriesTree_Cohorts_Utxo_Lth_Realized_Price} price
 * @property {SeriesPattern1<StoredF32>} mvrv
 * @property {BlockChangeCumulativeDeltaSumPattern} netPnl
 * @property {RatioValuePattern2} sopr
 * @property {BlockCumulativeSumPattern} grossPnl
 * @property {_1m1w1y24hPattern8} sellSideRiskRatio
 * @property {BlockCumulativeSumPattern} peakRegret
 * @property {PricePattern} capitalized
 * @property {_1m1w1y24hPattern<StoredF64>} profitToLossRatio
 */

/**
 * @typedef {Object} SeriesTree_Cohorts_Utxo_Lth_Realized_Price
 * @property {SeriesPattern1<Dollars>} usd
 * @property {SeriesPattern1<Cents>} cents
 * @property {SeriesPattern1<SatsFract>} sats
 * @property {SeriesPattern1<BasisPoints32>} bps
 * @property {SeriesPattern1<StoredF32>} ratio
 * @property {Pct0Pct1Pct2Pct5Pct95Pct98Pct99Pattern} percentiles
 * @property {_1m1w1y2y4yAllPattern} sma
 * @property {SeriesTree_Cohorts_Utxo_Lth_Realized_Price_StdDev} stdDev
 */

/**
 * @typedef {Object} SeriesTree_Cohorts_Utxo_Lth_Realized_Price_StdDev
 * @property {SeriesTree_Cohorts_Utxo_Lth_Realized_Price_StdDev_All} all
 * @property {SeriesTree_Cohorts_Utxo_Lth_Realized_Price_StdDev_4y} _4y
 * @property {SeriesTree_Cohorts_Utxo_Lth_Realized_Price_StdDev_2y} _2y
 * @property {SeriesTree_Cohorts_Utxo_Lth_Realized_Price_StdDev_1y} _1y
 */

/**
 * @typedef {Object} SeriesTree_Cohorts_Utxo_Lth_Realized_Price_StdDev_All
 * @property {SeriesPattern1<StoredF32>} sd
 * @property {SeriesPattern1<StoredF32>} zscore
 * @property {CentsSatsUsdPattern} _0sd
 * @property {PriceRatioPattern} p05sd
 * @property {PriceRatioPattern} p1sd
 * @property {PriceRatioPattern} p15sd
 * @property {PriceRatioPattern} p2sd
 * @property {PriceRatioPattern} p25sd
 * @property {PriceRatioPattern} p3sd
 * @property {PriceRatioPattern} m05sd
 * @property {PriceRatioPattern} m1sd
 * @property {PriceRatioPattern} m15sd
 * @property {PriceRatioPattern} m2sd
 * @property {PriceRatioPattern} m25sd
 * @property {PriceRatioPattern} m3sd
 */

/**
 * @typedef {Object} SeriesTree_Cohorts_Utxo_Lth_Realized_Price_StdDev_4y
 * @property {SeriesPattern1<StoredF32>} sd
 * @property {SeriesPattern1<StoredF32>} zscore
 * @property {CentsSatsUsdPattern} _0sd
 * @property {PriceRatioPattern} p05sd
 * @property {PriceRatioPattern} p1sd
 * @property {PriceRatioPattern} p15sd
 * @property {PriceRatioPattern} p2sd
 * @property {PriceRatioPattern} p25sd
 * @property {PriceRatioPattern} p3sd
 * @property {PriceRatioPattern} m05sd
 * @property {PriceRatioPattern} m1sd
 * @property {PriceRatioPattern} m15sd
 * @property {PriceRatioPattern} m2sd
 * @property {PriceRatioPattern} m25sd
 * @property {PriceRatioPattern} m3sd
 */

/**
 * @typedef {Object} SeriesTree_Cohorts_Utxo_Lth_Realized_Price_StdDev_2y
 * @property {SeriesPattern1<StoredF32>} sd
 * @property {SeriesPattern1<StoredF32>} zscore
 * @property {CentsSatsUsdPattern} _0sd
 * @property {PriceRatioPattern} p05sd
 * @property {PriceRatioPattern} p1sd
 * @property {PriceRatioPattern} p15sd
 * @property {PriceRatioPattern} p2sd
 * @property {PriceRatioPattern} p25sd
 * @property {PriceRatioPattern} p3sd
 * @property {PriceRatioPattern} m05sd
 * @property {PriceRatioPattern} m1sd
 * @property {PriceRatioPattern} m15sd
 * @property {PriceRatioPattern} m2sd
 * @property {PriceRatioPattern} m25sd
 * @property {PriceRatioPattern} m3sd
 */

/**
 * @typedef {Object} SeriesTree_Cohorts_Utxo_Lth_Realized_Price_StdDev_1y
 * @property {SeriesPattern1<StoredF32>} sd
 * @property {SeriesPattern1<StoredF32>} zscore
 * @property {CentsSatsUsdPattern} _0sd
 * @property {PriceRatioPattern} p05sd
 * @property {PriceRatioPattern} p1sd
 * @property {PriceRatioPattern} p15sd
 * @property {PriceRatioPattern} p2sd
 * @property {PriceRatioPattern} p25sd
 * @property {PriceRatioPattern} p3sd
 * @property {PriceRatioPattern} m05sd
 * @property {PriceRatioPattern} m1sd
 * @property {PriceRatioPattern} m15sd
 * @property {PriceRatioPattern} m2sd
 * @property {PriceRatioPattern} m25sd
 * @property {PriceRatioPattern} m3sd
 */

/**
 * @typedef {Object} SeriesTree_Cohorts_Utxo_AgeRange
 * @property {ActivityOutputsRealizedSupplyUnrealizedPattern} under1h
 * @property {ActivityOutputsRealizedSupplyUnrealizedPattern} _1hTo1d
 * @property {ActivityOutputsRealizedSupplyUnrealizedPattern} _1dTo1w
 * @property {ActivityOutputsRealizedSupplyUnrealizedPattern} _1wTo1m
 * @property {ActivityOutputsRealizedSupplyUnrealizedPattern} _1mTo2m
 * @property {ActivityOutputsRealizedSupplyUnrealizedPattern} _2mTo3m
 * @property {ActivityOutputsRealizedSupplyUnrealizedPattern} _3mTo4m
 * @property {ActivityOutputsRealizedSupplyUnrealizedPattern} _4mTo5m
 * @property {ActivityOutputsRealizedSupplyUnrealizedPattern} _5mTo6m
 * @property {ActivityOutputsRealizedSupplyUnrealizedPattern} _6mTo1y
 * @property {ActivityOutputsRealizedSupplyUnrealizedPattern} _1yTo2y
 * @property {ActivityOutputsRealizedSupplyUnrealizedPattern} _2yTo3y
 * @property {ActivityOutputsRealizedSupplyUnrealizedPattern} _3yTo4y
 * @property {ActivityOutputsRealizedSupplyUnrealizedPattern} _4yTo5y
 * @property {ActivityOutputsRealizedSupplyUnrealizedPattern} _5yTo6y
 * @property {ActivityOutputsRealizedSupplyUnrealizedPattern} _6yTo7y
 * @property {ActivityOutputsRealizedSupplyUnrealizedPattern} _7yTo8y
 * @property {ActivityOutputsRealizedSupplyUnrealizedPattern} _8yTo10y
 * @property {ActivityOutputsRealizedSupplyUnrealizedPattern} _10yTo12y
 * @property {ActivityOutputsRealizedSupplyUnrealizedPattern} _12yTo15y
 * @property {ActivityOutputsRealizedSupplyUnrealizedPattern} over15y
 */

/**
 * @typedef {Object} SeriesTree_Cohorts_Utxo_UnderAge
 * @property {ActivityOutputsRealizedSupplyUnrealizedPattern} _1w
 * @property {ActivityOutputsRealizedSupplyUnrealizedPattern} _1m
 * @property {ActivityOutputsRealizedSupplyUnrealizedPattern} _2m
 * @property {ActivityOutputsRealizedSupplyUnrealizedPattern} _3m
 * @property {ActivityOutputsRealizedSupplyUnrealizedPattern} _4m
 * @property {ActivityOutputsRealizedSupplyUnrealizedPattern} _5m
 * @property {ActivityOutputsRealizedSupplyUnrealizedPattern} _6m
 * @property {ActivityOutputsRealizedSupplyUnrealizedPattern} _1y
 * @property {ActivityOutputsRealizedSupplyUnrealizedPattern} _2y
 * @property {ActivityOutputsRealizedSupplyUnrealizedPattern} _3y
 * @property {ActivityOutputsRealizedSupplyUnrealizedPattern} _4y
 * @property {ActivityOutputsRealizedSupplyUnrealizedPattern} _5y
 * @property {ActivityOutputsRealizedSupplyUnrealizedPattern} _6y
 * @property {ActivityOutputsRealizedSupplyUnrealizedPattern} _7y
 * @property {ActivityOutputsRealizedSupplyUnrealizedPattern} _8y
 * @property {ActivityOutputsRealizedSupplyUnrealizedPattern} _10y
 * @property {ActivityOutputsRealizedSupplyUnrealizedPattern} _12y
 * @property {ActivityOutputsRealizedSupplyUnrealizedPattern} _15y
 */

/**
 * @typedef {Object} SeriesTree_Cohorts_Utxo_OverAge
 * @property {ActivityOutputsRealizedSupplyUnrealizedPattern} _1d
 * @property {ActivityOutputsRealizedSupplyUnrealizedPattern} _1w
 * @property {ActivityOutputsRealizedSupplyUnrealizedPattern} _1m
 * @property {ActivityOutputsRealizedSupplyUnrealizedPattern} _2m
 * @property {ActivityOutputsRealizedSupplyUnrealizedPattern} _3m
 * @property {ActivityOutputsRealizedSupplyUnrealizedPattern} _4m
 * @property {ActivityOutputsRealizedSupplyUnrealizedPattern} _5m
 * @property {ActivityOutputsRealizedSupplyUnrealizedPattern} _6m
 * @property {ActivityOutputsRealizedSupplyUnrealizedPattern} _1y
 * @property {ActivityOutputsRealizedSupplyUnrealizedPattern} _2y
 * @property {ActivityOutputsRealizedSupplyUnrealizedPattern} _3y
 * @property {ActivityOutputsRealizedSupplyUnrealizedPattern} _4y
 * @property {ActivityOutputsRealizedSupplyUnrealizedPattern} _5y
 * @property {ActivityOutputsRealizedSupplyUnrealizedPattern} _6y
 * @property {ActivityOutputsRealizedSupplyUnrealizedPattern} _7y
 * @property {ActivityOutputsRealizedSupplyUnrealizedPattern} _8y
 * @property {ActivityOutputsRealizedSupplyUnrealizedPattern} _10y
 * @property {ActivityOutputsRealizedSupplyUnrealizedPattern} _12y
 */

/**
 * @typedef {Object} SeriesTree_Cohorts_Utxo_Epoch
 * @property {ActivityOutputsRealizedSupplyUnrealizedPattern} _0
 * @property {ActivityOutputsRealizedSupplyUnrealizedPattern} _1
 * @property {ActivityOutputsRealizedSupplyUnrealizedPattern} _2
 * @property {ActivityOutputsRealizedSupplyUnrealizedPattern} _3
 * @property {ActivityOutputsRealizedSupplyUnrealizedPattern} _4
 */

/**
 * @typedef {Object} SeriesTree_Cohorts_Utxo_Class
 * @property {ActivityOutputsRealizedSupplyUnrealizedPattern} _2009
 * @property {ActivityOutputsRealizedSupplyUnrealizedPattern} _2010
 * @property {ActivityOutputsRealizedSupplyUnrealizedPattern} _2011
 * @property {ActivityOutputsRealizedSupplyUnrealizedPattern} _2012
 * @property {ActivityOutputsRealizedSupplyUnrealizedPattern} _2013
 * @property {ActivityOutputsRealizedSupplyUnrealizedPattern} _2014
 * @property {ActivityOutputsRealizedSupplyUnrealizedPattern} _2015
 * @property {ActivityOutputsRealizedSupplyUnrealizedPattern} _2016
 * @property {ActivityOutputsRealizedSupplyUnrealizedPattern} _2017
 * @property {ActivityOutputsRealizedSupplyUnrealizedPattern} _2018
 * @property {ActivityOutputsRealizedSupplyUnrealizedPattern} _2019
 * @property {ActivityOutputsRealizedSupplyUnrealizedPattern} _2020
 * @property {ActivityOutputsRealizedSupplyUnrealizedPattern} _2021
 * @property {ActivityOutputsRealizedSupplyUnrealizedPattern} _2022
 * @property {ActivityOutputsRealizedSupplyUnrealizedPattern} _2023
 * @property {ActivityOutputsRealizedSupplyUnrealizedPattern} _2024
 * @property {ActivityOutputsRealizedSupplyUnrealizedPattern} _2025
 * @property {ActivityOutputsRealizedSupplyUnrealizedPattern} _2026
 */

/**
 * @typedef {Object} SeriesTree_Cohorts_Utxo_Entry
 * @property {SeriesTree_Cohorts_Utxo_Entry_Discount} discount
 * @property {SeriesTree_Cohorts_Utxo_Entry_Premium} premium
 */

/**
 * @typedef {Object} SeriesTree_Cohorts_Utxo_Entry_Discount
 * @property {DeltaDominanceHalfInTotalPattern2} supply
 * @property {SpendingSpentUnspentPattern} outputs
 * @property {CoindaysCoinyearsDormancyTransferPattern} activity
 * @property {SeriesTree_Cohorts_Utxo_Entry_Discount_Realized} realized
 * @property {InMaxMinPerSupplyPattern} costBasis
 * @property {CapitalizedGrossInvestedLossNetNuplProfitSentimentPattern2} unrealized
 * @property {InPattern} investedCapital
 */

/**
 * @typedef {Object} SeriesTree_Cohorts_Utxo_Entry_Discount_Realized
 * @property {CentsDeltaToUsdPattern} cap
 * @property {BlockCumulativeSumPattern} profit
 * @property {BlockCumulativeNegativeSumPattern} loss
 * @property {SeriesTree_Cohorts_Utxo_Entry_Discount_Realized_Price} price
 * @property {SeriesPattern1<StoredF32>} mvrv
 * @property {BlockChangeCumulativeDeltaSumPattern} netPnl
 * @property {RatioValuePattern2} sopr
 * @property {BlockCumulativeSumPattern} grossPnl
 * @property {_1m1w1y24hPattern8} sellSideRiskRatio
 * @property {BlockCumulativeSumPattern} peakRegret
 * @property {PricePattern} capitalized
 * @property {_1m1w1y24hPattern<StoredF64>} profitToLossRatio
 */

/**
 * @typedef {Object} SeriesTree_Cohorts_Utxo_Entry_Discount_Realized_Price
 * @property {SeriesPattern1<Dollars>} usd
 * @property {SeriesPattern1<Cents>} cents
 * @property {SeriesPattern1<SatsFract>} sats
 * @property {SeriesPattern1<BasisPoints32>} bps
 * @property {SeriesPattern1<StoredF32>} ratio
 * @property {Pct0Pct1Pct2Pct5Pct95Pct98Pct99Pattern} percentiles
 * @property {_1m1w1y2y4yAllPattern} sma
 * @property {SeriesTree_Cohorts_Utxo_Entry_Discount_Realized_Price_StdDev} stdDev
 */

/**
 * @typedef {Object} SeriesTree_Cohorts_Utxo_Entry_Discount_Realized_Price_StdDev
 * @property {SeriesTree_Cohorts_Utxo_Entry_Discount_Realized_Price_StdDev_All} all
 * @property {SeriesTree_Cohorts_Utxo_Entry_Discount_Realized_Price_StdDev_4y} _4y
 * @property {SeriesTree_Cohorts_Utxo_Entry_Discount_Realized_Price_StdDev_2y} _2y
 * @property {SeriesTree_Cohorts_Utxo_Entry_Discount_Realized_Price_StdDev_1y} _1y
 */

/**
 * @typedef {Object} SeriesTree_Cohorts_Utxo_Entry_Discount_Realized_Price_StdDev_All
 * @property {SeriesPattern1<StoredF32>} sd
 * @property {SeriesPattern1<StoredF32>} zscore
 * @property {CentsSatsUsdPattern} _0sd
 * @property {PriceRatioPattern} p05sd
 * @property {PriceRatioPattern} p1sd
 * @property {PriceRatioPattern} p15sd
 * @property {PriceRatioPattern} p2sd
 * @property {PriceRatioPattern} p25sd
 * @property {PriceRatioPattern} p3sd
 * @property {PriceRatioPattern} m05sd
 * @property {PriceRatioPattern} m1sd
 * @property {PriceRatioPattern} m15sd
 * @property {PriceRatioPattern} m2sd
 * @property {PriceRatioPattern} m25sd
 * @property {PriceRatioPattern} m3sd
 */

/**
 * @typedef {Object} SeriesTree_Cohorts_Utxo_Entry_Discount_Realized_Price_StdDev_4y
 * @property {SeriesPattern1<StoredF32>} sd
 * @property {SeriesPattern1<StoredF32>} zscore
 * @property {CentsSatsUsdPattern} _0sd
 * @property {PriceRatioPattern} p05sd
 * @property {PriceRatioPattern} p1sd
 * @property {PriceRatioPattern} p15sd
 * @property {PriceRatioPattern} p2sd
 * @property {PriceRatioPattern} p25sd
 * @property {PriceRatioPattern} p3sd
 * @property {PriceRatioPattern} m05sd
 * @property {PriceRatioPattern} m1sd
 * @property {PriceRatioPattern} m15sd
 * @property {PriceRatioPattern} m2sd
 * @property {PriceRatioPattern} m25sd
 * @property {PriceRatioPattern} m3sd
 */

/**
 * @typedef {Object} SeriesTree_Cohorts_Utxo_Entry_Discount_Realized_Price_StdDev_2y
 * @property {SeriesPattern1<StoredF32>} sd
 * @property {SeriesPattern1<StoredF32>} zscore
 * @property {CentsSatsUsdPattern} _0sd
 * @property {PriceRatioPattern} p05sd
 * @property {PriceRatioPattern} p1sd
 * @property {PriceRatioPattern} p15sd
 * @property {PriceRatioPattern} p2sd
 * @property {PriceRatioPattern} p25sd
 * @property {PriceRatioPattern} p3sd
 * @property {PriceRatioPattern} m05sd
 * @property {PriceRatioPattern} m1sd
 * @property {PriceRatioPattern} m15sd
 * @property {PriceRatioPattern} m2sd
 * @property {PriceRatioPattern} m25sd
 * @property {PriceRatioPattern} m3sd
 */

/**
 * @typedef {Object} SeriesTree_Cohorts_Utxo_Entry_Discount_Realized_Price_StdDev_1y
 * @property {SeriesPattern1<StoredF32>} sd
 * @property {SeriesPattern1<StoredF32>} zscore
 * @property {CentsSatsUsdPattern} _0sd
 * @property {PriceRatioPattern} p05sd
 * @property {PriceRatioPattern} p1sd
 * @property {PriceRatioPattern} p15sd
 * @property {PriceRatioPattern} p2sd
 * @property {PriceRatioPattern} p25sd
 * @property {PriceRatioPattern} p3sd
 * @property {PriceRatioPattern} m05sd
 * @property {PriceRatioPattern} m1sd
 * @property {PriceRatioPattern} m15sd
 * @property {PriceRatioPattern} m2sd
 * @property {PriceRatioPattern} m25sd
 * @property {PriceRatioPattern} m3sd
 */

/**
 * @typedef {Object} SeriesTree_Cohorts_Utxo_Entry_Premium
 * @property {DeltaDominanceHalfInTotalPattern2} supply
 * @property {SpendingSpentUnspentPattern} outputs
 * @property {CoindaysCoinyearsDormancyTransferPattern} activity
 * @property {SeriesTree_Cohorts_Utxo_Entry_Premium_Realized} realized
 * @property {InMaxMinPerSupplyPattern} costBasis
 * @property {CapitalizedGrossInvestedLossNetNuplProfitSentimentPattern2} unrealized
 * @property {InPattern} investedCapital
 */

/**
 * @typedef {Object} SeriesTree_Cohorts_Utxo_Entry_Premium_Realized
 * @property {CentsDeltaToUsdPattern} cap
 * @property {BlockCumulativeSumPattern} profit
 * @property {BlockCumulativeNegativeSumPattern} loss
 * @property {SeriesTree_Cohorts_Utxo_Entry_Premium_Realized_Price} price
 * @property {SeriesPattern1<StoredF32>} mvrv
 * @property {BlockChangeCumulativeDeltaSumPattern} netPnl
 * @property {RatioValuePattern2} sopr
 * @property {BlockCumulativeSumPattern} grossPnl
 * @property {_1m1w1y24hPattern8} sellSideRiskRatio
 * @property {BlockCumulativeSumPattern} peakRegret
 * @property {PricePattern} capitalized
 * @property {_1m1w1y24hPattern<StoredF64>} profitToLossRatio
 */

/**
 * @typedef {Object} SeriesTree_Cohorts_Utxo_Entry_Premium_Realized_Price
 * @property {SeriesPattern1<Dollars>} usd
 * @property {SeriesPattern1<Cents>} cents
 * @property {SeriesPattern1<SatsFract>} sats
 * @property {SeriesPattern1<BasisPoints32>} bps
 * @property {SeriesPattern1<StoredF32>} ratio
 * @property {Pct0Pct1Pct2Pct5Pct95Pct98Pct99Pattern} percentiles
 * @property {_1m1w1y2y4yAllPattern} sma
 * @property {SeriesTree_Cohorts_Utxo_Entry_Premium_Realized_Price_StdDev} stdDev
 */

/**
 * @typedef {Object} SeriesTree_Cohorts_Utxo_Entry_Premium_Realized_Price_StdDev
 * @property {SeriesTree_Cohorts_Utxo_Entry_Premium_Realized_Price_StdDev_All} all
 * @property {SeriesTree_Cohorts_Utxo_Entry_Premium_Realized_Price_StdDev_4y} _4y
 * @property {SeriesTree_Cohorts_Utxo_Entry_Premium_Realized_Price_StdDev_2y} _2y
 * @property {SeriesTree_Cohorts_Utxo_Entry_Premium_Realized_Price_StdDev_1y} _1y
 */

/**
 * @typedef {Object} SeriesTree_Cohorts_Utxo_Entry_Premium_Realized_Price_StdDev_All
 * @property {SeriesPattern1<StoredF32>} sd
 * @property {SeriesPattern1<StoredF32>} zscore
 * @property {CentsSatsUsdPattern} _0sd
 * @property {PriceRatioPattern} p05sd
 * @property {PriceRatioPattern} p1sd
 * @property {PriceRatioPattern} p15sd
 * @property {PriceRatioPattern} p2sd
 * @property {PriceRatioPattern} p25sd
 * @property {PriceRatioPattern} p3sd
 * @property {PriceRatioPattern} m05sd
 * @property {PriceRatioPattern} m1sd
 * @property {PriceRatioPattern} m15sd
 * @property {PriceRatioPattern} m2sd
 * @property {PriceRatioPattern} m25sd
 * @property {PriceRatioPattern} m3sd
 */

/**
 * @typedef {Object} SeriesTree_Cohorts_Utxo_Entry_Premium_Realized_Price_StdDev_4y
 * @property {SeriesPattern1<StoredF32>} sd
 * @property {SeriesPattern1<StoredF32>} zscore
 * @property {CentsSatsUsdPattern} _0sd
 * @property {PriceRatioPattern} p05sd
 * @property {PriceRatioPattern} p1sd
 * @property {PriceRatioPattern} p15sd
 * @property {PriceRatioPattern} p2sd
 * @property {PriceRatioPattern} p25sd
 * @property {PriceRatioPattern} p3sd
 * @property {PriceRatioPattern} m05sd
 * @property {PriceRatioPattern} m1sd
 * @property {PriceRatioPattern} m15sd
 * @property {PriceRatioPattern} m2sd
 * @property {PriceRatioPattern} m25sd
 * @property {PriceRatioPattern} m3sd
 */

/**
 * @typedef {Object} SeriesTree_Cohorts_Utxo_Entry_Premium_Realized_Price_StdDev_2y
 * @property {SeriesPattern1<StoredF32>} sd
 * @property {SeriesPattern1<StoredF32>} zscore
 * @property {CentsSatsUsdPattern} _0sd
 * @property {PriceRatioPattern} p05sd
 * @property {PriceRatioPattern} p1sd
 * @property {PriceRatioPattern} p15sd
 * @property {PriceRatioPattern} p2sd
 * @property {PriceRatioPattern} p25sd
 * @property {PriceRatioPattern} p3sd
 * @property {PriceRatioPattern} m05sd
 * @property {PriceRatioPattern} m1sd
 * @property {PriceRatioPattern} m15sd
 * @property {PriceRatioPattern} m2sd
 * @property {PriceRatioPattern} m25sd
 * @property {PriceRatioPattern} m3sd
 */

/**
 * @typedef {Object} SeriesTree_Cohorts_Utxo_Entry_Premium_Realized_Price_StdDev_1y
 * @property {SeriesPattern1<StoredF32>} sd
 * @property {SeriesPattern1<StoredF32>} zscore
 * @property {CentsSatsUsdPattern} _0sd
 * @property {PriceRatioPattern} p05sd
 * @property {PriceRatioPattern} p1sd
 * @property {PriceRatioPattern} p15sd
 * @property {PriceRatioPattern} p2sd
 * @property {PriceRatioPattern} p25sd
 * @property {PriceRatioPattern} p3sd
 * @property {PriceRatioPattern} m05sd
 * @property {PriceRatioPattern} m1sd
 * @property {PriceRatioPattern} m15sd
 * @property {PriceRatioPattern} m2sd
 * @property {PriceRatioPattern} m25sd
 * @property {PriceRatioPattern} m3sd
 */

/**
 * @typedef {Object} SeriesTree_Cohorts_Utxo_OverAmount
 * @property {ActivityOutputsRealizedSupplyUnrealizedPattern2} _1sat
 * @property {ActivityOutputsRealizedSupplyUnrealizedPattern2} _10sats
 * @property {ActivityOutputsRealizedSupplyUnrealizedPattern2} _100sats
 * @property {ActivityOutputsRealizedSupplyUnrealizedPattern2} _1kSats
 * @property {ActivityOutputsRealizedSupplyUnrealizedPattern2} _10kSats
 * @property {ActivityOutputsRealizedSupplyUnrealizedPattern2} _100kSats
 * @property {ActivityOutputsRealizedSupplyUnrealizedPattern2} _1mSats
 * @property {ActivityOutputsRealizedSupplyUnrealizedPattern2} _10mSats
 * @property {ActivityOutputsRealizedSupplyUnrealizedPattern2} _1btc
 * @property {ActivityOutputsRealizedSupplyUnrealizedPattern2} _10btc
 * @property {ActivityOutputsRealizedSupplyUnrealizedPattern2} _100btc
 * @property {ActivityOutputsRealizedSupplyUnrealizedPattern2} _1kBtc
 * @property {ActivityOutputsRealizedSupplyUnrealizedPattern2} _10kBtc
 */

/**
 * @typedef {Object} SeriesTree_Cohorts_Utxo_AmountRange
 * @property {ActivityOutputsRealizedSupplyUnrealizedPattern2} _0sats
 * @property {ActivityOutputsRealizedSupplyUnrealizedPattern2} _1satTo10sats
 * @property {ActivityOutputsRealizedSupplyUnrealizedPattern2} _10satsTo100sats
 * @property {ActivityOutputsRealizedSupplyUnrealizedPattern2} _100satsTo1kSats
 * @property {ActivityOutputsRealizedSupplyUnrealizedPattern2} _1kSatsTo10kSats
 * @property {ActivityOutputsRealizedSupplyUnrealizedPattern2} _10kSatsTo100kSats
 * @property {ActivityOutputsRealizedSupplyUnrealizedPattern2} _100kSatsTo1mSats
 * @property {ActivityOutputsRealizedSupplyUnrealizedPattern2} _1mSatsTo10mSats
 * @property {ActivityOutputsRealizedSupplyUnrealizedPattern2} _10mSatsTo1btc
 * @property {ActivityOutputsRealizedSupplyUnrealizedPattern2} _1btcTo10btc
 * @property {ActivityOutputsRealizedSupplyUnrealizedPattern2} _10btcTo100btc
 * @property {ActivityOutputsRealizedSupplyUnrealizedPattern2} _100btcTo1kBtc
 * @property {ActivityOutputsRealizedSupplyUnrealizedPattern2} _1kBtcTo10kBtc
 * @property {ActivityOutputsRealizedSupplyUnrealizedPattern2} _10kBtcTo100kBtc
 * @property {ActivityOutputsRealizedSupplyUnrealizedPattern2} over100kBtc
 */

/**
 * @typedef {Object} SeriesTree_Cohorts_Utxo_UnderAmount
 * @property {ActivityOutputsRealizedSupplyUnrealizedPattern2} _10sats
 * @property {ActivityOutputsRealizedSupplyUnrealizedPattern2} _100sats
 * @property {ActivityOutputsRealizedSupplyUnrealizedPattern2} _1kSats
 * @property {ActivityOutputsRealizedSupplyUnrealizedPattern2} _10kSats
 * @property {ActivityOutputsRealizedSupplyUnrealizedPattern2} _100kSats
 * @property {ActivityOutputsRealizedSupplyUnrealizedPattern2} _1mSats
 * @property {ActivityOutputsRealizedSupplyUnrealizedPattern2} _10mSats
 * @property {ActivityOutputsRealizedSupplyUnrealizedPattern2} _1btc
 * @property {ActivityOutputsRealizedSupplyUnrealizedPattern2} _10btc
 * @property {ActivityOutputsRealizedSupplyUnrealizedPattern2} _100btc
 * @property {ActivityOutputsRealizedSupplyUnrealizedPattern2} _1kBtc
 * @property {ActivityOutputsRealizedSupplyUnrealizedPattern2} _10kBtc
 * @property {ActivityOutputsRealizedSupplyUnrealizedPattern2} _100kBtc
 */

/**
 * @typedef {Object} SeriesTree_Cohorts_Utxo_Type
 * @property {ActivityOutputsRealizedSupplyUnrealizedPattern3} p2pk65
 * @property {ActivityOutputsRealizedSupplyUnrealizedPattern3} p2pk33
 * @property {ActivityOutputsRealizedSupplyUnrealizedPattern3} p2pkh
 * @property {ActivityOutputsRealizedSupplyUnrealizedPattern3} p2ms
 * @property {ActivityOutputsRealizedSupplyUnrealizedPattern3} p2sh
 * @property {ActivityOutputsRealizedSupplyUnrealizedPattern3} p2wpkh
 * @property {ActivityOutputsRealizedSupplyUnrealizedPattern3} p2wsh
 * @property {ActivityOutputsRealizedSupplyUnrealizedPattern3} p2tr
 * @property {ActivityOutputsRealizedSupplyUnrealizedPattern3} p2a
 * @property {ActivityOutputsRealizedSupplyUnrealizedPattern3} unknown
 * @property {ActivityOutputsRealizedSupplyUnrealizedPattern3} empty
 */

/**
 * @typedef {Object} SeriesTree_Cohorts_Utxo_Profitability
 * @property {SeriesTree_Cohorts_Utxo_Profitability_Range} range
 * @property {SeriesTree_Cohorts_Utxo_Profitability_Profit} profit
 * @property {SeriesTree_Cohorts_Utxo_Profitability_Loss} loss
 */

/**
 * @typedef {Object} SeriesTree_Cohorts_Utxo_Profitability_Range
 * @property {NuplRealizedSupplyUnrealizedPattern} over1000pctInProfit
 * @property {NuplRealizedSupplyUnrealizedPattern} _500pctTo1000pctInProfit
 * @property {NuplRealizedSupplyUnrealizedPattern} _300pctTo500pctInProfit
 * @property {NuplRealizedSupplyUnrealizedPattern} _200pctTo300pctInProfit
 * @property {NuplRealizedSupplyUnrealizedPattern} _100pctTo200pctInProfit
 * @property {NuplRealizedSupplyUnrealizedPattern} _90pctTo100pctInProfit
 * @property {NuplRealizedSupplyUnrealizedPattern} _80pctTo90pctInProfit
 * @property {NuplRealizedSupplyUnrealizedPattern} _70pctTo80pctInProfit
 * @property {NuplRealizedSupplyUnrealizedPattern} _60pctTo70pctInProfit
 * @property {NuplRealizedSupplyUnrealizedPattern} _50pctTo60pctInProfit
 * @property {NuplRealizedSupplyUnrealizedPattern} _40pctTo50pctInProfit
 * @property {NuplRealizedSupplyUnrealizedPattern} _30pctTo40pctInProfit
 * @property {NuplRealizedSupplyUnrealizedPattern} _20pctTo30pctInProfit
 * @property {NuplRealizedSupplyUnrealizedPattern} _10pctTo20pctInProfit
 * @property {NuplRealizedSupplyUnrealizedPattern} _0pctTo10pctInProfit
 * @property {NuplRealizedSupplyUnrealizedPattern} _0pctTo10pctInLoss
 * @property {NuplRealizedSupplyUnrealizedPattern} _10pctTo20pctInLoss
 * @property {NuplRealizedSupplyUnrealizedPattern} _20pctTo30pctInLoss
 * @property {NuplRealizedSupplyUnrealizedPattern} _30pctTo40pctInLoss
 * @property {NuplRealizedSupplyUnrealizedPattern} _40pctTo50pctInLoss
 * @property {NuplRealizedSupplyUnrealizedPattern} _50pctTo60pctInLoss
 * @property {NuplRealizedSupplyUnrealizedPattern} _60pctTo70pctInLoss
 * @property {NuplRealizedSupplyUnrealizedPattern} _70pctTo80pctInLoss
 * @property {NuplRealizedSupplyUnrealizedPattern} _80pctTo90pctInLoss
 * @property {NuplRealizedSupplyUnrealizedPattern} _90pctTo100pctInLoss
 */

/**
 * @typedef {Object} SeriesTree_Cohorts_Utxo_Profitability_Profit
 * @property {NuplRealizedSupplyUnrealizedPattern} all
 * @property {NuplRealizedSupplyUnrealizedPattern} _10pct
 * @property {NuplRealizedSupplyUnrealizedPattern} _20pct
 * @property {NuplRealizedSupplyUnrealizedPattern} _30pct
 * @property {NuplRealizedSupplyUnrealizedPattern} _40pct
 * @property {NuplRealizedSupplyUnrealizedPattern} _50pct
 * @property {NuplRealizedSupplyUnrealizedPattern} _60pct
 * @property {NuplRealizedSupplyUnrealizedPattern} _70pct
 * @property {NuplRealizedSupplyUnrealizedPattern} _80pct
 * @property {NuplRealizedSupplyUnrealizedPattern} _90pct
 * @property {NuplRealizedSupplyUnrealizedPattern} _100pct
 * @property {NuplRealizedSupplyUnrealizedPattern} _200pct
 * @property {NuplRealizedSupplyUnrealizedPattern} _300pct
 * @property {NuplRealizedSupplyUnrealizedPattern} _500pct
 */

/**
 * @typedef {Object} SeriesTree_Cohorts_Utxo_Profitability_Loss
 * @property {NuplRealizedSupplyUnrealizedPattern} all
 * @property {NuplRealizedSupplyUnrealizedPattern} _10pct
 * @property {NuplRealizedSupplyUnrealizedPattern} _20pct
 * @property {NuplRealizedSupplyUnrealizedPattern} _30pct
 * @property {NuplRealizedSupplyUnrealizedPattern} _40pct
 * @property {NuplRealizedSupplyUnrealizedPattern} _50pct
 * @property {NuplRealizedSupplyUnrealizedPattern} _60pct
 * @property {NuplRealizedSupplyUnrealizedPattern} _70pct
 * @property {NuplRealizedSupplyUnrealizedPattern} _80pct
 */

/**
 * @typedef {Object} SeriesTree_Cohorts_Utxo_Matured
 * @property {AverageBlockCumulativeSumPattern3} under1h
 * @property {AverageBlockCumulativeSumPattern3} _1hTo1d
 * @property {AverageBlockCumulativeSumPattern3} _1dTo1w
 * @property {AverageBlockCumulativeSumPattern3} _1wTo1m
 * @property {AverageBlockCumulativeSumPattern3} _1mTo2m
 * @property {AverageBlockCumulativeSumPattern3} _2mTo3m
 * @property {AverageBlockCumulativeSumPattern3} _3mTo4m
 * @property {AverageBlockCumulativeSumPattern3} _4mTo5m
 * @property {AverageBlockCumulativeSumPattern3} _5mTo6m
 * @property {AverageBlockCumulativeSumPattern3} _6mTo1y
 * @property {AverageBlockCumulativeSumPattern3} _1yTo2y
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
 * @typedef {Object} SeriesTree_Cohorts_Addr
 * @property {SeriesTree_Cohorts_Addr_OverAmount} overAmount
 * @property {SeriesTree_Cohorts_Addr_AmountRange} amountRange
 * @property {SeriesTree_Cohorts_Addr_UnderAmount} underAmount
 */

/**
 * @typedef {Object} SeriesTree_Cohorts_Addr_OverAmount
 * @property {ActivityAddrOutputsRealizedSupplyUnrealizedPattern} _1sat
 * @property {ActivityAddrOutputsRealizedSupplyUnrealizedPattern} _10sats
 * @property {ActivityAddrOutputsRealizedSupplyUnrealizedPattern} _100sats
 * @property {ActivityAddrOutputsRealizedSupplyUnrealizedPattern} _1kSats
 * @property {ActivityAddrOutputsRealizedSupplyUnrealizedPattern} _10kSats
 * @property {ActivityAddrOutputsRealizedSupplyUnrealizedPattern} _100kSats
 * @property {ActivityAddrOutputsRealizedSupplyUnrealizedPattern} _1mSats
 * @property {ActivityAddrOutputsRealizedSupplyUnrealizedPattern} _10mSats
 * @property {ActivityAddrOutputsRealizedSupplyUnrealizedPattern} _1btc
 * @property {ActivityAddrOutputsRealizedSupplyUnrealizedPattern} _10btc
 * @property {ActivityAddrOutputsRealizedSupplyUnrealizedPattern} _100btc
 * @property {ActivityAddrOutputsRealizedSupplyUnrealizedPattern} _1kBtc
 * @property {ActivityAddrOutputsRealizedSupplyUnrealizedPattern} _10kBtc
 */

/**
 * @typedef {Object} SeriesTree_Cohorts_Addr_AmountRange
 * @property {ActivityAddrOutputsRealizedSupplyUnrealizedPattern} _0sats
 * @property {ActivityAddrOutputsRealizedSupplyUnrealizedPattern} _1satTo10sats
 * @property {ActivityAddrOutputsRealizedSupplyUnrealizedPattern} _10satsTo100sats
 * @property {ActivityAddrOutputsRealizedSupplyUnrealizedPattern} _100satsTo1kSats
 * @property {ActivityAddrOutputsRealizedSupplyUnrealizedPattern} _1kSatsTo10kSats
 * @property {ActivityAddrOutputsRealizedSupplyUnrealizedPattern} _10kSatsTo100kSats
 * @property {ActivityAddrOutputsRealizedSupplyUnrealizedPattern} _100kSatsTo1mSats
 * @property {ActivityAddrOutputsRealizedSupplyUnrealizedPattern} _1mSatsTo10mSats
 * @property {ActivityAddrOutputsRealizedSupplyUnrealizedPattern} _10mSatsTo1btc
 * @property {ActivityAddrOutputsRealizedSupplyUnrealizedPattern} _1btcTo10btc
 * @property {ActivityAddrOutputsRealizedSupplyUnrealizedPattern} _10btcTo100btc
 * @property {ActivityAddrOutputsRealizedSupplyUnrealizedPattern} _100btcTo1kBtc
 * @property {ActivityAddrOutputsRealizedSupplyUnrealizedPattern} _1kBtcTo10kBtc
 * @property {ActivityAddrOutputsRealizedSupplyUnrealizedPattern} _10kBtcTo100kBtc
 * @property {ActivityAddrOutputsRealizedSupplyUnrealizedPattern} over100kBtc
 */

/**
 * @typedef {Object} SeriesTree_Cohorts_Addr_UnderAmount
 * @property {ActivityAddrOutputsRealizedSupplyUnrealizedPattern} _10sats
 * @property {ActivityAddrOutputsRealizedSupplyUnrealizedPattern} _100sats
 * @property {ActivityAddrOutputsRealizedSupplyUnrealizedPattern} _1kSats
 * @property {ActivityAddrOutputsRealizedSupplyUnrealizedPattern} _10kSats
 * @property {ActivityAddrOutputsRealizedSupplyUnrealizedPattern} _100kSats
 * @property {ActivityAddrOutputsRealizedSupplyUnrealizedPattern} _1mSats
 * @property {ActivityAddrOutputsRealizedSupplyUnrealizedPattern} _10mSats
 * @property {ActivityAddrOutputsRealizedSupplyUnrealizedPattern} _1btc
 * @property {ActivityAddrOutputsRealizedSupplyUnrealizedPattern} _10btc
 * @property {ActivityAddrOutputsRealizedSupplyUnrealizedPattern} _100btc
 * @property {ActivityAddrOutputsRealizedSupplyUnrealizedPattern} _1kBtc
 * @property {ActivityAddrOutputsRealizedSupplyUnrealizedPattern} _10kBtc
 * @property {ActivityAddrOutputsRealizedSupplyUnrealizedPattern} _100kBtc
 */

/**
 * Main BRK client with series tree and API methods
 * @extends BrkClientBase
 */
class BrkClient extends BrkClientBase {
  VERSION = "v0.3.6";

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
    "empty_addr_index"
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
    "_6mTo1y": {
      "id": "6m_to_1y_old",
      "short": "6m-1y",
      "long": "6 Months to 1 Year Old"
    },
    "_1yTo2y": {
      "id": "1y_to_2y_old",
      "short": "1y-2y",
      "long": "1 to 2 Years Old"
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

  UNDER_AGE_NAMES = /** @type {const} */ ({
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
  });

  OVER_AGE_NAMES = /** @type {const} */ ({
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

  OVER_AMOUNT_NAMES = /** @type {const} */ ({
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
    "_1kSats": {
      "id": "over_1k_sats",
      "short": "1k+ sats",
      "long": "Over 1K Sats"
    },
    "_10kSats": {
      "id": "over_10k_sats",
      "short": "10k+ sats",
      "long": "Over 10K Sats"
    },
    "_100kSats": {
      "id": "over_100k_sats",
      "short": "100k+ sats",
      "long": "Over 100K Sats"
    },
    "_1mSats": {
      "id": "over_1m_sats",
      "short": "1M+ sats",
      "long": "Over 1M Sats"
    },
    "_10mSats": {
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
    "_1kBtc": {
      "id": "over_1k_btc",
      "short": "1k+ BTC",
      "long": "Over 1K BTC"
    },
    "_10kBtc": {
      "id": "over_10k_btc",
      "short": "10k+ BTC",
      "long": "Over 10K BTC"
    }
  });

  UNDER_AMOUNT_NAMES = /** @type {const} */ ({
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
    "_1kSats": {
      "id": "under_1k_sats",
      "short": "<1k sats",
      "long": "Under 1K Sats"
    },
    "_10kSats": {
      "id": "under_10k_sats",
      "short": "<10k sats",
      "long": "Under 10K Sats"
    },
    "_100kSats": {
      "id": "under_100k_sats",
      "short": "<100k sats",
      "long": "Under 100K Sats"
    },
    "_1mSats": {
      "id": "under_1m_sats",
      "short": "<1M sats",
      "long": "Under 1M Sats"
    },
    "_10mSats": {
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
    "_1kBtc": {
      "id": "under_1k_btc",
      "short": "<1k BTC",
      "long": "Under 1K BTC"
    },
    "_10kBtc": {
      "id": "under_10k_btc",
      "short": "<10k BTC",
      "long": "Under 10K BTC"
    },
    "_100kBtc": {
      "id": "under_100k_btc",
      "short": "<100k BTC",
      "long": "Under 100K BTC"
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

  PROFIT_NAMES = /** @type {const} */ ({
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
  });

  LOSS_NAMES = /** @type {const} */ ({
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
   * @param {BrkClientOptions|string} options
   */
  constructor(options) {
    super(options);
    /** @type {SeriesTree} */
    this.series = this._buildTree();
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
    return {
      blocks: {
        blockhash: createSeriesPattern18(this, 'blockhash'),
        coinbaseTag: createSeriesPattern18(this, 'coinbase_tag'),
        difficulty: {
          value: createSeriesPattern1(this, 'difficulty'),
          hashrate: createSeriesPattern1(this, 'difficulty_hashrate'),
          adjustment: createBpsPercentRatioPattern(this, 'difficulty_adjustment'),
          epoch: createSeriesPattern1(this, 'difficulty_epoch'),
          blocksToRetarget: createSeriesPattern1(this, 'blocks_to_retarget'),
          daysToRetarget: createSeriesPattern1(this, 'days_to_retarget'),
        },
        time: {
          timestamp: createSeriesPattern18(this, 'timestamp'),
        },
        size: {
          base: createSeriesPattern18(this, 'total_size'),
          cumulative: createSeriesPattern1(this, 'block_size_cumulative'),
          sum: create_1m1w1y24hPattern(this, 'block_size_sum'),
          average: create_1m1w1y24hPattern(this, 'block_size_average'),
          min: create_1m1w1y24hPattern(this, 'block_size_min'),
          max: create_1m1w1y24hPattern(this, 'block_size_max'),
          pct10: create_1m1w1y24hPattern(this, 'block_size_pct10'),
          pct25: create_1m1w1y24hPattern(this, 'block_size_pct25'),
          median: create_1m1w1y24hPattern(this, 'block_size_median'),
          pct75: create_1m1w1y24hPattern(this, 'block_size_pct75'),
          pct90: create_1m1w1y24hPattern(this, 'block_size_pct90'),
        },
        weight: createAverageBaseCumulativeMaxMedianMinPct10Pct25Pct75Pct90SumPattern(this, 'block_weight'),
        segwitTxs: createSeriesPattern18(this, 'segwit_txs'),
        segwitSize: createSeriesPattern18(this, 'segwit_size'),
        segwitWeight: createSeriesPattern18(this, 'segwit_weight'),
        count: {
          target: create_1m1w1y24hPattern(this, 'block_count_target'),
          total: createAverageBlockCumulativeSumPattern2(this, 'block_count'),
        },
        lookback: {
          _1h: createSeriesPattern18(this, 'height_1h_ago'),
          _24h: createSeriesPattern18(this, 'height_24h_ago'),
          _3d: createSeriesPattern18(this, 'height_3d_ago'),
          _1w: createSeriesPattern18(this, 'height_1w_ago'),
          _8d: createSeriesPattern18(this, 'height_8d_ago'),
          _9d: createSeriesPattern18(this, 'height_9d_ago'),
          _12d: createSeriesPattern18(this, 'height_12d_ago'),
          _13d: createSeriesPattern18(this, 'height_13d_ago'),
          _2w: createSeriesPattern18(this, 'height_2w_ago'),
          _21d: createSeriesPattern18(this, 'height_21d_ago'),
          _26d: createSeriesPattern18(this, 'height_26d_ago'),
          _1m: createSeriesPattern18(this, 'height_1m_ago'),
          _34d: createSeriesPattern18(this, 'height_34d_ago'),
          _55d: createSeriesPattern18(this, 'height_55d_ago'),
          _2m: createSeriesPattern18(this, 'height_2m_ago'),
          _9w: createSeriesPattern18(this, 'height_9w_ago'),
          _12w: createSeriesPattern18(this, 'height_12w_ago'),
          _89d: createSeriesPattern18(this, 'height_89d_ago'),
          _3m: createSeriesPattern18(this, 'height_3m_ago'),
          _14w: createSeriesPattern18(this, 'height_14w_ago'),
          _111d: createSeriesPattern18(this, 'height_111d_ago'),
          _144d: createSeriesPattern18(this, 'height_144d_ago'),
          _6m: createSeriesPattern18(this, 'height_6m_ago'),
          _26w: createSeriesPattern18(this, 'height_26w_ago'),
          _200d: createSeriesPattern18(this, 'height_200d_ago'),
          _9m: createSeriesPattern18(this, 'height_9m_ago'),
          _350d: createSeriesPattern18(this, 'height_350d_ago'),
          _12m: createSeriesPattern18(this, 'height_12m_ago'),
          _1y: createSeriesPattern18(this, 'height_1y_ago'),
          _14m: createSeriesPattern18(this, 'height_14m_ago'),
          _2y: createSeriesPattern18(this, 'height_2y_ago'),
          _26m: createSeriesPattern18(this, 'height_26m_ago'),
          _3y: createSeriesPattern18(this, 'height_3y_ago'),
          _200w: createSeriesPattern18(this, 'height_200w_ago'),
          _4y: createSeriesPattern18(this, 'height_4y_ago'),
          _5y: createSeriesPattern18(this, 'height_5y_ago'),
          _6y: createSeriesPattern18(this, 'height_6y_ago'),
          _8y: createSeriesPattern18(this, 'height_8y_ago'),
          _9y: createSeriesPattern18(this, 'height_9y_ago'),
          _10y: createSeriesPattern18(this, 'height_10y_ago'),
          _12y: createSeriesPattern18(this, 'height_12y_ago'),
          _14y: createSeriesPattern18(this, 'height_14y_ago'),
          _26y: createSeriesPattern18(this, 'height_26y_ago'),
        },
        interval: {
          block: createSeriesPattern18(this, 'block_interval'),
          _24h: createSeriesPattern1(this, 'block_interval_average_24h'),
          _1w: createSeriesPattern1(this, 'block_interval_average_1w'),
          _1m: createSeriesPattern1(this, 'block_interval_average_1m'),
          _1y: createSeriesPattern1(this, 'block_interval_average_1y'),
        },
        vbytes: createAverageBlockCumulativeMaxMedianMinPct10Pct25Pct75Pct90SumPattern(this, 'block_vbytes'),
        fullness: {
          bps: createSeriesPattern18(this, 'block_fullness_bps'),
          ratio: createSeriesPattern18(this, 'block_fullness_ratio'),
          percent: createSeriesPattern18(this, 'block_fullness'),
        },
        halving: {
          epoch: createSeriesPattern1(this, 'halving_epoch'),
          blocksToHalving: createSeriesPattern1(this, 'blocks_to_halving'),
          daysToHalving: createSeriesPattern1(this, 'days_to_halving'),
        },
      },
      transactions: {
        raw: {
          firstTxIndex: createSeriesPattern18(this, 'first_tx_index'),
          txid: createSeriesPattern19(this, 'txid'),
          txVersion: createSeriesPattern19(this, 'tx_version'),
          rawLocktime: createSeriesPattern19(this, 'raw_locktime'),
          baseSize: createSeriesPattern19(this, 'base_size'),
          totalSize: createSeriesPattern19(this, 'total_size'),
          totalSigopCost: createSeriesPattern19(this, 'total_sigop_cost'),
          isExplicitlyRbf: createSeriesPattern19(this, 'is_explicitly_rbf'),
          firstTxinIndex: createSeriesPattern19(this, 'first_txin_index'),
          firstTxoutIndex: createSeriesPattern19(this, 'first_txout_index'),
        },
        count: {
          total: createAverageBlockCumulativeMaxMedianMinPct10Pct25Pct75Pct90SumPattern(this, 'tx_count'),
        },
        size: {
          vsize: create_6bBlockTxPattern(this, 'tx_vsize'),
          weight: {
            txIndex: createSeriesPattern19(this, 'tx_weight'),
            block: createMaxMedianMinPct10Pct25Pct75Pct90Pattern2(this, 'tx_weight'),
            _6b: createMaxMedianMinPct10Pct25Pct75Pct90Pattern2(this, 'tx_weight_6b'),
          },
        },
        fees: {
          inputValue: createSeriesPattern19(this, 'input_value'),
          outputValue: createSeriesPattern19(this, 'output_value'),
          fee: create_6bBlockTxPattern(this, 'fee'),
          feeRate: createSeriesPattern19(this, 'fee_rate'),
          effectiveFeeRate: create_6bBlockTxPattern(this, 'effective_fee_rate'),
        },
        versions: {
          v1: createAverageBlockCumulativeSumPattern(this, 'tx_v1'),
          v2: createAverageBlockCumulativeSumPattern(this, 'tx_v2'),
          v3: createAverageBlockCumulativeSumPattern(this, 'tx_v3'),
        },
        volume: {
          transferVolume: createAverageBlockCumulativeSumPattern3(this, 'transfer_volume_bis'),
          txPerSec: create_1m1w1y24hPattern(this, 'tx_per_sec'),
        },
      },
      inputs: {
        raw: {
          firstTxinIndex: createSeriesPattern18(this, 'first_txin_index'),
          outpoint: createSeriesPattern20(this, 'outpoint'),
          txIndex: createSeriesPattern20(this, 'tx_index'),
          outputType: createSeriesPattern20(this, 'output_type'),
          typeIndex: createSeriesPattern20(this, 'type_index'),
        },
        spent: {
          txoutIndex: createSeriesPattern20(this, 'txout_index'),
          value: createSeriesPattern20(this, 'value'),
        },
        count: createCumulativeRollingSumPattern(this, 'input_count'),
        perSec: create_1m1w1y24hPattern(this, 'inputs_per_sec'),
        byType: {
          inputCount: {
            all: createAverageBlockCumulativeSumPattern(this, 'input_count_bis'),
            p2pk65: createAverageBlockCumulativeSumPattern(this, 'p2pk65_prevout_count'),
            p2pk33: createAverageBlockCumulativeSumPattern(this, 'p2pk33_prevout_count'),
            p2pkh: createAverageBlockCumulativeSumPattern(this, 'p2pkh_prevout_count'),
            p2ms: createAverageBlockCumulativeSumPattern(this, 'p2ms_prevout_count'),
            p2sh: createAverageBlockCumulativeSumPattern(this, 'p2sh_prevout_count'),
            p2wpkh: createAverageBlockCumulativeSumPattern(this, 'p2wpkh_prevout_count'),
            p2wsh: createAverageBlockCumulativeSumPattern(this, 'p2wsh_prevout_count'),
            p2tr: createAverageBlockCumulativeSumPattern(this, 'p2tr_prevout_count'),
            p2a: createAverageBlockCumulativeSumPattern(this, 'p2a_prevout_count'),
            unknown: createAverageBlockCumulativeSumPattern(this, 'unknown_outputs_prevout_count'),
            empty: createAverageBlockCumulativeSumPattern(this, 'empty_outputs_prevout_count'),
          },
          inputShare: {
            p2pk65: create_1m1w1y24hBpsPercentRatioPattern(this, 'p2pk65_prevout_share'),
            p2pk33: create_1m1w1y24hBpsPercentRatioPattern(this, 'p2pk33_prevout_share'),
            p2pkh: create_1m1w1y24hBpsPercentRatioPattern(this, 'p2pkh_prevout_share'),
            p2ms: create_1m1w1y24hBpsPercentRatioPattern(this, 'p2ms_prevout_share'),
            p2sh: create_1m1w1y24hBpsPercentRatioPattern(this, 'p2sh_prevout_share'),
            p2wpkh: create_1m1w1y24hBpsPercentRatioPattern(this, 'p2wpkh_prevout_share'),
            p2wsh: create_1m1w1y24hBpsPercentRatioPattern(this, 'p2wsh_prevout_share'),
            p2tr: create_1m1w1y24hBpsPercentRatioPattern(this, 'p2tr_prevout_share'),
            p2a: create_1m1w1y24hBpsPercentRatioPattern(this, 'p2a_prevout_share'),
            unknown: create_1m1w1y24hBpsPercentRatioPattern(this, 'unknown_outputs_prevout_share'),
            empty: create_1m1w1y24hBpsPercentRatioPattern(this, 'empty_outputs_prevout_share'),
          },
          txCount: {
            all: createAverageBlockCumulativeSumPattern(this, 'non_coinbase_tx_count'),
            p2pk65: createAverageBlockCumulativeSumPattern(this, 'tx_count_with_p2pk65_prevout'),
            p2pk33: createAverageBlockCumulativeSumPattern(this, 'tx_count_with_p2pk33_prevout'),
            p2pkh: createAverageBlockCumulativeSumPattern(this, 'tx_count_with_p2pkh_prevout'),
            p2ms: createAverageBlockCumulativeSumPattern(this, 'tx_count_with_p2ms_prevout'),
            p2sh: createAverageBlockCumulativeSumPattern(this, 'tx_count_with_p2sh_prevout'),
            p2wpkh: createAverageBlockCumulativeSumPattern(this, 'tx_count_with_p2wpkh_prevout'),
            p2wsh: createAverageBlockCumulativeSumPattern(this, 'tx_count_with_p2wsh_prevout'),
            p2tr: createAverageBlockCumulativeSumPattern(this, 'tx_count_with_p2tr_prevout'),
            p2a: createAverageBlockCumulativeSumPattern(this, 'tx_count_with_p2a_prevout'),
            unknown: createAverageBlockCumulativeSumPattern(this, 'tx_count_with_unknown_outputs_prevout'),
            empty: createAverageBlockCumulativeSumPattern(this, 'tx_count_with_empty_outputs_prevout'),
          },
          txShare: createEmptyP2aP2msP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshUnknownPattern2(this, 'tx_share_with'),
        },
      },
      outputs: {
        raw: {
          firstTxoutIndex: createSeriesPattern18(this, 'first_txout_index'),
          value: createSeriesPattern21(this, 'value'),
          outputType: createSeriesPattern21(this, 'output_type'),
          typeIndex: createSeriesPattern21(this, 'type_index'),
          txIndex: createSeriesPattern21(this, 'tx_index'),
        },
        spent: {
          txinIndex: createSeriesPattern21(this, 'txin_index'),
        },
        count: {
          total: createCumulativeRollingSumPattern(this, 'output_count'),
        },
        perSec: create_1m1w1y24hPattern(this, 'outputs_per_sec'),
        unspent: {
          count: createSeriesPattern1(this, 'utxo_count_bis'),
        },
        byType: {
          outputCount: {
            all: createAverageBlockCumulativeSumPattern(this, 'output_count_bis'),
            p2pk65: createAverageBlockCumulativeSumPattern(this, 'p2pk65_output_count'),
            p2pk33: createAverageBlockCumulativeSumPattern(this, 'p2pk33_output_count'),
            p2pkh: createAverageBlockCumulativeSumPattern(this, 'p2pkh_output_count'),
            p2ms: createAverageBlockCumulativeSumPattern(this, 'p2ms_output_count'),
            p2sh: createAverageBlockCumulativeSumPattern(this, 'p2sh_output_count'),
            p2wpkh: createAverageBlockCumulativeSumPattern(this, 'p2wpkh_output_count'),
            p2wsh: createAverageBlockCumulativeSumPattern(this, 'p2wsh_output_count'),
            p2tr: createAverageBlockCumulativeSumPattern(this, 'p2tr_output_count'),
            p2a: createAverageBlockCumulativeSumPattern(this, 'p2a_output_count'),
            unknown: createAverageBlockCumulativeSumPattern(this, 'unknown_outputs_output_count'),
            empty: createAverageBlockCumulativeSumPattern(this, 'empty_outputs_output_count'),
            opReturn: createAverageBlockCumulativeSumPattern(this, 'op_return_output_count'),
          },
          spendableOutputCount: createAverageBlockCumulativeSumPattern(this, 'spendable_output_count'),
          outputShare: {
            p2pk65: create_1m1w1y24hBpsPercentRatioPattern(this, 'p2pk65_output_share'),
            p2pk33: create_1m1w1y24hBpsPercentRatioPattern(this, 'p2pk33_output_share'),
            p2pkh: create_1m1w1y24hBpsPercentRatioPattern(this, 'p2pkh_output_share'),
            p2ms: create_1m1w1y24hBpsPercentRatioPattern(this, 'p2ms_output_share'),
            p2sh: create_1m1w1y24hBpsPercentRatioPattern(this, 'p2sh_output_share'),
            p2wpkh: create_1m1w1y24hBpsPercentRatioPattern(this, 'p2wpkh_output_share'),
            p2wsh: create_1m1w1y24hBpsPercentRatioPattern(this, 'p2wsh_output_share'),
            p2tr: create_1m1w1y24hBpsPercentRatioPattern(this, 'p2tr_output_share'),
            p2a: create_1m1w1y24hBpsPercentRatioPattern(this, 'p2a_output_share'),
            unknown: create_1m1w1y24hBpsPercentRatioPattern(this, 'unknown_outputs_output_share'),
            empty: create_1m1w1y24hBpsPercentRatioPattern(this, 'empty_outputs_output_share'),
            opReturn: create_1m1w1y24hBpsPercentRatioPattern(this, 'op_return_output_share'),
          },
          txCount: createAllEmptyOpP2aP2msP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshUnknownPattern(this, 'tx_count'),
          txShare: createEmptyOpP2aP2msP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshUnknownPattern2(this, 'tx_share_with'),
        },
        value: {
          opReturn: createBlockCumulativePattern(this, 'op_return_value'),
        },
      },
      addrs: {
        raw: {
          p2pk65: {
            firstIndex: createSeriesPattern18(this, 'first_p2pk65_addr_index'),
            bytes: createSeriesPattern27(this, 'p2pk65_bytes'),
          },
          p2pk33: {
            firstIndex: createSeriesPattern18(this, 'first_p2pk33_addr_index'),
            bytes: createSeriesPattern26(this, 'p2pk33_bytes'),
          },
          p2pkh: {
            firstIndex: createSeriesPattern18(this, 'first_p2pkh_addr_index'),
            bytes: createSeriesPattern28(this, 'p2pkh_bytes'),
          },
          p2sh: {
            firstIndex: createSeriesPattern18(this, 'first_p2sh_addr_index'),
            bytes: createSeriesPattern29(this, 'p2sh_bytes'),
          },
          p2wpkh: {
            firstIndex: createSeriesPattern18(this, 'first_p2wpkh_addr_index'),
            bytes: createSeriesPattern31(this, 'p2wpkh_bytes'),
          },
          p2wsh: {
            firstIndex: createSeriesPattern18(this, 'first_p2wsh_addr_index'),
            bytes: createSeriesPattern32(this, 'p2wsh_bytes'),
          },
          p2tr: {
            firstIndex: createSeriesPattern18(this, 'first_p2tr_addr_index'),
            bytes: createSeriesPattern30(this, 'p2tr_bytes'),
          },
          p2a: {
            firstIndex: createSeriesPattern18(this, 'first_p2a_addr_index'),
            bytes: createSeriesPattern24(this, 'p2a_bytes'),
          },
        },
        indexes: {
          p2a: createSeriesPattern24(this, 'any_addr_index'),
          p2pk33: createSeriesPattern26(this, 'any_addr_index'),
          p2pk65: createSeriesPattern27(this, 'any_addr_index'),
          p2pkh: createSeriesPattern28(this, 'any_addr_index'),
          p2sh: createSeriesPattern29(this, 'any_addr_index'),
          p2tr: createSeriesPattern30(this, 'any_addr_index'),
          p2wpkh: createSeriesPattern31(this, 'any_addr_index'),
          p2wsh: createSeriesPattern32(this, 'any_addr_index'),
          funded: createSeriesPattern34(this, 'funded_addr_index'),
          empty: createSeriesPattern35(this, 'empty_addr_index'),
        },
        data: {
          funded: createSeriesPattern34(this, 'funded_addr_data'),
          empty: createSeriesPattern35(this, 'empty_addr_data'),
        },
        funded: createAllP2aP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshPattern4(this, 'addr_count'),
        empty: createAllP2aP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshPattern4(this, 'empty_addr_count'),
        activity: {
          all: {
            reactivated: create_1m1w1y24hBlockPattern(this, 'reactivated_addrs'),
            sending: create_1m1w1y24hBlockPattern(this, 'sending_addrs'),
            receiving: create_1m1w1y24hBlockPattern(this, 'receiving_addrs'),
            bidirectional: create_1m1w1y24hBlockPattern(this, 'bidirectional_addrs'),
            active: create_1m1w1y24hBlockPattern(this, 'active_addrs'),
          },
          p2pk65: createActiveBidirectionalReactivatedReceivingSendingPattern(this, 'p2pk65'),
          p2pk33: createActiveBidirectionalReactivatedReceivingSendingPattern(this, 'p2pk33'),
          p2pkh: createActiveBidirectionalReactivatedReceivingSendingPattern(this, 'p2pkh'),
          p2sh: createActiveBidirectionalReactivatedReceivingSendingPattern(this, 'p2sh'),
          p2wpkh: createActiveBidirectionalReactivatedReceivingSendingPattern(this, 'p2wpkh'),
          p2wsh: createActiveBidirectionalReactivatedReceivingSendingPattern(this, 'p2wsh'),
          p2tr: createActiveBidirectionalReactivatedReceivingSendingPattern(this, 'p2tr'),
          p2a: createActiveBidirectionalReactivatedReceivingSendingPattern(this, 'p2a'),
        },
        total: createAllP2aP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshPattern4(this, 'total_addr_count'),
        new: createAllP2aP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshPattern6(this, 'new_addr_count'),
        reused: {
          count: createFundedTotalPattern(this, 'reused_addr_count'),
          events: {
            outputToReusedAddrCount: createAllP2aP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshPattern6(this, 'output_to_reused_addr_count'),
            outputToReusedAddrShare: createAllP2aP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshPattern7(this, 'output_to_reused_addr_share'),
            spendableOutputToReusedAddrShare: create_1m1w1y24hBpsPercentRatioPattern(this, 'spendable_output_to_reused_addr_share'),
            inputFromReusedAddrCount: createAllP2aP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshPattern6(this, 'input_from_reused_addr_count'),
            inputFromReusedAddrShare: createAllP2aP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshPattern7(this, 'input_from_reused_addr_share'),
            activeReusedAddrCount: create_1m1w1y24hBlockPattern(this, 'active_reused_addr_count'),
            activeReusedAddrShare: create_1m1w1y24hBlockPattern2(this, 'active_reused_addr_share'),
          },
          supply: {
            all: createBtcCentsSatsUsdPattern(this, 'reused_addr_supply'),
            p2pk65: createBtcCentsSatsUsdPattern(this, 'p2pk65_reused_addr_supply'),
            p2pk33: createBtcCentsSatsUsdPattern(this, 'p2pk33_reused_addr_supply'),
            p2pkh: createBtcCentsSatsUsdPattern(this, 'p2pkh_reused_addr_supply'),
            p2sh: createBtcCentsSatsUsdPattern(this, 'p2sh_reused_addr_supply'),
            p2wpkh: createBtcCentsSatsUsdPattern(this, 'p2wpkh_reused_addr_supply'),
            p2wsh: createBtcCentsSatsUsdPattern(this, 'p2wsh_reused_addr_supply'),
            p2tr: createBtcCentsSatsUsdPattern(this, 'p2tr_reused_addr_supply'),
            p2a: createBtcCentsSatsUsdPattern(this, 'p2a_reused_addr_supply'),
            share: createAllP2aP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshPattern5(this, 'reused_addr_supply_share'),
          },
        },
        respent: {
          count: createFundedTotalPattern(this, 'respent_addr_count'),
          events: {
            outputToReusedAddrCount: createAllP2aP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshPattern6(this, 'output_to_respent_addr_count'),
            outputToReusedAddrShare: createAllP2aP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshPattern7(this, 'output_to_respent_addr_share'),
            spendableOutputToReusedAddrShare: create_1m1w1y24hBpsPercentRatioPattern(this, 'spendable_output_to_respent_addr_share'),
            inputFromReusedAddrCount: createAllP2aP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshPattern6(this, 'input_from_respent_addr_count'),
            inputFromReusedAddrShare: createAllP2aP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshPattern7(this, 'input_from_respent_addr_share'),
            activeReusedAddrCount: create_1m1w1y24hBlockPattern(this, 'active_respent_addr_count'),
            activeReusedAddrShare: create_1m1w1y24hBlockPattern2(this, 'active_respent_addr_share'),
          },
          supply: {
            all: createBtcCentsSatsUsdPattern(this, 'respent_addr_supply'),
            p2pk65: createBtcCentsSatsUsdPattern(this, 'p2pk65_respent_addr_supply'),
            p2pk33: createBtcCentsSatsUsdPattern(this, 'p2pk33_respent_addr_supply'),
            p2pkh: createBtcCentsSatsUsdPattern(this, 'p2pkh_respent_addr_supply'),
            p2sh: createBtcCentsSatsUsdPattern(this, 'p2sh_respent_addr_supply'),
            p2wpkh: createBtcCentsSatsUsdPattern(this, 'p2wpkh_respent_addr_supply'),
            p2wsh: createBtcCentsSatsUsdPattern(this, 'p2wsh_respent_addr_supply'),
            p2tr: createBtcCentsSatsUsdPattern(this, 'p2tr_respent_addr_supply'),
            p2a: createBtcCentsSatsUsdPattern(this, 'p2a_respent_addr_supply'),
            share: createAllP2aP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshPattern5(this, 'respent_addr_supply_share'),
          },
        },
        exposed: {
          count: createFundedTotalPattern(this, 'exposed_addr_count'),
          supply: {
            all: createBtcCentsSatsUsdPattern(this, 'exposed_addr_supply'),
            p2pk65: createBtcCentsSatsUsdPattern(this, 'p2pk65_exposed_addr_supply'),
            p2pk33: createBtcCentsSatsUsdPattern(this, 'p2pk33_exposed_addr_supply'),
            p2pkh: createBtcCentsSatsUsdPattern(this, 'p2pkh_exposed_addr_supply'),
            p2sh: createBtcCentsSatsUsdPattern(this, 'p2sh_exposed_addr_supply'),
            p2wpkh: createBtcCentsSatsUsdPattern(this, 'p2wpkh_exposed_addr_supply'),
            p2wsh: createBtcCentsSatsUsdPattern(this, 'p2wsh_exposed_addr_supply'),
            p2tr: createBtcCentsSatsUsdPattern(this, 'p2tr_exposed_addr_supply'),
            p2a: createBtcCentsSatsUsdPattern(this, 'p2a_exposed_addr_supply'),
            share: createAllP2aP2pk33P2pk65P2pkhP2shP2trP2wpkhP2wshPattern5(this, 'exposed_addr_supply_share'),
          },
        },
        delta: {
          all: createAbsoluteRatePattern(this, 'addr_count'),
          p2pk65: createAbsoluteRatePattern(this, 'p2pk65_addr_count'),
          p2pk33: createAbsoluteRatePattern(this, 'p2pk33_addr_count'),
          p2pkh: createAbsoluteRatePattern(this, 'p2pkh_addr_count'),
          p2sh: createAbsoluteRatePattern(this, 'p2sh_addr_count'),
          p2wpkh: createAbsoluteRatePattern(this, 'p2wpkh_addr_count'),
          p2wsh: createAbsoluteRatePattern(this, 'p2wsh_addr_count'),
          p2tr: createAbsoluteRatePattern(this, 'p2tr_addr_count'),
          p2a: createAbsoluteRatePattern(this, 'p2a_addr_count'),
        },
        avgAmount: {
          all: createAddrUtxoPattern(this, 'avg'),
          p2pk65: createAddrUtxoPattern(this, 'p2pk65_avg'),
          p2pk33: createAddrUtxoPattern(this, 'p2pk33_avg'),
          p2pkh: createAddrUtxoPattern(this, 'p2pkh_avg'),
          p2sh: createAddrUtxoPattern(this, 'p2sh_avg'),
          p2wpkh: createAddrUtxoPattern(this, 'p2wpkh_avg'),
          p2wsh: createAddrUtxoPattern(this, 'p2wsh_avg'),
          p2tr: createAddrUtxoPattern(this, 'p2tr_avg'),
          p2a: createAddrUtxoPattern(this, 'p2a_avg'),
        },
      },
      scripts: {
        raw: {
          empty: {
            firstIndex: createSeriesPattern18(this, 'first_empty_output_index'),
            toTxIndex: createSeriesPattern22(this, 'tx_index'),
          },
          opReturn: {
            firstIndex: createSeriesPattern18(this, 'first_op_return_index'),
            toTxIndex: createSeriesPattern23(this, 'tx_index'),
          },
          p2ms: {
            firstIndex: createSeriesPattern18(this, 'first_p2ms_output_index'),
            toTxIndex: createSeriesPattern25(this, 'tx_index'),
          },
          unknown: {
            firstIndex: createSeriesPattern18(this, 'first_unknown_output_index'),
            toTxIndex: createSeriesPattern33(this, 'tx_index'),
          },
        },
      },
      mining: {
        rewards: {
          coinbase: createAverageBlockCumulativeSumPattern3(this, 'coinbase'),
          subsidy: {
            block: createBtcCentsSatsUsdPattern3(this, 'subsidy'),
            cumulative: createBtcCentsSatsUsdPattern(this, 'subsidy_cumulative'),
            sum: create_1m1w1y24hPattern4(this, 'subsidy_sum'),
            average: create_1m1w1y24hPattern3(this, 'subsidy_average'),
            dominance: create_1m1w1y24hBpsPercentRatioPattern(this, 'subsidy_dominance'),
          },
          fees: {
            block: createBtcCentsSatsUsdPattern3(this, 'fees'),
            cumulative: createBtcCentsSatsUsdPattern(this, 'fees_cumulative'),
            sum: create_1m1w1y24hPattern4(this, 'fees_sum'),
            average: create_1m1w1y24hPattern3(this, 'fees_average'),
            min: create_1m1w1y24hPattern4(this, 'fees_min'),
            max: create_1m1w1y24hPattern4(this, 'fees_max'),
            pct10: create_1m1w1y24hPattern4(this, 'fees_pct10'),
            pct25: create_1m1w1y24hPattern4(this, 'fees_pct25'),
            median: create_1m1w1y24hPattern4(this, 'fees_median'),
            pct75: create_1m1w1y24hPattern4(this, 'fees_pct75'),
            pct90: create_1m1w1y24hPattern4(this, 'fees_pct90'),
            dominance: create_1m1w1y24hBpsPercentRatioPattern(this, 'fee_dominance'),
            toSubsidyRatio: {
              _24h: createBpsRatioPattern2(this, 'fee_to_subsidy_ratio_24h'),
              _1w: createBpsRatioPattern2(this, 'fee_to_subsidy_ratio_1w'),
              _1m: createBpsRatioPattern2(this, 'fee_to_subsidy_ratio_1m'),
              _1y: createBpsRatioPattern2(this, 'fee_to_subsidy_ratio_1y'),
            },
          },
          outputVolume: createSeriesPattern18(this, 'output_volume'),
          unclaimed: createBlockCumulativePattern(this, 'unclaimed_rewards'),
        },
        hashrate: {
          rate: {
            base: createSeriesPattern1(this, 'hash_rate'),
            sma: {
              _1w: createSeriesPattern1(this, 'hash_rate_sma_1w'),
              _1m: createSeriesPattern1(this, 'hash_rate_sma_1m'),
              _2m: createSeriesPattern1(this, 'hash_rate_sma_2m'),
              _1y: createSeriesPattern1(this, 'hash_rate_sma_1y'),
            },
            ath: createSeriesPattern1(this, 'hash_rate_ath'),
            drawdown: createBpsPercentRatioPattern5(this, 'hash_rate_drawdown'),
          },
          price: createPhsReboundThsPattern(this, 'hash_price'),
          value: createPhsReboundThsPattern(this, 'hash_value'),
        },
      },
      cointime: {
        activity: {
          coinblocksCreated: createAverageBlockCumulativeSumPattern(this, 'coinblocks_created'),
          coinblocksStored: createAverageBlockCumulativeSumPattern(this, 'coinblocks_stored'),
          liveliness: createSeriesPattern1(this, 'liveliness'),
          vaultedness: createSeriesPattern1(this, 'vaultedness'),
          ratio: createSeriesPattern1(this, 'activity_to_vaultedness'),
          coinblocksDestroyed: createAverageBlockCumulativeSumPattern(this, 'coinblocks_destroyed'),
        },
        supply: {
          vaulted: createBtcCentsSatsUsdPattern(this, 'vaulted_supply'),
          active: createBtcCentsSatsUsdPattern(this, 'active_supply'),
        },
        value: {
          destroyed: createAverageBlockCumulativeSumPattern(this, 'cointime_value_destroyed'),
          created: createAverageBlockCumulativeSumPattern(this, 'cointime_value_created'),
          stored: createAverageBlockCumulativeSumPattern(this, 'cointime_value_stored'),
          vocdd: createAverageBlockCumulativeSumPattern(this, 'vocdd'),
        },
        cap: {
          thermo: createCentsUsdPattern3(this, 'thermo_cap'),
          investor: createCentsUsdPattern3(this, 'investor_cap'),
          vaulted: createCentsUsdPattern3(this, 'vaulted_cap'),
          active: createCentsUsdPattern3(this, 'active_cap'),
          cointime: createCentsUsdPattern3(this, 'cointime_cap'),
          aviv: createBpsRatioPattern2(this, 'aviv_ratio'),
        },
        prices: {
          vaulted: createBpsCentsPercentilesRatioSatsUsdPattern(this, 'vaulted_price'),
          active: createBpsCentsPercentilesRatioSatsUsdPattern(this, 'active_price'),
          trueMarketMean: createBpsCentsPercentilesRatioSatsUsdPattern(this, 'true_market_mean'),
          cointime: createBpsCentsPercentilesRatioSatsUsdPattern(this, 'cointime_price'),
        },
        adjusted: {
          inflationRate: createBpsPercentRatioPattern(this, 'cointime_adj_inflation_rate'),
          txVelocityNative: createSeriesPattern1(this, 'cointime_adj_tx_velocity_btc'),
          txVelocityFiat: createSeriesPattern1(this, 'cointime_adj_tx_velocity_usd'),
        },
        reserveRisk: {
          value: createSeriesPattern1(this, 'reserve_risk'),
          vocddMedian1y: createSeriesPattern18(this, 'vocdd_median_1y'),
          hodlBank: createSeriesPattern18(this, 'hodl_bank'),
        },
      },
      constants: {
        _0: createSeriesPattern1(this, 'constant_0'),
        _1: createSeriesPattern1(this, 'constant_1'),
        _2: createSeriesPattern1(this, 'constant_2'),
        _3: createSeriesPattern1(this, 'constant_3'),
        _4: createSeriesPattern1(this, 'constant_4'),
        _20: createSeriesPattern1(this, 'constant_20'),
        _30: createSeriesPattern1(this, 'constant_30'),
        _382: createSeriesPattern1(this, 'constant_38_2'),
        _50: createSeriesPattern1(this, 'constant_50'),
        _618: createSeriesPattern1(this, 'constant_61_8'),
        _70: createSeriesPattern1(this, 'constant_70'),
        _80: createSeriesPattern1(this, 'constant_80'),
        _100: createSeriesPattern1(this, 'constant_100'),
        _600: createSeriesPattern1(this, 'constant_600'),
        minus1: createSeriesPattern1(this, 'constant_minus_1'),
        minus2: createSeriesPattern1(this, 'constant_minus_2'),
        minus3: createSeriesPattern1(this, 'constant_minus_3'),
        minus4: createSeriesPattern1(this, 'constant_minus_4'),
      },
      indexes: {
        addr: {
          p2pk33: {
            identity: createSeriesPattern26(this, 'p2pk33_addr_index'),
            addr: createSeriesPattern26(this, 'p2pk33_addr'),
          },
          p2pk65: {
            identity: createSeriesPattern27(this, 'p2pk65_addr_index'),
            addr: createSeriesPattern27(this, 'p2pk65_addr'),
          },
          p2pkh: {
            identity: createSeriesPattern28(this, 'p2pkh_addr_index'),
            addr: createSeriesPattern28(this, 'p2pkh_addr'),
          },
          p2sh: {
            identity: createSeriesPattern29(this, 'p2sh_addr_index'),
            addr: createSeriesPattern29(this, 'p2sh_addr'),
          },
          p2tr: {
            identity: createSeriesPattern30(this, 'p2tr_addr_index'),
            addr: createSeriesPattern30(this, 'p2tr_addr'),
          },
          p2wpkh: {
            identity: createSeriesPattern31(this, 'p2wpkh_addr_index'),
            addr: createSeriesPattern31(this, 'p2wpkh_addr'),
          },
          p2wsh: {
            identity: createSeriesPattern32(this, 'p2wsh_addr_index'),
            addr: createSeriesPattern32(this, 'p2wsh_addr'),
          },
          p2a: {
            identity: createSeriesPattern24(this, 'p2a_addr_index'),
            addr: createSeriesPattern24(this, 'p2a_addr'),
          },
          p2ms: {
            identity: createSeriesPattern25(this, 'p2ms_output_index'),
          },
          empty: {
            identity: createSeriesPattern22(this, 'empty_output_index'),
          },
          unknown: {
            identity: createSeriesPattern33(this, 'unknown_output_index'),
          },
          opReturn: {
            identity: createSeriesPattern23(this, 'op_return_index'),
          },
        },
        height: {
          minute10: createSeriesPattern18(this, 'minute10'),
          minute30: createSeriesPattern18(this, 'minute30'),
          hour1: createSeriesPattern18(this, 'hour1'),
          hour4: createSeriesPattern18(this, 'hour4'),
          hour12: createSeriesPattern18(this, 'hour12'),
          day1: createSeriesPattern18(this, 'day1'),
          day3: createSeriesPattern18(this, 'day3'),
          epoch: createSeriesPattern18(this, 'epoch'),
          halving: createSeriesPattern18(this, 'halving'),
          week1: createSeriesPattern18(this, 'week1'),
          month1: createSeriesPattern18(this, 'month1'),
          month3: createSeriesPattern18(this, 'month3'),
          month6: createSeriesPattern18(this, 'month6'),
          year1: createSeriesPattern18(this, 'year1'),
          year10: createSeriesPattern18(this, 'year10'),
          txIndexCount: createSeriesPattern18(this, 'tx_index_count'),
        },
        epoch: {
          firstHeight: createSeriesPattern17(this, 'first_height'),
        },
        halving: {
          firstHeight: createSeriesPattern16(this, 'first_height'),
        },
        minute10: {
          firstHeight: createSeriesPattern3(this, 'first_height'),
        },
        minute30: {
          firstHeight: createSeriesPattern4(this, 'first_height'),
        },
        hour1: {
          firstHeight: createSeriesPattern5(this, 'first_height'),
        },
        hour4: {
          firstHeight: createSeriesPattern6(this, 'first_height'),
        },
        hour12: {
          firstHeight: createSeriesPattern7(this, 'first_height'),
        },
        day1: {
          date: createSeriesPattern8(this, 'date'),
          firstHeight: createSeriesPattern8(this, 'first_height'),
        },
        day3: {
          date: createSeriesPattern9(this, 'date'),
          firstHeight: createSeriesPattern9(this, 'first_height'),
        },
        week1: {
          date: createSeriesPattern10(this, 'date'),
          firstHeight: createSeriesPattern10(this, 'first_height'),
        },
        month1: {
          date: createSeriesPattern11(this, 'date'),
          firstHeight: createSeriesPattern11(this, 'first_height'),
        },
        month3: {
          date: createSeriesPattern12(this, 'date'),
          firstHeight: createSeriesPattern12(this, 'first_height'),
        },
        month6: {
          date: createSeriesPattern13(this, 'date'),
          firstHeight: createSeriesPattern13(this, 'first_height'),
        },
        year1: {
          date: createSeriesPattern14(this, 'date'),
          firstHeight: createSeriesPattern14(this, 'first_height'),
        },
        year10: {
          date: createSeriesPattern15(this, 'date'),
          firstHeight: createSeriesPattern15(this, 'first_height'),
        },
        txIndex: {
          identity: createSeriesPattern19(this, 'tx_index'),
          inputCount: createSeriesPattern19(this, 'input_count'),
          outputCount: createSeriesPattern19(this, 'output_count'),
        },
        txinIndex: {
          identity: createSeriesPattern20(this, 'txin_index'),
        },
        txoutIndex: {
          identity: createSeriesPattern21(this, 'txout_index'),
        },
        timestamp: {
          monotonic: createSeriesPattern18(this, 'timestamp_monotonic'),
          resolutions: createSeriesPattern2(this, 'timestamp'),
        },
      },
      indicators: {
        puellMultiple: createBpsRatioPattern2(this, 'puell_multiple'),
        nvt: createBpsRatioPattern2(this, 'nvt'),
        gini: createBpsPercentRatioPattern2(this, 'gini'),
        rhodlRatio: createBpsRatioPattern2(this, 'rhodl_ratio'),
        thermoCapMultiple: createBpsRatioPattern2(this, 'thermo_cap_multiple'),
        coindaysDestroyedSupplyAdj: createSeriesPattern1(this, 'coindays_destroyed_supply_adj'),
        coinyearsDestroyedSupplyAdj: createSeriesPattern1(this, 'coinyears_destroyed_supply_adj'),
        dormancy: {
          supplyAdj: createSeriesPattern1(this, 'dormancy_supply_adj'),
          flow: createSeriesPattern1(this, 'dormancy_flow'),
        },
        stockToFlow: createSeriesPattern1(this, 'stock_to_flow'),
        sellerExhaustion: createSeriesPattern1(this, 'seller_exhaustion'),
        rarityMeter: {
          full: createIndexPct0Pct1Pct2Pct5Pct95Pct98Pct99ScorePattern(this, 'rarity_meter'),
          local: createIndexPct0Pct1Pct2Pct5Pct95Pct98Pct99ScorePattern(this, 'local_rarity_meter'),
          cycle: createIndexPct0Pct1Pct2Pct5Pct95Pct98Pct99ScorePattern(this, 'cycle_rarity_meter'),
        },
      },
      investing: {
        satsPerDay: createSeriesPattern18(this, 'dca_sats_per_day'),
        period: {
          dcaStack: create_10y1m1w1y2y3m3y4y5y6m6y8yPattern3(this, 'dca_stack'),
          dcaCostBasis: {
            _1w: createCentsSatsUsdPattern(this, 'dca_cost_basis_1w'),
            _1m: createCentsSatsUsdPattern(this, 'dca_cost_basis_1m'),
            _3m: createCentsSatsUsdPattern(this, 'dca_cost_basis_3m'),
            _6m: createCentsSatsUsdPattern(this, 'dca_cost_basis_6m'),
            _1y: createCentsSatsUsdPattern(this, 'dca_cost_basis_1y'),
            _2y: createCentsSatsUsdPattern(this, 'dca_cost_basis_2y'),
            _3y: createCentsSatsUsdPattern(this, 'dca_cost_basis_3y'),
            _4y: createCentsSatsUsdPattern(this, 'dca_cost_basis_4y'),
            _5y: createCentsSatsUsdPattern(this, 'dca_cost_basis_5y'),
            _6y: createCentsSatsUsdPattern(this, 'dca_cost_basis_6y'),
            _8y: createCentsSatsUsdPattern(this, 'dca_cost_basis_8y'),
            _10y: createCentsSatsUsdPattern(this, 'dca_cost_basis_10y'),
          },
          dcaReturn: create_10y1m1w1y2y3m3y4y5y6m6y8yPattern2(this, 'dca_return'),
          dcaCagr: create_10y2y3y4y5y6y8yPattern(this, 'dca_cagr'),
          lumpSumStack: create_10y1m1w1y2y3m3y4y5y6m6y8yPattern3(this, 'lump_sum_stack'),
          lumpSumReturn: create_10y1m1w1y2y3m3y4y5y6m6y8yPattern2(this, 'lump_sum_return'),
        },
        class: {
          dcaStack: {
            from2015: createBtcCentsSatsUsdPattern(this, 'dca_stack_from_2015'),
            from2016: createBtcCentsSatsUsdPattern(this, 'dca_stack_from_2016'),
            from2017: createBtcCentsSatsUsdPattern(this, 'dca_stack_from_2017'),
            from2018: createBtcCentsSatsUsdPattern(this, 'dca_stack_from_2018'),
            from2019: createBtcCentsSatsUsdPattern(this, 'dca_stack_from_2019'),
            from2020: createBtcCentsSatsUsdPattern(this, 'dca_stack_from_2020'),
            from2021: createBtcCentsSatsUsdPattern(this, 'dca_stack_from_2021'),
            from2022: createBtcCentsSatsUsdPattern(this, 'dca_stack_from_2022'),
            from2023: createBtcCentsSatsUsdPattern(this, 'dca_stack_from_2023'),
            from2024: createBtcCentsSatsUsdPattern(this, 'dca_stack_from_2024'),
            from2025: createBtcCentsSatsUsdPattern(this, 'dca_stack_from_2025'),
            from2026: createBtcCentsSatsUsdPattern(this, 'dca_stack_from_2026'),
          },
          dcaCostBasis: {
            from2015: createCentsSatsUsdPattern(this, 'dca_cost_basis_from_2015'),
            from2016: createCentsSatsUsdPattern(this, 'dca_cost_basis_from_2016'),
            from2017: createCentsSatsUsdPattern(this, 'dca_cost_basis_from_2017'),
            from2018: createCentsSatsUsdPattern(this, 'dca_cost_basis_from_2018'),
            from2019: createCentsSatsUsdPattern(this, 'dca_cost_basis_from_2019'),
            from2020: createCentsSatsUsdPattern(this, 'dca_cost_basis_from_2020'),
            from2021: createCentsSatsUsdPattern(this, 'dca_cost_basis_from_2021'),
            from2022: createCentsSatsUsdPattern(this, 'dca_cost_basis_from_2022'),
            from2023: createCentsSatsUsdPattern(this, 'dca_cost_basis_from_2023'),
            from2024: createCentsSatsUsdPattern(this, 'dca_cost_basis_from_2024'),
            from2025: createCentsSatsUsdPattern(this, 'dca_cost_basis_from_2025'),
            from2026: createCentsSatsUsdPattern(this, 'dca_cost_basis_from_2026'),
          },
          dcaReturn: {
            from2015: createBpsPercentRatioPattern(this, 'dca_return_from_2015'),
            from2016: createBpsPercentRatioPattern(this, 'dca_return_from_2016'),
            from2017: createBpsPercentRatioPattern(this, 'dca_return_from_2017'),
            from2018: createBpsPercentRatioPattern(this, 'dca_return_from_2018'),
            from2019: createBpsPercentRatioPattern(this, 'dca_return_from_2019'),
            from2020: createBpsPercentRatioPattern(this, 'dca_return_from_2020'),
            from2021: createBpsPercentRatioPattern(this, 'dca_return_from_2021'),
            from2022: createBpsPercentRatioPattern(this, 'dca_return_from_2022'),
            from2023: createBpsPercentRatioPattern(this, 'dca_return_from_2023'),
            from2024: createBpsPercentRatioPattern(this, 'dca_return_from_2024'),
            from2025: createBpsPercentRatioPattern(this, 'dca_return_from_2025'),
            from2026: createBpsPercentRatioPattern(this, 'dca_return_from_2026'),
          },
        },
      },
      market: {
        ath: {
          high: createCentsSatsUsdPattern(this, 'price_ath'),
          drawdown: createBpsPercentRatioPattern5(this, 'price_drawdown'),
          daysSince: createSeriesPattern1(this, 'days_since_price_ath'),
          yearsSince: createSeriesPattern1(this, 'years_since_price_ath'),
          maxDaysBetween: createSeriesPattern1(this, 'max_days_between_price_ath'),
          maxYearsBetween: createSeriesPattern1(this, 'max_years_between_price_ath'),
        },
        lookback: {
          _24h: createCentsSatsUsdPattern(this, 'price_past_24h'),
          _1w: createCentsSatsUsdPattern(this, 'price_past_1w'),
          _1m: createCentsSatsUsdPattern(this, 'price_past_1m'),
          _3m: createCentsSatsUsdPattern(this, 'price_past_3m'),
          _6m: createCentsSatsUsdPattern(this, 'price_past_6m'),
          _1y: createCentsSatsUsdPattern(this, 'price_past_1y'),
          _2y: createCentsSatsUsdPattern(this, 'price_past_2y'),
          _3y: createCentsSatsUsdPattern(this, 'price_past_3y'),
          _4y: createCentsSatsUsdPattern(this, 'price_past_4y'),
          _5y: createCentsSatsUsdPattern(this, 'price_past_5y'),
          _6y: createCentsSatsUsdPattern(this, 'price_past_6y'),
          _8y: createCentsSatsUsdPattern(this, 'price_past_8y'),
          _10y: createCentsSatsUsdPattern(this, 'price_past_10y'),
        },
        returns: {
          periods: {
            _24h: createBpsPercentRatioPattern(this, 'price_return_24h'),
            _1w: createBpsPercentRatioPattern(this, 'price_return_1w'),
            _1m: createBpsPercentRatioPattern(this, 'price_return_1m'),
            _3m: createBpsPercentRatioPattern(this, 'price_return_3m'),
            _6m: createBpsPercentRatioPattern(this, 'price_return_6m'),
            _1y: createBpsPercentRatioPattern(this, 'price_return_1y'),
            _2y: createBpsPercentRatioPattern(this, 'price_return_2y'),
            _3y: createBpsPercentRatioPattern(this, 'price_return_3y'),
            _4y: createBpsPercentRatioPattern(this, 'price_return_4y'),
            _5y: createBpsPercentRatioPattern(this, 'price_return_5y'),
            _6y: createBpsPercentRatioPattern(this, 'price_return_6y'),
            _8y: createBpsPercentRatioPattern(this, 'price_return_8y'),
            _10y: createBpsPercentRatioPattern(this, 'price_return_10y'),
          },
          cagr: create_10y2y3y4y5y6y8yPattern(this, 'price_cagr'),
          sd24h: {
            _24h: {
              sma: createSeriesPattern1(this, 'price_return_24h_sma_24h'),
              sd: createSeriesPattern1(this, 'price_return_24h_sd_24h'),
            },
            _1w: {
              sma: createSeriesPattern1(this, 'price_return_24h_sma_1w'),
              sd: createSeriesPattern1(this, 'price_return_24h_sd_1w'),
            },
            _1m: {
              sma: createSeriesPattern1(this, 'price_return_24h_sma_1m'),
              sd: createSeriesPattern1(this, 'price_return_24h_sd_1m'),
            },
            _1y: {
              sma: createSeriesPattern1(this, 'price_return_24h_sma_1y'),
              sd: createSeriesPattern1(this, 'price_return_24h_sd_1y'),
            },
          },
        },
        volatility: create_1m1w1y24hPattern(this, 'price_volatility'),
        range: {
          min: create_1m1w1y2wPattern(this, 'price_min'),
          max: create_1m1w1y2wPattern(this, 'price_max'),
          trueRange: createSeriesPattern1(this, 'price_true_range'),
          trueRangeSum2w: createSeriesPattern1(this, 'price_true_range_sum_2w'),
          choppinessIndex2w: createBpsPercentRatioPattern2(this, 'price_choppiness_index_2w'),
        },
        movingAverage: {
          sma: {
            _1w: createBpsCentsRatioSatsUsdPattern(this, 'price_sma_1w'),
            _8d: createBpsCentsRatioSatsUsdPattern(this, 'price_sma_8d'),
            _13d: createBpsCentsRatioSatsUsdPattern(this, 'price_sma_13d'),
            _21d: createBpsCentsRatioSatsUsdPattern(this, 'price_sma_21d'),
            _1m: createBpsCentsRatioSatsUsdPattern(this, 'price_sma_1m'),
            _34d: createBpsCentsRatioSatsUsdPattern(this, 'price_sma_34d'),
            _55d: createBpsCentsRatioSatsUsdPattern(this, 'price_sma_55d'),
            _89d: createBpsCentsRatioSatsUsdPattern(this, 'price_sma_89d'),
            _111d: createBpsCentsRatioSatsUsdPattern(this, 'price_sma_111d'),
            _144d: createBpsCentsRatioSatsUsdPattern(this, 'price_sma_144d'),
            _200d: {
              usd: createSeriesPattern1(this, 'price_sma_200d'),
              cents: createSeriesPattern1(this, 'price_sma_200d_cents'),
              sats: createSeriesPattern1(this, 'price_sma_200d_sats'),
              bps: createSeriesPattern1(this, 'price_sma_200d_ratio_bps'),
              ratio: createSeriesPattern1(this, 'price_sma_200d_ratio'),
              x24: createCentsSatsUsdPattern(this, 'price_sma_200d_x2_4'),
              x08: createCentsSatsUsdPattern(this, 'price_sma_200d_x0_8'),
            },
            _350d: {
              usd: createSeriesPattern1(this, 'price_sma_350d'),
              cents: createSeriesPattern1(this, 'price_sma_350d_cents'),
              sats: createSeriesPattern1(this, 'price_sma_350d_sats'),
              bps: createSeriesPattern1(this, 'price_sma_350d_ratio_bps'),
              ratio: createSeriesPattern1(this, 'price_sma_350d_ratio'),
              x2: createCentsSatsUsdPattern(this, 'price_sma_350d_x2'),
            },
            _1y: createBpsCentsRatioSatsUsdPattern(this, 'price_sma_1y'),
            _2y: createBpsCentsRatioSatsUsdPattern(this, 'price_sma_2y'),
            _200w: createBpsCentsRatioSatsUsdPattern(this, 'price_sma_200w'),
            _4y: createBpsCentsRatioSatsUsdPattern(this, 'price_sma_4y'),
          },
          ema: {
            _1w: createBpsCentsRatioSatsUsdPattern(this, 'price_ema_1w'),
            _8d: createBpsCentsRatioSatsUsdPattern(this, 'price_ema_8d'),
            _12d: createBpsCentsRatioSatsUsdPattern(this, 'price_ema_12d'),
            _13d: createBpsCentsRatioSatsUsdPattern(this, 'price_ema_13d'),
            _21d: createBpsCentsRatioSatsUsdPattern(this, 'price_ema_21d'),
            _26d: createBpsCentsRatioSatsUsdPattern(this, 'price_ema_26d'),
            _1m: createBpsCentsRatioSatsUsdPattern(this, 'price_ema_1m'),
            _34d: createBpsCentsRatioSatsUsdPattern(this, 'price_ema_34d'),
            _55d: createBpsCentsRatioSatsUsdPattern(this, 'price_ema_55d'),
            _89d: createBpsCentsRatioSatsUsdPattern(this, 'price_ema_89d'),
            _144d: createBpsCentsRatioSatsUsdPattern(this, 'price_ema_144d'),
            _200d: createBpsCentsRatioSatsUsdPattern(this, 'price_ema_200d'),
            _1y: createBpsCentsRatioSatsUsdPattern(this, 'price_ema_1y'),
            _2y: createBpsCentsRatioSatsUsdPattern(this, 'price_ema_2y'),
            _200w: createBpsCentsRatioSatsUsdPattern(this, 'price_ema_200w'),
            _4y: createBpsCentsRatioSatsUsdPattern(this, 'price_ema_4y'),
          },
        },
        technical: {
          rsi: {
            _24h: createRsiStochPattern(this, 'rsi', '24h'),
            _1w: createRsiStochPattern(this, 'rsi', '1w'),
            _1m: createRsiStochPattern(this, 'rsi', '1m'),
          },
          piCycle: createBpsRatioPattern2(this, 'pi_cycle'),
          macd: {
            _24h: {
              emaFast: createSeriesPattern1(this, 'macd_ema_fast_24h'),
              emaSlow: createSeriesPattern1(this, 'macd_ema_slow_24h'),
              line: createSeriesPattern1(this, 'macd_line_24h'),
              signal: createSeriesPattern1(this, 'macd_signal_24h'),
              histogram: createSeriesPattern1(this, 'macd_histogram_24h'),
            },
            _1w: {
              emaFast: createSeriesPattern1(this, 'macd_ema_fast_1w'),
              emaSlow: createSeriesPattern1(this, 'macd_ema_slow_1w'),
              line: createSeriesPattern1(this, 'macd_line_1w'),
              signal: createSeriesPattern1(this, 'macd_signal_1w'),
              histogram: createSeriesPattern1(this, 'macd_histogram_1w'),
            },
            _1m: {
              emaFast: createSeriesPattern1(this, 'macd_ema_fast_1m'),
              emaSlow: createSeriesPattern1(this, 'macd_ema_slow_1m'),
              line: createSeriesPattern1(this, 'macd_line_1m'),
              signal: createSeriesPattern1(this, 'macd_signal_1m'),
              histogram: createSeriesPattern1(this, 'macd_histogram_1m'),
            },
          },
        },
      },
      pools: {
        pool: createSeriesPattern18(this, 'pool'),
        major: {
          unknown: createBlocksDominanceRewardsPattern(this, 'unknown'),
          luxor: createBlocksDominanceRewardsPattern(this, 'luxor'),
          btccom: createBlocksDominanceRewardsPattern(this, 'btccom'),
          btctop: createBlocksDominanceRewardsPattern(this, 'btctop'),
          btcguild: createBlocksDominanceRewardsPattern(this, 'btcguild'),
          eligius: createBlocksDominanceRewardsPattern(this, 'eligius'),
          f2pool: createBlocksDominanceRewardsPattern(this, 'f2pool'),
          braiinspool: createBlocksDominanceRewardsPattern(this, 'braiinspool'),
          antpool: createBlocksDominanceRewardsPattern(this, 'antpool'),
          btcc: createBlocksDominanceRewardsPattern(this, 'btcc'),
          bwpool: createBlocksDominanceRewardsPattern(this, 'bwpool'),
          bitfury: createBlocksDominanceRewardsPattern(this, 'bitfury'),
          viabtc: createBlocksDominanceRewardsPattern(this, 'viabtc'),
          poolin: createBlocksDominanceRewardsPattern(this, 'poolin'),
          spiderpool: createBlocksDominanceRewardsPattern(this, 'spiderpool'),
          binancepool: createBlocksDominanceRewardsPattern(this, 'binancepool'),
          foundryusa: createBlocksDominanceRewardsPattern(this, 'foundryusa'),
          sbicrypto: createBlocksDominanceRewardsPattern(this, 'sbicrypto'),
          marapool: createBlocksDominanceRewardsPattern(this, 'marapool'),
          secpool: createBlocksDominanceRewardsPattern(this, 'secpool'),
          ocean: createBlocksDominanceRewardsPattern(this, 'ocean'),
          whitepool: createBlocksDominanceRewardsPattern(this, 'whitepool'),
        },
        minor: {
          blockfills: createBlocksDominancePattern(this, 'blockfills'),
          ultimuspool: createBlocksDominancePattern(this, 'ultimuspool'),
          terrapool: createBlocksDominancePattern(this, 'terrapool'),
          onethash: createBlocksDominancePattern(this, 'onethash'),
          bitfarms: createBlocksDominancePattern(this, 'bitfarms'),
          huobipool: createBlocksDominancePattern(this, 'huobipool'),
          wayicn: createBlocksDominancePattern(this, 'wayicn'),
          canoepool: createBlocksDominancePattern(this, 'canoepool'),
          bitcoincom: createBlocksDominancePattern(this, 'bitcoincom'),
          pool175btc: createBlocksDominancePattern(this, 'pool175btc'),
          gbminers: createBlocksDominancePattern(this, 'gbminers'),
          axbt: createBlocksDominancePattern(this, 'axbt'),
          asicminer: createBlocksDominancePattern(this, 'asicminer'),
          bitminter: createBlocksDominancePattern(this, 'bitminter'),
          bitcoinrussia: createBlocksDominancePattern(this, 'bitcoinrussia'),
          btcserv: createBlocksDominancePattern(this, 'btcserv'),
          simplecoinus: createBlocksDominancePattern(this, 'simplecoinus'),
          ozcoin: createBlocksDominancePattern(this, 'ozcoin'),
          eclipsemc: createBlocksDominancePattern(this, 'eclipsemc'),
          maxbtc: createBlocksDominancePattern(this, 'maxbtc'),
          triplemining: createBlocksDominancePattern(this, 'triplemining'),
          coinlab: createBlocksDominancePattern(this, 'coinlab'),
          pool50btc: createBlocksDominancePattern(this, 'pool50btc'),
          ghashio: createBlocksDominancePattern(this, 'ghashio'),
          stminingcorp: createBlocksDominancePattern(this, 'stminingcorp'),
          bitparking: createBlocksDominancePattern(this, 'bitparking'),
          mmpool: createBlocksDominancePattern(this, 'mmpool'),
          polmine: createBlocksDominancePattern(this, 'polmine'),
          kncminer: createBlocksDominancePattern(this, 'kncminer'),
          bitalo: createBlocksDominancePattern(this, 'bitalo'),
          hhtt: createBlocksDominancePattern(this, 'hhtt'),
          megabigpower: createBlocksDominancePattern(this, 'megabigpower'),
          mtred: createBlocksDominancePattern(this, 'mtred'),
          nmcbit: createBlocksDominancePattern(this, 'nmcbit'),
          yourbtcnet: createBlocksDominancePattern(this, 'yourbtcnet'),
          givemecoins: createBlocksDominancePattern(this, 'givemecoins'),
          multicoinco: createBlocksDominancePattern(this, 'multicoinco'),
          bcpoolio: createBlocksDominancePattern(this, 'bcpoolio'),
          cointerra: createBlocksDominancePattern(this, 'cointerra'),
          kanopool: createBlocksDominancePattern(this, 'kanopool'),
          solock: createBlocksDominancePattern(this, 'solock'),
          ckpool: createBlocksDominancePattern(this, 'ckpool'),
          nicehash: createBlocksDominancePattern(this, 'nicehash'),
          bitclub: createBlocksDominancePattern(this, 'bitclub'),
          bitcoinaffiliatenetwork: createBlocksDominancePattern(this, 'bitcoinaffiliatenetwork'),
          exxbw: createBlocksDominancePattern(this, 'exxbw'),
          bitsolo: createBlocksDominancePattern(this, 'bitsolo'),
          twentyoneinc: createBlocksDominancePattern(this, 'twentyoneinc'),
          digitalbtc: createBlocksDominancePattern(this, 'digitalbtc'),
          eightbaochi: createBlocksDominancePattern(this, 'eightbaochi'),
          mybtccoinpool: createBlocksDominancePattern(this, 'mybtccoinpool'),
          tbdice: createBlocksDominancePattern(this, 'tbdice'),
          hashpool: createBlocksDominancePattern(this, 'hashpool'),
          nexious: createBlocksDominancePattern(this, 'nexious'),
          bravomining: createBlocksDominancePattern(this, 'bravomining'),
          hotpool: createBlocksDominancePattern(this, 'hotpool'),
          okexpool: createBlocksDominancePattern(this, 'okexpool'),
          bcmonster: createBlocksDominancePattern(this, 'bcmonster'),
          onehash: createBlocksDominancePattern(this, 'onehash'),
          bixin: createBlocksDominancePattern(this, 'bixin'),
          tatmaspool: createBlocksDominancePattern(this, 'tatmaspool'),
          connectbtc: createBlocksDominancePattern(this, 'connectbtc'),
          batpool: createBlocksDominancePattern(this, 'batpool'),
          waterhole: createBlocksDominancePattern(this, 'waterhole'),
          dcexploration: createBlocksDominancePattern(this, 'dcexploration'),
          dcex: createBlocksDominancePattern(this, 'dcex'),
          btpool: createBlocksDominancePattern(this, 'btpool'),
          fiftyeightcoin: createBlocksDominancePattern(this, 'fiftyeightcoin'),
          bitcoinindia: createBlocksDominancePattern(this, 'bitcoinindia'),
          shawnp0wers: createBlocksDominancePattern(this, 'shawnp0wers'),
          phashio: createBlocksDominancePattern(this, 'phashio'),
          rigpool: createBlocksDominancePattern(this, 'rigpool'),
          haozhuzhu: createBlocksDominancePattern(this, 'haozhuzhu'),
          sevenpool: createBlocksDominancePattern(this, 'sevenpool'),
          miningkings: createBlocksDominancePattern(this, 'miningkings'),
          hashbx: createBlocksDominancePattern(this, 'hashbx'),
          dpool: createBlocksDominancePattern(this, 'dpool'),
          rawpool: createBlocksDominancePattern(this, 'rawpool'),
          haominer: createBlocksDominancePattern(this, 'haominer'),
          helix: createBlocksDominancePattern(this, 'helix'),
          bitcoinukraine: createBlocksDominancePattern(this, 'bitcoinukraine'),
          secretsuperstar: createBlocksDominancePattern(this, 'secretsuperstar'),
          tigerpoolnet: createBlocksDominancePattern(this, 'tigerpoolnet'),
          sigmapoolcom: createBlocksDominancePattern(this, 'sigmapoolcom'),
          okpooltop: createBlocksDominancePattern(this, 'okpooltop'),
          hummerpool: createBlocksDominancePattern(this, 'hummerpool'),
          tangpool: createBlocksDominancePattern(this, 'tangpool'),
          bytepool: createBlocksDominancePattern(this, 'bytepool'),
          novablock: createBlocksDominancePattern(this, 'novablock'),
          miningcity: createBlocksDominancePattern(this, 'miningcity'),
          minerium: createBlocksDominancePattern(this, 'minerium'),
          lubiancom: createBlocksDominancePattern(this, 'lubiancom'),
          okkong: createBlocksDominancePattern(this, 'okkong'),
          aaopool: createBlocksDominancePattern(this, 'aaopool'),
          emcdpool: createBlocksDominancePattern(this, 'emcdpool'),
          arkpool: createBlocksDominancePattern(this, 'arkpool'),
          purebtccom: createBlocksDominancePattern(this, 'purebtccom'),
          kucoinpool: createBlocksDominancePattern(this, 'kucoinpool'),
          entrustcharitypool: createBlocksDominancePattern(this, 'entrustcharitypool'),
          okminer: createBlocksDominancePattern(this, 'okminer'),
          titan: createBlocksDominancePattern(this, 'titan'),
          pegapool: createBlocksDominancePattern(this, 'pegapool'),
          btcnuggets: createBlocksDominancePattern(this, 'btcnuggets'),
          cloudhashing: createBlocksDominancePattern(this, 'cloudhashing'),
          digitalxmintsy: createBlocksDominancePattern(this, 'digitalxmintsy'),
          telco214: createBlocksDominancePattern(this, 'telco214'),
          btcpoolparty: createBlocksDominancePattern(this, 'btcpoolparty'),
          multipool: createBlocksDominancePattern(this, 'multipool'),
          transactioncoinmining: createBlocksDominancePattern(this, 'transactioncoinmining'),
          btcdig: createBlocksDominancePattern(this, 'btcdig'),
          trickysbtcpool: createBlocksDominancePattern(this, 'trickysbtcpool'),
          btcmp: createBlocksDominancePattern(this, 'btcmp'),
          eobot: createBlocksDominancePattern(this, 'eobot'),
          unomp: createBlocksDominancePattern(this, 'unomp'),
          patels: createBlocksDominancePattern(this, 'patels'),
          gogreenlight: createBlocksDominancePattern(this, 'gogreenlight'),
          bitcoinindiapool: createBlocksDominancePattern(this, 'bitcoinindiapool'),
          ekanembtc: createBlocksDominancePattern(this, 'ekanembtc'),
          canoe: createBlocksDominancePattern(this, 'canoe'),
          tiger: createBlocksDominancePattern(this, 'tiger'),
          onem1x: createBlocksDominancePattern(this, 'onem1x'),
          zulupool: createBlocksDominancePattern(this, 'zulupool'),
          wiz: createBlocksDominancePattern(this, 'wiz'),
          wk057: createBlocksDominancePattern(this, 'wk057'),
          futurebitapollosolo: createBlocksDominancePattern(this, 'futurebitapollosolo'),
          carbonnegative: createBlocksDominancePattern(this, 'carbonnegative'),
          portlandhodl: createBlocksDominancePattern(this, 'portlandhodl'),
          phoenix: createBlocksDominancePattern(this, 'phoenix'),
          neopool: createBlocksDominancePattern(this, 'neopool'),
          maxipool: createBlocksDominancePattern(this, 'maxipool'),
          bitfufupool: createBlocksDominancePattern(this, 'bitfufupool'),
          gdpool: createBlocksDominancePattern(this, 'gdpool'),
          miningdutch: createBlocksDominancePattern(this, 'miningdutch'),
          publicpool: createBlocksDominancePattern(this, 'publicpool'),
          miningsquared: createBlocksDominancePattern(this, 'miningsquared'),
          innopolistech: createBlocksDominancePattern(this, 'innopolistech'),
          btclab: createBlocksDominancePattern(this, 'btclab'),
          parasite: createBlocksDominancePattern(this, 'parasite'),
          redrockpool: createBlocksDominancePattern(this, 'redrockpool'),
          est3lar: createBlocksDominancePattern(this, 'est3lar'),
          braiinssolo: createBlocksDominancePattern(this, 'braiinssolo'),
          solopool: createBlocksDominancePattern(this, 'solopool'),
          noderunners: createBlocksDominancePattern(this, 'noderunners'),
        },
      },
      price: {
        split: {
          open: createCentsSatsUsdPattern3(this, 'price_open'),
          high: createCentsSatsUsdPattern3(this, 'price_high'),
          low: createCentsSatsUsdPattern3(this, 'price_low'),
          close: createCentsSatsUsdPattern3(this, 'price_close'),
        },
        ohlc: {
          usd: createSeriesPattern2(this, 'price_ohlc'),
          cents: createSeriesPattern2(this, 'price_ohlc_cents'),
          sats: createSeriesPattern2(this, 'price_ohlc_sats'),
        },
        spot: {
          usd: createSeriesPattern1(this, 'price'),
          cents: createSeriesPattern1(this, 'price_cents'),
          sats: createSeriesPattern1(this, 'price_sats'),
        },
      },
      supply: {
        state: createSeriesPattern18(this, 'supply_state'),
        circulating: createBtcCentsSatsUsdPattern(this, 'circulating_supply'),
        burned: createBlockCumulativePattern(this, 'unspendable_supply'),
        inflationRate: createBpsPercentRatioPattern(this, 'inflation_rate'),
        velocity: {
          native: createSeriesPattern1(this, 'velocity_btc'),
          fiat: createSeriesPattern1(this, 'velocity_usd'),
        },
        marketCap: createCentsDeltaUsdPattern(this, 'market_cap'),
        marketMinusRealizedCapGrowthRate: create_1m1w1y24hPattern(this, 'market_minus_realized_cap_growth_rate'),
        hodledOrLost: createBtcCentsSatsUsdPattern(this, 'hodled_or_lost_supply'),
      },
      cohorts: {
        utxo: {
          all: {
            supply: createDeltaDominanceHalfInTotalPattern2(this, 'supply'),
            outputs: {
              unspentCount: createBaseDeltaPattern(this, 'utxo_count'),
              spentCount: createAverageBlockCumulativeSumPattern2(this, 'spent_utxo_count'),
              spendingRate: createSeriesPattern1(this, 'spending_rate'),
            },
            activity: {
              transferVolume: createAverageBlockCumulativeInSumPattern(this, 'transfer_volume'),
              coindaysDestroyed: createAverageBlockCumulativeSumPattern(this, 'coindays_destroyed'),
              coinyearsDestroyed: createSeriesPattern1(this, 'coinyears_destroyed'),
              dormancy: create_1m1w1y24hPattern(this, 'dormancy'),
            },
            realized: {
              cap: createCentsDeltaToUsdPattern(this, 'realized_cap'),
              profit: createBlockCumulativeSumPattern(this, 'realized_profit'),
              loss: createBlockCumulativeNegativeSumPattern(this, 'realized_loss'),
              price: {
                usd: createSeriesPattern1(this, 'realized_price'),
                cents: createSeriesPattern1(this, 'realized_price_cents'),
                sats: createSeriesPattern1(this, 'realized_price_sats'),
                bps: createSeriesPattern1(this, 'realized_price_ratio_bps'),
                ratio: createSeriesPattern1(this, 'realized_price_ratio'),
                percentiles: createPct0Pct1Pct2Pct5Pct95Pct98Pct99Pattern(this, 'realized_price'),
                sma: create_1m1w1y2y4yAllPattern(this, 'realized_price_ratio_sma'),
                stdDev: {
                  all: {
                    sd: createSeriesPattern1(this, 'realized_price_ratio_sd'),
                    zscore: createSeriesPattern1(this, 'realized_price_ratio_zscore'),
                    _0sd: createCentsSatsUsdPattern(this, 'realized_price_0sd'),
                    p05sd: createPriceRatioPattern(this, 'realized_price', 'p0_5sd'),
                    p1sd: createPriceRatioPattern(this, 'realized_price', 'p1sd'),
                    p15sd: createPriceRatioPattern(this, 'realized_price', 'p1_5sd'),
                    p2sd: createPriceRatioPattern(this, 'realized_price', 'p2sd'),
                    p25sd: createPriceRatioPattern(this, 'realized_price', 'p2_5sd'),
                    p3sd: createPriceRatioPattern(this, 'realized_price', 'p3sd'),
                    m05sd: createPriceRatioPattern(this, 'realized_price', 'm0_5sd'),
                    m1sd: createPriceRatioPattern(this, 'realized_price', 'm1sd'),
                    m15sd: createPriceRatioPattern(this, 'realized_price', 'm1_5sd'),
                    m2sd: createPriceRatioPattern(this, 'realized_price', 'm2sd'),
                    m25sd: createPriceRatioPattern(this, 'realized_price', 'm2_5sd'),
                    m3sd: createPriceRatioPattern(this, 'realized_price', 'm3sd'),
                  },
                  _4y: {
                    sd: createSeriesPattern1(this, 'realized_price_ratio_sd_4y'),
                    zscore: createSeriesPattern1(this, 'realized_price_ratio_zscore_4y'),
                    _0sd: createCentsSatsUsdPattern(this, 'realized_price_0sd_4y'),
                    p05sd: createPriceRatioPattern(this, 'realized_price', 'p0_5sd_4y'),
                    p1sd: createPriceRatioPattern(this, 'realized_price', 'p1sd_4y'),
                    p15sd: createPriceRatioPattern(this, 'realized_price', 'p1_5sd_4y'),
                    p2sd: createPriceRatioPattern(this, 'realized_price', 'p2sd_4y'),
                    p25sd: createPriceRatioPattern(this, 'realized_price', 'p2_5sd_4y'),
                    p3sd: createPriceRatioPattern(this, 'realized_price', 'p3sd_4y'),
                    m05sd: createPriceRatioPattern(this, 'realized_price', 'm0_5sd_4y'),
                    m1sd: createPriceRatioPattern(this, 'realized_price', 'm1sd_4y'),
                    m15sd: createPriceRatioPattern(this, 'realized_price', 'm1_5sd_4y'),
                    m2sd: createPriceRatioPattern(this, 'realized_price', 'm2sd_4y'),
                    m25sd: createPriceRatioPattern(this, 'realized_price', 'm2_5sd_4y'),
                    m3sd: createPriceRatioPattern(this, 'realized_price', 'm3sd_4y'),
                  },
                  _2y: {
                    sd: createSeriesPattern1(this, 'realized_price_ratio_sd_2y'),
                    zscore: createSeriesPattern1(this, 'realized_price_ratio_zscore_2y'),
                    _0sd: createCentsSatsUsdPattern(this, 'realized_price_0sd_2y'),
                    p05sd: createPriceRatioPattern(this, 'realized_price', 'p0_5sd_2y'),
                    p1sd: createPriceRatioPattern(this, 'realized_price', 'p1sd_2y'),
                    p15sd: createPriceRatioPattern(this, 'realized_price', 'p1_5sd_2y'),
                    p2sd: createPriceRatioPattern(this, 'realized_price', 'p2sd_2y'),
                    p25sd: createPriceRatioPattern(this, 'realized_price', 'p2_5sd_2y'),
                    p3sd: createPriceRatioPattern(this, 'realized_price', 'p3sd_2y'),
                    m05sd: createPriceRatioPattern(this, 'realized_price', 'm0_5sd_2y'),
                    m1sd: createPriceRatioPattern(this, 'realized_price', 'm1sd_2y'),
                    m15sd: createPriceRatioPattern(this, 'realized_price', 'm1_5sd_2y'),
                    m2sd: createPriceRatioPattern(this, 'realized_price', 'm2sd_2y'),
                    m25sd: createPriceRatioPattern(this, 'realized_price', 'm2_5sd_2y'),
                    m3sd: createPriceRatioPattern(this, 'realized_price', 'm3sd_2y'),
                  },
                  _1y: {
                    sd: createSeriesPattern1(this, 'realized_price_ratio_sd_1y'),
                    zscore: createSeriesPattern1(this, 'realized_price_ratio_zscore_1y'),
                    _0sd: createCentsSatsUsdPattern(this, 'realized_price_0sd_1y'),
                    p05sd: createPriceRatioPattern(this, 'realized_price', 'p0_5sd_1y'),
                    p1sd: createPriceRatioPattern(this, 'realized_price', 'p1sd_1y'),
                    p15sd: createPriceRatioPattern(this, 'realized_price', 'p1_5sd_1y'),
                    p2sd: createPriceRatioPattern(this, 'realized_price', 'p2sd_1y'),
                    p25sd: createPriceRatioPattern(this, 'realized_price', 'p2_5sd_1y'),
                    p3sd: createPriceRatioPattern(this, 'realized_price', 'p3sd_1y'),
                    m05sd: createPriceRatioPattern(this, 'realized_price', 'm0_5sd_1y'),
                    m1sd: createPriceRatioPattern(this, 'realized_price', 'm1sd_1y'),
                    m15sd: createPriceRatioPattern(this, 'realized_price', 'm1_5sd_1y'),
                    m2sd: createPriceRatioPattern(this, 'realized_price', 'm2sd_1y'),
                    m25sd: createPriceRatioPattern(this, 'realized_price', 'm2_5sd_1y'),
                    m3sd: createPriceRatioPattern(this, 'realized_price', 'm3sd_1y'),
                  },
                },
              },
              mvrv: createSeriesPattern1(this, 'mvrv'),
              netPnl: createBlockChangeCumulativeDeltaSumPattern(this, 'net'),
              sopr: {
                valueDestroyed: createAverageBlockCumulativeSumPattern(this, 'value_destroyed'),
                ratio: create_1m1w1y24hPattern(this, 'sopr'),
                adjusted: {
                  ratio: create_1m1w1y24hPattern(this, 'asopr'),
                  transferVolume: createAverageBlockCumulativeSumPattern(this, 'adj_value_created'),
                  valueDestroyed: createAverageBlockCumulativeSumPattern(this, 'adj_value_destroyed'),
                },
              },
              grossPnl: createBlockCumulativeSumPattern(this, 'realized_gross_pnl'),
              sellSideRiskRatio: create_1m1w1y24hPattern8(this, 'sell_side_risk_ratio'),
              peakRegret: createBlockCumulativeSumPattern(this, 'realized_peak_regret'),
              capitalized: createPricePattern(this, 'capitalized_price'),
              profitToLossRatio: create_1m1w1y24hPattern(this, 'realized_profit_to_loss_ratio'),
            },
            costBasis: {
              inProfit: createPerPattern(this, 'cost_basis_in_profit_per'),
              inLoss: createPerPattern(this, 'cost_basis_in_loss_per'),
              min: createCentsSatsUsdPattern(this, 'cost_basis_min'),
              max: createCentsSatsUsdPattern(this, 'cost_basis_max'),
              perCoin: createPct05Pct10Pct15Pct20Pct25Pct30Pct35Pct40Pct45Pct50Pct55Pct60Pct65Pct70Pct75Pct80Pct85Pct90Pct95Pattern(this, 'cost_basis_per_coin'),
              perDollar: createPct05Pct10Pct15Pct20Pct25Pct30Pct35Pct40Pct45Pct50Pct55Pct60Pct65Pct70Pct75Pct80Pct85Pct90Pct95Pattern(this, 'cost_basis_per_dollar'),
              supplyDensity: createBpsPercentRatioPattern2(this, 'supply_density'),
            },
            unrealized: {
              nupl: createBpsRatioPattern(this, 'nupl'),
              profit: {
                usd: createSeriesPattern1(this, 'unrealized_profit'),
                cents: createSeriesPattern1(this, 'unrealized_profit_cents'),
                toMcap: createBpsPercentRatioPattern2(this, 'unrealized_profit_to_mcap'),
                toOwnGrossPnl: createBpsPercentRatioPattern2(this, 'unrealized_profit_to_own_gross_pnl'),
              },
              loss: {
                usd: createSeriesPattern1(this, 'unrealized_loss'),
                cents: createSeriesPattern1(this, 'unrealized_loss_cents'),
                negative: createSeriesPattern1(this, 'unrealized_loss_neg'),
                toMcap: createBpsPercentRatioPattern2(this, 'unrealized_loss_to_mcap'),
                toOwnGrossPnl: createBpsPercentRatioPattern2(this, 'unrealized_loss_to_own_gross_pnl'),
              },
              netPnl: {
                usd: createSeriesPattern1(this, 'net_unrealized_pnl'),
                cents: createSeriesPattern1(this, 'net_unrealized_pnl_cents'),
                toOwnGrossPnl: createBpsPercentRatioPattern(this, 'net_unrealized_pnl_to_own_gross_pnl'),
              },
              grossPnl: createCentsUsdPattern3(this, 'unrealized_gross_pnl'),
              investedCapital: createInPattern2(this, 'invested_capital_in'),
              capitalizedCapInProfitRaw: createSeriesPattern18(this, 'capitalized_cap_in_profit_raw'),
              capitalizedCapInLossRaw: createSeriesPattern18(this, 'capitalized_cap_in_loss_raw'),
              sentiment: {
                painIndex: createCentsUsdPattern3(this, 'pain_index'),
                greedIndex: createCentsUsdPattern3(this, 'greed_index'),
                net: createCentsUsdPattern(this, 'net_sentiment'),
              },
            },
            investedCapital: createInPattern(this, 'invested_capital_in'),
          },
          sth: {
            supply: createDeltaDominanceHalfInTotalPattern2(this, 'sth_supply'),
            outputs: createSpendingSpentUnspentPattern(this, 'sth'),
            activity: createCoindaysCoinyearsDormancyTransferPattern(this, 'sth'),
            realized: {
              cap: createCentsDeltaToUsdPattern(this, 'sth_realized_cap'),
              profit: createBlockCumulativeSumPattern(this, 'sth_realized_profit'),
              loss: createBlockCumulativeNegativeSumPattern(this, 'sth_realized_loss'),
              price: {
                usd: createSeriesPattern1(this, 'sth_realized_price'),
                cents: createSeriesPattern1(this, 'sth_realized_price_cents'),
                sats: createSeriesPattern1(this, 'sth_realized_price_sats'),
                bps: createSeriesPattern1(this, 'sth_realized_price_ratio_bps'),
                ratio: createSeriesPattern1(this, 'sth_realized_price_ratio'),
                percentiles: createPct0Pct1Pct2Pct5Pct95Pct98Pct99Pattern(this, 'sth_realized_price'),
                sma: create_1m1w1y2y4yAllPattern(this, 'sth_realized_price_ratio_sma'),
                stdDev: {
                  all: {
                    sd: createSeriesPattern1(this, 'sth_realized_price_ratio_sd'),
                    zscore: createSeriesPattern1(this, 'sth_realized_price_ratio_zscore'),
                    _0sd: createCentsSatsUsdPattern(this, 'sth_realized_price_0sd'),
                    p05sd: createPriceRatioPattern(this, 'sth_realized_price', 'p0_5sd'),
                    p1sd: createPriceRatioPattern(this, 'sth_realized_price', 'p1sd'),
                    p15sd: createPriceRatioPattern(this, 'sth_realized_price', 'p1_5sd'),
                    p2sd: createPriceRatioPattern(this, 'sth_realized_price', 'p2sd'),
                    p25sd: createPriceRatioPattern(this, 'sth_realized_price', 'p2_5sd'),
                    p3sd: createPriceRatioPattern(this, 'sth_realized_price', 'p3sd'),
                    m05sd: createPriceRatioPattern(this, 'sth_realized_price', 'm0_5sd'),
                    m1sd: createPriceRatioPattern(this, 'sth_realized_price', 'm1sd'),
                    m15sd: createPriceRatioPattern(this, 'sth_realized_price', 'm1_5sd'),
                    m2sd: createPriceRatioPattern(this, 'sth_realized_price', 'm2sd'),
                    m25sd: createPriceRatioPattern(this, 'sth_realized_price', 'm2_5sd'),
                    m3sd: createPriceRatioPattern(this, 'sth_realized_price', 'm3sd'),
                  },
                  _4y: {
                    sd: createSeriesPattern1(this, 'sth_realized_price_ratio_sd_4y'),
                    zscore: createSeriesPattern1(this, 'sth_realized_price_ratio_zscore_4y'),
                    _0sd: createCentsSatsUsdPattern(this, 'sth_realized_price_0sd_4y'),
                    p05sd: createPriceRatioPattern(this, 'sth_realized_price', 'p0_5sd_4y'),
                    p1sd: createPriceRatioPattern(this, 'sth_realized_price', 'p1sd_4y'),
                    p15sd: createPriceRatioPattern(this, 'sth_realized_price', 'p1_5sd_4y'),
                    p2sd: createPriceRatioPattern(this, 'sth_realized_price', 'p2sd_4y'),
                    p25sd: createPriceRatioPattern(this, 'sth_realized_price', 'p2_5sd_4y'),
                    p3sd: createPriceRatioPattern(this, 'sth_realized_price', 'p3sd_4y'),
                    m05sd: createPriceRatioPattern(this, 'sth_realized_price', 'm0_5sd_4y'),
                    m1sd: createPriceRatioPattern(this, 'sth_realized_price', 'm1sd_4y'),
                    m15sd: createPriceRatioPattern(this, 'sth_realized_price', 'm1_5sd_4y'),
                    m2sd: createPriceRatioPattern(this, 'sth_realized_price', 'm2sd_4y'),
                    m25sd: createPriceRatioPattern(this, 'sth_realized_price', 'm2_5sd_4y'),
                    m3sd: createPriceRatioPattern(this, 'sth_realized_price', 'm3sd_4y'),
                  },
                  _2y: {
                    sd: createSeriesPattern1(this, 'sth_realized_price_ratio_sd_2y'),
                    zscore: createSeriesPattern1(this, 'sth_realized_price_ratio_zscore_2y'),
                    _0sd: createCentsSatsUsdPattern(this, 'sth_realized_price_0sd_2y'),
                    p05sd: createPriceRatioPattern(this, 'sth_realized_price', 'p0_5sd_2y'),
                    p1sd: createPriceRatioPattern(this, 'sth_realized_price', 'p1sd_2y'),
                    p15sd: createPriceRatioPattern(this, 'sth_realized_price', 'p1_5sd_2y'),
                    p2sd: createPriceRatioPattern(this, 'sth_realized_price', 'p2sd_2y'),
                    p25sd: createPriceRatioPattern(this, 'sth_realized_price', 'p2_5sd_2y'),
                    p3sd: createPriceRatioPattern(this, 'sth_realized_price', 'p3sd_2y'),
                    m05sd: createPriceRatioPattern(this, 'sth_realized_price', 'm0_5sd_2y'),
                    m1sd: createPriceRatioPattern(this, 'sth_realized_price', 'm1sd_2y'),
                    m15sd: createPriceRatioPattern(this, 'sth_realized_price', 'm1_5sd_2y'),
                    m2sd: createPriceRatioPattern(this, 'sth_realized_price', 'm2sd_2y'),
                    m25sd: createPriceRatioPattern(this, 'sth_realized_price', 'm2_5sd_2y'),
                    m3sd: createPriceRatioPattern(this, 'sth_realized_price', 'm3sd_2y'),
                  },
                  _1y: {
                    sd: createSeriesPattern1(this, 'sth_realized_price_ratio_sd_1y'),
                    zscore: createSeriesPattern1(this, 'sth_realized_price_ratio_zscore_1y'),
                    _0sd: createCentsSatsUsdPattern(this, 'sth_realized_price_0sd_1y'),
                    p05sd: createPriceRatioPattern(this, 'sth_realized_price', 'p0_5sd_1y'),
                    p1sd: createPriceRatioPattern(this, 'sth_realized_price', 'p1sd_1y'),
                    p15sd: createPriceRatioPattern(this, 'sth_realized_price', 'p1_5sd_1y'),
                    p2sd: createPriceRatioPattern(this, 'sth_realized_price', 'p2sd_1y'),
                    p25sd: createPriceRatioPattern(this, 'sth_realized_price', 'p2_5sd_1y'),
                    p3sd: createPriceRatioPattern(this, 'sth_realized_price', 'p3sd_1y'),
                    m05sd: createPriceRatioPattern(this, 'sth_realized_price', 'm0_5sd_1y'),
                    m1sd: createPriceRatioPattern(this, 'sth_realized_price', 'm1sd_1y'),
                    m15sd: createPriceRatioPattern(this, 'sth_realized_price', 'm1_5sd_1y'),
                    m2sd: createPriceRatioPattern(this, 'sth_realized_price', 'm2sd_1y'),
                    m25sd: createPriceRatioPattern(this, 'sth_realized_price', 'm2_5sd_1y'),
                    m3sd: createPriceRatioPattern(this, 'sth_realized_price', 'm3sd_1y'),
                  },
                },
              },
              mvrv: createSeriesPattern1(this, 'sth_mvrv'),
              netPnl: createBlockChangeCumulativeDeltaSumPattern(this, 'sth_net'),
              sopr: createAdjustedRatioValuePattern(this, 'sth'),
              grossPnl: createBlockCumulativeSumPattern(this, 'sth_realized_gross_pnl'),
              sellSideRiskRatio: create_1m1w1y24hPattern8(this, 'sth_sell_side_risk_ratio'),
              peakRegret: createBlockCumulativeSumPattern(this, 'sth_realized_peak_regret'),
              capitalized: createPricePattern(this, 'sth_capitalized_price'),
              profitToLossRatio: create_1m1w1y24hPattern(this, 'sth_realized_profit_to_loss_ratio'),
            },
            costBasis: createInMaxMinPerSupplyPattern(this, 'sth'),
            unrealized: createCapitalizedGrossInvestedLossNetNuplProfitSentimentPattern2(this, 'sth'),
            investedCapital: createInPattern(this, 'sth_invested_capital_in'),
          },
          lth: {
            supply: createDeltaDominanceHalfInTotalPattern2(this, 'lth_supply'),
            outputs: createSpendingSpentUnspentPattern(this, 'lth'),
            activity: createCoindaysCoinyearsDormancyTransferPattern(this, 'lth'),
            realized: {
              cap: createCentsDeltaToUsdPattern(this, 'lth_realized_cap'),
              profit: createBlockCumulativeSumPattern(this, 'lth_realized_profit'),
              loss: createBlockCumulativeNegativeSumPattern(this, 'lth_realized_loss'),
              price: {
                usd: createSeriesPattern1(this, 'lth_realized_price'),
                cents: createSeriesPattern1(this, 'lth_realized_price_cents'),
                sats: createSeriesPattern1(this, 'lth_realized_price_sats'),
                bps: createSeriesPattern1(this, 'lth_realized_price_ratio_bps'),
                ratio: createSeriesPattern1(this, 'lth_realized_price_ratio'),
                percentiles: createPct0Pct1Pct2Pct5Pct95Pct98Pct99Pattern(this, 'lth_realized_price'),
                sma: create_1m1w1y2y4yAllPattern(this, 'lth_realized_price_ratio_sma'),
                stdDev: {
                  all: {
                    sd: createSeriesPattern1(this, 'lth_realized_price_ratio_sd'),
                    zscore: createSeriesPattern1(this, 'lth_realized_price_ratio_zscore'),
                    _0sd: createCentsSatsUsdPattern(this, 'lth_realized_price_0sd'),
                    p05sd: createPriceRatioPattern(this, 'lth_realized_price', 'p0_5sd'),
                    p1sd: createPriceRatioPattern(this, 'lth_realized_price', 'p1sd'),
                    p15sd: createPriceRatioPattern(this, 'lth_realized_price', 'p1_5sd'),
                    p2sd: createPriceRatioPattern(this, 'lth_realized_price', 'p2sd'),
                    p25sd: createPriceRatioPattern(this, 'lth_realized_price', 'p2_5sd'),
                    p3sd: createPriceRatioPattern(this, 'lth_realized_price', 'p3sd'),
                    m05sd: createPriceRatioPattern(this, 'lth_realized_price', 'm0_5sd'),
                    m1sd: createPriceRatioPattern(this, 'lth_realized_price', 'm1sd'),
                    m15sd: createPriceRatioPattern(this, 'lth_realized_price', 'm1_5sd'),
                    m2sd: createPriceRatioPattern(this, 'lth_realized_price', 'm2sd'),
                    m25sd: createPriceRatioPattern(this, 'lth_realized_price', 'm2_5sd'),
                    m3sd: createPriceRatioPattern(this, 'lth_realized_price', 'm3sd'),
                  },
                  _4y: {
                    sd: createSeriesPattern1(this, 'lth_realized_price_ratio_sd_4y'),
                    zscore: createSeriesPattern1(this, 'lth_realized_price_ratio_zscore_4y'),
                    _0sd: createCentsSatsUsdPattern(this, 'lth_realized_price_0sd_4y'),
                    p05sd: createPriceRatioPattern(this, 'lth_realized_price', 'p0_5sd_4y'),
                    p1sd: createPriceRatioPattern(this, 'lth_realized_price', 'p1sd_4y'),
                    p15sd: createPriceRatioPattern(this, 'lth_realized_price', 'p1_5sd_4y'),
                    p2sd: createPriceRatioPattern(this, 'lth_realized_price', 'p2sd_4y'),
                    p25sd: createPriceRatioPattern(this, 'lth_realized_price', 'p2_5sd_4y'),
                    p3sd: createPriceRatioPattern(this, 'lth_realized_price', 'p3sd_4y'),
                    m05sd: createPriceRatioPattern(this, 'lth_realized_price', 'm0_5sd_4y'),
                    m1sd: createPriceRatioPattern(this, 'lth_realized_price', 'm1sd_4y'),
                    m15sd: createPriceRatioPattern(this, 'lth_realized_price', 'm1_5sd_4y'),
                    m2sd: createPriceRatioPattern(this, 'lth_realized_price', 'm2sd_4y'),
                    m25sd: createPriceRatioPattern(this, 'lth_realized_price', 'm2_5sd_4y'),
                    m3sd: createPriceRatioPattern(this, 'lth_realized_price', 'm3sd_4y'),
                  },
                  _2y: {
                    sd: createSeriesPattern1(this, 'lth_realized_price_ratio_sd_2y'),
                    zscore: createSeriesPattern1(this, 'lth_realized_price_ratio_zscore_2y'),
                    _0sd: createCentsSatsUsdPattern(this, 'lth_realized_price_0sd_2y'),
                    p05sd: createPriceRatioPattern(this, 'lth_realized_price', 'p0_5sd_2y'),
                    p1sd: createPriceRatioPattern(this, 'lth_realized_price', 'p1sd_2y'),
                    p15sd: createPriceRatioPattern(this, 'lth_realized_price', 'p1_5sd_2y'),
                    p2sd: createPriceRatioPattern(this, 'lth_realized_price', 'p2sd_2y'),
                    p25sd: createPriceRatioPattern(this, 'lth_realized_price', 'p2_5sd_2y'),
                    p3sd: createPriceRatioPattern(this, 'lth_realized_price', 'p3sd_2y'),
                    m05sd: createPriceRatioPattern(this, 'lth_realized_price', 'm0_5sd_2y'),
                    m1sd: createPriceRatioPattern(this, 'lth_realized_price', 'm1sd_2y'),
                    m15sd: createPriceRatioPattern(this, 'lth_realized_price', 'm1_5sd_2y'),
                    m2sd: createPriceRatioPattern(this, 'lth_realized_price', 'm2sd_2y'),
                    m25sd: createPriceRatioPattern(this, 'lth_realized_price', 'm2_5sd_2y'),
                    m3sd: createPriceRatioPattern(this, 'lth_realized_price', 'm3sd_2y'),
                  },
                  _1y: {
                    sd: createSeriesPattern1(this, 'lth_realized_price_ratio_sd_1y'),
                    zscore: createSeriesPattern1(this, 'lth_realized_price_ratio_zscore_1y'),
                    _0sd: createCentsSatsUsdPattern(this, 'lth_realized_price_0sd_1y'),
                    p05sd: createPriceRatioPattern(this, 'lth_realized_price', 'p0_5sd_1y'),
                    p1sd: createPriceRatioPattern(this, 'lth_realized_price', 'p1sd_1y'),
                    p15sd: createPriceRatioPattern(this, 'lth_realized_price', 'p1_5sd_1y'),
                    p2sd: createPriceRatioPattern(this, 'lth_realized_price', 'p2sd_1y'),
                    p25sd: createPriceRatioPattern(this, 'lth_realized_price', 'p2_5sd_1y'),
                    p3sd: createPriceRatioPattern(this, 'lth_realized_price', 'p3sd_1y'),
                    m05sd: createPriceRatioPattern(this, 'lth_realized_price', 'm0_5sd_1y'),
                    m1sd: createPriceRatioPattern(this, 'lth_realized_price', 'm1sd_1y'),
                    m15sd: createPriceRatioPattern(this, 'lth_realized_price', 'm1_5sd_1y'),
                    m2sd: createPriceRatioPattern(this, 'lth_realized_price', 'm2sd_1y'),
                    m25sd: createPriceRatioPattern(this, 'lth_realized_price', 'm2_5sd_1y'),
                    m3sd: createPriceRatioPattern(this, 'lth_realized_price', 'm3sd_1y'),
                  },
                },
              },
              mvrv: createSeriesPattern1(this, 'lth_mvrv'),
              netPnl: createBlockChangeCumulativeDeltaSumPattern(this, 'lth_net'),
              sopr: createRatioValuePattern2(this, 'lth'),
              grossPnl: createBlockCumulativeSumPattern(this, 'lth_realized_gross_pnl'),
              sellSideRiskRatio: create_1m1w1y24hPattern8(this, 'lth_sell_side_risk_ratio'),
              peakRegret: createBlockCumulativeSumPattern(this, 'lth_realized_peak_regret'),
              capitalized: createPricePattern(this, 'lth_capitalized_price'),
              profitToLossRatio: create_1m1w1y24hPattern(this, 'lth_realized_profit_to_loss_ratio'),
            },
            costBasis: createInMaxMinPerSupplyPattern(this, 'lth'),
            unrealized: createCapitalizedGrossInvestedLossNetNuplProfitSentimentPattern2(this, 'lth'),
            investedCapital: createInPattern(this, 'lth_invested_capital_in'),
          },
          ageRange: {
            under1h: createActivityOutputsRealizedSupplyUnrealizedPattern(this, 'utxos_under_1h_old'),
            _1hTo1d: createActivityOutputsRealizedSupplyUnrealizedPattern(this, 'utxos_1h_to_1d_old'),
            _1dTo1w: createActivityOutputsRealizedSupplyUnrealizedPattern(this, 'utxos_1d_to_1w_old'),
            _1wTo1m: createActivityOutputsRealizedSupplyUnrealizedPattern(this, 'utxos_1w_to_1m_old'),
            _1mTo2m: createActivityOutputsRealizedSupplyUnrealizedPattern(this, 'utxos_1m_to_2m_old'),
            _2mTo3m: createActivityOutputsRealizedSupplyUnrealizedPattern(this, 'utxos_2m_to_3m_old'),
            _3mTo4m: createActivityOutputsRealizedSupplyUnrealizedPattern(this, 'utxos_3m_to_4m_old'),
            _4mTo5m: createActivityOutputsRealizedSupplyUnrealizedPattern(this, 'utxos_4m_to_5m_old'),
            _5mTo6m: createActivityOutputsRealizedSupplyUnrealizedPattern(this, 'utxos_5m_to_6m_old'),
            _6mTo1y: createActivityOutputsRealizedSupplyUnrealizedPattern(this, 'utxos_6m_to_1y_old'),
            _1yTo2y: createActivityOutputsRealizedSupplyUnrealizedPattern(this, 'utxos_1y_to_2y_old'),
            _2yTo3y: createActivityOutputsRealizedSupplyUnrealizedPattern(this, 'utxos_2y_to_3y_old'),
            _3yTo4y: createActivityOutputsRealizedSupplyUnrealizedPattern(this, 'utxos_3y_to_4y_old'),
            _4yTo5y: createActivityOutputsRealizedSupplyUnrealizedPattern(this, 'utxos_4y_to_5y_old'),
            _5yTo6y: createActivityOutputsRealizedSupplyUnrealizedPattern(this, 'utxos_5y_to_6y_old'),
            _6yTo7y: createActivityOutputsRealizedSupplyUnrealizedPattern(this, 'utxos_6y_to_7y_old'),
            _7yTo8y: createActivityOutputsRealizedSupplyUnrealizedPattern(this, 'utxos_7y_to_8y_old'),
            _8yTo10y: createActivityOutputsRealizedSupplyUnrealizedPattern(this, 'utxos_8y_to_10y_old'),
            _10yTo12y: createActivityOutputsRealizedSupplyUnrealizedPattern(this, 'utxos_10y_to_12y_old'),
            _12yTo15y: createActivityOutputsRealizedSupplyUnrealizedPattern(this, 'utxos_12y_to_15y_old'),
            over15y: createActivityOutputsRealizedSupplyUnrealizedPattern(this, 'utxos_over_15y_old'),
          },
          underAge: {
            _1w: createActivityOutputsRealizedSupplyUnrealizedPattern(this, 'utxos_under_1w_old'),
            _1m: createActivityOutputsRealizedSupplyUnrealizedPattern(this, 'utxos_under_1m_old'),
            _2m: createActivityOutputsRealizedSupplyUnrealizedPattern(this, 'utxos_under_2m_old'),
            _3m: createActivityOutputsRealizedSupplyUnrealizedPattern(this, 'utxos_under_3m_old'),
            _4m: createActivityOutputsRealizedSupplyUnrealizedPattern(this, 'utxos_under_4m_old'),
            _5m: createActivityOutputsRealizedSupplyUnrealizedPattern(this, 'utxos_under_5m_old'),
            _6m: createActivityOutputsRealizedSupplyUnrealizedPattern(this, 'utxos_under_6m_old'),
            _1y: createActivityOutputsRealizedSupplyUnrealizedPattern(this, 'utxos_under_1y_old'),
            _2y: createActivityOutputsRealizedSupplyUnrealizedPattern(this, 'utxos_under_2y_old'),
            _3y: createActivityOutputsRealizedSupplyUnrealizedPattern(this, 'utxos_under_3y_old'),
            _4y: createActivityOutputsRealizedSupplyUnrealizedPattern(this, 'utxos_under_4y_old'),
            _5y: createActivityOutputsRealizedSupplyUnrealizedPattern(this, 'utxos_under_5y_old'),
            _6y: createActivityOutputsRealizedSupplyUnrealizedPattern(this, 'utxos_under_6y_old'),
            _7y: createActivityOutputsRealizedSupplyUnrealizedPattern(this, 'utxos_under_7y_old'),
            _8y: createActivityOutputsRealizedSupplyUnrealizedPattern(this, 'utxos_under_8y_old'),
            _10y: createActivityOutputsRealizedSupplyUnrealizedPattern(this, 'utxos_under_10y_old'),
            _12y: createActivityOutputsRealizedSupplyUnrealizedPattern(this, 'utxos_under_12y_old'),
            _15y: createActivityOutputsRealizedSupplyUnrealizedPattern(this, 'utxos_under_15y_old'),
          },
          overAge: {
            _1d: createActivityOutputsRealizedSupplyUnrealizedPattern(this, 'utxos_over_1d_old'),
            _1w: createActivityOutputsRealizedSupplyUnrealizedPattern(this, 'utxos_over_1w_old'),
            _1m: createActivityOutputsRealizedSupplyUnrealizedPattern(this, 'utxos_over_1m_old'),
            _2m: createActivityOutputsRealizedSupplyUnrealizedPattern(this, 'utxos_over_2m_old'),
            _3m: createActivityOutputsRealizedSupplyUnrealizedPattern(this, 'utxos_over_3m_old'),
            _4m: createActivityOutputsRealizedSupplyUnrealizedPattern(this, 'utxos_over_4m_old'),
            _5m: createActivityOutputsRealizedSupplyUnrealizedPattern(this, 'utxos_over_5m_old'),
            _6m: createActivityOutputsRealizedSupplyUnrealizedPattern(this, 'utxos_over_6m_old'),
            _1y: createActivityOutputsRealizedSupplyUnrealizedPattern(this, 'utxos_over_1y_old'),
            _2y: createActivityOutputsRealizedSupplyUnrealizedPattern(this, 'utxos_over_2y_old'),
            _3y: createActivityOutputsRealizedSupplyUnrealizedPattern(this, 'utxos_over_3y_old'),
            _4y: createActivityOutputsRealizedSupplyUnrealizedPattern(this, 'utxos_over_4y_old'),
            _5y: createActivityOutputsRealizedSupplyUnrealizedPattern(this, 'utxos_over_5y_old'),
            _6y: createActivityOutputsRealizedSupplyUnrealizedPattern(this, 'utxos_over_6y_old'),
            _7y: createActivityOutputsRealizedSupplyUnrealizedPattern(this, 'utxos_over_7y_old'),
            _8y: createActivityOutputsRealizedSupplyUnrealizedPattern(this, 'utxos_over_8y_old'),
            _10y: createActivityOutputsRealizedSupplyUnrealizedPattern(this, 'utxos_over_10y_old'),
            _12y: createActivityOutputsRealizedSupplyUnrealizedPattern(this, 'utxos_over_12y_old'),
          },
          epoch: {
            _0: createActivityOutputsRealizedSupplyUnrealizedPattern(this, 'epoch_0'),
            _1: createActivityOutputsRealizedSupplyUnrealizedPattern(this, 'epoch_1'),
            _2: createActivityOutputsRealizedSupplyUnrealizedPattern(this, 'epoch_2'),
            _3: createActivityOutputsRealizedSupplyUnrealizedPattern(this, 'epoch_3'),
            _4: createActivityOutputsRealizedSupplyUnrealizedPattern(this, 'epoch_4'),
          },
          class: {
            _2009: createActivityOutputsRealizedSupplyUnrealizedPattern(this, 'class_2009'),
            _2010: createActivityOutputsRealizedSupplyUnrealizedPattern(this, 'class_2010'),
            _2011: createActivityOutputsRealizedSupplyUnrealizedPattern(this, 'class_2011'),
            _2012: createActivityOutputsRealizedSupplyUnrealizedPattern(this, 'class_2012'),
            _2013: createActivityOutputsRealizedSupplyUnrealizedPattern(this, 'class_2013'),
            _2014: createActivityOutputsRealizedSupplyUnrealizedPattern(this, 'class_2014'),
            _2015: createActivityOutputsRealizedSupplyUnrealizedPattern(this, 'class_2015'),
            _2016: createActivityOutputsRealizedSupplyUnrealizedPattern(this, 'class_2016'),
            _2017: createActivityOutputsRealizedSupplyUnrealizedPattern(this, 'class_2017'),
            _2018: createActivityOutputsRealizedSupplyUnrealizedPattern(this, 'class_2018'),
            _2019: createActivityOutputsRealizedSupplyUnrealizedPattern(this, 'class_2019'),
            _2020: createActivityOutputsRealizedSupplyUnrealizedPattern(this, 'class_2020'),
            _2021: createActivityOutputsRealizedSupplyUnrealizedPattern(this, 'class_2021'),
            _2022: createActivityOutputsRealizedSupplyUnrealizedPattern(this, 'class_2022'),
            _2023: createActivityOutputsRealizedSupplyUnrealizedPattern(this, 'class_2023'),
            _2024: createActivityOutputsRealizedSupplyUnrealizedPattern(this, 'class_2024'),
            _2025: createActivityOutputsRealizedSupplyUnrealizedPattern(this, 'class_2025'),
            _2026: createActivityOutputsRealizedSupplyUnrealizedPattern(this, 'class_2026'),
          },
          entry: {
            discount: {
              supply: createDeltaDominanceHalfInTotalPattern2(this, 'veteran_supply'),
              outputs: createSpendingSpentUnspentPattern(this, 'veteran'),
              activity: createCoindaysCoinyearsDormancyTransferPattern(this, 'veteran'),
              realized: {
                cap: createCentsDeltaToUsdPattern(this, 'veteran_realized_cap'),
                profit: createBlockCumulativeSumPattern(this, 'veteran_realized_profit'),
                loss: createBlockCumulativeNegativeSumPattern(this, 'veteran_realized_loss'),
                price: {
                  usd: createSeriesPattern1(this, 'veteran_realized_price'),
                  cents: createSeriesPattern1(this, 'veteran_realized_price_cents'),
                  sats: createSeriesPattern1(this, 'veteran_realized_price_sats'),
                  bps: createSeriesPattern1(this, 'veteran_realized_price_ratio_bps'),
                  ratio: createSeriesPattern1(this, 'veteran_realized_price_ratio'),
                  percentiles: createPct0Pct1Pct2Pct5Pct95Pct98Pct99Pattern(this, 'veteran_realized_price'),
                  sma: create_1m1w1y2y4yAllPattern(this, 'veteran_realized_price_ratio_sma'),
                  stdDev: {
                    all: {
                      sd: createSeriesPattern1(this, 'veteran_realized_price_ratio_sd'),
                      zscore: createSeriesPattern1(this, 'veteran_realized_price_ratio_zscore'),
                      _0sd: createCentsSatsUsdPattern(this, 'veteran_realized_price_0sd'),
                      p05sd: createPriceRatioPattern(this, 'veteran_realized_price', 'p0_5sd'),
                      p1sd: createPriceRatioPattern(this, 'veteran_realized_price', 'p1sd'),
                      p15sd: createPriceRatioPattern(this, 'veteran_realized_price', 'p1_5sd'),
                      p2sd: createPriceRatioPattern(this, 'veteran_realized_price', 'p2sd'),
                      p25sd: createPriceRatioPattern(this, 'veteran_realized_price', 'p2_5sd'),
                      p3sd: createPriceRatioPattern(this, 'veteran_realized_price', 'p3sd'),
                      m05sd: createPriceRatioPattern(this, 'veteran_realized_price', 'm0_5sd'),
                      m1sd: createPriceRatioPattern(this, 'veteran_realized_price', 'm1sd'),
                      m15sd: createPriceRatioPattern(this, 'veteran_realized_price', 'm1_5sd'),
                      m2sd: createPriceRatioPattern(this, 'veteran_realized_price', 'm2sd'),
                      m25sd: createPriceRatioPattern(this, 'veteran_realized_price', 'm2_5sd'),
                      m3sd: createPriceRatioPattern(this, 'veteran_realized_price', 'm3sd'),
                    },
                    _4y: {
                      sd: createSeriesPattern1(this, 'veteran_realized_price_ratio_sd_4y'),
                      zscore: createSeriesPattern1(this, 'veteran_realized_price_ratio_zscore_4y'),
                      _0sd: createCentsSatsUsdPattern(this, 'veteran_realized_price_0sd_4y'),
                      p05sd: createPriceRatioPattern(this, 'veteran_realized_price', 'p0_5sd_4y'),
                      p1sd: createPriceRatioPattern(this, 'veteran_realized_price', 'p1sd_4y'),
                      p15sd: createPriceRatioPattern(this, 'veteran_realized_price', 'p1_5sd_4y'),
                      p2sd: createPriceRatioPattern(this, 'veteran_realized_price', 'p2sd_4y'),
                      p25sd: createPriceRatioPattern(this, 'veteran_realized_price', 'p2_5sd_4y'),
                      p3sd: createPriceRatioPattern(this, 'veteran_realized_price', 'p3sd_4y'),
                      m05sd: createPriceRatioPattern(this, 'veteran_realized_price', 'm0_5sd_4y'),
                      m1sd: createPriceRatioPattern(this, 'veteran_realized_price', 'm1sd_4y'),
                      m15sd: createPriceRatioPattern(this, 'veteran_realized_price', 'm1_5sd_4y'),
                      m2sd: createPriceRatioPattern(this, 'veteran_realized_price', 'm2sd_4y'),
                      m25sd: createPriceRatioPattern(this, 'veteran_realized_price', 'm2_5sd_4y'),
                      m3sd: createPriceRatioPattern(this, 'veteran_realized_price', 'm3sd_4y'),
                    },
                    _2y: {
                      sd: createSeriesPattern1(this, 'veteran_realized_price_ratio_sd_2y'),
                      zscore: createSeriesPattern1(this, 'veteran_realized_price_ratio_zscore_2y'),
                      _0sd: createCentsSatsUsdPattern(this, 'veteran_realized_price_0sd_2y'),
                      p05sd: createPriceRatioPattern(this, 'veteran_realized_price', 'p0_5sd_2y'),
                      p1sd: createPriceRatioPattern(this, 'veteran_realized_price', 'p1sd_2y'),
                      p15sd: createPriceRatioPattern(this, 'veteran_realized_price', 'p1_5sd_2y'),
                      p2sd: createPriceRatioPattern(this, 'veteran_realized_price', 'p2sd_2y'),
                      p25sd: createPriceRatioPattern(this, 'veteran_realized_price', 'p2_5sd_2y'),
                      p3sd: createPriceRatioPattern(this, 'veteran_realized_price', 'p3sd_2y'),
                      m05sd: createPriceRatioPattern(this, 'veteran_realized_price', 'm0_5sd_2y'),
                      m1sd: createPriceRatioPattern(this, 'veteran_realized_price', 'm1sd_2y'),
                      m15sd: createPriceRatioPattern(this, 'veteran_realized_price', 'm1_5sd_2y'),
                      m2sd: createPriceRatioPattern(this, 'veteran_realized_price', 'm2sd_2y'),
                      m25sd: createPriceRatioPattern(this, 'veteran_realized_price', 'm2_5sd_2y'),
                      m3sd: createPriceRatioPattern(this, 'veteran_realized_price', 'm3sd_2y'),
                    },
                    _1y: {
                      sd: createSeriesPattern1(this, 'veteran_realized_price_ratio_sd_1y'),
                      zscore: createSeriesPattern1(this, 'veteran_realized_price_ratio_zscore_1y'),
                      _0sd: createCentsSatsUsdPattern(this, 'veteran_realized_price_0sd_1y'),
                      p05sd: createPriceRatioPattern(this, 'veteran_realized_price', 'p0_5sd_1y'),
                      p1sd: createPriceRatioPattern(this, 'veteran_realized_price', 'p1sd_1y'),
                      p15sd: createPriceRatioPattern(this, 'veteran_realized_price', 'p1_5sd_1y'),
                      p2sd: createPriceRatioPattern(this, 'veteran_realized_price', 'p2sd_1y'),
                      p25sd: createPriceRatioPattern(this, 'veteran_realized_price', 'p2_5sd_1y'),
                      p3sd: createPriceRatioPattern(this, 'veteran_realized_price', 'p3sd_1y'),
                      m05sd: createPriceRatioPattern(this, 'veteran_realized_price', 'm0_5sd_1y'),
                      m1sd: createPriceRatioPattern(this, 'veteran_realized_price', 'm1sd_1y'),
                      m15sd: createPriceRatioPattern(this, 'veteran_realized_price', 'm1_5sd_1y'),
                      m2sd: createPriceRatioPattern(this, 'veteran_realized_price', 'm2sd_1y'),
                      m25sd: createPriceRatioPattern(this, 'veteran_realized_price', 'm2_5sd_1y'),
                      m3sd: createPriceRatioPattern(this, 'veteran_realized_price', 'm3sd_1y'),
                    },
                  },
                },
                mvrv: createSeriesPattern1(this, 'veteran_mvrv'),
                netPnl: createBlockChangeCumulativeDeltaSumPattern(this, 'veteran_net'),
                sopr: createRatioValuePattern2(this, 'veteran'),
                grossPnl: createBlockCumulativeSumPattern(this, 'veteran_realized_gross_pnl'),
                sellSideRiskRatio: create_1m1w1y24hPattern8(this, 'veteran_sell_side_risk_ratio'),
                peakRegret: createBlockCumulativeSumPattern(this, 'veteran_realized_peak_regret'),
                capitalized: createPricePattern(this, 'veteran_capitalized_price'),
                profitToLossRatio: create_1m1w1y24hPattern(this, 'veteran_realized_profit_to_loss_ratio'),
              },
              costBasis: createInMaxMinPerSupplyPattern(this, 'veteran'),
              unrealized: createCapitalizedGrossInvestedLossNetNuplProfitSentimentPattern2(this, 'veteran'),
              investedCapital: createInPattern(this, 'veteran_invested_capital_in'),
            },
            premium: {
              supply: createDeltaDominanceHalfInTotalPattern2(this, 'rookie_supply'),
              outputs: createSpendingSpentUnspentPattern(this, 'rookie'),
              activity: createCoindaysCoinyearsDormancyTransferPattern(this, 'rookie'),
              realized: {
                cap: createCentsDeltaToUsdPattern(this, 'rookie_realized_cap'),
                profit: createBlockCumulativeSumPattern(this, 'rookie_realized_profit'),
                loss: createBlockCumulativeNegativeSumPattern(this, 'rookie_realized_loss'),
                price: {
                  usd: createSeriesPattern1(this, 'rookie_realized_price'),
                  cents: createSeriesPattern1(this, 'rookie_realized_price_cents'),
                  sats: createSeriesPattern1(this, 'rookie_realized_price_sats'),
                  bps: createSeriesPattern1(this, 'rookie_realized_price_ratio_bps'),
                  ratio: createSeriesPattern1(this, 'rookie_realized_price_ratio'),
                  percentiles: createPct0Pct1Pct2Pct5Pct95Pct98Pct99Pattern(this, 'rookie_realized_price'),
                  sma: create_1m1w1y2y4yAllPattern(this, 'rookie_realized_price_ratio_sma'),
                  stdDev: {
                    all: {
                      sd: createSeriesPattern1(this, 'rookie_realized_price_ratio_sd'),
                      zscore: createSeriesPattern1(this, 'rookie_realized_price_ratio_zscore'),
                      _0sd: createCentsSatsUsdPattern(this, 'rookie_realized_price_0sd'),
                      p05sd: createPriceRatioPattern(this, 'rookie_realized_price', 'p0_5sd'),
                      p1sd: createPriceRatioPattern(this, 'rookie_realized_price', 'p1sd'),
                      p15sd: createPriceRatioPattern(this, 'rookie_realized_price', 'p1_5sd'),
                      p2sd: createPriceRatioPattern(this, 'rookie_realized_price', 'p2sd'),
                      p25sd: createPriceRatioPattern(this, 'rookie_realized_price', 'p2_5sd'),
                      p3sd: createPriceRatioPattern(this, 'rookie_realized_price', 'p3sd'),
                      m05sd: createPriceRatioPattern(this, 'rookie_realized_price', 'm0_5sd'),
                      m1sd: createPriceRatioPattern(this, 'rookie_realized_price', 'm1sd'),
                      m15sd: createPriceRatioPattern(this, 'rookie_realized_price', 'm1_5sd'),
                      m2sd: createPriceRatioPattern(this, 'rookie_realized_price', 'm2sd'),
                      m25sd: createPriceRatioPattern(this, 'rookie_realized_price', 'm2_5sd'),
                      m3sd: createPriceRatioPattern(this, 'rookie_realized_price', 'm3sd'),
                    },
                    _4y: {
                      sd: createSeriesPattern1(this, 'rookie_realized_price_ratio_sd_4y'),
                      zscore: createSeriesPattern1(this, 'rookie_realized_price_ratio_zscore_4y'),
                      _0sd: createCentsSatsUsdPattern(this, 'rookie_realized_price_0sd_4y'),
                      p05sd: createPriceRatioPattern(this, 'rookie_realized_price', 'p0_5sd_4y'),
                      p1sd: createPriceRatioPattern(this, 'rookie_realized_price', 'p1sd_4y'),
                      p15sd: createPriceRatioPattern(this, 'rookie_realized_price', 'p1_5sd_4y'),
                      p2sd: createPriceRatioPattern(this, 'rookie_realized_price', 'p2sd_4y'),
                      p25sd: createPriceRatioPattern(this, 'rookie_realized_price', 'p2_5sd_4y'),
                      p3sd: createPriceRatioPattern(this, 'rookie_realized_price', 'p3sd_4y'),
                      m05sd: createPriceRatioPattern(this, 'rookie_realized_price', 'm0_5sd_4y'),
                      m1sd: createPriceRatioPattern(this, 'rookie_realized_price', 'm1sd_4y'),
                      m15sd: createPriceRatioPattern(this, 'rookie_realized_price', 'm1_5sd_4y'),
                      m2sd: createPriceRatioPattern(this, 'rookie_realized_price', 'm2sd_4y'),
                      m25sd: createPriceRatioPattern(this, 'rookie_realized_price', 'm2_5sd_4y'),
                      m3sd: createPriceRatioPattern(this, 'rookie_realized_price', 'm3sd_4y'),
                    },
                    _2y: {
                      sd: createSeriesPattern1(this, 'rookie_realized_price_ratio_sd_2y'),
                      zscore: createSeriesPattern1(this, 'rookie_realized_price_ratio_zscore_2y'),
                      _0sd: createCentsSatsUsdPattern(this, 'rookie_realized_price_0sd_2y'),
                      p05sd: createPriceRatioPattern(this, 'rookie_realized_price', 'p0_5sd_2y'),
                      p1sd: createPriceRatioPattern(this, 'rookie_realized_price', 'p1sd_2y'),
                      p15sd: createPriceRatioPattern(this, 'rookie_realized_price', 'p1_5sd_2y'),
                      p2sd: createPriceRatioPattern(this, 'rookie_realized_price', 'p2sd_2y'),
                      p25sd: createPriceRatioPattern(this, 'rookie_realized_price', 'p2_5sd_2y'),
                      p3sd: createPriceRatioPattern(this, 'rookie_realized_price', 'p3sd_2y'),
                      m05sd: createPriceRatioPattern(this, 'rookie_realized_price', 'm0_5sd_2y'),
                      m1sd: createPriceRatioPattern(this, 'rookie_realized_price', 'm1sd_2y'),
                      m15sd: createPriceRatioPattern(this, 'rookie_realized_price', 'm1_5sd_2y'),
                      m2sd: createPriceRatioPattern(this, 'rookie_realized_price', 'm2sd_2y'),
                      m25sd: createPriceRatioPattern(this, 'rookie_realized_price', 'm2_5sd_2y'),
                      m3sd: createPriceRatioPattern(this, 'rookie_realized_price', 'm3sd_2y'),
                    },
                    _1y: {
                      sd: createSeriesPattern1(this, 'rookie_realized_price_ratio_sd_1y'),
                      zscore: createSeriesPattern1(this, 'rookie_realized_price_ratio_zscore_1y'),
                      _0sd: createCentsSatsUsdPattern(this, 'rookie_realized_price_0sd_1y'),
                      p05sd: createPriceRatioPattern(this, 'rookie_realized_price', 'p0_5sd_1y'),
                      p1sd: createPriceRatioPattern(this, 'rookie_realized_price', 'p1sd_1y'),
                      p15sd: createPriceRatioPattern(this, 'rookie_realized_price', 'p1_5sd_1y'),
                      p2sd: createPriceRatioPattern(this, 'rookie_realized_price', 'p2sd_1y'),
                      p25sd: createPriceRatioPattern(this, 'rookie_realized_price', 'p2_5sd_1y'),
                      p3sd: createPriceRatioPattern(this, 'rookie_realized_price', 'p3sd_1y'),
                      m05sd: createPriceRatioPattern(this, 'rookie_realized_price', 'm0_5sd_1y'),
                      m1sd: createPriceRatioPattern(this, 'rookie_realized_price', 'm1sd_1y'),
                      m15sd: createPriceRatioPattern(this, 'rookie_realized_price', 'm1_5sd_1y'),
                      m2sd: createPriceRatioPattern(this, 'rookie_realized_price', 'm2sd_1y'),
                      m25sd: createPriceRatioPattern(this, 'rookie_realized_price', 'm2_5sd_1y'),
                      m3sd: createPriceRatioPattern(this, 'rookie_realized_price', 'm3sd_1y'),
                    },
                  },
                },
                mvrv: createSeriesPattern1(this, 'rookie_mvrv'),
                netPnl: createBlockChangeCumulativeDeltaSumPattern(this, 'rookie_net'),
                sopr: createRatioValuePattern2(this, 'rookie'),
                grossPnl: createBlockCumulativeSumPattern(this, 'rookie_realized_gross_pnl'),
                sellSideRiskRatio: create_1m1w1y24hPattern8(this, 'rookie_sell_side_risk_ratio'),
                peakRegret: createBlockCumulativeSumPattern(this, 'rookie_realized_peak_regret'),
                capitalized: createPricePattern(this, 'rookie_capitalized_price'),
                profitToLossRatio: create_1m1w1y24hPattern(this, 'rookie_realized_profit_to_loss_ratio'),
              },
              costBasis: createInMaxMinPerSupplyPattern(this, 'rookie'),
              unrealized: createCapitalizedGrossInvestedLossNetNuplProfitSentimentPattern2(this, 'rookie'),
              investedCapital: createInPattern(this, 'rookie_invested_capital_in'),
            },
          },
          overAmount: {
            _1sat: createActivityOutputsRealizedSupplyUnrealizedPattern2(this, 'utxos_over_1sat'),
            _10sats: createActivityOutputsRealizedSupplyUnrealizedPattern2(this, 'utxos_over_10sats'),
            _100sats: createActivityOutputsRealizedSupplyUnrealizedPattern2(this, 'utxos_over_100sats'),
            _1kSats: createActivityOutputsRealizedSupplyUnrealizedPattern2(this, 'utxos_over_1k_sats'),
            _10kSats: createActivityOutputsRealizedSupplyUnrealizedPattern2(this, 'utxos_over_10k_sats'),
            _100kSats: createActivityOutputsRealizedSupplyUnrealizedPattern2(this, 'utxos_over_100k_sats'),
            _1mSats: createActivityOutputsRealizedSupplyUnrealizedPattern2(this, 'utxos_over_1m_sats'),
            _10mSats: createActivityOutputsRealizedSupplyUnrealizedPattern2(this, 'utxos_over_10m_sats'),
            _1btc: createActivityOutputsRealizedSupplyUnrealizedPattern2(this, 'utxos_over_1btc'),
            _10btc: createActivityOutputsRealizedSupplyUnrealizedPattern2(this, 'utxos_over_10btc'),
            _100btc: createActivityOutputsRealizedSupplyUnrealizedPattern2(this, 'utxos_over_100btc'),
            _1kBtc: createActivityOutputsRealizedSupplyUnrealizedPattern2(this, 'utxos_over_1k_btc'),
            _10kBtc: createActivityOutputsRealizedSupplyUnrealizedPattern2(this, 'utxos_over_10k_btc'),
          },
          amountRange: {
            _0sats: createActivityOutputsRealizedSupplyUnrealizedPattern2(this, 'utxos_0sats'),
            _1satTo10sats: createActivityOutputsRealizedSupplyUnrealizedPattern2(this, 'utxos_1sat_to_10sats'),
            _10satsTo100sats: createActivityOutputsRealizedSupplyUnrealizedPattern2(this, 'utxos_10sats_to_100sats'),
            _100satsTo1kSats: createActivityOutputsRealizedSupplyUnrealizedPattern2(this, 'utxos_100sats_to_1k_sats'),
            _1kSatsTo10kSats: createActivityOutputsRealizedSupplyUnrealizedPattern2(this, 'utxos_1k_sats_to_10k_sats'),
            _10kSatsTo100kSats: createActivityOutputsRealizedSupplyUnrealizedPattern2(this, 'utxos_10k_sats_to_100k_sats'),
            _100kSatsTo1mSats: createActivityOutputsRealizedSupplyUnrealizedPattern2(this, 'utxos_100k_sats_to_1m_sats'),
            _1mSatsTo10mSats: createActivityOutputsRealizedSupplyUnrealizedPattern2(this, 'utxos_1m_sats_to_10m_sats'),
            _10mSatsTo1btc: createActivityOutputsRealizedSupplyUnrealizedPattern2(this, 'utxos_10m_sats_to_1btc'),
            _1btcTo10btc: createActivityOutputsRealizedSupplyUnrealizedPattern2(this, 'utxos_1btc_to_10btc'),
            _10btcTo100btc: createActivityOutputsRealizedSupplyUnrealizedPattern2(this, 'utxos_10btc_to_100btc'),
            _100btcTo1kBtc: createActivityOutputsRealizedSupplyUnrealizedPattern2(this, 'utxos_100btc_to_1k_btc'),
            _1kBtcTo10kBtc: createActivityOutputsRealizedSupplyUnrealizedPattern2(this, 'utxos_1k_btc_to_10k_btc'),
            _10kBtcTo100kBtc: createActivityOutputsRealizedSupplyUnrealizedPattern2(this, 'utxos_10k_btc_to_100k_btc'),
            over100kBtc: createActivityOutputsRealizedSupplyUnrealizedPattern2(this, 'utxos_over_100k_btc'),
          },
          underAmount: {
            _10sats: createActivityOutputsRealizedSupplyUnrealizedPattern2(this, 'utxos_under_10sats'),
            _100sats: createActivityOutputsRealizedSupplyUnrealizedPattern2(this, 'utxos_under_100sats'),
            _1kSats: createActivityOutputsRealizedSupplyUnrealizedPattern2(this, 'utxos_under_1k_sats'),
            _10kSats: createActivityOutputsRealizedSupplyUnrealizedPattern2(this, 'utxos_under_10k_sats'),
            _100kSats: createActivityOutputsRealizedSupplyUnrealizedPattern2(this, 'utxos_under_100k_sats'),
            _1mSats: createActivityOutputsRealizedSupplyUnrealizedPattern2(this, 'utxos_under_1m_sats'),
            _10mSats: createActivityOutputsRealizedSupplyUnrealizedPattern2(this, 'utxos_under_10m_sats'),
            _1btc: createActivityOutputsRealizedSupplyUnrealizedPattern2(this, 'utxos_under_1btc'),
            _10btc: createActivityOutputsRealizedSupplyUnrealizedPattern2(this, 'utxos_under_10btc'),
            _100btc: createActivityOutputsRealizedSupplyUnrealizedPattern2(this, 'utxos_under_100btc'),
            _1kBtc: createActivityOutputsRealizedSupplyUnrealizedPattern2(this, 'utxos_under_1k_btc'),
            _10kBtc: createActivityOutputsRealizedSupplyUnrealizedPattern2(this, 'utxos_under_10k_btc'),
            _100kBtc: createActivityOutputsRealizedSupplyUnrealizedPattern2(this, 'utxos_under_100k_btc'),
          },
          type: {
            p2pk65: createActivityOutputsRealizedSupplyUnrealizedPattern3(this, 'p2pk65'),
            p2pk33: createActivityOutputsRealizedSupplyUnrealizedPattern3(this, 'p2pk33'),
            p2pkh: createActivityOutputsRealizedSupplyUnrealizedPattern3(this, 'p2pkh'),
            p2ms: createActivityOutputsRealizedSupplyUnrealizedPattern3(this, 'p2ms'),
            p2sh: createActivityOutputsRealizedSupplyUnrealizedPattern3(this, 'p2sh'),
            p2wpkh: createActivityOutputsRealizedSupplyUnrealizedPattern3(this, 'p2wpkh'),
            p2wsh: createActivityOutputsRealizedSupplyUnrealizedPattern3(this, 'p2wsh'),
            p2tr: createActivityOutputsRealizedSupplyUnrealizedPattern3(this, 'p2tr'),
            p2a: createActivityOutputsRealizedSupplyUnrealizedPattern3(this, 'p2a'),
            unknown: createActivityOutputsRealizedSupplyUnrealizedPattern3(this, 'unknown_outputs'),
            empty: createActivityOutputsRealizedSupplyUnrealizedPattern3(this, 'empty_outputs'),
          },
          profitability: {
            range: {
              over1000pctInProfit: createNuplRealizedSupplyUnrealizedPattern(this, 'utxos_over_1000pct_in_profit'),
              _500pctTo1000pctInProfit: createNuplRealizedSupplyUnrealizedPattern(this, 'utxos_500pct_to_1000pct_in_profit'),
              _300pctTo500pctInProfit: createNuplRealizedSupplyUnrealizedPattern(this, 'utxos_300pct_to_500pct_in_profit'),
              _200pctTo300pctInProfit: createNuplRealizedSupplyUnrealizedPattern(this, 'utxos_200pct_to_300pct_in_profit'),
              _100pctTo200pctInProfit: createNuplRealizedSupplyUnrealizedPattern(this, 'utxos_100pct_to_200pct_in_profit'),
              _90pctTo100pctInProfit: createNuplRealizedSupplyUnrealizedPattern(this, 'utxos_90pct_to_100pct_in_profit'),
              _80pctTo90pctInProfit: createNuplRealizedSupplyUnrealizedPattern(this, 'utxos_80pct_to_90pct_in_profit'),
              _70pctTo80pctInProfit: createNuplRealizedSupplyUnrealizedPattern(this, 'utxos_70pct_to_80pct_in_profit'),
              _60pctTo70pctInProfit: createNuplRealizedSupplyUnrealizedPattern(this, 'utxos_60pct_to_70pct_in_profit'),
              _50pctTo60pctInProfit: createNuplRealizedSupplyUnrealizedPattern(this, 'utxos_50pct_to_60pct_in_profit'),
              _40pctTo50pctInProfit: createNuplRealizedSupplyUnrealizedPattern(this, 'utxos_40pct_to_50pct_in_profit'),
              _30pctTo40pctInProfit: createNuplRealizedSupplyUnrealizedPattern(this, 'utxos_30pct_to_40pct_in_profit'),
              _20pctTo30pctInProfit: createNuplRealizedSupplyUnrealizedPattern(this, 'utxos_20pct_to_30pct_in_profit'),
              _10pctTo20pctInProfit: createNuplRealizedSupplyUnrealizedPattern(this, 'utxos_10pct_to_20pct_in_profit'),
              _0pctTo10pctInProfit: createNuplRealizedSupplyUnrealizedPattern(this, 'utxos_0pct_to_10pct_in_profit'),
              _0pctTo10pctInLoss: createNuplRealizedSupplyUnrealizedPattern(this, 'utxos_0pct_to_10pct_in_loss'),
              _10pctTo20pctInLoss: createNuplRealizedSupplyUnrealizedPattern(this, 'utxos_10pct_to_20pct_in_loss'),
              _20pctTo30pctInLoss: createNuplRealizedSupplyUnrealizedPattern(this, 'utxos_20pct_to_30pct_in_loss'),
              _30pctTo40pctInLoss: createNuplRealizedSupplyUnrealizedPattern(this, 'utxos_30pct_to_40pct_in_loss'),
              _40pctTo50pctInLoss: createNuplRealizedSupplyUnrealizedPattern(this, 'utxos_40pct_to_50pct_in_loss'),
              _50pctTo60pctInLoss: createNuplRealizedSupplyUnrealizedPattern(this, 'utxos_50pct_to_60pct_in_loss'),
              _60pctTo70pctInLoss: createNuplRealizedSupplyUnrealizedPattern(this, 'utxos_60pct_to_70pct_in_loss'),
              _70pctTo80pctInLoss: createNuplRealizedSupplyUnrealizedPattern(this, 'utxos_70pct_to_80pct_in_loss'),
              _80pctTo90pctInLoss: createNuplRealizedSupplyUnrealizedPattern(this, 'utxos_80pct_to_90pct_in_loss'),
              _90pctTo100pctInLoss: createNuplRealizedSupplyUnrealizedPattern(this, 'utxos_90pct_to_100pct_in_loss'),
            },
            profit: {
              all: createNuplRealizedSupplyUnrealizedPattern(this, 'utxos_in_profit'),
              _10pct: createNuplRealizedSupplyUnrealizedPattern(this, 'utxos_over_10pct_in_profit'),
              _20pct: createNuplRealizedSupplyUnrealizedPattern(this, 'utxos_over_20pct_in_profit'),
              _30pct: createNuplRealizedSupplyUnrealizedPattern(this, 'utxos_over_30pct_in_profit'),
              _40pct: createNuplRealizedSupplyUnrealizedPattern(this, 'utxos_over_40pct_in_profit'),
              _50pct: createNuplRealizedSupplyUnrealizedPattern(this, 'utxos_over_50pct_in_profit'),
              _60pct: createNuplRealizedSupplyUnrealizedPattern(this, 'utxos_over_60pct_in_profit'),
              _70pct: createNuplRealizedSupplyUnrealizedPattern(this, 'utxos_over_70pct_in_profit'),
              _80pct: createNuplRealizedSupplyUnrealizedPattern(this, 'utxos_over_80pct_in_profit'),
              _90pct: createNuplRealizedSupplyUnrealizedPattern(this, 'utxos_over_90pct_in_profit'),
              _100pct: createNuplRealizedSupplyUnrealizedPattern(this, 'utxos_over_100pct_in_profit'),
              _200pct: createNuplRealizedSupplyUnrealizedPattern(this, 'utxos_over_200pct_in_profit'),
              _300pct: createNuplRealizedSupplyUnrealizedPattern(this, 'utxos_over_300pct_in_profit'),
              _500pct: createNuplRealizedSupplyUnrealizedPattern(this, 'utxos_over_500pct_in_profit'),
            },
            loss: {
              all: createNuplRealizedSupplyUnrealizedPattern(this, 'utxos_in_loss'),
              _10pct: createNuplRealizedSupplyUnrealizedPattern(this, 'utxos_over_10pct_in_loss'),
              _20pct: createNuplRealizedSupplyUnrealizedPattern(this, 'utxos_over_20pct_in_loss'),
              _30pct: createNuplRealizedSupplyUnrealizedPattern(this, 'utxos_over_30pct_in_loss'),
              _40pct: createNuplRealizedSupplyUnrealizedPattern(this, 'utxos_over_40pct_in_loss'),
              _50pct: createNuplRealizedSupplyUnrealizedPattern(this, 'utxos_over_50pct_in_loss'),
              _60pct: createNuplRealizedSupplyUnrealizedPattern(this, 'utxos_over_60pct_in_loss'),
              _70pct: createNuplRealizedSupplyUnrealizedPattern(this, 'utxos_over_70pct_in_loss'),
              _80pct: createNuplRealizedSupplyUnrealizedPattern(this, 'utxos_over_80pct_in_loss'),
            },
          },
          matured: {
            under1h: createAverageBlockCumulativeSumPattern3(this, 'utxos_under_1h_old_matured_supply'),
            _1hTo1d: createAverageBlockCumulativeSumPattern3(this, 'utxos_1h_to_1d_old_matured_supply'),
            _1dTo1w: createAverageBlockCumulativeSumPattern3(this, 'utxos_1d_to_1w_old_matured_supply'),
            _1wTo1m: createAverageBlockCumulativeSumPattern3(this, 'utxos_1w_to_1m_old_matured_supply'),
            _1mTo2m: createAverageBlockCumulativeSumPattern3(this, 'utxos_1m_to_2m_old_matured_supply'),
            _2mTo3m: createAverageBlockCumulativeSumPattern3(this, 'utxos_2m_to_3m_old_matured_supply'),
            _3mTo4m: createAverageBlockCumulativeSumPattern3(this, 'utxos_3m_to_4m_old_matured_supply'),
            _4mTo5m: createAverageBlockCumulativeSumPattern3(this, 'utxos_4m_to_5m_old_matured_supply'),
            _5mTo6m: createAverageBlockCumulativeSumPattern3(this, 'utxos_5m_to_6m_old_matured_supply'),
            _6mTo1y: createAverageBlockCumulativeSumPattern3(this, 'utxos_6m_to_1y_old_matured_supply'),
            _1yTo2y: createAverageBlockCumulativeSumPattern3(this, 'utxos_1y_to_2y_old_matured_supply'),
            _2yTo3y: createAverageBlockCumulativeSumPattern3(this, 'utxos_2y_to_3y_old_matured_supply'),
            _3yTo4y: createAverageBlockCumulativeSumPattern3(this, 'utxos_3y_to_4y_old_matured_supply'),
            _4yTo5y: createAverageBlockCumulativeSumPattern3(this, 'utxos_4y_to_5y_old_matured_supply'),
            _5yTo6y: createAverageBlockCumulativeSumPattern3(this, 'utxos_5y_to_6y_old_matured_supply'),
            _6yTo7y: createAverageBlockCumulativeSumPattern3(this, 'utxos_6y_to_7y_old_matured_supply'),
            _7yTo8y: createAverageBlockCumulativeSumPattern3(this, 'utxos_7y_to_8y_old_matured_supply'),
            _8yTo10y: createAverageBlockCumulativeSumPattern3(this, 'utxos_8y_to_10y_old_matured_supply'),
            _10yTo12y: createAverageBlockCumulativeSumPattern3(this, 'utxos_10y_to_12y_old_matured_supply'),
            _12yTo15y: createAverageBlockCumulativeSumPattern3(this, 'utxos_12y_to_15y_old_matured_supply'),
            over15y: createAverageBlockCumulativeSumPattern3(this, 'utxos_over_15y_old_matured_supply'),
          },
        },
        addr: {
          overAmount: {
            _1sat: createActivityAddrOutputsRealizedSupplyUnrealizedPattern(this, 'addrs_over_1sat'),
            _10sats: createActivityAddrOutputsRealizedSupplyUnrealizedPattern(this, 'addrs_over_10sats'),
            _100sats: createActivityAddrOutputsRealizedSupplyUnrealizedPattern(this, 'addrs_over_100sats'),
            _1kSats: createActivityAddrOutputsRealizedSupplyUnrealizedPattern(this, 'addrs_over_1k_sats'),
            _10kSats: createActivityAddrOutputsRealizedSupplyUnrealizedPattern(this, 'addrs_over_10k_sats'),
            _100kSats: createActivityAddrOutputsRealizedSupplyUnrealizedPattern(this, 'addrs_over_100k_sats'),
            _1mSats: createActivityAddrOutputsRealizedSupplyUnrealizedPattern(this, 'addrs_over_1m_sats'),
            _10mSats: createActivityAddrOutputsRealizedSupplyUnrealizedPattern(this, 'addrs_over_10m_sats'),
            _1btc: createActivityAddrOutputsRealizedSupplyUnrealizedPattern(this, 'addrs_over_1btc'),
            _10btc: createActivityAddrOutputsRealizedSupplyUnrealizedPattern(this, 'addrs_over_10btc'),
            _100btc: createActivityAddrOutputsRealizedSupplyUnrealizedPattern(this, 'addrs_over_100btc'),
            _1kBtc: createActivityAddrOutputsRealizedSupplyUnrealizedPattern(this, 'addrs_over_1k_btc'),
            _10kBtc: createActivityAddrOutputsRealizedSupplyUnrealizedPattern(this, 'addrs_over_10k_btc'),
          },
          amountRange: {
            _0sats: createActivityAddrOutputsRealizedSupplyUnrealizedPattern(this, 'addrs_0sats'),
            _1satTo10sats: createActivityAddrOutputsRealizedSupplyUnrealizedPattern(this, 'addrs_1sat_to_10sats'),
            _10satsTo100sats: createActivityAddrOutputsRealizedSupplyUnrealizedPattern(this, 'addrs_10sats_to_100sats'),
            _100satsTo1kSats: createActivityAddrOutputsRealizedSupplyUnrealizedPattern(this, 'addrs_100sats_to_1k_sats'),
            _1kSatsTo10kSats: createActivityAddrOutputsRealizedSupplyUnrealizedPattern(this, 'addrs_1k_sats_to_10k_sats'),
            _10kSatsTo100kSats: createActivityAddrOutputsRealizedSupplyUnrealizedPattern(this, 'addrs_10k_sats_to_100k_sats'),
            _100kSatsTo1mSats: createActivityAddrOutputsRealizedSupplyUnrealizedPattern(this, 'addrs_100k_sats_to_1m_sats'),
            _1mSatsTo10mSats: createActivityAddrOutputsRealizedSupplyUnrealizedPattern(this, 'addrs_1m_sats_to_10m_sats'),
            _10mSatsTo1btc: createActivityAddrOutputsRealizedSupplyUnrealizedPattern(this, 'addrs_10m_sats_to_1btc'),
            _1btcTo10btc: createActivityAddrOutputsRealizedSupplyUnrealizedPattern(this, 'addrs_1btc_to_10btc'),
            _10btcTo100btc: createActivityAddrOutputsRealizedSupplyUnrealizedPattern(this, 'addrs_10btc_to_100btc'),
            _100btcTo1kBtc: createActivityAddrOutputsRealizedSupplyUnrealizedPattern(this, 'addrs_100btc_to_1k_btc'),
            _1kBtcTo10kBtc: createActivityAddrOutputsRealizedSupplyUnrealizedPattern(this, 'addrs_1k_btc_to_10k_btc'),
            _10kBtcTo100kBtc: createActivityAddrOutputsRealizedSupplyUnrealizedPattern(this, 'addrs_10k_btc_to_100k_btc'),
            over100kBtc: createActivityAddrOutputsRealizedSupplyUnrealizedPattern(this, 'addrs_over_100k_btc'),
          },
          underAmount: {
            _10sats: createActivityAddrOutputsRealizedSupplyUnrealizedPattern(this, 'addrs_under_10sats'),
            _100sats: createActivityAddrOutputsRealizedSupplyUnrealizedPattern(this, 'addrs_under_100sats'),
            _1kSats: createActivityAddrOutputsRealizedSupplyUnrealizedPattern(this, 'addrs_under_1k_sats'),
            _10kSats: createActivityAddrOutputsRealizedSupplyUnrealizedPattern(this, 'addrs_under_10k_sats'),
            _100kSats: createActivityAddrOutputsRealizedSupplyUnrealizedPattern(this, 'addrs_under_100k_sats'),
            _1mSats: createActivityAddrOutputsRealizedSupplyUnrealizedPattern(this, 'addrs_under_1m_sats'),
            _10mSats: createActivityAddrOutputsRealizedSupplyUnrealizedPattern(this, 'addrs_under_10m_sats'),
            _1btc: createActivityAddrOutputsRealizedSupplyUnrealizedPattern(this, 'addrs_under_1btc'),
            _10btc: createActivityAddrOutputsRealizedSupplyUnrealizedPattern(this, 'addrs_under_10btc'),
            _100btc: createActivityAddrOutputsRealizedSupplyUnrealizedPattern(this, 'addrs_under_100btc'),
            _1kBtc: createActivityAddrOutputsRealizedSupplyUnrealizedPattern(this, 'addrs_under_1k_btc'),
            _10kBtc: createActivityAddrOutputsRealizedSupplyUnrealizedPattern(this, 'addrs_under_10k_btc'),
            _100kBtc: createActivityAddrOutputsRealizedSupplyUnrealizedPattern(this, 'addrs_under_100k_btc'),
          },
        },
      },
    };
  }

  /**
   * Create a dynamic series endpoint builder for any series/index combination.
   *
   * Use this for programmatic access when the series name is determined at runtime.
   * For type-safe access, use the `series` tree instead.
   *
   * @param {string} series - The series name
   * @param {Index} index - The index name
   * @returns {SeriesEndpoint<unknown>}
   */
  seriesEndpoint(series, index) {
    return _endpoint(this, series, index);
  }

  /**
   * Health check
   *
   * Liveness probe. Returns server identity, uptime, and indexed/computed heights from local state only (no bitcoind round-trip). For real chain-tip catch-up, see `/api/server/sync`.
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
   * Returns the sync status of the indexer, including indexed height, tip height, blocks behind, and last indexed timestamp.
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
   * Returns the disk space used by BRK and Bitcoin data.
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
   * @param {{ signal?: AbortSignal, onValue?: (value: SeriesCount[]) => void, cache?: boolean, memCache?: boolean }} [options]
   * @returns {Promise<SeriesCount[]>}
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
   * Fuzzy search for series by name. Supports partial matches and typos.
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
   * Returns the supported indexes and value type for the specified series.
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
   * Returns just the data array without the SeriesData wrapper. Supports the same range and format parameters as the standard endpoint.
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
   * Returns the current version of a series. Changes when the series data is updated.
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
   * Cohorts for which URPD data is available. Returns names like `all`, `sth`, `lth`, `utxos_under_1h_old`.
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
   * Dates for which a URPD snapshot is available for the cohort. One entry per UTC day, sorted ascending.
   *
   * Endpoint: `GET /api/urpd/{cohort}/dates`
   *
   * @param {Cohort} cohort
   * @param {{ signal?: AbortSignal, onValue?: (value: Date[]) => void, cache?: boolean, memCache?: boolean }} [options]
   * @returns {Promise<Date[]>}
   */
  async listUrpdDates(cohort, { signal, onValue, cache, memCache } = {}) {
    const path = `/api/urpd/${cohort}/dates`;
    return this.getJson(path, { signal, onValue, cache, memCache });
  }

  /**
   * Latest URPD
   *
   * URPD for the most recent available date in the cohort. The response's `date` field echoes which date was served.
   *
   * See the URPD tag description for the response shape and `agg` options.
   *
   * Endpoint: `GET /api/urpd/{cohort}`
   *
   * @param {Cohort} cohort
   * @param {UrpdAggregation=} [agg] - Aggregation strategy. Default: raw (no aggregation). Accepts `bucket` as alias.
   * @param {{ signal?: AbortSignal, onValue?: (value: Urpd) => void, cache?: boolean, memCache?: boolean }} [options]
   * @returns {Promise<Urpd>}
   */
  async getUrpd(cohort, agg, { signal, onValue, cache, memCache } = {}) {
    const params = new URLSearchParams();
    if (agg !== undefined) params.set('agg', String(agg));
    const query = params.toString();
    const path = `/api/urpd/${cohort}${query ? '?' + query : ''}`;
    return this.getJson(path, { signal, onValue, cache, memCache });
  }

  /**
   * URPD at date
   *
   * URPD for a (cohort, date) pair. Returns `{ cohort, date, aggregation, close, total_supply, buckets }` where each bucket is `{ price_floor, supply, realized_cap, unrealized_pnl }`.
   *
   * See the URPD tag description for unit conventions and `agg` options.
   *
   * Endpoint: `GET /api/urpd/{cohort}/{date}`
   *
   * @param {Cohort} cohort
   * @param {string} date
   * @param {UrpdAggregation=} [agg] - Aggregation strategy. Default: raw (no aggregation). Accepts `bucket` as alias.
   * @param {{ signal?: AbortSignal, onValue?: (value: Urpd) => void, cache?: boolean, memCache?: boolean }} [options]
   * @returns {Promise<Urpd>}
   */
  async getUrpdAt(cohort, date, agg, { signal, onValue, cache, memCache } = {}) {
    const params = new URLSearchParams();
    if (agg !== undefined) params.set('agg', String(agg));
    const query = params.toString();
    const path = `/api/urpd/${cohort}/${date}${query ? '?' + query : ''}`;
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
   * Get historical BTC/USD price. Optionally specify a UNIX timestamp to get the price at that time.
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
   * Find addresses by address type and by the first 1-16 hex nibbles of RapidHash v3 over the raw address payload bytes. Intended for privacy-preserving client-side wallet discovery without sending raw addresses or xpubs. Fetch metadata for the returned addresses through `/api/address/{address}`.
   *
   * Endpoint: `GET /api/address/hash-prefix/{addr_type}/{prefix}`
   *
   * @param {OutputType} addr_type
   * @param {string} prefix
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
   * Retrieve address information including balance and transaction counts. Supports all standard Bitcoin address types (P2PKH, P2SH, P2WPKH, P2WSH, P2TR).
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
   * Get transaction history for an address, newest first. Returns up to 50 mempool transactions plus a confirmed page sized to fill the response to 50 total (chain floor of 25, so 25-50 confirmed depending on mempool weight). To paginate further confirmed history, use `/address/{address}/txs/chain/{last_seen_txid}`.
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
   * Get the first 25 confirmed transactions for an address. For pagination, use the path-style form `/txs/chain/{last_seen_txid}`.
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
   * Find the block closest to a given UNIX timestamp.
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
   * Get the 10 most recent blocks mined by a specific pool.
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
   * Get 10 blocks mined by a specific pool before (and including) the given height.
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
   * Precise recommended fees
   *
   * Recommended fee rates with sub-integer precision.
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
   * Returns an opaque hash that changes whenever the projected next block changes. Same value as the mempool ETag. Useful as a freshness/liveness signal: if it stays constant for tens of seconds on a live network, the mempool sync loop has stalled.
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
   * Like `/api/v1/replacements`, but limited to trees where at least one predecessor was non-signaling (full-RBF).
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
   * Bitcoin Core's `getblocktemplate` selection: full transaction bodies in GBT order with aggregate stats. The returned `hash` is an opaque content token; pass it as `<hash>` on `/api/v1/mempool/block-template/diff/{hash}` to fetch deltas instead of refetching the whole template.
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
   * Delta of the projected next block since `<hash>`. `order` is the full new template in order: each entry is either a number (index into the prior template the client cached at `<hash>`) or a transaction object (new body to insert at this position). Walk `order` once to rebuild; `removed` is a convenience list of txids that left so clients can evict cached bodies. After applying, use the response `hash` as `<hash>` on the next call to keep iterating. Returns `404` when `<hash>` has aged out of server history; clients should fall back to `/api/v1/mempool/block-template`.
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
   * Live BTC/USD price
   *
   * Current BTC/USD price in dollars. Same value as `/api/mempool/price`. Confirmed per-height history is available at `/api/vecs/height-to-price`.
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
   * Live smoothed histogram of oracle-eligible payment outputs, binned by output value on the oracle log scale. It combines the committed oracle window with the forming mempool block. A flat array of log-scale bins.
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
   * @param {string} point
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
   * Live unfiltered output value histogram for the forming mempool block. Every live output is binned by value on the oracle log scale; no oracle payment filters are applied. A flat array of log-scale bins, all zero when no mempool is configured.
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
   * @param {string} point
   * @param {{ signal?: AbortSignal, onValue?: (value: number[]) => void, cache?: boolean, memCache?: boolean }} [options]
   * @returns {Promise<number[]>}
   */
  async getOracleHistogramOutputs(point, { signal, onValue, cache, memCache } = {}) {
    const path = `/api/oracle/histogram/outputs/${point}`;
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
   * Broadcast a raw transaction to the network. The transaction should be provided as hex in the request body. The txid will be returned on success.
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
    return this.postJson(path, body, { signal });
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
   * Compact OpenAPI specification optimized for LLM consumption. Removes redundant fields while preserving essential API information. Full spec available at `/openapi.json`.
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

export { BrkClient, BrkError, addressPayloadHashPrefix };
