"""GET /api/v1/mining/difficulty-adjustments/{time_period}"""

import pytest

from bitview_client import BitviewError

from _lib import assert_same_structure, show, summary


PERIODS = ["24h", "3d", "1w", "1m", "3m", "6m", "1y", "2y", "3y", "all"]
RETARGET_INTERVAL = 2016


@pytest.mark.parametrize("period", PERIODS)
def test_mining_difficulty_adjustments_structure(brk, mempool, period):
    """Historical difficulty adjustments envelope must match across all periods."""
    path = f"/api/v1/mining/difficulty-adjustments/{period}"
    b = brk.get_difficulty_adjustments_by_period(period)
    m = mempool.get_json(path)
    show("GET", path, summary(b), summary(m))
    assert isinstance(b, list) and isinstance(m, list)
    assert_same_structure(b, m)


def test_mining_difficulty_adjustments_invariants(brk):
    """Tip-first ordering, retarget-aligned heights, genesis sentinel (period=all)."""
    period = "all"
    b = brk.get_difficulty_adjustments_by_period(period)
    show("GET", f"/api/v1/mining/difficulty-adjustments/{period}", summary(b), "-")
    assert len(b) > 0, "expected non-empty difficulty adjustments for period=all"
    heights = [entry[1] for entry in b]
    assert heights == sorted(heights, reverse=True), "entries not descending by height"
    assert len(set(heights)) == len(heights), "duplicate heights in series"
    assert heights[-1] == 0, f"last entry must be genesis (height 0), got {heights[-1]}"
    assert heights.count(0) == 1, "expected exactly one genesis entry"
    for entry in b[:-1]:
        timestamp, height, difficulty, change_ratio = entry
        assert height % RETARGET_INTERVAL == 0, (
            f"non-genesis height {height} not on retarget boundary"
        )
        assert difficulty > 0, f"non-positive difficulty: {difficulty} at height {height}"
        assert change_ratio > 0, f"non-positive change ratio: {change_ratio} at height {height}"
    genesis = b[-1]
    assert genesis[2] == 1.0, f"genesis difficulty must be 1.0, got {genesis[2]}"


@pytest.mark.parametrize("bad", ["9000y", "abc", "1d"])
def test_mining_difficulty_adjustments_malformed(brk, bad):
    """Unknown time period must produce BitviewError(status=400)."""
    with pytest.raises(BitviewError) as exc_info:
        brk.get_text(f"/api/v1/mining/difficulty-adjustments/{bad}")
    assert exc_info.value.status == 400, (
        f"expected status=400 for {bad!r}, got {exc_info.value.status}"
    )


# `all`: mempool.space's `difficulty_adjustments` table begins from when their tracker
# started, not genesis, so series length and earliest entries diverge by construction.
@pytest.mark.parametrize("period", [p for p in PERIODS if p != "all"])
def test_mining_difficulty_adjustments_values_match(brk, mempool, period):
    """For every bounded period, every retarget entry must match mempool.space:
    same height, same timestamp, and difficulty/change-ratio within float tolerance."""
    path = f"/api/v1/mining/difficulty-adjustments/{period}"
    b = brk.get_difficulty_adjustments_by_period(period)
    m = mempool.get_json(path)
    show("GET", path, summary(b), summary(m))
    assert len(b) == len(m), f"length mismatch: brk={len(b)} mempool={len(m)}"

    for be, me in zip(b, m):
        bt, bh, bd, br = be
        mt, mh, md, mr = me
        assert bh == mh, f"height mismatch at retarget: brk={bh} mempool={mh}"
        assert bt == mt, f"timestamp mismatch at height {bh}: brk={bt} mempool={mt}"
        # mempool.space serializes difficulty/change_ratio with limited precision,
        # so only require parity within mempool.space's ~6-decimal rounding window.
        assert abs(bd - md) / max(md, 1.0) < 1e-5, (
            f"difficulty drift at height {bh}: brk={bd} mempool={md}"
        )
        assert abs(br - mr) < 1e-5, (
            f"change_ratio drift at height {bh}: brk={br} mempool={mr}"
        )
