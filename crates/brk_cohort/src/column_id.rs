macro_rules! define_column_id {
    (
        $id:ident for $row:ident, version = $version:literal {
            $($variant:ident => $field:ident),+ $(,)?
        }
    ) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
        #[repr(u8)]
        pub enum $id {
            $($variant),+
        }

        impl $id {
            #[inline]
            pub fn select<T>(self, row: &$row<T>) -> &T {
                match self {
                    $(Self::$variant => &row.$field),+
                }
            }

            #[inline]
            pub fn select_mut<T>(self, row: &mut $row<T>) -> &mut T {
                match self {
                    $(Self::$variant => &mut row.$field),+
                }
            }
        }

        impl<T> $row<T> {
            pub fn from_fn(mut f: impl FnMut($id) -> T) -> Self {
                Self {
                    $($field: f($id::$variant)),+
                }
            }
        }

        impl vecdb::ColumnId for $id {
            type Row<T>
                = $row<T>
            where
                T: vecdb::VecValue;

            const VERSION: vecdb::Version = vecdb::Version::new($version);
            const ALL: &'static [Self] = &[$(Self::$variant),+];

            #[inline]
            fn index(self) -> usize {
                self as usize
            }

            #[inline]
            fn get<T: vecdb::VecValue>(self, row: &Self::Row<T>) -> &T {
                self.select(row)
            }

            #[inline]
            fn get_mut<T: vecdb::VecValue>(self, row: &mut Self::Row<T>) -> &mut T {
                self.select_mut(row)
            }

            #[inline]
            fn from_fn<T, F>(f: F) -> Self::Row<T>
            where
                T: vecdb::VecValue,
                F: FnMut(Self) -> T,
            {
                $row::from_fn(f)
            }

            #[inline]
            fn map<T, U, F>(row: Self::Row<T>, mut f: F) -> Self::Row<U>
            where
                T: vecdb::VecValue,
                U: vecdb::VecValue,
                F: FnMut(T) -> U,
            {
                $row {
                    $($field: f(row.$field)),+
                }
            }
        }

        impl<T: vecdb::Formattable> vecdb::Formattable for $row<T> {
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

            fn fmt_csv(&self, output: &mut String) -> std::fmt::Result {
                let mut json = Vec::new();
                vecdb::Formattable::write_to(self, &mut json);
                let json = std::str::from_utf8(&json).map_err(|_| std::fmt::Error)?;

                output.push('"');
                for character in json.chars() {
                    if character == '"' {
                        output.push('"');
                    }
                    output.push(character);
                }
                output.push('"');
                Ok(())
            }
        }
    };
}

macro_rules! impl_column_row_formattable {
    (
        $row:ident {
            $($field:ident),+ $(,)?
        }
    ) => {
        impl<T: vecdb::Formattable> vecdb::Formattable for $row<T> {
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

            fn fmt_csv(&self, output: &mut String) -> std::fmt::Result {
                let mut json = Vec::new();
                vecdb::Formattable::write_to(self, &mut json);
                let json = std::str::from_utf8(&json).map_err(|_| std::fmt::Error)?;

                output.push('"');
                for character in json.chars() {
                    if character == '"' {
                        output.push('"');
                    }
                    output.push(character);
                }
                output.push('"');
                Ok(())
            }
        }
    };
}
