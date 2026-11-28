use bitview_traversable::Traversable;

/// Declares every moving-average period once: field, id suffix and span in days. `every` period is
/// averaged both ways ([`Periods`]), `sma_only` ones only as simple averages ([`SmaPeriods`]), which
/// the Pi cycle serves, so the table itself hides them.
macro_rules! periods {
    (
        every: { $($(#[$doc:meta])* $field:ident: $suffix:literal = $days:expr,)* }
        sma_only: { $($only:tt)* }
    ) => {
        table!(
            /// The exponential averages' periods.
            Periods { $($(#[$doc])* $field: $suffix = $days,)* }
        );
        table!(
            /// The simple averages' periods: [`Periods`]' and the Pi cycle top's.
            SmaPeriods { $($(#[$doc])* $field: $suffix = $days,)* $($only)* }
        );

        impl<T> Periods<T> {
            /// One value per period from its id suffix and this period's value.
            pub fn try_map<U, E>(
                &self,
                mut map: impl FnMut(&'static str, &T) -> Result<U, E>,
            ) -> Result<Periods<U>, E> {
                Ok(Periods {
                    $($field: map($suffix, &self.$field)?,)*
                })
            }
        }
    };
}

macro_rules! table {
    ($(#[$meta:meta])* $name:ident { $($(#[$doc:meta])* $field:ident: $suffix:literal = $days:expr,)* }) => {
        $(#[$meta])*
        #[derive(Clone, Traversable)]
        pub struct $name<T> {
            $($(#[$doc])* pub $field: T,)*
        }

        impl<T> $name<T> {
            /// One value per period from its id suffix and span in days.
            pub fn try_from_fn<E>(
                mut create: impl FnMut(&'static str, usize) -> Result<T, E>,
            ) -> Result<Self, E> {
                Ok(Self {
                    $($field: create($suffix, $days)?,)*
                })
            }

            /// Each period's span in days and value.
            pub fn iter_mut(&mut self) -> impl Iterator<Item = (usize, &mut T)> {
                [$(($days, &mut self.$field),)*].into_iter()
            }
        }
    };
}

periods! {
    every: {
        /// Uses a trailing 7-day span.
        _1w: "1w" = 7,
        /// Uses a trailing 8-day span.
        _8d: "8d" = 8,
        /// Uses a trailing 13-day span.
        _13d: "13d" = 13,
        /// Uses a trailing 21-day span.
        _21d: "21d" = 21,
        /// Uses a trailing 30-day span.
        _1m: "1m" = 30,
        /// Uses a trailing 34-day span.
        _34d: "34d" = 34,
        /// Uses a trailing 50-day span.
        _50d: "50d" = 50,
        /// Uses a trailing 55-day span.
        _55d: "55d" = 55,
        /// Uses a trailing 89-day span.
        _89d: "89d" = 89,
        /// Uses a trailing 144-day span.
        _144d: "144d" = 144,
        /// Uses a trailing 200-day span.
        _200d: "200d" = 200,
        /// Uses a trailing 365-day span.
        _1y: "1y" = 365,
        /// Uses a trailing 730-day span.
        _2y: "2y" = 2 * 365,
        /// Uses a trailing 1,400-day span.
        _200w: "200w" = 200 * 7,
        /// Uses a trailing 1,460-day span.
        _4y: "4y" = 4 * 365,
    }
    sma_only: {
        #[traversable(hidden)]
        _111d: "111d" = 111,
        #[traversable(hidden)]
        _350d: "350d" = 350,
    }
}
