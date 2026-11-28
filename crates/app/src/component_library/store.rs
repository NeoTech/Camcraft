//! #3645: where the user's component library lives and how it is
//! edited: one `components.camcomponents` in the user components folder,
//! resolved as the user fixtures folder is (`fixture_library::store`):
//! `CAMCRAFT_COMPONENTS`, else `<platform data dir>/camcraft/components`.
//! A save and a delete are file edits, not document commands: nothing
//! here touches a world or the undo history, and each write is all or
//! nothing.
//!
//! Provenance: the platforms' published per-user data locations (Windows
//! `%APPDATA%`, macOS `~/Library/Application Support`, the XDG Base
//! Directory spec); the atomic save is the write-temp-then-rename
//! pattern from the `std::fs::rename` docs.

use std::ffi::OsString;
use std::path::{Path, PathBuf};

use super::{is_builtin_id, parse, to_text, LibraryComponent};

/// The override: an absolute folder that holds the library file.
pub const ENV_VAR: &str = "CAMCRAFT_COMPONENTS";
/// The library's file name in the folder.
pub const LIBRARY_FILE: &str = "components.camcomponents";

/// The user components folder from an environment lookup, for the named
/// platform (`std::env::consts::OS`). Pure, so the order is testable
/// without the process environment. `None`: no home to put one in.
#[must_use]
pub fn user_dir_for(os: &str, get: impl Fn(&str) -> Option<OsString>) -> Option<PathBuf> {
    let abs = |k: &str| get(k).map(PathBuf::from).filter(|p| p.is_absolute());
    if let Some(dir) = abs(ENV_VAR) {
        return Some(dir);
    }
    let data = match os {
        "windows" => abs("APPDATA"),
        "macos" => abs("HOME").map(|h| h.join("Library").join("Application Support")),
        _ => abs("XDG_DATA_HOME").or_else(|| abs("HOME").map(|h| h.join(".local").join("share"))),
    }?;
    Some(data.join("camcraft").join("components"))
}

/// The user components folder for this process.
#[must_use]
pub fn user_dir() -> Option<PathBuf> {
    user_dir_for(std::env::consts::OS, |k| std::env::var_os(k))
}

/// The library file in `dir`.
#[must_use]
pub fn library_path(dir: &Path) -> PathBuf {
    dir.join(LIBRARY_FILE)
}

/// The library at `path`. A missing file is the empty library (nothing
/// saved yet); an unreadable or refused file is an error, never "empty".
pub fn load(path: &Path) -> Result<Vec<LibraryComponent>, String> {
    match std::fs::read_to_string(path) {
        Ok(text) => parse(&text).map_err(|e| format!("{}: {e}", path.display())),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Vec::new()),
        Err(e) => Err(format!("{}: {e}", path.display())),
    }
}

/// Write `entries` to `path` atomically: a temp file in the same folder,
/// flushed, then renamed over the target. The folder is made when it is
/// missing. On a failure the temp file is removed and the target stays
/// as it was. A built-in id, or an id twice, is refused before a write.
pub fn save(path: &Path, entries: &[LibraryComponent]) -> Result<(), String> {
    use std::io::Write;
    for (i, e) in entries.iter().enumerate() {
        let id = e.component.id();
        if is_builtin_id(id) {
            return Err(format!("tool component '{id}' is built in - it is not saved to the user library"));
        }
        if entries[..i].iter().any(|o| o.component.id() == id) {
            return Err(format!("tool component '{id}' is saved twice"));
        }
    }
    let text = to_text(entries)?;
    let io = |e: std::io::Error| format!("{}: {e}", path.display());
    let dir = path.parent().ok_or_else(|| format!("{} has no folder", path.display()))?;
    std::fs::create_dir_all(dir).map_err(io)?;
    let name = path.file_name().map_or_else(|| "components".into(), |n| n.to_string_lossy().into_owned());
    let tmp = dir.join(format!(".{name}.{}.tmp", std::process::id()));
    let wrote = (|| {
        let mut f = std::fs::File::create(&tmp)?;
        f.write_all(text.as_bytes())?;
        f.sync_all()?;
        drop(f);
        std::fs::rename(&tmp, path)
    })();
    if let Err(e) = wrote {
        let _ = std::fs::remove_file(&tmp);
        return Err(io(e));
    }
    Ok(())
}

/// Add `entry` to the library at `path`, or replace the entry of its id.
pub fn put(path: &Path, entry: LibraryComponent) -> Result<(), String> {
    let mut lib = load(path)?;
    match lib.iter_mut().find(|e| e.component.id() == entry.component.id()) {
        Some(slot) => *slot = entry,
        None => lib.push(entry),
    }
    save(path, &lib)
}

/// Remove the entry `id` from the library at `path`. There is no undo,
/// as for a deleted machine file.
pub fn delete(path: &Path, id: &str) -> Result<(), String> {
    let id = id.trim();
    if is_builtin_id(id) {
        return Err(format!("tool component '{id}' is built in - it can not be deleted"));
    }
    let mut lib = load(path)?;
    let before = lib.len();
    lib.retain(|e| e.component.id() != id);
    if lib.len() == before {
        return Err(format!("no tool component '{id}' in the library"));
    }
    save(path, &lib)
}
