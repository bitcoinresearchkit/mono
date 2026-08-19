"""GET /api/v1/mining/pool/{slug}/blocks"""

import pytest

from bitview_client import BitviewError

from _lib import assert_same_structure, show


PAGE_SIZE = 100


def test_mining_pool_blocks_structure(brk, mempool, pool_slugs):
    """Per-pool block list element schema must match for top active pools."""
    for slug in pool_slugs:
        path = f"/api/v1/mining/pool/{slug}/blocks"
        b = brk.get_pool_blocks(slug)
        m = mempool.get_json(path)
        show("GET", path, f"({len(b)} blocks)", f"({len(m)} blocks)")
        assert isinstance(b, list) and isinstance(m, list)
        assert_same_structure(b, m)


def test_mining_pool_blocks_invariants(brk, pool_slug):
    """Page is descending, capped at 100, all blocks attributed to the requested pool."""
    b = brk.get_pool_blocks(pool_slug)
    show("GET", f"/api/v1/mining/pool/{pool_slug}/blocks", f"({len(b)} blocks)", "-")
    assert 0 < len(b) <= PAGE_SIZE, f"unexpected length: {len(b)}"
    heights = [blk["height"] for blk in b]
    assert heights == sorted(heights, reverse=True), f"not tip-first: {heights[:5]}..."
    assert len(set(heights)) == len(heights), "duplicate heights in page"
    for blk in b:
        assert blk["stale"] is False, f"stale block in page: {blk['id']}"
        assert blk["extras"]["pool"]["slug"] == pool_slug, (
            f"block {blk['id']} attributed to {blk['extras']['pool']['slug']}, "
            f"expected {pool_slug}"
        )


@pytest.mark.parametrize("bad", ["notapool", "FoundryUSA"])
def test_mining_pool_blocks_malformed(brk, bad):
    """Unknown slug must produce BitviewError(status=400 or 404)."""
    with pytest.raises(BitviewError) as exc_info:
        brk.get_text(f"/api/v1/mining/pool/{bad}/blocks")
    assert exc_info.value.status in (400, 404), (
        f"expected 400 or 404 for {bad!r}, got {exc_info.value.status}"
    )
