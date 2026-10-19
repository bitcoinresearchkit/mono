/// Classification for short-term vs long-term holders.
/// The threshold is 150 days (approximately 5 months) = 3600 hours.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Term {
    /// Short-Term Holder: < 150 days
    Sth,
    /// Long-Term Holder: >= 150 days
    Lth,
}

impl Term {
    pub const THRESHOLD_HOURS: usize = 24 * 150;
}
