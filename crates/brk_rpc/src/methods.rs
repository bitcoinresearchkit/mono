use std::{thread::sleep, time::Duration};

use bitcoin::{consensus::encode, hex::FromHex};
use brk_error::{Error, Result};
use brk_types::{
    Bitcoin, BlockHash, FeeRate, Height, MempoolEntryInfo, Sats, Timestamp, Txid, VSize, Vout,
    Weight,
};
use corepc_jsonrpc::error::Error as JsonRpcError;
use corepc_types::{
    v17::{
        BlockTemplateTransaction, GetBlockCount, GetBlockHash, GetBlockHeader,
        GetBlockHeaderVerbose, GetBlockTemplate, GetBlockVerboseOne, GetBlockVerboseZero,
        GetRawMempool, GetTxOut,
    },
    v24::{GetMempoolInfo, MempoolEntry},
    v28::GetBlockchainInfo,
};
use rustc_hash::FxHashMap;
use serde_json::Value;
use tracing::{debug, info};

/// Bitcoin Core's `-5` (`RPC_INVALID_ADDRESS_OR_KEY`) is the expected
/// response when querying a confirmed transaction without `-txindex`.
/// The mempool fetcher tolerates these per-item failures silently.
const RPC_NOT_FOUND: i32 = -5;

use crate::{BlockTemplateTx, Client};

/// Per-batch request count for `get_block_hashes_range`,
/// `fetch_new_pool_data`, and `get_raw_transactions`. Sized so the JSON
/// request body stays well under a megabyte and bitcoind doesn't spend
/// too long on a single batch before yielding results. For the mixed
/// `getmempoolentry`+`getrawtransaction` batch this is the *txid* count;
/// the wire batch is twice that.
const BATCH_CHUNK: usize = 2000;

/// Mempool snapshot data that survives one fetch cycle: the live
/// txid set, fee floor, and chain tip. Returned alongside the raw
/// `block_template` (which Fetcher consumes for GBT synthesis) by
/// `Client::fetch_mempool_state`.
pub struct MempoolState {
    pub live_txids: Vec<Txid>,
    pub min_fee: FeeRate,
    /// Chain tip's hash (block-template's `previousblockhash`).
    /// Compared between cycles to detect newly mined blocks.
    pub tip_hash: BlockHash,
    /// Chain tip's height (block-template's `height` minus one).
    pub tip_height: Height,
}

fn build_entry(txid: Txid, e: MempoolEntry) -> Result<MempoolEntryInfo> {
    let depends = e
        .depends
        .iter()
        .map(|s| Client::parse_txid(s, "depends txid"))
        .collect::<Result<Vec<_>>>()?;
    Ok(MempoolEntryInfo {
        txid,
        vsize: VSize::from(e.vsize as u64),
        weight: Weight::from(e.weight as u64),
        fee: Sats::from(Bitcoin::from(e.fees.base)),
        first_seen: Timestamp::from(e.time),
        depends,
    })
}

fn build_gbt(raw: GetBlockTemplate) -> Result<Vec<BlockTemplateTx>> {
    // Pass 1: decode bodies and stash the 1-based GBT-array indices aside
    // so each `data` hex string and `BlockTemplateTransaction` drops as
    // soon as the tx is pushed.
    let n = raw.transactions.len();
    let mut depends_idx: Vec<Vec<i64>> = Vec::with_capacity(n);
    let mut result: Vec<BlockTemplateTx> = Vec::with_capacity(n);
    for t in raw.transactions {
        let BlockTemplateTransaction {
            data,
            txid,
            depends,
            fee,
            weight,
            ..
        } = t;
        depends_idx.push(depends);
        result.push(BlockTemplateTx {
            txid: Client::parse_txid(&txid, "gbt txid")?,
            fee: Sats::from(fee as u64),
            weight: Weight::from(weight),
            depends: Vec::new(),
            tx: encode::deserialize_hex(&data)?,
        });
    }
    // Pass 2: resolve indices to txids now that the array is complete.
    for (i, indices) in depends_idx.iter().enumerate() {
        let resolved: Vec<Txid> = indices
            .iter()
            .filter_map(|d| {
                let idx = usize::try_from(*d).ok()?.checked_sub(1)?;
                result.get(idx).map(|t| t.txid)
            })
            .collect();
        result[i].depends = resolved;
    }
    Ok(result)
}

