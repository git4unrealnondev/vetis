use crate::{
    host::HostConfig,
    listener::ListenerConfig,
    server::{http::resolve_hostname, ServerConfig},
};
use caramelo::{expect, matchers::eq};
use http::{header, HeaderMap, HeaderValue, Uri};

#[test]
fn test_add_host_to_server_config() {
    let server_config = ServerConfig::builder()
        .add_listener(ListenerConfig::default())
        .add_host(HostConfig::default())
        .build()
        .unwrap();
    expect(
        server_config
            .hosts()
            .len(),
    )
    .to_be(eq(1));
}

#[test]
#[should_panic = "Server(\"No listeners configured\")"]
fn test_no_listeners_configured() {
    ServerConfig::builder()
        .add_host(HostConfig::default())
        .build()
        .unwrap();
}

#[test]
#[should_panic = "Server(\"No hosts configured\")"]
fn test_no_hosts_configured() {
    ServerConfig::builder()
        .add_listener(ListenerConfig::default())
        .build()
        .unwrap();
}

#[test]
fn test_has_hosts_configured() {
    let server_config = ServerConfig::builder()
        .add_listener(ListenerConfig::default())
        .add_host(HostConfig::default())
        .build()
        .unwrap();
    expect(
        server_config
            .hosts()
            .len(),
    )
    .to_be(eq(1));
}

#[test]
fn test_has_listsners_configured() {
    let server_config = ServerConfig::builder()
        .add_listener(ListenerConfig::default())
        .add_host(HostConfig::default())
        .build()
        .unwrap();
    expect(
        server_config
            .listeners()
            .len(),
    )
    .to_be(eq(1));
}

fn headers_with_host(value: &str) -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert(header::HOST, HeaderValue::from_str(value).expect("valid header value"));
    headers
}

#[test]
fn test_resolve_hostname_from_uri_authority() {
    // Absolute-form request target: the authority is parsed into the URI.
    let uri: Uri = "http://example.com/path"
        .parse()
        .unwrap();
    expect(resolve_hostname(&uri, &HeaderMap::new())).to_be(eq(Some("example.com".to_string())));
}

#[test]
fn test_resolve_hostname_from_host_header_origin_form() {
    // An HTTP/1.1 origin-form request target yields a URI with no authority, so the
    // host is only available from the `Host` header.
    let uri: Uri = "/health"
        .parse()
        .unwrap();
    let hostname = resolve_hostname(&uri, &headers_with_host("example.com"));
    expect(hostname).to_be(eq(Some("example.com".to_string())));
}

#[test]
fn test_resolve_hostname_strips_port_from_host_header() {
    // The `Host` header carries a port for any non-default port, but virtual hosts
    // are keyed by bare host name.
    let uri: Uri = "/health"
        .parse()
        .unwrap();
    let hostname = resolve_hostname(&uri, &headers_with_host("example.com:8080"));
    expect(hostname).to_be(eq(Some("example.com".to_string())));
}

#[test]
fn test_resolve_hostname_strips_port_from_uri_authority() {
    let uri: Uri = "http://example.com:8080/health"
        .parse()
        .unwrap();
    let hostname = resolve_hostname(&uri, &HeaderMap::new());
    expect(hostname).to_be(eq(Some("example.com".to_string())));
}

#[test]
fn test_resolve_hostname_ipv6_literal() {
    // Brackets are preserved: they are part of the registered form of the address.
    let uri: Uri = "/health"
        .parse()
        .unwrap();
    let hostname = resolve_hostname(&uri, &headers_with_host("[::1]:8080"));
    expect(hostname).to_be(eq(Some("[::1]".to_string())));
}

#[test]
fn test_resolve_hostname_prefers_uri_authority() {
    let uri: Uri = "http://authority.example/path"
        .parse()
        .unwrap();
    let hostname = resolve_hostname(&uri, &headers_with_host("header.example"));
    expect(hostname).to_be(eq(Some("authority.example".to_string())));
}

#[test]
fn test_resolve_hostname_missing_returns_none() {
    // HTTP/1.0 without a `Host` header.
    let uri: Uri = "/health"
        .parse()
        .unwrap();
    expect(resolve_hostname(&uri, &HeaderMap::new())).to_be(eq(None));
}
