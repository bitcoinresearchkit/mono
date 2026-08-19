"""GET /api/v1/mining/hashrate/{time_period}"""

import pytest

from bitview_client import BitviewError

from _lib import assert_same_structure, show, summary


PERIODS = ["24h", "3d", "1w", "1m", "3m", "6m", "1y", "2y", "3y", "all"]


@pytest.mark.parametrize("period", PERIODS)
def test_mining_hashrate_structure(brk, mempool, period):
    """Network hashrate envelope must match across all periods."""
    path = f"/api/v1/mining/hashrate/{period}"
    b = brk.get_hashrate_by_period(period)
    m = mempool.get_json(path)
    show("GET", path, summary(b), summary(m))
    assert_same_structure(b, m)


def test_mining_hashrate_invariants(brk):
    """Series ascending, values positive, current* fields populated (period=1m)."""
    period = "1m"
    b = brk.get_hashrate_by_period(period)
    show("GET", f"/api/v1/mining/hashrate/{period}", summary(b), "-")
    assert isinstance(b["currentHashrate"], int) and b["currentHashrate"] > 0
    assert isinstance(b["currentDifficulty"], (int, float)) and b["currentDifficulty"] > 0
    hashrates = b["hashrates"]
    assert len(hashrates) > 0, "expected non-empty hashrates list for 1m"
    timestamps = [h["timestamp"] for h in hashrates]
    assert timestamps == sorted(timestamps), "hashrate timestamps not ascending"
    assert len(set(timestamps)) == len(timestamps), "duplicate hashrate timestamps"
    for h in hashrates:
        assert isinstance(h["avgHashrate"], int) and h["avgHashrate"] > 0
    difficulty = b["difficulty"]
    times = [d["time"] for d in difficulty]
    heights = [d["height"] for d in difficulty]
    assert times == sorted(times), "difficulty entries not ascending by time"
    assert heights == sorted(heights), "difficulty entries not ascending by height"
    for d in difficulty:
        assert d["difficulty"] > 0, f"non-positive difficulty: {d}"


@pytest.mark.parametrize("bad", ["9000y", "abc", "1d"])
def test_mining_hashrate_malformed(brk, bad):
    """Unknown time period must produce BitviewError(status=400)."""
    with pytest.raises(BitviewError) as exc_info:
        brk.get_text(f"/api/v1/mining/hashrate/{bad}")
    assert exc_info.value.status == 400, (
        f"expected status=400 for {bad!r}, got {exc_info.value.status}"
    )
