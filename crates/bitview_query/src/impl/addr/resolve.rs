use bitview_primitives::{AddrHash, TypeIndex};
use brk_types::{Addr, AddrBytes, OutputType};

use super::parse_addr;
use crate::{Error, Query, Result};

impl Query {
    pub(crate) fn missing_addr(&self) -> Error {
        if self.indexer().publication().try_read().is_none() {
            Error::StateUpdating
        } else {
            Error::UnknownAddr
        }
    }

    pub(crate) fn resolve_addr(&self, addr: &Addr) -> Result<(OutputType, TypeIndex)> {
        let bytes = parse_addr(addr)?;
        self.resolve_addr_bytes(&bytes)
    }
    pub(crate) fn resolve_addr_bytes(&self, bytes: &AddrBytes) -> Result<(OutputType, TypeIndex)> {
        self.find_addr_bytes(bytes)?
            .ok_or_else(|| self.missing_addr())
    }
    pub(crate) fn find_addr_bytes(
        &self,
        bytes: &AddrBytes,
    ) -> Result<Option<(OutputType, TypeIndex)>> {
        let output_type = OutputType::from(bytes);
        let hash = AddrHash::from(bytes);
        Ok(self
            .indexer()
            .stores()
            .addr_index(output_type, &hash)?
            .filter(|type_index| *type_index < self.safe_lengths().to_type_index(output_type))
            .map(|type_index| (output_type, type_index)))
    }
}
