//! #1253: every message the troubleshooting page quotes still exists in the
//! sources, so a reworded refusal cannot leave the page describing a message
//! nobody sees. Provenance: our own docs convention (see the page's source).
//!
//! Gate map (#1253): covers site/content/guide/reference/troubleshooting.md vs crates/*/src.
//!
//! #4053: the Model to Machine page names machine files, posts and machine
//! kinds. Each of those claims must agree with `assets/machines/*.mch` and
//! with the post table of `posting.md`.

use std::fs;
use std::path::{Path, PathBuf};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn rs_files(dir: &Path, out: &mut String) {
    let Ok(rd) = fs::read_dir(dir) else { return };
    for e in rd.flatten() {
        let p = e.path();
        if p.is_dir() {
            rs_files(&p, out);
        } else if p.extension().is_some_and(|x| x == "rs")
            && let Ok(t) = fs::read_to_string(&p)
        {
            out.push_str(&join_continuations(&t));
        }
    }
}

/// #3619: a checkout with `core.autocrlf=true` holds CRLF line ends, so
/// bring every text that the test reads to LF before it is searched.
fn normalise_eol(t: &str) -> String {
    t.replace("\r\n", "\n")
}

/// Join string-literal line continuations ("...\<newline>   ...").
fn join_continuations(t: &str) -> String {
    let t = normalise_eol(t);
    let mut joined = String::new();
    let mut it = t.split("\\\n");
    if let Some(first) = it.next() {
        joined.push_str(first);
    }
    for part in it {
        joined.push_str(part.trim_start());
    }
    joined
}

/// Backticked spans of `### ` headings, split at `<...>` and `...`
/// placeholders into literal fragments of 8+ characters.
fn stems(page: &str) -> Vec<String> {
    let mut out = Vec::new();
    for line in page.lines().filter(|l| l.starts_with("### ")) {
        for (i, span) in line.split('`').enumerate() {
            if i % 2 == 0 {
                continue;
            }
            let mut frag = String::new();
            let mut depth = 0;
            let flush = |frag: &mut String, out: &mut Vec<String>| {
                let t = frag.trim_matches(|c: char| !c.is_alphanumeric()).to_owned();
                if t.len() >= 8 {
                    out.push(t);
                }
                frag.clear();
            };
            let chars: Vec<char> = span.chars().collect();
            let mut k = 0;
            while k < chars.len() {
                let c = chars[k];
                if c == '<' {
                    depth += 1;
                    flush(&mut frag, &mut out);
                } else if c == '>' && depth > 0 {
                    depth -= 1;
                } else if depth == 0 && chars[k..].starts_with(&['.', '.', '.']) {
                    flush(&mut frag, &mut out);
                    k += 2;
                } else if depth == 0 {
                    frag.push(c);
                }
                k += 1;
            }
            flush(&mut frag, &mut out);
        }
    }
    out
}

#[test]
fn troubleshooting_messages_still_exist_in_the_sources() {
    // #1908 grouped the guide by job stage: the page is under reference/.
    let page = normalise_eol(
        &fs::read_to_string(root().join("site/content/guide/reference/troubleshooting.md"))
            .expect("troubleshooting page is readable"),
    );
    let mut src = String::new();
    for entry in fs::read_dir(root().join("crates"))
        .expect("crates dir")
        .flatten()
    {
        rs_files(&entry.path().join("src"), &mut src);
    }
    let found = stems(&page);
    assert!(
        found.len() > 20,
        "the stem extractor found only {} stems",
        found.len()
    );
    let stale: Vec<&String> = found.iter().filter(|s| !src.contains(s.as_str())).collect();
    assert!(
        stale.is_empty(),
        "troubleshooting.md quotes message stems no longer in crates/*/src (reworded or removed?): {stale:#?}"
    );
}

/// #3619: a source with CRLF line ends and a backslash continuation joins
/// like an LF source, so the stem spans the two lines.
#[test]
fn join_continuations_accepts_crlf_line_ends() {
    let lf = "let m = \"the stem that spans \\\n        two lines\";\n";
    let crlf = lf.replace('\n', "\r\n");
    assert!(join_continuations(lf).contains("the stem that spans two lines"));
    let joined = join_continuations(&crlf);
    assert!(
        joined.contains("the stem that spans two lines"),
        "CRLF continuation was not joined: {joined:?}"
    );
}

// ---- #4053: Model to Machine page vs the machine files and posting.md ----

fn read(rel: &str) -> String {
    normalise_eol(&fs::read_to_string(root().join(rel)).unwrap_or_else(|e| panic!("{rel}: {e}")))
}

/// The cells of each body row of the first table under `heading`.
fn table_rows(page: &str, heading: &str) -> Vec<Vec<String>> {
    let body = page
        .split_once(&format!("\n{heading}\n"))
        .unwrap_or_else(|| panic!("heading {heading:?} is missing"))
        .1;
    let body = body.split("\n## ").next().unwrap_or(body);
    body.lines()
        .filter(|l| l.starts_with('|') && !l.starts_with("|---"))
        .skip(1)
        .map(|l| l.trim_matches('|').split('|').map(|c| c.trim().to_owned()).collect())
        .collect()
}

