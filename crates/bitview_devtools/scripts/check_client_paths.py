"""Walks every typed path of client-paths.tsv through the generated Python client and checks that
the leaf it reaches names the expected series and exposes the expected indexes.
Reads the recorded baseline: run `cargo api` (or a passing `cargo api -- --check`) first.
Usage: python3 crates/bitview_devtools/scripts/check_client_paths.py
"""
import sys
from functools import reduce
from pathlib import Path

root = Path(__file__).resolve().parents[3]
sys.path.insert(0, str(root / "packages" / "bitview_client"))
from bitview_client import BitviewClient  # noqa: E402

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
for failure in failures[:20]:
    print(failure, file=sys.stderr)
print(f"python: {len(rows) - len(failures)}/{len(rows)} typed paths resolve", file=sys.stderr)
sys.exit(1 if failures else 0)
