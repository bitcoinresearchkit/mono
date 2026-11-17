//! The next block's template (BIP 22): projected block 0 at the tip it extends, built as the
//! route is asked. Core isn't asked whether its block is valid, except by debug builds, which
//! check each published block 0 and panic on one Core rejects ([`Checker`]): the dev machine's
//! run is the check on the packer.

use std::{
    sync::{Arc, Condvar, Mutex},
    thread,
    time::Instant,
};

use bitcoin::{
    Amount, Block, CompactTarget, OutPoint, ScriptBuf, Sequence, Target, Transaction, TxIn,
    TxMerkleNode, TxOut, Witness, WitnessMerkleNode, Wtxid,
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
use brk_rpc::Client;
use brk_types::{BlockHash, BlockTemplate, BlockTemplateTransaction, Height, Sats, Timestamp, Txid};
use rapidhash::RapidHashMap;
use tracing::{debug, warn};

use crate::{snapshot::Snapshot, stores::tx_store::TxRecord};

/// BIP 9 version bits with nothing signaled, as Core sets it with no deployment in progress.
const VERSION: i32 = 0x2000_0000;
/// The witness reserved value Core's coinbase carries (BIP 141): 32 zero bytes.
const WITNESS_RESERVED_VALUE: [u8; 32] = [0; 32];
/// Core's answer to a proposal that doesn't extend its tip (BIP 23): no verdict on the block.
const NOT_ON_CORE_TIP: &str = "inconclusive-not-best-prevblk";

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
        if !((*self.height + 1) as u64).is_multiple_of(params.difficulty_adjustment_interval()) {
            return self.header.bits;
        }
        let timespan = self.header.time.saturating_sub(*self.epoch_start_time);
        CompactTarget::from_next_work_required(self.header.bits, u64::from(timespan), params)
    }
}

/// Debug builds' check on the packer: asks Core, on a thread of its own, whether each published
/// snapshot's block 0 makes a valid block at its tip, and panics with Core's reason when it
/// doesn't. Latest wins: a snapshot handed over while a check runs replaces one still waiting,
/// so checks never queue behind the cycle. Release builds ask Core nothing.
pub(crate) struct Checker {
    next: Arc<(Mutex<Option<Job>>, Condvar)>,
}

struct Job {
    snapshot: Arc<Snapshot>,
    tip: BlockHash,
}

impl Checker {
    /// `tip_facts`: what the chain says of a tip it published.
    pub fn spawn(
        client: Client,
        tip_facts: impl Fn(&BlockHash) -> Option<TipFacts> + Send + 'static,
    ) -> Self {
        let next = Arc::new((Mutex::new(None::<Job>), Condvar::new()));
        if !cfg!(debug_assertions) {
            return Self { next };
        }
        let waiting = Arc::clone(&next);
        thread::Builder::new()
            .name("block-template".into())
            .spawn(move || {
                // The last snapshot and tip Core accepted, and the facts of the tip last read.
                let mut accepted_for: Option<(Arc<Snapshot>, BlockHash)> = None;
                let mut facts: Option<(BlockHash, TipFacts)> = None;
                loop {
                    let job = {
                        let (slot, ready) = &*waiting;
                        let mut slot = ready
                            .wait_while(slot.lock().expect("checker slot"), |job| job.is_none())
                            .expect("checker slot");
                        slot.take().expect("woken with a job")
                    };
                    if accepted_for.as_ref().is_some_and(|(snapshot, tip)| {
                        Arc::ptr_eq(snapshot, &job.snapshot) && *tip == job.tip
                    }) {
                        continue;
                    }
                    if facts.as_ref().is_none_or(|(tip, _)| *tip != job.tip) {
                        facts = tip_facts(&job.tip).map(|read| (job.tip, read));
                    }
                    if facts
                        .as_ref()
                        .is_some_and(|(_, facts)| accepted(&job.snapshot.block0, facts, &client))
                    {
                        accepted_for = Some((job.snapshot, job.tip));
                    }
                }
            })
            .expect("the block template thread starts");
        Self { next }
    }

