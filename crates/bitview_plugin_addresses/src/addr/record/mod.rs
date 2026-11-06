//! The records the address-state vecs store: each address's four-byte state, the funded and
//! empty records it points to, and their indexes.

mod addr_state;
mod decoded_addr_state;
mod empty_addr_data;
mod extended_empty_addr_index;
mod funded_addr_data;
mod funded_addr_data_compact;
mod funded_addr_index;

pub use addr_state::AddressState;
pub use decoded_addr_state::DecodedAddrState;
pub use empty_addr_data::EmptyAddressData;
pub use extended_empty_addr_index::ExtendedEmptyAddressIndex;
pub use funded_addr_data::FundedAddressData;
use funded_addr_data_compact::FundedAddressDataCompact;
pub use funded_addr_index::FundedAddressIndex;
