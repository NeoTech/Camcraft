//! The UI link (#3751): `camcraft-cli --ui` finds a running window by
//! its discovery file and opens a session with the file's token. Pinned
//! here: no window is a clear error with exit 3 (never a headless run),
//! a wrong token is refused before a line reaches the app, a good
//! session carries a tool call to the window's inbox, and the file goes
//! when the window's inbox drops.
//! Provenance: our own tests over our own link; no HSMWorks material.

use camcraft_app::mcp::link::{self, Entry};
use camcraft_app::mcp::{link_call, Inbox, LinkError, Select};

use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, Instant};

fn fresh_dir(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("camcraft-ui-link-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("temp dir");
    dir
}

fn cli_ui(dir: &Path, args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_camcraft-cli"))
        .env(link::DIR_ENV, dir)
        .arg("--ui")
        .args(args)
        .output()
        .expect("the CLI binary runs")
}

fn write_entry(dir: &Path, entry: &Entry) {
    let text = serde_json::to_string(entry).expect("entry json");
    std::fs::write(dir.join(format!("{}.json", entry.pid)), text).expect("write entry");
}

/// Answer the inbox's requests on this thread (the window's frame loop
/// stand-in) until `client` ends; returns the lines that reached it.
fn serve_until<T: Send + 'static>(inbox: &Inbox, client: std::thread::JoinHandle<T>) -> (T, Vec<String>) {
    let mut lines = Vec::new();
    let deadline = Instant::now() + Duration::from_secs(20);
    while !client.is_finished() {
        assert!(Instant::now() < deadline, "the client did not finish");
        while let Some(req) = inbox.next() {
            let msg: Value = serde_json::from_str(req.line()).expect("json line");
            lines.push(req.line().to_owned());
            let reply = json!({
                "jsonrpc": "2.0", "id": msg["id"],
                "result": { "content": [], "structuredContent": { "echo": msg["params"]["name"] }, "isError": false }
            });
            req.answer(Some(reply.to_string()));
        }
        std::thread::sleep(Duration::from_millis(5));
    }
    (client.join().expect("client thread"), lines)
}

#[test]
fn a_cli_call_with_no_window_says_so_and_exits_3() {
    let dir = fresh_dir("none");
    let o = cli_ui(&dir, &["call", "get_job"]);
    let err = String::from_utf8_lossy(&o.stderr);
    assert_eq!(o.status.code(), Some(3), "{err}");
    assert!(err.contains("no running CamCraft window"), "{err}");
    assert!(err.contains("nothing ran"), "{err}");
    assert!(o.stdout.is_empty(), "no headless answer on stdout");

    // A file a crashed window left behind: its socket does not answer.
    let stale = Entry {
        format: link::FORMAT,
        pid: 4_000_000_000,
        transport: "tcp".into(),
        address: "127.0.0.1:1".into(),
        token: "00".into(),
        project: None,
        app_version: "0".into(),
    };
    write_entry(&dir, &stale);
    let o = cli_ui(&dir, &["call", "get_job"]);
    let err = String::from_utf8_lossy(&o.stderr);
    assert_eq!(o.status.code(), Some(3), "{err}");
    assert!(err.contains("stale"), "{err}");

    // A window asked for by a pid that no file names.
    let o = cli_ui(&dir, &["--window", "123", "call", "get_job"]);
    let err = String::from_utf8_lossy(&o.stderr);
    assert_eq!(o.status.code(), Some(3), "{err}");
    assert!(err.contains("process id 123"), "{err}");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn bad_ui_arguments_are_usage_errors() {
    let dir = fresh_dir("usage");
    for args in [&["call"][..], &["call", "get_job", "[1]"], &["call", "get_job", "{nope"], &["--window"], &["dance"]] {
        let o = cli_ui(&dir, args);
        assert_eq!(o.status.code(), Some(2), "{args:?}: {}", String::from_utf8_lossy(&o.stderr));
    }
    let o = cli_ui(&dir, &["--help"]);
    assert_eq!(o.status.code(), Some(0));
    assert!(String::from_utf8_lossy(&o.stdout).contains("Exit: 0 stood, 1 refused, 2 usage, 3 no running window, 4 window busy"));
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn a_wrong_token_is_refused_and_the_window_goes_on_serving() {
    let dir = fresh_dir("token");
    let inbox = Inbox::link(&dir, Box::new(|| {})).expect("open the link");
    let entry = inbox.link_entry().expect("a link entry").clone();
    assert_eq!(link::entries(&dir), vec![entry.clone()], "the discovery file names the window");
    assert_eq!(entry.token.len(), 64);

    // The same window, but a file with a wrong token.
    let forged = fresh_dir("token-forged");
    write_entry(&forged, &Entry { token: "f".repeat(64), ..entry.clone() });
    let forged_dir = forged.clone();
    let client = std::thread::spawn(move || link_call(&forged_dir, &Select::Only, "get_job", &json!({})));
    let (result, lines) = serve_until(&inbox, client);
    let err = result.expect_err("a wrong token is refused");
    assert!(matches!(err, LinkError::Failed(_)), "{err:?}");
    assert!(err.to_string().contains("refused the link token"), "{err}");
    assert_eq!(err.exit_code(), 1);
    assert!(lines.is_empty(), "no line of a refused client reached the app: {lines:?}");

    // A hello that is not JSON at all is refused the same way.
    let wrong_hello = json!({ "camcraft_link": 1 }).to_string();
    assert!(!link::hello_matches(wrong_hello.as_bytes(), &entry.token));
    assert!(!link::hello_matches(b"GET / HTTP/1.1", &entry.token));
    assert!(link::hello_matches(link::hello_line(&entry.token).as_bytes(), &entry.token));

    // The window still serves a client with the right token.
    let good_dir = dir.clone();
    let client = std::thread::spawn(move || link_call(&good_dir, &Select::Pid(std::process::id()), "get_job", &json!({})));
    let (result, lines) = serve_until(&inbox, client);
    let result = result.expect("the right token is served");
    assert_eq!(result["structuredContent"]["echo"], json!("get_job"));
    assert_eq!(lines.len(), 1, "one tools/call reached the window: {lines:?}");
    let line: Value = serde_json::from_str(&lines[0]).expect("json");
    assert_eq!(line["method"], json!("tools/call"));

    // The window's file goes with its inbox.
    drop(inbox);
    assert!(link::entries(&dir).is_empty(), "the discovery file is removed on close");
    let _ = std::fs::remove_dir_all(&dir);
    let _ = std::fs::remove_dir_all(&forged);
}

#[test]
fn the_link_file_follows_the_open_project() {
    let dir = fresh_dir("project");
    let mut inbox = Inbox::link(&dir, Box::new(|| {})).expect("open the link");
    let project = dir.join("part.camproj");
    inbox.note_project(Some(&project));
    let entries = link::entries(&dir);
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].project.as_deref(), Some(project.as_path()));
    assert!(Select::Project(project.clone()) != Select::Only);
    assert_eq!(Select::parse("42"), Select::Pid(42));
    assert_eq!(Select::parse("a/part.camproj"), Select::Project(PathBuf::from("a/part.camproj")));
    drop(inbox);
    let _ = std::fs::remove_dir_all(&dir);
}

