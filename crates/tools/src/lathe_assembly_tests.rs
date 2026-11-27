//! #3643: lathe tool assemblies resolved to the nose-centre tip, the
//! orientation, the turning values and the clearance bands - pinned
//! against today's placed-mesh lathe body and the tool record, stacked
//! in a sleeve, and refused by name.

use crate::component::{ComponentKind, CuttingItem, InsertSeat, ToolComponent};
use crate::component_fit::FitProperty;
use crate::component_geometry::{ComponentBody, ComponentGeometry};
use crate::connection_code::ConnectionCode;
use crate::frame::Frame;
use crate::insert_codes::Hand;
use crate::insert_iso1832::InsertSpec;
use crate::insert_seat::{Clamping, HolderDesignation, HolderStyle, InsertSeatSpec};
use crate::lathe_assembly::{
    BodyBox, LatheAssemblyError, LatheBody, LatheNode, LatheStation, ResolvedLathe, resolve_lathe,
};
use crate::lathe_holder::{LatheBodyError, LatheClearance, LathePlacement};
use crate::spun::SpunPoint;
use crate::tool_assembly::{AssemblyError, AssemblyNode, Mount};
use crate::tools::{Tool, ToolKind, catalog};

fn made(id: &str, kind: ComponentKind) -> ToolComponent {
    ToolComponent::new(id, id, kind).unwrap_or_else(|e| panic!("{id}: {e}"))
}

fn insert(code: &str) -> ToolComponent {
    let spec = InsertSpec::parse(code).unwrap_or_else(|e| panic!("{code}: {e}"));
    made(code, ComponentKind::CuttingItem(CuttingItem::Insert(spec)))
}

/// An ISO 5608 holder on its rectangular shank, one seat `CSW1`.
fn holder(code: &str) -> ToolComponent {
    let h = HolderDesignation::parse(code).unwrap_or_else(|e| panic!("{code}: {e}"));
    let mcs = ConnectionCode::RectShank { h: u16::from(h.shank_h), b: u16::from(h.shank_b) };
    made(code, ComponentKind::ToolItem { mcs, seats: vec![InsertSeat { port: "CSW1".into(), spec: h.seat }] })
}

fn bx(lo: [f64; 3], hi: [f64; 3]) -> BodyBox {
    BodyBox { lo, hi }
}

fn body(boxes: &[BodyBox]) -> LatheBody {
    LatheBody::boxes(boxes).unwrap_or_else(|e| panic!("{e}"))
}

/// The CRP of the 2525M holders below, in the holder frame: tool length
/// l1 = 150 mm (M), f1 = 32 mm from the shank's +X side face.
const CRP: [f64; 3] = [12.5 - 32.0, 0.0, -150.0];

/// A 2525M holder: the MCS on the shank's back face, +Z out of the
/// back, +X along the spindle away from the chuck; the shank below
/// centre height, and a head set 1 mm back from the CRP, under the
/// insert (5 mm below its top face).
fn holder_boxes() -> Vec<BodyBox> {
    vec![bx([-12.5, -25.0, -125.0], [12.5, 0.0, 0.0]), bx([CRP[0] + 1.0, -25.0, -149.0], [12.5, -5.0, -125.0])]
}

fn seat_at(crp: [f64; 3]) -> ComponentGeometry {
    let f = Frame::new(crp, [0.0, 0.0, 1.0], [1.0, 0.0, 0.0]).unwrap_or_else(|e| panic!("{e}"));
    ComponentGeometry::new(vec![("CSW1".into(), f)], None, None).unwrap_or_else(|e| panic!("{e}"))
}

fn node(
    component: ToolComponent,
    geometry: ComponentGeometry,
    mount: Option<Mount>,
    body: Option<LatheBody>,
) -> LatheNode {
    LatheNode { node: AssemblyNode { component, geometry, mount }, body }
}

fn on(parent: usize) -> Option<Mount> {
    Some(Mount { parent, port: None, z_rot_deg: 0.0, projection_mm: None })
}

