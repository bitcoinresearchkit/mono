use std::ops::Range;

use brk_error::{Error, Result};
use brk_types::{FeeRate, Sats, VSize};
use vecdb::{ReadableVec, Ro};

use crate::Vecs;

/// Owned transaction facts, without exposing the plugin's private storage graph. These
/// classifiers retain their documented native semantics; CPFP roles and payout membership
/// are deliberately not included because summary roles use different rules.
#[derive(Clone, Copy)]
pub struct SummaryFacts {
    pub fee: Sats,
    pub vsize: VSize,
    pub rate: FeeRate,
    pub nonstandard: bool,
    pub coinjoin: bool,
    pub consolidation: bool,
}

impl Vecs<Ro> {
    /// Caller retains a pipeline publication pin covering `range` for the entire read.
    /// Read each existing column once over exactly the selected transaction range. No new
    /// persisted data, cache, classification pass or indexing/version change.
    pub fn summary_facts(&self, range: Range<usize>) -> Result<Vec<SummaryFacts>> {
        let len = range.end.checked_sub(range.start)
            .ok_or(Error::Internal("Invalid summary range"))?;
        macro_rules! read {
            ($source:expr) => {{
                let values = $source.collect_range_at(range.start, range.end);
                if values.len() != len {
                    return Err(Error::Internal("Short transaction summary facts read"));
                }
                values
            }};
        }
        let fees = read!(self.fees.fee.tx);
        let vsizes = read!(self.vsize.tx);
        let rates = read!(self.fees.effective_fee_rate.tx);
        let nonstandard = read!(self.nonstandard.flag);
        let coinjoin = read!(self.patterns.coinjoin.flag);
        let consolidation = read!(self.patterns.consolidation.flag);
        (0..len).map(|i| {
            let rate = rates[i];
            if !f64::from(rate).is_finite() {
                return Err(Error::Internal("Undefined transaction summary rate"));
            }
            Ok(SummaryFacts {
                fee: fees[i],
                vsize: vsizes[i],
                rate,
                nonstandard: nonstandard[i].is_true(),
                coinjoin: coinjoin[i].is_true(),
                consolidation: consolidation[i].is_true(),
            })
        }).collect()
    }
}
