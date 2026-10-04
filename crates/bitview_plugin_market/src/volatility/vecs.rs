use bitview_collections::Windows;
use bitview_primitives::Ratio;
use bitview_traversable::Traversable;
use bitview_vecs::LazyPerBlock;
use derive_more::Deref;

#[derive(Clone, Deref, Traversable)]
pub struct Vecs(
    #[deref]
    #[traversable(flatten)]
    pub(super) Windows<LazyPerBlock<Ratio>>,
);
