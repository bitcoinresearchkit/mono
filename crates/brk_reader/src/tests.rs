use std::{
    env,
    fs::{self, File},
    process,
};

use crate::{
    XORBytes,
    scan::{HEAD_LEN, MAGIC_BYTES, MAX_HOLE, frames},
};

/// Every frame, eagerly.
fn all_frames(file: &File, xor_bytes: XORBytes) -> Vec<(usize, usize)> {
    frames(file, xor_bytes, 0).unwrap().collect::<std::io::Result<_>>().unwrap()
}

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
    assert_eq!(all_frames(&file, XORBytes::from(KEY)), expected);
}

#[test]
fn finds_a_header_its_key_made_start_with_zeros() {
    use Part::*;
    // The header after the hole starts at 108 + 1000 = 1108, phase 4: a key byte equal to the
    // magic's first makes it zero on disk, so the zeros seem to end a byte later.
    let mut key = KEY;
    key[1108 % 8] = MAGIC_BYTES[0];
    let (file, expected) = blk("zeroed-head", key, &[Frame(100), Zeros(1000), Frame(50)]);
    assert_eq!(all_frames(&file, XORBytes::from(key)), expected);
}

#[test]
fn stops_at_the_preallocated_tail() {
    use Part::*;
    let (file, expected) = blk("tail", KEY, &[Frame(100), Frame(30), Zeros(1 << 20)]);
    assert_eq!(all_frames(&file, XORBytes::from(KEY)), expected);
    // Zeros longer than any frame aren't a hole, whatever follows them.
    let (file, expected) = blk(
        "long-zeros",
        KEY,
        &[Frame(100), Zeros(MAX_HOLE + 16), Frame(50)],
    );
    assert_eq!(
        all_frames(&file, XORBytes::from(KEY)),
        expected[..1]
    );
}

#[test]
fn steps_over_a_hole_in_an_unobfuscated_file() {
    use Part::*;
    let (file, expected) = blk("plain", [0; 8], &[Frame(100), Zeros(500), Frame(50)]);
    assert_eq!(all_frames(&file, XORBytes::default()), expected);
}
