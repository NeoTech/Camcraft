//! #1890 deburr sizing law tests: `width` is the chamfer's chord
//! `w = 2 s sin(theta / 2)` on every wedge angle `theta`, Constant
//! Depth holds `d = s cos(theta / 2)`, the passes step from the outer
//! break in to the final one and stay inside the run's declared
//! `EdgeBreak`, and a Round profile on the sharp cone refuses by name.
//!
//! Gate map (#1890): covers camcraft_motion::finishing3d (deburr,
//! deburr_size). Lane shape: every law here runs in seconds.

use camcraft_motion::cl::{EdgeBreak, MoveKind, Toolpath};
use camcraft_motion::finishing3d::{deburr, BreakMode, BreakProfile, DeburrBreak, DeburrParams, UnreachableEdges};
use camcraft_motion::geometry::{Point3, Triangle};

fn p3(x: f64, y: f64, z: f64) -> Point3 {
    Point3::new(x, y, z)
}

/// A 90-deg cone (the setback is the drop), a 1 mm chord, and the tip
/// window z -3..0: only edges at z = 0 are in reach.
fn params(sizing: DeburrBreak) -> DeburrParams {
    DeburrParams {
        width: 1.0,
        tip_angle_deg: 90.0,
        clearance_z: 20.0,
        top_z: 0.0,
        bottom_z: -3.0,
        feed: 500.0,
        plunge_feed: 100.0,
        min_edge_angle_deg: 0.0,
        exclude_holes: false,
        hole_min_diameter: 0.0,
        hole_max_diameter: 0.0,
        unreachable: UnreachableEdges::SkipWithNote,
        sizing,
    }
}

/// Extrude a CCW (y, z) polygon along +x into a closed solid.
fn prism(poly: &[(f64, f64)], x_len: f64) -> Vec<Triangle> {
    let v = |side: f64, (y, z): (f64, f64)| p3(side, y, z);
    let mut t = Vec::new();
    for i in 0..poly.len() {
        let j = (i + 1) % poly.len();
        let (a, b) = (v(0.0, poly[i]), v(0.0, poly[j]));
        let (c, d) = (v(x_len, poly[j]), v(x_len, poly[i]));
        t.push(Triangle::new(a, b, c));
        t.push(Triangle::new(a, c, d));
    }
    for side in [0.0, x_len] {
        for i in 1..poly.len() - 1 {
            let (a, b, c) = (v(side, poly[0]), v(side, poly[i]), v(side, poly[i + 1]));
            t.push(if side > 0.0 { Triangle::new(a, b, c) } else { Triangle::new(a, c, b) });
        }
    }
    t
}

/// A block whose flat top (z = 0, y 5..20) meets a face that falls 60
/// deg below level at y = 5: a 120-degree wedge along x. The edge at
/// y = 20 is a 90-degree wedge.
fn wedge_block() -> Vec<Triangle> {
    let run = 10.0 / 60f64.to_radians().tan();
    prism(&[(5.0 - run, -10.0), (20.0, -10.0), (20.0, 0.0), (5.0, 0.0)], 30.0)
}

/// The declared break that runs along the long line y = `y`, z = 0.
fn break_on(t: &Toolpath, y: f64) -> &EdgeBreak {
    let on = |q: &Point3| (q.y - y).abs() < 1e-9 && q.z.abs() < 1e-9;
    t.breaks
        .iter()
        .find(|b| b.edge.windows(2).any(|s| on(&s[0]) && on(&s[1]) && (s[0].x - s[1].x).abs() > 29.0))
        .unwrap_or_else(|| panic!("no break on y = {y}: {:?}", t.breaks))
}

#[test]
fn constant_width_holds_the_chord_on_a_120_and_a_90_degree_wedge() {
    let t = deburr(&wedge_block(), &params(DeburrBreak::default())).expect("the top edges ride");
    // s = w / (2 sin(theta / 2)); the 90-deg cone's drop is s.
    let at120 = break_on(&t, 5.0).depth;
    let at90 = break_on(&t, 20.0).depth;
    assert!((at120 - 1.0 / 3f64.sqrt()).abs() < 1e-9, "120-deg wedge: {at120}");
    assert!((at90 - 1.0 / 2f64.sqrt()).abs() < 1e-9, "90-deg wedge: {at90}");
    // The chord back from each setback is the authored width.
    for (s, theta) in [(at120, 120f64), (at90, 90.0)] {
        let w = 2.0 * s * (0.5 * theta.to_radians()).sin();
        assert!((w - 1.0).abs() < 1e-9, "chord {w} at {theta} deg");
    }
}

