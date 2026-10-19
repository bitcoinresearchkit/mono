/// Compact index into one `Snapshot`'s dense `txs` vec, meaningless in any other snapshot.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct TxIndex(u32);

impl TxIndex {
    #[inline]
    pub fn as_usize(self) -> usize {
        self.0 as usize
    }
}

impl From<usize> for TxIndex {
    #[inline]
    fn from(value: usize) -> Self {
        Self(value as u32)
    }
}
