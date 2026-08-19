"""GET /api/mempool/txids"""

from _lib import show


HEX = set("0123456789abcdef")


def test_mempool_txids_structure(brk, mempool):
    """Txid list must be a non-empty array on both servers."""
    path = "/api/mempool/txids"
    b = brk.get_mempool_txids()
    m = mempool.get_json(path)
    show("GET", path, f"({len(b)} txids)", f"({len(m)} txids)")
    assert isinstance(b, list) and isinstance(m, list)
    assert len(b) > 0, "brk mempool txids list is empty"


def test_mempool_txids_format(brk):
    """Every txid must be a 64-char strict-lowercase hex string."""
    b = brk.get_mempool_txids()
    show("GET", "/api/mempool/txids", f"({len(b)} txids)", "-")
    bad = [t for t in b if not (isinstance(t, str) and len(t) == 64 and set(t) <= HEX)]
    assert not bad, f"{len(bad)} malformed txid(s), e.g. {bad[0] if bad else None!r}"


def test_mempool_txids_unique(brk):
    """No duplicates."""
    b = brk.get_mempool_txids()
    show("GET", "/api/mempool/txids", f"({len(b)} txids)", "-")
    assert len(b) == len(set(b)), (
        f"duplicate txids: {len(b) - len(set(b))} duplicates out of {len(b)}"
    )


def test_mempool_txids_count_matches_summary(brk):
    """`/api/mempool/txids` length must roughly track `/api/mempool`.count.

    The two endpoints are independent reads against a live mempool, so
    arrivals / evictions between fetches cause drift. We assert within
    max(50, count/100) tolerance to absorb normal churn.
    """
    txids = brk.get_mempool_txids()
    summary = brk.get_mempool()
    show("GET", "/api/mempool/txids", f"len={len(txids)}", f"count={summary['count']}")
    assert summary["count"] > 0 and len(txids) > 0
    drift = abs(len(txids) - summary["count"])
    assert drift <= max(50, summary["count"] // 100), (
        f"txids={len(txids)} vs /api/mempool.count={summary['count']} (drift={drift})"
    )
