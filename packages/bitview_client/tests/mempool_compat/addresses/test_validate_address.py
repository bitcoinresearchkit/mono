"""GET /api/v1/validate-address/{address}"""

import pytest

from _lib import assert_same_structure, assert_same_values, show


VALID_ADDRS = [
    ("p2pkh-genesis", "1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa"),
    ("p2sh", "3J98t1WpEZ73CNmQviecrnyiWrnqRhWNLy"),
    ("p2wpkh", "bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4"),
    ("p2wsh", "bc1qrp33g0q5c5txsp9arysrx4k6zdkfs4nce4xj0gdcccefvpysxf3qccfmv3"),
    ("p2tr", "bc1p0xlxvlhemja6c4dqv22uapctqupfhlxm9h8z3k2e72q4k9hcz7vqzk5jj0"),
]

INVALID_ADDRS = [
    ("garbage", "notanaddress123"),
    ("bad-checksum-p2pkh", "1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNb"),
    ("bad-checksum-p2sh", "3J98t1WpEZ73CNmQviecrnyiWrnqRhWNLz"),
    ("bad-checksum-p2wpkh", "bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t5"),
    ("wrong-network-bech32", "tb1qw508d6qejxtdg4y5r3zarvary0c5xw7kxpjzsx"),
    ("mixed-case-bech32", "bc1QRP33g0q5c5txsp9arysrx4k6zdkfs4nce4xj0gdcccefvpysxf3qccfmv3"),
]

# Satoshi's genesis-coinbase pubkey: brk validates this as p2pk; mempool.space
# rejects all raw-pubkey-hex inputs. Documents the intentional brk superset.
GENESIS_PUBKEY = (
    "04678afdb0fe5548271967f1a67130b7105cd6a828e03909a67962e0ea1f61deb6"
    "49f6bc3f4cef38c4f35504e51ec112de5c384df7ba0b8d578a4c702b6bf11d5f"
)


@pytest.mark.parametrize("kind,addr", VALID_ADDRS, ids=[k for k, _ in VALID_ADDRS])
def test_validate_address_static_valid(brk, mempool, kind, addr):
    """Well-known addresses across all script types must validate identically."""
    path = f"/api/v1/validate-address/{addr}"
    b = brk.validate_address(addr)
    m = mempool.get_json(path)
    show("GET", f"{path}  [{kind}]", b, m)
    assert b["isvalid"] is True
    assert_same_values(b, m)


def test_validate_address_discovered(brk, mempool, live_addrs):
    """Validation of each live-discovered scriptpubkey type must match exactly."""
    for atype, addr in live_addrs:
        path = f"/api/v1/validate-address/{addr}"
        b = brk.validate_address(addr)
        m = mempool.get_json(path)
        show("GET", f"{path}  [{atype}]", b, m)
        assert b["isvalid"] is True
        assert_same_values(b, m)


@pytest.mark.parametrize("kind,addr", INVALID_ADDRS, ids=[k for k, _ in INVALID_ADDRS])
def test_validate_address_invalid(brk, mempool, kind, addr):
    """Invalid addresses produce isvalid=false; structure must match (error strings differ by impl)."""
    path = f"/api/v1/validate-address/{addr}"
    b = brk.validate_address(addr)
    m = mempool.get_json(path)
    show("GET", f"{path}  [{kind}]", b, m)
    assert b["isvalid"] is False
    assert m["isvalid"] is False
    assert_same_structure(b, m)


def test_validate_address_pubkey_hex_brk_only(brk, mempool):
    """Raw pubkey hex: brk accepts as p2pk (superset); mempool.space rejects (non-2xx or no isvalid:true)."""
    path = f"/api/v1/validate-address/{GENESIS_PUBKEY}"
    b = brk.validate_address(GENESIS_PUBKEY)
    m_resp = mempool.get_raw(path)
    show("GET", path, b, f"<HTTP {m_resp.status_code}> {m_resp.text[:200]}")
    assert b["isvalid"] is True, "brk must accept raw pubkey hex as p2pk"
    assert b.get("isscript") is False
    assert b.get("iswitness") is False
    if 200 <= m_resp.status_code < 300:
        try:
            m = m_resp.json()
        except ValueError:
            m = None
        assert not (isinstance(m, dict) and m.get("isvalid") is True), (
            "mempool.space must not validate raw pubkey hex as a real address"
        )
