//! The next block's template (BIP 22): projected block 0 at the tip it extends, built as JSON
//! once per block 0 at a tip, each request refreshing only its `curtime`, as Core's
//! `getblocktemplate` reuses its block and refreshes the time. Core isn't asked whether its
//! block is valid, except by debug builds, which check each published one ([`check`]).

#[cfg(debug_assertions)]
mod check;

use std::{io::Write, sync::Arc};

use bitcoin::{
    Block, CompactTarget, Target, WitnessMerkleNode, Wtxid, block::Header, hashes::Hash,
    hex::DisplayHex, params::Params,
};
use brk_types::{Height, Sats, Timestamp, Txid, merkle_root, push_hex, reversed_hex, tag_hash};
use rapidhash::RapidHashMap;

#[cfg(debug_assertions)]
pub(crate) use check::Checker;

use crate::stores::tx_store::TxRecord;

/// BIP 9 version bits with nothing signaled, as Core sets it with no deployment in progress.
const VERSION: i32 = 0x2000_0000;
/// The witness reserved value Core's coinbase carries (BIP 141): 32 zero bytes.
const WITNESS_RESERVED_VALUE: [u8; 32] = [0; 32];
/// The bytes `curtime`'s number takes in the JSON, right-aligned in spaces: a `u32` has at most
/// 10 digits, so a request rewrites it in place.
const CURTIME_WIDTH: usize = 10;

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

    /// The earliest time the next block may carry: Core's `GetMinimumTime`, which applies BIP 94
    /// and BIP 54 on every network.
    fn next_mintime(&self) -> Timestamp {
        let height = u64::from(*self.height + 1);
        let interval = Params::MAINNET.difficulty_adjustment_interval();
        let mut mintime = *self.median_time + 1;
        if height.is_multiple_of(interval) {
            mintime = mintime.max(self.header.time.saturating_sub(600));
        }
        if height % interval == interval - 1 {
            mintime = mintime.max(*self.epoch_start_time);
        }
        Timestamp::from(mintime)
    }
}

/// One projection's block 0 as a template, at the tip it was packed at.
pub(crate) struct Template {
    /// Undated: `curtime`'s slot holds spaces until [`DatedTemplate::json`] writes a copy's.
    json: Box<[u8]>,
    /// Where `curtime`'s [`CURTIME_WIDTH`] bytes start in `json`.
    curtime_at: usize,
    mintime: Timestamp,
    /// `json`'s content hash, its date apart: the date joins it per request.
    hash: u64,
}

