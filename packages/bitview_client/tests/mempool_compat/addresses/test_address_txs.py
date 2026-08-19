"""GET /api/address/{address}/txs"""

import pytest

from bitview_client import BitviewError

from _lib import assert_same_structure, show


# Heavy address (recently active) — stresses the 50-cap path; cannot be ordered
# exactly against mempool.space because the two indexers drift at the chain tip.
ACTIVE_ADDR = "1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa"

# Inactive historical addresses — both indexers agree exactly on first-page
# ordering and on pagination.
STABLE_ADDRS = [
    "12cbQLTFMXRnSzktFkuoG3eHoMeFtpTu3S",  # p2pkh, ~125 txs
    "3D2oetdNuZUqQHPJmcMDDHYoqkyNVsFk9r",  # p2sh, ~5700 txs (heavy pagination)
]

STATIC_ADDRS = [ACTIVE_ADDR] + STABLE_ADDRS


@pytest.mark.parametrize("addr", STATIC_ADDRS)
def test_address_txs_shape(brk, mempool, addr):
    """Typed list response must structurally match mempool; brk's `index` extra is allowed."""
    path = f"/api/address/{addr}/txs"
    b = brk.get_address_txs(addr)
    m = mempool.get_json(path)
    show("GET", path, f"({len(b)} txs)", f"({len(m)} txs)")
    assert isinstance(b, list) and isinstance(m, list)
    if b and m:
        assert_same_structure(b[0], m[0])
        assert "index" in b[0], "brk-only `index` field missing"


def test_address_txs_shape_dynamic(brk, mempool, live_addrs):
    """Same shape contract over each live-discovered scriptpubkey type."""
    assert live_addrs, "no live addresses discovered"
    for atype, addr in live_addrs:
        path = f"/api/address/{addr}/txs"
        b = brk.get_address_txs(addr)
        m = mempool.get_json(path)
        show("GET", f"{path}  [{atype}]", f"({len(b)} txs)", f"({len(m)} txs)")
        assert isinstance(b, list) and isinstance(m, list)
        if b and m:
            assert_same_structure(b[0], m[0])


@pytest.mark.parametrize("addr", STATIC_ADDRS)
def test_address_txs_ordering(brk, addr):
    """Response is mempool-prefix (unconfirmed, newest-first) + chain-suffix (confirmed, height-desc)."""
    b = brk.get_address_txs(addr)
    if not b:
        pytest.skip(f"{addr} has no txs in brk")

    confirmed_flags = [tx["status"]["confirmed"] for tx in b]
    assert confirmed_flags == sorted(confirmed_flags), (
        f"{addr}: confirmed flags must be False*..*True* (mempool prefix then chain), got "
        f"{confirmed_flags[:10]}..."
    )

    chain = [tx for tx in b if tx["status"]["confirmed"]]
    heights = [tx["status"]["block_height"] for tx in chain]
    assert heights == sorted(heights, reverse=True), (
        f"{addr} chain segment not newest-first by height: {heights[:5]}..."
    )


@pytest.mark.parametrize("addr", STATIC_ADDRS)
def test_address_txs_limit(brk, addr):
    """Hard cap of 50 entries per call (mempool first, chain fills remainder)."""
    b = brk.get_address_txs(addr)
    assert len(b) <= 50, f"{addr} returned {len(b)} txs, exceeds 50-cap"


@pytest.mark.parametrize("addr", STABLE_ADDRS)
def test_address_txs_top_match_stable(brk, mempool, addr):
    """For inactive historical addresses, the confirmed tail must agree exactly with mempool.space."""
    b_chain = [t["txid"] for t in brk.get_address_txs(addr) if t["status"]["confirmed"]]
    m_chain = [
        t["txid"]
        for t in mempool.get_json(f"/api/address/{addr}/txs")
        if t["status"]["confirmed"]
    ]
    assert b_chain == m_chain, (
        f"{addr} confirmed-tail txid order diverges:\n"
        f"  brk:     {b_chain[:5]}...\n"
        f"  mempool: {m_chain[:5]}..."
    )


def test_address_txs_invalid(brk):
    """Garbage input must produce a BitviewError carrying HTTP 400."""
    with pytest.raises(BitviewError) as exc_info:
        brk.get_address_txs("abc")
    assert exc_info.value.status == 400, (
        f"expected status=400, got {exc_info.value.status}"
    )
