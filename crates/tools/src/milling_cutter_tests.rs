//! #3646: multi-seat indexable cutters resolved to the milling tool -
//! pinned on a face mill and a shoulder end mill that share one insert,
//! a round-insert end mill, the envelope that holds the real cutter,
//! the assembled body checked against the real profile and every seat,
//! and the refusals by name.

use crate::component::{ComponentKind, CuttingItem, InsertSeat, ToolComponent};
use crate::component_fit::FitProperty;
use crate::component_geometry::{ComponentBody, ComponentGeometry};
use crate::connection_code::ConnectionCode;
use crate::frame::Frame;
use crate::insert_iso1832::InsertSpec;
use crate::insert_seat::HolderDesignation;
use crate::milling_cutter::{derive, MillingBody, MillingCutter, MillingError};
use crate::spun::SpunPoint;
use crate::tool_assembly::{resolve, AssemblyNode, Mount};
use crate::tools::{catalog, Tool, ToolKind};

fn made(id: &str, kind: ComponentKind) -> ToolComponent {
    ToolComponent::new(id, id, kind).unwrap_or_else(|e| panic!("{id}: {e}"))
}

fn code(text: &str) -> ConnectionCode {
    ConnectionCode::parse(text).unwrap_or_else(|e| panic!("{text}: {e}"))
}

/// A cutter body whose `ports` all take the seat of the ISO 5608 code
/// `holder`.
fn body(name: &str, holder: &str, ports: &[&str]) -> ToolComponent {
    let Ok(h) = HolderDesignation::parse(holder) else { panic!("{holder}") };
    let seats = ports.iter().map(|p| InsertSeat { port: (*p).into(), spec: h.seat }).collect();
    made(name, ComponentKind::ToolItem { mcs: code("CYL 20"), seats })
}

fn insert(text: &str) -> ToolComponent {
    let Ok(spec) = InsertSpec::parse(text) else { panic!("{text}") };
    made(text, ComponentKind::CuttingItem(CuttingItem::Insert(spec)))
}

const FOUR: [&str; 4] = ["CSW1_1", "CSW1_2", "CSW1_3", "CSW1_4"];

fn spec(dc: f64, kapr_deg: f64, apmx: f64) -> MillingBody {
    MillingBody { dc, kapr_deg, apmx, center_cutting: false, seat_count: None }
}

fn ok(b: &ToolComponent, s: &MillingBody, i: &ToolComponent) -> MillingCutter {
    derive(b, s, i).unwrap_or_else(|e| panic!("refused: {e}"))
}

fn refused(b: &ToolComponent, s: &MillingBody, i: &ToolComponent) -> MillingError {
    match derive(b, s, i) {
        Ok(c) => panic!("accepted: {c:?}"),
        Err(e) => e,
    }
}

/// A catalog end mill: the running data under every derived tool.
fn base() -> Tool {
    catalog()
        .into_iter()
        .map(|(_, t)| t)
        .find(|t| t.kind == ToolKind::EndMill)
        .unwrap_or_else(|| panic!("the catalog has no end mill"))
}

fn near(a: f64, b: f64) -> bool {
    (a - b).abs() < 1e-9
}

