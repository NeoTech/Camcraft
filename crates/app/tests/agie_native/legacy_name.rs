//! Gate map: covers app::posting::aliases with app::project and
//! app::posting::agie - a project saved under an old post name posts.
//! Provenance: issue #4000 "Done means" and the #4022 review: a project
//! that an old release saved with an old Agie post name opens, posts the
//! unchanged control-checked job, and then saves the new name.

use super::{punch, separation};
use camcraft_app::commands::{Command, SetPostScriptOptions};
use camcraft_app::posting::{self, agie};
use camcraft_app::project;
use camcraft_app::schema::{PostRef, PostScriptOptions};
use std::collections::BTreeMap;
use std::fs;

/// The two old names, in the order the post had them.
const OLD_NAMES: [&str; 2] = ["Agie native wire EDM", "Native wire EDM job files"];

/// The project text of the control-checked punch, with its separation
/// option saved under the post key, as the current release writes it.
fn current_project_text() -> String {
    let world = punch(20.0, [0., 0., 20.], [-30., 0.]);
    let key = format!("builtin:{}", agie::POST);
    let map = PostScriptOptions { by_post: BTreeMap::from([(key, separation(1.0))]) };
    SetPostScriptOptions::new(&world, map).expect("options").apply(&world);
    let dir = tempfile::tempdir().expect("temp dir");
    let path = dir.path().join("current.camproj");
    project::save(&world, &path).expect("save");
    fs::read_to_string(&path).expect("read")
}

#[test]
fn a_project_saved_with_each_old_name_posts_the_unchanged_job_and_saves_the_new_name() {
    let new_text = current_project_text();
    assert!(new_text.contains(&format!("builtin:{}", agie::POST)), "{new_text}");
    for old in OLD_NAMES {
        // The file as an old release wrote it: the old name as the post and
        // in the option key.
        let old_text = new_text.replace(agie::POST, old);
        assert!(!old_text.contains(agie::POST), "{old}");
        assert!(old_text.contains(&format!("builtin:{old}")), "{old}");

        let opened = camcraft_app::world::new_world();
        project::open(&opened, &old_text).expect("a project with the old name opens");
        assert_eq!(posting::document_post_of(&opened), PostRef::Named(agie::POST.into()), "{old}");

        // The post reads the separation option from the moved key: no
        // per-run override goes in.
        let temp = tempfile::tempdir().expect("temp dir");
        let dest = temp.path().join("CCSEP20");
        posting::post_mounts_with_script_options(&opened, &dest, None)
            .unwrap_or_else(|e| panic!("{old}: the opened project does not post: {e}"));
        for (name, golden) in [
            ("CCSEP20.ISO", include_bytes!("../golden/nc/agie-native/CCSEP20.ISO").as_slice()),
            ("CCSEP20.SBR", include_bytes!("../golden/nc/agie-native/CCSEP20.SBR").as_slice()),
            ("CCSEP20.SBL", include_bytes!("../golden/nc/agie-native/CCSEP20.SBL").as_slice()),
        ] {
            let posted = fs::read(dest.join(name)).unwrap_or_else(|e| panic!("{old}: {name}: {e}"));
            assert!(posted == golden, "{old}: {name} differs from the golden");
        }

        // A save after the post writes the new name only.
        let path = temp.path().join("saved.camproj");
        project::save(&opened, &path).expect("save again");
        let saved = fs::read_to_string(&path).expect("read");
        assert!(!saved.contains(old), "{old}: {saved}");
        assert!(saved.contains(&format!("builtin:{}", agie::POST)), "{old}: {saved}");
    }
}
