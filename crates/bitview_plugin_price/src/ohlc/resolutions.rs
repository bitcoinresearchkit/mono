use bitview_primitives::OHLCCents;
use bitview_traversable::Traversable;
use bitview_vecs::{IndexSources, LazyIndexes, shapes::PerResolution, with_resolution_fields};
use brk_types::{Cents, Height, Version};
use schemars::JsonSchema;
use serde::Serialize;
use vecdb::{
    AnyVec, BytesVecValue, Formattable, LazyVec, ReadableBoxedVec, ReadableCloneableVec,
    UnaryTransform,
};

use super::candle::LazyOhlcVec;

const COMPUTE_VERSION: Version = Version::TWO;

macro_rules! define_lazy_ohlc_cents_vecs {
    (
        periods { $($field:ident: $index:ident => $param:ident,)* }
        epochs { $($epoch:ident: $epoch_index:ident => $epoch_param:ident,)* }
    ) => {
        use bitview_primitives::{$($index,)* $($epoch_index,)*};

        #[derive(Clone, Traversable)]
        pub(crate) struct LazyOhlcCentsVecs(
            PerResolution<
                $(LazyOhlcVec<$index>,)*
                $(LazyOhlcVec<$epoch_index>,)*
            >,
        );

        impl LazyOhlcCentsVecs {
            pub(super) fn new(
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

            /// Every resolution's candles through `Transform`.
            pub(crate) fn transformed<T, Transform>(&self, name: &str, version: Version) -> LazyIndexes<T, OHLCCents>
            where
                T: BytesVecValue + Formattable + Serialize + JsonSchema,
                Transform: UnaryTransform<OHLCCents, T>,
            {
                LazyIndexes(PerResolution {
                    $($field: LazyVec::transformed::<Transform>(name, version, self.0.$field.read_only_boxed_clone()),)*
                    $($epoch: LazyVec::transformed::<Transform>(name, version, self.0.$epoch.read_only_boxed_clone()),)*
                })
            }
        }
    };
}

with_resolution_fields!(define_lazy_ohlc_cents_vecs);