/// `holder_code` with `insert_code` in its seat at [`CRP`].
fn turning(holder_code: &str, insert_code: &str) -> Vec<LatheNode> {
    vec![
        node(holder(holder_code), seat_at(CRP), None, Some(body(&holder_boxes()))),
        node(insert(insert_code), ComponentGeometry::default(), on(0), None),
    ]
}

fn ok(nodes: &[LatheNode], station: LatheStation, q: Option<u8>) -> ResolvedLathe {
    resolve_lathe(nodes, station, q).unwrap_or_else(|e| panic!("refused: {e}"))
}

fn refused(nodes: &[LatheNode], station: LatheStation, q: Option<u8>) -> LatheAssemblyError {
    match resolve_lathe(nodes, station, q) {
        Ok(r) => panic!("accepted: orientation {}", r.params.orientation),
        Err(e) => e,
    }
}

fn near(a: [f64; 3], b: [f64; 3]) -> bool {
    (0..3).all(|k| (a[k] - b[k]).abs() < 1e-9)
}

/// Every corner of `boxes` (holder frame) inside the bands, placed by
/// `place`: the envelope holds the holder.
fn assert_encloses(clear: &LatheClearance, place: &LathePlacement, boxes: &[BodyBox]) {
    for b in boxes {
        for i in 0..8 {
            let c = [
                if i & 1 == 0 { b.lo[0] } else { b.hi[0] },
                if i & 2 == 0 { b.lo[1] } else { b.hi[1] },
                if i & 4 == 0 { b.lo[2] } else { b.hi[2] },
            ];
            let tri = [c, c, c];
            let p = place.place(&[tri]).unwrap_or_else(|e| panic!("{e}"))[0][0];
            let r = clear.over(p[2], p[2]).unwrap_or_else(|| panic!("no band at z {}", p[2]));
            assert!(r.x_lo <= p[0] && p[0] <= r.x_hi, "x {} outside {r:?}", p[0]);
            assert!(p[1].abs() <= r.y_hi, "|y| {} outside {r:?}", p[1]);
        }
    }
}

#[test]
fn a_cnmg_in_a_pclnr_has_its_nose_centre_at_the_tip() {
    let r = ok(&turning("PCLNR2525M12", "CNMG120408"), LatheStation::RADIAL_SHANK, None);
    let p = r.params;
    assert_eq!((p.orientation, p.stated, p.hand), (2, false, Hand::R));
    assert!((p.nose_radius - 0.8).abs() < 1e-12, "{}", p.nose_radius);
    assert_eq!(p.approach_deg, Some(95.0));
    assert_eq!(p.insert_angle_deg, Some(80.0));
    assert_eq!(p.clearance_deg, Some(0.0));
    assert_eq!(p.ic_mm, Some(12.7));
    // Orientation 2: the control point is at -X, -Z of the nose centre,
    // so the centre is 0.8 mm up the radial (root +Z) and 0.8 mm along
    // the spindle (root +X) from the CRP.
    assert!(near(r.placement.tip, [CRP[0] + 0.8, 0.0, CRP[2] + 0.8]), "{:?}", r.placement.tip);
    assert!(near(r.crp, [-0.8, 0.0, -0.8]), "{:?}", r.crp);
    let c = r.placement.place(&[[CRP, CRP, CRP]]).unwrap_or_else(|e| panic!("{e}"))[0][0];
    assert!(near(c, r.crp), "the CRP maps to {c:?}");
}

