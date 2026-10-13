use bitview_primitives::{DecodedAddrState, TypeIndex};
use bitview_types::{AddressChainStats, AddressStats};
use brk_types::{Address, CheckedSub, Dollars, OutputType, Sats};
use vecdb::ReadableVec;

use super::parse_addr;
use crate::r#impl::mempool::anchored;
use crate::{Error, OptionData, Query, Result};

impl Query {
    pub fn addr(&self, addr: Address) -> Result<AddressStats> {
        let bytes = parse_addr(&addr)?;
        // The mempool first (it never leads the chain), then the address state, written with the
        // published lengths under this lock, then the pin: all of one publication.
        let mempool = self.mempool();
        let stable = self.plugins().addresses.state.read_stable();
        let pin = self.pin_safe_lengths()?;
        // Only a rollback, a failed write or a long catch-up leaves it at another length.
        if *stable != Some(pin.lengths().height) {
            return Err(Error::StateUpdating);
        }
        let chain_stats = self
            .find_addr_bytes(&bytes, pin.lengths())?
            .map(|(output_type, type_index)| self.addr_chain_stats(output_type, type_index))
            .transpose()?;
        drop(stable);
        let mempool_stats = mempool
            .map(|m| anchored(m.addr_stats(&bytes, &pin.tip())))
            .transpose()?
            .unwrap_or_default();
        let chain_stats = match chain_stats {
            Some(stats) => stats,
            // Seen only in the mempool.
            None if mempool_stats.tx_count > 0 => AddressChainStats::default(),
            None => return Err(Error::UnknownAddress),
        };
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

    fn addr_chain_stats(
        &self,
        output_type: OutputType,
        type_index: TypeIndex,
    ) -> Result<AddressChainStats> {
        let plugins = self.plugins();
        let state = plugins.addresses.state.get_once(output_type, type_index)?;

        let (addr_data, is_funded) = match state.decode() {
            DecodedAddrState::Funded(index) => {
                let data = plugins.addresses.state.funded.collect_one(index).data()?;
                (data, true)
            }
            DecodedAddrState::ExtendedEmpty(index) => {
                let data = plugins
                    .addresses
                    .state
                    .extended_empty
                    .collect_one(index)
                    .data()?
                    .into();
                (data, false)
            }
            DecodedAddrState::Empty(data) => (data.into(), false),
        };

        let balance = addr_data
            .received
            .checked_sub(addr_data.sent)
            .ok_or(Error::Internal(
                "Address sent amount exceeds confirmed receipts",
            ))?;
        let realized_price = if is_funded && !balance.is_zero() {
            addr_data
                .realized_cap_raw()
                .realized_price(balance)
                .to_dollars()
        } else {
            Dollars::default()
        };

        Ok(AddressChainStats {
            balance,
            type_index: Some(type_index),
            funded_txo_count: addr_data.funded_txo_count,
            funded_txo_sum: addr_data.received,
            spent_txo_count: addr_data.spent_txo_count,
            spent_txo_sum: addr_data.sent,
            tx_count: addr_data.tx_count,
            realized_price,
        })
    }
}

/// The confirmed balance with the mempool's pending receipts and spends.
fn pending_balance(chain: Sats, pending_received: Sats, pending_sent: Sats) -> Result<Sats> {
    // The live mempool and indexer publish independently. A just-confirmed
    // spend can still be present in the mempool view; never underflow or
    // fabricate a zero balance when those views cannot be reconciled.
    u64::from(chain)
        .checked_add(pending_received.into())
        .and_then(|value| value.checked_sub(pending_sent.into()))
        .map(Sats::from)
        .ok_or(Error::StateUpdating)
}
