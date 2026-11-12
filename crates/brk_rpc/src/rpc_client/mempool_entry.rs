use bitcoin::Wtxid;
use brk_types::{Bitcoin, MempoolEntryInfo, Sats, Timestamp, Txid, VSize, Weight};
use serde::Deserialize;

/// The subset of a `getmempoolentry` response that BRK consumes.
#[derive(Deserialize)]
#[serde(try_from = "WireEntry")]
pub struct MempoolEntry {
    wtxid: Wtxid,
    vsize: VSize,
    weight: Weight,
    time: Timestamp,
    fees: MempoolFees,
}

/// A `getmempoolentry` response as sent. Core 32 adds `vsize_adjusted` and deprecates `vsize`, which
/// holds the same sigop-adjusted size.
#[derive(Deserialize)]
struct WireEntry {
    wtxid: Wtxid,
    vsize_adjusted: Option<VSize>,
    vsize: Option<VSize>,
    weight: Weight,
    time: Timestamp,
    fees: MempoolFees,
}

impl TryFrom<WireEntry> for MempoolEntry {
    type Error = &'static str;

    fn try_from(wire: WireEntry) -> Result<Self, Self::Error> {
        let vsize = wire
            .vsize_adjusted
            .or(wire.vsize)
            .ok_or("a mempool entry without a virtual size")?;
        if !vsize.bounds_entry(wire.weight) {
            return Err("a mempool entry empty or larger than a block");
        }
        Ok(Self {
            wtxid: wire.wtxid,
            vsize,
            weight: wire.weight,
            time: wire.time,
            fees: wire.fees,
        })
    }
}

impl MempoolEntry {
    pub fn into_info(self, txid: Txid) -> MempoolEntryInfo {
        MempoolEntryInfo {
            txid,
            wtxid: self.wtxid,
            vsize: self.vsize,
            weight: self.weight,
            fee: Sats::from(self.fees.base),
            first_seen: self.time,
        }
    }
}

#[derive(Deserialize)]
struct MempoolFees {
    base: Bitcoin,
}
