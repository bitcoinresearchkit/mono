/// Status of an address before a receive.
#[derive(Clone, Copy)]
pub enum AddrReceiveStatus {
    /// Brand new address (never seen before).
    New,
    /// Already tracked in a cohort (holds UTXOs).
    Tracked,
    /// Received before but holds no UTXOs; rejoins a cohort.
    WasEmpty,
}
