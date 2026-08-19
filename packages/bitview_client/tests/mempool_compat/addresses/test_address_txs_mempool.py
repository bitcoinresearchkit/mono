"""GET /api/address/{address}/txs/mempool"""

import pytest

from bitview_client import BitviewError

from _lib import assert_same_structure, show


def test_address_txs_mempool_shape_dynamic(brk, mempool, live_addrs):
    """Shape contract over each live-discovered scriptpubkey type."""
    assert live_addrs, "no live addresses discovered"
    for atype, addr in live_addrs:
        path = f"/api/address/{addr}/txs/mempool"
        b = brk.get_address_mempool_txs(addr)
        m = mempool.get_json(path)
        show("GET", f"{path}  [{atype}]", f"({len(b)} txs)", f"({len(m)} txs)")
        assert isinstance(b, list) and isinstance(m, list)
        if b and m:
            assert_same_structure(b[0], m[0])


def test_address_txs_mempool_limit(brk, live_addrs):
    """Hard cap of 50 mempool txs per call."""
    for _atype, addr in live_addrs:
        b = brk.get_address_mempool_txs(addr)
        assert len(b) <= 50, f"{addr} returned {len(b)} txs, exceeds 50-cap"


def test_address_txs_mempool_all_unconfirmed(brk, live_addrs):
    """Every entry must have status.confirmed == False."""
    for _atype, addr in live_addrs:
        b = brk.get_address_mempool_txs(addr)
        confirmed = [t for t in b if t["status"]["confirmed"]]
        assert not confirmed, (
            f"{addr}: {len(confirmed)} confirmed tx(s) returned by /txs/mempool: "
            f"{[t['txid'] for t in confirmed[:3]]}"
        )


def test_address_txs_mempool_unique_txids(brk, live_addrs):
    """No duplicate txids within a single response."""
    for _atype, addr in live_addrs:
        b = brk.get_address_mempool_txs(addr)
        txids = [t["txid"] for t in b]
        assert len(txids) == len(set(txids)), f"{addr}: duplicate txids in response"


def test_address_txs_mempool_invalid(brk):
    """Garbage input must produce a BitviewError carrying HTTP 400."""
    with pytest.raises(BitviewError) as exc_info:
        brk.get_address_mempool_txs("abc")
    assert exc_info.value.status == 400, (
        f"expected status=400, got {exc_info.value.status}"
    )
