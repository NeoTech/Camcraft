//! The edge a no-pick 2D chamfer rides (#2435): the wall at the corner
//! depth, matched to the edge at the top plane.
//!
//! Provenance: the chamfer mill's cut law (machine-tool handbook
//! geometry, [`super::chamfer`]): the cone's tip stands ON the wall at
//! the corner depth `top - width * tan(angle)`, its generatrix on the
//! face. Where the wall is vertical the section at the top plane and
//! the section at the corner depth are one outline. Where the model
//! already carries a bevel (a modelled chamfer or round), the top
//! plane's outline is the bevel's TOP edge, standing inside the wall
//! by the bevel's width - a tip on it sits inside the part. The wall
//! the cone must touch is the section at the corner depth. Our own law
//! over this repo's sections; no vendor material consulted.

use crate::geo2d::{Loop, Point2, Region};
use crate::geometry::Point3;

use super::face::GenError;

/// Plan slack of the match between a top edge and its wall, mm: the
/// chord sag two tessellated sections of one rounded corner can
/// disagree by. A loose match only picks which outline to ride; the
/// ride itself is always the wall section, and the program sim holds
/// a 2D chamfer's tip to its wall (#2435) whatever was matched.
const MATCH_SLACK: f64 = 0.05;

/// Below this plan gap the wall IS the top edge (a vertical wall): the
/// top plane's own loop is ridden, unchanged, so every sharp-edged
/// part posts exactly what it did before #2435.
const SAME_EDGE: f64 = 1e-6;

/// The loops a no-pick 2D chamfer of `width` rides: for each loop of
/// `top` (the section at the top plane - the edges to break), the loop
/// of `corner` (the section at the corner depth - the wall the tip
/// touches) that every vertex of the top loop stands within `width` of,
/// in plan (one way only: a wall loop's mitred corner stands farther
/// than `width` from a bevel's top corner).
/// A vertical wall rides the top loop itself. A `corner` loop matched
/// to no top loop is a lower edge (a ledge, the plate under an island)
/// and is left alone: it is not an edge at the top plane.
/// #2596: when any top loop stands over a bevel, only the bevelled loops
/// are ridden; the sharp ones are edges the model leaves sharp.
///
/// # Errors
/// A top loop with no wall within `width` of it under it - a blind
/// pocket shallower than the chamfer's depth, an island whose wall
/// stands lower than that depth - refuses by name at that loop: the
/// tip would ride below the edge's floor, into the material (a part or
/// the stock, whichever the chamfer's source is - the text fits both, #2521).
/// #2887: a top loop bevelled on only part of its length (a modelled
/// chamfer on one side of a plate) refuses by name too: no whole
/// outline of it can be ridden.
pub fn ride_loops(top: &Region, corner: &Region, width: f64, corner_z: f64) -> Result<Region, GenError> {
    if !(width.is_finite() && width > 0.0 && corner_z.is_finite()) {
        return Err(GenError::Invalid("chamfer width must be positive".into()));
    }
    let reach = width + MATCH_SLACK;
    // #2596: the sharp edges (a vertical wall) and the bevelled ones (a
    // modelled chamfer or round) apart.
    let mut sharp: Vec<Loop> = Vec::with_capacity(top.loops().len());
    let mut out: Vec<Loop> = Vec::with_capacity(top.loops().len());
    for t in top.loops() {
        let wall = corner
            .loops()
            .iter()
            .filter(|c| c.is_ccw() == t.is_ccw())
            .map(|c| (c, gap(t.points(), c.points())))
            .filter(|(_, g)| *g <= reach)
            .min_by(|a, b| a.1.total_cmp(&b.1));
        // #2887: a loop bevelled on only part of its length has no whole
        // outline to ride - refused by name, whichever way its
        // tessellation would have read it.
        if let Some((c, _)) = wall
            && let Some((bevel, sharp)) = partly_bevelled(t.points(), c.points(), width)
        {
            return Err(GenError::At {
                msg: format!(
                    "the edge loop near X{:.3} Y{:.3} carries a modelled chamfer on only part of its length - it is sharp near X{:.3} Y{:.3} - a chamfer with no picks would cut the sharp part; pick the edges to break",
                    bevel.x, bevel.y, sharp.x, sharp.y
                ),
                at: Point3::new(bevel.x, bevel.y, corner_z),
            });
        }
        match wall {
            Some((_, g)) if g <= SAME_EDGE => sharp.push(t.clone()),
            Some((c, _)) => out.push(c.clone()),
            None => {
                let at = t.points().first().copied().unwrap_or(Point2::new(0.0, 0.0));
                return Err(GenError::At {
                    msg: format!(
                        "the edge near X{:.3} Y{:.3} has no wall under it down to the chamfer's depth Z{corner_z:.3} - a {width} mm chamfer there would ride into the material under it; lower the width or pick the edges to break",
                        at.x, at.y
                    ),
                    at: Point3::new(at.x, at.y, corner_z),
                });
            }
        }
    }
    // #2596: where the model carries a bevel at the top plane, the model
    // says which edges are chamfered - a sharp edge beside it is sharp
    // by design and is left alone (pick it to break it). A model with
    // no bevel at all breaks every top edge, as before (#303).
    if out.is_empty() {
        out = sharp;
        return Region::new(out).map_err(GenError::Geo);
    }
    let sides = |loops: &[Loop]| loops.iter().fold((0, 0), |(o, h), l| if l.is_ccw() { (o + 1, h) } else { (o, h + 1) });
    let want = sides(&out);
    let at = out.first().and_then(|l| l.points().first().copied()).unwrap_or(Point2::new(0.0, 0.0));
    let got = Region::new(out).map_err(GenError::Geo)?;
    // A bevelled edge whose enclosing edge is sharp (a modelled chamfer
    // on a pocket in a sharp plate) loses its nesting once the sharp
    // edge is left out: the region would read it the other way round,
    // and the ride would travel against its material. Refuse by name.
    if sides(got.loops()) != want {
        return Err(GenError::At {
            msg: format!(
                "the edge near X{:.3} Y{:.3} carries a modelled chamfer, but the edge around it is sharp - a chamfer with no picks cannot ride it alone; pick the edges to break",
                at.x, at.y
            ),
            at: Point3::new(at.x, at.y, corner_z),
        });
    }
    Ok(got)
}

