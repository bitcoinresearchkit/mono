mod u8x2;
pub use u8x2::*;
mod u8x20;
pub use u8x20::*;
mod u8x32;
pub use u8x32::*;
mod u8x33;
pub use u8x33::*;
mod u8x65;
pub use u8x65::*;

#[cfg(feature = "storage")]
const HEX: &[u8; 16] = b"0123456789abcdef";

/// Appends `bytes` as lowercase hex.
#[cfg(feature = "storage")]
#[inline]
pub(crate) fn push_hex(bytes: &[u8], buf: &mut Vec<u8>) {
    buf.reserve(bytes.len() * 2);
    for &byte in bytes {
        buf.push(HEX[usize::from(byte >> 4)]);
        buf.push(HEX[usize::from(byte & 0x0f)]);
    }
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
