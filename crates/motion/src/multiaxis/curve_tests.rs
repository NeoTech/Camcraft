//! The drive-curve contour's unit pins (M5.1 J): the edge rest, the
//! flat pose, the off-model refusal and the closed ring's roll.

use super::*;
use crate::geometry::cube;

fn boxy() -> Vec<Triangle> {
    cube(Point3::ZERO, Point3::new(40.0, 40.0, 15.0)).facets.to_vec()
}

fn cp() -> CurveParams {
    CurveParams {
        tool_diameter: 6.0,
        clearance_z: 25.0,
        retract_z: None,
        feed: 500.0,
        plunge_feed: 100.0,
        climb: true,
    }
}

#[test]
fn a_top_edge_chain_rests_on_the_bisector() {
    // The box's top-front edge (y = 0, z = 15) is CONVEX: both
    // faces fall away behind the ball, so the ball rides the edge
    // itself - centre a radius out along the 45-degree bisector,
    // (y, z) = (-r/√2, 15 + r/√2), touching the edge line (the
    // old plane law stood it at (-r, 15 + r), r·(√2 - 1) of air
    // off everything). Its axis leans off the fold, and the
    // stream locates the TIP: the centre backed off r along the
    // pose - with no lead, the edge point itself (the pivot law).
    let facets = boxy();
    let chain = vec![vec![
        Point3::new(5.0, 0.0, 15.0),
        Point3::new(20.0, 0.0, 15.0),
        Point3::new(35.0, 0.0, 15.0),
    ]];
    let path = contour_curve(&chain, &facets, &cp(), &AxisControl::default(), None).unwrap();
    let mids: Vec<_> = path.moves.iter().filter(|m| m.kind != MoveKind::Rapid).collect();
    assert!(mids.len() >= 2, "the edge walk stated {} stations", mids.len());
    let r = 3.0;
    let h = r * std::f64::consts::FRAC_1_SQRT_2;
    let mut reached = (f64::INFINITY, f64::NEG_INFINITY);
    for m in &mids {
        let a = m.axis.expect("a cut move poses");
        let c = add(m.to, scale(Point3::new(a[0], a[1], a[2]), r));
        assert!(
            (c.y + h).abs() < 1e-6 && (c.z - 15.0 - h).abs() < 1e-6,
            "centre {c:?} not on the edge rest (y, z) = ({}, {})",
            -h,
            15.0 + h
        );
        assert!(
            m.to.y.abs() < 1e-6 && (m.to.z - 15.0).abs() < 1e-6,
            "tip {:?} should stand on the edge it rides",
            m.to
        );
        assert!(a[1] < -0.6 && a[2] > 0.6, "pose {a:?} should lean off the fold");
        reached = (reached.0.min(c.x), reached.1.max(c.x));
    }
    assert!(reached.0 < 5.1 && reached.1 > 34.9, "the walk did not span the chain: {reached:?}");
}

/// Every cut station's ball touches the box: its centre (tip + r
/// along the pose) stands exactly r off the nearest facet.
fn all_touch(path: &Toolpath, facets: &[Triangle], r: f64) {
    for m in path.moves.iter().filter(|m| m.kind != MoveKind::Rapid && m.kind != MoveKind::Plunge) {
        let a = m.axis.expect("a station poses");
        let c = add(m.to, scale(Point3::new(a[0], a[1], a[2]), r));
        let d = facets
            .iter()
            .map(|f| {
                let e = sub(c, closest_pt_tri(c, &[f.a, f.b, f.c]));
                dot(e, e).sqrt()
            })
            .fold(f64::INFINITY, f64::min);
        assert!((d - r).abs() < 1e-6, "a station's ball stands {d} off the part, not r: {c:?}");
    }
}

