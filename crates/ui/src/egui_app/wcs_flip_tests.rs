//! #1720's Flip X / Flip Y / Flip Z buttons on the WCS page: three equal
//! toggles under the quarter turns, each a live button whose lit state is
//! the stored flag; a click lands one undoable `SetWcsFlip`, the viewport
//! triad draws the flipped axis reversed, and undo clears it again.
//! The status line names a flip that the post refuses (#1649).
//! Structural assertions only (no snapshot PNG changes).
//! Provenance: our own page law; no vendor material consulted.

use egui::accesskit::Toggled;
use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT, Queryable};

use super::wcs_page::FLIPS;
use super::wcs_page_tests::page;
use crate::interact::gizmos;
use crate::state::{CommandKind, State};

fn lit(h: &Harness<'static, State>, label: &str) -> bool {
    h.get_by_label(label).accesskit_node().toggled() == Some(Toggled::True)
}

fn flip(state: &State) -> [bool; 3] {
    camcraft_app::setup_wcs_flip::of_setup(&state.world, None)
}

fn run(state: &mut State, command: CommandKind) {
    state.pending.push_back(command);
    state.drain_actions();
}

// Optional real page renders for PR evidence, never snapshot goldens.
fn evidence(h: &mut Harness<'static, State>, name: &str) {
    if let Some(dir) = std::env::var_os("CAMCRAFT_WCS_EVIDENCE_DIR") {
        let _gpu = crate::headless_gpu::gpu_guard();
        let path = std::path::PathBuf::from(dir).join(format!("{name}.png"));
        h.render().expect("render WCS page").save(path).expect("save WCS evidence");
    }
}

#[test]
fn a_flipped_non_z_rotation_paints_the_actual_post_refusal() {
    let mut state = State::new();
    run(&mut state, CommandKind::SetWcs { pos: [0.0; 3], roll_deg: 15.0, pitch_deg: 0.0, yaw_deg: 0.0 });
    run(&mut state, CommandKind::SetWcsFlip { flip: [true, false, false] });
    let why = camcraft_app::setup_wcs_flip::setup_problem(&state.world, None).expect("rotated flip refusal");
    assert!(why.contains("only a finite Z rotation is supported"), "{why}");
    let mut h = page(state);
    evidence(&mut h, "rotated-flip");
    assert!(h.query_by_label_contains(&why).is_some(), "the page must paint the actual refusal: {why}");
    assert!(h.query_by_label_contains("The posted program is mirrored").is_none());
}

#[test]
fn a_probe_paints_the_refusal_and_operation_edits_and_undo_refresh_it() {
    use camcraft_app::operation::{Height, OpParams, SrcRef};
    use camcraft_app::probing::{ProbeDir, ProbeMode};
    let mut state = State::new();
    run(&mut state, CommandKind::SetStock {
        shape: camcraft_app::stock::StockShape::Box { dx: 40.0, dy: 30.0, dz: 10.0 },
        pos: [0.0; 3], yaw_deg: 0.0,
    });
    run(&mut state, CommandKind::AddTool { name: "PROBE-4".into(), tool: camcraft_app::tools::probe_tool(4.0, 30.0) });
    run(&mut state, CommandKind::CreateOperation {
        name: "Probe1".into(), tool: "PROBE-4".into(), src: SrcRef::Stock,
        params: OpParams::Probe {
            clearance_z: Height::fixed(35.0), mode: ProbeMode::Face, dir: ProbeDir::ZNeg, dir_y: ProbeDir::YPos,
            target: [0.0, 0.0, 10.0], radius: 0.0, approach: 5.0, overtravel: 1.0, backoff: 1.0,
            fast_feed: 300.0, slow_feed: 30.0, depth: 3.0, inset: 5.0, set_wcs: Some(0), tolerance: 0.5,
        },
    });
    assert!(camcraft_app::model::find_by_name(&state.world, "Probe1").is_some(), "{}", state.status);
    run(&mut state, CommandKind::SetWcsFlip { flip: [true, false, false] });
    let why = camcraft_app::setup_wcs_flip::setup_post_problems(&state.world, None).join("; ");
    assert!(why.contains("probing does not post in a flipped WCS"), "{why}");
    let mut h = page(state);
    evidence(&mut h, "probe-flip");
    assert!(h.query_by_label_contains(&why).is_some());
    assert!(h.query_by_label_contains("The posted program is mirrored").is_none());
    run(h.state_mut(), CommandKind::SuppressOperation { name: "Probe1".into(), suppressed: true });
    h.run();
    assert!(h.query_by_label_contains("The posted program is mirrored").is_some());
    assert!(h.query_by_label_contains("The post refuses this flip").is_none());
    run(h.state_mut(), CommandKind::Undo);
    h.run();
    assert!(h.query_by_label_contains(&why).is_some());
    run(h.state_mut(), CommandKind::Redo);
    h.run();
    assert!(h.query_by_label_contains("The posted program is mirrored").is_some());
}

