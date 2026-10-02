use std::cmp::Reverse;

use crate::record::Record;

#[derive(Default)]
pub struct Group {
    pub durations: Vec<u64>,
    pub total: u128,
    pub slowest: Vec<Record>,
    pub matched: bool,
}

impl Group {
    pub fn push(&mut self, record: Record) {
        self.durations.push(record.nanos);
        self.total += record.nanos as u128;
        self.slowest.push(record);
        self.slowest.sort_by_key(|r| Reverse(r.nanos));
        self.slowest.truncate(3);
    }
    pub fn percentile(&self, p: usize) -> u64 {
        self.quantile(p, 100)
    }
    fn quantile(&self, numerator: usize, denominator: usize) -> u64 {
        let rank = (self.durations.len() as u128 * numerator as u128).div_ceil(denominator as u128)
            as usize;
        self.durations[rank - 1]
    }
    pub fn p999(&self) -> Option<u64> {
        (self.durations.len() >= 1_000).then(|| self.quantile(999, 1_000))
    }
}
