"""GET /api/tx/{txid}/merkle-proof"""

import pytest
from bitview_client import BitviewError

from _lib import assert_same_values, show


HEX = set("0123456789abcdef")


def test_tx_merkle_proof_value_parity(brk, mempool, block):
    """Merkle proof must match for a confirmed regular tx (multi-era)."""
    path = f"/api/tx/{block.txid}/merkle-proof"
    b = brk.get_tx_merkle_proof(block.txid)
    m = mempool.get_json(path)
    show("GET", path, b, m)
    assert_same_values(b, m)


def test_tx_merkle_proof_coinbase_value_parity(brk, mempool, block):
    """Merkle proof must match for a coinbase tx (multi-era)."""
    path = f"/api/tx/{block.coinbase_txid}/merkle-proof"
    b = brk.get_tx_merkle_proof(block.coinbase_txid)
    m = mempool.get_json(path)
    show("GET", path, b, m)
    assert_same_values(b, m)


def test_tx_merkle_proof_invariants_regular(brk, live):
    """Recent regular tx: block_height matches, pos > 0, all siblings 64-char hex."""
    sample = live.blocks[-1]
    if sample.txid == sample.coinbase_txid:
        pytest.skip("recent block has only coinbase")
    p = brk.get_tx_merkle_proof(sample.txid)
    show("GET", f"/api/tx/{sample.txid}/merkle-proof", p, "-")
    assert int(p["block_height"]) == sample.height
    assert int(p["pos"]) > 0, "regular tx pos must be > 0 (coinbase is at 0)"
    assert isinstance(p["merkle"], list)
    for i, sib in enumerate(p["merkle"]):
        assert isinstance(sib, str) and len(sib) == 64 and set(sib) <= HEX, (
            f"merkle[{i}] malformed: {sib!r}"
        )


def test_tx_merkle_proof_invariants_coinbase(brk, live):
    """Recent coinbase: pos == 0, block_height matches."""
    sample = live.blocks[-1]
    p = brk.get_tx_merkle_proof(sample.coinbase_txid)
    show("GET", f"/api/tx/{sample.coinbase_txid}/merkle-proof", p, "-")
    assert int(p["block_height"]) == sample.height
    assert int(p["pos"]) == 0
    for sib in p["merkle"]:
        assert isinstance(sib, str) and len(sib) == 64 and set(sib) <= HEX


@pytest.mark.parametrize("bad", ["abc", "deadbeef"])
def test_tx_merkle_proof_malformed_short(brk, bad):
    """Short txid must produce BitviewError(status=400)."""
    with pytest.raises(BitviewError) as exc_info:
        brk.get_text(f"/api/tx/{bad}/merkle-proof")
    assert exc_info.value.status == 400


def test_tx_merkle_proof_malformed_unknown(brk):
    """Valid 64-char hex with no matching tx must produce BitviewError(status=404)."""
    bad = "0" * 64
    with pytest.raises(BitviewError) as exc_info:
        brk.get_text(f"/api/tx/{bad}/merkle-proof")
    assert exc_info.value.status == 404
