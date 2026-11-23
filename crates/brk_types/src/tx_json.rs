//! Transactions' JSON (Esplora's and mempool.space's shape), written straight into bytes: serde
//! went through a formatter call and an escape scan for every hex and assembly string, most of an
//! address or block page's CPU. Every string here is hex, assembly, an address or a type name,
//! all ASCII with nothing to escape.

use bitcoin::Script;

use crate::{
    Address, OutputType, Transaction, TxIn, TxOut,
    bytes::{push_asm, push_hex, reversed_hex},
    tx::p2sh_redeem_script,
};

/// A JSON array of `txs`.
pub fn txs_json<'a>(txs: impl IntoIterator<Item = &'a Transaction>) -> Vec<u8> {
    let mut out = vec![b'['];
    for (n, tx) in txs.into_iter().enumerate() {
        if n > 0 {
            out.push(b',');
        }
        tx.write_json(&mut out);
    }
    out.push(b']');
    out
}

impl Transaction {
    pub fn to_json(&self) -> Vec<u8> {
        let mut out = Vec::new();
        self.write_json(&mut out);
        out
    }

    pub fn write_json(&self, out: &mut Vec<u8>) {
        // An estimate, to spare most regrowths: hex doubles each byte and a script's assembly
        // repeats its hex; each input adds its keys and prevout (not in the transaction's bytes),
        // each output its keys and address.
        out.reserve(self.total_size * 4 + self.input.len() * 512 + self.output.len() * 192 + 256);
        out.extend_from_slice(b"{\"index\":");
        match self.index {
            Some(index) => int(out, *index),
            None => out.extend_from_slice(b"null"),
        }
        out.extend_from_slice(b",\"txid\":\"");
        out.extend_from_slice(&reversed_hex(&self.txid));
        out.extend_from_slice(b"\",\"version\":");
        int(out, *self.version);
        out.extend_from_slice(b",\"locktime\":");
        int(out, u32::from(self.lock_time));
        out.extend_from_slice(b",\"vin\":[");
        for (n, input) in self.input.iter().enumerate() {
            if n > 0 {
                out.push(b',');
            }
            input.write_json(out);
        }
        out.extend_from_slice(b"],\"vout\":[");
        for (n, output) in self.output.iter().enumerate() {
            if n > 0 {
                out.push(b',');
            }
            output.write_json(out);
        }
        out.extend_from_slice(b"],\"size\":");
        int(out, self.total_size);
        out.extend_from_slice(b",\"weight\":");
        int(out, *self.weight);
        out.extend_from_slice(b",\"sigops\":");
        int(out, *self.total_sigop_cost);
        out.extend_from_slice(b",\"fee\":");
        int(out, *self.fee);
        out.extend_from_slice(b",\"status\":");
        serde_json::to_writer(&mut *out, &self.status).expect("a status serializes");
        out.push(b'}');
    }
}

impl TxIn {
    fn write_json(&self, out: &mut Vec<u8>) {
        let prevout_script = self.prevout.as_ref().map(|p| p.script_pubkey.as_script());
        let is_p2sh = !self.is_coinbase && prevout_script.is_some_and(Script::is_p2sh);
        let redeem_script = is_p2sh
            .then(|| p2sh_redeem_script(&self.script_sig))
            .flatten();
        let inner_witness = prevout_script.and_then(|spk| {
            if spk.is_p2wsh() || redeem_script.is_some_and(Script::is_p2wsh) {
                self.witness.witness_script()
            } else if spk.is_p2tr() {
                self.witness.taproot_leaf_script().map(|leaf| leaf.script)
            } else {
                None
            }
        });

        out.extend_from_slice(b"{\"txid\":\"");
        out.extend_from_slice(&reversed_hex(&self.txid));
        out.extend_from_slice(b"\",\"vout\":");
        int(out, u32::from(self.vout));
        out.extend_from_slice(b",\"prevout\":");
        match &self.prevout {
            Some(prevout) => prevout.write_json(out),
            None => out.extend_from_slice(b"null"),
        }
        out.extend_from_slice(b",\"scriptsig\":\"");
        push_hex(self.script_sig.as_bytes(), out);
        out.extend_from_slice(b"\",\"scriptsig_asm\":\"");
        push_asm(&self.script_sig, out);
        out.push(b'"');
        if !self.witness.is_empty() {
            out.extend_from_slice(b",\"witness\":[");
            for (n, item) in self.witness.iter().enumerate() {
                out.extend_from_slice(if n > 0 { b",\"" } else { b"\"" });
                push_hex(item, out);
                out.push(b'"');
            }
            out.push(b']');
        }
        out.extend_from_slice(if self.is_coinbase {
            b",\"is_coinbase\":true,\"sequence\":"
        } else {
            b",\"is_coinbase\":false,\"sequence\":"
        });
        int(out, self.sequence);
        if is_p2sh {
            out.extend_from_slice(b",\"inner_redeemscript_asm\":\"");
            push_asm(redeem_script.unwrap_or(Script::new()), out);
            out.push(b'"');
        }
        if let Some(script) = inner_witness {
            out.extend_from_slice(b",\"inner_witnessscript_asm\":\"");
            push_asm(script, out);
            out.push(b'"');
        }
        out.push(b'}');
    }
}

impl TxOut {
    fn write_json(&self, out: &mut Vec<u8>) {
        let output_type = self.output_type();
        out.extend_from_slice(b"{\"scriptpubkey\":\"");
        push_hex(self.script_pubkey.as_bytes(), out);
        out.extend_from_slice(b"\",\"scriptpubkey_asm\":\"");
        push_asm(&self.script_pubkey, out);
        out.extend_from_slice(b"\",\"scriptpubkey_type\":\"");
        out.extend_from_slice(output_type.normalized().as_str().as_bytes());
        out.push(b'"');
        // As Esplora, P2PK has no address here; the pubkey hex stands for one only in the
        // address endpoints.
        if !matches!(output_type, OutputType::P2PK65 | OutputType::P2PK33)
            && let Ok(addr) = Address::try_from((&self.script_pubkey, output_type))
        {
            out.extend_from_slice(b",\"scriptpubkey_address\":\"");
            out.extend_from_slice(addr.as_bytes());
            out.push(b'"');
        }
        out.extend_from_slice(b",\"value\":");
        int(out, *self.value);
        out.push(b'}');
    }
}

#[inline]
fn int(out: &mut Vec<u8>, value: impl itoa::Integer) {
    out.extend_from_slice(itoa::Buffer::new().format(value).as_bytes());
}
