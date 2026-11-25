use crate::chunk_config::ModeSpec;
use crate::compression_intermediates::Bid;
use crate::constants::Bitlen;
use crate::data_types::{LatentPriv, Number, NumberPriv};
use crate::dyn_slices::DynLatentSlice;
use crate::errors::{PcoError, PcoResult};
use crate::metadata::Mode;
use crate::mode::float_mult::FloatMultConfig;
use crate::mode::{classic, dict, float_mult, float_quant};
use crate::{sampling, ChunkConfig};
use std::fmt::{Debug, Display};
use std::mem::MaybeUninit;
use std::ops::*;

pub(crate) trait Float:
  Add<Output = Self>
  + AddAssign
  + Copy
  + Debug
  + Display
  + Mul<Output = Self>
  + Neg<Output = Self>
  + Number
  + PartialOrd
  + RemAssign
  + Send
  + Sync
  + Sub<Output = Self>
  + SubAssign
  + Div<Output = Self>
{
  /// Number of bits that aren't used for exponent or sign.
  /// E.g. for f32 this should be 23.
  const PRECISION_BITS: Bitlen;
  const ZERO: Self;
  const MAX_FOR_SAMPLING: Self;

  fn abs(self) -> Self;
  fn inv(self) -> Self;
  fn round(self) -> Self;
  /// This only needs to cover a small range (from 2^-BITS to 2^BITS) and might
  /// not be valid outside of it.
  fn exp2(power: i32) -> Self;
  fn from_f64(x: f64) -> Self;
  fn to_f64(self) -> f64;
  fn is_normal(self) -> bool;
  fn is_sign_positive_(&self) -> bool;
  /// Returns the float's exponent. For instance, for f32 this should be
  /// between -127 and +126.
  fn exponent(&self) -> i32;
  fn trailing_zeros(&self) -> u32;
  fn max(a: Self, b: Self) -> Self;
  fn min(a: Self, b: Self) -> Self;

  /// This should use something like [`f32::to_bits()`]
  fn to_latent_bits(self) -> Self::L;
  /// This should be the inverse of `to_latent_bits`.
  fn from_latent_bits(l: Self::L) -> Self;
  /// Surjectively maps the latent to the set of integers in this floating
  /// point type. E.g. 3.0, Inf, and NaN are int floats, but 3.5 is not.
  ///
  /// This is written branchlessly, treating the sign as arithmetic rather than
  /// control flow, so that hot loops vectorize to a single int->float
  /// conversion per number instead of one per sign case.
  #[inline]
  fn int_float_from_latent(l: Self::L) -> Self {
    let sign = Self::L::MID;
    let abs_mask = sign - Self::L::ONE;
    let gpi = Self::L::ONE << (Self::PRECISION_BITS + 1);
    // above gpi, int floats continue linearly in bit space rather than in value
    let gpi_bits_offset = Self::from_latent_numerical(gpi)
      .to_latent_bits()
      .wrapping_sub(gpi);

    // l >= MID means positive, with magnitude l - MID; below MID it's negative
    // with magnitude MID - 1 - l. Both are `(l ^ neg) & abs_mask` for the right
    // choice of `neg`.
    let neg = !(Self::L::ZERO.wrapping_sub(l >> (Self::L::BITS - 1)));
    let abs_int = (l ^ neg) & abs_mask;
    let abs_float = if abs_int < gpi {
      Self::from_latent_numerical(abs_int)
    } else {
      Self::from_latent_bits(abs_int.wrapping_add(gpi_bits_offset))
    };
    Self::from_latent_bits(abs_float.to_latent_bits() | (neg & sign))
  }
  /// This should be the inverse of `int_float_from_latent`.
  fn int_float_to_latent(self) -> Self::L;
  /// This should map from e.g. 7_u32 -> 7.0_f32
  fn from_latent_numerical(l: Self::L) -> Self;
}

