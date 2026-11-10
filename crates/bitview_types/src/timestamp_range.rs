use brk_types::Timestamp;

/// Inclusive raw header timestamps. Validated once before a conditional response
/// can bypass the query; callers cannot construct a descending range.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TimestampRange {
    from: Timestamp,
    to: Timestamp,
}

impl TimestampRange {
    pub fn new(from: Timestamp, to: Timestamp) -> Option<Self> {
        (from <= to).then_some(Self { from, to })
    }

    pub fn from(self) -> Timestamp {
        self.from
    }

    pub fn to(self) -> Timestamp {
        self.to
    }

    /// Original requested span, independent of published chain coverage.
    pub fn span(self) -> u32 {
        *self.to - *self.from
    }

    pub fn contains(self, timestamp: Timestamp) -> bool {
        self.from <= timestamp && timestamp <= self.to
    }
}
