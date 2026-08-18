use crate::{Checksum, SeqNo, TableId};

pub struct RecoveredTable {
    pub id: TableId,
    pub checksum: Checksum,
    pub global_seqno: SeqNo,
}
