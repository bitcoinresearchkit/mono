use brk_error::truncate_series_name;

#[test]
fn truncation_respects_utf8_boundaries_and_the_byte_limit() {
    for prefix in ["", "a", "ab", "abc", "abcd"] {
        for unit in ["a", "é", "€", "🦀"] {
            for count in 0..110 {
                let original = format!("{prefix}{}", unit.repeat(count));
                let result = truncate_series_name(original.clone());
                if original.len() <= 100 {
                    assert_eq!(result, original);
                } else {
                    let shortened = result.strip_suffix("...").unwrap();
                    assert!(original.starts_with(shortened));
                    assert!(shortened.len() <= 100);
                    let next = original[shortened.len()..].chars().next().unwrap();
                    assert!(shortened.len() + next.len_utf8() > 100);
                }
            }
        }
    }
}
