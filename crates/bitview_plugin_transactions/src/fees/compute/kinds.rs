//! The kinds of transaction the fee pass values, a bit each in a transaction's mask.

use std::ops::Range;

use bitview_plugin_indexer::Indexer;
use bitview_primitives::{Boolean, Count, Count16};
use bitview_vecs::{CachedSeries, PerBlockCumulativeRolling};
use brk_types::{Height, Sats, TxIndex, Version};
use rayon::prelude::*;
use vecdb::{AnyStoredVec, AnyVec, PcoVec, ReadableVec};

use crate::{
    features::{FEATURES, with_flagged_features},
    flagged::TxValue,
};

/// The features with a flag, read first (bits from 0, in `with_flagged_features` order).
const FLAGGED: usize = FEATURES - 3;
pub const INSCRIPTION: usize = FLAGGED;
/// Read for nonstandard alone: valued by nothing.
pub const UNCONDITIONAL: usize = FLAGGED + 1;
/// The indexer's flags read, from bit 0.
pub const READ: usize = UNCONDITIONAL + 1;
/// Derived from what the pass reads.
pub const SEGWIT: usize = READ;
pub const ONE_INPUT: usize = SEGWIT + 1;
pub const ONE_OUTPUT: usize = ONE_INPUT + 1;
/// Versions 1, 2, 3 and the others, from here.
pub const VERSIONS: usize = ONE_OUTPUT + 1;
pub const NONSTANDARD: usize = VERSIONS + 4;
pub const CPFP_PARENT: usize = NONSTANDARD + 1;
pub const CPFP_CHILD: usize = CPFP_PARENT + 1;
pub const BITS: usize = CPFP_CHILD + 1;
const _: () = assert!(BITS <= 64);

/// The dust feature's bit, for nonstandard.
pub const DUST: usize = {
    macro_rules! position {
        (
            scripts: [$($script:ident),*],
            signatures: [$($signature:ident),*],
            others: [$($other:ident),*]
        ) => {
            [$(stringify!($script),)* $(stringify!($signature),)* $(stringify!($other),)*]
        };
    }
    let names = with_flagged_features!(position);
    let mut i = 0;
    while !eq(names[i].as_bytes(), b"dust_output") {
        i += 1;
    }
    i
};

const fn eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut i = 0;
    while i < a.len() {
        if a[i] != b[i] {
            return false;
        }
        i += 1;
    }
    true
}

/// What the fee pass values beside its own CPFP roles: the features (as `split_mut` lists them,
/// SegWit and the single input and output last), inscriptions, the versions and nonstandard;
/// with nonstandard's flag and count, and inscriptions' fees, which the same transactions give.
pub(crate) struct Valued<'a> {
    pub features: [&'a mut TxValue; FEATURES],
    pub inscription: &'a mut TxValue,
    pub versions: [&'a mut TxValue; 4],
    pub nonstandard: &'a mut TxValue,
    pub nonstandard_flag: &'a mut PcoVec<TxIndex, Boolean>,
    pub nonstandard_count: &'a mut PerBlockCumulativeRolling<Count>,
    /// Inscription transactions' fees through every block, in sats.
    pub inscription_fees: &'a mut CachedSeries<Height, Sats>,
}

impl Valued<'_> {
    /// Each value with its bit.
    pub fn values_mut(&mut self) -> impl Iterator<Item = (usize, &mut TxValue)> {
        let [features @ .., segwit, one_input, one_output] = &mut self.features;
        features
            .iter_mut()
            .enumerate()
            .map(|(bit, value)| (bit, &mut **value))
            .chain([
                (INSCRIPTION, &mut *self.inscription),
                (SEGWIT, &mut **segwit),
                (ONE_INPUT, &mut **one_input),
                (ONE_OUTPUT, &mut **one_output),
                (NONSTANDARD, &mut *self.nonstandard),
            ])
            .chain(
                self.versions
                    .iter_mut()
                    .enumerate()
                    .map(|(i, value)| (VERSIONS + i, &mut **value)),
            )
    }

    /// Its stored outputs: per transaction, then per block.
    pub fn stored_mut(&mut self) -> (&mut dyn AnyStoredVec, Vec<&mut dyn AnyStoredVec>) {
        let Self {
            features,
            inscription,
            versions,
            nonstandard,
            nonstandard_flag,
            nonstandard_count,
            inscription_fees,
        } = self;
        let mut blocks: Vec<&mut dyn AnyStoredVec> = features
            .iter_mut()
            .chain(versions.iter_mut())
            .map(|value| value.stored_mut())
            .collect();
        blocks.extend([
            inscription.stored_mut(),
            nonstandard.stored_mut(),
            nonstandard_count.stored_mut(),
            &mut **inscription_fees as &mut dyn AnyStoredVec,
        ]);
        (&mut **nonstandard_flag, blocks)
    }
}

