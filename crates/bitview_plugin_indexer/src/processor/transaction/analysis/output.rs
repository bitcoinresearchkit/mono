use bitcoin::{Script, TxOut, script::Instruction, secp256k1::PublicKey};
use brk_types::OutputType;

use crate::{TxFeatureFlags, processor::txout::ProcessedOutput};

#[derive(Default)]
pub struct Scanner {
    p2wsh_count: usize,
    payload_size: usize,
}

/// The keys a P2PK or P2MS output locks to that are curve points, by encoding: each parsed once,
/// for its count and for the fake-key flag. A key is what consensus parses as one: compressed,
/// uncompressed or hybrid (65 bytes opening 0x06 or 0x07).
#[derive(Clone, Copy, Default)]
pub struct Keys {
    pub compressed: u32,
    pub uncompressed: u32,
}

impl Keys {
    /// Counts `bytes` if they are a key, and says whether they are.
    fn add(&mut self, bytes: &[u8]) -> bool {
        if PublicKey::from_slice(bytes).is_err() {
            return false;
        }
        *if bytes.len() == 33 {
            &mut self.compressed
        } else {
            &mut self.uncompressed
        } += 1;
        true
    }
}

impl Scanner {
    /// Flags `txout`'s features, and returns its keys.
    pub fn scan(
        &mut self,
        txout: &TxOut,
        output: &ProcessedOutput,
        flags: &mut TxFeatureFlags,
    ) -> Keys {
        flags.insert_type(output.output_type);
        let mut keys = Keys::default();

        match output.output_type {
            OutputType::P2PK33 | OutputType::P2PK65 => {
                // The key's one push, then OP_CHECKSIG.
                let script = txout.script_pubkey.as_bytes();
                if !keys.add(&script[1..script.len() - 1]) {
                    flags.insert(TxFeatureFlags::FAKE_PUBKEY);
                }
                self.p2wsh_count = 0;
            }
            OutputType::P2MS => {
                for push in pushes(&txout.script_pubkey) {
                    if !keys.add(push) || is_burn_key(push) {
                        flags.insert(TxFeatureFlags::FAKE_PUBKEY);
                    }
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
        keys
    }
}

/// The data pushes of `script`, up to the first it can't parse.
fn pushes(script: &Script) -> impl Iterator<Item = &[u8]> {
    script
        .instructions()
        .map_while(Result::ok)
        .filter_map(|instruction| match instruction {
            Instruction::PushBytes(bytes) => Some(bytes.as_bytes()),
            Instruction::Op(_) => None,
        })
}

fn is_burn_key(bytes: &[u8]) -> bool {
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
