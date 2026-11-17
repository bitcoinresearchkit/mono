use bitview_plugin_age::Vecs as Age;
use bitview_plugin_price::Vecs as Price;
use brk_types::Height;

pub struct Dependencies<'a> {
    pub from: Height,
    pub age: &'a Age,
    pub price: &'a Price,
}
