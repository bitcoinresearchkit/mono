use std::{fs::File, ops::ControlFlow, os::unix::fs::FileExt};

use brk_error::Error;
use brk_rpc::Client;
use brk_types::{BlockHash, Height, ReadBlock};
use crossbeam::channel::Sender;

use crate::{
    BlkIndexToBlkPath, OUT_OF_ORDER_FILE_BACKOFF, XORBytes, bisect,
    canonical::CanonicalRange,
    parse::{parse_canonical_body, peek_canonical},
    scan::scan_bytes,
};

const TAIL_CHUNK: usize = 8 * 1024 * 1024;

pub(super) fn pipeline_tail(
    client: &Client,
    paths: &BlkIndexToBlkPath,
    xor_bytes: XORBytes,
    canonical: &CanonicalRange,
    send: &Sender<brk_error::Result<ReadBlock>>,
) -> brk_error::Result<()> {
    let mut slots: Vec<Option<ReadBlock>> = (0..canonical.len()).map(|_| None).collect();
    let mut remaining = canonical.len();
    let mut parse_failure: Option<Error> = None;
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
        let file_len = file.metadata()?.len() as usize;
        if file_len == 0 {
            continue;
        }

        let mut end = file_len;
        let mut spillover: Vec<u8> = Vec::new();

        while end > 0 && remaining > 0 {
            let start = end.saturating_sub(TAIL_CHUNK);
            let chunk_len = end - start;
            let mut buf = vec![0u8; chunk_len + spillover.len()];
            file.read_exact_at(&mut buf[..chunk_len], start as u64)?;
            buf[chunk_len..].copy_from_slice(&spillover);
            spillover.clear();

            let result = scan_bytes(
                &mut buf,
                blk_index,
                start,
                xor_bytes,
                |metadata, block_bytes, xor_state| {
                    let Some((offset, header)) =
                        peek_canonical(block_bytes, xor_state, xor_bytes, canonical)
                    else {
                        return ControlFlow::Continue(());
                    };
                    if slots[offset as usize].is_some() {
                        return ControlFlow::Continue(());
                    }
                    if !canonical.verify_prev(offset, &BlockHash::from(header.prev_blockhash)) {
                        parse_failure = Some(Error::Internal(
                            "tail pipeline: canonical batch stitched across a reorg",
                        ));
                        return ControlFlow::Break(());
                    }
                    let height = Height::from(*canonical.start + offset);
                    match parse_canonical_body(
                        block_bytes.to_vec(),
                        metadata,
                        xor_state,
                        xor_bytes,
                        height,
                        header,
                    ) {
                        Ok(block) => {
                            slots[offset as usize] = Some(block);
                            remaining -= 1;
                        }
                        Err(e) => {
                            parse_failure = Some(e);
                            return ControlFlow::Break(());
                        }
                    }
                    if remaining == 0 {
                        ControlFlow::Break(())
                    } else {
                        ControlFlow::Continue(())
                    }
                },
            );

            if let Some(e) = parse_failure {
                return Err(e);
            }
            if remaining == 0 {
                break 'files;
            }

            // Carry pre-first-magic bytes into the earlier chunk so a
            // block straddling the boundary is stitched back together.
            end = start;
            if end > 0 {
                let prefix_len = result.first_magic.unwrap_or(buf.len());
                spillover.extend_from_slice(&buf[..prefix_len]);
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
