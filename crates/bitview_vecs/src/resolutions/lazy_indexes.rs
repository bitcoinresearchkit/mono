use bitview_traversable::Traversable;
use brk_types::Version;
use derive_more::{Deref, DerefMut};
use schemars::JsonSchema;
use serde::Serialize;
use vecdb::{BytesVecValue, Formattable, LazyVec, ReadableCloneableVec, UnaryTransform};

use bitview_primitives::with_resolution_fields;

use crate::shapes::PerResolution;

macro_rules! define_lazy_indexes {
    (
        periods { $($field:ident: $index:ident => $param:ident,)* }
        epochs { $($epoch:ident: $epoch_index:ident => $epoch_param:ident,)* }
    ) => {
        use bitview_primitives::{$($index,)* $($epoch_index,)*};

        #[derive(Clone, Deref, DerefMut, Traversable)]
        pub struct LazyIndexes<T, S>(
            pub PerResolution<
                $(LazyVec<$index, T, S>,)*
                $(LazyVec<$epoch_index, T, S>,)*
            >,
        )
        where
            T: BytesVecValue + Formattable + Serialize + JsonSchema,
            S: BytesVecValue;

        impl<T, S> LazyIndexes<T, S>
        where
            T: BytesVecValue + Formattable + Serialize + JsonSchema,
            S: BytesVecValue,
        {
            /// Each resolution of `source` through `Transform`.
            pub fn from_resolutions<Transform: UnaryTransform<S, T>>(
                name: &str,
                version: Version,
                source: &PerResolution<
                    $(impl ReadableCloneableVec<$index, S>,)*
                    $(impl ReadableCloneableVec<$epoch_index, S>,)*
                >,
            ) -> Self {
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
