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
/// - `Deref` / `DerefMut`
/// - `ImportableVec`
/// - `AnyVec`
/// - `TypedVec`
/// - `AnyStoredVec`
/// - `WritableVec`
/// - `ReadableVec` (delegates `for_each_range_dyn` / `fold_range` to inner)
macro_rules! impl_vec_wrapper {
    ($wrapper:ident, $inner:ty, $value_trait:ident, $format:expr, $read_only:ty) => {
        impl<I, T> ::std::ops::Deref for $wrapper<I, T> {
            type Target = $inner;

            #[inline]
            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }

        impl<I, T> ::std::ops::DerefMut for $wrapper<I, T> {
            #[inline]
            fn deref_mut(&mut self) -> &mut Self::Target {
                &mut self.0
            }
        }

        impl<I, T> $crate::ImportableVec for $wrapper<I, T>
        where
            I: $crate::VecIndex,
            T: $value_trait,
        {
            fn import(
                db: &::rawdb::Database,
                name: &str,
                version: $crate::Version,
            ) -> $crate::Result<Self> {
                Self::import_with((db, name, version).into())
            }

            fn import_with(options: $crate::ImportOptions) -> $crate::Result<Self> {
                Ok(Self(<$inner>::import_with(options, $format)?))
            }

            fn forced_import(
                db: &::rawdb::Database,
                name: &str,
                version: $crate::Version,
            ) -> $crate::Result<Self> {
                Self::forced_import_with((db, name, version).into())
            }

            fn forced_import_with(options: $crate::ImportOptions) -> $crate::Result<Self> {
                Ok(Self(<$inner>::forced_import_with(options, $format)?))
            }
        }

        impl<I, T> $crate::AnyVec for $wrapper<I, T>
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

        impl<I, T> $crate::TypedVec for $wrapper<I, T>
        where
            I: $crate::VecIndex,
            T: $value_trait,
        {
            type I = I;
            type T = T;
        }

        impl<I, T> $crate::AnyStoredVec for $wrapper<I, T>
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

        impl<I, T> $crate::WritableVec<I, T> for $wrapper<I, T>
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

        impl<I, T> $crate::ReadableCloneableVec<I, T> for $wrapper<I, T>
        where
            I: $crate::VecIndex,
            T: $value_trait,
        {
            #[inline]
            fn read_only_boxed_clone(&self) -> $crate::ReadableBoxedVec<I, T> {
                $crate::ReadableBoxedVec::new(self.0.read_only_clone())
            }
        }

        impl<I, T> $crate::StoredVec for $wrapper<I, T>
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

        impl<I, T> $crate::ReadableVec<I, T> for $wrapper<I, T>
        where
            I: $crate::VecIndex,
            T: $value_trait,
        {
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

            #[inline]
            fn for_each_range_dyn_at(&self, from: usize, to: usize, f: &mut dyn FnMut(T)) {
                $crate::ReadableVec::<I, T>::for_each_range_dyn_at(&self.0, from, to, f)
            }

            #[inline]
            fn fold_range_at<B, F: FnMut(B, T) -> B>(
                &self,
                from: usize,
                to: usize,
                init: B,
                f: F,
            ) -> B
            where
                Self: Sized,
            {
                $crate::ReadableVec::<I, T>::fold_range_at(&self.0, from, to, init, f)
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
}

pub(crate) use impl_vec_wrapper;
