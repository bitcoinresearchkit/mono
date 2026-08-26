use std::{thread::sleep, time::Duration};

use bitcoin::{
    Block, BlockHash as BitcoinBlockHash, Network, Transaction, Txid as BitcoinTxid, block::Header,
    consensus::encode,
};
use brk_error::{Error, Result};
use brk_types::{BlockHash, FeeRate, Height, MempoolEntryInfo, Sats, Txid, Vout, Weight};
use corepc_jsonrpc::error::Error as JsonRpcError;
use corepc_types::{
    v17::{
        BlockTemplateTransaction, GetBlockCount, GetBlockHeader, GetBlockHeaderVerbose,
        GetBlockTemplate, GetBlockVerboseOne, GetBlockVerboseZero, GetTxOut,
    },
    v24::GetMempoolInfo,
    v28::GetBlockchainInfo,
};
use rustc_hash::FxHashMap;
use serde_json::{from_str, json, value::RawValue};
use tracing::{debug, info};

/// Bitcoin Core's `-5` (`RPC_INVALID_ADDRESS_OR_KEY`) is the expected
/// response when querying a confirmed transaction without `-txindex`.
/// The mempool fetcher tolerates these per-item failures silently.
const RPC_NOT_FOUND: i32 = -5;
const NO_ARGS: [(); 0] = [];

use crate::BlockTemplateTx;

use super::{
    Client, mempool_entry::MempoolEntry, mempool_state::MempoolState, rpc_call::RpcCall,
    txid_array_parser::TxidArrayParser,
};

/// Per-batch request count for `get_block_hashes_range`,
/// `fetch_new_pool_data`, and `get_raw_transactions`. Sized so the JSON
/// request body stays well under a megabyte and bitcoind doesn't spend
/// too long on a single batch before yielding results. For the mixed
/// `getmempoolentry`+`getrawtransaction` batch this is the *txid* count;
/// the wire batch is twice that.
const BATCH_CHUNK: usize = 2000;

impl Client {
    fn build_gbt(raw: GetBlockTemplate) -> Result<Vec<BlockTemplateTx>> {
        let mut result = Vec::with_capacity(raw.transactions.len());
        for transaction in raw.transactions {
            let BlockTemplateTransaction {
                data,
                txid,
                depends,
                fee,
                weight,
                ..
            } = transaction;
            let fee =
                u64::try_from(fee).map_err(|_| Error::Parse(format!("negative gbt fee: {fee}")))?;
            // Core defines dependencies as one-based indices of preceding
            // transactions, so resolve them while consuming the template.
            let depends = depends
                .into_iter()
                .filter_map(|dependency| {
                    result
                        .get(usize::try_from(dependency).ok()?.checked_sub(1)?)
                        .map(|tx: &BlockTemplateTx| tx.txid)
                })
                .collect();
            result.push(BlockTemplateTx {
                txid: Self::parse_txid(&txid, "gbt txid")?,
                fee: Sats::from(fee),
                weight: Weight::from(weight),
                depends,
                tx: encode::deserialize_hex(&data)?,
            });
        }
        Ok(result)
    }

    /// Convert bitcoind's `mempoolminfee` (BTC/kvB f64) to sat/vB. Round-trip
    /// via integer sat/kvB (bitcoind's native CFeeRate unit) so JSON f64 drift
    /// cannot move an exact sat/vB boundary upward.
    fn build_min_fee(raw: GetMempoolInfo) -> FeeRate {
        let sat_per_kvb = (raw.mempool_min_fee * 100_000_000.0).round() as u64;
        FeeRate::from_milli(sat_per_kvb)
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

    pub fn get_block_header<'a, H>(&self, hash: &'a H) -> Result<Header>
    where
        &'a H: Into<&'a BitcoinBlockHash>,
    {
        let hash: &BitcoinBlockHash = hash.into();
        let r: GetBlockHeader = self.0.call_with_retry("getblockheader", &(hash, false))?;
        Ok(encode::deserialize_hex(&r.0)?)
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

    pub fn get_tx_out(
        &self,
        txid: &Txid,
        vout: Vout,
        include_mempool: Option<bool>,
    ) -> Result<Option<GetTxOut>> {
        let txid: &BitcoinTxid = txid.into();
        let vout = u32::from(vout);
        match include_mempool {
            Some(include_mempool) => self
                .0
                .call_with_retry("gettxout", &(txid, vout, include_mempool)),
            None => self.0.call_with_retry("gettxout", &(txid, vout)),
        }
    }

    pub fn get_raw_mempool(&self) -> Result<Vec<Txid>> {
        let raw: Box<RawValue> = self.0.call_with_retry("getrawmempool", &NO_ARGS)?;
        Ok(TxidArrayParser::parse(raw.get())?)
    }

    pub fn get_raw_transaction<'a, T>(&self, txid: &'a T) -> Result<Transaction>
    where
        &'a T: Into<&'a BitcoinTxid>,
    {
        let hex = self.get_raw_transaction_hex(txid)?;
        Ok(encode::deserialize_hex(&hex)?)
    }

    pub fn get_raw_transaction_from<'a, T, H>(
        &self,
        txid: &'a T,
        block_hash: &'a H,
    ) -> Result<Transaction>
    where
        &'a T: Into<&'a BitcoinTxid>,
        &'a H: Into<&'a BitcoinBlockHash>,
    {
        let hex = self.get_raw_transaction_hex_from(txid, block_hash)?;
        Ok(encode::deserialize_hex(&hex)?)
    }

    pub fn get_raw_transaction_hex<'a, T>(&self, txid: &'a T) -> Result<String>
    where
        &'a T: Into<&'a BitcoinTxid>,
    {
        let txid: &BitcoinTxid = txid.into();
        self.0.call_with_retry("getrawtransaction", &(txid, false))
    }

    pub fn get_raw_transaction_hex_from<'a, T, H>(
        &self,
        txid: &'a T,
        block_hash: &'a H,
    ) -> Result<String>
    where
        &'a T: Into<&'a BitcoinTxid>,
        &'a H: Into<&'a BitcoinBlockHash>,
    {
        let txid: &BitcoinTxid = txid.into();
        let block_hash: &BitcoinBlockHash = block_hash.into();
        self.0
            .call_with_retry("getrawtransaction", &(txid, false, block_hash))
    }

    pub fn get_mempool_raw_tx(&self, txid: &Txid) -> Result<Transaction> {
        self.get_raw_transaction(txid)
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

    pub fn send_raw_transaction(&self, hex: &str) -> Result<Txid> {
        let txid: BitcoinTxid = self
            .0
            .call_once("sendrawtransaction", &(hex,))
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
        let tip_height =
            Height::from(u64::try_from(template.height - 1).map_err(|_| {
                Error::Parse(format!("gbt height out of range: {}", template.height))
            })?);
        let block_template = Self::build_gbt(template)?;
        let min_fee = Self::build_min_fee(from_str(info_raw.get())?);

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

    pub fn get_blockchain_info(&self) -> Result<GetBlockchainInfo> {
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

    fn parse_txid(s: &str, label: &str) -> Result<Txid> {
        s.parse::<BitcoinTxid>()
            .map(Txid::from)
            .map_err(|e| Error::Parse(format!("{label}: {e}")))
    }

    fn parse_block_hash(s: &str, label: &str) -> Result<BlockHash> {
        s.parse::<BitcoinBlockHash>()
            .map(BlockHash::from)
            .map_err(|e| Error::Parse(format!("{label}: {e}")))
    }
}
