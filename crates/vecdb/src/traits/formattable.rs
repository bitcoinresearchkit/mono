/// Fast formatting trait that writes UTF-8 bytes directly into a buffer,
/// avoiding the `std::fmt` machinery for number types. JSON and CSV bodies are
/// built as bytes, so no method needs a `String`.
pub trait Formattable {
    /// Whether this type has an undefined value of its own (NaN, or a sentinel such as
    /// `u64::MAX`), written as JSON `null`.
    const UNDEFINED: bool = false;

    /// Whether a value can be missing (`Option`), written as JSON `null`.
    const MISSING: bool = false;

    /// Write formatted UTF-8 bytes. Primary method — all others derive from it.
    fn write_to(&self, buf: &mut Vec<u8>);

    /// Write one CSV cell. Override for types needing CSV escaping ([`escape_csv_cell`]) or an
    /// empty cell for undefined values.
    #[inline(always)]
    fn fmt_csv(&self, buf: &mut Vec<u8>) {
        self.write_to(buf);
    }

    /// Write in JSON format. Override for types needing JSON wrapping (e.g., string quotes).
    #[inline(always)]
    fn fmt_json(&self, buf: &mut Vec<u8>) {
        self.write_to(buf);
    }
}

/// Makes the bytes written from `start` one CSV cell: when they hold a comma, quote or line
/// break, they are quoted and their quotes doubled, in place.
pub fn escape_csv_cell(buf: &mut Vec<u8>, start: usize) {
    let cell = &buf[start..];
    if !cell
        .iter()
        .any(|byte| matches!(byte, b',' | b'"' | b'\n' | b'\r'))
    {
        return;
    }
    let quotes = cell.iter().filter(|&&byte| byte == b'"').count();
    let end = buf.len();
    buf.resize(end + quotes + 2, 0);
    // Shift the cell right, back to front, doubling quotes on the way.
    let mut write = buf.len() - 1;
    buf[write] = b'"';
    for read in (start..end).rev() {
        let byte = buf[read];
        write -= 1;
        buf[write] = byte;
        if byte == b'"' {
            write -= 1;
            buf[write] = b'"';
        }
    }
    write -= 1;
    buf[write] = b'"';
    debug_assert_eq!(write, start);
}

/// Whether `T`'s JSON values can be `null`: missing or undefined.
pub const fn nullable<T: Formattable>() -> bool {
    T::MISSING || T::UNDEFINED
}

macro_rules! impl_formattable_int {
    ($($t:ty),*) => {
        $(
            impl Formattable for $t {
                #[inline(always)]
                fn write_to(&self, buf: &mut Vec<u8>) {
                    let mut b = itoa::Buffer::new();
                    buf.extend_from_slice(b.format(*self).as_bytes());
                }
            }
        )*
    };
}

impl_formattable_int!(
    u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize
);

macro_rules! impl_formattable_float {
    ($($t:ty),*) => {
        $(
            impl Formattable for $t {
                #[inline(always)]
                fn write_to(&self, buf: &mut Vec<u8>) {
                    let mut b = ryu::Buffer::new();
                    buf.extend_from_slice(b.format(*self).as_bytes());
                }
            }
        )*
    };
}

impl_formattable_float!(f32, f64);

impl Formattable for bool {
    #[inline(always)]
    fn write_to(&self, buf: &mut Vec<u8>) {
        buf.extend_from_slice(if *self { b"true" } else { b"false" });
    }
}

impl<T: Formattable, const N: usize> Formattable for [T; N] {
    fn write_to(&self, buf: &mut Vec<u8>) {
        buf.push(b'[');
        for (index, value) in self.iter().enumerate() {
            if index > 0 {
                buf.push(b',');
            }
            value.fmt_json(buf);
        }
        buf.push(b']');
    }

    fn fmt_csv(&self, buf: &mut Vec<u8>) {
        let start = buf.len();
        self.write_to(buf);
        escape_csv_cell(buf, start);
    }
}

impl<T: Formattable> Formattable for Option<T> {
    const UNDEFINED: bool = T::UNDEFINED;
    const MISSING: bool = true;

    #[inline]
    fn write_to(&self, buf: &mut Vec<u8>) {
        if let Some(v) = self {
            v.write_to(buf);
        }
    }

    #[inline]
    fn fmt_csv(&self, buf: &mut Vec<u8>) {
        if let Some(v) = self {
            v.fmt_csv(buf);
        }
    }

    #[inline]
    fn fmt_json(&self, buf: &mut Vec<u8>) {
        match self {
            Some(v) => v.fmt_json(buf),
            None => buf.extend_from_slice(b"null"),
        }
    }
}
