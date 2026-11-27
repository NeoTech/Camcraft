//! #3644: the document component table and the assembly commands. One
//! shared insert sits in two indexable cutters (Ø25 and Ø32 on one BT40
//! holder). Pinned: an insert edit updates both tools in one undo step,
//! their operations go Stale, and undo and redo restore the exact state;
//! an edit that one assembly cannot take, a delete of a used component,
//! and an incompatible stack are refused by name and change nothing.
//! A milling insert stands in for the issue's turning insert: the lathe
//! side of the resolver is still open on #3643. Provenance: our own
//! tests over our own commands.

use camcraft_app::commands::{
    CreateOperation, DeleteDocumentComponent, DeleteTool, EditDocumentComponent, GenerateOperation, History,
    SetToolAssembly, UpsertDocumentComponent,
};
use camcraft_app::model;
use camcraft_app::operation::{Height, OpParams, OpStatus, SrcRef, status_of};
use camcraft_app::schema::{AssemblyLink, DocumentComponent, ToolAssembly, ToolBody};
use camcraft_app::tool_assembly::{assembly_of, components, ASSEMBLY_SOURCE};
use camcraft_app::tool_body::body_of;
use camcraft_app::tools::{Tool, ToolKind};
use camcraft_tools::component::{ComponentKind, CuttingItem, InsertSeat, ToolComponent};
use camcraft_tools::component_geometry::{ComponentBody, ComponentGeometry};
use camcraft_tools::connection_code::ConnectionCode;
use camcraft_tools::frame::Frame;
use camcraft_tools::insert_iso1832::InsertSpec;
use camcraft_tools::insert_seat::HolderDesignation;
use camcraft_tools::spun::SpunPoint;
use camcraft_tools::tool_assembly::Mount;
use flecs_ecs::prelude::World;

use super::cat;

const A: &str = "Fly25";
const B: &str = "Fly32";
const INSERT: &str = "insert-cc";

fn code(text: &str) -> ConnectionCode {
    ConnectionCode::parse(text).expect("connection code")
}

fn holder() -> (ToolComponent, ComponentGeometry) {
    let p = SpunPoint::line;
    let body = ComponentBody::new(vec![p(0.0, 0.0), p(0.0, 25.0), p(50.0, 25.0), p(50.0, 0.0)], -50.0).expect("body");
    let kind = ComponentKind::AdaptiveItem { mcs: code("BT40"), csw: code("CYL 20") };
    let c = ToolComponent::new("bt40-cyl20", "BT40 CYL 20", kind).expect("holder");
    (c, ComponentGeometry::axial("CSW", 50.0, Some(body)).expect("geometry"))
}

/// A single-seat cutter body with a CC seat `r` off the axis.
fn fly(id: &str, r: f64) -> (ToolComponent, ComponentGeometry) {
    let seat = HolderDesignation::parse("SCLCR2020K09").expect("holder code").seat;
    let kind = ComponentKind::ToolItem { mcs: code("CYL 20"), seats: vec![InsertSeat { port: "CSW1".into(), spec: seat }] };
    let frame = Frame::new([r, 0.0, -60.0], [0.0, 0.0, 1.0], [1.0, 0.0, 0.0]).expect("seat frame");
    let g = ComponentGeometry::new(vec![("CSW1".into(), frame)], None, None).expect("geometry");
    (ToolComponent::new(id, id, kind).expect("cutter body"), g)
}

fn insert(text: &str) -> (ToolComponent, ComponentGeometry) {
    let spec = InsertSpec::parse(text).expect("insert code");
    let c = ToolComponent::new(INSERT, text, ComponentKind::CuttingItem(CuttingItem::Insert(spec))).expect("insert");
    (c, ComponentGeometry::default())
}

fn link(id: &str, mount: Option<Mount>) -> AssemblyLink {
    AssemblyLink { component: id.into(), mount }
}

fn on(parent: usize, port: Option<&str>) -> Option<Mount> {
    Some(Mount { parent, port: port.map(str::to_owned), z_rot_deg: 0.0, projection_mm: None })
}

fn stack(body: &str) -> Vec<AssemblyLink> {
    vec![link("bt40-cyl20", None), link(body, on(0, None)), link(INSERT, on(1, Some("CSW1")))]
}

/// A short indexable cutter record: what the assembly does not state.
fn cutter() -> Tool {
    Tool { kind: ToolKind::BullNose, diameter: 20.0, corner_r: 0.2, flute_len: 5.0, shank_d: 20.0, ..cat("EM-1200-2FL") }
}

