// Copyright (c) 2024-present, fjall-rs
// This source code is licensed under both the Apache 2.0 and MIT License
// (found in the LICENSE-* files in the repository)

use std::{fs::File, io::BufReader, marker::PhantomData, path::Path};

use super::{Block, DataBlock};
use crate::{
    CompressionType, Error, InternalValue, RecordBytes, Result, Slice,
    table::{block::BlockType, owned_data_block_iter::OwnedDataBlockIter},
};

/// Table reader that is optimized for consuming an entire table
pub struct Scanner<K = Slice, V = Slice> {
    record: PhantomData<(K, V)>,
    reader: BufReader<File>,
    iter: OwnedDataBlockIter,

    compression: CompressionType,
    block_count: usize,
    read_count: usize,

    global_seqno: u64,
}

impl<K: RecordBytes, V: RecordBytes> Scanner<K, V> {
    pub fn new(
        path: &Path,
        block_count: usize,
        compression: CompressionType,
        global_seqno: u64,
    ) -> Result<Self> {
        // TODO: a larger buffer size may be better for HDD, maybe make this configurable
        // TODO: benchmarks were inconclusive on SSD, not much difference between 4KB - 2MB
        let mut reader = BufReader::with_capacity(8 * 4_096, File::open(path)?);

        let block = Self::fetch_next_block(&mut reader, compression)?;
        let iter = OwnedDataBlockIter::new(block, DataBlock::iter);

        Ok(Self {
            record: PhantomData,
            reader,
            iter,

            compression,
            block_count,
            read_count: 1,

            global_seqno,
        })
    }

    fn fetch_next_block(
        reader: &mut BufReader<File>,
        compression: CompressionType,
    ) -> Result<DataBlock> {
        let block = Block::from_reader(reader, compression);

        match block {
            Ok(block) => {
                if block.header.block_type != BlockType::Data {
                    return Err(Error::InvalidTag((
                        "BlockType",
                        block.header.block_type.into(),
                    )));
                }

                Ok(DataBlock::new(block))
            }
            Err(e) => Err(e),
        }
    }
}

impl<K: RecordBytes, V: RecordBytes> Iterator for Scanner<K, V> {
    type Item = Result<InternalValue<K, V>>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(item) = self.iter.next_as::<K, V>() {
                let mut item = fail_iter!(item);
                item.key.seqno += self.global_seqno;
                return Some(Ok(item));
            }

            if self.read_count >= self.block_count {
                return None;
            }

            // Init new block
            let block = fail_iter!(Self::fetch_next_block(&mut self.reader, self.compression));
            self.iter = OwnedDataBlockIter::new(block, DataBlock::iter);

            self.read_count += 1;
        }
    }
}
