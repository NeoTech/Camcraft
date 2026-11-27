//! #3646: multi-seat indexable cutters in the document. A 4-seat Ø50
//! face mill (not centre-cutting) and a 2-seat Ø25 shoulder end mill
//! share one SCMT12 insert on one BT40 holder. Pinned: each tool takes
//! its whole milling record from its body row and the insert row (the
//! flutes, the flute length, centre cutting, the tool material and the
//! cutting rows, not only the diameter and the corner); one edit of the
//! shared insert updates both tools in one undo step and their
//! operations go Stale, and undo and redo restore the exact state; an
//! edit that breaks a seat is refused by the tool's name and changes
//! nothing; the face mill is refused a plunge into solid. Provenance:
//! our own tests over our own commands; standard entry practice (a
//! cutter without centre teeth never plunges into solid).

use camcraft_app::cl::Toolpath;
use camcraft_app::commands::{AddTool, CreateOperation, EditDocumentComponent, GenerateOperation, History, SetStock};
use camcraft_app::commands::{SetToolAssembly, UpsertDocumentComponent};
use camcraft_app::drilling::Hole;
use camcraft_app::geo2d::Point2;
use camcraft_app::model;
use camcraft_app::operation::{self, Height, OpParams, OpStatus, SrcRef, status_of};
use camcraft_app::schema::{AssemblyLink, DocumentComponent, InsertRunning, Location, Operation, ToolAssembly, ToolBody};
use camcraft_app::tool_assembly::{assembly_of, component, components};
use camcraft_app::tool_body::body_of;
use camcraft_app::tools::{Tool, ToolKind};
use camcraft_motion::approach::CENTER_CUT_REFUSAL;
use camcraft_tools::component::{ComponentKind, CuttingItem, InsertSeat, ToolComponent};
use camcraft_tools::component_geometry::{ComponentBody, ComponentGeometry};
use camcraft_tools::connection_code::ConnectionCode;
use camcraft_tools::cutting::{ToolMaterial, WorkMaterial};
use camcraft_tools::cutting_rows::CuttingRow;
use camcraft_tools::frame::Frame;
use camcraft_tools::insert_iso1832::InsertSpec;
use camcraft_tools::insert_seat::HolderDesignation;
use camcraft_tools::milling_cutter::MillingBody;
use camcraft_tools::spun::SpunPoint;
use camcraft_tools::tool_assembly::Mount;
use flecs_ecs::prelude::World;

use super::cat;

const EM: &str = "EndMill25";
const FM: &str = "FaceMill50";
const INSERT: &str = "insert-scmt12";
const SEAT: &str = "SSDCN2525M12";

fn code(text: &str) -> ConnectionCode {
    ConnectionCode::parse(text).expect("connection code")
}

fn cyl(len: f64, r: f64) -> ComponentBody {
    let p = SpunPoint::line;
    ComponentBody::new(vec![p(0.0, 0.0), p(0.0, r), p(len, r), p(len, 0.0)], -len).expect("body")
}

fn holder() -> DocumentComponent {
    let kind = ComponentKind::AdaptiveItem { mcs: code("BT40"), csw: code("CYL 20") };
    let c = ToolComponent::new("bt40-cyl20", "BT40 CYL 20", kind).expect("holder");
    DocumentComponent::new(c, ComponentGeometry::axial("CSW", 50.0, Some(cyl(50.0, 25.0))).expect("geometry"))
}

/// A cutter body `id` with `n` radial seats on Ø`dc`, 45 mm below its
/// MCS; its own body a Ø`2 body_r` cylinder 36 mm long (above the
/// flutes); its catalog facts `DC`, `KAPR` 90, `APMX` 8.
fn body(id: &str, n: usize, dc: f64, body_r: f64, center_cutting: bool) -> DocumentComponent {
    let seat = HolderDesignation::parse(SEAT).expect("holder code").seat;
    let ports: Vec<String> = (1..=n).map(|i| format!("CSW1_{i}")).collect();
    let seats = ports.iter().map(|p| InsertSeat { port: p.clone(), spec: seat }).collect();
    let kind = ComponentKind::ToolItem { mcs: code("CYL 20"), seats };
    let step = 360.0 / n as f64;
    let frames = ports
        .iter()
        .enumerate()
        .map(|(i, p)| {
            let (s, c) = (step * i as f64).to_radians().sin_cos();
            (p.clone(), Frame::new([0.5 * dc * c, 0.5 * dc * s, -45.0], [0.0, 0.0, 1.0], [c, s, 0.0]).expect("seat frame"))
        })
        .collect();
    let g = ComponentGeometry::new(frames, Some(cyl(36.0, body_r)), None).expect("geometry");
    let milling = MillingBody { dc, kapr_deg: 90.0, apmx: 8.0, center_cutting, seat_count: None };
    DocumentComponent { milling: Some(milling), ..DocumentComponent::new(ToolComponent::new(id, id, kind).expect("body"), g) }
}

