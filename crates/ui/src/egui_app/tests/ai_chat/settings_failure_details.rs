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
    assert!(text.contains("the key was rejected (HTTP 401)"), "{text}");
    assert!(
        text.contains(&format!("Model tried: gpt-test at {base}")),
        "{text}"
    );
    assert!(
        text.contains("Incorrect API key provided"),
        "message: {text}"
    );
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

/// The Test text for one cloud answer: `status` like `401 Unauthorized`, with
/// `body` as the provider reply. Provenance: our own loopback HTTP fixture, no
/// real credentials or remote services.
fn failed_test_for(provider: Provider, status: &'static str, body: String) -> String {
    let listener = TcpListener::bind("127.0.0.1:0").expect("loopback");
    let base = format!("http://{}/v1", listener.local_addr().expect("addr"));
    let ep = Endpoint::parse(&base).expect("endpoint");
    let server = std::thread::spawn(move || {
        let (mut s, _) = listener.accept().expect("accept");
        read_request(&mut s);
        write!(
            s,
            "HTTP/1.1 {status}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
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
    let text = cloud_test(&client, &ep, provider, "m-test", &key).expect_err("failure");
    server.join().expect("server");
    text
}

/// Each cloud provider behind Test reports a rejected key on its documented
/// authentication failure, and the status stays safe: no key and no raw
/// response beyond the redacted classified fields.
#[test]
fn authentication_failure_shows_key_rejected_for_each_cloud_provider() {
    let cases = [
        (
            Provider::Anthropic,
            "401 Unauthorized",
            serde_json::json!({"type":"error","error":{
                "type":"authentication_error",
                "message": format!("invalid x-api-key. key: {KEY}"),
            }})
            .to_string(),
        ),
        (
            Provider::OpenAiChat,
            "401 Unauthorized",
            serde_json::json!({"error":{
                "message": format!("Incorrect API key provided: {KEY}"),
                "type":"invalid_request_error",
                "code":"invalid_api_key",
            }})
            .to_string(),
        ),
        (
            Provider::OpenAiResponses,
            "401 Unauthorized",
            serde_json::json!({"error":{
                "message":"Incorrect API key provided",
                "type":"invalid_request_error",
                "code":"invalid_api_key",
            }})
            .to_string(),
        ),
        (
            Provider::Gemini,
            "403 Forbidden",
            serde_json::json!({"error":{
                "code":403,
                "message":"API key not valid. Please pass a valid API key.",
                "status":"PERMISSION_DENIED",
            }})
            .to_string(),
        ),
    ];
    for (provider, status, body) in cases {
        let text = failed_test_for(provider, status, body);
        assert!(
            text.starts_with("Test failed: the key was rejected"),
            "{provider:?}: {text}"
        );
        assert!(!text.contains(KEY), "{provider:?}: key leaked: {text}");
        assert!(
            !text.contains("placeholder"),
            "{provider:?}: partial key leaked: {text}"
        );
    }
}

/// Gemini answers a rejected key with HTTP 400 and a documented key-error
/// status, not with 401. The documented shape classifies as key rejected; a
/// plain 400 without a marker stays a request or model failure.
#[test]
fn gemini_bad_request_with_documented_key_error_shows_key_rejected() {
    let body = serde_json::json!({"error":{
        "code":400,
        "message":"API key not valid. Please pass a valid API key.",
        "status":"INVALID_ARGUMENT",
    }})
    .to_string();
    let text = failed_test_for(Provider::Gemini, "400 Bad Request", body);
    assert!(
        text.starts_with("Test failed: the key was rejected"),
        "{text}"
    );
    assert!(!text.contains(KEY), "key leaked: {text}");
}

/// A wrong model, missing credit, a rate limit and a provider outage keep
/// their own statuses and never read as a key rejection.
#[test]
fn non_auth_cloud_test_failures_remain_distinct_from_key_rejected() {
    let model = failed_test_for(
        Provider::OpenAiChat,
        "404 Not Found",
        serde_json::json!({"error":{
            "message":"The model 'm-test' does not exist",
            "type":"invalid_request_error",
            "code":"model_not_found",
        }})
        .to_string(),
    );
    assert!(model.contains("does not know this model"), "{model}");
    assert!(!model.contains("the key was rejected"), "{model}");
    let credit = failed_test_for(
        Provider::OpenAiChat,
        "402 Payment Required",
        serde_json::json!({"error":{
            "message":"You have insufficient credits",
            "type":"insufficient_quota",
        }})
        .to_string(),
    );
    assert!(credit.contains("billing or credits"), "{credit}");
    assert!(!credit.contains("the key was rejected"), "{credit}");
    let quota = failed_test_for(
        Provider::OpenAiChat,
        "429 Too Many Requests",
        serde_json::json!({"error":{
            "message":"Rate limit reached",
            "type":"rate_limit_error",
        }})
        .to_string(),
    );
    assert!(quota.contains("the rate limit or the quota"), "{quota}");
    assert!(!quota.contains("the key was rejected"), "{quota}");
    let outage = failed_test_for(
        Provider::Anthropic,
        "503 Service Unavailable",
        serde_json::json!({"type":"error","error":{
            "type":"overloaded_error","message":"Overloaded",
        }})
        .to_string(),
    );
    assert!(
        outage.contains("the provider is not available now"),
        "{outage}"
    );
    assert!(!outage.contains("the key was rejected"), "{outage}");
}
