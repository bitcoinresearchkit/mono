// Copyright (c) 2024-present, fjall-rs
// This source code is licensed under both the Apache 2.0 and MIT License
// (found in the LICENSE-* files in the repository)

const BIT_MASK: u8 = 0b1000_0000_u8;

/// Sets a bit in the byte to `true`
#[must_use]
pub fn enable_bit(byte: u8, idx: usize) -> u8 {
    let bit_mask = BIT_MASK >> idx;
    byte | bit_mask
}

/// Fixed-size bit array
#[derive(Debug)]
pub struct Builder(Box<[u8]>);

impl Builder {
    #[must_use]
    pub fn with_capacity(bytes: usize) -> Self {
        let vec = vec![0; bytes];
        Self(vec.into_boxed_slice())
    }

    #[must_use]
    pub fn bytes(&self) -> &[u8] {
        &self.0
    }

    /// Sets the i-th bit
    pub fn enable_bit(&mut self, idx: usize) {
        let byte_idx = idx / 8;
        debug_assert!(byte_idx < self.0.len());

        #[expect(unsafe_code, reason = "the caller guarantees that idx is in bounds")]
        let byte = unsafe { self.0.get_unchecked_mut(byte_idx) };

        let bit_idx = idx % 8;
        *byte = enable_bit(*byte, bit_idx);
    }
}
