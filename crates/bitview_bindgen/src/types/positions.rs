//! Pattern mode and field parts for series name reconstruction.
//!
//! Patterns are either suffix mode or prefix mode:
//! - Suffix mode: `_m(acc, relative)` → `acc_relative` or just `relative` if acc empty
//! - Prefix mode: `_p(prefix, acc)` → `prefix_acc` or just `acc` if prefix empty

use std::collections::BTreeMap;

/// How a pattern constructs series names from the accumulator.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PatternMode {
    /// Fields append their relative name to acc.
    /// Formula: `_m(acc, relative)` → `{acc}_{relative}` or `{relative}` if acc empty
    /// Example: `_m("lth", "max_cost_basis")` → `"lth_max_cost_basis"`
    Suffix {
        /// Maps field name to its relative name (full series name when acc = "")
        relatives: BTreeMap<String, String>,
    },
    /// Fields prepend their prefix to acc.
    /// Formula: `_p(prefix, acc)` → `{prefix}_{acc}` or `{acc}` if prefix empty
    /// Example: `_p("cumulative", "lth_realized_loss")` → `"cumulative_lth_realized_loss"`
    Prefix {
        /// Maps field name to its prefix (empty string for identity)
        prefixes: BTreeMap<String, String>,
    },
    /// Fields construct series names using a template with a discriminator placeholder.
    /// Factory takes two params: `acc` (base) and `disc` (discriminator).
    /// Formula: `_m(acc, template.replace("{disc}", disc))`
    /// Example: template `"ratio_{disc}_ppm"` with disc `"pct99"` → `_m(acc, "ratio_pct99_ppm")`
    Templated {
        /// Maps field name to its template string containing `{disc}` placeholder
        templates: BTreeMap<String, String>,
    },
}
