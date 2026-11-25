//! Unit tests of the undercut cutter record law (#1444): each kind's
//! sound numbers pass, each broken number is refused by name, and the
//! drawn outline is the head the laws read.
//!
//! Gate map: `camcraft_tools::undercut`.

use super::*;
use crate::cutter_profile::{check, parametric};

fn tslot() -> Tool {
    cutter(ToolKind::TSlot, 20.0, 6.0, 8.0, 30.0, 0.0)
}

fn dovetail() -> Tool {
    cutter(ToolKind::Dovetail, 20.0, 5.0, 8.0, 30.0, 60.0)
}

fn lollipop() -> Tool {
    cutter(ToolKind::Lollipop, 10.0, 5.0, 4.0, 30.0, 0.0)
}

#[test]
fn each_undercut_kind_is_a_sane_tool_with_a_head_wider_than_its_neck() {
    for t in [tslot(), cutter(ToolKind::Keyseat, 16.0, 3.0, 6.0, 25.0, 0.0), dovetail(), lollipop()] {
        assert_eq!(t.why_not_sane(), None, "{:?}", t.kind);
        let u = Undercut::of(&t).unwrap();
        assert!(u.overhang() > 0.0, "{:?}", t.kind);
        assert!((u.contact().0 - t.diameter * 0.5).abs() < 1e-12, "{:?}", t.kind);
    }
    assert_eq!(Undercut::of(&crate::tools::probe_tool(2.0, 20.0)), None);
}

#[test]
fn a_neck_as_wide_as_the_head_is_refused_by_name() {
    let t = Tool { shank_d: 20.0, ..tslot() };
    let why = t.why_not_sane().unwrap();
    assert!(why.contains("T-slot cutter") && why.contains("neck") && why.contains("under its 20 mm head"), "{why}");
    assert_eq!(Undercut::of(&t), None);
}

#[test]
fn a_dovetail_whose_flank_cuts_under_its_neck_is_refused_by_name() {
    // 30 degrees over 5 mm takes 8.66 mm off the radius: 10 -> 1.34 < 4.
    let why = Tool { tip_angle: 30.0, ..dovetail() }.why_not_sane().unwrap();
    assert!(why.contains("dovetail cutter") && why.contains("under its 8 mm neck"), "{why}");
    let why = Tool { tip_angle: 90.0, ..dovetail() }.why_not_sane().unwrap();
    assert!(why.contains("flank angle 90 degrees"), "{why}");
}

/// #3662: an inch dovetail (0.24 in head, 0.0575 in high, 45 degree
/// flank, 1/8 in neck) in mm. Its flank meets the neck exactly, and the
/// conversion puts the flank top one ulp under the neck.
#[test]
fn a_neck_that_meets_the_flank_exactly_is_accepted() {
    let (d, h, n) = (0.24 * 25.4, 0.0575 * 25.4, 0.125 * 25.4);
    let t = cutter(ToolKind::Dovetail, d, h, n, 20.0, 45.0);
    let top = d * 0.5 - h / 45f64.to_radians().tan();
    assert!(top < n * 0.5 && n * 0.5 - top < 1e-12, "the fixture is float dust under the neck: {top}");
    assert_eq!(t.why_not_sane(), None);
    assert!(Undercut::of(&t).is_some_and(|u| u.overhang() > 0.0));
    check(&t, parametric(&t)).expect("its drawn outline passes the drawn cutter law");
    // A neck wider than the flank top by a real amount is still refused.
    for wider in [1e-6, 0.002] {
        let why = Tool { shank_d: n + wider, ..t.clone() }.why_not_sane().unwrap_or_default();
        assert!(why.contains("dovetail cutter") && why.contains("neck"), "{wider}: {why}");
    }
}

#[test]
fn a_lollipop_neck_must_leave_the_ball_above_its_centre() {
    let why = Tool { flute_len: 4.0, ..lollipop() }.why_not_sane().unwrap();
    assert!(why.contains("lollipop cutter") && why.contains("ball's centre"), "{why}");
    // The neck leaves the 10 mm ball at 5 + sqrt(25 - 4) = 9.583 mm.
    assert_eq!(Tool { flute_len: 9.58, ..lollipop() }.why_not_sane(), None);
    assert!(Tool { flute_len: 9.6, ..lollipop() }.why_not_sane().is_some());
}

