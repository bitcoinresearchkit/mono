mod block_loop;
mod context;
mod price_range_max;
mod readers;
mod recover;
mod write;

pub use block_loop::process_blocks;
pub use context::ComputeContext;
pub use price_range_max::PriceRangeMax;
pub use readers::{AddrReaders, IndexToTxIndexBuf, TxInReaders, TxOutData, TxOutReaders};
pub use recover::{StartMode, determine_start_mode, reset_state};

/// Flush checkpoint interval (every N blocks).
pub const FLUSH_INTERVAL: usize = 10_000;

// BIP30 duplicate coinbase heights (special case handling)
pub const BIP30_DUPLICATE_HEIGHT_1: u32 = 91_842;
pub const BIP30_DUPLICATE_HEIGHT_2: u32 = 91_880;
pub const BIP30_ORIGINAL_HEIGHT_1: u32 = 91_812;
pub const BIP30_ORIGINAL_HEIGHT_2: u32 = 91_722;
