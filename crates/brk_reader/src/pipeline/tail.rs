use std::{fs::File, os::unix::fs::FileExt, sync::Arc};

use bitcoin::block::Header;
use brk_error::{Error, Result};
use brk_types::{BlkMetadata, BlkPosition, BlockHash, Height, ReadBlock};
use crossbeam::channel::Sender;

use crate::{
    BlkIndexToBlkPath, OUT_OF_ORDER_FILE_BACKOFF, ReaderInner, XORIndex, bisect,
    canonical::CanonicalRange,
    parse::{HEADER_LEN, parse_canonical_body, peek_canonical},
    reader_inner::AppendPosition,
    scan::frames,
};

/// A canonical block located by its header: bodies are read only once every
/// slot is found, in canonical order, so at most one decoded block is held.
pub struct Found {
    file: Arc<File>,
    blk_index: u16,
    offset: usize,
    len: usize,
    header: Header,
}

/// The canonical range's blocks, as their frames are found in any order.
struct Slots<'a> {
    canonical: &'a CanonicalRange,
    slots: Vec<Option<Found>>,
    remaining: usize,
}

impl<'a> Slots<'a> {
    fn new(canonical: &'a CanonicalRange) -> Self {
        Self {
            canonical,
            slots: (0..canonical.len()).map(|_| None).collect(),
            remaining: canonical.len(),
        }
    }

    /// The height of the lowest block not found yet.
    fn lowest_missing(&self) -> Option<Height> {
        let missing = self.slots.iter().position(Option::is_none)?;
        Some(Height::from(*self.canonical.start + missing as u32))
    }

    /// Fills a slot when the frame at `offset` holds a missing canonical block, its header read
    /// before its body. True once every slot is filled.
    fn offer(
        &mut self,
        reader: &ReaderInner,
        file: &Arc<File>,
        blk_index: u16,
        (offset, len): (usize, usize),
    ) -> Result<bool> {
        if len < HEADER_LEN {
            return Ok(false);
        }
        let mut header_bytes = [0u8; HEADER_LEN];
        file.read_exact_at(&mut header_bytes, offset as u64)?;
        let Some((slot, header)) = peek_canonical(
            &header_bytes,
            XORIndex::at_offset(offset),
            reader.xor_bytes,
            self.canonical,
        ) else {
            return Ok(false);
        };
        if self.slots[slot as usize].is_some() {
            return Ok(false);
        }
        if !self
            .canonical
            .verify_prev(slot, &BlockHash::from(header.prev_blockhash))
        {
            return Err(Error::Internal(
                "tail pipeline: canonical batch stitched across a reorg",
            ));
        }
        self.slots[slot as usize] = Some(Found {
            file: file.clone(),
            blk_index,
            offset,
            len,
            header,
        });
        self.remaining -= 1;
        Ok(self.remaining == 0)
    }

    fn into_found(self) -> Option<Vec<Found>> {
        (self.remaining == 0).then(|| self.slots.into_iter().flatten().collect())
    }
}

/// The canonical blocks appended after the last block streamed, in the same file: an update
/// right after the previous one reads only their frames. `None` on any discontinuity (another
/// anchor, a rollover, a gap, a missing block), for the full tail walk to resolve.
pub fn find_appended(
    reader: &ReaderInner,
    canonical: &CanonicalRange,
) -> Result<Option<Vec<Found>>> {
    let Some(position) = reader.append_position() else {
        return Ok(None);
    };
    if canonical.anchor() != Some(&position.hash) {
        return Ok(None);
    }
    let Ok(file) = reader.open_blk(position.blk_index) else {
        return Ok(None);
    };
    let mut slots = Slots::new(canonical);
    for frame in frames(&file, reader.xor_bytes, position.end)? {
        if slots.offer(reader, &file, position.blk_index, frame)? {
            break;
        }
    }
    Ok(slots.into_found())
}

/// `probed`: first-block heights of the newest files, newest first, already
/// known from picking this strategy.
pub fn pipeline_tail(
    reader: &ReaderInner,
    paths: &BlkIndexToBlkPath,
    probed: &[Option<Height>],
    canonical: &CanonicalRange,
    send: &Sender<Result<ReadBlock>>,
) -> Result<()> {
    let mut slots = Slots::new(canonical);
    let mut below_floor_streak: usize = 0;

    'files: for (i, (&blk_index, path)) in paths.iter().rev().enumerate() {
        if let Some(lowest_missing) = slots.lowest_missing()
            && let Some(first_height) = match probed.get(i) {
                Some(&height) => height,
                None => bisect::first_block_height(&reader.client, path, reader.xor_bytes).ok(),
            }
        {
            if first_height < lowest_missing {
                below_floor_streak += 1;
                if below_floor_streak >= OUT_OF_ORDER_FILE_BACKOFF {
                    return Err(Error::Internal(
                        "tail pipeline: walked past the canonical window without finding all blocks",
                    ));
                }
            } else {
                below_floor_streak = 0;
            }
        }

        let file = reader.open_blk(blk_index)?;
        // Newest blocks first.
        for frame in frames(&file, reader.xor_bytes, 0)?.into_iter().rev() {
            if slots.offer(reader, &file, blk_index, frame)? {
                break 'files;
            }
        }
    }

    let found = slots.into_found().ok_or(Error::Internal(
        "tail pipeline: blk files missing canonical blocks",
    ))?;
    emit(reader, found, canonical, send)
}

/// Reads, parses and sends `found` in canonical order, then retains where the last block's
/// frame ends for the next call's [`find_appended`].
pub fn emit(
    reader: &ReaderInner,
    found: Vec<Found>,
    canonical: &CanonicalRange,
    send: &Sender<Result<ReadBlock>>,
) -> Result<()> {
    let mut last = None;
    for (slot, found) in found.into_iter().enumerate() {
        let mut block_bytes = vec![0u8; found.len];
        found
            .file
            .read_exact_at(&mut block_bytes, found.offset as u64)?;
        let block = parse_canonical_body(
            block_bytes,
            BlkMetadata::new(BlkPosition::new(found.blk_index, found.offset as u32)),
            XORIndex::at_offset(found.offset),
            reader.xor_bytes,
            Height::from(*canonical.start + slot as u32),
            found.header,
        )?;
        last = Some(AppendPosition {
            hash: *block.hash(),
            blk_index: found.blk_index,
            end: found.offset + found.len,
        });
        if send.send(Ok(block)).is_err() {
            return Ok(());
        }
    }
    if let Some(last) = last {
        reader.retain_append_position(last);
    }
    Ok(())
}
