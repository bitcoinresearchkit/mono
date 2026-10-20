use bitcoin::ScriptBuf;
use bitview_primitives::OpReturnKind;

use crate::processor::txout::op_return::analyze;

/// `OP_RETURN` and one push of `payload`.
fn kind(payload: &[u8]) -> OpReturnKind {
    let mut script = vec![0x6a, payload.len() as u8];
    script.extend_from_slice(payload);
    analyze(&ScriptBuf::from_bytes(script)).kind
}

/// Payload shapes as they appear on chain.
#[test]
fn op_return_protocol_markers() {
    let hash = [0x5a; 64];
    let factom = [b"Fa".as_slice(), &[0; 6], &hash[..32]].concat();
    assert_eq!(kind(&factom), OpReturnKind::Factom);
    assert_eq!(kind(b"ASCRIBESPOOL01PIECE"), OpReturnKind::Ascribe);
    for stampery in [
        [b"S1".as_slice(), &hash[..32]].concat(),
        [b"S5".as_slice(), &[0x9b, 0x73], &hash[..32]].concat(),
        [b"S5".as_slice(), &[0x9b, 0x73], &hash[..64]].concat(),
    ] {
        assert_eq!(kind(&stampery), OpReturnKind::Stampery);
    }
    // An OpenTimestamps calendar commitment is a bare hash.
    assert_eq!(kind(&hash[..32]), OpReturnKind::BareHash);
}
