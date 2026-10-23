use bitview_primitives::{P2MSOutputIndex, UnknownScriptOutputIndex};
use brk_types::SigOps32;
use vecdb::BytesVecReader;

pub struct ScriptReaders {
    pub p2ms_legacy_sigop_cost: BytesVecReader<P2MSOutputIndex, SigOps32>,
    pub unknown_script_legacy_sigop_cost: BytesVecReader<UnknownScriptOutputIndex, SigOps32>,
}
