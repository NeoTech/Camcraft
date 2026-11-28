//! #3908: the flute rule of the 3D tool preview - the count, the helix,
//! the plain kinds, the facet bound, and that every fluted point stays
//! inside the conservative revolved envelope the laws read.
//! Provenance: our own tests over the shipped catalog tools.

use super::*;
use camcraft_app::tools::catalog;

fn shipped(name: &str) -> Tool {
    match catalog().into_iter().find(|(n, _)| *n == name) {
        Some((_, t)) => t,
        None => panic!("{name} is a shipped tool"),
    }
}

/// `name` with `n` flutes and the helix `helix` degrees.
fn with(name: &str, n: Option<u32>, helix: Option<f64>) -> Tool {
    let mut t = shipped(name);
    t.flutes = n;
    t.catalog.helix_deg = helix;
    t
}

/// The steel meridian as the preview revolves it: the profile to the
/// shank top, closed on the axis.
fn steel(t: &Tool) -> Vec<(f64, f64)> {
    let mut p = Tool { holder_len: 0.0, ..t.clone() }.profile();
    let top = t.flute_len + t.shank_len;
    p.push((top, 0.0));
    p
}

/// The vertices `(angle, radius)` of the ring nearest height `z`.
fn ring(mesh: &[Triangle], z: f64) -> Vec<(f64, f64)> {
    let pts: Vec<Point3> = mesh.iter().flat_map(|t| [t.a, t.b, t.c]).collect();
    let near = pts.iter().map(|p| p.z).min_by(|a, b| (a - z).abs().total_cmp(&(b - z).abs())).unwrap_or(z);
    let mut v: Vec<(f64, f64)> = pts
        .iter()
        .filter(|p| (p.z - near).abs() < 1e-9)
        .map(|p| (p.y.atan2(p.x).rem_euclid(TAU), p.x.hypot(p.y)))
        .collect();
    v.sort_by(|a, b| a.0.total_cmp(&b.0));
    v.dedup_by(|a, b| (a.0 - b.0).abs() < 1e-9);
    v
}

/// The grooves around a ring: the runs below the land radius `r`.
fn grooves(ring: &[(f64, f64)], r: f64) -> usize {
    let deep: Vec<bool> = ring.iter().map(|&(_, q)| q < r * 0.99).collect();
    (0..deep.len()).filter(|&i| deep[i] && !deep[(i + deep.len() - 1) % deep.len()]).count()
}

/// The angle of the deepest point of a ring.
fn floor_at(ring: &[(f64, f64)]) -> f64 {
    ring.iter().copied().min_by(|a, b| a.1.total_cmp(&b.1)).map_or(f64::NAN, |p| p.0)
}

#[test]
fn an_end_mill_shows_its_flute_count_on_the_fluted_length() {
    for n in [2, 3, 4, 6] {
        let t = with("EM-1000-4FL", Some(n), Some(30.0));
        let Some(m) = fluted(&t, &steel(&t)) else { panic!("{n} flutes are drawn") };
        let mid = ring(&m, t.flute_len * 0.5);
        assert_eq!(grooves(&mid, t.diameter * 0.5), n as usize, "{n} grooves at mid flute");
        let floor = mid.iter().map(|p| p.1).fold(f64::INFINITY, f64::min);
        assert!((floor - CORE * t.diameter * 0.5).abs() < 0.1, "the groove floor is the core: {floor}");
        let shank = ring(&m, t.flute_len + t.shank_len * 0.5);
        assert_eq!(grooves(&shank, t.shank_d * 0.5), 0, "the shank is plain");
    }
}

#[test]
fn helix_zero_cuts_straight_grooves_and_a_helix_twists_them() {
    let t = with("EM-1000-4FL", Some(4), Some(0.0));
    let m = fluted(&t, &steel(&t)).unwrap_or_default();
    let (z0, z1) = (t.flute_len * 0.2, t.flute_len * 0.7);
    let (a0, a1) = (floor_at(&ring(&m, z0)), floor_at(&ring(&m, z1)));
    assert!((a0 - a1).abs() < 1e-9, "a straight groove keeps its angle: {a0} {a1}");
    let unstated = with("EM-1000-4FL", Some(4), None);
    assert_eq!(fluted(&unstated, &steel(&unstated)), Some(m), "an unknown helix draws straight flutes");
    let h = with("EM-1000-4FL", Some(4), Some(30.0));
    let g = Grooves::of(&h).unwrap_or_else(|| panic!("the helix tool has grooves"));
    assert!((g.twist - 30f64.to_radians().tan() / 5.0).abs() < 1e-12, "the twist is tan(helix) / R");
    // A right-hand helix: the groove floor turns by the twist as it rises.
    let (b0, b1) = (g.scale(0.0, z0), g.scale(g.twist * (z1 - z0), z1));
    assert!((b0 - b1).abs() < 1e-9, "the groove follows the helix: {b0} {b1}");
}

