"""GET /api/v1/tx/{txid}/rbf

brk's `tx_graveyard` retains RBF tree data for 1 hour after a tx leaves the
live mempool (whether mined or replaced). Past that window, brk returns
`{replacements: null, replaces: null}`. mempool.space retains RBF history
for longer, so the cross-server response can diverge for txs older than
brk's window but newer than mempool's. The tests verify:
  - brk's contract (always-null) holds for txs deeply past its retention window;
  - value parity holds when mempool also reports null (steady state);
  - within retention, brk and mempool agree structurally on the tree shape.
"""

import pytest
from bitview_client import BitviewError

from _lib import assert_same_structure, show


NULL_RBF = {"replacements": None, "replaces": None}


def test_tx_rbf_brk_null_for_confirmed(brk, block):
    """brk contract: confirmed regular tx always has null replacements/replaces."""
    r = brk.get_tx_rbf(block.txid)
    show("GET", f"/api/v1/tx/{block.txid}/rbf", r, "-")
    assert r == NULL_RBF


def test_tx_rbf_brk_null_for_coinbase(brk, block):
    """brk contract: coinbase tx always has null replacements/replaces."""
    r = brk.get_tx_rbf(block.coinbase_txid)
    show("GET", f"/api/v1/tx/{block.coinbase_txid}/rbf", r, "-")
    assert r == NULL_RBF


def test_tx_rbf_value_parity_when_mempool_null(brk, mempool, block):
    """When mempool also reports null, brk and mempool must agree exactly."""
    path = f"/api/v1/tx/{block.txid}/rbf"
    m = mempool.get_json(path)
    if m != NULL_RBF:
        pytest.skip("mempool retained RBF history (recently-confirmed); brk doesn't")
    b = brk.get_tx_rbf(block.txid)
    show("GET", path, b, m)
    assert b == m


def test_tx_rbf_within_retention_window(brk, mempool):
    """A root from brk's /replacements list is within the 1h retention window;
    brk must return its tree, and mempool (longer retention) must agree on shape."""
    trees = brk.get_replacements()
    if not trees:
        pytest.skip("no recent RBF replacements observed by brk")
    root_txid = trees[0]["tx"]["txid"]
    path = f"/api/v1/tx/{root_txid}/rbf"
    b = brk.get_tx_rbf(root_txid)
    show("GET", path, b, "-")
    assert b["replacements"] is not None, (
        "brk evicted RBF tree it just listed in /replacements"
    )
    m = mempool.get_json(path)
    if m["replacements"] is None:
        pytest.skip("mempool has no RBF history for brk's recent root")
    assert_same_structure(b, m)


def test_tx_rbf_unknown_tx_returns_null(brk, mempool):
    """Both servers return null replacements/replaces for any 64-char hex (no 404)."""
    bad = "0" * 64
    path = f"/api/v1/tx/{bad}/rbf"
    b = brk.get_tx_rbf(bad)
    m = mempool.get_json(path)
    show("GET", path, b, m)
    assert b == NULL_RBF
    assert m == NULL_RBF


@pytest.mark.parametrize("bad", ["abc", "deadbeef"])
def test_tx_rbf_malformed_short(brk, bad):
    """Short txid must produce BitviewError(status=400) on brk (mempool returns 501)."""
    with pytest.raises(BitviewError) as exc_info:
        brk.get_text(f"/api/v1/tx/{bad}/rbf")
    assert exc_info.value.status == 400
