use brk_types::{EmptyAddrData, FundedAddrData};

use crate::addr::{AddrTypeToTypeIndexMap, SourcedAddrData};

/// Context for selecting one address type's cached data.
pub struct AddrLookup<'a> {
    pub funded: &'a mut AddrTypeToTypeIndexMap<SourcedAddrData<FundedAddrData>>,
    pub empty: &'a mut AddrTypeToTypeIndexMap<SourcedAddrData<EmptyAddrData>>,
}
