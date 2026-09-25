use std::{fmt::Debug, ops::Range};

use crate::{Error, Result, Slice};

/// Owned bytes used for decoded records and ingestion buffers.
///
/// Fixed arrays carry their length in the type and need no allocation or destructor.
/// `Slice` supports variable-length records, metadata, and general-purpose callers.
pub trait RecordBytes: AsRef<[u8]> + Clone + Ord + Debug + Send + 'static {
    /// The encoded length, or `None` for variable-length bytes.
    ///
    /// Implementations must return this length from `AsRef` when it is specified.
    const FIXED_LEN: Option<usize> = None;

    /// Creates a placeholder for a tombstone's absent value.
    fn empty() -> Self;

    /// Copies or shares the indicated bytes from a block.
    ///
    /// # Errors
    /// Returns an error if the bytes do not match this representation's length.
    fn from_block(bytes: &Slice, range: Range<usize>) -> Result<Self>;

    /// Reconstructs a prefix-compressed key.
    ///
    /// # Errors
    /// Returns an error if the combined bytes do not match this representation's length.
    fn from_parts(prefix: &[u8], suffix: &[u8]) -> Result<Self>;
}

impl RecordBytes for Slice {
    fn empty() -> Self {
        Self::empty()
    }
    fn from_block(bytes: &Slice, range: Range<usize>) -> Result<Self> {
        Ok(bytes.slice(range))
    }
    fn from_parts(prefix: &[u8], suffix: &[u8]) -> Result<Self> {
        Ok(Self::fused(prefix, suffix))
    }
}

impl<const N: usize> RecordBytes for [u8; N] {
    const FIXED_LEN: Option<usize> = Some(N);

    fn empty() -> Self {
        [0; N]
    }

    fn from_block(bytes: &Slice, range: Range<usize>) -> Result<Self> {
        let bytes = bytes.get(range).ok_or(Error::InvalidTrailer)?;
        bytes.try_into().map_err(|_| Error::InvalidRecordLength {
            expected: N,
            actual: bytes.len(),
        })
    }

    fn from_parts(prefix: &[u8], suffix: &[u8]) -> Result<Self> {
        let actual = prefix.len() + suffix.len();
        if actual != N {
            return Err(Error::InvalidRecordLength {
                expected: N,
                actual,
            });
        }
        let mut bytes = [0; N];
        let (left, right) = bytes.split_at_mut(prefix.len());
        left.copy_from_slice(prefix);
        right.copy_from_slice(suffix);
        Ok(bytes)
    }
}
