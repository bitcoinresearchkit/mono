/// Status of an address before a receive.
#[derive(Clone, Copy)]
pub enum AddrReceiveStatus {
    /// Brand new address (never seen before).
    New,
    /// Already tracked in a cohort (has existing balance).
    Tracked,
    /// Was in the empty cache and is rejoining a cohort.
    WasEmpty,
}
