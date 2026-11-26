//! #3643: spindle tool assemblies resolved to gauge length, cutting
//! parameters and the judged body - pinned against today's library
//! holder, stacked, indexed, and refused by name.

use crate::component::{ComponentKind, CuttingItem, InsertSeat, ToolComponent};
use crate::component_fit::FitProperty;
use crate::component_geometry::{ComponentBody, ComponentGeometry, GeometryError};
use crate::connection_code::ConnectionCode;
use crate::frame::{Frame, FrameError};
use crate::holder_library::{find, HolderRef};
use crate::holder_profile::HolderBand;
use crate::insert_iso1832::InsertSpec;
use crate::insert_seat::HolderDesignation;
use crate::spun::{SpunPoint, SpunProfile};
use crate::tool_assembly::{resolve, AssemblyError, AssemblyNode, Mount, ResolvedAssembly};
use crate::tools::{catalog, Tool, ToolKind};

fn made(id: &str, kind: ComponentKind) -> ToolComponent {
    ToolComponent::new(id, id, kind).unwrap_or_else(|e| panic!("{id}: {e}"))
}

fn code(text: &str) -> ConnectionCode {
    ConnectionCode::parse(text).unwrap_or_else(|e| panic!("{text}: {e}"))
}

fn adaptive(id: &str, mcs: &str, csw: &str) -> ToolComponent {
    made(id, ComponentKind::AdaptiveItem { mcs: code(mcs), csw: code(csw) })
}

/// A flat end mill of the catalog with its shank set to `shank_d`.
fn end_mill(shank_d: f64) -> Tool {
    let t = catalog()
        .into_iter()
        .map(|(_, t)| t)
        .find(|t| t.kind == ToolKind::EndMill && t.corner_r == 0.0)
        .unwrap_or_else(|| panic!("the catalog has no flat end mill"));
    Tool { diameter: shank_d, shank_d, ..t }
}

fn solid(tool: &Tool, mcs: &str) -> ToolComponent {
    made("end mill", ComponentKind::CuttingItem(CuttingItem::Solid { tool: Box::new(tool.clone()), mcs: code(mcs) }))
}

fn node(component: ToolComponent, geometry: ComponentGeometry, mount: Option<Mount>) -> AssemblyNode {
    AssemblyNode { component, geometry, mount }
}

fn on(parent: usize) -> Option<Mount> {
    Some(Mount { parent, port: None, z_rot_deg: 0.0, projection_mm: None })
}

fn stickout(parent: usize, p: f64) -> Option<Mount> {
    Some(Mount { projection_mm: Some(p), ..on(parent).unwrap_or_else(|| unreachable!()) })
}

fn axial(depth: f64, body: Option<ComponentBody>) -> ComponentGeometry {
    ComponentGeometry::axial("CSW", depth, body).unwrap_or_else(|e| panic!("{e}"))
}

/// A plain cylinder body `len` long of radius `r`, hanging below the MCS.
fn cylinder(len: f64, r: f64) -> ComponentBody {
    let p = SpunPoint::line;
    ComponentBody::new(vec![p(0.0, 0.0), p(0.0, r), p(len, r), p(len, 0.0)], -len).unwrap_or_else(|e| panic!("{e}"))
}

fn ok(nodes: &[AssemblyNode]) -> ResolvedAssembly {
    resolve(nodes).unwrap_or_else(|e| panic!("refused: {e}"))
}

fn refused(nodes: &[AssemblyNode]) -> AssemblyError {
    match resolve(nodes) {
        Ok(r) => panic!("accepted: gauge {}", r.gauge_len),
        Err(e) => e,
    }
}

fn near_bands(a: &[HolderBand], b: &[HolderBand]) {
    assert_eq!(a.len(), b.len(), "{a:?} vs {b:?}");
    for (x, y) in a.iter().zip(b) {
        let d = (x.above - y.above).abs().max((x.len - y.len).abs()).max((x.r - y.r).abs());
        assert!(d < 1e-9, "{x:?} vs {y:?}");
    }
}

