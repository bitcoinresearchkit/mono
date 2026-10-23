use std::{cmp::max, collections::HashMap};

use crate::{constants::CLASSIC_MEMORIZABLE_BINS_LOG, data_types::Latent};

pub const MIN_SAMPLE: usize = 10;
// 1 in this many nums get put into sample
const SAMPLE_RATIO: usize = 40;
// Int mults will be considered infrequent if they occur less than 1/this of
// the time.
const CLASSIC_MEMORIZABLE_BINS: f64 = (1 << CLASSIC_MEMORIZABLE_BINS_LOG) as f64;
// how many times over to try collecting samples without replacement before
// giving up
const SAMPLING_PERSISTENCE: usize = 4;

fn calc_sample_n(n: usize) -> Option<usize> {
  if n >= MIN_SAMPLE {
    Some(MIN_SAMPLE + (n - MIN_SAMPLE) / SAMPLE_RATIO)
  } else {
    None
  }
}

/// Xoroshiro128++, seeded as rand_xoshiro's `seed_from_u64` seeds it: its two state words are
/// SplitMix64's first two outputs. Sampling draws the same indices from it, so compression stays
/// byte for byte the same.
struct Xoroshiro128PlusPlus {
  s0: u64,
  s1: u64,
}

impl Xoroshiro128PlusPlus {
  fn seed_from_u64(seed: u64) -> Self {
    let mut state = seed;
    let mut splitmix = || {
      state = state.wrapping_add(0x9e3779b97f4a7c15);
      let mut z = state;
      z = (z ^ (z >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
      z = (z ^ (z >> 27)).wrapping_mul(0x94d049bb133111eb);
      z ^ (z >> 31)
    };
    Self {
      s0: splitmix(),
      s1: splitmix(),
    }
  }

  fn next_u64(&mut self) -> u64 {
    let result = self
      .s0
      .wrapping_add(self.s1)
      .rotate_left(17)
      .wrapping_add(self.s0);
    self.s1 ^= self.s0;
    self.s0 = self.s0.rotate_left(49) ^ self.s1 ^ (self.s1 << 21);
    self.s1 = self.s1.rotate_left(28);
    result
  }
}

#[inline(never)]
pub fn choose_sample<T, S, Filter: Fn(&T) -> Option<S>>(
  nums: &[T],
  filter: Filter,
) -> Option<Vec<S>> {
  // We can't modify the list, and copying it may be expensive, but we want to
  // sample a small fraction from it without replacement, so we keep a
  // bitpacked vector representing whether each one is used yet and just keep
  // resampling.
  // Maybe this is a bad idea, but it works for now.
  let target_sample_size = calc_sample_n(nums.len())?;

  let mut rng = Xoroshiro128PlusPlus::seed_from_u64(0);
  let mut visited = vec![0_u8; nums.len().div_ceil(8)];
  let mut res = Vec::with_capacity(target_sample_size);
  let mut n_iters = 0;
  while res.len() < target_sample_size && n_iters < SAMPLING_PERSISTENCE * target_sample_size {
    let rand_idx = rng.next_u64() as usize % nums.len();
    let visited_idx = rand_idx / 8;
    let visited_bit = rand_idx % 8;
    let mask = 1 << visited_bit;
    let is_visited = visited[visited_idx] & mask;
    if is_visited == 0 {
      if let Some(x) = filter(&nums[rand_idx]) {
        res.push(x);
      }
      visited[visited_idx] |= mask;
    }
    n_iters += 1;
  }

  if res.len() >= MIN_SAMPLE {
    Some(res)
  } else {
    None
  }
}

pub struct PrimaryLatentAndSavings<L: Latent> {
  pub primary: L,
  pub bits_saved: f64,
}

#[inline(never)]
pub fn est_bits_saved_per_num<L: Latent, S: Copy, F: Fn(S) -> PrimaryLatentAndSavings<L>>(
  sample: &[S],
  primary_fn: F,
) -> f64 {
  let mut primary_counts_and_savings = HashMap::<L, (usize, f64)>::with_capacity(sample.len());
  for &x in sample {
    let PrimaryLatentAndSavings {
      primary: primary_latent,
      bits_saved,
    } = primary_fn(x);
    let entry = primary_counts_and_savings
      .entry(primary_latent)
      .or_default();
    entry.0 += 1;
    entry.1 += bits_saved;
  }

  let infrequent_cutoff = max(
    1,
    (sample.len() as f64 / CLASSIC_MEMORIZABLE_BINS) as usize,
  );

  // Maybe this should be made fuzzy instead of a hard cutoff because it's just
  // a sample.
  let sample_bits_saved = primary_counts_and_savings
    .values()
    .filter(|&&(count, _)| count <= infrequent_cutoff)
    .map(|&(_, bits_saved)| bits_saved)
    .sum::<f64>();
  sample_bits_saved / sample.len() as f64
}
