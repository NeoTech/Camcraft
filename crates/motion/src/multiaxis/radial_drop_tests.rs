//! The radial drop kernel's own tests (#2772, the radial drop slice):
//! the exact face, edge and vertex contacts on hand-built facets; a
//! brute-force oracle that agrees on a helical screw rotor (the
//! reporter's kind of part, which shares no rotation axis); the
//! gouge-free law (no sampled centre outward of the contact comes
//! nearer than the ball radius, and the contact itself touches);
//! a ray past the part's end that meets nothing; and the refusals
//! of input that cannot be meant.
//!
//! Provenance: our own fixtures and our own oracle (the point-to-
//! triangle distance of `finishing3d::support`); nothing of vendor
//! binaries.

use super::axis::Axis;
use super::radial_drop::RadialDrop;
use crate::finishing3d::closest_pt_tri;
use crate::geometry::{Point3, Triangle};

const TAU: f64 = std::f64::consts::TAU;

fn p(x: f64, y: f64, z: f64) -> Point3 {
    Point3::new(x, y, z)
}

/// World X through the origin: the A table's line.
fn x_axis() -> Axis {
    Axis { origin: p(0.0, 0.0, 0.0), dir: p(1.0, 0.0, 0.0) }
}

/// The nearest distance from `c` to the mesh.
fn mesh_dist(c: Point3, mesh: &[Triangle]) -> f64 {
    mesh.iter()
        .map(|t| {
            let q = closest_pt_tri(c, &[t.a, t.b, t.c]);
            let d = c - q;
            d.dot(d).sqrt()
        })
        .fold(f64::INFINITY, f64::min)
}

/// The centre at radius `rho` on the ray of station `s`, angle `a`.
fn centre(k: &RadialDrop, s: f64, a: f64, rho: f64) -> Point3 {
    let u = k.radial(a);
    p(s + rho * u.x, rho * u.y, rho * u.z)
}

/// The angle whose radial is world +Z (the axis.rs basis law decides
/// where angle zero points; the tests ask, never assume).
fn up_angle(k: &RadialDrop) -> f64 {
    let (u0, u1) = (k.radial(0.0), k.radial(TAU / 4.0));
    u1.z.atan2(u0.z)
}

/// A twin-lobed helical screw along X (radius 8 +- 3 by the lobe
/// phase, which turns with X), the reporter's kind of part: its
/// normal lines share no axis. Closed with flat end caps.
fn screw(n_th: usize, n_x: usize, len: f64, turns: f64) -> Vec<Triangle> {
    let rad = |x: f64, th: f64| 8.0 + 3.0 * (2.0 * (th - TAU * turns * x / len)).cos();
    let pt = |i: usize, j: usize| {
        let x = len * i as f64 / n_x as f64 - len / 2.0;
        let th = TAU * j as f64 / n_th as f64;
        let r = rad(x + len / 2.0, th);
        p(x, r * th.cos(), r * th.sin())
    };
    let mut f = Vec::new();
    for i in 0..n_x {
        for j in 0..n_th {
            let (a, b, c, d) = (pt(i, j), pt(i + 1, j), pt(i + 1, j + 1), pt(i, j + 1));
            f.push(Triangle::new(a, b, c));
            f.push(Triangle::new(a, c, d));
        }
    }
    for (i, x) in [(0, -len / 2.0), (n_x, len / 2.0)] {
        let hub = p(x, 0.0, 0.0);
        for j in 0..n_th {
            f.push(Triangle::new(hub, pt(i, j + 1), pt(i, j)));
        }
    }
    f
}

#[test]
fn a_face_contact_rests_the_ball_r_off_the_plane() {
    // A wide flat facet at z = 10 across the ray: the ball centre
    // rests exactly r above it.
    let mesh = vec![Triangle::new(p(-50.0, -50.0, 10.0), p(50.0, -50.0, 10.0), p(0.0, 50.0, 10.0))];
    let k = RadialDrop::new(&mesh, &x_axis(), 3.0).expect("a valid kernel");
    let rho = k.centre_radius(0.0, up_angle(&k)).expect("the facet is under the ray");
    assert!((rho - 13.0).abs() < 1e-9, "face contact at 10 + r, got {rho}");
}

