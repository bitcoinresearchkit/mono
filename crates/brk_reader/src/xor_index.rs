use crate::xor_bytes::{XOR_LEN, XORBytes};

#[derive(Debug, Default, PartialEq, Eq, Clone, Copy)]
pub struct XORIndex(usize);

impl XORIndex {
    /// Phase-aligned `XORIndex` for a buffer that conceptually starts
    /// at `offset` in the blk file.
    #[inline]
    pub fn at_offset(offset: usize) -> Self {
        Self(offset & (XOR_LEN - 1))
    }

    #[inline]
    pub(crate) fn phase(self) -> usize {
        self.0
    }

    #[inline]
    pub(crate) fn set_phase(&mut self, phase: usize) {
        self.0 = phase & (XOR_LEN - 1);
    }

    #[inline]
    pub fn add_assign(&mut self, i: usize) {
        self.0 = (self.0 + i) & (XOR_LEN - 1);
    }

    /// XOR-decode `bytes` in place, advancing the phase. Aligned 8-byte
    /// chunks XOR against the full mask in one go (auto-vectorised by
    /// LLVM); only the head/tail straddling alignment are scalar.
    pub fn bytes<'a>(&mut self, bytes: &'a mut [u8], xor_bytes: XORBytes) -> &'a mut [u8] {
        if xor_bytes.is_identity() {
            return bytes;
        }
        let xb = *xor_bytes;
        let mut phase = self.0;
        let len = bytes.len();
        let mut i = 0;

        while phase != 0 && i < len {
            bytes[i] ^= xb[phase];
            phase = (phase + 1) & (XOR_LEN - 1);
            i += 1;
        }

        let body_len = (len - i) & !(XOR_LEN - 1);
        for chunk in bytes[i..i + body_len].as_chunks_mut::<XOR_LEN>().0 {
            for (b, m) in chunk.iter_mut().zip(xb) {
                *b ^= m;
            }
        }
        i += body_len;

        while i < len {
            bytes[i] ^= xb[phase];
            phase = (phase + 1) & (XOR_LEN - 1);
            i += 1;
        }

        self.0 = phase;
        bytes
    }

    /// XOR-decode `buffer` as if it lived at `offset` in the blk file.
    #[inline]
    pub fn decode_at(buffer: &mut [u8], offset: usize, xor_bytes: XORBytes) {
        Self::at_offset(offset).bytes(buffer, xor_bytes);
    }
}
