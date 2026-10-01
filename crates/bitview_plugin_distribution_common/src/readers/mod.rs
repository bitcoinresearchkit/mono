mod tx_in;
mod tx_out;
pub use tx_in::TxInReaders;
pub use tx_out::TxOutReaders;

use std::ops::Range;
use vecdb::VecIndex;

pub fn index_range<I: VecIndex>(first: &[I], from: usize, to: usize, len: usize) -> Range<usize> {
    first[from].to_usize()..first.get(to).map_or(len, |i| i.to_usize())
}