#[test]
fn the_derived_values_equal_todays_tool_record_and_placed_mesh_body() {
    let r = ok(&turning("PCLNR2525M12", "CNMG120408"), LatheStation::RADIAL_SHANK, None);
    // The tool record a user types today for the same insert.
    let base = catalog().into_iter().map(|(_, t)| t).next().unwrap_or_else(|| panic!("empty catalog"));
    let tool = Tool { kind: ToolKind::Turning, nose_radius: 0.8, orientation: None, ..base };
    assert_eq!(tool.lathe_orientation(), Some(r.params.orientation));
    assert!((tool.nose_radius - r.params.nose_radius).abs() < 1e-12);
    // The same holder as a mesh placed by hand on today's lathe body
    // path: the nose centre typed as the tip.
    let tris: Vec<_> = body(&holder_boxes()).facets().to_vec();
    let place =
        LathePlacement { radial: [0.0, 0.0, 1.0], along: [1.0, 0.0, 0.0], tip: [CRP[0] + 0.8, 0.0, CRP[2] + 0.8] };
    let today =
        LatheClearance::of(&place.place(&tris).unwrap_or_else(|e| panic!("{e}"))).unwrap_or_else(|e| panic!("{e}"));
    let got = r.clearance.unwrap_or_else(|| panic!("no clearance"));
    let (a, b): (Vec<f64>, Vec<f64>) = (got.words().collect(), today.words().collect());
    assert_eq!(a.len(), b.len());
    assert!(a.iter().zip(&b).all(|(x, y)| (x - y).abs() < 1e-9), "{a:?} vs {b:?}");
}

#[test]
fn the_clearance_envelope_encloses_the_holder() {
    let r = ok(&turning("PCLNR2525M12", "CNMG120408"), LatheStation::RADIAL_SHANK, None);
    let clear = r.clearance.unwrap_or_else(|| panic!("no clearance"));
    assert_encloses(&clear, &r.placement, &holder_boxes());
    // The shank reaches 149.2 mm out from the nose, the head starts
    // 0.2 mm behind it.
    let span = clear.spans();
    let x_hi = span.iter().map(|s| s.2.x_hi).fold(f64::NEG_INFINITY, f64::max);
    assert!((x_hi - 149.2).abs() < 1e-5, "{x_hi}");
    assert!((clear.z_lo() - 0.2).abs() < 1e-9, "{}", clear.z_lo());
}

#[test]
fn the_insert_plate_is_left_out_of_the_bands() {
    let bare = ok(&turning("PCLNR2525M12", "CNMG120408"), LatheStation::RADIAL_SHANK, None);
    let mut nodes = turning("PCLNR2525M12", "CNMG120408");
    nodes[1].body = Some(body(&[bx([-12.0, -1.0, -1.0], [1.0, 4.76, 12.0])]));
    let plated = ok(&nodes, LatheStation::RADIAL_SHANK, None);
    assert_eq!(bare.clearance, plated.clearance);
}

#[test]
fn hand_and_shape_give_the_orientation_and_a_stated_one_wins() {
    // Left hand: cuts away from the chuck, so the seat is on the +X
    // side and the head with it.
    let mirror = |b: &BodyBox| bx([-b.hi[0], b.lo[1], b.lo[2]], [-b.lo[0], b.hi[1], b.hi[2]]);
    let left: Vec<BodyBox> = holder_boxes().iter().map(mirror).collect();
    let crp = [-CRP[0], 0.0, CRP[2]];
    let nodes = vec![
        node(holder("PCLNL2525M12"), seat_at(crp), None, Some(body(&left))),
        node(insert("CNMG120408"), ComponentGeometry::default(), on(0), None),
    ];
    let r = ok(&nodes, LatheStation::RADIAL_SHANK, None);
    assert_eq!(r.params.orientation, 1);
    assert!(near(r.placement.tip, [crp[0] - 0.8, 0.0, crp[2] + 0.8]), "{:?}", r.placement.tip);
    // A round insert in a neutral holder points -X: orientation 6.
    let mut round = turning("SRDCN2525M12", "RCMT1204M0");
    round[0].body = Some(body(&[bx([-12.5, -25.0, -125.0], [12.5, 0.0, 0.0])]));
    let r = ok(&round, LatheStation::RADIAL_SHANK, None);
    assert_eq!((r.params.orientation, r.params.nose_radius, r.params.insert_angle_deg), (6, 6.0, None));
    assert!(near(r.placement.tip, [CRP[0], 0.0, CRP[2] + 6.0]), "{:?}", r.placement.tip);
    // A stated orientation wins and is reported as stated.
    let r = ok(&round, LatheStation::RADIAL_SHANK, Some(6));
    assert!(r.params.stated);
}

