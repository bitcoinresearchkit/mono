use bitcoin::{Transaction, Weight as BitcoinWeight, consensus::deserialize};
use brk_error::{Error, OptionData, Result};
use brk_reader::Reader;
use brk_types::{BlkPosition, Lengths, TxIndex, Txid};
use vecdb::ReadableVec;

use crate::Query;

/// Caller retains publication exclusion across metadata selection and reading.
pub fn read_at(query: &Query, index: TxIndex, safe: Lengths) -> Result<(Vec<u8>, Transaction)> {
    if index >= safe.tx_index {
        return Err(Error::UnknownTxid);
    }
    let transactions = &query.indexer().vecs().transactions;
    let size = *transactions.total_size.collect_one(index).data()? as usize;
    let position = transactions.position.collect_one(index).data()?;
    let txid = transactions.txid.collect_one(index).data()?;
    read(query.reader(), position, size, txid)
}

/// Read one indexed transaction without trusting persisted allocation lengths
/// or accepting a different transaction at a stale/corrupt file position.
pub fn read(
    reader: &Reader,
    position: BlkPosition,
    size: usize,
    txid: Txid,
) -> Result<(Vec<u8>, Transaction)> {
    if size == 0 || size > BitcoinWeight::MAX_BLOCK.to_wu() as usize {
        return Err(Error::Internal(
            "Indexed transaction size exceeds block bounds",
        ));
    }
    let bytes = reader.read_raw_bytes(position, size)?;
    let transaction = decode(&bytes, txid)?;
    Ok((bytes, transaction))
}

fn decode(bytes: &[u8], txid: Txid) -> Result<Transaction> {
    let transaction: Transaction =
        deserialize(bytes).map_err(|_| Error::Internal("Invalid indexed transaction encoding"))?;
    if Txid::from(transaction.compute_txid()) != txid
        || transaction.weight() > BitcoinWeight::MAX_BLOCK
    {
        return Err(Error::Internal(
            "Indexed transaction identity or weight mismatch",
        ));
    }
    Ok(transaction)
}
