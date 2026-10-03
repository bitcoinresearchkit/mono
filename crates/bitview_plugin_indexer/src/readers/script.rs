use bitview_primitives::{P2MSOutputIndex, UnknownOutputIndex};
use brk_types::SigOps;
use vecdb::BytesVecReader;

pub struct ScriptReaders {
    pub p2ms_legacy_sigops: BytesVecReader<P2MSOutputIndex, SigOps>,
    pub unknown_legacy_sigops: BytesVecReader<UnknownOutputIndex, SigOps>,
}
