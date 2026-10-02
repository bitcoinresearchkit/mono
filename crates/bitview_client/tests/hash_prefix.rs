use bitview_client::address_payload_hash_prefix;

#[test]
fn address_payload_hash_prefix_vectors_and_boundaries() {
    let vectors = [
        (vec![0x4e, 0x73], "58101afa51a1ecfd"),
        ((0_u8..20).collect::<Vec<_>>(), "c3327ecb8ae1ff23"),
        ((0_u8..32).collect::<Vec<_>>(), "c0186990f026b180"),
        ((0_u8..65).collect::<Vec<_>>(), "0d4b77027ae7d700"),
    ];

    for (payload, expected) in vectors.iter() {
        assert_eq!(address_payload_hash_prefix(payload, 16).unwrap(), *expected);
        assert_eq!(
            address_payload_hash_prefix(payload, 8).unwrap(),
            &expected[..8]
        );
    }

    assert!(address_payload_hash_prefix(&[], 16).is_err());
    assert!(address_payload_hash_prefix(&[0; 66], 16).is_err());
    assert!(address_payload_hash_prefix(&[1, 2], 0).is_err());
    assert!(address_payload_hash_prefix(&[1, 2], 17).is_err());
}
