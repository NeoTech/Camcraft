//! #4338 (U12 of #3663), its "Done means": the Holders page searches the
//! taper, the clamp, a bore or collet size and the name (each term must
//! match) and counts "N of M holders"; Esc clears it; a still frame
//! searches, reads and checks nothing. A user holder opens in the holder
//! form: each field lands on Apply as one undo step, the tools that use
//! the holder derive again in that step, and undo gives both back; an edit
//! that breaks a tool's fit is refused by name and changes nothing; a
//! built-in holder opens read-only with "Copy as user holder"; New holder
//! and Duplicate make user holders.
//! Gate map: covers ui::egui_app::tool_library_dialog, ui::state.
//! Provenance: our own structural tests over the AccessKit tree and the
//! command history; no HSMWorks material involved.

use camcraft_app::commands::{AddTool, SetToolAssembly, UpsertDocumentComponent};
use camcraft_app::component::{ADAPTIVE_PORT, ComponentKind, ToolComponent};
use camcraft_app::connection_code::ConnectionCode;
use camcraft_app::holder_library::{BT40, CAT40, ER32, HolderShape, ShapeClamp};
use camcraft_app::schema::AssemblyLink;
use camcraft_app::tool_assembly::{self as ta, Mount};
use camcraft_app::tools::{Tool, ToolKind};
use egui::accesskit::Role;

use super::tool_library_components::{A, B, INSERT, done, fly, insert, lib_dir, shell, shows, undo};
use super::tool_library_editor::{H, click_window_button, library_window};
use super::*;
use crate::egui_app::tool_library_dialog::{HOLDERS_PAGE, frame, holder_drawing, holder_form, holders};
use crate::state::{ComponentDraft, ErrorScope};

const ID: &str = "user:holder:Shrink 20";

/// #4455: a gauge length at which the outline shows the Ø44 body (at 80
/// mm the flange envelope covers it, and the form does not edit it).
fn shrink20() -> HolderShape {
    HolderShape { taper: BT40, clamp: ShapeClamp::Shrink, bore: 20.0, gauge_len: 120.0, nose_d: 32.0, largest_d: 44.0 }
}

fn cutter() -> Tool {
    let Some((_, base)) = camcraft_app::tools::catalog().into_iter().find(|(n, _)| *n == "EM-1200-2FL") else {
        panic!("the catalog has EM-1200-2FL");
    };
    Tool { kind: ToolKind::BullNose, diameter: 20.0, corner_r: 0.2, flute_len: 5.0, shank_d: 20.0, ..base }
}

fn on(parent: usize, port: Option<&str>) -> Option<Mount> {
    Some(Mount { parent, port: port.map(str::to_owned), z_rot_deg: 0.0, projection_mm: None })
}

/// The user holder `Shrink 20` (a shape the form edits), two cutter bodies
/// with a Ø20 shank, the insert, and the tools A and B stacked on them.
fn holder_shop(h: &mut H) {
    let st = &mut h.state_mut().state;
    let s = shrink20();
    let holder = ToolComponent::new(ID, "Shrink 20", s.kind().expect("codes")).expect("holder");
    let g = s.geometry(ADAPTIVE_PORT).expect("outline");
    for (c, g) in [(holder, g), fly("fly-25", 12.5), fly("fly-32", 16.0), insert("CCMT09T304")] {
        let cmd = UpsertDocumentComponent::new(&st.world, c, g).expect("upsert");
        st.history.execute(&st.world, cmd);
    }
    for (name, body) in [(A, "fly-25"), (B, "fly-32")] {
        let add = AddTool::new(&st.world, name, cutter()).expect("tool");
        st.history.execute(&st.world, add);
        let links = vec![
            AssemblyLink { component: ID.into(), mount: None },
            AssemblyLink { component: body.into(), mount: on(0, None) },
            AssemblyLink { component: INSERT.into(), mount: on(1, Some("CSW1")) },
        ];
        let set = SetToolAssembly::new(&st.world, name, links).expect("assembly");
        st.history.execute(&st.world, set);
    }
    st.doc_version += 1;
    h.run_steps(2);
}

