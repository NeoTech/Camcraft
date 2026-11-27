//! Idle-window configuration (#3940), over synthetic local sockets.
//! Gate map (M4.4): covers app::mcp::listen and app::mcp::port_flag.

use camcraft_app::mcp::{IDLE_TIMEOUT, Inbox, idle_timeout_from};
use std::io::{BufRead, BufReader, Write};
use std::net::TcpStream;
use std::time::{Duration, Instant};

fn argv(args: &[&str]) -> Vec<String> {
    args.iter().map(|s| (*s).to_owned()).collect()
}

#[test]
fn idle_timeout_configuration_defaults_overrides_and_refuses_bad_values() {
    assert_eq!(IDLE_TIMEOUT, Duration::from_secs(300));
    assert_eq!(idle_timeout_from(&[], None), Ok(IDLE_TIMEOUT));
    assert_eq!(idle_timeout_from(&[], Some("  ")), Ok(IDLE_TIMEOUT));
    assert_eq!(
        idle_timeout_from(&[], Some(" 600 ")),
        Ok(Duration::from_secs(600))
    );
    assert_eq!(
        idle_timeout_from(&argv(&["--mcp-idle-timeout", "900"]), Some("600")),
        Ok(Duration::from_secs(900))
    );
    assert_eq!(
        idle_timeout_from(&argv(&["--mcp-idle-timeout=0"]), Some("bad")),
        Ok(Duration::ZERO)
    );
    assert_eq!(idle_timeout_from(&[], Some("0")), Ok(Duration::ZERO));
    assert_eq!(
        idle_timeout_from(&[], Some("18446744073709551615")),
        Ok(Duration::from_secs(u64::MAX))
    );
    for value in ["", "-1", "NaN", "inf", "1.5", "bad", "18446744073709551616"] {
        let err = idle_timeout_from(
            &argv(&[&format!("--mcp-idle-timeout={value}")]),
            Some("600"),
        )
        .expect_err(value);
        assert!(
            err.contains("--mcp-idle-timeout") && err.contains("whole seconds"),
            "{err}"
        );
        if !value.is_empty() {
            let err = idle_timeout_from(&[], Some(value)).expect_err(value);
            assert!(err.contains("CAMCRAFT_MCP_IDLE_TIMEOUT"), "{err}");
        }
    }
    assert!(idle_timeout_from(&argv(&["--mcp-idle-timeout"]), Some("600")).is_err());
    assert!(idle_timeout_from(&argv(&["--mcp-idle-timeout", "-1"]), None).is_err());
}

#[test]
fn a_configured_finite_window_expires_on_the_listener() {
    let idle = idle_timeout_from(&argv(&["--mcp-idle-timeout=1"]), None).expect("seconds");
    let inbox = Inbox::listen_with_idle(0, Box::new(|| {}), idle).expect("listen");
    let client = TcpStream::connect(inbox.addr().expect("addr")).expect("connect");
    client
        .set_read_timeout(Some(Duration::from_secs(3)))
        .expect("timeout");
    let start = Instant::now();
    let mut reader = BufReader::new(client);
    let mut line = String::new();
    reader.read_line(&mut line).expect("idle notice");
    assert!(
        line.contains("notifications/message") && line.contains("idle"),
        "{line}"
    );
    assert!(start.elapsed() >= Duration::from_millis(900));
    line.clear();
    assert_eq!(reader.read_line(&mut line).expect("closed"), 0);
}

#[test]
fn zero_idle_window_keeps_a_silent_client_and_stops_promptly() {
    let idle = idle_timeout_from(&argv(&["--mcp-idle-timeout=0"]), None).expect("disabled");
    let inbox = Inbox::listen_with_idle(0, Box::new(|| {}), idle).expect("listen");
    let mut client = TcpStream::connect(inbox.addr().expect("addr")).expect("connect");
    client
        .set_read_timeout(Some(Duration::from_secs(2)))
        .expect("timeout");
    std::thread::sleep(Duration::from_millis(250));
    writeln!(client, r#"{{"jsonrpc":"2.0","id":3940,"method":"ping"}}"#).expect("send");
    let end = Instant::now() + Duration::from_secs(2);
    loop {
        if let Some(req) = inbox.next() {
            req.answer(Some(r#"{"jsonrpc":"2.0","id":3940,"result":{}}"#.into()));
            break;
        }
        assert!(
            Instant::now() < end,
            "zero must disable idle expiry, not clamp it to the poll step"
        );
        std::thread::sleep(Duration::from_millis(5));
    }
    let mut line = String::new();
    BufReader::new(client).read_line(&mut line).expect("reply");
    assert!(line.contains("\"id\":3940"), "{line}");
    let start = Instant::now();
    drop(inbox);
    assert!(
        start.elapsed() < Duration::from_secs(1),
        "stop stays bounded"
    );
}
