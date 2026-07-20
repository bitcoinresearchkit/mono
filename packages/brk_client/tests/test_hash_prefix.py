import pytest

from brk_client import BrkClient, address_payload_hash_prefix


VECTORS = (
    (bytes([0x4E, 0x73]), "58101afa51a1ecfd"),
    (bytes(range(20)), "c3327ecb8ae1ff23"),
    (bytes(range(32)), "c0186990f026b180"),
    (bytes(range(65)), "0d4b77027ae7d700"),
)


def test_address_payload_hash_prefix_vectors():
    for payload, expected in VECTORS:
        assert address_payload_hash_prefix(payload, 16) == expected
        assert BrkClient.address_payload_hash_prefix(payload, 8) == expected[:8]


def test_address_payload_hash_prefix_validation():
    with pytest.raises(ValueError, match="non-empty"):
        address_payload_hash_prefix(b"", 16)
    with pytest.raises(ValueError, match="at most 65"):
        address_payload_hash_prefix(bytes(range(66)), 16)
    with pytest.raises(ValueError, match="1 to 16"):
        address_payload_hash_prefix(b"\x01\x02", 0)


def test_address_payload_hash_prefix_match_validation():
    client = BrkClient("http://127.0.0.1:0")
    client.get_address_hash_prefix_matches = lambda addr_type, prefix: {
        "addr_type": addr_type,
        "prefix": prefix,
        "truncated": False,
        "addresses": [],
    }

    assert client.get_address_payload_hash_prefix_matches("p2pkh", bytes(range(20)), 8) == {
        "addr_type": "p2pkh",
        "prefix": "c3327ecb",
        "truncated": False,
        "addresses": [],
    }

    for addr_type, length in (("p2pk33", 33), ("p2pk65", 65)):
        payload = bytes(range(length))
        assert client.get_address_payload_hash_prefix_matches(addr_type, payload, 8) == {
            "addr_type": addr_type,
            "prefix": address_payload_hash_prefix(payload, 8),
            "truncated": False,
            "addresses": [],
        }

    with pytest.raises(ValueError, match="p2pkh address payload length 20 bytes"):
        client.get_address_payload_hash_prefix_matches("p2pkh", b"\x01\x02", 8)
    with pytest.raises(ValueError, match="p2pk33 address payload length 33 bytes"):
        client.get_address_payload_hash_prefix_matches("p2pk33", bytes(65), 8)
    with pytest.raises(ValueError, match="p2pk65 address payload length 65 bytes"):
        client.get_address_payload_hash_prefix_matches("p2pk65", bytes(33), 8)
    with pytest.raises(ValueError, match="Unsupported address type"):
        client.get_address_payload_hash_prefix_matches("p2pk", bytes(33), 8)
    with pytest.raises(ValueError, match="Unsupported address type"):
        client.get_address_payload_hash_prefix_matches("opreturn", b"\x01\x02", 8)
