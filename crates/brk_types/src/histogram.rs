use std::{
    fmt,
    marker::PhantomData,
    ops::{Deref, DerefMut},
};

use schemars::{JsonSchema, SchemaGenerator};
use serde::{
    Deserialize, Deserializer, Serialize, Serializer,
    de::{SeqAccess, Visitor},
};

/// Fixed-length, log-scale bin histogram generic over the per-bin counter type.
/// Instantiated as raw counts (`u32`), the smoothed EMA buffer (`f64`), or the
/// quantized wire projection (`u16`). Serializes as a flat JSON array of `N`
/// values. `Deref` exposes the underlying array for indexing and iteration.
///
/// Backed by a fixed `[T; N]` (not a `Vec`) to keep the always-`N` invariant the
/// callers rely on.
#[derive(Clone, Debug)]
pub struct Histogram<T, const N: usize>([T; N]);

impl<T: Copy + Default, const N: usize> Histogram<T, N> {
    #[inline]
    pub fn zeros() -> Self {
        Self([T::default(); N])
    }
}

impl<T: Copy + Default, const N: usize> Default for Histogram<T, N> {
    fn default() -> Self {
        Self::zeros()
    }
}

impl<T, const N: usize> Deref for Histogram<T, N> {
    type Target = [T; N];
    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T, const N: usize> DerefMut for Histogram<T, N> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<const N: usize> Histogram<u32, N> {
    /// Bump the count in `bin` by one.
    #[inline]
    pub fn increment(&mut self, bin: usize) {
        self.0[bin] += 1;
    }
}

impl<const N: usize> Histogram<f64, N> {
    /// Quantize each bin to `u16` (round, then clamp into range) for the wire.
    /// Lossy by design: faint sub-0.5 bins vanish, which is invisible on a heatmap.
    pub fn to_compact(&self) -> Histogram<u16, N> {
        let mut out = [0u16; N];
        for (o, &v) in out.iter_mut().zip(self.0.iter()) {
            *o = v.round().clamp(0.0, u16::MAX as f64) as u16;
        }
        Histogram(out)
    }

    /// Add another floating-point histogram bin-by-bin.
    #[inline]
    pub fn add_from(&mut self, rhs: &Self) {
        self.0
            .iter_mut()
            .zip(rhs.0.iter())
            .for_each(|(a, &b)| *a += b);
    }

    /// Divide every bin by `rhs`.
    #[inline]
    pub fn divide_by(&mut self, rhs: f64) {
        self.0.iter_mut().for_each(|v| *v /= rhs);
    }
}

impl<T: Serialize, const N: usize> Serialize for Histogram<T, N> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.0.as_slice().serialize(serializer)
    }
}

impl<'de, T: Deserialize<'de> + Copy + Default, const N: usize> Deserialize<'de>
    for Histogram<T, N>
{
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct ArrayVisitor<T, const N: usize>(PhantomData<T>);

        impl<'de, T: Deserialize<'de> + Copy + Default, const N: usize> Visitor<'de>
            for ArrayVisitor<T, N>
        {
            type Value = Histogram<T, N>;

            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                write!(f, "an array of {N} values")
            }

            fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
                let mut bins = [T::default(); N];
                for (i, bin) in bins.iter_mut().enumerate() {
                    *bin = seq
                        .next_element()?
                        .ok_or_else(|| serde::de::Error::invalid_length(i, &self))?;
                }
                Ok(Histogram(bins))
            }
        }

        deserializer.deserialize_seq(ArrayVisitor::<T, N>(PhantomData))
    }
}

impl<T: JsonSchema, const N: usize> JsonSchema for Histogram<T, N> {
    fn schema_name() -> std::borrow::Cow<'static, str> {
        format!("Histogram_{}", T::schema_name()).into()
    }

    fn json_schema(generator: &mut SchemaGenerator) -> schemars::Schema {
        let mut schema = Vec::<T>::json_schema(generator);
        schema.insert("minItems".to_owned(), N.into());
        schema.insert("maxItems".to_owned(), N.into());
        schema
    }

    /// Inline as a plain array rather than registering a named `Histogram_uintN`
    /// component: the wire shape is just a flat array of counts, and the synthetic
    /// generic-mangled name has no real type for the Rust client to resolve to.
    fn inline_schema() -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn float_histogram_add_and_divide_are_binwise() {
        let mut a = Histogram::<f64, 3>::zeros();
        a[0] = 1.0;
        a[1] = 2.0;
        a[2] = 3.0;

        let mut b = Histogram::<f64, 3>::zeros();
        b[0] = 3.0;
        b[1] = 4.0;
        b[2] = 5.0;

        a.add_from(&b);
        a.divide_by(2.0);

        assert_eq!(*a, [2.0, 3.0, 4.0]);
    }

    #[test]
    fn schema_preserves_the_fixed_bin_count() {
        let schema = serde_json::to_value(schemars::schema_for!(Histogram<u16, 3>)).unwrap();
        assert_eq!(schema["type"], "array");
        assert_eq!(schema["minItems"], 3);
        assert_eq!(schema["maxItems"], 3);
    }
}
