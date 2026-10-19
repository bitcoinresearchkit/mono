use std::{
  cmp::{max, min},
  mem::{self, MaybeUninit},
};

use crate::{
  compression_intermediates::Bid,
  constants::{Bitlen, MULT_REQUIRED_BITS_SAVED_PER_NUM},
  data_types::{float::Float, latent_priv::LatentPriv, SplitLatents},
  dyn_slices::DynLatentSlice,
  errors::PcoResult,
  metadata::{DynLatents, Mode},
  mode::int_mult,
  sampling::{self, PrimaryLatentAndSavings},
};

#[inline(never)]
pub(crate) fn join_latents<F: Float>(
  base: F,
  primary: DynLatentSlice,
  secondary: Option<DynLatentSlice>,
  dst: &mut [MaybeUninit<F>],
) -> PcoResult<()> {
  let primary = primary.downcast::<F::L>().unwrap();
  let secondary = secondary.unwrap().downcast::<F::L>().unwrap();
  for ((&mult, &adj), dst) in primary.iter().zip(secondary.iter()).zip(dst.iter_mut()) {
    let unadjusted = F::int_float_from_latent(mult) * base;
    dst.write(F::from_latent_ordered(
      unadjusted
        .to_latent_ordered()
        .wrapping_add(adj)
        .toggle_center(),
    ));
  }

  Ok(())
}

pub(crate) fn split_latents<F: Float>(page_nums: &[F], config: FloatMultConfig<F>) -> SplitLatents {
  let FloatMultConfig { base, inv_base } = config;
  let n = page_nums.len();
  let mut primary = Vec::with_capacity(n);
  let mut adjustments = Vec::with_capacity(n);
  for &num in page_nums {
    let mult = (num * inv_base).round();
    primary.push(F::int_float_to_latent(mult));
    adjustments.push(
      num
        .to_latent_ordered()
        .wrapping_sub((mult * base).to_latent_ordered())
        // ULP adjustments are naturally signed quantities, so we toggle them so
        // that 0 is in the middle of the range
        .toggle_center(),
    );
  }

  SplitLatents {
    primary: DynLatents::new(primary),
    secondary: Some(DynLatents::new(adjustments)),
  }
}

// The rest of this file concerns automatically detecting the float `base`
// such that `x = mult * base + adj * ULP` usefully splits a delta `x` into
// latent variables `mult` and `adj` (if such a `base` exists).
//
// Somewhat different from int mult, we simplistically model that each `x` is
// a multiple of `base` with floating point errors; we would identify `base`
// for the numbers e, 2e, 3e; but if we add 1 to all the numbers, even
// though `base=e` would be just as useful in either case.
// As a result, we can think of the "loss" of an error from a multiple of base
// as O(ln|error|).
//
// I (Martin) thought about using an FFT here, but I'm not sure how to pull it
// off computationally efficiently when the frequency of interest could be in
// such a large range and must be determined so precisely.
// So instead we use an approximate Euclidean algorithm on pairs of floats.

const REQUIRED_PRECISION_BITS: Bitlen = 6;
const SNAP_THRESHOLD_ABSOLUTE: f64 = 0.02;
const SNAP_THRESHOLD_DECIMAL_RELATIVE: f64 = 0.01;
const INTERESTING_TRAILING_ZEROS: u32 = 5;
const REQUIRED_TRAILING_ZEROS_FREQUENCY: f64 = 0.5;
const REQUIRED_GCD_PAIR_FREQUENCY: f64 = 0.001;

fn insignificant_float_to<F: Float>(x: F) -> F {
  let spare_precision_bits = F::PRECISION_BITS.saturating_sub(REQUIRED_PRECISION_BITS) as i32;
  x * F::exp2(-spare_precision_bits)
}

fn is_approx_zero<F: Float>(small: F, big: F) -> bool {
  small <= insignificant_float_to(big)
}

fn is_small_remainder<F: Float>(remainder: F, original: F) -> bool {
  remainder <= original * F::exp2(-16)
}

fn is_imprecise<F: Float>(value: F, err: F) -> bool {
  value <= err * F::exp2(REQUIRED_PRECISION_BITS as i32)
}