#[test]
fn the_fluted_steel_stays_inside_the_revolved_envelope() {
    let names = ["EM-1000-4FL", "BB-1000-R1", "CM-0600-90", "TN-1600-0R8", "EM-0200-2FL"];
    for name in names.into_iter().chain(FLUTED.iter().map(|(n, _)| *n)) {
        let t = with(name, Some(3), Some(40.0));
        let prof = steel(&t);
        let Some(m) = fluted(&t, &prof) else { continue };
        let env = |z: f64| {
            prof.windows(2)
                .filter(|w| (w[0].0.min(w[1].0) - 1e-9..=w[0].0.max(w[1].0) + 1e-9).contains(&z))
                .map(|w| w[0].1.max(w[1].1))
                .fold(0.0, f64::max)
        };
        for p in m.iter().flat_map(|q| [q.a, q.b, q.c]) {
            assert!(p.x.hypot(p.y) <= env(p.z) + 1e-9, "{name}: {p:?} leaves the envelope");
        }
    }
}

#[test]
fn tools_without_flutes_draw_the_plain_model() {
    // #4452: the thread mill, the T-slot and the dovetail cutter draw
    // flutes now (`each_fluted_kind_shows_its_flutes`).
    for name in ["TD-1732-60", "FG-0600-04", "TN-1600-0R8", "BR-1000", "PRB-0400", "EDM-E10X6", "GT-0200"] {
        let t = with(name, Some(4), Some(30.0));
        assert_eq!(fluted(&t, &steel(&t)), None, "{name} ({:?}) draws no flutes", t.kind);
    }
    let em = |n| with("EM-1000-4FL", n, Some(30.0));
    for n in [None, Some(0), Some(MAX_FLUTES + 1)] {
        assert_eq!(fluted(&em(n), &steel(&em(n))), None, "{n:?} flutes draw none");
    }
}

#[test]
fn the_fluted_steel_keeps_to_its_facet_bound() {
    let names = ["EM-1000-4FL", "BB-1000-R1", "CM-0600-90", "EM-0200-2FL"];
    for name in names.into_iter().chain(FLUTED.iter().map(|(n, _)| *n)) {
        let t = with(name, Some(MAX_FLUTES), Some(80.0));
        let n = fluted(&t, &steel(&t)).map_or(0, |m| m.len());
        assert!(n > 0 && n <= MAX_TRIS, "{name}: {n} facets, bound {MAX_TRIS}");
    }
}

/// #4336: the side view's flute lines stay on the cutter (inside its
/// radius, under the run-out), slant with a helix, stand straight without
/// one, and are absent when the tool states no flute count.
#[test]
fn side_flute_lines_stay_on_the_cutter() {
    let t = with("EM-1000-4FL", Some(4), Some(30.0));
    let lines = side_lines(&t, &steel(&t)).unwrap_or_else(|| panic!("a 4-flute end mill shows flutes"));
    assert!(!lines.is_empty(), "the visible flutes draw lines");
    let r = t.diameter * 0.5;
    for p in lines.iter().flatten() {
        assert!(p.0 >= 0.0 && p.0 <= t.flute_len * (1.0 - RUN_OUT) + 1e-9, "a line point {p:?} leaves the fluted length");
        assert!(p.1.abs() <= r + 1e-9, "a line point {p:?} leaves the cutter radius {r}");
    }
    let slant = lines.iter().any(|l| l.windows(2).any(|w| (w[1].1 - w[0].1).abs() > 1e-6));
    assert!(slant, "a 30-degree helix slants its lines");
    let straight = with("EM-1000-4FL", Some(4), None);
    let lines = side_lines(&straight, &steel(&straight)).unwrap_or_else(|| panic!("a straight flute shows"));
    assert!(lines.iter().all(|l| l.windows(2).all(|w| (w[1].1 - w[0].1).abs() < 1e-9)), "a straight flute is a vertical line");
    assert!(side_lines(&with("EM-1000-4FL", None, Some(30.0)), &steel(&t)).is_none(), "no stated count, no lines");
    let many = with("EM-1000-4FL", Some(MAX_FLUTES), Some(60.0));
    let n: usize = side_lines(&many, &steel(&many)).unwrap_or_default().iter().map(Vec::len).sum();
    assert!(n <= MAX_FLUTES as usize * (SIDE_STEPS + 1), "the lines keep their bound: {n} points");
}

