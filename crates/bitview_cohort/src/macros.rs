macro_rules! define_cohort_id {
    (
        $id:ident for $collection:ident {
            $($variant:ident => $field:ident),+ $(,)?
        }
    ) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
        #[repr(u8)]
        pub enum $id {
            $($variant),+
        }

        impl $id {
            pub const ALL: &'static [Self] = &[$(Self::$variant),+];

            #[inline]
            pub const fn index(self) -> usize {
                self as usize
            }

            #[inline]
            pub fn select<T>(self, values: &$collection<T>) -> &T {
                match self {
                    $(Self::$variant => &values.$field),+
                }
            }

            #[inline]
            pub fn select_mut<T>(self, values: &mut $collection<T>) -> &mut T {
                match self {
                    $(Self::$variant => &mut values.$field),+
                }
            }
        }

        impl_cohort_collection!($id for $collection { $($variant => $field),+ });
    };
}

macro_rules! impl_cohort_collection {
    ($id:ident for $collection:ident { $($variant:ident => $field:ident),+ $(,)? }) => {
        impl<T> $collection<T> {
            pub fn as_array(&self) -> [&T; [$($id::$variant),+].len()] {
                [$(&self.$field),+]
            }

            pub fn as_array_mut(&mut self) -> [&mut T; [$($id::$variant),+].len()] {
                [$(&mut self.$field),+]
            }

            pub fn iter(&self) -> impl DoubleEndedIterator<Item = &T> + ExactSizeIterator {
                self.as_array().into_iter()
            }

            pub fn iter_mut(&mut self) -> impl DoubleEndedIterator<Item = &mut T> + ExactSizeIterator {
                self.as_array_mut().into_iter()
            }

            pub fn par_iter_mut(&mut self) -> impl rayon::iter::ParallelIterator<Item = &mut T>
            where
                T: Send + Sync,
            {
                rayon::iter::IntoParallelIterator::into_par_iter(self.as_array_mut())
            }

            pub fn from_fn(mut f: impl FnMut($id) -> T) -> Self {
                Self {
                    $($field: f($id::$variant)),+
                }
            }

            pub fn try_from_fn<E>(mut f: impl FnMut($id) -> Result<T, E>) -> Result<Self, E> {
                Ok(Self {
                    $($field: f($id::$variant)?),+
                })
            }
        }
        impl_collection_formattable!($collection { $($field),+ });
    };
}

macro_rules! impl_collection_formattable {
    (
        $collection:ident {
            $($field:ident),+ $(,)?
        }
    ) => {
        #[cfg(feature = "storage")]
        impl<T: vecdb::Formattable> vecdb::Formattable for $collection<T> {
            fn write_to(&self, output: &mut Vec<u8>) {
                output.push(b'{');
                let mut first = true;
                $(
                    if !first {
                        output.push(b',');
                    }
                    first = false;
                    output.extend_from_slice(concat!("\"", stringify!($field), "\":").as_bytes());
                    vecdb::Formattable::fmt_json(&self.$field, output);
                )+
                let _ = first;
                output.push(b'}');
            }

            fn fmt_csv(&self, output: &mut Vec<u8>) {
                let start = output.len();
                vecdb::Formattable::write_to(self, output);
                vecdb::escape_csv_cell(output, start);
            }
        }
    };
}
