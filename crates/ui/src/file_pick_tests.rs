//! #31: what the browser's async file picker hands over - a name and the
//! file's bytes - lands through the same commands as a file opened from
//! disk, and a failed read or a refused file is said by name. Run natively:
//! the picker's DOM half is wasm32-only, but everything after the bytes
//! arrive is this shared code.
//!
//! Gate map (M4.4): covers ui::file_pick, ui::state (state_picked).
//! Provenance: our own tests.

use std::path::{Path, PathBuf};

use camcraft_app::{model, project};

use crate::file_dialog::FileDialog;
use crate::file_pick::{self, PickFor, PickedFile};
use crate::state::{CommandKind, State};

fn testdata(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../testdata/models").join(name)
}

fn picked(purpose: PickFor, name: &str, bytes: Result<Vec<u8>, String>) -> PickedFile {
    PickedFile { purpose, name: name.to_string(), bytes }
}

fn has_model(state: &State, name: &str) -> bool {
    model::find_by_name(&state.world, name).is_some()
}

#[test]
fn a_picked_model_imports_through_the_undoable_command() {
    let mut state = State::new();
    let bytes = std::fs::read(testdata("cube.stl")).expect("cube.stl");
    file_pick::deliver(picked(PickFor::Model, "cube.stl", Ok(bytes)));
    state.drain_actions();
    assert!(has_model(&state, "cube"), "status: {}", state.status);
    assert!(state.status.starts_with("imported cube"), "status: {}", state.status);
    assert!(file_pick::take_picked().is_empty(), "the inbox is drained");
    // One undo step takes it away, like an import from disk.
    state.run(CommandKind::Undo);
    assert!(!has_model(&state, "cube"));
    state.run(CommandKind::Redo);
    assert!(has_model(&state, "cube"));
}

#[test]
fn a_picked_file_that_cannot_be_used_is_refused_by_name() {
    let mut state = State::new();
    file_pick::deliver(picked(PickFor::Model, "big.stl", Err("over the limit".into())));
    state.drain_actions();
    assert_eq!(state.status, "import refused: big.stl could not be read: over the limit");
    file_pick::deliver(picked(PickFor::Model, "part.igs", Ok(b"anything".to_vec())));
    state.drain_actions();
    assert!(state.status.starts_with("import refused:") && state.status.contains("IGS"), "{}", state.status);
    file_pick::deliver(picked(PickFor::Model, "bad.stl", Ok(vec![0xff; 97])));
    state.drain_actions();
    assert!(state.status.starts_with("import refused:"), "{}", state.status);
    file_pick::deliver(picked(PickFor::Project, "p.camproj", Err("the browser could not read it".into())));
    state.drain_actions();
    assert_eq!(state.status, "open failed: p.camproj could not be read: the browser could not read it");
    assert!(model::find_by_name(&state.world, "big").is_none());
    assert!(!state.history.is_dirty(), "nothing reached the history");
}

#[test]
fn a_picked_project_opens_in_place_of_the_document() {
    // A saved project with one model in it.
    let mut author = State::new();
    author.run(CommandKind::ImportFile(testdata("cube.stl")));
    while author.import_in_progress() {
        author.drain_actions();
        std::thread::yield_now();
    }
    assert!(has_model(&author, "cube"), "{}", author.status);
    let dir = std::env::temp_dir().join(format!("camcraft-pick-{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("temp dir");
    let file = dir.join("job.camproj");
    project::save(&author.world, &file).expect("save");
    let bytes = std::fs::read(&file).expect("read back");
    let _ = std::fs::remove_dir_all(&dir);

    let mut state = State::new();
    let other = std::fs::read(testdata("box.obj")).expect("box.obj");
    file_pick::deliver(picked(PickFor::Model, "box.obj", Ok(other)));
    state.drain_actions();
    assert!(has_model(&state, "box"));
    file_pick::deliver(picked(PickFor::Project, "job.camproj", Ok(bytes)));
    state.drain_actions();
    assert_eq!(state.status, "opened job.camproj");
    assert!(has_model(&state, "cube"), "the project's model is there");
    assert!(!has_model(&state, "box"), "the old document is gone");
    assert!(!state.history.is_dirty(), "an opened project is the clean point");

    // Not text, and text that is no project: refused by name.
    file_pick::deliver(picked(PickFor::Project, "x.camproj", Ok(vec![0xff, 0xfe, 0x00])));
    state.drain_actions();
    assert_eq!(state.status, "open failed: x.camproj is not a CamCraft project (not text)");
    assert!(has_model(&state, "cube"), "a non-text file changes nothing");
    file_pick::deliver(picked(PickFor::Project, "y.camproj", Ok(b"{ not json".to_vec())));
    state.drain_actions();
    assert!(state.status.starts_with("open failed: y.camproj:"), "{}", state.status);
}

#[test]
fn the_browser_picker_accepts_the_dialogs_extensions() {
    let mesh = FileDialog::new().add_filter("Mesh", &["stl", "obj", "step", "stp"]);
    assert_eq!(file_pick::accept_attr(&mesh), ".stl,.obj,.step,.stp");
    let messy = FileDialog::new().add_filter("A", &[".STL", "obj"]).add_filter("B", &["stl", " "]);
    assert_eq!(file_pick::accept_attr(&messy), ".stl,.obj");
    assert_eq!(file_pick::accept_attr(&FileDialog::new()), "");
}

#[test]
fn a_file_over_the_browser_read_limit_is_refused_before_it_is_read() {
    let limit = (file_pick::BROWSER_READ_LIMIT) as f64;
    assert_eq!(file_pick::too_big("ok.stl", limit), None);
    let why = file_pick::too_big("huge.step", limit + 1.0).expect("refused");
    assert!(why.starts_with("huge.step is 256 MiB, over the browser build's 256 MiB read limit"), "{why}");
    assert!(file_pick::too_big("nan.stl", f64::NAN).is_some());
    // The inbox's Debug never prints the bytes themselves.
    let shown = format!("{:?}", picked(PickFor::Model, "m.stl", Ok(vec![7; 1000])));
    assert!(shown.contains("1000 bytes") && !shown.contains("7, 7"), "{shown}");
}
