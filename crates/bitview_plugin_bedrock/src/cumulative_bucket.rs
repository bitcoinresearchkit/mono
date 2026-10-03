use bitview_primitives::CentsCompact;

use crate::MODE_COUNT;

/// Cumulative mode supplies at one price in a sorted, unique-price distribution.
pub(crate) struct CumulativeBucket {
    pub price: CentsCompact,
    pub supplies: [u64; MODE_COUNT],
}