#[test]
fn a_lollipop_stack_that_ends_inside_the_ball_is_refused_by_name() {
    // The review's 2 mm neck ends at Z 7. Even a neck reaching the
    // ball/neck junction (Z 9.583) still omits the cap up to Z 10.
    for neck_len in [2.0, 4.6, 5.0 - 1e-10] {
        let t = cutter(ToolKind::Lollipop, 10.0, 5.0, 4.0, neck_len, 0.0);
        let why = t.why_not_sane().unwrap();
        assert!(why.contains("lollipop cutter") && why.contains("neck length") && why.contains("10 mm ball top"), "{why}");
        assert_eq!(Undercut::of(&t), None, "a refused stack has no analytic head");
    }
}

#[test]
fn the_head_radius_follows_the_profile_not_the_tip() {
    let d = Undercut::of(&dovetail()).unwrap();
    assert!((d.rho(0.0) - 10.0).abs() < 1e-12);
    // 60 degrees: the radius falls by 5 / tan(60) = 2.887 over the head.
    assert!((d.rho(5.0) - (10.0 - 5.0 / 60f64.to_radians().tan())).abs() < 1e-12);
    assert_eq!(d.contact(), (10.0, 0.0));
    let b = Undercut::of(&lollipop()).unwrap();
    assert_eq!(b.contact(), (5.0, 5.0));
    assert!(b.rho(0.0).abs() < 1e-12 && (b.rho(5.0) - 5.0).abs() < 1e-12);
    assert!((b.head_top() - 10.0).abs() < 1e-12, "the whole ball is the head");
    assert_eq!(b.rho(10.5), 0.0);
}

#[test]
fn the_drawn_outline_is_the_head_then_the_neck() {
    assert_eq!(tslot().profile(), vec![(0.0, 0.0), (0.0, 10.0), (6.0, 10.0), (6.0, 4.0), (36.0, 4.0), (36.0, 0.0)]);
    let top = 10.0 - 5.0 / 60f64.to_radians().tan();
    assert_eq!(dovetail().profile(), vec![(0.0, 0.0), (0.0, 10.0), (5.0, top), (5.0, 4.0), (35.0, 4.0), (35.0, 0.0)]);
    let ball = lollipop().profile();
    assert!(ball.iter().any(|&(z, r)| z == 5.0 && (r - 5.0).abs() < 1e-12), "the equator is drawn: {ball:?}");
    assert!(ball.iter().all(|&(z, r)| z.is_finite() && r.is_finite() && r <= 5.0 + 1e-12));
}

/// The widest drawn radius at height `z` on the outline `p`.
fn drawn_at(p: &[(f64, f64)], z: f64) -> f64 {
    p.windows(2)
        .filter(|w| w[0].0.min(w[1].0) <= z && z <= w[0].0.max(w[1].0))
        .map(|w| {
            let (a, b) = (w[0], w[1]);
            if (b.0 - a.0).abs() < 1e-12 { a.1.max(b.1) } else { a.1 + (b.1 - a.1) * (z - a.0) / (b.0 - a.0) }
        })
        .fold(0.0, f64::max)
}

#[test]
fn the_drawn_lollipop_is_the_whole_ball_united_with_the_neck() {
    // The 10 mm ball on a 4 mm neck that starts at its equator (Z 5).
    // At Z 8 the ball is sqrt(25 - 9) = 4 mm wide, not the 2 mm neck.
    for neck_len in [5.0, 30.0] {
        let t = Tool { shank_len: neck_len, ..lollipop() };
        assert_eq!(t.why_not_sane(), None, "the exact ball-top boundary is valid");
        let p = t.profile();
        assert!((drawn_at(&p, 8.0) - 4.0).abs() < 0.05, "Z 8 draws {}: {p:?}", drawn_at(&p, 8.0));
        // Above the equator up to the neck top: never narrower than the
        // ball the verifier reads, or the neck. The chords may stand a
        // little inside the circle.
        let u = Undercut::of(&t).unwrap();
        for k in 0..=200 {
            let z = 5.0 + neck_len * f64::from(k) / 200.0;
            let want = u.rho(z).max(2.0);
            assert!(drawn_at(&p, z) >= want - 0.05, "Z {z}: drawn {} < {want}", drawn_at(&p, z));
        }
        // The outline climbs: no point goes back down in Z.
        assert!(p.windows(2).all(|w| w[1].0 >= w[0].0), "{p:?}");
        check(&t, parametric(&t)).unwrap();
    }
}