/// A 4-seat Ø50 90-degree face mill and a 2-seat Ø25 shoulder end mill
/// share one square insert: the diameter is DC, the corner the insert's
/// nose, the flutes the seats, the flute length APMX or the edge.
#[test]
fn a_face_mill_and_an_end_mill_share_one_insert() {
    let ins = insert("SCMT120408");
    let face = body("face mill", "SSDCN2525M12", &FOUR);
    let c = ok(&face, &spec(50.0, 90.0, 8.0), &ins);
    assert!(near(c.diameter, 50.0) && near(c.corner_r, 0.8) && near(c.flute_len, 8.0), "{c:?}");
    assert_eq!((c.flutes, c.kind, c.center_cutting), (4, ToolKind::BullNose, false));
    let t = c.tool(&base()).unwrap_or_else(|e| panic!("{e}"));
    assert_eq!((t.flutes, t.center_cutting, t.kind), (Some(4), false, ToolKind::BullNose));
    assert!(near(t.diameter, 50.0) && near(t.corner_r, 0.8) && near(t.flute_len, 8.0));
    // APMX past the edge: the 12 mm edge (the size code) bounds the depth.
    assert!(near(ok(&face, &spec(50.0, 90.0, 20.0), &ins).flute_len, 12.0));
    let end = body("shoulder mill", "SSDCN2525M12", &["CSW1_1", "CSW1_2"]);
    let e = ok(&end, &MillingBody { center_cutting: true, ..spec(25.0, 90.0, 10.0) }, &ins);
    assert!(near(e.diameter, 25.0) && near(e.corner_r, 0.8) && e.center_cutting && e.flutes == 2);
    // An edit of the shared insert reaches both cutters.
    let edited = insert("SCMT120404");
    assert!(near(ok(&face, &spec(50.0, 90.0, 8.0), &edited).corner_r, 0.4));
    assert!(near(ok(&end, &spec(25.0, 90.0, 10.0), &edited).corner_r, 0.4));
    let sharp = ok(&face, &spec(50.0, 90.0, 8.0), &insert("SCMT120400"));
    assert_eq!(sharp.kind, ToolKind::EndMill);
}

/// Below 90 degrees the edge runs outward: the depth is the edge's rise
/// at KAPR, and the cutting diameter is the edge's reach at that depth.
#[test]
fn a_45_degree_face_mill_cuts_its_edge_reach() {
    let face = body("face mill", "SSDCN2525M12", &FOUR);
    let ins = insert("SCMT120408");
    let c = ok(&face, &spec(50.0, 45.0, 6.0), &ins);
    assert!(near(c.flute_len, 6.0) && near(c.diameter, 62.0), "{c:?}");
    let deep = ok(&face, &spec(50.0, 45.0, 30.0), &ins);
    assert!(near(deep.flute_len, 12.0 * 45f64.to_radians().sin()), "{deep:?}");
}

/// A round-insert end mill with one seat port and a seat count: the
/// corner is half the insert, the depth no more than that corner.
#[test]
fn a_round_insert_end_mill_takes_half_its_insert_as_corner() {
    let b = body("button mill", "SRDCN2525M12", &["CSW1"]);
    let s = MillingBody { seat_count: Some(3), ..spec(32.0, 90.0, 4.0) };
    let c = ok(&b, &s, &insert("RCMT1204M0"));
    assert!(near(c.diameter, 32.0) && near(c.corner_r, 6.0) && near(c.flute_len, 4.0), "{c:?}");
    assert_eq!((c.flutes, c.kind), (3, ToolKind::BullNose));
    let deep = ok(&b, &MillingBody { apmx: 9.0, ..s }, &insert("RCMT1204M0"));
    assert!(near(deep.flute_len, 6.0));
}

/// The model radius of `t` at height `z`: the corner, the cutting
/// cylinder, the shank; 0 above the shank.
fn reach(t: &Tool, z: f64) -> f64 {
    let (r, rc) = (0.5 * t.diameter, t.corner_r);
    if z < -1e-9 || z > t.flute_len + t.shank_len + 1e-9 {
        0.0
    } else if z <= t.flute_len + 1e-9 {
        if z < rc { r - rc + (rc * rc - (rc - z) * (rc - z)).max(0.0).sqrt() } else { r }
    } else {
        0.5 * t.shank_d
    }
}

fn inside(t: &Tool, pts: &[[f64; 2]], what: &str) {
    for p in pts {
        assert!(p[0] <= reach(t, p[1]) + 1e-6, "{what}: ({:.4}, {:.4}) is outside the tool {:.4}", p[0], p[1], reach(t, p[1]));
    }
}

