use std::{convert::Infallible, result::Result};

use crate::{AnyVec, ReadableBoxedVec, VecIndex, VecValue, cursor::Cursor};

/// Default chunk size for chunked iteration (matches PcoVec page size).
pub const READ_CHUNK_SIZE: usize = 4096;

/// High-performance reading of vector values.
///
/// This is the primary trait for reading data from any vec type — stored, compressed,
/// lazy, or computed. All methods see the full state including uncommitted (pushed) values.
///
/// # Method overview
///
/// | Method | Use when |
/// |--------|----------|
/// | `for_each` / `for_each_range_at` | Processing every element, static dispatch |
/// | `fold` / `fold_range_at` | Accumulating a result (SIMD-optimized on stored vecs) |
/// | `collect` / `collect_range` | Materializing values into a `Vec<T>` |
/// | `collect_one` / `collect_last` | Materializing a single value |
/// | `for_each_range_dyn` | Trait-object contexts (`&dyn ReadableVec`) |
/// | `try_fold_range_at` | Fold with early exit on error |
/// | `read_into` / `cursor` | Sequential access with buffer reuse |
///
/// # Typed vs `_at` methods
///
/// Methods like `collect_one(I)` accept the typed index `I`. The `_at` variants
/// (`collect_one_at(usize)`, `fold_range_at(usize, usize, …)`) accept raw `usize`
/// and are what implementations override.
///
/// # Point reads
///
/// For raw vecs, use `VecReader::get()` for O(1) random access, including short
/// sequential persisted reads without the general cursor's staging buffer.
/// For a known persisted raw range, use `vec.range_cursor_at(from, to)` so a
/// cold scan can use buffered I/O while resident data stays zero-copy.
/// Compressed vectors expose the same bounded `range_cursor_at` role: pages are
/// decoded once while a cold range uses sequential buffered I/O.
/// For any vec through the trait, use `collect_one(i)` — this materializes
/// a single value (decodes a page for compressed vecs).
///
/// # Performance
///
/// Stored vecs implement `try_fold_range_at` with their internal source's tight
/// loop, enabling SIMD auto-vectorization; `fold_range_at` reuses it.
///
/// For maximum throughput on stored vecs, prefer `fold_range_at` / `for_each_range_at`
/// with static dispatch (`&impl ReadableVec` or concrete type).
pub trait ReadableVec<I: VecIndex, T: VecValue>: AnyVec {
    /// Append a complete retained range without source reads or cache fills.
    /// On a miss (including unavailable bounds), leave `out` unchanged.
    fn read_cached_into_at(&self, _from: usize, _to: usize, _out: &mut Vec<T>) -> bool {
        false
    }

    // ── Required ─────────────────────────────────────────────────────

    /// Preferred number of values per cursor refill.
    ///
    /// Compressed vectors return their format page size so sequential cursors
    /// decode each page once. Other vectors use [`READ_CHUNK_SIZE`].
    #[inline]
    fn cursor_chunk_size(&self) -> usize {
        READ_CHUNK_SIZE
    }

    /// Appends elements in `[from, to)` to `buf`.
    ///
    /// Implementations MUST NOT clear `buf` — they only append. This enables
    /// chunked fold (clear + fill per chunk) and multi-range reads (append
    /// multiple ranges into one buffer).
    ///
    /// Object-safe: `&mut Vec<T>` is a concrete type, no `Self: Sized` needed.
    fn read_into_at(&self, from: usize, to: usize, buf: &mut Vec<T>);

    /// Visits borrowed chunks of the values emitted by `[from, to)`, in order.
    /// Each offset is `from + previously_emitted_values`; for vectors with
    /// holes this is not the physical position of every value in the slice.
    /// Slices are valid only during the callback. Sources may choose any chunk
    /// size. Empty chunks and ranges do not invoke the callback.
    /// Callbacks must finish before rewriting, truncating or moving the source's
    /// region: borrowed chunks may hold its read lock.
    ///
    /// The default reuses one read buffer. Resident sources can lend their
    /// storage directly, avoiding copies through type-erased readers.
    fn for_each_chunk_at(&self, from: usize, to: usize, f: &mut dyn FnMut(usize, &[T])) {
        super::chunk_folds::for_each_chunk(self, from, to, f);
    }

    /// Iterates over `[from, to)` by raw index, calling `f` for each value.
    ///
    /// Object-safe: callable on `&dyn ReadableVec`. Every implementor must
    /// provide this — there is intentionally no default to prevent silent
    /// fallback to a slow buffered path.
    fn for_each_range_dyn_at(&self, from: usize, to: usize, f: &mut dyn FnMut(T));

