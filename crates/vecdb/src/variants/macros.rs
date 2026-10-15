/// Generates trait implementations for vec wrappers (LZ4Vec, PcoVec, ZstdVec, BytesVec, ZeroCopyVec).
///
/// # Usage
/// ```ignore
/// impl_vec_wrapper!(
///     LZ4Vec,
///     ReadWriteCompressedVec<I, T, LZ4Strategy<T>>,
///     LZ4VecValue,
///     Format::LZ4,
/// );
/// ```
///
/// This generates implementations for:
/// - `Deref`, plus `DerefMut` unless `no_deref_mut` is requested
/// - `ImportableVec`
/// - `AnyVec`
/// - `TypedVec`
/// - `AnyStoredVec`
/// - `WritableVec`
/// - `ReadableVec` (delegates `for_each_range_dyn_at` / `fold_range_at` to inner)
macro_rules! impl_vec_wrapper {
    ($wrapper:ident, $inner:ty, $value_trait:ident, $format:expr, $read_only:ty) => {
        impl_vec_wrapper!(
            @with_deref_mut
            $wrapper,
            $inner,
            $value_trait,
            $format,
            $read_only,
            yes
        );
    };
    ($wrapper:ident, $inner:ty, $value_trait:ident, $format:expr, $read_only:ty, no_deref_mut) => {
        impl_vec_wrapper!(
            @with_deref_mut
            $wrapper,
            $inner,
            $value_trait,
            $format,
            $read_only,
            no
        );
    };
    (@with_deref_mut $wrapper:ident, $inner:ty, $value_trait:ident, $format:expr, $read_only:ty, $deref_mut:ident) => {
        impl<I, T: $crate::VecValue, C: $crate::cache::CachePolicy> ::std::ops::Deref for $wrapper<I, T, C> {
            type Target = $inner;

            #[inline]
            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }

        impl_vec_wrapper!(@deref_mut $deref_mut, $wrapper, $inner);

        impl<I, T: $crate::VecValue, C: $crate::cache::CachePolicy> $wrapper<I, T, C>
        where
            I: $crate::VecIndex,
            T: $value_trait,
        {
            /// On-disk format of this vector type.
            pub(crate) const FORMAT: $crate::Format = $format;
        }

        impl<I, T: $crate::VecValue, C: $crate::cache::CachePolicy> $crate::ImportableVec for $wrapper<I, T, C>
        where
            I: $crate::VecIndex,
            T: $value_trait,
        {
            fn import_with(options: $crate::ImportOptions) -> $crate::Result<Self> {
                Ok(Self(<$inner>::import_with(options, Self::FORMAT)?))
            }
        }

        impl<I, T: $crate::VecValue, C: $crate::cache::CachePolicy> $crate::AnyVec for $wrapper<I, T, C>
        where
            I: $crate::VecIndex,
            T: $value_trait,
        {
            #[inline]
            fn version(&self) -> $crate::Version {
                self.0.version()
            }

            #[inline]
            fn name(&self) -> &str {
                self.0.name()
            }

            #[inline]
            fn len(&self) -> usize {
                self.0.len()
            }

            #[inline]
            fn index_type_to_string(&self) -> &'static str {
                self.0.index_type_to_string()
            }

            #[inline]
            fn value_type_to_size_of(&self) -> usize {
                self.0.value_type_to_size_of()
            }

            #[inline]
            fn value_type_to_string(&self) -> &'static str {
                self.0.value_type_to_string()
            }

            #[inline]
            fn region_names(&self) -> Vec<String> {
                self.0.region_names()
            }
        }

        impl<I, T: $crate::VecValue, C: $crate::cache::CachePolicy> $crate::TypedVec for $wrapper<I, T, C>
        where
            I: $crate::VecIndex,
            T: $value_trait,
        {
            type I = I;
            type T = T;
        }

        impl<I, T: $crate::VecValue, C: $crate::cache::CachePolicy> $crate::AnyStoredVec for $wrapper<I, T, C>
        where
            I: $crate::VecIndex,
            T: $value_trait,
        {
            #[inline]
            fn db_path(&self) -> ::std::path::PathBuf {
                self.0.db_path()
            }

            #[inline]
            fn region(&self) -> &::rawdb::Region {
                self.0.region()
            }

            #[inline]
            fn header(&self) -> &$crate::Header {
                self.0.header()
            }

            #[inline]
            fn mut_header(&mut self) -> &mut $crate::Header {
                self.0.mut_header()
            }

            #[inline]
            fn saved_stamped_changes(&self) -> u16 {
                self.0.saved_stamped_changes()
            }

            #[inline]
            fn db(&self) -> ::rawdb::Database {
                self.0.db()
            }

            #[inline]
            fn real_stored_len(&self) -> usize {
                self.0.real_stored_len()
            }

            #[inline]
            fn stored_len(&self) -> usize {
                self.0.stored_len()
            }

            #[inline]
            fn write(&mut self) -> $crate::Result<bool> {
                self.0.write()
            }

            #[inline]
            fn serialize_changes(&self) -> $crate::Result<Vec<u8>> {
                self.0.serialize_changes()
            }

            #[inline]
            fn any_stamped_write_with_changes(
                &mut self,
                stamp: $crate::Stamp,
            ) -> $crate::Result<()> {
                $crate::WritableVec::stamped_write_with_changes(&mut self.0, stamp)
            }

            fn any_save_rollback_state(&mut self) {
                $crate::WritableVec::save_rollback_state(&mut self.0)
            }

            fn any_remove_changes_above(&self, stamp: $crate::Stamp) -> $crate::Result<()> {
                self.0.any_remove_changes_above(stamp)
            }

            fn remove(self) -> $crate::Result<()> {
                self.0.remove()
            }

            fn any_truncate_if_needed_at(&mut self, index: usize) -> $crate::Result<()> {
                $crate::WritableVec::truncate_if_needed_at(&mut self.0, index)
            }

            fn any_reset(&mut self) -> $crate::Result<()> {
                $crate::WritableVec::reset(self)
            }
        }

        impl<I, T: $crate::VecValue, C: $crate::cache::CachePolicy> $crate::WritableVec<I, T> for $wrapper<I, T, C>
        where
            I: $crate::VecIndex,
            T: $value_trait,
        {
            #[inline]
            fn push(&mut self, value: T) {
                $crate::WritableVec::push(&mut self.0, value)
            }

            #[inline]
            fn pushed(&self) -> &[T] {
                $crate::WritableVec::pushed(&self.0)
            }

            #[inline]
            fn truncate_if_needed_at(&mut self, index: usize) -> $crate::Result<()> {
                self.0.truncate_if_needed_at(index)
            }

            #[inline]
            fn reset(&mut self) -> $crate::Result<()> {
                self.0.reset()
            }

            #[inline]
            fn reset_unsaved(&mut self) {
                self.0.reset_unsaved()
            }

            #[inline]
            fn is_dirty(&self) -> bool {
                self.0.is_dirty()
            }

            #[inline]
            fn stamped_write_with_changes(&mut self, stamp: $crate::Stamp) -> $crate::Result<()> {
                self.0.stamped_write_with_changes(stamp)
            }

            #[inline]
            fn rollback(&mut self) -> $crate::Result<()> {
                self.0.rollback()
            }

            fn find_rollback_files(
                &self,
            ) -> ::std::result::Result<
                ::std::collections::BTreeMap<$crate::Stamp, ::std::path::PathBuf>,
                $crate::Error,
            > {
                self.0.find_rollback_files()
            }

            fn save_rollback_state(&mut self) {
                self.0.save_rollback_state()
            }
        }

        impl<I, T: $crate::VecValue, C: $crate::cache::CachePolicy> $crate::ReadableCloneableVec<I, T> for $wrapper<I, T, C>
        where
            I: $crate::VecIndex,
            T: $value_trait,
        {
            #[inline]
            fn read_only_boxed_clone(&self) -> $crate::ReadableBoxedVec<I, T> {
                $crate::ReadableBoxedVec::new(self.0.read_only_clone())
            }
        }

        impl<I, T: $crate::VecValue, C: $crate::cache::CachePolicy> $crate::StoredVec for $wrapper<I, T, C>
        where
            I: $crate::VecIndex,
            T: $value_trait,
        {
            type ReadOnly = $read_only;

            #[inline]
            fn read_only_clone(&self) -> Self::ReadOnly {
                self.0.read_only_clone()
            }
        }

        impl<I, T: $crate::VecValue, C: $crate::cache::CachePolicy> $crate::ReadableVec<I, T> for $wrapper<I, T, C>
        where
            I: $crate::VecIndex,
            T: $value_trait,
        {
            fn read_cached_into_at(&self, from: usize, to: usize, out: &mut Vec<T>) -> bool {
                $crate::ReadableVec::<I, T>::read_cached_into_at(&self.0, from, to, out)
            }

            #[inline(always)]
            fn cursor_chunk_size(&self) -> usize {
                $crate::ReadableVec::<I, T>::cursor_chunk_size(&self.0)
            }

            #[inline(always)]
            fn collect_one_at(&self, index: usize) -> Option<T> {
                $crate::ReadableVec::<I, T>::collect_one_at(&self.0, index)
            }

            #[inline(always)]
            fn read_into_at(&self, from: usize, to: usize, buf: &mut Vec<T>) {
                $crate::ReadableVec::<I, T>::read_into_at(&self.0, from, to, buf)
            }

            #[inline(always)]
            fn for_each_chunk_at(&self, from: usize, to: usize, f: &mut dyn FnMut(usize, &[T])) {
                $crate::ReadableVec::<I, T>::for_each_chunk_at(&self.0, from, to, f)
            }

            #[inline(always)]
            fn read_sorted_into_at(&self, indices: &[usize], out: &mut Vec<T>) {
                $crate::ReadableVec::<I, T>::read_sorted_into_at(&self.0, indices, out)
            }

            #[inline]
            fn for_each_range_dyn_at(&self, from: usize, to: usize, f: &mut dyn FnMut(T)) {
                $crate::ReadableVec::<I, T>::for_each_range_dyn_at(&self.0, from, to, f)
            }

            #[inline]
            fn try_fold_range_at<B, E, F: FnMut(B, T) -> ::std::result::Result<B, E>>(
                &self,
                from: usize,
                to: usize,
                init: B,
                f: F,
            ) -> ::std::result::Result<B, E>
            where
                Self: Sized,
            {
                $crate::ReadableVec::<I, T>::try_fold_range_at(&self.0, from, to, init, f)
            }
        }
    };
    (@deref_mut yes, $wrapper:ident, $inner:ty) => {
        impl<I, T: $crate::VecValue, C: $crate::cache::CachePolicy> ::std::ops::DerefMut for $wrapper<I, T, C> {
            #[inline]
            fn deref_mut(&mut self) -> &mut Self::Target {
                &mut self.0
            }
        }
    };
    (@deref_mut no, $wrapper:ident, $inner:ty) => {};
}
macro_rules! impl_mutable_raw_vec {
    ($vec:ident, $value:ident, $strategy:ident) => {
        const _: () = {
            use crate::variants::mutable::raw::MutableRawVec;
            use crate::*;
            use std::collections::{BTreeMap, BTreeSet};
            impl<I, T> MutableRawVec for $vec<I, T>
            where
                I: VecIndex,
                T: $value,
            {
                type Reader = VecReader<I, T, $strategy<T>>;

                fn import_strict_with(options: ImportOptions) -> Result<Self> {
                    Ok(Self(ReadWriteRawVec::import_strict_with(
                        options,
                        Self::FORMAT,
                    )?))
                }

                #[inline]
                fn reader(&self) -> Self::Reader {
                    self.0.reader()
                }

                #[inline(always)]
                fn reader_len(reader: &Self::Reader) -> usize {
                    reader.stored_len()
                }

                #[inline(always)]
                fn read_stored(reader: &Self::Reader, index: usize) -> T {
                    reader.get_at(index)
                }

                #[inline(always)]
                fn rollback_len(&self) -> usize {
                    self.0.base().prev_stored_len()
                }

                #[inline]
                fn pushed_mut(&mut self) -> &mut Vec<T> {
                    self.0.mut_pushed()
                }

                #[inline]
                fn reserve_pushed(&mut self, additional: usize) {
                    self.0.reserve_pushed(additional);
                }

                fn write_updates(&mut self, updated: BTreeMap<usize, T>) -> Result<()> {
                    self.0.write_updates(updated)
                }

                fn append_previous_values(
                    &self,
                    indices: &[usize],
                    previous: &BTreeMap<usize, T>,
                    bytes: &mut Vec<u8>,
                ) {
                    self.0.append_previous_values(indices, previous, bytes)
                }

                fn parse_mutable_changes(
                    bytes: &[u8],
                ) -> crate::Result<(Vec<(usize, T)>, BTreeSet<usize>)> {
                    ReadWriteRawVec::<I, T, $strategy<T>>::parse_mutable_changes(bytes)
                }

                fn save_change_file(&self, stamp: Stamp, bytes: &[u8]) -> crate::Result<()> {
                    self.0.base().save_change_file(stamp, bytes)
                }

                fn read_current_change_file(&self) -> crate::Result<Vec<u8>> {
                    self.0.base().read_current_change_file()
                }

                fn save_previous(&mut self) {
                    self.0.base_mut().save_prev();
                }

                fn save_previous_for_rollback(&mut self) {
                    self.0.base_mut().save_prev_for_rollback();
                }
            }
        };
    };
}

