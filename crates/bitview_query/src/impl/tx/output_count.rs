use bitview_primitives::TxOutIndex;
use brk_types::Vout;

use crate::{Error, Result};

/// Validate stored boundaries before allocating or converting offsets to Vout.
/// This is the indexer's representable range, not a Bitcoin consensus limit.
pub fn output_count(first: TxOutIndex, next: TxOutIndex, published: TxOutIndex) -> Result<usize> {
    let count = u64::from(next)
        .checked_sub(u64::from(first))
        .ok_or(Error::Internal("Invalid transaction output boundaries"))?;
    if next > published {
        return Err(Error::Internal("Transaction outputs exceed published data"));
    }
    if count > u64::from(Vout::MAX) + 1 {
        return Err(Error::Internal(
            "Transaction output count exceeds index capacity",
        ));
    }
    Ok(count as usize)
}
