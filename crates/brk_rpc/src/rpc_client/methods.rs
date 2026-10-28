use std::{thread::sleep, time::Duration};

use bitcoin::{
    BlockHash as BitcoinBlockHash, Network, Transaction, Txid as BitcoinTxid, consensus::encode,
};
use brk_error::{Error, Result};
use brk_types::{BlockHash, FeeRate, Height, MempoolEntryInfo, Txid};
use corepc_jsonrpc::error::Error as JsonRpcError;
use rustc_hash::FxHashMap;
use serde_json::{from_str, json, value::RawValue};
use tracing::{debug, info};

use super::{
    Client, ClientInner,
    mempool_entry::MempoolEntry,
    mempool_state::MempoolState,
    responses::{
        GetBlockCount, GetBlockHeaderVerbose, GetBlockTemplate, GetBlockchainInfo, GetMempoolInfo,
    },
    rpc_call::RpcCall,
    txid_array_parser::TxidArrayParser,
};
use crate::BlockTemplateTx;

/// Bitcoin Core's `-5` (`RPC_INVALID_ADDRESS_OR_KEY`): a block hash the node
/// doesn't know, or a confirmed transaction queried without `-txindex`. The
/// mempool fetcher tolerates these per-item failures silently.
const RPC_NOT_FOUND: i32 = -5;
const NO_ARGS: [(); 0] = [];

/// Per-batch request count for `get_block_hashes_range`,
/// `fetch_new_pool_data`, and `get_raw_transactions`. Sized so the JSON
/// request body stays well under a megabyte and bitcoind doesn't spend
/// too long on a single batch before yielding results. For the mixed
/// `getmempoolentry`+`getrawtransaction` batch this is the *txid* count:
/// one request per `entries_only` txid, two per `full` one.
const BATCH_CHUNK: usize = 2000;

impl Client {
    /// Full current best-chain identity for bracketing related observations.
    pub fn get_best_block_hash(&self) -> Result<BlockHash> {
        let hash: BitcoinBlockHash = self.0.call_with_retry("getbestblockhash", &NO_ARGS)?;
        Ok(hash.into())
    }

    /// Convert bitcoind's `mempoolminfee` (BTC/kvB f64) to sat/vB. Round-trip
    /// via integer sat/kvB (bitcoind's native CFeeRate unit) so JSON f64 drift
    /// cannot move an exact sat/vB boundary upward.
    fn build_min_fee(btc_per_kvb: f64) -> Result<FeeRate> {
        let sat_per_kvb = (btc_per_kvb * 100_000_000.0).round();
        if !btc_per_kvb.is_finite() || btc_per_kvb < 0.0 || sat_per_kvb >= u64::MAX as f64 {
            return Err(Error::Parse(format!(
                "mempool fee floor out of range: {btc_per_kvb}"
            )));
        }
        Ok(FeeRate::from_milli(sat_per_kvb as u64))
    }