/// The equivalence pin: a BT30 ER20 chuck, an ER20 collet and a solid
/// end mill at a stickout give the gauge length and the judged body
/// that today's library holder reference gives for the same numbers.
#[test]
fn a_collet_chuck_assembly_is_judged_as_the_library_holder() {
    let entry = find("BT30 ER20").unwrap_or_else(|e| panic!("{e}"));
    let tool = end_mill(10.0);
    let p = tool.flute_len + 12.5;
    let chuck = ComponentGeometry::holder(&entry, "CSW", entry.gauge_len).unwrap_or_else(|e| panic!("{e}"));
    let nodes = [
        node(adaptive("BT30 ER20 chuck", "BT30", "ER20"), chuck, None),
        node(adaptive("ER20 collet 10", "ER20", "CYL 10"), axial(0.0, None), on(0)),
        node(solid(&tool, "CYL 10"), ComponentGeometry::default(), stickout(1, p)),
    ];
    let r = ok(&nodes);
    let today = HolderRef::to(&entry, entry.gauge_len, p);
    let spun = SpunProfile::new(today.points().unwrap_or_else(|e| panic!("{e}"))).unwrap_or_else(|e| panic!("{e}"));
    assert!((r.gauge_len - (p + entry.gauge_len)).abs() < 1e-9, "gauge {}", r.gauge_len);
    let Some(body) = r.body else { panic!("no body") };
    near_bands(body.profile.bands(), spun.holder_profile().bands());
    assert_eq!(r.tool.as_ref(), Some(&tool));
    assert_eq!((r.cutting.diameter, r.cutting.corner_r), (tool.diameter, tool.corner_r));
    assert!(body.profile.enclose(&tool).is_some());
}

/// The same pin for a Weldon holder with a stated bore stop: with no
/// stickout the shank sits home on the stop, so the tip stands the
/// tool's length less the stop depth below the nose face - exactly as
/// that stickout, stated, gives it.
#[test]
fn a_weldon_holder_with_the_shank_home_is_judged_as_the_library_holder() {
    let entry = find("BT40 Weldon 12 mm").unwrap_or_else(|e| panic!("{e}"));
    let tool = end_mill(12.0);
    let depth = 0.5 * tool.shank_len;
    let p = tool.flute_len + tool.shank_len - depth;
    let holder = ComponentGeometry::holder(&entry, "CSW", entry.gauge_len)
        .and_then(|g| g.with_stop("CSW", depth))
        .unwrap_or_else(|e| panic!("{e}"));
    let today = HolderRef::to(&entry, entry.gauge_len, p).points().unwrap_or_else(|e| panic!("{e}"));
    let spun = SpunProfile::new(today).unwrap_or_else(|e| panic!("{e}"));
    for mount in [on(0), stickout(0, p)] {
        let nodes = [
            node(adaptive("BT40 Weldon 12", "BT40", "WELDON 12"), holder.clone(), None),
            node(solid(&tool, "WELDON 12"), ComponentGeometry::default(), mount),
        ];
        let r = ok(&nodes);
        assert!((r.gauge_len - (p + entry.gauge_len)).abs() < 1e-9, "gauge {}", r.gauge_len);
        let Some(body) = r.body else { panic!("no body") };
        near_bands(body.profile.bands(), spun.holder_profile().bands());
    }
}

