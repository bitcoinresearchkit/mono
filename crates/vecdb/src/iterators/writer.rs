use crate::Result;

/// Stateful writer for streaming values one at a time to a string buffer.
///
/// Useful for incremental serialization when memory constraints prevent
/// materializing entire collections.
pub trait ValueWriter {
    /// Writes the next value to the buffer in CSV format.
    ///
    /// # Errors
    /// Returns `IteratorEnded` when no more values are available.
    fn write_next(&mut self, buf: &mut String) -> Result<()>;
}
