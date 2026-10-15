use schemars::JsonSchema;
use serde::Serialize;
use vecdb::{Formattable, VecIndex, VecValue};

use super::{AggFold, LazyAggVec};

bitview_traversable::leaf!([I, O, S1I, S1T, Strat] LazyAggVec<I, O, S1I, S1T, Strat> where
    I: VecIndex,
    O: VecValue + Formattable + Serialize + JsonSchema,
    S1I: VecIndex,
    S1T: VecValue,
    Strat: AggFold<O, S1I, S1T>,
);
