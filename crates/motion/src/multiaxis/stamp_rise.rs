//! The swarf exit rise (#48): a raster line's climb off its last
//! contact to the clearance plane, posed as a RAPID under the pose
//! the line cut with, wherever that is proved to be air.
//!
//! The base raster leaves every line with a straight feed up Z to the
//! clearance plane. Its end stands in air, where the stamp states no
//! lean (a lean is a claim of contact: the verifier's posed-contact
//! law reads a posed feed in air as a cut standing off its surface),
//! so a swarf program that lay along the blade turned upright on the
//! rise - a turn past the link threshold on every raster end, taken
//! through a retract / swing / re-enter air link, and back again on
//! the next line's plunge. The ball's path up the rise does not
//! depend on the pose (a sphere about its own centre); only the flank
//! does. So where
//!
//! - the station is a feed straight up Z (no XY travel),
//! - its end ball stands in air (past the contact band),
//! - the held pose is a lean, not under level, whose flank is proved
//!   in air at the rise's start ([`flank_clear`]),
//! - and the whole tool - ball and flank - never comes nearer the part
//!   on the way up than it stood at the rise's start (the link law's
//!   own floor: `min(d0, r)`, with `d0` the start pose's own gap).
//!   This is measured exactly, not sampled (#4315): the tool's axis
//!   swept up the rise is a parallelogram, and its distance to each
//!   facet is the [`super::stamp_sweep`] closed form. A sampled proof
//!   let a facet edge or vertex between two samples into the swept
//!   tool, and the verifier's rapid law then blocked the post.
//!
//! the rise is a rapid under the held pose: no turn at all, so no link
//! (the lying pose then rides the clearance rapid that follows, as
//! every rapid inherits its pose, and the next line's plunge states
//! its own). A rapid is not a cut and claims no contact; the
//! verifier's posed-rapid solid law judges its swept stack as it does
//! every link retract's. Anything not proved keeps the upright feed,
//! and the link law takes its turn through air as before.
//!
//! The plan's own RAPID rises (the side pass's retract off each level)
//! answer to the same proof (#4315). Each rapid inherits the held lean,
//! so before this law nothing proved that the flank, or the ball under
//! the tube's equator, rose clear. A plan rapid rise that the proof
//! refuses cannot turn upright where it stands: under the equator, an
//! upright flute at that ball centre is inside the tube. So it leaves
//! along the contact normal first, under the same pose, and rises from
//! there ([`normal_exit`]), with each leg proved by the same exact sweep.
//! Where no exit is proved, it stays the plan's own, and the verifier's
//! rapid law judges it.
//!
//! Provenance: retract-in-air under the cutting pose, with no tool
//! axis change off the surface, is general multi-axis practice; the
//! flank proof is the stamp's own. Our own design notes; no vendor
//! material consulted.

use crate::cl::{is_vertical, MoveKind};
use crate::finishing3d::closest_pt_tri;
use crate::geometry::Point3;

use super::stamp::Station;
use super::stamp_link::{contact_normal, flank_clear};
use super::stamp_pose::{Mesh, Ruled};
use super::stamp_sweep::quad_gap;

/// What a swarf station posts as once a lean is held (module header):
/// its kind, and the ball centres of an exit it takes first. A rise
/// proved in air is a rapid under `held`. A plan RAPID straight up
/// that is not proved leaves along the contact normal first, still
/// under `held`, where [`normal_exit`] proves that exit (#4315: the
/// plan's own retract rapids under the lean swept the flank and,
/// under the tube's equator, the ball through the blade). Every other
/// station is the plan's own, and the verifier's rapid law judges it.
pub(super) fn rise_kind(
    s: &Station,
    mesh: &Mesh,
    r: f64,
    band: f64,
    rl: Ruled,
    held: [f64; 3],
) -> (MoveKind, Option<[Point3; 2]>) {
    if rapid_rise(s, mesh, r, band, rl, held) {
        (MoveKind::Rapid, None)
    } else if s.kind == MoveKind::Rapid {
        (MoveKind::Rapid, normal_exit(s, mesh, r, band, rl, held))
    } else {
        (s.kind, None)
    }
}

