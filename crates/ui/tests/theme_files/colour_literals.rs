//! The colour-literal guard (#1120): a colour in `crates/ui` is a named
//! token of the theme's table (`egui_app::theme`), never a literal
//! written at the draw site. A theme - ours or anyone's - can restyle
//! only what reads its tokens.
//!
//! The guard walks `crates/ui/src` and counts colour literals in each
//! file that is not a test file. Each count must equal its row in
//! `ALLOWED`:
//! - a file with a new literal, or a file not in the list, fails;
//! - a file whose literals moved to tokens fails until its row is
//!   lowered or removed, so the list only shrinks.
//!
//! What counts as a literal is `colour_scan`'s rule. Colour data kept as
//! plain numbers (a `[u8; 3]` const, a table of tuples) is not seen; the
//! #1120 family PRs move those by name.
//!
//! The theme columns and the contrast maths moved to the UI kit crate
//! with the theme (#4394), so they are no longer rows here. The
//! permanent rows are the verdict palettes and a colour that only
//! measures text. Each row is permanent: the draw sites of #1120 all
//! read tokens, so a new row needs a reason that is not a theme choice.
//! Provenance: our own test over our own source tree.
//!
//! A module of the `theme_files` suite (#3187: no new test binary). Its
//! gate map names `ui::egui_app`, so each UI change runs the guard.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use super::colour_scan::count;

/// `(path under crates/ui/src, literal count, why)`. Keep it sorted by
/// path. Each row is a `keep:` row: a colour that is not a choice for
/// a theme, with the reason.
#[rustfmt::skip]
const ALLOWED: &[(&str, usize, &str)] = &[
    ("egui_app/theme/verdict.rs", 15, "keep: the verdict palette table (#1120: ours, held to the colour-vision gate, no theme key)"),
    ("egui_app/viewport_layout.rs", 1, "keep: the colour only measures text, it is not drawn"),
];

fn src_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("src")
}

/// A test file: `tests.rs`, `*_tests.rs`, or anything under a `tests/` folder.
fn is_test_file(rel: &str) -> bool {
    let name = rel.rsplit('/').next().unwrap_or(rel);
    name == "tests.rs"
        || name.ends_with("_tests.rs")
        || rel.starts_with("tests/")
        || rel.contains("/tests/")
}

fn rust_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        panic!("read {}", dir.display());
    };
    for e in entries.flatten() {
        let p = e.path();
        if p.is_dir() {
            rust_files(&p, out);
        } else if p.extension().is_some_and(|x| x == "rs") {
            out.push(p);
        }
    }
}

fn counts() -> BTreeMap<String, usize> {
    let root = src_root();
    let mut files = Vec::new();
    rust_files(&root, &mut files);
    let mut out = BTreeMap::new();
    for f in files {
        let rel = f
            .strip_prefix(&root)
            .unwrap_or(&f)
            .to_string_lossy()
            .replace('\\', "/");
        if is_test_file(&rel) {
            continue;
        }
        let text = std::fs::read_to_string(&f).unwrap_or_else(|e| panic!("read {rel}: {e}"));
        let n = count(&text);
        if n > 0 {
            out.insert(rel, n);
        }
    }
    out
}

#[test]
fn no_colour_literal_outside_the_allowed_rows() {
    let found = counts();
    let allowed: BTreeMap<&str, usize> = ALLOWED.iter().map(|(p, n, _)| (*p, *n)).collect();
    let mut errors = Vec::new();
    for (rel, n) in &found {
        match allowed.get(rel.as_str()) {
            None => errors.push(format!(
                "{rel}: {n} colour literal(s), and the file has no row: read a theme token instead"
            )),
            Some(a) if n > a => errors.push(format!(
                "{rel}: {n} colour literals, the row allows {a}: read a theme token instead"
            )),
            Some(a) if n < a => errors.push(format!(
                "{rel}: {n} colour literals, the row says {a}: lower the row to {n}"
            )),
            Some(_) => {}
        }
    }
    for (rel, a) in &allowed {
        if !found.contains_key(*rel) {
            errors.push(format!(
                "{rel}: no colour literal left, the row says {a}: remove the row"
            ));
        }
    }
    let table: Vec<String> = found
        .iter()
        .map(|(p, n)| format!("    (\"{p}\", {n}, \"\"),"))
        .collect();
    assert!(
        errors.is_empty(),
        "{}\n\nToday's counts:\n{}",
        errors.join("\n"),
        table.join("\n")
    );
}

#[test]
fn the_rows_are_sorted_and_each_says_why() {
    let paths: Vec<&str> = ALLOWED.iter().map(|r| r.0).collect();
    let mut sorted = paths.clone();
    sorted.sort_unstable();
    sorted.dedup();
    assert_eq!(
        paths, sorted,
        "ALLOWED is sorted by path, one row for each file"
    );
    for (p, n, why) in ALLOWED {
        assert!(*n > 0, "{p}: a row with 0 literals is removed, not kept");
        assert!(!why.trim().is_empty(), "{p}: each row says why");
    }
}
