use std::cell::OnceCell;

use bitcoin::{
    Address, Block, Network, ScriptBuf, Transaction, TxIn, TxOut, consensus::encode::serialize_hex,
    hex::DisplayHex,
};
use brk_error::Error;
use brk_types::ReadBlock;
use serde_json::{Map, Value, json};

use crate::path::{Path, Step};

// `hex` is intentionally absent: matches `bitcoin-cli getblock <hash> 2`
// and keeps NDJSON dumps tractable. Still reachable explicitly via `blk N hex`.
const BLOCK_FIELDS: &[&str] = &[
    "height",
    "hash",
    "version",
    "version_hex",
    "merkle",
    "time",
    "nonce",
    "bits",
    "difficulty",
    "prev",
    "txs",
    "n_inputs",
    "n_outputs",
    "witness_txs",
    "size",
    "strippedsize",
    "weight",
    "subsidy",
    "coinbase",
    "coinbase_hex",
    "header_hex",
    "tx",
];

const TX_FIELDS: &[&str] = &[
    "txid",
    "wtxid",
    "version",
    "locktime",
    "size",
    "base_size",
    "vsize",
    "weight",
    "inputs",
    "outputs",
    "is_coinbase",
    "has_witness",
    "is_rbf",
    "total_out",
    "hex",
    "vin",
    "vout",
];

const VIN_FIELDS: &[&str] = &[
    "prev_txid",
    "prev_vout",
    "sequence",
    "script_sig",
    "script_sig_asm",
    "witness",
    "has_witness",
    "is_rbf",
    "coinbase",
];

const VOUT_FIELDS: &[&str] = &[
    "value",
    "script_pubkey",
    "script_pubkey_asm",
    "type",
    "address",
];

pub struct Ctx<'a> {
    block: &'a ReadBlock,
    network: Network,
    size_weight: OnceCell<(usize, usize)>,
}

impl<'a> Ctx<'a> {
    pub fn new(block: &'a ReadBlock, network: Network) -> Self {
        Self {
            block,
            network,
            size_weight: OnceCell::new(),
        }
    }

    pub fn resolve(&self, path: &Path) -> brk_error::Result<Value> {
        let (step, rest) = pop(&path.steps)?;
        self.block_field(&step.name, step.index, rest)
    }

    pub fn resolve_str(&self, path: &Path) -> brk_error::Result<String> {
        Ok(match self.resolve(path)? {
            Value::String(s) => s,
            other => other.to_string(),
        })
    }

    pub fn full(&self) -> Value {
        let mut obj = Map::with_capacity(BLOCK_FIELDS.len());
        for &name in BLOCK_FIELDS {
            obj.insert(
                name.into(),
                self.block_field(name, None, &[])
                    .expect("known block field"),
            );
        }
        Value::Object(obj)
    }

    fn size_and_weight(&self) -> (usize, usize) {
        *self
            .size_weight
            .get_or_init(|| self.block.total_size_and_weight())
    }

    fn block_field(
        &self,
        name: &str,
        index: Option<usize>,
        rest: &[Step],
    ) -> brk_error::Result<Value> {
        let b = self.block;
        let raw: &Block = b;
        let scalar = |v| scalar_leaf(v, name, index, rest);
        match name {
            "height" => scalar(json!(*b.height())),
            "hash" => scalar(json!(b.hash().to_string())),
            "time" => scalar(json!(b.header.time)),
            "version" => scalar(json!(b.header.version.to_consensus())),
            "version_hex" => scalar(json!(format!(
                "{:08x}",
                b.header.version.to_consensus() as u32
            ))),
            "bits" => scalar(json!(format!("{:08x}", b.header.bits.to_consensus()))),
            "nonce" => scalar(json!(b.header.nonce)),
            "prev" => scalar(json!(b.header.prev_blockhash.to_string())),
            "merkle" => scalar(json!(b.header.merkle_root.to_string())),
            "difficulty" => scalar(json!(b.header.difficulty_float())),
            "txs" => scalar(json!(b.txdata.len())),
            "n_inputs" => scalar(json!(
                b.txdata.iter().map(|tx| tx.input.len()).sum::<usize>()
            )),
            "n_outputs" => scalar(json!(
                b.txdata.iter().map(|tx| tx.output.len()).sum::<usize>()
            )),
            "witness_txs" => scalar(json!(
                b.txdata.iter().filter(|tx| tx_has_witness(tx)).count()
            )),
            "size" => scalar(json!(self.size_and_weight().0)),
            "weight" => scalar(json!(self.size_and_weight().1)),
            "strippedsize" => {
                let (size, weight) = self.size_and_weight();
                scalar(json!((weight - size) / 3))
            }
            "subsidy" => scalar(json!(subsidy_sats(*b.height()))),
            "header_hex" => scalar(json!(serialize_hex(&b.header))),
            "hex" => scalar(json!(serialize_hex(raw))),
            "coinbase" => scalar(json!(b.coinbase_tag().as_str())),
            "coinbase_hex" => {
                debug_assert!(
                    !b.txdata.is_empty() && !b.txdata[0].input.is_empty(),
                    "consensus-valid block has a coinbase tx with at least one input"
                );
                scalar(json!(b.txdata[0].input[0].script_sig.to_hex_string()))
            }
            "tx" => pick(&b.txdata, name, index, |i, tx| {
                self.resolve_tx(tx, i == 0, rest)
            }),
            other => Err(unknown("block", other)),
        }
    }

