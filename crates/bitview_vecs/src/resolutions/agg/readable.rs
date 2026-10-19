use super::{AggFold, LazyAggVec};
use vecdb::{ReadableVec, VecIndex, VecValue};

impl<I, O, S1I, S1T, Strat> ReadableVec<I, O> for LazyAggVec<I, O, S1I, S1T, Strat>
where
    I: VecIndex,
    O: VecValue,
    S1I: VecIndex,
    S1T: VecValue,
    Strat: AggFold<O, S1I, S1T>,
{
    fn read_into_at(&self, from: usize, to: usize, buf: &mut Vec<O>) {
        let mapping = self.mapping.load();
        let to = to.min(mapping.len());
        if from >= to {
            return;
        }
        buf.reserve(to - from);
        Strat::fold(&*self.source, &mapping, from, to, (), |(), v| buf.push(v));
    }

    fn for_each_range_dyn_at(&self, from: usize, to: usize, f: &mut dyn FnMut(O)) {
        let mapping = self.mapping.load();
        let to = to.min(mapping.len());
        if from >= to {
            return;
        }
        Strat::fold(&*self.source, &mapping, from, to, (), |(), v| f(v));
    }

    #[inline]
    fn try_fold_range_at<B, E, F: FnMut(B, O) -> Result<B, E>>(
        &self,
        from: usize,
        to: usize,
        init: B,
        f: F,
    ) -> Result<B, E>
    where
        Self: Sized,
    {
        let mapping = self.mapping.load();
        let to = to.min(mapping.len());
        if from >= to {
            return Ok(init);
        }
        Strat::try_fold(&*self.source, &mapping, from, to, init, f)
    }

    #[inline]
    fn collect_one_at(&self, index: usize) -> Option<O> {
        let mapping = self.mapping.load();
        if index >= mapping.len() {
            return None;
        }
        Strat::collect_one(&*self.source, &mapping, index)
    }

    fn read_sorted_into_at(&self, indices: &[usize], out: &mut Vec<O>) {
        if indices.is_empty() {
            return;
        }
        let mapping = self.mapping.load();
        let indices = &indices[..indices.partition_point(|&index| index < mapping.len())];
        if indices.is_empty() {
            return;
        }
        if indices.len() > 1 && indices.windows(2).all(|pair| pair[1] == pair[0] + 1) {
            out.reserve(indices.len());
            Strat::fold(
                &*self.source,
                &mapping,
                indices[0],
                indices[indices.len() - 1] + 1,
                (),
                |(), value| out.push(value),
            );
            return;
        }
        Strat::read_sorted_into(&*self.source, &mapping, indices, out);
    }
}
