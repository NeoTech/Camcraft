//! Lathe chucks generated from their figures (#1442): the envelope each
//! chuck's shells stand in (body diameter and length, jaw reach and
//! height, collet nose), the mesh law reads them as closed solids, and
//! every nonsense figure is refused by name.
//!
//! Gate map (M4.4): covers camcraft_sim::chuck.

use camcraft_sim::chuck::{self, ColletChuck, JawChuck};
use camcraft_sim::fixture_mesh::FixtureMesh;
use camcraft_sim::geometry::{Point3, Triangle};

fn three_jaw() -> JawChuck {
    JawChuck {
        body_diameter: 160.0,
        body_length: 70.0,
        bore_diameter: 40.0,
        grip_diameter: 40.0,
        jaw_length: 35.0,
        jaw_width: 20.0,
        jaw_height: 30.0,
    }
}

fn collet() -> ColletChuck {
    ColletChuck { body_diameter: 100.0, body_length: 40.0, nose_diameter: 50.0, nose_length: 30.0, collet_bore: 20.0 }
}

fn bounds(t: &[Triangle]) -> (Point3, Point3) {
    let mut lo = Point3::new(f64::INFINITY, f64::INFINITY, f64::INFINITY);
    let mut hi = Point3::new(f64::NEG_INFINITY, f64::NEG_INFINITY, f64::NEG_INFINITY);
    for p in t.iter().flat_map(|t| [t.a, t.b, t.c]) {
        lo = Point3::new(lo.x.min(p.x), lo.y.min(p.y), lo.z.min(p.z));
        hi = Point3::new(hi.x.max(p.x), hi.y.max(p.y), hi.z.max(p.z));
    }
    (lo, hi)
}

fn max_radius(t: &[Triangle]) -> f64 {
    t.iter().flat_map(|t| [t.a, t.b, t.c]).map(|p| p.x.hypot(p.y)).fold(0.0, f64::max)
}

#[test]
fn three_jaw_envelope_is_pinned() {
    let c = three_jaw();
    c.check("3-jaw chuck", 3).expect("default figures pass");
    let t = c.solid(3);
    let (lo, hi) = bounds(&t);
    assert!((lo.z + 70.0).abs() < 1e-9, "body length 70 below the face, got {}", lo.z);
    assert!((hi.z - 30.0).abs() < 1e-9, "jaws stand 30 above the face, got {}", hi.z);
    // The drawn body encloses the true 80 mm radius, within one flat's sag.
    let sag = 80.0 / (std::f64::consts::PI / chuck::SIDES as f64).cos();
    let r = max_radius(&t);
    assert!(r >= 80.0 && r <= sag + 1e-9, "body radius drawn {r}");
    let m = FixtureMesh::build(&t).expect("closed shells");
    // Steel: just inside the body's rim, in the body under the face, a
    // jaw's tip (the first jaw on +x reaches 20 + 35 = 55).
    assert!(m.contains(Point3::new(79.5, 0.0, -35.0)), "body rim");
    assert!(m.contains(Point3::new(0.0, 50.0, -1.0)), "body under the face");
    assert!(m.contains(Point3::new(54.5, 0.0, 15.0)), "first jaw's tip");
    assert!(m.contains(Point3::new(20.5, 0.0, 29.5)), "first jaw's gripping face");
    // Air: past the rim, past the jaw tip, the bore, the work's room
    // between the jaws, and above the jaws.
    assert!(!m.contains(Point3::new(81.0, 0.0, -35.0)), "past the rim");
    assert!(!m.contains(Point3::new(56.0, 0.0, 15.0)), "past the jaw tip");
    assert!(!m.contains(Point3::new(0.0, 0.0, -35.0)), "the through bore");
    assert!(!m.contains(Point3::new(19.0, 0.0, 15.0)), "the work's room");
    assert!(!m.contains(Point3::new(54.0, 0.0, 31.0)), "above the jaws");
    // The other two jaws stand 120 degrees round.
    let a = std::f64::consts::TAU / 3.0;
    assert!(m.contains(Point3::new(50.0 * a.cos(), 50.0 * a.sin(), 15.0)), "second jaw");
    assert!(!m.contains(Point3::new(-50.0, 0.0, 15.0)), "no jaw at 180 degrees on a 3-jaw");
}

