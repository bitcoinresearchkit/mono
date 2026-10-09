use std::{borrow::Cow, cmp::Ordering, iter, mem};

use rustc_hash::{FxHashMap, FxHashSet};

mod config;

pub use config::*;

#[derive(Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct ItemId(u32);

type RankedItem = (ItemId, usize, usize, ItemId);

fn ranked_item_order(a: &RankedItem, b: &RankedItem) -> Ordering {
    b.1.cmp(&a.1) // fuzzy score, desc
        .then(a.2.cmp(&b.2)) // match position, asc
        .then(a.3.cmp(&b.3)) // item length then text, asc
}

fn select_and_sort(bucket: &mut [RankedItem], take: usize) {
    if take < bucket.len() {
        bucket.select_nth_unstable_by(take, ranked_item_order);
    }
    bucket[..take].sort_unstable_by(ranked_item_order);
}

/// Instant search over a list of strings.
///
/// Supports exact words, prefixes ("dom" → "dominance"), joined words
/// ("hashrate" → "hash_rate"), and typo tolerance ("suply" → "supply").
/// Results are ranked: exact matches first, then by specificity.
pub struct QuickMatch<'a> {
    items: Vec<Cow<'a, str>>,
    item_rank: Vec<ItemId>,
    max_word_count: usize,
    max_word_len: usize,
    max_query_len: usize,
    word_index: FxHashMap<String, Vec<ItemId>>,
    trigram_index: FxHashMap<[u8; 3], Vec<ItemId>>,
}

