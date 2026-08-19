"""GET /api/blocks/{height}"""

import pytest

from bitview_client import BitviewError

from _lib import assert_same_values, show


PAGE_SIZE = 10


def test_blocks_from_height(brk, mempool, block):
    """Up to 10 blocks descending from `block.height` must match mempool tx-for-tx."""
    path = f"/api/blocks/{block.height}"
    b = brk.get_blocks_from_height(block.height)
    m = mempool.get_json(path)
    show("GET", path, f"({len(b)} blocks)", f"({len(m)} blocks)", max_lines=3)
    assert len(b) == min(PAGE_SIZE, block.height + 1)
    assert_same_values(b, m)


def test_blocks_from_height_chain(brk, block):
    """Heights strictly descending; previousblockhash links the page."""
    b = brk.get_blocks_from_height(block.height)
    heights = [blk["height"] for blk in b]
    assert heights == list(range(block.height, block.height - len(b), -1)), (
        f"page is not contiguous descending: {heights}"
    )
    for i in range(len(b) - 1):
        assert b[i]["previousblockhash"] == b[i + 1]["id"], (
            f"chain break at index {i}"
        )


def test_blocks_from_height_genesis(brk, mempool):
    """height=0 returns exactly the genesis block."""
    path = "/api/blocks/0"
    b = brk.get_blocks_from_height(0)
    m = mempool.get_json(path)
    show("GET", path, b, m, max_lines=4)
    assert len(b) == 1
    assert b[0]["id"] == "000000000019d6689c085ae165831e934ff763ae46a2a6c172b3f1b60a8ce26f"
    assert_same_values(b, m)


def test_blocks_from_height_small(brk, mempool):
    """height=5 returns 6 blocks (5,4,3,2,1,0), byte-deterministic against mempool."""
    path = "/api/blocks/5"
    b = brk.get_blocks_from_height(5)
    m = mempool.get_json(path)
    show("GET", path, f"({len(b)} blocks)", f"({len(m)} blocks)", max_lines=3)
    assert len(b) == 6
    assert [blk["height"] for blk in b] == [5, 4, 3, 2, 1, 0]
    assert_same_values(b, m)


def test_blocks_from_height_clamp_to_tip(brk):
    """height past the tip clamps to 10 tip blocks."""
    b = brk.get_blocks_from_height(99_999_999)
    show("GET", "/api/blocks/99999999", f"({len(b)} blocks)", "-")
    assert len(b) == PAGE_SIZE
    assert b[0]["id"] == brk.get_block_tip_hash(), (
        "head of clamped page must equal /api/blocks/tip/hash"
    )


@pytest.mark.parametrize("bad", ["-1", "abc"])
def test_blocks_from_height_malformed(brk, bad):
    """Negative or non-numeric height must produce BitviewError(status=400)."""
    with pytest.raises(BitviewError) as exc_info:
        brk.get_text(f"/api/blocks/{bad}")
    assert exc_info.value.status == 400, (
        f"expected status=400 for {bad!r}, got {exc_info.value.status}"
    )
