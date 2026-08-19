"""GET /api/block/{hash}/txs"""

import pytest

from bitview_client import BitviewError

from _lib import assert_same_values, show


# brk and mempool's sigop counting diverges (different rules for redeemscript/witness).
# Documented divergence — same source data, different aggregation.
SIGOPS_DIFF = {"sigops"}

PAGE_SIZE = 25


def test_block_txs(brk, mempool, block):
    """First page (up to 25 txs) must match mempool.space tx-for-tx, in order."""
    path = f"/api/block/{block.hash}/txs"
    b = brk.get_block_txs(block.hash)
    m = mempool.get_json(path)
    show("GET", path, f"({len(b)} txs)", f"({len(m)} txs)", max_lines=4)
    assert len(b) == len(m), f"Page size: brk={len(b)} vs mempool={len(m)}"
    assert_same_values(b, m, exclude=SIGOPS_DIFF)


def test_block_txs_page_size(brk, block):
    """Page size invariant: 25 if block has ≥25 txs, else exactly tx_count."""
    txids = brk.get_block_txids(block.hash)
    b = brk.get_block_txs(block.hash)
    expected = min(PAGE_SIZE, len(txids))
    assert len(b) == expected, (
        f"page size: got {len(b)}, expected min({PAGE_SIZE}, {len(txids)})={expected}"
    )


def test_block_txs_order_and_coinbase(brk, block):
    """Page order matches /txids and tx[0] is the coinbase."""
    txids = brk.get_block_txids(block.hash)
    b = brk.get_block_txs(block.hash)
    assert [t["txid"] for t in b] == txids[: len(b)], "order must match /txids"
    assert b[0]["vin"][0]["is_coinbase"] is True, "tx[0] must be coinbase"


def test_block_txs_genesis(brk, mempool):
    """Genesis: single coinbase tx with the well-known scriptsig."""
    genesis_hash = mempool.get_text("/api/block-height/0")
    path = f"/api/block/{genesis_hash}/txs"
    b = brk.get_block_txs(genesis_hash)
    m = mempool.get_json(path)
    show("GET", path, f"({len(b)} txs)", f"({len(m)} txs)", max_lines=4)
    assert len(b) == 1
    assert_same_values(b, m, exclude=SIGOPS_DIFF)
    assert b[0]["txid"] == "4a5e1e4baab89f3a32518a88c31bc87f618f76673e2cc77ab2127b7afdeda33b"


def test_block_txs_invalid_hash(brk):
    """Non-hex / wrong-length hash must produce BitviewError(status=400)."""
    with pytest.raises(BitviewError) as exc_info:
        brk.get_block_txs("notavalidhash")
    assert exc_info.value.status == 400, (
        f"expected status=400, got {exc_info.value.status}"
    )


def test_block_txs_unknown_hash(brk):
    """Syntactically valid but unknown hash must produce BitviewError(status=404)."""
    unknown = "0000000000000000000000000000000000000000000000000000000000000001"
    with pytest.raises(BitviewError) as exc_info:
        brk.get_block_txs(unknown)
    assert exc_info.value.status == 404, (
        f"expected status=404, got {exc_info.value.status}"
    )
