use bitcoin::{Script, script::Instruction};
use brk_types::OutputType;

use super::{
    super::{features, sigops::SigOpCount},
    ScriptSigFacts,
};
use crate::TxFeatureFlags;

pub fn analyze<'a>(
    script: &'a Script,
    output_type: OutputType,
    scan_signatures: bool,
    flags: &mut TxFeatureFlags,
) -> ScriptSigFacts<'a> {
    if scan_signatures && let Some((signature, last_push)) = direct_push_spend(script, output_type)
    {
        features::record_validated_ecdsa_sighash(signature, flags);
        return ScriptSigFacts {
            accurate_sigops: 0,
            last_push: Some(last_push),
            legacy_sigops: 0,
            push_only: true,
        };
    }

    let mut first_push = None;
    let mut last_push = None;
    let mut only_push_bytes = true;
    let mut push_count = 0;
    let mut push_only = true;
    let mut sigops = SigOpCount::default();
    let validated_shape = scan_signatures
        && matches!(
            output_type,
            OutputType::P2PKH | OutputType::P2PK33 | OutputType::P2PK65
        );

    for instruction in script.instructions() {
        match instruction {
            Ok(instruction @ Instruction::PushBytes(bytes)) => {
                let bytes = bytes.as_bytes();
                first_push.get_or_insert(bytes);
                last_push = Some(bytes);
                push_count += 1;
                sigops.step(instruction);
                if scan_signatures && !validated_shape {
                    features::scan_ecdsa_signature(bytes, flags);
                }
            }
            Ok(instruction @ Instruction::Op(opcode)) => {
                only_push_bytes = false;
                last_push = None;
                if opcode.to_u8() > 0x60 {
                    push_only = false;
                }
                sigops.step(instruction);
            }
            Err(_) => {
                only_push_bytes = false;
                last_push = None;
                push_only = false;
                break;
            }
        }
    }

    if validated_shape {
        let expected_pushes = match output_type {
            OutputType::P2PKH => 2,
            OutputType::P2PK33 | OutputType::P2PK65 => 1,
            _ => unreachable!(),
        };

        if only_push_bytes && push_count == expected_pushes {
            features::record_validated_ecdsa_sighash(first_push.unwrap(), flags);
        } else {
            scan_ecdsa_signatures(script, flags);
        }
    }

    ScriptSigFacts {
        accurate_sigops: sigops.accurate,
        last_push,
        legacy_sigops: sigops.legacy,
        push_only,
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

pub fn scan_ecdsa_signatures(script: &Script, flags: &mut TxFeatureFlags) {
    for instruction in script.instructions() {
        if let Ok(Instruction::PushBytes(bytes)) = instruction {
            features::scan_ecdsa_signature(bytes.as_bytes(), flags);
        }
    }
}
