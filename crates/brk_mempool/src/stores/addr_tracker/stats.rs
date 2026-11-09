use brk_types::{AddressMempoolStats, SatsSigned, TxOut};

pub(crate) fn add_funding(stats: &mut AddressMempoolStats, txout: &TxOut) {
    stats.balance_delta += SatsSigned::from(txout.value);
    stats.funded_txo_count += 1;
    stats.funded_txo_sum += txout.value;
}

pub(crate) fn remove_funding(stats: &mut AddressMempoolStats, txout: &TxOut) {
    stats.balance_delta -= SatsSigned::from(txout.value);
    stats.funded_txo_count -= 1;
    stats.funded_txo_sum -= txout.value;
}

pub(crate) fn add_spend(stats: &mut AddressMempoolStats, txout: &TxOut) {
    stats.balance_delta -= SatsSigned::from(txout.value);
    stats.spent_txo_count += 1;
    stats.spent_txo_sum += txout.value;
}

pub(crate) fn remove_spend(stats: &mut AddressMempoolStats, txout: &TxOut) {
    stats.balance_delta += SatsSigned::from(txout.value);
    stats.spent_txo_count -= 1;
    stats.spent_txo_sum -= txout.value;
}
