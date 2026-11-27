//! The reference files of `camcraft-sim check` (#2586): the `.cps`
//! manifest (read, no hook called), the dialect profile, and the `.mch`
//! machine with its part tree.
//! Provenance: our own post manifest and machine-file grammars
//! (`assets/posts`, `assets/machines/AGENTS.md`); the dialect profiles
//! are `camcraft-gcode`'s.

use camcraft_gcode::Dialect;
use camcraft_post::machine::{kind, parse_mch, parse_mch_tree, Kind, Machine, MachineTree};
use camcraft_post::post::{dialect, engine, js::Script, work_offset};

/// The dialect names that `--dialect` and a post's file name can give.
pub const DIALECTS: &[&str] = &["rs274", "fanuc", "haas", "tormach", "grblhal", "tinyg", "carvera"];

/// The profile of a dialect name, or `None` for a name this reader does not know.
#[must_use]
pub fn dialect_named(name: &str) -> Option<Dialect> {
    match name.to_ascii_lowercase().as_str() {
        "rs274" | "linuxcnc" => Some(Dialect::rs274()),
        "fanuc" => Some(Dialect::fanuc()),
        "haas" => Some(Dialect::haas()),
        "tormach" => Some(Dialect::tormach()),
        "grblhal" => Some(Dialect::grblhal()),
        "tinyg" => Some(Dialect::tinyg()),
        "carvera" => Some(Dialect::carvera()),
        _ => None,
    }
}

/// What a post's manifest states. Each field comes from a top-level
/// variable of the script; no `on...` hook is called.
#[derive(Clone, Debug, PartialEq)]
pub struct PostManifest {
    /// The file name, for the report.
    pub name: String,
    pub extension: String,
    /// `serves`, or `None` where the manifest states none.
    pub serves: Option<Vec<Kind>>,
    /// The words of `programUnits` (`mm`, `inch`), or `None`.
    pub units: Option<Vec<String>>,
    pub work_offset_max: u32,
    /// The dialect that the post's file name names, if it names one.
    pub dialect: Option<String>,
}

/// Read the manifest of the post at `path`.
pub fn read_post(path: &str) -> Result<PostManifest, String> {
    let text = std::fs::read_to_string(path).map_err(|e| format!("cannot read post {path}: {e}"))?;
    let bad = |e: camcraft_post::post::PostError| format!("post {path}: {e}");
    let extension = engine::extension_of(&text).map_err(bad)?;
    let serves = dialect::declared_serves(&text).map_err(bad)?;
    let units = Script::load(&text)
        .and_then(|mut s| s.string_global("programUnits"))
        .map_err(bad)?
        .map(|u| u.split_whitespace().map(str::to_owned).collect());
    let work_offset_max = work_offset::work_offsets(&text).map_err(bad)?.max;
    let stem = std::path::Path::new(path).file_stem().and_then(|s| s.to_str()).unwrap_or_default();
    let dialect = dialect_named(stem).map(|_| stem.to_ascii_lowercase());
    let name = std::path::Path::new(path).file_name().and_then(|s| s.to_str()).unwrap_or(path).to_owned();
    Ok(PostManifest { name, extension, serves, units, work_offset_max, dialect })
}

/// The kinds a post serves: what it states, or milling where it states
/// none (the reading of a post with no `serves`, #1266).
#[must_use]
pub fn serves(post: &PostManifest, k: Kind) -> bool {
    post.serves.as_ref().map_or(k == Kind::Mill, |list| list.contains(&k))
}

/// The machine of a `.mch` file: its limits and its part tree.
#[derive(Clone, Debug)]
pub struct MachineFile {
    pub name: String,
    pub machine: Machine,
    pub tree: MachineTree,
    pub kind: Kind,
}

/// Read the machine file at `path`. A machine that is not a mill
/// refuses: this slice judges 3-axis milling only.
pub fn read_machine(path: &str) -> Result<MachineFile, String> {
    let text = std::fs::read_to_string(path).map_err(|e| format!("cannot read machine {path}: {e}"))?;
    let machine = parse_mch(&text).map_err(|e| format!("machine {path}: {e}"))?;
    let tree = parse_mch_tree(&text).map_err(|e| format!("machine {path}: {e}"))?;
    let k = kind(Some(&machine));
    if k != Kind::Mill {
        return Err(format!("machine {path} is a {k:?} machine: this check judges 3-axis milling programs only"));
    }
    let name = std::path::Path::new(path).file_name().and_then(|s| s.to_str()).unwrap_or(path).to_owned();
    Ok(MachineFile { name, machine, tree, kind: k })
}

/// The dialect for the run: `--dialect` first, then the post's file name.
pub fn choose_dialect(flag: Option<&str>, post: Option<&PostManifest>) -> Result<Dialect, String> {
    if let Some(name) = flag {
        return dialect_named(name).ok_or_else(|| format!("--dialect {name:?} is not one of {}", DIALECTS.join(", ")));
    }
    match post.and_then(|p| p.dialect.as_deref()).and_then(dialect_named) {
        Some(d) => Ok(d),
        None => Err(format!(
            "the post's file name names no dialect profile: give --dialect ({})",
            DIALECTS.join(", ")
        )),
    }
}
