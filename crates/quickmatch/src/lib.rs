use std::{borrow::Cow, iter};

use rustc_hash::{FxHashMap, FxHashSet};

mod config;

pub use config::*;

#[derive(Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct ItemId(u32);

/// Instant search over a list of strings.
///
/// Supports exact words, prefixes ("dom" → "dominance"), joined words
/// ("hashrate" → "hash_rate"), and typo tolerance ("suply" → "supply").
/// Results are ranked: exact matches first, then by specificity.
pub struct QuickMatch<'a> {
    config: QuickMatchConfig,
    items: Vec<Cow<'a, str>>,
    item_rank: Vec<ItemId>,
    max_word_count: usize,
    max_word_len: usize,
    max_query_len: usize,
    word_index: FxHashMap<String, Vec<ItemId>>,
    trigram_index: FxHashMap<[char; 3], Vec<ItemId>>,
}

impl<'a> QuickMatch<'a> {
    /// Expect the items to be pre-formatted (lowercase)
    pub fn new(items: &[&'a str]) -> Self {
        Self::new_with(items, QuickMatchConfig::default())
    }

    /// Expect the items to be pre-formatted (lowercase)
    pub fn new_with(items: &[&'a str], config: QuickMatchConfig) -> Self {
        Self::build(items.iter().copied().map(Cow::Borrowed).collect(), config)
    }

    fn build(items: Vec<Cow<'a, str>>, config: QuickMatchConfig) -> Self {
        assert!(u32::try_from(items.len()).is_ok(), "Too many items");

        let mut word_index: FxHashMap<String, Vec<ItemId>> = FxHashMap::default();
        let mut trigram_index: FxHashMap<[char; 3], Vec<ItemId>> = FxHashMap::default();
        let mut max_word_len = 0;
        let mut max_query_len = 0;
        let mut max_words = 0;
        let sep = sep_table(config.separators());

        for (id, item) in items.iter().enumerate() {
            let id = ItemId(id as u32);
            let item = item.as_ref();
            max_query_len = max_query_len.max(item.len());
            let item_words: Vec<&str> = words(item, &sep).collect();
            max_words = max_words.max(item_words.len());

            for word in &item_words {
                max_word_len = max_word_len.max(word.len());

                for len in 1..=word.len() {
                    Self::insert_word(&mut word_index, &word[..len], id);
                }

                let mut chars = word.chars();
                if let (Some(mut a), Some(mut b)) = (chars.next(), chars.next()) {
                    for c in chars {
                        let items = trigram_index.entry([a, b, c]).or_default();
                        if items.last() != Some(&id) {
                            items.push(id);
                        }
                        a = b;
                        b = c;
                    }
                }
            }

            for pair in item_words.windows(2) {
                let compound = format!("{}{}", pair[0], pair[1]);
                // A joined-word query ("hashrate") can be longer than any
                // single word. Capping at the longest index key keeps the
                // DDoS guard data-bounded while still letting it match.
                max_word_len = max_word_len.max(compound.len());
                let from = pair[0].len() + 1;
                for len in from..=compound.len() {
                    Self::insert_word(&mut word_index, &compound[..len], id);
                }
            }
        }

        let item_rank = if items
            .windows(2)
            .all(|pair| item_order(pair[0].as_ref(), pair[1].as_ref()).is_le())
        {
            (0..items.len()).map(|id| ItemId(id as u32)).collect()
        } else {
            let mut ranked_ids = (0..items.len())
                .map(|id| ItemId(id as u32))
                .collect::<Vec<_>>();
            ranked_ids.sort_unstable_by(|a, b| {
                item_order(items[a.0 as usize].as_ref(), items[b.0 as usize].as_ref())
            });
            let mut item_rank = vec![ItemId::default(); items.len()];
            for (rank, id) in ranked_ids.into_iter().enumerate() {
                item_rank[id.0 as usize] = ItemId(rank as u32);
            }
            item_rank
        };

        Self {
            max_query_len: max_query_len + 6,
            max_word_len: max_word_len + 4,
            max_word_count: max_words + 2,
            items,
            item_rank,
            word_index,
            trigram_index,
            config,
        }
    }

    fn insert_word(index: &mut FxHashMap<String, Vec<ItemId>>, word: &str, id: ItemId) {
        if let Some(items) = index.get_mut(word) {
            if items.last() != Some(&id) {
                items.push(id);
            }
        } else {
            index.insert(word.to_owned(), vec![id]);
        }
    }

    pub fn matches(&self, query: &str) -> Vec<&str> {
        self.matches_with(query, &self.config)
    }

    pub fn matches_with(&self, query: &str, config: &QuickMatchConfig) -> Vec<&str> {
        self.matches_with_matched_words(query, config)
            .into_iter()
            .map(|(item, _)| item)
            .collect()
    }

    /// Matches with the number of query words found in each result.
    pub fn matches_with_matched_words(
        &self,
        query: &str,
        config: &QuickMatchConfig,
    ) -> Vec<(&str, usize)> {
        self.matches_with_ids_and_matched_words(query, config)
            .into_iter()
            .map(|(id, matched_words)| (self.item(ItemId(id)), matched_words as usize))
            .collect()
    }

    /// Matches and returns each result's compact zero-based position in the
    /// original item slice and matched query-word count.
    pub fn matches_with_ids_and_matched_words(
        &self,
        query: &str,
        config: &QuickMatchConfig,
    ) -> Vec<(u32, u32)> {
        let limit = config.limit().min(self.items.len());
        let trigram_budget = config.trigram_budget();

        if limit == 0 {
            return vec![];
        }

        let query: String = query
            .trim()
            .chars()
            .filter(|c| c.is_ascii())
            .map(|c| c.to_ascii_lowercase())
            .collect();

        if query.is_empty() || query.len() > self.max_query_len {
            return vec![];
        }

        let sep = sep_table(config.separators());

        let mut query_words: Vec<&str> = vec![];
        for w in words(&query, &sep) {
            if w.len() <= self.max_word_len && !query_words.contains(&w) {
                query_words.push(w);
            }
        }

        if query_words.is_empty() || query_words.len() > self.max_word_count {
            return vec![];
        }

        let mut unknown_words: Vec<&str> = vec![];
        let mut known_lists: Vec<&[ItemId]> = vec![];

        for &word in &query_words {
            if let Some(items) = self.word_index.get(word) {
                known_lists.push(items)
            } else if word.len() >= 3 && unknown_words.len() < trigram_budget {
                unknown_words.push(word)
            }
        }

        let pool = Self::intersect_lists(&known_lists);

        // Try typo matching for unknown words
        if !unknown_words.is_empty() && trigram_budget > 0 {
            let min_len = query.len().saturating_sub(3);
            let (scores, hit_count) =
                self.score_trigrams(&unknown_words, trigram_budget, pool.as_deref(), min_len);
            let min_score = hit_count.div_ceil(2).max(config.min_score());
            let results = self.rank(
                scores.into_iter().filter(|(_, s)| *s >= min_score),
                &query_words,
                &sep,
                limit,
            );

            if !results.is_empty() {
                return results
                    .into_iter()
                    .map(|(id, matched)| (id.0, matched))
                    .collect();
            }
        }

        // Rank known candidates (intersection, or union as fallback)
        let candidates = pool.unwrap_or_else(|| {
            if config.union_fallback() {
                Self::union_lists(&known_lists)
            } else {
                Vec::new()
            }
        });
        self.rank(
            candidates.into_iter().map(|id| (id, 0)),
            &query_words,
            &sep,
            limit,
        )
        .into_iter()
        .map(|(id, matched)| (id.0, matched))
        .collect()
    }

    /// Intersection of all posting lists, or `None` when there are no lists or
    /// no overlap. IDs are appended in ascending order while the index builds.
    fn intersect_lists(lists: &[&[ItemId]]) -> Option<Vec<ItemId>> {
        let (smallest_index, smallest) = lists
            .iter()
            .copied()
            .enumerate()
            .min_by_key(|(_, items)| items.len())?;
        let result = smallest
            .iter()
            .copied()
            .filter(|item| {
                lists.iter().enumerate().all(|(index, items)| {
                    index == smallest_index || items.binary_search(item).is_ok()
                })
            })
            .collect::<Vec<_>>();

        (!result.is_empty()).then_some(result)
    }

    /// Union of all posting lists.
    fn union_lists(lists: &[&[ItemId]]) -> Vec<ItemId> {
        lists
            .iter()
            .flat_map(|items| items.iter().copied())
            .collect::<FxHashSet<_>>()
            .into_iter()
            .collect()
    }

    /// Bucket by matched-word count, then sort each needed bucket by fuzzy
    /// score, match position, and length.
    fn rank(
        &self,
        candidates: impl IntoIterator<Item = (ItemId, usize)>,
        query_words: &[&str],
        sep: &[bool; 256],
        limit: usize,
    ) -> Vec<(ItemId, u32)> {
        let mut buckets: Vec<Vec<(ItemId, usize, usize, ItemId)>> =
            vec![vec![]; query_words.len() + 1];

        for (item, fuzzy) in candidates {
            let s = self.item(item);
            let (matched, position) = word_match(s, query_words, sep);
            buckets[matched].push((item, fuzzy, position, self.item_rank[item.0 as usize]));
        }

        let mut results = Vec::with_capacity(limit);
        for (matched, bucket) in buckets.iter_mut().enumerate().rev() {
            if bucket.is_empty() {
                continue;
            }
            let order = |a: &(ItemId, usize, usize, ItemId), b: &(ItemId, usize, usize, ItemId)| {
                b.1.cmp(&a.1) // fuzzy score, desc
                    .then(a.2.cmp(&b.2)) // match position, asc
                    .then(a.3.cmp(&b.3)) // item length then text, asc
            };
            let take = (limit - results.len()).min(bucket.len());
            if take < bucket.len() {
                bucket.select_nth_unstable_by(take, order);
            }
            bucket[..take].sort_unstable_by(order);
            results.extend(bucket[..take].iter().map(|&(id, ..)| (id, matched as u32)));
            if results.len() >= limit {
                break;
            }
        }

        results
    }

    fn item(&self, id: ItemId) -> &str {
        self.items[id.0 as usize].as_ref()
    }

    /// Builds per-item trigram-overlap scores for the unknown (typo) words.
    /// With a `pool`, only pooled items can score (each pre-seeded to 1);
    /// otherwise any item at least `min_len` chars long is eligible. Returns
    /// the score map and how many probed trigrams were found in the index.
    fn score_trigrams(
        &self,
        unknown_words: &[&str],
        trigram_budget: usize,
        pool: Option<&[ItemId]>,
        min_len: usize,
    ) -> (FxHashMap<ItemId, usize>, usize) {
        let mut scores: FxHashMap<ItemId, usize> = FxHashMap::default();
        scores.reserve(256);
        if let Some(pool) = pool {
            for &item in pool {
                scores.insert(item, 1);
            }
        }
        let has_pool = pool.is_some();

        let mut budget = trigram_budget;
        let mut hit_count = 0;
        let mut visited: FxHashSet<[char; 3]> = FxHashSet::default();

        'outer: for round in 0..trigram_budget {
            for word in unknown_words {
                if budget == 0 {
                    break 'outer;
                }

                let bytes = word.as_bytes();
                let Some(pos) = trigram_position(bytes.len(), round) else {
                    continue;
                };
                let trigram = [
                    bytes[pos] as char,
                    bytes[pos + 1] as char,
                    bytes[pos + 2] as char,
                ];

                if !visited.insert(trigram) {
                    continue;
                }
                budget -= 1;

                let Some(items) = self.trigram_index.get(&trigram) else {
                    continue;
                };
                hit_count += 1;

                if has_pool {
                    for &item in items {
                        if let Some(score) = scores.get_mut(&item) {
                            *score += 1;
                        }
                    }
                } else {
                    for &item in items {
                        if self.item(item).len() >= min_len {
                            *scores.entry(item).or_default() += 1;
                        }
                    }
                }
            }
        }

        (scores, hit_count)
    }
}

impl QuickMatch<'static> {
    /// Builds a matcher that owns its pre-formatted (lowercase) items.
    pub fn new_owned(items: Vec<String>) -> Self {
        Self::new_owned_with(items, QuickMatchConfig::default())
    }

