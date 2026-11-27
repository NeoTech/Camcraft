//! #3898 / #3648 "Done means": a turning tool assembled from linked
//! components in the tool library window. The user's library holds two
//! ISO 5608 lathe holders and an ISO 1832 insert; "Add to document"
//! puts them in the document; the turning tool's "Add component" combo
//! box offers only the lathe holders (not a spindle holder), then the
//! insert in the holder's seat; the group shows the lathe resolver's
//! values (gauge length 0, the inscribed circle as the diameter, the nose
//! radius and the orientation); "Apply stack" lands one undoable
//! `SetToolAssembly`, and "Swap" offers the other lathe holder. A groove
//! tool on the same stack shows the resolver's refusal by name, Apply
//! adds no step, and a spindle tool's empty stack offers no lathe holder.
//! Gate map: covers ui::egui_app::tool_library_dialog, ui::state.
//! Provenance: our own structural tests over the AccessKit tree and the
//! command history; no HSMWorks material involved.

use camcraft_app::component::{ComponentKind, CuttingItem, InsertSeat, ToolComponent};
use camcraft_app::component_geometry::ComponentGeometry;
use camcraft_app::component_library::{LibraryComponent, store};
use camcraft_app::connection_code::ConnectionCode;
use camcraft_app::frame::Frame;
use camcraft_app::insert_iso1832::InsertSpec;
use camcraft_app::insert_seat::HolderDesignation;
use camcraft_app::schema::AssemblyLink;
use camcraft_app::tool_assembly as ta;
use egui::accesskit::Role;
use egui_kittest::kittest::{NodeT, Queryable};

use super::tool_library_components::{done, lib_dir, open_library, pick, redo, shell, shows, undo};
use super::tool_library_editor::{H, click_window_button, library_window, open_in_window, tool};
use crate::egui_app::tool_library_dialog::{components, stack};

const TURNING: &str = "TN-1600-0R8";
const GROOVE: &str = "GT-0200";
const MILL: &str = "EM-1200-2FL";
const HOLDER_M: &str = "user:holder:PCLNR2525M12";
const HOLDER_K: &str = "user:holder:PCLNR2020K12";
const CNMG: &str = "user:insert:CNMG120408";
const SPINDLE: &str = "builtin:holder:BT40 ER32";

/// An ISO 5608 holder on its rectangular shank with its one seat `CSW1`
/// at `crp` (the frame of the #3643 app fixture). A library row holds no
/// lathe body, so the stack builds the record and no holder body.
fn holder(id: &str, code: &str, crp: [f64; 3]) -> LibraryComponent {
    let h = HolderDesignation::parse(code).expect("holder code");
    let mcs = ConnectionCode::RectShank { h: u16::from(h.shank_h), b: u16::from(h.shank_b) };
    let kind = ComponentKind::ToolItem { mcs, seats: vec![InsertSeat { port: "CSW1".into(), spec: h.seat }] };
    let f = Frame::new(crp, [0.0, 0.0, 1.0], [1.0, 0.0, 0.0]).expect("seat frame");
    let geometry = ComponentGeometry::new(vec![("CSW1".into(), f)], None, None).expect("geometry");
    LibraryComponent { component: ToolComponent::new(id, code, kind).expect("holder"), geometry }
}

fn insert(id: &str, code: &str) -> LibraryComponent {
    let spec = InsertSpec::parse(code).expect("insert code");
    let component = ToolComponent::new(id, code, ComponentKind::CuttingItem(CuttingItem::Insert(spec))).expect("insert");
    LibraryComponent { component, geometry: ComponentGeometry::default() }
}

/// A shell whose user library holds the two lathe holders and the insert,
/// with `tools` from the catalog in the document and the three lathe parts
/// and one spindle holder added to the document by clicks.
fn lathe_shop(test: &str, tools: &[&str]) -> H {
    let dir = lib_dir(test);
    let rows = [
        holder(HOLDER_M, "PCLNR2525M12", [12.5 - 32.0, 0.0, -150.0]),
        holder(HOLDER_K, "PCLNR2020K12", [10.0 - 25.0, 0.0, -125.0]),
        insert(CNMG, "CNMG120408"),
    ];
    if let Err(e) = store::save(&store::library_path(&dir), &rows) {
        panic!("library file: {e}");
    }
    let mut h = shell(&dir);
    super::rack(&mut h.state_mut().state, tools);
    h.run_steps(2);
    open_library(&mut h);
    for id in [HOLDER_M, HOLDER_K, CNMG, SPINDLE] {
        pick(&mut h, id);
        let before = done(&h);
        click_window_button(&mut h, components::ADD);
        assert_eq!(done(&h), before + 1, "Add to document of {id} is one undo step");
        assert!(ta::component(&h.state().state.world, id).is_some(), "{id} is in the document");
    }
    h
}

fn assembly(h: &H, name: &str) -> Option<Vec<AssemblyLink>> {
    let w = &h.state().state.world;
    camcraft_app::model::find_by_name(w, name).and_then(|e| ta::assembly_of(w, &e)).map(|a| a.0)
}

