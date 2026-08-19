"""GET /api/v1/mining/pool/{slug}/blocks/{height}"""

import pytest

from bitview_client import BitviewError

from _lib import assert_same_structure, show


PAGE_SIZE = 100


def test_mining_pool_blocks_from_height_structure(brk, mempool, pool_slug, block):
    """Per-pool block list before a height must match mempool's element schema."""
    path = f"/api/v1/mining/pool/{pool_slug}/blocks/{block.height}"
    b = brk.get_pool_blocks_from(pool_slug, block.height)
    m = mempool.get_json(path)
    show("GET", path, f"({len(b)} blocks)", f"({len(m)} blocks)")
    assert isinstance(b, list) and isinstance(m, list)
    assert_same_structure(b, m)


def test_mining_pool_blocks_from_height_invariants(brk, pool_slug, block):
    """Page is descending, capped at 100, height-bounded, attributed to the pool."""
    b = brk.get_pool_blocks_from(pool_slug, block.height)
    show("GET", f"/api/v1/mining/pool/{pool_slug}/blocks/{block.height}", f"({len(b)} blocks)", "-")
    assert 0 <= len(b) <= PAGE_SIZE, f"unexpected length: {len(b)}"
    if not b:
        return
    heights = [blk["height"] for blk in b]
    assert heights == sorted(heights, reverse=True), f"not descending: {heights[:5]}..."
    assert max(heights) <= block.height, (
        f"page contains height > requested {block.height}: max={max(heights)}"
    )
    assert len(set(heights)) == len(heights), "duplicate heights in page"
    for blk in b:
        assert blk["stale"] is False, f"stale block in page: {blk['id']}"
        assert blk["extras"]["pool"]["slug"] == pool_slug, (
            f"block {blk['id']} attributed to {blk['extras']['pool']['slug']}, "
            f"expected {pool_slug}"
        )


@pytest.mark.parametrize("bad_slug", ["notapool", "FoundryUSA"])
def test_mining_pool_blocks_from_height_malformed_slug(brk, bad_slug):
    """Unknown slug must produce BitviewError(status=400 or 404)."""
    with pytest.raises(BitviewError) as exc_info:
        brk.get_text(f"/api/v1/mining/pool/{bad_slug}/blocks/100000")
    assert exc_info.value.status in (400, 404), (
        f"expected 400 or 404 for slug {bad_slug!r}, got {exc_info.value.status}"
    )


@pytest.mark.parametrize("bad_height", ["-1", "abc"])
def test_mining_pool_blocks_from_height_malformed_height(brk, pool_slug, bad_height):
    """Negative or non-numeric height must produce BitviewError(status=400)."""
    with pytest.raises(BitviewError) as exc_info:
        brk.get_text(f"/api/v1/mining/pool/{pool_slug}/blocks/{bad_height}")
    assert exc_info.value.status == 400, (
        f"expected 400 for height {bad_height!r}, got {exc_info.value.status}"
    )
