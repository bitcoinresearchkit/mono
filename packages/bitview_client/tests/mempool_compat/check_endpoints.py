"""
Global registry checks for the mempool.space compatibility suite.

These tests don't poke individual endpoints — they verify the *set* of
endpoints brk exposes matches the registry in `_endpoints.py`. If
mempool.space adds a new endpoint, classify it as covered or skipped here so
this file fails loudly on the next CI run.

Checks:
  1. Every `covered` endpoint actually appears in brk's live `/openapi.json`.
  2. Every `covered` endpoint has a test file at its declared `test_file` path.
  3. Every `skipped` endpoint is NOT exposed by brk (proves the skip is real).
  4. Every brk path that *looks* like a mempool path is classified — no
     orphan routes that we silently added without registering.
  5. Brk extensions listed in `BRK_EXTENSIONS` actually exist on brk.
"""

from pathlib import Path

import pytest

from _endpoints import (
    BRK_EXTENSIONS,
    MEMPOOL_ENDPOINTS,
    Endpoint,
    covered_endpoints,
    skipped_endpoints,
)


HERE = Path(__file__).parent


# ---- Brk-side discovery -------------------------------------------------


_HTTP_METHODS = {"get", "post", "put", "delete", "patch", "head", "options"}


@pytest.fixture(scope="module")
def brk_routes(brk) -> set[tuple[str, str]]:
    """Every `(METHOD, /api/...)` pair brk reports in its OpenAPI spec."""
    spec = brk.get_json("/openapi.json")
    return {
        (method.upper(), path)
        for path, ops in spec["paths"].items()
        if path.startswith("/api")
        for method in ops.keys()
        if method.lower() in _HTTP_METHODS
    }


@pytest.fixture(scope="module")
def brk_paths(brk_routes) -> set[str]:
    """Just the path strings (collapsed across methods)."""
    return {path for _, path in brk_routes}


@pytest.fixture(scope="module")
def brk_compat_paths(brk_paths) -> set[str]:
    """Brk paths that are part of the mempool.space compat surface.

    Strips out brk-only namespaces (series, metrics, urpd, vecs, server, etc.)
    so we're left with paths that belong in the registry.
    """
    brk_only_prefixes = (
        "/api/series",
        "/api/metric",
        "/api/metrics",
        "/api/urpd",
        "/api/vecs",
        "/api/server",
        "/api.json",
    )
    return {p for p in brk_paths if not p.startswith(brk_only_prefixes)}


# ---- Checks -------------------------------------------------------------


@pytest.mark.parametrize("endpoint", covered_endpoints(), ids=lambda e: e.path)
def test_covered_endpoint_exposed_by_brk(brk_routes, endpoint: Endpoint):
    """Every covered endpoint must appear in brk's OpenAPI under the same method."""
    assert (endpoint.method, endpoint.path) in brk_routes, (
        f"{endpoint.method} {endpoint.path} is marked covered in _endpoints.py "
        f"but brk's /openapi.json doesn't expose it"
    )


@pytest.mark.parametrize("endpoint", covered_endpoints(), ids=lambda e: e.path)
def test_covered_endpoint_has_test_file(endpoint: Endpoint):
    """Every covered endpoint must have a test file at its declared path."""
    assert endpoint.test_file is not None, (
        f"{endpoint.path} is covered but has no test_file"
    )
    file = HERE / endpoint.test_file
    assert file.is_file(), (
        f"{endpoint.path} declares test_file={endpoint.test_file!r}, "
        f"but {file} doesn't exist"
    )


@pytest.mark.parametrize("endpoint", skipped_endpoints(), ids=lambda e: e.path)
def test_skipped_endpoint_not_exposed_by_brk(brk_routes, endpoint: Endpoint):
    """Every skipped endpoint must be absent from brk's OpenAPI for that method."""
    assert (endpoint.method, endpoint.path) not in brk_routes, (
        f"{endpoint.method} {endpoint.path} is marked skipped "
        f"({endpoint.skip_reason!r}) but brk now exposes it — please update "
        f"_endpoints.py to mark it covered and add a test"
    )


def test_no_orphan_brk_routes(brk_compat_paths):
    """Every brk compat path must be classified in the registry.

    If this fails, brk has a route that looks like a mempool.space endpoint
    but isn't tracked. Either add it to MEMPOOL_ENDPOINTS (covered + a test)
    or to BRK_EXTENSIONS (brk-only with a one-line justification in source).
    """
    registry_paths = {e.path for e in MEMPOOL_ENDPOINTS}
    extension_paths = set(BRK_EXTENSIONS)
    known = registry_paths | extension_paths
    orphans = brk_compat_paths - known
    assert not orphans, (
        f"Brk exposes {len(orphans)} unclassified mempool-style routes:\n  "
        + "\n  ".join(sorted(orphans))
        + "\nClassify each in mempool_compat/_endpoints.py."
    )


@pytest.mark.parametrize("path", BRK_EXTENSIONS, ids=lambda p: p)
def test_brk_extension_actually_exists(brk_paths, path: str):
    """Each path in BRK_EXTENSIONS must exist in brk's OpenAPI.

    Stale entries get caught here so the list stays accurate.
    """
    assert path in brk_paths, (
        f"{path} is listed in BRK_EXTENSIONS but brk's /openapi.json doesn't "
        f"expose it — remove it from _endpoints.py"
    )


def test_registry_has_no_duplicates():
    """Each (method, path) pair appears at most once in MEMPOOL_ENDPOINTS."""
    seen: set[tuple[str, str]] = set()
    dups: list[tuple[str, str]] = []
    for e in MEMPOOL_ENDPOINTS:
        key = (e.method, e.path)
        if key in seen:
            dups.append(key)
        seen.add(key)
    assert not dups, f"Duplicate registry entries: {dups}"


def test_skipped_endpoints_have_reason():
    """Every skipped endpoint must include a skip_reason."""
    bad = [e for e in skipped_endpoints() if not e.skip_reason]
    assert not bad, f"Skipped endpoints missing skip_reason: {[e.path for e in bad]}"
