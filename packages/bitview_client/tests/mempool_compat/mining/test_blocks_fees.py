"""GET /api/v1/mining/blocks/fees/{time_period}"""

import pytest

from bitview_client import BitviewError

from _lib import assert_same_structure, show, summary


PERIODS = ["24h", "3d", "1w", "1m", "3m", "6m", "1y", "2y", "3y", "all"]


@pytest.mark.parametrize("period", PERIODS)
def test_mining_blocks_fees_structure(brk, mempool, period):
    """Average block fees envelope must match across all periods."""
    path = f"/api/v1/mining/blocks/fees/{period}"
    b = brk.get_block_fees(period)
    m = mempool.get_json(path)
    show("GET", path, summary(b), summary(m))
    assert isinstance(b, list) and isinstance(m, list)
    assert_same_structure(b, m)


def test_mining_blocks_fees_invariants(brk):
    """Series ascending by height and timestamp, fees and USD non-negative (period=1m)."""
    period = "1m"
    b = brk.get_block_fees(period)
    show("GET", f"/api/v1/mining/blocks/fees/{period}", summary(b), "-")
    assert len(b) > 0, "expected non-empty fees series for 1m"
    heights = [entry["avgHeight"] for entry in b]
    timestamps = [entry["timestamp"] for entry in b]
    assert heights == sorted(heights), "avgHeight not ascending"
    assert timestamps == sorted(timestamps), "timestamps not ascending"
    assert len(set(heights)) == len(heights), "duplicate avgHeight in series"
    for entry in b:
        assert entry["avgFees"] >= 0, f"negative avgFees: {entry}"
        assert entry["USD"] >= 0, f"negative USD: {entry}"


@pytest.mark.parametrize("bad", ["9000y", "abc", "1d"])
def test_mining_blocks_fees_malformed(brk, bad):
    """Unknown time period must produce BitviewError(status=400)."""
    with pytest.raises(BitviewError) as exc_info:
        brk.get_text(f"/api/v1/mining/blocks/fees/{bad}")
    assert exc_info.value.status == 400, (
        f"expected status=400 for {bad!r}, got {exc_info.value.status}"
    )


@pytest.mark.parametrize("period", PERIODS)
def test_mining_blocks_fees_values_match(brk, mempool, period):
    """For shared buckets (keyed by timestamp), avgHeight and avgFees must equal mempool.space.
    USD is sourced from different price oracles and is intentionally not compared."""
    path = f"/api/v1/mining/blocks/fees/{period}"
    b = brk.get_block_fees(period)
    m = mempool.get_json(path)
    show("GET", path, summary(b), summary(m))

    m_by_ts = {e["timestamp"]: e for e in m}
    matched = 0
    for be in b:
        me = m_by_ts.get(be["timestamp"])
        if me is None:
            continue
        matched += 1
        assert be["avgHeight"] == me["avgHeight"], (
            f"avgHeight drift at timestamp {be['timestamp']}: "
            f"brk={be['avgHeight']} mempool={me['avgHeight']}"
        )
        assert be["avgFees"] == me["avgFees"], (
            f"avgFees mismatch at timestamp {be['timestamp']}: "
            f"brk={be['avgFees']} mempool={me['avgFees']}"
        )
    assert matched > 0, "no overlapping bucket timestamps between brk and mempool"
