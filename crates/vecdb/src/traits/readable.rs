use std::ops::AddAssign;

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
/// | `for_each` / `for_each_range` | Processing every element, static dispatch |
/// | `fold` / `fold_range` | Accumulating a result (SIMD-optimized on stored vecs) |
/// | `collect` / `collect_range` | Materializing values into a `Vec<T>` |
/// | `collect_one` / `collect_first` / `collect_last` | Materializing a single value |
/// | `for_each_range_dyn` | Trait-object contexts (`&dyn ReadableVec`) |
/// | `try_fold_range` | Fold with early exit on error |
/// | `read_into` / `cursor` | Sequential access with buffer reuse |
///
/// # Typed vs `_at` methods
///
/// Methods like `collect_one(I)`, `fold_range(I, I, …)` accept the typed index `I`.
/// The `_at` variants (`collect_one_at(usize)`, `fold_range_at(usize, usize, …)`)
/// accept raw `usize` and are what implementations override.
///
/// # Point reads
///
/// For raw vecs, use `VecReader::get()` for O(1) random access.
/// For short or unbounded sequential persisted raw reads, use
/// `vec.reader().cursor()` to avoid the general cursor's staging buffer.
/// For a known persisted raw range, use `vec.range_cursor_at(from, to)` so a
/// cold scan can use buffered I/O while resident data stays zero-copy.
/// For any vec through the trait, use `collect_one(i)` — this materializes
/// a single value (decodes a page for compressed vecs).
///
/// # Performance
///
/// Stored vecs override `fold_range_at` and `try_fold_range_at` to delegate to their
/// internal source's optimized `fold()`, enabling SIMD auto-vectorization.
///
/// The default `fold_range_at` uses chunked `read_into_at` calls with a monomorphized
/// inner loop — LLVM can auto-vectorize this without `&mut dyn FnMut` overhead.
///
/// For maximum throughput on stored vecs, prefer `fold_range` / `for_each_range`
/// with static dispatch (`&impl ReadableVec` or concrete type).
pub trait ReadableVec<I: VecIndex, T: VecValue>: AnyVec {
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

    /// Iterates over `[from, to)` by raw index, calling `f` for each value.
    ///
    /// Object-safe: callable on `&dyn ReadableVec`. Every implementor must
    /// provide this — there is intentionally no default to prevent silent
    /// fallback to a slow buffered path.
    fn for_each_range_dyn_at(&self, from: usize, to: usize, f: &mut dyn FnMut(T));

    /// Folds over `[from, to)` by raw index with an accumulator.
    ///
    /// Every implementor must provide an optimal path — there is intentionally
    /// no default to prevent silent fallback to a slow buffered path.
    fn fold_range_at<B, F: FnMut(B, T) -> B>(&self, from: usize, to: usize, init: B, f: F) -> B
    where
        Self: Sized;

    /// Fallible fold over `[from, to)` by raw index with early exit on error.
    ///
    /// Every implementor must provide an optimal path — there is intentionally
    /// no default to prevent silent fallback to a slow buffered path.
    fn try_fold_range_at<B, E, F: FnMut(B, T) -> std::result::Result<B, E>>(
        &self,
        from: usize,
        to: usize,
        init: B,
        f: F,
    ) -> std::result::Result<B, E>
    where
        Self: Sized;

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

    /// Iterates over `[from, to)` by typed index, calling `f` for each value (object-safe).
    #[inline]
    fn for_each_range_dyn(&self, from: I, to: I, f: &mut dyn FnMut(T)) {
        self.for_each_range_dyn_at(from.to_usize(), to.to_usize(), f)
    }

    /// Folds over `[from, to)` by typed index with an accumulator.
    #[inline]
    fn fold_range<B, F: FnMut(B, T) -> B>(&self, from: I, to: I, init: B, f: F) -> B
    where
        Self: Sized,
    {
        self.fold_range_at(from.to_usize(), to.to_usize(), init, f)
    }

    /// Fallible fold over `[from, to)` by typed index with early exit on error.
    #[inline]
    fn try_fold_range<B, E, F: FnMut(B, T) -> std::result::Result<B, E>>(
        &self,
        from: I,
        to: I,
        init: B,
        f: F,
    ) -> std::result::Result<B, E>
    where
        Self: Sized,
    {
        self.try_fold_range_at(from.to_usize(), to.to_usize(), init, f)
    }

