use std::fmt;

/// Fast formatting trait that writes UTF-8 bytes directly into a buffer,
/// avoiding the `std::fmt` machinery for number types.
pub trait Formattable {
    /// Write formatted UTF-8 bytes. Primary method — all others derive from it.
    fn write_to(&self, buf: &mut Vec<u8>);

    /// Write to a String via write_to.
    #[inline(always)]
    fn fmt_into(&self, f: &mut String) {
        // SAFETY: write_to produces valid UTF-8 (itoa/ryu/Display guarantee this).
        unsafe {
            self.write_to(f.as_mut_vec());
        }
    }

    /// Write in CSV format. Override for types needing CSV escaping (e.g., quoting commas).
    #[inline(always)]
    fn fmt_csv(&self, f: &mut String) -> fmt::Result {
        self.fmt_into(f);
        Ok(())
    }

    /// Write in JSON format. Override for types needing JSON wrapping (e.g., string quotes).
    #[inline(always)]
    fn fmt_json(&self, buf: &mut Vec<u8>) {
        self.write_to(buf);
    }
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

    fn fmt_csv(&self, output: &mut String) -> fmt::Result {
        let mut json = Vec::new();
        self.write_to(&mut json);
        let json = std::str::from_utf8(&json).map_err(|_| fmt::Error)?;

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

impl<T: Formattable> Formattable for Option<T> {
    #[inline]
    fn write_to(&self, buf: &mut Vec<u8>) {
        if let Some(v) = self {
            v.write_to(buf);
        }
    }

    #[inline]
    fn fmt_csv(&self, f: &mut String) -> fmt::Result {
        if let Some(v) = self {
            v.fmt_csv(f)?;
        }
        Ok(())
    }

    #[inline]
    fn fmt_json(&self, buf: &mut Vec<u8>) {
        match self {
            Some(v) => v.fmt_json(buf),
            None => buf.extend_from_slice(b"null"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Formattable;

    #[test]
    fn arrays_format_as_json_and_one_csv_cell() {
        let values = [1_u16, 2, 3];
        let mut json = Vec::new();
        values.fmt_json(&mut json);
        assert_eq!(json, b"[1,2,3]");

        let mut csv = String::new();
        values.fmt_csv(&mut csv).unwrap();
        assert_eq!(csv, "\"[1,2,3]\"");
    }
}