    /// Builds a matcher that owns its pre-formatted (lowercase) items.
    pub fn new_owned_with(items: Vec<String>, config: QuickMatchConfig) -> Self {
        Self::build(items.into_iter().map(Cow::Owned).collect(), config)
    }
}

fn item_order(a: &str, b: &str) -> std::cmp::Ordering {
    a.len().cmp(&b.len()).then_with(|| a.cmp(b))
}

/// Builds a byte lookup table from the configured separator chars. Separators
/// are ASCII, so a byte-indexed table is exact even for multi-byte UTF-8:
/// continuation and lead bytes are all >= 128 and never flagged.
fn sep_table(separators: &[char]) -> [bool; 256] {
    let mut table = [false; 256];
    for &c in separators {
        if (c as usize) < 256 {
            table[c as usize] = true;
        }
    }
    table
}

/// Splits `text` into non-empty words on any separator byte flagged in `sep`.
fn words<'s>(text: &'s str, sep: &'s [bool; 256]) -> impl Iterator<Item = &'s str> {
    let bytes = text.as_bytes();
    let mut i = 0;
    iter::from_fn(move || {
        while i < bytes.len() && sep[bytes[i] as usize] {
            i += 1;
        }
        let start = i;
        while i < bytes.len() && !sep[bytes[i] as usize] {
            i += 1;
        }
        (i > start).then(|| &text[start..i])
    })
}

