use std::{
    fs::File,
    io::{BufReader, Read, Result},
    os::unix::fs::FileExt,
    sync::Arc,
};

use brk_error::Result as ErrorResult;
use brk_types::BlkPosition;

use crate::{Reader, XORBytes, XORIndex};

/// Sequential reader from a blk-file position, XOR-decoded. Buffered: the
/// consensus decoders read field by field, and each unbuffered read would be
/// its own `pread`.
pub struct BlkRead {
    inner: BufReader<PositionedFile>,
    /// Phase at the logical (consumed) offset, not the buffered file offset.
    xor_index: XORIndex,
    xor_bytes: XORBytes,
}

struct PositionedFile {
    file: Arc<File>,
    offset: u64,
}

impl Reader {
    pub fn reader_at(&self, position: BlkPosition) -> ErrorResult<BlkRead> {
        let file = self.0.open_blk(position.blk_index())?;
        Ok(BlkRead {
            inner: BufReader::new(PositionedFile {
                file,
                offset: u64::from(position.offset()),
            }),
            xor_index: XORIndex::at_offset(position.offset() as usize),
            xor_bytes: self.0.xor_bytes,
        })
    }
}

impl Read for PositionedFile {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        let n = self.file.read_at(buf, self.offset)?;
        self.offset += n as u64;
        Ok(n)
    }
}

impl Read for BlkRead {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        let n = self.inner.read(buf)?;
        self.xor_index.bytes(&mut buf[..n], self.xor_bytes);
        Ok(n)
    }
}
