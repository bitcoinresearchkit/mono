"""GET /api/tx/{txid}/merkleblock-proof"""

import pytest
from bitview_client import BitviewError

from _lib import show


HEX = set("0123456789abcdef")
HEADER_HEX_LEN = 160  # 80-byte BIP37 block header prefix


def test_tx_merkleblock_proof_value_parity(brk, mempool, block):
    """Merkleblock proof hex must be byte-identical for a regular tx (multi-era)."""
    path = f"/api/tx/{block.txid}/merkleblock-proof"
    b = brk.get_tx_merkleblock_proof(block.txid)
    m = mempool.get_text(path)
    show("GET", path, b[:80] + "...", m[:80] + "...")
    assert b == m


def test_tx_merkleblock_proof_coinbase_value_parity(brk, mempool, block):
    """Merkleblock proof hex must be byte-identical for a coinbase tx (multi-era)."""
    path = f"/api/tx/{block.coinbase_txid}/merkleblock-proof"
    b = brk.get_tx_merkleblock_proof(block.coinbase_txid)
    m = mempool.get_text(path)
    show("GET", path, b[:80] + "...", m[:80] + "...")
    assert b == m


def test_tx_merkleblock_proof_invariants(brk, live):
    """Recent tx: even hex, lowercase, header prefix matches /block/{hash}/header."""
    sample = live.blocks[-1]
    proof = brk.get_tx_merkleblock_proof(sample.txid)
    show("GET", f"/api/tx/{sample.txid}/merkleblock-proof", f"({len(proof)} chars)", "-")
    assert isinstance(proof, str) and len(proof) > HEADER_HEX_LEN
    assert len(proof) % 2 == 0, f"odd hex length: {len(proof)}"
    assert set(proof) <= HEX, f"non-hex chars: {set(proof) - HEX}"
    header = brk.get_block_header(sample.hash)
    assert proof[:HEADER_HEX_LEN] == header, (
        "merkleblock-proof header prefix must match /block/{hash}/header"
    )


@pytest.mark.parametrize("bad", ["abc", "deadbeef"])
def test_tx_merkleblock_proof_malformed_short(brk, bad):
    """Short txid must produce BitviewError(status=400)."""
    with pytest.raises(BitviewError) as exc_info:
        brk.get_text(f"/api/tx/{bad}/merkleblock-proof")
    assert exc_info.value.status == 400


def test_tx_merkleblock_proof_malformed_unknown(brk):
    """Valid 64-char hex with no matching tx must produce BitviewError(status=404)."""
    bad = "0" * 64
    with pytest.raises(BitviewError) as exc_info:
        brk.get_text(f"/api/tx/{bad}/merkleblock-proof")
    assert exc_info.value.status == 404
