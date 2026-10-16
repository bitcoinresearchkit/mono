use std::io::Cursor;

use bitcoin::{Block as BitcoinBlock, Transaction, VarInt, block::Header, consensus::Decodable};
use brk_error::{Error, Result};
use brk_types::{BlkPosition, BlockHash, Height, ReadBlock};

use crate::{XORBytes, XORIndex, canonical::CanonicalRange};

pub const HEADER_LEN: usize = 80;

/// The canonical offset and header of the block in `bytes`, `None` for any other block. A block
/// of the range that does not follow its canonical predecessor is an error: the range was resolved
/// across a reorg. Decodes the header onto a stack buffer so `bytes` stays untouched: the body
/// parse later re-XORs the full block from the original phase.
pub fn peek_canonical(
    bytes: &[u8],
    mut xor_state: XORIndex,
    xor_bytes: XORBytes,
    canonical: &CanonicalRange,
) -> Result<Option<(u32, Header)>> {
    if bytes.len() < HEADER_LEN {
        return Ok(None);
    }
    let mut header_buf = [0u8; HEADER_LEN];
    header_buf.copy_from_slice(&bytes[..HEADER_LEN]);
    xor_state.bytes(&mut header_buf, xor_bytes);
    let Ok(header) = Header::consensus_decode_from_finite_reader(&mut &header_buf[..]) else {
        return Ok(None);
    };
    let Some(offset) = canonical.offset_of(&BlockHash::from(header.block_hash())) else {
        return Ok(None);
    };
    if !canonical.verify_prev(offset, &BlockHash::from(header.prev_blockhash)) {
        return Err(Error::Internal("canonical batch stitched across a reorg"));
    }
    Ok(Some((offset, header)))
}

pub fn parse_canonical_body(
    mut bytes: Vec<u8>,
    position: BlkPosition,
    mut xor_state: XORIndex,
    xor_bytes: XORBytes,
    height: Height,
    header: Header,
) -> Result<ReadBlock> {
    if bytes.len() < HEADER_LEN {
        return Err(Error::Internal("Block bytes shorter than header"));
    }

    xor_state.bytes(&mut bytes, xor_bytes);
    let bitcoin_hash = header.block_hash();

    let mut cursor = Cursor::new(bytes);
    cursor.set_position(HEADER_LEN as u64);

    // `from_finite_reader` skips the `Take<R>` wrap that
    // `consensus_decode` applies to every nested field for memory
    // safety: our cursor is already a bounded `Vec<u8>`, so the
    // wrapping is pure overhead and compounds across ~2000 tx fields.
    let tx_count = VarInt::consensus_decode_from_finite_reader(&mut cursor)?.0 as usize;
    let mut txdata = Vec::with_capacity(tx_count);
    let mut tx_offsets = Vec::with_capacity(tx_count);
    for _ in 0..tx_count {
        tx_offsets.push(cursor.position() as u32);
        let tx = Transaction::consensus_decode_from_finite_reader(&mut cursor)?;
        txdata.push(tx);
    }

    Ok(ReadBlock::new(
        height,
        BlockHash::from(bitcoin_hash),
        BitcoinBlock { header, txdata },
        position,
        cursor.into_inner(),
        tx_offsets,
    ))
}
