use crate::{EmptyAddressData, ExtendedEmptyAddressIndex, FundedAddressIndex};

/// Decoded form of an address's four-byte primary state.
#[derive(Debug, Clone)]
pub enum DecodedAddrState {
    Empty(EmptyAddressData),
    ExtendedEmpty(ExtendedEmptyAddressIndex),
    Funded(FundedAddressIndex),
}
