use std::{cell::OnceCell, ops::Deref};

use bitcoin::{
    Address, Block, Network, Transaction, TxIn, TxOut, Txid, consensus::encode::serialize_hex,
    hex::DisplayHex,
};
use brk_error::{Error, Result};
use brk_types::{OutputType, ReadBlock};
use serde_json::{Map, Value, json};

use crate::path::{Path, Step};

/// An object's leaf fields in output order, each with how it is read; nested objects (`tx`,
/// `vin`, `vout`) follow them. The help lists these names.
pub type Fields<F> = &'static [(&'static str, F)];

/// `hex` is left out of the whole block: matches `bitcoin-cli getblock <hash> 2` and keeps
/// NDJSON dumps tractable. Still reachable explicitly via `blk N hex`.
pub const BLOCK: Fields<fn(&Ctx) -> Value> = &[
    ("height", |c| json!(*c.height())),
    ("hash", |c| json!(c.hash().to_string())),
    ("version", |c| json!(c.header.version.to_consensus())),
    ("version_hex", |c| {
        json!(format!("{:08x}", c.header.version.to_consensus() as u32))
    }),
    ("merkle", |c| json!(c.header.merkle_root.to_string())),
    ("time", |c| json!(c.header.time)),
    ("nonce", |c| json!(c.header.nonce)),
    ("bits", |c| {
        json!(format!("{:08x}", c.header.bits.to_consensus()))
    }),
    ("difficulty", |c| json!(c.header.difficulty_float())),
    ("prev", |c| json!(c.header.prev_blockhash.to_string())),
    ("txs", |c| json!(c.txdata.len())),
    ("n_inputs", |c| {
        json!(c.txdata.iter().map(|tx| tx.input.len()).sum::<usize>())
    }),
    ("n_outputs", |c| {
        json!(c.txdata.iter().map(|tx| tx.output.len()).sum::<usize>())
    }),
    ("witness_txs", |c| {
        json!(c.txdata.iter().filter(|tx| tx_has_witness(tx)).count())
    }),
    ("size", |c| json!(c.size_and_weight().0)),
    ("strippedsize", |c| {
        let (size, weight) = c.size_and_weight();
        json!((weight - size) / 3)
    }),
    ("weight", |c| json!(c.size_and_weight().1)),
    ("subsidy", |c| json!(u64::from(c.height().subsidy()))),
    ("coinbase", |c| json!(c.coinbase_tag().to_string())),
    ("coinbase_hex", |c| {
        debug_assert!(
            !c.txdata.is_empty() && !c.txdata[0].input.is_empty(),
            "consensus-valid block has a coinbase tx with at least one input"
        );
        json!(c.txdata[0].input[0].script_sig.to_hex_string())
    }),
    ("header_hex", |c| json!(serialize_hex(&c.header))),
    ("hex", |c| json!(serialize_hex::<Block>(c))),
];

pub const TX: Fields<fn(&Tx) -> Value> = &[
    ("txid", |tx| json!(tx.id_and_sizes().0.to_string())),
    ("wtxid", |tx| json!(tx.compute_wtxid().to_string())),
    ("version", |tx| json!(tx.version.0)),
    ("locktime", |tx| json!(tx.lock_time.to_consensus_u32())),
    ("size", |tx| json!(tx.total_size())),
    ("base_size", |tx| json!(tx.base_size())),
    ("vsize", |tx| json!(tx.weight().div_ceil(4))),
    ("weight", |tx| json!(tx.weight())),
    ("inputs", |tx| json!(tx.input.len())),
    ("outputs", |tx| json!(tx.output.len())),
    ("is_coinbase", |tx| json!(tx.index == 0)),
    ("has_witness", |tx| json!(tx_has_witness(tx))),
    ("is_rbf", |tx| {
        json!(tx.input.iter().any(|i| i.sequence.is_rbf()))
    }),
    ("total_out", |tx| {
        json!(tx.output.iter().map(|o| o.value.to_sat()).sum::<u64>())
    }),
    ("hex", |tx| json!(serialize_hex(tx.tx))),
];

