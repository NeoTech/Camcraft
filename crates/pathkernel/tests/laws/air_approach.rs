//! The approach from the air (#4254): a stay-down link may end with a
//! straight stretch from the air into the start of an open piece on the
//! stock edge, when the disc swept on that stretch cuts no stock off the
//! planned curve. The negative pins first: stock off the curve, a keep-out
//! near the stock edge, a part that reaches out of the stock, a piece that
//! does not start at the stock edge. Then the positive case, and a check of
//! each approach of the two bands against the stock of the case (not the
//! kernel's air): the stock that it sweeps is in the swath of the next cut.
//! The stock is the 80 x 50 box of the K4 pins (its top edge is y = 25),
//! the tool radius is 4: the edge band is 25 < y < 29, the air 29 < y < 33.

use camcraft_pathkernel::guide::{Curve, Family};
use camcraft_pathkernel::link::{Hop, Level, LinkPolicy, LinkProof, LinkStage, LinkTally, UNCUT_STOCK};
use camcraft_pathkernel::order::StartSide;
use camcraft_pathkernel::plan::{Cut, Step};
use camcraft_pathkernel::{Bounds, KernelRegion, Loop, MaterialSide, PlanePoint, Point2, Region, Selection, region};

use super::guide_kit::{circle, rect, tool, xy};
use super::kit::{outside, run};
use super::order_link::{cases, kernel, stock_box};
use super::standins::{end_of, start_of};

const R: f64 = 4.0;

fn p(x: f64, y: f64) -> Point2 {
    Point2::new(x, y)
}

fn edge_region(stock: Vec<Point2>, part: Vec<Point2>, keep: Region) -> KernelRegion {
    let sel = Selection::face(part, vec![]).unwrap();
    let stock = Region::new(vec![Loop::new(stock).unwrap()]).unwrap();
    let b = Bounds::new(Some(stock), keep, 0.0, -5.0).unwrap();
    region(&sel, MaterialSide::Outside, &b, xy(), tool(R), 0.0).unwrap()
}

fn line(xy: &[(f64, f64)]) -> Cut {
    let pts = xy.iter().map(|&(x, y)| PlanePoint::flat(p(x, y)).unwrap()).collect();
    Cut::new(Curve::new(pts, None, false, 0, Family::Ring).unwrap(), None).unwrap()
}

/// The hop from `from` (the only cut made) to `to` (the only cut planned).
fn hop(r: &KernelRegion, from: &Cut, to: &Cut) -> Hop {
    let (made, planned) = ([from.clone()], [to.clone()]);
    LinkStage.link(from, to, r, Level::new(&made, &planned), LinkPolicy::StayWhereProved, &mut LinkTally::default())
}

fn lift_reason(h: &Hop) -> &'static str {
    match h {
        Hop::Lift(why) => why.get(),
        Hop::Stay(path) => panic!("the link must not stay: {:?}", path.points()),
    }
}

/// The cut just made ends on the top edge at x = -20; the next piece
/// starts on it at x = 20 and runs into the stock. No cut has cleared the
/// stock at the start of the next piece.
fn pieces() -> (Cut, Cut) {
    (line(&[(-30.0, 22.0), (-20.0, 22.0), (-20.0, 25.0)]), line(&[(20.0, 25.0), (20.0, 22.0), (30.0, 22.0)]))
}

fn disc_part() -> Vec<Point2> {
    circle(0.0, 0.0, 12.0, 128)
}

/// The area of the stock that the disc sweeps along `stretch` and that the
/// swath of `next` does not cover (read from the stock of the case).
fn off_curve(stock: &Region, stretch: &[Point2], next: &Cut) -> f64 {
    let on: Vec<Point2> = next.curve().pts().iter().map(|q| q.at()).collect();
    let swept = Region::sweep(&[stretch.to_vec()], R).unwrap().intersect(stock).unwrap();
    swept.difference(&Region::sweep(&[on], R + 0.01).unwrap()).unwrap().area()
}

#[test]
fn an_approach_that_would_cut_stock_off_the_curve_is_refused() {
    // A tooth of stock stands on the top edge beside the start of the next
    // piece (x 23.6 to 24.4, up to y = 27). The air is still 4 mm above
    // the start, but the disc on the way in cuts the tooth, and the swath
    // of the next piece does not reach it.
    let mut stock = rect(-40.0, -25.0, 40.0, 25.0);
    stock.splice(3..3, [p(24.4, 25.0), p(24.4, 27.0), p(23.6, 27.0), p(23.6, 25.0)]);
    let r = edge_region(stock.clone(), disc_part(), Region::empty());
    let (from, to) = pieces();
    let tooth = Region::new(vec![Loop::new(stock).unwrap()]).unwrap();
    assert!(off_curve(&tooth, &[p(20.0, 29.0), p(20.0, 25.0)], &to) > 0.1, "the straight way in from the air cuts the tooth");
    assert_eq!(lift_reason(&hop(&r, &from, &to)), UNCUT_STOCK);
}