    fn resolve_tx(
        &self,
        tx: &Transaction,
        is_coinbase: bool,
        steps: &[Step],
    ) -> brk_error::Result<Value> {
        if steps.is_empty() {
            let mut obj = Map::with_capacity(TX_FIELDS.len());
            for &name in TX_FIELDS {
                obj.insert(
                    name.into(),
                    self.tx_field(tx, is_coinbase, name, None, &[])
                        .expect("known tx field"),
                );
            }
            return Ok(Value::Object(obj));
        }
        let (step, rest) = pop(steps)?;
        self.tx_field(tx, is_coinbase, &step.name, step.index, rest)
    }

    fn tx_field(
        &self,
        tx: &Transaction,
        is_coinbase: bool,
        name: &str,
        index: Option<usize>,
        rest: &[Step],
    ) -> brk_error::Result<Value> {
        let scalar = |v| scalar_leaf(v, name, index, rest);
        match name {
            "txid" => scalar(json!(tx.compute_txid().to_string())),
            "wtxid" => scalar(json!(tx.compute_wtxid().to_string())),
            "version" => scalar(json!(tx.version.0)),
            "locktime" => scalar(json!(tx.lock_time.to_consensus_u32())),
            "size" => scalar(json!(tx.total_size())),
            "base_size" => scalar(json!(tx.base_size())),
            "vsize" => scalar(json!(tx.vsize())),
            "weight" => scalar(json!(tx.weight().to_wu())),
            "inputs" => scalar(json!(tx.input.len())),
            "outputs" => scalar(json!(tx.output.len())),
            "is_coinbase" => scalar(json!(is_coinbase)),
            "has_witness" => scalar(json!(tx_has_witness(tx))),
            "is_rbf" => scalar(json!(tx_is_rbf(tx))),
            "total_out" => scalar(json!(tx_total_out(tx))),
            "hex" => scalar(json!(serialize_hex(tx))),
            "vin" => pick(&tx.input, name, index, |j, vin| {
                resolve_vin(vin, is_coinbase && j == 0, rest)
            }),
            "vout" => pick(&tx.output, name, index, |_, vout| {
                self.resolve_vout(vout, rest)
            }),
            other => Err(unknown("tx", other)),
        }
    }

    fn resolve_vout(&self, vout: &TxOut, steps: &[Step]) -> brk_error::Result<Value> {
        if steps.is_empty() {
            let mut obj = Map::with_capacity(VOUT_FIELDS.len());
            for &name in VOUT_FIELDS {
                obj.insert(
                    name.into(),
                    self.vout_field(vout, name, None, &[])
                        .expect("known vout field"),
                );
            }
            return Ok(Value::Object(obj));
        }
        let (step, rest) = pop(steps)?;
        self.vout_field(vout, &step.name, step.index, rest)
    }

    fn vout_field(
        &self,
        vout: &TxOut,
        name: &str,
        index: Option<usize>,
        rest: &[Step],
    ) -> brk_error::Result<Value> {
        let scalar = |v| scalar_leaf(v, name, index, rest);
        match name {
            "value" => scalar(json!(vout.value.to_sat())),
            "script_pubkey" => scalar(json!(vout.script_pubkey.to_hex_string())),
            "script_pubkey_asm" => scalar(json!(vout.script_pubkey.to_asm_string())),
            "type" => scalar(json!(script_type(&vout.script_pubkey))),
            "address" => scalar(self.address_value(&vout.script_pubkey)),
            other => Err(unknown("vout", other)),
        }
    }

    fn address_value(&self, s: &ScriptBuf) -> Value {
        Address::from_script(s, self.network)
            .map(|a| Value::String(a.to_string()))
            .unwrap_or(Value::Null)
    }
}

