/// A structural record family: ordered catalog fields, parameterized by their child types.
#[derive(Debug, Clone)]
pub struct CatalogFamily {
    pub(crate) name: String,
    /// Canonical catalog path (keys) the family is named after: shallowest, then smallest.
    pub(crate) path: Vec<String>,
    /// Catalog field key and its type-parameter slot (slots are numbered densely from 0).
    pub(crate) fields: Vec<(String, usize)>,
}

impl CatalogFamily {
    pub(crate) fn parameters(&self) -> usize {
        self.fields
            .iter()
            .map(|(_, slot)| slot + 1)
            .max()
            .unwrap_or(0)
    }
}
