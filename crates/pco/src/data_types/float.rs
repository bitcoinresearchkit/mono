use crate::chunk_config::ModeSpec;
use crate::compression_intermediates::Bid;
use crate::constants::Bitlen;
use crate::data_types::{LatentPriv, Number, NumberPriv};
use crate::describers::LatentDescriber;
use crate::dyn_slices::DynLatentSlice;
use crate::errors::{PcoError, PcoResult};
use crate::metadata::per_latent_var::PerLatentVar;
use crate::metadata::{ChunkMeta, Mode};
use crate::mode::float_mult::FloatMultConfig;
use crate::mode::{classic, dict, float_mult, float_quant};
use crate::{describers, sampling, ChunkConfig};
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
  /// This should surjectively map the latent to the set of integers in its
  /// floating point type. E.g. 3.0, Inf, and NaN are int floats, but 3.5 is
  /// not.
  fn int_float_from_latent(l: Self::L) -> Self;
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
      fn int_float_from_latent(l: Self::L) -> Self {
        let mid = Self::L::MID;
        let (negative, abs_int) = if l >= mid {
          (false, l - mid)
        } else {
          (true, mid - 1 - l)
        };
        let gpi = 1 << Self::MANTISSA_DIGITS;
        let abs_float = if abs_int < gpi {
          abs_int as Self
        } else {
          Self::from_bits((gpi as Self).to_bits() + (abs_int - gpi))
        };
        if negative {
          -abs_float
        } else {
          abs_float
        }
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
        if l & $sign_bit_mask > 0 {
          // positive float
          Self::from_bits(l ^ $sign_bit_mask)
        } else {
          // negative float
          Self::from_bits(!l)
        }
      }
      #[inline]
      fn to_latent_ordered(self) -> Self::L {
        let mem_layout = self.to_bits();
        if mem_layout & $sign_bit_mask > 0 {
          // negative float
          !mem_layout
        } else {
          // positive float
          mem_layout ^ $sign_bit_mask
        }
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

    impl Number for $t {
      fn get_latent_describers(meta: &ChunkMeta) -> PerLatentVar<LatentDescriber> {
        describers::match_classic_mode::<Self>(meta, " ULPs")
          .or_else(|| describers::match_float_modes::<Self>(meta))
          .expect("invalid mode for float type")
      }
    }
  };
}

impl_float!(f32, u32, 127);
impl_float!(f64, u64, 1023);
impl_float_number!(f32, u32, 1_u32 << 31, 5);
impl_float_number!(f64, u64, 1_u64 << 63, 6);
