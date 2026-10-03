use bitview_collections::{PerResolution, with_resolution_fields};
use bitview_primitives::{Date, StoredU64};
use brk_types::{Height, Timestamp};
use vecdb::ReadableBoxedVec;

use crate::{LazyPreviousDeltaVec, RangeMapVec};

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
            pub timestamp: PerResolution<
                $(ReadableBoxedVec<$index, Timestamp>,)*
                $(ReadableBoxedVec<$epoch_index, Timestamp>,)*
            >,
            pub height_minute10: ReadableBoxedVec<Height, Minute10>,
            pub height_day1: ReadableBoxedVec<Height, Day1>,
            pub height_tx_index_count: LazyPreviousDeltaVec<Height, StoredU64>,
            pub day3_date: ReadableBoxedVec<Day3, Date>,
            pub week1_date: ReadableBoxedVec<Week1, Date>,
            pub month1_date: ReadableBoxedVec<Month1, Date>,
            pub month3_date: ReadableBoxedVec<Month3, Date>,
            pub month6_date: ReadableBoxedVec<Month6, Date>,
            pub year1_date: ReadableBoxedVec<Year1, Date>,
            pub year10_date: ReadableBoxedVec<Year10, Date>,
        }
    };
}

with_resolution_fields!(define_index_sources);
