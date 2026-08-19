# Run:
# uv run pytest tests/tree.py -s

"""Comprehensive test that fetches all endpoints in the tree."""

from bitview_client import BitviewClient


def is_series_pattern(obj):
    """Check if an object is a series pattern (has indexes() method and by attribute)."""
    return (
        hasattr(obj, "indexes")
        and callable(getattr(obj, "indexes", None))
        and hasattr(obj, "by")
    )


def get_all_series(obj, path=""):
    """Recursively collect all SeriesPattern instances from the tree."""
    series = []

    for attr_name in dir(obj):
        # Skip dunder methods and internal attributes (_letter), but allow _digit (e.g., _10y, _2017)
        if attr_name.startswith("__"):
            continue
        if attr_name.startswith("_") and len(attr_name) > 1 and attr_name[1].isalpha():
            continue

        try:
            attr = getattr(obj, attr_name)
        except Exception:
            continue

        if attr is None or callable(attr):
            continue

        current_path = f"{path}.{attr_name}" if path else attr_name

        # Check if this is a series pattern using the indexes() method
        if is_series_pattern(attr):
            series.append((current_path, attr))

        # Recurse into nested tree nodes
        if hasattr(attr, "__dict__"):
            series.extend(get_all_series(attr, current_path))

    return series


def test_all_endpoints():
    """Test fetching last value from all series endpoints."""
    client = BitviewClient("http://localhost:3110")

    series = get_all_series(client.series)
    print(f"\nFound {len(series)} series")

    success = 0

    for path, s in series:
        # Use the indexes() method to get all available indexes
        indexes = s.indexes()

        for idx_name in indexes:
            full_path = f"{path}.by.{idx_name}"

            try:
                # Verify both access methods work: .by.index() and .get(index)
                by = s.by
                endpoint_by_property = getattr(by, idx_name)()
                endpoint_by_get = s.get(idx_name)

                if endpoint_by_property is None:
                    raise Exception(f"series.by.{idx_name}() returned None")
                if endpoint_by_get is None:
                    raise Exception(f"series.get('{idx_name}') returned None")

                endpoint_by_property.tail(1).fetch()
                success += 1
                print(f"OK: {full_path}")
            except Exception as e:
                print(f"FAIL: {full_path} -> {e}")
                return

    print("\n=== Results ===")
    print(f"Success: {success}")


if __name__ == "__main__":
    test_all_endpoints()
