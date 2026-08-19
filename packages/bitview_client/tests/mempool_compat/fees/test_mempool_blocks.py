"""GET /api/v1/fees/mempool-blocks"""

from _lib import assert_same_structure, show


MAX_PROJECTED_BLOCKS = 8
FEE_RANGE_LEN = 7


def test_fees_mempool_blocks_structure(brk, mempool):
    """Projected mempool blocks envelope must match across the full list."""
    path = "/api/v1/fees/mempool-blocks"
    b = brk.get_mempool_blocks()
    m = mempool.get_json(path)
    show("GET", path, f"({len(b)} blocks)", f"({len(m)} blocks)")
    assert isinstance(b, list) and isinstance(m, list)
    assert len(b) > 0, "expected non-empty projected blocks"
    assert_same_structure(b, m)


def test_fees_mempool_blocks_invariants(brk):
    """Block counts, sizes, fees, medianFee in feeRange, ordering by descending medianFee."""
    b = brk.get_mempool_blocks()
    show("GET", "/api/v1/fees/mempool-blocks", f"({len(b)} blocks)", "-")
    assert 1 <= len(b) <= MAX_PROJECTED_BLOCKS, (
        f"projected block count out of range: {len(b)}"
    )
    medians = [block["medianFee"] for block in b]
    assert medians == sorted(medians, reverse=True), (
        f"blocks not ordered by descending medianFee: {medians}"
    )
    for i, block in enumerate(b):
        assert block["blockSize"] > 0, f"block {i} has non-positive blockSize"
        assert block["blockVSize"] > 0, f"block {i} has non-positive blockVSize"
        assert block["nTx"] > 0, f"block {i} has non-positive nTx"
        assert block["totalFees"] >= 0, f"block {i} has negative totalFees"
        assert block["medianFee"] > 0, f"block {i} has non-positive medianFee"
        fr = block["feeRange"]
        assert len(fr) == FEE_RANGE_LEN, (
            f"block {i} feeRange has {len(fr)} items, expected {FEE_RANGE_LEN}"
        )
        assert fr == sorted(fr), f"block {i} feeRange not ascending: {fr}"
        assert fr[0] <= block["medianFee"] <= fr[-1], (
            f"block {i} medianFee {block['medianFee']} outside feeRange [{fr[0]}, {fr[-1]}]"
        )
