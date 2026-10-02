use super::*;

fn headers(values: &[&'static str]) -> HeaderMap {
    let mut headers = HeaderMap::new();
    for value in values {
        headers.append(IF_NONE_MATCH, HeaderValue::from_static(value));
    }
    headers
}

#[test]
fn commas_inside_tags_are_not_list_separators() {
    let etag = Etag::from("s1-abc".to_string());
    assert!(!etag.matches(&headers(&["\"other,s1-abc,other\""])));
    assert!(!etag.matches(&headers(&["s1-abc"])));
    assert!(!etag.matches(&headers(&["W/s1-abc"])));
    assert!(etag.matches(&headers(&["\"other,tag\", W/\"s1-abc\""])));
    let comma_tag = Etag::from("a,b".to_string());
    assert!(comma_tag.matches(&headers(&["W/\"a,b\""])));
}
