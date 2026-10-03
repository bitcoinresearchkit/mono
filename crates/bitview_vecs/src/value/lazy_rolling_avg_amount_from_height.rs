use bitview_primitives::StoredF32;
use brk_types::{Bitcoin, Cents, Dollars, Sats};

use crate::{LazyPerBlock, LazyRollingAvgFromHeight, Value};

pub type LazyRollingAvgAmountFromHeight = Value<
    LazyRollingAvgFromHeight<Sats>,
    LazyRollingAvgFromHeight<Cents>,
    LazyPerBlock<Bitcoin, StoredF32>,
    LazyPerBlock<Dollars, StoredF32>,
>;