#[test]
fn constant_depth_holds_the_depth_on_a_120_and_a_90_degree_wedge() {
    let sizing = DeburrBreak { mode: BreakMode::ConstantDepth, depth: 0.5, ..DeburrBreak::default() };
    let t = deburr(&wedge_block(), &params(sizing)).expect("the top edges ride");
    // s = d / cos(theta / 2).
    let at120 = break_on(&t, 5.0).depth;
    let at90 = break_on(&t, 20.0).depth;
    assert!((at120 - 1.0).abs() < 1e-9, "120-deg wedge: {at120}");
    assert!((at90 - 0.5 * 2f64.sqrt()).abs() < 1e-9, "90-deg wedge: {at90}");
    // The plunge onto each edge stands its own drop under it.
    let plunges: Vec<f64> = t.moves.iter().filter(|m| m.kind == MoveKind::Plunge).map(|m| m.to.z).collect();
    assert!(plunges.iter().any(|z| (z + 1.0).abs() < 1e-9), "{plunges:?}");
    assert!(plunges.iter().any(|z| (z + 0.5 * 2f64.sqrt()).abs() < 1e-9), "{plunges:?}");
}

#[test]
fn passes_step_from_the_outer_break_to_the_final_one_inside_the_declared_break() {
    let one = deburr(&wedge_block(), &params(DeburrBreak::default())).expect("one pass");
    let three = deburr(&wedge_block(), &params(DeburrBreak { passes: 3, ..DeburrBreak::default() })).expect("three passes");
    let count = |t: &Toolpath| t.moves.iter().filter(|m| m.kind == MoveKind::Plunge).count();
    assert_eq!(count(&three), 3 * count(&one), "each run rides three times");
    assert_eq!(three.breaks, one.breaks, "the passes declare the final break only");
    // The 120-deg run: plunges at s/3, 2s/3, s, outer to final.
    let s = break_on(&three, 5.0).depth;
    // (Its plunges are the ones whose next feed stays on y = 5; the
    // 90-deg run starts at the same corner and turns to y = 20.)
    let on_line: Vec<f64> = three
        .moves
        .windows(2)
        .filter(|w| w[0].kind == MoveKind::Plunge && (w[0].to.y - 5.0).abs() < 1e-9 && (w[1].to.y - 5.0).abs() < 1e-9)
        .map(|w| -w[0].to.z)
        .collect();
    assert_eq!(on_line.len(), 3, "{on_line:?}");
    for (k, d) in on_line.iter().enumerate() {
        let want = s * (k + 1) as f64 / 3.0;
        assert!((d - want).abs() < 1e-9, "pass {k}: drop {d}, want {want}");
    }
    // Every cutting tip stands on or above the declared bevel.
    for m in three.moves.iter().filter(|m| m.kind != MoveKind::Rapid) {
        let floor = three.breaks.iter().filter_map(|b| b.floor_at(m.to.x, m.to.y)).map(|(f, _)| f).fold(f64::INFINITY, f64::min);
        assert!(m.to.z >= floor - 1e-9, "tip {:?} under the declared floor {floor}", m.to);
    }
}

#[test]
fn a_round_profile_on_a_sharp_cone_refuses_by_name() {
    let sizing = DeburrBreak { profile: BreakProfile::Round, ..DeburrBreak::default() };
    let e = deburr(&wedge_block(), &params(sizing)).expect_err("a sharp cone cuts no round");
    let msg = e.to_string();
    assert!(msg.contains("Round break profile needs a ball-tipped cutter"), "{msg}");
    let e = deburr(&wedge_block(), &params(DeburrBreak { passes: 0, ..DeburrBreak::default() })).expect_err("no pass");
    assert!(e.to_string().contains("passes 1 to"), "{e}");
}

#[test]
fn the_size_law_names_the_wedges_it_reaches_and_its_bound() {
    let w = DeburrBreak::default();
    let d = DeburrBreak { mode: BreakMode::ConstantDepth, depth: 0.5, ..DeburrBreak::default() };
    assert!(w.setback(1.0, 20.0).is_none(), "a wedge sharper than 30 deg is out of Constant Width's range");
    assert!(d.setback(1.0, 160.0).is_none(), "a wedge flatter than 150 deg is out of Constant Depth's range");
    // The bound is the largest setback each mode can give.
    assert!((w.max_setback(1.0) - 1.0 / (2.0 * 15f64.to_radians().sin())).abs() < 1e-12);
    assert!((d.max_setback(1.0) - 0.5 / 75f64.to_radians().cos()).abs() < 1e-12);
    for theta in [30.0, 60.0, 90.0, 120.0, 179.0] {
        assert!(w.setback(1.0, theta).is_some_and(|s| s <= w.max_setback(1.0) + 1e-12), "{theta}");
    }
}
