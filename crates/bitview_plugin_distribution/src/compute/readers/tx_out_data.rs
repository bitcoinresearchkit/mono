use brk_types::{OutputType, Sats, TypeIndex};

/// Output data collected from separate vectors.
#[derive(Debug, Clone, Copy)]
pub struct TxOutData {
    pub value: Sats,
    pub output_type: OutputType,
    pub type_index: TypeIndex,
}
