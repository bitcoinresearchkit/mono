use super::{super::RawStrategy, ReadWriteRawVec};
use crate::AnyStoredVec;

impl_read_write_storage!(ReadWriteRawVec, RawStrategy);
