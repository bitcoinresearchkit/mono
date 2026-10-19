use brk_types::SigOps32;

#[derive(Clone, Copy, Default)]
pub struct ComputedSigOps {
    pub total: SigOps32,
    pub executed_legacy: SigOps32,
}