/// The rise of a station that travels straight up Z (no XY travel),
/// or None.
fn straight_up(s: &Station) -> Option<f64> {
    let (dx, dy, dz) = (s.to.x - s.from.x, s.to.y - s.from.y, s.to.z - s.from.z);
    (dz > 1e-9 && dx.hypot(dy) <= 1e-9 && dz.is_finite()).then_some(dz)
}

/// The rise's preconditions (module header), and the floor no part of
/// the swept tool may come under: the start pose's own gap, at most
/// `r`. None where the station is no lying exit rise.
fn rise_floor(s: &Station, mesh: &Mesh, r: f64, band: f64, rl: Ruled, held: [f64; 3]) -> Option<f64> {
    if !matches!(s.kind, MoveKind::Feed | MoveKind::Rapid) || is_vertical(&held) || held[2] < -1e-9 {
        return None;
    }
    straight_up(s)?;
    let d0 = nearest(s.from, mesh);
    if !(d0.is_finite() && nearest(s.to, mesh) > r + band) {
        return None;
    }
    if !flank_clear(s.from, held, r, rl.tol, mesh.facets, mesh.boxes, &mesh.bounds) {
        return None;
    }
    let len = column(s.from, held, r, &mesh.bounds)?;
    let u = Point3::new(held[0] * len, held[1] * len, held[2] * len);
    let zero = Point3::new(0.0, 0.0, 0.0);
    let start = quad_gap(s.from, u, zero, mesh.facets, mesh.boxes, r, f64::NEG_INFINITY);
    Some(start.min(d0).min(r) - 1e-6)
}

/// Whether the tool posed along `a` with its ball centre at `o`,
/// moved straight to `o + v`, stays at least `floor` from the part
/// over the whole move: the exact sweep of [`super::stamp_sweep`].
fn sweep_clear(o: Point3, v: Point3, a: [f64; 3], mesh: &Mesh, r: f64, floor: f64) -> bool {
    let end = Point3::new(o.x + v.x, o.y + v.y, o.z + v.z);
    // The axis past the part's shell grown by `r` stands more than `r`
    // from every facet, so the column ends there; the longer of the
    // two ends' columns covers every point of a straight move.
    let (Some(l0), Some(l1)) = (column(o, a, r, &mesh.bounds), column(end, a, r, &mesh.bounds)) else {
        return false;
    };
    let len = l0.max(l1);
    let u = Point3::new(a[0] * len, a[1] * len, a[2] * len);
    quad_gap(o, u, v, mesh.facets, mesh.boxes, floor, floor) >= floor
}

/// Whether station `s` (a plan feed or rapid) is an exit rise the
/// swarf stamp may run as a rapid under `held` (module header). `band`
/// is the stamp's contact band, `r` the ball radius.
pub(super) fn rapid_rise(s: &Station, mesh: &Mesh, r: f64, band: f64, rl: Ruled, held: [f64; 3]) -> bool {
    let Some(floor) = rise_floor(s, mesh, r, band, rl, held) else {
        return false;
    };
    sweep_clear(s.from, s.to - s.from, held, mesh, r, floor)
}