/// #2887: a top loop bevelled on only part of its length. Judged per
/// segment of `top` - both ends and the middle against the wall ring -
/// so an extra collinear vertex, or the lack of one, changes nothing
/// (the vertex-only `gap` reads a one-sided bevel either way round).
/// A segment standing on the wall (within `SAME_EDGE`) is sharp; one
/// standing farther than `MATCH_SLACK` from it is bevelled; between
/// the two is tessellation noise and counts as neither. `Some((bevel,
/// sharp))` - a point on each - when the loop has a bevelled segment
/// AND more than `width` of sharp length: a ride of the wall would
/// cut the sharp length, a ride of the top loop would stand inside
/// the part at the bevel. #2936: a loop that is its wall moved whole
/// (`leans`) carries no bevel at all.
fn partly_bevelled(top: &[Point2], wall: &[Point2], width: f64) -> Option<(Point2, Point2)> {
    if wall.is_empty() || leans(top, wall) {
        return None;
    }
    let mut sharp_len: f64 = 0.0;
    let mut sharp_at: Option<Point2> = None;
    let mut bevel_at: Option<Point2> = None;
    for (i, &a) in top.iter().enumerate() {
        let b = top[(i + 1) % top.len()];
        let mid = Point2::new((a.x + b.x) * 0.5, (a.y + b.y) * 0.5);
        let d = near(a, wall).max(near(mid, wall)).max(near(b, wall));
        if d <= SAME_EDGE {
            sharp_len += (b.x - a.x).hypot(b.y - a.y);
            sharp_at = sharp_at.or(Some(mid));
        } else if d > MATCH_SLACK {
            bevel_at = bevel_at.or(Some(mid));
        }
    }
    match (bevel_at, sharp_at) {
        (Some(b), Some(s)) if sharp_len > width => Some((b, s)),
        _ => None,
    }
}

