//! The link rule (#4023 stage 5, #4027): stay at depth; lift only with a
//! named reason. The check counts the lifts of the plan against the link
//! tally (in total and by reason), counts the lifts on a fixture with no
//! split ring (there must be none), and reads each stay-down hop - from
//! the end of its cut, along its points, to the start of the next cut - for
//! a crossing out of the centre area, the air and the edge band (#4241:
//! outside the stock within a tool radius of it, the disc clear of each
//! part wall loop and each keep-out area; read here from the stock of the
//! case, not from the kernel's room).

use std::collections::BTreeMap;

use camcraft_pathkernel::guide::{Curve, Family};
use camcraft_pathkernel::link::{CUTS_PART, HITS_FIXTURE, Hop, Level, LinkPolicy, LinkProof, LinkStage, LinkTally, UNCUT_STOCK};
use camcraft_pathkernel::plan::{Cut, Step};
use camcraft_pathkernel::region::{KernelRegion, PlanarRegion};
use camcraft_pathkernel::{Bounds, Loop, MaterialSide, PlanePoint, Point2, Refusal, Region, Selection, region};

use super::grid::samples;
use super::guide_kit::{circle, rect, tool, xy};
use super::order_link::stock_box;
use super::kit::{
    Case, Law, LawVerdict, MATCH_TOL, Measure, Planners, Run, assert_broken, assert_held, judge, outside, planar, poly_dist, refused_held, run,
};
use super::standins::{LEAVES_AREA, end_of, good, island_case, square_case, start_of};

/// L-link over one run of the case `c`, or over its refusal.
pub fn check_link(run: &Result<Run, Refusal>, c: &Case) -> LawVerdict {
    let r = match run {
        Ok(r) => r,
        Err(e) => return refused_held(Law::Link, e),
    };
    let steps = r.plan.steps();
    let mut by_reason: BTreeMap<&'static str, u32> = BTreeMap::new();
    for s in steps {
        if let Step::Link(Hop::Lift(why)) = s {
            *by_reason.entry(why.get()).or_insert(0) += 1;
        }
    }
    let lifts: u32 = by_reason.values().sum();
    let tally = r.tally.lifts();
    let mut m =
        vec![Measure::new(f64::from(lifts.abs_diff(tally)), 0.0, format!("lifts: {lifts} in the plan, {tally} in the tally"), None)];
    let odd = by_reason.iter().filter(|(k, v)| r.tally.by_reason().get(*k) != Some(*v)).count();
    m.push(Measure::count(odd, format!("lift reasons: {by_reason:?} in the plan, {:?} in the tally", r.tally.by_reason()), None));
    if c.no_split {
        m.push(Measure::new(f64::from(lifts), 0.0, format!("lifts on a fixture with no split ring: {by_reason:?}"), None));
    }
    let Ok(p) = planar(&r.region) else {
        return judge(Law::Link, m);
    };
    for (i, s) in steps.iter().enumerate() {
        let Step::Link(Hop::Stay(path)) = s else {
            continue;
        };
        let cut = |j: Option<usize>| match j.and_then(|j| steps.get(j)) {
            Some(Step::Cut(c)) => Some(c),
            _ => None,
        };
        let mut line: Vec<Point2> = cut(i.checked_sub(1)).map(end_of).into_iter().collect();
        line.extend(path.points());
        line.extend(cut(i.checked_add(1)).map(start_of));
        let worst = samples(&line, 0.05)
            .iter()
            .map(|(q, _)| (outside(p.center(), *q).min(outside(p.air(), *q)).min(off_band(p, &c.stock, *q)), *q))
            .max_by(|a, b| a.0.total_cmp(&b.0));
        if let Some((d, q)) = worst {
            m.push(Measure::new(d, MATCH_TOL, format!("stay link at step {i}: a crossing out of the area"), Some(q)));
        }
    }
    judge(Law::Link, m)
}

/// The distance from `q` to the nearest loop of `loops`.
fn to_loops(q: Point2, loops: &[Loop]) -> f64 {
    let ring = |l: &Loop| {
        let mut v = l.points().to_vec();
        v.extend(l.points().first().copied());
        poly_dist(q, &v)
    };
    loops.iter().map(ring).fold(f64::INFINITY, f64::min)
}

