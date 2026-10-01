use bitview_collections::Windows;
use bitview_plugin_mappings::Vecs as Mappings;
use bitview_transforms::NegCentsUnsignedToDollars;
use bitview_traversable::Traversable;
use bitview_vecs::{LazyFiatPerBlockCumulativeWithSums, LazyPerBlock};
use brk_types::{Cents, Dollars, Height, Version};
use vecdb::{LazyVec, ReadableCloneableVec};

#[derive(Clone, Traversable)]
pub struct NegRealizedLoss {
    #[traversable(flatten)]
    /// Negative realized loss for the represented block.
    pub base: LazyVec<Height, Dollars, Height, Cents>,
    /// Sum of negative realized loss over each supported trailing window.
    pub sum: Windows<LazyPerBlock<Dollars, Cents>>,
}

impl NegRealizedLoss {
    pub fn from_source(
        name: &str,
        version: Version,
        loss: &LazyFiatPerBlockCumulativeWithSums<Cents>,
        mappings: &Mappings,
    ) -> Self {
        let base = LazyVec::transformed::<NegCentsUnsignedToDollars>(
            name,
            version,
            loss.block.cents.read_only_boxed_clone(),
        );
        let sum = loss.sum.0.map_with_suffix(|suffix, slot| {
            LazyPerBlock::from_height_source::<NegCentsUnsignedToDollars>(
                &format!("{name}_sum_{suffix}"),
                version,
                &slot.cents.height,
                mappings,
            )
        });
        Self { base, sum }
    }
}
