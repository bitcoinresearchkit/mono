"""GET /api/address/{address}"""

import pytest

from brk_client import BrkError

from _lib import assert_same_structure, show


KNOWN_ADDR_TYPES = {
    "p2pk33", "p2pk65", "p2pkh", "p2sh", "p2wpkh", "p2wsh", "p2tr",
    "p2ms", "opreturn", "p2a", "empty", "unknown",
}

# Static fixtures: stable addresses with known shapes.
STATIC_ADDRS = [
    "1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa",  # genesis coinbase, p2pkh — heavy path
    "12cbQLTFMXRnSzktFkuoG3eHoMeFtpTu3S",  # p2pkh — exercises tx_count divergence
    "3D2oetdNuZUqQHPJmcMDDHYoqkyNVsFk9r",  # p2sh
]

# Satoshi's genesis pubkey (uncompressed). Brk-only: mempool returns 400.
SATOSHI_GENESIS_PUBKEY = (
    "04678afdb0fe5548271967f1a67130b7105cd6a828e03909a67962e0ea1f61deb649f6bc3f4cef38c4f35504e51ec112de5c384df7ba0b8d578a4c702b6bf11d5f"
)


def _tx_count_tolerance(m_tx_count: int) -> int:
    """Allow drift between brk's distinct-tx and mempool's output-count semantics."""
    import math
    return max(5, math.ceil(0.05 * m_tx_count))


@pytest.mark.parametrize("addr", STATIC_ADDRS)
def test_address_info_shape(brk, mempool, addr):
    """Typed brk response must structurally match mempool and echo the input address."""
    path = f"/api/address/{addr}"
    b = brk.get_address(addr)
    m = mempool.get_json(path)
    show("GET", path, b, m)
    assert_same_structure(b, m)
    assert b["address"] == addr
    assert "addr_type" in b
    assert "type_index" in b["chain_stats"]
    assert "realized_price" in b["chain_stats"]


def test_address_info_shape_dynamic(brk, mempool, live_addrs):
    """Same shape contract over each live-discovered scriptpubkey type."""
    assert live_addrs, "no live addresses discovered"
    for atype, addr in live_addrs:
        path = f"/api/address/{addr}"
        b = brk.get_address(addr)
        m = mempool.get_json(path)
        show("GET", f"{path}  [{atype}]", b, m)
        assert_same_structure(b, m)
        assert b["address"] == addr


@pytest.mark.parametrize("addr", STATIC_ADDRS)
def test_address_chain_stats_match(brk, mempool, addr):
    """Funded/spent counts and sums must match exactly; tx_count tolerated within 5% (min 5)."""
    path = f"/api/address/{addr}"
    b = brk.get_address(addr)["chain_stats"]
    m = mempool.get_json(path)["chain_stats"]
    show("GET", f"{path}  [chain_stats]", b, m)
    for key in ("funded_txo_count", "funded_txo_sum", "spent_txo_count", "spent_txo_sum"):
        assert b[key] == m[key], (
            f"{addr} {key}: brk={b[key]} vs mempool={m[key]}"
        )
    tol = _tx_count_tolerance(m["tx_count"])
    assert abs(b["tx_count"] - m["tx_count"]) <= tol, (
        f"{addr} tx_count drift {abs(b['tx_count'] - m['tx_count'])} > tol {tol}: "
        f"brk={b['tx_count']} vs mempool={m['tx_count']}"
    )


def test_address_chain_stats_match_dynamic(brk, mempool, live_addrs):
    """Same equality/tolerance contract on dynamically discovered addresses."""
    assert live_addrs, "no live addresses discovered"
    for atype, addr in live_addrs:
        path = f"/api/address/{addr}"
        b = brk.get_address(addr)["chain_stats"]
        m = mempool.get_json(path)["chain_stats"]
        show("GET", f"{path}  [chain_stats, {atype}]", b, m)
        for key in ("funded_txo_count", "funded_txo_sum", "spent_txo_count", "spent_txo_sum"):
            assert b[key] == m[key], (
                f"{atype} {addr} {key}: brk={b[key]} vs mempool={m[key]}"
            )
        tol = _tx_count_tolerance(m["tx_count"])
        assert abs(b["tx_count"] - m["tx_count"]) <= tol, (
            f"{atype} {addr} tx_count drift {abs(b['tx_count'] - m['tx_count'])} > tol {tol}: "
            f"brk={b['tx_count']} vs mempool={m['tx_count']}"
        )


@pytest.mark.parametrize("addr", STATIC_ADDRS)
def test_address_brk_extras(brk, addr):
    """Brk-only extras must be coherent: known addr_type, non-negative type_index/realized_price."""
    b = brk.get_address(addr)
    assert b["addr_type"] in KNOWN_ADDR_TYPES, (
        f"unknown addr_type {b['addr_type']!r} for {addr}"
    )
    cs = b["chain_stats"]
    assert cs["type_index"] >= 0, f"negative type_index for {addr}: {cs['type_index']}"
    assert cs["realized_price"] >= 0, (
        f"negative realized_price for {addr}: {cs['realized_price']}"
    )
    if cs["tx_count"] == 0:
        assert cs["realized_price"] == 0, (
            f"unfunded address {addr} must have realized_price=0, got {cs['realized_price']}"
        )


def test_address_invalid(brk):
    """Garbage input must produce a BrkError carrying HTTP 400."""
    with pytest.raises(BrkError) as exc_info:
        brk.get_address("abc")
    assert exc_info.value.status == 400, (
        f"expected status=400, got {exc_info.value.status}"
    )


def test_address_pubkey_as_address(brk):
    """Brk-only: hex-encoded pubkey is accepted as a P2PK address."""
    b = brk.get_address(SATOSHI_GENESIS_PUBKEY)
    assert b["addr_type"] == "p2pk65", f"expected p2pk65, got {b['addr_type']!r}"
    assert b["chain_stats"]["funded_txo_count"] >= 1, (
        f"genesis pubkey must have at least one funded output, got "
        f"{b['chain_stats']['funded_txo_count']}"
    )
