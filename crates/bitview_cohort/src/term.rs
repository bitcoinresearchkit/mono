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
    /// Threshold in hours (150 days * 24 hours = 3600 hours)
    pub const THRESHOLD_HOURS: usize = 24 * 150; // 3600
}
