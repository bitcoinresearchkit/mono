const DEFAULT_SEPARATORS: &[char] = &['_', '-', ' ', ':', '/'];
const DEFAULT_TRIGRAM_BUDGET: usize = 6;
const DEFAULT_LIMIT: usize = 100;
const DEFAULT_MIN_SCORE: usize = 2;

pub struct QuickMatchConfig {
    /// Separators used to split words.
    ///
    /// Default: ['_', '-', ' ', ':', '/']
    separators: &'static [char],
    /// Maximum number of results to return.
    ///
    /// Default: 100
    /// - Min: 1
    /// - Max: No hard limit (but large values may impact performance)
    limit: usize,
    /// Budget of trigrams to process from unknown words.
    /// This budget is distributed fairly across all unknown words.
    ///
    /// Default: 6 (recommended: 3-9)
    /// - 0: Disable trigram matching (only exact word matches)
    /// - Low (3-6): Faster, less accurate fuzzy matching
    /// - High (9-15): Slower, more accurate fuzzy matching
    /// - Max: 20
    trigram_budget: usize,
    /// Whether disjoint known query words fall back to their union.
    ///
    /// Default: true
    union_fallback: bool,
    /// Minimum trigram score required for fuzzy matches.
    /// Higher values require more trigram overlap, reducing noise.
    ///
    /// Default: 2
    /// - Min: 1
    min_score: usize,
}

impl Default for QuickMatchConfig {
    fn default() -> Self {
        Self {
            separators: DEFAULT_SEPARATORS,
            limit: DEFAULT_LIMIT,
            trigram_budget: DEFAULT_TRIGRAM_BUDGET,
            union_fallback: true,
            min_score: DEFAULT_MIN_SCORE,
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

    pub fn with_trigram_budget(mut self, trigram_budget: usize) -> Self {
        self.trigram_budget = trigram_budget.clamp(0, 20);
        self
    }

    pub fn with_union_fallback(mut self, union_fallback: bool) -> Self {
        self.union_fallback = union_fallback;
        self
    }

    pub fn with_separators(mut self, separators: &'static [char]) -> Self {
        self.separators = separators;
        self
    }

    pub fn with_min_score(mut self, min_score: usize) -> Self {
        self.min_score = min_score.max(1);
        self
    }

    pub fn limit(&self) -> usize {
        self.limit
    }

    pub fn trigram_budget(&self) -> usize {
        self.trigram_budget
    }

    pub fn union_fallback(&self) -> bool {
        self.union_fallback
    }

    pub fn separators(&self) -> &[char] {
        self.separators
    }

    pub fn min_score(&self) -> usize {
        self.min_score
    }
}