/// Read with whether the input is the coinbase input.
pub const VIN: Fields<fn(&TxIn, bool) -> Value> = &[
    ("prev_txid", |vin, _| {
        json!(vin.previous_output.txid.to_string())
    }),
    ("prev_vout", |vin, _| json!(vin.previous_output.vout)),
    ("sequence", |vin, _| json!(vin.sequence.0)),
    ("script_sig", |vin, _| json!(vin.script_sig.to_hex_string())),
    ("script_sig_asm", |vin, _| {
        json!(vin.script_sig.to_asm_string())
    }),
    ("witness", |vin, _| vin.witness.iter().map(hex).collect()),
    ("has_witness", |vin, _| json!(!vin.witness.is_empty())),
    ("is_rbf", |vin, _| json!(vin.sequence.is_rbf())),
    ("coinbase", |_, is_coinbase| json!(is_coinbase)),
];

pub const VOUT: Fields<fn(&TxOut, Network) -> Value> = &[
    ("value", |vout, _| json!(vout.value.to_sat())),
    ("script_pubkey", |vout, _| {
        json!(vout.script_pubkey.to_hex_string())
    }),
    ("script_pubkey_asm", |vout, _| {
        json!(vout.script_pubkey.to_asm_string())
    }),
    ("type", |vout, _| {
        json!(OutputType::from(&vout.script_pubkey))
    }),
    ("address", |vout, network| {
        Address::from_script(&vout.script_pubkey, network)
            .map_or(Value::Null, |a| Value::String(a.to_string()))
    }),
];

pub struct Ctx<'a> {
    block: &'a ReadBlock,
    network: Network,
    size_weight: OnceCell<(usize, usize)>,
}

impl Deref for Ctx<'_> {
    type Target = ReadBlock;

    fn deref(&self) -> &ReadBlock {
        self.block
    }
}

impl<'a> Ctx<'a> {
    pub fn new(block: &'a ReadBlock, network: Network) -> Self {
        Self {
            block,
            network,
            size_weight: OnceCell::new(),
        }
    }

    pub fn resolve(&self, path: &Path) -> Result<Value> {
        let (step, rest) = path
            .steps
            .split_first()
            .ok_or_else(|| Error::Parse("empty path segment".into()))?;
        self.block_field(&step.name, step.index, rest)
    }

    pub fn resolve_str(&self, path: &Path) -> Result<String> {
        Ok(match self.resolve(path)? {
            Value::String(s) => s,
            other => other.to_string(),
        })
    }

    pub fn full(&self) -> Value {
        // Every leaf but `hex`, then `tx`.
        let leaves = BLOCK.iter().filter(|&&(name, _)| name != "hex");
        let mut block = object(BLOCK.len(), leaves, |field| field(self));
        block.insert("tx".into(), self.txs(None, &[]).expect("whole txs"));
        Value::Object(block)
    }

    fn size_and_weight(&self) -> (usize, usize) {
        *self.size_weight.get_or_init(|| {
            (
                self.block.total_size(),
                self.block.weight().to_wu() as usize,
            )
        })
    }

    fn block_field(&self, name: &str, index: Option<usize>, rest: &[Step]) -> Result<Value> {
        match name {
            "tx" => self.txs(index, rest),
            _ => leaf(lookup(BLOCK, "block", name)?(self), name, index, rest),
        }
    }

    fn txs(&self, index: Option<usize>, rest: &[Step]) -> Result<Value> {
        pick(&self.block.txdata, "tx", index, |i, tx| {
            self.resolve_tx(&Tx::new(self.block, tx, i), rest)
        })
    }

    fn resolve_tx(&self, tx: &Tx, steps: &[Step]) -> Result<Value> {
        let Some((step, rest)) = steps.split_first() else {
            let mut whole = object(TX.len() + 2, TX, |field| field(tx));
            for name in ["vin", "vout"] {
                whole.insert(name.into(), self.tx_field(tx, name, None, &[])?);
            }
            return Ok(Value::Object(whole));
        };
        self.tx_field(tx, &step.name, step.index, rest)
    }

    fn tx_field(&self, tx: &Tx, name: &str, index: Option<usize>, rest: &[Step]) -> Result<Value> {
        match name {
            "vin" => pick(&tx.input, name, index, |j, vin| match rest.split_first() {
                Some((step, rest)) if step.name == "witness" => witness(vin, step.index, rest),
                _ => resolve_leaves(VIN, "vin", rest, |field| {
                    field(vin, tx.index == 0 && j == 0)
                }),
            }),
            "vout" => pick(&tx.output, name, index, |_, vout| {
                resolve_leaves(VOUT, "vout", rest, |field| field(vout, self.network))
            }),
            _ => leaf(lookup(TX, "tx", name)?(tx), name, index, rest),
        }
    }
}

