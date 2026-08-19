use crate::Checksum;

pub struct RecoveredTable {
    pub id: u32,
    pub checksum: Checksum,
    pub global_seqno: u64,
}
