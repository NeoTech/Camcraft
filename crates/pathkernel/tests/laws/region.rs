//! Stage 1 (K1, #4025): one selection gives one region; the material side
//! is a parameter of its own; each degenerate selection is refused by name.

use super::common::{square, square_region, tool};
use camcraft_pathkernel::region::{
    BAD_LEAVE, EMPTY_CENTER, EMPTY_MATERIAL, NEEDS_STOCK, NOT_A_PARTITION, OFF_STOCK, OPEN_NEEDS_SIDE,
};
use camcraft_pathkernel::selection::{
    CHAINS_CROSS, EMPTY_SELECTION, REPEATED_POINT, SELECTION_NOT_FINITE, SELF_CROSSING, SHORT_CHAIN, ZERO_AREA,
};
use camcraft_pathkernel::{
    Bounds, Chain, KernelRegion, MaterialSide, PlanarRegion, Point2, Refusal, Region, Selection, ToolShape, WorkPlane,
    region,
};

fn p(x: f64, y: f64) -> Point2 {
    Point2::new(x, y)
}

fn sq(x0: f64, y0: f64, s: f64) -> Vec<Point2> {
    vec![p(x0, y0), p(x0 + s, y0), p(x0 + s, y0 + s), p(x0, y0 + s)]
}

fn xy() -> WorkPlane {
    WorkPlane::setup_xy(0.0).unwrap()
}

/// A 60 mm stock from (-20, -20), no keep-out.
fn stock() -> Bounds {
    Bounds::new(Some(square_region(-20.0, -20.0, 60.0)), Region::empty(), 0.0, -5.0).unwrap()
}

/// A 20 mm square with a 6 mm square hole: a pocket round a boss.
fn frame_selection() -> Selection {
    Selection::face(sq(0.0, 0.0, 20.0), vec![sq(7.0, 7.0, 6.0)]).unwrap()
}

fn planar(k: &KernelRegion) -> &PlanarRegion {
    k.planar().unwrap()
}

/// The area of the symmetric difference of two regions (mm^2).
fn apart(a: &Region, b: &Region) -> f64 {
    a.difference(b).unwrap().area().abs() + b.difference(a).unwrap().area().abs()
}

fn refused<T: std::fmt::Debug>(r: Result<T, Refusal>) -> &'static str {
    r.unwrap_err().reason()
}

#[test]
fn the_region_stage_takes_no_start_side() {
    // Law 2 in the type system: the signature has a material side and no
    // start side.
    type RegionStage = fn(&Selection, MaterialSide, &Bounds, WorkPlane, ToolShape, f64) -> Result<KernelRegion, Refusal>;
    let _: RegionStage = region;
}

#[test]
fn one_selection_gives_one_region_in_any_pick_order_start_or_direction() {
    let base = region(&frame_selection(), MaterialSide::Inside, &stock(), xy(), tool(2.0, true), 0.5).unwrap();
    let mut hole = sq(7.0, 7.0, 6.0);
    hole.rotate_left(2);
    let mut outer = sq(0.0, 0.0, 20.0);
    outer.reverse();
    outer.rotate_left(1);
    let picks = [
        Selection::new(vec![Chain::closed(hole.clone()).unwrap(), Chain::closed(outer.clone()).unwrap()]).unwrap(),
        Selection::face(outer.clone(), vec![hole.clone()]).unwrap(),
        Selection::new(vec![Chain::closed(sq(7.0, 7.0, 6.0)).unwrap(), Chain::closed(sq(0.0, 0.0, 20.0)).unwrap()]).unwrap(),
    ];
    for sel in &picks {
        for _ in 0..2 {
            let r = region(sel, MaterialSide::Inside, &stock(), xy(), tool(2.0, true), 0.5).unwrap();
            let (a, b) = (planar(&base), planar(&r));
            assert!(apart(a.center(), b.center()) < 1e-6, "one selection, one region");
            assert_eq!((a.part_walls().len(), a.stock_edge().len()), (b.part_walls().len(), b.stock_edge().len()));
        }
    }
    // The pocket round the boss: (20 - 5)^2 less the boss grown by 2.5 mm.
    let c = planar(&base);
    assert_eq!((c.part_walls().len(), c.stock_edge().len()), (2, 0));
    let boss = 11.0 * 11.0 - (4.0 - std::f64::consts::PI) * 2.5 * 2.5;
    assert!((c.center().area() - (15.0 * 15.0 - boss)).abs() < 0.2, "area {}", c.center().area());
}

