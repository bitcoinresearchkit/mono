use bitview_collections::{PerResolution, with_resolution_fields};
use bitview_primitives::OHLCCents;
use bitview_traversable::Traversable;
use brk_types::{Cents, Height, Version};
use derive_more::{Deref, DerefMut};
use schemars::JsonSchema;
use serde::Serialize;
use vecdb::{AnyVec, BytesVecValue, Formattable, LazyVec, ReadableCloneableVec, UnaryTransform};

use super::agg::open::Open;
use crate::{IndexSources, LazyAggVec, LazyOhlcCentsVecs};

macro_rules! define_lazy_indexes {
    (
        periods { $($field:ident: $index:ident => $param:ident,)* }
        epochs { $($epoch:ident: $epoch_index:ident => $epoch_param:ident,)* }
    ) => {
        use bitview_primitives::{$($index,)* $($epoch_index,)*};

        #[derive(Clone, Deref, DerefMut, Traversable)]
        #[traversable(transparent)]
        pub struct LazyIndexes<T, S>(
            pub PerResolution<
                $(LazyVec<$index, T, $index, S>,)*
                $(LazyVec<$epoch_index, T, $epoch_index, S>,)*
            >,
        )
        where
            T: BytesVecValue + Formattable + Serialize + JsonSchema,
            S: BytesVecValue;

        impl LazyIndexes<Cents, Cents> {
            /// Opening prices read only period boundaries, carrying the previous
            /// close for empty periods just like full OHLC candles.
            pub fn from_open_source(
                name: &str,
                version: Version,
                source: &impl ReadableCloneableVec<Height, Cents>,
                indexes: &IndexSources,
            ) -> Self {
                let source = source.read_only_boxed_clone();
                Self(PerResolution {
                    $($field: LazyVec::init(
                        name, version,
                        LazyAggVec::<$index, Cents, Height, Cents, Open>::new(
                            name, version + indexes.first_height.$field.version(),
                            source.clone(), indexes.first_height.$field.mapping().clone(),
                        ).read_only_boxed_clone(),
                        |_, price| price,
                    ),)*
                    $($epoch: LazyVec::init(
                        name, version,
                        LazyAggVec::<$epoch_index, Cents, Height, Cents, Open>::new(
                            name, version + indexes.first_height.$epoch.version(),
                            source.clone(), indexes.first_height.$epoch.mapping().clone(),
                        ).read_only_boxed_clone(),
                        |_, price| price,
                    ),)*
                })
            }
        }

        impl<T> LazyIndexes<T, OHLCCents>
        where
            T: BytesVecValue + Formattable + Serialize + JsonSchema,
        {
            pub fn from_ohlc_indexes<Transform: UnaryTransform<OHLCCents, T>>(
                name: &str,
                version: Version,
                source: &LazyOhlcCentsVecs,
            ) -> Self {
                Self(PerResolution {
                    $($field: LazyVec::transformed::<Transform>(name, version, source.$field.read_only_boxed_clone()),)*
                    $($epoch: LazyVec::transformed::<Transform>(name, version, source.$epoch.read_only_boxed_clone()),)*
                })
            }
        }

        impl<T, S> LazyIndexes<T, S>
        where
            T: BytesVecValue + Formattable + Serialize + JsonSchema,
            S: BytesVecValue,
        {
            pub fn from_lazy_indexes<Transform, U>(
                name: &str,
                version: Version,
                source: &LazyIndexes<S, U>,
            ) -> Self
            where
                Transform: UnaryTransform<S, T>,
                S: BytesVecValue + Formattable + Serialize + JsonSchema,
                U: BytesVecValue,
            {
                Self(PerResolution {
                    $($field: LazyVec::transformed::<Transform>(
                        name, version, source.$field.read_only_boxed_clone(),
                    ),)*
                    $($epoch: LazyVec::transformed::<Transform>(
                        name, version, source.$epoch.read_only_boxed_clone(),
                    ),)*
                })
            }
        }
    };
}

with_resolution_fields!(define_lazy_indexes);
