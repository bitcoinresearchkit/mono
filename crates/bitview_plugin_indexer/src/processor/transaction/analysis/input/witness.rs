use bitcoin::{
    Script, Witness,
    opcodes::all::OP_IF,
    script::Instruction,
    taproot::{LeafVersion, TAPROOT_ANNEX_PREFIX, TAPROOT_CONTROL_BASE_SIZE, TAPROOT_LEAF_MASK},
};
use brk_types::OutputType;

use super::{super::features, WitnessFacts};
use crate::TxFeatureFlags;

pub fn analyze<'a>(
    witness: &'a Witness,
    output_type: OutputType,
    flags: &mut TxFeatureFlags,
) -> WitnessFacts<'a> {
    match output_type {
        OutputType::P2TR => analyze_taproot(witness, flags),
        OutputType::P2WPKH => analyze_p2wpkh(witness, flags),
        _ => analyze_ecdsa(witness, flags),
    }
}

pub fn analyze_p2wpkh<'a>(witness: &'a Witness, flags: &mut TxFeatureFlags) -> WitnessFacts<'a> {
    let mut stack = witness.iter();
    let signature = stack.next().unwrap();
    let public_key = stack.next().unwrap();
    debug_assert!(stack.next().is_none());
    features::record_validated_ecdsa_sighash(signature, flags);

    WitnessFacts {
        has_annex: false,
        last: Some(public_key),
        leaf_version: None,
        max_argument_bytes: signature.len(),
        stack_items: 2,
    }
}

pub fn analyze_ecdsa<'a>(witness: &'a Witness, flags: &mut TxFeatureFlags) -> WitnessFacts<'a> {
    let stack_items = witness.len();
    let mut last = None;
    let mut max_argument_bytes = 0;

    for (index, item) in witness.iter().enumerate() {
        features::scan_ecdsa_signature(item, flags);
        if index + 1 < stack_items {
            max_argument_bytes = max_argument_bytes.max(item.len());
        }
        last = Some(item);
    }

    WitnessFacts {
        has_annex: false,
        last,
        leaf_version: None,
        max_argument_bytes,
        stack_items,
    }
}

pub fn analyze_taproot<'a>(witness: &'a Witness, flags: &mut TxFeatureFlags) -> WitnessFacts<'a> {
    let len = witness.len();
    if len == 1 {
        let signature = witness.last().unwrap();
        features::record_validated_taproot_sighash(signature, flags);
        return WitnessFacts {
            has_annex: false,
            last: Some(signature),
            leaf_version: None,
            max_argument_bytes: signature.len(),
            stack_items: 1,
        };
    }

    let last = witness.last();
    let has_annex = len > 1 && last.is_some_and(|item| item.first() == Some(&TAPROOT_ANNEX_PREFIX));
    if has_annex {
        flags.insert(TxFeatureFlags::ANNEX);
    }

    let stack_items = len - usize::from(has_annex);
    let argument_count = if stack_items == 1 {
        1
    } else {
        stack_items.saturating_sub(2)
    };
    let script_index = (stack_items >= 2).then(|| stack_items - 2);
    let control_index = (stack_items >= 2).then(|| stack_items - 1);
    let is_key_path = stack_items == 1;
    let mut leaf_version = None;
    let mut max_argument_bytes = 0;

    for (index, item) in witness.iter().enumerate() {
        if index < argument_count {
            if is_key_path {
                features::record_validated_taproot_sighash(item, flags);
            } else {
                features::scan_taproot_signature(item, flags);
            }
            max_argument_bytes = max_argument_bytes.max(item.len());
        } else if Some(index) == script_index {
            if has_inscription_envelope(Script::from_bytes(item)) {
                flags.insert(TxFeatureFlags::INSCRIPTION);
            }
        } else if Some(index) == control_index && item.len() >= TAPROOT_CONTROL_BASE_SIZE {
            leaf_version = LeafVersion::from_consensus(item[0] & TAPROOT_LEAF_MASK).ok();
        }
    }

    WitnessFacts {
        has_annex,
        last,
        leaf_version,
        max_argument_bytes,
        stack_items,
    }
}

pub fn has_inscription_envelope(script: &Script) -> bool {
    let mut state = 0;
    for instruction in script.instructions() {
        state = match (state, instruction) {
            (0, Ok(Instruction::PushBytes(bytes))) if bytes.is_empty() => 1,
            (1, Ok(Instruction::Op(OP_IF))) => 2,
            (2, Ok(Instruction::PushBytes(bytes))) if bytes.as_bytes() == b"ord" => return true,
            (_, Ok(Instruction::PushBytes(bytes))) if bytes.is_empty() => 1,
            _ => 0,
        };
    }
    false
}

#[cfg(test)]
mod tests {
    use bitcoin::{ScriptBuf, TxIn, Witness, taproot::TAPROOT_ANNEX_PREFIX};
    use brk_types::OutputType;

    use super::{analyze, has_inscription_envelope};
    use crate::TxFeatureFlags;

    #[test]
    pub fn recognizes_only_ord_envelopes() {
        let inscription = ScriptBuf::from_hex("0063036f726468").unwrap();
        let generic_envelope = ScriptBuf::from_hex("006303666f6f68").unwrap();

        assert!(has_inscription_envelope(&inscription));
        assert!(!has_inscription_envelope(&generic_envelope));
    }

    #[test]
    pub fn reads_tapscript_before_control_block_and_annex() {
        let script = ScriptBuf::from_hex("0063036f726468").unwrap();
        let control_block = [0xc0; 33];
        let annex = [TAPROOT_ANNEX_PREFIX, 0x01];
        let input = TxIn {
            witness: Witness::from_slice(&[
                [0x01].as_slice(),
                script.as_bytes(),
                control_block.as_slice(),
                annex.as_slice(),
            ]),
            ..TxIn::default()
        };
        let mut flags = TxFeatureFlags::default();

        let facts = analyze(&input.witness, OutputType::P2TR, &mut flags);

        assert!(facts.has_annex);
        assert!(flags.is_set(TxFeatureFlags::ANNEX));
        assert!(flags.is_set(TxFeatureFlags::INSCRIPTION));
    }

    #[test]
    pub fn reads_fixed_shape_p2wpkh_once() {
        let signature = [0x01; 71];
        let public_key = [0x02; 33];
        let witness = Witness::from_slice(&[signature.as_slice(), public_key.as_slice()]);
        let mut flags = TxFeatureFlags::default();

        let facts = analyze(&witness, OutputType::P2WPKH, &mut flags);

        assert_eq!(facts.last, Some(public_key.as_slice()));
        assert_eq!(facts.max_argument_bytes, signature.len());
        assert_eq!(facts.stack_items, 2);
        assert!(flags.is_set(TxFeatureFlags::SIGHASH_ALL));
    }

    #[test]
    pub fn reads_taproot_key_path_once() {
        let signature = [0x00; 64];
        let witness = Witness::from_slice(&[signature]);
        let mut flags = TxFeatureFlags::default();

        let facts = analyze(&witness, OutputType::P2TR, &mut flags);

        assert_eq!(facts.last, Some(signature.as_slice()));
        assert_eq!(facts.max_argument_bytes, signature.len());
        assert_eq!(facts.stack_items, 1);
        assert!(flags.is_set(TxFeatureFlags::SIGHASH_DEFAULT));
    }
}
