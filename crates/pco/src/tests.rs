//! One model-based round trip: random numbers of every type through every mode, delta
//! encoding and paging, decoded the ways vecdb (wrapped pages in any order) and statedb
//! (standalone into a slice) do, each compared bit for bit with the input itself.

use std::{any::type_name, collections::BTreeSet};

use half::f16;

use crate::{
  constants::Bitlen,
  data_types::{latent_priv::LatentPriv, Number},
  errors::PcoResult,
  metadata::{DeltaEncoding, Mode},
  standalone, wrapped, ChunkConfig, DeltaSpec, ModeSpec, PagingSpec, FULL_BATCH_N,
};

/// xorshift64*: deterministic and dependency free.
struct Rng(u64);

impl Rng {
  fn next(&mut self) -> u64 {
    self.0 ^= self.0 << 13;
    self.0 ^= self.0 >> 7;
    self.0 ^= self.0 << 17;
    self.0.wrapping_mul(0x2545_f491_4f6c_dd1d)
  }

  fn below(&mut self, n: usize) -> usize {
    (self.next() % n as u64) as usize
  }

  fn coin(&mut self) -> bool {
    self.next() >> 63 == 1
  }
}

trait Sample: Number {
  /// Mantissa bits, the valid `FloatQuant` range; 0 for integers.
  const PRECISION: Bitlen;
  fn from_f64(x: f64) -> Self;
}

macro_rules! sample {
  ($($t:ty = $precision:literal),*) => {$(
    impl Sample for $t {
      const PRECISION: Bitlen = $precision;
      fn from_f64(x: f64) -> Self {
        x as Self
      }
    }
  )*};
}

sample!(
  u8 = 0,
  u16 = 0,
  u32 = 0,
  u64 = 0,
  i8 = 0,
  i16 = 0,
  i32 = 0,
  i64 = 0,
  f32 = 23,
  f64 = 52
);

impl Sample for f16 {
  const PRECISION: Bitlen = 10;
  fn from_f64(x: f64) -> Self {
    f16::from_f64(x)
  }
}

/// Bitwise equality, so NaNs and signed zeros count.
fn check<T: Number>(desc: &str, got: &[T], want: &[T]) {
  assert_eq!(got.len(), want.len(), "{desc}");
  let latents = |x: &T| x.to_latent_ordered();
  if let Some(i) = (0..got.len()).find(|&i| latents(&got[i]) != latents(&want[i])) {
    panic!(
      "{desc}: at {i} got {:?}, want {:?}",
      got[i], want[i]
    );
  }
}

