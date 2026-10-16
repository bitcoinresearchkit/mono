use std::{array, cmp, mem::MaybeUninit};

use super::toggle_center_in_place;
use crate::{
  constants::{Bitlen, DeltaLookback},
  data_types::Latent,
  metadata::DeltaLookbackConfig,
  FULL_BATCH_N,
};

// there are 3 types of proposed lookbacks:
// * brute force: just try the most recent few latents
// * repeating: try the most recent lookbacks we actually used
// * hash: look up similar values by hash
const PROPOSED_LOOKBACKS: usize = 16;
const BRUTE_LOOKBACKS: usize = 6;
const REPEATING_LOOKBACKS: usize = 4;
// To help locate similar latents for lookback encoding, we hash each latent at
// different "coarsenesses" and write them into a vector. e.g. a coarseness
// of 8 means that (l >> 8) gets hashed, so we can lookup recent values by
// quotient by 256.
const COARSENESSES: [Bitlen; 2] = [0, 8];

fn hash_lookup(
  l: u64,
  i: usize,
  hash_table_n: usize,
  window_n: usize,
  idx_hash_table: &mut [usize],
  proposed_lookbacks: &mut [usize; PROPOSED_LOOKBACKS],
) {
  let hash_mask = hash_table_n - 1;
  // might be possible to improve this hash fn
  let hash_fn = |mut x: u64| {
    // constant is roughly 2**64 / phi
    x = (x ^ (x >> 32)).wrapping_mul(11400714819323197441);
    x = x ^ (x >> 32);
    x as usize & hash_mask
  };

  let mut proposal_idx = BRUTE_LOOKBACKS + REPEATING_LOOKBACKS;
  let mut offset = 0;
  for coarseness in COARSENESSES {
    let bucket = l >> coarseness;
    let buckets = [bucket.wrapping_sub(1), bucket, bucket.wrapping_add(1)];
    let hashes = buckets.map(hash_fn);
    for h in hashes {
      // SAFETY: `h = hash_fn(...) & hash_mask < hash_table_n`, and
      // `offset` is a multiple of `hash_table_n` with at most
      // `COARSENESSES.len() - 1` increments, so `offset + h < idx_hash_table.len()`.
      let lookback_to_last_instance = unsafe { i - *idx_hash_table.get_unchecked(offset + h) };
      proposed_lookbacks[proposal_idx] = if lookback_to_last_instance <= window_n {
        lookback_to_last_instance
      } else {
        proposal_idx.min(i)
      };
      proposal_idx += 1;
    }
    let h = hashes[1];
    // SAFETY: same bounds argument as the read above.
    unsafe {
      *idx_hash_table.get_unchecked_mut(offset + h) = i;
    }
    offset += hash_table_n;
  }
}

#[inline(never)]
fn find_best_lookback<L: Latent>(
  l: L,
  i: usize,
  latents: &[L],
  proposed_lookbacks: &[usize; PROPOSED_LOOKBACKS],
  lookback_counts: &mut [u32],
) -> usize {
  let mut best_goodness = 0;
  let mut best_lookback: usize = 0;
  for &lookback in proposed_lookbacks {
    // SAFETY: every proposed lookback is in `1..=min(i, window_n)`: initial
    // ones are `<= state_n <= min(i, window_n)`, brute ones
    // `i.min(PROPOSED_LOOKBACKS)` (asserted `<= window_n`), hash ones either
    // `i - j` for a table entry `j < i` (when `<= window_n`) or
    // `proposal_idx.min(i)` with `proposal_idx < PROPOSED_LOOKBACKS`, and
    // repeating ones are earlier picks among these (every proposal scores at
    // least 1, so a pick is never the initial 0). With `i < latents.len()`,
    // `lookback - 1 < min(window_n, latents.len()) = lookback_counts.len()`
    // and `i - lookback` lies in `[0, i)`.
    let (lookback_count, other) = unsafe {
      (
        *lookback_counts.get_unchecked(lookback - 1),
        *latents.get_unchecked(i - lookback),
      )
    };
    let lookback_goodness = Bitlen::BITS - lookback_count.leading_zeros();
    let delta = L::min(l.wrapping_sub(other), other.wrapping_sub(l));
    let delta_goodness = delta.leading_zeros();
    let goodness = lookback_goodness + delta_goodness;
    if goodness > best_goodness {
      best_goodness = goodness;
      best_lookback = lookback;
    }
  }
  best_lookback
}

