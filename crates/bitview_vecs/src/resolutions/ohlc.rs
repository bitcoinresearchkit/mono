use bitview_collections::{PerResolution, with_resolution_fields};
use bitview_traversable::Traversable;
use brk_types::{Cents, Height, Version};
use derive_more::{Deref, DerefMut};
use vecdb::{AnyVec, ReadableBoxedVec};

use crate::{IndexSources, LazyOhlcVec};

const COMPUTE_VERSION: Version = Version::TWO;

macro_rules! define_lazy_ohlc_cents_vecs {
    (
        periods { $($field:ident: $index:ident => $param:ident,)* }
        epochs { $($epoch:ident: $epoch_index:ident => $epoch_param:ident,)* }
    ) => {
        use bitview_primitives::{$($index,)* $($epoch_index,)*};

        #[derive(Clone, Deref, DerefMut, Traversable)]
        #[traversable(merge)]
        pub struct LazyOhlcCentsVecs(
            pub PerResolution<
                $(LazyOhlcVec<$index>,)*
                $(LazyOhlcVec<$epoch_index>,)*
            >,
        );

        impl LazyOhlcCentsVecs {
            pub fn new(
                name: &str,
                version: Version,
                mappings: &IndexSources,
                prices: ReadableBoxedVec<Height, Cents>,
            ) -> Self {
                let v = version + COMPUTE_VERSION;
                Self(PerResolution {
                    $($field: LazyOhlcVec::new(
                        name, v + mappings.first_height.$field.version(), &prices,
                        mappings.first_height.$field.mapping().clone(),
                    ),)*
                    $($epoch: LazyOhlcVec::new(
                        name, v + mappings.first_height.$epoch.version(), &prices,
                        mappings.first_height.$epoch.mapping().clone(),
                    ),)*
                })
            }
        }
    };
}

with_resolution_fields!(define_lazy_ohlc_cents_vecs);
