use bitview_plugin_indexer::SafeLengths;
use bitview_primitives::{AddrHash, TypeIndex};
use brk_types::{Address, AddressBytes, OutputType};

use super::parse_addr;
use crate::{Error, Query, Result, check_bound};

impl Query {
    pub(crate) fn resolve_addr(
        &self,
        addr: &Address,
        pin: &SafeLengths,
    ) -> Result<(OutputType, TypeIndex)> {
        let bytes = parse_addr(addr)?;
        self.find_addr_bytes(&bytes, pin)?
            .ok_or(Error::UnknownAddress)
    }

    /// The address's type and index, if the publication `pin` has it.
    pub(crate) fn find_addr_bytes(
        &self,
        bytes: &AddressBytes,
        pin: &SafeLengths,
    ) -> Result<Option<(OutputType, TypeIndex)>> {
        let output_type = OutputType::from(bytes);
        let hash = AddrHash::from(bytes);
        let Some(type_index) = self.indexer().stores().addr_index(output_type, &hash)? else {
            return Ok(None);
        };
        let bound = pin.lengths().to_type_index(output_type);
        check_bound(pin, type_index, bound)?;
        Ok((type_index < bound).then_some((output_type, type_index)))
    }
}
