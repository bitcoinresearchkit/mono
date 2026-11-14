//! The next block's template (BIP 22), built from projected block 0 at the tip it extends and
//! served only once Core accepts the block it describes.

use std::{sync::Arc, time::Instant};

use bitcoin::{
    Amount, Block, BlockHash as BitcoinBlockHash, CompactTarget, OutPoint, ScriptBuf, Sequence,
    Target, Transaction, TxIn, TxMerkleNode, TxOut, Witness, WitnessMerkleNode, Wtxid,
    absolute::LockTime,
    block::{self, Header},
    consensus::encode::{VarInt, serialize},
    hashes::Hash,
    hex::DisplayHex,
    merkle_tree,
    opcodes::OP_TRUE,
    params::Params,
    transaction,
};
use brk_rpc::{Client, CoreTemplateHeader};
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

/// The template of one snapshot's block 0 at one tip, accepted by Core.
pub(crate) struct Built {
    snapshot: Arc<Snapshot>,
    tip: BlockHash,
    pub json: BlockTemplateJson,
}

impl Built {
    /// Builds the template and has Core check the block it describes: none when Core rejects it
    /// (its reason logged) or can't be asked.
    pub fn new(
        snapshot: Arc<Snapshot>,
        tip: BlockHash,
        facts: &TipFacts,
        core: Option<&CoreTemplateHeader>,
        client: &Client,
    ) -> Option<Self> {
        let json = accepted(&snapshot.block0, facts, core, client)?;
        Some(Self { snapshot, tip, json })
    }

    pub fn is_of(&self, snapshot: &Arc<Snapshot>, tip: &BlockHash) -> bool {
        Arc::ptr_eq(&self.snapshot, snapshot) && self.tip == *tip
    }
}

/// `block0`'s template at `facts`' tip if Core accepts its block, as JSON.
fn accepted(
    block0: &[TxRecord],
    facts: &TipFacts,
    core: Option<&CoreTemplateHeader>,
    client: &Client,
) -> Option<BlockTemplateJson> {
    let started = Instant::now();
    let (template, proposal) = build(block0, facts, Timestamp::now());
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
    let verdict = client.propose_block(&proposal);
    let proposed = started.elapsed() - built;
    match verdict {
        Ok(None) => {}
        Ok(Some(reason)) => {
            warn!(height, reason, "block template: Core rejects its block");
            return None;
        }
        Err(error) => {
            warn!(height, %error, "block template: Core's check failed");
            return None;
        }
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
        propose_us = proposed.as_micros() as u64,
        serialize_us = (started.elapsed() - built - proposed).as_micros() as u64,
        "block template: accepted"
    );
    Some(BlockTemplateJson::new(json))
}

/// The template of the block after `tip` holding `block0`'s transactions in its order (every
/// parent first), dated `now` or the earliest time the chain allows, and the block it describes
/// ([`proposal`]).
fn build(block0: &[TxRecord], tip: &TipFacts, now: Timestamp) -> (BlockTemplate, String) {
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
    let template = BlockTemplate {
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
    };
    let proposal = proposal(&template, tip.header.block_hash(), bits, commitment);
    (template, proposal)
}

/// The block `template` describes as hex, under the least coinbase a miner could add: the BIP 34
/// height, the witness reserved value, and outputs paying `coinbasevalue` and committing to the
/// witnesses.
fn proposal(template: &BlockTemplate, tip: BitcoinBlockHash, bits: CompactTarget, commitment: Vec<u8>) -> String {
    let coinbase = Transaction {
        version: transaction::Version::TWO,
        lock_time: LockTime::ZERO,
        input: vec![TxIn {
            previous_output: OutPoint::null(),
            script_sig: ScriptBuf::builder().push_int(i64::from(*template.height)).into_script(),
            sequence: Sequence::MAX,
            witness: Witness::from_slice(&[WITNESS_RESERVED_VALUE]),
        }],
        output: vec![
            TxOut {
                value: Amount::from_sat(u64::from(template.coinbasevalue)),
                script_pubkey: ScriptBuf::from_bytes(vec![OP_TRUE.to_u8()]),
            },
            TxOut { value: Amount::ZERO, script_pubkey: ScriptBuf::from_bytes(commitment) },
        ],
    };
    let txids = std::iter::once(coinbase.compute_txid())
        .chain(template.transactions.iter().map(|tx| tx.txid.into()))
        .map(|txid: bitcoin::Txid| txid.to_raw_hash());
    let header = Header {
        version: block::Version::from_consensus(template.version),
        prev_blockhash: tip,
        merkle_root: TxMerkleNode::from_raw_hash(merkle_tree::calculate_root(txids).expect("the coinbase's")),
        time: *template.curtime,
        bits,
        nonce: 0,
    };
    let body: usize = template.transactions.iter().map(|tx| tx.data.len()).sum();
    let mut hex = String::with_capacity(body + 2_000);
    hex.push_str(&serialize(&header).to_lower_hex_string());
    hex.push_str(&serialize(&VarInt(template.transactions.len() as u64 + 1)).to_lower_hex_string());
    hex.push_str(&serialize(&coinbase).to_lower_hex_string());
    template.transactions.iter().for_each(|tx| hex.push_str(&tx.data));
    hex
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
