use quickmatch::{QuickMatch, QuickMatchConfig};
use rustc_hash::{FxHashMap, FxHashSet};

use super::{SeriesRegistry, normalize::normalize};

impl SeriesRegistry<'_> {
    pub(crate) fn matches(&self, query: &str, limit: usize) -> Vec<&'_ str> {
        matches(
            &self.series_names,
            &self.matcher,
            &self.sentences,
            query,
            limit,
        )
    }
}

fn matches<'a>(
    names: &[&'a str],
    matcher: &QuickMatch<'_>,
    sentences: &SentenceIndex,
    query: &str,
    limit: usize,
) -> Vec<&'a str> {
    let limit = limit.min(names.len());
    if limit == 0 {
        return Vec::new();
    }
    let query = normalize(query);
    let config = QuickMatchConfig::new()
        .with_limit(limit)
        .with_union_fallback(false);
    let mut result = Vec::with_capacity(limit);
    let mut seen = FxHashSet::default();

    // Four tiers: a name's whole words; a name every word of which matches
    // (its starts, joined words, a plural); a description's whole words,
    // singular or plural; a name some word of which only a typo matches. What
    // a series' name says comes first, unless it takes a typo to. Descriptions
    // take no other start nor a typo: those answered with series nobody asked
    // for.
    append(
        matcher
            .matches_exact_with_ids_and_matched_words(&query, &config)
            .into_iter()
            .map(|(id, _)| id),
        &mut result,
        &mut seen,
        limit,
    );
    if result.len() >= limit {
        return result.into_iter().map(|id| names[id as usize]).collect();
    }
    // Normalization leaves a single space between words.
    let count = query
        .split_ascii_whitespace()
        .collect::<FxHashSet<_>>()
        .len();
    let (whole, typos): (Vec<_>, Vec<_>) = matcher
        .matches_with_ids_and_matched_words(&query, &config)
        .into_iter()
        .partition(|&(_, matched)| matched as usize >= count);
    append(
        whole.into_iter().map(|(id, _)| id),
        &mut result,
        &mut seen,
        limit,
    );
    if result.len() < limit {
        let name_words = matcher.words_held(&query);
        append(
            sentences.ranked(&query, names, &name_words, limit),
            &mut result,
            &mut seen,
            limit,
        );
        append(
            typos.into_iter().map(|(id, _)| id),
            &mut result,
            &mut seen,
            limit,
        );
    }
    result.into_iter().map(|id| names[id as usize]).collect()
}

fn append(
    ids: impl IntoIterator<Item = u32>,
    result: &mut Vec<u32>,
    seen: &mut FxHashSet<u32>,
    limit: usize,
) {
    for id in ids {
        if result.len() == limit {
            break;
        }
        if seen.insert(id) {
            result.push(id);
        }
    }
}

pub(super) struct SentenceIndex {
    matcher: QuickMatch<'static>,
    series: Vec<Box<[u32]>>,
}

impl SentenceIndex {
    pub(super) fn new<'a>(descriptions: impl Iterator<Item = &'a str>) -> Self {
        let mut postings = FxHashMap::<&str, Vec<u32>>::default();
        for (id, text) in descriptions.enumerate() {
            let id = u32::try_from(id).expect("too many series");
            // A decimal point belongs to its number. Only split punctuation before whitespace.
            for paragraph in text.split("\n\n") {
                let mut punctuation = false;
                for sentence in paragraph
                    .split(|c: char| {
                        let boundary = punctuation && c.is_whitespace();
                        punctuation = matches!(c, '.' | '!' | '?');
                        boundary
                    })
                    .map(str::trim)
                    .filter(|s| !s.is_empty())
                {
                    let ids = postings.entry(sentence).or_default();
                    if ids.last() != Some(&id) {
                        ids.push(id);
                    }
                }
            }
        }
        let (sentences, series) = postings
            .into_iter()
            .map(|(text, ids)| (normalize(text), ids.into_boxed_slice()))
            .unzip();
        Self {
            matcher: QuickMatch::new_owned_whole_words(sentences),
            series,
        }
    }

    /// The series whose description holds every query word whole, singular or plural, best first.
    fn ranked(
        &self,
        query: &str,
        names: &[&str],
        name_words: &[u8],
        limit: usize,
    ) -> Vec<u32> {
        let hits = self.matcher.word_hits(query);
        // A series' words are counted in a byte: no hit, or more words than that, answers nothing.
        let words = u8::try_from(hits.len()).unwrap_or(0);
        if words == 0 {
            return Vec::new();
        }
        // Advance a series at most once per word, however many matching sentences it contains.
        let mut held = vec![0u8; names.len()];
        for (word, sentences) in (0..).zip(&hits) {
            for &sentence in sentences {
                for &id in &self.series[sentence as usize] {
                    let count = &mut held[id as usize];
                    if *count == word {
                        *count += 1;
                    }
                }
            }
        }
        let mut candidates = held
            .into_iter()
            .enumerate()
            .filter(|&(_, count)| count == words)
            .map(|(id, _)| {
                let words = if name_words[id] == 0 {
                    1
                } else {
                    names[id].split('_').count()
                };
                (id, usize::from(name_words[id]), words)
            })
            .collect::<Vec<_>>();
        // Registry ids already sort by name length, then spelling.
        let compare = |a: &(usize, usize, usize), b: &(usize, usize, usize)| {
            (b.1 * a.2).cmp(&(a.1 * b.2)).then(a.0.cmp(&b.0))
        };
        if candidates.len() > limit {
            candidates.select_nth_unstable_by(limit, compare);
            candidates.truncate(limit);
        }
        candidates.sort_unstable_by(compare);
        candidates.into_iter().map(|(id, ..)| id as u32).collect()
    }
}
