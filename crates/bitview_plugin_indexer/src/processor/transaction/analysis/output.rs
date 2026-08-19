use crate::TxFeatureFlags;
use crate::processor::txout::ProcessedOutput;
use bitcoin::{PublicKey, TxOut, script::Instruction};
use brk_types::OutputType;

#[derive(Default)]
pub struct Scanner {
    p2wsh_count: usize,
    payload_size: usize,
}

impl Scanner {
    pub fn scan(&mut self, txout: &TxOut, output: &ProcessedOutput, flags: &mut TxFeatureFlags) {
        flags.insert_type(output.output_type);

        match output.output_type {
            OutputType::P2PK33 | OutputType::P2PK65 => {
                if txout.script_pubkey.p2pk_public_key().is_none() {
                    flags.insert(TxFeatureFlags::FAKE_PUBKEY);
                }
                self.p2wsh_count = 0;
            }
            OutputType::P2MS => {
                if has_fake_multisig_key(&txout.script_pubkey) {
                    flags.insert(TxFeatureFlags::FAKE_PUBKEY);
                }
                self.p2wsh_count = 0;
            }
            OutputType::P2WSH => {
                let program = &txout.script_pubkey.as_bytes()[2..];
                if self.p2wsh_count == 0 {
                    self.payload_size = u16::from_be_bytes([program[0], program[1]]) as usize;
                }
                self.p2wsh_count += 1;

                let output_count = (self.payload_size + 33) / 32;
                if self.p2wsh_count == output_count {
                    let padding = output_count * 32 - self.payload_size - 2;
                    if program[32 - padding..].iter().all(|byte| *byte == 0) {
                        flags.insert(TxFeatureFlags::FAKE_SCRIPTHASH);
                    }
                }
            }
            _ => self.p2wsh_count = 0,
        }
    }
}

pub fn has_fake_multisig_key(script: &bitcoin::Script) -> bool {
    script.instructions().any(|instruction| {
        let Ok(Instruction::PushBytes(bytes)) = instruction else {
            return false;
        };
        let bytes = bytes.as_bytes();
        is_burn_key(bytes) || PublicKey::from_slice(bytes).is_err()
    })
}

pub fn is_burn_key(bytes: &[u8]) -> bool {
    bytes.len() == 33
        && matches!(
            bytes,
            [0x02, rest @ ..]
                if rest.iter().all(|byte| *byte == 0x02)
                    || rest.iter().all(|byte| *byte == 0x22)
        )
        || bytes.len() == 33
            && matches!(
                bytes,
                [0x03, rest @ ..]
                    if rest.iter().all(|byte| *byte == 0x03)
                        || rest.iter().all(|byte| *byte == 0x33)
            )
}
