/// Defines `$id`, one variant per field of `$collection` (named after the field), and the
/// collection's constructors, selectors and iterators, all in field order.
macro_rules! collection {
    (
        $(#[$meta:meta])*
        $id:ident for $collection:ident { $($variant:ident => $field:ident),+ $(,)? }
    ) => {
        $(#[$meta])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
        pub enum $id {
            $($variant),+
        }

        #[allow(dead_code)]
        impl $id {
            pub const ALL: [Self; [$(Self::$variant),+].len()] = [$(Self::$variant),+];

            pub const fn name(self) -> &'static str {
                match self {
                    $(Self::$variant => stringify!($field)),+
                }
            }
        }

        #[allow(dead_code)]
        impl<T> $collection<T> {
            pub fn from_fn(mut create: impl FnMut($id) -> T) -> Self {
                Self {
                    $($field: create($id::$variant)),+
                }
            }

            pub fn try_from_fn<E>(mut create: impl FnMut($id) -> Result<T, E>) -> Result<Self, E> {
                Ok(Self {
                    $($field: create($id::$variant)?),+
                })
            }

            pub fn select(&self, id: $id) -> &T {
                match id {
                    $($id::$variant => &self.$field),+
                }
            }

            pub fn select_mut(&mut self, id: $id) -> &mut T {
                match id {
                    $($id::$variant => &mut self.$field),+
                }
            }

            pub fn iter(&self) -> impl Iterator<Item = &T> {
                [$(&self.$field),+].into_iter()
            }

            pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut T> {
                [$(&mut self.$field),+].into_iter()
            }
        }
    };
}
