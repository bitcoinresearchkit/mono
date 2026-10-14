/// Expand a typed resolution family in its canonical field order.
///
/// Time buckets may be empty; halving/epoch buckets use non-optional values.
/// The callback receives field names, index types, and container type parameters.
#[macro_export]
macro_rules! with_resolution_fields {
    ($callback:ident) => {
        $callback! {
            periods {
                minute10: Minute10 => M10,
                minute30: Minute30 => M30,
                hour1: Hour1 => H1,
                hour4: Hour4 => H4,
                hour12: Hour12 => H12,
                day1: Day1 => D1,
                day3: Day3 => D3,
                week1: Week1 => W1,
                month1: Month1 => Mo1,
                month3: Month3 => Mo3,
                month6: Month6 => Mo6,
                year1: Year1 => Y1,
                year10: Year10 => Y10,
            }
            epochs {
                halving: Halving => HE,
                epoch: Epoch => DE,
            }
        }
    };
}
