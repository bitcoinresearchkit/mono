use std::{fs::File, io, ops::ControlFlow, os::unix::fs::FileExt};

use brk_types::{BlkMetadata, BlkPosition};

use crate::{XORBytes, XORIndex, xor_bytes::XOR_LEN};

const MAGIC_BYTES: [u8; 4] = [0xF9, 0xBE, 0xB4, 0xD9];

/// Returns the position immediately after the matched magic, or
/// `None` if no match. Advances `xor_i` by the bytes consumed either
/// way.
pub fn find_magic(bytes: &[u8], xor_i: &mut XORIndex, xor_bytes: XORBytes) -> Option<usize> {
    let len = bytes.len();
    if len < MAGIC_BYTES.len() {
        xor_i.add_assign(len);
        return None;
    }

    let xb = *xor_bytes;
    let mut phase = xor_i.phase();
    let mut i = 0;
    let stop = len - MAGIC_BYTES.len();

    while i <= stop {
        if bytes[i] ^ xb[phase] == MAGIC_BYTES[0] {
            let p1 = (phase + 1) & (XOR_LEN - 1);
            let p2 = (phase + 2) & (XOR_LEN - 1);
            let p3 = (phase + 3) & (XOR_LEN - 1);
            if bytes[i + 1] ^ xb[p1] == MAGIC_BYTES[1]
                && bytes[i + 2] ^ xb[p2] == MAGIC_BYTES[2]
                && bytes[i + 3] ^ xb[p3] == MAGIC_BYTES[3]
            {
                xor_i.set_phase(phase + MAGIC_BYTES.len());
                return Some(i + MAGIC_BYTES.len());
            }
        }
        phase = (phase + 1) & (XOR_LEN - 1);
        i += 1;
    }

    xor_i.set_phase(phase + (len - i));
    None
}

/// Scans a whole blk file's bytes for blocks and calls `on_block` for each.
pub fn scan_bytes(
    buf: &mut [u8],
    blk_index: u16,
    xor_bytes: XORBytes,
    mut on_block: impl FnMut(BlkMetadata, &mut [u8], XORIndex) -> ControlFlow<()>,
) {
    let mut xor_i = XORIndex::default();
    let mut i = 0;

    while let Some(off) = find_magic(&buf[i..], &mut xor_i, xor_bytes) {
        i += off;
        if i + 4 > buf.len() {
            break;
        }
        let mut size_bytes = [buf[i], buf[i + 1], buf[i + 2], buf[i + 3]];
        xor_i.bytes(&mut size_bytes, xor_bytes);
        let len = u32::from_le_bytes(size_bytes) as usize;
        i += 4;
        if i + len > buf.len() {
            break;
        }
        let metadata = BlkMetadata::new(BlkPosition::new(blk_index, i as u32));
        if on_block(metadata, &mut buf[i..i + len], xor_i).is_break() {
            break;
        }
        i += len;
        xor_i.add_assign(len);
    }
}

/// Each block's payload offset and length in a blk file, hopping from frame to frame from
/// `start`, a frame boundary (the file's start or a known frame's end): a scan from an arbitrary
/// offset could take magic bytes inside a transaction for a frame. Stops at the first gap: the
/// preallocated zero tail, or the file's end once Core trims that tail on rollover.
pub fn frames(file: &File, xor_bytes: XORBytes, start: usize) -> io::Result<Vec<(usize, usize)>> {
    let file_len = file.metadata()?.len() as usize;
    let mut frames = Vec::new();
    let mut offset = start;
    let mut head = [0u8; 8];
    while offset + head.len() <= file_len {
        match file.read_exact_at(&mut head, offset as u64) {
            Err(e) if e.kind() == io::ErrorKind::UnexpectedEof => break,
            result => result?,
        }
        XORIndex::at_offset(offset).bytes(&mut head, xor_bytes);
        if head[..4] != MAGIC_BYTES {
            break;
        }
        let len = u32::from_le_bytes([head[4], head[5], head[6], head[7]]) as usize;
        let payload = offset + head.len();
        if len == 0 || payload + len > file_len {
            break;
        }
        frames.push((payload, len));
        offset = payload + len;
    }
    Ok(frames)
}
