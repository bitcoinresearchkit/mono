use super::*;

fn has_inscription_envelope(script: &Script) -> bool {
    walk_envelope::<true>(script, |_| {})
}

#[test]
fn existing_ord_classifier_still_requires_its_payload() {
    assert!(has_inscription_envelope(Script::from_bytes(&[0, 0x63, 3, b'o', b'r', b'd'])));
    assert!(has_inscription_envelope(Script::from_bytes(&[0, 0, 0x63, 3, b'o', b'r', b'd'])));
    assert!(!has_inscription_envelope(Script::from_bytes(&[0, 0x63])));
    assert!(!has_inscription_envelope(Script::from_bytes(&[0, 0x63, 3, b'f', b'o', b'o'])));
    assert!(!has_inscription_envelope(Script::from_bytes(&[2, 0, 0x63, 3, b'o', b'r', b'd'])));
    // The old parser accepts nonminimal data pushes; this refactor preserves that behavior.
    assert!(has_inscription_envelope(Script::from_bytes(&[0, 0x63, 0x4c, 3, b'o', b'r', b'd'])));
}

#[test]
fn indexer_walk_counts_opcodes_after_the_inscription_and_stops_at_bad_pushes() {
    let bytes = [0, 0x63, 3, b'o', b'r', b'd', 0x68, 0x75, 0x4c];
    let mut opcodes = Vec::new();
    assert!(walk_envelope::<true>(Script::from_bytes(&bytes), |op| opcodes.push(op.to_u8())));
    assert_eq!(opcodes, [0x63, 0x68, 0x75]);
}
