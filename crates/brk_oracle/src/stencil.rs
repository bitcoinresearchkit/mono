use std::ops::Range;

use derive_more::{Deref, DerefMut};

use crate::{HistogramEma, NUM_BINS};

/// Bin offsets for 19 round-USD amounts relative to the $100 reference (offset 0).
/// Each offset = log10(amount / 100) * BINS_PER_DECADE.
const STENCIL_OFFSETS: [i32; 19] = [
    -400, // $1
    -340, // $2
    -305, // $3
    -260, // $5
    -200, // $10
    -165, // $15
    -140, // $20
    -120, // $25
    -105, // $30
    -60,  // $50
    0,    // $100
    35,   // $150
    60,   // $200
    95,   // $300
    140,  // $500
    200,  // $1000
    260,  // $2000
    340,  // $5000
    400,  // $10000
];

/// Number of round-USD stencil arms.
const N_ARMS: usize = STENCIL_OFFSETS.len();

#[derive(Clone, Deref, DerefMut)]
struct Arms([f64; N_ARMS]);

/// EMA rate for the adaptive shape template (~250-block time constant), slow
/// enough that a transient octave slide can't corrupt the profile before the
/// pick recovers.
const SHAPE_BETA: f64 = 0.004;

/// EMA mass at `idx`, or 0.0 when the index falls outside the histogram.
#[inline(always)]
fn bin_value(ema: &HistogramEma, idx: i64) -> f64 {
    if idx >= 0 && (idx as usize) < NUM_BINS {
        ema[idx as usize]
    } else {
        0.0
    }
}

/// Raw EMA mass on each of the 19 stencil arms at `center`.
fn arms_at(ema: &HistogramEma, center: i64) -> Arms {
    Arms(STENCIL_OFFSETS.map(|offset| bin_value(ema, center + offset as i64)))
}

/// [`arms_at`] L1-normalized to sum 1, or `None` when the center carries no mass.
fn normalized_arms_at(ema: &HistogramEma, center: i64) -> Option<Arms> {
    let mut arms = arms_at(ema, center);
    let sum: f64 = arms.iter().sum();
    if sum <= 0.0 {
        return None;
    }
    for arm in arms.iter_mut() {
        *arm /= sum;
    }
    Some(arms)
}

/// Round-dollar stencil picker.
///
/// Input: current EMA histogram, previous reference bin, and search bounds.
/// Output: next reference bin. Internal state is only the adaptive shape profile
/// used by the slow cold-start regime.
#[derive(Clone)]
pub(super) struct Stencil {
    shape: ShapeAnchor,
}

impl Stencil {
    pub(super) fn new(shape_weight: f64) -> Self {
        Self {
            shape: ShapeAnchor::new(shape_weight),
        }
    }

    pub(super) fn pick(
        &mut self,
        ema: &HistogramEma,
        prev_bin: f64,
        search_below: usize,
        search_above: usize,
    ) -> f64 {
        let ref_bin = find_best_bin(ema, prev_bin, search_below, search_above, &self.shape);
        self.shape.update(ema, ref_bin.round() as i64);
        ref_bin
    }
}

/// Adaptive shape-anchoring restoring force for the slow cold-start regime.
///
/// Holds a round-USD shape template (`profile`), re-estimated each block from the
/// arm vector at the pick, and adds a per-candidate score pulling the search
/// toward the octave whose payment shape looks real. This lets the slow EMA
/// resist round-USD octave aliasing in the thin pre-2018 output mix.
///
/// A zero `weight` makes it inert ([`score`](Self::score) returns 0,
/// [`update`](Self::update) is a no-op), so the fast regime carries it for free
/// without call sites special-casing the disabled path.
#[derive(Clone)]
struct ShapeAnchor {
    weight: f64,
    /// Seeded flat (every arm equal). The slow EMA learns the real payment shape
    /// within a few hundred blocks, so no hand-tuned starting guess is needed.
    profile: Arms,
}

impl ShapeAnchor {
    fn new(weight: f64) -> Self {
        Self {
            weight,
            profile: Arms([1.0 / N_ARMS as f64; N_ARMS]),
        }
    }

    /// Restoring-force contribution to a candidate bin's score: `weight` times the
    /// shape match against the learned profile. 0 when inert or the bin is empty.
    fn score(&self, ema: &HistogramEma, bin: i64) -> f64 {
        if self.weight == 0.0 {
            return 0.0;
        }
        self.weight * self.shape_match(ema, bin)
    }

