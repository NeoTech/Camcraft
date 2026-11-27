//! Gate map: covers app::posting::aliases (the hidden alias table),
//! app::posting::agie (the registry name), app::project (a saved post name
//! and option key), app::commands::SelectPost.
//! Provenance: issues #3537 and #4000 - the Agie Wire EDM post was renamed
//! two times; its old names stay hidden aliases so a saved project still
//! opens and posts.

use std::collections::BTreeMap;
use std::path::Path;

use camcraft_app::commands::{Command, History, SelectPost, SetPostScriptOptions};
use camcraft_app::post::options::OptionValue;
use camcraft_app::posting::{self, agie, aliases};
use camcraft_app::schema::{PostRef, PostScriptOptions};
use camcraft_app::{project, setup_post_options};

/// The two old names, in the order the post had them.
const OLD_NAMES: [&str; 2] = ["Agie native wire EDM", "Native wire EDM job files"];

fn tmp_path(tag: &str) -> std::path::PathBuf {
    std::env::temp_dir().join(format!("camcraft_alias_{tag}_{}.camproj", std::process::id()))
}

#[test]
fn the_new_name_is_listed_and_the_old_names_are_not() {
    assert_eq!(agie::POST, "Agie Wire EDM (AgieVision)");
    assert!(posting::WIRE_POSTS.contains(&agie::POST));
    assert!(posting::BUILTINS.iter().any(|(n, _)| *n == agie::POST));
    for old in OLD_NAMES {
        assert!(!posting::WIRE_POSTS.contains(&old), "{old} is listed");
        assert!(posting::BUILTINS.iter().all(|(n, _)| *n != old), "{old} is a registry name");
    }
}

#[test]
fn the_alias_table_resolves_both_old_names() {
    let olds: Vec<&str> = aliases::POST_ALIASES.iter().map(|(old, _)| *old).collect();
    assert_eq!(olds, OLD_NAMES);
    for (old, new) in aliases::POST_ALIASES {
        assert!(posting::BUILTINS.iter().any(|(n, _)| n == new), "{new} is not a registry name");
        assert_eq!(aliases::current_name(old), *new);
        assert_eq!(aliases::current_ref(PostRef::Named((*old).into())), PostRef::Named((*new).into()));
        assert_eq!(aliases::current_key(&format!("builtin:{old}")), format!("builtin:{new}"));
    }
    assert_eq!(aliases::current_name("Generic ISO Wire EDM"), "Generic ISO Wire EDM");
    assert_eq!(aliases::current_key("builtin:Fanuc"), "builtin:Fanuc");
    assert_eq!(aliases::current_key("file:Native wire EDM job files"), "file:Native wire EDM job files");
}

#[test]
fn each_old_name_selects_the_new_post_by_command_script_and_lookup() {
    for old in OLD_NAMES {
        assert!(agie::is_native(&PostRef::Named(old.into())), "{old}");

        // A script or MCP `cps` with the old name resolves to the new name.
        let (post, text) = posting::resolve_cps(old).expect("the alias resolves");
        assert_eq!(post, PostRef::Named(agie::POST.into()));
        assert_eq!(text, posting::builtin_post_text(agie::POST).expect("the post text reads"));

        // The command stores the registry name; undo restores the previous post.
        let world = camcraft_app::world::new_world();
        let before = posting::document_post_of(&world);
        let mut h = History::default();
        h.execute(&world, SelectPost::new(&world, PostRef::Named(old.into())).expect("the alias selects"));
        assert_eq!(posting::document_post_of(&world), PostRef::Named(agie::POST.into()));
        assert_eq!(posting::post_ref_label(&posting::document_post_of(&world)), format!("{} (built-in)", agie::POST));
        assert!(
            SelectPost::new(&world, PostRef::Named(agie::POST.into())).is_err(),
            "the alias and the name are the same post"
        );
        h.undo(&world);
        assert_eq!(posting::document_post_of(&world), before);
    }
}

