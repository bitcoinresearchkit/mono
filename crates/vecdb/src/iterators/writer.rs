use crate::{Error, Formattable, VecValue};

/// Stateful writer for streaming values one at a time to a string buffer.
///
/// Useful for incremental serialization when memory constraints prevent
/// materializing entire collections.
pub trait ValueWriter {
    /// Writes the next value to the buffer in CSV format.
    ///
    /// # Errors
    /// Returns `Error::WrongLength` when no more values are available.
    fn write_next(&mut self, buf: &mut String) -> crate::Result<()>;
}

/// Iterator-backed writer that formats values as CSV.
pub struct VecIteratorWriter<T> {
    pub iter: std::vec::IntoIter<T>,
}

impl<T> ValueWriter for VecIteratorWriter<T>
where
    T: VecValue + Formattable,
{
    fn write_next(&mut self, buf: &mut String) -> crate::Result<()> {
        if let Some(value) = self.iter.next() {
            value.fmt_csv(buf)?;
            Ok(())
        } else {
            Err(Error::IteratorEnded)
        }
    }
}
