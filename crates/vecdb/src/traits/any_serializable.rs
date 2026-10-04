#[cfg(feature = "serde")]
use serde::Serialize;

use super::AnyReadableVec;
use crate::Result;

#[cfg(feature = "serde")]
use crate::{Formattable, ReadableVec, TypedVec, nullable};

/// Type-erased trait for serializable vectors.
pub trait AnySerializableVec: AnyReadableVec {
    /// Whether JSON values can be `null`: missing or undefined (see [`Formattable`]).
    fn nullable(&self) -> bool;

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
    V::T: Serialize + Formattable,
{
    fn nullable(&self) -> bool {
        nullable::<V::T>()
    }

    fn write_json(&self, from: Option<usize>, to: Option<usize>, buf: &mut Vec<u8>) -> Result<()> {
        let len = self.len();
        let from_idx = from.unwrap_or(0);
        let to_idx = to.unwrap_or(len).min(len);

        let count = to_idx.saturating_sub(from_idx);
        buf.reserve(count * 20 + 2);

        buf.push(b'[');
        self.for_each_range_at(from_idx, to_idx, |value: V::T| {
            value.fmt_json(buf);
            // Clients type `null` only where a type declares it.
            debug_assert!(
                nullable::<V::T>() || !buf.ends_with(b"null"),
                "a JSON `null` from a type that declares neither `MISSING` nor `UNDEFINED`"
            );
            buf.push(b',');
        });
        if buf.last() == Some(&b',') {
            buf.pop();
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
