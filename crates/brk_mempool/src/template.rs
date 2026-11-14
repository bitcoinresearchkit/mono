//! The next block's template (BIP 22), built from projected block 0 at the tip it extends.

use std::{sync::Arc, time::Instant};

use bitcoin::{
    Block, CompactTarget, Target, WitnessMerkleNode, Wtxid, block::Header, hashes::Hash,
    hex::DisplayHex, merkle_tree, params::Params,
};
use brk_rpc::CoreTemplateHeader;
use brk_types::{BlockHash, BlockTemplate, BlockTemplateTransaction, Height, Sats, Timestamp, Txid};
use rapidhash::RapidHashMap;
use tracing::{debug, warn};

use crate::{BlockTemplateJson, snapshot::Snapshot, stores::tx_store::TxRecord};

/// BIP 9 version bits with nothing signaled, as Core sets it with no deployment in progress.
const VERSION: i32 = 0x2000_0000;
/// The witness reserved value Core's coinbase carries (BIP 141): 32 zero bytes.
const WITNESS_RESERVED_VALUE: [u8; 32] = [0; 32];

/// What the next block's template needs of the chain at its tip, as the indexed chain holds it.
pub struct TipFacts {
    pub height: Height,
    pub header: Header,
    /// The tip's median time past: the next block's time must exceed it.
    pub median_time: Timestamp,
    /// The time of the first block of the tip's difficulty epoch.
    pub epoch_start_time: Timestamp,
}

impl TipFacts {
    /// The next block's compact target: the tip's, or a retarget's at an epoch boundary. Mainnet
    /// rules, which Bitview indexes (no testnet minimum-difficulty blocks).
    fn next_bits(&self) -> CompactTarget {
        let params = &Params::MAINNET;
        if (*self.height + 1) as u64 % params.difficulty_adjustment_interval() != 0 {
            return self.header.bits;
        }
        let timespan = self.header.time.saturating_sub(*self.epoch_start_time);
        CompactTarget::from_next_work_required(self.header.bits, u64::from(timespan), params)
    }
}

/// The template served for one snapshot's block 0 at one tip.
pub(crate) struct Built {
    snapshot: Arc<Snapshot>,
    tip: BlockHash,
    /// None while the chain can't say what the next block's header is.
    pub json: Option<BlockTemplateJson>,
}

impl Built {
    pub fn new(
        snapshot: Arc<Snapshot>,
        tip: BlockHash,
        facts: Option<&TipFacts>,
        core: Option<&CoreTemplateHeader>,
    ) -> Self {
        let json = facts.map(|facts| served(&snapshot.block0, facts, core));
        Self { snapshot, tip, json }
    }

    pub fn is_of(&self, snapshot: &Arc<Snapshot>, tip: &BlockHash) -> bool {
        Arc::ptr_eq(&self.snapshot, snapshot) && self.tip == *tip
    }
}

/// `block0`'s template at `facts`' tip, as JSON.
fn served(block0: &[TxRecord], facts: &TipFacts, core: Option<&CoreTemplateHeader>) -> BlockTemplateJson {
    let started = Instant::now();
    let template = build(block0, facts, Timestamp::now());
    let built = started.elapsed();
    let height = *template.height;
    if let Some(core) = core.filter(|core| core.height == height)
        && (core.bits != template.bits || core.mintime != *template.mintime)
    {
        warn!(
            height,
            bits = template.bits,
            core_bits = core.bits,
            mintime = *template.mintime,
            core_mintime = core.mintime,
            "block template: the chain's next header differs from Core's"
        );
    }
    let json = serde_json::to_vec(&template).expect("a template serializes");
    let fees = u64::from(template.coinbasevalue - template.height.subsidy());
    let weight: u64 = template.transactions.iter().map(|tx| u64::from(tx.weight)).sum();
    let core = core.filter(|core| core.height == height);
    debug!(
        height,
        txs = template.transactions.len(),
        fees,
        weight,
        core_fees = core.map(|core| core.coinbase_value - u64::from(template.height.subsidy())),
        core_weight = core.map(|core| core.weight),
        core_txs = core.map(|core| core.txs),
        build_us = built.as_micros() as u64,
        serialize_us = (started.elapsed() - built).as_micros() as u64,
        "block template: built"
    );
    BlockTemplateJson::new(json)
}

/// The template of the block after `tip` holding `block0`'s transactions in its order (every
/// parent first), dated `now` or the earliest time the chain allows.
fn build(block0: &[TxRecord], tip: &TipFacts, now: Timestamp) -> BlockTemplate {
    let height = Height::new(*tip.height + 1);
    let mintime = Timestamp::from(*tip.median_time + 1);
    let position: RapidHashMap<Txid, u32> =
        block0.iter().enumerate().map(|(i, record)| (record.entry.txid, i as u32 + 1)).collect();
    let transactions: Vec<BlockTemplateTransaction> = block0
        .iter()
        .map(|record| {
            let mut depends: Vec<u32> =
                record.tx.input.iter().filter_map(|input| position.get(&input.txid).copied()).collect();
            depends.sort_unstable();
            depends.dedup();
            BlockTemplateTransaction {
                data: record.tx.encode_bytes().to_lower_hex_string(),
                txid: record.entry.txid,
                hash: record.entry.wtxid.to_string(),
                depends,
                fee: record.entry.fee,
                sigops: record.policy_sigops(),
                weight: record.entry.weight,
            }
        })
        .collect();
    let fees: Sats = block0.iter().map(|record| record.entry.fee).sum();
    let bits = tip.next_bits();
    let commitment = witness_commitment(block0.iter().map(|record| record.entry.wtxid));
    BlockTemplate {
        version: VERSION,
        rules: ["csv", "!segwit", "taproot"].map(String::from).into(),
        vbavailable: Default::default(),
        vbrequired: 0,
        previousblockhash: tip.header.block_hash().into(),
        transactions,
        coinbaseaux: Default::default(),
        coinbasevalue: height.subsidy() + fees,
        target: Target::from_compact(bits).to_be_bytes().to_lower_hex_string(),
        mintime,
        mutable: ["time", "transactions", "prevblock"].map(String::from).into(),
        noncerange: "00000000ffffffff".into(),
        sigoplimit: 80_000,
        sizelimit: 4_000_000,
        weightlimit: 4_000_000,
        curtime: now.max(mintime),
        bits: format!("{:08x}", bits.to_consensus()),
        height,
        default_witness_commitment: commitment.to_lower_hex_string(),
    }
}

/// The coinbase output script committing to `wtxids` (the coinbase's own counted as zeros) and
/// the witness reserved value (BIP 141): `OP_RETURN` pushing `aa21a9ed` and the commitment.
fn witness_commitment(wtxids: impl Iterator<Item = Wtxid>) -> Vec<u8> {
    let hashes = std::iter::once(Wtxid::all_zeros()).chain(wtxids).map(|wtxid| wtxid.to_raw_hash());
    let root = WitnessMerkleNode::from_raw_hash(merkle_tree::calculate_root(hashes).expect("the coinbase's"));
    let commitment = Block::compute_witness_commitment(&root, &WITNESS_RESERVED_VALUE);
    let mut script = vec![0x6a, 0x24, 0xaa, 0x21, 0xa9, 0xed];
    script.extend_from_slice(&commitment.to_byte_array());
    script
}