#[test]
fn a_flat_top_chain_poses_near_vertical() {
    let facets = boxy();
    let chain = vec![vec![
        Point3::new(5.0, 20.0, 15.0),
        Point3::new(35.0, 20.0, 15.0),
    ]];
    let path = contour_curve(&chain, &facets, &cp(), &AxisControl::default(), None).unwrap();
    let last_feed = path
        .moves
        .iter()
        .rev()
        .find(|m| m.kind == MoveKind::Feed)
        .unwrap();
    // The ball centre stands a radius over the top (15 + r); the
    // upright stream states its TIP, on the face itself.
    assert!((last_feed.to.z - 15.0).abs() < 1e-6, "tip {:?}", last_feed.to);
    let ax = last_feed.axis.unwrap();
    assert!((ax[2] - 1.0).abs() < 1e-9, "a floor curve holds vertical, got {ax:?}");
}

#[test]
fn a_floating_chain_is_named_not_projected() {
    let facets = boxy();
    let chain = vec![vec![
        Point3::new(5.0, 20.0, 40.0),
        Point3::new(35.0, 20.0, 40.0),
    ]];
    let e = contour_curve(&chain, &facets, &cp(), &AxisControl::default(), None).unwrap_err();
    assert!(format!("{e}").contains("off the model"), "{e}");
}

#[test]
fn a_ring_closes_and_rapids_hold_the_pose_they_inherit() {
    let facets = boxy();
    let z = 7.5;
    let mut ring = Vec::new();
    for (x, y) in [
        (0.0, 0.0),
        (40.0, 0.0),
        (40.0, 40.0),
        (0.0, 40.0),
        (0.0, 0.0),
    ] {
        ring.push(Point3::new(x, y, z));
    }
    let path = contour_curve(&[ring], &facets, &cp(), &AxisControl::default(), None).unwrap();
    assert!(path.is_chained());
    assert!(path.moves.iter().all(|m| m.axis.is_none_or(|a| {
        let l = (a[0] * a[0] + a[1] * a[1] + a[2] * a[2]).sqrt();
        (l - 1.0).abs() < 1e-9
    })));
    // The first hop states nothing (nothing is posed yet); every
    // later rapid that travels goes under the pose the machine
    // already holds - the post keeps it modal, the stream says so.
    // The exception is the zero-travel rotary index (#3414): the
    // machine swings at a standstill, in air, to the pose the
    // first descend cuts under.
    let mut held: Option<[f64; 3]> = None;
    for m in &path.moves {
        if m.kind == MoveKind::Rapid && m.from != m.to {
            assert_eq!(m.axis, held, "a rapid changed the pose it inherited: {m:?}");
        }
        held = m.axis.or(held);
    }
    // Four vertical CONVEX corner edges: the ball rides each wall
    // a radius off it and ROLLS round each corner edge on the
    // radius-r cylinder about it, touching the edge line all the
    // way (no straight chord cuts the corner, no station floats
    // off it) - and the walk brings it home to station 0.
    all_touch(&path, &facets, 3.0);
    let centres: Vec<(Point3, [f64; 3])> = path
        .moves
        .iter()
        .filter(|m| m.kind != MoveKind::Rapid)
        .map(|m| {
            let a = m.axis.expect("posed");
            (add(m.to, scale(Point3::new(a[0], a[1], a[2]), 3.0)), a)
        })
        .collect();
    let (first, last) = (centres[0].0, centres[centres.len() - 1].0);
    assert!(dot(sub(first, last), sub(first, last)).sqrt() < 1e-9, "the ring did not come home");
    // Round the (+,-) corner the roll passes the horizontal
    // bisector out of the corner edge.
    let h = 3.0 * std::f64::consts::FRAC_1_SQRT_2;
    assert!(
        centres.iter().any(|(c, a)| {
            let e = sub(*c, Point3::new(40.0 + h, -h, 7.5));
            dot(e, e).sqrt() < 0.2 && a[2].abs() < 1e-9 && a[0] > 0.6 && a[1] < -0.6
        }),
        "the roll never passed the (+,-) corner's bisector"
    );
}
