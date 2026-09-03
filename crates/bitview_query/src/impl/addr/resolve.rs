use std::str::FromStr;

use brk_error::{Error, Result};
use brk_types::{Addr, AddrBytes, AddrHash, OutputType, TypeIndex};

use crate::Query;

impl Query {
    fn resolve_addr(&self, addr: &Addr) -> Result<(OutputType, TypeIndex)> {
        let bytes = AddrBytes::from_str(addr)?;
        self.resolve_addr_bytes(&bytes)
    }

    fn resolve_addr_bytes(&self, bytes: &AddrBytes) -> Result<(OutputType, TypeIndex)> {
        self.find_addr_bytes(bytes)?.ok_or(Error::UnknownAddr)
    }

    fn find_addr_bytes(&self, bytes: &AddrBytes) -> Result<Option<(OutputType, TypeIndex)>> {
        let output_type = OutputType::from(bytes);
        let hash = AddrHash::from(bytes);
        Ok(self
            .indexer()
            .stores()
            .addr_index(output_type, &hash)?
            .map(|type_index| (output_type, type_index)))
    }

    /// Lookup the per-type index of an address by `(output_type, hash)`.
    /// Returns `UnknownAddr` if the hash is absent from the type's index.
    fn type_index_for(&self, output_type: OutputType, hash: &AddrHash) -> Result<TypeIndex> {
        self.indexer()
            .stores()
            .addr_index(output_type, hash)?
            .ok_or(Error::UnknownAddr)
    }
}

#[inline]
pub fn resolve_addr(query: &Query, addr: &Addr) -> Result<(OutputType, TypeIndex)> {
    query.resolve_addr(addr)
}

#[inline]
pub(super) fn resolve_addr_bytes(
    query: &Query,
    bytes: &AddrBytes,
) -> Result<(OutputType, TypeIndex)> {
    query.resolve_addr_bytes(bytes)
}

#[inline]
pub fn type_index_for(
    query: &Query,
    output_type: OutputType,
    hash: &AddrHash,
) -> Result<TypeIndex> {
    query.type_index_for(output_type, hash)
}

#[inline]
pub(super) fn find_addr_bytes(
    query: &Query,
    bytes: &AddrBytes,
) -> Result<Option<(OutputType, TypeIndex)>> {
    query.find_addr_bytes(bytes)
}
