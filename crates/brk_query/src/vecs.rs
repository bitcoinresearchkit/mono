use std::{borrow::Cow, collections::BTreeMap};

use brk_computer::Computer;
use brk_indexer::Indexer;
use brk_plugin::Plugin;
use brk_traversable::{Traversable, TreeNode};
use brk_types::{
    CacheClass, Index, IndexInfo, Limit, PaginatedSeries, Pagination, SeriesCount, SeriesInfo,
    SeriesName,
};
use quickmatch::{QuickMatch, QuickMatchConfig};
use rustc_hash::{FxHashMap, FxHashSet};
use vecdb::{AnyExportableVec, Ro};

mod index_to_vec;
mod series_entry;
mod series_to_vec;

pub use index_to_vec::IndexToVec;
pub use series_entry::SeriesEntry;
pub use series_to_vec::SeriesToVec;

pub struct Vecs<'a> {
    pub series_to_index_to_vec: BTreeMap<&'a str, IndexToVec<'a>>,
    pub index_to_series_to_vec: BTreeMap<Index, SeriesToVec<'a>>,
    pub series: Vec<&'a str>,
    pub indexes: Vec<IndexInfo>,
    pub counts: SeriesCount,
    pub counts_by_db: BTreeMap<String, SeriesCount>,
    catalog: TreeNode,
    matcher: QuickMatch<'a>,
    series_to_indexes: BTreeMap<&'a str, Vec<Index>>,
}

impl<'a> Vecs<'a> {
    pub fn build(indexer: &'a Indexer<Ro>, computer: &'a Computer<Ro>) -> Self {
        let mut description_fragments = Vec::new();
        let mut series_to_description = BTreeMap::new();
        indexer
            .vecs()
            .collect_series_descriptions(&mut description_fragments, &mut series_to_description);
        computer
            .collect_series_descriptions(&mut description_fragments, &mut series_to_description);
        assert!(description_fragments.is_empty());
        Self::build_from(
            indexer
                .vecs()
                .iter_any_visible()
                .map(|vec| (indexer as &dyn Plugin, "indexed", vec)),
            indexer.vecs().to_tree_node(),
            computer
                .iter_plugin_visible()
                .map(|(plugin, vec)| (plugin, plugin.id(), vec)),
            computer.to_tree_node(),
            series_to_description,
        )
    }

    pub fn build_rw(indexer: &'a Indexer, computer: &'a Computer) -> Self {
        let mut description_fragments = Vec::new();
        let mut series_to_description = BTreeMap::new();
        indexer
            .vecs()
            .collect_series_descriptions(&mut description_fragments, &mut series_to_description);
        computer
            .collect_series_descriptions(&mut description_fragments, &mut series_to_description);
        assert!(description_fragments.is_empty());
        Self::build_from(
            indexer
                .vecs()
                .iter_any_visible()
                .map(|vec| (indexer as &dyn Plugin, "indexed", vec)),
            indexer.vecs().to_tree_node(),
            computer
                .iter_plugin_visible()
                .map(|(plugin, vec)| (plugin, plugin.id(), vec)),
            computer.to_tree_node(),
            series_to_description,
        )
    }

    fn build_from(
        indexed_vecs: impl Iterator<Item = (&'a dyn Plugin, &'static str, &'a dyn AnyExportableVec)>,
        indexed_tree: TreeNode,
        computed_vecs: impl Iterator<Item = (&'a dyn Plugin, &'static str, &'a dyn AnyExportableVec)>,
        computed_tree: TreeNode,
        series_to_description: BTreeMap<&'a str, Vec<&'static str>>,
    ) -> Self {
        let mut builder = Builder::default();
        indexed_vecs.for_each(|(plugin, db, vec)| builder.insert(plugin, vec, db));
        computed_vecs.for_each(|(plugin, db, vec)| builder.insert(plugin, vec, db));
        let mut interned_descriptions = BTreeMap::new();
        for (series, fragments) in series_to_description {
            let description = match interned_descriptions.entry(fragments) {
                std::collections::btree_map::Entry::Vacant(entry) => {
                    let description: &'static str =
                        Box::leak(entry.key().join(" ").into_boxed_str());
                    entry.insert(description);
                    description
                }
                std::collections::btree_map::Entry::Occupied(entry) => *entry.get(),
            };
            builder
                .series_to_index_to_vec
                .get_mut(series)
                .unwrap_or_else(|| panic!("Description references unknown series: {series}"))
                .set_description(description);
        }
        builder.counts.distinct = builder.series_to_index_to_vec.len();
        let Builder {
            series_to_index_to_vec,
            index_to_series_to_vec,
            counts,
            counts_by_db,
            ..
        } = builder;

        let sort_ids = |ids: &mut Vec<&str>| {
            ids.sort_unstable_by(|a, b| a.len().cmp(&b.len()).then_with(|| a.cmp(b)))
        };

        let mut series = series_to_index_to_vec.keys().copied().collect::<Vec<_>>();
        sort_ids(&mut series);

        let indexes = index_to_series_to_vec
            .keys()
            .map(|i| IndexInfo {
                index: *i,
                aliases: i
                    .possible_values()
                    .iter()
                    .map(|v| Cow::Borrowed(*v))
                    .collect(),
            })
            .collect();

        let series_to_indexes = series_to_index_to_vec
            .iter()
            .map(|(id, index_to_vec)| (*id, index_to_vec.keys().copied().collect::<Vec<_>>()))
            .collect();

        let catalog = TreeNode::Branch(
            [
                ("indexed".to_string(), indexed_tree),
                ("computed".to_string(), computed_tree),
            ]
            .into_iter()
            .collect(),
        )
        .merge_branches()
        .expect("indexed/computed catalog merge: same series leaf with incompatible schemas");

        let matcher = QuickMatch::new(&series);

        Self {
            series_to_index_to_vec,
            index_to_series_to_vec,
            series,
            indexes,
            counts,
            counts_by_db,
            catalog,
            matcher,
            series_to_indexes,
        }
    }

    pub fn series(&'static self, pagination: Pagination) -> PaginatedSeries {
        let len = self.series.len();
        let per_page = pagination.per_page();
        let start = pagination.start(len);
        let end = pagination.end(len);
        let max_page = len.div_ceil(per_page).saturating_sub(1);

        PaginatedSeries {
            current_page: pagination.page(),
            max_page,
            total_count: len,
            per_page,
            has_more: pagination.page() < max_page,
            series: self.series[start..end]
                .iter()
                .map(|&s| Cow::Borrowed(s))
                .collect(),
        }
    }

    pub fn series_to_indexes(&self, series: &SeriesName) -> Option<&Vec<Index>> {
        self.series_to_indexes.get(series.normalize().as_ref())
    }

    pub fn series_info(&self, series: &SeriesName) -> Option<SeriesInfo> {
        let index_to_vec = self
            .series_to_index_to_vec
            .get(series.normalize().as_ref())?;
        let value_type = index_to_vec.values().next()?.vec().value_type_to_string();
        let indexes = index_to_vec.keys().copied().collect();
        let description = index_to_vec.description().map(Cow::Borrowed);
        Some(SeriesInfo {
            description,
            indexes,
            value_type: value_type.into(),
        })
    }

    pub fn catalog(&self) -> &TreeNode {
        &self.catalog
    }

    pub fn matches(&self, series: &SeriesName, limit: Limit) -> Vec<&'_ str> {
        if limit.is_zero() {
            return Vec::new();
        }
        self.matcher
            .matches_with(series, &QuickMatchConfig::new().with_limit(*limit))
    }

    pub fn get_entry(&self, series: &SeriesName, index: Index) -> Option<SeriesEntry<'a>> {
        self.series_to_index_to_vec
            .get(series.normalize().as_ref())
            .and_then(|index_to_vec| index_to_vec.get(&index).copied())
    }
}

#[derive(Default)]
struct Builder<'a> {
    series_to_index_to_vec: BTreeMap<&'a str, IndexToVec<'a>>,
    index_to_series_to_vec: BTreeMap<Index, SeriesToVec<'a>>,
    counts: SeriesCount,
    counts_by_db: BTreeMap<String, SeriesCount>,
    seen_by_db: FxHashMap<&'a str, FxHashSet<&'a str>>,
}

