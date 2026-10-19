use crate::constants::Bitlen;
use crate::data_types::Latent;
use crate::read_write_uint::ReadWriteUint;

// doesn't handle the case when n >= U::BITS
#[inline]
pub fn lowest_bits_fast<U: ReadWriteUint>(x: U, n: Bitlen) -> U {
  x & ((U::ONE << n) - U::ONE)
}

#[inline]
pub fn lowest_bits<U: ReadWriteUint>(x: U, n: Bitlen) -> U {
  if n >= U::BITS {
    x
  } else {
    lowest_bits_fast(x, n)
  }
}

pub fn bits_to_encode_offset<L: Latent>(max_offset: L) -> Bitlen {
  L::BITS - max_offset.leading_zeros()
}

pub const fn bits_to_encode_offset_bits<L: Latent>() -> Bitlen {
  (Bitlen::BITS - L::BITS.leading_zeros()) as Bitlen
}