#[test]
fn document_rotation_and_post_changes_refresh_the_painted_note() {
    let mut state = State::new();
    run(&mut state, CommandKind::SetWcsFlip { flip: [true, false, false] });
    let mut h = page(state);
    let count = h.state().wcs_ui.flip_note.refreshes;
    h.run_steps(6);
    assert_eq!(h.state().wcs_ui.flip_note.refreshes, count, "still frames reuse the click's cache");
    run(h.state_mut(), CommandKind::SetWcs { pos: [0.0; 3], roll_deg: 15.0, pitch_deg: 0.0, yaw_deg: 0.0 });
    h.run();
    assert!(h.query_by_label_contains("only a finite Z rotation is supported").is_some());
    assert_eq!(h.state().wcs_ui.flip_note.refreshes, count + 1);
    run(h.state_mut(), CommandKind::Undo);
    h.run();
    assert!(h.query_by_label_contains("The posted program is mirrored").is_some());
    run(h.state_mut(), CommandKind::SelectPostNamed("Fanuc Lathe".into()));
    h.run();
    assert!(h.query_by_label_contains("is not verified for a mirrored program").is_some());
    assert!(h.query_by_label_contains("The posted program is mirrored").is_none());
    run(h.state_mut(), CommandKind::Undo);
    h.run();
    assert!(h.query_by_label_contains("The posted program is mirrored").is_some());
    run(h.state_mut(), CommandKind::Redo);
    h.run();
    assert!(h.query_by_label_contains("is not verified for a mirrored program").is_some());
}

#[test]
fn the_active_setup_is_part_of_the_cache_key_even_at_the_same_revision() {
    let mut state = State::new();
    run(&mut state, CommandKind::CreateSetup { name: None, datum_from: None });
    run(&mut state, CommandKind::CreateSetup { name: None, datum_from: None });
    run(&mut state, CommandKind::SetWcsFlip { flip: [false, false, true] });
    let mut h = page(state);
    assert!(h.query_by_label_contains("a flipped Z is not carried").is_some());
    let count = h.state().wcs_ui.flip_note.refreshes;
    // Isolate identity from the revision: the app command changes the cursor
    // without the UI dispatcher's additional document bump.
    let command = camcraft_app::commands::ActivateSetup::new(&h.state().world, Some("Setup1")).expect("Setup1");
    let state = h.state_mut();
    state.history.execute(&state.world, command);
    h.run();
    assert_eq!(h.state().wcs_ui.flip_note.refreshes, count + 1);
    assert!(h.query_by_label_contains("The post refuses this flip").is_none());
    assert!(h.query_by_label_contains("The posted program is mirrored").is_none());
    run(h.state_mut(), CommandKind::Undo);
    h.run();
    assert!(h.query_by_label_contains("a flipped Z is not carried").is_some());
}

#[test]
fn project_replacement_and_new_clear_the_old_painted_refusal() {
    let mut state = State::new();
    run(&mut state, CommandKind::SetWcsFlip { flip: [false, false, true] });
    let text = camcraft_app::project::save_text(&state.world).expect("flipped project");
    let path = std::env::temp_dir().join(format!("camcraft-2497-{}.camproj", std::process::id()));
    std::fs::write(&path, text).expect("test project");
    let mut h = page(state);
    assert!(h.query_by_label_contains("a flipped Z is not carried").is_some());
    h.state_mut().history.mark_clean();
    run(h.state_mut(), CommandKind::NewDocument);
    h.run();
    assert!(h.query_by_label_contains("The post refuses this flip").is_none());
    run(h.state_mut(), CommandKind::OpenProjectFile(path.clone()));
    h.run();
    assert!(h.query_by_label_contains("a flipped Z is not carried").is_some());
    std::fs::remove_file(path).expect("remove test project");
}

