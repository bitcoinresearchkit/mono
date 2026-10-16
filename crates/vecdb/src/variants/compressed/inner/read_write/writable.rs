use super::{super::CompressionStrategy, ReadWriteCompressedVec};
use crate::AnyStoredVec;

impl_read_write_storage!(ReadWriteCompressedVec, CompressionStrategy);
