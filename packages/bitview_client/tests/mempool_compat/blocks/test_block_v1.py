"""GET /api/v1/block/{hash}"""

import pytest

from bitview_client import BitviewError

from _lib import assert_same_structure, assert_same_values, show


# Fee-distribution fields where mempool uses positional/cut-based percentiles
# and brk uses a single vsize-weighted percentile distribution. Same source
# data, different aggregation — diverges anywhere tx sizes vary.
FEE_ALGO_DIFF = {"medianFee", "medianFeeAmt", "feeRange", "feePercentiles"}

# avgFeeRate: mempool returns Bitcoin Core's getblockstats.avgfeerate (integer
# sat/vB), brk returns the float version. Same formula, brk preserves precision.
ROUNDING_DIFF = {"avgFeeRate"}

EXTRAS_EXCLUDE = FEE_ALGO_DIFF | ROUNDING_DIFF


def test_block_v1_envelope(brk, mempool, block):
    """Top-level v1 envelope: id matches, brk-only `stale` and `extras.price` are present."""
    path = f"/api/v1/block/{block.hash}"
    b = brk.get_block_v1(block.hash)
    m = mempool.get_json(path)
    show("GET", path, b, m, max_lines=30)
    assert b["id"] == block.hash
    assert b["stale"] is False, f"confirmed block must have stale=False, got {b['stale']!r}"
    assert isinstance(b["extras"]["price"], (int, float))
    assert b["extras"]["price"] >= 0


def test_block_v1_extras(brk, mempool, block):
    """Every shared extras field must match (excluding documented algorithm divergences)."""
    path = f"/api/v1/block/{block.hash}"
    b = brk.get_block_v1(block.hash)["extras"]
    m = mempool.get_json(path)["extras"]
    show("GET", f"{path}  [extras]", b, m, max_lines=50)
    assert_same_structure(b, m)
    assert_same_values(b, m, exclude=EXTRAS_EXCLUDE)


# Genesis-only divergence: Bitcoin Core treats the genesis coinbase output as
# unspendable and excludes it from the UTXO set (Satoshi quirk). brk counts
# it like any other output, so genesis utxoSetChange is 1 on brk vs 0 on
# mempool.space. Documented test-only exclude.
GENESIS_EXTRAS_EXCLUDE = EXTRAS_EXCLUDE | {"utxoSetChange"}


def test_block_v1_genesis(brk, mempool):
    """Genesis: extras must match (excluding fee-algo divergence and the genesis utxoSetChange quirk)."""
    genesis_hash = mempool.get_text("/api/block-height/0")
    path = f"/api/v1/block/{genesis_hash}"
    b = brk.get_block_v1(genesis_hash)
    m = mempool.get_json(path)
    show("GET", path, b, m, max_lines=30)
    assert b["height"] == 0
    assert b["stale"] is False
    assert_same_structure(b["extras"], m["extras"])
    assert_same_values(b["extras"], m["extras"], exclude=GENESIS_EXTRAS_EXCLUDE)


def test_block_v1_invalid_hash(brk):
    """Non-hex / wrong-length hash must produce BitviewError(status=400)."""
    with pytest.raises(BitviewError) as exc_info:
        brk.get_block_v1("notavalidhash")
    assert exc_info.value.status == 400, (
        f"expected status=400, got {exc_info.value.status}"
    )


def test_block_v1_unknown_hash(brk):
    """Syntactically valid but unknown hash must produce BitviewError(status=404)."""
    unknown = "0000000000000000000000000000000000000000000000000000000000000001"
    with pytest.raises(BitviewError) as exc_info:
        brk.get_block_v1(unknown)
    assert exc_info.value.status == 404, (
        f"expected status=404, got {exc_info.value.status}"
    )
