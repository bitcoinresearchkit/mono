use bitview_distribution::state::cost_basis::{HeldExtremes, age_index::AgeIndexLive};
use brk_types::Version;

/// One compact derived index; canonical origins remain owned by History.
pub(crate) type LiveState = AgeIndexLive<Version, HeldExtremes>;
