//! #2074: the MCP post tools - `list_posts`, `select_post` through the
//! undo history, the read-only `preview_post`, and `save_post_script`
//! into the user posts folder with a plain file name only.
//! Provenance: our own tests over the MCP surface of `SelectPost`, the
//! post engine and the user posts folder.

use super::Client;
use camcraft_app::posting::BUILTINS;
use serde_json::{Value, json};

/// A script that writes one line at open and one per linear move, and
/// logs one message.
const SMALL: &str = "description = \"Small test post\";\nprogramUnits = \"mm\";\nextension = \"tap\";\nfunction onOpen() { writeln(\"%\"); message(\"hello\"); }\nfunction onLinear(x, y, z) { writeln(\"G1\"); }\n";

fn names(v: &Value) -> Vec<String> {
    v.as_array().map(|a| a.iter().filter_map(|r| r["name"].as_str().map(str::to_owned)).collect()).unwrap_or_default()
}

#[test]
fn list_posts_lists_every_builtin_post_with_what_it_declares() {
    let mut c = Client::new();
    let dir = tempfile::tempdir().expect("temp dir");
    c.server.set_user_posts_dir(dir.path().to_path_buf());
    let got = c.ok("list_posts", json!({}));
    let want: Vec<String> = BUILTINS.iter().map(|(n, _)| (*n).to_owned()).collect();
    assert_eq!(names(&got["builtin"]), want);
    for row in got["builtin"].as_array().expect("rows") {
        assert!(row.get("problem").is_none(), "{row}");
        assert!(row["extension"].as_str().is_some_and(|e| !e.is_empty()), "{row}");
        assert!(row["serves"].as_array().is_some_and(|s| !s.is_empty()), "each shipped post serves a kind: {row}");
        assert!(row["options"].is_array(), "{row}");
    }
    assert_eq!(got["user"], json!([]), "an empty folder lists no user post");
    assert_eq!(got["user_folder"], json!(dir.path().display().to_string()));
    assert!(got["selected"].as_str().is_some(), "{got}");
}

#[test]
fn select_post_is_one_undo_step_and_refuses_a_stranger() {
    let mut c = Client::new();
    let before = c.ok("list_posts", json!({}))["selected"].clone();
    let name = BUILTINS.get(1).map(|(n, _)| *n).expect("two shipped posts");
    let set = c.ok("select_post", json!({ "post": name }));
    assert_eq!(set["previous"], before);
    let after = set["post"].clone();
    assert_ne!(after, before, "{set}");
    assert_eq!(c.ok("list_posts", json!({}))["selected"], after);

    c.ok("undo", json!({}));
    assert_eq!(c.ok("list_posts", json!({}))["selected"], before, "undo restores the post");
    c.ok("redo", json!({}));
    assert_eq!(c.ok("list_posts", json!({}))["selected"], after, "redo selects it again");

    let why = c.refused("select_post", json!({ "post": "No Such Post" }));
    assert!(why.contains("not in the registry"), "{why}");
    let why = c.refused("select_post", json!({ "post": name }));
    assert!(why.contains("already selected"), "{why}");
    let why = c.refused("select_post", json!({}));
    assert!(why.contains("required"), "{why}");
    assert_eq!(c.ok("list_posts", json!({}))["selected"], after, "a refusal changes nothing");
}

#[test]
fn preview_post_runs_a_script_on_a_sample_and_writes_no_file() {
    let mut c = Client::new();
    let dir = tempfile::tempdir().expect("temp dir");
    c.server.set_user_posts_dir(dir.path().to_path_buf());
    let got = c.ok("preview_post", json!({ "source": SMALL, "sample": "Contour" }));
    assert_eq!(got["ok"], json!(true), "{got}");
    let program = got["program"].as_str().expect("program");
    assert!(program.starts_with('%'), "{program}");
    assert!(program.lines().filter(|l| *l == "G1").count() > 4, "a contour has feed moves: {program}");
    assert!(got["warnings"].as_array().is_some_and(|w| w.iter().any(|m| m == "hello")), "{got}");
    assert_eq!(got["swallowed"], json!(0));
    assert_eq!(std::fs::read_dir(dir.path()).map(Iterator::count).unwrap_or(0), 0, "a preview writes no file");

    // Without a sample the stream is the document's: an empty one has none.
    let why = c.refused("preview_post", json!({ "source": SMALL }));
    assert!(why.contains("no operations"), "{why}");

    let why = c.refused("preview_post", json!({ "source": SMALL, "sample": "Nope" }));
    assert!(why.contains("Contour"), "the refusal names the samples: {why}");
    let why = c.refused("preview_post", json!({ "sample": "Contour" }));
    assert!(why.contains("exactly one"), "{why}");
}

