//! #4025 pins: one selection through each present path of the phase-1
//! kinds gives one kernel region, where those paths agree; where two paths
//! disagree today, the pin names the fact and the kernel's reading.

use super::*;
use crate::machining2d::contour::prepare_band;
use crate::machining2d::contour_params::ContourParams;
use crate::machining2d::pass_start::PassStart;
use crate::machining2d::pocket_plan::non_sliver_center;
use crate::machining2d::Lead;
use camcraft_pathkernel::PlanarRegion;
use camcraft_pathkernel::region::OPEN_NEEDS_SIDE;

const R: f64 = 3.0;
/// The arc tolerance of the polygon offset of `camcraft-geometry` (mm).
const ARC_TOL: f64 = 0.005;

fn p(x: f64, y: f64) -> Point2 {
    Point2::new(x, y)
}

fn sq(x0: f64, y0: f64, s: f64) -> Loop {
    Loop::new(vec![p(x0, y0), p(x0 + s, y0), p(x0 + s, y0 + s), p(x0, y0 + s)]).expect("square")
}

fn area(loops: Vec<Loop>) -> Region {
    Region::new(loops).expect("region")
}

/// The 60 mm stock from (-20, -20).
fn stock() -> Region {
    area(vec![sq(-20.0, -20.0, 60.0)])
}

fn xy() -> WorkPlane {
    WorkPlane::setup_xy(0.0).expect("plane")
}

fn tool() -> ToolShape {
    ToolShape::disc(R, 0.0, true, 20.0).expect("tool")
}

fn bounds() -> Bounds {
    Bounds::new(Some(stock()), Region::empty(), 0.0, -5.0).expect("bounds")
}

fn planar(k: &KernelRegion) -> &PlanarRegion {
    k.planar().expect("planar")
}

/// The largest distance from a boundary vertex of one region to the
/// boundary of the other, both ways (the Hausdorff distance of the
/// boundaries, sampled at the vertices; mm).
fn gap(a: &Region, b: &Region) -> f64 {
    let one_way = |a: &Region, b: &Region| {
        a.loops()
            .iter()
            .flat_map(|l| l.points().iter())
            .map(|q| {
                b.loops()
                    .iter()
                    .flat_map(|l| {
                        let pts = l.points();
                        pts.iter().zip(pts.iter().cycle().skip(1)).map(|(s, e)| seg(*q, *s, *e)).collect::<Vec<_>>()
                    })
                    .fold(f64::INFINITY, f64::min)
            })
            .fold(0.0, f64::max)
    };
    one_way(a, b).max(one_way(b, a))
}

fn seg(q: Point2, a: Point2, b: Point2) -> f64 {
    let (dx, dy) = (b.x - a.x, b.y - a.y);
    let len2 = dx * dx + dy * dy;
    let t = if len2 > 0.0 { (((q.x - a.x) * dx + (q.y - a.y) * dy) / len2).clamp(0.0, 1.0) } else { 0.0 };
    (q.x - (a.x + t * dx)).hypot(q.y - (a.y + t * dy))
}

/// The area of the symmetric difference of two regions (mm^2).
fn apart(a: &Region, b: &Region) -> f64 {
    a.difference(b).expect("diff").area().abs() + b.difference(a).expect("diff").area().abs()
}

fn contour(side: Side) -> ContourParams {
    ContourParams {
        tool_diameter: 2.0 * R,
        stock_to_leave: 0.0,
        clearance_z: 10.0,
        retract_z: 5.0,
        top_z: 0.0,
        bottom_z: -5.0,
        stepdown: 5.0,
        feed: 500.0,
        plunge_feed: 100.0,
        climb: true,
        side,
        lead: Lead::None,
        tolerance: 0.01,
        tab_count: 0,
        tab_width: 0.0,
        tab_height: 0.0,
        entry: crate::entry::Entry3D::Plunge,
        helix_radius: 1.0,
        blend_turns: false,
        blend_radius: None,
        links: crate::links::Links::PLANAR,
        picks: crate::path_picks::PathPicks::default(),
    }
}

/// The kernel region of `a` on `side` through the Contour carrier.
fn kernel_contour(a: &Region, side: Side) -> KernelRegion {
    let material = contour_material(side).expect("a named side");
    region(&selection_of_area(a).expect("selection"), material, &bounds(), xy(), tool(), 0.0).expect("region")
}

#[test]
fn one_selection_gives_one_region_through_pocket_contour_and_each_start_side() {
    // A pocket round a boss: a 20 mm square with a 6 mm square hole.
    let a = area(vec![sq(0.0, 0.0, 20.0), sq(7.0, 7.0, 6.0)]);
    let kernel = kernel_contour(&a, Side::Inside);
    let k = planar(&kernel);
    let pocket = non_sliver_center(&a, R).expect("pocket centre");
    let contour_in = prepare_band(&a, &contour(Side::Inside)).expect("contour band").center;
    assert!(apart(k.center(), &pocket) < 1e-6, "Pocket and the kernel: one region");
    assert!(apart(k.center(), &contour_in) < 1e-6, "Contour Inside and the kernel: one region");
    assert_eq!((k.part_walls().len(), k.stock_edge().len()), (2, 0));
    // The pocket family's carrier gives the same region for each start
    // side: the start side never reaches the region.
    for start in [PassStart::Auto, PassStart::Inside, PassStart::Outside] {
        let side = StartSide { start, stock: Some(stock()), around_part: false };
        let r = pocket_region(&a, &side, &bounds(), xy(), tool(), 0.0).expect("pocket region");
        assert_eq!(&r, &kernel, "{start:?}");
    }
}

