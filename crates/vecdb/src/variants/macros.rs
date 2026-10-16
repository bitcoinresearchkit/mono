/// Implements `AnyStoredVec` and `WritableVec` for the raw and compressed read-write vecs, which
/// share their base vec, cache updates and change files. Each format supplies `write_pushed`
/// (persists the pushed values and a pending truncation) and `written_len` (rows on disk).
macro_rules! impl_read_write_storage {
    ($vec:ident, $strategy:ident) => {
        impl<I, T, S, C: $crate::cache::CachePolicy> $crate::AnyStoredVec for $vec<I, T, S, C>
        where
            I: $crate::VecIndex,
            T: $crate::VecValue,
            S: $strategy<T>,
        {
            #[inline]
            fn db_path(&self) -> ::std::path::PathBuf {
                self.base.db_path()
            }

            #[inline]
            fn region(&self) -> &::rawdb::Region {
                self.base.region()
            }

            #[inline]
            fn header(&self) -> &$crate::Header {
                self.base.header()
            }

            #[inline]
            fn mut_header(&mut self) -> &mut $crate::Header {
                self.base.mut_header()
            }

            #[inline]
            fn saved_stamped_changes(&self) -> u16 {
                self.base.saved_stamped_changes()
            }

            #[inline]
            fn db(&self) -> ::rawdb::Database {
                self.base.db()
            }

            #[inline]
            fn real_stored_len(&self) -> usize {
                self.written_len()
            }

            #[inline]
            fn stored_len(&self) -> usize {
                self.base.stored_len()
            }

            fn write(&mut self) -> $crate::Result<bool> {
                let guard = self.header().begin_write()?;
                let result = self.write_pushed();
                if result.is_ok() {
                    self.base.unwritten_baseline = false;
                }
                guard.finish(result)
            }

            fn serialize_changes(&self) -> $crate::Result<Vec<u8>> {
                self.base
                    .serialize_changes::<S>(|from, to| self.collect_stored_range(from, to))
            }

            fn any_save_changes(&self, stamp: $crate::Stamp) -> $crate::Result<()> {
                self.base
                    .save_change_file(stamp, &self.serialize_changes()?)
            }

            fn any_write_checkpoint(&mut self, stamp: $crate::Stamp) -> $crate::Result<()> {
                self.stamped_write(stamp)?;
                if self.base.saved_stamped_changes() > 0 {
                    self.base.save_prev();
                }
                Ok(())
            }

            fn any_remove_changes_above(&self, stamp: $crate::Stamp) -> $crate::Result<()> {
                self.base.remove_changes_above(stamp)
            }

            fn remove(self) -> $crate::Result<()> {
                Self::remove(self)
            }

            fn any_truncate_if_needed_at(&mut self, index: usize) -> $crate::Result<()> {
                $crate::WritableVec::truncate_if_needed_at(self, index)
            }

            fn any_reset(&mut self) -> $crate::Result<()> {
                $crate::WritableVec::reset(self)
            }
        }

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

            /// Drops pushed values and a pending truncation: back to the written state, or to an
            /// unwritten rollback's or reset's.
            fn reset_unsaved(&mut self) {
                self.with_cache_update(0, |this| {
                    let written_len = $crate::AnyStoredVec::real_stored_len(this);
                    this.base.reset_unsaved_base(written_len);
                    Ok(())
                })
                .expect("resetting unsaved state cannot fail");
            }

            fn is_dirty(&self) -> bool {
                self.base.unwritten_baseline
                    || !self.base.pushed().is_empty()
                    || self.base.stored_len() != $crate::AnyStoredVec::real_stored_len(self)
            }

            fn rollback(&mut self) -> $crate::Result<()> {
                self.header().check_writable()?;
                let bytes = self.base.read_current_change_file()?;
                let change = $crate::ReadWriteBaseVec::<I, T>::parse_change_data::<S>(
                    &mut $crate::ChangeCursor::new(&bytes),
                )?;
                self.rollback_change(change)
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

        impl<I, T, S, C: $crate::cache::CachePolicy> $vec<I, T, S, C>
        where
            I: $crate::VecIndex,
            T: $crate::VecValue,
            S: $strategy<T>,
        {
            /// Undoes one parsed change record (see [`$crate::ReadWriteBaseVec::apply_rollback`]).
            pub(crate) fn rollback_change(
                &mut self,
                change: $crate::ChangeData<T>,
            ) -> $crate::Result<()> {
                let from = self.base.rollback_from(&change);
                self.with_cache_update(from, |this| {
                    this.base.apply_rollback(change);
                    Ok(())
                })
            }
        }
    };
}

/// Implements `ReadableVec` for a raw or compressed stored vec, or a read-only clone of one,
/// through its [`StoredReads`](crate::variants::stored_reads::StoredReads).
macro_rules! impl_stored_readable {
    ($vec:ident, $strategy:ident) => {
        impl<I, T, S, C: $crate::cache::CachePolicy> $crate::ReadableVec<I, T> for $vec<I, T, S, C>
        where
            I: $crate::VecIndex,
            T: $crate::VecValue,
            S: $strategy<T>,
        {
            #[inline]
            fn read_cached_into_at(&self, from: usize, to: usize, out: &mut Vec<T>) -> bool {
                $crate::variants::stored_reads::read_cached(self, from, to, out)
            }

            #[inline(always)]
            fn cursor_chunk_size(&self) -> usize {
                <Self as $crate::variants::stored_reads::StoredReads<T>>::CHUNK_SIZE
            }

            #[inline(always)]
            fn collect_one_at(&self, index: usize) -> Option<T> {
                $crate::variants::stored_reads::collect_one(self, index)
            }

            #[inline(always)]
            fn read_into_at(&self, from: usize, to: usize, buf: &mut Vec<T>) {
                $crate::variants::stored_reads::read_into(self, from, to, buf)
            }

            #[inline]
            fn read_sorted_into_at(&self, indices: &[usize], out: &mut Vec<T>) {
                $crate::variants::stored_reads::read_sorted(self, indices, out)
            }

            fn for_each_chunk_at(&self, from: usize, to: usize, f: &mut dyn FnMut(usize, &[T])) {
                $crate::variants::stored_reads::for_each_chunk(self, from, to, f)
            }

            #[inline]
            fn for_each_range_dyn_at(&self, from: usize, to: usize, f: &mut dyn FnMut(T)) {
                self.fold_range_at(from, to, (), |(), v| f(v));
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
                $crate::variants::stored_reads::try_fold(self, from, to, init, f)
            }
        }
    };
}