/// The exit of a plan rapid rise that [`rapid_rise`] refused (#4315):
/// out along the contact normal by `h`, straight up to the rise's top
/// height, then across to the rise's own top - all under `held`, and
/// each leg proved by [`sweep_clear`] against the start's floor. The
/// shortest `h` of a doubling ladder (r/16 up to 4r) that proves all
/// three legs wins. The two via centres, or None (no exit proved).
pub(super) fn normal_exit(
    s: &Station,
    mesh: &Mesh,
    r: f64,
    band: f64,
    rl: Ruled,
    held: [f64; 3],
) -> Option<[Point3; 2]> {
    if s.kind != MoveKind::Rapid {
        return None;
    }
    let floor = rise_floor(s, mesh, r, band, rl, held)?;
    let (n, _) = contact_normal(s.from, mesh.facets, mesh.boxes);
    let n = Point3::new(n[0], n[1], n[2]);
    (0..=6).map(|i| r / 16.0 * f64::from(1u32 << i)).find_map(|h| {
        let v = Point3::new(n.x * h, n.y * h, n.z * h);
        let out = Point3::new(s.from.x + v.x, s.from.y + v.y, s.from.z + v.z);
        let top = Point3::new(out.x, out.y, s.to.z);
        let clear = out.z < s.to.z
            && sweep_clear(s.from, v, held, mesh, r, floor)
            && sweep_clear(out, top - out, held, mesh, r, floor)
            && sweep_clear(top, s.to - top, held, mesh, r, floor);
        clear.then_some([out, top])
    })
}

/// How far along `a` from `c` the axis stays inside the part's shell
/// grown by `r` (zero when it never enters). None for an axis that
/// frames no exit (a zero or non-finite pose): fail closed.
fn column(c: Point3, a: [f64; 3], r: f64, bounds: &[f64; 6]) -> Option<f64> {
    let p = [c.x, c.y, c.z];
    let mut exit = f64::INFINITY;
    for i in 0..3 {
        let (lo, hi) = (bounds[i] - r, bounds[i + 3] + r);
        if a[i] > 1e-12 {
            exit = exit.min((hi - p[i]) / a[i]);
        } else if a[i] < -1e-12 {
            exit = exit.min((lo - p[i]) / a[i]);
        } else if p[i] < lo || p[i] > hi {
            return Some(0.0);
        }
    }
    exit.is_finite().then_some(exit.max(0.0))
}

