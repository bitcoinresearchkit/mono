use bitview_primitives::{Bytes, OP_RETURN_KIND_COUNT, OpReturnKind};
use brk_types::{Sats, VSize};

use crate::{breakdown::BlockMetrics, policy::Policy};

const PRE_V30_MAX_POST_OP_RETURN_BYTES: Bytes = Bytes::new(82);
const _: () = assert!(OP_RETURN_KIND_COUNT <= u32::BITS as usize);

#[derive(Clone, Copy, Default)]
pub struct Carrier {
    kinds: u32,
    output_count: u64,
    data_bytes: Bytes,
    oversized_output_count: u64,
    oversized_data_bytes: Bytes,
    vsize: VSize,
    fees: Sats,
    /// Coinbase transactions are never relayed: the relay-policy shapes skip them.
    coinbase: bool,
}

impl Carrier {
    pub fn new(vsize: VSize, fees: Sats, coinbase: bool) -> Self {
        Self {
            vsize,
            fees,
            coinbase,
            ..Self::default()
        }
    }

    pub fn add_output(&mut self, kind: OpReturnKind, data_bytes: Bytes) {
        self.kinds |= 1 << kind as u8;
        self.output_count += 1;
        self.data_bytes += data_bytes;
        if data_bytes > PRE_V30_MAX_POST_OP_RETURN_BYTES {
            self.oversized_output_count += 1;
            self.oversized_data_bytes += data_bytes;
        }
    }

    pub fn finalize_into(
        self,
        total: &mut BlockMetrics,
        by_kind: &mut [BlockMetrics; OP_RETURN_KIND_COUNT],
        policy: &mut Policy<BlockMetrics>,
    ) {
        if self.output_count == 0 {
            return;
        }

        self.add_tx_to(total);
        let mut kinds = self.kinds;
        while kinds != 0 {
            let kind_index = kinds.trailing_zeros() as usize;
            self.add_tx_to(&mut by_kind[kind_index]);
            kinds &= kinds - 1;
        }

        if self.coinbase {
            return;
        }

        if self.oversized_output_count > 0 {
            policy.oversized.output_count += self.oversized_output_count;
            policy.oversized.data_bytes += self.oversized_data_bytes;
            self.add_tx_to(&mut policy.oversized);
        }

        if self.output_count > 1 {
            policy.multiple.output_count += self.output_count;
            policy.multiple.data_bytes += self.data_bytes;
            self.add_tx_to(&mut policy.multiple);
        }

        if self.oversized_output_count > 0 || self.output_count > 1 {
            policy.pre_v30_nonstandard.output_count += self.output_count;
            policy.pre_v30_nonstandard.data_bytes += self.data_bytes;
            self.add_tx_to(&mut policy.pre_v30_nonstandard);
        } else {
            policy.pre_v30_standard.output_count += self.output_count;
            policy.pre_v30_standard.data_bytes += self.data_bytes;
            self.add_tx_to(&mut policy.pre_v30_standard);
        }
    }

    fn add_tx_to(self, metrics: &mut BlockMetrics) {
        metrics.tx_count += 1;
        metrics.tx_vsize += self.vsize;
        metrics.fees += self.fees;
    }
}
