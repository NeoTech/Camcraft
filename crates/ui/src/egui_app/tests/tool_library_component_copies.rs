//! #3898 (U7 of #3663), the review of PR #4271: "Copy as user component"
//! of a document row keeps the whole row - its #3646 cutter facts and
//! running data, its #4278 lathe body - under the new id, without the library link, in one
//! undo step; "Add to document" by a click is one undo step that undo
//! takes back with its link and redo lands again; the component pages'
//! caches in State build only when their input changes.
//! Gate map: covers ui::egui_app::tool_library_dialog, ui::state.
//! Provenance: our own structural tests over the AccessKit tree and the
//! command history; no HSMWorks material involved.

use camcraft_app::commands::UpsertDocumentComponent;
use camcraft_app::lathe_assembly::{BodyBox, LatheBody};
use camcraft_app::milling_cutter::MillingBody;
use camcraft_app::schema::{DocumentComponent, InsertRunning, LibraryRef};
use camcraft_app::tool_assembly as ta;
use camcraft_app::tools::ToolMaterial;

use super::tool_library_components::{A, INSERT, done, fly, insert, lib_dir, open_library, pick, redo, shell, shop, undo};
use super::tool_library_editor::{H, click_window_button, open_in_window};
use crate::egui_app::tool_library_dialog::components;
use crate::state::{CommandKind, ComponentAction};

fn row(h: &H, id: &str) -> Option<DocumentComponent> {
    ta::component(&h.state().state.world, id)
}

fn queue(h: &mut H, action: ComponentAction) {
    h.state_mut().state.pending.push_back(CommandKind::Component(action));
    h.run_steps(2);
}

fn views(h: &H) -> crate::state::ViewBuilds {
    h.state().state.component_views.builds()
}

/// Three linked document rows: a cutter body with its #3646 facts, an
/// insert with its running data, a turning holder with its lathe body.
fn rows_with_facts(h: &mut H) {
    let link = Some(LibraryRef { id: "user:body:Shop".into(), content_hash: "0".into() });
    let st = &mut h.state_mut().state;
    let (c, g) = fly("fly-25", 12.5);
    let milling = MillingBody { dc: 26.0, kapr_deg: 90.0, apmx: 8.0, center_cutting: false, seat_count: Some(1) };
    let body = DocumentComponent { milling: Some(milling), library_ref: link.clone(), ..DocumentComponent::new(c, g) };
    let (c, g) = insert("CCMT09T304");
    let running = InsertRunning { tool_material: Some(ToolMaterial::Carbide), cutting: Vec::new() };
    let ins = DocumentComponent { running: Some(running), library_ref: link.clone(), ..DocumentComponent::new(c, g) };
    let (c, g) = fly("turn-holder", 0.0);
    let boxes = [BodyBox { lo: [-10.0, -10.0, -60.0], hi: [10.0, 10.0, 0.0] }];
    let lathe_body = LatheBody::boxes(&boxes).unwrap_or_else(|e| panic!("lathe body: {e:?}"));
    let turn = DocumentComponent { lathe_body: Some(lathe_body), library_ref: link.clone(), ..DocumentComponent::new(c, g) };
    for r in [body, ins, turn] {
        let cmd = UpsertDocumentComponent::with_row(&st.world, r).unwrap_or_else(|e| panic!("row: {e}"));
        st.history.execute(&st.world, cmd);
    }
    st.doc_version += 1;
    h.run_steps(2);
}