/// The real meridian outline of a square insert of edge `e` and nose
/// `rn` whose main edge leaves the tip point (`dc`/2, 0) at `kapr`.
fn square_outline(dc: f64, kapr: f64, e: f64, rn: f64) -> Vec<[f64; 2]> {
    let (s, c) = kapr.to_radians().sin_cos();
    let (u, v) = ([c, s], [-s, c]);
    let p0 = [0.5 * dc, 0.0];
    let at = |p: [f64; 2], d: [f64; 2], k: f64| [p[0] + d[0] * k, p[1] + d[1] * k];
    let corners = [at(p0, u, rn), at(p0, u, e), at(at(p0, u, e), v, e), at(p0, v, e), at(p0, v, rn)];
    let mut pts = Vec::new();
    for w in corners.windows(2) {
        for i in 0..=200 {
            let k = f64::from(i) / 200.0;
            pts.push([w[0][0] + (w[1][0] - w[0][0]) * k, w[0][1] + (w[1][1] - w[0][1]) * k]);
        }
    }
    // The nose arc at the tip: centre rn in from both edges.
    let centre = at(at(p0, u, rn), v, rn);
    for i in 0..=200 {
        let a = std::f64::consts::FRAC_PI_2 * f64::from(i) / 200.0;
        let (sa, ca) = a.sin_cos();
        pts.push([centre[0] - rn * (v[0] * ca + u[0] * sa), centre[1] - rn * (v[1] * ca + u[1] * sa)]);
    }
    pts
}

/// The envelope pin: the derived tool holds every point of the real
/// cutter - the edges, the nose, the rest of the insert above the
/// cutting depth, a round insert of its metric or its inch size.
#[test]
fn the_tool_holds_the_real_cutter() {
    let face = body("face mill", "SSDCN2525M12", &FOUR);
    for (kapr, apmx) in [(90.0, 8.0), (75.0, 6.0), (45.0, 6.0), (45.0, 30.0), (20.0, 3.0)] {
        for nose in ["SCMT120400", "SCMT120408"] {
            let ins = insert(nose);
            let t = ok(&face, &spec(50.0, kapr, apmx), &ins).tool(&base()).unwrap_or_else(|e| panic!("{e}"));
            let rn = t.corner_r;
            // SCMT12: a 12.7 mm inscribed circle, so a 12.7 mm square edge.
            inside(&t, &square_outline(50.0, kapr, 12.7, rn), &format!("{nose} at {kapr} deg"));
        }
    }
    let b = body("button mill", "SRDCN2525M12", &["CSW1"]);
    let t = ok(&b, &MillingBody { seat_count: Some(3), ..spec(32.0, 90.0, 4.0) }, &insert("RCMT1204M0"))
        .tool(&base())
        .unwrap_or_else(|e| panic!("{e}"));
    for ic in [12.0, 12.7] {
        let r = 0.5 * ic;
        let pts: Vec<[f64; 2]> = (0..=720)
            .map(|i| {
                let (s, c) = (std::f64::consts::TAU * f64::from(i) / 720.0).sin_cos();
                [16.0 - r + r * c, r + r * s]
            })
            .collect();
        inside(&t, &pts, &format!("a Ø{ic} round insert"));
    }
}

