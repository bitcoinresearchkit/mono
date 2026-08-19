use std::io::Cursor;

use bitcoin::{Transaction, VarInt, block::Header, consensus::Decodable};
use brk_error::Error;
use brk_types::{BlkMetadata, Block, BlockHash, Height, ReadBlock};

use crate::{XORBytes, XORIndex, canonical::CanonicalRange};

pub(crate) const HEADER_LEN: usize = 80;

/// Decodes the header onto a stack buffer so `bytes` stays untouched:
/// the body parse later re-XORs the full block from the original phase.
pub(crate) fn peek_canonical(
    bytes: &[u8],
    mut xor_state: XORIndex,
    xor_bytes: XORBytes,
    canonical: &CanonicalRange,
) -> Option<(u32, Header)> {
    if bytes.len() < HEADER_LEN {
        return None;
    }
    let mut header_buf = [0u8; HEADER_LEN];
    header_buf.copy_from_slice(&bytes[..HEADER_LEN]);
    xor_state.bytes(&mut header_buf, xor_bytes);
    let header = Header::consensus_decode_from_finite_reader(&mut &header_buf[..]).ok()?;
    let offset = canonical.offset_of(&BlockHash::from(header.block_hash()))?;
    Some((offset, header))
}

pub(crate) fn parse_canonical_body(
    mut bytes: Vec<u8>,
    metadata: BlkMetadata,
    mut xor_state: XORIndex,
    xor_bytes: XORBytes,
    height: Height,
    header: Header,
) -> brk_error::Result<ReadBlock> {
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
    let mut tx_metadata = Vec::with_capacity(tx_count);
    let mut tx_offsets = Vec::with_capacity(tx_count);
    for _ in 0..tx_count {
        let tx_start = cursor.position() as u32;
        tx_offsets.push(tx_start);
        let tx = Transaction::consensus_decode_from_finite_reader(&mut cursor)?;
        let tx_len = cursor.position() as u32 - tx_start;
        txdata.push(tx);
        tx_metadata.push(BlkMetadata::new(metadata.position() + tx_start, tx_len));
    }

    let raw_bytes = cursor.into_inner();
    let mut block = Block::from((height, bitcoin_hash, bitcoin::Block { header, txdata }));
    block.set_raw_data(raw_bytes, tx_offsets);
    Ok(ReadBlock::from((block, metadata, tx_metadata)))
}
