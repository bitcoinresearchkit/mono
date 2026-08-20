use std::str::FromStr;

use brk_error::Error;
use brk_types::{
    Addr, AddrBytes, AddrChainStats, AddrHash, AddrStats, AnyAddrDataIndexEnum, Dollars,
    OutputType, TypeIndex,
};
use vecdb::ReadableVec;

use crate::Query;

impl Query {
    pub fn addr(&self, addr: Addr) -> brk_error::Result<AddrStats> {
        let _guard = self.read_plugin(self.plugins().distribution)?;
        let bytes = AddrBytes::from_str(&addr)?;
        let output_type = OutputType::from(&bytes);
        let hash = AddrHash::from(&bytes);
        let type_index = super::resolve::type_index_for(self, output_type, &hash)?;
        self.addr_stats(addr, bytes, output_type, type_index)
    }

    fn addr_stats(
        &self,
        addr: Addr,
        bytes: AddrBytes,
        output_type: OutputType,
        type_index: TypeIndex,
    ) -> brk_error::Result<AddrStats> {
        if type_index >= self.safe_lengths().to_type_index(output_type) {
            return Err(Error::UnknownAddr);
        }

        let plugins = self.plugins();
        let any_addr_index = plugins
            .distribution
            .any_addr_indexes
            .get_once(output_type, type_index)?;

        let (addr_data, realized_price) = match any_addr_index.to_enum() {
            AnyAddrDataIndexEnum::Funded(index) => {
                let data = plugins
                    .distribution
                    .addrs_data
                    .funded
                    .collect_one(index)
                    .expect("funded address data index should be in bounds");
                let price = data.realized_price().to_dollars();
                (data, price)
            }
            AnyAddrDataIndexEnum::Empty(index) => {
                let data = plugins
                    .distribution
                    .addrs_data
                    .empty
                    .collect_one(index)
                    .expect("empty address data index should be in bounds")
                    .into();
                (data, Dollars::default())
            }
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