/// Implements `WritableVec` for the raw and compressed read-write vecs, which share their base
/// vec, cache updates and change files.
macro_rules! impl_read_write_writable {
    ($vec:ident, $strategy:ident) => {
        impl<I, T, S, C: $crate::cache::CachePolicy> $crate::WritableVec<I, T> for $vec<I, T, S, C>
        where
            I: $crate::VecIndex,
            T: $crate::VecValue,
            S: $strategy<T>,
        {
            #[inline]
            fn push(&mut self, value: T) {
                self.base.mut_pushed().push(value);
            }

            #[inline]
            fn pushed(&self) -> &[T] {
                self.base.pushed()
            }

            fn truncate_if_needed_at(&mut self, index: usize) -> $crate::Result<()> {
                self.header().check_writable()?;
                self.with_cache_update(index, |this| {
                    if this.base.truncate_pushed(index) {
                        this.base.update_stored_len(index);
                    }
                    Ok(())
                })
            }

            fn reset(&mut self) -> $crate::Result<()> {
                let guard = self.header().begin_write()?;
                let result = self.with_cache_update(0, |this| {
                    this.base.truncate_pushed(0);
                    this.base.update_stored_len(0);
                    this.base.reset_base()
                });
                guard.finish(result)
            }

            fn reset_unsaved(&mut self) {
                self.with_cache_update(0, |this| {
                    this.base.reset_unsaved_base();
                    Ok(())
                })
                .expect("resetting unsaved state cannot fail");
            }

            fn is_dirty(&self) -> bool {
                !self.base.pushed().is_empty()
            }

            fn stamped_write_with_changes(&mut self, stamp: $crate::Stamp) -> $crate::Result<()> {
                self.header().check_writable()?;
                if self.base.saved_stamped_changes() == 0 {
                    return self.stamped_write(stamp);
                }

                let guard = self.header().begin_write()?;
                let data = self.serialize_changes()?;
                self.base.save_change_file(stamp, &data)?;
                self.stamped_write(stamp)?;
                self.base.save_prev();

                guard.finish(Ok(()))
            }

            fn rollback(&mut self) -> $crate::Result<()> {
                self.header().check_writable()?;
                let bytes = self.base.read_current_change_file()?;
                let change = $crate::ReadWriteBaseVec::<I, T>::parse_change_data::<S>(
                    &mut $crate::ChangeCursor::new(&bytes),
                )?;
                let from = self.base.rollback_from(&change);
                self.with_cache_update(from, |this| {
                    this.base.apply_rollback(change);
                    Ok(())
                })
            }

            fn find_rollback_files(
                &self,
            ) -> $crate::Result<::std::collections::BTreeMap<$crate::Stamp, ::std::path::PathBuf>>
            {
                self.base.find_rollback_files()
            }

            fn save_rollback_state(&mut self) {
                self.header().assert_writable();
                self.base.save_prev_for_rollback();
            }
        }
    };
}
