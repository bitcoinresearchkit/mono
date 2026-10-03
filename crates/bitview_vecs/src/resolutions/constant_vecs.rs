use bitview_collections::with_resolution_fields;
use bitview_traversable::Traversable;
use brk_types::{Height, Version};
use schemars::JsonSchema;
use serde::Serialize;
use vecdb::{Formattable, IndexVec, ReadableBoxedVec, UnaryTransform, VecValue};

use crate::{IndexSources, RangeMapVec};

macro_rules! define_constant_vecs {
    (
        periods { $($field:ident: $index:ident => $param:ident,)* }
        epochs { $($epoch:ident: $epoch_index:ident => $epoch_param:ident,)* }
    ) => {
        use bitview_primitives::{$($index,)* $($epoch_index,)*};

        #[derive(Clone, Traversable)]
        #[traversable(merge)]
        pub struct ConstantVecs<T>
        where
            T: VecValue + Formattable + Serialize + JsonSchema,
        {
            pub height: IndexVec<Height, T, ReadableBoxedVec<Height, Minute10>>,
            $(pub $field: IndexVec<$index, T, RangeMapVec<$index, Height>>,)*
            $(pub $epoch: IndexVec<$epoch_index, T, RangeMapVec<$epoch_index, Height>>,)*
        }

        impl<T: VecValue + Formattable + Serialize + JsonSchema> ConstantVecs<T> {
            pub fn new<F>(name: &str, version: Version, indexes: &IndexSources) -> Self
            where
                F: UnaryTransform<Height, T>
                    $(+ UnaryTransform<$index, T>)*
                    $(+ UnaryTransform<$epoch_index, T>)*,
            {
                Self {
                    height: IndexVec::new(name, version, indexes.height_minute10.clone(), F::apply),
                    $($field: IndexVec::new(
                        name, version, indexes.first_height.$field.clone(),
                        F::apply,
                    ),)*
                    $($epoch: IndexVec::new(
                        name, version, indexes.first_height.$epoch.clone(),
                        F::apply,
                    ),)*
                }
            }
        }
    };
}

with_resolution_fields!(define_constant_vecs);
