//! #4338: a user holder's shape reads back from each built-in holder
//! component, draws an outline never smaller than the steel it states,
//! reads back from its own outline, and refuses by name a value that
//! cannot describe steel.

use super::super::{library, Clamp, ER32, BT30, BT40, CAT40, SHRINK_TAPER};
use super::*;
use crate::component::ADAPTIVE_PORT;
use crate::component_geometry::ComponentBody;
use crate::spun::SpunPoint;

fn zr(g: &ComponentGeometry) -> Vec<(f64, f64)> {
    g.body().expect("a body").profile().points().iter().map(|p| (p.z, p.r)).collect()
}

fn close(a: &[(f64, f64)], b: &[(f64, f64)]) -> bool {
    let pts = |v: &[(f64, f64)]| settled(&v.iter().map(|&(z, r)| SpunPoint::line(z, r)).collect::<Vec<_>>());
    let (a, b) = (pts(a), pts(b));
    a.len() == b.len() && a.iter().zip(&b).all(|(p, q)| (p.0 - q.0).abs() < 1e-5 && (p.1 - q.1).abs() < 1e-5)
}

/// The outline's radius at height `z` (the larger one at a step).
fn radius_at(pts: &[(f64, f64)], z: f64) -> f64 {
    pts.windows(2)
        .filter(|w| w[0].0 <= z + 1e-9 && z <= w[1].0 + 1e-9)
        .map(|w| {
            if (w[1].0 - w[0].0).abs() < 1e-12 {
                w[0].1.max(w[1].1)
            } else {
                w[0].1 + (w[1].1 - w[0].1) * (z - w[0].0) / (w[1].0 - w[0].0)
            }
        })
        .fold(0.0, f64::max)
}

fn er32(taper: Taper, gauge_len: f64, nose_d: f64) -> HolderShape {
    HolderShape { taper, clamp: ShapeClamp::Er(ER32), bore: 0.0, gauge_len, nose_d, largest_d: nose_d }
}

#[test]
fn each_builtin_taper_holder_reads_back_as_its_shape() {
    let mut n = 0;
    for e in library() {
        let Some(taper) = e.taper else { continue };
        let csw = match e.clamp {
            Clamp::Er(er) => ConnectionCode::parse(er.name),
            Clamp::Weldon { bore, .. } => ConnectionCode::parse(&format!("WELDON {bore}")),
            Clamp::Shrink { bore, .. } => ConnectionCode::parse(&format!("CYL {bore}")),
        };
        let (Ok(mcs), Ok(csw)) = (ConnectionCode::parse(taper.name), csw) else { continue };
        let kind = ComponentKind::AdaptiveItem { mcs, csw };
        let g = ComponentGeometry::holder(&e, ADAPTIVE_PORT, e.gauge_len).expect("the library draws it");
        let s = HolderShape::of(&kind, &g, ADAPTIVE_PORT).unwrap_or_else(|| panic!("{} reads back", e.name()));
        assert_eq!(s.kind().expect("codes"), kind, "{}", e.name());
        assert!(close(&zr(&s.geometry(ADAPTIVE_PORT).expect("draws")), &zr(&g)), "{}: the same outline", e.name());
        assert!((s.gauge_len - e.gauge_len).abs() < 1e-9);
        n += 1;
    }
    assert!(n > 50, "the built-in taper holders read back ({n})");
}

