// Auto-generated Bitview JavaScript client
// Do not edit manually

/**
 * How a series' values combine over time.
 *
 * @typedef {(("flow")|("level")|("cumulative"))} Accumulation
 */
/**
 * Bitcoin address, or the hex public key of a P2PK output.
 *
 * @typedef {string} Address
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
 * @property {?TypeIndex} typeIndex - Index of this address within its type on the blockchain, null for an address without
 *   confirmed activity
 * @property {?Dollars} realizedPrice - Realized price (average cost basis) in USD
 */
/**
 * @typedef {Object} AddressHashPrefixMatches
 * @property {AddressOutputType} addressType
 * @property {string} prefix - The requested prefix, lowercased.
 * @property {boolean} truncated
 * @property {Address[]} addresses
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
 * The output type of an address: any output type but `p2ms`, `op_return`, `empty_script` and `unknown_script`.
 *
 * @typedef {("p2pk65"|"p2pk33"|"p2pkh"|"p2sh"|"p2wpkh"|"p2wsh"|"p2tr"|"p2a")} AddressOutputType
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
 * @typedef {Object} AddressValidation
 * @property {boolean} isvalid
 * @property {?Address=} address - The validated address
 * @property {?Hex=} scriptPubKey - The scriptPubKey in hex
 * @property {?boolean=} isscript - Whether the address is P2SH, P2WSH or P2TR
 * @property {?boolean=} iswitness
 * @property {?number=} witnessVersion - Witness version (0 for P2WPKH/P2WSH, 1 for P2TR, 2 to 16 for future versions; none for P2A)
 * @property {?Hex=} witnessProgram - Witness program in hex
 * @property {?number[]=} errorLocations - Positions of the characters at fault in an invalid address (Bitcoin Core's), possibly none;
 *   omitted for valid ones
 * @property {?string=} error - Error message for invalid addresses
 */
/**
 * This server's permanent live observation of a block expectation.
 *
 * @typedef {Object} AuditDetail
 * @property {number} version
 * @property {Height} height
 * @property {BlockHash} id
 * @property {Timestamp} timestamp
 * @property {number} matchRate
 * @property {Sats} expectedFees
 * @property {Weight} expectedWeight
 * @property {Txid[]} unseenTxs
 * @property {Txid[]} missingTxs
 * @property {Txid[]} addedTxs
 * @property {Txid[]} prioritizedTxs
 * @property {Txid[]} freshTxs
 * @property {Txid[]} sigopTxs
 * @property {Txid[]} fullrbfTxs
 * @property {Txid[]} acceleratedTxs
 */
/**
 * @typedef {Object} AuditScore
 * @property {BlockHash} hash
 * @property {number} matchRate
 * @property {Sats} expectedFees
 * @property {Weight} expectedWeight
 */
/**
 * Mempool v1 uses the JSON string `"null"` for an absent score.
 *
 * @typedef {(AuditScore|string)} AuditScoreResponse
 */
/**
 * This server's permanent live observation of a block expectation.
 *
 * @typedef {Object} AuditSummary
 * @property {number} version
 * @property {Height} height
 * @property {BlockHash} id
 * @property {Timestamp} timestamp
 * @property {number} matchRate
 * @property {Sats} expectedFees
 * @property {Weight} expectedWeight
 */
/**
 * An amount in bitcoin, as a decimal number (1 BTC = 100,000,000 satoshis).
 *
 * @typedef {number} Bitcoin
 */
/**
 * A block's hash.
 *
 * @typedef {Object} BlindbitBlockHash
 * @property {BlockHash} blockHash
 */
/**
 * The last indexed block's height.
 *
 * @typedef {Object} BlindbitBlockHeight
 * @property {Height} blockHeight
 */
/**
 * A BIP158 Golomb-coded set (the basic filters' parameters, keyed by the block hash).
 *
 * @typedef {Object} BlindbitFilter
 * @property {BlockHash} blockHash
 * @property {Height} blockHeight
 * @property {Hex} data - The filter: the element count, then the coded deltas.
 * @property {number} filterType - Always 4, BlindBit's Taproot-only filters.
 */
/**
 * Which of a block's two filters.
 *
 * @typedef {(("new-utxos")|("spent"))} BlindbitFilterKind
 */
/**
 * What the server serves.
 *
 * @typedef {Object} BlindbitInfo
 * @property {string} network - Bitcoin Core's chain name: `main`.
 * @property {Height} height - The last indexed block.
 * @property {boolean} tweaksOnly - Whether only tweaks are served: `false`, filters and UTXOs are too.
 * @property {boolean} tweaksFullBasic - Whether the full tweak index is served without dust limits: `false`, transactions whose
 *   Taproot outputs all hold less than 600 sats aren't held.
 * @property {boolean} tweaksFullWithDustFilter - Whether the full tweak index takes a dust limit: `true`.
 * @property {boolean} tweaksCutThroughWithDustFilter - Whether cut-through tweaks (of transactions with an unspent Taproot output) take a dust
 *   limit: `true`.
 */
/**
 * A block's spends of Taproot outputs.
 *
 * @typedef {Object} BlindbitSpentIndex
 * @property {BlockHash} blockHash
 * @property {SpentOutputHash[]} data
 */
/**
 * A Taproot output of an eligible transaction.
 *
 * @typedef {Object} BlindbitUtxo
 * @property {Txid} txid
 * @property {Vout} vout
 * @property {Sats} value
 * @property {Hex} scriptpubkey - Its script, `OP_1` and a 32-byte push of the output key.
 * @property {Height} blockHeight
 * @property {BlockHash} blockHash
 * @property {Timestamp} timestamp - The block's header time.
 * @property {boolean} spent - Whether a published block spends it.
 */
/**
 * Extended block data matching mempool.space /api/v1/blocks extras
 *
 * @typedef {Object} BlockExtras
 * @property {Sats} totalFees - Total fees in satoshis
 * @property {FeeRate} medianFee - Median effective (CPFP) fee rate in sat/vB, by vsize: the rate half the block's space
 *   paid at most (Bitcoin Core's `getblockstats` rule; mempool.space averages the middle
 *   0.25% of a 4M WU block instead)
 * @property {FeeRate[]} feeRange - Effective (CPFP) fee rate range in sat/vB: [min, 10%, 25%, 50%, 75%, 90%, max], the
 *   percentiles of block space (vsize-weighted, as `medianFee`; mempool.space ranks
 *   transactions by count), min and max the lower/higher of the 1st/99th percentile and the
 *   extreme rate in the block's last/first 2% of vsize
 * @property {Sats} reward - Total block reward (subsidy + fees) in satoshis
 * @property {?BlockPool} pool
 * @property {Sats} avgFee - Mean fee per non-coinbase transaction in satoshis, rounded down (0 without any)
 * @property {FeeRate} avgFeeRate - Average fee rate in sat/vB
 * @property {Hex} coinbaseRaw - Raw coinbase transaction scriptsig as hex
 * @property {?Address} coinbaseAddress - Primary coinbase output address
 * @property {Address[]} coinbaseAddresses - All coinbase output addresses
 * @property {string} coinbaseSignature - Coinbase output script in ASM format
 * @property {string} coinbaseSignatureAscii - Coinbase scriptsig with each byte read as the Unicode code point of the same value
 * @property {number} avgTxSize - Non-coinbase transaction bytes over the transaction count (coinbase included), rounded to
 *   2 decimals
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
 *   inputs, unspendable outputs (OP_RETURN) included, unlike `utxoSetSize`
 * @property {number} utxoSetSize - Total spendable UTXO set size at this height (excludes OP_RETURN and other unspendable outputs)
 * @property {Sats} totalInputAmt - Total input amount in satoshis
 * @property {number} virtualSize - Virtual size in vbytes: weight / 4, unrounded
 * @property {?boolean} coinbaseBip54 - Whether the coinbase follows BIP54 (its lock time is the height minus one and its input's
 *   sequence isn't final), from timestamp 1,771,507,776 on; null before
 * @property {?Timestamp} firstSeen - Timestamp when the block was first seen (always null, not yet supported)
 * @property {BlockOrphan[]} orphans - Stale blocks at this height the server's node knows, each server's own
 * @property {?number} matchRate - Match rate of this server's live pre-block observation; null for unaudited blocks.
 * @property {?Sats} expectedFees - Base fees of this server's expected block, excluding coinbase.
 * @property {?Weight} expectedWeight - Raw transaction weight of the expected block, excluding header and coinbase reserve.
 * @property {?number} similarity - Matched expected weight plus actual coinbase weight, divided by expected plus coinbase.
 * @property {?Dollars} price - USD price at block height
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
 * @property {?Dollars} USD - Mean BTC/USD spot price over the bucket's blocks (the price, not the fees in USD)
 */
/**
 * Double-SHA256 block-header hash, serialized in Bitcoin's conventional
 * hexadecimal byte order.
 *
 * @typedef {string} BlockHash
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
 * @property {?BlockHash=} previousblockhash - Hash of the previous block (null for the genesis block)
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
 * @property {?BlockHash=} previousblockhash - Hash of the previous block (null for the genesis block)
 * @property {Timestamp} mediantime - Median time of the last 11 blocks
 * @property {boolean=} stale - Whether this block has been replaced by a longer chain
 * @property {BlockExtras} extras
 */
/**
 * A stale block at a height of the best chain: a block of a fork the node knows (bitcoind's
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
 * How far the node validated a fork (bitcoind's `getchaintips` statuses for the forks it holds
 * as valid).
 *
 * @typedef {(("valid-headers")|("valid-fork"))} BlockOrphanStatus
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
 * @property {?Dollars} USD - Mean BTC/USD spot price over the bucket's blocks (the price, not the rewards in USD)
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
 * @property {?Height=} height - Block height (null unless in the best chain)
 * @property {?BlockHash=} nextBest - Hash of the next block in the best chain (null if tip)
 */
/**
 * The next block's template in BIP 22/23 form, as Bitcoin Core's `getblocktemplate` answers
 * (`rules: ["segwit"]`): what pools and mining software read. Complete once a miner adds a
 * coinbase paying `coinbasevalue` and carrying `default_witness_commitment`. Returned by
 * `GET /api/block-template`.
 *
 * @typedef {Object} BlockTemplate
 * @property {number} version - The block version: BIP 9 version bits, no deployment signaled.
 * @property {string[]} rules - The rules the template follows: `!` marks one a client must understand.
 * @property {{ [key: string]: number }} vbavailable - BIP 9 deployments the client may signal: none.
 * @property {number} vbrequired - Version bits the client must set: none.
 * @property {BlockHash} previousblockhash
 * @property {BlockTemplateTransaction[]} transactions - The transactions, every parent before its children.
 * @property {{ [key: string]: string }} coinbaseaux - Data to put in the coinbase's script: none.
 * @property {Sats} coinbasevalue - What the coinbase may pay: the subsidy at `height` plus every transaction's fee (sats).
 * @property {string} target - The proof-of-work target, as 64 hex digits.
 * @property {Timestamp} mintime - The earliest valid `curtime`: the median time past plus one second.
 * @property {string[]} mutable - What the client may change: the time, the transactions, the previous block.
 * @property {string} noncerange
 * @property {number} sigoplimit - Consensus limits for the whole block, coinbase included.
 * @property {number} sizelimit
 * @property {number} weightlimit
 * @property {Timestamp} curtime - The block time to use: now, or `mintime` if later.
 * @property {string} bits - The compact target, as 8 hex digits.
 * @property {Height} height
 * @property {string} defaultWitnessCommitment - The coinbase output committing to the transactions' witnesses (BIP 141), as script hex.
 */
/**
 * One transaction of a [`BlockTemplate`].
 *
 * @typedef {Object} BlockTemplateTransaction
 * @property {string} data - The transaction serialized with its witness, as hex.
 * @property {Txid} txid
 * @property {string} hash - The witness transaction id.
 * @property {number[]} depends - The 1-based positions in `transactions` of the ones it spends.
 * @property {Sats} fee - Its fee (sats).
 * @property {SigOps} sigops - Its signature-operation cost.
 * @property {Weight} weight
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
 *   (sat/vB).
 * @property {CpfpCluster} cluster - The transaction's relatives in its block.
 */
/**
 * A number of things.
 *
 * @typedef {number} Count
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
 * @property {?CpfpEntry=} bestDescendant - Best (highest fee rate) descendant, if any.
 * @property {CpfpEntry[]} descendants
 * @property {FeeRate} effectiveFeePerVsize - Effective fee rate considering CPFP relationships (sat/vB).
 *   This is the seed's chunk feerate after lift-merging, i.e. the
 *   rate Core/mempool.space would surface for this tx.
 * @property {SigOps} sigops - BIP-141 sigop cost for the seed tx (witness sigops count as 1,
 *   legacy and P2SH-redeem sigops count as 4), counted from its body:
 *   at most a block's sigop budget.
 * @property {Sats} fee - Transaction fee (sats).
 * @property {VSize} vsize - Virtual size of the seed tx (vbytes).
 * @property {VSize} adjustedVsize - Policy-adjusted virtual size: `max(vsize, sigops * 5)`.
 * @property {?CpfpCluster=} cluster - Cluster the seed belongs to; omitted when the seed has no ancestors and no descendants
 *   (matches mempool.space).
 */
/**
 * A quote currency prices are served in: USD only.
 *
 * @typedef {("USD")} Currency
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
 * @property {number} estimatedRetargetDate - Estimated timestamp of next retarget (milliseconds): the tip block's time plus
 *   `remainingTime`, so it moves with blocks, not with the clock. It is early by the tip's
 *   age, and with a block or two left in the epoch it can be before now: for a date counted
 *   from now, as mempool.space gives, add `remainingTime` to your own clock
 * @property {number} remainingBlocks - Blocks remaining until retarget
 * @property {number} remainingTime - Estimated time until retarget at `adjustedTimeAvg` (milliseconds)
 * @property {number} previousRetarget - Previous difficulty adjustment (%)
 * @property {Timestamp} previousTime - Timestamp of most recent retarget (seconds)
 * @property {Height} nextRetargetHeight
 * @property {number} timeAvg - Average block time in current epoch (milliseconds)
 * @property {number} adjustedTimeAvg - Block time the projection uses (milliseconds): `timeAvg`, except in an epoch's first 503
 *   blocks, where it spans the last 503 blocks, those before the retarget scaled by it
 * @property {number} timeOffset - Testnet's offset for its 20-minute minimum-difficulty rule (milliseconds): always 0 on
 *   mainnet, as mempool.space
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
 * Index of a difficulty epoch on the active chain.
 *
 * @typedef {number} DifficultyEpoch
 */
/**
 * @typedef {Object} DiskUsage
 * @property {string} brk - Human-readable brk data size (e.g., "48.8 GiB")
 * @property {number} brkBytes - brk data size in bytes: the indexer stores' table files plus the allocated bytes of the
 *   other files, the server's logs left out
 * @property {string} bitcoin - Human-readable size of the node's block and undo files
 * @property {number} bitcoinBytes - The node's own estimate of its block and undo files' bytes (`size_on_disk`)
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
 * @property {CpfpEntry[]} ancestors - Always empty.
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
/** @typedef {("not_found"|"invalid_address"|"invalid_network"|"unsupported_type"|"no_series"|"series_unsupported_index"|"weight_exceeded"|"too_many_utxos"|"limit_exceeded"|"unknown_after_txid"|"unknown_address"|"unknown_txid"|"unknown_pool"|"out_of_range"|"unindexable_date"|"no_data"|"series_not_found"|"mempool_not_available"|"state_updating"|"internal_error"|"bad_request"|"overloaded"|"timeout"|"method_not_allowed")} ErrorCode */
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
 * A number of things expected from chance: a sum of probabilities.
 *
 * @typedef {number} ExpectedCount
 */
/**
 * Fee rate stored in milli-sat/vB and exposed as sat/vB.
 *
 * @typedef {number} FeeRate
 */
/**
 * Yes or no: 1 yes, 0 no; undefined (null) where the question doesn't apply yet.
 *
 * @typedef {number} Flag
 */
/**
 * A floating-point value without a specific unit.
 *
 * @typedef {number} Float
 */
/** @typedef {("json"|"csv")} Format */
/**
 * BlindBit's transaction submission body.
 *
 * @typedef {Object} ForwardTx
 * @property {string} data - The raw transaction, hexadecimal.
 */
/**
 * Index of a subsidy halving epoch on the active chain.
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
 * @property {Height} tipHeight - Height of the Bitcoin node's chain tip, as the server last saw it (it looks for a new tip
 *   every second)
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
/** @typedef {("error"|"warn"|"info"|"debug"|"trace")} LogLevel */
/**
 * One line of the server's log.
 *
 * @typedef {Object} LogLine
 * @property {number} seq - Its place in this process's log, from 0 with no gaps.
 * @property {number} unixMs - When it was logged, in Unix milliseconds.
 * @property {LogLevel} level
 * @property {string} text - What the log files show after the level: a request's status, method, path and duration, or a message and its fields.
 */
/**
 * The server's last log lines, oldest first.
 *
 * @typedef {Object} Logs
 * @property {number} startedUnixMs - When this server process began its log, in Unix milliseconds: another than the last answer's shows the
 *   server restarted.
 * @property {LogLine[]} lines
 * @property {string} next - Where this answer ends: sent back as `after`, the lines since. `{started_unix_ms}-{seq}`, the process and the
 *   `seq` of the next line it will log; to pass back as it is, not to read.
 */
/**
 * Block info in a mempool.space like format for fee estimation.
 *
 * @typedef {Object} MempoolBlock
 * @property {number} blockSize - Sum of the transactions' serialized sizes in bytes (witness + non-witness), without header
 *   or coinbase.
 * @property {number} blockVSize - The transactions' weight over 4, unrounded (mempool.space's): the block's virtual size
 * @property {number} nTx - Number of transactions in the projected block
 * @property {Sats} totalFees - Total fees in satoshis
 * @property {FeeRate} medianFee - Median fee rate in sat/vB
 * @property {FeeRate[]} feeRange - Vsize-weighted percentiles of the transactions' CPFP chunk fee rates, in sat/vB:
 *   [0, 10, 25, 50, 75, 90, 100] for the first block, [5, 10, 25, 50, 75, 90, 95] for the
 *   later projected ones
 */
/**
 * A projected mempool block with its transactions in block order: every block, block 0 the next,
 * packed as Core's block assembler packs, ranked by signed modified fees within Core's default
 * budgets, with paid-fee statistics. Returned by `GET /api/mempool-blocks/{index}`.
 *
 * @typedef {Object} MempoolBlockContent
 * @property {MempoolBlockHash} hash - Pass as `since` to `GET /api/mempool-blocks/{index}/diff/{since}` to follow changes.
 * @property {MempoolBlock} stats
 * @property {MempoolTxSummary[]} transactions - The transactions in block order, each in its light shape: the whole body is at
 *   `GET /api/tx/{txid}`.
 */
/**
 * A projected mempool block's changes since a prior copy identified by `since`. Returned by
 * `GET /api/mempool-blocks/{index}/diff/{since}`.
 *
 * `order` alone is the new block, and a `New` entry replaces what was known of its txid (a
 * transaction whose body changed comes back as `New`, not in `removed`). `removed` lists the
 * txids that left the block since `since`: evict what was cached for them. When the server
 * doesn't hold the copy at `since` (aged out, another server's, or from before a restart), the
 * diff is from no copy: `since` is `"0"`, every entry is `New` and `removed` is empty, so replace
 * the copy and drop what was cached for txids not in `order`.
 *
 * @typedef {Object} MempoolBlockDiff
 * @property {MempoolBlockHash} hash - The block's current hash. Use as `since` on the next diff call.
 * @property {MempoolBlockHash} since - The prior hash the diff was computed against: the `since` asked for, or `"0"` when the
 *   server doesn't hold that copy (then `order` replaces it).
 * @property {MempoolBlock} stats - Stats of the new block; they can change while every transaction is retained.
 * @property {MempoolBlockDiffEntry[]} order - The new block in order. Each entry is either a run of the prior copy's transactions,
 *   `[start, count]`, or a transaction in its light shape.
 * @property {Txid[]} removed - Txids that left the projected block since `since` (confirmed, evicted, replaced, or moved
 *   to another block).
 */
/**
 * One stretch of the new block in a `MempoolBlockDiff`.
 *
 * Untagged on the wire so JSON type disambiguates the variants:
 * - `Retained(run)` serializes as `[start, count]`: the prior block's transactions
 *   `start..start + count` (which the client cached at `since`), kept in order.
 * - `New(tx)` serializes as a transaction in its light shape: one that was new or changed since
 *   the prior block and takes this position.
 *
 * @typedef {(RetainedRun|MempoolTxSummary)} MempoolBlockDiffEntry
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
 * next block.
 *
 * @typedef {number} MempoolBlockIndex
 */
/**
 * Mempool statistics with incrementally maintained fee histogram.
 *
 * @typedef {Object} MempoolInfo
 * @property {number} count - Number of transactions in the mempool
 * @property {VSize} vsize - Total virtual size of all transactions in the mempool (vbytes), sigop-adjusted as Bitcoin
 *   Core's `getmempoolinfo` `bytes` and every fee rate measure it
 * @property {Sats} totalFee - Total fees of all transactions in the mempool (satoshis)
 * @property {number[][]} feeHistogram - Fee histogram as Esplora bins it: `[[fee_rate, vsize], ...]` by descending fee rate, each
 *   entry the vsize of the transactions paying from its fee rate up to the previous entry's,
 *   a bin closing at the first fee rate change past 50,000 vB
 */
/**
 * An unconfirmed transaction in esplora's light shape, as `/api/mempool/recent` and the projected
 * mempool blocks list them: the whole body is at `GET /api/tx/{txid}`.
 *
 * @typedef {Object} MempoolTxSummary
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
 * A Compass mood: 0 Optimism, 1 Calm, 2 Anxiety, 3 Hope, 4 Doubt, 5 Worry, 6 Relief, 7 Apathy,
 * 8 Fear; undefined (null) before the Compass has its inputs.
 *
 * @typedef {number} Mood
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
 * @typedef {("runes"|"witness_commitment"|"veriblock"|"omni"|"stacks"|"blockstack"|"colu"|"open_assets"|"komodo"|"coinspark"|"poet"|"docproof"|"factom"|"eternity_wall"|"memo"|"bitproof"|"ascribe"|"stampery"|"bip47"|"bare_hash"|"text"|"empty"|"unknown")} OpReturnKind
 */
/**
 * Index of an OP_RETURN output.
 *
 * @typedef {TypeIndex} OpReturnOutputIndex
 */
/**
 * One of the stencil's round-dollar amounts.
 *
 * @typedef {Object} OracleAmount
 * @property {number} dollars - The amount, in dollars.
 * @property {number} offset - Its bin minus $100's: `round(log10(dollars / 100) * 200)`.
 */
/**
 * The oracle at one block: what it counted (`histogram`, `outputs`), where it looked (`first_bin`,
 * `scores`), what it found (`bin`, `price`) and how well its stencil fits around that (`fit`,
 * `stencil`).
 *
 * Bins are the oracle's log scale of amounts in sats, 200 a decade: amount `sats` is in bin
 * `round(log10(sats) * 200)`, bin 0 at 1 sat to bin 2399 just under 10^12. Read as where $100
 * lands, a bin is a price: `10^(10 - bin / 200)` dollars, higher bins lower prices.
 *
 * @typedef {Object} OracleBlock
 * @property {Height} height - The block's height; live with the mempool, the next block's.
 * @property {BlockHash} hash - The confirmed block it was read at: this block's hash; live with the mempool, the tip's
 *   (block `height - 1`), the one the mempool is blended onto.
 * @property {boolean} mempool - Whether it's the live reading with the mempool blended in as the next block (`outputs` the
 *   mempool's), rather than a confirmed block's.
 * @property {Dollars} price - The price it found, in dollars: `bin` as a price, to the cent.
 * @property {number} bin - The bin it picked, refined between bins by a parabola through its score and its neighbours'
 *   when it's their local maximum.
 * @property {number} firstBin - The search window's first candidate bin: 12 below the block before's answer, rounded, 24
 *   candidates in all.
 * @property {number[]} scores - Each candidate's score, from `first_bin` on, unitless: for each of the 19 round-dollar
 *   amounts ($1, 2, 3, 5, 10, 15, 20, 25, 30, 50, 100, 150, 200, 300, 500, 1k, 2k, 5k, 10k),
 *   the histogram's count where that amount lands if $100 is at the candidate, over that
 *   amount's peak in the window, summed. The pick is the highest of the candidates with counts on
 *   at least two amounts; without one, or with all zero (no payments in the window), the oracle
 *   kept the block before's answer.
 * @property {number[]} histogram - The smoothed histogram it scored, 2,400 bins: the block's payments and the 11 blocks'
 *   before it (live, the mempool's as the next block's), each block's counts weighted
 *   `(2/7) * (5/7)^age`, the newest at age 0.
 * @property {OracleOutputs} outputs - The block's own outputs (live, the mempool's) as the oracle's filter judged them.
 * @property {OracleFit} fit - How well the stencil fits around the answer: every bin within 120 of `bin` (rounded),
 *   scored as `scores` scores a candidate, with each amount's peak taken over this wider
 *   window. It peaks at the price and echoes where other amounts pass for $100, as at double
 *   and half.
 * @property {OracleAmount[]} stencil - The stencil it scores with: each round-dollar amount and where it lands from $100's bin.
 */
/**
 * One block's payments read as round dollars, over the oracle's search window for it.
 *
 * @typedef {Object} OracleBlockPayments
 * @property {Height} height
 * @property {number} firstBucket - The first bucket's index: the window's first bin's lower edge (half a bin under it) over
 *   `step`.
 * @property {number[]} counts - The readings in each bucket from `first_bucket` on: bucket `first_bucket + k` at index `k`.
 */
/**
 * Scores over a run of bins.
 *
 * @typedef {Object} OracleFit
 * @property {number} firstBin - The first bin scored.
 * @property {number[]} scores - Each bin's score, from `first_bin` on, unitless.
 */
/**
 * A block's outputs as the oracle's filter judged them: each skipped one under the first rule
 * that skips it, in the filter's order, and the payments it kept. They add up to the block's
 * outputs.
 *
 * @typedef {Object} OracleOutputs
 * @property {number} coinbase - The coinbase's: new coins, not payments.
 * @property {number} fanOut - Those of transactions with more outputs than the fan-out cap (250; 100 before height
 *   630,000): batch payouts.
 * @property {number} opReturn - Those of transactions carrying an OP_RETURN output: data, not payments.
 * @property {number} taproot - Taproot outputs: mostly protocol traffic.
 * @property {number} dust - Under 1,000 sats.
 * @property {number} roundBtc - Round amounts in bitcoin (within 0.1% of 1, 2, 3, 5 or 6 times a power of ten sats).
 * @property {number} payments - The rest: the payments it counted.
 */
/**
 * 144 blocks' payments, each read as each of the oracle's 19 round-dollar amounts ($1 to
 * $10,000) at its exact amount in sats, counted by the price each reading implies in buckets a
 * tenth of the oracle's bins.
 *
 * A reading of `sats` as `amount` dollars implies $100 lands at bin
 * `(log10(sats) + 2 - log10(amount)) * 200`; bucket `b` holds the readings in bins
 * `[b * step, (b + 1) * step)`, prices from `10^(10 - (b + 1) * step / 200)` up to
 * `10^(10 - b * step / 200)` dollars.
 *
 * @typedef {Object} OraclePayments
 * @property {BlockHash} hash - The last block's hash: it commits to every block before it.
 * @property {number} step - A bucket's width in oracle bins: 0.1, a price range of 0.115%.
 * @property {OracleBlockPayments[]} blocks - Each block's, in order.
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
 *   the mempool as it was)
 * @property {string} totalReward - Total reward earned by this pool: coinbase output value of every block it mined (sats),
 *   as a decimal string
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
 * A mining pool's slug, as written in URLs: `GET /api/v1/mining/pools` lists them.
 *
 * @typedef {("unknown"|"blockfills"|"ultimuspool"|"terrapool"|"luxor"|"1thash"|"cloverpool"|"bitfarms"|"huobipool"|"wayicn"|"canoepool"|"btctop"|"bitcoincom"|"175btc"|"gbminers"|"axbt"|"asicminer"|"bitminter"|"bitcoinrussia"|"btcserv"|"simplecoinus"|"btcguild"|"eligius"|"ozcoin"|"eclipsemc"|"maxbtc"|"triplemining"|"coinlab"|"50btc"|"ghashio"|"stminingcorp"|"bitparking"|"mmpool"|"polmine"|"kncminer"|"bitalo"|"f2pool"|"hhtt"|"megabigpower"|"mtred"|"nmcbit"|"yourbtcnet"|"givemecoins"|"braiinspool"|"antpool"|"multicoinco"|"bcpoolio"|"cointerra"|"kanopool"|"solock"|"ckpool"|"nicehash"|"bitclub"|"bitcoinaffiliatenetwork"|"btcc"|"bwpool"|"exxbw"|"bitsolo"|"bitfury"|"21inc"|"digitalbtc"|"8baochi"|"mybtccoinpool"|"tbdice"|"hashpool"|"nexious"|"bravomining"|"hotpool"|"okexpool"|"bcmonster"|"1hash"|"bixin"|"tatmaspool"|"viabtc"|"connectbtc"|"batpool"|"waterhole"|"dcexploration"|"dcex"|"btpool"|"58coin"|"bitcoinindia"|"shawnp0wers"|"phashio"|"rigpool"|"haozhuzhu"|"7pool"|"miningkings"|"hashbx"|"dpool"|"rawpool"|"haominer"|"helix"|"bitcoinukraine"|"poolin"|"secretsuperstar"|"tigerpoolnet"|"sigmapoolcom"|"okpooltop"|"hummerpool"|"tangpool"|"bytepool"|"spiderpool"|"novablock"|"miningcity"|"binancepool"|"minerium"|"lubiancom"|"okkong"|"aaopool"|"emcdpool"|"foundryusa"|"sbicrypto"|"arkpool"|"purebtccom"|"marapool"|"kucoinpool"|"entrustcharitypool"|"okminer"|"titan"|"pegapool"|"btcnuggets"|"cloudhashing"|"digitalxmintsy"|"telco214"|"btcpoolparty"|"multipool"|"transactioncoinmining"|"btcdig"|"trickysbtcpool"|"btcmp"|"eobot"|"unomp"|"patels"|"gogreenlight"|"bitcoinindiapool"|"ekanembtc"|"canoe"|"tiger"|"1m1x"|"zulupool"|"secpool"|"ocean"|"whitepool"|"wiz"|"wk057"|"futurebitapollosolo"|"carbonnegative"|"portlandhodl"|"phoenix"|"neopool"|"maxipool"|"bitfufupool"|"gdpool"|"miningdutch"|"publicpool"|"miningsquared"|"innopolistech"|"btclab"|"parasite"|"redrockpool"|"est3lar"|"braiinssolo"|"solopoolcom"|"noderunners"|"dmnd"|"trustpool")} PoolSlug
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
 *   own audits (always null: it needs the mempool as it was)
 * @property {?string=} avgFeeDelta - Mean relative difference of the pool's fees from the expected template's, a decimal
 *   string at mempool.space, from its own audits (always null: it needs the mempool as it was)
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
 *   it; `minimumFee` when no third projected block exists
 * @property {FeeRate} minimumFee - The node's current mempool minimum fee rate, rounded up to 0.001 sat/vB (at least 0.001)
 */