fn approx_pair_gcd<F: Float>(greater: F, lesser: F) -> Option<F> {
  if is_approx_zero(lesser, greater) || lesser == greater {
    return None;
  }

  #[derive(Clone, Copy, Debug)]
  struct PairMult<F: Float> {
    value: F,
    err: F,
  }

  let machine_eps = F::exp2(-(F::PRECISION_BITS as i32));
  let rem_assign = |lhs: &mut PairMult<F>, rhs: &PairMult<F>| {
    let ratio = (lhs.value / rhs.value).round();
    lhs.err += ratio * rhs.err + lhs.value * machine_eps;
    lhs.value = (lhs.value - ratio * rhs.value).abs();
  };

  let mut p_greater = PairMult {
    value: greater,
    err: F::ZERO,
  };
  let mut p_lesser = PairMult {
    value: lesser,
    err: F::ZERO,
  };

  loop {
    let prev = p_greater.value;
    rem_assign(&mut p_greater, &p_lesser);
    if is_small_remainder(p_greater.value, prev) || p_greater.value <= p_greater.err {
      return Some(p_lesser.value);
    }

    if is_approx_zero(p_greater.value, greater) || is_imprecise(p_greater.value, p_greater.err) {
      return None;
    }

    mem::swap(&mut p_greater, &mut p_lesser);
  }
}

#[inline(never)]
fn choose_config_by_trailing_zeros<F: Float>(sample: &[F]) -> Option<FloatMultConfig<F>> {
  let precision_bits = F::PRECISION_BITS;
  let calc_power_of_2_divisor =
    |exponent, trailing_zeros| exponent - precision_bits.saturating_sub(trailing_zeros) as i32;

  // the greatest k such that 2^k divides all the floats exactly
  let mut k = i32::MAX;
  let mut count = 0;
  for x in sample {
    let trailing_zeros = x.trailing_zeros();
    if *x != F::ZERO && trailing_zeros >= INTERESTING_TRAILING_ZEROS {
      let k_prime = calc_power_of_2_divisor(x.exponent(), trailing_zeros);
      count += 1;
      k = min(k, k_prime);
    }
  }

  let required_samples = max(
    (sample.len() as f64 * REQUIRED_TRAILING_ZEROS_FREQUENCY).ceil() as usize,
    sampling::MIN_SAMPLE,
  );
  if count < required_samples {
    return None;
  }

  let mut int_sample = Vec::new();
  let lshift = F::L::BITS - precision_bits - 1;
  let explicit_mantissa = F::L::MID;
  for x in sample {
    let exponent = x.exponent();
    // the greatest k' such that 2^k' divides this float exactly
    let k_prime = calc_power_of_2_divisor(x.exponent(), x.trailing_zeros());
    if k_prime >= k && exponent < k + F::L::BITS as i32 {
      let rshift = F::L::BITS - 1 - (exponent - k) as u32;
      let lshifted_w_explicit_mantissa = (x.to_latent_bits() << lshift) | explicit_mantissa;
      let multiple_of_k = lshifted_w_explicit_mantissa >> rshift;
      int_sample.push(multiple_of_k);
    }
  }

  if int_sample.len() >= required_samples {
    let int_base = int_mult::choose_candidate_base(&mut int_sample)
      .map(|(base, _)| base)
      .unwrap_or(F::L::ONE);
    let base = F::from_latent_numerical(int_base) * F::exp2(k);
    Some(FloatMultConfig::from_base(base))
  } else {
    None
  }
}

#[inline(never)]
fn approx_sample_gcd_euclidean<F: Float>(sample: &[F]) -> Option<F> {
  let mut gcds = Vec::new();
  for i in (0..sample.len() - 1).step_by(2) {
    let a = sample[i];
    let b = sample[i + 1];
    if let Some(gcd) = approx_pair_gcd(F::max(a, b), F::min(a, b)) {
      gcds.push(gcd);
    }
  }

  let required_pairs_with_common_gcd =
    1 + (sample.len() as f64 * REQUIRED_GCD_PAIR_FREQUENCY).ceil() as usize;
  if gcds.len() < required_pairs_with_common_gcd {
    return None;
  }

  // safe because we filtered out poorly-behaved floats
  gcds.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());
  // we check a few GCDs in the middle and see if they show up frequently enough
  for percentile in [0.1, 0.3, 0.5] {
    let candidate = gcds[(percentile * gcds.len() as f64) as usize];
    let similar_gcd_count = gcds
      .iter()
      .filter(|&&gcd| (gcd - candidate).abs() < F::from_f64(0.01) * candidate)
      .count();

    if similar_gcd_count >= required_pairs_with_common_gcd {
      return Some(candidate);
    }
  }

  None
}

fn choose_config_by_euclidean<F: Float>(sample: &[F]) -> Option<FloatMultConfig<F>> {
  let base = approx_sample_gcd_euclidean(sample)?;
  let base = center_sample_base(base, sample);
  let config = snap_to_int_reciprocal(base);
  Some(config)
}