/// A copy of a document row keeps every field but its id and its library
/// link: the cutter body's facts, the insert's running data and the
/// turning holder's lathe body too. The
/// copy is one undo step; undo removes it and redo lands it whole again.
#[test]
fn a_copy_of_a_document_row_keeps_its_facts_and_running_data() {
    let dir = lib_dir("copy-row");
    let mut h = shell(&dir);
    rows_with_facts(&mut h);
    let pairs = [("fly-25", "user:body:fly-25"), (INSERT, "user:insert:CCMT09T304"), ("turn-holder", "user:body:turn-holder")];
    for (src, copy) in pairs {
        let before = done(&h);
        queue(&mut h, ComponentAction::CopyAsUser { id: src.into() });
        assert_eq!(done(&h), before + 1, "the copy of {src} is one undo step");
        let (Some(old), Some(new)) = (row(&h, src), row(&h, copy)) else {
            panic!("the document holds {src} and its copy {copy}");
        };
        assert!(old.milling.is_some() || old.running.is_some() || old.lathe_body.is_some(), "{src} has row data");
        assert_eq!(new.component.id(), copy);
        assert_eq!(new.library_ref, None, "the copy has no library link");
        let want = DocumentComponent { component: new.component.clone(), library_ref: None, ..old };
        assert_eq!(new, want, "the copy of {src} keeps the whole row");
        undo(&mut h);
        assert_eq!(row(&h, copy), None, "undo removes the copy of {src}");
        redo(&mut h);
        assert_eq!(row(&h, copy), Some(want), "redo lands the whole copy of {src} again");
    }
}

/// "Add to document" by a click: one undo step for the row and its
/// library link; one undo removes both, redo lands both again.
#[test]
fn add_to_document_by_a_click_is_one_undo_step_with_its_link() {
    let dir = lib_dir("add-undo");
    let mut h = shell(&dir);
    open_library(&mut h);
    let id = "builtin:holder:BT40 ER32";
    pick(&mut h, id);
    let before = done(&h);
    click_window_button(&mut h, components::ADD);
    let linked = |h: &H| row(h, id).map(|r| r.library_ref.is_some());
    assert_eq!(done(&h), before + 1, "the add and its link are one undo step");
    assert_eq!(linked(&h), Some(true), "the added row is linked");
    undo(&mut h);
    assert_eq!(done(&h), before, "one undo takes the whole add back");
    assert_eq!(linked(&h), None, "one undo removes the row and its link");
    redo(&mut h);
    assert_eq!(linked(&h), Some(true), "redo lands the row and its link");
}

/// Frame cost: the compatible set, the rows and the stack offer are kept
/// in State. A still frame reads and builds nothing; a document change
/// that leaves the table and the stacks reads them once and builds
/// nothing; a filter, a draft or a table change builds its set again.
#[test]
fn the_component_caches_build_on_a_changed_input_only() {
    let dir = lib_dir("views");
    let mut h = shell(&dir);
    shop(&mut h, true);
    open_library(&mut h);
    pick(&mut h, "builtin:holder:BT40 ER32");
    let b0 = views(&h);
    click_window_button(&mut h, components::SHOW_COMPATIBLE);
    assert_eq!(views(&h).compatible, b0.compatible + 1, "a new filter builds its set once");
    let b1 = views(&h);
    h.run_steps(3);
    assert_eq!(views(&h), b1, "a still frame reads and builds nothing");
    open_in_window(&mut h, A);
    let b2 = views(&h);
    assert!(b2.stack > b1.stack, "the stack group reads its offer through the cache");
    h.run_steps(3);
    assert_eq!(views(&h), b2, "a still frame of the stack group reads and builds nothing");
    h.state_mut().state.doc_version += 1;
    h.run_steps(2);
    let b3 = views(&h);
    assert_eq!(b3.reads, b2.reads + 1, "a document change reads the table once");
    assert_eq!((b3.rows, b3.stack), (b2.rows, b2.stack), "the same table and stacks build nothing");
    if let Some(s) = h.state_mut().state.tool_library.as_mut().and_then(|v| v.stack.as_mut()) {
        s.links.pop();
    }
    h.run_steps(3);
    let b4 = views(&h);
    assert_eq!(b4.stack, b3.stack + 1, "a draft edit builds the offer once");
    assert_eq!(b4.rows, b3.rows, "a draft edit builds no rows");
    queue(&mut h, ComponentAction::CopyAsUser { id: "fly-32".into() });
    // The copy opens in the component editor; back to the tool's stack.
    if let Some(v) = h.state_mut().state.tool_library.as_mut() {
        v.component_edit = None;
    }
    h.run_steps(2);
    let b5 = views(&h);
    assert!(b5.rows > b4.rows && b5.stack > b4.stack, "a table change builds the rows and the offer again");
    h.run_steps(3);
    assert_eq!(views(&h), b5, "and then holds");
}