fn filter_sample<F: Float>(num: &F) -> Option<F> {
  // We can compress infinities, nans, and baby floats, but we can't learn
  // the mode from them.
  if num.is_normal() {
    let abs = num.abs();
    if abs <= F::MAX_FOR_SAMPLING {
      return Some(abs);
    }
  }
  None
}

fn choose_mode_bids<F: Float>(nums: &[F], chunk_config: &ChunkConfig) -> PcoResult<Vec<Bid<F>>> {
  let bids = match chunk_config.mode_spec {
    ModeSpec::Auto => {
      // up to 3 bids: classic, float mult, float quant modes
      let mut bids: Vec<Bid<F>> = vec![Bid {
        mode: Mode::Classic,
        bits_saved_per_num: 0.0,
        split_fn: Box::new(|nums| classic::split_latents(nums)),
      }];

      if let Some(sample) = sampling::choose_sample(nums, filter_sample) {
        bids.extend(float_mult::compute_bid(&sample));
        bids.extend(float_quant::compute_bid(&sample));
      }

      bids
    }
    ModeSpec::Classic => vec![Bid {
      mode: Mode::Classic,
      bits_saved_per_num: 0.0,
      split_fn: Box::new(|nums| classic::split_latents(nums)),
    }],
    ModeSpec::TryFloatMult(base_f64) => {
      let base = F::from_f64(base_f64);
      let float_mult_config = FloatMultConfig {
        base,
        inv_base: base.inv(),
      };
      vec![Bid {
        mode: Mode::float_mult(base),
        bits_saved_per_num: 0.0,
        split_fn: Box::new(move |nums| float_mult::split_latents(nums, float_mult_config)),
      }]
    }
    ModeSpec::TryFloatQuant(k) => vec![Bid {
      mode: Mode::FloatQuant(k),
      bits_saved_per_num: 0.0,
      // Note that this is only invoked after the mode is validated; k is a
      // shift width, so splitting with an out-of-range one is UB-adjacent.
      split_fn: Box::new(move |nums| float_quant::split_latents(nums, k)),
    }],
    ModeSpec::TryIntMult(_) => {
      return Err(PcoError::invalid_argument(
        "unable to use int mult mode on floats",
      ))
    }
    ModeSpec::TryDict => vec![dict::compute_bid(nums)],
  };

  Ok(bids)
}

macro_rules! impl_float {
  ($t: ty, $latent: ty, $exp_offset: expr) => {
    impl Float for $t {
      /// Number of bits in the representation of the significand, excluding the implicit
      /// leading bit.  (In Rust, `MANTISSA_DIGITS` does include the implicit leading bit.)
      const PRECISION_BITS: Bitlen = Self::MANTISSA_DIGITS as Bitlen - 1;
      const ZERO: Self = 0.0;
      const MAX_FOR_SAMPLING: Self = Self::MAX * 0.5;

      #[inline]
      fn abs(self) -> Self {
        self.abs()
      }

      fn inv(self) -> Self {
        1.0 / self
      }

      #[inline]
      fn round(self) -> Self {
        self.round()
      }

      #[inline]
      fn exp2(power: i32) -> Self {
        Self::from_bits((($exp_offset + power) as $latent) << Self::PRECISION_BITS)
      }

      #[inline]
      fn from_f64(x: f64) -> Self {
        x as Self
      }

      #[inline]
      fn to_f64(self) -> f64 {
        self as f64
      }

      #[inline]
      fn is_normal(self) -> bool {
        self.is_normal()
      }

      #[inline]
      fn is_sign_positive_(&self) -> bool {
        self.is_sign_positive()
      }

      #[inline]
      fn exponent(&self) -> i32 {
        (self.abs().to_bits() >> Self::PRECISION_BITS) as i32 - $exp_offset
      }

      #[inline]
      fn trailing_zeros(&self) -> u32 {
        self.to_bits().trailing_zeros()
      }

      #[inline]
      fn max(a: Self, b: Self) -> Self {
        Self::max(a, b)
      }

      #[inline]
      fn min(a: Self, b: Self) -> Self {
        Self::min(a, b)
      }

      #[inline]
      fn to_latent_bits(self) -> Self::L {
        self.to_bits()
      }

      #[inline]
      fn from_latent_bits(l: Self::L) -> Self {
        Self::from_bits(l)
      }

      #[inline]
      fn int_float_to_latent(self) -> Self::L {
        let abs = self.abs();
        let gpi = 1 << Self::MANTISSA_DIGITS;
        let gpi_float = gpi as Self;
        let abs_int = if abs < gpi_float {
          abs as Self::L
        } else {
          gpi + (abs.to_bits() - gpi_float.to_bits())
        };
        if self.is_sign_positive() {
          Self::L::MID + abs_int
        } else {
          // -1 because we need to distinguish -0.0 from +0.0
          Self::L::MID - 1 - abs_int
        }
      }

      #[inline]
      fn from_latent_numerical(l: Self::L) -> Self {
        l as Self
      }
    }
  };
}

