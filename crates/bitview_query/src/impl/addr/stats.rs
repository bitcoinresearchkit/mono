use std::str::FromStr;

use brk_error::{Error, Result};
use brk_types::{
    Addr, AddrBytes, AddrChainStats, AddrHash, AddrStats, DecodedAddrState, Dollars, OutputType,
    TypeIndex,
};
use vecdb::ReadableVec;

use crate::Query;

impl Query {
    pub fn addr(&self, addr: Addr) -> Result<AddrStats> {
        let bytes = AddrBytes::from_str(&addr)?;
        let _guard = self.read_plugin(self.plugins().distribution)?;
        let (output_type, type_index) = self.resolve_addr_stats(&bytes)?;
        self.addr_stats(addr, bytes, output_type, type_index)
    }

    /// Resolve complete address stats without waiting for an in-flight
    /// distribution update. `None` asks the caller to use the blocking path.
    pub fn addr_stats_preflight(&self, addr: &Addr) -> Result<Option<AddrStats>> {
        let bytes = AddrBytes::from_str(addr)?;
        let Some(_guard) = self.try_read_plugin(self.plugins().distribution) else {
            return Ok(None);
        };
        let (output_type, type_index) = self.resolve_addr_stats(&bytes)?;
        self.addr_stats(addr.clone(), bytes, output_type, type_index)
            .map(Some)
    }

    fn resolve_addr_stats(&self, bytes: &AddrBytes) -> Result<(OutputType, TypeIndex)> {
        let output_type = OutputType::from(bytes);
        let hash = AddrHash::from(bytes);
        let type_index = super::resolve::type_index_for(self, output_type, &hash)?;
        if type_index >= self.safe_lengths().to_type_index(output_type) {
            return Err(Error::UnknownAddr);
        }
        Ok((output_type, type_index))
    }

    fn addr_stats(
        &self,
        addr: Addr,
        bytes: AddrBytes,
        output_type: OutputType,
        type_index: TypeIndex,
    ) -> Result<AddrStats> {
        let plugins = self.plugins();
        let state = plugins
            .distribution
            .addr_state
            .get_once(output_type, type_index)?;

        let (addr_data, realized_price) = match state.decode() {
            DecodedAddrState::Funded(index) => {
                let data = plugins
                    .distribution
                    .addr_state
                    .funded
                    .collect_one(index)
                    .expect("funded address data index should be in bounds");
                let price = data.realized_price().to_dollars();
                (data, price)
            }
            DecodedAddrState::ExtendedEmpty(index) => {
                let data = plugins
                    .distribution
                    .addr_state
                    .extended_empty
                    .collect_one(index)
                    .expect("extended empty address data index should be in bounds")
                    .into();
                (data, Dollars::default())
            }
            DecodedAddrState::Empty(data) => (data.into(), Dollars::default()),
        };

        let mempool_stats = self
            .mempool()
            .and_then(|m| m.addr_stats(&bytes))
            .unwrap_or_default();
        let balance = addr_data.received + mempool_stats.funded_txo_sum
            - addr_data.sent
            - mempool_stats.spent_txo_sum;

        Ok(AddrStats {
            addr,
            addr_type: output_type,
            chain_stats: AddrChainStats {
                balance: addr_data.received - addr_data.sent,
                type_index,
                funded_txo_count: addr_data.funded_txo_count,
                funded_txo_sum: addr_data.received,
                spent_txo_count: addr_data.spent_txo_count,
                spent_txo_sum: addr_data.sent,
                tx_count: addr_data.tx_count,
                realized_price,
            },
            mempool_stats,
            balance,
        })
    }
}
