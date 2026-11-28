//! #4496: a holder at another gauge length is the built-in outline only
//! when that outline is the holder's own with its body section longer or
//! shorter; a look-alike with a wider hidden body, or a wider visible
//! body, is refused by name. Each library holder that shows its body
//! edits as before.

use super::*;
use crate::component::ADAPTIVE_PORT;
use crate::holder_library::{find, library, Clamp, HolderShape, ShapeClamp, BT40};

fn zr(g: &ComponentGeometry) -> Vec<(f64, f64)> {
    g.body().map(|b| b.profile().points().iter().map(|p| (p.z, p.r)).collect()).unwrap_or_default()
}

/// The outline's radius at `h` mm below the gauge line (the larger one at
/// a step); 0 outside the outline. `top` is the gauge line's height.
fn radius_below_gauge(pts: &[(f64, f64)], top: f64, h: f64) -> f64 {
    let z = top - h;
    pts.windows(2)
        .filter(|w| w[0].0.min(w[1].0) <= z + 1e-9 && z <= w[0].0.max(w[1].0) + 1e-9)
        .map(|w| {
            if (w[1].0 - w[0].0).abs() < 1e-12 {
                w[0].1.max(w[1].1)
            } else {
                w[0].1 + (w[1].1 - w[0].1) * (z - w[0].0) / (w[1].0 - w[0].0)
            }
        })
        .fold(0.0, f64::max)
}

/// The new outline is nowhere smaller than the old one, at each height
/// below the gauge line that the old outline has.
fn never_smaller(old: &[(f64, f64)], old_g: f64, new: &[(f64, f64)], new_g: f64) -> bool {
    let heights = old.iter().chain(new).flat_map(|p| [old_g - p.0, new_g - p.0]);
    heights.filter(|h| (0.0..=old_g).contains(h)).all(|h| {
        // Just inside each step, both sides.
        [h - 1e-6, h, h + 1e-6].into_iter().filter(|h| (1e-6..=old_g - 1e-6).contains(h)).all(|h| {
            radius_below_gauge(new, new_g, h) + 1e-9 >= radius_below_gauge(old, old_g, h)
        })
    })
}

/// The look-alike of #4455 with a library name: a Ø50 shrink body under
/// the flange envelope draws the outline of "BT40 Shrink 20 mm". A longer
/// gauge length must not draw the library's Ø42 body: it is refused.
#[test]
fn a_look_alike_with_a_wider_hidden_body_is_refused_by_name() {
    let lib = find("BT40 Shrink 20 mm").expect("BT40 Shrink 20 mm");
    let g0 = lib.gauge_len;
    let look_alike = HolderShape { taper: BT40, clamp: ShapeClamp::Shrink, bore: 20.0, gauge_len: g0, nose_d: 33.0, largest_d: 50.0 };
    let old = look_alike.geometry(ADAPTIVE_PORT).expect("draws: the envelope covers the body");
    let copy = ComponentGeometry::holder(&lib, ADAPTIVE_PORT, g0).expect("draws");
    assert_eq!(zr(&old), zr(&copy), "the look-alike draws the library chuck's outline");
    let library_body = ComponentGeometry::holder(&lib, ADAPTIVE_PORT, g0 + 40.0).expect("draws");
    let real = HolderShape { gauge_len: g0 + 40.0, ..look_alike }.geometry(ADAPTIVE_PORT).expect("draws");
    assert!(
        !never_smaller(&zr(&real), g0 + 40.0, &zr(&library_body), g0 + 40.0),
        "the defect: the library body at the new length is smaller than the Ø50 steel"
    );
    for d in [10.0, 40.0, -5.0] {
        let e = old.regauged_holder(&lib, ADAPTIVE_PORT, g0 + d).expect_err("refused");
        assert_eq!(e, RegaugeError::NoSection { name: "BT40 Shrink 20 mm".into() });
        assert!(e.to_string().contains("the flange envelope covers the body"), "{e}");
        // The library's own copy draws the same outline: it is refused too.
        assert_eq!(copy.regauged_holder(&lib, ADAPTIVE_PORT, g0 + d), Err(e));
    }
}

