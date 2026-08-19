use brk_types::{OutputType, Sats};

use crate::{HistogramRaw, sats_to_bin};

/// Dust floor: outputs below this many sats are too small to be payments.
const MIN_SATS: u64 = 1000;

/// Output types skipped entirely (protocol-dominated, too noisy to carry the
/// round-dollar signal).
const EXCLUDED_OUTPUT_TYPES: &[OutputType] = &[OutputType::P2TR];

/// Bitmask form of [`EXCLUDED_OUTPUT_TYPES`], folded at compile time so
/// [`PaymentFilter::eligible_bin`] checks membership with a single AND.
const EXCLUDED_MASK: u16 = {
    let mut mask = 0u16;
    let mut i = 0;
    while i < EXCLUDED_OUTPUT_TYPES.len() {
        mask |= 1u16 << EXCLUDED_OUTPUT_TYPES[i] as u8;
        i += 1;
    }
    mask
};

/// Round-dollar payment filter.
///
/// Input: transaction outputs. Output: eligible log-scale bins or a fresh block
/// histogram. The only state is the transaction-output fan-out cap selected by
/// block height, or [`MODERN`](Self::MODERN) for live modern transaction streams.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PaymentFilter {
    tx_output_fanout_cap: usize,
}

impl PaymentFilter {
    /// Pre-modern transaction-output fan-out cap. Above this, the transaction is
    /// a batch payout (exchange sweep, mixer fan-out), not a round-dollar
    /// payment.
    pub const PRE_MODERN_TX_OUTPUT_FANOUT_CAP: usize = 100;

    /// Modern-chain transaction-output fan-out cap. Dense post-630k blocks can
    /// carry more genuine payment outputs, but very large fan-outs can still
    /// dominate one EMA slot and create a false round-dollar ladder.
    pub const MODERN_TX_OUTPUT_FANOUT_CAP: usize = 250;

    /// Height where [`Self::PRE_MODERN_TX_OUTPUT_FANOUT_CAP`] relaxes to
    /// [`Self::MODERN_TX_OUTPUT_FANOUT_CAP`].
    pub const MODERN_TX_OUTPUT_FANOUT_CAP_START_HEIGHT: usize = 630_000;

    /// Filter for live or otherwise guaranteed-modern transaction streams.
    pub const MODERN: Self = Self::with_fanout_cap(Self::MODERN_TX_OUTPUT_FANOUT_CAP);

    const fn with_fanout_cap(tx_output_fanout_cap: usize) -> Self {
        Self {
            tx_output_fanout_cap,
        }
    }

    /// Filter for transactions in `height`.
    pub const fn for_height(height: usize) -> Self {
        if height < Self::MODERN_TX_OUTPUT_FANOUT_CAP_START_HEIGHT {
            Self::with_fanout_cap(Self::PRE_MODERN_TX_OUTPUT_FANOUT_CAP)
        } else {
            Self::MODERN
        }
    }

    /// Bin index for `(sats, output_type)`, or `None` for an excluded type
    /// (P2TR), dust, a round-BTC value, or an out-of-range bin. The per-output
    /// half of the round-dollar payment filter.
    #[inline(always)]
    pub fn eligible_bin(sats: Sats, output_type: OutputType) -> Option<u16> {
        if EXCLUDED_MASK & (1u16 << output_type as u8) != 0 {
            return None;
        }
        if *sats < MIN_SATS || sats.is_common_round_value() {
            return None;
        }
        sats_to_bin(sats).map(|b| b as u16)
    }

    /// Apply the transaction-level payment filter and call `emit(bin)` for each
    /// eligible output, in order.
    ///
    /// A whole transaction is dropped when it carries any OP_RETURN output (data
    /// carriers, not payments) or when it has more outputs than this filter's
    /// fan-out cap.
    #[inline]
    pub fn for_each_bin(
        self,
        outputs: impl ExactSizeIterator<Item = (Sats, OutputType)> + Clone,
        mut emit: impl FnMut(u16),
    ) {
        if outputs.len() > self.tx_output_fanout_cap {
            return;
        }
        if outputs.clone().any(|(_, ty)| ty == OutputType::OpReturn) {
            return;
        }
        for (sats, ty) in outputs {
            if let Some(bin) = Self::eligible_bin(sats, ty) {
                emit(bin);
            }
        }
    }

