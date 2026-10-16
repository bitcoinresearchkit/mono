use bitcoin::{
    Script,
    opcodes::all::{OP_CHECKMULTISIG, OP_CHECKMULTISIGVERIFY, OP_CHECKSIG, OP_CHECKSIGVERIFY},
    script::Instruction,
};

/// Bitcoin Core's `CScript::GetSigOpCount`, both modes in one walk: legacy counts every
/// CHECKMULTISIG as 20, accurate as the `OP_n` right before it (20 without one). Every instruction
/// sets that preceding opcode, signature opcodes included (rust-bitcoin's accurate count keeps a
/// stale `OP_n` across them). A malformed push ends the walk, as in Core.
#[derive(Default)]
pub struct SigOpCount {
    pub legacy: usize,
    pub accurate: usize,
    pushnum: Option<usize>,
}

impl SigOpCount {
    const MAX_PUBKEYS_PER_MULTISIG: usize = 20;

    pub fn of(script: &Script) -> Self {
        let mut count = Self::default();
        for instruction in script.instructions() {
            let Ok(instruction) = instruction else {
                break;
            };
            count.step(instruction);
        }
        count
    }

    #[inline]
    pub fn step(&mut self, instruction: Instruction<'_>) {
        let Instruction::Op(opcode) = instruction else {
            self.pushnum = None;
            return;
        };
        match opcode {
            OP_CHECKSIG | OP_CHECKSIGVERIFY => {
                self.legacy += 1;
                self.accurate += 1;
            }
            OP_CHECKMULTISIG | OP_CHECKMULTISIGVERIFY => {
                self.legacy += Self::MAX_PUBKEYS_PER_MULTISIG;
                self.accurate += self.pushnum.unwrap_or(Self::MAX_PUBKEYS_PER_MULTISIG);
            }
            _ => {}
        }
        let opcode = opcode.to_u8();
        self.pushnum = (0x51..=0x60)
            .contains(&opcode)
            .then(|| usize::from(opcode - 0x50));
    }
}
