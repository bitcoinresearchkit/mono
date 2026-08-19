use crate::{Bytes, Error, SIZE_OF_U64, Stamp};

/// Position-tracking reader for change-file payloads.
pub(crate) struct ChangeCursor<'a> {
    bytes: &'a [u8],
    pos: usize,
}

impl<'a> ChangeCursor<'a> {
    pub fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, pos: 0 }
    }

    pub fn read_u64(&mut self) -> crate::Result<usize> {
        self.check_remaining(SIZE_OF_U64)?;
        let v = usize::from_bytes(&self.bytes[self.pos..self.pos + SIZE_OF_U64])?;
        self.pos += SIZE_OF_U64;
        Ok(v)
    }

    pub fn read_stamp(&mut self) -> crate::Result<Stamp> {
        self.check_remaining(SIZE_OF_U64)?;
        let v = Stamp::from_bytes(&self.bytes[self.pos..self.pos + SIZE_OF_U64])?;
        self.pos += SIZE_OF_U64;
        Ok(v)
    }

    pub fn skip(&mut self, n: usize) -> crate::Result<()> {
        self.check_remaining(n)?;
        self.pos += n;
        Ok(())
    }

    pub fn read_values<T, F: FnMut(&[u8]) -> crate::Result<T>>(
        &mut self,
        count: usize,
        size_of_t: usize,
        mut read: F,
    ) -> crate::Result<Vec<T>> {
        let total = size_of_t.checked_mul(count).ok_or(Error::Overflow)?;
        self.check_remaining(total)?;
        let vals = self.bytes[self.pos..self.pos + total]
            .chunks(size_of_t)
            .map(&mut read)
            .collect::<crate::Result<Vec<_>>>()?;
        self.pos += total;
        Ok(vals)
    }

    fn check_remaining(&self, len: usize) -> crate::Result<()> {
        let end = self.pos.checked_add(len).ok_or(Error::Overflow)?;
        if end > self.bytes.len() {
            return Err(Error::WrongLength {
                received: self.bytes.len(),
                expected: end,
            });
        }
        Ok(())
    }
}
