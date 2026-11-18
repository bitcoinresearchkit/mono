//! Debug builds' check on the packer: Core is asked whether each published template makes a
//! valid block, as served, read back as clients read it. The dev machine's run is the check.

use std::{
    sync::{Arc, Condvar, Mutex},
    thread,
    time::Instant,
};

use bitcoin::{
    Amount, CompactTarget, OutPoint, ScriptBuf, Sequence, Transaction, TxIn, TxMerkleNode, TxOut,
    Witness,
    absolute::LockTime,
    block::{self, Header},
    consensus::encode::{VarInt, serialize},
    hashes::Hash,
    hex::DisplayHex,
    opcodes::OP_TRUE,
    transaction,
};
use brk_rpc::Client;
use brk_types::{BlockTemplate, merkle_root};
use tracing::{debug, error, warn};

use super::{Template, WITNESS_RESERVED_VALUE};

/// Core's answer to a proposal that doesn't extend its tip (BIP 23): no verdict on the block.
const NOT_ON_CORE_TIP: &str = "inconclusive-not-best-prevblk";

/// Asks Core on a thread of its own, latest wins: a template handed over while a check runs
/// replaces one still waiting, so checks never queue behind the cycle.
pub(crate) struct Checker {
    next: Arc<(Mutex<Option<Arc<Template>>>, Condvar)>,
}

impl Checker {
    pub fn spawn(client: Client) -> Self {
        let next = Arc::new((Mutex::new(None::<Arc<Template>>), Condvar::new()));
        let waiting = Arc::clone(&next);
        thread::Builder::new()
            .name("block-template".into())
            .spawn(move || {
                // The last template Core gave a verdict on.
                let mut judged: Option<Arc<Template>> = None;
                loop {
                    let template = {
                        let (slot, ready) = &*waiting;
                        let mut slot = ready
                            .wait_while(slot.lock().expect("checker slot"), |job| job.is_none())
                            .expect("checker slot");
                        slot.take().expect("woken with a job")
                    };
                    if judged.as_ref().is_some_and(|last| Arc::ptr_eq(last, &template)) {
                        continue;
                    }
                    if judged_by_core(&template, &client) {
                        judged = Some(template);
                    }
                }
            })
            .expect("the block template thread starts");
        Self { next }
    }

    /// Hands over the published `template`, replacing one not yet taken. Handed over at every
    /// publication: one Core gave no verdict on is tried again, one it judged is skipped.
    pub fn check(&self, template: Arc<Template>) {
        let (slot, ready) = &*self.next;
        *slot.lock().expect("checker slot") = Some(template);
        ready.notify_one();
    }
}

/// Whether Core gave a verdict on `template`'s block. A rejection, or a template that doesn't
/// read as `BlockTemplate`, is logged as an error for each template, and checking goes on.
fn judged_by_core(template: &Template, client: &Client) -> bool {
    let started = Instant::now();
    let template: BlockTemplate = match serde_json::from_slice(&template.json) {
        Ok(template) => template,
        Err(reason) => {
            error!(%reason, "block template: the JSON doesn't read as a BlockTemplate");
            return true;
        }
    };
    let proposal = proposal(&template);
    let built = started.elapsed();
    let height = *template.height;
    match client.propose_block(&proposal) {
        Ok(None) => {
            debug!(
                height,
                txs = template.transactions.len(),
                fees = u64::from(template.coinbasevalue - template.height.subsidy()),
                weight = template.transactions.iter().map(|tx| u64::from(tx.weight)).sum::<u64>(),
                read_us = built.as_micros() as u64,
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
            error!(height, %reason, "block template: Core rejects block 0");
            true
        }
        Err(error) => {
            warn!(height, %error, "block template: Core's check failed");
            false
        }
    }
}

/// The block `template` describes as hex, under the least coinbase a miner could add: the BIP 34
/// height, the witness reserved value, and outputs paying `coinbasevalue` and carrying
/// `default_witness_commitment`.
fn proposal(template: &BlockTemplate) -> String {
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
        .map(|txid: bitcoin::Txid| txid.to_byte_array())
        .collect();
    let header = Header {
        version: block::Version::from_consensus(template.version),
        prev_blockhash: template.previousblockhash.into(),
        merkle_root: TxMerkleNode::from_byte_array(merkle_root(txids).expect("the coinbase's")),
        time: *template.curtime,
        bits: CompactTarget::from_consensus(
            u32::from_str_radix(&template.bits, 16).expect("the template's own hex"),
        ),
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
