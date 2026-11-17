//! JSON bodies of numeric columns, written straight from integers. The UTXO set (~12 MB) and raw
//! URPD points (~1 MB) are almost all short numbers: each is formatted into a 32-byte scratch
//! that is appended whole and cut back, about 4× as fast as `serde_json` over `f64` columns.

use bitview_primitives::Date;
use brk_types::{BlockHash, Height};
use serde::Serialize;

use crate::Result;

/// Digits of every number below 10,000, zero-padded to four.
static QUADS: [[u8; 4]; 10_000] = {
    let mut quads = [[0; 4]; 10_000];
    let mut i = 0;
    while i < 10_000 {
        quads[i] = [
            b'0' + (i / 1000) as u8,
            b'0' + (i / 100 % 10) as u8,
            b'0' + (i / 10 % 10) as u8,
            b'0' + (i % 10) as u8,
        ];
        i += 1;
    }
    quads
};

/// Eight ASCII zeros, read as a `u64`.
const ZEROS: u64 = u64::from_le_bytes([b'0'; 8]);

// A number's writer writes into `out` at `at` and returns where it ended. No number takes more
// than 22 bytes, so a writer starting below 10 stays in bounds.

/// Writes `value` at `at`.
#[inline(always)]
pub(super) fn int(out: &mut [u8; 32], at: usize, value: u64) -> usize {
    if value >= 10_000 {
        return long(out, at, value);
    }
    // The four padded digits, shifted past the leading zeros.
    let len = 1 + usize::from(value >= 10) + usize::from(value >= 100) + usize::from(value >= 1000);
    let digits = u32::from_le_bytes(QUADS[value as usize]) >> (8 * (4 - len));
    out[at..at + 4].copy_from_slice(&digits.to_le_bytes());
    at + len
}

/// `int` for values of five digits or more, rare in these columns: out of line.
#[inline(never)]
fn long(out: &mut [u8; 32], at: usize, value: u64) -> usize {
    let len = value.ilog10() as usize + 1;
    let mut rest = value;
    // Four digits at a time from the right, the leading group short when `last` < 3.
    for last in (0..len).rev().step_by(4) {
        let digits = QUADS[(rest % 10_000) as usize];
        rest /= 10_000;
        let skip = 3usize.saturating_sub(last);
        out[at + last + skip - 3..=at + last].copy_from_slice(&digits[skip..]);
    }
    at + len
}

/// Writes `value / 10^D` as `serde_json` writes that `f64`: ryu's shortest decimal reading back
/// as the double, which is the exact one while doubles there are closer than `10^-D` (below
/// 2^52 · 10^-D: 45 million BTC, 45 trillion USD), in ryu's layout: `50.0`, `0.5`, `0.0001`, and
/// below 0.00001 `1e-8` or `5.46e-6`.
#[inline(always)]
pub(super) fn decimal<const D: u32>(out: &mut [u8; 32], at: usize, value: u64) -> usize {
    // Constants, so the division is a multiplication (a runtime `pow` cost the body 20%).
    let unit = const { 10_u64.pow(D) };
    let (whole, fraction) = (value / unit, value % unit);
    if fraction == 0 {
        let at = int(out, at, whole);
        out[at..at + 2].copy_from_slice(b".0");
        return at + 2;
    }
    if whole == 0 && fraction < const { 10_u64.pow(D) / 100_000 } {
        return tiny::<D>(out, at, fraction);
    }
    let digits = fraction_digits::<D>(fraction);
    let at = int(out, at, whole);
    out[at] = b'.';
    out[at + 1..at + 9].copy_from_slice(&digits.to_le_bytes());
    at + 1 + D as usize - trailing_zeros::<D>(digits)
}

/// `decimal` with a sign.
#[inline(always)]
pub(super) fn signed_decimal<const D: u32>(out: &mut [u8; 32], at: usize, value: i64) -> usize {
    if value < 0 {
        out[at] = b'-';
        return decimal::<D>(out, at + 1, value.unsigned_abs());
    }
    decimal::<D>(out, at, value as u64)
}

/// The `D` digits of `fraction`, zero-padded, as the bytes of a `u64`: the first in the low byte,
/// the bytes past them zero.
fn fraction_digits<const D: u32>(fraction: u64) -> u64 {
    let quad = |n: u64| u64::from(u32::from_le_bytes(QUADS[n as usize]));
    match D {
        8 => quad(fraction / 10_000) | quad(fraction % 10_000) << 32,
        2 => quad(fraction) >> 16,
        _ => unreachable!("decimals are 8 (BTC) or 2 (USD)"),
    }
}

/// How many of `digits`' `D` digits trail as zeros.
fn trailing_zeros<const D: u32>(digits: u64) -> usize {
    ((digits ^ ZEROS >> (64 - 8 * D)) << (64 - 8 * D)).leading_zeros() as usize / 8
}

/// Under 0.00001, ryu's scientific form: `1e-8`, `5.46e-6`.
#[inline(never)]
fn tiny<const D: u32>(out: &mut [u8; 32], at: usize, fraction: u64) -> usize {
    let digits = fraction_digits::<D>(fraction);
    let leading = (digits ^ ZEROS).trailing_zeros() as usize / 8;
    let digits = &digits.to_le_bytes()[leading..D as usize - trailing_zeros::<D>(digits)];
    out[at] = digits[0];
    let mut end = at + 1;
    if digits.len() > 1 {
        out[end] = b'.';
        out[end + 1..end + digits.len()].copy_from_slice(&digits[1..]);
        end += digits.len();
    }
    out[end..end + 3].copy_from_slice(&[b'e', b'-', b'1' + leading as u8]);
    end + 3
}

/// Opens a body with the block header both the UTXO set and the URPD start with.
pub(super) fn open(body: &mut Vec<u8>, height: Height, hash: &BlockHash, date: &Date) -> Result<()> {
    body.extend_from_slice(b"{\"height\":");
    append(body, u64::from(height), int);
    field(body, "hash", hash)?;
    field(body, "date", date)
}

/// Appends `,"name":` and `value` as `serde_json` writes it.
pub(super) fn field(body: &mut Vec<u8>, name: &str, value: &impl Serialize) -> Result<()> {
    key(body, name);
    Ok(serde_json::to_writer(&mut *body, value)?)
}

/// Appends `,"name":`.
pub(super) fn key(body: &mut Vec<u8>, name: &str) {
    body.extend_from_slice(b",\"");
    body.extend_from_slice(name.as_bytes());
    body.extend_from_slice(b"\":");
}

/// Appends `value` as `put` writes it.
pub(super) fn append<T>(body: &mut Vec<u8>, value: T, put: fn(&mut [u8; 32], usize, T) -> usize) {
    let mut scratch = [0; 32];
    let len = put(&mut scratch, 0, value);
    body.extend_from_slice(&scratch[..len]);
}

/// Appends one number of each row as a JSON array: the number and its comma go to a scratch
/// that is appended whole, then cut back to their length. `put` is a closure calling a writer:
/// the writer passed itself measured 20% slower (not inlined).
#[inline(always)]
pub(super) fn append_array<R>(
    body: &mut Vec<u8>,
    rows: &[R],
    put: impl Fn(&mut [u8; 32], &R) -> usize,
) {
    body.push(b'[');
    let mut scratch = [0; 32];
    for row in rows {
        let len = put(&mut scratch, row);
        scratch[len] = b',';
        body.extend_from_slice(&scratch);
        body.truncate(body.len() - 31 + len);
    }
    if body.last() == Some(&b',') {
        body.pop();
    }
    body.push(b']');
}
