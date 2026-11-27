//! The user posts folder of the MCP post tools (#2074): where the
//! user's own `.cps` files live, what each one declares, and the plain
//! file name save that can only land inside that folder.
//!
//! Provenance: our own resolver, the same order as the user machines
//! folder (`machine::store`, the #287 decision): `CAMCRAFT_POSTS`, else
//! `<platform data dir>/camcraft/posts` from the platforms' published
//! per-user data locations. The save is `machine::store::write_atomic`
//! (write a temp file, then rename it over the target).

use std::ffi::OsString;
use std::path::{Path, PathBuf};

use serde_json::{json, Value};

use crate::machine::store::write_atomic;
use crate::post::{dialect, engine, js::Script, options};

/// The override: an absolute folder of user `.cps` files.
pub(crate) const ENV_VAR: &str = "CAMCRAFT_POSTS";

/// The user posts folder from an environment lookup, for the named
/// platform. Pure, so the order is testable without the process
/// environment. `None` = no home to put one in.
#[must_use]
pub(crate) fn user_dir_for(os: &str, get: impl Fn(&str) -> Option<OsString>) -> Option<PathBuf> {
    let abs = |k: &str| get(k).map(PathBuf::from).filter(|p| p.is_absolute());
    if let Some(dir) = abs(ENV_VAR) {
        return Some(dir);
    }
    let data = match os {
        "windows" => abs("APPDATA"),
        "macos" => abs("HOME").map(|h| h.join("Library").join("Application Support")),
        _ => abs("XDG_DATA_HOME").or_else(|| abs("HOME").map(|h| h.join(".local").join("share"))),
    }?;
    Some(data.join("camcraft").join("posts"))
}

/// The user posts folder for this process.
#[must_use]
pub(crate) fn user_dir() -> Option<PathBuf> {
    user_dir_for(std::env::consts::OS, |k| std::env::var_os(k))
}

/// The machine kinds a listing asks each shipped post about.
const KINDS: [crate::machine::Kind; 7] = {
    use crate::machine::Kind;
    [Kind::Mill, Kind::Lathe, Kind::MillTurn, Kind::Plasma, Kind::Waterjet, Kind::WireEdm, Kind::SinkerEdm]
};

/// What a script's manifest declares: description, program units,
/// file extension and the declared options. A script that does not
/// load gives `problem` and nothing else, never a guess.
pub(crate) fn manifest(cps: &str) -> Value {
    let read = || -> Result<Value, String> {
        let mut s = Script::load(cps).map_err(|e| e.to_string())?;
        let mut text = |k: &str| s.string_global(k).map_err(|e| e.to_string());
        let description = text("description")?;
        let program_units = text("programUnits")?;
        let extension = engine::extension_of(cps).map_err(|e| e.to_string())?;
        let opts = options::declared_options(cps).map_err(|e| e.to_string())?;
        Ok(json!({
            "description": description,
            "program_units": program_units,
            "extension": extension,
            "options": opts,
        }))
    };
    read().unwrap_or_else(|why| json!({ "problem": why }))
}

/// One shipped post: its registry name, what it declares, and the
/// machine kinds the registry lets it serve.
pub(crate) fn builtin_row(name: &str) -> Value {
    let mut row = match crate::posting::builtin_post_text(name) {
        Ok(cps) => manifest(cps),
        Err(why) => json!({ "problem": why }),
    };
    let serves: Vec<&str> = KINDS.iter().filter(|k| crate::posting::post_serves(name, **k)).map(|k| k.word()).collect();
    if let Some(o) = row.as_object_mut() {
        o.insert("name".into(), json!(name));
        o.insert("serves".into(), json!(serves));
    }
    row
}

/// One user script: its file, its path (what `select_post` takes), what
/// it declares, and the kinds its manifest states (`null` = none stated:
/// posting then judges it by its dialect).
pub(crate) fn user_row(path: &Path) -> Value {
    let file = path.file_name().map(|f| f.to_string_lossy().into_owned()).unwrap_or_default();
    let mut row = match std::fs::read_to_string(path) {
        Ok(cps) => {
            let mut m = manifest(&cps);
            let serves = match dialect::declared_serves(&cps) {
                Ok(Some(k)) => json!(k.iter().map(|k| k.word()).collect::<Vec<_>>()),
                Ok(None) => Value::Null,
                Err(why) => json!({ "problem": why.to_string() }),
            };
            if let Some(o) = m.as_object_mut() {
                o.insert("serves".into(), serves);
            }
            m
        }
        Err(e) => json!({ "problem": format!("{file}: {e}") }),
    };
    if let Some(o) = row.as_object_mut() {
        o.insert("file".into(), json!(file));
        o.insert("path".into(), json!(path.display().to_string()));
    }
    row
}

