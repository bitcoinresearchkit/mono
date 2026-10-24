// Auto-generated Bitview JavaScript client
// Do not edit manually

/**
 * Bitcoin address, or the hex public key of a P2PK output.
 *
 * @typedef {string} Address
 */
/**
 * @typedef {Object} AddressAfterTxidParam
 * @property {Address} address
 * @property {Txid} afterTxid - Last txid from the previous page (return transactions strictly older than this)
 */
/**
 * Address statistics on the blockchain (confirmed transactions only)
 *
 * Based on mempool.space's format with type_index extension.
 *
 * @typedef {Object} AddressChainStats
 * @property {Sats} balance - Current confirmed balance in satoshis
 * @property {number} fundedTxoCount - Total number of transaction outputs that funded this address
 * @property {Sats} fundedTxoSum - Total amount in satoshis received by this address across all funded outputs
 * @property {number} spentTxoCount - Total number of transaction outputs spent from this address
 * @property {Sats} spentTxoSum - Total amount in satoshis spent from this address
 * @property {number} txCount - Total number of confirmed transactions involving this address
 * @property {(TypeIndex|null)=} typeIndex - Index of this address within its type on the blockchain, null for an address without
confirmed activity
 * @property {Dollars} realizedPrice - Realized price (average cost basis) in USD
 */
/**
 * A page of an address's confirmed transactions.
 *
 * @typedef {Object} AddressChainTxsQuery
 * @property {?number=} maxTxs - Transactions to return, at most 25 (the default); larger values return 25.
 */
/**
 * @typedef {Object} AddressHashPrefixMatches
 * @property {OutputType} addressType
 * @property {string} prefix - The requested prefix, lowercased.
 * @property {boolean} truncated
 * @property {Address[]} addresses
 */
/**
 * @typedef {Object} AddressHashPrefixParam
 * @property {OutputType} addressType
 * @property {string} prefix - First 1–16 hexadecimal nibbles of the RapidHash v3 hash over the raw
address payload bytes.
 */
/**
 * Address statistics in the mempool (unconfirmed transactions only), in mempool.space's format.
 *
 * @typedef {Object} AddressMempoolStats
 * @property {SatsSigned} balanceDelta - Net pending (unconfirmed) balance change in satoshis; negative when pending spends exceed receipts
 * @property {number} fundedTxoCount - Number of unconfirmed transaction outputs funding this address
 * @property {Sats} fundedTxoSum - Total amount in satoshis being received in unconfirmed transactions
 * @property {number} spentTxoCount - Number of unconfirmed transaction inputs spending from this address
 * @property {Sats} spentTxoSum - Total amount in satoshis being spent in unconfirmed transactions
 * @property {number} txCount - Number of unconfirmed transactions involving this address
 */
/**
 * A page of an address's mempool transactions.
 *
 * @typedef {Object} AddressMempoolTxsQuery
 * @property {?number=} maxTxs - Transactions to return, at most 50 (the default); larger values return 50.
 */
/**
 * @typedef {Object} AddressParam
 * @property {Address} address
 */
/**
 * Address information compatible with mempool.space API format.
 *
 * @typedef {Object} AddressStats
 * @property {Address} address - Bitcoin address, or the hex public key of a P2PK output
 * @property {OutputType} addressType - Output script type
 * @property {AddressChainStats} chainStats - Statistics for confirmed transactions on the blockchain
 * @property {AddressMempoolStats} mempoolStats - Statistics for unconfirmed transactions in the mempool
 * @property {Sats} balance - Total current balance in satoshis, including pending (unconfirmed) mempool changes
 */
/**
 * A page of an address's transactions, mempool ones first.
 *
 * @typedef {Object} AddressTxsQuery
 * @property {(Txid|null)=} afterTxid - Continue after this transaction, a mempool or a confirmed one: the older ones (a confirmed
cursor skips the mempool). One that is neither answers 422.
 * @property {?number=} maxTxs - Transactions to return, at most 50 (the default); larger values return 50.
 */
/**
 * @typedef {Object} AddressValidation
 * @property {boolean} isvalid
 * @property {(Address|null)=} address - The validated address
 * @property {(Hex|null)=} scriptPubKey - The scriptPubKey in hex
 * @property {?boolean=} isscript - Whether the address is P2SH, P2WSH or P2TR
 * @property {?boolean=} iswitness
 * @property {?number=} witnessVersion - Witness version (0 for P2WPKH/P2WSH, 1 for P2TR and P2A)
 * @property {(Hex|null)=} witnessProgram - Witness program in hex
 * @property {?number[]=} errorLocations - Error locations: an empty array for invalid addresses, omitted for valid ones
 * @property {?string=} error - Error message for invalid addresses
 */
/**
 * An amount in bitcoin, as a decimal number (1 BTC = 100,000,000 satoshis).
 *
 * @typedef {number} Bitcoin
 */
/**
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
 * @property {BlockPool} pool
 * @property {Sats} avgFee - Mean fee per non-coinbase transaction in satoshis, rounded down (0 without any)
 * @property {FeeRate} avgFeeRate - Average fee rate in sat/vB
 * @property {Hex} coinbaseRaw - Raw coinbase transaction scriptsig as hex
 * @property {(Address|null)=} coinbaseAddress - Primary coinbase output address
 * @property {Address[]} coinbaseAddresses - All coinbase output addresses
 * @property {string} coinbaseSignature - Coinbase output script in ASM format
 * @property {string} coinbaseSignatureAscii - Coinbase scriptsig with each byte read as the Unicode code point of the same value
 * @property {number} avgTxSize - Non-coinbase transaction bytes over the transaction count (coinbase included), rounded to
2 decimals
 * @property {number} totalInputs - Total number of inputs (excluding coinbase)
 * @property {number} totalOutputs
 * @property {Sats} totalOutputAmt - Total output amount in satoshis, coinbase excluded
 * @property {Sats} medianFeeAmt - Median fee amount in satoshis
 * @property {Sats[]} feePercentiles - Fee amount percentiles in satoshis: [min, 10%, 25%, 50%, 75%, 90%, max]
 * @property {number} segwitTotalTxs
 * @property {number} segwitTotalSize - Total size of segwit transactions in bytes
 * @property {Weight} segwitTotalWeight
 * @property {Hex} header - Raw 80-byte block header as hex
 * @property {number} utxoSetChange - UTXO set change as mempool.space and bitcoin-cli report it: total outputs minus total
inputs, unspendable outputs (OP_RETURN) included, unlike `utxoSetSize`
 * @property {number} utxoSetSize - Total spendable UTXO set size at this height (excludes OP_RETURN and other unspendable outputs)
 * @property {Sats} totalInputAmt - Total input amount in satoshis
 * @property {number} virtualSize - Virtual size in vbytes: weight / 4, unrounded
 * @property {?boolean=} coinbaseBip54 - Whether the coinbase follows BIP54 (its lock time is the height minus one and its input's
sequence isn't final), from timestamp 1,771,507,776 on; null before
 * @property {(Timestamp|null)=} firstSeen - Timestamp when the block was first seen (always null, not yet supported)
 * @property {BlockOrphan[]} orphans - Stale blocks at this height (always empty, not yet supported)
 * @property {?number=} matchRate - Share of the expected block template the block matched, from mempool.space's own audits
(always null: it needs the mempool as it was)
 * @property {(Sats|null)=} expectedFees - Fees of the expected block template (always null: it needs the mempool as it was)
 * @property {(Weight|null)=} expectedWeight - Weight of the expected block template (always null: it needs the mempool as it was)
 * @property {Dollars} price - USD price at block height
 */
/**
 * One time bucket of blocks. Each `avgFee_*` field is the bucket's mean of its blocks' own
 * percentile of their non-coinbase transactions' effective fee rates, weighted by vsize (a block
 * without such transactions counts as 0), rounded to a whole sat/vB.
 *
 * @typedef {Object} BlockFeeRatesEntry
 * @property {Height} avgHeight - Average block height in this bucket
 * @property {Timestamp} timestamp - Average block timestamp in this bucket
 * @property {number} avgFee_0 - Mean of the blocks' minimum fee rate (sat/vB)
 * @property {number} avgFee_10 - Mean of the blocks' 10th percentile fee rate (sat/vB)
 * @property {number} avgFee_25 - Mean of the blocks' 25th percentile fee rate (sat/vB)
 * @property {number} avgFee_50 - Mean of the blocks' median fee rate (sat/vB)
 * @property {number} avgFee_75 - Mean of the blocks' 75th percentile fee rate (sat/vB)
 * @property {number} avgFee_90 - Mean of the blocks' 90th percentile fee rate (sat/vB)
 * @property {number} avgFee_100 - Mean of the blocks' maximum fee rate (sat/vB)
 */
/**
 * One time bucket of blocks, as [`BlockFeeRatesEntry`] with each `avgFee_*` field to 3
 * decimals.
 *
 * @typedef {Object} BlockFeeRatesPreciseEntry
 * @property {Height} avgHeight - Average block height in this bucket
 * @property {Timestamp} timestamp - Average block timestamp in this bucket
 * @property {FeeRate} avgFee_0 - Mean of the blocks' minimum fee rate (sat/vB)
 * @property {FeeRate} avgFee_10 - Mean of the blocks' 10th percentile fee rate (sat/vB)
 * @property {FeeRate} avgFee_25 - Mean of the blocks' 25th percentile fee rate (sat/vB)
 * @property {FeeRate} avgFee_50 - Mean of the blocks' median fee rate (sat/vB)
 * @property {FeeRate} avgFee_75 - Mean of the blocks' 75th percentile fee rate (sat/vB)
 * @property {FeeRate} avgFee_90 - Mean of the blocks' 90th percentile fee rate (sat/vB)
 * @property {FeeRate} avgFee_100 - Mean of the blocks' maximum fee rate (sat/vB)
 */
/**
 * One time bucket of blocks.
 *
 * @typedef {Object} BlockFeesEntry
 * @property {Height} avgHeight - Average block height in this bucket
 * @property {Timestamp} timestamp - Average block timestamp in this bucket
 * @property {Sats} avgFees - Mean fees per block in this bucket, rounded half up (sats)
 * @property {Dollars} USD - Mean BTC/USD spot price over the bucket's blocks (the price, not the fees in USD)
 */
/**
 * Double-SHA256 block-header hash, serialized in Bitcoin's conventional
 * hexadecimal byte order.
 *
 * @typedef {string} BlockHash
 */
/**
 * @typedef {Object} BlockHashParam
 * @property {BlockHash} hash
 */
/**
 * @typedef {Object} BlockHashStartIndex
 * @property {BlockHash} hash
 * @property {BlockTxIndex} startIndex - Starting transaction index within the block (0-based)
 */
/**
 * @typedef {Object} BlockHashTxIndex
 * @property {BlockHash} hash
 * @property {BlockTxIndex} index - Transaction index within the block (0-based)
 */
/**
 * Block information matching mempool.space /api/block/{hash}
 *
 * @typedef {Object} BlockInfo
 * @property {BlockHash} id - Block hash
 * @property {Height} height
 * @property {number} version
 * @property {Timestamp} timestamp - Block timestamp (Unix time)
 * @property {number} bits - Compact target (bits)
 * @property {number} nonce
 * @property {number} difficulty
 * @property {MerkleNode} merkleRoot
 * @property {number} txCount
 * @property {number} size - Block size in bytes
 * @property {Weight} weight - Block weight in weight units
 * @property {(BlockHash|null)=} previousblockhash - Hash of the previous block (null for the genesis block)
 * @property {Timestamp} mediantime - Median time of the last 11 blocks
 */
/**
 * Block information with extras, matching mempool.space /api/v1/blocks
 *
 * @typedef {Object} BlockInfoV1
 * @property {BlockHash} id - Block hash
 * @property {Height} height
 * @property {number} version
 * @property {Timestamp} timestamp - Block timestamp (Unix time)
 * @property {number} bits - Compact target (bits)
 * @property {number} nonce
 * @property {number} difficulty
 * @property {MerkleNode} merkleRoot
 * @property {number} txCount
 * @property {number} size - Block size in bytes
 * @property {Weight} weight - Block weight in weight units
 * @property {(BlockHash|null)=} previousblockhash - Hash of the previous block (null for the genesis block)
 * @property {Timestamp} mediantime - Median time of the last 11 blocks
 * @property {boolean=} stale - Whether this block has been replaced by a longer chain
 * @property {BlockExtras} extras
 */
/**
 * A stale block at a height of the best chain: a fork tip the node knows (bitcoind's
 * `getchaintips`), as mempool.space lists it.
 *
 * @typedef {Object} BlockOrphan
 * @property {Height} height
 * @property {BlockHash} hash
 * @property {number} branchlen - Length of the fork's branch, in blocks
 * @property {BlockOrphanStatus} status
 * @property {BlockHash} prevhash - Hash of the block the stale block builds on
 */
/**
 * How far the node validated a fork tip (bitcoind's `getchaintips` statuses other than
 * `active`).
 *
 * @typedef {(("invalid")|("headers-only")|("valid-headers")|("valid-fork"))} BlockOrphanStatus
 */
/**
 * @typedef {Object} BlockPool
 * @property {number} id - Zero-based pool ID, matching mempool.space's `unique_id`
 * @property {string} name
 * @property {PoolSlug} slug - URL-friendly pool identifier
 * @property {number} blockNumber - This block's ordinal among blocks attributed to this pool
 * @property {?string[]=} minerNames - Miner name tags found in coinbase scriptsig
 */
/**
 * One time bucket of blocks.
 *
 * @typedef {Object} BlockRewardsEntry
 * @property {Height} avgHeight - Average block height in this bucket
 * @property {Timestamp} timestamp - Average block timestamp in this bucket
 * @property {Sats} avgRewards - Mean coinbase reward (subsidy + fees) per block in this bucket, rounded half up (sats)
 * @property {Dollars} USD - Mean BTC/USD spot price over the bucket's blocks (the price, not the rewards in USD)
 */
/**
 * One time bucket of blocks.
 *
 * @typedef {Object} BlockSizeEntry
 * @property {Height} avgHeight - Average block height in this bucket
 * @property {Timestamp} timestamp - Average block timestamp in this bucket
 * @property {number} avgSize - Mean block size in this bucket, rounded half up (bytes)
 */
/**
 * @typedef {Object} BlockSizesWeights
 * @property {BlockSizeEntry[]} sizes
 * @property {BlockWeightEntry[]} weights
 */
/**
 * @typedef {Object} BlockStatus
 * @property {boolean} inBestChain
 * @property {(Height|null)=} height - Block height (null unless in the best chain)
 * @property {(BlockHash|null)=} nextBest - Hash of the next block in the best chain (null if tip)
 */
/**
 * Block information returned for timestamp queries
 *
 * @typedef {Object} BlockTimestamp
 * @property {Height} height
 * @property {BlockHash} hash
 * @property {string} timestamp - Block timestamp in ISO 8601 format
 */
/**
 * Position of a transaction within a single block (0 = coinbase).
 * Distinct from `TxIndex`, which is the chain-wide global tx index.
 *
 * @typedef {number} BlockTxIndex
 */
/**
 * One time bucket of blocks.
 *
 * @typedef {Object} BlockWeightEntry
 * @property {Height} avgHeight - Average block height in this bucket
 * @property {Timestamp} timestamp - Average block timestamp in this bucket
 * @property {Weight} avgWeight - Mean block weight in this bucket, rounded half up (weight units)
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
 * A mean size in bytes.
 *
 * @typedef {number} BytesFract
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
 * CPFP of a confirmed transaction with relatives in its block: its package there.
 *
 * @typedef {Object} ConfirmedCpfp
 * @property {CpfpEntry[]} ancestors
 * @property {CpfpEntry[]} descendants
 * @property {FeeRate} effectiveFeePerVsize - Feerate of the chunk holding the transaction when its block's relatives are linearized
(sat/vB).
 * @property {CpfpCluster} cluster - The transaction's relatives in its block.
 */
/**
 * A number of things.
 *
 * @typedef {number} Count
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
 * A transaction's CPFP (Child Pays For Parent) answer, in mempool.space's shapes, told apart by
 * their fields (untagged on the wire): a mempool transaction's package, a confirmed transaction's
 * package among its block's transactions, or `{"ancestors":[]}` for an unknown transaction or a
 * confirmed one without relatives in its block.
 *
 * @typedef {(CpfpInfo|ConfirmedCpfp|EmptyCpfp)} Cpfp
 */
/**
 * CPFP cluster: the connected component the seed belongs to, plus its
 * linearization.
 *
 * @typedef {Object} CpfpCluster
 * @property {CpfpClusterTx[]} txs - All txs in the cluster, in topological order (parents before children).
 * @property {CpfpClusterChunk[]} chunks - Chunks ordered by descending feerate.
 * @property {number} chunkIndex - Index into `chunks` of the chunk containing the seed tx.
 */
/**
 * One linearized chunk inside a `CpfpCluster`. `txs` follows `CpfpCluster.txs` order (topological);
 * `feerate` is the chunk's, shared by all its transactions.
 *
 * @typedef {Object} CpfpClusterChunk
 * @property {CpfpClusterTxIndex[]} txs
 * @property {FeeRate} feerate
 */
/**
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
 * CPFP (Child Pays For Parent) information for a mempool transaction.
 *
 * @typedef {Object} CpfpInfo
 * @property {CpfpEntry[]} ancestors
 * @property {(CpfpEntry|null)=} bestDescendant - Best (highest fee rate) descendant, if any.
 * @property {CpfpEntry[]} descendants
 * @property {FeeRate} effectiveFeePerVsize - Effective fee rate considering CPFP relationships (sat/vB).
This is the seed's chunk feerate after lift-merging, i.e. the
rate Core/mempool.space would surface for this tx.
 * @property {SigOps} sigops - BIP-141 sigop cost for the seed tx (witness sigops count as 1,
legacy and P2SH-redeem sigops count as 4).
 * @property {Sats} fee - Transaction fee (sats).
 * @property {VSize} vsize - Virtual size of the seed tx (vbytes).
 * @property {VSize} adjustedVsize - Policy-adjusted virtual size: `max(vsize, sigops * 5)`.
 * @property {(CpfpCluster|null)=} cluster - Cluster the seed belongs to; omitted when the seed has no ancestors and no descendants
(matches mempool.space).
 */
/**
 * A quote currency prices are served in: USD only.
 *
 * @typedef {("USD")} Currency
 */
/**
 * @typedef {Object} DataRangeFormat
 * @property {(RangeIndex|null)=} start - Inclusive start: integer index, date (YYYY-MM-DD), or timestamp (ISO 8601). Negative integers count from end.
 * @property {(RangeIndex|null)=} end - Exclusive end: integer index, date (YYYY-MM-DD), or timestamp (ISO 8601). Negative integers count from end.
 * @property {(Limit|null)=} limit - Maximum number of values to return (ignored if `end` is set); omitted, the rest of the range.
 * @property {Format=} format
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
 * @property {{ [key: string]: SeriesCount }} byDb
 */
/**
 * Proof-of-work difficulty.
 *
 * @typedef {number} Difficulty
 */
/**
 * @typedef {Object} DifficultyAdjustment
 * @property {number} progressPercent - Progress through current difficulty epoch (0-100%)
 * @property {number} difficultyChange - Estimated difficulty change at next retarget at the `adjustedTimeAvg` pace (%)
 * @property {number} estimatedRetargetDate - Estimated timestamp of next retarget (milliseconds)
 * @property {number} remainingBlocks - Blocks remaining until retarget
 * @property {number} remainingTime - Estimated time until retarget at `adjustedTimeAvg` (milliseconds)
 * @property {number} previousRetarget - Previous difficulty adjustment (%)
 * @property {Timestamp} previousTime - Timestamp of most recent retarget (seconds)
 * @property {Height} nextRetargetHeight
 * @property {number} timeAvg - Average block time in current epoch (milliseconds)
 * @property {number} adjustedTimeAvg - Block time the projection uses (milliseconds): `timeAvg`, except in an epoch's first 503
blocks, where it spans the last 503 blocks, those before the retarget scaled by it
 * @property {number} timeOffset - Testnet's offset for its 20-minute minimum-difficulty rule (milliseconds): always 0 on
mainnet, as mempool.space
 * @property {number} expectedBlocks - Blocks expected at the 10-minute target between the epoch's first block and the tip block
 */
/**
 * A difficulty adjustment, written as an array (as mempool.space does): `[timestamp, height, difficulty, adjustment]`.
 *
 * @typedef {[Timestamp, Height, number, number]} DifficultyAdjustmentEntry
 */
/**
 * A single difficulty data point in the hashrate summary.
 *
 * @typedef {Object} DifficultyEntry
 * @property {Timestamp} time - Unix timestamp of the difficulty adjustment
 * @property {Height} height
 * @property {number} difficulty
 * @property {number} adjustment - Adjustment ratio (new/previous, e.g. 1.068 = +6.8%)
 */
/**
 * Index of a difficulty epoch (2,016 blocks).
 *
 * @typedef {number} DifficultyEpoch
 */
/**
 * @typedef {Object} DiskUsage
 * @property {string} brk - Human-readable brk data size (e.g., "48.8 GiB")
 * @property {number} brkBytes - brk data size in bytes
 * @property {string} bitcoin - Human-readable Bitcoin blocks directory size
 * @property {number} bitcoinBytes - Bitcoin blocks directory size in bytes
 * @property {number} ratio - Ratio of BRK bytes to Bitcoin bytes; zero when Bitcoin bytes are zero.
 */
/**
 * US dollars multiplied by blocks: coinblocks valued at the spot price.
 *
 * @typedef {number} DollarBlocks
 */
/**
 * US dollars multiplied by days, per bitcoin: coin days valued at the spot price, per
 * bitcoin of supply.
 *
 * @typedef {number} DollarDaysPerCoin
 */
/**
 * An amount in US dollars.
 *
 * @typedef {number} Dollars
 */
/**
 * CPFP of a transaction without known relatives: `{"ancestors":[]}`.
 *
 * @typedef {Object} EmptyCpfp
 * @property {*[]} ancestors
 */
/**
 * Index of an output with an empty script.
 *
 * @typedef {TypeIndex} EmptyScriptOutputIndex
 */
/**
 * The JSON body of every API error (`application/problem+json`).
 *
 * @typedef {Object} ErrorBody
 * @property {ErrorDetail} error
 */
/** @typedef {("not_found"|"invalid_address"|"invalid_network"|"unsupported_type"|"no_series"|"series_unsupported_index"|"weight_exceeded"|"too_many_utxos"|"unknown_after_txid"|"unknown_address"|"unknown_txid"|"unknown_pool"|"out_of_range"|"unindexable_date"|"no_data"|"series_not_found"|"mempool_not_available"|"state_updating"|"internal_error"|"bad_request"|"overloaded"|"timeout"|"method_not_allowed")} ErrorCode */
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
 * Always empty: prices are quoted in USD only.
 *
 * @typedef {Object} ExchangeRates
 */
/**
 * Fee rate stored in milli-sat/vB and exposed as sat/vB.
 *
 * @typedef {number} FeeRate
 */
/**
 * A floating-point value without a specific unit.
 *
 * @typedef {number} Float
 */
/** @typedef {("json"|"csv")} Format */
/**
 * Index of a halving epoch (210,000 blocks).
 *
 * @typedef {number} HalvingEpoch
 */
/**
 * US dollars per PH/s of hash rate per day.
 *
 * @typedef {number} HashPrice
 */
/**
 * Satoshis per PH/s of hash rate per day.
 *
 * @typedef {number} HashValue
 */
/**
 * Hashes per second.
 *
 * @typedef {number} Hashrate
 */
/**
 * @typedef {Object} HashrateEntry
 * @property {Timestamp} timestamp - End of the day the hashrate covers (Unix timestamp), as mempool.space stamps it
 * @property {number} avgHashrate - Average hashrate (H/s)
 */
/**
 * @typedef {Object} HashrateSummary
 * @property {HashrateEntry[]} hashrates
 * @property {DifficultyEntry[]} difficulty
 * @property {number} currentHashrate - Current network hashrate over the trailing 7 days (H/s)
 * @property {number} currentDifficulty
 */
/**
 * @typedef {Object} Health
 * @property {HealthStatus} status
 * @property {("brk")} service
 * @property {ServerVersion} version
 * @property {string} timestamp - Current server time (ISO 8601)
 * @property {string} startedAt - Server start time (ISO 8601)
 * @property {number} uptimeSeconds - Uptime in seconds
 * @property {Height} indexedHeight - Height of the last indexed block
 * @property {Height} computedHeight - Height of the last computed block (series)
 * @property {Height} tipHeight - Height of the Bitcoin node's chain tip; `/health` reports `indexed_height` instead, without
asking the node
 * @property {Height} blocksBehind - Number of blocks behind the tip
 * @property {string} lastIndexedAt - Human-readable timestamp of the last indexed block (ISO 8601)
 * @property {Timestamp} lastIndexedAtUnix - Unix timestamp of the last indexed block
 */
/**
 * The server's health: `healthy` whenever it answers.
 *
 * @typedef {("healthy")} HealthStatus
 */
/**
 * A block height: how many blocks came before it.
 *
 * @typedef {number} Height
 */
/**
 * A confirmed point of the chain: a block height (`840000`) or a UTC calendar day (`YYYY-MM-DD`).
 *
 * @typedef {(Height|Date)} HeightOrDate
 */
/**
 * @typedef {Object} HeightOrDateParam
 * @property {HeightOrDate} point - Confirmed block height (`840000`) or UTC calendar day (`YYYY-MM-DD`)
 */
/**
 * @typedef {Object} HeightParam
 * @property {Height} height
 */
/**
 * Hex-encoded data as a JSON string.
 *
 * @typedef {string} Hex
 */
/**
 * @typedef {Object} HistoricalPrice
 * @property {PricePoint[]} prices
 * @property {ExchangeRates} exchangeRates - Exchange rates (currently empty)
 */
/**
 * Historical price query parameters, as mempool.space's
 *
 * @typedef {Object} HistoricalPriceParam
 * @property {(Timestamp|null)=} timestamp - UNIX timestamp: the latest close at or before it
 * @property {(Currency|null)=} currency - Quote currency: USD, the only one served
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
 * Aggregation dimension for querying series. Includes time-based (date, week, month, year), block-based (height, tx_index), and address/output type indexes.
 *
 * @typedef {("minute10"|"minute30"|"hour1"|"hour4"|"hour12"|"day1"|"day3"|"week1"|"month1"|"month3"|"month6"|"year1"|"year10"|"halving_epoch"|"difficulty_epoch"|"height"|"tx_index"|"txin_index"|"txout_index"|"empty_script_output_index"|"op_return_output_index"|"p2a_address_index"|"p2ms_output_index"|"p2pk33_address_index"|"p2pk65_address_index"|"p2pkh_address_index"|"p2sh_address_index"|"p2tr_address_index"|"p2wpkh_address_index"|"p2wsh_address_index"|"unknown_script_output_index")} Index
 */
/**
 * A JSON Schema type: what a series' values are on the wire.
 *
 * @typedef {("null"|"boolean"|"integer"|"number"|"string"|"array"|"object")} JsonType
 */
/**
 * Maximum number of results to return; each parameter states what omitting it means.
 *
 * @typedef {number} Limit
 */
/**
 * Block info in a mempool.space like format for fee estimation.
 *
 * @typedef {Object} MempoolBlock
 * @property {number} blockSize - Sum of the transactions' serialized sizes in bytes (witness + non-witness), without header
or coinbase.
 * @property {number} blockVSize - Sum of the transactions' virtual sizes in vbytes
 * @property {number} nTx - Number of transactions in the projected block
 * @property {Sats} totalFees - Total fees in satoshis
 * @property {FeeRate} medianFee - Median fee rate in sat/vB
 * @property {FeeRate[]} feeRange - Vsize-weighted percentiles of the transactions' CPFP chunk fee rates, in sat/vB:
[0, 10, 25, 50, 75, 90, 100] for the first block, [5, 10, 25, 50, 75, 90, 95] for the
later projected ones
 */
/**
 * A projected mempool block with its transactions: block 0 is Bitcoin Core's
 * `getblocktemplate` selection. Returned by `GET /api/mempool-blocks/{index}`.
 *
 * @typedef {Object} MempoolBlockContent
 * @property {MempoolBlockHash} hash - Pass as `since` to `GET /api/mempool-blocks/{index}/diff/{since}` to follow changes.
 * @property {MempoolBlock} stats
 * @property {Transaction[]} transactions - Full transaction bodies in `getblocktemplate` order.
 */
/**
 * A projected mempool block's changes since a prior copy identified by `since`. Returned by
 * `GET /api/mempool-blocks/{index}/diff/{since}`.
 *
 * `removed` lists txids no longer present. A changed body can be emitted as `New` without
 * removing its txid; absence from every retained run alone does not imply removal.
 *
 * @typedef {Object} MempoolBlockDiff
 * @property {MempoolBlockHash} hash - The block's current hash. Use as `since` on the next diff call.
 * @property {MempoolBlockHash} since - Echoed prior hash the diff was computed against.
 * @property {MempoolBlock} stats - Stats of the new block; they can change while every transaction is retained.
 * @property {MempoolBlockDiffEntry[]} order - The new block in order. Each entry is either a run of the prior copy's transactions,
`[start, count]`, or a full transaction body.
 * @property {Txid[]} removed - Txids that left the projected next block since `since`
(confirmed, evicted, replaced, or pushed past block 0).
 */
/**
 * One stretch of the new block in a `MempoolBlockDiff`.
 *
 * Untagged on the wire so JSON type disambiguates the variants:
 * - `Retained(run)` serializes as `[start, count]`: the prior block's transactions
 *   `start..start + count` (which the client cached at `since`), kept in order.
 * - `New(tx)` serializes as a transaction object: a body that was new or changed since the
 *   prior block and takes this position.
 *
 * @typedef {(RetainedRun|Transaction)} MempoolBlockDiffEntry
 */
/**
 * The block and the content token a client holds for it.
 *
 * @typedef {Object} MempoolBlockDiffParam
 * @property {MempoolBlockIndex} index
 * @property {MempoolBlockHash} since - The `hash` of the client's copy, from the block or a previous diff
 */
/**
 * Content hash of a projected mempool block, including its statistics and complete
 * transaction bodies. Opaque token, distinct from HTTP ETag formatting: pass back to
 * `GET /api/mempool-blocks/{index}/diff/{since}` to follow changes.
 * A decimal string, since JavaScript numbers can't hold every 64-bit value.
 *
 * @typedef {string} MempoolBlockHash
 */
/**
 * A projected mempool block's position, as `/api/v1/fees/mempool-blocks` lists them: 0 is the
 * next block, Bitcoin Core's `getblocktemplate` selection.
 *
 * @typedef {number} MempoolBlockIndex
 */
/**
 * @typedef {Object} MempoolBlockParam
 * @property {MempoolBlockIndex} index
 */
/**
 * Mempool statistics with incrementally maintained fee histogram.
 *
 * @typedef {Object} MempoolInfo
 * @property {number} count - Number of transactions in the mempool
 * @property {VSize} vsize - Total virtual size of all transactions in the mempool (vbytes)
 * @property {Sats} totalFee - Total fees of all transactions in the mempool (satoshis)
 * @property {number[][]} feeHistogram - Fee histogram as Esplora bins it: `[[fee_rate, vsize], ...]` by descending fee rate, each
entry the vsize of the transactions paying from its fee rate up to the previous entry's,
a bin closing at the first fee rate change past 50,000 vB
 */
/**
 * Simplified mempool transaction for the `/api/mempool/recent` endpoint.
 *
 * @typedef {Object} MempoolRecentTx
 * @property {Txid} txid
 * @property {Sats} fee - Transaction fee (sats)
 * @property {VSize} vsize - Virtual size (vbytes)
 * @property {Sats} value - Total output value (sats)
 */
/**
 * A merkle tree node: a double-SHA256 hash, hex-encoded in Bitcoin's display byte order.
 *
 * @typedef {string} MerkleNode
 */
/**
 * Merkle inclusion proof for a transaction
 *
 * @typedef {Object} MerkleProof
 * @property {Height} blockHeight
 * @property {MerkleNode[]} merkle - Merkle proof path, from the transaction's sibling up
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
 * A multiple: one value over another of its kind, 1 at parity (MVRV, SOPR, a price over its
 * average).
 *
 * @typedef {number} Multiple
 */
/**
 * Open, high, low and close prices in US dollars: [open, high, low, close].
 *
 * @typedef {Dollars[]} OHLCDollars
 */
/**
 * Protocol or format detected in an OP_RETURN output.
 *
 * @typedef {("runes"|"witness_commitment"|"veriblock"|"omni"|"stacks"|"blockstack"|"colu"|"open_assets"|"komodo"|"coinspark"|"poet"|"docproof"|"factom"|"eternity_wall"|"memo"|"bitproof"|"ascribe"|"stampery"|"bare_hash"|"text"|"empty"|"unknown")} OpReturnKind
 */
/**
 * Index of an OP_RETURN output.
 *
 * @typedef {TypeIndex} OpReturnOutputIndex
 */
/**
 * The output a transaction input spends: its transaction index and output position, written as
 * `{"tx_index": N, "vout": M}` (coinbase inputs: `{"tx_index": 4294967295, "vout": 4294967295}`).
 *
 * @typedef {Object} OutPoint
 * @property {TxIndex} txIndex
 * @property {Vout} vout
 */
/**
 * An output's script type (P2PKH, P2WPKH, P2SH, P2TR…).
 *
 * @typedef {("p2pk65"|"p2pk33"|"p2pkh"|"p2ms"|"p2sh"|"op_return"|"p2wpkh"|"p2wsh"|"p2tr"|"p2a"|"empty_script"|"unknown_script")} OutputType
 */
/**
 * Output type names used by Esplora and mempool.space.
 *
 * @typedef {("p2pk"|"p2pkh"|"multisig"|"p2sh"|"op_return"|"v0_p2wpkh"|"v0_p2wsh"|"v1_p2tr"|"anchor"|"empty"|"unknown")} OutputTypeNormalized
 */
/**
 * Index of a P2A (pay-to-anchor) address.
 *
 * @typedef {TypeIndex} P2AAddressIndex
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
 * @typedef {TypeIndex} P2PK33AddressIndex
 */
/**
 * A compressed (33-byte) public key.
 *
 * @typedef {string} P2PK33Bytes
 */
/**
 * Index of a P2PK address with an uncompressed (65-byte) public key.
 *
 * @typedef {TypeIndex} P2PK65AddressIndex
 */
/**
 * An uncompressed (65-byte) public key.
 *
 * @typedef {string} P2PK65Bytes
 */
/**
 * Index of a P2PKH address.
 *
 * @typedef {TypeIndex} P2PKHAddressIndex
 */
/**
 * The 20-byte public key hash of a P2PKH output.
 *
 * @typedef {string} P2PKHBytes
 */
/**
 * Index of a P2SH address.
 *
 * @typedef {TypeIndex} P2SHAddressIndex
 */
/**
 * The 20-byte script hash of a P2SH output.
 *
 * @typedef {string} P2SHBytes
 */
/**
 * Index of a P2TR (taproot) address.
 *
 * @typedef {TypeIndex} P2TRAddressIndex
 */
/**
 * The 32-byte output key of a P2TR output.
 *
 * @typedef {string} P2TRBytes
 */
/**
 * Index of a P2WPKH address.
 *
 * @typedef {TypeIndex} P2WPKHAddressIndex
 */
/**
 * The 20-byte public key hash of a P2WPKH output.
 *
 * @typedef {string} P2WPKHBytes
 */
/**
 * Index of a P2WSH address.
 *
 * @typedef {TypeIndex} P2WSHAddressIndex
 */
/**
 * The 32-byte script hash of a P2WSH output.
 *
 * @typedef {string} P2WSHBytes
 */
/**
 * A paginated list of available series names (up to 1,000 per page)
 *
 * @typedef {Object} PaginatedSeries
 * @property {number} currentPage - Current page number (0-indexed)
 * @property {number} maxPage - Maximum valid page index (0-indexed)
 * @property {number} totalCount
 * @property {number} perPage
 * @property {boolean} hasMore
 * @property {SeriesName[]} series
 */
/**
 * @typedef {Object} Pagination
 * @property {?number=} page - Zero-based page number (default: 0)
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
 * - `all`: Total blocks mined (all time)
 * - `24h`: Blocks mined in last 24 hours
 * - `1w`: Blocks mined in last week
 * @typedef {{ all: number, "24h": number, "1w": number }} PoolBlockCounts
 */
/**
 * Pool's share of total blocks for different time periods
 *
 * - `all`: Share of all blocks (0.0 - 1.0)
 * - `24h`: Share of blocks in last 24 hours (0.0 - 1.0)
 * - `1w`: Share of blocks in last week (0.0 - 1.0)
 * @typedef {{ all: number, "24h": number, "1w": number }} PoolBlockShares
 */
/**
 * @typedef {Object} PoolDetail
 * @property {PoolDetailInfo} pool
 * @property {PoolBlockCounts} blockCount
 * @property {PoolBlockShares} blockShare
 * @property {number} estimatedHashrate - Estimated hashrate based on blocks mined (H/s)
 * @property {?number=} reportedHashrate - Self-reported hashrate (if available, H/s)
 * @property {?number=} avgBlockHealth - Mean health of the pool's blocks, from mempool.space's own audits (always null: it needs
the mempool as it was)
 * @property {string} totalReward - Total reward earned by this pool: coinbase output value of every block it mined (sats),
as a decimal string
 */
/**
 * @typedef {Object} PoolDetailInfo
 * @property {number} id - One-based pool ID, matching mempool.space's `id`
 * @property {string} name
 * @property {string} link - Pool website URL
 * @property {Address[]} addresses - Known payout addresses
 * @property {string[]} regexes - Coinbase tags, matched as case-insensitive substrings
 * @property {PoolSlug} slug - URL-friendly pool identifier
 * @property {number} uniqueId - Zero-based pool ID, matching mempool.space's `unique_id`
 */
/**
 * @typedef {Object} PoolHashrateEntry
 * @property {Timestamp} timestamp - Unix timestamp
 * @property {number} avgHashrate - Average hashrate (H/s)
 * @property {number} share - Pool's share of total network hashrate (0.0 - 1.0)
 * @property {string} poolName
 */
/**
 * @typedef {Object} PoolInfo
 * @property {string} name
 * @property {PoolSlug} slug - URL-friendly pool identifier
 * @property {number} uniqueId - Zero-based pool ID, matching mempool.space's `unique_id`
 */
/**
 * A mining pool's identifier, as written in URLs.
 *
 * @typedef {("unknown"|"blockfills"|"ultimuspool"|"terrapool"|"luxor"|"1thash"|"btccom"|"bitfarms"|"huobipool"|"wayicn"|"canoepool"|"btctop"|"bitcoincom"|"175btc"|"gbminers"|"axbt"|"asicminer"|"bitminter"|"bitcoinrussia"|"btcserv"|"simplecoinus"|"btcguild"|"eligius"|"ozcoin"|"eclipsemc"|"maxbtc"|"triplemining"|"coinlab"|"50btc"|"ghashio"|"stminingcorp"|"bitparking"|"mmpool"|"polmine"|"kncminer"|"bitalo"|"f2pool"|"hhtt"|"megabigpower"|"mtred"|"nmcbit"|"yourbtcnet"|"givemecoins"|"braiinspool"|"antpool"|"multicoinco"|"bcpoolio"|"cointerra"|"kanopool"|"solock"|"ckpool"|"nicehash"|"bitclub"|"bitcoinaffiliatenetwork"|"btcc"|"bwpool"|"exxbw"|"bitsolo"|"bitfury"|"21inc"|"digitalbtc"|"8baochi"|"mybtccoinpool"|"tbdice"|"hashpool"|"nexious"|"bravomining"|"hotpool"|"okexpool"|"bcmonster"|"1hash"|"bixin"|"tatmaspool"|"viabtc"|"connectbtc"|"batpool"|"waterhole"|"dcexploration"|"dcex"|"btpool"|"58coin"|"bitcoinindia"|"shawnp0wers"|"phashio"|"rigpool"|"haozhuzhu"|"7pool"|"miningkings"|"hashbx"|"dpool"|"rawpool"|"haominer"|"helix"|"bitcoinukraine"|"poolin"|"secretsuperstar"|"tigerpoolnet"|"sigmapoolcom"|"okpooltop"|"hummerpool"|"tangpool"|"bytepool"|"spiderpool"|"novablock"|"miningcity"|"binancepool"|"minerium"|"lubiancom"|"okkong"|"aaopool"|"emcdpool"|"foundryusa"|"sbicrypto"|"arkpool"|"purebtccom"|"marapool"|"kucoinpool"|"entrustcharitypool"|"okminer"|"titan"|"pegapool"|"btcnuggets"|"cloudhashing"|"digitalxmintsy"|"telco214"|"btcpoolparty"|"multipool"|"transactioncoinmining"|"btcdig"|"trickysbtcpool"|"btcmp"|"eobot"|"unomp"|"patels"|"gogreenlight"|"bitcoinindiapool"|"ekanembtc"|"canoe"|"tiger"|"1m1x"|"zulupool"|"secpool"|"ocean"|"whitepool"|"wiz"|"wk057"|"futurebitapollosolo"|"carbonnegative"|"portlandhodl"|"phoenix"|"neopool"|"maxipool"|"bitfufupool"|"gdpool"|"miningdutch"|"publicpool"|"miningsquared"|"innopolistech"|"btclab"|"parasite"|"redrockpool"|"est3lar"|"braiinssolo"|"solopoolcom"|"noderunners"|"dmnd")} PoolSlug
 */
/**
 * @typedef {Object} PoolSlugAndHeightParam
 * @property {PoolSlug} slug
 * @property {Height} height
 */
/**
 * @typedef {Object} PoolSlugParam
 * @property {PoolSlug} slug
 */
/**
 * Mining pool with block statistics for a time period
 *
 * @typedef {Object} PoolStats
 * @property {number} poolId - One-based pool ID, matching mempool.space's `id`
 * @property {string} name
 * @property {string} link - Pool website URL
 * @property {number} blockCount
 * @property {number} rank - Pool ranking by block count (1 = most blocks)
 * @property {number} emptyBlocks - Number of empty (coinbase-only) blocks mined in the time period
 * @property {PoolSlug} slug - URL-friendly pool identifier
 * @property {?number=} avgMatchRate - Mean share of the expected block template the pool's blocks matched, from mempool.space's
own audits (always null: it needs the mempool as it was)
 * @property {?string=} avgFeeDelta - Mean relative difference of the pool's fees from the expected template's, a decimal
string at mempool.space, from its own audits (always null: it needs the mempool as it was)
 * @property {number} poolUniqueId - Zero-based pool ID, matching mempool.space's `unique_id`
 * @property {number} share - Pool's share of total blocks (0.0 - 1.0)
 */
/**
 * @typedef {Object} PoolsSummary
 * @property {PoolStats[]} pools - List of pools sorted by block count descending
 * @property {number} blockCount - Total blocks in the time period
 * @property {number} lastEstimatedHashrate - Estimated network hashrate (H/s)
 * @property {number} lastEstimatedHashrate3d - Estimated network hashrate over last 3 days (H/s)
 * @property {number} lastEstimatedHashrate1w - Estimated network hashrate over last 1 week (H/s)
 */
/**
 * Recommended fee rates in sat/vB, to the thousandth (`/api/v1/fees/precise`)
 *
 * @typedef {Object} PreciseRecommendedFees
 * @property {FeeRate} fastestFee - Fee rate for fastest confirmation (next block)
 * @property {FeeRate} halfHourFee - Fee rate for confirmation within ~30 minutes (second projected block)
 * @property {FeeRate} hourFee - Fee rate for confirmation within ~1 hour (third projected block)
 * @property {FeeRate} economyFee - Smoothed, fullness-adjusted third-block fee estimate, bounded by `minimumFee` and twice
it; `minimumFee` when no third projected block exists
 * @property {FeeRate} minimumFee - The node's current mempool minimum fee rate, rounded up to 0.001 sat/vB (at least 0.001)
 */
/**
 * A BTC/USD price at a time: the current price (`/api/v1/prices`) or one historical close
 *
 * @typedef {Object} PricePoint
 * @property {Timestamp} time - Unix timestamp
 * @property {Dollars} USD - BTC/USD price
 */
/**
 * A positional index, YYYY-MM-DD date, or ISO 8601 timestamp.
 *
 * @typedef {(number|string|string)} RangeIndex
 */
/**
 * A dimensionless ratio: a quotient or share, read as a plain number.
 *
 * @typedef {number} Ratio
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
 * Recommended fee rates in whole sat/vB (`/api/v1/fees/recommended`)
 *
 * @typedef {Object} RecommendedFees
 * @property {number} fastestFee - Fee rate for fastest confirmation (next block)
 * @property {number} halfHourFee - Fee rate for confirmation within ~30 minutes (second projected block)
 * @property {number} hourFee - Fee rate for confirmation within ~1 hour (third projected block)
 * @property {number} economyFee - Smoothed, fullness-adjusted third-block fee estimate, bounded by `minimumFee` and twice
it; `minimumFee` when no third projected block exists
 * @property {number} minimumFee - The node's current mempool minimum fee rate, rounded up to a whole sat/vB (at least 1)
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
 * Consecutive transactions kept from the prior block, `[start, count]`: its positions
 * `start..start + count`, in order.
 *
 * @typedef {[number, number]} RetainedRun
 */
/**
 * Block reward statistics over a range of blocks
 *
 * @typedef {Object} RewardStats
 * @property {Height} startBlock - First block in the range
 * @property {Height} endBlock - Last block in the range
 * @property {string} totalReward - Total coinbase rewards (subsidy + fees) in sats
 * @property {string} totalFee - Total transaction fees in sats
 * @property {string} totalTx - Total transactions, coinbase included
 */
/**
 * An amount in satoshis (100,000,000 to a bitcoin).
 *
 * @typedef {number} Sats
 */
/**
 * A signed amount in satoshis.
 *
 * @typedef {number} SatsSigned
 */
/**
 * A signed model score; undefined (null) where the model has no calibration yet.
 *
 * @typedef {number} Score
 */
/**
 * @typedef {Object} SearchQuery
 * @property {string} q - Words to find in series names and descriptions (`realized price`, `mvrv`), in any order
 * @property {Limit=} limit - Maximum number of results, 100 when omitted
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
 * @typedef {Object} SeriesCount
 * @property {number} distinct - Number of unique series available (e.g., realized_price, market_cap)
 * @property {number} total - Total number of series-index combinations across all timeframes
 * @property {number} lazy - Number of lazy (computed on-the-fly) series-index combinations
 * @property {number} stored - Number of eager (stored on disk) series-index combinations
 */
/**
 * @typedef {Object} SeriesInfo
 * @property {?string=} description - Human-readable metric definition, when documented
 * @property {Index[]} indexes
 * @property {Index[]} nullable - Indexes whose values can be null: a missing value (e.g. a period without blocks) or an
undefined one (e.g. NaN)
 * @property {string} type - Value type (e.g. "Ratio", "Sats", "Cents")
 * @property {?string=} unit - What the value type measures, when documented (e.g. "A duration in days.")
 */
/**
 * Series leaf with JSON Schema for client generation.
 *
 * @typedef {Object} SeriesLeafWithSchema
 * @property {SeriesName} name
 * @property {string} kind - The value type, by its schema's name (e.g., "Sats", "Ratio").
 * @property {Index[]} indexes
 * @property {Index[]} nullable - Indexes whose values can be null: a missing value (e.g. a period without blocks) or an
undefined one (e.g. NaN).
 * @property {?string=} description - Human-readable metric definition, when documented.
 * @property {JsonType} type - What the values are on the wire.
 */
/**
 * Comma-separated list of series names
 *
 * Deserialization permits at most 32 normalized names and 2,048 decoded input
 * string bytes. For arrays, the byte budget is shared by their string values.
 *
 * @typedef {string} SeriesList
 */
/** @typedef {string} SeriesName */
/**
 * @typedef {Object} SeriesNameWithIndex
 * @property {SeriesName} series
 * @property {Index} index
 */
/**
 * @typedef {Object} SeriesParam
 * @property {SeriesName} series
 */
/**
 * @typedef {Object} SeriesSelection
 * @property {SeriesList} series
 * @property {Index} index
 * @property {(RangeIndex|null)=} start - Inclusive start: integer index, date (YYYY-MM-DD), or timestamp (ISO 8601). Negative integers count from end.
 * @property {(RangeIndex|null)=} end - Exclusive end: integer index, date (YYYY-MM-DD), or timestamp (ISO 8601). Negative integers count from end.
 * @property {(Limit|null)=} limit - Maximum number of values to return (ignored if `end` is set); omitted, the rest of the range.
 * @property {Format=} format
 */
/**
 * The server's version: its crate's, in SemVer (`0.12.2`; a pre-release, `0.13.0-beta.1`).
 *
 * @typedef {string} ServerVersion
 */
/**
 * BIP-141 signature-operation cost: legacy and P2SH-redeem sigops count four and witness sigops
 * one. Sums over many transactions or blocks use the full 64 bits.
 *
 * @typedef {number} SigOps
 */
/**
 * @typedef {Object} SyncStatus
 * @property {Height} indexedHeight - Height of the last indexed block
 * @property {Height} computedHeight - Height of the last computed block (series)
 * @property {Height} tipHeight - Height of the Bitcoin node's chain tip; `/health` reports `indexed_height` instead, without
asking the node
 * @property {Height} blocksBehind - Number of blocks behind the tip
 * @property {string} lastIndexedAt - Human-readable timestamp of the last indexed block (ISO 8601)
 * @property {Timestamp} lastIndexedAtUnix - Unix timestamp of the last indexed block
 */
/**
 * Lookback window for mining statistics.
 *
 * @typedef {("24h"|"3d"|"1w"|"1m"|"3m"|"6m"|"1y"|"2y"|"3y"|"4y"|"all")} TimePeriod
 */
/**
 * @typedef {Object} TimePeriodParam
 * @property {TimePeriod} timePeriod - Lookback period: `24h`, `3d`, `1w`, `1m`, `3m`, `6m`, `1y`, `2y`, `3y`, `4y` or `all`.
 */
/**
 * A Unix timestamp, in seconds.
 *
 * @typedef {number} Timestamp
 */
/**
 * @typedef {Object} TimestampParam
 * @property {Timestamp} timestamp
 */
/**
 * Transaction information compatible with mempool.space API format
 *
 * @typedef {Object} Transaction
 * @property {(TxIndex|null)=} index - Internal transaction index (brk-specific, not in mempool.space)
 * @property {Txid} txid
 * @property {TxVersionRaw} version - Transaction version (the protocol's 32 bits read unsigned, as Esplora reports them; may contain non-standard values in coinbase txs)
 * @property {RawLockTime} locktime
 * @property {TxIn[]} vin - Transaction inputs
 * @property {TxOut[]} vout - Transaction outputs
 * @property {number} size - Transaction size in bytes
 * @property {Weight} weight
 * @property {SigOps} sigops - BIP-141 signature-operation cost
 * @property {Sats} fee - Transaction fee in satoshis
 * @property {TxStatus} status
 */
/** @typedef {({ [key: string]: TreeNode }|SeriesLeafWithSchema)} TreeNode */
/**
 * @typedef {Object} TxIn
 * @property {Txid} txid - Transaction ID of the output being spent
 * @property {Vout} vout - Output index being spent (coinbase: 4294967295)
 * @property {(TxOut|null)} prevout - The output being spent
 * @property {Hex} scriptsig - Signature script in hex (empty for native SegWit inputs)
 * @property {string} scriptsigAsm - Signature script in assembly format
 * @property {Hex[]=} witness - Witness data (stack items, present for SegWit inputs; hex-encoded on the wire)
 * @property {boolean} isCoinbase
 * @property {number} sequence
 * @property {string=} innerRedeemscriptAsm - Inner redeemscript in assembly (P2SH inputs)
 * @property {string=} innerWitnessscriptAsm - Inner witnessscript in assembly (P2WSH and P2SH-P2WSH: the witness script; P2TR script path: the leaf script)
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
 * @typedef {Object} TxIndexParam
 * @property {TxIndex} index - Global transaction index across the chain
 */
/**
 * @typedef {Object} TxOut
 * @property {Hex} scriptpubkey - Script pubkey (locking script), encoded as hexadecimal.
 * @property {string} scriptpubkeyAsm - Script pubkey in assembly format.
 * @property {OutputTypeNormalized} scriptpubkeyType - Esplora/mempool.space script type.
 * @property {Address=} scriptpubkeyAddress - Bitcoin address, omitted for scripts without an address (P2PK included, as Esplora).
 * @property {Sats} value - Value of the output in satoshis.
 */
/**
 * Index of a transaction output.
 *
 * @typedef {number} TxOutIndex
 */
/**
 * @typedef {Object} TxOutspend
 * @property {boolean} spent
 * @property {(Txid|null)=} txid - Transaction ID of the spending transaction (only present if spent)
 * @property {(Vin|null)=} vin - Input index in the spending transaction (only present if spent)
 * @property {(TxStatus|null)=} status - Status of the spending transaction (only present if spent)
 */
/**
 * @typedef {Object} TxStatus
 * @property {boolean} confirmed
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
 * Raw transaction version: the protocol's 32 bits read unsigned, as Esplora reports them.
 * Unlike TxVersion (u8, indexed), this preserves non-standard values
 * used in coinbase txs for miner signaling/branding.
 *
 * @typedef {number} TxVersionRaw
 */
/**
 * A transaction ID: its hash.
 *
 * @typedef {string} Txid
 */
/**
 * @typedef {Object} TxidParam
 * @property {Txid} txid
 */
/**
 * @typedef {Object} TxidVout
 * @property {Txid} txid
 * @property {Vout} vout - Output index
 */
/**
 * Transaction IDs as repeated `txId[]` query parameters.
 *
 * @typedef {Object} TxidsParam
 * @property {Txid[]} txId - Transaction IDs to look up (max 100 per request).
 */
/**
 * An index within its type (0 for the first P2WPKH address).
 *
 * @typedef {number} TypeIndex
 */
/**
 * Index of an output with an unrecognized script.
 *
 * @typedef {TypeIndex} UnknownScriptOutputIndex
 */
/**
 * UTXO Realized Price Distribution for a cohort at a specific block.
 *
 * Supply is grouped by the price at the block in which each UTXO was last moved; each bucket
 * holds its supply, realized cap and unrealized P&L. The same block header as `UtxoSet`
 * (`height`, `hash`, `date`), then its own fields and its buckets as an object of columns.
 *
 * @typedef {Object} Urpd
 * @property {Height} height - Exact published block represented by this distribution.
 * @property {BlockHash} hash - Hash of block `height`.
 * @property {Date} date - UTC date of the represented block's monotonic timestamp.
 * @property {UrpdCohort} cohort
 * @property {UrpdWeight} weight
 * @property {UrpdAggregation} aggregation
 * @property {Dollars} close - Price at `height`, in USD. Anchor for `unrealized_pnl`.
 * @property {Bitcoin} supply - Sum of `buckets.supply`, in BTC.
 * @property {UrpdBuckets} buckets
 */
/**
 * Aggregation strategy for URPD buckets.
 * Options: raw (no aggregation), lin200/lin500/lin1000 (linear $200/$500/$1000),
 * log10/log50/log100/log200/log500/log1000/log2000 (logarithmic with 10/50/100/200/500/1000/2000 buckets per decade).
 *
 * @typedef {("raw"|"lin200"|"lin500"|"lin1000"|"log10"|"log50"|"log100"|"log200"|"log500"|"log1000"|"log2000")} UrpdAggregation
 */
/**
 * Columnar buckets in ascending price order: entry `i` of each column is bucket `i`.
 *
 * @typedef {Object} UrpdBuckets
 * @property {Dollars[]} price - Each bucket's price, in USD: its lower bound when aggregated, and for `raw` the price its
coins' creation prices round to (four significant digits of the price in cents).
 * @property {Bitcoin[]} supply - Supply held with a last-move price inside each bucket, in BTC. With a `wakefulness` or
`mobility` weight, the weighted supply, floored to whole satoshis at each grid price.
 * @property {Dollars[]} realizedCap - Realized cap contribution in USD: `price * supply` summed over the coins in each bucket,
each coin's creation price taken on the price grid (the `raw` bucket price, within 0.05%
of the exact price), rounded to the cent: a bucket worth under half a cent shows 0. The
buckets therefore need not sum to the cohort's realized cap series.
 * @property {Dollars[]} unrealizedPnl - Unrealized P&L in USD at the represented block: `close * supply` minus each bucket's grid
capital, rounded to the cent once. Can be negative.
 */
/**
 * A URPD cohort, by its series' prefix: an age aggregate (`all`, `sth`, `lth`, `age_under_4m`, ...) or an age band (`age_under_1h`, `age_1h_to_1d`, ...).
 *
 * @typedef {("all"|"sth"|"lth"|"age_under_4m"|"age_under_6m"|"age_over_4m"|"age_over_6m"|"age_under_1h"|"age_1h_to_1d"|"age_1d_to_1w"|"age_1w_to_1m"|"age_1m_to_2m"|"age_2m_to_3m"|"age_3m_to_4m"|"age_4m_to_5m"|"age_5m_to_6m"|"age_6m_to_9m"|"age_9m_to_1y"|"age_1y_to_18m"|"age_18m_to_2y"|"age_2y_to_3y"|"age_3y_to_4y"|"age_4y_to_5y"|"age_5y_to_6y"|"age_6y_to_7y"|"age_7y_to_8y"|"age_8y_to_10y"|"age_10y_to_12y"|"age_12y_to_15y"|"age_over_15y")} UrpdCohort
 */
/**
 * @typedef {Object} UrpdCohortParam
 * @property {UrpdCohort} cohort
 */
/**
 * A URPD cohort and the confirmed point to read it at.
 *
 * @typedef {Object} UrpdParams
 * @property {UrpdCohort} cohort
 * @property {HeightOrDate} point - Confirmed block height (`840000`) or UTC calendar day (`YYYY-MM-DD`)
 */
/**
 * @typedef {Object} UrpdQuery
 * @property {UrpdAggregation=} agg - Aggregation strategy. Default: raw (no aggregation).
 * @property {UrpdWeight=} weight - Supply weighting. Default: none (unweighted).
 */
/**
 * The weight applied to a URPD's age bands: `none` (every satoshi counts once), `wakefulness` (each band
 * weighted by the share of its coin days already consumed, Cointime's; the distribution behind the
 * `*_awake_*` series) or `mobility` (each band weighted by the estimated probability its coins are ever
 * spent, Coinflow's; behind `*_mobile_*`).
 *
 * @typedef {("none"|"wakefulness"|"mobility")} UrpdWeight
 */
/**
 * Unspent transaction output
 *
 * @typedef {Object} Utxo
 * @property {Txid} txid
 * @property {Vout} vout
 * @property {TxStatus} status
 * @property {Sats} value - Output value in satoshis
 */
/**
 * Columnar counts and supplies by creation height.
 *
 * @typedef {Object} UtxoChanges
 * @property {Height[]} height - Creation height of each row.
 * @property {number[]} count - Number of outputs in each row.
 * @property {Bitcoin[]} supply - BTC held by the outputs in each row.
 */
/**
 * Columnar unspent amounts indexed by creation height.
 *
 * @typedef {Object} UtxoOrigins
 * @property {number[]} count - Number of unspent outputs per creation height.
 * @property {Bitcoin[]} supply - BTC held by the unspent outputs per creation height.
 */
/**
 * The UTXO set after a block, grouped by the height of the block that created each output.
 *
 * The set excludes `OP_RETURN` outputs and the genesis coinbase, which are unspendable, and the
 * coinbases of blocks 91812 and 91722 once duplicates at 91842 and 91880 overwrite them; it
 * includes zero-value outputs. The set after block `first - 1` plus a diff's `created` minus its
 * `spent` gives the set after block `last`; rounding each supply to 8 decimals keeps it exact.
 *
 * @typedef {Object} UtxoSet
 * @property {Height} height
 * @property {BlockHash} hash
 * @property {Date} date - UTC date of the represented block's monotonic timestamp (the running maximum of block
times).
 * @property {number} count - Number of unspent outputs.
 * @property {Bitcoin} supply - BTC held by the unspent outputs.
 * @property {UtxoOrigins} origins - Unspent outputs by creation height: entry `i` is what remains of block `i`'s outputs.
 */
/**
 * What blocks `first` through `last` changed in the UTXO set.
 *
 * @typedef {Object} UtxoSetDiff
 * @property {Height} first - First block of the diff; it applies to the set after block `first - 1` (the empty set for
block 0).
 * @property {Height} last - Last block of the diff, described by `hash` and `date`.
 * @property {BlockHash} hash - Hash of block `last`.
 * @property {Date} date - UTC date of block `last`'s monotonic timestamp.
 * @property {UtxoChanges} created - Outputs created, one row per block, including those spent again within the range (which
`spent` lists too).
 * @property {UtxoChanges} spent - Outputs removed, one row per creation height in ascending order: spent outputs, plus the
coinbase outputs of blocks 91812 and 91722, which duplicates at 91842 and 91880 overwrote.
 */
/**
 * Virtual size in vbytes (weight / 4, rounded up). Max block vsize is ~1,000,000 vB.
 *
 * @typedef {number} VSize
 */
/**
 * @typedef {Object} ValidateAddressParam
 * @property {string} address - Bitcoin address to validate: 2 to 130 letters and digits (a P2PK public key in hex is 130);
anything else answers 400, as mempool.space does
 */
/**
 * Schema or computation version of a stored vector: when it changes, the data is reset and
 * recomputed.
 *
 * @typedef {number} Version
 */
/**
 * Input index in the spending transaction
 *
 * @typedef {number} Vin
 */
/**
 * Index of the output being spent in the previous transaction (coinbase inputs: 4294967295)
 *
 * @typedef {number} Vout
 */
/**
 * Index of a week.
 *
 * @typedef {number} Week1
 */
/**
 * Weight in weight units (WU): non-witness bytes count four and witness bytes one. A block holds
 * at most 4,000,000 WU; sums over many blocks use the full 64 bits.
 *
 * @typedef {number} Weight
 */
/**
 * A mean weight in weight units.
 *
 * @typedef {number} WeightFract
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

/** @typedef {'minute10'|'minute30'|'hour1'|'hour4'|'hour12'|'day1'|'day3'|'week1'|'month1'|'month3'|'month6'|'year1'|'year10'} DateIndex */

/**
 * Each date-based index's buckets as [unit, size, phase]: bucket i starts i * size - phase units
 * after 2009-01-01 UTC, except that a month calendar's bucket 0 starts at January 2009.
 * @type {ReadonlyMap<string, readonly ['second' | 'day' | 'month', number, number]>}
 */
const _CALENDARS = new Map([
  ['minute10', ['second', 600, 0]],
  ['minute30', ['second', 1800, 0]],
  ['hour1', ['second', 3600, 0]],
  ['hour4', ['second', 14400, 0]],
  ['hour12', ['second', 43200, 0]],
  ['day1', ['day', 1, 0]],
  ['day3', ['day', 3, 1]],
  ['week1', ['day', 7, 3]],
  ['month1', ['month', 1, 0]],
  ['month3', ['month', 3, 0]],
  ['month6', ['month', 6, 0]],
  ['year1', ['month', 12, 0]],
  ['year10', ['month', 120, 108]],
]);

/**
 * Each address type's payload length in bytes.
 * @type {ReadonlyMap<string, number>}
 */
const _ADDRESS_PAYLOAD_LENGTHS = new Map([
  ['p2pk65', 65],
  ['p2pk33', 33],
  ['p2pkh', 20],
  ['p2sh', 20],
  ['p2wpkh', 20],
  ['p2wsh', 32],
  ['p2tr', 32],
  ['p2a', 2],
]);

/**
 * @typedef {Object} BitviewClientOptions
 * @property {string} baseUrl
 * @property {number} [timeout] - Request timeout in milliseconds
 * @property {string|boolean} [browserCache] - Enable browser Cache API with default name (true), custom name (string), or disable (false). No effect in Node.js. Default: true
 * @property {number|boolean} [memCache] - LRU capacity of the in-memory parsed-response cache, which lets 304 responses skip the JSON parse: true/undefined → 1000, false/0 → disabled
 */

const _isBrowser = typeof window !== 'undefined' && 'caches' in window;
const _runIdle = (/** @type {VoidFunction} */ fn) => (globalThis.requestIdleCallback ?? setTimeout)(fn);
const _defaultBrowserCacheName = '__BRK_CLIENT__';
const _DEFAULT_MEM_CACHE_SIZE = 1000;

/** @template T @typedef {{ etag: string | null, value: T }} _MemEntry */
/** @type {Map<string, string>} */
const _camelNames = new Map();
/** `tx_count` → `txCount`; `avgFee_0` and `24h` stay as they are. @param {string} key */
const _camelName = (key) => {
  let name = _camelNames.get(key);
  if (name === undefined) {
    name = key.replace(/_([a-z])/g, (_, letter) => letter.toUpperCase());
    _camelNames.set(key, name);
  }
  return name;
};

/**
 * A response with camelCase field names, renamed once after parsing. `keep` says where the schema
 * has maps, whose keys are data and stay as sent with everything under them: `true` for the whole
 * value, or the fields (by wire name) that hold one; omitted, every key is a field name.
 * @param {*} value
 * @param {*} [keep]
 */
const _camelKeys = (value, keep) => {
  if (keep === true || value === null || typeof value !== 'object') return value;
  if (Array.isArray(value)) {
    for (let i = 0; i < value.length; i++) value[i] = _camelKeys(value[i], keep);
    return value;
  }
  let renamed = false;
  for (const key in value) {
    if (key.includes('_')) { renamed = true; break; }
  }
  if (!renamed) {
    for (const key in value) value[key] = _camelKeys(value[key], keep?.[key]);
    return value;
  }
  /** @type {Record<string, *>} */
  const out = {};
  for (const key in value) out[_camelName(key)] = _camelKeys(value[key], keep?.[key]);
  return out;
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

// Date conversion, mirroring the server's indexes (`_CALENDARS`): UTC, from 2009-01-01.
const _EPOCH_MS = 1230768000000;
const _UNIT_MS = { second: 1000, day: 86400000 };

/** @param {string} index */
const _calendar = (index) => {
  const calendar = _CALENDARS.get(index);
  if (!calendar) throw new Error(`${index} is not a date-based index`);
  return calendar;
};

/**
 * The UTC start of bucket `i`; throws for an index without a date calendar.
 * @param {Index} index
 * @param {number} i
 * @returns {globalThis.Date}
 */
function indexToDate(index, i) {
  const [unit, size, phase] = _calendar(index);
  if (unit === 'month') return new Date(Date.UTC(2009, i === 0 ? 0 : i * size - phase, 1));
  return new Date(_EPOCH_MS + (i * size - phase) * _UNIT_MS[unit]);
}

/**
 * The floor index (latest index whose date is <= the given date).
 * Throws for dates before the index's first date (2009-01-01; day3's is 2008-12-31, week1's 2008-12-29).
 * @param {Index} index
 * @param {globalThis.Date} d
 * @returns {number}
 */
function dateToIndex(index, d) {
  const ms = d.getTime(), first = indexToDate(index, 0);
  if (!(ms >= first.getTime())) throw new RangeError(`Date is before the first ${index} date (${first.toISOString().slice(0, 10)})`);
  const [unit, size, phase] = _calendar(index);
  if (unit === 'month') return Math.floor(((d.getUTCFullYear() - 2009) * 12 + d.getUTCMonth() + phase) / size);
  return Math.floor((ms - _EPOCH_MS + phase * _UNIT_MS[unit]) / (size * _UNIT_MS[unit]));
}

/**
 * @template T
 * @param {SeriesData<T>} raw
 * @returns {DateSeriesData<T>}
 */
function _wrapSeriesData(raw) {
  const { index, start, end, data } = raw;
  const _dateBased = _CALENDARS.has(index);
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
 * @property {number} version
 * @property {Index} index
 * @property {string} type - Value type (e.g. "Ratio", "Sats", "Cents")
 * @property {number} start - Start index (inclusive)
 * @property {number} end - End index (exclusive)
 * @property {T[]} data - The series data (`null` where a value is missing or undefined)
 * @property {boolean} isDateBased
 * @property {() => number[]} indexes
 * @property {() => number[]} keys - Alias for indexes
 * @property {() => Array<[number, T]>} entries - Get [index, value] pairs
 * @property {() => Map<number, T>} toMap - Convert to Map<index, value>
 */

/** @template T @typedef {SeriesDataBase<T> & Iterable<[number, T]>} SeriesData */

/**
 * @template T
 * @typedef {Object} DateSeriesDataExtras
 * @property {() => globalThis.Date[]} dates - Get dates for each data point
 * @property {() => Array<[globalThis.Date, T]>} dateEntries
 * @property {() => Map<globalThis.Date, T>} toDateMap
 */

/** @template T @typedef {SeriesData<T> & DateSeriesDataExtras<T>} DateSeriesData */
/** @typedef {SeriesData<any>} AnySeriesData */

/** @template T @typedef {PromiseLike<SeriesData<T>>['then']} Thenable */
/** @template T @typedef {PromiseLike<DateSeriesData<T>>['then']} DateThenable */

/**
 * @template T
 * @typedef {Object} SeriesEndpoint
 * @property {(index: number) => RangeBuilder<T>} get
 * @property {(start?: number, end?: number) => RangeBuilder<T>} slice
 * @property {(n: number) => RangeBuilder<T>} first
 * @property {(n: number) => RangeBuilder<T>} last
 * @property {(n: number) => SkippedBuilder<T>} skip - Skip first n items, chain with take()
 * @property {(arg?: SeriesFetchArg<T>, options?: ClientFetchOptions<SeriesData<T>>) => Promise<SeriesData<T>>} fetch - Fetch all data
 * @property {(options?: ClientFetchOptions<string>) => Promise<string>} fetchCsv - Fetch all data as CSV
 * @property {() => Promise<number>} len - Get total number of data points
 * @property {() => Promise<Version>} version
 * @property {Thenable<T>} then - Thenable (await endpoint)
 * @property {string} path
 */

/**
 * @template T
 * @typedef {Object} DateSeriesEndpoint
 * @property {(index: number | globalThis.Date) => DateRangeBuilder<T>} get
 * @property {(start?: number | globalThis.Date, end?: number | globalThis.Date) => DateRangeBuilder<T>} slice
 * @property {(n: number) => DateRangeBuilder<T>} first
 * @property {(n: number) => DateRangeBuilder<T>} last
 * @property {(n: number) => DateSkippedBuilder<T>} skip - Skip first n items, chain with take()
 * @property {(arg?: DateSeriesFetchArg<T>, options?: ClientFetchOptions<DateSeriesData<T>>) => Promise<DateSeriesData<T>>} fetch - Fetch all data
 * @property {(options?: ClientFetchOptions<string>) => Promise<string>} fetchCsv - Fetch all data as CSV
 * @property {() => Promise<number>} len - Get total number of data points
 * @property {() => Promise<Version>} version
 * @property {DateThenable<T>} then - Thenable (await endpoint)
 * @property {string} path
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

/** @template T @typedef {Object} SkippedBuilder
 * @property {(n: number) => RangeBuilder<T>} take - Take n items after skipped position
 * @property {(arg?: SeriesFetchArg<T>, options?: ClientFetchOptions<SeriesData<T>>) => Promise<SeriesData<T>>} fetch - Fetch from skipped position to end
 * @property {(options?: ClientFetchOptions<string>) => Promise<string>} fetchCsv
 * @property {Thenable<T>} then
 */

/** @template T @typedef {Object} DateSkippedBuilder
 * @property {(n: number) => DateRangeBuilder<T>} take - Take n items after skipped position
 * @property {(arg?: DateSeriesFetchArg<T>, options?: ClientFetchOptions<DateSeriesData<T>>) => Promise<DateSeriesData<T>>} fetch - Fetch from skipped position to end
 * @property {(options?: ClientFetchOptions<string>) => Promise<string>} fetchCsv
 * @property {DateThenable<T>} then
 */

/** @template T @typedef {Object} RangeBuilder
 * @property {(arg?: SeriesFetchArg<T>, options?: ClientFetchOptions<SeriesData<T>>) => Promise<SeriesData<T>>} fetch - Fetch the selection (one item or a range)
 * @property {(options?: ClientFetchOptions<string>) => Promise<string>} fetchCsv
 * @property {Thenable<T>} then
 */

/** @template T @typedef {Object} DateRangeBuilder
 * @property {(arg?: DateSeriesFetchArg<T>, options?: ClientFetchOptions<DateSeriesData<T>>) => Promise<DateSeriesData<T>>} fetch - Fetch the selection (one item or a range)
 * @property {(options?: ClientFetchOptions<string>) => Promise<string>} fetchCsv
 * @property {DateThenable<T>} then
 */

/**
 * @template T
 * @typedef {Object} SeriesPattern
 * @property {string} name
 * @property {Readonly<Partial<Record<Index, SeriesEndpoint<T>>>>} by - Index endpoints as lazy getters
 * @property {() => readonly Index[]} indexes
 * @property {(index: Index) => SeriesEndpoint<T>|undefined} get
 */

/** @typedef {SeriesPattern<any>} AnySeriesPattern */

/**
 * @template T
 * @param {BitviewClient} client
 * @param {string} name
 * @param {Index} index
 * @returns {DateSeriesEndpoint<T>}
 */
function _endpoint(client, name, index) {
  const path = `/api/series/${encodeURIComponent(name)}/${index}`;

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
    return query ? `${path}?${query}` : path;
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
    get path() { return path; },
  };

  return endpoint;
}

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
   * The request's timeout, joined with the caller's signal when one is given.
   * @param {AbortSignal} [signal]
   * @returns {AbortSignal}
   */
  _signal(signal) {
    const timeout = AbortSignal.timeout(this.timeout);
    return signal ? AbortSignal.any([timeout, signal]) : timeout;
  }

  /**
   * @param {string} path
   * @param {{ signal?: AbortSignal, cache?: boolean, etag?: string | null }} [options]
   * @returns {Promise<Response>}
   */
  async get(path, { signal, cache = true, etag } = {}) {
    const url = `${this.baseUrl}${path}`;
    /** @type {RequestInit} */
    const init = { signal: this._signal(signal) };
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
   * - The returned Promise resolves with the **freshest** value (post-revalidation) and rejects
   *   when that fails, even if a stale value was already delivered.
   * - `onValue` fires once with the freshest value, or twice if a stale snapshot
   *   could be shown first (stale-while-revalidate). On a 304 there is no second fire.
   * - Responses marked `Cache-Control: no-store` are never cached.
   *
   * Layers:
   * - L1 (memCache): in-memory parsed values keyed by body kind + URL, with their ETag. Lets 304s
   *   skip the parse entirely.
   * - L2 (browserCache): Cache API, survives reload and feeds onValue fast on cold start.
   *
   * @template T
   * @param {string} path
   * @param {'json' | 'text' | 'bytes'} kind - The body reader, part of the memCache key
   * @param {(res: Response) => Promise<T>} parse - Response body reader
   * @param {ClientFetchOptions<T>} [options]
   * @returns {Promise<T>}
   */
  async _getCached(path, kind, parse, { onValue, signal, cache = true, memCache = true } = {}) {
    if (!cache) {
      const res = await this.get(path, { signal, cache });
      const value = await parse(res);
      if (onValue) onValue(value);
      return value;
    }

    const url = `${this.baseUrl}${path}`;
    const key = `${kind} ${url}`;
    const useMemCache = memCache !== false;
    /** @type {_MemEntry<T> | undefined} */
    const memHit = useMemCache ? this._memGet(key) : undefined;
    const browserCache = this._browserCache;
    /** @param {Response} res */
    const storable = (res) => !/no-store/i.test(res.headers.get('Cache-Control') ?? '');
    /** Parse a fresh response, cache it unless `no-store`, and deliver it. @param {Response} res */
    const fresh = async (res) => {
      const keep = storable(res);
      const cloned = keep && browserCache ? res.clone() : null;
      const value = await parse(res);
      if (keep && useMemCache) this._memSet(key, res.headers.get('ETag'), value);
      if (cloned && browserCache) {
        const cacheStore = browserCache;
        _runIdle(() => cacheStore.put(url, cloned));
      }
      if (onValue) onValue(value);
      return value;
    };

    // L1 fast path: deliver from memCache, revalidate via network.
    // ETag match → zero parse, zero clone, zero cache write, no second onValue fire.
    if (memHit) {
      if (onValue) onValue(memHit.value);
      const res = await this.get(path, { signal, etag: memHit.etag });
      const netEtag = res.headers.get('ETag');
      if (res.status === 304 || (netEtag && netEtag === memHit.etag)) {
        await res.body?.cancel();
        return memHit.value;
      }
      return fresh(res);
    }

    // L1 miss: race browserCache (stale snapshot) vs network (fresh).
    let networkSettled = false;
    // What this request's stale snapshot delivered, if anything.
    let shown = /** @type {{ value: T } | null} */ (null);
    const stale = onValue && browserCache
      ? browserCache.match(url).then(async (res) => {
          if (!res || networkSettled || signal?.aborted) return;
          // Older clients stored `no-store` responses: drop them.
          if (!storable(res)) return void browserCache.delete(url);
          const value = await parse(res);
          if (networkSettled || signal?.aborted) return;
          if (useMemCache) this._memSet(key, res.headers.get('ETag'), value);
          shown = { value };
          onValue(value);
        }).catch(() => {})
      : null;

    let res;
    try {
      res = await this.get(path, { signal });
    } catch (e) {
      // The stale snapshot, if any, arrives before the rejection, never after.
      await stale;
      throw e;
    }
    networkSettled = true;
    const netEtag = res.headers.get('ETag');
    // memCache already holds this ETag (our stale snapshot or a concurrent request) → reuse, skip
    // the parse; deliver it unless this request already showed that very value.
    const populated = useMemCache ? /** @type {_MemEntry<T> | undefined} */ (this._memGet(key)) : undefined;
    if (populated && netEtag && netEtag === populated.etag) {
      await res.body?.cancel();
      if (onValue && shown?.value !== populated.value) onValue(populated.value);
      return populated.value;
    }
    return fresh(res);
  }

  /**
   * Cached and supports `onValue`. Field names come camelCased, map keys as sent (`keep`).
   * @template T
   * @param {string} path
   * @param {ClientFetchOptions<T>} [options]
   * @param {*} [keep] Where the response has maps, as `_camelKeys` takes it
   * @returns {Promise<T>}
   */
  getJson(path, options, keep) {
    return this._getCached(path, 'json', async (res) => _camelKeys(await res.json(), keep), options);
  }

  /**
   * Cached and supports `onValue`, same as `getJson`.
   * @param {string} path
   * @param {ClientFetchOptions<string>} [options]
   * @returns {Promise<string>}
   */
  getText(path, options) {
    return this._getCached(path, 'text', (res) => res.text(), options);
  }

  /**
   * Cached and supports `onValue`, same as `getJson`.
   * @param {string} path
   * @param {ClientFetchOptions<Uint8Array>} [options]
   * @returns {Promise<Uint8Array>}
   */
  getBytes(path, options) {
    return this._getCached(path, 'bytes', async (res) => new Uint8Array(await res.arrayBuffer()), options);
  }

  /**
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
    const res = await fetch(url, { method: 'POST', body, signal: this._signal(signal) });
    if (!res.ok) throw await _responseError(res);
    return res;
  }

  /**
   * @template T
   * @param {string} path
   * @param {string} body
   * @param {{ signal?: AbortSignal }} [options]
   * @param {*} [keep] Where the response has maps, as `_camelKeys` takes it
   * @returns {Promise<T>}
   */
  async postJson(path, body, options, keep) {
    const res = await this.post(path, body, options);
    return _camelKeys(await res.json(), keep);
  }

  /**
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
// RapidHash v3's secrets 1 and 2: the only ones payloads of at most 65 bytes reach.
const _RAPIDHASH_SECRET_1 = 0x8bb84b93962eacc9n;
const _RAPIDHASH_SECRET_2 = 0x4b33a62ed433d4a3n;
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
  return _u64(seed ^ _rapidMix(seed ^ _RAPIDHASH_SECRET_2, _RAPIDHASH_SECRET_1));
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
    } else {
      a ^= (BigInt(bytes[0]) << 45n) | BigInt(bytes[length - 1]);
      b ^= BigInt(bytes[length >> 1]);
    }
    remainder = BigInt(length);
  } else {
    seed = _rapidMix(_readU64(bytes, 0) ^ _RAPIDHASH_SECRET_2, _readU64(bytes, 8) ^ seed);
    if (length > 32) {
      seed = _rapidMix(_readU64(bytes, 16) ^ _RAPIDHASH_SECRET_2, _readU64(bytes, 24) ^ seed);
      if (length > 48) {
        seed = _rapidMix(_readU64(bytes, 32) ^ _RAPIDHASH_SECRET_1, _readU64(bytes, 40) ^ seed);
        if (length > 64) {
          seed = _rapidMix(_readU64(bytes, 48) ^ _RAPIDHASH_SECRET_1, _readU64(bytes, 56) ^ seed);
        }
      }
    }
    remainder = BigInt(length);
    a ^= _readU64(bytes, length - 16) ^ remainder;
    b ^= _readU64(bytes, length - 8);
  }

  a ^= _RAPIDHASH_SECRET_1;
  b ^= seed;
  [a, b] = _rapidMum(a, b);
  return _rapidMix(a ^ 0xaaaaaaaaaaaaaaaan, b ^ _RAPIDHASH_SECRET_1 ^ remainder);
}

/** @param {number} nibbles */
function _validateHashPrefixNibbles(nibbles) {
  if (!Number.isInteger(nibbles) || nibbles < 1 || nibbles > 16) {
    throw new Error("Expected hash-prefix length from 1 to 16 hex nibbles");
  }
}

/**
 * @param {OutputType} addressType
 * @param {Uint8Array | ArrayBuffer | ArrayBufferView | number[]} payload
 */
function _validateAddressPayloadForType(addressType, payload) {
  const length = _asUint8Array(payload).length;
  const expected = _ADDRESS_PAYLOAD_LENGTHS.get(addressType);
  if (expected === undefined) {
    throw new Error(`Unsupported address type for address payload hash-prefix: ${addressType}`);
  }
  if (length !== expected) {
    throw new Error(`Expected ${addressType} address payload length ${expected} bytes`);
  }
}

/**
 * Compute the RapidHash v3 hash-prefix used by `/api/address/hash-prefix/{address_type}/{prefix}`.
 * @param {Uint8Array | ArrayBuffer | ArrayBufferView | number[]} payload
 * @param {number} nibbles - Prefix length from 1 to 16 hex nibbles
 * @returns {string}
 */
function addressPayloadHashPrefix(payload, nibbles) {
  _validateHashPrefixNibbles(nibbles);
  return _rapidHashV3(payload).toString(16).padStart(16, "0").slice(0, nibbles);
}

const _i1 = /** @type {const} */ (["minute10", "minute30", "hour1", "hour4", "hour12", "day1", "day3", "week1", "month1", "month3", "month6", "year1", "year10", "halving_epoch", "difficulty_epoch", "height"]);
const _i2 = /** @type {const} */ (["minute10", "minute30", "hour1", "hour4", "hour12", "day1", "day3", "week1", "month1", "month3", "month6", "year1", "year10", "halving_epoch", "difficulty_epoch"]);
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
const _i16 = /** @type {const} */ (["halving_epoch"]);
const _i17 = /** @type {const} */ (["difficulty_epoch"]);
const _i18 = /** @type {const} */ (["height"]);
const _i19 = /** @type {const} */ (["tx_index"]);
const _i20 = /** @type {const} */ (["txin_index"]);
const _i21 = /** @type {const} */ (["txin_index"]);
const _i22 = /** @type {const} */ (["txout_index"]);
const _i23 = /** @type {const} */ (["txout_index"]);
const _i24 = /** @type {const} */ (["empty_script_output_index"]);
const _i25 = /** @type {const} */ (["op_return_output_index"]);
const _i26 = /** @type {const} */ (["p2a_address_index"]);
const _i27 = /** @type {const} */ (["p2ms_output_index"]);
const _i28 = /** @type {const} */ (["p2pk33_address_index"]);
const _i29 = /** @type {const} */ (["p2pk65_address_index"]);
const _i30 = /** @type {const} */ (["p2pkh_address_index"]);
const _i31 = /** @type {const} */ (["p2sh_address_index"]);
const _i32 = /** @type {const} */ (["p2tr_address_index"]);
const _i33 = /** @type {const} */ (["p2wpkh_address_index"]);
const _i34 = /** @type {const} */ (["p2wsh_address_index"]);
const _i35 = /** @type {const} */ (["unknown_script_output_index"]);

/**
 * @template T
 * @param {BitviewClient} client
 * @param {string} name
 * @param {readonly Index[]} indexes
 */
function _seriesPattern(client, name, indexes) {
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

/** @template T @typedef {{ name: string, by: { readonly minute10: DateSeriesEndpoint<T | null>, readonly minute30: DateSeriesEndpoint<T | null>, readonly hour1: DateSeriesEndpoint<T | null>, readonly hour4: DateSeriesEndpoint<T | null>, readonly hour12: DateSeriesEndpoint<T | null>, readonly day1: DateSeriesEndpoint<T | null>, readonly day3: DateSeriesEndpoint<T | null>, readonly week1: DateSeriesEndpoint<T | null>, readonly month1: DateSeriesEndpoint<T | null>, readonly month3: DateSeriesEndpoint<T | null>, readonly month6: DateSeriesEndpoint<T | null>, readonly year1: DateSeriesEndpoint<T | null>, readonly year10: DateSeriesEndpoint<T | null>, readonly halving_epoch: SeriesEndpoint<T>, readonly difficulty_epoch: SeriesEndpoint<T>, readonly height: SeriesEndpoint<T> }, indexes: () => readonly Index[], get: (index: Index) => SeriesEndpoint<T | null>|undefined }} SeriesPattern1 */
/** @template T @typedef {{ name: string, by: { readonly minute10: DateSeriesEndpoint<T>, readonly minute30: DateSeriesEndpoint<T>, readonly hour1: DateSeriesEndpoint<T>, readonly hour4: DateSeriesEndpoint<T>, readonly hour12: DateSeriesEndpoint<T>, readonly day1: DateSeriesEndpoint<T>, readonly day3: DateSeriesEndpoint<T>, readonly week1: DateSeriesEndpoint<T>, readonly month1: DateSeriesEndpoint<T>, readonly month3: DateSeriesEndpoint<T>, readonly month6: DateSeriesEndpoint<T>, readonly year1: DateSeriesEndpoint<T>, readonly year10: DateSeriesEndpoint<T>, readonly halving_epoch: SeriesEndpoint<T>, readonly difficulty_epoch: SeriesEndpoint<T> }, indexes: () => readonly Index[], get: (index: Index) => SeriesEndpoint<T>|undefined }} SeriesPattern2 */
/** @template T @typedef {{ name: string, by: { readonly minute10: DateSeriesEndpoint<T> }, indexes: () => readonly Index[], get: (index: Index) => SeriesEndpoint<T>|undefined }} SeriesPattern3 */
/** @template T @typedef {{ name: string, by: { readonly minute30: DateSeriesEndpoint<T> }, indexes: () => readonly Index[], get: (index: Index) => SeriesEndpoint<T>|undefined }} SeriesPattern4 */
/** @template T @typedef {{ name: string, by: { readonly hour1: DateSeriesEndpoint<T> }, indexes: () => readonly Index[], get: (index: Index) => SeriesEndpoint<T>|undefined }} SeriesPattern5 */
/** @template T @typedef {{ name: string, by: { readonly hour4: DateSeriesEndpoint<T> }, indexes: () => readonly Index[], get: (index: Index) => SeriesEndpoint<T>|undefined }} SeriesPattern6 */
/** @template T @typedef {{ name: string, by: { readonly hour12: DateSeriesEndpoint<T> }, indexes: () => readonly Index[], get: (index: Index) => SeriesEndpoint<T>|undefined }} SeriesPattern7 */
/** @template T @typedef {{ name: string, by: { readonly day1: DateSeriesEndpoint<T> }, indexes: () => readonly Index[], get: (index: Index) => SeriesEndpoint<T>|undefined }} SeriesPattern8 */
/** @template T @typedef {{ name: string, by: { readonly day3: DateSeriesEndpoint<T> }, indexes: () => readonly Index[], get: (index: Index) => SeriesEndpoint<T>|undefined }} SeriesPattern9 */
/** @template T @typedef {{ name: string, by: { readonly week1: DateSeriesEndpoint<T> }, indexes: () => readonly Index[], get: (index: Index) => SeriesEndpoint<T>|undefined }} SeriesPattern10 */
/** @template T @typedef {{ name: string, by: { readonly month1: DateSeriesEndpoint<T> }, indexes: () => readonly Index[], get: (index: Index) => SeriesEndpoint<T>|undefined }} SeriesPattern11 */
/** @template T @typedef {{ name: string, by: { readonly month3: DateSeriesEndpoint<T> }, indexes: () => readonly Index[], get: (index: Index) => SeriesEndpoint<T>|undefined }} SeriesPattern12 */
/** @template T @typedef {{ name: string, by: { readonly month6: DateSeriesEndpoint<T> }, indexes: () => readonly Index[], get: (index: Index) => SeriesEndpoint<T>|undefined }} SeriesPattern13 */
/** @template T @typedef {{ name: string, by: { readonly year1: DateSeriesEndpoint<T> }, indexes: () => readonly Index[], get: (index: Index) => SeriesEndpoint<T>|undefined }} SeriesPattern14 */
/** @template T @typedef {{ name: string, by: { readonly year10: DateSeriesEndpoint<T> }, indexes: () => readonly Index[], get: (index: Index) => SeriesEndpoint<T>|undefined }} SeriesPattern15 */
/** @template T @typedef {{ name: string, by: { readonly halving_epoch: SeriesEndpoint<T> }, indexes: () => readonly Index[], get: (index: Index) => SeriesEndpoint<T>|undefined }} SeriesPattern16 */
/** @template T @typedef {{ name: string, by: { readonly difficulty_epoch: SeriesEndpoint<T> }, indexes: () => readonly Index[], get: (index: Index) => SeriesEndpoint<T>|undefined }} SeriesPattern17 */
/** @template T @typedef {{ name: string, by: { readonly height: SeriesEndpoint<T> }, indexes: () => readonly Index[], get: (index: Index) => SeriesEndpoint<T>|undefined }} SeriesPattern18 */
/** @template T @typedef {{ name: string, by: { readonly tx_index: SeriesEndpoint<T> }, indexes: () => readonly Index[], get: (index: Index) => SeriesEndpoint<T>|undefined }} SeriesPattern19 */
/** @template T @typedef {{ name: string, by: { readonly txin_index: SeriesEndpoint<T> }, indexes: () => readonly Index[], get: (index: Index) => SeriesEndpoint<T>|undefined }} SeriesPattern20 */
/** @template T @typedef {{ name: string, by: { readonly txin_index: SeriesEndpoint<T | null> }, indexes: () => readonly Index[], get: (index: Index) => SeriesEndpoint<T | null>|undefined }} SeriesPattern21 */
/** @template T @typedef {{ name: string, by: { readonly txout_index: SeriesEndpoint<T> }, indexes: () => readonly Index[], get: (index: Index) => SeriesEndpoint<T>|undefined }} SeriesPattern22 */
/** @template T @typedef {{ name: string, by: { readonly txout_index: SeriesEndpoint<T | null> }, indexes: () => readonly Index[], get: (index: Index) => SeriesEndpoint<T | null>|undefined }} SeriesPattern23 */
/** @template T @typedef {{ name: string, by: { readonly empty_script_output_index: SeriesEndpoint<T> }, indexes: () => readonly Index[], get: (index: Index) => SeriesEndpoint<T>|undefined }} SeriesPattern24 */
/** @template T @typedef {{ name: string, by: { readonly op_return_output_index: SeriesEndpoint<T> }, indexes: () => readonly Index[], get: (index: Index) => SeriesEndpoint<T>|undefined }} SeriesPattern25 */
/** @template T @typedef {{ name: string, by: { readonly p2a_address_index: SeriesEndpoint<T> }, indexes: () => readonly Index[], get: (index: Index) => SeriesEndpoint<T>|undefined }} SeriesPattern26 */
/** @template T @typedef {{ name: string, by: { readonly p2ms_output_index: SeriesEndpoint<T> }, indexes: () => readonly Index[], get: (index: Index) => SeriesEndpoint<T>|undefined }} SeriesPattern27 */
/** @template T @typedef {{ name: string, by: { readonly p2pk33_address_index: SeriesEndpoint<T> }, indexes: () => readonly Index[], get: (index: Index) => SeriesEndpoint<T>|undefined }} SeriesPattern28 */
/** @template T @typedef {{ name: string, by: { readonly p2pk65_address_index: SeriesEndpoint<T> }, indexes: () => readonly Index[], get: (index: Index) => SeriesEndpoint<T>|undefined }} SeriesPattern29 */
/** @template T @typedef {{ name: string, by: { readonly p2pkh_address_index: SeriesEndpoint<T> }, indexes: () => readonly Index[], get: (index: Index) => SeriesEndpoint<T>|undefined }} SeriesPattern30 */
/** @template T @typedef {{ name: string, by: { readonly p2sh_address_index: SeriesEndpoint<T> }, indexes: () => readonly Index[], get: (index: Index) => SeriesEndpoint<T>|undefined }} SeriesPattern31 */
/** @template T @typedef {{ name: string, by: { readonly p2tr_address_index: SeriesEndpoint<T> }, indexes: () => readonly Index[], get: (index: Index) => SeriesEndpoint<T>|undefined }} SeriesPattern32 */
/** @template T @typedef {{ name: string, by: { readonly p2wpkh_address_index: SeriesEndpoint<T> }, indexes: () => readonly Index[], get: (index: Index) => SeriesEndpoint<T>|undefined }} SeriesPattern33 */
/** @template T @typedef {{ name: string, by: { readonly p2wsh_address_index: SeriesEndpoint<T> }, indexes: () => readonly Index[], get: (index: Index) => SeriesEndpoint<T>|undefined }} SeriesPattern34 */
/** @template T @typedef {{ name: string, by: { readonly unknown_script_output_index: SeriesEndpoint<T> }, indexes: () => readonly Index[], get: (index: Index) => SeriesEndpoint<T>|undefined }} SeriesPattern35 */

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
  for (const [key, [make, template]] of Object.entries(children)) {
    Object.defineProperty(node, key, {
      get() {
        const name = template.replace(b ? '*' : /^\*_|_?\*/, b);
        return _lazy(this, key, () => typeof make === 'function' ? make(c, name) : _seriesPattern(c, name, make));
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
 *   p0_1: SeriesPattern1<?Dollars>,
 *   p0_5: SeriesPattern1<?Dollars>,
 *   p1: SeriesPattern1<?Dollars>,
 *   p2: SeriesPattern1<?Dollars>,
 *   p5: SeriesPattern1<?Dollars>,
 *   p10: SeriesPattern1<?Dollars>,
 *   p20: SeriesPattern1<?Dollars>,
 *   p30: SeriesPattern1<?Dollars>,
 *   p40: SeriesPattern1<?Dollars>,
 *   median: SeriesPattern1<?Dollars>,
 *   p60: SeriesPattern1<?Dollars>,
 *   p70: SeriesPattern1<?Dollars>,
 *   p80: SeriesPattern1<?Dollars>,
 *   p90: SeriesPattern1<?Dollars>,
 *   p95: SeriesPattern1<?Dollars>,
 *   p98: SeriesPattern1<?Dollars>,
 *   p99: SeriesPattern1<?Dollars>,
 *   p99_5: SeriesPattern1<?Dollars>,
 *   p99_9: SeriesPattern1<?Dollars>,
 *   level: SeriesPattern1<?Score>,
 *   score: SeriesPattern1<?Score>,
 * }} Meter
 */
const _Meter = _s({
  p0_1: [_i1, '*_p0_1'],
  p0_5: [_i1, '*_p0_5'],
  p1: [_i1, '*_p1'],
  p2: [_i1, '*_p2'],
  p5: [_i1, '*_p5'],
  p10: [_i1, '*_p10'],
  p20: [_i1, '*_p20'],
  p30: [_i1, '*_p30'],
  p40: [_i1, '*_p40'],
  median: [_i1, '*_median'],
  p60: [_i1, '*_p60'],
  p70: [_i1, '*_p70'],
  p80: [_i1, '*_p80'],
  p90: [_i1, '*_p90'],
  p95: [_i1, '*_p95'],
  p98: [_i1, '*_p98'],
  p99: [_i1, '*_p99'],
  p99_5: [_i1, '*_p99_5'],
  p99_9: [_i1, '*_p99_9'],
  level: [_i1, '*_level'],
  score: [_i1, '*_score'],
});

/**
 * @template A
 * @typedef {{
 *   value: SeriesPattern1<A>,
 *   p0_1: SeriesPattern1<A>,
 *   p0_5: SeriesPattern1<A>,
 *   p1: SeriesPattern1<A>,
 *   p2: SeriesPattern1<A>,
 *   p5: SeriesPattern1<A>,
 *   p10: SeriesPattern1<A>,
 *   p20: SeriesPattern1<A>,
 *   p30: SeriesPattern1<A>,
 *   p40: SeriesPattern1<A>,
 *   median: SeriesPattern1<A>,
 *   p60: SeriesPattern1<A>,
 *   p70: SeriesPattern1<A>,
 *   p80: SeriesPattern1<A>,
 *   p90: SeriesPattern1<A>,
 *   p95: SeriesPattern1<A>,
 *   p98: SeriesPattern1<A>,
 *   p99: SeriesPattern1<A>,
 *   p99_5: SeriesPattern1<A>,
 *   p99_9: SeriesPattern1<A>,
 * }} Extreme
 */
const _Extreme = _s({
  value: [_i1, '*'],
  p0_1: [_i1, '*_rarity_p0_1'],
  p0_5: [_i1, '*_rarity_p0_5'],
  p1: [_i1, '*_rarity_p1'],
  p2: [_i1, '*_rarity_p2'],
  p5: [_i1, '*_rarity_p5'],
  p10: [_i1, '*_rarity_p10'],
  p20: [_i1, '*_rarity_p20'],
  p30: [_i1, '*_rarity_p30'],
  p40: [_i1, '*_rarity_p40'],
  median: [_i1, '*_rarity_median'],
  p60: [_i1, '*_rarity_p60'],
  p70: [_i1, '*_rarity_p70'],
  p80: [_i1, '*_rarity_p80'],
  p90: [_i1, '*_rarity_p90'],
  p95: [_i1, '*_rarity_p95'],
  p98: [_i1, '*_rarity_p98'],
  p99: [_i1, '*_rarity_p99'],
  p99_5: [_i1, '*_rarity_p99_5'],
  p99_9: [_i1, '*_rarity_p99_9'],
});

/**
 * @typedef {{
 *   supplyInLoss: Extreme<?Bitcoin>,
 *   realizedProfitSum24h: Extreme<?Dollars>,
 *   realizedLossSum24h: Extreme<?Dollars>,
 *   realizedPeakRegretSum24h: Extreme<?Dollars>,
 *   sellSideRiskRatio24h: Extreme<?Ratio>,
 * }} Extremes
 */
const _Extremes = _s({
  supplyInLoss: [_Extreme, 'supply_in_loss'],
  realizedProfitSum24h: [_Extreme, 'realized_profit_sum_*'],
  realizedLossSum24h: [_Extreme, 'realized_loss_sum_*'],
  realizedPeakRegretSum24h: [_Extreme, 'realized_peak_regret_sum_*'],
  sellSideRiskRatio24h: [_Extreme, 'sell_side_risk_ratio_*'],
});

/**
 * @typedef {{
 *   ratio: SeriesPattern1<?Multiple>,
 *   band: SeriesPattern1<?Dollars>,
 * }} MobileMedianCapitalizedPriceMedian
 */
const _MobileMedianCapitalizedPriceMedian = _s({
  ratio: [_i1, 'mobile_median_capitalized_price_ratio_*'],
  band: [_i1, 'mobile_median_capitalized_price_*'],
});

/**
 * @typedef {{
 *   ratio: SeriesPattern1<?Multiple>,
 *   band: SeriesPattern1<?Dollars>,
 * }} MobileMedianRealizedPriceMedian
 */
const _MobileMedianRealizedPriceMedian = _s({
  ratio: [_i1, 'mobile_median_realized_price_ratio_*'],
  band: [_i1, 'mobile_median_realized_price_*'],
});

/**
 * @typedef {{
 *   ratio: SeriesPattern1<?Multiple>,
 *   band: SeriesPattern1<?Dollars>,
 * }} MobileRealizedPriceMedian
 */
const _MobileRealizedPriceMedian = _s({
  ratio: [_i1, 'mobile_realized_price_ratio_*'],
  band: [_i1, 'mobile_realized_price_*'],
});

/**
 * @typedef {{
 *   ratio: SeriesPattern1<?Multiple>,
 *   band: SeriesPattern1<?Dollars>,
 * }} AwakeRealizedPriceMedian
 */
const _AwakeRealizedPriceMedian = _s({
  ratio: [_i1, 'awake_realized_price_ratio_*'],
  band: [_i1, 'awake_realized_price_*'],
});

/**
 * @typedef {{
 *   ratio: SeriesPattern1<?Multiple>,
 *   band: SeriesPattern1<?Dollars>,
 * }} Under6mCapitalizedPriceMedian
 */
const _Under6mCapitalizedPriceMedian = _s({
  ratio: [_i1, 'age_under_6m_capitalized_price_ratio_*'],
  band: [_i1, 'age_under_6m_capitalized_price_*'],
});

/**
 * @typedef {{
 *   ratio: SeriesPattern1<?Multiple>,
 *   band: SeriesPattern1<?Dollars>,
 * }} Under6mRealizedPriceMedian
 */
const _Under6mRealizedPriceMedian = _s({
  ratio: [_i1, 'age_under_6m_realized_price_ratio_*'],
  band: [_i1, 'age_under_6m_realized_price_*'],
});

/**
 * @typedef {{
 *   ratio: SeriesPattern1<?Multiple>,
 *   band: SeriesPattern1<?Dollars>,
 * }} SthMedianCapitalizedPriceMedian
 */
const _SthMedianCapitalizedPriceMedian = _s({
  ratio: [_i1, 'sth_median_capitalized_price_ratio_*'],
  band: [_i1, 'sth_median_capitalized_price_*'],
});

/**
 * @typedef {{
 *   ratio: SeriesPattern1<?Multiple>,
 *   band: SeriesPattern1<?Dollars>,
 * }} SthMedianRealizedPriceMedian
 */
const _SthMedianRealizedPriceMedian = _s({
  ratio: [_i1, 'sth_median_realized_price_ratio_*'],
  band: [_i1, 'sth_median_realized_price_*'],
});

/**
 * @typedef {{
 *   ratio: SeriesPattern1<?Multiple>,
 *   band: SeriesPattern1<?Dollars>,
 * }} SthCapitalizedPriceMedian
 */
const _SthCapitalizedPriceMedian = _s({
  ratio: [_i1, 'sth_capitalized_price_ratio_*'],
  band: [_i1, 'sth_capitalized_price_*'],
});

/**
 * @typedef {{
 *   ratio: SeriesPattern1<?Multiple>,
 *   band: SeriesPattern1<?Dollars>,
 * }} SthRealizedPriceMedian
 */
const _SthRealizedPriceMedian = _s({
  ratio: [_i1, 'sth_realized_price_ratio_*'],
  band: [_i1, 'sth_realized_price_*'],
});

/**
 * @typedef {{
 *   ratio: SeriesPattern1<?Multiple>,
 *   band: SeriesPattern1<?Dollars>,
 * }} TrueMarketMeanMedian
 */
const _TrueMarketMeanMedian = _s({
  ratio: [_i1, 'true_market_mean_ratio_*'],
  band: [_i1, 'true_market_mean_*'],
});

/**
 * @typedef {{
 *   ratio: SeriesPattern1<?Multiple>,
 *   band: SeriesPattern1<?Dollars>,
 * }} VaultedPriceMedian
 */
const _VaultedPriceMedian = _s({
  ratio: [_i1, 'vaulted_price_ratio_*'],
  band: [_i1, 'vaulted_price_*'],
});

/**
 * @typedef {{
 *   ratio: SeriesPattern1<?Multiple>,
 *   band: SeriesPattern1<?Dollars>,
 * }} AllMedianCapitalizedPriceMedian
 */
const _AllMedianCapitalizedPriceMedian = _s({
  ratio: [_i1, 'median_capitalized_price_ratio_*'],
  band: [_i1, 'median_capitalized_price_*'],
});

/**
 * @typedef {{
 *   ratio: SeriesPattern1<?Multiple>,
 *   band: SeriesPattern1<?Dollars>,
 * }} AllMedianRealizedPriceMedian
 */
const _AllMedianRealizedPriceMedian = _s({
  ratio: [_i1, 'median_realized_price_ratio_*'],
  band: [_i1, 'median_realized_price_*'],
});

/**
 * @template A
 * @typedef {{
 *   value: SeriesPattern1<?Dollars>,
 *   ratio: SeriesPattern1<?Multiple>,
 *   p0_1: A,
 *   p0_5: A,
 *   p1: A,
 *   p2: A,
 *   p5: A,
 *   p10: A,
 *   p20: A,
 *   p30: A,
 *   p40: A,
 *   median: A,
 *   p60: A,
 *   p70: A,
 *   p80: A,
 *   p90: A,
 *   p95: A,
 *   p98: A,
 *   p99: A,
 *   p99_5: A,
 *   p99_9: A,
 * }} MedianComponent
 */
/** @type {_Make} */
const _MedianComponent = (c, b, f0) => _n(c, b, {
  value: [_i1, '*'],
  ratio: [_i1, '*_ratio'],
  p0_1: [f0, 'rarity_p0_1'],
  p0_5: [f0, 'rarity_p0_5'],
  p1: [f0, 'rarity_p1'],
  p2: [f0, 'rarity_p2'],
  p5: [f0, 'rarity_p5'],
  p10: [f0, 'rarity_p10'],
  p20: [f0, 'rarity_p20'],
  p30: [f0, 'rarity_p30'],
  p40: [f0, 'rarity_p40'],
  median: [f0, 'rarity_median'],
  p60: [f0, 'rarity_p60'],
  p70: [f0, 'rarity_p70'],
  p80: [f0, 'rarity_p80'],
  p90: [f0, 'rarity_p90'],
  p95: [f0, 'rarity_p95'],
  p98: [f0, 'rarity_p98'],
  p99: [f0, 'rarity_p99'],
  p99_5: [f0, 'rarity_p99_5'],
  p99_9: [f0, 'rarity_p99_9'],
});

/**
 * @typedef {{
 *   ratio: SeriesPattern1<?Multiple>,
 *   band: SeriesPattern1<?Dollars>,
 * }} AllCapitalizedPriceMedian
 */
const _AllCapitalizedPriceMedian = _s({
  ratio: [_i1, 'capitalized_price_ratio_*'],
  band: [_i1, 'capitalized_price_*'],
});

/**
 * @typedef {{
 *   ratio: SeriesPattern1<?Multiple>,
 *   band: SeriesPattern1<?Dollars>,
 * }} AllRealizedPriceMedian
 */
const _AllRealizedPriceMedian = _s({
  ratio: [_i1, 'realized_price_ratio_*'],
  band: [_i1, 'realized_price_*'],
});

/**
 * @template A
 * @typedef {{
 *   p0_1: A,
 *   p0_5: A,
 *   p1: A,
 *   p2: A,
 *   p5: A,
 *   p10: A,
 *   p20: A,
 *   p30: A,
 *   p40: A,
 *   median: A,
 *   p60: A,
 *   p70: A,
 *   p80: A,
 *   p90: A,
 *   p95: A,
 *   p98: A,
 *   p99: A,
 *   p99_5: A,
 *   p99_9: A,
 * }} RarityPercentiles
 */
/** @type {_Make} */
const _RarityPercentiles = (c, b, f0) => _n(c, b, {
  p0_1: [f0, '*_p0_1'],
  p0_5: [f0, '*_p0_5'],
  p1: [f0, '*_p1'],
  p2: [f0, '*_p2'],
  p5: [f0, '*_p5'],
  p10: [f0, '*_p10'],
  p20: [f0, '*_p20'],
  p30: [f0, '*_p30'],
  p40: [f0, '*_p40'],
  median: [f0, '*_median'],
  p60: [f0, '*_p60'],
  p70: [f0, '*_p70'],
  p80: [f0, '*_p80'],
  p90: [f0, '*_p90'],
  p95: [f0, '*_p95'],
  p98: [f0, '*_p98'],
  p99: [f0, '*_p99'],
  p99_5: [f0, '*_p99_5'],
  p99_9: [f0, '*_p99_9'],
});

/**
 * @typedef {{
 *   realizedPrice: RarityPercentiles<MobileRealizedPriceMedian>,
 *   medianRealizedPrice: MedianComponent<MobileMedianRealizedPriceMedian>,
 *   medianCapitalizedPrice: MedianComponent<MobileMedianCapitalizedPriceMedian>,
 * }} MobileComponents
 */
const _MobileComponents = _s({
  realizedPrice: [(c, b) => _RarityPercentiles(c, b, _MobileRealizedPriceMedian), 'rarity'],
  medianRealizedPrice: [(c, b) => _MedianComponent(c, b, _MobileMedianRealizedPriceMedian), '*_realized_price'],
  medianCapitalizedPrice: [(c, b) => _MedianComponent(c, b, _MobileMedianCapitalizedPriceMedian), '*_capitalized_price'],
});

/**
 * @typedef {{
 *   realizedPrice: RarityPercentiles<AwakeRealizedPriceMedian>,
 * }} AwakeComponents
 */
const _AwakeComponents = _s({
  realizedPrice: [(c, b) => _RarityPercentiles(c, b, _AwakeRealizedPriceMedian), '*'],
});

/**
 * @typedef {{
 *   realizedPrice: RarityPercentiles<Under6mRealizedPriceMedian>,
 *   capitalizedPrice: RarityPercentiles<Under6mCapitalizedPriceMedian>,
 * }} MeanComponents
 */
const _MeanComponents = _s({
  realizedPrice: [(c, b) => _RarityPercentiles(c, b, _Under6mRealizedPriceMedian), '*'],
  capitalizedPrice: [(c, b) => _RarityPercentiles(c, b, _Under6mCapitalizedPriceMedian), '*'],
});

/**
 * @typedef {{
 *   realizedPrice: RarityPercentiles<SthRealizedPriceMedian>,
 *   capitalizedPrice: RarityPercentiles<SthCapitalizedPriceMedian>,
 *   medianRealizedPrice: MedianComponent<SthMedianRealizedPriceMedian>,
 *   medianCapitalizedPrice: MedianComponent<SthMedianCapitalizedPriceMedian>,
 * }} CohortComponents
 */
const _CohortComponents = _s({
  realizedPrice: [(c, b) => _RarityPercentiles(c, b, _SthRealizedPriceMedian), '*'],
  capitalizedPrice: [(c, b) => _RarityPercentiles(c, b, _SthCapitalizedPriceMedian), '*'],
  medianRealizedPrice: [(c, b) => _MedianComponent(c, b, _SthMedianRealizedPriceMedian), 'sth_median_realized_price'],
  medianCapitalizedPrice: [(c, b) => _MedianComponent(c, b, _SthMedianCapitalizedPriceMedian), 'sth_median_capitalized_price'],
});

/**
 * @typedef {{
 *   realizedPrice: RarityPercentiles<AllRealizedPriceMedian>,
 *   capitalizedPrice: RarityPercentiles<AllCapitalizedPriceMedian>,
 *   medianRealizedPrice: MedianComponent<AllMedianRealizedPriceMedian>,
 *   medianCapitalizedPrice: MedianComponent<AllMedianCapitalizedPriceMedian>,
 *   vaultedPrice: RarityPercentiles<VaultedPriceMedian>,
 *   trueMarketMean: RarityPercentiles<TrueMarketMeanMedian>,
 * }} AllComponents
 */
const _AllComponents = _s({
  realizedPrice: [(c, b) => _RarityPercentiles(c, b, _AllRealizedPriceMedian), '*'],
  capitalizedPrice: [(c, b) => _RarityPercentiles(c, b, _AllCapitalizedPriceMedian), '*'],
  medianRealizedPrice: [(c, b) => _MedianComponent(c, b, _AllMedianRealizedPriceMedian), 'median_realized_price'],
  medianCapitalizedPrice: [(c, b) => _MedianComponent(c, b, _AllMedianCapitalizedPriceMedian), 'median_capitalized_price'],
  vaultedPrice: [(c, b) => _RarityPercentiles(c, b, _VaultedPriceMedian), '*'],
  trueMarketMean: [(c, b) => _RarityPercentiles(c, b, _TrueMarketMeanMedian), '*'],
});

/**
 * @typedef {{
 *   all: AllComponents,
 *   sth: CohortComponents,
 *   under6m: MeanComponents,
 *   awake: AwakeComponents,
 *   mobile: MobileComponents,
 * }} Components
 */
const _Components = _s({
  all: [_AllComponents, '*'],
  sth: [_CohortComponents, '*'],
  under6m: [_MeanComponents, '*'],
  awake: [_AwakeComponents, '*'],
  mobile: [_MobileComponents, 'mobile_median'],
});

/**
 * @typedef {{
 *   components: Components,
 *   extremes: Extremes,
 *   full: Meter,
 *   local: Meter,
 *   cycle: Meter,
 * }} RarityMeter
 */
const _RarityMeter = _s({
  components: [_Components, '*'],
  extremes: [_Extremes, '24h'],
  full: [_Meter, '*_meter'],
  local: [_Meter, 'local_*_meter'],
  cycle: [_Meter, 'cycle_*_meter'],
});

/**
 * @typedef {{
 *   p95ToP98: SeriesPattern1<?Dollars>,
 *   p98ToP99: SeriesPattern1<?Dollars>,
 *   p99ToP99_5: SeriesPattern1<?Dollars>,
 *   p99_5ToP99_9: SeriesPattern1<?Dollars>,
 *   belowP99_9: SeriesPattern1<?Dollars>,
 * }} EntryCapital
 */
const _EntryCapital = _s({
  p95ToP98: [_i1, '*_p95_to_p98'],
  p98ToP99: [_i1, '*_p98_to_p99'],
  p99ToP99_5: [_i1, '*_p99_to_p99_5'],
  p99_5ToP99_9: [_i1, '*_p99_5_to_p99_9'],
  belowP99_9: [_i1, '*_below_p99_9'],
});

/**
 * @typedef {{
 *   p10: SeriesPattern1<?Dollars>,
 *   p20: SeriesPattern1<?Dollars>,
 *   p30: SeriesPattern1<?Dollars>,
 *   p40: SeriesPattern1<?Dollars>,
 *   median: SeriesPattern1<?Dollars>,
 *   p60: SeriesPattern1<?Dollars>,
 *   p70: SeriesPattern1<?Dollars>,
 *   p80: SeriesPattern1<?Dollars>,
 *   p90: SeriesPattern1<?Dollars>,
 * }} Levels
 */
const _Levels = _s({
  p10: [_i1, '*_p10'],
  p20: [_i1, '*_p20'],
  p30: [_i1, '*_p30'],
  p40: [_i1, '*_p40'],
  median: [_i1, '*_median'],
  p60: [_i1, '*_p60'],
  p70: [_i1, '*_p70'],
  p80: [_i1, '*_p80'],
  p90: [_i1, '*_p90'],
});

/**
 * @template A
 * @typedef {{
 *   p95: SeriesPattern1<A>,
 *   p98: SeriesPattern1<A>,
 *   p99: SeriesPattern1<A>,
 *   p99_5: SeriesPattern1<A>,
 *   p99_9: SeriesPattern1<A>,
 * }} Percentiles
 */
const _Percentiles = _s({
  p95: [_i1, '*_p95'],
  p98: [_i1, '*_p98'],
  p99: [_i1, '*_p99'],
  p99_5: [_i1, '*_p99_5'],
  p99_9: [_i1, '*_p99_9'],
});

/**
 * @typedef {{
 *   value: SeriesPattern1<?Dollars>,
 *   ratio: SeriesPattern1<?Multiple>,
 *   aviv: SeriesPattern1<?Multiple>,
 *   avivNupl: SeriesPattern1<?Ratio>,
 * }} CointimePriceWithRatio
 */
const _CointimePriceWithRatio = _s({
  value: [_i1, '*'],
  ratio: [_i1, '*_ratio'],
  aviv: [_i1, 'aviv'],
  avivNupl: [_i1, 'aviv_nupl'],
});

/**
 * @typedef {{
 *   value: SeriesPattern1<?Dollars>,
 *   ratio: SeriesPattern1<?Multiple>,
 *   mvrv: SeriesPattern1<?Multiple>,
 * }} CointimePriceWithMvrv
 */
const _CointimePriceWithMvrv = _s({
  value: [_i1, '*_price'],
  ratio: [_i1, '*_price_ratio'],
  mvrv: [_i1, '*_mvrv'],
});

/**
 * @typedef {{
 *   vaulted: CointimePriceWithMvrv,
 *   active: CointimePriceWithMvrv,
 *   trueMarketMean: CointimePriceWithRatio,
 *   cointime: CointimePriceWithMvrv,
 * }} Prices
 */
const _Prices = _s({
  vaulted: [_CointimePriceWithMvrv, '*'],
  active: [_CointimePriceWithMvrv, 'active'],
  trueMarketMean: [_CointimePriceWithRatio, 'true_market_mean'],
  cointime: [_CointimePriceWithMvrv, 'cointime'],
});

/**
 * @typedef {{
 *   thermo: SeriesPattern1<?Dollars>,
 *   investor: SeriesPattern1<?Dollars>,
 *   active: SeriesPattern1<?Dollars>,
 *   vaulted: SeriesPattern1<?Dollars>,
 *   cointime: SeriesPattern1<?Dollars>,
 *   investorness: SeriesPattern1<?Percent>,
 *   producerness: SeriesPattern1<?Percent>,
 * }} Caps
 */
const _Caps = _s({
  thermo: [_i1, 'thermo_*'],
  investor: [_i1, 'investor_*'],
  active: [_i1, 'active_*'],
  vaulted: [_i1, 'vaulted_*'],
  cointime: [_i1, 'cointime_*'],
  investorness: [_i1, 'investorness'],
  producerness: [_i1, 'producerness'],
});

/**
 * @typedef {{
 *   total: SeriesPattern1<?Dollars>,
 *   realizedCap: SeriesPattern1<?Dollars>,
 * }} MobileCapital
 */
const _MobileCapital = _s({
  total: [_i1, '*_capital'],
  realizedCap: [_i1, '*_realized_cap'],
});

/**
 * @typedef {{
 *   share: SeriesPattern1<?Percent>,
 * }} InLoss
 */
const _InLoss = _s({
  share: [_i1, '*'],
});

/**
 * @typedef {{
 *   btc: SeriesPattern1<?Ratio>,
 *   usd: SeriesPattern1<?Ratio>,
 * }} Velocity
 */
const _Velocity = _s({
  btc: [_i1, '*_btc'],
  usd: [_i1, '*_usd'],
});

/**
 * @typedef {{
 *   inflationRate: SeriesPattern1<?Percent>,
 *   velocity: Velocity,
 * }} Adjusted
 */
const _Adjusted = _s({
  inflationRate: [_i1, '*_inflation_rate'],
  velocity: [_Velocity, '*_velocity'],
});

/**
 * @typedef {{
 *   total: SeriesPattern1<?Dollars>,
 *   realizedCap: SeriesPattern1<?Dollars>,
 *   share: SeriesPattern1<?Percent>,
 * }} BandCapital
 */
const _BandCapital = _s({
  total: [_i1, '*_capital'],
  realizedCap: [_i1, '*_realized_cap'],
  share: [_i1, '*_capital_share'],
});

/**
 * @typedef {{
 *   total: SeriesPattern1<?Percent>,
 *   inProfit: SeriesPattern1<?Percent>,
 *   inLoss: SeriesPattern1<?Percent>,
 * }} Density
 */
const _Density = _s({
  total: [_i1, '*'],
  inProfit: [_i1, '*_in_profit'],
  inLoss: [_i1, '*_in_loss'],
});

/**
 * @typedef {{
 *   value: SeriesPattern1<?Dollars>,
 *   ratio: SeriesPattern1<?Multiple>,
 *   mvrv: SeriesPattern1<?Multiple>,
 * }} PriceWithMvrv
 */
const _PriceWithMvrv = _s({
  value: [_i1, '*_cost_basis_by_coin_avg'],
  ratio: [_i1, '*_cost_basis_by_coin_avg_ratio'],
  mvrv: [_i1, '*_mvrv'],
});

/**
 * @typedef {{
 *   avg: PriceWithMvrv,
 *   realizedPrice: SeriesPattern1<?Dollars>,
 *   p5: SeriesPattern1<?Dollars>,
 *   p10: SeriesPattern1<?Dollars>,
 *   p15: SeriesPattern1<?Dollars>,
 *   p20: SeriesPattern1<?Dollars>,
 *   p25: SeriesPattern1<?Dollars>,
 *   p30: SeriesPattern1<?Dollars>,
 *   p35: SeriesPattern1<?Dollars>,
 *   p40: SeriesPattern1<?Dollars>,
 *   p45: SeriesPattern1<?Dollars>,
 *   median: SeriesPattern1<?Dollars>,
 *   p55: SeriesPattern1<?Dollars>,
 *   p60: SeriesPattern1<?Dollars>,
 *   p65: SeriesPattern1<?Dollars>,
 *   p70: SeriesPattern1<?Dollars>,
 *   p75: SeriesPattern1<?Dollars>,
 *   p80: SeriesPattern1<?Dollars>,
 *   p85: SeriesPattern1<?Dollars>,
 *   p90: SeriesPattern1<?Dollars>,
 *   p95: SeriesPattern1<?Dollars>,
 *   medianRealizedPrice: SeriesPattern1<?Dollars>,
 * }} CostBasisByCoin
 */
const _CostBasisByCoin = _s({
  avg: [_PriceWithMvrv, '*'],
  realizedPrice: [_i1, '*_realized_price'],
  p5: [_i1, '*_cost_basis_by_coin_p5'],
  p10: [_i1, '*_cost_basis_by_coin_p10'],
  p15: [_i1, '*_cost_basis_by_coin_p15'],
  p20: [_i1, '*_cost_basis_by_coin_p20'],
  p25: [_i1, '*_cost_basis_by_coin_p25'],
  p30: [_i1, '*_cost_basis_by_coin_p30'],
  p35: [_i1, '*_cost_basis_by_coin_p35'],
  p40: [_i1, '*_cost_basis_by_coin_p40'],
  p45: [_i1, '*_cost_basis_by_coin_p45'],
  median: [_i1, '*_cost_basis_by_coin_median'],
  p55: [_i1, '*_cost_basis_by_coin_p55'],
  p60: [_i1, '*_cost_basis_by_coin_p60'],
  p65: [_i1, '*_cost_basis_by_coin_p65'],
  p70: [_i1, '*_cost_basis_by_coin_p70'],
  p75: [_i1, '*_cost_basis_by_coin_p75'],
  p80: [_i1, '*_cost_basis_by_coin_p80'],
  p85: [_i1, '*_cost_basis_by_coin_p85'],
  p90: [_i1, '*_cost_basis_by_coin_p90'],
  p95: [_i1, '*_cost_basis_by_coin_p95'],
  medianRealizedPrice: [_i1, '*_median_realized_price'],
});

/**
 * @typedef {{
 *   profit: SeriesPattern1<?Dollars>,
 *   loss: SeriesPattern1<?Dollars>,
 *   netPnl: SeriesPattern1<?Dollars>,
 *   grossPnl: SeriesPattern1<?Dollars>,
 *   nupl: SeriesPattern1<?Ratio>,
 * }} Unrealized
 */
const _Unrealized = _s({
  profit: [_i1, '*_unrealized_profit'],
  loss: [_i1, '*_unrealized_loss'],
  netPnl: [_i1, '*_net_unrealized_pnl'],
  grossPnl: [_i1, '*_gross_unrealized_pnl'],
  nupl: [_i1, '*_nupl'],
});

/**
 * @typedef {{
 *   value: SeriesPattern1<?Dollars>,
 *   share: SeriesPattern1<?Percent>,
 * }} Part
 */
const _Part = _s({
  value: [_i1, '*'],
  share: [_i1, '*_share'],
});

/**
 * @typedef {{
 *   btc: SeriesPattern1<?Bitcoin>,
 *   usd: SeriesPattern1<?Dollars>,
 * }} SpotValue
 */
const _SpotValue = _s({
  btc: [_i1, '*'],
  usd: [_i1, 'market_cap'],
});

/**
 * @typedef {{
 *   funded: SeriesPattern1<Count>,
 *   total: SeriesPattern1<Count>,
 * }} FundedTotal
 */
const _FundedTotal = _s({
  funded: [_i1, '*_funded_exposed_address_count'],
  total: [_i1, '*_total_exposed_address_count'],
});

/**
 * @typedef {{
 *   funded: SeriesPattern1<Count>,
 *   total: SeriesPattern1<Count>,
 * }} RespentCount
 */
const _RespentCount = _s({
  funded: [_i1, '*_funded_respent_address_count'],
  total: [_i1, '*_total_respent_address_count'],
});

/**
 * @typedef {{
 *   btc: SeriesPattern1<?Bitcoin>,
 *   usd: SeriesPattern1<?Dollars>,
 *   share: SeriesPattern1<?Percent>,
 * }} PredicateSupply
 */
const _PredicateSupply = _s({
  btc: [_i1, '*'],
  usd: [_i1, '*_usd'],
  share: [_i1, '*_share'],
});

/**
 * @typedef {{
 *   count: FundedTotal,
 *   supply: PredicateSupply,
 * }} Exposed
 */
const _Exposed = _s({
  count: [_FundedTotal, '*'],
  supply: [_PredicateSupply, '*_exposed_address_supply'],
});

/**
 * @typedef {{
 *   funded: SeriesPattern1<Count>,
 *   total: SeriesPattern1<Count>,
 * }} ReusedCount
 */
const _ReusedCount = _s({
  funded: [_i1, '*_funded_reused_address_count'],
  total: [_i1, '*_total_reused_address_count'],
});

/**
 * @typedef {{
 *   avg: SeriesPattern1<?Dollars>,
 *   realizedPrice: SeriesPattern1<?Dollars>,
 * }} ByCoin
 */
const _ByCoin = _s({
  avg: [_i1, '*_cost_basis_by_coin_avg'],
  realizedPrice: [_i1, '*_realized_price'],
});

/**
 * @typedef {{
 *   byCoin: ByCoin,
 * }} CohortCostBasis
 */
const _CohortCostBasis = _s({
  byCoin: [_ByCoin, '*'],
});

/**
 * @typedef {{
 *   profit: SeriesPattern1<?Dollars>,
 *   loss: SeriesPattern1<?Dollars>,
 *   netPnl: SeriesPattern1<?Dollars>,
 * }} CreationUnrealized
 */
const _CreationUnrealized = _s({
  profit: [_i1, '*_unrealized_profit'],
  loss: [_i1, '*_unrealized_loss'],
  netPnl: [_i1, '*_net_unrealized_pnl'],
});

/**
 * @typedef {{
 *   _24h: SeriesPattern1<?Percent>,
 *   _1w: SeriesPattern1<?Percent>,
 *   _1m: SeriesPattern1<?Percent>,
 *   _1y: SeriesPattern1<?Percent>,
 * }} AddressesWindows
 */
const _AddressesWindows = _s({
  _24h: [_i1, '*_24h_percent'],
  _1w: [_i1, '*_1w_percent'],
  _1m: [_i1, '*_1m_percent'],
  _1y: [_i1, '*_1y_percent'],
});

/**
 * @typedef {{
 *   value: SeriesPattern1<?Dollars>,
 *   ratio: SeriesPattern1<?Multiple>,
 *   mayerMultiple: SeriesPattern1<?Multiple>,
 * }} MarketPriceWithRatio
 */
const _MarketPriceWithRatio = _s({
  value: [_i1, '*'],
  ratio: [_i1, '*_ratio'],
  mayerMultiple: [_i1, 'mayer_multiple'],
});

/**
 * @typedef {{
 *   value: SeriesPattern1<?Dollars>,
 *   ratio: SeriesPattern1<?Multiple>,
 * }} PriceWithRatio
 */
const _PriceWithRatio = _s({
  value: [_i1, '*'],
  ratio: [_i1, '*_ratio'],
});

/**
 * @typedef {{
 *   avg: PriceWithRatio,
 *   capitalizedPrice: SeriesPattern1<?Dollars>,
 *   p5: SeriesPattern1<?Dollars>,
 *   p10: SeriesPattern1<?Dollars>,
 *   p15: SeriesPattern1<?Dollars>,
 *   p20: SeriesPattern1<?Dollars>,
 *   p25: SeriesPattern1<?Dollars>,
 *   p30: SeriesPattern1<?Dollars>,
 *   p35: SeriesPattern1<?Dollars>,
 *   p40: SeriesPattern1<?Dollars>,
 *   p45: SeriesPattern1<?Dollars>,
 *   median: SeriesPattern1<?Dollars>,
 *   p55: SeriesPattern1<?Dollars>,
 *   p60: SeriesPattern1<?Dollars>,
 *   p65: SeriesPattern1<?Dollars>,
 *   p70: SeriesPattern1<?Dollars>,
 *   p75: SeriesPattern1<?Dollars>,
 *   p80: SeriesPattern1<?Dollars>,
 *   p85: SeriesPattern1<?Dollars>,
 *   p90: SeriesPattern1<?Dollars>,
 *   p95: SeriesPattern1<?Dollars>,
 *   medianCapitalizedPrice: SeriesPattern1<?Dollars>,
 * }} CostBasisByDollar
 */
const _CostBasisByDollar = _s({
  avg: [_PriceWithRatio, '*_cost_basis_by_dollar_avg'],
  capitalizedPrice: [_i1, '*_capitalized_price'],
  p5: [_i1, '*_cost_basis_by_dollar_p5'],
  p10: [_i1, '*_cost_basis_by_dollar_p10'],
  p15: [_i1, '*_cost_basis_by_dollar_p15'],
  p20: [_i1, '*_cost_basis_by_dollar_p20'],
  p25: [_i1, '*_cost_basis_by_dollar_p25'],
  p30: [_i1, '*_cost_basis_by_dollar_p30'],
  p35: [_i1, '*_cost_basis_by_dollar_p35'],
  p40: [_i1, '*_cost_basis_by_dollar_p40'],
  p45: [_i1, '*_cost_basis_by_dollar_p45'],
  median: [_i1, '*_cost_basis_by_dollar_median'],
  p55: [_i1, '*_cost_basis_by_dollar_p55'],
  p60: [_i1, '*_cost_basis_by_dollar_p60'],
  p65: [_i1, '*_cost_basis_by_dollar_p65'],
  p70: [_i1, '*_cost_basis_by_dollar_p70'],
  p75: [_i1, '*_cost_basis_by_dollar_p75'],
  p80: [_i1, '*_cost_basis_by_dollar_p80'],
  p85: [_i1, '*_cost_basis_by_dollar_p85'],
  p90: [_i1, '*_cost_basis_by_dollar_p90'],
  p95: [_i1, '*_cost_basis_by_dollar_p95'],
  medianCapitalizedPrice: [_i1, '*_median_capitalized_price'],
});

/**
 * @typedef {{
 *   byCoin: CostBasisByCoin,
 *   byDollar: CostBasisByDollar,
 *   supplyDensity: Density,
 *   capitalDensity: Density,
 * }} CostBasis
 */
const _CostBasis = _s({
  byCoin: [_CostBasisByCoin, '*'],
  byDollar: [_CostBasisByDollar, '*'],
  supplyDensity: [_Density, '*_supply_density'],
  capitalDensity: [_Density, '*_capital_density'],
});

/**
 * @typedef {{
 *   byCoin: CostBasisByCoin,
 *   byDollar: CostBasisByDollar,
 *   min: SeriesPattern1<?Dollars>,
 *   max: SeriesPattern1<?Dollars>,
 *   supplyDensity: Density,
 *   capitalDensity: Density,
 * }} HoldersCostBasis
 */
const _HoldersCostBasis = _s({
  byCoin: [_CostBasisByCoin, '*'],
  byDollar: [_CostBasisByDollar, '*'],
  min: [_i1, '*_cost_basis_min'],
  max: [_i1, '*_cost_basis_max'],
  supplyDensity: [_Density, '*_supply_density'],
  capitalDensity: [_Density, '*_capital_density'],
});

/**
 * @template A
 * @typedef {{
 *   _1w: PriceWithRatio,
 *   _8d: PriceWithRatio,
 *   _13d: PriceWithRatio,
 *   _21d: PriceWithRatio,
 *   _1m: PriceWithRatio,
 *   _34d: PriceWithRatio,
 *   _50d: PriceWithRatio,
 *   _55d: PriceWithRatio,
 *   _89d: PriceWithRatio,
 *   _144d: PriceWithRatio,
 *   _200d: A,
 *   _1y: PriceWithRatio,
 *   _2y: PriceWithRatio,
 *   _200w: PriceWithRatio,
 *   _4y: PriceWithRatio,
 * }} Ema
 */
/** @type {_Make} */
const _Ema = (c, b, f0) => _n(c, b, {
  _1w: [_PriceWithRatio, '*_1w'],
  _8d: [_PriceWithRatio, '*_8d'],
  _13d: [_PriceWithRatio, '*_13d'],
  _21d: [_PriceWithRatio, '*_21d'],
  _1m: [_PriceWithRatio, '*_1m'],
  _34d: [_PriceWithRatio, '*_34d'],
  _50d: [_PriceWithRatio, '*_50d'],
  _55d: [_PriceWithRatio, '*_55d'],
  _89d: [_PriceWithRatio, '*_89d'],
  _144d: [_PriceWithRatio, '*_144d'],
  _200d: [f0, '*_200d'],
  _1y: [_PriceWithRatio, '*_1y'],
  _2y: [_PriceWithRatio, '*_2y'],
  _200w: [_PriceWithRatio, '*_200w'],
  _4y: [_PriceWithRatio, '*_4y'],
});

/**
 * @typedef {{
 *   _1w: SeriesPattern1<?Dollars>,
 *   _1m: SeriesPattern1<?Dollars>,
 *   _1y: SeriesPattern1<?Dollars>,
 * }} PriceMinMax
 */
const _PriceMinMax = _s({
  _1w: [_i1, '*_1w'],
  _1m: [_i1, '*_1m'],
  _1y: [_i1, '*_1y'],
});

/**
 * @typedef {{
 *   min: PriceMinMax,
 *   max: PriceMinMax,
 * }} MarketRange
 */
const _MarketRange = _s({
  min: [_PriceMinMax, '*_min'],
  max: [_PriceMinMax, '*_max'],
});

/**
 * @typedef {{
 *   _24h: SeriesPattern1<?Percent>,
 *   _1w: SeriesPattern1<?Percent>,
 *   _1m: SeriesPattern1<?Percent>,
 *   _3m: SeriesPattern1<?Percent>,
 *   _6m: SeriesPattern1<?Percent>,
 *   _1y: SeriesPattern1<?Percent>,
 *   _2y: SeriesPattern1<?Percent>,
 *   _3y: SeriesPattern1<?Percent>,
 *   _4y: SeriesPattern1<?Percent>,
 *   _5y: SeriesPattern1<?Percent>,
 *   _6y: SeriesPattern1<?Percent>,
 *   _8y: SeriesPattern1<?Percent>,
 *   _10y: SeriesPattern1<?Percent>,
 * }} ByLookbackPeriod
 */
const _ByLookbackPeriod = _s({
  _24h: [_i1, '*_24h'],
  _1w: [_i1, '*_1w'],
  _1m: [_i1, '*_1m'],
  _3m: [_i1, '*_3m'],
  _6m: [_i1, '*_6m'],
  _1y: [_i1, '*_1y'],
  _2y: [_i1, '*_2y'],
  _3y: [_i1, '*_3y'],
  _4y: [_i1, '*_4y'],
  _5y: [_i1, '*_5y'],
  _6y: [_i1, '*_6y'],
  _8y: [_i1, '*_8y'],
  _10y: [_i1, '*_10y'],
});

/**
 * @typedef {{
 *   high: SeriesPattern1<?Dollars>,
 *   drawdown: SeriesPattern1<?Percent>,
 *   daysSince: SeriesPattern1<?Days>,
 *   daysBetweenMax: SeriesPattern1<?Days>,
 * }} Ath
 */
const _Ath = _s({
  high: [_i1, '*_ath'],
  drawdown: [_i1, '*_drawdown'],
  daysSince: [_i1, 'days_since_*_ath'],
  daysBetweenMax: [_i1, 'days_between_*_ath_max'],
});

/**
 * @typedef {{
 *   ath: Ath,
 *   returns: ByLookbackPeriod,
 *   range: MarketRange,
 *   sma: Ema<MarketPriceWithRatio>,
 *   ema: Ema<PriceWithRatio>,
 * }} Market
 */
const _Market = _s({
  ath: [_Ath, '*'],
  returns: [_ByLookbackPeriod, '*_return'],
  range: [_MarketRange, '*'],
  sma: [(c, b) => _Ema(c, b, _MarketPriceWithRatio), '*_sma'],
  ema: [(c, b) => _Ema(c, b, _PriceWithRatio), '*_ema'],
});

/**
 * @typedef {{
 *   supply: SeriesPattern1<?Bitcoin>,
 *   count: SeriesPattern1<Count>,
 * }} UtxoSetSeries
 */
const _UtxoSetSeries = _s({
  supply: [_i1, '*'],
  count: [_i1, 'utxo_count'],
});

/**
 * @template A
 * @typedef {{
 *   value: SeriesPattern1<A>,
 *   atl: SeriesPattern1<A>,
 *   rebound: SeriesPattern1<?Percent>,
 * }} HashPriceValue
 */
const _HashPriceValue = _s({
  value: [_i1, '*'],
  atl: [_i1, '*_atl'],
  rebound: [_i1, '*_rebound'],
});

/**
 * @typedef {{
 *   _1w: SeriesPattern1<?Hashrate>,
 *   _1m: SeriesPattern1<?Hashrate>,
 *   _2m: SeriesPattern1<?Hashrate>,
 *   _1y: SeriesPattern1<?Hashrate>,
 * }} HashRateSma
 */
const _HashRateSma = _s({
  _1w: [_i1, '*_1w'],
  _1m: [_i1, '*_1m'],
  _2m: [_i1, '*_2m'],
  _1y: [_i1, '*_1y'],
});

/**
 * @typedef {{
 *   value: SeriesPattern1<?Hashrate>,
 *   sma: HashRateSma,
 *   ath: SeriesPattern1<?Hashrate>,
 *   drawdown: SeriesPattern1<?Percent>,
 * }} Rate
 */
const _Rate = _s({
  value: [_i1, '*'],
  sma: [_HashRateSma, '*_sma'],
  ath: [_i1, '*_ath'],
  drawdown: [_i1, '*_drawdown'],
});

/**
 * @template A
 * @typedef {{
 *   tx: SeriesPattern19<A>,
 *   min: SeriesPattern1<A>,
 *   max: SeriesPattern1<A>,
 *   p10: SeriesPattern1<A>,
 *   p25: SeriesPattern1<A>,
 *   median: SeriesPattern1<A>,
 *   p75: SeriesPattern1<A>,
 *   p90: SeriesPattern1<A>,
 * }} PerTxDistribution
 */
const _PerTxDistribution = _s({
  tx: [_i19, '*'],
  min: [_i1, '*_min'],
  max: [_i1, '*_max'],
  p10: [_i1, '*_p10'],
  p25: [_i1, '*_p25'],
  median: [_i1, '*_median'],
  p75: [_i1, '*_p75'],
  p90: [_i1, '*_p90'],
});

/**
 * @typedef {{
 *   btc: SeriesPattern1<?Bitcoin>,
 *   usd: SeriesPattern1<?Dollars>,
 * }} Value
 */
const _Value = _s({
  btc: [_i1, '*'],
  usd: [_i1, '*_usd'],
});

/**
 * @typedef {{
 *   p95ToP98: Value,
 *   p98ToP99: Value,
 *   p99ToP99_5: Value,
 *   p99_5ToP99_9: Value,
 *   belowP99_9: Value,
 * }} EntrySupply
 */
const _EntrySupply = _s({
  p95ToP98: [_Value, '*_p95_to_p98'],
  p98ToP99: [_Value, '*_p98_to_p99'],
  p99ToP99_5: [_Value, '*_p99_to_p99_5'],
  p99_5ToP99_9: [_Value, '*_p99_5_to_p99_9'],
  belowP99_9: [_Value, '*_below_p99_9'],
});

/**
 * @typedef {{
 *   supplyInLossShareThreshold: Percentiles<?Percent>,
 *   floor: Percentiles<?Dollars>,
 *   costBasisAboveP95Floor: Levels,
 *   entrySupply: EntrySupply,
 *   entryCapital: EntryCapital,
 * }} Mode
 */
const _Mode = _s({
  supplyInLossShareThreshold: [_Percentiles, '*_supply_in_loss_share_threshold'],
  floor: [_Percentiles, '*_floor'],
  costBasisAboveP95Floor: [_Levels, '*_cost_basis_above_p95_floor'],
  entrySupply: [_EntrySupply, '*_entry_supply'],
  entryCapital: [_EntryCapital, '*_entry_capital'],
});

/**
 * @typedef {{
 *   awake: Mode,
 *   mobile: Mode,
 *   unweighted: Mode,
 * }} Bedrock
 */
const _Bedrock = _s({
  awake: [_Mode, '*_awake'],
  mobile: [_Mode, '*_mobile'],
  unweighted: [_Mode, '*_unweighted'],
});

/**
 * @typedef {{
 *   vaulted: Value,
 *   hodledOrLost: Value,
 *   active: Value,
 * }} CointimeSupply
 */
const _CointimeSupply = _s({
  vaulted: [_Value, 'vaulted_*'],
  hodledOrLost: [_Value, 'hodled_or_lost_*'],
  active: [_Value, 'active_*'],
});

/**
 * @typedef {{
 *   total: Value,
 *   inLoss: InLoss,
 * }} MobileSupply
 */
const _MobileSupply = _s({
  total: [_Value, '*'],
  inLoss: [_InLoss, '*_in_loss_share'],
});

/**
 * @typedef {{
 *   supply: MobileSupply,
 *   capital: MobileCapital,
 *   costBasis: CostBasis,
 * }} Awake
 */
const _Awake = _s({
  supply: [_MobileSupply, '*_supply'],
  capital: [_MobileCapital, '*'],
  costBasis: [_CostBasis, '*'],
});

/**
 * @typedef {{
 *   supply: Value,
 * }} Side
 */
const _Side = _s({
  supply: [_Value, '*'],
});

/**
 * @typedef {{
 *   mobile: Awake,
 *   immobile: Side,
 * }} CoinflowCohort
 */
const _CoinflowCohort = _s({
  mobile: [_Awake, '*_mobile'],
  immobile: [_Side, '*_immobile_supply'],
});

/**
 * @typedef {{
 *   spendingRate: SeriesPattern1<?PerDay>,
 *   spendingExposure: SeriesPattern1<?Float>,
 *   mobility: SeriesPattern1<?Ratio>,
 *   mobile: Side,
 *   immobile: Side,
 * }} CoinflowRange
 */
const _CoinflowRange = _s({
  spendingRate: [_i1, '*_spending_rate'],
  spendingExposure: [_i1, '*_spending_exposure'],
  mobility: [_i1, '*_mobility'],
  mobile: [_Side, '*_mobile_supply'],
  immobile: [_Side, '*_immobile_supply'],
});

/**
 * @typedef {{
 *   awake: Awake,
 *   dormant: Side,
 * }} CointimeCohort
 */
const _CointimeCohort = _s({
  awake: [_Awake, '*_awake'],
  dormant: [_Side, '*_dormant_supply'],
});

/**
 * @template A
 * @typedef {{
 *   all: A,
 *   sth: A,
 *   lth: A,
 *   under4m: A,
 *   under6m: A,
 *   over4m: A,
 *   over6m: A,
 * }} AgeAggregate
 */
/** @type {_Make} */
const _AgeAggregate = (c, b, f0) => _n(c, b, {
  all: [f0, ''],
  sth: [f0, 'sth'],
  lth: [f0, 'lth'],
  under4m: [f0, '*_under_4m'],
  under6m: [f0, '*_under_6m'],
  over4m: [f0, '*_over_4m'],
  over6m: [f0, '*_over_6m'],
});

/**
 * @typedef {{
 *   total: Value,
 *   share: SeriesPattern1<?Percent>,
 * }} BandSupply
 */
const _BandSupply = _s({
  total: [_Value, '*'],
  share: [_i1, '*_share'],
});

/**
 * @typedef {{
 *   supply: BandSupply,
 *   capital: BandCapital,
 * }} Band
 */
const _Band = _s({
  supply: [_BandSupply, '*_supply'],
  capital: [_BandCapital, '*'],
});

/**
 * @typedef {{
 *   profitOver1000Percent: Band,
 *   profit500To1000Percent: Band,
 *   profit300To500Percent: Band,
 *   profit200To300Percent: Band,
 *   profit100To200Percent: Band,
 *   profit90To100Percent: Band,
 *   profit80To90Percent: Band,
 *   profit70To80Percent: Band,
 *   profit60To70Percent: Band,
 *   profit50To60Percent: Band,
 *   profit40To50Percent: Band,
 *   profit30To40Percent: Band,
 *   profit20To30Percent: Band,
 *   profit10To20Percent: Band,
 *   profit0To10Percent: Band,
 *   loss0To10Percent: Band,
 *   loss10To20Percent: Band,
 *   loss20To30Percent: Band,
 *   loss30To40Percent: Band,
 *   loss40To50Percent: Band,
 *   loss50To60Percent: Band,
 *   loss60To70Percent: Band,
 *   loss70To80Percent: Band,
 *   loss80To90Percent: Band,
 *   loss90To100Percent: Band,
 * }} ProfitabilityRange
 */
const _ProfitabilityRange = _s({
  profitOver1000Percent: [_Band, '*_profit_over_1000_percent'],
  profit500To1000Percent: [_Band, '*_profit_500_to_1000_percent'],
  profit300To500Percent: [_Band, '*_profit_300_to_500_percent'],
  profit200To300Percent: [_Band, '*_profit_200_to_300_percent'],
  profit100To200Percent: [_Band, '*_profit_100_to_200_percent'],
  profit90To100Percent: [_Band, '*_profit_90_to_100_percent'],
  profit80To90Percent: [_Band, '*_profit_80_to_90_percent'],
  profit70To80Percent: [_Band, '*_profit_70_to_80_percent'],
  profit60To70Percent: [_Band, '*_profit_60_to_70_percent'],
  profit50To60Percent: [_Band, '*_profit_50_to_60_percent'],
  profit40To50Percent: [_Band, '*_profit_40_to_50_percent'],
  profit30To40Percent: [_Band, '*_profit_30_to_40_percent'],
  profit20To30Percent: [_Band, '*_profit_20_to_30_percent'],
  profit10To20Percent: [_Band, '*_profit_10_to_20_percent'],
  profit0To10Percent: [_Band, '*_profit_0_to_10_percent'],
  loss0To10Percent: [_Band, '*_loss_0_to_10_percent'],
  loss10To20Percent: [_Band, '*_loss_10_to_20_percent'],
  loss20To30Percent: [_Band, '*_loss_20_to_30_percent'],
  loss30To40Percent: [_Band, '*_loss_30_to_40_percent'],
  loss40To50Percent: [_Band, '*_loss_40_to_50_percent'],
  loss50To60Percent: [_Band, '*_loss_50_to_60_percent'],
  loss60To70Percent: [_Band, '*_loss_60_to_70_percent'],
  loss70To80Percent: [_Band, '*_loss_70_to_80_percent'],
  loss80To90Percent: [_Band, '*_loss_80_to_90_percent'],
  loss90To100Percent: [_Band, '*_loss_90_to_100_percent'],
});

/**
 * @typedef {{
 *   all: ProfitabilityRange,
 *   sth: ProfitabilityRange,
 *   lth: ProfitabilityRange,
 * }} Profitability
 */
const _Profitability = _s({
  all: [_ProfitabilityRange, ''],
  sth: [_ProfitabilityRange, '*'],
  lth: [_ProfitabilityRange, 'lth'],
});

/**
 * @typedef {{
 *   _24h: Value,
 *   _1w: Value,
 *   _1m: Value,
 *   _1y: Value,
 * }} OpReturnWindows
 */
const _OpReturnWindows = _s({
  _24h: [_Value, '*_24h'],
  _1w: [_Value, '*_1w'],
  _1m: [_Value, '*_1m'],
  _1y: [_Value, '*_1y'],
});

/**
 * @typedef {{
 *   avg: OpReturnWindows,
 * }} PoolFees
 */
const _PoolFees = _s({
  avg: [_OpReturnWindows, '*'],
});

/**
 * @typedef {{
 *   cumulative: Value,
 *   sum: OpReturnWindows,
 * }} Coinbase
 */
const _Coinbase = _s({
  cumulative: [_Value, '*_cumulative'],
  sum: [_OpReturnWindows, '*_sum'],
});

/**
 * @typedef {{
 *   btc: SeriesPattern18<?Bitcoin>,
 *   usd: SeriesPattern18<?Dollars>,
 * }} OpReturnValue
 */
const _OpReturnValue = _s({
  btc: [_i18, '*'],
  usd: [_i18, '*_usd'],
});

/**
 * @typedef {{
 *   block: OpReturnValue,
 *   cumulative: Value,
 *   sum: OpReturnWindows,
 * }} ValueFlow
 */
const _ValueFlow = _s({
  block: [_OpReturnValue, '*'],
  cumulative: [_Value, '*_cumulative'],
  sum: [_OpReturnWindows, '*_sum'],
});

/**
 * @typedef {{
 *   transferVolume: ValueFlow,
 * }} Activity
 */
const _Activity = _s({
  transferVolume: [_ValueFlow, '*'],
});

/**
 * @typedef {{
 *   block: OpReturnValue,
 *   cumulative: Value,
 *   sum: OpReturnWindows,
 *   inProfit: ValueFlow,
 *   inLoss: ValueFlow,
 * }} HoldersValueFlow
 */
const _HoldersValueFlow = _s({
  block: [_OpReturnValue, '*'],
  cumulative: [_Value, '*_cumulative'],
  sum: [_OpReturnWindows, '*_sum'],
  inProfit: [_ValueFlow, '*_in_profit'],
  inLoss: [_ValueFlow, '*_in_loss'],
});

/**
 * @typedef {{
 *   cumulative: SeriesPattern1<?Percent>,
 *   _24h: SeriesPattern1<?Percent>,
 *   _1w: SeriesPattern1<?Percent>,
 *   _1m: SeriesPattern1<?Percent>,
 *   _1y: SeriesPattern1<?Percent>,
 * }} Share
 */
const _Share = _s({
  cumulative: [_i1, '*_cumulative'],
  _24h: [_i1, '*_24h'],
  _1w: [_i1, '*_1w'],
  _1m: [_i1, '*_1m'],
  _1y: [_i1, '*_1y'],
});

/**
 * @typedef {{
 *   block: OpReturnValue,
 *   cumulative: Value,
 *   sum: OpReturnWindows,
 *   avg: OpReturnWindows,
 *   min: OpReturnWindows,
 *   max: OpReturnWindows,
 *   p10: OpReturnWindows,
 *   p25: OpReturnWindows,
 *   median: OpReturnWindows,
 *   p75: OpReturnWindows,
 *   p90: OpReturnWindows,
 *   share: Share,
 * }} ValueFlowStats
 */
const _ValueFlowStats = _s({
  block: [_OpReturnValue, '*'],
  cumulative: [_Value, '*_cumulative'],
  sum: [_OpReturnWindows, '*_sum'],
  avg: [_OpReturnWindows, '*_avg'],
  min: [_OpReturnWindows, '*_min'],
  max: [_OpReturnWindows, '*_max'],
  p10: [_OpReturnWindows, '*_p10'],
  p25: [_OpReturnWindows, '*_p25'],
  median: [_OpReturnWindows, '*_median'],
  p75: [_OpReturnWindows, '*_p75'],
  p90: [_OpReturnWindows, '*_p90'],
  share: [_Share, 'fee_share'],
});

/**
 * @typedef {{
 *   block: OpReturnValue,
 *   cumulative: Value,
 *   sum: OpReturnWindows,
 *   share: Share,
 * }} MiningValueFlow
 */
const _MiningValueFlow = _s({
  block: [_OpReturnValue, '*'],
  cumulative: [_Value, '*_cumulative'],
  sum: [_OpReturnWindows, '*_sum'],
  share: [_Share, '*_share'],
});

/**
 * @typedef {{
 *   coinbase: ValueFlow,
 *   subsidy: MiningValueFlow,
 *   fees: ValueFlowStats,
 *   unclaimed: ValueFlow,
 * }} Rewards
 */
const _Rewards = _s({
  coinbase: [_ValueFlow, '*'],
  subsidy: [_MiningValueFlow, 'subsidy'],
  fees: [_ValueFlowStats, 'fees'],
  unclaimed: [_ValueFlow, 'unclaimed_rewards'],
});

/**
 * @typedef {{
 *   rewards: Rewards,
 *   hashrate: Rate,
 *   hashPrice: HashPriceValue<?HashPrice>,
 *   hashValue: HashPriceValue<?HashValue>,
 * }} Mining
 */
const _Mining = _s({
  rewards: [_Rewards, 'coinbase'],
  hashrate: [_Rate, 'hashrate'],
  hashPrice: [_HashPriceValue, '*_price'],
  hashValue: [_HashPriceValue, '*_value'],
});

/**
 * @typedef {{
 *   block: OpReturnValue,
 *   cumulative: Value,
 *   sum: OpReturnWindows,
 *   chainShare: Share,
 * }} FeesSeries
 */
const _FeesSeries = _s({
  block: [_OpReturnValue, '*_fees'],
  cumulative: [_Value, '*_fees_cumulative'],
  sum: [_OpReturnWindows, '*_fees_sum'],
  chainShare: [_Share, '*_fee_chain_share'],
});

/**
 * @typedef {{
 *   epoch: SeriesPattern1<HalvingEpoch>,
 *   blocksToHalving: SeriesPattern1<Count>,
 *   daysToHalving: SeriesPattern1<?Days>,
 * }} Halving
 */
const _Halving = _s({
  epoch: [_i1, '*_epoch'],
  blocksToHalving: [_i1, 'blocks_to_*'],
  daysToHalving: [_i1, 'days_to_*'],
});

/**
 * @typedef {{
 *   value: SeriesPattern1<?Difficulty>,
 *   hashrate: SeriesPattern1<?Hashrate>,
 *   adjustment: SeriesPattern1<?Percent>,
 *   epoch: SeriesPattern1<DifficultyEpoch>,
 *   blocksToRetarget: SeriesPattern1<Count>,
 *   daysToRetarget: SeriesPattern1<?Days>,
 * }} BlocksDifficulty
 */
const _BlocksDifficulty = _s({
  value: [_i1, '*'],
  hashrate: [_i1, '*_hashrate'],
  adjustment: [_i1, '*_adjustment'],
  epoch: [_i1, '*_epoch'],
  blocksToRetarget: [_i1, 'blocks_to_retarget'],
  daysToRetarget: [_i1, 'days_to_retarget'],
});

/**
 * @template A
 * @typedef {{
 *   _24h: SeriesPattern1<A>,
 *   _1w: SeriesPattern1<A>,
 *   _1m: SeriesPattern1<A>,
 *   _1y: SeriesPattern1<A>,
 * }} Windows
 */
const _Windows = _s({
  _24h: [_i1, '*_24h'],
  _1w: [_i1, '*_1w'],
  _1m: [_i1, '*_1m'],
  _1y: [_i1, '*_1y'],
});

/**
 * @typedef {{
 *   block: SeriesPattern18<Count>,
 *   sum: Windows<Count>,
 * }} NewAddressCount
 */
const _NewAddressCount = _s({
  block: [_i18, '*'],
  sum: [_Windows, '*_sum'],
});

/**
 * @template A
 * @typedef {{
 *   absolute: Windows<A>,
 *   percent: AddressesWindows,
 * }} Delta
 */
const _Delta = _s({
  absolute: [_Windows, '*'],
  percent: [_AddressesWindows, '*'],
});

/**
 * @typedef {{
 *   total: SeriesPattern1<?Dollars>,
 *   realizedCap: SeriesPattern1<?Dollars>,
 *   inProfit: Part,
 *   inLoss: Part,
 *   delta: Delta<?Dollars>,
 *   share: SeriesPattern1<?Percent>,
 * }} SharedCapital
 */
const _SharedCapital = _s({
  total: [_i1, '*_capital'],
  realizedCap: [_i1, '*_realized_cap'],
  inProfit: [_Part, '*_capital_in_profit'],
  inLoss: [_Part, '*_capital_in_loss'],
  delta: [_Delta, '*_capital_delta'],
  share: [_i1, '*_capital_share'],
});

/**
 * @typedef {{
 *   total: Value,
 *   inProfit: PredicateSupply,
 *   inLoss: PredicateSupply,
 *   delta: Delta<?Bitcoin>,
 *   share: SeriesPattern1<?Percent>,
 * }} SharedSupply
 */
const _SharedSupply = _s({
  total: [_Value, '*'],
  inProfit: [_PredicateSupply, '*_in_profit'],
  inLoss: [_PredicateSupply, '*_in_loss'],
  delta: [_Delta, '*_delta'],
  share: [_i1, '*_share'],
});

/**
 * @typedef {{
 *   total: SeriesPattern1<?Dollars>,
 *   realizedCap: SeriesPattern1<?Dollars>,
 *   inProfit: Part,
 *   inLoss: Part,
 *   delta: Delta<?Dollars>,
 * }} Capital
 */
const _Capital = _s({
  total: [_i1, '*'],
  realizedCap: [_i1, 'realized_cap'],
  inProfit: [_Part, '*_in_profit'],
  inLoss: [_Part, '*_in_loss'],
  delta: [_Delta, '*_delta'],
});

/**
 * @typedef {{
 *   total: SpotValue,
 *   inProfit: PredicateSupply,
 *   inLoss: PredicateSupply,
 *   delta: Delta<?Bitcoin>,
 * }} HoldersSupply
 */
const _HoldersSupply = _s({
  total: [_SpotValue, 'circulating_*'],
  inProfit: [_PredicateSupply, '*_in_profit'],
  inLoss: [_PredicateSupply, '*_in_loss'],
  delta: [_Delta, '*_delta'],
});

/**
 * @typedef {{
 *   total: SeriesPattern1<?Dollars>,
 *   realizedCap: SeriesPattern1<?Dollars>,
 *   delta: Delta<?Dollars>,
 *   share: SeriesPattern1<?Percent>,
 * }} CapitalViews
 */
const _CapitalViews = _s({
  total: [_i1, '*_capital'],
  realizedCap: [_i1, '*_realized_cap'],
  delta: [_Delta, '*_capital_delta'],
  share: [_i1, '*_capital_share'],
});

/**
 * @typedef {{
 *   total: Value,
 *   delta: Delta<?Bitcoin>,
 *   share: SeriesPattern1<?Percent>,
 * }} CohortSupply
 */
const _CohortSupply = _s({
  total: [_Value, '*'],
  delta: [_Delta, '*_delta'],
  share: [_i1, '*_share'],
});

/**
 * @typedef {{
 *   total: Value,
 *   delta: Delta<?Bitcoin>,
 *   share: SeriesPattern1<?Percent>,
 *   inProfit: Value,
 *   inLoss: Value,
 * }} CreationSupply
 */
const _CreationSupply = _s({
  total: [_Value, '*'],
  delta: [_Delta, '*_delta'],
  share: [_i1, '*_share'],
  inProfit: [_Value, '*_in_profit'],
  inLoss: [_Value, '*_in_loss'],
});

/**
 * @template A, B
 * @typedef {{
 *   value: SeriesPattern1<A>,
 *   delta: Delta<B>,
 * }} WithDelta
 */
const _WithDelta = _s({
  value: [_i1, '*'],
  delta: [_Delta, '*_delta'],
});

/**
 * @typedef {{
 *   circulating: SeriesPattern1<?Bitcoin>,
 *   burned: ValueFlow,
 *   inflationRate: SeriesPattern1<?Percent>,
 *   velocity: Velocity,
 *   marketCap: WithDelta<?Dollars, ?Dollars>,
 * }} Supply
 */
const _Supply = _s({
  circulating: [_i1, '*'],
  burned: [_ValueFlow, 'burned'],
  inflationRate: [_i1, 'inflation_rate'],
  velocity: [_Velocity, 'velocity'],
  marketCap: [_WithDelta, 'market_cap'],
});

/**
 * @typedef {{
 *   count: WithDelta<Count, CountSigned>,
 * }} BalanceUtxos
 */
const _BalanceUtxos = _s({
  count: [_WithDelta, '*'],
});

/**
 * @typedef {{
 *   funded: WithDelta<Count, CountSigned>,
 *   utxos: BalanceUtxos,
 * }} ZeroBalance
 */
const _ZeroBalance = _s({
  funded: [_WithDelta, '*_address_count'],
  utxos: [_BalanceUtxos, '*_utxo_count'],
});

/**
 * @typedef {{
 *   total: SeriesPattern1<?Dollars>,
 *   realizedCap: SeriesPattern1<?Dollars>,
 *   delta: Delta<?Dollars>,
 *   share: SeriesPattern1<?Percent>,
 *   inProfit: SeriesPattern1<?Dollars>,
 *   inLoss: SeriesPattern1<?Dollars>,
 * }} CreationCapital
 */
const _CreationCapital = _s({
  total: [_i1, '*_capital'],
  realizedCap: [_i1, '*_realized_cap'],
  delta: [_Delta, '*_capital_delta'],
  share: [_i1, '*_capital_share'],
  inProfit: [_i1, '*_capital_in_profit'],
  inLoss: [_i1, '*_capital_in_loss'],
});

/**
 * @typedef {{
 *   total: Value,
 *   delta: Delta<?Bitcoin>,
 *   share: SeriesPattern1<?Percent>,
 *   inProfit: Value,
 *   inLoss: Value,
 *   matured: ValueFlow,
 * }} AgeCreationSupply
 */
const _AgeCreationSupply = _s({
  total: [_Value, '*_supply'],
  delta: [_Delta, '*_supply_delta'],
  share: [_i1, '*_supply_share'],
  inProfit: [_Value, '*_supply_in_profit'],
  inLoss: [_Value, '*_supply_in_loss'],
  matured: [_ValueFlow, '*_matured_supply'],
});

/**
 * @typedef {{
 *   block: SeriesPattern18<Bytes>,
 *   cumulative: SeriesPattern1<Bytes>,
 *   sum: Windows<Bytes>,
 *   chainShare: Share,
 * }} DataBytesSeries
 */
const _DataBytesSeries = _s({
  block: [_i18, '*_bytes'],
  cumulative: [_i1, '*_bytes_cumulative'],
  sum: [_Windows, '*_bytes_sum'],
  chainShare: [_Share, '*_chain_share'],
});

/**
 * @typedef {{
 *   flag: SeriesPattern19<Boolean>,
 *   block: SeriesPattern18<Count>,
 *   cumulative: SeriesPattern1<Count>,
 *   sum: Windows<Count>,
 *   share: Share,
 *   fees: FeesSeries,
 * }} Inscription
 */
const _Inscription = _s({
  flag: [_i19, 'has_*'],
  block: [_i18, '*_tx_count'],
  cumulative: [_i1, '*_tx_count_cumulative'],
  sum: [_Windows, '*_tx_count_sum'],
  share: [_Share, '*_tx_share'],
  fees: [_FeesSeries, '*'],
});

/**
 * @typedef {{
 *   flag: SeriesPattern19<Boolean>,
 *   block: SeriesPattern18<Count>,
 *   cumulative: SeriesPattern1<Count>,
 *   sum: Windows<Count>,
 *   share: Share,
 * }} TransactionsFlagged
 */
const _TransactionsFlagged = _s({
  flag: [_i19, 'is_*'],
  block: [_i18, '*_tx_count'],
  cumulative: [_i1, '*_tx_count_cumulative'],
  sum: [_Windows, '*_tx_count_sum'],
  share: [_Share, '*_tx_share'],
});

/**
 * @typedef {{
 *   coinjoin: TransactionsFlagged,
 *   consolidation: TransactionsFlagged,
 *   batchPayout: TransactionsFlagged,
 * }} Patterns
 */
const _Patterns = _s({
  coinjoin: [_TransactionsFlagged, '*'],
  consolidation: [_TransactionsFlagged, 'consolidation'],
  batchPayout: [_TransactionsFlagged, 'batch_payout'],
});

/**
 * @typedef {{
 *   fee: PerTxDistribution<Sats>,
 *   feeRate: SeriesPattern19<?FeeRate>,
 *   effectiveFeeRate: PerTxDistribution<?FeeRate>,
 *   cpfpParent: TransactionsFlagged,
 *   cpfpChild: TransactionsFlagged,
 * }} Fees
 */
const _Fees = _s({
  fee: [_PerTxDistribution, '*_fee'],
  feeRate: [_i19, '*_fee_rate'],
  effectiveFeeRate: [_PerTxDistribution, '*_effective_fee_rate'],
  cpfpParent: [_TransactionsFlagged, 'cpfp_parent'],
  cpfpChild: [_TransactionsFlagged, 'cpfp_child'],
});

/**
 * @typedef {{
 *   block: SeriesPattern18<Count>,
 *   cumulative: SeriesPattern1<Count>,
 *   sum: Windows<Count>,
 *   share: Share,
 * }} TxCount
 */
const _TxCount = _s({
  block: [_i18, '*_count'],
  cumulative: [_i1, '*_count_cumulative'],
  sum: [_Windows, '*_count_sum'],
  share: [_Share, '*_share'],
});

/**
 * @typedef {{
 *   flag: SeriesPattern19<Boolean>,
 *   block: SeriesPattern18<Count>,
 *   cumulative: SeriesPattern1<Count>,
 *   sum: Windows<Count>,
 *   share: Share,
 * }} Flagged
 */
const _Flagged = _s({
  flag: [_i19, 'has_*'],
  block: [_i18, '*_tx_count'],
  cumulative: [_i1, '*_tx_count_cumulative'],
  sum: [_Windows, '*_tx_count_sum'],
  share: [_Share, '*_tx_share'],
});

/**
 * @typedef {{
 *   annex: Flagged,
 *   sighashAll: Flagged,
 *   sighashNone: Flagged,
 *   sighashSingle: Flagged,
 *   sighashDefault: Flagged,
 *   sighashAnyoneCanPay: Flagged,
 * }} Signatures
 */
const _Signatures = _s({
  annex: [_Flagged, 'annex'],
  sighashAll: [_Flagged, '*_all'],
  sighashNone: [_Flagged, '*_none'],
  sighashSingle: [_Flagged, '*_single'],
  sighashDefault: [_Flagged, '*_default'],
  sighashAnyoneCanPay: [_Flagged, '*_anyone_can_pay'],
});

/**
 * @typedef {{
 *   p2pk: Flagged,
 *   p2pkh: Flagged,
 *   p2ms: Flagged,
 *   p2sh: Flagged,
 *   p2wpkh: Flagged,
 *   p2wsh: Flagged,
 *   p2tr: Flagged,
 *   p2a: Flagged,
 *   emptyScript: Flagged,
 *   unknownScript: Flagged,
 *   fakePubkey: Flagged,
 *   fakeScripthash: Flagged,
 * }} TransactionsScripts
 */
const _TransactionsScripts = _s({
  p2pk: [_Flagged, 'p2pk'],
  p2pkh: [_Flagged, 'p2pkh'],
  p2ms: [_Flagged, 'p2ms'],
  p2sh: [_Flagged, 'p2sh'],
  p2wpkh: [_Flagged, 'p2wpkh'],
  p2wsh: [_Flagged, 'p2wsh'],
  p2tr: [_Flagged, 'p2tr'],
  p2a: [_Flagged, 'p2a'],
  emptyScript: [_Flagged, 'empty_*'],
  unknownScript: [_Flagged, 'unknown_*'],
  fakePubkey: [_Flagged, 'fake_pubkey'],
  fakeScripthash: [_Flagged, 'fake_scripthash'],
});

/**
 * @typedef {{
 *   scripts: TransactionsScripts,
 *   signatures: Signatures,
 *   segwit: TxCount,
 *   rbfSignaling: TransactionsFlagged,
 *   dustOutput: Flagged,
 *   epobc: TransactionsFlagged,
 *   oneInput: TxCount,
 *   oneOutput: TxCount,
 * }} Features
 */
const _Features = _s({
  scripts: [_TransactionsScripts, 'script'],
  signatures: [_Signatures, 'sighash'],
  segwit: [_TxCount, 'segwit_*'],
  rbfSignaling: [_TransactionsFlagged, 'rbf_signaling'],
  dustOutput: [_Flagged, 'dust_output'],
  epobc: [_TransactionsFlagged, 'epobc'],
  oneInput: [_TxCount, 'one_input_*'],
  oneOutput: [_TxCount, 'one_output_*'],
});

/**
 * @template A
 * @typedef {{
 *   block: SeriesPattern18<A>,
 *   cumulative: SeriesPattern1<A>,
 *   sum: Windows<A>,
 * }} Flow
 */
const _Flow = _s({
  block: [_i18, '*'],
  cumulative: [_i1, '*_cumulative'],
  sum: [_Windows, '*_sum'],
});

/**
 * @typedef {{
 *   value: SeriesPattern1<?Float>,
 *   hodlBank: SeriesPattern1<?Float>,
 *   vocdd: Flow<?DollarDaysPerCoin>,
 * }} ReserveRisk
 */
const _ReserveRisk = _s({
  value: [_i1, '*'],
  hodlBank: [_i1, 'hodl_bank'],
  vocdd: [_Flow, 'vocdd'],
});

/**
 * @typedef {{
 *   destroyed: Flow<?DollarBlocks>,
 *   created: Flow<?DollarBlocks>,
 *   stored: Flow<?DollarBlocks>,
 * }} CointimeValue
 */
const _CointimeValue = _s({
  destroyed: [_Flow, '*_destroyed'],
  created: [_Flow, '*_created'],
  stored: [_Flow, '*_stored'],
});

/**
 * @typedef {{
 *   coindaysCreated: Flow<?CoinDays>,
 *   coindaysConsumed: Flow<?CoinDays>,
 *   wakefulness: SeriesPattern1<?Ratio>,
 *   awakeToDormant: SeriesPattern1<?Multiple>,
 *   awake: Side,
 *   dormant: Side,
 * }} CointimeRange
 */
const _CointimeRange = _s({
  coindaysCreated: [_Flow, '*_coindays_created'],
  coindaysConsumed: [_Flow, '*_coindays_consumed'],
  wakefulness: [_i1, '*_wakefulness'],
  awakeToDormant: [_i1, '*_awake_to_dormant'],
  awake: [_Side, '*_awake_supply'],
  dormant: [_Side, '*_dormant_supply'],
});

/**
 * @typedef {{
 *   coinblocksCreated: Flow<?CoinBlocks>,
 *   coinblocksDestroyed: Flow<?CoinBlocks>,
 *   coinblocksStored: Flow<?CoinBlocks>,
 *   liveliness: SeriesPattern1<?Ratio>,
 *   vaultedness: SeriesPattern1<?Ratio>,
 *   livelinessToVaultedness: SeriesPattern1<?Multiple>,
 *   concurrentLiveliness: Windows<?Ratio>,
 * }} CointimeActivity
 */
const _CointimeActivity = _s({
  coinblocksCreated: [_Flow, '*_created'],
  coinblocksDestroyed: [_Flow, '*_destroyed'],
  coinblocksStored: [_Flow, '*_stored'],
  liveliness: [_i1, 'liveliness'],
  vaultedness: [_i1, 'vaultedness'],
  livelinessToVaultedness: [_i1, 'liveliness_to_vaultedness'],
  concurrentLiveliness: [_Windows, 'concurrent_liveliness'],
});

/**
 * @typedef {{
 *   profit: Flow<?Dollars>,
 *   loss: Flow<?Dollars>,
 *   netPnl: Flow<?Dollars>,
 *   grossPnl: Flow<?Dollars>,
 *   profitToLoss: Windows<?Multiple>,
 *   valueDestroyed: Flow<?Dollars>,
 *   peakRegret: Flow<?Dollars>,
 *   sopr: Windows<?Multiple>,
 *   sellSideRiskRatio: Windows<?Ratio>,
 * }} HoldersRealized
 */
const _HoldersRealized = _s({
  profit: [_Flow, '*_realized_profit'],
  loss: [_Flow, '*_realized_loss'],
  netPnl: [_Flow, '*_net_realized_pnl'],
  grossPnl: [_Flow, '*_gross_realized_pnl'],
  profitToLoss: [_Windows, '*_realized_profit_to_loss'],
  valueDestroyed: [_Flow, '*_value_destroyed'],
  peakRegret: [_Flow, '*_realized_peak_regret'],
  sopr: [_Windows, '*_sopr'],
  sellSideRiskRatio: [_Windows, '*_sell_side_risk_ratio'],
});

/**
 * @typedef {{
 *   _24h: SeriesPattern1<?Multiple>,
 *   _1w: SeriesPattern1<?Multiple>,
 *   _1m: SeriesPattern1<?Multiple>,
 *   _1y: SeriesPattern1<?Multiple>,
 *   valueCreated: Flow<?Dollars>,
 *   valueDestroyed: Flow<?Dollars>,
 * }} AdjustedSopr
 */
const _AdjustedSopr = _s({
  _24h: [_i1, '*_sopr_24h'],
  _1w: [_i1, '*_sopr_1w'],
  _1m: [_i1, '*_sopr_1m'],
  _1y: [_i1, '*_sopr_1y'],
  valueCreated: [_Flow, '*_value_created'],
  valueDestroyed: [_Flow, '*_value_destroyed'],
});

/**
 * @typedef {{
 *   profit: Flow<?Dollars>,
 *   loss: Flow<?Dollars>,
 *   netPnl: Flow<?Dollars>,
 *   grossPnl: Flow<?Dollars>,
 *   profitToLoss: Windows<?Multiple>,
 *   valueDestroyed: Flow<?Dollars>,
 *   peakRegret: Flow<?Dollars>,
 *   sopr: Windows<?Multiple>,
 *   sellSideRiskRatio: Windows<?Ratio>,
 *   adjustedSopr: AdjustedSopr,
 * }} AdjustedRealized
 */
const _AdjustedRealized = _s({
  profit: [_Flow, '*_realized_profit'],
  loss: [_Flow, '*_realized_loss'],
  netPnl: [_Flow, '*_net_realized_pnl'],
  grossPnl: [_Flow, '*_gross_realized_pnl'],
  profitToLoss: [_Windows, '*_realized_profit_to_loss'],
  valueDestroyed: [_Flow, '*_value_destroyed'],
  peakRegret: [_Flow, '*_realized_peak_regret'],
  sopr: [_Windows, '*_sopr'],
  sellSideRiskRatio: [_Windows, '*_sell_side_risk_ratio'],
  adjustedSopr: [_AdjustedSopr, '*_adjusted'],
});

/**
 * @typedef {{
 *   transferVolume: HoldersValueFlow,
 *   coindaysDestroyed: Flow<?CoinDays>,
 *   dormancy: Windows<?Days>,
 * }} HoldersActivity
 */
const _HoldersActivity = _s({
  transferVolume: [_HoldersValueFlow, '*_transfer_volume'],
  coindaysDestroyed: [_Flow, '*_coindays_destroyed'],
  dormancy: [_Windows, '*_dormancy'],
});

/**
 * @typedef {{
 *   profit: Flow<?Dollars>,
 *   loss: Flow<?Dollars>,
 * }} Realized
 */
const _Realized = _s({
  profit: [_Flow, '*_profit'],
  loss: [_Flow, '*_loss'],
});

/**
 * @typedef {{
 *   funded: WithDelta<Count, CountSigned>,
 *   supply: CohortSupply,
 *   capital: CapitalViews,
 *   utxos: BalanceUtxos,
 *   activity: Activity,
 *   realized: Realized,
 * }} Balance
 */
const _Balance = _s({
  funded: [_WithDelta, '*_address_count'],
  supply: [_CohortSupply, '*_supply'],
  capital: [_CapitalViews, '*'],
  utxos: [_BalanceUtxos, '*_utxo_count'],
  activity: [_Activity, '*_transfer_volume'],
  realized: [_Realized, '*_realized'],
});

/**
 * @typedef {{
 *   transferVolume: ValueFlow,
 *   coindaysDestroyed: Flow<?CoinDays>,
 * }} CreationActivity
 */
const _CreationActivity = _s({
  transferVolume: [_ValueFlow, '*_transfer_volume'],
  coindaysDestroyed: [_Flow, '*_coindays_destroyed'],
});

/**
 * @typedef {{
 *   profit: Flow<?Dollars>,
 *   loss: Flow<?Dollars>,
 *   netPnl: Flow<?Dollars>,
 *   valueDestroyed: Flow<?Dollars>,
 * }} CreationRealized
 */
const _CreationRealized = _s({
  profit: [_Flow, '*_realized_profit'],
  loss: [_Flow, '*_realized_loss'],
  netPnl: [_Flow, '*_net_realized_pnl'],
  valueDestroyed: [_Flow, '*_value_destroyed'],
});

/**
 * @typedef {{
 *   transferVolume: HoldersValueFlow,
 *   coindaysDestroyed: Flow<?CoinDays>,
 *   coindaysCreated: Flow<?CoinDays>,
 * }} AgeCreationActivity
 */
const _AgeCreationActivity = _s({
  transferVolume: [_HoldersValueFlow, '*_transfer_volume'],
  coindaysDestroyed: [_Flow, '*_coindays_destroyed'],
  coindaysCreated: [_Flow, '*_coindays_created'],
});

/**
 * @typedef {{
 *   count: WithDelta<Count, CountSigned>,
 *   spentCount: Flow<Count>,
 * }} CohortUtxos
 */
const _CohortUtxos = _s({
  count: [_WithDelta, '*_utxo_count'],
  spentCount: [_Flow, '*_spent_utxo_count'],
});

/**
 * @typedef {{
 *   supply: HoldersSupply,
 *   capital: Capital,
 *   utxos: CohortUtxos,
 *   activity: HoldersActivity,
 *   realized: AdjustedRealized,
 *   unrealized: Unrealized,
 *   costBasis: HoldersCostBasis,
 * }} Metrics
 */
const _Metrics = _s({
  supply: [_HoldersSupply, '*'],
  capital: [_Capital, 'capital'],
  utxos: [_CohortUtxos, ''],
  activity: [_HoldersActivity, ''],
  realized: [_AdjustedRealized, ''],
  unrealized: [_Unrealized, ''],
  costBasis: [_HoldersCostBasis, ''],
});

/**
 * @typedef {{
 *   supply: CohortSupply,
 *   capital: CapitalViews,
 *   utxos: CohortUtxos,
 *   activity: Activity,
 *   realized: Realized,
 *   costBasis: CohortCostBasis,
 * }} Cohort
 */
const _Cohort = _s({
  supply: [_CohortSupply, '*_supply'],
  capital: [_CapitalViews, '*'],
  utxos: [_CohortUtxos, '*'],
  activity: [_Activity, '*_transfer_volume'],
  realized: [_Realized, '*_realized'],
  costBasis: [_CohortCostBasis, '*'],
});

/**
 * @typedef {{
 *   p2pk: Cohort,
 *   p2pkh: Cohort,
 *   p2ms: Cohort,
 *   p2sh: Cohort,
 *   p2wpkh: Cohort,
 *   p2wsh: Cohort,
 *   p2tr: Cohort,
 *   p2a: Cohort,
 *   emptyScript: Cohort,
 *   unknownScript: Cohort,
 * }} UtxosSpendableType
 */
const _UtxosSpendableType = _s({
  p2pk: [_Cohort, 'p2pk'],
  p2pkh: [_Cohort, 'p2pkh'],
  p2ms: [_Cohort, 'p2ms'],
  p2sh: [_Cohort, 'p2sh'],
  p2wpkh: [_Cohort, 'p2wpkh'],
  p2wsh: [_Cohort, 'p2wsh'],
  p2tr: [_Cohort, 'p2tr'],
  p2a: [_Cohort, 'p2a'],
  emptyScript: [_Cohort, 'empty_*'],
  unknownScript: [_Cohort, 'unknown_*'],
});

/**
 * @typedef {{
 *   utxos: CohortUtxos,
 * }} Zero
 */
const _Zero = _s({
  utxos: [_CohortUtxos, '*'],
});

/**
 * @template A, B
 * @typedef {{
 *   _0Sats: A,
 *   _1To10Sats: B,
 *   _10To100Sats: B,
 *   _100To1kSats: B,
 *   _1kTo10kSats: B,
 *   _10kTo100kSats: B,
 *   _100kTo1mSats: B,
 *   _1mTo10mSats: B,
 *   _10mTo100mSats: B,
 *   _1To10Btc: B,
 *   _10To100Btc: B,
 *   _100To1kBtc: B,
 *   _1kTo10kBtc: B,
 *   _10kTo100kBtc: B,
 *   over100kBtc: B,
 * }} AmountRange
 */
/** @type {_Make} */
const _AmountRange = (c, b, f0, f1) => _n(c, b, {
  _0Sats: [f0, '*_0_sats'],
  _1To10Sats: [f1, '*_1_to_10_sats'],
  _10To100Sats: [f1, '*_10_to_100_sats'],
  _100To1kSats: [f1, '*_100_to_1k_sats'],
  _1kTo10kSats: [f1, '*_1k_to_10k_sats'],
  _10kTo100kSats: [f1, '*_10k_to_100k_sats'],
  _100kTo1mSats: [f1, '*_100k_to_1m_sats'],
  _1mTo10mSats: [f1, '*_1m_to_10m_sats'],
  _10mTo100mSats: [f1, '*_10m_to_100m_sats'],
  _1To10Btc: [f1, '*_1_to_10_btc'],
  _10To100Btc: [f1, '*_10_to_100_btc'],
  _100To1kBtc: [f1, '*_100_to_1k_btc'],
  _1kTo10kBtc: [f1, '*_1k_to_10k_btc'],
  _10kTo100kBtc: [f1, '*_10k_to_100k_btc'],
  over100kBtc: [f1, '*_over_100k_btc'],
});

/**
 * @typedef {{
 *   amounts: AmountRange<Zero, Cohort>,
 *   types: UtxosSpendableType,
 * }} Utxos
 */
const _Utxos = _s({
  amounts: [(c, b) => _AmountRange(c, b, _Zero, _Cohort), '*'],
  types: [_UtxosSpendableType, 'script'],
});

/**
 * @template A, B, C, D, E, F
 * @typedef {{
 *   supply: A,
 *   capital: B,
 *   utxos: CohortUtxos,
 *   activity: C,
 *   realized: D,
 *   unrealized: E,
 *   costBasis: F,
 * }} CreationCohort
 */
/** @type {_Make} */
const _CreationCohort = (c, b, f0, f1, f2, f3, f4, f5) => _n(c, b, {
  supply: [f0, '*_supply'],
  capital: [f1, '*'],
  utxos: [_CohortUtxos, '*'],
  activity: [f2, '*'],
  realized: [f3, '*'],
  unrealized: [f4, '*'],
  costBasis: [f5, '*'],
});

/**
 * @typedef {{
 *   discount: CreationCohort<CreationSupply, CreationCapital, CreationActivity, CreationRealized, CreationUnrealized, CohortCostBasis>,
 *   premium: CreationCohort<CreationSupply, CreationCapital, CreationActivity, CreationRealized, CreationUnrealized, CohortCostBasis>,
 * }} Entry
 */
const _Entry = _s({
  discount: [(c, b) => _CreationCohort(c, b, _CreationSupply, _CreationCapital, _CreationActivity, _CreationRealized, _CreationUnrealized, _CohortCostBasis), '*_discount'],
  premium: [(c, b) => _CreationCohort(c, b, _CreationSupply, _CreationCapital, _CreationActivity, _CreationRealized, _CreationUnrealized, _CohortCostBasis), '*_premium'],
});

/**
 * @typedef {{
 *   all: Metrics,
 *   sth: CreationCohort<SharedSupply, SharedCapital, HoldersActivity, AdjustedRealized, Unrealized, HoldersCostBasis>,
 *   lth: CreationCohort<SharedSupply, SharedCapital, HoldersActivity, HoldersRealized, Unrealized, HoldersCostBasis>,
 *   under4m: CreationCohort<SharedSupply, SharedCapital, HoldersActivity, AdjustedRealized, Unrealized, HoldersCostBasis>,
 *   under6m: CreationCohort<SharedSupply, SharedCapital, HoldersActivity, AdjustedRealized, Unrealized, HoldersCostBasis>,
 *   over4m: CreationCohort<SharedSupply, SharedCapital, HoldersActivity, HoldersRealized, Unrealized, HoldersCostBasis>,
 *   over6m: CreationCohort<SharedSupply, SharedCapital, HoldersActivity, HoldersRealized, Unrealized, HoldersCostBasis>,
 * }} Holders
 */
const _Holders = _s({
  all: [_Metrics, 'supply'],
  sth: [(c, b) => _CreationCohort(c, b, _SharedSupply, _SharedCapital, _HoldersActivity, _AdjustedRealized, _Unrealized, _HoldersCostBasis), 'sth'],
  lth: [(c, b) => _CreationCohort(c, b, _SharedSupply, _SharedCapital, _HoldersActivity, _HoldersRealized, _Unrealized, _HoldersCostBasis), 'lth'],
  under4m: [(c, b) => _CreationCohort(c, b, _SharedSupply, _SharedCapital, _HoldersActivity, _AdjustedRealized, _Unrealized, _HoldersCostBasis), '*_under_4m'],
  under6m: [(c, b) => _CreationCohort(c, b, _SharedSupply, _SharedCapital, _HoldersActivity, _AdjustedRealized, _Unrealized, _HoldersCostBasis), '*_under_6m'],
  over4m: [(c, b) => _CreationCohort(c, b, _SharedSupply, _SharedCapital, _HoldersActivity, _HoldersRealized, _Unrealized, _HoldersCostBasis), '*_over_4m'],
  over6m: [(c, b) => _CreationCohort(c, b, _SharedSupply, _SharedCapital, _HoldersActivity, _HoldersRealized, _Unrealized, _HoldersCostBasis), '*_over_6m'],
});

/**
 * @typedef {{
 *   _2009: CreationCohort<CreationSupply, CreationCapital, CreationActivity, CreationRealized, CreationUnrealized, CohortCostBasis>,
 *   _2010: CreationCohort<CreationSupply, CreationCapital, CreationActivity, CreationRealized, CreationUnrealized, CohortCostBasis>,
 *   _2011: CreationCohort<CreationSupply, CreationCapital, CreationActivity, CreationRealized, CreationUnrealized, CohortCostBasis>,
 *   _2012: CreationCohort<CreationSupply, CreationCapital, CreationActivity, CreationRealized, CreationUnrealized, CohortCostBasis>,
 *   _2013: CreationCohort<CreationSupply, CreationCapital, CreationActivity, CreationRealized, CreationUnrealized, CohortCostBasis>,
 *   _2014: CreationCohort<CreationSupply, CreationCapital, CreationActivity, CreationRealized, CreationUnrealized, CohortCostBasis>,
 *   _2015: CreationCohort<CreationSupply, CreationCapital, CreationActivity, CreationRealized, CreationUnrealized, CohortCostBasis>,
 *   _2016: CreationCohort<CreationSupply, CreationCapital, CreationActivity, CreationRealized, CreationUnrealized, CohortCostBasis>,
 *   _2017: CreationCohort<CreationSupply, CreationCapital, CreationActivity, CreationRealized, CreationUnrealized, CohortCostBasis>,
 *   _2018: CreationCohort<CreationSupply, CreationCapital, CreationActivity, CreationRealized, CreationUnrealized, CohortCostBasis>,
 *   _2019: CreationCohort<CreationSupply, CreationCapital, CreationActivity, CreationRealized, CreationUnrealized, CohortCostBasis>,
 *   _2020: CreationCohort<CreationSupply, CreationCapital, CreationActivity, CreationRealized, CreationUnrealized, CohortCostBasis>,
 *   _2021: CreationCohort<CreationSupply, CreationCapital, CreationActivity, CreationRealized, CreationUnrealized, CohortCostBasis>,
 *   _2022: CreationCohort<CreationSupply, CreationCapital, CreationActivity, CreationRealized, CreationUnrealized, CohortCostBasis>,
 *   _2023: CreationCohort<CreationSupply, CreationCapital, CreationActivity, CreationRealized, CreationUnrealized, CohortCostBasis>,
 *   _2024: CreationCohort<CreationSupply, CreationCapital, CreationActivity, CreationRealized, CreationUnrealized, CohortCostBasis>,
 *   _2025: CreationCohort<CreationSupply, CreationCapital, CreationActivity, CreationRealized, CreationUnrealized, CohortCostBasis>,
 *   _2026: CreationCohort<CreationSupply, CreationCapital, CreationActivity, CreationRealized, CreationUnrealized, CohortCostBasis>,
 *   _2027: CreationCohort<CreationSupply, CreationCapital, CreationActivity, CreationRealized, CreationUnrealized, CohortCostBasis>,
 * }} Vintage
 */
const _Vintage = _s({
  _2009: [(c, b) => _CreationCohort(c, b, _CreationSupply, _CreationCapital, _CreationActivity, _CreationRealized, _CreationUnrealized, _CohortCostBasis), '*_2009'],
  _2010: [(c, b) => _CreationCohort(c, b, _CreationSupply, _CreationCapital, _CreationActivity, _CreationRealized, _CreationUnrealized, _CohortCostBasis), '*_2010'],
  _2011: [(c, b) => _CreationCohort(c, b, _CreationSupply, _CreationCapital, _CreationActivity, _CreationRealized, _CreationUnrealized, _CohortCostBasis), '*_2011'],
  _2012: [(c, b) => _CreationCohort(c, b, _CreationSupply, _CreationCapital, _CreationActivity, _CreationRealized, _CreationUnrealized, _CohortCostBasis), '*_2012'],
  _2013: [(c, b) => _CreationCohort(c, b, _CreationSupply, _CreationCapital, _CreationActivity, _CreationRealized, _CreationUnrealized, _CohortCostBasis), '*_2013'],
  _2014: [(c, b) => _CreationCohort(c, b, _CreationSupply, _CreationCapital, _CreationActivity, _CreationRealized, _CreationUnrealized, _CohortCostBasis), '*_2014'],
  _2015: [(c, b) => _CreationCohort(c, b, _CreationSupply, _CreationCapital, _CreationActivity, _CreationRealized, _CreationUnrealized, _CohortCostBasis), '*_2015'],
  _2016: [(c, b) => _CreationCohort(c, b, _CreationSupply, _CreationCapital, _CreationActivity, _CreationRealized, _CreationUnrealized, _CohortCostBasis), '*_2016'],
  _2017: [(c, b) => _CreationCohort(c, b, _CreationSupply, _CreationCapital, _CreationActivity, _CreationRealized, _CreationUnrealized, _CohortCostBasis), '*_2017'],
  _2018: [(c, b) => _CreationCohort(c, b, _CreationSupply, _CreationCapital, _CreationActivity, _CreationRealized, _CreationUnrealized, _CohortCostBasis), '*_2018'],
  _2019: [(c, b) => _CreationCohort(c, b, _CreationSupply, _CreationCapital, _CreationActivity, _CreationRealized, _CreationUnrealized, _CohortCostBasis), '*_2019'],
  _2020: [(c, b) => _CreationCohort(c, b, _CreationSupply, _CreationCapital, _CreationActivity, _CreationRealized, _CreationUnrealized, _CohortCostBasis), '*_2020'],
  _2021: [(c, b) => _CreationCohort(c, b, _CreationSupply, _CreationCapital, _CreationActivity, _CreationRealized, _CreationUnrealized, _CohortCostBasis), '*_2021'],
  _2022: [(c, b) => _CreationCohort(c, b, _CreationSupply, _CreationCapital, _CreationActivity, _CreationRealized, _CreationUnrealized, _CohortCostBasis), '*_2022'],
  _2023: [(c, b) => _CreationCohort(c, b, _CreationSupply, _CreationCapital, _CreationActivity, _CreationRealized, _CreationUnrealized, _CohortCostBasis), '*_2023'],
  _2024: [(c, b) => _CreationCohort(c, b, _CreationSupply, _CreationCapital, _CreationActivity, _CreationRealized, _CreationUnrealized, _CohortCostBasis), '*_2024'],
  _2025: [(c, b) => _CreationCohort(c, b, _CreationSupply, _CreationCapital, _CreationActivity, _CreationRealized, _CreationUnrealized, _CohortCostBasis), '*_2025'],
  _2026: [(c, b) => _CreationCohort(c, b, _CreationSupply, _CreationCapital, _CreationActivity, _CreationRealized, _CreationUnrealized, _CohortCostBasis), '*_2026'],
  _2027: [(c, b) => _CreationCohort(c, b, _CreationSupply, _CreationCapital, _CreationActivity, _CreationRealized, _CreationUnrealized, _CohortCostBasis), '*_2027'],
});

/**
 * @typedef {{
 *   _0: CreationCohort<CreationSupply, CreationCapital, CreationActivity, CreationRealized, CreationUnrealized, CohortCostBasis>,
 *   _1: CreationCohort<CreationSupply, CreationCapital, CreationActivity, CreationRealized, CreationUnrealized, CohortCostBasis>,
 *   _2: CreationCohort<CreationSupply, CreationCapital, CreationActivity, CreationRealized, CreationUnrealized, CohortCostBasis>,
 *   _3: CreationCohort<CreationSupply, CreationCapital, CreationActivity, CreationRealized, CreationUnrealized, CohortCostBasis>,
 *   _4: CreationCohort<CreationSupply, CreationCapital, CreationActivity, CreationRealized, CreationUnrealized, CohortCostBasis>,
 *   _5: CreationCohort<CreationSupply, CreationCapital, CreationActivity, CreationRealized, CreationUnrealized, CohortCostBasis>,
 * }} ByHalvingEpoch
 */
const _ByHalvingEpoch = _s({
  _0: [(c, b) => _CreationCohort(c, b, _CreationSupply, _CreationCapital, _CreationActivity, _CreationRealized, _CreationUnrealized, _CohortCostBasis), '*_0'],
  _1: [(c, b) => _CreationCohort(c, b, _CreationSupply, _CreationCapital, _CreationActivity, _CreationRealized, _CreationUnrealized, _CohortCostBasis), '*_1'],
  _2: [(c, b) => _CreationCohort(c, b, _CreationSupply, _CreationCapital, _CreationActivity, _CreationRealized, _CreationUnrealized, _CohortCostBasis), '*_2'],
  _3: [(c, b) => _CreationCohort(c, b, _CreationSupply, _CreationCapital, _CreationActivity, _CreationRealized, _CreationUnrealized, _CohortCostBasis), '*_3'],
  _4: [(c, b) => _CreationCohort(c, b, _CreationSupply, _CreationCapital, _CreationActivity, _CreationRealized, _CreationUnrealized, _CohortCostBasis), '*_4'],
  _5: [(c, b) => _CreationCohort(c, b, _CreationSupply, _CreationCapital, _CreationActivity, _CreationRealized, _CreationUnrealized, _CohortCostBasis), '*_5'],
});

/**
 * @typedef {{
 *   supply: AgeCreationSupply,
 *   capital: CreationCapital,
 *   utxos: CohortUtxos,
 *   activity: AgeCreationActivity,
 *   realized: CreationRealized,
 *   unrealized: CreationUnrealized,
 *   costBasis: CohortCostBasis,
 * }} Range
 */
const _Range = _s({
  supply: [_AgeCreationSupply, '*'],
  capital: [_CreationCapital, '*'],
  utxos: [_CohortUtxos, '*'],
  activity: [_AgeCreationActivity, '*'],
  realized: [_CreationRealized, '*'],
  unrealized: [_CreationUnrealized, '*'],
  costBasis: [_CohortCostBasis, '*'],
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
 * }} AgeRange
 */
/** @type {_Make} */
const _AgeRange = (c, b, f0) => _n(c, b, {
  under1h: [f0, '*_under_1h'],
  _1hTo1d: [f0, '*_1h_to_1d'],
  _1dTo1w: [f0, '*_1d_to_1w'],
  _1wTo1m: [f0, '*_1w_to_1m'],
  _1mTo2m: [f0, '*_1m_to_2m'],
  _2mTo3m: [f0, '*_2m_to_3m'],
  _3mTo4m: [f0, '*_3m_to_4m'],
  _4mTo5m: [f0, '*_4m_to_5m'],
  _5mTo6m: [f0, '*_5m_to_6m'],
  _6mTo9m: [f0, '*_6m_to_9m'],
  _9mTo1y: [f0, '*_9m_to_1y'],
  _1yTo18m: [f0, '*_1y_to_18m'],
  _18mTo2y: [f0, '*_18m_to_2y'],
  _2yTo3y: [f0, '*_2y_to_3y'],
  _3yTo4y: [f0, '*_3y_to_4y'],
  _4yTo5y: [f0, '*_4y_to_5y'],
  _5yTo6y: [f0, '*_5y_to_6y'],
  _6yTo7y: [f0, '*_6y_to_7y'],
  _7yTo8y: [f0, '*_7y_to_8y'],
  _8yTo10y: [f0, '*_8y_to_10y'],
  _10yTo12y: [f0, '*_10y_to_12y'],
  _12yTo15y: [f0, '*_12y_to_15y'],
  over15y: [f0, '*_over_15y'],
});

/**
 * @typedef {{
 *   ageRanges: AgeRange<CoinflowRange>,
 *   cohorts: AgeAggregate<CoinflowCohort>,
 * }} Coinflow
 */
const _Coinflow = _s({
  ageRanges: [(c, b) => _AgeRange(c, b, _CoinflowRange), '*'],
  cohorts: [(c, b) => _AgeAggregate(c, b, _CoinflowCohort), '*'],
});

/**
 * @typedef {{
 *   activity: CointimeActivity,
 *   ageRanges: AgeRange<CointimeRange>,
 *   cohorts: AgeAggregate<CointimeCohort>,
 *   supply: CointimeSupply,
 *   value: CointimeValue,
 *   caps: Caps,
 *   prices: Prices,
 *   adjusted: Adjusted,
 *   reserveRisk: ReserveRisk,
 * }} Cointime
 */
const _Cointime = _s({
  activity: [_CointimeActivity, 'coinblocks'],
  ageRanges: [(c, b) => _AgeRange(c, b, _CointimeRange), '*'],
  cohorts: [(c, b) => _AgeAggregate(c, b, _CointimeCohort), '*'],
  supply: [_CointimeSupply, 'supply'],
  value: [_CointimeValue, 'cointime_value'],
  caps: [_Caps, 'cap'],
  prices: [_Prices, 'vaulted'],
  adjusted: [_Adjusted, 'cointime_adjusted'],
  reserveRisk: [_ReserveRisk, 'reserve_risk'],
});

/**
 * @typedef {{
 *   ranges: AgeRange<Range>,
 *   halvingEpochs: ByHalvingEpoch,
 *   vintages: Vintage,
 * }} Age
 */
const _Age = _s({
  ranges: [(c, b) => _AgeRange(c, b, _Range), '*'],
  halvingEpochs: [_ByHalvingEpoch, 'halving_epoch'],
  vintages: [_Vintage, 'vintage'],
});

/**
 * @typedef {{
 *   outputCount: Flow<Count>,
 *   dataBytes: DataBytesSeries,
 *   txCount: Flow<Count>,
 *   txShare: Share,
 *   txVsize: Flow<VSize>,
 *   fees: FeesSeries,
 * }} Bucket
 */
const _Bucket = _s({
  outputCount: [_Flow, '*_output_count'],
  dataBytes: [_DataBytesSeries, '*_data'],
  txCount: [_Flow, '*_tx_count'],
  txShare: [_Share, '*_tx_share'],
  txVsize: [_Flow, '*_tx_vsize'],
  fees: [_FeesSeries, '*'],
});

/**
 * @typedef {{
 *   preV30Standard: Bucket,
 *   preV30Nonstandard: Bucket,
 *   oversized: Bucket,
 *   multiple: Bucket,
 * }} Policy
 */
const _Policy = _s({
  preV30Standard: [_Bucket, '*_pre_v30_standard'],
  preV30Nonstandard: [_Bucket, '*_pre_v30_nonstandard'],
  oversized: [_Bucket, '*_oversized'],
  multiple: [_Bucket, '*_multiple'],
});

/**
 * @typedef {{
 *   runes: Bucket,
 *   witnessCommitment: Bucket,
 *   veriblock: Bucket,
 *   omni: Bucket,
 *   stacks: Bucket,
 *   blockstack: Bucket,
 *   colu: Bucket,
 *   openAssets: Bucket,
 *   komodo: Bucket,
 *   coinspark: Bucket,
 *   poet: Bucket,
 *   docproof: Bucket,
 *   factom: Bucket,
 *   eternityWall: Bucket,
 *   memo: Bucket,
 *   bitproof: Bucket,
 *   ascribe: Bucket,
 *   stampery: Bucket,
 *   bareHash: Bucket,
 *   text: Bucket,
 *   empty: Bucket,
 *   unknown: Bucket,
 * }} ByKind
 */
const _ByKind = _s({
  runes: [_Bucket, '*_runes'],
  witnessCommitment: [_Bucket, '*_witness_commitment'],
  veriblock: [_Bucket, '*_veriblock'],
  omni: [_Bucket, '*_omni'],
  stacks: [_Bucket, '*_stacks'],
  blockstack: [_Bucket, '*_blockstack'],
  colu: [_Bucket, '*_colu'],
  openAssets: [_Bucket, '*_open_assets'],
  komodo: [_Bucket, '*_komodo'],
  coinspark: [_Bucket, '*_coinspark'],
  poet: [_Bucket, '*_poet'],
  docproof: [_Bucket, '*_docproof'],
  factom: [_Bucket, '*_factom'],
  eternityWall: [_Bucket, '*_eternity_wall'],
  memo: [_Bucket, '*_memo'],
  bitproof: [_Bucket, '*_bitproof'],
  ascribe: [_Bucket, '*_ascribe'],
  stampery: [_Bucket, '*_stampery'],
  bareHash: [_Bucket, '*_bare_hash'],
  text: [_Bucket, '*_text'],
  empty: [_Bucket, '*_empty'],
  unknown: [_Bucket, '*_unknown'],
});

/**
 * @typedef {{
 *   flag: SeriesPattern19<Boolean>,
 *   outputCount: Flow<Count>,
 *   dataBytes: DataBytesSeries,
 *   txCount: Flow<Count>,
 *   txShare: Share,
 *   txVsize: Flow<VSize>,
 *   fees: FeesSeries,
 *   protocols: ByKind,
 *   policies: Policy,
 * }} OpReturn
 */
const _OpReturn = _s({
  flag: [_i19, 'has_*'],
  outputCount: [_Flow, '*_output_count'],
  dataBytes: [_DataBytesSeries, '*_data'],
  txCount: [_Flow, '*_tx_count'],
  txShare: [_Share, '*_tx_share'],
  txVsize: [_Flow, '*_tx_vsize'],
  fees: [_FeesSeries, '*'],
  protocols: [_ByKind, '*'],
  policies: [_Policy, '*'],
});

/**
 * @typedef {{
 *   count: Flow<Count>,
 *   share: Share,
 *   txCount: Flow<Count>,
 *   txShare: Share,
 * }} OutputTypeSeries
 */
const _OutputTypeSeries = _s({
  count: [_Flow, '*_count'],
  share: [_Share, '*_share'],
  txCount: [_Flow, '*_tx_count'],
  txShare: [_Share, '*_tx_share'],
});

/**
 * @typedef {{
 *   p2pk: OutputTypeSeries,
 *   p2pkh: OutputTypeSeries,
 *   p2ms: OutputTypeSeries,
 *   p2sh: OutputTypeSeries,
 *   p2wpkh: OutputTypeSeries,
 *   p2wsh: OutputTypeSeries,
 *   p2tr: OutputTypeSeries,
 *   p2a: OutputTypeSeries,
 *   emptyScript: OutputTypeSeries,
 *   unknownScript: OutputTypeSeries,
 *   opReturn: OutputTypeSeries,
 * }} ByType
 */
const _ByType = _s({
  p2pk: [_OutputTypeSeries, 'p2pk_*'],
  p2pkh: [_OutputTypeSeries, 'p2pkh_*'],
  p2ms: [_OutputTypeSeries, 'p2ms_*'],
  p2sh: [_OutputTypeSeries, 'p2sh_*'],
  p2wpkh: [_OutputTypeSeries, 'p2wpkh_*'],
  p2wsh: [_OutputTypeSeries, 'p2wsh_*'],
  p2tr: [_OutputTypeSeries, 'p2tr_*'],
  p2a: [_OutputTypeSeries, 'p2a_*'],
  emptyScript: [_OutputTypeSeries, 'empty_script_*'],
  unknownScript: [_OutputTypeSeries, 'unknown_script_*'],
  opReturn: [_OutputTypeSeries, 'op_return_*'],
});

/**
 * @typedef {{
 *   p2pk: OutputTypeSeries,
 *   p2pkh: OutputTypeSeries,
 *   p2ms: OutputTypeSeries,
 *   p2sh: OutputTypeSeries,
 *   p2wpkh: OutputTypeSeries,
 *   p2wsh: OutputTypeSeries,
 *   p2tr: OutputTypeSeries,
 *   p2a: OutputTypeSeries,
 *   emptyScript: OutputTypeSeries,
 *   unknownScript: OutputTypeSeries,
 * }} SpendableType
 */
const _SpendableType = _s({
  p2pk: [_OutputTypeSeries, 'p2pk_*'],
  p2pkh: [_OutputTypeSeries, 'p2pkh_*'],
  p2ms: [_OutputTypeSeries, 'p2ms_*'],
  p2sh: [_OutputTypeSeries, 'p2sh_*'],
  p2wpkh: [_OutputTypeSeries, 'p2wpkh_*'],
  p2wsh: [_OutputTypeSeries, 'p2wsh_*'],
  p2tr: [_OutputTypeSeries, 'p2tr_*'],
  p2a: [_OutputTypeSeries, 'p2a_*'],
  emptyScript: [_OutputTypeSeries, 'empty_script_*'],
  unknownScript: [_OutputTypeSeries, 'unknown_script_*'],
});

/**
 * @typedef {{
 *   block: SeriesPattern18<?Percent>,
 *   avg: Windows<?Percent>,
 *   min: Windows<?Percent>,
 *   max: Windows<?Percent>,
 *   p10: Windows<?Percent>,
 *   p25: Windows<?Percent>,
 *   median: Windows<?Percent>,
 *   p75: Windows<?Percent>,
 *   p90: Windows<?Percent>,
 * }} Fullness
 */
const _Fullness = _s({
  block: [_i18, '*'],
  avg: [_Windows, '*_avg'],
  min: [_Windows, '*_min'],
  max: [_Windows, '*_max'],
  p10: [_Windows, '*_p10'],
  p25: [_Windows, '*_p25'],
  median: [_Windows, '*_median'],
  p75: [_Windows, '*_p75'],
  p90: [_Windows, '*_p90'],
});

/**
 * @template A, B
 * @typedef {{
 *   block: SeriesPattern18<A>,
 *   cumulative: SeriesPattern1<A>,
 *   sum: Windows<A>,
 *   avg: Windows<B>,
 *   min: Windows<A>,
 *   max: Windows<A>,
 *   p10: Windows<A>,
 *   p25: Windows<A>,
 *   median: Windows<A>,
 *   p75: Windows<A>,
 *   p90: Windows<A>,
 * }} FlowStats
 */
const _FlowStats = _s({
  block: [_i18, '*'],
  cumulative: [_i1, '*_cumulative'],
  sum: [_Windows, '*_sum'],
  avg: [_Windows, '*_avg'],
  min: [_Windows, '*_min'],
  max: [_Windows, '*_max'],
  p10: [_Windows, '*_p10'],
  p25: [_Windows, '*_p25'],
  median: [_Windows, '*_median'],
  p75: [_Windows, '*_p75'],
  p90: [_Windows, '*_p90'],
});

/**
 * @typedef {{
 *   spendingTxinIndex: SeriesPattern23<TxInIndex>,
 *   count: FlowStats<Count, ?CountFract>,
 *   perSecond: Windows<?PerSecond>,
 *   spendableCount: Flow<Count>,
 *   types: ByType,
 *   opReturnValue: ValueFlow,
 * }} Outputs
 */
const _Outputs = _s({
  spendingTxinIndex: [_i23, 'spending_txin_index'],
  count: [_FlowStats, '*_count'],
  perSecond: [_Windows, 'outputs_per_second'],
  spendableCount: [_Flow, 'spendable_*_count'],
  types: [_ByType, '*'],
  opReturnValue: [_ValueFlow, 'op_return_value'],
});

/**
 * @typedef {{
 *   count: FlowStats<Count, ?CountFract>,
 *   perSecond: Windows<?PerSecond>,
 *   types: SpendableType,
 * }} Inputs
 */
const _Inputs = _s({
  count: [_FlowStats, '*_count'],
  perSecond: [_Windows, 'inputs_per_second'],
  types: [_SpendableType, '*'],
});

/**
 * @template A, B
 * @typedef {{
 *   block: SeriesPattern18<A>,
 *   avg: Windows<B>,
 * }} Averaged
 */
const _Averaged = _s({
  block: [_i18, '*'],
  avg: [_Windows, '*_avg'],
});

/**
 * @template A
 * @typedef {{
 *   count: A,
 *   share: Share,
 * }} EventCount
 */
/** @type {_Make} */
const _EventCount = (c, b, f0) => _n(c, b, {
  count: [f0, '*_count'],
  share: [_Share, '*_share'],
});

/**
 * @typedef {{
 *   outputs: EventCount<Flow<Count>>,
 *   inputs: EventCount<Flow<Count>>,
 *   active: EventCount<Averaged<Count, ?CountFract>>,
 * }} ReuseEvents
 */
const _ReuseEvents = _s({
  outputs: [(c, b) => _EventCount(c, b, _Flow), '*_output_to_respent_address'],
  inputs: [(c, b) => _EventCount(c, b, _Flow), '*_input_from_respent_address'],
  active: [(c, b) => _EventCount(c, b, _Averaged), '*_active_respent_address'],
});

/**
 * @typedef {{
 *   count: RespentCount,
 *   supply: PredicateSupply,
 *   events: ReuseEvents,
 * }} Respent
 */
const _Respent = _s({
  count: [_RespentCount, '*'],
  supply: [_PredicateSupply, '*_respent_address_supply'],
  events: [_ReuseEvents, '*'],
});

/**
 * @typedef {{
 *   outputs: EventCount<Flow<Count>>,
 *   inputs: EventCount<Flow<Count>>,
 *   active: EventCount<Averaged<Count, ?CountFract>>,
 * }} AddressesReuseEvents
 */
const _AddressesReuseEvents = _s({
  outputs: [(c, b) => _EventCount(c, b, _Flow), '*_output_to_reused_address'],
  inputs: [(c, b) => _EventCount(c, b, _Flow), '*_input_from_reused_address'],
  active: [(c, b) => _EventCount(c, b, _Averaged), '*_active_reused_address'],
});

/**
 * @typedef {{
 *   count: ReusedCount,
 *   supply: PredicateSupply,
 *   events: AddressesReuseEvents,
 * }} Reused
 */
const _Reused = _s({
  count: [_ReusedCount, '*'],
  supply: [_PredicateSupply, '*_reused_address_supply'],
  events: [_AddressesReuseEvents, '*'],
});

/**
 * @typedef {{
 *   reactivated: Averaged<Count, ?CountFract>,
 *   sending: Averaged<Count, ?CountFract>,
 *   receiving: Averaged<Count, ?CountFract>,
 *   bidirectional: Averaged<Count, ?CountFract>,
 *   active: Averaged<Count, ?CountFract>,
 * }} AddressesActivity
 */
const _AddressesActivity = _s({
  reactivated: [_Averaged, '*_reactivated_address_count'],
  sending: [_Averaged, '*_sending_address_count'],
  receiving: [_Averaged, '*_receiving_address_count'],
  bidirectional: [_Averaged, '*_bidirectional_address_count'],
  active: [_Averaged, '*_active_address_count'],
});

/**
 * @typedef {{
 *   funded: WithDelta<Count, CountSigned>,
 *   empty: SeriesPattern1<Count>,
 *   total: SeriesPattern1<Count>,
 *   new: NewAddressCount,
 *   activity: AddressesActivity,
 *   avgBalance: Value,
 *   reused: Reused,
 *   respent: Respent,
 *   exposed: Exposed,
 * }} AddressSeries
 */
const _AddressSeries = _s({
  funded: [_WithDelta, '*_funded_address_count'],
  empty: [_i1, '*_empty_address_count'],
  total: [_i1, '*_total_address_count'],
  new: [_NewAddressCount, '*_new_address_count'],
  activity: [_AddressesActivity, '*'],
  avgBalance: [_Value, '*_address_balance_avg'],
  reused: [_Reused, '*'],
  respent: [_Respent, '*'],
  exposed: [_Exposed, '*'],
});

/**
 * @typedef {{
 *   p2pk: AddressSeries,
 *   p2pkh: AddressSeries,
 *   p2sh: AddressSeries,
 *   p2wpkh: AddressSeries,
 *   p2wsh: AddressSeries,
 *   p2tr: AddressSeries,
 * }} AddressType
 */
const _AddressType = _s({
  p2pk: [_AddressSeries, '*'],
  p2pkh: [_AddressSeries, 'p2pkh'],
  p2sh: [_AddressSeries, 'p2sh'],
  p2wpkh: [_AddressSeries, 'p2wpkh'],
  p2wsh: [_AddressSeries, 'p2wsh'],
  p2tr: [_AddressSeries, 'p2tr'],
});

/**
 * @typedef {{
 *   funded: WithDelta<Count, CountSigned>,
 *   empty: SeriesPattern1<Count>,
 *   total: SeriesPattern1<Count>,
 *   new: NewAddressCount,
 *   activity: AddressesActivity,
 *   avgBalance: Value,
 *   reused: Reused,
 *   respent: Respent,
 *   exposed: Exposed,
 *   supply: CohortSupply,
 *   capital: CapitalViews,
 *   types: AddressType,
 *   balances: AmountRange<ZeroBalance, Balance>,
 * }} Addresses
 */
const _Addresses = _s({
  funded: [_WithDelta, 'funded_*_count'],
  empty: [_i1, 'empty_*_count'],
  total: [_i1, 'total_*_count'],
  new: [_NewAddressCount, 'new_*_count'],
  activity: [_AddressesActivity, ''],
  avgBalance: [_Value, '*_balance_avg'],
  reused: [_Reused, ''],
  respent: [_Respent, ''],
  exposed: [_Exposed, ''],
  supply: [_CohortSupply, '*_supply'],
  capital: [_CapitalViews, '*'],
  types: [_AddressType, 'p2pk'],
  balances: [(c, b) => _AmountRange(c, b, _ZeroBalance, _Balance), 'balance'],
});

/**
 * @typedef {{
 *   cumulative: SeriesPattern1<Count>,
 *   sum: Windows<Count>,
 * }} CumulativeFlow
 */
const _CumulativeFlow = _s({
  cumulative: [_i1, '*_cumulative'],
  sum: [_Windows, '*_sum'],
});

/**
 * @typedef {{
 *   blockCount: CumulativeFlow,
 *   blockShare: Share,
 *   coinbase: Coinbase,
 *   fees: PoolFees,
 *   hashrate: Windows<?Hashrate>,
 * }} Pool
 */
const _Pool = _s({
  blockCount: [_CumulativeFlow, '*_block_count'],
  blockShare: [_Share, '*_block_share'],
  coinbase: [_Coinbase, '*_coinbase'],
  fees: [_PoolFees, '*_fees_avg'],
  hashrate: [_Windows, '*_hashrate'],
});

/**
 * @typedef {{
 *   blockPool: SeriesPattern18<PoolSlug>,
 *   unknown: Pool,
 *   blockfills: Pool,
 *   ultimuspool: Pool,
 *   terrapool: Pool,
 *   luxor: Pool,
 *   _1thash: Pool,
 *   btccom: Pool,
 *   bitfarms: Pool,
 *   huobipool: Pool,
 *   wayicn: Pool,
 *   canoepool: Pool,
 *   btctop: Pool,
 *   bitcoincom: Pool,
 *   _175btc: Pool,
 *   gbminers: Pool,
 *   axbt: Pool,
 *   asicminer: Pool,
 *   bitminter: Pool,
 *   bitcoinrussia: Pool,
 *   btcserv: Pool,
 *   simplecoinus: Pool,
 *   btcguild: Pool,
 *   eligius: Pool,
 *   ozcoin: Pool,
 *   eclipsemc: Pool,
 *   maxbtc: Pool,
 *   triplemining: Pool,
 *   coinlab: Pool,
 *   _50btc: Pool,
 *   ghashio: Pool,
 *   stminingcorp: Pool,
 *   bitparking: Pool,
 *   mmpool: Pool,
 *   polmine: Pool,
 *   kncminer: Pool,
 *   bitalo: Pool,
 *   f2pool: Pool,
 *   hhtt: Pool,
 *   megabigpower: Pool,
 *   mtred: Pool,
 *   nmcbit: Pool,
 *   yourbtcnet: Pool,
 *   givemecoins: Pool,
 *   braiinspool: Pool,
 *   antpool: Pool,
 *   multicoinco: Pool,
 *   bcpoolio: Pool,
 *   cointerra: Pool,
 *   kanopool: Pool,
 *   solock: Pool,
 *   ckpool: Pool,
 *   nicehash: Pool,
 *   bitclub: Pool,
 *   bitcoinaffiliatenetwork: Pool,
 *   btcc: Pool,
 *   bwpool: Pool,
 *   exxbw: Pool,
 *   bitsolo: Pool,
 *   bitfury: Pool,
 *   _21inc: Pool,
 *   digitalbtc: Pool,
 *   _8baochi: Pool,
 *   mybtccoinpool: Pool,
 *   tbdice: Pool,
 *   hashpool: Pool,
 *   nexious: Pool,
 *   bravomining: Pool,
 *   hotpool: Pool,
 *   okexpool: Pool,
 *   bcmonster: Pool,
 *   _1hash: Pool,
 *   bixin: Pool,
 *   tatmaspool: Pool,
 *   viabtc: Pool,
 *   connectbtc: Pool,
 *   batpool: Pool,
 *   waterhole: Pool,
 *   dcexploration: Pool,
 *   dcex: Pool,
 *   btpool: Pool,
 *   _58coin: Pool,
 *   bitcoinindia: Pool,
 *   shawnp0wers: Pool,
 *   phashio: Pool,
 *   rigpool: Pool,
 *   haozhuzhu: Pool,
 *   _7pool: Pool,
 *   miningkings: Pool,
 *   hashbx: Pool,
 *   dpool: Pool,
 *   rawpool: Pool,
 *   haominer: Pool,
 *   helix: Pool,
 *   bitcoinukraine: Pool,
 *   poolin: Pool,
 *   secretsuperstar: Pool,
 *   tigerpoolnet: Pool,
 *   sigmapoolcom: Pool,
 *   okpooltop: Pool,
 *   hummerpool: Pool,
 *   tangpool: Pool,
 *   bytepool: Pool,
 *   spiderpool: Pool,
 *   novablock: Pool,
 *   miningcity: Pool,
 *   binancepool: Pool,
 *   minerium: Pool,
 *   lubiancom: Pool,
 *   okkong: Pool,
 *   aaopool: Pool,
 *   emcdpool: Pool,
 *   foundryusa: Pool,
 *   sbicrypto: Pool,
 *   arkpool: Pool,
 *   purebtccom: Pool,
 *   marapool: Pool,
 *   kucoinpool: Pool,
 *   entrustcharitypool: Pool,
 *   okminer: Pool,
 *   titan: Pool,
 *   pegapool: Pool,
 *   btcnuggets: Pool,
 *   cloudhashing: Pool,
 *   digitalxmintsy: Pool,
 *   telco214: Pool,
 *   btcpoolparty: Pool,
 *   multipool: Pool,
 *   transactioncoinmining: Pool,
 *   btcdig: Pool,
 *   trickysbtcpool: Pool,
 *   btcmp: Pool,
 *   eobot: Pool,
 *   unomp: Pool,
 *   patels: Pool,
 *   gogreenlight: Pool,
 *   bitcoinindiapool: Pool,
 *   ekanembtc: Pool,
 *   canoe: Pool,
 *   tiger: Pool,
 *   _1m1x: Pool,
 *   zulupool: Pool,
 *   secpool: Pool,
 *   ocean: Pool,
 *   whitepool: Pool,
 *   wiz: Pool,
 *   wk057: Pool,
 *   futurebitapollosolo: Pool,
 *   carbonnegative: Pool,
 *   portlandhodl: Pool,
 *   phoenix: Pool,
 *   neopool: Pool,
 *   maxipool: Pool,
 *   bitfufupool: Pool,
 *   gdpool: Pool,
 *   miningdutch: Pool,
 *   publicpool: Pool,
 *   miningsquared: Pool,
 *   innopolistech: Pool,
 *   btclab: Pool,
 *   parasite: Pool,
 *   redrockpool: Pool,
 *   est3lar: Pool,
 *   braiinssolo: Pool,
 *   solopoolcom: Pool,
 *   noderunners: Pool,
 *   dmnd: Pool,
 * }} Pools
 */
const _Pools = _s({
  blockPool: [_i18, 'block_*'],
  unknown: [_Pool, '*_unknown'],
  blockfills: [_Pool, '*_blockfills'],
  ultimuspool: [_Pool, '*_ultimuspool'],
  terrapool: [_Pool, '*_terrapool'],
  luxor: [_Pool, '*_luxor'],
  _1thash: [_Pool, '*_1thash'],
  btccom: [_Pool, '*_btccom'],
  bitfarms: [_Pool, '*_bitfarms'],
  huobipool: [_Pool, '*_huobipool'],
  wayicn: [_Pool, '*_wayicn'],
  canoepool: [_Pool, '*_canoepool'],
  btctop: [_Pool, '*_btctop'],
  bitcoincom: [_Pool, '*_bitcoincom'],
  _175btc: [_Pool, '*_175btc'],
  gbminers: [_Pool, '*_gbminers'],
  axbt: [_Pool, '*_axbt'],
  asicminer: [_Pool, '*_asicminer'],
  bitminter: [_Pool, '*_bitminter'],
  bitcoinrussia: [_Pool, '*_bitcoinrussia'],
  btcserv: [_Pool, '*_btcserv'],
  simplecoinus: [_Pool, '*_simplecoinus'],
  btcguild: [_Pool, '*_btcguild'],
  eligius: [_Pool, '*_eligius'],
  ozcoin: [_Pool, '*_ozcoin'],
  eclipsemc: [_Pool, '*_eclipsemc'],
  maxbtc: [_Pool, '*_maxbtc'],
  triplemining: [_Pool, '*_triplemining'],
  coinlab: [_Pool, '*_coinlab'],
  _50btc: [_Pool, '*_50btc'],
  ghashio: [_Pool, '*_ghashio'],
  stminingcorp: [_Pool, '*_stminingcorp'],
  bitparking: [_Pool, '*_bitparking'],
  mmpool: [_Pool, '*_mmpool'],
  polmine: [_Pool, '*_polmine'],
  kncminer: [_Pool, '*_kncminer'],
  bitalo: [_Pool, '*_bitalo'],
  f2pool: [_Pool, '*_f2pool'],
  hhtt: [_Pool, '*_hhtt'],
  megabigpower: [_Pool, '*_megabigpower'],
  mtred: [_Pool, '*_mtred'],
  nmcbit: [_Pool, '*_nmcbit'],
  yourbtcnet: [_Pool, '*_yourbtcnet'],
  givemecoins: [_Pool, '*_givemecoins'],
  braiinspool: [_Pool, '*_braiinspool'],
  antpool: [_Pool, '*_antpool'],
  multicoinco: [_Pool, '*_multicoinco'],
  bcpoolio: [_Pool, '*_bcpoolio'],
  cointerra: [_Pool, '*_cointerra'],
  kanopool: [_Pool, '*_kanopool'],
  solock: [_Pool, '*_solock'],
  ckpool: [_Pool, '*_ckpool'],
  nicehash: [_Pool, '*_nicehash'],
  bitclub: [_Pool, '*_bitclub'],
  bitcoinaffiliatenetwork: [_Pool, '*_bitcoinaffiliatenetwork'],
  btcc: [_Pool, '*_btcc'],
  bwpool: [_Pool, '*_bwpool'],
  exxbw: [_Pool, '*_exxbw'],
  bitsolo: [_Pool, '*_bitsolo'],
  bitfury: [_Pool, '*_bitfury'],
  _21inc: [_Pool, '*_21inc'],
  digitalbtc: [_Pool, '*_digitalbtc'],
  _8baochi: [_Pool, '*_8baochi'],
  mybtccoinpool: [_Pool, '*_mybtccoinpool'],
  tbdice: [_Pool, '*_tbdice'],
  hashpool: [_Pool, '*_hashpool'],
  nexious: [_Pool, '*_nexious'],
  bravomining: [_Pool, '*_bravomining'],
  hotpool: [_Pool, '*_hotpool'],
  okexpool: [_Pool, '*_okexpool'],
  bcmonster: [_Pool, '*_bcmonster'],
  _1hash: [_Pool, '*_1hash'],
  bixin: [_Pool, '*_bixin'],
  tatmaspool: [_Pool, '*_tatmaspool'],
  viabtc: [_Pool, '*_viabtc'],
  connectbtc: [_Pool, '*_connectbtc'],
  batpool: [_Pool, '*_batpool'],
  waterhole: [_Pool, '*_waterhole'],
  dcexploration: [_Pool, '*_dcexploration'],
  dcex: [_Pool, '*_dcex'],
  btpool: [_Pool, '*_btpool'],
  _58coin: [_Pool, '*_58coin'],
  bitcoinindia: [_Pool, '*_bitcoinindia'],
  shawnp0wers: [_Pool, '*_shawnp0wers'],
  phashio: [_Pool, '*_phashio'],
  rigpool: [_Pool, '*_rigpool'],
  haozhuzhu: [_Pool, '*_haozhuzhu'],
  _7pool: [_Pool, '*_7pool'],
  miningkings: [_Pool, '*_miningkings'],
  hashbx: [_Pool, '*_hashbx'],
  dpool: [_Pool, '*_dpool'],
  rawpool: [_Pool, '*_rawpool'],
  haominer: [_Pool, '*_haominer'],
  helix: [_Pool, '*_helix'],
  bitcoinukraine: [_Pool, '*_bitcoinukraine'],
  poolin: [_Pool, '*_poolin'],
  secretsuperstar: [_Pool, '*_secretsuperstar'],
  tigerpoolnet: [_Pool, '*_tigerpoolnet'],
  sigmapoolcom: [_Pool, '*_sigmapoolcom'],
  okpooltop: [_Pool, '*_okpooltop'],
  hummerpool: [_Pool, '*_hummerpool'],
  tangpool: [_Pool, '*_tangpool'],
  bytepool: [_Pool, '*_bytepool'],
  spiderpool: [_Pool, '*_spiderpool'],
  novablock: [_Pool, '*_novablock'],
  miningcity: [_Pool, '*_miningcity'],
  binancepool: [_Pool, '*_binancepool'],
  minerium: [_Pool, '*_minerium'],
  lubiancom: [_Pool, '*_lubiancom'],
  okkong: [_Pool, '*_okkong'],
  aaopool: [_Pool, '*_aaopool'],
  emcdpool: [_Pool, '*_emcdpool'],
  foundryusa: [_Pool, '*_foundryusa'],
  sbicrypto: [_Pool, '*_sbicrypto'],
  arkpool: [_Pool, '*_arkpool'],
  purebtccom: [_Pool, '*_purebtccom'],
  marapool: [_Pool, '*_marapool'],
  kucoinpool: [_Pool, '*_kucoinpool'],
  entrustcharitypool: [_Pool, '*_entrustcharitypool'],
  okminer: [_Pool, '*_okminer'],
  titan: [_Pool, '*_titan'],
  pegapool: [_Pool, '*_pegapool'],
  btcnuggets: [_Pool, '*_btcnuggets'],
  cloudhashing: [_Pool, '*_cloudhashing'],
  digitalxmintsy: [_Pool, '*_digitalxmintsy'],
  telco214: [_Pool, '*_telco214'],
  btcpoolparty: [_Pool, '*_btcpoolparty'],
  multipool: [_Pool, '*_multipool'],
  transactioncoinmining: [_Pool, '*_transactioncoinmining'],
  btcdig: [_Pool, '*_btcdig'],
  trickysbtcpool: [_Pool, '*_trickysbtcpool'],
  btcmp: [_Pool, '*_btcmp'],
  eobot: [_Pool, '*_eobot'],
  unomp: [_Pool, '*_unomp'],
  patels: [_Pool, '*_patels'],
  gogreenlight: [_Pool, '*_gogreenlight'],
  bitcoinindiapool: [_Pool, '*_bitcoinindiapool'],
  ekanembtc: [_Pool, '*_ekanembtc'],
  canoe: [_Pool, '*_canoe'],
  tiger: [_Pool, '*_tiger'],
  _1m1x: [_Pool, '*_1m1x'],
  zulupool: [_Pool, '*_zulupool'],
  secpool: [_Pool, '*_secpool'],
  ocean: [_Pool, '*_ocean'],
  whitepool: [_Pool, '*_whitepool'],
  wiz: [_Pool, '*_wiz'],
  wk057: [_Pool, '*_wk057'],
  futurebitapollosolo: [_Pool, '*_futurebitapollosolo'],
  carbonnegative: [_Pool, '*_carbonnegative'],
  portlandhodl: [_Pool, '*_portlandhodl'],
  phoenix: [_Pool, '*_phoenix'],
  neopool: [_Pool, '*_neopool'],
  maxipool: [_Pool, '*_maxipool'],
  bitfufupool: [_Pool, '*_bitfufupool'],
  gdpool: [_Pool, '*_gdpool'],
  miningdutch: [_Pool, '*_miningdutch'],
  publicpool: [_Pool, '*_publicpool'],
  miningsquared: [_Pool, '*_miningsquared'],
  innopolistech: [_Pool, '*_innopolistech'],
  btclab: [_Pool, '*_btclab'],
  parasite: [_Pool, '*_parasite'],
  redrockpool: [_Pool, '*_redrockpool'],
  est3lar: [_Pool, '*_est3lar'],
  braiinssolo: [_Pool, '*_braiinssolo'],
  solopoolcom: [_Pool, '*_solopoolcom'],
  noderunners: [_Pool, '*_noderunners'],
  dmnd: [_Pool, '*_dmnd'],
});

/**
 * @typedef {{
 *   count: CumulativeFlow,
 *   interval: Averaged<Seconds, ?SecondsFract>,
 *   size: FlowStats<Bytes, ?BytesFract>,
 *   weight: FlowStats<Weight, ?WeightFract>,
 *   fullness: Fullness,
 *   difficulty: BlocksDifficulty,
 *   halving: Halving,
 * }} Blocks
 */
const _Blocks = _s({
  count: [_CumulativeFlow, '*_count'],
  interval: [_Averaged, '*_interval'],
  size: [_FlowStats, '*_size'],
  weight: [_FlowStats, '*_weight'],
  fullness: [_Fullness, '*_fullness'],
  difficulty: [_BlocksDifficulty, 'difficulty'],
  halving: [_Halving, 'halving'],
});

/**
 * @typedef {{
 *   open: SeriesPattern2<?Dollars>,
 *   high: SeriesPattern2<?Dollars>,
 *   low: SeriesPattern2<?Dollars>,
 *   close: SeriesPattern2<?Dollars>,
 *   ohlc: SeriesPattern2<OHLCDollars>,
 *   spot: SeriesPattern1<?Dollars>,
 *   satsPerDollar: SeriesPattern1<Sats>,
 * }} Price
 */
const _Price = _s({
  open: [_i2, '*_open'],
  high: [_i2, '*_high'],
  low: [_i2, '*_low'],
  close: [_i2, '*_close'],
  ohlc: [_i2, '*_ohlc'],
  spot: [_i1, '*'],
  satsPerDollar: [_i1, 'sats_per_dollar'],
});

/**
 * @typedef {{
 *   monotonic: SeriesPattern18<Timestamp>,
 *   periodStart: SeriesPattern2<Timestamp>,
 * }} Timestamps
 */
const _Timestamps = _s({
  monotonic: [_i18, '*_monotonic'],
  periodStart: [_i2, '*'],
});

/**
 * @typedef {{
 *   identity: SeriesPattern22<TxOutIndex>,
 * }} TxoutIndex
 */
const _TxoutIndex = _s({
  identity: [_i22, '*'],
});

/**
 * @typedef {{
 *   identity: SeriesPattern20<TxInIndex>,
 * }} TxinIndex
 */
const _TxinIndex = _s({
  identity: [_i20, '*'],
});

/**
 * @typedef {{
 *   identity: SeriesPattern19<TxIndex>,
 *   inputCount: SeriesPattern19<Count>,
 *   outputCount: SeriesPattern19<Count>,
 * }} MappingsTxIndex
 */
const _MappingsTxIndex = _s({
  identity: [_i19, '*_index'],
  inputCount: [_i19, '*_input_count'],
  outputCount: [_i19, '*_output_count'],
});

/**
 * @typedef {{
 *   date: SeriesPattern15<Date>,
 *   firstHeight: SeriesPattern15<Height>,
 * }} MappingsYear10
 */
const _MappingsYear10 = _s({
  date: [_i15, '*'],
  firstHeight: [_i15, 'first_height'],
});

/**
 * @typedef {{
 *   date: SeriesPattern14<Date>,
 *   firstHeight: SeriesPattern14<Height>,
 * }} MappingsYear1
 */
const _MappingsYear1 = _s({
  date: [_i14, '*'],
  firstHeight: [_i14, 'first_height'],
});

/**
 * @typedef {{
 *   date: SeriesPattern13<Date>,
 *   firstHeight: SeriesPattern13<Height>,
 * }} MappingsMonth6
 */
const _MappingsMonth6 = _s({
  date: [_i13, '*'],
  firstHeight: [_i13, 'first_height'],
});

/**
 * @typedef {{
 *   date: SeriesPattern12<Date>,
 *   firstHeight: SeriesPattern12<Height>,
 * }} MappingsMonth3
 */
const _MappingsMonth3 = _s({
  date: [_i12, '*'],
  firstHeight: [_i12, 'first_height'],
});

/**
 * @typedef {{
 *   date: SeriesPattern11<Date>,
 *   firstHeight: SeriesPattern11<Height>,
 * }} MappingsMonth1
 */
const _MappingsMonth1 = _s({
  date: [_i11, '*'],
  firstHeight: [_i11, 'first_height'],
});

/**
 * @typedef {{
 *   date: SeriesPattern10<Date>,
 *   firstHeight: SeriesPattern10<Height>,
 * }} MappingsWeek1
 */
const _MappingsWeek1 = _s({
  date: [_i10, '*'],
  firstHeight: [_i10, 'first_height'],
});

/**
 * @typedef {{
 *   date: SeriesPattern9<Date>,
 *   firstHeight: SeriesPattern9<Height>,
 * }} MappingsDay3
 */
const _MappingsDay3 = _s({
  date: [_i9, '*'],
  firstHeight: [_i9, 'first_height'],
});

/**
 * @typedef {{
 *   date: SeriesPattern8<Date>,
 *   firstHeight: SeriesPattern8<Height>,
 * }} MappingsDay1
 */
const _MappingsDay1 = _s({
  date: [_i8, '*'],
  firstHeight: [_i8, 'first_height'],
});

/**
 * @typedef {{
 *   firstHeight: SeriesPattern7<Height>,
 * }} MappingsHour12
 */
const _MappingsHour12 = _s({
  firstHeight: [_i7, '*'],
});

/**
 * @typedef {{
 *   firstHeight: SeriesPattern6<Height>,
 * }} MappingsHour4
 */
const _MappingsHour4 = _s({
  firstHeight: [_i6, '*'],
});

/**
 * @typedef {{
 *   firstHeight: SeriesPattern5<Height>,
 * }} MappingsHour1
 */
const _MappingsHour1 = _s({
  firstHeight: [_i5, '*'],
});

/**
 * @typedef {{
 *   firstHeight: SeriesPattern4<Height>,
 * }} MappingsMinute30
 */
const _MappingsMinute30 = _s({
  firstHeight: [_i4, '*'],
});

/**
 * @typedef {{
 *   firstHeight: SeriesPattern3<Height>,
 * }} MappingsMinute10
 */
const _MappingsMinute10 = _s({
  firstHeight: [_i3, '*'],
});

/**
 * @typedef {{
 *   firstHeight: SeriesPattern16<Height>,
 * }} MappingsHalvingEpoch
 */
const _MappingsHalvingEpoch = _s({
  firstHeight: [_i16, '*'],
});

/**
 * @typedef {{
 *   firstHeight: SeriesPattern17<Height>,
 * }} MappingsDifficultyEpoch
 */
const _MappingsDifficultyEpoch = _s({
  firstHeight: [_i17, '*'],
});

/**
 * @typedef {{
 *   minute10: SeriesPattern18<Minute10>,
 *   minute30: SeriesPattern18<Minute30>,
 *   hour1: SeriesPattern18<Hour1>,
 *   hour4: SeriesPattern18<Hour4>,
 *   hour12: SeriesPattern18<Hour12>,
 *   day1: SeriesPattern18<Day1>,
 *   day3: SeriesPattern18<Day3>,
 *   difficultyEpoch: SeriesPattern18<DifficultyEpoch>,
 *   halvingEpoch: SeriesPattern18<HalvingEpoch>,
 *   week1: SeriesPattern18<Week1>,
 *   month1: SeriesPattern18<Month1>,
 *   month3: SeriesPattern18<Month3>,
 *   month6: SeriesPattern18<Month6>,
 *   year1: SeriesPattern18<Year1>,
 *   year10: SeriesPattern18<Year10>,
 * }} MappingsHeight
 */
const _MappingsHeight = _s({
  minute10: [_i18, '*_minute10'],
  minute30: [_i18, '*_minute30'],
  hour1: [_i18, '*_hour1'],
  hour4: [_i18, '*_hour4'],
  hour12: [_i18, '*_hour12'],
  day1: [_i18, '*_day1'],
  day3: [_i18, '*_day3'],
  difficultyEpoch: [_i18, 'difficulty_epoch'],
  halvingEpoch: [_i18, 'halving_epoch'],
  week1: [_i18, '*_week1'],
  month1: [_i18, '*_month1'],
  month3: [_i18, '*_month3'],
  month6: [_i18, '*_month6'],
  year1: [_i18, '*_year1'],
  year10: [_i18, '*_year10'],
});

/**
 * @typedef {{
 *   identity: SeriesPattern25<OpReturnOutputIndex>,
 * }} MappingsOpReturn
 */
const _MappingsOpReturn = _s({
  identity: [_i25, '*'],
});

/**
 * @typedef {{
 *   identity: SeriesPattern35<UnknownScriptOutputIndex>,
 * }} MappingsUnknownScript
 */
const _MappingsUnknownScript = _s({
  identity: [_i35, '*'],
});

/**
 * @typedef {{
 *   identity: SeriesPattern24<EmptyScriptOutputIndex>,
 * }} EmptyScript
 */
const _EmptyScript = _s({
  identity: [_i24, '*'],
});

/**
 * @typedef {{
 *   identity: SeriesPattern27<P2MSOutputIndex>,
 * }} MappingsP2ms
 */
const _MappingsP2ms = _s({
  identity: [_i27, '*'],
});

/**
 * @typedef {{
 *   p2ms: MappingsP2ms,
 *   emptyScript: EmptyScript,
 *   unknownScript: MappingsUnknownScript,
 *   opReturn: MappingsOpReturn,
 * }} Script
 */
const _Script = _s({
  p2ms: [_MappingsP2ms, 'p2ms_*'],
  emptyScript: [_EmptyScript, 'empty_script_*'],
  unknownScript: [_MappingsUnknownScript, 'unknown_script_*'],
  opReturn: [_MappingsOpReturn, 'op_return_*'],
});

/**
 * @typedef {{
 *   identity: SeriesPattern26<P2AAddressIndex>,
 *   address: SeriesPattern26<Address>,
 * }} MappingsP2a
 */
const _MappingsP2a = _s({
  identity: [_i26, '*_index'],
  address: [_i26, '*'],
});

/**
 * @typedef {{
 *   identity: SeriesPattern32<P2TRAddressIndex>,
 *   address: SeriesPattern32<Address>,
 * }} MappingsP2tr
 */
const _MappingsP2tr = _s({
  identity: [_i32, '*_index'],
  address: [_i32, '*'],
});

/**
 * @typedef {{
 *   identity: SeriesPattern34<P2WSHAddressIndex>,
 *   address: SeriesPattern34<Address>,
 * }} MappingsP2wsh
 */
const _MappingsP2wsh = _s({
  identity: [_i34, '*_index'],
  address: [_i34, '*'],
});

/**
 * @typedef {{
 *   identity: SeriesPattern33<P2WPKHAddressIndex>,
 *   address: SeriesPattern33<Address>,
 * }} MappingsP2wpkh
 */
const _MappingsP2wpkh = _s({
  identity: [_i33, '*_index'],
  address: [_i33, '*'],
});

/**
 * @typedef {{
 *   identity: SeriesPattern31<P2SHAddressIndex>,
 *   address: SeriesPattern31<Address>,
 * }} MappingsP2sh
 */
const _MappingsP2sh = _s({
  identity: [_i31, '*_index'],
  address: [_i31, '*'],
});

/**
 * @typedef {{
 *   identity: SeriesPattern30<P2PKHAddressIndex>,
 *   address: SeriesPattern30<Address>,
 * }} MappingsP2pkh
 */
const _MappingsP2pkh = _s({
  identity: [_i30, '*_index'],
  address: [_i30, '*'],
});

/**
 * @typedef {{
 *   identity: SeriesPattern28<P2PK33AddressIndex>,
 *   address: SeriesPattern28<Address>,
 * }} MappingsP2pk33
 */
const _MappingsP2pk33 = _s({
  identity: [_i28, '*_index'],
  address: [_i28, '*'],
});

/**
 * @typedef {{
 *   identity: SeriesPattern29<P2PK65AddressIndex>,
 *   address: SeriesPattern29<Address>,
 * }} MappingsP2pk65
 */
const _MappingsP2pk65 = _s({
  identity: [_i29, '*_index'],
  address: [_i29, '*'],
});

/**
 * @typedef {{
 *   firstIndex: SeriesPattern18<OpReturnOutputIndex>,
 *   txIndex: SeriesPattern25<TxIndex>,
 *   protocol: SeriesPattern25<OpReturnKind>,
 *   postOpReturnBytes: SeriesPattern25<Bytes>,
 * }} IndexerOpReturn
 */
const _IndexerOpReturn = _s({
  firstIndex: [_i18, 'first_*_output_index'],
  txIndex: [_i25, '*_output_tx_index'],
  protocol: [_i25, '*_protocol'],
  postOpReturnBytes: [_i25, 'post_*_bytes'],
});

/**
 * @typedef {{
 *   firstIndex: SeriesPattern18<UnknownScriptOutputIndex>,
 *   txIndex: SeriesPattern35<TxIndex>,
 *   legacySigopCost: SeriesPattern35<SigOps>,
 * }} IndexerUnknownScript
 */
const _IndexerUnknownScript = _s({
  firstIndex: [_i18, 'first_*_output_index'],
  txIndex: [_i35, '*_output_tx_index'],
  legacySigopCost: [_i35, '*_legacy_sigop_cost'],
});

/**
 * @typedef {{
 *   firstIndex: SeriesPattern18<EmptyScriptOutputIndex>,
 *   txIndex: SeriesPattern24<TxIndex>,
 * }} ScriptType
 */
const _ScriptType = _s({
  firstIndex: [_i18, 'first_*_index'],
  txIndex: [_i24, '*_tx_index'],
});

/**
 * @typedef {{
 *   firstIndex: SeriesPattern18<P2MSOutputIndex>,
 *   txIndex: SeriesPattern27<TxIndex>,
 *   legacySigopCost: SeriesPattern27<SigOps>,
 * }} IndexerP2ms
 */
const _IndexerP2ms = _s({
  firstIndex: [_i18, 'first_*_output_index'],
  txIndex: [_i27, '*_output_tx_index'],
  legacySigopCost: [_i27, '*_legacy_sigop_cost'],
});

/**
 * @typedef {{
 *   p2ms: IndexerP2ms,
 *   emptyScript: ScriptType,
 *   unknownScript: IndexerUnknownScript,
 * }} Scripts
 */
const _Scripts = _s({
  p2ms: [_IndexerP2ms, 'p2ms'],
  emptyScript: [_ScriptType, 'empty_*_output'],
  unknownScript: [_IndexerUnknownScript, 'unknown_*'],
});

/**
 * @typedef {{
 *   firstIndex: SeriesPattern18<P2AAddressIndex>,
 *   bytes: SeriesPattern26<P2ABytes>,
 * }} IndexerP2a
 */
const _IndexerP2a = _s({
  firstIndex: [_i18, 'first_*_address_index'],
  bytes: [_i26, '*_bytes'],
});

/**
 * @typedef {{
 *   firstIndex: SeriesPattern18<P2TRAddressIndex>,
 *   bytes: SeriesPattern32<P2TRBytes>,
 * }} IndexerP2tr
 */
const _IndexerP2tr = _s({
  firstIndex: [_i18, 'first_*_address_index'],
  bytes: [_i32, '*_bytes'],
});

/**
 * @typedef {{
 *   firstIndex: SeriesPattern18<P2WSHAddressIndex>,
 *   bytes: SeriesPattern34<P2WSHBytes>,
 * }} IndexerP2wsh
 */
const _IndexerP2wsh = _s({
  firstIndex: [_i18, 'first_*_address_index'],
  bytes: [_i34, '*_bytes'],
});

/**
 * @typedef {{
 *   firstIndex: SeriesPattern18<P2WPKHAddressIndex>,
 *   bytes: SeriesPattern33<P2WPKHBytes>,
 * }} IndexerP2wpkh
 */
const _IndexerP2wpkh = _s({
  firstIndex: [_i18, 'first_*_address_index'],
  bytes: [_i33, '*_bytes'],
});

/**
 * @typedef {{
 *   firstIndex: SeriesPattern18<P2SHAddressIndex>,
 *   bytes: SeriesPattern31<P2SHBytes>,
 * }} IndexerP2sh
 */
const _IndexerP2sh = _s({
  firstIndex: [_i18, 'first_*_address_index'],
  bytes: [_i31, '*_bytes'],
});

/**
 * @typedef {{
 *   firstIndex: SeriesPattern18<P2PKHAddressIndex>,
 *   bytes: SeriesPattern30<P2PKHBytes>,
 * }} IndexerP2pkh
 */
const _IndexerP2pkh = _s({
  firstIndex: [_i18, 'first_*_address_index'],
  bytes: [_i30, '*_bytes'],
});

/**
 * @typedef {{
 *   firstIndex: SeriesPattern18<P2PK33AddressIndex>,
 *   bytes: SeriesPattern28<P2PK33Bytes>,
 * }} IndexerP2pk33
 */
const _IndexerP2pk33 = _s({
  firstIndex: [_i18, 'first_*_address_index'],
  bytes: [_i28, '*_bytes'],
});

/**
 * @typedef {{
 *   firstIndex: SeriesPattern18<P2PK65AddressIndex>,
 *   bytes: SeriesPattern29<P2PK65Bytes>,
 * }} IndexerP2pk65
 */
const _IndexerP2pk65 = _s({
  firstIndex: [_i18, 'first_*_address_index'],
  bytes: [_i29, '*_bytes'],
});

/**
 * @template A, B, C, D, E, F, G, H
 * @typedef {{
 *   p2pk65: A,
 *   p2pk33: B,
 *   p2pkh: C,
 *   p2sh: D,
 *   p2wpkh: E,
 *   p2wsh: F,
 *   p2tr: G,
 *   p2a: H,
 * }} Addrs
 */
/** @type {_Make} */
const _Addrs = (c, b, f0, f1, f2, f3, f4, f5, f6, f7) => _n(c, b, {
  p2pk65: [f0, 'p2pk65_*'],
  p2pk33: [f1, 'p2pk33_*'],
  p2pkh: [f2, 'p2pkh_*'],
  p2sh: [f3, 'p2sh_*'],
  p2wpkh: [f4, 'p2wpkh_*'],
  p2wsh: [f5, 'p2wsh_*'],
  p2tr: [f6, 'p2tr_*'],
  p2a: [f7, 'p2a_*'],
});

/**
 * @typedef {{
 *   addresses: Addrs<MappingsP2pk65, MappingsP2pk33, MappingsP2pkh, MappingsP2sh, MappingsP2wpkh, MappingsP2wsh, MappingsP2tr, MappingsP2a>,
 *   scripts: Script,
 *   height: MappingsHeight,
 *   difficultyEpoch: MappingsDifficultyEpoch,
 *   halvingEpoch: MappingsHalvingEpoch,
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
 *   timestamp: Timestamps,
 * }} Mappings
 */
const _Mappings = _s({
  addresses: [(c, b) => _Addrs(c, b, _MappingsP2pk65, _MappingsP2pk33, _MappingsP2pkh, _MappingsP2sh, _MappingsP2wpkh, _MappingsP2wsh, _MappingsP2tr, _MappingsP2a), 'address'],
  scripts: [_Script, 'output_index'],
  height: [_MappingsHeight, 'block'],
  difficultyEpoch: [_MappingsDifficultyEpoch, 'first_height'],
  halvingEpoch: [_MappingsHalvingEpoch, 'first_height'],
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
  timestamp: [_Timestamps, 'timestamp'],
});

/**
 * @typedef {{
 *   firstTxoutIndex: SeriesPattern18<TxOutIndex>,
 *   value: SeriesPattern22<Sats>,
 *   outputType: SeriesPattern22<OutputType>,
 *   typeIndex: SeriesPattern22<TypeIndex>,
 * }} IndexerOutputs
 */
const _IndexerOutputs = _s({
  firstTxoutIndex: [_i18, 'first_txout_index'],
  value: [_i22, '*_value'],
  outputType: [_i22, '*_type'],
  typeIndex: [_i22, 'type_index'],
});

/**
 * @typedef {{
 *   firstTxinIndex: SeriesPattern18<TxInIndex>,
 *   outpoint: SeriesPattern21<OutPoint>,
 *   txoutIndex: SeriesPattern21<TxOutIndex>,
 *   txIndex: SeriesPattern20<TxIndex>,
 *   outputType: SeriesPattern20<OutputType>,
 *   typeIndex: SeriesPattern21<TypeIndex>,
 *   value: SeriesPattern20<Sats>,
 * }} IndexerInputs
 */
const _IndexerInputs = _s({
  firstTxinIndex: [_i18, 'first_txin_*'],
  outpoint: [_i21, 'outpoint'],
  txoutIndex: [_i21, 'spent_txout_*'],
  txIndex: [_i20, 'input_tx_*'],
  outputType: [_i20, 'output_type'],
  typeIndex: [_i21, 'type_*'],
  value: [_i20, 'input_value'],
});

/**
 * @typedef {{
 *   block: SeriesPattern18<Count>,
 * }} CountSeries
 */
const _CountSeries = _s({
  block: [_i18, '*'],
});

/**
 * @template A
 * @typedef {{
 *   v1: A,
 *   v2: A,
 *   v3: A,
 *   other: A,
 * }} VersionCount
 */
/** @type {_Make} */
const _VersionCount = (c, b, f0) => _n(c, b, {
  v1: [f0, 'v1_*'],
  v2: [f0, 'v2_*'],
  v3: [f0, 'v3_*'],
  other: [f0, 'other_version_*'],
});

/**
 * @typedef {{
 *   count: FlowStats<Count, ?CountFract>,
 *   features: Features,
 *   vsize: PerTxDistribution<VSize>,
 *   fees: Fees,
 *   inscription: Inscription,
 *   patterns: Patterns,
 *   nonstandard: TransactionsFlagged,
 *   sigopCost: Flow<SigOps>,
 *   versions: VersionCount<TxCount>,
 *   transferVolume: ValueFlow,
 *   perSecond: Windows<?PerSecond>,
 * }} Transactions
 */
const _Transactions = _s({
  count: [_FlowStats, '*_count'],
  features: [_Features, '*'],
  vsize: [_PerTxDistribution, '*_vsize'],
  fees: [_Fees, '*'],
  inscription: [_Inscription, 'inscription'],
  patterns: [_Patterns, 'coinjoin'],
  nonstandard: [_TransactionsFlagged, 'nonstandard'],
  sigopCost: [_Flow, 'block_sigop_cost'],
  versions: [(c, b) => _VersionCount(c, b, _TxCount), '*'],
  transferVolume: [_ValueFlow, 'transfer_volume'],
  perSecond: [_Windows, '*_per_second'],
});

/**
 * @typedef {{
 *   flag: SeriesPattern19<Boolean>,
 *   block: SeriesPattern18<Count>,
 * }} IndexerFeature
 */
const _IndexerFeature = _s({
  flag: [_i19, 'is_*'],
  block: [_i18, '*_tx_count'],
});

/**
 * @typedef {{
 *   flag: SeriesPattern19<Boolean>,
 *   block: SeriesPattern18<Count>,
 * }} Feature
 */
const _Feature = _s({
  flag: [_i19, 'has_*'],
  block: [_i18, '*_tx_count'],
});

/**
 * @typedef {{
 *   p2pk: Feature,
 *   p2pkh: Feature,
 *   p2ms: Feature,
 *   p2sh: Feature,
 *   p2wpkh: Feature,
 *   p2wsh: Feature,
 *   p2tr: Feature,
 *   p2a: Feature,
 *   emptyScript: Feature,
 *   unknownScript: Feature,
 *   opReturn: Feature,
 *   fakePubkey: Feature,
 *   fakeScripthash: Feature,
 *   inscription: Feature,
 *   annex: Feature,
 *   sighashAll: Feature,
 *   sighashNone: Feature,
 *   sighashSingle: Feature,
 *   sighashDefault: Feature,
 *   sighashAnyoneCanPay: Feature,
 *   rbfSignaling: IndexerFeature,
 *   dustOutput: Feature,
 *   epobc: IndexerFeature,
 *   oneInput: CountSeries,
 *   oneOutput: CountSeries,
 * }} TransactionFeatures
 */
const _TransactionFeatures = _s({
  p2pk: [_Feature, 'p2pk'],
  p2pkh: [_Feature, 'p2pkh'],
  p2ms: [_Feature, 'p2ms'],
  p2sh: [_Feature, 'p2sh'],
  p2wpkh: [_Feature, 'p2wpkh'],
  p2wsh: [_Feature, 'p2wsh'],
  p2tr: [_Feature, 'p2tr'],
  p2a: [_Feature, 'p2a'],
  emptyScript: [_Feature, 'empty_script'],
  unknownScript: [_Feature, 'unknown_script'],
  opReturn: [_Feature, 'op_return'],
  fakePubkey: [_Feature, 'fake_pubkey'],
  fakeScripthash: [_Feature, 'fake_scripthash'],
  inscription: [_Feature, 'inscription'],
  annex: [_Feature, 'annex'],
  sighashAll: [_Feature, '*_all'],
  sighashNone: [_Feature, '*_none'],
  sighashSingle: [_Feature, '*_single'],
  sighashDefault: [_Feature, '*_default'],
  sighashAnyoneCanPay: [_Feature, '*_anyone_can_pay'],
  rbfSignaling: [_IndexerFeature, 'rbf_signaling'],
  dustOutput: [_Feature, 'dust_output'],
  epobc: [_IndexerFeature, 'epobc'],
  oneInput: [_CountSeries, 'one_input_tx_count'],
  oneOutput: [_CountSeries, 'one_output_tx_count'],
});

/**
 * @typedef {{
 *   firstTxIndex: SeriesPattern18<TxIndex>,
 *   txid: SeriesPattern19<Txid>,
 *   version: SeriesPattern19<TxVersion>,
 *   locktime: SeriesPattern19<RawLockTime>,
 *   weight: SeriesPattern19<Weight>,
 *   size: SeriesPattern19<Bytes>,
 *   sigopCost: SeriesPattern19<SigOps>,
 *   firstTxinIndex: SeriesPattern19<TxInIndex>,
 *   firstTxoutIndex: SeriesPattern19<TxOutIndex>,
 *   features: TransactionFeatures,
 *   versions: VersionCount<CountSeries>,
 * }} IndexerTransactions
 */
const _IndexerTransactions = _s({
  firstTxIndex: [_i18, 'first_*_index'],
  txid: [_i19, 'txid'],
  version: [_i19, '*_version'],
  locktime: [_i19, '*_locktime'],
  weight: [_i19, '*_weight'],
  size: [_i19, '*_size'],
  sigopCost: [_i19, '*_sigop_cost'],
  firstTxinIndex: [_i19, 'first_txin_index'],
  firstTxoutIndex: [_i19, 'first_txout_index'],
  features: [_TransactionFeatures, 'sighash'],
  versions: [(c, b) => _VersionCount(c, b, _CountSeries), '*_count'],
});

/**
 * @typedef {{
 *   blockhash: SeriesPattern18<BlockHash>,
 *   coinbaseTag: SeriesPattern18<CoinbaseTag>,
 *   timestamp: SeriesPattern18<Timestamp>,
 *   medianTime: SeriesPattern18<Timestamp>,
 *   size: SeriesPattern18<Bytes>,
 *   weight: SeriesPattern18<Weight>,
 *   segwitTxCount: SeriesPattern18<Count>,
 *   segwitTxSize: SeriesPattern18<Bytes>,
 *   segwitTxWeight: SeriesPattern18<Weight>,
 * }} IndexerBlocks
 */
const _IndexerBlocks = _s({
  blockhash: [_i18, 'blockhash'],
  coinbaseTag: [_i18, 'coinbase_tag'],
  timestamp: [_i18, 'timestamp'],
  medianTime: [_i18, 'median_time'],
  size: [_i18, 'block_size'],
  weight: [_i18, 'block_weight'],
  segwitTxCount: [_i18, '*_count'],
  segwitTxSize: [_i18, '*_size'],
  segwitTxWeight: [_i18, '*_weight'],
});

/**
 * @typedef {{
 *   blocks: IndexerBlocks,
 *   transactions: IndexerTransactions,
 *   inputs: IndexerInputs,
 *   outputs: IndexerOutputs,
 *   addresses: Addrs<IndexerP2pk65, IndexerP2pk33, IndexerP2pkh, IndexerP2sh, IndexerP2wpkh, IndexerP2wsh, IndexerP2tr, IndexerP2a>,
 *   scripts: Scripts,
 *   opReturn: IndexerOpReturn,
 * }} Indexer
 */
const _Indexer = _s({
  blocks: [_IndexerBlocks, 'segwit_*'],
  transactions: [_IndexerTransactions, '*'],
  inputs: [_IndexerInputs, 'index'],
  outputs: [_IndexerOutputs, 'output'],
  addresses: [(c, b) => _Addrs(c, b, _IndexerP2pk65, _IndexerP2pk33, _IndexerP2pkh, _IndexerP2sh, _IndexerP2wpkh, _IndexerP2wsh, _IndexerP2tr, _IndexerP2a), ''],
  scripts: [_Scripts, 'script'],
  opReturn: [_IndexerOpReturn, 'op_return'],
});

/**
 * @typedef {{
 *   indexer: Indexer,
 *   mappings: Mappings,
 *   price: Price,
 *   blocks: Blocks,
 *   inputs: Inputs,
 *   outputs: Outputs,
 *   transactions: Transactions,
 *   mining: Mining,
 *   opReturn: OpReturn,
 *   pools: Pools,
 *   utxoSet: UtxoSetSeries,
 *   market: Market,
 *   age: Age,
 *   utxos: Utxos,
 *   addresses: Addresses,
 *   holders: Holders,
 *   entry: Entry,
 *   profitability: Profitability,
 *   supply: Supply,
 *   cointime: Cointime,
 *   coinflow: Coinflow,
 *   bedrock: Bedrock,
 *   rarityMeter: RarityMeter,
 * }} SeriesTree
 */
const _SeriesTree = _s({
  indexer: [_Indexer, 'tx'],
  mappings: [_Mappings, 'date'],
  price: [_Price, 'price'],
  blocks: [_Blocks, 'block'],
  inputs: [_Inputs, 'input'],
  outputs: [_Outputs, 'output'],
  transactions: [_Transactions, 'tx'],
  mining: [_Mining, 'hash'],
  opReturn: [_OpReturn, 'op_return'],
  pools: [_Pools, 'pool'],
  utxoSet: [_UtxoSetSeries, 'circulating_supply'],
  market: [_Market, 'price'],
  age: [_Age, 'age'],
  utxos: [_Utxos, 'amount'],
  addresses: [_Addresses, 'address'],
  holders: [_Holders, 'age'],
  entry: [_Entry, 'entry'],
  profitability: [_Profitability, 'sth'],
  supply: [_Supply, 'circulating_supply'],
  cointime: [_Cointime, 'age'],
  coinflow: [_Coinflow, 'age'],
  bedrock: [_Bedrock, 'bedrock'],
  rarityMeter: [_RarityMeter, 'rarity'],
});

/**
 * A method per API endpoint, and the typed series tree.
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
    "halving_epoch",
    "difficulty_epoch",
    "height",
    "tx_index",
    "txin_index",
    "txout_index",
    "empty_script_output_index",
    "op_return_output_index",
    "p2a_address_index",
    "p2ms_output_index",
    "p2pk33_address_index",
    "p2pk65_address_index",
    "p2pkh_address_index",
    "p2sh_address_index",
    "p2tr_address_index",
    "p2wpkh_address_index",
    "p2wsh_address_index",
    "unknown_script_output_index"
  ]);

  POOL_SLUG_TO_NAME = /** @type {const} */ ({
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

  HALVING_EPOCH_NAMES = /** @type {const} */ ({
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
  });

  VINTAGE_NAMES = /** @type {const} */ ({
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
  });

  ENTRY_NAMES = /** @type {const} */ ({
    "discount": {
      "id": "discount",
      "short": "Discount",
      "long": "Discount Coins"
    },
    "premium": {
      "id": "premium",
      "short": "Premium",
      "long": "Premium Coins"
    }
  });

  SPENDABLE_TYPE_NAMES = /** @type {const} */ ({
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
    "emptyScript": {
      "id": "empty_script",
      "short": "Empty",
      "long": "Empty Script"
    },
    "unknownScript": {
      "id": "unknown_script",
      "short": "Unknown",
      "long": "Unknown Script"
    }
  });

  AGE_RANGE_NAMES = /** @type {const} */ ({
    "under1h": {
      "id": "under_1h",
      "short": "<1h",
      "long": "Under 1 Hour Old"
    },
    "_1hTo1d": {
      "id": "1h_to_1d",
      "short": "1h-1d",
      "long": "1 Hour to 1 Day Old"
    },
    "_1dTo1w": {
      "id": "1d_to_1w",
      "short": "1d-1w",
      "long": "1 Day to 1 Week Old"
    },
    "_1wTo1m": {
      "id": "1w_to_1m",
      "short": "1w-1m",
      "long": "1 Week to 1 Month Old"
    },
    "_1mTo2m": {
      "id": "1m_to_2m",
      "short": "1m-2m",
      "long": "1 to 2 Months Old"
    },
    "_2mTo3m": {
      "id": "2m_to_3m",
      "short": "2m-3m",
      "long": "2 to 3 Months Old"
    },
    "_3mTo4m": {
      "id": "3m_to_4m",
      "short": "3m-4m",
      "long": "3 to 4 Months Old"
    },
    "_4mTo5m": {
      "id": "4m_to_5m",
      "short": "4m-5m",
      "long": "4 to 5 Months Old"
    },
    "_5mTo6m": {
      "id": "5m_to_6m",
      "short": "5m-6m",
      "long": "5 to 6 Months Old"
    },
    "_6mTo9m": {
      "id": "6m_to_9m",
      "short": "6m-9m",
      "long": "6 to 9 Months Old"
    },
    "_9mTo1y": {
      "id": "9m_to_1y",
      "short": "9m-1y",
      "long": "9 Months to 1 Year Old"
    },
    "_1yTo18m": {
      "id": "1y_to_18m",
      "short": "1y-18m",
      "long": "1 Year to 18 Months Old"
    },
    "_18mTo2y": {
      "id": "18m_to_2y",
      "short": "18m-2y",
      "long": "18 Months to 2 Years Old"
    },
    "_2yTo3y": {
      "id": "2y_to_3y",
      "short": "2y-3y",
      "long": "2 to 3 Years Old"
    },
    "_3yTo4y": {
      "id": "3y_to_4y",
      "short": "3y-4y",
      "long": "3 to 4 Years Old"
    },
    "_4yTo5y": {
      "id": "4y_to_5y",
      "short": "4y-5y",
      "long": "4 to 5 Years Old"
    },
    "_5yTo6y": {
      "id": "5y_to_6y",
      "short": "5y-6y",
      "long": "5 to 6 Years Old"
    },
    "_6yTo7y": {
      "id": "6y_to_7y",
      "short": "6y-7y",
      "long": "6 to 7 Years Old"
    },
    "_7yTo8y": {
      "id": "7y_to_8y",
      "short": "7y-8y",
      "long": "7 to 8 Years Old"
    },
    "_8yTo10y": {
      "id": "8y_to_10y",
      "short": "8y-10y",
      "long": "8 to 10 Years Old"
    },
    "_10yTo12y": {
      "id": "10y_to_12y",
      "short": "10y-12y",
      "long": "10 to 12 Years Old"
    },
    "_12yTo15y": {
      "id": "12y_to_15y",
      "short": "12y-15y",
      "long": "12 to 15 Years Old"
    },
    "over15y": {
      "id": "over_15y",
      "short": "15y+",
      "long": "15+ Years Old"
    }
  });

  AMOUNT_RANGE_NAMES = /** @type {const} */ ({
    "_0Sats": {
      "id": "0_sats",
      "short": "0 sats",
      "long": "0 Sats"
    },
    "_1To10Sats": {
      "id": "1_to_10_sats",
      "short": "1-10 sats",
      "long": "1-10 Sats"
    },
    "_10To100Sats": {
      "id": "10_to_100_sats",
      "short": "10-100 sats",
      "long": "10-100 Sats"
    },
    "_100To1kSats": {
      "id": "100_to_1k_sats",
      "short": "100-1k sats",
      "long": "100-1K Sats"
    },
    "_1kTo10kSats": {
      "id": "1k_to_10k_sats",
      "short": "1k-10k sats",
      "long": "1K-10K Sats"
    },
    "_10kTo100kSats": {
      "id": "10k_to_100k_sats",
      "short": "10k-100k sats",
      "long": "10K-100K Sats"
    },
    "_100kTo1mSats": {
      "id": "100k_to_1m_sats",
      "short": "100k-1M sats",
      "long": "100K-1M Sats"
    },
    "_1mTo10mSats": {
      "id": "1m_to_10m_sats",
      "short": "1M-10M sats",
      "long": "1M-10M Sats"
    },
    "_10mTo100mSats": {
      "id": "10m_to_100m_sats",
      "short": "0.1-1 BTC",
      "long": "0.1-1 BTC"
    },
    "_1To10Btc": {
      "id": "1_to_10_btc",
      "short": "1-10 BTC",
      "long": "1-10 BTC"
    },
    "_10To100Btc": {
      "id": "10_to_100_btc",
      "short": "10-100 BTC",
      "long": "10-100 BTC"
    },
    "_100To1kBtc": {
      "id": "100_to_1k_btc",
      "short": "100-1k BTC",
      "long": "100-1K BTC"
    },
    "_1kTo10kBtc": {
      "id": "1k_to_10k_btc",
      "short": "1k-10k BTC",
      "long": "1K-10K BTC"
    },
    "_10kTo100kBtc": {
      "id": "10k_to_100k_btc",
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
    "profitOver1000Percent": {
      "id": "profit_over_1000_percent",
      "short": "+>1000%",
      "long": "Over 1000% in Profit"
    },
    "profit500To1000Percent": {
      "id": "profit_500_to_1000_percent",
      "short": "+500-1000%",
      "long": "500-1000% in Profit"
    },
    "profit300To500Percent": {
      "id": "profit_300_to_500_percent",
      "short": "+300-500%",
      "long": "300-500% in Profit"
    },
    "profit200To300Percent": {
      "id": "profit_200_to_300_percent",
      "short": "+200-300%",
      "long": "200-300% in Profit"
    },
    "profit100To200Percent": {
      "id": "profit_100_to_200_percent",
      "short": "+100-200%",
      "long": "100-200% in Profit"
    },
    "profit90To100Percent": {
      "id": "profit_90_to_100_percent",
      "short": "+90-100%",
      "long": "90-100% in Profit"
    },
    "profit80To90Percent": {
      "id": "profit_80_to_90_percent",
      "short": "+80-90%",
      "long": "80-90% in Profit"
    },
    "profit70To80Percent": {
      "id": "profit_70_to_80_percent",
      "short": "+70-80%",
      "long": "70-80% in Profit"
    },
    "profit60To70Percent": {
      "id": "profit_60_to_70_percent",
      "short": "+60-70%",
      "long": "60-70% in Profit"
    },
    "profit50To60Percent": {
      "id": "profit_50_to_60_percent",
      "short": "+50-60%",
      "long": "50-60% in Profit"
    },
    "profit40To50Percent": {
      "id": "profit_40_to_50_percent",
      "short": "+40-50%",
      "long": "40-50% in Profit"
    },
    "profit30To40Percent": {
      "id": "profit_30_to_40_percent",
      "short": "+30-40%",
      "long": "30-40% in Profit"
    },
    "profit20To30Percent": {
      "id": "profit_20_to_30_percent",
      "short": "+20-30%",
      "long": "20-30% in Profit"
    },
    "profit10To20Percent": {
      "id": "profit_10_to_20_percent",
      "short": "+10-20%",
      "long": "10-20% in Profit"
    },
    "profit0To10Percent": {
      "id": "profit_0_to_10_percent",
      "short": "+0-10%",
      "long": "0-10% in Profit"
    },
    "loss0To10Percent": {
      "id": "loss_0_to_10_percent",
      "short": "-0-10%",
      "long": "0-10% in Loss"
    },
    "loss10To20Percent": {
      "id": "loss_10_to_20_percent",
      "short": "-10-20%",
      "long": "10-20% in Loss"
    },
    "loss20To30Percent": {
      "id": "loss_20_to_30_percent",
      "short": "-20-30%",
      "long": "20-30% in Loss"
    },
    "loss30To40Percent": {
      "id": "loss_30_to_40_percent",
      "short": "-30-40%",
      "long": "30-40% in Loss"
    },
    "loss40To50Percent": {
      "id": "loss_40_to_50_percent",
      "short": "-40-50%",
      "long": "40-50% in Loss"
    },
    "loss50To60Percent": {
      "id": "loss_50_to_60_percent",
      "short": "-50-60%",
      "long": "50-60% in Loss"
    },
    "loss60To70Percent": {
      "id": "loss_60_to_70_percent",
      "short": "-60-70%",
      "long": "60-70% in Loss"
    },
    "loss70To80Percent": {
      "id": "loss_70_to_80_percent",
      "short": "-70-80%",
      "long": "70-80% in Loss"
    },
    "loss80To90Percent": {
      "id": "loss_80_to_90_percent",
      "short": "-80-90%",
      "long": "80-90% in Loss"
    },
    "loss90To100Percent": {
      "id": "loss_90_to_100_percent",
      "short": "-90-100%",
      "long": "90-100% in Loss"
    }
  });

  OP_RETURN_KIND_NAMES = /** @type {const} */ ({
    "runes": "Runes",
    "witnessCommitment": "Witness commitment",
    "veriblock": "VeriBlock",
    "omni": "Omni",
    "stacks": "Stacks",
    "blockstack": "Blockstack",
    "colu": "Colu",
    "openAssets": "Open Assets",
    "komodo": "Komodo",
    "coinspark": "CoinSpark",
    "poet": "Po.et",
    "docproof": "DocProof",
    "factom": "Factom",
    "eternityWall": "Eternity Wall",
    "memo": "Memo",
    "bitproof": "Bitproof",
    "ascribe": "Ascribe",
    "stampery": "Stampery",
    "bareHash": "Bare hash",
    "text": "Text",
    "empty": "Empty",
    "unknown": "Unknown"
  });

  /**
   * The UTC start of bucket `i`; throws for an index without a date calendar.
   * @param {Index} index
   * @param {number} i
   * @returns {globalThis.Date}
   */
  indexToDate(index, i) {
    return indexToDate(index, i);
  }

  /**
   * The floor index (latest index whose date is <= the given date); throws for dates before the
   * index's first date.
   * @param {Index} index
   * @param {globalThis.Date} d
   * @returns {number}
   */
  dateToIndex(index, d) {
    return dateToIndex(index, d);
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
   * @param {OutputType} addressType
   * @param {Uint8Array | ArrayBuffer | ArrayBufferView | number[]} payload - Raw payload bytes matching addressType length
   * @param {number} nibbles
   * @param {{ signal?: AbortSignal, onValue?: (value: AddressHashPrefixMatches) => void, cache?: boolean, memCache?: boolean }} [options]
   * @returns {Promise<AddressHashPrefixMatches>}
   */
  getAddressPayloadHashPrefixMatches(addressType, payload, nibbles, options = {}) {
    _validateAddressPayloadForType(addressType, payload);
    const prefix = addressPayloadHashPrefix(payload, nibbles);
    return this.getAddressHashPrefixMatches(addressType, prefix, options);
  }

  /**
   * A series by a name known only at runtime; `series` has the typed paths.
   * @template {Index} I
   * @param {string} series
   * @param {I} index - The index name; date indexes also slice by Date
   * @returns {I extends DateIndex ? DateSeriesEndpoint<unknown> : SeriesEndpoint<unknown>}
   */
  seriesEndpoint(series, index) {
    return /** @type {any} */ (_endpoint(this, series, index));
  }

  /**
   * Health check
   *
   * Local health and query-readiness check. Returns server identity, uptime, and a coherent local sync snapshot without a bitcoind round-trip. Reads the published prefix during processing; an empty index returns 503 `state_updating` at once. Responses are not cached. For chain-tip catch-up, request `GET /api/server/sync`.
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
   * Endpoint: `GET /version`
   * @param {{ signal?: AbortSignal, onValue?: (value: ServerVersion) => void, cache?: boolean, memCache?: boolean }} [options]
   * @returns {Promise<ServerVersion>}
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
   * Returns allocated file bytes (file lengths on non-Unix platforms) for BRK and Bitcoin data. Each request scans both trees; these are independent observations, not an atomic filesystem snapshot. Conditional requests validate the newly observed totals. Directory-link cycles and excessive nesting fail without returning partial totals.
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
   * The available series as a tree of categories and subcategories.
   *
   * Endpoint: `GET /api/series`
   * @param {{ signal?: AbortSignal, onValue?: (value: TreeNode) => void, cache?: boolean, memCache?: boolean }} [options]
   * @returns {Promise<TreeNode>}
   */
  async getSeriesTree({ signal, onValue, cache, memCache } = {}) {
    const path = `/api/series`;
    return this.getJson(path, { signal, onValue, cache, memCache }, true);
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
    return this.getJson(path, { signal, onValue, cache, memCache }, { "by_db": true });
  }

  /**
   * List available indexes
   *
   * Returns every index a series can be read at, by the name series routes take.
   *
   * Endpoint: `GET /api/series/indexes`
   * @param {{ signal?: AbortSignal, onValue?: (value: Index[]) => void, cache?: boolean, memCache?: boolean }} [options]
   * @returns {Promise<Index[]>}
   */
  async listIndexes({ signal, onValue, cache, memCache } = {}) {
    const path = `/api/series/indexes`;
    return this.getJson(path, { signal, onValue, cache, memCache });
  }

  /**
   * Series list
   *
   * Paginated flat list of all available series names.
   *
   * Endpoint: `GET /api/series/list`
   *
   * @param {number=} [page] - Zero-based page number (default: 0)
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
   * @param {string} q - Words to find in series names and descriptions (`realized price`, `mvrv`), in any order
   * @param {Limit=} [limit] - Maximum number of results, 100 when omitted
   * @param {{ signal?: AbortSignal, onValue?: (value: SeriesName[]) => void, cache?: boolean, memCache?: boolean }} [options]
   * @returns {Promise<SeriesName[]>}
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
    const path = `/api/series/${encodeURIComponent(series)}`;
    return this.getJson(path, { signal, onValue, cache, memCache });
  }

  /**
   * Get series data
   *
   * Rows selected by a range of positions, dates or timestamps, as JSON or CSV.
   *
   * Endpoint: `GET /api/series/{series}/{index}`
   *
   * @param {SeriesName} series
   * @param {Index} index
   * @param {RangeIndex=} [start] - Inclusive start: integer index, date (YYYY-MM-DD), or timestamp (ISO 8601). Negative integers count from end.
   * @param {RangeIndex=} [end] - Exclusive end: integer index, date (YYYY-MM-DD), or timestamp (ISO 8601). Negative integers count from end.
   * @param {Limit=} [limit] - Maximum number of values to return (ignored if `end` is set); omitted, the rest of the range.
   * @param {Format=} [format]
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
    const path = `/api/series/${encodeURIComponent(series)}/${encodeURIComponent(index)}${query ? '?' + query : ''}`;
    if (format === 'csv') return this.getText(path, { signal, onValue, cache, memCache });
    return _wrapSeriesData(await this.getJson(path, { signal, cache, memCache, onValue: onValue && ((v) => onValue(_wrapSeriesData(v))) }));
  }

  /**
   * Get raw series data
   *
   * The data array without the `SeriesData` wrapper. Supports the same range and format parameters as `GET /api/series/{series}/{index}`.
   *
   * Endpoint: `GET /api/series/{series}/{index}/data`
   *
   * @param {SeriesName} series
   * @param {Index} index
   * @param {RangeIndex=} [start] - Inclusive start: integer index, date (YYYY-MM-DD), or timestamp (ISO 8601). Negative integers count from end.
   * @param {RangeIndex=} [end] - Exclusive end: integer index, date (YYYY-MM-DD), or timestamp (ISO 8601). Negative integers count from end.
   * @param {Limit=} [limit] - Maximum number of values to return (ignored if `end` is set); omitted, the rest of the range.
   * @param {Format=} [format]
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
    const path = `/api/series/${encodeURIComponent(series)}/${encodeURIComponent(index)}/data${query ? '?' + query : ''}`;
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
   * @param {SeriesName} series
   * @param {Index} index
   * @param {{ signal?: AbortSignal, onValue?: (value: *) => void, cache?: boolean, memCache?: boolean }} [options]
   * @returns {Promise<*>}
   */
  async getSeriesLatest(series, index, { signal, onValue, cache, memCache } = {}) {
    const path = `/api/series/${encodeURIComponent(series)}/${encodeURIComponent(index)}/latest`;
    return this.getJson(path, { signal, onValue, cache, memCache });
  }

  /**
   * Get series data length
   *
   * Endpoint: `GET /api/series/{series}/{index}/len`
   *
   * @param {SeriesName} series
   * @param {Index} index
   * @param {{ signal?: AbortSignal, onValue?: (value: number) => void, cache?: boolean, memCache?: boolean }} [options]
   * @returns {Promise<number>}
   */
  async getSeriesLen(series, index, { signal, onValue, cache, memCache } = {}) {
    const path = `/api/series/${encodeURIComponent(series)}/${encodeURIComponent(index)}/len`;
    return this.getJson(path, { signal, onValue, cache, memCache });
  }

  /**
   * Get series version
   *
   * Returns the vector's schema/computation version, not its length or latest update. Appends and reorgs do not by themselves change this version.
   *
   * Endpoint: `GET /api/series/{series}/{index}/version`
   *
   * @param {SeriesName} series
   * @param {Index} index
   * @param {{ signal?: AbortSignal, onValue?: (value: Version) => void, cache?: boolean, memCache?: boolean }} [options]
   * @returns {Promise<Version>}
   */
  async getSeriesVersion(series, index, { signal, onValue, cache, memCache } = {}) {
    const path = `/api/series/${encodeURIComponent(series)}/${encodeURIComponent(index)}/version`;
    return this.getJson(path, { signal, onValue, cache, memCache });
  }

  /**
   * Bulk series data
   *
   * Fetch multiple series in a single request. Supports filtering by index and date range. Returns an array of SeriesData objects. For a single series, use `get_series` instead.
   *
   * Endpoint: `GET /api/series/bulk`
   *
   * @param {SeriesList} series
   * @param {Index} index
   * @param {RangeIndex=} [start] - Inclusive start: integer index, date (YYYY-MM-DD), or timestamp (ISO 8601). Negative integers count from end.
   * @param {RangeIndex=} [end] - Exclusive end: integer index, date (YYYY-MM-DD), or timestamp (ISO 8601). Negative integers count from end.
   * @param {Limit=} [limit] - Maximum number of values to return (ignored if `end` is set); omitted, the rest of the range.
   * @param {Format=} [format]
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
   * Every URPD cohort once a UTXO set is published (none before): the age aggregates (`all`, `sth`, `lth`, `age_under_4m`, ...), then the age bands from the youngest (`age_under_1h`, `age_1h_to_1d`, ...).
   *
   * Endpoint: `GET /api/urpd`
   * @param {{ signal?: AbortSignal, onValue?: (value: UrpdCohort[]) => void, cache?: boolean, memCache?: boolean }} [options]
   * @returns {Promise<UrpdCohort[]>}
   */
  async listUrpdCohorts({ signal, onValue, cache, memCache } = {}) {
    const path = `/api/urpd`;
    return this.getJson(path, { signal, onValue, cache, memCache });
  }

  /**
   * Latest URPD
   *
   * URPD for the latest published block; `date` echoes the date served. Fields, units, price-grid rounding and weighting as in `GET /api/urpd/{cohort}/{point}`.
   *
   * Endpoint: `GET /api/urpd/{cohort}`
   *
   * @param {UrpdCohort} cohort
   * @param {UrpdAggregation=} [agg] - Aggregation strategy. Default: raw (no aggregation).
   * @param {UrpdWeight=} [weight] - Supply weighting. Default: none (unweighted).
   * @param {{ signal?: AbortSignal, onValue?: (value: Urpd) => void, cache?: boolean, memCache?: boolean }} [options]
   * @returns {Promise<Urpd>}
   */
  async getUrpdLatest(cohort, agg, weight, { signal, onValue, cache, memCache } = {}) {
    const params = new URLSearchParams();
    if (agg !== undefined) params.set('agg', String(agg));
    if (weight !== undefined) params.set('weight', String(weight));
    const query = params.toString();
    const path = `/api/urpd/${encodeURIComponent(cohort)}${query ? '?' + query : ''}`;
    return this.getJson(path, { signal, onValue, cache, memCache });
  }

  /**
   * URPD at block height or date
   *
   * URPD for a cohort at a block height or the last block of a UTC day. Returns `{ height, hash, date, cohort, weight, aggregation, close, supply, buckets }` where `buckets` is columns, `{ price, supply, realized_cap, unrealized_pnl }`, one entry per bucket in ascending price: the same block header and columnar layout as `GET /api/utxo-set/{point}`. `close`, `price`, `realized_cap`, and `unrealized_pnl` are USD; `supply` is BTC. `unrealized_pnl` can be negative. Bucket `realized_cap` and `unrealized_pnl` value each coin at its creation price on the price grid (four significant digits of the price in cents, within 0.05% of the exact price), rounded to the cent per bucket, so the buckets need not sum to the realized cap series. With a `wakefulness` or `mobility` `weight`, `supply` is the weighted supply (whole satoshis) and the bucket values follow it.
   *
   * Endpoint: `GET /api/urpd/{cohort}/{point}`
   *
   * @param {UrpdCohort} cohort
   * @param {HeightOrDate} point - Confirmed block height (`840000`) or UTC calendar day (`YYYY-MM-DD`)
   * @param {UrpdAggregation=} [agg] - Aggregation strategy. Default: raw (no aggregation).
   * @param {UrpdWeight=} [weight] - Supply weighting. Default: none (unweighted).
   * @param {{ signal?: AbortSignal, onValue?: (value: Urpd) => void, cache?: boolean, memCache?: boolean }} [options]
   * @returns {Promise<Urpd>}
   */
  async getUrpd(cohort, point, agg, weight, { signal, onValue, cache, memCache } = {}) {
    const params = new URLSearchParams();
    if (agg !== undefined) params.set('agg', String(agg));
    if (weight !== undefined) params.set('weight', String(weight));
    const query = params.toString();
    const path = `/api/urpd/${encodeURIComponent(cohort)}/${encodeURIComponent(point)}${query ? '?' + query : ''}`;
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
   * @param {{ signal?: AbortSignal, onValue?: (value: PricePoint) => void, cache?: boolean, memCache?: boolean }} [options]
   * @returns {Promise<PricePoint>}
   */
  async getPrices({ signal, onValue, cache, memCache } = {}) {
    const path = `/api/v1/prices`;
    return this.getJson(path, { signal, onValue, cache, memCache });
  }

  /**
   * Historical price
   *
   * Completed four-hour BTC/USD closes, newest first as mempool.space lists them, labeled by interval end, from the first priced interval (2010-08-16). With a UNIX timestamp, returns the latest nonempty completed close at or before it; before the first close returns an empty list. The current partial interval is excluded. USD only (`currency` accepts `USD`); exchangeRates is empty.
   *
   * *[Mempool.space docs](https://mempool.space/docs/api/rest#get-historical-price)*
   *
   * Endpoint: `GET /api/v1/historical-price`
   *
   * @param {Timestamp=} [timestamp] - UNIX timestamp: the latest close at or before it
   * @param {Currency=} [currency] - Quote currency: USD, the only one served
   * @param {{ signal?: AbortSignal, onValue?: (value: HistoricalPrice) => void, cache?: boolean, memCache?: boolean }} [options]
   * @returns {Promise<HistoricalPrice>}
   */
  async getHistoricalPrice(timestamp, currency, { signal, onValue, cache, memCache } = {}) {
    const params = new URLSearchParams();
    if (timestamp !== undefined) params.set('timestamp', String(timestamp));
    if (currency !== undefined) params.set('currency', String(currency));
    const query = params.toString();
    const path = `/api/v1/historical-price${query ? '?' + query : ''}`;
    return this.getJson(path, { signal, onValue, cache, memCache });
  }

  /**
   * Address hash-prefix matches
   *
   * Find addresses by address type and by the first 1-16 hex nibbles of RapidHash v3 over the raw address payload bytes. Intended for privacy-preserving client-side wallet discovery without sending raw addresses or xpubs. Fetch metadata with `GET /api/address/{address}`.
   *
   * Endpoint: `GET /api/address/hash-prefix/{address_type}/{prefix}`
   *
   * @param {OutputType} address_type
   * @param {string} prefix - First 1–16 hexadecimal nibbles of the RapidHash v3 hash over the raw
address payload bytes.
   * @param {{ signal?: AbortSignal, onValue?: (value: AddressHashPrefixMatches) => void, cache?: boolean, memCache?: boolean }} [options]
   * @returns {Promise<AddressHashPrefixMatches>}
   */
  async getAddressHashPrefixMatches(address_type, prefix, { signal, onValue, cache, memCache } = {}) {
    const path = `/api/address/hash-prefix/${encodeURIComponent(address_type)}/${encodeURIComponent(prefix)}`;
    return this.getJson(path, { signal, onValue, cache, memCache });
  }

  /**
   * Address information
   *
   * Balance and transaction counts. Supports P2PKH, P2SH, P2WPKH, P2WSH, P2TR and P2A addresses, and P2PK public keys in hex (33 or 65 bytes).
   *
   * *[Mempool.space docs](https://mempool.space/docs/api/rest#get-address)*
   *
   * Endpoint: `GET /api/address/{address}`
   *
   * @param {Address} address
   * @param {{ signal?: AbortSignal, onValue?: (value: AddressStats) => void, cache?: boolean, memCache?: boolean }} [options]
   * @returns {Promise<AddressStats>}
   */
  async getAddress(address, { signal, onValue, cache, memCache } = {}) {
    const path = `/api/address/${encodeURIComponent(address)}`;
    return this.getJson(path, { signal, onValue, cache, memCache });
  }

  /**
   * Address transactions
   *
   * Get transaction history for an address, newest first: up to `max_txs` (50 by default and at most), mempool transactions first and confirmed ones filling the rest. To continue, pass the last returned txid as `after_txid`, or request `GET /api/address/{address}/txs/chain/{after_txid}` for confirmed history.
   *
   * *[Mempool.space docs](https://mempool.space/docs/api/rest#get-address-transactions)*
   *
   * Endpoint: `GET /api/address/{address}/txs`
   *
   * @param {Address} address
   * @param {Txid=} [after_txid] - Continue after this transaction, a mempool or a confirmed one: the older ones (a confirmed
cursor skips the mempool). One that is neither answers 422.
   * @param {number=} [max_txs] - Transactions to return, at most 50 (the default); larger values return 50.
   * @param {{ signal?: AbortSignal, onValue?: (value: Transaction[]) => void, cache?: boolean, memCache?: boolean }} [options]
   * @returns {Promise<Transaction[]>}
   */
  async getAddressTxs(address, after_txid, max_txs, { signal, onValue, cache, memCache } = {}) {
    const params = new URLSearchParams();
    if (after_txid !== undefined) params.set('after_txid', String(after_txid));
    if (max_txs !== undefined) params.set('max_txs', String(max_txs));
    const query = params.toString();
    const path = `/api/address/${encodeURIComponent(address)}/txs${query ? '?' + query : ''}`;
    return this.getJson(path, { signal, onValue, cache, memCache });
  }

  /**
   * Address confirmed transactions
   *
   * Get the newest confirmed transactions for an address, up to `max_txs` (25 by default and at most). For pagination, request `GET /api/address/{address}/txs/chain/{after_txid}` with the last returned txid.
   *
   * *[Mempool.space docs](https://mempool.space/docs/api/rest#get-address-transactions-chain)*
   *
   * Endpoint: `GET /api/address/{address}/txs/chain`
   *
   * @param {Address} address
   * @param {number=} [max_txs] - Transactions to return, at most 25 (the default); larger values return 25.
   * @param {{ signal?: AbortSignal, onValue?: (value: Transaction[]) => void, cache?: boolean, memCache?: boolean }} [options]
   * @returns {Promise<Transaction[]>}
   */
  async getAddressConfirmedTxs(address, max_txs, { signal, onValue, cache, memCache } = {}) {
    const params = new URLSearchParams();
    if (max_txs !== undefined) params.set('max_txs', String(max_txs));
    const query = params.toString();
    const path = `/api/address/${encodeURIComponent(address)}/txs/chain${query ? '?' + query : ''}`;
    return this.getJson(path, { signal, onValue, cache, memCache });
  }

  /**
   * Address confirmed transactions (paginated)
   *
   * Get the next confirmed transactions strictly older than `after_txid`, up to `max_txs` (25 by default and at most); a txid outside the address's history answers `[]` (Esplora-canonical pagination form, matches mempool.space).
   *
   * *[Mempool.space docs](https://mempool.space/docs/api/rest#get-address-transactions-chain)*
   *
   * Endpoint: `GET /api/address/{address}/txs/chain/{after_txid}`
   *
   * @param {Address} address
   * @param {Txid} after_txid - Last txid from the previous page (return transactions strictly older than this)
   * @param {number=} [max_txs] - Transactions to return, at most 25 (the default); larger values return 25.
   * @param {{ signal?: AbortSignal, onValue?: (value: Transaction[]) => void, cache?: boolean, memCache?: boolean }} [options]
   * @returns {Promise<Transaction[]>}
   */
  async getAddressConfirmedTxsAfter(address, after_txid, max_txs, { signal, onValue, cache, memCache } = {}) {
    const params = new URLSearchParams();
    if (max_txs !== undefined) params.set('max_txs', String(max_txs));
    const query = params.toString();
    const path = `/api/address/${encodeURIComponent(address)}/txs/chain/${encodeURIComponent(after_txid)}${query ? '?' + query : ''}`;
    return this.getJson(path, { signal, onValue, cache, memCache });
  }

  /**
   * Address mempool transactions
   *
   * Get unconfirmed transactions for an address from the mempool, newest first, up to `max_txs` (50 by default and at most).
   *
   * *[Mempool.space docs](https://mempool.space/docs/api/rest#get-address-transactions-mempool)*
   *
   * Endpoint: `GET /api/address/{address}/txs/mempool`
   *
   * @param {Address} address
   * @param {number=} [max_txs] - Transactions to return, at most 50 (the default); larger values return 50.
   * @param {{ signal?: AbortSignal, onValue?: (value: Transaction[]) => void, cache?: boolean, memCache?: boolean }} [options]
   * @returns {Promise<Transaction[]>}
   */
  async getAddressMempoolTxs(address, max_txs, { signal, onValue, cache, memCache } = {}) {
    const params = new URLSearchParams();
    if (max_txs !== undefined) params.set('max_txs', String(max_txs));
    const query = params.toString();
    const path = `/api/address/${encodeURIComponent(address)}/txs/mempool${query ? '?' + query : ''}`;
    return this.getJson(path, { signal, onValue, cache, memCache });
  }

  /**
   * Address UTXOs
   *
   * Get unspent transaction outputs (UTXOs) for an address, including unconfirmed ones; outputs that mempool transactions spend are left out.
   *
   * *[Mempool.space docs](https://mempool.space/docs/api/rest#get-address-utxo)*
   *
   * Endpoint: `GET /api/address/{address}/utxo`
   *
   * @param {Address} address
   * @param {{ signal?: AbortSignal, onValue?: (value: Utxo[]) => void, cache?: boolean, memCache?: boolean }} [options]
   * @returns {Promise<Utxo[]>}
   */
  async getAddressUtxos(address, { signal, onValue, cache, memCache } = {}) {
    const path = `/api/address/${encodeURIComponent(address)}/utxo`;
    return this.getJson(path, { signal, onValue, cache, memCache });
  }

  /**
   * Validate address
   *
   * Validate a Bitcoin address and get information about its type and scriptPubKey. Returns `isvalid: false` with an error message for invalid addresses; a string that isn't 2 to 130 letters and digits answers 400.
   *
   * *[Mempool.space docs](https://mempool.space/docs/api/rest#get-address-validate)*
   *
   * Endpoint: `GET /api/v1/validate-address/{address}`
   *
   * @param {string} address - Bitcoin address to validate: 2 to 130 letters and digits (a P2PK public key in hex is 130);
anything else answers 400, as mempool.space does
   * @param {{ signal?: AbortSignal, onValue?: (value: AddressValidation) => void, cache?: boolean, memCache?: boolean }} [options]
   * @returns {Promise<AddressValidation>}
   */
  async validateAddress(address, { signal, onValue, cache, memCache } = {}) {
    const path = `/api/v1/validate-address/${encodeURIComponent(address)}`;
    return this.getJson(path, { signal, onValue, cache, memCache });
  }

  /**
   * Block information
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
    const path = `/api/block/${encodeURIComponent(hash)}`;
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
    const path = `/api/v1/block/${encodeURIComponent(hash)}`;
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
    const path = `/api/block/${encodeURIComponent(hash)}/header`;
    return this.getText(path, { signal, onValue, cache, memCache });
  }

  /**
   * Block hash by height
   *
   * Block hash as plain text.
   *
   * *[Mempool.space docs](https://mempool.space/docs/api/rest#get-block-height)*
   *
   * Endpoint: `GET /api/block-height/{height}`
   *
   * @param {Height} height
   * @param {{ signal?: AbortSignal, onValue?: (value: BlockHash) => void, cache?: boolean, memCache?: boolean }} [options]
   * @returns {Promise<BlockHash>}
   */
  async getBlockHashByHeight(height, { signal, onValue, cache, memCache } = {}) {
    const path = `/api/block-height/${encodeURIComponent(height)}`;
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
    const path = `/api/v1/mining/blocks/timestamp/${encodeURIComponent(timestamp)}`;
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
    const path = `/api/block/${encodeURIComponent(hash)}/raw`;
    return this.getBytes(path, { signal, onValue, cache, memCache });
  }

  /**
   * Block status
   *
   * Whether the block is in the best chain and, if so, its height and the hash of the next block. A block outside the best chain (orphaned or unknown) reports `in_best_chain: false`.
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
    const path = `/api/block/${encodeURIComponent(hash)}/status`;
    return this.getJson(path, { signal, onValue, cache, memCache });
  }

  /**
   * Block tip height
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
   * @param {BlockHash} hash
   * @param {BlockTxIndex} index - Transaction index within the block (0-based)
   * @param {{ signal?: AbortSignal, onValue?: (value: Txid) => void, cache?: boolean, memCache?: boolean }} [options]
   * @returns {Promise<Txid>}
   */
  async getBlockTxid(hash, index, { signal, onValue, cache, memCache } = {}) {
    const path = `/api/block/${encodeURIComponent(hash)}/txid/${encodeURIComponent(index)}`;
    return this.getText(path, { signal, onValue, cache, memCache });
  }

  /**
   * Block transaction IDs
   *
   * Transaction IDs in block order.
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
    const path = `/api/block/${encodeURIComponent(hash)}/txids`;
    return this.getJson(path, { signal, onValue, cache, memCache });
  }

  /**
   * Block transactions
   *
   * Up to 25 transactions starting at index 0.
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
    const path = `/api/block/${encodeURIComponent(hash)}/txs`;
    return this.getJson(path, { signal, onValue, cache, memCache });
  }

  /**
   * Block transactions (paginated)
   *
   * Up to 25 transactions starting at `start_index`, a multiple of 25.
   *
   * *[Mempool.space docs](https://mempool.space/docs/api/rest#get-block-transactions)*
   *
   * Endpoint: `GET /api/block/{hash}/txs/{start_index}`
   *
   * @param {BlockHash} hash
   * @param {BlockTxIndex} start_index - Starting transaction index within the block (0-based)
   * @param {{ signal?: AbortSignal, onValue?: (value: Transaction[]) => void, cache?: boolean, memCache?: boolean }} [options]
   * @returns {Promise<Transaction[]>}
   */
  async getBlockTxsFromIndex(hash, start_index, { signal, onValue, cache, memCache } = {}) {
    const path = `/api/block/${encodeURIComponent(hash)}/txs/${encodeURIComponent(start_index)}`;
    return this.getJson(path, { signal, onValue, cache, memCache });
  }

  /**
   * Recent blocks
   *
   * The last 10 blocks.
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
   * Retrieve up to 10 blocks going backwards from the given height. For example, height=100 returns blocks 100, 99, 98, ..., 91. Height=0 returns only block 0. A height past the tip is not found.
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
    const path = `/api/blocks/${encodeURIComponent(height)}`;
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
   * Retrieve up to 15 blocks with extended data going backwards from the given height: the published blocks among the 15 heights ending at it, so fewer (or none) past the tip.
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
    const path = `/api/v1/blocks/${encodeURIComponent(height)}`;
    return this.getJson(path, { signal, onValue, cache, memCache });
  }

  /**
   * List all mining pools
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
   * Get mining pool statistics for a time period.
   *
   * *[Mempool.space docs](https://mempool.space/docs/api/rest#get-mining-pools)*
   *
   * Endpoint: `GET /api/v1/mining/pools/{time_period}`
   *
   * @param {TimePeriod} time_period - Lookback period: `24h`, `3d`, `1w`, `1m`, `3m`, `6m`, `1y`, `2y`, `3y`, `4y` or `all`.
   * @param {{ signal?: AbortSignal, onValue?: (value: PoolsSummary) => void, cache?: boolean, memCache?: boolean }} [options]
   * @returns {Promise<PoolsSummary>}
   */
  async getPoolRanking(time_period, { signal, onValue, cache, memCache } = {}) {
    const path = `/api/v1/mining/pools/${encodeURIComponent(time_period)}`;
    return this.getJson(path, { signal, onValue, cache, memCache });
  }

  /**
   * Mining pool details
   *
   * Block counts and mining shares by time period for one pool.
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
    const path = `/api/v1/mining/pool/${encodeURIComponent(slug)}`;
    return this.getJson(path, { signal, onValue, cache, memCache });
  }

  /**
   * All pools hashrate (all time)
   *
   * Get weekly hashrate data, over the whole chain, of the mining pools that mined in the last week, by time and then their weekly rank.
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
   * Get weekly hashrate data, for a time period, of the mining pools that mined in the last week, by time and then their weekly rank.
   *
   * *[Mempool.space docs](https://mempool.space/docs/api/rest#get-mining-pool-hashrates)*
   *
   * Endpoint: `GET /api/v1/mining/hashrate/pools/{time_period}`
   *
   * @param {TimePeriod} time_period - Lookback period: `24h`, `3d`, `1w`, `1m`, `3m`, `6m`, `1y`, `2y`, `3y`, `4y` or `all`.
   * @param {{ signal?: AbortSignal, onValue?: (value: PoolHashrateEntry[]) => void, cache?: boolean, memCache?: boolean }} [options]
   * @returns {Promise<PoolHashrateEntry[]>}
   */
  async getPoolsHashrateByPeriod(time_period, { signal, onValue, cache, memCache } = {}) {
    const path = `/api/v1/mining/hashrate/pools/${encodeURIComponent(time_period)}`;
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
    const path = `/api/v1/mining/pool/${encodeURIComponent(slug)}/hashrate`;
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
    const path = `/api/v1/mining/pool/${encodeURIComponent(slug)}/blocks`;
    return this.getJson(path, { signal, onValue, cache, memCache });
  }

  /**
   * Mining pool blocks before height
   *
   * Get up to 100 blocks mined by a specific pool strictly below the given height (pass the last height of a page to get the next one).
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
  async getPoolBlocksFromHeight(slug, height, { signal, onValue, cache, memCache } = {}) {
    const path = `/api/v1/mining/pool/${encodeURIComponent(slug)}/blocks/${encodeURIComponent(height)}`;
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
   * Get network hashrate and difficulty data for a time period.
   *
   * *[Mempool.space docs](https://mempool.space/docs/api/rest#get-hashrate)*
   *
   * Endpoint: `GET /api/v1/mining/hashrate/{time_period}`
   *
   * @param {TimePeriod} time_period - Lookback period: `24h`, `3d`, `1w`, `1m`, `3m`, `6m`, `1y`, `2y`, `3y`, `4y` or `all`.
   * @param {{ signal?: AbortSignal, onValue?: (value: HashrateSummary) => void, cache?: boolean, memCache?: boolean }} [options]
   * @returns {Promise<HashrateSummary>}
   */
  async getHashrateByPeriod(time_period, { signal, onValue, cache, memCache } = {}) {
    const path = `/api/v1/mining/hashrate/${encodeURIComponent(time_period)}`;
    return this.getJson(path, { signal, onValue, cache, memCache });
  }

  /**
   * Difficulty adjustments (all time)
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
   * Get historical difficulty adjustments for a time period.
   *
   * *[Mempool.space docs](https://mempool.space/docs/api/rest#get-difficulty-adjustments)*
   *
   * Endpoint: `GET /api/v1/mining/difficulty-adjustments/{time_period}`
   *
   * @param {TimePeriod} time_period - Lookback period: `24h`, `3d`, `1w`, `1m`, `3m`, `6m`, `1y`, `2y`, `3y`, `4y` or `all`.
   * @param {{ signal?: AbortSignal, onValue?: (value: DifficultyAdjustmentEntry[]) => void, cache?: boolean, memCache?: boolean }} [options]
   * @returns {Promise<DifficultyAdjustmentEntry[]>}
   */
  async getDifficultyAdjustmentsByPeriod(time_period, { signal, onValue, cache, memCache } = {}) {
    const path = `/api/v1/mining/difficulty-adjustments/${encodeURIComponent(time_period)}`;
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
    const path = `/api/v1/mining/reward-stats/${encodeURIComponent(block_count)}`;
    return this.getJson(path, { signal, onValue, cache, memCache });
  }

  /**
   * Block fees
   *
   * Get average total fees per block for a time period.
   *
   * *[Mempool.space docs](https://mempool.space/docs/api/rest#get-block-fees)*
   *
   * Endpoint: `GET /api/v1/mining/blocks/fees/{time_period}`
   *
   * @param {TimePeriod} time_period - Lookback period: `24h`, `3d`, `1w`, `1m`, `3m`, `6m`, `1y`, `2y`, `3y`, `4y` or `all`.
   * @param {{ signal?: AbortSignal, onValue?: (value: BlockFeesEntry[]) => void, cache?: boolean, memCache?: boolean }} [options]
   * @returns {Promise<BlockFeesEntry[]>}
   */
  async getBlockFees(time_period, { signal, onValue, cache, memCache } = {}) {
    const path = `/api/v1/mining/blocks/fees/${encodeURIComponent(time_period)}`;
    return this.getJson(path, { signal, onValue, cache, memCache });
  }

  /**
   * Block rewards
   *
   * Get average coinbase reward (subsidy + fees) per block for a time period.
   *
   * *[Mempool.space docs](https://mempool.space/docs/api/rest#get-block-rewards)*
   *
   * Endpoint: `GET /api/v1/mining/blocks/rewards/{time_period}`
   *
   * @param {TimePeriod} time_period - Lookback period: `24h`, `3d`, `1w`, `1m`, `3m`, `6m`, `1y`, `2y`, `3y`, `4y` or `all`.
   * @param {{ signal?: AbortSignal, onValue?: (value: BlockRewardsEntry[]) => void, cache?: boolean, memCache?: boolean }} [options]
   * @returns {Promise<BlockRewardsEntry[]>}
   */
  async getBlockRewards(time_period, { signal, onValue, cache, memCache } = {}) {
    const path = `/api/v1/mining/blocks/rewards/${encodeURIComponent(time_period)}`;
    return this.getJson(path, { signal, onValue, cache, memCache });
  }

  /**
   * Block fee rates
   *
   * Get block fee rate percentiles (min, 10th, 25th, median, 75th, 90th, max) for a time period, each a whole sat/vB (`/precise` for 3 decimals).
   *
   * *[Mempool.space docs](https://mempool.space/docs/api/rest#get-block-feerates)*
   *
   * Endpoint: `GET /api/v1/mining/blocks/fee-rates/{time_period}`
   *
   * @param {TimePeriod} time_period - Lookback period: `24h`, `3d`, `1w`, `1m`, `3m`, `6m`, `1y`, `2y`, `3y`, `4y` or `all`.
   * @param {{ signal?: AbortSignal, onValue?: (value: BlockFeeRatesEntry[]) => void, cache?: boolean, memCache?: boolean }} [options]
   * @returns {Promise<BlockFeeRatesEntry[]>}
   */
  async getBlockFeeRates(time_period, { signal, onValue, cache, memCache } = {}) {
    const path = `/api/v1/mining/blocks/fee-rates/${encodeURIComponent(time_period)}`;
    return this.getJson(path, { signal, onValue, cache, memCache });
  }

  /**
   * Block fee rates (precise)
   *
   * Get block fee rate percentiles (min, 10th, 25th, median, 75th, 90th, max) for a time period, each in sat/vB to 3 decimals.
   *
   * *[Mempool.space docs](https://mempool.space/docs/api/rest#get-block-feerates)*
   *
   * Endpoint: `GET /api/v1/mining/blocks/fee-rates/{time_period}/precise`
   *
   * @param {TimePeriod} time_period - Lookback period: `24h`, `3d`, `1w`, `1m`, `3m`, `6m`, `1y`, `2y`, `3y`, `4y` or `all`.
   * @param {{ signal?: AbortSignal, onValue?: (value: BlockFeeRatesPreciseEntry[]) => void, cache?: boolean, memCache?: boolean }} [options]
   * @returns {Promise<BlockFeeRatesPreciseEntry[]>}
   */
  async getBlockFeeRatesPrecise(time_period, { signal, onValue, cache, memCache } = {}) {
    const path = `/api/v1/mining/blocks/fee-rates/${encodeURIComponent(time_period)}/precise`;
    return this.getJson(path, { signal, onValue, cache, memCache });
  }

  /**
   * Block sizes and weights
   *
   * Get average block sizes and weights for a time period.
   *
   * *[Mempool.space docs](https://mempool.space/docs/api/rest#get-sizes-weights)*
   *
   * Endpoint: `GET /api/v1/mining/blocks/sizes-weights/{time_period}`
   *
   * @param {TimePeriod} time_period - Lookback period: `24h`, `3d`, `1w`, `1m`, `3m`, `6m`, `1y`, `2y`, `3y`, `4y` or `all`.
   * @param {{ signal?: AbortSignal, onValue?: (value: BlockSizesWeights) => void, cache?: boolean, memCache?: boolean }} [options]
   * @returns {Promise<BlockSizesWeights>}
   */
  async getBlockSizesWeights(time_period, { signal, onValue, cache, memCache } = {}) {
    const path = `/api/v1/mining/blocks/sizes-weights/${encodeURIComponent(time_period)}`;
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
   * Recommended fee rates by confirmation target, in whole sat/vB.
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
   * @param {{ signal?: AbortSignal, onValue?: (value: PreciseRecommendedFees) => void, cache?: boolean, memCache?: boolean }} [options]
   * @returns {Promise<PreciseRecommendedFees>}
   */
  async getPreciseFees({ signal, onValue, cache, memCache } = {}) {
    const path = `/api/v1/fees/precise`;
    return this.getJson(path, { signal, onValue, cache, memCache });
  }

  /**
   * Mempool statistics
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
   * Returns up to 25 most-recent RBF replacement trees across the whole mempool. Each entry has the shape of `replacements` in `GET /api/v1/tx/{txid}/rbf`.
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
   * Projected mempool block
   *
   * A projected mempool block with its transactions in order and its stats, by position as `GET /api/v1/fees/mempool-blocks` lists them: 0 is the next block, Bitcoin Core's `getblocktemplate` selection, and the only one served for now (others answer 404). The returned `hash` is an opaque content token: poll `GET /api/mempool-blocks/{index}/diff/{hash}` to follow changes instead of refetching the block.
   *
   * Endpoint: `GET /api/mempool-blocks/{index}`
   *
   * @param {MempoolBlockIndex} index
   * @param {{ signal?: AbortSignal, onValue?: (value: MempoolBlockContent) => void, cache?: boolean, memCache?: boolean }} [options]
   * @returns {Promise<MempoolBlockContent>}
   */
  async getMempoolBlock(index, { signal, onValue, cache, memCache } = {}) {
    const path = `/api/mempool-blocks/${encodeURIComponent(index)}`;
    return this.getJson(path, { signal, onValue, cache, memCache });
  }

  /**
   * Projected mempool block changes
   *
   * The projected mempool block's changes since the client's copy at `since`. `order` is the new block in order: each entry is either `[start, count]`, the client's transactions `start..start + count` kept in order, or a transaction object, a body to place there. Walk `order` once to rebuild, and replace the cached `stats` with the response's; `removed` lists the txids that left, so clients can evict cached bodies. Use the response `hash` as the next `since`; with `If-None-Match` an unchanged block answers 304. Returns `404` when `since` has aged out of server history: fetch `GET /api/mempool-blocks/{index}` again.
   *
   * Endpoint: `GET /api/mempool-blocks/{index}/diff/{since}`
   *
   * @param {MempoolBlockIndex} index
   * @param {MempoolBlockHash} since - The `hash` of the client's copy, from the block or a previous diff
   * @param {{ signal?: AbortSignal, onValue?: (value: MempoolBlockDiff) => void, cache?: boolean, memCache?: boolean }} [options]
   * @returns {Promise<MempoolBlockDiff>}
   */
  async getMempoolBlockDiff(index, since, { signal, onValue, cache, memCache } = {}) {
    const path = `/api/mempool-blocks/${encodeURIComponent(index)}/diff/${encodeURIComponent(since)}`;
    return this.getJson(path, { signal, onValue, cache, memCache });
  }

  /**
   * Txid by index
   *
   * Retrieve the transaction ID (txid) at a given global transaction index. Returns the txid as plain text.
   *
   * Endpoint: `GET /api/tx-index/{index}`
   *
   * @param {TxIndex} index - Global transaction index across the chain
   * @param {{ signal?: AbortSignal, onValue?: (value: Txid) => void, cache?: boolean, memCache?: boolean }} [options]
   * @returns {Promise<Txid>}
   */
  async getTxidByIndex(index, { signal, onValue, cache, memCache } = {}) {
    const path = `/api/tx-index/${encodeURIComponent(index)}`;
    return this.getText(path, { signal, onValue, cache, memCache });
  }

  /**
   * CPFP info
   *
   * Returns ancestors and descendants for a CPFP (Child Pays For Parent) transaction, including the effective fee rate of the package. A mempool transaction answers its live package; a confirmed one its package among its block's transactions; an unknown transaction, or a confirmed one without relatives in its block, `{"ancestors":[]}`.
   *
   * *[Mempool.space docs](https://mempool.space/docs/api/rest#get-children-pay-for-parent)*
   *
   * Endpoint: `GET /api/v1/cpfp/{txid}`
   *
   * @param {Txid} txid
   * @param {{ signal?: AbortSignal, onValue?: (value: Cpfp) => void, cache?: boolean, memCache?: boolean }} [options]
   * @returns {Promise<Cpfp>}
   */
  async getCpfp(txid, { signal, onValue, cache, memCache } = {}) {
    const path = `/api/v1/cpfp/${encodeURIComponent(txid)}`;
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
    const path = `/api/v1/tx/${encodeURIComponent(txid)}/rbf`;
    return this.getJson(path, { signal, onValue, cache, memCache });
  }

  /**
   * Transaction information
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
    const path = `/api/tx/${encodeURIComponent(txid)}`;
    return this.getJson(path, { signal, onValue, cache, memCache });
  }

  /**
   * Transaction hex
   *
   * Hex-encoded serialized transaction.
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
    const path = `/api/tx/${encodeURIComponent(txid)}/hex`;
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
    const path = `/api/tx/${encodeURIComponent(txid)}/merkleblock-proof`;
    return this.getText(path, { signal, onValue, cache, memCache });
  }

  /**
   * Transaction merkle proof
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
    const path = `/api/tx/${encodeURIComponent(txid)}/merkle-proof`;
    return this.getJson(path, { signal, onValue, cache, memCache });
  }

  /**
   * Output spend status
   *
   * Whether the output has been spent and, if so, the spending transaction details. An output no known transaction holds is unspent.
   *
   * *[Mempool.space docs](https://mempool.space/docs/api/rest#get-transaction-outspend)*
   *
   * Endpoint: `GET /api/tx/{txid}/outspend/{vout}`
   *
   * @param {Txid} txid
   * @param {Vout} vout - Output index
   * @param {{ signal?: AbortSignal, onValue?: (value: TxOutspend) => void, cache?: boolean, memCache?: boolean }} [options]
   * @returns {Promise<TxOutspend>}
   */
  async getTxOutspend(txid, vout, { signal, onValue, cache, memCache } = {}) {
    const path = `/api/tx/${encodeURIComponent(txid)}/outspend/${encodeURIComponent(vout)}`;
    return this.getJson(path, { signal, onValue, cache, memCache });
  }

  /**
   * All output spend statuses
   *
   * Spend statuses in output-index order.
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
    const path = `/api/tx/${encodeURIComponent(txid)}/outspends`;
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
    const path = `/api/tx/${encodeURIComponent(txid)}/raw`;
    return this.getBytes(path, { signal, onValue, cache, memCache });
  }

  /**
   * Transaction status
   *
   * Retrieve the confirmation status of a transaction. Returns whether the transaction is confirmed and, if so, the block height, hash, and timestamp. A transaction not in the chain, pending or unknown, is `{"confirmed":false}`.
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
    const path = `/api/tx/${encodeURIComponent(txid)}/status`;
    return this.getJson(path, { signal, onValue, cache, memCache });
  }

  /**
   * Transaction first-seen times
   *
   * First-seen UNIX timestamps in seconds, in request order, of transactions in the mempool or that recently left it other than by replacement (mined ones included); 0 otherwise.
   *
   * *[Mempool.space docs](https://mempool.space/docs/api/rest#get-transaction-times)*
   *
   * Endpoint: `GET /api/v1/transaction-times`
   *
   * @param {Txid[]} txids - Transaction IDs to look up (max 100 per request).
   * @param {{ signal?: AbortSignal, onValue?: (value: Timestamp[]) => void, cache?: boolean, memCache?: boolean }} [options]
   * @returns {Promise<Timestamp[]>}
   */
  async getTransactionTimes(txids, { signal, onValue, cache, memCache } = {}) {
    const params = new URLSearchParams();
    for (const _v of txids) params.append('txId[]', String(_v));
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
   * @param {string} body
   * @param {{ signal?: AbortSignal }} [options]
   * @returns {Promise<Txid>}
   */
  async postTx(body, { signal } = {}) {
    const path = `/api/tx`;
    return this.postText(path, body, { signal });
  }

  /**
   * Latest UTXO set
   *
   * The UTXO set after the latest published block, in the layout of `GET /api/utxo-set/{point}`.
   *
   * Endpoint: `GET /api/utxo-set`
   * @param {{ signal?: AbortSignal, onValue?: (value: UtxoSet) => void, cache?: boolean, memCache?: boolean }} [options]
   * @returns {Promise<UtxoSet>}
   */
  async getUtxoSetLatest({ signal, onValue, cache, memCache } = {}) {
    const path = `/api/utxo-set`;
    return this.getJson(path, { signal, onValue, cache, memCache });
  }

  /**
   * UTXO set at block height or date
   *
   * The UTXO set after a block (`840000`) or after the last published block of a UTC day (`YYYY-MM-DD`), by creation height. Returns `{ height, hash, date, count, supply, origins }`; entry `i` of `origins.count` and `origins.supply` (BTC) is what remains unspent of block `i`'s outputs.
   *
   * Endpoint: `GET /api/utxo-set/{point}`
   *
   * @param {HeightOrDate} point - Confirmed block height (`840000`) or UTC calendar day (`YYYY-MM-DD`)
   * @param {{ signal?: AbortSignal, onValue?: (value: UtxoSet) => void, cache?: boolean, memCache?: boolean }} [options]
   * @returns {Promise<UtxoSet>}
   */
  async getUtxoSet(point, { signal, onValue, cache, memCache } = {}) {
    const path = `/api/utxo-set/${encodeURIComponent(point)}`;
    return this.getJson(path, { signal, onValue, cache, memCache });
  }

  /**
   * UTXO set changes of a block or date
   *
   * What a block (`840000`) or a UTC day's published blocks (`YYYY-MM-DD`) changed in the UTXO set. Returns `{ first, last, hash, date, created, spent }`: `created` has one row per block, `spent` one row per creation height, each as columnar `height`, `count` and `supply` (BTC). The set after block `first - 1` plus `created` minus `spent` is the set after block `last`.
   *
   * Endpoint: `GET /api/utxo-set/{point}/diff`
   *
   * @param {HeightOrDate} point - Confirmed block height (`840000`) or UTC calendar day (`YYYY-MM-DD`)
   * @param {{ signal?: AbortSignal, onValue?: (value: UtxoSetDiff) => void, cache?: boolean, memCache?: boolean }} [options]
   * @returns {Promise<UtxoSetDiff>}
   */
  async getUtxoSetDiff(point, { signal, onValue, cache, memCache } = {}) {
    const path = `/api/utxo-set/${encodeURIComponent(point)}/diff`;
    return this.getJson(path, { signal, onValue, cache, memCache });
  }

  /**
   * Live BTC/USD price
   *
   * Current BTC/USD price in dollars, derived from on-chain round-dollar output patterns in the oracle's recent-block window, with the mempool's outputs blended in when available. Confirmed per-height history is available at `GET /api/series/price/height`.
   *
   * Endpoint: `GET /api/oracle/price`
   * @param {{ signal?: AbortSignal, onValue?: (value: Dollars) => void, cache?: boolean, memCache?: boolean }} [options]
   * @returns {Promise<Dollars>}
   */
  async getLivePrice({ signal, onValue, cache, memCache } = {}) {
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
   * @param {HeightOrDate} point - Confirmed block height (`840000`) or UTC calendar day (`YYYY-MM-DD`)
   * @param {{ signal?: AbortSignal, onValue?: (value: number[]) => void, cache?: boolean, memCache?: boolean }} [options]
   * @returns {Promise<number[]>}
   */
  async getOracleHistogramPayments(point, { signal, onValue, cache, memCache } = {}) {
    const path = `/api/oracle/histogram/payments/${encodeURIComponent(point)}`;
    return this.getJson(path, { signal, onValue, cache, memCache });
  }

  /**
   * Live output value histogram
   *
   * Live unfiltered output value histogram for the complete published mempool. Nonzero outputs within the histogram's bin range are counted by value on the oracle log scale; no oracle payment filters are applied. A flat array of log-scale bins, all zero when no mempool is configured.
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
   * Unfiltered output value histogram for a confirmed point. A block height (`840000`) counts that block's nonzero outputs within the histogram's bin range, coinbase included, by value on the oracle log scale; a calendar date (`YYYY-MM-DD`) sums every block that day. A flat array of log-scale bins.
   *
   * Endpoint: `GET /api/oracle/histogram/outputs/{point}`
   *
   * @param {HeightOrDate} point - Confirmed block height (`840000`) or UTC calendar day (`YYYY-MM-DD`)
   * @param {{ signal?: AbortSignal, onValue?: (value: number[]) => void, cache?: boolean, memCache?: boolean }} [options]
   * @returns {Promise<number[]>}
   */
  async getOracleHistogramOutputs(point, { signal, onValue, cache, memCache } = {}) {
    const path = `/api/oracle/histogram/outputs/${encodeURIComponent(point)}`;
    return this.getJson(path, { signal, onValue, cache, memCache });
  }

  /**
   * OpenAPI specification
   *
   * Full OpenAPI 3.1 specification for this API. Build tools from this document.
   *
   * Endpoint: `GET /openapi.json`
   * @param {{ signal?: AbortSignal, onValue?: (value: *) => void, cache?: boolean, memCache?: boolean }} [options]
   * @returns {Promise<*>}
   */
  async getOpenapi({ signal, onValue, cache, memCache } = {}) {
    const path = `/openapi.json`;
    return this.getJson(path, { signal, onValue, cache, memCache });
  }

  /**
   * Compact API summary
   *
   * Compact custom summary of the OpenAPI specification for LLM reading: types instead of schemas, `returns` instead of responses, no examples; parameters keep their enum, default and bounds. Not valid OpenAPI: build tools from `GET /openapi.json`.
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
