use brk_types::Cents;

use bitview_compute::LazyFiatPerBlock;

#[derive(Clone)]
pub struct UnrealizedAggregateSources {
    pub gross_pnl: LazyFiatPerBlock<Cents>,
    pub invested_capital_in_profit: LazyFiatPerBlock<Cents>,
    pub invested_capital_in_loss: LazyFiatPerBlock<Cents>,
}
