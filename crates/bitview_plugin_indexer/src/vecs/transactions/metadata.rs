use brk_types::{RawLockTime, SigOps, StoredBool, StoredU32, TxIndex, TxVersion, Txid, Weight};
use vecdb::{BytesVec, PcoVec};

pub struct TxMetadataVecs<'a> {
    pub tx_version: &'a mut PcoVec<TxIndex, TxVersion>,
    pub txid: &'a mut BytesVec<TxIndex, Txid>,
    pub raw_locktime: &'a mut PcoVec<TxIndex, RawLockTime>,
    pub weight: &'a mut PcoVec<TxIndex, Weight>,
    pub total_size: &'a mut PcoVec<TxIndex, StoredU32>,
    pub total_sigop_cost: &'a mut PcoVec<TxIndex, SigOps>,
    pub is_explicitly_rbf: &'a mut PcoVec<TxIndex, StoredBool>,
}