    /// Build a fresh eligible round-dollar payment histogram for one block's
    /// non-coinbase transaction outputs.
    #[inline]
    pub fn histogram<Outputs>(self, txs: impl IntoIterator<Item = Outputs>) -> HistogramRaw
    where
        Outputs: ExactSizeIterator<Item = (Sats, OutputType)> + Clone,
    {
        let mut hist = HistogramRaw::zeros();
        for outputs in txs {
            self.for_each_bin(outputs, |bin| hist.increment(bin as usize));
        }
        hist
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn payment_outputs(len: usize) -> impl ExactSizeIterator<Item = (Sats, OutputType)> + Clone {
        std::iter::repeat_n((Sats::new(12_345), OutputType::P2WPKH), len)
    }

    fn emitted_count(height: usize, len: usize) -> usize {
        let mut count = 0;
        PaymentFilter::for_height(height).for_each_bin(payment_outputs(len), |_| count += 1);
        count
    }

    #[test]
    fn early_fanout_cap_is_strict() {
        assert_eq!(
            emitted_count(
                PaymentFilter::MODERN_TX_OUTPUT_FANOUT_CAP_START_HEIGHT - 1,
                PaymentFilter::PRE_MODERN_TX_OUTPUT_FANOUT_CAP,
            ),
            PaymentFilter::PRE_MODERN_TX_OUTPUT_FANOUT_CAP
        );
        assert_eq!(
            emitted_count(
                PaymentFilter::MODERN_TX_OUTPUT_FANOUT_CAP_START_HEIGHT - 1,
                PaymentFilter::PRE_MODERN_TX_OUTPUT_FANOUT_CAP + 1,
            ),
            0
        );
    }

    #[test]
    fn modern_fanout_cap_is_relaxed_but_not_lifted() {
        assert_eq!(
            emitted_count(
                PaymentFilter::MODERN_TX_OUTPUT_FANOUT_CAP_START_HEIGHT,
                PaymentFilter::MODERN_TX_OUTPUT_FANOUT_CAP,
            ),
            PaymentFilter::MODERN_TX_OUTPUT_FANOUT_CAP
        );
        assert_eq!(
            emitted_count(
                PaymentFilter::MODERN_TX_OUTPUT_FANOUT_CAP_START_HEIGHT,
                PaymentFilter::MODERN_TX_OUTPUT_FANOUT_CAP + 1,
            ),
            0
        );
    }

    fn emitted_count_modern(len: usize) -> usize {
        let mut count = 0;
        PaymentFilter::MODERN.for_each_bin(payment_outputs(len), |_| count += 1);
        count
    }

    #[test]
    fn modern_helper_uses_modern_fanout_cap() {
        assert_eq!(
            emitted_count_modern(PaymentFilter::MODERN_TX_OUTPUT_FANOUT_CAP),
            PaymentFilter::MODERN_TX_OUTPUT_FANOUT_CAP
        );
        assert_eq!(
            emitted_count_modern(PaymentFilter::MODERN_TX_OUTPUT_FANOUT_CAP + 1),
            0
        );
    }

    #[test]
    fn payment_histogram_drops_op_return_transaction() {
        let sats = Sats::new(12_345);
        let txs = vec![
            vec![(sats, OutputType::P2WPKH), (sats, OutputType::P2PKH)],
            vec![
                (Sats::new(54_321), OutputType::OpReturn),
                (sats, OutputType::P2WPKH),
            ],
        ];
        let hist = PaymentFilter::MODERN.histogram(txs.into_iter().map(|tx| tx.into_iter()));

        let bin = PaymentFilter::eligible_bin(sats, OutputType::P2WPKH).unwrap() as usize;
        assert_eq!(hist[bin], 2);
    }

    #[test]
    fn builds_fresh_payment_histogram() {
        let sats = Sats::new(12_345);
        let txs = vec![vec![
            (sats, OutputType::P2WPKH),
            (Sats::new(100_000_000), OutputType::P2WPKH),
        ]];

        let hist = PaymentFilter::MODERN.histogram(txs.into_iter().map(|tx| tx.into_iter()));

        let bin = PaymentFilter::eligible_bin(sats, OutputType::P2WPKH).unwrap() as usize;
        assert_eq!(hist[bin], 1);
    }
}