#[test]
fn an_edge_contact_rests_the_ball_on_the_ridge() {
    // A roof whose ridge runs along X at z = 10: the ball touches
    // the ridge line, not a face (each face leans 45 degrees away).
    let mesh = vec![
        Triangle::new(p(-20.0, -20.0, -10.0), p(20.0, -20.0, -10.0), p(20.0, 0.0, 10.0)),
        Triangle::new(p(-20.0, -20.0, -10.0), p(20.0, 0.0, 10.0), p(-20.0, 0.0, 10.0)),
        Triangle::new(p(-20.0, 0.0, 10.0), p(20.0, 0.0, 10.0), p(20.0, 20.0, -10.0)),
        Triangle::new(p(-20.0, 0.0, 10.0), p(20.0, 20.0, -10.0), p(-20.0, 20.0, -10.0)),
    ];
    let k = RadialDrop::new(&mesh, &x_axis(), 2.0).expect("a valid kernel");
    let rho = k.centre_radius(0.0, up_angle(&k)).expect("the ridge is under the ray");
    assert!((rho - 12.0).abs() < 1e-9, "edge contact at 10 + r, got {rho}");
}

#[test]
fn a_vertex_contact_rests_the_ball_on_the_tip() {
    // A pyramid tip at z = 10 on the ray, its faces steeper than the
    // ball can reach: the ball touches the apex vertex alone.
    let apex = p(0.0, 0.0, 10.0);
    let base = [p(-1.0, -1.0, 0.0), p(1.0, -1.0, 0.0), p(1.0, 1.0, 0.0), p(-1.0, 1.0, 0.0)];
    let mesh: Vec<Triangle> = (0..4).map(|i| Triangle::new(base[i], base[(i + 1) % 4], apex)).collect();
    let k = RadialDrop::new(&mesh, &x_axis(), 4.0).expect("a valid kernel");
    let rho = k.centre_radius(0.0, up_angle(&k)).expect("the tip is under the ray");
    assert!((rho - 14.0).abs() < 1e-9, "vertex contact at 10 + r, got {rho}");
}

#[test]
fn the_screw_rotor_drop_touches_and_never_gouges() {
    // The reporter's part: helical lobes, no common axis. At every
    // sampled (X, A) station the ball centre touches the model (its
    // nearest distance is r), and no centre farther out on the same
    // ray comes nearer than r - the ball arrived from outside without
    // cutting in.
    let mesh = screw(48, 40, 80.0, 1.0);
    let r = 3.0;
    let k = RadialDrop::new(&mesh, &x_axis(), r).expect("a valid kernel");
    let mut checked = 0;
    for i in 0..9 {
        let s = -36.0 + 9.0 * f64::from(i);
        for j in 0..24 {
            let a = TAU * f64::from(j) / 24.0;
            let rho = k.centre_radius(s, a).expect("the rotor is under every ray inside its length");
            let touch = mesh_dist(centre(&k, s, a, rho), &mesh);
            assert!((touch - r).abs() < 1e-7, "station ({s}, {a}): the ball does not touch ({touch})");
            for step in 1..=200 {
                let out = rho + 0.05 * f64::from(step);
                let d = mesh_dist(centre(&k, s, a, out), &mesh);
                assert!(d >= r - 1e-9, "station ({s}, {a}): a centre at {out} gouges ({d} < {r})");
            }
            checked += 1;
        }
    }
    assert_eq!(checked, 9 * 24);
}

#[test]
fn the_kernel_agrees_with_a_brute_force_march() {
    // An independent oracle: march the ball in from far outside in
    // small steps, then bisect the first touch. The exact kernel and
    // the march agree on the screw rotor.
    let mesh = screw(32, 24, 60.0, 0.75);
    let r = 2.5;
    let k = RadialDrop::new(&mesh, &x_axis(), r).expect("a valid kernel");
    for (s, a) in [(-20.0, 0.3), (-5.0, 1.9), (0.0, 3.0), (7.5, 4.4), (22.0, 5.9)] {
        let rho = k.centre_radius(s, a).expect("inside the length");
        let f = |t: f64| mesh_dist(centre(&k, s, a, t), &mesh) - r;
        let mut hi = 40.0;
        assert!(f(hi) > 0.0, "the march starts in air");
        let mut lo = hi;
        while f(lo) > 0.0 {
            hi = lo;
            lo -= 0.01;
        }
        for _ in 0..60 {
            let mid = 0.5 * (lo + hi);
            if f(mid) > 0.0 { hi = mid } else { lo = mid }
        }
        assert!((rho - hi).abs() < 1e-6, "station ({s}, {a}): kernel {rho} vs march {hi}");
    }
}