fn rows(rpm: f64) -> Vec<CuttingRow> {
    vec![CuttingRow::plain(Some(WorkMaterial::Aluminium6061), rpm, 2400.0)]
}

/// The shared insert with its own tool material and cutting rows.
fn insert(text: &str, rpm: f64) -> DocumentComponent {
    let spec = InsertSpec::parse(text).expect("insert code");
    let c = ToolComponent::new(INSERT, text, ComponentKind::CuttingItem(CuttingItem::Insert(spec))).expect("insert");
    let running = InsertRunning { tool_material: Some(ToolMaterial::Carbide), cutting: rows(rpm) };
    DocumentComponent { running: Some(running), ..DocumentComponent::new(c, ComponentGeometry::default()) }
}

fn stack(body: &str) -> Vec<AssemblyLink> {
    let on = |parent: usize, port: Option<&str>| {
        Some(Mount { parent, port: port.map(str::to_owned), z_rot_deg: 0.0, projection_mm: None })
    };
    vec![
        AssemblyLink { component: "bt40-cyl20".into(), mount: None },
        AssemblyLink { component: body.into(), mount: on(0, None) },
        AssemblyLink { component: INSERT.into(), mount: on(1, Some("CSW1_1")) },
    ]
}

/// What the assembly overwrites: a plain end mill record.
fn bare() -> Tool {
    Tool { tool_material: None, cutting: Vec::new(), ..cat("EM-1200-2FL") }
}

