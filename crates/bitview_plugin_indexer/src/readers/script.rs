use brk_types::{P2MSOutputIndex, SigOps, UnknownOutputIndex};
use vecdb::BytesVecReader;

pub struct ScriptReaders {
    pub p2ms_legacy_sigops: BytesVecReader<P2MSOutputIndex, SigOps>,
    pub unknown_legacy_sigops: BytesVecReader<UnknownOutputIndex, SigOps>,
}