/// How far the tool centre `q` is from the edge band (0 in the band): it
/// must be outside the stock, within a tool radius of it, and its disc
/// must meet no part wall loop and no keep-out area.
fn off_band(p: &PlanarRegion, stock: &Region, q: Point2) -> f64 {
    if stock.is_empty() {
        return f64::INFINITY;
    }
    let r = p.tool().radius();
    let edge = to_loops(q, stock.loops());
    let (inside, far) = if stock.contains(q) { (edge, 0.0) } else { (0.0, (edge - r).max(0.0)) };
    let wall = (r - to_loops(q, p.part_walls())).max(0.0);
    let keep = p.keep_out();
    let fixture = if keep.is_empty() { 0.0 } else if keep.contains(q) { f64::INFINITY } else { (r - to_loops(q, keep.loops())).max(0.0) };
    inside.max(far).max(wall).max(fixture)
}

/// Breaks the link rule one of three ways.
enum BadLink {
    /// Lifts and does not count the lift.
    Silent,
    /// Lifts (counted) where the straight hop could stay.
    Always,
    /// Stays at depth across the island.
    Across,
}

impl LinkProof for BadLink {
    fn link(&self, _from: &Cut, _to: &Cut, _r: &KernelRegion, _level: Level<'_>, _policy: LinkPolicy, tally: &mut LinkTally) -> Hop {
        let lift = || Hop::lift(LEAVES_AREA).expect("a named lift");
        match self {
            Self::Silent => lift(),
            Self::Always => tally.record(lift()),
            // The island of the fixture is the square from (10, 10) to (20, 20).
            Self::Across => Hop::stay(vec![Point2::new(15.0, 15.0)]).expect("a stay hop"),
        }
    }
}

fn link(p: &Planners, c: &Case) -> LawVerdict {
    check_link(&run(p, c, Default::default()), c)
}

#[test]
fn link_rule_holds_for_counted_lifts_and_proved_stays() {
    assert_held(&link(&good(), &square_case().unwrap()));
    assert_held(&link(&good(), &island_case().unwrap()));
}

#[test]
fn link_rule_breaks_for_an_uncounted_lift() {
    assert_broken(&link(&Planners { link: Box::new(BadLink::Silent), ..good() }, &island_case().unwrap()), Law::Link, "in the tally");
}

#[test]
fn link_rule_breaks_for_a_lift_on_a_fixture_with_no_split_ring() {
    let v = link(&Planners { link: Box::new(BadLink::Always), ..good() }, &square_case().unwrap());
    assert_broken(&v, Law::Link, "no split ring");
}

#[test]
fn link_rule_breaks_for_a_stay_across_the_island() {
    let v = link(&Planners { link: Box::new(BadLink::Across), ..good() }, &island_case().unwrap());
    assert_broken(&v, Law::Link, "a crossing out of the area");
    assert!(v.worst.value > 1.0, "{v}");
}

// The edge band (#4241). The stock is the 80 x 50 box of the K4 pins (its
// top edge is the line y = 25), the tool radius is 4: the band is the strip
// 25 < y < 29 above the top edge, and the air is 29 < y < 33.

const BAND_R: f64 = 4.0;

fn band_region(part: Vec<Point2>, keep: Region) -> KernelRegion {
    let sel = Selection::face(part, vec![]).unwrap();
    let b = Bounds::new(Some(stock_box()), keep, 0.0, -5.0).unwrap();
    region(&sel, MaterialSide::Outside, &b, xy(), tool(BAND_R), 0.0).unwrap()
}

fn line(xy: &[(f64, f64)]) -> Cut {
    let pts = xy.iter().map(|&(x, y)| PlanePoint::flat(Point2::new(x, y)).unwrap()).collect();
    Cut::new(Curve::new(pts, None, false, 0, Family::Ring).unwrap(), None).unwrap()
}

/// The hop from `from` to `to`, with `made` cut before it (`from` last).
fn hop(r: &KernelRegion, made: &[Cut], to: &Cut) -> Hop {
    let from = made.last().unwrap();
    let planned = [to.clone()];
    LinkStage.link(from, to, r, Level::new(made, &planned), LinkPolicy::StayWhereProved, &mut LinkTally::default())
}

fn lift_reason(h: &Hop) -> &'static str {
    match h {
        Hop::Lift(why) => why.get(),
        Hop::Stay(p) => panic!("the link must not stay: {:?}", p.points()),
    }
}

#[test]
fn a_link_in_the_edge_band_over_uncut_stock_is_refused() {
    // The link runs at y = 27, half a tool radius outside the stock: its
    // disc cuts half a radius into the stock below, and no cut has cleared
    // that stock. The band is open ground for the part, not for the stock.
    let r = band_region(circle(0.0, 0.0, 12.0, 128), Region::empty());
    let from = line(&[(-20.0, 20.0), (-20.0, 27.0)]);
    let to = line(&[(20.0, 27.0), (20.0, 20.0)]);
    assert_eq!(lift_reason(&hop(&r, &[from], &to)), UNCUT_STOCK);
}

