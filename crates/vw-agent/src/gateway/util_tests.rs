use super::*;
use axum::http::{HeaderMap, HeaderValue};
use std::net::{IpAddr, Ipv4Addr};

#[test]
fn parse_client_ip_handles_forwarded_values() {
    assert_eq!(parse_client_ip("203.0.113.7:443"), Some(IpAddr::V4(Ipv4Addr::new(203, 0, 113, 7))));
    assert_eq!(parse_client_ip("not an ip"), None);
}

#[test]
fn forwarded_client_ip_prefers_x_forwarded_for_first_hop() {
    let mut headers = HeaderMap::new();
    headers.insert("x-forwarded-for", HeaderValue::from_static("203.0.113.7, 10.0.0.1"));

    assert_eq!(forwarded_client_ip(&headers), Some(IpAddr::V4(Ipv4Addr::new(203, 0, 113, 7))));
}
