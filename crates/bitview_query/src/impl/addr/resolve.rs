use bitview_primitives::{AddrHash, Lengths, TypeIndex};
use brk_types::{Address, AddressBytes, OutputType};

use super::parse_addr;
use crate::{Error, Query, Result};

impl Query {
    pub(crate) fn resolve_addr(
        &self,
        addr: &Address,
        safe: Lengths,
    ) -> Result<(OutputType, TypeIndex)> {
        let bytes = parse_addr(addr)?;
        self.find_addr_bytes(&bytes, safe)?
            .ok_or(Error::UnknownAddress)
    }

    /// The address's type and index, if the published state `safe` has it.
    pub(crate) fn find_addr_bytes(
        &self,
        bytes: &AddressBytes,
        safe: Lengths,
    ) -> Result<Option<(OutputType, TypeIndex)>> {
        let output_type = OutputType::from(bytes);
        let hash = AddrHash::from(bytes);
        Ok(self
            .indexer()
            .stores()
            .addr_index(output_type, &hash)?
            .filter(|type_index| *type_index < safe.to_type_index(output_type))
            .map(|type_index| (output_type, type_index)))
    }
}