/// Plan slack of the lean test, mm: the sections stand on a 1e-6 mm
/// grid, so two copies of one outline disagree by about that much.
/// Far under `MATCH_SLACK`: no bevelled segment hides in it.
const LEAN_SLACK: f64 = 1e-4;

/// #2936: `top` is the whole of `wall` moved by one plan vector - the
/// two sections of a prism whose walls lean in the operation's frame
/// (a box stock under a tilted frame). No segment of such a loop is a
/// modelled chamfer: the edge is the same edge all round, and the ride
/// is the wall at the corner depth. The vector is the move of the area
/// centroid, which no collinear vertex changes; each segment of each
/// ring - both ends and the middle - must then stand on the other
/// ring. A loop bevelled on part of its length is no moved copy of its
/// wall: its bevelled side moved and its sharp sides did not.
fn leans(top: &[Point2], wall: &[Point2]) -> bool {
    let (Some(t), Some(w)) = (centroid(top), centroid(wall)) else {
        return false;
    };
    let on = |from: &[Point2], to: &[Point2], dx: f64, dy: f64| {
        from.iter().enumerate().all(|(i, &a)| {
            let b = from[(i + 1) % from.len()];
            let mid = Point2::new((a.x + b.x) * 0.5, (a.y + b.y) * 0.5);
            [a, mid, b].iter().all(|p| near(Point2::new(p.x + dx, p.y + dy), to) <= LEAN_SLACK)
        })
    };
    let (dx, dy) = (t.x - w.x, t.y - w.y);
    on(top, wall, -dx, -dy) && on(wall, top, dx, dy)
}

/// The area centroid of the closed polyline `ring`; `None` when it
/// encloses no area or is not finite.
fn centroid(ring: &[Point2]) -> Option<Point2> {
    let o = *ring.first()?;
    let (mut area, mut cx, mut cy) = (0.0, 0.0, 0.0);
    for (i, &p) in ring.iter().enumerate() {
        let q = ring[(i + 1) % ring.len()];
        // About the first vertex: the cross terms stay small.
        let (ax, ay, bx, by) = (p.x - o.x, p.y - o.y, q.x - o.x, q.y - o.y);
        let k = ax * by - bx * ay;
        area += k;
        cx += (ax + bx) * k;
        cy += (ay + by) * k;
    }
    let c = Point2::new(o.x + cx / (3.0 * area), o.y + cy / (3.0 * area));
    (area.abs() > 1e-12 && c.x.is_finite() && c.y.is_finite()).then_some(c)
}

/// The farthest any vertex of `from` stands from the closed polyline
/// `to`, in plan (the one-sided Hausdorff gap over the vertices).
/// `+inf` when either side has no points.
fn gap(from: &[Point2], to: &[Point2]) -> f64 {
    if from.is_empty() || to.is_empty() {
        return f64::INFINITY;
    }
    from.iter()
        .map(|&p| near(p, to))
        .fold(0.0, f64::max)
}

/// Plan distance from `p` to the closed polyline `ring`.
fn near(p: Point2, ring: &[Point2]) -> f64 {
    let n = ring.len();
    (0..n)
        .map(|i| seg(p, ring[i], ring[(i + 1) % n]))
        .fold(f64::INFINITY, f64::min)
}