fn face() -> OpParams {
    OpParams::Face {
        holes: Default::default(),
        tilt: None,
        blend_turns: false,
        blend_radius: None,
        stepover: 3.0,
        clearance_z: Height::fixed(25.0),
        cut_z: Height::fixed(19.5),
        top_z: None,
        stepdown: None,
        feed: 1200.0,
        plunge_feed: 300.0,
        climb: true,
        picks: Default::default(),
    }
}

fn upsert(w: &World, h: &mut History, (c, g): (ToolComponent, ComponentGeometry)) {
    h.execute(w, UpsertDocumentComponent::new(w, c, g).expect("upsert"));
}

/// Stock, the four components, both tools on their stacks, and a
/// generated Face operation on each.
fn shop() -> (World, History) {
    let w = camcraft_app::world::new_world();
    let mut h = History::default();
    let stock = camcraft_app::stock::StockShape::Box { dx: 60.0, dy: 60.0, dz: 20.0 };
    h.execute(&w, camcraft_app::commands::SetStock::new(&w, stock, Default::default()).expect("stock"));
    for c in [holder(), fly("fly-25", 12.5), fly("fly-32", 16.0), insert("CCMT09T304")] {
        upsert(&w, &mut h, c);
    }
    for (tool, body) in [(A, "fly-25"), (B, "fly-32")] {
        h.execute(&w, camcraft_app::commands::AddTool::new(&w, tool, cutter()).expect("tool"));
        h.execute(&w, SetToolAssembly::new(&w, tool, stack(body)).expect("assembly"));
        let op = format!("Face{tool}");
        h.execute(&w, CreateOperation::new(&w, &op, tool, SrcRef::Stock, face()).expect("op"));
        h.execute(&w, GenerateOperation::new(&w, &op).expect("generate"));
    }
    (w, h)
}

type Snapshot = (Vec<DocumentComponent>, Vec<(Tool, Option<ToolBody>, Option<ToolAssembly>)>);

fn snapshot(w: &World) -> Snapshot {
    let tool = |n: &str| {
        let e = model::find_by_name(w, n).expect("tool");
        (model::tool_of(w, &e).expect("a tool"), body_of(w, &e), assembly_of(w, &e))
    };
    (components(w), vec![tool(A), tool(B)])
}

fn op_status(w: &World, tool: &str) -> OpStatus {
    status_of(w, &model::find_by_name(w, &format!("Face{tool}")).expect("op"))
}

#[test]
fn set_tool_assembly_lands_the_derived_tool_and_body_in_one_undo_step() {
    let (w, mut h) = shop();
    let (_, tools) = snapshot(&w);
    let (a, b) = (&tools[0], &tools[1]);
    assert!((a.0.diameter - 25.0).abs() < 1e-9 && (b.0.diameter - 32.0).abs() < 1e-9, "{} {}", a.0.diameter, b.0.diameter);
    assert!((a.0.corner_r - 0.4).abs() < 1e-12, "the insert's nose radius");
    assert_eq!(a.1.as_ref().map(|b| b.source.as_str()), Some(ASSEMBLY_SOURCE));
    assert_ne!(op_status(&w, A), OpStatus::Stale, "generated after the assembly");

    // Undo the generate, the op, then the assembly: the tool is bare again.
    for _ in 0..3 {
        assert!(h.undo(&w));
    }
    let e = model::find_by_name(&w, B).expect("tool");
    assert_eq!(model::tool_of(&w, &e), Some(cutter()), "the record before the assembly");
    assert!(body_of(&w, &e).is_none() && assembly_of(&w, &e).is_none(), "no body, no stack");
}

#[test]
fn one_shared_insert_edit_updates_two_assemblies_in_one_undo_step() {
    let (w, mut h) = shop();
    let before = snapshot(&w);
    let (c, g) = insert("CCMT09T308");
    let edit = EditDocumentComponent::new(&w, c, g).expect("an insert both seats take");
    assert_eq!(edit.updated_tools(), vec![A.to_owned(), B.to_owned()]);
    h.execute(&w, edit);
    let after = snapshot(&w);
    for (t, d) in after.1.iter().zip([25.0, 32.0]) {
        assert!((t.0.corner_r - 0.8).abs() < 1e-12 && (t.0.diameter - d).abs() < 1e-9, "{:?}", (t.0.corner_r, t.0.diameter));
    }
    assert_eq!(op_status(&w, A), OpStatus::Stale, "the Ø25 operation re-marks");
    assert_eq!(op_status(&w, B), OpStatus::Stale, "the Ø32 operation re-marks");

    assert!(h.undo(&w), "one undo step");
    assert_eq!(snapshot(&w), before, "undo restores the component, both tools and both bodies");
    assert_ne!(op_status(&w, A), OpStatus::Stale);
    assert!(h.redo(&w));
    assert_eq!(snapshot(&w), after, "redo lands the same edit");
}

