#[cfg(feature = "serde")]
use crate::Formattable;

use super::AnyReadableVec;

/// Type-erased trait for serializable vectors.
pub trait AnySerializableVec: AnyReadableVec {
    /// Write JSON array to output buffer
    #[cfg(feature = "serde")]
    fn write_json(
        &self,
        from: Option<usize>,
        to: Option<usize>,
        buf: &mut Vec<u8>,
    ) -> crate::Result<()>;

    /// Write one value as raw JSON, if the index is in bounds.
    #[cfg(feature = "serde")]
    fn write_json_value_at(&self, index: usize, buf: &mut Vec<u8>) -> crate::Result<()>;

    /// Return the last value as a serde_json::Value, or None if empty.
    #[cfg(feature = "serde_json")]
    fn last_json_value(&self) -> Option<serde_json::Value>;

    /// Write all values as CSV cells (newline-separated) directly without materializing a Vec.
    fn write_csv_column(
        &self,
        from: Option<usize>,
        to: Option<usize>,
        buf: &mut String,
    ) -> crate::Result<()>;
}

#[cfg(feature = "serde")]
impl<V> AnySerializableVec for V
where
    V: crate::TypedVec,
    V: crate::ReadableVec<V::I, V::T>,
    V::T: serde::Serialize + crate::Formattable,
{
    fn write_json(
        &self,
        from: Option<usize>,
        to: Option<usize>,
        buf: &mut Vec<u8>,
    ) -> crate::Result<()> {
        let len = self.len();
        let from_idx = from.unwrap_or(0);
        let to_idx = to.unwrap_or(len).min(len);

        let count = to_idx.saturating_sub(from_idx);
        buf.reserve(count * 20 + 2);

        buf.push(b'[');
        let mut first = true;
        self.for_each_range_at(from_idx, to_idx, |value: V::T| {
            if !first {
                buf.push(b',');
            }
            first = false;
            value.fmt_json(buf);
        });
        buf.push(b']');

        Ok(())
    }

    fn write_json_value_at(&self, index: usize, buf: &mut Vec<u8>) -> crate::Result<()> {
        if let Some(value) = self.collect_one_at(index) {
            value.fmt_json(buf);
        }
        Ok(())
    }

    #[cfg(feature = "serde_json")]
    fn last_json_value(&self) -> Option<serde_json::Value> {
        let value: V::T = self.collect_last()?;
        serde_json::to_value(&value).ok()
    }

    fn write_csv_column(
        &self,
        from: Option<usize>,
        to: Option<usize>,
        buf: &mut String,
    ) -> crate::Result<()> {
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