    /// Fallible fold over `[from, to)` by raw index with early exit on error.
    ///
    /// Every implementor must provide an optimal path — there is intentionally
    /// no default to prevent silent fallback to a slow buffered path.
    fn try_fold_range_at<B, E, F: FnMut(B, T) -> Result<B, E>>(
        &self,
        from: usize,
        to: usize,
        init: B,
        f: F,
    ) -> Result<B, E>
    where
        Self: Sized;

    /// Folds over `[from, to)` by raw index with an accumulator.
    ///
    /// Defaults to [`Self::try_fold_range_at`] with an error that cannot occur, which
    /// compiles to the same loop. Override only for a cheaper infallible path.
    #[inline]
    fn fold_range_at<B, F: FnMut(B, T) -> B>(&self, from: usize, to: usize, init: B, mut f: F) -> B
    where
        Self: Sized,
    {
        let Ok(acc) = self.try_fold_range_at(from, to, init, |acc, value| {
            Ok::<_, Infallible>(f(acc, value))
        });
        acc
    }

    // ── Typed-index wrappers ───────────────────────────────────────────

    /// Appends elements in `[from, to)` to `buf` using typed indices.
    #[inline]
    fn read_into(&self, from: I, to: I, buf: &mut Vec<T>) {
        self.read_into_at(from.to_usize(), to.to_usize(), buf)
    }

    /// Creates a forward-only `Cursor` that reuses an internal buffer across
    /// chunked `read_into_at` calls. One allocation for the lifetime of the cursor.
    #[inline]
    fn cursor(&self) -> Cursor<'_, I, T, Self>
    where
        Self: Sized,
    {
        Cursor::new(self)
    }

    /// A [`Self::cursor`] that reads only `[from, to)`, starting at `from`.
    #[inline]
    fn bounded_cursor_at(&self, from: usize, to: usize) -> Cursor<'_, I, T, Self>
    where
        Self: Sized,
    {
        Cursor::bounded(self, from, to)
    }

    /// Iterates over `[from, to)` by typed index, calling `f` for each value (object-safe).
    #[inline]
    fn for_each_range_dyn(&self, from: I, to: I, f: &mut dyn FnMut(T)) {
        self.for_each_range_dyn_at(from.to_usize(), to.to_usize(), f)
    }

    /// Collects values in `[from, to)` by typed index into a `Vec<T>`.
    #[inline]
    fn collect_range(&self, from: I, to: I) -> Vec<T>
    where
        Self: Sized,
    {
        self.collect_range_at(from.to_usize(), to.to_usize())
    }

    /// Collects a single value at typed `index`, or `None` if out of bounds.
    #[inline]
    fn collect_one(&self, index: I) -> Option<T> {
        self.collect_one_at(index.to_usize())
    }

    // ── Raw-index convenience (all have defaults) ──────────────────────

    /// Calls `f` for each value in `[from, to)` by raw index. Requires `Sized` (static dispatch).
    #[inline]
    fn for_each_range_at<F: FnMut(T)>(&self, from: usize, to: usize, mut f: F)
    where
        Self: Sized,
    {
        self.fold_range_at(from, to, (), |(), v| f(v));
    }

    /// Fallible for-each over `[from, to)` by raw index with early exit on error.
    #[inline]
    fn try_for_each_range_at<E, F: FnMut(T) -> Result<(), E>>(
        &self,
        from: usize,
        to: usize,
        mut f: F,
    ) -> Result<(), E>
    where
        Self: Sized,
    {
        self.try_fold_range_at(from, to, (), |(), v| f(v))
    }

    /// Calls `f` for every value in the vector.
    #[inline]
    fn for_each<F: FnMut(T)>(&self, f: F)
    where
        Self: Sized,
    {
        self.for_each_range_at(0, self.len(), f);
    }

    /// Folds over all values with an accumulator.
    #[inline]
    fn fold<B, F: FnMut(B, T) -> B>(&self, init: B, f: F) -> B
    where
        Self: Sized,
    {
        self.fold_range_at(0, self.len(), init, f)
    }

    /// Collects values in `[from, to)` by raw index into a `Vec<T>`.
    #[inline]
    fn collect_range_at(&self, from: usize, to: usize) -> Vec<T>
    where
        Self: Sized,
    {
        self.collect_range_dyn(from, to)
    }

    /// Clears `buf` then fills it with values in `[from, to)`. Reuses the buffer allocation.
    #[inline]
    fn collect_range_into_at(&self, from: usize, to: usize, buf: &mut Vec<T>) {
        buf.clear();
        self.read_into_at(from, to, buf);
    }

    /// Collects values in `[from, to)` into a `Vec<T>` (object-safe).
    #[inline]
    fn collect_range_dyn(&self, from: usize, to: usize) -> Vec<T> {
        let mut buf = Vec::with_capacity(to.min(self.len()).saturating_sub(from));
        self.read_into_at(from, to, &mut buf);
        buf
    }