/// Aligns the query words against the item's words, in order:
/// - `matched`: query words matched as an in-order subsequence of item words
/// - `position`: index of the item word where that run starts (or the item's
///   word count when nothing matched)
fn word_match(item: &str, query_words: &[&str], sep: &[bool; 256]) -> (usize, usize) {
    let mut matched = 0;
    let mut position = 0;
    for iw in words(item, sep) {
        if query_words
            .get(matched)
            .is_some_and(|qw| iw.starts_with(*qw))
        {
            matched += 1;
        } else if matched == 0 {
            position += 1;
        }
    }
    (matched, position)
}

/// Picks which trigram of a length-`len` word to probe on `round`, spreading
/// probes outward from the two ends toward the middle. Returns `None` when the
/// round offers no fresh position.
fn trigram_position(len: usize, round: usize) -> Option<usize> {
    let max = len - 3;
    if round == 0 {
        return Some(0);
    }
    if round == 1 && max > 0 {
        return Some(max);
    }
    if round == 2 && max > 1 {
        return Some(max / 2);
    }
    if max <= 2 {
        return None;
    }

    let mid = max / 2;
    let offset = (round - 2) >> 1;
    let pos = if round & 1 == 1 {
        mid.saturating_sub(offset)
    } else {
        mid + offset
    };
    if pos == 0 || pos >= max || pos == mid {
        None
    } else {
        Some(pos)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ITEMS: &[&str] = &[
        "hash_rate",
        "realized_price",
        "supply_in_profit",
        "sth_realized_price",
        "dominance",
    ];

    #[test]
    fn owned_and_borrowed_matchers_are_equivalent() {
        let borrowed = QuickMatch::new(ITEMS);
        let owned = QuickMatch::new_owned(ITEMS.iter().map(|item| (*item).to_string()).collect());
        let config = QuickMatchConfig::new().with_limit(ITEMS.len());

        for query in [
            "hashrate",
            "realized price",
            "suply",
            "dom",
            "sth realized price",
            "missing",
        ] {
            assert_eq!(
                borrowed.matches_with_matched_words(query, &config),
                owned.matches_with_matched_words(query, &config),
                "owned matcher changed results for {query}"
            );

            let indexed = borrowed.matches_with_ids_and_matched_words(query, &config);
            let resolved = indexed
                .iter()
                .map(|&(id, matched)| (ITEMS[id as usize], matched as usize))
                .collect::<Vec<_>>();
            assert_eq!(
                resolved,
                borrowed.matches_with_matched_words(query, &config),
                "indexed API changed results for {query}"
            );
        }

        assert_eq!(borrowed.matches("hashrate")[0], "hash_rate");
        assert_eq!(borrowed.matches("realized price")[0], "realized_price");
        assert_eq!(borrowed.matches("suply")[0], "supply_in_profit");
        assert_eq!(borrowed.matches("dom")[0], "dominance");
    }

    #[test]
    fn union_fallback_remains_configurable() {
        let items = ["alpha_x", "beta_y"];
        let matcher = QuickMatch::new(&items);
        let union = QuickMatchConfig::new().with_limit(2);
        let intersection_only = QuickMatchConfig::new()
            .with_limit(2)
            .with_union_fallback(false);

        assert_eq!(matcher.matches_with("alpha beta", &union).len(), 2);
        assert!(
            matcher
                .matches_with("alpha beta", &intersection_only)
                .is_empty()
        );
    }

    #[test]
    fn matcher_is_naturally_send_and_sync() {
        fn assert_send_sync<T: Send + Sync>() {}

        assert_send_sync::<QuickMatch<'static>>();
    }
}
