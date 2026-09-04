use super::TipJsonCache;

#[derive(Clone, Default)]
pub(crate) struct BlockCaches {
    pub(crate) recent: TipJsonCache<()>,
    pub(crate) recent_v1: TipJsonCache<()>,
}