#[test]
fn an_approach_near_a_keep_out_at_the_stock_edge_is_refused() {
    // A clamp outside the stock, 3.4 mm from the place of the air above the
    // start of the next piece (and 6 mm from the start): the way in from
    // the air passes within a tool radius of it.
    let clamp = Region::new(vec![Loop::new(rect(23.0, 30.5, 25.0, 33.0)).unwrap()]).unwrap();
    let r = edge_region(rect(-40.0, -25.0, 40.0, 25.0), disc_part(), clamp);
    assert!(!r.planar().unwrap().stock_edge().is_empty(), "the clamp does not reach the stock: the top edge stays a stock edge");
    let (from, to) = pieces();
    assert_eq!(lift_reason(&hop(&r, &from, &to)), UNCUT_STOCK);
}

#[test]
fn an_approach_beside_a_part_that_reaches_out_of_the_stock_is_refused() {
    // The part reaches 5 mm out of the stock through the top edge. The
    // stock outline is then a part wall, not a stock edge: no piece starts
    // on a stock edge there, and no approach from the air is proved.
    let r = edge_region(rect(-40.0, -25.0, 40.0, 25.0), rect(-5.0, 10.0, 5.0, 30.0), Region::empty());
    let to = line(&[(10.0, 25.0), (10.0, 20.0), (20.0, 20.0)]);
    let from = line(&[(-30.0, 22.0), (-20.0, 22.0), (-20.0, 25.0)]);
    assert!(matches!(hop(&r, &from, &to), Hop::Lift(_)), "the link beside the part lifts");
}

#[test]
fn a_piece_that_does_not_start_at_the_stock_edge_keeps_its_lift_and_its_reason() {
    // The next piece starts 3 mm inside the top edge: the present rule
    // holds, and the link lifts with the reason of the straight link.
    let r = edge_region(rect(-40.0, -25.0, 40.0, 25.0), disc_part(), Region::empty());
    let (from, _) = pieces();
    let to = line(&[(20.0, 22.0), (20.0, 20.0), (30.0, 20.0)]);
    assert_eq!(lift_reason(&hop(&r, &from, &to)), UNCUT_STOCK);
}

#[test]
fn a_link_comes_in_from_the_air_to_a_piece_at_the_stock_edge() {
    let r = edge_region(rect(-40.0, -25.0, 40.0, 25.0), disc_part(), Region::empty());
    let (from, to) = pieces();
    let h = hop(&r, &from, &to);
    let Hop::Stay(path) = &h else { panic!("the approach from the air must stay: {h:?}") };
    let pts = path.points();
    let n = pts.len();
    assert!(n >= 3, "out to the air, along it and in: {pts:?}");
    let (door, start) = (pts[n - 2], pts[n - 1]);
    assert!((start.x - 20.0).abs() < 1e-9 && (start.y - 25.0).abs() < 1e-9, "the link ends at the start of the piece: {pts:?}");
    assert!(door.y >= 29.0 - 1e-6 && (door.x - 20.0).abs() < 1e-6, "the last stretch comes from the air: {pts:?}");
    let stock = stock_box();
    assert!(off_curve(&stock, &[door, start], &to) < 1e-6, "the approach cuts no stock off the planned curve");
    let p = r.planar().unwrap();
    assert!(pts.iter().all(|&q| outside(p.center(), q) < 1e-6 || outside(p.air(), q) < 1e-6 || !stock.contains(q)));
}

#[test]
fn each_approach_of_the_bands_cuts_only_the_swath_of_its_piece() {
    // Outside on the disc band and on the dog-bone band: each stay link
    // whose last stretch comes from outside the stock by a tool radius (the
    // air) to the start of the next piece cuts no stock off that piece.
    let stock = stock_box();
    let mut approaches = 0;
    for c in cases().into_iter().filter(|c| c.material == MaterialSide::Outside) {
        let r = run(&kernel(), &c, StartSide::Outside).unwrap();
        let steps = r.plan.steps();
        for (i, s) in steps.iter().enumerate() {
            let (Step::Link(Hop::Stay(path)), Some(Step::Cut(next))) = (s, steps.get(i + 1)) else { continue };
            let Some(Step::Cut(prev)) = i.checked_sub(1).and_then(|j| steps.get(j)) else { continue };
            let mut line = vec![end_of(prev)];
            line.extend(path.points());
            line.push(start_of(next));
            let Some(k) = line.iter().rposition(|&q| !stock.contains(q) && outside(&stock, q) >= R - 1e-6) else { continue };
            let stretch = &line[k..];
            if stretch.len() < 2 || stretch.iter().any(|&q| stock.contains(q) && outside(&stock, q) < -1e-6) {
                continue;
            }
            approaches += 1;
            let area = off_curve(&stock, stretch, next);
            assert!(area < 1e-6, "{}: the approach at step {i} cuts {area} mm2 of stock off its piece: {stretch:?}", c.name);
        }
    }
    assert!(approaches > 0, "the bands come in from the air");
}