fn lookup(pairs: &[(&str, &str)]) -> impl Fn(&str) -> Option<std::ffi::OsString> {
    let pairs: Vec<(String, String)> = pairs.iter().map(|(k, v)| ((*k).to_owned(), (*v).to_owned())).collect();
    move |k: &str| pairs.iter().find(|(n, _)| n == k).map(|(_, v)| v.into())
}

#[test]
fn the_link_folder_is_the_users_own() {
    let tail = Path::new("camcraft").join("ui-link");
    // An absolute path is spelled per host, so each host checks its own.
    if cfg!(windows) {
        let roaming = r"C:\Users\u\AppData\Roaming";
        assert_eq!(link::dir_for("windows", lookup(&[("APPDATA", roaming)])), Some(Path::new(roaming).join(&tail)));
        assert_eq!(link::dir_for("windows", lookup(&[(link::DIR_ENV, r"D:\link")])), Some(PathBuf::from(r"D:\link")));
    } else {
        let run = "/run/user/1000";
        let both = [("XDG_RUNTIME_DIR", run), ("HOME", "/home/u")];
        assert_eq!(link::dir_for("linux", lookup(&both)), Some(Path::new(run).join(&tail)), "the runtime folder first");
        assert_eq!(link::dir_for("linux", lookup(&[("HOME", "/home/u")])), Some(Path::new("/home/u/.local/share").join(&tail)));
        assert_eq!(link::dir_for("macos", lookup(&[("HOME", "/Users/u")])), Some(Path::new("/Users/u/Library/Application Support").join(&tail)));
    }
    assert_eq!(link::dir_for("linux", lookup(&[])), None, "no home, no folder");
    assert_eq!(link::dir_for("linux", lookup(&[("HOME", "relative")])), None, "a relative home is refused");
    assert!(link::enabled(None));
    assert!(link::enabled(Some("on")));
    assert!(!link::enabled(Some(" OFF ")));
}

#[test]
fn a_second_ui_call_during_a_slow_call_reports_busy_and_exits_4() {
    let dir = fresh_dir("busy");
    let inbox = Inbox::link(&dir, Box::new(|| {})).expect("open the link");
    let entry = inbox.link_entry().expect("a link entry").clone();

    // Client 1 connects directly to the socket to occupy the session
    let addr = entry.address.clone();
    let transport = entry.transport.clone();
    let (connected_tx, connected_rx) = std::sync::mpsc::channel();
    let (finish_tx, finish_rx) = std::sync::mpsc::channel();

    let client1 = std::thread::spawn(move || {
        let stream = match transport.as_str() {
            "tcp" => Box::new(std::net::TcpStream::connect(&addr).expect("tcp")) as Box<dyn std::io::Read + Send>,
            #[cfg(unix)]
            "unix" => Box::new(std::os::unix::net::UnixStream::connect(&addr).expect("unix")) as Box<dyn std::io::Read + Send>,
            other => panic!("unknown: {other}"),
        };
        let _ = connected_tx.send(());
        let _ = finish_rx.recv();
        drop(stream);
    });

    connected_rx.recv().expect("client 1 connected");
    std::thread::sleep(Duration::from_millis(50));

    // Client 2 attempts to call a tool via cli_ui; times out waiting on busy window
    let o = cli_ui(&dir, &["call", "get_job"]);
    let err = String::from_utf8_lossy(&o.stderr);
    assert_eq!(o.status.code(), Some(4), "{err}");
    assert!(err.contains("is busy with another call"), "{err}");

    let _ = finish_tx.send(());
    let _ = client1.join();
    drop(inbox);
    let _ = std::fs::remove_dir_all(&dir);
}
