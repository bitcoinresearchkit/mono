use std::{
    fs::File,
    io::{self, Cursor},
    os::unix::fs::FileExt,
};

use bitcoin::{Block as BitcoinBlock, Transaction, VarInt, block::Header, consensus::Decodable};
use brk_error::{Error, Result};
use brk_types::{BlkPosition, BlockHash, Height, ReadBlock};

use crate::{XORBytes, XORIndex};

const HEADER_LEN: usize = 80;

/// The smallest serialized transaction: version, one input (outpoint, empty script, sequence), one
/// output (value, empty script) and lock time.
const MIN_TX_LEN: usize = 60;

/// The header of the block in the frame at `(offset, len)`, `None` when the frame is too short to
/// hold one.
pub fn read_header(
    file: &File,
    (offset, len): (usize, usize),
    xor_bytes: XORBytes,
) -> io::Result<Option<Header>> {
    if len < HEADER_LEN {
        return Ok(None);
    }
    let mut bytes = [0u8; HEADER_LEN];
    file.read_exact_at(&mut bytes, offset as u64)?;
    XORIndex::at_offset(offset).bytes(&mut bytes, xor_bytes);
    Ok(Some(
        Header::consensus_decode_from_finite_reader(&mut &bytes[..])
            .expect("an 80-byte header always decodes"),
    ))
}

pub fn parse_canonical_body(
    mut bytes: Vec<u8>,
    position: BlkPosition,
    mut xor_state: XORIndex,
    xor_bytes: XORBytes,
    height: Height,
    header: Header,
) -> Result<ReadBlock> {
    debug_assert!(bytes.len() >= HEADER_LEN);
    xor_state.bytes(&mut bytes, xor_bytes);
    // Read again for its body: Core rewriting the file in place since (crash recovery) would put
    // another block's transactions under the header found first.
    let reread = Header::consensus_decode_from_finite_reader(&mut &bytes[..HEADER_LEN])
        .expect("an 80-byte header always decodes");
    if reread != header {
        return Err(Error::Internal("Block file changed while it was read"));
    }
    let bitcoin_hash = header.block_hash();
    let len = bytes.len();

    let mut cursor = Cursor::new(bytes);
    cursor.set_position(HEADER_LEN as u64);

    // `from_finite_reader` skips the `Take<R>` wrap that
    // `consensus_decode` applies to every nested field for memory
    // safety: our cursor is already a bounded `Vec<u8>`, so the
    // wrapping is pure overhead and compounds across ~2000 tx fields.
    let tx_count = VarInt::consensus_decode_from_finite_reader(&mut cursor)?.0 as usize;
    if tx_count == 0 {
        return Err(Error::Internal("Block without a coinbase"));
    }
    // A count the bytes can't hold (a corrupted file) fails decoding below, not allocating here.
    let capacity = tx_count.min(len / MIN_TX_LEN);
    let mut txdata = Vec::with_capacity(capacity);
    let mut tx_offsets = Vec::with_capacity(capacity);
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
