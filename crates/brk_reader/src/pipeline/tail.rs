use std::{fs::File, os::unix::fs::FileExt};

use bitcoin::block::Header;
use brk_error::{Error, Result};
use brk_rpc::Client;
use brk_types::{BlkMetadata, BlkPosition, BlockHash, Height, ReadBlock};
use crossbeam::channel::Sender;

use crate::{
    BlkIndexToBlkPath, OUT_OF_ORDER_FILE_BACKOFF, XORBytes, XORIndex, bisect,
    canonical::CanonicalRange,
    parse::{HEADER_LEN, parse_canonical_body, peek_canonical},
    scan::frames,
};

/// A canonical block located by its header: bodies are read only once every
/// slot is found, in canonical order, so at most one decoded block is held.
struct Found {
    file: usize,
    blk_index: u16,
    offset: usize,
    len: usize,
    header: Header,
}

/// `probed`: first-block heights of the newest files, newest first, already
/// known from picking this strategy.
pub fn pipeline_tail(
    client: &Client,
    paths: &BlkIndexToBlkPath,
    probed: &[Option<Height>],
    xor_bytes: XORBytes,
    canonical: &CanonicalRange,
    send: &Sender<Result<ReadBlock>>,
) -> Result<()> {
    let mut slots: Vec<Option<Found>> = (0..canonical.len()).map(|_| None).collect();
    let mut files: Vec<File> = Vec::new();
    let mut remaining = canonical.len();
    let mut below_floor_streak: usize = 0;

    'files: for (i, (&blk_index, path)) in paths.iter().rev().enumerate() {
        if let Some(missing_idx) = slots.iter().position(Option::is_none)
            && let Some(first_height) = match probed.get(i) {
                Some(&height) => height,
                None => bisect::first_block_height(client, path, xor_bytes).ok(),
            }
        {
            let lowest_missing = Height::from(*canonical.start + missing_idx as u32);
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

        let file = File::open(path)?;
        // Newest blocks first. Each candidate's header decides before its body is read.
        for (offset, len) in frames(&file, xor_bytes)?.into_iter().rev() {
            if len < HEADER_LEN {
                continue;
            }
            let mut header_bytes = [0u8; HEADER_LEN];
            file.read_exact_at(&mut header_bytes, offset as u64)?;
            let Some((slot, header)) = peek_canonical(
                &header_bytes,
                XORIndex::at_offset(offset),
                xor_bytes,
                canonical,
            ) else {
                continue;
            };
            if slots[slot as usize].is_some() {
                continue;
            }
            if !canonical.verify_prev(slot, &BlockHash::from(header.prev_blockhash)) {
                return Err(Error::Internal(
                    "tail pipeline: canonical batch stitched across a reorg",
                ));
            }
            slots[slot as usize] = Some(Found {
                file: files.len(),
                blk_index,
                offset,
                len,
                header,
            });
            remaining -= 1;
            if remaining == 0 {
                files.push(file);
                break 'files;
            }
        }
        files.push(file);
    }

    if remaining > 0 {
        return Err(Error::Internal(
            "tail pipeline: blk files missing canonical blocks",
        ));
    }

    for (slot, found) in slots.into_iter().flatten().enumerate() {
        let mut block_bytes = vec![0u8; found.len];
        files[found.file].read_exact_at(&mut block_bytes, found.offset as u64)?;
        let block = parse_canonical_body(
            block_bytes,
            BlkMetadata::new(BlkPosition::new(found.blk_index, found.offset as u32)),
            XORIndex::at_offset(found.offset),
            xor_bytes,
            Height::from(*canonical.start + slot as u32),
            found.header,
        )?;
        if send.send(Ok(block)).is_err() {
            return Ok(());
        }
    }
    Ok(())
}
