use crate::{EmptyAddrData, ExtendedEmptyAddrIndex, FundedAddrIndex};

/// Decoded form of an address's four-byte primary state.
#[derive(Debug, Clone)]
pub enum DecodedAddrState {
    Empty(EmptyAddrData),
    ExtendedEmpty(ExtendedEmptyAddrIndex),
    Funded(FundedAddrIndex),
}
