use bitcoin::{Script, script::Instruction};
use brk_types::{OutputType, SigOpCount};

use super::{super::scan::TxScan, ScriptSigFacts};

/// A scriptSig, its every data push handed to `scan` by its place.
pub fn analyze<'a>(
    script: &'a Script,
    output_type: OutputType,
    scan: &mut TxScan<'_>,
) -> ScriptSigFacts<'a> {
    if let Some((signature, last_push)) = direct_push_spend(script, output_type) {
        // P2PK's one push, or P2PKH's signature and key.
        scan.ecdsa(signature);
        if output_type == OutputType::P2PKH {
            scan.input_pubkey(last_push);
        }
        return ScriptSigFacts {
            accurate_sigops: 0,
            last_push: Some(last_push),
            legacy_sigops: 0,
            push_only: true,
        };
    }

    let mut last_push = None;
    // The latest data push, handed on once the next shows whether it was the last.
    let mut pending = None;
    let mut push_only = true;
    let mut sigops = SigOpCount::default();

    for instruction in script.instructions() {
        match instruction {
            Ok(instruction @ Instruction::PushBytes(bytes)) => {
                let bytes = bytes.as_bytes();
                last_push = Some(bytes);
                if let Some(previous) = pending.replace(bytes) {
                    push(scan, output_type, previous, false);
                }
                sigops.step(instruction);
            }
            Ok(instruction @ Instruction::Op(opcode)) => {
                last_push = None;
                if opcode.to_u8() > 0x60 {
                    push_only = false;
                }
                sigops.step(instruction);
            }
            Err(_) => {
                last_push = None;
                push_only = false;
                break;
            }
        }
    }
    if let Some(last) = pending {
        push(scan, output_type, last, true);
    }

    ScriptSigFacts {
        accurate_sigops: sigops.accurate,
        last_push,
        legacy_sigops: sigops.legacy,
        push_only,
    }
}

/// A data push: a P2SH spend's last is its redeem script, which consensus requires on top of a
/// push-only scriptSig; a P2PKH spend's pushes are each read as a signature or a key by their
/// shape, as opcodes may reorder them; any other is an argument.
fn push(scan: &mut TxScan<'_>, spent: OutputType, bytes: &[u8], is_last: bool) {
    match spent {
        OutputType::P2SH if is_last => {}
        OutputType::P2PKH => {
            scan.ecdsa(bytes);
            scan.input_pubkey(bytes);
        }
        _ => scan.ecdsa(bytes),
    }
}

pub fn direct_push_spend(script: &Script, output_type: OutputType) -> Option<(&[u8], &[u8])> {
    let bytes = script.as_bytes();
    let signature_len = usize::from(*bytes.first()?);
    if !(1..=75).contains(&signature_len) {
        return None;
    }

    let signature_end = 1 + signature_len;
    let signature = bytes.get(1..signature_end)?;

    match output_type {
        OutputType::P2PK33 | OutputType::P2PK65 => {
            (signature_end == bytes.len()).then_some((signature, signature))
        }
        OutputType::P2PKH => {
            let public_key_len = usize::from(*bytes.get(signature_end)?);
            if !matches!(public_key_len, 33 | 65) {
                return None;
            }

            let public_key_start = signature_end + 1;
            let public_key_end = public_key_start + public_key_len;
            (public_key_end == bytes.len())
                .then(|| (signature, &bytes[public_key_start..public_key_end]))
        }
        _ => None,
    }
}
