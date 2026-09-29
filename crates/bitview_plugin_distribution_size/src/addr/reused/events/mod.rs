//! Per-block address-reuse event tracking. Holds both the output-side
//! ("an output landed on a previously-used address") and input-side
//! ("an input spent from an address in the reused set") event counters.
//! Shared between reused (receive-based) and respent (spend-based) flavors.
//! See [`vecs::AddrEventsVecs`] for the full description of each metric.

mod state;
mod vecs;

pub use state::AddrTypeToAddrEventCount;
pub use vecs::AddrEventsVecs;
