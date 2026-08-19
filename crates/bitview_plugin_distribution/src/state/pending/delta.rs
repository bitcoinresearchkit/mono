use brk_types::Sats;

#[derive(Clone, Copy, Debug, Default)]
pub struct PendingDelta {
    pub inc: Sats,
    pub dec: Sats,
}