/// A VDI sleeve (a body 60 mm deep, 30 mm about its axis) with a
/// 25 mm boring bar 200 mm long and its insert 17 mm off the bar axis.
fn bar_in_sleeve(z_rot_deg: f64) -> Vec<LatheNode> {
    let sleeve = made(
        "VDI40 sleeve",
        ComponentKind::AdaptiveItem {
            mcs: ConnectionCode::Vdi { d: 40 },
            csw: ConnectionCode::Cylindrical { d: 25.0 },
        },
    );
    let seat = InsertSeatSpec {
        clamping: Clamping::P,
        shape: crate::insert_codes::InsertShape::C,
        style: HolderStyle::from_letter('L').unwrap_or_else(|| panic!("style")),
        clearance: crate::insert_codes::Clearance::N,
        hand: Hand::R,
        size: 12,
        thickness: None,
    };
    let bar = made(
        "S25 bar",
        ComponentKind::ToolItem {
            mcs: ConnectionCode::Cylindrical { d: 25.0 },
            seats: vec![InsertSeat { port: "CSW1".into(), spec: seat }],
        },
    );
    let sleeve_geo = ComponentGeometry::axial("CSW", 60.0, None).unwrap_or_else(|e| panic!("{e}"));
    let mount = Some(Mount { z_rot_deg, ..on(0).unwrap_or_else(|| unreachable!()) });
    vec![
        node(sleeve, sleeve_geo, None, Some(body(&[bx([-30.0, -30.0, -60.0], [30.0, 30.0, 0.0])]))),
        node(bar, seat_at([17.0, 0.0, -201.0]), mount, Some(body(&[bx([-12.5, -12.5, -200.0], [12.5, 12.5, 0.0])]))),
        node(insert("CNMG120408"), ComponentGeometry::default(), on(1), None),
    ]
}

#[test]
fn a_boring_bar_in_a_sleeve_is_judged_as_one_body() {
    let nodes = bar_in_sleeve(0.0);
    // A VDI root says nothing of OD or ID: the orientation is stated.
    assert!(matches!(
        refused(&nodes, LatheStation::AXIAL_SHANK, None),
        LatheAssemblyError::NeedOrientation { ref name, .. } if name == "VDI40 sleeve"
    ));
    let r = ok(&nodes, LatheStation::AXIAL_SHANK, Some(3));
    // The bar's MCS sits 60 mm down the sleeve; orientation 3 puts the
    // nose centre 0.8 mm in from the CRP and 0.8 mm away from the chuck.
    assert!(near(r.placement.tip, [16.2, 0.0, -260.2]), "{:?}", r.placement.tip);
    let clear = r.clearance.unwrap_or_else(|| panic!("no clearance"));
    assert!((clear.z_hi() - 260.2).abs() < 1e-6, "the sleeve is in the bands: {}", clear.z_hi());
    // The sleeve alone stands 200 mm from the tip: the tip gap is read
    // on the whole stack, never on the sleeve by itself.
    let sleeve = body(&[bx([-30.0, -30.0, -60.0], [30.0, 30.0, 0.0])]);
    let alone = r.placement.place(sleeve.facets()).unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(LatheClearance::of(&alone), Err(LatheBodyError::TipAway));
    // Turned half a turn, the insert stands behind the bar.
    assert!(matches!(
        refused(&bar_in_sleeve(180.0), LatheStation::AXIAL_SHANK, Some(3)),
        LatheAssemblyError::BodyPastEdge { ref name, axis: 'X', .. } if name == "S25 bar"
    ));
}