#[test]
fn contour_outside_rings_are_the_part_walls_of_the_outside_region() {
    let part = area(vec![sq(0.0, 0.0, 20.0)]);
    let kernel = kernel_contour(&part, Side::Outside);
    let k = planar(&kernel);
    let ring = prepare_band(&part, &contour(Side::Outside)).expect("contour band").center;
    // The Contour ring is unbounded; inside the stock the kernel's centre is
    // the stock less that ring's area.
    let cut_off = stock().difference(k.center()).expect("diff");
    assert!(apart(&cut_off, &ring.intersect(&stock()).expect("cap")) < 1e-6);
    assert_eq!((k.part_walls().len(), k.stock_edge().len()), (1, 1));
}

#[test]
fn the_offset_band_keeps_off_the_stock_edge_and_the_kernel_reaches_it() {
    // Disagreement, pinned: today's 2D Offset band (around_part) erodes the
    // whole band, so its centre stays one radius inside the stock outline.
    // The kernel's Outside centre reaches the outline (decision 4 of
    // docs/decisions/4025-kernel-region.md). The part walls agree.
    let part = area(vec![sq(0.0, 0.0, 20.0)]);
    let band = stock().difference(&part).expect("band");
    let side = StartSide { start: PassStart::Outside, stock: Some(stock()), around_part: true };
    let (_, material) = pocket_selection(&band, &side).expect("selection");
    assert_eq!(material, MaterialSide::Outside);
    let kernel = pocket_region(&band, &side, &bounds(), xy(), tool(), 0.0).expect("kernel region");
    let k = planar(&kernel);
    let present = non_sliver_center(&band, R).expect("band centre");
    let inner = area(vec![sq(-20.0 + R, -20.0 + R, 60.0 - 2.0 * R)]);
    // The band erodes the part's hole and the kernel grows the part: two
    // chord sets of one arc, each within the offset's arc tolerance.
    let capped = k.center().intersect(&inner).expect("cap");
    let dist = gap(&capped, &present);
    assert!(dist <= 2.0 * ARC_TOL, "the part walls agree within the arc tolerance: {dist} mm");
    let strip = k.center().area() - present.area();
    assert!((strip - (3600.0 - 54.0 * 54.0)).abs() < 0.1, "the stock-edge strip {strip}");
    let via_contour = kernel_contour(&part, Side::Outside);
    assert!(apart(k.center(), planar(&via_contour).center()) < 1e-6, "the Offset band and Contour Outside: one region");
}

#[test]
fn the_pocket_drops_a_sliver_that_contour_and_the_kernel_keep() {
    // Disagreement, pinned: a second pocket 0.01 mm wider than the tool
    // leaves a 0.01 mm square of cutter-centre area. The Pocket path drops
    // it as a sliver (#2232); Contour Inside and the kernel keep it. The
    // kernel keeps the exact area; the guide stage's completeness guard
    // (0.1 mm^2) decides what is cut (decision 5).
    let a = area(vec![sq(0.0, 0.0, 20.0), sq(25.0, 0.0, 2.0 * R + 0.01)]);
    let k = kernel_contour(&a, Side::Inside);
    let pocket = non_sliver_center(&a, R).expect("pocket centre");
    let contour_in = prepare_band(&a, &contour(Side::Inside)).expect("contour band").center;
    assert_eq!((pocket.loops().len(), contour_in.loops().len(), planar(&k).part_walls().len()), (1, 2, 2));
    assert!(apart(planar(&k).center(), &contour_in) < 1e-6);
}

#[test]
fn an_open_run_and_an_auto_side_reach_the_kernel_without_a_side() {
    let run = vec![p(0.0, 0.0), p(10.0, 0.0), p(10.0, 5.0)];
    let sel = selection_of_picks(&[sq(-10.0, -10.0, 5.0)], &[run]).expect("selection");
    let r = region(&sel, MaterialSide::Inside, &bounds(), xy(), tool(), 0.0);
    assert_eq!(r.expect_err("refused").reason(), OPEN_NEEDS_SIDE);
    assert_eq!(contour_material(Side::Auto), None);
    let kept = kernel_bounds_check();
    assert_eq!(kept.stock(), Some(&stock()));
}

fn kernel_bounds_check() -> Bounds {
    let side = StartSide { start: PassStart::Auto, stock: Some(stock()), around_part: false };
    bounds_of(&side, Region::empty(), 0.0, -5.0).expect("bounds")
}