    /// Hands over `snapshot` at `tip`, replacing one not yet taken. Handed over at every
    /// publication: one Core couldn't check is tried again, one it accepted is skipped.
    pub fn check(&self, snapshot: Arc<Snapshot>, tip: BlockHash) {
        if !cfg!(debug_assertions) {
            return;
        }
        let (slot, ready) = &*self.next;
        *slot.lock().expect("checker slot") = Some(Job { snapshot, tip });
        ready.notify_one();
    }
}

/// Whether Core accepts the block of `block0`'s template at `facts`' tip. Panics with Core's
/// reason when it rejects it; false when Core gave no verdict (its tip moved on, or the call
/// failed).
fn accepted(block0: &[TxRecord], facts: &TipFacts, client: &Client) -> bool {
    let started = Instant::now();
    let template = build(block0, facts, Timestamp::now());
    let proposal = proposal(&template, facts);
    let built = started.elapsed();
    let height = *template.height;
    match client.propose_block(&proposal) {
        Ok(None) => {
            debug!(
                height,
                txs = template.transactions.len(),
                fees = u64::from(template.coinbasevalue - template.height.subsidy()),
                weight = template.transactions.iter().map(|tx| u64::from(tx.weight)).sum::<u64>(),
                build_us = built.as_micros() as u64,
                propose_us = (started.elapsed() - built).as_micros() as u64,
                "block template: Core accepts block 0"
            );
            true
        }
        Ok(Some(reason)) if reason == NOT_ON_CORE_TIP => {
            debug!(height, "block template: Core's tip moved before its check");
            false
        }
        Ok(Some(reason)) => {
            panic!("block template: Core rejects block 0 at height {height}: {reason}")
        }
        Err(error) => {
            warn!(height, %error, "block template: Core's check failed");
            false
        }
    }
}

/// The template of the block after `tip` holding `block0`'s transactions in its order (every
/// parent first), dated `now` or the earliest time the chain allows.
pub(crate) fn build(block0: &[TxRecord], tip: &TipFacts, now: Timestamp) -> BlockTemplate {
    let height = Height::new(*tip.height + 1);
    // Core's GetMinimumTime applies BIP94 and BIP54 on every network.
    let interval = Params::MAINNET.difficulty_adjustment_interval();
    let mut mintime = *tip.median_time + 1;
    if u64::from(*height).is_multiple_of(interval) {
        mintime = mintime.max(tip.header.time.saturating_sub(600));
    }
    if u64::from(*height) % interval == interval - 1 {
        mintime = mintime.max(*tip.epoch_start_time);
    }
    let mintime = Timestamp::from(mintime);
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
                data: record.tx.encode_hex(),
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

/// The block `template` describes at `tip` as hex, under the least coinbase a miner could add:
/// the BIP 34 height, the witness reserved value, and outputs paying `coinbasevalue` and carrying
/// `default_witness_commitment`.
fn proposal(template: &BlockTemplate, tip: &TipFacts) -> String {
    let commitment = ScriptBuf::from_hex(&template.default_witness_commitment)
        .expect("the template's own hex");
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
            TxOut { value: Amount::ZERO, script_pubkey: commitment },
        ],
    };
    let txids = std::iter::once(coinbase.compute_txid())
        .chain(template.transactions.iter().map(|tx| tx.txid.into()))
        .map(|txid: bitcoin::Txid| txid.to_raw_hash());
    let header = Header {
        version: block::Version::from_consensus(template.version),
        prev_blockhash: tip.header.block_hash(),
        merkle_root: TxMerkleNode::from_raw_hash(merkle_tree::calculate_root(txids).expect("the coinbase's")),
        time: *template.curtime,
        bits: tip.next_bits(),
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
