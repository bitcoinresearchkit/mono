use crate::distribution::metrics::{ActivitySources, RealizedSources};

pub(crate) struct RealizedAggregateSources {
    pub activity: ActivitySources,
    pub realized: RealizedSources,
}
