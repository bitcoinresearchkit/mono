use bitview_traversable::Traversable;

use super::Percentiles;

#[derive(Traversable)]
pub struct Modes<T> {
    /// Bedrock's unweighted mode gives every unspent satoshi equal weight and
    /// calibrates against the share of all-chain supply whose creation price
    /// exceeds spot.
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

impl ModeId {
    pub const ALL: [Self; 3] = [Self::Unweighted, Self::Awake, Self::Mobile];
}

pub const MODE_COUNT: usize = ModeId::ALL.len();

/// Each mode's historical loss-share thresholds, when it has enough history.
pub type Thresholds = Modes<Option<Percentiles<f64>>>;