/// The labels the "Add component" combo box offers for tool `name`'s draft.
fn offer(h: &mut H, name: &str) -> Vec<String> {
    let st = &mut h.state_mut().state;
    let links = st.tool_library.as_ref().and_then(|v| v.stack.as_ref()).map(|d| d.links.clone()).unwrap_or_default();
    let t = camcraft_app::model::find_by_name(&st.world, name).and_then(|e| camcraft_app::model::tool_of(&st.world, &e));
    st.component_stack_view(name, t.as_ref(), &links).offer.iter().map(|(l, _)| l.clone()).collect()
}

/// Open the "Add component" combo box and pick the item `text`.
fn add_by_clicks(h: &mut H, text: &str) {
    library_window(h).get_by_role_and_label(Role::ComboBox, stack::ADD).click();
    h.run_steps(2);
    h.get_by_role_and_label(Role::Button, text).click();
    h.run_steps(3);
}

#[test]
fn a_turning_tool_is_assembled_from_linked_lathe_components_in_one_undo_step() {
    let mut h = lathe_shop("turning-stack", &[TURNING]);
    open_in_window(&mut h, TURNING);
    assert!(shows(&h, stack::HEADING), "the turning tool has the stack group");
    assert!(shows(&h, "No component stack: the tool is standalone."));
    assert_eq!(offer(&mut h, TURNING), ["PCLNR2525M12", "PCLNR2020K12"], "an empty lathe stack offers the lathe holders only");
    add_by_clicks(&mut h, "PCLNR2525M12");
    assert_eq!(offer(&mut h, TURNING), ["CNMG120408 in PCLNR2525M12 CSW1"], "the holder's seat takes the insert");
    add_by_clicks(&mut h, "CNMG120408 in PCLNR2525M12 CSW1");
    let ic = InsertSpec::parse("CNMG120408").ok().and_then(|s| s.inscribed_circle_mm()).expect("a C insert of size 12 has its IC");
    let texts = [
        format!("{}: 0.000 mm", stack::GAUGE),
        format!("{}: {ic:.3} mm", stack::INSCRIBED),
        format!("{}: 0.800 mm", stack::NOSE),
        format!("{}: 2", stack::ORIENTATION),
    ];
    for t in &texts {
        assert!(shows(&h, t), "the draft shows {t}");
    }
    assert!(shows(&h, "(from the stack edits; not applied)"));
    let corner = format!("{}: ", stack::CORNER);
    assert!(library_window(&h).query_all_by_label_contains(&corner).next().is_none(), "a lathe stack shows no corner radius");
    let before = tool(&h, TURNING);
    let steps = done(&h);
    click_window_button(&mut h, stack::APPLY);
    assert_eq!(done(&h), steps + 1, "Apply stack is one undo step");
    let ids: Vec<String> = assembly(&h, TURNING).unwrap_or_default().into_iter().map(|l| l.component).collect();
    assert_eq!(ids, [HOLDER_M, CNMG], "the stack is stored");
    let built = tool(&h, TURNING);
    assert!((built.diameter - ic).abs() < 1e-9 && (built.nose_radius - 0.8).abs() < 1e-9, "the record follows: {built:?}");
    assert!(!shows(&h, "(from the stack edits; not applied)"), "the applied stack shows its values");
    let label = stack::swap_label("PCLNR2525M12");
    let swap = library_window(&h).get_by_role_and_label(Role::Button, &label).accesskit_node().is_disabled();
    assert!(!swap, "the other lathe holder takes the insert: Swap is offered");
    undo(&mut h);
    assert_eq!(assembly(&h, TURNING), None, "one undo takes the stack away");
    assert_eq!(tool(&h, TURNING), before, "and restores the record");
    redo(&mut h);
    assert_eq!(tool(&h, TURNING), built, "redo lands the stack again");
}

#[test]
fn a_lathe_stack_the_resolver_refuses_shows_the_refusal_by_name() {
    let mut h = lathe_shop("turning-refusal", &[GROOVE, MILL]);
    open_in_window(&mut h, MILL);
    assert!(
        offer(&mut h, MILL).iter().all(|l| !l.starts_with("PCLNR")),
        "a spindle tool's empty stack offers no lathe holder: {:?}",
        offer(&mut h, MILL)
    );
    open_in_window(&mut h, GROOVE);
    add_by_clicks(&mut h, "PCLNR2525M12");
    add_by_clicks(&mut h, "CNMG120408 in PCLNR2525M12 CSW1");
    let refusal = format!("The stack does not resolve: tool '{GROOVE}': a ");
    assert!(
        library_window(&h).query_all_by_label_contains(&refusal).next().is_some(),
        "the group names the refusal"
    );
    assert!(library_window(&h).query_all_by_label_contains("assembly is not resolved yet").next().is_some());
    let steps = done(&h);
    click_window_button(&mut h, stack::APPLY);
    assert_eq!(done(&h), steps, "a refused stack adds no undo step");
    assert_eq!(assembly(&h, GROOVE), None, "and stores nothing");
}
