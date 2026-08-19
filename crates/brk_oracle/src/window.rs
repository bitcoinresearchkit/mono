use crate::{HistogramEma, HistogramRaw};

/// A sliding window of the most recent raw block histograms and their weighted
/// exponential moving average.
///
/// [`push`](Self::push) records a block into a fixed-size ring.
/// [`recompute`](Self::recompute) folds the ring into [`ema`](Self::ema) with
/// per-age weights `alpha * (1 - alpha)^age` (newest block is age 0). Recording
/// and recomputing are separate steps so warm-up can fill the ring without
/// paying for the EMA until the first real query.
#[derive(Clone)]
pub(crate) struct EmaWindow {
    histograms: Vec<HistogramRaw>,
    weights: Vec<f64>,
    ema: Box<HistogramEma>,
    cursor: usize,
    filled: usize,
}

impl EmaWindow {
    pub(crate) fn new(window_size: usize, alpha: f64) -> Self {
        let decay = 1.0 - alpha;
        let weights = (0..window_size)
            .map(|i| alpha * decay.powi(i as i32))
            .collect();
        Self {
            histograms: vec![HistogramRaw::zeros(); window_size],
            weights,
            ema: Box::new(HistogramEma::zeros()),
            cursor: 0,
            filled: 0,
        }
    }

    /// Record `hist` as the newest block, evicting the oldest once full.
    pub(crate) fn push(&mut self, hist: &HistogramRaw) {
        let window = self.histograms.len();
        self.histograms[self.cursor] = hist.clone();
        self.cursor = (self.cursor + 1) % window;
        self.filled = (self.filled + 1).min(window);
    }

    /// Ring index of the block `age` steps back from the newest (age 0).
    fn index_at_age(&self, age: usize) -> usize {
        let window = self.histograms.len();
        (self.cursor + window - 1 - age) % window
    }

    /// Fold the ring into the weighted EMA, newest block weighted `weights[0]`.
    pub(crate) fn recompute(&mut self) {
        self.ema.fill(0.0);
        for age in 0..self.filled {
            let weight = self.weights[age];
            let h = &self.histograms[self.index_at_age(age)];
            self.ema
                .iter_mut()
                .zip(h.iter())
                .for_each(|(e, &c)| *e += weight * c as f64);
        }
    }

    pub(crate) fn ema(&self) -> &HistogramEma {
        &self.ema
    }

    /// The most recent `min(filled, n)` raw histograms, oldest first - the
    /// hand-off a regime switch replays into a fresh window of size `n`.
    pub(crate) fn recent(&self, n: usize) -> Vec<HistogramRaw> {
        (0..self.filled.min(n))
            .rev()
            .map(|age| self.histograms[self.index_at_age(age)].clone())
            .collect()
    }
}
