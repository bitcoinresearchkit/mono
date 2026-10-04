use std::iter;

use bitview_traversable::{Traversable, TreeNode, make_leaf};
use schemars::JsonSchema;
use serde::Serialize;
use vecdb::{AnyExportableVec, Formattable, VecIndex, VecValue};

use super::{AggFold, LazyAggVec};

impl<I, O, S1I, S1T, Strat> Traversable for LazyAggVec<I, O, S1I, S1T, Strat>
where
    I: VecIndex,
    O: VecValue + Formattable + Serialize + JsonSchema,
    S1I: VecIndex,
    S1T: VecValue,
    Strat: AggFold<O, S1I, S1T>,
{
    fn iter_any_exportable(&self) -> impl Iterator<Item = &dyn AnyExportableVec> {
        iter::once(self as &dyn AnyExportableVec)
    }

    fn to_tree_node(&self) -> TreeNode {
        make_leaf(self)
    }
}
