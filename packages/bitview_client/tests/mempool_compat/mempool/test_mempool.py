"""GET /api/mempool"""

from _lib import assert_same_structure, show


def test_mempool_info_structure(brk, mempool):
    """Mempool stats envelope must match mempool's keys and types."""
    path = "/api/mempool"
    b = brk.get_mempool()
    m = mempool.get_json(path)
    show("GET", path, b, m, max_lines=15)
    assert_same_structure(b, m)


def test_mempool_info_invariants(brk):
    """Counts positive, fee histogram descending and accounting-exact (sum bin_vsizes == vsize)."""
    b = brk.get_mempool()
    show("GET", "/api/mempool", b, "-", max_lines=15)
    assert isinstance(b["count"], int) and b["count"] > 0
    assert isinstance(b["vsize"], int) and b["vsize"] > 0
    assert b["total_fee"] >= 0, f"negative total_fee: {b['total_fee']}"
    fh = b["fee_histogram"]
    assert isinstance(fh, list) and len(fh) > 0, "fee_histogram must be non-empty list"
    rates = []
    bin_vsize_sum = 0
    for i, entry in enumerate(fh):
        assert isinstance(entry, list) and len(entry) == 2, (
            f"histogram entry {i} not a 2-element list: {entry}"
        )
        rate, bvs = entry
        # Zero-rate bins are legitimate (CPFP/package-relay anchors with
        # zero-fee parents); mempool.space's API returns them too.
        assert isinstance(rate, (int, float)) and rate >= 0, (
            f"negative rate at bin {i}: {rate}"
        )
        assert isinstance(bvs, int) and bvs > 0, f"non-positive vsize at bin {i}: {bvs}"
        rates.append(rate)
        bin_vsize_sum += bvs
    assert rates == sorted(rates, reverse=True), (
        f"fee_histogram not descending by rate: {rates[:5]}..."
    )
    assert bin_vsize_sum == b["vsize"], (
        f"sum(bin_vsizes)={bin_vsize_sum} != vsize={b['vsize']}"
    )
