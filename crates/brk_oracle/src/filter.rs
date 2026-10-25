use brk_types::{OutputType, Sats};

use crate::{HistogramRaw, sats_to_bin};

/// Dust floor: outputs below this many sats are too small to be payments.
const MIN_SATS: u64 = 1000;

/// Why an output isn't a payment: the first rule that skips it, in the order the filter asks.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Skip {
    /// The block's coinbase: new coins, not a payment (dropped before the filter).
    Coinbase,
    /// Its transaction has more outputs than the fan-out cap: a batch payout.
    FanOut,
    /// Its transaction carries an OP_RETURN output: data, not a payment.
    OpReturn,
    /// A taproot output: protocol traffic drowns the signal.
    Taproot,
    /// Under the dust floor.
    Dust,
    /// A round amount in bitcoin, so not in dollars.
    RoundBtc,
}

/// Outputs by what the filter made of them: each skipped one under the first rule that skips it,
/// the rest payments.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Tally {
    pub coinbase: u32,
    pub fan_out: u32,
    pub op_return: u32,
    pub taproot: u32,
    pub dust: u32,
    pub round_btc: u32,
    pub payments: u32,
}

impl Tally {
    fn slot(&mut self, skip: Option<Skip>) -> &mut u32 {
        match skip {
            None => &mut self.payments,
            Some(Skip::Coinbase) => &mut self.coinbase,
            Some(Skip::FanOut) => &mut self.fan_out,
            Some(Skip::OpReturn) => &mut self.op_return,
            Some(Skip::Taproot) => &mut self.taproot,
            Some(Skip::Dust) => &mut self.dust,
            Some(Skip::RoundBtc) => &mut self.round_btc,
        }
    }

    pub fn add(&mut self, skip: Option<Skip>, outputs: u32) {
        *self.slot(skip) += outputs;
    }

    /// Reverse a previous `add`.
    pub fn remove(&mut self, skip: Option<Skip>, outputs: u32) {
        let slot = self.slot(skip);
        debug_assert!(*slot >= outputs, "removing outputs never added");
        *slot -= outputs;
    }
}

/// Round-dollar payment filter. Its only state is the transaction-output fan-out cap, selected
/// by block height ([`for_height`](Self::for_height)) or [`MODERN`](Self::MODERN) for live
/// modern transaction streams.
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
    const PRE_MODERN_TX_OUTPUT_FANOUT_CAP: usize = 100;

    /// Modern-chain transaction-output fan-out cap. Dense post-630k blocks can
    /// carry more genuine payment outputs, but very large fan-outs can still
    /// dominate one EMA slot and create a false round-dollar ladder.
    const MODERN_TX_OUTPUT_FANOUT_CAP: usize = 250;

    /// Height where [`Self::PRE_MODERN_TX_OUTPUT_FANOUT_CAP`] relaxes to
    /// [`Self::MODERN_TX_OUTPUT_FANOUT_CAP`].
    const MODERN_TX_OUTPUT_FANOUT_CAP_START_HEIGHT: usize = 630_000;

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

    /// The transaction-level half of the round-dollar payment filter: a whole transaction is
    /// skipped when it has more outputs than this filter's fan-out cap, or when it carries any
    /// OP_RETURN output (data carriers, not payments).
    #[inline(always)]
    fn skips_tx(
        self,
        outputs: impl ExactSizeIterator<Item = (Sats, OutputType)> + Clone,
    ) -> Option<Skip> {
        if outputs.len() > self.tx_output_fanout_cap {
            Some(Skip::FanOut)
        } else if outputs.clone().any(|(_, ty)| ty == OutputType::OpReturn) {
            Some(Skip::OpReturn)
        } else {
            None
        }
    }

    /// The per-output half: an excluded type (P2TR), dust or a round-BTC value.
    #[inline(always)]
    fn skips_output(sats: Sats, output_type: OutputType) -> Option<Skip> {
        // P2TR is protocol-dominated, too noisy to carry the round-dollar signal.
        if output_type == OutputType::P2TR {
            Some(Skip::Taproot)
        } else if *sats < MIN_SATS {
            Some(Skip::Dust)
        } else if Self::is_common_round_value(sats) {
            Some(Skip::RoundBtc)
        } else {
            None
        }
    }

    /// A transaction's outputs as the filter judges them: `each(skip, outputs)` once for a whole
    /// transaction skipped, else once per output (`None`: a payment).
    #[inline]
    pub fn classify(
        self,
        outputs: impl ExactSizeIterator<Item = (Sats, OutputType)> + Clone,
        mut each: impl FnMut(Option<Skip>, u32),
    ) {
        if let Some(skip) = self.skips_tx(outputs.clone()) {
            each(Some(skip), outputs.len() as u32);
            return;
        }
        for (sats, ty) in outputs {
            each(Self::skips_output(sats, ty), 1);
        }
    }

    /// A block's outputs tallied: its coinbase's, then each other transaction's as
    /// [`Self::classify`] judges them.
    pub fn tally<Outputs>(self, coinbase: usize, txs: impl IntoIterator<Item = Outputs>) -> Tally
    where
        Outputs: ExactSizeIterator<Item = (Sats, OutputType)> + Clone,
    {
        let mut tally = Tally::default();
        tally.add(Some(Skip::Coinbase), coinbase as u32);
        for outputs in txs {
            self.classify(outputs, |skip, n| tally.add(skip, n));
        }
        tally
    }

    /// Apply the transaction-level payment filter and call `emit(bin)` for each
    /// eligible output inside the histogram, in order.
    #[inline]
    pub fn for_each_bin(
        self,
        outputs: impl ExactSizeIterator<Item = (Sats, OutputType)> + Clone,
        mut emit: impl FnMut(u16),
    ) {
        self.for_each_payment(outputs, |sats| {
            if let Some(bin) = sats_to_bin(sats) {
                emit(bin as u16);
            }
        });
    }

    /// Apply the payment filter and call `emit(sats)` for each eligible output, its exact amount,
    /// in order.
    #[inline]
    pub fn for_each_payment(
        self,
        outputs: impl ExactSizeIterator<Item = (Sats, OutputType)> + Clone,
        mut emit: impl FnMut(Sats),
    ) {
        if self.skips_tx(outputs.clone()).is_some() {
            return;
        }
        for (sats, ty) in outputs {
            if Self::skips_output(sats, ty).is_none() {
                emit(sats);
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
        let mut hist = HistogramRaw::default();
        for outputs in txs {
            self.for_each_bin(outputs, |bin| hist.increment(bin as usize));
        }
        hist
    }
}
