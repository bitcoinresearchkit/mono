"""GET /api/block/{hash}/status"""

import pytest

from bitview_client import BitviewError

from _lib import assert_same_values, show


def test_block_status(brk, mempool, block):
    """Block status must be identical for every height in the fixture."""
    path = f"/api/block/{block.hash}/status"
    b = brk.get_block_status(block.hash)
    m = mempool.get_json(path)
    show("GET", path, b, m)
    assert_same_values(b, m)


def test_block_status_genesis(brk, mempool):
    """Genesis: in_best_chain=true, height=0, next_best is block 1."""
    genesis_hash = mempool.get_text("/api/block-height/0")
    h1_hash = mempool.get_text("/api/block-height/1")
    path = f"/api/block/{genesis_hash}/status"
    b = brk.get_block_status(genesis_hash)
    m = mempool.get_json(path)
    show("GET", path, b, m)
    assert b["in_best_chain"] is True
    assert b["height"] == 0
    assert b["next_best"] == h1_hash
    assert_same_values(b, m)


def test_block_status_tip(brk, mempool):
    """Tip: next_best must be null (only block with no successor)."""
    tip_hash = mempool.get_text("/api/blocks/tip/hash")
    path = f"/api/block/{tip_hash}/status"
    b = brk.get_block_status(tip_hash)
    m = mempool.get_json(path)
    show("GET", path, b, m)
    assert b["in_best_chain"] is True
    assert b["next_best"] is None, f"tip next_best must be null, got {b['next_best']!r}"
    assert_same_values(b, m)


def test_block_status_invalid_hash(brk):
    """Non-hex / wrong-length hash must produce BitviewError(status=400)."""
    with pytest.raises(BitviewError) as exc_info:
        brk.get_block_status("notavalidhash")
    assert exc_info.value.status == 400, (
        f"expected status=400, got {exc_info.value.status}"
    )


def test_block_status_unknown_hash(brk):
    """Syntactically valid but unknown hash must produce BitviewError(status=404)."""
    unknown = "0000000000000000000000000000000000000000000000000000000000000001"
    with pytest.raises(BitviewError) as exc_info:
        brk.get_block_status(unknown)
    assert exc_info.value.status == 404, (
        f"expected status=404, got {exc_info.value.status}"
    )
