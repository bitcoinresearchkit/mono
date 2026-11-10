use bitcoin::{
    Script, Witness,
    opcodes::all::OP_IF,
    script::Instruction,
    taproot::{LeafVersion, TAPROOT_ANNEX_PREFIX, TAPROOT_CONTROL_BASE_SIZE, TAPROOT_LEAF_MASK},
};
use brk_types::OutputType;

use super::{super::scan::TxScan, WitnessFacts, revealed::Revealed};
use crate::TxFeatureFlags;

/// A witness, read by the rules of `output_type`, the program it spends. A v0 program's items but
/// the last are its arguments, signatures among them: P2WPKH's last is its key, P2WSH's its
/// script, walked. A future program's items mean nothing yet.
pub fn analyze<'a>(
    witness: &'a Witness,
    output_type: OutputType,
    scan: &mut TxScan<'_>,
) -> WitnessFacts<'a> {
    match output_type {
        OutputType::P2TR => analyze_taproot(witness, scan),
        // Its only valid witness since SegWit activation; before it, a v0 program could be spent
        // without one (none on mainnet).
        OutputType::P2WPKH if witness.len() == 2 => {
            let facts = read(witness, |signature| scan.ecdsa(signature));
            scan.input_pubkey(&witness[1]);
            facts
        }
        OutputType::P2WSH => {
            let facts = read(witness, |argument| scan.ecdsa(argument));
            let script = facts.last.map_or(Revealed::NONE, |script| {
                Revealed::of(Script::from_bytes(script), scan)
            });
            WitnessFacts { script, ..facts }
        }
        _ => read(witness, |_| {}),
    }
}

/// A witness whose items but the last are its arguments, each handed to `argument`.
fn read<'a>(witness: &'a Witness, mut argument: impl FnMut(&'a [u8])) -> WitnessFacts<'a> {
    let stack_items = witness.len();
    let mut max_argument_bytes = 0;
    for item in witness.iter().take(stack_items.saturating_sub(1)) {
        argument(item);
        max_argument_bytes = max_argument_bytes.max(item.len());
    }
    WitnessFacts {
        has_annex: false,
        last: witness.last(),
        leaf_version: None,
        max_argument_bytes,
        stack_items,
        script: Revealed::NONE,
        inscription: false,
    }
}

fn analyze_taproot<'a>(witness: &'a Witness, scan: &mut TxScan<'_>) -> WitnessFacts<'a> {
    let len = witness.len();
    let last = witness.last();
    let has_annex = len > 1 && last.is_some_and(|item| item.first() == Some(&TAPROOT_ANNEX_PREFIX));
    if has_annex {
        scan.flags.insert(TxFeatureFlags::ANNEX);
    }

    let stack_items = len - usize::from(has_annex);
    let is_key_path = stack_items == 1;
    // A script path's control block comes first: only a tapscript leaf has signatures or
    // inscriptions to look for, a future leaf version's arguments and script mean nothing yet.
    let leaf_version = stack_items
        .checked_sub(1)
        .filter(|_| !is_key_path)
        .and_then(|control_index| witness.nth(control_index))
        .filter(|control| control.len() >= TAPROOT_CONTROL_BASE_SIZE)
        .and_then(|control| LeafVersion::from_consensus(control[0] & TAPROOT_LEAF_MASK).ok());
    let is_tapscript = leaf_version == Some(LeafVersion::TapScript);
    let has_signatures = is_key_path || is_tapscript;
    let argument_count = if is_key_path {
        1
    } else {
        stack_items.saturating_sub(2)
    };
    let mut max_argument_bytes = 0;

    for item in witness.iter().take(argument_count) {
        if has_signatures {
            scan.schnorr(item);
        }
        max_argument_bytes = max_argument_bytes.max(item.len());
    }

    let inscription = is_tapscript
        && witness
            .nth(stack_items - 2)
            .is_some_and(|script| {
                walk_envelope::<true>(Script::from_bytes(script), |opcode| scan.opcode(opcode))
            });
    if inscription {
        scan.flags.insert(TxFeatureFlags::INSCRIPTION);
    }

    WitnessFacts {
        has_annex,
        last,
        leaf_version,
        max_argument_bytes,
        stack_items,
        script: Revealed::NONE,
        inscription,
    }
}

/// Any real OP_0 OP_IF prefix at instruction boundaries, ignoring protocol payload and leaf
/// version, as the summary's broad envelope heuristic requires. No signature/key analysis.
/// Only call for a known P2TR prevout; witness shape alone does not identify its script type.
pub fn has_taproot_envelope(witness: &Witness) -> bool {
    let has_annex = witness.len() > 1
        && witness.last().is_some_and(|last| last.first() == Some(&TAPROOT_ANNEX_PREFIX));
    let stack_items = witness.len() - usize::from(has_annex);
    stack_items.checked_sub(2)
        .and_then(|script| witness.nth(script))
        .is_some_and(|script| walk_envelope::<false>(Script::from_bytes(script), |_| {}))
}

/// The indexer's tapscript walk counts every parsed opcode, including those after an envelope.
/// The broad query-only detector has no counters and can stop at its first match.
fn walk_envelope<const REQUIRE_ORD: bool>(
    script: &Script,
    mut opcode: impl FnMut(bitcoin::opcodes::Opcode),
) -> bool {
    let mut state = 0;
    for instruction in script.instructions() {
        let Ok(instruction) = instruction else {
            break;
        };
        if let Instruction::Op(op) = instruction {
            opcode(op);
        }
        state = match (state, instruction) {
            (3, _) => 3,
            (0, Instruction::PushBytes(bytes)) if bytes.is_empty() => 1,
            (1, Instruction::Op(OP_IF)) if !REQUIRE_ORD => return true,
            (1, Instruction::Op(OP_IF)) => 2,
            (2, Instruction::PushBytes(bytes)) if bytes.as_bytes() == b"ord" => 3,
            (_, Instruction::PushBytes(bytes)) if bytes.is_empty() => 1,
            _ => 0,
        };
    }
    state == 3
}

#[cfg(test)]
mod tests;
