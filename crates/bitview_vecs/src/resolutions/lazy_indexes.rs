use bitview_collections::{PerResolution, with_resolution_fields};
use bitview_primitives::OHLCCents;
use bitview_traversable::Traversable;
use brk_types::Version;
use derive_more::{Deref, DerefMut};
use schemars::JsonSchema;
use serde::Serialize;
use vecdb::{BytesVecValue, Formattable, LazyVec, ReadableCloneableVec, UnaryTransform};

use crate::LazyOhlcCentsVecs;

macro_rules! define_lazy_indexes {
    (
        periods { $($field:ident: $index:ident => $param:ident,)* }
        epochs { $($epoch:ident: $epoch_index:ident => $epoch_param:ident,)* }
    ) => {
        use bitview_primitives::{$($index,)* $($epoch_index,)*};

        #[derive(Clone, Deref, DerefMut, Traversable)]
        pub struct LazyIndexes<T, S>(
            pub PerResolution<
                $(LazyVec<$index, T, $index, S>,)*
                $(LazyVec<$epoch_index, T, $epoch_index, S>,)*
            >,
        )
        where
            T: BytesVecValue + Formattable + Serialize + JsonSchema,
            S: BytesVecValue;

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
