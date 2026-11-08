use crate::canonical_url::is_canonical;

/// A URL has one spelling; what a query or path legitimately escapes stays accepted.
#[test]
fn canonical_urls() {
    for (path, query) in [
        ("/", None),
        ("/api/block/", None),
        ("/a", Some("b=1&c=2")),
        ("/a", Some("q=a%20b")),
        ("/a", Some("q=%2B1")),
        ("/a%2Fb", None),
    ] {
        assert!(is_canonical(path, query), "{path} {query:?}");
    }
    for (path, query) in [
        ("//a", None),
        ("/a//", None),
        ("/a//b", None),
        ("/a", Some("")),
        ("/a", Some("&b=1")),
        ("/a", Some("b=1&")),
        ("/a", Some("b=1&&c=2")),
        ("/%61", None),
        ("/a", Some("b=%36")),
        ("/a", Some("%62=1")),
        ("/a", Some("b=%2D")),
        ("/a", Some("b=%7e")),
    ] {
        assert!(!is_canonical(path, query), "{path} {query:?}");
    }
}
