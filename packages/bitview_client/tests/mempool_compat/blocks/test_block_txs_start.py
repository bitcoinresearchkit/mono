"""GET /api/block/{hash}/txs/{start_index}"""

import pytest

from bitview_client import BitviewError

from _lib import assert_same_values, show


SIGOPS_DIFF = {"sigops"}
PAGE_SIZE = 25


def test_block_txs_start_default(brk, block):
    """/txs/0 must equal /txs (the default page)."""
    b0 = brk.get_block_txs_from_index(block.hash, 0)
    bx = brk.get_block_txs(block.hash)
    show("GET", f"/api/block/{block.hash}/txs/0", f"({len(b0)} txs)", f"vs /txs ({len(bx)} txs)")
    assert b0 == bx


def test_block_txs_start_aligned(brk, block):
    """Every aligned page is the matching slice of /txids; no overlap, no gaps."""
    txids = brk.get_block_txids(block.hash)
    n = len(txids)
    for start in range(0, n, PAGE_SIZE):
        page = brk.get_block_txs_from_index(block.hash, start)
        end = min(start + PAGE_SIZE, n)
        assert [t["txid"] for t in page] == txids[start:end], (
            f"page at start={start} txids do not match /txids[{start}:{end}]"
        )


def test_block_txs_start_last_partial_page(brk, block):
    """The final page returns exactly the trailing remainder."""
    txids = brk.get_block_txids(block.hash)
    n = len(txids)
    last_start = ((n - 1) // PAGE_SIZE) * PAGE_SIZE
    expected = n - last_start
    page = brk.get_block_txs_from_index(block.hash, last_start)
    assert len(page) == expected, (
        f"last page from start={last_start}: got {len(page)}, expected {expected}"
    )


def test_block_txs_start_against_mempool(brk, mempool, block):
    """Mid-block page: full body must match mempool tx-for-tx."""
    txids = brk.get_block_txids(block.hash)
    if len(txids) <= PAGE_SIZE:
        pytest.skip(f"block has only {len(txids)} txs (<= page size)")
    path = f"/api/block/{block.hash}/txs/{PAGE_SIZE}"
    b = brk.get_block_txs_from_index(block.hash, PAGE_SIZE)
    m = mempool.get_json(path)
    show("GET", path, f"({len(b)} txs)", f"({len(m)} txs)", max_lines=4)
    assert_same_values(b, m, exclude=SIGOPS_DIFF)


def test_block_txs_start_genesis(brk, mempool):
    """Genesis: /txs/0 returns the 1 coinbase tx; /txs/1 must 404."""
    genesis_hash = mempool.get_text("/api/block-height/0")
    page0 = brk.get_block_txs_from_index(genesis_hash, 0)
    assert len(page0) == 1
    assert page0[0]["txid"] == "4a5e1e4baab89f3a32518a88c31bc87f618f76673e2cc77ab2127b7afdeda33b"
    with pytest.raises(BitviewError) as exc_info:
        brk.get_block_txs_from_index(genesis_hash, 1)
    assert exc_info.value.status == 404, (
        f"expected status=404 for past-end on genesis, got {exc_info.value.status}"
    )


def test_block_txs_start_past_end(brk, block):
    """Start past the last tx must produce BitviewError(status=404)."""
    txids = brk.get_block_txids(block.hash)
    past = len(txids) + 1000
    with pytest.raises(BitviewError) as exc_info:
        brk.get_block_txs_from_index(block.hash, past)
    assert exc_info.value.status == 404, (
        f"expected status=404 for past-end, got {exc_info.value.status}"
    )


def test_block_txs_start_invalid_hash(brk):
    """Non-hex / wrong-length hash must produce BitviewError(status=400)."""
    with pytest.raises(BitviewError) as exc_info:
        brk.get_block_txs_from_index("notavalidhash", 0)
    assert exc_info.value.status == 400, (
        f"expected status=400, got {exc_info.value.status}"
    )


def test_block_txs_start_unknown_hash(brk):
    """Syntactically valid but unknown hash must produce BitviewError(status=404)."""
    unknown = "0000000000000000000000000000000000000000000000000000000000000001"
    with pytest.raises(BitviewError) as exc_info:
        brk.get_block_txs_from_index(unknown, 0)
    assert exc_info.value.status == 404, (
        f"expected status=404, got {exc_info.value.status}"
    )
