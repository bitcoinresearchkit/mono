use bitcoin::{Script, opcodes::all::OP_PUSHNUM_13, script::Instruction};
use bitview_primitives::{Bytes32, OpReturnKind};
use brk_types::SigOpCount;

#[derive(Debug, Clone, Copy)]
pub struct Facts {
    pub kind: OpReturnKind,
    /// Signature opcodes outside a push (0xac/0xae in the payload), which never execute but count
    /// in the transaction's legacy total.
    pub legacy_sigops: u32,
    pub post_op_return_bytes: Bytes32,
}

pub fn analyze(script: &Script) -> Facts {
    let data = &script.as_bytes()[1..];
    let (kind, legacy_sigops) = if data.first().copied() == Some(OP_PUSHNUM_13.to_u8()) {
        (OpReturnKind::Runes, SigOpCount::of(&script[1..]).legacy)
    } else {
        let (prefix, legacy_sigops) = scan(script);
        (classify(data, prefix), legacy_sigops)
    };
    Facts {
        kind,
        legacy_sigops: legacy_sigops as u32,
        post_op_return_bytes: Bytes32::from(data.len()),
    }
}

/// BIP141: a 36-byte push of `aa21a9ed` and the 32-byte commitment, right after `OP_RETURN`.
const WITNESS_COMMITMENT_HEADER: [u8; 5] = [0x24, 0xaa, 0x21, 0xa9, 0xed];

fn classify(data: &[u8], prefix: Option<&[u8]>) -> OpReturnKind {
    if data.len() >= 37 && data.starts_with(&WITNESS_COMMITMENT_HEADER) {
        return OpReturnKind::WitnessCommitment;
    }
    let Some(prefix) = prefix else {
        return OpReturnKind::Empty;
    };

    if prefix.starts_with(b"omni") {
        OpReturnKind::Omni
    } else if prefix.starts_with(b"X2") || prefix.starts_with(b"X1") {
        OpReturnKind::Stacks
    } else if prefix.starts_with(b"id") {
        OpReturnKind::Blockstack
    } else if prefix.starts_with(b"CC") {
        OpReturnKind::Colu
    } else if prefix.starts_with(b"OA\x01\x00") {
        OpReturnKind::OpenAssets
    } else if prefix.starts_with(b"SPK") {
        OpReturnKind::CoinSpark
    } else if prefix.starts_with(b"POET") {
        OpReturnKind::Poet
    } else if prefix.starts_with(b"DOCPROOF") {
        OpReturnKind::Docproof
    } else if prefix.len() == 40 && prefix.starts_with(b"Fa") {
        OpReturnKind::Factom
    } else if prefix.starts_with(b"EW") {
        OpReturnKind::EternityWall
    } else if is_memo(prefix) {
        OpReturnKind::Memo
    } else if prefix.starts_with(b"BP") {
        OpReturnKind::Bitproof
    } else if prefix.starts_with(b"ASCRIBESPOOL") {
        OpReturnKind::Ascribe
    } else if is_stampery(prefix) {
        OpReturnKind::Stampery
    } else if data.len() == 82 {
        OpReturnKind::VeriBlock
    } else if (36..=38).contains(&data.len()) {
        OpReturnKind::Komodo
    } else if matches!(prefix.len(), 20 | 32) {
        OpReturnKind::BareHash
    } else if is_text(prefix) {
        OpReturnKind::Text
    } else {
        OpReturnKind::Unknown
    }
}

fn scan(script: &Script) -> (Option<&[u8]>, usize) {
    let mut first_push = None;
    let mut sigops = SigOpCount::default();

    for instruction in script.instructions().skip(1) {
        let Ok(instruction) = instruction else {
            break;
        };
        if let Instruction::PushBytes(bytes) = instruction
            && first_push.is_none()
            && !bytes.is_empty()
        {
            first_push = Some(bytes.as_bytes());
        }
        sigops.step(instruction);
    }

    (first_push, sigops.legacy)
}

/// A Stampery anchor (its BTA whitepaper): `S`, a version digit, an optional 2-byte server
/// identifier, and a 32- or 64-byte hash.
fn is_stampery(prefix: &[u8]) -> bool {
    matches!(prefix, [b'S', b'1'..=b'9', ..]) && matches!(prefix.len(), 34 | 36 | 66 | 68)
}

fn is_memo(prefix: &[u8]) -> bool {
    prefix.len() >= 2
        && prefix[0] == 0x6d
        && (matches!(prefix[1], 0x01..=0x07) || prefix[1] == 0x0c)
}

fn is_text(prefix: &[u8]) -> bool {
    prefix.len() >= 4
        && prefix
            .iter()
            .filter(|byte| byte.is_ascii_graphic() || **byte == b' ')
            .count()
            * 10
            >= prefix.len() * 9
}
