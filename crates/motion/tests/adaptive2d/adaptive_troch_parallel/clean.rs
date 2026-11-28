//! Issue #3849: 2D Adaptive (Trochoidal) clean-up passes are checked against
//! `max_engagement` (including the 90 deg default), stepped down when needed,
//! and surfaced with a named diagnostic when even reduced steps exceed it.
//! A module of the `adaptive_troch_parallel` binary (no binary of its own, #3187).

use super::fixtures::*;
use camcraft_motion::cl::Toolpath;
use camcraft_motion::geo2d::Region;
use camcraft_motion::geometry::Point3;
use camcraft_motion::machining2d::{
    AdaptiveTrochParams, CLEANUP_ENGAGEMENT_NOTE_PREFIX, PocketParams, adaptive_trochoidal_with,
    cleanup_engagement_exceeded, cleanup_engagement_note,
};
use camcraft_motion::quality::{QualitySpec, measure, stock_for};

fn assert_cleanup_checked(name: &str, region: &Region, p: &PocketParams, max_engagement: f64) -> Toolpath {
    let t = AdaptiveTrochParams { max_engagement, ..AdaptiveTrochParams::default() };
    let path = adaptive_trochoidal_with(region, p, &t).unwrap_or_else(|e| panic!("{name}: {e}"));
    let spec = QualitySpec::new(p.tool_diameter);
    let mut stock = stock_for(region, p.top_z, p.bottom_z, p.tool_diameter, &spec).expect("stock");
    let q = measure(&path, &mut stock, &spec).expect("measure");
    let warned = cleanup_engagement_exceeded(&path.notes);
    if q.max_deg > max_engagement + 0.5 {
        let (measured, limit) = warned.unwrap_or_else(|| {
            panic!(
                "{name}: engagement {:.1} deg exceeds {:.1} deg without a named warning; notes={:?}",
                q.max_deg, max_engagement, path.notes
            )
        });
        assert!(measured > limit && (limit - max_engagement).abs() < 0.2, "{name}: {warned:?}");
    } else {
        assert!(warned.is_none(), "{name}: unexpected warning {warned:?} at {:.1} deg", q.max_deg);
    }
    path
}

/// The step-down note, and no over-limit note: the ladder held the limit.
fn assert_stepped_down(name: &str, path: &Toolpath, max_engagement: f64) {
    let held = format!("clean-up peel step was reduced to hold the {max_engagement:.1} deg engagement limit");
    assert!(path.notes.iter().any(|n| n.contains(&held)), "{name}: no step-down note: {:?}", path.notes);
    assert_eq!(cleanup_engagement_exceeded(&path.notes), None, "{name}: {:?}", path.notes);
}

#[test]
fn cleanup_engagement_is_checked_on_the_default_limit() {
    let p = PocketParams { bottom_z: -3.0, ..params() };
    // Default 90 deg limit on #1587 face fixture stays <= 90 deg cleanly.
    assert_cleanup_checked("face 34x16 D6 (90 deg)", &face(), &p, 90.0);
}

// #3849 step-down: at the full peel step the clean-up of wide 30x24 D6
// and of L-shape D10 reads 64.0 and 62.5 deg against a 60 deg limit;
// upstream/main refused both paths (GenError::Invalid). The peel now
// steps down, holds the limit, and the paths post.
#[test]
fn cleanup_steps_down_to_a_custom_limit_wide() {
    let p = PocketParams { bottom_z: -3.0, ..params() };
    let wide = rect(0.0, 0.0, 30.0, 24.0);
    let path = assert_cleanup_checked("wide 30x24 D6 (60 deg)", &wide, &p, 60.0);
    assert_stepped_down("wide 30x24 D6 (60 deg)", &path, 60.0);
}

#[test]
fn cleanup_steps_down_to_a_custom_limit_l_shape() {
    let p = PocketParams { bottom_z: -3.0, ..params() };
    let l_shape = Region::new(vec![poly(&[
        (0.0, 0.0), (60.0, 0.0), (60.0, 30.0), (30.0, 30.0), (30.0, 60.0), (0.0, 60.0),
    ])])
    .expect("L-shape");
    let p10 = PocketParams { tool_diameter: 10.0, ..p };
    let path = assert_cleanup_checked("L-shape D10 (60 deg)", &l_shape, &p10, 60.0);
    assert_stepped_down("L-shape D10 (60 deg)", &path, 60.0);
}

/// A 20 x 14 pocket with a 7 mm notch in its +Y wall (#3849).
fn notch() -> Region {
    Region::new(vec![poly(&[
        (-10.0, -7.0), (10.0, -7.0), (10.0, 7.0), (3.5, 7.0), (3.5, 14.0), (-3.5, 14.0), (-3.5, 7.0), (-10.0, 7.0),
    ])])
    .expect("notch")
}

/// #3849, red on upstream/main: the notch pocket, a 6 mm tool, the 90
/// deg default. The clean-up into the notch reads about 97.5 deg even at
/// the smallest peel step (118.5 deg at main's full step). Main never
/// measured the clean-up at the default limit, so the path posted with no
/// note. Now it carries the named note (the operation's warning).
#[test]
fn cleanup_over_the_default_limit_is_named() {
    let p = PocketParams { bottom_z: -3.0, ..params() };
    let path = assert_cleanup_checked("notch 20x14 D6 (90 deg)", &notch(), &p, 90.0);
    let (measured, limit) = cleanup_engagement_exceeded(&path.notes).expect("the over-limit clean-up is named");
    assert!(measured > 90.0 && (limit - 90.0).abs() < 1e-9, "{measured} / {limit}");
}

/// #3849 keeps main's refusal: under a custom limit the WHOLE path
/// (lanes, links, clean-up, every level) is measured. The notch pocket
/// that warns at the 90 deg default reads about 95.5 deg against a 75
/// deg limit: the step-down cannot hold it, so the path is refused by
/// name - never posted with an advisory warning.
#[test]
fn a_custom_limit_the_whole_path_cannot_hold_is_refused() {
    let p = PocketParams { bottom_z: -3.0, ..params() };
    let t = AdaptiveTrochParams { max_engagement: 75.0, ..AdaptiveTrochParams::default() };
    let err = match adaptive_trochoidal_with(&notch(), &p, &t) {
        Ok(path) => panic!("notch 20x14 D6 (75 deg) posted: notes={:?}", path.notes),
        Err(e) => e.to_string(),
    };
    assert!(err.contains("cleanup and lanes exceed the 75.0 deg engagement limit"), "{err}");
}

#[test]
fn cleanup_engagement_note_round_trips_for_2d_and_3d() {
    let note_2d = cleanup_engagement_note(102.5, Point3::new(1.25, -2.5, -3.0), 90.0);
    assert!(note_2d.contains("at (1.250, -2.500, -3.000)"), "{note_2d}");
    assert!(note_2d.starts_with(CLEANUP_ENGAGEMENT_NOTE_PREFIX), "{note_2d}");
    assert_eq!(cleanup_engagement_exceeded(std::slice::from_ref(&note_2d)), Some((102.5, 90.0)));

    let note_3d = note_2d.replacen("2D Adaptive (Trochoidal):", "3D Adaptive (Trochoidal):", 1);
    assert_eq!(cleanup_engagement_exceeded(&[note_3d]), Some((102.5, 90.0)));
    assert_eq!(cleanup_engagement_exceeded(&["2D Adaptive (Trochoidal): 16 corner/wall clean-up pass(es) emitted".into()]), None);
}
