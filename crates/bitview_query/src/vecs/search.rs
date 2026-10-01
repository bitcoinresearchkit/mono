use bitview_types::{Limit, SeriesName};
use quickmatch::{QuickMatch, QuickMatchConfig};
use rustc_hash::{FxHashMap, FxHashSet};

use super::{DescriptionSearch, SeriesId, Vecs, normalize::normalize};

impl Vecs<'_> {
    pub(crate) fn matches(&self, series: &SeriesName, limit: Limit) -> Vec<&'_ str> {
        matches(
            &self.series_names,
            &self.matcher,
            &self.description_search,
            series,
            *limit,
        )
    }
}

fn matches<'a>(
    names: &[&'a str],
    matcher: &QuickMatch<'_>,
    descriptions: &DescriptionSearch,
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

    // Each field gets its own tier. Description scores never compete with
    // complete name-word matches, and fuzzy matches cannot outrank exact ones.
    append(
        matcher
            .matches_exact_with_ids_and_matched_words(&query, &config)
            .into_iter()
            .map(|(id, _)| SeriesId(id)),
        &mut result,
        &mut seen,
        limit,
    );
    if result.len() < limit {
        let name_words: FxHashMap<_, _> = matcher
            .matches_exact_with_ids_and_matched_words(
                &query,
                &QuickMatchConfig::new().with_limit(names.len()),
            )
            .into_iter()
            .map(|(id, count)| (SeriesId(id), count))
            .collect();
        let description_config = QuickMatchConfig::new()
            .with_limit(descriptions.series_by_description.len())
            .with_union_fallback(false);
        append(
            rank_descriptions(
                descriptions
                    .matcher
                    .matches_exact_with_ids_and_matched_words(&query, &description_config),
                descriptions,
                names,
                &name_words,
                limit,
            ),
            &mut result,
            &mut seen,
            limit,
        );
        if result.len() < limit {
            append(
                matcher
                    .matches_with_ids_and_matched_words(&query, &config)
                    .into_iter()
                    .map(|(id, _)| SeriesId(id)),
                &mut result,
                &mut seen,
                limit,
            );
        }
        if result.len() < limit {
            append(
                rank_descriptions(
                    descriptions
                        .matcher
                        .matches_with_ids_and_matched_words(&query, &description_config),
                    descriptions,
                    names,
                    &name_words,
                    limit,
                ),
                &mut result,
                &mut seen,
                limit,
            );
        }
    }
    result.into_iter().map(|id| names[id.as_usize()]).collect()
}

fn append(
    ids: impl IntoIterator<Item = SeriesId>,
    result: &mut Vec<SeriesId>,
    seen: &mut FxHashSet<SeriesId>,
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

fn rank_descriptions(
    matches: Vec<(u32, u32)>,
    descriptions: &DescriptionSearch,
    names: &[&str],
    name_words: &FxHashMap<SeriesId, u32>,
    limit: usize,
) -> Vec<SeriesId> {
    // Among matching descriptions, prefer names with fewer unrelated words.
    // This uses only the canonical ID, without cohort aliases or metric rules.
    let mut candidates = matches
        .into_iter()
        .enumerate()
        .flat_map(|(rank, (description, _))| {
            descriptions.series_by_description[description as usize]
                .iter()
                .map(move |&id| {
                    let name = names[id.as_usize()];
                    (
                        id,
                        rank,
                        name_words.get(&id).copied().unwrap_or(0) as usize,
                        name.split('_').count(),
                    )
                })
        })
        .collect::<Vec<_>>();
    let compare = |a: &(SeriesId, usize, usize, usize), b: &(SeriesId, usize, usize, usize)| {
        let an = names[a.0.as_usize()];
        let bn = names[b.0.as_usize()];
        (b.2 * a.3)
            .cmp(&(a.2 * b.3))
            .then(an.len().cmp(&bn.len()))
            .then(a.1.cmp(&b.1))
            .then(an.cmp(bn))
    };
    // Each tier has unique IDs, so the first `limit` candidates suffice
    // even when some were already returned by earlier tiers.
    if candidates.len() > limit {
        candidates.select_nth_unstable_by(limit, compare);
        candidates.truncate(limit);
    }
    candidates.sort_unstable_by(compare);
    candidates.into_iter().map(|(id, ..)| id).collect()
}

#[cfg(test)]
#[path = "../../tests/unit/vecs/search.rs"]
mod tests;
