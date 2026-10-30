use std::{fs::File, io, ops::ControlFlow, os::unix::fs::FileExt};

use brk_types::BlkPosition;

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
const HEAD_LEN: usize = 8;
/// The longest run of unwritten bytes taken for a hole rather than the file's end: one frame of
/// the largest block (4,000,000 bytes serialized), all Core reserves for a block it then fails to
/// write.
const MAX_HOLE: usize = HEAD_LEN + 4_000_000;

/// Each block's payload offset and length in a blk file, hopping from frame to frame from
/// `start`, a frame boundary (the file's start or a known frame's end): a scan from an arbitrary
/// offset could take magic bytes inside a transaction for a frame. A hole of unwritten bytes,
/// where Core reserved a block's frame and failed to write it (its disk gone, say), is stepped
/// over to the frames Core wrote after it. Stops at the first other gap: the preallocated zero
/// tail, or the file's end once Core trims that tail on rollover.
pub fn frames(file: &File, xor_bytes: XORBytes, start: usize) -> io::Result<Vec<(usize, usize)>> {
    let file_len = file.metadata()?.len() as usize;
    let mut frames = Vec::new();
    let mut offset = start;
    loop {
        if let Some(len) = frame_at(file, xor_bytes, offset, file_len)? {
            frames.push((offset + HEAD_LEN, len));
            offset += HEAD_LEN + len;
        } else if let Some(next) = after_hole(file, xor_bytes, offset, file_len)? {
            offset = next;
        } else {
            break;
        }
    }
    Ok(frames)
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

#[cfg(test)]
mod tests {
    use std::{env, fs, process};

    use super::*;

    /// A blk file's parts, in order: a frame with this long a payload, or this many unwritten bytes.
    enum Part {
        Frame(usize),
        Zeros(usize),
    }

    /// The file the parts make, obfuscated with `key` (unwritten bytes stay zero on disk), and
    /// the payload offset and length `frames` should find for each frame.
    fn blk(name: &str, key: [u8; 8], parts: &[Part]) -> (File, Vec<(usize, usize)>) {
        let mut bytes = Vec::new();
        let mut expected = Vec::new();
        for part in parts {
            match *part {
                Part::Frame(len) => {
                    let start = bytes.len();
                    bytes.extend(MAGIC_BYTES);
                    bytes.extend((len as u32).to_le_bytes());
                    bytes.extend((0..len).map(|i| (i * 7 + 1) as u8));
                    for (i, byte) in bytes[start..].iter_mut().enumerate() {
                        *byte ^= key[(start + i) % 8];
                    }
                    expected.push((start + HEAD_LEN, len));
                }
                Part::Zeros(len) => bytes.resize(bytes.len() + len, 0),
            }
        }
        let path = env::temp_dir().join(format!("brk-reader-scan-{}-{name}", process::id()));
        fs::write(&path, &bytes).unwrap();
        let file = File::open(&path).unwrap();
        fs::remove_file(&path).unwrap();
        (file, expected)
    }

    const KEY: [u8; 8] = [0x0b, 0x84, 0x8d, 0x8a, 0x9a, 0xdc, 0x43, 0x89];

    #[test]
    fn steps_over_a_hole_to_the_frames_after_it() {
        use Part::*;
        let (file, expected) = blk(
            "hole",
            KEY,
            &[
                Frame(100),
                Zeros(1_331_259),
                Frame(50),
                Frame(70),
                Zeros(4096),
            ],
        );
        assert_eq!(frames(&file, XORBytes::from(KEY), 0).unwrap(), expected);
    }

    #[test]
    fn finds_a_header_its_key_made_start_with_zeros() {
        use Part::*;
        // The header after the hole starts at 108 + 1000 = 1108, phase 4: a key byte equal to the
        // magic's first makes it zero on disk, so the zeros seem to end a byte later.
        let mut key = KEY;
        key[1108 % 8] = MAGIC_BYTES[0];
        let (file, expected) = blk("zeroed-head", key, &[Frame(100), Zeros(1000), Frame(50)]);
        assert_eq!(frames(&file, XORBytes::from(key), 0).unwrap(), expected);
    }

    #[test]
    fn stops_at_the_preallocated_tail() {
        use Part::*;
        let (file, expected) = blk("tail", KEY, &[Frame(100), Frame(30), Zeros(1 << 20)]);
        assert_eq!(frames(&file, XORBytes::from(KEY), 0).unwrap(), expected);
        // Zeros longer than any frame aren't a hole, whatever follows them.
        let (file, expected) = blk(
            "long-zeros",
            KEY,
            &[Frame(100), Zeros(MAX_HOLE + 16), Frame(50)],
        );
        assert_eq!(
            frames(&file, XORBytes::from(KEY), 0).unwrap(),
            expected[..1]
        );
    }

    #[test]
    fn steps_over_a_hole_in_an_unobfuscated_file() {
        use Part::*;
        let (file, expected) = blk("plain", [0; 8], &[Frame(100), Zeros(500), Frame(50)]);
        assert_eq!(frames(&file, XORBytes::default(), 0).unwrap(), expected);
    }
}
