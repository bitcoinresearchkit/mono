use bitview_primitives::{CentsFract, SatsFract};
use brk_types::{Bitcoin, Cents, Dollars, Sats};

use crate::{LazyPerBlock, LazyRollingAvgFromHeight, Value};

pub type LazyRollingAvgAmountFromHeight = Value<
    LazyRollingAvgFromHeight<Sats>,
    LazyRollingAvgFromHeight<Cents>,
    LazyPerBlock<Bitcoin, SatsFract>,
    LazyPerBlock<Dollars, CentsFract>,
>;
