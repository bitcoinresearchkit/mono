use bitview_cohort::{AgeAggregateId, AgeRange};
use bitview_primitives::CentsCompact;
use bitview_traversable::Traversable;
use bitview_urpd::OriginUrpd;

use super::Percentiles;

#[derive(Clone, Copy, Default, Traversable)]
pub struct Modes<T> {
    /// Bedrock's awake mode weights each UTXO age range by wakefulness—the
    /// share of its accumulated coin days that has been consumed—and
    /// calibrates against the resulting weighted share of supply in loss.
    #[traversable(lead = "Awake mode: coins weighted by how much of their holding time has been spent.")]
    pub awake: T,
    /// Bedrock's mobile mode weights each UTXO age range by mobility—the
    /// estimated probability that UTXOs of that age will ever be spent—and
    /// calibrates against the resulting weighted share of supply in loss.
    #[traversable(lead = "Mobile mode: coins weighted by how likely they are to be spent.")]
    pub mobile: T,
    /// Bedrock's unweighted mode gives every unspent satoshi equal weight and
    /// calibrates against the share of all-chain supply whose creation price
    /// exceeds spot.
    #[traversable(lead = "Unweighted mode: every coin counts the same.")]
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

/// The two weighted modes' age-range weights for one block; the unweighted mode needs none.
pub(crate) struct ModeWeights<'w> {
    pub awake: Option<&'w AgeRange<f64>>,
    pub mobile: Option<&'w AgeRange<f64>>,
}

impl ModeWeights<'_> {
    /// Keep URPD's compact weighted slots at this boundary: awake first, mobile second,
    /// and unweighted supply from the raw all-age projection.
    pub(crate) fn project<'a>(
        &self,
        source: &'a OriginUrpd,
    ) -> impl Iterator<Item = (CentsCompact, Modes<u64>)> + 'a + use<'a> {
        source
            .project(&[self.awake, self.mobile], [AgeAggregateId::All.age_range_ids()])
            .map(|bucket| {
                let [awake, mobile] = bucket.weighted[0];
                (
                    bucket.price,
                    Modes {
                        awake: u64::from(awake),
                        mobile: u64::from(mobile),
                        unweighted: u64::from(bucket.raw[0]),
                    },
                )
            })
    }
}

/// Each mode's historical loss-share thresholds, when it has enough history.
pub type Thresholds = Modes<Option<Percentiles<f64>>>;
