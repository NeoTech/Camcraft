//! The session's own machine (`.mch`) and post (`.cps`) files (#2950): the
//! texts the browser build read from the user's disk as bytes, kept by file
//! name for this run of the app only. A page has no paths, so the machine
//! mount and the post choice name such a file by its bare name; the readers
//! here answer that name before a disk read is tried. A text is only kept
//! when it passes the same checks a file from disk passes (machine:
//! validation and parse; post: compile and run on an empty stream), so a
//! bad file never enters a list. A saved project that names a file which
//! is not loaded again reads from disk, fails, and is refused by name.
//!
//! The store belongs to the thread that filled it: the browser runs the
//! whole app on one thread, and each native test thread gets its own
//! files. A reader on another thread does not find a session name, so the
//! disk read fails and the caller refuses that name - never a swap.
//! Provenance: our own store over the app's machine and post laws; no
//! HSMWorks material.

use std::cell::RefCell;
use std::collections::BTreeMap;

use crate::machine::{Machine, parse_mch, validate::validate_text};
use crate::post::engine;

#[derive(Default)]
struct Store {
    machines: BTreeMap<String, String>,
    posts: BTreeMap<String, String>,
}

thread_local! {
    static STORE: RefCell<Store> = RefCell::new(Store::default());
}

/// Run `f` on this thread's store; `None` only on a re-entrant call.
fn with<R>(f: impl FnOnce(&mut Store) -> R) -> Option<R> {
    STORE.with(|s| s.try_borrow_mut().ok().map(|mut s| f(&mut s)))
}

fn keep(name: &str, what: &str, insert: impl FnOnce(&mut Store)) -> Result<(), String> {
    with(insert).ok_or_else(|| format!("{name}: the session's {what} list is busy - load it again"))
}

/// Check a machine file's text the way a mount from disk does (validation,
/// then the parse) and keep it under `name`. A later file of the same name
/// replaces it. The error names the file and the first refusal.
pub fn add_machine(name: &str, text: &str) -> Result<Machine, String> {
    if let Some(first) = validate_text(text).first() {
        return Err(format!("{name} fails validation - {first}"));
    }
    let machine = parse_mch(text).map_err(|e| format!("{name} is not a machine file: {e}"))?;
    keep(name, "machine", |s| {
        s.machines.insert(name.to_owned(), text.to_owned());
    })?;
    Ok(machine)
}

/// Check a post script the way `SelectPost` does (compile and run on an
/// empty stream) and keep it under `name`. The error names the file.
pub fn add_post(name: &str, text: &str) -> Result<(), String> {
    engine::run(text, &serde_json::Value::Null, "").map_err(|e| format!("post script {name} does not run: {e}"))?;
    keep(name, "post", |s| {
        s.posts.insert(name.to_owned(), text.to_owned());
    })
}

/// The kept text of the machine file `name`, if this session loaded it.
#[must_use]
pub fn machine_text(name: &str) -> Option<String> {
    with(|s| s.machines.get(name).cloned()).flatten()
}

/// The kept text of the post script `name`, if this session loaded it.
#[must_use]
pub fn post_text(name: &str) -> Option<String> {
    with(|s| s.posts.get(name).cloned()).flatten()
}

/// The session's machine file names, sorted.
#[must_use]
pub fn machine_names() -> Vec<String> {
    with(|s| s.machines.keys().cloned().collect()).unwrap_or_default()
}

/// The session's post script names, sorted.
#[must_use]
pub fn post_names() -> Vec<String> {
    with(|s| s.posts.keys().cloned().collect()).unwrap_or_default()
}

/// A machine file's text: the session's copy of `path` when it has one,
/// else the file on disk.
pub fn read_machine(path: &std::path::Path) -> std::io::Result<String> {
    match path.to_str().and_then(machine_text) {
        Some(text) => Ok(text),
        None => std::fs::read_to_string(path),
    }
}

/// A post script's text: the session's copy of `path` when it has one,
/// else the file on disk.
pub fn read_post(path: &str) -> std::io::Result<String> {
    match post_text(path) {
        Some(text) => Ok(text),
        None => std::fs::read_to_string(path),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_bad_file_is_refused_by_name_and_never_kept() {
        assert!(add_machine("unit-2950-bad.mch", "[]").expect_err("not a machine").contains("unit-2950-bad.mch"));
        assert!(add_post("unit-2950-bad.cps", "throw 1;").expect_err("does not run").contains("unit-2950-bad.cps"));
        assert!(!machine_names().contains(&"unit-2950-bad.mch".to_owned()));
        assert!(!post_names().contains(&"unit-2950-bad.cps".to_owned()));
        assert!(read_post("unit-2950-bad.cps").is_err(), "an unloaded name falls to the disk and fails");
        // A saved project that names a file this session did not load is
        // refused by that name at post time, never posted with a default.
        let post = crate::schema::PostRef::File("unit-2950-never.cps".to_owned());
        assert!(crate::posting::script_of(&post).expect_err("not loaded").contains("unit-2950-never.cps"));
    }
}