/**
 * A BTC/USD price at a time: the current price (`/api/v1/prices`) or one historical close
 *
 * @typedef {Object} PricePoint
 * @property {Timestamp} time - Unix timestamp: a historical close's, or for the current price when it was derived (the
 *   mempool publication counted in it, else the tip block's time)
 * @property {?Dollars} USD - BTC/USD price
 */
/**
 * A positional index, a YYYY-MM-DD date, or a UTC timestamp with uppercase T and Z and an optional fraction of 1 to 9 digits. Offsets are refused.
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
 * @property {?ReplacementNode=} replacements
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
 * @property {?boolean=} fullRbf - On a transaction that replaced others: whether one it directly replaced didn't signal
 *   (mempool.space's `tx.fullRbf`). Absent on one that replaced none.
 * @property {?boolean=} mined - `true` when this transaction is confirmed; absent otherwise.
 */
/**
 * Recommended fee rates in whole sat/vB (`/api/v1/fees/recommended`)
 *
 * @typedef {Object} RecommendedFees
 * @property {number} fastestFee - Fee rate for fastest confirmation (next block)
 * @property {number} halfHourFee - Fee rate for confirmation within ~30 minutes (second projected block)
 * @property {number} hourFee - Fee rate for confirmation within ~1 hour (third projected block)
 * @property {number} economyFee - Smoothed, fullness-adjusted third-block fee estimate, bounded by `minimumFee` and twice
 *   it; `minimumFee` when no third projected block exists
 * @property {number} minimumFee - The node's current mempool minimum fee rate, rounded up to a whole sat/vB (at least 1)
 */
/**
 * One node in an RBF replacement tree. The node's `tx` replaced each
 * entry in `replaces`, recursively.
 *
 * @typedef {Object} ReplacementNode
 * @property {RbfTx} tx
 * @property {Timestamp} time - First-seen timestamp, duplicated here to match mempool.space's
 *   on-the-wire shape.
 * @property {boolean} fullRbf - Any predecessor in this subtree was non-signaling.
 * @property {?number=} interval - Seconds between this node's `time` and the successor that
 *   replaced it. Omitted on the root of an RBF response.
 * @property {?boolean=} mined - On a tree's root: `true` when a transaction anywhere in the tree is confirmed
 *   (mempool.space's tree `mined`; the confirmed one is marked in its `tx`); absent otherwise.
 * @property {?boolean=} truncated - On a tree's root: `true` when the tree was cut to its bounds (the newest 63 versions, at
 *   most 4,096 transactions; its `fullRbf` flags then describe what it holds); absent
 *   otherwise. mempool.space serves trees whole.
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
 *   undefined one (e.g. NaN)
 * @property {string} type - Value type (e.g. "Ratio", "Sats", "Cents")
 * @property {?string=} unit - What the value type measures, when documented (e.g. "A duration in days.")
 * @property {Accumulation} accumulation - How values combine over time: a `flow` sums over a window, a `level` is read at a point, and a `cumulative` series is already a running total, its flow the difference.
 */
/**
 * Series leaf with JSON Schema for client generation.
 *
 * @typedef {Object} SeriesLeafWithSchema
 * @property {SeriesName} name
 * @property {string} kind - The value type, by its schema's name (e.g., "Sats", "Ratio").
 * @property {Index[]} indexes
 * @property {Index[]} nullable - Indexes whose values can be null: a missing value (e.g. a period without blocks) or an
 *   undefined one (e.g. NaN).
 * @property {?string=} description - Human-readable metric definition, when documented.
 * @property {Accumulation} accumulation - How its values combine over time: summed over a window, read at a point, or differenced
 *   across one.
 * @property {JsonType} type - What the values are on the wire.
 */
/**
 * Comma-separated list of series names
 *
 * Deserialization permits at most 32 names, each exact, and 2,048 decoded input
 * string bytes. Array members must all be strings and share the byte budget.
 *
 * @typedef {string} SeriesList
 */
/** @typedef {string} SeriesName */
/**
 * The server's version: a release's, `v` and its SemVer (`v0.12.2`; a pre-release,
 * `v0.13.0-beta.1`); built from a commit of the repository (`cargo install --git`, next's), that
 * commit, short (`a1b2c3d`).
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
 * A spend of a Taproot output as BlindBit's light clients test for it: the first 8 bytes of
 * `sha256(txid ‖ vout ‖ block hash)`, the txid and the spending block's hash in internal
 * byte order, the vout little-endian.
 *
 * @typedef {string} SpentOutputHash
 */
/**
 * @typedef {Object} SyncStatus
 * @property {Height} indexedHeight - Height of the last indexed block
 * @property {Height} computedHeight - Height of the last computed block (series)
 * @property {Height} tipHeight - Height of the Bitcoin node's chain tip, as the server last saw it (it looks for a new tip
 *   every second)
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
 * A Unix timestamp, in seconds.
 *
 * @typedef {number} Timestamp
 */
/**
 * Transaction information compatible with mempool.space API format
 *
 * @typedef {Object} Transaction
 * @property {?TxIndex=} index - Internal transaction index (brk-specific, not in mempool.space)
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
 * A BIP352 silent payments tweak, `input_hash·A`: the compressed point a wallet multiplies by
 * its scan key for the secret it shares with the transaction.
 *
 * @typedef {string} Tweak
 */
/**
 * @typedef {Object} TxAudit
 * @property {boolean} seen
 * @property {boolean} expected
 * @property {boolean} added
 * @property {boolean} prioritized
 * @property {boolean} conflict
 * @property {boolean} accelerated
 */
/**
 * @typedef {Object} TxIn
 * @property {Txid} txid - Transaction ID of the output being spent
 * @property {Vout} vout - Output index being spent (coinbase: 4294967295)
 * @property {?TxOut} prevout - The output being spent
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
 * @property {?Txid=} txid - Transaction ID of the spending transaction (only present if spent)
 * @property {?Vin=} vin - Input index in the spending transaction (only present if spent)
 * @property {?TxStatus=} status - Status of the spending transaction (only present if spent)
 */
/**
 * @typedef {Object} TxStatus
 * @property {boolean} confirmed
 * @property {?Height=} blockHeight - Block height (only present if confirmed)
 * @property {?BlockHash=} blockHash - Block hash (only present if confirmed)
 * @property {?Timestamp=} blockTime - Block timestamp (only present if confirmed)
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
 * log10/log50/log100/log200/log500/log1000/log2000 (logarithmic, that many buckets per decade).
 *
 * @typedef {("raw"|"lin200"|"lin500"|"lin1000"|"log10"|"log50"|"log100"|"log200"|"log500"|"log1000"|"log2000")} UrpdAggregation
 */
/**
 * Columnar buckets in ascending price order: entry `i` of each column is bucket `i`.
 *
 * @typedef {Object} UrpdBuckets
 * @property {Dollars[]} price - Each bucket's price, in USD: its lower bound when aggregated, and for `raw` the price its
 *   coins' creation prices round to (four significant digits of the price in cents).
 * @property {Bitcoin[]} supply - Supply held with a last-move price inside each bucket, in BTC. With a `wakefulness` or
 *   `mobility` weight, the weighted supply, floored to whole satoshis at each grid price.
 * @property {Dollars[]} realizedCap - Realized cap contribution in USD: `price * supply` summed over the coins in each bucket,
 *   each coin's creation price taken on the price grid (the `raw` bucket price, within 0.05%
 *   of the exact price), rounded to the cent: a bucket worth under half a cent shows 0. The
 *   buckets therefore need not sum to the cohort's realized cap series.
 * @property {Dollars[]} unrealizedPnl - Unrealized P&L in USD at the represented block: `close * supply` minus each bucket's grid
 *   capital, rounded to the cent once. Can be negative.
 */
/**
 * A URPD cohort, by its series' prefix: an age aggregate (`all`, `sth`, `lth`, `age_under_4m`, ...) or an age band (`age_under_1h`, `age_1h_to_1d`, ...).
 *
 * @typedef {("all"|"sth"|"lth"|"age_under_4m"|"age_under_6m"|"age_over_4m"|"age_over_6m"|"age_under_1h"|"age_1h_to_1d"|"age_1d_to_1w"|"age_1w_to_1m"|"age_1m_to_2m"|"age_2m_to_3m"|"age_3m_to_4m"|"age_4m_to_5m"|"age_5m_to_6m"|"age_6m_to_9m"|"age_9m_to_1y"|"age_1y_to_18m"|"age_18m_to_2y"|"age_2y_to_3y"|"age_3y_to_4y"|"age_4y_to_5y"|"age_5y_to_6y"|"age_6y_to_7y"|"age_7y_to_8y"|"age_8y_to_10y"|"age_10y_to_12y"|"age_12y_to_15y"|"age_over_15y")} UrpdCohort
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
 *   times).
 * @property {number} count - Number of unspent outputs.
 * @property {Bitcoin} supply - BTC held by the unspent outputs.
 * @property {UtxoOrigins} origins - Unspent outputs by creation height: entry `i` is what remains of block `i`'s outputs.
 */
/**
 * What blocks `first` through `last` changed in the UTXO set.
 *
 * @typedef {Object} UtxoSetDiff
 * @property {Height} first - First block of the diff; it applies to the set after block `first - 1` (the empty set for
 *   block 0).
 * @property {Height} last - Last block of the diff, described by `hash` and `date`.
 * @property {BlockHash} hash - Hash of block `last`.
 * @property {Date} date - UTC date of block `last`'s monotonic timestamp.
 * @property {UtxoChanges} created - Outputs created, one row per block, including those spent again within the range (which
 *   `spent` lists too).
 * @property {UtxoChanges} spent - Outputs removed, one row per creation height in ascending order: spent outputs, plus the
 *   coinbase outputs of blocks 91812 and 91722, which duplicates at 91842 and 91880 overwrote.
 */
/**
 * Virtual size in vbytes (weight / 4, rounded up). Max block vsize is ~1,000,000 vB.
 *
 * @typedef {number} VSize
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
 * Proof of work, in the hashes expected to find it.
 *
 * @typedef {number} Work
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
 * @typedef {Object} BitviewClientOptions
 * @property {string} baseUrl
 * @property {number} [timeout] - Request timeout in milliseconds, the download included. Default: 30000
 * @property {string|boolean} [browserCache] - Enable browser Cache API with default name (true), custom name (string), or disable (false). No effect in Node.js. Default: true
 * @property {number|boolean} [memCache] - LRU capacity of the in-memory parsed-response cache, which lets 304 responses skip the JSON parse: true/undefined → 1000, false/0 → disabled. Callers of a URL with the same body reader and JSON key-preservation schema share the value it holds, frozen deeply (bytes, a Uint8Array, can't be frozen: don't change them): copy it before changing it, or disable the cache to own every value
 */

/**
 * A JSON value as the memory cache shares it, frozen deeply (`_deepFreeze`): readonly all the way
 * down, `any` and `unknown` left as they are. Plain data only (a series, with its methods, types its
 * own data so).
 * @template T
 * @typedef {unknown extends T ? T : FrozenObject<T>} Frozen
 */

/**
 * `Frozen`'s readonly mapping of a known type (a primitive maps to itself).
 * @template T
 * @typedef {{ readonly [K in keyof T]: Frozen<T[K]> }} FrozenObject
 */

/**
 * @template T
 * @typedef {Object} SeriesDataBase
 * @property {number} version
 * @property {Index} index
 * @property {string} type - Value type (e.g. "Ratio", "Sats", "Cents")
 * @property {number} start - Start index (inclusive)
 * @property {number} end - End index (exclusive)
 * @property {readonly Frozen<T>[]} data - The series data (`null` where a value is missing or undefined), frozen when the memory cache shares it
 * @property {boolean} isDateBased
 * @property {() => number[]} indexes
 * @property {() => number[]} keys - Alias for indexes
 * @property {() => Array<[number, Frozen<T>]>} entries - Get [index, value] pairs
 * @property {() => Map<number, Frozen<T>>} toMap - Convert to Map<index, value>
 */

/** @template T @typedef {SeriesDataBase<T> & Iterable<[number, Frozen<T>]>} SeriesData */

/**
 * @template T
 * @typedef {Object} DateSeriesDataExtras
 * @property {() => globalThis.Date[]} dates - Get dates for each data point
 * @property {() => Array<[globalThis.Date, Frozen<T>]>} dateEntries
 * @property {() => Map<globalThis.Date, Frozen<T>>} toDateMap
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
 * @property {boolean} [memCache] - Use the parsed in-memory response cache, whose values are shared and frozen deeply (bytes can't be frozen: don't change them). Set false for large one-shot reads, or a value of your own to change.
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

/** @template T @typedef {{ etag: string | null, value: T }} _MemEntry */

import { BitviewClientBase, BitviewError, _SERIES_KEEP, _endpoint, _lazy, _readJson, _validateAddressPayloadForType, _wrapSeriesData, addressPayloadHashPrefix, dateToIndex, indexToDate } from './core.js';

const _i1 = /** @type {const} */ (["minute10", "minute30", "hour1", "hour4", "hour12", "day1", "day3", "week1", "month1", "month3", "month6", "year1", "year10", "halving_epoch", "difficulty_epoch", "height"]);
const _i2 = /** @type {const} */ (["minute10", "minute30", "hour1", "hour4", "hour12", "day1", "day3", "week1", "month1", "month3", "month6", "year1", "year10", "halving_epoch", "difficulty_epoch", "height"]);
const _i3 = /** @type {const} */ (["minute10", "minute30", "hour1", "hour4", "hour12", "day1", "day3", "week1", "month1", "month3", "month6", "year1", "year10", "halving_epoch", "difficulty_epoch"]);
const _i4 = /** @type {const} */ (["minute10"]);
const _i5 = /** @type {const} */ (["minute30"]);
const _i6 = /** @type {const} */ (["hour1"]);
const _i7 = /** @type {const} */ (["hour4"]);
const _i8 = /** @type {const} */ (["hour12"]);
const _i9 = /** @type {const} */ (["day1"]);
const _i10 = /** @type {const} */ (["day3"]);
const _i11 = /** @type {const} */ (["week1"]);
const _i12 = /** @type {const} */ (["month1"]);
const _i13 = /** @type {const} */ (["month3"]);
const _i14 = /** @type {const} */ (["month6"]);
const _i15 = /** @type {const} */ (["year1"]);
const _i16 = /** @type {const} */ (["year10"]);
const _i17 = /** @type {const} */ (["halving_epoch"]);
const _i18 = /** @type {const} */ (["difficulty_epoch"]);
const _i19 = /** @type {const} */ (["height"]);
const _i20 = /** @type {const} */ (["tx_index"]);
const _i21 = /** @type {const} */ (["txin_index"]);
const _i22 = /** @type {const} */ (["txin_index"]);
const _i23 = /** @type {const} */ (["txout_index"]);
const _i24 = /** @type {const} */ (["txout_index"]);
const _i25 = /** @type {const} */ (["empty_script_output_index"]);
const _i26 = /** @type {const} */ (["op_return_output_index"]);
const _i27 = /** @type {const} */ (["p2a_address_index"]);
const _i28 = /** @type {const} */ (["p2ms_output_index"]);
const _i29 = /** @type {const} */ (["p2pk33_address_index"]);
const _i30 = /** @type {const} */ (["p2pk65_address_index"]);
const _i31 = /** @type {const} */ (["p2pkh_address_index"]);
const _i32 = /** @type {const} */ (["p2sh_address_index"]);
const _i33 = /** @type {const} */ (["p2tr_address_index"]);
const _i34 = /** @type {const} */ (["p2wpkh_address_index"]);
const _i35 = /** @type {const} */ (["p2wsh_address_index"]);
const _i36 = /** @type {const} */ (["unknown_script_output_index"]);

/**
 * @template T
 * @param {BitviewClient} client
 * @param {string} name
 * @param {readonly Index[]} indexes
 */
function _seriesPattern(client, name, indexes) {
  return {
    name,
    /** Each index's endpoint: made when first read, as the tree's nodes are (its thousands of series are walked, few read). */
    get by() {
      const by = {};
      for (const idx of indexes) {
        Object.defineProperty(by, idx, {
          get() { return _endpoint(client, name, idx); },
          enumerable: true,
          configurable: true
        });
      }
      return _lazy(this, 'by', () => by);
    },
    /** @returns {readonly Index[]} */
    indexes() { return indexes; },
    /** @param {Index} index @returns {SeriesEndpoint<T>|undefined} */
    get(index) { return indexes.includes(index) ? _endpoint(client, name, index) : undefined; }
  };
}

/** @template T @typedef {{ name: string, by: { readonly minute10: DateSeriesEndpoint<T | null>, readonly minute30: DateSeriesEndpoint<T | null>, readonly hour1: DateSeriesEndpoint<T | null>, readonly hour4: DateSeriesEndpoint<T | null>, readonly hour12: DateSeriesEndpoint<T | null>, readonly day1: DateSeriesEndpoint<T | null>, readonly day3: DateSeriesEndpoint<T | null>, readonly week1: DateSeriesEndpoint<T | null>, readonly month1: DateSeriesEndpoint<T | null>, readonly month3: DateSeriesEndpoint<T | null>, readonly month6: DateSeriesEndpoint<T | null>, readonly year1: DateSeriesEndpoint<T | null>, readonly year10: DateSeriesEndpoint<T | null>, readonly halving_epoch: SeriesEndpoint<T>, readonly difficulty_epoch: SeriesEndpoint<T>, readonly height: SeriesEndpoint<T> }, indexes: () => readonly Index[], get: (index: Index) => SeriesEndpoint<T | null>|undefined }} SeriesPattern1 */
/** @template T @typedef {{ name: string, by: { readonly minute10: DateSeriesEndpoint<T | null>, readonly minute30: DateSeriesEndpoint<T | null>, readonly hour1: DateSeriesEndpoint<T | null>, readonly hour4: DateSeriesEndpoint<T | null>, readonly hour12: DateSeriesEndpoint<T | null>, readonly day1: DateSeriesEndpoint<T | null>, readonly day3: DateSeriesEndpoint<T | null>, readonly week1: DateSeriesEndpoint<T | null>, readonly month1: DateSeriesEndpoint<T | null>, readonly month3: DateSeriesEndpoint<T | null>, readonly month6: DateSeriesEndpoint<T | null>, readonly year1: DateSeriesEndpoint<T | null>, readonly year10: DateSeriesEndpoint<T | null>, readonly halving_epoch: SeriesEndpoint<T | null>, readonly difficulty_epoch: SeriesEndpoint<T | null>, readonly height: SeriesEndpoint<T | null> }, indexes: () => readonly Index[], get: (index: Index) => SeriesEndpoint<T | null>|undefined }} SeriesPattern2 */
/** @template T @typedef {{ name: string, by: { readonly minute10: DateSeriesEndpoint<T>, readonly minute30: DateSeriesEndpoint<T>, readonly hour1: DateSeriesEndpoint<T>, readonly hour4: DateSeriesEndpoint<T>, readonly hour12: DateSeriesEndpoint<T>, readonly day1: DateSeriesEndpoint<T>, readonly day3: DateSeriesEndpoint<T>, readonly week1: DateSeriesEndpoint<T>, readonly month1: DateSeriesEndpoint<T>, readonly month3: DateSeriesEndpoint<T>, readonly month6: DateSeriesEndpoint<T>, readonly year1: DateSeriesEndpoint<T>, readonly year10: DateSeriesEndpoint<T>, readonly halving_epoch: SeriesEndpoint<T>, readonly difficulty_epoch: SeriesEndpoint<T> }, indexes: () => readonly Index[], get: (index: Index) => SeriesEndpoint<T>|undefined }} SeriesPattern3 */
/** @template T @typedef {{ name: string, by: { readonly minute10: DateSeriesEndpoint<T> }, indexes: () => readonly Index[], get: (index: Index) => SeriesEndpoint<T>|undefined }} SeriesPattern4 */
/** @template T @typedef {{ name: string, by: { readonly minute30: DateSeriesEndpoint<T> }, indexes: () => readonly Index[], get: (index: Index) => SeriesEndpoint<T>|undefined }} SeriesPattern5 */
/** @template T @typedef {{ name: string, by: { readonly hour1: DateSeriesEndpoint<T> }, indexes: () => readonly Index[], get: (index: Index) => SeriesEndpoint<T>|undefined }} SeriesPattern6 */
/** @template T @typedef {{ name: string, by: { readonly hour4: DateSeriesEndpoint<T> }, indexes: () => readonly Index[], get: (index: Index) => SeriesEndpoint<T>|undefined }} SeriesPattern7 */
/** @template T @typedef {{ name: string, by: { readonly hour12: DateSeriesEndpoint<T> }, indexes: () => readonly Index[], get: (index: Index) => SeriesEndpoint<T>|undefined }} SeriesPattern8 */
/** @template T @typedef {{ name: string, by: { readonly day1: DateSeriesEndpoint<T> }, indexes: () => readonly Index[], get: (index: Index) => SeriesEndpoint<T>|undefined }} SeriesPattern9 */
/** @template T @typedef {{ name: string, by: { readonly day3: DateSeriesEndpoint<T> }, indexes: () => readonly Index[], get: (index: Index) => SeriesEndpoint<T>|undefined }} SeriesPattern10 */
/** @template T @typedef {{ name: string, by: { readonly week1: DateSeriesEndpoint<T> }, indexes: () => readonly Index[], get: (index: Index) => SeriesEndpoint<T>|undefined }} SeriesPattern11 */
/** @template T @typedef {{ name: string, by: { readonly month1: DateSeriesEndpoint<T> }, indexes: () => readonly Index[], get: (index: Index) => SeriesEndpoint<T>|undefined }} SeriesPattern12 */
/** @template T @typedef {{ name: string, by: { readonly month3: DateSeriesEndpoint<T> }, indexes: () => readonly Index[], get: (index: Index) => SeriesEndpoint<T>|undefined }} SeriesPattern13 */
/** @template T @typedef {{ name: string, by: { readonly month6: DateSeriesEndpoint<T> }, indexes: () => readonly Index[], get: (index: Index) => SeriesEndpoint<T>|undefined }} SeriesPattern14 */
/** @template T @typedef {{ name: string, by: { readonly year1: DateSeriesEndpoint<T> }, indexes: () => readonly Index[], get: (index: Index) => SeriesEndpoint<T>|undefined }} SeriesPattern15 */
/** @template T @typedef {{ name: string, by: { readonly year10: DateSeriesEndpoint<T> }, indexes: () => readonly Index[], get: (index: Index) => SeriesEndpoint<T>|undefined }} SeriesPattern16 */
/** @template T @typedef {{ name: string, by: { readonly halving_epoch: SeriesEndpoint<T> }, indexes: () => readonly Index[], get: (index: Index) => SeriesEndpoint<T>|undefined }} SeriesPattern17 */
/** @template T @typedef {{ name: string, by: { readonly difficulty_epoch: SeriesEndpoint<T> }, indexes: () => readonly Index[], get: (index: Index) => SeriesEndpoint<T>|undefined }} SeriesPattern18 */
/** @template T @typedef {{ name: string, by: { readonly height: SeriesEndpoint<T> }, indexes: () => readonly Index[], get: (index: Index) => SeriesEndpoint<T>|undefined }} SeriesPattern19 */
/** @template T @typedef {{ name: string, by: { readonly tx_index: SeriesEndpoint<T> }, indexes: () => readonly Index[], get: (index: Index) => SeriesEndpoint<T>|undefined }} SeriesPattern20 */
/** @template T @typedef {{ name: string, by: { readonly txin_index: SeriesEndpoint<T> }, indexes: () => readonly Index[], get: (index: Index) => SeriesEndpoint<T>|undefined }} SeriesPattern21 */
/** @template T @typedef {{ name: string, by: { readonly txin_index: SeriesEndpoint<T | null> }, indexes: () => readonly Index[], get: (index: Index) => SeriesEndpoint<T | null>|undefined }} SeriesPattern22 */
/** @template T @typedef {{ name: string, by: { readonly txout_index: SeriesEndpoint<T> }, indexes: () => readonly Index[], get: (index: Index) => SeriesEndpoint<T>|undefined }} SeriesPattern23 */
/** @template T @typedef {{ name: string, by: { readonly txout_index: SeriesEndpoint<T | null> }, indexes: () => readonly Index[], get: (index: Index) => SeriesEndpoint<T | null>|undefined }} SeriesPattern24 */
/** @template T @typedef {{ name: string, by: { readonly empty_script_output_index: SeriesEndpoint<T> }, indexes: () => readonly Index[], get: (index: Index) => SeriesEndpoint<T>|undefined }} SeriesPattern25 */
/** @template T @typedef {{ name: string, by: { readonly op_return_output_index: SeriesEndpoint<T> }, indexes: () => readonly Index[], get: (index: Index) => SeriesEndpoint<T>|undefined }} SeriesPattern26 */
/** @template T @typedef {{ name: string, by: { readonly p2a_address_index: SeriesEndpoint<T> }, indexes: () => readonly Index[], get: (index: Index) => SeriesEndpoint<T>|undefined }} SeriesPattern27 */
/** @template T @typedef {{ name: string, by: { readonly p2ms_output_index: SeriesEndpoint<T> }, indexes: () => readonly Index[], get: (index: Index) => SeriesEndpoint<T>|undefined }} SeriesPattern28 */
/** @template T @typedef {{ name: string, by: { readonly p2pk33_address_index: SeriesEndpoint<T> }, indexes: () => readonly Index[], get: (index: Index) => SeriesEndpoint<T>|undefined }} SeriesPattern29 */
/** @template T @typedef {{ name: string, by: { readonly p2pk65_address_index: SeriesEndpoint<T> }, indexes: () => readonly Index[], get: (index: Index) => SeriesEndpoint<T>|undefined }} SeriesPattern30 */
/** @template T @typedef {{ name: string, by: { readonly p2pkh_address_index: SeriesEndpoint<T> }, indexes: () => readonly Index[], get: (index: Index) => SeriesEndpoint<T>|undefined }} SeriesPattern31 */
/** @template T @typedef {{ name: string, by: { readonly p2sh_address_index: SeriesEndpoint<T> }, indexes: () => readonly Index[], get: (index: Index) => SeriesEndpoint<T>|undefined }} SeriesPattern32 */
/** @template T @typedef {{ name: string, by: { readonly p2tr_address_index: SeriesEndpoint<T> }, indexes: () => readonly Index[], get: (index: Index) => SeriesEndpoint<T>|undefined }} SeriesPattern33 */
/** @template T @typedef {{ name: string, by: { readonly p2wpkh_address_index: SeriesEndpoint<T> }, indexes: () => readonly Index[], get: (index: Index) => SeriesEndpoint<T>|undefined }} SeriesPattern34 */
/** @template T @typedef {{ name: string, by: { readonly p2wsh_address_index: SeriesEndpoint<T> }, indexes: () => readonly Index[], get: (index: Index) => SeriesEndpoint<T>|undefined }} SeriesPattern35 */
/** @template T @typedef {{ name: string, by: { readonly unknown_script_output_index: SeriesEndpoint<T> }, indexes: () => readonly Index[], get: (index: Index) => SeriesEndpoint<T>|undefined }} SeriesPattern36 */

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
  // (By index, nothing destructured: no iterator stepped for each child, thousands of nodes on an engine without a JIT.)
  const keys = Object.keys(children);
  for (let i = 0; i < keys.length; i++) {
    const key = keys[i], child = children[key], make = child[0], template = child[1];
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
 *   index: SeriesPattern9<?Float>,
 *   shortTermCloudPosition: SeriesPattern9<?Float>,
 *   longTermCloudPosition: SeriesPattern9<?Float>,
 *   mood: SeriesPattern9<?Mood>,
 *   extreme: SeriesPattern9<?Flag>,
 *   moodDays: SeriesPattern9<?Days>,
 * }} CompassSeries
 */
const _CompassSeries = _s({
  index: [_i9, '*_index'],
  shortTermCloudPosition: [_i9, '*_short_term_cloud_position'],
  longTermCloudPosition: [_i9, '*_long_term_cloud_position'],
  mood: [_i9, '*_mood'],
  extreme: [_i9, '*_extreme'],
  moodDays: [_i9, '*_mood_days'],
});

/**
 * @typedef {{
 *   mobile: CompassSeries,
 *   awake: CompassSeries,
 * }} Compass
 */
const _Compass = _s({
  mobile: [_CompassSeries, 'mobile_*'],
  awake: [_CompassSeries, 'awake_*'],
});

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
  p10: [_i1, '*_interpolated_p10'],
  p20: [_i1, '*_interpolated_p20'],
  p30: [_i1, '*_interpolated_p30'],
  p40: [_i1, '*_interpolated_p40'],
  median: [_i1, '*_interpolated_midpoint'],
  p60: [_i1, '*_interpolated_p60'],
  p70: [_i1, '*_interpolated_p70'],
  p80: [_i1, '*_interpolated_p80'],
  p90: [_i1, '*_interpolated_p90'],
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
 * }} P99_9
 */
const _P99_9 = _s({
  ratio: [_i1, '*_ratio_rarity_p99_9'],
  band: [_i1, '*_rarity_p99_9'],
});

/**
 * @typedef {{
 *   ratio: SeriesPattern1<?Multiple>,
 *   band: SeriesPattern1<?Dollars>,
 * }} P99_5
 */
const _P99_5 = _s({
  ratio: [_i1, '*_ratio_rarity_p99_5'],
  band: [_i1, '*_rarity_p99_5'],
});

/**
 * @typedef {{
 *   ratio: SeriesPattern1<?Multiple>,
 *   band: SeriesPattern1<?Dollars>,
 * }} P99
 */
const _P99 = _s({
  ratio: [_i1, '*_ratio_rarity_p99'],
  band: [_i1, '*_rarity_p99'],
});

/**
 * @typedef {{
 *   ratio: SeriesPattern1<?Multiple>,
 *   band: SeriesPattern1<?Dollars>,
 * }} P98
 */
const _P98 = _s({
  ratio: [_i1, '*_ratio_rarity_p98'],
  band: [_i1, '*_rarity_p98'],
});

/**
 * @typedef {{
 *   ratio: SeriesPattern1<?Multiple>,
 *   band: SeriesPattern1<?Dollars>,
 * }} P95
 */
const _P95 = _s({
  ratio: [_i1, '*_ratio_rarity_p95'],
  band: [_i1, '*_rarity_p95'],
});

/**
 * @typedef {{
 *   ratio: SeriesPattern1<?Multiple>,
 *   band: SeriesPattern1<?Dollars>,
 * }} P90
 */
const _P90 = _s({
  ratio: [_i1, '*_ratio_rarity_p90'],
  band: [_i1, '*_rarity_p90'],
});

/**
 * @typedef {{
 *   ratio: SeriesPattern1<?Multiple>,
 *   band: SeriesPattern1<?Dollars>,
 * }} P80
 */
const _P80 = _s({
  ratio: [_i1, '*_ratio_rarity_p80'],
  band: [_i1, '*_rarity_p80'],
});

