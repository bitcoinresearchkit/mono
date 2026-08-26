use bitview_cohort::ByAddrType;
use brk_store::Store;
use brk_types::{
    AddrHash, AddrIndexOutPoint, AddrIndexTxIndex, TxIndex, TxidPrefix, TypeIndex, Unit,
};

pub struct TransactionStoresMut<'a> {
    pub addr_hashes: &'a mut ByAddrType<Store<AddrHash, TypeIndex>>,
    pub addr_tx_indexes: &'a mut ByAddrType<Store<AddrIndexTxIndex, Unit>>,
    pub addr_unspent_outpoints: &'a mut ByAddrType<Store<AddrIndexOutPoint, Unit>>,
    pub txid_prefixes: &'a mut Store<TxidPrefix, TxIndex>,
}
