use bitview_traversable::Traversable;

use super::Percentiles;

#[derive(Clone, Copy, Default, Traversable)]
pub struct Modes<T> {
    /// Bedrock's awake mode weights each UTXO age range by wakefulness—the
    /// share of its accumulated coin days that has been consumed—and
    /// calibrates against the resulting weighted share of supply in loss.
    pub awake: T,
    /// Bedrock's mobile mode weights each UTXO age range by mobility—the
    /// estimated probability that UTXOs of that age will ever be spent—and
    /// calibrates against the resulting weighted share of supply in loss.
    pub mobile: T,
    /// Bedrock's unweighted mode gives every unspent satoshi equal weight and
    /// calibrates against the share of all-chain supply whose creation price
    /// exceeds spot.
    pub unweighted: T,
}

collection!(
    /// A Bedrock mode: the distribution weighted by wakefulness (Cointime) or mobility
    /// (Coinflow), or, last, the unweighted one they improve on.
    ModeId for Modes {
        Awake => awake,
        Mobile => mobile,
        Unweighted => unweighted,
    }
);

pub const MODE_COUNT: usize = ModeId::ALL.len();

/// Each mode's historical loss-share thresholds, when it has enough history.
pub type Thresholds = Modes<Option<Percentiles<f64>>>;