fn resolve_vin(vin: &TxIn, is_coinbase: bool, steps: &[Step]) -> brk_error::Result<Value> {
    if steps.is_empty() {
        let mut obj = Map::with_capacity(VIN_FIELDS.len());
        for &name in VIN_FIELDS {
            obj.insert(
                name.into(),
                vin_field(vin, is_coinbase, name, None, &[]).expect("known vin field"),
            );
        }
        return Ok(Value::Object(obj));
    }
    let (step, rest) = pop(steps)?;
    vin_field(vin, is_coinbase, &step.name, step.index, rest)
}

fn vin_field(
    vin: &TxIn,
    is_coinbase: bool,
    name: &str,
    index: Option<usize>,
    rest: &[Step],
) -> brk_error::Result<Value> {
    let scalar = |v| scalar_leaf(v, name, index, rest);
    match name {
        "prev_txid" => scalar(json!(vin.previous_output.txid.to_string())),
        "prev_vout" => scalar(json!(vin.previous_output.vout)),
        "sequence" => scalar(json!(vin.sequence.0)),
        "script_sig" => scalar(json!(vin.script_sig.to_hex_string())),
        "script_sig_asm" => scalar(json!(vin.script_sig.to_asm_string())),
        "witness" => {
            if !rest.is_empty() {
                return Err(Error::Parse(
                    "'witness' element has no fields to drill into".into(),
                ));
            }
            let items: Vec<String> = vin
                .witness
                .iter()
                .map(|w| w.to_lower_hex_string())
                .collect();
            pick(&items, name, index, |_, hex| Ok(Value::String(hex.clone())))
        }
        "has_witness" => scalar(json!(!vin.witness.is_empty())),
        "is_rbf" => scalar(json!(vin.sequence.is_rbf())),
        "coinbase" => scalar(json!(is_coinbase)),
        other => Err(unknown("vin", other)),
    }
}

fn pick<T>(
    items: &[T],
    name: &str,
    index: Option<usize>,
    mut resolve: impl FnMut(usize, &T) -> brk_error::Result<Value>,
) -> brk_error::Result<Value> {
    match index {
        Some(i) => {
            let item = items
                .get(i)
                .ok_or_else(|| out_of_range(name, i, items.len()))?;
            resolve(i, item)
        }
        None => Ok(Value::Array(
            items
                .iter()
                .enumerate()
                .map(|(i, item)| resolve(i, item))
                .collect::<brk_error::Result<_>>()?,
        )),
    }
}

fn pop(steps: &[Step]) -> brk_error::Result<(&Step, &[Step])> {
    steps
        .split_first()
        .ok_or_else(|| Error::Parse("empty path segment".into()))
}

fn scalar_leaf(
    v: Value,
    name: &str,
    index: Option<usize>,
    rest: &[Step],
) -> brk_error::Result<Value> {
    if index.is_some() {
        return Err(Error::Parse(format!("'{name}' is not an array")));
    }
    if !rest.is_empty() {
        return Err(Error::Parse(format!(
            "'{name}' has no fields to drill into"
        )));
    }
    Ok(v)
}

fn out_of_range(name: &str, i: usize, len: usize) -> Error {
    Error::Parse(format!("{name}.{i} out of range (len {len})"))
}

fn unknown(level: &str, name: &str) -> Error {
    Error::Parse(format!(
        "unknown {level} field '{name}' (run `blk --help` for the list)"
    ))
}

fn tx_has_witness(tx: &Transaction) -> bool {
    tx.input.iter().any(|i| !i.witness.is_empty())
}

fn tx_is_rbf(tx: &Transaction) -> bool {
    tx.input.iter().any(|i| i.sequence.is_rbf())
}

fn tx_total_out(tx: &Transaction) -> u64 {
    tx.output.iter().map(|o| o.value.to_sat()).sum()
}

fn subsidy_sats(height: u32) -> u64 {
    let halvings = height / 210_000;
    if halvings >= 64 {
        0
    } else {
        (50 * 100_000_000u64) >> halvings
    }
}

fn script_type(s: &ScriptBuf) -> &'static str {
    if s.is_p2pkh() {
        "p2pkh"
    } else if s.is_p2sh() {
        "p2sh"
    } else if s.is_p2wpkh() {
        "p2wpkh"
    } else if s.is_p2wsh() {
        "p2wsh"
    } else if s.is_p2tr() {
        "p2tr"
    } else if s.is_op_return() {
        "op_return"
    } else if s.is_p2pk() {
        "p2pk"
    } else {
        "unknown"
    }
}