/// A shank has a home only on a stated stop: with no stickout and no
/// bore stop it is refused (the nose face is no stop), as it is when
/// the stop is as deep as the tool is long, or when a stickout pushes
/// the shank past the stop.
#[test]
fn a_shank_with_no_stated_stop_or_past_its_stop_is_refused() {
    let entry = find("BT40 Weldon 12 mm").unwrap_or_else(|e| panic!("{e}"));
    let tool = end_mill(12.0);
    let len = tool.flute_len + tool.shank_len;
    let holder = ComponentGeometry::holder(&entry, "CSW", entry.gauge_len).unwrap_or_else(|e| panic!("{e}"));
    let stickout_refused = |n: &[AssemblyNode], want: &str| match refused(n) {
        AssemblyError::Stickout { ref name, why } => assert!(name == "end mill" && why.contains(want), "{why}"),
        e => panic!("not a stickout refusal: {e}"),
    };
    let mut n = [
        node(adaptive("BT40 Weldon 12", "BT40", "WELDON 12"), holder.clone(), None),
        node(solid(&tool, "WELDON 12"), ComponentGeometry::default(), on(0)),
    ];
    stickout_refused(&n, "no bore stop");
    stickout_refused(&plain(&tool), "no bore stop");
    let depth = 0.5 * tool.shank_len;
    let stop = |d: f64| holder.clone().with_stop("CSW", d).unwrap_or_else(|e| panic!("{e}"));
    n[0].geometry = stop(len);
    stickout_refused(&n, "as deep as the tool");
    n[0].geometry = stop(depth);
    n[1].mount = stickout(0, len - depth - 1.0);
    stickout_refused(&n, "past the bore stop");
    n[1].mount = stickout(0, len - depth + 1.0);
    assert!((ok(&n).gauge_len - (len - depth + 1.0 + entry.gauge_len)).abs() < 1e-9, "a shallower shank is legal");
    assert_eq!(holder.clone().with_stop("CSW2", depth), Err(GeometryError::UnknownPort("CSW2".into())));
    for bad in [0.0, -1.0, f64::NAN] {
        assert!(matches!(holder.clone().with_stop("CSW", bad), Err(GeometryError::BadStop { .. })), "{bad}");
    }
}

/// Where two bodies overlap the union keeps the larger radius: an ER
/// collet whose body reaches 20 mm up into the chuck nose does not
/// shrink the chuck there.
#[test]
fn overlapping_bodies_keep_the_larger_radius() {
    let tool = end_mill(10.0);
    let p = tool.flute_len + 20.0;
    let pt = SpunPoint::line;
    let pts = vec![pt(0.0, 0.0), pt(0.0, 10.0), pt(30.0, 10.0), pt(30.0, 0.0)];
    let collet = ComponentBody::new(pts, -10.0).unwrap_or_else(|e| panic!("{e}"));
    let nodes = [
        node(adaptive("BT40 ER20 chuck", "BT40", "ER20"), axial(60.0, Some(cylinder(60.0, 17.5))), None),
        node(adaptive("ER20 collet 10", "ER20", "CYL 10"), axial(0.0, Some(collet)), on(0)),
        node(solid(&tool, "CYL 10"), ComponentGeometry::default(), stickout(1, p)),
    ];
    let Some(body) = ok(&nodes).body else { panic!("no body") };
    let b = &body.profile;
    assert!((b.bottom() - (p - 10.0)).abs() < 1e-9 && (b.top() - (p + 60.0)).abs() < 1e-9, "{:?}", b.bands());
    for (z, r) in [(p - 5.0, 10.0), (p + 10.0, 17.5), (p + 40.0, 17.5)] {
        assert!((b.reach_over(z, z) - r).abs() < 1e-9, "at {z}: {:?}", b.bands());
    }
}