#[test]
fn inside_and_outside_change_only_the_material_side() {
    let sel = Selection::points(sq(0.0, 0.0, 20.0), true).unwrap();
    let t = tool(3.0, true);
    let inside = region(&sel, MaterialSide::Inside, &stock(), xy(), t, 0.0).unwrap();
    let outside = region(&sel, MaterialSide::Outside, &stock(), xy(), t, 0.0).unwrap();
    let (i, o) = (planar(&inside), planar(&outside));
    assert_eq!((i.material(), o.material()), (MaterialSide::Inside, MaterialSide::Outside));
    assert_eq!((i.tool(), i.plane(), i.keep_out()), (o.tool(), o.plane(), o.keep_out()));
    // The two centres keep the same distance from the selection on its two
    // sides, so they never meet.
    assert!(i.center().intersect(o.center()).unwrap().area().abs() < 1e-9);
    assert!((i.center().area() - 14.0 * 14.0).abs() < 1e-6);
    let grown = 26.0 * 26.0 - (4.0 - std::f64::consts::PI) * 9.0;
    assert!((o.center().area() - (3600.0 - grown)).abs() < 0.2, "area {}", o.center().area());
    // Inside is a closed area; Outside reaches the stock outline and has
    // the air band outside it.
    assert_eq!((i.part_walls().len(), i.stock_edge().len(), i.air().is_empty()), (1, 0, true));
    assert_eq!((o.part_walls().len(), o.stock_edge().len()), (1, 1));
    assert!(o.air().area() > 0.0 && !o.air().contains(p(0.0, 0.0)) && o.air().contains(p(-24.5, 10.0)));
    assert!(o.air().intersect(o.center()).unwrap().is_empty());
}

#[test]
fn a_region_is_planned_in_the_plane_of_its_recipe_not_in_xy() {
    // The lathe's Z-X plane: the tool side along -Y, the first axis along Z.
    let zx = WorkPlane::planar([0.0, 0.0, 0.0], [0.0, -1.0, 0.0], [0.0, 0.0, 1.0]).unwrap();
    let a = region(&frame_selection(), MaterialSide::Inside, &stock(), xy(), tool(2.0, true), 0.0).unwrap();
    let b = region(&frame_selection(), MaterialSide::Inside, &stock(), zx, tool(2.0, true), 0.0).unwrap();
    assert_eq!(planar(&b).plane(), zx);
    assert!(apart(planar(&a).center(), planar(&b).center()) < 1e-9);
}

#[test]
fn a_keep_out_is_a_part_wall_the_centre_keeps_one_radius_from() {
    let clamp = square_region(30.0, 0.0, 5.0);
    let b = Bounds::new(Some(square_region(-20.0, -20.0, 60.0)), clamp.clone(), 0.0, -5.0).unwrap();
    let sel = Selection::points(sq(0.0, 0.0, 20.0), true).unwrap();
    let r = region(&sel, MaterialSide::Outside, &b, xy(), tool(2.0, true), 0.0).unwrap();
    let o = planar(&r);
    assert_eq!((o.part_walls().len(), o.stock_edge().len()), (2, 1));
    assert!(!o.center().contains(p(29.0, 2.5)) && o.center().contains(p(26.5, 2.5)));
    assert_eq!(o.keep_out(), &clamp);
}

#[test]
fn a_degenerate_selection_is_refused_by_name() {
    let nan = f64::NAN;
    assert_eq!(refused(Selection::new(vec![])), EMPTY_SELECTION);
    assert_eq!(refused(Chain::open(vec![p(0.0, 0.0)])), SHORT_CHAIN);
    assert_eq!(refused(Chain::closed(vec![p(0.0, 0.0), p(1.0, 0.0)])), SHORT_CHAIN);
    assert_eq!(refused(Chain::closed(vec![p(0.0, 0.0), p(nan, 0.0), p(1.0, 1.0)])), SELECTION_NOT_FINITE);
    assert_eq!(refused(Chain::open(vec![p(0.0, 0.0), p(0.0, 0.0), p(1.0, 1.0)])), REPEATED_POINT);
    assert_eq!(refused(Chain::closed(vec![p(0.0, 0.0), p(1.0, 0.0), p(2.0, 0.0)])), ZERO_AREA);
    let bow_tie = vec![p(0.0, 0.0), p(10.0, 10.0), p(10.0, 0.0), p(0.0, 10.0)];
    assert_eq!(refused(Chain::closed(bow_tie)), SELF_CROSSING);
    assert_eq!(refused(Chain::open(vec![p(0.0, 0.0), p(10.0, 0.0), p(5.0, 0.0)])), SELF_CROSSING);
    let crossing = vec![Chain::closed(sq(0.0, 0.0, 10.0)).unwrap(), Chain::closed(sq(5.0, 5.0, 10.0)).unwrap()];
    assert_eq!(refused(Selection::new(crossing)), CHAINS_CROSS);
    let touching = vec![Chain::closed(sq(0.0, 0.0, 10.0)).unwrap(), Chain::closed(sq(10.0, 0.0, 10.0)).unwrap()];
    assert_eq!(refused(Selection::new(touching)), CHAINS_CROSS);
}

