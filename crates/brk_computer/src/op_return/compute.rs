use brk_error::Result;
use brk_indexer::Indexer;
use brk_types::{Bytes, OP_RETURN_KIND_COUNT, OpReturnKind, OpReturnPolicyId, Sats, VSize};
use vecdb::{AnyVec, ColumnId, Exit, ReadableVec, VecIndex};

use super::{Vecs, breakdown::BlockMetrics, policy::Policy};
use crate::transactions;

const OLD_STANDARD_MAX_POST_OP_RETURN_BYTES: Bytes = Bytes::new(82);
const WRITE_INTERVAL: usize = 10_000;
const _: () = assert!(OP_RETURN_KIND_COUNT <= u32::BITS as usize);

#[derive(Clone, Copy, Default)]
struct Carrier {
    kinds: u32,
    output_count: u64,
    data_bytes: Bytes,
    oversized_output_count: u64,
    oversized_data_bytes: Bytes,
    vsize: VSize,
    fees: Sats,
}

impl Carrier {
    fn add_output(&mut self, kind: OpReturnKind, data_bytes: Bytes) {
        self.kinds |= Self::kind_bit(kind);
        self.output_count += 1;
        self.data_bytes += data_bytes;
        if data_bytes > OLD_STANDARD_MAX_POST_OP_RETURN_BYTES {
            self.oversized_output_count += 1;
            self.oversized_data_bytes += data_bytes;
        }
    }

    fn finalize_into(
        self,
        total: &mut BlockMetrics,
        by_kind: &mut [BlockMetrics; OP_RETURN_KIND_COUNT],
        policy: &mut Policy<BlockMetrics>,
    ) {
        if self.output_count == 0 {
            return;
        }

        total.add_carrier(self);
        let mut kinds = self.kinds;
        while kinds != 0 {
            let kind_index = kinds.trailing_zeros() as usize;
            by_kind[kind_index].add_carrier(self);
            kinds &= kinds - 1;
        }

        if self.oversized_output_count > 0 {
            policy.oversized.output_count += self.oversized_output_count;
            policy.oversized.data_bytes += self.oversized_data_bytes;
            policy.oversized.add_carrier(self);
        }

        if self.output_count > 1 {
            policy.multiple.output_count += self.output_count;
            policy.multiple.data_bytes += self.data_bytes;
            policy.multiple.add_carrier(self);
        }

        if self.oversized_output_count > 0 || self.output_count > 1 {
            policy.pre_v30_nonstandard.output_count += self.output_count;
            policy.pre_v30_nonstandard.data_bytes += self.data_bytes;
            policy.pre_v30_nonstandard.add_carrier(self);
        } else {
            policy.pre_v30_standard.output_count += self.output_count;
            policy.pre_v30_standard.data_bytes += self.data_bytes;
            policy.pre_v30_standard.add_carrier(self);
        }
    }

    const fn kind_bit(kind: OpReturnKind) -> u32 {
        1_u32 << kind as u8
    }
}

impl BlockMetrics {
    fn add_carrier(&mut self, carrier: Carrier) {
        self.tx_count += 1;
        self.tx_vsize += carrier.vsize;
        self.fees += carrier.fees;
    }
}

