use std::cmp::min;
use std::fmt::Debug;

use half::f16;

use crate::data_types::Number;
use crate::errors::PcoResult;
use crate::standalone::{self, DecompressorItem};
use crate::wrapped::{FileCompressor, FileDecompressor, PageDecompressor};
use crate::{ChunkConfig, ModeSpec, PagingSpec, FULL_BATCH_N};

fn read_page_uninit<T: Number, R: better_io::BetterBufRead>(
  page: &mut PageDecompressor<T, R>,
  len: usize,
) -> PcoResult<Vec<T>> {
  let mut values = Vec::with_capacity(len);
  while values.len() < len {
    let batch_len = min(FULL_BATCH_N, len - values.len());
    let progress = page.read_uninit(&mut values.spare_capacity_mut()[..batch_len])?;
    assert_eq!(progress.n_processed, batch_len);
    // SAFETY: read_uninit initialized progress.n_processed elements.
    unsafe { values.set_len(values.len() + progress.n_processed) };
  }
  Ok(values)
}

fn wrapped_round_trip<T>(values: &[T], config: ChunkConfig) -> PcoResult<()>
where
  T: Number + Debug + PartialEq,
{
  let config = config.with_paging_spec(PagingSpec::EqualPagesUpTo(values.len()));
  let file = FileCompressor::default();
  let mut bytes = file.write_header(Vec::new())?;
  let mut chunk = file.chunk_compressor(values, &config)?;
  bytes = chunk.write_meta(bytes)?;
  bytes = chunk.write_page(0, bytes)?;

  let (file, src) = FileDecompressor::new(bytes.as_slice())?;
  let (mut chunk, src) = file.chunk_decompressor::<T, _>(src)?;
  let mut page = chunk.page_decompressor(src, values.len())?;
  let decoded = read_page_uninit(&mut page, values.len())?;
  assert_eq!(decoded, values);
  Ok(())
}

#[test]
fn wrapped_uninit_supports_every_number_type() -> PcoResult<()> {
  macro_rules! check_integer {
    ($type:ty) => {{
      let values = (0..1025)
        .map(|i| (i as $type).wrapping_mul(17).wrapping_add(3))
        .collect::<Vec<_>>();
      wrapped_round_trip(
        &values,
        ChunkConfig::default()
          .with_mode_spec(ModeSpec::Classic)
          .with_enable_8_bit(true),
      )?;
    }};
  }

  check_integer!(u8);
  check_integer!(u16);
  check_integer!(u32);
  check_integer!(u64);
  check_integer!(i8);
  check_integer!(i16);
  check_integer!(i32);
  check_integer!(i64);

  let f16_values = (0..1025)
    .map(|i| f16::from_f32(i as f32 * 0.25 - 100.0))
    .collect::<Vec<_>>();
  wrapped_round_trip(
    &f16_values,
    ChunkConfig::default().with_mode_spec(ModeSpec::Classic),
  )?;

  let f32_values = (0..1025)
    .map(|i| i as f32 * 0.25 - 100.0)
    .collect::<Vec<_>>();
  wrapped_round_trip(
    &f32_values,
    ChunkConfig::default().with_mode_spec(ModeSpec::Classic),
  )?;

  let f64_values = (0..1025)
    .map(|i| i as f64 * 0.25 - 100.0)
    .collect::<Vec<_>>();
  wrapped_round_trip(
    &f64_values,
    ChunkConfig::default().with_mode_spec(ModeSpec::Classic),
  )
}

#[test]
fn wrapped_uninit_supports_every_mode() -> PcoResult<()> {
  let ints = (0..4097)
    .map(|i| ((i % 97) * 1_000_000 + i % 13) as u64)
    .collect::<Vec<_>>();
  wrapped_round_trip(
    &ints,
    ChunkConfig::default().with_mode_spec(ModeSpec::TryIntMult(1_000_000)),
  )?;

  let dict = (0..4097)
    .map(|i| [7_u64, 11, 10_000_000_000, u64::MAX][i % 4])
    .collect::<Vec<_>>();
  wrapped_round_trip(
    &dict,
    ChunkConfig::default().with_mode_spec(ModeSpec::TryDict),
  )?;

  let floats = (0..4097)
    .map(|i| (i as f64 - 2048.0) * 0.25)
    .collect::<Vec<_>>();
  wrapped_round_trip(
    &floats,
    ChunkConfig::default().with_mode_spec(ModeSpec::TryFloatMult(0.25)),
  )?;
  wrapped_round_trip(
    &floats,
    ChunkConfig::default().with_mode_spec(ModeSpec::TryFloatQuant(20)),
  )
}

#[test]
fn standalone_uninit_supports_batched_reads() -> PcoResult<()> {
  let expected = (0..1025).map(|i| i * i).collect::<Vec<u64>>();
  let compressed = standalone::simple_compress(&expected, &ChunkConfig::default())?;
  let (file, src) = standalone::FileDecompressor::new(compressed.as_slice())?;
  let DecompressorItem::Chunk(mut chunk) = file.chunk_decompressor::<u64, _>(src)? else {
    panic!("expected a compressed chunk");
  };

  let mut actual = Vec::with_capacity(expected.len());
  while actual.len() < expected.len() {
    let batch_len = min(FULL_BATCH_N, expected.len() - actual.len());
    let progress = chunk.read_uninit(&mut actual.spare_capacity_mut()[..batch_len])?;
    // SAFETY: read_uninit initialized progress.n_processed elements.
    unsafe { actual.set_len(actual.len() + progress.n_processed) };
  }
  assert_eq!(actual, expected);
  Ok(())
}
