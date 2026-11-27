//! #3898 (U7 of #3663), the "Done means" of #3648: the "My library"
//! source lists the built-in components and the user's own file, and its
//! compatible filter shows the cutter body that takes a picked insert; a tool is
//! assembled from linked document components in the window, in one undo
//! step; an edit of the shared insert updates both tools built from it in
//! one undo step; a newer user library copy is synced into the document
//! and both tools follow; a built-in holder is copied as a user holder
//! and its gauge length is edited; undo and redo restore each step. A
//! milling insert stands in for the turning insert of #3648: the lathe
//! side of the resolver is still open on #3643. The library file writes
//! (save, delete) have no undo. A still frame reads no library file and
//! builds no derived set.
//! Gate map: covers ui::egui_app::tool_library_dialog, ui::state.
//! Provenance: our own structural tests over the AccessKit tree and the
//! command history; no HSMWorks material involved.

use std::path::PathBuf;

use camcraft_app::commands::{AddTool, SetToolAssembly, UpsertDocumentComponent};
use camcraft_app::component::{ComponentKind, CuttingItem, InsertSeat, ToolComponent};
use camcraft_app::component_fit::compatible_parents;
use camcraft_app::component_geometry::{ComponentBody, ComponentGeometry};
use camcraft_app::component_library::{self, LibraryComponent, store};
use camcraft_app::connection_code::ConnectionCode;
use camcraft_app::frame::Frame;
use camcraft_app::insert_iso1832::InsertSpec;
use camcraft_app::insert_seat::HolderDesignation;
use camcraft_app::schema::AssemblyLink;
use camcraft_app::spun::SpunPoint;
use camcraft_app::tool_assembly::{self as ta, Mount};
use camcraft_app::tools::{Tool, ToolKind};
use egui::accesskit::Role;
use egui_kittest::kittest::NodeT;

use super::tool_library_editor::{H, click_window_button, library_window, open_in_window, tool};
use super::*;
use crate::egui_app::tool_library_dialog::{HOLDERS_PAGE, component_edit, components, holders, stack};
use crate::state::{ComponentAction, CommandKind, ToolSource};

pub(super) const A: &str = "Fly25";
pub(super) const B: &str = "Fly32";
pub(super) const INSERT: &str = "insert-cc";

/// A fresh, empty user library folder for one test.
pub(super) fn lib_dir(test: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("camcraft-3898-{test}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    if let Err(e) = std::fs::create_dir_all(&dir) {
        panic!("temp folder {}: {e}", dir.display());
    }
    dir
}

fn lib_file(dir: &std::path::Path) -> PathBuf {
    store::library_path(dir)
}

/// A shell whose window reads the user library in `dir`.
pub(super) fn shell(dir: &std::path::Path) -> H {
    let mut h = Harness::builder().with_size(egui::vec2(1600.0, 1400.0)).build_ui_state(super::frame, ShellUi::seeded());
    h.state_mut().state.component_lib.dir = Some(dir.to_path_buf());
    h.run_steps(2);
    h
}

fn code(text: &str) -> ConnectionCode {
    ConnectionCode::parse(text).unwrap_or_else(|e| panic!("connection code {text}: {e}"))
}

fn holder() -> (ToolComponent, ComponentGeometry) {
    let p = SpunPoint::line;
    let body = ComponentBody::new(vec![p(0.0, 0.0), p(0.0, 25.0), p(50.0, 25.0), p(50.0, 0.0)], -50.0).expect("body");
    let kind = ComponentKind::AdaptiveItem { mcs: code("BT40"), csw: code("CYL 20") };
    let c = ToolComponent::new("bt40-cyl20", "BT40 CYL 20", kind).expect("holder");
    (c, ComponentGeometry::axial("CSW", 50.0, Some(body)).expect("geometry"))
}