/// A block transaction whose txid is computed at most once, from the raw block
/// bytes when available, with its sizes alongside; size-only queries skip the hash.
pub struct Tx<'a> {
    block: &'a ReadBlock,
    tx: &'a Transaction,
    index: usize,
    id_and_sizes: OnceCell<(Txid, u32, u32)>,
}

impl<'a> Tx<'a> {
    fn new(block: &'a ReadBlock, tx: &'a Transaction, index: usize) -> Self {
        Self {
            block,
            tx,
            index,
            id_and_sizes: OnceCell::new(),
        }
    }

    /// Txid, base size and total size.
    fn id_and_sizes(&self) -> (Txid, u32, u32) {
        *self
            .id_and_sizes
            .get_or_init(|| self.block.compute_tx_id_and_sizes(self.index))
    }

    fn base_size(&self) -> u32 {
        self.id_and_sizes.get().map_or_else(
            || self.tx.base_size() as u32,
            |&(_, base_size, _)| base_size,
        )
    }

    fn total_size(&self) -> u32 {
        self.id_and_sizes.get().map_or_else(
            || self.tx.total_size() as u32,
            |&(_, _, total_size)| total_size,
        )
    }

    fn weight(&self) -> u32 {
        3 * self.base_size() + self.total_size()
    }
}

impl Deref for Tx<'_> {
    type Target = Transaction;

    fn deref(&self) -> &Transaction {
        self.tx
    }
}

/// The whole object's leaf fields, in order, in a map reserved for `len` fields.
fn object<'a, F: Copy + 'a>(
    len: usize,
    fields: impl IntoIterator<Item = &'a (&'static str, F)>,
    read: impl Fn(F) -> Value,
) -> Map<String, Value> {
    let mut object = Map::with_capacity(len);
    object.extend(
        fields
            .into_iter()
            .map(|&(name, field)| (name.to_owned(), read(field))),
    );
    object
}

/// An object made of leaf fields only: whole, or the field the path names.
fn resolve_leaves<F: Copy>(
    fields: Fields<F>,
    level: &str,
    steps: &[Step],
    read: impl Fn(F) -> Value,
) -> Result<Value> {
    match steps.split_first() {
        None => Ok(Value::Object(object(fields.len(), fields, read))),
        Some((step, rest)) => leaf(
            read(lookup(fields, level, &step.name)?),
            &step.name,
            step.index,
            rest,
        ),
    }
}

fn lookup<F: Copy>(fields: Fields<F>, level: &str, name: &str) -> Result<F> {
    fields
        .iter()
        .find(|&&(field, _)| field == name)
        .map(|&(_, read)| read)
        .ok_or_else(|| {
            Error::Parse(format!(
                "unknown {level} field '{name}' (run `blk --help` for the list)"
            ))
        })
}

/// A scalar leaf: no index, no fields to drill into.
fn leaf(value: Value, name: &str, index: Option<usize>, rest: &[Step]) -> Result<Value> {
    if index.is_some() {
        return Err(Error::Parse(format!("'{name}' is not an array")));
    }
    if !rest.is_empty() {
        return Err(Error::Parse(format!(
            "'{name}' has no fields to drill into"
        )));
    }
    Ok(value)
}

/// The witness stack, or one element of it: only the elements returned are hex-encoded.
fn witness(vin: &TxIn, index: Option<usize>, rest: &[Step]) -> Result<Value> {
    if !rest.is_empty() {
        return Err(Error::Parse(
            "'witness' element has no fields to drill into".into(),
        ));
    }
    match index {
        Some(i) => vin
            .witness
            .nth(i)
            .map(hex)
            .ok_or_else(|| out_of_range("witness", i, vin.witness.len())),
        None => Ok(vin.witness.iter().map(hex).collect()),
    }
}

fn hex(bytes: &[u8]) -> Value {
    Value::String(bytes.to_lower_hex_string())
}

fn pick<T>(
    items: &[T],
    name: &str,
    index: Option<usize>,
    mut resolve: impl FnMut(usize, &T) -> Result<Value>,
) -> Result<Value> {
    match index {
        Some(i) => {
            let item = items
                .get(i)
                .ok_or_else(|| out_of_range(name, i, items.len()))?;
            resolve(i, item)
        }
        None => items
            .iter()
            .enumerate()
            .map(|(i, item)| resolve(i, item))
            .collect(),
    }
}

fn out_of_range(name: &str, i: usize, len: usize) -> Error {
    Error::Parse(format!("{name}.{i} out of range (len {len})"))
}

fn tx_has_witness(tx: &Transaction) -> bool {
    tx.input.iter().any(|i| !i.witness.is_empty())
}
