use std::{fs::File, os::unix::fs::FileExt};

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

pub fn pipeline_tail(
    client: &Client,
    paths: &BlkIndexToBlkPath,
    xor_bytes: XORBytes,
    canonical: &CanonicalRange,
    send: &Sender<Result<ReadBlock>>,
) -> Result<()> {
    let mut slots: Vec<Option<ReadBlock>> = (0..canonical.len()).map(|_| None).collect();
    let mut remaining = canonical.len();
    let mut below_floor_streak: usize = 0;

    'files: for (&blk_index, path) in paths.iter().rev() {
        if let Some(missing_idx) = slots.iter().position(Option::is_none)
            && let Ok(first_height) = bisect::first_block_height(client, path, xor_bytes)
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
            let xor_state = XORIndex::at_offset(offset);
            let Some((slot, header)) =
                peek_canonical(&header_bytes, xor_state, xor_bytes, canonical)
            else {
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
            let mut block_bytes = vec![0u8; len];
            file.read_exact_at(&mut block_bytes, offset as u64)?;
            let metadata = BlkMetadata::new(BlkPosition::new(blk_index, offset as u32));
            let height = Height::from(*canonical.start + slot);
            slots[slot as usize] = Some(parse_canonical_body(
                block_bytes,
                metadata,
                xor_state,
                xor_bytes,
                height,
                header,
            )?);
            remaining -= 1;
            if remaining == 0 {
                break 'files;
            }
        }
    }

    if remaining > 0 {
        return Err(Error::Internal(
            "tail pipeline: blk files missing canonical blocks",
        ));
    }

    for slot in slots {
        let block = slot.expect("tail pipeline left a slot empty after `remaining == 0`");
        if send.send(Ok(block)).is_err() {
            return Ok(());
        }
    }
    Ok(())
}
