//! #3643: a turning tool assembled from linked document components (an
//! ISO 5608 holder and an ISO 1832 insert) through the document
//! commands. Pinned: a CNMG in a PCLNR gives the same Turning record and
//! the same lathe body as the tools-crate resolver; one edit of the
//! shared insert updates two holder assemblies in one undo step, their
//! turning operations go Stale, and undo and redo are exact; a kind,
//! a root or a stack that the lathe resolver does not take is refused
//! by name and changes nothing. #3645: the rows with their lathe bodies
//! save in the `.camproj` and open again identically, byte for byte, and
//! the reopened document derives the same records and bodies; a corrupt
//! lathe body refuses the file by name; a component library save of a
//! row with a lathe body is refused by name. Provenance: our own tests
//! over our own commands and format.

use camcraft_app::commands::{
    AddTool, CreateOperation, EditDocumentComponent, GenerateOperation, History, ImportModel, LoadMachine,
    SetStock, SetToolAssembly, UpsertDocumentComponent,
};
use camcraft_app::geometry::{Mesh, Point3, Triangle};
use camcraft_app::machine::Machine;
use camcraft_app::model;
use camcraft_app::operation::{Height, OpParams, OpStatus, SrcRef, status_of};
use camcraft_app::schema::{AssemblyLink, DocumentComponent, Location, ToolAssembly, ToolBody};
use camcraft_app::tool_assembly::{ASSEMBLY_SOURCE, assembly_of, components};
use camcraft_app::tool_body::{BodyFrame, body_of};
use camcraft_app::tools::{Tool, ToolKind, catalog};
use camcraft_tools::component::{ComponentKind, CuttingItem, InsertSeat, ToolComponent};
use camcraft_tools::component_geometry::ComponentGeometry;
use camcraft_tools::connection_code::ConnectionCode;
use camcraft_tools::frame::Frame;
use camcraft_tools::insert_iso1832::InsertSpec;
use camcraft_tools::insert_seat::HolderDesignation;
use camcraft_tools::lathe_assembly::{BodyBox, LatheBody, LatheNode, LatheStation, resolve_lathe};
use camcraft_tools::tool_assembly::{AssemblyNode, Mount};
use flecs_ecs::prelude::World;

use super::cat;

pub(super) const A: &str = "OD-M";
const B: &str = "OD-K";
const INSERT: &str = "cnmg";
pub(super) const HOLDER_M: &str = "pclnr-2525m";
const HOLDER_K: &str = "pclnr-2020k";

fn bx(lo: [f64; 3], hi: [f64; 3]) -> BodyBox {
    BodyBox { lo, hi }
}

/// An ISO 5608 holder on its rectangular shank (one seat `CSW1` at the
/// CRP `crp`) and its boxes: the MCS on the shank's back face, +Z out
/// of the back, +X along the spindle away from the chuck; the shank
/// below centre height and a head 1 mm back from the CRP, 5 mm under
/// the insert's top face.
fn holder(id: &str, code: &str, crp: [f64; 3]) -> DocumentComponent {
    let h = HolderDesignation::parse(code).expect("holder code");
    let (sh, sb) = (f64::from(h.shank_h), f64::from(h.shank_b));
    let mcs = ConnectionCode::RectShank { h: u16::from(h.shank_h), b: u16::from(h.shank_b) };
    let kind = ComponentKind::ToolItem { mcs, seats: vec![InsertSeat { port: "CSW1".into(), spec: h.seat }] };
    let f = Frame::new(crp, [0.0, 0.0, 1.0], [1.0, 0.0, 0.0]).expect("seat frame");
    let g = ComponentGeometry::new(vec![("CSW1".into(), f)], None, None).expect("geometry");
    let shank_end = crp[2] + 25.0;
    let boxes = [
        bx([-sb * 0.5, -sh, shank_end], [sb * 0.5, 0.0, 0.0]),
        bx([crp[0] + 1.0, -sh, crp[2] + 1.0], [sb * 0.5, -5.0, shank_end]),
    ];
    DocumentComponent {
        lathe_body: Some(LatheBody::boxes(&boxes).expect("boxes")),
        ..DocumentComponent::new(ToolComponent::new(id, code, kind).expect("holder"), g)
    }
}

