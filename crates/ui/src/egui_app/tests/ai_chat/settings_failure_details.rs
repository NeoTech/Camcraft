//! Provenance: PR #3798 review on cba232f82 (reporter finding 7); our own loopback HTTP fixture,
//! no real credentials or remote services.
//! Gate map (M4.4): covers ui::egui_app::ai_chat::settings_native, ui::egui_app::ai_chat::settings_text.

use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::time::Duration;

use camcraft_app::ai::http::{Client, Timeouts};
use camcraft_app::ai::{ApiKey, Endpoint};
use camcraft_app::mcp::provider::Provider;

use crate::egui_app::ai_chat::settings_native::cloud_test;

const KEY: &str = "placeholder-not-a-real-key";

fn read_request(s: &mut TcpStream) {
    s.set_read_timeout(Some(Duration::from_secs(5)))
        .expect("request timeout");
    let mut received = Vec::new();
    let mut chunk = [0; 4096];
    loop {
        let count = s.read(&mut chunk).expect("request bytes");
        assert!(count > 0, "request ended early");
        received.extend_from_slice(&chunk[..count]);
        if let Some(end) = received.windows(4).position(|w| w == b"\r\n\r\n") {
            let headers = String::from_utf8_lossy(&received[..end]);
            let length = headers
                .lines()
                .find_map(|line| {
                    let (name, value) = line.split_once(':')?;
                    name.eq_ignore_ascii_case("content-length")
                        .then(|| value.trim().parse::<usize>().expect("body length"))
                })
                .unwrap_or(0);
            if received.len() >= end + 4 + length {
                return;
            }
        }
    }
}

/// A provider answer with separate message, type and code fields reaches the
/// Test text and the log. The key is removed, also from the type field.
#[test]
fn failed_cloud_test_shows_provider_type_code_and_endpoint() {
    let listener = TcpListener::bind("127.0.0.1:0").expect("loopback");
    let base = format!("http://{}/v1", listener.local_addr().expect("addr"));
    let ep = Endpoint::parse(&base).expect("endpoint");
    let body = serde_json::json!({"error": {
        "message": format!("Incorrect API key provided: {KEY}"),
        "type": format!("invalid_request_error for {KEY}"),
        "code": "invalid_api_key",
    }})
    .to_string();
    let server = std::thread::spawn(move || {
        let (mut s, _) = listener.accept().expect("accept");
        read_request(&mut s);
        write!(
            s,
            "HTTP/1.1 401 Unauthorized\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
            body.len()
        )
        .expect("reply");
    });
    let client = Client {
        offline_only: true,
        timeouts: Timeouts {
            connect: Duration::from_secs(2),
            total: Duration::from_secs(5),
        },
    };
    let key = ApiKey::new(KEY).expect("placeholder");
    let text = cloud_test(&client, &ep, Provider::OpenAiChat, "gpt-test", &key)
        .expect_err("401 fails the Test");
    server.join().expect("server");
    assert!(text.contains("rejected the key (HTTP 401)"), "{text}");
    assert!(text.contains(&format!("Model tried: gpt-test at {base}")), "{text}");
    assert!(text.contains("Incorrect API key provided"), "message: {text}");
    assert!(text.contains("type: invalid_request_error"), "type: {text}");
    assert!(text.contains("code: invalid_api_key"), "code: {text}");
    assert!(
        text.contains(&format!("endpoint: POST {base}/chat/completions")),
        "request endpoint: {text}"
    );
    assert!(!text.contains(KEY), "key leaked: {text}");
    assert!(!text.contains("placeholder"), "partial key leaked: {text}");
    assert!(text.contains("[redacted]"), "{text}");
}
