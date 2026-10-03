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
