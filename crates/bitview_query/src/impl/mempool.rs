use crate::Query;
use brk_error::Error;
use brk_mempool::{Mempool, RbfForTx, RbfNode};
use brk_types::{
    BlockTemplate, BlockTemplateDiff, CheckedSub, FeeRate, MempoolBlock, MempoolInfo,
    MempoolRecentTx, NextBlockHash, OutputType, RbfResponse, RbfTx, RecommendedFees,
    ReplacementNode, Sats, Timestamp, TxOut, TxOutIndex, Txid, TxidPrefix, TypeIndex, Vout,
};
use rustc_hash::FxHashMap;

const RECENT_REPLACEMENTS_LIMIT: usize = 25;

impl Query {
    fn require_mempool(&self) -> brk_error::Result<&Mempool> {
        self.mempool().ok_or(Error::MempoolNotAvailable)
    }

    pub fn mempool_info(&self) -> brk_error::Result<MempoolInfo> {
        Ok(self.require_mempool()?.info())
    }

    pub fn mempool_txids(&self) -> brk_error::Result<Vec<Txid>> {
        Ok(self.require_mempool()?.txids())
    }

    pub fn recommended_fees(&self) -> brk_error::Result<RecommendedFees> {
        self.require_mempool().map(|m| m.fees())
    }

    pub fn mempool_blocks(&self) -> brk_error::Result<Vec<MempoolBlock>> {
        let mempool = self.require_mempool()?;
        Ok(mempool
            .block_stats()
            .iter()
            .map(MempoolBlock::from)
            .collect())
    }

    /// Indexer-backed resolver for confirmed-parent prevouts. Boxed so
    /// the caller (typically [`Mempool::start_with`]) can stash one
    /// resolver behind a stable type for the lifetime of the loop.
    #[allow(clippy::type_complexity)]
    pub fn indexer_prevout_resolver(
        &self,
    ) -> Box<dyn Fn(&[(Txid, Vout)]) -> FxHashMap<(Txid, Vout), TxOut> + Send + Sync> {
        let indexer = self.0.plugins.indexer;

        Box::new(move |holes: &[(Txid, Vout)]| {
            if holes.is_empty() {
                return FxHashMap::default();
            }
            let safe = indexer.safe_lengths();
            let first_txout_reader = indexer.vecs().transactions.first_txout_index.reader();
            let output_type_reader = indexer.vecs().outputs.output_type.reader();
            let type_index_reader = indexer.vecs().outputs.type_index.reader();
            let value_reader = indexer.vecs().outputs.value.reader();
            let addr_readers = indexer.vecs().addrs.addr_readers();
            holes
                .iter()
                .filter_map(|(prev_txid, vout)| {
                    let prev_tx_index = indexer
                        .stores()
                        .tx_index(&TxidPrefix::from(prev_txid))
                        .ok()??;
                    if prev_tx_index >= safe.tx_index {
                        return None;
                    }
                    let first_txout: TxOutIndex = first_txout_reader.try_get(prev_tx_index)?;
                    let txout = first_txout + *vout;
                    if txout >= safe.txout_index {
                        return None;
                    }
                    let output_type: OutputType = output_type_reader.try_get(txout)?;
                    let type_index: TypeIndex = type_index_reader.try_get(txout)?;
                    let value: Sats = value_reader.try_get(txout)?;
                    let script_pubkey = addr_readers.script_pubkey(output_type, type_index);
                    Some(((*prev_txid, *vout), TxOut::from((script_pubkey, value))))
                })
                .collect()
        })
    }

    pub fn mempool_recent(&self) -> brk_error::Result<Vec<MempoolRecentTx>> {
        Ok(self.require_mempool()?.recent_txs())
    }

    /// RBF history for a tx. Matches mempool.space's
    /// `GET /api/v1/tx/:txid/rbf`. Mempool builds the owned tree under
    /// one read-lock window; this then layers on `mined` + effective
    /// fee rate from the indexer/plugins.
    pub fn tx_rbf(&self, txid: &Txid) -> brk_error::Result<RbfResponse> {
        let RbfForTx { root, replaces } = self.require_mempool()?.rbf_for_tx(txid);
        let replacements = root.map(|n| self.enrich_rbf_node(n, None));
        let replaces = (!replaces.is_empty()).then_some(replaces);
        Ok(RbfResponse {
            replacements,
            replaces,
        })
    }

    /// Recent RBF replacements. Matches mempool.space's
    /// `GET /api/v1/replacements` and `GET /api/v1/fullrbf/replacements`.
    /// Most-recent first, capped at 25. `full_rbf_only` keeps only
    /// trees with at least one non-signaling predecessor.
    pub fn recent_replacements(
        &self,
        full_rbf_only: bool,
    ) -> brk_error::Result<Vec<ReplacementNode>> {
        Ok(self
            .require_mempool()?
            .recent_rbf_trees(full_rbf_only, RECENT_REPLACEMENTS_LIMIT)
            .into_iter()
            .map(|n| self.enrich_rbf_node(n, None))
            .collect())
    }

    /// Layer `mined` + effective fee rate onto an `RbfNode` tree.
    /// Must run after the mempool lock has dropped (effective_fee_rate
    /// re-enters Mempool).
    fn enrich_rbf_node(&self, node: RbfNode, successor_time: Option<Timestamp>) -> ReplacementNode {
        let interval = successor_time
            .and_then(|st| st.checked_sub(node.first_seen))
            .map(|d| *d);
        let mined = super::tx::resolve_tx_index(self, &node.txid)
            .is_ok()
            .then_some(true);
        let rate = self
            .effective_fee_rate(&node.txid)
            .unwrap_or_else(|_| FeeRate::from((node.fee, node.vsize)));
        let first_seen = node.first_seen;
        let replaces = node
            .replaces
            .into_iter()
            .map(|child| self.enrich_rbf_node(child, Some(first_seen)))
            .collect();
        ReplacementNode {
            tx: RbfTx {
                txid: node.txid,
                fee: node.fee,
                vsize: node.vsize,
                value: node.value,
                rate,
                time: first_seen,
                rbf: node.rbf,
                full_rbf: Some(node.full_rbf),
            },
            time: first_seen,
            full_rbf: node.full_rbf,
            interval,
            mined,
            replaces,
        }
    }

    /// `first_seen` Unix-second timestamps. Matches mempool.space's
    /// `POST /api/v1/transaction-times`. Returns 0 for unknowns.
    pub fn transaction_times(&self, txids: &[Txid]) -> brk_error::Result<Vec<u64>> {
        Ok(self.require_mempool()?.transaction_times(txids))
    }

    /// Content hash of the projected next block. Same value as the
    /// mempool ETag. Polling lets monitors detect a stalled sync.
    pub fn mempool_hash(&self) -> brk_error::Result<NextBlockHash> {
        Ok(self.require_mempool()?.next_block_hash())
    }

    /// Full projected next block (Core's `getblocktemplate` selection)
    /// with stats and full tx bodies in GBT order.
    pub fn block_template(&self) -> brk_error::Result<BlockTemplate> {
        Ok(self.require_mempool()?.block_template())
    }

    /// Delta of the projected next block since `since`. `NotFound`
    /// when `since` has aged out (client should fall back to
    /// `block_template`).
    pub fn block_template_diff(
        &self,
        since: NextBlockHash,
    ) -> brk_error::Result<BlockTemplateDiff> {
        self.require_mempool()?
            .block_template_diff(since)
            .ok_or_else(|| Error::NotFound(format!("unknown since hash: {since}")))
    }
}
