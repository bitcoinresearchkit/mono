// Copyright (c) 2024-present, fjall-rs
// This source code is licensed under both the Apache 2.0 and MIT License
// (found in the LICENSE-* files in the repository)

const BIT_MASK: u8 = 0b1000_0000_u8;

/// Gets a bit from the byte
fn get_bit(byte: u8, idx: usize) -> bool {
    let bit_mask = BIT_MASK >> idx;

    let masked = byte & bit_mask;
    masked > 0
}

/// Fixed-size bit array reader
#[derive(Debug)]
pub struct BitArrayReader<'a>(&'a [u8]);

impl<'a> BitArrayReader<'a> {
    #[must_use]
    pub fn new(bytes: &'a [u8]) -> Self {
        Self(bytes)
    }

    /// Gets the i-th bit without checking the byte index.
    ///
    /// # Safety
    ///
    /// `idx` must be less than the number of bits in this array.
    #[must_use]
    pub unsafe fn get_unchecked(&self, idx: usize) -> bool {
        let byte_idx = idx / 8;
        debug_assert!(byte_idx < self.0.len());
        let byte = unsafe { self.0.get_unchecked(byte_idx) };

        let bit_idx = idx % 8;
        get_bit(*byte, bit_idx)
    }
}