fn open_holders(h: &mut H) {
    h.get_by_role_and_label(Role::Button, "Tools").click();
    h.run_steps(2);
    click_window_button(h, HOLDERS_PAGE);
}

fn set_filter(h: &mut H, text: &str) {
    if let Some(v) = h.state_mut().state.tool_library.as_mut() {
        v.holders.filter = text.to_owned();
    }
    h.run_steps(2);
}

fn draft(h: &mut H, f: impl FnOnce(&mut ComponentDraft)) {
    if let Some(d) = h.state_mut().state.tool_library.as_mut().and_then(|v| v.component_edit.as_mut()) {
        f(d);
    }
    h.run_steps(2);
}

fn row_shape(h: &H, id: &str) -> Option<(String, HolderShape)> {
    let r = ta::component(&h.state().state.world, id)?;
    Some((r.component.name().to_owned(), HolderShape::of(r.component.kind(), &r.geometry, ADAPTIVE_PORT)?))
}

/// The derived body of tool `name`, as text.
fn body(h: &H, name: &str) -> String {
    let w = &h.state().state.world;
    let e = camcraft_app::model::find_by_name(w, name).unwrap_or_else(|| panic!("tool {name}"));
    format!("{:?}", camcraft_app::tool_body::body_of(w, &e))
}

/// "N of M holders" for the library alone (M: every library holder).
fn count(n: usize) -> String {
    format!("{n} of {} holders", holders::rows().len())
}

#[test]
fn the_holder_search_matches_each_term_and_counts() {
    let mut h = shell_harness();
    h.run_steps(2);
    open_holders(&mut h);
    assert!(shows_label(&h, holders::SEARCH), "the search box heads the list");
    assert!(shows_label(&h, holders::NEW) && shows_label(&h, holders::IMPORT) && shows_label(&h, holders::EXPORT));
    let all = count(holders::rows().len());
    assert!(shows_label(&h, &all), "the list counts its holders: {all}");
    set_filter(&mut h, "bt40 er32");
    assert!(shows_label(&h, "BT40 ER32"), "taper and clamp match together");
    assert!(!shows_label(&h, "BT30 ER32"), "each term must match");
    assert!(shows_label(&h, &count(1)), "one holder of all");
    set_filter(&mut h, "shrink 12");
    assert!(shows_label(&h, "BT40 Shrink 12 mm"), "a clamp word and a bore size match");
    assert!(!shows_label(&h, "BT40 ER32"), "an ER chuck is not a shrink-fit chuck");
    set_filter(&mut h, "BT40 er 16");
    assert!(shows_label(&h, "BT40 ER32"), "a number matches a chuck that clamps it");
    assert!(!shows_label(&h, "BT40 ER20"), "an ER20 clamps 13 mm at most");
    // Esc in the box clears it.
    library_window(&h).get_by_label(holders::SEARCH).click();
    h.run_steps(1);
    h.key_press(egui::Key::Escape);
    h.run_steps(2);
    assert_eq!(h.state().state.tool_library.as_ref().map(|v| v.holders.filter.clone()), Some(String::new()), "Esc clears the search");
    assert!(shows_label(&h, &all), "the list shows all holders again");
}

fn shows_label(h: &H, label: &str) -> bool {
    library_window(h).query_all_by_label(label).next().is_some()
}

#[test]
fn a_still_frame_searches_reads_and_checks_nothing() {
    let dir = lib_dir("4338-still");
    let mut h = shell(&dir);
    holder_shop(&mut h);
    open_holders(&mut h);
    set_filter(&mut h, "shrink");
    let s = crate::state::holder_searches();
    h.run_steps(4);
    assert_eq!(crate::state::holder_searches(), s, "a still frame searches nothing");
    set_filter(&mut h, "shrink 20");
    assert!(crate::state::holder_searches() > s, "a new text searches again");
    click_window_button(&mut h, "Shrink 20");
    assert!(shows(&h, holder_form::NOSE), "the user holder opens in the form");
    let (r, c, b) = (crate::state::shape_reads(), holder_form::checks(), holder_drawing::builds());
    h.run_steps(4);
    assert_eq!(crate::state::shape_reads(), r, "a still frame reads no shape");
    assert_eq!(holder_form::checks(), c, "a still frame checks nothing");
    assert_eq!(holder_drawing::builds(), b, "a still frame builds no drawing");
    draft(&mut h, |d| {
        if let Some(s) = d.shape.as_mut() {
            s.gauge_len = 90.0;
        }
    });
    assert!(holder_form::checks() > c && holder_drawing::builds() > b, "a new value checks and draws again");
}

