mod cluster;
mod cluster_chunk;
mod cluster_tx;
mod cluster_tx_index;
mod entry;
mod info;

pub use cluster::CpfpCluster;
pub use cluster_chunk::CpfpClusterChunk;
pub use cluster_tx::CpfpClusterTx;
pub use cluster_tx_index::CpfpClusterTxIndex;
pub use entry::CpfpEntry;
pub use info::CpfpInfo;

/// Bitcoin Core's default mempool ancestor/descendant chain cap, also
/// used by mempool.space-style truncation in CPFP walks.
pub const CPFP_CHAIN_LIMIT: usize = 25;
