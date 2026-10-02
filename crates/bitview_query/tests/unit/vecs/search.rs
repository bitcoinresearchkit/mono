use std::{collections::BTreeMap, sync::Arc};

use super::*;

struct Fixture<'a> {
    names: Vec<&'a str>,
    matcher: QuickMatch<'a>,
    descriptions: DescriptionSearch,
}

impl<'a> Fixture<'a> {
    fn new(items: &[(&'a str, &str)]) -> Self {
        let names: Vec<_> = items.iter().map(|(name, _)| *name).collect();
        let descriptions = items
            .iter()
            .map(|(name, description)| (*name, Arc::from(*description)))
            .collect::<BTreeMap<_, _>>();
        Self {
            matcher: QuickMatch::new(&names),
            descriptions: DescriptionSearch::new(&names, &descriptions),
            names,
        }
    }

    fn search(&self, query: &str, limit: usize) -> Vec<&'a str> {
        matches(&self.names, &self.matcher, &self.descriptions, query, limit)
    }
}

#[test]
fn exact_names_then_exact_descriptions_then_fuzzy_names_then_fuzzy_descriptions() {
    let fixture = Fixture::new(&[
        ("epsilon", "alpha betamax"),
        ("alpha_betamax", "unrelated"),
        ("delta", "alpha beta"),
        ("alpha_beta", "unrelated"),
    ]);
    let expected = ["alpha_beta", "delta", "alpha_betamax", "epsilon"];
    for query in ["alpha beta", "beta alpha", "ALPHA_BETA"] {
        for limit in 0..=6 {
            assert_eq!(
                fixture.search(query, limit),
                expected[..limit.min(expected.len())]
            );
        }
    }
}

#[test]
fn limited_results_preserve_ranking_with_duplicates_across_tiers() {
    let names = (0..100)
        .map(|id| format!("metric_{id}"))
        .collect::<Vec<_>>();
    let mut items = names
        .iter()
        .map(|name| (name.as_str(), "alpha beta"))
        .collect::<Vec<_>>();
    items.push(("alpha_beta", "alpha beta"));
    items.push(("alpha_betamax", "alpha beta"));
    let fixture = Fixture::new(&items);
    let all = fixture.search("alpha beta", items.len());
    assert_eq!(all.len(), items.len());
    for limit in [0, 1, 2, 3, 10, 50, 101, 102, 103] {
        assert_eq!(
            fixture.search("alpha beta", limit),
            all[..limit.min(all.len())]
        );
    }
}
