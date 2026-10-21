use brk_types::{Bitcoin, Cents, Dollars, Sats};

use crate::{LazyPerBlock, LazyRollingSumFromHeight, PricedRollingSumFromHeight, Value};

pub type LazyRollingSumAmountFromHeight = Value<
    LazyRollingSumFromHeight<Sats>,
    PricedRollingSumFromHeight<Cents>,
    LazyPerBlock<Bitcoin, Sats>,
    LazyPerBlock<Dollars, Cents>,
>;
