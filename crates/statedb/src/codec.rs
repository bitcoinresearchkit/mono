use crate::util::invalid;
use std::io::Result;

pub(crate) fn encode(mut value: u64, out: &mut Vec<u8>) {
    while value >= 128 {
        out.push(value as u8 | 128);
        value >>= 7;
    }
    out.push(value as u8);
}
pub(crate) fn decode(data: &mut &[u8]) -> Result<u64> {
    let mut value = 0;
    for shift in (0..70).step_by(7) {
        let (&byte, tail) = data
            .split_first()
            .ok_or_else(|| invalid("truncated integer"))?;
        *data = tail;
        if shift == 63 && byte > 1 {
            return Err(invalid("integer overflow"));
        }
        value |= u64::from(byte & 127) << shift;
        if byte < 128 {
            if shift != 0 && byte == 0 {
                return Err(invalid("noncanonical integer"));
            }
            return Ok(value);
        }
    }
    Err(invalid("integer overflow"))
}

#[cfg(test)]
mod tests {
    use super::{decode, encode};

    #[test]
    fn canonical_integers_reject_truncation_overflow_and_overlong_forms() {
        for value in [
            0,
            1,
            127,
            128,
            255,
            16_383,
            16_384,
            u32::MAX as u64,
            u64::MAX,
        ] {
            let mut bytes = Vec::new();
            encode(value, &mut bytes);
            let mut input = bytes.as_slice();
            assert_eq!(decode(&mut input).unwrap(), value);
            assert!(input.is_empty());
            for end in 0..bytes.len() {
                assert!(decode(&mut &bytes[..end]).is_err());
            }
        }
        for bytes in [&[128, 0][..], &[255; 10][..], &[128; 11][..]] {
            assert!(decode(&mut &bytes[..]).is_err());
        }
    }
}
