pub mod bytes;

/// Storage format, recorded in each vector's header.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Format {
    /// Serialization through [`Bytes`](crate::Bytes): portable across endianness.
    Bytes,
    /// Native layout through zerocopy: endianness-dependent.
    ZeroCopy,
    /// Pcodec compression of numeric values.
    Pco = 64,
    LZ4 = 65,
    Zstd = 66,
}
