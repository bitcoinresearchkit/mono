use bitcoin::Transaction as BitcoinTransaction;
use brk_types::{Sats, SigOps, Transaction, TxIn, TxOut, TxStatus, Txid, Vout, Weight};

/// A pending body with the caller's metadata and prevouts. Live ingestion supplies Core's
/// metadata and pooled prevouts; restore derives metadata and reads prevouts at its boundary.
pub(crate) fn pending_body(
    raw: BitcoinTransaction,
    txid: Txid,
    fee: Sats,
    weight: Weight,
    total_size: usize,
    mut prevout: impl FnMut(Txid, Vout) -> Option<TxOut>,
) -> Transaction {
    let input = raw
        .input
        .into_iter()
        .map(|txin| {
            let txid = Txid::from(txin.previous_output.txid);
            let vout = Vout::from(txin.previous_output.vout);
            TxIn {
                // Pending transactions are never coinbase.
                is_coinbase: false,
                prevout: prevout(txid, vout),
                txid,
                vout,
                script_sig: txin.script_sig,
                witness: txin.witness,
                sequence: txin.sequence.into(),
            }
        })
        .collect();
    let mut tx = Transaction {
        index: None,
        txid,
        version: raw.version.into(),
        total_sigop_cost: SigOps::ZERO,
        weight,
        lock_time: raw.lock_time.into(),
        total_size,
        fee,
        input,
        output: raw.output.into_iter().map(TxOut::from).collect(),
        status: TxStatus::UNCONFIRMED,
    };
    tx.refresh_sigops();
    tx
}