#[test]
fn an_edited_outline_is_never_smaller_than_the_steel() {
    let mut shapes = Vec::new();
    for taper in [BT30, BT40, CAT40] {
        for gauge_len in [70.0, 95.0, 140.0] {
            for nose_d in [36.0, 50.0, 58.0, 72.0] {
                shapes.push(er32(taper, gauge_len, nose_d));
                shapes.push(HolderShape { clamp: ShapeClamp::Weldon, bore: 12.0, ..er32(taper, gauge_len, nose_d * 0.6) });
                for largest_d in [nose_d * 0.5, nose_d * 0.8, nose_d] {
                    shapes.push(HolderShape {
                        clamp: ShapeClamp::Shrink,
                        bore: 12.0,
                        nose_d: largest_d.min(nose_d * 0.5),
                        largest_d,
                        ..er32(taper, gauge_len, 0.0)
                    });
                }
            }
        }
    }
    let mut checked = 0;
    for s in shapes {
        let Ok(g) = s.geometry(ADAPTIVE_PORT) else { continue };
        let pts = zr(&g);
        let front = s.gauge_len - s.taper.front_face();
        let steps = 200;
        for i in 0..=steps {
            let z = front * f64::from(i) / f64::from(steps);
            let steel = match s.clamp {
                // An ER or Weldon body is its nose steel up to the flange.
                ShapeClamp::Er(_) | ShapeClamp::Weldon => s.nose_d * 0.5,
                // A shrink nose widens at the outer taper to its body.
                ShapeClamp::Shrink => (s.nose_d * 0.5 + z * SHRINK_TAPER).min(s.largest_d * 0.5),
            };
            assert!(radius_at(&pts, z) >= steel - 1e-9, "{s:?}: radius {} < steel {steel} at z {z}", radius_at(&pts, z));
        }
        for i in 0..=20 {
            let z = front + (s.taper.flange_t) * f64::from(i) / 20.0;
            assert!(radius_at(&pts, z) >= s.taper.flange_d * 0.5 - 1e-9, "{s:?}: the flange at z {z}");
        }
        // The form reads back what it wrote.
        let kind = s.kind().expect("codes");
        let back = HolderShape::of(&kind, &g, ADAPTIVE_PORT).unwrap_or_else(|| panic!("{s:?} reads back: {pts:?}"));
        assert!(close(&zr(&back.geometry(ADAPTIVE_PORT).expect("draws")), &pts), "{s:?} reads back as {back:?}");
        checked += 1;
    }
    assert!(checked > 60, "shapes checked: {checked}");
}

#[test]
fn a_value_that_cannot_be_steel_is_refused_by_name() {
    let ok = er32(BT40, 70.0, 50.0);
    assert!(ok.entry().is_ok());
    let refused = |s: HolderShape| s.geometry(ADAPTIVE_PORT).expect_err("refused").to_string();
    assert!(refused(er32(BT40, 70.0, 20.0)).contains("nose diameter Ø20 mm must be larger than the Ø20 mm"));
    assert!(refused(er32(BT40, f64::NAN, 50.0)).contains("gauge length must be a positive number"));
    assert!(refused(HolderShape { largest_d: 60.0, ..ok }).contains("one body diameter"));
    assert!(refused(er32(BT40, 20.0, 50.0)).contains("gauge length is shorter than the flange and nose need"));
    let weldon = HolderShape { clamp: ShapeClamp::Weldon, bore: 16.0, nose_d: 16.0, largest_d: 16.0, ..ok };
    assert!(refused(weldon).contains("larger than the Ø16 mm it clamps"));
    let shrink = HolderShape { clamp: ShapeClamp::Shrink, bore: 12.0, nose_d: 24.0, largest_d: 20.0, ..ok };
    assert!(refused(shrink).contains("a shrink-fit body widens from its nose"));
    assert!(refused(HolderShape { bore: 500.0, nose_d: 600.0, largest_d: 600.0, ..shrink }).contains("larger than Ø200 mm"));
}

#[test]
fn an_outline_from_elsewhere_is_not_a_form_shape() {
    let kind = er32(BT40, 70.0, 50.0).kind().expect("codes");
    let p = SpunPoint::line;
    let body = ComponentBody::new(vec![p(0.0, 0.0), p(0.0, 30.0), p(70.0, 30.0), p(70.0, 0.0)], -70.0).expect("a body");
    let g = ComponentGeometry::axial(ADAPTIVE_PORT, 70.0, Some(body)).expect("geometry");
    assert_eq!(HolderShape::of(&kind, &g, ADAPTIVE_PORT), None);
    assert!(tapers().iter().any(|t| t.name == "BT40"));
}
