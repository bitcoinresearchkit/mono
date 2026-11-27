//! An exact two-parameter quantile regression, refit as points arrive.
//!
//! The best line passes through two of the points. Turned about one point, the loss as a function of
//! the slope is `Σ w·ρτ(s − slope)` over the slopes `s` to the other points, with `w = |Δx|` and `τ`
//! the quantile right of the point and its complement left of it; its minimum is the weighted
//! quantile of those slopes. So the fit turns the line about a point to that slope, then about the
//! point the slope reaches, until the loss stops falling (Wesolowsky's descent for least absolute
//! deviations, its weights made asymmetric). The line is linear on each of the four cones the two
//! points' residuals cut, and the two turns check all four edges, so where neither lowers the loss
//! the line is the optimum: exact, with no tolerance on the slope. Each refit starts from the
//! previous one's point and usually takes two turns.

/// A slope from the turning point, its weight and the point it reaches.
type Slope = (f64, f64, u32);

pub(super) struct Points {
    quantile: f64,
    x: Vec<f64>,
    y: Vec<f64>,
    /// The point the last fit turned on, where the next one starts.
    pivot: usize,
    slopes: Vec<Slope>,
}

impl Points {
    pub(super) fn new(quantile: f64) -> Self {
        Self { quantile, x: Vec::new(), y: Vec::new(), pivot: 0, slopes: Vec::new() }
    }

    pub(super) fn len(&self) -> usize {
        self.x.len()
    }

    pub(super) fn push(&mut self, x: f64, y: f64) {
        debug_assert!(x.is_finite() && y.is_finite(), "a point must be finite: ({x}, {y})");
        self.x.push(x);
        self.y.push(y);
    }

    /// The quantile line through the points so far, as `(intercept, slope)`. Needs two points of
    /// different `x`.
    pub(super) fn fit(&mut self) -> (f64, f64) {
        let (x, y, q) = (&self.x, &self.y, self.quantile);
        let (mut best, mut basis, mut p) = (f64::INFINITY, (0, 0), self.pivot);
        loop {
            self.slopes.clear();
            let mut target = 0.0;
            for (k, (&xk, &yk)) in x.iter().zip(y).enumerate() {
                let dx = xk - x[p];
                if dx != 0.0 {
                    self.slopes.push(((yk - y[p]) / dx, dx.abs(), k as u32));
                    target += dx.abs() * if dx > 0.0 { q } else { 1.0 - q };
                }
            }
            let (slope, _, k) = weighted_quantile(&mut self.slopes, target);
            let intercept = y[p] - slope * x[p];
            let loss: f64 = x
                .iter()
                .zip(y)
                .map(|(&xi, &yi)| {
                    let e = yi - intercept - slope * xi;
                    if e >= 0.0 { q * e } else { (q - 1.0) * e }
                })
                .sum();
            // A turn that doesn't lower the loss: the line before it was the best.
            if loss >= best - 1e-12 {
                break;
            }
            (best, basis, self.pivot, p) = (loss, (p, k as usize), p, k as usize);
        }
        // The line from its two points in index order, so a refit from another starting point
        // (after a restart) gives the same bits.
        let (i, j) = (basis.0.min(basis.1), basis.0.max(basis.1));
        let slope = (y[j] - y[i]) / (x[j] - x[i]);
        (y[i] - slope * x[i], slope)
    }
}

/// The smallest slope whose weight, with every smaller slope's, reaches `target`, by weighted
/// quickselect: expected O(n). `target` is under the total weight, so one is always reached.
fn weighted_quantile(mut slopes: &mut [Slope], mut target: f64) -> Slope {
    loop {
        let middle = slopes.len() / 2;
        let (under, at, over) = slopes.select_nth_unstable_by(middle, |a, b| a.0.total_cmp(&b.0));
        let below: f64 = under.iter().map(|s| s.1).sum();
        if target <= below {
            slopes = under;
        } else if target <= below + at.1 || over.is_empty() {
            return *at;
        } else {
            target -= below + at.1;
            slopes = over;
        }
    }
}
