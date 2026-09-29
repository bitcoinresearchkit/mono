use brk_types::{Cents, Height};
pub struct ComputeContext<'a> {
    pub starting_height: Height,
    pub last_height: Height,
    pub height_to_price: &'a [Cents],
}
impl ComputeContext<'_> {
    pub fn price_at(&self, height: Height) -> Cents {
        self.height_to_price[usize::from(height)]
    }
}
