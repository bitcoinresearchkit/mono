use bitview_plugin_distribution_common::state::cost_basis::age_index::AgeIndexLive;
use brk_types::Version;

/// One compact derived index; canonical origins remain owned by History.
pub(crate) type LiveState = AgeIndexLive<(Version, Version, (u64, u64), Version)>;
