use bitview_traversable::Traversable;

use super::Percentiles;

#[derive(Traversable)]
pub struct Modes<T> {
    /// Bedrock's unweighted mode gives every unspent satoshi equal weight. It uses the
    /// all-chain distribution of UTXO creation prices and the unweighted share
    /// of supply whose creation price exceeds spot. A UTXO's creation price is
    /// Bitcoin's spot price when that output was created.
    pub unweighted: T,
    /// Bedrock's awake mode weights each UTXO age range by wakefulness—the
    /// share of its accumulated coin days that has been consumed—and
    /// calibrates against the resulting weighted share of supply in loss.
    pub awake: T,
    /// Bedrock's mobile mode weights each UTXO age range by mobility—the
    /// estimated probability that UTXOs of that age will ever be spent—and
    /// calibrates against the resulting weighted share of supply in loss.
    pub mobile: T,
}

collection!(
    /// A Bedrock mode: the unweighted distribution, or one weighted by wakefulness (Cointime) or
    /// mobility (Coinflow).
    ModeId for Modes {
        Unweighted => unweighted,
        Awake => awake,
        Mobile => mobile,
    }
);

pub const MODE_COUNT: usize = ModeId::ALL.len();

/// Each mode's historical loss-share thresholds, when it has enough history.
pub type Thresholds = Modes<Option<Percentiles<f64>>>;
