use bitview_primitives::{Bytes32, TxVersion};
use brk_types::{RawLockTime, SigOps, TxIndex, Txid, Weight};
use vecdb::{BytesVec, PcoVec};

use super::{TransactionFeaturesVecs, VersionCountVecs};

pub struct TxMetadataVecs<'a> {
    pub tx_version: &'a mut PcoVec<TxIndex, TxVersion>,
    pub txid: &'a mut BytesVec<TxIndex, Txid>,
    pub raw_locktime: &'a mut PcoVec<TxIndex, RawLockTime>,
    pub weight: &'a mut PcoVec<TxIndex, Weight>,
    pub total_size: &'a mut PcoVec<TxIndex, Bytes32>,
    pub total_sigop_cost: &'a mut PcoVec<TxIndex, SigOps>,
    pub features: &'a mut TransactionFeaturesVecs,
    pub versions: &'a mut VersionCountVecs,
}