#[test]
fn a_project_saved_with_each_old_name_opens_and_saves_the_new_name() {
    let world = camcraft_app::world::new_world();
    let mut h = History::default();
    h.execute(&world, SelectPost::new(&world, PostRef::Named(agie::POST.into())).expect("select"));
    let key = format!("builtin:{}", agie::POST);
    let ov = BTreeMap::from([("separationLengthMm".to_owned(), OptionValue::Float(1.0))]);
    let map = PostScriptOptions { by_post: BTreeMap::from([(key.clone(), ov.clone())]) };
    let cmd = SetPostScriptOptions::new(&world, map).expect("options");
    cmd.apply(&world);
    let p = tmp_path("current");
    project::save(&world, &p).expect("save");
    let new_text = std::fs::read_to_string(&p).expect("read");
    let _ = std::fs::remove_file(&p);
    assert!(new_text.contains(agie::POST), "{new_text}");

    for (i, old) in OLD_NAMES.iter().enumerate() {
        // Write a file as an old release wrote it: the old name as the
        // post and in the option key.
        let old_text = new_text.replace(agie::POST, old);
        assert!(old_text.contains(&format!("builtin:{old}")), "{old_text}");

        let opened = camcraft_app::world::new_world();
        project::open(&opened, &old_text).expect("a project with the old name opens");
        assert_eq!(posting::document_post_of(&opened), PostRef::Named(agie::POST.into()));
        // The script resolves here; `agie_native::legacy_name` posts the
        // opened project and compares its job files with the goldens.
        assert!(posting::script_source(&opened).is_ok(), "the old project resolves the shipped script");
        let doc = setup_post_options::script_document(&opened);
        assert_eq!(doc.for_post(&key), &ov, "the option under the old key moves to the new key");
        assert!(doc.by_post.keys().all(|k| !k.contains(old)));

        // A save after the open writes the new name only.
        let p = tmp_path(&format!("legacy{i}"));
        project::save(&opened, &p).expect("save again");
        let saved = std::fs::read_to_string(&p).expect("read");
        let _ = std::fs::remove_file(&p);
        assert!(!saved.contains(old), "{saved}");
        assert!(saved.contains(&key), "{saved}");
    }
}

#[test]
fn an_option_map_under_each_old_key_normalizes_to_the_new_key() {
    for old in OLD_NAMES {
        let ov = BTreeMap::from([("separationLengthMm".to_owned(), OptionValue::Float(1.0))]);
        let mut map = PostScriptOptions { by_post: BTreeMap::from([(format!("builtin:{old}"), ov.clone())]) };
        map.normalize();
        assert_eq!(map.for_post(&format!("builtin:{}", agie::POST)), &ov);
        assert_eq!(map.by_post.len(), 1);
    }
}

/// The text files under `dir`, recursively.
fn text_files(dir: &Path, out: &mut Vec<std::path::PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else { return };
    for e in entries.flatten() {
        let p = e.path();
        if p.is_dir() {
            text_files(&p, out);
        } else if p.extension().is_some_and(|x| x == "rs" || x == "cps") {
            out.push(p);
        }
    }
}

/// #4000 "Done means": the source names the post "Agie Wire EDM", never
/// "native wire EDM" or "native controller". Two files keep an old name on
/// purpose: the alias table, and the script receipt `format` value that
/// scripts read.
#[test]
fn no_source_text_names_the_post_native_wire_edm() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let mut files = Vec::new();
    for dir in ["crates/app/src", "crates/ui/src", "crates/geometry/src"] {
        text_files(&root.join(dir), &mut files);
    }
    files.push(root.join("assets/posts/agie-native.cps"));
    assert!(files.len() > 100, "the source tree was not found under {}", root.display());
    let kept = ["posting_aliases.rs", "report.rs"];
    for f in &files {
        let text = std::fs::read_to_string(f).unwrap_or_default();
        let keep = f.file_name().and_then(|n| n.to_str()).is_some_and(|n| kept.contains(&n));
        for (n, line) in text.lines().enumerate() {
            let lower = line.to_lowercase();
            if keep && (line.contains("\"Agie native wire EDM\"") || line.contains("\"Native wire EDM job files\"")) {
                continue;
            }
            assert!(
                !lower.contains("native wire edm") && !lower.contains("native controller"),
                "{}:{}: {line}",
                f.display(),
                n + 1
            );
        }
    }
}
