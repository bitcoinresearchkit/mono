"""GET /api/address/{address}/txs/chain (and /txs/chain/{after_txid})"""

import pytest

from bitview_client import BitviewError

from _lib import assert_same_structure, show


# Heavy active address (chain-tip drift expected, no exact-order assertion)
ACTIVE_ADDR = "1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa"

# Inactive historical addresses — both indexers agree exactly on first-page ordering
STABLE_ADDRS = [
    "12cbQLTFMXRnSzktFkuoG3eHoMeFtpTu3S",  # p2pkh, ~125 txs
    "3D2oetdNuZUqQHPJmcMDDHYoqkyNVsFk9r",  # p2sh, ~5700 txs (heavy pagination)
]

STATIC_ADDRS = [ACTIVE_ADDR] + STABLE_ADDRS


@pytest.mark.parametrize("addr", STATIC_ADDRS)
def test_address_txs_chain_shape(brk, mempool, addr):
    """Typed list response must structurally match mempool; brk's `index` extra is allowed."""
    path = f"/api/address/{addr}/txs/chain"
    b = brk.get_address_confirmed_txs(addr)
    m = mempool.get_json(path)
    show("GET", path, f"({len(b)} txs)", f"({len(m)} txs)")
    assert isinstance(b, list) and isinstance(m, list)
    if b and m:
        assert_same_structure(b[0], m[0])
        assert "index" in b[0], "brk-only `index` field missing"


def test_address_txs_chain_shape_dynamic(brk, mempool, live_addrs):
    """Same shape contract over each live-discovered scriptpubkey type."""
    assert live_addrs, "no live addresses discovered"
    for atype, addr in live_addrs:
        path = f"/api/address/{addr}/txs/chain"
        b = brk.get_address_confirmed_txs(addr)
        m = mempool.get_json(path)
        show("GET", f"{path}  [{atype}]", f"({len(b)} txs)", f"({len(m)} txs)")
        assert isinstance(b, list) and isinstance(m, list)
        if b and m:
            assert_same_structure(b[0], m[0])


@pytest.mark.parametrize("addr", STATIC_ADDRS)
def test_address_txs_chain_all_confirmed(brk, addr):
    """Every entry must have `status.confirmed == True`."""
    b = brk.get_address_confirmed_txs(addr)
    if not b:
        pytest.skip(f"{addr} has no confirmed txs in brk")
    unconfirmed = [t for t in b if not t["status"]["confirmed"]]
    assert not unconfirmed, (
        f"{addr}: {len(unconfirmed)} unconfirmed tx(s) returned: "
        f"{[t['txid'] for t in unconfirmed[:3]]}"
    )


@pytest.mark.parametrize("addr", STATIC_ADDRS)
def test_address_txs_chain_ordering(brk, addr):
    """Heights must be monotonically non-increasing (newest first)."""
    b = brk.get_address_confirmed_txs(addr)
    if not b:
        pytest.skip(f"{addr} has no confirmed txs in brk")
    heights = [t["status"]["block_height"] for t in b]
    assert heights == sorted(heights, reverse=True), (
        f"{addr} not newest-first by height: {heights[:5]}..."
    )


@pytest.mark.parametrize("addr", STATIC_ADDRS)
def test_address_txs_chain_limit(brk, addr):
    """Hard cap of 25 confirmed txs per call."""
    b = brk.get_address_confirmed_txs(addr)
    assert len(b) <= 25, f"{addr} returned {len(b)} txs, exceeds 25-cap"


@pytest.mark.parametrize("addr", STABLE_ADDRS)
def test_address_txs_chain_top_match_stable(brk, mempool, addr):
    """For inactive historical addresses, brk and mempool agree on first-page order."""
    b_txids = [t["txid"] for t in brk.get_address_confirmed_txs(addr)]
    m_txids = [t["txid"] for t in mempool.get_json(f"/api/address/{addr}/txs/chain")]
    assert b_txids == m_txids, (
        f"{addr} first-page txid order diverges:\n"
        f"  brk:     {b_txids[:5]}...\n"
        f"  mempool: {m_txids[:5]}..."
    )


def test_address_txs_chain_pagination(brk, mempool):
    """Path-style pagination must match mempool.space's Esplora-canonical form exactly."""
    addr = "3D2oetdNuZUqQHPJmcMDDHYoqkyNVsFk9r"
    first = brk.get_address_confirmed_txs(addr)
    assert len(first) == 25, f"expected full first page (25), got {len(first)}"
    last_txid = first[-1]["txid"]
    last_height = first[-1]["status"]["block_height"]

    second = brk.get_address_confirmed_txs_after(addr, last_txid)
    assert second, "second page must be non-empty for a 5700-tx address"
    assert len(second) <= 25, f"page 2 exceeds 25-cap: {len(second)}"

    first_txids = {t["txid"] for t in first}
    second_txids = {t["txid"] for t in second}
    assert not (first_txids & second_txids), "pagination must not return overlapping txs"

    for tx in second:
        assert tx["status"]["confirmed"] is True, f"page 2 has unconfirmed tx {tx['txid']}"
        assert tx["status"]["block_height"] <= last_height, (
            f"page 2 tx {tx['txid']} at height {tx['status']['block_height']} "
            f"exceeds page-1 tail height {last_height}"
        )

    # Cross-check against mempool.space's path-style form.
    m_second = mempool.get_json(f"/api/address/{addr}/txs/chain/{last_txid}")
    b_ids = [t["txid"] for t in second]
    m_ids = [t["txid"] for t in m_second]
    assert b_ids == m_ids, (
        f"page-2 order diverges from mempool path-style:\n"
        f"  brk:     {b_ids[:5]}...\n"
        f"  mempool: {m_ids[:5]}..."
    )


def test_address_txs_chain_invalid(brk):
    """Garbage input must produce a BitviewError carrying HTTP 400."""
    with pytest.raises(BitviewError) as exc_info:
        brk.get_address_confirmed_txs("abc")
    assert exc_info.value.status == 400, (
        f"expected status=400, got {exc_info.value.status}"
    )
