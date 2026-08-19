"""GET /api/v1/fullrbf/replacements

Like `/api/v1/replacements`, but limited to trees where at least one
predecessor was non-signaling (full-RBF).
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


def test_fullrbf_replacements_structure(brk, mempool):
    """Full-RBF replacement-tree envelope must match across the full list."""
    path = "/api/v1/fullrbf/replacements"
    b = brk.get_fullrbf_replacements()
    m = mempool.get_json(path)
    show("GET", path, b, m)
    assert isinstance(b, list) and isinstance(m, list)
    assert len(b) <= MAX_REPLACEMENTS and len(m) <= MAX_REPLACEMENTS
    if b and m:
        assert_same_structure(b, m)


def test_fullrbf_replacements_invariants(brk):
    """Length cap, recursive node validation, every root must be full-RBF."""
    b = brk.get_fullrbf_replacements()
    show("GET", "/api/v1/fullrbf/replacements", f"({len(b)} trees)", "-")
    assert 0 <= len(b) <= MAX_REPLACEMENTS, f"unexpected length: {len(b)}"
    for i, root in enumerate(b):
        assert root["fullRbf"] is True, (
            f"root[{i}] is not fullRbf - endpoint contract violated: {root['tx']['txid']}"
        )
        _validate_node(root, f"root[{i}]")
