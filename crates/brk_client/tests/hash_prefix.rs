use brk_client::{
    BrkClient, OutputType, address_hash_prefix, address_payload_hash_prefix, decode_address_payload,
};

#[test]
fn address_payload_hash_prefix_vectors() {
    let vectors = [
        (vec![0x4e, 0x73], "58101afa51a1ecfd"),
        ((0_u8..20).collect::<Vec<_>>(), "c3327ecb8ae1ff23"),
        ((0_u8..32).collect::<Vec<_>>(), "c0186990f026b180"),
        ((0_u8..65).collect::<Vec<_>>(), "0d4b77027ae7d700"),
    ];

    for (payload, expected) in vectors.iter() {
        assert_eq!(address_payload_hash_prefix(payload, 16).unwrap(), *expected);
        assert_eq!(
            BrkClient::address_payload_hash_prefix(payload, 8).unwrap(),
            &expected[..8]
        );
    }
}

#[test]
fn address_payload_hash_prefix_validation() {
    assert!(address_payload_hash_prefix(&[], 16).is_err());
    assert!(address_payload_hash_prefix(&[0; 66], 16).is_err());
    assert!(address_payload_hash_prefix(&[1, 2], 0).is_err());
    assert!(address_payload_hash_prefix(&[1, 2], 17).is_err());
}

#[test]
fn address_hash_prefix_uses_brk_address_parser() {
    let address = "1BoatSLRHtKNngkdXEeobR76b53LETtpyT";
    let decoded = decode_address_payload(address).unwrap();
    assert_eq!(decoded.addr_type, OutputType::P2PKH);
    assert_eq!(decoded.payload.len(), 20);

    let hashed = address_hash_prefix(address, 8).unwrap();
    assert_eq!(hashed.addr_type, OutputType::P2PKH);
    assert_eq!(
        hashed.prefix,
        address_payload_hash_prefix(&decoded.payload, 8).unwrap()
    );
}

#[test]
fn p2pk_payload_types_are_distinct() {
    let p2pk33 = decode_address_payload(
        "0279be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798",
    )
    .unwrap();
    assert_eq!(p2pk33.addr_type, OutputType::P2PK33);
    assert_eq!(p2pk33.payload.len(), 33);

    let p2pk65 = decode_address_payload(
        "04678afdb0fe5548271967f1a67130b7105cd6a828e03909a67962e0ea1f61deb649f6bc3f4cef38c4f35504e51ec112de5c384df7ba0b8d578a4c702b6bf11d5f",
    )
    .unwrap();
    assert_eq!(p2pk65.addr_type, OutputType::P2PK65);
    assert_eq!(p2pk65.payload.len(), 65);
}
