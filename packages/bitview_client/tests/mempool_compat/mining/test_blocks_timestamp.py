"""GET /api/v1/mining/blocks/timestamp/{timestamp}"""

import pytest

from bitview_client import BitviewError

from _lib import assert_same_structure, assert_same_values, show


GENESIS_TIMESTAMP = 1231006505


def test_mining_blocks_timestamp_structure_and_parity(brk, mempool, live):
    """For each live era, brk and mempool must resolve the same block."""
    for block in live.blocks:
        info = brk.get_json(f"/api/block/{block.hash}")
        ts = info["timestamp"]
        path = f"/api/v1/mining/blocks/timestamp/{ts}"
        b = brk.get_block_by_timestamp(ts)
        m = mempool.get_json(path)
        show("GET", path, b, m)
        assert_same_structure(b, m)
        assert_same_values(b, m)


def test_mining_blocks_timestamp_round_trip(brk, live):
    """Looking up a block's own timestamp must return that block (or an earlier one with same ts)."""
    for block in live.blocks:
        info = brk.get_json(f"/api/block/{block.hash}")
        ts = info["timestamp"]
        b = brk.get_block_by_timestamp(ts)
        show("GET", f"/api/v1/mining/blocks/timestamp/{ts}", b, "-")
        assert b["height"] <= block.height, (
            f"resolved height {b['height']} > requested block height {block.height}"
        )


def test_mining_blocks_timestamp_genesis(brk):
    """Genesis Unix timestamp must resolve to genesis (height 0)."""
    b = brk.get_block_by_timestamp(GENESIS_TIMESTAMP)
    show("GET", f"/api/v1/mining/blocks/timestamp/{GENESIS_TIMESTAMP}", b, "-")
    assert b["height"] == 0, f"genesis ts must resolve to height 0, got {b['height']}"


@pytest.mark.parametrize("bad", ["abc", "-1"])
def test_mining_blocks_timestamp_malformed(brk, bad):
    """Non-numeric or negative timestamp must produce BitviewError(status=400)."""
    with pytest.raises(BitviewError) as exc_info:
        brk.get_text(f"/api/v1/mining/blocks/timestamp/{bad}")
    assert exc_info.value.status == 400, (
        f"expected status=400 for {bad!r}, got {exc_info.value.status}"
    )
