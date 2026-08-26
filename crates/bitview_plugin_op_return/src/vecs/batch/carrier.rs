use brk_types::{Bytes, OP_RETURN_KIND_COUNT, OpReturnKind, Sats, VSize};

use crate::{breakdown::BlockMetrics, policy::Policy};

const OLD_STANDARD_MAX_POST_OP_RETURN_BYTES: Bytes = Bytes::new(82);
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
}

impl Carrier {
    pub fn new(vsize: VSize, fees: Sats) -> Self {
        Self {
            vsize,
            fees,
            ..Self::default()
        }
    }

    pub fn add_output(&mut self, kind: OpReturnKind, data_bytes: Bytes) {
        self.kinds |= Self::kind_bit(kind);
        self.output_count += 1;
        self.data_bytes += data_bytes;
        if data_bytes > OLD_STANDARD_MAX_POST_OP_RETURN_BYTES {
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

        self.add_to(total);
        let mut kinds = self.kinds;
        while kinds != 0 {
            let kind_index = kinds.trailing_zeros() as usize;
            self.add_to(&mut by_kind[kind_index]);
            kinds &= kinds - 1;
        }

        if self.oversized_output_count > 0 {
            policy.oversized.output_count += self.oversized_output_count;
            policy.oversized.data_bytes += self.oversized_data_bytes;
            self.add_to(&mut policy.oversized);
        }

        if self.output_count > 1 {
            policy.multiple.output_count += self.output_count;
            policy.multiple.data_bytes += self.data_bytes;
            self.add_to(&mut policy.multiple);
        }

        if self.oversized_output_count > 0 || self.output_count > 1 {
            policy.pre_v30_nonstandard.output_count += self.output_count;
            policy.pre_v30_nonstandard.data_bytes += self.data_bytes;
            self.add_to(&mut policy.pre_v30_nonstandard);
        } else {
            policy.pre_v30_standard.output_count += self.output_count;
            policy.pre_v30_standard.data_bytes += self.data_bytes;
            self.add_to(&mut policy.pre_v30_standard);
        }
    }

    const fn kind_bit(kind: OpReturnKind) -> u32 {
        1_u32 << kind as u8
    }

    fn add_to(self, metrics: &mut BlockMetrics) {
        metrics.tx_count += 1;
        metrics.tx_vsize += self.vsize;
        metrics.fees += self.fees;
    }
}

#[cfg(test)]
mod tests {
    use brk_types::{Bytes, OP_RETURN_KIND_COUNT, OpReturnKind, Sats, VSize};
    use vecdb::ColumnId;

    use super::Carrier;
    use crate::{breakdown::BlockMetrics, policy::Policy};

    #[test]
    fn multiple_kinds_count_one_total_carrier() {
        let mut total = BlockMetrics::default();
        let mut by_kind = [BlockMetrics::default(); OP_RETURN_KIND_COUNT];
        let mut policy = Policy::default();
        let mut carrier = Carrier::new(VSize::new(100), Sats::new(500));
        carrier.add_output(OpReturnKind::Runes, Bytes::new(15));
        carrier.add_output(OpReturnKind::Omni, Bytes::new(15));

        carrier.finalize_into(&mut total, &mut by_kind, &mut policy);

        assert_eq!(total.tx_count, 1);
        assert_eq!(by_kind[OpReturnKind::Runes.index()].tx_count, 1);
        assert_eq!(by_kind[OpReturnKind::Omni.index()].tx_count, 1);
        assert_eq!(total.fees, Sats::new(500));
        assert_eq!(by_kind[OpReturnKind::Runes.index()].fees, Sats::new(500));
        assert_eq!(by_kind[OpReturnKind::Omni.index()].fees, Sats::new(500));
        assert_eq!(policy.multiple.fees, Sats::new(500));
        assert_eq!(policy.pre_v30_nonstandard.fees, Sats::new(500));
        assert_eq!(policy.multiple.output_count, 2);
        assert_eq!(policy.pre_v30_nonstandard.tx_count, 1);
        assert_eq!(policy.oversized.tx_count, 0);
        assert_eq!(policy.pre_v30_standard.tx_count, 0);
    }

    #[test]
    fn oversized_output_marks_pre_v30_nonstandard_once() {
        let mut total = BlockMetrics::default();
        let mut by_kind = [BlockMetrics::default(); OP_RETURN_KIND_COUNT];
        let mut policy = Policy::default();
        let mut carrier = Carrier::new(VSize::new(120), Sats::ZERO);
        carrier.add_output(OpReturnKind::Unknown, Bytes::new(83));

        carrier.finalize_into(&mut total, &mut by_kind, &mut policy);

        assert_eq!(policy.oversized.output_count, 1);
        assert_eq!(policy.oversized.tx_vsize, VSize::new(120));
        assert_eq!(policy.pre_v30_nonstandard.tx_count, 1);
        assert_eq!(policy.multiple.tx_count, 0);
        assert_eq!(policy.pre_v30_standard.tx_count, 0);
    }

    #[test]
    fn standard_output_is_recorded_directly() {
        let mut total = BlockMetrics::default();
        let mut by_kind = [BlockMetrics::default(); OP_RETURN_KIND_COUNT];
        let mut policy = Policy::default();
        let mut carrier = Carrier::new(VSize::new(100), Sats::ZERO);
        carrier.add_output(OpReturnKind::Runes, Bytes::new(15));

        carrier.finalize_into(&mut total, &mut by_kind, &mut policy);

        assert_eq!(policy.pre_v30_standard.output_count, 1);
        assert_eq!(policy.pre_v30_standard.data_bytes, Bytes::new(15));
        assert_eq!(policy.pre_v30_standard.tx_count, 1);
        assert_eq!(policy.pre_v30_standard.tx_vsize, VSize::new(100));
        assert_eq!(policy.pre_v30_nonstandard.tx_count, 0);
    }
}
