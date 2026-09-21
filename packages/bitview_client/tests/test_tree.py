from bitview_client import BitviewClient


def test_catalog_exposes_matching_named_and_indexed_endpoints():
    with BitviewClient("http://fixture.invalid") as client:
        def visit(node):
            if callable(getattr(node, "indexes", None)):
                for index in node.indexes():
                    expected = f"/api/series/{node.name}/{index}"
                    assert getattr(node.by, index)().path() == expected
                    assert node.get(index).path() == expected
                return 1
            children = (getattr(node, name) for name in dir(node)
                        if not name.startswith("_") or name[1:2].isdigit())
            return sum(visit(child) for child in children
                       if hasattr(child, "__dict__") and not callable(child)
                       and not isinstance(child, BitviewClient))

        assert visit(client.series) > 0
