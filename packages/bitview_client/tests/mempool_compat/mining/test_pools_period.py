"""GET /api/v1/mining/pools/{time_period}"""

import pytest

from bitview_client import BitviewError

from _lib import assert_same_structure, show, summary


PERIODS = ["24h", "3d", "1w", "1m", "3m", "6m", "1y", "2y", "3y", "all"]


@pytest.mark.parametrize("period", PERIODS)
def test_mining_pools_by_period_structure(brk, mempool, period):
    """Pool stats envelope must structurally match mempool across all periods."""
    path = f"/api/v1/mining/pools/{period}"
    b = brk.get_pool_stats(period)
    m = mempool.get_json(path)
    show("GET", path, summary(b), summary(m))
    assert_same_structure(b, m)


def test_mining_pools_by_period_invariants(brk):
    """A single deep-period sanity pass on `1w`."""
    period = "1w"
    b = brk.get_pool_stats(period)
    show("GET", f"/api/v1/mining/pools/{period}", summary(b), "-")
    assert isinstance(b["blockCount"], int) and b["blockCount"] > 0
    assert isinstance(b["lastEstimatedHashrate"], int) and b["lastEstimatedHashrate"] > 0
    pools = b["pools"]
    assert pools, "expected non-empty pools list for 1w"
    slugs = [p["slug"] for p in pools]
    assert len(slugs) == len(set(slugs)), "duplicate slugs in pools list"
    ranks = [p["rank"] for p in pools]
    assert ranks == list(range(1, len(pools) + 1)), f"ranks not 1..N: {ranks}"
    block_total = 0
    for p in pools:
        assert isinstance(p["blockCount"], int) and p["blockCount"] >= 0
        assert isinstance(p["emptyBlocks"], int) and p["emptyBlocks"] >= 0
        assert 0.0 <= p["share"] <= 1.0, f"share out of range for {p['slug']}: {p['share']}"
        block_total += p["blockCount"]
    assert block_total <= b["blockCount"], (
        f"sum(pool.blockCount)={block_total} exceeds envelope.blockCount={b['blockCount']}"
    )


@pytest.mark.parametrize("bad", ["9000y", "abc", "1d"])
def test_mining_pools_by_period_malformed(brk, bad):
    """Unknown time period must produce BitviewError(status=400)."""
    with pytest.raises(BitviewError) as exc_info:
        brk.get_text(f"/api/v1/mining/pools/{bad}")
    assert exc_info.value.status == 400, (
        f"expected status=400 for {bad!r}, got {exc_info.value.status}"
    )