impl<'a> QuickMatch<'a> {
    /// Build from pre-formatted lowercase ASCII items.
    ///
    /// # Panics
    /// Panics if an item is not lowercase ASCII or there are more than u32::MAX items.
    pub fn new(items: &[&'a str]) -> Self {
        Self::build(items.iter().copied().map(Cow::Borrowed).collect())
    }

    fn build(items: Vec<Cow<'a, str>>) -> Self {
        assert!(u32::try_from(items.len()).is_ok(), "Too many items");
        assert!(
            items
                .iter()
                .all(|item| item.is_ascii() && !item.bytes().any(|byte| byte.is_ascii_uppercase())),
            "QuickMatch items must be lowercase ASCII",
        );

        let mut word_index: FxHashMap<String, Vec<ItemId>> = FxHashMap::default();
        let mut trigram_index: FxHashMap<[u8; 3], Vec<ItemId>> = FxHashMap::default();
        let mut max_word_len = 0;
        let mut max_query_len = 0;
        let mut max_words = 0;
        let sep = sep_table(SEPARATORS);

        for (id, item) in items.iter().enumerate() {
            let id = ItemId(id as u32);
            let item = item.as_ref();
            max_query_len = max_query_len.max(item.len());
            let mut previous_word: Option<&str> = None;
            let mut word_count = 0;
            for word in words(item, &sep) {
                word_count += 1;
                max_word_len = max_word_len.max(word.len());

                for len in 1..=word.len() {
                    Self::insert_word(&mut word_index, &word[..len], id);
                }

                let mut bytes = word.bytes();
                if let (Some(mut a), Some(mut b)) = (bytes.next(), bytes.next()) {
                    for c in bytes {
                        let items = trigram_index.entry([a, b, c]).or_default();
                        if items.last() != Some(&id) {
                            items.push(id);
                        }
                        a = b;
                        b = c;
                    }
                }

                if let Some(previous_word) = previous_word {
                    let compound = format!("{previous_word}{word}");
                    // A joined-word query ("hashrate") can be longer than any
                    // single word. Capping at the longest index key keeps the
                    // DDoS guard data-bounded while still letting it match.
                    max_word_len = max_word_len.max(compound.len());
                    let from = previous_word.len() + 1;
                    for len in from..=compound.len() {
                        Self::insert_word(&mut word_index, &compound[..len], id);
                    }
                }
                previous_word = Some(word);
            }
            max_words = max_words.max(word_count);
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
                    .then(a.cmp(b))
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

    /// Matches and returns each result's compact zero-based position in the
    /// original item slice and matched query-word count.
    pub fn matches_with_ids_and_matched_words(
        &self,
        query: &str,
        config: &QuickMatchConfig,
    ) -> Vec<(u32, u32)> {
        self.matches_with_ids_and_matched_words_inner::<false>(query, config)
    }

    /// Whole query words, in any order, without prefix or typo matching.
    /// With union fallback disabled, every query word must match.
    pub fn matches_exact_with_ids_and_matched_words(
        &self,
        query: &str,
        config: &QuickMatchConfig,
    ) -> Vec<(u32, u32)> {
        self.matches_with_ids_and_matched_words_inner::<true>(query, config)
    }

    fn matches_with_ids_and_matched_words_inner<const EXACT_WORDS: bool>(
        &self,
        query: &str,
        config: &QuickMatchConfig,
    ) -> Vec<(u32, u32)> {
        let limit = config.limit().min(self.items.len());
        let trigram_budget = TRIGRAM_BUDGET;

        if limit == 0 {
            return vec![];
        }

        let query: String = query
            .trim()
            .chars()
            .filter(|c| c.is_ascii())
            .map(|c| c.to_ascii_lowercase())
            .collect();

        if query.is_empty() || (!EXACT_WORDS && query.len() > self.max_query_len) {
            return vec![];
        }

        let sep = sep_table(SEPARATORS);

        let mut query_words: Vec<&str> = vec![];
        for w in words(&query, &sep) {
            if (EXACT_WORDS || w.len() <= self.max_word_len) && !query_words.contains(&w) {
                query_words.push(w);
            }
        }

        if query_words.is_empty() || (!EXACT_WORDS && query_words.len() > self.max_word_count) {
            return vec![];
        }

        let mut unknown_words: Vec<&str> = vec![];
        // Every word of three letters or more the index doesn't hold: each must be
        // a typo of a candidate's (the trigrams probe only some of them; shorter
        // ones are left out, as they are when nothing's mistyped).
        let mut unmatched: Vec<&str> = vec![];
        let mut known_lists: Vec<&[ItemId]> = vec![];

        for &word in &query_words {
            if let Some(items) = self.word_index.get(word) {
                known_lists.push(items)
            } else if word.len() >= 3 {
                unmatched.push(word);
                if unknown_words.len() < trigram_budget {
                    unknown_words.push(word)
                }
            }
        }

        if EXACT_WORDS {
            if !config.union_fallback() && known_lists.len() != query_words.len() {
                return Vec::new();
            }
            let candidates = if config.union_fallback() {
                Self::union_lists(&known_lists)
            } else {
                Self::intersect_lists(&known_lists).unwrap_or_default()
            };
            return self.rank::<true>(
                candidates.into_iter().map(|id| (id, 0)),
                &query_words,
                &sep,
                limit,
                if config.union_fallback() {
                    1
                } else {
                    query_words.len()
                },
            );
        }

        let pool = Self::intersect_lists(&known_lists);

        // A swapped pair often shares no trigrams ("prcie" → "price").
        // Probe indexed corrections before the broader trigram fallback.
        if !unknown_words.is_empty() && trigram_budget > 0 {
            let mut corrections: Vec<Vec<ItemId>> = vec![vec![]; unknown_words.len()];
            let mut budget = trigram_budget;
            for round in 0..trigram_budget {
                for (i, word) in unknown_words.iter().enumerate() {
                    if budget == 0 {
                        break;
                    }
                    let bytes = word.as_bytes();
                    if round >= bytes.len() - 1 {
                        continue;
                    }
                    let at = if round % 2 == 0 {
                        round / 2
                    } else {
                        bytes.len() - 2 - round / 2
                    };
                    if bytes[at] == bytes[at + 1] {
                        continue;
                    }
                    budget -= 1;
                    let mut corrected = bytes.to_vec();
                    corrected.swap(at, at + 1);
                    let corrected = String::from_utf8(corrected).expect("query is ASCII");
                    if let Some(hits) = self.word_index.get(corrected.as_str()) {
                        corrections[i].extend(hits.iter().copied().filter(|id| {
                            word_match(self.item(*id), &[corrected.as_str()], &sep, true).0 > 0
                        }));
                    }
                }
                if budget == 0 {
                    break;
                }
            }
            if corrections.iter().all(|lists| !lists.is_empty()) {
                for hits in &mut corrections {
                    hits.sort_unstable();
                    hits.dedup();
                }
                let mut lists = known_lists.clone();
                lists.extend(corrections.iter().map(Vec::as_slice));
                if let Some(candidates) = Self::intersect_lists(&lists) {
                    return self.rank::<false>(
                        candidates.into_iter().map(|id| (id, 0)),
                        &query_words,
                        &sep,
                        limit,
                        0,
                    );
                }
            }
        }

        // Try typo matching for unknown words
        if !unknown_words.is_empty() && trigram_budget > 0 {
            let min_len = query.len().saturating_sub(3);
            let (scores, hit_count) =
                self.score_trigrams(&unknown_words, trigram_budget, pool.as_deref(), min_len);
            let min_score = hit_count.div_ceil(2).max(MIN_SCORE);
            // Shared trigrams only nominate: each unknown word must be a typo of
            // one of the item's words, not letters scattered across several.
            let mut typos: Vec<Typo> = unmatched.iter().map(|word| Typo::new(word)).collect();
            let results = self.rank::<false>(
                scores.into_iter().filter(|&(id, s)| {
                    s >= min_score && typos.iter_mut().all(|typo| typo.of(self.item(id), &sep))
                }),
                &query_words,
                &sep,
                limit,
                0,
            );

            if !results.is_empty() {
                return results;
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
        self.rank::<false>(
            candidates.into_iter().map(|id| (id, 0)),
            &query_words,
            &sep,
            limit,
            0,
        )
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
    fn rank<const EXACT_WORDS: bool>(
        &self,
        candidates: impl IntoIterator<Item = (ItemId, usize)>,
        query_words: &[&str],
        sep: &[bool; 256],
        limit: usize,
        min_matched: usize,
    ) -> Vec<(u32, u32)> {
        let candidates = candidates.into_iter().filter_map(|(item, fuzzy)| {
            let (matched, position) = word_match(self.item(item), query_words, sep, EXACT_WORDS);
            (matched >= min_matched).then_some((item, fuzzy, matched, position))
        });
        let mut buckets: Vec<Vec<RankedItem>> = vec![vec![]; query_words.len() + 1];

        for (item, fuzzy, matched, position) in candidates {
            buckets[matched].push((item, fuzzy, position, self.item_rank[item.0 as usize]));
        }

        let mut results = Vec::with_capacity(limit);
        for (matched, bucket) in buckets.iter_mut().enumerate().rev() {
            if bucket.is_empty() {
                continue;
            }
            let take = (limit - results.len()).min(bucket.len());
            select_and_sort(bucket, take);
            results.extend(
                bucket[..take]
                    .iter()
                    .map(|&(id, ..)| (id.0, matched as u32)),
            );
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
        let mut visited: FxHashSet<[u8; 3]> = FxHashSet::default();

        'outer: for round in 0..trigram_budget {
            for word in unknown_words {
                if budget == 0 {
                    break 'outer;
                }

                let bytes = word.as_bytes();
                let Some(pos) = trigram_position(bytes.len(), round) else {
                    continue;
                };
                let trigram = [bytes[pos], bytes[pos + 1], bytes[pos + 2]];

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
    /// Own pre-formatted lowercase ASCII items. Panics on invalid corpus input,
    /// as documented by [`Self::new`].
    pub fn new_owned(items: Vec<String>) -> Self {
        Self::build(items.into_iter().map(Cow::Owned).collect())
    }
}

fn item_order(a: &str, b: &str) -> Ordering {
    a.len().cmp(&b.len()).then_with(|| a.cmp(b))
}

/// Builds a byte lookup table from the separator chars. Separators
/// are ASCII, so a byte-indexed table is exact even for multi-byte UTF-8:
/// continuation and lead bytes are all >= 128 and never flagged.
fn sep_table(separators: &[char]) -> [bool; 256] {
    let mut table = [false; 256];
    for &c in separators {
        table[c as usize] = true;
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

/// Matches query words in any order against item words or adjacent joined pairs.
/// Returns the matched count and earliest item position, or the item's word
/// count when nothing matches.
fn word_match(item: &str, query_words: &[&str], sep: &[bool; 256], exact: bool) -> (usize, usize) {
    let mut matched = 0;
    let mut first_position = words(item, sep).count();
    for qw in query_words {
        let mut previous: Option<&str> = None;
        for (position, iw) in words(item, sep).enumerate() {
            let joined = previous.is_some_and(|pw| {
                qw.len() > pw.len()
                    && qw.starts_with(pw)
                    && if exact {
                        iw == &qw[pw.len()..]
                    } else {
                        iw.starts_with(&qw[pw.len()..])
                    }
            });
            if (if exact {
                iw == *qw
            } else {
                iw.starts_with(*qw)
            }) || joined
            {
                matched += 1;
                first_position = first_position.min(if joined { position - 1 } else { position });
                break;
            }
            previous = Some(iw);
        }
    }
    (matched, first_position)
}

/// A query word not in the index, checked against candidates: whether it's a
/// typo of a run of an item's adjacent words (one word, or several joined):
/// within one edit of the whole run (a swap counts as one; two from seven
/// letters), or, from five letters, within one edit of the run's start (a word
/// typed partway). Never letters scattered across words. Runs too short to be
/// close are skipped, longer ones stop growing, and each is compared once per
/// query.
struct Typo<'q, 's> {
    word: &'q [u8],
    most: usize,
    seen: FxHashMap<&'s str, bool>,
    /// Whether a word can start a close run.
    starts: FxHashMap<&'s str, bool>,
    spans: Vec<(usize, usize)>,
    letters: Vec<u8>,
    rows: [Vec<usize>; 3],
}

impl<'q, 's> Typo<'q, 's> {
    fn new(word: &'q str) -> Self {
        Self {
            word: word.as_bytes(),
            most: if word.len() >= 7 { 2 } else { 1 },
            seen: FxHashMap::default(),
            starts: FxHashMap::default(),
            spans: Vec::new(),
            letters: Vec::new(),
            rows: Default::default(),
        }
    }

    fn of(&mut self, item: &'s str, sep: &[bool; 256]) -> bool {
        let bytes = item.as_bytes();
        self.spans.clear();
        let mut at = 0;
        while at < bytes.len() {
            while at < bytes.len() && sep[bytes[at] as usize] {
                at += 1;
            }
            let start = at;
            while at < bytes.len() && !sep[bytes[at] as usize] {
                at += 1;
            }
            if at > start {
                self.spans.push((start, at));
            }
        }
        let n = self.word.len();
        for first in 0..self.spans.len() {
            let (start, end) = self.spans[first];
            let length = end - start;
            if length + self.most >= n && self.remembered(item, first, first, length) {
                return true;
            }
            // (Longer runs from here: only from a word that can start one, out of
            // reach of every start of this one, its alignment cut after it costs
            // no more.)
            if length > n || first + 1 == self.spans.len() {
                continue;
            }
            let word = &item[start..end];
            let starts = match self.starts.get(word) {
                Some(&starts) => starts,
                None => {
                    let starts = edits_within(word.as_bytes(), self.word, self.most, true, &mut self.rows);
                    self.starts.insert(word, starts);
                    starts
                }
            };
            if !starts {
                continue;
            }
            let mut len = length;
            for last in first + 1..self.spans.len() {
                if len > n {
                    break;
                }
                len += self.spans[last].1 - self.spans[last].0;
                if len + self.most >= n && self.remembered(item, first, last, len) {
                    return true;
                }
            }
        }
        false
    }

    /// Whether the run of words `first..=last` is close, each run's answer kept
    /// by its span in the item.
    fn remembered(&mut self, item: &'s str, first: usize, last: usize, len: usize) -> bool {
        let run = &item[self.spans[first].0..self.spans[last].1];
        if let Some(&close) = self.seen.get(run) {
            return close;
        }
        let close = self.close(item.as_bytes(), first, last, len);
        self.seen.insert(run, close);
        close
    }

    /// The run of words `first..=last` (`len` letters in all), cut to this word's
    /// length and `most` more: within reach of it whole, or of its start.
    fn close(&mut self, bytes: &[u8], first: usize, last: usize, len: usize) -> bool {
        let n = self.word.len();
        let cut = len.min(n + self.most);
        self.letters.clear();
        for &(start, end) in &self.spans[first..=last] {
            let take = (cut - self.letters.len()).min(end - start);
            self.letters.extend_from_slice(&bytes[start..start + take]);
            if self.letters.len() == cut {
                break;
            }
        }
        (cut == len && edits_within(self.word, &self.letters, self.most, false, &mut self.rows))
            || (n >= 5 && edits_within(self.word, &self.letters[..cut.min(n + 1)], 1, true, &mut self.rows))
    }
}

/// Whether `a` and `b` are at most `most` edits apart (insertions, deletions,
/// substitutions, adjacent swaps), giving up as soon as they can't be; `start`:
/// `a` against the closest start of `b` instead (the last row's least, which no
/// row lowers). `rows` are reused from one call to the next.
fn edits_within(a: &[u8], b: &[u8], most: usize, start: bool, rows: &mut [Vec<usize>; 3]) -> bool {
    if !start && a.len().abs_diff(b.len()) > most {
        return false;
    }
    let [before, row, next] = rows;
    for cells in [&mut *before, &mut *row, &mut *next] {
        cells.clear();
        cells.resize(b.len() + 1, 0);
    }
    for (j, cell) in row.iter_mut().enumerate() {
        *cell = j;
    }
    for i in 1..=a.len() {
        next[0] = i;
        let mut least = i;
        for j in 1..=b.len() {
            let cost = usize::from(a[i - 1] != b[j - 1]);
            let mut value = (row[j] + 1).min(next[j - 1] + 1).min(row[j - 1] + cost);
            if i > 1 && j > 1 && a[i - 1] == b[j - 2] && a[i - 2] == b[j - 1] {
                value = value.min(before[j - 2] + 1);
            }
            next[j] = value;
            least = least.min(value);
        }
        if least > most {
            return false;
        }
        mem::swap(before, row);
        mem::swap(row, next);
    }
    start || row[b.len()] <= most
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
