// Reuse the generated API catalog rather than maintaining another route list.
use std::cmp::Reverse;

const CATALOG: &str = include_str!("../../bitview_cli/src/generated.rs");

pub fn catalog() -> Vec<&'static str> {
    let mut routes: Vec<_> = CATALOG
        .lines()
        .filter_map(|line| line.trim().strip_prefix("path: \"")?.split('"').next())
        .collect();
    // Literal segments take precedence over parameters, like the HTTP router.
    routes.sort_by_key(|route| {
        Reverse(
            route
                .split('/')
                .map(|s| !s.starts_with('{'))
                .collect::<Vec<_>>(),
        )
    });
    routes.dedup();
    routes
}

pub fn endpoint<'a>(uri: &'a str, routes: &'a [&str]) -> &'a str {
    let path = uri.split('?').next().unwrap_or(uri).trim_end_matches('/');
    let path = if path.is_empty() { "/" } else { path };
    for route in routes {
        let mut actual = path.split('/');
        if route.split('/').all(|part| {
            actual.next().is_some_and(|value| {
                part == value || (part.starts_with('{') && part.ends_with('}') && !value.is_empty())
            })
        }) && actual.next().is_none()
        {
            return route;
        }
    }
    // Don't guess historical or website routes. Keep their paths visible.
    path
}