#[test]
fn preview_post_returns_the_1_based_line_of_a_syntax_error_and_stops_a_loop() {
    let mut c = Client::new();
    let bad = "description = \"x\";\nfunction onOpen() {\n  writeln(\"%\";\n}\n";
    let got = c.ok("preview_post", json!({ "source": bad, "sample": "Facing" }));
    assert_eq!(got["ok"], json!(false), "{got}");
    let err = &got["errors"][0];
    assert_eq!(err["line"], json!(3), "the error is on line 3: {err}");

    let looping = "function onOpen() { while (true) {} }\n";
    let got = c.ok("preview_post", json!({ "source": looping, "sample": "Facing" }));
    assert_eq!(got["ok"], json!(false), "{got}");
    assert!(got["errors"][0]["message"].as_str().is_some_and(|m| !m.is_empty()), "{got}");
}

#[test]
fn save_post_script_writes_a_plain_name_into_the_user_posts_folder_only() {
    let mut c = Client::new();
    let root = tempfile::tempdir().expect("temp dir");
    let dir = root.path().join("posts");
    c.server.set_user_posts_dir(dir.clone());

    let saved = c.ok("save_post_script", json!({ "file": "mine", "source": SMALL }));
    let path = dir.join("mine.cps");
    assert_eq!(saved["path"], json!(path.display().to_string()));
    assert_eq!(saved["replaced"], json!(false));
    assert_eq!(std::fs::read_to_string(&path).expect("saved"), SMALL);

    let listed = c.ok("list_posts", json!({}));
    let user = &listed["user"][0];
    assert_eq!(user["file"], json!("mine.cps"), "{listed}");
    assert_eq!(user["extension"], json!("tap"));
    assert_eq!(user["description"], json!("Small test post"));
    assert_eq!(user["serves"], Value::Null, "no serves stated");

    let set = c.ok("select_post", json!({ "post": path.display().to_string() }));
    assert!(set["post"].as_str().is_some_and(|p| p.contains("mine.cps")), "{set}");

    let why = c.refused("save_post_script", json!({ "file": "mine.cps", "source": SMALL }));
    assert!(why.contains("already exists"), "{why}");
    let again = c.ok("save_post_script", json!({ "file": "mine.cps", "source": SMALL, "overwrite": true }));
    assert_eq!(again["replaced"], json!(true));

    for bad in ["../escape.cps", "sub/x.cps", "..\\x.cps", ".hidden.cps", ""] {
        let why = c.refused("save_post_script", json!({ "file": bad, "source": SMALL }));
        assert!(why.contains("plain file name") || why.contains("empty"), "{bad:?}: {why}");
    }
    let why = c.refused("save_post_script", json!({ "file": "broken", "source": "function onOpen( {" }));
    assert!(why.contains("not saved"), "{why}");
    assert!(!dir.join("broken.cps").exists(), "a script that does not run is not written");
    assert!(!root.path().join("escape.cps").exists(), "nothing lands outside the folder");
    let files: Vec<_> = std::fs::read_dir(&dir).expect("folder").filter_map(Result::ok).map(|e| e.file_name()).collect();
    assert_eq!(files, vec![std::ffi::OsString::from("mine.cps")], "no temp file is left");
}

/// A live app session asks the user before an agent writes a post file
/// or changes the post that the program posts with.
#[test]
fn a_live_session_asks_before_a_post_file_write_or_a_post_choice() {
    use camcraft_app::mcp::{requires_approval, GateKind};
    assert_eq!(requires_approval("save_post_script", &json!({ "file": "a", "source": "" })), Some(GateKind::FileWrite));
    assert_eq!(requires_approval("select_post", &json!({ "post": "Haas" })), Some(GateKind::LimitEdit));
    assert_eq!(requires_approval("preview_post", &json!({ "path": "/x.cps" })), Some(GateKind::FilePathAccess));
    assert_eq!(requires_approval("preview_post", &json!({ "source": "" })), None);
    assert_eq!(requires_approval("list_posts", &json!({})), None);
}
