//! CPFP queries shared by live mempool and confirmed transactions.

pub mod confirmed;
pub mod resolved;

pub use resolved::ResolvedCpfp;
