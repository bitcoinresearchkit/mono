use bitview_collections::{PerResolution, with_resolution_fields};
use bitview_compute::ComputedVecValue;
use bitview_traversable::Traversable;
use brk_types::Version;
use derive_more::{Deref, DerefMut};
use schemars::JsonSchema;
use vecdb::{LazyVec, MapOption, ReadableBoxedVec, ReadableCloneableVec, UnaryTransform, VecValue};

use crate::Resolutions;

macro_rules! define_derived_resolutions {
    (
        periods { $($field:ident: $index:ident => $param:ident,)* }
        epochs { $($epoch:ident: $epoch_index:ident => $epoch_param:ident,)* }
    ) => {
        use bitview_primitives::{$($index,)* $($epoch_index,)*};

        #[derive(Clone, Deref, DerefMut, Traversable)]
        #[traversable(transparent)]
        pub struct DerivedResolutions<T, S1T = T>(
            pub PerResolution<
                $(LazyVec<$index, Option<T>, $index, Option<S1T>>,)*
                $(LazyVec<$epoch_index, T, $epoch_index, S1T>,)*
            >,
        )
        where
            T: VecValue + PartialOrd + JsonSchema,
            S1T: VecValue;

        impl<T, S1T> DerivedResolutions<T, S1T>
        where
            T: VecValue + PartialOrd + JsonSchema + 'static,
            S1T: VecValue + PartialOrd + JsonSchema,
        {
            pub fn from_derived_computed<F: UnaryTransform<S1T, T>>(
                name: &str,
                version: Version,
                source: &Resolutions<S1T>,
            ) -> Self {
                Self::from_sources::<F>(
                    name, version,
                    PerResolution {
                        $($field: source.$field.read_only_boxed_clone(),)*
                        $($epoch: source.$epoch.read_only_boxed_clone(),)*
                    },
                )
            }

            pub fn from_lazy<F, S2T>(
                name: &str,
                version: Version,
                source: &DerivedResolutions<S1T, S2T>,
            ) -> Self
            where
                F: UnaryTransform<S1T, T>,
                S2T: ComputedVecValue + JsonSchema,
            {
                Self::from_sources::<F>(
                    name, version,
                    PerResolution {
                        $($field: source.$field.read_only_boxed_clone(),)*
                        $($epoch: source.$epoch.read_only_boxed_clone(),)*
                    },
                )
            }

            fn from_sources<F: UnaryTransform<S1T, T>>(
                name: &str,
                version: Version,
                source: PerResolution<
                    $(ReadableBoxedVec<$index, Option<S1T>>,)*
                    $(ReadableBoxedVec<$epoch_index, S1T>,)*
                >,
            ) -> Self {
                Self(PerResolution {
                    $($field: LazyVec::transformed::<MapOption<F>>(
                        name, version, source.$field,
                    ),)*
                    $($epoch: LazyVec::transformed::<F>(
                        name, version, source.$epoch,
                    ),)*
                })
            }
        }
    };
}

with_resolution_fields!(define_derived_resolutions);
