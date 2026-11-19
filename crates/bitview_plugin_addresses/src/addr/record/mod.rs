//! The records the address-state vecs store: each address's four-byte state, and the funded and
//! empty records it points to.

mod addr_state;
mod decoded_addr_state;
mod empty_addr_data;
mod funded_addr_data;
mod funded_addr_data_compact;

pub use addr_state::AddressState;
pub use decoded_addr_state::DecodedAddrState;
pub use empty_addr_data::EmptyAddressData;
pub use funded_addr_data::FundedAddressData;
use funded_addr_data_compact::FundedAddressDataCompact;