/// A span that no body covers, between two bodies, takes the smaller
/// radius of its neighbours (the child's shank or spigot runs through
/// it), never 0.
#[test]
fn a_gap_between_bodies_takes_the_smaller_neighbour() {
    let tool = end_mill(12.0);
    let p = tool.flute_len + 10.0;
    let pt = SpunPoint::line;
    let pts = vec![pt(0.0, 0.0), pt(0.0, 15.0), pt(20.0, 15.0), pt(20.0, 0.0)];
    let lower = ComponentBody::new(pts, -50.0).unwrap_or_else(|e| panic!("{e}"));
    let nodes = [
        node(adaptive("BT40 to HSK-A63", "BT40", "HSK-A63"), axial(30.0, Some(cylinder(30.0, 40.0))), None),
        node(adaptive("HSK-A63 Weldon 12", "HSK-A63", "WELDON 12"), axial(50.0, Some(lower)), on(0)),
        node(solid(&tool, "WELDON 12"), ComponentGeometry::default(), stickout(1, p)),
    ];
    let Some(body) = ok(&nodes).body else { panic!("no body") };
    let b = &body.profile;
    assert!((b.bottom() - p).abs() < 1e-9 && (b.top() - (p + 80.0)).abs() < 1e-9, "{:?}", b.bands());
    for (z, r) in [(p + 10.0, 15.0), (p + 35.0, 15.0), (p + 65.0, 40.0)] {
        assert!((b.reach_over(z, z) - r).abs() < 1e-9, "at {z}: {:?}", b.bands());
    }
}

/// Adapter + holder + tool: the lengths sum, each MCS sits where the
/// stack puts it, and the bodies are judged as one union.
#[test]
fn a_stack_up_sums_its_lengths_and_unites_its_bodies() {
    let tool = end_mill(12.0);
    let p = tool.flute_len + 10.0;
    let nodes = [
        node(adaptive("BT40 to HSK-A63", "BT40", "HSK-A63"), axial(30.0, Some(cylinder(30.0, 40.0))), None),
        node(adaptive("HSK-A63 Weldon 12", "HSK-A63", "WELDON 12"), axial(80.0, Some(cylinder(80.0, 21.0))), on(0)),
        node(solid(&tool, "WELDON 12"), ComponentGeometry::default(), stickout(1, p)),
    ];
    let r = ok(&nodes);
    assert!((r.gauge_len - (30.0 + 80.0 + p)).abs() < 1e-9);
    let z: Vec<f64> = r.frames.iter().map(|f| f.origin()[2]).collect();
    assert!((z[0] - r.gauge_len).abs() < 1e-9 && (z[1] - (p + 80.0)).abs() < 1e-9, "{z:?}");
    assert!((z[2] - (tool.flute_len + tool.shank_len)).abs() < 1e-9, "the shank's back end: {z:?}");
    let Some(body) = r.body else { panic!("no body") };
    let want = [HolderBand { above: p, len: 80.0, r: 21.0 }, HolderBand { above: p + 80.0, len: 30.0, r: 40.0 }];
    near_bands(body.profile.bands(), &want);
    assert!(body.spun.is_some(), "the united body has a drawn outline");
}

/// An insert in a cutter body's seat: the cutting diameter is twice the
/// seat point's distance from the axis, the corner radius the insert's
/// nose radius. An index turn moves the frames, not the diameter.
#[test]
fn an_indexable_cutter_takes_its_diameter_and_corner_from_seat_and_insert() {
    for z_rot in [0.0, 30.0, 90.0] {
        let nodes = indexable("CCMT09T304", "CSW1", z_rot);
        let r = ok(&nodes);
        assert!((r.cutting.diameter - 25.0).abs() < 1e-9, "{z_rot}: {}", r.cutting.diameter);
        assert!((r.cutting.corner_r - 0.4).abs() < 1e-12);
        assert!((r.gauge_len - (50.0 + 60.0)).abs() < 1e-9);
        assert!(r.tool.is_none());
        let x = r.frames[1].x();
        let (s, c) = z_rot.to_radians().sin_cos();
        assert!((x[0] - c).abs() < 1e-9 && (x[1] - s).abs() < 1e-9, "{z_rot}: {x:?}");
    }
}

