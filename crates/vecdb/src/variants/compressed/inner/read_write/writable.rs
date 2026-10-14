use super::{super::CompressionStrategy, ReadWriteCompressedVec};
use crate::AnyStoredVec;

impl_read_write_writable!(ReadWriteCompressedVec, CompressionStrategy);