/// #4452: one shipped tool of each kind that has flutes, and the flute
/// count its preview draws (its stated count, or the kind's default).
const FLUTED: [(&str, u32); 13] = [
    ("EM-1000-4FL", 4),
    ("BN-0600", 2),
    ("BB-1000-R1", 2),
    ("DR-0600", 2),
    ("CM-0600-90", 1),
    ("TAP-M6", 3),
    ("TM-0600-P125", 3),
    ("TMS-0600-P050-150", 3),
    ("TS-2000-0600", 6),
    ("KS-1600-0300", 8),
    ("DT-2000-60", 6),
    ("LP-1000", 2),
    ("FM-CV-R6", 2),
];

/// #4452: each kind with a flute length shows its flutes in the 3D
/// preview - the grooves take radius away somewhere on the cutting part,
/// and never above it.
#[test]
fn each_fluted_kind_shows_its_flutes() {
    for (name, n) in FLUTED {
        let t = shipped(name);
        assert!(fluted_kind(t.kind), "{name} ({:?}) is a fluted kind", t.kind);
        let g = Grooves::of(&t).unwrap_or_else(|| panic!("{name} ({:?}) has grooves", t.kind));
        assert_eq!(g.n, n, "{name}: the flute count");
        let prof = steel(&t);
        let Some(m) = fluted(&t, &prof) else { panic!("{name} ({:?}) draws a flute mesh", t.kind) };
        let land = |z: f64| radius_at(&prof, z).unwrap_or(0.0);
        let cut = m.iter().flat_map(|q| [q.a, q.b, q.c]).any(|p| p.z < t.flute_len && p.x.hypot(p.y) < land(p.z) * 0.9);
        assert!(cut, "{name}: a groove cuts into the cutting part");
        let above = m.iter().flat_map(|q| [q.a, q.b, q.c]).filter(|p| p.z > t.flute_len + 1e-9);
        for p in above {
            assert!(on_meridian(&prof, p.z, p.x.hypot(p.y)), "{name}: {p:?} above the cutting part is plain");
        }
    }
}

/// `(z, r)` lies on the meridian polyline `prof`.
fn on_meridian(prof: &[(f64, f64)], z: f64, r: f64) -> bool {
    prof.windows(2).any(|w| {
        let ((z0, r0), (z1, r1)) = (w[0], w[1]);
        let (dz, dr) = (z1 - z0, r1 - r0);
        let len2 = dz * dz + dr * dr;
        let s = if len2 > 1e-18 { (((z - z0) * dz + (r - r0) * dr) / len2).clamp(0.0, 1.0) } else { 0.0 };
        (z0 + dz * s - z).hypot(r0 + dr * s - r) < 1e-6
    })
}

/// #4452: a twist drill or a tap that states no count draws the kind's
/// own (2 helical, 3 straight); a stated count wins; a milling cutter
/// still draws none without a count.
#[test]
fn a_drill_and_a_tap_without_a_count_draw_their_kinds_flutes() {
    let d = shipped("DR-0600");
    assert_eq!(d.flutes, None, "the shipped drill states no count");
    let g = Grooves::of(&d).unwrap_or_else(|| panic!("a drill has grooves"));
    assert_eq!(g.n, 2, "a twist drill has two flutes");
    assert!((g.twist - 30f64.to_radians().tan() / (d.diameter * 0.5)).abs() < 1e-12, "a 30-degree helix");
    let tap = shipped("TAP-M6");
    let g = Grooves::of(&tap).unwrap_or_else(|| panic!("a tap has grooves"));
    assert_eq!((g.n, g.twist), (3, 0.0), "a tap has three straight flutes");
    let mut stated = d.clone();
    stated.flutes = Some(3);
    assert_eq!(Grooves::of(&stated).map(|g| g.n), Some(3), "a stated count wins");
    assert_eq!(Grooves::of(&with("KS-1600-0300", None, None)), None, "a keyseat cutter without a count draws none");
}

/// #4452: the thread mill's toothed steel (#4451) takes its flutes too.
#[test]
fn a_thread_mill_cuts_its_flutes_into_its_teeth() {
    for name in ["TM-0600-P125", "TMS-0600-P050-150"] {
        let t = shipped(name);
        let plain = steel(&t);
        let teeth = super::super::thread_form::toothed(&t, &plain).unwrap_or_else(|| panic!("{name} has teeth"));
        let m = fluted(&t, &teeth).unwrap_or_else(|| panic!("{name}: the teeth take flutes"));
        assert!(!m.is_empty() && m.len() <= MAX_TRIS, "{name}: {} facets", m.len());
    }
}
