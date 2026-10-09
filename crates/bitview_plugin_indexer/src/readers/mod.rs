pub mod addr;
pub mod script;

pub use addr::AddrReaders;
pub use script::ScriptReaders;

use brk_types::{TxIndex, Txid};
use vecdb::BytesVecReader;

use crate::Vecs;

/// Readers for vectors that need to be accessed during block processing.
///
/// All fields use `BytesVecReader` which caches the mmap base pointer for O(1)
/// random access without recomputing `region.start() + HEADER_OFFSET` per read.
pub struct Readers {
    pub txid: BytesVecReader<TxIndex, Txid>,
    pub scripts: ScriptReaders,
    pub addrs: AddrReaders,
}

impl Readers {
    pub fn new(vecs: &Vecs) -> Self {
        Self {
            txid: vecs.transactions.txid.reader(),
            scripts: ScriptReaders {
                p2ms_legacy_sigops: vecs.scripts.p2ms.legacy_sigops.reader(),
                unknown_legacy_sigops: vecs.scripts.unknown.legacy_sigops.reader(),
            },
            addrs: vecs.addrs.addr_readers(),
        }
    }
}