impl Vecs {
    pub(crate) fn compute(
        &mut self,
        indexer: &Indexer,
        fees: &transactions::FeesVecs,
        exit: &Exit,
    ) -> Result<()> {
        self.db.sync_bg_tasks()?;

        let starting_lengths = indexer.safe_lengths();
        let raw = &indexer.vecs().op_return;
        let txs = &indexer.vecs().transactions;
        let version = raw.first_index.version()
            + raw.to_tx_index.version()
            + raw.kind.version()
            + raw.post_op_return_bytes.version()
            + txs.weight.version()
            + fees.fee.tx_index.version();

        self.validate_and_truncate(version, starting_lengths.height)?;

        let skip = self.min_len();
        let end = raw.first_index.len();
        if skip < end {
            self.truncate_if_needed_at(skip)?;

            let op_return_len = raw.to_tx_index.len();
            let mut tx_cursor = raw.to_tx_index.cursor();
            let mut kind_cursor = raw.kind.cursor();
            let mut post_op_return_bytes = raw.post_op_return_bytes.cursor();
            let mut first_index_cursor = raw.first_index.cursor();
            let mut weight_cursor = txs.weight.cursor();
            let mut fee_cursor = fees.fee.tx_index.cursor();
            first_index_cursor.advance(skip);
            let mut start = first_index_cursor.next().unwrap().to_usize();

            for height in skip..end {
                let block_end = if height + 1 < end {
                    first_index_cursor.next().unwrap().to_usize()
                } else {
                    op_return_len
                };

                tx_cursor.advance(start - tx_cursor.position());
                kind_cursor.advance(start - kind_cursor.position());
                post_op_return_bytes.advance(start - post_op_return_bytes.position());

                let mut total = BlockMetrics::default();
                let mut by_kind = [BlockMetrics::default(); OP_RETURN_KIND_COUNT];
                let mut policy = Policy::default();
                let mut current_tx = None;
                let mut carrier = Carrier::default();

                for _ in start..block_end {
                    let tx_index = tx_cursor.next().unwrap();
                    let kind = kind_cursor.next().unwrap();
                    let bytes = Bytes::from(u32::from(post_op_return_bytes.next().unwrap()));
                    let kind_index = kind.index();

                    if current_tx != Some(tx_index) {
                        carrier.finalize_into(&mut total, &mut by_kind, &mut policy);
                        current_tx = Some(tx_index);
                        carrier = Carrier::default();

                        let tx_position = tx_index.to_usize();
                        weight_cursor.advance(tx_position - weight_cursor.position());
                        carrier.vsize = VSize::from(weight_cursor.next().unwrap());
                        fee_cursor.advance(tx_position - fee_cursor.position());
                        carrier.fees = fee_cursor.next().unwrap();
                    }

                    total.data_bytes += bytes;
                    by_kind[kind_index].output_count += 1;
                    by_kind[kind_index].data_bytes += bytes;
                    carrier.add_output(kind, bytes);
                }

                carrier.finalize_into(&mut total, &mut by_kind, &mut policy);

                self.total.push(total);
                self.by_kind.push(by_kind);
                self.policy
                    .push(OpReturnPolicyId::from_fn(|id| *policy.get(id)));

                if (height + 1).is_multiple_of(WRITE_INTERVAL) {
                    let _lock = exit.lock();
                    self.write()?;
                }
                start = block_end;
            }

            let _lock = exit.lock();
            self.write()?;
        }

        let exit = exit.clone();
        self.db.run_bg(move |db| {
            let _lock = exit.lock();
            db.compact_deferred_default()
        });
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn multiple_kinds_count_one_total_carrier() {
        let mut total = BlockMetrics::default();
        let mut by_kind = [BlockMetrics::default(); OP_RETURN_KIND_COUNT];
        let mut policy = Policy::default();
        let mut carrier = Carrier {
            vsize: VSize::new(100),
            fees: Sats::new(500),
            ..Carrier::default()
        };
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
        let mut carrier = Carrier {
            vsize: VSize::new(120),
            ..Carrier::default()
        };
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
        let mut carrier = Carrier {
            vsize: VSize::new(100),
            ..Carrier::default()
        };
        carrier.add_output(OpReturnKind::Runes, Bytes::new(15));

        carrier.finalize_into(&mut total, &mut by_kind, &mut policy);

        assert_eq!(policy.pre_v30_standard.output_count, 1);
        assert_eq!(policy.pre_v30_standard.data_bytes, Bytes::new(15));
        assert_eq!(policy.pre_v30_standard.tx_count, 1);
        assert_eq!(policy.pre_v30_standard.tx_vsize, VSize::new(100));
        assert_eq!(policy.pre_v30_nonstandard.tx_count, 0);
    }
}
