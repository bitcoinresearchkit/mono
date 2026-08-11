use crate::distribution::metrics::{ActivitySources, RealizedSources};

pub(crate) struct AdjustedSoprComputeSource {
    pub activity: ActivitySources,
    pub realized: RealizedSources,
}