    /// Collects all values into a `Vec<T>`.
    #[inline]
    fn collect(&self) -> Vec<T>
    where
        Self: Sized,
    {
        self.collect_range_at(0, self.len())
    }

    /// Collects a single value at raw `index`, or `None` if out of bounds.
    ///
    /// Uses `for_each_range_dyn_at` with a stack-local `Option`. Stored vecs
    /// override `for_each_range_dyn_at` → `fold_range_at` for zero-alloc reads;
    /// lazy vecs override `collect_one_at` directly.
    #[inline(always)]
    fn collect_one_at(&self, index: usize) -> Option<T> {
        if index >= self.len() {
            return None;
        }
        let mut result = None;
        self.for_each_range_dyn_at(index, index + 1, &mut |v| result = Some(v));
        result
    }

    /// Collects the last value, or `None` if empty.
    #[inline]
    fn collect_last(&self) -> Option<T> {
        let len = self.len();
        if len > 0 {
            self.collect_one_at(len - 1)
        } else {
            None
        }
    }

    // ── Sparse reads ──────────────────────────────────────────────

    /// Reads values at specific sorted indices (ascending), skipping holes.
    ///
    /// Default uses a forward-only [`Cursor`] — each underlying page is read
    /// at most once. Lazy vecs override to map indices to their source and
    /// call this method recursively, bottoming out at stored vecs.
    ///
    /// `indices` **must** be sorted ascending. Output order matches `indices`.
    fn read_sorted_into_at(&self, indices: &[usize], out: &mut Vec<T>) {
        let mut cursor = Cursor::new(self);
        out.reserve(indices.len());
        indices.iter().for_each(|&i| {
            if let Some(v) = cursor.get(i) {
                out.push(v);
            }
        });
    }

    /// Reads values at specific sorted indices (ascending), returning a new `Vec`.
    #[inline]
    fn read_sorted_at(&self, indices: &[usize]) -> Vec<T> {
        let mut out = Vec::with_capacity(indices.len());
        self.read_sorted_into_at(indices, &mut out);
        out
    }
}

/// A reader that can produce an owned, read-only trait object.
///
/// The returned reader is `'static`; the source need not be. Borrowed consumers
/// can use this trait without requiring `Clone` or `'static` on the source.
pub trait ReadableCloneableVec<I: VecIndex, T: VecValue>: ReadableVec<I, T> {
    fn read_only_boxed_clone(&self) -> ReadableBoxedVec<I, T>;
}

impl<I: VecIndex, T: VecValue, U> ReadableCloneableVec<I, T> for U
where
    U: 'static + ReadableVec<I, T> + Clone,
{
    fn read_only_boxed_clone(&self) -> ReadableBoxedVec<I, T> {
        ReadableBoxedVec::new(self.clone())
    }
}

/// Emits `read_into_at`, `for_each_range_dyn_at`, `fold_range_at` and `try_fold_range_at`
/// inside an `impl ReadableVec<_, $value>` block, from the type's own value visitors:
/// `$each(&self, from, to, impl FnMut($value))` and its fallible twin
/// `$try_each(&self, from, to, impl FnMut($value) -> Result<(), E>) -> Result<(), E>`.
/// Point and sorted reads stay explicit.
#[macro_export]
macro_rules! visitor_read_methods {
    ($value:ty, $each:ident, $try_each:ident) => {
        fn read_into_at(&self, from: usize, to: usize, buf: &mut Vec<$value>) {
            buf.reserve(to.min($crate::AnyVec::len(self)).saturating_sub(from));
            self.$each(from, to, |value| buf.push(value));
        }

        fn for_each_range_dyn_at(&self, from: usize, to: usize, each: &mut dyn FnMut($value)) {
            self.$each(from, to, each);
        }

        fn fold_range_at<B, F: FnMut(B, $value) -> B>(
            &self,
            from: usize,
            to: usize,
            init: B,
            mut fold: F,
        ) -> B {
            let mut acc = Some(init);
            self.$each(from, to, |value| {
                acc = Some(fold(acc.take().unwrap(), value));
            });
            acc.unwrap()
        }

        fn try_fold_range_at<B, E, F: FnMut(B, $value) -> ::std::result::Result<B, E>>(
            &self,
            from: usize,
            to: usize,
            init: B,
            mut fold: F,
        ) -> ::std::result::Result<B, E> {
            let mut acc = Some(init);
            self.$try_each(from, to, |value| {
                acc = Some(fold(acc.take().unwrap(), value)?);
                Ok(())
            })?;
            Ok(acc.unwrap())
        }
    };
}
