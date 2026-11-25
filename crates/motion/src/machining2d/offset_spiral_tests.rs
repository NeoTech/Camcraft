//! Pins of the Offset rings spiral (#3933).

use super::offset_spiral;
use crate::geo2d::{Loop, Point2, Region};
use crate::machining2d::adaptive::first_crossing;
use crate::machining2d::level_rings::unswept;

fn disc(rad: f64) -> Region {
    let n = 256;
    let pts = (0..n)
        .map(|i| {
            let a = std::f64::consts::TAU * i as f64 / n as f64;
            Point2::new(rad * a.cos(), rad * a.sin())
        })
        .collect();
    let Ok(l) = Loop::new(pts) else { panic!("fixture loop") };
    let Ok(r) = Region::new(vec![l]) else { panic!("fixture region") };
    r
}

/// A round area: one pass, its radius never steps back in, and from the
/// core ring out no two consecutive revolutions stand further apart than
/// the step - one spiral, no step across between rings.
#[test]
fn a_round_area_is_one_outward_spiral_at_the_load_step() {
    let center = disc(10.0);
    let Ok((passes, notes)) = offset_spiral(&center, 1.0, true, 2.0) else { panic!("plans") };
    assert_eq!(passes.len(), 1, "one spiral: {notes:?}");
    assert!(notes.is_empty(), "a round area holds the load: {notes:?}");
    let radii: Vec<f64> = passes[0].iter().map(|p| p.x.hypot(p.y)).collect();
    let jumps = radii.windows(2).filter(|w| w[1] - w[0] > 0.2).count();
    assert_eq!(jumps, 0, "no radial step across between rings");
    let last = radii.last().copied().unwrap_or(0.0);
    assert!((last - 10.0).abs() < 0.02, "ends on the wall ring: {last}");
}

fn poly(pts: &[(f64, f64)]) -> Loop {
    let Ok(l) = Loop::new(pts.iter().map(|&(x, y)| Point2::new(x, y)).collect()) else { panic!("fixture loop") };
    l
}

/// Every pass point stays in the cutter-centre area (no gouge), the
/// passes sweep the whole area, and no pass crosses itself or another.
fn sound(center: &Region, passes: &[Vec<Point2>], r: f64) {
    let Ok(grown) = center.offset(1e-3) else { panic!("offset") };
    for p in passes.iter().flatten() {
        assert!(grown.contains(*p), "{p:?} leaves the cutter-centre area");
    }
    let Ok(left) = unswept(center, passes, r) else { panic!("sweep") };
    assert!(left <= 0.1, "the spiral leaves {left} mm2 uncut");
    assert!(first_crossing(passes).is_none(), "the passes cross: {:?}", first_crossing(passes));
}

/// A rectangle: the rings' sharp corners load the tool past the optimal
/// load, and the long core ring slots - both named, never silent; the
/// walk stays in the area and cuts all of it.
#[test]
fn a_rectangle_names_its_corner_load_and_its_core_slot() {
    let Ok(center) = Region::new(vec![poly(&[(0.0, 0.0), (30.0, 0.0), (30.0, 20.0), (0.0, 20.0)])]) else {
        panic!("fixture region")
    };
    let Ok((passes, notes)) = offset_spiral(&center, 1.0, true, 2.0) else { panic!("plans") };
    assert_eq!(passes.len(), 1, "one spiral: {notes:?}");
    sound(&center, &passes, 2.0);
    assert!(notes.iter().any(|n| n.contains("worst load")), "the corner load is named: {notes:?}");
    assert!(notes.iter().any(|n| n.contains("core ring cuts")), "the core slot is named: {notes:?}");
}

/// An island splits the rings into families: each family is its own
/// spiral (or rings where no morph stays in its band), the whole stays
/// in the area and cuts all of it.
#[test]
fn an_island_area_stays_in_its_band_and_cuts_all_of_it() {
    let Ok(center) = Region::new(vec![
        poly(&[(0.0, 0.0), (30.0, 0.0), (30.0, 30.0), (0.0, 30.0)]),
        poly(&[(12.0, 12.0), (12.0, 18.0), (18.0, 18.0), (18.0, 12.0)]),
    ]) else {
        panic!("fixture region")
    };
    for climb in [true, false] {
        let Ok((passes, _)) = offset_spiral(&center, 1.0, climb, 2.0) else { panic!("plans") };
        sound(&center, &passes, 2.0);
    }
}
