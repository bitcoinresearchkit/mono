"""GET /api/v1/mining/pool/{slug}"""

import pytest

from bitview_client import BitviewError

from _lib import assert_same_structure, assert_same_values, show, summary


# Tip-race / mempool-only / int-vs-str fields excluded from value equality.
VOLATILE = {
    "blockCount", "blockShare", "estimatedHashrate", "reportedHashrate",
    "totalReward", "avgBlockHealth", "avgMatchRate", "avgFeeDelta",
}

# Digit/punctuation slugs that previously diverged between brk and mempool.
# Pinning them here lets the slug rename fixes regress loudly if reverted.
SLUG_RENAME_REGRESSION_GUARD = ["1thash", "175btc", "21inc", "1hash", "58coin", "7pool"]


def test_mining_pool_detail_structure(brk, mempool, pool_slugs):
    """Pool detail envelope must match mempool for the top active pools."""
    for slug in pool_slugs:
        path = f"/api/v1/mining/pool/{slug}"
        b = brk.get_pool(slug)
        m = mempool.get_json(path)
        show("GET", path, summary(b), summary(m))
        assert_same_structure(b, m)


def test_mining_pool_detail_static_fields(brk, mempool, pool_slug):
    """The pool registry fields (id, name, link, slug, unique_id) must value-match."""
    path = f"/api/v1/mining/pool/{pool_slug}"
    b = brk.get_pool(pool_slug)
    m = mempool.get_json(path)
    show("GET", path, b["pool"], m["pool"])
    assert_same_values(b["pool"], m["pool"], path=f"{path}.pool")


def test_mining_pool_detail_invariants(brk, pool_slug):
    """blockCount monotonic by window; blockShare in [0,1]; pool.slug round-trips."""
    b = brk.get_pool(pool_slug)
    show("GET", f"/api/v1/mining/pool/{pool_slug}", summary(b), "-")
    assert b["pool"]["slug"] == pool_slug, (
        f"response.pool.slug={b['pool']['slug']!r} vs URL slug={pool_slug!r}"
    )
    bc = b["blockCount"]
    assert bc["all"] >= bc["1w"] >= bc["24h"] >= 0, f"blockCount not monotonic: {bc}"
    bs = b["blockShare"]
    for window, value in bs.items():
        assert 0.0 <= value <= 1.0, f"blockShare[{window}]={value} out of [0,1]"
    assert isinstance(b["estimatedHashrate"], int) and b["estimatedHashrate"] >= 0


@pytest.mark.parametrize("slug", SLUG_RENAME_REGRESSION_GUARD)
def test_mining_pool_detail_slug_renames(brk, mempool, slug):
    """Pools whose slugs were renamed to match mempool must remain reachable."""
    path = f"/api/v1/mining/pool/{slug}"
    b = brk.get_pool(slug)
    m = mempool.get_json(path)
    show("GET", path, b["pool"], m["pool"])
    assert b["pool"]["slug"] == slug
    assert_same_values(b["pool"], m["pool"], path=f"{path}.pool")


@pytest.mark.parametrize("bad", ["notapool", "FoundryUSA", ""])
def test_mining_pool_detail_malformed(brk, bad):
    """Unknown slug must produce BitviewError(status=400 or 404)."""
    with pytest.raises(BitviewError) as exc_info:
        brk.get_text(f"/api/v1/mining/pool/{bad}")
    assert exc_info.value.status in (400, 404), (
        f"expected 400 or 404 for {bad!r}, got {exc_info.value.status}"
    )
