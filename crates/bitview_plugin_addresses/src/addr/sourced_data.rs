use bitview_primitives::{ExtendedEmptyAddressIndex, FundedAddressData, FundedAddressIndex};
use derive_more::{Deref, DerefMut};

/// The persistent representation address data came from.
#[derive(Debug, Clone, Copy, Default)]
pub enum AddrOrigin {
    #[default]
    New,
    Funded(FundedAddressIndex),
    InlineEmpty,
    ExtendedEmpty(ExtendedEmptyAddressIndex),
}

/// Address data and the persistent representation it came from.
#[derive(Debug, Clone, Default, Deref, DerefMut)]
pub struct SourcedAddrData {
    pub origin: AddrOrigin,
    #[deref]
    #[deref_mut]
    pub data: FundedAddressData,
}
