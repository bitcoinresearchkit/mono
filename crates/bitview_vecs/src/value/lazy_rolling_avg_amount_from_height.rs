use bitview_primitives::{CentsFract, SatsFract};
use brk_types::{Bitcoin, Cents, Dollars, Sats};

use crate::{LazyPerBlock, LazyRollingAvgFromHeight, PricedRollingAvgFromHeight, Value};

pub type LazyRollingAvgAmountFromHeight = Value<
    LazyRollingAvgFromHeight<Sats>,
    PricedRollingAvgFromHeight<Cents>,
    LazyPerBlock<Bitcoin, SatsFract>,
    LazyPerBlock<Dollars, CentsFract>,
>;
