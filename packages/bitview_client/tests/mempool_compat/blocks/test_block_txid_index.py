"""GET /api/block/{hash}/txid/{index}"""

import pytest

from bitview_client import BitviewError

from _lib import show


def test_block_txid_coinbase(brk, mempool, block):
    """Position 0 is the coinbase txid; must match mempool.space byte-for-byte."""
    path = f"/api/block/{block.hash}/txid/0"
    b = brk.get_block_txid(block.hash, 0)
    m = mempool.get_text(path)
    show("GET", path, b, m)
    assert b == m


def test_block_txid_positions(brk, mempool, block):
    """First, middle, and last positions in the block must all match."""
    txids = mempool.get_json(f"/api/block/{block.hash}/txids")
    n = len(txids)
    indices = sorted({0, 1, n // 2, n - 1})
    indices = [i for i in indices if 0 <= i < n]
    for i in indices:
        path = f"/api/block/{block.hash}/txid/{i}"
        b = brk.get_block_txid(block.hash, i)
        m = mempool.get_text(path)
        show("GET", path, b, m)
        assert b == m, f"index {i} differs: brk={b!r} mempool={m!r}"


def test_block_txid_genesis(brk, mempool):
    """Genesis: only one tx (coinbase) at index 0, byte-deterministic."""
    genesis_hash = mempool.get_text("/api/block-height/0")
    path = f"/api/block/{genesis_hash}/txid/0"
    b = brk.get_block_txid(genesis_hash, 0)
    m = mempool.get_text(path)
    show("GET", path, b, m)
    assert b == m
    assert b == "4a5e1e4baab89f3a32518a88c31bc87f618f76673e2cc77ab2127b7afdeda33b"


def test_block_txid_out_of_range(brk, mempool, block):
    """Index past the last tx in the block must produce BitviewError(status=404) on both."""
    txids = mempool.get_json(f"/api/block/{block.hash}/txids")
    bad_index = len(txids) + 1000
    with pytest.raises(BitviewError) as exc_info:
        brk.get_block_txid(block.hash, bad_index)
    assert exc_info.value.status == 404, (
        f"expected status=404 for out-of-range index, got {exc_info.value.status}"
    )


def test_block_txid_invalid_hash(brk):
    """Non-hex / wrong-length hash must produce BitviewError(status=400)."""
    with pytest.raises(BitviewError) as exc_info:
        brk.get_block_txid("notavalidhash", 0)
    assert exc_info.value.status == 400, (
        f"expected status=400, got {exc_info.value.status}"
    )


def test_block_txid_unknown_hash(brk):
    """Syntactically valid but unknown hash must produce BitviewError(status=404)."""
    unknown = "0000000000000000000000000000000000000000000000000000000000000001"
    with pytest.raises(BitviewError) as exc_info:
        brk.get_block_txid(unknown, 0)
    assert exc_info.value.status == 404, (
        f"expected status=404, got {exc_info.value.status}"
    )