/// A seat of the probe rig: its port, the diameter of its cutting
/// point, its depth below the body MCS, and its yaw (degrees) off the
/// radial direction.
type Seat = (&'static str, f64, f64, f64);

/// Four seats on Ø`d`, 45 mm below the body MCS, all radial.
fn even(d: f64) -> Vec<Seat> {
    FOUR.iter().map(|p| (*p, d, 45.0, 0.0)).collect()
}

/// A BT40 holder, a cutter body "face mill" (the seats of `holder` at
/// `seats`, spaced evenly; its own body a cylinder of `body_r`,
/// `body_len` long below its MCS) and the insert `ins` in the first
/// seat.
fn rig(holder: &str, seats: &[Seat], body_len: f64, body_r: f64, ins: &str) -> Vec<AssemblyNode> {
    let p = SpunPoint::line;
    let cyl = |len: f64, r: f64| {
        ComponentBody::new(vec![p(0.0, 0.0), p(0.0, r), p(len, r), p(len, 0.0)], -len).unwrap_or_else(|e| panic!("{e}"))
    };
    let step = 360.0 / seats.len() as f64;
    let frames = seats
        .iter()
        .enumerate()
        .map(|(i, (port, d, depth, yaw))| {
            let (s, c) = (step * i as f64).to_radians().sin_cos();
            let (sy, cy) = (step * i as f64 + yaw).to_radians().sin_cos();
            let f = Frame::new([0.5 * d * c, 0.5 * d * s, -depth], [0.0, 0.0, 1.0], [cy, sy, 0.0]);
            ((*port).to_owned(), f.unwrap_or_else(|e| panic!("{e}")))
        })
        .collect();
    let geometry = ComponentGeometry::new(frames, Some(cyl(body_len, body_r)), None).unwrap_or_else(|e| panic!("{e}"));
    let bt40 = made("BT40 CYL 20", ComponentKind::AdaptiveItem { mcs: code("BT40"), csw: code("CYL 20") });
    let held = ComponentGeometry::axial("CSW", 50.0, Some(cyl(50.0, 25.0))).unwrap_or_else(|e| panic!("{e}"));
    let on = |parent: usize, port: Option<&str>| {
        Some(Mount { parent, port: port.map(str::to_owned), z_rot_deg: 0.0, projection_mm: None })
    };
    let ports: Vec<&str> = seats.iter().map(|s| s.0).collect();
    vec![
        AssemblyNode { component: bt40, geometry: held, mount: None },
        AssemblyNode { component: body("face mill", holder, &ports), geometry, mount: on(0, None) },
        AssemblyNode { component: insert(ins), geometry: ComponentGeometry::default(), mount: on(1, Some(ports[0])) },
    ]
}

/// The 4-seat face mill rig of the SCMT12 square insert.
fn assembly(seat_d: f64, body_r: f64) -> Vec<AssemblyNode> {
    rig("SSDCN2525M12", &even(seat_d), 40.0, body_r, "SCMT120408")
}

/// The cutter in its assembly: the gauge length and the judged body of
/// the holder and the cutter body, enclosed over the tool. The body
/// inside the cutting depth stays inside the cutter; one that stands
/// outside it, or seats off DC, are refused by name.
#[test]
fn the_assembled_face_mill_encloses_holder_and_body() {
    let c = ok(&body("face mill", "SSDCN2525M12", &FOUR), &spec(50.0, 45.0, 6.0), &insert("SCMT120408"));
    let nodes = assembly(50.0, 20.0);
    let r = resolve(&nodes).unwrap_or_else(|e| panic!("{e}"));
    assert!(near(r.gauge_len, 95.0), "{}", r.gauge_len);
    let alone = c.tool(&base()).unwrap_or_else(|e| panic!("{e}"));
    let t = c.assembled(&base(), &nodes).unwrap_or_else(|e| panic!("{e}"));
    assert!(near(t.diameter, 62.0) && near(t.flute_len, 6.0));
    assert!(t.shank_d >= alone.shank_d && t.shank_d >= 40.0, "{t:?}");
    assert!(near(t.holder_d, 50.0) && near(t.flute_len + t.shank_len + t.holder_len, 95.0), "{t:?}");
    let Err(MillingError::BodyOutside { r: br, .. }) = c.assembled(&base(), &assembly(50.0, 40.0)) else {
        panic!("a wide body passed")
    };
    assert!(near(br, 40.0));
    assert!(matches!(c.assembled(&base(), &assembly(48.0, 20.0)), Err(MillingError::SeatDiameter { .. })));
}

/// #4192 review 1: a body inside the cutting depth is checked against
/// the tool's real profile, the corner arc included - on a bull-nose
/// (square insert, 0.8 nose) and on a round-insert cutter. A body under
/// the flat of the tip passes.
#[test]
fn a_body_outside_the_rounded_corner_is_refused() {
    let c = ok(&body("face mill", "SSDCN2525M12", &FOUR), &spec(50.0, 90.0, 8.0), &insert("SCMT120408"));
    // Down to the tip at r 24.8: the 0.8 corner reaches only 24.2 there.
    let nodes = rig("SSDCN2525M12", &even(50.0), 45.0, 24.8, "SCMT120408");
    let Err(MillingError::BodyOutside { r, cut_r, .. }) = c.assembled(&base(), &nodes) else { panic!("bull-nose body passed") };
    assert!(near(r, 24.8) && near(cut_r, 24.2), "{r} {cut_r}");
    let under = rig("SSDCN2525M12", &even(50.0), 45.0, 24.2, "SCMT120408");
    assert!(c.assembled(&base(), &under).is_ok(), "a body under the flat of the tip was refused");
    // A round insert: the Ø32 cutter with a 6 mm corner reaches r 10 at the tip.
    let round = ok(&body("face mill", "SRDCN2525M12", &FOUR), &spec(32.0, 90.0, 4.0), &insert("RCMT1204M0"));
    let nodes = rig("SRDCN2525M12", &even(32.0), 45.0, 15.0, "RCMT1204M0");
    let Err(MillingError::BodyOutside { cut_r, .. }) = round.assembled(&base(), &nodes) else { panic!("round body passed") };
    assert!(near(cut_r, 10.0), "{cut_r}");
    let under = rig("SRDCN2525M12", &even(32.0), 45.0, 9.5, "RCMT1204M0");
    assert!(round.assembled(&base(), &under).is_ok(), "a body under the round tip was refused");
}

/// #4192 review 2: every seat port is checked, not only the mounted
/// one. The selected seat CSW1_1 is valid; a second seat off DC, off
/// the tip plane or turned other than CSW1_1 is refused by port. A one
/// port body with a seat count checks its one port.
#[test]
fn every_seat_is_checked_not_only_the_mounted_one() {
    let c = ok(&body("face mill", "SSDCN2525M12", &FOUR), &spec(50.0, 90.0, 8.0), &insert("SCMT120408"));
    for (i, edit, what) in [(1, (80.0, 45.0, 0.0), "radius"), (2, (50.0, 44.0, 0.0), "height"), (3, (50.0, 45.0, 10.0), "orientation")] {
        let mut seats = even(50.0);
        (seats[i].1, seats[i].2, seats[i].3) = edit;
        let e = c.assembled(&base(), &rig("SSDCN2525M12", &seats, 40.0, 20.0, "SCMT120408"));
        let Err(MillingError::SeatOff { port, what: w, .. }) = e else { panic!("{what}: {e:?}") };
        assert_eq!((port.as_str(), w), (FOUR[i], what));
    }
    let one = MillingBody { seat_count: Some(4), ..spec(50.0, 90.0, 8.0) };
    let c = ok(&body("face mill", "SSDCN2525M12", &["CSW1"]), &one, &insert("SCMT120408"));
    let seat = [("CSW1", 50.0, 45.0, 0.0)];
    let t = c.assembled(&base(), &rig("SSDCN2525M12", &seat, 40.0, 20.0, "SCMT120408"));
    assert_eq!(t.map(|t| t.flutes), Ok(Some(4)));
}

/// #4192 review 3: the assembly must mount the insert and the body the
/// cutter was derived from. An insert changed before assembly (a sharp
/// SCMT120400 under a cutter derived with a 0.8 nose) and another body
/// are refused, though both fit and cut on DC.
#[test]
fn an_assembly_with_other_components_is_refused() {
    let c = ok(&body("face mill", "SSDCN2525M12", &FOUR), &spec(50.0, 90.0, 8.0), &insert("SCMT120408"));
    let e = c.assembled(&base(), &rig("SSDCN2525M12", &even(50.0), 40.0, 20.0, "SCMT120400"));
    assert!(matches!(&e, Err(MillingError::Mismatch { why, .. }) if why.contains("SCMT120400")), "{e:?}");
    let other = ok(&body("other mill", "SSDCN2525M12", &FOUR), &spec(50.0, 90.0, 8.0), &insert("SCMT120408"));
    let e = other.assembled(&base(), &assembly(50.0, 20.0));
    assert!(matches!(&e, Err(MillingError::Mismatch { why, .. }) if why.contains("face mill")), "{e:?}");
}

#[test]
fn seats_that_do_not_take_the_insert_are_refused_by_the_fit_checks() {
    let face = body("face mill", "SSDCN2525M12", &FOUR);
    let MillingError::Fit(e) = refused(&face, &spec(50.0, 90.0, 8.0), &insert("SCMT09T308")) else { panic!("size") };
    assert_eq!((e.property, e.parent.as_str(), e.child.as_str()), (FitProperty::Size, "face mill", "SCMT09T308"));
    let MillingError::Fit(e) = refused(&face, &spec(50.0, 90.0, 8.0), &insert("CCMT120408")) else { panic!("shape") };
    assert_eq!(e.property, FitProperty::Shape);
    let MillingError::Fit(e) = refused(&face, &spec(50.0, 90.0, 8.0), &face) else { panic!("kind") };
    assert_eq!(e.property, FitProperty::Kind);
}

#[test]
fn degenerate_cutters_are_refused_by_name() {
    let face = body("face mill", "SSDCN2525M12", &FOUR);
    let ins = insert("SCMT120408");
    let s = spec(50.0, 90.0, 8.0);
    for (bad, what) in [(MillingBody { dc: f64::NAN, ..s }, "DC"), (MillingBody { apmx: f64::INFINITY, ..s }, "APMX")] {
        assert!(matches!(refused(&face, &bad, &ins), MillingError::NonFinite { what: w, .. } if w == what));
    }
    let nan = MillingBody { kapr_deg: f64::NAN, ..s };
    assert!(matches!(refused(&face, &nan, &ins), MillingError::NonFinite { .. }));
    for dc in [0.0, -10.0, 600.0] {
        assert!(matches!(refused(&face, &MillingBody { dc, ..s }, &ins), MillingError::BadDiameter { .. }));
    }
    assert!(matches!(refused(&face, &MillingBody { apmx: 0.0, ..s }, &ins), MillingError::BadDepth { .. }));
    for k in [0.0, -45.0, 95.0] {
        assert!(matches!(refused(&face, &MillingBody { kapr_deg: k, ..s }, &ins), MillingError::BadLeadAngle { .. }));
    }
    assert!(matches!(refused(&face, &MillingBody { seat_count: Some(0), ..s }, &ins), MillingError::NoSeats { .. }));
    let e = refused(&face, &MillingBody { seat_count: Some(5), ..s }, &ins);
    assert!(matches!(e, MillingError::SeatCount { stated: 5, ports: 4, .. }), "{e}");
    let one = body("one port", "SSDCN2525M12", &["CSW1"]);
    let e = refused(&one, &MillingBody { seat_count: Some(33), ..s }, &ins);
    assert!(matches!(e, MillingError::TooManySeats { count: 33, .. }), "{e}");
    assert!(matches!(refused(&ins, &s, &ins), MillingError::NotABody { .. }));
    // A nose larger than half DC, and one the 45-degree run-out cannot hold.
    let small = MillingBody { dc: 1.0, ..s };
    assert!(matches!(refused(&face, &small, &insert("SCMT120408")), MillingError::Corner { .. }));
    let shallow = MillingBody { kapr_deg: 45.0, apmx: 0.5, ..s };
    assert!(matches!(refused(&face, &shallow, &ins), MillingError::Corner { .. }));
    let b = body("button mill", "SRDCN2525M12", &["CSW1"]);
    let e = refused(&b, &MillingBody { dc: 10.0, ..s }, &insert("RCMT1204M0"));
    assert!(matches!(e, MillingError::Corner { .. }), "{e}");
    assert!(e.to_string().contains("button mill"), "{e}");
}