#[test]
fn a_region_that_cannot_hold_material_is_refused_by_name() {
    let sel = Selection::points(sq(0.0, 0.0, 20.0), true).unwrap();
    let t = tool(2.0, true);
    let run = Selection::points(vec![p(0.0, 0.0), p(10.0, 0.0), p(10.0, 5.0)], false).unwrap();
    assert_eq!(refused(region(&run, MaterialSide::Inside, &stock(), xy(), t, 0.0)), OPEN_NEEDS_SIDE);
    let no_stock = Bounds::new(None, Region::empty(), 0.0, -5.0).unwrap();
    assert_eq!(refused(region(&sel, MaterialSide::Outside, &no_stock, xy(), t, 0.0)), NEEDS_STOCK);
    let far = Selection::points(sq(100.0, 100.0, 5.0), true).unwrap();
    for side in [MaterialSide::Inside, MaterialSide::Outside] {
        assert_eq!(refused(region(&far, side, &stock(), xy(), t, 0.0)), OFF_STOCK);
    }
    let all = Selection::points(sq(-30.0, -30.0, 80.0), true).unwrap();
    assert_eq!(refused(region(&all, MaterialSide::Outside, &stock(), xy(), t, 0.0)), EMPTY_MATERIAL);
    assert_eq!(refused(region(&sel, MaterialSide::Inside, &stock(), xy(), tool(10.5, true), 0.0)), EMPTY_CENTER);
    for leave in [f64::NAN, f64::INFINITY, -2.0, -3.0] {
        assert_eq!(refused(region(&sel, MaterialSide::Inside, &stock(), xy(), t, leave)), BAD_LEAVE);
    }
    let fenced = Bounds::new(Some(square_region(-20.0, -20.0, 60.0)), square_region(-5.0, -5.0, 30.0), 0.0, -5.0);
    assert_eq!(refused(region(&sel, MaterialSide::Inside, &fenced.unwrap(), xy(), t, 0.0)), EMPTY_CENTER);
}

#[test]
fn the_edges_must_partition_the_boundary_of_the_centre() {
    let c = Region::new(vec![square(0.0, 0.0, 20.0), square(5.0, 5.0, 5.0)]).unwrap();
    let outer = c.loops().iter().find(|l| l.is_ccw()).unwrap().clone();
    let hole = c.loops().iter().find(|l| !l.is_ccw()).unwrap().clone();
    let mk = |walls: Vec<_>, edge: Vec<_>| {
        PlanarRegion::new(xy(), c.clone(), MaterialSide::Inside, walls, edge, Region::empty(), Region::empty(), tool(1.0, true))
    };
    assert!(mk(vec![hole.clone()], vec![outer.clone()]).is_ok());
    assert_eq!(refused(mk(vec![hole.clone()], vec![])), NOT_A_PARTITION);
    assert_eq!(refused(mk(vec![hole.clone(), hole.clone()], vec![outer.clone()])), NOT_A_PARTITION);
    assert_eq!(refused(mk(vec![hole.clone()], vec![square(0.0, 0.0, 21.0)])), NOT_A_PARTITION);
    // A loop of the centre in the other direction is not that loop.
    let flipped = camcraft_pathkernel::Loop::new(hole.points().iter().rev().copied().collect()).unwrap();
    assert_eq!(refused(mk(vec![flipped], vec![outer.clone()])), NOT_A_PARTITION);
    // Each region that `region` builds is oriented: outer CCW, hole CW.
    let r = region(&frame_selection(), MaterialSide::Inside, &stock(), xy(), tool(1.0, true), 0.0).unwrap();
    let ccw: Vec<bool> = planar(&r).center().loops().iter().map(|l| l.is_ccw()).collect();
    assert_eq!((ccw.iter().filter(|c| **c).count(), ccw.len()), (1, 2));
}
