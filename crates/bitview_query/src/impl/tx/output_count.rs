use bitview_primitives::TxOutIndex;

use crate::{Error, Result};

/// Validate stored boundaries before allocating or converting offsets to Vout.
pub fn output_count(first: TxOutIndex, next: TxOutIndex, published: TxOutIndex) -> Result<usize> {
    let count = u64::from(next)
        .checked_sub(u64::from(first))
        .ok_or(Error::Internal("Invalid transaction output boundaries"))?;
    if next > published {
        return Err(Error::Internal("Transaction outputs exceed published data"));
    }
    Ok(count as usize)
}
