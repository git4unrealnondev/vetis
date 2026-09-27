use crate::server::http::resolve_hostname;
use http::{header, HeaderMap, Uri};

fn headers_with_host(value: &str) -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert(header::HOST, http::HeaderValue::from_str(value).expect("valid header value"));
    headers
}

#[test]
fn test_resolve_hostname_from_uri_authority() {
    // Absolute-form request target: hyper parses the authority into the URI.
    let uri: Uri = "http://example.com/path"
        .parse()
        .unwrap();
    let hostname = resolve_hostname(&uri, &HeaderMap::new()).unwrap();

    assert_eq!(hostname, "example.com");
}

#[test]
fn test_resolve_hostname_from_host_header_origin_form() {
    // Regression: an HTTP/1.1 origin-form request target yields a URI with no
    // authority, so the host is only available from the `Host` header. Before the
    // fallback this returned `None` and the request was rejected with 400.
    let uri: Uri = "/health"
        .parse()
        .unwrap();
    let hostname = resolve_hostname(&uri, &headers_with_host("example.com")).unwrap();

    assert_eq!(hostname, "example.com");
}

#[test]
fn test_resolve_hostname_strips_port() {
    // Regression: `Host` and URI authorities may carry a port, but virtual hosts
    // are keyed by bare host name. Leaving the port attached made every lookup miss.
    let uri: Uri = "/health"
        .parse()
        .unwrap();
    let hostname = resolve_hostname(&uri, &headers_with_host("example.com:8080")).unwrap();

    assert_eq!(hostname, "example.com");

    let uri: Uri = "http://example.com:8080/health"
        .parse()
        .unwrap();
    let hostname = resolve_hostname(&uri, &HeaderMap::new()).unwrap();

    assert_eq!(hostname, "example.com");
}

#[test]
fn test_resolve_hostname_keeps_bare_host_intact() {
    let uri: Uri = "/health"
        .parse()
        .unwrap();
    let hostname = resolve_hostname(&uri, &headers_with_host("example.com")).unwrap();

    assert_eq!(hostname, "example.com");
}

#[test]
fn test_resolve_hostname_ipv6_literal() {
    // A bracketed IPv6 literal carries colons that are not a port separator, so the
    // port is stripped without eating the address. `Authority::host` keeps the
    // brackets, which is the form a host is registered under.
    let uri: Uri = "/health"
        .parse()
        .unwrap();
    let hostname = resolve_hostname(&uri, &headers_with_host("[::1]:8080")).unwrap();

    assert_eq!(hostname, "[::1]");
}

#[test]
fn test_resolve_hostname_prefers_uri_authority() {
    // When both are present the URI authority wins, matching how HTTP/2 clients
    // are routed when a proxy rewrites `Host`.
    let uri: Uri = "http://authority.example/path"
        .parse()
        .unwrap();
    let hostname = resolve_hostname(&uri, &headers_with_host("header.example")).unwrap();

    assert_eq!(hostname, "authority.example");
}

#[test]
fn test_resolve_hostname_missing_returns_none() {
    // Neither source available: HTTP/1.0 without a `Host` header.
    let uri: Uri = "/health"
        .parse()
        .unwrap();
    assert!(resolve_hostname(&uri, &HeaderMap::new()).is_none());
}
