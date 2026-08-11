use brk_types::CentsSquaredSats;

#[derive(Clone, Debug, Default)]
pub(crate) struct PendingCapitalizedCapRawDelta {
    pub inc: CentsSquaredSats,
    pub dec: CentsSquaredSats,
}
