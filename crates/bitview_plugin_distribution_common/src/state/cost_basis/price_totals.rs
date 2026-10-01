use bitview_compute::FenwickNode;

/// Additive supply and creation-value totals for overlapping price-index filters.
#[derive(Clone, Copy, Debug)]
pub struct PriceTotals<const N: usize> {
    pub sats: [i64; N],
    pub cap: [i128; N],
}

impl<const N: usize> Default for PriceTotals<N> {
    fn default() -> Self {
        Self {
            sats: [0; N],
            cap: [0; N],
        }
    }
}

impl<const N: usize> FenwickNode for PriceTotals<N> {
    #[inline(always)]
    fn add_assign(&mut self, other: &Self) {
        for (value, delta) in self.sats.iter_mut().zip(other.sats) {
            *value += delta;
        }
        for (value, delta) in self.cap.iter_mut().zip(other.cap) {
            *value += delta;
        }
    }
}
