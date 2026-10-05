use bitview_primitives::FundedAddrData;

use crate::addr::{AddrTypeToTypeIndexMap, SourcedAddrData};

/// Context for selecting one address type's cached data.
pub struct AddrLookup<'a> {
    pub addrs: &'a mut AddrTypeToTypeIndexMap<SourcedAddrData<FundedAddrData>>,
}
