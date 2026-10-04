"""Walks every typed path of client-paths.tsv through the generated Python client and checks that
the leaf it reaches names the expected series and exposes the expected indexes, and that
enumerating the tree finds exactly those leaves.
Reads the recorded baseline: run `cargo api` (or a passing `cargo api -- --check`) first.
Usage: python3 crates/bitview_devtools/scripts/check_client_paths.py
"""
import sys
from functools import reduce
from pathlib import Path

root = Path(__file__).resolve().parents[3]
sys.path.insert(0, str(root / "packages" / "bitview_client"))
from bitview_client import BitviewClient, _Child  # noqa: E402

rows = [
    line.split("\t")
    for line in (root / "crates/bitview_devtools/snapshots/client-paths.tsv").read_text().splitlines()
    if line and not line.startswith("#")
]
if not rows or any(len(row) != 5 for row in rows):
    sys.exit("client-paths.tsv is empty or malformed: run `cargo api`")

series = BitviewClient("http://fixture.invalid").series
failures = []
for name, _rust, _javascript, python, indexes in rows:
    leaf = reduce(lambda node, key: getattr(node, key, None), python.split("."), series)
    actual_name = getattr(leaf, "name", None)
    actual_indexes = sorted(leaf.indexes()) if hasattr(leaf, "indexes") else None
    expected_indexes = sorted(index for index in indexes.split(",") if index)
    if actual_name != name or actual_indexes != expected_indexes:
        failures.append(
            f"{python}: expected {name} {expected_indexes}, got {actual_name} {actual_indexes}"
        )


def count_leaves(node):
    """Leaves reachable through the tree's lazy attributes."""
    keys = [key for key, value in vars(type(node)).items() if isinstance(value, _Child)]
    return sum(
        1 if hasattr(child, "indexes") else count_leaves(child)
        for child in (getattr(node, key) for key in keys)
    )


leaves = count_leaves(series)
paths = len({python for _name, _rust, _javascript, python, _indexes in rows})
problems = []
if paths != len(rows):
    problems.append(f"client-paths.tsv lists {len(rows) - paths} paths twice")
if leaves != len(rows):
    problems.append(f"the series tree enumerates {leaves} leaves, expected {len(rows)}")
for failure in (problems + failures)[:20]:
    print(failure, file=sys.stderr)
print(f"python: {len(rows) - len(failures)}/{len(rows)} typed paths resolve", file=sys.stderr)
sys.exit(1 if failures or problems else 0)