/// Each field of the form, on its own: lands on Apply as one undo step,
/// both tools derive again in that step, and undo gives both back.
#[test]
fn each_holder_field_lands_in_one_step_with_its_tools_and_undoes() {
    let dir = lib_dir("4338-fields");
    let mut h = shell(&dir);
    holder_shop(&mut h);
    open_holders(&mut h);
    set_filter(&mut h, "shrink 20");
    click_window_button(&mut h, "Shrink 20");
    for l in [holder_form::NAME, holder_form::TAPER, holder_form::CLAMP, holder_form::FROM, holder_form::TO, holder_form::GAUGE, holder_form::BODY, holder_form::LARGEST, holder_form::NOSE] {
        assert!(shows(&h, l), "the form has the field {l}");
    }
    assert!(shows(&h, A) && shows(&h, B), "the form lists the tools that use the holder");
    type Edit = fn(&mut String, &mut HolderShape);
    let edits: [(&str, Edit, bool); 5] = [
        ("name", |n, _| *n = "Shrink 20 long".into(), false),
        ("taper", |_, s| s.taper = CAT40, true),
        ("gauge length", |_, s| s.gauge_len = 130.0, true),
        ("body diameter", |_, s| s.largest_d = 36.0, true),
        ("nose diameter", |_, s| s.nose_d = 36.0, true),
    ];
    for (what, edit, tools_change) in edits {
        let before = row_shape(&h, ID).unwrap_or_else(|| panic!("the holder reads back"));
        let (a0, b0, n0) = (body(&h, A), body(&h, B), done(&h));
        draft(&mut h, |d| {
            if let Some(s) = d.shape.as_mut() {
                edit(&mut d.name, s);
            }
        });
        let why = holder_form::refusal(&mut h.state_mut().state);
        click_window_button(&mut h, frame::APPLY);
        let err = h.state().state.error_for(ErrorScope::Tool).map(str::to_owned);
        assert_eq!(done(&h), n0 + 1, "{what}: one undo step (refusal {why:?}, error {err:?})");
        let after = row_shape(&h, ID).unwrap_or_else(|| panic!("{what}: the holder still reads back"));
        assert_ne!(after, before, "{what}: the edit lands");
        if tools_change {
            assert!(body(&h, A) != a0 && body(&h, B) != b0, "{what}: both tools derive again in the same step");
        }
        undo(&mut h);
        assert_eq!(row_shape(&h, ID), Some(before), "{what}: undo gives the holder back");
        assert!(body(&h, A) == a0 && body(&h, B) == b0, "{what}: undo gives both tools back");
        // The form follows the document again: the next edit starts there.
        h.state_mut().state.tool_library.as_mut().and_then(|v| v.component_edit.take());
        crate::egui_app::tool_library_dialog::component_edit::open(&mut h.state_mut().state, ID);
        h.run_steps(2);
    }
}

