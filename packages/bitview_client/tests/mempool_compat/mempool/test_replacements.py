"""GET /api/v1/replacements

Returns up to 25 most-recent RBF replacement trees. Both servers may
report an empty list at any moment; the element structure is what's
load-bearing.
"""

from _lib import assert_same_structure, show


HEX = set("0123456789abcdef")
MAX_REPLACEMENTS = 25


def _validate_node(node, path):
    """Recursively validate a ReplacementNode and its replaces children."""
    assert "tx" in node and "replaces" in node, f"{path}: missing tx/replaces"
    assert node["time"] > 0, f"{path}: non-positive time {node['time']}"
    tx = node["tx"]
    txid = tx["txid"]
    assert isinstance(txid, str) and len(txid) == 64 and set(txid) <= HEX, (
        f"{path}.tx.txid malformed: {txid!r}"
    )
    assert int(tx["fee"]) >= 0, f"{path}.tx.fee negative: {tx['fee']}"
    assert int(tx["vsize"]) > 0, f"{path}.tx.vsize non-positive: {tx['vsize']}"
    assert int(tx["value"]) >= 0, f"{path}.tx.value negative: {tx['value']}"
    assert tx["rate"] >= 0, f"{path}.tx.rate negative: {tx['rate']}"
    assert tx["time"] > 0, f"{path}.tx.time non-positive: {tx['time']}"
    replaces = node["replaces"]
    assert isinstance(replaces, list), f"{path}.replaces not a list"
    for i, child in enumerate(replaces):
        _validate_node(child, f"{path}.replaces[{i}]")


def test_replacements_structure(brk, mempool):
    """Replacement-tree envelope must match across the full list."""
    path = "/api/v1/replacements"
    b = brk.get_replacements()
    m = mempool.get_json(path)
    show("GET", path, b, m)
    assert isinstance(b, list) and isinstance(m, list)
    assert len(b) <= MAX_REPLACEMENTS and len(m) <= MAX_REPLACEMENTS
    if b and m:
        assert_same_structure(b, m)


def test_replacements_invariants(brk):
    """Length cap, recursive node validation."""
    b = brk.get_replacements()
    show("GET", "/api/v1/replacements", f"({len(b)} trees)", "-")
    assert 0 <= len(b) <= MAX_REPLACEMENTS, f"unexpected length: {len(b)}"
    for i, root in enumerate(b):
        _validate_node(root, f"root[{i}]")
