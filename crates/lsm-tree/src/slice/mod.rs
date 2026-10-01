// Copyright (c) 2024-present, fjall-rs
// This source code is licensed under both the Apache 2.0 and MIT License
// (found in the LICENSE-* files in the repository)

use std::{
    borrow::Borrow,
    cmp::Ordering,
    io::{Read, Result},
    ops::{Deref, RangeBounds},
    path::{Path, PathBuf},
    sync::Arc,
};

use byteview::{Builder, ByteView};

/// An immutable byte slice that can be cloned without additional heap allocation.
/// There is no guarantee of alignment for zero-copy (de)serialization.
#[derive(Debug, Default, Clone, Eq, Hash, Ord)]
pub struct Slice(ByteView);

impl Slice {
    /// Creates an owned subslice.
    #[must_use]
    pub(crate) fn slice(&self, range: impl RangeBounds<usize>) -> Self {
        Self(self.0.slice(range))
    }

    /// Joins two byte slices into one owned slice.
    #[must_use]
    pub(crate) fn fused(left: &[u8], right: &[u8]) -> Self {
        Self(ByteView::fused(left, right))
    }

    /// Construct a [`Slice`] from a byte slice.
    #[must_use]
    pub(crate) fn new(bytes: &[u8]) -> Self {
        Self(bytes.into())
    }

    /// Returns the bytes slice containing the entire data.
    #[must_use]
    pub fn as_slice(&self) -> &[u8] {
        self
    }

    #[doc(hidden)]
    #[must_use]
    pub(crate) fn empty() -> Self {
        Self::default()
    }

    #[doc(hidden)]
    #[must_use]
    pub(crate) fn builder(len: usize) -> Builder {
        ByteView::builder(len)
    }

    #[doc(hidden)]
    pub(crate) fn from_reader<R: Read>(reader: &mut R, len: usize) -> Result<Self> {
        ByteView::from_reader(reader, len).map(Self)
    }
}

impl From<Vec<u8>> for Slice {
    fn from(value: Vec<u8>) -> Self {
        Self(ByteView::from(value))
    }
}

impl From<String> for Slice {
    fn from(value: String) -> Self {
        Self(ByteView::from(value.into_bytes()))
    }
}

impl From<ByteView> for Slice {
    fn from(value: ByteView) -> Self {
        Self(value)
    }
}

impl From<Slice> for ByteView {
    fn from(value: Slice) -> Self {
        value.0
    }
}

impl AsRef<[u8]> for Slice {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl From<&[u8]> for Slice {
    fn from(value: &[u8]) -> Self {
        Self(ByteView::new(value))
    }
}

impl From<Arc<[u8]>> for Slice {
    fn from(value: Arc<[u8]>) -> Self {
        Self::from(&*value)
    }
}

impl From<&Vec<u8>> for Slice {
    fn from(value: &Vec<u8>) -> Self {
        Self::from(value.as_slice())
    }
}

impl From<&str> for Slice {
    fn from(value: &str) -> Self {
        Self::from(value.as_bytes())
    }
}

impl From<&String> for Slice {
    fn from(value: &String) -> Self {
        Self::from(value.as_str())
    }
}

impl From<&Path> for Slice {
    fn from(value: &Path) -> Self {
        Self::from(value.as_os_str().as_encoded_bytes())
    }
}

impl From<PathBuf> for Slice {
    fn from(value: PathBuf) -> Self {
        Self::from(value.as_os_str().as_encoded_bytes())
    }
}

impl From<Arc<str>> for Slice {
    fn from(value: Arc<str>) -> Self {
        Self::from(&*value)
    }
}

impl<const N: usize> From<[u8; N]> for Slice {
    fn from(value: [u8; N]) -> Self {
        Self::from(value.as_slice())
    }
}

impl<const N: usize> From<&[u8; N]> for Slice {
    fn from(value: &[u8; N]) -> Self {
        Self::from(value.as_slice())
    }
}

impl FromIterator<u8> for Slice {
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = u8>,
    {
        Vec::from_iter(iter).into()
    }
}

impl Deref for Slice {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

impl Borrow<[u8]> for Slice {
    fn borrow(&self) -> &[u8] {
        self
    }
}

impl<T> PartialEq<T> for Slice
where
    T: AsRef<[u8]>,
{
    fn eq(&self, other: &T) -> bool {
        self.as_ref() == other.as_ref()
    }
}

impl PartialEq<Slice> for &[u8] {
    fn eq(&self, other: &Slice) -> bool {
        *self == other.as_ref()
    }
}

impl<T> PartialOrd<T> for Slice
where
    T: AsRef<[u8]>,
{
    fn partial_cmp(&self, other: &T) -> Option<Ordering> {
        self.as_ref().partial_cmp(other.as_ref())
    }
}

impl PartialOrd<Slice> for &[u8] {
    fn partial_cmp(&self, other: &Slice) -> Option<Ordering> {
        (*self).partial_cmp(other.as_ref())
    }
}

#[cfg(test)]
#[expect(clippy::expect_used)]
mod tests {
    use std::{fmt::Debug, io::Cursor, sync::Arc};

