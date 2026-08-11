use crate::distribution::metrics::{ActivitySources, RealizedSources};

pub struct AdjustedSoprComputeSource {
    pub activity: ActivitySources,
    pub realized: RealizedSources,
}
