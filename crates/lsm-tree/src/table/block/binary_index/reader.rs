// Copyright (c) 2025-present, fjall-rs
// This source code is licensed under both the Apache 2.0 and MIT License
// (found in the LICENSE-* files in the repository)

use byteorder::{LittleEndian, ReadBytesExt};

pub struct Reader<'a> {
    bytes: &'a [u8],
    step_size: usize,
}

impl<'a> Reader<'a> {
    pub fn new(bytes: &'a [u8], offset: u32, len: u32, step_size: u8) -> Self {
        let offset = offset as usize;
        let len = len as usize;
        let step_size = step_size as usize;
        let size = len * step_size;
        let end = offset + size;

        Self {
            #[expect(
                clippy::indexing_slicing,
                reason = "we consider the caller to be trustworthy"
            )]
            bytes: &bytes[offset..end],
            step_size,
        }
    }

    pub fn len(&self) -> usize {
        self.bytes.len() / self.step_size
    }

    pub fn partition_point(&self, pred: impl Fn(usize) -> bool) -> usize {
        if self.step_size == 2 {
            self.bytes
                .as_chunks::<2>()
                .0
                .partition_point(|bytes| pred(usize::from(u16::from_le_bytes(*bytes))))
        } else {
            self.bytes
                .as_chunks::<4>()
                .0
                .partition_point(|bytes| pred(u32::from_le_bytes(*bytes) as usize))
        }
    }

    pub fn get(&self, idx: usize) -> usize {
        let offset = idx * self.step_size;

        #[expect(
            clippy::indexing_slicing,
            reason = "we consider the caller to be trustworthy"
        )]
        let mut bytes = &self.bytes[offset..];

        if self.step_size == 2 {
            unwrap!(bytes.read_u16::<LittleEndian>()).into()
        } else {
            unwrap!(bytes.read_u32::<LittleEndian>()) as usize
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Reader;

    #[test]
    fn partition_search_handles_both_offset_widths() {
        for (offsets, width) in [
            ([0_u32, 17, 31, 100, 65535], 2),
            ([0_u32, 17, 65536, 100_000, u32::MAX], 4),
        ] {
            // Include a prefix to exercise an index at a nonzero block offset.
            let mut bytes = vec![9; 3];
            for offset in offsets {
                bytes.extend_from_slice(&offset.to_le_bytes()[..usize::from(width)]);
            }
            let reader = Reader::new(&bytes, 3, 5, width);
            for needle in [0, 1, 17, 18, 31, 100, 65535, 65536, 100_001, u32::MAX] {
                let expected = offsets
                    .iter()
                    .take_while(|offset| **offset < needle)
                    .count();
                assert_eq!(
                    reader.partition_point(|offset| offset < needle as usize),
                    expected
                );
            }
            for (index, offset) in offsets.into_iter().enumerate() {
                assert_eq!(reader.get(index), offset as usize);
            }
        }
        for width in [2, 4] {
            assert_eq!(Reader::new(&[], 0, 0, width).partition_point(|_| true), 0);
        }
    }
}
