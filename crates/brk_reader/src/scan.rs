use std::{fs::File, io, ops::ControlFlow, os::unix::fs::FileExt};

use brk_types::BlkPosition;

use crate::{XORBytes, XORIndex, xor_bytes::XOR_LEN};

pub(crate) const MAGIC_BYTES: [u8; 4] = [0xF9, 0xBE, 0xB4, 0xD9];

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
    mut on_block: impl FnMut(BlkPosition, &mut [u8], XORIndex) -> ControlFlow<()>,
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
        let position = BlkPosition::new(blk_index, i as u32);
        if on_block(position, &mut buf[i..i + len], xor_i).is_break() {
            break;
        }
        i += len;
        xor_i.add_assign(len);
    }
}

/// A frame's header: the magic, then the payload's length.
pub(crate) const HEAD_LEN: usize = 8;
/// The longest run of unwritten bytes taken for a hole rather than the file's end: one frame of
/// the largest block (4,000,000 bytes serialized), all Core reserves for a block it then fails to
/// write.
pub(crate) const MAX_HOLE: usize = HEAD_LEN + 4_000_000;

/// Each block's payload offset and length in a blk file, hopping from frame to frame from
/// `start`, a frame boundary (the file's start or a known frame's end): a scan from an arbitrary
/// offset could take magic bytes inside a transaction for a frame. A hole of unwritten bytes,
/// where Core reserved a block's frame and failed to write it (its disk gone, say), is stepped
/// over to the frames Core wrote after it. Stops at the first other gap: the preallocated zero
/// tail, or the file's end once Core trims that tail on rollover. Lazy: a caller that stops at the
/// blocks it wants reads no further (the tail's check reads up to a frame's worth of zeros).
pub fn frames(file: &File, xor_bytes: XORBytes, start: usize) -> io::Result<Frames<'_>> {
    Ok(Frames {
        file,
        xor_bytes,
        offset: start,
        file_len: file.metadata()?.len() as usize,
        done: false,
    })
}

/// See [`frames`]. Ends after the first error.
pub struct Frames<'a> {
    file: &'a File,
    xor_bytes: XORBytes,
    offset: usize,
    file_len: usize,
    done: bool,
}

impl Frames<'_> {
    /// One hop: the frame at the offset, or over a hole (`None`, the offset moved), or the end
    /// (`None`, done).
    fn hop(&mut self) -> io::Result<Option<(usize, usize)>> {
        if let Some(len) = frame_at(self.file, self.xor_bytes, self.offset, self.file_len)? {
            let frame = (self.offset + HEAD_LEN, len);
            self.offset += HEAD_LEN + len;
            return Ok(Some(frame));
        }
        match after_hole(self.file, self.xor_bytes, self.offset, self.file_len)? {
            Some(next) => self.offset = next,
            None => self.done = true,
        }
        Ok(None)
    }
}

impl Iterator for Frames<'_> {
    type Item = io::Result<(usize, usize)>;

    fn next(&mut self) -> Option<Self::Item> {
        while !self.done {
            match self.hop() {
                Ok(Some(frame)) => return Some(Ok(frame)),
                Ok(None) => {}
                Err(e) => {
                    self.done = true;
                    return Some(Err(e));
                }
            }
        }
        None
    }
}

/// The payload length of the frame whose header is at `offset`, if one is, whole in the file.
fn frame_at(
    file: &File,
    xor_bytes: XORBytes,
    offset: usize,
    file_len: usize,
) -> io::Result<Option<usize>> {
    let mut head = [0u8; HEAD_LEN];
    if offset + HEAD_LEN > file_len {
        return Ok(None);
    }
    match file.read_exact_at(&mut head, offset as u64) {
        Err(e) if e.kind() == io::ErrorKind::UnexpectedEof => return Ok(None),
        result => result?,
    }
    XORIndex::at_offset(offset).bytes(&mut head, xor_bytes);
    let len = u32::from_le_bytes([head[4], head[5], head[6], head[7]]) as usize;
    Ok(
        (head[..4] == MAGIC_BYTES && len != 0 && offset + HEAD_LEN + len <= file_len)
            .then_some(len),
    )
}

/// Where the frame after a hole at `offset` begins: a run of unwritten bytes (zeros on disk,
/// whatever the obfuscation key), no longer than a frame, then a frame. Its header may begin up to
/// seven bytes before the zeros end, its first bytes zero on disk as the key made them. `None` if
/// `offset` isn't a hole: not zero, zeros to the end (the preallocated tail), or no frame after.
fn after_hole(
    file: &File,
    xor_bytes: XORBytes,
    offset: usize,
    file_len: usize,
) -> io::Result<Option<usize>> {
    let scan_len = (MAX_HOLE + HEAD_LEN).min(file_len.saturating_sub(offset));
    let mut bytes = vec![0u8; scan_len];
    file.read_exact_at(&mut bytes, offset as u64)?;
    let Some(end) = bytes.iter().position(|&byte| byte != 0) else {
        return Ok(None);
    };
    if end == 0 {
        return Ok(None);
    }
    for start in end.saturating_sub(HEAD_LEN - 1).max(1)..=end {
        if frame_at(file, xor_bytes, offset + start, file_len)?.is_some() {
            return Ok(Some(offset + start));
        }
    }
    Ok(None)
}