#[test]
fn a_lollipop_drawn_cutter_still_ends_at_the_stated_stack_top() {
    let t = Tool { shank_len: 5.0, ..lollipop() };
    let points = parametric(&t);
    check(&t, points.clone()).unwrap();
    // A number edit cannot keep a longer drawing to conceal a short
    // stack. Nor can the drawing end short of a valid stated stack.
    for neck_len in [4.9, 5.1] {
        let edited = Tool { shank_len: neck_len, ..t.clone() };
        assert!(matches!(check(&edited, points.clone()), Err(crate::cutter_profile::CutterError::Length { .. })));
    }
}

#[test]
fn each_outline_passes_the_drawn_cutter_law() {
    for t in [tslot(), dovetail(), lollipop()] {
        let s = check(&t, parametric(&t)).unwrap_or_else(|e| panic!("{:?}: {e}", t.kind));
        assert!((s.max_radius() - t.diameter * 0.5).abs() < 1e-9, "{:?}", t.kind);
    }
}

#[test]
fn the_fixture_stack_carries_the_whole_ball_and_keeps_the_neck_top() {
    // The 10 mm ball's neck leaves it at 5 mm; its pole is at 10 mm.
    let g = fixture_geom(&lollipop());
    let plain = lollipop().geom();
    assert_eq!(g.radius, 5.0);
    assert_eq!(g.flute_len, 10.0, "the head band reaches the ball's pole");
    assert_eq!(g.flute_len + g.shank_len, plain.flute_len + plain.shank_len, "the neck top and the holder stay");
    assert_eq!(g.shank_radius, plain.shank_radius);
    // At the shortest accepted stack the ball fills the whole band.
    let shortest = Tool { shank_len: 5.0, ..lollipop() };
    assert_eq!(shortest.why_not_sane(), None);
    assert_eq!(fixture_geom(&shortest).shank_len, 0.0);
    assert_eq!(fixture_geom(&shortest).flute_len, 10.0);
    // The disc and frustum heads end at the neck start: unchanged.
    assert_eq!(fixture_geom(&tslot()), tslot().geom());
    assert_eq!(fixture_geom(&dovetail()), dovetail().geom());
}

