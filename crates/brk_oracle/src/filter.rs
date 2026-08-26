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
    // Five leading digits across 10^0..10^18, plus the only 10^19 target that
    // fits in u64.
    const COMMON_ROUND_RANGES: [(u64, u64); 96] = Self::common_round_ranges();

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

    const fn common_round_ranges() -> [(u64, u64); 96] {
        const LEADING_DIGITS: [u64; 5] = [1, 2, 3, 5, 6];

        let mut ranges = [(0, 0); 96];
        let mut index = 0;
        let mut magnitude = 1;
        while magnitude <= 1_000_000_000_000_000_000 {
            let mut digit = 0;
            while digit < LEADING_DIGITS.len() {
                let round = LEADING_DIGITS[digit] * magnitude;
                let tolerance = round / 1000;
                ranges[index] = (round - tolerance, round + tolerance);
                index += 1;
                digit += 1;
            }
            magnitude *= 10;
        }

        let round = 10_000_000_000_000_000_000;
        let tolerance = round / 1000;
        ranges[index] = (round - tolerance, round + tolerance);
        ranges
    }

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

    #[inline(always)]
    fn is_common_round_value(sats: Sats) -> bool {
        let value = *sats;
        let index = Self::COMMON_ROUND_RANGES.partition_point(|range| range.1 < value);
        index < Self::COMMON_ROUND_RANGES.len() && Self::COMMON_ROUND_RANGES[index].0 <= value
    }

    /// Bin index for `(sats, output_type)`, or `None` for an excluded type
    /// (P2TR), dust, a round-BTC value, or an out-of-range bin. The per-output
    /// half of the round-dollar payment filter.
    #[inline(always)]
    pub fn eligible_bin(sats: Sats, output_type: OutputType) -> Option<u16> {
        if EXCLUDED_MASK & (1u16 << output_type as u8) != 0 {
            return None;
        }
        if *sats < MIN_SATS || Self::is_common_round_value(sats) {
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

    fn reference_is_common_round_value(value: u64) -> bool {
        if value == 0 {
            return false;
        }
        let value = u128::from(value);
        let magnitude = 10u128.pow(value.ilog10());
        let leading = (value + magnitude / 2) / magnitude;
        if !matches!(leading, 1 | 2 | 3 | 5 | 6 | 10) {
            return false;
        }
        let round = leading * magnitude;
        value.abs_diff(round) * 1000 <= round
    }

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
    fn common_round_ranges_match_reference() {
        for &(start, end) in &PaymentFilter::COMMON_ROUND_RANGES {
            for value in [
                start.saturating_sub(2),
                start.saturating_sub(1),
                start,
                start.saturating_add(1),
                end.saturating_sub(1),
                end,
                end.saturating_add(1),
                end.saturating_add(2),
            ] {
                assert_eq!(
                    PaymentFilter::is_common_round_value(Sats::new(value)),
                    reference_is_common_round_value(value),
                    "value {value}"
                );
            }
        }

        let mut value = 0x9e37_79b9_7f4a_7c15_u64;
        for _ in 0..1_000_000 {
            value = value
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1);
            assert_eq!(
                PaymentFilter::is_common_round_value(Sats::new(value)),
                reference_is_common_round_value(value),
                "value {value}"
            );
        }
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
