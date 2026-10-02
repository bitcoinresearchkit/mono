use std::{thread::sleep, time::Duration};

use bitcoin::{
    Block, BlockHash as BitcoinBlockHash, Network, Transaction, Txid as BitcoinTxid,
    consensus::encode,
};
use brk_error::{Error, Result};
use brk_types::{BlockHash, FeeRate, Height, MempoolEntryInfo, Txid};
use corepc_jsonrpc::error::Error as JsonRpcError;
use corepc_types::{
    v17::{
        GetBlockCount, GetBlockHeaderVerbose, GetBlockTemplate, GetBlockVerboseOne,
        GetBlockVerboseZero,
    },
    v24::GetMempoolInfo,
    v28::GetBlockchainInfo,
};
use rustc_hash::FxHashMap;
use serde_json::{from_str, json, value::RawValue};
use tracing::{debug, info};

use super::{
    Client, ClientInner, mempool_entry::MempoolEntry, mempool_state::MempoolState,
    rpc_call::RpcCall, txid_array_parser::TxidArrayParser,
};
use crate::BlockTemplateTx;

/// Bitcoin Core's `-5` (`RPC_INVALID_ADDRESS_OR_KEY`) is the expected
/// response when querying a confirmed transaction without `-txindex`.
/// The mempool fetcher tolerates these per-item failures silently.
const RPC_NOT_FOUND: i32 = -5;
const NO_ARGS: [(); 0] = [];

