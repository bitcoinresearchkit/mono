//! Common prefix/suffix detection for series names.
//!
//! This module provides utilities to find common prefixes and suffixes
//! among series names, which is used to detect pattern mode (suffix vs prefix).

/// Find the longest common prefix among all strings.
/// Returns the prefix WITH trailing underscore if found at word boundary.
/// Returns None if no common prefix exists.
pub fn find_common_prefix(names: &[&str]) -> Option<String> {
    if names.is_empty() || names.iter().any(|n| n.is_empty()) {
        return None;
    }

    let first = names[0];

    // Find character-by-character common prefix
    let mut prefix_len = 0;
    for (i, ch) in first.chars().enumerate() {
        if names.iter().all(|n| n.chars().nth(i) == Some(ch)) {
            prefix_len = i + 1;
        } else {
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
    if let Some(last_underscore) = raw_prefix.rfind('_') {
        let clean_prefix = &first[..=last_underscore];
        if names.iter().all(|n| n.starts_with(clean_prefix)) {
            return Some(clean_prefix.to_string());
        }
    }

    None
}

/// Find the longest common suffix among all strings.
/// Returns the suffix WITH leading underscore if found at word boundary.
/// Returns None if no common suffix exists.
pub fn find_common_suffix(names: &[&str]) -> Option<String> {
    if names.is_empty() || names.iter().any(|n| n.is_empty()) {
        return None;
    }

    let first = names[0];
    let first_chars: Vec<char> = first.chars().collect();

    // Find character-by-character common suffix (from the end)
    let mut suffix_len = 0;
    for i in 0..first_chars.len() {
        let idx_from_end = first_chars.len() - 1 - i;
        let ch = first_chars[idx_from_end];

        let all_match = names.iter().all(|n| {
            let n_chars: Vec<char> = n.chars().collect();
            if i >= n_chars.len() {
                return false;
            }
            n_chars[n_chars.len() - 1 - i] == ch
        });

        if all_match {
            suffix_len = i + 1;
        } else {
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
    if let Some(first_underscore) = raw_suffix.find('_') {
        let clean_suffix = &raw_suffix[first_underscore..];
        if names.iter().all(|n| n.ends_with(clean_suffix)) {
            return Some(clean_suffix.to_string());
        }
    }

    None
}

/// Normalize a prefix string by ensuring it ends with underscore.
/// Returns empty string if input is empty.
pub fn normalize_prefix(s: &str) -> String {
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
