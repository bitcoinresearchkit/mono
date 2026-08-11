use crate::distribution::metrics::{ActivitySources, RealizedSources};

pub struct RealizedAggregateSources {
    pub activity: ActivitySources,
    pub realized: RealizedSources,
}
