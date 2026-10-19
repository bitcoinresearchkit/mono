pub(crate) const SEPARATORS: &[char] = &['_', '-', ' ', ':', '/'];
/// Budget of trigrams to process from unknown words, distributed fairly across
/// them. The same bound limits adjacent-letter correction probes.
pub(crate) const TRIGRAM_BUDGET: usize = 6;
pub(crate) const MIN_SCORE: usize = 2;
const DEFAULT_LIMIT: usize = 100;

pub struct QuickMatchConfig {
    /// Maximum number of results to return (at least 1). Default: 100.
    pub(crate) limit: usize,
    /// Allow partial matches through the union of the known query words: fuzzy searches use it when
    /// those words are disjoint, exact-word searches always. Default: true.
    pub(crate) union_fallback: bool,
}

impl Default for QuickMatchConfig {
    fn default() -> Self {
        Self {
            limit: DEFAULT_LIMIT,
            union_fallback: true,
        }
    }
}

impl QuickMatchConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_limit(mut self, limit: usize) -> Self {
        self.limit = limit.max(1);
        self
    }

    pub fn with_union_fallback(mut self, union_fallback: bool) -> Self {
        self.union_fallback = union_fallback;
        self
    }
}
