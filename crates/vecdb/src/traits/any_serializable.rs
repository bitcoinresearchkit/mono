use crate::Result;
#[cfg(feature = "serde")]
use crate::{Formattable, ReadableVec, TypedVec};

use super::AnyReadableVec;

/// Type-erased trait for serializable vectors.
pub trait AnySerializableVec: AnyReadableVec {
    /// Write JSON array to output buffer
    #[cfg(feature = "serde")]
    fn write_json(&self, from: Option<usize>, to: Option<usize>, buf: &mut Vec<u8>) -> Result<()>;

    /// Write one value as raw JSON, if the index is in bounds.
    #[cfg(feature = "serde")]
    fn write_json_value_at(&self, index: usize, buf: &mut Vec<u8>) -> Result<()>;

    /// Write all values as CSV cells (newline-separated) directly without materializing a Vec.
    fn write_csv_column(
        &self,
        from: Option<usize>,
        to: Option<usize>,
        buf: &mut String,
    ) -> Result<()>;
}

#[cfg(feature = "serde")]
impl<V> AnySerializableVec for V
where
    V: TypedVec,
    V: ReadableVec<V::I, V::T>,
    V::T: serde::Serialize + Formattable,
{
    fn write_json(&self, from: Option<usize>, to: Option<usize>, buf: &mut Vec<u8>) -> Result<()> {
        let len = self.len();
        let from_idx = from.unwrap_or(0);
        let to_idx = to.unwrap_or(len).min(len);

        let count = to_idx.saturating_sub(from_idx);
        buf.reserve(count * 20 + 2);

        buf.push(b'[');
        self.for_each_range_at(from_idx, to_idx, |value: V::T| {
            value.fmt_json(buf);
            buf.push(b',');
        });
        if count > 0 {
            let _ = buf.pop();
        }
        buf.push(b']');

        Ok(())
    }

    fn write_json_value_at(&self, index: usize, buf: &mut Vec<u8>) -> Result<()> {
        if let Some(value) = self.collect_one_at(index) {
            value.fmt_json(buf);
        }
        Ok(())
    }

    fn write_csv_column(
        &self,
        from: Option<usize>,
        to: Option<usize>,
        buf: &mut String,
    ) -> Result<()> {
        let len = self.len();
        let from_idx = from.unwrap_or(0);
        let to_idx = to.unwrap_or(len).min(len);

        let count = to_idx.saturating_sub(from_idx);
        buf.reserve(count * 20);

        self.for_each_range_at(from_idx, to_idx, |value: V::T| {
            value.fmt_csv(buf).expect("csv formatting failed");
            buf.push('\n');
        });

        Ok(())
    }
}
