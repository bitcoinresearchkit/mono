use bitview_primitives::{CentsFract, SatsFract};
use brk_types::{Bitcoin, Cents, Dollars, Sats};

use crate::{
    LazyPerBlock, LazyRollingAvgFromHeight, PricedAvgUsd, PricedRollingAvgFromHeight, Value,
};

pub type LazyRollingAvgAmountFromHeight = Value<
    LazyRollingAvgFromHeight<Sats>,
    PricedRollingAvgFromHeight<Cents>,
    LazyPerBlock<Bitcoin, SatsFract>,
    PricedAvgUsd<LazyPerBlock<Dollars, CentsFract>>,
>;