/// A single-seat cutter body with a CC seat `r` off the axis.
pub(super) fn fly(id: &str, r: f64) -> (ToolComponent, ComponentGeometry) {
    let seat = HolderDesignation::parse("SCLCR2020K09").expect("holder code").seat;
    let kind = ComponentKind::ToolItem { mcs: code("CYL 20"), seats: vec![InsertSeat { port: "CSW1".into(), spec: seat }] };
    let frame = Frame::new([r, 0.0, -60.0], [0.0, 0.0, 1.0], [1.0, 0.0, 0.0]).expect("seat frame");
    let g = ComponentGeometry::new(vec![("CSW1".into(), frame)], None, None).expect("geometry");
    (ToolComponent::new(id, id, kind).expect("cutter body"), g)
}

pub(super) fn insert(text: &str) -> (ToolComponent, ComponentGeometry) {
    let spec = InsertSpec::parse(text).expect("insert code");
    let c = ToolComponent::new(INSERT, text, ComponentKind::CuttingItem(CuttingItem::Insert(spec))).expect("insert");
    (c, ComponentGeometry::default())
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

fn stack_of(body: &str) -> Vec<AssemblyLink> {
    vec![
        AssemblyLink { component: "bt40-cyl20".into(), mount: None },
        AssemblyLink { component: body.into(), mount: on(0, None) },
        AssemblyLink { component: INSERT.into(), mount: on(1, Some("CSW1")) },
    ]
}

/// The fixture of the #3644 tests, landed through the commands: the four
/// components, the two tools, and (`stacks`) their stacks.
pub(super) fn shop(h: &mut H, stacks: bool) {
    let st = &mut h.state_mut().state;
    for (c, g) in [holder(), fly("fly-25", 12.5), fly("fly-32", 16.0), insert("CCMT09T304")] {
        let cmd = UpsertDocumentComponent::new(&st.world, c, g).expect("upsert");
        st.history.execute(&st.world, cmd);
    }
    for (name, body) in [(A, "fly-25"), (B, "fly-32")] {
        let add = AddTool::new(&st.world, name, cutter()).expect("tool");
        st.history.execute(&st.world, add);
        if stacks {
            let set = SetToolAssembly::new(&st.world, name, stack_of(body)).expect("assembly");
            st.history.execute(&st.world, set);
        }
    }
    st.doc_version += 1;
    h.run_steps(2);
}

pub(super) fn shows(h: &H, text: &str) -> bool {
    library_window(h).query_all_by_label(text).next().is_some()
}

pub(super) fn done(h: &H) -> usize {
    h.state().state.history.done_len()
}

pub(super) fn undo(h: &mut H) {
    crate::actions::undo(&mut h.state_mut().state);
    h.run_steps(2);
}

pub(super) fn redo(h: &mut H) {
    crate::actions::redo(&mut h.state_mut().state);
    h.run_steps(2);
}

pub(super) fn open_library(h: &mut H) {
    h.get_by_role_and_label(Role::Button, "Tools").click();
    h.run_steps(2);
    click_window_button(h, components::SOURCE);
}

pub(super) fn pick(h: &mut H, id: &str) {
    if let Some(v) = h.state_mut().state.tool_library.as_mut() {
        v.components.selected = Some(id.to_owned());
    }
    h.run_steps(2);
}

fn corner(h: &H, name: &str) -> f64 {
    tool(h, name).corner_r
}

fn user_holder(id: &str, name: &str) -> LibraryComponent {
    let (c, geometry) = holder();
    let component = ToolComponent::new(id, name, c.kind().clone()).expect("user holder");
    LibraryComponent { component, geometry }
}

/// "My library" lists the built-in holders and the user's file; deleting
/// a user component from the library writes the file and adds no undo
/// step; a built-in one cannot be deleted.
#[test]
fn my_library_lists_built_ins_and_the_user_file() {
    let dir = lib_dir("list");
    if let Err(e) = store::save(&lib_file(&dir), &[user_holder("user:holder:Mine", "My BT40 holder")]) {
        panic!("library file: {e}");
    }
    let mut h = shell(&dir);
    open_library(&mut h);
    assert_eq!(h.state().state.tool_library.as_ref().map(|v| v.source), Some(ToolSource::Library));
    assert!(shows(&h, "My BT40 holder"), "the user's component is listed");
    assert!(shows(&h, "BT40 ER32"), "the built-in holders are listed");
    pick(&mut h, "builtin:holder:BT40 ER32");
    let delete = library_window(&h).get_by_role_and_label(Role::Button, components::DELETE_LIB).accesskit_node().is_disabled();
    assert!(delete, "a built-in component cannot be deleted");
    pick(&mut h, "user:holder:Mine");
    let before = done(&h);
    click_window_button(&mut h, components::DELETE_LIB);
    let left = store::load(&lib_file(&dir)).unwrap_or_else(|e| panic!("library file: {e}"));
    assert!(left.is_empty(), "the file no longer holds the component: {left:?}");
    assert!(!shows(&h, "My BT40 holder"), "the list reads the file again");
    assert_eq!(done(&h), before, "a library file write is not a document undo step");
}

/// The compatible filter of a picked insert shows the cutter body whose
/// seat takes it, and hides the parts that do not take it (a spindle
/// holder, a built-in holder).
#[test]
fn the_compatible_filter_shows_the_bodies_that_take_an_insert() {
    let dir = lib_dir("compatible");
    let own: Vec<LibraryComponent> = [holder(), fly("fly-25", 12.5), insert("CCMT09T304")]
        .into_iter()
        .map(|(component, geometry)| LibraryComponent { component, geometry })
        .collect();
    if let Err(e) = store::save(&lib_file(&dir), &own) {
        panic!("library file: {e}");
    }
    let mut h = shell(&dir);
    open_library(&mut h);
    let lib = component_library::with_builtins(own.clone());
    let fits: Vec<&str> = compatible_parents(&own[2].component, lib.iter().map(|c| &c.component)).into_iter().map(|f| f.component.name()).collect();
    assert_eq!(fits, vec!["fly-25"], "only the cutter body takes the insert");
    pick(&mut h, INSERT);
    click_window_button(&mut h, components::SHOW_COMPATIBLE);
    assert!(shows(&h, "Compatible with CCMT09T304"), "the filter names its component");
    assert!(shows(&h, "fly-25"), "the cutter body takes the insert and is shown");
    assert!(!shows(&h, "BT40 CYL 20"), "a spindle holder does not take an insert and is hidden");
    assert!(!shows(&h, "BT40 ER32"), "a built-in holder does not take an insert and is hidden");
    click_window_button(&mut h, components::SHOW_ALL);
    assert!(shows(&h, "BT40 CYL 20") && shows(&h, "BT40 ER32"), "Show all parts lists every part again");
}

/// A tool is assembled from the document's linked components in the
/// window: "Add component" offers only the parts that fit, Apply stack is
/// one undo step that lands the insert's corner on the tool, and undo and
/// redo restore it.
#[test]
fn a_tool_is_assembled_from_linked_components_in_one_undo_step() {
    let dir = lib_dir("assemble");
    let mut h = shell(&dir);
    shop(&mut h, false);
    open_in_window(&mut h, A);
    let table = ta::components(&h.state().state.world);
    let mut links: Vec<AssemblyLink> = Vec::new();
    for want in ["bt40-cyl20", "fly-25", INSERT] {
        let offer = stack::candidates(&links, &table);
        let Some((_, link)) = offer.iter().find(|(_, l)| l.component == want) else {
            panic!("{want} is offered: {offer:?}");
        };
        links.push(link.clone());
    }
    let offer = stack::candidates(&links[..2], &table);
    assert!(offer.iter().all(|(_, l)| l.component == INSERT), "the cutter body takes only the insert: {offer:?}");
    let name = A.to_owned();
    if let Some(v) = h.state_mut().state.tool_library.as_mut() {
        v.stack = Some(crate::state::StackDraft { tool: name, synced: 0, links: links.clone() });
    }
    // The draft is read at the shown document version.
    let doc = h.state().state.doc_version;
    if let Some(s) = h.state_mut().state.tool_library.as_mut().and_then(|v| v.stack.as_mut()) {
        s.synced = doc;
    }
    h.run_steps(2);
    let before = done(&h);
    click_window_button(&mut h, stack::APPLY);
    let assembly = |h: &H| {
        let st = &h.state().state;
        model::find_by_name(&st.world, A).and_then(|e| ta::assembly_of(&st.world, &e)).map(|a| a.0)
    };
    assert_eq!(assembly(&h).as_ref(), Some(&links), "the stack is stored");
    assert_eq!(done(&h), before + 1, "Apply stack is one undo step");
    assert!((corner(&h, A) - 0.4).abs() < 1e-9, "the insert's corner lands on the tool");
    undo(&mut h);
    assert_eq!(assembly(&h), None, "undo takes the stack away");
    assert!((corner(&h, A) - 0.2).abs() < 1e-9, "undo gives the old record back");
    redo(&mut h);
    assert_eq!(assembly(&h).as_ref(), Some(&links), "redo lands it again");
}

/// An edit of the shared insert in the component editor updates both
/// tools built from it in one undo step; undo restores both.
#[test]
fn an_edit_of_the_shared_insert_updates_both_tools_in_one_undo_step() {
    let dir = lib_dir("shared");
    let mut h = shell(&dir);
    shop(&mut h, true);
    open_library(&mut h);
    assert!((corner(&h, A) - 0.4).abs() < 1e-9 && (corner(&h, B) - 0.4).abs() < 1e-9, "both start at r0.4");
    component_edit::open(&mut h.state_mut().state, INSERT);
    h.run_steps(2);
    assert!(shows(&h, component_edit::CODE), "an insert shows its code");
    if let Some(d) = h.state_mut().state.tool_library.as_mut().and_then(|v| v.component_edit.as_mut()) {
        d.code = "CCMT09T308".to_owned();
    }
    h.run_steps(2);
    let before = done(&h);
    click_window_button(&mut h, component_edit::APPLY);
    assert_eq!(done(&h), before + 1, "the edit is one undo step");
    assert!((corner(&h, A) - 0.8).abs() < 1e-9 && (corner(&h, B) - 0.8).abs() < 1e-9, "both tools follow the insert");
    undo(&mut h);
    assert!((corner(&h, A) - 0.4).abs() < 1e-9 && (corner(&h, B) - 0.4).abs() < 1e-9, "undo restores both tools");
    redo(&mut h);
    assert!((corner(&h, A) - 0.8).abs() < 1e-9 && (corner(&h, B) - 0.8).abs() < 1e-9, "redo lands both again");
}

/// Save to My library writes the file and links the document row; a
/// newer copy in the file shows as "differs", and Sync from library takes
/// it into the document with both tools, in one undo step.
#[test]
fn a_newer_library_copy_syncs_into_the_document_and_its_tools() {
    let dir = lib_dir("sync");
    let mut h = shell(&dir);
    shop(&mut h, true);
    open_library(&mut h);
    component_edit::open(&mut h.state_mut().state, INSERT);
    h.run_steps(2);
    click_window_button(&mut h, components::SAVE);
    let saved = store::load(&lib_file(&dir)).unwrap_or_else(|e| panic!("library file: {e}"));
    assert!(saved.iter().any(|c| c.component.id() == INSERT), "the file holds the insert");
    assert!(shows(&h, "up to date"), "the row is linked to its library copy");
    let (c, g) = insert("CCMT09T308");
    if let Err(e) = store::put(&lib_file(&dir), LibraryComponent { component: c, geometry: g }) {
        panic!("library file: {e}");
    }
    h.state_mut().state.reload_component_library();
    h.run_steps(2);
    assert!(shows(&h, "differs from the library"), "the newer copy is reported, not applied");
    assert!((corner(&h, A) - 0.4).abs() < 1e-9, "an open applies nothing");
    let before = done(&h);
    click_window_button(&mut h, components::SYNC);
    assert_eq!(done(&h), before + 1, "the sync is one undo step");
    assert!((corner(&h, A) - 0.8).abs() < 1e-9 && (corner(&h, B) - 0.8).abs() < 1e-9, "both tools follow the sync");
    undo(&mut h);
    assert!((corner(&h, A) - 0.4).abs() < 1e-9 && (corner(&h, B) - 0.4).abs() < 1e-9, "undo restores both tools");
}

/// A built-in holder copies as a user holder (one undo step), opens in
/// the editor with its gauge length, and a gauge edit lands as one undo
/// step that undo takes back.
#[test]
fn a_built_in_holder_copies_as_a_user_holder_and_its_gauge_is_edited() {
    let dir = lib_dir("holder");
    let mut h = shell(&dir);
    h.get_by_role_and_label(Role::Button, "Tools").click();
    h.run_steps(2);
    click_window_button(&mut h, HOLDERS_PAGE);
    h.state_mut().state.open_holder_in_library(Some(super::tool_library_mockup::HOLDER));
    h.run_steps(2);
    let before = done(&h);
    click_window_button(&mut h, holders::COPY);
    let id = "user:holder:BT40 ER32";
    let gauge = |h: &H| {
        ta::component(&h.state().state.world, id).and_then(|r| r.geometry.port("CSW").map(|f| -f.origin()[2]))
    };
    assert_eq!(done(&h), before + 1, "the copy is one undo step");
    let Some(g0) = gauge(&h) else { panic!("the document holds the user holder {id}") };
    assert!(shows(&h, component_edit::GAUGE), "the editor opens on the copy with its gauge length");
    if let Some(d) = h.state_mut().state.tool_library.as_mut().and_then(|v| v.component_edit.as_mut()) {
        d.gauge = Some(g0 + 10.0);
    }
    h.run_steps(2);
    click_window_button(&mut h, component_edit::APPLY);
    assert_eq!(done(&h), before + 2, "the gauge edit is one undo step");
    assert!(gauge(&h).is_some_and(|g| (g - g0 - 10.0).abs() < 1e-9), "the holder's port moves to the new gauge length");
    click_window_button(&mut h, component_edit::BACK);
    h.state_mut().state.open_holder_in_library(None);
    h.run_steps(2);
    assert!(shows(&h, holders::USER_HOLDERS) && shows(&h, "BT40 ER32"), "the user holder heads the holder list");
    undo(&mut h);
    assert!(gauge(&h).is_some_and(|g| (g - g0).abs() < 1e-9), "undo gives the old gauge length back");
}

/// Frame cost: a still frame reads no library file and builds no derived
/// set; a document change builds them again.
#[test]
fn a_still_frame_reads_no_library_and_builds_nothing() {
    let dir = lib_dir("still");
    let mut h = shell(&dir);
    open_library(&mut h);
    let views = |h: &H| h.state().state.component_views.builds();
    let (loads, builds) = (crate::state::library_loads(), views(&h));
    h.run_steps(3);
    assert_eq!(crate::state::library_loads(), loads, "a still frame reads no library file");
    assert_eq!(views(&h), builds, "a still frame reads and builds no derived set");
    let add = ComponentAction::AddFromLibrary { id: "builtin:holder:BT40 ER32".into() };
    h.state_mut().state.pending.push_back(CommandKind::Component(add));
    h.run_steps(2);
    assert!(views(&h).rows > builds.rows, "a document change builds the sets again");
    assert!(ta::component(&h.state().state.world, "builtin:holder:BT40 ER32").is_some_and(|r| r.library_ref.is_some()), "the added row is linked");
}