/// Convert bitcoind's `mempoolminfee` (BTC/kvB f64) to sat/vB. Round-trip
/// via integer sat/kvB (bitcoind's native CFeeRate unit) so the f64 drift
/// in the JSON-decoded value can't push 1.0 sat/vB to 1.0...e-13 above 1.0
/// and trip `ceil_to(0.001)` downstream.
fn build_min_fee(raw: GetMempoolInfo) -> FeeRate {
    let sat_per_kvb = (raw.mempool_min_fee * 100_000_000.0).round() as u64;
    FeeRate::from_milli(sat_per_kvb)
}

impl Client {
    /// Returns the numbers of block in the longest chain.
    pub fn get_block_count(&self) -> Result<u64> {
        let r: GetBlockCount = self.0.call_with_retry("getblockcount", &[])?;
        Ok(r.0)
    }

    /// Returns the numbers of block in the longest chain.
    pub fn get_last_height(&self) -> Result<Height> {
        self.get_block_count().map(Height::from)
    }

    pub fn get_block<'a, H>(&self, hash: &'a H) -> Result<bitcoin::Block>
    where
        &'a H: Into<&'a bitcoin::BlockHash>,
    {
        let hash: &bitcoin::BlockHash = hash.into();
        let r: GetBlockVerboseZero = self
            .0
            .call_with_retry("getblock", &[serde_json::to_value(hash)?, Value::from(0u8)])?;
        r.block()
            .map_err(|e| Error::Parse(format!("decode getblock: {e}")))
    }

    pub fn get_block_info<'a, H>(&self, hash: &'a H) -> Result<GetBlockVerboseOne>
    where
        &'a H: Into<&'a bitcoin::BlockHash>,
    {
        let hash: &bitcoin::BlockHash = hash.into();
        self.0
            .call_with_retry("getblock", &[serde_json::to_value(hash)?, Value::from(1u8)])
    }

    pub fn get_block_header<'a, H>(&self, hash: &'a H) -> Result<bitcoin::block::Header>
    where
        &'a H: Into<&'a bitcoin::BlockHash>,
    {
        let hash: &bitcoin::BlockHash = hash.into();
        let r: GetBlockHeader = self.0.call_with_retry(
            "getblockheader",
            &[serde_json::to_value(hash)?, Value::Bool(false)],
        )?;
        let bytes = Vec::from_hex(&r.0).map_err(|e| Error::Parse(format!("header hex: {e}")))?;
        bitcoin::consensus::deserialize::<bitcoin::block::Header>(&bytes).map_err(Error::from)
    }

    pub fn get_block_header_info<'a, H>(&self, hash: &'a H) -> Result<GetBlockHeaderVerbose>
    where
        &'a H: Into<&'a bitcoin::BlockHash>,
    {
        let hash: &bitcoin::BlockHash = hash.into();
        self.0
            .call_with_retry("getblockheader", &[serde_json::to_value(hash)?])
    }

    pub fn get_block_hash<H>(&self, height: H) -> Result<BlockHash>
    where
        H: Into<u64> + Copy,
    {
        let height: u64 = height.into();
        let r: GetBlockHash = self
            .0
            .call_with_retry("getblockhash", &[serde_json::to_value(height)?])?;
        Ok(BlockHash::from(r.block_hash()?))
    }

    /// Get every canonical block hash for the inclusive height range
    /// `start..=end` in a single JSON-RPC batch request. Returns hashes
    /// in canonical order (`start`, `start+1`, …, `end`). Use this
    /// whenever resolving more than ~2 heights — one HTTP round-trip
    /// beats N sequential `get_block_hash` calls once the per-call
    /// overhead dominates.
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
            let args = (chunk_start..=chunk_end).map(|h| vec![Value::from(h)]);
            let chunk: Vec<String> = self.0.call_batch("getblockhash", args)?;
            for hex in chunk {
                hashes.push(Self::parse_block_hash(&hex, "getblockhash batch")?);
            }
            chunk_start = chunk_end + 1;
        }
        Ok(hashes)
    }

    pub fn get_tx_out(
        &self,
        txid: &Txid,
        vout: Vout,
        include_mempool: Option<bool>,
    ) -> Result<Option<GetTxOut>> {
        let txid: &bitcoin::Txid = txid.into();
        let mut args: Vec<Value> = vec![
            serde_json::to_value(txid)?,
            serde_json::to_value(u32::from(vout))?,
        ];
        if let Some(mempool) = include_mempool {
            args.push(Value::Bool(mempool));
        }
        self.0.call_with_retry("gettxout", &args)
    }

    pub fn get_raw_mempool(&self) -> Result<Vec<Txid>> {
        let r: GetRawMempool = self.0.call_with_retry("getrawmempool", &[])?;
        r.0.iter()
            .map(|s| Self::parse_txid(s, "mempool txid"))
            .collect()
    }

    pub fn get_raw_transaction<'a, T>(&self, txid: &'a T) -> Result<bitcoin::Transaction>
    where
        &'a T: Into<&'a bitcoin::Txid>,
    {
        let hex = self.get_raw_transaction_hex(txid)?;
        Ok(encode::deserialize_hex::<bitcoin::Transaction>(&hex)?)
    }

    pub fn get_raw_transaction_from<'a, T, H>(
        &self,
        txid: &'a T,
        block_hash: &'a H,
    ) -> Result<bitcoin::Transaction>
    where
        &'a T: Into<&'a bitcoin::Txid>,
        &'a H: Into<&'a bitcoin::BlockHash>,
    {
        let hex = self.get_raw_transaction_hex_from(txid, block_hash)?;
        Ok(encode::deserialize_hex::<bitcoin::Transaction>(&hex)?)
    }

    pub fn get_raw_transaction_hex<'a, T>(&self, txid: &'a T) -> Result<String>
    where
        &'a T: Into<&'a bitcoin::Txid>,
    {
        let txid: &bitcoin::Txid = txid.into();
        let args = [serde_json::to_value(txid)?, Value::Bool(false)];
        self.0.call_with_retry("getrawtransaction", &args)
    }

    pub fn get_raw_transaction_hex_from<'a, T, H>(
        &self,
        txid: &'a T,
        block_hash: &'a H,
    ) -> Result<String>
    where
        &'a T: Into<&'a bitcoin::Txid>,
        &'a H: Into<&'a bitcoin::BlockHash>,
    {
        let txid: &bitcoin::Txid = txid.into();
        let bh: &bitcoin::BlockHash = block_hash.into();
        let args = [
            serde_json::to_value(txid)?,
            Value::Bool(false),
            serde_json::to_value(bh)?,
        ];
        self.0.call_with_retry("getrawtransaction", &args)
    }

    pub fn get_mempool_raw_tx(&self, txid: &Txid) -> Result<bitcoin::Transaction> {
        self.get_raw_transaction(txid)
    }

    /// Batched `getrawtransaction` over a slice of txids. Returns a map keyed
    /// by txid containing the deserialized tx. Individual failures (e.g. a
    /// tx that evicted between the listing and this call) are logged and
    /// dropped so a single bad entry doesn't kill the batch.
    ///
    /// Chunked at `BATCH_CHUNK` requests per round-trip.
    pub fn get_raw_transactions(
        &self,
        txids: &[Txid],
    ) -> Result<FxHashMap<Txid, bitcoin::Transaction>> {
        let mut out: FxHashMap<Txid, bitcoin::Transaction> =
            FxHashMap::with_capacity_and_hasher(txids.len(), Default::default());

        for chunk in txids.chunks(BATCH_CHUNK) {
            let args = chunk.iter().map(|t| {
                let bt: &bitcoin::Txid = t.into();
                vec![
                    serde_json::to_value(bt).unwrap_or(Value::Null),
                    Value::Bool(false),
                ]
            });
            let results: Vec<Result<String>> =
                self.0.call_batch_per_item("getrawtransaction", args)?;

            for (txid, res) in chunk.iter().zip(results) {
                match res.and_then(|hex| Ok(encode::deserialize_hex::<bitcoin::Transaction>(&hex)?))
                {
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

    pub fn send_raw_transaction(&self, hex: &str) -> Result<Txid> {
        let txid: bitcoin::Txid = self
            .0
            .call_once("sendrawtransaction", &[Value::String(hex.to_string())])
            .map_err(|e| {
                // Bitcoin Core returns RPC error codes for client-side problems
                // (decode failed, verification failed, already in chain, etc.).
                // Surface these as 400 (Parse) so HTTP callers see a 4xx, matching
                // mempool.space's POST /api/tx behavior.
                if let Error::CorepcRPC(JsonRpcError::Rpc(rpc)) = &e
                    && matches!(rpc.code, -22 | -25 | -26 | -27)
                {
                    return Error::Parse(rpc.message.clone());
                }
                e
            })?;
        Ok(Txid::from(txid))
    }

    /// Core's projected next block + live mempool txid set +
    /// `mempoolminfee`, fetched in a single bitcoind round-trip. GBT
    /// carries each tx's full body and stats, so block 0 is exact even
    /// when a tx vanishes from the mempool listing between the GBT and
    /// `getrawmempool` calls; no follow-up entry fetch can race it.
    /// Returns the passthrough `MempoolState` and the raw
    /// `block_template` (consumed downstream by GBT synthesis), in one
    /// batched round-trip: `getblocktemplate` + `getrawmempool false`
    /// + `getmempoolinfo`.
    pub fn fetch_mempool_state(&self) -> Result<(MempoolState, Vec<BlockTemplateTx>)> {
        let requests: [(&str, Vec<Value>); 3] = [
            (
                "getblocktemplate",
                vec![serde_json::json!({ "rules": ["segwit"] })],
            ),
            ("getrawmempool", vec![Value::Bool(false)]),
            ("getmempoolinfo", vec![]),
        ];
        let mut out = self.0.call_mixed_batch(&requests)?.into_iter();
        let template_raw = out.next().ok_or(Error::Internal("missing gbt"))??;
        let txids_raw = out.next().ok_or(Error::Internal("missing rawmempool"))??;
        let info_raw = out.next().ok_or(Error::Internal("missing mempoolinfo"))??;

        let txid_strs: Vec<String> = serde_json::from_str(txids_raw.get())?;
        let live_txids: Vec<Txid> = txid_strs
            .iter()
            .map(|s| Self::parse_txid(s, "mempool txid"))
            .collect::<Result<Vec<_>>>()?;
        let template: GetBlockTemplate = serde_json::from_str(template_raw.get())?;
        let tip_hash = Self::parse_block_hash(&template.previous_block_hash, "previousblockhash")?;
        let tip_height =
            Height::from(u64::try_from(template.height - 1).map_err(|_| {
                Error::Parse(format!("gbt height out of range: {}", template.height))
            })?);
        let block_template = build_gbt(template)?;
        let min_fee = build_min_fee(serde_json::from_str(info_raw.get())?);

        Ok((
            MempoolState {
                live_txids,
                min_fee,
                tip_hash,
                tip_height,
            },
            block_template,
        ))
    }

    /// Mixed batch of `getmempoolentry` + `getrawtransaction` for the
    /// same txid set in one round-trip. Returns the entries vec and the
    /// raw-tx map keyed by txid. Per-item -5 (NOT_FOUND — tx evicted
    /// between the listing and this call) drops silently for either leg;
    /// transport-level failures still propagate. Chunked at `BATCH_CHUNK`
    /// txids per round-trip (2× that on the wire).
    pub fn fetch_new_pool_data(
        &self,
        txids: &[Txid],
    ) -> Result<(Vec<MempoolEntryInfo>, FxHashMap<Txid, bitcoin::Transaction>)> {
        let mut entries: Vec<MempoolEntryInfo> = Vec::with_capacity(txids.len());
        let mut txs: FxHashMap<Txid, bitcoin::Transaction> =
            FxHashMap::with_capacity_and_hasher(txids.len(), Default::default());

        for chunk in txids.chunks(BATCH_CHUNK) {
            let mut requests: Vec<(&str, Vec<Value>)> = Vec::with_capacity(chunk.len() * 2);
            for txid in chunk {
                let bt: &bitcoin::Txid = txid.into();
                let tv = serde_json::to_value(bt).unwrap_or(Value::Null);
                requests.push(("getmempoolentry", vec![tv.clone()]));
                requests.push(("getrawtransaction", vec![tv, Value::Bool(false)]));
            }

            let results = self.0.call_mixed_batch(&requests)?;
            let mut iter = results.into_iter();
            for txid in chunk {
                let entry_res = iter.next().ok_or(Error::Internal("missing entry"))?;
                let raw_res = iter.next().ok_or(Error::Internal("missing raw"))?;

                match entry_res.and_then(|raw| {
                    let me: MempoolEntry = serde_json::from_str(raw.get())?;
                    build_entry(*txid, me)
                }) {
                    Ok(info) => entries.push(info),
                    Err(Error::CorepcRPC(JsonRpcError::Rpc(rpc))) if rpc.code == RPC_NOT_FOUND => {}
                    Err(e) => {
                        debug!(txid = %txid, error = %e, "getmempoolentry mixed batch: item failed")
                    }
                }

                match raw_res.and_then(|raw| {
                    let hex: String = serde_json::from_str(raw.get())?;
                    Ok(encode::deserialize_hex::<bitcoin::Transaction>(&hex)?)
                }) {
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

    pub fn get_closest_valid_height(&self, hash: BlockHash) -> Result<(Height, BlockHash)> {
        debug!("Get closest valid height...");

        let mut current = hash;
        loop {
            let info = self.get_block_header_info(&current)?;
            if info.confirmations > 0 {
                return Ok((Height::from(info.height as u64), current));
            }
            let prev = info.previous_block_hash.ok_or(Error::NotFound(
                "Reached genesis without finding main chain".into(),
            ))?;
            current = Self::parse_block_hash(&prev, "previousblockhash")?;
        }
    }

    pub fn get_blockchain_info(&self) -> Result<GetBlockchainInfo> {
        self.0.call_with_retry("getblockchaininfo", &[])
    }

    /// Bitcoin network the connected node is running on, derived from
    /// `getblockchaininfo.chain`.
    pub fn get_network(&self) -> Result<bitcoin::Network> {
        let chain = self.get_blockchain_info()?.chain;
        bitcoin::Network::from_core_arg(&chain)
            .map_err(|e| Error::Parse(format!("getblockchaininfo.chain '{chain}': {e}")))
    }

    pub fn wait_for_synced_node(&self) -> Result<()> {
        let is_synced = || -> Result<bool> {
            let info = self.get_blockchain_info()?;
            Ok(info.headers == info.blocks)
        };

        if !is_synced()? {
            info!("Waiting for node to sync...");
            while !is_synced()? {
                sleep(Duration::from_secs(1))
            }
        }

        Ok(())
    }

    fn parse_txid(s: &str, label: &str) -> Result<Txid> {
        s.parse::<bitcoin::Txid>()
            .map(Txid::from)
            .map_err(|e| Error::Parse(format!("{label}: {e}")))
    }

    fn parse_block_hash(s: &str, label: &str) -> Result<BlockHash> {
        s.parse::<bitcoin::BlockHash>()
            .map(BlockHash::from)
            .map_err(|e| Error::Parse(format!("{label}: {e}")))
    }
}
