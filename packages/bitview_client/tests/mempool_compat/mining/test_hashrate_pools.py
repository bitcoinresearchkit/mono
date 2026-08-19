"""GET /api/v1/mining/hashrate/pools/{time_period}"""

import pytest

from bitview_client import BitviewError

from _lib import assert_same_structure, show, summary


PERIODS = ["24h", "3d", "1w", "1m", "3m", "6m", "1y", "2y", "3y", "all"]


@pytest.mark.parametrize("period", PERIODS)
def test_mining_hashrate_pools_structure(brk, mempool, period):
    """Per-pool hashrate snapshot envelope must match across all periods."""
    path = f"/api/v1/mining/hashrate/pools/{period}"
    b = brk.get_pools_hashrate_by_period(period)
    m = mempool.get_json(path)
    show("GET", path, summary(b), summary(m))
    assert isinstance(b, list) and isinstance(m, list)
    assert_same_structure(b, m)


def test_mining_hashrate_pools_invariants(brk):
    """Snapshot has single timestamp, valid shares summing to <=1, unique pool names (period=1w)."""
    period = "1w"
    b = brk.get_pools_hashrate_by_period(period)
    show("GET", f"/api/v1/mining/hashrate/pools/{period}", summary(b), "-")
    assert len(b) > 0, "expected non-empty per-pool hashrate snapshot for 1w"
    timestamps = {entry["timestamp"] for entry in b}
    assert len(timestamps) == 1, f"expected single snapshot timestamp, got {timestamps}"
    pool_names = [entry["poolName"] for entry in b]
    assert len(set(pool_names)) == len(pool_names), "duplicate poolName in snapshot"
    for entry in b:
        assert entry["poolName"], "empty poolName"
        assert isinstance(entry["avgHashrate"], int) and entry["avgHashrate"] >= 0
        assert isinstance(entry["share"], (int, float)) and 0.0 <= entry["share"] <= 1.0
    total_share = sum(entry["share"] for entry in b)
    assert total_share <= 1.0001, f"share sum > 1: {total_share}"


@pytest.mark.parametrize("bad", ["9000y", "abc", "1d"])
def test_mining_hashrate_pools_malformed(brk, bad):
    """Unknown time period must produce BitviewError(status=400)."""
    with pytest.raises(BitviewError) as exc_info:
        brk.get_text(f"/api/v1/mining/hashrate/pools/{bad}")
    assert exc_info.value.status == 400, (
        f"expected status=400 for {bad!r}, got {exc_info.value.status}"
    )