#[test]
fn an_edit_one_assembly_cannot_take_is_refused_whole_by_name() {
    let (w, _h) = shop();
    let before = snapshot(&w);
    let (c, g) = insert("DCMT11T304");
    let Err(e) = EditDocumentComponent::new(&w, c.clone(), g.clone()) else { panic!("a D insert in a C seat") };
    let msg = e.to_string();
    assert!(msg.contains(INSERT) && msg.contains(A) && msg.contains("does not fit"), "{msg}");
    assert!(UpsertDocumentComponent::new(&w, c, g).is_err(), "the upsert of a held id takes the same law");
    assert_eq!(snapshot(&w), before, "nothing changed");
}

#[test]
fn a_used_component_is_not_deleted_and_an_unused_one_is() {
    let (w, mut h) = shop();
    let Err(e) = DeleteDocumentComponent::new(&w, INSERT) else { panic!("the insert is in use") };
    let msg = e.to_string();
    assert!(msg.contains(INSERT) && msg.contains(&format!("{A}, {B}")), "{msg}");
    assert!(DeleteDocumentComponent::new(&w, "no-such").is_err(), "an id the table does not hold");

    upsert(&w, &mut h, fly("spare", 10.0));
    let rows = components(&w);
    h.execute(&w, DeleteDocumentComponent::new(&w, "spare").expect("unused"));
    assert!(!components(&w).iter().any(|r| r.component.id() == "spare"));
    assert!(h.undo(&w));
    assert_eq!(components(&w), rows, "undo restores the row in its place");
}

#[test]
fn an_incompatible_stack_is_refused_by_name_and_changes_nothing() {
    let (w, h) = shop();
    let before = snapshot(&w);
    let depth = h.can_undo();
    // The insert straight on the holder: a CYL 20 bore takes no insert.
    let bad = vec![link("bt40-cyl20", None), link(INSERT, on(0, None))];
    let Err(e) = SetToolAssembly::new(&w, A, bad) else { panic!("an insert at a bore") };
    assert!(e.to_string().contains(A), "{e}");
    let Err(e) = SetToolAssembly::new(&w, A, stack("no-such-body")) else { panic!("an unknown component") };
    assert!(e.to_string().contains("no-such-body"), "{e}");
    assert_eq!(snapshot(&w), before, "nothing changed");
    assert_eq!(h.can_undo(), depth);
}

#[test]
fn a_deleted_tool_comes_back_with_its_stack() {
    let (w, mut h) = shop();
    let before = snapshot(&w);
    h.execute(&w, DeleteTool::new(&w, B).expect("delete"));
    assert!(model::find_by_name(&w, B).is_none());
    assert!(camcraft_app::tool_assembly::users_of(&w, INSERT) == vec![A.to_owned()], "only the tool left uses it");
    assert!(h.undo(&w));
    assert_eq!(snapshot(&w), before, "the stack, the record and the body come back");
}

#[test]
fn review_generic_delete_undo_keeps_assembly() {
    // #4201 review: the tree's delete runs the generic `DeleteEntity`. Its
    // undo must bring the stack back too, or a later shared edit skips
    // the tool.
    let (w, mut h) = shop();
    let before = snapshot(&w);
    h.execute(&w, camcraft_app::commands::DeleteEntity::new(&w, B).expect("delete"));
    assert!(model::find_by_name(&w, B).is_none());
    assert!(h.undo(&w));
    let e = model::find_by_name(&w, B).expect("restored");
    assert_eq!(assembly_of(&w, &e), before.1[1].2, "generic delete undo must restore the assembly");
    assert_eq!(snapshot(&w), before, "the stack, the record and the body come back");
    let (c, g) = insert("CCMT09T308");
    let edit = EditDocumentComponent::new(&w, c, g).expect("an insert both seats take");
    assert_eq!(edit.updated_tools(), vec![A.to_owned(), B.to_owned()], "the restored tool takes shared edits");
}