/// The `.cps` files of `dir`, by file name. A folder that does not
/// exist yet holds no file (the first save makes it).
pub(crate) fn user_files(dir: &Path) -> Result<Vec<PathBuf>, String> {
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(e) => return Err(format!("user posts folder {}: {e}", dir.display())),
    };
    let mut out: Vec<PathBuf> = entries
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| p.is_file() && p.extension().is_some_and(|x| x.eq_ignore_ascii_case("cps")))
        .collect();
    out.sort();
    Ok(out)
}

/// A plain file name made into a `.cps` path in `dir`. Refused when it
/// is empty, names a folder (`/`, `\`, `:`), starts with a dot (hidden
/// files, `..`) or has a control character - so a save lands only in
/// the user posts folder.
pub(crate) fn file_in(dir: &Path, name: &str) -> Result<PathBuf, String> {
    let n = name.trim();
    if n.is_empty() {
        return Err("\"file\" is empty - give a plain file name such as my-post.cps".into());
    }
    if n.contains(['/', '\\', ':']) || n.starts_with('.') || n.chars().any(char::is_control) {
        return Err(format!("\"file\" '{n}' must be a plain file name in the user posts folder, not a path"));
    }
    let lower = n.to_ascii_lowercase();
    let file = if lower.ends_with(".cps") { n.to_owned() } else { format!("{n}.cps") };
    Ok(dir.join(file))
}

/// Save `source` as `file` in `dir`: the script must load and run on an
/// empty stream first (`SelectPost`'s front door), an existing file is
/// replaced only with `overwrite`, and the write is atomic. Returns the
/// path and whether a file was replaced.
pub(crate) fn save(dir: &Path, file: &str, source: &str, overwrite: bool) -> Result<(PathBuf, bool), String> {
    let path = file_in(dir, file)?;
    engine::run(source, &Value::Null, "").map_err(|e| format!("post script does not run, not saved: {e}"))?;
    let existed = path.exists();
    if existed && !overwrite {
        return Err(format!("{} already exists - pass \"overwrite\": true to replace it", path.display()));
    }
    write_atomic(&path, source).map_err(|e| e.to_string())?;
    Ok((path, existed))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_folder_follows_the_override_then_the_platform_data_dir() {
        let env = |pairs: &'static [(&'static str, &'static str)]| {
            move |k: &str| pairs.iter().find(|(n, _)| *n == k).map(|(_, v)| OsString::from(v))
        };
        let over = user_dir_for("linux", env(&[(ENV_VAR, "/p/posts"), ("HOME", "/h")]));
        assert_eq!(over, Some(PathBuf::from("/p/posts")));
        let rel = user_dir_for("linux", env(&[(ENV_VAR, "rel"), ("HOME", "/h")]));
        assert_eq!(rel, Some(PathBuf::from("/h/.local/share/camcraft/posts")), "a relative override is ignored");
        let mac = user_dir_for("macos", env(&[("HOME", "/h")]));
        assert_eq!(mac, Some(PathBuf::from("/h/Library/Application Support/camcraft/posts")));
        assert_eq!(user_dir_for("windows", env(&[("HOME", "/h")])), None);
    }

    #[test]
    fn a_file_name_must_be_plain() {
        let d = Path::new("/u");
        assert_eq!(file_in(d, "mine"), Ok(PathBuf::from("/u/mine.cps")));
        assert_eq!(file_in(d, "Mine.CPS"), Ok(PathBuf::from("/u/Mine.CPS")));
        for bad in ["", "  ", "../x.cps", "a/b.cps", "a\\b.cps", "C:x.cps", ".hidden.cps", "a\nb.cps"] {
            assert!(file_in(d, bad).is_err(), "{bad:?} must be refused");
        }
    }
}
