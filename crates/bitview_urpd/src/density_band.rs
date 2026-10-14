use brk_types::Cents;

/// The density band: creation prices within 5% of spot, as exact rational edges on grid prices.
/// A grid price `p` (cents) is inside when `spot * 95 / 100 <= p <= spot * 105 / 100`, and in
/// profit through the spot's own grid price.
#[derive(Clone, Copy)]
pub struct DensityBand {
    spot: Cents,
}

impl DensityBand {
    /// `None` without a positive spot: no band, null densities.
    pub fn new(spot: Cents) -> Option<Self> {
        spot.finite_inner()
            .is_some_and(|spot| spot > 0)
            .then_some(Self { spot })
    }

    pub fn spot(self) -> Cents {
        self.spot
    }

    /// The band starts at the first price `p` with `p * denom >= numer`: `(numer, denom)`.
    pub fn first_inside(self) -> (u128, u128) {
        (self.spot.as_u128() * 95, 100)
    }

    /// The band ends before the first price `p` with `p * denom >= numer`: `(numer, denom)`.
    pub fn first_above(self) -> (u128, u128) {
        (self.spot.as_u128() * 105 + 1, 100)
    }
}
