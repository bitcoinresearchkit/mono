use brk_types::{AddrBytes, BlockHash, OutputType, TypeIndex};

use super::{ResolvedAddrChainTxs, combined::AddrTxsLimits};
use crate::r#impl::addr::mempool::AddrMempoolTxsPagePreflight;

/// A mixed address transaction page resolved before its body is loaded.
pub struct ResolvedAddrTxs {
    addr: AddrBytes,
    mempool: AddrMempoolTxsPagePreflight,
    chain_addr: Option<(OutputType, TypeIndex)>,
    chain: Option<ResolvedAddrChainTxs>,
    chain_tip: BlockHash,
    limits: AddrTxsLimits,
}

impl ResolvedAddrTxs {
    pub(super) fn new(
        addr: AddrBytes,
        mempool: AddrMempoolTxsPagePreflight,
        chain_addr: Option<(OutputType, TypeIndex)>,
        chain: Option<ResolvedAddrChainTxs>,
        chain_tip: BlockHash,
        limits: AddrTxsLimits,
    ) -> Self {
        Self {
            addr,
            mempool,
            chain_addr,
            chain,
            chain_tip,
            limits,
        }
    }

    pub(super) fn into_parts(
        self,
    ) -> (
        AddrBytes,
        AddrMempoolTxsPagePreflight,
        Option<(OutputType, TypeIndex)>,
        Option<ResolvedAddrChainTxs>,
        BlockHash,
        AddrTxsLimits,
    ) {
        (
            self.addr,
            self.mempool,
            self.chain_addr,
            self.chain,
            self.chain_tip,
            self.limits,
        )
    }
}