#[inline(never)]
pub fn choose_lookbacks<L: Latent>(
  config: DeltaLookbackConfig,
  latents: &[L],
) -> Vec<DeltaLookback> {
  let state_n = config.state_n();

  if latents.len() <= state_n {
    return vec![];
  }

  let hash_table_n_log = config.window_n_log + 1;
  let hash_table_n = 1 << hash_table_n_log;
  let window_n = config.window_n();
  assert!(
    window_n >= PROPOSED_LOOKBACKS,
    "we do not support tiny windows during compression"
  );

  let mut lookback_counts = vec![1_u32; window_n.min(latents.len())];
  let mut lookbacks = Vec::with_capacity(latents.len() - state_n);
  let uninit_lookbacks = lookbacks.spare_capacity_mut();
  let mut idx_hash_table = vec![0_usize; COARSENESSES.len() * hash_table_n];
  let mut proposed_lookbacks = array::from_fn::<_, PROPOSED_LOOKBACKS, _>(|i| (i + 1).min(state_n));
  let mut best_lookback = 1;
  let mut repeating_lookback_idx: usize = 0;
  for i in state_n..latents.len() {
    let l = latents[i];

    let new_brute_lookback = i.min(PROPOSED_LOOKBACKS);
    proposed_lookbacks[new_brute_lookback - 1] = new_brute_lookback;

    hash_lookup(
      l.to_u64(),
      i,
      hash_table_n,
      window_n,
      &mut idx_hash_table,
      &mut proposed_lookbacks,
    );
    let new_best_lookback = find_best_lookback(
      l,
      i,
      latents,
      &proposed_lookbacks,
      &mut lookback_counts,
    );
    if new_best_lookback != best_lookback {
      repeating_lookback_idx += 1;
    }
    proposed_lookbacks[BRUTE_LOOKBACKS + (repeating_lookback_idx) % REPEATING_LOOKBACKS] =
      new_best_lookback;
    best_lookback = new_best_lookback;
    uninit_lookbacks[i - state_n] = MaybeUninit::new(best_lookback as DeltaLookback);
    lookback_counts[best_lookback - 1] += 1;
  }

  unsafe { lookbacks.set_len(latents.len() - state_n) };
  lookbacks
}

// All encode in place functions leave junk data (`state_n` latents in this
// case) at the front of the latents.
// Using the front instead of the back is preferable because it means we don't
// need an extra copy of the latents in this case.
#[inline(never)]
pub fn encode_in_place<L: Latent>(
  config: DeltaLookbackConfig,
  lookbacks: &[DeltaLookback],
  latents: &mut [L],
) -> Vec<L> {
  let state_n = config.state_n();
  let real_state_n = cmp::min(latents.len(), state_n);
  // TODO make this fast
  for i in (real_state_n..latents.len()).rev() {
    let lookback = lookbacks[i - state_n] as usize;
    latents[i] = latents[i].wrapping_sub(latents[i - lookback])
  }

  let mut state = vec![L::ZERO; state_n];
  state[state_n - real_state_n..].copy_from_slice(&latents[..real_state_n]);

  toggle_center_in_place(latents);

  state
}

pub fn new_window_buffer_and_pos<L: Latent>(
  config: DeltaLookbackConfig,
  state: &[L],
) -> (Vec<L>, usize) {
  let window_n = config.window_n();
  let buffer_n = cmp::max(window_n, FULL_BATCH_N) * 2;
  // TODO better default window
  let mut res = vec![L::ZERO; buffer_n];
  res[window_n - state.len()..window_n].copy_from_slice(state);
  (res, window_n)
}

// returns whether it was corrupt
pub fn decode_in_place<L: Latent>(
  config: DeltaLookbackConfig,
  lookbacks: &[DeltaLookback],
  window_buffer_pos: &mut usize,
  window_buffer: &mut [L],
  latents: &mut [L],
) -> bool {
  toggle_center_in_place(latents);

  let (window_n, state_n) = (config.window_n(), config.state_n());
  let mut start_pos = *window_buffer_pos;
  // Lookbacks can be shorter than latents in the final batch,
  // but we always decompress latents.len() numbers
  let batch_n = latents.len();
  if start_pos + batch_n > window_buffer.len() {
    // we need to cycle the buffer
    window_buffer.copy_within(start_pos - window_n..start_pos, 0);
    start_pos = window_n;
  }
  let mut has_oob_lookbacks = false;

  for (i, (&latent, &lookback)) in latents.iter().zip(lookbacks).enumerate() {
    let pos = start_pos + i;
    // Here we return whether the data is corrupt because it's
    // better than the alternatives:
    // * Taking min(lookback, window_n) or modulo is just as slow but silences
    //   the problem.
    // * Doing a checked set is slower, panics, and get doesn't catch all
    //   cases.
    let lookback = if lookback <= window_n as DeltaLookback {
      lookback as usize
    } else {
      has_oob_lookbacks = true;
      1
    };
    unsafe {
      *window_buffer.get_unchecked_mut(pos) =
        latent.wrapping_add(*window_buffer.get_unchecked(pos - lookback));
    }
  }

  let end_pos = start_pos + batch_n;
  latents.copy_from_slice(&window_buffer[start_pos - state_n..end_pos - state_n]);
  *window_buffer_pos = end_pos;

  has_oob_lookbacks
}
