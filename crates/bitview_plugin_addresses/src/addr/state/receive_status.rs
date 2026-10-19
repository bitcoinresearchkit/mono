/// Status of an address before a receive.
pub enum AddrReceiveStatus {
    New,
    /// Already tracked in a cohort (holds UTXOs).
    Tracked,
    /// Received before but holds no UTXOs; rejoins a cohort.
    WasEmpty,
}