/// A BT40 holder, a single-seat cutter body (a CC seat 12.5 mm off the
/// axis, 60 mm below its shank's MCS) turned `z_rot`, and an insert.
fn indexable(insert: &str, port: &str, z_rot: f64) -> [AssemblyNode; 3] {
    let Ok(h) = HolderDesignation::parse("SCLCR2020K09") else { panic!("holder code") };
    let seats = vec![InsertSeat { port: "CSW1".into(), spec: h.seat }];
    let body = made("fly cutter", ComponentKind::ToolItem { mcs: code("CYL 20"), seats });
    let seat = Frame::new([12.5, 0.0, -60.0], [0.0, 0.0, 1.0], [1.0, 0.0, 0.0]).unwrap_or_else(|e| panic!("{e}"));
    let geometry = ComponentGeometry::new(vec![("CSW1".into(), seat)], None, None).unwrap_or_else(|e| panic!("{e}"));
    let Ok(spec) = InsertSpec::parse(insert) else { panic!("{insert}") };
    let mount = Mount { parent: 1, port: Some(port.into()), z_rot_deg: 0.0, projection_mm: None };
    [
        node(adaptive("BT40 CYL 20", "BT40", "CYL 20"), axial(50.0, Some(cylinder(50.0, 25.0))), None),
        node(body, geometry, Some(Mount { z_rot_deg: z_rot, ..on(0).unwrap_or_else(|| unreachable!()) })),
        node(made(insert, ComponentKind::CuttingItem(CuttingItem::Insert(spec))), ComponentGeometry::default(), Some(mount)),
    ]
}

#[test]
fn a_wrong_insert_or_a_missing_seat_is_refused_by_the_fit_checks() {
    let AssemblyError::Fit(e) = refused(&indexable("DCMT11T304", "CSW1", 0.0)) else { panic!("not a fit refusal") };
    assert_eq!(e.property, FitProperty::Shape);
    let AssemblyError::Fit(e) = refused(&indexable("CCMT09T304", "CSW2", 0.0)) else { panic!("not a fit refusal") };
    assert_eq!(e.property, FitProperty::Port);
    let tool = end_mill(10.0);
    let nodes = [
        node(adaptive("chuck", "BT30", "ER20"), axial(60.0, None), None),
        node(solid(&tool, "CYL 10"), ComponentGeometry::default(), on(0)),
    ];
    let AssemblyError::Fit(e) = refused(&nodes) else { panic!("not a fit refusal") };
    assert_eq!(e.property, FitProperty::Coupling, "an ER chuck takes a collet, not a shank");
}

fn plain(tool: &Tool) -> Vec<AssemblyNode> {
    vec![
        node(adaptive("holder", "BT40", "CYL 12"), axial(60.0, Some(cylinder(60.0, 20.0))), None),
        node(solid(tool, "CYL 12"), ComponentGeometry::default(), on(0)),
    ]
}

#[test]
fn links_frames_and_transforms_are_refused_by_name() {
    let tool = end_mill(12.0);
    let mut n = plain(&tool);
    n[0].geometry = ComponentGeometry::default();
    assert_eq!(refused(&n), AssemblyError::MissingFrame { name: "holder".into(), port: "CSW".into() });
    let mut n = indexable("CCMT09T304", "CSW1", 0.0);
    let Ok(h) = HolderDesignation::parse("SCLCR2020K09") else { panic!() };
    let seats = vec![InsertSeat { port: "CSW1".into(), spec: h.seat }, InsertSeat { port: "CSW2".into(), spec: h.seat }];
    n[1].component = made("two seats", ComponentKind::ToolItem { mcs: code("CYL 20"), seats });
    n[2].mount = on(1);
    assert!(matches!(refused(&n), AssemblyError::AmbiguousPort { ref ports, .. } if ports == "CSW1, CSW2"));
    let mut n = plain(&tool);
    n.push(node(adaptive("ext", "CYL 12", "CYL 12"), axial(10.0, None), on(1)));
    n[1].mount = on(2);
    assert_eq!(refused(&n), AssemblyError::Cycle { name: "end mill".into() });
    let mut n = plain(&tool);
    n[1].mount = Some(Mount { z_rot_deg: f64::NAN, ..on(0).unwrap_or_else(|| unreachable!()) });
    assert_eq!(refused(&n), AssemblyError::NonFinite { name: "end mill".into(), what: "z_rot_deg" });
    let mut n = plain(&tool);
    n[1].mount = stickout(0, f64::INFINITY);
    assert_eq!(refused(&n), AssemblyError::NonFinite { name: "end mill".into(), what: "projection_mm" });
    let mut n = plain(&tool);
    n[1].mount = on(7);
    assert_eq!(refused(&n), AssemblyError::BadParent { name: "end mill".into(), parent: 7 });
    let mut n = plain(&tool);
    n[1].mount = None;
    assert!(matches!(refused(&n), AssemblyError::TwoRoots { .. }));
    assert_eq!(refused(&plain(&tool)[..1]), AssemblyError::NoCutter);
    assert_eq!(refused(&[]), AssemblyError::Empty);
}

