use std::{thread::sleep, time::Duration};

use bitcoin::{
    BlockHash as BitcoinBlockHash, Network, ScriptBuf, Transaction, Txid as BitcoinTxid,
    consensus::encode,
};
use brk_error::{Error, Result};
use brk_types::{Bitcoin, BlockHash, FeeRate, Height, MempoolEntryInfo, Sats, TxOut, Txid, Vout};
use corepc_jsonrpc::error::Error as JsonRpcError;
use rapidhash::RapidHashMap;
use serde_json::{from_str, json, value::RawValue};
use tracing::{debug, info, warn};

use super::{
    BlockSpends, Client, ClientInner,
    mempool_entry::MempoolEntry,
    mempool_state::MempoolState,
    responses::{
        GetBlockCount, GetBlockHeaderVerbose, GetBlockTemplate, GetBlockchainInfo, GetChainTip,
        GetMempoolInfo,
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
/// `fetch_new_pool_data`, and `get_tx_outs`. Sized so the JSON
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

    /// The tip of every branch the node knows, the best chain's included.
    pub fn get_chain_tips(&self) -> Result<Vec<GetChainTip>> {
        self.0.call_with_retry("getchaintips", &NO_ARGS)
    }

    /// The canonical block hashes of the inclusive height range `start..=end`, in height order,
    /// in sequential batches of `BATCH_CHUNK` (2,000) requests, each retried as single calls are
    /// (lost connections, the node warming up).
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

    /// Core's projected next block (`getblocktemplate`), the live mempool txids
    /// (`getrawmempool false`), `mempoolminfee` (`getmempoolinfo`) and the operator's fee deltas
    /// (`getprioritisedtransactions`), in one batched round trip. GBT carries each tx's full body and stats, so block 0
    /// is exact even when a tx vanishes from the mempool listing between the GBT
    /// and `getrawmempool` calls; no follow-up entry fetch can race it. No template when Core
    /// refuses one (in initial sync, or with no peers): the tip is then `getbestblockhash`'s.
    pub fn fetch_mempool_state(&self) -> Result<(MempoolState, Option<Vec<BlockTemplateTx>>)> {
        let template_args = (json!({ "rules": ["segwit"] }),);
        let calls = [
            RpcCall::new("getbestblockhash", &NO_ARGS)?,
            RpcCall::new("getblocktemplate", &template_args)?,
            // Before the listing (Core runs a batch in order): a load reported finished was
            // finished when the listing was taken.
            RpcCall::new("getmempoolinfo", &NO_ARGS)?,
            RpcCall::new("getrawmempool", &(false,))?,
            RpcCall::new("getprioritisedtransactions", &NO_ARGS)?,
        ];
        // The batch answers every request, a missing response as an item error.
        let [tip_raw, template_raw, info_raw, txids_raw, deltas_raw] =
            <[_; 5]>::try_from(self.0.call_mixed_batch(&calls)?).expect("one result per call");
        let (tip_raw, txids_raw, info_raw) = (tip_raw?, txids_raw?, info_raw?);

        let live_txids = TxidArrayParser::parse(txids_raw.get())?;
        // A template that doesn't read, or breaks a block's limits, is none: block 0 packs from
        // the pool, and the cycle goes on.
        let template = match template_raw {
            Ok(raw) => {
                let read = || -> Result<_> {
                    let template: GetBlockTemplate = from_str(raw.get())?;
                    Ok((
                        Self::parse_block_hash(&template.previous_block_hash, "previousblockhash")?,
                        ClientInner::build_gbt(template.transactions)?,
                    ))
                };
                read()
                    .inspect_err(|error| warn!(%error, "getblocktemplate: unreadable, set aside"))
                    .ok()
            }
            Err(error) => {
                debug!(%error, "getblocktemplate: none");
                None
            }
        };
        let (tip_hash, block_template) = match template {
            Some((tip, template)) => (tip, Some(template)),
            None => {
                let tip: String = from_str(tip_raw.get())?;
                (Self::parse_block_hash(&tip, "getbestblockhash")?, None)
            }
        };
        let info: GetMempoolInfo = from_str(info_raw.get())?;
        let min_fee = Self::build_min_fee(info.mempool_min_fee)?;
        let fee_deltas = Self::build_fee_deltas(deltas_raw);

        Ok((
            MempoolState {
                live_txids,
                min_fee,
                tip_hash,
                fee_deltas,
                loaded: info.loaded,
            },
            block_template,
        ))
    }

    /// The nonzero deltas of transactions in the mempool, in txid byte order. A node without the
    /// call (before Core 27) has none to report: an error reads as none.
    fn build_fee_deltas(raw: Result<Box<RawValue>>) -> Box<[(Txid, i64)]> {
        #[derive(serde::Deserialize)]
        struct Prioritised {
            fee_delta: i64,
            in_mempool: bool,
        }
        let parsed =
            raw.and_then(|raw| Ok(from_str::<RapidHashMap<Txid, Prioritised>>(raw.get())?));
        let mut deltas: Vec<(Txid, i64)> = match parsed {
            Ok(map) => map
                .into_iter()
                .filter(|(_, p)| p.in_mempool && p.fee_delta != 0)
                .map(|(txid, p)| (txid, p.fee_delta))
                .collect(),
            Err(error) => {
                debug!(%error, "getprioritisedtransactions: no fee deltas");
                Vec::new()
            }
        };
        deltas.sort_unstable_by_key(|(txid, _)| **txid);
        deltas.into()
    }

    /// The outputs at `outpoints` from the node's UTXO set (`gettxout`, the mempool's spends not
    /// counted): what a pending transaction's confirmed parents left it, with no `-txindex`
    /// needed. One the set doesn't hold (unknown, or spent in a block) is left out, and so is a
    /// batch that fails: the others' answers stand.
    pub fn get_tx_outs(&self, outpoints: &[(Txid, Vout)]) -> RapidHashMap<(Txid, Vout), TxOut> {
        #[derive(serde::Deserialize)]
        struct Unspent {
            value: Bitcoin,
            #[serde(rename = "scriptPubKey")]
            script: Script,
        }
        #[derive(serde::Deserialize)]
        struct Script {
            hex: String,
        }
        let mut out = RapidHashMap::with_capacity_and_hasher(outpoints.len(), Default::default());
        for chunk in outpoints.chunks(BATCH_CHUNK) {
            let args = chunk.iter().map(|(txid, vout)| {
                let txid: &BitcoinTxid = txid.into();
                (txid, u32::from(*vout), false)
            });
            let results = match self.0.call_batch_raw("gettxout", args) {
                Ok(results) => results,
                Err(error) => {
                    warn!(%error, outpoints = chunk.len(), "gettxout batch failed");
                    continue;
                }
            };
            for (outpoint, result) in chunk.iter().zip(results) {
                let unspent = result.and_then(|raw| Ok(from_str::<Option<Unspent>>(raw.get())?));
                match unspent {
                    Ok(Some(unspent)) => match ScriptBuf::from_hex(&unspent.script.hex) {
                        Ok(script) => {
                            out.insert(*outpoint, TxOut::from((script, Sats::from(unspent.value))));
                        }
                        Err(error) => debug!(%error, "gettxout: an unreadable script"),
                    },
                    Ok(None) => {}
                    Err(error) => debug!(%error, "gettxout batch: item failed"),
                }
            }
        }
        out
    }

    /// Block `hash`'s transactions and what their inputs spend (`getblock` raw, decoded once).
    pub fn get_block_spends(&self, hash: &BlockHash) -> Result<BlockSpends> {
        let bitcoin_hash: &BitcoinBlockHash = hash.into();
        let hex: String = self.0.call_with_retry("getblock", &(bitcoin_hash, 0))?;
        let block: bitcoin::Block = encode::deserialize_hex(&hex)?;
        let mut spends = Vec::new();
        let txids = block
            .txdata
            .iter()
            .map(|tx| {
                let txid = Txid::from(tx.compute_txid());
                if !tx.is_coinbase() {
                    spends.extend(tx.input.iter().map(|input| {
                        let outpoint = &input.previous_output;
                        ((Txid::from(outpoint.txid), Vout::from(outpoint.vout)), txid)
                    }));
                }
                txid
            })
            .collect();
        Ok(BlockSpends {
            prev: BlockHash::from(block.header.prev_blockhash),
            txids,
            spends,
        })
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
    ) -> Result<(Vec<MempoolEntryInfo>, RapidHashMap<Txid, Transaction>)> {
        let requests: Vec<(Txid, bool)> = entries_only
            .iter()
            .map(|txid| (*txid, false))
            .chain(full.iter().map(|txid| (*txid, true)))
            .collect();
        let mut entries: Vec<MempoolEntryInfo> = Vec::with_capacity(requests.len());
        let mut txs: RapidHashMap<Txid, Transaction> =
            RapidHashMap::with_capacity_and_hasher(full.len(), Default::default());

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
        self.wait_until(|info| Ok(info.headers == info.blocks))
            .map(|_| ())
    }

    /// Waits until the node has validated every header it knows and either its chain reaches
    /// `height` or it left `ours` (our block at a height), returning the lowest height it reported
    /// meanwhile. A node below `height` with no header left to validate is behind (back from a
    /// crash, it comes back below the blocks it lost; until its peers return them it knows of no
    /// header left to validate) while our block at its height is in its best chain; once it isn't,
    /// its chain left ours, shorter (heavier): a reorg to find, not a node to wait for. Free while the
    /// node is at `height` or above. Left as is: a node back from a crash on the branch it had just
    /// left (ours lost with it) is followed there, then back once it recovers ours: the index always
    /// matches the node, the blocks between reindexed twice.
    pub fn wait_for_node_at(
        &self,
        height: Height,
        ours: impl Fn(Height) -> BlockHash,
    ) -> Result<Height> {
        self.wait_until(|info| {
            if info.headers != info.blocks {
                return Ok(false);
            }
            if info.blocks >= i64::from(*height) {
                return Ok(true);
            }
            let tip = Height::from(info.blocks as u64);
            Ok(!self.is_in_best_chain(&ours(tip))?)
        })
    }

    /// Polls the node every second until `done`, returning the lowest height it reported.
    fn wait_until(
        &self,
        mut done: impl FnMut(&GetBlockchainInfo) -> Result<bool>,
    ) -> Result<Height> {
        let mut info = self.get_blockchain_info()?;
        let mut lowest = info.blocks;
        if !done(&info)? {
            info!("Waiting for node to sync...");
            loop {
                sleep(Duration::from_secs(1));
                info = self.get_blockchain_info()?;
                lowest = lowest.min(info.blocks);
                if done(&info)? {
                    break;
                }
            }
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
