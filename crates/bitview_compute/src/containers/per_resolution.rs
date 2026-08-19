use bitview_traversable::Traversable;

#[derive(Clone, Traversable)]
#[traversable(merge)]
pub struct PerResolution<M10, M30, H1, H4, H12, D1, D3, W1, Mo1, Mo3, Mo6, Y1, Y10, HE, DE> {
    pub minute10: M10,
    pub minute30: M30,
    pub hour1: H1,
    pub hour4: H4,
    pub hour12: H12,
    pub day1: D1,
    pub day3: D3,
    pub week1: W1,
    pub month1: Mo1,
    pub month3: Mo3,
    pub month6: Mo6,
    pub year1: Y1,
    pub year10: Y10,
    pub halving: HE,
    pub epoch: DE,
}
