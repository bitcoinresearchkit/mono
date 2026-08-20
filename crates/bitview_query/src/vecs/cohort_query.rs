use std::{collections::BTreeMap, sync::LazyLock};

use bitview_cohort::CohortName;

static ALIASES: LazyLock<Vec<Alias>> = LazyLock::new(Alias::all);

struct Alias {
    id: &'static str,
    words: Vec<String>,
}

pub struct ExpandedQuery {
    pub normalized: String,
    pub expanded: String,
    pub semantic: String,
    pub cohorts: String,
}

impl Alias {
    fn all() -> Vec<Self> {
        let mut aliases: BTreeMap<Vec<String>, Option<&'static str>> = BTreeMap::new();

        for name in CohortName::all() {
            for alias in [name.id, name.short, name.long] {
                let words = words(alias);
                if words.is_empty() {
                    continue;
                }
                aliases
                    .entry(words)
                    .and_modify(|id| {
                        if id.is_some_and(|id| id != name.id) {
                            *id = None;
                        }
                    })
                    .or_insert(Some(name.id));
            }
        }

        let mut aliases = aliases
            .into_iter()
            .filter_map(|(words, id)| id.map(|id| Self { id, words }))
            .collect::<Vec<_>>();
        aliases.sort_unstable_by(|a, b| {
            b.words.len().cmp(&a.words.len()).then_with(|| {
                b.words
                    .iter()
                    .map(String::len)
                    .sum::<usize>()
                    .cmp(&a.words.iter().map(String::len).sum::<usize>())
            })
        });
        aliases
    }

    fn matches(&self, query: &[String]) -> bool {
        query.len() >= self.words.len()
            && query.iter().zip(&self.words).all(|(query, alias)| {
                query == alias || (query.len() >= 3 && alias.starts_with(query))
            })
    }
}

pub fn expand(query: &str) -> ExpandedQuery {
    let query = words(query);
    let normalized = query.join(" ");
    let mut expanded = String::with_capacity(normalized.len() + 8);
    let mut semantic = String::with_capacity(normalized.len());
    let mut cohorts = String::new();
    let mut index = 0;

    while index < query.len() {
        let matched = ALIASES.iter().find(|alias| alias.matches(&query[index..]));
        let (word, consumed) = match matched {
            Some(alias) => {
                push_word(&mut cohorts, alias.id);
                (alias.id, alias.words.len())
            }
            None => {
                push_word(&mut semantic, &query[index]);
                (query[index].as_str(), 1)
            }
        };

        push_word(&mut expanded, word);
        index += consumed;
    }

    ExpandedQuery {
        normalized,
        expanded,
        semantic,
        cohorts,
    }
}

pub fn normalize(text: &str) -> String {
    words(text).join(" ")
}

fn words(text: &str) -> Vec<String> {
    let bytes = text.as_bytes();
    let mut words = Vec::new();
    let mut word = String::new();

    for (index, char) in text.char_indices() {
        let is_decimal_point = char == '.'
            && index > 0
            && bytes[index - 1].is_ascii_digit()
            && bytes
                .get(index + 1)
                .is_some_and(|byte| byte.is_ascii_digit());
        if char.is_ascii_alphanumeric()
            || matches!(char, '<' | '>' | '=' | '+' | '%')
            || is_decimal_point
        {
            word.push(char.to_ascii_lowercase());
        } else if !word.is_empty() {
            words.push(std::mem::take(&mut word));
        }
    }

    if !word.is_empty() {
        words.push(word);
    }
    words
}

fn push_word(text: &mut String, word: &str) {
    if !text.is_empty() {
        text.push(' ');
    }
    text.push_str(word);
}

#[cfg(test)]
mod tests {
    use super::{expand, normalize};

    fn canonicalize(query: &str) -> String {
        expand(query).expanded
    }

    #[test]
    fn canonicalizes_long_and_short_cohort_names() {
        assert_eq!(
            canonicalize("Short-Term Holder realized price"),
            "sth realized price"
        );
        assert_eq!(
            canonicalize("1 Year to 18 Months Old realized price"),
            "1y_to_18m_old realized price"
        );
        assert_eq!(canonicalize("STH realized price"), "sth realized price");
        assert_eq!(
            canonicalize("utxos_1y_to_18m_old_realized_price"),
            "utxos 1y_to_18m_old realized price"
        );
    }

    #[test]
    fn leaves_ambiguous_aliases_unchanged() {
        assert_eq!(canonicalize("all supply"), "all supply");
        assert_eq!(canonicalize(">=10% supply"), ">=10% supply");
    }

    #[test]
    fn separates_periods_except_inside_decimal_numbers() {
        assert_eq!(normalize("Realized price."), "realized price");
        assert_eq!(normalize("Price... then 0.1 BTC."), "price then 0.1 btc");
        assert_eq!(
            canonicalize("0.1-1 BTC realized price."),
            "10m_sats_to_1btc realized price"
        );
    }
}