#[test]
fn a_link_in_the_edge_band_near_a_keep_out_is_refused() {
    // A clamp outside the stock, 1.5 to 2 tool radii above the top edge:
    // the link at y = 28 passes within a tool radius of it.
    let clamp = Region::new(vec![Loop::new(rect(-2.0, 31.0, 2.0, 33.0)).unwrap()]).unwrap();
    let r = band_region(circle(0.0, 0.0, 12.0, 128), clamp);
    assert!(!r.planar().unwrap().stock_edge().is_empty(), "the clamp does not reach the stock: the top edge stays a stock edge");
    let from = line(&[(-20.0, 20.0), (-20.0, 28.0)]);
    let to = line(&[(20.0, 28.0), (20.0, 20.0)]);
    assert_eq!(lift_reason(&hop(&r, &[from], &to)), HITS_FIXTURE);
}

#[test]
fn a_link_beside_a_part_that_reaches_out_of_the_stock_is_refused() {
    // The part reaches 5 mm out of the stock through the top edge. The
    // stock outline there is no stock edge, so no band is next to it: the
    // link outside the stock beside the part wall is refused.
    let part = rect(-5.0, 10.0, 5.0, 30.0);
    let r = band_region(part.clone(), Region::empty());
    let from = line(&[(-20.0, 20.0), (-20.0, 27.0)]);
    let to = line(&[(-8.0, 27.0), (-12.0, 31.0)]);
    assert_eq!(lift_reason(&hop(&r, &[from], &to)), CUTS_PART);
    // A link in the air across the part (y = 30 is in the part) does not
    // cross it: the air leaves out the part grown by the tool radius. It
    // lifts, or it goes round the stock in the air.
    let from = line(&[(-20.0, 20.0), (-20.0, 30.0)]);
    let to = line(&[(20.0, 30.0), (20.0, 20.0)]);
    let h = hop(&r, std::slice::from_ref(&from), &to);
    if let Hop::Stay(p) = &h {
        let mut pts = vec![Point2::new(-20.0, 30.0)];
        pts.extend(p.points());
        pts.push(Point2::new(20.0, 30.0));
        let near = samples(&pts, 0.05).iter().map(|(q, _)| to_loops(*q, &[Loop::new(part.clone()).unwrap()])).fold(f64::INFINITY, f64::min);
        let inside = samples(&pts, 0.05).iter().any(|(q, _)| q.x.abs() < 5.0 && (10.0..30.0).contains(&q.y));
        assert!(!inside && near >= BAND_R - 1e-6, "the stay link keeps a tool radius from the part: {near}");
    }
}

#[test]
fn a_link_leaves_a_cut_at_the_stock_edge_runs_in_the_air_and_comes_back_over_cleared_stock() {
    let r = band_region(circle(0.0, 0.0, 12.0, 128), Region::empty());
    // The cut just made ends on the top edge at x = -20; the next cut starts
    // on it at x = 20. The stock between them along the edge is not cut.
    let from = line(&[(-30.0, 22.0), (-20.0, 22.0), (-20.0, 25.0)]);
    let to = line(&[(20.0, 25.0), (20.0, 22.0), (30.0, 22.0)]);
    // With no cut made at the start of the next cut, the way back in from
    // the air runs over stock that no cut has cleared. It stays only as an
    // approach from the air (#4254, `air_approach.rs`): straight in to the
    // start of the piece, the disc cutting only the swath of that piece.
    let h = hop(&r, std::slice::from_ref(&from), &to);
    let Hop::Stay(p) = &h else { panic!("the approach from the air must stay: {h:?}") };
    let pts = p.points();
    assert!(pts.len() >= 2 && (pts[pts.len() - 2].x - 20.0).abs() < 1e-6 && pts[pts.len() - 2].y >= 29.0 - 1e-6, "in from the air above the start: {pts:?}");
    // A cut already made along the edge at the start of the next cut
    // clears the stock there: the link goes out into the air, along it,
    // and back in, at depth.
    let cleared = line(&[(15.0, 25.0), (25.0, 25.0)]);
    let h = hop(&r, &[cleared, from], &to);
    let Hop::Stay(p) = &h else { panic!("the link through the air must stay: {h:?}") };
    let pts = p.points();
    assert!(pts.iter().any(|q| q.y >= 29.0 - 1e-6), "the link runs in the air: {pts:?}");
    let p = r.planar().unwrap();
    let worst = samples(&pts, 0.05)
        .iter()
        .map(|(q, _)| outside(p.center(), *q).min(outside(p.air(), *q)).min(off_band(p, &stock_box(), *q)))
        .fold(0.0, f64::max);
    assert!(worst <= MATCH_TOL, "the link stays in the centre area, the band or the air: {worst}");
}