/**
 * @typedef {{
 *   ratio: SeriesPattern1<?Multiple>,
 *   band: SeriesPattern1<?Dollars>,
 * }} P70
 */
const _P70 = _s({
  ratio: [_i1, '*_ratio_rarity_p70'],
  band: [_i1, '*_rarity_p70'],
});

/**
 * @typedef {{
 *   ratio: SeriesPattern1<?Multiple>,
 *   band: SeriesPattern1<?Dollars>,
 * }} P60
 */
const _P60 = _s({
  ratio: [_i1, '*_ratio_rarity_p60'],
  band: [_i1, '*_rarity_p60'],
});

/**
 * @typedef {{
 *   ratio: SeriesPattern1<?Multiple>,
 *   band: SeriesPattern1<?Dollars>,
 * }} Median
 */
const _Median = _s({
  ratio: [_i1, '*_ratio_rarity_median'],
  band: [_i1, '*_rarity_median'],
});

/**
 * @typedef {{
 *   ratio: SeriesPattern1<?Multiple>,
 *   band: SeriesPattern1<?Dollars>,
 * }} P40
 */
const _P40 = _s({
  ratio: [_i1, '*_ratio_rarity_p40'],
  band: [_i1, '*_rarity_p40'],
});

/**
 * @typedef {{
 *   ratio: SeriesPattern1<?Multiple>,
 *   band: SeriesPattern1<?Dollars>,
 * }} P30
 */
const _P30 = _s({
  ratio: [_i1, '*_ratio_rarity_p30'],
  band: [_i1, '*_rarity_p30'],
});

/**
 * @typedef {{
 *   ratio: SeriesPattern1<?Multiple>,
 *   band: SeriesPattern1<?Dollars>,
 * }} P20
 */
const _P20 = _s({
  ratio: [_i1, '*_ratio_rarity_p20'],
  band: [_i1, '*_rarity_p20'],
});

/**
 * @typedef {{
 *   ratio: SeriesPattern1<?Multiple>,
 *   band: SeriesPattern1<?Dollars>,
 * }} P10
 */
const _P10 = _s({
  ratio: [_i1, '*_ratio_rarity_p10'],
  band: [_i1, '*_rarity_p10'],
});

/**
 * @typedef {{
 *   ratio: SeriesPattern1<?Multiple>,
 *   band: SeriesPattern1<?Dollars>,
 * }} P5
 */
const _P5 = _s({
  ratio: [_i1, '*_ratio_rarity_p5'],
  band: [_i1, '*_rarity_p5'],
});

/**
 * @typedef {{
 *   ratio: SeriesPattern1<?Multiple>,
 *   band: SeriesPattern1<?Dollars>,
 * }} P2
 */
const _P2 = _s({
  ratio: [_i1, '*_ratio_rarity_p2'],
  band: [_i1, '*_rarity_p2'],
});

/**
 * @typedef {{
 *   ratio: SeriesPattern1<?Multiple>,
 *   band: SeriesPattern1<?Dollars>,
 * }} P1
 */
const _P1 = _s({
  ratio: [_i1, '*_ratio_rarity_p1'],
  band: [_i1, '*_rarity_p1'],
});

/**
 * @typedef {{
 *   ratio: SeriesPattern1<?Multiple>,
 *   band: SeriesPattern1<?Dollars>,
 * }} P0_5
 */
const _P0_5 = _s({
  ratio: [_i1, '*_ratio_rarity_p0_5'],
  band: [_i1, '*_rarity_p0_5'],
});

/**
 * @typedef {{
 *   ratio: SeriesPattern1<?Multiple>,
 *   band: SeriesPattern1<?Dollars>,
 * }} P0_1
 */
const _P0_1 = _s({
  ratio: [_i1, '*_ratio_rarity_p0_1'],
  band: [_i1, '*_rarity_p0_1'],
});

/**
 * @typedef {{
 *   value: SeriesPattern1<?Dollars>,
 *   ratio: SeriesPattern1<?Multiple>,
 *   p0_1: P0_1,
 *   p0_5: P0_5,
 *   p1: P1,
 *   p2: P2,
 *   p5: P5,
 *   p10: P10,
 *   p20: P20,
 *   p30: P30,
 *   p40: P40,
 *   median: Median,
 *   p60: P60,
 *   p70: P70,
 *   p80: P80,
 *   p90: P90,
 *   p95: P95,
 *   p98: P98,
 *   p99: P99,
 *   p99_5: P99_5,
 *   p99_9: P99_9,
 * }} LazyComponent
 */
const _LazyComponent = _s({
  value: [_i1, '*'],
  ratio: [_i1, '*_ratio'],
  p0_1: [_P0_1, '*'],
  p0_5: [_P0_5, '*'],
  p1: [_P1, '*'],
  p2: [_P2, '*'],
  p5: [_P5, '*'],
  p10: [_P10, '*'],
  p20: [_P20, '*'],
  p30: [_P30, '*'],
  p40: [_P40, '*'],
  median: [_Median, '*'],
  p60: [_P60, '*'],
  p70: [_P70, '*'],
  p80: [_P80, '*'],
  p90: [_P90, '*'],
  p95: [_P95, '*'],
  p98: [_P98, '*'],
  p99: [_P99, '*'],
  p99_5: [_P99_5, '*'],
  p99_9: [_P99_9, '*'],
});

/**
 * @typedef {{
 *   p0_1: P0_1,
 *   p0_5: P0_5,
 *   p1: P1,
 *   p2: P2,
 *   p5: P5,
 *   p10: P10,
 *   p20: P20,
 *   p30: P30,
 *   p40: P40,
 *   median: Median,
 *   p60: P60,
 *   p70: P70,
 *   p80: P80,
 *   p90: P90,
 *   p95: P95,
 *   p98: P98,
 *   p99: P99,
 *   p99_5: P99_5,
 *   p99_9: P99_9,
 * }} RarityPercentiles
 */
const _RarityPercentiles = _s({
  p0_1: [_P0_1, '*'],
  p0_5: [_P0_5, '*'],
  p1: [_P1, '*'],
  p2: [_P2, '*'],
  p5: [_P5, '*'],
  p10: [_P10, '*'],
  p20: [_P20, '*'],
  p30: [_P30, '*'],
  p40: [_P40, '*'],
  median: [_Median, '*'],
  p60: [_P60, '*'],
  p70: [_P70, '*'],
  p80: [_P80, '*'],
  p90: [_P90, '*'],
  p95: [_P95, '*'],
  p98: [_P98, '*'],
  p99: [_P99, '*'],
  p99_5: [_P99_5, '*'],
  p99_9: [_P99_9, '*'],
});

/**
 * @typedef {{
 *   realizedPrice: RarityPercentiles,
 *   capitalizedPrice: RarityPercentiles,
 *   medianRealizedPrice: LazyComponent,
 *   medianCapitalizedPrice: LazyComponent,
 * }} CohortComponents
 */
const _CohortComponents = _s({
  realizedPrice: [_RarityPercentiles, '*_realized_price'],
  capitalizedPrice: [_RarityPercentiles, '*_capitalized_price'],
  medianRealizedPrice: [_LazyComponent, '*_median_realized_price'],
  medianCapitalizedPrice: [_LazyComponent, '*_median_capitalized_price'],
});

/**
 * @typedef {{
 *   realizedPrice: RarityPercentiles,
 *   capitalizedPrice: RarityPercentiles,
 *   medianRealizedPrice: LazyComponent,
 *   medianCapitalizedPrice: LazyComponent,
 *   awake: CohortComponents,
 *   mobile: CohortComponents,
 * }} BySupply
 */
const _BySupply = _s({
  realizedPrice: [_RarityPercentiles, '*_realized_price'],
  capitalizedPrice: [_RarityPercentiles, '*_capitalized_price'],
  medianRealizedPrice: [_LazyComponent, '*_median_realized_price'],
  medianCapitalizedPrice: [_LazyComponent, '*_median_capitalized_price'],
  awake: [_CohortComponents, '*_awake'],
  mobile: [_CohortComponents, '*_mobile'],
});

/**
 * @typedef {{
 *   realizedPrice: RarityPercentiles,
 *   capitalizedPrice: RarityPercentiles,
 *   medianRealizedPrice: LazyComponent,
 *   medianCapitalizedPrice: LazyComponent,
 *   awake: CohortComponents,
 *   mobile: CohortComponents,
 *   vaultedPrice: RarityPercentiles,
 *   trueMarketMean: RarityPercentiles,
 *   cointimePrice: RarityPercentiles,
 *   activePrice: RarityPercentiles,
 *   cvdd: LazyComponent,
 *   balancedPrice: LazyComponent,
 *   deltaPrice: LazyComponent,
 * }} AllComponents
 */
const _AllComponents = _s({
  realizedPrice: [_RarityPercentiles, 'realized_*'],
  capitalizedPrice: [_RarityPercentiles, 'capitalized_*'],
  medianRealizedPrice: [_LazyComponent, 'median_realized_*'],
  medianCapitalizedPrice: [_LazyComponent, 'median_capitalized_*'],
  awake: [_CohortComponents, 'awake'],
  mobile: [_CohortComponents, 'mobile'],
  vaultedPrice: [_RarityPercentiles, 'vaulted_*'],
  trueMarketMean: [_RarityPercentiles, 'true_market_mean'],
  cointimePrice: [_RarityPercentiles, 'cointime_*'],
  activePrice: [_RarityPercentiles, 'active_*'],
  cvdd: [_LazyComponent, 'cvdd'],
  balancedPrice: [_LazyComponent, 'balanced_*'],
  deltaPrice: [_LazyComponent, 'delta_*'],
});

/**
 * @typedef {{
 *   all: AllComponents,
 *   sth: BySupply,
 *   lth: BySupply,
 *   under4m: BySupply,
 *   under6m: BySupply,
 *   over4m: BySupply,
 *   over6m: BySupply,
 * }} RarityMeterAgeAggregate
 */
const _RarityMeterAgeAggregate = _s({
  all: [_AllComponents, 'price'],
  sth: [_BySupply, 'sth'],
  lth: [_BySupply, 'lth'],
  under4m: [_BySupply, '*_under_4m'],
  under6m: [_BySupply, '*_under_6m'],
  over4m: [_BySupply, '*_over_4m'],
  over6m: [_BySupply, '*_over_6m'],
});

/**
 * @typedef {{
 *   components: RarityMeterAgeAggregate,
 *   extremes: Extremes,
 *   full: Meter,
 *   local: Meter,
 *   cycle: Meter,
 *   compass: Compass,
 * }} RarityMeter
 */
const _RarityMeter = _s({
  components: [_RarityMeterAgeAggregate, 'age'],
  extremes: [_Extremes, '24h'],
  full: [_Meter, '*'],
  local: [_Meter, 'local_*'],
  cycle: [_Meter, 'cycle_*'],
  compass: [_Compass, 'compass'],
});

/**
 * @typedef {{
 *   btc: SeriesPattern1<?Bitcoin>,
 *   usd: SeriesPattern1<?Dollars>,
 *   marketCap: SeriesPattern1<?Dollars>,
 * }} UnweightedEntrySupplyBelowP99_9Floor
 */
const _UnweightedEntrySupplyBelowP99_9Floor = _s({
  btc: [_i1, 'bedrock_unweighted_entry_supply_*'],
  usd: [_i1, 'bedrock_unweighted_entry_supply_*_usd'],
  marketCap: [_i1, 'bedrock_unweighted_entry_market_cap_*'],
});

/**
 * @typedef {{
 *   btc: SeriesPattern1<?Bitcoin>,
 *   usd: SeriesPattern1<?Dollars>,
 *   marketCap: SeriesPattern1<?Dollars>,
 * }} MobileEntrySupplyBelowP99_9Floor
 */
const _MobileEntrySupplyBelowP99_9Floor = _s({
  btc: [_i1, 'bedrock_mobile_entry_supply_*'],
  usd: [_i1, 'bedrock_mobile_entry_supply_*_usd'],
  marketCap: [_i1, 'bedrock_mobile_entry_market_cap_*'],
});

/**
 * @typedef {{
 *   betweenP95AndP98Floors: SeriesPattern1<?Dollars>,
 *   betweenP98AndP99Floors: SeriesPattern1<?Dollars>,
 *   betweenP99AndP99_5Floors: SeriesPattern1<?Dollars>,
 *   betweenP99_5AndP99_9Floors: SeriesPattern1<?Dollars>,
 *   belowP99_9Floor: SeriesPattern1<?Dollars>,
 * }} EntryCapital
 */
const _EntryCapital = _s({
  betweenP95AndP98Floors: [_i1, '*_between_p95_and_p98_floors'],
  betweenP98AndP99Floors: [_i1, '*_between_p98_and_p99_floors'],
  betweenP99AndP99_5Floors: [_i1, '*_between_p99_and_p99_5_floors'],
  betweenP99_5AndP99_9Floors: [_i1, '*_between_p99_5_and_p99_9_floors'],
  belowP99_9Floor: [_i1, '*_below_p99_9_floor'],
});

/**
 * @typedef {{
 *   btc: SeriesPattern1<?Bitcoin>,
 *   usd: SeriesPattern1<?Dollars>,
 *   marketCap: SeriesPattern1<?Dollars>,
 * }} AwakeEntrySupplyBelowP99_9Floor
 */
const _AwakeEntrySupplyBelowP99_9Floor = _s({
  btc: [_i1, 'bedrock_awake_entry_supply_*'],
  usd: [_i1, 'bedrock_awake_entry_supply_*_usd'],
  marketCap: [_i1, 'bedrock_awake_entry_market_cap_*'],
});

/**
 * @template A
 * @typedef {{
 *   betweenP95AndP98Floors: A,
 *   betweenP98AndP99Floors: A,
 *   betweenP99AndP99_5Floors: A,
 *   betweenP99_5AndP99_9Floors: A,
 *   belowP99_9Floor: A,
 * }} EntrySupply
 */
/** @type {_Make} */
const _EntrySupply = (c, b, f0) => _n(c, b, {
  betweenP95AndP98Floors: [f0, 'between_p95_and_p98_floors'],
  betweenP98AndP99Floors: [f0, 'between_p98_and_*_floors'],
  betweenP99AndP99_5Floors: [f0, 'between_*_and_p99_5_floors'],
  betweenP99_5AndP99_9Floors: [f0, 'between_*_5_and_p99_9_floors'],
  belowP99_9Floor: [f0, 'below_*_9_floor'],
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
 *   p5: SeriesPattern1<?Percent>,
 *   p2: SeriesPattern1<?Percent>,
 *   p1: SeriesPattern1<?Percent>,
 *   p0_5: SeriesPattern1<?Percent>,
 *   p0_1: SeriesPattern1<?Percent>,
 * }} ProfitPercentiles
 */
const _ProfitPercentiles = _s({
  p5: [_i1, '*_p5'],
  p2: [_i1, '*_p2'],
  p1: [_i1, '*_p1'],
  p0_5: [_i1, '*_p0_5'],
  p0_1: [_i1, '*_p0_1'],
});

/**
 * @template A
 * @typedef {{
 *   supplyInProfitShareThreshold: ProfitPercentiles,
 *   supplyInLossShareThreshold: Percentiles<?Percent>,
 *   supplyInProfitShareRank: SeriesPattern9<?Ratio>,
 *   floor: Percentiles<?Dollars>,
 *   costBasisAboveP95Floor: Levels,
 *   entrySupply: EntrySupply<A>,
 *   entryCapital: EntryCapital,
 * }} Mode
 */
/** @type {_Make} */
const _Mode = (c, b, f0) => _n(c, b, {
  supplyInProfitShareThreshold: [_ProfitPercentiles, '*_supply_in_profit_share_threshold'],
  supplyInLossShareThreshold: [_Percentiles, '*_supply_in_loss_share_threshold'],
  supplyInProfitShareRank: [_i9, '*_supply_in_profit_share_rank'],
  floor: [_Percentiles, '*_floor'],
  costBasisAboveP95Floor: [_Levels, '*_cost_basis_above_p95_floor'],
  entrySupply: [(c, b) => _EntrySupply(c, b, f0), 'p99'],
  entryCapital: [_EntryCapital, '*_entry_capital'],
});

/**
 * @typedef {{
 *   awake: Mode<AwakeEntrySupplyBelowP99_9Floor>,
 *   mobile: Mode<MobileEntrySupplyBelowP99_9Floor>,
 *   unweighted: Mode<UnweightedEntrySupplyBelowP99_9Floor>,
 * }} Bedrock
 */
const _Bedrock = _s({
  awake: [(c, b) => _Mode(c, b, _AwakeEntrySupplyBelowP99_9Floor), '*_awake'],
  mobile: [(c, b) => _Mode(c, b, _MobileEntrySupplyBelowP99_9Floor), '*_mobile'],
  unweighted: [(c, b) => _Mode(c, b, _UnweightedEntrySupplyBelowP99_9Floor), '*_unweighted'],
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
 *   transferred: SeriesPattern1<?Dollars>,
 *   balanced: SeriesPattern1<?Dollars>,
 *   terminal: SeriesPattern1<?Dollars>,
 *   cvdd: SeriesPattern1<?Dollars>,
 *   delta: SeriesPattern1<?Dollars>,
 * }} Prices
 */
const _Prices = _s({
  vaulted: [_CointimePriceWithMvrv, 'vaulted'],
  active: [_CointimePriceWithMvrv, 'active'],
  trueMarketMean: [_CointimePriceWithRatio, 'true_market_mean'],
  cointime: [_CointimePriceWithMvrv, 'cointime'],
  transferred: [_i1, 'transferred_*'],
  balanced: [_i1, 'balanced_*'],
  terminal: [_i1, 'terminal_*'],
  cvdd: [_i1, 'cvdd'],
  delta: [_i1, 'delta_*'],
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
 *   marketCap: SeriesPattern1<?Dollars>,
 *   share: SeriesPattern1<?Percent>,
 * }} AllSupplyInLoss
 */
const _AllSupplyInLoss = _s({
  btc: [_i1, '*_supply_in_loss'],
  usd: [_i1, '*_supply_in_loss_usd'],
  marketCap: [_i1, '*_market_cap_in_loss'],
  share: [_i1, '*_supply_in_loss_share'],
});

/**
 * @typedef {{
 *   btc: SeriesPattern1<?Bitcoin>,
 *   usd: SeriesPattern1<?Dollars>,
 *   marketCap: SeriesPattern1<?Dollars>,
 *   share: SeriesPattern1<?Percent>,
 * }} SupplyInProfit
 */
const _SupplyInProfit = _s({
  btc: [_i1, '*_supply_in_profit'],
  usd: [_i1, '*_supply_in_profit_usd'],
  marketCap: [_i1, '*_market_cap_in_profit'],
  share: [_i1, '*_supply_in_profit_share'],
});

/**
 * @typedef {{
 *   btc: SeriesPattern1<?Bitcoin>,
 *   usd: SeriesPattern1<?Dollars>,
 *   marketCap: SeriesPattern1<?Dollars>,
 * }} HoldersSpotValue
 */
const _HoldersSpotValue = _s({
  btc: [_i1, '*'],
  usd: [_i1, '*_usd'],
  marketCap: [_i1, 'market_cap'],
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
 *   marketCap: SeriesPattern1<?Dollars>,
 *   share: SeriesPattern1<?Percent>,
 * }} PredicateSupply
 */
const _PredicateSupply = _s({
  btc: [_i1, '*_supply'],
  usd: [_i1, '*_supply_usd'],
  marketCap: [_i1, '*_market_cap'],
  share: [_i1, '*_supply_share'],
});

/**
 * @typedef {{
 *   count: FundedTotal,
 *   supply: PredicateSupply,
 * }} Exposed
 */
const _Exposed = _s({
  count: [_FundedTotal, '*'],
  supply: [_PredicateSupply, '*_exposed_address'],
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
 *   btc: SeriesPattern1<?Bitcoin>,
 *   usd: SeriesPattern1<?Dollars>,
 *   marketCap: SeriesPattern1<?Dollars>,
 * }} InLoss
 */
const _InLoss = _s({
  btc: [_i1, '*_supply_in_loss'],
  usd: [_i1, '*_supply_in_loss_usd'],
  marketCap: [_i1, '*_market_cap_in_loss'],
});

/**
 * @typedef {{
 *   btc: SeriesPattern1<?Bitcoin>,
 *   usd: SeriesPattern1<?Dollars>,
 *   marketCap: SeriesPattern1<?Dollars>,
 * }} InProfit
 */
const _InProfit = _s({
  btc: [_i1, '*_supply_in_profit'],
  usd: [_i1, '*_supply_in_profit_usd'],
  marketCap: [_i1, '*_market_cap_in_profit'],
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
 *   btc: SeriesPattern1<?Bitcoin>,
 *   usd: SeriesPattern1<?Dollars>,
 *   marketCap: SeriesPattern1<?Dollars>,
 * }} SpotValue
 */
const _SpotValue = _s({
  btc: [_i1, '*_supply'],
  usd: [_i1, '*_supply_usd'],
  marketCap: [_i1, '*_market_cap'],
});

/**
 * @typedef {{
 *   vaulted: SpotValue,
 *   hodledOrLost: SpotValue,
 *   active: SpotValue,
 * }} CointimeSupply
 */
const _CointimeSupply = _s({
  vaulted: [_SpotValue, '*'],
  hodledOrLost: [_SpotValue, 'hodled_or_lost'],
  active: [_SpotValue, 'active'],
});

/**
 * @typedef {{
 *   supply: SpotValue,
 * }} Side
 */
const _Side = _s({
  supply: [_SpotValue, '*'],
});

/**
 * @typedef {{
 *   spendingRate: SeriesPattern1<?PerDay>,
 *   spendingExposure: SeriesPattern1<?Float>,
 *   mobility: SeriesPattern1<?Ratio>,
 *   mobile: Side,
 *   immobile: Side,
 * }} Range
 */
const _Range = _s({
  spendingRate: [_i1, '*_spending_rate'],
  spendingExposure: [_i1, '*_spending_exposure'],
  mobility: [_i1, '*_mobility'],
  mobile: [_Side, '*_mobile'],
  immobile: [_Side, '*_immobile'],
});

/**
 * @typedef {{
 *   total: SpotValue,
 *   share: SeriesPattern1<?Percent>,
 * }} BandSupply
 */
const _BandSupply = _s({
  total: [_SpotValue, '*'],
  share: [_i1, '*_supply_share'],
});

/**
 * @typedef {{
 *   supply: BandSupply,
 *   capital: BandCapital,
 * }} Band
 */
const _Band = _s({
  supply: [_BandSupply, '*'],
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
 *   burger: SeriesPattern1<?Dollars>,
 *   burgerFloor: SeriesPattern1<?Dollars>,
 *   mooijFloor: SeriesPattern1<?Dollars>,
 *   rainbowLowestBand: SeriesPattern1<?Dollars>,
 *   sminstonFloor: SeriesPattern1<?Dollars>,
 * }} PowerLaw
 */
const _PowerLaw = _s({
  burger: [_i1, 'burger_*'],
  burgerFloor: [_i1, 'burger_*_floor'],
  mooijFloor: [_i1, 'mooij_*_floor'],
  rainbowLowestBand: [_i1, 'rainbow_lowest_band'],
  sminstonFloor: [_i1, 'sminston_*_floor'],
});

/**
 * @typedef {{
 *   sma111d: SeriesPattern1<?Dollars>,
 *   sma111dRatio: SeriesPattern1<?Multiple>,
 *   sma350d: SeriesPattern1<?Dollars>,
 *   sma350dRatio: SeriesPattern1<?Multiple>,
 *   line: SeriesPattern1<?Dollars>,
 *   top: SeriesPattern1<?Multiple>,
 * }} PiCycle
 */
const _PiCycle = _s({
  sma111d: [_i1, '*_111d'],
  sma111dRatio: [_i1, '*_111d_ratio'],
  sma350d: [_i1, '*_350d'],
  sma350dRatio: [_i1, '*_350d_ratio'],
  line: [_i1, 'pi_cycle_top_line'],
  top: [_i1, 'pi_cycle_top'],
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
 *   piCycle: PiCycle,
 *   ema: Ema<PriceWithRatio>,
 *   powerLaw: PowerLaw,
 * }} Market
 */
