/// A concrete catalog binding. Names are never reconstructed from a template.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct CatalogNode {
    pub(crate) type_id: usize,
    pub(crate) value: CatalogValue,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum CatalogValue {
    Leaf(String),
    Branch(Vec<usize>),
}