    /// The best chain's tip height.
    pub fn get_last_height(&self) -> Result<Height> {
        let r: GetBlockCount = self.0.call_with_retry("getblockcount", &NO_ARGS)?;
        Ok(Height::from(r.0))
    }
    pub fn get_block_header_info<'a, H>(&self, hash: &'a H) -> Result<GetBlockHeaderVerbose>
    where
        &'a H: Into<&'a BitcoinBlockHash>,
    {
        let hash: &BitcoinBlockHash = hash.into();
        self.0.call_with_retry("getblockheader", &(hash,))
    }
    /// The canonical block hashes of the inclusive height range `start..=end`, in height order,
    /// in sequential batches of `BATCH_CHUNK` (2,000) requests. A failed batch returns its error
    /// at once: batches are not retried like single calls.
    pub fn get_block_hashes_range<H1, H2>(&self, start: H1, end: H2) -> Result<Vec<BlockHash>>
    where
        H1: Into<u64>,
        H2: Into<u64>,
    {
        let start: u64 = start.into();
        let end: u64 = end.into();
        if end < start {
            return Ok(Vec::new());
        }
        let total = (end - start + 1) as usize;
        let mut hashes = Vec::with_capacity(total);

        let mut chunk_start = start;
        while chunk_start <= end {
            let chunk_end = (chunk_start + BATCH_CHUNK as u64 - 1).min(end);
            let args = (chunk_start..=chunk_end).map(|height| (height,));
            let chunk: Vec<BitcoinBlockHash> = self.0.call_batch("getblockhash", args)?;
            hashes.extend(chunk.into_iter().map(BlockHash::from));
            chunk_start = chunk_end + 1;
        }
        Ok(hashes)
    }

    /// Batched `getrawtransaction`, chunked at `BATCH_CHUNK` requests, keyed by txid. A failed
    /// item is dropped so one bad entry doesn't kill the batch: NOT_FOUND (a tx evicted since the
    /// listing) silently, others with a debug log. Transport failures propagate.
    pub fn get_raw_transactions(&self, txids: &[Txid]) -> Result<FxHashMap<Txid, Transaction>> {
        let mut out: FxHashMap<Txid, Transaction> =
            FxHashMap::with_capacity_and_hasher(txids.len(), Default::default());

        for chunk in txids.chunks(BATCH_CHUNK) {
            let args = chunk.iter().map(|t| {
                let txid: &BitcoinTxid = t.into();
                (txid, false)
            });
            let results = self.0.call_batch_raw("getrawtransaction", args)?;

            for (txid, res) in chunk.iter().zip(results) {
                match res.and_then(|raw| Self::decode_raw_tx(&raw)) {
                    Ok(tx) => {
                        out.insert(*txid, tx);
                    }
                    Err(Error::CorepcRPC(JsonRpcError::Rpc(rpc))) if rpc.code == RPC_NOT_FOUND => {}
                    Err(e) => {
                        debug!(txid = %txid, error = %e, "getrawtransaction batch: item failed")
                    }
                }
            }
        }

        Ok(out)
    }

    /// Core's projected next block (`getblocktemplate`), the live mempool txids
    /// (`getrawmempool false`) and `mempoolminfee` (`getmempoolinfo`), in one
    /// batched round trip. GBT carries each tx's full body and stats, so block 0
    /// is exact even when a tx vanishes from the mempool listing between the GBT
    /// and `getrawmempool` calls; no follow-up entry fetch can race it.
    pub fn fetch_mempool_state(&self) -> Result<(MempoolState, Vec<BlockTemplateTx>)> {
        let template_args = (json!({ "rules": ["segwit"] }),);
        let calls = [
            RpcCall::new("getblocktemplate", &template_args)?,
            RpcCall::new("getrawmempool", &(false,))?,
            RpcCall::new("getmempoolinfo", &NO_ARGS)?,
        ];
        // The batch answers every request, a missing response as an item error.
        let [template_raw, txids_raw, info_raw] =
            <[_; 3]>::try_from(self.0.call_mixed_batch(&calls)?).expect("one result per call");
        let (template_raw, txids_raw, info_raw) = (template_raw?, txids_raw?, info_raw?);

        let live_txids = TxidArrayParser::parse(txids_raw.get())?;
        let template: GetBlockTemplate = from_str(template_raw.get())?;
        let tip_hash = Self::parse_block_hash(&template.previous_block_hash, "previousblockhash")?;
        let block_template = ClientInner::build_gbt(template.transactions)?;
        let info: GetMempoolInfo = from_str(info_raw.get())?;
        let min_fee = Self::build_min_fee(info.mempool_min_fee)?;

        Ok((
            MempoolState {
                live_txids,
                min_fee,
                tip_hash,
            },
            block_template,
        ))
    }

    /// `getmempoolentry` for every txid of `entries_only` and `full`, plus `getrawtransaction`
    /// for those of `full`, in mixed batches. Returns the entries and the raw-tx map keyed by
    /// txid. Per-item -5 (NOT_FOUND — tx evicted between the listing and this call) drops
    /// silently for either leg; transport-level failures still propagate. Chunked at
    /// `BATCH_CHUNK` txids per round-trip.
    pub fn fetch_new_pool_data(
        &self,
        entries_only: &[Txid],
        full: &[Txid],
    ) -> Result<(Vec<MempoolEntryInfo>, FxHashMap<Txid, Transaction>)> {
        let requests: Vec<(Txid, bool)> = entries_only
            .iter()
            .map(|txid| (*txid, false))
            .chain(full.iter().map(|txid| (*txid, true)))
            .collect();
        let mut entries: Vec<MempoolEntryInfo> = Vec::with_capacity(requests.len());
        let mut txs: FxHashMap<Txid, Transaction> =
            FxHashMap::with_capacity_and_hasher(full.len(), Default::default());

        for chunk in requests.chunks(BATCH_CHUNK) {
            let mut calls = Vec::with_capacity(chunk.len() * 2);
            for (txid, body) in chunk {
                let txid: &BitcoinTxid = txid.into();
                let txid = txid.to_string();
                calls.push(RpcCall::new("getmempoolentry", &(&txid,))?);
                if *body {
                    calls.push(RpcCall::new("getrawtransaction", &(&txid, false))?);
                }
            }

            // The batch answers every request, a missing response as an item error.
            let mut results = self.0.call_mixed_batch(&calls)?.into_iter();
            for (txid, body) in chunk {
                let entry_res = results.next().expect("one result per call");
                match entry_res.and_then(|raw| {
                    let entry: MempoolEntry = from_str(raw.get())?;
                    Ok(entry.into_info(*txid))
                }) {
                    Ok(info) => entries.push(info),
                    Err(Error::CorepcRPC(JsonRpcError::Rpc(rpc))) if rpc.code == RPC_NOT_FOUND => {}
                    Err(e) => {
                        debug!(txid = %txid, error = %e, "getmempoolentry mixed batch: item failed")
                    }
                }
                if !body {
                    continue;
                }
                let raw_res = results.next().expect("one result per call");
                match raw_res.and_then(|raw| Self::decode_raw_tx(&raw)) {
                    Ok(tx) => {
                        txs.insert(*txid, tx);
                    }
                    Err(Error::CorepcRPC(JsonRpcError::Rpc(rpc))) if rpc.code == RPC_NOT_FOUND => {}
                    Err(e) => {
                        debug!(txid = %txid, error = %e, "getrawtransaction mixed batch: item failed")
                    }
                }
            }
        }

        Ok((entries, txs))
    }

    /// Whether `hash` is on the node's best chain: false off it or unknown.
    pub fn is_in_best_chain(&self, hash: &BlockHash) -> Result<bool> {
        match self.get_block_header_info(hash) {
            Ok(info) => Ok(info.confirmations > 0),
            Err(Error::CorepcRPC(JsonRpcError::Rpc(rpc))) if rpc.code == RPC_NOT_FOUND => Ok(false),
            Err(e) => Err(e),
        }
    }

    fn get_blockchain_info(&self) -> Result<GetBlockchainInfo> {
        self.0.call_with_retry("getblockchaininfo", &NO_ARGS)
    }

    /// Bitcoin network the connected node is running on, derived from
    /// `getblockchaininfo.chain`.
    pub fn get_network(&self) -> Result<Network> {
        let chain = self.get_blockchain_info()?.chain;
        Network::from_core_arg(&chain)
            .map_err(|e| Error::Parse(format!("getblockchaininfo.chain '{chain}': {e}")))
    }

    pub fn wait_for_synced_node(&self) -> Result<()> {
        self.wait_for_node_at(Height::ZERO).map(|_| ())
    }

    /// Waits until the node has validated every header it knows and its chain reaches `height`,
    /// returning the lowest height it reported meanwhile: a node back from a crash comes back
    /// below the blocks it lost. `headers == blocks` alone passes such a node: until its peers
    /// return them, it knows of no header left to validate.
    pub fn wait_for_node_at(&self, height: Height) -> Result<Height> {
        let is_synced = |info: &GetBlockchainInfo| {
            info.headers == info.blocks && info.blocks >= i64::from(*height)
        };

        let mut info = self.get_blockchain_info()?;
        let mut lowest = info.blocks;
        if !is_synced(&info) {
            info!("Waiting for node to sync...");
        }
        while !is_synced(&info) {
            sleep(Duration::from_secs(1));
            info = self.get_blockchain_info()?;
            lowest = lowest.min(info.blocks);
        }

        Ok(Height::from(lowest as u64))
    }

    /// A `getrawtransaction` (non-verbose) result: the JSON string of the tx hex.
    fn decode_raw_tx(raw: &RawValue) -> Result<Transaction> {
        let hex: &str = from_str(raw.get())?;
        Ok(encode::deserialize_hex(hex)?)
    }

    fn parse_block_hash(s: &str, label: &str) -> Result<BlockHash> {
        s.parse::<BitcoinBlockHash>()
            .map(BlockHash::from)
            .map_err(|e| Error::Parse(format!("{label}: {e}")))
    }
}
