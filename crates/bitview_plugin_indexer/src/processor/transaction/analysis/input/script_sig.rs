use bitcoin::{
    Script,
    opcodes::all::{OP_CHECKMULTISIG, OP_CHECKMULTISIGVERIFY, OP_CHECKSIG, OP_CHECKSIGVERIFY},
    script::Instruction,
};
use brk_types::OutputType;

use super::{super::features, ScriptSigFacts};
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

    let mut accurate_sigops = 0;
    let mut first_push = None;
    let mut last_push = None;
    let mut legacy_sigops = 0;
    let mut only_push_bytes = true;
    let mut push_count = 0;
    let mut push_only = true;
    let mut pushnum = None;
    let validated_shape = scan_signatures
        && matches!(
            output_type,
            OutputType::P2PKH | OutputType::P2PK33 | OutputType::P2PK65
        );

    for instruction in script.instructions() {
        match instruction {
            Ok(Instruction::PushBytes(bytes)) => {
                let bytes = bytes.as_bytes();
                first_push.get_or_insert(bytes);
                last_push = Some(bytes);
                push_count += 1;
                pushnum = None;
                if scan_signatures && !validated_shape {
                    features::scan_ecdsa_signature(bytes, flags);
                }
            }
            Ok(Instruction::Op(opcode)) => {
                only_push_bytes = false;
                last_push = None;
                if opcode.to_u8() > 0x60 {
                    push_only = false;
                }
                match opcode {
                    OP_CHECKSIG | OP_CHECKSIGVERIFY => {
                        accurate_sigops += 1;
                        legacy_sigops += 1;
                    }
                    OP_CHECKMULTISIG | OP_CHECKMULTISIGVERIFY => {
                        accurate_sigops += pushnum.unwrap_or(20);
                        legacy_sigops += 20;
                    }
                    _ => pushnum = decode_pushnum(opcode.to_u8()),
                }
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
        accurate_sigops,
        last_push,
        legacy_sigops,
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

#[inline]
pub fn decode_pushnum(opcode: u8) -> Option<usize> {
    (0x51..=0x60)
        .contains(&opcode)
        .then(|| usize::from(opcode - 0x50))
}

#[cfg(test)]
mod tests {
    use bitcoin::ScriptBuf;
    use brk_types::OutputType;

    use super::analyze;
    use crate::TxFeatureFlags;

    pub fn analyze_script(script: &ScriptBuf) -> super::ScriptSigFacts<'_> {
        analyze(
            script,
            OutputType::Unknown,
            false,
            &mut TxFeatureFlags::default(),
        )
    }

    #[test]
    pub fn analyzes_last_push_and_push_only_together() {
        let pushed = ScriptBuf::from_hex("03616263").unwrap();
        let facts = analyze_script(&pushed);
        assert_eq!(facts.last_push, Some(b"abc".as_slice()));
        assert!(facts.push_only);

        let followed_by_checksig = ScriptBuf::from_hex("03616263ac").unwrap();
        let facts = analyze_script(&followed_by_checksig);
        assert_eq!(facts.last_push, None);
        assert!(!facts.push_only);

        let push_number = ScriptBuf::from_hex("51").unwrap();
        let facts = analyze_script(&push_number);
        assert_eq!(facts.last_push, None);
        assert!(facts.push_only);
    }

    #[test]
    pub fn counts_legacy_and_accurate_sigops_together() {
        let script = ScriptBuf::from_hex("52aeac").unwrap();
        let facts = analyze_script(&script);

        assert_eq!(facts.legacy_sigops, 21);
        assert_eq!(facts.accurate_sigops, 3);
    }

    #[test]
    pub fn records_validated_p2pkh_and_p2pk_sighashes() {
        let signature = [0x30, 0x06, 0x02, 0x01, 0x01, 0x02, 0x01, 0x01, 0x01];
        let compressed_key = [0x02; 33];
        let uncompressed_key = [0x04; 65];
        let mut scripts = Vec::new();

        for public_key in [compressed_key.as_slice(), uncompressed_key.as_slice()] {
            let mut bytes = Vec::with_capacity(signature.len() + public_key.len() + 2);
            bytes.push(signature.len() as u8);
            bytes.extend(signature);
            bytes.push(public_key.len() as u8);
            bytes.extend(public_key);
            scripts.push((ScriptBuf::from_bytes(bytes), OutputType::P2PKH, public_key));
        }

        let mut p2pk = Vec::with_capacity(signature.len() + 1);
        p2pk.push(signature.len() as u8);
        p2pk.extend(signature);
        scripts.push((
            ScriptBuf::from_bytes(p2pk),
            OutputType::P2PK33,
            signature.as_slice(),
        ));

        for (script, output_type, last_push) in scripts {
            let mut flags = TxFeatureFlags::default();
            let facts = analyze(&script, output_type, true, &mut flags);

            assert_eq!(facts.accurate_sigops, 0);
            assert_eq!(facts.last_push, Some(last_push));
            assert_eq!(facts.legacy_sigops, 0);
            assert!(facts.push_only);
            assert!(flags.is_set(TxFeatureFlags::SIGHASH_ALL));
        }
    }

    #[test]
    pub fn falls_back_for_noncanonical_p2pkh_shape() {
        let script = ScriptBuf::from_hex(
            "000930060201010201010121020000000000000000000000000000000000000000000000000000000000000000",
        )
        .unwrap();
        let mut flags = TxFeatureFlags::default();

        analyze(&script, OutputType::P2PKH, true, &mut flags);

        assert!(flags.is_set(TxFeatureFlags::SIGHASH_ALL));
    }
}
