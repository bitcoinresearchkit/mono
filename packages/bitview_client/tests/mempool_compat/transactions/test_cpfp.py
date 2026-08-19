"""GET /api/v1/cpfp/{txid}"""

import pytest
from bitview_client import BitviewError

from _lib import assert_same_structure, show


def test_cpfp_structure(brk, mempool, block):
    """CPFP structure must match for a confirmed regular tx (multi-era)."""
    path = f"/api/v1/cpfp/{block.txid}"
    b = brk.get_cpfp(block.txid)
    m = mempool.get_json(path)
    show("GET", path, b, m)
    assert_same_structure(b, m)


def test_cpfp_coinbase_structure(brk, mempool, block):
    """CPFP structure must match for a coinbase tx (multi-era)."""
    path = f"/api/v1/cpfp/{block.coinbase_txid}"
    b = brk.get_cpfp(block.coinbase_txid)
    m = mempool.get_json(path)
    show("GET", path, b, m)
    assert_same_structure(b, m)


def test_cpfp_invariants(brk, live):
    """Recent confirmed tx: ancestors empty, any brk-computed extras non-negative."""
    sample = live.blocks[-1]
    c = brk.get_cpfp(sample.txid)
    show("GET", f"/api/v1/cpfp/{sample.txid}", c, "-")
    assert c["ancestors"] == [], "confirmed tx must have empty ancestors"
    if "fee" in c:
        assert int(c["fee"]) >= 0
    if "effectiveFeePerVsize" in c:
        assert c["effectiveFeePerVsize"] >= 0
    if "adjustedVsize" in c:
        assert int(c["adjustedVsize"]) > 0


def test_cpfp_unknown_txid(brk, mempool):
    """mempool.space returns 200 with {ancestors: []}; brk distinguishes
    'unknown txid' from 'tx with no neighbors' and returns an error."""
    bad = "0" * 64
    path = f"/api/v1/cpfp/{bad}"
    m = mempool.get_json(path)
    assert m.get("ancestors") == []
    with pytest.raises(BitviewError):
        brk.get_cpfp(bad)


@pytest.mark.parametrize("bad", ["abc", "deadbeef"])
def test_cpfp_malformed_short(brk, bad):
    """Short txid must produce BitviewError(status=400) on brk (mempool returns 501)."""
    with pytest.raises(BitviewError) as exc_info:
        brk.get_text(f"/api/v1/cpfp/{bad}")
    assert exc_info.value.status == 400


def test_cpfp_mempool_unconfirmed(brk, mempool):
    """Unconfirmed mempool tx: brk and mempool.space agree on cpfp shape."""
    txids = mempool.get_json("/api/mempool/txids")
    if not txids:
        pytest.skip("mempool.space mempool currently empty")

    for txid in txids[:50]:
        try:
            b = brk.get_cpfp(txid)
        except BitviewError:
            continue
        try:
            m = mempool.get_json(f"/api/v1/cpfp/{txid}")
        except Exception:
            continue
        show("GET", f"/api/v1/cpfp/{txid}", b, m)
        assert_same_structure(b, m)
        assert isinstance(b.get("ancestors"), list)
        assert isinstance(b.get("descendants", []), list)
        return
    pytest.skip("no shared unconfirmed tx between brk and mempool.space")
