use bitview_primitives::Count;
use brk_types::Height;

use crate::{LazyPreviousDeltaVec, RangeMapVec, shapes::PerResolution, with_resolution_fields};

macro_rules! define_index_sources {
    (
        periods { $($field:ident: $index:ident => $param:ident,)* }
        epochs { $($epoch:ident: $epoch_index:ident => $epoch_param:ident,)* }
    ) => {
        use bitview_primitives::{$($index,)* $($epoch_index,)*};

        #[derive(Clone)]
        pub struct IndexSources {
            pub first_height: PerResolution<
                $(RangeMapVec<$index, Height>,)*
                $(RangeMapVec<$epoch_index, Height>,)*
            >,
            pub height_tx_index_count: LazyPreviousDeltaVec<Height, Count>,
        }
    };
}

with_resolution_fields!(define_index_sources);