#[test]
fn bodies_in_the_flutes_and_bad_stickouts_are_refused_by_name() {
    let tool = end_mill(12.0);
    let mut n = plain(&tool);
    n[1].mount = stickout(0, tool.flute_len * 0.5);
    assert!(matches!(refused(&n), AssemblyError::BodyInFlutes { .. }), "{}", refused(&n));
    n[1].mount = stickout(0, tool.flute_len + tool.shank_len);
    assert!(matches!(refused(&n), AssemblyError::Stickout { .. }));
    n[1].mount = stickout(0, -1.0);
    assert!(matches!(refused(&n), AssemblyError::Stickout { .. }));
    let mut n = plain(&tool);
    n.insert(1, node(adaptive("ext", "CYL 12", "CYL 12"), axial(40.0, None), stickout(0, 20.0)));
    n[2].mount = on(1);
    assert!(matches!(refused(&n), AssemblyError::Stickout { ref name, .. } if name == "ext"));
}

#[test]
fn an_off_axis_body_and_degenerate_geometry_are_refused_by_name() {
    let tool = end_mill(12.0);
    let mut n = plain(&tool);
    let off = Frame::new([5.0, 0.0, -60.0], [0.0, 0.0, 1.0], [1.0, 0.0, 0.0]).unwrap_or_else(|e| panic!("{e}"));
    n.insert(1, node(adaptive("ext", "CYL 12", "CYL 12"), axial(30.0, Some(cylinder(30.0, 9.0))), on(0)));
    n[0].geometry = ComponentGeometry::new(vec![("CSW".into(), off)], None, None).unwrap_or_else(|e| panic!("{e}"));
    n[2].mount = stickout(1, tool.flute_len + 10.0);
    assert_eq!(refused(&n), AssemblyError::OffAxis { name: "ext".into() });
    assert_eq!(Frame::new([0.0; 3], [0.0; 3], [1.0, 0.0, 0.0]), Err(FrameError::ZeroAxis));
    assert_eq!(Frame::new([f64::NAN, 0.0, 0.0], [0.0, 0.0, 1.0], [1.0, 0.0, 0.0]), Err(FrameError::NonFinite));
    assert_eq!(Frame::new([0.0; 3], [0.0, 0.0, 1.0], [0.0, 0.0, 2.0]), Err(FrameError::CollinearX));
    let p = SpunPoint::line;
    let pts = vec![p(0.0, 0.0), p(0.0, 5.0), p(5.0, 5.0), p(5.0, 0.0)];
    assert_eq!(ComponentBody::new(pts, f64::NAN), Err(GeometryError::NonFiniteBase));
    let twice = vec![("CSW".to_owned(), Frame::IDENTITY), ("CSW".to_owned(), Frame::IDENTITY)];
    assert_eq!(ComponentGeometry::new(twice, None, None), Err(GeometryError::DuplicatePort("CSW".into())));
    let entry = find("BT30 ER20").unwrap_or_else(|e| panic!("{e}"));
    assert!(matches!(ComponentGeometry::holder(&entry, "CSW", 1.0), Err(GeometryError::Holder(_))));
}