/// 2525M: l1 = 150 mm, f1 = 32 mm from the shank's +X side face.
pub(super) fn holder_m() -> DocumentComponent {
    holder(HOLDER_M, "PCLNR2525M12", [12.5 - 32.0, 0.0, -150.0])
}

/// 2020K: l1 = 125 mm, f1 = 25 mm.
fn holder_k() -> DocumentComponent {
    holder(HOLDER_K, "PCLNR2020K12", [10.0 - 25.0, 0.0, -125.0])
}

pub(super) fn insert(code: &str) -> DocumentComponent {
    let spec = InsertSpec::parse(code).expect("insert code");
    let c = ToolComponent::new(INSERT, code, ComponentKind::CuttingItem(CuttingItem::Insert(spec))).expect("insert");
    DocumentComponent::new(c, ComponentGeometry::default())
}

pub(super) fn on(parent: usize) -> Option<Mount> {
    Some(Mount { parent, port: None, z_rot_deg: 0.0, projection_mm: None })
}

pub(super) fn stack(holder: &str) -> Vec<AssemblyLink> {
    vec![AssemblyLink { component: holder.into(), mount: None }, AssemblyLink { component: INSERT.into(), mount: on(0) }]
}

pub(super) fn turning() -> Tool {
    cat("TN-1600-0R8")
}

pub(super) fn upsert(w: &World, h: &mut History, row: DocumentComponent) {
    h.execute(w, UpsertDocumentComponent::with_row(w, row).expect("upsert"));
}

/// The closed revolve about world Z of `meridian` (r, z).
fn revolve(meridian: &[(f64, f64)], segs: usize) -> Mesh {
    let v = |j: usize, r: f64, z: f64| {
        let th = std::f64::consts::TAU * j as f64 / segs as f64;
        Point3::new(r * th.cos(), r * th.sin(), z)
    };
    let mut facets = Vec::new();
    for seg in meridian.windows(2) {
        let ((ra, za), (rb, zb)) = (seg[0], seg[1]);
        for j in 0..segs {
            let (a1, a2, b1, b2) = (v(j, ra, za), v(j + 1, ra, za), v(j, rb, zb), v(j + 1, rb, zb));
            if ra <= 1e-12 {
                facets.push(Triangle::new(a1, b2, b1));
            } else if rb <= 1e-12 {
                facets.push(Triangle::new(a1, a2, b2));
            } else {
                facets.push(Triangle::new(a1, a2, b2));
                facets.push(Triangle::new(a1, b2, b1));
            }
        }
    }
    Mesh::from_facets(facets)
}

fn facing() -> OpParams {
    OpParams::Facing {
        tolerance: 0.05,
        clearance_z: Height::fixed(25.0),
        top_z: Height::fixed(16.0),
        bottom_z: Height::fixed(14.0),
        stepdown: 0.5,
        feed: 150.0,
        plunge_feed: 80.0,
        picks: Default::default(),
    }
}

/// A lathe, a bar, the three components, both tools on their stacks
/// and a generated Facing operation on each.
pub(super) fn lathe_shop() -> (World, History) {
    let w = camcraft_app::world::new_world();
    let mut h = History::default();
    let stock = camcraft_app::stock::StockShape::Box { dx: 40.0, dy: 40.0, dz: 40.0 };
    h.execute(&w, SetStock::new(&w, stock, Location::default()).expect("stock"));
    let lathe = Machine { model: "Bench lathe".into(), kind: Some("lathe".into()), spindle_max: Some(6000.0), ..Default::default() };
    h.execute(&w, LoadMachine::new(&w, "lathe.mch".into(), Some(lathe)).expect("lathe"));
    let bar = [(0.0, -16.0), (6.0, -16.0), (6.0, 2.0), (5.0, 2.0), (5.0, 16.0), (0.0, 16.0)];
    h.execute(&w, ImportModel::from_mesh("bar", revolve(&bar, 16)));
    for row in [holder_m(), holder_k(), insert("CNMG120408")] {
        upsert(&w, &mut h, row);
    }
    for (tool, holder) in [(A, HOLDER_M), (B, HOLDER_K)] {
        h.execute(&w, AddTool::new(&w, tool, turning()).expect("tool"));
        h.execute(&w, SetToolAssembly::new(&w, tool, stack(holder)).expect("assembly"));
        let op = format!("Face{tool}");
        h.execute(&w, CreateOperation::new(&w, &op, tool, SrcRef::Entity("bar".into()), facing()).expect("op"));
        h.execute(&w, GenerateOperation::new(&w, &op).expect("generate"));
    }
    (w, h)
}

