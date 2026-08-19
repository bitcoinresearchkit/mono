"""GET /api/v1/mining/pool/{slug}/hashrate"""

import pytest

from bitview_client import BitviewError

from _lib import assert_same_structure, show, summary


def test_mining_pool_hashrate_structure(brk, mempool, pool_slugs):
    """Pool hashrate history element schema must match for top active pools."""
    for slug in pool_slugs:
        path = f"/api/v1/mining/pool/{slug}/hashrate"
        b = brk.get_pool_hashrate(slug)
        m = mempool.get_json(path)
        show("GET", path, summary(b), summary(m))
        assert isinstance(b, list) and isinstance(m, list)
        assert_same_structure(b, m)


def test_mining_pool_hashrate_invariants(brk, pool_slug):
    """Series must be non-empty, ascending in time, with valid hashrate/share/poolName."""
    b = brk.get_pool_hashrate(pool_slug)
    show("GET", f"/api/v1/mining/pool/{pool_slug}/hashrate", summary(b), "-")
    assert len(b) > 0, f"empty hashrate history for {pool_slug}"
    timestamps = [entry["timestamp"] for entry in b]
    assert timestamps == sorted(timestamps), "timestamps not ascending"
    assert len(set(timestamps)) == len(timestamps), "duplicate timestamps"
    pool_names = {entry["poolName"] for entry in b}
    assert len(pool_names) == 1, f"poolName not consistent across series: {pool_names}"
    for entry in b:
        assert isinstance(entry["avgHashrate"], int) and entry["avgHashrate"] >= 0
        assert isinstance(entry["share"], (int, float)) and 0.0 <= entry["share"] <= 1.0


@pytest.mark.parametrize("bad", ["notapool", "FoundryUSA"])
def test_mining_pool_hashrate_malformed(brk, bad):
    """Unknown slug must produce BitviewError(status=400 or 404)."""
    with pytest.raises(BitviewError) as exc_info:
        brk.get_text(f"/api/v1/mining/pool/{bad}/hashrate")
    assert exc_info.value.status in (400, 404), (
        f"expected 400 or 404 for {bad!r}, got {exc_info.value.status}"
    )