#[test]
fn a_ray_past_the_part_meets_nothing() {
    // The rotor ends at X = +-30; a station 10 mm past its end, with
    // a 2 mm ball, meets no facet: no contact, never a made-up radius.
    let mesh = screw(24, 12, 60.0, 0.5);
    let k = RadialDrop::new(&mesh, &x_axis(), 2.0).expect("a valid kernel");
    assert_eq!(k.centre_radius(40.0, 1.0), None);
    assert!(k.centre_radius(29.0, 1.0).is_some(), "the ball still reaches the end cap 1 mm inside");
}

#[test]
fn a_facet_reaches_every_station_within_the_ball_radius() {
    // A 1 mm strip at z = 10 (x in 0..1) and a far facet that makes
    // the axial cull cut the part into many bins. Every station whose
    // ball reaches the strip's rim from the side - up to r before and
    // after it - finds the contact and touches; none is culled away.
    let r = 3.0;
    let mesh = vec![
        Triangle::new(p(0.0, -5.0, 10.0), p(1.0, -5.0, 10.0), p(1.0, 5.0, 10.0)),
        Triangle::new(p(0.0, -5.0, 10.0), p(1.0, 5.0, 10.0), p(0.0, 5.0, 10.0)),
        Triangle::new(p(100.0, -5.0, 10.0), p(101.0, -5.0, 10.0), p(101.0, 5.0, 10.0)),
    ];
    let k = RadialDrop::new(&mesh, &x_axis(), r).expect("a valid kernel");
    let up = up_angle(&k);
    for i in 0..68 {
        let s = -2.95 + 0.1 * f64::from(i);
        let rho = k.centre_radius(s, up).unwrap_or_else(|| panic!("station {s} is within r of the strip"));
        let touch = mesh_dist(centre(&k, s, up, rho), &mesh);
        assert!((touch - r).abs() < 1e-9, "station {s}: the ball does not touch ({touch})");
    }
    assert_eq!(k.centre_radius(-3.05, up), None, "past r before the strip nothing is reached");
}

#[test]
fn the_radial_is_a_unit_vector_square_to_the_axis() {
    let ax = Axis { origin: p(1.0, 2.0, 3.0), dir: p(0.0, 0.6, 0.8) };
    let mesh = vec![Triangle::new(p(0.0, 0.0, 0.0), p(1.0, 0.0, 0.0), p(0.0, 1.0, 0.0))];
    let k = RadialDrop::new(&mesh, &ax, 1.0).expect("a valid kernel");
    for j in 0..12 {
        let u = k.radial(TAU * f64::from(j) / 12.0);
        assert!((u.dot(u) - 1.0).abs() < 1e-12 && u.dot(ax.dir).abs() < 1e-12);
    }
}

#[test]
fn input_that_cannot_be_meant_is_refused_by_name() {
    let mesh = vec![Triangle::new(p(0.0, 0.0, 5.0), p(1.0, 0.0, 5.0), p(0.0, 1.0, 5.0))];
    let ax = x_axis();
    for r in [0.0, -1.0, f64::NAN, f64::INFINITY] {
        let e = RadialDrop::new(&mesh, &ax, r).err().expect("a bad radius refuses");
        assert!(e.to_string().contains("ball radius"), "{e}");
    }
    let e = RadialDrop::new(&[], &ax, 1.0).err().expect("no facets refuses");
    assert!(e.to_string().contains("no facets"), "{e}");
    let bad = Axis { origin: p(0.0, 0.0, 0.0), dir: p(0.0, 0.0, 0.0) };
    let e = RadialDrop::new(&mesh, &bad, 1.0).err().expect("a zero axis refuses");
    assert!(e.to_string().contains("axis"), "{e}");
    let nan = vec![Triangle::new(p(f64::NAN, 0.0, 5.0), p(1.0, 0.0, 5.0), p(0.0, 1.0, 5.0))];
    let e = RadialDrop::new(&nan, &ax, 1.0).err().expect("a non-finite facet refuses");
    assert!(e.to_string().contains("not finite"), "{e}");
    let k = RadialDrop::new(&mesh, &ax, 1.0).expect("a valid kernel");
    assert_eq!(k.centre_radius(f64::NAN, 0.0), None, "a non-finite station has no contact");
}
