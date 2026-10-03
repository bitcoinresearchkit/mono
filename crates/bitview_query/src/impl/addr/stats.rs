use bitview_primitives::{DecodedAddrState, TypeIndex};
use bitview_types::{AddrChainStats, AddrStats};
use brk_types::{Addr, AddrBytes, Dollars, OutputType, Sats};
use vecdb::ReadableVec;

use super::parse_addr;
use crate::{Error, OptionData, Query, Result};

impl Query {
    pub fn addr(&self, addr: Addr) -> Result<AddrStats> {
        let bytes = parse_addr(&addr)?;
        let _guard = self.read_publication()?;
        let (output_type, type_index) = self.resolve_addr_bytes(&bytes)?;
        self.addr_stats(addr, bytes, output_type, type_index)
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
            .distribution_addresses
            .addr_state
            .get_once(output_type, type_index)?;

        let (addr_data, is_funded) = match state.decode() {
            DecodedAddrState::Funded(index) => {
                let data = plugins
                    .distribution_addresses
                    .addr_state
                    .funded
                    .collect_one(index)
                    .data()?;
                (data, true)
            }
            DecodedAddrState::ExtendedEmpty(index) => {
                let data = plugins
                    .distribution_addresses
                    .addr_state
                    .extended_empty
                    .collect_one(index)
                    .data()?
                    .into();
                (data, false)
            }
            DecodedAddrState::Empty(data) => (data.into(), false),
        };

        let mempool_stats = self
            .mempool()
            .map(|m| m.addr_stats(&bytes, &self.tip_blockhash()))
            .transpose()?
            .unwrap_or_default();
        let (chain_balance, balance) = address_balances(
            addr_data.received,
            addr_data.sent,
            mempool_stats.funded_txo_sum,
            mempool_stats.spent_txo_sum,
        )?;
        let realized_price = if is_funded {
            addr_data
                .realized_cap_raw()
                .realized_price(chain_balance)
                .to_dollars()
        } else {
            Dollars::default()
        };

        Ok(AddrStats {
            addr,
            addr_type: output_type,
            chain_stats: AddrChainStats {
                balance: chain_balance,
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

fn address_balances(
    received: Sats,
    sent: Sats,
    pending_received: Sats,
    pending_sent: Sats,
) -> Result<(Sats, Sats)> {
    let chain = u64::from(received)
        .checked_sub(sent.into())
        .ok_or(Error::Internal(
            "Address sent amount exceeds confirmed receipts",
        ))?;
    // The live mempool and indexer publish independently. A just-confirmed
    // spend can still be present in the mempool view; never underflow or
    // fabricate a zero balance when those views cannot be reconciled.
    let combined = chain
        .checked_add(pending_received.into())
        .and_then(|value| value.checked_sub(pending_sent.into()))
        .ok_or(Error::StateUpdating)?;
    Ok((chain.into(), combined.into()))
}
