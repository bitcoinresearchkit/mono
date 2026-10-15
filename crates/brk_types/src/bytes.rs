use std::fmt;

const HEX: &[u8; 16] = b"0123456789abcdef";

/// Appends `bytes` as lowercase hex.
#[inline]
pub(crate) fn push_hex(bytes: &[u8], buf: &mut Vec<u8>) {
    buf.reserve(bytes.len() * 2);
    for &byte in bytes {
        buf.push(HEX[usize::from(byte >> 4)]);
        buf.push(HEX[usize::from(byte & 0x0f)]);
    }
}

/// A 32-byte hash as lowercase hex in reversed byte order (Bitcoin's display order for txids
/// and block hashes).
#[inline]
pub(crate) fn reversed_hex(bytes: &[u8; 32]) -> [u8; 64] {
    std::array::from_fn(|i| {
        let byte = bytes[31 - i / 2];
        HEX[usize::from(if i % 2 == 0 { byte >> 4 } else { byte & 0x0f })]
    })
}

/// Writes `bytes` as lowercase hex.
pub(crate) fn fmt_hex(bytes: &[u8], f: &mut fmt::Formatter<'_>) -> fmt::Result {
    use fmt::Write;
    bytes.iter().try_for_each(|&byte| {
        f.write_char(char::from(HEX[usize::from(byte >> 4)]))?;
        f.write_char(char::from(HEX[usize::from(byte & 0x0f)]))
    })
}

/// Parses exactly `N` bytes of hex (either case).
pub(crate) fn parse_hex<const N: usize>(hex: &str) -> Option<[u8; N]> {
    let digits = hex.as_bytes();
    if digits.len() != N * 2 {
        return None;
    }
    let nibble = |digit: u8| (digit as char).to_digit(16).map(|value| value as u8);
    let mut bytes = [0; N];
    let (pairs, _) = digits.as_chunks::<2>();
    for (byte, &[high, low]) in bytes.iter_mut().zip(pairs) {
        *byte = (nibble(high)? << 4) | nibble(low)?;
    }
    Some(bytes)
}

/// Serializes a `Display` value as a string without building one (scripts: `Display` is assembly,
/// `format_args!("{:x}")` is hex).
pub(crate) struct DisplayStr<T>(pub T);

impl<T: fmt::Display> serde::Serialize for DisplayStr<T> {
    #[inline]
    fn serialize<S: serde::Serializer>(
        &self,
        serializer: S,
    ) -> std::result::Result<S::Ok, S::Error> {
        serializer.collect_str(&self.0)
    }
}
