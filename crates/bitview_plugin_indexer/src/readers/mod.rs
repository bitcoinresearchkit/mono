pub mod addr;
pub mod script;

pub use addr::AddrReaders;
pub use script::ScriptReaders;

use crate::Vecs;

/// All fields use `BytesVecReader` which caches the mmap base pointer for O(1)
/// random access without recomputing `region.start() + HEADER_OFFSET` per read.
pub struct Readers {
    pub scripts: ScriptReaders,
    pub addrs: AddrReaders,
}

impl Readers {
    pub fn new(vecs: &Vecs) -> Self {
        Self {
            scripts: ScriptReaders {
                p2ms_legacy_sigop_cost: vecs.scripts.p2ms.legacy_sigop_cost.reader(),
                unknown_script_legacy_sigop_cost: vecs.scripts.unknown_script.legacy_sigop_cost.reader(),
            },
            addrs: vecs.addrs.addr_readers(),
        }
    }
}
