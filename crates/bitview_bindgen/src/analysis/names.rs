//! Common prefix/suffix detection for series names.
//!
//! This module provides utilities to find common prefixes and suffixes
//! among series names, which is used to detect pattern mode (suffix vs prefix).

/// Find the longest common prefix among all strings.
/// Returns the prefix WITH trailing underscore if found at word boundary.
/// Returns None if no common prefix exists.
pub fn find_common_prefix(names: &[&str]) -> Option<String> {
    let first = *names.first()?;
    let mut prefix_len = first.len();
    // Track UTF-8 byte lengths while comparing each name only once.
    for name in &names[1..] {
        prefix_len = first[..prefix_len]
            .chars()
            .zip(name.chars())
            .take_while(|(left, right)| left == right)
            .map(|(ch, _)| ch.len_utf8())
            .sum();
        if prefix_len == 0 {
            break;
        }
    }

    if prefix_len == 0 {
        return None;
    }

    let raw_prefix = &first[..prefix_len];

    // Must end at underscore boundary for semantic coherence
    if raw_prefix.ends_with('_') {
        return Some(raw_prefix.to_string());
    }

    // If raw_prefix equals one of the full names (one name is a prefix of all others),
    // return it with trailing underscore for proper base detection
    if names.contains(&raw_prefix) {
        return Some(format!("{}_", raw_prefix));
    }

    // Find the last underscore position
    raw_prefix.rfind('_').map(|end| first[..=end].to_string())
}

/// Find the longest common suffix among all strings.
/// Returns the suffix WITH leading underscore if found at word boundary.
/// Returns None if no common suffix exists.
pub fn find_common_suffix(names: &[&str]) -> Option<String> {
    let first = *names.first()?;
    let mut suffix_len = first.len();
    for name in &names[1..] {
        suffix_len = first[first.len() - suffix_len..]
            .chars()
            .rev()
            .zip(name.chars().rev())
            .take_while(|(left, right)| left == right)
            .map(|(ch, _)| ch.len_utf8())
            .sum();
        if suffix_len == 0 {
            break;
        }
    }

    if suffix_len == 0 {
        return None;
    }

    let raw_suffix = &first[first.len() - suffix_len..];

    // Must start at underscore boundary for semantic coherence
    if raw_suffix.starts_with('_') {
        return Some(raw_suffix.to_string());
    }

    // Check if preceded by underscore in all names (word boundary)
    let at_word_boundary = names.iter().all(|n| {
        if *n == raw_suffix {
            true // Suffix is the whole string
        } else if let Some(prefix) = n.strip_suffix(raw_suffix) {
            prefix.ends_with('_')
        } else {
            false
        }
    });

    if at_word_boundary {
        return Some(format!("_{}", raw_suffix));
    }

    // Find the first underscore position in suffix
    raw_suffix
        .find('_')
        .map(|start| raw_suffix[start..].to_string())
}

/// Normalize a prefix string by ensuring it ends with underscore.
/// Returns empty string if input is empty.
pub(crate) fn normalize_prefix(s: &str) -> String {
    if s.is_empty() {
        String::new()
    } else if s.ends_with('_') {
        s.to_string()
    } else {
        format!("{}_", s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_common_prefix_basic() {
        let names = vec!["addrs_0sats", "addrs_1sats", "addrs_2sats"];
        assert_eq!(find_common_prefix(&names), Some("addrs_".to_string()));
    }

    #[test]
    fn test_common_prefix_none() {
        let names = vec!["foo", "bar", "baz"];
        assert_eq!(find_common_prefix(&names), None);
    }

    #[test]
    fn test_common_prefix_lth() {
        let names = vec!["lth_cost_basis_max", "lth_cost_basis_min", "lth_cost_basis"];
        assert_eq!(
            find_common_prefix(&names),
            Some("lth_cost_basis_".to_string())
        );
    }

    #[test]
    fn test_common_suffix_basic() {
        let names = vec!["cumulative_supply", "net_supply", "total_supply"];
        assert_eq!(find_common_suffix(&names), Some("_supply".to_string()));
    }

    #[test]
    fn test_common_prefix_cost_basis() {
        // With suffix naming convention, cost_basis variants share a common prefix
        let names = vec!["cost_basis_max", "cost_basis_min", "cost_basis"];
        assert_eq!(find_common_prefix(&names), Some("cost_basis_".to_string()));
    }

    #[test]
    fn test_common_suffix_none() {
        let names = vec!["foo", "bar", "baz"];
        assert_eq!(find_common_suffix(&names), None);
    }

    #[test]
    fn test_common_prefix_one_is_prefix_of_other() {
        // When one name is a prefix of another (block_count vs block_count_cumulative)
        let names = vec!["block_count_cumulative", "block_count"];
        assert_eq!(find_common_prefix(&names), Some("block_count_".to_string()));
    }

    #[test]
    fn test_common_suffix_realized_loss() {
        let names = vec![
            "cumulative_realized_loss",
            "net_realized_loss",
            "realized_loss",
        ];
        assert_eq!(
            find_common_suffix(&names),
            Some("_realized_loss".to_string())
        );
    }
}