fn face() -> OpParams {
    OpParams::Face {
        holes: Default::default(),
        tilt: None,
        blend_turns: false,
        blend_radius: None,
        stepover: 10.0,
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

/// A 120 mm stock block, the four rows, both tools on their stacks, and
/// a generated Face operation on each.
fn shop() -> (World, History) {
    let w = camcraft_app::world::new_world();
    let mut h = History::default();
    let stock = camcraft_app::stock::StockShape::Box { dx: 120.0, dy: 120.0, dz: 20.0 };
    h.execute(&w, SetStock::new(&w, stock, Location::default()).expect("stock"));
    let rows = [holder(), body("face-50", 4, 50.0, 20.0, false), body("shoulder-25", 2, 25.0, 10.0, true), insert("SCMT120408", 9000.0)];
    for row in rows {
        h.execute(&w, UpsertDocumentComponent::with_row(&w, row).expect("upsert"));
    }
    for (tool, body) in [(EM, "shoulder-25"), (FM, "face-50")] {
        h.execute(&w, AddTool::new(&w, tool, bare()).expect("tool"));
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
    (components(w), vec![tool(EM), tool(FM)])
}

fn record(w: &World, name: &str) -> Tool {
    model::tool_of(w, &model::find_by_name(w, name).expect("tool")).expect("a tool")
}

fn op_status(w: &World, tool: &str) -> OpStatus {
    status_of(w, &model::find_by_name(w, &format!("Face{tool}")).expect("op"))
}

#[test]
fn a_multi_seat_cutter_takes_its_whole_record_from_the_document() {
    let (w, _h) = shop();
    let near = |a: f64, b: f64| (a - b).abs() < 1e-9;
    let f = record(&w, FM);
    assert_eq!((f.kind, f.flutes, f.center_cutting), (ToolKind::BullNose, Some(4), false), "{f:?}");
    assert!(near(f.diameter, 50.0) && near(f.corner_r, 0.8) && near(f.flute_len, 8.0), "{f:?}");
    let e = record(&w, EM);
    assert_eq!((e.kind, e.flutes, e.center_cutting), (ToolKind::BullNose, Some(2), true), "{e:?}");
    assert!(near(e.diameter, 25.0) && near(e.corner_r, 0.8) && near(e.flute_len, 8.0), "{e:?}");
    for t in [&f, &e] {
        assert_eq!(t.tool_material, Some(ToolMaterial::Carbide), "the insert's tool material");
        assert_eq!(t.cutting, rows(9000.0), "the insert's cutting rows");
    }
    assert!(f.shank_d >= 40.0 && f.holder_d >= 50.0, "the holder is enclosed over the face mill: {f:?}");
    assert_ne!(op_status(&w, FM), OpStatus::Stale, "generated after the assembly");
}

#[test]
fn a_shared_insert_edit_updates_both_multi_seat_tools_in_one_undo_step() {
    let (w, mut h) = shop();
    let before = snapshot(&w);
    let edit = EditDocumentComponent::with_row(&w, insert("SCMT120404", 7000.0)).expect("an insert both bodies take");
    assert_eq!(edit.updated_tools(), vec![EM.to_owned(), FM.to_owned()]);
    h.execute(&w, edit);
    let after = snapshot(&w);
    for (t, flutes) in after.1.iter().zip([2, 4]) {
        assert!((t.0.corner_r - 0.4).abs() < 1e-12, "the new nose radius: {}", t.0.corner_r);
        assert_eq!((t.0.flutes, t.0.cutting.clone()), (Some(flutes), rows(7000.0)), "the new cutting rows");
    }
    assert_eq!(op_status(&w, EM), OpStatus::Stale, "the end mill operation re-marks");
    assert_eq!(op_status(&w, FM), OpStatus::Stale, "the face mill operation re-marks");

    assert!(h.undo(&w), "one undo step");
    assert_eq!(snapshot(&w), before, "undo restores the insert, both records and both bodies");
    assert_ne!(op_status(&w, FM), OpStatus::Stale);
    assert!(h.redo(&w));
    assert_eq!(snapshot(&w), after, "redo lands the same edit");

    // A catalog-fact edit (`new`) keeps the row's running data.
    let held = component(&w, INSERT).expect("row");
    let spec = InsertSpec::parse("SCMT120408").expect("insert code");
    let c = ToolComponent::new(INSERT, "SCMT120408", ComponentKind::CuttingItem(CuttingItem::Insert(spec))).expect("insert");
    h.execute(&w, EditDocumentComponent::new(&w, c, ComponentGeometry::default()).expect("edit"));
    assert_eq!(component(&w, INSERT).expect("row").running, held.running, "the running data stays");
    assert_eq!(record(&w, FM).cutting, rows(7000.0));
}

#[test]
fn an_edit_that_breaks_a_seat_is_refused_by_the_tool_name() {
    let (w, h) = shop();
    let before = snapshot(&w);
    let depth = h.can_undo();
    // A D insert in the S seats of both bodies.
    let Err(e) = EditDocumentComponent::with_row(&w, insert("DCMT11T304", 9000.0)) else { panic!("a D insert in an S seat") };
    let msg = e.to_string();
    assert!(msg.contains(INSERT) && msg.contains(EM) && msg.contains("does not fit"), "{msg}");
    // The face mill body restated on Ø48: its seats stay on Ø50.
    let mut off = body("face-50", 4, 50.0, 20.0, false);
    off.milling = off.milling.map(|m| MillingBody { dc: 48.0, ..m });
    let Err(e) = EditDocumentComponent::with_row(&w, off) else { panic!("DC off the seats") };
    let msg = e.to_string();
    assert!(msg.contains("face-50") && msg.contains(FM), "{msg}");
    // Cutter facts on an insert, running data on a body.
    let wrong = DocumentComponent { milling: body("x", 4, 50.0, 20.0, false).milling, ..insert("SCMT120408", 9000.0) };
    assert!(EditDocumentComponent::with_row(&w, wrong).is_err_and(|e| e.to_string().contains("not a cutter body")));
    let wrong = DocumentComponent { running: Some(InsertRunning::default()), ..body("face-50", 4, 50.0, 20.0, false) };
    assert!(EditDocumentComponent::with_row(&w, wrong).is_err_and(|e| e.to_string().contains("not an insert")));
    assert_eq!(snapshot(&w), before, "nothing changed");
    assert_eq!(h.can_undo(), depth);
}

fn bore(w: &World, tool: &str, radius: f64) -> Result<Toolpath, String> {
    let params = OpParams::Bore {
        entry: camcraft_app::machining3d::Entry3D::Plunge,
        pass_start: camcraft_app::machining2d::PassStart::Auto,
        break_through: None,
        links: camcraft_app::links::Links::PLANAR,
        clearance_z: Height::fixed(25.0),
        retract_z: None,
        top_z: Height::fixed(20.0),
        bottom_z: None,
        stock_to_leave: 0.0,
        stepdown: 5.0,
        feed: 500.0,
        plunge_feed: 150.0,
        climb: true,
        tolerance: 0.01,
        holes: vec![Hole { xy: Point2::new(60.0, 60.0), radius, entry_z: 20.0, floor_z: 15.0, through: false, stepped: false }],
        picks: Default::default(),
    };
    let op = Operation {
        tool: tool.into(),
        src: SrcRef::Stock,
        params,
        stamp: None,
        setup: None,
        area: Default::default(),
        coolant: Default::default(),
        after: Default::default(),
        suppressed: false,
    };
    operation::generate(w, &op, "B").map(|(p, _)| p).map_err(|e| e.to_string())
}

#[test]
fn the_face_mill_is_refused_a_plunge_into_solid() {
    let (w, _h) = shop();
    // Ø50 in an undrilled Ø51 hole: no helix fits and no ramp is long
    // enough, and the face mill has no centre teeth.
    let msg = bore(&w, FM, 25.5).err().unwrap_or_default();
    assert!(msg.contains(CENTER_CUT_REFUSAL), "refused by the centre-cutting law: {msg}");
    // The centre-cutting end mill plunges into its own tight hole.
    assert!(bore(&w, EM, 13.0).is_ok());
}
