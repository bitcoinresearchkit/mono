use bitview_primitives::{FundedAddressData, TypeIndex};
use bitview_types::{AddressChainStats, AddressStats};
use brk_types::{Address, Dollars, OutputType, Sats};

use super::parse_addr;
use crate::r#impl::mempool::anchored;
use crate::{Error, Query, Result};

impl Query {
    pub fn addr(&self, addr: Address) -> Result<AddressStats> {
        let bytes = parse_addr(&addr)?;
        // The mempool first (it never leads the chain), then the address state, published with the
        // lengths under this lock, then the pin: the state and the pin are of one publication, and
        // a mempool at another tip adds nothing (`anchored`).
        let mempool = self.mempool();
        let state = self
            .plugins()
            .addresses
            .read_state()
            .ok_or(Error::StateUpdating)?;
        let pin = self.pin_safe_lengths()?;
        // Only a rollback, a failed write or a long catch-up leaves it at another length.
        if state.written() != Some(pin.lengths().height) {
            return Err(Error::StateUpdating);
        }
        let chain_stats = self
            .find_addr_bytes(&bytes, &pin)?
            .map(|(output_type, type_index)| -> Result<_> {
                let (addr_data, is_funded) = state.addr_data(output_type, type_index)?;
                Ok(addr_chain_stats(type_index, &addr_data, is_funded))
            })
            .transpose()?;
        drop(state);
        let mempool_stats = mempool
            .map(|m| anchored(m.addr_stats(&bytes, &pin.tip())))
            .transpose()?
            .unwrap_or_default();
        // Never confirmed: zeroed stats, as Esplora answers.
        let chain_stats = chain_stats.unwrap_or_default();
        let balance = pending_balance(
            chain_stats.balance,
            mempool_stats.funded_txo_sum,
            mempool_stats.spent_txo_sum,
        )?;
        Ok(AddressStats {
            address: addr,
            address_type: OutputType::from(&bytes),
            chain_stats,
            mempool_stats,
            balance,
        })
    }
}

fn addr_chain_stats(
    type_index: TypeIndex,
    addr_data: &FundedAddressData,
    is_funded: bool,
) -> AddressChainStats {
    let balance = addr_data.balance();
    let realized_price = if is_funded && !balance.is_zero() {
        addr_data
            .realized_cap_raw()
            .realized_price(balance)
            .to_dollars()
    } else {
        Dollars::default()
    };

    AddressChainStats {
        balance,
        type_index: Some(type_index),
        funded_txo_count: addr_data.funded_txo_count,
        funded_txo_sum: addr_data.received,
        spent_txo_count: addr_data.spent_txo_count,
        spent_txo_sum: addr_data.sent,
        tx_count: addr_data.tx_count,
        realized_price,
    }
}

fn pending_balance(chain: Sats, pending_received: Sats, pending_sent: Sats) -> Result<Sats> {
    // Mempool stats are anchored to the chain tip; reject inconsistent totals.
    u64::from(chain)
        .checked_add(pending_received.into())
        .and_then(|value| value.checked_sub(pending_sent.into()))
        .map(Sats::from)
        .ok_or(Error::StateUpdating)
}