pub(super) type Snapshot = (Vec<DocumentComponent>, Vec<(Tool, Option<ToolBody>, Option<ToolAssembly>)>);

pub(super) fn snapshot(w: &World) -> Snapshot {
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
fn a_linked_cnmg_in_a_pclnr_equals_the_tools_crate_result() {
    equals_the_tools_crate_result(0.0);
}

/// #3643: a holder body with a pad (a meshed body's chord deviation)
/// gives the app's lathe body the resolver's grown bands, never the
/// bands of the bare facets.
#[test]
fn a_padded_holder_body_keeps_the_grown_bands_in_the_app() {
    equals_the_tools_crate_result(0.05);
}

/// The 2525M holder, its body grown by `pad`.
pub(super) fn holder_m_padded(pad: f64) -> DocumentComponent {
    let m = holder_m();
    DocumentComponent { lathe_body: m.lathe_body.map(|b| b.padded(pad).expect("pad")), ..m }
}

fn equals_the_tools_crate_result(pad: f64) {
    let w = camcraft_app::world::new_world();
    let mut h = History::default();
    for row in [holder_m_padded(pad), insert("CNMG120408")] {
        upsert(&w, &mut h, row);
    }
    h.execute(&w, AddTool::new(&w, A, turning()).expect("tool"));
    h.execute(&w, SetToolAssembly::new(&w, A, stack(HOLDER_M)).expect("assembly"));
    let e = model::find_by_name(&w, A).expect("tool");
    let (tool, body) = (model::tool_of(&w, &e).expect("a tool"), body_of(&w, &e).expect("a lathe body"));

    // The tools-crate result for the same two components.
    let (m, i) = (holder_m_padded(pad), insert("CNMG120408"));
    let nodes = vec![
        LatheNode { node: AssemblyNode { component: m.component, geometry: m.geometry, mount: None }, body: m.lathe_body },
        LatheNode { node: AssemblyNode { component: i.component, geometry: i.geometry, mount: on(0) }, body: None },
    ];
    let r = resolve_lathe(&nodes, LatheStation::RADIAL_SHANK, None).expect("resolved");

    // The record: the user's Turning tool with the insert's values.
    let want = Tool { nose_radius: 0.8, diameter: 12.7, orientation: None, ..turning() };
    assert_eq!(tool, want, "the record a user types for the same insert");
    assert_eq!(tool.lathe_orientation(), Some(r.params.orientation));
    assert!((tool.nose_radius - r.params.nose_radius).abs() < 1e-12);

    // The body: the resolver's placement, facets and bands.
    assert_eq!(body.source, ASSEMBLY_SOURCE);
    let BodyFrame::Lathe { place, zone, clearance, pad: body_pad } = &body.frame else { panic!("a lathe-frame body") };
    assert_eq!(*place, r.placement, "the nose centre is the tip");
    assert!(zone.is_none(), "the insert plate is left out by the resolver, not by a zone");
    assert!((r.pad - pad).abs() < 1e-12, "the stack's pad");
    assert_eq!(*body_pad, r.pad, "the body keeps the stack's pad");
    let want_bands = r.clearance.expect("bands");
    let (a, b): (Vec<f64>, Vec<f64>) = (clearance.words().collect(), want_bands.words().collect());
    assert_eq!(a, b, "the same clearance bands");
    let facets: Vec<[[f64; 3]; 3]> =
        body.facets.iter().map(|t| [[t.a.x, t.a.y, t.a.z], [t.b.x, t.b.y, t.b.z], [t.c.x, t.c.y, t.c.z]]).collect();
    assert_eq!(facets, r.placed, "the same placed facets");
}

#[test]
fn one_shared_turning_insert_edit_updates_two_holders_in_one_undo_step() {
    let (w, mut h) = lathe_shop();
    assert_ne!(op_status(&w, A), OpStatus::Stale, "generated after the assembly");
    assert_ne!(op_status(&w, B), OpStatus::Stale, "generated after the assembly");
    let before = snapshot(&w);
    let edit = EditDocumentComponent::with_row(&w, insert("CNMG120404")).expect("both seats take it");
    assert_eq!(edit.updated_tools(), vec![B.to_owned(), A.to_owned()]);
    h.execute(&w, edit);
    let after = snapshot(&w);
    for (old, new) in before.1.iter().zip(&after.1) {
        assert!((new.0.nose_radius - 0.4).abs() < 1e-12, "the new nose radius: {}", new.0.nose_radius);
        let tip = |b: &Option<ToolBody>| match b.as_ref().map(|b| &b.frame) {
            Some(BodyFrame::Lathe { place, .. }) => place.tip,
            _ => panic!("a lathe body"),
        };
        assert_ne!(tip(&old.1), tip(&new.1), "the nose centre moved with the nose radius");
    }
    assert_eq!(op_status(&w, A), OpStatus::Stale, "the 2525M operation re-marks");
    assert_eq!(op_status(&w, B), OpStatus::Stale, "the 2020K operation re-marks");

    assert!(h.undo(&w), "one undo step");
    assert_eq!(snapshot(&w), before, "undo restores the insert, both tools and both bodies");
    assert_ne!(op_status(&w, A), OpStatus::Stale);
    assert!(h.redo(&w));
    assert_eq!(snapshot(&w), after, "redo lands the same edit");
}

#[test]
fn a_kind_a_root_or_a_stack_the_lathe_resolver_does_not_take_is_refused_by_name() {
    let w = camcraft_app::world::new_world();
    let mut h = History::default();
    for row in [holder_m(), insert("CNMG120408")] {
        upsert(&w, &mut h, row);
    }
    // A grooving tool: no blade values from the resolver yet.
    let groove = catalog().into_iter().map(|(_, t)| t).find(|t| t.kind == ToolKind::Groove).expect("a grooving tool");
    h.execute(&w, AddTool::new(&w, "G1", groove).expect("tool"));
    let Err(e) = SetToolAssembly::new(&w, "G1", stack(HOLDER_M)) else { panic!("a grooving stack") };
    assert!(e.to_string().contains("G1") && e.to_string().contains("not resolved yet"), "{e}");

    // A VDI root: its station is not stated.
    let vdi = ComponentKind::AdaptiveItem { mcs: ConnectionCode::Vdi { d: 40 }, csw: ConnectionCode::RectShank { h: 25, b: 25 } };
    let sleeve = ToolComponent::new("vdi40", "VDI40 block", vdi).expect("block");
    upsert(&w, &mut h, DocumentComponent::new(sleeve, ComponentGeometry::axial("CSW", 60.0, None).expect("geometry")));
    h.execute(&w, AddTool::new(&w, A, turning()).expect("tool"));
    let on_vdi = vec![
        AssemblyLink { component: "vdi40".into(), mount: None },
        AssemblyLink { component: HOLDER_M.into(), mount: on(0) },
        AssemblyLink { component: INSERT.into(), mount: on(1) },
    ];
    let Err(e) = SetToolAssembly::new(&w, A, on_vdi) else { panic!("a VDI root") };
    assert!(e.to_string().contains(A) && e.to_string().contains("VDI40 block"), "{e}");

    // A resolver refusal: a left-hand code on the right-hand geometry.
    upsert(&w, &mut h, holder("left", "PCLNL2525M12", [12.5 - 32.0, 0.0, -150.0]));
    let Err(e) = SetToolAssembly::new(&w, A, stack("left")) else { panic!("a wrong hand") };
    assert!(e.to_string().contains(A) && e.to_string().contains("lathe assembly"), "{e}");

    // A spindle tool never drops a lathe body in silence.
    h.execute(&w, AddTool::new(&w, "E1", cat("EM-1200-2FL")).expect("tool"));
    let Err(e) = SetToolAssembly::new(&w, "E1", stack(HOLDER_M)) else { panic!("a lathe body on a spindle tool") };
    assert!(e.to_string().contains("E1") && e.to_string().contains(HOLDER_M), "{e}");

    // Nothing landed.
    for n in ["G1", A, "E1"] {
        let e = model::find_by_name(&w, n).expect("tool");
        assert!(assembly_of(&w, &e).is_none() && body_of(&w, &e).is_none(), "{n} is bare");
    }
}

pub(super) fn reopen(text: &str) -> World {
    let w = camcraft_app::world::new_world();
    camcraft_app::project::open(&w, text).expect("open");
    w
}

#[test]
fn a_linked_turning_stack_with_its_lathe_body_reopens_identically() {
    let (w, _h) = lathe_shop();
    let text = camcraft_app::project::save_text(&w).expect("save");
    let file: serde_json::Value = serde_json::from_str(&text).expect("json");
    let row = |id: &str| file["components"].as_array().and_then(|r| r.iter().find(|c| c["id"] == id)).cloned();
    let facets = row(HOLDER_M).and_then(|r| r["lathe_body"]["facets"].as_array().map(Vec::len));
    assert_eq!(facets, Some(24), "two boxes, twelve facets each: {:?}", row(HOLDER_M));
    assert!(row(INSERT).is_some_and(|r| r.get("lathe_body").is_none()), "an insert row writes no lathe body");

    let w2 = reopen(&text);
    assert_eq!(snapshot(&w2), snapshot(&w), "the rows, the lathe bodies, the Turning records and the bodies");
    assert_eq!(camcraft_app::project::save_text(&w2).expect("save"), text, "save -> open -> save is byte-stable");

    // The reopened rows derive again: the same edit gives the same result.
    let mut h1 = History::default();
    let mut h2 = History::default();
    h1.execute(&w, EditDocumentComponent::with_row(&w, insert("CNMG120404")).expect("edit"));
    h2.execute(&w2, EditDocumentComponent::with_row(&w2, insert("CNMG120404")).expect("edit"));
    assert_eq!(snapshot(&w2), snapshot(&w), "the reopened lathe body derives the same records and bodies");
}

#[test]
fn a_corrupt_lathe_body_refuses_the_file_and_a_library_save_refuses_the_row() {
    let (w, _h) = lathe_shop();
    let text = camcraft_app::project::save_text(&w).expect("save");
    let refused = |f: &dyn Fn(&mut serde_json::Value)| {
        let mut v: serde_json::Value = serde_json::from_str(&text).expect("json");
        if let Some(r) = v["components"].as_array_mut().and_then(|r| r.iter_mut().find(|c| c["id"] == HOLDER_M)) {
            f(&mut r["lathe_body"]);
        }
        let w = camcraft_app::world::new_world();
        let Err(e) = camcraft_app::project::open(&w, &v.to_string()) else { panic!("a corrupt lathe body opened") };
        assert!(components(&w).is_empty(), "no row landed");
        e.to_string()
    };
    let e = refused(&|b| b["facets"] = serde_json::json!([]));
    assert!(e.contains(HOLDER_M) && e.contains("no facets"), "{e}");
    let e = refused(&|b| b["facets"][0][0][0] = serde_json::json!("x"));
    assert!(!e.is_empty(), "a facet that is not a number: {e}");
    let e = refused(&|b| b["pad"] = serde_json::json!(-0.5));
    assert!(e.contains(HOLDER_M) && e.contains("lathe body"), "a negative pad: {e}");
    let e = refused(&|b| b["boxes"] = serde_json::json!([]));
    assert!(e.contains("boxes"), "an unknown key: {e}");

    let Err(e) = camcraft_app::component_library::LibraryComponent::from_row(holder_m()) else {
        panic!("a library save took a lathe body")
    };
    assert!(e.contains(HOLDER_M) && e.contains("lathe body"), "{e}");
}

/// #3643: a meshed body's pad saves with its row and opens again, so a
/// save and an open never shrink the bands to the bare facets.
#[test]
fn a_padded_lathe_body_keeps_its_pad_through_save_and_open() {
    let w = camcraft_app::world::new_world();
    let mut h = History::default();
    upsert(&w, &mut h, holder_m_padded(0.05));
    let text = camcraft_app::project::save_text(&w).expect("save");
    let file: serde_json::Value = serde_json::from_str(&text).expect("json");
    let row = file["components"].as_array().and_then(|r| r.iter().find(|c| c["id"] == HOLDER_M)).cloned();
    assert_eq!(row.as_ref().and_then(|r| r["lathe_body"]["pad"].as_f64()), Some(0.05), "{row:?}");
    let w2 = reopen(&text);
    assert_eq!(components(&w2), components(&w), "the row and its padded body");
    assert_eq!(camcraft_app::project::save_text(&w2).expect("save"), text, "save -> open -> save is byte-stable");
}
