use bitview_primitives::{Count, Index, with_resolution_fields};
use brk_types::Height;
use vecdb::ReadableVec;

use crate::{LazyPreviousDeltaVec, RangeMapVec, shapes::PerResolution};

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

        impl IndexSources {
            /// The first block of period `row` of a date-based, halving or epoch `index` (the
            /// next period's when it holds none); `None` past the last period.
            pub fn first_height_at(&self, index: Index, row: usize) -> Option<Height> {
                match index {
                    $(Index::$index => self.first_height.$field.collect_one_at(row),)*
                    $(Index::$epoch_index => self.first_height.$epoch.collect_one_at(row),)*
                    _ => unreachable!("{index:?} has no periods"),
                }
            }
        }
    };
}

with_resolution_fields!(define_index_sources);
