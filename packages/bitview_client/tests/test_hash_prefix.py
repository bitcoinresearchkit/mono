from bitview_client import BitviewClient, address_payload_hash_prefix


VECTORS = (
    (bytes([0x4E, 0x73]), "58101afa51a1ecfd"),
    (bytes(range(20)), "c3327ecb8ae1ff23"),
    (bytes(range(32)), "c0186990f026b180"),
    (bytes(range(65)), "0d4b77027ae7d700"),
)


def test_address_payload_hash_prefix_vectors():
    for payload, expected in VECTORS:
        assert address_payload_hash_prefix(payload, 16) == expected
        assert BitviewClient.address_payload_hash_prefix(payload, 8) == expected[:8]