const _Market = _s({
  ath: [_Ath, '*'],
  returns: [_ByLookbackPeriod, '*_return'],
  range: [_MarketRange, '*'],
  sma: [(c, b) => _Ema(c, b, _MarketPriceWithRatio), '*_sma'],
  piCycle: [_PiCycle, '*_sma'],
  ema: [(c, b) => _Ema(c, b, _PriceWithRatio), '*_ema'],
  powerLaw: [_PowerLaw, 'power_law'],
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
 * @typedef {{
 *   top2: SeriesPattern9<Count32>,
 *   top3: SeriesPattern9<Count32>,
 *   top4: SeriesPattern9<Count32>,
 *   top5: SeriesPattern9<Count32>,
 *   top6: SeriesPattern9<Count32>,
 * }} Concentration
 */
const _Concentration = _s({
  top2: [_i9, 'top_2_*'],
  top3: [_i9, 'top_3_*'],
  top4: [_i9, 'top_4_*'],
  top5: [_i9, 'top_5_*'],
  top6: [_i9, 'top_6_*'],
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
 * @typedef {{
 *   tx: SeriesPattern20<?FeeRate>,
 *   min: SeriesPattern1<?FeeRate>,
 *   max: SeriesPattern1<?FeeRate>,
 *   p10: SeriesPattern1<?FeeRate>,
 *   p25: SeriesPattern1<?FeeRate>,
 *   median: SeriesPattern1<?FeeRate>,
 *   p75: SeriesPattern1<?FeeRate>,
 *   p90: SeriesPattern1<?FeeRate>,
 * }} EffectiveFeeRate
 */
const _EffectiveFeeRate = _s({
  tx: [_i20, '*'],
  min: [_i1, '*_low_estimate'],
  max: [_i1, '*_high_estimate'],
  p10: [_i1, '*_p10'],
  p25: [_i1, '*_p25'],
  median: [_i1, '*_median'],
  p75: [_i1, '*_p75'],
  p90: [_i1, '*_p90'],
});

/**
 * @typedef {{
 *   tx: SeriesPattern20<Sats>,
 *   min: SeriesPattern1<Sats>,
 *   max: SeriesPattern1<Sats>,
 *   p10: SeriesPattern1<Sats>,
 *   p25: SeriesPattern1<Sats>,
 *   median: SeriesPattern1<Sats>,
 *   p75: SeriesPattern1<Sats>,
 *   p90: SeriesPattern1<Sats>,
 * }} Fee
 */
const _Fee = _s({
  tx: [_i20, '*'],
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
 *   cumulative: Value,
 *   sum: OpReturnWindows,
 *   avg: OpReturnWindows,
 * }} PoolFees
 */
const _PoolFees = _s({
  cumulative: [_Value, '*_cumulative'],
  sum: [_OpReturnWindows, '*_sum'],
  avg: [_OpReturnWindows, '*_avg'],
});

/**
 * @typedef {{
 *   cumulative: Value,
 *   sum: OpReturnWindows,
 * }} Amount
 */
const _Amount = _s({
  cumulative: [_Value, '*_cumulative'],
  sum: [_OpReturnWindows, '*_sum'],
});

/**
 * @typedef {{
 *   btc: SeriesPattern19<?Bitcoin>,
 *   usd: SeriesPattern19<?Dollars>,
 * }} OpReturnValue
 */
const _OpReturnValue = _s({
  btc: [_i19, '*'],
  usd: [_i19, '*_usd'],
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
 *   block: OpReturnValue,
 *   cumulative: Value,
 *   sum: OpReturnWindows,
 *   share: Share,
 * }} TxValue
 */
const _TxValue = _s({
  block: [_OpReturnValue, '*'],
  cumulative: [_Value, '*_cumulative'],
  sum: [_OpReturnWindows, '*_sum'],
  share: [_Share, '*_share'],
});

/**
 * @typedef {{
 *   coinbase: ValueFlow,
 *   subsidy: TxValue,
 *   puellMultiple: SeriesPattern1<?Multiple>,
 *   stockToFlow: SeriesPattern1<?Float>,
 *   stockToFlowPrice: SeriesPattern1<?Dollars>,
 *   fees: ValueFlowStats,
 *   unclaimed: ValueFlow,
 * }} Rewards
 */
const _Rewards = _s({
  coinbase: [_ValueFlow, 'coinbase'],
  subsidy: [_TxValue, 'subsidy'],
  puellMultiple: [_i1, 'puell_multiple'],
  stockToFlow: [_i1, '*'],
  stockToFlowPrice: [_i1, '*_price'],
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
  rewards: [_Rewards, 'stock_to_flow'],
  hashrate: [_Rate, 'hashrate'],
  hashPrice: [_HashPriceValue, '*_price'],
  hashValue: [_HashPriceValue, '*_value'],
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
 *   block: SeriesPattern19<Count>,
 *   sum: Windows<Count>,
 * }} NewAddressCount
 */
const _NewAddressCount = _s({
  block: [_i19, '*'],
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
 *   total: SpotValue,
 *   inProfit: SupplyInProfit,
 *   inLoss: AllSupplyInLoss,
 *   delta: Delta<?Bitcoin>,
 *   share: SeriesPattern1<?Percent>,
 * }} SharedSupply
 */
const _SharedSupply = _s({
  total: [_SpotValue, '*'],
  inProfit: [_SupplyInProfit, '*'],
  inLoss: [_AllSupplyInLoss, '*'],
  delta: [_Delta, '*_supply_delta'],
  share: [_i1, '*_supply_share'],
});

/**
 * @typedef {{
 *   supply: SharedSupply,
 *   capital: SharedCapital,
 *   unrealized: Unrealized,
 *   costBasis: CostBasis,
 * }} WeightedCohort
 */
const _WeightedCohort = _s({
  supply: [_SharedSupply, '*'],
  capital: [_SharedCapital, '*'],
  unrealized: [_Unrealized, '*'],
  costBasis: [_CostBasis, '*'],
});

/**
 * @typedef {{
 *   mobile: WeightedCohort,
 *   immobile: Side,
 * }} CoinflowCohort
 */
const _CoinflowCohort = _s({
  mobile: [_WeightedCohort, '*_mobile'],
  immobile: [_Side, '*_immobile'],
});

/**
 * @typedef {{
 *   awake: WeightedCohort,
 *   dormant: Side,
 * }} CointimeCohort
 */
const _CointimeCohort = _s({
  awake: [_WeightedCohort, '*_awake'],
  dormant: [_Side, '*_dormant'],
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
 *   total: HoldersSpotValue,
 *   inProfit: SupplyInProfit,
 *   inLoss: AllSupplyInLoss,
 *   delta: Delta<?Bitcoin>,
 * }} HoldersSupply
 */
const _HoldersSupply = _s({
  total: [_HoldersSpotValue, 'circulating_*'],
  inProfit: [_SupplyInProfit, ''],
  inLoss: [_AllSupplyInLoss, ''],
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
 *   total: SpotValue,
 *   delta: Delta<?Bitcoin>,
 *   share: SeriesPattern1<?Percent>,
 * }} CohortSupply
 */
const _CohortSupply = _s({
  total: [_SpotValue, '*'],
  delta: [_Delta, '*_supply_delta'],
  share: [_i1, '*_supply_share'],
});

/**
 * @typedef {{
 *   total: SpotValue,
 *   delta: Delta<?Bitcoin>,
 *   share: SeriesPattern1<?Percent>,
 *   inProfit: InProfit,
 *   inLoss: InLoss,
 * }} CreationSupply
 */
const _CreationSupply = _s({
  total: [_SpotValue, '*'],
  delta: [_Delta, '*_supply_delta'],
  share: [_i1, '*_supply_share'],
  inProfit: [_InProfit, '*'],
  inLoss: [_InLoss, '*'],
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
 *   nvt: SeriesPattern1<?Multiple>,
 *   nvtSignal: SeriesPattern1<?Multiple>,
 * }} Supply
 */
const _Supply = _s({
  circulating: [_i1, 'circulating_supply'],
  burned: [_ValueFlow, 'burned'],
  inflationRate: [_i1, 'inflation_rate'],
  velocity: [_Velocity, 'velocity'],
  marketCap: [_WithDelta, 'market_cap'],
  nvt: [_i1, '*'],
  nvtSignal: [_i1, '*_signal'],
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
 *   total: SpotValue,
 *   delta: Delta<?Bitcoin>,
 *   share: SeriesPattern1<?Percent>,
 *   inProfit: InProfit,
 *   inLoss: InLoss,
 *   agedOut: ValueFlow,
 * }} AgeCreationSupply
 */
const _AgeCreationSupply = _s({
  total: [_SpotValue, '*'],
  delta: [_Delta, '*_supply_delta'],
  share: [_i1, '*_supply_share'],
  inProfit: [_InProfit, '*'],
  inLoss: [_InLoss, '*'],
  agedOut: [_ValueFlow, '*_aged_out_supply'],
});

/**
 * @typedef {{
 *   block: SeriesPattern19<Bytes>,
 *   cumulative: SeriesPattern1<Bytes>,
 *   sum: Windows<Bytes>,
 *   chainShare: Share,
 * }} DataBytesSeries
 */
const _DataBytesSeries = _s({
  block: [_i19, '*_bytes'],
  cumulative: [_i1, '*_bytes_cumulative'],
  sum: [_Windows, '*_bytes_sum'],
  chainShare: [_Share, '*_chain_share'],
});

/**
 * @typedef {{
 *   flag: SeriesPattern20<Boolean>,
 *   block: SeriesPattern19<Count>,
 *   cumulative: SeriesPattern1<Count>,
 *   sum: Windows<Count>,
 *   share: Share,
 *   value: TxValue,
 *   fees: FeesSeries,
 * }} Inscription
 */
const _Inscription = _s({
  flag: [_i20, 'has_*'],
  block: [_i19, '*_tx_count'],
  cumulative: [_i1, '*_tx_count_cumulative'],
  sum: [_Windows, '*_tx_count_sum'],
  share: [_Share, '*_tx_share'],
  value: [_TxValue, '*_tx_value'],
  fees: [_FeesSeries, '*'],
});

/**
 * @typedef {{
 *   tx: SeriesPattern20<VSize>,
 *   min: SeriesPattern1<VSize>,
 *   max: SeriesPattern1<VSize>,
 *   p10: SeriesPattern1<VSize>,
 *   p25: SeriesPattern1<VSize>,
 *   median: SeriesPattern1<VSize>,
 *   p75: SeriesPattern1<VSize>,
 *   p90: SeriesPattern1<VSize>,
 *   cumulative: SeriesPattern1<VSize>,
 *   sum: Windows<VSize>,
 * }} Vsize
 */
const _Vsize = _s({
  tx: [_i20, '*'],
  min: [_i1, '*_min'],
  max: [_i1, '*_max'],
  p10: [_i1, '*_p10'],
  p25: [_i1, '*_p25'],
  median: [_i1, '*_median'],
  p75: [_i1, '*_p75'],
  p90: [_i1, '*_p90'],
  cumulative: [_i1, '*_cumulative'],
  sum: [_Windows, '*_sum'],
});

/**
 * @typedef {{
 *   flag: SeriesPattern20<Boolean>,
 *   block: SeriesPattern19<Count>,
 *   cumulative: SeriesPattern1<Count>,
 *   sum: Windows<Count>,
 *   share: Share,
 *   value: TxValue,
 * }} TransactionsFlagged
 */
const _TransactionsFlagged = _s({
  flag: [_i20, 'is_*'],
  block: [_i19, '*_tx_count'],
  cumulative: [_i1, '*_tx_count_cumulative'],
  sum: [_Windows, '*_tx_count_sum'],
  share: [_Share, '*_tx_share'],
  value: [_TxValue, '*_tx_value'],
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
 *   block: SeriesPattern19<Count>,
 *   cumulative: SeriesPattern1<Count>,
 *   sum: Windows<Count>,
 *   share: Share,
 *   value: TxValue,
 * }} TxCount
 */
const _TxCount = _s({
  block: [_i19, '*_count'],
  cumulative: [_i1, '*_count_cumulative'],
  sum: [_Windows, '*_count_sum'],
  share: [_Share, '*_share'],
  value: [_TxValue, '*_value'],
});

/**
 * @typedef {{
 *   flag: SeriesPattern20<Boolean>,
 *   block: SeriesPattern19<Count>,
 *   cumulative: SeriesPattern1<Count>,
 *   sum: Windows<Count>,
 *   share: Share,
 *   value: TxValue,
 * }} Flagged
 */
const _Flagged = _s({
  flag: [_i20, 'has_*'],
  block: [_i19, '*_tx_count'],
  cumulative: [_i1, '*_tx_count_cumulative'],
  sum: [_Windows, '*_tx_count_sum'],
  share: [_Share, '*_tx_share'],
  value: [_TxValue, '*_tx_value'],
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
 *   block: SeriesPattern19<?Percent>,
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
  block: [_i19, '*'],
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
 * @template A
 * @typedef {{
 *   block: SeriesPattern19<A>,
 *   cumulative: SeriesPattern1<A>,
 *   sum: Windows<A>,
 * }} Flow
 */
const _Flow = _s({
  block: [_i19, '*'],
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
  awake: [_Side, '*_awake'],
  dormant: [_Side, '*_dormant'],
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
  supply: [_CohortSupply, '*'],
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
  supply: [_CohortSupply, '*'],
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
 *   amountGini: SeriesPattern9<?Float>,
 * }} Utxos
 */
const _Utxos = _s({
  amounts: [(c, b) => _AmountRange(c, b, _Zero, _Cohort), '*'],
  types: [_UtxosSpendableType, 'script'],
  amountGini: [_i9, 'utxo_*_gini'],
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
  supply: [f0, '*'],
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
 *   mvrvZ: SeriesPattern9<?Float>,
 *   sellerExhaustion: SeriesPattern9<?Float>,
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
  mvrvZ: [_i9, 'mvrv_z'],
  sellerExhaustion: [_i9, 'seller_exhaustion'],
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
 *   ageRanges: AgeRange<Range>,
 *   cohorts: AgeAggregate<CoinflowCohort>,
 * }} Coinflow
 */
const _Coinflow = _s({
  ageRanges: [(c, b) => _AgeRange(c, b, _Range), '*'],
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
  supply: [_CointimeSupply, 'vaulted'],
  value: [_CointimeValue, 'cointime_value'],
  caps: [_Caps, 'cap'],
  prices: [_Prices, 'price'],
  adjusted: [_Adjusted, 'cointime_adjusted'],
  reserveRisk: [_ReserveRisk, 'reserve_risk'],
});

/**
 * @typedef {{
 *   ranges: AgeRange<CreationCohort<AgeCreationSupply, CreationCapital, AgeCreationActivity, CreationRealized, CreationUnrealized, CohortCostBasis>>,
 *   rhodlRatio: SeriesPattern1<?Ratio>,
 *   halvingEpochs: ByHalvingEpoch,
 *   vintages: Vintage,
 * }} Age
 */
const _Age = _s({
  ranges: [(c, b) => _AgeRange(c, b, (c, b) => _CreationCohort(c, b, _AgeCreationSupply, _CreationCapital, _AgeCreationActivity, _CreationRealized, _CreationUnrealized, _CohortCostBasis)), '*'],
  rhodlRatio: [_i1, 'rhodl_ratio'],
  halvingEpochs: [_ByHalvingEpoch, 'halving_epoch'],
  vintages: [_Vintage, 'vintage'],
});

/**
 * @typedef {{
 *   taggedBlockCount: Flow<Count>,
 * }} StratumV2
 */
const _StratumV2 = _s({
  taggedBlockCount: [_Flow, '*'],
});

/**
 * @typedef {{
 *   outputCount: Flow<Count>,
 * }} OutputCount
 */
const _OutputCount = _s({
  outputCount: [_Flow, '*'],
});

/**
 * @typedef {{
 *   over42Bytes: OutputCount,
 *   over520Bytes: OutputCount,
 *   over10kBytes: OutputCount,
 * }} Sizes
 */
const _Sizes = _s({
  over42Bytes: [_OutputCount, '*_42_bytes_output_count'],
  over520Bytes: [_OutputCount, '*_520_bytes_output_count'],
  over10kBytes: [_OutputCount, '*_10k_bytes_output_count'],
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
 *   withinPreV30RelayLimits: Bucket,
 *   beyondPreV30RelayLimits: Bucket,
 *   oversized: Bucket,
 *   multiple: Bucket,
 * }} Policy
 */
const _Policy = _s({
  withinPreV30RelayLimits: [_Bucket, '*_within_pre_v30_relay_limits'],
  beyondPreV30RelayLimits: [_Bucket, '*_beyond_pre_v30_relay_limits'],
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
 *   bip47: Bucket,
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
  bip47: [_Bucket, '*_bip47'],
  bareHash: [_Bucket, '*_bare_hash'],
  text: [_Bucket, '*_text'],
  empty: [_Bucket, '*_empty'],
  unknown: [_Bucket, '*_unknown'],
});

/**
 * @typedef {{
 *   flag: SeriesPattern20<Boolean>,
 *   outputCount: Flow<Count>,
 *   dataBytes: DataBytesSeries,
 *   txCount: Flow<Count>,
 *   txShare: Share,
 *   txVsize: Flow<VSize>,
 *   fees: FeesSeries,
 *   protocols: ByKind,
 *   policies: Policy,
 *   sizes: Sizes,
 * }} OpReturn
 */
const _OpReturn = _s({
  flag: [_i20, 'has_*'],
  outputCount: [_Flow, '*_output_count'],
  dataBytes: [_DataBytesSeries, '*_data'],
  txCount: [_Flow, '*_tx_count'],
  txShare: [_Share, '*_tx_share'],
  txVsize: [_Flow, '*_tx_vsize'],
  fees: [_FeesSeries, '*'],
  protocols: [_ByKind, '*'],
  policies: [_Policy, '*'],
  sizes: [_Sizes, '*_data_over'],
});

/**
 * @typedef {{
 *   onlySegwit: Flow<Count>,
 *   onlyLegacy: Flow<Count>,
 *   segwitAndLegacy: Flow<Count>,
 * }} Spending
 */
const _Spending = _s({
  onlySegwit: [_Flow, 'only_segwit_*'],
  onlyLegacy: [_Flow, 'only_legacy_*'],
  segwitAndLegacy: [_Flow, 'segwit_and_legacy_*'],
});

/**
 * @typedef {{
 *   height: Flow<Count>,
 *   time: Flow<Count>,
 *   unenforced: Flow<Count>,
 *   relativeHeight: Flow<Count>,
 *   relativeTime: Flow<Count>,
 * }} Timelocks
 */
const _Timelocks = _s({
  height: [_Flow, 'height_*'],
  time: [_Flow, 'time_*'],
  unenforced: [_Flow, 'unenforced_*'],
  relativeHeight: [_Flow, 'relative_height_*'],
  relativeTime: [_Flow, 'relative_time_*'],
});

/**
 * @typedef {{
 *   count: Flow<Count>,
 *   segwitSpending: Flow<Count>,
 *   rbfSignaling: Flow<Count>,
 * }} Payments
 */
const _Payments = _s({
  count: [_Flow, '*'],
  segwitSpending: [_Flow, 'segwit_spending_*'],
  rbfSignaling: [_Flow, 'rbf_signaling_*'],
});

/**
 * @typedef {{
 *   under0p1SatVb: Flow<VSize>,
 *   _0p1To0p25SatVb: Flow<VSize>,
 *   _0p25To0p5SatVb: Flow<VSize>,
 *   _0p5To1SatVb: Flow<VSize>,
 *   _1To2SatVb: Flow<VSize>,
 *   _2To5SatVb: Flow<VSize>,
 *   _5To10SatVb: Flow<VSize>,
 *   _10To25SatVb: Flow<VSize>,
 *   _25To50SatVb: Flow<VSize>,
 *   _50To100SatVb: Flow<VSize>,
 *   _100To250SatVb: Flow<VSize>,
 *   _250To500SatVb: Flow<VSize>,
 *   _500To1kSatVb: Flow<VSize>,
 *   over1kSatVb: Flow<VSize>,
 * }} VsizeByEffectiveFeeRate
 */
const _VsizeByEffectiveFeeRate = _s({
  under0p1SatVb: [_Flow, 'effective_fee_rate_under_0p1_*'],
  _0p1To0p25SatVb: [_Flow, 'effective_fee_rate_0p1_to_0p25_*'],
  _0p25To0p5SatVb: [_Flow, 'effective_fee_rate_0p25_to_0p5_*'],
  _0p5To1SatVb: [_Flow, 'effective_fee_rate_0p5_to_1_*'],
  _1To2SatVb: [_Flow, 'effective_fee_rate_1_to_2_*'],
  _2To5SatVb: [_Flow, 'effective_fee_rate_2_to_5_*'],
  _5To10SatVb: [_Flow, 'effective_fee_rate_5_to_10_*'],
  _10To25SatVb: [_Flow, 'effective_fee_rate_10_to_25_*'],
  _25To50SatVb: [_Flow, 'effective_fee_rate_25_to_50_*'],
  _50To100SatVb: [_Flow, 'effective_fee_rate_50_to_100_*'],
  _100To250SatVb: [_Flow, 'effective_fee_rate_100_to_250_*'],
  _250To500SatVb: [_Flow, 'effective_fee_rate_250_to_500_*'],
  _500To1kSatVb: [_Flow, 'effective_fee_rate_500_to_1k_*'],
  over1kSatVb: [_Flow, 'effective_fee_rate_over_1k_*'],
});

/**
 * @typedef {{
 *   under0p1SatVb: Flow<Count>,
 *   _0p1To0p25SatVb: Flow<Count>,
 *   _0p25To0p5SatVb: Flow<Count>,
 *   _0p5To1SatVb: Flow<Count>,
 *   _1To2SatVb: Flow<Count>,
 *   _2To5SatVb: Flow<Count>,
 *   _5To10SatVb: Flow<Count>,
 *   _10To25SatVb: Flow<Count>,
 *   _25To50SatVb: Flow<Count>,
 *   _50To100SatVb: Flow<Count>,
 *   _100To250SatVb: Flow<Count>,
 *   _250To500SatVb: Flow<Count>,
 *   _500To1kSatVb: Flow<Count>,
 *   over1kSatVb: Flow<Count>,
 * }} TxCountByFeeRate
 */
const _TxCountByFeeRate = _s({
  under0p1SatVb: [_Flow, 'fee_rate_under_0p1_*'],
  _0p1To0p25SatVb: [_Flow, 'fee_rate_0p1_to_0p25_*'],
  _0p25To0p5SatVb: [_Flow, 'fee_rate_0p25_to_0p5_*'],
  _0p5To1SatVb: [_Flow, 'fee_rate_0p5_to_1_*'],
  _1To2SatVb: [_Flow, 'fee_rate_1_to_2_*'],
  _2To5SatVb: [_Flow, 'fee_rate_2_to_5_*'],
  _5To10SatVb: [_Flow, 'fee_rate_5_to_10_*'],
  _10To25SatVb: [_Flow, 'fee_rate_10_to_25_*'],
  _25To50SatVb: [_Flow, 'fee_rate_25_to_50_*'],
  _50To100SatVb: [_Flow, 'fee_rate_50_to_100_*'],
  _100To250SatVb: [_Flow, 'fee_rate_100_to_250_*'],
  _250To500SatVb: [_Flow, 'fee_rate_250_to_500_*'],
  _500To1kSatVb: [_Flow, 'fee_rate_500_to_1k_*'],
  over1kSatVb: [_Flow, 'fee_rate_over_1k_*'],
});

/**
 * @typedef {{
 *   txCount: Flow<Count>,
 * }} ZeroFee
 */
const _ZeroFee = _s({
  txCount: [_Flow, '*'],
});

/**
 * @typedef {{
 *   fee: Fee,
 *   feeRate: SeriesPattern20<?FeeRate>,
 *   effectiveFeeRate: EffectiveFeeRate,
 *   cpfpParent: TransactionsFlagged,
 *   cpfpChild: TransactionsFlagged,
 *   zeroFee: ZeroFee,
 *   txCountByFeeRate: TxCountByFeeRate,
 *   vsizeByEffectiveFeeRate: VsizeByEffectiveFeeRate,
 * }} Fees
 */
const _Fees = _s({
  fee: [_Fee, '*_fee'],
  feeRate: [_i20, '*_fee_rate'],
  effectiveFeeRate: [_EffectiveFeeRate, '*_effective_fee_rate'],
  cpfpParent: [_TransactionsFlagged, 'cpfp_parent'],
  cpfpChild: [_TransactionsFlagged, 'cpfp_child'],
  zeroFee: [_ZeroFee, 'zero_fee_*_count'],
  txCountByFeeRate: [_TxCountByFeeRate, 'sat_vb_*_count'],
  vsizeByEffectiveFeeRate: [_VsizeByEffectiveFeeRate, 'sat_vb_*_vsize'],
});

/**
 * @typedef {{
 *   oneInputOneOutput: Flow<Count>,
 *   oneInputTwoOutput: Flow<Count>,
 *   bip69: Flow<Count>,
 *   bip69ExpectedByChance: Flow<?ExpectedCount>,
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
  oneInputOneOutput: [_Flow, 'one_input_one_output_*_count'],
  oneInputTwoOutput: [_Flow, 'one_input_two_output_*_count'],
  bip69: [_Flow, 'bip69_*_count'],
  bip69ExpectedByChance: [_Flow, 'bip69_expected_by_chance_*_count'],
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
 * @typedef {{
 *   p2pk: Flow<Count>,
 *   p2pkh: Flow<Count>,
 *   p2ms: Flow<Count>,
 *   p2sh: Flow<Count>,
 *   p2wpkh: Flow<Count>,
 *   p2wsh: Flow<Count>,
 *   p2tr: Flow<Count>,
 *   p2a: Flow<Count>,
 *   emptyScript: Flow<Count>,
 *   unknownScript: Flow<Count>,
 *   opReturn: Flow<Count>,
 * }} ByType
 */
const _ByType = _s({
  p2pk: [_Flow, 'coinbase_p2pk_*'],
  p2pkh: [_Flow, 'coinbase_p2pkh_*'],
  p2ms: [_Flow, 'coinbase_p2ms_*'],
  p2sh: [_Flow, 'coinbase_p2sh_*'],
  p2wpkh: [_Flow, 'coinbase_p2wpkh_*'],
  p2wsh: [_Flow, 'coinbase_p2wsh_*'],
  p2tr: [_Flow, 'coinbase_p2tr_*'],
  p2a: [_Flow, 'coinbase_p2a_*'],
  emptyScript: [_Flow, 'coinbase_empty_script_*'],
  unknownScript: [_Flow, 'coinbase_unknown_script_*'],
  opReturn: [_Flow, 'coinbase_op_return_*'],
});

/**
 * @typedef {{
 *   count: Flow<Count>,
 *   types: ByType,
 * }} Coinbase
 */
const _Coinbase = _s({
  count: [_Flow, 'coinbase_*'],
  types: [_ByType, '*'],
});

/**
 * @typedef {{
 *   value: ValueFlow,
 *   count: Flow<Count>,
 *   share: Share,
 *   txCount: Flow<Count>,
 *   txShare: Share,
 * }} OutputTypeSeries
 */
const _OutputTypeSeries = _s({
  value: [_ValueFlow, '*_value'],
  count: [_Flow, '*_count'],
  share: [_Share, '*_share'],
  txCount: [_Flow, '*_tx_count'],
  txShare: [_Share, '*_tx_share'],
});

/**
 * @typedef {{
 *   _1Block: Flow<Count>,
 *   _6Blocks: Flow<Count>,
 *   _144Blocks: Flow<Count>,
 *   _2016Blocks: Flow<Count>,
 * }} SpentWithin
 */
const _SpentWithin = _s({
  _1Block: [_Flow, 'within_1_block_*'],
  _6Blocks: [_Flow, 'within_6_blocks_*'],
  _144Blocks: [_Flow, 'within_144_blocks_*'],
  _2016Blocks: [_Flow, 'within_2016_blocks_*'],
});

/**
 * @typedef {{
 *   checksig: Flow<Count>,
 *   checksigverify: Flow<Count>,
 *   checkmultisig: Flow<Count>,
 *   checksigadd: Flow<Count>,
 *   checklocktimeverify: Flow<Count>,
 *   checksequenceverify: Flow<Count>,
 *   hash160: Flow<Count>,
 *   sha256: Flow<Count>,
 *   if: Flow<Count>,
 *   notif: Flow<Count>,
 * }} Opcodes
 */
const _Opcodes = _s({
  checksig: [_Flow, 'op_checksig_*'],
  checksigverify: [_Flow, 'op_checksigverify_*'],
  checkmultisig: [_Flow, 'op_checkmultisig_*'],
  checksigadd: [_Flow, 'op_checksigadd_*'],
  checklocktimeverify: [_Flow, 'op_checklocktimeverify_*'],
  checksequenceverify: [_Flow, 'op_checksequenceverify_*'],
  hash160: [_Flow, 'op_hash160_*'],
  sha256: [_Flow, 'op_sha256_*'],
  if: [_Flow, 'op_if_*'],
  notif: [_Flow, 'op_notif_*'],
});

/**
 * @typedef {{
 *   p2sh: Flow<Count>,
 *   p2shP2wsh: Flow<Count>,
 *   p2wsh: Flow<Count>,
 * }} Multisig
 */
const _Multisig = _s({
  p2sh: [_Flow, 'p2sh_*'],
  p2shP2wsh: [_Flow, 'p2sh_p2wsh_*'],
  p2wsh: [_Flow, 'p2wsh_*'],
});

/**
 * @typedef {{
 *   p2wpkh: Flow<Count>,
 *   p2wsh: Flow<Count>,
 * }} NestedSegwit
 */
const _NestedSegwit = _s({
  p2wpkh: [_Flow, 'p2sh_p2wpkh_*'],
  p2wsh: [_Flow, 'p2sh_p2wsh_*'],
});

/**
 * @typedef {{
 *   compressed: Flow<Count>,
 *   uncompressed: Flow<Count>,
 * }} Pubkeys
 */
const _Pubkeys = _s({
  compressed: [_Flow, 'compressed_*'],
  uncompressed: [_Flow, 'uncompressed_*'],
});

/**
 * @typedef {{
 *   all: Flow<Count>,
 *   none: Flow<Count>,
 *   single: Flow<Count>,
 *   allAnyoneCanPay: Flow<Count>,
 *   noneAnyoneCanPay: Flow<Count>,
 *   singleAnyoneCanPay: Flow<Count>,
 * }} EcdsaSighashes
 */
const _EcdsaSighashes = _s({
  all: [_Flow, 'sighash_all_*'],
  none: [_Flow, 'sighash_none_*'],
  single: [_Flow, 'sighash_single_*'],
  allAnyoneCanPay: [_Flow, 'sighash_all_anyone_can_pay_*'],
  noneAnyoneCanPay: [_Flow, 'sighash_none_anyone_can_pay_*'],
  singleAnyoneCanPay: [_Flow, 'sighash_single_anyone_can_pay_*'],
});

/**
 * @typedef {{
 *   _70Bytes: Flow<Count>,
 *   _71Bytes: Flow<Count>,
 *   _72Bytes: Flow<Count>,
 *   _73Bytes: Flow<Count>,
 *   _74Bytes: Flow<Count>,
 *   over74Bytes: Flow<Count>,
 * }} EcdsaLengths
 */
const _EcdsaLengths = _s({
  _70Bytes: [_Flow, 'ecdsa_70_*'],
  _71Bytes: [_Flow, 'ecdsa_71_*'],
  _72Bytes: [_Flow, 'ecdsa_72_*'],
  _73Bytes: [_Flow, 'ecdsa_73_*'],
  _74Bytes: [_Flow, 'ecdsa_74_*'],
  over74Bytes: [_Flow, 'ecdsa_over_74_*'],
});

/**
 * @typedef {{
 *   count: Flow<Count>,
 *   bytes: Flow<Bytes>,
 *   strictDer: Flow<Count>,
 *   lowR: Flow<Count>,
 *   lowS: Flow<Count>,
 *   lowRLowS: Flow<Count>,
 *   length: EcdsaLengths,
 *   sighash: EcdsaSighashes,
 * }} EcdsaSignatures
 */
const _EcdsaSignatures = _s({
  count: [_Flow, 'ecdsa_*_count'],
  bytes: [_Flow, 'ecdsa_*_bytes'],
  strictDer: [_Flow, 'strict_der_ecdsa_*_count'],
  lowR: [_Flow, 'low_r_ecdsa_*_count'],
  lowS: [_Flow, 'low_s_ecdsa_*_count'],
  lowRLowS: [_Flow, 'low_r_low_s_ecdsa_*_count'],
  length: [_EcdsaLengths, 'bytes_*_count'],
  sighash: [_EcdsaSighashes, 'ecdsa_*_count'],
});

/**
 * @typedef {{
 *   count: Flow<Count>,
 *   bytes: Flow<Bytes>,
 * }} SchnorrSignatures
 */
const _SchnorrSignatures = _s({
  count: [_Flow, '*_count'],
  bytes: [_Flow, '*_bytes'],
});

/**
 * @typedef {{
 *   schnorr: SchnorrSignatures,
 *   ecdsa: EcdsaSignatures,
 * }} InputSignatures
 */
const _InputSignatures = _s({
  schnorr: [_SchnorrSignatures, 'schnorr_*'],
  ecdsa: [_EcdsaSignatures, '*'],
});

/**
 * @typedef {{
 *   count: Flow<Count>,
 *   share: Share,
 *   txCount: Flow<Count>,
 *   txShare: Share,
 * }} TypeSeries
 */
const _TypeSeries = _s({
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
 *   opReturn: TypeSeries,
 * }} Types
 */
const _Types = _s({
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
  opReturn: [_TypeSeries, 'op_return_*'],
});

/**
 * @typedef {{
 *   p2pk: TypeSeries,
 *   p2pkh: TypeSeries,
 *   p2ms: TypeSeries,
 *   p2sh: TypeSeries,
 *   p2wpkh: TypeSeries,
 *   p2wsh: TypeSeries,
 *   p2tr: TypeSeries,
 *   p2a: TypeSeries,
 *   emptyScript: TypeSeries,
 *   unknownScript: TypeSeries,
 * }} SpendableType
 */
const _SpendableType = _s({
  p2pk: [_TypeSeries, 'p2pk_*'],
  p2pkh: [_TypeSeries, 'p2pkh_*'],
  p2ms: [_TypeSeries, 'p2ms_*'],
  p2sh: [_TypeSeries, 'p2sh_*'],
  p2wpkh: [_TypeSeries, 'p2wpkh_*'],
  p2wsh: [_TypeSeries, 'p2wsh_*'],
  p2tr: [_TypeSeries, 'p2tr_*'],
  p2a: [_TypeSeries, 'p2a_*'],
  emptyScript: [_TypeSeries, 'empty_script_*'],
  unknownScript: [_TypeSeries, 'unknown_script_*'],
});

/**
 * @template A, B
 * @typedef {{
 *   block: SeriesPattern19<A>,
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
  block: [_i19, '*'],
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
 *   spendingTxinIndex: SeriesPattern24<TxInIndex>,
 *   count: FlowStats<Count, ?CountFract>,
 *   perSecond: Windows<?PerSecond>,
 *   nonOpReturnCount: Flow<Count>,
 *   types: Types,
 *   opReturnValue: ValueFlow,
 *   coinbase: Coinbase,
 *   pubkeys: Pubkeys,
 * }} Outputs
 */
const _Outputs = _s({
  spendingTxinIndex: [_i24, 'spending_txin_index'],
  count: [_FlowStats, '*_count'],
  perSecond: [_Windows, 'outputs_per_second'],
  nonOpReturnCount: [_Flow, 'non_op_return_*_count'],
  types: [_Types, '*'],
  opReturnValue: [_ValueFlow, 'op_return_value'],
  coinbase: [_Coinbase, '*_count'],
  pubkeys: [_Pubkeys, 'pubkey_in_*_count'],
});

/**
 * @template A, B
 * @typedef {{
 *   block: SeriesPattern19<A>,
 *   cumulative: SeriesPattern1<A>,
 *   sum: Windows<A>,
 *   avg: Windows<B>,
 * }} Averaged
 */
const _Averaged = _s({
  block: [_i19, '*'],
  cumulative: [_i1, '*_cumulative'],
  sum: [_Windows, '*_sum'],
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
  supply: [_PredicateSupply, '*_respent_address'],
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
 * }} AddressesReused
 */
const _AddressesReused = _s({
  count: [_ReusedCount, '*'],
  supply: [_PredicateSupply, '*_reused_address'],
  events: [_AddressesReuseEvents, '*'],
});

/**
 * @typedef {{
 *   count: ReusedCount,
 *   supply: PredicateSupply,
 *   events: AddressesReuseEvents,
 *   direct: OutputCount,
 * }} Reused
 */
const _Reused = _s({
  count: [_ReusedCount, ''],
  supply: [_PredicateSupply, 'reused_*'],
  events: [_AddressesReuseEvents, ''],
  direct: [_OutputCount, 'output_back_to_input_*_count'],
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
 *   reused: AddressesReused,
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
  reused: [_AddressesReused, '*'],
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
  reused: [_Reused, '*'],
  respent: [_Respent, ''],
  exposed: [_Exposed, ''],
  supply: [_CohortSupply, '*'],
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
 * }} Antpool
 */
const _Antpool = _s({
  blockCount: [_CumulativeFlow, '*'],
});

/**
 * @typedef {{
 *   antpool: Antpool,
 * }} Groups
 */
const _Groups = _s({
  antpool: [_Antpool, '*'],
});

/**
 * @typedef {{
 *   blockCount: CumulativeFlow,
 *   blockShare: Share,
 *   coinbase: Amount,
 *   fees: PoolFees,
 *   emptyBlockCount: CumulativeFlow,
 *   bip54CoinbaseLocktimeBlockCount: CumulativeFlow,
 *   nonzeroCoinbaseLocktimeBlockCount: CumulativeFlow,
 *   feeRateUnder0p1SatVbTxCount: CumulativeFlow,
 *   hashrate: Windows<?Hashrate>,
 * }} Pool
 */
const _Pool = _s({
  blockCount: [_CumulativeFlow, '*_block_count'],
  blockShare: [_Share, '*_block_share'],
  coinbase: [_Amount, '*_coinbase'],
  fees: [_PoolFees, '*_fees'],
  emptyBlockCount: [_CumulativeFlow, '*_empty_block_count'],
  bip54CoinbaseLocktimeBlockCount: [_CumulativeFlow, '*_bip54_coinbase_locktime_block_count'],
  nonzeroCoinbaseLocktimeBlockCount: [_CumulativeFlow, '*_nonzero_coinbase_locktime_block_count'],
  feeRateUnder0p1SatVbTxCount: [_CumulativeFlow, '*_fee_rate_under_0p1_sat_vb_tx_count'],
  hashrate: [_Windows, '*_hashrate'],
});

/**
 * @typedef {{
 *   blockPool: SeriesPattern19<PoolSlug>,
 *   unknown: Pool,
 *   blockfills: Pool,
 *   ultimuspool: Pool,
 *   terrapool: Pool,
 *   luxor: Pool,
 *   _1thash: Pool,
 *   cloverpool: Pool,
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
 *   trustpool: Pool,
 *   concentration: Concentration,
 *   concentrationGrouped: Concentration,
 *   groups: Groups,
 *   stratumV2: StratumV2,
 * }} Pools
 */
const _Pools = _s({
  blockPool: [_i19, 'block_*'],
  unknown: [_Pool, '*_unknown'],
  blockfills: [_Pool, '*_blockfills'],
  ultimuspool: [_Pool, '*_ultimuspool'],
  terrapool: [_Pool, '*_terrapool'],
  luxor: [_Pool, '*_luxor'],
  _1thash: [_Pool, '*_1thash'],
  cloverpool: [_Pool, '*_cloverpool'],
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
  trustpool: [_Pool, '*_trustpool'],
  concentration: [_Concentration, 'pools_block_count'],
  concentrationGrouped: [_Concentration, 'pools_antpool_group_as_one_block_count'],
  groups: [_Groups, '*_antpool_group_block_count'],
  stratumV2: [_StratumV2, 'stratum_v2_tagged_block_count'],
});

/**
 * @typedef {{
 *   count: FlowStats<Count, ?CountFract>,
 *   perSecond: Windows<?PerSecond>,
 *   types: SpendableType,
 *   signatures: InputSignatures,
 *   pubkeys: Pubkeys,
 *   nestedSegwit: NestedSegwit,
 *   multisig: Multisig,
 *   opcodes: Opcodes,
 *   p2trScriptPath: Flow<Count>,
 *   inscriptions: Flow<Count>,
 *   spentWithin: SpentWithin,
 *   sameBlock: Flow<Count>,
 *   coinbaseWitness: CumulativeFlow,
 *   spentTxoutHeight: SeriesPattern22<Height>,
 * }} Inputs
 */
const _Inputs = _s({
  count: [_FlowStats, 'input_*'],
  perSecond: [_Windows, 'inputs_per_second'],
  types: [_SpendableType, 'input'],
  signatures: [_InputSignatures, 'signature'],
  pubkeys: [_Pubkeys, 'pubkey_in_input_*'],
  nestedSegwit: [_NestedSegwit, 'input_*'],
  multisig: [_Multisig, 'multisig_input_*'],
  opcodes: [_Opcodes, 'in_input_*'],
  p2trScriptPath: [_Flow, 'p2tr_script_path_input_*'],
  inscriptions: [_Flow, 'inscription_input_*'],
  spentWithin: [_SpentWithin, 'spent_utxo_*'],
  sameBlock: [_Flow, 'same_block_spent_utxo_*'],
  coinbaseWitness: [_CumulativeFlow, 'coinbase_witness_block_*'],
  spentTxoutHeight: [_i22, 'spent_txout_height'],
});

/**
 * @typedef {{
 *   nonzero: CumulativeFlow,
 *   bip54: CumulativeFlow,
 * }} CoinbaseLocktime
 */
const _CoinbaseLocktime = _s({
  nonzero: [_CumulativeFlow, 'nonzero_*'],
  bip54: [_CumulativeFlow, 'bip54_*'],
});

/**
 * @typedef {{
 *   bit4: CumulativeFlow,
 * }} VersionBits
 */
const _VersionBits = _s({
  bit4: [_CumulativeFlow, '*'],
});

/**
 * @typedef {{
 *   count: CumulativeFlow,
 *   interval: Averaged<Seconds, ?SecondsFract>,
 *   size: FlowStats<Bytes, ?BytesFract>,
 *   weight: FlowStats<Weight, ?WeightFract>,
 *   sigopCost: Flow<SigOps>,
 *   fullness: Fullness,
 *   difficulty: BlocksDifficulty,
 *   halving: Halving,
 *   emptyCount: CumulativeFlow,
 *   work: Flow<?Work>,
 *   versionBits: VersionBits,
 *   coinbaseLocktime: CoinbaseLocktime,
 * }} Blocks
 */
const _Blocks = _s({
  count: [_CumulativeFlow, '*_count'],
  interval: [_Averaged, '*_interval'],
  size: [_FlowStats, '*_size'],
  weight: [_FlowStats, '*_weight'],
  sigopCost: [_Flow, '*_sigop_cost'],
  fullness: [_Fullness, '*_fullness'],
  difficulty: [_BlocksDifficulty, 'difficulty'],
  halving: [_Halving, 'halving'],
  emptyCount: [_CumulativeFlow, 'empty_*_count'],
  work: [_Flow, '*_work'],
  versionBits: [_VersionBits, 'version_bit_4_*_count'],
  coinbaseLocktime: [_CoinbaseLocktime, 'coinbase_locktime_*_count'],
});

/**
 * @typedef {{
 *   open: SeriesPattern3<?Dollars>,
 *   high: SeriesPattern3<?Dollars>,
 *   low: SeriesPattern3<?Dollars>,
 *   close: SeriesPattern3<?Dollars>,
 *   ohlc: SeriesPattern3<OHLCDollars>,
 *   spot: SeriesPattern1<?Dollars>,
 *   satsPerDollar: SeriesPattern2<Sats>,
 * }} Price
 */
const _Price = _s({
  open: [_i3, '*_open'],
  high: [_i3, '*_high'],
  low: [_i3, '*_low'],
  close: [_i3, '*_close'],
  ohlc: [_i3, '*_ohlc'],
  spot: [_i1, '*'],
  satsPerDollar: [_i2, 'sats_per_dollar'],
});

/**
 * @typedef {{
 *   monotonic: SeriesPattern19<Timestamp>,
 *   periodStart: SeriesPattern3<Timestamp>,
 * }} Timestamps
 */
const _Timestamps = _s({
  monotonic: [_i19, '*_monotonic'],
  periodStart: [_i3, '*'],
});

/**
 * @typedef {{
 *   identity: SeriesPattern23<TxOutIndex>,
 * }} TxoutIndex
 */
const _TxoutIndex = _s({
  identity: [_i23, '*'],
});

/**
 * @typedef {{
 *   identity: SeriesPattern21<TxInIndex>,
 * }} TxinIndex
 */
const _TxinIndex = _s({
  identity: [_i21, '*'],
});

/**
 * @typedef {{
 *   identity: SeriesPattern20<TxIndex>,
 *   inputCount: SeriesPattern20<Count>,
 *   outputCount: SeriesPattern20<Count>,
 * }} MappingsTxIndex
 */
const _MappingsTxIndex = _s({
  identity: [_i20, '*_index'],
  inputCount: [_i20, '*_input_count'],
  outputCount: [_i20, '*_output_count'],
});

/**
 * @typedef {{
 *   date: SeriesPattern16<Date>,
 *   firstHeight: SeriesPattern16<Height>,
 * }} MappingsYear10
 */
const _MappingsYear10 = _s({
  date: [_i16, '*'],
  firstHeight: [_i16, 'first_height'],
});

/**
 * @typedef {{
 *   date: SeriesPattern15<Date>,
 *   firstHeight: SeriesPattern15<Height>,
 * }} MappingsYear1
 */
const _MappingsYear1 = _s({
  date: [_i15, '*'],
  firstHeight: [_i15, 'first_height'],
});

/**
 * @typedef {{
 *   date: SeriesPattern14<Date>,
 *   firstHeight: SeriesPattern14<Height>,
 * }} MappingsMonth6
 */
const _MappingsMonth6 = _s({
  date: [_i14, '*'],
  firstHeight: [_i14, 'first_height'],
});

/**
 * @typedef {{
 *   date: SeriesPattern13<Date>,
 *   firstHeight: SeriesPattern13<Height>,
 * }} MappingsMonth3
 */
const _MappingsMonth3 = _s({
  date: [_i13, '*'],
  firstHeight: [_i13, 'first_height'],
});

/**
 * @typedef {{
 *   date: SeriesPattern12<Date>,
 *   firstHeight: SeriesPattern12<Height>,
 * }} MappingsMonth1
 */
const _MappingsMonth1 = _s({
  date: [_i12, '*'],
  firstHeight: [_i12, 'first_height'],
});

/**
 * @typedef {{
 *   date: SeriesPattern11<Date>,
 *   firstHeight: SeriesPattern11<Height>,
 * }} MappingsWeek1
 */
const _MappingsWeek1 = _s({
  date: [_i11, '*'],
  firstHeight: [_i11, 'first_height'],
});

/**
 * @typedef {{
 *   date: SeriesPattern10<Date>,
 *   firstHeight: SeriesPattern10<Height>,
 * }} MappingsDay3
 */
const _MappingsDay3 = _s({
  date: [_i10, '*'],
  firstHeight: [_i10, 'first_height'],
});

/**
 * @typedef {{
 *   date: SeriesPattern9<Date>,
 *   firstHeight: SeriesPattern9<Height>,
 * }} MappingsDay1
 */
const _MappingsDay1 = _s({
  date: [_i9, '*'],
  firstHeight: [_i9, 'first_height'],
});

/**
 * @typedef {{
 *   firstHeight: SeriesPattern8<Height>,
 * }} MappingsHour12
 */
const _MappingsHour12 = _s({
  firstHeight: [_i8, '*'],
});

/**
 * @typedef {{
 *   firstHeight: SeriesPattern7<Height>,
 * }} MappingsHour4
 */
const _MappingsHour4 = _s({
  firstHeight: [_i7, '*'],
});

/**
 * @typedef {{
 *   firstHeight: SeriesPattern6<Height>,
 * }} MappingsHour1
 */
const _MappingsHour1 = _s({
  firstHeight: [_i6, '*'],
});

/**
 * @typedef {{
 *   firstHeight: SeriesPattern5<Height>,
 * }} MappingsMinute30
 */
const _MappingsMinute30 = _s({
  firstHeight: [_i5, '*'],
});

/**
 * @typedef {{
 *   firstHeight: SeriesPattern4<Height>,
 * }} MappingsMinute10
 */
const _MappingsMinute10 = _s({
  firstHeight: [_i4, '*'],
});

/**
 * @typedef {{
 *   firstHeight: SeriesPattern17<Height>,
 * }} MappingsHalvingEpoch
 */
const _MappingsHalvingEpoch = _s({
  firstHeight: [_i17, '*'],
});

/**
 * @typedef {{
 *   firstHeight: SeriesPattern18<Height>,
 * }} MappingsDifficultyEpoch
 */
const _MappingsDifficultyEpoch = _s({
  firstHeight: [_i18, '*'],
});

/**
 * @typedef {{
 *   minute10: SeriesPattern19<Minute10>,
 *   minute30: SeriesPattern19<Minute30>,
 *   hour1: SeriesPattern19<Hour1>,
 *   hour4: SeriesPattern19<Hour4>,
 *   hour12: SeriesPattern19<Hour12>,
 *   day1: SeriesPattern19<Day1>,
 *   day3: SeriesPattern19<Day3>,
 *   difficultyEpoch: SeriesPattern19<DifficultyEpoch>,
 *   halvingEpoch: SeriesPattern19<HalvingEpoch>,
 *   week1: SeriesPattern19<Week1>,
 *   month1: SeriesPattern19<Month1>,
 *   month3: SeriesPattern19<Month3>,
 *   month6: SeriesPattern19<Month6>,
 *   year1: SeriesPattern19<Year1>,
 *   year10: SeriesPattern19<Year10>,
 * }} MappingsHeight
 */
const _MappingsHeight = _s({
  minute10: [_i19, '*_minute10'],
  minute30: [_i19, '*_minute30'],
  hour1: [_i19, '*_hour1'],
  hour4: [_i19, '*_hour4'],
  hour12: [_i19, '*_hour12'],
  day1: [_i19, '*_day1'],
  day3: [_i19, '*_day3'],
  difficultyEpoch: [_i19, 'difficulty_epoch'],
  halvingEpoch: [_i19, 'halving_epoch'],
  week1: [_i19, '*_week1'],
  month1: [_i19, '*_month1'],
  month3: [_i19, '*_month3'],
  month6: [_i19, '*_month6'],
  year1: [_i19, '*_year1'],
  year10: [_i19, '*_year10'],
});

/**
 * @typedef {{
 *   identity: SeriesPattern26<OpReturnOutputIndex>,
 * }} MappingsOpReturn
 */
const _MappingsOpReturn = _s({
  identity: [_i26, '*'],
});

/**
 * @typedef {{
 *   identity: SeriesPattern36<UnknownScriptOutputIndex>,
 * }} MappingsUnknownScript
 */
const _MappingsUnknownScript = _s({
  identity: [_i36, '*'],
});

/**
 * @typedef {{
 *   identity: SeriesPattern25<EmptyScriptOutputIndex>,
 * }} EmptyScript
 */
const _EmptyScript = _s({
  identity: [_i25, '*'],
});

/**
 * @typedef {{
 *   identity: SeriesPattern28<P2MSOutputIndex>,
 * }} MappingsP2ms
 */
const _MappingsP2ms = _s({
  identity: [_i28, '*'],
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
 *   identity: SeriesPattern27<P2AAddressIndex>,
 *   address: SeriesPattern27<Address>,
 * }} MappingsP2a
 */
const _MappingsP2a = _s({
  identity: [_i27, '*_index'],
  address: [_i27, '*'],
});

/**
 * @typedef {{
 *   identity: SeriesPattern33<P2TRAddressIndex>,
 *   address: SeriesPattern33<Address>,
 * }} MappingsP2tr
 */
const _MappingsP2tr = _s({
  identity: [_i33, '*_index'],
  address: [_i33, '*'],
});

/**
 * @typedef {{
 *   identity: SeriesPattern35<P2WSHAddressIndex>,
 *   address: SeriesPattern35<Address>,
 * }} MappingsP2wsh
 */
const _MappingsP2wsh = _s({
  identity: [_i35, '*_index'],
  address: [_i35, '*'],
});

/**
 * @typedef {{
 *   identity: SeriesPattern34<P2WPKHAddressIndex>,
 *   address: SeriesPattern34<Address>,
 * }} MappingsP2wpkh
 */
const _MappingsP2wpkh = _s({
  identity: [_i34, '*_index'],
  address: [_i34, '*'],
});

/**
 * @typedef {{
 *   identity: SeriesPattern32<P2SHAddressIndex>,
 *   address: SeriesPattern32<Address>,
 * }} MappingsP2sh
 */
const _MappingsP2sh = _s({
  identity: [_i32, '*_index'],
  address: [_i32, '*'],
});

/**
 * @typedef {{
 *   identity: SeriesPattern31<P2PKHAddressIndex>,
 *   address: SeriesPattern31<Address>,
 * }} MappingsP2pkh
 */
const _MappingsP2pkh = _s({
  identity: [_i31, '*_index'],
  address: [_i31, '*'],
});

/**
 * @typedef {{
 *   identity: SeriesPattern29<P2PK33AddressIndex>,
 *   address: SeriesPattern29<Address>,
 * }} MappingsP2pk33
 */
const _MappingsP2pk33 = _s({
  identity: [_i29, '*_index'],
  address: [_i29, '*'],
});

/**
 * @typedef {{
 *   identity: SeriesPattern30<P2PK65AddressIndex>,
 *   address: SeriesPattern30<Address>,
 * }} MappingsP2pk65
 */
const _MappingsP2pk65 = _s({
  identity: [_i30, '*_index'],
  address: [_i30, '*'],
});

/**
 * @typedef {{
 *   firstIndex: SeriesPattern19<OpReturnOutputIndex>,
 *   txIndex: SeriesPattern26<TxIndex>,
 *   protocol: SeriesPattern26<OpReturnKind>,
 *   postOpReturnBytes: SeriesPattern26<Bytes>,
 * }} IndexerOpReturn
 */
const _IndexerOpReturn = _s({
  firstIndex: [_i19, 'first_*_output_index'],
  txIndex: [_i26, '*_output_tx_index'],
  protocol: [_i26, '*_protocol'],
  postOpReturnBytes: [_i26, 'post_*_bytes'],
});

/**
 * @typedef {{
 *   firstIndex: SeriesPattern19<UnknownScriptOutputIndex>,
 *   txIndex: SeriesPattern36<TxIndex>,
 *   legacySigopCost: SeriesPattern36<SigOps>,
 * }} IndexerUnknownScript
 */
const _IndexerUnknownScript = _s({
  firstIndex: [_i19, 'first_*_output_index'],
  txIndex: [_i36, '*_output_tx_index'],
  legacySigopCost: [_i36, '*_legacy_sigop_cost'],
});

/**
 * @typedef {{
 *   firstIndex: SeriesPattern19<EmptyScriptOutputIndex>,
 *   txIndex: SeriesPattern25<TxIndex>,
 * }} ScriptType
 */
const _ScriptType = _s({
  firstIndex: [_i19, 'first_*_index'],
  txIndex: [_i25, '*_tx_index'],
});

/**
 * @typedef {{
 *   firstIndex: SeriesPattern19<P2MSOutputIndex>,
 *   txIndex: SeriesPattern28<TxIndex>,
 *   legacySigopCost: SeriesPattern28<SigOps>,
 * }} IndexerP2ms
 */
const _IndexerP2ms = _s({
  firstIndex: [_i19, 'first_*_output_index'],
  txIndex: [_i28, '*_output_tx_index'],
  legacySigopCost: [_i28, '*_legacy_sigop_cost'],
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
 *   firstIndex: SeriesPattern19<P2AAddressIndex>,
 *   bytes: SeriesPattern27<P2ABytes>,
 * }} IndexerP2a
 */
const _IndexerP2a = _s({
  firstIndex: [_i19, 'first_*_address_index'],
  bytes: [_i27, '*_bytes'],
});

/**
 * @typedef {{
 *   firstIndex: SeriesPattern19<P2TRAddressIndex>,
 *   bytes: SeriesPattern33<P2TRBytes>,
 * }} IndexerP2tr
 */
const _IndexerP2tr = _s({
  firstIndex: [_i19, 'first_*_address_index'],
  bytes: [_i33, '*_bytes'],
});

/**
 * @typedef {{
 *   firstIndex: SeriesPattern19<P2WSHAddressIndex>,
 *   bytes: SeriesPattern35<P2WSHBytes>,
 * }} IndexerP2wsh
 */
const _IndexerP2wsh = _s({
  firstIndex: [_i19, 'first_*_address_index'],
  bytes: [_i35, '*_bytes'],
});

/**
 * @typedef {{
 *   firstIndex: SeriesPattern19<P2WPKHAddressIndex>,
 *   bytes: SeriesPattern34<P2WPKHBytes>,
 * }} IndexerP2wpkh
 */
const _IndexerP2wpkh = _s({
  firstIndex: [_i19, 'first_*_address_index'],
  bytes: [_i34, '*_bytes'],
});

/**
 * @typedef {{
 *   firstIndex: SeriesPattern19<P2SHAddressIndex>,
 *   bytes: SeriesPattern32<P2SHBytes>,
 * }} IndexerP2sh
 */
const _IndexerP2sh = _s({
  firstIndex: [_i19, 'first_*_address_index'],
  bytes: [_i32, '*_bytes'],
});

/**
 * @typedef {{
 *   firstIndex: SeriesPattern19<P2PKHAddressIndex>,
 *   bytes: SeriesPattern31<P2PKHBytes>,
 * }} IndexerP2pkh
 */
const _IndexerP2pkh = _s({
  firstIndex: [_i19, 'first_*_address_index'],
  bytes: [_i31, '*_bytes'],
});

/**
 * @typedef {{
 *   firstIndex: SeriesPattern19<P2PK33AddressIndex>,
 *   bytes: SeriesPattern29<P2PK33Bytes>,
 * }} IndexerP2pk33
 */
const _IndexerP2pk33 = _s({
  firstIndex: [_i19, 'first_*_address_index'],
  bytes: [_i29, '*_bytes'],
});

/**
 * @typedef {{
 *   firstIndex: SeriesPattern19<P2PK65AddressIndex>,
 *   bytes: SeriesPattern30<P2PK65Bytes>,
 * }} IndexerP2pk65
 */
const _IndexerP2pk65 = _s({
  firstIndex: [_i19, 'first_*_address_index'],
  bytes: [_i30, '*_bytes'],
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
 *   firstTxoutIndex: SeriesPattern19<TxOutIndex>,
 *   value: SeriesPattern23<Sats>,
 *   outputType: SeriesPattern23<OutputType>,
 *   typeIndex: SeriesPattern23<TypeIndex>,
 * }} IndexerOutputs
 */
const _IndexerOutputs = _s({
  firstTxoutIndex: [_i19, 'first_txout_index'],
  value: [_i23, '*_value'],
  outputType: [_i23, '*_type'],
  typeIndex: [_i23, 'type_index'],
});

/**
 * @typedef {{
 *   firstTxinIndex: SeriesPattern19<TxInIndex>,
 *   outpoint: SeriesPattern22<OutPoint>,
 *   txoutIndex: SeriesPattern22<TxOutIndex>,
 *   txIndex: SeriesPattern21<TxIndex>,
 *   outputType: SeriesPattern21<OutputType>,
 *   typeIndex: SeriesPattern22<TypeIndex>,
 *   value: SeriesPattern21<Sats>,
 * }} IndexerInputs
 */
const _IndexerInputs = _s({
  firstTxinIndex: [_i19, 'first_txin_*'],
  outpoint: [_i22, 'outpoint'],
  txoutIndex: [_i22, 'spent_txout_*'],
  txIndex: [_i21, 'input_tx_*'],
  outputType: [_i21, 'output_type'],
  typeIndex: [_i22, 'type_*'],
  value: [_i21, 'input_value'],
});

/**
 * @typedef {{
 *   block: SeriesPattern19<Count>,
 * }} CountSeries
 */
const _CountSeries = _s({
  block: [_i19, '*'],
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
 *   vsize: Vsize,
 *   fees: Fees,
 *   inscription: Inscription,
 *   patterns: Patterns,
 *   nonstandard: TransactionsFlagged,
 *   versions: VersionCount<TxCount>,
 *   transferVolume: ValueFlow,
 *   payments: Payments,
 *   timelocks: Timelocks,
 *   spending: Spending,
 *   ephemeralDust: Flow<Count>,
 *   sameBlockSpending: Flow<Count>,
 *   perSecond: Windows<?PerSecond>,
 * }} Transactions
 */
const _Transactions = _s({
  count: [_FlowStats, '*_count'],
  features: [_Features, '*'],
  vsize: [_Vsize, '*_vsize'],
  fees: [_Fees, '*'],
  inscription: [_Inscription, 'inscription'],
  patterns: [_Patterns, 'coinjoin'],
  nonstandard: [_TransactionsFlagged, 'nonstandard'],
  versions: [(c, b) => _VersionCount(c, b, _TxCount), '*'],
  transferVolume: [_ValueFlow, 'transfer_volume'],
  payments: [_Payments, 'payment_count'],
  timelocks: [_Timelocks, 'locktime_*_count'],
  spending: [_Spending, 'spending_*_count'],
  ephemeralDust: [_Flow, 'ephemeral_dust_spending_*_count'],
  sameBlockSpending: [_Flow, 'same_block_spending_*_count'],
  perSecond: [_Windows, '*_per_second'],
});

/**
 * @typedef {{
 *   flag: SeriesPattern20<Boolean>,
 *   block: SeriesPattern19<Count>,
 * }} IndexerFeature
 */
const _IndexerFeature = _s({
  flag: [_i20, 'is_*'],
  block: [_i19, '*_tx_count'],
});

/**
 * @typedef {{
 *   flag: SeriesPattern20<Boolean>,
 *   block: SeriesPattern19<Count>,
 * }} Feature
 */
const _Feature = _s({
  flag: [_i20, 'has_*'],
  block: [_i19, '*_tx_count'],
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
 *   firstTxIndex: SeriesPattern19<TxIndex>,
 *   txid: SeriesPattern20<Txid>,
 *   version: SeriesPattern20<TxVersion>,
 *   locktime: SeriesPattern20<RawLockTime>,
 *   weight: SeriesPattern20<Weight>,
 *   size: SeriesPattern20<Bytes>,
 *   sigopCost: SeriesPattern20<SigOps>,
 *   firstTxinIndex: SeriesPattern20<TxInIndex>,
 *   firstTxoutIndex: SeriesPattern20<TxOutIndex>,
 *   features: TransactionFeatures,
 *   versions: VersionCount<CountSeries>,
 * }} IndexerTransactions
 */
const _IndexerTransactions = _s({
  firstTxIndex: [_i19, 'first_*_index'],
  txid: [_i20, 'txid'],
  version: [_i20, '*_version'],
  locktime: [_i20, '*_locktime'],
  weight: [_i20, '*_weight'],
  size: [_i20, '*_size'],
  sigopCost: [_i20, '*_sigop_cost'],
  firstTxinIndex: [_i20, 'first_txin_index'],
  firstTxoutIndex: [_i20, 'first_txout_index'],
  features: [_TransactionFeatures, 'sighash'],
  versions: [(c, b) => _VersionCount(c, b, _CountSeries), '*_count'],
});

/**
 * @typedef {{
 *   blockhash: SeriesPattern19<BlockHash>,
 *   coinbaseTag: SeriesPattern19<CoinbaseTag>,
 *   timestamp: SeriesPattern19<Timestamp>,
 *   medianTime: SeriesPattern19<Timestamp>,
 *   size: SeriesPattern19<Bytes>,
 *   weight: SeriesPattern19<Weight>,
 *   segwitTxCount: SeriesPattern19<Count>,
 *   segwitTxSize: SeriesPattern19<Bytes>,
 *   segwitTxWeight: SeriesPattern19<Weight>,
 * }} IndexerBlocks
 */
const _IndexerBlocks = _s({
  blockhash: [_i19, 'blockhash'],
  coinbaseTag: [_i19, 'coinbase_tag'],
  timestamp: [_i19, 'timestamp'],
  medianTime: [_i19, 'median_time'],
  size: [_i19, 'block_size'],
  weight: [_i19, 'block_weight'],
  segwitTxCount: [_i19, '*_count'],
  segwitTxSize: [_i19, '*_size'],
  segwitTxWeight: [_i19, '*_weight'],
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
  inputs: [_Inputs, 'count'],
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
  supply: [_Supply, 'nvt'],
  cointime: [_Cointime, 'age'],
  coinflow: [_Coinflow, 'age'],
  bedrock: [_Bedrock, 'bedrock'],
  rarityMeter: [_RarityMeter, 'rarity_meter'],
});

/**
 * A method per API endpoint, and the typed series tree.
 * @extends BitviewClientBase
 */
class BitviewClient extends BitviewClientBase {
  VERSION = "v0.13.0";

  /** The client's content stamp: the SHA-256 of `core.js`, then this file with this value empty, so any other byte of either changes it. */
  static HASH = "f41963e7406a835703909553a374d81684edd99aa78ac73ec3e3f39228d90241";

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
    "cloverpool": "CloverPool",
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
    "trustpool": "TrustPool",
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
      "long": "Bought at a Discount"
    },
    "premium": {
      "id": "premium",
      "short": "Premium",
      "long": "Bought at a Premium"
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
    "bip47": "BIP-47",
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
   * @param {AddressOutputType} addressType
   * @param {Uint8Array | ArrayBuffer | ArrayBufferView | number[]} payload - Raw payload bytes matching addressType length
   * @param {number} nibbles
   * @param {{ signal?: AbortSignal, onValue?: (value: Frozen<AddressHashPrefixMatches>) => void, cache?: boolean, memCache?: boolean }} [options]
   * @returns {Promise<Frozen<AddressHashPrefixMatches>>}
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
   * Local health and query-readiness check. Returns server identity, uptime, and the sync status of `GET /api/server/sync`, without a bitcoind round-trip. `status` says the server answers, whatever `blocks_behind` is. Reads the published prefix during processing; an empty index returns 503 `state_updating` at once. Responses are not cached.
   *
   * Endpoint: `GET /health`
   * @param {object} [options]
   * @param {AbortSignal} [options.signal]
   * @param {(value: Frozen<Health>) => void} [options.onValue]
   * @param {boolean} [options.cache]
   * @param {boolean} [options.memCache]
   * @returns {Promise<Frozen<Health>>}
   */
  async getHealth({ signal, onValue, cache, memCache } = {}) {
    const path = `/health`;
    return this._getCached(path, 'json', _readJson(undefined), { signal, onValue, cache, memCache });
  }

  /**
   * API version
   *
   * Endpoint: `GET /version`
   * @param {object} [options]
   * @param {AbortSignal} [options.signal]
   * @param {(value: Frozen<ServerVersion>) => void} [options.onValue]
   * @param {boolean} [options.cache]
   * @param {boolean} [options.memCache]
   * @returns {Promise<Frozen<ServerVersion>>}
   */
  async getVersion({ signal, onValue, cache, memCache } = {}) {
    const path = `/version`;
    return this._getCached(path, 'json', _readJson(undefined), { signal, onValue, cache, memCache });
  }

  /**
   * Sync status
   *
   * Returns a coherent local index snapshot and the Bitcoin Core tip height the server last saw: it looks for a new tip every second and no request asks the node. The two heights differ while a new block is being indexed.
   *
   * Endpoint: `GET /api/server/sync`
   * @param {object} [options]
   * @param {AbortSignal} [options.signal]
   * @param {(value: Frozen<SyncStatus>) => void} [options.onValue]
   * @param {boolean} [options.cache]
   * @param {boolean} [options.memCache]
   * @returns {Promise<Frozen<SyncStatus>>}
   */
  async getSyncStatus({ signal, onValue, cache, memCache } = {}) {
    const path = `/api/server/sync`;
    return this._getCached(path, 'json', _readJson(undefined), { signal, onValue, cache, memCache });
  }

  /**
   * Logs
   *
   * The server's last log lines, at most 1,000, oldest first: requests (status, method, path and duration; never who made them) and everything else it logs. To read on, send the answer's `next` back as `after`: the lines since, each once. A jump in `seq` shows lines that left memory before the next request; another `started_unix_ms` shows the server restarted (and its answer has every kept line). Answers are cached for a second.
   *
   * Endpoint: `GET /api/server/logs`
   *
   * @param {object} [options]
   * @param {string} [options.after] - The last answer's `next`: only the lines since. From a process that's since restarted, or none, every kept line.
   * @param {AbortSignal} [options.signal]
   * @param {(value: Frozen<Logs>) => void} [options.onValue]
   * @param {boolean} [options.cache]
   * @param {boolean} [options.memCache]
   * @returns {Promise<Frozen<Logs>>}
   */
  async getLogs({ after, signal, onValue, cache, memCache } = {}) {
    const params = new URLSearchParams();
    if (after != null) params.set('after', String(after));
    const query = params.toString();
    const path = `/api/server/logs${query ? '?' + query : ''}`;
    return this._getCached(path, 'json', _readJson(undefined), { signal, onValue, cache, memCache });
  }

  /**
   * Disk usage
   *
   * Returns the bytes Bitview's data takes on disk and the bytes of the node's block files. Bitview's total is its indexer stores' table files as the stores track them, plus the allocated bytes (file lengths on non-Unix platforms) of every other file in its data directory, the server's own logs left out, measured when the server starts and after each indexed block. The node's total is its own estimate (`getblockchaininfo`'s `size_on_disk`: block and undo files, without its block index), as of the node's last new tip. No request reads the filesystem or asks the node. The ETag is made from both totals.
   *
   * Endpoint: `GET /api/server/disk`
   * @param {object} [options]
   * @param {AbortSignal} [options.signal]
   * @param {(value: Frozen<DiskUsage>) => void} [options.onValue]
   * @param {boolean} [options.cache]
   * @param {boolean} [options.memCache]
   * @returns {Promise<Frozen<DiskUsage>>}
   */
  async getDiskUsage({ signal, onValue, cache, memCache } = {}) {
    const path = `/api/server/disk`;
    return this._getCached(path, 'json', _readJson(undefined), { signal, onValue, cache, memCache });
  }

  /**
   * Series catalog
   *
   * The available series as a tree of categories and subcategories.
   *
   * Endpoint: `GET /api/series`
   * @param {object} [options]
   * @param {AbortSignal} [options.signal]
   * @param {(value: Frozen<TreeNode>) => void} [options.onValue]
   * @param {boolean} [options.cache]
   * @param {boolean} [options.memCache]
   * @returns {Promise<Frozen<TreeNode>>}
   */
  async getSeriesTree({ signal, onValue, cache, memCache } = {}) {
    const path = `/api/series`;
    return this._getCached(path, 'json', _readJson(true), { signal, onValue, cache, memCache });
  }

  /**
   * Series count
   *
   * Returns the number of series available per index type.
   *
   * Endpoint: `GET /api/series/count`
   * @param {object} [options]
   * @param {AbortSignal} [options.signal]
   * @param {(value: Frozen<DetailedSeriesCount>) => void} [options.onValue]
   * @param {boolean} [options.cache]
   * @param {boolean} [options.memCache]
   * @returns {Promise<Frozen<DetailedSeriesCount>>}
   */
  async getSeriesCount({ signal, onValue, cache, memCache } = {}) {
    const path = `/api/series/count`;
    return this._getCached(path, 'json', _readJson({ "by_db": true }), { signal, onValue, cache, memCache });
  }

  /**
   * List available indexes
   *
   * Returns every index a series can be read at, by the name series routes take.
   *
   * Endpoint: `GET /api/series/indexes`
   * @param {object} [options]
   * @param {AbortSignal} [options.signal]
   * @param {(value: Frozen<Index[]>) => void} [options.onValue]
   * @param {boolean} [options.cache]
   * @param {boolean} [options.memCache]
   * @returns {Promise<Frozen<Index[]>>}
   */
  async listIndexes({ signal, onValue, cache, memCache } = {}) {
    const path = `/api/series/indexes`;
    return this._getCached(path, 'json', _readJson(undefined), { signal, onValue, cache, memCache });
  }

  /**
   * Series list
   *
   * Paginated flat list of the available series names: all of them, or with `q` only those that contain that text as written (a plain match, unlike `search_series`; at most 1024 UTF-8 bytes).
   *
   * Endpoint: `GET /api/series/list`
   *
   * @param {object} [options]
   * @param {string} [options.q] - Only the names that contain this text, as written: no words, case folding or wildcards (default: every name)
   * @param {number} [options.page] - Zero-based page number (default: 0)
   * @param {number} [options.per_page] - Results per page, 1 to 1,000 (default: 1000)
   * @param {AbortSignal} [options.signal]
   * @param {(value: Frozen<PaginatedSeries>) => void} [options.onValue]
   * @param {boolean} [options.cache]
   * @param {boolean} [options.memCache]
   * @returns {Promise<Frozen<PaginatedSeries>>}
   */
  async listSeries({ q, page, per_page, signal, onValue, cache, memCache } = {}) {
    const params = new URLSearchParams();
    if (q != null) params.set('q', String(q));
    if (page != null) params.set('page', String(page));
    if (per_page != null) params.set('per_page', String(per_page));
    const query = params.toString();
    const path = `/api/series/list${query ? '?' + query : ''}`;
    return this._getCached(path, 'json', _readJson(undefined), { signal, onValue, cache, memCache });
  }

  /**
   * Search series
   *
   * Find on-chain metrics by plain words: MVRV, realized price, SOPR, NUPL, supply held by long-term or short-term holders, coin age, addresses by balance, hashrate, fees, volume, and thousands more. Start here for any metric question: it returns the series ids to pass to `get_series_latest` or `get_series`. Results prioritize whole query words in names, then names matching every word by its start, joined words or a plural, then descriptions holding every word whole, singular or plural (never a part of a word or a typo), then names with a typo. Word order does not matter. Descriptions provide cohort terminology and formulas. The decoded q parameter is limited to 1024 UTF-8 bytes.
   *
   * Endpoint: `GET /api/series/search`
   *
   * @param {object} options
   * @param {string} options.q - Words to find in series names and descriptions (`realized price`, `mvrv`), in any order
   * @param {number} [options.limit] - Maximum number of results, 1 to 1,000 (default: 100)
   * @param {AbortSignal} [options.signal]
   * @param {(value: Frozen<SeriesName[]>) => void} [options.onValue]
   * @param {boolean} [options.cache]
   * @param {boolean} [options.memCache]
   * @returns {Promise<Frozen<SeriesName[]>>}
   */
  // @ts-expect-error Missing options are checked at runtime.
  async searchSeries({ q, limit, signal, onValue, cache, memCache } = {}) {
    if (q == null) throw new TypeError('searchSeries: options.q is required');
    const params = new URLSearchParams();
    params.set('q', String(q));
    if (limit != null) params.set('limit', String(limit));
    const query = params.toString();
    const path = `/api/series/search${query ? '?' + query : ''}`;
    return this._getCached(path, 'json', _readJson(undefined), { signal, onValue, cache, memCache });
  }

  /**
   * Get series info
   *
   * Returns the optional description, supported indexes, the indexes whose values can be null, the value type, the optional unit (what the value type measures), and the accumulation (how its values combine over time) of the specified series. The decoded series name is limited to 1024 UTF-8 bytes.
   *
   * Endpoint: `GET /api/series/{series}`
   *
   * @param {SeriesName} series
   * @param {object} [options]
   * @param {AbortSignal} [options.signal]
   * @param {(value: Frozen<SeriesInfo>) => void} [options.onValue]
   * @param {boolean} [options.cache]
   * @param {boolean} [options.memCache]
   * @returns {Promise<Frozen<SeriesInfo>>}
   */
  async getSeriesInfo(series, { signal, onValue, cache, memCache } = {}) {
    const path = `/api/series/${encodeURIComponent(series)}`;
    return this._getCached(path, 'json', _readJson(undefined), { signal, onValue, cache, memCache });
  }

  /**
   * Get series data
   *
   * History of an on-chain metric, by the series id from `search_series`: rows selected by a range of positions, dates or timestamps, as JSON or CSV.
   *
   * Endpoint: `GET /api/series/{series}/{index}`
   *
   * @param {SeriesName} series
   * @param {Index} index
   * @param {object} [options]
   * @param {RangeIndex} [options.start] - Inclusive start: integer index, date (YYYY-MM-DD), or UTC timestamp (YYYY-MM-DDTHH:MM:SSZ, optionally with a fraction of 1 to 9 digits before the Z). Negative integers count from end.
   * @param {RangeIndex} [options.end] - Exclusive end: integer index, date (YYYY-MM-DD), or UTC timestamp (YYYY-MM-DDTHH:MM:SSZ, optionally with a fraction of 1 to 9 digits before the Z). Negative integers count from end.
   * @param {Limit} [options.limit] - Maximum number of values to return; with `end` too, the tighter of the two applies. Omitted, the rest of the range.
   * @param {Format} [options.format]
   * @param {AbortSignal} [options.signal]
   * @param {(value: AnySeriesData | string) => void} [options.onValue]
   * @param {boolean} [options.cache]
   * @param {boolean} [options.memCache]
   * @returns {Promise<AnySeriesData | string>}
   */
  async getSeries(series, index, { start, end, limit, format, signal, onValue, cache, memCache } = {}) {
    const params = new URLSearchParams();
    if (start != null) params.set('start', String(start));
    if (end != null) params.set('end', String(end));
    if (limit != null) params.set('limit', String(limit));
    if (format != null) params.set('format', String(format));
    const query = params.toString();
    const path = `/api/series/${encodeURIComponent(series)}/${encodeURIComponent(index)}${query ? '?' + query : ''}`;
    if (format === 'csv') return this.getText(path, { signal, onValue, cache, memCache });
    return _wrapSeriesData(await this._getCached(path, 'json', _readJson(_SERIES_KEEP), { signal, onValue: onValue && ((v) => onValue(_wrapSeriesData(v))), cache, memCache }));
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
   * @param {object} [options]
   * @param {RangeIndex} [options.start] - Inclusive start: integer index, date (YYYY-MM-DD), or UTC timestamp (YYYY-MM-DDTHH:MM:SSZ, optionally with a fraction of 1 to 9 digits before the Z). Negative integers count from end.
   * @param {RangeIndex} [options.end] - Exclusive end: integer index, date (YYYY-MM-DD), or UTC timestamp (YYYY-MM-DDTHH:MM:SSZ, optionally with a fraction of 1 to 9 digits before the Z). Negative integers count from end.
   * @param {Limit} [options.limit] - Maximum number of values to return; with `end` too, the tighter of the two applies. Omitted, the rest of the range.
   * @param {Format} [options.format]
   * @param {AbortSignal} [options.signal]
   * @param {(value: Frozen<*[]> | string) => void} [options.onValue]
   * @param {boolean} [options.cache]
   * @param {boolean} [options.memCache]
   * @returns {Promise<Frozen<*[]> | string>}
   */
  async getSeriesData(series, index, { start, end, limit, format, signal, onValue, cache, memCache } = {}) {
    const params = new URLSearchParams();
    if (start != null) params.set('start', String(start));
    if (end != null) params.set('end', String(end));
    if (limit != null) params.set('limit', String(limit));
    if (format != null) params.set('format', String(format));
    const query = params.toString();
    const path = `/api/series/${encodeURIComponent(series)}/${encodeURIComponent(index)}/data${query ? '?' + query : ''}`;
    if (format === 'csv') return this.getText(path, { signal, onValue, cache, memCache });
    return this._getCached(path, 'json', _readJson(true), { signal, onValue, cache, memCache });
  }

  /**
   * Get latest series value
   *
   * Current value of an on-chain metric, by the series id from `search_series`. Returns the value alone, not inside a SeriesData object.
   *
   * Endpoint: `GET /api/series/{series}/{index}/latest`
   *
   * @param {SeriesName} series
   * @param {Index} index
   * @param {object} [options]
   * @param {AbortSignal} [options.signal]
   * @param {(value: *) => void} [options.onValue]
   * @param {boolean} [options.cache]
   * @param {boolean} [options.memCache]
   * @returns {Promise<*>}
   */
  async getSeriesLatest(series, index, { signal, onValue, cache, memCache } = {}) {
    const path = `/api/series/${encodeURIComponent(series)}/${encodeURIComponent(index)}/latest`;
    return this._getCached(path, 'json', _readJson(true), { signal, onValue, cache, memCache });
  }

  /**
   * Get series data length
   *
   * Endpoint: `GET /api/series/{series}/{index}/len`
   *
   * @param {SeriesName} series
   * @param {Index} index
   * @param {object} [options]
   * @param {AbortSignal} [options.signal]
   * @param {(value: Frozen<number>) => void} [options.onValue]
   * @param {boolean} [options.cache]
   * @param {boolean} [options.memCache]
   * @returns {Promise<Frozen<number>>}
   */
  async getSeriesLen(series, index, { signal, onValue, cache, memCache } = {}) {
    const path = `/api/series/${encodeURIComponent(series)}/${encodeURIComponent(index)}/len`;
    return this._getCached(path, 'json', _readJson(undefined), { signal, onValue, cache, memCache });
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
   * @param {object} [options]
   * @param {AbortSignal} [options.signal]
   * @param {(value: Frozen<Version>) => void} [options.onValue]
   * @param {boolean} [options.cache]
   * @param {boolean} [options.memCache]
   * @returns {Promise<Frozen<Version>>}
   */
  async getSeriesVersion(series, index, { signal, onValue, cache, memCache } = {}) {
    const path = `/api/series/${encodeURIComponent(series)}/${encodeURIComponent(index)}/version`;
    return this._getCached(path, 'json', _readJson(undefined), { signal, onValue, cache, memCache });
  }

  /**
   * Bulk series data
   *
   * Fetch multiple series in a single request. Supports filtering by index and date range. Returns an array of SeriesData objects. For a single series, use `get_series` instead.
   *
   * Endpoint: `GET /api/series/bulk`
   *
   * @param {object} options
   * @param {SeriesList} options.series
   * @param {Index} options.index
   * @param {RangeIndex} [options.start] - Inclusive start: integer index, date (YYYY-MM-DD), or UTC timestamp (YYYY-MM-DDTHH:MM:SSZ, optionally with a fraction of 1 to 9 digits before the Z). Negative integers count from end.
   * @param {RangeIndex} [options.end] - Exclusive end: integer index, date (YYYY-MM-DD), or UTC timestamp (YYYY-MM-DDTHH:MM:SSZ, optionally with a fraction of 1 to 9 digits before the Z). Negative integers count from end.
   * @param {Limit} [options.limit] - Maximum number of values to return; with `end` too, the tighter of the two applies. Omitted, the rest of the range.
   * @param {Format} [options.format]
   * @param {AbortSignal} [options.signal]
   * @param {(value: AnySeriesData[] | string) => void} [options.onValue]
   * @param {boolean} [options.cache]
   * @param {boolean} [options.memCache]
   * @returns {Promise<AnySeriesData[] | string>}
   */
  // @ts-expect-error Missing options are checked at runtime.
  async getSeriesBulk({ series, index, start, end, limit, format, signal, onValue, cache, memCache } = {}) {
    if (series == null) throw new TypeError('getSeriesBulk: options.series is required');
    if (index == null) throw new TypeError('getSeriesBulk: options.index is required');
    const params = new URLSearchParams();
    params.set('series', String(series));
    params.set('index', String(index));
    if (start != null) params.set('start', String(start));
    if (end != null) params.set('end', String(end));
    if (limit != null) params.set('limit', String(limit));
    if (format != null) params.set('format', String(format));
    const query = params.toString();
    const path = `/api/series/bulk${query ? '?' + query : ''}`;
    if (format === 'csv') return this.getText(path, { signal, onValue, cache, memCache });
    return (await this._getCached(path, 'json', _readJson(_SERIES_KEEP), { signal, onValue: onValue && ((v) => onValue(v.map(_wrapSeriesData))), cache, memCache })).map(_wrapSeriesData);
  }

  /**
   * Available URPD cohorts
   *
   * Every URPD cohort once a UTXO set is published (none before): the age aggregates (`all`, `sth`, `lth`, `age_under_4m`, ...), then the age bands from the youngest (`age_under_1h`, `age_1h_to_1d`, ...).
   *
   * Endpoint: `GET /api/urpd`
   * @param {object} [options]
   * @param {AbortSignal} [options.signal]
   * @param {(value: Frozen<UrpdCohort[]>) => void} [options.onValue]
   * @param {boolean} [options.cache]
   * @param {boolean} [options.memCache]
   * @returns {Promise<Frozen<UrpdCohort[]>>}
   */
  async listUrpdCohorts({ signal, onValue, cache, memCache } = {}) {
    const path = `/api/urpd`;
    return this._getCached(path, 'json', _readJson(undefined), { signal, onValue, cache, memCache });
  }

  /**
   * Latest URPD
   *
   * URPD (UTXO realized price distribution: how much supply last moved at each price) for the latest published block; `date` echoes the date served. Fields, units, price-grid rounding and weighting as in `GET /api/urpd/{cohort}/{point}`.
   *
   * Endpoint: `GET /api/urpd/{cohort}`
   *
   * @param {UrpdCohort} cohort
   * @param {object} [options]
   * @param {UrpdAggregation} [options.agg] - Aggregation strategy. Default: raw (no aggregation).
   * @param {UrpdWeight} [options.weight] - Supply weighting. Default: none (unweighted).
   * @param {AbortSignal} [options.signal]
   * @param {(value: Frozen<Urpd>) => void} [options.onValue]
   * @param {boolean} [options.cache]
   * @param {boolean} [options.memCache]
   * @returns {Promise<Frozen<Urpd>>}
   */
  async getUrpdLatest(cohort, { agg, weight, signal, onValue, cache, memCache } = {}) {
    const params = new URLSearchParams();
    if (agg != null) params.set('agg', String(agg));
    if (weight != null) params.set('weight', String(weight));
    const query = params.toString();
    const path = `/api/urpd/${encodeURIComponent(cohort)}${query ? '?' + query : ''}`;
    return this._getCached(path, 'json', _readJson(undefined), { signal, onValue, cache, memCache });
  }

  /**
   * URPD at block height or date
   *
   * URPD (UTXO realized price distribution: how much supply last moved at each price) for a cohort at a block height or the last block of a UTC day. Returns `{ height, hash, date, cohort, weight, aggregation, close, supply, buckets }` where `buckets` is columns, `{ price, supply, realized_cap, unrealized_pnl }`, one entry per bucket in ascending price: the same block header and columnar layout as `GET /api/utxo-set/{point}`. `close`, `price`, `realized_cap`, and `unrealized_pnl` are USD; `supply` is BTC. `unrealized_pnl` can be negative. Bucket `realized_cap` and `unrealized_pnl` value each coin at its creation price on the price grid (four significant digits of the price in cents, within 0.05% of the exact price), rounded to the cent per bucket, so the buckets need not sum to the realized cap series. With a `wakefulness` or `mobility` `weight`, `supply` is the weighted supply (whole satoshis) and the bucket values follow it.
   *
   * Endpoint: `GET /api/urpd/{cohort}/{point}`
   *
   * @param {UrpdCohort} cohort
   * @param {HeightOrDate} point - Confirmed block height (`840000`) or UTC calendar day (`YYYY-MM-DD`)
   * @param {object} [options]
   * @param {UrpdAggregation} [options.agg] - Aggregation strategy. Default: raw (no aggregation).
   * @param {UrpdWeight} [options.weight] - Supply weighting. Default: none (unweighted).
   * @param {AbortSignal} [options.signal]
   * @param {(value: Frozen<Urpd>) => void} [options.onValue]
   * @param {boolean} [options.cache]
   * @param {boolean} [options.memCache]
   * @returns {Promise<Frozen<Urpd>>}
   */
  async getUrpd(cohort, point, { agg, weight, signal, onValue, cache, memCache } = {}) {
    const params = new URLSearchParams();
    if (agg != null) params.set('agg', String(agg));
    if (weight != null) params.set('weight', String(weight));
    const query = params.toString();
    const path = `/api/urpd/${encodeURIComponent(cohort)}/${encodeURIComponent(point)}${query ? '?' + query : ''}`;
    return this._getCached(path, 'json', _readJson(undefined), { signal, onValue, cache, memCache });
  }

  /**
   * Difficulty adjustment
   *
   * Get current difficulty adjustment progress and estimates. Every field is computed from the chain alone, so the answer changes only with a new block: elapsed time is measured to the tip block, and `estimatedRetargetDate` is the tip block's time plus `remainingTime`, where mempool.space counts both from its clock. That date is early by the tip's age and, with a block or two left in the epoch, can be before now; add `remainingTime` to your own clock for a date counted from now.
   *
   * *[Mempool.space docs](https://mempool.space/docs/api/rest#get-difficulty-adjustment)*
   *
   * Endpoint: `GET /api/v1/difficulty-adjustment`
   * @param {object} [options]
   * @param {AbortSignal} [options.signal]
   * @param {(value: Frozen<DifficultyAdjustment>) => void} [options.onValue]
   * @param {boolean} [options.cache]
   * @param {boolean} [options.memCache]
   * @returns {Promise<Frozen<DifficultyAdjustment>>}
   */
  async getDifficultyAdjustment({ signal, onValue, cache, memCache } = {}) {
    const path = `/api/v1/difficulty-adjustment`;
    return this._getCached(path, 'json', _readJson(undefined), { signal, onValue, cache, memCache });
  }

  /**
   * Current BTC price
   *
   * Returns bitcoin latest price (on-chain derived, USD only).
   *
   * *[Mempool.space docs](https://mempool.space/docs/api/rest#get-price)*
   *
   * Endpoint: `GET /api/v1/prices`
   * @param {object} [options]
   * @param {AbortSignal} [options.signal]
   * @param {(value: Frozen<PricePoint>) => void} [options.onValue]
   * @param {boolean} [options.cache]
   * @param {boolean} [options.memCache]
   * @returns {Promise<Frozen<PricePoint>>}
   */
  async getPrices({ signal, onValue, cache, memCache } = {}) {
    const path = `/api/v1/prices`;
    return this._getCached(path, 'json', _readJson(undefined), { signal, onValue, cache, memCache });
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
   * @param {object} [options]
   * @param {Timestamp} [options.timestamp] - UNIX timestamp: the latest close at or before it
   * @param {Currency} [options.currency] - Quote currency: USD, the only one served
   * @param {AbortSignal} [options.signal]
   * @param {(value: Frozen<HistoricalPrice>) => void} [options.onValue]
   * @param {boolean} [options.cache]
   * @param {boolean} [options.memCache]
   * @returns {Promise<Frozen<HistoricalPrice>>}
   */
  async getHistoricalPrice({ timestamp, currency, signal, onValue, cache, memCache } = {}) {
    const params = new URLSearchParams();
    if (timestamp != null) params.set('timestamp', String(timestamp));
    if (currency != null) params.set('currency', String(currency));
    const query = params.toString();
    const path = `/api/v1/historical-price${query ? '?' + query : ''}`;
    return this._getCached(path, 'json', _readJson({ "exchangeRates": true }), { signal, onValue, cache, memCache });
  }

  /**
   * Address hash-prefix matches
   *
   * Find addresses by address type and by the first 1-16 hex nibbles of RapidHash v3 over the raw address payload bytes. Intended for privacy-preserving client-side wallet discovery without sending raw addresses or xpubs. Fetch metadata with `GET /api/address/{address}`.
   *
   * Endpoint: `GET /api/address/hash-prefix/{address_type}/{prefix}`
   *
   * @param {AddressOutputType} address_type
   * @param {string} prefix - First 1–16 hexadecimal nibbles of the RapidHash v3 hash over the raw
   *   address payload bytes.
   * @param {object} [options]
   * @param {AbortSignal} [options.signal]
   * @param {(value: Frozen<AddressHashPrefixMatches>) => void} [options.onValue]
   * @param {boolean} [options.cache]
   * @param {boolean} [options.memCache]
   * @returns {Promise<Frozen<AddressHashPrefixMatches>>}
   */
  async getAddressHashPrefixMatches(address_type, prefix, { signal, onValue, cache, memCache } = {}) {
    const path = `/api/address/hash-prefix/${encodeURIComponent(address_type)}/${encodeURIComponent(prefix)}`;
    return this._getCached(path, 'json', _readJson(undefined), { signal, onValue, cache, memCache });
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
   * @param {object} [options]
   * @param {AbortSignal} [options.signal]
   * @param {(value: Frozen<AddressStats>) => void} [options.onValue]
   * @param {boolean} [options.cache]
   * @param {boolean} [options.memCache]
   * @returns {Promise<Frozen<AddressStats>>}
   */
  async getAddress(address, { signal, onValue, cache, memCache } = {}) {
    const path = `/api/address/${encodeURIComponent(address)}`;
    return this._getCached(path, 'json', _readJson(undefined), { signal, onValue, cache, memCache });
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
   * @param {object} [options]
   * @param {Txid} [options.after_txid] - Continue after this transaction, a mempool or a confirmed one: the older ones (a confirmed
   *   cursor skips the mempool). One that is neither answers 422.
   * @param {number} [options.max_txs] - Transactions to return, at most 50 (the default); larger values return 50.
   * @param {AbortSignal} [options.signal]
   * @param {(value: Frozen<Transaction[]>) => void} [options.onValue]
   * @param {boolean} [options.cache]
   * @param {boolean} [options.memCache]
   * @returns {Promise<Frozen<Transaction[]>>}
   */
  async getAddressTxs(address, { after_txid, max_txs, signal, onValue, cache, memCache } = {}) {
    const params = new URLSearchParams();
    if (after_txid != null) params.set('after_txid', String(after_txid));
    if (max_txs != null) params.set('max_txs', String(max_txs));
    const query = params.toString();
    const path = `/api/address/${encodeURIComponent(address)}/txs${query ? '?' + query : ''}`;
    return this._getCached(path, 'json', _readJson(undefined), { signal, onValue, cache, memCache });
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
   * @param {object} [options]
   * @param {number} [options.max_txs] - Transactions to return, at most 25 (the default); larger values return 25.
   * @param {AbortSignal} [options.signal]
   * @param {(value: Frozen<Transaction[]>) => void} [options.onValue]
   * @param {boolean} [options.cache]
   * @param {boolean} [options.memCache]
   * @returns {Promise<Frozen<Transaction[]>>}
   */
  async getAddressConfirmedTxs(address, { max_txs, signal, onValue, cache, memCache } = {}) {
    const params = new URLSearchParams();
    if (max_txs != null) params.set('max_txs', String(max_txs));
    const query = params.toString();
    const path = `/api/address/${encodeURIComponent(address)}/txs/chain${query ? '?' + query : ''}`;
    return this._getCached(path, 'json', _readJson(undefined), { signal, onValue, cache, memCache });
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
   * @param {object} [options]
   * @param {number} [options.max_txs] - Transactions to return, at most 25 (the default); larger values return 25.
   * @param {AbortSignal} [options.signal]
   * @param {(value: Frozen<Transaction[]>) => void} [options.onValue]
   * @param {boolean} [options.cache]
   * @param {boolean} [options.memCache]
   * @returns {Promise<Frozen<Transaction[]>>}
   */
  async getAddressConfirmedTxsAfter(address, after_txid, { max_txs, signal, onValue, cache, memCache } = {}) {
    const params = new URLSearchParams();
    if (max_txs != null) params.set('max_txs', String(max_txs));
    const query = params.toString();
    const path = `/api/address/${encodeURIComponent(address)}/txs/chain/${encodeURIComponent(after_txid)}${query ? '?' + query : ''}`;
    return this._getCached(path, 'json', _readJson(undefined), { signal, onValue, cache, memCache });
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
   * @param {object} [options]
   * @param {number} [options.max_txs] - Transactions to return, at most 50 (the default); larger values return 50.
   * @param {AbortSignal} [options.signal]
   * @param {(value: Frozen<Transaction[]>) => void} [options.onValue]
   * @param {boolean} [options.cache]
   * @param {boolean} [options.memCache]
   * @returns {Promise<Frozen<Transaction[]>>}
   */
  async getAddressMempoolTxs(address, { max_txs, signal, onValue, cache, memCache } = {}) {
    const params = new URLSearchParams();
    if (max_txs != null) params.set('max_txs', String(max_txs));
    const query = params.toString();
    const path = `/api/address/${encodeURIComponent(address)}/txs/mempool${query ? '?' + query : ''}`;
    return this._getCached(path, 'json', _readJson(undefined), { signal, onValue, cache, memCache });
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
   * @param {object} [options]
   * @param {AbortSignal} [options.signal]
   * @param {(value: Frozen<Utxo[]>) => void} [options.onValue]
   * @param {boolean} [options.cache]
   * @param {boolean} [options.memCache]
   * @returns {Promise<Frozen<Utxo[]>>}
   */
  async getAddressUtxos(address, { signal, onValue, cache, memCache } = {}) {
    const path = `/api/address/${encodeURIComponent(address)}/utxo`;
    return this._getCached(path, 'json', _readJson(undefined), { signal, onValue, cache, memCache });
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
   *   anything else answers 400, as mempool.space does
   * @param {object} [options]
   * @param {AbortSignal} [options.signal]
   * @param {(value: Frozen<AddressValidation>) => void} [options.onValue]
   * @param {boolean} [options.cache]
   * @param {boolean} [options.memCache]
   * @returns {Promise<Frozen<AddressValidation>>}
   */
  async validateAddress(address, { signal, onValue, cache, memCache } = {}) {
    const path = `/api/v1/validate-address/${encodeURIComponent(address)}`;
    return this._getCached(path, 'json', _readJson(undefined), { signal, onValue, cache, memCache });
  }

  /**
   * Block information
   *
   * Details of a block by hash: height, time, transaction count, size, weight, and difficulty.
   *
   * *[Mempool.space docs](https://mempool.space/docs/api/rest#get-block)*
   *
   * Endpoint: `GET /api/block/{hash}`
   *
   * @param {BlockHash} hash
   * @param {object} [options]
   * @param {AbortSignal} [options.signal]
   * @param {(value: Frozen<BlockInfo>) => void} [options.onValue]
   * @param {boolean} [options.cache]
   * @param {boolean} [options.memCache]
   * @returns {Promise<Frozen<BlockInfo>>}
   */
  async getBlock(hash, { signal, onValue, cache, memCache } = {}) {
    const path = `/api/block/${encodeURIComponent(hash)}`;
    return this._getCached(path, 'json', _readJson(undefined), { signal, onValue, cache, memCache });
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
   * @param {object} [options]
   * @param {AbortSignal} [options.signal]
   * @param {(value: Frozen<BlockInfoV1>) => void} [options.onValue]
   * @param {boolean} [options.cache]
   * @param {boolean} [options.memCache]
   * @returns {Promise<Frozen<BlockInfoV1>>}
   */
  async getBlockV1(hash, { signal, onValue, cache, memCache } = {}) {
    const path = `/api/v1/block/${encodeURIComponent(hash)}`;
    return this._getCached(path, 'json', _readJson(undefined), { signal, onValue, cache, memCache });
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
   * @param {object} [options]
   * @param {AbortSignal} [options.signal]
   * @param {(value: Hex) => void} [options.onValue]
   * @param {boolean} [options.cache]
   * @param {boolean} [options.memCache]
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
   * @param {object} [options]
   * @param {AbortSignal} [options.signal]
   * @param {(value: BlockHash) => void} [options.onValue]
   * @param {boolean} [options.cache]
   * @param {boolean} [options.memCache]
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
   * @param {object} [options]
   * @param {AbortSignal} [options.signal]
   * @param {(value: Frozen<BlockTimestamp>) => void} [options.onValue]
   * @param {boolean} [options.cache]
   * @param {boolean} [options.memCache]
   * @returns {Promise<Frozen<BlockTimestamp>>}
   */
  async getBlockByTimestamp(timestamp, { signal, onValue, cache, memCache } = {}) {
    const path = `/api/v1/mining/blocks/timestamp/${encodeURIComponent(timestamp)}`;
    return this._getCached(path, 'json', _readJson(undefined), { signal, onValue, cache, memCache });
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
   * @param {object} [options]
   * @param {AbortSignal} [options.signal]
   * @param {(value: Uint8Array) => void} [options.onValue]
   * @param {boolean} [options.cache]
   * @param {boolean} [options.memCache]
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
   * @param {object} [options]
   * @param {AbortSignal} [options.signal]
   * @param {(value: Frozen<BlockStatus>) => void} [options.onValue]
   * @param {boolean} [options.cache]
   * @param {boolean} [options.memCache]
   * @returns {Promise<Frozen<BlockStatus>>}
   */
  async getBlockStatus(hash, { signal, onValue, cache, memCache } = {}) {
    const path = `/api/block/${encodeURIComponent(hash)}/status`;
    return this._getCached(path, 'json', _readJson(undefined), { signal, onValue, cache, memCache });
  }

  /**
   * Block tip height
   *
   * Height of the latest block.
   *
   * *[Mempool.space docs](https://mempool.space/docs/api/rest#get-block-tip-height)*
   *
   * Endpoint: `GET /api/blocks/tip/height`
   * @param {object} [options]
   * @param {AbortSignal} [options.signal]
   * @param {(value: Height) => void} [options.onValue]
   * @param {boolean} [options.cache]
   * @param {boolean} [options.memCache]
   * @returns {Promise<Height>}
   */
  async getBlockTipHeight({ signal, onValue, cache, memCache } = {}) {
    const path = `/api/blocks/tip/height`;
    return Number(await this.getText(path, { signal, onValue: onValue ? (v) => onValue(Number(v)) : undefined, cache, memCache }));
  }

  /**
   * Block tip hash
   *
   * Hash of the latest block.
   *
   * *[Mempool.space docs](https://mempool.space/docs/api/rest#get-block-tip-hash)*
   *
   * Endpoint: `GET /api/blocks/tip/hash`
   * @param {object} [options]
   * @param {AbortSignal} [options.signal]
   * @param {(value: BlockHash) => void} [options.onValue]
   * @param {boolean} [options.cache]
   * @param {boolean} [options.memCache]
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
   * @param {object} [options]
   * @param {AbortSignal} [options.signal]
   * @param {(value: Txid) => void} [options.onValue]
   * @param {boolean} [options.cache]
   * @param {boolean} [options.memCache]
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
   * @param {object} [options]
   * @param {AbortSignal} [options.signal]
   * @param {(value: Frozen<Txid[]>) => void} [options.onValue]
   * @param {boolean} [options.cache]
   * @param {boolean} [options.memCache]
   * @returns {Promise<Frozen<Txid[]>>}
   */
  async getBlockTxids(hash, { signal, onValue, cache, memCache } = {}) {
    const path = `/api/block/${encodeURIComponent(hash)}/txids`;
    return this._getCached(path, 'json', _readJson(undefined), { signal, onValue, cache, memCache });
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
   * @param {object} [options]
   * @param {AbortSignal} [options.signal]
   * @param {(value: Frozen<Transaction[]>) => void} [options.onValue]
   * @param {boolean} [options.cache]
   * @param {boolean} [options.memCache]
   * @returns {Promise<Frozen<Transaction[]>>}
   */
  async getBlockTxs(hash, { signal, onValue, cache, memCache } = {}) {
    const path = `/api/block/${encodeURIComponent(hash)}/txs`;
    return this._getCached(path, 'json', _readJson(undefined), { signal, onValue, cache, memCache });
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
   * @param {object} [options]
   * @param {AbortSignal} [options.signal]
   * @param {(value: Frozen<Transaction[]>) => void} [options.onValue]
   * @param {boolean} [options.cache]
   * @param {boolean} [options.memCache]
   * @returns {Promise<Frozen<Transaction[]>>}
   */
  async getBlockTxsFromIndex(hash, start_index, { signal, onValue, cache, memCache } = {}) {
    const path = `/api/block/${encodeURIComponent(hash)}/txs/${encodeURIComponent(start_index)}`;
    return this._getCached(path, 'json', _readJson(undefined), { signal, onValue, cache, memCache });
  }

  /**
   * Recent blocks
   *
   * The last 10 blocks.
   *
   * *[Mempool.space docs](https://mempool.space/docs/api/rest#get-blocks)*
   *
   * Endpoint: `GET /api/blocks`
   * @param {object} [options]
   * @param {AbortSignal} [options.signal]
   * @param {(value: Frozen<BlockInfo[]>) => void} [options.onValue]
   * @param {boolean} [options.cache]
   * @param {boolean} [options.memCache]
   * @returns {Promise<Frozen<BlockInfo[]>>}
   */
  async getBlocks({ signal, onValue, cache, memCache } = {}) {
    const path = `/api/blocks`;
    return this._getCached(path, 'json', _readJson(undefined), { signal, onValue, cache, memCache });
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
   * @param {object} [options]
   * @param {AbortSignal} [options.signal]
   * @param {(value: Frozen<BlockInfo[]>) => void} [options.onValue]
   * @param {boolean} [options.cache]
   * @param {boolean} [options.memCache]
   * @returns {Promise<Frozen<BlockInfo[]>>}
   */
  async getBlocksFromHeight(height, { signal, onValue, cache, memCache } = {}) {
    const path = `/api/blocks/${encodeURIComponent(height)}`;
    return this._getCached(path, 'json', _readJson(undefined), { signal, onValue, cache, memCache });
  }

  /**
   * Recent blocks with extras
   *
   * Retrieve the last 15 blocks with extended data including pool identification and fee statistics.
   *
   * *[Mempool.space docs](https://mempool.space/docs/api/rest#get-blocks-v1)*
   *
   * Endpoint: `GET /api/v1/blocks`
   * @param {object} [options]
   * @param {AbortSignal} [options.signal]
   * @param {(value: Frozen<BlockInfoV1[]>) => void} [options.onValue]
   * @param {boolean} [options.cache]
   * @param {boolean} [options.memCache]
   * @returns {Promise<Frozen<BlockInfoV1[]>>}
   */
  async getBlocksV1({ signal, onValue, cache, memCache } = {}) {
    const path = `/api/v1/blocks`;
    return this._getCached(path, 'json', _readJson(undefined), { signal, onValue, cache, memCache });
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
   * @param {object} [options]
   * @param {AbortSignal} [options.signal]
   * @param {(value: Frozen<BlockInfoV1[]>) => void} [options.onValue]
   * @param {boolean} [options.cache]
   * @param {boolean} [options.memCache]
   * @returns {Promise<Frozen<BlockInfoV1[]>>}
   */
  async getBlocksV1FromHeight(height, { signal, onValue, cache, memCache } = {}) {
    const path = `/api/v1/blocks/${encodeURIComponent(height)}`;
    return this._getCached(path, 'json', _readJson(undefined), { signal, onValue, cache, memCache });
  }

  /**
   * This server's recorded block expectation
   *
   * Endpoint: `GET /api/v1/block/{hash}/audit-summary`
   *
   * @param {BlockHash} hash
   * @param {object} [options]
   * @param {AbortSignal} [options.signal]
   * @param {(value: Frozen<AuditSummary>) => void} [options.onValue]
   * @param {boolean} [options.cache]
   * @param {boolean} [options.memCache]
   * @returns {Promise<Frozen<AuditSummary>>}
   */
  async getBlockAuditSummary(hash, { signal, onValue, cache, memCache } = {}) {
    const path = `/api/v1/block/${encodeURIComponent(hash)}/audit-summary`;
    return this._getCached(path, 'json', _readJson(undefined), { signal, onValue, cache, memCache });
  }

  /**
   * Recorded missing, fresh, conflicting, added and unseen transactions
   *
   * Endpoint: `GET /api/v1/mining/blocks/audit/{hash}`
   *
   * @param {BlockHash} hash
   * @param {object} [options]
   * @param {AbortSignal} [options.signal]
   * @param {(value: Frozen<AuditDetail>) => void} [options.onValue]
   * @param {boolean} [options.cache]
   * @param {boolean} [options.memCache]
   * @returns {Promise<Frozen<AuditDetail>>}
   */
  async getBlockAudit(hash, { signal, onValue, cache, memCache } = {}) {
    const path = `/api/v1/mining/blocks/audit/${encodeURIComponent(hash)}`;
    return this._getCached(path, 'json', _readJson(undefined), { signal, onValue, cache, memCache });
  }

  /**
   * Recorded audit score for a block hash
   *
   * A permanent server-local score. A missing audit returns the JSON string "null".
   *
   * Endpoint: `GET /api/v1/mining/blocks/audit/score/{hash}`
   *
   * @param {BlockHash} hash
   * @param {object} [options]
   * @param {AbortSignal} [options.signal]
   * @param {(value: Frozen<AuditScoreResponse>) => void} [options.onValue]
   * @param {boolean} [options.cache]
   * @param {boolean} [options.memCache]
   * @returns {Promise<Frozen<AuditScoreResponse>>}
   */
  async getBlockAuditScore(hash, { signal, onValue, cache, memCache } = {}) {
    const path = `/api/v1/mining/blocks/audit/score/${encodeURIComponent(hash)}`;
    return this._getCached(path, 'json', _readJson(undefined), { signal, onValue, cache, memCache });
  }

  /**
   * Audited scores in the latest sixteen height slots
   *
   * Endpoint: `GET /api/v1/mining/blocks/audit/scores`
   * @param {object} [options]
   * @param {AbortSignal} [options.signal]
   * @param {(value: Frozen<AuditScore[]>) => void} [options.onValue]
   * @param {boolean} [options.cache]
   * @param {boolean} [options.memCache]
   * @returns {Promise<Frozen<AuditScore[]>>}
   */
  async getRecentAuditScores({ signal, onValue, cache, memCache } = {}) {
    const path = `/api/v1/mining/blocks/audit/scores`;
    return this._getCached(path, 'json', _readJson(undefined), { signal, onValue, cache, memCache });
  }

  /**
   * Audited scores through a canonical height
   *
   * Endpoint: `GET /api/v1/mining/blocks/audit/scores/{height}`
   *
   * @param {Height} height
   * @param {object} [options]
   * @param {AbortSignal} [options.signal]
   * @param {(value: Frozen<AuditScore[]>) => void} [options.onValue]
   * @param {boolean} [options.cache]
   * @param {boolean} [options.memCache]
   * @returns {Promise<Frozen<AuditScore[]>>}
   */
  async getAuditScores(height, { signal, onValue, cache, memCache } = {}) {
    const path = `/api/v1/mining/blocks/audit/scores/${encodeURIComponent(height)}`;
    return this._getCached(path, 'json', _readJson(undefined), { signal, onValue, cache, memCache });
  }

  /**
   * Time-bucketed canonical audit match rates
   *
   * Endpoint: `GET /api/v1/mining/blocks/predictions/{time_period}`
   *
   * @param {TimePeriod} time_period - Lookback period: `24h`, `3d`, `1w`, `1m`, `3m`, `6m`, `1y`, `2y`, `3y`, `4y` or `all`.
   * @param {object} [options]
   * @param {AbortSignal} [options.signal]
   * @param {(value: Frozen<[number, Height, number][]>) => void} [options.onValue]
   * @param {boolean} [options.cache]
   * @param {boolean} [options.memCache]
   * @returns {Promise<Frozen<[number, Height, number][]>>}
   */
  async getAuditPredictions(time_period, { signal, onValue, cache, memCache } = {}) {
    const path = `/api/v1/mining/blocks/predictions/${encodeURIComponent(time_period)}`;
    return this._getCached(path, 'json', _readJson(undefined), { signal, onValue, cache, memCache });
  }

  /**
   * How this server's expectation classified a confirmed transaction
   *
   * Endpoint: `GET /api/v1/block/{hash}/tx/{txid}/audit`
   *
   * @param {BlockHash} hash
   * @param {Txid} txid
   * @param {object} [options]
   * @param {AbortSignal} [options.signal]
   * @param {(value: Frozen<TxAudit>) => void} [options.onValue]
   * @param {boolean} [options.cache]
   * @param {boolean} [options.memCache]
   * @returns {Promise<Frozen<TxAudit>>}
   */
  async getTransactionAudit(hash, txid, { signal, onValue, cache, memCache } = {}) {
    const path = `/api/v1/block/${encodeURIComponent(hash)}/tx/${encodeURIComponent(txid)}/audit`;
    return this._getCached(path, 'json', _readJson(undefined), { signal, onValue, cache, memCache });
  }

  /**
   * List all mining pools
   *
   * Every known mining pool.
   *
   * *[Mempool.space docs](https://mempool.space/docs/api/rest#get-mining-pools)*
   *
   * Endpoint: `GET /api/v1/mining/pools`
   * @param {object} [options]
   * @param {AbortSignal} [options.signal]
   * @param {(value: Frozen<PoolInfo[]>) => void} [options.onValue]
   * @param {boolean} [options.cache]
   * @param {boolean} [options.memCache]
   * @returns {Promise<Frozen<PoolInfo[]>>}
   */
  async getPools({ signal, onValue, cache, memCache } = {}) {
    const path = `/api/v1/mining/pools`;
    return this._getCached(path, 'json', _readJson(undefined), { signal, onValue, cache, memCache });
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
   * @param {object} [options]
   * @param {AbortSignal} [options.signal]
   * @param {(value: Frozen<PoolsSummary>) => void} [options.onValue]
   * @param {boolean} [options.cache]
   * @param {boolean} [options.memCache]
   * @returns {Promise<Frozen<PoolsSummary>>}
   */
  async getPoolRanking(time_period, { signal, onValue, cache, memCache } = {}) {
    const path = `/api/v1/mining/pools/${encodeURIComponent(time_period)}`;
    return this._getCached(path, 'json', _readJson(undefined), { signal, onValue, cache, memCache });
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
   * @param {object} [options]
   * @param {AbortSignal} [options.signal]
   * @param {(value: Frozen<PoolDetail>) => void} [options.onValue]
   * @param {boolean} [options.cache]
   * @param {boolean} [options.memCache]
   * @returns {Promise<Frozen<PoolDetail>>}
   */
  async getPool(slug, { signal, onValue, cache, memCache } = {}) {
    const path = `/api/v1/mining/pool/${encodeURIComponent(slug)}`;
    return this._getCached(path, 'json', _readJson(undefined), { signal, onValue, cache, memCache });
  }

  /**
   * All pools hashrate (all time)
   *
   * Get weekly hashrate data, over the whole chain, of the mining pools that mined in the last week, by time and then their weekly rank.
   *
   * *[Mempool.space docs](https://mempool.space/docs/api/rest#get-mining-pool-hashrates)*
   *
   * Endpoint: `GET /api/v1/mining/hashrate/pools`
   * @param {object} [options]
   * @param {AbortSignal} [options.signal]
   * @param {(value: Frozen<PoolHashrateEntry[]>) => void} [options.onValue]
   * @param {boolean} [options.cache]
   * @param {boolean} [options.memCache]
   * @returns {Promise<Frozen<PoolHashrateEntry[]>>}
   */
  async getPoolsHashrate({ signal, onValue, cache, memCache } = {}) {
    const path = `/api/v1/mining/hashrate/pools`;
    return this._getCached(path, 'json', _readJson(undefined), { signal, onValue, cache, memCache });
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
   * @param {object} [options]
   * @param {AbortSignal} [options.signal]
   * @param {(value: Frozen<PoolHashrateEntry[]>) => void} [options.onValue]
   * @param {boolean} [options.cache]
   * @param {boolean} [options.memCache]
   * @returns {Promise<Frozen<PoolHashrateEntry[]>>}
   */
  async getPoolsHashrateByPeriod(time_period, { signal, onValue, cache, memCache } = {}) {
    const path = `/api/v1/mining/hashrate/pools/${encodeURIComponent(time_period)}`;
    return this._getCached(path, 'json', _readJson(undefined), { signal, onValue, cache, memCache });
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
   * @param {object} [options]
   * @param {AbortSignal} [options.signal]
   * @param {(value: Frozen<PoolHashrateEntry[]>) => void} [options.onValue]
   * @param {boolean} [options.cache]
   * @param {boolean} [options.memCache]
   * @returns {Promise<Frozen<PoolHashrateEntry[]>>}
   */
  async getPoolHashrate(slug, { signal, onValue, cache, memCache } = {}) {
    const path = `/api/v1/mining/pool/${encodeURIComponent(slug)}/hashrate`;
    return this._getCached(path, 'json', _readJson(undefined), { signal, onValue, cache, memCache });
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
   * @param {object} [options]
   * @param {AbortSignal} [options.signal]
   * @param {(value: Frozen<BlockInfoV1[]>) => void} [options.onValue]
   * @param {boolean} [options.cache]
   * @param {boolean} [options.memCache]
   * @returns {Promise<Frozen<BlockInfoV1[]>>}
   */
  async getPoolBlocks(slug, { signal, onValue, cache, memCache } = {}) {
    const path = `/api/v1/mining/pool/${encodeURIComponent(slug)}/blocks`;
    return this._getCached(path, 'json', _readJson(undefined), { signal, onValue, cache, memCache });
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
   * @param {object} [options]
   * @param {AbortSignal} [options.signal]
   * @param {(value: Frozen<BlockInfoV1[]>) => void} [options.onValue]
   * @param {boolean} [options.cache]
   * @param {boolean} [options.memCache]
   * @returns {Promise<Frozen<BlockInfoV1[]>>}
   */
  async getPoolBlocksFromHeight(slug, height, { signal, onValue, cache, memCache } = {}) {
    const path = `/api/v1/mining/pool/${encodeURIComponent(slug)}/blocks/${encodeURIComponent(height)}`;
    return this._getCached(path, 'json', _readJson(undefined), { signal, onValue, cache, memCache });
  }

  /**
   * Network hashrate (all time)
   *
   * Get network hashrate and difficulty data for all time.
   *
   * *[Mempool.space docs](https://mempool.space/docs/api/rest#get-hashrate)*
   *
   * Endpoint: `GET /api/v1/mining/hashrate`
   * @param {object} [options]
   * @param {AbortSignal} [options.signal]
   * @param {(value: Frozen<HashrateSummary>) => void} [options.onValue]
   * @param {boolean} [options.cache]
   * @param {boolean} [options.memCache]
   * @returns {Promise<Frozen<HashrateSummary>>}
   */
  async getHashrate({ signal, onValue, cache, memCache } = {}) {
    const path = `/api/v1/mining/hashrate`;
    return this._getCached(path, 'json', _readJson(undefined), { signal, onValue, cache, memCache });
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
   * @param {object} [options]
   * @param {AbortSignal} [options.signal]
   * @param {(value: Frozen<HashrateSummary>) => void} [options.onValue]
   * @param {boolean} [options.cache]
   * @param {boolean} [options.memCache]
   * @returns {Promise<Frozen<HashrateSummary>>}
   */
  async getHashrateByPeriod(time_period, { signal, onValue, cache, memCache } = {}) {
    const path = `/api/v1/mining/hashrate/${encodeURIComponent(time_period)}`;
    return this._getCached(path, 'json', _readJson(undefined), { signal, onValue, cache, memCache });
  }

  /**
   * Difficulty adjustments (all time)
   *
   * Every difficulty adjustment in the chain's history.
   *
   * *[Mempool.space docs](https://mempool.space/docs/api/rest#get-difficulty-adjustments)*
   *
   * Endpoint: `GET /api/v1/mining/difficulty-adjustments`
   * @param {object} [options]
   * @param {AbortSignal} [options.signal]
   * @param {(value: Frozen<DifficultyAdjustmentEntry[]>) => void} [options.onValue]
   * @param {boolean} [options.cache]
   * @param {boolean} [options.memCache]
   * @returns {Promise<Frozen<DifficultyAdjustmentEntry[]>>}
   */
  async getDifficultyAdjustments({ signal, onValue, cache, memCache } = {}) {
    const path = `/api/v1/mining/difficulty-adjustments`;
    return this._getCached(path, 'json', _readJson(undefined), { signal, onValue, cache, memCache });
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
   * @param {object} [options]
   * @param {AbortSignal} [options.signal]
   * @param {(value: Frozen<DifficultyAdjustmentEntry[]>) => void} [options.onValue]
   * @param {boolean} [options.cache]
   * @param {boolean} [options.memCache]
   * @returns {Promise<Frozen<DifficultyAdjustmentEntry[]>>}
   */
  async getDifficultyAdjustmentsByPeriod(time_period, { signal, onValue, cache, memCache } = {}) {
    const path = `/api/v1/mining/difficulty-adjustments/${encodeURIComponent(time_period)}`;
    return this._getCached(path, 'json', _readJson(undefined), { signal, onValue, cache, memCache });
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
   * @param {object} [options]
   * @param {AbortSignal} [options.signal]
   * @param {(value: Frozen<RewardStats>) => void} [options.onValue]
   * @param {boolean} [options.cache]
   * @param {boolean} [options.memCache]
   * @returns {Promise<Frozen<RewardStats>>}
   */
  async getRewardStats(block_count, { signal, onValue, cache, memCache } = {}) {
    const path = `/api/v1/mining/reward-stats/${encodeURIComponent(block_count)}`;
    return this._getCached(path, 'json', _readJson(undefined), { signal, onValue, cache, memCache });
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
   * @param {object} [options]
   * @param {AbortSignal} [options.signal]
   * @param {(value: Frozen<BlockFeesEntry[]>) => void} [options.onValue]
   * @param {boolean} [options.cache]
   * @param {boolean} [options.memCache]
   * @returns {Promise<Frozen<BlockFeesEntry[]>>}
   */
  async getBlockFees(time_period, { signal, onValue, cache, memCache } = {}) {
    const path = `/api/v1/mining/blocks/fees/${encodeURIComponent(time_period)}`;
    return this._getCached(path, 'json', _readJson(undefined), { signal, onValue, cache, memCache });
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
   * @param {object} [options]
   * @param {AbortSignal} [options.signal]
   * @param {(value: Frozen<BlockRewardsEntry[]>) => void} [options.onValue]
   * @param {boolean} [options.cache]
   * @param {boolean} [options.memCache]
   * @returns {Promise<Frozen<BlockRewardsEntry[]>>}
   */
  async getBlockRewards(time_period, { signal, onValue, cache, memCache } = {}) {
    const path = `/api/v1/mining/blocks/rewards/${encodeURIComponent(time_period)}`;
    return this._getCached(path, 'json', _readJson(undefined), { signal, onValue, cache, memCache });
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
   * @param {object} [options]
   * @param {AbortSignal} [options.signal]
   * @param {(value: Frozen<BlockFeeRatesEntry[]>) => void} [options.onValue]
   * @param {boolean} [options.cache]
   * @param {boolean} [options.memCache]
   * @returns {Promise<Frozen<BlockFeeRatesEntry[]>>}
   */
  async getBlockFeeRates(time_period, { signal, onValue, cache, memCache } = {}) {
    const path = `/api/v1/mining/blocks/fee-rates/${encodeURIComponent(time_period)}`;
    return this._getCached(path, 'json', _readJson(undefined), { signal, onValue, cache, memCache });
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
   * @param {object} [options]
   * @param {AbortSignal} [options.signal]
   * @param {(value: Frozen<BlockFeeRatesPreciseEntry[]>) => void} [options.onValue]
   * @param {boolean} [options.cache]
   * @param {boolean} [options.memCache]
   * @returns {Promise<Frozen<BlockFeeRatesPreciseEntry[]>>}
   */
  async getBlockFeeRatesPrecise(time_period, { signal, onValue, cache, memCache } = {}) {
    const path = `/api/v1/mining/blocks/fee-rates/${encodeURIComponent(time_period)}/precise`;
    return this._getCached(path, 'json', _readJson(undefined), { signal, onValue, cache, memCache });
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
   * @param {object} [options]
   * @param {AbortSignal} [options.signal]
   * @param {(value: Frozen<BlockSizesWeights>) => void} [options.onValue]
   * @param {boolean} [options.cache]
   * @param {boolean} [options.memCache]
   * @returns {Promise<Frozen<BlockSizesWeights>>}
   */
  async getBlockSizesWeights(time_period, { signal, onValue, cache, memCache } = {}) {
    const path = `/api/v1/mining/blocks/sizes-weights/${encodeURIComponent(time_period)}`;
    return this._getCached(path, 'json', _readJson(undefined), { signal, onValue, cache, memCache });
  }

  /**
   * Projected mempool blocks
   *
   * Projected blocks for fee estimation, up to eight as mempool.space's. Every block, block 0 the next, is packed as Core's block assembler packs (whole CPFP chunks by feerate, in dependency order, within Core's default weight and sigop budgets), ranked by the node's signed modified fees (including `prioritisetransaction`), with paid fees in the statistics. The last holds whatever is left: more than a block when the mempool is busy. `GET /api/mempool-blocks/{index}` serves each with its transactions, except such a last one.
   *
   * *[Mempool.space docs](https://mempool.space/docs/api/rest#get-mempool-blocks-fees)*
   *
   * Endpoint: `GET /api/v1/fees/mempool-blocks`
   * @param {object} [options]
   * @param {AbortSignal} [options.signal]
   * @param {(value: Frozen<MempoolBlock[]>) => void} [options.onValue]
   * @param {boolean} [options.cache]
   * @param {boolean} [options.memCache]
   * @returns {Promise<Frozen<MempoolBlock[]>>}
   */
  async getMempoolBlocks({ signal, onValue, cache, memCache } = {}) {
    const path = `/api/v1/fees/mempool-blocks`;
    return this._getCached(path, 'json', _readJson(undefined), { signal, onValue, cache, memCache });
  }

  /**
   * Recommended fees
   *
   * Recommended fee rates by confirmation target, in whole sat/vB.
   *
   * *[Mempool.space docs](https://mempool.space/docs/api/rest#get-recommended-fees)*
   *
   * Endpoint: `GET /api/v1/fees/recommended`
   * @param {object} [options]
   * @param {AbortSignal} [options.signal]
   * @param {(value: Frozen<RecommendedFees>) => void} [options.onValue]
   * @param {boolean} [options.cache]
   * @param {boolean} [options.memCache]
   * @returns {Promise<Frozen<RecommendedFees>>}
   */
  async getRecommendedFees({ signal, onValue, cache, memCache } = {}) {
    const path = `/api/v1/fees/recommended`;
    return this._getCached(path, 'json', _readJson(undefined), { signal, onValue, cache, memCache });
  }

  /**
   * Recommended fee rates (precise)
   *
   * Recommended fee rates by confirmation target, with up to three decimal places and support for sub-sat/vB rates.
   *
   * *[Mempool.space docs](https://mempool.space/docs/api/rest#get-recommended-fees-precise)*
   *
   * Endpoint: `GET /api/v1/fees/precise`
   * @param {object} [options]
   * @param {AbortSignal} [options.signal]
   * @param {(value: Frozen<PreciseRecommendedFees>) => void} [options.onValue]
   * @param {boolean} [options.cache]
   * @param {boolean} [options.memCache]
   * @returns {Promise<Frozen<PreciseRecommendedFees>>}
   */
  async getPreciseFees({ signal, onValue, cache, memCache } = {}) {
    const path = `/api/v1/fees/precise`;
    return this._getCached(path, 'json', _readJson(undefined), { signal, onValue, cache, memCache });
  }

  /**
   * Mempool statistics
   *
   * Current mempool backlog: transaction count, total virtual size, total fees, and fee histogram.
   *
   * *[Mempool.space docs](https://mempool.space/docs/api/rest#get-mempool)*
   *
   * Endpoint: `GET /api/mempool`
   * @param {object} [options]
   * @param {AbortSignal} [options.signal]
   * @param {(value: Frozen<MempoolInfo>) => void} [options.onValue]
   * @param {boolean} [options.cache]
   * @param {boolean} [options.memCache]
   * @returns {Promise<Frozen<MempoolInfo>>}
   */
  async getMempool({ signal, onValue, cache, memCache } = {}) {
    const path = `/api/mempool`;
    return this._getCached(path, 'json', _readJson(undefined), { signal, onValue, cache, memCache });
  }

  /**
   * Mempool transaction IDs
   *
   * Get all transaction IDs currently in the mempool.
   *
   * *[Mempool.space docs](https://mempool.space/docs/api/rest#get-mempool-transaction-ids)*
   *
   * Endpoint: `GET /api/mempool/txids`
   * @param {object} [options]
   * @param {AbortSignal} [options.signal]
   * @param {(value: Frozen<Txid[]>) => void} [options.onValue]
   * @param {boolean} [options.cache]
   * @param {boolean} [options.memCache]
   * @returns {Promise<Frozen<Txid[]>>}
   */
  async getMempoolTxids({ signal, onValue, cache, memCache } = {}) {
    const path = `/api/mempool/txids`;
    return this._getCached(path, 'json', _readJson(undefined), { signal, onValue, cache, memCache });
  }

  /**
   * Recent mempool transactions
   *
   * Get the last 10 transactions to enter the mempool.
   *
   * *[Mempool.space docs](https://mempool.space/docs/api/rest#get-mempool-recent)*
   *
   * Endpoint: `GET /api/mempool/recent`
   * @param {object} [options]
   * @param {AbortSignal} [options.signal]
   * @param {(value: Frozen<MempoolTxSummary[]>) => void} [options.onValue]
   * @param {boolean} [options.cache]
   * @param {boolean} [options.memCache]
   * @returns {Promise<Frozen<MempoolTxSummary[]>>}
   */
  async getMempoolRecent({ signal, onValue, cache, memCache } = {}) {
    const path = `/api/mempool/recent`;
    return this._getCached(path, 'json', _readJson(undefined), { signal, onValue, cache, memCache });
  }

  /**
   * Recent RBF replacements
   *
   * Returns up to 25 most-recent RBF replacement trees across the whole mempool. Each entry has the shape of `replacements` in `GET /api/v1/tx/{txid}/rbf`.
   *
   * *[Mempool.space docs](https://mempool.space/docs/api/rest#get-replacements)*
   *
   * Endpoint: `GET /api/v1/replacements`
   * @param {object} [options]
   * @param {AbortSignal} [options.signal]
   * @param {(value: Frozen<ReplacementNode[]>) => void} [options.onValue]
   * @param {boolean} [options.cache]
   * @param {boolean} [options.memCache]
   * @returns {Promise<Frozen<ReplacementNode[]>>}
   */
  async getReplacements({ signal, onValue, cache, memCache } = {}) {
    const path = `/api/v1/replacements`;
    return this._getCached(path, 'json', _readJson(undefined), { signal, onValue, cache, memCache });
  }

  /**
   * Recent full-RBF replacements
   *
   * Same response shape as `GET /api/v1/replacements`, but limited to trees where at least one predecessor was non-signaling (full-RBF).
   *
   * *[Mempool.space docs](https://mempool.space/docs/api/rest#get-fullrbf-replacements)*
   *
   * Endpoint: `GET /api/v1/fullrbf/replacements`
   * @param {object} [options]
   * @param {AbortSignal} [options.signal]
   * @param {(value: Frozen<ReplacementNode[]>) => void} [options.onValue]
   * @param {boolean} [options.cache]
   * @param {boolean} [options.memCache]
   * @returns {Promise<Frozen<ReplacementNode[]>>}
   */
  async getFullrbfReplacements({ signal, onValue, cache, memCache } = {}) {
    const path = `/api/v1/fullrbf/replacements`;
    return this._getCached(path, 'json', _readJson(undefined), { signal, onValue, cache, memCache });
  }

  /**
   * Projected mempool block
   *
   * A projected mempool block with its transactions in block order, each in esplora's light shape as `GET /api/mempool/recent` lists them (`txid`, `fee`, `vsize`, `value`: the whole body is at `GET /api/tx/{txid}`), and its stats, by position as `GET /api/v1/fees/mempool-blocks` lists them: 0 is the next block, packed by Bitview as the rest are. The last one answers 404 while it holds more than a block (what's left of a busy mempool: only its stats are served), as does a block past the projection. The returned `hash` is an opaque content token: poll `GET /api/mempool-blocks/{index}/diff/{hash}` to follow changes instead of refetching the block.
   *
   * Endpoint: `GET /api/mempool-blocks/{index}`
   *
   * @param {MempoolBlockIndex} index
   * @param {object} [options]
   * @param {AbortSignal} [options.signal]
   * @param {(value: Frozen<MempoolBlockContent>) => void} [options.onValue]
   * @param {boolean} [options.cache]
   * @param {boolean} [options.memCache]
   * @returns {Promise<Frozen<MempoolBlockContent>>}
   */
  async getMempoolBlock(index, { signal, onValue, cache, memCache } = {}) {
    const path = `/api/mempool-blocks/${encodeURIComponent(index)}`;
    return this._getCached(path, 'json', _readJson(undefined), { signal, onValue, cache, memCache });
  }

  /**
   * Projected mempool block changes
   *
   * The projected mempool block's changes since the client's copy at `since`. `order` is the new block in order: each entry is either `[start, count]`, the client's transactions `start..start + count` kept in order, or a transaction in the same light shape as the block's, to place there, replacing what was known of its txid. Walk `order` once to rebuild, replace the cached `stats` with the response's, and evict what was cached for the txids in `removed`. Use the response `hash` as the next `since`; with `If-None-Match` an unchanged block answers 304. A `since` this server doesn't hold (aged out, another server's, or from before a restart) is answered from no copy, with `since` `"0"`: every entry is new and `removed` is empty, so replace the copy and drop what was cached for txids not in `order`.
   *
   * Endpoint: `GET /api/mempool-blocks/{index}/diff/{since}`
   *
   * @param {MempoolBlockIndex} index
   * @param {MempoolBlockHash} since - The `hash` of the client's copy, from the block or a previous diff
   * @param {object} [options]
   * @param {AbortSignal} [options.signal]
   * @param {(value: Frozen<MempoolBlockDiff>) => void} [options.onValue]
   * @param {boolean} [options.cache]
   * @param {boolean} [options.memCache]
   * @returns {Promise<Frozen<MempoolBlockDiff>>}
   */
  async getMempoolBlockDiff(index, since, { signal, onValue, cache, memCache } = {}) {
    const path = `/api/mempool-blocks/${encodeURIComponent(index)}/diff/${encodeURIComponent(since)}`;
    return this._getCached(path, 'json', _readJson(undefined), { signal, onValue, cache, memCache });
  }

  /**
   * Next block template
   *
   * The next block's template (BIP 22), built by Bitview: projected block 0's transactions in its order, as `GET /api/mempool-blocks/0` serves them, every parent before its children. It holds `version`, `previousblockhash`, `transactions` (each with `data`, `txid`, `hash`, `depends`, `fee`, `sigops`, `weight`), `coinbasevalue` (the subsidy plus the transactions' fees), `target`, `bits`, `curtime`, `mintime` (the tip's median time past plus one, raised by BIP 94 and BIP 54 at a retarget's edges, as Bitcoin Core computes it), `height`, `default_witness_commitment` and the rest, the format pools and mining software read. A complete block once a miner adds a coinbase paying `coinbasevalue` and carrying `default_witness_commitment`. Built once per projection of block 0 and dated as the request arrives (`curtime`), always on the current tip: `503` for the moment after a new block, until block 0 is projected on it. Bitcoin Core doesn't validate it first: check it as any template from a third party.
   *
   * *[BIP 22](https://github.com/bitcoin/bips/blob/master/bip-0022.mediawiki)*
   *
   * Endpoint: `GET /api/block-template`
   * @param {object} [options]
   * @param {AbortSignal} [options.signal]
   * @param {(value: Frozen<BlockTemplate>) => void} [options.onValue]
   * @param {boolean} [options.cache]
   * @param {boolean} [options.memCache]
   * @returns {Promise<Frozen<BlockTemplate>>}
   */
  async getBlockTemplate({ signal, onValue, cache, memCache } = {}) {
    const path = `/api/block-template`;
    return this._getCached(path, 'json', _readJson({ "coinbaseaux": true, "vbavailable": true }), { signal, onValue, cache, memCache });
  }

  /**
   * Txid by index
   *
   * Retrieve the transaction ID (txid) at a given global transaction index. Returns the txid as plain text.
   *
   * Endpoint: `GET /api/tx-index/{index}`
   *
   * @param {TxIndex} index - Global transaction index across the chain
   * @param {object} [options]
   * @param {AbortSignal} [options.signal]
   * @param {(value: Txid) => void} [options.onValue]
   * @param {boolean} [options.cache]
   * @param {boolean} [options.memCache]
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
   * @param {object} [options]
   * @param {AbortSignal} [options.signal]
   * @param {(value: Frozen<Cpfp>) => void} [options.onValue]
   * @param {boolean} [options.cache]
   * @param {boolean} [options.memCache]
   * @returns {Promise<Frozen<Cpfp>>}
   */
  async getCpfp(txid, { signal, onValue, cache, memCache } = {}) {
    const path = `/api/v1/cpfp/${encodeURIComponent(txid)}`;
    return this._getCached(path, 'json', _readJson(undefined), { signal, onValue, cache, memCache });
  }

  /**
   * RBF replacement history
   *
   * Returns the RBF replacement tree for a transaction, if any. Both `replacements` and `replaces` are null when the tx has no known RBF history within the mempool monitor's retention window. A history past 63 versions or 4,096 transactions comes cut to its newest part, its root marked `"truncated": true`.
   *
   * *[Mempool.space docs](https://mempool.space/docs/api/rest#get-transaction-rbf-history)*
   *
   * Endpoint: `GET /api/v1/tx/{txid}/rbf`
   *
   * @param {Txid} txid
   * @param {object} [options]
   * @param {AbortSignal} [options.signal]
   * @param {(value: Frozen<RbfResponse>) => void} [options.onValue]
   * @param {boolean} [options.cache]
   * @param {boolean} [options.memCache]
   * @returns {Promise<Frozen<RbfResponse>>}
   */
  async getTxRbf(txid, { signal, onValue, cache, memCache } = {}) {
    const path = `/api/v1/tx/${encodeURIComponent(txid)}/rbf`;
    return this._getCached(path, 'json', _readJson(undefined), { signal, onValue, cache, memCache });
  }

  /**
   * Transaction information
   *
   * Full details of a transaction by txid: inputs, outputs, fee, size, and confirmation status.
   *
   * *[Mempool.space docs](https://mempool.space/docs/api/rest#get-transaction)*
   *
   * Endpoint: `GET /api/tx/{txid}`
   *
   * @param {Txid} txid
   * @param {object} [options]
   * @param {AbortSignal} [options.signal]
   * @param {(value: Frozen<Transaction>) => void} [options.onValue]
   * @param {boolean} [options.cache]
   * @param {boolean} [options.memCache]
   * @returns {Promise<Frozen<Transaction>>}
   */
  async getTx(txid, { signal, onValue, cache, memCache } = {}) {
    const path = `/api/tx/${encodeURIComponent(txid)}`;
    return this._getCached(path, 'json', _readJson(undefined), { signal, onValue, cache, memCache });
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
   * @param {object} [options]
   * @param {AbortSignal} [options.signal]
   * @param {(value: Hex) => void} [options.onValue]
   * @param {boolean} [options.cache]
   * @param {boolean} [options.memCache]
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
   * @param {object} [options]
   * @param {AbortSignal} [options.signal]
   * @param {(value: Hex) => void} [options.onValue]
   * @param {boolean} [options.cache]
   * @param {boolean} [options.memCache]
   * @returns {Promise<Hex>}
   */
  async getTxMerkleblockProof(txid, { signal, onValue, cache, memCache } = {}) {
    const path = `/api/tx/${encodeURIComponent(txid)}/merkleblock-proof`;
    return this.getText(path, { signal, onValue, cache, memCache });
  }

  /**
   * Transaction merkle proof
   *
   * Merkle inclusion proof of a confirmed transaction in its block.
   *
   * *[Mempool.space docs](https://mempool.space/docs/api/rest#get-transaction-merkle-proof)*
   *
   * Endpoint: `GET /api/tx/{txid}/merkle-proof`
   *
   * @param {Txid} txid
   * @param {object} [options]
   * @param {AbortSignal} [options.signal]
   * @param {(value: Frozen<MerkleProof>) => void} [options.onValue]
   * @param {boolean} [options.cache]
   * @param {boolean} [options.memCache]
   * @returns {Promise<Frozen<MerkleProof>>}
   */
  async getTxMerkleProof(txid, { signal, onValue, cache, memCache } = {}) {
    const path = `/api/tx/${encodeURIComponent(txid)}/merkle-proof`;
    return this._getCached(path, 'json', _readJson(undefined), { signal, onValue, cache, memCache });
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
   * @param {object} [options]
   * @param {AbortSignal} [options.signal]
   * @param {(value: Frozen<TxOutspend>) => void} [options.onValue]
   * @param {boolean} [options.cache]
   * @param {boolean} [options.memCache]
   * @returns {Promise<Frozen<TxOutspend>>}
   */
  async getTxOutspend(txid, vout, { signal, onValue, cache, memCache } = {}) {
    const path = `/api/tx/${encodeURIComponent(txid)}/outspend/${encodeURIComponent(vout)}`;
    return this._getCached(path, 'json', _readJson(undefined), { signal, onValue, cache, memCache });
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
   * @param {object} [options]
   * @param {AbortSignal} [options.signal]
   * @param {(value: Frozen<TxOutspend[]>) => void} [options.onValue]
   * @param {boolean} [options.cache]
   * @param {boolean} [options.memCache]
   * @returns {Promise<Frozen<TxOutspend[]>>}
   */
  async getTxOutspends(txid, { signal, onValue, cache, memCache } = {}) {
    const path = `/api/tx/${encodeURIComponent(txid)}/outspends`;
    return this._getCached(path, 'json', _readJson(undefined), { signal, onValue, cache, memCache });
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
   * @param {object} [options]
   * @param {AbortSignal} [options.signal]
   * @param {(value: Uint8Array) => void} [options.onValue]
   * @param {boolean} [options.cache]
   * @param {boolean} [options.memCache]
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
   * @param {object} [options]
   * @param {AbortSignal} [options.signal]
   * @param {(value: Frozen<TxStatus>) => void} [options.onValue]
   * @param {boolean} [options.cache]
   * @param {boolean} [options.memCache]
   * @returns {Promise<Frozen<TxStatus>>}
   */
  async getTxStatus(txid, { signal, onValue, cache, memCache } = {}) {
    const path = `/api/tx/${encodeURIComponent(txid)}/status`;
    return this._getCached(path, 'json', _readJson(undefined), { signal, onValue, cache, memCache });
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
   * @param {object} options
   * @param {Txid[]} options.txids - Transaction IDs to look up (max 100 per request).
   * @param {AbortSignal} [options.signal]
   * @param {(value: Frozen<Timestamp[]>) => void} [options.onValue]
   * @param {boolean} [options.cache]
   * @param {boolean} [options.memCache]
   * @returns {Promise<Frozen<Timestamp[]>>}
   */
  // @ts-expect-error Missing options are checked at runtime.
  async getTransactionTimes({ txids, signal, onValue, cache, memCache } = {}) {
    if (txids == null) throw new TypeError('getTransactionTimes: options.txids is required');
    const params = new URLSearchParams();
    for (const _v of txids) params.append('txId[]', String(_v));
    const query = params.toString();
    const path = `/api/v1/transaction-times${query ? '?' + query : ''}`;
    return this._getCached(path, 'json', _readJson(undefined), { signal, onValue, cache, memCache });
  }

  /**
   * Broadcast transaction
   *
   * Submit a raw transaction as hexadecimal text (at most 800,000 request bytes, including whitespace: a standard transaction's largest size). Returns its txid as plain text. No responses are cached. Cancellation or a transport error after dispatch may leave the submission outcome unknown; do not automatically retry.
   *
   * *[Mempool.space docs](https://mempool.space/docs/api/rest#post-transaction)*
   *
   * Endpoint: `POST /api/tx`
   *
   * @param {string} body
   * @param {object} [options]
   * @param {AbortSignal} [options.signal]
   * @returns {Promise<Txid>}
   */
  async postTx(body, { signal } = {}) {
    const path = `/api/tx`;
    return this.postText(path, body, { signal, contentType: 'text/plain; charset=utf-8' });
  }

  /**
   * Latest UTXO set
   *
   * The UTXO set after the latest published block, in the layout of `GET /api/utxo-set/{point}`.
   *
   * Endpoint: `GET /api/utxo-set`
   * @param {object} [options]
   * @param {AbortSignal} [options.signal]
   * @param {(value: Frozen<UtxoSet>) => void} [options.onValue]
   * @param {boolean} [options.cache]
   * @param {boolean} [options.memCache]
   * @returns {Promise<Frozen<UtxoSet>>}
   */
  async getUtxoSetLatest({ signal, onValue, cache, memCache } = {}) {
    const path = `/api/utxo-set`;
    return this._getCached(path, 'json', _readJson(undefined), { signal, onValue, cache, memCache });
  }

  /**
   * UTXO set at block height or date
   *
   * The UTXO set after a block (`840000`) or after the last published block of a UTC day (`YYYY-MM-DD`), by creation height. Returns `{ height, hash, date, count, supply, origins }`; entry `i` of `origins.count` and `origins.supply` (BTC) is what remains unspent of block `i`'s outputs.
   *
   * Endpoint: `GET /api/utxo-set/{point}`
   *
   * @param {HeightOrDate} point - Confirmed block height (`840000`) or UTC calendar day (`YYYY-MM-DD`)
   * @param {object} [options]
   * @param {AbortSignal} [options.signal]
   * @param {(value: Frozen<UtxoSet>) => void} [options.onValue]
   * @param {boolean} [options.cache]
   * @param {boolean} [options.memCache]
   * @returns {Promise<Frozen<UtxoSet>>}
   */
  async getUtxoSet(point, { signal, onValue, cache, memCache } = {}) {
    const path = `/api/utxo-set/${encodeURIComponent(point)}`;
    return this._getCached(path, 'json', _readJson(undefined), { signal, onValue, cache, memCache });
  }

  /**
   * UTXO set changes of a block or date
   *
   * What a block (`840000`) or a UTC day's published blocks (`YYYY-MM-DD`) changed in the UTXO set. Returns `{ first, last, hash, date, created, spent }`: `created` has one row per block, `spent` one row per creation height, each as columnar `height`, `count` and `supply` (BTC). The set after block `first - 1` plus `created` minus `spent` is the set after block `last`.
   *
   * Endpoint: `GET /api/utxo-set/{point}/diff`
   *
   * @param {HeightOrDate} point - Confirmed block height (`840000`) or UTC calendar day (`YYYY-MM-DD`)
   * @param {object} [options]
   * @param {AbortSignal} [options.signal]
   * @param {(value: Frozen<UtxoSetDiff>) => void} [options.onValue]
   * @param {boolean} [options.cache]
   * @param {boolean} [options.memCache]
   * @returns {Promise<Frozen<UtxoSetDiff>>}
   */
  async getUtxoSetDiff(point, { signal, onValue, cache, memCache } = {}) {
    const path = `/api/utxo-set/${encodeURIComponent(point)}/diff`;
    return this._getCached(path, 'json', _readJson(undefined), { signal, onValue, cache, memCache });
  }

  /**
   * Server info
   *
   * The network, the last indexed block, and which tweak modes the server serves: the full index and cut-through, both taking a dust limit.
   *
   * Endpoint: `GET /blindbit/mainnet/info`
   * @param {object} [options]
   * @param {AbortSignal} [options.signal]
   * @param {(value: Frozen<BlindbitInfo>) => void} [options.onValue]
   * @param {boolean} [options.cache]
   * @param {boolean} [options.memCache]
   * @returns {Promise<Frozen<BlindbitInfo>>}
   */
  async getBlindbitInfo({ signal, onValue, cache, memCache } = {}) {
    const path = `/blindbit/mainnet/info`;
    return this._getCached(path, 'json', _readJson(undefined), { signal, onValue, cache, memCache });
  }

  /**
   * Last indexed block
   *
   * The height of the last indexed block.
   *
   * Endpoint: `GET /blindbit/mainnet/block-height`
   * @param {object} [options]
   * @param {AbortSignal} [options.signal]
   * @param {(value: Frozen<BlindbitBlockHeight>) => void} [options.onValue]
   * @param {boolean} [options.cache]
   * @param {boolean} [options.memCache]
   * @returns {Promise<Frozen<BlindbitBlockHeight>>}
   */
  async getBlindbitBlockHeight({ signal, onValue, cache, memCache } = {}) {
    const path = `/blindbit/mainnet/block-height`;
    return this._getCached(path, 'json', _readJson(undefined), { signal, onValue, cache, memCache });
  }

  /**
   * Block hash
   *
   * The hash of the block at a height.
   *
   * Endpoint: `GET /blindbit/mainnet/block-hash/{height}`
   *
   * @param {Height} height
   * @param {object} [options]
   * @param {AbortSignal} [options.signal]
   * @param {(value: Frozen<BlindbitBlockHash>) => void} [options.onValue]
   * @param {boolean} [options.cache]
   * @param {boolean} [options.memCache]
   * @returns {Promise<Frozen<BlindbitBlockHash>>}
   */
  async getBlindbitBlockHash(height, { signal, onValue, cache, memCache } = {}) {
    const path = `/blindbit/mainnet/block-hash/${encodeURIComponent(height)}`;
    return this._getCached(path, 'json', _readJson(undefined), { signal, onValue, cache, memCache });
  }

  /**
   * Tweak index
   *
   * The tweaks of a block's eligible transactions with a Taproot output of at least `dustLimit` sats, as compressed points in hex, in block order. A light wallet multiplies each by its scan key and tests the resulting output keys against the block's `filter/new-utxos`. `dustLimit` is optional and takes any value, as BlindBit takes it (0 when left out), but transactions whose Taproot outputs all hold less than 600 sats aren't held, so any limit below 600 answers as 600.
   *
   * Endpoint: `GET /blindbit/mainnet/tweak-index/{height}`
   *
   * @param {Height} height
   * @param {object} [options]
   * @param {number} [options.dustLimit] - Minimum Taproot output value in sats. Defaults to 0.
   * @param {AbortSignal} [options.signal]
   * @param {(value: Frozen<Tweak[]>) => void} [options.onValue]
   * @param {boolean} [options.cache]
   * @param {boolean} [options.memCache]
   * @returns {Promise<Frozen<Tweak[]>>}
   */
  async getBlindbitTweakIndex(height, { dustLimit, signal, onValue, cache, memCache } = {}) {
    const params = new URLSearchParams();
    if (dustLimit != null) params.set('dustLimit', String(dustLimit));
    const query = params.toString();
    const path = `/blindbit/mainnet/tweak-index/${encodeURIComponent(height)}${query ? '?' + query : ''}`;
    return this._getCached(path, 'json', _readJson(undefined), { signal, onValue, cache, memCache });
  }

  /**
   * Cut-through tweaks
   *
   * `tweak-index`'s tweaks of the transactions that still have an unspent Taproot output of at least `dustLimit` sats, as of the last indexed block: what a wallet restoring its balance alone scans. The wallet's limit applies to unspent outputs even below 600 sats.
   *
   * Endpoint: `GET /blindbit/mainnet/tweaks/{height}`
   *
   * @param {Height} height
   * @param {object} [options]
   * @param {number} [options.dustLimit] - Minimum Taproot output value in sats. Defaults to 0.
   * @param {AbortSignal} [options.signal]
   * @param {(value: Frozen<Tweak[]>) => void} [options.onValue]
   * @param {boolean} [options.cache]
   * @param {boolean} [options.memCache]
   * @returns {Promise<Frozen<Tweak[]>>}
   */
  async getBlindbitTweaks(height, { dustLimit, signal, onValue, cache, memCache } = {}) {
    const params = new URLSearchParams();
    if (dustLimit != null) params.set('dustLimit', String(dustLimit));
    const query = params.toString();
    const path = `/blindbit/mainnet/tweaks/${encodeURIComponent(height)}${query ? '?' + query : ''}`;
    return this._getCached(path, 'json', _readJson(undefined), { signal, onValue, cache, memCache });
  }

  /**
   * Block filter
   *
   * A BIP158 Golomb-coded set (P = 19, M = 784,931, keyed by the block hash) of the block's new Taproot output keys (`new-utxos`, the x-only keys) or of its spends of Taproot outputs (`spent`, hashed as in `spent-index`). Repeated elements count once, as in BIP158 (BlindBit repeats them), which leaves every membership test's answer the same.
   *
   * Endpoint: `GET /blindbit/mainnet/filter/{kind}/{height}`
   *
   * @param {BlindbitFilterKind} kind
   * @param {Height} height
   * @param {object} [options]
   * @param {AbortSignal} [options.signal]
   * @param {(value: Frozen<BlindbitFilter>) => void} [options.onValue]
   * @param {boolean} [options.cache]
   * @param {boolean} [options.memCache]
   * @returns {Promise<Frozen<BlindbitFilter>>}
   */
  async getBlindbitFilter(kind, height, { signal, onValue, cache, memCache } = {}) {
    const path = `/blindbit/mainnet/filter/${encodeURIComponent(kind)}/${encodeURIComponent(height)}`;
    return this._getCached(path, 'json', _readJson(undefined), { signal, onValue, cache, memCache });
  }

  /**
   * Spent index
   *
   * The block's spends of Taproot outputs, each the first 8 bytes of `sha256(txid ‖ vout ‖ block hash)` in hex (the txid and this block's hash in internal byte order, the vout little-endian): a wallet hashes its own outpoints to find which this block spends. Always an object, even when empty (BlindBit then answers `[]`, which spdk can't read).
   *
   * Endpoint: `GET /blindbit/mainnet/spent-index/{height}`
   *
   * @param {Height} height
   * @param {object} [options]
   * @param {AbortSignal} [options.signal]
   * @param {(value: Frozen<BlindbitSpentIndex>) => void} [options.onValue]
   * @param {boolean} [options.cache]
   * @param {boolean} [options.memCache]
   * @returns {Promise<Frozen<BlindbitSpentIndex>>}
   */
  async getBlindbitSpentIndex(height, { signal, onValue, cache, memCache } = {}) {
    const path = `/blindbit/mainnet/spent-index/${encodeURIComponent(height)}`;
    return this._getCached(path, 'json', _readJson(undefined), { signal, onValue, cache, memCache });
  }

  /**
   * Taproot outputs
   *
   * Every Taproot output of the block's eligible transactions (those `tweak-index` lists), in block order, each marked spent or not as of the last indexed block. `block_height` and `timestamp` are the block's (BlindBit sends 0 for the height, and for the time of spent outputs). An output whose key isn't a point on the curve is left out: it can't be a payment, and spdk refuses a whole block over one.
   *
   * Endpoint: `GET /blindbit/mainnet/utxos/{height}`
   *
   * @param {Height} height
   * @param {object} [options]
   * @param {AbortSignal} [options.signal]
   * @param {(value: Frozen<BlindbitUtxo[]>) => void} [options.onValue]
   * @param {boolean} [options.cache]
   * @param {boolean} [options.memCache]
   * @returns {Promise<Frozen<BlindbitUtxo[]>>}
   */
  async getBlindbitUtxos(height, { signal, onValue, cache, memCache } = {}) {
    const path = `/blindbit/mainnet/utxos/${encodeURIComponent(height)}`;
    return this._getCached(path, 'json', _readJson(undefined), { signal, onValue, cache, memCache });
  }

  /**
   * Broadcast transaction
   *
   * Submits a raw transaction, `{"data": "<hex>"}`, to the node, as `POST /api/tx` does. Returns its txid as a JSON string. No responses are cached.
   *
   * Endpoint: `POST /blindbit/mainnet/forward-tx`
   *
   * @param {ForwardTx} body
   * @param {object} [options]
   * @param {AbortSignal} [options.signal]
   * @returns {Promise<Txid>}
   */
  async postBlindbitForwardTx(body, { signal } = {}) {
    const path = `/blindbit/mainnet/forward-tx`;
    return this.postJson(path, JSON.stringify(body), { signal, contentType: 'application/json' });
  }

  /**
   * Live BTC/USD price
   *
   * Current BTC/USD price in dollars, derived from on-chain round-dollar output patterns in the oracle's recent-block window, with the mempool's outputs blended in when available, to the cent: `price` of `GET /api/oracle/live`. Confirmed per-height history is available at `GET /api/series/price/height`. Not found before the oracle's first prices.
   *
   * Endpoint: `GET /api/oracle/price`
   * @param {object} [options]
   * @param {AbortSignal} [options.signal]
   * @param {(value: Frozen<?Dollars>) => void} [options.onValue]
   * @param {boolean} [options.cache]
   * @param {boolean} [options.memCache]
   * @returns {Promise<Frozen<?Dollars>>}
   */
  async getLivePrice({ signal, onValue, cache, memCache } = {}) {
    const path = `/api/oracle/price`;
    return this._getCached(path, 'json', _readJson(undefined), { signal, onValue, cache, memCache });
  }

  /**
   * The oracle now
   *
   * The oracle's reading now, as `GET /api/oracle/blocks/{height}` reads a block: with the mempool's outputs blended in as the next block (`mempool` true, `height` the next block's, `outputs` the mempool's), its search from the tip's answer, `price` the live price; without a mempool at the tip, the tip block's own. One snapshot: its parts never from two moments, `hash` the confirmed tip it was read at. Not found before the oracle's first prices, nor before height 508,000.
   *
   * Endpoint: `GET /api/oracle/live`
   * @param {object} [options]
   * @param {AbortSignal} [options.signal]
   * @param {(value: Frozen<OracleBlock>) => void} [options.onValue]
   * @param {boolean} [options.cache]
   * @param {boolean} [options.memCache]
   * @returns {Promise<Frozen<OracleBlock>>}
   */
  async getOracleLive({ signal, onValue, cache, memCache } = {}) {
    const path = `/api/oracle/live`;
    return this._getCached(path, 'json', _readJson(undefined), { signal, onValue, cache, memCache });
  }

  /**
   * The oracle at a block
   *
   * How the oracle read a confirmed block, after height 508,000: what it counted (the smoothed histogram of the payments of the block and the 11 before it, by amount on its log scale; the block's outputs as its filter judged them, each skipped one under the first rule that skips it), where it looked (every candidate bin in its window, 12 below the block before's answer to 11 above, with its score), what it found (the bin it picked and its price) and how well its stencil fits around that answer (every bin within 120 of it, with its score), with the stencil's 19 round-dollar amounts. Bins: 200 a decade, a bin `round(log10(sats) * 200)`; as where $100 lands, a bin is the price `10^(10 - bin / 200)` dollars. A settled block's never changes.
   *
   * Endpoint: `GET /api/oracle/blocks/{height}`
   *
   * @param {Height} height
   * @param {object} [options]
   * @param {AbortSignal} [options.signal]
   * @param {(value: Frozen<OracleBlock>) => void} [options.onValue]
   * @param {boolean} [options.cache]
   * @param {boolean} [options.memCache]
   * @returns {Promise<Frozen<OracleBlock>>}
   */
  async getOracleBlock(height, { signal, onValue, cache, memCache } = {}) {
    const path = `/api/oracle/blocks/${encodeURIComponent(height)}`;
    return this._getCached(path, 'json', _readJson(undefined), { signal, onValue, cache, memCache });
  }

  /**
   * The oracle's payments, 144 blocks at a time
   *
   * For each of the 144 blocks from `start` (a multiple of 144, else a bad request; from 508,032, earlier ones out of range): its payments as the oracle filters them, each read as each of its 19 round-dollar amounts ($1 to $10,000) at its exact amount in sats, counted by the price each reading implies, in buckets a tenth of a bin wide (0.115% of price) over the block's search window: finer than the oracle's bins, the evidence behind its prices. The chunk at the tip holds its blocks so far, `hash` its last block's; a whole, settled chunk never changes.
   *
   * Endpoint: `GET /api/oracle/payments/{start}`
   *
   * @param {Height} start - The first block's height.
   * @param {object} [options]
   * @param {AbortSignal} [options.signal]
   * @param {(value: Frozen<OraclePayments>) => void} [options.onValue]
   * @param {boolean} [options.cache]
   * @param {boolean} [options.memCache]
   * @returns {Promise<Frozen<OraclePayments>>}
   */
  async getOraclePayments(start, { signal, onValue, cache, memCache } = {}) {
    const path = `/api/oracle/payments/${encodeURIComponent(start)}`;
    return this._getCached(path, 'json', _readJson(undefined), { signal, onValue, cache, memCache });
  }

  /**
   * OpenAPI specification
   *
   * Full OpenAPI 3.1 specification for this API. Build tools from this document.
   *
   * Endpoint: `GET /openapi.json`
   * @param {object} [options]
   * @param {AbortSignal} [options.signal]
   * @param {(value: *) => void} [options.onValue]
   * @param {boolean} [options.cache]
   * @param {boolean} [options.memCache]
   * @returns {Promise<*>}
   */
  async getOpenapi({ signal, onValue, cache, memCache } = {}) {
    const path = `/openapi.json`;
    return this._getCached(path, 'json', _readJson(true), { signal, onValue, cache, memCache });
  }

  /**
   * Compact API summary
   *
   * Compact custom summary of the OpenAPI specification for LLM reading: types instead of schemas, `returns` instead of responses, no examples; parameters keep their enum, default and bounds. Not valid OpenAPI: build tools from `GET /openapi.json`.
   *
   * Endpoint: `GET /api.json`
   * @param {object} [options]
   * @param {AbortSignal} [options.signal]
   * @param {(value: *) => void} [options.onValue]
   * @param {boolean} [options.cache]
   * @param {boolean} [options.memCache]
   * @returns {Promise<*>}
   */
  async getApi({ signal, onValue, cache, memCache } = {}) {
    const path = `/api.json`;
    return this._getCached(path, 'json', _readJson(true), { signal, onValue, cache, memCache });
  }

}

export { BitviewClient, BitviewError, addressPayloadHashPrefix };