fn case<T: Sample>(i: usize, rng: &mut Rng, seen: &mut BTreeSet<&'static str>) {
  // the rare long chunk has pages past lookback's largest window (2^15)
  let n = match rng.below(64) {
    0..=15 => rng.below(17),
    16..=47 => rng.below(1100),
    48..=62 => rng.below(5000),
    _ => 40_000 + rng.below(30_000),
  };
  let is_float = T::PRECISION > 0;
  let base = if is_float {
    [0.1, 0.25, 1.0 / 3.0]
  } else {
    [3.0, 10.0, 1000.0]
  }[rng.below(3)];
  let bits = 1 + rng.below(if is_float { T::PRECISION as usize } else { 7 }) as Bitlen;
  let period = 2 + rng.below(63);
  let noise = |rng: &mut Rng| T::from_latent_ordered(T::L::from_u64(rng.next()));
  let pool: Vec<T> = (0..1 << rng.below(5)).map(|_| noise(rng)).collect();
  let (zero, one, mid, max) = (T::L::ZERO, T::L::ONE, T::L::MID, T::L::MAX);
  let extremes = [
    zero,
    one,
    mid.wrapping_sub(one),
    mid,
    mid.wrapping_add(one),
    max.wrapping_sub(one),
    max,
  ]
  .map(T::from_latent_ordered)
  .into_iter()
  .chain(
    [
      f64::INFINITY,
      f64::NEG_INFINITY,
      f64::MAX,
      f64::MIN,
      f64::NAN,
    ]
    .map(T::from_f64),
  )
  .collect::<Vec<_>>();

  let shape = rng.below(7);
  let mut walk = rng.below(1000) as f64;
  let mut nums = Vec::with_capacity(n);
  for j in 0..n {
    walk += rng.below(201) as f64 - 100.0;
    nums.push(match shape {
      0 => noise(rng),
      1 => extremes[rng.below(extremes.len())],
      2 => T::from_f64(walk),
      3 if j >= period && rng.below(8) > 0 => nums[j - period],
      3 => T::from_f64(walk),
      4 => T::from_f64(rng.below(1000) as f64 * base + (rng.below(4) == 0) as u8 as f64),
      5 => T::from_latent_ordered((T::from_f64(walk * 0.37).to_latent_ordered() >> bits) << bits),
      _ => pool[rng.below(pool.len())],
    });
  }

  let mode = match rng.below(4) {
    0 => ModeSpec::Auto,
    1 => ModeSpec::Classic,
    2 => ModeSpec::TryDict,
    _ if !is_float => ModeSpec::TryIntMult(base as u64),
    _ if rng.coin() => ModeSpec::TryFloatMult(base),
    _ => ModeSpec::TryFloatQuant(bits),
  };
  let delta = match rng.below(5) {
    0 => DeltaSpec::Auto,
    1 => DeltaSpec::NoOp,
    2 => DeltaSpec::TryConsecutive(rng.below(8)),
    3 => DeltaSpec::TryLookback,
    // conv1 only exists for latents of up to 32 bits
    _ if T::L::BITS > 32 => DeltaSpec::TryConsecutive(1),
    _ => DeltaSpec::TryConv1(if rng.coin() { 6 } else { rng.below(33) }),
  };
  let paging = if rng.coin() {
    PagingSpec::EqualPagesUpTo(n.div_ceil(1 + rng.below(8)).max(1))
  } else {
    let (mut parts, mut rest) = (Vec::new(), n);
    while rest > 0 {
      parts.push(1 + rng.below(rest));
      rest -= parts[parts.len() - 1];
    }
    PagingSpec::Exact(parts)
  };
  let level = rng.below(13);
  let desc = format!(
    "case {i}: {} n={n} shape={shape} level={level} {mode:?} {delta:?} {paging:?}",
    type_name::<T>()
  );
  let config = ChunkConfig::default()
    .with_compression_level(level)
    .with_mode_spec(mode)
    .with_delta_spec(delta)
    .with_paging_spec(paging)
    .with_enable_8_bit(true);

  let mut run = || -> PcoResult<()> {
    // vecdb: header and chunk meta in one blob, then each page decoded alone from its bytes
    let file = wrapped::FileCompressor::default();
    if n == 0 {
      assert!(
        file.chunk_compressor(&nums, &config).is_err(),
        "{desc}"
      );
    } else {
      let mut bytes = file.write_header(Vec::new())?;
      let mut chunk = file.chunk_compressor(&nums, &config)?;
      let meta = chunk.meta();
      seen.insert(match meta.mode {
        Mode::Classic => "Classic",
        Mode::IntMult(_) => "IntMult",
        Mode::FloatMult(_) => "FloatMult",
        Mode::FloatQuant(_) => "FloatQuant",
        Mode::Dict(_) => "Dict",
      });
      seen.insert(match meta.delta_encoding {
        DeltaEncoding::NoOp => "NoOp",
        DeltaEncoding::Consecutive { .. } => "Consecutive",
        DeltaEncoding::Lookback { .. } => "Lookback",
        DeltaEncoding::Conv1(_) => "Conv1",
      });
      bytes = chunk.write_meta(bytes)?;
      let meta_end = bytes.len();
      let mut pages = Vec::new();
      let mut start = 0;
      for (p, page_n) in chunk.n_per_page().into_iter().enumerate() {
        let from = bytes.len();
        bytes = chunk.write_page(p, bytes)?;
        pages.push((start..start + page_n, from..bytes.len()));
        start += page_n;
      }
      assert_eq!(start, n, "{desc}");

      let (file, rest) = wrapped::FileDecompressor::new(&bytes[..meta_end])?;
      let (mut decoder, rest) = file.chunk_decompressor::<T, _>(rest)?;
      assert!(rest.is_empty(), "{desc}");
      // one decoder serves every page, in any order and again
      for p in (1..pages.len()).rev() {
        pages.swap(p, rng.below(p + 1));
      }
      pages.push(pages[rng.below(pages.len())].clone());
      for (values, body) in pages {
        let body = if rng.coin() {
          &bytes[body]
        } else {
          &bytes[body.start..]
        };
        let page_n = values.len();
        let mut page = decoder.page_decompressor(body, page_n)?;
        if rng.coin() {
          // vecdb: the whole page into spare capacity
          let mut out = Vec::with_capacity(page_n);
          let progress = page.read_uninit(&mut out.spare_capacity_mut()[..page_n])?;
          assert!(
            progress.n_processed == page_n && progress.finished,
            "{desc}"
          );
          // SAFETY: the read initialized all page_n values.
          unsafe { out.set_len(page_n) };
          check(&desc, &out, &nums[values]);
        } else {
          // whole batches or everything left, into an exact or longer destination
          let extra = if rng.coin() { 0 } else { 1 + rng.below(300) };
          let mut out = vec![T::default(); page_n + extra];
          let mut done = 0;
          while done < page_n {
            let left = out.len() - done;
            let step = if rng.coin() {
              left
            } else {
              (FULL_BATCH_N << rng.below(2)).min(left)
            };
            let progress = page.read(&mut out[done..done + step])?;
            let expected = step.min(page_n - done);
            assert_eq!(progress.n_processed, expected, "{desc}");
            done += expected;
            assert_eq!(progress.finished, done == page_n, "{desc}");
          }
          check(&desc, &out[..page_n], &nums[values]);
          check(
            &desc,
            &out[page_n..],
            &vec![T::default(); extra],
          );
        }
      }
    }

    // statedb: standalone (each page its own chunk) into an exact slice; shorter ones get a
    // prefix, longer ones keep their tail
    let bytes = if rng.coin() {
      standalone::simple_compress(&nums, &config)?
    } else {
      standalone::simple_compress_into(&nums, &config, Vec::new())?
    };
    check(
      &desc,
      &standalone::simple_decompress::<T>(&bytes)?,
      &nums,
    );
    let len = match rng.below(3) {
      0 => n,
      1 => n + 1 + rng.below(300),
      _ => rng.below(n + 1),
    };
    let mut out = vec![T::default(); len];
    let progress = standalone::simple_decompress_into(&bytes, &mut out)?;
    let k = len.min(n);
    assert_eq!(
      (progress.n_processed, progress.finished),
      (k, len >= n),
      "{desc}"
    );
    check(&desc, &out[..k], &nums[..k]);
    check(
      &desc,
      &out[k..],
      &vec![T::default(); len - k],
    );
    Ok(())
  };
  run().unwrap_or_else(|e| panic!("{desc}: {e}"));
}

#[test]
fn round_trips_match_the_input() {
  let mut rng = Rng(0x9e37_79b9_7f4a_7c15);
  let mut seen = BTreeSet::new();
  for i in 0..10_000 {
    let rng = &mut rng;
    let seen = &mut seen;
    match rng.below(11) {
      0 => case::<u8>(i, rng, seen),
      1 => case::<u16>(i, rng, seen),
      2 => case::<u32>(i, rng, seen),
      3 => case::<u64>(i, rng, seen),
      4 => case::<i8>(i, rng, seen),
      5 => case::<i16>(i, rng, seen),
      6 => case::<i32>(i, rng, seen),
      7 => case::<i64>(i, rng, seen),
      8 => case::<f16>(i, rng, seen),
      9 => case::<f32>(i, rng, seen),
      _ => case::<f64>(i, rng, seen),
    }
  }
  // every mode and delta encoding was actually chosen at least once
  let all = [
    "Classic",
    "IntMult",
    "FloatMult",
    "FloatQuant",
    "Dict",
    "NoOp",
    "Consecutive",
    "Lookback",
    "Conv1",
  ];
  assert_eq!(seen, BTreeSet::from(all));
}
