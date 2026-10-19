use std::{convert::Infallible, ops::Range, result::Result as FoldResult};

use crate::VecValue;

use super::{Cache, Request};

impl<T: VecValue> Cache<T> {
    pub(crate) fn get_source<'a>(
        &self,
        index: usize,
        source: impl FnOnce() -> (usize, &'a [T]),
        mut load: impl FnMut(usize, &[Range<usize>]) -> Vec<(usize, Vec<T>)>,
    ) -> Option<T> {
        self.read_scope(|| {
            let (stored, pushed) = source();
            if index >= stored {
                return pushed.get(index - stored).cloned();
            }
            Some(self.get(index, |ranges| load(stored, ranges)))
        })
    }

    pub(crate) fn read_source<'a>(
        &self,
        request: Request<'_>,
        out: &mut Vec<T>,
        source: impl FnOnce() -> (usize, &'a [T]),
        mut load: impl FnMut(usize, &[Range<usize>]) -> Vec<(usize, Vec<T>)>,
    ) {
        self.read_scope(|| {
            let (stored, pushed) = source();
            let request = request.clamp(stored + pushed.len());
            self.read(request.clamp(stored), out, |ranges| load(stored, ranges));
            match request {
                Request::Range(from, to) if to > stored && from < to => {
                    out.extend_from_slice(&pushed[from.max(stored) - stored..to - stored]);
                }
                Request::Sorted(indices) => {
                    let split = indices.partition_point(|&i| i < stored);
                    out.extend(indices[split..].iter().map(|&i| pushed[i - stored].clone()));
                }
                _ => {}
            }
        });
    }

    pub(crate) fn for_each_source<'a>(
        &self,
        from: usize,
        to: usize,
        source: impl FnOnce() -> (usize, &'a [T]),
        load: impl FnMut(usize, &[Range<usize>]) -> Vec<(usize, Vec<T>)>,
        mut each: impl FnMut(usize, &[T]),
    ) {
        let Ok(()) = self.try_for_each_source(from, to, source, load, |at, values| {
            each(at, values);
            Ok::<_, Infallible>(())
        });
    }

    #[inline]
    pub(crate) fn try_fold_source<'a, B, E>(
        &self,
        from: usize,
        to: usize,
        init: B,
        source: impl FnOnce() -> (usize, &'a [T]),
        load: impl FnMut(usize, &[Range<usize>]) -> Vec<(usize, Vec<T>)>,
        mut fold: impl FnMut(B, T) -> FoldResult<B, E>,
    ) -> FoldResult<B, E> {
        let mut acc = Some(init);
        self.try_for_each_source(from, to, source, load, |_, values| {
            acc = Some(
                values
                    .iter()
                    .cloned()
                    .try_fold(acc.take().unwrap(), &mut fold)?,
            );
            Ok(())
        })?;
        Ok(acc.unwrap())
    }

    fn try_for_each_source<'a, E>(
        &self,
        from: usize,
        to: usize,
        source: impl FnOnce() -> (usize, &'a [T]),
        mut load: impl FnMut(usize, &[Range<usize>]) -> Vec<(usize, Vec<T>)>,
        mut each: impl FnMut(usize, &[T]) -> FoldResult<(), E>,
    ) -> FoldResult<(), E> {
        self.read_scope(|| {
            let (stored, pushed) = source();
            let to = to.min(stored + pushed.len());
            self.try_for_each_chunk(
                from,
                to.min(stored),
                |ranges| load(stored, ranges),
                &mut each,
            )?;
            if to > stored && from < to {
                let from = from.max(stored);
                each(from, &pushed[from - stored..to - stored])?;
            }
            Ok(())
        })
    }
}
