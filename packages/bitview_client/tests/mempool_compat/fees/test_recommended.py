"""GET /api/v1/fees/recommended"""

from _lib import assert_same_structure, show


EXPECTED_FEE_KEYS = ["fastestFee", "halfHourFee", "hourFee", "economyFee", "minimumFee"]


def test_fees_recommended_structure(brk, mempool):
    """Recommended fees envelope must match mempool's keys and numeric types."""
    path = "/api/v1/fees/recommended"
    b = brk.get_recommended_fees()
    m = mempool.get_json(path)
    show("GET", path, b, m)
    assert_same_structure(b, m)


def test_fees_recommended_invariants(brk):
    """All tiers numeric, positive, and monotonically non-increasing."""
    b = brk.get_recommended_fees()
    show("GET", "/api/v1/fees/recommended", b, "-")
    for key in EXPECTED_FEE_KEYS:
        assert key in b, f"missing '{key}'"
        assert isinstance(b[key], (int, float)), f"'{key}' not numeric: {type(b[key])}"
        assert b[key] > 0, f"'{key}' must be positive, got {b[key]}"
    assert b["fastestFee"] >= b["halfHourFee"] >= b["hourFee"], (
        f"fast tiers not ordered: {b}"
    )
    assert b["hourFee"] >= b["economyFee"] >= b["minimumFee"], (
        f"slow tiers not ordered: {b}"
    )


def test_fees_recommended_mempool_ordering_sanity(mempool):
    """Sanity: mempool itself follows the documented ordering (pins our reading of the contract)."""
    d = mempool.get_json("/api/v1/fees/recommended")
    assert d["fastestFee"] >= d["halfHourFee"] >= d["hourFee"] >= d["economyFee"] >= d["minimumFee"], (
        f"mempool tiers not ordered: {d}"
    )