#[test]
fn four_jaw_has_four_jaws_at_quarters() {
    let c = JawChuck { body_diameter: 200.0, jaw_length: 45.0, ..three_jaw() };
    c.check("4-jaw chuck", 4).expect("passes");
    let m = FixtureMesh::build(&c.solid(4)).expect("closed");
    for p in [(50.0, 0.0), (0.0, 50.0), (-50.0, 0.0), (0.0, -50.0)] {
        assert!(m.contains(Point3::new(p.0, p.1, 15.0)), "jaw at {p:?}");
    }
    let d = 50.0 / 2f64.sqrt();
    assert!(!m.contains(Point3::new(d, d, 15.0)), "no jaw at 45 degrees");
}

#[test]
fn collet_envelope_is_pinned() {
    let c = collet();
    c.check("Collet chuck").expect("passes");
    let t = c.solid();
    let (lo, hi) = bounds(&t);
    assert!((lo.z + 70.0).abs() < 1e-9 && hi.z.abs() < 1e-9, "z {}..{}", lo.z, hi.z);
    let m = FixtureMesh::build(&t).expect("closed shells");
    assert!(m.contains(Point3::new(24.5, 0.0, -15.0)), "nose");
    assert!(!m.contains(Point3::new(26.0, 0.0, -15.0)), "beside the nose");
    assert!(m.contains(Point3::new(49.5, 0.0, -50.0)), "body");
    assert!(!m.contains(Point3::new(0.0, 0.0, -50.0)), "the collet bore");
    assert!(!m.contains(Point3::new(0.0, 0.0, 5.0)), "above the nose");
}

#[test]
fn jaw_chuck_nonsense_is_refused_by_name() {
    let bad = |c: JawChuck, jaws: u32, needle: &str| {
        let e = c.check("3-jaw chuck", jaws).expect_err(needle);
        assert!(e.contains("3-jaw chuck") && e.contains(needle), "{e}");
    };
    bad(JawChuck { body_diameter: 0.0, ..three_jaw() }, 3, "body diameter must be more than zero");
    bad(JawChuck { body_length: -5.0, ..three_jaw() }, 3, "body length must be more than zero");
    bad(JawChuck { jaw_width: f64::NAN, ..three_jaw() }, 3, "jaw width must be a finite length");
    bad(JawChuck { jaw_height: 1e9, ..three_jaw() }, 3, "jaw height (above the face)");
    bad(JawChuck { bore_diameter: 160.0, ..three_jaw() }, 3, "through bore");
    bad(JawChuck { jaw_length: 61.0, ..three_jaw() }, 3, "past the body");
    bad(JawChuck { jaw_width: 70.0, ..three_jaw() }, 3, "would meet each other");
    bad(three_jaw(), 1, "2 to 12 jaws");
}

#[test]
fn collet_nonsense_is_refused_by_name() {
    let bad = |c: ColletChuck, needle: &str| {
        let e = c.check("Collet chuck").expect_err(needle);
        assert!(e.contains("Collet chuck") && e.contains(needle), "{e}");
    };
    bad(ColletChuck { collet_bore: 50.0, ..collet() }, "collet bore");
    bad(ColletChuck { nose_diameter: 120.0, ..collet() }, "nose diameter");
    bad(ColletChuck { nose_length: f64::INFINITY, ..collet() }, "nose length must be a finite length");
}

#[test]
fn round_stock_fits_and_square_stock_is_refused() {
    let (d, at) = chuck::round_stock(Point3::new(-12.0, -12.0, -16.0), Point3::new(12.0, 12.0, 20.0)).expect("round");
    assert!((d - 24.0).abs() < 1e-12 && at.x.abs() < 1e-12 && at.y.abs() < 1e-12 && (at.z + 16.0).abs() < 1e-12);
    let e = chuck::round_stock(Point3::new(0.0, 0.0, 0.0), Point3::new(30.0, 20.0, 10.0)).expect_err("not round");
    assert!(e.contains("only round stock"), "{e}");
    assert!(chuck::round_stock(Point3::new(0.0, 0.0, 0.0), Point3::new(f64::NAN, 1.0, 1.0)).is_err());
    assert_eq!(three_jaw().gripping(24.0).grip_diameter, 24.0);
    assert_eq!(collet().gripping(18.0).collet_bore, 18.0);
}
