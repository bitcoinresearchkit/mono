#[cfg(feature = "schemars")]
use std::borrow::Cow;
use std::{
    fmt,
    marker::PhantomData,
    ops::{Deref, DerefMut},
};

#[cfg(feature = "schemars")]
use schemars::{JsonSchema, Schema, SchemaGenerator};
use serde::{
    Deserialize, Deserializer, Serialize, Serializer,
    de::{Error, SeqAccess, Visitor},
};

/// Fixed-length, log-scale bin histogram generic over the per-bin counter type: raw counts
/// (`u32`), the smoothed EMA buffer (`f64`) or the quantized wire projection (`u16`). Serializes
/// as a flat JSON array of `N` values.
#[derive(Clone, Debug)]
pub struct Histogram<T, const N: usize>([T; N]);

impl<T: Copy + Default, const N: usize> Default for Histogram<T, N> {
    #[inline]
    fn default() -> Self {
        Self([T::default(); N])
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
    #[inline]
    pub fn increment(&mut self, bin: usize) {
        self.0[bin] += 1;
    }
}

impl<const N: usize> Histogram<f64, N> {
    /// Quantize each bin to `u16` (rounded, saturating) for the wire.
    /// Lossy by design: faint sub-0.5 bins vanish, which is invisible on a heatmap.
    pub fn to_compact(&self) -> Histogram<u16, N> {
        let mut out = [0u16; N];
        for (o, &v) in out.iter_mut().zip(self.0.iter()) {
            *o = v.round() as u16;
        }
        Histogram(out)
    }

    #[inline]
    pub fn add_from(&mut self, rhs: &Self) {
        self.0
            .iter_mut()
            .zip(rhs.0.iter())
            .for_each(|(a, &b)| *a += b);
    }

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
                        .ok_or_else(|| Error::invalid_length(i, &self))?;
                }
                Ok(Histogram(bins))
            }
        }

        deserializer.deserialize_seq(ArrayVisitor::<T, N>(PhantomData))
    }
}

#[cfg(feature = "schemars")]
impl<T: JsonSchema, const N: usize> JsonSchema for Histogram<T, N> {
    fn schema_name() -> Cow<'static, str> {
        format!("Histogram_{}", T::schema_name()).into()
    }

    fn json_schema(generator: &mut SchemaGenerator) -> Schema {
        let mut schema = Vec::<T>::json_schema(generator);
        schema.insert("minItems".to_owned(), N.into());
        schema.insert("maxItems".to_owned(), N.into());
        schema
    }

    /// Inline as a plain array rather than registering a named `Histogram_uintN`
    /// component: the wire shape is a flat array of counts, and the synthetic
    /// generic-mangled name has no real type for the Rust client to resolve to.
    fn inline_schema() -> bool {
        true
    }
}