#[inline(never)]
fn center_sample_base<F: Float>(base: F, sample: &[F]) -> F {
  // Go back through the sample, holding all mults fixed, and adjust the gcd to
  // minimize the average deviation from mult * gcd, weighting by mult.
  // Ideally we would tweak by something between the weighted median and mode
  // of the individual tweaks, since we model loss as proportional to
  // sum[log|error|], but doing so would be computationally harder.
  let inv_base = base.inv();
  let mut tweak_sum = F::ZERO;
  let mut tweak_weight = F::ZERO;
  for &x in sample {
    let mult = (x * inv_base).round();
    let mult_exponent = mult.exponent() as Bitlen;
    if mult_exponent < F::PRECISION_BITS && mult != F::ZERO {
      let overshoot = (mult * base) - x;
      let weight = F::from_f64((F::PRECISION_BITS - mult_exponent) as f64);
      tweak_sum += weight * (overshoot / mult);
      tweak_weight += weight;
    }
  }
  base - tweak_sum / tweak_weight
}

fn snap_to_int_reciprocal<F: Float>(base: F) -> FloatMultConfig<F> {
  let inv_base = base.inv();
  let round_inv_base = inv_base.round();
  let decimal_inv_base = F::from_f64(10.0_f64.powf(inv_base.to_f64().log10().round()));
  // check if relative error is below a threshold
  if (inv_base - round_inv_base).abs() < F::from_f64(SNAP_THRESHOLD_ABSOLUTE) {
    FloatMultConfig::from_inv_base(round_inv_base)
  } else if (inv_base - decimal_inv_base).abs() / inv_base
    < F::from_f64(SNAP_THRESHOLD_DECIMAL_RELATIVE)
  {
    FloatMultConfig::from_inv_base(decimal_inv_base)
  } else {
    FloatMultConfig::from_base(base)
  }
}

fn bits_saved_per_num_over_classic<F: Float>(
  config: FloatMultConfig<F>,
  sample: &[F],
) -> Option<f64> {
  let bits_saved_per_num = sampling::est_bits_saved_per_num(sample, |x| {
    let mult = (x * config.inv_base).round();
    let primary = mult.int_float_to_latent();
    // We treat a mult of 0 as if there are only PRECISION_BITS bits between it and 1,
    // which is not true (there are actually BITS - 2), but this is more useful
    // for estimating bit savings.
    let inter_base_bits = F::PRECISION_BITS.saturating_sub(mult.exponent() as Bitlen);
    let approx_unsigned = (mult * config.base).to_latent_ordered();
    let x_as_unsigned = x.to_latent_ordered();
    let abs_adj = max(x_as_unsigned, approx_unsigned) - min(x_as_unsigned, approx_unsigned);
    // Estimating bit cost of adjustments, we conservatively assume the
    // probability distribution is
    //
    // P(adj) = 1/(2 * (adj^2 + 1)),
    //   -2^k < adj < 2^k where k = inter_base_bits - 1
    //
    // So adj should cost the log_2 of this, or approximately
    //
    // B(adj) = 1 + 2 * log_2|adj|
    //
    // We relax this slightly for 0 and let it slide.
    let adj_bits = 1 + 2 * (F::L::BITS - abs_adj.leading_zeros());
    PrimaryLatentAndSavings {
      primary,
      bits_saved: inter_base_bits as f64 - adj_bits as f64,
    }
  });

  if bits_saved_per_num >= MULT_REQUIRED_BITS_SAVED_PER_NUM {
    Some(bits_saved_per_num)
  } else {
    None
  }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct FloatMultConfig<F: Float> {
  pub base: F,
  pub inv_base: F,
}

impl<F: Float> FloatMultConfig<F> {
  fn from_base(base: F) -> Self {
    Self {
      base,
      inv_base: base.inv(),
    }
  }

  fn from_inv_base(inv_base: F) -> Self {
    Self {
      base: inv_base.inv(),
      inv_base,
    }
  }
}

fn choose_config<F: Float>(sample: &[F]) -> Option<FloatMultConfig<F>> {
  choose_config_by_trailing_zeros(sample).or_else(|| choose_config_by_euclidean(sample))
}

pub(crate) fn compute_bid<F: Float>(sample: &[F]) -> Option<Bid<F>> {
  choose_config(sample).and_then(|config| {
    let bits_saved_per_num = bits_saved_per_num_over_classic(config, sample)?;
    Some(Bid {
      mode: Mode::float_mult(config.base),
      bits_saved_per_num,
      split_fn: Box::new(move |nums| split_latents(nums, config)),
    })
  })
}
