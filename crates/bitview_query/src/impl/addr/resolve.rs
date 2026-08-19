use std::str::FromStr;

use brk_error::Error;
use brk_types::{Addr, AddrBytes, AddrHash, OutputType, TypeIndex};

use crate::Query;

impl Query {
    fn resolve_addr(&self, addr: &Addr) -> brk_error::Result<(OutputType, TypeIndex)> {
        let bytes = AddrBytes::from_str(addr)?;
        let output_type = OutputType::from(&bytes);
        let hash = AddrHash::from(&bytes);
        let type_index = self.type_index_for(output_type, &hash)?;
        Ok((output_type, type_index))
    }

    /// Lookup the per-type index of an address by `(output_type, hash)`.
    /// Returns `UnknownAddr` if the hash is absent from the type's index.
    fn type_index_for(
        &self,
        output_type: OutputType,
        hash: &AddrHash,
    ) -> brk_error::Result<TypeIndex> {
        self.indexer()
            .stores()
            .addr_index(output_type, hash)?
            .ok_or(Error::UnknownAddr)
    }
}

#[inline]
pub fn resolve_addr(query: &Query, addr: &Addr) -> brk_error::Result<(OutputType, TypeIndex)> {
    query.resolve_addr(addr)
}

#[inline]
pub fn type_index_for(
    query: &Query,
    output_type: OutputType,
    hash: &AddrHash,
) -> brk_error::Result<TypeIndex> {
    query.type_index_for(output_type, hash)
}