#[test]
fn a_rounded_dovetail_reads_its_corner_radius_and_widened_flank() {
    // Issue #4004 example: Ø19.05 mm, 30 deg off axis (tip_angle 60),
    // head height 9.525 mm, corner_r 0.254 mm and 3.0 mm.
    let base = cutter(ToolKind::Dovetail, 19.05, 9.525, 8.0, 30.0, 60.0);
    let sharp = Undercut::of(&base).unwrap();
    let t = 30.0_f64.to_radians();
    let sharp_top = 9.525 - 9.525 * t.tan();
    assert!((sharp.rho(9.525) - sharp_top).abs() < 1e-12);
    assert!((sharp_top - 4.025738686).abs() < 1e-6);

    let r254 = Tool { corner_r: 0.254, ..base.clone() };
    assert_eq!(r254.why_not_sane(), None);
    let u254 = Undercut::of(&r254).unwrap();
    assert_eq!(u254.contact(), (9.525, 0.254));
    assert!((u254.rho(0.0) - (9.525 - 0.254)).abs() < 1e-12);
    assert!((u254.rho(0.254) - 9.525).abs() < 1e-12);
    let s_tan = 0.254 * (1.0 + t.sin());
    let arc_tan = 9.525 - 0.254 + (0.254 * 0.254 - (s_tan - 0.254).powi(2)).sqrt();
    let flank_tan = 9.525 - 0.254 + 0.254 * (1.0 + t.sin()) / t.cos() - s_tan * t.tan();
    assert!((arc_tan - flank_tan).abs() < 1e-12);
    assert!((u254.rho(s_tan) - flank_tan).abs() < 1e-12);
    let want_top_254 = 9.525 - 0.254 + 0.254 * (1.0 + t.sin()) / t.cos() - 9.525 * t.tan();
    assert!((u254.rho(9.525) - want_top_254).abs() < 1e-12);
    assert!((want_top_254 - 4.21167959).abs() < 1e-6);
    assert!(((u254.rho(9.525) - sharp.rho(9.525)) - 0.1859409).abs() < 1e-6);

    let r3 = Tool { corner_r: 3.0, ..base.clone() };
    let u3 = Undercut::of(&r3).unwrap();
    assert_eq!(u3.contact(), (9.525, 3.0));
    let want_top_3 = 9.525 - 3.0 + 3.0 * (1.0 + t.sin()) / t.cos() - 9.525 * t.tan();
    assert!((u3.rho(9.525) - want_top_3).abs() < 1e-12);
    assert!(((u3.rho(9.525) - sharp.rho(9.525)) - 2.1961524).abs() < 1e-6);

    // Drawn profile + spun law pass for small, medium, large, and boundary radii.
    let exact_arc = Tool { corner_r: 2.0, ..cutter(ToolKind::Dovetail, 20.0, 3.0, 8.0, 20.0, 60.0) };
    let full_r = Tool { corner_r: 10.0, ..cutter(ToolKind::Dovetail, 20.0, 16.0, 4.0, 20.0, 60.0) };
    let tiny = Tool { corner_r: 0.005, ..base };
    for tool in [&r254, &r3, &exact_arc, &full_r, &tiny] {
        assert_eq!(tool.why_not_sane(), None, "{tool:?}");
        let u = Undercut::of(tool).unwrap();
        let p = tool.profile();
        assert!(p.windows(2).all(|w| w[1].0 >= w[0].0 - 1e-12), "{p:?}");
        assert!(
            p.iter().any(|&(z, rad)| (z - tool.corner_r).abs() < 1e-12 && (rad - u.r).abs() < 1e-12),
            "profile hits widest corner point ({}, {}): {p:?}",
            tool.corner_r,
            u.r
        );
        for &(z, rad) in p.iter().filter(|q| q.0 > 0.0 && q.0 < tool.flute_len) {
            assert!((rad - u.rho(z)).abs() < 1e-9, "z {z}: profile {rad} vs rho {}", u.rho(z));
        }
        assert!(
            p.iter().any(|&(z, rad)| (z - tool.flute_len).abs() < 1e-12 && (rad - u.rho(tool.flute_len)).abs() < 1e-9),
            "flute top radius matches rho({}): {p:?}",
            tool.flute_len
        );
        check(tool, parametric(tool)).unwrap_or_else(|e| panic!("corner_r {}: {e}", tool.corner_r));
    }
}

#[test]
fn a_dovetail_corner_radius_refusals_name_the_rule() {
    // 3 mm corner at 60 deg reaches 4.5 mm up the flank, over a 4 mm head.
    let tall_arc = Tool { corner_r: 3.0, ..cutter(ToolKind::Dovetail, 20.0, 4.0, 8.0, 30.0, 60.0) };
    let why = tall_arc.why_not_sane().unwrap();
    assert!(why.contains("dovetail cutter") && why.contains("corner radius"), "{why}");
    assert_eq!(Undercut::of(&tall_arc), None);

    // Direct undercut::refusal also guards negative and over-radius values.
    let neg = Tool { corner_r: -0.2, ..dovetail() };
    assert!(refusal(&neg).unwrap().contains("corner radius"));
    let over_r = Tool { corner_r: 10.5, ..dovetail() };
    assert!(refusal(&over_r).unwrap().contains("corner radius"));

    // Neck clearance checks the widened rounded flank, not the sharp cone:
    // Ø20, h=5, 60 deg -> sharp top radius 7.113, corner_r=2 -> rounded top 8.577.
    let sd15_sharp = Tool { shank_d: 15.0, ..dovetail() };
    assert!(sd15_sharp.why_not_sane().is_some());
    let sd15_round = Tool { corner_r: 2.0, shank_d: 15.0, ..dovetail() };
    assert_eq!(sd15_round.why_not_sane(), None);
    let sd175_round = Tool { corner_r: 2.0, shank_d: 17.5, ..dovetail() };
    let why = sd175_round.why_not_sane().unwrap();
    assert!(why.contains("dovetail cutter") && why.contains("neck"), "{why}");
}