#[test]
fn a_mismatch_a_wrong_hand_and_degenerate_input_are_refused_by_name() {
    let st = LatheStation::RADIAL_SHANK;
    // The #3642 fit check names the size.
    match refused(&turning("PCLNR2525M12", "CNMG090308"), st, None) {
        LatheAssemblyError::Assembly(AssemblyError::Fit(e)) => assert_eq!(e.property, FitProperty::Size),
        e => panic!("{e}"),
    }
    // A left-hand code on a right-hand geometry: the head stands ahead
    // of the cutting line in Z.
    let wrong = turning("PCLNL2525M12", "CNMG120408");
    assert!(matches!(refused(&wrong, st, None), LatheAssemblyError::BodyPastEdge { axis: 'Z', .. }));
    // A stated orientation that disagrees with the geometry too.
    let r = refused(&turning("PCLNR2525M12", "CNMG120408"), st, Some(1));
    assert!(r.to_string().contains("check the hand"), "{r}");
    for q in [0, 9, 10] {
        assert_eq!(refused(&turning("PCLNR2525M12", "CNMG120408"), st, Some(q)), LatheAssemblyError::Orientation { q });
    }
    assert!(matches!(refused(&turning("PCLNR2525M12", "CNMG120400"), st, None), LatheAssemblyError::NoNose { .. }));
    let bad = LatheStation { radial: [0.0; 3], ..st };
    assert_eq!(
        refused(&turning("PCLNR2525M12", "CNMG120408"), bad, None),
        LatheAssemblyError::Station(LatheBodyError::Direction)
    );
    let skew = LatheStation { along: [1.0, 0.0, 0.5], ..st };
    assert_eq!(
        refused(&turning("PCLNR2525M12", "CNMG120408"), skew, None),
        LatheAssemblyError::Station(LatheBodyError::NotSquare)
    );
    // A revolved body is never dropped in silence.
    let mut spun = turning("PCLNR2525M12", "CNMG120408");
    let p = SpunPoint::line;
    let outline = ComponentBody::new(vec![p(0.0, 0.0), p(0.0, 5.0), p(10.0, 5.0), p(10.0, 0.0)], -10.0)
        .unwrap_or_else(|e| panic!("{e}"));
    spun[0].node.geometry = ComponentGeometry::new(vec![("CSW1".into(), Frame::IDENTITY)], Some(outline), None)
        .unwrap_or_else(|e| panic!("{e}"));
    assert!(matches!(refused(&spun, st, None), LatheAssemblyError::SpunBody { ref name } if name == "PCLNR2525M12"));
    // A lone end mill is no lathe assembly.
    let base = catalog()
        .into_iter()
        .map(|(_, t)| t)
        .find(|t| t.kind == ToolKind::EndMill)
        .unwrap_or_else(|| panic!("no end mill"));
    let mcs = ConnectionCode::Cylindrical { d: base.shank_d };
    let mill = made("mill", ComponentKind::CuttingItem(CuttingItem::Solid { tool: Box::new(base), mcs }));
    let lone = vec![node(mill, ComponentGeometry::default(), None, None)];
    assert!(matches!(refused(&lone, st, None), LatheAssemblyError::NotInsert { .. }));
    // A body far from the nose is refused on the whole stack.
    let mut far = turning("PCLNR2525M12", "CNMG120408");
    far[0].body = Some(body(&[bx([-12.5, -25.0, -60.0], [12.5, 0.0, 0.0])]));
    assert_eq!(refused(&far, st, None), LatheAssemblyError::Clearance(LatheBodyError::TipAway));
    // Degenerate bodies.
    assert_eq!(LatheBody::boxes(&[]), Err(LatheBodyError::Empty));
    assert_eq!(LatheBody::boxes(&[bx([0.0; 3], [1.0, 1.0, f64::NAN])]), Err(LatheBodyError::NonFinite));
    assert_eq!(LatheBody::boxes(&[bx([0.0; 3], [1.0, 0.0, 1.0])]), Err(LatheBodyError::Degenerate));
    assert_eq!(LatheBody::mesh(Vec::new()), Err(LatheBodyError::Empty));
    assert_eq!(
        LatheBody::mesh(vec![[[0.0; 3], [1.0, 0.0, 0.0], [0.0, f64::INFINITY, 0.0]]]),
        Err(LatheBodyError::NonFinite)
    );
}
