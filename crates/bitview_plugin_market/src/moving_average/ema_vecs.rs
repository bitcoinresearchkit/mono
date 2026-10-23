use bitview_traversable::Traversable;

/// Declares every EMA period once: field, id suffix and span in days.
macro_rules! ema_periods {
    ($($(#[$doc:meta])* $field:ident: $suffix:literal = $days:expr,)*) => {
        #[derive(Clone, Traversable)]
        pub struct EmaVecs<T> {
            $($(#[$doc])* pub $field: T,)*
        }

        impl<T> EmaVecs<T> {
            /// One value per period from its id suffix and span in days.
            pub fn try_from_fn<E>(
                mut create: impl FnMut(&'static str, usize) -> Result<T, E>,
            ) -> Result<Self, E> {
                Ok(Self {
                    $($field: create($suffix, $days)?,)*
                })
            }

            /// One value per period from its id suffix and this period's value.
            pub fn try_map<U, E>(
                &self,
                mut map: impl FnMut(&'static str, &T) -> Result<U, E>,
            ) -> Result<EmaVecs<U>, E> {
                Ok(EmaVecs {
                    $($field: map($suffix, &self.$field)?,)*
                })
            }

            /// Each period's span in days and value.
            pub fn iter_mut(&mut self) -> impl Iterator<Item = (usize, &mut T)> {
                [$(($days, &mut self.$field),)*].into_iter()
            }
        }
    };
}

ema_periods! {
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