    /// Blend the L1-normalized arm shape at `pick` into the profile (slow EMA,
    /// [`SHAPE_BETA`]). No-op when inert or the pick is empty.
    fn update(&mut self, ema: &HistogramEma, pick: i64) {
        if self.weight == 0.0 {
            return;
        }
        if let Some(arms) = normalized_arms_at(ema, pick) {
            (0..N_ARMS).for_each(|i| {
                self.profile[i] = (1.0 - SHAPE_BETA) * self.profile[i] + SHAPE_BETA * arms[i];
            });
        }
    }

    /// Shape match `1 - L1distance` between the candidate's L1-normalized arm
    /// vector and the profile. 1.0 is an identical shape and it falls as mass
    /// shifts off the round-USD ladder. 0 for an empty (no-mass) center.
    fn shape_match(&self, ema: &HistogramEma, center: i64) -> f64 {
        match normalized_arms_at(ema, center) {
            Some(arms) => {
                1.0 - (0..N_ARMS)
                    .map(|i| (arms[i] - self.profile[i]).abs())
                    .sum::<f64>()
            }
            None => 0.0,
        }
    }
}

struct CandidateScorer<'a> {
    ema: &'a HistogramEma,
    shape: &'a ShapeAnchor,
    range: Range<usize>,
    arm_peaks: Arms,
}

impl<'a> CandidateScorer<'a> {
    fn new(ema: &'a HistogramEma, shape: &'a ShapeAnchor, range: Range<usize>) -> Self {
        Self {
            ema,
            shape,
            arm_peaks: arm_peaks(ema, range.clone()),
            range,
        }
    }

    fn score(&self, bin: usize) -> f64 {
        let mut total = 0.0;
        for (i, &offset) in STENCIL_OFFSETS.iter().enumerate() {
            if self.arm_peaks[i] > 0.0 {
                total += bin_value(self.ema, bin as i64 + offset as i64) / self.arm_peaks[i];
            }
        }
        total + self.shape.score(self.ema, bin as i64)
    }

    fn best_bin(&self) -> (usize, f64) {
        let mut bins = self.range.clone();
        let mut best_bin = bins.next().expect("candidate range must not be empty");
        let mut best_score = self.score(best_bin);

        for bin in bins {
            let candidate = self.score(bin);
            if candidate > best_score {
                best_score = candidate;
                best_bin = bin;
            }
        }

        (best_bin, best_score)
    }

    /// Parabolic sub-bin interpolation for fractional precision.
    fn interpolated_bin(&self, best_bin: usize, best_score: f64) -> f64 {
        let score_center = best_score;
        let score_left = if best_bin > self.range.start {
            self.score(best_bin - 1)
        } else {
            score_center
        };
        let score_right = if best_bin + 1 < self.range.end {
            self.score(best_bin + 1)
        } else {
            score_center
        };
        let denom = score_left - 2.0 * score_center + score_right;
        let sub_bin = if denom.abs() > 1e-10 {
            (0.5 * (score_left - score_right) / denom).clamp(-0.5, 0.5)
        } else {
            0.0
        };

        best_bin as f64 + sub_bin
    }
}

fn search_range(prev_bin: f64, search_below: usize, search_above: usize) -> Option<Range<usize>> {
    let center = prev_bin.round() as usize;
    let search_start = center.saturating_sub(search_below);
    let search_end = (center + search_above + 1).min(NUM_BINS);

    (search_start < search_end).then_some(search_start..search_end)
}

fn arm_peaks(ema: &HistogramEma, range: Range<usize>) -> Arms {
    let mut peaks = Arms([0.0; N_ARMS]);
    for (i, &offset) in STENCIL_OFFSETS.iter().enumerate() {
        for bin in range.clone() {
            peaks[i] = peaks[i].max(bin_value(ema, bin as i64 + offset as i64));
        }
    }
    peaks
}

/// Scores each candidate bin in the search window by summing normalized stencil
/// matches across the EMA histogram, then refines with parabolic interpolation.
/// Each candidate also picks up `shape`'s shape-anchoring restoring force, which
/// is inert (adds 0) outside the slow cold-start regime.
fn find_best_bin(
    ema: &HistogramEma,
    prev_bin: f64,
    search_below: usize,
    search_above: usize,
    shape: &ShapeAnchor,
) -> f64 {
    let Some(range) = search_range(prev_bin, search_below, search_above) else {
        return prev_bin;
    };

    let scorer = CandidateScorer::new(ema, shape, range);
    let (best_bin, best_score) = scorer.best_bin();
    scorer.interpolated_bin(best_bin, best_score)
}
