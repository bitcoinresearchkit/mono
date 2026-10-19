use bitview_primitives::{AddrHash, TypeIndex};
use brk_types::{OutputType, SigOps32};

use super::op_return;

#[derive(Clone, Copy)]
pub enum ProcessedOutputData {
    Address(AddrHash),
    OpReturn(op_return::Facts),
    /// A P2MS or unknown script, with its legacy sigop count.
    Script {
        legacy_sigops: u32,
    },
    /// An empty script.
    None,
    Resolved(TypeIndex),
}

#[derive(Clone, Copy)]
pub struct ProcessedOutput {
    pub output_type: OutputType,
    /// The legacy sigop cost a spend executes: the locking script's accurate count, scaled.
    pub legacy_sigop_cost: SigOps32,
    pub data: ProcessedOutputData,
}

/// Whether an address type's script ends in a single OP_CHECKSIG (P2PKH, P2PK): the others hold
/// no signature opcode outside their pushdata.
#[inline]
pub fn is_single_sig(output_type: OutputType) -> bool {
    matches!(
        output_type,
        OutputType::P2PKH | OutputType::P2PK33 | OutputType::P2PK65
    )
}

impl ProcessedOutput {
    /// The locking script's legacy sigop count, which its transaction's total includes: read
    /// before the output is resolved.
    #[inline]
    pub fn script_legacy_sigops(&self) -> usize {
        match self.data {
            ProcessedOutputData::Address(_) => usize::from(is_single_sig(self.output_type)),
            ProcessedOutputData::OpReturn(facts) => facts.legacy_sigops as usize,
            ProcessedOutputData::Script { legacy_sigops } => legacy_sigops as usize,
            ProcessedOutputData::None => 0,
            ProcessedOutputData::Resolved(_) => unreachable!(),
        }
    }

    pub fn resolved_type_index(&self) -> TypeIndex {
        let ProcessedOutputData::Resolved(type_index) = self.data else {
            unreachable!();
        };
        type_index
    }
}