impl<'a> Builder<'a> {
    fn insert(&mut self, plugin: &'a dyn Plugin, vec: &'a dyn AnyExportableVec, db: &'a str) {
        let name = vec.name();
        let serialized_index = vec.index_type_to_string();
        let index = Index::try_from(serialized_index)
            .unwrap_or_else(|_| panic!("Unknown index type: {serialized_index}"));
        let requires_gate =
            matches!(index.cache_class(), CacheClass::Mutable) || plugin.mutates_existing(vec);
        let entry = SeriesEntry::new(vec, plugin, requires_gate);

        let prev = self
            .series_to_index_to_vec
            .entry(name)
            .or_default()
            .insert(index, entry);
        assert!(
            prev.is_none(),
            "Duplicate series: {name} for index {index:?}"
        );
        self.index_to_series_to_vec
            .entry(index)
            .or_default()
            .insert(name, entry);

        let is_lazy = vec.region_names().is_empty();
        let by_db = self.counts_by_db.entry(db.to_string()).or_default();
        self.counts.total += 1;
        by_db.total += 1;
        if is_lazy {
            self.counts.lazy += 1;
            by_db.lazy += 1;
        } else {
            self.counts.stored += 1;
            by_db.stored += 1;
        }
        if self.seen_by_db.entry(db).or_default().insert(name) {
            by_db.distinct += 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use brk_computer::Computer;
    use brk_indexer::Indexer;
    use brk_reader::Reader;
    use brk_rpc::{Auth, Client};
    use brk_types::{Index, SeriesName};

    use super::Vecs;

    const REALIZED_PRICE_DESCRIPTION: &str = "The sats-weighted average USD creation price of the unspent outputs in the selected cohort: Σ(creation price × unspent sats) / Σ(unspent sats). Returns zero when the cohort has no unspent supply.";
    const USD_DESCRIPTION: &str = "Reported in USD per BTC.";
    const CENTS_DESCRIPTION: &str = "Reported in cents per BTC.";
    const SATS_DESCRIPTION: &str =
        "Reported in sats per USD: 100,000,000 divided by the price in USD per BTC.";
    const PPM_DESCRIPTION: &str = "Spot price divided by this price in parts per million; 1,000,000 represents a ratio of 1.0.";
    const RATIO_DESCRIPTION: &str = "Spot price divided by this price as a unitless decimal ratio.";
    const AMOUNT_BTC_DESCRIPTION: &str = "Reported in BTC; one BTC equals 100,000,000 satoshis.";
    const AMOUNT_SATS_DESCRIPTION: &str = "Reported in satoshis.";
    const AMOUNT_USD_DESCRIPTION: &str = "Reported in US dollars.";
    const AMOUNT_CENTS_DESCRIPTION: &str = "Reported in US cents; 100 cents equal one US dollar.";
    const GENERIC_PPM_DESCRIPTION: &str =
        "Unitless ratio in parts per million; 1,000,000 represents 1.0.";
    const GENERIC_RATIO_DESCRIPTION: &str =
        "Unitless decimal ratio derived as parts per million divided by 1,000,000.";
    const PERCENT_DESCRIPTION: &str = "Percentage derived as the decimal ratio multiplied by 100.";
    const TOTAL_SIZE_DESCRIPTION: &str = "Total serialized size in bytes, including witness data. At `tx_index`, this is the byte length of the transaction's consensus serialization. At `height`, this is the entire block: its 80-byte header, transaction-count CompactSize, and every serialized transaction.";
    const BLOCK_SIZE_DESCRIPTION: &str = "Total serialized block size in bytes, including the header, transaction-count CompactSize, and witness data.";
    const BLOCK_VBYTES_DESCRIPTION: &str = "Virtual size of the block in vbytes, computed as the block weight in weight units divided by four and rounded down.";
    const BLOCK_WEIGHT_DESCRIPTION: &str = "BIP-141 block weight in weight units: non-witness bytes count as four weight units and witness bytes count as one.";
    const BLOCK_FULLNESS_DESCRIPTION: &str =
        "Block weight divided by the 4,000,000-weight-unit consensus limit.";
    const SEGWIT_TXS_DESCRIPTION: &str =
        "Number of non-coinbase transactions using SegWit serialization.";
    const SEGWIT_SIZE_DESCRIPTION: &str = "Combined total serialized size in bytes of the block's non-coinbase SegWit transactions; excludes block overhead and all other transactions.";
    const SEGWIT_WEIGHT_DESCRIPTION: &str = "Combined BIP-141 weight in weight units of the block's non-coinbase SegWit transactions; excludes block overhead and all other transactions.";
    const BLOCK_COUNT_TARGET_DESCRIPTION: &str = "Expected number of blocks in the selected window at Bitcoin's target interval of ten minutes per block.";
    const BLOCK_COUNT_DESCRIPTION: &str = "Number of indexed blocks. The per-block value is one, the cumulative count is height plus one because genesis is included, and rolling sums count the blocks in the selected window.";
    const BLOCK_INTERVAL_DESCRIPTION: &str = "Nonnegative difference in seconds between this block's header timestamp and the previous block's. Genesis is zero, and a timestamp earlier than its predecessor is clamped to zero.";
    const DIFFICULTY_DESCRIPTION: &str = "Mining difficulty encoded by the block header, calculated as Bitcoin's maximum target divided by this block's proof-of-work target.";
    const DIFFICULTY_HASHRATE_DESCRIPTION: &str = "Theoretical hash rate implied by difficulty at the ten-minute target: difficulty multiplied by 2^32 and divided by 600, in hashes per second.";
    const DIFFICULTY_ADJUSTMENT_DESCRIPTION: &str = "Relative difficulty change versus 2,016 block heights earlier: current difficulty divided by lookback difficulty, minus one. Unavailable for the first 2,016 blocks.";
    const DIFFICULTY_EPOCH_DESCRIPTION: &str = "Zero-based difficulty epoch number, equal to block height divided by 2,016 and rounded down.";
    const BLOCKS_TO_RETARGET_DESCRIPTION: &str = "Number of blocks from the represented height to the first block of the next difficulty epoch: 2,016 minus height modulo 2,016.";
    const DAYS_TO_RETARGET_DESCRIPTION: &str = "Nominal days to the next difficulty epoch, calculated as `blocks_to_retarget / 144`; this does not use observed mining pace.";
    const HALVING_EPOCH_DESCRIPTION: &str = "Zero-based block-subsidy era, equal to block height divided by 210,000 and rounded down. Era zero begins at genesis.";
    const BLOCKS_TO_HALVING_DESCRIPTION: &str = "Number of blocks from the represented height to the first block of the next subsidy era: 210,000 minus height modulo 210,000.";
    const DAYS_TO_HALVING_DESCRIPTION: &str = "Nominal days to the next subsidy halving, calculated as `blocks_to_halving / 144`; this does not use observed mining pace.";
    const BLOCKHASH_DESCRIPTION: &str = "Double-SHA256 hash of the block header, displayed in Bitcoin's conventional hexadecimal byte order.";
    const COINBASE_TAG_DESCRIPTION: &str = "First 100 bytes of the coinbase transaction's first-input `scriptSig`, exposed as a string by mapping each byte to the same-valued Unicode code point. This is raw coinbase data, not a normalized mining-pool label.";
    const LOOKBACK_HEIGHT_DESCRIPTION: &str = "First block height inside this series' trailing duration, found from the running maximum of block-header timestamps. A height exactly at the cutoff is excluded; returns genesis height zero when less history exists. Duration suffixes are fixed: `h` is 3,600 seconds, `d` is 24 hours, `w` is 7 days, `m` is 30 days, and `y` is 365 days.";
    const COINBASE_REWARD_DESCRIPTION: &str = "Sum of the output values of the block's coinbase transaction. This is the miner reward actually assigned to coinbase outputs, not the total reward available.";
    const DERIVED_SUBSIDY_DESCRIPTION: &str = "Coinbase output value minus the block's total transaction fees. This is a derived subsidy component, not the scheduled consensus subsidy, and can be lower when the miner leaves some available reward unclaimed.";
    const TRANSACTION_FEES_DESCRIPTION: &str =
        "Sum of input value minus output value across the block's non-coinbase transactions.";
    const OUTPUT_VOLUME_DESCRIPTION: &str = "Sum of the output values of the block's non-coinbase transactions, equivalently their total input value minus transaction fees. Reported in satoshis.";
    const UNCLAIMED_REWARDS_DESCRIPTION: &str = "Portion of the available block reward not assigned to coinbase outputs: scheduled subsidy plus transaction fees minus coinbase output value.";
    const FEE_DOMINANCE_DESCRIPTION: &str = "Transaction fees divided by coinbase output value. Cumulative variants use cumulative totals; rolling variants use totals within the trailing window.";
    const SUBSIDY_DOMINANCE_DESCRIPTION: &str = "One minus fee dominance, equivalently the derived subsidy component divided by coinbase output value. Cumulative variants use cumulative totals; rolling variants use totals within the trailing window.";
    const FEE_TO_SUBSIDY_DESCRIPTION: &str = "Total transaction fees in the trailing window divided by the total derived subsidy component in the same window.";
    const HASH_RATE_DESCRIPTION: &str = "Estimated network hash rate in hashes per second: the current block's difficulty-implied target hash rate multiplied by the number of blocks in the trailing 24 hours and divided by 144. The current difficulty is used for the entire estimate.";
    const HASH_RATE_SMA_DESCRIPTION: &str = "Arithmetic mean of the per-block network hash-rate estimates over the trailing duration named by the series; every block has equal weight. Durations are fixed at 7, 30, 60, or 365 days.";
    const HASH_RATE_ATH_DESCRIPTION: &str =
        "Running all-time high of the estimated network hash rate, in hashes per second.";
    const HASH_RATE_DRAWDOWN_DESCRIPTION: &str =
        "Estimated network hash rate divided by its running all-time high, minus one.";
    const HASH_PRICE_DESCRIPTION: &str = "Estimated miner revenue over the trailing 24-hour window, with each coinbase output valued in USD at its block's spot price, divided by the current estimated network hash rate.";
    const HASH_VALUE_DESCRIPTION: &str = "Coinbase output value in satoshis over the trailing 24-hour window, divided by the current estimated network hash rate.";
    const PER_THS_DESCRIPTION: &str =
        "Reported per TH/s, where one TH/s is 10^12 hashes per second.";
    const THS_MIN_DESCRIPTION: &str = "Running all-time minimum of the per-TH/s series, where one TH/s is 10^12 hashes per second. Zero values are excluded; returns zero until the first nonzero value exists.";
    const PER_PHS_DESCRIPTION: &str = "Reported per PH/s, where one PH/s is 10^15 hashes per second; exactly 1,000 times the corresponding per-TH/s series.";
    const PHS_MIN_DESCRIPTION: &str = "Running all-time minimum of the per-PH/s series, where one PH/s is 10^15 hashes per second. It is exactly 1,000 times the corresponding per-TH/s minimum and returns zero until the first nonzero value exists.";
    const HASH_REBOUND_DESCRIPTION: &str = "Current per-PH/s value divided by its running nonzero all-time minimum, minus one. Returns zero before a nonzero minimum exists.";
    const TX_COUNT_DESCRIPTION: &str =
        "Number of transactions, including the coinbase transaction.";
    const TX_VSIZE_DESCRIPTION: &str = "Transaction virtual size in vbytes, calculated as BIP-141 weight divided by four and rounded up. The transaction-index series gives each transaction's value. Distribution series count every transaction equally and include coinbase, either in the represented block or the six-block window ending there; time-period indexes take the value from the period's final block.";
    const TX_WEIGHT_DESCRIPTION: &str = "BIP-141 transaction weight in weight units: non-witness bytes count as four weight units and witness bytes count as one. The transaction-index series gives each transaction's value. Distribution series count every transaction equally and include coinbase, either in the represented block or the six-block window ending there; time-period indexes take the value from the period's final block.";
    const TRANSFER_VOLUME_DESCRIPTION: &str = "Sum of the input values of non-coinbase transactions. This equals their total output value plus transaction fees and is not adjusted to estimate economic payment volume.";
    const TX_PER_SECOND_DESCRIPTION: &str = "Number of transactions, including coinbase, in the trailing fixed window divided by that window's full duration in seconds. The divisor remains the full duration before enough chain history exists. At time-period indexes, the value is taken from the period's final block.";
    const TX_INPUT_VALUE_DESCRIPTION: &str = "Sum of the transaction's referenced previous-output values, in satoshis. Coinbase uses `Sats::MAX` as a sentinel because it has no previous outputs to spend.";
    const TX_OUTPUT_VALUE_DESCRIPTION: &str =
        "Sum of the transaction's output values, in satoshis.";
    const TX_FEE_DESCRIPTION: &str = "Transaction fee in satoshis: input value minus output value; coinbase is zero. The transaction-index series includes zero-fee transactions. Distribution series count every included transaction equally and exclude coinbase and zero-fee transactions, either in the represented block or the six-block window ending there; time-period indexes take the value from the period's final block.";
    const TX_FEE_RATE_DESCRIPTION: &str = "Raw transaction fee rate in sat/vB: fee divided by virtual size and rounded upward to the nearest 0.001 sat/vB. Coinbase and zero-fee transactions are zero.";
    const TX_EFFECTIVE_FEE_RATE_DESCRIPTION: &str = "Effective transaction fee rate in sat/vB after applying Bitcoin Core's Single Fee Linearization independently to each same-block dependency component. Every transaction in an ancestor-closed SFL chunk receives the chunk's combined fees divided by combined virtual size, rounded upward to the nearest 0.001 sat/vB. The transaction-index series includes zero effective rates. Distribution series exclude coinbase and zero effective rates and weight percentile ranks by transaction virtual size, either in the represented block or the six-block window ending there; time-period indexes take the value from the period's final block.";
    const CPFP_PARENT_DESCRIPTION: &str = "Whether the transaction's Single Fee Linearization effective fee rate is higher than its raw fee rate, indicating that same-block descendants raise the rate at which the transaction's SFL chunk is evaluated.";
    const CPFP_CHILD_DESCRIPTION: &str = "Whether the transaction's Single Fee Linearization effective fee rate is lower than its raw fee rate, indicating that its fee raises the rate at which a same-block ancestor-closed SFL chunk is evaluated.";
    const CPFP_PARENT_COUNT_DESCRIPTION: &str = "Number of confirmed transactions whose same-block descendants raise their fee rate under Single Fee Linearization.";
    const CPFP_CHILD_COUNT_DESCRIPTION: &str = "Number of confirmed transactions whose fee raises the effective rate of a same-block ancestor-closed chunk under Single Fee Linearization.";
    const TX_VERSION_CATEGORY_DESCRIPTION: &str = "Compact transaction-version category for the indexed transaction. Values 1, 2, and 3 preserve those exact signed 32-bit Bitcoin transaction versions; 255 represents every other version. The series includes coinbase transactions. Use individual raw transaction data to inspect the original version when this value is 255.";
    const TX_COUNT_V1_DESCRIPTION: &str = "Number of transactions in the block whose signed 32-bit Bitcoin transaction version is exactly 1, including coinbase.";
    const TX_COUNT_V2_DESCRIPTION: &str = "Number of transactions in the block whose signed 32-bit Bitcoin transaction version is exactly 2, including coinbase.";
    const TX_COUNT_V3_DESCRIPTION: &str = "Number of transactions in the block whose signed 32-bit Bitcoin transaction version is exactly 3, including coinbase.";
    const TX_COUNT_OTHER_VERSION_DESCRIPTION: &str = "Number of transactions in the block whose signed 32-bit Bitcoin transaction version is not 1, 2, or 3, including coinbase. This category combines every other value; use individual raw transaction data to inspect the original version.";
    const TX_VERSION_COUNTS_DESCRIPTION: &str = "Counts every transaction, including coinbase, by its signed 32-bit Bitcoin transaction version.";
    const TX_VERSION_1_DESCRIPTION: &str = "Transactions whose version is exactly 1.";
    const TX_VERSION_2_DESCRIPTION: &str = "Transactions whose version is exactly 2.";
    const TX_VERSION_3_DESCRIPTION: &str = "Transactions whose version is exactly 3.";
    const TX_OTHER_VERSION_DESCRIPTION: &str = "Transactions whose version is not 1, 2, or 3. This category combines every other value; use individual raw transaction data to inspect the original version.";
    const IS_EXPLICITLY_RBF_DESCRIPTION: &str = "Whether at least one input has a sequence number below `0xfffffffe`, the explicit opt-in RBF signal defined by BIP 125. This is a mechanical sequence signal: it does not prove the transaction was replaceable or replaced, does not include inherited signaling, and does not account for full-RBF policy. Coinbase transactions are evaluated by the same sequence rule.";
    const EXPLICITLY_RBF_COUNT_DESCRIPTION: &str = "Number of transactions in the block with at least one input sequence number below `0xfffffffe`, the explicit opt-in RBF signal defined by BIP 125. This counts the mechanical sequence signal, not whether a transaction was replaceable or replaced, inherited signaling, or full-RBF policy. Coinbase transactions are evaluated by the same sequence rule.";
    const TX_POLICY_DESCRIPTION: &str = "BRK's deterministic, transaction-local approximation of default Bitcoin Core relay standardness, selected by mainnet block height rather than actual node adoption. It checks transaction version, weight and stripped size, script and witness forms, signature-operation limits, dust, and `OP_RETURN` policy. The approximation changes after height 863,500 and at heights 905,000 and 921,000. It does not reconstruct fee-floor, mempool/package topology, conflict or replacement policy, or node-specific settings. Coinbase transactions are classified false.";
    const IS_NONSTANDARD_DESCRIPTION: &str =
        "Whether the indexed transaction is classified as nonstandard under this approximation.";
    const NONSTANDARD_COUNT_DESCRIPTION: &str =
        "Number of transactions classified as nonstandard under this approximation.";
    const COINDAYS_CREATED_DESCRIPTION: &str = "Coin days accrued by unspent supply between block timestamps, allocated to the age range in which they accrue. One coin day is one BTC remaining unspent for one day.";
    const CAPITAL_SENTIMENT_LONG_DESCRIPTION: &str = "Whether the stateful capital-sentiment strategy is long for the indexed day. It enters when spot crosses from below to at or above the STH capitalized price and exits when the phase is classified as sell.";
    const CAPITAL_SENTIMENT_SHORT_DESCRIPTION: &str = "Whether the stateful capital-sentiment strategy is short for the indexed day; exactly the complement of `is_long`.";
    const UTXO_COUNT_DESCRIPTION: &str =
        "Number of transaction outputs tracked as unspent after each block.";
    const SIGHASH_ALL_DESCRIPTION: &str =
        "Counts transactions containing at least one detected `SIGHASH_ALL` signature.";
    const SIGHASH_NONE_DESCRIPTION: &str =
        "Counts transactions containing at least one detected `SIGHASH_NONE` signature.";
    const SIGHASH_SINGLE_DESCRIPTION: &str =
        "Counts transactions containing at least one detected `SIGHASH_SINGLE` signature.";
    const SIGHASH_DEFAULT_DESCRIPTION: &str =
        "Counts transactions containing at least one detected Taproot `SIGHASH_DEFAULT` signature.";
    const SIGHASH_ANYONE_CAN_PAY_DESCRIPTION: &str = "Counts transactions containing at least one detected signature with the `SIGHASH_ANYONECANPAY` modifier. This is counted independently from `SIGHASH_ALL`, `SIGHASH_NONE`, and `SIGHASH_SINGLE`.";
    const COINJOIN_DESCRIPTION: &str = "Counts transactions heuristically classified as CoinJoin candidates: at least five inputs and outputs, neither count five times the other, sufficiently repeated input/output values, no recognized address reuse, and no detected `OP_RETURN` or inscription.";
    const CONSOLIDATION_DESCRIPTION: &str =
        "Counts transactions with at least five times as many inputs as outputs.";
    const BATCH_PAYOUT_DESCRIPTION: &str =
        "Counts non-coinbase transactions with at least five times as many outputs as inputs.";
    const SIGOP_COST_DESCRIPTION: &str = "BIP-141 signature-operation cost. At `tx_index`, this is the cost of the indexed transaction. At `height`, this is the sum across every transaction in the block, including coinbase. Sigops in legacy scriptPubKeys, scriptSigs, and P2SH redeemScripts cost four units; P2WPKH and P2WSH sigops cost one. This statically counts signature-checking operations rather than signatures actually executed. Tapscript signature opcodes are excluded because BIP-342 uses a separate per-input execution budget. The post-SegWit consensus block limit is 80,000 cost units.";
    const DATE_DESCRIPTION: &str = "UTC calendar date in `YYYY-MM-DD` format associated with the time-period index. At `day1`, this is the represented calendar day. At coarser indexes, it is derived from the first monotonic block timestamp at or after the period.";
    const FIRST_HEIGHT_DESCRIPTION: &str = "Lowest block height whose resolution index is at least the requested index. Empty indexes therefore use the first height of the next populated index; indexes preceding the first populated one resolve to height 0.";
    const MINUTE10_INDEX_DESCRIPTION: &str = "Zero-based 10-minute UTC period containing the block's monotonic timestamp, counted from 2009-01-01 00:00:00 UTC.";
    const MINUTE30_INDEX_DESCRIPTION: &str = "Zero-based 30-minute UTC period containing the block's monotonic timestamp, counted from 2009-01-01 00:00:00 UTC.";
    const HOUR1_INDEX_DESCRIPTION: &str = "Zero-based one-hour UTC period containing the block's monotonic timestamp, counted from 2009-01-01 00:00:00 UTC.";
    const HOUR4_INDEX_DESCRIPTION: &str = "Zero-based four-hour UTC period containing the block's monotonic timestamp, counted from 2009-01-01 00:00:00 UTC.";
    const HOUR12_INDEX_DESCRIPTION: &str = "Zero-based 12-hour UTC period containing the block's monotonic timestamp, counted from 2009-01-01 00:00:00 UTC.";
    const DAY1_INDEX_DESCRIPTION: &str = "Zero-based UTC calendar day containing the block's monotonic timestamp, with 2009-01-01 equal to 0.";
    const DAY3_INDEX_DESCRIPTION: &str = "Zero-based three-day UTC period containing the block's monotonic timestamp, with period 1 beginning on 2009-01-03.";
    const EPOCH_INDEX_DESCRIPTION: &str = "Zero-based Bitcoin difficulty-adjustment epoch: block height divided by 2,016 using integer division.";
    const HALVING_INDEX_DESCRIPTION: &str = "Zero-based Bitcoin subsidy-halving epoch: block height divided by 210,000 using integer division.";
    const WEEK1_INDEX_DESCRIPTION: &str = "Zero-based ISO week containing the block's monotonic timestamp, counted from ISO week 1 of 2009.";
    const MONTH1_INDEX_DESCRIPTION: &str = "Zero-based UTC calendar month containing the block's monotonic timestamp, with January 2009 equal to 0.";
    const MONTH3_INDEX_DESCRIPTION: &str = "Zero-based UTC calendar quarter containing the block's monotonic timestamp, with Q1 2009 equal to 0.";
    const MONTH6_INDEX_DESCRIPTION: &str = "Zero-based UTC calendar half-year containing the block's monotonic timestamp, with the first half of 2009 equal to 0.";
    const YEAR1_INDEX_DESCRIPTION: &str = "Zero-based UTC calendar year containing the block's monotonic timestamp, with 2009 equal to 0.";
    const YEAR10_INDEX_DESCRIPTION: &str = "Zero-based ten-year UTC period containing the block's monotonic timestamp, with 2009 through 2018 equal to 0.";
    const TX_INDEX_DESCRIPTION: &str = "Global zero-based index of a transaction in canonical blockchain order. At `tx_index`, this is the identity value; at `txin_index`, it identifies the transaction containing the input; at type-specific output indexes, it identifies the transaction containing that output.";
    const TXIN_INDEX_DESCRIPTION: &str = "Global zero-based transaction-input index in canonical blockchain order. At `txin_index`, this is the identity value; at `txout_index`, it identifies the input that spends the output, with `u64::MAX` representing an unspent output.";
    const TXOUT_INDEX_DESCRIPTION: &str = "Global zero-based transaction-output index in canonical blockchain order. At `txout_index`, this is the identity value; at `txin_index`, it identifies the previous output spent by the input, with `u64::MAX` representing a coinbase input.";
    const INPUT_COUNT_DESCRIPTION: &str = "Number of inputs in the indexed transaction, including the coinbase transaction's single input.";
    const OUTPUT_COUNT_DESCRIPTION: &str = "Number of outputs in the indexed transaction.";
    const TX_INDEX_COUNT_DESCRIPTION: &str =
        "Number of transactions in the indexed block, including coinbase.";
    const MONOTONIC_TIMESTAMP_DESCRIPTION: &str = "Nondecreasing Unix timestamp in seconds at each block height, computed as the maximum of the current raw block-header timestamp and the preceding monotonic timestamp.";
    const P2PK33_DESCRIPTION: &str = "P2PK-shaped outputs containing a 33-byte key field; the field is not required to be a valid compressed public key.";
    const P2PK65_DESCRIPTION: &str = "P2PK-shaped outputs containing a 65-byte key field; the field is not required to be a valid uncompressed public key.";
    const P2PKH_DESCRIPTION: &str = "Pay-to-public-key-hash outputs.";
    const P2SH_DESCRIPTION: &str = "Pay-to-script-hash outputs.";
    const P2TR_DESCRIPTION: &str = "Pay-to-Taproot outputs.";
    const P2WPKH_DESCRIPTION: &str = "Version-0 pay-to-witness-public-key-hash outputs.";
    const P2WSH_DESCRIPTION: &str = "Version-0 pay-to-witness-script-hash outputs.";
    const P2A_DESCRIPTION: &str = "Pay-to-Anchor outputs matching `OP_1 PUSHBYTES_2 0x4e73`.";
    const P2MS_DESCRIPTION: &str = "Bare multisig outputs recognized by Bitcoin script parsing.";
    const EMPTY_OUTPUT_DESCRIPTION: &str = "Outputs with an empty locking script.";
    const UNKNOWN_OUTPUT_DESCRIPTION: &str =
        "Outputs not matching another recognized locking-script type.";
    const OP_RETURN_OUTPUT_DESCRIPTION: &str =
        "Outputs whose locking script begins with `OP_RETURN`.";
    const TYPE_SPECIFIC_ADDR_INDEX_DESCRIPTION: &str =
        "Zero-based type-specific address index assigned in first-seen canonical chain order.";
    const ADDR_TEXT_DESCRIPTION: &str = "Textual identifier reconstructed from the indexed locking script: raw public-key hex for P2PK, otherwise the standard mainnet Bitcoin address.";
    const TYPE_SPECIFIC_OUTPUT_INDEX_DESCRIPTION: &str =
        "Zero-based type-specific output index assigned in canonical chain order.";
    const RAW_ADDR_FIRST_INDEX_DESCRIPTION: &str = "Zero-based type-specific address index at the start of the indexed block, equal to the number of distinct addresses of this type first seen in preceding blocks.";
    const RAW_ADDR_BYTES_DESCRIPTION: &str = "Raw locking-script payload identifying the address, with script opcodes and push-length bytes removed.";
    const INDEXER_DIRECT_DESCRIPTIONS: &[(&str, &str)] = &[
        (
            "first_empty_output_index",
            "Zero-based type-specific output index at which the indexed block begins, equal to the number of outputs of this script type in preceding blocks.",
        ),
        (
            "first_op_return_index",
            "Zero-based OP_RETURN-output index at which the indexed block begins, equal to the number of OP_RETURN outputs in all preceding blocks.",
        ),
        (
            "first_p2ms_output_index",
            "Zero-based type-specific output index at which the indexed block begins, equal to the number of outputs of this script type in preceding blocks.",
        ),
        (
            "first_tx_index",
            "Global zero-based transaction index at which the indexed block begins, equal to the number of transactions in all preceding blocks.",
        ),
        (
            "first_txin_index",
            "Global zero-based transaction-input index in canonical blockchain order. At `height`, this is where the block begins and equals the number of inputs in preceding blocks; at `tx_index`, it identifies the transaction's first input.",
        ),
        (
            "first_txout_index",
            "Global zero-based transaction-output index in canonical blockchain order. At `height`, this is where the block begins and equals the number of outputs in preceding blocks; at `tx_index`, it identifies the transaction's first output.",
        ),
        (
            "first_unknown_output_index",
            "Zero-based type-specific output index at which the indexed block begins, equal to the number of outputs of this script type in preceding blocks.",
        ),
        (
            "has_p2pk",
            "Whether the transaction creates or, outside coinbase, spends at least one P2PK-shaped output with a 33- or 65-byte key field.",
        ),
        (
            "has_p2ms",
            "Whether the transaction creates or, outside coinbase, spends at least one bare multisig output recognized by Bitcoin script parsing.",
        ),
        (
            "has_p2pkh",
            "Whether the transaction creates or, outside coinbase, spends at least one pay-to-public-key-hash output.",
        ),
        (
            "has_p2sh",
            "Whether the transaction creates or, outside coinbase, spends at least one pay-to-script-hash output.",
        ),
        (
            "has_p2wpkh",
            "Whether the transaction creates or, outside coinbase, spends at least one version-0 pay-to-witness-public-key-hash output.",
        ),
        (
            "has_p2wsh",
            "Whether the transaction creates or, outside coinbase, spends at least one version-0 pay-to-witness-script-hash output.",
        ),
        (
            "has_p2tr",
            "Whether the transaction creates or, outside coinbase, spends at least one pay-to-Taproot output.",
        ),
        (
            "has_p2a",
            "Whether the transaction creates or, outside coinbase, spends at least one pay-to-Anchor output matching `OP_1 PUSHBYTES_2 0x4e73`.",
        ),
        (
            "has_op_return",
            "Whether the transaction creates or, outside coinbase, spends at least one output whose locking script begins with `OP_RETURN`.",
        ),
        (
            "has_empty",
            "Whether the transaction creates or, outside coinbase, spends at least one output with an empty locking script.",
        ),
        (
            "has_unknown",
            "Whether the transaction creates or, outside coinbase, spends at least one output not matching another recognized locking-script type.",
        ),
        (
            "has_fake_pubkey",
            "Whether the transaction creates a P2PK output whose shaped public key is invalid, or a bare multisig output containing an invalid or recognized burn public key.",
        ),
        (
            "has_fake_scripthash",
            "Whether the transaction contains a consecutive run of P2WSH outputs whose 32-byte programs encode a big-endian two-byte payload length and the required zero padding in the final program.",
        ),
        (
            "has_inscription",
            "Whether at least one Taproot script-path input contains the Ordinals envelope prefix `OP_0 OP_IF PUSH 'ord'` in its tapscript.",
        ),
        (
            "has_annex",
            "Whether at least one Taproot input with more than one witness element ends in an annex whose first byte is `0x50`.",
        ),
        (
            "has_sighash_all",
            "Whether the transaction contains at least one detected ECDSA or Schnorr signature encoding using `SIGHASH_ALL`.",
        ),
        (
            "has_sighash_none",
            "Whether the transaction contains at least one detected ECDSA or Schnorr signature encoding using `SIGHASH_NONE`.",
        ),
        (
            "has_sighash_single",
            "Whether the transaction contains at least one detected ECDSA or Schnorr signature encoding using `SIGHASH_SINGLE`.",
        ),
        (
            "has_sighash_default",
            "Whether the transaction contains at least one detected Taproot signature encoding using `SIGHASH_DEFAULT`.",
        ),
        (
            "has_sighash_anyone_can_pay",
            "Whether the transaction contains at least one detected ECDSA or Schnorr signature encoding with the `SIGHASH_ANYONECANPAY` modifier. This is independent of ALL, NONE, and SINGLE.",
        ),
        (
            "has_dust_output",
            "Whether a non-coinbase transaction has at least one output below BRK's type-specific dust threshold: 672 sats for P2PK65, 576 for P2PK33, 546 for P2PKH, 540 for P2SH, 294 for P2WPKH, 330 for P2WSH or P2TR, 240 for P2A, and 471 for an empty script. P2MS and unknown scripts use their computed minimal non-dust value; OP_RETURN is excluded.",
        ),
        (
            "kind",
            "Heuristic classification of the indexed OP_RETURN output from the first non-empty pushed-data prefix, or from the full post-OP_RETURN byte count for length-based formats. `runes` is recognized from an immediate `OP_13`; unmatched payloads are `text`, `bare_hash`, or `unknown`.",
        ),
        (
            "op_return_post_op_return_bytes",
            "Number of serialized locking-script bytes after the initial `OP_RETURN` opcode, including push opcodes and push-length prefixes.",
        ),
        (
            "outpoint",
            "Previous-output reference encoded as the global transaction index and zero-based output position within that transaction. Coinbase inputs use `u32::MAX` for both components.",
        ),
        (
            "output_type",
            "BRK locking-script classification of an output. At `txout_index`, this classifies the indexed output; at `txin_index`, it classifies the previous output spent by the input. Coinbase inputs use `unknown`.",
        ),
        (
            "p2ms_legacy_sigops",
            "BIP-141 signature-operation cost attributable to the indexed locking script using accurate multisig counting. Each `CHECKSIG` costs four; `CHECKMULTISIG` costs four times a preceding `OP_1` through `OP_16`, or 80 when no such key-count opcode precedes it.",
        ),
        (
            "raw_locktime",
            "Raw transaction `nLockTime`. Values below 500,000,000 represent block heights and values at or above it represent Unix timestamps; zero disables absolute locktime. This does not account for whether input sequence numbers make the locktime effective.",
        ),
        (
            "tx_count_one_input",
            "Number of transactions in the block with exactly one input, including the coinbase transaction.",
        ),
        (
            "tx_count_one_output",
            "Number of transactions in the block with exactly one output, including the coinbase transaction.",
        ),
        (
            "tx_count_p2pk",
            "Number of transactions in the block that create or, outside coinbase, spend at least one P2PK-shaped output with a 33- or 65-byte key field.",
        ),
        (
            "tx_count_p2ms",
            "Number of transactions in the block that create or, outside coinbase, spend at least one bare multisig output recognized by Bitcoin script parsing.",
        ),
        (
            "tx_count_p2pkh",
            "Number of transactions in the block that create or, outside coinbase, spend at least one pay-to-public-key-hash output.",
        ),
        (
            "tx_count_p2sh",
            "Number of transactions in the block that create or, outside coinbase, spend at least one pay-to-script-hash output.",
        ),
        (
            "tx_count_p2wpkh",
            "Number of transactions in the block that create or, outside coinbase, spend at least one version-0 pay-to-witness-public-key-hash output.",
        ),
        (
            "tx_count_p2wsh",
            "Number of transactions in the block that create or, outside coinbase, spend at least one version-0 pay-to-witness-script-hash output.",
        ),
        (
            "tx_count_p2tr",
            "Number of transactions in the block that create or, outside coinbase, spend at least one pay-to-Taproot output.",
        ),
        (
            "tx_count_p2a",
            "Number of transactions in the block that create or, outside coinbase, spend at least one pay-to-Anchor output matching `OP_1 PUSHBYTES_2 0x4e73`.",
        ),
        (
            "tx_count_op_return",
            "Number of transactions in the block that create or, outside coinbase, spend at least one output whose locking script begins with `OP_RETURN`.",
        ),
        (
            "tx_count_empty",
            "Number of transactions in the block that create or, outside coinbase, spend at least one output with an empty locking script.",
        ),
        (
            "tx_count_unknown",
            "Number of transactions in the block that create or, outside coinbase, spend at least one output not matching another recognized locking-script type.",
        ),
        (
            "tx_count_fake_pubkey",
            "Number of transactions in the block that create a P2PK output whose shaped public key is invalid, or a bare multisig output containing an invalid or recognized burn public key.",
        ),
        (
            "tx_count_fake_scripthash",
            "Number of transactions in the block containing a consecutive run of P2WSH outputs whose 32-byte programs encode a big-endian two-byte payload length and the required zero padding in the final program.",
        ),
        (
            "txid",
            "Transaction ID: the double-SHA256 hash of the transaction's non-witness serialization, displayed in Bitcoin's conventional hexadecimal byte order.",
        ),
        (
            "type_index",
            "Zero-based index within the output's BRK type-specific collection. At `txout_index`, this identifies the indexed output; at `txin_index`, it identifies the previous output spent by the input. Address types index distinct addresses, while other types index outputs in canonical order. Coinbase inputs use `u32::MAX`.",
        ),
        (
            "unknown_legacy_sigops",
            "BIP-141 signature-operation cost attributable to the indexed locking script using accurate multisig counting. Each `CHECKSIG` costs four; `CHECKMULTISIG` costs four times a preceding `OP_1` through `OP_16`, or 80 when no such key-count opcode precedes it.",
        ),
        (
            "value",
            "Value of the indexed transaction output in satoshis.",
        ),
    ];
    const MACD_DESCRIPTION: &str = "MACD chains at base intervals of 1, 7, and 30 days. Each chain uses fast, slow, and signal durations of 12, 26, and 9 times its base interval, respectively.";
    const MACD_EMA_FAST_DESCRIPTION: &str = "EMA of spot price in USD per BTC using the chain's fast span. It recursively applies `alpha = 2 / (span + 1)`, where `span` is the number of blocks in the corresponding trailing monotonic-time duration.";
    const MACD_EMA_SLOW_DESCRIPTION: &str = "EMA of spot price in USD per BTC using the chain's slow span. It recursively applies `alpha = 2 / (span + 1)`, where `span` is the number of blocks in the corresponding trailing monotonic-time duration.";
    const MACD_LINE_DESCRIPTION: &str = "Fast EMA minus slow EMA, in USD per BTC.";
    const MACD_SIGNAL_DESCRIPTION: &str = "EMA of the MACD line using the chain's signal span, in USD per BTC. It recursively applies `alpha = 2 / (span + 1)`, where `span` is the number of blocks in the corresponding trailing monotonic-time duration.";
    const MACD_HISTOGRAM_DESCRIPTION: &str = "MACD line minus signal line, in USD per BTC.";
    const SMALL_PLUGIN_DIRECT_DESCRIPTIONS: &[(&str, &str)] = &[
        (
            "coindays_destroyed_supply_adj",
            "Trailing 24-hour coin days destroyed divided by the current all-chain supply in BTC. Returns zero when supply is zero.",
        ),
        (
            "coinyears_destroyed_supply_adj",
            "Trailing 365-day total of coin days destroyed divided by the current all-chain supply in BTC. Despite the series name, the numerator is not divided by 365. Returns zero when supply is zero.",
        ),
        (
            "constant_0",
            "Constant numeric value 0 at every supported index.",
        ),
        (
            "constant_1",
            "Constant numeric value 1 at every supported index.",
        ),
        (
            "constant_2",
            "Constant numeric value 2 at every supported index.",
        ),
        (
            "constant_3",
            "Constant numeric value 3 at every supported index.",
        ),
        (
            "constant_4",
            "Constant numeric value 4 at every supported index.",
        ),
        (
            "constant_20",
            "Constant numeric value 20 at every supported index.",
        ),
        (
            "constant_30",
            "Constant numeric value 30 at every supported index.",
        ),
        (
            "constant_38_2",
            "Constant numeric value 38.2 at every supported index.",
        ),
        (
            "constant_50",
            "Constant numeric value 50 at every supported index.",
        ),
        (
            "constant_61_8",
            "Constant numeric value 61.8 at every supported index.",
        ),
        (
            "constant_70",
            "Constant numeric value 70 at every supported index.",
        ),
        (
            "constant_80",
            "Constant numeric value 80 at every supported index.",
        ),
        (
            "constant_100",
            "Constant numeric value 100 at every supported index.",
        ),
        (
            "constant_600",
            "Constant numeric value 600 at every supported index.",
        ),
        (
            "constant_minus_1",
            "Constant numeric value -1 at every supported index.",
        ),
        (
            "constant_minus_2",
            "Constant numeric value -2 at every supported index.",
        ),
        (
            "constant_minus_3",
            "Constant numeric value -3 at every supported index.",
        ),
        (
            "constant_minus_4",
            "Constant numeric value -4 at every supported index.",
        ),
        (
            "days_since_price_ath",
            "Fractional days, using monotonic block time, since the latest block whose spot price equaled the running all-time high. Resets to zero at equality.",
        ),
        (
            "years_since_price_ath",
            "`days_since_price_ath` divided by 365.",
        ),
        (
            "max_days_between_price_ath",
            "Running maximum of `days_since_price_ath`, including the current unfinished interval between all-time highs.",
        ),
        (
            "max_years_between_price_ath",
            "`max_days_between_price_ath` divided by 365.",
        ),
        (
            "price_true_range",
            "Absolute difference in cents per BTC between the current and previous block's spot prices. The first block is zero; this is not an OHLC range.",
        ),
        (
            "price_true_range_sum_2w",
            "Sum of `price_true_range` over the trailing 14-day monotonic-time window, in cents per BTC. This measures the spot-price path length over the window, not its high-low range.",
        ),
        (
            "price_ema_cents",
            "Exponential moving averages of spot price in cents per BTC. Each period recursively applies `alpha = 2 / (span + 1)`, where `span` is the number of blocks from the trailing period's monotonic-time start through the current block. Periods are 7, 8, 12, 13, 21, 26, 30, 34, 55, 89, 144, 200, 365, 730, 1,400, and 1,460 days, in column order.",
        ),
        (
            "dca_sats_per_day",
            "Satoshis purchased by investing 100 USD at each UTC daily close newly crossed at this block. It is zero within a day, includes every intervening daily purchase when block time skips days, and treats a missing or zero daily close as a zero purchase.",
        ),
        (
            "is_coinjoin",
            "Whether the transaction is heuristically classified as a CoinJoin candidate: at least five inputs and outputs, neither count five times the other, sufficiently repeated input/output values, no recognized address reuse, and no detected `OP_RETURN` or inscription.",
        ),
        (
            "is_consolidation",
            "Whether the transaction has at least five times as many inputs as outputs.",
        ),
        (
            "is_batch_payout",
            "Whether the transaction is non-coinbase and has at least five times as many outputs as inputs.",
        ),
        (
            "op_return_tx_count",
            "Number of transactions containing at least one `OP_RETURN` output; each transaction is counted once regardless of how many such outputs it has.",
        ),
        (
            "op_return_tx_vsize",
            "Sum of the full virtual sizes of transactions containing at least one `OP_RETURN` output; each transaction is included once.",
        ),
        (
            "op_return_fees",
            "Sum of the full fees of transactions containing at least one `OP_RETURN` output; each transaction is included once.",
        ),
        (
            "prevout_count_by_type",
            "Per-block counts of non-coinbase transaction inputs, partitioned by the BRK output type of the previous output they spend. Column order is P2PK65, P2PK33, P2PKH, P2MS, P2SH, P2WPKH, P2WSH, P2TR, P2A, unknown, and empty; `OP_RETURN` is excluded because it is unspendable.",
        ),
        (
            "output_count_by_type",
            "Per-block counts of transaction outputs, including coinbase, partitioned by BRK locking-script type. Column order is P2PK65, P2PK33, P2PKH, P2MS, P2SH, P2WPKH, P2WSH, P2TR, P2A, unknown, empty, and `OP_RETURN`.",
        ),
        (
            "velocity_btc",
            "Trailing 365-day transfer volume in satoshis divided by current all-chain supply in satoshis. Returns zero when supply is zero.",
        ),
        (
            "velocity_usd",
            "Trailing 365-day transfer volume valued in cents divided by current all-chain market capitalization in cents. Returns zero when market capitalization is zero.",
        ),
        (
            "dormancy_supply_adj",
            "Trailing 24-hour dormancy divided by the current all-chain supply in BTC. Dormancy is trailing 24-hour coin days destroyed divided by trailing 24-hour transfer volume in BTC. Returns zero when supply is zero.",
        ),
        (
            "dormancy_flow",
            "Current all-chain supply in BTC divided by trailing 24-hour dormancy. Dormancy is trailing 24-hour coin days destroyed divided by trailing 24-hour transfer volume in BTC. Returns zero when dormancy is zero.",
        ),
        (
            "stock_to_flow",
            "Current all-chain supply in satoshis divided by the current block's derived subsidy component annualized at 52,560 blocks. Returns zero when the annualized flow is zero.",
        ),
        (
            "seller_exhaustion",
            "Current all-chain supply-in-profit share multiplied by the population standard deviation of per-block trailing-24-hour spot-price returns over the trailing 30-day monotonic-time window, scaled by the square root of 30. Returns zero when total supply is zero.",
        ),
    ];
    const CAPITAL_SENTIMENT_PHASE_DESCRIPTION: &str = "Daily investor phase classified from the final available block's spot price relative to the all, STH, and LTH capitalized prices, with the one-year spot-price SMA used only for confirmation and disambiguation. Values are `raging_bull`, `bull`, `cautious_bull`, `hopeful_bull`, `early_bull`, `weak_bull`, `limbo`, `deep_bear`, `bear`, or `early_bear`. Missing when spot or any capitalized-price input is absent, nonpositive, or non-finite; the SMA itself may be unavailable.";
    const CAPITAL_SENTIMENT_SCORE_DESCRIPTION: &str = "Coarse directional score derived from `capital_sentiment_phase`: `raging_bull`, `bull`, and `early_bull` map to 2; `cautious_bull`, `hopeful_bull`, and `weak_bull` map to 1; `limbo` maps to -1; and `deep_bear`, `bear`, and `early_bear` map to -2. Missing when the phase is missing.";
    const BEDROCK_MODE_DESCRIPTIONS: &[(&str, &str)] = &[
        (
            "raw",
            "Uses the unweighted all-chain URPD and raw all-chain supply-in-loss share.",
        ),
        (
            "cointime",
            "Weights age cohorts by cointime wakefulness and calibrates against active supply in loss share.",
        ),
        (
            "coinflow",
            "Weights age cohorts by coinflow mobility and calibrates against coinflow-weighted supply in loss share.",
        ),
        (
            "coinflow_8y",
            "Weights age cohorts by their probability of spending within eight years, derived from coinflow spending rates, and calibrates against the corresponding horizon supply-in-loss share.",
        ),
        (
            "coinflow_4y",
            "Weights age cohorts by their probability of spending within four years, derived from coinflow spending rates, and calibrates against the corresponding horizon supply-in-loss share.",
        ),
        (
            "coinflow_2y",
            "Weights age cohorts by their probability of spending within two years, derived from coinflow spending rates, and calibrates against the corresponding horizon supply-in-loss share.",
        ),
        (
            "coinflow_1y",
            "Weights age cohorts by their probability of spending within one year, derived from coinflow spending rates, and calibrates against the corresponding horizon supply-in-loss share.",
        ),
        (
            "coinflow_6m",
            "Weights age cohorts by their probability of spending within six months, derived from coinflow spending rates, and calibrates against the corresponding horizon supply-in-loss share.",
        ),
        (
            "coinflow_3m",
            "Weights age cohorts by their probability of spending within three months, derived from coinflow spending rates, and calibrates against the corresponding horizon supply-in-loss share.",
        ),
        (
            "coinflow_1m",
            "Weights age cohorts by their probability of spending within one month, derived from coinflow spending rates, and calibrates against the corresponding horizon supply-in-loss share.",
        ),
    ];
    const BEDROCK_LOSS_THRESHOLD_DESCRIPTION: &str = "Linearly interpolated 95th, 98th, 99th, 99.5th, and 99.9th percentiles of the mode's prior finite daily supply-in-loss shares, in that column order. The current day is excluded from its calibration history and a value is unavailable until the current loss share exists and at least 365 prior observations are available. Stored as unitless decimal shares.";
    const BEDROCK_PRICE_BANDS_DESCRIPTION: &str = "Daily price bands derived from the mode's URPD. The five floor bands are the first ascending creation prices where the share of supply remaining above the price is at or below the corresponding calibrated loss threshold. The nine level bands are the 10th through 90th percentiles of supply at or above the 95th-percentile floor. The stored matrix column order is the five floors followed by the nine levels.";
    const RARITY_COMPONENT_BANDS_DESCRIPTION: &str = "Block-decay-weighted historical percentile bands of spot price divided by the component price named by the series. Observations begin at height 210,000, include the current block, and receive twice the weight every 210,000 blocks, equivalent to a 210,000-block backward half-life. Ratios are rounded to 0.001, clamped from 0 through 43, and NaNs are excluded. Percentiles are 0.1, 0.5, 1, 2, 5, 10, 20, 30, 40, 50, 60, 70, 80, 90, 95, 98, 99, 99.5, and 99.9 percent. Each price band is the component price multiplied by its percentile ratio.";
    const RARITY_COMPONENT_RATIOS_DESCRIPTION: &str = "Block-decay-weighted historical percentiles of spot price divided by the component price named by the series, stored in parts per million. Observations begin at height 210,000, include the current block, and receive twice the weight every 210,000 blocks, equivalent to a 210,000-block backward half-life. Ratios are rounded to 0.001, clamped from 0 through 43, and NaNs are excluded. Column order is 0.1, 0.5, 1, 2, 5, 10, 20, 30, 40, 50, 60, 70, 80, 90, 95, 98, 99, 99.5, and 99.9 percent.";
    const RARITY_INNER_DESCRIPTIONS: &[(&str, &str)] = &[
        (
            "rarity_meter",
            "Combined from ten component models: under-four-month, under-six-month, over-four-month, and over-six-month realized price; STH and LTH realized and capitalized price; and all-chain realized and capitalized price.",
        ),
        (
            "local_rarity_meter",
            "Combined from four young-coin models: under-four-month and under-six-month realized price plus STH realized and capitalized price.",
        ),
        (
            "cycle_rarity_meter",
            "Combined from six old-coin and all-chain models: over-four-month and over-six-month realized price, all-chain realized and capitalized price, and LTH realized and capitalized price.",
        ),
    ];
    const RARITY_PRICES_DESCRIPTION: &str = "Combined rarity price bands in cents per BTC. Each lower boundary from 0.1% through 5% is the maximum of that boundary across the selected components; each upper boundary from 95% through 99.9% is the minimum. The 10th through 90th percentiles are logarithmically interpolated between the combined 5th and 95th boundaries when both are positive, otherwise linearly interpolated. Column order follows the 19 rarity percentiles from 0.1% through 99.9%.";
    const RARITY_INDEX_DESCRIPTION: &str = "Position of spot price against the ten combined boundary bands: number of upper boundaries exceeded minus number of lower boundaries not reached. Ranges from -5 through 5.";
    const RARITY_SCORE_DESCRIPTION: &str = "Sum of the per-component rarity indexes, each calculated against that component's own ten boundary bands. Each selected component contributes from -5 through 5.";
    const RARITY_EXTREME_DESCRIPTIONS: &[(&str, &str)] = &[
        (
            "rarity_meter_coins_in_loss",
            "Upper-tail extremeness of total all-chain supply in loss, in BTC, using all prior finite positive observations. Outputs require 210,000 accepted historical observations; thresholds exclude the current block, while the reported tail share includes it as one observation.",
        ),
        (
            "rarity_meter_profit_taking",
            "Upper-tail extremeness of trailing-24-hour all-chain realized profit, in USD, using all prior finite observations. Outputs require 210,000 accepted historical observations; thresholds exclude the current block, while the reported tail share includes it as one observation.",
        ),
        (
            "rarity_meter_capitulation",
            "Upper-tail extremeness of trailing-24-hour all-chain realized loss, in USD, using all prior finite observations. Outputs require 210,000 accepted historical observations; thresholds exclude the current block, while the reported tail share includes it as one observation.",
        ),
        (
            "rarity_meter_peak_regret",
            "Upper-tail extremeness of trailing-24-hour all-chain realized peak regret, in USD, using all prior finite observations. Outputs require 210,000 accepted historical observations; thresholds exclude the current block, while the reported tail share includes it as one observation.",
        ),
        (
            "rarity_meter_seller_exhaustion",
            "Lower-tail extremeness of the trailing-24-hour all-chain sell-side risk ratio, expressed as a percentage, using the most recent 210,000 finite positive observations. Outputs require a full 210,000-observation history; thresholds exclude the current block, while the reported tail share includes it as one observation.",
        ),
    ];
    const RARITY_EXTREME_THRESHOLDS_DESCRIPTION: &str = "Historical source-value boundaries for 0.1%, 0.05%, and 0.025% tail shares, in that column order. Boundaries use the configured history before the current observation and the tail direction specified by the series' extreme model.";
    const RARITY_THRESHOLD_PCT0_1_DESCRIPTION: &str =
        "Source-value boundary for a 0.1% historical tail share.";
    const RARITY_THRESHOLD_PCT0_05_DESCRIPTION: &str =
        "Source-value boundary for a 0.05% historical tail share.";
    const RARITY_THRESHOLD_PCT0_025_DESCRIPTION: &str = "Source-value boundary for a 0.025% historical tail share. The public scalar series uses the unsuffixed `threshold` name.";
    const RARITY_EXTREME_RANK_DESCRIPTION: &str = "Discrete extremeness rank: 3 at or beyond the 0.025% tail boundary, 2 at or beyond 0.05%, 1 at or beyond 0.1%, and 0 otherwise or while the model is unavailable. Boundary direction is upper or lower as specified by the series' extreme model.";
    const COINBLOCKS_CREATED_DESCRIPTION: &str = "Coinblocks created by each block, equal to the circulating supply in BTC at that height. One coinblock is one BTC held for one block interval.";
    const COINBLOCKS_STORED_DESCRIPTION: &str = "Net coinblocks stored. Its cumulative value is cumulative coinblocks created minus cumulative coinblocks destroyed; its per-block value is the change in that cumulative stock.";
    const LIVELINESS_DESCRIPTION: &str =
        "Cumulative coinblocks destroyed divided by cumulative coinblocks created.";
    const VAULTEDNESS_DESCRIPTION: &str = "One minus liveliness.";
    const ACTIVITY_TO_VAULTEDNESS_DESCRIPTION: &str = "Liveliness divided by vaultedness.";
    const COINTIME_ADJ_INFLATION_DESCRIPTION: &str =
        "Supply inflation rate multiplied by the activity-to-vaultedness ratio.";
    const COINTIME_ADJ_NATIVE_VELOCITY_DESCRIPTION: &str =
        "Native transaction velocity multiplied by the activity-to-vaultedness ratio.";
    const COINTIME_ADJ_FIAT_VELOCITY_DESCRIPTION: &str =
        "Fiat transaction velocity multiplied by the activity-to-vaultedness ratio.";
    const THERMO_CAP_DESCRIPTION: &str = "Cumulative USD value, at each block's spot price, of the derived subsidy component equal to coinbase output value minus transaction fees.";
    const INVESTOR_CAP_DESCRIPTION: &str = "Realized capitalization minus thermo capitalization.";
    const VAULTED_CAP_DESCRIPTION: &str = "Realized capitalization multiplied by vaultedness.";
    const ACTIVE_CAP_DESCRIPTION: &str = "Realized capitalization multiplied by liveliness.";
    const COINTIME_CAP_DESCRIPTION: &str = "Cumulative cointime value destroyed multiplied by circulating supply, then divided by cumulative coinblocks stored.";
    const AVIV_DESCRIPTION: &str = "Active capitalization divided by investor capitalization.";
    const VAULTED_PRICE_DESCRIPTION: &str = "Realized price divided by vaultedness.";
    const ACTIVE_PRICE_DESCRIPTION: &str = "Realized price divided by liveliness.";
    const TRUE_MARKET_MEAN_DESCRIPTION: &str =
        "Investor capitalization divided by active supply in BTC.";
    const COINTIME_PRICE_DESCRIPTION: &str =
        "Cointime capitalization divided by circulating supply in BTC.";
    const RESERVE_RISK_DESCRIPTION: &str = "Spot price in USD divided by the HODL bank.";
    const VOCDD_MEDIAN_1Y_DESCRIPTION: &str = "Median per-block value of coin days destroyed over the trailing one-year timestamp window.";
    const HODL_BANK_DESCRIPTION: &str = "Cumulative sum of spot price in USD minus the trailing one-year median value of coin days destroyed.";
    const COINTIME_VALUE_DESTROYED_DESCRIPTION: &str =
        "Spot price in USD multiplied by coinblocks destroyed by the block.";
    const COINTIME_VALUE_CREATED_DESCRIPTION: &str =
        "Spot price in USD multiplied by coinblocks created by the block.";
    const COINTIME_VALUE_STORED_DESCRIPTION: &str =
        "Spot price in USD multiplied by net coinblocks stored by the block.";
    const VOCDD_DESCRIPTION: &str = "Supply-adjusted value of coin days destroyed: spot price in USD multiplied by the block's coin days destroyed and divided by circulating supply in BTC. Returns zero when circulating supply is zero.";
    const VAULTED_SUPPLY_DESCRIPTION: &str = "Circulating supply multiplied by vaultedness.";
    const ACTIVE_SUPPLY_DESCRIPTION: &str = "Circulating supply multiplied by liveliness.";
    const COINTIME_AWAKE_LOSS_DESCRIPTION: &str = "Share of awake supply that is in loss: the sum of supply in loss multiplied by wakefulness divided by the sum of total supply multiplied by wakefulness. Returns NaN when the weighted supply is zero.";
    const COINDAYS_CONSUMED_DESCRIPTION: &str = "Coin days destroyed by spent outputs, allocated across every age range the outputs traversed. The portion above a spent output's age-range lower bound remains in that range; each fully traversed younger range receives spent BTC multiplied by that range's duration. The allocation preserves total coin days destroyed.";
    const COINDAYS_STORED_DESCRIPTION: &str = "Cumulative coin days created in each age range minus cumulative coin days consumed from that range.";
    const WAKEFULNESS_DESCRIPTION: &str = "Wakefulness for each UTXO age range: cumulative coin days consumed from the range divided by cumulative coin days created in the range.";
    const DORMANCY_DESCRIPTION: &str = "One minus wakefulness for the selected age range.";
    const WAKEFULNESS_TO_DORMANCY_DESCRIPTION: &str =
        "Wakefulness divided by dormancy for the selected age range.";
    const AGE_AWAKE_SUPPLY_DESCRIPTION: &str = "Supply in each UTXO age range multiplied by that range's wakefulness. Each result is rounded down to whole satoshis.";
    const AGE_DORMANT_SUPPLY_DESCRIPTION: &str = "Supply in each UTXO age range multiplied by one minus that range's wakefulness. Each result is rounded down to whole satoshis.";
    const AGGREGATE_AWAKE_SUPPLY_DESCRIPTION: &str = "Sum of supply multiplied by wakefulness across the selected UTXO age ranges. Each age-range contribution is rounded down to whole satoshis.";
    const AGGREGATE_DORMANT_SUPPLY_DESCRIPTION: &str = "Sum of supply multiplied by one minus wakefulness across the selected UTXO age ranges. Each age-range contribution is rounded down to whole satoshis.";
    const AGGREGATE_AWAKE_CAP_DESCRIPTION: &str = "Sum of realized capitalization multiplied by wakefulness across the selected UTXO age ranges.";
    const AGGREGATE_AWAKE_PRICE_DESCRIPTION: &str = "Awake capitalization divided by awake supply in BTC. Returns zero when awake supply is zero.";
    const SPENDING_RATE_DESCRIPTION: &str = "Empirical daily spending hazard for each UTXO age range: cumulative transfer volume in BTC divided by cumulative coin days created in that range. Returns zero when cumulative coin days created is zero.";
    const SPENDING_EXPOSURE_DESCRIPTION: &str = "Estimated remaining-lifetime spending exposure for each UTXO age range. It integrates observed positive spending hazards from the range midpoint through subsequent complete ranges, then integrates an exponential tail fitted by duration-weighted regression of log hazard on age. Returns zero when a decreasing finite tail cannot be fitted.";
    const MOBILITY_DESCRIPTION: &str = "Estimated probability that supply in the selected age range will ever be spent: one minus exp of negative spending exposure. Nonpositive or NaN exposure returns zero; positive results are capped just below one.";
    const MOBILE_SUPPLY_DESCRIPTION: &str = "Supply multiplied by the estimated remaining-lifetime probability of spending for its UTXO age range. Each age-range contribution is rounded down to whole satoshis before aggregation.";
    const IMMOBILE_SUPPLY_DESCRIPTION: &str = "Supply multiplied by one minus the estimated remaining-lifetime probability of spending for its UTXO age range. Each age-range contribution is rounded down to whole satoshis before aggregation.";
    const COINFLOW_LOSS_DESCRIPTION: &str = "Share of estimated mobile supply that is in loss: the sum of supply in loss multiplied by remaining-lifetime spending probability divided by the sum of total supply multiplied by that probability. Returns NaN when the weighted supply is zero.";
    const COINFLOW_HORIZON_LOSS_DESCRIPTION: &str = "Share of supply likely to move within the named forward horizon that is currently in loss. Each age range is weighted by one minus exp of the negative sum of its observed spending hazards times days across that horizon. Returns NaN when the weighted supply is zero.";
    const COINFLOW_CAP_DESCRIPTION: &str = "Sum of realized capitalization multiplied by remaining-lifetime spending probability across the selected UTXO age ranges.";
    const COINFLOW_PRICE_DESCRIPTION: &str = "Coinflow capitalization divided by estimated mobile supply in BTC. Returns zero when mobile supply is zero.";
    const HORIZON_DESCRIPTIONS: [&str; 7] = [
        "Uses an eight-year forward spending horizon.",
        "Uses a four-year forward spending horizon.",
        "Uses a two-year forward spending horizon.",
        "Uses a one-year forward spending horizon.",
        "Uses a 180-day forward spending horizon.",
        "Uses a 90-day forward spending horizon.",
        "Uses a 30-day forward spending horizon.",
    ];
    const FRAMEWORK_SEMANTIC_DESCRIPTIONS: &[&str] = &[
        COINBLOCKS_CREATED_DESCRIPTION,
        COINBLOCKS_STORED_DESCRIPTION,
        LIVELINESS_DESCRIPTION,
        VAULTEDNESS_DESCRIPTION,
        ACTIVITY_TO_VAULTEDNESS_DESCRIPTION,
        COINTIME_ADJ_INFLATION_DESCRIPTION,
        COINTIME_ADJ_NATIVE_VELOCITY_DESCRIPTION,
        COINTIME_ADJ_FIAT_VELOCITY_DESCRIPTION,
        THERMO_CAP_DESCRIPTION,
        INVESTOR_CAP_DESCRIPTION,
        VAULTED_CAP_DESCRIPTION,
        ACTIVE_CAP_DESCRIPTION,
        COINTIME_CAP_DESCRIPTION,
        AVIV_DESCRIPTION,
        VAULTED_PRICE_DESCRIPTION,
        ACTIVE_PRICE_DESCRIPTION,
        TRUE_MARKET_MEAN_DESCRIPTION,
        COINTIME_PRICE_DESCRIPTION,
        RESERVE_RISK_DESCRIPTION,
        VOCDD_MEDIAN_1Y_DESCRIPTION,
        HODL_BANK_DESCRIPTION,
        COINTIME_VALUE_DESTROYED_DESCRIPTION,
        COINTIME_VALUE_CREATED_DESCRIPTION,
        COINTIME_VALUE_STORED_DESCRIPTION,
        VOCDD_DESCRIPTION,
        VAULTED_SUPPLY_DESCRIPTION,
        ACTIVE_SUPPLY_DESCRIPTION,
        COINTIME_AWAKE_LOSS_DESCRIPTION,
        COINDAYS_CONSUMED_DESCRIPTION,
        COINDAYS_STORED_DESCRIPTION,
        WAKEFULNESS_DESCRIPTION,
        DORMANCY_DESCRIPTION,
        WAKEFULNESS_TO_DORMANCY_DESCRIPTION,
        AGE_AWAKE_SUPPLY_DESCRIPTION,
        AGE_DORMANT_SUPPLY_DESCRIPTION,
        AGGREGATE_AWAKE_SUPPLY_DESCRIPTION,
        AGGREGATE_DORMANT_SUPPLY_DESCRIPTION,
        AGGREGATE_AWAKE_CAP_DESCRIPTION,
        AGGREGATE_AWAKE_PRICE_DESCRIPTION,
        SPENDING_RATE_DESCRIPTION,
        SPENDING_EXPOSURE_DESCRIPTION,
        MOBILITY_DESCRIPTION,
        MOBILE_SUPPLY_DESCRIPTION,
        IMMOBILE_SUPPLY_DESCRIPTION,
        COINFLOW_LOSS_DESCRIPTION,
        COINFLOW_HORIZON_LOSS_DESCRIPTION,
        COINFLOW_CAP_DESCRIPTION,
        COINFLOW_PRICE_DESCRIPTION,
    ];
    const TIMESTAMP_DESCRIPTION: &str = "Unix timestamp in seconds associated with the indexed block or time period. Block-header timestamps are not guaranteed to increase between consecutive heights.";
    const BLOCK_DESCRIPTION: &str = "Value for the represented block. At time-period indexes, the value is taken from the period's final block.";
    const CUMULATIVE_DESCRIPTION: &str = "Cumulative value through the represented block. At time-period indexes, the value is taken at the period's final block.";
    const ROLLING_SUM_DESCRIPTION: &str = "Total of the per-block values over the trailing window ending at the represented block. At time-period indexes, the value is taken at the period's final block.";
    const ROLLING_AVERAGE_DESCRIPTION: &str = "Arithmetic mean of the per-block values over the trailing window ending at the represented block; each block has equal weight. At time-period indexes, the value is taken at the period's final block.";
    const MIN_DESCRIPTION: &str = "Minimum value in the represented distribution.";
    const MAX_DESCRIPTION: &str = "Maximum value in the represented distribution.";
    const PCT10_DESCRIPTION: &str = "10th percentile of the represented distribution.";
    const PCT25_DESCRIPTION: &str = "25th percentile of the represented distribution.";
    const MEDIAN_DESCRIPTION: &str = "Median of the represented distribution.";
    const PCT75_DESCRIPTION: &str = "75th percentile of the represented distribution.";
    const PCT90_DESCRIPTION: &str = "90th percentile of the represented distribution.";
    const WINDOW_DESCRIPTIONS: [&str; 4] = [
        "Uses a trailing 24-hour window.",
        "Uses a trailing 7-day window.",
        "Uses a trailing 30-day window.",
        "Uses a trailing 365-day window.",
    ];

    #[test]
    fn exposes_only_audited_descriptions() {
        std::thread::Builder::new()
            .stack_size(8 * 1024 * 1024)
            .spawn(assert_audited_descriptions)
            .unwrap()
            .join()
            .unwrap();
    }

    fn assert_audited_descriptions() {
        let directory = tempfile::tempdir().unwrap();
        let client = Client::new("http://127.0.0.1:1", Auth::None).unwrap();
        let reader = Reader::new_without_rlimit(directory.path().join("blocks"), &client);
        let indexer = Indexer::import(directory.path(), &reader).unwrap();
        let computer = Computer::forced_import(directory.path(), &indexer).unwrap();
        let vecs = Vecs::build_rw(&indexer, &computer);

        let txin_index = SeriesName::from("txin_index");
        let spent = vecs.get_entry(&txin_index, Index::TxOutIndex).unwrap();
        let outputs = spent.plugin();
        assert_eq!(outputs.id(), "outputs");
        assert!(outputs.mutates_existing(spent.vec()));
        assert!(spent.requires_gate());

        let identity = vecs.get_entry(&txin_index, Index::TxInIndex).unwrap();
        let indexes = identity.plugin();
        assert_eq!(indexes.id(), "indexes");
        assert!(!indexes.mutates_existing(identity.vec()));
        assert!(!identity.requires_gate());

        let any_addr_index = SeriesName::from("any_addr_index");
        let address = vecs
            .get_entry(&any_addr_index, Index::P2AAddrIndex)
            .unwrap();
        let distribution = address.plugin();
        assert_eq!(distribution.id(), "distribution");
        assert!(distribution.mutates_existing(address.vec()));
        assert!(address.requires_gate());

        for (name, representation) in [
            ("realized_price", USD_DESCRIPTION),
            ("realized_price_cents", CENTS_DESCRIPTION),
            ("realized_price_sats", SATS_DESCRIPTION),
            ("realized_price_ratio_ppm", PPM_DESCRIPTION),
            ("realized_price_ratio", RATIO_DESCRIPTION),
            ("lth_realized_price", USD_DESCRIPTION),
            ("lth_realized_price_cents", CENTS_DESCRIPTION),
            ("lth_realized_price_sats", SATS_DESCRIPTION),
            ("lth_realized_price_ratio_ppm", PPM_DESCRIPTION),
            ("lth_realized_price_ratio", RATIO_DESCRIPTION),
            ("realized_price_cents_by_aggregate", CENTS_DESCRIPTION),
        ] {
            let info = vecs.series_info(&SeriesName::from(name)).unwrap();
            assert_eq!(
                info.description.as_deref(),
                Some(format!("{REALIZED_PRICE_DESCRIPTION} {representation}").as_str()),
                "wrong description for {name}"
            );
        }

        for (name, description) in [
            ("timestamp", TIMESTAMP_DESCRIPTION),
            ("utxo_count_bis", UTXO_COUNT_DESCRIPTION),
            (
                "capital_sentiment_is_long",
                CAPITAL_SENTIMENT_LONG_DESCRIPTION,
            ),
            (
                "capital_sentiment_is_short",
                CAPITAL_SENTIMENT_SHORT_DESCRIPTION,
            ),
        ] {
            let info = vecs.series_info(&SeriesName::from(name)).unwrap();
            assert_eq!(
                info.description.as_deref(),
                Some(description),
                "wrong description for {name}"
            );
        }

        for (name, description) in [
            ("price", USD_DESCRIPTION),
            ("price_cents", CENTS_DESCRIPTION),
            ("price_sats", SATS_DESCRIPTION),
            ("price_close", USD_DESCRIPTION),
            ("price_close_cents", CENTS_DESCRIPTION),
            ("price_close_sats", SATS_DESCRIPTION),
            ("supply_in_profit", AMOUNT_BTC_DESCRIPTION),
            ("supply_in_profit_sats", AMOUNT_SATS_DESCRIPTION),
            ("supply_in_profit_usd", AMOUNT_USD_DESCRIPTION),
            ("supply_in_profit_cents", AMOUNT_CENTS_DESCRIPTION),
            ("dca_stack_from_2020", AMOUNT_BTC_DESCRIPTION),
            ("dca_stack_from_2020_sats", AMOUNT_SATS_DESCRIPTION),
            ("dca_stack_from_2020_usd", AMOUNT_USD_DESCRIPTION),
            ("dca_stack_from_2020_cents", AMOUNT_CENTS_DESCRIPTION),
            ("lump_sum_stack_1y", AMOUNT_BTC_DESCRIPTION),
            ("lump_sum_stack_1y_sats", AMOUNT_SATS_DESCRIPTION),
            ("lump_sum_stack_1y_usd", AMOUNT_USD_DESCRIPTION),
            ("lump_sum_stack_1y_cents", AMOUNT_CENTS_DESCRIPTION),
            ("all_supply_in_profit_share_ppm", GENERIC_PPM_DESCRIPTION),
            (
                "all_supply_in_profit_share_ratio",
                GENERIC_RATIO_DESCRIPTION,
            ),
            ("all_supply_in_profit_share", PERCENT_DESCRIPTION),
        ] {
            let info = vecs
                .series_info(&SeriesName::from(name))
                .unwrap_or_else(|| panic!("missing series {name}"));
            assert_eq!(
                info.description.as_deref(),
                Some(description),
                "wrong description for {name}"
            );
        }

        let info = vecs.series_info(&SeriesName::from("total_size")).unwrap();
        assert_eq!(
            info.description.as_deref(),
            Some(TOTAL_SIZE_DESCRIPTION),
            "wrong description for total_size"
        );
        assert_eq!(info.indexes, vec![Index::Height, Index::TxIndex]);

        for (name, description) in [
            ("block_weight", BLOCK_WEIGHT_DESCRIPTION),
            ("segwit_txs", SEGWIT_TXS_DESCRIPTION),
            ("segwit_size", SEGWIT_SIZE_DESCRIPTION),
            ("segwit_weight", SEGWIT_WEIGHT_DESCRIPTION),
        ] {
            let info = vecs.series_info(&SeriesName::from(name)).unwrap();
            assert_eq!(
                info.description.as_deref(),
                Some(description),
                "wrong description for {name}"
            );
        }

        for (name, metric, aggregation) in [
            ("block_vbytes", BLOCK_VBYTES_DESCRIPTION, BLOCK_DESCRIPTION),
            (
                "block_size_cumulative",
                BLOCK_SIZE_DESCRIPTION,
                CUMULATIVE_DESCRIPTION,
            ),
            (
                "block_weight_cumulative",
                BLOCK_WEIGHT_DESCRIPTION,
                CUMULATIVE_DESCRIPTION,
            ),
            (
                "block_vbytes_cumulative",
                BLOCK_VBYTES_DESCRIPTION,
                CUMULATIVE_DESCRIPTION,
            ),
        ] {
            let info = vecs.series_info(&SeriesName::from(name)).unwrap();
            assert_eq!(
                info.description.as_deref(),
                Some(format!("{metric} {aggregation}").as_str()),
                "wrong description for {name}"
            );
        }

        for (name, metric, aggregation, window) in [
            (
                "block_size_sum_24h",
                BLOCK_SIZE_DESCRIPTION,
                ROLLING_SUM_DESCRIPTION,
                WINDOW_DESCRIPTIONS[0],
            ),
            (
                "block_vbytes_average_1w",
                BLOCK_VBYTES_DESCRIPTION,
                ROLLING_AVERAGE_DESCRIPTION,
                WINDOW_DESCRIPTIONS[1],
            ),
            (
                "block_weight_sum_1m",
                BLOCK_WEIGHT_DESCRIPTION,
                ROLLING_SUM_DESCRIPTION,
                WINDOW_DESCRIPTIONS[2],
            ),
            (
                "block_size_min_24h",
                BLOCK_SIZE_DESCRIPTION,
                MIN_DESCRIPTION,
                WINDOW_DESCRIPTIONS[0],
            ),
            (
                "block_weight_median_1w",
                BLOCK_WEIGHT_DESCRIPTION,
                MEDIAN_DESCRIPTION,
                WINDOW_DESCRIPTIONS[1],
            ),
            (
                "block_vbytes_pct90_1y",
                BLOCK_VBYTES_DESCRIPTION,
                PCT90_DESCRIPTION,
                WINDOW_DESCRIPTIONS[3],
            ),
        ] {
            let info = vecs.series_info(&SeriesName::from(name)).unwrap();
            assert_eq!(
                info.description.as_deref(),
                Some(format!("{metric} {aggregation} {window}").as_str()),
                "wrong description for {name}"
            );
        }

        for (name, description) in [
            ("difficulty", DIFFICULTY_DESCRIPTION),
            ("difficulty_hashrate", DIFFICULTY_HASHRATE_DESCRIPTION),
            ("difficulty_epoch", DIFFICULTY_EPOCH_DESCRIPTION),
            ("blocks_to_retarget", BLOCKS_TO_RETARGET_DESCRIPTION),
            ("days_to_retarget", DAYS_TO_RETARGET_DESCRIPTION),
        ] {
            let info = vecs.series_info(&SeriesName::from(name)).unwrap();
            assert_eq!(
                info.description.as_deref(),
                Some(description),
                "wrong description for {name}"
            );
        }

        for (name, representation) in [
            ("difficulty_adjustment_ppm", GENERIC_PPM_DESCRIPTION),
            ("difficulty_adjustment_ratio", GENERIC_RATIO_DESCRIPTION),
            ("difficulty_adjustment", PERCENT_DESCRIPTION),
        ] {
            let info = vecs.series_info(&SeriesName::from(name)).unwrap();
            assert_eq!(
                info.description.as_deref(),
                Some(format!("{DIFFICULTY_ADJUSTMENT_DESCRIPTION} {representation}").as_str()),
                "wrong description for {name}"
            );
        }

        for (name, description) in [
            ("halving_epoch", HALVING_EPOCH_DESCRIPTION),
            ("blocks_to_halving", BLOCKS_TO_HALVING_DESCRIPTION),
            ("days_to_halving", DAYS_TO_HALVING_DESCRIPTION),
        ] {
            let info = vecs.series_info(&SeriesName::from(name)).unwrap();
            assert_eq!(
                info.description.as_deref(),
                Some(description),
                "wrong description for {name}"
            );
        }

        for (name, description) in [
            ("blockhash", BLOCKHASH_DESCRIPTION),
            ("coinbase_tag", COINBASE_TAG_DESCRIPTION),
        ] {
            let info = vecs.series_info(&SeriesName::from(name)).unwrap();
            assert_eq!(
                info.description.as_deref(),
                Some(description),
                "wrong description for {name}"
            );
        }

        for suffix in [
            "1h", "24h", "3d", "1w", "8d", "9d", "12d", "13d", "2w", "21d", "26d", "1m", "34d",
            "50d", "55d", "2m", "9w", "12w", "89d", "3m", "14w", "111d", "144d", "6m", "26w",
            "200d", "9m", "350d", "12m", "1y", "14m", "2y", "26m", "3y", "200w", "4y", "5y", "6y",
            "8y", "9y", "10y", "12y", "14y", "26y",
        ] {
            let name = format!("height_{suffix}_ago");
            let info = vecs.series_info(&SeriesName::from(name.as_str())).unwrap();
            assert_eq!(
                info.description.as_deref(),
                Some(LOOKBACK_HEIGHT_DESCRIPTION),
                "wrong description for {name}"
            );
        }

        for (name, metric, aggregation, representation) in [
            (
                "coinbase",
                COINBASE_REWARD_DESCRIPTION,
                BLOCK_DESCRIPTION,
                AMOUNT_BTC_DESCRIPTION,
            ),
            (
                "coinbase_cumulative_sats",
                COINBASE_REWARD_DESCRIPTION,
                CUMULATIVE_DESCRIPTION,
                AMOUNT_SATS_DESCRIPTION,
            ),
            (
                "subsidy_usd",
                DERIVED_SUBSIDY_DESCRIPTION,
                BLOCK_DESCRIPTION,
                AMOUNT_USD_DESCRIPTION,
            ),
            (
                "fees_cumulative_cents",
                TRANSACTION_FEES_DESCRIPTION,
                CUMULATIVE_DESCRIPTION,
                AMOUNT_CENTS_DESCRIPTION,
            ),
            (
                "unclaimed_rewards",
                UNCLAIMED_REWARDS_DESCRIPTION,
                BLOCK_DESCRIPTION,
                AMOUNT_BTC_DESCRIPTION,
            ),
        ] {
            let info = vecs.series_info(&SeriesName::from(name)).unwrap();
            assert_eq!(
                info.description.as_deref(),
                Some(format!("{metric} {aggregation} {representation}").as_str()),
                "wrong description for {name}"
            );
        }

        let info = vecs
            .series_info(&SeriesName::from("output_volume"))
            .unwrap();
        assert_eq!(
            info.description.as_deref(),
            Some(OUTPUT_VOLUME_DESCRIPTION),
            "wrong description for output_volume"
        );

        for (name, metric, window, representation) in [
            (
                "fee_dominance_ppm",
                FEE_DOMINANCE_DESCRIPTION,
                None,
                GENERIC_PPM_DESCRIPTION,
            ),
            (
                "fee_dominance_24h_ratio",
                FEE_DOMINANCE_DESCRIPTION,
                Some(WINDOW_DESCRIPTIONS[0]),
                GENERIC_RATIO_DESCRIPTION,
            ),
            (
                "subsidy_dominance_1m",
                SUBSIDY_DOMINANCE_DESCRIPTION,
                Some(WINDOW_DESCRIPTIONS[2]),
                PERCENT_DESCRIPTION,
            ),
            (
                "fee_to_subsidy_1y_ppm",
                FEE_TO_SUBSIDY_DESCRIPTION,
                Some(WINDOW_DESCRIPTIONS[3]),
                GENERIC_PPM_DESCRIPTION,
            ),
        ] {
            let expected = match window {
                Some(window) => format!("{metric} {window} {representation}"),
                None => format!("{metric} {representation}"),
            };
            let info = vecs.series_info(&SeriesName::from(name)).unwrap();
            assert_eq!(
                info.description.as_deref(),
                Some(expected.as_str()),
                "wrong description for {name}"
            );
        }

        for name in [
            "hash_rate_sma_1w",
            "hash_rate_sma_1m",
            "hash_rate_sma_2m",
            "hash_rate_sma_1y",
        ] {
            let info = vecs.series_info(&SeriesName::from(name)).unwrap();
            assert_eq!(
                info.description.as_deref(),
                Some(HASH_RATE_SMA_DESCRIPTION),
                "wrong description for {name}"
            );
        }

        for (name, description) in [
            ("hash_rate", HASH_RATE_DESCRIPTION),
            ("hash_rate_ath", HASH_RATE_ATH_DESCRIPTION),
        ] {
            let info = vecs.series_info(&SeriesName::from(name)).unwrap();
            assert_eq!(
                info.description.as_deref(),
                Some(description),
                "wrong description for {name}"
            );
        }

        for (suffix, representation) in [
            ("ppm", GENERIC_PPM_DESCRIPTION),
            ("ratio", GENERIC_RATIO_DESCRIPTION),
            ("", PERCENT_DESCRIPTION),
        ] {
            let name = if suffix.is_empty() {
                "hash_rate_drawdown".to_string()
            } else {
                format!("hash_rate_drawdown_{suffix}")
            };
            let info = vecs.series_info(&SeriesName::from(name.as_str())).unwrap();
            assert_eq!(
                info.description.as_deref(),
                Some(format!("{HASH_RATE_DRAWDOWN_DESCRIPTION} {representation}").as_str()),
                "wrong description for {name}"
            );
        }

        for (prefix, metric) in [
            ("hash_price", HASH_PRICE_DESCRIPTION),
            ("hash_value", HASH_VALUE_DESCRIPTION),
        ] {
            for (suffix, detail) in [
                ("ths", PER_THS_DESCRIPTION),
                ("ths_min", THS_MIN_DESCRIPTION),
                ("phs", PER_PHS_DESCRIPTION),
                ("phs_min", PHS_MIN_DESCRIPTION),
            ] {
                let name = format!("{prefix}_{suffix}");
                let info = vecs.series_info(&SeriesName::from(name.as_str())).unwrap();
                assert_eq!(
                    info.description.as_deref(),
                    Some(format!("{metric} {detail}").as_str()),
                    "wrong description for {name}"
                );
            }

            for (suffix, representation) in [
                ("ppm", GENERIC_PPM_DESCRIPTION),
                ("ratio", GENERIC_RATIO_DESCRIPTION),
                ("", PERCENT_DESCRIPTION),
            ] {
                let name = if suffix.is_empty() {
                    format!("{prefix}_rebound")
                } else {
                    format!("{prefix}_rebound_{suffix}")
                };
                let info = vecs.series_info(&SeriesName::from(name.as_str())).unwrap();
                assert_eq!(
                    info.description.as_deref(),
                    Some(format!("{metric} {HASH_REBOUND_DESCRIPTION} {representation}").as_str()),
                    "wrong description for {name}"
                );
            }
        }

        for (name, aggregation) in [
            ("tx_count", BLOCK_DESCRIPTION),
            ("tx_count_cumulative", CUMULATIVE_DESCRIPTION),
        ] {
            let info = vecs.series_info(&SeriesName::from(name)).unwrap();
            assert_eq!(
                info.description.as_deref(),
                Some(format!("{TX_COUNT_DESCRIPTION} {aggregation}").as_str()),
                "wrong description for {name}"
            );
        }

        for (suffix, window) in ["24h", "1w", "1m", "1y"]
            .into_iter()
            .zip(WINDOW_DESCRIPTIONS)
        {
            for (kind, aggregation) in [
                ("sum", ROLLING_SUM_DESCRIPTION),
                ("average", ROLLING_AVERAGE_DESCRIPTION),
            ] {
                let name = format!("tx_count_{kind}_{suffix}");
                let info = vecs.series_info(&SeriesName::from(name.as_str())).unwrap();
                assert_eq!(
                    info.description.as_deref(),
                    Some(format!("{TX_COUNT_DESCRIPTION} {aggregation} {window}").as_str()),
                    "wrong description for {name}"
                );
            }

            for (stat, description) in [
                ("min", MIN_DESCRIPTION),
                ("max", MAX_DESCRIPTION),
                ("pct10", PCT10_DESCRIPTION),
                ("pct25", PCT25_DESCRIPTION),
                ("median", MEDIAN_DESCRIPTION),
                ("pct75", PCT75_DESCRIPTION),
                ("pct90", PCT90_DESCRIPTION),
            ] {
                let name = format!("tx_count_{stat}_{suffix}");
                let info = vecs.series_info(&SeriesName::from(name.as_str())).unwrap();
                assert_eq!(
                    info.description.as_deref(),
                    Some(format!("{TX_COUNT_DESCRIPTION} {description} {window}").as_str()),
                    "wrong description for {name}"
                );
            }
        }

        for (name, description) in [
            ("input_value", TX_INPUT_VALUE_DESCRIPTION),
            ("output_value", TX_OUTPUT_VALUE_DESCRIPTION),
            ("fee_rate", TX_FEE_RATE_DESCRIPTION),
            ("is_cpfp_parent", CPFP_PARENT_DESCRIPTION),
            ("is_cpfp_child", CPFP_CHILD_DESCRIPTION),
        ] {
            let info = vecs.series_info(&SeriesName::from(name)).unwrap();
            assert_eq!(
                info.description.as_deref(),
                Some(description),
                "wrong description for {name}"
            );
        }

        for (metric_name, metric) in [
            ("fee", TX_FEE_DESCRIPTION),
            ("effective_fee_rate", TX_EFFECTIVE_FEE_RATE_DESCRIPTION),
        ] {
            let info = vecs.series_info(&SeriesName::from(metric_name)).unwrap();
            assert_eq!(
                info.description.as_deref(),
                Some(metric),
                "wrong description for {metric_name}"
            );

            for prefix in [metric_name.to_string(), format!("{metric_name}_6b")] {
                for (stat, description) in [
                    ("min", MIN_DESCRIPTION),
                    ("max", MAX_DESCRIPTION),
                    ("pct10", PCT10_DESCRIPTION),
                    ("pct25", PCT25_DESCRIPTION),
                    ("median", MEDIAN_DESCRIPTION),
                    ("pct75", PCT75_DESCRIPTION),
                    ("pct90", PCT90_DESCRIPTION),
                ] {
                    let name = format!("{prefix}_{stat}");
                    let info = vecs.series_info(&SeriesName::from(name.as_str())).unwrap();
                    assert_eq!(
                        info.description.as_deref(),
                        Some(format!("{metric} {description}").as_str()),
                        "wrong description for {name}"
                    );
                }
            }
        }

        for (prefix, metric) in [
            ("cpfp_parent_count", CPFP_PARENT_COUNT_DESCRIPTION),
            ("cpfp_child_count", CPFP_CHILD_COUNT_DESCRIPTION),
        ] {
            for (name, aggregation) in [
                (prefix.to_string(), BLOCK_DESCRIPTION),
                (format!("{prefix}_cumulative"), CUMULATIVE_DESCRIPTION),
            ] {
                let info = vecs.series_info(&SeriesName::from(name.as_str())).unwrap();
                assert_eq!(
                    info.description.as_deref(),
                    Some(format!("{metric} {aggregation}").as_str()),
                    "wrong description for {name}"
                );
            }

            for (suffix, window) in ["24h", "1w", "1m", "1y"]
                .into_iter()
                .zip(WINDOW_DESCRIPTIONS)
            {
                for (kind, aggregation) in [
                    ("sum", ROLLING_SUM_DESCRIPTION),
                    ("average", ROLLING_AVERAGE_DESCRIPTION),
                ] {
                    let name = format!("{prefix}_{kind}_{suffix}");
                    let info = vecs.series_info(&SeriesName::from(name.as_str())).unwrap();
                    assert_eq!(
                        info.description.as_deref(),
                        Some(format!("{metric} {aggregation} {window}").as_str()),
                        "wrong description for {name}"
                    );
                }
            }
        }

        for (name, description) in [
            ("tx_version", TX_VERSION_CATEGORY_DESCRIPTION),
            ("tx_count_v1", TX_COUNT_V1_DESCRIPTION),
            ("tx_count_v2", TX_COUNT_V2_DESCRIPTION),
            ("tx_count_v3", TX_COUNT_V3_DESCRIPTION),
            ("tx_count_other_version", TX_COUNT_OTHER_VERSION_DESCRIPTION),
        ] {
            let info = vecs.series_info(&SeriesName::from(name)).unwrap();
            assert_eq!(
                info.description.as_deref(),
                Some(description),
                "wrong description for {name}"
            );
        }

        for (name, description) in [
            ("is_explicitly_rbf", IS_EXPLICITLY_RBF_DESCRIPTION),
            ("tx_count_explicitly_rbf", EXPLICITLY_RBF_COUNT_DESCRIPTION),
        ] {
            let info = vecs.series_info(&SeriesName::from(name)).unwrap();
            assert_eq!(
                info.description.as_deref(),
                Some(description),
                "wrong description for {name}"
            );
        }

        for (prefix, metric) in [
            ("tx_v1", TX_VERSION_1_DESCRIPTION),
            ("tx_v2", TX_VERSION_2_DESCRIPTION),
            ("tx_v3", TX_VERSION_3_DESCRIPTION),
            ("tx_other_version", TX_OTHER_VERSION_DESCRIPTION),
        ] {
            for (name, aggregation) in [
                (prefix.to_string(), BLOCK_DESCRIPTION),
                (format!("{prefix}_cumulative"), CUMULATIVE_DESCRIPTION),
            ] {
                let info = vecs.series_info(&SeriesName::from(name.as_str())).unwrap();
                assert_eq!(
                    info.description.as_deref(),
                    Some(
                        format!("{TX_VERSION_COUNTS_DESCRIPTION} {metric} {aggregation}").as_str()
                    ),
                    "wrong description for {name}"
                );
            }

            for (suffix, window) in ["24h", "1w", "1m", "1y"]
                .into_iter()
                .zip(WINDOW_DESCRIPTIONS)
            {
                for (kind, aggregation) in [
                    ("sum", ROLLING_SUM_DESCRIPTION),
                    ("average", ROLLING_AVERAGE_DESCRIPTION),
                ] {
                    let name = format!("{prefix}_{kind}_{suffix}");
                    let info = vecs.series_info(&SeriesName::from(name.as_str())).unwrap();
                    assert_eq!(
                        info.description.as_deref(),
                        Some(
                            format!(
                                "{TX_VERSION_COUNTS_DESCRIPTION} {metric} {aggregation} {window}"
                            )
                            .as_str()
                        ),
                        "wrong description for {name}"
                    );
                }
            }
        }

        let info = vecs
            .series_info(&SeriesName::from("is_nonstandard"))
            .unwrap();
        assert_eq!(
            info.description.as_deref(),
            Some(format!("{TX_POLICY_DESCRIPTION} {IS_NONSTANDARD_DESCRIPTION}").as_str()),
            "wrong description for is_nonstandard"
        );

        let info = vecs
            .series_info(&SeriesName::from("nonstandard_count"))
            .unwrap();
        assert_eq!(
            info.description.as_deref(),
            Some(format!("{TX_POLICY_DESCRIPTION} {NONSTANDARD_COUNT_DESCRIPTION}").as_str()),
            "wrong description for nonstandard_count"
        );

        let info = vecs
            .series_info(&SeriesName::from("nonstandard_count_cumulative"))
            .unwrap();
        assert_eq!(
            info.description.as_deref(),
            Some(
                format!(
                    "{TX_POLICY_DESCRIPTION} {NONSTANDARD_COUNT_DESCRIPTION} {CUMULATIVE_DESCRIPTION}"
                )
                .as_str()
            ),
            "wrong description for nonstandard_count_cumulative"
        );

        for (suffix, window) in ["24h", "1w", "1m", "1y"]
            .into_iter()
            .zip(WINDOW_DESCRIPTIONS)
        {
            for (kind, aggregation) in [
                ("sum", ROLLING_SUM_DESCRIPTION),
                ("average", ROLLING_AVERAGE_DESCRIPTION),
            ] {
                let name = format!("nonstandard_count_{kind}_{suffix}");
                let info = vecs.series_info(&SeriesName::from(name.as_str())).unwrap();
                assert_eq!(
                    info.description.as_deref(),
                    Some(
                        format!(
                            "{TX_POLICY_DESCRIPTION} {NONSTANDARD_COUNT_DESCRIPTION} {aggregation} {window}"
                        )
                        .as_str()
                    ),
                    "wrong description for {name}"
                );
            }
        }

        for (metric_name, metric) in [
            ("tx_vsize", TX_VSIZE_DESCRIPTION),
            ("tx_weight", TX_WEIGHT_DESCRIPTION),
        ] {
            let info = vecs.series_info(&SeriesName::from(metric_name)).unwrap();
            assert_eq!(
                info.description.as_deref(),
                Some(metric),
                "wrong description for {metric_name}"
            );

            for prefix in [metric_name.to_string(), format!("{metric_name}_6b")] {
                for (stat, description) in [
                    ("min", MIN_DESCRIPTION),
                    ("max", MAX_DESCRIPTION),
                    ("pct10", PCT10_DESCRIPTION),
                    ("pct25", PCT25_DESCRIPTION),
                    ("median", MEDIAN_DESCRIPTION),
                    ("pct75", PCT75_DESCRIPTION),
                    ("pct90", PCT90_DESCRIPTION),
                ] {
                    let name = format!("{prefix}_{stat}");
                    let info = vecs.series_info(&SeriesName::from(name.as_str())).unwrap();
                    assert_eq!(
                        info.description.as_deref(),
                        Some(format!("{metric} {description}").as_str()),
                        "wrong description for {name}"
                    );
                }
            }
        }

        for (unit_suffix, representation) in [
            ("", AMOUNT_BTC_DESCRIPTION),
            ("_sats", AMOUNT_SATS_DESCRIPTION),
            ("_usd", AMOUNT_USD_DESCRIPTION),
            ("_cents", AMOUNT_CENTS_DESCRIPTION),
        ] {
            for (base, aggregation) in [
                ("transfer_volume_bis", BLOCK_DESCRIPTION),
                ("transfer_volume_bis_cumulative", CUMULATIVE_DESCRIPTION),
            ] {
                let name = format!("{base}{unit_suffix}");
                let info = vecs.series_info(&SeriesName::from(name.as_str())).unwrap();
                assert_eq!(
                    info.description.as_deref(),
                    Some(
                        format!("{TRANSFER_VOLUME_DESCRIPTION} {aggregation} {representation}")
                            .as_str()
                    ),
                    "wrong description for {name}"
                );
            }

            for (suffix, window) in ["24h", "1w", "1m", "1y"]
                .into_iter()
                .zip(WINDOW_DESCRIPTIONS)
            {
                for (kind, aggregation) in [
                    ("sum", ROLLING_SUM_DESCRIPTION),
                    ("average", ROLLING_AVERAGE_DESCRIPTION),
                ] {
                    let name = format!("transfer_volume_bis_{kind}_{suffix}{unit_suffix}");
                    let info = vecs.series_info(&SeriesName::from(name.as_str())).unwrap();
                    assert_eq!(
                        info.description.as_deref(),
                        Some(
                            format!(
                                "{TRANSFER_VOLUME_DESCRIPTION} {aggregation} {window} {representation}"
                            )
                            .as_str()
                        ),
                        "wrong description for {name}"
                    );
                }
            }
        }

        for (suffix, window) in ["24h", "1w", "1m", "1y"]
            .into_iter()
            .zip(WINDOW_DESCRIPTIONS)
        {
            let name = format!("tx_per_sec_{suffix}");
            let info = vecs.series_info(&SeriesName::from(name.as_str())).unwrap();
            assert_eq!(
                info.description.as_deref(),
                Some(format!("{TX_PER_SECOND_DESCRIPTION} {window}").as_str()),
                "wrong description for {name}"
            );
        }

        for (name, representation) in [
            ("block_fullness_ppm", GENERIC_PPM_DESCRIPTION),
            ("block_fullness_ratio", GENERIC_RATIO_DESCRIPTION),
            ("block_fullness", PERCENT_DESCRIPTION),
        ] {
            let info = vecs.series_info(&SeriesName::from(name)).unwrap();
            assert_eq!(
                info.description.as_deref(),
                Some(format!("{BLOCK_FULLNESS_DESCRIPTION} {representation}").as_str()),
                "wrong description for {name}"
            );
        }

        for (name, window) in ["24h", "1w", "1m", "1y"]
            .into_iter()
            .zip(WINDOW_DESCRIPTIONS)
        {
            let name = format!("block_count_target_{name}");
            let info = vecs.series_info(&SeriesName::from(name.as_str())).unwrap();
            assert_eq!(
                info.description.as_deref(),
                Some(format!("{BLOCK_COUNT_TARGET_DESCRIPTION} {window}").as_str()),
                "wrong description for {name}"
            );
        }

        for (name, metric, aggregation) in [
            ("block_count", BLOCK_COUNT_DESCRIPTION, BLOCK_DESCRIPTION),
            (
                "block_count_cumulative",
                BLOCK_COUNT_DESCRIPTION,
                CUMULATIVE_DESCRIPTION,
            ),
            (
                "block_interval",
                BLOCK_INTERVAL_DESCRIPTION,
                BLOCK_DESCRIPTION,
            ),
        ] {
            let info = vecs.series_info(&SeriesName::from(name)).unwrap();
            assert_eq!(
                info.description.as_deref(),
                Some(format!("{metric} {aggregation}").as_str()),
                "wrong description for {name}"
            );
        }

        for (name, metric, aggregation, window) in [
            (
                "block_count_sum_24h",
                BLOCK_COUNT_DESCRIPTION,
                ROLLING_SUM_DESCRIPTION,
                WINDOW_DESCRIPTIONS[0],
            ),
            (
                "block_count_average_1w",
                BLOCK_COUNT_DESCRIPTION,
                ROLLING_AVERAGE_DESCRIPTION,
                WINDOW_DESCRIPTIONS[1],
            ),
            (
                "block_interval_average_24h",
                BLOCK_INTERVAL_DESCRIPTION,
                ROLLING_AVERAGE_DESCRIPTION,
                WINDOW_DESCRIPTIONS[0],
            ),
            (
                "block_interval_average_1y",
                BLOCK_INTERVAL_DESCRIPTION,
                ROLLING_AVERAGE_DESCRIPTION,
                WINDOW_DESCRIPTIONS[3],
            ),
        ] {
            let info = vecs.series_info(&SeriesName::from(name)).unwrap();
            assert_eq!(
                info.description.as_deref(),
                Some(format!("{metric} {aggregation} {window}").as_str()),
                "wrong description for {name}"
            );
        }

        for (name, metric) in [
            ("tx_count_sighash_all", SIGHASH_ALL_DESCRIPTION),
            ("tx_count_sighash_none", SIGHASH_NONE_DESCRIPTION),
            ("tx_count_sighash_single", SIGHASH_SINGLE_DESCRIPTION),
            ("tx_count_sighash_default", SIGHASH_DEFAULT_DESCRIPTION),
            (
                "tx_count_sighash_anyone_can_pay",
                SIGHASH_ANYONE_CAN_PAY_DESCRIPTION,
            ),
            ("coinjoin_count", COINJOIN_DESCRIPTION),
            ("consolidation_count", CONSOLIDATION_DESCRIPTION),
            ("batch_payout_count", BATCH_PAYOUT_DESCRIPTION),
            (
                "utxos_under_1h_old_coindays_created",
                COINDAYS_CREATED_DESCRIPTION,
            ),
        ] {
            let info = vecs.series_info(&SeriesName::from(name)).unwrap();
            assert_eq!(
                info.description.as_deref(),
                Some(format!("{metric} {BLOCK_DESCRIPTION}").as_str()),
                "wrong description for {name}"
            );
        }

        let info = vecs
            .series_info(&SeriesName::from("total_sigop_cost"))
            .unwrap();
        assert_eq!(
            info.description.as_deref(),
            Some(SIGOP_COST_DESCRIPTION),
            "wrong description for total_sigop_cost"
        );
        assert_eq!(info.indexes, vec![Index::Height, Index::TxIndex]);

        let info = vecs
            .series_info(&SeriesName::from("total_sigop_cost_cumulative"))
            .unwrap();
        assert_eq!(
            info.description.as_deref(),
            Some(format!("{SIGOP_COST_DESCRIPTION} {CUMULATIVE_DESCRIPTION}").as_str()),
            "wrong description for total_sigop_cost_cumulative"
        );

        for (suffix, window) in ["24h", "1w", "1m", "1y"]
            .into_iter()
            .zip(WINDOW_DESCRIPTIONS)
        {
            for (kind, aggregation) in [
                ("sum", ROLLING_SUM_DESCRIPTION),
                ("average", ROLLING_AVERAGE_DESCRIPTION),
            ] {
                let name = format!("total_sigop_cost_{kind}_{suffix}");
                let info = vecs.series_info(&SeriesName::from(name.as_str())).unwrap();
                assert_eq!(
                    info.description.as_deref(),
                    Some(format!("{SIGOP_COST_DESCRIPTION} {aggregation} {window}").as_str()),
                    "wrong description for {name}"
                );
            }
        }

        for (name, metric) in [
            ("coinjoin_count_cumulative", COINJOIN_DESCRIPTION),
            (
                "utxos_age_range_coindays_created_cumulative",
                COINDAYS_CREATED_DESCRIPTION,
            ),
        ] {
            let info = vecs.series_info(&SeriesName::from(name)).unwrap();
            assert_eq!(
                info.description.as_deref(),
                Some(format!("{metric} {CUMULATIVE_DESCRIPTION}").as_str()),
                "wrong description for {name}"
            );
        }

        for (suffix, window) in ["24h", "1w", "1m", "1y"]
            .into_iter()
            .zip(WINDOW_DESCRIPTIONS)
        {
            for (kind, aggregation) in [
                ("sum", ROLLING_SUM_DESCRIPTION),
                ("average", ROLLING_AVERAGE_DESCRIPTION),
            ] {
                let name = format!("coinjoin_count_{kind}_{suffix}");
                let info = vecs.series_info(&SeriesName::from(name.as_str())).unwrap();
                assert_eq!(
                    info.description.as_deref(),
                    Some(format!("{COINJOIN_DESCRIPTION} {aggregation} {window}").as_str()),
                    "wrong description for {name}"
                );
            }
        }

        let info = vecs.series_info(&SeriesName::from("date")).unwrap();
        assert_eq!(info.description.as_deref(), Some(DATE_DESCRIPTION));
        assert_eq!(
            info.indexes,
            vec![
                Index::Day1,
                Index::Day3,
                Index::Week1,
                Index::Month1,
                Index::Month3,
                Index::Month6,
                Index::Year1,
                Index::Year10,
            ]
        );

        let info = vecs.series_info(&SeriesName::from("first_height")).unwrap();
        assert_eq!(info.description.as_deref(), Some(FIRST_HEIGHT_DESCRIPTION));
        assert_eq!(
            info.indexes,
            vec![
                Index::Minute10,
                Index::Minute30,
                Index::Hour1,
                Index::Hour4,
                Index::Hour12,
                Index::Day1,
                Index::Day3,
                Index::Week1,
                Index::Month1,
                Index::Month3,
                Index::Month6,
                Index::Year1,
                Index::Year10,
                Index::Halving,
                Index::Epoch,
            ]
        );

        for (name, description) in [
            ("minute10", MINUTE10_INDEX_DESCRIPTION),
            ("minute30", MINUTE30_INDEX_DESCRIPTION),
            ("hour1", HOUR1_INDEX_DESCRIPTION),
            ("hour4", HOUR4_INDEX_DESCRIPTION),
            ("hour12", HOUR12_INDEX_DESCRIPTION),
            ("day1", DAY1_INDEX_DESCRIPTION),
            ("day3", DAY3_INDEX_DESCRIPTION),
            ("epoch", EPOCH_INDEX_DESCRIPTION),
            ("halving", HALVING_INDEX_DESCRIPTION),
            ("week1", WEEK1_INDEX_DESCRIPTION),
            ("month1", MONTH1_INDEX_DESCRIPTION),
            ("month3", MONTH3_INDEX_DESCRIPTION),
            ("month6", MONTH6_INDEX_DESCRIPTION),
            ("year1", YEAR1_INDEX_DESCRIPTION),
            ("year10", YEAR10_INDEX_DESCRIPTION),
        ] {
            let info = vecs.series_info(&SeriesName::from(name)).unwrap();
            assert_eq!(
                info.description.as_deref(),
                Some(description),
                "wrong description for {name}"
            );
            assert_eq!(info.indexes, vec![Index::Height]);
        }

        for (prefix, index, output_type) in [
            ("p2pk33", Index::P2PK33AddrIndex, P2PK33_DESCRIPTION),
            ("p2pk65", Index::P2PK65AddrIndex, P2PK65_DESCRIPTION),
            ("p2pkh", Index::P2PKHAddrIndex, P2PKH_DESCRIPTION),
            ("p2sh", Index::P2SHAddrIndex, P2SH_DESCRIPTION),
            ("p2tr", Index::P2TRAddrIndex, P2TR_DESCRIPTION),
            ("p2wpkh", Index::P2WPKHAddrIndex, P2WPKH_DESCRIPTION),
            ("p2wsh", Index::P2WSHAddrIndex, P2WSH_DESCRIPTION),
            ("p2a", Index::P2AAddrIndex, P2A_DESCRIPTION),
        ] {
            for (suffix, detail) in [
                ("addr_index", TYPE_SPECIFIC_ADDR_INDEX_DESCRIPTION),
                ("addr", ADDR_TEXT_DESCRIPTION),
            ] {
                let name = format!("{prefix}_{suffix}");
                let info = vecs.series_info(&SeriesName::from(name.as_str())).unwrap();
                assert_eq!(
                    info.description.as_deref(),
                    Some(format!("{output_type} {detail}").as_str()),
                    "wrong description for {name}"
                );
                assert_eq!(info.indexes, vec![index]);
            }
        }

        for (name, index, output_type) in [
            (
                "p2ms_output_index",
                Index::P2MSOutputIndex,
                P2MS_DESCRIPTION,
            ),
            (
                "empty_output_index",
                Index::EmptyOutputIndex,
                EMPTY_OUTPUT_DESCRIPTION,
            ),
            (
                "unknown_output_index",
                Index::UnknownOutputIndex,
                UNKNOWN_OUTPUT_DESCRIPTION,
            ),
            (
                "op_return_index",
                Index::OpReturnIndex,
                OP_RETURN_OUTPUT_DESCRIPTION,
            ),
        ] {
            let info = vecs.series_info(&SeriesName::from(name)).unwrap();
            assert_eq!(
                info.description.as_deref(),
                Some(format!("{output_type} {TYPE_SPECIFIC_OUTPUT_INDEX_DESCRIPTION}").as_str()),
                "wrong description for {name}"
            );
            assert_eq!(info.indexes, vec![index]);
        }

        for (name, description, indexes) in [
            (
                "tx_index",
                TX_INDEX_DESCRIPTION,
                vec![
                    Index::TxIndex,
                    Index::TxInIndex,
                    Index::EmptyOutputIndex,
                    Index::OpReturnIndex,
                    Index::P2MSOutputIndex,
                    Index::UnknownOutputIndex,
                ],
            ),
            (
                "txin_index",
                TXIN_INDEX_DESCRIPTION,
                vec![Index::TxInIndex, Index::TxOutIndex],
            ),
            (
                "txout_index",
                TXOUT_INDEX_DESCRIPTION,
                vec![Index::TxInIndex, Index::TxOutIndex],
            ),
            ("input_count", INPUT_COUNT_DESCRIPTION, vec![Index::TxIndex]),
            (
                "output_count",
                OUTPUT_COUNT_DESCRIPTION,
                vec![Index::TxIndex],
            ),
            (
                "tx_index_count",
                TX_INDEX_COUNT_DESCRIPTION,
                vec![Index::Height],
            ),
            (
                "timestamp_monotonic",
                MONOTONIC_TIMESTAMP_DESCRIPTION,
                vec![Index::Height],
            ),
        ] {
            let info = vecs.series_info(&SeriesName::from(name)).unwrap();
            assert_eq!(
                info.description.as_deref(),
                Some(description),
                "wrong description for {name}"
            );
            assert_eq!(info.indexes, indexes, "wrong indexes for {name}");
        }

        let undocumented_indexes = vecs
            .series_to_index_to_vec
            .iter()
            .filter(|(_, index_to_vec)| {
                index_to_vec.description().is_none()
                    && index_to_vec
                        .values()
                        .any(|entry| entry.plugin().id() == "indexes")
            })
            .map(|(name, _)| *name)
            .collect::<Vec<_>>();
        assert!(
            undocumented_indexes.is_empty(),
            "undocumented indexes series: {undocumented_indexes:#?}"
        );

        for (name, description) in INDEXER_DIRECT_DESCRIPTIONS {
            let info = vecs.series_info(&SeriesName::from(*name)).unwrap();
            assert_eq!(
                info.description.as_deref(),
                Some(*description),
                "wrong description for {name}"
            );
        }

        for (prefix, output_type) in [
            ("p2pk65", P2PK65_DESCRIPTION),
            ("p2pk33", P2PK33_DESCRIPTION),
            ("p2pkh", P2PKH_DESCRIPTION),
            ("p2sh", P2SH_DESCRIPTION),
            ("p2wpkh", P2WPKH_DESCRIPTION),
            ("p2wsh", P2WSH_DESCRIPTION),
            ("p2tr", P2TR_DESCRIPTION),
            ("p2a", P2A_DESCRIPTION),
        ] {
            for (name, detail) in [
                (
                    format!("first_{prefix}_addr_index"),
                    RAW_ADDR_FIRST_INDEX_DESCRIPTION,
                ),
                (format!("{prefix}_bytes"), RAW_ADDR_BYTES_DESCRIPTION),
            ] {
                let info = vecs.series_info(&SeriesName::from(name.as_str())).unwrap();
                assert_eq!(
                    info.description.as_deref(),
                    Some(format!("{output_type} {detail}").as_str()),
                    "wrong description for {name}"
                );
            }
        }

        let undocumented_indexer = vecs
            .series_to_index_to_vec
            .iter()
            .filter(|(_, index_to_vec)| {
                index_to_vec.description().is_none()
                    && index_to_vec
                        .values()
                        .any(|entry| entry.plugin().id() == "indexer")
            })
            .map(|(name, _)| *name)
            .collect::<Vec<_>>();
        assert!(
            undocumented_indexer.is_empty(),
            "undocumented indexer series: {undocumented_indexer:#?}"
        );

        assert_eq!(SMALL_PLUGIN_DIRECT_DESCRIPTIONS.len(), 42);
        assert_eq!(SMALL_PLUGIN_DIRECT_DESCRIPTIONS.len() + 3 * 5, 57);
        for (name, description) in SMALL_PLUGIN_DIRECT_DESCRIPTIONS {
            let info = vecs.series_info(&SeriesName::from(*name)).unwrap();
            assert_eq!(
                info.description.as_deref(),
                Some(*description),
                "wrong description for {name}"
            );
        }

        for suffix in ["24h", "1w", "1m"] {
            for (field, detail) in [
                ("ema_fast", MACD_EMA_FAST_DESCRIPTION),
                ("ema_slow", MACD_EMA_SLOW_DESCRIPTION),
                ("line", MACD_LINE_DESCRIPTION),
                ("signal", MACD_SIGNAL_DESCRIPTION),
                ("histogram", MACD_HISTOGRAM_DESCRIPTION),
            ] {
                let name = format!("macd_{field}_{suffix}");
                let expected = format!("{MACD_DESCRIPTION} {detail}");
                let info = vecs.series_info(&SeriesName::from(name.as_str())).unwrap();
                assert_eq!(
                    info.description.as_deref(),
                    Some(expected.as_str()),
                    "wrong description for {name}"
                );
            }
        }

        let small_plugins = [
            "constants",
            "indicators",
            "inputs",
            "investing",
            "market",
            "op_return",
            "outputs",
            "supply",
            "transactions",
        ];
        let mut undocumented_small_plugins = Vec::new();
        for (name, index_to_vec) in &vecs.series_to_index_to_vec {
            if index_to_vec.description().is_some() {
                continue;
            }
            for plugin in small_plugins {
                if index_to_vec
                    .values()
                    .any(|entry| entry.plugin().id() == plugin)
                {
                    undocumented_small_plugins.push((plugin, *name));
                }
            }
        }
        assert!(
            undocumented_small_plugins.is_empty(),
            "undocumented small-plugin series: {undocumented_small_plugins:#?}"
        );

        let mut audited_model_roots = 0;
        for (mode, mode_description) in BEDROCK_MODE_DESCRIPTIONS {
            for percentile in ["pct95", "pct98", "pct99", "pct99_5", "pct99_9"] {
                let name = format!("bedrock_{mode}_loss_threshold_{percentile}");
                let expected = format!("{mode_description} {BEDROCK_LOSS_THRESHOLD_DESCRIPTION}");
                let info = vecs.series_info(&SeriesName::from(name.as_str())).unwrap();
                assert_eq!(
                    info.description.as_deref(),
                    Some(expected.as_str()),
                    "wrong description for {name}"
                );
                audited_model_roots += 1;
            }
        }

        for component in [
            "realized_price",
            "capitalized_price",
            "sth_realized_price",
            "sth_capitalized_price",
            "lth_realized_price",
            "lth_capitalized_price",
            "over_6m_realized_price",
            "over_4m_realized_price",
            "under_4m_realized_price",
            "under_6m_realized_price",
            "vaulted_price",
            "active_price",
            "true_market_mean_price",
            "cointime_price",
            "coinflow_price",
        ] {
            let name = format!("{component}_ratios_ppm");
            let info = vecs.series_info(&SeriesName::from(name.as_str())).unwrap();
            assert_eq!(
                info.description.as_deref(),
                Some(RARITY_COMPONENT_RATIOS_DESCRIPTION),
                "wrong description for {name}"
            );
            audited_model_roots += 1;
        }

        for (name, description) in [
            (
                "capital_sentiment_phase",
                CAPITAL_SENTIMENT_PHASE_DESCRIPTION,
            ),
            (
                "capital_sentiment_score",
                CAPITAL_SENTIMENT_SCORE_DESCRIPTION,
            ),
        ] {
            let info = vecs.series_info(&SeriesName::from(name)).unwrap();
            assert_eq!(
                info.description.as_deref(),
                Some(description),
                "wrong description for {name}"
            );
            audited_model_roots += 1;
        }

        for (prefix, context) in RARITY_INNER_DESCRIPTIONS {
            for (suffix, detail) in [
                ("percentiles_cents", RARITY_PRICES_DESCRIPTION),
                ("index", RARITY_INDEX_DESCRIPTION),
                ("score", RARITY_SCORE_DESCRIPTION),
            ] {
                let name = format!("{prefix}_{suffix}");
                let expected = format!("{context} {detail}");
                let info = vecs.series_info(&SeriesName::from(name.as_str())).unwrap();
                assert_eq!(
                    info.description.as_deref(),
                    Some(expected.as_str()),
                    "wrong description for {name}"
                );
                audited_model_roots += 1;
            }
        }

        for (prefix, context) in RARITY_EXTREME_DESCRIPTIONS {
            for (suffix, detail) in [
                ("thresholds", None),
                (
                    "threshold_pct0_1",
                    Some(RARITY_THRESHOLD_PCT0_1_DESCRIPTION),
                ),
                (
                    "threshold_pct0_05",
                    Some(RARITY_THRESHOLD_PCT0_05_DESCRIPTION),
                ),
                ("threshold", Some(RARITY_THRESHOLD_PCT0_025_DESCRIPTION)),
            ] {
                let name = format!("{prefix}_{suffix}");
                let expected = match detail {
                    Some(detail) => {
                        format!("{context} {RARITY_EXTREME_THRESHOLDS_DESCRIPTION} {detail}")
                    }
                    None => format!("{context} {RARITY_EXTREME_THRESHOLDS_DESCRIPTION}"),
                };
                let info = vecs.series_info(&SeriesName::from(name.as_str())).unwrap();
                assert_eq!(
                    info.description.as_deref(),
                    Some(expected.as_str()),
                    "wrong description for {name}"
                );
                audited_model_roots += 1;
            }

            let name = format!("{prefix}_rank");
            let expected = format!("{context} {RARITY_EXTREME_RANK_DESCRIPTION}");
            let info = vecs.series_info(&SeriesName::from(name.as_str())).unwrap();
            assert_eq!(
                info.description.as_deref(),
                Some(expected.as_str()),
                "wrong description for {name}"
            );
            audited_model_roots += 1;
        }
        assert_eq!(audited_model_roots, 101);

        let undocumented_models = vecs
            .series_to_index_to_vec
            .iter()
            .filter(|(_, index_to_vec)| {
                index_to_vec.description().is_none()
                    && index_to_vec
                        .values()
                        .any(|entry| entry.plugin().id() == "models")
            })
            .map(|(name, _)| *name)
            .collect::<Vec<_>>();
        assert!(
            undocumented_models.is_empty(),
            "undocumented models series: {undocumented_models:#?}"
        );

        let undocumented_frameworks = vecs
            .series_to_index_to_vec
            .iter()
            .filter(|(_, index_to_vec)| {
                index_to_vec.description().is_none()
                    && index_to_vec
                        .values()
                        .any(|entry| entry.plugin().id() == "frameworks")
            })
            .map(|(name, _)| *name)
            .collect::<Vec<_>>();
        assert!(
            undocumented_frameworks.is_empty(),
            "undocumented frameworks series: {undocumented_frameworks:#?}"
        );

        let generic_only_frameworks = vecs
            .series_to_index_to_vec
            .iter()
            .filter(|(_, index_to_vec)| {
                index_to_vec
                    .values()
                    .any(|entry| entry.plugin().id() == "frameworks")
                    && index_to_vec.description().is_some_and(|description| {
                        !FRAMEWORK_SEMANTIC_DESCRIPTIONS
                            .iter()
                            .any(|semantic| description.contains(semantic))
                    })
            })
            .map(|(name, index_to_vec)| (*name, index_to_vec.description().unwrap()))
            .collect::<Vec<_>>();
        assert!(
            generic_only_frameworks.is_empty(),
            "frameworks series with only generic descriptions: {generic_only_frameworks:#?}"
        );

        let mut audited_framework_roots = 0;
        let mut assert_framework_description = |name: &str, expected: &str| {
            let info = vecs.series_info(&SeriesName::from(name)).unwrap();
            assert_eq!(
                info.description.as_deref(),
                Some(expected),
                "wrong description for {name}"
            );
            audited_framework_roots += 1;
        };

        for age in [
            "under_1h",
            "1h_to_1d",
            "1d_to_1w",
            "1w_to_1m",
            "1m_to_2m",
            "2m_to_3m",
            "3m_to_4m",
            "4m_to_5m",
            "5m_to_6m",
            "6m_to_9m",
            "9m_to_1y",
            "1y_to_18m",
            "18m_to_2y",
            "2y_to_3y",
            "3y_to_4y",
            "4y_to_5y",
            "5y_to_6y",
            "6y_to_7y",
            "7y_to_8y",
            "8y_to_10y",
            "10y_to_12y",
            "12y_to_15y",
            "over_15y",
        ] {
            let prefix = format!("utxos_{age}_old");
            for (suffix, expected) in [
                ("wakefulness", WAKEFULNESS_DESCRIPTION.to_owned()),
                (
                    "dormancy",
                    format!("{WAKEFULNESS_DESCRIPTION} {DORMANCY_DESCRIPTION}"),
                ),
                (
                    "wakefulness_to_dormancy",
                    format!("{WAKEFULNESS_DESCRIPTION} {WAKEFULNESS_TO_DORMANCY_DESCRIPTION}"),
                ),
                ("spending_rate", SPENDING_RATE_DESCRIPTION.to_owned()),
                (
                    "spending_exposure",
                    SPENDING_EXPOSURE_DESCRIPTION.to_owned(),
                ),
                (
                    "mobility",
                    format!("{SPENDING_EXPOSURE_DESCRIPTION} {MOBILITY_DESCRIPTION}"),
                ),
            ] {
                assert_framework_description(&format!("{prefix}_{suffix}"), &expected);
            }
        }

        for (name, expected) in [
            ("utxos_age_range_wakefulness", WAKEFULNESS_DESCRIPTION),
            (
                "utxos_age_range_awake_supply_sats",
                AGE_AWAKE_SUPPLY_DESCRIPTION,
            ),
            (
                "utxos_age_range_dormant_supply_sats",
                AGE_DORMANT_SUPPLY_DESCRIPTION,
            ),
            ("utxos_age_range_spending_rate", SPENDING_RATE_DESCRIPTION),
            (
                "utxos_age_range_spending_exposure",
                SPENDING_EXPOSURE_DESCRIPTION,
            ),
            (
                "utxos_age_range_mobile_supply_sats",
                MOBILE_SUPPLY_DESCRIPTION,
            ),
            (
                "utxos_age_range_immobile_supply_sats",
                IMMOBILE_SUPPLY_DESCRIPTION,
            ),
            ("coinblocks_created", COINBLOCKS_CREATED_DESCRIPTION),
            ("coinblocks_stored", COINBLOCKS_STORED_DESCRIPTION),
            ("liveliness", LIVELINESS_DESCRIPTION),
            ("vaultedness", VAULTEDNESS_DESCRIPTION),
            (
                "activity_to_vaultedness",
                ACTIVITY_TO_VAULTEDNESS_DESCRIPTION,
            ),
            (
                "cointime_adj_tx_velocity_btc",
                COINTIME_ADJ_NATIVE_VELOCITY_DESCRIPTION,
            ),
            (
                "cointime_adj_tx_velocity_usd",
                COINTIME_ADJ_FIAT_VELOCITY_DESCRIPTION,
            ),
            (
                "cointime_awake_supply_sats_by_term",
                AGGREGATE_AWAKE_SUPPLY_DESCRIPTION,
            ),
            (
                "cointime_dormant_supply_sats_by_term",
                AGGREGATE_DORMANT_SUPPLY_DESCRIPTION,
            ),
            (
                "cointime_awake_cap_cents_by_term",
                AGGREGATE_AWAKE_CAP_DESCRIPTION,
            ),
            (
                "cointime_awake_price_cents_by_aggregate",
                AGGREGATE_AWAKE_PRICE_DESCRIPTION,
            ),
            (
                "cointime_awake_supply_in_loss_share_by_term",
                COINTIME_AWAKE_LOSS_DESCRIPTION,
            ),
            (
                "all_awake_supply_in_loss_share",
                COINTIME_AWAKE_LOSS_DESCRIPTION,
            ),
            (
                "sth_awake_supply_in_loss_share",
                COINTIME_AWAKE_LOSS_DESCRIPTION,
            ),
            (
                "lth_awake_supply_in_loss_share",
                COINTIME_AWAKE_LOSS_DESCRIPTION,
            ),
            (
                "cointime_supply_in_loss_share",
                COINTIME_AWAKE_LOSS_DESCRIPTION,
            ),
            (
                "cointime_value_destroyed",
                COINTIME_VALUE_DESTROYED_DESCRIPTION,
            ),
            ("cointime_value_created", COINTIME_VALUE_CREATED_DESCRIPTION),
            ("cointime_value_stored", COINTIME_VALUE_STORED_DESCRIPTION),
            ("vocdd", VOCDD_DESCRIPTION),
            ("reserve_risk", RESERVE_RISK_DESCRIPTION),
            ("vocdd_median_1y", VOCDD_MEDIAN_1Y_DESCRIPTION),
            ("hodl_bank", HODL_BANK_DESCRIPTION),
        ] {
            assert_framework_description(name, expected);
        }

        for aggregate in ["all", "sth", "lth"] {
            assert_framework_description(
                &format!("{aggregate}_coinflow_supply_in_loss_share"),
                COINFLOW_LOSS_DESCRIPTION,
            );
            for (horizon, horizon_description) in ["8y", "4y", "2y", "1y", "6m", "3m", "1m"]
                .into_iter()
                .zip(HORIZON_DESCRIPTIONS)
            {
                let expected = format!("{COINFLOW_HORIZON_LOSS_DESCRIPTION} {horizon_description}");
                assert_framework_description(
                    &format!("{aggregate}_coinflow_{horizon}_supply_in_loss_share"),
                    &expected,
                );
            }
        }

        for (name, expected) in [
            (
                "coinflow_mobile_supply_sats_by_term",
                MOBILE_SUPPLY_DESCRIPTION,
            ),
            (
                "coinflow_immobile_supply_sats_by_term",
                IMMOBILE_SUPPLY_DESCRIPTION,
            ),
            (
                "coinflow_supply_in_loss_share_by_aggregate",
                COINFLOW_LOSS_DESCRIPTION,
            ),
            ("coinflow_cap_cents_by_term", COINFLOW_CAP_DESCRIPTION),
            (
                "coinflow_price_cents_by_aggregate",
                COINFLOW_PRICE_DESCRIPTION,
            ),
        ] {
            assert_framework_description(name, expected);
        }
        for (horizon, horizon_description) in ["8y", "4y", "2y", "1y", "6m", "3m", "1m"]
            .into_iter()
            .zip(HORIZON_DESCRIPTIONS)
        {
            let expected = format!("{COINFLOW_HORIZON_LOSS_DESCRIPTION} {horizon_description}");
            assert_framework_description(
                &format!("coinflow_{horizon}_supply_in_loss_share_by_aggregate"),
                &expected,
            );
        }
        assert_eq!(audited_framework_roots, 204);

        let documented = vecs
            .series_to_index_to_vec
            .iter()
            .filter_map(|(name, vecs)| vecs.description().map(|description| (*name, description)))
            .collect::<Vec<_>>();
        assert!(!documented.is_empty());
        let unexpected = documented
            .iter()
            .filter(|(_, description)| !is_audited_description(description))
            .collect::<Vec<_>>();
        assert!(
            unexpected.is_empty(),
            "unexpected documented series: {unexpected:#?}"
        );
    }

    fn is_audited_description(mut description: &str) -> bool {
        let fragments = [
            REALIZED_PRICE_DESCRIPTION,
            CAPITAL_SENTIMENT_LONG_DESCRIPTION,
            CAPITAL_SENTIMENT_SHORT_DESCRIPTION,
            UTXO_COUNT_DESCRIPTION,
            TIMESTAMP_DESCRIPTION,
            COINDAYS_CREATED_DESCRIPTION,
            SIGHASH_ALL_DESCRIPTION,
            SIGHASH_NONE_DESCRIPTION,
            SIGHASH_SINGLE_DESCRIPTION,
            SIGHASH_DEFAULT_DESCRIPTION,
            SIGHASH_ANYONE_CAN_PAY_DESCRIPTION,
            COINJOIN_DESCRIPTION,
            CONSOLIDATION_DESCRIPTION,
            BATCH_PAYOUT_DESCRIPTION,
            SIGOP_COST_DESCRIPTION,
            DATE_DESCRIPTION,
            FIRST_HEIGHT_DESCRIPTION,
            MINUTE10_INDEX_DESCRIPTION,
            MINUTE30_INDEX_DESCRIPTION,
            HOUR1_INDEX_DESCRIPTION,
            HOUR4_INDEX_DESCRIPTION,
            HOUR12_INDEX_DESCRIPTION,
            DAY1_INDEX_DESCRIPTION,
            DAY3_INDEX_DESCRIPTION,
            EPOCH_INDEX_DESCRIPTION,
            HALVING_INDEX_DESCRIPTION,
            WEEK1_INDEX_DESCRIPTION,
            MONTH1_INDEX_DESCRIPTION,
            MONTH3_INDEX_DESCRIPTION,
            MONTH6_INDEX_DESCRIPTION,
            YEAR1_INDEX_DESCRIPTION,
            YEAR10_INDEX_DESCRIPTION,
            TX_INDEX_DESCRIPTION,
            TXIN_INDEX_DESCRIPTION,
            TXOUT_INDEX_DESCRIPTION,
            INPUT_COUNT_DESCRIPTION,
            OUTPUT_COUNT_DESCRIPTION,
            TX_INDEX_COUNT_DESCRIPTION,
            MONOTONIC_TIMESTAMP_DESCRIPTION,
            P2PK33_DESCRIPTION,
            P2PK65_DESCRIPTION,
            P2PKH_DESCRIPTION,
            P2SH_DESCRIPTION,
            P2TR_DESCRIPTION,
            P2WPKH_DESCRIPTION,
            P2WSH_DESCRIPTION,
            P2A_DESCRIPTION,
            P2MS_DESCRIPTION,
            EMPTY_OUTPUT_DESCRIPTION,
            UNKNOWN_OUTPUT_DESCRIPTION,
            OP_RETURN_OUTPUT_DESCRIPTION,
            TYPE_SPECIFIC_ADDR_INDEX_DESCRIPTION,
            ADDR_TEXT_DESCRIPTION,
            TYPE_SPECIFIC_OUTPUT_INDEX_DESCRIPTION,
            RAW_ADDR_FIRST_INDEX_DESCRIPTION,
            RAW_ADDR_BYTES_DESCRIPTION,
            BLOCK_DESCRIPTION,
            CUMULATIVE_DESCRIPTION,
            ROLLING_SUM_DESCRIPTION,
            ROLLING_AVERAGE_DESCRIPTION,
            WINDOW_DESCRIPTIONS[0],
            WINDOW_DESCRIPTIONS[1],
            WINDOW_DESCRIPTIONS[2],
            WINDOW_DESCRIPTIONS[3],
            USD_DESCRIPTION,
            CENTS_DESCRIPTION,
            SATS_DESCRIPTION,
            PPM_DESCRIPTION,
            RATIO_DESCRIPTION,
            AMOUNT_BTC_DESCRIPTION,
            AMOUNT_SATS_DESCRIPTION,
            AMOUNT_USD_DESCRIPTION,
            AMOUNT_CENTS_DESCRIPTION,
            GENERIC_PPM_DESCRIPTION,
            GENERIC_RATIO_DESCRIPTION,
            PERCENT_DESCRIPTION,
            TOTAL_SIZE_DESCRIPTION,
            BLOCK_SIZE_DESCRIPTION,
            BLOCK_VBYTES_DESCRIPTION,
            BLOCK_WEIGHT_DESCRIPTION,
            BLOCK_FULLNESS_DESCRIPTION,
            SEGWIT_TXS_DESCRIPTION,
            SEGWIT_SIZE_DESCRIPTION,
            SEGWIT_WEIGHT_DESCRIPTION,
            BLOCK_COUNT_TARGET_DESCRIPTION,
            BLOCK_COUNT_DESCRIPTION,
            BLOCK_INTERVAL_DESCRIPTION,
            DIFFICULTY_DESCRIPTION,
            DIFFICULTY_HASHRATE_DESCRIPTION,
            DIFFICULTY_ADJUSTMENT_DESCRIPTION,
            DIFFICULTY_EPOCH_DESCRIPTION,
            BLOCKS_TO_RETARGET_DESCRIPTION,
            DAYS_TO_RETARGET_DESCRIPTION,
            HALVING_EPOCH_DESCRIPTION,
            BLOCKS_TO_HALVING_DESCRIPTION,
            DAYS_TO_HALVING_DESCRIPTION,
            BLOCKHASH_DESCRIPTION,
            COINBASE_TAG_DESCRIPTION,
            LOOKBACK_HEIGHT_DESCRIPTION,
            COINBASE_REWARD_DESCRIPTION,
            DERIVED_SUBSIDY_DESCRIPTION,
            TRANSACTION_FEES_DESCRIPTION,
            OUTPUT_VOLUME_DESCRIPTION,
            UNCLAIMED_REWARDS_DESCRIPTION,
            FEE_DOMINANCE_DESCRIPTION,
            SUBSIDY_DOMINANCE_DESCRIPTION,
            FEE_TO_SUBSIDY_DESCRIPTION,
            HASH_RATE_DESCRIPTION,
            HASH_RATE_SMA_DESCRIPTION,
            HASH_RATE_ATH_DESCRIPTION,
            HASH_RATE_DRAWDOWN_DESCRIPTION,
            HASH_PRICE_DESCRIPTION,
            HASH_VALUE_DESCRIPTION,
            PER_THS_DESCRIPTION,
            THS_MIN_DESCRIPTION,
            PER_PHS_DESCRIPTION,
            PHS_MIN_DESCRIPTION,
            HASH_REBOUND_DESCRIPTION,
            TX_COUNT_DESCRIPTION,
            TX_VSIZE_DESCRIPTION,
            TX_WEIGHT_DESCRIPTION,
            TRANSFER_VOLUME_DESCRIPTION,
            TX_PER_SECOND_DESCRIPTION,
            TX_INPUT_VALUE_DESCRIPTION,
            TX_OUTPUT_VALUE_DESCRIPTION,
            TX_FEE_DESCRIPTION,
            TX_FEE_RATE_DESCRIPTION,
            TX_EFFECTIVE_FEE_RATE_DESCRIPTION,
            CPFP_PARENT_DESCRIPTION,
            CPFP_CHILD_DESCRIPTION,
            CPFP_PARENT_COUNT_DESCRIPTION,
            CPFP_CHILD_COUNT_DESCRIPTION,
            TX_VERSION_CATEGORY_DESCRIPTION,
            TX_COUNT_V1_DESCRIPTION,
            TX_COUNT_V2_DESCRIPTION,
            TX_COUNT_V3_DESCRIPTION,
            TX_COUNT_OTHER_VERSION_DESCRIPTION,
            TX_VERSION_COUNTS_DESCRIPTION,
            TX_VERSION_1_DESCRIPTION,
            TX_VERSION_2_DESCRIPTION,
            TX_VERSION_3_DESCRIPTION,
            TX_OTHER_VERSION_DESCRIPTION,
            IS_EXPLICITLY_RBF_DESCRIPTION,
            EXPLICITLY_RBF_COUNT_DESCRIPTION,
            TX_POLICY_DESCRIPTION,
            IS_NONSTANDARD_DESCRIPTION,
            NONSTANDARD_COUNT_DESCRIPTION,
            MIN_DESCRIPTION,
            MAX_DESCRIPTION,
            PCT10_DESCRIPTION,
            PCT25_DESCRIPTION,
            MEDIAN_DESCRIPTION,
            PCT75_DESCRIPTION,
            PCT90_DESCRIPTION,
            MACD_DESCRIPTION,
            MACD_EMA_FAST_DESCRIPTION,
            MACD_EMA_SLOW_DESCRIPTION,
            MACD_LINE_DESCRIPTION,
            MACD_SIGNAL_DESCRIPTION,
            MACD_HISTOGRAM_DESCRIPTION,
            CAPITAL_SENTIMENT_PHASE_DESCRIPTION,
            CAPITAL_SENTIMENT_SCORE_DESCRIPTION,
            BEDROCK_LOSS_THRESHOLD_DESCRIPTION,
            BEDROCK_PRICE_BANDS_DESCRIPTION,
            RARITY_COMPONENT_BANDS_DESCRIPTION,
            RARITY_COMPONENT_RATIOS_DESCRIPTION,
            RARITY_PRICES_DESCRIPTION,
            RARITY_INDEX_DESCRIPTION,
            RARITY_SCORE_DESCRIPTION,
            RARITY_EXTREME_THRESHOLDS_DESCRIPTION,
            RARITY_THRESHOLD_PCT0_1_DESCRIPTION,
            RARITY_THRESHOLD_PCT0_05_DESCRIPTION,
            RARITY_THRESHOLD_PCT0_025_DESCRIPTION,
            RARITY_EXTREME_RANK_DESCRIPTION,
        ];

        while !description.is_empty() {
            let Some(fragment) = fragments
                .iter()
                .copied()
                .chain(FRAMEWORK_SEMANTIC_DESCRIPTIONS.iter().copied())
                .chain(HORIZON_DESCRIPTIONS.iter().copied())
                .chain(
                    INDEXER_DIRECT_DESCRIPTIONS
                        .iter()
                        .map(|(_, description)| *description),
                )
                .chain(
                    SMALL_PLUGIN_DIRECT_DESCRIPTIONS
                        .iter()
                        .map(|(_, description)| *description),
                )
                .chain(
                    BEDROCK_MODE_DESCRIPTIONS
                        .iter()
                        .map(|(_, description)| *description),
                )
                .chain(
                    RARITY_INNER_DESCRIPTIONS
                        .iter()
                        .map(|(_, description)| *description),
                )
                .chain(
                    RARITY_EXTREME_DESCRIPTIONS
                        .iter()
                        .map(|(_, description)| *description),
                )
                .filter(|fragment| {
                    description == *fragment
                        || description
                            .strip_prefix(*fragment)
                            .is_some_and(|tail| tail.starts_with(' '))
                })
                .max_by_key(|fragment| fragment.len())
            else {
                return false;
            };

            if description == fragment {
                return true;
            }
            description = &description[fragment.len() + 1..];
        }

        true
    }
}