    use test_log::test;

    use super::Slice;

    fn assert_slice_handles<T>(v: T)
    where
        T: Clone + Debug,
        Slice: From<T> + PartialEq<T> + PartialOrd<T>,
    {
        // verify slice arc roundtrips
        let slice: Slice = v.clone().into();
        assert_eq!(slice, v, "slice_arc: {slice:?}, v: {v:?}");
        assert!(slice >= v, "slice_arc: {slice:?}, v: {v:?}");
    }

    #[test]
    fn slice_empty() {
        assert_eq!(Slice::empty(), []);
    }

    #[test]
    fn slice_fuse_empty() {
        let bytes = Slice::fused(&[], &[]);
        assert_eq!(&*bytes, &[] as &[u8]);
    }

    #[test]
    fn slice_fuse_one() {
        let bytes = Slice::fused(b"abc", &[]);
        assert_eq!(&*bytes, b"abc");
    }

    #[test]
    fn slice_fuse_two() {
        let bytes = Slice::fused(b"abc", b"def");
        assert_eq!(&*bytes, b"abcdef");
    }

    #[test]
    fn slice_with_size() {
        assert_eq!(
            &*{
                let mut b = Slice::builder(5);
                b.fill(0);
                b.freeze()
            },
            [0; 5],
        );
        assert_eq!(
            &*{
                let mut b = Slice::builder(50);
                b.fill(0);
                b.freeze()
            },
            [0; 50],
        );
        assert_eq!(
            &*{
                let mut b = Slice::builder(50);
                b.fill(77);
                b.freeze()
            },
            [77; 50],
        );
    }

    /// This test verifies that we can create a `Slice` from various types and compare a `Slice` with them.
    #[test]
    fn test_slice_instantiation() {
        // - &[u8]
        assert_slice_handles::<&[u8]>(&[1, 2, 3, 4]);
        // - Arc<u8>
        assert_slice_handles::<Arc<[u8]>>(Arc::new([1, 2, 3, 4]));
        // - Vec<u8>
        assert_slice_handles::<Vec<u8>>(vec![1, 2, 3, 4]);
        // - &str
        assert_slice_handles::<&str>("hello");
        // - String
        assert_slice_handles::<String>("hello".to_string());
        // - [u8; N]
        assert_slice_handles::<[u8; 4]>([1, 2, 3, 4]);

        // Special case for these types
        // - Iterator<Item = u8>
        let slice = Slice::from_iter(vec![1, 2, 3, 4]);
        assert_eq!(slice, vec![1, 2, 3, 4]);

        // - Arc<str>
        let arc_str: Arc<str> = Arc::from("hello");
        let slice = Slice::from(arc_str.clone());
        assert_eq!(slice.as_ref(), arc_str.as_bytes());

        // - io::Read
        let mut reader = Cursor::new(vec![1, 2, 3, 4]);
        let slice = Slice::from_reader(&mut reader, 4).expect("read");
        assert_eq!(slice, vec![1, 2, 3, 4]);
    }

    /// Verifies that `Slice::as_slice` returns the data as-is.
    #[test]
    fn test_slice_as_slice() {
        let original = [1_u8, 2, 3, 4];
        let slice = Slice::from_iter(original.iter().copied());
        assert_eq!(slice.as_slice(), original);
    }
}