    /// Calls `f` for each value in `[from, to)` by typed index. Requires `Sized`.
    #[inline]
    fn for_each_range<F: FnMut(T)>(&self, from: I, to: I, f: F)
    where
        Self: Sized,
    {
        self.for_each_range_at(from.to_usize(), to.to_usize(), f)
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

    /// Returns the minimum value in `[from, to)` by typed index, or `None` if empty.
    #[inline]
    fn min(&self, from: I, to: I) -> Option<T>
    where
        Self: Sized,
        T: PartialOrd,
    {
        self.min_at(from.to_usize(), to.to_usize())
    }

    /// Returns the maximum value in `[from, to)` by typed index, or `None` if empty.
    #[inline]
    fn max(&self, from: I, to: I) -> Option<T>
    where
        Self: Sized,
        T: PartialOrd,
    {
        self.max_at(from.to_usize(), to.to_usize())
    }

    /// Returns the sum of values in `[from, to)` by typed index, or `None` if empty.
    #[inline]
    fn sum(&self, from: I, to: I) -> Option<T>
    where
        Self: Sized,
        T: AddAssign + From<u8>,
    {
        self.sum_at(from.to_usize(), to.to_usize())
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
    fn try_for_each_range_at<E, F: FnMut(T) -> std::result::Result<(), E>>(
        &self,
        from: usize,
        to: usize,
        mut f: F,
    ) -> std::result::Result<(), E>
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
        let mut buf = Vec::with_capacity(to.saturating_sub(from));
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

    /// Collects the first value, or `None` if empty.
    #[inline]
    fn collect_first(&self) -> Option<T> {
        self.collect_one_at(0)
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

    /// Collects values using signed indices. Negative indices count from the end
    /// (Python-style): `-1` is the last element, `-2` is second-to-last, etc.
    #[inline]
    fn collect_signed_range(&self, from: Option<i64>, to: Option<i64>) -> Vec<T>
    where
        Self: Sized,
    {
        let from = from.map(|i| self.i64_to_usize(i)).unwrap_or(0);
        let to = to
            .map(|i| self.i64_to_usize(i))
            .unwrap_or_else(|| self.len());
        self.collect_range_at(from, to)
    }

    /// Collects values using signed indices (object-safe).
    #[inline]
    fn collect_signed_range_dyn(&self, from: Option<i64>, to: Option<i64>) -> Vec<T> {
        let from = from.map(|i| self.i64_to_usize(i)).unwrap_or(0);
        let to = to
            .map(|i| self.i64_to_usize(i))
            .unwrap_or_else(|| self.len());
        self.collect_range_dyn(from, to)
    }

    /// Collects all values into a `Vec<T>` (object-safe).
    #[inline]
    fn collect_dyn(&self) -> Vec<T> {
        self.collect_range_dyn(0, self.len())
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

    /// Reads values at specific sorted typed indices, appending to `out`.
    #[inline]
    fn read_sorted_into(&self, indices: &[I], out: &mut Vec<T>) {
        let raw: Vec<usize> = indices.iter().map(|i| i.to_usize()).collect();
        self.read_sorted_into_at(&raw, out);
    }

    /// Reads values at specific sorted typed indices, returning a new `Vec`.
    #[inline]
    fn read_sorted(&self, indices: &[I]) -> Vec<T> {
        let mut out = Vec::with_capacity(indices.len());
        self.read_sorted_into(indices, &mut out);
        out
    }

    // ── Aggregations ───────────────────────────────────────────────

    /// Returns the minimum value in `[from, to)` by raw index, or `None` if empty.
    #[inline]
    fn min_at(&self, from: usize, to: usize) -> Option<T>
    where
        Self: Sized,
        T: PartialOrd,
    {
        self.fold_range_at(from, to, None, |acc: Option<T>, v| match acc {
            Some(cur) if cur <= v => Some(cur),
            _ => Some(v),
        })
    }

    /// Returns the minimum value in `[from, to)`, or `None` if empty (object-safe).
    #[inline]
    fn min_dyn(&self, from: usize, to: usize) -> Option<T>
    where
        T: PartialOrd,
    {
        let mut result: Option<T> = None;
        self.for_each_range_dyn_at(from, to, &mut |v| match &result {
            Some(cur) if *cur <= v => {}
            _ => result = Some(v),
        });
        result
    }

    /// Returns the maximum value in `[from, to)` by raw index, or `None` if empty.
    #[inline]
    fn max_at(&self, from: usize, to: usize) -> Option<T>
    where
        Self: Sized,
        T: PartialOrd,
    {
        self.fold_range_at(from, to, None, |acc: Option<T>, v| match acc {
            Some(cur) if cur >= v => Some(cur),
            _ => Some(v),
        })
    }

    /// Returns the maximum value in `[from, to)`, or `None` if empty (object-safe).
    #[inline]
    fn max_dyn(&self, from: usize, to: usize) -> Option<T>
    where
        T: PartialOrd,
    {
        let mut result: Option<T> = None;
        self.for_each_range_dyn_at(from, to, &mut |v| match &result {
            Some(cur) if *cur >= v => {}
            _ => result = Some(v),
        });
        result
    }

    /// Returns the sum of values in `[from, to)` by raw index, or `None` if empty.
    #[inline]
    fn sum_at(&self, from: usize, to: usize) -> Option<T>
    where
        Self: Sized,
        T: AddAssign + From<u8>,
    {
        let mut has_values = false;
        let result = self.fold_range_at(from, to, T::from(0), |mut acc, v| {
            acc += v;
            has_values = true;
            acc
        });
        has_values.then_some(result)
    }

    /// Returns the sum of values in `[from, to)`, or `None` if empty (object-safe).
    #[inline]
    fn sum_dyn(&self, from: usize, to: usize) -> Option<T>
    where
        T: AddAssign + From<u8>,
    {
        let mut result = T::from(0);
        let mut has_values = false;
        self.for_each_range_dyn_at(from, to, &mut |v| {
            result += v;
            has_values = true;
        });
        has_values.then_some(result)
    }
}

/// Trait for readable vectors that can be cloned as trait objects.
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