/// The distance from `p` to the nearest facet (infinite with none).
fn nearest(p: Point3, mesh: &Mesh) -> f64 {
    let mut best = f64::INFINITY;
    for (f, bx) in mesh.facets.iter().zip(mesh.boxes) {
        let gx = (bx[0] - p.x).max(0.0).max(p.x - bx[3]);
        let gy = (bx[1] - p.y).max(0.0).max(p.y - bx[4]);
        let gz = (bx[2] - p.z).max(0.0).max(p.z - bx[5]);
        if (gx * gx + gy * gy + gz * gz).sqrt() >= best {
            continue;
        }
        let q = closest_pt_tri(p, &[f.a, f.b, f.c]);
        let (ex, ey, ez) = (p.x - q.x, p.y - q.y, p.z - q.z);
        best = best.min((ex * ex + ey * ey + ez * ez).sqrt());
    }
    best
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::Triangle;

    const R: f64 = 3.0;
    const LYING: [f64; 3] = [1.0, 0.0, 0.0];

    fn p(x: f64, y: f64, z: f64) -> Point3 {
        Point3::new(x, y, z)
    }

    /// A 40 mm floor at Z 0, and (with `wall`) a wall standing across
    /// +X at X 10, up to Z 40.
    fn part(wall: bool) -> Vec<Triangle> {
        let mut f = vec![
            Triangle::new(p(-20.0, -20.0, 0.0), p(20.0, -20.0, 0.0), p(20.0, 20.0, 0.0)),
            Triangle::new(p(-20.0, -20.0, 0.0), p(20.0, 20.0, 0.0), p(-20.0, 20.0, 0.0)),
        ];
        if wall {
            f.push(Triangle::new(p(10.0, -20.0, 0.0), p(10.0, 20.0, 0.0), p(10.0, 20.0, 40.0)));
            f.push(Triangle::new(p(10.0, -20.0, 0.0), p(10.0, 20.0, 40.0), p(10.0, -20.0, 40.0)));
        }
        f
    }

    fn rise(facets: &[Triangle], kind: MoveKind, to: Point3, held: [f64; 3]) -> bool {
        posted(facets, kind, to, held).0
    }

    /// Whether `rapid_rise` proves the station, and what `rise_kind`
    /// posts it as (last cutting feed 500).
    fn posted(facets: &[Triangle], kind: MoveKind, to: Point3, held: [f64; 3]) -> (bool, (MoveKind, Option<[Point3; 2]>)) {
        posted_from(facets, p(0.0, 0.0, R), kind, to, held)
    }

    /// A tube of radius 5 along X (X -20 to 20, 32 sides), axis at the
    /// origin.
    fn tube_part() -> Vec<Triangle> {
        let n = 32;
        let at = |k: usize, x: f64| {
            let t = std::f64::consts::TAU * k as f64 / n as f64;
            p(x, 5.0 * t.cos(), 5.0 * t.sin())
        };
        (0..n)
            .flat_map(|k| {
                let (a, b, c, d) = (at(k, -20.0), at(k + 1, -20.0), at(k + 1, 20.0), at(k, 20.0));
                [Triangle::new(a, b, c), Triangle::new(a, c, d)]
            })
            .collect()
    }

    fn posted_from(
        facets: &[Triangle],
        from: Point3,
        kind: MoveKind,
        to: Point3,
        held: [f64; 3],
    ) -> (bool, (MoveKind, Option<[Point3; 2]>)) {
        let boxes: Vec<[f64; 6]> = facets
            .iter()
            .map(|f| {
                [
                    f.a.x.min(f.b.x.min(f.c.x)),
                    f.a.y.min(f.b.y.min(f.c.y)),
                    f.a.z.min(f.b.z.min(f.c.z)),
                    f.a.x.max(f.b.x.max(f.c.x)),
                    f.a.y.max(f.b.y.max(f.c.y)),
                    f.a.z.max(f.b.z.max(f.c.z)),
                ]
            })
            .collect();
        let mesh = Mesh::new(facets, &boxes);
        let ruled = vec![true; facets.len()];
        let rl = Ruled { dir: p(1.0, 0.0, 0.0), ruled: &ruled, tol: 0.1 };
        let feed = (kind != MoveKind::Rapid).then_some(500.0);
        let s = Station { kind, feed, from, to, mv: 0 };
        (rapid_rise(&s, &mesh, R, 0.1, rl, held), rise_kind(&s, &mesh, R, 0.1, rl, held))
    }

    #[test]
    fn a_plan_rapid_up_from_the_contact_must_prove_its_rise_too() {
        // #4315: the plan's own retract rapid straight up off a cut,
        // under the held lean. Proved in air, it stays a plain rapid.
        let up = p(0.0, 0.0, 28.0);
        assert_eq!(posted(&part(false), MoveKind::Rapid, up, LYING), (true, (MoveKind::Rapid, None)));
        // A ledge's edge hangs 2.6 mm off the rise's line, over the
        // start: straight up the ball cuts it. The plan rapid leaves
        // along the contact normal (up, off the floor) only as far as
        // proves clear, so here no exit is proved and it stays the
        // plan's own, for the verifier's rapid law to judge.
        let mut f = part(false);
        f.push(Triangle::new(p(-2.6, -20.0, 6.0), p(-2.6, 20.0, 6.0), p(-20.0, 0.0, 6.0)));
        assert_eq!(posted(&f, MoveKind::Rapid, up, [-1.0, 0.0, 0.0]), (false, (MoveKind::Rapid, None)));
        // A ball under the equator of a tube (radius 5, along X): its
        // contact normal points out and down, so the exit steps off
        // the tube before it rises.
        let tube = tube_part();
        let from = p(0.0, -0.6 * (5.0 + R), -0.8 * (5.0 + R));
        let (proved, (kind, via)) = posted_from(&tube, from, MoveKind::Rapid, p(from.x, from.y, 28.0), LYING);
        assert!(!proved, "straight up climbs through the tube");
        assert_eq!(kind, MoveKind::Rapid);
        let [out, top] = via.expect("an exit along the normal is proved");
        assert!(out.y < from.y && top.z == 28.0 && top.x == out.x && top.y == out.y, "{out:?} {top:?}");
        // A plan rapid that does not rise straight up is the plan's own.
        let over = p(1.0, 0.0, 28.0);
        assert_eq!(posted(&part(true), MoveKind::Rapid, over, LYING), (false, (MoveKind::Rapid, None)));
        // A feed rise that is not proved keeps its feed (the link law).
        assert_eq!(posted(&part(true), MoveKind::Feed, up, LYING), (false, (MoveKind::Feed, None)));
    }

    #[test]
    fn a_rise_into_air_under_a_clear_lean_is_a_rapid() {
        assert!(rise(&part(false), MoveKind::Feed, p(0.0, 0.0, 28.0), LYING));
    }

    #[test]
    fn a_rise_whose_flank_meets_a_wall_keeps_its_feed() {
        // The lying flank points +X into the wall 10 mm off: the link
        // law takes the turn through air instead.
        assert!(!rise(&part(true), MoveKind::Feed, p(0.0, 0.0, 28.0), LYING));
    }

    #[test]
    fn a_rise_that_brushes_an_edge_between_samples_keeps_its_feed() {
        // #4315: a fin's knife edge 2.95 mm off the rise's line, at a
        // height between two of the old r/2 samples (Z 3 and 4.47
        // for this 25 mm rise). Each sample stands 3.04 from it, but
        // the ball passes 0.05 mm inside it on the way up. The lean
        // points -X, away from the fin, so only the ball meets it.
        let mut f = part(false);
        let z = R + 25.0 / 17.0 * 0.5;
        f.push(Triangle::new(p(2.95, -20.0, z), p(2.95, 20.0, z), p(10.0, 0.0, z)));
        let away = [-1.0, 0.0, 0.0];
        assert!(rise(&part(false), MoveKind::Feed, p(0.0, 0.0, 28.0), away), "control: no fin");
        assert!(!rise(&f, MoveKind::Feed, p(0.0, 0.0, 28.0), away), "the ball's sweep cuts the fin's edge");
    }

    #[test]
    fn a_rise_whose_flank_sweeps_a_vertex_between_samples_keeps_its_feed() {
        // #4315: a flat spike whose tip points at the lying flank from
        // the side, 2.95 mm off its axis. The tip stands at one of the
        // old Z samples, but between the old axial samples (the ball
        // centre at X 0, the first flank sample at X -1.44), each of
        // which is 3.03 from it. The flank's swept band reaches it.
        let mut f = part(false);
        let z = R + 25.0 / 17.0 * 5.0;
        f.push(Triangle::new(p(-0.72, 2.95, z), p(-5.0, 10.0, z), p(3.0, 10.0, z)));
        let away = [-1.0, 0.0, 0.0];
        assert!(!rise(&f, MoveKind::Feed, p(0.0, 0.0, 28.0), away), "the flank rises into the spike");
    }

    #[test]
    fn only_a_straight_feed_up_from_a_lean_into_air_qualifies() {
        let f = part(false);
        assert!(!rise(&f, MoveKind::Feed, p(1.0, 0.0, 28.0), LYING), "XY travel");
        assert!(!rise(&f, MoveKind::Plunge, p(0.0, 0.0, 28.0), LYING), "a plunge");
        assert!(!rise(&f, MoveKind::Feed, p(0.0, 0.0, 28.0), [0.0, 0.0, 1.0]), "upright held");
        assert!(!rise(&f, MoveKind::Feed, p(0.0, 0.0, 28.0), [0.6, 0.0, -0.8]), "under level");
        assert!(!rise(&f, MoveKind::Feed, p(0.0, 0.0, R + 0.05), LYING), "ends in contact");
        assert!(!rise(&[], MoveKind::Feed, p(0.0, 0.0, 28.0), LYING), "no part to prove against");
    }
}