impl Template {
    /// The template of the block after `tip` holding `block0`'s transactions in its order (every
    /// parent first). The JSON is written here, field by field in Core's order, as
    /// `brk_types::BlockTemplate` reads it: through `serde_json`, scanning every transaction's
    /// hex for characters to escape took a third of the build.
    pub fn build(block0: &[TxRecord], tip: &TipFacts) -> Self {
        let height = Height::new(*tip.height + 1);
        let mintime = tip.next_mintime();
        let bits = tip.next_bits();
        let fees: Sats = block0.iter().map(|record| record.entry.fee).sum();
        let position: RapidHashMap<Txid, u32> =
            block0.iter().enumerate().map(|(i, record)| (record.entry.txid, i as u32 + 1)).collect();
        let body: usize = block0.iter().map(|record| record.tx.total_size).sum();
        let mut json = Vec::with_capacity(2 * body + 300 * block0.len() + 1_000);
        let w = &mut json;
        write!(
            w,
            r#"{{"version":{VERSION},"rules":["csv","!segwit","taproot"],"vbavailable":{{}},"vbrequired":0,"previousblockhash":"{}","transactions":["#,
            tip.header.block_hash()
        )
        .expect("writes to memory");
        for (i, record) in block0.iter().enumerate() {
            if i > 0 {
                w.push(b',');
            }
            w.extend_from_slice(br#"{"data":""#);
            push_hex(&record.tx.encode_bytes(), w);
            write!(w, r#"","txid":"{}","hash":""#, record.entry.txid).expect("writes to memory");
            w.extend_from_slice(&reversed_hex(&record.entry.wtxid.to_byte_array()));
            w.extend_from_slice(br#"","depends":["#);
            let mut depends: Vec<u32> =
                record.tx.input.iter().filter_map(|input| position.get(&input.txid).copied()).collect();
            depends.sort_unstable();
            depends.dedup();
            for (j, parent) in depends.iter().enumerate() {
                write!(w, "{}{parent}", if j > 0 { "," } else { "" }).expect("writes to memory");
            }
            write!(
                w,
                r#"],"fee":{},"sigops":{},"weight":{}}}"#,
                u64::from(record.entry.fee),
                u64::from(record.policy_sigops()),
                u64::from(record.entry.weight)
            )
            .expect("writes to memory");
        }
        write!(
            w,
            r#"],"coinbaseaux":{{}},"coinbasevalue":{},"target":"{}","mintime":{},"mutable":["time","transactions","prevblock"],"noncerange":"00000000ffffffff","sigoplimit":80000,"sizelimit":4000000,"weightlimit":4000000,"curtime":"#,
            u64::from(height.subsidy() + fees),
            Target::from_compact(bits).to_be_bytes().as_hex(),
            *mintime
        )
        .expect("writes to memory");
        let curtime_at = w.len();
        w.extend_from_slice(&[b' '; CURTIME_WIDTH]);
        write!(
            w,
            r#","bits":"{:08x}","height":{},"default_witness_commitment":"{}"}}"#,
            bits.to_consensus(),
            *height,
            witness_commitment(block0.iter().map(|record| record.entry.wtxid)).as_hex()
        )
        .expect("writes to memory");
        let hash = tag_hash(&json);
        Self { json: json.into(), curtime_at, mintime, hash }
    }

    /// The template as a request gets it: dated now, or `mintime` if later.
    pub fn dated(self: &Arc<Self>) -> DatedTemplate {
        DatedTemplate { curtime: *Timestamp::now().max(self.mintime), template: Arc::clone(self) }
    }
}

/// A template with its date, named before its bytes are copied: a request whose validator
/// matches costs no copy.
pub struct DatedTemplate {
    template: Arc<Template>,
    curtime: u32,
}

impl DatedTemplate {
    /// What names the bytes: the undated bytes' hash with the date.
    pub fn identity(&self) -> u64 {
        let mut identity = [0; 12];
        identity[..8].copy_from_slice(&self.template.hash.to_le_bytes());
        identity[8..].copy_from_slice(&self.curtime.to_le_bytes());
        tag_hash(&identity)
    }

    /// The stored bytes copied, the date written into `curtime`'s slot.
    pub fn json(&self) -> Vec<u8> {
        let Template { json, curtime_at, .. } = &*self.template;
        let mut json = json.to_vec();
        let slot = &mut json[*curtime_at..][..CURTIME_WIDTH];
        write!(&mut *slot, "{:>CURTIME_WIDTH$}", self.curtime).expect("a u32 fits its slot");
        json
    }
}

/// The coinbase output script committing to `wtxids` (the coinbase's own counted as zeros) and
/// the witness reserved value (BIP 141): `OP_RETURN` pushing `aa21a9ed` and the commitment.
fn witness_commitment(wtxids: impl Iterator<Item = Wtxid>) -> Vec<u8> {
    let hashes = std::iter::once(Wtxid::all_zeros()).chain(wtxids).map(|wtxid| wtxid.to_byte_array()).collect();
    let root = WitnessMerkleNode::from_byte_array(merkle_root(hashes).expect("the coinbase's"));
    let commitment = Block::compute_witness_commitment(&root, &WITNESS_RESERVED_VALUE);
    let mut script = vec![0x6a, 0x24, 0xaa, 0x21, 0xa9, 0xed];
    script.extend_from_slice(&commitment.to_byte_array());
    script
}
