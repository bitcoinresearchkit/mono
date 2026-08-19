"""GET /api/block/{hash}"""

import pytest

from bitview_client import BitviewError

from _lib import assert_same_values, show


def test_block_by_hash(brk, mempool, block):
    """Confirmed block info must be byte-identical for every height in the fixture."""
    path = f"/api/block/{block.hash}"
    b = brk.get_block(block.hash)
    m = mempool.get_json(path)
    show("GET", path, b, m)
    assert_same_values(b, m)


def test_block_genesis(brk, mempool):
    """Genesis (h=0): all fields match except previousblockhash.

    Known divergence: brk returns the all-zero hash, mempool.space returns null.
    Excluded from value comparison so this test surfaces if any *other* genesis
    field drifts, without blocking on the known nullability gap.
    """
    genesis_hash = mempool.get_text("/api/block-height/0")
    path = f"/api/block/{genesis_hash}"
    b = brk.get_block(genesis_hash)
    m = mempool.get_json(path)
    show("GET", path, b, m)
    assert b["height"] == 0
    assert b["id"] == genesis_hash
    assert_same_values(b, m, exclude={"previousblockhash"})


def test_block_invalid_hash(brk):
    """Non-hex / wrong-length hash must produce BitviewError(status=400)."""
    with pytest.raises(BitviewError) as exc_info:
        brk.get_block("notavalidhash")
    assert exc_info.value.status == 400, (
        f"expected status=400, got {exc_info.value.status}"
    )


def test_block_unknown_hash(brk):
    """Syntactically valid but unknown hash must produce BitviewError(status=404)."""
    unknown = "0000000000000000000000000000000000000000000000000000000000000001"
    with pytest.raises(BitviewError) as exc_info:
        brk.get_block(unknown)
    assert exc_info.value.status == 404, (
        f"expected status=404, got {exc_info.value.status}"
    )