/// A user holder with a library name whose body is wider than the
/// library's, and shows: its outline is not the library's, so the library
/// body is never drawn for it.
#[test]
fn a_wider_visible_body_under_a_library_name_is_refused_by_name() {
    let lib = find("BT40 Shrink 25 mm").expect("BT40 Shrink 25 mm");
    let Clamp::Shrink { bore, nose_d, body_d } = lib.clamp else { panic!("a shrink chuck") };
    let g0 = lib.gauge_len + 30.0;
    let user = HolderShape { taper: BT40, clamp: ShapeClamp::Shrink, bore, gauge_len: g0, nose_d, largest_d: body_d + 6.0 };
    let old = user.geometry(ADAPTIVE_PORT).expect("draws");
    let e = old.regauged_holder(&lib, ADAPTIVE_PORT, g0 + 10.0).expect_err("refused");
    assert_eq!(e, RegaugeError::NotLibrary { name: "BT40 Shrink 25 mm".into(), gauge_len: g0 });
    assert!(e.to_string().contains("is not the built-in holder 'BT40 Shrink 25 mm'"), "{e}");
}

/// Each library holder that shows its body takes a longer gauge length as
/// before (the library outline at the new length, never smaller than the
/// old outline at any height below the gauge line), and a shorter one
/// inside its body section. Only a shrink chuck whose body the flange
/// envelope covers is refused.
#[test]
fn each_library_holder_that_shows_its_body_edits_as_before() {
    let (mut kept, mut refused) = (0, 0);
    for e in library() {
        let g0 = e.gauge_len;
        let old = ComponentGeometry::holder(&e, ADAPTIVE_PORT, g0).expect("draws");
        let covered = match e.clamp {
            Clamp::Shrink { bore, nose_d, body_d } => e.taper.is_some_and(|taper| {
                let wider = HolderShape { taper, clamp: ShapeClamp::Shrink, bore, gauge_len: g0, nose_d, largest_d: body_d + 1.0 };
                wider.geometry(ADAPTIVE_PORT).is_ok_and(|w| zr(&w) == zr(&old))
            }),
            _ => false,
        };
        match old.regauged_holder(&e, ADAPTIVE_PORT, g0 + 10.0) {
            Ok(new) => {
                assert!(!covered, "{}: a covered body is never lengthened", e.name());
                assert_eq!(new, ComponentGeometry::holder(&e, ADAPTIVE_PORT, g0 + 10.0).expect("draws"), "{}", e.name());
                assert!(never_smaller(&zr(&old), g0, &zr(&new), g0 + 10.0), "{}: smaller than before", e.name());
                kept += 1;
            }
            Err(err) => {
                assert!(covered, "{}: refused although its body shows: {err}", e.name());
                assert_eq!(err, RegaugeError::NoSection { name: e.name() });
                refused += 1;
            }
        }
        // A shorter gauge length inside the body section.
        let longer = ComponentGeometry::holder(&e, ADAPTIVE_PORT, g0 + 10.0).expect("draws");
        if !covered {
            let back = longer.regauged_holder(&e, ADAPTIVE_PORT, g0).expect("shortens inside the body section");
            assert_eq!(back, old, "{}", e.name());
        }
    }
    assert!(kept > 100 && refused > 0, "kept {kept}, refused {refused}");
}

/// A holder with no gauge length or no body is refused by name.
#[test]
fn a_holder_without_a_port_or_a_body_is_refused() {
    let lib = find("BT40 ER32").expect("BT40 ER32");
    let no_port = ComponentGeometry::default();
    assert_eq!(no_port.regauged_holder(&lib, ADAPTIVE_PORT, 80.0), Err(RegaugeError::NoGauge));
    let no_body = ComponentGeometry::axial(ADAPTIVE_PORT, 70.0, None).expect("a port");
    assert_eq!(no_body.regauged_holder(&lib, ADAPTIVE_PORT, 80.0), Err(RegaugeError::NoBody));
    let old = ComponentGeometry::holder(&lib, ADAPTIVE_PORT, lib.gauge_len).expect("draws");
    assert!(matches!(old.regauged_holder(&lib, ADAPTIVE_PORT, 1e6), Err(RegaugeError::Library(_))));
}
