use brk_types::{Bytes, Sats, VSize};

#[derive(Clone, Copy, Debug, Default)]
pub struct BlockMetrics {
    pub output_count: u64,
    pub data_bytes: Bytes,
    pub tx_count: u64,
    pub tx_vsize: VSize,
    pub fees: Sats,
}
