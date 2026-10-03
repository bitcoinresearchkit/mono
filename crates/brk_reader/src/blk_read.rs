use std::{
    fs::File,
    io::{Read, Result},
    os::unix::fs::FileExt,
    sync::Arc,
};

use brk_error::Result as ErrorResult;
use brk_types::BlkPosition;

use crate::{Reader, XORBytes, XORIndex};

pub struct BlkRead {
    file: Arc<File>,
    offset: u64,
    xor_index: XORIndex,
    xor_bytes: XORBytes,
}

impl BlkRead {
    fn new(file: Arc<File>, position: BlkPosition, xor_bytes: XORBytes) -> Self {
        Self {
            file,
            offset: u64::from(position.offset()),
            xor_index: XORIndex::at_offset(position.offset() as usize),
            xor_bytes,
        }
    }
}

impl Reader {
    pub fn reader_at(&self, position: BlkPosition) -> ErrorResult<BlkRead> {
        let file = self.0.open_blk(position.blk_index())?;
        Ok(BlkRead::new(file, position, self.0.xor_bytes))
    }
}

impl Read for BlkRead {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        let n = self.file.read_at(buf, self.offset)?;
        self.xor_index.bytes(&mut buf[..n], self.xor_bytes);
        self.offset += n as u64;
        Ok(n)
    }
}
