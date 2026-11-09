use bitview_primitives::CentsCompact;

use crate::Modes;

/// Cumulative mode supplies at one price in a sorted, unique-price distribution.
pub(crate) struct CumulativeBucket {
    pub price: CentsCompact,
    pub supplies: Modes<u64>,
}
