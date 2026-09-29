//! Generic per-address-type supply tracking, shared across predicate-based
//! supply categories (exposed, reused, respent). A "category supply" is the
//! running sum of balances held by addresses currently in the funded subset
//! defined by some predicate.

mod share;
mod state;
mod vecs;

pub use share::AddrSupplyShareVecs;
pub use state::AddrTypeToSupply;
pub use vecs::AddrSupplyVecs;