/// An indexer flag the pass reads, with its per-block count where it has one: a block without
/// any is skipped undecoded.
struct Column<'a> {
    flags: &'a PcoVec<TxIndex, Boolean>,
    counts: Option<&'a PcoVec<Height, Count16>>,
}

/// The indexer's flags, read a batch of blocks at a time, each into its own bits.
pub struct Columns<'a>([Column<'a>; READ]);

impl<'a> Columns<'a> {
    pub fn new(indexer: &'a Indexer) -> Self {
        let source = &indexer.vecs().transactions.features;
        macro_rules! columns {
            (
                scripts: [$($script:ident),*],
                signatures: [$($signature:ident),*],
                others: [$($other:ident),*]
            ) => {
                [
                    $(Column { flags: &source.$script.flag, counts: Some(&*source.$script.block) },)*
                    $(Column { flags: &source.$signature.flag, counts: Some(&*source.$signature.block) },)*
                    $(Column { flags: &source.$other.flag, counts: Some(&*source.$other.block) },)*
                    Column { flags: &source.inscription.flag, counts: Some(&*source.inscription.block) },
                    Column { flags: &source.is_unconditionally_nonstandard, counts: None },
                ]
            };
        }
        Self(with_flagged_features!(columns))
    }

    /// Every column's version, for the pass's.
    pub fn version(&self) -> Version {
        self.0.iter().fold(Version::ZERO, |version, column| {
            version + column.flags.version() + column.counts.map_or(Version::ZERO, |c| c.version())
        })
    }

    /// Reads the flags of `heights`' blocks (`txs` each, from transaction `first_tx`) into `bits`,
    /// each column's over the batch's transactions, 64 a word; the columns in parallel.
    pub fn read(
        &self,
        heights: Range<usize>,
        txs: &[usize],
        first_tx: usize,
        bits: &mut [Vec<u64>; READ],
    ) {
        let total: usize = txs.iter().sum();
        let words = total.div_ceil(64);
        self.0.par_iter().zip(bits.par_iter_mut()).for_each_init(
            Vec::new,
            |counts, (column, bits)| {
                bits.clear();
                bits.resize(words, 0);
                if let Some(source) = column.counts {
                    source.collect_range_into_at(heights.start, heights.end, counts);
                }
                let mut flags = column.flags.range_cursor_at(first_tx, first_tx + total);
                let mut at = 0;
                for (block, &n) in txs.iter().enumerate() {
                    if column.counts.is_some() && *counts[block] == 0 {
                        flags.advance(n);
                    } else {
                        let mut i = at;
                        flags.for_each(n, |flag| {
                            bits[i / 64] |= u64::from(flag.is_true()) << (i % 64);
                            i += 1;
                        });
                    }
                    at += n;
                }
            },
        );
    }

    /// Sets each read bit of the batch's transactions from `offset` in theirs (`masks`), going by
    /// the bits set alone.
    pub fn or_into(bits: &[Vec<u64>; READ], offset: usize, masks: &mut [u64]) {
        let end = offset + masks.len();
        for (bit, column) in bits.iter().enumerate() {
            let first = offset / 64;
            for (i, &word) in column[first..end.div_ceil(64)].iter().enumerate() {
                let base = (first + i) * 64;
                let mut word = word;
                if base < offset {
                    word &= u64::MAX << (offset - base);
                }
                if base + 64 > end {
                    word &= u64::MAX >> (base + 64 - end);
                }
                while word != 0 {
                    masks[base + word.trailing_zeros() as usize - offset] |= 1 << bit;
                    word &= word - 1;
                }
            }
        }
    }
}
