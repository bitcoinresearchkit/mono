use std::collections::BTreeMap;

use brk_types::{FeeRate, MempoolInfo, Sats, VSize};

/// Writer totals and the exact per-rate histogram, retained privately by each publication.
/// Wire bins are lossy and cannot be used to restore this state.
#[derive(Clone, Default)]
pub(crate) struct MempoolAccounting {
    pub count: usize,
    vsize: VSize,
    total_fee: Sats,
    fee_histogram: BTreeMap<FeeRate, VSize>,
}

impl MempoolAccounting {
    /// `vsize`: the entry's, sigop-adjusted (Core's `GetTxSize`).
    #[inline]
    pub(crate) fn add(&mut self, vsize: VSize, fee: Sats) {
        self.count += 1;
        self.vsize += vsize;
        self.total_fee += fee;
        let rate = FeeRate::from((fee, vsize));
        *self.fee_histogram.entry(rate).or_insert(VSize::from(0u64)) += vsize;
    }

    /// `vsize`: the same the transaction was added with.
    #[inline]
    pub(crate) fn remove(&mut self, vsize: VSize, fee: Sats) {
        self.count -= 1;
        self.vsize -= vsize;
        self.total_fee -= fee;
        let rate = FeeRate::from((fee, vsize));
        if let Some(v) = self.fee_histogram.get_mut(&rate) {
            *v -= vsize;
            if u64::from(*v) == 0 {
                self.fee_histogram.remove(&rate);
            }
        }
    }

    /// Build the wire record only when a publication needs new info JSON.
    pub(crate) fn wire(&self) -> MempoolInfo {
        // Esplora's make_fee_histogram: descend through exact rates, closing a bin at the
        // first rate change after its size exceeds (strictly) 50,000 vbytes.
        let mut bins = Vec::new();
        let (mut rate, mut vsize) = (FeeRate::ZERO, 0);
        for (&next_rate, &next_vsize) in self.fee_histogram.iter().rev() {
            if vsize > VSIZE_BIN_WIDTH {
                bins.push((rate, VSize::from(vsize)));
                vsize = 0;
            }
            rate = next_rate;
            vsize += u64::from(next_vsize);
        }
        if vsize > 0 {
            bins.push((rate, VSize::from(vsize)));
        }
        MempoolInfo {
            count: self.count,
            vsize: self.vsize,
            total_fee: self.total_fee,
            fee_histogram: bins,
        }
    }
}

/// Esplora's histogram bin width (vbytes).
const VSIZE_BIN_WIDTH: u64 = 50_000;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wire_bins_keep_strict_threshold_and_exact_rate_groups() {
        let mut accounting = MempoolAccounting::default();
        let high = (VSize::from(25_000u64), Sats::from(250_000u64));
        accounting.add(high.0, high.1);
        accounting.add(high.0, high.1);
        accounting.add(VSize::from(1u64), Sats::from(9u64));
        accounting.add(VSize::from(7u64), Sats::ZERO);
        let bytes = serde_json::to_vec(&accounting.wire()).unwrap();
        assert_eq!(
            bytes.as_slice(),
            br#"{"count":4,"vsize":50008,"total_fee":500009,"fee_histogram":[[9.0,50001],[0.0,7]]}"#,
        );
        let decoded: MempoolInfo = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(serde_json::to_vec(&decoded).unwrap(), bytes);

        // Remove one of two transactions sharing a rate, then its last member.
        accounting.remove(high.0, high.1);
        assert_eq!(accounting.fee_histogram[&FeeRate::from_milli(10_000)], high.0);
        accounting.remove(high.0, high.1);
        assert!(!accounting.fee_histogram.contains_key(&FeeRate::from_milli(10_000)));
        accounting.remove(VSize::from(1u64), Sats::from(9u64));
        accounting.remove(VSize::from(7u64), Sats::ZERO);
        assert_eq!(
            serde_json::to_vec(&accounting.wire()).unwrap().as_slice(),
            br#"{"count":0,"vsize":0,"total_fee":0,"fee_histogram":[]}"#,
        );
    }
}