/// Per-batch request count for `get_block_hashes_range`,
/// `fetch_new_pool_data`, and `get_raw_transactions`. Sized so the JSON
/// request body stays well under a megabyte and bitcoind doesn't spend
/// too long on a single batch before yielding results. For the mixed
/// `getmempoolentry`+`getrawtransaction` batch this is the *txid* count;
/// the wire batch is twice that.
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

    /// Returns the numbers of block in the longest chain.
    pub fn get_block_count(&self) -> Result<u64> {
        let r: GetBlockCount = self.0.call_with_retry("getblockcount", &NO_ARGS)?;
        Ok(r.0)
    }

    /// Returns the numbers of block in the longest chain.
    pub fn get_last_height(&self) -> Result<Height> {
        self.get_block_count().map(Height::from)
    }

    pub fn get_block<'a, H>(&self, hash: &'a H) -> Result<Block>
    where
        &'a H: Into<&'a BitcoinBlockHash>,
    {
        let hash: &BitcoinBlockHash = hash.into();
        let r: GetBlockVerboseZero = self.0.call_with_retry("getblock", &(hash, 0u8))?;
        r.block()
            .map_err(|e| Error::Parse(format!("decode getblock: {e}")))
    }

    pub fn get_block_info<'a, H>(&self, hash: &'a H) -> Result<GetBlockVerboseOne>
    where
        &'a H: Into<&'a BitcoinBlockHash>,
    {
        let hash: &BitcoinBlockHash = hash.into();
        self.0.call_with_retry("getblock", &(hash, 1u8))
    }

    pub fn get_block_header_info<'a, H>(&self, hash: &'a H) -> Result<GetBlockHeaderVerbose>
    where
        &'a H: Into<&'a BitcoinBlockHash>,
    {
        let hash: &BitcoinBlockHash = hash.into();
        self.0.call_with_retry("getblockheader", &(hash,))
    }

    pub fn get_block_hash<H>(&self, height: H) -> Result<BlockHash>
    where
        H: Into<u64> + Copy,
    {
        let height: u64 = height.into();
        let hash: BitcoinBlockHash = self.0.call_with_retry("getblockhash", &(height,))?;
        Ok(BlockHash::from(hash))
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
            let args = (chunk_start..=chunk_end).map(|height| (height,));
            let chunk: Vec<BitcoinBlockHash> = self.0.call_batch("getblockhash", args)?;
            hashes.extend(chunk.into_iter().map(BlockHash::from));
            chunk_start = chunk_end + 1;
        }
        Ok(hashes)
    }

    /// Batched `getrawtransaction` over a slice of txids. Returns a map keyed
    /// by txid containing the deserialized tx. Individual failures (e.g. a
    /// tx that evicted between the listing and this call) are logged and
    /// dropped so a single bad entry doesn't kill the batch.
    ///
    /// Chunked at `BATCH_CHUNK` requests per round-trip.
    pub fn get_raw_transactions(&self, txids: &[Txid]) -> Result<FxHashMap<Txid, Transaction>> {
        let mut out: FxHashMap<Txid, Transaction> =
            FxHashMap::with_capacity_and_hasher(txids.len(), Default::default());

        for chunk in txids.chunks(BATCH_CHUNK) {
            let args = chunk.iter().map(|t| {
                let txid: &BitcoinTxid = t.into();
                (txid, false)
            });
            let results: Vec<Result<Box<RawValue>>> =
                self.0.call_batch_per_item("getrawtransaction", args)?;

            for (txid, res) in chunk.iter().zip(results) {
                match res.and_then(|raw| {
                    let hex: &str = from_str(raw.get())?;
                    Ok(encode::deserialize_hex(hex)?)
                }) {
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
        let template_args = (json!({ "rules": ["segwit"] }),);
        let calls = [
            RpcCall::new("getblocktemplate", &template_args)?,
            RpcCall::new("getrawmempool", &(false,))?,
            RpcCall::empty("getmempoolinfo")?,
        ];
        let mut out = self.0.call_mixed_batch(&calls)?.into_iter();
        let template_raw = out.next().ok_or(Error::Internal("missing gbt"))??;
        let txids_raw = out.next().ok_or(Error::Internal("missing rawmempool"))??;
        let info_raw = out.next().ok_or(Error::Internal("missing mempoolinfo"))??;

        let live_txids = TxidArrayParser::parse(txids_raw.get())?;
        let template: GetBlockTemplate = from_str(template_raw.get())?;
        let tip_hash = Self::parse_block_hash(&template.previous_block_hash, "previousblockhash")?;
        let tip_height = Self::template_tip_height(template.height)?;
        let block_template = ClientInner::build_gbt(template.transactions)?;
        let info: GetMempoolInfo = from_str(info_raw.get())?;
        let min_fee = Self::build_min_fee(info.mempool_min_fee)?;

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
    ) -> Result<(Vec<MempoolEntryInfo>, FxHashMap<Txid, Transaction>)> {
        let mut entries: Vec<MempoolEntryInfo> = Vec::with_capacity(txids.len());
        let mut txs: FxHashMap<Txid, Transaction> =
            FxHashMap::with_capacity_and_hasher(txids.len(), Default::default());

        for chunk in txids.chunks(BATCH_CHUNK) {
            let mut calls = Vec::with_capacity(chunk.len() * 2);
            for txid in chunk {
                let txid: &BitcoinTxid = txid.into();
                let txid = txid.to_string();
                calls.push(RpcCall::new("getmempoolentry", &(&txid,))?);
                calls.push(RpcCall::new("getrawtransaction", &(&txid, false))?);
            }

            let results = self.0.call_mixed_batch(&calls)?;
            let mut iter = results.into_iter();
            for txid in chunk {
                let entry_res = iter.next().ok_or(Error::Internal("missing entry"))?;
                let raw_res = iter.next().ok_or(Error::Internal("missing raw"))?;

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

                match raw_res.and_then(|raw| {
                    let hex: &str = from_str(raw.get())?;
                    Ok(encode::deserialize_hex(hex)?)
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

    fn template_tip_height(next_height: i64) -> Result<Height> {
        next_height
            .checked_sub(1)
            .and_then(|tip| u32::try_from(tip).ok())
            .map(Height::from)
            .ok_or_else(|| Error::Parse(format!("gbt height out of range: {next_height}")))
    }

    fn parse_block_hash(s: &str, label: &str) -> Result<BlockHash> {
        s.parse::<BitcoinBlockHash>()
            .map(BlockHash::from)
            .map_err(|e| Error::Parse(format!("{label}: {e}")))
    }
}
