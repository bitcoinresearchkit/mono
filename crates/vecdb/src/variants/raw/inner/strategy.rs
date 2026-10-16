use crate::{Format, ValueStrategy};

/// Strategy for raw (uncompressed) storage vectors.
///
/// Extends `ValueStrategy` with fast pointer-based reads used by mmap/IO iterators.
pub trait RawStrategy<T>: ValueStrategy<T> + 'static {
    /// On-disk format of vectors stored with this strategy.
    const FORMAT: Format;

    /// Reads a single T from a raw byte pointer at the given byte offset.
    ///
    /// For native-layout types, this compiles to a single `mov` instruction,
    /// bypassing slice creation, bounds checking, and Result overhead.
    ///
    /// # Safety
    /// - `ptr.add(byte_offset)` must be valid for reading `size_of::<T>()` bytes.
    /// - The bytes at that location must be a valid serialized T.
    unsafe fn read_from_ptr(ptr: *const u8, byte_offset: usize) -> T;
}