#[test]
fn unchanged_frames_do_not_reload_a_post_script() {
    let path = std::env::temp_dir().join(format!("camcraft-2497-{}.cps", std::process::id()));
    std::fs::write(&path, "axisInversion = true;").expect("verified test manifest");
    let mut state = State::new();
    run(&mut state, CommandKind::SelectPostFile(path.clone()));
    assert!(state.status.starts_with("post:"), "{}", state.status);
    run(&mut state, CommandKind::SetWcsFlip { flip: [true, false, false] });
    let mut h = page(state);
    assert!(h.query_by_label_contains("The posted program is mirrored").is_some());
    let count = h.state().wcs_ui.flip_note.refreshes;
    // A disk edit is deliberately not a document change. If any still
    // frame reloads the script, it would paint the unverified-post refusal.
    std::fs::write(&path, "axisInversion = false;").expect("unverified test manifest");
    assert!(camcraft_app::setup_wcs_flip::setup_problem(&h.state().world, None).is_some(), "the actual reader sees the disk edit");
    h.run_steps(6);
    assert_eq!(h.state().wcs_ui.flip_note.refreshes, count);
    assert!(h.query_by_label_contains("The posted program is mirrored").is_some());
    // A real document edit refreshes the existing reader. Selecting the
    // identical post is a no-op, so use a placed-origin edit instead.
    run(h.state_mut(), CommandKind::SetWcs { pos: [1.0, 0.0, 0.0], roll_deg: 0.0, pitch_deg: 0.0, yaw_deg: 0.0 });
    h.run();
    assert!(h.query_by_label_contains("is not verified for a mirrored program").is_some());
    assert_eq!(h.state().wcs_ui.flip_note.refreshes, count + 1);
    std::fs::remove_file(path).expect("remove test script");
}

#[test]
fn the_flip_buttons_sit_under_the_turns_as_equal_live_toggles() {
    let h = page(State::new());
    let turn = h.get_by_label("Turn X-").rect();
    let level = h.get_by_label("Level").rect();
    let r: Vec<egui::Rect> = FLIPS.iter().map(|l| h.get_by_label(l).rect()).collect();
    for (i, l) in FLIPS.iter().enumerate() {
        assert!(!h.get_by_label(l).accesskit_node().is_disabled(), "{l} is live");
        assert!(!lit(&h, l), "{l} starts off");
        assert!((r[i].width() - turn.width()).abs() < 0.5, "{l} is a turn's width");
        assert!(r[i].min.y >= turn.max.y && r[i].max.y <= level.min.y, "{l} sits between the turns and Level");
    }
    assert!(h.query_by_label_contains("The posted program is mirrored").is_none(), "no note with no flip");
    assert!(h.query_by_label_contains("A flipped Z does not post yet").is_none(), "no note with no flip");
}

#[test]
fn a_flip_click_lands_one_undoable_command_and_reverses_the_triad() {
    let mut h = page(State::new());
    h.get_by_label("Flip Y").click();
    h.run();
    h.run();
    assert_eq!(flip(h.state()), [false, true, false], "{}", h.state().status);
    assert!(lit(&h, "Flip Y") && !lit(&h, "Flip X") && !lit(&h, "Flip Z"));
    assert!(h.query_by_label_contains("The posted program is mirrored").is_some(), "the page says what the post does");
    assert!(h.state().status.contains("the posted program is mirrored"), "{}", h.state().status);
    let triad = gizmos(h.state()).into_iter().find(|g| !g.world_axes).expect("the G54 triad");
    assert_eq!(triad.flip, [false, true, false]);
    assert_eq!(triad.axis_dir(1, &[0.0, 1.0, 0.0]), [0.0, -1.0, 0.0], "the Y axis points the other way");
    assert_eq!(triad.axis_dir(0, &[1.0, 0.0, 0.0]), [1.0, 0.0, 0.0], "X is not touched");
    assert!(
        gizmos(h.state()).iter().any(|g| g.world_axes) || !h.state().show_world_axes,
        "a flipped WCS at the origin does not hide the world axes"
    );

    h.state_mut().pending.push_back(CommandKind::Undo);
    h.run();
    h.run();
    assert_eq!(flip(h.state()), [false; 3], "undo clears the flip");
    assert!(!lit(&h, "Flip Y"));
    h.state_mut().pending.push_back(CommandKind::Redo);
    h.run();
    h.run();
    assert_eq!(flip(h.state()), [false, true, false], "redo sets it again");
}

#[test]
fn two_clicks_on_one_axis_clear_it_and_the_others_stack() {
    let mut state = State::new();
    for f in [[true, false, false], [true, false, true], [false, false, true]] {
        state.pending.push_back(CommandKind::SetWcsFlip { flip: f });
        state.drain_actions();
        assert_eq!(flip(&state), f, "{}", state.status);
    }
    state.pending.push_back(CommandKind::SetWcsFlip { flip: [false; 3] });
    state.drain_actions();
    assert_eq!(flip(&state), [false; 3]);
    assert!(state.status.contains("no axis flipped"), "{}", state.status);
}

/// #1649 review: the status line says "mirrored" only when the post carries
/// the flip; a flip that the post refuses names the refusal instead.
#[test]
fn the_status_line_names_a_flip_that_the_post_refuses() {
    let mut state = State::new();
    state.pending.push_back(CommandKind::SetWcsFlip { flip: [false, false, true] });
    state.drain_actions();
    assert!(state.status.contains("the post refuses it"), "{}", state.status);
    assert!(state.status.contains("a flipped Z is not carried"), "{}", state.status);
    assert!(!state.status.contains("mirrored"), "{}", state.status);
}
