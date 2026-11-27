//! #3643: a turning tool assembled from linked document components (an
//! ISO 5608 holder and an ISO 1832 insert) through the document
//! commands. Pinned: a CNMG in a PCLNR gives the same Turning record and
//! the same lathe body as the tools-crate resolver; one edit of the
//! shared insert updates two holder assemblies in one undo step, their
//! turning operations go Stale, and undo and redo are exact; a kind,
//! a root or a stack that the lathe resolver does not take is refused
//! by name and changes nothing. Provenance: our own tests over our own
//! commands.

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

const A: &str = "OD-M";
const B: &str = "OD-K";
const INSERT: &str = "cnmg";
const HOLDER_M: &str = "pclnr-2525m";
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
fn holder_m() -> DocumentComponent {
    holder(HOLDER_M, "PCLNR2525M12", [12.5 - 32.0, 0.0, -150.0])
}

/// 2020K: l1 = 125 mm, f1 = 25 mm.
fn holder_k() -> DocumentComponent {
    holder(HOLDER_K, "PCLNR2020K12", [10.0 - 25.0, 0.0, -125.0])
}

fn insert(code: &str) -> DocumentComponent {
    let spec = InsertSpec::parse(code).expect("insert code");
    let c = ToolComponent::new(INSERT, code, ComponentKind::CuttingItem(CuttingItem::Insert(spec))).expect("insert");
    DocumentComponent::new(c, ComponentGeometry::default())
}

fn on(parent: usize) -> Option<Mount> {
    Some(Mount { parent, port: None, z_rot_deg: 0.0, projection_mm: None })
}

fn stack(holder: &str) -> Vec<AssemblyLink> {
    vec![AssemblyLink { component: holder.into(), mount: None }, AssemblyLink { component: INSERT.into(), mount: on(0) }]
}

fn turning() -> Tool {
    cat("TN-1600-0R8")
}

fn upsert(w: &World, h: &mut History, row: DocumentComponent) {
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
fn lathe_shop() -> (World, History) {
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
fn a_linked_cnmg_in_a_pclnr_equals_the_tools_crate_result() {
    let w = camcraft_app::world::new_world();
    let mut h = History::default();
    for row in [holder_m(), insert("CNMG120408")] {
        upsert(&w, &mut h, row);
    }
    h.execute(&w, AddTool::new(&w, A, turning()).expect("tool"));
    h.execute(&w, SetToolAssembly::new(&w, A, stack(HOLDER_M)).expect("assembly"));
    let e = model::find_by_name(&w, A).expect("tool");
    let (tool, body) = (model::tool_of(&w, &e).expect("a tool"), body_of(&w, &e).expect("a lathe body"));

    // The tools-crate result for the same two components.
    let (m, i) = (holder_m(), insert("CNMG120408"));
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
    let BodyFrame::Lathe { place, zone, clearance } = &body.frame else { panic!("a lathe-frame body") };
    assert_eq!(*place, r.placement, "the nose centre is the tip");
    assert!(zone.is_none(), "the insert plate is left out by the resolver, not by a zone");
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