/// The clamp and the shank range of a holder no tool uses: an ER32
/// collet, then a Weldon bore, each one undo step.
#[test]
fn the_clamp_and_the_shank_range_land_and_undo() {
    let dir = lib_dir("4338-clamp");
    let mut h = shell(&dir);
    open_holders(&mut h);
    let n0 = done(&h);
    click_window_button(&mut h, holders::NEW);
    assert_eq!(done(&h), n0 + 1, "New holder is one undo step");
    let id = "user:holder:BT40 ER32";
    assert!(shows(&h, holder_form::GAUGE), "the new holder opens in the form");
    let to_weldon = |_: &mut String, s: &mut HolderShape| {
        s.clamp = ShapeClamp::Weldon;
        s.bore = 16.0;
        s.nose_d = 40.0;
        s.largest_d = 40.0;
    };
    draft(&mut h, |d| {
        if let Some(s) = d.shape.as_mut() {
            to_weldon(&mut d.name, s);
        }
    });
    click_window_button(&mut h, frame::APPLY);
    assert_eq!(done(&h), n0 + 2, "the clamp edit is one undo step");
    let kind = ta::component(&h.state().state.world, id).map(|r| r.component.kind().clone());
    let weldon = |d: f64| ComponentKind::AdaptiveItem { mcs: ConnectionCode::parse("BT40").expect("BT40"), csw: ConnectionCode::Weldon { d } };
    assert_eq!(kind, Some(weldon(16.0)), "the workpiece side is the Weldon bore");
    draft(&mut h, |d| {
        if let Some(s) = d.shape.as_mut() {
            s.bore = 12.0;
        }
    });
    click_window_button(&mut h, frame::APPLY);
    let kind = ta::component(&h.state().state.world, id).map(|r| r.component.kind().clone());
    assert_eq!(kind, Some(weldon(12.0)), "Clamps from / to sets the bore");
    undo(&mut h);
    undo(&mut h);
    let back = row_shape(&h, id).map(|(_, s)| s.clamp);
    assert_eq!(back, Some(ShapeClamp::Er(ER32)), "undo gives the ER32 collet back");
}

/// A bore that the cutter bodies' Ø20 shank does not fit is refused by
/// name: the holder and both tools stay as they are.
#[test]
fn an_edit_that_breaks_a_tools_fit_is_refused_by_name() {
    let dir = lib_dir("4338-fit");
    let mut h = shell(&dir);
    holder_shop(&mut h);
    open_holders(&mut h);
    set_filter(&mut h, "shrink 20");
    click_window_button(&mut h, "Shrink 20");
    let (before, a0, n0) = (row_shape(&h, ID), body(&h, A), done(&h));
    draft(&mut h, |d| {
        if let Some(s) = d.shape.as_mut() {
            s.bore = 16.0;
        }
    });
    click_window_button(&mut h, frame::APPLY);
    let msg = h.state().state.error_for(ErrorScope::Tool).map(str::to_owned).unwrap_or_default();
    assert!(msg.contains("refused") && msg.contains(ID), "the refusal names the holder: {msg}");
    assert!(msg.contains(A) || msg.contains("fly-25"), "the refusal names what no longer fits: {msg}");
    assert!(shows(&h, &msg), "the form shows the refusal");
    assert_eq!(done(&h), n0, "a refused edit is no undo step");
    assert_eq!(row_shape(&h, ID), before, "the holder stays");
    assert_eq!(body(&h, A), a0, "the tool stays");
    // A value that cannot be steel is refused before Apply.
    draft(&mut h, |d| {
        if let Some(s) = d.shape.as_mut() {
            s.largest_d = 10.0;
        }
    });
    assert!(holder_form::refusal(&mut h.state_mut().state).is_some_and(|w| w.contains("widens from its nose")));
}

/// A built-in holder opens read-only (no inputs, no Apply); "Copy as user
/// holder" makes the editable copy; Duplicate copies a user holder.
#[test]
fn a_built_in_holder_is_read_only_and_copies_and_duplicates() {
    let dir = lib_dir("4338-builtin");
    let mut h = shell(&dir);
    open_holders(&mut h);
    h.state_mut().state.open_holder_in_library(Some("BT40 ER32"));
    h.run_steps(2);
    assert!(shows(&h, holders::READ_ONLY), "the view says it is read-only");
    assert!(library_window(&h).query_all_by_role(Role::SpinButton).next().is_none(), "no value input");
    assert!(library_window(&h).query_all_by_role_and_label(Role::Button, frame::APPLY).next().is_none(), "no Apply");
    let n0 = done(&h);
    click_window_button(&mut h, holders::COPY);
    assert_eq!(done(&h), n0 + 1);
    assert!(shows(&h, holder_form::GAUGE), "the copy opens in the form");
    click_window_button(&mut h, frame::DUPLICATE);
    assert_eq!(done(&h), n0 + 2, "Duplicate is one undo step");
    assert!(ta::component(&h.state().state.world, "user:holder:BT40 ER32 (2)").is_some(), "the duplicate has its own id");
}
