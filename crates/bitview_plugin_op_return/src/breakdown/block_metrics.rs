use bitview_primitives::Bytes;
use brk_types::{Sats, VSize};

#[derive(Clone, Copy, Default)]
pub struct BlockMetrics {
    pub output_count: u64,
    pub data_bytes: Bytes,
    pub tx_count: u64,
    pub tx_vsize: VSize,
    pub fees: Sats,
}
