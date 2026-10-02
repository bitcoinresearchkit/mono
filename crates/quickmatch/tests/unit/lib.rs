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
            borrowed.matches_with_ids_and_matched_words(query, &config),
            owned.matches_with_ids_and_matched_words(query, &config),
            "owned matcher changed results for {query}"
        );

        let indexed = borrowed.matches_with_ids_and_matched_words(query, &config);
        let resolved = indexed
            .iter()
            .map(|&(id, _)| ITEMS[id as usize])
            .collect::<Vec<_>>();
        assert_eq!(
            resolved,
            borrowed.matches(query),
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

    assert_eq!(
        matcher
            .matches_with_ids_and_matched_words("alpha beta", &union)
            .len(),
        2
    );
    assert!(
        matcher
            .matches_with_ids_and_matched_words("alpha beta", &intersection_only)
            .is_empty()
    );
}

#[test]
fn best_matched_word_tier_matches_full_ranking_then_filtering() {
    let items = [
        "short term holder realized capitalization",
        "long term holder realized capitalization",
        "realized capitalization adjusted by entity",
        "address count with positive balance",
        "supply held by short term holders",
        "supply held by long term holders",
        "realized profit and realized loss",
        "market capitalization divided by realized capitalization",
        "coin days destroyed",
        "bitcoin closing price",
    ];
    let matcher = QuickMatch::new(&items);

    for config in [
        QuickMatchConfig::new().with_limit(items.len()),
        QuickMatchConfig::new().with_limit(2),
        QuickMatchConfig::new()
            .with_limit(items.len())
            .with_trigram_budget(0),
        QuickMatchConfig::new()
            .with_limit(items.len())
            .with_union_fallback(false),
    ] {
        for query in [
            "short term holder capitalization",
            "long term price",
            "address supply",
            "market cap",
            "coin days",
            "realized proft loss",
            "bitcoin price",
            "capitalization holder",
            "missing",
            "",
        ] {
            let full = matcher.matches_with_ids_and_matched_words(query, &config);
            let expected = full
                .first()
                .map(|(_, best)| {
                    full.iter()
                        .copied()
                        .take_while(|(_, matched)| matched == best)
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();

            assert_eq!(
                matcher.matches_best_with_ids_and_matched_words(query, &config),
                expected,
                "best-only ranking changed results for {query:?}"
            );
        }
    }
}

#[test]
fn complete_metric_words_rank_first_in_any_order() {
    let items = [
        "sth_realized_price_pct1",
        "realized_price_sth_ratio",
        "price_price_sth",
        "sth_realized_price",
    ];
    let matcher = QuickMatch::new(&items);
    let config = QuickMatchConfig::new().with_limit(1);
    for query in [
        "sth realized price",
        "sth price realized",
        "realized sth price",
        "realized price sth",
        "price sth realized",
        "price realized sth",
    ] {
        assert_eq!(
            matcher.matches_with_ids_and_matched_words(query, &config),
            vec![(3, 3)],
            "{query}"
        );
        assert_eq!(
            matcher.matches_best_with_ids_and_matched_words(query, &config),
            vec![(3, 3)],
            "{query}"
        );
    }
    let repeated = QuickMatch::new(&["price_price_sth"]);
    assert_eq!(
        repeated.matches_with_ids_and_matched_words("realized price sth", &config),
        vec![(0, 2)]
    );
}

#[test]
fn unordered_terms_joined_words_and_swapped_letters_rank_the_base_metric_first() {
    let items = [
        "utxos_in_profit_sth_supply",
        "sth_supply_in_profit",
        "difficulty_hashrate",
        "hash_rate",
        "hash_rate_ath",
        "sth_realized_cap",
        "sth_realized_price_pct1",
        "sth_realized_price",
        "op_return_hash_fee",
    ];
    let matcher = QuickMatch::new(&items);
    for (query, expected) in [
        ("profit supply sth", "sth_supply_in_profit"),
        ("sth profit supply", "sth_supply_in_profit"),
        ("hashrate", "hash_rate"),
        ("hash rate", "hash_rate"),
        ("hashraet", "hash_rate"),
        ("realized prcie sth", "sth_realized_price"),
        ("hash rte", "hash_rate"),
    ] {
        assert_eq!(matcher.matches(query)[0], expected, "{query}");
    }
    let results = matcher.matches_with_ids_and_matched_words(
        "realized prcie sth",
        &QuickMatchConfig::new().with_trigram_budget(0),
    );
    assert_eq!(items[results[0].0 as usize], "sth_realized_cap");
}