/// Backticked spans of `cell` that end with `ext`.
fn ticked(cell: &str, ext: &str) -> Vec<String> {
    cell.split('`')
        .enumerate()
        .filter(|(i, s)| i % 2 == 1 && s.ends_with(ext))
        .map(|(_, s)| s.to_owned())
        .collect()
}

fn machine(file: &str) -> serde_json::Value {
    let rel = format!("assets/machines/{file}");
    assert!(!file.contains('*'), "{file}: name each machine file, a glob can include a wrong one");
    serde_json::from_str(&read(&rel)).unwrap_or_else(|e| panic!("{rel}: {e}"))
}

const PAGE: &str = "site/content/guide/model-to-machine.md";

/// Review of PR #4059, findings 2 and 4: each row of "Machine and post
/// pairs" names only machine files that `posting.md` lists for that post,
/// and a machine file with no `controller.post` says that the user selects
/// the post.
#[test]
fn model_to_machine_pairs_agree_with_posting_and_machine_files() {
    let rows = table_rows(&read(PAGE), "## Machine and post pairs");
    let posting = table_rows(&read("site/content/guide/check-and-post/posting.md"), "## Post processors");
    let mut checked = 0;
    for row in &rows {
        assert_eq!(row.len(), 5, "row has not 5 cells: {row:?}");
        assert!(!row[2].contains('*') && !row[3].contains('*'), "a glob in {row:?}");
        let scripts = ticked(&row[3], ".cps");
        for s in &scripts {
            assert!(root().join("assets/posts").join(s).is_file(), "{s} is not in assets/posts");
        }
        let listed: String = posting
            .iter()
            .filter(|p| p.len() >= 4 && scripts.iter().any(|s| ticked(&p[1], ".cps").contains(s)))
            .map(|p| p[3].clone())
            .collect();
        for file in ticked(&row[2], ".mch") {
            let m = machine(&file);
            assert!(
                listed.contains(&format!("`{file}`")),
                "{file}: posting.md does not list it for {scripts:?}"
            );
            if m["controller"]["post"].as_str().is_none() {
                assert!(
                    row[4].to_lowercase().contains("select the post manually"),
                    "{file} states no controller.post: the State cell must say \"Select the post manually\": {row:?}"
                );
                assert!(
                    file != "sodick-alc400g.mch" || row[1].contains("Not confirmed"),
                    "machines.md: the ALC400G control is not confirmed, the Control cell must say so: {row:?}"
                );
            }
            checked += 1;
        }
    }
    assert!(checked >= 10, "only {checked} machine files were checked");
}

/// Review of PR #4059, finding 3: a shipped machine that the "Kinds of CNC
/// machines" table names is in the row of the `kind` that its file declares.
#[test]
fn model_to_machine_kinds_agree_with_machine_files() {
    let rows = table_rows(&read(PAGE), "## Kinds of CNC machines");
    let kind_of = |cell: &str| match cell {
        c if c.starts_with("Mill") => "mill",
        c if c.starts_with("Router") => "router",
        c if c.starts_with("Lathe") => "lathe",
        c if c.starts_with("Plasma") => "plasma",
        c if c.starts_with("Waterjet") => "waterjet",
        c if c.starts_with("Wire EDM") => "wire_edm",
        c if c.starts_with("Sinker EDM") => "sinker_edm",
        c => panic!("no machine kind for the row {c:?}"),
    };
    let mut checked = 0;
    for e in fs::read_dir(root().join("assets/machines")).expect("machines dir").flatten() {
        let name = e.file_name().to_string_lossy().into_owned();
        if !name.ends_with(".mch") {
            continue;
        }
        let m = machine(&name);
        let (Some(model), Some(kind)) = (m["general"]["model"].as_str(), m["general"]["kind"].as_str()) else {
            continue;
        };
        for row in rows.iter().filter(|r| r.len() >= 3) {
            if row[2].to_lowercase().contains(&model.to_lowercase()) {
                assert_eq!(kind_of(&row[0]), kind, "{name} ({model}) is in the row {:?}", row[0]);
                checked += 1;
            }
        }
    }
    assert!(checked >= 6, "only {checked} named machines were checked");
}

/// Review of PR #4059, finding 1: no machine simulation ships (#2584 has no
/// caller). The page names what ships: the work envelope outline.
#[test]
fn model_to_machine_claims_no_moving_machine_in_simulation() {
    let page = read(PAGE).replace('\n', " ").to_lowercase();
    assert!(!page.contains("moving machine in the simulation"), "no machine simulation ships");
    assert!(!page.contains("simulates the machine"), "no machine simulation ships");
    assert!(page.contains("**show work envelope**"), "the page must name Show work envelope");
}