fn seg(p: Point2, a: Point2, b: Point2) -> f64 {
    let (dx, dy) = (b.x - a.x, b.y - a.y);
    let l2 = dx * dx + dy * dy;
    let t = if l2 > 1e-24 { (((p.x - a.x) * dx + (p.y - a.y) * dy) / l2).clamp(0.0, 1.0) } else { 0.0 };
    let (qx, qy) = (a.x + t * dx - p.x, a.y + t * dy - p.y);
    (qx * qx + qy * qy).sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rect(x0: f64, y0: f64, x1: f64, y1: f64) -> Loop {
        Loop::new(vec![Point2::new(x0, y0), Point2::new(x1, y0), Point2::new(x1, y1), Point2::new(x0, y1)]).expect("rect")
    }

    #[test]
    fn a_vertical_wall_rides_its_own_top_loop() {
        let top = Region::new(vec![rect(-20.0, -10.0, 20.0, 10.0)]).expect("top");
        let got = ride_loops(&top, &top.clone(), 1.0, 9.0).expect("ride");
        assert_eq!(got, top);
    }

    #[test]
    fn a_modelled_bevel_rides_the_wall_under_it() {
        // The bevel's top edge 1 mm inside the wall.
        let top = Region::new(vec![rect(-19.0, -9.0, 19.0, 9.0)]).expect("top");
        let wall = Region::new(vec![rect(-20.0, -10.0, 20.0, 10.0)]).expect("wall");
        let got = ride_loops(&top, &wall, 1.0, 9.0).expect("ride");
        assert_eq!(got, wall);
    }

    #[test]
    fn a_wall_farther_than_the_width_refuses_by_name() {
        let top = Region::new(vec![rect(-19.0, -9.0, 19.0, 9.0)]).expect("top");
        let plate = Region::new(vec![rect(-50.0, -30.0, 50.0, 30.0)]).expect("plate");
        let err = ride_loops(&top, &plate, 1.0, 9.0).expect_err("no wall within the width");
        assert!(err.to_string().contains("no wall under it"), "{err}");
        // #2521: the source may be the stock, so the text names no part.
        assert!(err.to_string().contains("would ride into the material under it;"), "{err}");
        assert!(!err.to_string().contains("part"), "a stock-source chamfer has no part to name: {err}");
    }

    #[test]
    fn a_lower_edge_matched_to_no_top_loop_is_left_alone() {
        let top = Region::new(vec![rect(-20.0, -10.0, 20.0, 10.0)]).expect("top");
        // A second, lower boss beside the island: its wall is in the
        // corner section, but no edge of the top plane stands over it.
        let corner = Region::new(vec![rect(-20.0, -10.0, 20.0, 10.0), rect(30.0, -10.0, 40.0, 10.0)]).expect("corner");
        let got = ride_loops(&top, &corner, 1.0, 9.0).expect("ride");
        assert_eq!(got, top);
    }

    fn hole(x0: f64, y0: f64, x1: f64, y1: f64) -> Loop {
        Loop::new(vec![Point2::new(x0, y0), Point2::new(x0, y1), Point2::new(x1, y1), Point2::new(x1, y0)]).expect("hole")
    }

    #[test]
    fn a_sharp_pocket_beside_a_modelled_bevel_is_left_alone() {
        // #2596: the outer edge carries a 1 mm bevel, the pocket rim is
        // sharp. Only the outer wall is ridden.
        let top = Region::new(vec![rect(-29.0, -19.0, 29.0, 19.0), hole(-10.0, -5.0, 10.0, 5.0)]).expect("top");
        let corner = Region::new(vec![rect(-30.0, -20.0, 30.0, 20.0), hole(-10.0, -5.0, 10.0, 5.0)]).expect("corner");
        let got = ride_loops(&top, &corner, 1.0, 9.0).expect("ride");
        assert_eq!(got, Region::new(vec![rect(-30.0, -20.0, 30.0, 20.0)]).expect("wall"));
    }

    #[test]
    fn a_bevelled_pocket_rim_inside_a_sharp_plate_refuses_by_name() {
        // Alone, the rim would read as a boss and the ride would travel
        // against its material (#1555): refused, the user picks it.
        let top = Region::new(vec![rect(-30.0, -20.0, 30.0, 20.0), hole(-9.0, -4.0, 9.0, 4.0)]).expect("top");
        let corner = Region::new(vec![rect(-30.0, -20.0, 30.0, 20.0), hole(-10.0, -5.0, 10.0, 5.0)]).expect("corner");
        let err = ride_loops(&top, &corner, 1.0, 9.0).expect_err("an orphan bevelled rim");
        assert!(err.to_string().contains("pick the edges to break"), "{err}");
    }

    /// #2887's plate: a 1 mm bevel on the X0 side only, the other three
    /// sides sharp, and a sharp pocket. `extra` adds a collinear vertex
    /// on the bevel's top edge.
    fn one_side_bevelled(extra: bool) -> (Region, Region) {
        let mut outer = vec![Point2::new(1.0, 0.0), Point2::new(60.0, 0.0), Point2::new(60.0, 40.0), Point2::new(1.0, 40.0)];
        if extra {
            outer.push(Point2::new(1.0, 20.0));
        }
        let top = Region::new(vec![Loop::new(outer).expect("outer"), hole(20.0, 15.0, 40.0, 25.0)]).expect("top");
        let corner = Region::new(vec![rect(0.0, 0.0, 60.0, 40.0), hole(20.0, 15.0, 40.0, 25.0)]).expect("corner");
        (top, corner)
    }

    #[test]
    fn a_loop_bevelled_on_one_side_refuses_by_name() {
        // Vertex gap 0 (both bevel corners lie on the wall's sides): the
        // old reading was "sharp", the tip riding x = 1 inside the part.
        let (top, corner) = one_side_bevelled(false);
        let err = ride_loops(&top, &corner, 1.0, 9.0).expect_err("a partly bevelled loop").to_string();
        assert!(err.contains("only part of its length"), "{err}");
        assert!(err.contains("pick the edges to break"), "{err}");
        assert!(err.contains("near X1.000 Y20.000"), "names the bevelled part: {err}");
    }

    #[test]
    fn a_loop_bevelled_on_one_side_refuses_with_an_extra_vertex_too() {
        // Vertex gap 1: the old reading was "bevelled", the wall ridden
        // along the three sharp sides as well.
        let (top, corner) = one_side_bevelled(true);
        let err = ride_loops(&top, &corner, 1.0, 9.0).expect_err("a partly bevelled loop").to_string();
        assert!(err.contains("only part of its length"), "{err}");
        assert!(err.contains("pick the edges to break"), "{err}");
    }

    #[test]
    fn a_loop_bevelled_all_round_is_not_partly_bevelled() {
        let (top, wall) = (rect(-19.0, -9.0, 19.0, 9.0), rect(-20.0, -10.0, 20.0, 10.0));
        assert_eq!(partly_bevelled(top.points(), wall.points(), 1.0), None);
        assert_eq!(partly_bevelled(wall.points(), wall.points(), 1.0), None);
    }

    /// #2936: the sections of a 70 mm box stock in a frame tilted 10
    /// degrees, 1 mm apart - the corner section and the same outline
    /// moved by `tan(10 deg)` in Y. The mesh diagonals leave a collinear
    /// vertex at a different place on each side of each section.
    fn leaning_box() -> (Region, Region) {
        let ring = |y0: f64, y1: f64, mids: [f64; 4]| {
            let p = [(35.0, mids[0]), (35.0, y1), (mids[1], y1), (-35.0, y1), (-35.0, mids[2]), (-35.0, y0), (mids[3], y0), (35.0, y0)];
            Region::new(vec![Loop::new(p.iter().map(|&(x, y)| Point2::new(x, y)).collect()).expect("ring")]).expect("region")
        };
        let top = ring(-33.776662, 37.303201, [3.1953, -22.139987, 1.42424, -21.060124]);
        let corner = ring(-33.952989, 37.126874, [-6.407086, -18.585993, 3.479512, -24.614117]);
        (top, corner)
    }

    #[test]
    fn a_wall_leaning_in_the_frame_is_not_a_modelled_chamfer() {
        // No edge of the box carries a chamfer: the whole outline moved.
        // The ride is the wall at the corner depth, as before #2887.
        let (top, corner) = leaning_box();
        let got = ride_loops(&top, &corner, 1.0, 9.0).expect("a leaning box rides its wall");
        assert_eq!(got, corner);
    }

    #[test]
    fn a_moved_outline_with_one_side_bevelled_still_refuses() {
        // The leaning box with a 1 mm bevel on its X35 side as well:
        // that side is no part of the lean, and the #2887 refusal holds.
        let (top, corner) = leaning_box();
        let bevelled: Vec<Point2> = top.loops()[0].points().iter().map(|p| Point2::new(p.x.min(34.0), p.y)).collect();
        let top = Region::new(vec![Loop::new(bevelled).expect("ring")]).expect("top");
        let err = ride_loops(&top, &corner, 1.0, 9.0).expect_err("a partly bevelled loop").to_string();
        assert!(err.contains("only part of its length"), "{err}");
    }
}
