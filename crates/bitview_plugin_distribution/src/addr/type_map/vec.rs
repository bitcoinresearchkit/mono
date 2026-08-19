use brk_cohort::ByAddrType;
use derive_more::{Deref, DerefMut};

/// A vector for each address type.
#[derive(Debug, Deref, DerefMut)]
pub struct AddrTypeToVec<T>(ByAddrType<Vec<T>>);

impl<T> Default for AddrTypeToVec<T> {
    fn default() -> Self {
        Self(ByAddrType {
            p2a: vec![],
            p2pk33: vec![],
            p2pk65: vec![],
            p2pkh: vec![],
            p2sh: vec![],
            p2tr: vec![],
            p2wpkh: vec![],
            p2wsh: vec![],
        })
    }
}

impl<T> AddrTypeToVec<T> {
    /// Create with pre-allocated capacity per address type.
    pub fn with_capacity(capacity: usize) -> Self {
        Self(ByAddrType {
            p2a: Vec::with_capacity(capacity),
            p2pk33: Vec::with_capacity(capacity),
            p2pk65: Vec::with_capacity(capacity),
            p2pkh: Vec::with_capacity(capacity),
            p2sh: Vec::with_capacity(capacity),
            p2tr: Vec::with_capacity(capacity),
            p2wpkh: Vec::with_capacity(capacity),
            p2wsh: Vec::with_capacity(capacity),
        })
    }
}

impl<T> AddrTypeToVec<T> {
    /// Unwrap the inner ByAddrType.
    pub fn unwrap(self) -> ByAddrType<Vec<T>> {
        self.0
    }
}
