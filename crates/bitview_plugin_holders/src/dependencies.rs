use bitview_plugin_age::Vecs as Age;
use bitview_plugin_mappings::Vecs as Mappings;
use bitview_plugin_price::Vecs as Price;
use brk_types::Height;
use statedb::Reader;

pub struct Dependencies<'a> {
    pub history: &'a Reader<'a>,
    pub from: Height,
    pub age: &'a Age,
    pub mappings: &'a Mappings,
    pub price: &'a Price,
}