macro_rules! impl_float_number {
  ($t: ty, $latent: ty, $sign_bit_mask: expr, $header_byte: expr) => {
    impl NumberPriv for $t {
      const NUMBER_TYPE_BYTE: u8 = $header_byte;

      type L = $latent;

      fn mode_is_valid(mode: &Mode) -> bool {
        match mode {
          Mode::Classic | Mode::Dict(_) => true,
          Mode::FloatMult(dyn_latent) => {
            let base_latent = *dyn_latent.downcast_ref::<Self::L>().unwrap();
            let base = Self::from_latent_ordered(base_latent);
            base.is_finite() && base.abs() > Self::ZERO
          }
          Mode::FloatQuant(k) => *k > 0 && *k <= Self::PRECISION_BITS,
          Mode::IntMult(_) => false,
        }
      }
      fn choose_mode_bids(nums: &[Self], config: &ChunkConfig) -> PcoResult<Vec<Bid<Self>>> {
        choose_mode_bids(nums, config)
      }

      #[inline]
      fn from_latent_ordered(l: Self::L) -> Self {
        // sign bit set means a positive float, where we flip only that bit;
        // otherwise it's a negative float and we flip everything
        let flip_all = !(Self::L::ZERO.wrapping_sub(l >> (Self::L::BITS - 1)));
        Self::from_bits(l ^ (flip_all | $sign_bit_mask))
      }
      #[inline]
      fn to_latent_ordered(self) -> Self::L {
        let mem_layout = self.to_bits();
        // flip everything for a negative float, else just the sign
        let flip_all = Self::L::ZERO.wrapping_sub(mem_layout >> (Self::L::BITS - 1));
        mem_layout ^ (flip_all | $sign_bit_mask)
      }
      fn join_latents(
        mode: &Mode,
        primary: DynLatentSlice,
        secondary: Option<DynLatentSlice>,
        dst: &mut [MaybeUninit<Self>],
      ) -> PcoResult<()> {
        match mode {
          Mode::Classic => classic::join_latents(primary, dst),
          Mode::Dict(dict) => dict::join_latents(dict, primary, dst),
          Mode::FloatMult(dyn_latent) => {
            let base = Self::from_latent_ordered(*dyn_latent.downcast_ref::<Self::L>().unwrap());
            float_mult::join_latents(base, primary, secondary, dst)
          }
          Mode::FloatQuant(k) => float_quant::join_latents::<Self>(*k, primary, secondary, dst),
          Mode::IntMult(_) => unreachable!("impossible mode for floats"),
        }
      }
    }
  };
}

impl_float!(f32, u32, 127);
impl_float!(f64, u64, 1023);
impl_float_number!(f32, u32, 1_u32 << 31, 5);
impl_float_number!(f64, u64, 1_u64 << 63, 6);
